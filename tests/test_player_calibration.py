"""Fixed game split and complete original participant validation, before fitting."""
import copy
import csv
import hashlib
import json
from pathlib import Path
import tempfile
import unittest
from unittest import mock
from audits import player_calibration as calibration
from tests.test_player_opportunities import mixed, row


class CalibrationPreparationTests(unittest.TestCase):
    def test_game_split_literal_hash_and_row_order_invariance(self):
        game='OAK201903200'
        expected=int.from_bytes(hashlib.sha256(b'player-calibration-v1\0OAK201903200').digest()[:8], 'big')%10
        self.assertEqual(calibration.fold(game),expected)
        rows=mixed()+[row(6,'Single',game_id='GAME2',batter_id='A',pitcher_id='B')]
        pairs,report=calibration.collect(iter(rows))
        other,report2=calibration.collect(iter(reversed(rows)))
        self.assertEqual(pairs,other)
        self.assertEqual(report,report2)
        self.assertEqual(sum(sum(p[4]) for p in pairs),3)
        self.assertTrue(all(p[0]==calibration.fold(p[1]) for p in pairs))
        self.assertEqual(report['exclusions'],dict(excluded_ibb=1,excluded_interference=1))

    def test_complete_original_validation_and_no_mutation(self):
        rows=mixed(); before=copy.deepcopy(rows)
        calibration.collect(iter(rows))
        self.assertEqual(rows,before)
        for bad in [[row(1),row(1)],[row(1,batter_id='')],[row(1,'Unknown')],[row(1,season='2020')],[]]:
            with self.subTest(bad=bad),self.assertRaises(ValueError):
                calibration.collect(iter(bad))

    def test_publication_refuses_changed_inputs_recipe_collisions_and_bad_provenance(self):
        with tempfile.TemporaryDirectory() as directory:
            root=Path(directory);events=root/'events.csv';report_file=root/'report.json';rows=mixed()
            with events.open('w',newline='') as stream:
                writer=csv.DictWriter(stream,fieldnames=sorted(calibration.outcome.FIELDS));writer.writeheader();writer.writerows(rows)
            _,report=calibration.collect(iter(rows));report_file.write_text(json.dumps(dict(report,provenance={'synthetic':True})))
            pins=tuple(calibration.outcome.digest(p) for p in [events,report_file,Path(calibration.__file__)])
            with mock.patch.object(calibration,'EVENT_SHA',pins[0]),mock.patch.object(calibration,'REPORT_SHA',pins[1]):
                metadata=calibration.prepare(events,report_file,root/'good')
                self.assertEqual(metadata['input_sha256'],list(pins))
                for index in range(3):
                    changed=list(pins);changed[index]='f'*64;target=root/f'changed-{index}'
                    with mock.patch.object(calibration.outcome,'digest',side_effect=[*pins,*changed]),self.assertRaisesRegex(ValueError,'changed before publication'):
                        calibration.prepare(events,report_file,target)
                    self.assertFalse(target.exists())
                collision=root/'collision';collision.write_text('keep')
                with self.assertRaisesRegex(ValueError,'existing'):calibration.prepare(events,report_file,collision)
                self.assertEqual(collision.read_text(),'keep')
            with self.assertRaisesRegex(ValueError,'source digest'):calibration.prepare(events,report_file,root/'unsupported')
            with self.assertRaisesRegex(ValueError,'outside repository'):calibration.prepare(events,report_file,Path(calibration.__file__).parent/'uncreated-calibration-output')
            self.assertFalse((Path(calibration.__file__).parent/'uncreated-calibration-output').exists())

    def test_tsv_has_explicit_game_partition_and_nine_counts(self):
        pairs,_=calibration.collect(iter([row(1,'Single',batter_id='A',pitcher_id='B')]))
        text=calibration.encode(pairs)
        self.assertEqual(text.splitlines()[0], 'fold\tgame_id\tbatter_id\tpitcher_id\tstrikeout\twalk\thbp\tsingle\tdouble\ttriple\thome_run\troe\tcontact_out')
        cells=text.splitlines()[1].split('\t')
        self.assertEqual(cells[2:4],['A','B'])
        self.assertEqual(list(map(int,cells[4:])),[0,0,0,1,0,0,0,0,0])
