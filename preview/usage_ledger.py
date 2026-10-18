"""Observational canonical usage; no roster inference, personnel policy or storage."""
from collections import Counter
from copy import deepcopy
from datetime import date
import hashlib
import json
import re
from preview import season_runner as season

FORMAT='observational-season-usage-ledger-v1'
TARGET_FORMAT='2019-season-usage-targets-v1'
LIMITS=('Historical-lineup reconstruction, not verified roster/stint eligibility. '
        'Dates are modeled game-start dates, including suspended games; observed appearances '
        'are not transaction, injury or availability boundaries. Recent workload is not measured fatigue. '
        'Explicit refusals advance observation only and leave workload/pace history incomplete; not policy context.')
CORE=('game_id','date','away_team_id','home_team_id')
BANDS={'batter':('0','1-99','100-299','300+','unknown'),
       'pitcher':('0','1-59','60-179','180+','unknown')}


def _count(value):
    if type(value) is not int or value<0:raise ValueError('Invalid nonnegative usage count')
    return value


def _identity(g):
    row={k:g[k] for k in CORE}
    season.game_seed(0,row['game_id'])
    if (not isinstance(row['date'],str) or date.fromisoformat(row['date']).isoformat()!=row['date']
            or row['date'].replace('-','')!=row['game_id'][3:11]
            or any(not isinstance(row[k],str) or not re.fullmatch('[A-Z]{3}',row[k]) for k in CORE[2:])
            or row['away_team_id']==row['home_team_id'] or row['game_id'][:3]!=row['home_team_id']):
        raise ValueError('Invalid schedule identity/date')
    return row


def _total(values):
    values=list(values)
    return None if None in values else sum(values)


def _canonical(value):
    return json.dumps(value,sort_keys=True,separators=(',',':'),allow_nan=False).encode()


def _verify_result(entry,result):
    """Reuse the wire parser's role, optional-cell and conservation checks at acceptance."""
    metadata=result['accounting']
    lines=[f'{key}\t{value}' for key,value in metadata.items()]
    for side,tid in [('Away',entry['away_team_id']),('Home',entry['home_team_id'])]:
        stats=result['teams'][tid]
        if any(n is not None and type(n) is not int for n in stats.values()):raise ValueError('Invalid team totals')
        lines.append(f'team\t{side}\t{tid}\t{stats["runs"]}')
    sides={entry['away_team_id']:'Away',entry['home_team_id']:'Home'}
    for p in result['players']:
        cells=['?' if p['stats'][k] is None else str(p['stats'][k]) for k in season.METRICS[p['role']]]
        lines.append('\t'.join([p['role'],sides[p['team_id']],p['player_id'],*cells]))
    verified=season.parse_accounting('\n'.join(lines)+'\n',entry)
    if verified!=result:raise ValueError('Mismatched parsed accounting totals/metadata')
    away=result['teams'][entry['away_team_id']];home=result['teams'][entry['home_team_id']]
    if (home['outs']<27 or home['outs']%3 or
            (away['runs']>home['runs'] and away['outs']!=home['outs']) or
            (home['runs']>away['runs'] and not home['outs']-3<=away['outs']<=home['outs'])):
        raise ValueError('Partial/nonregular completed game')


