from __future__ import annotations

import itertools
import json
import random
import unittest
from pathlib import Path
from typing import Any

from tools import d006_shared as shared


REPOSITORY_ROOT = Path(__file__).resolve().parents[2]
LAB = REPOSITORY_ROOT / shared.LAB
SHARED = LAB / "shared-inputs"


def load(name: str) -> Any:
    return json.loads((SHARED / name).read_text(encoding="utf-8"))


class D006SharedInputTests(unittest.TestCase):
    def test_committed_laboratory_matches_the_reference_byte_for_byte(self) -> None:
        golden = (SHARED / shared.GOLDEN).read_text(encoding="ascii")
        expected = shared.build(golden, (SHARED / shared.SEMANTICS).read_bytes())
        present = {path.relative_to(LAB).as_posix() for path in LAB.rglob("*") if path.is_file()}
        self.assertEqual(set(expected), present)
        for name, data in expected.items():
            self.assertEqual((LAB / name).read_bytes(), data, name)

    def test_manifest_binds_every_shared_file_and_the_overlay_binds_the_manifest(self) -> None:
        manifest = load("manifest.json")
        listed = {entry["name"] for entry in manifest["files"]}
        on_disk = {path.name for path in SHARED.iterdir() if path.name != "manifest.json"}
        self.assertEqual(listed, on_disk)
        for entry in manifest["files"]:
            data = (SHARED / entry["name"]).read_bytes()
            self.assertEqual(entry["sha256"], shared.sha256(data))
            self.assertEqual(entry["bytes"], len(data))
        digest = shared.sha256(shared.canonical({"cases": manifest["cases"], "files": manifest["files"]}))
        self.assertEqual(manifest["input_manifest_sha256"], digest)
        overlay = json.loads((LAB / "protocol/suite-overlay.json").read_text(encoding="utf-8"))
        self.assertEqual(overlay["shared_inputs"]["input_manifest_sha256"], digest)
        self.assertEqual(overlay["status"], "prerequisites_draft")

    def test_overlay_names_every_v02_gap_as_closed_or_remaining(self) -> None:
        packet = json.loads((REPOSITORY_ROOT / shared.BASE["packet"]["path"]).read_text(encoding="utf-8"))
        overlay = json.loads((LAB / "protocol/suite-overlay.json").read_text(encoding="utf-8"))
        closed = {row["gap"] for row in overlay["closes"]}
        remaining = " | ".join(overlay["remaining"])
        for gap in packet["protocol_gaps"]:
            self.assertTrue(gap in closed or gap in remaining, gap)
        self.assertEqual(shared.sha256(shared.canonical(packet)), shared.BASE["packet"]["canonical_sha256"])
        for key in ("suite", "index"):
            raw = (REPOSITORY_ROOT / shared.BASE[key]["path"]).read_bytes()
            self.assertEqual(shared.sha256(raw), shared.BASE[key]["sha256"])


class D006CoreReferenceTests(unittest.TestCase):
    def test_rfc8439_quarter_round_and_word_operations(self) -> None:
        self.assertEqual(shared.quarter_round(32, (16, 12, 8, 7), [0x11111111, 0x01020304, 0x9B8D6F43, 0x01234567]), [0xEA2A92F4, 0xCB1CF8CE, 0x4581472E, 0x5881C4BB])
        self.assertEqual(shared.w_rotl(32, 0x12345678, 8), 0x34567812)
        self.assertEqual(shared.w_rotl(32, 0x80000001, 33), 3)
        self.assertEqual(shared.w_shl(8, 0x81, 1), 2)

    def test_core_theorems_hold_on_small_widths(self) -> None:
        for width in range(0, 6):
            for x, r, s in itertools.product(range(1 << width), range(8), range(8)):
                self.assertEqual(shared.w_rotl(width, shared.w_rotl(width, x, r), s), shared.w_rotl(width, x, r + s))
        for value in (0, 1, 0xDEADBEEF, 0xFFFFFFFF):
            self.assertEqual(shared.be32_of_bytes(shared.bytes_of_be32(value)), value)
            self.assertEqual(shared.le32_of_bytes(shared.bytes_of_le32(value)), value)
        generator = random.Random(6)
        for _ in range(200):
            words = [generator.getrandbits(32) for _ in range(generator.randrange(17))]
            self.assertEqual(shared.decode_words(shared.encode_words(words)), ("ok", words))
            data = [generator.randrange(256) for _ in range(generator.randrange(12))]
            state, value = shared.decode_words(data)
            if state == "ok":
                self.assertEqual(shared.encode_words(value), data)

    def test_negative_obligations_are_false_in_the_reference(self) -> None:
        self.assertNotEqual(shared.be32_of_bytes([0xDE, 0xAD, 0xBE, 0xEF]), shared.le32_of_bytes([0xDE, 0xAD, 0xBE, 0xEF]))
        self.assertEqual(shared.decode_words([1, 0, 0, 0]), ("err", "C-02"))
        self.assertNotEqual(shared.w_add(32, 1, 1), 3)


