"""Independent offline audit of fixed player-calibration candidate probabilities and evidence."""
import argparse
from collections import defaultdict
import csv
import hashlib
import json
import math
from pathlib import Path
from audits import player_calibration as preparation
from audits import outcome_inventory as outcome
from audits.raw_mechanisms import unique_object

KEYS=preparation.CATEGORIES
RISK=((0,1,2,3,4,5,6,7,8),(1,2,3,4,5,6,7,8),(2,3,4,5,6,7,8),(3,4,5,6,7,8),(3,4,5,6),(3,4,5),(3,4),(7,8))
YES=((0,),(1,),(2,),(3,4,5,6),(6,),(5,),(4,),(7,))


def read_effects(text,required):
    if any(ord(c)<32 and c not in '\t\n' for c in text):raise ValueError('Invalid calibration control')
    lines=text.splitlines()
    if not lines or lines[0]!='key\tvalue':raise ValueError('Invalid calibration header')
    fields={}
    for line in lines[1:]:
        cells=line.split('\t')
        if len(cells)!=2 or not all(cells) or cells[0] in fields:raise ValueError('Invalid/duplicate calibration field')
        fields[cells[0]]=cells[1]
    version=fields.pop('format',None)
    tree=version=='player-calibration-tree-candidate-v2'
    if not tree and version!='player-calibration-candidate-v1':raise ValueError('Unsupported calibration version')
    if not tree:
        candidate=fields.pop('candidate',None)
        if not candidate:raise ValueError('Missing calibration candidate')
    rates=fields.pop('league_nodes' if tree else 'league',None)
    def vector(value):
        try:q=tuple(map(float,value.split(',')))
        except (AttributeError,ValueError) as error:raise ValueError('Invalid calibration vector') from error
        if len(q)!=(8 if tree else 9) or any(not math.isfinite(x) or not 0<x<1 for x in q) or (not tree and abs(sum(q)-1)>1e-12):raise ValueError('Invalid calibration vector')
        return q
    league=vector(rates)
    if set(fields)!=set(required):raise ValueError('Calibration role identity set mismatch')
    return dict(tree=tree,league=league,profiles={key:vector(value) for key,value in fields.items()})


def predict(model,batter,pitcher):
    try:b=model['profiles']['batter:'+batter];p=model['profiles']['pitcher:'+pitcher]
    except KeyError as error:raise ValueError('Missing current calibration role') from error
    league=model['league']
    if not model['tree']:
        w=[x*y/z for x,y,z in zip(b,p,league)];total=sum(w);return tuple(x/total for x in w)
    r=[]
    for x,y,z in zip(b,p,league):
        yes=x*y/z;no=(1-x)*(1-y)/(1-z);r.append(yes/(yes+no))
    rem=(1-r[0])*(1-r[1])*(1-r[2]);hits=rem*r[3];nonhits=rem*(1-r[3])
    return (r[0],(1-r[0])*r[1],(1-r[0])*(1-r[1])*r[2],
        hits*(1-r[4])*(1-r[5])*(1-r[6]),hits*(1-r[4])*(1-r[5])*r[6],
        hits*(1-r[4])*r[5],hits*r[4],nonhits*r[7],nonhits*(1-r[7]))


def cluster_difference(games):
    if len(games)<2 or any(not math.isfinite(d) or n<=0 for d,n in games):raise ValueError('Invalid game-cluster population')
    n=sum(x[1] for x in games);mean=sum(x[0] for x in games)/n
    se=math.sqrt(len(games)/(len(games)-1)*sum((d-mean*size)**2 for d,size in games))/n
    return [mean,mean-1.96*se,mean+1.96*se]


def guardrails(league,baseline):
    return (len(league)==len(baseline)==3 and all(math.isfinite(x) for x in [*league,*baseline])
        and league[1]<=league[0]<=league[2]<0 and baseline[1]<=baseline[0]<=baseline[2] and baseline[0]<=.01)


def _close(a,b,label,tolerance=1e-10):
    if not math.isfinite(a) or not math.isfinite(b) or abs(a-b)>tolerance:raise ValueError(f'Calibration numerical mismatch: {label}: {a!r} vs {b!r}')


