"""Legal season-pool inputs; historical daily lineups are preferences, not eligibility."""
from preview import game_inputs as game
from preview import season_manager as manager
from preview import pitcher_policy_v2 as pitcher_v2
from preview import bench_inputs as bench

WORKLOAD_BASIS='source empirical role outings; explicitly marked pooled role priors when unavailable'


def _text(fields):
    return 'key\tvalue\n'+''.join(game.real.literal(k)+'\t'+game.real.literal(v)+'\n' for k,v in sorted(fields.items()))


def _vector(actor,role):
    counts=actor[role+'_counts']
    return game.ratings.count_vector(dict(counts=dict(zip(game.ratings.KEYS,counts)),included_pa=sum(counts)))


def _pitching(actor):
    # Incidental position-player pitching is not a primary bullpen assignment.
    return 1 in actor['positions'] and (actor['starts']>0 or
        not any(2<=p<=10 for p in actor['positions']) or
        (actor['targets']['pitcher'] or 0)>=(actor['targets']['batter'] or 0))


def rotations(pool):
    return {tid:sorted((pid for pid,a in t['players'].items() if a['starts'] and 1 in a['positions']),
        key=lambda pid:(-t['players'][pid]['starts'],pid)) for tid,t in pool['teams'].items()}


def _lineups(policy,g,starters,occupied):
    dh=g['use_dh'];choices={}
    for side in ['Away','Home']:
        tid=g[side.lower()+'_team_id'];actors=policy.pool['teams'][tid]['players']
        original=g.get('fielding_map_'+side.lower(),{}) or {}
        for pos in range(2,11 if dh else 10):
            choices[side,pos]=sorted((pid for pid,a in actors.items() if pid not in occupied and not _pitching(a) and pos in a['positions']),
                key=lambda pid:(-policy._pace(tid,pid,'batter')['preference'],pid!=original.get(str(pos)),
                    -(actors[pid]['targets']['batter'] or 0),pid))
    owners={}
    def assign(slot,seen):
        if slot in seen:return False
        seen.add(slot)
        # Prefer free actors before rerouting another position; polynomial matching, not exponential search.
        for pid in sorted(choices[slot],key=lambda p:p in owners):
            if pid not in owners or assign(owners[pid],seen):
                owners[pid]=slot;return True
        return False
    for slot in sorted(choices,key=lambda s:(len(choices[s]),s)):
        if not assign(slot,set()):raise ValueError('No complete supported positional lineup')
    selected={s:{1:starters[s]} for s in ['Away','Home']};orders={}
    for pid,(side,pos) in owners.items():selected[side][pos]=pid
    for side in selected:
        original=g.get('fielding_map_'+side.lower(),{}) or {};source_order=g.get('lineup_map_'+side.lower(),{}) or {};order=[]
        for slot in range(1,10):
            pid=source_order.get(str(slot));pos=next((int(p) for p,a in original.items() if a==pid and p.isdecimal()),None)
            if pos in selected[side] and (not dh or pos!=1) and pos not in order:order.append(pos)
        # Broken historical maps do not prevent a legal modeled lineup; no positions are invented.
        order.extend(p for p in sorted(selected[side]) if p not in order and p!=1)
        if not dh and 1 not in order:order.append(1)
        if len(order)!=9:raise ValueError('Invalid modeled batting order')
        orders[side]=order
    return selected,orders


def _v2_decision(choice):
    def candidate(row):
        return dict(player_id=row['player_id'],starts_target=row['starts_target'],
            starts_before=row['starts_before'],debt_numerator=row['debt'].numerator,
            debt_denominator=row['debt'].denominator,last_pitched=row['last_pitched'],
            excluded=row['excluded'],budget_scale=row['budget_scale'])
    return dict(player_id=choice['player_id'],budget_scale=choice['budget_scale'],
        reason=choice['reason'],candidates=[candidate(row) for row in choice['candidates']])


