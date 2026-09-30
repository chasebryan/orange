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
TOOLCHAINS_AARCH64 = {"rocq": "/opt/d006/ocaml-4.14.1-aarch64", "lean4": "/opt/d006/lean-4.34.1-linux_aarch64"}
DEV_SOLVER = "/opt/d006/cadical-src/build/cadical"
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


def undeclared(cand: Candidate, assumptions: list[str] | None, case: str) -> list[str]:
    if assumptions is None:
        return ["<no audit>"]
    trust = cand.adapter.get("trust", {})
    allowed = set(trust.get("allowed_assumptions", []))
    # Tools that name one assumption per use (Lean's native_decide) are declared by one pattern
    # that must match the whole name, never by listing case ids. A row with `cases` applies to
    # those cases only, so a widening declared for one case cannot pass another's audit.
    patterns = [re.compile(row["pattern"]) for row in trust.get("allowed_assumption_patterns", [])
                if case in row.get("cases", [case])]
    return [a for a in assumptions if a not in allowed and not any(p.fullmatch(a) for p in patterns)]


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


# ---------------------------------------------------------------------------
# DS-04 run-time cases D4-R02 to D4-R05: what a solver run lets the lab claim

SOLVER_CNF = {"B-C01": "ds04-carry-save.cnf", "B-C02": "ds04-carry-save-unshifted.cnf"}


def solver_argv(solver: str, key: str, cnf: str, certificate: str) -> list[str]:
    """The pinned argv (`argv` or `unknown_argv`) with the solver, CNF and certificate paths filled in."""

    spec = shared("ds04-lrat-obligation.json")["solver"]
    fill = {spec["tool"]: solver, "{cnf}": cnf, "{certificate}": certificate}
    return [fill.get(a, a) for a in spec[key]]


def solver_claim(state: str, exit_status: int | None, certificate_present: bool) -> str:
    """The claim a solver run supports before any certificate is checked.

    `certificate` means an unsatisfiability claim whose certificate the candidate must now check;
    every other value is a taxonomy category that leaves the obligation unproved.
    """

    if not isinstance(state, str):
        raise TypeError(f"solver_claim takes a step state kind, not {type(state).__name__}")
    if state == "timeout":
        return "timeout"
    if state in ("resource_exhaustion", "oversized_output"):
        return "resource_exhaustion"
    codes = shared("ds04-lrat-obligation.json")["solver"]["exit_codes"]
    result = codes.get(str(exit_status)) if state in ("completed", "failed") else None
    if result == "satisfiable":
        return "disproved_obligation"
    if result == "unsatisfiable":
        return "certificate" if certificate_present else "failed_certificate"
    return "unknown"


def solver_model(stdout: bytes) -> dict[int, bool] | None:
    """The `v` lines of a satisfiable run as variable -> value, or None when there are none."""

    model: dict[int, bool] = {}
    for line in stdout.decode("ascii", "replace").splitlines():
        if line.startswith("v "):
            for token in line[2:].split():
                literal = int(token)
                if literal:
                    model[abs(literal)] = literal > 0
    return model or None


def model_words(model: dict[int, bool]) -> tuple[int, int]:
    """x and y from a model: bit i of x is variable 2 + i and of y is variable 2 + width + i."""

    width = shared("ds04-lrat-obligation.json")["width"]
    x = sum(1 << i for i in range(width) if model.get(2 + i))
    y = sum(1 << i for i in range(width) if model.get(2 + width + i))
    return x, y


def counterexample_observations(x: int, y: int) -> tuple[list[dict[str, Any]], int, int]:
    """Both sides of B-C02 at (x, y) as observations, with the values the reference computes."""

    width = shared("ds04-lrat-obligation.json")["width"]
    w = ["nat", str(width)]
    mask = (1 << width) - 1

    def word(n: int) -> list[Any]:
        return ["app", ["sym", "F-01"], w, ["nat", str(n)]]

    wx, wy = word(x), word(y)
    left, right = (x + y) & mask, ((x ^ y) + (x & y)) & mask
    rows = [
        {"id": "D4-R02-L", "note": "left side of B-C02 at the solver's model",
         "lhs": ["app", ["sym", "F-03"], w, wx, wy], "rhs": word(left)},
        {"id": "D4-R02-R", "note": "right side of B-C02 at the solver's model",
         "lhs": ["app", ["sym", "F-03"], w, ["app", ["sym", "F-04"], w, wx, wy], ["app", ["sym", "F-05"], w, wx, wy]],
         "rhs": word(right)},
    ]
    return rows, left, right