def _league(counts):
    n=sum(counts);return tuple((c+.5)/(n+4.5) for c in counts)


def _baseline(counts):
    league=_league([sum(c[k] for key,c in counts.items() if key.startswith('batter:')) for k in range(9)])
    return dict(tree=False,league=league,profiles={key:tuple((x+200*league[k])/(sum(c)+200) for k,x in enumerate(c)) if sum(c) else league for key,c in counts.items()})


def _moments(model,pairs,counts):
    expected={key:[0.0]*9 for key in counts};variance={key:[0.0]*9 for key in counts};minimum=1.0
    for _,_,b,p,c in pairs:
        q=predict(model,b,p);n=sum(c);minimum=min(minimum,*q)
        if any(not math.isfinite(x) or not 0<x<1 for x in q):raise ValueError('Invalid consumed calibration vector')
        _close(sum(q),1,'complete probability mass')
        for role,key in [('batter',b),('pitcher',p)]:
            e=expected[role+':'+key];v=variance[role+':'+key]
            for k,x in enumerate(q):e[k]+=n*x;v[k]+=n*x*(1-x)
    return expected,variance,minimum


def _bias(counts,expected):
    output={}
    for role in ['batter','pitcher']:
        for k in range(10):
            values=[]
            for key,c in sorted(counts.items()):
                n=sum(c)
                if key.startswith(role+':') and n>=200:
                    delta=((sum(expected[key][3:7])-sum(c[3:7])) if k==9 else expected[key][k]-c[k])/n
                    values.append((key.split(':',1)[1],delta))
            worst=max(values,key=lambda x:abs(x[1])) if values else ('',0)
            output[role,'hits' if k==9 else KEYS[k]]=dict(players=len(values),mae=sum(abs(v) for _,v in values)/len(values) if values else 0,maximum=abs(worst[1]),worst=worst[0],signed=worst[1])
    return output


