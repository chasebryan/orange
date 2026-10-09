"""Discover book checks and independently audit the printed continuation results.

These are Python reference calculations, not execution of Orange. The Rust
book_novice integration test checks the actual Orange programs separately.
AI-assisted continuation audit: ChatGPT (GPT-6 Astra Pro), 2026-10-05.
"""
import importlib.util
from pathlib import Path
import re
import unittest

ROOT = Path(__file__).resolve().parents[2]


def rotate_byte(value: int, amount: int) -> int:
    """Independent eight-position rotation for the printed reference values."""
    amount %= 8
    return ((value << amount) | (value >> (8 - amount))) & 255


class PrintedContinuationAudit(unittest.TestCase):
    def test_all_23_printed_results_match_reference_calculations(self):
        text = (ROOT / 'docs/book/NOVICE_PROGRAMMING.md').read_text(encoding='utf-8')
        byte = lambda value: value % 256
        round_value = rotate_byte(byte(0xfa + 7) ^ 0x3c, 1)
        # Values are calculated here, not copied from the output transcript.
        cases = {
            'first_steps': [('answer', 8, 13)],
            'masks': [('example', 8, 0x0b ^ 0x06),
                      ('recovered', 8, (0x0b ^ 0x06) ^ 0x06)],
            'word_edges': [('wrapped', 8, byte(255 + 1)),
                           ('backward', 8, byte(0 - 1)),
                           ('product', 8, byte(200 * 2)),
                           ('integer_sum', None, 255 + 1),
                           ('wider_sum', 16, (255 + 1) % 65536)],
            'movement': [('shifted_left', 8, byte(0x81 << 1)),
                         ('shifted_right', 8, 0x81 >> 1),
                         ('rotated_left', 8, rotate_byte(0x81, 1)),
                         ('rotated_right', 8, rotate_byte(0x81, -1)),
                         ('restored', 8, rotate_byte(rotate_byte(0x81, 1), -1))],
            'boundary_moves': [('no_turn', 8, rotate_byte(0x81, 0)),
                               ('full_turn', 8, rotate_byte(0x81, 8)),
                               ('extra_turn', 8, rotate_byte(0x81, 9)),
                               ('lost_left', 8, byte(0x81 << 8)),
                               ('lost_right', 8, 0x81 >> 9),
                               ('reverse_direction', 8, rotate_byte(0x81, -1))],
            'grouping': [('add_first', 8, (1 + 1) ^ 1),
                         ('xor_first', 8, 1 + (1 ^ 1))],
            'small_round': [('example', 8, round_value),
                            ('recovered', 8,
                             byte((rotate_byte(round_value, -1) ^ 0x3c) - 7))],
        }
        blocks = re.findall(r'^```text\n(.*?)\n```', text, re.M | re.S)
        checked = 0
        for module, results in cases.items():
            lines = []
            for name, width, value in results:
                kind = 'Int' if width is None else f'Word[{width}]'
                number = str(value) if width is None else f'0x{value:0{width // 4}x}'
                lines.append(f'{module}::{name}: {kind} = {number}')
                checked += 1
            printed = [block for block in blocks if block.startswith(module + '::')]
            self.assertEqual(printed, ['\n'.join(lines)], module)
        self.assertEqual(checked, 23)

    def test_continuation_navigation_targets_exist(self):
        text = (ROOT / 'docs/book/NOVICE_PROGRAMMING.md').read_text(encoding='utf-8')
        index = (ROOT / 'docs/book/README.md').read_text(encoding='utf-8')
        headings = re.findall(r'^#{1,6} (.+)$', text, re.M)
        anchors = {re.sub(r'[^\w\- ]', '', h.lower()).replace(' ', '-')
                   for h in headings}
        targets = re.findall(r'NOVICE_PROGRAMMING\.md#([^\)]+)', index)
        self.assertGreaterEqual(len(set(targets)), 5)
        for target in targets:
            self.assertIn(target, anchors)


def load_tests(loader: unittest.TestLoader, tests: unittest.TestSuite,
               pattern: str | None) -> unittest.TestSuite:
    path = Path(__file__).resolve().parents[1] / 'test_book_foundations.py'
    spec = importlib.util.spec_from_file_location('orange_book_checks', path)
    if spec is None or spec.loader is None:
        raise ImportError(f'cannot load book checks: {path}')
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    suite = loader.loadTestsFromModule(module)
    suite.addTests(loader.loadTestsFromTestCase(PrintedContinuationAudit))
    return suite
