"""Check every Orange algorithm entry under algorithms/ against its recorded vectors.

Usage: python3 algorithms/verify.py [PATH ...]

Each PATH is an algorithm folder or a single `.or` file; with none, every
folder under algorithms/ is checked. For each `.or` file the script runs
`orangec check` and `orangec eval`, then pairs every parameterless spec named
`<name>_expected` with the spec `<name>` and requires the two printed values to
agree in type and value. `<name>` computes a result with the algorithm and
`<name>_expected` states the value published with the standard or vector
source that the file's comments cite, so a matching pair is one reproduced
vector. Every file must carry at least one pair, must pass `check` without
diagnostics, and must keep the repository's text rules: LF line endings, a
final newline, no tabs, no trailing whitespace, at most 512 KiB. Every folder
must carry a README.md whose sections include "Analysis" and "Dissemination".

The compiler is found through ORANGEC, then compiler/target/release/orangec,
then compiler/target/debug/orangec. Build one with
`cargo build --manifest-path compiler/Cargo.toml -p orangec --release`.
"""

from __future__ import annotations

import os
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
ALGORITHMS = ROOT / "algorithms"
MAXIMUM_TEXT_BYTES = 512 * 1024
VALUE_LINE = re.compile(r"^([A-Za-z_][A-Za-z0-9_]*)::([A-Za-z_][A-Za-z0-9_]*): (.+?) = (.+)$")


def find_orangec() -> Path:
    candidates = [os.environ.get("ORANGEC")]
    candidates += [ROOT / "compiler/target/release/orangec", ROOT / "compiler/target/debug/orangec"]
    for candidate in candidates:
        if candidate and Path(candidate).is_file() and os.access(candidate, os.X_OK):
            return Path(candidate)
    sys.exit("verify: no orangec binary; set ORANGEC or build compiler/target/release/orangec")


def text_findings(path: Path) -> list[str]:
    findings = []
    data = path.read_bytes()
    if len(data) > MAXIMUM_TEXT_BYTES:
        findings.append(f"{len(data)} bytes exceeds the {MAXIMUM_TEXT_BYTES}-byte text-file cap")
    if b"\r" in data:
        findings.append("carriage return present; use LF line endings")
    if b"\t" in data:
        findings.append("tab present; indent with spaces")
    if not data.endswith(b"\n"):
        findings.append("missing final newline")
    try:
        text = data.decode("utf-8")
    except UnicodeDecodeError:
        return findings + ["not valid UTF-8"]
    for number, line in enumerate(text.split("\n"), start=1):
        if line != line.rstrip(" \t"):
            findings.append(f"line {number}: trailing whitespace")
    return findings


def run(orangec: Path, command: str, path: Path) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        [str(orangec), command, str(path)], capture_output=True, text=True, check=False
    )


def check_source(orangec: Path, path: Path) -> tuple[list[str], list[str]]:
    """Return (findings, reproduced vector names) for one `.or` file."""
    findings = text_findings(path)
    checked = run(orangec, "check", path)
    if checked.returncode != 0 or checked.stderr.strip():
        findings.append("orangec check failed:\n" + (checked.stderr or checked.stdout).rstrip())
        return findings, []
    evaluated = run(orangec, "eval", path)
    if evaluated.returncode != 0 or evaluated.stderr.strip():
        findings.append("orangec eval failed:\n" + (evaluated.stderr or evaluated.stdout).rstrip())
        return findings, []
    values: dict[str, tuple[str, str]] = {}
    for line in evaluated.stdout.splitlines():
        match = VALUE_LINE.match(line)
        if match is None:
            findings.append(f"unrecognized eval line: {line}")
            continue
        _module, name, type_name, value = match.groups()
        values[name] = (type_name, value)
    reproduced = []
    for name, (type_name, value) in sorted(values.items()):
        if not name.endswith("_expected"):
            continue
        computed = name[: -len("_expected")]
        if computed not in values:
            findings.append(f"{name} has no computed twin {computed}")
            continue
        computed_type, computed_value = values[computed]
        if computed_type != type_name:
            findings.append(f"{computed}: type {computed_type} differs from expected {type_name}")
        elif computed_value != value:
            findings.append(f"{computed}: value differs from {name}\n  computed {computed_value}\n  expected {value}")
        else:
            reproduced.append(computed)
    if not reproduced and not findings:
        findings.append("no `<name>` and `<name>_expected` pair; every file states at least one vector")
    return findings, reproduced


def check_readme(folder: Path) -> list[str]:
    readme = folder / "README.md"
    if not readme.is_file():
        return ["README.md is missing"]
    findings = text_findings(readme)
    text = readme.read_text(encoding="utf-8", errors="replace")
    headings = {line.strip("# ").strip() for line in text.splitlines() if line.startswith("#")}
    for required in ("Analysis", "Dissemination"):
        if required not in headings:
            findings.append(f"README.md has no `{required}` section")
    if not text.startswith("# "):
        findings.append("README.md must begin with its title heading")
    return findings


def targets(arguments: list[str]) -> list[Path]:
    if not arguments:
        return sorted(p for p in ALGORITHMS.iterdir() if p.is_dir())
    resolved = []
    for argument in arguments:
        path = Path(argument).resolve()
        if not path.exists():
            sys.exit(f"verify: {argument} does not exist")
        resolved.append(path)
    return resolved


def main(arguments: list[str]) -> int:
    orangec = find_orangec()
    failures = 0
    total_vectors = 0
    for target in targets(arguments):
        sources = [target] if target.is_file() else sorted(target.glob("*.or"))
        folder = target.parent if target.is_file() else target
        findings = check_readme(folder) if target.is_dir() else []
        if target.is_dir() and not sources:
            findings.append("no `.or` source in the folder")
        for source in sources:
            source_findings, reproduced = check_source(orangec, source)
            total_vectors += len(reproduced)
            relative = source.relative_to(ROOT) if source.is_relative_to(ROOT) else source
            for name in reproduced:
                print(f"ok    {relative}: {name}")
            findings.extend(f"{relative}: {finding}" for finding in source_findings)
        for finding in findings:
            failures += 1
            print(f"FAIL  {finding}")
    print(f"{total_vectors} vectors reproduced, {failures} findings")
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