def missing_certificate_observation() -> dict[str, Any]:
    """An absent certificate is read as empty text; the candidate's checker must not accept it."""

    ref = _reference()
    cnf = ref.cnf_text("B-C01")
    return {"id": "D4-R05-E", "note": "the absent certificate read as empty text",
            "lhs": ["app", ["sym", "B-F02"], ["sym", "B-C01"], ["str", cnf], ["str", ""]],
            "rhs": verdict_term(list(ref.lrat_verdict("B-C01", cnf, "")))}


def run_time_verdict(ident: str, claim: str, observations: list[Outcome], left: int | None = None, right: int | None = None) -> Outcome:
    """One run-time case as a negative outcome: the exact category, and the candidate's part where it has one."""

    expected = {row["id"]: row for row in shared("ds04-lrat-obligation.json")["run_time_cases"]}[ident]
    category = {"D4-R02": "disproved_obligation", "D4-R03": "unknown", "D4-R04": "timeout", "D4-R05": "failed_certificate"}[ident]
    failed = [o.ident for o in observations if not o.passed]
    ok = claim == category and not failed
    detail = f"claim {claim}"
    if ident == "D4-R02":
        ok = ok and left is not None and right is not None and left != right and len(observations) == 2
        detail += f"; the prover computes {left} and {right} for the two sides" if left is not None else "; no model"
    if ident == "D4-R05":
        ok = ok and len(observations) == 1
    if failed:
        detail += f"; failed in the prover: {failed}"
    return Outcome(ident, "run_time", ok, claim if claim != "certificate" else None, f"{expected['name']}: {detail}")


Solve = Callable[[str, str, str, bool], tuple[str, "int | None", bytes, Path]]


def run_time_cases(solve: Solve, check: Callable[[list[dict[str, Any]]], list[Outcome]]) -> list[Outcome]:
    """D4-R02 to D4-R05 (D4-R01 is the fresh certificate).

    ``solve(ident, argv_key, obligation, zero_wall)`` runs the pinned solver on the obligation's
    shared CNF (which D4-CNF01 and D4-CNF02 prove equal to the candidate's cnf_text) and returns its
    state, exit status, stdout and certificate path; ``check`` proves observations in the candidate.
    """

    outcomes = []
    state, code, stdout, certificate = solve("D4-R02", "argv", "B-C02", False)
    claim = solver_claim(state, code, certificate.is_file())
    model = solver_model(stdout) if claim == "disproved_obligation" else None
    if model:
        rows, left, right = counterexample_observations(*model_words(model))
        outcomes.append(run_time_verdict("D4-R02", claim, check(rows), left, right))
    else:
        outcomes.append(run_time_verdict("D4-R02", claim, []))
    state, code, _, certificate = solve("D4-R03", "unknown_argv", "B-C01", False)
    outcomes.append(run_time_verdict("D4-R03", solver_claim(state, code, certificate.is_file()), []))
    state, code, _, certificate = solve("D4-R04", "argv", "B-C01", True)
    outcomes.append(run_time_verdict("D4-R04", solver_claim(state, code, certificate.is_file()), []))
    state, code, _, certificate = solve("D4-R05", "argv", "B-C01", False)
    certificate.unlink(missing_ok=True)  # the lab loses the solver's output
    outcomes.append(run_time_verdict("D4-R05", solver_claim(state, code, certificate.is_file()), check([missing_certificate_observation()])))
    return outcomes


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


def run_check_source(cand: Candidate, case: str, directory: Path, source: R.Source, ceiling: dict[str, int]) -> list[Outcome]:
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
        extra = undeclared(cand, found.get(item.declaration), case)
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


def check_observations(cand: Candidate, case: str, directory: Path, observations: list[dict[str, Any]], ceiling: dict[str, int] = MEASURED_CEILING) -> list[Outcome]:
    """Check extra observations of a case, re-checking items singly after a Rocq stop."""

    source = R.check_file(cand.lang, case, [], observations, [])
    outcomes = run_check_source(cand, case, directory, source, ceiling)
    if all(o.passed for o in outcomes) or cand.lang.name == "lean4":
        return outcomes
    single = []
    for row in observations:
        one = R.check_file(cand.lang, case, [], [row], [])
        single += run_check_source(cand, case, directory / R.ident(row["id"]), one, ceiling)
    return single


