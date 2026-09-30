from __future__ import annotations

import json
import shutil
import tempfile
import unittest
from pathlib import Path

from tools import d006_run as run
from tools import d006_shared as shared


REPOSITORY_ROOT = Path(__file__).resolve().parents[2]
OVERLAY = json.loads((REPOSITORY_ROOT / run.OVERLAY_PATH).read_text(encoding="utf-8"))


def amendment(identifier: str) -> str:
    return next(row["rule"] for row in OVERLAY["amendments"] if row["id"] == identifier)


class D006RunnerContractTests(unittest.TestCase):
    """The runner's constants are the overlay's, so the epoch it binds runs the protocol it names."""

    def test_ceilings_cpu_sets_and_isolation_match_the_overlay(self) -> None:
        self.assertEqual(run.CEILINGS, OVERLAY["execution"]["ceilings"])
        self.assertEqual(run.CPUS["serial"], (0,))
        self.assertEqual(len(run.CPUS["declared_parallel"]), 4)
        rule = amendment("AM-05")
        self.assertIn(f"uid {run.LAB_UID}", rule)
        self.assertEqual(run.SANDBOX_CAPS["address_space_bytes"], 4 << 30)
        self.assertEqual(run.SANDBOX_CAPS["cpu_seconds"], 600)
        self.assertEqual(run.SANDBOX_CAPS["file_bytes"], 512 << 20)
        for text in ("4 GiB of address space", "600 CPU seconds", "512 MiB per file", "1024 open files", "256 processes", "no core files"):
            self.assertIn(text, rule)
        self.assertEqual((run.SANDBOX_CAPS["open_files"], run.SANDBOX_CAPS["processes"], run.SANDBOX_CAPS["core_bytes"]), (1024, 256, 0))
        self.assertIn("--net", run.NAMESPACE)
        self.assertIn("--no-new-privs", run.PRIVILEGES)

    def test_the_sandbox_execs_the_command_itself(self) -> None:
        # env clears the environment before the sandbox; the sandbox's execv never retries a
        # command the host cannot execute as a shell script (DS-05 negative D5-N01).
        argv = run.Launcher(Path("/lab/fs-sandbox"), {}).argv(["/opt/tool", "x"], {"LANG": "C"}, run.Sandbox([], ["/run"]))
        sandbox = argv.index("/lab/fs-sandbox")
        self.assertEqual(argv[sandbox - 2:sandbox], ["-i", "LANG=C"])
        self.assertEqual(argv[sandbox - 3], "/usr/bin/env")
        self.assertEqual(argv[argv.index("--", sandbox):], ["--", "/opt/tool", "x"])

    def test_the_default_plan_is_the_preregistered_protocol(self) -> None:
        plan = run.Plan()
        self.assertEqual((plan.cold_runs, plan.replay_runs, plan.timed_pairs), (5, 3, 30))
        self.assertEqual(plan.timed_cases, ("DS-01", "DS-02", "DS-03", "DS-04", "DS-05", "DS-06"))
        self.assertEqual(plan.faults, tuple(f"D6-F{i:02d}" for i in range(1, 10)))
        self.assertTrue(plan.second_workspace and plan.negatives and plan.standalone)
        self.assertEqual([plan.order(n) for n in (1, 2, 3)], [("C-01", "C-02"), ("C-02", "C-01"), ("C-01", "C-02")])
        self.assertNotIn("revision", run.full_plan())
        self.assertEqual(OVERLAY["correction_window"]["rounds_per_candidate"], 1)

    def test_the_epoch_binds_the_overlay_the_shared_manifest_and_every_tool(self) -> None:
        bound = [row["path"] for row in run.bound_inputs(REPOSITORY_ROOT)]
        self.assertIn(run.OVERLAY_PATH, bound)
        self.assertIn(f"{run.SHARED_DIR}/manifest.json", bound)
        for tool in ("tools/d006_run.py", "tools/d006_check.py", "tools/d006_render.py", "tools/d006_shared.py", "tools/fs_sandbox.c"):
            self.assertIn(tool, bound)
        identity = {"suite_version": run.SUITE_VERSION, "bindings": run.bound_inputs(REPOSITORY_ROOT)}
        self.assertRegex(run.epoch_name(identity), r"^d006-e-[0-9a-f]{20}$")
        self.assertEqual(run.bootstrap_seed(identity), int(run.digest(identity)[:16], 16))
        changed = dict(identity, suite_version="other")
        self.assertNotEqual(run.epoch_name(identity), run.epoch_name(changed))

    def test_candidate_sources_leave_out_build_products(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            for name in ("theories/Core.v", "theories/Core.vo", "theories/.Core.aux", ".lake/build/x.olean", "adapter.json"):
                (root / name).parent.mkdir(parents=True, exist_ok=True)
                (root / name).write_text(name)
            self.assertEqual([path for path, _ in run.source_rows(root)], ["adapter.json", "theories/Core.v"])


class D006StatisticsTests(unittest.TestCase):
    def test_order_statistics(self) -> None:
        self.assertEqual(run.median([3, 1, 2]), 2.0)
        self.assertEqual(run.p95(list(range(1, 21))), 19.0)
        self.assertEqual(run.mad([1, 2, 3, 4, 100]), 1.0)
        self.assertIsNone(run.bootstrap_ci([5.0], 1))

    def test_bootstrap_is_seeded_and_brackets_the_median(self) -> None:
        values = [float(v) for v in (10, 12, 11, 13, 9, 10, 14, 12)]
        first, second = run.bootstrap_ci(values, 7), run.bootstrap_ci(values, 7)
        self.assertEqual(first, second)
        self.assertLessEqual(first[0], run.median(values))
        self.assertGreaterEqual(first[1], run.median(values))
        pairs = [(a, a * 2) for a in values]
        ratio = run.paired_ratio(pairs, 7)
        self.assertAlmostEqual(ratio["r"], 0.5)
        self.assertEqual(ratio, run.paired_ratio(pairs, 7))
        self.assertIsNone(run.paired_ratio([(1.0, 0.0), (1.0, 1.0)], 7)["r"])

    def test_materiality_bands_follow_the_overlay_order(self) -> None:
        self.assertEqual(run.band_label("interval", {"r": 0.8, "lo": 0.7, "hi": 0.9}), "rocq_better")
        self.assertEqual(run.band_label("interval", {"r": 0.95, "lo": 0.8, "hi": 0.99}), "inconclusive")
        self.assertEqual(run.band_label("interval", {"r": 1.2, "lo": 1.1, "hi": 1.3}), "lean_better")
        self.assertEqual(run.band_label("interval", {"r": 1.0, "lo": 0.95, "hi": 1.05}), "practically_equivalent")
        self.assertEqual(run.band_label("interval", {"r": None, "lo": None, "hi": None}), "inconclusive")
        self.assertEqual(run.band_label("exact", {"r": 0.9}), "rocq_better")
        self.assertEqual(run.band_label("exact", {"r": 1 / 0.9}), "lean_better")
        self.assertEqual(run.band_label("exact", {"r": 1.0}), "practically_equivalent")
        self.assertEqual(run.band_label("owner", {"r": 0.8}), "practically_equivalent")
        self.assertEqual(run.band_label("owner", {"r": 0.75}), "rocq_better")
        rows = run.comparative_rows({"M-07": {"label": "inconclusive"}, "M-08": {"DS-01": {"label": "rocq_better"}, "DS-02": {"label": "lean_better"}}})
        self.assertEqual([row["label"] for row in rows], ["inconclusive", "rocq_better", "lean_better"])


class D006FaultVerdictTests(unittest.TestCase):
    """The DS-06 fault rules: a stale or partial result is never success."""

    DONE = {"completed": True, "steps": 2}
    STOPPED = {"completed": False, "steps": 1}

    def verdict(self, fault: str, build: dict, kind: str | None, projection=None, reference=None, evidence=None, output: bytes = b"") -> bool:
        last = {"kind": kind} if kind else None
        return run.fault_verdict(fault, build, last, projection, reference, evidence or {}, {"stdlib_names": ["Stdlib"]}, output)

    def test_rebuild_faults_need_the_reference_projection_or_a_refusal(self) -> None:
        for fault in ("D6-F02", "D6-F05", "D6-F06"):
            self.assertTrue(self.verdict(fault, self.DONE, "completed", {"a": 1}, {"a": 1}))
            self.assertFalse(self.verdict(fault, self.DONE, "completed", {"a": 2}, {"a": 1}))
            self.assertTrue(self.verdict(fault, self.STOPPED, "failed"))
            self.assertFalse(self.verdict(fault, self.STOPPED, "timeout"))
        self.assertFalse(self.verdict("D6-F09", self.DONE, "completed", {"a": 1}, {"a": 1}))
        self.assertTrue(self.verdict("D6-F09", self.DONE, "completed", {"a": 1}, {"a": 1}, {"killed_after_first_artifact": True}))

    def test_attributable_failures(self) -> None:
        self.assertTrue(self.verdict("D6-F03", self.STOPPED, "failed", evidence={"removed": ["lib"]}, output=b"cannot find Stdlib"))
        self.assertFalse(self.verdict("D6-F03", self.STOPPED, "failed", evidence={"removed": ["lib"]}, output=b"error"))
        self.assertFalse(self.verdict("D6-F03", self.DONE, "completed", evidence={"removed": ["lib"]}, output=b"Stdlib"))
        self.assertTrue(self.verdict("D6-F04", self.DONE, "completed", evidence={"home_unchanged": True}))
        self.assertFalse(self.verdict("D6-F04", self.DONE, "completed", evidence={"home_unchanged": False}))
        self.assertTrue(self.verdict("D6-F07", self.STOPPED, "timeout"))
        self.assertFalse(self.verdict("D6-F07", self.STOPPED, "crash"))
        self.assertTrue(self.verdict("D6-F08", self.STOPPED, "timeout"))
        self.assertFalse(self.verdict("D6-F08", self.DONE, "completed"))
        self.assertTrue(self.verdict("D6-F01", self.DONE, "completed"))


class D006SolverClaimTests(unittest.TestCase):
    """DS-04's run-time cases: a solver run never becomes a proved claim without a checked certificate."""

    def test_claims_follow_the_pinned_exit_codes_and_step_states(self) -> None:
        claim = run.H.solver_claim
        self.assertEqual(claim("failed", 10, False), "disproved_obligation")
        self.assertEqual(claim("failed", 20, True), "certificate")
        self.assertEqual(claim("failed", 20, False), "failed_certificate")
        self.assertEqual(claim("completed", 0, True), "unknown")
        self.assertEqual(claim("timeout", None, True), "timeout")
        self.assertEqual(claim("resource_exhaustion", 20, True), "resource_exhaustion")
        self.assertEqual(claim("crash", -9, True), "unknown")
        self.assertEqual(claim("failed", 1, True), "unknown")

    def test_a_step_state_record_is_refused(self) -> None:
        with self.assertRaises(TypeError):
            run.H.solver_claim({"kind": "failed", "exit_code": 20, "signal": None}, 20, True)

    def test_the_argv_is_the_pinned_one(self) -> None:
        spec = json.loads((REPOSITORY_ROOT / run.SHARED_DIR / "ds04-lrat-obligation.json").read_text(encoding="utf-8"))["solver"]
        argv = run.H.solver_argv("/s/cadical", "unknown_argv", "/c.cnf", "/p.lrat")
        self.assertEqual(argv, ["/s/cadical", *spec["unknown_argv"][1:-2], "/c.cnf", "/p.lrat"])
        self.assertEqual(argv[1:3], ["-c", "0"])

    def test_a_model_gives_x_and_y_and_both_sides_of_the_identity(self) -> None:
        model = run.H.solver_model(b"s SATISFIABLE\nv -1 2 -3 " + b" ".join(str(v).encode() for v in range(4, 34)) + b"\nv 34 -35 0\n")
        x, y = run.H.model_words(model)
        self.assertEqual(x, 0xFFFFFFFD)
        self.assertEqual(y, 1)
        rows, left, right = run.H.counterexample_observations(x, y)
        self.assertEqual((left, right), ((x + y) & 0xFFFFFFFF, ((x ^ y) + (x & y)) & 0xFFFFFFFF))
        self.assertNotEqual(left, right)
        self.assertEqual([r["id"] for r in rows], ["D4-R02-L", "D4-R02-R"])
        self.assertIsNone(run.H.solver_model(b"s UNSATISFIABLE\n"))

    def test_verdicts_need_the_exact_category_and_the_candidates_part(self) -> None:
        ok = run.H.Outcome("D4-R02-L", "observation", True)
        bad = run.H.Outcome("D4-R02-R", "observation", False)
        self.assertTrue(run.H.run_time_verdict("D4-R02", "disproved_obligation", [ok, ok], 1, 2).passed)
        self.assertFalse(run.H.run_time_verdict("D4-R02", "disproved_obligation", [ok, bad], 1, 2).passed)
        self.assertFalse(run.H.run_time_verdict("D4-R02", "disproved_obligation", [], None, None).passed)
        self.assertFalse(run.H.run_time_verdict("D4-R03", "timeout", []).passed)
        self.assertTrue(run.H.run_time_verdict("D4-R04", "timeout", []).passed)
        self.assertFalse(run.H.run_time_verdict("D4-R05", "failed_certificate", []).passed)
        self.assertTrue(run.H.run_time_verdict("D4-R05", "failed_certificate", [ok]).passed)
        self.assertFalse(run.H.run_time_verdict("D4-R05", "certificate", [ok]).passed)


class D006ArchiveVerifyTests(unittest.TestCase):
    def make_archive(self, root: Path) -> Path:
        identity = {"suite_version": run.SUITE_VERSION, "bindings": [], "archives": {}, "input_manifest_sha256": "0" * 64,
                    "ceilings": run.CEILINGS, "sandbox_caps": run.SANDBOX_CAPS, "lab_uid": run.LAB_UID, "meter": "test", "dev": True}
        archive = root / run.epoch_name(identity)
        (archive / "records").mkdir(parents=True)
        packet = {"schema_version": "d006-v0.3-packet-1", "epoch": archive.name, "revision": "0" * 40, **identity,
                  "bootstrap_seed": run.bootstrap_seed(identity)}
        (archive / "packet.json").write_bytes(run.canonical_file(packet))
        projection = {"build": {"completed": True}, "cases": {}, "artifacts": []}
        record = {"schema_version": run.RECORD_SCHEMA, "suite_version": run.SUITE_VERSION, "epoch": archive.name, "ordinal": 1,
                  "profile": "deterministic_replay", "candidate": "C-01", "projection": projection, "projection_sha256": run.digest(projection)}
        (archive / "records/0001-deterministic_replay-C-01.json").write_bytes(run.canonical_file(record))
        run.write_manifest(archive)
        return archive

    def test_a_sound_archive_verifies_and_tampering_is_named(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            archive = self.make_archive(Path(tmp))
            self.assertEqual(run.command_verify(REPOSITORY_ROOT, archive), [])
            path = archive / "records/0001-deterministic_replay-C-01.json"
            record = json.loads(path.read_text())
            record["projection"]["build"]["completed"] = False
            path.write_bytes(run.canonical_file(record))
            problems = run.command_verify(REPOSITORY_ROOT, archive)
            self.assertIn("records/0001-deterministic_replay-C-01.json differs from the archive manifest", problems)
            self.assertIn("record 1 projection digest mismatch", problems)

    def test_a_renamed_epoch_fails_verification(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            archive = self.make_archive(Path(tmp))
            packet = json.loads((archive / "packet.json").read_text())
            packet["lab_uid"] = 0
            (archive / "packet.json").write_bytes(run.canonical_file(packet))
            problems = run.command_verify(REPOSITORY_ROOT, archive)
            self.assertIn("the packet's epoch name or bootstrap seed is not the hash of what it binds", problems)


class D006ExportTests(unittest.TestCase):
    def test_chunks_keep_order_and_stay_under_the_cap(self) -> None:
        lines = [(b"x" * 1000) + b"\n" for _ in range(900)]
        files = run.chunked(lines, "records")
        self.assertEqual(sorted(files), list(files))
        self.assertEqual(b"".join(files.values()), b"".join(lines))
        self.assertTrue(all(len(data) <= run.EXPORT_CHUNK for data in files.values()))

    def test_logs_round_trip_through_the_export_layout(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            export = Path(tmp)
            texts = {"plain": b"ok\n", "binary": bytes(range(256))}
            rows = []
            for data in texts.values():
                name = run.sha256(data)
                try:
                    rows.append(run.canonical({"sha256": name, "bytes": len(data), "text": data.decode("utf-8")}) + b"\n")
                except UnicodeDecodeError:
                    rows.append(run.canonical({"sha256": name, "bytes": len(data), "base64": run.base64.b64encode(data).decode("ascii")}) + b"\n")
            for name, data in run.chunked(rows, "logs").items():
                (export / name).write_bytes(data)
            self.assertEqual(run.load_logs(export), {run.sha256(data): data for data in texts.values()})


    def test_repeated_fields_are_stored_once_and_restored(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            export = Path(tmp)
            projection = {"build": {"completed": True}, "cases": {"DS-01": {"positives": []}}, "artifacts": []}
            records = [{"ordinal": n, "profile": "deterministic_replay", "projection": projection, "steps": [n]} for n in (1, 2, 3)]
            objects: dict = {}
            packed = [run.pack_record(r, objects) for r in records]
            self.assertEqual(list(objects), [run.digest(projection)])
            self.assertEqual(packed[0]["projection"], {"$object": run.digest(projection)})
            for name, data in run.chunked([run.canonical(r) + b"\n" for r in packed], "records").items():
                (export / name).write_bytes(data)
            rows = [run.canonical({"sha256": k, "value": v}) + b"\n" for k, v in objects.items()]
            for name, data in run.chunked(rows, "objects").items():
                (export / name).write_bytes(data)
            self.assertEqual(run.load_records(export), records)
            (export / "objects-01.jsonl").write_bytes(b"")
            with self.assertRaises(run.RunError):
                run.load_records(export)


class D006SharedReferenceBoundaryTests(unittest.TestCase):
    def test_candidates_and_archives_sit_outside_the_generated_laboratory(self) -> None:
        self.assertEqual(shared.GENERATED_ROOTS, ("shared-inputs", "protocol"))
        for path in run.CANDIDATE_DIRS.values():
            self.assertTrue(path.startswith(run.LAB + "/"))
            self.assertNotIn(Path(path).relative_to(run.LAB).parts[0], shared.GENERATED_ROOTS)


if __name__ == "__main__":
    unittest.main()
