"""Opt-in season-pool simulation with canonical usage feedback; not dated roster replay."""
import argparse
from collections import Counter
import hashlib
import json
from pathlib import Path
import subprocess
from preview import game_inputs as game, managed_inputs as managed, season_manager as manager
from preview import season_runner as season, usage_ledger, season_usage
from preview.season_usage import _starters, _json, _file, TARGET_SHA256

LIMITS=('Whole-season club pools/observed positions assumed usable, not daily injuries/minor-league rosters. '
    'Original date then game-ID ordering, including normalized suspensions and unresolved source schedule flags. '
    'Global simulated pitching rest; empirical terminal-PA budgets with marked role priors, not measured pitch counts/fatigue. '
    'Soft usage preferences, no forced outcomes, exact quotas or historical/statistical fidelity certification.')


def _dump(value):return json.dumps(value,sort_keys=True,indent=2,allow_nan=False)+'\n'


def _pins(inputs,pins):
    if len(inputs)!=len(pins) or tuple(game.real.digest(p) for p in inputs)!=tuple(pins):
        raise ValueError('Managed season inputs changed before acceptance')


def _entry(row,bundle):
    return dict({k:row[k] for k in (*usage_ledger.CORE,'seed')},
        accounting_format='controlled-accounting-managed-v1',role_actors=season.actor_bindings(bundle),
        **_starters(bundle['lineup'],row))


def run(source,pool,targets,inputs,pins,binary,baseline,contact,master_seed,output):
    """Caller authenticates full sources/targets. Failure stops; never continue on incomplete history."""
    output=game._destination(output);rows=season.manifest(source,master_seed);_pins(inputs,pins)
    ledger=usage_ledger.Ledger(rows);turns={};results=[];artifacts={}
    prepared=dict(source,_managed_budgets=managed._budgets(source))
    output.mkdir();(output/'games').mkdir()
    def publish(name,text):
        path=output/name;game.ratings.publish(text,(),(),path);artifacts[name]=game.real.digest(path)
    for row in rows:
        snapshot=ledger.snapshot()
        try:
            bundle=managed.build_game(prepared,pool,snapshot,row['game_id'],turns)
            entry=_entry(row,bundle)
            if entry['role_actors']!=bundle['role_actors']:raise ValueError('Managed actor declaration mismatch')
        except ValueError as error:
            row.update(status='input_refused',reason=str(error));ledger.record_refusal(row,str(error));break
        prefix='games/'+row['game_id']+'/'
        folder=output/prefix;folder.mkdir()
        for name in ['lineup','cast','bench']:publish(prefix+name+'.tsv',bundle[name])
        command=[str(Path(binary).resolve()),'--baseline',str(Path(baseline).resolve()),
            '--contact-counts',str(Path(contact).resolve()),'--lineup','lineup.tsv','--cast','cast.tsv',
            '--bench','bench.tsv','--seed',str(row['seed']),'--format','accounting']
        row.update(entry,cold_profiles=bundle['cold_profiles'],budget_priors=bundle['budget_priors'],
            starter_decisions=bundle['starter_decisions'],cast_sizes=bundle['cast_sizes'])
        try:
            with (folder/'stdout.tsv').open('xb') as out,(folder/'stderr.txt').open('xb') as err:
                result=subprocess.run(command,cwd=folder,stdout=out,stderr=err,timeout=30,check=False)
            row['exit']=result.returncode
            if result.returncode:raise ValueError(f'Native game exited {result.returncode}; see raw stderr')
            parsed=season.parse_accounting((folder/'stdout.tsv').read_text(),entry)
            game_receipt=dict(format='managed-game-receipt-v1',entry=entry,command=command,
                pregame_sha256=hashlib.sha256(usage_ledger._canonical(snapshot)).hexdigest(),
                rotation_before=turns,next_turns=bundle['next_turns'],
                starter_decisions=bundle['starter_decisions'],
                output_sha256={n:game.real.digest(folder/n) for n in ['lineup.tsv','cast.tsv','bench.tsv','stdout.tsv','stderr.txt']})
            receipt_text=_dump(game_receipt)
            ledger.accept(entry,parsed,hashlib.sha256(receipt_text.encode()).hexdigest())
        except (ValueError,subprocess.TimeoutExpired,OSError) as error:
            row.update(status='simulation_failed',reason=str(error));ledger.record_refusal(row,str(error))
        else:
            publish(prefix+'receipt.json',receipt_text)
            turns=bundle['next_turns'];results.append(parsed);row.update(status='complete',teams=parsed['teams'])
        for name in ['stdout.tsv','stderr.txt']:artifacts[prefix+name]=game.real.digest(folder/name)
        if row['status']!='complete':break
    report=dict(format='managed-season-v1',season_policy=manager.POLICY,availability_basis=manager.AVAILABILITY,
        master_seed=master_seed,seed_algorithm='sha256:bbsim-season-v1:first8-big-endian',games=rows,
        counts=dict(sorted(Counter(r['status'] for r in rows).items())),rotation_turns=turns,
        complete_season=bool(rows) and all(r['status']=='complete' for r in rows),
        versions=results[0]['versions'] if results else None,limits=LIMITS)
    for name,value in [('manifest.json',report),('schedule.json',source['readiness']['rows']),
            ('summary.json',season.accumulate(results)),('ledger.json',ledger.snapshot()),('comparison.json',ledger.compare(targets))]:
        publish(name,_dump(value))
    _pins(inputs,pins)
    if any(game.real.digest(output/name)!=sha for name,sha in artifacts.items()):raise ValueError('Managed output changed before acceptance')
    receipt=dict(format='managed-season-receipt-v1',input_sha256={str(p):h for p,h in zip(inputs,pins)},
        output_sha256=artifacts,counts=report['counts'],complete_season=report['complete_season'],attribution=game.real.ATTRIBUTION)
    rebuild(output,source,pool,targets,receipt)
    game.ratings.publish(_dump(receipt),inputs,tuple(pins),output/'receipt.json')
    return report


