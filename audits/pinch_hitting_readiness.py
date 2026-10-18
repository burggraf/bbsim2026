"""Read-only same-game bench evidence; never pregame eligibility."""
import argparse
from collections import Counter,defaultdict
import copy
import json
from pathlib import Path
from preview import game_inputs as game

FORMAT='2019-pinch-hitting-readiness-v1'
LIMITS=('fielding-appearance source; PH-only/PR-only participation coverage unproven; '
        'same-game observed participant proxy, not an opening bench or active roster; '
        'unobserved availability, injury/rest and defensive eligibility remain unknown; '
        'no automatic pinch-hitting policy or statistical-fidelity claim')


def inventory(source):
    """Pure inventory; production callers must authenticate the complete cohort first."""
    data=source['data']
    validated=game.lineup.inventory(data['game'],data['person'],data['appearance'])
    supported=set(validated['candidate_game_ids'])
    people=defaultdict(list);by_actor=defaultdict(list);by_side=defaultdict(set)
    for p in data['person']:people[p['player_id']].append(p)
    for a in data['appearance']:
        by_actor[a['game_id'],a['player_id']].append(a)
        if a['side'] in ['Away','Home']:by_side[a['game_id'],a['side']].add(a['player_id'])
    profiles={}
    for p in source['report']['roles']['batter']['profiles']:
        if p['player_id'] in profiles:raise ValueError('Duplicate batter profile')
        game.ratings.count_vector(p)
        profiles[p['player_id']]=p
    readiness={g['game_id']:g['results'] for g in source['readiness']['games']}
    if set(readiness)!={g['game_id'] for g in data['game']}:raise ValueError('Readiness game set mismatch')
    games=[];teams={};modes={};members=defaultdict(set);reasons=Counter();counts=Counter(games=0,sides=0,observed_actors=0,participant_proxies=0,cold_proxies=0)
    def totals():return dict(games=0,observed_actors=0,participant_proxies=0,cold_proxies=0,opening_bench_available=None)
    for g in sorted(data['game'],key=lambda g:(g['date'] or '',g['game_id'])):
        gid=g['game_id'];sides=[]
        mode='dh' if g['use_dh'] is True else 'no_dh' if g['use_dh'] is False else 'unknown'
        mode_totals=modes.setdefault(mode,totals());mode_totals['games']+=1;counts['games']+=1
        for side in ['Away','Home']:
            prefix=side.lower();tid=g[prefix+'_team_id']
            start={g[prefix+'_starting_pitcher_id']}
            for key in ['lineup_map_','fielding_map_']:
                if isinstance(g[key+prefix],dict):start.update(g[key+prefix].values())
            start={p for p in start if game.lineup.nonempty(p)}
            actors=[];team=teams.setdefault(tid,totals());team['games']+=1;counts['sides']+=1
            for pid in sorted(by_side[gid,side]):
                rows=by_actor[gid,pid];issues=[]
                known_sides={a['side'] for a in rows if a['side'] in ['Away','Home']}
                if len(known_sides)>1:issues.append('cross_side_identity')
                if any(a['side'] not in ['Away','Home'] for a in rows):issues.append('unknown_side')
                if pid in start:issues.append('starting_player')
                if any(a['fielding_position']==1 for a in rows):issues.append('recorded_pitcher')
                if gid not in supported:issues.append('unsupported_starting_maps')
                bio=people[pid]
                if len(bio)!=1 or any(not game.lineup.nonempty(bio[0][k]) for k in ['first_name','last_name']):issues.append('person_lookup')
                if any(a['fielding_position'] is None or not 1<=a['fielding_position']<=12 for a in rows):issues.append('unknown_position')
                if any(a[k] is None for a in rows for k in ['start_event_id','end_event_id']):issues.append('unknown_range')
                if any(a['start_event_id'] is not None and a['end_event_id'] is not None and a['start_event_id']>a['end_event_id'] for a in rows):issues.append('reversed_range')
                profile=profiles.get(pid)
                batting=dict(status='cold_zero',included_pa=0,counts=[0]*9) if profile is None else dict(
                    status='profile_zero' if profile['included_pa']==0 else 'profile',included_pa=profile['included_pa'],
                    counts=[profile['counts'][k] for k in game.ratings.KEYS])
                actor=dict(player_id=pid,positions=sorted({a['fielding_position'] for a in rows},key=lambda x:-1 if x is None else x),
                    ranges=sorted([[a['fielding_position'],a['start_event_id'],a['end_event_id']] for a in rows],key=lambda xs:tuple(-1 if n is None else n for n in xs)),
                    reasons=sorted(issues),participant_proxy=not issues,batting=batting)
                actors.append(actor);members[pid].add(tid);reasons.update(issues)
                for aggregate in [team,mode_totals,counts]:
                    aggregate['observed_actors']+=1
                    aggregate['participant_proxies']+=int(not issues)
                    aggregate['cold_proxies']+=int(not issues and batting['included_pa']==0)
            sides.append(dict(side=side,team_id=tid,starting_ids=sorted(start),actors=actors,opening_bench_available=None))
        games.append(dict(game_id=gid,date=g['date'],use_dh=g['use_dh'],starting_maps_supported=gid in supported,
            results=copy.deepcopy(readiness[gid]),sides=sides))
    unassigned=sorted((dict(a) for a in data['appearance'] if a['side'] not in ['Away','Home']),key=lambda a:json.dumps(a,sort_keys=True))
    return dict(format=FORMAT,limits=LIMITS,opening_bench_available=None,counts=dict(sorted(counts.items())),
        position_appearance_counts=validated['coverage']['position_appearance_counts'],
        reason_actor_counts=dict(sorted(reasons.items())),games=games,teams=teams,dh_modes=modes,
        multi_team_player_ids=sorted(pid for pid,tids in members.items() if len(tids)>1),unassigned_appearances=unassigned)