class Ledger:
    """Pass the entire actual played schedule in manifest order, including refused slots.

    accept requires selected starting_batter_ids/starting_pitcher_ids keyed by team,
    role_actors keyed by native side, and parse_accounting's unchanged result. The
    receipt hash binds externally verified raw bytes; this class does not read files.
    Explicit refusals permit later observational collection, never complete policy context.
    """
    def __init__(self,games):
        try:
            self._games=[_identity(g) for g in games]
            order=[(g['date'],g['game_id']) for g in self._games]
            if order!=sorted(order) or len({g['game_id'] for g in self._games})!=len(order):
                raise ValueError('Duplicate/nonchronological schedule')
        except (KeyError,TypeError,AttributeError) as error:raise ValueError('Invalid schedule') from error
        self._accepted={}
        self._refusals={}
        self._versions=None

    def _check_next(self,entry):
        position=len(self._accepted)+len(self._refusals)
        if position>=len(self._games) or _identity(entry)!=self._games[position]:
            raise ValueError('Expected next chronological schedule entry')

    def record_refusal(self,entry,reason):
        """Observe an explicit missing game, never count its usage or supply policy context."""
        try:
            if not isinstance(reason,str) or not reason.strip():raise ValueError('Missing refusal reason')
            payload=hashlib.sha256(_canonical([entry,reason])).hexdigest()
            gid=entry['game_id']
            if gid in self._accepted:raise ValueError('Cannot refuse an accepted game')
            if gid in self._refusals:
                if self._refusals[gid]['payload']!=payload:raise ValueError('Altered recorded refusal')
                return False
            self._check_next(entry)
            season.game_seed(entry['seed'],gid)
            refused=deepcopy(dict(entry=entry,reason=reason,payload=payload))
        except (KeyError,TypeError,AttributeError,OverflowError) as error:raise ValueError('Invalid refusal payload') from error
        self._refusals[gid]=refused
        return True

    def accept(self,entry,result,receipt_sha256):
        """True for a new complete game, False for an identical replay; otherwise refuse atomically."""
        try:
            if not isinstance(receipt_sha256,str) or not re.fullmatch('[0-9a-f]{64}',receipt_sha256):
                raise ValueError('Invalid receipt SHA-256')
            payload=hashlib.sha256(_canonical([entry,result,receipt_sha256])).hexdigest()
            gid=entry['game_id']
            if gid in self._accepted:
                if self._accepted[gid]['payload']!=payload:raise ValueError('Altered accepted game/receipt')
                return False
            if gid in self._refusals:raise ValueError('Cannot accept a recorded refusal')
            self._check_next(entry)
            season.game_seed(entry['seed'],gid)
            if entry.get('status','pending') not in {'pending','complete'}:raise ValueError('Refused/failed game')
            _verify_result(entry,result)
            if self._versions is not None and self._versions!=result['versions']:raise ValueError('Accounting version drift')
            teams={entry[k] for k in CORE[2:]}
            if set(entry['starting_batter_ids'])!=teams or set(entry['starting_pitcher_ids'])!=teams:
                raise ValueError('Missing/misbound selected starters')
            for side,tid in [('Away',entry['away_team_id']),('Home',entry['home_team_id'])]:
                batters=entry['starting_batter_ids'][tid];pitcher=entry['starting_pitcher_ids'][tid]
                if (not isinstance(batters,list) or not batters or len(batters)!=len(set(batters))
                        or not set(batters)<=set(entry['role_actors'][side]['batter'])
                        or pitcher not in entry['role_actors'][side]['pitcher']):
                    raise ValueError('Invalid selected starters')
                for role,ids in [('batter',batters),('pitcher',[pitcher])]:
                    appeared={p['player_id'] for p in result['players'] if p['team_id']==tid and p['role']==role}
                    if not set(ids)<=appeared:raise ValueError('Missing starting actor accounting')
            accepted=deepcopy(dict(entry=entry,result=result,receipt_sha256=receipt_sha256,payload=payload))
        except (KeyError,TypeError,AttributeError,OverflowError) as error:raise ValueError('Invalid usage acceptance payload') from error
        self._accepted[gid]=accepted
        self._versions=deepcopy(result['versions'])
        return True

    def snapshot(self):
        counts=Counter(tid for g in self._games for tid in [g['away_team_id'],g['home_team_id']])
        teams={tid:dict(scheduled_games=n,completed_games=0,observed_games=0,refused_games=0,
                       history_complete=True,pa=0,outs=0,bf=0,progress=0)
               for tid,n in sorted(counts.items())}
        for refused in self._refusals.values():
            for key in CORE[2:]:
                t=teams[refused['entry'][key]];t['observed_games']+=1;t['refused_games']+=1;t['history_complete']=False
        players={}
        for accepted in self._accepted.values():
            entry=accepted['entry'];result=accepted['result']
            for tid,stats in result['teams'].items():
                t=teams[tid];t['completed_games']+=1;t['observed_games']+=1
                for key in ['pa','outs','bf']:t[key]+=stats[key]
                t['progress']=t['completed_games']/t['scheduled_games']
            for p in result['players']:
                role=p['role']
                if role not in {'batter','pitcher'}:continue
                pid=p['player_id'];tid=p['team_id'];usage=p['stats']['pa' if role=='batter' else 'outs']
                started=(pid in entry['starting_batter_ids'][tid] if role=='batter' else pid==entry['starting_pitcher_ids'][tid])
                actor=players.setdefault(role+':'+pid,dict(role=role,player_id=pid,usage=0,appearances=0,starts=0,bf=0,teams={},workload=[]))
                split=actor['teams'].setdefault(tid,dict(usage=0,appearances=0,starts=0,bf=0))
                for target in [actor,split]:
                    target['usage']+=usage;target['appearances']+=1;target['starts']+=int(started)
                    if role=='pitcher':
                        target['bf']+=p['stats']['bf']
                        target['start_outs']=target.get('start_outs',0)+(usage if started else 0)
                        target['relief_outs']=target.get('relief_outs',0)+(0 if started else usage)
                actor['workload'].append(dict(game_id=entry['game_id'],date=entry['date'],team_id=tid,
                    usage=usage,bf=p['stats'].get('bf'),started=started))
        ids=list(self._accepted);position=len(ids)+len(self._refusals)
        return dict(format=FORMAT,completed_game_ids=ids,
            observed_game_ids=[g['game_id'] for g in self._games[:position]],
            observed_schedule_complete=bool(self._games) and position==len(self._games),
            history_complete=not self._refusals,
            refused_games=[dict(_identity(r['entry']),reason=r['reason'],payload_sha256=r['payload']) for r in self._refusals.values()],
            next_game_id=self._games[position]['game_id'] if position<len(self._games) else None,
            complete_schedule=bool(self._games) and len(ids)==len(self._games),versions=deepcopy(self._versions),teams=teams,players=players,
            receipt_sha256={gid:a['receipt_sha256'] for gid,a in self._accepted.items()},
            payload_sha256={gid:a['payload'] for gid,a in self._accepted.items()},limits=LIMITS)

    def compare(self,targets):
        """Frozen descriptive criteria on accepted-game targets only; missing is never zero."""
        try:
            _validate_targets(targets)
            games={g['game_id']:g for g in targets['games']}
            for gid,a in self._accepted.items():
                if gid not in games or _identity(games[gid])!=_identity(a['entry']):raise ValueError('Missing/mismatched comparison game')
            return self._compare(targets)
        except (KeyError,TypeError,AttributeError) as error:raise ValueError('Invalid historical usage targets') from error

    def _compare(self,targets):
        actual={};sim={};matched=set(self._accepted)
        target_games={g['game_id']:g for g in targets['games']}
        for r in targets['records']:
            if r['game_id'] not in matched:continue
            key=r['role'],r['team_id'],r['player_id']
            a=actual.setdefault(key,dict(values=[],unknown_rows=0,source_rows=0,start_outs=[],relief_outs=[]))
            a['values'].append(r['usage']);a['unknown_rows']+=r['unknown_rows'];a['source_rows']+=r['source_rows']
            if r['role']=='pitcher':
                started=r['player_id']==target_games[r['game_id']]['starting_pitcher_ids'][r['team_id']]
                a['start_outs' if started else 'relief_outs'].append(r['usage'])
        snap=self.snapshot()
        for actor in snap['players'].values():
            for tid,split in actor['teams'].items():sim[(actor['role'],tid,actor['player_id'])]=split
        actual_starts=Counter((tid,pid) for g in targets['games'] if g['game_id'] in matched
            for tid,pid in g['starting_pitcher_ids'].items())
        rows=[]
        for role,tid,pid in sorted(actual.keys()|sim.keys()):
            key=role,tid,pid;a=actual.get(key);s=sim.get(key)
            observed=_total(a['values']) if a else None;modeled=s['usage'] if s else None
            absolute,fraction=(20,0.25) if role=='batter' else (18,0.30)
            allowance=max(absolute,fraction*observed) if observed is not None else None
            delta=modeled-observed if observed is not None and modeled is not None else None
            rows.append(dict(role=role,team_id=tid,player_id=pid,actual=observed,simulated=modeled,
                deviation=delta,relative_deviation=delta/observed if delta is not None and observed else None,
                allowance=allowance,within_band=abs(delta)<=allowance if delta is not None else None,
                actual_present=a is not None,simulated_present=s is not None,
                unknown_rows=a['unknown_rows'] if a else None,source_rows=a['source_rows'] if a else None,
                actual_starts=actual_starts[(tid,pid)] if role=='pitcher' and a else None,
                actual_start_outs=_total(a['start_outs']) if role=='pitcher' and a else None,
                actual_relief_outs=_total(a['relief_outs']) if role=='pitcher' and a else None,
                simulated_start_outs=s.get('start_outs') if s else None,
                simulated_relief_outs=s.get('relief_outs') if s else None,
                simulated_starts=s['starts'] if s else None,simulated_appearances=s['appearances'] if s else None))
        headline={};bands={};aggregate={}
        for role,minimum,metric in [('batter',100,'pa'),('pitcher',60,'outs')]:
            role_rows=[r for r in rows if r['role']==role]
            eligible=[r for r in role_rows if r['actual'] is not None and r['actual']>=minimum]
            within=sum(r['within_band'] is True for r in eligible)
            headline[role]=dict(minimum_actual=minimum,required_fraction=0.8,
                absolute_allowance=20 if role=='batter' else 18,relative_allowance=0.25 if role=='batter' else 0.30,
                count=len(eligible),within=within,
                fraction=within/len(eligible) if eligible else None,passed=within/len(eligible)>=0.8 if eligible else None)
            bands[role]={band:dict(count=0,compared=0,within=0) for band in BANDS[role]}
            for r in role_rows:
                n=r['actual'];band=('unknown' if n is None else '0' if n==0 else
                    BANDS[role][1] if n<minimum else BANDS[role][2] if n<minimum*3 else BANDS[role][3])
                b=bands[role][band];b['count']+=1;b['compared']+=int(r['deviation'] is not None);b['within']+=int(r['within_band'] is True)
            values=[r['usage'] for r in targets['records'] if r['game_id'] in matched and r['role']==role]
            observed=_total(values) if matched else None
            modeled=sum(t[metric] for t in snap['teams'].values()) if matched else None
            delta=modeled-observed if observed is not None else None
            aggregate[metric]=dict(actual=observed,known_subtotal=sum(n for n in values if n is not None),simulated=modeled,
                deviation=delta,relative_deviation=delta/observed if delta is not None and observed else None,
                relative_bound=0.05,passed=abs(delta)<=0.05*observed if delta is not None else None)
        player_totals=[]
        for role,pid in sorted({(r['role'],r['player_id']) for r in rows}):
            grouped=[r for r in rows if r['role']==role and r['player_id']==pid]
            a=_total(r['actual'] for r in grouped);s=_total(r['simulated'] for r in grouped)
            player_totals.append(dict(role=role,player_id=pid,actual=a,simulated=s,deviation=s-a if a is not None and s is not None else None))
        return dict(format='season-usage-comparison-v1',matched_game_ids=list(self._accepted),
            history_complete=not self._refusals,refused_game_ids=list(self._refusals),
            full_target_schedule=matched=={g['game_id'] for g in targets['games']},
            aggregate=aggregate,headline=headline,bands=bands,player_team=rows,players=player_totals,
            coverage=dict(matched_games=len(matched),target_games=len(targets['games']),player_team_records=len(rows),
                refused_games=len(self._refusals),observed_games=len(matched)+len(self._refusals),
                actual_only=sum(r['actual_present'] and not r['simulated_present'] for r in rows),
                sim_only=sum(r['simulated_present'] and not r['actual_present'] for r in rows),
                unknown_actual=sum(r['actual'] is None for r in rows),compared=sum(r['deviation'] is not None for r in rows),
                source_rows=sum(r['source_rows'] for r in rows if r['source_rows'] is not None),
                unknown_rows=sum(r['unknown_rows'] for r in rows if r['unknown_rows'] is not None)),
            team_games=deepcopy(targets['team_games']),
            provenance=deepcopy(targets['provenance']),warnings=deepcopy(targets['warnings']),limits=LIMITS)


