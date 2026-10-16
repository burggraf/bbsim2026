"""Read-only original source cohort;no causal mechanism or fitted occurrence rate."""
from collections import Counter
import copy
import hashlib
import json
from audits import contact_proxy_qualification as proxy
from audits import contact_consequence_support as literal
from audits import joint_contact as joint

SOURCE_FIELDS = ('result', 'pa', 'base_mask', 'outs_before', 'event_outs',
    'event_runs', 'split', 'joint_category', 'contact_bin', 'residual_reason')


def _json(value):
    return json.dumps(value, sort_keys=True, allow_nan=False)


def _fold(cells, fields, weight='events'):
    result = Counter()
    for cell in cells:
        result[_json({name: cell[name] for name in fields})] += cell[weight]
    return result


def _reconcile(sources, members, accepted, legacy):
    original = accepted['literal_support']
    if (sum(c['events'] for c in sources) != original['totals']['real_events'] or
            sum(c['events'] for c in sources if c['pa'] == 1) != original['source_totals']['terminal_pa'] or
            sum(c['events'] for c in sources if c['pa'] != 1) != original['source_totals']['non_pa'] or
            sum(c['labeled_events'] for c in sources) != original['totals']['candidate_events'] or
            sum(c['overlap_events'] for c in sources) != original['totals']['overlap_events'] or
            any(not 0 <= c['overlap_events'] <= c['labeled_events'] <= c['events'] for c in sources)):
        raise ValueError('Contact joint-cohort source unit conservation failed')
    for split, group in original['splits'].items():
        if (sum(c['events'] for c in sources if c['split'] == split) != group['real_events'] or
                sum(c['labeled_events'] for c in sources if c['split'] == split) != group['candidate_events']):
            raise ValueError('Contact joint-cohort split conservation failed')
    categories = Counter(dict.fromkeys(joint.CATEGORIES, 0))
    exclusions = Counter(dict.fromkeys(legacy['exclusions'], 0))
    bins = Counter(dict.fromkeys(joint.BINS, 0))
    reasons = Counter()
    for cell in sources:
        category = cell['joint_category']
        if category in categories:
            categories[category] += cell['events']
        elif category in exclusions:
            exclusions[category] += cell['events']
        elif category != 'non_pa':
            raise ValueError('Unknown contact joint-cohort primary category')
        if category == 'contact_out':
            bins[cell['contact_bin']] += cell['events']
            if cell['contact_bin'] == 'residual':
                reasons[cell['residual_reason']] += cell['events']
    contact_cells = [dict(bin=c['contact_bin'], base_mask=c['base_mask'],
        outs_before=c['outs_before'], events=c['events']) for c in sources
        if c['joint_category'] == 'contact_out']
    if (dict(categories) != legacy['counts'] or dict(exclusions) != legacy['exclusions'] or
            dict(bins) != legacy['contact_bins'] or dict(reasons) != legacy['residual_reasons'] or
            _fold(contact_cells, ('bin', 'base_mask', 'outs_before')) !=
            _fold(legacy['contexts'], ('bin', 'base_mask', 'outs_before'))):
        raise ValueError('Contact joint-cohort complete legacy partition conservation failed')
    previous = [dict(family=c['family'], split=c['split'],
        event_geometry_proxy=c['event_geometry_proxy'], literal_shape=c['shape'],
        literal_reasons=c['reasons'], events=c['events']) for c in accepted['contexts']]
    axes = ('family', 'split', 'event_geometry_proxy', 'literal_shape', 'literal_reasons')
    if (_fold(members, axes) != _fold(previous, axes) or
            any(sum(c['events'] for c in members if c['family'] == family) != group['events']
                for family, group in accepted['families'].items())):
        raise ValueError('Contact joint-cohort family/geometry/literal conservation failed')
    labeled = _fold(sources, SOURCE_FIELDS, 'labeled_events')
    for key, events in _fold(members, SOURCE_FIELDS + ('family',)).items():
        parent = json.loads(key)
        parent.pop('family')
        if events > labeled[_json(parent)]:
            raise ValueError('Contact joint-cohort family exceeds distinct labeled source units')


