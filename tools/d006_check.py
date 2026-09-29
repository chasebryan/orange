"""D-006 v0.3 candidate checks (research-only).

Builds a candidate from its adapter, renders every case's check and negative
files with ``d006_render``, runs them with the candidate's toolchain, reads
the diagnostics and trust audits, classifies every failure into the shared
taxonomy with the adapter's classifier, and reports each item's outcome
against the shared expectation.

``python3 d006_check.py dev CANDIDATE_DIR CASE...`` is the development loop:
it runs without isolation or metering and prints a summary. The epoch runner
calls the same functions inside its isolated, metered launcher.
"""

from __future__ import annotations

import json
import os
import re
import shutil
import signal
import subprocess
import sys
import time
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any, Callable

sys.path.insert(0, str(Path(__file__).resolve().parent))
import d006_render as R  # noqa: E402

REPO = Path(os.environ.get("ORANGE_REPO", "/home/claude/orange"))
SHARED = REPO / "research/decisions/D-006/d006-v0.3/shared-inputs"
TOOLCHAINS = {"rocq": "/opt/d006/rocq-9.2.0", "lean4": "/opt/d006/lean-4.34.1-linux"}
CATEGORIES = (
    "parse_failure", "type_failure", "non_total", "proof_failure", "disproved_obligation", "unknown",
    "timeout", "resource_exhaustion", "unsupported_feature", "untrusted_solver_step",
    "failed_certificate", "unmet_target_assumption", "undeclared_trust",
)
NEGATIVE_CEILING = {"wall_seconds": 120, "memory_bytes": 4 << 30, "output_bytes": 1 << 20}
MEASURED_CEILING = {"wall_seconds": 1800, "memory_bytes": 8 << 30, "output_bytes": 64 << 20}
DIAGNOSTIC_LIMIT = 64 * 1024
M01_INSTANCES = {"F-18": (32, 16, 12, 8, 7), "F-19": (8, 4, 3, 2, 1)}


class CheckError(Exception):
    pass


def shared(name: str) -> Any:
    return json.loads((SHARED / name).read_text())


# ---------------------------------------------------------------------------
# Execution (development: plain subprocess with a wall clock and an
# address-space limit; the epoch runner substitutes its isolated launcher)


@dataclass
class Launched:
    argv: list[str]
    exit_status: int | None
    stdout: bytes
    stderr: bytes
    timed_out: bool
    oversized: bool
    wall_ms: int
    memory_exhausted: bool = False

    @property
    def output(self) -> str:
        return (self.stdout + self.stderr).decode("utf-8", "replace")


def _limit(memory: int) -> Callable[[], None]:
    def apply() -> None:
        import resource
        os.setsid()
        resource.setrlimit(resource.RLIMIT_AS, (memory, memory))
    return apply


def run_plain(argv: list[str], cwd: Path, env: dict[str, str], ceiling: dict[str, int]) -> Launched:
    started = time.monotonic()
    proc = subprocess.Popen(argv, cwd=cwd, env=env, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                            stdin=subprocess.DEVNULL, preexec_fn=_limit(ceiling["memory_bytes"]))
    timed_out = False
    try:
        out, err = proc.communicate(timeout=ceiling["wall_seconds"])
    except subprocess.TimeoutExpired:
        timed_out = True
        try:
            os.killpg(proc.pid, signal.SIGKILL)
        except ProcessLookupError:
            pass
        out, err = proc.communicate()
    wall = int((time.monotonic() - started) * 1000)
    oversized = len(out) + len(err) > ceiling["output_bytes"]
    text = (out + err).decode("utf-8", "replace")
    memory = bool(re.search(r"Out of memory|out of memory|std::bad_alloc|Cannot allocate memory|Fatal error: out of memory", text))
    return Launched(argv, None if timed_out else proc.returncode, out, err, timed_out, oversized, wall, memory)


Runner = Callable[[list[str], Path, dict[str, str], dict[str, int]], Launched]


# ---------------------------------------------------------------------------
# Candidate context


