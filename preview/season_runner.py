"""Controlled historical-lineup batch; not a managed or certified historical season."""
import argparse
from collections import Counter
from datetime import date
import hashlib
import json
from pathlib import Path
import re
import subprocess
from preview import game_inputs as game

METRICS={
    'batter':('pa','ab','r','h','rbi','bb','so','single','double','triple','hr','ibb','hbp','sh','sf','gdp','sb','cs','ci'),
    'pitcher':('bf','outs','ab','r','er','h','bb','so','single','double','triple','hr','ibb','hbp','sh','sf','wp','balk'),
    'fielder':('events','po','a','e','pb'),
}
OPTIONAL={
    'batter':{'ab','h','rbi','single','double','triple','hr','ibb'},
    'pitcher':{'ab','er','h','single','double','triple','hr','ibb'},
    'fielder':{'po','a','e','pb'},
}
VERSIONS={'engine','mechanics','ratings','manager','rng','rules'}


def game_seed(master_seed,game_id):
    if type(master_seed) is not int or not 0<=master_seed<2**64:
        raise ValueError('Master seed must be an unsigned 64-bit integer')
    if not isinstance(game_id,str) or not re.fullmatch(r'[A-Z]{3}2019[0-9]{4}[012]',game_id):
        raise ValueError('Invalid season game ID')
    date.fromisoformat(f'{game_id[3:7]}-{game_id[7:9]}-{game_id[9:11]}')
    return int.from_bytes(hashlib.sha256(f'bbsim-season-v1:{master_seed}:{game_id}'.encode('ascii')).digest()[:8],'big')


def manifest(source,master_seed):
    result=[];seen=set()
    for g in sorted(source['data']['game'],key=lambda g:(g['date'],g['game_id'])):
        if g['game_id'] in seen:raise ValueError('Invalid duplicate season game ID')
        seen.add(g['game_id'])
        row={k:g[k] for k in ['game_id','date','away_team_id','home_team_id']}
        row.update(seed=game_seed(master_seed,g['game_id']),status='pending')
        result.append(row)
    return result


def actor_bindings(bundle):
    lineup=[line.split('\t')[0].split(':') for line in bundle['lineup'].splitlines() if line.startswith(('Away:','Home:'))]
    keys=[line.split('\t')[0].split(':') for line in bundle['cast'].splitlines()]
    batting={k[1] for k in keys if k[0]=='batter'}
    result={}
    for side in ['Away','Home']:
        pitchers={k[-1] for k in keys if k[0]=='cast' and k[1]==side}
        starting={k[3] for k in lineup if k[0]==side}
        fielders={k[3] for k in lineup if k[0]==side and k[2]!='10'}
        result[side]=dict(batter=sorted(batting&(starting|pitchers)),
            pitcher=sorted(pitchers),fielder=sorted(fielders|pitchers))
    return result


def _number(value):
    if not re.fullmatch(r'0|[1-9][0-9]*',value) or int(value)>2**32-1:
        raise ValueError('Invalid accounting count')
    return int(value)


def _sum(values):
    values=list(values)
    return None if None in values else sum(values)


