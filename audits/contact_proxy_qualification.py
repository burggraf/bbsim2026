"""Observed four-axis geometry descriptors,never repaired vectors or mechanisms."""
import copy
import json
from audits import contact_consequence_support as literal

VETOES = frozenset(('not_terminal_pa', 'conflicting_labels', 'unsupported_before', 'event_units'))


def _project(report):
    source = copy.deepcopy(report)
    contexts = source.pop('contexts')
    families = {family: dict(events=0, geometry_proxy=0, not_geometry_proxy=0)
        for family in literal.FAMILIES}
    for cell in contexts:
        events = cell['events']
        if type(events) is not int or events <= 0 or cell['family'] not in families:
            raise ValueError('Invalid contact proxy context units/family')
        # All literal facts,shape and ordered reasons remain untouched. This
        # conjunction does not certify handling,retirement,timing or a vector.
        matched = not VETOES.intersection(cell['reasons'])
        cell['event_geometry_proxy'] = matched
        group = families[cell['family']]
        group['events'] += events
        group['geometry_proxy' if matched else 'not_geometry_proxy'] += events
    for family, group in families.items():
        if (group['events'] != source['label_counts'][family] or
                group['geometry_proxy'] + group['not_geometry_proxy'] != group['events']):
            raise ValueError('Contact proxy family membership conservation failed')
    return dict(format='2019-contact-proxy-qualification-v1',
        basis='Original source family memberships and four-axis geometry;not repaired vectors,mechanisms or rates',
        causal_certification='unavailable', literal_support=source, contexts=contexts, families=families)


def inventory(raw_rows, outcome_rows, supported_game_ids, *, native_groups,
              expected_census, native_joint, cohort, expected_support):
    """Complete original reconciliation and strict baseline equality precede counters."""
    if (any(value is None for value in
            (native_groups, expected_census, native_joint, cohort, expected_support)) or
            not isinstance(expected_support, dict)):
        raise ValueError('Complete contact proxy reconciliation evidence is required')
    report = literal.inventory(raw_rows, outcome_rows, supported_game_ids,
        native_groups=native_groups, expected_census=expected_census,
        native_joint=native_joint, cohort=cohort)
    expected = {name: value for name, value in expected_support.items() if name != 'provenance'}
    # JSON equality distinguishes bool/int/float aliases that Python == merges;
    # nonfinite supplied values are refused,not silently accepted or normalized.
    if (json.dumps(report, sort_keys=True, allow_nan=False) !=
            json.dumps(expected, sort_keys=True, allow_nan=False)):
        raise ValueError('Complete contact proxy literal support baseline mismatch')
    return _project(report)
