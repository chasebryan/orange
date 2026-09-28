from __future__ import annotations

import copy
import json
import tempfile
import unittest
from pathlib import Path
from unittest import mock

from tools import d004_v08_adapter as adapter
from tools import d004_v08_run as run


REPOSITORY_ROOT = Path(__file__).resolve().parents[2]
REPOSITORY = run.Repository(REPOSITORY_ROOT)
SUITE = run.Suite(REPOSITORY)


class _Overlaid:
    """The repository with some committed documents replaced."""

    def __init__(self, base: run.Repository, documents: dict[str, bytes]) -> None:
        self._base, self._documents = base, documents

    def raw(self, path: str) -> bytes:
        return self._documents[path] if path in self._documents else self._base.raw(path)

    def json(self, path: str):
        return run.strict_json(self.raw(path))

    def __getattr__(self, name: str):
        return getattr(self._base, name)


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


def _positive(case: str) -> dict[str, object]:
    subject = copy.deepcopy(SUITE.rows[case][0]["subject"])
    subject["id"] = f"SYNTHETIC-POS-{case}"
    return subject


def _request(candidate: str, subjects: list[dict[str, object]], case: str, model: dict[str, object] | None = None) -> bytes:
    return adapter.canonical_bytes(
        {
            "schema_version": adapter.REQUEST_SCHEMA,
            "suite_version": adapter.SUITE_VERSION,
            "execution": _execution(candidate, case),
            "candidate_model": model or SUITE.models[candidate],
            "subjects": [
                {
                    "subject_id": subject["id"],
                    "subject_sha256": adapter.digest(subject),
                    "source_catalog": "case_subject_catalog",
                    "subject": subject,
                }
                for subject in subjects
            ],
        }
    ) + b"\n"


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


def _observe(candidate: str, subject: dict[str, object], case: str) -> dict[str, object]:
    response = json.loads(adapter.run_request(_request(candidate, [subject], case), "f" * 64))
    return response["observations"][0]


def _category(observation: dict[str, object]) -> object:
    return observation["normalized_observation"]["decision"]["category"]


