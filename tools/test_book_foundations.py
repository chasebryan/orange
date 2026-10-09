#!/usr/bin/env python3
"""Check Orange Book opening examples; does not execute or verify Orange.

Run from any directory with Python 3. No third-party dependencies or network
access are required. Exhaustive results concern only the finite models below.
The manuscript also contains a separate mathematical proof for arbitrary
finite, equal-length bit strings.
"""
from fractions import Fraction
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
        self.assertEqual(exercises, [f'N7.{n}' for n in range(1, 13)])
        self.assertEqual(sorted(exercises), sorted(answers))
        for number in range(7, 18):
            self.assertNotIn(f'Chapter {number}', text)
        self.assertNotIn('not an exponent', text)
        self.assertIn('not the XOR operator from Chapter 5', text)
        self.assertIn('111 + 19 = 130', text)
        self.assertIn('5461067566 = 0x14581472e', text)
        self.assertIn('3928658676 = 0xea2a92f4', text)
        self.assertIn(
            'an `Int` index may use only integer literals, loop indices, '
            'and words converted with `as Int`',
            text)
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
        self.assertEqual(c1, 0xecff8273)
        self.assertEqual(b1, 0xd8177edf)
        self.assertEqual(0x6f + 0x13, 0x82)
        self.assertEqual(a1, 303240213)
        self.assertEqual(b1, 3625418463)
        self.assertEqual(a1 + b1, 3928658676)
        self.assertEqual(a2, 0xea2a92f4)
        self.assertEqual(d2, 0x5881c4bb)
        self.assertEqual(c1 + d2, 0x14581472e)
        self.assertEqual(3976168051 + 1484899515, c1 + d2)
        self.assertEqual(c2, (c1 + d2) % (2 ** 32))
        self.assertLess(a1 + b1, 2 ** 32)
        self.assertEqual(a1, 0x12131415)
        self.assertLess(a + b, 2 ** 32)
        words = [a, b, c, d]
        self.assertEqual(words[0], a)
        self.assertEqual(words[3], d)
        self.assertEqual([k for k in range(4) if 0 <= k < 4], [0, 1, 2, 3])
        self.assertFalse(4 < 4)
        self.assertEqual((0xff + 0x01) & 0xff, 0)
        self.assertEqual(0xff + 0x01, 0x100)
        self.assertTrue(0x81 > 0x7f)
        self.assertFalse(0x7f > 0x7f)
        for value in range(256):
            self.assertEqual(value > 0x7f, (value & 0x80) != 0)
        self.assertEqual(abs(-12), 12)
        self.assertFalse(True and False)
        words = [0x11111111, 0x01020304, 0x9b8d6f43, 0x01234567]
        self.assertEqual(words, [286331153, 16909060, 2609737539, 19088743])
        self.assertEqual(sum(words), 2932066495)
        self.assertLess(sum(words), 2 ** 32)
        self.assertEqual((0xffffffff + 1) % (2 ** 32), 0)
        self.assertNotEqual(0xffffffff + 1, 0)
        self.assertEqual([3 - i for i in range(4)], [3, 2, 1, 0])
        self.assertEqual([i + 1 for i in range(4)], [1, 2, 3, 4])
        self.assertTrue(all(0 <= 3 - i < 4 for i in range(4)))
        self.assertFalse(all(0 <= i + 1 < 4 for i in range(4)))

    def test_n8_exercises_answers_and_label(self):
        text = (ROOT / 'docs' / 'book' / 'NOVICE_N8_READ_AND_REPAIR.md').read_text(
            encoding='utf-8')
        exercises = re.findall(r'^\*\*Exercise (N8\.\d+) —', text, re.M)
        answers = re.findall(r'^\*\*(N8\.\d+)\.\*\*', text, re.M)
        self.assertEqual(exercises, [f'N8.{n}' for n in range(1, 11)])
        self.assertEqual(sorted(exercises), sorted(answers))
        self.assertNotIn('Chapter 8', text)
        self.assertIn('verified', text)
        index = INDEX.read_text(encoding='utf-8')
        self.assertIn('**N8.**', index)
        self.assertIn(
            'NOVICE_N8_READ_AND_REPAIR.md#n8-read-and-repair-a-program',
            index)
        self.assertNotIn('Chapter 8', index)
        headings = re.findall(r'^#{1,6} (.+)$', text, re.M)
        anchors = {github_anchor(h) for h in headings}
        self.assertIn('n8-read-and-repair-a-program', anchors)
        sources = re.findall(r'^```orange\n(.*?)\n```', text, re.M | re.S)
        self.assertEqual(len(sources), 18)
        self.assertEqual(len({re.search(r'module (\w+)', s).group(1) for s in sources}), 18)

    def test_n8_repair_arithmetic(self):
        def rotl(value, amount):
            value &= 0xFFFFFFFF
            return ((value << amount) | (value >> (32 - amount))) & 0xFFFFFFFF

        a, b, c, d = 0x11111111, 0x01020304, 0x9b8d6f43, 0x01234567
        a1 = (a + b) & 0xFFFFFFFF
        mixed = a1 ^ a
        self.assertEqual(a1, 0x12131415)
        self.assertEqual(mixed, 0x03020504)
        self.assertEqual((a + (b ^ a)) & 0xFFFFFFFF, 0x21242326)
        self.assertNotEqual(mixed, (a + (b ^ a)) & 0xFFFFFFFF)
        xor_d = d ^ a1
        self.assertEqual(xor_d, 0x13305172)
        self.assertEqual(rotl(xor_d, 8), 0x30517213)
        self.assertEqual(rotl(xor_d, 16), 0x51721330)
        d1 = rotl(xor_d, 8)
        c1 = (c + d1) & 0xFFFFFFFF
        b1 = rotl(b ^ c1, 12)
        a2 = (a1 + b1) & 0xFFFFFFFF
        d2 = rotl(d1 ^ a2, 8)
        c2 = (c1 + d2) & 0xFFFFFFFF
        b2 = rotl(b1 ^ c2, 7)
        wrong = (a2, b2, c2, d2)
        rfc = (0xea2a92f4, 0xcb1cf8ce, 0x4581472e, 0x5881c4bb)
        self.assertEqual(wrong, (0xe03840c2, 0x9a4fc5fd, 0x3511b326, 0x6932d1d0))
        self.assertNotEqual(wrong, rfc)
        self.assertEqual(wrong[3], 0x6932d1d0)
        self.assertEqual(rfc[3], 0x5881c4bb)
        self.assertEqual((0xff + 0x01) & 0xff, 0)
        self.assertEqual(0xff + 0x01, 0x100)
        self.assertFalse(4 < 4)
        self.assertTrue(0 <= 3 < 4)
        self.assertEqual([3 - i for i in range(4)], [3, 2, 1, 0])
        self.assertEqual((True and False), False)

    def test_opening_retained_byte_for_byte(self):
        import hashlib
        data = MANUSCRIPT.read_bytes()
        blob = b'blob ' + str(len(data)).encode('ascii') + b'\0' + data
        self.assertEqual(hashlib.sha1(blob).hexdigest(),
                         '62f7463f9008d3b56f935c84195adb1289b59d1b')


def n9_answers(text: str) -> dict[str, str]:
    """Split lesson N9 worked answers. The region stops before the source notes."""
    region = text.split('## Worked answers: N9', 1)[1]
    region = region.split('## Sources and epigraph record', 1)[0]
    parts = re.split(r'\n\*\*(N9\.\d+)\.\*\* ', '\n' + region)
    answers = {}
    items = iter(parts[1:])
    for ident, body in zip(items, items):
        answers[ident] = body
    return answers


def imply(hypothesis: int, conclusion: int) -> int:
    """Material implication on the lesson's two truth values."""
    return 0 if hypothesis == 1 and conclusion == 0 else 1


def bit_or_via_xor_and(left: int, right: int) -> tuple[int, int]:
    """Return (left OR right, (left XOR right) XOR (left AND right))."""
    return (left | right, (left ^ right) ^ (left & right))


def rotate_left_byte(value: int) -> int:
    """One-position mathematical left rotation, not an Orange interpreter."""
    return ((value << 1) | (value >> 7)) & 255


