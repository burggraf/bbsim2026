"""Distinct residual-parent response census;not rates or causal certificates."""
from collections import Counter
import copy
import json
from audits import contact_proxy_observable_targets as previous
from audits import contact_proxy_joint_cohort as cohort_rules


RESPONSES = ('SF', 'SH', 'GIDP', 'remaining')
SPLITS = ('training', 'heldout', 'excluded')
SOURCE_FIELDS = cohort_rules.SOURCE_FIELDS + previous.AFTER_FIELDS


def _project(accepted):
    """Allocate already reconciled source weights;no transition or rate model."""
    parents = [c for c in accepted['source_contexts'] if
        c['joint_category'] == 'contact_out' and c['contact_bin'] == 'residual']
    parent_events = sum(c['events'] for c in parents)
    if parent_events != accepted['joint_cohort_support']['joint_support']['contact_bins']['residual']:
        raise ValueError('Complete response-count residual parent conservation failed')
    sources = {}
    for cell in parents:
        if (type(cell['pa']) is not int or cell['pa'] != 1 or
                type(cell['base_mask']) is not int or not 0 <= cell['base_mask'] <= 7 or
                type(cell['outs_before']) is not int or not 0 <= cell['outs_before'] <= 2 or
                cell['split'] not in SPLITS):
            raise ValueError('Invalid response-count parent units/context')
        sources[cohort_rules._json({k: cell[k] for k in SOURCE_FIELDS})] = cell
    geometry = Counter()
    for member in accepted['memberships']:
        if not member['event_geometry_proxy']:
            continue
        key = cohort_rules._json({k: member[k] for k in SOURCE_FIELDS})
        family, mask, outs = member['family'], member['base_mask'], member['outs_before']
        if (key not in sources or family not in RESPONSES[:3] or outs not in (0, 1) or
                mask not in ((4, 5, 6, 7) if family == 'SF' else (1, 5))):
            raise ValueError('Unjoined or unsupported response-count geometry')
        geometry[key, family] += member['events']
    cells, contexts, totals = [], Counter(), Counter()
    for key, source in sorted(sources.items()):
        allocated = sum(geometry[key, f] for f in RESPONSES[:3])
        # Distinct families have disjoint out/run units in the unchanged predicates.
        if sum(geometry[key, f] > 0 for f in RESPONSES[:3]) > 1:
            raise ValueError('Conflicting response-count geometry allocation')
        for response in RESPONSES:
            n = source['events'] - allocated if response == 'remaining' else geometry[key, response]
            labeled = source['labeled_events'] - allocated if response == 'remaining' else n
            overlap = source['overlap_events'] if response == 'remaining' else 0
            if not 0 <= overlap <= labeled <= n:
                raise ValueError('Invalid response-count source weight allocation')
            if n:
                cells.append(dict(json.loads(key), response=response, events=n,
                    labeled_events=labeled, overlap_events=overlap))
                contexts[source['split'], source['base_mask'], source['outs_before'], response] += n
                totals[response] += n
    for weight in ('events', 'labeled_events', 'overlap_events'):
        if cohort_rules._fold(cells, SOURCE_FIELDS, weight) != cohort_rules._fold(parents, SOURCE_FIELDS, weight):
            raise ValueError('Complete response-count source weight conservation failed')
    rows = []
    for split in SPLITS:
        for mask in range(8):
            for outs in range(3):
                counts = {f: contexts[split, mask, outs, f] for f in RESPONSES}
                rows.append(dict(split=split, base_mask=mask, outs_before=outs,
                    parent_events=sum(counts.values()), response_events=counts))
    if (sum(r['parent_events'] for r in rows) != parent_events or
            sum(totals.values()) != parent_events or
            cohort_rules._fold(rows, ('split', 'base_mask', 'outs_before'), 'parent_events') !=
            cohort_rules._fold(parents, ('split', 'base_mask', 'outs_before'))):
        raise ValueError('Complete response-count context conservation failed')
    return dict(format='2019-contact-proxy-response-counts-v1',
        basis='Complete distinct residual-parent source-event counts;not causes,opportunities or fitted rates',
        causal_certification='unavailable', historical_canonical_eligibility='unavailable',
        response_order=list(RESPONSES), observable_support=copy.deepcopy(accepted),
        source_responses=cells, contexts=rows,
        totals=dict(parent_events=parent_events,
            labeled_parent_events=sum(c['labeled_events'] for c in parents),
            overlap_parent_events=sum(c['overlap_events'] for c in parents),
            response_events={f: totals[f] for f in RESPONSES}))


def inventory(raw_rows, outcome_rows, supported_game_ids, *, native_groups,
              expected_census, native_joint, cohort, expected_support, expected_proxy,
              expected_joint, expected_joint_cohort, canonical_shapes,
              expected_observable_targets):
    """Complete inherited and whole-observable reconciliation before new counters."""
    if not isinstance(expected_observable_targets, dict):
        raise ValueError('Complete response-count observable evidence is required')
    rows, outcomes = tuple(raw_rows), tuple(outcome_rows)
    accepted = previous.inventory(rows, outcomes, supported_game_ids,
        native_groups=native_groups, expected_census=expected_census,
        native_joint=native_joint, cohort=cohort, expected_support=expected_support,
        expected_proxy=expected_proxy, expected_joint=expected_joint,
        expected_joint_cohort=expected_joint_cohort, canonical_shapes=canonical_shapes)
    if cohort_rules._json(accepted) != cohort_rules._json(expected_observable_targets):
        raise ValueError('Complete response-count observable baseline mismatch')
    return _project(accepted)
