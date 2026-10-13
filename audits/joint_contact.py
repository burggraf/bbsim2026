"""Count conservative 2019 source-shape proxies; not legal event certificates."""
import argparse
from collections import Counter
import csv
import json
from pathlib import Path
import re
from audits import outcome_inventory as outcome
from audits import raw_mechanisms as raw
from audits.raw_mechanisms import read_export

BINS = ('ground_candidate', 'fly_candidate', 'line_candidate', 'pop_candidate', 'residual')
GROUPS = {
    'StrikeOut':'strikeout', 'Walk':'walk', 'HitByPitch':'hbp', 'Single':'single',
    'Double':'double', 'GroundRuleDouble':'double', 'Triple':'triple',
    'HomeRun':'home_run', 'InsideTheParkHomeRun':'home_run', 'ReachedOnError':'roe',
    'InPlayOut':'contact_out', 'FieldersChoice':'contact_out', 'SacrificeFly':'contact_out',
    'SacrificeHit':'contact_out', 'IntentionalWalk':'excluded_ibb', 'Interference':'excluded_interference',
}
CATEGORIES = ('strikeout','walk','hbp','single','double','triple','home_run','roe','contact_out')


def classify(row, context):
    """First-reason partition only; never reject an otherwise valid history."""
    if row['result']!='InPlayOut':
        return 'residual', 'non_inplay_result'
    if row['outs']!='1' or row['runs']!='0':
        return 'residual', 'non_single_scoreless_out'
    tags={'GroundBall':('G','ground_candidate'), 'Fly':('F','fly_candidate'),
          'LineDrive':('L','line_candidate'), 'PopUp':('P','pop_candidate')}
    if row['trajectory'] not in tags:
        return 'residual', 'unknown_trajectory'
    prefix, bin_name=tags[row['trajectory']]
    if raw.child_rows(row['flags'],raw.FLAG_FIELDS):
        return 'residual', 'flags_present'
    credits=raw.child_rows(row['credits'],raw.CREDIT_FIELDS)
    putouts=[c for c in credits if c['fielding_play']=='Putout']
    assists=[c for c in credits if c['fielding_play']=='Assist']
    ground=prefix=='G'
    if (len(putouts)!=1 or len(assists)!=int(ground) or len(credits)!=1+int(ground)
            or any(not 1<=c['fielding_position']<=9 for c in credits)
            or ground and assists[0]['fielding_position']==putouts[0]['fielding_position']):
        return 'residual', 'credit_shape'
    for runner in raw.child_rows(row['runners'],raw.RUNNER_FIELDS):
        role=runner['baserunner']
        if role=='Batter':
            held=(runner['runner_id']==context['batter_id'] and runner['is_out'] is True
                  and runner['base_end']=='Out' and runner['run_scored_flag'] is False)
        else:
            held=(bool(context[role.lower()]) and runner['runner_id']==context[role.lower()]
                  and runner['base_end']==role and runner['is_out'] is False
                  and runner['run_scored_flag'] is False and runner['advanced_on_error_flag'] is False
                  and runner['is_advance_attempt'] is False)
        if not held:
            return 'residual', 'runner_effect_or_unknown'
    value=raw.lex(row['raw_play'])
    if (value['status']!='split' or value['base_separators'] or value['advances']
            or len(value['base_atoms'])!=1 or len(value['modifiers'])!=1
            or not re.fullmatch(prefix+r'[1-9]?[LRSMDX]*[-+]?',value['modifiers'][0])):
        return 'residual', 'literal_not_supported'
    atom=value['base_atoms'][0]
    expected=(''.join(str(c['fielding_position']) for c in assists)+str(putouts[0]['fielding_position']))
    if atom!=expected:
        return 'residual', 'root_credit_mismatch'
    return bin_name, ''


