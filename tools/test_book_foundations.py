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
        self.assertNotIn('```orange', self.text)
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


def rotate_byte(value: int, amount: int) -> int:
    """Reference mathematical rotation, not an Orange interpreter."""
    if not 0 <= value < 256:
        raise ValueError('value must fit a byte')
    amount %= 8
    return ((value << amount) | (value >> (8 - amount))) & 255


if __name__ == '__main__':
    unittest.main(verbosity=2)
