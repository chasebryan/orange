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


if __name__ == '__main__':
    unittest.main(verbosity=2)
