"""Retrospective fixed-opportunity joint endpoint diagnostics, never a generator."""
import argparse
from collections import Counter
import copy
from fractions import Fraction
import hashlib
from itertools import combinations_with_replacement
import json
import math
from pathlib import Path
import sys

from audits import hit_advancement as hit


def legal_vectors(value, mask):
    """Full ordered safe support; no independent runner-trial approximation."""
    if type(value) is not int or value not in (1,2) or type(mask) is not int or not 0<=mask<=7:
        raise ValueError('Invalid hit value/base mask')
    origins=[i+1 for i in range(3) if mask & (1<<i)]
    vectors=[]
    for targets in combinations_with_replacement(range(1,5),len(origins)):
        vector=(value,*targets)
        if (all(target>=origin for target,origin in zip(targets,origins))
                and all(a<b or a==b==4 for a,b in zip(vector,vector[1:]))):
            vectors.append(vector)
    return tuple(vectors)


def count_total(counts):
    if (not isinstance(counts,(list,tuple)) or not counts
            or any(type(n) is not int or not 0<=n<2**64 for n in counts)):
        raise ValueError('Invalid u64 count vector')
    total=sum(counts)
    if total>=2**64:
        raise ValueError('u64 count aggregation overflow')
    return total


def jeffreys(counts):
    """Exact half per structural category, never per observed category."""
    total=count_total(counts)
    return tuple(Fraction(2*n+1,2*total+len(counts)) for n in counts)


def scores(probabilities, counts):
    """Count-weighted proper scores; zero scoring exposure is unmeasured."""
    total=count_total(counts)
    if (not isinstance(probabilities,(tuple,list)) or len(probabilities)!=len(counts)
            or any(type(p) is not Fraction or p<=0 for p in probabilities)
            or sum(probabilities)!=1):
        raise ValueError('Invalid exact probability vector')
    if not total:
        return dict(events=0,negative_log_score=None,brier_score=None)
    log_score=math.fsum(-n*math.log(float(p)) for n,p in zip(counts,probabilities))/total
    brier=math.fsum(n*float(sum((p-int(i==j))**2 for j,p in enumerate(probabilities)))
        for i,n in enumerate(counts))/total
    if not all(math.isfinite(score) and score>=0 for score in (log_score,brier)):
        raise ValueError('Invalid finite proper score')
    return dict(events=total,negative_log_score=log_score,brier_score=brier)


MODELS=('uniform-legal-v1','pooled-mask-jeffreys-v1','base-out-jeffreys-v1')


def _collect(report):
    """Check every qualified vector/partition before any prior or score is computed.

    This pure aggregate seam does not authenticate source evidence. Publication
    separately reruns the unchanged full audit and requires accepted byte equality.
    """
    if (not isinstance(report,dict) or report.get('format')!=hit.FORMAT
            or not isinstance(report.get('joint_vectors'),list)):
        raise ValueError('Invalid hit qualification report')
    splits=('training','heldout','excluded')
    keys=('candidate_events','qualified_events','unresolved_events')
    totals={key:0 for key in keys}
    split_totals={split:dict(totals) for split in splits}
    data={(value,mask,outs):{split:Counter() for split in splits}
        for value in (1,2) for mask in range(8) for outs in range(3)}
    for cell in report['joint_vectors']:
        if (not isinstance(cell,dict) or cell.get('result') not in ('Single','Double')
                or cell.get('split') not in splits or type(cell.get('qualified')) is not bool
                or type(cell.get('outs_before')) is not int or not 0<=cell['outs_before']<=2):
            raise ValueError('Invalid hit qualification context')
        value=1 if cell['result']=='Single' else 2
        support=legal_vectors(value,cell.get('base_mask'))
        count=count_total((cell.get('events'),))
        if not count:
            raise ValueError('Empty hit qualification cell')
        split=cell['split']
        label='qualified_events' if cell['qualified'] else 'unresolved_events'
        for counts in (totals,split_totals[split]):
            counts['candidate_events']+=count;counts[label]+=count
        if cell['qualified']:
            expected=['Batter',*[role for i,role in enumerate(hit.TARGETS[:3]) if cell['base_mask'] & (1<<i)]]
            children=cell.get('runners')
            if (not isinstance(children,list) or any(not isinstance(child,dict) for child in children)
                    or [child.get('baserunner') for child in children]!=expected
                    or any(child.get('base_end') not in hit.TARGETS for child in children)
                    or cell.get('reasons')!=[]):
                raise ValueError('Invalid qualified hit endpoint identities')
            vector=tuple(hit.TARGETS.index(child['base_end'])+1 for child in children)
            if vector not in support:
                raise ValueError('Qualified hit vector outside full legal support')
            data[(value,cell['base_mask'],cell['outs_before'])][split][vector]+=count
    count_total((totals['candidate_events'],))
    actual=dict(totals=totals,split_totals=split_totals)
    for name,expected in actual.items():
        received=report.get(name)
        groups=[received] if name=='totals' else list(received.values()) if isinstance(received,dict) else []
        if (not isinstance(received,dict) or not groups
                or any(not isinstance(group,dict) or set(group)!=set(keys)
                    or any(type(n) is not int or not 0<=n<2**64 for n in group.values()) for group in groups)
                or received!=expected):
            raise ValueError('Hit qualification partition conservation failed')
    return data,actual