def _budgets(source):
    individual={};pooled={r:[] for r in ['start','relief']}
    for pid,p in sorted(source['_workloads'].items()):
        if p['issues']:continue
        for role in pooled:
            rows=[o for o in p['outing_rows'] if o['training'] and o['role']==role]
            samples=sorted(o['terminal_pa'] for o in rows)
            s=p[role]
            if (len({o['game_id'] for o in rows})!=len(rows) or len(samples)!=s['outings'] or sum(samples)!=s['pa_sum']
                    or any(type(n) is not int or n<0 for n in samples)):
                raise ValueError('Unreconciled managed role workload')
            individual[pid,role]=samples;pooled[role].extend(samples)
    for role,values in pooled.items():
        values.sort();n=min(101,len(values))
        pooled[role]=[values[(2*i+1)*len(values)//(2*n)] for i in range(n)]
    return individual,pooled


def build_game(source,pool,snapshot,game_id,rotation_turns,pitcher_policy=manager.POLICY):
    """Pure choice/export; caller authenticates sources/targets and persists only accepted turns."""
    if pitcher_policy not in {manager.POLICY,pitcher_v2.POLICY}:raise ValueError('Unsupported pitcher policy')
    v2=pitcher_policy==pitcher_v2.POLICY
    g=next((g for g in source['data']['game'] if g['game_id']==game_id),None)
    if g is None or type(g['use_dh']) is not bool:raise ValueError('Unsupported managed game/rules')
    policy=(pitcher_v2.Policy if v2 else manager.Policy)(pool,snapshot,g['date'])
    rotation=rotations(pool) if not v2 else None
    individual,pooled=source['_managed_budgets'] if '_managed_budgets' in source else _budgets(source)
    common=dict(season_policy=pitcher_v2.POLICY if v2 else manager.POLICY,availability_basis=manager.AVAILABILITY)
    lineup=dict(format='season-pool-lineup-v2' if v2 else 'season-pool-lineup-v1',game_id=game_id,date=g['date'],park_id=g['park_id'],
        use_dh=str(g['use_dh']).lower(),audit_sha256=game.real.REPORT_SHA,
        snapshot_sha256=game.real.SNAPSHOT_SHA,supplement_sha256=game.real.SUPPLEMENT_SHA,
        attribution=game.real.ATTRIBUTION,**common)
    cast=dict(format='season-pool-pitching-cast-v2' if v2 else 'season-pool-pitching-cast-v1',game_id=game_id,policy='empirical-pa-closed-cast-v2',
        model=game.ratings.MODEL,opportunity_report_sha256=game.ratings.REPORT_SHA,
        lineup_report_sha256=game.real.REPORT_SHA,snapshot_sha256=game.real.SNAPSHOT_SHA,
        supplement_sha256=game.real.SUPPLEMENT_SHA,attribution=game.real.ATTRIBUTION,
        batter_kappa='200',pitcher_kappa='200',league_counts=','.join(map(str,game.ratings.COUNTS)),
        opportunity_basis=game.ratings.BASIS,workload_basis=WORKLOAD_BASIS,exhaustion='last_pitcher_stays',
        workload_recipe_sha256=game.cast.WORKLOAD_RECIPE_SHA,outcome_events_sha256=game.cast.EVENTS_SHA,**common)
    bench_fields=dict(format='season-pool-bench-v2' if v2 else 'season-pool-bench-v1',game_id=game_id,policy=bench.POLICY,
        defense_basis=bench.DEFENSE_BASIS,snapshot_sha256=game.real.SNAPSHOT_SHA,
        supplement_sha256=game.real.SUPPLEMENT_SHA,opportunity_report_sha256=game.ratings.REPORT_SHA,
        attribution=game.real.ATTRIBUTION,**common)
    occupied=set();starters={};turns={} if v2 else dict(rotation_turns);decisions={};selected={};orders={}
    if v2:
        choices=policy.starters(g['away_team_id'],g['home_team_id'])
        for side in ['Away','Home']:
            choice=choices[side];starters[side]=choice['player_id'];occupied.add(choice['player_id'])
            decisions[side]=_v2_decision(choice);cast['starter_scale:'+side]=str(choice['budget_scale'])
    else:
        # Allocate the scarcer starter pool first so a shared actor cannot strand the other club.
        starter_sides=sorted(['Away','Home'],key=lambda side:sum(policy.rested(pid,starter=True)
            for pid in rotation[g[side.lower()+'_team_id']]))
        for side in starter_sides:
            tid=g[side.lower()+'_team_id'];r=rotation[tid];principal=r[:5];turn=rotation_turns.get(tid,0)
            if type(turn) is not int or not 0<=turn<len(principal):raise ValueError('Invalid principal rotation pointer')
            available=[pid for pid in principal if pid not in occupied and policy.rested(pid,starter=True)]
            if available:
                decision=policy.starter(tid,principal,turn,occupied=occupied)
            else:
                extras=r[5:]
                if not extras:raise ValueError('No rested supported starter: '+tid)
                decision=policy.starter(tid,extras,0,occupied=occupied);decision['reason']='rested_spot_start'
            pid=decision['player_id'];starters[side]=pid;occupied.add(pid)
            turns[tid]=(turn+1)%len(principal);decisions[tid]=decision
            cast['starter_scale:'+side]='80' if decision['budget_scale']==0.8 else '100'
    selected,orders=_lineups(policy,g,starters,occupied)
    occupied.update(pid for positions in selected.values() for pid in positions.values())
    for side in ['Away','Home']:
        tid=g[side.lower()+'_team_id'];actors=policy.pool['teams'][tid]['players']
        lineup[side.lower()+'_team_id']=tid;lineup[side.lower()+'_team']=g[side.lower()+'_team_name']
        slots={pos:i+1 for i,pos in enumerate(orders[side])}
        if g['use_dh']:slots[1]=0
        for pos,pid in selected[side].items():lineup[f'{side}:{slots[pos]}:{pos}:{pid}']=actors[pid]['name']
    bindings={};sizes={};cold=[];priors=[]
    for side in ['Away','Home']:
        tid=g[side.lower()+'_team_id'];actors=policy.pool['teams'][tid]['players'];starter=starters[side]
        if v2:
            relief_profiles={}
            for pid,a in actors.items():
                target=a['targets']['relief_outs'];work=source['_workloads'].get(pid,{})
                support=((target or 0)>0 or work.get('relief',{}).get('outings',0)>0
                    or (target is None and not a['starts']))
                if pid in occupied or not _pitching(a) or not support:continue
                samples=individual.get((pid,'relief'),[]) or pooled['relief']
                if not samples:raise ValueError('Missing empirical relief prior: '+pid)
                relief_profiles[pid]=(sum(samples),len(samples))
            relievers=policy.relievers(tid,relief_profiles,inning=1,score_gap=0,occupied=occupied)
            clock=snapshot['teams'][tid]
            cast['club_clock:'+side]=f"{clock['completed_games']},{clock['scheduled_games']}"
        else:
            principal=set(rotation[tid][:5])
            relief={pid:0 for pid,a in actors.items() if _pitching(a) and pid not in principal and
                ((a['targets']['relief_outs'] or 0)>0 or source['_workloads'].get(pid,{}).get('relief',{}).get('outings',0)>0
                    or (a['targets']['relief_outs'] is None and not a['starts']))}
            relievers=policy.relievers(tid,relief,occupied=occupied)
        pitchers=[starter,*relievers]
        if len(pitchers)>50:raise ValueError('Season pitching pool exceeds native bound')
        sizes[side]=len(pitchers);occupied.update(pitchers)
        batters={selected[side][pos] for pos in orders[side]}
        if not g['use_dh']:batters.update(pitchers)
        for pid in sorted(batters):cast['batter:'+pid]=_vector(actors[pid],'batter')
        for rank,pid in enumerate(pitchers):
            a=actors[pid];cast[f'cast:{side}:{rank}:{pid}']=a['name'];cast['pitcher:'+pid]=_vector(a,'pitcher')
            if v2:
                target=a['targets']['relief_outs'];used=policy._usage(tid,pid,'relief_outs')
                cast['relief_usage:'+pid]=f'{used},{target if target is not None else "?"}'
            else:cast['relief_pace:'+pid]=str(int(policy._pace(tid,pid,'relief')['preference']))
            for role in ['start','relief']:
                samples=individual.get((pid,role),[])
                if not samples and role==('start' if rank==0 else 'relief'):
                    samples=pooled[role]
                    if not samples:raise ValueError('Missing empirical role prior: '+role)
                    cast[f'budget_prior:{role}:{pid}']='true';priors.append(dict(player_id=pid,role=role))
                cast[f'workload:{role}:{pid}']=f'{sum(samples)},{len(samples)}'
                cast[f'usage:{role}:{pid}']=','.join(map(str,samples)) if samples else '-'
        reserves=[pid for pid,a in actors.items() if pid not in occupied and not _pitching(a)]
        if len(reserves)>50:raise ValueError('Season batting bench exceeds native bound')
        occupied.update(reserves)
        for pid in sorted(reserves):
            a=actors[pid];prefix=f'{side}:{pid}:';positions=[p for p in a['positions'] if 2<=p<=9]
            bench_fields[prefix+'name']=a['name'];bench_fields[prefix+'positions']=','.join(map(str,positions)) if positions else '?'
            bench_fields[prefix+'counts']=_vector(a,'batter');bench_fields[prefix+'pace']=str(int(policy._pace(tid,pid,'batter')['preference']))
        for role,ids in [('batter',batters|set(reserves)),('pitcher',set(pitchers))]:
            cold.extend(dict(role=role,player_id=pid) for pid in sorted(ids) if actors[pid]['cold_'+role])
        bindings[side]=dict(batter=sorted(batters|set(reserves)),pitcher=sorted(pitchers),
            fielder=sorted({pid for pos,pid in selected[side].items() if pos!=10}|set(pitchers)|set(reserves)))
    cast['cast_sizes']=','.join(str(sizes[s]) for s in ['Away','Home'])
    return dict(game_id=game_id,use_dh=g['use_dh'],lineup=_text(lineup),cast=_text(cast),bench=_text(bench_fields),
        role_actors=bindings,next_turns=turns,starter_decisions=decisions,cast_sizes=sizes,
        cold_profiles=sorted(cold,key=lambda r:(r['role'],r['player_id'])),budget_priors=priors)