def parse_accounting(text,entry):
    metadata={};teams={};players=[];seen=set()
    ids=dict(Away=entry['away_team_id'],Home=entry['home_team_id'])
    for line in text.splitlines():
        parts=line.split('\t');kind=parts[0]
        if kind in {'format','game','seed'}|VERSIONS:
            if len(parts)!=2 or not parts[1] or kind in metadata:raise ValueError('Invalid accounting metadata')
            metadata[kind]=parts[1]
        elif kind=='team':
            if len(parts)!=4 or parts[1] not in ids or parts[2]!=ids[parts[1]] or parts[2] in teams:
                raise ValueError('Invalid accounting team binding')
            teams[parts[2]]=dict(runs=_number(parts[3]))
        elif kind in METRICS:
            if len(parts)!=3+len(METRICS[kind]) or parts[1] not in ids:
                raise ValueError('Invalid accounting role row')
            _,side,pid,*cells=parts;key=kind,side,pid
            if key in seen or pid not in entry['role_actors'][side][kind]:raise ValueError('Duplicate/unknown accounting role actor')
            seen.add(key)
            stats={k:None if v=='?' and k in OPTIONAL[kind] else _number(v) for k,v in zip(METRICS[kind],cells)}
            players.append(dict(role=kind,team_id=ids[side],player_id=pid,stats=stats))
        else:raise ValueError('Unknown accounting row')
    if (set(metadata)!={'format','game','seed'}|VERSIONS or metadata['format']!='controlled-accounting-v1'
            or metadata['game']!=entry['game_id'] or metadata['seed']!=str(entry['seed'])
            or metadata['rules']!='2019' or set(teams)!=set(ids.values())):
        raise ValueError('Incomplete/mismatched accounting identity')
    if teams[ids['Away']]['runs']==teams[ids['Home']]['runs']:raise ValueError('Tied/incomplete accounting game')
    for side in ['Away','Home']:
        tid=ids[side];opponent=ids['Home' if side=='Away' else 'Away']
        grouped={r:[p['stats'] for p in players if p['team_id']==tid and p['role']==r] for r in METRICS}
        if any(not values for values in grouped.values()):raise ValueError('Missing accounting role')
        opposing=[p['stats'] for p in players if p['team_id']==opponent and p['role']=='pitcher']
        total=lambda role,key:_sum(p[key] for p in grouped[role])
        checks=[(total('batter','r'),teams[tid]['runs']),
                (total('pitcher','r'),teams[opponent]['runs']),
                (total('pitcher','outs'),total('fielder','po'))]
        checks.extend((total('batter',a),_sum(p[b] for p in opposing)) for a,b in
            [('pa','bf')]+[(k,k) for k in ['ab','h','bb','so','single','double','triple','hr','ibb','hbp','sh','sf']])
        for a,b in checks:
            if a is not None and b is not None and a!=b:raise ValueError('Accounting conservation failed')
        for role,rows in grouped.items():
            for p in rows:
                if role=='batter' and p['ab'] is not None:
                    if p['pa']!=sum(p[k] for k in ['ab','bb','hbp','sh','sf','ci']):raise ValueError('Accounting PA formula failed')
                    if p['h'] is not None and p['h']+p['so']>p['ab']:raise ValueError('Accounting AB credits exceeded')
                if role!='fielder' and p['ibb'] is not None and p['ibb']>p['bb']:raise ValueError('Accounting IBB credits exceeded')
                if role=='pitcher' and p['er'] is not None and p['er']>p['r']:raise ValueError('Accounting ER credits exceeded')
                if role!='fielder' and p['h'] is not None:
                    hits=_sum(p[k] for k in ['single','double','triple','hr'])
                    if hits is not None and p['h']!=hits:raise ValueError('Accounting hit conservation failed')
        teams[tid].update(games=1,wins=int(teams[tid]['runs']>teams[opponent]['runs']),
            losses=int(teams[tid]['runs']<teams[opponent]['runs']),runs_allowed=teams[opponent]['runs'],
            pa=total('batter','pa'),bf=total('pitcher','bf'),outs=total('pitcher','outs'),h=total('batter','h'))
    return dict(teams=teams,players=players,versions={k:metadata[k] for k in sorted(VERSIONS)})


def accumulate(results):
    teams={};players={}
    def add(target,values):
        for k,n in values.items():target[k]=None if n is None or target.get(k,0) is None else target.get(k,0)+n
    for result in results:
        for tid,stats in result['teams'].items():add(teams.setdefault(tid,{}),stats)
        for p in result['players']:
            actor=players.setdefault(p['role']+':'+p['player_id'],dict(role=p['role'],player_id=p['player_id'],stats={},teams={}))
            add(actor['stats'],p['stats']);add(actor['teams'].setdefault(p['team_id'],{}),p['stats'])
    return dict(teams=teams,players=players)


