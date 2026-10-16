"""Toy whole-source authentication and private witness review;no actual rates.

Synthetic witness fixtures are complete25-record data,not native-execution proof.
The isolated parsed-count core properties do not certify whole-source authority.
"""
import copy
from fractions import Fraction
import hashlib
import itertools
import json
import struct
import unittest
from audits import contact_proxy_training_counts as codec
from audits import contact_proxy_probability_review as subject
from tests.test_contact_proxy_training_counts import source, payload

CLASSES = ('SF', 'SH', 'GIDP', 'remaining')
MODEL = 'geometry-proxy-context-jeffreys-v1'
LIMITS = dict(proper_scores=False, optimized_fitting=False, model_selection=False,
              sampling=False, integration=False, causal_certification=False,
              predictive_validation=False)


def digest(raw):
    return hashlib.sha256(raw).hexdigest()


def bits(value):
    return struct.unpack('>Q', struct.pack('>d', value))[0]


def support(mask, outs):
    return [mask in (4, 5, 6, 7) and outs in (0, 1),
            mask in (1, 5) and outs in (0, 1),
            mask in (1, 5) and outs in (0, 1), True]


def witness(table):
    """Independent literal-support Fraction toy witness in the real complete shape."""
    lines = ['\t'.join(['H', '0', table['source_sha256'], MODEL,
                        str(table['training_events'])])]
    for row in table['contexts']:
        mask, outs, n = (row[k] for k in ('base_mask', 'outs_before', 'parent_events'))
        counts = [row['response_events'][f] for f in CLASSES]
        legal = support(mask, outs); denominator = 2*n+sum(legal)
        nums = [2*c+1 if ok else 0 for c, ok in zip(counts, legal)]
        p = [float(Fraction(a, denominator)) for a in nums]
        running = 0.0; cdf = []
        for value in p:
            running += value; cdf.append(running)
        cdf[-1] = 1.0
        fields = ['C', str(mask), str(outs), str(n), json.dumps(counts),
                  json.dumps(legal), str(n == 0).lower(), json.dumps(nums),
                  str(denominator), json.dumps(list(map(bits, p))),
                  json.dumps(list(map(bits, cdf)))]
        lines.append('\t'.join(fields))
    return ('\n'.join(lines)+'\n').encode('ascii')


def inputs():
    raw, _ = source(); source_hash = digest(raw)
    counts = codec.encode(raw, source_hash); table = codec.parse(counts, source_hash)
    return raw, source_hash, (counts, counts), digest(counts), witness(table), table


def changed_field(raw, record, column, replacement):
    lines = raw.decode('ascii').splitlines(); fields = lines[record].split('\t')
    fields[column] = replacement; lines[record] = '\t'.join(fields)
    return ('\n'.join(lines)+'\n').encode('ascii')