def audit(events,source_report,input_dir,result_dir,output):
    if output.exists():raise ValueError('Refusing existing calibration audit')
    if output.resolve().is_relative_to(Path(__file__).resolve().parents[1]):raise ValueError('Audit output must be outside repository')
    paths=[events,source_report,input_dir/'opportunities.tsv',input_dir/'source.json',result_dir/'training-effects.tsv',result_dir/'full-effects.tsv',result_dir/'selection.json',result_dir/'test.csv',result_dir/'full-bias.csv',result_dir/'baseline-bias.csv',result_dir/'variation.csv',result_dir/'ensembles.csv',Path(__file__),Path(preparation.__file__)]
    before={str(p):outcome.digest(p) for p in paths}
    if (before[str(events)],before[str(source_report)])!=(preparation.EVENT_SHA,preparation.REPORT_SHA):raise ValueError('Unsupported calibration audit source')
    original=json.loads(source_report.read_text(),object_pairs_hook=unique_object)
    with events.open(newline='') as stream:
        reader=csv.DictReader(stream,strict=True)
        if reader.fieldnames is None or len(reader.fieldnames)!=len(set(reader.fieldnames)) or set(reader.fieldnames)!=outcome.FIELDS:raise ValueError('Invalid audit source header')
        pairs,report=preparation.collect(reader)
    if report!={k:v for k,v in original.items() if k!='provenance'}:raise ValueError('Audit source/profile reconciliation failed')
    encoded=preparation.encode(pairs)
    if encoded!=(input_dir/'opportunities.tsv').read_text():raise ValueError('Calibration game partition/source differs')
    source=json.loads((input_dir/'source.json').read_text(),object_pairs_hook=unique_object)
    if source['dataset_sha256']!=hashlib.sha256(encoded.encode()).hexdigest():raise ValueError('Calibration dataset digest differs')
    counts={role+':'+p['player_id']:[p['counts'][k] for k in KEYS] for role,data in report['roles'].items() for p in data['profiles']}
    training={key:[0]*9 for key in counts}
    for fold,_,b,p,c in pairs:
        if fold>=2:
            for key in ['batter:'+b,'pitcher:'+p]:
                for k,x in enumerate(c):training[key][k]+=x
    model=read_effects((result_dir/'full-effects.tsv').read_text(),counts)
    trained=read_effects((result_dir/'training-effects.tsv').read_text(),counts)
    full_league=_league([report['counts'][k] for k in KEYS]);train_league=_baseline(training)['league']
    for fx,league in [(model,full_league),(trained,train_league)]:
        rates=[sum(league[k] for k in yes)/sum(league[k] for k in risk) for yes,risk in zip(YES,RISK)] if fx['tree'] else league
        for x,y in zip(fx['league'],rates):_close(x,y,'training/full league')
    cold={role:0 for role in ['batter','pitcher']}
    for key,c in training.items():
        if sum(c)==0:cold[key.split(':')[0]]+=1
        for i,rate in enumerate(trained['profiles'][key]):
            n=sum(c[k] for k in RISK[i]) if trained['tree'] else sum(c)
            if n==0:_close(rate,trained['league'][i],'cold-start conditional neutral',1e-15)
    baseline=_baseline(training);neutral=dict(baseline,profiles={key:baseline['league'] for key in counts})
    games_base=defaultdict(lambda:[0.0,0]);games_league=defaultdict(lambda:[0.0,0]);observed=[0]*9;expected=[0.0]*9;n=0;loss=0.0;brier=0.0
    for fold,game,b,p,c in pairs:
        if fold!=0:continue
        q=predict(trained,b,p);qb=predict(baseline,b,p);ql=neutral['league'];size=sum(c);n+=size
        one=-sum(x*math.log(y) for x,y in zip(c,q));loss+=one
        sq=sum(x*x for x in q);brier+=sum(c[k]*(sq+1-2*q[k]) for k in range(9))
        for k in range(9):observed[k]+=c[k];expected[k]+=size*q[k]
        for model_q,groups in [(qb,games_base),(ql,games_league)]:
            groups[game][0]+=one+sum(x*math.log(y) for x,y in zip(c,model_q));groups[game][1]+=size
    vs_league=cluster_difference(list(games_league.values()));vs_baseline=cluster_difference(list(games_base.values()))
    selection=json.loads((result_dir/'selection.json').read_text(),object_pairs_hook=unique_object)
    for key,value in [('test_nll_difference_vs_league',vs_league),('test_nll_difference_vs_fixed200',vs_baseline)]:
        for x,y in zip(selection[key],value):_close(x,y,key)
    label='outcome-tree-v2' if trained['tree'] else selection['candidate']
    with (result_dir/'test.csv').open(newline='') as stream:
        rows=list(csv.DictReader(stream,strict=True))
    selected=next(r for r in rows if r['candidate']==label)
    for field,value in [('n',n),('nll',loss/n),('brier',brier/n)]:_close(float(selected[field]),value,field)
    for field,values in [('observed',observed),('expected',expected)]:
        for x,y in zip(json.loads(selected[field]),values):_close(x,y,field,1e-8)
    e,v,minimum=_moments(model,pairs,counts);old_e,_,_=_moments(_baseline(counts),pairs,counts)
    bias=_bias(counts,e);old_bias=_bias(counts,old_e)
    for filename,reference in [('full-bias.csv',bias),('baseline-bias.csv',old_bias)]:
        with (result_dir/filename).open(newline='') as stream:rows=list(csv.DictReader(stream,strict=True))
        if len(rows)!=20:raise ValueError('Incomplete bias report')
        for row in rows:
            item=reference[row['role'],row['category']]
            for field,key in [('players','players'),('mae_rate','mae'),('max_abs_rate','maximum'),('worst_signed_rate','signed')]:_close(float(row[field]),item[key],filename)
            if row['worst_id']!=item['worst']:raise ValueError('Wrong worst-profile identity')
    reconstruction=True
    for (role,category),item in bias.items():
        if category in ['strikeout','walk','home_run','hits']:
            reconstruction&=item['mae']<=.01 and item['maximum']<=.03 and item['mae']<=old_bias[role,category]['mae'] and item['maximum']<=old_bias[role,category]['maximum']
        elif category in ['hbp','triple','roe']:reconstruction&=item['mae']<=.005 and item['maximum']<=.02
    histories={key:[[] for _ in KEYS] for key in counts};seen=set();totals=defaultdict(lambda:[0]*9)
    with (result_dir/'ensembles.csv').open(newline='') as stream:
        for row in csv.DictReader(stream,strict=True):
            seed=int(row['seed']);key=row['role']+':'+row['player_id'];identity=(seed,key)
            if not 1<=seed<=200 or key not in counts or identity in seen:raise ValueError('Invalid ensemble grain/identity')
            seen.add(identity);values=[int(row[k]) for k in KEYS]
            if any(x<0 for x in values) or sum(values)!=sum(counts[key]):raise ValueError('Ensemble opportunity conservation failed')
            for k,x in enumerate(values):histories[key][k].append(x);totals[seed,row['role']][k]+=x
    if len(seen)!=200*len(counts) or any(totals[s,'batter']!=totals[s,'pitcher'] for s in range(1,201)):raise ValueError('Incomplete/nonconserving ensemble')
    variation=True;variance_failures=[];seen=set()
    with (result_dir/'variation.csv').open(newline='') as stream:
        for row in csv.DictReader(stream,strict=True):
            key=row['role']+':'+row['player_id'];k=KEYS.index(row['category']);identity=(key,k)
            if identity in seen:raise ValueError('Duplicate variation cell')
            seen.add(identity);samples=histories[key][k];mean=sum(samples)/200;sd=math.sqrt(sum((x-mean)**2 for x in samples)/199);ordered=sorted(samples)
            for field,value in [('exposure',sum(counts[key])),('observed',counts[key][k]),('expected',e[key][k]),('variance',v[key][k]),('mean',mean),('sd',sd),('q05',ordered[9]),('median',ordered[99]),('q95',ordered[189])]:_close(float(row[field]),value,field,1e-8)
            mean_ok=abs(mean-e[key][k])<=5*math.sqrt(v[key][k]/200)+.05
            sd_ok=(v[key][k]<5 or sd>0) and (v[key][k]<25 or abs(sd/math.sqrt(v[key][k])-1)<=.2)
            if (row['mean_pass']=='true')!=mean_ok or (row['sd_pass']=='true')!=sd_ok:raise ValueError('Variation disposition mismatch')
            if not (mean_ok and sd_ok):variance_failures.append([key,KEYS[k]])
            variation&=mean_ok and sd_ok
    if len(seen)!=9*len(counts):raise ValueError('Incomplete variation population')
    gates=guardrails(vs_league,vs_baseline)
    for key,value in [('heldout_guardrails',gates),('full_reconstruction',reconstruction),('variation',variation),('accepted',gates and reconstruction and variation)]:
        if selection[key]!=value:raise ValueError('Calibration acceptance disposition mismatch')
    if {str(p):outcome.digest(p) for p in paths}!=before:raise ValueError('Calibration evidence changed during independent audit')
    result=dict(format='independent-player-calibration-audit-v1',candidate=label,accepted=selection['accepted'],
        reused_test_benchmark=model['tree'],test_opportunities=n,test_games=len(games_base),test_nll=loss/n,test_brier=brier/n,
        vs_league=vs_league,vs_fixed200=vs_baseline,full_minimum_probability=minimum,cold_training_profiles=cold,
        role_profiles=len(counts),variation_cells=len(seen),ensemble_rows=200*len(counts),variation_failures=variance_failures,
        full_bias={role+':'+category:value for (role,category),value in bias.items()},pins=before)
    with output.open('x') as stream:json.dump(result,stream,sort_keys=True,indent=2);stream.write('\n')
    return result


