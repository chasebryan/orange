from __future__ import annotations

import copy
import json
import struct
import unittest
from pathlib import Path
from typing import Any

from tools import d011_suite as suite


REPOSITORY_ROOT = Path(__file__).resolve().parents[2]


def packet() -> dict[str, Any]:
    return json.loads((REPOSITORY_ROOT / suite.PACKET_PATH).read_text(encoding="utf-8"))


def libcrypto_available() -> bool:
    try:
        suite._libcrypto()
    except OSError:
        return False
    return True


class D011PacketTests(unittest.TestCase):
    def test_committed_packet_matches_the_generator_byte_for_byte(self) -> None:
        self.assertEqual(suite.check(REPOSITORY_ROOT), [])
        raw = (REPOSITORY_ROOT / suite.PACKET_PATH).read_bytes()
        self.assertEqual(raw, suite.canonical_file(json.loads(raw.decode("utf-8"))))

    def test_packet_binds_the_kernels_and_the_suite_document(self) -> None:
        rows = {row["path"]: row for row in packet()["inputs"]}
        self.assertEqual(list(rows), list(suite.BOUND_INPUTS))
        for path, row in rows.items():
            self.assertEqual(suite.file_sha256(REPOSITORY_ROOT / path), row["sha256"], path)

    def test_validation_rejects_mutations(self) -> None:
        base = packet()
        self.assertEqual(suite.validate_packet(base), [])
        mutations = {
            "owner rule": lambda p: p.__setitem__("owner_distinguishing_rule", "DR-1"),
            "owner review": lambda p: p["review_scopes"][0].__setitem__("recorded", True),
            "status": lambda p: p.__setitem__("status", "frozen"),
            "candidate dropped": lambda p: p["candidates"].pop(),
            "parity": lambda p: p["candidates"][0].__setitem__("extra", 1),
            "float": lambda p: p["subjects"][0].__setitem__("weight", 0.5),
            "arity": lambda p: p["subjects"][0]["args"].append("00"),
            "hex": lambda p: p["subjects"][0]["expect"].__setitem__("output", "zz"),
            "duplicate": lambda p: p["negatives"].append(copy.deepcopy(p["negatives"][0])),
            "inventory verified": lambda p: p["isa_abi_inventory"][0].__setitem__("status", "verified"),
            "no control": lambda p: p.__setitem__("trace_groups", [g for g in p["trace_groups"]
                                                                   if g["expect"] == "equal"]),
            "same envelope": lambda p: p["candidates"][1].__setitem__("native_tuples", ["T-X64", "T-A64"]),
        }
        for name, mutate in mutations.items():
            mutated = copy.deepcopy(base)
            mutate(mutated)
            self.assertNotEqual(suite.validate_packet(mutated), [], name)

    def test_candidates_are_symmetric_and_nested(self) -> None:
        cands = {c["id"]: c for c in packet()["candidates"]}
        self.assertEqual(len({tuple(sorted(c)) for c in cands.values()}), 1)
        env = {cid: set(c["native_tuples"]) for cid, c in cands.items()}
        self.assertTrue(env["TE-02"] < env["TE-01"] < env["TE-05"])
        self.assertTrue(env["TE-03"] < env["TE-01"])
        self.assertEqual(env["TE-04"], set())
        for cand in cands.values():
            self.assertEqual(cand["cases"], list(suite.CASE_IDS))
            self.assertEqual(cand["gates"], list(suite.GATE_IDS))
            self.assertEqual(cand["portable_path"], "C-PORTABLE")

    def test_every_case_names_an_acceptance_evidence_item_and_its_gates(self) -> None:
        items = {c["acceptance_evidence"] for c in suite.CASES}
        self.assertEqual(items, {"resource estimate per target", "ISA and ABI model availability",
                                 "owner-accessible hardware evidence", "flagship-corpus feasibility"})
        gated = {g for c in suite.CASES for g in c["gates"]}
        self.assertEqual(gated, set(suite.GATE_IDS))

    def test_the_owner_template_is_empty(self) -> None:
        template = suite.owner_template()
        self.assertEqual(suite.owner_input_errors(template), [])
        self.assertEqual(packet()["owner_input_template"], template)
        self.assertTrue(all(row["verdict"] == "" for row in template["isa_abi_verification"]))
        self.assertTrue(all(row["done"] is False for row in template["review_scopes"]))
        self.assertIsNone(template["distinguishing_rule"])
        self.assertEqual(template["hardware"], [])

    def test_kernel_sources_say_they_are_stand_ins(self) -> None:
        for name in suite.KERNEL_FILES:
            text = (REPOSITORY_ROOT / suite.KERNEL_DIR / name).read_text(encoding="utf-8")
            self.assertRegex(text[:1200], r"(?i)not (an )?Orange output|NOT Orange output|not Orange output", name)

    def test_target_sections_are_found_for_every_architecture(self) -> None:
        for name in ("accel.c", "runtime.c"):
            counts = suite.target_specific_lines(
                (REPOSITORY_ROOT / suite.KERNEL_DIR / name).read_text(encoding="utf-8"))
            self.assertTrue(all(counts[arch] > 0 for arch in ("x86_64", "aarch64", "riscv64")), (name, counts))


