"""Reproducible, bounded internal-2019 factual audit, not a production importer.

No remote catalog fetch, installs, legacy fallback or automatic retry. Requires
exact local F-03/catalog inputs. Data/evidence directory must be outside this repo.
"""
import argparse
import csv
import hashlib
import io
import json
from pathlib import Path
import re
import shutil
import subprocess
import time

from audits.bounded_ranges import RangeRelay

CATALOG_SHA = '2a3362ba0bc970515048c87c5bf5bcaab1bd093593b7bafc1e50e2ebaa241583'
SNAPSHOT_SHA = 'd036b4664c8ce4cab51e86aacc075727558f9461c813b51f53d5974fc39a4528'
SOURCE_COMMIT = 'b449f7bec49f852c27803e096303a4c78ac661b9'
BASE = 'https://data.baseball.computer/baseball/v1/bc_publish_data/'
TABLES = ('stg_events', 'stg_event_audit', 'stg_event_flags', 'stg_event_baserunners',
          'stg_event_fielding_plays', 'stg_game_fielding_appearances')
CAP, SECONDS = 1024**3, 300
ATTRIBUTION = ('The information used here was obtained free of charge from and is copyrighted by '
               'Retrosheet. Interested parties may contact Retrosheet at "www.retrosheet.org".')


def quoted(value):
    return "'" + str(value).replace("'", "''") + "'"


def extraction_sql(snapshot, groups, base_url):
    if (set(groups) != set(TABLES) or not all(groups.values())
            or not re.fullmatch(r'http://127\.0\.0\.1:[0-9]+', base_url)
            or any(not re.fullmatch(r'[0-9]+', str(key)) or size <= 0
                   for rows in groups.values() for key, size in rows)):
        raise ValueError('Exact nonempty source groups and loopback URLs required')
    sql = f"""SET autoinstall_known_extensions=FALSE;
SET autoload_known_extensions=FALSE;
LOAD httpfs;
SET http_proxy='';
SET http_retries=0;
SET http_timeout=30;
SET http_keep_alive=FALSE;
SET enable_external_file_cache=FALSE;
SET enable_http_metadata_cache=FALSE;
SET threads=2;
SET memory_limit='512MB';
ATTACH {quoted(snapshot)} AS cached (READ_ONLY);
CREATE TABLE selected_games AS SELECT * FROM cached.games;
CREATE TABLE selected_events AS SELECT * FROM cached.events;
"""
    for name in TABLES:
        urls = ','.join(quoted(base_url + '/' + str(key)) for key, _ in groups[name])
        key = 'game_id' if name in ('stg_events', 'stg_game_fielding_appearances') else 'event_key'
        selected = 'selected_games' if key == 'game_id' else 'selected_events'
        output = 'source_events' if name == 'stg_events' else name
        season = ' AND season=2019' if name == 'stg_events' else ''
        sql += (f'EXPLAIN ANALYZE CREATE TABLE {output} AS SELECT p.* FROM read_parquet([{urls}]) p '
                f'WHERE {key} IN (SELECT {key} FROM {selected}){season};\n')
        if name == 'stg_events':
            sql += """CREATE TEMP TABLE bridge_assertion AS SELECT CASE WHEN NOT EXISTS (
(SELECT * FROM source_events EXCEPT ALL SELECT * FROM selected_events)
UNION ALL (SELECT * FROM selected_events EXCEPT ALL SELECT * FROM source_events)
) THEN TRUE ELSE error('Source/cached identity bridge differs') END;
"""
    return sql + 'CHECKPOINT;\n'


def publication_check(stage, records, received, reserved, cap, elapsed):
    if (not records or any(r['status'] != 'complete' for r in records)
            or sum(r['bytes'] for r in records) != received
            or not 0 <= received <= reserved <= cap or elapsed > SECONDS
            or not 0 < stage.stat().st_size < 250*1024**2
            or Path(str(stage) + '.wal').exists()):
        raise ValueError('Partial/range/cap/runtime/file evidence cannot publish')


