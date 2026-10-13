import csv
import io
import re
import subprocess
import tempfile
import unittest
from pathlib import Path
from audits.bounded_ranges import RangeRelay
from audits.outcome_details import extraction_sql, publication_check
from tests.test_outcome_details import FIXTURE

OBJECTS={name:[('1',10)] for name in ('stg_events','stg_event_audit','stg_event_flags',
 'stg_event_baserunners','stg_event_fielding_plays','stg_game_fielding_appearances')}

class DetailExtractTests(unittest.TestCase):
    def test_identity_bridge_precedes_sparse_key_reads(self):
        sql=extraction_sql(Path("/tmp/cache's.duckdb"),OBJECTS,'http://127.0.0.1:1234')
        self.assertIn("cache''s.duckdb",sql)
        self.assertLess(sql.index('CREATE TABLE source_events'),sql.index('CREATE TABLE stg_event_audit'))
        self.assertLess(sql.index('error('),sql.index('CREATE TABLE stg_event_audit'))
        self.assertIn('EXCEPT ALL',sql)
        self.assertIn('AND season=2019',sql)
        self.assertIn('autoinstall_known_extensions=FALSE',sql)
        self.assertIn('http_retries=0',sql)
        self.assertNotIn('INSTALL httpfs',sql)
        self.assertIn('CHECKPOINT',sql)

    def test_native_pipeline_filters_foreign_rows_and_verifies_without_network(self):
        class Response(io.BytesIO):
            status=206
            def __init__(self, body, url, start, end, size):
                super().__init__(body)
                self.url=url
                self.headers={'Content-Range':f'bytes {start}-{end}/{size}',
                              'Content-Length':str(len(body)),'ETag':'"fixture"'}
            def geturl(self):return self.url
        with tempfile.TemporaryDirectory() as folder:
            root=Path(folder);cache=root/'cache.duckdb';stage=root/'stage.duckdb'
            sql=FIXTURE+"""
ALTER TABLE selected_events ADD COLUMN season SMALLINT DEFAULT 2019;
ALTER TABLE source_events ADD COLUMN season SMALLINT DEFAULT 2019;
CREATE TABLE games AS SELECT * FROM selected_games;
CREATE TABLE events AS SELECT * FROM selected_events;
CREATE TABLE fielding AS SELECT * FROM expected_fielding;
CREATE TABLE running AS SELECT * FROM expected_running;
INSERT INTO source_events SELECT 99,'OTHER',1,FALSE,'Away','X','Y',1,0,2018;
INSERT INTO stg_event_audit VALUES(99,'OTHER',1,'2018.EVN',1,'K');
"""
            groups,objects,data={},{},{}
            for i,name in enumerate(OBJECTS,1):
                table='source_events' if name=='stg_events' else name
                path=root/(name+'.parquet')
                sql+=f"COPY {table} TO '{path}' (FORMAT PARQUET);\n"
            result=subprocess.run(['duckdb','-bail','-batch','-init','/dev/null',str(cache)],
                                  input=sql,text=True,capture_output=True)
            self.assertEqual(result.returncode,0,result.stderr)
            for i,name in enumerate(OBJECTS,1):
                body=(root/(name+'.parquet')).read_bytes()
                url=f'https://data.baseball.computer/baseball/v1/bc_publish_data/{name}.parquet'
                data[url]=body;objects[str(i)]={'url':url,'size':len(body)}
                groups[name]=[(str(i),len(body))]
            def opener(request,**kwargs):
                start,end=map(int,re.fullmatch(r'bytes=(\d+)-(\d+)',request.get_header('Range')).groups())
                body=data[request.full_url]
                return Response(body[start:end+1],request.full_url,start,end,len(body))
            relay=RangeRelay(objects,4*1024**2,root/'requests.jsonl',opener=opener)
            try:
                result=subprocess.run(['duckdb','-bail','-batch','-init','/dev/null',str(stage)],
                    input=extraction_sql(cache,groups,relay.base_url),text=True,capture_output=True,timeout=20)
                self.assertEqual(result.returncode,0,result.stderr)
            finally:
                relay.close()
            prefix=f"ATTACH '{cache}' AS cached (READ_ONLY); SET enable_external_access=FALSE; CREATE TEMP VIEW expected_fielding AS SELECT * FROM cached.fielding; CREATE TEMP VIEW expected_running AS SELECT * FROM cached.running;\n"
            result=subprocess.run(['duckdb','-readonly','-bail','-batch','-init','/dev/null','-csv',str(stage)],
                input=prefix+Path('audits/2019-outcome-details-verify.sql').read_text(),text=True,capture_output=True)
            self.assertEqual(result.returncode,0,result.stderr)
            values={r['metric']:int(r['n']) for r in csv.DictReader(io.StringIO(result.stdout))}
            self.assertEqual((values['events'],values['raw_audit_rows'],values['putouts']),(1,1,1))

    def test_missing_or_extra_source_groups_are_rejected(self):
        for groups in [{},dict(OBJECTS,unexpected=[('2',10)]),{k:[] for k in OBJECTS}]:
            with self.subTest(groups=groups),self.assertRaises(ValueError):
                extraction_sql(Path('/tmp/cache.duckdb'),groups,'http://127.0.0.1:1234')

    def test_partial_cannot_publish_with_failed_ranges_wal_over_cap_or_timeout(self):
        with tempfile.TemporaryDirectory() as folder:
            path=Path(folder)/'partial.duckdb';path.write_bytes(b'valid')
            record=dict(status='complete',bytes=4)
            for records,received,reserved,elapsed in [([],0,0,1),([dict(record,status='failed')],4,4,1),
              ([record],3,4,1),([record],4,11,1),([record],4,4,301)]:
                with self.subTest(records=records,reserved=reserved,elapsed=elapsed),self.assertRaises(ValueError):
                    publication_check(path,records,received,reserved,10,elapsed)
            Path(str(path)+'.wal').write_bytes(b'wal')
            with self.assertRaises(ValueError):publication_check(path,[record],4,4,10,1)
            Path(str(path)+'.wal').unlink()
            self.assertIsNone(publication_check(path,[record],4,4,10,1))

if __name__=='__main__': unittest.main()
