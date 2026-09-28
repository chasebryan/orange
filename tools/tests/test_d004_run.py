from __future__ import annotations

import copy
import json
import shutil
import signal
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock

from tools import d004_adapter as adapter
from tools import d004_run as run
from tools.validate_foundation import FoundationValidator


REPOSITORY_ROOT = Path(__file__).resolve().parents[2]
REPOSITORY = run.Repository(REPOSITORY_ROOT)
SUITE = run.Suite(REPOSITORY)
PYTHON = sys.executable


def _execution(candidate: str, case: str) -> dict[str, object]:
    return {
        "epoch": "d004-e-synthetic",
        "packet_sha256": "0" * 64,
        "replay_plan_sha256": "1" * 64,
        "scheduled_slot_sha256": "2" * 64,
        "logical_slot_ordinal": 1,
        "round": 1,
        "position": 1,
        "candidate": candidate,
        "case": case,
    }


def _synthetic_sc01() -> dict[str, object]:
    """An SC-01 positive with vectors that are not in the reviewed catalog."""
    subject = copy.deepcopy(SUITE.rows["SC-01"][0]["subject"])
    subject["id"] = "SYNTHETIC-POS-SC01"
    tree = adapter._parse_expression(subject["model"]["round_like_ast"]["operation"])
    vectors = []
    for a, b, c in ((0x12345678, 0x9ABCDEF0, 0x0F0F0F0F), (0xDEADBEEF, 0x00000001, 0x80000000)):
        inputs = {"a": a, "b": b, "c": c}
        vectors.append(
            {
                "inputs": {name: f"0x{value:08x}" for name, value in inputs.items()},
                "normalized_word32_result": adapter._hex_word(adapter._evaluate(tree, inputs)),
            }
        )
    subject["model"]["boundary_vectors"] = vectors
    return subject


def _entry(subject: dict[str, object], catalog: str = "case_subject_catalog") -> dict[str, object]:
    return {
        "subject_id": subject.get("id", subject.get("proposal_id")),
        "subject_sha256": adapter.digest(subject),
        "source_catalog": catalog,
        "subject": subject,
    }


def _request(candidate: str, subjects: list[dict[str, object]], case: str = "SC-01") -> bytes:
    return adapter.canonical_bytes(
        {
            "schema_version": adapter.REQUEST_SCHEMA,
            "suite_version": adapter.SUITE_VERSION,
            "execution": _execution(candidate, case),
            "candidate_model": SUITE.models[candidate],
            "subjects": [_entry(subject) for subject in subjects],
        }
    ) + b"\n"


def _response(request: bytes) -> dict[str, object]:
    return json.loads(adapter.run_request(request, "f" * 64))


def _rows(subjects: list[dict[str, object]]) -> list[dict[str, object]]:
    return [
        {
            "source_catalog": "case_subject_catalog",
            "subject": subject,
            "oracle": {
                "subject_id": subject["id"],
                "subject_sha256": adapter.digest(subject),
                "relationship_scope": list(subject["relationship_scope"]),
            },
        }
        for subject in subjects
    ]


