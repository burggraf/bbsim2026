"""Direct after descriptors versus supplied fictional canonical shapes,not causes."""
from collections import Counter
import copy
import hashlib
import itertools
import json
from audits import contact_proxy_joint_cohort as previous

AFTER_FIELDS = ('frame_end', 'after_outs', 'after_base_mask')
SHAPE_FIELDS = frozenset(('family base_mask outs_before delta_outs delta_runs '
    'event_end_outs normalized_after_outs normalized_after_base_mask frame_changed').split())


def _shapes(reference):
    if (not isinstance(reference, dict) or set(reference) !=
            {'format', 'policy', 'control_state', 'assumptions', 'contexts'} or
            reference['format'] != 'contact-proxy-controlled-shapes-v1' or
            reference['policy'] != 'supplied-contact-consequence-projection-v1' or
            previous._json(reference['control_state']) !=
            previous._json(dict(inning=2, half='Top', score=[0, 0])) or
            not isinstance(reference['assumptions'], str) or not reference['assumptions'] or
            not isinstance(reference['contexts'], list)):
        raise ValueError('Invalid supplied canonical controlled-shape reference')
    previous._json(reference)
    expected = {(f, mask, outs) for f in ('SF', 'SH', 'GIDP') for mask, outs in
        itertools.product((4, 5, 6, 7) if f == 'SF' else (1, 5), (0, 1))}
    result = {}
    for row in reference['contexts']:
        if not isinstance(row, dict) or set(row) != SHAPE_FIELDS or type(row['family']) is not str:
            raise ValueError('Invalid supplied canonical shape fields')
        for name, maximum in [('base_mask', 7), ('outs_before', 1), ('delta_outs', 3),
                ('delta_runs', 1), ('event_end_outs', 3), ('normalized_after_outs', 2),
                ('normalized_after_base_mask', 7)]:
            if type(row[name]) is not int or not 0 <= row[name] <= maximum:
                raise ValueError('Invalid supplied canonical shape integer kind/domain')
        key = row['family'], row['base_mask'], row['outs_before']
        if key not in expected or key in result or type(row['frame_changed']) is not bool:
            raise ValueError('Invalid supplied canonical context grain/boundary kind')
        end = row['outs_before'] + row['delta_outs']
        changed = row['frame_changed']
        if (row['delta_outs'] != (2 if row['family'] == 'GIDP' else 1) or
                row['delta_runs'] != int(row['family'] == 'SF') or
                row['event_end_outs'] != end or changed != (end == 3) or
                row['normalized_after_outs'] != (0 if changed else end) or
                changed and row['normalized_after_base_mask'] != 0):
            raise ValueError('Inconsistent supplied canonical delta/normalization units')
        result[key] = row
    if set(result) != expected:
        raise ValueError('Incomplete supplied canonical shape contexts')
    return result


def _project(rows, outcomes, accepted, canonical_shapes, native_joint):
    """Finite reference lookup/comparison only;Rust owns legal transitions."""
    contexts = {row['event_key']: row for row in outcomes}
    shapes = {(r['family'], r['base_mask'], r['outs_before']): r
        for r in canonical_shapes['contexts']}
    excluded = set(accepted['proxy_support']['literal_support']['excluded_game_ids'])
    sources, labeled, overlaps, members = Counter(), Counter(), Counter(), Counter()
    for row in rows:
        if row['no_play_flag'] == 'true':
            continue
        context, facts = contexts[row['event_key']], native_joint[row['event_key']]
        category = previous.joint.GROUPS[row['result']] if row['result'] else 'non_pa'
        bin_name, reason = previous.joint.classify(row, context) if category == 'contact_out' else (None, None)
        split = ('excluded' if row['game_id'] in excluded else
            'heldout' if hashlib.sha256(row['game_id'].encode('utf-8')).hexdigest()[0] in '012' else 'training')
        common = {name: facts[name] for name in previous.SOURCE_FIELDS[:6] + AFTER_FIELDS}
        common.update(split=split, joint_category=category, contact_bin=bin_name, residual_reason=reason)
        value = previous.literal.classify(row, context)
        key = previous._json(common)
        sources[key] += 1
        labeled[key] += int(bool(value['labels']))
        overlaps[key] += int(len(value['labels']) > 1)
        for family in value['labels']:
            literal = value['by_family'][family]
            geometry = not previous.proxy.VETOES.intersection(literal['reasons'])
            out_relation = mask_relation = 'not_geometry_proxy'
            if geometry:
                shape = shapes[family, facts['base_mask'], facts['outs_before']]
                out_relation = ('unknown' if facts['after_outs'] is None else
                    'match' if facts['after_outs'] == shape['event_end_outs'] else 'different')
                mask_relation = ('not_compared_boundary' if facts['frame_end'] or shape['frame_changed'] else
                    'unknown' if facts['after_base_mask'] is None else
                    'match' if facts['after_base_mask'] == shape['normalized_after_base_mask'] else 'different')
            cell = dict(common, family=family, event_geometry_proxy=geometry,
                literal_shape=literal['shape'], literal_reasons=literal['reasons'],
                literal_after_check_reached=literal['shape'] in
                    ('complete_literal', 'compatible_with_missing_retired_endpoint'),
                raw_out_sum_relation=out_relation, controlled_mask_relation=mask_relation)
            members[previous._json(cell)] += 1
    source_cells = [dict(json.loads(key), events=n, labeled_events=labeled[key], overlap_events=overlaps[key])
        for key, n in sorted(sources.items())]
    member_cells = [dict(json.loads(key), events=n) for key, n in sorted(members.items())]
    for weight in ('events', 'labeled_events', 'overlap_events'):
        if previous._fold(source_cells, previous.SOURCE_FIELDS, weight) != previous._fold(
                accepted['source_contexts'], previous.SOURCE_FIELDS, weight):
            raise ValueError('Complete observable-target source weight conservation failed')
    axes = previous.SOURCE_FIELDS + ('family', 'event_geometry_proxy', 'literal_shape', 'literal_reasons')
    if previous._fold(member_cells, axes) != previous._fold(accepted['memberships'], axes):
        raise ValueError('Complete observable-target membership conservation failed')
    return dict(format='2019-contact-proxy-observable-targets-v1',
        basis='Direct source after descriptors versus supplied fictional canonical shapes;not causes,opportunities or rates',
        causal_certification='unavailable', historical_canonical_eligibility='unavailable',
        joint_cohort_support=copy.deepcopy(accepted), canonical_shapes=copy.deepcopy(canonical_shapes),
        source_contexts=source_cells, memberships=member_cells)


def inventory(raw_rows, outcome_rows, supported_game_ids, *, native_groups,
              expected_census, native_joint, cohort, expected_support, expected_proxy,
              expected_joint, expected_joint_cohort, canonical_shapes):
    """Complete inherited reconciliation before any new descriptor counters."""
    if not isinstance(expected_joint_cohort, dict) or canonical_shapes is None:
        raise ValueError('Complete observable-target cohort/reference evidence is required')
    rows, outcomes = tuple(raw_rows), tuple(outcome_rows)
    accepted = previous.inventory(rows, outcomes, supported_game_ids,
        native_groups=native_groups, expected_census=expected_census,
        native_joint=native_joint, cohort=cohort, expected_support=expected_support,
        expected_proxy=expected_proxy, expected_joint=expected_joint)
    if previous._json(accepted) != previous._json(expected_joint_cohort):
        raise ValueError('Complete observable-target joint-cohort baseline mismatch')
    _shapes(canonical_shapes)
    return _project(rows, outcomes, accepted, canonical_shapes, native_joint)