@dataclass
class Candidate:
    root: Path              # the candidate's source directory (read-only)
    adapter: dict[str, Any]
    toolchain: str
    work: Path              # writable root for this check
    runner: Runner = run_plain
    extra: dict[str, str] = field(default_factory=dict)  # further toolchain roots, e.g. {toolchain_aarch64}
    lang: R.Language = field(init=False)

    def __post_init__(self) -> None:
        self.lang = R.Language(self.adapter["language"], self.adapter)

    @property
    def src(self) -> Path:
        return self.work / "src"

    def fill(self, text: str, **extra: str) -> str:
        values = {"toolchain": self.toolchain, "src": str(self.src), "work": str(self.work), **self.extra, **extra}
        return re.sub(r"\{(\w+)\}", lambda m: values.get(m.group(1), m.group(0)), text)

    def environment(self, extra: dict[str, str] | None = None) -> dict[str, str]:
        env = {
            "HOME": str(self.work / "home"), "LANG": "C", "LC_ALL": "C", "TZ": "UTC",
            "PATH": f"{self.toolchain}/bin:/usr/bin:/bin", "SOURCE_DATE_EPOCH": "0",
            "TMPDIR": str(self.work / "tmp"),
        }
        for key, value in {**self.adapter.get("environment", {}), **(extra or {})}.items():
            env[key] = self.fill(value)
        return env

    def prepare(self) -> None:
        if self.work.exists():
            shutil.rmtree(self.work)
        for sub in ("home", "tmp", "checks", "audit"):
            (self.work / sub).mkdir(parents=True)
        shutil.copytree(self.root, self.src, ignore=shutil.ignore_patterns(".lake", "*.vo", "*.vok", "*.vos", "*.glob", ".*.aux", "_build"))

    def build(self, mode: str = "serial", ceiling: dict[str, int] = MEASURED_CEILING) -> list[Launched]:
        steps = self.adapter["build"][mode]
        results = []
        for step in steps:
            argv = [self.fill(a, jobs="4" if mode == "declared_parallel" else "1") for a in step]
            launched = self.runner(argv, self.src, self.environment(), ceiling)
            results.append(launched)
            if launched.exit_status != 0:
                break
        return results

    def check(self, path: Path, ceiling: dict[str, int]) -> Launched:
        spec = self.adapter["check"]
        argv = [self.fill(a, file=str(path), audit=str(self.work / "audit")) for a in spec["argv"]]
        return self.runner(argv, path.parent, self.environment(spec.get("environment")), ceiling)


# ---------------------------------------------------------------------------
# Diagnostics


@dataclass
class Message:
    severity: str
    line: int | None
    column: int | None
    text: str


ROCQ_HEADER = re.compile(r'^File "(?P<file>[^"]*)", line (?P<line>\d+), characters (?P<c1>\d+)-(?P<c2>\d+):\n(?P<kind>Error|Warning):?', re.M)


def parse_messages(language: str, launched: Launched) -> list[Message]:
    text = launched.output
    messages: list[Message] = []
    if language == "lean4":
        for raw in text.splitlines():
            raw = raw.strip()
            if not raw.startswith("{"):
                if raw:
                    messages.append(Message("error" if "error" in raw.lower() else "info", None, None, raw))
                continue
            try:
                item = json.loads(raw)
            except json.JSONDecodeError:
                messages.append(Message("info", None, None, raw))
                continue
            pos = item.get("pos") or {}
            messages.append(Message(item.get("severity", "information"), pos.get("line"), pos.get("column"), item.get("data", "")))
        return messages
    heads = list(ROCQ_HEADER.finditer(text))
    consumed = 0
    for index, head in enumerate(heads):
        end = heads[index + 1].start() if index + 1 < len(heads) else len(text)
        body = text[head.end():end].strip()
        pre = text[consumed:head.start()].strip()
        if pre and "Error" in pre:
            messages.append(Message("error", None, None, pre))
        consumed = end
        messages.append(Message("error" if head.group("kind") == "Error" else "warning", int(head.group("line")), int(head.group("c1")), body))
    rest = text[consumed:].strip()
    if rest and re.search(r"Error|Anomaly|Stack overflow|Out of memory", rest):
        messages.append(Message("error", None, None, rest))
    return messages