def run_positive(cand: Candidate, case: str, ceiling: dict[str, int] = MEASURED_CEILING) -> list[Outcome]:
    mat = case_material(case)
    source = R.check_file(cand.lang, case, mat["theorems"], mat["observations"], instances(cand, case))
    outcomes = run_check_source(cand, case, cand.work / "checks" / case, source, ceiling)
    if all(o.passed for o in outcomes) or cand.lang.name == "lean4":
        return outcomes
    # Rocq stops at its first error: re-check each item on its own.
    single = []
    for item_index, item in enumerate(source.items):
        theorems = [t for t in mat["theorems"] if t["id"] == item.ident]
        observations = [o for o in mat["observations"] if o["id"] == item.ident]
        insts = [i for i in instances(cand, case) if i["id"] == item.ident]
        one = R.check_file(cand.lang, case, theorems, observations, insts)
        single += run_check_source(cand, case, cand.work / "checks" / case / R.ident(item.ident), one, ceiling)
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
        extra = undeclared(cand, found.get(item.declaration), case)
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
        # An artifact's location is its file: the diagnostic must name the truncated file (M-15).
        conforms = (negative["id"] in str(target) and cat in CATEGORIES and str(target) in launched.output
                    and len(launched.output.encode()) <= DIAGNOSTIC_LIMIT)
        return Outcome(negative["id"], negative["form"], cat in negative["expected"], cat, msg.text[:300], None, conforms, launched.wall_ms)
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
    return Candidate(root, adapter, TOOLCHAINS[adapter["language"]], work, extra={"toolchain_aarch64": TOOLCHAINS_AARCH64[adapter["language"]]})


# ---------------------------------------------------------------------------
# DS-05: the standalone checker's corpus (shared by the dev loop and the epoch runner)


def verdict_text(verdict: tuple[Any, ...]) -> str:
    return "accept" if verdict[0] == "accept" else f"reject {verdict[1]} {verdict[2]}"


def standalone_corpus(root: Path, fresh: str | None) -> list[dict[str, Any]]:
    """Every DS-05 corpus item with its argv tail, expected output and in-prover twin."""

    ref = _reference()
    root.mkdir(parents=True, exist_ok=True)
    items: list[dict[str, Any]] = []
    expected_lines = {row["fixture"]: row["standalone"] for row in case_material("DS-03")["observations"] if "fixture" in row}
    for ident, data, _ in ref.record_fixtures():
        path = root / f"{ident}.ocr"
        path.write_bytes(data)
        items.append({"id": f"D5-{ident}", "args": ["records", str(path)], "expected": expected_lines[ident], "in_prover": ident.replace("R-F", "D3-O")})
    for index, obligation in enumerate(("B-C01", "B-C02"), 1):
        items.append({"id": f"D5-CNF-{obligation}", "args": ["cnf", obligation], "expected": ref.cnf_text(obligation).rstrip("\n"), "in_prover": f"D4-CNF{index:02d}", "multiline": True})
    golden = (SHARED / "ds04-carry-save-golden.lrat").read_text(encoding="ascii")
    for label, certificate, prefix in (("G", golden, "D4-G"), ("F", fresh, "D4-F")):
        if certificate is None:
            continue
        for variant, _ in ref.VARIANTS:
            claimed, cnf, text = ref.mutate_certificate(certificate, variant)
            cnf_path, cert_path = root / f"{label}-{variant}.cnf", root / f"{label}-{variant}.lrat"
            cnf_path.write_text(cnf, encoding="ascii")
            cert_path.write_text(text, encoding="ascii")
            items.append({"id": f"D5-{label}-{variant}", "args": ["lrat", claimed, str(cnf_path), str(cert_path)],
                          "expected": verdict_text(ref.lrat_verdict(claimed, cnf, text)), "in_prover": f"{prefix}-{variant}"})
    for path in root.iterdir():
        path.chmod(0o444)
    return items