class D004AdapterTests(unittest.TestCase):
    def test_candidate_models_derive_from_their_graphs(self) -> None:
        for candidate, model in SUITE.models.items():
            self.assertEqual(model["candidate"], candidate)
            self.assertEqual(model["model_sha256"], adapter.digest(model["model"]))
            self.assertEqual(len(model["model"]["crossings"]), 14)
            self.assertEqual(adapter.derive_candidate_model({"candidate": candidate, "graph": model["graph"], "graph_sha256": model["graph_sha256"]}), model)

    def test_synthetic_positive_is_evaluated_not_echoed(self) -> None:
        subject = _synthetic_sc01()
        response = _response(_request("ST-REL", [subject]))
        observation = response["observations"][0]
        self.assertEqual(observation["observed_state"], "succeeded")
        self.assertEqual(observation["normalized_observation"]["values"]["word32_results"], [item["normalized_word32_result"] for item in subject["model"]["boundary_vectors"]])
        wrong = copy.deepcopy(subject)
        wrong["model"]["boundary_vectors"][0]["normalized_word32_result"] = "0x00000000"
        observation = _response(_request("ST-REL", [wrong]))["observations"][0]
        self.assertEqual(observation["observed_state"], "rejected")
        self.assertEqual(observation["normalized_observation"]["decision"]["category"], "evaluation_mismatch")

    def test_mutation_must_bind_the_positive_baseline(self) -> None:
        positive = _synthetic_sc01()
        mutation = {
            "case": "SC-01",
            "id": "SYNTHETIC-MUT",
            "kind": "suite-only-named-mutation",
            "model": {
                "baseline_value": "explicit_only",
                "dependent_result": {"id": "dependent_result", "required_target": "shift_bound", "required_value": "x"},
                "kind": "suite-only-single-invariant-mutation",
                "mutated_value": "unbounded",
                "operator": "replace",
                "target": "shift_bound",
            },
            "mutation_id": "SYNTHETIC-M01",
            "positive_subject_sha256": adapter.digest(positive),
            "relationship_scope": ["SR-01"],
            "schema_version": "d004-case-subject-v0.1",
        }
        with self.assertRaises(adapter.AdapterFailure) as raised:
            adapter.run_request(_request("ST-REL", [positive, mutation]), "f" * 64)
        self.assertEqual(raised.exception.kind, "digest_mismatch")
        mutation["model"]["baseline_value"] = "bounded_to_word_width"
        observation = _response(_request("ST-REL", [positive, mutation]))["observations"][1]
        self.assertEqual((observation["observed_state"], observation["observed_invalidation"]), ("rejected", "satisfied"))

    def test_delegated_relationships_with_an_open_host_are_unsupported(self) -> None:
        response = _response(_request("ST-HOST", [_synthetic_sc01()]))
        delegated = {item["relationship"] for item in SUITE.models["ST-HOST"]["model"]["crossings"] if item["execution"] == "delegated"}
        self.assertTrue(delegated)
        states = {item["relationship"]: item["conformance_state"] for item in response["sr_conformance"]}
        self.assertEqual({key for key, value in states.items() if value == "unsupported"}, delegated)
        self.assertEqual([item["id"] for item in response["unsupported_features"]], [f"U-{item}" for item in sorted(delegated)])
        self.assertNotIn("unsupported", _response(_request("ST-REL", [_synthetic_sc01()]))["sr_conformance"][0].values())

    def test_request_transport_is_closed(self) -> None:
        request = _request("ST-REL", [_synthetic_sc01()])
        for mutate, kind in (
            (lambda value: value.update(extra=1), "unsupported_behavior"),
            (lambda value: value["subjects"][0].update(subject_sha256="0" * 64), "digest_mismatch"),
            (lambda value: value["candidate_model"].update(model_sha256="0" * 64), "digest_mismatch"),
            (lambda value: value.update(subjects=[]), "missing_input"),
        ):
            value = json.loads(request)
            mutate(value)
            with self.assertRaises(adapter.AdapterFailure) as raised:
                adapter.run_request(adapter.canonical_bytes(value) + b"\n", "f" * 64)
            self.assertEqual(raised.exception.kind, kind)
        with self.assertRaises(adapter.AdapterFailure):
            adapter.run_request(request.replace(b'":', b'": ', 1), "f" * 64)


