#!/usr/bin/env python3
"""Check Orange Book opening examples; does not execute or verify Orange.

Run from any directory with Python 3. No third-party dependencies or network
access are required. Exhaustive results concern only the finite models below.
The manuscript also contains a separate mathematical proof for arbitrary
finite, equal-length bit strings.
"""
from itertools import product
from pathlib import Path
import re
import unittest

ROOT = Path(__file__).resolve().parents[1]
MANUSCRIPT = ROOT / 'docs' / 'book' / 'NOVICE_OPENING.md'
INDEX = ROOT / 'docs' / 'book' / 'README.md'


def xor_bits(left: str, right: str) -> str:
    """Apply the manuscript's XOR definition to equal-length bit strings."""
    if not isinstance(left, str) or not isinstance(right, str):
        raise TypeError('operands must be strings')
    if len(left) != len(right):
        raise ValueError('operands must have equal length')
    if any(ch not in '01' for ch in left + right):
        raise ValueError('operands may contain only 0 and 1')
    return ''.join('1' if a != b else '0' for a, b in zip(left, right))


def github_anchor(heading: str) -> str:
    """Slug the simple, unique prose headings used in this increment."""
    return re.sub(r'[^\w\- ]', '', heading.lower()).replace(' ', '-')


class FoundationExamples(unittest.TestCase):
    def test_reversal_examples(self):
        self.assertEqual('MEET AT THE BRIDGE'[::-1], 'EGDIRB EHT TA TEEM')
        self.assertEqual('AB C'[::-1], 'C BA')
        self.assertEqual('AB CD'[::-1], 'DC BA')
        self.assertEqual(' '.join(w[::-1] for w in 'AB CD'.split(' ')), 'BA DC')

    def test_named_reverse_readings_and_counting_proof_are_checkable(self):
        # §1.4 names three readings. One-letter words collapse two of them.
        collapsed = 'A B'
        self.assertEqual(
            collapsed[::-1], ' '.join(reversed(collapsed.split(' ')))
        )
        example = 'AB C'
        full = example[::-1]
        within = ' '.join(word[::-1] for word in example.split(' '))
        order = ' '.join(reversed(example.split(' ')))
        self.assertEqual((full, within, order), ('C BA', 'BA C', 'C AB'))
        self.assertEqual(len({full, within, order}), 3)
        text = MANUSCRIPT.read_text(encoding='utf-8')
        section = text.split('### 1.4 ', 1)[1].split('### 1.5 ', 1)[0]
        for result in (full, within, order):
            self.assertIn(f'`{result}`', section)
        counting = text.split('### 2.4 ', 1)[1].split('### 2.5 ', 1)[0]
        counting = re.sub(r'\s+', ' ', counting)
        self.assertIn('misses none and counts none twice', counting)
        self.assertIn('(1 × 16) + 3 = 19', text)
        self.assertEqual((1 * 16) + 3, 19)

    def test_reversal_finite_reference(self):
        # 1,093 strings, including empty; not every possible finite sequence.
        checked = 0
        for length in range(7):
            for symbols in product('AB ', repeat=length):
                text = ''.join(symbols)
                self.assertEqual(text[::-1][::-1], text)
                checked += 1
        self.assertEqual(checked, 1093)

    def test_empty_and_one_character_inputs(self):
        self.assertEqual(len({ '', ' ', '0' }), 3)
        for value in ('', ' ', '0'):
            self.assertEqual(value[::-1], value)

    def test_binary_and_hex_worked_values(self):
        for binary, value, hexdigits in (
            ('1101', 13, 'D'), ('1001', 9, '9'),
            ('10100110', 166, 'A6'), ('10110', 22, '16'),
            ('00010011', 19, '13'), ('00111100', 60, '3C'),
            ('11111111', 255, 'FF'),
        ):
            self.assertEqual(int(binary, 2), value)
            self.assertEqual(int(hexdigits, 16), value)
        for value in range(256):
            self.assertEqual(int(format(value, '08b'), 2), value)
            self.assertEqual(int(format(value, '02X'), 16), value)

    def test_counts_and_maxima(self):
        for width in range(9):
            strings = list(product('01', repeat=width))
            self.assertEqual(len(strings), 2 ** width)
        self.assertEqual(2 ** 5, 32)
        self.assertEqual(2 ** 5 - 1, 31)
        self.assertEqual(256 ** 2, 65536)
        self.assertEqual(65536 ** 2, 4294967296)

    def test_bit_truth_table(self):
        expected = ((0, 0, 0), (0, 1, 1), (0, 1, 1), (1, 1, 0))
        for (a, b), outputs in zip(product((0, 1), repeat=2), expected):
            self.assertEqual((a & b, a | b, a ^ b), outputs)

    def test_xor_examples_and_recovery(self):
        for value, mask, expected in (
            ('1011', '0110', '1101'),
            ('10100110', '00111100', '10011010'),
            ('1010', '1100', '0110'),
            ('', '', ''),
        ):
            self.assertEqual(xor_bits(value, mask), expected)
            self.assertEqual(xor_bits(expected, mask), value)
        self.assertEqual(xor_bits('1101', '0000'), '1101')

    def test_byte_cancellation_complete_domain(self):
        checked = 0
        for value in range(256):
            for mask in range(256):
                self.assertEqual((value ^ mask) ^ mask, value)
                checked += 1
        self.assertEqual(checked, 65536)

    def test_string_cancellation_finite_reference(self):
        for width in range(6):
            strings = [''.join(b) for b in product('01', repeat=width)]
            for value in strings:
                for mask in strings:
                    self.assertEqual(xor_bits(xor_bits(value, mask), mask), value)

    def test_invalid_operands(self):
        for left, right in (('101', '11'), ('2', '0'), ('a', '0')):
            with self.assertRaises(ValueError):
                xor_bits(left, right)
        with self.assertRaises(TypeError):
            xor_bits(1, '1')

    def test_xor_grouping_all_bit_triples(self):
        values = []
        for a, b, c in product((0, 1), repeat=3):
            self.assertEqual((a ^ b) ^ c, a ^ (b ^ c))
            values.append((a ^ b) ^ c)
        self.assertEqual(values, [0, 1, 1, 0, 1, 0, 0, 1])

    def test_other_operations_and_counterexamples(self):
        self.assertEqual(format(0b1010 & 0b1100, '04b'), '1000')
        self.assertEqual(format(0b1010 | 0b1100, '04b'), '1110')
        self.assertNotEqual((1 & 0) & 0, 1)
        self.assertEqual((1 | 0) & 0, 0)
        self.assertEqual(1 | (0 & 0), 1)


