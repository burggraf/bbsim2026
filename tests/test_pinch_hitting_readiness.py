"""Literal synthetic bench evidence, not a pregame-eligibility certificate."""
import copy
import json
from pathlib import Path
import random
import tempfile
import unittest
from unittest.mock import patch
from audits import pinch_hitting_readiness as audit
from preview import game_inputs as game
from preview import player_ratings as ratings
from test_lineup_readiness import fixture


def source(dh=True):
    games,people,appearances=fixture(dh=dh)
    return dict(data=dict(game=games,person=people,appearance=appearances),
        report=dict(roles=dict(batter=dict(profiles=[]))),
        readiness=dict(games=[dict(game_id='G1',results=[dict(forfeit=False,
            shortened=False,suspended=False,tied=False,duration_outs=54)])]))


def add(s,pid='bench',side='Away',position=11,start=40,end=50,person=True):
    if person and not any(p['player_id']==pid for p in s['data']['person']):
        s['data']['person'].append(dict(player_id=pid,first_name='Bench',last_name='Source',bats='Right',throws='Right'))
    s['data']['appearance'].append(dict(game_id='G1',player_id=pid,side=side,
        fielding_position=position,start_event_id=start,end_event_id=end))


def actors(report,side='Away'):
    return {p['player_id']:p for p in next(t for t in report['games'][0]['sides'] if t['side']==side)['actors']}