def errors(language: str, launched: Launched) -> list[Message]:
    return [m for m in parse_messages(language, launched) if m.severity == "error"]


def phase_of(item: R.Item, line: int | None) -> str | None:
    if line is None:
        return None
    for phase, (start, end) in item.phases.items():
        if start <= line <= end:
            return phase
    return None


def classify(adapter: dict[str, Any], launched: Launched, message: Message | None, phase: str | None) -> str | None:
    if launched.timed_out:
        return "timeout"
    if launched.oversized or launched.memory_exhausted:
        return "resource_exhaustion"
    if message is None:
        return None
    for rule in adapter.get("classifier", []):
        phases = rule.get("phases")
        if phases and phase not in phases:
            continue
        if re.search(rule["pattern"], message.text, re.S):
            return rule["category"]
    return None


# ---------------------------------------------------------------------------
# Trust audits


def rocq_audit(path: Path) -> list[str]:
    if not path.exists():
        raise CheckError(f"missing audit {path.name}")
    names = []
    for line in path.read_text().splitlines():
        m = re.match(r"^(\S+) : ", line)
        if m:
            names.append(m.group(1))
    return names


def lean_audits(launched: Launched) -> dict[str, list[str]]:
    found = {}
    for m in parse_messages("lean4", launched):
        a = re.match(r"^'(.+)' depends on axioms: \[(.*)\]$", m.text.strip(), re.S)
        if a:
            found[a.group(1).split(".")[-1]] = [x.strip() for x in a.group(2).split(",") if x.strip()]
        n = re.match(r"^'(.+)' does not depend on any axioms$", m.text.strip())
        if n:
            found[n.group(1).split(".")[-1]] = []
    return found


def audits(cand: Candidate, launched: Launched, names: list[str]) -> dict[str, list[str] | None]:
    if cand.lang.name == "rocq":
        out = {}
        for name in names:
            try:
                out[name] = rocq_audit(cand.work / "audit" / f"audit-{name}.out")
            except CheckError:
                out[name] = None
        return out
    lean = lean_audits(launched)
    return {name: lean.get(name) for name in names}


def undeclared(cand: Candidate, assumptions: list[str] | None) -> list[str]:
    if assumptions is None:
        return ["<no audit>"]
    allowed = set(cand.adapter.get("trust", {}).get("allowed_assumptions", []))
    return [a for a in assumptions if a not in allowed]


# ---------------------------------------------------------------------------
# Case inventories


def _reference() -> Any:
    sys.path.insert(0, str(REPO / "tools"))
    import d006_shared  # noqa: E402  (the plain-Python reference; an oracle for expected values only)
    return d006_shared


def verdict_term(verdict: list[Any]) -> list[Any]:
    if verdict[0] == "accept":
        return ["sym", "B-C03"]
    return ["app", ["sym", "B-C04"], ["str", verdict[1]], ["nat", str(verdict[2])]]


def certificate_observations(prefix: str, certificate: str) -> list[dict[str, Any]]:
    """lrat_verdict observations for a certificate and its V-01 to V-11 mutations."""

    ref = _reference()
    rows = []
    for ident, change in ref.VARIANTS:
        claimed, cnf, text = ref.mutate_certificate(certificate, ident)
        rows.append({
            "id": f"{prefix}-{ident}",
            "note": change,
            "lhs": ["app", ["sym", "B-F02"], ["sym", claimed], ["str", cnf], ["str", text]],
            "rhs": verdict_term(list(ref.lrat_verdict(claimed, cnf, text))),
        })
    return rows


def case_material(case: str) -> dict[str, Any]:
    files = {"DS-01": "ds01-core-fragment.json", "DS-02": "ds02-sieve.json", "DS-03": "ds03-canonical-records.json", "DS-04": "ds04-lrat-obligation.json"}
    data = shared(files[case])
    theorems = data.get("theorems", [])
    observations = list(data.get("observations", []))
    negatives = list(data.get("negatives", [])) + list(data.get("mutations", []))
    if case == "DS-04":
        ref = _reference()
        for index, ob in enumerate(("B-C01", "B-C02"), 1):
            observations.append({"id": f"D4-CNF{index:02d}", "note": f"canonical CNF of {ob}",
                                 "lhs": ["app", ["sym", "B-F01"], ["sym", ob]], "rhs": ["str", ref.cnf_text(ob)]})
        golden = (SHARED / data["golden"]["certificate_path"]).read_text(encoding="ascii")
        observations += certificate_observations("D4-G", golden)
    return {"theorems": theorems, "observations": observations, "negatives": negatives, "data": data}


