"""Authenticated-report count transcoder;no probability arithmetic or game integration."""
import hashlib
import json
import re
from audits import contact_proxy_response_counts as census
from audits import contact_proxy_joint_cohort as cohort_rules
from audits.raw_mechanisms import unique_object

RESPONSES = ('SF', 'SH', 'GIDP', 'remaining')
MAX_COUNT = 2147483647
MAX_BYTES = 16384


def _digest(value):
    if not isinstance(value, str) or re.fullmatch('[0-9a-f]{64}', value) is None:
        raise ValueError('Invalid expected source digest')


def _headers(source_sha256):
    return ['contact-proxy-context-counts-v1', 'model\tgeometry-proxy-context-jeffreys-v1',
            'season\t2019', 'split\ttraining', 'source_sha256\t'+source_sha256,
            'prior\t1\t2', 'classes\tSF\tSH\tGIDP\tremaining', 'contexts\t24']


def _decimal(token):
    if len(token) > 10 or re.fullmatch('0|[1-9][0-9]*', token) is None:
        raise ValueError('Invalid canonical count integer')
    value = int(token)
    if value > MAX_COUNT:
        raise ValueError('Count integer exceeds bound')
    return value


def parse(payload_bytes, expected_source_sha256):
    """Strict integer loading;expected digest metadata is not source authentication."""
    _digest(expected_source_sha256)
    if (type(payload_bytes) is not bytes or len(payload_bytes) > MAX_BYTES or
            not payload_bytes.endswith(b'\n') or b'\r' in payload_bytes):
        raise ValueError('Invalid count ASCII/LF length or encoding')
    try:
        lines = payload_bytes.decode('ascii')[:-1].split('\n')
    except UnicodeDecodeError as error:
        raise ValueError('Invalid count ASCII encoding') from error
    if len(lines) != 32 or lines[:8] != _headers(expected_source_sha256):
        raise ValueError('Invalid count headers,source digest or line count')
    contexts, total = [], 0
    for i, line in enumerate(lines[8:]):
        mask, outs = i//3, i%3
        fields = line.split('\t')
        if len(fields) != 8 or fields[:3] != ['C', str(mask), str(outs)]:
            raise ValueError('Invalid count context order or row shape')
        n, *counts = map(_decimal, fields[3:])
        supported = (mask >= 4 and outs < 2, mask in (1, 5) and outs < 2,
                     mask in (1, 5) and outs < 2, True)
        if sum(counts) != n or any(c and not legal for c, legal in zip(counts, supported)):
            raise ValueError('Invalid count conservation or structural support')
        total += n
        if total > MAX_COUNT:
            raise ValueError('Training aggregate exceeds count bound')
        contexts.append(dict(split='training', base_mask=mask, outs_before=outs,
            parent_events=n, response_events=dict(zip(RESPONSES, counts))))
    return dict(source_sha256=expected_source_sha256, training_events=total, contexts=contexts)


def _nonfinite(value):
    raise ValueError('Nonfinite source JSON is not evidence')


def encode(source_bytes, expected_source_sha256):
    """Select training only after external hash and complete typed report equality."""
    _digest(expected_source_sha256)
    if type(source_bytes) is not bytes or hashlib.sha256(source_bytes).hexdigest() != expected_source_sha256:
        raise ValueError('Complete training source digest mismatch')
    try:
        report = json.loads(source_bytes.decode('utf-8'), object_pairs_hook=unique_object,
                            parse_constant=_nonfinite)
        if not isinstance(report, dict):
            raise ValueError('Invalid complete training source report')
        expected = census._project(report['observable_support'])
        if cohort_rules._json(report) != cohort_rules._json(expected):
            raise ValueError('Complete training source report mismatch')
        lines = _headers(expected_source_sha256)
        for row in report['contexts']:
            if row['split'] == 'training':
                fields = [row['base_mask'], row['outs_before'], row['parent_events'],
                          *(row['response_events'][f] for f in RESPONSES)]
                lines.append('C\t'+'\t'.join(map(str, fields)))
        payload = ('\n'.join(lines)+'\n').encode('ascii')
        parse(payload, expected_source_sha256)
        return payload
    except (KeyError, IndexError, TypeError, UnicodeError) as error:
        raise ValueError('Invalid complete training source report') from error
