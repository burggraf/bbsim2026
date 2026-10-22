"""Controlled historical-lineup batch; not a managed or certified historical season."""
import argparse
from collections import Counter
from datetime import date
import hashlib
import json
from pathlib import Path
import re
import subprocess
import unicodedata
from preview import game_inputs as game
from preview import bench_inputs as bench

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


def _v2_actor_bindings(bundle):
    from preview import bench_inputs as bench_policy, managed_inputs, pitcher_policy_v2 as pitcher_v2, real_lineup, season_manager

    def read(name):
        text=bundle.get(name)
        if not isinstance(text,str):raise ValueError('Missing v2 personnel file: '+name)
        if (name=='bench' and len(text.encode('utf-8'))>65536) or any(
                unicodedata.category(c)=='Cc' and c not in '\n\t' for c in text):
            raise ValueError('Invalid v2 personnel controls/size')
        lines=text.splitlines()
        if not lines or lines[0]!='key\tvalue':raise ValueError('Invalid v2 personnel header: '+name)
        fields={}
        for line in lines[1:]:
            cells=line.split('\t')
            if len(cells)!=2 or not all(cells) or cells[0] in fields:raise ValueError('Invalid/duplicate v2 personnel row')
            fields[cells[0]]=cells[1]
        return fields

    def number(value,limit=2**64-1):
        if not re.fullmatch(r'0|[1-9][0-9]*',value) or int(value)>limit:
            raise ValueError('Invalid v2 personnel count')
        return int(value)

    def vector(value):
        values=value.split(',')
        if len(values)!=9:return False
        return all(number(n)<=maximum for n,maximum in zip(values,game.ratings.COUNTS))

    def pair(value,unknown=False,limit=2**32-1):
        cells=value.split(',')
        if len(cells)!=2:raise ValueError('Invalid v2 personnel pair')
        first=number(cells[0],limit)
        second=None if unknown and cells[1]=='?' else number(cells[1],limit)
        return first,second

    lineup=read('lineup');cast=read('cast');bench=read('bench')
    lineup_meta={'format','game_id','date','park_id','away_team','home_team','away_team_id','home_team_id',
        'use_dh','audit_sha256','snapshot_sha256','supplement_sha256','attribution','season_policy','availability_basis'}
    cast_meta={'format','policy','game_id','cast_sizes','season_policy','availability_basis','starter_scale:Away',
        'starter_scale:Home','workload_recipe_sha256','outcome_events_sha256','attribution','batter_kappa',
        'pitcher_kappa','league_counts','lineup_report_sha256','model','opportunity_basis',
        'opportunity_report_sha256','snapshot_sha256','supplement_sha256','workload_basis','exhaustion'}
    bench_meta={'format','game_id','policy','availability_basis','defense_basis','snapshot_sha256',
        'supplement_sha256','opportunity_report_sha256','attribution','season_policy'}
    common=dict(season_policy=pitcher_v2.POLICY,availability_basis=season_manager.AVAILABILITY)
    if any(lineup.get(k)!=v for k,v in dict(format='season-pool-lineup-v2',**common,
            audit_sha256=real_lineup.REPORT_SHA,snapshot_sha256=real_lineup.SNAPSHOT_SHA,
            supplement_sha256=real_lineup.SUPPLEMENT_SHA,attribution=real_lineup.ATTRIBUTION).items()):
        raise ValueError('Mismatched v2 lineup metadata')
    if any(cast.get(k)!=v for k,v in dict(format='season-pool-pitching-cast-v2',
            policy='empirical-pa-closed-cast-v2',**common,
            workload_recipe_sha256=game.cast.WORKLOAD_RECIPE_SHA,
            outcome_events_sha256=game.cast.EVENTS_SHA,attribution=real_lineup.ATTRIBUTION,
            batter_kappa='200',pitcher_kappa='200',league_counts=','.join(map(str,game.ratings.COUNTS)),
            lineup_report_sha256=real_lineup.REPORT_SHA,model=game.ratings.MODEL,
            opportunity_basis=game.ratings.BASIS,
            opportunity_report_sha256=game.ratings.REPORT_SHA,
            snapshot_sha256=real_lineup.SNAPSHOT_SHA,supplement_sha256=real_lineup.SUPPLEMENT_SHA,
            workload_basis=managed_inputs.WORKLOAD_BASIS,exhaustion='last_pitcher_stays').items()):
        raise ValueError('Mismatched v2 cast metadata')
    if any(bench.get(k)!=v for k,v in dict(format='season-pool-bench-v2',game_id=lineup.get('game_id'),
            policy=bench_policy.POLICY,**common,defense_basis=bench_policy.DEFENSE_BASIS,
            snapshot_sha256=real_lineup.SNAPSHOT_SHA,supplement_sha256=real_lineup.SUPPLEMENT_SHA,
            opportunity_report_sha256=game.ratings.REPORT_SHA,
            attribution=real_lineup.ATTRIBUTION).items()):
        raise ValueError('Mismatched v2 bench metadata')
    if set(lineup)&lineup_meta!=lineup_meta or set(cast)&cast_meta!=cast_meta or set(bench)&bench_meta!=bench_meta:
        raise ValueError('Missing v2 personnel metadata')
    game_id=lineup['game_id'];away_id=lineup['away_team_id'];home_id=lineup['home_team_id']
    game_date=date.fromisoformat(lineup['date'])
    if (not re.fullmatch(r'[A-Z]{3}2019[0-9]{4}[012]',game_id) or game_id[:3]!=home_id
            or away_id==home_id or game_date.isoformat()!=lineup['date']
            or game_date.strftime('%Y%m%d')!=game_id[3:11] or lineup['use_dh'] not in {'true','false'}):
        raise ValueError('Invalid v2 lineup identity')
    for team_id in [away_id,home_id]:pitcher_v2.scheduled_games(team_id)
    if (cast['game_id']!=game_id or bench['game_id']!=game_id
            or cast['lineup_report_sha256']!=lineup['audit_sha256']
            or len({lineup['snapshot_sha256'],cast['snapshot_sha256'],bench['snapshot_sha256']})!=1
            or len({lineup['supplement_sha256'],cast['supplement_sha256'],bench['supplement_sha256']})!=1):
        raise ValueError('Mismatched v2 personnel source/game binding')

    line_players={'Away':[],'Home':[]};line_ids=set();line_names={}
    for key,value in lineup.items():
        if key in lineup_meta:continue
        parts=key.split(':')
        if (len(parts)!=4 or parts[0] not in line_players or not re.fullmatch(r'[a-z0-9-]+',parts[3])
                or value.strip()!=value):raise ValueError('Unknown v2 lineup actor')
        slot=number(parts[1],9);position=number(parts[2],10);pid=parts[3]
        if (position==0 or pid in line_ids or (lineup['use_dh']=='true' and ((slot==0)!=(position==1)))
                or (lineup['use_dh']=='false' and slot==0)):
            raise ValueError('Invalid v2 lineup actor')
        line_ids.add(pid);line_names[pid]=value.strip();line_players[parts[0]].append((slot,position,pid))
    dh=lineup['use_dh']=='true'
    expected_slots=set(range(10)) if dh else set(range(1,10))
    expected_positions=set(range(1,11)) if dh else set(range(1,10))
    for side,players in line_players.items():
        if ({slot for slot,_,_ in players}!=expected_slots or {pos for _,pos,_ in players}!=expected_positions
                or len(players)!=len(expected_slots)):
            raise ValueError('Invalid v2 lineup grain')
    starters={side:next(pid for _,pos,pid in players if pos==1) for side,players in line_players.items()}
    lineup_batters={side:{pid for slot,_,pid in players if slot!=0} for side,players in line_players.items()}

    cast_rows={'Away':[],'Home':[]};cast_names={};pitcher_profiles=set();batter_profiles=set();workloads={};usage={};
    relief_usage=set();clocks={};budget_priors=set()
    for key,value in cast.items():
        if key in cast_meta:continue
        parts=key.split(':')
        if parts[0]=='cast' and len(parts)==4 and parts[1] in cast_rows:
            rank=number(parts[2],49);pid=parts[3]
            if not re.fullmatch(r'[a-z0-9-]+',pid) or value.strip()!=value:raise ValueError('Invalid v2 cast identity')
            cast_rows[parts[1]].append((rank,pid));cast_names[pid]=value.strip()
        elif parts[0] in {'pitcher','batter'} and len(parts)==2:
            if not re.fullmatch(r'[a-z0-9-]+',parts[1]) or not vector(value):raise ValueError('Invalid v2 ability profile')
            (pitcher_profiles if parts[0]=='pitcher' else batter_profiles).add(parts[1])
        elif parts[0] in {'workload','usage'} and len(parts)==3 and parts[1] in {'start','relief'}:
            pid=parts[2]
            if not re.fullmatch(r'[a-z0-9-]+',pid):raise ValueError('Invalid v2 workload actor')
            k=(parts[1],pid)
            if parts[0]=='workload':
                pa,outings=pair(value,limit=2**64-1)
                if pa>2**52 or outings>2429 or (outings==0 and pa):raise ValueError('Invalid v2 workload')
                workloads[k]=(pa,outings)
            else:
                samples=[] if value=='-' else [number(n) for n in value.split(',')]
                if samples!=sorted(samples):raise ValueError('Unsorted v2 workload samples')
                usage[k]=samples
        elif parts[0]=='budget_prior' and len(parts)==3 and parts[1] in {'start','relief'}:
            if value!='true':raise ValueError('Invalid v2 budget prior')
            budget_priors.add((parts[1],parts[2]))
        elif parts[0]=='club_clock' and len(parts)==2 and parts[1] in {'Away','Home'}:
            clocks[parts[1]]=pair(value,unknown=False)
        elif parts[0]=='relief_usage' and len(parts)==2:
            pair(value,unknown=True);relief_usage.add(parts[1])
        else:raise ValueError('Unknown v2 cast field')
    sizes=[number(n,50) for n in cast['cast_sizes'].split(',')]
    if len(sizes)!=2 or not all(sizes) or any(cast[f'starter_scale:{side}'] not in {'80','100'} for side in cast_rows):
        raise ValueError('Invalid v2 cast sizes/starter scale')
    cast_ids={}
    for side,index in [('Away',0),('Home',1)]:
        rows=sorted(cast_rows[side])
        if len(rows)!=sizes[index] or [rank for rank,_ in rows]!=list(range(sizes[index])):
            raise ValueError('Incomplete v2 cast rows')
        cast_ids[side]={pid for _,pid in rows}
        if len(cast_ids[side])!=len(rows):raise ValueError('Duplicate v2 cast identity')
    all_cast=cast_ids['Away']|cast_ids['Home']
    if any(cast_names[pid]!=line_names[pid] for pid in all_cast&line_ids):
        raise ValueError('V2 lineup/cast display-name mismatch')
    expected_workloads={(role,pid) for pid in all_cast for role in ('start','relief')}
    if (cast_ids['Away']&cast_ids['Home'] or pitcher_profiles!=all_cast or relief_usage!=all_cast
            or set(workloads)!=expected_workloads or set(usage)!=expected_workloads
            or not budget_priors<=expected_workloads or len(clocks)!=2):
        raise ValueError('Inconsistent v2 cast actor profiles')
    for side,tid in [('Away',away_id),('Home',home_id)]:
        completed,scheduled=clocks[side]
        if completed>=scheduled or scheduled!=pitcher_v2.scheduled_games(tid):
            raise ValueError('Invalid v2 club clock binding')
        if next(pid for rank,pid in sorted(cast_rows[side]) if rank==0)!=starters[side]:
            raise ValueError('V2 starter does not match lineup')
        other_side='Home' if side=='Away' else 'Away'
        other_line_ids={pid for _,_,pid in line_players[other_side]}
        if (any(pid in line_ids and pid!=starters[side] for pid in cast_ids[side])
                or cast_ids[side]&other_line_ids):
            raise ValueError('V2 cast conflicts with lineup identities')
        if workloads['start',starters[side]][1]==0:
            raise ValueError('Missing v2 starter workload')
    for k,(pa,outings) in workloads.items():
        samples=usage[k]
        if len(samples)!=outings or sum(samples)!=pa:raise ValueError('V2 workload samples do not reconcile')
    expected_batters=set().union(*lineup_batters.values())
    if not dh:expected_batters|=all_cast
    if batter_profiles!=expected_batters:raise ValueError('V2 batter profiles do not match lineup/cast')

    bench_records={};bench_ids={'Away':set(),'Home':set()}
    for key,value in bench.items():
        if key in bench_meta:continue
        parts=key.split(':')
        if (len(parts)!=3 or parts[0] not in bench_ids or not re.fullmatch(r'[a-z0-9-]+',parts[1])
                or parts[2] not in {'name','positions','counts','pace'}):
            raise ValueError('Unknown v2 bench actor')
        record=bench_records.setdefault((parts[0],parts[1]),{})
        if parts[2] in record:raise ValueError('Duplicate v2 bench actor field')
        record[parts[2]]=value
    for (side,pid),record in bench_records.items():
        if set(record)!={'name','positions','counts','pace'} or record['name'].strip()!=record['name'] or not vector(record['counts']) or record['pace'] not in {'-1','0','1'}:
            raise ValueError('Invalid v2 bench actor')
        if record['positions']!='?':
            positions=[number(n,9) for n in record['positions'].split(',')]
            if any(n<2 for n in positions) or positions!=sorted(set(positions)):raise ValueError('Invalid v2 bench positions')
        bench_ids[side].add(pid)
    if (bench_ids['Away']&bench_ids['Home'] or set().union(*bench_ids.values())&(line_ids|all_cast)):
        raise ValueError('Conflicting v2 bench identity')

    result={}
    for side in ['Away','Home']:
        bats=set(lineup_batters[side])|(cast_ids[side] if not dh else set())|bench_ids[side]
        fielders={pid for _,position,pid in line_players[side] if position!=10}|cast_ids[side]|bench_ids[side]
        result[side]=dict(batter=sorted(bats),pitcher=sorted(cast_ids[side]),fielder=sorted(fielders))
    if ({pid for roles in result.values() for ids in roles.values() for pid in ids}
            !=set().union(*lineup_batters.values())|all_cast|set().union(*bench_ids.values())):
        raise ValueError('Inconsistent v2 actor bindings')
    return result


