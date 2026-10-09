"""Check every Orange algorithm entry under algorithms/ against its published vectors.

Usage: python3 algorithms/verify.py [PATH ...]

Each PATH is an algorithm folder or a single `.or` file; with none, every
folder under algorithms/ is checked. For each `.or` file the script runs
`orangec check`, which must pass without diagnostics, and then one or both of:

* `orangec test`, when the file declares `test "..." { ... }` blocks. Every
  test states one published value, as the title cites it (the document, the
  section or the case), and claims that the algorithm reproduces it. Every
  test must pass; a file with tests must hold at least one.
* `orangec eval`, when the file declares parameterless specs named
  `<name>_expected`. Each is paired with the spec `<name>`, and the two printed
  values must agree in type and value. This is the form the first entries used
  before Orange had test blocks; new entries write tests.

A file with neither tests nor pairs passes only when a sibling file in the
same folder names it in a `use` declaration, that is, when it is a module of a
larger program whose root carries the tests. Both commands run with the
largest step budget `orangec` admits, so an entry's cost is what its README
records, not what the gate allows; the evaluator terminates on every program,
and wall-clock time stays under a second per entry.

Every file must keep the repository's text rules: LF line endings, a final
newline, no tabs, no trailing whitespace, at most 512 KiB. Every folder must
carry a README.md whose sections include "Analysis" and "Dissemination".

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
STEP_BUDGET = 1_073_741_824
VALUE_LINE = re.compile(r"^([A-Za-z_][A-Za-z0-9_]*)::([A-Za-z_][A-Za-z0-9_]*): (.+?) = (.+)$")
TEST_LINE = re.compile(r'^test "(.*)" \.\.\. (ok|FAILED)$')
SUMMARY_LINE = re.compile(r"^(\d+) tests?: (\d+) passed, (\d+) failed$")
TEST_DECLARATION = re.compile(r'^\s*test\s+"', re.MULTILINE)
EXPECTED_DECLARATION = re.compile(r"^\s*spec\s+[A-Za-z_][A-Za-z0-9_]*_expected\s*\(", re.MULTILINE)
USE_DECLARATION = re.compile(r"^\s*use\s+([a-z0-9]+)\s*;", re.MULTILINE)


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


def run(orangec: Path, arguments: list[str], path: Path) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        [str(orangec), *arguments, str(path)], capture_output=True, text=True, check=False
    )


def used_by_sibling(path: Path) -> bool:
    """Whether another `.or` file beside `path` declares `use <stem>;`."""
    for sibling in path.parent.glob("*.or"):
        if sibling == path:
            continue
        text = sibling.read_text(encoding="utf-8", errors="replace")
        if path.stem in USE_DECLARATION.findall(text):
            return True
    return False


def run_tests(orangec: Path, path: Path) -> tuple[list[str], list[str]]:
    """Return (findings, titles of passed tests) from `orangec test`."""
    tested = run(orangec, ["test", "--steps", str(STEP_BUDGET)], path)
    if tested.stderr.strip():
        return ["orangec test failed:\n" + tested.stderr.rstrip()], []
    findings: list[str] = []
    passed: list[str] = []
    lines = tested.stdout.splitlines()
    index = 0
    while index < len(lines):
        line = lines[index]
        index += 1
        match = TEST_LINE.match(line)
        if match is not None:
            title, outcome = match.groups()
            if outcome == "ok":
                passed.append(title)
                continue
            detail = []
            while index < len(lines) and lines[index].startswith("    "):
                detail.append(lines[index])
                index += 1
            findings.append(f'test "{title}" failed' + ("\n" + "\n".join(detail) if detail else ""))
            continue
        summary = SUMMARY_LINE.match(line)
        if summary is None:
            findings.append(f"unrecognized test line: {line}")
            continue
        total, _passed, failed = (int(group) for group in summary.groups())
        if total == 0:
            findings.append("no test ran")
        if failed and not findings:
            findings.append(f"{failed} tests failed")
    if tested.returncode != 0 and not findings:
        findings.append(f"orangec test exited with {tested.returncode}:\n" + tested.stdout.rstrip())
    return findings, passed


def run_pairs(orangec: Path, path: Path) -> tuple[list[str], list[str]]:
    """Return (findings, reproduced vector names) from the `<name>_expected` pairs."""
    evaluated = run(orangec, ["eval", "--steps", str(STEP_BUDGET)], path)
    if evaluated.returncode != 0 or evaluated.stderr.strip():
        return ["orangec eval failed:\n" + (evaluated.stderr or evaluated.stdout).rstrip()], []
    findings: list[str] = []
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
        findings.append("no `<name>` and `<name>_expected` pair evaluated")
    return findings, reproduced


def check_source(orangec: Path, path: Path) -> tuple[list[str], list[str], list[str]]:
    """Return (findings, passed test titles, reproduced pair names) for one `.or` file."""
    findings = text_findings(path)
    checked = run(orangec, ["check"], path)
    if checked.returncode != 0 or checked.stderr.strip():
        findings.append("orangec check failed:\n" + (checked.stderr or checked.stdout).rstrip())
        return findings, [], []
    text = path.read_text(encoding="utf-8", errors="replace")
    passed: list[str] = []
    reproduced: list[str] = []
    if TEST_DECLARATION.search(text):
        test_findings, passed = run_tests(orangec, path)
        findings.extend(test_findings)
    if EXPECTED_DECLARATION.search(text):
        pair_findings, reproduced = run_pairs(orangec, path)
        findings.extend(pair_findings)
    if not passed and not reproduced and not findings and not used_by_sibling(path):
        findings.append(
            "no `test` block, no `<name>_expected` pair, and no sibling `use`s it; "
            "every file states at least one vector or serves a root that does"
        )
    return findings, passed, reproduced


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
    total_tests = 0
    total_pairs = 0
    for target in targets(arguments):
        sources = [target] if target.is_file() else sorted(target.glob("*.or"))
        folder = target.parent if target.is_file() else target
        findings = check_readme(folder) if target.is_dir() else []
        if target.is_dir() and not sources:
            findings.append("no `.or` source in the folder")
        for source in sources:
            source_findings, passed, reproduced = check_source(orangec, source)
            total_tests += len(passed)
            total_pairs += len(reproduced)
            relative = source.relative_to(ROOT) if source.is_relative_to(ROOT) else source
            for title in passed:
                print(f'ok    {relative}: test "{title}"')
            for name in reproduced:
                print(f"ok    {relative}: {name}")
            findings.extend(f"{relative}: {finding}" for finding in source_findings)
        for finding in findings:
            failures += 1
            print(f"FAIL  {finding}")
    print(f"{total_tests} tests passed, {total_pairs} vector pairs reproduced, {failures} findings")
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