def _validate_targets(targets):
    if targets['format']!=TARGET_FORMAT:raise ValueError('Unknown historical targets schema')
    games={};counts=Counter()
    for g in targets['games']:
        row=_identity(g);gid=row['game_id']
        if gid in games:raise ValueError('Duplicate historical game')
        games[gid]=row
        for key in CORE[2:]:counts[row[key]]+=1
        teams={row[k] for k in CORE[2:]}
        if (set(g['starting_pitcher_ids'])!=teams or any(not isinstance(pid,str) or not pid for pid in g['starting_pitcher_ids'].values())
                or type(g['suspended']) is not bool or type(g['shortened']) is not bool):raise ValueError('Invalid historical game metadata')
        _count(g['duration_outs'])
    if targets['team_games']!=dict(counts) or any(type(n) is not int for n in targets['team_games'].values()):
        raise ValueError('Mismatched actual played denominator')
    seen=set();coverage=set();values={'pa':[],'outs':[]}
    for r in targets['records']:
        gid,tid,pid,role=(r[k] for k in ['game_id','team_id','player_id','role'])
        if (gid not in games or tid not in [games[gid][k] for k in CORE[2:]] or role not in {'batter','pitcher'}
                or not isinstance(pid,str) or not pid):raise ValueError('Invalid historical usage binding')
        key=gid,tid,pid,role
        if key in seen:raise ValueError('Duplicate historical usage record')
        seen.add(key);coverage.add((gid,tid,role))
        source=_count(r['source_rows']);unknown=_count(r['unknown_rows'])
        if not source or unknown>source or (r['usage'] is None)!=(unknown>0):raise ValueError('Invalid historical unknown coverage')
        if r['usage'] is not None:_count(r['usage'])
        values['pa' if role=='batter' else 'outs'].append(r['usage'])
    expected={(gid,g[k],role) for gid,g in games.items() for k in CORE[2:] for role in ['batter','pitcher']}
    if coverage!=expected:raise ValueError('Missing historical game/team/role coverage')
    for g in targets['games']:
        for tid,pid in g['starting_pitcher_ids'].items():
            if (g['game_id'],tid,pid,'pitcher') not in seen:raise ValueError('Missing historical starter usage')
    for metric,counts in values.items():
        if targets['totals'][metric] is not None:_count(targets['totals'][metric])
        if targets['totals'][metric]!=_total(counts):raise ValueError('Mismatched historical usage totals')
    if not isinstance(targets['provenance'],dict) or not isinstance(targets['warnings'],list):raise ValueError('Invalid target provenance/warnings')
