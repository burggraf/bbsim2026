"""Pure season-pool choices; not yet wired into native game or season execution."""
from collections import Counter
from copy import deepcopy
from datetime import date, timedelta
from preview import player_ratings as ratings
from preview.real_lineup import literal
from preview.usage_ledger import _validate_targets, _count, CORE

POLICY='season-pool-soft-usage-v1'
AVAILABILITY='observed season club players assumed available; no daily injury/minor-league reconstruction'


def build_pool(source,targets):
    """Caller authenticates source with game_inputs.read_sources and the approved targets.

    Entire-season affiliation/positions are intentional hindsight assumptions, not
    observation windows. A missing target stays unknown; absent rating observations
    in the authenticated full cohort use the existing zero-count league prior.
    """
    _validate_targets(targets)
    games={g['game_id']:g for g in source['data']['game']}
    historical={g['game_id']:g for g in targets['games']}
    if len(games)!=len(source['data']['game']) or set(games)!=set(historical):
        raise ValueError('Pool source/target schedule mismatch')
    if any(any(g[k]!=historical[gid][k] for k in CORE) for gid,g in games.items()):
        raise ValueError('Pool source/target game mismatch')
    people={p['player_id']:p for p in source['data']['person']}
    if len(people)!=len(source['data']['person']):raise ValueError('Duplicate pool identity')
    teams={tid:dict(scheduled_games=n,players={}) for tid,n in sorted(targets['team_games'].items())}
    def actor(tid,pid):
        if pid not in people:raise ValueError('Unsupported pool identity')
        literal(pid)
        p=people[pid];name=literal(p['first_name']+' '+p['last_name'])
        return teams[tid]['players'].setdefault(pid,dict(player_id=pid,name=name,positions=set(),
            targets=dict(batter=None,pitcher=None,start_outs=None,relief_outs=None),starts=0))
    for a in source['data']['appearance']:
        if a['game_id'] not in games or a['side'] not in {'Away','Home'}:raise ValueError('Invalid pool appearance')
        g=games[a['game_id']];tid=g['away_team_id' if a['side']=='Away' else 'home_team_id']
        p=actor(tid,a['player_id']);pos=a['fielding_position']
        if type(pos) is int and 1<=pos<=10:p['positions'].add(pos)
    sums={};starts=Counter()
    for g in targets['games']:
        for tid,pid in g['starting_pitcher_ids'].items():starts[tid,pid]+=1
    for r in targets['records']:
        tid,pid,role=r['team_id'],r['player_id'],r['role'];actor(tid,pid)
        keys=[role]
        if role=='pitcher':
            started=historical[r['game_id']]['starting_pitcher_ids'][tid]==pid
            keys+=['start_outs' if started else 'relief_outs']
        for key in keys:sums.setdefault((tid,pid,key),[]).append(r['usage'])
    profiles={}
    for role in ['batter','pitcher']:
        profiles[role]={}
        for p in source['report']['roles'][role]['profiles']:
            if p['player_id'] in profiles[role]:raise ValueError('Duplicate pool ability profile')
            profiles[role][p['player_id']]=list(map(int,ratings.count_vector(p).split(',')))
    for tid,t in teams.items():
        for pid,p in sorted(t['players'].items()):
            p['positions']=sorted(p['positions']);p['starts']=starts[tid,pid]
            for key in p['targets']:
                values=sums.get((tid,pid,key))
                # With a known pitching role, an absent split is a known zero.
                if values is None and key.endswith('_outs') and (tid,pid,'pitcher') in sums:values=[]
                p['targets'][key]=None if values is None or None in values else sum(values)
            for role in profiles:
                counts=profiles[role].get(pid)
                p[role+'_counts']=list(counts) if counts is not None else [0]*9
                p['cold_'+role]=counts is None or sum(counts)==0
        t['players']=dict(sorted(t['players'].items()))
    return dict(format=POLICY,availability_basis=AVAILABILITY,teams=teams)


def pace(usage,target,completed,scheduled,role):
    """Bounded [-1,1] preference, neutral in band; positive means underused."""
    for n in [usage,completed,scheduled]:_count(n)
    if not scheduled or completed>scheduled or role not in {'batter','starter','relief'}:
        raise ValueError('Invalid usage clock/role')
    if target is not None:_count(target)
    expected=target*completed/scheduled if target is not None else None
    allowance=max(12 if role=='batter' else 18 if role=='starter' else 6,0.2*(expected or 0))
    difference=expected-usage if expected is not None else 0
    preference=(max(-1.0,min(1.0,difference/allowance)) if abs(difference)>allowance else 0.0)
    return dict(usage=usage,expected=expected,allowance=allowance,preference=preference,unknown=target is None)


