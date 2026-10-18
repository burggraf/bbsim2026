"""Exclusive per-game inputs from authenticated complete offline 2019 audit artifacts."""
import argparse
import copy
import csv
import hashlib
import json
from pathlib import Path
from audits import lineup_readiness as lineup
from audits import player_opportunities as player
from audits import pitcher_workload as workload
from audits import bullpen_readiness as bullpen
from audits.outcome_inventory import FIELDS
from audits.raw_mechanisms import unique_object
from preview import real_lineup as real
from preview import player_ratings as ratings
from preview import pitching_cast as cast

READINESS_SHA='8e32100590fd615fded3a29ff8693677c8675cad8f0bfcc077cca2840bcc7df6'
NATIVE_SHA='bef8108455784113702dd4003172ac99162d24dfb8522da4e2a0c53fa16dbf02'
INPUT_SHA=(real.REPORT_SHA,NATIVE_SHA,ratings.REPORT_SHA,cast.EVENTS_SHA,READINESS_SHA)


def _destination(output):
    output=Path(output)
    if output.exists() or output.is_symlink():raise ValueError('Refusing to overwrite game input destination')
    if output.resolve().is_relative_to(Path(__file__).resolve().parents[1]):
        raise ValueError('Game inputs must be outside repository')
    return output.resolve()


def read_sources(paths):
    """Authenticate the complete cohort before absence may mean zero observations."""
    if tuple(real.digest(p) for p in paths)!=INPUT_SHA:raise ValueError('Unsupported game input source digest')
    accepted,report,readiness=[json.loads(paths[i].read_text(),object_pairs_hook=unique_object) for i in [0,2,4]]
    data=lineup.read_export(paths[1].read_text())
    if accepted['provenance']['native_export_sha256']!=NATIVE_SHA:
        raise ValueError('Unsupported lineup export provenance')
    if lineup.inventory(data['game'],data['person'],data['appearance'])!={k:v for k,v in accepted.items() if k!='provenance'}:
        raise ValueError('Full lineup source reconciliation failed')
    with paths[3].open(newline='',encoding='utf-8') as stream:
        reader=csv.DictReader(stream,strict=True)
        if reader.fieldnames is None or len(reader.fieldnames)!=len(FIELDS) or set(reader.fieldnames)!=FIELDS:
            raise ValueError('Invalid game input event header')
        rows=list(reader)
    if player.inventory(rows)!={k:v for k,v in report.items() if k!='provenance'}:
        raise ValueError('Full opportunity source reconciliation failed')
    source_games={g['game_id']:g for g in data['game']}
    if len(readiness['games'])!=len(source_games) or {g['game_id'] for g in readiness['games']}!=set(source_games):
        raise ValueError('Readiness game set reconciliation failed')
    for g in readiness['games']:
        if any(g[k]!=source_games[g['game_id']][k] for k in
                ['date','use_dh','park_id','away_team_id','home_team_id']):
            raise ValueError('Readiness game metadata reconciliation failed')
    return dict(data=data,report=report,rows=rows,readiness=readiness)


def prepare_season(source):
    """Reuse one unchanged full-cohort workload audit across game exports."""
    data=source['data']
    selected=sorted({a['player_id'] for a in data['appearance'] if a['fielding_position']==1})
    measured=workload.inventory(data['game'],data['person'],data['appearance'],source['rows'],selected)
    return dict(source,_workloads={p['player_id']:p for p in measured['players']})


