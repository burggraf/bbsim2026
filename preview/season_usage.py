"""Rebuild observational usage from a verified controlled batch; never select personnel."""
import argparse
from collections import Counter
import hashlib
import json
from pathlib import Path
from audits import season_usage_targets as target
from preview import game_inputs as game
from preview import season_runner as season
from preview import usage_ledger
from preview.usage_ledger import Ledger, CORE, _canonical

TARGET_SHA256='f45cd37997e1027ac17e4e4d752c4aa44e2be96f871f510408f72e432108972a'


def _json(path):
    def unique(pairs):
        result={}
        for key,value in pairs:
            if key in result:raise ValueError('Duplicate JSON key')
            result[key]=value
        return result
    return json.loads(path.read_text(),object_pairs_hook=unique)


def _file(root,name):
    path=Path(name)
    if path.is_absolute() or '..' in path.parts or not path.parts or not (root/path).resolve().is_relative_to(root.resolve()):
        raise ValueError('Unsafe receipt artifact path')
    return root/path


def _starters(text,entry):
    batters={entry[k]:[] for k in CORE[2:]};pitchers={};slots=set()
    for line in text.splitlines():
        key=line.split('\t')[0].split(':')
        if key[0] not in {'Away','Home'}:continue
        if len(key)!=4:raise ValueError('Invalid selected lineup identity')
        side,slot,position,pid=key;tid=entry['away_team_id' if side=='Away' else 'home_team_id']
        if slot not in {str(n) for n in range(10)} or (side,slot) in slots:
            raise ValueError('Invalid/duplicate selected lineup slot')
        slots.add((side,slot))
        if slot!='0':batters[tid].append(pid)
        if position=='1':
            if tid in pitchers:raise ValueError('Duplicate starting pitcher')
            pitchers[tid]=pid
    if any(len(ids)!=9 or len(set(ids))!=9 for ids in batters.values()) or set(pitchers)!=set(batters):
        raise ValueError('Incomplete selected starting personnel')
    return dict(starting_batter_ids=batters,starting_pitcher_ids=pitchers)