class Policy:
    """Read-only pregame policy over a complete simulated ledger, not real injury data.

    Returns choices to a future integration caller; it does not apply personnel,
    advance rotation pointers, spend targets, change ratings or generate outcomes.
    """
    def __init__(self,pool,snapshot,day):
        self.day=date.fromisoformat(day)
        if pool['format']!=POLICY or not snapshot['history_complete']:
            raise ValueError('Incomplete/unsupported manager context')
        self.pool=deepcopy(pool);self.snapshot=deepcopy(snapshot)
        for tid,team in self.pool['teams'].items():
            clock=self.snapshot['teams'][tid]
            if clock['scheduled_games']!=team['scheduled_games'] or not clock['history_complete']:
                raise ValueError('Mismatched/incomplete team usage clock')
            pace(0,None,clock['completed_games'],clock['scheduled_games'],'batter')
        for p in self.snapshot['players'].values():
            for w in p['workload']:
                if date.fromisoformat(w['date'])>self.day:raise ValueError('Future workload in pregame context')
                _count(w['usage'])
                if type(w['started']) is not bool:raise ValueError('Invalid workload role')

    def _actor(self,tid,pid):
        try:return self.pool['teams'][tid]['players'][pid]
        except KeyError as error:raise ValueError('Actor is not in season club pool') from error

    def _pace(self,tid,pid,role):
        p=self._actor(tid,pid);kind='batter' if role=='batter' else 'pitcher'
        a=self.snapshot['players'].get(kind+':'+pid,{})
        key='usage' if role=='batter' else 'start_outs' if role=='starter' else 'relief_outs'
        target_key='batter' if role=='batter' else key
        usage=a.get('teams',{}).get(tid,{}).get(key,0)
        clock=self.snapshot['teams'][tid]
        return pace(usage,p['targets'][target_key],clock['completed_games'],clock['scheduled_games'],role)

    def hitter(self,tid,incumbent,position,*,occupied=()):
        """One position-compatible swap; caller supplies all other selected IDs."""
        p=self._actor(tid,incumbent)
        if type(position) is not int or not 2<=position<=10 or position not in p['positions'] or incumbent in occupied:
            raise ValueError('Invalid incumbent/position')
        current=self._pace(tid,incumbent,'batter')['preference']
        choices=[pid for pid,p in self.pool['teams'][tid]['players'].items()
            if pid not in occupied and position in p['positions']]
        preferred=sorted(choices,key=lambda pid:(-self._pace(tid,pid,'batter')['preference'],pid))
        best=preferred[0]
        return best if self._pace(tid,best,'batter')['preference']>current else incumbent

    def rested(self,pid,*,starter):
        history=self.snapshot['players'].get('pitcher:'+pid,{}).get('workload',[])
        days={date.fromisoformat(w['date']) for w in history}
        if self.day in days:return False
        if all(self.day-timedelta(days=n) in days for n in [1,2,3]):return False
        for w in history:
            ago=(self.day-date.fromisoformat(w['date'])).days
            if w['started'] and ago<5:return False
        yesterday=sum(w['usage'] for w in history if w['date']==(self.day-timedelta(days=1)).isoformat())
        return yesterday<=6

    def _overused(self,tid,pid):
        p=self._pace(tid,pid,'starter')
        if p['unknown']:return False
        excess=p['usage']-p['expected']
        return excess>36 and excess>0.4*p['expected']

    def starter(self,tid,rotation,turn,*,occupied=()):
        if (not rotation or len(set(rotation))!=len(rotation) or type(turn) is not int or not 0<=turn<len(rotation)):
            raise ValueError('Invalid rotation/pointer')
        for pid in rotation:
            p=self._actor(tid,pid)
            if 1 not in p['positions'] or not p['starts']:raise ValueError('Unsupported starter role')
        normal=rotation[turn]
        order=rotation[turn:]+rotation[:turn]
        rested=[pid for pid in order if pid not in occupied and self.rested(pid,starter=True)]
        if not rested:raise ValueError('No rested supported starter')
        chosen=rested[0];reason='normal_rotation' if chosen==normal else 'rest'
        if chosen==normal and self._overused(tid,normal):
            alternate=next((pid for pid in rested[1:] if not self._overused(tid,pid)),None)
            if alternate is not None:chosen=alternate;reason='substantial_overuse'
        # Advance the scheduled turn, not the substitute's position, so skips cannot swallow later turns.
        return dict(player_id=chosen,next_turn=(turn+1)%len(rotation),reason=reason,
            budget_scale=0.8 if self._overused(tid,chosen) else 1.0)

    def relievers(self,tid,suitability,*,occupied=()):
        """Caller declares situational classes (lower better); pace only breaks class ties."""
        for pid,rank in suitability.items():
            p=self._actor(tid,pid);_count(rank)
            if 1 not in p['positions']:raise ValueError('Unsupported pitching position')
        return sorted((pid for pid in suitability if pid not in occupied and self.rested(pid,starter=False)),
            key=lambda pid:(suitability[pid],-self._pace(tid,pid,'relief')['preference'],pid))
