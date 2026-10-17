import copy
import unittest
from pathlib import Path
import tempfile
from audits import season_readiness as audit


def scheduled(day='2019-04-01', number=0, **extra):
    return dict(date=day+' 00:00:00', visiting_team='SEA', home_team='OAK',
                double_header=number, postponement_indicator=None, makeup_dates=None, **extra)


def game(gid='one', day='2019-04-01', status='SingleGame'):
    return dict(game_id=gid, date=day, away_team_id='SEA', home_team_id='OAK', doubleheader_status=status)


class SeasonReadinessTests(unittest.TestCase):
    def test_schedule_matches_and_retains_all_rows_without_mutation(self):
        rows=[scheduled(), scheduled('2019-04-02')];games=[game(),game('two','2019-04-03')]
        before=copy.deepcopy((rows,games));r=audit.reconcile_schedule(rows,games)
        self.assertEqual(r['counts'],dict(matched=1, unmatched=1, ambiguous=0, not_made_up=0, unknown=0))
        self.assertEqual(r['unassigned_game_ids'],['two']);self.assertEqual((rows,games),before)
        self.assertEqual(r,audit.reconcile_schedule(list(reversed(rows)),list(reversed(games))))

    def test_rescheduled_date_is_not_original_date_or_a_new_game(self):
        row=scheduled();row.update(postponement_indicator='Rain',makeup_dates='20190403')
        r=audit.reconcile_schedule([row],[game('makeup','2019-04-03')]);d=r['rows'][0]
        self.assertEqual(d['status'],'matched');self.assertEqual(d['game_id'],'makeup')
        self.assertEqual(d['source'],row);self.assertEqual(d['effective_date'],'2019-04-03')
        row['makeup_dates']='20190927 - Not made up'
        self.assertEqual(audit.reconcile_schedule([row],[])['counts']['not_made_up'],1)
        row['makeup_dates']=None
        self.assertEqual(audit.reconcile_schedule([row],[])['counts']['unknown'],1)

    def test_doubleheaders_use_explicit_numbers_and_ambiguity_is_not_guessed(self):
        games=[game('one',status='DoubleHeaderGame1'),game('two',status='DoubleHeaderGame2')]
        r=audit.reconcile_schedule([scheduled(number=1),scheduled(number=2)],games)
        self.assertEqual(r['counts']['matched'],2)
        r=audit.reconcile_schedule([scheduled()],games)
        self.assertEqual(r['counts']['ambiguous'],1);self.assertEqual(r['unassigned_game_ids'],['one','two'])

    def test_no_two_schedule_slots_can_claim_one_game(self):
        rows=[scheduled(),scheduled('2019-03-31')]
        rows[1].update(postponement_indicator='Rain',makeup_dates='20190401')
        r=audit.reconcile_schedule(rows,[game()]);self.assertEqual(r['counts']['ambiguous'],2)
        self.assertEqual(r['unassigned_game_ids'],['one'])

    def test_publication_refuses_repository_existing_and_unpinned_inputs(self):
        with self.assertRaises(ValueError):audit.publish(Path('missing'),Path('missing'),Path('bad-season.json'))
        with tempfile.TemporaryDirectory() as directory:
            p=Path(directory);a=p/'a';b=p/'b';output=p/'report.json'
            a.write_text('a');b.write_text('b')
            with self.assertRaises(ValueError):audit.publish(a,b,output)
            self.assertFalse(output.exists());output.write_text('keep')
            with self.assertRaises(ValueError):audit.publish(a,b,output)
            self.assertEqual(output.read_text(),'keep')

    def test_unknown_date_shape_refuses_instead_of_truncating_garbage(self):
        for value in [None,'2019-04-01garbage']:
            row=scheduled();row['date']=value
            with self.subTest(value=value),self.assertRaises(ValueError):audit.reconcile_schedule([row],[game()])

    def test_invalid_grains_and_dimensions_refuse(self):
        for rows,games in [([scheduled(),scheduled()],[game()]),([scheduled()],[game(),game()]),
                           ([scheduled(day='2020-04-01')],[game()]),([scheduled(number=True)],[game()]),
                           ([scheduled()],[game(status='Invented')])]:
            with self.subTest(rows=rows,games=games),self.assertRaises(ValueError):audit.reconcile_schedule(rows,games)


if __name__=='__main__':unittest.main()
