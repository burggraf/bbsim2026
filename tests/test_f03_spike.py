import tempfile
import unittest
from pathlib import Path

from audits.filtered_spike import ROOT, run_import


class AtomicImportTests(unittest.TestCase):
    def scripts(self, root, verify="SELECT CASE WHEN (SELECT count(*) FROM games)=1 THEN 1 ELSE error('bad count') END;"):
        extract = root / 'extract.sql'
        check = root / 'check.sql'
        extract.write_text('CREATE TABLE games AS SELECT 1 AS game_id;\n')
        check.write_text(verify + '\n')
        return extract, check

    def test_promotes_only_after_offline_verification(self):
        with tempfile.TemporaryDirectory() as name:
            root = Path(name)
            extract, check = self.scripts(root)
            result = run_import(root, extract, check, min_free_bytes=0)
            self.assertEqual(result.name, '2019-spike.duckdb')
            self.assertTrue(result.is_file())
            self.assertEqual(list(root.glob('.f03-stage-*')), [])
            self.assertTrue((root / 'f03-verify.csv').is_file())

    def test_injected_failure_removes_stage_and_keeps_no_playable_file(self):
        with tempfile.TemporaryDirectory() as name:
            root = Path(name)
            extract, check = self.scripts(root)
            with self.assertRaisesRegex(RuntimeError, 'injected'):
                run_import(root, extract, check, min_free_bytes=0, fail_after_extract=True)
            self.assertFalse((root / '2019-spike.duckdb').exists())
            self.assertEqual(list(root.glob('.f03-stage-*')), [])
            self.assertTrue((root / 'f03-profiles.txt').is_file())

    def test_failed_reconciliation_does_not_promote(self):
        with tempfile.TemporaryDirectory() as name:
            root = Path(name)
            extract, check = self.scripts(root, "SELECT error('bad count');")
            with self.assertRaisesRegex(RuntimeError, 'offline verification'):
                run_import(root, extract, check, min_free_bytes=0)
            self.assertFalse((root / '2019-spike.duckdb').exists())
            self.assertEqual(list(root.glob('.f03-stage-*')), [])

    def test_rejects_catalog_drift_before_creating_stage(self):
        with tempfile.TemporaryDirectory() as name:
            root = Path(name)
            extract, check = self.scripts(root)
            catalog = root / 'catalog.bin'
            catalog.write_bytes(b'changed')
            with self.assertRaisesRegex(ValueError, 'catalog SHA-256'):
                run_import(root, extract, check, min_free_bytes=0,
                           catalog_url=catalog.as_uri(), expected_catalog_sha256='0' * 64)
            self.assertFalse((root / '2019-spike.duckdb').exists())
            self.assertEqual(list(root.glob('.f03-stage-*')), [])

    def test_refuses_output_inside_repository(self):
        with tempfile.TemporaryDirectory() as name:
            root = Path(name)
            extract, check = self.scripts(root)
            with self.assertRaisesRegex(ValueError, 'outside the repository'):
                run_import(ROOT, extract, check, min_free_bytes=0)

    def test_refuses_to_replace_existing_snapshot(self):
        with tempfile.TemporaryDirectory() as name:
            root = Path(name)
            (root / '2019-spike.duckdb').write_text('keep')
            extract, check = self.scripts(root)
            with self.assertRaisesRegex(FileExistsError, '2019-spike'):
                run_import(root, extract, check, min_free_bytes=0)
            self.assertEqual((root / '2019-spike.duckdb').read_text(), 'keep')


if __name__ == '__main__':
    unittest.main()