class D004V08AdapterTests(unittest.TestCase):
    def test_models_name_their_judgments_and_definitions(self) -> None:
        for candidate, model in SUITE.models.items():
            again = adapter.derive_candidate_model({"candidate": candidate, "graph": model["graph"], "graph_sha256": model["graph_sha256"]})
            self.assertEqual(again, model)
            for judgment in model["model"]["discrimination_judgments"]:
                self.assertNotIn(judgment["relationship"], ("SR-06", "SR-12"))
                self.assertTrue(judgment["id"].startswith(f"DJ-{judgment['relationship']}-"))
                self.assertTrue(judgment["identity_inputs"])
            self.assertTrue(model["model"]["semantic_definitions"])

    def test_evolution_is_computed_from_the_subject(self) -> None:
        subject = _positive("SC-06")
        full = _observe("ST-UNI", subject, "SC-06")
        self.assertEqual(full["observed_state"], "succeeded")
        narrowed = copy.deepcopy(subject)
        narrowed["model"]["changes"] = [item for item in narrowed["model"]["changes"] if item["id"] == "E4"]
        observation = _observe("ST-UNI", narrowed, "SC-06")
        self.assertEqual(observation["observed_state"], "succeeded")
        values = observation["normalized_observation"]["values"]
        self.assertEqual([item["change"] for item in values["evolution"]], ["E4"])
        self.assertEqual(values["measures"]["spec_isolation_obligations"], 0)
        self.assertEqual(values["measures"]["isolation_obligations"], len(values["evolution"][0]["obligations"]))
        self.assertLess(values["measures"]["isolation_obligations"], full["normalized_observation"]["values"]["measures"]["isolation_obligations"])

    def test_only_a_change_that_reaches_a_protected_class_needs_isolation(self) -> None:
        subject = _positive("SC-06")
        expected = {"ST-REL": (0, 0, 6), "ST-MIRROR": (0, 0, 7), "ST-DUAL": (4, 0, 28), "ST-UNI": (10, 6, 36)}
        for candidate, counts in expected.items():
            measures = _observe(candidate, subject, "SC-06")["normalized_observation"]["values"]["measures"]
            with self.subTest(candidate=candidate):
                self.assertEqual((measures["isolation_obligations"], measures["spec_isolation_obligations"], measures["reidentified_classes"]), counts)
        # E1 changes the target operation class. ST-REL confines it to its
        # machine IR member; ST-UNI's single calculus also owns Spec Core.
        narrowed = copy.deepcopy(subject)
        narrowed["model"]["changes"] = [item for item in narrowed["model"]["changes"] if item["id"] == "E1"]
        split = _observe("ST-REL", narrowed, "SC-06")["normalized_observation"]["values"]
        self.assertEqual([len(item["reidentified"]) for item in split["evolution"]], [1])
        self.assertEqual((split["measures"]["isolation_obligations"], split["measures"]["spec_isolation_obligations"]), (0, 0))
        shared = _observe("ST-UNI", narrowed, "SC-06")["normalized_observation"]["values"]["measures"]
        self.assertEqual((shared["isolation_obligations"], shared["spec_isolation_obligations"]), (2, 2))

    def test_an_unowned_construct_is_rejected(self) -> None:
        subject = _positive("SC-06")
        subject["model"]["changes"][0]["facet"] = "synthetic_unheld_facet"
        observation = _observe("ST-REL", subject, "SC-06")
        self.assertEqual(observation["observed_state"], "rejected")
        self.assertEqual(_category(observation), "construct_facet_not_unique")

    def test_relabeling_is_rejected_by_boundary_or_named_judgment(self) -> None:
        subject = _positive("SC-07")
        subject["model"]["probes"] = [item for item in subject["model"]["probes"] if item["relationship"] == "SR-10"]
        separate = _observe("ST-REL", subject, "SC-07")["normalized_observation"]["values"]["relabel_probes"]
        self.assertEqual([item["rejected_by"] for item in separate], ["member_boundary"])
        shared = _observe("ST-UNI", subject, "SC-07")["normalized_observation"]["values"]["relabel_probes"]
        self.assertTrue(shared[0]["rejected_by"].startswith("DJ-SR-10-"))
        subject["model"]["probes"][0]["to_facet"] = "synthetic_unheld_facet"
        observation = _observe("ST-UNI", subject, "SC-07")
        self.assertEqual(observation["observed_state"], "rejected")
        self.assertEqual(_category(observation), "probe_does_not_match_crossing")

    def test_a_within_authority_crossing_without_a_named_judgment_fails(self) -> None:
        # The derived models always name these judgments; strip them to reach the check.
        stripped = copy.deepcopy(SUITE.models["ST-UNI"])
        stripped["model"]["discrimination_judgments"] = []
        stripped["model_sha256"] = adapter.digest(stripped["model"])
        subject = _positive("SC-07")
        with mock.patch.object(adapter, "derive_candidate_model", return_value=stripped):
            response = json.loads(adapter.run_request(_request("ST-UNI", [subject], "SC-07", stripped), "f" * 64))
        observation = response["observations"][0]
        self.assertEqual(observation["observed_state"], "rejected")
        self.assertEqual(_category(observation), "undiscriminated_within_authority_crossing")

    def test_mutation_breaks_the_bound_fact(self) -> None:
        positive = _positive("SC-06")
        mutation = copy.deepcopy(SUITE.rows["SC-06"][1]["subject"])
        mutation["id"] = "SYNTHETIC-MUT-SC06"
        mutation["positive_subject_sha256"] = adapter.digest(positive)
        response = json.loads(adapter.run_request(_request("ST-DUAL", [positive, mutation], "SC-06"), "f" * 64))
        self.assertEqual([item["observed_state"] for item in response["observations"]], ["succeeded", "rejected"])