def _fit(data):
    fitted={}
    for (value,mask,outs),counts in data.items():
        support=legal_vectors(value,mask)
        training=tuple(counts['training'][vector] for vector in support)
        pooled=tuple(sum(data[(value,mask,o)]['training'][vector] for o in range(3)) for vector in support)
        fitted[(value,mask,outs)]=dict(support=support,training_counts=training,
            models={MODELS[0]:(Fraction(1,len(support)),)*len(support),
                MODELS[1]:jeffreys(pooled),MODELS[2]:jeffreys(training)})
    return fitted


def fit(report):
    return _fit(_collect(report)[0])


def _summary(contexts):
    models={}
    for model in MODELS:
        counts=[context['scores'][model] for context in contexts]
        total=count_total(tuple(score['events'] for score in counts) or (0,))
        models[model]=dict(events=total,**{metric:math.fsum(score['events']*score[metric]
            for score in counts if score['events'])/total if total else None
            for metric in ('negative_log_score','brier_score')})
    comparisons={}
    for name,a,b in (('pooled-minus-uniform',MODELS[1],MODELS[0]),
            ('base-out-minus-uniform',MODELS[2],MODELS[0]),('base-out-minus-pooled',MODELS[2],MODELS[1])):
        delta={metric:models[a][metric]-models[b][metric] if models[a]['events'] else None
            for metric in ('negative_log_score','brier_score')}
        comparisons[name]=dict(delta,nonworsening_log=delta['negative_log_score']<=0 if models[a]['events'] else None,
            nonworsening_brier=delta['brier_score']<=0 if models[a]['events'] else None)
    return dict(models=models,comparisons=comparisons)


def inventory(report):
    data,coverage=_collect(report)
    contexts=[]
    for (value,mask,outs),context in _fit(data).items():
        heldout=tuple(data[(value,mask,outs)]['heldout'][vector] for vector in context['support'])
        presentation={}
        for name,q in context['models'].items():
            floats=tuple(float(p) for p in q)
            if (any(not math.isfinite(p) or p<=0 for p in floats)
                    or abs(math.fsum(floats)-1)>32*sys.float_info.epsilon or sum(q)!=1):
                raise ValueError('Invalid finite normalized probability presentation')
            presentation[name]=[dict(vector=list(vector),numerator=p.numerator,denominator=p.denominator,
                probability=float(p)) for vector,p in zip(context['support'],q)]
        contexts.append(dict(value=value,base_mask=mask,outs_before=outs,support_size=len(context['support']),
            training_counts=list(context['training_counts']),scoring_counts=list(heldout),
            training_events=count_total(context['training_counts']),scoring_events=count_total(heldout),
            model_training_events={MODELS[0]:0,MODELS[1]:count_total(tuple(
                sum(data[(value,mask,o)]['training'].values()) for o in range(3))),
                MODELS[2]:count_total(context['training_counts'])},
            probabilities=presentation,scores={name:scores(q,heldout) for name,q in context['models'].items()}))
        contexts[-1]['comparisons']=_summary(contexts[-1:])['comparisons']
    subsets={name:_summary([context for context in contexts if select(context)])
        for name,select in (('all',lambda c:True),('occupied',lambda c:c['base_mask']!=0),
            ('single',lambda c:c['value']==1),('double',lambda c:c['value']==2),
            ('outs0',lambda c:c['outs_before']==0),('outs1',lambda c:c['outs_before']==1),('outs2',lambda c:c['outs_before']==2))}
    return dict(format='retrospective-hit-advancement-comparison-v1',coverage=coverage,contexts=contexts,subsets=subsets,
        source_coverage={key:copy.deepcopy(report[key]) for key in
            ('outside_results','first_reasons','all_reasons','source_totals') if key in report},
        diagnostic='Retrospective reserved-split diagnostics; previously inspected, not fresh blinded validation. No integration approval.')


