"""Observed terminal-PA outing workloads, not pitch counts or measured fatigue."""
from collections import defaultdict
import argparse
import copy
import csv
import hashlib
import json
import subprocess
from pathlib import Path
from audits import lineup_readiness as lineup
from audits import player_opportunities as player

BASIS='literal terminal-PA participants per pitcher-game appearance; not pitch counts, official BF or fatigue'
TARGET='OAK201903200'
INPUT_SHA=('d036b4664c8ce4cab51e86aacc075727558f9461c813b51f53d5974fc39a4528',
    '554632467416eef202b7a55cadad2d09e5355ce98c83868b26b5267f98852b82',
    '382be5a84b3e95687e6356b00d3b1b68522f41a5c2a6a065daf876a5a4c6881e',
    '5385231905a1f0e57b57a736e5229def6e94653c2894a093b4542b131b8df63b',
    '758186cceb67815573e7e109763f7be7d5e57b9ec9583bc8a3cca299d39edc88')


def inventory(games,people,appearances,rows,selected):
    support=lineup.inventory(games,people,appearances)
    rows=list(rows)
    source=player.inventory(rows)  # Complete accepted outcome/participant validator.
    selected=list(selected)
    if len(selected)!=len(set(selected)) or any(not lineup.nonempty(p) for p in selected):
        raise ValueError('Duplicate/invalid selected pitcher')
    by_game={g['game_id']:g for g in games}
    training=set(support['candidate_game_ids'])
    profiles={p['player_id']:p for p in source['roles']['pitcher']['profiles']}
    bios=defaultdict(list);mounds=defaultdict(list);terminals=defaultdict(lambda:dict(terminal_pa=0,non_pa=0))
    for p in people:bios[p['player_id']].append(p)
    for a in appearances:
        if a['fielding_position']==1:mounds[a['game_id'],a['player_id']].append(a)
    for row in rows:
        if row['game_id'] not in by_game:raise ValueError('Orphan workload event')
        key=(row['game_id'],row['pitcher_id'])
        terminals[key]['terminal_pa' if row['result'] else 'non_pa']+=1
    result=[]
    for pid in sorted(selected):
        issues=[];outings=[]
        persons=bios[pid];bio=persons[0] if len(persons)==1 else None
        named=bio is not None and lineup.nonempty(bio['first_name']) and lineup.nonempty(bio['last_name'])
        if not named:issues.append('person_lookup')
        for (gid,pitcher),records in sorted(mounds.items()):
            if pitcher!=pid:continue
            g=by_game[gid];sides={a['side'] for a in records};side=next(iter(sides)) if len(sides)==1 else None
            role=None
            if side in lineup.SIDES:
                prefix=side.lower();starter=g.get(prefix+'_starting_pitcher_id');field=g.get('fielding_map_'+prefix)
                if lineup.nonempty(starter) and field is not None and field.get('1')==starter:
                    role='start' if starter==pid else 'relief'
            if role is None:issues.append('unknown_appearance_role')
            cell=terminals.get((gid,pid),dict(terminal_pa=0,non_pa=0))
            outings.append(dict(game_id=gid,side=side,role=role,position1_rows=len(records),
                training=gid in training and role is not None,
                appearances=copy.deepcopy(sorted(records,key=lambda a:json.dumps(a,sort_keys=True))),**cell))
        summaries={}
        for role in ['start','relief']:
            observed=[o for o in outings if o['training'] and o['role']==role]
            n=len(observed);total=sum(o['terminal_pa'] for o in observed)
            summaries[role]=dict(outings=n,pa_sum=total,mean_pa=total/n if n else None,
                limit=max(1,(total+n-1)//n) if n else None)
        missing=sum(v['terminal_pa'] for (gid,p),v in terminals.items() if p==pid and (gid,p) not in mounds)
        missing_np=sum(v['non_pa'] for (gid,p),v in terminals.items() if p==pid and (gid,p) not in mounds)
        if missing or missing_np:issues.append('missing_appearance')
        original=profiles.get(pid);source_pa=original['selected_pa'] if original else 0
        fit_pa=sum(s['pa_sum'] for s in summaries.values())
        if fit_pa>source_pa:raise ValueError('Workload PA conservation failed')
        result.append(dict(player_id=pid,name=bio['first_name']+' '+bio['last_name'] if named else None,
            start=summaries['start'],relief=summaries['relief'],outing_rows=outings,
            source_pa=source_pa,excluded_pa=source_pa-fit_pa,unmatched_terminal_pa=missing,unmatched_non_pa=missing_np,
            appearance_rows=copy.deepcopy(sorted([a for a in appearances if a['player_id']==pid],key=lambda a:json.dumps(a,sort_keys=True))),
            ability_profile=copy.deepcopy(original),issues=sorted(set(issues))))
    unmatched=sum(v['terminal_pa'] for k,v in terminals.items() if k not in mounds)
    return dict(format='2019-pitcher-workload-v1',basis=BASIS,training_game_ids=sorted(training),
        source_coverage=support['coverage'],source_outcome_totals=source['outcome_totals'],
        opportunity_roles_sha256=hashlib.sha256(json.dumps(source['roles'],sort_keys=True).encode()).hexdigest(),
        unmatched_terminal_pa=unmatched,unmatched_non_pa=sum(v['non_pa'] for k,v in terminals.items() if k not in mounds),players=result)


def write_report(report,inputs,pins,output):
    current=tuple(hashlib.sha256(Path(p).read_bytes()).hexdigest() for p in inputs)
    if current!=tuple(pins):raise ValueError('Workload audit inputs changed before publication')
    text=json.dumps(report,sort_keys=True,indent=2,allow_nan=False)+'\n'
    with Path(output).open('x',encoding='utf-8') as f:f.write(text)


def publish(snapshot,supplement,opportunities,lineups,bullpen,output):
    output=Path(output).resolve()
    if output.is_relative_to(Path(__file__).resolve().parents[1]):raise ValueError('Audit output must be outside repository')
    queries=[Path(__file__).with_name(name) for name in ['2019-lineup-readiness.sql','2019-outcomes.sql','2019-pitcher-workload.sql']]
    exports=[output.with_suffix('.'+label+'.csv') for label in ['lineup','events','workload']]
    errors=[p.with_suffix('.stderr') for p in exports]
    if any(p.exists() for p in [output,*exports,*errors]):raise ValueError('Refusing to overwrite workload report/native evidence')
    inputs=list(map(Path,[snapshot,supplement,opportunities,lineups,bullpen]))+queries+[Path(__file__)]
    digest=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
    pins=tuple(digest(p) for p in inputs)
    if pins[:5]!=INPUT_SHA:raise ValueError('Unsupported accepted workload input digest')
    detail=str(inputs[1].resolve()).replace("'","''")
    setup=f"ATTACH '{detail}' AS detail (READ_ONLY); SET enable_external_access=false;"
    for query,raw,err in zip(queries,exports,errors):
        with raw.open('x',encoding='utf-8') as out,err.open('x',encoding='utf-8') as stderr:
            result=subprocess.run(['duckdb','-init','/dev/null','-readonly','-bail','-batch','-csv','-nullvalue','','-cmd',setup,
                str(inputs[0].resolve()),'-f',str(query.resolve())],stdout=out,stderr=stderr,check=False,timeout=300)
        if result.returncode:raise RuntimeError(f'Native workload query failed ({result.returncode}); see {err}')
    from audits.raw_mechanisms import unique_object
    from audits.outcome_inventory import FIELDS
    from audits import bullpen_readiness as bullpen_audit
    counts,accepted_lineups,accepted_cast=[json.loads(p.read_text(),object_pairs_hook=unique_object) for p in inputs[2:5]]
    exported=lineup.read_export(exports[0].read_text());manifests=exported['manifest']
    if (len(manifests)!=1 or manifests[0].get('catalog_sha256')!=lineup.CATALOG
            or manifests[0].get('schema_version')!='f03-spike-1' or manifests[0].get('query_version')!='2019-filtered-extract-v1'
            or not all(lineup.nonempty(manifests[0].get(k)) for k in ['source_url','extracted_at','attribution'])):
        raise ValueError('Unsupported workload snapshot provenance')
    for report in [counts,accepted_lineups,accepted_cast]:
        provenance=report.get('provenance',{})
        if provenance.get('snapshot_sha256')!=pins[0] or provenance.get('attribution')!=manifests[0]['attribution']:
            raise ValueError('Unsupported workload report provenance')
    if counts['provenance'].get('opportunity_basis')!=bullpen_audit.BASIS:raise ValueError('Unsupported workload opportunity basis')
    games,people,appearances=(exported[k] for k in ['game','person','appearance'])
    fresh=lineup.inventory(games,people,appearances)
    if fresh!={k:v for k,v in accepted_lineups.items() if k!='provenance'}:raise ValueError('Workload lineup reconciliation failed')
    fresh_cast=bullpen_audit.inventory(games,people,appearances,counts,target=TARGET)
    if fresh_cast!={k:v for k,v in accepted_cast.items() if k!='provenance'}:raise ValueError('Workload cast reconciliation failed')
    selected=[p['player_id'] for t in fresh_cast['teams'] for p in t['observed_pitchers']]
    with exports[1].open(newline='',encoding='utf-8') as f:
        reader=csv.DictReader(f,strict=True)
        if reader.fieldnames is None or len(reader.fieldnames)!=len(FIELDS) or set(reader.fieldnames)!=FIELDS:
            raise ValueError('Invalid workload outcome export header')
        rows=list(reader)
    validated=player.inventory(rows)
    if validated['roles']!=counts['roles']:
        raise ValueError('Workload opportunity profile reconciliation failed')
    expected=defaultdict(lambda:dict(terminal_pa=0,non_pa=0))
    for row in rows:expected[row['game_id'],row['pitcher_id']]['terminal_pa' if row['result'] else 'non_pa']+=1
    native={}
    with exports[2].open(newline='',encoding='utf-8') as f:
        reader=csv.DictReader(f,strict=True)
        if reader.fieldnames!=['game_id','pitcher_id','terminal_pa','non_pa']:raise ValueError('Invalid native workload export header')
        for row in reader:
            if set(row)!=set(reader.fieldnames) or any(not isinstance(v,str) for v in row.values()):raise ValueError('Invalid native workload row')
            key=row['game_id'],row['pitcher_id']
            if key in native or any(not row[k].isascii() or not row[k].isdigit() for k in ['terminal_pa','non_pa']):
                raise ValueError('Duplicate/invalid native workload group')
            native[key]={k:int(row[k]) for k in ['terminal_pa','non_pa']}
    if native!=expected:raise ValueError('Native workload group reconciliation failed')
    # All source/participant/native totals agree before any outing mean is measured.
    report=inventory(games,people,appearances,rows,selected)
    report['game_id']=TARGET
    report['provenance']=dict(snapshot=manifests[0],source_commit=lineup.SOURCE_COMMIT,
        snapshot_sha256=pins[0],supplement_sha256=pins[1],opportunity_report_sha256=pins[2],
        lineup_report_sha256=pins[3],bullpen_report_sha256=pins[4],query_sha256=list(pins[5:8]),recipe_sha256=pins[8],
        native_export_sha256=[digest(p) for p in exports],attribution=manifests[0]['attribution'])
    write_report(report,inputs,pins,output)
    return report


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    for name in ['snapshot','supplement','opportunities','lineups','bullpen','output']:parser.add_argument(name,type=Path)
    args=parser.parse_args()
    r=publish(args.snapshot,args.supplement,args.opportunities,args.lineups,args.bullpen,args.output)
    print(json.dumps(dict(game_id=r['game_id'],players=len(r['players']),basis=r['basis']),sort_keys=True))


if __name__=='__main__':main()