def run(source,inputs,pins,binary,baseline,contact,master_seed,output):
    output=game._destination(output);rows=manifest(source,master_seed)
    output.mkdir();(output/'games').mkdir()
    artifacts={};results=[]
    def pin(path):artifacts[str(path.relative_to(output))]=game.real.digest(path)
    def publish(name,text):
        path=output/name;game.ratings.publish(text,(),(),path);pin(path)
    for row in rows:
        try:bundle=game.build_game(source,row['game_id'])
        except ValueError as error:
            row.update(status='input_refused',reason=str(error));continue
        folder=output/'games'/row['game_id'];folder.mkdir()
        for name in ['lineup','cast']:publish(str((folder/(name+'.tsv')).relative_to(output)),bundle[name])
        row['cold_profiles']=bundle['cold_profiles'];row['cast_sizes']=bundle['cast_sizes']
        row['role_actors']=actor_bindings(bundle)
        command=[str(Path(binary).resolve()),'--baseline',str(Path(baseline).resolve()),'--contact-counts',str(Path(contact).resolve()),
            '--lineup',str(folder/'lineup.tsv'),'--cast',str(folder/'cast.tsv'),'--seed',str(row['seed']),'--format','accounting']
        try:
            with (folder/'stdout.tsv').open('xb') as out,(folder/'stderr.txt').open('xb') as err:
                result=subprocess.run(command,stdout=out,stderr=err,timeout=30,check=False)
            row['exit']=result.returncode
            if result.returncode:raise ValueError(f'Native game exited {result.returncode}; see raw stderr')
            parsed=parse_accounting((folder/'stdout.tsv').read_text(),row)
            if results and parsed['versions']!=results[0]['versions']:raise ValueError('Native model versions changed')
            results.append(parsed);row.update(status='complete',teams=parsed['teams'])
        except (ValueError,subprocess.TimeoutExpired) as error:
            row.update(status='simulation_failed',reason=str(error))
        for name in ['stdout.tsv','stderr.txt']:pin(folder/name)
    summary=accumulate(results)
    report=dict(format='controlled-season-v1',master_seed=master_seed,seed_algorithm='sha256:bbsim-season-v1:first8-big-endian',
        games=rows,counts=dict(sorted(Counter(r['status'] for r in rows).items())),schedule=source['readiness']['rows'],
        complete_season=bool(rows) and all(r['status']=='complete' for r in rows),
        versions=results[0]['versions'] if results else None,
        limits='controlled source lineups/recorded closed casts; not verified eligibility, rest, fatigue, pinch hitting, full historical season, official statistics or statistical fidelity')
    for name,value in [('manifest.json',{k:v for k,v in report.items() if k not in ['schedule']}),
                       ('schedule.json',report['schedule']),('summary.json',summary)]:
        publish(name,json.dumps(value,sort_keys=True,indent=2,allow_nan=False)+'\n')
    if tuple(game.real.digest(p) for p in inputs)!=tuple(pins):raise ValueError('Season inputs changed before acceptance')
    if any(game.real.digest(output/p)!=h for p,h in artifacts.items()):raise ValueError('Season output changed before acceptance')
    receipt=dict(format='controlled-season-receipt-v1',input_sha256={str(p):h for p,h in zip(inputs,pins)},
        output_sha256=artifacts,counts=report['counts'],complete_season=report['complete_season'],attribution=game.real.ATTRIBUTION)
    game.ratings.publish(json.dumps(receipt,sort_keys=True,indent=2,allow_nan=False)+'\n',inputs,tuple(pins),output/'receipt.json')
    return report


def export(lineups,native,opportunities,events,readiness,baseline,contact,binary,master_seed,output):
    output=game._destination(output)
    paths=list(map(Path,[lineups,native,opportunities,events,readiness]))
    recipes=[Path(m.__file__) for m in [game.lineup,game.player,game.workload,game.bullpen,game.real,game.ratings,game.cast,game]]+[Path(__file__)]
    inputs=paths+list(map(Path,[baseline,contact,binary]))+recipes
    pins=tuple(game.real.digest(p) for p in inputs)
    if pins[:5]!=game.INPUT_SHA or game.real.digest(Path(game.workload.__file__))!=game.cast.WORKLOAD_RECIPE_SHA:
        raise ValueError('Unsupported season input source/recipe digest')
    source=game.prepare_season(game.read_sources(paths))
    return run(source,inputs,pins,Path(binary),Path(baseline),Path(contact),master_seed,output)


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    for name in ['lineups','native','opportunities','events','readiness','baseline','contact','binary','output']:
        parser.add_argument(name,type=Path)
    parser.add_argument('--seed',type=int,default=1)
    args=parser.parse_args()
    result=export(args.lineups,args.native,args.opportunities,args.events,args.readiness,args.baseline,args.contact,args.binary,args.seed,args.output)
    print(json.dumps(dict(counts=result['counts'],complete_season=result['complete_season']),sort_keys=True))
    return 0 if result['complete_season'] else 2


if __name__=='__main__':raise SystemExit(main())