INPUT_SHA=hit.INPUT_SHA+('6551fe2e0146d844362fb704121d7b79a64d95ac49834990c4a490d35432dcf0',)
EXPORT_SHA=hit.EXPORT_SHA


def _pins(paths):
    return tuple(hashlib.sha256(path.read_bytes()).hexdigest() for path in paths)


def publish(snapshot, supplement, lineups, manifest, census, accepted, output):
    """Requalify complete accepted evidence before fitting; publish exclusively.

    A completed qualification artifact is intentionally retained even if subsequent
    scoring, finite serialization, input stability or exclusive publication fails.
    """
    output=Path(output)
    qualification=output.with_suffix('.qualification.json')
    if any(path.exists() or path.is_symlink() for path in (output,qualification)):
        raise ValueError('Refusing to overwrite comparison or qualification evidence')
    output=output.resolve();qualification=output.with_suffix('.qualification.json')
    if output.is_relative_to(Path(hit.__file__).resolve().parents[1]):
        raise ValueError('Audit output must be outside repository')
    inputs=list(map(Path,(snapshot,supplement,lineups,manifest,census,accepted)))
    pins=_pins(inputs)
    if pins!=INPUT_SHA:
        raise ValueError('Unsupported hit comparison input digest')
    exports=[inputs[4].with_suffix('.'+name+'.csv') for name in ('lineup','events','raw','groups')]
    queries=[Path(hit.runner.__file__).with_name(name) for name in
        ('2019-lineup-readiness.sql','2019-outcomes.sql','2019-raw-mechanisms.sql','2019-runner-transitions.sql')]
    recipes=[Path(__file__),*[Path(module.__file__) for module in
        (hit,hit.runner,hit.lineup,hit.outcome,hit.raw,hit.runner.joint_contact)]]
    frozen=inputs+exports+queries+recipes
    all_pins=_pins(frozen)
    if all_pins[:6]!=pins:
        raise ValueError('Hit comparison inputs changed during read')
    if all_pins[6:10]!=EXPORT_SHA:
        raise ValueError('Unsupported hit comparison native export digest')
    # This is the unchanged complete source/native/census validator and classifier,
    # not an aggregate shortcut. Its exclusive final-reread writer is unchanged.
    hit.publish(*inputs[:5],qualification)
    if output.resolve()!=output or qualification.resolve()!=qualification:
        raise ValueError('Hit comparison output path changed before fitting')
    qualified_bytes=qualification.read_bytes()
    if qualified_bytes!=inputs[5].read_bytes():
        raise ValueError('Hit qualification report byte mismatch')
    if _pins(frozen)!=all_pins:
        raise ValueError('Hit comparison inputs changed before fitting')
    qualified=json.loads(qualified_bytes,object_pairs_hook=hit.raw.unique_object)
    report=inventory(qualified)
    report['provenance']=dict(accepted_qualification_sha256=pins[5],input_sha256=list(pins),
        native_export_sha256=list(all_pins[6:10]),query_sha256=list(all_pins[10:14]),
        recipe_sha256=list(all_pins[14:]),source_audit=qualified['provenance'])
    if output.resolve()!=output or qualification.resolve()!=qualification:
        raise ValueError('Hit comparison output path changed before publication')
    hit.write_report(report,frozen+[qualification],all_pins+(pins[5],),output)
    return report


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    for name in ('snapshot','supplement','lineups','manifest','census','accepted','output'):
        parser.add_argument(name,type=Path)
    args=parser.parse_args()
    report=publish(args.snapshot,args.supplement,args.lineups,args.manifest,args.census,args.accepted,args.output)
    print(json.dumps(dict(format=report['format'],totals=report['coverage']['totals']),sort_keys=True))


if __name__=='__main__':
    main()