class ProbabilityReview(unittest.TestCase):
    def test_public_whole_source_review_retains_every_context_and_exact_pairs(self):
        raw, sh, pair, ch, native, table = inputs()
        self.assertIsNone(subject.authenticate(raw, sh, pair, ch))
        result = subject.review(raw, sh, pair, ch, native)
        self.assertEqual(len(result.get('contexts', [])), 24)
        self.assertEqual(set(result), {'scope', 'model_id', 'source_sha256', 'counts_sha256',
                                      'native_sha256', 'training_events', 'contexts', 'limits'})
        self.assertEqual(result['scope'], 'geometry_proxy_fixed_proposal_review')
        self.assertEqual((result['model_id'], result['source_sha256'], result['counts_sha256'],
                          result['native_sha256'], result['training_events']),
                         (MODEL, sh, ch, digest(native), 4))
        self.assertEqual(result['limits'], LIMITS)
        self.assert_rows(result['contexts'], table)

    def assert_rows(self, rows, table):
        self.assertEqual(len(rows), 24)
        for i, (row, count) in enumerate(zip(rows, table['contexts'])):
            mask, outs = divmod(i, 3); legal = support(mask, outs)
            n = count['parent_events']; d = 2*n+sum(legal)
            values = [count['response_events'][f] for f in CLASSES]
            nums = [2*c+1 if ok else 0 for c, ok in zip(values, legal)]
            self.assertEqual((row['base_mask'], row['outs_before'], row['parent_events'],
                              row['response_events'], row['supported'], row['prior_only'],
                              row['numerators'], row['denominator']),
                             (mask, outs, n, count['response_events'], legal, n == 0, nums, d))
            p = [float(Fraction(a, d)) for a in nums]; total = 0.0; expected_cdf = []
            for value in p:
                total += value; expected_cdf.append(total)
            expected_cdf[-1] = 1.0
            self.assertEqual(row['probability_bits'], list(map(bits, p)))
            self.assertEqual(row['cdf_bits'], list(map(bits, expected_cdf)))
            exact_cdf = Fraction()
            for j in range(4):
                exact = Fraction(nums[j], d); exact_cdf += exact
                pe = abs(Fraction.from_float(p[j])-exact)
                ce = abs(Fraction.from_float(expected_cdf[j])-exact_cdf)
                self.assertEqual(row['probability_error'][j], [pe.numerator, pe.denominator])
                self.assertEqual(row['cdf_error'][j], [ce.numerator, ce.denominator])
                self.assertLessEqual(pe, Fraction(32, 2**52))
                self.assertLessEqual(ce, Fraction(32, 2**52))

    def test_authentication_rejects_wrong_bindings_or_nonidentical_immutable_pair(self):
        raw, sh, pair, ch, native, _ = inputs()
        variants = [(raw, '0'*64, pair, ch), (raw, sh, pair, '0'*64),
                    (raw, sh, (pair[0],), ch), (raw, sh, list(pair), ch),
                    (raw, sh, (bytearray(pair[0]), pair[1]), ch),
                    (raw, sh, (pair[0], pair[1]+b'\n'), ch),
                    (bytearray(raw), sh, pair, ch)]
        for expected in (None, '', 'A'*64, 'z'*64, '0'*63, 1):
            variants.extend([(raw, expected, pair, ch), (raw, sh, pair, expected)])
        for args in variants:
            with self.subTest(source_type=type(args[0]).__name__, bindings=(args[1], args[3])):
                with self.assertRaises(ValueError): subject.authenticate(*args)
                with self.assertRaises(ValueError): subject.review(*args, native)

    def test_nontraining_and_diagnostic_corruption_cannot_hide_behind_matching_metadata(self):
        raw, sh, pair, _, _, _ = inputs(); original = json.loads(raw)
        variants = []
        for split in ('heldout', 'excluded'):
            changed = copy.deepcopy(original)
            row = next(r for r in changed['contexts'] if r['split'] == split)
            row['parent_events'] += 1; variants.append(changed)
        changed = copy.deepcopy(original); changed['totals']['parent_events'] += 1
        variants.append(changed)
        variants.extend([dict(original, extra=0), [], None])
        bad_bytes = [json.dumps(v, sort_keys=True, allow_nan=False).encode('ascii') for v in variants]
        bad_bytes += [b'{"format":0,'+raw[1:], b'{"x":NaN}', b'{"x":Infinity}', b'\xff']
        for bad in bad_bytes:
            bh = digest(bad); count = pair[0].replace(sh.encode(), bh.encode())
            bp = (count, count); bc = digest(count)
            native = witness(codec.parse(count, bh))
            with self.subTest(size=len(bad)):
                with self.assertRaises(ValueError): subject.authenticate(bad, bh, bp, bc)
                with self.assertRaises(ValueError): subject.review(bad, bh, bp, bc, native)

    def test_count_schema_refusals_survive_the_public_review_boundary(self):
        raw, sh, pair, _, native, _ = inputs(); good = pair[0]
        variants = [good[:-1], b'\xef\xbb\xbf'+good, good.replace(b'\n', b'\r\n'),
                    good.replace(b'2019', b'2020'), good.replace(b'training', b'heldout'),
                    good.replace(b'prior\t1\t2', b'prior\t1\t3'),
                    good.replace(b'contexts\t24', b'contexts\t23'),
                    good.replace(b'classes\tSF\tSH', b'classes\tSH\tSF'), good+b'#comment\n']
        lines = good.splitlines()
        variants += [b'\n'.join(lines[:-1])+b'\n', good+lines[-1]+b'\n']
        for bad in variants:
            with self.subTest(size=len(bad)):
                with self.assertRaises(ValueError): subject.review(raw, sh, (bad, bad), digest(bad), native)

    def test_every_native_metadata_rational_boolean_and_bit_field_is_checked(self):
        raw, sh, pair, ch, native, _ = inputs()
        variants = [(0, 0, 'X'), (0, 1, '1'), (0, 2, '0'*64),
                    (0, 3, 'wrong-model'), (0, 4, '0')]
        for record in range(1, 25):
            for column in range(11):
                for token in ('', 'null', 'True', '1.0', '-1', '+1', '01', 'NaN', '[]'):
                    variants.append((record, column, token))
        for record, column, token in variants:
            with self.subTest(record=record, column=column, token=token):
                bad = changed_field(native, record, column, token)
                with self.assertRaises(ValueError): subject.review(raw, sh, pair, ch, bad)

    def test_native_byte_shape_order_high_bits_and_floating_aliases_refuse(self):
        raw, sh, pair, ch, native, _ = inputs(); lines = native.splitlines()
        swapped = lines.copy(); swapped[1], swapped[2] = swapped[2], swapped[1]
        variants = [b'', native[:-1], native+b'\n', b'\xef\xbb\xbf'+native,
                    native.replace(b'\n', b'\r\n'), b'x'*16385,
                    b'\n'.join(lines[:-1])+b'\n', native+lines[-1]+b'\n',
                    b'\n'.join(swapped)+b'\n']
        for column in (9, 10):
            for values in ([True, 0, 0, 0], [0.0]*4, [None]*4, [-1]*4,
                           [2**64]*4, [bits(float('inf'))]*4,
                           [bits(float('nan'))]*4, [bits(-0.0)]*4):
                variants.append(changed_field(native, 1, column, json.dumps(values)))
        # A one-ULP supported alteration remains within the loose error bound,but
        # violates the fixed division/left-fold bit contract and must still refuse.
        fields = lines[1].decode().split('\t'); values = json.loads(fields[9])
        values[-1] -= 1; variants.append(changed_field(native, 1, 9, json.dumps(values)))
        for bad in variants:
            with self.subTest(size=len(bad)):
                with self.assertRaises(ValueError): subject.review(raw, sh, pair, ch, bad)
        for index in range(len(native)):
            bad = bytearray(native); bad[index] ^= 128
            with self.subTest(high_bit=index):
                with self.assertRaises(ValueError): subject.review(raw, sh, pair, ch, bytes(bad))

    def test_native_canonical_arrays_do_not_accept_boolean_or_spacing_aliases(self):
        raw, sh, pair, ch, native, _ = inputs(); lines = native.decode().splitlines()
        variants = [None, '', bytearray(native)]
        for record in range(1, 25):
            fields = lines[record].split('\t')
            for column in (4, 7, 9, 10):
                values = json.loads(fields[column])
                variants.append(changed_field(native, record, column,
                                              json.dumps([bool(x) for x in values])))
                variants.append(changed_field(native, record, column,
                                              json.dumps(values, separators=(',', ':'))))
            variants.append(changed_field(native, record, 5,
                                          json.dumps([int(x) for x in json.loads(fields[5])])) )
        for bad in variants:
            with self.subTest(kind=type(bad).__name__):
                with self.assertRaises(ValueError): subject.review(raw, sh, pair, ch, bad)

    def test_every_small_supported_partition_in_isolated_strict_parsed_core(self):
        cases = 0
        for mask in range(8):
            for outs in range(3):
                selected = [i for i, ok in enumerate(support(mask, outs)) if ok]
                for n in range(9):
                    for prefix in itertools.product(range(n+1), repeat=len(selected)-1):
                        if sum(prefix) > n: continue
                        values = [0]*4
                        for i, value in zip(selected, (*prefix, n-sum(prefix))): values[i] = value
                        rows = [[m, o, 0, 0, 0, 0, 0] for m in range(8) for o in range(3)]
                        rows[mask*3+outs] = [mask, outs, n, *values]
                        table = codec.parse(payload(rows), 'a'*64)
                        with self.subTest(mask=mask, outs=outs, n=n, values=values):
                            result = subject._review_native(table, witness(table))
                            self.assertEqual(len(result), 24)
                            self.assert_rows(result, table)
                        cases += 1
        self.assertEqual(cases, 1716)

    def test_cap_denominator_empty_rows_and_supported_rotations_in_parsed_core(self):
        for mask, outs in ((0, 0), (1, 1), (4, 0), (5, 0)):
            selected = [i for i, ok in enumerate(support(mask, outs)) if ok]
            for n in (0, 1, 2, 8, 32768, 65536, 1073741824, 2147483647):
                for chosen in selected:
                    values = [0]*4; values[chosen] = n
                    rows = [[m, o, 0, 0, 0, 0, 0] for m in range(8) for o in range(3)]
                    rows[mask*3+outs] = [mask, outs, n, *values]
                    table = codec.parse(payload(rows), 'a'*64)
                    with self.subTest(mask=mask, outs=outs, n=n, chosen=chosen):
                        result = subject._review_native(table, witness(table))
                        self.assertEqual(len(result), 24); self.assert_rows(result, table)
                        self.assertEqual(result[mask*3+outs]['denominator'], 2*n+len(selected))

    def test_same_whole_bytes_repeat_exact_private_json_and_do_not_mutate_inputs(self):
        args = inputs(); before = args[:5]
        first = subject.review(*args[:5]); second = subject.review(*args[:5])
        self.assertEqual(len(first.get('contexts', [])), 24)
        self.assertEqual(json.dumps(first, sort_keys=True, allow_nan=False),
                         json.dumps(second, sort_keys=True, allow_nan=False))
        self.assertEqual(args[:5], before)


if __name__ == '__main__':
    unittest.main()