def instances(cand: Candidate, case: str) -> list[dict[str, Any]]:
    if case != "DS-01":
        return []
    spec = cand.adapter["modules"]["M-01"]
    rows = []
    for fid, params in M01_INSTANCES.items():
        names = dict(zip(("w", "r1", "r2", "r3", "r4"), (cand.lang.nat(str(p)) for p in params)))
        inst = "M01_" + R.ident(fid)
        rows.append({
            "id": fid,
            "setup": R.re.sub(r"\{(\w+)\}", lambda m: {**names, "inst": inst}.get(m.group(1), m.group(0)), spec.get("setup", "")),
            "function": R.re.sub(r"\{(\w+)\}", lambda m: {**names, "inst": inst}.get(m.group(1), m.group(0)), spec["function"]),
        })
    return rows


# ---------------------------------------------------------------------------
# Running one case


@dataclass
class Outcome:
    ident: str
    kind: str
    passed: bool
    category: str | None = None
    detail: str = ""
    location: int | None = None
    diagnostic_conforms: bool | None = None
    wall_ms: int | None = None


def file_name(cand: Candidate, stem: str) -> str:
    return stem + (".v" if cand.lang.name == "rocq" else ".lean")


def write_source(cand: Candidate, directory: Path, source: R.Source) -> Path:
    directory.mkdir(parents=True, exist_ok=True)
    stem = "Check" if cand.lang.name == "rocq" else "Check"
    path = directory / file_name(cand, stem)
    path.write_text(source.text())
    return path


def run_check_source(cand: Candidate, directory: Path, source: R.Source, ceiling: dict[str, int]) -> list[Outcome]:
    path = write_source(cand, directory, source)
    launched = cand.check(path, ceiling)
    errs = errors(cand.lang.name, launched)
    names = [item.declaration for item in source.items if item.declaration]
    found = audits(cand, launched, names)
    outcomes = []
    for item in source.items:
        own = [m for m in errs if m.line is not None and phase_of(item, m.line) is not None]
        if launched.timed_out or launched.oversized or launched.memory_exhausted:
            outcomes.append(Outcome(item.ident, item.kind, False, classify(cand.adapter, launched, None, None), wall_ms=launched.wall_ms))
            continue
        if own:
            m = own[0]
            outcomes.append(Outcome(item.ident, item.kind, False, classify(cand.adapter, launched, m, phase_of(item, m.line)), m.text[:400], m.line))
            continue
        extra = undeclared(cand, found.get(item.declaration))
        if extra:
            reached = found.get(item.declaration) is not None
            detail = f"undeclared: {extra}" if reached else "not reached (an earlier error stopped the file)" if errs else "no audit"
            outcomes.append(Outcome(item.ident, item.kind, False, "undeclared_trust" if reached else None, detail))
            continue
        outcomes.append(Outcome(item.ident, item.kind, True))
    stray = [m for m in errs if m.line is None or not any(phase_of(i, m.line) for i in source.items)]
    if stray and all(o.passed for o in outcomes):
        for o in outcomes:
            o.passed = False
            o.detail = "file error outside any item: " + stray[0].text[:300]
    return outcomes


def run_positive(cand: Candidate, case: str, ceiling: dict[str, int] = MEASURED_CEILING) -> list[Outcome]:
    mat = case_material(case)
    source = R.check_file(cand.lang, case, mat["theorems"], mat["observations"], instances(cand, case))
    outcomes = run_check_source(cand, cand.work / "checks" / case, source, ceiling)
    if all(o.passed for o in outcomes) or cand.lang.name == "lean4":
        return outcomes
    # Rocq stops at its first error: re-check each item on its own.
    single = []
    for item_index, item in enumerate(source.items):
        theorems = [t for t in mat["theorems"] if t["id"] == item.ident]
        observations = [o for o in mat["observations"] if o["id"] == item.ident]
        insts = [i for i in instances(cand, case) if i["id"] == item.ident]
        one = R.check_file(cand.lang, case, theorems, observations, insts)
        single += run_check_source(cand, cand.work / "checks" / case / R.ident(item.ident), one, ceiling)
    return single