class LessonN9Reference(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.text = (ROOT / 'docs' / 'book' / 'NOVICE_LOGIC.md').read_text(encoding='utf-8')
        cls.answers = n9_answers(cls.text)
        cls.index = INDEX.read_text(encoding='utf-8')

    def test_twenty_exercises_have_one_answer_each(self):
        exercises = re.findall(r'^\*\*Exercise (N9\.\d+) —', self.text, re.M)
        self.assertEqual(exercises, [f'N9.{n}' for n in range(1, 21)])
        self.assertEqual(set(self.answers), set(exercises))

    def test_membership_cardinality_products_and_congruence(self):
        byte = {n for n in range(-2, 300) if 0 <= n <= 255}
        for member in (0, 1, 2, 255):
            self.assertIn(member, byte)
        for outsider in (256, -1):
            self.assertNotIn(outsider, byte)
        self.assertEqual(len({1, 0, 1}), 2)
        evens = {n for n in byte if n % 2 == 0}
        self.assertEqual(len(evens), 128)
        self.assertEqual(len(list(product((0, 1), repeat=2))), 4)
        self.assertEqual(256 * 256, 65536)
        self.assertEqual((2 ** 16) * (2 ** 16), 4294967296)
        # Congruence is transitive because differences add.
        self.assertEqual((19 - 3) + (3 - 35), 19 - 35)
        self.assertEqual((19 - 35) % 16, 0)
        answer = self.answers['N9.1'] + self.answers['N9.3'] + self.answers['N9.4']
        self.assertIn('Belong: 0, 1, 2, and 255.', self.answers['N9.1'])
        self.assertIn('Do not belong: 256 and -1.', self.answers['N9.1'])
        self.assertIn('The set has two elements.', self.answers['N9.2'])
        self.assertIn('The empty set is a subset.', self.answers['N9.3'])
        self.assertIn('`{256}` is not a subset.', self.answers['N9.3'])
        folded = re.sub(r'\s+', ' ', self.answers['N9.3'])
        self.assertIn(f'The even byte values number {len(evens)}.', folded)
        self.assertIn('There are 4 pairs.', self.answers['N9.4'])
        self.assertIn('The byte pair set has 65536 elements', self.answers['N9.4'])
        self.assertIn('4294967296', self.answers['N9.19'])
        self.assertIn('Belong: 0, 1, 2, and 255.', answer)

    def test_doubling_successor_rotation_and_induction_counts(self):
        image = {(2 * value) % 256 for value in range(256)}
        self.assertEqual((2 * 0) % 256, 0)
        self.assertEqual((2 * 128) % 256, 0)
        self.assertNotIn(1, image)
        self.assertEqual(image, set(range(0, 256, 2)))
        self.assertEqual(len(image), 128)
        for value in range(5):
            self.assertNotEqual(value + 1, 0)
        self.assertEqual(2 ** 0, 1)
        self.assertEqual(2 ** 3, 8)
        powers = [1, 2, 4, 8, 16, 32, 64, 128, 256]
        self.assertEqual([2 ** n for n in range(9)], powers)
        self.assertEqual(powers[0], 1)
        for index in range(8):
            self.assertEqual(powers[index + 1], powers[index] * 2)
        self.assertEqual(powers[8], 256)
        self.assertIn('[1, 2, 4, 8, 16, 32, 64, 128, 256]', self.text)
        self.assertEqual(f'{0x81:08b}', '10000001')
        self.assertEqual(f'{rotate_left_byte(0x81):08b}', '00000011')
        self.assertEqual(f'{(0x81 << 1) & 255:08b}', '00000010')
        self.assertEqual(bin(0x81).count('1'), 2)
        self.assertEqual(bin(rotate_left_byte(0x81)).count('1'), 2)
        self.assertEqual(bin((0x81 << 1) & 255).count('1'), 1)
        self.assertEqual((0x80 * 2) % 256, 0)
        self.assertEqual(bin(0x80).count('1'), 1)
        self.assertEqual(bin(0).count('1'), 0)
        for value in range(256):
            self.assertEqual(bin(rotate_left_byte(value)).count('1'), bin(value).count('1'))
            self.assertEqual(rotate_left_byte(rotate_left_byte(value) & 255) & 255,
                             rotate_left_byte(rotate_left_byte(value)))
            undone = rotate_left_byte(value)
            # Right rotation by one undoes left rotation by one.
            restored = ((undone >> 1) | ((undone & 1) << 7)) & 255
            self.assertEqual(restored, value)
        self.assertIn('Both 0 and 128 send to 0. The value 1 is missed.', self.answers['N9.7'])
        self.assertIn('the 128 even byte values', self.answers['N9.7'])
        self.assertIn('true; false; false; false; true.', self.answers['N9.12'])
        self.assertIn('`2^0 = 1` and `2^3 = 8`', self.answers['N9.17'])
        self.assertIn('`00000011`, which still has two ones', self.answers['N9.18'])
        self.assertIn('`00000010`, which has one', self.answers['N9.18'])
        self.assertIn('`N(0) = 0`', self.answers['N9.18'])
        self.assertIn('send 0 to both 0 and 128', self.answers['N9.15'])

    def test_printed_truth_tables_match_definitions(self):
        rows = re.findall(
            r'^\| `([01])` \| `([01])` \| `([01])` \|$',
            self.text,
            re.M,
        )
        self.assertEqual(rows, [('0', '0', '1'), ('0', '1', '1'), ('1', '0', '0'), ('1', '1', '1')])
        for left, right, result in rows:
            self.assertEqual(imply(int(left), int(right)), int(result))

        contra_section = self.text.split('¬ Q ⇒ ¬ P', 1)[1].split('The third column', 1)[0]
        contra = re.findall(
            r'^\| `([01])` \| `([01])` \| `([01])` \| `([01])` \| `([01])` \| `([01])` \|$',
            contra_section,
            re.M,
        )
        self.assertEqual(len(contra), 4)
        for p, q, forward, not_q, not_p, backward in contra:
            p, q = int(p), int(q)
            self.assertEqual(int(forward), imply(p, q))
            self.assertEqual(int(not_q), 1 - q)
            self.assertEqual(int(not_p), 1 - p)
            self.assertEqual(int(backward), imply(1 - q, 1 - p))
            self.assertEqual(int(forward), int(backward))

        or_section = self.text.split('`(a XOR b) XOR (a AND b)` |', 1)[1]
        or_section = or_section.split('Each row uses', 1)[0]
        or_rows = []
        for line in or_section.splitlines():
            cells = re.findall(r'`([01])`', line)
            if len(cells) == 6:
                or_rows.append(tuple(int(cell) for cell in cells))
        self.assertEqual(len(or_rows), 4)
        for a, b, or_bit, xor_bit, and_bit, combined in or_rows:
            self.assertEqual((a | b, a ^ b, a & b), (or_bit, xor_bit, and_bit))
            self.assertEqual(bit_or_via_xor_and(a, b), (or_bit, combined))
        folded_cases = re.sub(r'\s+', ' ', self.answers['N9.14'])
        self.assertIn('the common values `0`, `1`, `1`, `1`', folded_cases)
        self.assertIn('The implication is true.', self.answers['N9.11'])

    def test_quantified_doubling_answers_match_the_image(self):
        byte = range(256)
        doubling = lambda value: (2 * value) % 256
        claims = (
            all(doubling(value) % 2 == 0 for value in byte),
            any(doubling(value) == 1 for value in byte),
            all(any(doubling(value) == target for value in byte) for target in byte),
            any(all(doubling(value) == target for target in byte) for value in byte),
            all(any(output == doubling(value) for output in byte) for value in byte),
        )
        self.assertEqual(claims, (True, False, False, False, True))
        words = ['true' if claim else 'false' for claim in claims]
        self.assertIn('; '.join(words) + '.', self.answers['N9.12'])
        self.assertIn('that negation is false', self.answers['N9.13'])
        self.assertIn('that negation is true', self.answers['N9.13'])

    def test_labels_links_epigraph_and_scope(self):
        headings = re.findall(r'^#{1,6} (.+)$', self.text, re.M)
        self.assertIn('N9: Say What You Mean', headings)
        for heading in headings:
            self.assertNotRegex(heading, r'Chapter\s+(?:[1-9]|1[0-7])\b')
            self.assertNotRegex(heading, r'\bN[78]\b')
        quotes = re.findall(r'^> “(.+)”$', self.text, re.M)
        self.assertEqual([len(quote.split()) for quote in quotes], [11])
        self.assertIn('**[S7]', self.text)
        self.assertIn('pp. 644–654', self.text)
        self.assertEqual(self.text.count('```orange'), 1)
        self.assertIn('test "successor step through length 8"', self.text)
        self.assertIn('for i in 0..8', self.text)
        self.assertIn('**Assumption N9.4 — Induction by a successor step.**', self.text)
        self.assertIn('**Proposition N9.16 — Eight successor steps.**', self.text)
        self.assertIn('**Listing N9.1 — `length_count.or`**', self.text)
        self.assertIn('proof by contradiction in §N9.18', self.text)
        self.assertNotIn('proof by contradiction in §N9.20', self.text)
        self.assertNotIn('will eventually', self.text)
        self.assertNotIn('unwritten', self.text)
        self.assertNotIn('complete induction', self.text.lower())
        anchors = {github_anchor(heading) for heading in headings}
        for fragment in re.findall(r'NOVICE_LOGIC\.md#([^)]+)', self.index):
            self.assertIn(fragment, anchors)
        programming = (ROOT / 'docs' / 'book' / 'NOVICE_PROGRAMMING.md').read_text(encoding='utf-8')
        programming_anchors = {
            github_anchor(heading)
            for heading in re.findall(r'^#{1,6} (.+)$', programming, re.M)
        }
        for fragment in re.findall(r'NOVICE_PROGRAMMING\.md#([^)]+)', self.text):
            self.assertIn(fragment, programming_anchors)
        self.assertIn('from the output alone, both', self.answers['N9.20'])
        self.assertIn('hides nothing', self.answers['N9.8'])
        self.assertGreaterEqual(len(set(re.findall(r'NOVICE_LOGIC\.md#([^)]+)', self.index))), 3)


class N10Probability(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.text = (ROOT / 'docs' / 'book' / 'NOVICE_PROBABILITY.md').read_text(encoding='utf-8')
        cls.index = INDEX.read_text(encoding='utf-8')

    def test_n10_exercises_label_epigraph_and_anchors(self):
        exercises = re.findall(r'^\*\*Exercise (N10\.\d+) —', self.text, re.M)
        answers = re.findall(r'^\*\*(N10\.\d+)\.\*\*', self.text, re.M)
        self.assertEqual(exercises, [f'N10.{n}' for n in range(1, 17)])
        self.assertEqual(sorted(exercises), sorted(answers))
        self.assertNotRegex(self.text, r'(?m)^#+ Chapter (?:7|8|9|10|11|12)\b')
        self.assertNotRegex(self.text, r'(?m)^#+ N[789]\b')
        self.assertRegex(self.text, r'(?m)^## N10: Count What You Do Not Know$')
        quotes = re.findall(r'^> “(.+)”$', self.text, re.M)
        self.assertEqual(quotes, [
            'Although it is always possible in principle to determine these solutions '
            '(by trial of each possible key for example), different enciphering systems '
            'show a wide variation in the amount of work required.'
        ])
        self.assertIn('https://pages.cs.wisc.edu/~rist/642-spring-2014/shannon-secrecy.pdf', self.text)
        self.assertIn('**N10.**', self.index)
        self.assertIn('NOVICE_PROBABILITY.md#n10-count-what-you-do-not-know', self.index)
        self.assertNotIn('Chapter 7', self.index)
        headings = re.findall(r'^#{1,6} (.+)$', self.text, re.M)
        anchors = {github_anchor(h) for h in headings}
        self.assertIn('n10-count-what-you-do-not-know', anchors)
        for fragment in re.findall(r'NOVICE_PROBABILITY\.md#([^)\s]+)', self.index):
            self.assertIn(fragment, anchors)
        opening = (ROOT / 'docs' / 'book' / 'NOVICE_OPENING.md').read_text(encoding='utf-8')
        opening_anchors = {github_anchor(h) for h in re.findall(r'^#{1,6} (.+)$', opening, re.M)}
        for fragment in re.findall(r'NOVICE_OPENING\.md#([^)\s]+)', self.text):
            self.assertIn(fragment, opening_anchors)

    def test_n10_ledger_matches_exact_rationals(self):
        block = re.search(r'^```text\nn10-ledger\n(.*?)\n```', self.text, re.M | re.S)
        self.assertIsNotNone(block)
        printed = {}
        for line in block.group(1).splitlines():
            name, value = line.split(' = ')
            num, den = value.split('/')
            printed[name] = Fraction(int(num), int(den))
        distinct_3_5 = Fraction(4, 5) * Fraction(3, 5)
        distinct_5_5 = Fraction(4, 5) * Fraction(3, 5) * Fraction(2, 5) * Fraction(1, 5)
        s_3_5 = Fraction(3 * 2, 2 * 5)
        s_23 = Fraction(23 * 22, 2 * 365)
        expected = {
            'sum-half-third': Fraction(1, 2) + Fraction(1, 3),
            'bad-numerator-sum': Fraction(1 + 1, 2 + 3),
            'three-fraction-sum': Fraction(1, 2) + Fraction(1, 3) + Fraction(1, 6),
            'rejected-weights': Fraction(1, 2) + Fraction(1, 3) + Fraction(1, 7),
            'conditional-unequal-00': Fraction(1, 2) / (Fraction(1, 2) + Fraction(1, 6)),
            'conditional-unequal-01': Fraction(1, 6) / (Fraction(1, 2) + Fraction(1, 6)),
            'conditional-uniform-00': Fraction(1, 4) / Fraction(1, 2),
            'conditional-reverse': Fraction(1, 2) / Fraction(1, 2),
            'second-bit-one': Fraction(1, 6) + Fraction(1, 6),
            'second-bit-complement': 1 - (Fraction(1, 6) + Fraction(1, 6)),
            'disjoint-product': Fraction(1, 2) * Fraction(1, 2),
            'draw-product': Fraction(1, 3) * Fraction(1, 3),
            'byte-bit-count': Fraction(256 * 2, 1),
            'byte-bit-zero': Fraction(1, 256) * Fraction(1, 2),
            'birthday-3-5-distinct': distinct_3_5,
            'birthday-3-5-collision': 1 - distinct_3_5,
            'birthday-3-5-upper': s_3_5,
            'birthday-3-5-lower': s_3_5 / (1 + s_3_5),
            'birthday-5-5-collision': 1 - distinct_5_5,
            'birthday-5-5-lower': Fraction(2, 3),
            'birthday-23-upper': s_23,
            'birthday-23-lower': s_23 / (1 + s_23),
            'expected-pairs-3-5': s_3_5,
            'triple-pair-event': Fraction(1, 25),
            'triple-pair-product': Fraction(1, 125),
            'expected-trials-4': Fraction(4 + 1, 2),
            'early-stop-4': Fraction(2, 4),
            'expected-trials-remaining': Fraction(2 + 1, 2),
            'sixteen-bit-count': Fraction(256 * 256, 1),
        }
        self.assertEqual(printed, expected)
        outside = self.text.replace(block.group(0), '')
        for value in ('5/6', '2/5', '41/42', '3/4', '13/25', '3/8', '601/625',
                      '253/365', '253/618', '5/2', '3/2', '1/512', '1/125'):
            self.assertIn(value, outside)

    def test_n10_bounds_trials_and_fraction_rules(self):
        def collision(people, days):
            if people > days:
                return Fraction(1)
            distinct = Fraction(1)
            for k in range(1, people):
                distinct *= Fraction(days - k, days)
            return 1 - distinct

        def pair_ratio(people, days):
            return Fraction(people * (people - 1), 2 * days)

        for days in range(1, 9):
            for people in range(1, days + 3):
                probability = collision(people, days)
                bound = pair_ratio(people, days)
                self.assertGreaterEqual(probability, 0)
                self.assertLessEqual(probability, 1)
                self.assertLessEqual(probability, bound)
                if people <= days:
                    self.assertGreaterEqual(probability, bound / (1 + bound))
                else:
                    self.assertEqual(probability, 1)
        self.assertEqual(5 * 4 * 3 + 3 * 5 * 4 + 5, 125)
        self.assertEqual(Fraction(65, 125), Fraction(13, 25))
        numerator = 1
        denominator = 1
        for k in range(1, 23):
            numerator *= 365 - k
            denominator *= 365
        self.assertLess(2 * numerator, denominator)
        upper = Fraction(253, 365)
        lower = Fraction(253, 618)
        self.assertLess(lower, Fraction(1, 2))
        self.assertGreater(upper, Fraction(1, 2))
        for width in range(0, 41):
            self.assertEqual(sum(range(width + 1)), width * (width + 1) // 2)
        for size in range(1, 31):
            self.assertEqual(sum(range(1, size + 1)) / size, (size + 1) / 2)
        for numerator in range(-3, 4):
            for denominator in range(1, 5):
                for other_num in range(-3, 4):
                    for other_den in range(1, 5):
                        for factor in range(1, 4):
                            left = Fraction(numerator, denominator)
                            right = Fraction(other_num, other_den)
                            scaled = Fraction(factor * numerator, factor * denominator)
                            self.assertEqual(left, scaled)
                            self.assertEqual(left + right, Fraction(
                                numerator * other_den + denominator * other_num,
                                denominator * other_den))
                            self.assertEqual(left * right, Fraction(
                                numerator * other_num, denominator * other_den))


class N11Protect(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.text = (ROOT / 'docs' / 'book' / 'NOVICE_PROTECT.md').read_text(encoding='utf-8')
        cls.index = INDEX.read_text(encoding='utf-8')

    def test_n11_exercises_label_epigraph_and_anchors(self):
        exercises = re.findall(r'^\*\*Exercise (N11\.\d+) —', self.text, re.M)
        answers = re.findall(r'^\*\*(N11\.\d+)\.\*\*', self.text, re.M)
        self.assertEqual(exercises, [f'N11.{n}' for n in range(1, 17)])
        self.assertEqual(sorted(exercises), sorted(answers))
        self.assertNotRegex(self.text, r'(?m)^#+ Chapter (?:7|8|9|10|11|12)\b')
        self.assertNotRegex(self.text, r'(?m)^#+ N1[02]\b')
        self.assertRegex(self.text, r'(?m)^## N11: Protect More Than Appearance$')
        quotes = re.findall(r'^> “(.+)”$', self.text, re.M)
        self.assertEqual(quotes, [
            'In this case, intercepting the message has given the '
            'cryptanalyst no information.'
        ])
        self.assertIn('https://pages.cs.wisc.edu/~rist/642-spring-2014/shannon-secrecy.pdf', self.text)
        self.assertIn('**N11.**', self.index)
        self.assertIn('NOVICE_PROTECT.md#n11-protect-more-than-appearance', self.index)
        headings = re.findall(r'^#{1,6} (.+)$', self.text, re.M)
        anchors = {github_anchor(h) for h in headings}
        self.assertIn('n11-protect-more-than-appearance', anchors)
        for fragment in re.findall(r'NOVICE_PROTECT\.md#([^)\s]+)', self.index):
            self.assertIn(fragment, anchors)
        opening = (ROOT / 'docs' / 'book' / 'NOVICE_OPENING.md').read_text(encoding='utf-8')
        opening_anchors = {github_anchor(h) for h in re.findall(r'^#{1,6} (.+)$', opening, re.M)}
        for fragment in re.findall(r'NOVICE_OPENING\.md#([^)\s]+)', self.text):
            self.assertIn(fragment, opening_anchors)
        probability = (ROOT / 'docs' / 'book' / 'NOVICE_PROBABILITY.md').read_text(encoding='utf-8')
        probability_anchors = {
            github_anchor(h) for h in re.findall(r'^#{1,6} (.+)$', probability, re.M)
        }
        for fragment in re.findall(r'NOVICE_PROBABILITY\.md#([^)\s]+)', self.text):
            self.assertIn(fragment, probability_anchors)

    def test_n11_ledger_matches_exact_values(self):
        block = re.search(r'^```text\nn11-ledger\n(.*?)\n```', self.text, re.M | re.S)
        self.assertIsNotNone(block)
        printed = {}
        for line in block.group(1).splitlines():
            name, value = line.split(' = ')
            num, den = value.split('/')
            printed[name] = Fraction(int(num), int(den))
        factorial = 1
        for n in range(1, 27):
            factorial *= n
        units = [a for a in range(26) if math_gcd(a, 26) == 1]
        expected = {
            'shift-keys': Fraction(26, 1),
            'affine-units': Fraction(len(units), 1),
            'affine-keys': Fraction(len(units) * 26, 1),
            'excluded-affine-pairs': Fraction(26 * 26 - len(units) * 26, 1),
            'shift-expected-trials': Fraction(26 + 1, 2),
            'affine-expected-trials': Fraction(len(units) * 26 + 1, 2),
            'substitution-keys': Fraction(factorial, 1),
            'small-substitution-keys': Fraction(120, 1),
            'uniform-joint-m0-c0': Fraction(1, 4) * Fraction(1, 2),
            'uniform-joint-m1-c0': Fraction(3, 4) * Fraction(1, 2),
            'uniform-cipher': Fraction(1, 2),
            'uniform-posterior-m0': Fraction(1, 4),
            'biased-joint-m0-c0': Fraction(1, 4) * Fraction(3, 4),
            'biased-joint-m1-c0': Fraction(3, 4) * Fraction(1, 4),
            'biased-cipher-0': Fraction(3, 8),
            'biased-posterior-m0-c0': Fraction(1, 2),
            'biased-joint-m0-c1': Fraction(1, 4) * Fraction(1, 4),
            'biased-cipher-1': Fraction(5, 8),
            'biased-posterior-m0-c1': Fraction(1, 10),
            'two-time-prior': Fraction(1, 256),
            'two-time-posterior': Fraction(1, 1),
            'pad-difference': Fraction(0x41 ^ 0x42, 1),
            'hello-row': Fraction(3, 1),
            'inverse-of-5': Fraction(21, 1),
            'inverse-of-9': Fraction(3, 1),
            'inverse-of-25': Fraction(25, 1),
            'known-plaintext-b': Fraction(3, 1),
        }
        self.assertEqual(printed, expected)
        self.assertEqual(len(units), 12)
        self.assertEqual((5 * 21) % 26, 1)
        self.assertEqual((9 * 3) % 26, 1)
        self.assertEqual((25 * 25) % 26, 1)

    def test_n11_exhaustive_pad_shift_and_affine(self):
        for q in range(2, 8):
            for message in range(q):
                for ciphertext in range(q):
                    keys = [key for key in range(q) if (message + key) % q == ciphertext]
                    self.assertEqual(keys, [(ciphertext - message) % q])
                    self.assertEqual((ciphertext - keys[0]) % q, message)
        for q, length in ((2, 1), (2, 2), (3, 2), (5, 2)):
            symbols = range(q)
            strings = list(product(symbols, repeat=length))
            for message in strings:
                for ciphertext in strings:
                    keys = [
                        key for key in strings
                        if all((message[i] + key[i]) % q == ciphertext[i] for i in range(length))
                    ]
                    self.assertEqual(len(keys), 1)
                    self.assertEqual(
                        tuple((ciphertext[i] - keys[0][i]) % q for i in range(length)),
                        message,
                    )
        for message in range(26):
            for key in range(26):
                ciphertext = (message + key) % 26
                self.assertEqual((ciphertext - key) % 26, message)
        units = [a for a in range(26) if math_gcd(a, 26) == 1]
        for multiplier in units:
            inverse = next(x for x in range(26) if (multiplier * x) % 26 == 1)
            for addend in range(26):
                for message in range(26):
                    ciphertext = (multiplier * message + addend) % 26
                    recovered = (inverse * ((ciphertext - addend) % 26)) % 26
                    self.assertEqual(recovered, message)
        for multiplier in range(26):
            if math_gcd(multiplier, 26) == 1:
                continue
            images = {(multiplier * message) % 26 for message in range(26)}
            self.assertLess(len(images), 26)
        for first in range(256):
            for second in range(256):
                self.assertEqual(first ^ second ^ second, first)
                for key in (0x00, 0x3c, 0xff):
                    self.assertEqual((first ^ key) ^ (second ^ key), first ^ second)
        prior_message = {0: Fraction(1, 4), 1: Fraction(3, 4)}
        for key_weights, expected_posterior in (
            ({0: Fraction(1, 2), 1: Fraction(1, 2)}, Fraction(1, 4)),
            ({0: Fraction(3, 4), 1: Fraction(1, 4)}, Fraction(1, 2)),
        ):
            joint = {}
            for message in (0, 1):
                for key in (0, 1):
                    ciphertext = message ^ key
                    joint[(message, ciphertext)] = prior_message[message] * key_weights[key]
            cipher = sum(weight for (message, ciphertext), weight in joint.items() if ciphertext == 0)
            posterior = joint[(0, 0)] / cipher
            self.assertEqual(posterior, expected_posterior)

    def test_n11_bezout_is_the_successor_step(self):
        folded = re.sub(r'\s+', ' ', self.text)
        self.assertEqual(self.text.count('Assumption N9.4'), 5)
        self.assertIn('the base `P(0)`, and the step', folded)
        self.assertIn('`P(n) ⇒ P(n + 1)` with hypothesis `P(n)` alone', folded)
        self.assertIn('The step is `Q(n) ⇒ Q(n + 1)`.', folded)
        self.assertIn('The hypothesis was `Q(n)` only.', folded)
        self.assertIn(
            'The second paragraph of Assumption N9.4 therefore',
            folded,
        )
        self.assertIn(
            'Assumption N9.4 only as the successor step `Q(n) ⇒ Q(n + 1)`',
            folded,
        )
        self.assertNotIn('complete induction', folded.lower())
        self.assertNotIn('modulus strictly less than', folded)
        self.assertNotIn('descent cannot', folded)
        self.assertNotIn('Induct on the positive integer', folded)
        for modulus in range(1, 41):
            for integer in range(-40, 41):
                coefficient_x, coefficient_y = successor_bezout(integer, modulus)
                divisor = positive_gcd(integer, modulus)
                self.assertEqual(
                    integer * coefficient_x + modulus * coefficient_y,
                    divisor,
                )
        for multiplier, inverse in ((5, 21), (9, 3), (25, 25)):
            coefficient_x, _coefficient_y = successor_bezout(multiplier, 26)
            self.assertEqual(coefficient_x % 26, inverse)
            self.assertEqual((multiplier * inverse) % 26, 1)


class N12CompleteStudy(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.text = (ROOT / 'docs' / 'book' / 'NOVICE_N12_THE_FIRST_COMPLETE_STUDY.md').read_text(
            encoding='utf-8'
        )
        cls.index = INDEX.read_text(encoding='utf-8')
        cls.protect = (ROOT / 'docs' / 'book' / 'NOVICE_PROTECT.md').read_text(encoding='utf-8')

    def test_n12_exercises_label_and_anchor(self):
        exercises = re.findall(r'^\*\*Exercise (N12\.\d+) —', self.text, re.M)
        answers = re.findall(r'^\*\*(N12\.\d+)\.\*\*', self.text, re.M)
        self.assertEqual(exercises, [f'N12.{n}' for n in range(1, 13)])
        self.assertEqual(sorted(exercises), sorted(answers))
        self.assertNotRegex(self.text, r'(?m)^#+ Chapter (?:7|8|9|10|11|12)\b')
        self.assertRegex(self.text, r'(?m)^## N12: The First Complete Study$')
        quotes = re.findall(r'^> “(.+)”$', self.text, re.M)
        self.assertEqual(quotes, [
            'I speculate that ChaCha has similar resistance to “ChaCha” against the attack, '
            'but of course this has to be checked carefully.'
        ])
        self.assertIn('https://cr.yp.to/chacha/chacha-20080128.pdf', self.text)
        self.assertIn('https://www.rfc-editor.org/rfc/rfc8439', self.text)
        self.assertIn('**N12.**', self.index)
        self.assertIn(
            'NOVICE_N12_THE_FIRST_COMPLETE_STUDY.md#n12-the-first-complete-study',
            self.index,
        )
        headings = re.findall(r'^#{1,6} (.+)$', self.text, re.M)
        anchors = {github_anchor(h) for h in headings}
        self.assertIn('n12-the-first-complete-study', anchors)
        self.assertIn('worked-answers', anchors)
        for fragment in re.findall(r'NOVICE_N12_THE_FIRST_COMPLETE_STUDY\.md#([^)\s]+)', self.index):
            self.assertIn(fragment, anchors)
        self.assertIn('[S10]', self.text)
        self.assertNotIn('[S9]', self.text)
        self.assertIn('**[S10] Daniel J. Bernstein.**', self.text)
        self.assertIn('**[T6] Orange tests.**', self.text)
        self.assertNotIn('[T1]', self.text)
        self.assertIn('**[S9] Claude E. Shannon.**', self.protect)
        self.assertNotIn('provisional', self.text.lower())
        self.assertNotIn('integration plan', self.text.lower())

    def test_n9_through_n12_labels_are_locked(self):
        probability = (ROOT / 'docs' / 'book' / 'NOVICE_PROBABILITY.md').read_text(
            encoding='utf-8'
        )
        logic = (ROOT / 'docs' / 'book' / 'NOVICE_LOGIC.md').read_text(encoding='utf-8')
        for name, text in (
            ('index', self.index),
            ('n9', logic),
            ('n10', probability),
            ('n11', self.protect),
            ('n12', self.text),
        ):
            self.assertNotIn('provisional', text.lower(), name)
            self.assertNotIn('integration plan', text.lower(), name)
            self.assertNotIn('Final numbering', text, name)
        self.assertIn('The locked label is N9.', self.index)
        self.assertIn('The locked label is N10.', self.index)
        self.assertIn('The locked label is N11.', self.index)
        self.assertIn('The locked label is N12.', self.index)
        self.assertIn('The locked label is N10.', probability)
        self.assertIn('The locked label is N11.', self.protect)
        self.assertIn('The locked label is N12.', self.text)
        self.assertIn('**N9.**', self.index)
        self.assertIn('**N10.**', self.index)
        self.assertIn('**N11.**', self.index)
        self.assertIn('**N12.**', self.index)

    def test_novice_s_and_t_tags_have_one_referent(self):
        """[S*] and [T*] source records are unique across the novice arc."""
        definition = re.compile(r'\*\*\[([ST]\d+)\] ([^*]+)\*\*')
        citation = re.compile(r'\[([ST]\d+)\]')
        records = {}
        texts = []
        for path in sorted((ROOT / 'docs' / 'book').glob('NOVICE*.md')):
            text = path.read_text(encoding='utf-8')
            texts.append((path.name, text))
            for match in definition.finditer(text):
                tag, referent = match.group(1), match.group(2).strip()
                previous = records.get(tag)
                self.assertIsNone(
                    previous,
                    f'{tag} already names {previous} and also {path.name}: {referent}',
                )
                records[tag] = (path.name, referent)
        for name, text in texts:
            for tag in citation.findall(text):
                self.assertIn(tag, records, f'{name} cites undefined [{tag}]')
        self.assertEqual(records['S10'], (
            'NOVICE_N12_THE_FIRST_COMPLETE_STUDY.md',
            'Daniel J. Bernstein.',
        ))
        self.assertEqual(records['T1'], ('NOVICE_PROGRAMMING.md', 'GNU Bash.'))
        self.assertEqual(records['T6'], (
            'NOVICE_N12_THE_FIRST_COMPLETE_STUDY.md',
            'Orange tests.',
        ))
        self.assertEqual(records['S11'], (
            'NOVICE_N13_MODULES_AND_PROVENANCE.md',
            'H. Krawczyk, M. Bellare, and R. Canetti.',
        ))
        self.assertEqual(records['T7'], (
            'NOVICE_N13_MODULES_AND_PROVENANCE.md',
            'Orange modules.',
        ))
        n12_tags = {
            tag for tag, (name, _) in records.items()
            if name == 'NOVICE_N12_THE_FIRST_COMPLETE_STUDY.md'
        }
        self.assertIn('S10', n12_tags)
        self.assertIn('T6', n12_tags)
        self.assertNotIn('T1', n12_tags)
        self.assertNotIn('S9', n12_tags)

    def test_n12_ledger_matches_the_word_arithmetic(self):
        block = re.search(r'^```text\nn12-ledger\n(.*?)\n```', self.text, re.M | re.S)
        self.assertIsNotNone(block)
        printed = {}
        for line in block.group(1).splitlines():
            name, value = line.split(' = ')
            printed[name] = int(value)
        modulus = 2 ** 32
        expected = {
            'sample-sum': 0x77777777 + 0x01234567,
            'quarter-wrap-sum': 0xecff8273 + 0x5881c4bb,
            'quarter-wrap-residue': (0xecff8273 + 0x5881c4bb) % modulus,
            'state-wrap-sum': 0x53372767 + 0xc47446a0,
            'state-wrap-residue': (0x53372767 + 0xc47446a0) % modulus,
            'column-wrap-sum': 0x8c767582 + 0xdec62ed2,
            'column-wrap-residue': (0x8c767582 + 0xdec62ed2) % modulus,
            'feed0': 0x837778ab + 0x61707865,
            'feed1-sum': 0xe238d763 + 0x3320646e,
            'feed1-residue': (0xe238d763 + 0x3320646e) % modulus,
        }
        self.assertEqual(printed, expected)
        self.assertEqual(expected['quarter-wrap-residue'], 0x4581472e)
        self.assertEqual(expected['state-wrap-residue'], 0x17ab6e07)
        self.assertEqual(expected['column-wrap-residue'], 0x6b3ca454)
        self.assertEqual(expected['feed0'], 0xe4e7f110)
        self.assertEqual(expected['feed1-residue'], 0x15593bd1)
        self.assertLess(expected['feed0'], modulus)
        self.assertGreaterEqual(expected['feed1-sum'], modulus)
        self.assertLess(expected['feed1-sum'], 2 * modulus)
        self.assertEqual((0xfc62bb2f + 0x07060504) % modulus, 0x0368c033)
        self.assertEqual(0xd19c12b4 + 1, 0xd19c12b5)


class N13ModulesProvenance(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.text = (ROOT / 'docs' / 'book' / 'NOVICE_N13_MODULES_AND_PROVENANCE.md').read_text(
            encoding='utf-8'
        )
        cls.index = INDEX.read_text(encoding='utf-8')

    def test_n13_exercises_label_and_anchor(self):
        exercises = re.findall(r'^\*\*Exercise (N13\.\d+) —', self.text, re.M)
        answers = re.findall(r'^\*\*(N13\.\d+)\.\*\*', self.text, re.M)
        self.assertEqual(exercises, [f'N13.{n}' for n in range(1, 11)])
        self.assertEqual(sorted(exercises), sorted(answers))
        self.assertNotRegex(self.text, r'(?m)^#+ .*Chapter 13\b')
        self.assertRegex(self.text, r'(?m)^## N13: Modules and Provenance$')
        quotes = re.findall(r'^> “(.+)”$', self.text, re.M)
        self.assertEqual(quotes, [
            'The definition of HMAC requires a cryptographic hash function, which we denote by H, '
            'and a secret key K.'
        ])
        self.assertIn('https://www.rfc-editor.org/rfc/rfc2104', self.text)
        self.assertIn('https://doi.org/10.6028/NIST.FIPS.180-4', self.text)
        self.assertIn(
            'https://csrc.nist.gov/CSRC/media/Projects/Cryptographic-Standards-and-Guidelines/documents/examples/SHA256.pdf',
            self.text,
        )
        self.assertIn('https://www.rfc-editor.org/rfc/rfc4231', self.text)
        self.assertIn('**N13.**', self.index)
        self.assertIn(
            'NOVICE_N13_MODULES_AND_PROVENANCE.md#n13-modules-and-provenance',
            self.index,
        )
        headings = re.findall(r'^#{1,6} (.+)$', self.text, re.M)
        anchors = {github_anchor(h) for h in headings}
        self.assertIn('n13-modules-and-provenance', anchors)
        self.assertIn('worked-answers', anchors)
        for fragment in re.findall(r'NOVICE_N13_MODULES_AND_PROVENANCE\.md#([^)\s]+)', self.index):
            self.assertIn(fragment, anchors)
        self.assertIn('The locked label is N13.', self.text)
        self.assertIn('The locked label is N13.', self.index)
        self.assertIn('**[S11] H. Krawczyk, M. Bellare, and R. Canetti.**', self.text)
        self.assertIn('**[T7] Orange modules.**', self.text)
        self.assertNotIn('**[T6] Orange tests.**', self.text)
        self.assertNotIn('complete induction', self.text.lower())
        self.assertNotIn('provisional', self.text.lower())
        self.assertNotIn('integration plan', self.text.lower())

    def test_n13_modules_call_by_name_with_provenance(self):
        """Separate files, a qualified call, a pinned vector, and one Match."""
        sources = re.findall(r'^```orange\n(.*?)\n```', self.text, re.M | re.S)
        self.assertEqual(len(sources), 6)
        sha = [source for source in sources if '\nmodule sha256 {' in source]
        hmac = [source for source in sources if '\nmodule hmac {' in source]
        self.assertEqual(len(sha), 1)
        self.assertEqual(len(hmac), 1)
        self.assertNotEqual(sha[0], hmac[0])
        self.assertNotIn('\n  use ', sha[0])
        self.assertIn('\n  use sha256;\n', hmac[0])
        self.assertIn('sha256::compress(', hmac[0])
        self.assertNotIn('spec compress(', hmac[0])
        self.assertNotIn('spec hash(', hmac[0])
        folded = re.sub(r'\s+', ' ', self.text)
        self.assertIn('FIPS PUB 180-4', self.text)
        self.assertIn('August 2015', self.text)
        self.assertIn('10.6028/NIST.FIPS.180-4', self.text)
        self.assertIn('§5.1.1', self.text)
        self.assertIn('one-block message sample', folded)
        self.assertIn('RFC 4231', self.text)
        self.assertIn('§4.2', self.text)
        self.assertIn('test case 1', folded)
        self.assertIn('test case 2', folded)
        self.assertIn('does not', self.text)
        self.assertIn('Do not call that Match verified.', self.text)
        self.assertIn('A passing test is a Match on the inputs it writes.', folded)
        self.assertIn('`Q(n) ⇒ Q(n + 1)`', self.text)
        self.assertIn('The pad claim does not have that step.', folded)
        self.assertIn('February 1997', self.text)
        mac = hmac[0].split('spec mac(', 1)[1].split('spec case1_key', 1)[0]
        self.assertEqual(mac.count('sha256::'), 10)

    def test_n13_ledger_matches_the_pad_arithmetic(self):
        block = re.search(r'^```text\nn13-ledger\n(.*?)\n```', self.text, re.M | re.S)
        self.assertIsNotNone(block)
        printed = {}
        for line in block.group(1).splitlines():
            name, value = line.split(' = ')
            printed[name] = int(value)
        expected = {
            'block-bits': 512,
            'block-bytes': 512 // 8,
            'pad-xor': 0x36 ^ 0x5c,
            'byte0-inner': 0x0b ^ 0x36,
            'byte0-outer': 0x0b ^ 0x5c,
            'case1-key-len': 20,
            'case1-zero-bytes': 64 - 20,
            'inner-total': 64 + 8,
            'inner-bits': 8 * (64 + 8),
            'outer-total': 64 + 32,
            'outer-bits': 8 * (64 + 32),
        }
        self.assertEqual(printed, expected)
        self.assertEqual(expected['pad-xor'], 106)
        self.assertEqual(expected['byte0-inner'], 61)
        self.assertEqual(expected['byte0-outer'], 87)
        self.assertEqual(expected['case1-zero-bytes'], 44)
        self.assertEqual(expected['inner-bits'], 576)
        self.assertEqual(expected['outer-bits'], 768)
        for byte in range(256):
            self.assertEqual((byte ^ 0x36) ^ (byte ^ 0x5c), 0x6a)
            self.assertNotEqual(byte ^ 0x36, byte ^ 0x5c)


class N14ReadyForStandards(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.text = (ROOT / 'docs' / 'book' / 'NOVICE_N14_READY_FOR_STANDARDS.md').read_text(
            encoding='utf-8'
        )
        cls.index = INDEX.read_text(encoding='utf-8')

    def test_n14_exercises_label_and_anchor(self):
        exercises = re.findall(r'^\*\*Exercise (N14\.\d+) —', self.text, re.M)
        answers = re.findall(r'^\*\*(N14\.\d+)\.\*\*', self.text, re.M)
        self.assertEqual(exercises, [f'N14.{n}' for n in range(1, 9)])
        self.assertEqual(sorted(exercises), sorted(answers))
        self.assertNotRegex(self.text, r'(?m)^#+ .*Chapter 14\b')
        self.assertNotIn('Chapter 14', self.text)
        self.assertRegex(self.text, r'(?m)^## N14: Ready for Standards$')
        for number in range(1, 7):
            self.assertRegex(self.text, rf'(?m)^### N14\.{number} ')
        quotes = re.findall(r'^> “(.+)”$', self.text, re.M)
        self.assertEqual(quotes, [
            'Perhaps the largest group of readers will consist of people who want to read a full '
            'and unambiguous description of Rijndael.'
        ])
        self.assertIn('https://cs.ru.nl/~joan/papers/JDA_VRI_Rijndael_2002.pdf', self.text)
        self.assertIn('**N14.**', self.index)
        self.assertIn(
            'NOVICE_N14_READY_FOR_STANDARDS.md#n14-ready-for-standards',
            self.index,
        )
        headings = re.findall(r'^#{1,6} (.+)$', self.text, re.M)
        anchors = {github_anchor(h) for h in headings}
        self.assertIn('n14-ready-for-standards', anchors)
        self.assertIn('worked-answers', anchors)
        for fragment in re.findall(r'NOVICE_N14_READY_FOR_STANDARDS\.md#([^)\s]+)', self.index):
            self.assertIn(fragment, anchors)
        self.assertIn('The locked label is N14.', self.text)
        self.assertIn('The locked label is N14.', self.index)
        self.assertIn('**[S12] Joan Daemen and Vincent Rijmen.**', self.text)
        self.assertIn('**[T8] Orange edition.**', self.text)
        self.assertIn('**[C2] Gate surface.**', self.text)
        self.assertNotIn('provisional', self.text.lower())
        self.assertNotIn('integration plan', self.text.lower())
        self.assertNotIn('complete induction', self.text.lower())

    def test_n14_is_a_gate_with_interpret_domain_and_assumptions(self):
        """Interpret walk, finite Match, assumption list, no compress transcription."""
        self.assertIn('Read the constructs in source order.', self.text)
        self.assertIn('**Listing N14.1 — `pad.or`**', self.text)
        self.assertIn('**Listing N14.2 — `pad_seam.or`**', self.text)
        self.assertIn('`use pad;`', self.text)
        self.assertIn('spec inner0()', self.text)
        self.assertIn('**Proposition N14.1.**', self.text)
        self.assertIn('inner0', self.text)
        self.assertIn('0x3d', self.text)
        self.assertIn('Do not call that Match verified.', self.text)
        self.assertIn('The test does not cover SHA-256.', self.text)
        self.assertIn('The test does not cover HMAC.', self.text)
        self.assertIn('The test does not cover `inner0`.', self.text)
        for heading in ('Edition.', 'Endianness.', 'Padding.', 'Module seam.', 'Non-claims.'):
            self.assertIn(heading, self.text)
        self.assertIn('you are not ready for J2–J4', self.text)
        self.assertIn('I am not ready for J2–J4.', self.text)
        self.assertIn('FIPS 180-4 §6.2.2', self.text)
        for forbidden in (
            'small_sigma0',
            'small_sigma1',
            'big_sigma0',
            'big_sigma1',
            '0x428a2f98',
            'spec schedule(',
            'spec compress(',
            'spec round(',
            'round_constants',
        ):
            self.assertNotIn(forbidden, self.text)
        sources = re.findall(r'^```orange\n(.*?)\n```', self.text, re.M | re.S)
        self.assertEqual(len(sources), 3)
        self.assertTrue(any('\nmodule pad {' in source for source in sources))
        self.assertTrue(any('\nmodule pad_seam {' in source for source in sources))
        self.assertTrue(any('\nmodule sample_line {' in source for source in sources))
        self.assertFalse(any('compress' in source for source in sources))

    def test_n14_ledger_matches_the_byte_arithmetic(self):
        block = re.search(r'^```text\nn14-ledger\n(.*?)\n```', self.text, re.M | re.S)
        self.assertIsNotNone(block)
        printed = {}
        for line in block.group(1).splitlines():
            name, value = line.split(' = ')
            printed[name] = int(value)
        expected = {
            'byte0': 0x0b,
            'inner-pad': 0x36,
            'inner0': 0x0b ^ 0x36,
            'outer-pad': 0x5c,
            'pad-xor': 0x36 ^ 0x5c,
            'zero-inner': 0x00 ^ 0x36,
            'zero-outer': 0x00 ^ 0x5c,
            'key-ones': 20,
            'key-zeros': 64 - 20,
            'sample-sum': 0x77777777 + 0x01234567,
        }
        self.assertEqual(printed, expected)
        self.assertEqual(expected['inner0'], 0x3d)
        self.assertEqual(expected['pad-xor'], 0x6a)
        self.assertEqual(expected['sample-sum'], 0x789abcde)
        self.assertLess(expected['sample-sum'], 2 ** 32)

    def test_n14_tags_are_the_next_free_numbers(self):
        definition = re.compile(r'\*\*\[([STC]\d+)\] ([^*]+)\*\*')
        records = {}
        for path in sorted((ROOT / 'docs' / 'book').glob('NOVICE*.md')):
            text = path.read_text(encoding='utf-8')
            for match in definition.finditer(text):
                tag, referent = match.group(1), match.group(2).strip()
                previous = records.get(tag)
                self.assertIsNone(
                    previous,
                    f'{tag} already names {previous} and also {path.name}: {referent}',
                )
                records[tag] = (path.name, referent)
        self.assertEqual(records['S12'], (
            'NOVICE_N14_READY_FOR_STANDARDS.md',
            'Joan Daemen and Vincent Rijmen.',
        ))
        self.assertEqual(records['T8'], (
            'NOVICE_N14_READY_FOR_STANDARDS.md',
            'Orange edition.',
        ))
        self.assertEqual(records['C2'], (
            'NOVICE_N14_READY_FOR_STANDARDS.md',
            'Gate surface.',
        ))
        self.assertEqual(records['C1'][0], 'NOVICE_PROBABILITY.md')
        self.assertNotIn('S13', records)
        self.assertNotIn('T9', records)
        self.assertNotIn('C3', records)


def math_gcd(left: int, right: int) -> int:
    while right:
        left, right = right, left % right
    return left


def positive_gcd(left: int, right: int) -> int:
    value = math_gcd(left, right)
    return value if value >= 0 else -value


def successor_bezout(integer: int, modulus: int) -> tuple[int, int]:
    """Coefficients from the one step in Proposition N11.8.

    Modulus 1 is Q(0). A larger modulus divides once, as §6.3 does, and
    applies the same step to the remainder.
    """
    if modulus < 1:
        raise ValueError('modulus must be positive')
    if modulus == 1:
        return 0, 1
    quotient, remainder = divmod(integer, modulus)
    if remainder == 0:
        return 0, 1
    inner_x, inner_y = successor_bezout(modulus, remainder)
    return inner_y, inner_x - quotient * inner_y


class ManuscriptManifest(unittest.TestCase):
    def test_manifest_names_drafted_and_planned_chapters(self):
        sys_path = str(ROOT / 'tools')
        if sys_path not in __import__('sys').path:
            __import__('sys').path.insert(0, sys_path)
        from render_book import PART_TITLES, load_manifest

        manifest = load_manifest(ROOT)
        self.assertEqual(manifest['status'], 'in-progress')
        self.assertEqual(manifest['manifest'], 'docs/book/manifest.json')
        drafted = [chapter for chapter in manifest['chapters'] if chapter['status'] == 'draft']
        planned = [chapter for chapter in manifest['chapters'] if chapter['status'] == 'planned']
        self.assertEqual(
            [chapter['part'] for chapter in drafted],
            ['novice'] * len(drafted),
        )
        self.assertEqual({chapter['part'] for chapter in planned}, {'journeyman', 'master'})
        self.assertTrue(all(chapter['review_state'] != 'reviewed' for chapter in drafted))
        self.assertIn('owner-approved-with-unreviewed-corrections', {
            chapter['review_state'] for chapter in drafted
        })
        self.assertIn('unreviewed', {chapter['review_state'] for chapter in drafted})
        index = INDEX.read_text(encoding='utf-8')
        self.assertIn('## Manuscript status', index)
        self.assertIn('[manifest.json](manifest.json)', index)
        self.assertIn('living, in-progress manuscript', index)
        for part in ('Part 1, The Novice', 'Part 2, The Journeyman', 'Part 3, The Master'):
            self.assertIn(part, index)
            self.assertEqual(PART_TITLES[{
                'Part 1, The Novice': 'novice',
                'Part 2, The Journeyman': 'journeyman',
                'Part 3, The Master': 'master',
            }[part]], part)
        for chapter in drafted:
            short = chapter['title'].split('. ', 1)[-1].split(': ', 1)[-1]
            self.assertIn(short, index)
        self.assertIn('| Part 2, The Journeyman | None |', index.replace('\n', ' '))
        self.assertIn('| Part 3, The Master | None |', index.replace('\n', ' '))
        for name in ('NOVICE_PROGRAMMING.md', 'NOVICE_LOGIC.md', 'NOVICE_PROTECT.md'):
            text = (ROOT / 'docs' / 'book' / name).read_text(encoding='utf-8')
            self.assertIn('S3u', text)
            self.assertNotIn('S3t', text)

    def test_rendered_book_shows_planned_chapters_without_pages(self):
        import shutil
        sys_path = str(ROOT / 'tools')
        if sys_path not in __import__('sys').path:
            __import__('sys').path.insert(0, sys_path)
        from render_book import render

        output = ROOT / 'build' / 'book'
        try:
            self.assertEqual(render(ROOT), output.resolve())
            index = (output / 'index.html').read_text(encoding='utf-8')
            self.assertIn('Living, in-progress manuscript', index)
            self.assertIn('J2', index)
            self.assertIn('planned', index)
            self.assertIn('Auditable claim dossier', index)
            self.assertTrue((output / 'docs' / 'book' / 'NOVICE_OPENING.html').is_file())
            self.assertTrue((output / 'docs' / 'THE_ORANGE_BOOK.html').is_file())
            self.assertFalse((output / 'docs' / 'book' / 'J2.html').exists())
            opening = (output / 'docs' / 'book' / 'NOVICE_OPENING.html').read_text(encoding='utf-8')
            self.assertIn('Draft.', opening)
            self.assertNotIn('<script', opening.lower())
            self.assertIn('&lt;', (output / 'docs' / 'book' / 'NOVICE_N8_READ_AND_REPAIR.html').read_text(encoding='utf-8')[:5000] or 'skip')
            from render_book import load_manifest, manuscript_files
            manifest = load_manifest(ROOT)
            for source in manuscript_files(manifest):
                page = output / source.replace('.md', '.html')
                self.assertTrue(page.is_file(), source)
                text = page.read_text(encoding='utf-8')
                self.assertTrue(text.strip(), source)
                self.assertIn('<article>', text)
                self.assertNotRegex(text, r'<article>\s*</article>')
            index_text = (output / 'index.html').read_text(encoding='utf-8')
            self.assertTrue(index_text.strip())
            self.assertNotRegex(index_text, r'<article>\s*</article>')
        finally:
            shutil.rmtree(ROOT / 'build', ignore_errors=True)

    def test_malformed_chapters_fail(self):
        sys_path = str(ROOT / 'tools')
        if sys_path not in __import__('sys').path:
            __import__('sys').path.insert(0, sys_path)
        from render_book import require_well_formed
        source = ROOT / 'docs' / 'book' / 'NOVICE_OPENING.md'
        samples = (
            '---\ntitle: draft\n---\n# Chapter\n',
            '# Chapter\n\n```\nnot closed\n',
            '# Chapter\n\n{% include missing-chapter.md %}\n',
        )
        for sample in samples:
            with self.assertRaises(ValueError):
                require_well_formed(sample, source, ROOT)

    def test_dead_links_fail(self):
        import tempfile
        sys_path = str(ROOT / 'tools')
        if sys_path not in __import__('sys').path:
            __import__('sys').path.insert(0, sys_path)
        from render_book import source_link_errors, written_href_errors
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            docs = root / 'docs'
            docs.mkdir()
            chapter = docs / 'chapter.md'
            chapter.write_text(
                '# Title\n\n[missing file](missing.md)\n[missing anchor](#absent)\n',
                encoding='utf-8',
            )
            errors = source_link_errors(root, [chapter])
            self.assertTrue(any('missing.md' in error for error in errors))
            self.assertTrue(any('#absent' in error for error in errors))
            sound = docs / 'sound.md'
            sound.write_text(
                '# Title\n\n[here](#title)\n[chapter](chapter.md#title)\n',
                encoding='utf-8',
            )
            self.assertEqual(source_link_errors(root, [sound]), [])
            output = root / 'build' / 'book'
            page = output / 'docs'
            page.mkdir(parents=True)
            (page / 'chapter.html').write_text(
                '<article><a href="missing.html">x</a>'
                '<a href="#absent">y</a></article>',
                encoding='utf-8',
            )
            (output / 'index.html').write_text('<article><p>index</p></article>', encoding='utf-8')
            href_errors = written_href_errors(
                output, root, {'docs/chapter.md': 'docs/chapter.html'}
            )
            self.assertTrue(any('missing.html' in error for error in href_errors))
            self.assertTrue(any('#absent' in error for error in href_errors))

    def test_hollow_output_fails(self):
        import tempfile
        sys_path = str(ROOT / 'tools')
        if sys_path not in __import__('sys').path:
            __import__('sys').path.insert(0, sys_path)
        from render_book import require_complete_output
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory)
            (output / 'docs').mkdir()
            (output / 'docs' / 'chapter.html').write_text(
                '<article></article>', encoding='utf-8'
            )
            (output / 'index.html').write_text('', encoding='utf-8')
            with self.assertRaises(ValueError):
                require_complete_output(output, ['docs/chapter.html'])

    def test_symlink_output_refuses_and_keeps_the_canary(self):
        import shutil
        import subprocess
        import sys
        import tempfile

        script = ROOT / 'tools' / 'render_book.py'
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            notes = root / 'notes'
            notes.mkdir()
            canary = notes / 'canary'
            canary.write_text('canary', encoding='utf-8')
            (root / 'build').mkdir()
            (root / 'build' / 'book').symlink_to('../notes')
            tools = root / 'tools'
            tools.mkdir()
            shutil.copy(script, tools / 'render_book.py')
            completed = subprocess.run(
                [
                    sys.executable, '-S', '-P', '-B', '-X', 'utf8',
                    '-W', 'error::ResourceWarning', 'tools/render_book.py',
                ],
                cwd=root,
                check=False,
                capture_output=True,
                text=True,
            )
            self.assertNotEqual(completed.returncode, 0, completed.stderr)
            self.assertEqual(canary.read_text(encoding='utf-8'), 'canary')
            self.assertTrue((root / 'build' / 'book').is_symlink())

        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            docs = root / 'docs'
            docs.mkdir()
            marker = docs / 'kept.md'
            marker.write_text('keep', encoding='utf-8')
            (root / 'build').mkdir()
            (root / 'build' / 'book').symlink_to('../docs')
            tools = root / 'tools'
            tools.mkdir()
            shutil.copy(script, tools / 'render_book.py')
            completed = subprocess.run(
                [
                    sys.executable, '-S', '-P', '-B', '-X', 'utf8',
                    '-W', 'error::ResourceWarning', 'tools/render_book.py',
                ],
                cwd=root,
                check=False,
                capture_output=True,
                text=True,
            )
            self.assertNotEqual(completed.returncode, 0, completed.stderr)
            self.assertEqual(marker.read_text(encoding='utf-8'), 'keep')
            self.assertTrue(docs.is_dir())
            self.assertFalse(docs.is_symlink())

    def test_render_rejects_malformed_dead_and_hollow_manuscripts(self):
        import json
        import shutil
        import subprocess
        import sys
        import tempfile

        def run_case(chapter: str) -> subprocess.CompletedProcess[str]:
            with tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                book = root / 'docs' / 'book'
                book.mkdir(parents=True)
                (root / 'docs' / 'THE_ORANGE_BOOK.md').write_text(
                    '# The Orange Book\n\nA sentence.\n',
                    encoding='utf-8',
                )
                (book / 'NOVICE_OPENING.md').write_text(chapter, encoding='utf-8')
                manifest = {
                    'kind': 'orange-book-manuscript-manifest',
                    'version': 1,
                    'status': 'in-progress',
                    'review': 'Draft.',
                    'chapters': [
                        {
                            'part': 'novice',
                            'id': 'opening',
                            'title': 'Opening',
                            'status': 'draft',
                            'review_state': 'unreviewed',
                            'path': 'docs/book/NOVICE_OPENING.md',
                        },
                        {
                            'part': 'original',
                            'id': 'original',
                            'title': 'The Orange Book',
                            'status': 'original',
                            'path': 'docs/THE_ORANGE_BOOK.md',
                        },
                    ],
                }
                (book / 'manifest.json').write_text(
                    json.dumps(manifest),
                    encoding='utf-8',
                )
                tools = root / 'tools'
                tools.mkdir()
                shutil.copy(ROOT / 'tools' / 'render_book.py', tools / 'render_book.py')
                return subprocess.run(
                    [
                        sys.executable, '-S', '-P', '-B', '-X', 'utf8',
                        '-W', 'error::ResourceWarning', 'tools/render_book.py',
                    ],
                    cwd=root,
                    check=False,
                    capture_output=True,
                    text=True,
                )

        malformed = run_case('---\ntitle: draft\n---\n# Chapter\n')
        self.assertNotEqual(malformed.returncode, 0, malformed.stderr)
        self.assertIn('malformed chapter', malformed.stderr)
        dead = run_case('# Title\n\n[missing file](missing.md)\n')
        self.assertNotEqual(dead.returncode, 0, dead.stderr)
        self.assertIn('missing.md', dead.stderr)
        hollow = run_case('\n')
        self.assertNotEqual(hollow.returncode, 0, hollow.stderr)
        self.assertIn('hollow', hollow.stderr)

    def test_renderer_cli_writes_the_index(self):
        import shutil
        import subprocess
        import sys

        output = ROOT / 'build' / 'book'
        script = str(ROOT / 'tools' / 'render_book.py')
        try:
            rejected = subprocess.run(
                [sys.executable, script, '/tmp/elsewhere'],
                cwd=ROOT,
                check=False,
                capture_output=True,
                text=True,
            )
            self.assertEqual(rejected.returncode, 2, rejected.stderr)
            self.assertFalse(output.exists())
            completed = subprocess.run(
                [sys.executable, script],
                cwd=ROOT,
                check=False,
                capture_output=True,
                text=True,
            )
            self.assertEqual(completed.returncode, 0, completed.stderr)
            index = (output / 'index.html').read_text(encoding='utf-8')
            self.assertIn('Living, in-progress manuscript', index)
            self.assertIn('planned', index)
        finally:
            shutil.rmtree(ROOT / 'build', ignore_errors=True)

    def test_table_escaped_pipes_keep_four_cells(self):
        import tempfile

        opening = (
            '# Operators\n\n'
            '| Expression | On `Int` | On `Word[n]` | On `Mod[m]` |\n'
            '| --- | --- | --- | --- |\n'
            '| `a & b`, `a \\| b`, `a ^ b` | Not defined | Bitwise and, or, exclusive or | Not defined |\n'
            '| `a % b` | Euclidean remainder, 0 ≤ `a % b` < \\|b\\| | Unsigned remainder | Not defined |\n'
            '| see `a|b` here | left | right | end |\n'
        )
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            _write_min_manuscript(root, opening, '# The Orange Book\n\nA sentence.\n')
            completed = _run_render(root)
            self.assertEqual(completed.returncode, 0, completed.stderr)
            page = (root / 'build' / 'book' / 'docs' / 'book' / 'NOVICE_OPENING.html').read_text(encoding='utf-8')
            rows = re.findall(r'<tr>(.*?)</tr>', page, re.S)
            self.assertEqual(len(rows), 4, page)
            for row in rows:
                cells = re.findall(r'<t[dh]>', row)
                self.assertEqual(len(cells), 4, row)
            self.assertIn('<code>a | b</code>', page)
            self.assertIn('|b|', page)
            self.assertIn('<code>a|b</code>', page)
            self.assertNotIn('\\|', page)

    def test_n14_outcomes_render_as_one_ordered_list(self):
        import tempfile

        source = (ROOT / 'docs' / 'book' / 'NOVICE_N14_READY_FOR_STANDARDS.md').read_text(encoding='utf-8')
        excerpt = '\n'.join(source.splitlines()[52:65])
        self.assertTrue(excerpt.startswith('1. You can walk one Orange program'))
        self.assertIn('are not ready for J2', excerpt)
        opening = '# Ready for standards\n\n' + excerpt + '\n'
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            _write_min_manuscript(root, opening, '# The Orange Book\n\nA sentence.\n')
            completed = _run_render(root)
            self.assertEqual(completed.returncode, 0, completed.stderr)
            page = (root / 'build' / 'book' / 'docs' / 'book' / 'NOVICE_OPENING.html').read_text(encoding='utf-8')
            article = re.search(r'<article>(.*)</article>', page, re.S)
            self.assertIsNotNone(article)
            body = article.group(1)
            lists = re.findall(r'<ol>.*?</ol>', body, re.S)
            self.assertEqual(len(lists), 1, body)
            items = re.findall(r'<li>.*?</li>', lists[0], re.S)
            self.assertEqual(len(items), 5)
            self.assertNotIn('<p>', lists[0])
            self.assertNotRegex(body, r'</ol>\s*<p>')
            self.assertNotRegex(body, r'</p>\s*<ol>')
            self.assertIn('actually contains.', lists[0])
            self.assertIn('arithmetic beside the Orange name.', lists[0])
            self.assertIn('verified.', lists[0])
            self.assertIn('non-claims.', lists[0])
            self.assertIn('are not ready for J2', lists[0])

    def test_non_manuscript_links_pin_to_the_rendered_commit(self):
        import os
        import tempfile

        sha = '0123456789abcdef0123456789abcdef01234567'
        opening = '# Opening\n\nA sentence.\n'
        original = (
            '# The Orange Book\n\n'
            '[notes](NOTES.md#section)\n\n'
            '[directory](book/)\n\n'
            '[readme](book/README.md)\n\n'
            '[status](book/README.md#manuscript-status)\n\n'
            '[chapter](book/NOVICE_OPENING.md#opening)\n'
        )
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            _write_min_manuscript(root, opening, original)
            (root / 'docs' / 'NOTES.md').write_text('# Notes\n\n## Section\n\nA note.\n', encoding='utf-8')
            (root / 'docs' / 'book' / 'README.md').write_text(
                '# Book index\n\n## Manuscript status\n\nRead me.\n',
                encoding='utf-8',
            )
            env = os.environ.copy()
            env['GITHUB_SHA'] = sha
            completed = _run_render(root, env)
            self.assertEqual(completed.returncode, 0, completed.stderr)
            page = (root / 'build' / 'book' / 'docs' / 'THE_ORANGE_BOOK.html').read_text(encoding='utf-8')
            base = f'https://github.com/chasebryan/orange'
            self.assertIn(f'{base}/blob/{sha}/docs/NOTES.md#section', page)
            self.assertIn(f'{base}/tree/{sha}/docs/book"', page)
            self.assertIn(f'{base}/blob/{sha}/docs/book/README.md"', page)
            self.assertIn(f'{base}/blob/{sha}/docs/book/README.md#manuscript-status', page)
            self.assertIn('href="book/NOVICE_OPENING.html#opening"', page)
            self.assertNotIn(f'{base}/blob/{sha}/docs/book/NOVICE_OPENING.md', page)
            self.assertNotIn('README.html', page)
            self.assertNotIn('NOTES.html', page)

        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            _write_min_manuscript(
                root,
                opening,
                '# The Orange Book\n\n[notes](NOTES.md)\n',
            )
            (root / 'docs' / 'NOTES.md').write_text('# Notes\n\nA note.\n', encoding='utf-8')
            env = os.environ.copy()
            env.pop('GITHUB_SHA', None)
            env.pop('GIT_DIR', None)
            env.pop('GIT_WORK_TREE', None)
            completed = _run_render(root, env)
            self.assertNotEqual(completed.returncode, 0, completed.stdout)
            self.assertIn('rendered commit is unavailable', completed.stderr)

    def test_dangling_local_href_fails_the_render(self):
        import tempfile

        opening = '# Opening\n\nA sentence.\n'
        original = '# The Orange Book\n\n[alias](book/alias.md)\n'
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            _write_min_manuscript(root, opening, original)
            alias = root / 'docs' / 'book' / 'alias.md'
            alias.symlink_to('NOVICE_OPENING.md')
            completed = _run_render(root)
            self.assertNotEqual(completed.returncode, 0, completed.stderr)
            self.assertIn('local href does not resolve to a file in the artifact', completed.stderr)
            self.assertIn('book/alias.html', completed.stderr)
            page = (root / 'build' / 'book' / 'docs' / 'THE_ORANGE_BOOK.html').read_text(encoding='utf-8')
            self.assertIn('href="book/alias.html"', page)

    def test_wrapped_number_stays_in_the_paragraph(self):
        import tempfile

        opening = '# Shift\n\nshift by 11 and\n54. Exclusive or\n'
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            _write_min_manuscript(root, opening, '# The Orange Book\n\nA sentence.\n')
            completed = _run_render(root)
            self.assertEqual(completed.returncode, 0, completed.stderr)
            page = (root / 'build' / 'book' / 'docs' / 'book' / 'NOVICE_OPENING.html').read_text(encoding='utf-8')
            self.assertIn('<p>shift by 11 and 54. Exclusive or</p>', page)
            self.assertNotIn('<ol', page)

    def test_ordered_list_after_blank_line_keeps_its_start(self):
        import tempfile

        opening = '# Count\n\nA paragraph.\n\n3. alpha\n4. beta\n'
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            _write_min_manuscript(root, opening, '# The Orange Book\n\nA sentence.\n')
            completed = _run_render(root)
            self.assertEqual(completed.returncode, 0, completed.stderr)
            page = (root / 'build' / 'book' / 'docs' / 'book' / 'NOVICE_OPENING.html').read_text(encoding='utf-8')
            self.assertIn('<p>A paragraph.</p>', page)
            self.assertIn('<ol start="3"><li>alpha</li><li>beta</li></ol>', page)

    def test_ordered_list_starting_at_one_interrupts_a_paragraph(self):
        import tempfile

        opening = '# Count\n\nA paragraph\n1. item\n'
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            _write_min_manuscript(root, opening, '# The Orange Book\n\nA sentence.\n')
            completed = _run_render(root)
            self.assertEqual(completed.returncode, 0, completed.stderr)
            page = (root / 'build' / 'book' / 'docs' / 'book' / 'NOVICE_OPENING.html').read_text(encoding='utf-8')
            self.assertIn('<p>A paragraph</p>', page)
            self.assertIn('<ol><li>item</li></ol>', page)
            self.assertNotIn('1. item', page)


def _write_min_manuscript(root: Path, opening: str, original: str) -> None:
    import json

    book = root / 'docs' / 'book'
    book.mkdir(parents=True)
    (root / 'docs' / 'THE_ORANGE_BOOK.md').write_text(original, encoding='utf-8')
    (book / 'NOVICE_OPENING.md').write_text(opening, encoding='utf-8')
    manifest = {
        'kind': 'orange-book-manuscript-manifest',
        'version': 1,
        'status': 'in-progress',
        'review': 'Draft.',
        'chapters': [
            {
                'part': 'novice',
                'id': 'opening',
                'title': 'Opening',
                'status': 'draft',
                'review_state': 'unreviewed',
                'path': 'docs/book/NOVICE_OPENING.md',
            },
            {
                'part': 'original',
                'id': 'original',
                'title': 'The Orange Book',
                'status': 'original',
                'path': 'docs/THE_ORANGE_BOOK.md',
            },
        ],
    }
    (book / 'manifest.json').write_text(json.dumps(manifest), encoding='utf-8')


def _run_render(root: Path, env: dict | None = None):
    import os
    import shutil
    import subprocess
    import sys

    tools = root / 'tools'
    tools.mkdir(exist_ok=True)
    shutil.copy(ROOT / 'tools' / 'render_book.py', tools / 'render_book.py')
    run_env = os.environ.copy()
    if env is not None:
        run_env = env
    return subprocess.run(
        [
            sys.executable, '-S', '-P', '-B', '-X', 'utf8',
            '-W', 'error::ResourceWarning', 'tools/render_book.py',
        ],
        cwd=root,
        check=False,
        capture_output=True,
        text=True,
        env=run_env,
    )


def rotate_byte(value: int, amount: int) -> int:
    """Reference mathematical rotation, not an Orange interpreter."""
    if not 0 <= value < 256:
        raise ValueError('value must fit a byte')
    amount %= 8
    return ((value << amount) | (value >> (8 - amount))) & 255


if __name__ == '__main__':
    unittest.main(verbosity=2)