def digest(path):
    with path.open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def native(database, sql, directory, label, *, readonly=True, timeout=30):
    script = directory / (label + '.sql')
    script.write_text(sql)
    command = ['duckdb', '-bail', '-batch', '-init', '/dev/null', '-csv']
    if readonly:
        command.append('-readonly')
    command += [str(database), '-f', str(script)]
    with (directory/(label+'.stdout')).open('w') as out, (directory/(label+'.stderr')).open('w') as err:
        subprocess.run(command, stdout=out, stderr=err, check=True, timeout=timeout)
    return (directory/(label+'.stdout')).read_text()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--catalog', type=Path, required=True)
    parser.add_argument('--snapshot', type=Path, required=True)
    parser.add_argument('--directory', type=Path, required=True)
    args = parser.parse_args()
    catalog, snapshot, directory = (p.resolve() for p in (args.catalog, args.snapshot, args.directory))
    if directory.is_relative_to(Path(__file__).resolve().parents[1]):
        raise ValueError('Raw data must remain outside Git')
    if digest(catalog) != CATALOG_SHA or digest(snapshot) != SNAPSHOT_SHA:
        raise ValueError('Input revision differs; reconcile explicitly before reads')
    if shutil.disk_usage(directory.parent).free < 1024**3:
        raise ValueError('At least1 GiB free space required')
    directory.mkdir()  # Never overwrite or silently resume an existing attempt.
    names = ','.join(quoted(name) for name in TABLES)
    where = f"s.schema_name='main_models' AND t.table_name IN ({names}) AND t.end_snapshot IS NULL"
    controls = native(catalog, f"""SET enable_external_access=FALSE;
SELECT (SELECT count(*) FROM ducklake_delete_file WHERE end_snapshot IS NULL) deletes,
 (SELECT count(*) FROM ducklake_inlined_data_tables i JOIN ducklake_table t USING(table_id)
 JOIN ducklake_schema s USING(schema_id) WHERE {where}) inlined,
 (SELECT count(*) FROM ducklake_data_file f JOIN ducklake_table t USING(table_id)
 JOIN ducklake_schema s USING(schema_id) WHERE {where} AND f.end_snapshot IS NULL
 AND f.encryption_key IS NOT NULL) encrypted;
""", directory, 'metadata-controls')
    row = next(csv.DictReader(io.StringIO(controls)))
    if any(int(value) for value in row.values()):
        raise ValueError('Delete/inline/encrypted data requires another native read contract')
    files = native(catalog, f"""SET enable_external_access=FALSE;
SELECT t.table_name,f.data_file_id,s.path||t.path||f.path path,f.file_size_bytes bytes
FROM ducklake_data_file f JOIN ducklake_table t USING(table_id)
JOIN ducklake_schema s USING(schema_id) WHERE {where} AND f.end_snapshot IS NULL
AND s.path_is_relative AND t.path_is_relative AND f.path_is_relative
ORDER BY t.table_name,f.data_file_id;
""", directory, 'metadata-files')
    groups, objects = {}, {}
    for row in csv.DictReader(io.StringIO(files)):
        key, size = row['data_file_id'], int(row['bytes'])
        objects[key] = {'url': BASE + row['path'], 'size': size}
        groups.setdefault(row['table_name'], []).append((key, size))
    if [len(groups.get(name, [])) for name in TABLES] != [14, 3, 1, 7, 3, 1]:
        raise ValueError('Pinned file inventory differs')
    (directory/'objects.json').write_text(json.dumps(objects, indent=2)+'\n')
    stage = directory/'2019-outcomes.partial.duckdb'
    relay = RangeRelay(objects, CAP, directory/'requests.jsonl')
    start = time.monotonic()
    try:
        native(stage, extraction_sql(snapshot, groups, relay.base_url), directory,
               'extract', readonly=False, timeout=270)  # Allow cleanup within300s.
    finally:
        relay.close()
        elapsed = time.monotonic()-start
        (directory/'transfer.json').write_text(json.dumps(dict(elapsed_seconds=elapsed,
            received_bytes=relay.budget.received, reserved_bytes=relay.budget.reserved, cap_bytes=CAP), indent=2)+'\n')
    prefix = f"""ATTACH {quoted(snapshot)} AS cached (READ_ONLY);
SET enable_external_access=FALSE;
CREATE TEMP VIEW expected_fielding AS SELECT f.event_key,sum(f.putouts) putouts,
sum(f.assists) assists,sum(f.errors) errors FROM cached.fielding f
JOIN selected_events e USING(event_key) GROUP BY f.event_key;
CREATE TEMP VIEW expected_running AS SELECT c.event_key,c.baserunner,c.runner_id,c.runs,c.outs_on_basepaths
FROM cached.running c JOIN selected_events e USING(event_key) WHERE NOT e.no_play_flag;
"""
    checks = Path(__file__).with_name('2019-outcome-details-verify.sql').read_text()
    native(stage, prefix+checks, directory, 'verify', timeout=60)
    records = [json.loads(line) for line in relay.log.read_text().splitlines()]
    publication_check(stage, records, relay.budget.received, relay.budget.reserved, CAP, elapsed)
    if digest(catalog) != CATALOG_SHA or digest(snapshot) != SNAPSHOT_SHA:
        raise ValueError('Input bytes changed during audit')
    manifest = dict(format='outcome-detail-2019-v1', catalog_sha256=CATALOG_SHA,
        snapshot_sha256=SNAPSHOT_SHA, snapshot_id=857, source_commit=SOURCE_COMMIT,
        attribution=ATTRIBUTION, database_sha256=digest(stage), objects=objects,
        verification_sha256=digest(directory/'verify.stdout'))
    (directory/'manifest.json').write_text(json.dumps(manifest, indent=2)+'\n')
    # Both paths are in the new audit-owned directory; no existing dataset is replaced.
    stage.rename(directory/'2019-outcomes.duckdb')
    print('PASS offline factual gate; bounded internal supplement published locally; '+ATTRIBUTION)


if __name__ == '__main__':
    main()