class D011OracleTests(unittest.TestCase):
    def test_subject_expectations_agree_with_the_library_oracles(self) -> None:
        if not libcrypto_available():
            self.skipTest("libcrypto.so.3 is not available")
        for subj in packet()["subjects"]:
            status, output = suite.library_compute(subj["op"], [bytes.fromhex(a) for a in subj["args"]])
            output = suite.apply_slices(output, subj["driver_slices"])
            self.assertEqual((status, output.hex()), (subj["expect"]["status"], subj["expect"]["output"]), subj["id"])

    def test_tampered_negatives_are_rejected_by_the_library(self) -> None:
        if not libcrypto_available():
            self.skipTest("libcrypto.so.3 is not available")
        for neg in packet()["negatives"]:
            if neg["kind"] == "request" and neg["expect"] == "rejected" and neg["op"] in suite.PORTABLE_OPS:
                status, output = suite.library_compute(neg["op"], [bytes.fromhex(a) for a in neg["args"]])
                self.assertEqual((status, output), ("rejected", b""), neg["id"])

    def test_sha256_probe_reference_matches_the_worked_example(self) -> None:
        block = bytes.fromhex("61626380" + "00" * 56 + "00000018")
        words = struct.unpack(">18I", suite._sha256_probe(block))
        self.assertEqual(words[:2], (0x61626380, 0x000F0000))
        self.assertEqual(words[2], 0x5D6AEBCD)
        self.assertEqual(words[10], 0x5A6AD9AD)

    def test_evaluation_output_parses_and_compares(self) -> None:
        text = ("m::x: Word[32]^2 = [0x61626380, 0x000f0000]\n"
                "m::x_expected: Word[32]^2 = [0x61626380, 0x000f0000]\n"
                "m::ok: Bool = true\n")
        values = suite.parse_eval(text)
        self.assertEqual(suite.value_bytes(*values["x"]), bytes.fromhex("61626380000f0000"))
        self.assertIs(suite.value_bytes(*values["ok"]), True)
        subject = {"expect": {"status": "ok", "output": "61626380000f0000"},
                   "orange": {"compute": ["x"], "literal": ["x_expected"], "slices": None, "status_spec": "ok"}}
        self.assertEqual(suite.orange_verdict(subject, values),
                         {"computed": "agree", "literal": "agree", "status": "agree"})
        subject["expect"]["output"] = "00"
        self.assertEqual(suite.orange_verdict(subject, values)["computed"], "conflict")
        self.assertEqual(suite.apply_slices(b"abcdef", [[0, 2], [4, 6]]), b"abef")
        with self.assertRaises(suite.SuiteError):
            suite.parse_eval("not an evaluation line")