def inventory_participants(source):
    """Add literal participation sets; unresolved defensive/roster evidence stays unknown."""
    report=inventory(source)
    report['format']='2019-pinch-hitting-participants-v2'
    report['limits']+='; terminal-batter union is not complete PH/PR coverage or an eligible bench'
    pitchers=defaultdict(set);fielding=defaultdict(set);people=defaultdict(list)
    for a in source['data']['appearance']:
        fielding[a['game_id'],a['player_id']].add(a['side'])
        if a['fielding_position']==1:pitchers[a['game_id'],a['player_id']].add(a['side'])
    for p in source['data']['person']:people[p['player_id']].append(p)
    profiles={p['player_id']:p for p in source['report']['roles']['batter']['profiles']}
    sides={(g['game_id'],s['side']):s for g in report['games'] for s in g['sides']}
    games={g['game_id']:g for g in report['games']}
    assigned=defaultdict(lambda:defaultdict(list));unassigned=[]
    counts=Counter(dict(terminal_rows=0,assigned_terminal_rows=0,unassigned_terminal_rows=0,non_pa_rows=0,
        fielding_only_ids=0,batting_only_ids=0,both_ids=0,union_ids=0))
    categories=Counter()
    for row in source['rows']:
        if not row['result']:
            counts['non_pa_rows']+=1
            continue
        category=game.player.GROUPS.get(row['result'])
        if category is None:raise ValueError('Unknown PA classification')
        category=category if category.startswith('excluded_') else 'included'
        counts['terminal_rows']+=1;categories[category]+=1
        gid=row['game_id'];known=pitchers[gid,row['pitcher_id']];issues=[]
        if gid not in games:issues.append('unknown_game')
        if not known:issues.append('missing_pitcher_side')
        if known-{'Away','Home'}:issues.append('unknown_pitcher_side')
        if {'Away','Home'}<=known:issues.append('cross_side_pitcher')
        event=dict(game_id=gid,event_key=int(row['event_key']),event_id=int(row['event_id']),
            batter_id=row['batter_id'],pitcher_id=row['pitcher_id'],category=category)
        if issues:
            unassigned.append(dict(event,reasons=sorted(issues)));counts['unassigned_terminal_rows']+=1
        else:
            side='Home' if known=={'Away'} else 'Away'
            assigned[gid,side][row['batter_id']].append(event);counts['assigned_terminal_rows']+=1
    batting_sides=defaultdict(set)
    for (gid,side),batters in assigned.items():
        for pid in batters:batting_sides[gid,pid].add(side)
    team_counts={};mode_counts={}
    for (gid,side),target in sides.items():
        actors={a['player_id']:a for a in target['actors']};batters=assigned[gid,side];f=set(actors);b=set(batters)
        comparison=dict(fielding_only_ids=sorted(f-b),batting_only_ids=sorted(b-f),both_ids=sorted(f&b),union_ids=sorted(f|b))
        target['participant_comparison']=comparison;target['terminal_batters']=[]
        for pid,events in sorted(batters.items()):
            issues=set(actors[pid]['reasons']) if pid in actors else {'missing_fielding_evidence'}
            observed=fielding[gid,pid]
            if pid in target['starting_ids']:issues.add('starting_player')
            if pitchers[gid,pid]:issues.add('recorded_pitcher')
            if observed-{'Away','Home'}:issues.add('unknown_side')
            if (observed|batting_sides[gid,pid])&({'Away','Home'}-{side}):issues.add('cross_side_identity')
            bio=people[pid]
            if len(bio)!=1 or any(not game.lineup.nonempty(bio[0][k]) for k in ['first_name','last_name']):issues.add('person_lookup')
            if not games[gid]['starting_maps_supported']:issues.add('unsupported_starting_maps')
            profile=profiles.get(pid)
            batting=dict(status='cold_zero',included_pa=0,counts=[0]*9) if profile is None else dict(
                status='profile_zero' if profile['included_pa']==0 else 'profile',included_pa=profile['included_pa'],
                counts=[profile['counts'][k] for k in game.ratings.KEYS])
            target['terminal_batters'].append(dict(player_id=pid,events=sorted(e['event_key'] for e in events),
                categories=dict(sorted(Counter(e['category'] for e in events).items())),reasons=sorted(issues),
                batting=batting,defensive_eligibility=None))
        mode='dh' if games[gid]['use_dh'] is True else 'no_dh' if games[gid]['use_dh'] is False else 'unknown'
        for aggregate in [counts,team_counts.setdefault(target['team_id'],Counter()),mode_counts.setdefault(mode,Counter())]:
            for key,ids in comparison.items():aggregate[key]+=len(ids)
    report.update(participant_counts=dict(sorted(counts.items())),terminal_categories=dict(sorted(categories.items())),
        participant_teams={k:dict(sorted(v.items())) for k,v in sorted(team_counts.items())},
        participant_dh_modes={k:dict(sorted(v.items())) for k,v in sorted(mode_counts.items())},
        unassigned_terminal_events=sorted(unassigned,key=lambda e:(e['game_id'],e['event_key'])))
    return report


