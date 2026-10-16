"""Count-only bytes;toy digests are metadata,not source authentication."""
import copy
import hashlib
import json
import unittest
from audits import contact_proxy_training_counts as codec
from audits import contact_proxy_response_counts as census
from tests.test_contact_proxy_response_counts import inputs
from tests.test_contact_consequence_support import fixture

DIGEST = 'a' * 64
HEADER = ['contact-proxy-context-counts-v1', 'model\tgeometry-proxy-context-jeffreys-v1',
          'season\t2019', 'split\ttraining', 'source_sha256\t'+DIGEST,
          'prior\t1\t2', 'classes\tSF\tSH\tGIDP\tremaining', 'contexts\t24']


def digest(value):
    return hashlib.sha256(value).hexdigest()


def payload(rows=None, source=DIGEST):
    if rows is None:
        rows = []
        for mask in range(8):
            for outs in range(3):
                values = [int(mask >= 4 and outs < 2), int(mask in (1, 5) and outs < 2)*2,
                          int(mask in (1, 5) and outs < 2)*3, mask*3+outs+4]
                rows.append([mask, outs, sum(values), *values])
    header = HEADER.copy(); header[4] = 'source_sha256\t'+source
    return ('\n'.join(header+['C\t'+'\t'.join(map(str, row)) for row in rows])+'\n').encode('ascii')


def source(pairs=None, supported=None):
    if pairs is None:
        pairs = [fixture(f, key=i+1) for i, f in enumerate(('SF', 'SH', 'GIDP'))]
        pairs.append(fixture('SH', mask=0, key=4))
    args = inputs(pairs, supported)
    report = census.inventory(*args[:3], **args[3])
    raw = json.dumps(report, sort_keys=True, indent=2, allow_nan=False).encode('ascii')
    return raw, report


