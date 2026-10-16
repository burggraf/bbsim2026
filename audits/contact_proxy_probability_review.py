"""Private fixed-proposal computational review;not a probability loader or engine.

Expected digests must come from approved external authority,not presented metadata.
The public review always authenticates whole bytes;parsed objects are not a bypass.
"""
from fractions import Fraction
import hashlib
import json
import math
import re
import struct
from audits import contact_proxy_training_counts as counts

CLASSES = ('SF', 'SH', 'GIDP', 'remaining')
MODEL = 'geometry-proxy-context-jeffreys-v1'
MAX_WITNESS_BYTES = 16384
TOLERANCE = Fraction(32, 2**52)


def _authenticated_table(source_bytes, expected_source_sha256, count_pair, expected_count_sha256):
    if (not isinstance(expected_count_sha256, str) or
            re.fullmatch('[0-9a-f]{64}', expected_count_sha256) is None):
        raise ValueError('Invalid expected count artifact digest')
    if (type(count_pair) is not tuple or len(count_pair) != 2 or
            any(type(raw) is not bytes for raw in count_pair)):
        raise ValueError('Exactly two immutable count artifacts are required')
    # encode authenticates and reprojects the entire report before selecting training.
    canonical = counts.encode(source_bytes, expected_source_sha256)
    if any(hashlib.sha256(raw).hexdigest() != expected_count_sha256 or raw != canonical
           for raw in count_pair):
        raise ValueError('Approved count pair/source reconciliation mismatch')
    return counts.parse(canonical, expected_source_sha256)


def authenticate(source_bytes: bytes, expected_source_sha256: str,
                 count_pair: tuple[bytes, bytes], expected_count_sha256: str) -> None:
    """Authenticate before native invocation;return no mutable trusted parsed object."""
    _authenticated_table(source_bytes, expected_source_sha256, count_pair, expected_count_sha256)


def _bits(value):
    return struct.unpack('>Q', struct.pack('>d', value))[0]


def _bit_array(token):
    try:
        values = json.loads(token)
    except ValueError as error:
        raise ValueError('Invalid native bit array') from error
    if (type(values) is not list or len(values) != 4 or
            any(type(v) is not int or not 0 <= v < 2**64 for v in values) or
            json.dumps(values) != token):
        raise ValueError('Invalid canonical native bit array')
    return values


def _review_native(table: dict, native_bytes: bytes) -> list[dict]:
    """Internal comparison of a freshly strict-parsed table,not source authentication."""
    if (type(native_bytes) is not bytes or len(native_bytes) > MAX_WITNESS_BYTES or
            not native_bytes.endswith(b'\n') or b'\r' in native_bytes):
        raise ValueError('Invalid native witness ASCII/LF shape')
    try:
        lines = native_bytes.decode('ascii')[:-1].split('\n')
    except UnicodeDecodeError as error:
        raise ValueError('Invalid native witness ASCII') from error
    header = ['H', '0', table['source_sha256'], MODEL, str(table['training_events'])]
    if len(lines) != 25 or lines[0].split('\t') != header:
        raise ValueError('Invalid native witness header or record count')
    reviewed = []
    for index, row in enumerate(table['contexts']):
        mask, outs = divmod(index, 3)
        n = row['parent_events']; values = [row['response_events'][f] for f in CLASSES]
        # Literal support independently expresses the frozen geometry proposal.
        supported = [mask in (4, 5, 6, 7) and outs in (0, 1),
                     mask in (1, 5) and outs in (0, 1),
                     mask in (1, 5) and outs in (0, 1), True]
        denominator = 2*n+sum(supported)
        nums = [2*c+1 if ok else 0 for c, ok in zip(values, supported)]
        if sum(nums) != denominator:
            raise ValueError('Invalid exact review rational closure')
        expected = ['C', str(mask), str(outs), str(n), json.dumps(values),
                    json.dumps(supported), str(n == 0).lower(), json.dumps(nums), str(denominator)]
        fields = lines[index+1].split('\t')
        if len(fields) != 11 or fields[:9] != expected:
            raise ValueError('Native metadata/support/prior/count/rational mismatch')
        pbits, cbits = _bit_array(fields[9]), _bit_array(fields[10])
        probabilities = [struct.unpack('>d', struct.pack('>Q', b))[0] for b in pbits]
        if (any(not math.isfinite(p) or not 0 <= p <= 1 for p in probabilities) or
                pbits != [_bits(a/denominator) for a in nums]):
            raise ValueError('Native probability division/support bits mismatch')
        running = 0.0; cumulative = []
        for p in probabilities:
            running += p; cumulative.append(running)
        if (not math.isfinite(running) or abs(running-1.0) > float(TOLERANCE) or
                any(not 0 <= v <= 1 for v in cumulative[:3])):
            raise ValueError('Invalid raw native probability/CDF closure')
        cumulative[-1] = 1.0
        if cbits != list(map(_bits, cumulative)):
            raise ValueError('Native left-fold/sentinel CDF bits mismatch')
        exact_cdf = Fraction(); pe, ce = [], []
        for a, p, cdf in zip(nums, probabilities, cumulative):
            exact = Fraction(a, denominator); exact_cdf += exact
            p_error = abs(Fraction.from_float(p)-exact)
            c_error = abs(Fraction.from_float(cdf)-exact_cdf)
            if p_error > TOLERANCE or c_error > TOLERANCE:
                raise ValueError('Native/exact Fraction discrepancy exceeds bound')
            pe.append([p_error.numerator, p_error.denominator])
            ce.append([c_error.numerator, c_error.denominator])
        reviewed.append(dict(base_mask=mask, outs_before=outs, parent_events=n,
            response_events=dict(row['response_events']), supported=supported, prior_only=n == 0,
            numerators=nums, denominator=denominator, probability_bits=pbits, cdf_bits=cbits,
            probability_error=pe, cdf_error=ce))
    return reviewed


def review(source_bytes: bytes, expected_source_sha256: str,
           count_pair: tuple[bytes, bytes], expected_count_sha256: str,
           native_bytes: bytes) -> dict:
    """Reauthenticate original whole bytes,then review all24 native context records."""
    table = _authenticated_table(source_bytes, expected_source_sha256,
                                 count_pair, expected_count_sha256)
    contexts = _review_native(table, native_bytes)
    return dict(scope='geometry_proxy_fixed_proposal_review', model_id=MODEL,
        source_sha256=expected_source_sha256, counts_sha256=expected_count_sha256,
        native_sha256=hashlib.sha256(native_bytes).hexdigest(),
        training_events=table['training_events'], contexts=contexts,
        limits=dict(proper_scores=False, optimized_fitting=False, model_selection=False,
                    sampling=False, integration=False, causal_certification=False,
                    predictive_validation=False))