def _project(rows, outcomes, accepted_proxy, accepted_joint, native_joint):
    contexts = {row['event_key']: row for row in outcomes}
    excluded = set(accepted_proxy['literal_support']['excluded_game_ids'])
    sources, labeled, overlaps, members = Counter(), Counter(), Counter(), Counter()
    for row in rows:
        if row['no_play_flag'] == 'true':
            continue
        context = contexts[row['event_key']]
        # The mandatory native map was fully reconciled before this function.
        facts = native_joint[row['event_key']]
        split = ('excluded' if row['game_id'] in excluded else
            'heldout' if hashlib.sha256(row['game_id'].encode('utf-8')).hexdigest()[0] in '012' else 'training')
        category = joint.GROUPS[row['result']] if row['result'] else 'non_pa'
        bin_name, reason = joint.classify(row, context) if category == 'contact_out' else (None, None)
        common = {name: facts[name] for name in SOURCE_FIELDS[:6]}
        common.update(split=split, joint_category=category,
            contact_bin=bin_name, residual_reason=reason)
        value = literal.classify(row, context)
        key = _json(common)
        sources[key] += 1
        labeled[key] += int(bool(value['labels']))
        overlaps[key] += int(len(value['labels']) > 1)
        for family in value['labels']:
            result = value['by_family'][family]
            cell = dict(common, family=family,
                event_geometry_proxy=not proxy.VETOES.intersection(result['reasons']),
                literal_shape=result['shape'], literal_reasons=result['reasons'])
            members[_json(cell)] += 1
    source_cells = [dict(json.loads(key), events=events, labeled_events=labeled[key],
        overlap_events=overlaps[key]) for key, events in sorted(sources.items())]
    member_cells = [dict(json.loads(key), events=events) for key, events in sorted(members.items())]
    _reconcile(source_cells, member_cells, accepted_proxy, accepted_joint)
    return dict(format='2019-contact-proxy-joint-cohort-v1',
        basis='Original real source events versus separate family memberships;not mechanisms,opportunities or rates',
        causal_certification='unavailable', proxy_support=accepted_proxy,
        joint_support=copy.deepcopy(accepted_joint), source_contexts=source_cells, memberships=member_cells)


def inventory(raw_rows, outcome_rows, supported_game_ids, *, native_groups,
              expected_census, native_joint, cohort, expected_support,
              expected_proxy, expected_joint):
    """Reconcile complete inherited evidence/baselines before any new counters."""
    if (any(value is None for value in (native_groups, expected_census,
            native_joint, cohort, expected_support, expected_proxy, expected_joint)) or
            any(not isinstance(value, dict) for value in
                (expected_support, expected_proxy, expected_joint))):
        raise ValueError('Complete contact joint-cohort reconciliation evidence is required')
    rows, outcomes = tuple(raw_rows), tuple(outcome_rows)
    accepted = proxy.inventory(rows, outcomes, supported_game_ids,
        native_groups=native_groups, expected_census=expected_census,
        native_joint=native_joint, cohort=cohort, expected_support=expected_support)
    if _json(accepted) != _json(expected_proxy):
        raise ValueError('Complete contact joint-cohort proxy baseline mismatch')
    observed_joint = joint.inventory(rows, outcomes)
    # Original publication metadata is preserved,not rewritten as fresh evidence.
    _json(expected_joint)
    expected_core = {k: v for k, v in expected_joint.items() if k != 'provenance'}
    if _json(observed_joint) != _json(expected_core):
        raise ValueError('Complete contact joint-cohort legacy baseline mismatch')
    return _project(rows, outcomes, accepted, expected_joint, native_joint)
