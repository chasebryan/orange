from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

from tools import d004_archive as archive
from tools import d004_run as run
from tools import d004_v08_run as run08


REPOSITORY = run.Repository(Path(__file__).resolve().parents[2])
INDEX_RAW = REPOSITORY.raw(archive.INDEX_PATH)
OUTPUTS_RAW = REPOSITORY.raw(archive.OUTPUTS_PATH)
V08 = archive.LAYOUTS["v0.8"]
REPOSITORY08 = run08.Repository(REPOSITORY.root)


def _unpack(index_raw: bytes, outputs_raw: bytes) -> dict[str, object]:
    with tempfile.TemporaryDirectory(prefix="d004-archive-test-") as scratch:
        return archive.unpack(REPOSITORY, index_raw, outputs_raw, Path(scratch) / "archive")


class D004ArchiveTests(unittest.TestCase):
    def test_committed_run_rebuilds_and_verifies(self) -> None:
        self.assertEqual(archive.check(REPOSITORY), [])
        index = run.canonical_document(INDEX_RAW)
        self.assertEqual(index["epoch"], index["packet"]["epoch"])
        self.assertEqual(len(index["executions"]), 75)

    def test_committed_run_summary(self) -> None:
        summary = _unpack(INDEX_RAW, OUTPUTS_RAW)
        self.assertEqual(
            summary["execution"],
            {
                "required_execution_records": 75,
                "result_record_count": 75,
                "required_candidate_cases": 25,
                "completed_candidate_cases": 20,
                "complete_candidates": 4,
                "complete_cross_candidate_cases": 0,
            },
        )
        self.assertIsNone(summary["selection"])
        failed = [slot for slot in summary["slots"] if slot["closure"] != "closed"]
        self.assertEqual({slot["candidate"] for slot in failed}, {"ST-HOST"})
        self.assertTrue(all(slot["repetitions_identical"] for slot in summary["slots"]))

    def test_tampered_measurement_is_rejected(self) -> None:
        index = run.canonical_document(INDEX_RAW)
        index["executions"][0]["measured_resources"]["wall_milliseconds"] += 1
        with self.assertRaisesRegex(run.RunError, "rebuilt archive differs"):
            _unpack(run.canonical_file(index), OUTPUTS_RAW)

    def test_tampered_or_extra_output_is_rejected(self) -> None:
        index = run.canonical_document(INDEX_RAW)
        outputs = run.canonical_document(OUTPUTS_RAW)
        first = index["executions"][0]["stdout_sha256"]
        tampered = {**outputs, "outputs": {**outputs["outputs"], first: {"utf8": "{}\n"}}}
        with self.assertRaisesRegex(run.RunError, "stdout digest mismatch"):
            _unpack(INDEX_RAW, run.canonical_file(tampered))
        extra = {**outputs, "outputs": {**outputs["outputs"], "0" * 64: {"utf8": ""}}}
        with self.assertRaisesRegex(run.RunError, "unreferenced"):
            _unpack(INDEX_RAW, run.canonical_file(extra))


class D004V08ArchiveTests(unittest.TestCase):
    def test_committed_run_rebuilds_and_verifies(self) -> None:
        self.assertEqual(archive.check(REPOSITORY08, V08), [])
        index = run08.canonical_document(REPOSITORY08.raw(V08.index_path))
        self.assertEqual(index["epoch"], index["packet"]["epoch"])
        self.assertEqual(len(index["executions"]), 105)

    def test_committed_run_summary(self) -> None:
        with tempfile.TemporaryDirectory(prefix="d004-archive-test-") as scratch:
            summary = archive.unpack(REPOSITORY08, REPOSITORY08.raw(V08.index_path), REPOSITORY08.raw(V08.outputs_path), Path(scratch) / "archive", V08)
        self.assertEqual(summary["execution"]["completed_candidate_cases"], 28)
        self.assertEqual(summary["execution"]["result_record_count"], 105)
        self.assertIsNone(summary["selection"])
        failed = [slot for slot in summary["slots"] if slot["closure"] != "closed"]
        self.assertEqual({slot["candidate"] for slot in failed}, {"ST-HOST"})
        result = summary["distinguishing_rule_result"]
        self.assertEqual((result["rule"], result["result"]), ("isolation_first", "recommend_st_rel"))
        self.assertEqual((result["compared"], result["unmeasured"]), (["ST-DUAL", "ST-MIRROR", "ST-REL", "ST-UNI"], []))

    def test_a_complete_candidate_without_measures_leaves_the_rule_inconclusive(self) -> None:
        with tempfile.TemporaryDirectory(prefix="d004-archive-test-") as scratch:
            root = Path(scratch) / "archive"
            archive.unpack(REPOSITORY08, REPOSITORY08.raw(V08.index_path), REPOSITORY08.raw(V08.outputs_path), root, V08)
            ctx = run08.EpochContext(root, REPOSITORY08)
            records = [
                run08.canonical_document((root / "executions" / f"{row['execution_ordinal']:03d}" / "record.json").read_bytes())
                for row in ctx.schedule
            ]
            for record in records:
                if (record["candidate"], record["case"]) == ("ST-REL", "SC-07"):
                    record["measures"] = None
            _, summary = run08.summarize(ctx, records)
        self.assertEqual(summary["execution"]["complete_candidates"], 4)
        self.assertNotIn("ST-REL", summary["measures"])
        result = summary["distinguishing_rule_result"]
        self.assertEqual((result["remaining"], result["result"], result["unmeasured"]), ([], "inconclusive", ["ST-REL"]))


if __name__ == "__main__":
    unittest.main()