def export(paths,output,*,participants=False):
    output=game._destination(output);paths=list(map(Path,paths))
    recipes=[Path(m.__file__) for m in [game.lineup,game.player,game.workload,game.bullpen,game.real,game.ratings,game.cast,game]]+[Path(__file__)]
    inputs=paths+recipes;pins=tuple(game.real.digest(p) for p in inputs)
    source=game.read_sources(paths)
    report=inventory_participants(source) if participants else inventory(source)
    report['provenance']=dict(input_sha256={str(p):h for p,h in zip(inputs,pins)},
        snapshot_sha256=game.real.SNAPSHOT_SHA,supplement_sha256=game.real.SUPPLEMENT_SHA,attribution=game.real.ATTRIBUTION)
    if game._destination(output)!=output:raise ValueError('Bench audit destination changed')
    game.ratings.publish(json.dumps(report,sort_keys=True,indent=2,allow_nan=False)+'\n',inputs,pins,output)
    return report


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    for name in ['lineups','native','opportunities','events','readiness','output']:parser.add_argument(name,type=Path)
    parser.add_argument('--participants',action='store_true',help='Compare cached terminal batters with fielding participants; not eligibility')
    args=parser.parse_args()
    result=export([args.lineups,args.native,args.opportunities,args.events,args.readiness],args.output,participants=args.participants)
    print(json.dumps(result['counts'],sort_keys=True))


if __name__=='__main__':main()