class ReadinessTests(unittest.TestCase):
    def test_fielding_only_source_does_not_claim_pinch_hit_participation_coverage(self):
        r=audit.inventory(source())
        self.assertIn('fielding-appearance source',r['limits'])
        self.assertIn('PH-only/PR-only participation coverage unproven',r['limits'])

    def test_candidates_are_not_starting_players_or_recorded_pitchers(self):
        s=source();add(s);add(s,'relief',position=1);add(s,'bench',position=7,start=51,end=70)
        before=copy.deepcopy(s);r=audit.inventory(s);a=actors(r)
        self.assertTrue(a['bench']['participant_proxy'])
        self.assertEqual(a['bench']['positions'],[7,11])
        self.assertEqual(a['ap']['reasons'],['recorded_pitcher','starting_player'])
        self.assertEqual(a['relief']['reasons'],['recorded_pitcher'])
        self.assertIsNone(r['opening_bench_available'])
        self.assertEqual(s,before)

    def test_ambiguous_identity_unknown_and_reversed_ranges_are_retained(self):
        s=source();add(s,'both');add(s,'both',side='Home',start=41);add(s,'missing',person=False)
        add(s,'unknown',position=None,start=None);add(s,'reversed',start=60,end=59)
        add(s,'uncertain');add(s,'uncertain',side=None,start=41)
        r=audit.inventory(s);a=actors(r)
        for pid,reason in [('both','cross_side_identity'),('missing','person_lookup'),
                ('unknown','unknown_position'),('unknown','unknown_range'),
                ('reversed','reversed_range'),('uncertain','unknown_side')]:
            with self.subTest(pid=pid,reason=reason):
                self.assertIn(reason,a[pid]['reasons']);self.assertFalse(a[pid]['participant_proxy'])
        self.assertEqual(len(r['unassigned_appearances']),1)
        self.assertEqual(r['unassigned_appearances'][0]['player_id'],'uncertain')
        self.assertEqual(a['reversed']['ranges'],[[11,60,59]])

    def test_explicit_cold_counts_and_profile_opportunity_are_separate(self):
        s=source();add(s);add(s,'rated')
        counts=dict.fromkeys(ratings.KEYS,0);counts['single']=3
        s['report']['roles']['batter']['profiles']=[dict(player_id='rated',included_pa=3,counts=counts)]
        a=actors(audit.inventory(s))
        self.assertEqual(a['bench']['batting'],dict(status='cold_zero',included_pa=0,counts=[0]*9))
        self.assertEqual(a['rated']['batting']['included_pa'],3)
        self.assertEqual(sum(a['rated']['batting']['counts']),3)
        s['report']['roles']['batter']['profiles'][0]['included_pa']=2
        with self.assertRaises(ValueError):audit.inventory(s)

    def test_both_sides_exceptions_empty_proxies_and_shuffled_input_are_visible(self):
        s=source(False);s['readiness']['games'][0]['results'][0]['shortened']=True
        r=audit.inventory(s)
        self.assertEqual(len(r['games']),1);self.assertEqual(len(r['games'][0]['sides']),2)
        self.assertFalse(r['games'][0]['use_dh'])
        self.assertTrue(r['games'][0]['results'][0]['shortened'])
        self.assertEqual(r['teams']['A']['participant_proxies'],0)
        self.assertIsNone(r['teams']['A']['opening_bench_available'])
        for values in s['data'].values():random.Random(42).shuffle(values)
        self.assertEqual(audit.inventory(s),r)

    def test_same_identity_retains_team_splits_and_equal_names_do_not_merge(self):
        s=source();add(s);add(s,'other')
        second=copy.deepcopy(s['data']['game'][0]);second.update(game_id='G2',date='2019-03-21',away_team_id='A2')
        second['results'][0]['away_team_id']='A2'
        s['data']['game'].append(second)
        s['data']['appearance'].extend(dict(a,game_id='G2') for a in list(s['data']['appearance']))
        s['readiness']['games'].append(dict(s['readiness']['games'][0],game_id='G2'))
        r=audit.inventory(s)
        self.assertEqual(r['teams']['A']['participant_proxies'],2)
        self.assertEqual(r['teams']['A2']['participant_proxies'],2)
        self.assertIn('bench',r['multi_team_player_ids'])
        self.assertEqual({pid for pid,p in actors(r).items() if p['participant_proxy']},{'bench','other'})

    def test_unsupported_lineups_and_duplicate_profiles_do_not_create_candidates(self):
        s=source();add(s);s['data']['game'][0]['results'][0]['suspension_flag']=True
        r=audit.inventory(s)
        self.assertFalse(r['games'][0]['starting_maps_supported'])
        self.assertIn('unsupported_starting_maps',actors(r)['bench']['reasons'])
        self.assertFalse(actors(r)['bench']['participant_proxy'])
        profile=dict(player_id='bench',included_pa=0,counts=dict.fromkeys(ratings.KEYS,0))
        s['report']['roles']['batter']['profiles']=[profile,profile]
        with self.assertRaisesRegex(ValueError,'Duplicate'):audit.inventory(s)

    def test_destination_redirect_is_rechecked_before_any_publication(self):
        for into_project in [True,False]:
            with self.subTest(into_project=into_project),tempfile.TemporaryDirectory() as directory:
                root=Path(directory);project=root/'project';(project/'preview').mkdir(parents=True)
                recipe=project/'preview'/'game_inputs.py';recipe.write_text('private recipe stand-in')
                target=project if into_project else root/'elsewhere'
                if not into_project:target.mkdir()
                destination=root/'destination';destination.mkdir()
                paths=[root/str(i) for i in range(5)]
                for p in paths:p.write_text('source')
                def redirect(_):
                    destination.rename(root/'original-destination')
                    destination.symlink_to(target,target_is_directory=True)
                    return source()
                with patch.object(game,'__file__',str(recipe)),patch.object(game,'read_sources',side_effect=redirect):
                    with self.assertRaisesRegex(ValueError,'outside repository' if into_project else 'destination changed'):
                        audit.export(paths,destination/'report.json')
                self.assertFalse((target/'report.json').exists())
                self.assertEqual(sorted(p.name for p in project.iterdir()),['preview'])

    def test_export_is_exclusive_authenticated_and_rechecks_before_publication(self):
        with tempfile.TemporaryDirectory() as directory:
            root=Path(directory);paths=[root/str(i) for i in range(5)]
            for p in paths:p.write_text('source')
            with patch.object(game,'read_sources',return_value=source()) as read:
                out=root/'report.json';audit.export(paths,out)
                self.assertTrue(json.loads(out.read_text())['provenance']['input_sha256'])
                with self.assertRaisesRegex(ValueError,'overwrite'):audit.export(paths,out)
                self.assertEqual(read.call_count,1)
                def drift(_):paths[0].write_text('changed');return source()
                read.side_effect=drift
                with self.assertRaisesRegex(ValueError,'changed'):audit.export(paths,root/'drift.json')
                self.assertFalse((root/'drift.json').exists())
            with self.assertRaises(ValueError):audit.export(paths,root/'unsupported.json')
            self.assertFalse((root/'unsupported.json').exists())
        with self.assertRaisesRegex(ValueError,'outside repository'):
            audit.export([Path('/missing')]*5,Path(__file__).resolve().parents[1]/'.invalid-pinch.json')


if __name__=='__main__':unittest.main()
