"""Committed form of the D-004 run archive (research-only).

``tools/d004_run.py execute`` writes an archive of about 13.5 MB. Most of it
can be re-derived from the repository: the requests, candidate models and
input manifests come from the suite, the replay plan and the 75 scheduled
identities come from the packet, and every case record, repetition closure and
the summary come from each execution's archived outputs. The committed form
keeps only what cannot be re-derived:

* ``archive-index.json``: the epoch packet, the three host captures, the
  provenance note, and each execution's state, measurements, diagnostics and
  output digests, plus the SHA-256 of the archive manifest written at run
  time;
* ``adapter-outputs.json``: each distinct adapter output, keyed by its raw
  SHA-256.

``unpack`` rebuilds the archive byte for byte with the run harness's own
functions and fails unless the rebuilt archive manifest has the recorded
digest. ``check`` unpacks the committed form and runs the harness's
``verify`` over the result.
"""

from __future__ import annotations

import sys
import tempfile
from pathlib import Path
from typing import Any

try:
    from tools import d004_run as run
except ImportError:  # run as a script from the tools directory
    import d004_run as run  # type: ignore[no-redef]

PACK_ROOT = "research/decisions/D-004/d004-v0.7/run"
INDEX_PATH = f"{PACK_ROOT}/archive-index.json"
OUTPUTS_PATH = f"{PACK_ROOT}/adapter-outputs.json"
INDEX_SCHEMA = "d004-archive-index-v0.1"
OUTPUTS_SCHEMA = "d004-adapter-outputs-v0.1"
HOST_CAPTURES = (
    ("tool_manifest", "tool-manifest.json"),
    ("dependency_manifest", "dependency-manifest.json"),
    ("environment_manifest", "environment-manifest.json"),
)
EXECUTION_FIELDS = ("execution_ordinal", "execution_state", "measured_resources", "diagnostics", "stdout_sha256", "stderr_sha256")


def _encode_output(data: bytes) -> dict[str, Any]:
    try:
        return {"canonical_json": run.canonical_document(data)}
    except (UnicodeError, ValueError):
        return {"utf8": data.decode("utf-8")}


def _decode_output(item: Any) -> bytes:
    if isinstance(item, dict) and set(item) == {"canonical_json"}:
        return run.canonical_file(item["canonical_json"])
    if isinstance(item, dict) and set(item) == {"utf8"} and isinstance(item["utf8"], str):
        return item["utf8"].encode("utf-8")
    raise run.RunError("adapter output entry is not closed")


def pack(repository: run.Repository, archive: Path) -> tuple[bytes, bytes]:
    """Reduce a verified archive to its committed form, then prove the round trip."""
    errors = run.command_verify(repository, archive)
    if errors:
        raise run.RunError(f"archive does not verify: {errors[:3]}")
    ctx = run.EpochContext(archive, repository)
    outputs: dict[str, Any] = {}
    executions = []
    for row in ctx.schedule:
        directory = archive / "executions" / f"{row['execution_ordinal']:02d}"
        record = run.canonical_document((directory / "record.json").read_bytes())
        streams = {}
        for name in ("stdout", "stderr"):
            data = (directory / name).read_bytes()
            streams[name] = run.sha256(data)
            outputs[streams[name]] = _encode_output(data)
        executions.append(
            {
                "execution_ordinal": row["execution_ordinal"],
                "execution_state": record["execution_state"],
                "measured_resources": record["measured_resources"],
                "diagnostics": run.canonical_document((directory / "diagnostics.json").read_bytes()),
                "stdout_sha256": streams["stdout"],
                "stderr_sha256": streams["stderr"],
            }
        )
    index = {
        "schema_version": INDEX_SCHEMA,
        "epoch": ctx.packet["epoch"],
        "archive_manifest_sha256": run.sha256((archive / "manifest.json").read_bytes()),
        "packet": ctx.packet,
        "tool_manifest": ctx.tool,
        "dependency_manifest": ctx.dependency,
        "environment_manifest": ctx.environment,
        "provenance": run.canonical_document((archive / "provenance.json").read_bytes()),
        "executions": executions,
    }
    packed = run.canonical_file(index), run.canonical_file({"schema_version": OUTPUTS_SCHEMA, "outputs": outputs})
    with tempfile.TemporaryDirectory(prefix="d004-pack-") as scratch:
        unpack(repository, *packed, Path(scratch) / "archive")
    return packed