def dev_standalone(cand: Candidate, hosts: set[str] | None, fresh: bool, only: set[str] | None) -> int:
    """Build each host's checker and run the corpus directly (no sandbox; the epoch adds it)."""

    spec = cand.adapter.get("standalone")
    if not spec:
        print("FAIL no standalone entry in the adapter")
        return 1
    corpus = cand.work / "corpus"
    shutil.rmtree(corpus, ignore_errors=True)
    certificate = None
    if fresh:
        corpus.mkdir(parents=True)
        cnf = corpus.parent / "fresh.cnf"
        cnf.write_text(_reference().cnf_text("B-C01"), encoding="ascii")
        done = subprocess.run(solver_argv(DEV_SOLVER, "argv", str(cnf), str(corpus.parent / "fresh.lrat")), capture_output=True, check=False)
        certificate = (corpus.parent / "fresh.lrat").read_text(encoding="ascii") if done.returncode == 20 else None
        print(f"{'ok  ' if certificate else 'FAIL'} fresh certificate (solver exit {done.returncode})")
    items = standalone_corpus(corpus, certificate)
    failures = 0
    for host, host_spec in spec.get("hosts", {}).items():
        if hosts and host not in hosts:
            continue
        built = True
        for step in host_spec.get("build", []):
            launched = cand.runner([cand.fill(a, jobs="1") for a in step], cand.src, cand.environment(host_spec.get("build_environment")), MEASURED_CEILING)
            if launched.exit_status != 0:
                print(f"FAIL {host} build: {' '.join(launched.argv)}\n{launched.output[-4000:]}")
                built = False
                break
        binary = cand.src / host_spec.get("binary", "")
        if not built or not binary.is_file():
            failures += 1
            continue
        print(f"ok   {host} build: {binary.relative_to(cand.src)} ({binary.stat().st_size} bytes)")
        launch = [cand.fill(a).replace("{binary}", str(binary)) for a in host_spec["launch"]]
        env = {"PATH": "/usr/bin:/bin", "LANG": "C", "LC_ALL": "C", "TZ": "UTC", "HOME": str(cand.work), "TMPDIR": str(cand.work),
               **{k: cand.fill(v) for k, v in host_spec.get("environment", {}).items()}}
        for item in items:
            if only and item["id"] not in only:
                continue
            started = time.monotonic()
            done = subprocess.run(launch + item["args"], capture_output=True, env=env, timeout=600, check=False)
            out = done.stdout.decode("utf-8", "replace")
            got = out.rstrip("\n") if item.get("multiline") else out.strip()
            ok = done.returncode == 0 and got == item["expected"]
            failures += not ok
            shown = "(canonical CNF)" if item.get("multiline") and ok else repr(got[:120])
            print(f"{'ok  ' if ok else 'FAIL'} {host} {item['id']:14} {int((time.monotonic() - started) * 1000)}ms expected={item['expected'][:60]!r} got={shown}")
        usage = subprocess.run(launch, capture_output=True, env=env, timeout=60, check=False)
        ok = usage.returncode == 2 and bool(usage.stderr.strip()) and not usage.stdout
        failures += not ok
        print(f"{'ok  ' if ok else 'FAIL'} {host} usage (exit {usage.returncode})")
    return failures


def dev_solve(cand: Candidate) -> Solve:
    """Run the development solver directly (the epoch runs the pinned archive in the sandbox)."""

    def solve(ident: str, key: str, obligation: str, zero_wall: bool) -> tuple[str, int | None, bytes, Path]:
        out = cand.work / "solver" / ident
        shutil.rmtree(out, ignore_errors=True)
        out.mkdir(parents=True)
        certificate = out / "certificate.lrat"
        argv = solver_argv(DEV_SOLVER, key, str(SHARED / SOLVER_CNF[obligation]), str(certificate))
        try:
            done = subprocess.run(argv, capture_output=True, check=False, timeout=0 if zero_wall else 600)
        except subprocess.TimeoutExpired:
            return "timeout", None, b"", certificate
        return ("completed" if done.returncode == 0 else "failed"), done.returncode, done.stdout, certificate

    return solve


def main(argv: list[str]) -> int:
    if len(argv) < 3 or argv[0] != "dev":
        print("usage: d006_check.py dev CANDIDATE_DIR CASE... [--no-build] [--only ID,...] [--negatives-only|--positives-only]\n"
              "       (DS-04 --run-time also runs D4-R02 to D4-R05; DS-05 builds and runs the standalone checker: [--host H-01|H-02] [--fresh])", file=sys.stderr)
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
        if case == "DS-05":
            hosts = {argv[i + 1] for i, a in enumerate(argv) if a == "--host"} or None
            failures += dev_standalone(cand, hosts, "--fresh" in flags, only)
            continue
        mat = case_material(case)
        if "--negatives-only" not in flags:
            for o in run_positive(cand, case):
                if only and o.ident not in only:
                    continue
                status = "ok  " if o.passed else "FAIL"
                failures += not o.passed
                print(f"{status} {o.ident:10} {o.kind:12} {o.category or ''} {o.detail[:300]}")
        if case == "DS-04" and "--run-time" in flags:
            for o in run_time_cases(dev_solve(cand), lambda rows: check_observations(cand, case, cand.work / "checks" / "DS-04-run-time", rows)):
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
