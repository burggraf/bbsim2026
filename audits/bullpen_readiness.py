"""Read-only pitching-cast support; observed appearances never certify eligibility."""
import argparse
from collections import defaultdict
import copy
import json
from pathlib import Path
import subprocess
from audits import lineup_readiness as lineup
from audits.raw_mechanisms import unique_object, SOURCE_COMMIT
from audits.outcome_inventory import CATALOG, digest
from audits.joint_contact import CATEGORIES

TARGET='OAK201903200'
REPORT_SHA='382be5a84b3e95687e6356b00d3b1b68522f41a5c2a6a065daf876a5a4c6881e'
BASIS='literal terminal-event participants; not official credits or PA/BF'


def inventory(games,people,appearances,opportunities,target=TARGET):
    source=lineup.inventory(games,people,appearances)
    if target not in source['candidate_game_ids']:
        raise ValueError('Unsupported target lineup')
    if (opportunities['format']!='2019-player-opportunity-support-v1'
            or set(opportunities['counts'])!=set(CATEGORIES)
            or any(type(n) is not int or n<0 for n in opportunities['counts'].values())):
        raise ValueError('Unsupported opportunity format/counts')
    profiles={}
    for p in opportunities['roles']['pitcher']['profiles']:
        player=p['player_id'];counts=p['counts'];exclusions=p['exclusions']
        if (not lineup.nonempty(player) or player in profiles
                or set(counts)!=set(CATEGORIES) or set(exclusions)!={'excluded_ibb','excluded_interference'}
                or any(type(v) is not int or v<0 for v in [p['events'],p['selected_pa'],p['non_pa'],p['included_pa'],p['games'],*counts.values(),*exclusions.values()])
                or any(counts[k]>opportunities['counts'][k] for k in CATEGORIES)
                or sum(counts.values())!=p['included_pa']
                or p['selected_pa']!=p['included_pa']+sum(exclusions.values())
                or p['events']!=p['selected_pa']+p['non_pa']):
            raise ValueError('Duplicate/invalid pitcher profile conservation')
        profiles[player]=p
    by_game={g['game_id']:g for g in games};g=by_game[target]
    bios=defaultdict(list);team_ids=defaultdict(set)
    for p in people:bios[p['player_id']].append(p)
    for a in appearances:
        if a['side'] in lineup.SIDES:
            team_ids[a['player_id']].add(by_game[a['game_id']][a['side'].lower()+'_team_id'])
    target_rows=[a for a in appearances if a['game_id']==target]
    witness=lineup.inventory([g],people,target_rows)['witnesses']['primary']
    def facts(player,rows,pitching=True):
        persons=bios[player];bio=persons[0] if len(persons)==1 else None
        known=bio is not None and lineup.nonempty(bio['first_name']) and lineup.nonempty(bio['last_name'])
        profile=profiles.get(player);issues=[]
        if not known:issues.append('person_lookup')
        if pitching and profile is None:issues.append('missing_pitcher_profile')
        return dict(player_id=player,name=bio['first_name']+' '+bio['last_name'] if known else None,
            bats=bio['bats'] if bio else None,throws=bio['throws'] if bio else None,
            appearances=sorted((dict(a) for a in rows),key=lambda a:(a['game_id'],a['fielding_position'] if a['fielding_position'] is not None else -1,a['start_event_id'] if a['start_event_id'] is not None else -1)),
            profile=copy.deepcopy(profile),issues=issues,eligible_at_start=None)
    teams=[]
    for start in witness['teams']:
        side=start['side'];team_id=start['team_id'];starter=start['pitcher_id']
        rows=[a for a in target_rows if a['side']==side];cast=defaultdict(list);nonstarting=defaultdict(list);season=defaultdict(list)
        starting={p['player_id'] for p in start['fielding']}
        for a in rows:
            if a['fielding_position']==1:cast[a['player_id']].append(a)
            if a['player_id'] not in starting:nonstarting[a['player_id']].append(a)
        for a in appearances:
            if (a['fielding_position']==1 and a['side'] in lineup.SIDES
                    and by_game[a['game_id']][a['side'].lower()+'_team_id']==team_id):
                season[a['player_id']].append(a)
        members=[]
        for player,records in sorted(season.items()):
            row=facts(player,records);ids=sorted({a['game_id'] for a in records});partition=dict(before=0,on=0,after=0,unknown=0)
            for gid in ids:
                day=by_game[gid]['date']
                key='unknown' if not lineup.known_date(day) else 'before' if day<g['date'] else 'after' if day>g['date'] else 'on'
                partition[key]+=1
            row.update(pitching_games=len(ids),game_ids=ids,date_partition=partition,
                observed_team_ids=sorted(team_ids[player],key=repr),target_game_pitcher=player in cast)
            members.append(row)
        teams.append(dict(side=side,team_id=team_id,team_name=start['team_name'],starter_id=starter,
            observed_pitchers=[facts(p,r) for p,r in sorted(cast.items())],
            observed_nonstarter_pitcher_ids=sorted(set(cast)-{starter}),
            nonstarting_participants=[facts(p,r,any(a['fielding_position']==1 for a in r)) for p,r in sorted(nonstarting.items())],season_pitchers=members))
    return dict(format='2019-bullpen-readiness-v1',game_id=target,date=g['date'],use_dh=g['use_dh'],
        pregame_bullpen_status='unsupported',membership_basis=lineup.MEMBERSHIP_BASIS,opportunity_basis=BASIS,
        missing_inputs=['pregame active roster/bench declaration','transactions/availability','measured workload/fatigue'],
        source_coverage=source['coverage'],target_appearance_rows=len(target_rows),
        target_unknown_fields=dict(side=sum(a['side'] not in lineup.SIDES for a in target_rows),
            position=sum(a['fielding_position'] is None or a['fielding_position']==0 or a['fielding_position']>12 for a in target_rows),
            start=sum(a['start_event_id'] is None for a in target_rows),end=sum(a['end_event_id'] is None for a in target_rows),
            reversed_range=sum(a['start_event_id'] is not None and a['end_event_id'] is not None and a['start_event_id']>a['end_event_id'] for a in target_rows)),teams=teams)