def diagnostic_conforms(neg_id: str, path: Path, launched: Launched, message: Message | None, category: str | None) -> bool:
    text = launched.output
    named = neg_id in str(path) or neg_id in (message.text if message else "")
    return bool(named and category in CATEGORIES and (message is not None and message.line is not None or category in ("timeout", "resource_exhaustion")) and len(text.encode()) <= DIAGNOSTIC_LIMIT)


def run_negative(cand: Candidate, case: str, negative: dict[str, Any], ceiling: dict[str, int] = NEGATIVE_CEILING) -> Outcome:
    form = negative["form"]
    if form in ("candidate_patch", "artifact_truncation"):
        return run_patch(cand, case, negative)
    source = R.negative_file(cand.lang, case, negative)
    directory = cand.work / "negatives" / negative["id"]
    path = write_source(cand, directory, source)
    launched = cand.check(path, ceiling)
    item = source.items[0]
    errs = [m for m in errors(cand.lang.name, launched)]
    expected = list(negative["expected"])
    if form == "axiom_use":
        found = audits(cand, launched, [item.declaration])
        extra = undeclared(cand, found.get(item.declaration))
        ok = bool(extra) and extra != ["<no audit>"]
        cat = "undeclared_trust" if ok else None
        return Outcome(negative["id"], form, ok and cat in expected, cat, f"audit reports {extra}", item.phases["audit"][0], ok, launched.wall_ms)
    if launched.timed_out or launched.oversized or launched.memory_exhausted:
        cat = classify(cand.adapter, launched, None, None)
        return Outcome(negative["id"], form, cat in expected, cat, "ceiling", None, diagnostic_conforms(negative["id"], path, launched, None, cat), launched.wall_ms)
    own = [m for m in errs if phase_of(item, m.line)]
    if not own:
        detail = "accepted" if launched.exit_status == 0 and not errs else "failed outside the case: " + (errs[0].text[:300] if errs else launched.output[:300])
        return Outcome(negative["id"], form, False, None, detail, wall_ms=launched.wall_ms)
    m = own[0]
    cat = classify(cand.adapter, launched, m, phase_of(item, m.line))
    return Outcome(negative["id"], form, cat in expected, cat, m.text[:300], m.line, diagnostic_conforms(negative["id"], path, launched, m, cat), launched.wall_ms)


