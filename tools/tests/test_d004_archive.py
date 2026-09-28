from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

from tools import d004_archive as archive
from tools import d004_run as run


REPOSITORY = run.Repository(Path(__file__).resolve().parents[2])
INDEX_RAW = REPOSITORY.raw(archive.INDEX_PATH)
OUTPUTS_RAW = REPOSITORY.raw(archive.OUTPUTS_PATH)


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


if __name__ == "__main__":
    unittest.main()