CAST_SHA='d284339066222212dc79c57b284bb764d49a156937b2b6762ac943a7ccec2bec'
STRENGTHS='20,20;20,20;20,20;20,20;20,20;200,200;200,200;200,200'

def encode_tree(model,counts,audit_sha,selection_sha):
    from preview import real_lineup as real
    from preview import player_ratings as ratings
    if not model['tree'] or not counts or any(key not in model['profiles'] or len(c)!=9 or any(type(x) is not int or not 0<=x<=cap for x,cap in zip(c,ratings.COUNTS)) for key,c in counts.items()):raise ValueError('Invalid closed tree role/count population')
    if any(len(h)!=64 or any(c not in '0123456789abcdef' for c in h) for h in [audit_sha,selection_sha]):raise ValueError('Invalid tree evidence digest')
    fields=dict(format='closed-opponent-tree-ratings-v2',model='outcome-tree-player-proxy-v2-development',
        opportunity_report_sha256=ratings.REPORT_SHA,snapshot_sha256=real.SNAPSHOT_SHA,supplement_sha256=real.SUPPLEMENT_SHA,
        cast_sha256=CAST_SHA,reused_test_benchmark='true',strengths=STRENGTHS,attribution=real.ATTRIBUTION,
        audit_sha256=audit_sha,selection_sha256=selection_sha,league_nodes=','.join(map(str,model['league'])))
    for key,c in counts.items():
        fields['counts:'+key]=','.join(map(str,c));fields[key]=','.join(map(str,model['profiles'][key]))
    return 'key\tvalue\n'+''.join(key+'\t'+value+'\n' for key,value in sorted(fields.items()))