def run_patch(cand: Candidate, case: str, negative: dict[str, Any]) -> Outcome:
    """Candidate patches rebuild a patched copy; affected theorems must stop checking."""

    spec = cand.adapter.get("patches", {}).get(negative["id"])
    if spec is None:
        return Outcome(negative["id"], negative["form"], False, None, "no patch declared")
    patched = Candidate(cand.root, cand.adapter, cand.toolchain, cand.work.parent / (cand.work.name + "-" + negative["id"]), cand.runner, cand.extra)
    patched.prepare()
    if negative["form"] == "artifact_truncation":
        built = patched.build()
        if built[-1].exit_status != 0:
            return Outcome(negative["id"], negative["form"], False, None, "unpatched build failed")
        target = patched.src / spec["artifact"]
        data = target.read_bytes()
        target.write_bytes(data[: len(data) // 2])
        argv = [patched.fill(a, artifact=str(target)) for a in spec["check"]]
        launched = patched.runner(argv, patched.src, patched.environment(), NEGATIVE_CEILING)
        msg = (errors(cand.lang.name, launched) or [Message("error", None, None, launched.output[:2000])])[0]
        cat = classify(cand.adapter, launched, msg, "artifact") if launched.exit_status != 0 else None
        return Outcome(negative["id"], negative["form"], cat in negative["expected"], cat, msg.text[:300], None, None, launched.wall_ms)
    patch_file = cand.root / spec
    applied = subprocess.run(["patch", "-p1", "--no-backup-if-mismatch", "-i", str(patch_file)], cwd=patched.src, capture_output=True)
    if applied.returncode != 0:
        return Outcome(negative["id"], negative["form"], False, None, "patch did not apply: " + applied.stdout.decode()[:300])
    built = patched.build()
    last = built[-1]
    if last.exit_status != 0:
        errs = errors(cand.lang.name, last)
        m = errs[0] if errs else Message("error", None, None, last.output[:2000])
        cat = classify(cand.adapter, last, m, "build")
        conforms = negative["id"] in str(patched.work) and m.line is not None and cat in CATEGORIES and len(last.output.encode()) <= DIAGNOSTIC_LIMIT
        return Outcome(negative["id"], negative["form"], cat in negative["expected"], cat, m.text[:300], m.line, conforms, last.wall_ms)
    outcomes = run_positive(patched, case)
    affected = set(negative.get("affected", []))
    broken = [o for o in outcomes if o.ident in affected and not o.passed]
    if len(broken) == len(affected) and affected:
        cats = {o.category for o in broken}
        cat = broken[0].category
        return Outcome(negative["id"], negative["form"], all(c in negative["expected"] for c in cats), cat, f"affected theorems fail: {sorted(o.ident for o in broken)}")
    return Outcome(negative["id"], negative["form"], False, None, f"affected theorems still check: {sorted(affected - {o.ident for o in broken})}")


# ---------------------------------------------------------------------------
# Development entry point


def load_adapter(root: Path) -> dict[str, Any]:
    adapter = R.load_adapter((root / "adapter.json").read_bytes())
    fragments = root / "adapter.d"
    if fragments.is_dir():
        for path in sorted(fragments.glob("*.json")):
            fragment = json.loads(path.read_text())
            for key in ("schema_version", "language", "candidate"):
                fragment.pop(key, None)
            R.merge_adapter(adapter, fragment)
    return adapter


def load_candidate(root: Path, work: Path) -> Candidate:
    adapter = load_adapter(root)
    return Candidate(root, adapter, TOOLCHAINS[adapter["language"]], work)


def main(argv: list[str]) -> int:
    if len(argv) < 3 or argv[0] != "dev":
        print("usage: d006_check.py dev CANDIDATE_DIR CASE... [--no-build] [--only ID,...] [--negatives-only|--positives-only]", file=sys.stderr)
        return 2
    root = Path(argv[1]).resolve()
    flags = [a for a in argv[2:] if a.startswith("--")]
    cases = [a for a in argv[2:] if not a.startswith("--") and a.startswith("DS-")]
    only = None
    for i, a in enumerate(argv):
        if a == "--only":
            only = set(argv[i + 1].split(","))
    work = Path(os.environ.get("D006_WORK", "/tmp/d006-dev")) / root.name
    cand = load_candidate(root, work)
    if "--no-build" not in flags:
        cand.prepare()
        built = cand.build()
        for b in built:
            if b.exit_status != 0:
                print("BUILD FAILED:", " ".join(b.argv))
                print(b.output[-6000:])
                return 1
        print(f"build ok ({sum(b.wall_ms for b in built)} ms)")
    failures = 0
    for case in cases:
        mat = case_material(case)
        if "--negatives-only" not in flags:
            for o in run_positive(cand, case):
                if only and o.ident not in only:
                    continue
                status = "ok  " if o.passed else "FAIL"
                failures += not o.passed
                print(f"{status} {o.ident:10} {o.kind:12} {o.category or ''} {o.detail[:300]}")
        if "--positives-only" not in flags:
            for neg in mat["negatives"]:
                if only and neg["id"] not in only:
                    continue
                o = run_negative(cand, case, neg)
                status = "ok  " if o.passed else "FAIL"
                failures += not o.passed
                print(f"{status} {o.ident:10} {o.kind:20} expected={neg.get('expected')} got={o.category} diag={o.diagnostic_conforms} {o.wall_ms}ms {o.detail[:200]!r}")
    print(f"{failures} failure(s)")
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