class ManuscriptIntegrity(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.manuscript = MANUSCRIPT.read_text(encoding='utf-8')
        cls.index = INDEX.read_text(encoding='utf-8')

    def test_every_exercise_has_exactly_one_answer(self):
        exercises = re.findall(r'^\*\*Exercise (\d+\.\d+) —', self.manuscript, re.M)
        answers = re.findall(r'^\*\*(\d+\.\d+)\.\*\*', self.manuscript, re.M)
        self.assertEqual(len(exercises), 25)
        self.assertEqual(len(set(exercises)), 25)
        self.assertEqual(sorted(exercises), sorted(answers))

    def test_exact_part_titles(self):
        found = re.findall(r'^## (Part .+)$', self.index, re.M)
        self.assertEqual(found, [
            'Part 1, The Novice', 'Part 2, The Journeyman', 'Part 3, The Master'
        ])

    def test_three_sourced_short_epigraphs(self):
        quotes = re.findall(r'^> “(.+)”$', self.manuscript, re.M)
        self.assertEqual([len(q.split()) for q in quotes], [7, 17, 19])
        for ref in ('S1', 'S2', 'S3'):
            self.assertIn(f'[{ref}]', self.manuscript)
            self.assertIn(f'**[{ref}]', self.manuscript)

    def test_hex_table_all_sixteen_rows(self):
        rows = re.findall(r'^\| (\d+) \| `([01]{4})` \| `([0-9A-F])` \|$',
                          self.manuscript, re.M)
        self.assertEqual(len(rows), 16)
        for decimal, binary, hexadecimal in rows:
            self.assertEqual(int(decimal), int(binary, 2))
            self.assertEqual(int(decimal), int(hexadecimal, 16))

    def test_new_local_heading_links(self):
        anchors = {github_anchor(h) for h in re.findall(r'^#{1,6} (.+)$',
                                                        self.manuscript, re.M)}
        for fragment in re.findall(r'NOVICE_OPENING\.md#([^\)]+)', self.index):
            self.assertIn(fragment, anchors)

    def test_original_chapters_each_mapped_once(self):
        # Expected anchors read from manuscript 0.26's existing contents.
        expected = {
            1: 'the-seams-are-the-system', 2: 'claims-not-labels',
            3: 'one-language-several-semantic-worlds',
            4: 'from-surface-text-to-meaning',
            5: 'proof-search-is-not-proof-checking',
            6: 'secrets-are-a-semantic-concern',
            7: 'no-disposable-prototype',
            8: 'orange-2026-the-smallest-honest-slice',
            9: 'from-core-to-native-bytes', 10: 'the-foreign-boundary',
            11: 'standards-as-versioned-inputs',
            12: 'the-corpus-as-acceptance-test',
            13: 'interoperability-and-external-validation',
            14: 'evidence-that-survives-the-build',
            15: 'offline-replay-and-trust-budgets',
            16: 'solo-work-through-incremental-gates',
            17: 'releases-updates-and-failure',
        }
        mapped = re.findall(r'\.\./THE_ORANGE_BOOK\.md#(chapter-[^)]+)', self.index)
        self.assertEqual(len(mapped), 17)
        self.assertEqual(set(mapped), {f'chapter-{n}-{slug}' for n, slug in expected.items()})


class ContinuationExamples(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.text = (ROOT / 'docs' / 'book' / 'NOVICE_PROGRAMMING.md').read_text(encoding='utf-8')

    def test_all_27_continuation_exercises_have_answers(self):
        exercises = re.findall(r'^\*\*Exercise (\d+\.\d+) —', self.text, re.M)
        answers = re.findall(r'^\*\*(\d+\.\d+)\.\*\*', self.text, re.M)
        self.assertEqual(len(exercises), 27)
        self.assertEqual(len(set(exercises)), 27)
        self.assertEqual(sorted(exercises), sorted(answers))

    def test_epigraph_lengths(self):
        quotes = re.findall(r'^> “(.+)”$', self.text, re.M)
        self.assertEqual([len(q.split()) for q in quotes], [7, 19, 10])

    def test_nine_complete_listings_seven_expected_outputs(self):
        sources = re.findall(r'^```orange\n(.*?)\n```', self.text, re.M | re.S)
        outputs = re.findall(r'^```text\n(.*?)\n```', self.text, re.M | re.S)
        self.assertEqual(len(sources), 9)
        names = []
        for source in sources:
            self.assertTrue(source.startswith('edition 2026;'))
            name = re.search(r'module (\w+) \{', source).group(1)
            names.append(name)
            expected = [out for out in outputs if out.startswith(name + '::')]
            self.assertEqual(len(expected), 0 if name in ('too_large', 'ungrouped') else 1)
        self.assertEqual(len(set(names)), 9)

    def test_boundary_amounts_are_explicitly_computed(self):
        source = re.search(r'module boundary_moves \{(.*?)\n\}', self.text, re.S).group(1)
        for operation in ('<<< (8)', '<<< (9)', '<< (8)', '>> (9)', '<<< (-1)'):
            self.assertIn(operation, source)
        self.assertNotRegex(source, r'[<>]{2,3} [89]')

    def test_wrapping_examples(self):
        self.assertEqual((255 + 1) % 256, 0)
        self.assertEqual((0 - 1) % 256, 255)
        self.assertEqual((200 * 2) % 256, 0x90)
        self.assertEqual((250 + 10) % 256, 4)
        self.assertEqual((4 - 10) % 256, 250)
        self.assertEqual((128 * 2) % 256, 0)

    def test_euclidean_division_identity_and_unique_representative(self):
        for modulus in range(1, 33):
            for value in range(-512, 513):
                quotient, remainder = divmod(value, modulus)
                self.assertEqual(value, quotient * modulus + remainder)
                self.assertTrue(0 <= remainder < modulus)
                self.assertNotEqual(remainder + modulus, remainder)
                self.assertNotEqual(remainder - modulus, remainder)

    def test_rotation_inverse_and_ones_for_all_byte_values(self):
        for value in range(256):
            for amount in range(-17, 18):
                result = rotate_byte(value, amount)
                self.assertEqual(rotate_byte(result, -amount), value)
                self.assertEqual(result.bit_count(), value.bit_count())

    def test_shift_and_rotation_boundaries(self):
        self.assertEqual((0x81 << 1) & 255, 2)
        self.assertEqual(0x81 >> 1, 0x40)
        self.assertEqual(rotate_byte(0x81, 1), 3)
        self.assertEqual(rotate_byte(0x81, -1), 0xc0)
        self.assertEqual(rotate_byte(0x81, 8), 0x81)
        self.assertEqual(rotate_byte(0x81, 9), 3)
        self.assertEqual(rotate_byte(0xa5, 9), 0x4b)
        self.assertEqual((0xa5 << 8) & 255, 0)

    def test_small_round_inverse_complete_byte_domain(self):
        for value in range(256):
            encoded = rotate_byte(((value + 7) & 255) ^ 0x3c, 1)
            decoded = ((rotate_byte(encoded, -1) ^ 0x3c) - 7) & 255
            self.assertEqual(decoded, value)
        self.assertEqual(rotate_byte(((0xfa + 7) & 255) ^ 0x3c, 1), 0x7a)
        self.assertEqual(rotate_byte((7 ^ 0x3c), 1), 0x76)
        self.assertEqual(rotate_byte(((0x7a - 7) & 255) ^ 0x3c, -1), 0xa7)

    def test_grouping_is_not_interchangeable(self):
        self.assertEqual((1 + 1) ^ 1, 3)
        self.assertEqual(1 + (1 ^ 1), 1)

    def test_n7_exercises_answers_and_label(self):
        text = (ROOT / 'docs' / 'book' / 'NOVICE_N7_NAME_THE_INTERMEDIATE_STEP.md').read_text(
            encoding='utf-8')
        exercises = re.findall(r'^\*\*Exercise (N7\.\d+) —', text, re.M)
        answers = re.findall(r'^\*\*(N7\.\d+)\.\*\*', text, re.M)
        self.assertEqual(exercises, [f'N7.{n}' for n in range(1, 8)])
        self.assertEqual(sorted(exercises), sorted(answers))
        self.assertNotIn('Chapter 7', text)
        index = INDEX.read_text(encoding='utf-8')
        self.assertIn('**N7.**', index)
        self.assertIn(
            'NOVICE_N7_NAME_THE_INTERMEDIATE_STEP.md#n7-name-the-intermediate-step',
            index)
        self.assertNotIn('Chapter 7', index)
        headings = re.findall(r'^#{1,6} (.+)$', text, re.M)
        anchors = {github_anchor(h) for h in headings}
        self.assertIn('n7-name-the-intermediate-step', anchors)

    def test_n7_quarter_round_and_index_bounds(self):
        def rotl(value, amount):
            value &= 0xFFFFFFFF
            return ((value << amount) | (value >> (32 - amount))) & 0xFFFFFFFF

        a, b, c, d = 0x11111111, 0x01020304, 0x9b8d6f43, 0x01234567
        a1 = (a + b) & 0xFFFFFFFF
        d1 = rotl(d ^ a1, 16)
        c1 = (c + d1) & 0xFFFFFFFF
        b1 = rotl(b ^ c1, 12)
        a2 = (a1 + b1) & 0xFFFFFFFF
        d2 = rotl(d1 ^ a2, 8)
        c2 = (c1 + d2) & 0xFFFFFFFF
        b2 = rotl(b1 ^ c2, 7)
        self.assertEqual((a2, b2, c2, d2),
                         (0xea2a92f4, 0xcb1cf8ce, 0x4581472e, 0x5881c4bb))
        self.assertEqual(a1, 0x12131415)
        self.assertLess(a + b, 2 ** 32)
        words = [a, b, c, d]
        self.assertEqual(words[0], a)
        self.assertEqual(words[3], d)
        self.assertEqual([k for k in range(4) if 0 <= k < 4], [0, 1, 2, 3])
        self.assertFalse(4 < 4)
        self.assertEqual((0xff + 0x01) & 0xff, 0)
        self.assertEqual(0xff + 0x01, 0x100)

    def test_opening_retained_byte_for_byte(self):
        import hashlib
        data = MANUSCRIPT.read_bytes()
        blob = b'blob ' + str(len(data)).encode('ascii') + b'\0' + data
        self.assertEqual(hashlib.sha1(blob).hexdigest(),
                         '62f7463f9008d3b56f935c84195adb1289b59d1b')


def rotate_byte(value: int, amount: int) -> int:
    """Reference mathematical rotation, not an Orange interpreter."""
    if not 0 <= value < 256:
        raise ValueError('value must fit a byte')
    amount %= 8
    return ((value << amount) | (value >> (8 - amount))) & 255


if __name__ == '__main__':
    unittest.main(verbosity=2)