def export_tree(result_dir,audit_file,cast,output):
    from preview import pitching_cast
    if output.exists():raise ValueError('Refusing existing closed tree artifact')
    if output.resolve().is_relative_to(Path(__file__).resolve().parents[1]):raise ValueError('Tree artifact must be outside repository')
    if outcome.digest(cast)!=CAST_SHA:raise ValueError('Unsupported closed tree cast')
    audit_result=json.loads(audit_file.read_text(),object_pairs_hook=unique_object)
    if audit_result.get('format')!='independent-player-calibration-audit-v1' or audit_result.get('candidate')!='outcome-tree-v2' or audit_result.get('accepted') is not True or audit_result.get('reused_test_benchmark') is not True:raise ValueError('Tree development evidence did not pass')
    for path,digest in audit_result['pins'].items():
        if outcome.digest(Path(path))!=digest:raise ValueError('Tree audit input changed')
    selection_path=result_dir/'selection.json';effects_path=result_dir/'full-effects.tsv'
    for path in [selection_path,effects_path]:
        if audit_result['pins'].get(str(path))!=outcome.digest(path):raise ValueError('Tree evidence binding mismatch')
    selection=json.loads(selection_path.read_text(),object_pairs_hook=unique_object)
    if selection.get('accepted') is not True or selection.get('strengths')!=[[20,20]]*5+[[200,200]]*3:raise ValueError('Unsupported tree selection')
    report=json.loads(Path(next(p for p in audit_result['pins'] if audit_result['pins'][p]==preparation.REPORT_SHA)).read_text(),object_pairs_hook=unique_object)
    required={role+':'+p['player_id'] for role,data in report['roles'].items() for p in data['profiles']}
    model=read_effects(effects_path.read_text(),required)
    cast_fields=dict(line.split('\t') for line in cast.read_text().splitlines()[1:])
    counts={key:list(map(int,value.split(','))) for key,value in cast_fields.items() if key.startswith(('batter:','pitcher:'))}
    if sum(key.startswith('batter:') for key in counts)!=18 or sum(key.startswith('pitcher:') for key in counts)!=12:raise ValueError('Incomplete tree cast')
    inputs=tuple(Path(p) for p in audit_result['pins'])+(audit_file,cast)
    pins=tuple(audit_result['pins'].values())+(outcome.digest(audit_file),CAST_SHA)
    text=encode_tree(model,counts,outcome.digest(audit_file),outcome.digest(selection_path))
    pitching_cast.publish(text,inputs,pins,output)
    return text


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    for name in ['events','source_report','input_dir','result_dir','output']:parser.add_argument(name,type=Path)
    args=parser.parse_args();result=audit(args.events,args.source_report,args.input_dir,args.result_dir,args.output)
    print(json.dumps({k:v for k,v in result.items() if k not in ['pins','full_bias']},sort_keys=True))

if __name__=='__main__':main()
