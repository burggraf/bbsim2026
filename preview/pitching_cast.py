"""Source-bound closed pitching cast, workload proxies and participant ability counts."""
import argparse
from fractions import Fraction
import json
from pathlib import Path
import re
from audits import pitcher_workload as workload
from audits.raw_mechanisms import unique_object
from preview import player_ratings as ratings
from preview import real_lineup as real

BULLPEN_SHA='758186cceb67815573e7e109763f7be7d5e57b9ec9583bc8a3cca299d39edc88'
WORKLOAD_SHA='125768bd82ce54ead277d92301fbb7434e823a8fc4432777087c15b8e46ad2f9'
POLICY='pa-mean-closed-cast-v1'
AVAILABILITY='recorded nonstarters assumed available at start; not verified eligibility'
publish=ratings.publish  # Existing fsync/exclusive-link/final-reread implementation.


def encode(report,witness,bullpen,workloads):
    fields=dict(line.split('\t') for line in ratings.encode(report,witness).splitlines()[1:])
    if (bullpen['format']!='2019-bullpen-readiness-v1' or bullpen['game_id']!=witness['game_id']
            or bullpen['pregame_bullpen_status']!='unsupported' or workloads['format']!='2019-pitcher-workload-v1'
            or workloads['game_id']!=witness['game_id'] or workloads['basis']!=workload.BASIS):
        raise ValueError('Unsupported pitching cast/workload basis')
    for k,v in dict(snapshot_sha256=real.SNAPSHOT_SHA,supplement_sha256=real.SUPPLEMENT_SHA,
            opportunity_report_sha256=ratings.REPORT_SHA,lineup_report_sha256=real.REPORT_SHA,
            bullpen_report_sha256=BULLPEN_SHA,attribution=real.ATTRIBUTION).items():
        if workloads['provenance'].get(k)!=v:raise ValueError('Unsupported pitching workload provenance')
    profiles={}
    for p in report['roles']['pitcher']['profiles']:
        pid=real.literal(p['player_id'])
        if pid in profiles:raise ValueError('Duplicate pitcher ability profile')
        profiles[pid]=p
    observed={}
    for p in workloads['players']:
        pid=real.literal(p['player_id'])
        if pid in observed:raise ValueError('Duplicate pitcher workload profile')
        observed[pid]=p
    if len(bullpen['teams'])!=2 or {t['side'] for t in bullpen['teams']}!={'Away','Home'}:
        raise ValueError('Invalid pitching cast sides')
    fields.update(format='closed-pitching-cast-v1',policy=POLICY,workload_report_sha256=WORKLOAD_SHA,
        bullpen_report_sha256=BULLPEN_SHA,workload_basis=workload.BASIS,availability_basis=AVAILABILITY,
        exhaustion='last_pitcher_stays')
    selected=set();starting={p['player_id']:p['name'] for t in witness['teams'] for p in t['fielding']}
    for side,team_id,n in [('Away','SEA',5),('Home','OAK',7)]:
        team=next(t for t in bullpen['teams'] if t['side']==side)
        initial=next(t for t in witness['teams'] if t['side']==side)
        facts=team['observed_pitchers'];ids=[p['player_id'] for p in facts];starter=initial['pitcher_id']
        if (team['team_id']!=team_id or team['starter_id']!=starter or len(ids)!=n or len(set(ids))!=n
                or starter not in ids or set(team['observed_nonstarter_pitcher_ids'])!=set(ids)-{starter}
                or len(team['observed_nonstarter_pitcher_ids'])!=n-1):raise ValueError('Invalid closed pitching cast')
        means={};names={}
        for fact in facts:
            pid=real.literal(fact['player_id']);name=real.literal(fact['name'])
            if (not re.fullmatch('[a-z0-9]+',pid) or pid in selected or fact['eligible_at_start'] is not None
                    or (pid in starting and (pid!=starter or starting[pid]!=name))):raise ValueError('Invalid pitching identity/name/eligibility')
            selected.add(pid);names[pid]=name
            if pid not in profiles or pid not in observed:raise ValueError('Missing pitcher ability/workload profile')
            ability=profiles[pid];counts=ability['counts'];p=observed[pid];copied=p['ability_profile']
            if (not isinstance(copied,dict) or type(copied.get('included_pa')) is not int
                    or set(copied.get('counts',{}))!=set(ratings.KEYS)
                    or any(type(copied['counts'][k]) is not int for k in ratings.KEYS)
                    or p['name']!=name or copied!=ability
                    or set(counts)!=set(ratings.KEYS) or type(ability['included_pa']) is not int
                    or any(type(counts[k]) is not int or not 0<=counts[k]<=cap for k,cap in zip(ratings.KEYS,ratings.COUNTS))
                    or sum(counts.values())!=ability['included_pa']):raise ValueError('Invalid pitcher count/name conservation')
            fields['pitcher:'+pid]=','.join(str(counts[k]) for k in ratings.KEYS)
            role='start' if pid==starter else 'relief'
            for kind in ['start','relief']:
                summary=p[kind];s,g=summary['pa_sum'],summary['outings']
                if (set(summary)!={'pa_sum','outings','mean_pa','limit'}
                        or (g!=0 and (type(summary['limit']) is not int or type(summary['mean_pa']) not in (int,float)))):
                    raise ValueError('Invalid workload summary types')
                if type(s) is not int or type(g) is not int or not 0<=s<=1<<52 or not 0<=g<=2429:
                    raise ValueError('Invalid workload sum/outing denominator')
                rows=p['outing_rows']
                if any(type(o['training']) is not bool or type(o['terminal_pa']) is not int or o['terminal_pa']<0 for o in rows):
                    raise ValueError('Invalid workload outing facts')
                keys=[o['game_id'] for o in rows]
                if len(keys)!=len(set(keys)):raise ValueError('Duplicate workload outing')
                training=[o for o in rows if o['training'] and o['role']==kind]
                if (len(training)!=g or sum(o['terminal_pa'] for o in training)!=s
                        or summary['mean_pa']!=(s/g if g else None)
                        or summary['limit']!=(max(1,(s+g-1)//g) if g else None)):
                    raise ValueError('Invalid workload role reconciliation')
                if kind==role and g==0:raise ValueError('Missing selected workload role')
                fields[f'workload:{kind}:{pid}']=f'{s},{g}'
                if kind==role:means[pid]=Fraction(s,g)
        ordered=[starter]+sorted(set(ids)-{starter},key=lambda pid:(-means[pid],pid))
        for rank,pid in enumerate(ordered):fields[f'cast:{side}:{rank}:{pid}']=names[pid]
    if set(observed)!=selected or len(selected)!=12:raise ValueError('Missing/extra closed-cast workload identities')
    return 'key\tvalue\n'+''.join(real.literal(k)+'\t'+real.literal(v)+'\n' for k,v in sorted(fields.items()))


def export(report,lineup,bullpen,workloads,output):
    if output.exists():raise ValueError('Refusing to overwrite pitching cast output')
    if output.resolve().is_relative_to(Path(__file__).resolve().parents[1]):raise ValueError('Cast output must be outside repository')
    inputs=(report,lineup,bullpen,workloads);pins=tuple(real.digest(p) for p in inputs)
    if pins!=(ratings.REPORT_SHA,real.REPORT_SHA,BULLPEN_SHA,WORKLOAD_SHA):raise ValueError('Unsupported pitching cast input digest')
    r,l,b,w=(json.loads(p.read_text(),object_pairs_hook=unique_object) for p in inputs)
    if l['format']!='2019-lineup-readiness-v1':raise ValueError('Unsupported pitching lineup report')
    text=encode(r,l['witnesses']['primary'],b,w)
    publish(text,inputs,pins,output)
    return text


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    for name in ['report','lineup','bullpen','workloads','output']:parser.add_argument(name,type=Path)
    args=parser.parse_args();text=export(args.report,args.lineup,args.bullpen,args.workloads,args.output)
    print(f'Exported30 ability vectors and12 pitcher workloads: {len(text.encode())} bytes; SHA256 {real.digest(args.output)}')


if __name__=='__main__':main()