def collect(batch_directory,target_directory):
    """Authenticate retained bytes and rebuild from scratch; no authoritative cache."""
    batch=Path(batch_directory).resolve();targets=Path(target_directory).resolve()
    watched={}
    def watch(path,expected=None):
        value=game.real.digest(path)
        if expected is not None and value!=expected:raise ValueError('Receipt/input digest mismatch: '+str(path))
        watched[path]=value
        return value
    batch_sha=watch(batch/'receipt.json');receipt=_json(batch/'receipt.json')
    target_sha=watch(targets/'receipt.json');target_receipt=_json(targets/'receipt.json')
    if receipt['format']!='controlled-season-receipt-v1' or target_receipt['format']!=target.FORMAT+'-receipt':
        raise ValueError('Unsupported source receipt')
    for name,pin in receipt['output_sha256'].items():watch(_file(batch,name),pin)
    for name,pin in receipt['input_sha256'].items():watch(Path(name),pin)
    if target_receipt['payload_sha256']!=TARGET_SHA256:raise ValueError('Unrecognized approved target digest')
    watch(targets/'targets.json',TARGET_SHA256)
    watch(targets/'native.json',target_receipt['native_export_sha256'])
    expected={ '2019-spike.duckdb':target.SNAPSHOT_SHA256,
        '2019-season-usage-targets.sql':target.SQL_SHA256,
        'season_usage_targets.py':watch(Path(target.__file__)) }
    if target_receipt['inputs_sha256']!=expected:raise ValueError('Unrecognized target provenance')
    historical=_json(targets/'targets.json')
    provenance=historical['provenance']
    if (provenance['snapshot_sha256']!=expected['2019-spike.duckdb'] or provenance['query_sha256']!=target.SQL_SHA256
            or provenance['recipe_sha256']!=expected['season_usage_targets.py']
            or provenance['native_export_sha256']!=target_receipt['native_export_sha256']):
        raise ValueError('Target provenance mismatch')
    manifest=_json(batch/'manifest.json')
    if (manifest['format']!='controlled-season-v1' or manifest['simulation_policy']!='regular-full-game-v1'
            or manifest['seed_algorithm']!='sha256:bbsim-season-v1:first8-big-endian'):
        raise ValueError('Unsupported controlled batch policy')
    rows=manifest['games'];schedule=[{k:g[k] for k in CORE} for g in historical['games']]
    if [{k:g[k] for k in CORE} for g in rows]!=schedule:raise ValueError('Batch/target schedule mismatch')
    counts=dict(sorted(Counter(r['status'] for r in rows).items()))
    complete=bool(rows) and all(r['status']=='complete' for r in rows)
    if (manifest['counts']!=counts or receipt['counts']!=counts
            or manifest['complete_season']!=complete or receipt['complete_season']!=complete):
        raise ValueError('Batch completion/count mismatch')
    expected_outputs={'manifest.json','schedule.json','summary.json'}
    ledger=Ledger(schedule);witnesses=[];results=[]
    for row in rows:
        if row['seed']!=season.game_seed(manifest['master_seed'],row['game_id']):raise ValueError('Batch seed mismatch')
        entry={k:row[k] for k in (*CORE,'seed')}
        status=row['status'];witness=dict(entry=entry,status=status,batch_receipt_sha256=batch_sha)
        if status=='complete':
            prefix='games/'+row['game_id']+'/'
            names=[prefix+n for n in ['lineup.tsv','cast.tsv','stdout.tsv','stderr.txt']]
            expected_outputs.update(names)
            bundle={k:(batch/(prefix+k+'.tsv')).read_text() for k in ['lineup','cast']}
            entry.update(role_actors=season.actor_bindings(bundle),**_starters(bundle['lineup'],entry))
            if row['role_actors']!=entry['role_actors'] or row.get('exit')!=0:raise ValueError('Batch actor/exit mismatch')
            parsed=season.parse_accounting((batch/(prefix+'stdout.tsv')).read_text(),entry)
            if parsed['teams']!=row['teams'] or parsed['versions']!=manifest['versions']:raise ValueError('Batch accounting mismatch')
            witness['artifacts_sha256']={name:receipt['output_sha256'][name] for name in names}
            witness['receipt_sha256']=hashlib.sha256(_canonical(witness)).hexdigest()
            ledger.accept(entry,parsed,witness['receipt_sha256']);results.append(parsed)
        elif status in {'input_refused','simulation_failed'}:
            witness['reason']=row['reason'];ledger.record_refusal(entry,status+': '+row['reason'])
            if status=='simulation_failed':
                prefix='games/'+row['game_id']+'/'
                expected_outputs.update(prefix+n for n in ['lineup.tsv','cast.tsv','stdout.tsv','stderr.txt'])
        else:raise ValueError('Unfinished/unknown controlled batch status')
        witnesses.append(witness)
    if set(receipt['output_sha256'])!=expected_outputs:raise ValueError('Incomplete/extra batch receipt artifacts')
    if season.accumulate(results)!=_json(batch/'summary.json'):raise ValueError('Batch summary mismatch')
    comparison=ledger.compare(historical)
    def unchanged():
        if any(game.real.digest(path)!=pin for path,pin in watched.items()):raise ValueError('Usage replay inputs changed')
    unchanged()
    return dict(ledger=ledger.snapshot(),comparison=comparison,game_receipts=witnesses,
        source_receipts=dict(batch=batch_sha,targets=target_sha)),watched


def publish(batch_directory,target_directory,output):
    output=game._destination(output)
    recipes=[Path(m.__file__).resolve() for m in [season,game,usage_ledger,target]]+[Path(__file__).resolve()]
    pins={p:game.real.digest(p) for p in recipes}
    report,inputs=collect(batch_directory,target_directory)
    inputs.update(pins)
    output.mkdir();outputs={}
    for key in ['ledger','comparison','game_receipts','source_receipts']:
        path=output/(key+'.json')
        game.ratings.publish(json.dumps(report[key],sort_keys=True,indent=2,allow_nan=False)+'\n',(),(),path)
        outputs[path.name]=game.real.digest(path)
    if any(game.real.digest(path)!=pin for path,pin in inputs.items()):raise ValueError('Usage inputs changed before publication')
    if any(game.real.digest(output/name)!=pin for name,pin in outputs.items()):raise ValueError('Usage output changed before receipt')
    receipt=dict(format='observational-season-usage-receipt-v1',input_sha256={str(p):h for p,h in inputs.items()},
        output_sha256=outputs,attribution=game.real.ATTRIBUTION)
    game.ratings.publish(json.dumps(receipt,sort_keys=True,indent=2,allow_nan=False)+'\n',tuple(inputs),tuple(inputs.values()),output/'receipt.json')
    return report


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    for name in ['batch','targets','output']:parser.add_argument(name,type=Path)
    args=parser.parse_args();report=publish(args.batch,args.targets,args.output)
    print(json.dumps(dict(coverage=report['comparison']['coverage'],headline=report['comparison']['headline'],
        aggregate=report['comparison']['aggregate']),sort_keys=True))


if __name__=='__main__':main()