def random_expr(generator: random.Random, depth: int) -> tuple[Any, ...]:
    choice = generator.randrange(7 if depth else 2)
    if choice == 0:
        return shared.lit_w(generator.randrange(256)) if generator.random() < 0.8 else ("S-C07", shared.vbool(generator.random() < 0.5))
    if choice == 1:
        return shared.ev(generator.randrange(5))
    if choice == 6:
        return ("S-C13", generator.randrange(3), random_expr(generator, depth - 1))
    kind = ("S-C09", "S-C10", "S-C11", "S-C12")[choice - 2]
    return (kind, random_expr(generator, depth - 1), random_expr(generator, depth - 1))


def random_command(generator: random.Random, depth: int) -> tuple[Any, ...]:
    choice = generator.randrange(7 if depth else 4)
    if choice == 0:
        return shared.SKIP
    if choice == 1:
        return ("S-C15", generator.randrange(5), random_expr(generator, 2))
    if choice == 2:
        return ("S-C16", generator.randrange(3), random_expr(generator, 2), random_expr(generator, 2))
    if choice == 3:
        return ("S-C20", generator.randrange(5), random_expr(generator, 2))
    if choice == 4:
        return ("S-C17", random_command(generator, depth - 1), random_command(generator, depth - 1))
    if choice == 5:
        return ("S-C18", random_expr(generator, 2), random_command(generator, depth - 1), random_command(generator, depth - 1))
    return ("S-C19", generator.randrange(3), random_command(generator, depth - 1))


def random_state(generator: random.Random, env: tuple[Any, ...]) -> tuple[Any, ...]:
    variables = [shared.vword(generator.randrange(256)) if ty == shared.TY_WORD else shared.vbool(generator.random() < 0.5) for ty, _ in env[0]]
    arrays = [[generator.randrange(256) for _ in range(size)] for size, _ in env[1]]
    return (variables, arrays)


def low_twin(generator: random.Random, env: tuple[Any, ...], state: tuple[Any, ...]) -> tuple[Any, ...]:
    other = random_state(generator, env)
    variables = [value if label == shared.PUBLIC else other[0][index] for index, (value, (_, label)) in enumerate(zip(state[0], env[0]))]
    arrays = [cells if label == shared.PUBLIC else other[1][index] for index, (cells, (_, label)) in enumerate(zip(state[1], env[1]))]
    return (variables, arrays)


def low_eq(env: tuple[Any, ...], left: tuple[Any, ...], right: tuple[Any, ...]) -> bool:
    return all(label == shared.SECRET or left[0][i] == right[0][i] for i, (_, label) in enumerate(env[0])) and all(label == shared.SECRET or left[1][i] == right[1][i] for i, (_, label) in enumerate(env[1]))


def wf(env: tuple[Any, ...], state: tuple[Any, ...]) -> bool:
    return (
        len(state[0]) == len(env[0])
        and all(shared.val_ty(value) == ty for value, (ty, _) in zip(state[0], env[0]))
        and len(state[1]) == len(env[1])
        and all(len(cells) == size for cells, (size, _) in zip(state[1], env[1]))
    )


class D006SieveReferenceTests(unittest.TestCase):
    """The DS-02 theorems hold in the reference on many generated programs."""

    ENVS = [
        ([(shared.TY_WORD, shared.SECRET), (shared.TY_WORD, shared.PUBLIC), (shared.TY_WORD, shared.SECRET), (shared.TY_BOOL, shared.PUBLIC), (shared.TY_BOOL, shared.SECRET)], [(4, shared.PUBLIC), (4, shared.SECRET), (2, shared.PUBLIC)]),
        ([(shared.TY_WORD, shared.PUBLIC), (shared.TY_WORD, shared.PUBLIC), (shared.TY_WORD, shared.SECRET), (shared.TY_BOOL, shared.PUBLIC), (shared.TY_WORD, shared.PUBLIC)], [(3, shared.SECRET), (1, shared.PUBLIC), (5, shared.SECRET)]),
    ]

    def test_progress_preservation_and_lock_step_noninterference(self) -> None:
        generator = random.Random(2)
        typed = 0
        for _ in range(6000):
            env = self.ENVS[generator.randrange(len(self.ENVS))]
            command = random_command(generator, 3)
            if not shared.well_typed(env, command):
                continue
            typed += 1
            left = random_state(generator, env)
            right = low_twin(generator, env, left)
            for _ in range(40):
                first, second = shared.sieve_step(left, command), shared.sieve_step(right, command)
                self.assertNotEqual(first[0], "S-C33")
                self.assertEqual(first[0], second[0])
                if first[0] == "S-C30":
                    break
                if first[0] == "S-C32":
                    self.assertEqual(first[1], second[1])
                    break
                self.assertTrue(shared.well_typed(env, first[1]))
                self.assertTrue(wf(env, first[2]) and wf(env, second[2]))
                self.assertEqual(first[1], second[1])
                if first[3] != second[3]:
                    self.assertEqual(first[3][:-1], second[3][:-1])
                    self.assertEqual(first[3][-1][0], "S-C24")
                    self.assertNotEqual(first[3][-1], second[3][-1])
                    break
                self.assertTrue(low_eq(env, first[2], second[2]))
                command, left, right = first[1], first[2], second[2]
        self.assertGreater(typed, 500)

    def test_witness_programs(self) -> None:
        self.assertTrue(shared.well_typed(shared.GAMMA, shared.P_PLUS))
        self.assertFalse(shared.well_typed(shared.GAMMA, shared.P_MINUS))
        (_, first), (_, second) = (shared.sieve_run(8, sigma, shared.P_MINUS) for sigma in (shared.SIGMA_A, shared.SIGMA_B))
        self.assertEqual(first, [("S-C23", True)])
        self.assertEqual(second, [("S-C23", False)])
        self.assertTrue(low_eq(shared.GAMMA, shared.SIGMA_A, shared.SIGMA_B))
        (_, plus_a), (_, plus_b) = (shared.sieve_run(shared.RUN_FUEL, sigma, shared.P_PLUS) for sigma in (shared.SIGMA_A, shared.SIGMA_B))
        self.assertEqual(plus_a[:-1], plus_b[:-1])
        self.assertNotEqual(plus_a[-1], plus_b[-1])
        self.assertEqual(plus_a[-1][0], "S-C24")