def write_report(report,inputs,pins,output):
    text=json.dumps(report,sort_keys=True,indent=2)+'\n'
    if tuple(digest(p) for p in inputs)!=pins:raise ValueError('Bullpen audit inputs changed before publication')
    with output.open('x',encoding='utf-8') as stream:stream.write(text)


def publish(snapshot,supplement,opportunities,output):
    raw=output.with_suffix('.native.csv');errors=output.with_suffix('.native.stderr')
    if any(p.exists() for p in [output,raw,errors]):raise ValueError('Refusing to overwrite report/native evidence')
    if output.resolve().is_relative_to(Path(__file__).resolve().parents[1]):raise ValueError('Audit output must be outside repository')
    query=Path(lineup.__file__).with_name('2019-lineup-readiness.sql')
    inputs=(snapshot,supplement,opportunities,query,Path(__file__))
    pins=tuple(digest(p) for p in inputs)
    if pins[:3]!=(*lineup.INPUT_SHA,REPORT_SHA):raise ValueError('Unsupported accepted input digest')
    detail=str(supplement.resolve()).replace("'","''")
    setup=f"ATTACH '{detail}' AS detail (READ_ONLY); SET enable_external_access=false;"
    with raw.open('x') as out,errors.open('x') as err:
        result=subprocess.run(['duckdb','-init','/dev/null','-readonly','-bail','-batch','-csv','-cmd',setup,str(snapshot.resolve()),'-f',str(query.resolve())],stdout=out,stderr=err,timeout=300,check=False)
    if result.returncode:raise RuntimeError(f'Native bullpen query failed ({result.returncode}); see {errors}')
    exported=lineup.read_export(raw.read_text());manifests=exported['manifest']
    if (len(manifests)!=1 or manifests[0].get('catalog_sha256')!=CATALOG
            or manifests[0].get('schema_version')!='f03-spike-1' or manifests[0].get('query_version')!='2019-filtered-extract-v1'
            or not all(lineup.nonempty(manifests[0].get(k)) for k in ['source_url','extracted_at','attribution'])):
        raise ValueError('Unsupported snapshot provenance')
    counts=json.loads(opportunities.read_text(),object_pairs_hook=unique_object)
    if (counts['provenance']['snapshot_sha256']!=pins[0] or counts['provenance']['opportunity_basis']!=BASIS
            or counts['provenance']['attribution']!=manifests[0]['attribution']):raise ValueError('Unsupported opportunity provenance')
    report=inventory(exported['game'],exported['person'],exported['appearance'],counts)
    report['provenance']=dict(snapshot=manifests[0],source_commit=SOURCE_COMMIT,snapshot_sha256=pins[0],supplement_sha256=pins[1],opportunity_report_sha256=pins[2],query_sha256=pins[3],recipe_sha256=pins[4],native_export_sha256=digest(raw),attribution=manifests[0]['attribution'])
    write_report(report,inputs,pins,output)
    return report


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    for name in ['snapshot','supplement','opportunities','output']:parser.add_argument(name,type=Path)
    args=parser.parse_args();r=publish(args.snapshot,args.supplement,args.opportunities,args.output)
    print(json.dumps(dict(game_id=r['game_id'],pregame_bullpen_status=r['pregame_bullpen_status'],teams=[dict(side=t['side'],observed_nonstarter_pitcher_ids=t['observed_nonstarter_pitcher_ids'],season_pitchers=len(t['season_pitchers'])) for t in r['teams']]),sort_keys=True))

if __name__=='__main__':main()
