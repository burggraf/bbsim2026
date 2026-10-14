"""Training-only count serialization prerequisite; not a playable model."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import tempfile
from audits import hit_advancement_comparison as comparison
from audits import hit_advancement as hit

FORMAT = 'joint-hit-training-counts-v1'


def encode(report, qualification_sha256):
    """Encode a validated aggregate; digest syntax is metadata, not authentication."""
    if (not isinstance(qualification_sha256, str) or len(qualification_sha256) != 64
            or any(c not in '0123456789abcdef' for c in qualification_sha256)):
        raise ValueError('Invalid qualification digest')
    fitted = comparison.fit(report)
    contexts = [(value, mask, outs) for value in (1, 2) for mask in range(8) for outs in range(3)]
    total = comparison.count_total(tuple(n for context in contexts for n in fitted[context]['training_counts']))
    lines = [FORMAT, 'proxy_endpoints_not_certified_causes\ttrue',
             'qualification_sha256\t' + qualification_sha256, 'training_events\t' + str(total),
             'value\tbase_mask\touts_before\tcounts']
    for value, mask, outs in contexts:
        counts = fitted[(value, mask, outs)]['training_counts']
        lines.append(f'{value}\t{mask}\t{outs}\t' + ','.join(map(str, counts)))
    return ('\n'.join(lines) + '\n').encode('ascii')


INPUT_SHA = comparison.INPUT_SHA
EXPORT_SHA = comparison.EXPORT_SHA


def _pins(paths):
    return tuple(hashlib.sha256(path.read_bytes()).hexdigest() for path in paths)


def publish(snapshot, supplement, lineups, manifest, census, accepted, output):
    """Authenticate unchanged full qualification before training-only publication.

    Complete qualification survives any later failure. Stage complete count bytes
    outside the repository, then hard-link exclusively; never publish partial counts.
    """
    output = Path(output)
    qualification = output.with_suffix('.qualification.json')
    if any(p.exists() or p.is_symlink() for p in (output, qualification)):
        raise ValueError('Refusing to overwrite count or qualification evidence')
    output = output.resolve()
    qualification = output.with_suffix('.qualification.json')
    if output.is_relative_to(Path(hit.__file__).resolve().parents[1]):
        raise ValueError('Audit output must be outside repository')
    inputs = list(map(Path, (snapshot, supplement, lineups, manifest, census, accepted)))
    pins = _pins(inputs)
    if pins != INPUT_SHA:
        raise ValueError('Unsupported hit training input digest')
    exports = [inputs[4].with_suffix('.' + name + '.csv') for name in ('lineup', 'events', 'raw', 'groups')]
    queries = [Path(hit.runner.__file__).with_name(name) for name in
               ('2019-lineup-readiness.sql', '2019-outcomes.sql', '2019-raw-mechanisms.sql', '2019-runner-transitions.sql')]
    recipes = [Path(__file__), Path(comparison.__file__), *[Path(module.__file__) for module in
               (hit, hit.runner, hit.lineup, hit.outcome, hit.raw, hit.runner.joint_contact)]]
    frozen = inputs + exports + queries + recipes
    identities = tuple(p.resolve(strict=True) for p in frozen)
    if output in identities or qualification in identities:
        raise ValueError('Audit output aliases input evidence')
    all_pins = _pins(frozen)
    if all_pins[:6] != pins:
        raise ValueError('Hit training inputs changed during read')
    if all_pins[6:10] != EXPORT_SHA:
        raise ValueError('Unsupported hit training native export digest')

    def unchanged(include_qualification=False):
        if output.resolve() != output or qualification.resolve() != qualification:
            raise ValueError('Hit training output path changed before publication')
        if tuple(p.resolve(strict=True) for p in frozen) != identities:
            raise ValueError('Hit training input path changed before publication')
        paths = frozen + ([qualification] if include_qualification else [])
        expected = all_pins + ((pins[5],) if include_qualification else ())
        if _pins(paths) != expected:
            raise ValueError('Hit training inputs changed before publication')

    # Do not replace full source/native/census reconciliation with aggregate fit.
    hit.publish(*inputs[:5], qualification)
    if qualification.read_bytes() != inputs[5].read_bytes():
        raise ValueError('Hit qualification report byte mismatch')
    unchanged(include_qualification=True)
    qualified = json.loads(qualification.read_bytes(), object_pairs_hook=hit.raw.unique_object)
    payload = encode(qualified, pins[5])
    unchanged(include_qualification=True)
    # The final name appears only after complete bytes have been flushed. No
    # overwrite operation or cleanup of preexisting evidence is permitted.
    stage = None
    try:
        with tempfile.NamedTemporaryFile(dir=output.parent, prefix='.hit-counts-', delete=False) as stream:
            stage = Path(stream.name)
            stream.write(payload)
            stream.flush()
            os.fsync(stream.fileno())
        unchanged(include_qualification=True)
        os.link(stage, output)
    finally:
        if stage is not None:
            stage.unlink()
    return payload


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('snapshot', 'supplement', 'lineups', 'manifest', 'census', 'accepted', 'output'):
        parser.add_argument(name, type=Path)
    args = parser.parse_args()
    payload = publish(args.snapshot, args.supplement, args.lineups, args.manifest, args.census, args.accepted, args.output)
    print(json.dumps(dict(format=FORMAT, training_events=int(payload.splitlines()[3].split(b'\t')[1])), sort_keys=True))


if __name__ == '__main__':
    main()
