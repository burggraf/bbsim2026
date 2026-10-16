"""Pure private retrospective diagnostics;not a parameter loader or engine hook.

Expected digests come from closed external authority. Public entry points always
reauthenticate original complete bytes;parsed summaries are not a trust boundary.
Primary NLL is binary64 arithmetic,not an independent interval certificate.
"""
from fractions import Fraction
import hashlib
import json
import math
import re
import struct
from audits import contact_proxy_probability_review as probability_review

CLASSES = ('SF', 'SH', 'GIDP', 'remaining')
SPLITS = ('training', 'heldout', 'excluded')


def _digest(value):
    if not isinstance(value, str) or re.fullmatch('[0-9a-f]{64}', value) is None:
        raise ValueError('Invalid expected diagnostics authority digest')


def _inputs(source_bytes, expected_source_sha256, count_pair, expected_count_sha256,
            parameter_pair, expected_parameter_sha256, native_bytes, expected_native_sha256):
    _digest(expected_parameter_sha256); _digest(expected_native_sha256)
    if (type(native_bytes) is not bytes or
            hashlib.sha256(native_bytes).hexdigest() != expected_native_sha256):
        raise ValueError('Approved diagnostics native witness mismatch')
    if (type(parameter_pair) is not tuple or len(parameter_pair) != 2 or
            any(type(raw) is not bytes for raw in parameter_pair)):
        raise ValueError('Exactly two immutable parameter artifacts are required')
    proposal = probability_review.review(source_bytes, expected_source_sha256,
        count_pair, expected_count_sha256, native_bytes)
    canonical = (json.dumps(proposal, sort_keys=True, indent=2, allow_nan=False)+'\n').encode('ascii')
    if any(hashlib.sha256(raw).hexdigest() != expected_parameter_sha256 or raw != canonical
           for raw in parameter_pair):
        raise ValueError('Approved parameter pair/complete native review mismatch')
    # These same immutable whole bytes have already passed typed full reprojection.
    source = json.loads(source_bytes)
    for row in source['contexts']:
        p = proposal['contexts'][row['base_mask']*3+row['outs_before']]
        if any(row['response_events'][f] and not a for f, a in zip(CLASSES, p['numerators'])):
            raise ValueError('Positive diagnostic response at structural probability zero')
    return source, proposal


def authenticate(source_bytes: bytes, expected_source_sha256: str,
                 count_pair: tuple[bytes, bytes], expected_count_sha256: str,
                 parameter_pair: tuple[bytes, bytes], expected_parameter_sha256: str,
                 native_bytes: bytes, expected_native_sha256: str) -> None:
    """Authenticate whole evidence before output creation/scoring;return no token."""
    _inputs(source_bytes, expected_source_sha256, count_pair, expected_count_sha256,
            parameter_pair, expected_parameter_sha256, native_bytes, expected_native_sha256)


def _pair(value):
    value = Fraction(value)
    return [value.numerator, value.denominator]


def _bits(value):
    return struct.unpack('>Q', struct.pack('>d', value))[0]


def _summarize(rows: list[tuple[dict, dict]]) -> dict:
    """Internal freshly authenticated row/proposal arithmetic,not source authority."""
    # Check the entire group before calling any logarithm,including excluded rows.
    if any(row['response_events'][f] and not a
           for row, p in rows for f, a in zip(CLASSES, p['numerators'])):
        raise ValueError('Positive diagnostic response at structural probability zero')
    n = sum(row['parent_events'] for row, _ in rows)
    observed = [sum(row['response_events'][f] for row, _ in rows) for f in CLASSES]
    expected = [Fraction() for _ in CLASSES]; brier = Fraction()
    losses, means = [], []
    for row, p in rows:
        probabilities = [Fraction(a, p['denominator']) for a in p['numerators']]
        squared = sum(value*value for value in probabilities)
        for j, f in enumerate(CLASSES):
            count = row['response_events'][f]
            expected[j] += row['parent_events']*probabilities[j]
            brier += count*(1-2*probabilities[j]+squared)
            if count:
                value = struct.unpack('>d', struct.pack('>Q', p['probability_bits'][j]))[0]
                if not math.isfinite(value) or not 0 < value <= 1:
                    raise ValueError('Invalid positive primary diagnostic probability')
                loss = -math.log(value)
                losses.append(count*loss); means.append((count/n)*loss)
    loss_sum = math.fsum(losses); mean = math.fsum(means) if n else None
    if any(value is not None and (not math.isfinite(value) or value < 0)
           for value in (loss_sum, mean)):
        raise ValueError('Invalid finite nonnegative primary diagnostic NLL')
    return dict(parent_events=n, response_events=dict(zip(CLASSES, observed)),
        expected_events=list(map(_pair, expected)),
        frequency=[_pair(Fraction(c, n)) for c in observed] if n else [None]*4,
        frequency_residual=[_pair((c-e)/n) for c, e in zip(observed, expected)] if n else [None]*4,
        count_residual=[_pair(c-e) for c, e in zip(observed, expected)],
        brier_loss_sum=_pair(brier), mean_brier=_pair(brier/n) if n else None,
        nll_loss_sum=loss_sum, nll_loss_sum_bits=_bits(loss_sum),
        mean_nll=mean, mean_nll_bits=_bits(mean) if mean is not None else None,
        undefined_reason=None if n else 'no_observed_parent_events')


def diagnose(source_bytes: bytes, expected_source_sha256: str,
             count_pair: tuple[bytes, bytes], expected_count_sha256: str,
             parameter_pair: tuple[bytes, bytes], expected_parameter_sha256: str,
             native_bytes: bytes, expected_native_sha256: str) -> dict:
    """Reauthenticate complete bytes,then retain all72 contexts/three original splits."""
    source, proposal = _inputs(source_bytes, expected_source_sha256, count_pair,
        expected_count_sha256, parameter_pair, expected_parameter_sha256,
        native_bytes, expected_native_sha256)
    rows = [(row, proposal['contexts'][row['base_mask']*3+row['outs_before']])
            for row in source['contexts']]
    contexts = []
    for row, p in rows:
        result = _summarize([(row, p)])
        result.update(split=row['split'], base_mask=row['base_mask'], outs_before=row['outs_before'],
            training_parent_events=p['parent_events'], prior_only=p['prior_only'],
            supported=list(p['supported']), numerators=list(p['numerators']),
            denominator=p['denominator'], probability_bits=list(p['probability_bits']))
        contexts.append(result)
    splits = [dict(split=split, **_summarize([(r, p) for r, p in rows if r['split'] == split]))
              for split in SPLITS]
    return dict(scope='geometry_proxy_retrospective_diagnostics', model_id=proposal['model_id'],
        source_sha256=expected_source_sha256, counts_sha256=expected_count_sha256,
        parameters_sha256=expected_parameter_sha256, native_sha256=expected_native_sha256,
        proper_scores_computed=True, proposal=proposal, contexts=contexts, splits=splits,
        limits=dict(optimized_fitting=False, model_selection=False, sampling=False,
                    integration=False, causal_certification=False, predictive_validation=False,
                    calibration=False, uncertainty_coverage=False))