def build_game(source,game_id):
    """Caller must use read_sources; no partial-season observations authorize cold profiles."""
    data=source['data'];games=data['game'];people=data['person'];appearances=data['appearance']
    target=next((g for g in games if g['game_id']==game_id),None)
    if target is None:raise ValueError('Unknown source game ID')
    result=next(g for g in source['readiness']['games'] if g['game_id']==game_id)['results']
    if (len(result)!=1 or any(result[0][k] is not False for k in ['forfeit','shortened','suspended','tied'])
            or type(result[0]['duration_outs']) is not int or result[0]['duration_outs']<=0):
        raise ValueError('Exceptional/unknown game result requires a separate replay policy')
    target_rows=[a for a in appearances if a['game_id']==game_id]
    witness=lineup.inventory([target],people,target_rows)['witnesses']['primary']
    if witness is None:raise ValueError('Unsupported historical starting lineup')
    selected={a['player_id'] for a in target_rows if a['fielding_position']==1}
    required_batters={p['player_id'] for t in witness['teams'] for p in t['batting_order']}
    if not witness['use_dh']:required_batters.update(selected)
    report=copy.deepcopy(source['report']);cold=[]
    for role,ids in [('batter',required_batters),('pitcher',selected)]:
        profiles={p['player_id']:p for p in report['roles'][role]['profiles']}
        for pid in sorted(ids):
            if pid not in profiles:
                # Authenticated full-cohort absence is zero observed events, not missing input data.
                profiles[pid]=dict(player_id=pid,events=0,games=0,selected_pa=0,non_pa=0,
                    included_pa=0,counts=dict.fromkeys(ratings.KEYS,0),zero_categories=9,
                    exclusions=dict(excluded_ibb=0,excluded_interference=0))
                report['roles'][role]['profiles'].append(profiles[pid])
            p=profiles[pid]
            if p['included_pa']==0:cold.append(dict(role=role,player_id=pid,events=p['events'],included_pa=0))
    support=bullpen.inventory([target],people,target_rows,report,target=game_id)
    if '_workloads' in source:
        measured=dict(format='2019-pitcher-workload-v1',basis=workload.BASIS,
            players=[copy.deepcopy(source['_workloads'][pid]) for pid in sorted(selected)])
    else:
        measured=workload.inventory(games,people,appearances,source['rows'],sorted(selected))
    profiles={p['player_id']:p for p in report['roles']['pitcher']['profiles']}
    for p in measured['players']:
        if p['ability_profile'] is None:
            if p['source_pa']!=0:raise ValueError('Unreconciled absent pitching profile')
            p['ability_profile']=copy.deepcopy(profiles[p['player_id']])
    measured['game_id']=game_id
    measured['provenance']=dict(snapshot_sha256=real.SNAPSHOT_SHA,supplement_sha256=real.SUPPLEMENT_SHA,
        opportunity_report_sha256=ratings.REPORT_SHA,lineup_report_sha256=real.REPORT_SHA,
        attribution=real.ATTRIBUTION,workload_recipe_sha256=cast.WORKLOAD_RECIPE_SHA,outcome_events_sha256=cast.EVENTS_SHA)
    return dict(game_id=game_id,use_dh=witness['use_dh'],cold_profiles=cold,
        cast_sizes={t['side']:len(t['observed_pitchers']) for t in support['teams']},
        lineup=real.encode_witness(witness,generic=True),
        cast=cast.encode(report,witness,support,measured,empirical=True,generic=True))


def _publish(bundle,inputs,pins,output):
    output=_destination(output)
    output.mkdir()  # A failed partial directory stays unaccepted and cannot be reused.
    receipt={k:v for k,v in bundle.items() if k not in ['lineup','cast']}
    receipt.update(format='historical-game-inputs-v1',input_sha256={str(p):h for p,h in zip(inputs,pins)},
        output_sha256={name+'.tsv':hashlib.sha256(bundle[name].encode()).hexdigest() for name in ['lineup','cast']},
        attribution=real.ATTRIBUTION,
        limits='controlled historical lineup/recorded closed cast; not daily eligibility, fatigue, pinch hitting, official season accounting or statistical fidelity')
    for name in ['lineup','cast']:
        ratings.publish(bundle[name],inputs,pins,output/(name+'.tsv'))
    ratings.publish(json.dumps(receipt,sort_keys=True,indent=2,allow_nan=False)+'\n',inputs,pins,output/'receipt.json')
    return receipt


def export(lineups,native,opportunities,events,readiness,game_id,output):
    output=_destination(output)
    paths=list(map(Path,[lineups,native,opportunities,events,readiness]))
    recipes=[Path(m.__file__) for m in [lineup,player,workload,bullpen,real,ratings,cast]]+[Path(__file__)]
    inputs=paths+recipes;pins=tuple(real.digest(p) for p in inputs)
    if pins[:5]!=INPUT_SHA or real.digest(Path(workload.__file__))!=cast.WORKLOAD_RECIPE_SHA:
        raise ValueError('Unsupported game input source/recipe digest')
    bundle=build_game(read_sources(paths),game_id)
    return _publish(bundle,inputs,pins,output)


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    for name in ['lineups','native','opportunities','events','readiness']:parser.add_argument(name,type=Path)
    parser.add_argument('game_id');parser.add_argument('output',type=Path)
    args=parser.parse_args()
    receipt=export(args.lineups,args.native,args.opportunities,args.events,args.readiness,args.game_id,args.output)
    print(json.dumps({k:receipt[k] for k in ['game_id','use_dh','cast_sizes','cold_profiles']},sort_keys=True))


if __name__=='__main__':main()