class D011LaboratoryLogicTests(unittest.TestCase):
    def test_protocol_round_trip_and_truncation(self) -> None:
        request = suite.encode_request(1, [b"abc"])
        self.assertEqual(request, bytes([1, 1]) + struct.pack("<I", 3) + b"abc")
        stream = bytes([0]) + struct.pack("<I", 2) + b"hi" + bytes([1]) + struct.pack("<I", 0)
        self.assertEqual(suite.parse_responses(stream), ([("ok", b"hi"), ("rejected", b"")], True))
        self.assertEqual(suite.parse_responses(stream[:-1])[1], False)

    def test_instruction_classes(self) -> None:
        cases = [
            ("x86_64", "div", "esi", "A"), ("x86_64", "vsqrtsd", "xmm0, xmm0, xmm1", "A"),
            ("x86_64", "jne", "0x10 <f+0x10>", "B"), ("x86_64", "jmp", "rax", "C"),
            ("x86_64", "notrack jmp", "rax", "C"), ("x86_64", "jmp", "0x20 <f+0x20>", None),
            ("x86_64", "call", "0x0 <g>", None), ("x86_64", "imul", "eax, ecx", "D"),
            ("x86_64", "aesenc", "xmm0, xmm1", "E"), ("x86_64", "pclmulqdq", "xmm0, xmm1, 0x0", "E"),
            ("aarch64", "udiv", "w0, w1, w2", "A"), ("aarch64", "b.ne", "0x10", "B"),
            ("aarch64", "cbz", "x0, 0x10", "B"), ("aarch64", "blr", "x8", "C"), ("aarch64", "ret", "", None),
            ("aarch64", "umulh", "x0, x1, x2", "D"), ("aarch64", "aese", "v0.16b, v1.16b", "E"),
            ("aarch64", "pmull", "v0.1q, v1.1d, v2.1d", "E"),
            ("riscv64", "divuw", "a0, a0, a1", "A"), ("riscv64", "remu", "a0, a0, a1", "A"),
            ("riscv64", "bnez", "a0, 0x10", "B"), ("riscv64", "jalr", "a5", "C"), ("riscv64", "mulhu", "a0, a1, a2", "D"),
            ("riscv64", "aes64esm", "a0, a1, a2", "E"), ("riscv64", "clmulh", "a0, a1, a2", "E"),
            ("riscv64", "add", "a0, a1, a2", None),
        ]
        for arch, mnemonic, operands, expected in cases:
            self.assertEqual(suite.classify(arch, mnemonic, operands), expected, (arch, mnemonic))
        sequence = [("auipc", "ra, 0x0"), ("jalr", "ra <f+0x8>"), ("jalr", "a5"), ("auipc", "t1, 0x0"),
                    ("jr", "t1 <g>")]
        self.assertEqual(suite.classify_sequence("riscv64", sequence), [None, None, "C", None, None])

    def test_objdump_and_trace_parsing(self) -> None:
        text = ("x.o:\tfile format elf64-x86-64\n\nDisassembly of section .text:\n\n<f>:\n"
                "               \tendbr64\n               \tdiv\tesi\n               \tret\n\n<g>:\n"
                "               \tjmp\trax\n")
        self.assertEqual(suite.parse_objdump(text), {"f": [("endbr64", ""), ("div", "esi"), ("ret", "")],
                                                     "g": [("jmp", "rax")]})
        line = b"Trace 0: 0x7f00 [00000000/0000000000401000/00000000/ff200000] d011_start"
        self.assertEqual(suite.trace_pc(line), b"0000000000401000")
        self.assertIsNone(suite.trace_pc(b"qemu: uncaught target signal 4"))

    def test_gate_precedence_and_vacuity(self) -> None:
        self.assertEqual(suite.combine(["pass", "unresolved", "fail"]), "fail")
        self.assertEqual(suite.combine(["pass", "unsupported", "fail"]), "unsupported")
        self.assertEqual(suite.combine([]), "unresolved")
        passing = {g: {"state": "pass", "vacuous": False} for g in suite.GATE_IDS}
        portable = {g: {"state": "pass", "vacuous": g in suite.PORTABLE_VACUOUS} for g in suite.GATE_IDS}
        members = {"C-PORTABLE": portable, "T-X64": dict(passing, **{"HG-07": {"state": "unresolved"}})}
        empty = suite.candidate_gates(members, [])
        self.assertTrue(all(v["state"] == "pass" for v in empty.values()))
        self.assertTrue(empty["HG-04"]["vacuous"])
        one = suite.candidate_gates(members, ["T-X64"])
        self.assertEqual(one["HG-07"], {"state": "unresolved", "vacuous": False})
        self.assertFalse(one["HG-04"]["vacuous"])

    def test_conclusions_need_a_measured_epoch_owner_input_and_a_rule(self) -> None:
        axes = {"TE-02": {"AX-01": 1, "AX-02": 12, "AX-03": 26, "AX-04": 20000, "AX-05": 130, "AX-06": 3, "AX-07": 1},
                "TE-01": {"AX-01": 2, "AX-02": 17, "AX-03": 52, "AX-04": 40000, "AX-05": 250, "AX-06": 6, "AX-07": 2},
                "TE-04": {"AX-01": 0, "AX-02": 5, "AX-03": 0, "AX-04": 3000, "AX-05": 0, "AX-06": 3, "AX-07": 0}}
        cands = {c: {"eligible": True, "axes": a} for c, a in axes.items()}
        content = {"schema": suite.OWNER_SCHEMA}
        owner = suite._owner_view(None)
        self.assertEqual(suite.conclude(cands, "dev", None, owner)["result"], "inconclusive")
        self.assertEqual(suite.conclude(cands, "measured", None, owner)["result"], "inconclusive")
        done = {"review_scopes": [{"scope": s[0], "done": True} for s in suite.REVIEW_SCOPES]}
        for rule, capacity, expected in (
                (None, None, "inconclusive"), ("DR-1", 1, "recommend_te_02"), ("DR-1", 2, "recommend_te_01"),
                ("DR-2", None, "recommend_te_04"), ("DR-3", None, "recommend_te_04"), ("DR-4", None, "inconclusive")):
            view = suite._owner_view(dict(content, **done, distinguishing_rule=rule, solo_slice_capacity=capacity))
            result = suite.conclude(cands, "measured", content, view)["result"]
            self.assertEqual(result, expected, (rule, capacity))
        partial = suite._owner_view(dict(content, review_scopes=[{"scope": "NR-01", "done": True}]))
        self.assertIn("NR-02", suite.conclude(cands, "measured", content, partial)["reason"])
        only = {"TE-02": {"eligible": True, "axes": axes["TE-02"]}, "TE-01": {"eligible": False, "axes": axes["TE-01"]}}
        view = suite._owner_view(dict(content, **done))
        self.assertEqual(suite.conclude(only, "measured", content, view)["result"], "recommend_te_02")

    def test_epoch_names_and_derived_variants_are_deterministic(self) -> None:
        identity = {"a": 1, "b": [2, 3]}
        self.assertEqual(suite.epoch_name(identity), suite.epoch_name({"b": [2, 3], "a": 1}))
        self.assertRegex(suite.epoch_name(identity), r"^d011-e-[0-9a-f]{20}$")
        self.assertEqual(suite.derive("TG-X|v0|a0", 70), suite.derive("TG-X|v0|a0", 70))
        self.assertEqual(len(suite.derive("TG-X|v0|a0", 70)), 70)
        self.assertNotEqual(suite.derive("TG-X|v0|a0", 32), suite.derive("TG-X|v1|a0", 32))

    def test_corruption_flips_one_bit_of_the_first_round_constant(self) -> None:
        image = b"\x00" * 7 + suite.K256_PREFIX + b"\x01"
        mutated = suite.corrupt_constants(image)
        self.assertIsNotNone(mutated)
        assert mutated is not None
        self.assertEqual(sum(bin(a ^ b).count("1") for a, b in zip(image, mutated)), 1)
        self.assertIsNone(suite.corrupt_constants(b"no constants here"))
        self.assertEqual(suite.symbol_violations(["memcpy", "d011_wipe", "printf"]), ["printf"])

    def test_owner_input_errors(self) -> None:
        good = suite.owner_template()
        self.assertEqual(suite.owner_input_errors(good), [])
        self.assertEqual(suite.owner_input_errors({"schema": "other"}), ["schema"])
        bad = dict(good, distinguishing_rule="DR-9", solo_slice_capacity=-1,
                   native_runs=[{"tuple": "T-X64", "driver_sha256": "x", "stdout_sha256": "y"}])
        self.assertEqual(set(suite.owner_input_errors(bad)),
                         {"distinguishing_rule", "solo_slice_capacity", "native_runs digests"})


if __name__ == "__main__":
    unittest.main()