class TrainingCounts(unittest.TestCase):
    def test_exact_training_bytes_from_complete_source(self):
        raw, report = source()
        rows = [[r['base_mask'], r['outs_before'], r['parent_events'],
                 *(r['response_events'][f] for f in ('SF', 'SH', 'GIDP', 'remaining'))]
                for r in report['contexts'] if r['split'] == 'training']
        self.assertEqual(codec.encode(raw, digest(raw)), payload(rows, digest(raw)))
        self.assertEqual(sum(r[2] for r in rows), 4)

    def test_parse_all24_contexts_and_integer_total(self):
        value = codec.parse(payload(), DIGEST)
        self.assertEqual(value['source_sha256'], DIGEST)
        self.assertEqual(len(value['contexts']), 24)
        self.assertEqual([(r['base_mask'], r['outs_before']) for r in value['contexts']],
                         [(m, o) for m in range(8) for o in range(3)])
        self.assertEqual(value['training_events'], sum(r['parent_events'] for r in value['contexts']))
        for row in value['contexts']:
            self.assertEqual(tuple(row['response_events']), ('SF', 'SH', 'GIDP', 'remaining'))
            self.assertTrue(all(type(n) is int for n in row['response_events'].values()))
            self.assertEqual(row['response_events']['remaining'], row['base_mask']*3+row['outs_before']+4)

    def test_encode_parse_round_trip_preserves_input_bytes_and_whole_report(self):
        raw, report = source(); before = copy.deepcopy(report)
        value = codec.parse(codec.encode(raw, digest(raw)), digest(raw))
        self.assertEqual(value['contexts'], [r for r in report['contexts'] if r['split'] == 'training'])
        self.assertEqual(value['training_events'], 4)
        self.assertEqual(report, before)

    def test_expected_source_digest_is_required_before_json_selection(self):
        for bad in (None, '', 'A'*64, '0'*63, 1, 'z'*64):
            with self.subTest(expected=bad):
                with self.assertRaises(ValueError): codec.encode(b'not JSON', bad)
                with self.assertRaises(ValueError): codec.parse(payload(), bad)
        with self.assertRaisesRegex(ValueError, 'digest'):
            codec.encode(b'not JSON', DIGEST)
        with self.assertRaises(ValueError): codec.parse(payload(), 'b'*64)

    def test_whole_typed_response_report_must_reproject_exactly(self):
        _, report = source()
        variants = [[], None, {}, dict(report, format='other'), dict(report, extra=0)]
        for alias in (False, 0.0, True, 1.0):
            changed = copy.deepcopy(report); changed['contexts'][0]['parent_events'] = alias
            variants.append(changed)
        changed = copy.deepcopy(report); changed['contexts'].pop(); variants.append(changed)
        changed = copy.deepcopy(report); changed['totals']['parent_events'] += 1; variants.append(changed)
        for bad in variants:
            raw = json.dumps(bad).encode('ascii')
            with self.subTest(kind=type(bad).__name__):
                with self.assertRaises(ValueError): codec.encode(raw, digest(raw))

    def test_duplicate_keys_and_nonfinite_json_are_not_source_evidence(self):
        raw, _ = source()
        for bad in (b'{"format":0,'+raw[1:], b'{"x":NaN}', b'{"x":Infinity}', b'\xff'):
            with self.subTest(raw=bad[:32]):
                with self.assertRaises(ValueError): codec.encode(bad, digest(bad))

    def test_heldout_and_excluded_counts_never_enter_training_artifact(self):
        pairs = [fixture('SH', key=i+1) for i in range(3)]
        games = ('ANA201904040', 'ANA201904050', 'BOS201908070')
        for pair, game in zip(pairs, games):
            pair[0]['game_id'] = pair[1]['game_id'] = game
        raw, report = source(pairs, supported=games[:2])
        parsed = codec.parse(codec.encode(raw, digest(raw)), digest(raw))
        self.assertEqual(parsed['training_events'], 1)
        self.assertEqual(sum(r['response_events']['SH'] for r in parsed['contexts']), 1)
        self.assertEqual(report['totals']['parent_events'], 3)

    def test_missing_empty_context_is_not_an_observed_zero(self):
        lines = payload().splitlines()
        for bad in (b'\n'.join(lines[:-1])+b'\n', payload()+b'\n', payload()+lines[-1]+b'\n'):
            with self.subTest(size=len(bad)):
                with self.assertRaises(ValueError): codec.parse(bad, DIGEST)

    def test_noncanonical_encoding_and_headers_refuse(self):
        good = payload()
        bad = [good[:-1], b'\xef\xbb\xbf'+good, good.replace(b'\n', b'\r\n'), b'\xff'+good,
               good.replace(b'2019', b'2020'), good.replace(b'training', b'heldout'),
               good.replace(b'prior\t1\t2', b'prior\t1\t3'), good.replace(b'contexts\t24', b'contexts\t23'),
               good.replace(b'classes\tSF\tSH', b'classes\tSH\tSF'), good+b'#comment\n', b' '*16385]
        for data in bad:
            with self.subTest(data=data[:40]):
                with self.assertRaises(ValueError): codec.parse(data, DIGEST)

    def test_each_context_token_mutation_refuses(self):
        lines = payload().decode('ascii').splitlines()
        for i in range(8, 32):
            for column in range(1, 8):
                for token in ('-1', '+1', '01', '1.0', 'True', 'NaN', '', '2147483648', '9'*100):
                    mutated = lines.copy(); fields = mutated[i].split('\t'); fields[column] = token
                    mutated[i] = '\t'.join(fields)
                    with self.subTest(row=i-8, column=column, token=token):
                        with self.assertRaises(ValueError):
                            codec.parse(('\n'.join(mutated)+'\n').encode('ascii'), DIGEST)

    def test_order_shape_conservation_and_structural_positive_refuse(self):
        lines = payload().decode('ascii').splitlines(); variants = []
        changed = lines.copy(); changed[8], changed[9] = changed[9], changed[8]; variants.append(changed)
        changed = lines.copy(); changed[8] += '\t0'; variants.append(changed)
        changed = lines.copy(); changed[8] = changed[8].replace('C\t', 'D\t'); variants.append(changed)
        for row in (['C', '0', '0', '5', '1', '0', '0', '4'],
                    ['C', '0', '0', '5', '0', '0', '0', '4']):
            changed = lines.copy(); changed[8] = '\t'.join(row); variants.append(changed)
        for bad in variants:
            with self.subTest(row=bad[8]):
                with self.assertRaises(ValueError): codec.parse(('\n'.join(bad)+'\n').encode('ascii'), DIGEST)

    def test_every_one_byte_high_bit_mutation_refuses(self):
        good = payload()
        for index in range(len(good)):
            bad = bytearray(good); bad[index] ^= 128
            with self.subTest(byte=index):
                with self.assertRaises(ValueError): codec.parse(bytes(bad), DIGEST)

    def test_integer_cap_inclusive_and_aggregate_overflow_refuses(self):
        rows = [[m, o, 0, 0, 0, 0, 0] for m in range(8) for o in range(3)]
        self.assertEqual(codec.parse(payload(rows), DIGEST)['training_events'], 0)
        rows[0][2] = rows[0][-1] = 2147483647
        self.assertEqual(codec.parse(payload(rows), DIGEST)['training_events'], 2147483647)
        rows[1][2] = rows[1][-1] = 1
        with self.assertRaises(ValueError): codec.parse(payload(rows), DIGEST)


if __name__ == '__main__':
    unittest.main()