def rebuild(directory,source,pool,targets,receipt=None):
    """Authenticate retained bytes and reconstruct every pregame choice/ledger from supplied authenticated sources."""
    root=Path(directory).resolve();receipt=_json(root/'receipt.json') if receipt is None else receipt
    report=_json(root/'manifest.json')
    if (receipt['format']!='managed-season-receipt-v1' or report['format']!='managed-season-v1'
            or report['season_policy']!=manager.POLICY or report['availability_basis']!=manager.AVAILABILITY
            or report['seed_algorithm']!='sha256:bbsim-season-v1:first8-big-endian' or report['limits']!=LIMITS):
        raise ValueError('Unsupported managed receipt/policy')
    required={'manifest.json','schedule.json','summary.json','ledger.json','comparison.json'}
    for row in report['games']:
        if row['status'] in {'complete','simulation_failed'}:
            required.update('games/'+row['game_id']+'/'+name for name in ['lineup.tsv','cast.tsv','bench.tsv','stdout.tsv','stderr.txt'])
        if row['status']=='complete':required.add('games/'+row['game_id']+'/receipt.json')
    if set(receipt['output_sha256'])!=required:raise ValueError('Missing/unknown managed artifact pins')
    inputs=[Path(p) for p in receipt['input_sha256']];pins=list(receipt['input_sha256'].values());_pins(inputs,pins)
    for name,sha in receipt['output_sha256'].items():
        if game.real.digest(_file(root,name))!=sha:raise ValueError('Managed retained artifact changed')
    expected=season.manifest(source,report['master_seed']);rows=report['games']
    if len(rows)!=len(expected) or any(any(a[k]!=b[k] for k in (*usage_ledger.CORE,'seed')) for a,b in zip(rows,expected)):
        raise ValueError('Managed schedule/seed mismatch')
    ledger=usage_ledger.Ledger(expected);turns={};results=[];stopped=False
    prepared=dict(source,_managed_budgets=managed._budgets(source))
    for row in rows:
        if stopped:
            if row['status']!='pending':raise ValueError('Managed batch continued after failure')
            continue
        if row['status'] in {'input_refused','simulation_failed'}:
            ledger.record_refusal(row,row['reason']);stopped=True;continue
        if row['status']!='complete':raise ValueError('Unexplained pending managed game')
        if type(row.get('exit')) is not int or row['exit']!=0:raise ValueError('Invalid completed native exit')
        b=managed.build_game(prepared,pool,ledger.snapshot(),row['game_id'],turns);entry=_entry(row,b)
        prefix='games/'+row['game_id']+'/';folder=root/prefix;gr=_json(folder/'receipt.json')
        if (gr['format']!='managed-game-receipt-v1' or gr['entry']!=entry or gr['rotation_before']!=turns
                or gr['next_turns']!=b['next_turns'] or gr['starter_decisions']!=b['starter_decisions']
                or gr['pregame_sha256']!=hashlib.sha256(usage_ledger._canonical(ledger.snapshot())).hexdigest()):
            raise ValueError('Managed pregame recipe/ledger mismatch')
        for name in ['lineup','cast','bench']:
            if (folder/(name+'.tsv')).read_text()!=b[name]:raise ValueError('Managed personnel recipe mismatch')
        if set(gr['output_sha256'])!={'lineup.tsv','cast.tsv','bench.tsv','stdout.tsv','stderr.txt'}:
            raise ValueError('Missing/unknown managed game pins')
        for name,sha in gr['output_sha256'].items():
            if game.real.digest(_file(folder,name))!=sha:raise ValueError('Managed game artifact changed')
        parsed=season.parse_accounting((folder/'stdout.tsv').read_text(),entry)
        ledger.accept(entry,parsed,game.real.digest(folder/'receipt.json'));results.append(parsed);turns=b['next_turns']
        if any(row[k]!=entry[k] for k in entry) or row['teams']!=parsed['teams']:raise ValueError('Managed manifest entry mismatch')
    rebuilt=ledger.snapshot();comparison=ledger.compare(targets)
    counts=dict(sorted(Counter(r['status'] for r in rows).items()));complete=bool(rows) and all(r['status']=='complete' for r in rows)
    if (report['versions']!=(results[0]['versions'] if results else None)
            or any(r['versions']!=report['versions'] for r in results)
            or report['counts']!=counts or receipt['counts']!=counts or report['complete_season']!=complete
            or receipt['complete_season']!=complete or report['rotation_turns']!=turns
            or _json(root/'ledger.json')!=rebuilt or _json(root/'comparison.json')!=comparison
            or _json(root/'summary.json')!=season.accumulate(results) or _json(root/'schedule.json')!=source['readiness']['rows']):
        raise ValueError('Managed reconstructed summary mismatch')
    _pins(inputs,pins)
    return rebuilt,comparison


