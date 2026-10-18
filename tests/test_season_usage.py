"""Receipt-backed integration, including corrupt retained outputs and honest gaps."""
from copy import deepcopy
import hashlib
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch
from preview import season_usage as usage
from preview import season_runner as season
from test_usage_ledger import game,targets
from test_season_runner import accounting


class SeasonUsageTests(unittest.TestCase):
    def setUp(self):
        self.temp=tempfile.TemporaryDirectory();self.addCleanup(self.temp.cleanup)
        self.root=Path(self.temp.name);self.batch=self.root/'batch';self.batch.mkdir()
        self.target=self.root/'targets';self.target.mkdir()
        e=game();self.gid=e['game_id'];folder=self.batch/'games'/self.gid;folder.mkdir(parents=True)
        lineup='game_id\t'+self.gid+'\n';cast=''
        text=accounting()
        for side,pid in [('Away','a'),('Home','h')]:
            cast+=f'cast:{side}:0:{pid}\tx\n'
            for i in range(1,10):
                actor=pid if i==1 else pid+str(i)
                lineup+=f'{side}:{i}:{i}:{actor}\tx\n';cast+=f'batter:{actor}\tx\n'
                if i>1:
                    text+='\t'.join(['batter',side,actor]+['0']*len(season.METRICS['batter']))+'\n'
        (folder/'lineup.tsv').write_text(lineup);(folder/'cast.tsv').write_text(cast)
        (folder/'stdout.tsv').write_text(text);(folder/'stderr.txt').write_text('')
        e['seed']=season.game_seed(1,self.gid);text=text.replace('seed\t7\n','seed\t'+str(e['seed'])+'\n');(folder/'stdout.tsv').write_text(text)
        e['role_actors']=season.actor_bindings(dict(lineup=lineup,cast=cast));e.update(usage._starters(lineup,e))
        parsed=season.parse_accounting(text,e)
        row={**e,'exit':0,'status':'complete','teams':parsed['teams']}
        self.manifest=dict(format='controlled-season-v1',simulation_policy='regular-full-game-v1',master_seed=1,
            seed_algorithm='sha256:bbsim-season-v1:first8-big-endian',games=[row],counts={'complete':1},
            complete_season=True,versions=parsed['versions'])
        self.put(self.batch/'manifest.json',self.manifest);self.put(self.batch/'schedule.json',[])
        self.put(self.batch/'summary.json',season.accumulate([parsed]))
        self.receipt=dict(format='controlled-season-receipt-v1',input_sha256={},counts={'complete':1},complete_season=True)
        self.repin()
        historical=targets();native=self.target/'native.json';native.write_text('[]\n')
        historical['provenance']=dict(snapshot_sha256=usage.target.SNAPSHOT_SHA256,query_sha256=usage.target.SQL_SHA256,
            recipe_sha256=usage.game.real.digest(Path(usage.target.__file__)),native_export_sha256=usage.game.real.digest(native))
        self.put(self.target/'targets.json',historical)
        self.put(self.target/'receipt.json',dict(format=usage.target.FORMAT+'-receipt',
            payload_sha256=usage.game.real.digest(self.target/'targets.json'),native_export_sha256=usage.game.real.digest(native),
            inputs_sha256={'2019-spike.duckdb':usage.target.SNAPSHOT_SHA256,'2019-season-usage-targets.sql':usage.target.SQL_SHA256,
                'season_usage_targets.py':historical['provenance']['recipe_sha256']}))
        approved=patch.object(usage,'TARGET_SHA256',usage.game.real.digest(self.target/'targets.json'),create=True)
        approved.start();self.addCleanup(approved.stop)

    def put(self,path,value):path.write_text(json.dumps(value))

    def repin(self):
        self.receipt['output_sha256']={str(p.relative_to(self.batch)):usage.game.real.digest(p)
            for p in self.batch.rglob('*') if p.is_file() and p.name!='receipt.json'}
        self.put(self.batch/'receipt.json',self.receipt)

    def test_duplicate_selected_slots_refuse_even_with_nine_distinct_actors(self):
        text=(self.batch/'games'/self.gid/'lineup.tsv').read_text().replace('Away:2:2:a2','Away:1:2:a2')
        with self.assertRaisesRegex(ValueError,'slot'):usage._starters(text,self.manifest['games'][0])

    def test_rebuild_repeat_publication_and_matched_totals(self):
        first,_=usage.collect(self.batch,self.target);second,_=usage.collect(self.batch,self.target)
        self.assertEqual(first,second)
        self.assertEqual(first['ledger']['teams']['SFN']['pa'],29)
        self.assertTrue(first['comparison']['aggregate']['outs']['passed'])
        out=self.root/'out';usage.publish(self.batch,self.target,out)
        self.assertTrue((out/'receipt.json').is_file())
        with self.assertRaises(ValueError):usage.publish(self.batch,self.target,out)
        receipt=json.loads((out/'receipt.json').read_text())
        for name,pin in receipt['output_sha256'].items():self.assertEqual(usage.game.real.digest(out/name),pin)

    def test_selected_batter_removed_before_first_pa_is_not_invented_as_appearance(self):
        path=self.batch/'games'/self.gid/'stdout.tsv'
        path.write_text('\n'.join(line for line in path.read_text().splitlines() if not line.startswith('batter\tAway\ta2\t'))+'\n')
        parsed=season.parse_accounting(path.read_text(),self.manifest['games'][0])
        self.put(self.batch/'summary.json',season.accumulate([parsed]));self.repin()
        result,_=usage.collect(self.batch,self.target)
        self.assertNotIn('batter:a2',result['ledger']['players'])
        self.assertIn('a2',result['game_receipts'][0]['entry']['starting_batter_ids']['SFN'])
        self.assertEqual(result['ledger']['teams']['SFN']['pa'],29)

    def test_corrupt_raw_or_target_refused_before_publication(self):
        path=self.batch/'games'/self.gid/'stdout.tsv';path.write_text(path.read_text()+'bad\n')
        with self.assertRaisesRegex(ValueError,'digest mismatch'):usage.publish(self.batch,self.target,self.root/'bad')
        self.assertFalse((self.root/'bad').exists())

    def test_rehashed_seed_actors_summary_and_schedule_still_refused(self):
        original=deepcopy(self.manifest)
        for kind in ['seed','actors','schedule','summary']:
            with self.subTest(kind=kind):
                self.manifest=deepcopy(original)
                self.put(self.batch/'summary.json',season.accumulate([season.parse_accounting(
                    (self.batch/'games'/self.gid/'stdout.tsv').read_text(),original['games'][0])]))
                if kind=='seed':self.manifest['games'][0]['seed']+=1
                if kind=='actors':self.manifest['games'][0]['role_actors']['Home']['pitcher']=['impostor']
                if kind=='schedule':self.manifest['games'][0]['date']='2019-04-02'
                if kind=='summary':self.put(self.batch/'summary.json',{})
                self.put(self.batch/'manifest.json',self.manifest);self.repin()
                with self.assertRaises(ValueError):usage.collect(self.batch,self.target)

    def test_refusal_leaves_history_incomplete_without_usage(self):
        row=self.manifest['games'][0];self.manifest['games']=[{k:row[k] for k in (*usage.CORE,'seed')}]
        self.manifest['games'][0].update(status='input_refused',reason='unsupported lineup')
        for p in (self.batch/'games'/self.gid).iterdir():p.unlink()
        self.manifest.update(counts={'input_refused':1},complete_season=False,versions=None)
        self.receipt.update(counts={'input_refused':1},complete_season=False)
        self.put(self.batch/'manifest.json',self.manifest);self.put(self.batch/'summary.json',dict(teams={},players={}));self.repin()
        result,_=usage.collect(self.batch,self.target)
        self.assertFalse(result['ledger']['history_complete']);self.assertEqual(result['ledger']['teams']['SFN']['pa'],0)
        self.assertEqual(result['comparison']['coverage']['matched_games'],0)
        self.assertIsNone(result['comparison']['aggregate']['pa']['passed'])

    def test_path_escape_duplicate_json_and_drift_no_receipt(self):
        self.receipt['output_sha256']['../outside']='a'*64;self.put(self.batch/'receipt.json',self.receipt)
        with self.assertRaisesRegex(ValueError,'Unsafe'):usage.collect(self.batch,self.target)
        self.repin();path=self.batch/'manifest.json';path.write_text('{"format":1,"format":2}')
        self.repin()
        with self.assertRaisesRegex(ValueError,'Duplicate JSON'):usage.collect(self.batch,self.target)

    def test_consistently_rehashed_target_tampering_is_not_approved(self):
        p=self.target/'targets.json';historical=json.loads(p.read_text())
        historical['records'][0]['usage']+=1;historical['totals']['pa']+=1;self.put(p,historical)
        receipt=json.loads((self.target/'receipt.json').read_text());receipt['payload_sha256']=usage.game.real.digest(p)
        self.put(self.target/'receipt.json',receipt)
        with self.assertRaisesRegex(ValueError,'approved target'):usage.collect(self.batch,self.target)

    def test_recipe_drift_during_collection_refuses_receipt(self):
        recipe=self.root/'replay-recipe.py';recipe.write_text('loaded recipe')
        original=usage.collect
        def drift(*args):
            report=original(*args);recipe.write_text('different recipe');return report
        out=self.root/'recipe-drift'
        with patch.object(usage,'__file__',str(recipe)),patch.object(usage,'collect',side_effect=drift):
            with self.assertRaisesRegex(ValueError,'inputs changed'):usage.publish(self.batch,self.target,out)
        self.assertFalse((out/'receipt.json').exists())

    def test_publication_rechecks_replayed_inputs(self):
        original=usage.game.ratings.publish
        def drift(text,inputs,pins,path):
            original(text,inputs,pins,path)
            if path.name=='ledger.json':(self.batch/'summary.json').write_text('{}')
        out=self.root/'drift'
        with patch.object(usage.game.ratings,'publish',side_effect=drift):
            with self.assertRaisesRegex(ValueError,'inputs changed'):usage.publish(self.batch,self.target,out)
        self.assertFalse((out/'receipt.json').exists())
