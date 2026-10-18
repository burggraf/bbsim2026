"""Exclusive game inputs with an explicitly assumed participant-proxy bench, not eligible rosters."""
import argparse
import hashlib
import json
from pathlib import Path
from audits import pinch_hitting_readiness as audit
from preview import game_inputs as game

POLICY='late-close-expected-bases-ph-v1'
AVAILABILITY='same-game participant proxies assumed available at start; not verified eligibility'
DEFENSE_BASIS='supplied observed positions assumed usable; not verified defensive eligibility'


def build_bench(source,game_id):
    """Caller authenticates the complete cohort; reuse the existing conservative evidence audit."""
    inventory=audit.inventory(source)
    target=next((g for g in inventory['games'] if g['game_id']==game_id),None)
    if target is None:raise ValueError('Unknown bench source game ID')
    if not target['starting_maps_supported']:raise ValueError('Unsupported bench starting maps')
    people={p['player_id']:p for p in source['data']['person']}
    starting={pid for side in target['sides'] for pid in side['starting_ids']}
    fields=dict(format='assumed-game-bench-v1',game_id=game_id,policy=POLICY,
        availability_basis=AVAILABILITY,defense_basis=DEFENSE_BASIS,
        snapshot_sha256=game.real.SNAPSHOT_SHA,supplement_sha256=game.real.SUPPLEMENT_SHA,
        opportunity_report_sha256=game.ratings.REPORT_SHA,attribution=game.real.ATTRIBUTION)
    sizes={};excluded={}
    for side in target['sides']:
        label=side['side'];sizes[label]=0;excluded[label]=[]
        for actor in side['actors']:
            pid=actor['player_id'];reasons=list(actor['reasons'])
            if pid in starting and 'starting_player' not in reasons:reasons.append('starting_player')
            if reasons:
                excluded[label].append(dict(player_id=pid,reasons=sorted(reasons)))
                continue
            if not pid or len(pid)>64 or any(c not in 'abcdefghijklmnopqrstuvwxyz0123456789-' for c in pid):
                raise ValueError('Invalid bench actor identity')
            bio=people[pid];prefix=f'{label}:{pid}'
            fields[prefix+':name']=bio['first_name']+' '+bio['last_name']
            positions=[p for p in actor['positions'] if 2<=p<=9]
            fields[prefix+':positions']=','.join(map(str,positions)) or '?'
            fields[prefix+':counts']=','.join(map(str,actor['batting']['counts']))
            sizes[label]+=1
        if sizes[label]>50:raise ValueError('Oversized bench pool')
    for key,value in fields.items():
        if not value or value.strip()!=value or any(ord(c)<32 or ord(c)==127 for c in value):
            raise ValueError('Invalid bench field/name')
    text='key\tvalue\n'+''.join(f'{key}\t{value}\n' for key,value in sorted(fields.items()))
    if len(text.encode())>65536:raise ValueError('Oversized bench document')
    return dict(bench=text,pool_sizes=sizes,bench_exclusions=excluded)


def export(paths,game_id,output):
    output=game._destination(output);paths=list(map(Path,paths))
    recipes=[Path(m.__file__) for m in [game.lineup,game.player,game.workload,game.bullpen,
        game.real,game.ratings,game.cast,game,audit]]+[Path(__file__)]
    inputs=paths+recipes;pins=tuple(game.real.digest(p) for p in inputs)
    if pins[:5]!=game.INPUT_SHA or game.real.digest(Path(game.workload.__file__))!=game.cast.WORKLOAD_RECIPE_SHA:
        raise ValueError('Unsupported bench input source/recipe digest')
    source=game.read_sources(paths)
    bundle=game.build_game(source,game_id)
    bundle.update(build_bench(source,game_id))
    output.mkdir()  # A partial failed destination remains unaccepted; receipt is published last.
    names=['lineup','cast','bench']
    receipt={k:v for k,v in bundle.items() if k not in names}
    receipt.update(format='assumed-ph-game-inputs-v1',policy=POLICY,
        availability_basis=AVAILABILITY,defense_basis=DEFENSE_BASIS,
        input_sha256={str(p):h for p,h in zip(inputs,pins)},
        output_sha256={name+'.tsv':hashlib.sha256(bundle[name].encode()).hexdigest() for name in names},
        attribution=game.real.ATTRIBUTION,
        limits='observed same-game proxies, not complete bench availability, daily eligibility, defensive certification, fatigue, official statistics or fidelity; no event timing/outcomes supplied to PH policy')
    for name in names:game.ratings.publish(bundle[name],inputs,pins,output/(name+'.tsv'))
    game.ratings.publish(json.dumps(receipt,sort_keys=True,indent=2,allow_nan=False)+'\n',inputs,pins,output/'receipt.json')
    return receipt


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    for name in ['lineups','native','opportunities','events','readiness']:parser.add_argument(name,type=Path)
    parser.add_argument('game_id');parser.add_argument('output',type=Path)
    args=parser.parse_args()
    receipt=export([args.lineups,args.native,args.opportunities,args.events,args.readiness],args.game_id,args.output)
    print(json.dumps({k:receipt[k] for k in ['game_id','pool_sizes','policy']},sort_keys=True))


if __name__=='__main__':main()