class D004V08RunnerTests(unittest.TestCase):
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

    def test_committed_documents_are_the_mechanical_derivation(self) -> None:
        committed = tuple(REPOSITORY.raw(path) for path in (run.CATALOG_PATH, run.BUNDLE_PATH, run.OVERLAY_PATH))
        self.assertEqual(committed, run.generated_documents(REPOSITORY))

    def test_overlay_binds_the_owner_rule(self) -> None:
        overlay = run.canonical_document(REPOSITORY.raw(run.OVERLAY_PATH))
        rule = overlay["distinguishing_rule"]
        self.assertEqual(rule["id"], run.OWNER_RULE_CHOICE["rule"])
        self.assertEqual((rule["kind"], tuple(rule["order"])), run.DISTINGUISHING_RULES[rule["id"]])
        self.assertEqual(overlay["status"], "suite_built_run_pending")

    def test_prepare_waits_for_the_owner_rule(self) -> None:
        paths = (run.CATALOG_PATH, run.BUNDLE_PATH, run.OVERLAY_PATH)
        with mock.patch.object(run, "OWNER_RULE_CHOICE", None):
            unruled = _Overlaid(REPOSITORY, dict(zip(paths, run.generated_documents(REPOSITORY))))
            self.assertIsNone(run.canonical_document(unruled.raw(run.OVERLAY_PATH))["distinguishing_rule"])
            with tempfile.TemporaryDirectory(prefix="d004-v08-test-") as scratch:
                with self.assertRaisesRegex(run.RunError, "distinguishing rule is not recorded"):
                    run.command_prepare(unruled, Path(scratch), "synthetic")
                self.assertEqual(list(Path(scratch).iterdir()), [])

    def test_rotation_schedule_covers_every_slot_once_per_repetition(self) -> None:
        schedule = run.rotation_schedule()
        self.assertEqual(len(schedule), run.SLOTS * run.REPETITIONS)
        self.assertEqual([row["execution_ordinal"] for row in schedule], list(range(1, len(schedule) + 1)))
        pairs = {(candidate, case) for candidate in run.CANDIDATES for case in run.CASES}
        for repetition in range(run.REPETITIONS):
            rows = schedule[repetition * run.SLOTS:(repetition + 1) * run.SLOTS]
            self.assertEqual({(row["candidate"], row["case"]) for row in rows}, pairs)
            self.assertEqual([row["logical_slot_ordinal"] for row in rows], list(range(1, run.SLOTS + 1)))
            for start in range(0, run.SLOTS, len(run.CANDIDATES)):
                self.assertEqual({row["candidate"] for row in rows[start:start + len(run.CANDIDATES)]}, set(run.CANDIDATES))

    def test_rules_never_trade_one_measure_against_another(self) -> None:
        names = [name for name, _, _ in run.MEASURES]
        measures = {
            "ST-A": dict.fromkeys(names, 1) | {"semantic_definitions": 9},
            "ST-B": dict.fromkeys(names, 2) | {"semantic_definitions": 1},
            "ST-C": dict.fromkeys(names, 2) | {"semantic_definitions": 2},
        }
        self.assertEqual(run.apply_rule("isolation_first", measures)["result"], "recommend_st_a")
        self.assertEqual(run.apply_rule("fewest_definitions_first", measures)["result"], "recommend_st_b")
        dominance = run.apply_rule("dominance_only", measures)
        self.assertEqual(dominance["remaining"], ["ST-A", "ST-B"])
        self.assertEqual(dominance["result"], "inconclusive")
        tied = {"ST-A": measures["ST-A"], "ST-D": dict(measures["ST-A"])}
        self.assertEqual(run.apply_rule("isolation_first", tied)["result"], "inconclusive")

    def test_measures_are_read_only_from_a_matched_positive(self) -> None:
        sc06 = {"isolation_obligations": 0, "spec_isolation_obligations": 0, "reidentified_classes": 6, "semantic_definitions": 5}
        response = {"observations": [{"normalized_observation": {"values": {"measures": dict(sc06)}}}]}
        matched = [{"comparison": "matched"}]
        self.assertEqual(run.positive_measures(response, matched, "SC-06"), sc06)
        self.assertIsNone(run.positive_measures(response, matched, "SC-07"))
        self.assertIsNone(run.positive_measures(response, [{"comparison": "mismatched"}], "SC-06"))
        self.assertIsNone(run.positive_measures(None, matched, "SC-06"))
        for bad in (sc06 | {"unknown_measure": 1}, sc06 | {"semantic_definitions": -1}, sc06 | {"semantic_definitions": True}, {"semantic_definitions": 5}):
            response["observations"][0]["normalized_observation"]["values"]["measures"] = bad
            self.assertIsNone(run.positive_measures(response, matched, "SC-06"))
        response["observations"][0]["normalized_observation"]["values"] = []
        self.assertIsNone(run.positive_measures(response, matched, "SC-06"))

    def test_a_complete_candidate_without_measures_makes_the_rule_inconclusive(self) -> None:
        names = [name for name, _, _ in run.MEASURES]
        measures = {"ST-A": dict.fromkeys(names, 0), "ST-B": dict.fromkeys(names, 1)}
        self.assertEqual(run.rule_result("isolation_first", measures, [])["result"], "recommend_st_a")
        result = run.rule_result("isolation_first", measures, ["ST-C"])
        self.assertEqual((result["remaining"], result["result"], result["unmeasured"]), ([], "inconclusive", ["ST-C"]))

    def test_response_validator_requires_an_object_of_values(self) -> None:
        subject = _positive("SC-06")
        request = _request("ST-REL", [subject], "SC-06")
        stdout = adapter.run_request(request, "f" * 64)
        contract, rows, model = adapter.adapter_contract(), _rows([subject]), SUITE.models["ST-REL"]
        run.validate_response(contract, stdout, b"", request, rows, model)
        value = json.loads(stdout)
        normalized = value["observations"][0]["normalized_observation"]
        normalized["values"] = []
        value["observations"][0]["normalized_observation_sha256"] = run.digest(normalized)
        with self.assertRaisesRegex(run.PayloadError, "normalized values is not an object"):
            run.validate_response(contract, run.canonical_file(value), b"", request, rows, model)


if __name__ == "__main__":
    unittest.main()