def export(lineups,native,opportunities,events,readiness,target_directory,baseline,contact,binary,master_seed,output):
    paths=[Path(p).resolve() for p in [lineups,native,opportunities,events,readiness]]
    target=Path(target_directory).resolve()/'targets.json'
    if game.real.digest(target)!=TARGET_SHA256:raise ValueError('Unsupported approved usage targets')
    targets=_json(target)
    recipes=[Path(m.__file__) for m in [game.lineup,game.player,game.workload,game.bullpen,game.real,game.ratings,
        game.cast,game,managed,manager,season,usage_ledger,season_usage,managed.bench]]+[Path(__file__)]
    inputs=[*paths,target,*[Path(p).resolve() for p in [baseline,contact,binary]],*recipes]
    pins=tuple(game.real.digest(p) for p in inputs)
    if pins[:5]!=game.INPUT_SHA or game.real.digest(Path(game.workload.__file__))!=game.cast.WORKLOAD_RECIPE_SHA:
        raise ValueError('Unsupported managed source/recipe digest')
    source=game.prepare_season(game.read_sources(paths));pool=manager.build_pool(source,targets)
    report=run(source,pool,targets,inputs,pins,binary,baseline,contact,master_seed,output)
    return report


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    for name in ['lineups','native','opportunities','events','readiness','targets','baseline','contact','binary','output']:
        parser.add_argument(name,type=Path)
    parser.add_argument('--seed',type=int,default=1)
    args=parser.parse_args()
    result=export(args.lineups,args.native,args.opportunities,args.events,args.readiness,args.targets,
        args.baseline,args.contact,args.binary,args.seed,args.output)
    print(json.dumps(dict(counts=result['counts'],complete_season=result['complete_season']),sort_keys=True))
    return 0 if result['complete_season'] else 2


if __name__=='__main__':raise SystemExit(main())
