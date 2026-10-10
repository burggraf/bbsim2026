"""F-03 development-host spike: atomic local DuckDB promotion, not a production importer."""

import argparse
import hashlib
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
from urllib.request import Request, urlopen


ROOT = Path(__file__).resolve().parent.parent


def run_import(output_dir, extract_sql, verify_sql, *, min_free_bytes=1 << 30,
               fail_after_extract=False, catalog_url=None, expected_catalog_sha256=None):
    output_dir = Path(output_dir).resolve()
    extract_sql, verify_sql = Path(extract_sql).resolve(), Path(verify_sql).resolve()
    if output_dir.is_relative_to(ROOT):
        raise ValueError('snapshot output must be outside the repository')
    if not output_dir.is_dir():
        raise ValueError('output directory must already exist')
    if shutil.disk_usage(output_dir).free < min_free_bytes:
        raise RuntimeError(f'insufficient free disk space (need {min_free_bytes} bytes)')
    final = output_dir / '2019-spike.duckdb'
    if final.exists():
        raise FileExistsError(f'will not replace existing snapshot: {final}')
    if (catalog_url is None) != (expected_catalog_sha256 is None):
        raise ValueError('catalog URL and expected SHA-256 must be supplied together')
    if catalog_url is not None:
        digest = hashlib.sha256()
        total = 0
        with urlopen(Request(catalog_url, headers={'User-Agent': 'bbsim2026-audit/1.0'}),
                     timeout=60) as response:
            while chunk := response.read(1 << 20):
                digest.update(chunk)
                total += len(chunk)
        (output_dir / 'f03-catalog-check.txt').write_text(
            f'url={catalog_url}\nbytes={total}\nsha256={digest.hexdigest()}\n')
        if digest.hexdigest() != expected_catalog_sha256:
            raise ValueError(f'catalog SHA-256 changed: {digest.hexdigest()}')
    # Logs stay outside the stage for failure diagnosis; source rows never go to stdout.
    profile = output_dir / 'f03-profiles.txt'
    extract_error = output_dir / 'f03-extract.stderr.txt'
    verify_log = output_dir / 'f03-verify.csv'
    verify_error = output_dir / 'f03-verify.stderr.txt'
    with tempfile.TemporaryDirectory(prefix='.f03-stage-', dir=output_dir) as stage_dir:
        stage = Path(stage_dir) / 'snapshot.partial.duckdb'
        with profile.open('w') as out, extract_error.open('w') as err:
            result = subprocess.run(['duckdb', '-bail', '-batch', str(stage), '-f', str(extract_sql)],
                                    stdout=out, stderr=err, check=False)
        if result.returncode:
            raise RuntimeError(f'extract failed ({result.returncode}); see {extract_error}')
        if fail_after_extract:
            raise RuntimeError('injected failure after extract, before verification/promotion')
        with verify_log.open('w') as out, verify_error.open('w') as err:
            result = subprocess.run(['duckdb', '-readonly', '-bail', '-batch', '-csv', '-noheader',
                                     '-cmd', 'SET enable_external_access=false', str(stage),
                                     '-f', str(verify_sql)], stdout=out, stderr=err, check=False)
        if result.returncode:
            raise RuntimeError(f'offline verification failed ({result.returncode}); see {verify_error}')
        if not stage.is_file() or any(Path(stage_dir).glob('*.wal')):
            raise RuntimeError('no clean closed DuckDB snapshot to promote')
        # Hard-link creation is atomic and fails if a concurrent import already promoted.
        os.link(stage, final)
    return final


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output-dir', required=True, type=Path)
    parser.add_argument('--extract-sql', type=Path, default=ROOT / 'audits/2019-filtered-extract.sql')
    parser.add_argument('--verify-sql', type=Path, default=ROOT / 'audits/2019-offline-verify.sql')
    parser.add_argument('--inject-failure-after-extract', action='store_true')
    args = parser.parse_args()
    print(run_import(args.output_dir, args.extract_sql, args.verify_sql,
                     fail_after_extract=args.inject_failure_after_extract,
                     catalog_url='https://data.baseball.computer/baseball/v1/baseball.ducklake',
                     expected_catalog_sha256='2a3362ba0bc970515048c87c5bf5bcaab1bd093593b7bafc1e50e2ebaa241583'))
