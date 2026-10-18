"""Assumptions are explicit; same-game evidence is never promoted to an eligible roster."""
import copy
import hashlib
import json
from pathlib import Path
import random
import tempfile
import unittest
from unittest.mock import patch
from preview import bench_inputs as bench
from preview import game_inputs as game
from test_pinch_hitting_readiness import source, add


class BenchInputsTests(unittest.TestCase):
    def test_closed_pool_counts_positions_and_names_are_deterministic_not_future_entries(self):
        s=source();add(s,'bench',position=7);add(s,'phonly',position=11)
        add(s,'reliever',position=1);add(s,'reversed',position=6,start=92,end=91)
        add(s,'unknown',position=None);add(s,'conflict',side='Away');add(s,'conflict',side='Home',start=41)
        original=copy.deepcopy(s)
        a=bench.build_bench(s,'G1')
        self.assertEqual(s,original)
        self.assertEqual(a['pool_sizes'],{'Away':2,'Home':0})
        self.assertIn('Away:bench:positions\t7\n',a['bench'])
        self.assertIn('Away:phonly:positions\t?\n',a['bench'])
        self.assertIn('Away:bench:counts\t0,0,0,0,0,0,0,0,0\n',a['bench'])
        self.assertIn('Away:bench:name\tBench Source\n',a['bench'])
        self.assertIn('not verified eligibility',a['bench'])
        self.assertNotIn('reliever:name',a['bench']);self.assertNotIn('reversed:name',a['bench'])
        self.assertNotIn('unknown:name',a['bench']);self.assertNotIn('conflict:name',a['bench'])
        for rows in s['data'].values():random.Random(4).shuffle(rows)
        self.assertEqual(bench.build_bench(s,'G1'),a)
        # Different legal observed entry innings never become decision inputs.
        for row in s['data']['appearance']:
            if row['player_id']=='bench':row.update(start_event_id=1,end_event_id=2)
        self.assertEqual(bench.build_bench(s,'G1')['bench'],a['bench'])
        with self.assertRaisesRegex(ValueError,'Unknown'):bench.build_bench(s,'absent')

    def test_exclusive_receipt_last_and_source_drift(self):
        s=source();add(s)
        with tempfile.TemporaryDirectory() as d:
            root=Path(d);inputs=[root/f'i{i}' for i in range(5)]
            for p in inputs:p.write_text('pinned')
            pins=tuple(game.real.digest(p) for p in inputs)
            with patch.object(game,'INPUT_SHA',pins),patch.object(game,'read_sources',return_value=s),patch.object(game,'build_game',return_value={'lineup':'lineup','cast':'cast','game_id':'G1'}):
                out=root/'good';receipt=bench.export(inputs,'G1',out)
                self.assertEqual(set(receipt['output_sha256']),{'lineup.tsv','cast.tsv','bench.tsv'})
                for name,sha in receipt['output_sha256'].items():self.assertEqual(hashlib.sha256((out/name).read_bytes()).hexdigest(),sha)
                self.assertEqual(json.loads((out/'receipt.json').read_text()),receipt)
                with self.assertRaisesRegex(ValueError,'overwrite'):bench.export(inputs,'G1',out)
                original=game.ratings.publish
                def drift(text,paths,expected,target):
                    original(text,paths,expected,target)
                    inputs[0].write_text('changed')
                with patch.object(game.ratings,'publish',side_effect=drift):
                    with self.assertRaises(ValueError):bench.export(inputs,'G1',root/'drift')
                self.assertFalse((root/'drift'/'receipt.json').exists())
            with self.assertRaises(ValueError):bench.export(inputs,'G1',root/'bad-source')
            self.assertFalse((root/'bad-source').exists())


if __name__=='__main__':unittest.main()