def unpack(repository: run.Repository, index_raw: bytes, outputs_raw: bytes, target: Path) -> dict[str, Any]:
    """Rebuild the full archive from its committed form; return the summary."""
    if target.exists():
        raise run.RunError("unpack target already exists")
    try:
        index = run.canonical_document(index_raw)
        outputs_document = run.canonical_document(outputs_raw)
    except (UnicodeError, ValueError) as exc:
        raise run.RunError(f"committed archive is not canonical JSON: {exc}") from exc
    expected = {"schema_version", "epoch", "archive_manifest_sha256", "packet", "provenance", "executions", *(field for field, _ in HOST_CAPTURES)}
    if not isinstance(index, dict) or set(index) != expected or index["schema_version"] != INDEX_SCHEMA:
        raise run.RunError("archive index is not closed")
    if not isinstance(outputs_document, dict) or set(outputs_document) != {"schema_version", "outputs"} or outputs_document["schema_version"] != OUTPUTS_SCHEMA:
        raise run.RunError("adapter outputs are not closed")
    outputs = outputs_document["outputs"]
    packet = index["packet"]
    if packet.get("epoch") != index["epoch"]:
        raise run.RunError("index epoch differs from its packet")
    if packet["runner"] != {"path": run.RUNNER_PATH, "raw_sha256": run.sha256(repository.raw(run.RUNNER_PATH))}:
        raise run.RunError("repository run harness differs from the one the epoch bound")
    for field, path in (("bundle", run.BUNDLE_PATH), ("overlay", run.OVERLAY_PATH)):
        if packet[field] != run.binding(path, repository.raw(path)):
            raise run.RunError(f"repository {field} differs from the one the epoch bound")
    adapter_raw = repository.raw(run.ADAPTER_PATH)
    if index["tool_manifest"]["adapter"]["raw_sha256"] != run.sha256(adapter_raw):
        raise run.RunError("repository adapter differs from the one the epoch ran")
    suite = run.Suite(repository)
    plan = run.epoch_replay_plan(packet, suite)
    schedule = run.schedule_identities(packet, plan, suite, index["tool_manifest"], index["dependency_manifest"], index["environment_manifest"])
    epoch = target / "epoch"
    for name, value in (("packet.json", packet), ("replay-plan.json", plan), ("schedule.json", schedule)):
        run._write(epoch / name, run.canonical_file(value))
    for field, name in HOST_CAPTURES:
        run._write(epoch / name, run.canonical_file(index[field]))
    for candidate in run.CANDIDATES:
        run._write(epoch / "models" / f"{candidate}.json", run.canonical_file(suite.models[candidate]))
    for case in run.CASES:
        run._write(epoch / "inputs" / f"{case}.json", run.canonical_file(suite.manifests[case]))
    for row in schedule[:25]:
        run._write(epoch / "requests" / f"slot-{row['logical_slot_ordinal']:02d}.json", run.canonical_file(run.request_document(suite, packet, plan, row)))
    run._write(target / "tool" / "d004_adapter.py", adapter_raw)
    run._write(target / "provenance.json", run.canonical_file(index["provenance"]))
    ctx = run.EpochContext(target, repository)
    executions = index["executions"]
    if not isinstance(executions, list) or [item.get("execution_ordinal") if isinstance(item, dict) else None for item in executions] != [row["execution_ordinal"] for row in ctx.schedule]:
        raise run.RunError("index executions do not follow the schedule")
    records = []
    for row, item in zip(ctx.schedule, executions):
        if set(item) != set(EXECUTION_FIELDS):
            raise run.RunError(f"execution {row['execution_ordinal']} entry is not closed")
        streams = {}
        for name in ("stdout", "stderr"):
            data = _decode_output(outputs.get(item[f"{name}_sha256"]))
            if run.sha256(data) != item[f"{name}_sha256"]:
                raise run.RunError(f"execution {row['execution_ordinal']} {name} digest mismatch")
            streams[name] = data
        directory = target / "executions" / f"{row['execution_ordinal']:02d}"
        run._write(directory / "stdout", streams["stdout"])
        run._write(directory / "stderr", streams["stderr"])
        record = run.build_record(ctx, row, item["execution_state"], item["measured_resources"], streams["stdout"], streams["stderr"])
        run._write(directory / "record.json", run.canonical_file(record))
        run._write(directory / "diagnostics.json", run.canonical_file(item["diagnostics"]))
        records.append(record)
    used = {item[f"{name}_sha256"] for item in executions for name in ("stdout", "stderr")}
    if set(outputs) != used:
        raise run.RunError("adapter outputs hold unreferenced entries")
    closures, summary = run.summarize(ctx, records)
    run._write(target / "closures.json", run.canonical_file(closures))
    run._write(target / "summary.json", run.canonical_file(summary))
    manifest_raw = run.canonical_file(run.archive_manifest(target))
    if run.sha256(manifest_raw) != index["archive_manifest_sha256"]:
        raise run.RunError("rebuilt archive differs from the archive the run wrote")
    run._write(target / "manifest.json", manifest_raw)
    return summary


def check(repository: run.Repository) -> list[str]:
    """Unpack the committed form into a scratch directory and verify it."""
    with tempfile.TemporaryDirectory(prefix="d004-unpack-") as scratch:
        target = Path(scratch) / "archive"
        try:
            unpack(repository, repository.raw(INDEX_PATH), repository.raw(OUTPUTS_PATH), target)
        except (OSError, run.RunError) as exc:
            return [str(exc)]
        return run.command_verify(repository, target)


def main(argv: list[str]) -> int:
    repository = run.Repository(Path(__file__).resolve().parents[1])
    command = argv[1] if len(argv) > 1 else ""
    try:
        if command == "pack" and len(argv) == 3:
            index_raw, outputs_raw = pack(repository, Path(argv[2]))
            run._write(repository.root / INDEX_PATH, index_raw)
            run._write(repository.root / OUTPUTS_PATH, outputs_raw)
            return 0
        if command == "unpack" and len(argv) == 3:
            unpack(repository, repository.raw(INDEX_PATH), repository.raw(OUTPUTS_PATH), Path(argv[2]))
            return 0
        if command == "check" and len(argv) == 2:
            errors = check(repository)
            for error in errors:
                sys.stderr.write(error + "\n")
            return 1 if errors else 0
    except run.RunError as exc:
        sys.stderr.write(f"d004 archive invalid: {exc}\n")
        return 2
    sys.stderr.write("usage: d004_archive.py pack ARCHIVE | unpack DIRECTORY | check\n")
    return 64


if __name__ == "__main__":
    sys.exit(main(sys.argv))