def inventory(raw_rows, outcome_rows):
    """Reuse both complete validators while retaining only compact join contexts."""
    contexts = {}
    def validated_contexts():
        for row in outcome_rows:
            yield row
            contexts[int(row['event_key'])] = {name:row[name] for name in
                ('game_id','event_id','result','trajectory','angle','outs','runs',
                 'base_mask','outs_before','batter_id','first','second','third')}
    outcome_report = outcome.inventory(validated_contexts())
    counts = Counter(dict.fromkeys(CATEGORIES, 0))
    exclusions = Counter(dict(excluded_ibb=0, excluded_interference=0))
    bins = Counter(dict.fromkeys(BINS, 0))
    reasons, cells, examples = Counter(), Counter(), {}
    def validated_raw():
        for row in raw_rows:
            yield row
            if row['no_play_flag']=='true':
                continue
            key = int(row['event_key'])
            context = contexts.pop(key, None)
            if context is None or any(row[name]!=context[name] for name in
                ('game_id','event_id','result','trajectory','angle','outs','runs')):
                raise ValueError('Missing or disagreeing real-event join')
            if not row['result']:
                continue
            category = GROUPS.get(row['result'])
            if category is None:
                raise ValueError('Unknown PA classification')
            if category.startswith('excluded_'):
                exclusions[category] += 1
                continue
            counts[category] += 1
            if category!='contact_out':
                continue
            bin_name, reason = classify(row, context)
            bins[bin_name] += 1
            if reason:
                reasons[reason] += 1
            cells[(bin_name,int(context['base_mask']),int(context['outs_before']))] += 1
            example_key=(bin_name,reason)
            if example_key not in examples or key<int(examples[example_key]['event']['event_key']):
                examples[example_key]=dict(bin=bin_name,reason=reason,event=dict(row),context=dict(context))
    raw_report = raw.inventory(validated_raw())
    if contexts:
        raise ValueError('Unmatched outcome events')
    if sum(bins.values())!=counts['contact_out'] or sum(reasons.values())!=bins['residual']:
        raise ValueError('Contact partition conservation failed')
    return dict(format='2019-joint-contact-proxies-v1',counts=dict(counts),exclusions=dict(exclusions),
                contact_bins=dict(bins),residual_reasons=dict(sorted(reasons.items())),
                contexts=[dict(bin=b,base_mask=m,outs_before=o,events=n) for (b,m,o),n in sorted(cells.items())],
                examples=[examples[k] for k in sorted(examples)],raw_totals=raw_report['totals'],
                outcome_totals=outcome_report['totals'],games=raw_report['games'],
                supplied_credit_units=raw_report['supplied_credit_units'])


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('raw_events',type=Path)
    parser.add_argument('outcome_events',type=Path)
    parser.add_argument('raw_manifest',type=Path)
    parser.add_argument('outcome_manifest',type=Path)
    parser.add_argument('output',type=Path)
    args=parser.parse_args()
    if args.output.exists():
        raise ValueError('Refusing to overwrite an existing file')
    inputs=(args.raw_events,args.outcome_events,args.raw_manifest,args.outcome_manifest)
    before=tuple(outcome.digest(path) for path in inputs)
    manifest=json.loads(args.raw_manifest.read_text(),object_pairs_hook=raw.unique_object)
    snapshot=outcome.read_manifest(args.outcome_manifest)
    with args.raw_events.open(newline='') as raw_stream, args.outcome_events.open(newline='') as outcome_stream:
        reader=csv.DictReader(outcome_stream,strict=True)
        if (reader.fieldnames is None or len(reader.fieldnames)!=len(set(reader.fieldnames))
                or set(reader.fieldnames)!=outcome.FIELDS):
            raise ValueError('Invalid outcome export header')
        report=inventory(read_export(raw_stream),reader)
    raw.check_scope(dict(games=report['games'],totals=report['raw_totals'],
                         supplied_credit_units=report['supplied_credit_units']),manifest)
    expected=dict(zip(CATEGORIES,(42823,15142,1984,25947,8531,785,6776,1470,82245)))
    totals=dict(events=192025,pa=186517,non_pa=5508,outs=130270,runs=23467,putouts=130270,runner_run_units=23467)
    if (report['counts']!=expected or report['exclusions']!=dict(excluded_ibb=753,excluded_interference=61)
            or report['outcome_totals']!=totals):
        raise ValueError('Original 2019 preview cohort conservation failed')
    if before!=tuple(outcome.digest(path) for path in inputs):
        raise ValueError('Count inputs changed during read')
    report['provenance']=dict(raw_events_sha256=before[0],outcome_events_sha256=before[1],
        raw_manifest_sha256=before[2],outcome_manifest_sha256=before[3],snapshot=snapshot,
        supplement_sha256=manifest['database_sha256'],source_commit=raw.SOURCE_COMMIT,
        catalog_sha256=raw.CATALOG_SHA,snapshot_sha256=raw.SNAPSHOT_SHA,snapshot_id=857,
        raw_query_sha256=outcome.digest(Path(raw.__file__).with_name('2019-raw-mechanisms.sql')),
        outcome_query_sha256=outcome.digest(Path(outcome.__file__).with_name('2019-outcomes.sql')),
        recipe_sha256=outcome.digest(Path(__file__)),attribution=manifest['attribution'])
    with args.output.open('x') as output:
        output.write(json.dumps(report,sort_keys=True,indent=2)+'\n')
    print(json.dumps(dict(counts=report['counts'],contact_bins=report['contact_bins'],
                         residual_reasons=report['residual_reasons']),sort_keys=True))


if __name__=='__main__':main()