class D006RecordReferenceTests(unittest.TestCase):
    def test_accepted_fixtures_are_canonical_and_valid(self) -> None:
        accepted = 0
        for ident, data, _ in shared.record_fixtures():
            try:
                value = shared.decode_records(data)
            except shared.RecordError:
                continue
            accepted += 1
            self.assertEqual(shared.encode_records(value), data, ident)
            self.assertTrue(shared.records_valid(value), ident)
        self.assertEqual(accepted, 4)

    def test_every_error_code_is_exercised(self) -> None:
        seen = set()
        for _, data, _ in shared.record_fixtures():
            try:
                shared.decode_records(data)
            except shared.RecordError as error:
                seen.add(error.code)
        self.assertEqual(seen, {name for _, name in shared.RECORD_ERRORS})

    def test_utf8_table_agrees_with_python_on_short_sequences(self) -> None:
        for length in (1, 2, 3):
            for data in itertools.product(range(0x7E, 0x100) if length > 1 else range(256), repeat=length):
                raw = bytes(data)
                try:
                    raw.decode("utf-8")
                    expected = True
                except UnicodeDecodeError:
                    expected = False
                self.assertEqual(shared.utf8_valid(raw), expected, raw.hex())


class D006CertificateReferenceTests(unittest.TestCase):
    def test_blast_is_sound_on_samples_and_matches_the_committed_cnf(self) -> None:
        self.assertEqual((SHARED / "ds04-carry-save.cnf").read_text(encoding="ascii"), shared.cnf_text("B-C01"))
        variables, clauses = shared.blast("B-C01")
        self.assertEqual((variables, len(clauses)), (512, 1535))
        generator = random.Random(4)
        for _ in range(300):
            x, y = generator.getrandbits(32), generator.getrandbits(32)
            terms = shared.obligation_terms()
            self.assertEqual(shared.bv_eval(terms["B-C01"]["lhs"], x, y), shared.bv_eval(terms["B-C01"]["rhs"], x, y))

    def test_golden_certificate_and_recorded_variant_verdicts(self) -> None:
        golden = (SHARED / shared.GOLDEN).read_text(encoding="ascii")
        self.assertEqual(shared.lrat_verdict("B-C01", shared.cnf_text("B-C01"), golden), ("accept",))
        recorded = load("ds04-lrat-obligation.json")["golden"]["variants"]
        self.assertEqual([row["id"] for row in recorded], [ident for ident, _ in shared.VARIANTS])
        for row in recorded:
            claimed, cnf, text = shared.mutate_certificate(golden, row["id"])
            self.assertEqual(list(shared.lrat_verdict(claimed, cnf, text)), row["verdict"], row["id"])
        self.assertEqual({row["verdict"][0] for row in recorded[1:]}, {"reject"})

    def test_checker_rejects_malformed_lines(self) -> None:
        cnf = shared.cnf_text("B-C01")
        for text, code in (
            ("1536 0 1 0\n", "rup"),
            ("1536  0 1 0\n", "parse"),
            ("01536 0 1 0\n", "parse"),
            ("1536 -0 0 1 0\n", "parse"),
            ("1 0 1 0\n", "id_order"),
            ("1536 600 0 1 0\n", "var_range"),
            ("1536 5 5 0 1 0\n", "lemma_form"),
            ("1536 5 -5 0 1 0\n", "lemma_form"),
            ("1536 d 99999 0\n", "unknown_deletion"),
            ("1536 d 1 0\n1537 1 0 1 0\n", "unknown_hint"),
        ):
            self.assertEqual(shared.lrat_verdict("B-C01", cnf, text)[1], code, text)


if __name__ == "__main__":
    unittest.main()
