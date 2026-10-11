import pathlib
import subprocess
import unittest
import csv
import io

QUERY = pathlib.Path(__file__).resolve().parents[1] / "audits/2019-preview-baseline.sql"


class PreviewBaselineTests(unittest.TestCase):
    def query(self, extra=""):
        setup = """
CREATE TABLE events(event_key INTEGER, plate_appearance_result VARCHAR, season INTEGER DEFAULT 2019);
CREATE TABLE batting(event_key INTEGER, plate_appearances INTEGER);
CREATE TABLE audit_manifest(catalog_sha256 VARCHAR);
INSERT INTO audit_manifest VALUES ('2a3362ba0bc970515048c87c5bf5bcaab1bd093593b7bafc1e50e2ebaa241583');
INSERT INTO events VALUES (1,'Single'),(2,'StrikeOut'),(3,'Walk'),(4,'IntentionalWalk'),(5,'Interference'),(6,NULL);
INSERT INTO batting VALUES (1,1),(2,1),(3,1),(4,1),(5,1),(6,0);
"""
        return subprocess.run(["duckdb", "-csv", ":memory:"], input=(setup + extra).replace("INSERT INTO events VALUES", "INSERT INTO events(event_key,plate_appearance_result) VALUES") + QUERY.read_text(), text=True, capture_output=True)

    def test_observed_categories_and_exclusions_reconcile_without_invented_pa(self):
        result = self.query()
        self.assertEqual(result.returncode, 0, result.stderr)
        rows = dict(csv.reader(io.StringIO(result.stdout)))
        self.assertEqual(rows["format"], "preview-counts-v1")
        self.assertEqual(rows["selected_pa"], "5")
        self.assertEqual(rows["excluded_ibb"], "1")
        self.assertEqual(rows["excluded_interference"], "1")
        self.assertEqual([rows[k] for k in ["strikeout", "walk", "hbp", "single", "double", "triple", "home_run", "roe", "contact_out"]], ["1", "1", "0", "1", "0", "0", "0", "0", "0"])

    def test_unknown_pa_and_duplicate_grains_are_errors(self):
        for extra in ["INSERT INTO events VALUES(7,'Mystery'); INSERT INTO batting VALUES(7,1);", "INSERT INTO events VALUES(1,'Single');", "INSERT INTO batting VALUES(1,1);", "INSERT INTO batting VALUES(7,1);", "UPDATE events SET plate_appearance_result=NULL WHERE event_key=1;", "UPDATE events SET season=2020 WHERE event_key=1;", "UPDATE audit_manifest SET catalog_sha256=NULL;", "DELETE FROM batting WHERE event_key=1;", "UPDATE batting SET plate_appearances=0 WHERE event_key=1;"]:
            with self.subTest(extra=extra):
                result = self.query(extra)
                self.assertNotEqual(result.returncode, 0, result.stdout)
                self.assertIn("preview baseline:", result.stderr)

    def test_documented_coarsening_is_explicit(self):
        result = self.query("INSERT INTO events VALUES(7,'GroundRuleDouble'),(8,'InsideTheParkHomeRun'),(9,'FieldersChoice'),(10,'SacrificeFly'),(11,'SacrificeHit'); INSERT INTO batting VALUES(7,1),(8,1),(9,1),(10,1),(11,1);")
        self.assertEqual(result.returncode, 0, result.stderr)
        rows = dict(csv.reader(io.StringIO(result.stdout)))
        self.assertIn("double", rows)
        self.assertEqual([rows[k] for k in ["double", "home_run", "contact_out", "selected_pa"]], ["1", "1", "3", "10"])