def actor_bindings(bundle):
    markers={('format','season-pool-lineup-v2'),('format','season-pool-pitching-cast-v2'),
        ('format','season-pool-bench-v2'),('season_policy','season-pool-pitcher-usage-v2')}
    for name in ('lineup','cast','bench'):
        text=bundle.get(name,'')
        if not isinstance(text,str):continue
        fields=(line.partition('\t') for line in text.splitlines())
        if (any(separator and (key,value) in markers for key,separator,value in fields)
                or any(line.startswith(('club_clock:','relief_usage:')) for line in text.splitlines())):
            return _v2_actor_bindings(bundle)
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
    if 'bench' in bundle:
        fields={};records={}
        lines=bundle['bench'].splitlines()
        if not lines or lines[0]!='key\tvalue':raise ValueError('Invalid bench header')
        managed='format\tseason-pool-bench-v1\n' in bundle['bench']
        metadata={'format','game_id','policy','availability_basis','defense_basis',
            'snapshot_sha256','supplement_sha256','opportunity_report_sha256','attribution'}
        actor_fields={'name','positions','counts'}
        expected=dict(format='assumed-game-bench-v1',policy=bench.POLICY,
            availability_basis=bench.AVAILABILITY,defense_basis=bench.DEFENSE_BASIS)
        if managed:
            from preview import season_manager as manager
            metadata.add('season_policy');actor_fields.add('pace')
            expected.update(format='season-pool-bench-v1',season_policy=manager.POLICY,availability_basis=manager.AVAILABILITY)
            for key,fmt in [('lineup','season-pool-lineup-v1'),('cast','season-pool-pitching-cast-v1')]:
                f=dict(line.split('\t') for line in bundle[key].splitlines()[1:])
                if any(f.get(k)!=v for k,v in dict(format=fmt,season_policy=manager.POLICY,
                        availability_basis=manager.AVAILABILITY).items()):raise ValueError('Mixed managed personnel modes')
        for line in lines[1:]:
            cells=line.split('\t')
            if len(cells)!=2 or not all(cells) or cells[0] in fields:raise ValueError('Invalid/duplicate bench field')
            key,value=cells;fields[key]=value
            if key in metadata:continue
            parts=key.split(':')
            if (len(parts)!=3 or parts[0] not in result or not re.fullmatch(r'[a-z0-9-]{1,64}',parts[1])
                    or parts[2] not in actor_fields):raise ValueError('Unknown bench actor role')
            records.setdefault((parts[0],parts[1]),{})[parts[2]]=value
        lineup_game=next((line.split('\t',1)[1] for line in bundle['lineup'].splitlines() if line.startswith('game_id\t')),None)
        if fields.get('game_id')!=lineup_game or lineup_game is None:raise ValueError('Mismatched bench game')
        for key,value in expected.items():
            if fields.get(key)!=value:raise ValueError('Mismatched bench metadata')
        occupied={pid for roles in result.values() for ids in roles.values() for pid in ids}
        for (side,pid),record in records.items():
            if set(record)!=actor_fields or pid in occupied:raise ValueError('Conflicting/incomplete bench actor')
            if managed and record['pace'] not in {'-1','0','1'}:raise ValueError('Invalid bench usage preference')
            occupied.add(pid)
            result[side]['batter'].append(pid);result[side]['fielder'].append(pid)
        for roles in result.values():
            for ids in roles.values():ids.sort()
    away={pid for ids in result['Away'].values() for pid in ids}
    home={pid for ids in result['Home'].values() for pid in ids}
    if away&home:raise ValueError('Cross-side actor binding')
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
    expected_format=entry.get('accounting_format','controlled-accounting-v1')
    ph_metadata=dict(pinch_hitting=bench.POLICY,availability_basis=bench.AVAILABILITY,defense_basis=bench.DEFENSE_BASIS)
    expected_keys={'format','game','seed'}|VERSIONS
    if expected_format=='controlled-accounting-ph-v1':expected_keys|=ph_metadata.keys()
    elif expected_format in {'controlled-accounting-managed-v1','controlled-accounting-managed-v2'}:
        from preview import pitcher_policy_v2, season_manager as manager
        ph=entry.get('pinch_hitting','late-close-expected-bases-ph-usage-v1')
        if ph not in {'disabled','late-close-expected-bases-ph-usage-v1'}:raise ValueError('Unknown managed PH policy')
        v2=expected_format=='controlled-accounting-managed-v2'
        ph_metadata.update(pinch_hitting=ph,availability_basis=manager.AVAILABILITY,
            season_policy=pitcher_policy_v2.POLICY if v2 else manager.POLICY,
            manager=pitcher_policy_v2.MANAGER if v2 else 'season-pool-workload-context-2019-v1')
        expected_keys|=ph_metadata.keys()
    elif expected_format!='controlled-accounting-v1':raise ValueError('Unknown accounting schema')
    ids=dict(Away=entry['away_team_id'],Home=entry['home_team_id'])
    bindings=entry['role_actors']
    if (not isinstance(bindings,dict) or set(bindings)!=set(ids)
            or any(not isinstance(roles,dict) or set(roles)!=set(METRICS) for roles in bindings.values())
            or any(not isinstance(actors,list) or any(not isinstance(pid,str) or not pid for pid in actors)
                or len(actors)!=len(set(actors))
                for roles in bindings.values() for actors in roles.values())
            or {pid for actors in bindings['Away'].values() for pid in actors}&
               {pid for actors in bindings['Home'].values() for pid in actors}):
        raise ValueError('Invalid accounting actor declarations')
    for line in text.splitlines():
        parts=line.split('\t');kind=parts[0]
        if kind in expected_keys:
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
    if (set(metadata)!=expected_keys or metadata['format']!=expected_format
            or any(metadata.get(k)!=v for k,v in ph_metadata.items() if k in expected_keys)
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
    return dict(teams=teams,players=players,versions={k:metadata[k] for k in sorted(VERSIONS)},accounting=metadata)


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
    report=dict(format='controlled-season-v1',simulation_policy='regular-full-game-v1',master_seed=master_seed,seed_algorithm='sha256:bbsim-season-v1:first8-big-endian',
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