class D004RunnerTests(unittest.TestCase):
    def test_committed_bundle_and_overlay_are_the_mechanical_derivation(self) -> None:
        bundle, overlay = run.generated_documents(REPOSITORY)
        self.assertEqual(REPOSITORY.raw(run.BUNDLE_PATH), bundle)
        self.assertEqual(REPOSITORY.raw(run.OVERLAY_PATH), overlay)
        validator = FoundationValidator(REPOSITORY_ROOT)
        run.validate_d004_prerequisites(validator)
        self.assertEqual([item.code for item in validator.findings if item.code.startswith("d004_prerequisites.")], [])

    def test_validator_hook_detects_adapter_and_input_drift(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            shutil.copytree(REPOSITORY_ROOT / run.D004, root / run.D004)
            (root / "tools").mkdir()
            (root / run.ADAPTER_PATH).write_bytes(REPOSITORY.raw(run.ADAPTER_PATH) + b"\n")
            protocol = root / run.INPUT_PATHS["reviewed_protocol"]
            protocol.write_bytes(protocol.read_bytes() + b"\n")
            validator = FoundationValidator(root)
            run.validate_d004_prerequisites(validator)
            codes = {item.code for item in validator.findings}
            self.assertIn("d004_prerequisites.adapter_identity", codes)
            self.assertIn("d004_prerequisites.input_identity", codes)

    def test_schedule_and_identity_preimages_follow_the_reviewed_plan(self) -> None:
        self.assertEqual(SUITE.replay_plan["schedule"], run.latin_schedule())
        self.assertEqual(list(run.EXECUTION_PREIMAGE), SUITE.protocol["replay_contract"]["execution_identity_preimage_fields"])
        tool, dependency, environment = {"t": 1}, {"d": 1}, {"e": 1}
        bundle = run.binding(run.BUNDLE_PATH, REPOSITORY.raw(run.BUNDLE_PATH))
        overlay = run.binding(run.OVERLAY_PATH, REPOSITORY.raw(run.OVERLAY_PATH))
        packet = run.epoch_packet(bundle, overlay, b"runner", tool, dependency, environment, SUITE)
        plan = run.epoch_replay_plan(packet, SUITE)
        rows = run.schedule_identities(packet, plan, SUITE, tool, dependency, environment)
        self.assertEqual(len({row["scheduled_execution_sha256"] for row in rows}), 75)
        self.assertEqual(len({row["scheduled_slot_sha256"] for row in rows}), 25)
        other = run.epoch_packet(bundle, overlay, b"runner", tool, {"d": 2}, environment, SUITE)
        self.assertNotEqual(packet["epoch"], other["epoch"])
        self.assertEqual(packet["selection"], {"rule": None, "tie_break": "forbidden_after_results_exist"})
        first = run.canonical(run.request_document(SUITE, packet, plan, rows[0]))
        self.assertEqual(first, run.canonical(run.request_document(SUITE, packet, plan, rows[25])))
        for leaked in (b"declared_expectation", b"expected_observation", b"allowed_domain_states", b"required_invalidation"):
            self.assertNotIn(leaked, first)

    def test_case_inventory_orders_positive_mutations_then_fixtures(self) -> None:
        for case in run.CASES:
            rows = SUITE.rows[case]
            kinds = [row["source_catalog"] for row in rows]
            self.assertEqual(rows[0]["subject"]["kind"], "suite-only-positive-case")
            self.assertEqual(kinds, sorted(kinds, key=lambda item: item != "case_subject_catalog"))
            self.assertEqual(SUITE.manifests[case]["required_relationships"], list(run.RELATIONSHIPS))

    def test_response_validator_rejects_contract_drift(self) -> None:
        subjects = [_synthetic_sc01()]
        request = _request("ST-REL", subjects)
        stdout = adapter.run_request(request, "f" * 64)
        contract = adapter.adapter_contract()
        rows, model = _rows(subjects), SUITE.models["ST-REL"]
        run.validate_response(contract, stdout, b"", request, rows, model)
        with self.assertRaises(run.PayloadError):
            run.validate_response(contract, stdout, b"warning\n", request, rows, model)
        value = json.loads(stdout)
        value["observations"][0]["normalized_observation"]["values"] = {}
        with self.assertRaises(run.PayloadError):
            run.validate_response(contract, run.canonical_file(value), b"", request, rows, model)
        value = json.loads(stdout)
        value["sr_conformance"][0]["dependent_observation_ids"] = []
        with self.assertRaises(run.PayloadError):
            run.validate_response(contract, run.canonical_file(value), b"", request, rows, model)
        with self.assertRaises(run.PayloadError):
            run.validate_response(contract, stdout, b"", request, rows * 2, model)

    def _launch(self, code: str, **bounds: object) -> dict[str, object]:
        with tempfile.TemporaryDirectory() as directory:
            launched = run.run_bounded([PYTHON, "-I", "-S", "-c", code], Path(directory), run.RusageMeter(), **bounds)
        return launched

    def test_launcher_classifies_exits_signals_timeouts_and_oversized_output(self) -> None:
        self.assertEqual(run.classify(self._launch("print('ok')"))["kind"], "completed")
        self.assertEqual(run.classify(self._launch("raise SystemExit(4)"))["kind"], "digest_mismatch")
        self.assertEqual(run.classify(self._launch("raise SystemExit(9)"))["kind"], "crash")
        segv = run.classify(self._launch("import os,signal; os.kill(os.getpid(), signal.SIGSEGV)"))
        self.assertEqual((segv["kind"], segv["signal"], segv["exit_code"]), ("crash", "SIGSEGV", None))
        cpu = run.classify(self._launch(f"raise SystemExit({128 + signal.SIGXCPU})"))
        self.assertEqual((cpu["kind"], cpu["signal"]), ("resource_exhaustion", "SIGXCPU"))
        slow = self._launch("import time; time.sleep(30)", timeout_seconds=0.3)
        self.assertEqual(run.classify(slow)["kind"], "timeout")
        self.assertLess(slow["wall_milliseconds"], 20000)
        large = self._launch("import sys; sys.stdout.write('x' * 5000)", output_cap=100)
        state = run.classify(large)
        self.assertEqual((state["kind"], state["stdout_truncated"], len(large["stdout"])), ("oversized_output", True, 100))
        self.assertGreater(self._launch("x = bytearray(32 * 1024 * 1024)")["peak_memory_bytes"], 32 * 1024 * 1024)

    def test_stage_cleanup_detects_tampering(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            base = Path(directory)
            stage = run.stage_execution(base, 1, b"adapter", b"request")
            (stage / "tmp" / "scratch").write_bytes(b"12345")
            self.assertEqual(run.measure_temp(stage), 5)
            self.assertEqual(run.teardown(stage, b"adapter", b"request"), {"inputs_unchanged": True, "stage_removed": True})
            self.assertFalse(stage.exists())
            stage = run.stage_execution(base, 2, b"adapter", b"request")
            (stage / "in").chmod(0o700)
            (stage / "in" / "extra").write_bytes(b"x")
            with self.assertRaises(run.RunError):
                run.teardown(stage, b"adapter", b"request")

    @staticmethod
    def _record(repetition: int, **changes: object) -> dict[str, object]:
        record = {
            "logical_slot_ordinal": 1, "candidate": "ST-REL", "case": "SC-01", "repetition": repetition,
            "execution_ordinal": 25 * (repetition - 1) + 1, "case_verdict": "pass", "observations": [],
            "identities": {"scheduled_execution_sha256": str(repetition), "model_sha256": "m"},
            "measured_resources": {"wall_milliseconds": 10 * repetition, "peak_memory_bytes": 1000 + repetition, "temp_storage_bytes": repetition, "stdout_bytes": 7, "stderr_bytes": 0},
        }
        record.update(changes)
        return record

    def test_repetition_closure_mirrors_the_reviewed_contract(self) -> None:
        records = [self._record(index) for index in (1, 2, 3)]
        self.assertEqual(run.repetition_closure(records)["closure"], "closed")
        self.assertEqual(run.repetition_closure(records[:2])["error"], "Cardinality")
        self.assertEqual(run.repetition_closure([records[1], records[0], records[2]])["error"], "RepetitionOrder")
        self.assertEqual(run.repetition_closure([records[0], self._record(2, case_verdict="fail"), records[2]])["error"], "IndependentFailure")
        self.assertEqual(run.repetition_closure([records[0], self._record(2, observations=[1]), records[2]])["error"], "DeterministicMismatch")
        heavy = self._record(3)
        heavy["measured_resources"] = {**heavy["measured_resources"], "wall_milliseconds": 900001}
        self.assertEqual(run.repetition_closure([records[0], records[1], heavy])["error"], "ResourceLimit")
        self.assertEqual(run.repetition_closure([self._record(index, candidate="ST-UNI") for index in (1, 2, 3)])["error"], "CoordinateMismatch")

    def test_correction_records_cannot_widen_a_shared_change(self) -> None:
        bundle = json.loads(REPOSITORY.raw(run.BUNDLE_PATH))
        prior = {"ST-REL": [f"{index:064x}" for index in range(15)]}
        correction = {
            "schema_version": "d004-correction-record-v0.1",
            "record_id": "D004-COR-ST-REL-01",
            "candidate": "ST-REL",
            "epoch": "d004-e-x",
            "first_packet_record_sha256s": prior["ST-REL"],
            "changed_paths": [{"path": run.ADAPTER_PATH}],
            "reason": "synthetic",
        }
        errors = run.parse_corrections([correction], bundle, "d004-e-x", prior)
        self.assertEqual(errors, ["correction 0 changes a shared path; a shared change starts a new epoch for all candidates"])
        second = dict(correction, record_id="D004-COR-ST-REL-02", first_packet_record_sha256s=[])
        errors = run.parse_corrections([correction, second], bundle, "d004-e-x", prior)
        self.assertTrue(any("second correction window" in item for item in errors))
        self.assertTrue(any("identifier" in item for item in errors))
        self.assertTrue(any("complete first candidate packet" in item for item in errors))

    def test_archive_manifest_detects_changed_missing_and_unlisted_files(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            archive = Path(directory)
            (archive / "epoch").mkdir()
            (archive / "epoch" / "packet.json").write_bytes(b"{}\n")
            (archive / "summary.json").write_bytes(b"{}\n")
            (archive / "manifest.json").write_bytes(run.canonical_file(run.archive_manifest(archive)))
            self.assertEqual(run.parse_archive_manifest(archive), [])
            (archive / "summary.json").write_bytes(b"[]\n")
            (archive / "extra").write_bytes(b"x")
            errors = run.parse_archive_manifest(archive)
            self.assertIn("summary.json bytes changed", errors)
            self.assertIn("unlisted archive files: ['extra']", errors)
            manifest = json.loads((archive / "manifest.json").read_bytes())
            manifest["entries"][0]["path"] = "../escape"
            (archive / "manifest.json").write_bytes(run.canonical_file(manifest))
            self.assertTrue(any("escapes the archive" in item for item in run.parse_archive_manifest(archive)))

    def test_host_context_changes_names_each_recaptured_manifest_that_drifted(self) -> None:
        prepared = {
            "tool": {"interpreter": {"raw_sha256": "a" * 64}, "sandbox": {"binary_raw_sha256": "b" * 64}},
            "dependency": {"mapped_files": [{"file": "/usr/lib/libc.so.6", "raw_sha256": "c" * 64}]},
            "environment": {"argv": ["/usr/bin/python3"], "isolation_probe": {"network": "denied"}},
        }

        class Host:
            def __init__(self, **current: dict[str, object]) -> None:
                self.current, self.adapters = {**copy.deepcopy(prepared), **current}, []

            def isolation_probe(self, base: Path, adapter_raw: bytes) -> dict[str, object]:
                self.adapters.append(adapter_raw)
                return self.current["environment"]["isolation_probe"]

            def tool_manifest(self, adapter_raw: bytes) -> dict[str, object]:
                self.adapters.append(adapter_raw)
                return self.current["tool"]

            def dependency_manifest(self, base: Path, adapter_raw: bytes) -> dict[str, object]:
                self.adapters.append(adapter_raw)
                return self.current["dependency"]

            def environment_manifest(self, probe: dict[str, object]) -> dict[str, object]:
                return {**self.current["environment"], "isolation_probe": probe}

        ctx = type("Context", (), {"adapter_raw": b"archived adapter", **copy.deepcopy(prepared)})()
        same = Host()
        self.assertEqual(run.host_context_changes(same, ctx, Path("work")), [])
        self.assertEqual(same.adapters, [b"archived adapter"] * 3)
        swapped_libc = {"mapped_files": [{"file": "/usr/lib/libc.so.6", "raw_sha256": "d" * 64}]}
        self.assertEqual(run.host_context_changes(Host(dependency=swapped_libc), ctx, Path("work")), ["dependency manifest"])
        swapped_python = {**prepared["tool"], "interpreter": {"raw_sha256": "e" * 64}}
        self.assertEqual(run.host_context_changes(Host(tool=swapped_python), ctx, Path("work")), ["tool manifest"])
        open_network = {**prepared["environment"], "isolation_probe": {"network": "allowed"}}
        self.assertEqual(run.host_context_changes(Host(environment=open_network), ctx, Path("work")), ["environment manifest"])

    def test_execute_refuses_before_any_launch_when_the_host_changed(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            archive = Path(directory)
            host = mock.MagicMock()
            with (
                mock.patch.object(run, "EpochContext", return_value=mock.MagicMock(schedule=[{"execution_ordinal": 1, "logical_slot_ordinal": 1}])),
                mock.patch.object(run, "Host", return_value=host),
                mock.patch.object(run, "host_context_changes", return_value=["dependency manifest"]) as recheck,
            ):
                with self.assertRaisesRegex(run.RunError, "the host differs from the prepared epoch: dependency manifest"):
                    run.command_execute(REPOSITORY, archive)
            recheck.assert_called_once()
            host.run_staged.assert_not_called()
            self.assertFalse((archive / "executions").exists())


if __name__ == "__main__":
    unittest.main()
