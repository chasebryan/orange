"""D-006 v0.3 epoch runner (research-only).

Runs the preregistered D-006 protocol for the two candidates, C-01 Rocq and
C-02 Lean 4, against the frozen shared inputs, and archives every record.

* ``prepare ARCHIVE_ROOT`` binds the shared-input manifest, the suite overlay,
  the toolchain record, this runner, the harness and the sandbox by digest,
  derives the epoch identity, captures the host and writes the content-
  addressed toolchain archives the workspaces unpack.
* ``execute ARCHIVE`` provisions workspace W1, runs the cold bootstraps, the
  deterministic replays (serial, then declared parallel), the timed replays and
  the fault injections in the preregistered alternating order, then provisions
  W2 and recreates the deterministic manifests there.
* ``summarize ARCHIVE`` derives every metric, hard gate, materiality label and
  the suite conclusion from the archived records alone.
* ``verify ARCHIVE`` re-checks every archived file against the archive
  manifest and recomputes the summary.

Every candidate process runs through ``tools/fs_sandbox.c`` (Landlock) inside
fresh user, mount, PID, IPC, UTS and network namespaces with no capabilities,
an allowlisted environment, a per-step cgroup (memory, pids, cpuacct) and a
monotonic wall clock. The runner itself (rendering check files, applying
patches, comparing outputs) runs outside the sandbox and is bound by digest.
"""

from __future__ import annotations

import base64
import fnmatch
import hashlib
import json
import math
import os
import random
import re
import selectors
import shutil
import signal
import statistics
import subprocess
import sys
import tarfile
import time
from dataclasses import asdict, dataclass, field
from pathlib import Path
from typing import Any, Callable, Iterable

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import d006_check as H  # noqa: E402
import d006_render as R  # noqa: E402

SUITE_VERSION = "d006-v0.3"
LAB = "research/decisions/D-006/d006-v0.3"
SHARED_DIR = f"{LAB}/shared-inputs"
OVERLAY_PATH = f"{LAB}/protocol/suite-overlay.json"
TOOLCHAINS_PATH = f"{LAB}/protocol/toolchains.json"
CANDIDATE_DIRS = {"C-01": f"{LAB}/rocq", "C-02": f"{LAB}/lean4"}
LANGUAGE = {"C-01": "rocq", "C-02": "lean4"}
NAMES = {"C-01": "Rocq", "C-02": "Lean 4"}
BOUND_TOOLS = (
    "tools/d006_run.py", "tools/d006_check.py", "tools/d006_render.py",
    "tools/d006_shared.py", "tools/fs_sandbox.c",
)
CASES = ("DS-01", "DS-02", "DS-03", "DS-04", "DS-05", "DS-06", "DS-07")
PROOF_CASES = ("DS-01", "DS-02", "DS-03", "DS-04")
TIMED_CASES = ("DS-01", "DS-02", "DS-03", "DS-04", "DS-05", "DS-06")
ARCHIVE_ROOT = Path("/tmp/orange-d006")
RECORD_SCHEMA = "d006-v0.3-record-1"

# Namespaces and privileges follow the D-004 launcher; the sandbox binary is
# built from the unchanged tools/fs_sandbox.c with the same flags.
# Candidate processes run as a dedicated unprivileged lab user, so the
# sandbox's per-user process cap counts only the step's own processes.
LAB_UID = 60606
IDENTITY = ("/usr/bin/setpriv", f"--reuid={LAB_UID}", f"--regid={LAB_UID}", "--clear-groups")
NAMESPACE = (
    "/usr/bin/unshare", "--user", "--map-root-user", "--mount", "--ipc", "--uts",
    "--pid", "--fork", "--kill-child=KILL", "--mount-proc", "--net",
)
PRIVILEGES = (
    "/usr/bin/setpriv", "--bounding-set=-all", "--inh-caps=-all",
    "--ambient-caps=-all", "--no-new-privs",
)
INIT = ("/bin/sh", "-c", '"$@"; exit $?', "d006-init")
SANDBOX_SOURCE = "tools/fs_sandbox.c"
SANDBOX_COMPILER = "/usr/bin/cc"
SANDBOX_FLAGS = (
    "-std=c17", "-O2", "-D_FORTIFY_SOURCE=3", "-fPIE", "-pie", "-Wall", "-Wextra",
    "-Werror", "-pedantic", "-Wl,-z,relro,-z,now",
)
# The sandbox's own per-process caps, recorded in every epoch: they hold for
# both candidates in addition to the per-step ceilings below.
SANDBOX_CAPS = {
    "address_space_bytes": 4 << 30, "cpu_seconds": 600, "file_bytes": 512 << 20,
    "open_files": 1024, "processes": 256, "core_bytes": 0,
}
CEILINGS = {
    "measured_step": {"wall_seconds": 1800, "memory_bytes": 8 << 30, "pids": 4096, "temp_bytes": 8 << 30, "output_bytes": 64 << 20},
    "negative_case": {"wall_seconds": 120, "memory_bytes": 4 << 30, "pids": 1024, "temp_bytes": 1 << 30, "output_bytes": 1 << 20},
    "timed_replay_step": {"wall_seconds": 600, "memory_bytes": 8 << 30, "pids": 4096, "temp_bytes": 8 << 30, "output_bytes": 64 << 20},
    # D4-R04: the pinned solver under a wall ceiling of 0 seconds
    "solver_zero_wall": {"wall_seconds": 0, "memory_bytes": 8 << 30, "pids": 4096, "temp_bytes": 8 << 30, "output_bytes": 64 << 20},
}
LAUNCH_FAILURES = (b"orange filesystem sandbox failed", b"unshare:", b"setpriv:", b"/usr/bin/env:")
# The sandbox's last stage is execv of the step's own command; its failure is the step's, not the launcher's.
COMMAND_EXEC_FAILURE = b"orange filesystem sandbox failed at execute "
CPUS = {"serial": (0,), "declared_parallel": (0, 1, 2, 3)}
JOBS = {"serial": "1", "declared_parallel": "4"}


class RunError(Exception):
    """A runner or environment defect: the epoch is invalid, not a candidate result."""


# ---------------------------------------------------------------------------
# Canonical JSON and digests


def canonical(value: Any) -> bytes:
    return json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=True).encode("ascii")


def canonical_file(value: Any) -> bytes:
    return canonical(value) + b"\n"


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def digest(value: Any) -> str:
    return sha256(canonical(value))


def file_sha256(path: Path) -> str:
    h = hashlib.sha256()
    with open(path, "rb") as handle:
        for block in iter(lambda: handle.read(1 << 20), b""):
            h.update(block)
    return h.hexdigest()


def tree_manifest(root: Path, patterns: Iterable[str] | None = None, exclude: Callable[[str], bool] | None = None) -> list[dict[str, Any]]:
    """Ordered (path, mode, size, sha256) rows for regular files under root."""

    rows = []
    paths: set[Path] = set()
    if patterns is None:
        paths = {p for p in root.rglob("*") if p.is_file() and not p.is_symlink()}
    else:
        for pattern in patterns:
            paths |= {p for p in root.glob(pattern) if p.is_file() and not p.is_symlink()}
    for path in sorted(paths, key=lambda p: p.relative_to(root).as_posix()):
        rel = path.relative_to(root).as_posix()
        if exclude and exclude(rel):
            continue
        info = path.stat()
        rows.append({"path": rel, "mode": oct(info.st_mode & 0o777), "size": info.st_size, "sha256": file_sha256(path)})
    return rows


# ---------------------------------------------------------------------------
# Launcher: namespaces, Landlock sandbox, cgroup meter, wall clock


def build_sandbox(repo: Path, work: Path) -> tuple[Path, dict[str, Any]]:
    source = repo / SANDBOX_SOURCE
    binary = work / "fs-sandbox"
    command = [SANDBOX_COMPILER, *SANDBOX_FLAGS, str(source), "-o", str(binary)]
    done = subprocess.run(command, env={"LANG": "C", "LC_ALL": "C", "PATH": "/usr/bin:/bin", "TZ": "UTC"}, capture_output=True, check=False)
    if done.returncode != 0 or not binary.is_file():
        raise RunError(f"sandbox build failed: {done.stderr.decode('utf-8', 'replace')}")
    version = subprocess.run([SANDBOX_COMPILER, "--version"], capture_output=True, check=True)
    return binary, {
        "source": {"path": SANDBOX_SOURCE, "sha256": file_sha256(source)},
        "compiler": {"path": SANDBOX_COMPILER, "realpath": os.path.realpath(SANDBOX_COMPILER),
                     "sha256": file_sha256(Path(os.path.realpath(SANDBOX_COMPILER))),
                     "version": version.stdout.decode("utf-8", "replace").splitlines()[0]},
        "flags": list(SANDBOX_FLAGS),
        "binary_sha256": file_sha256(binary),
        "caps": SANDBOX_CAPS,
    }


class CgroupMeter:
    """Per-step cgroup-v1 memory, pids and cpuacct groups holding the whole process tree."""

    kind = "cgroup-v1-memory-pids-cpuacct"

    def __init__(self, ceiling: dict[str, int], cpus: tuple[int, ...]) -> None:
        self.groups: dict[str, Path] = {}
        self.cpus = cpus
        own: dict[str, str] = {}
        for line in Path("/proc/self/cgroup").read_text().splitlines():
            _, controllers, path = line.split(":", 2)
            for controller in controllers.split(","):
                own[controller] = path
        label = f"d006-{os.getpid()}-{time.monotonic_ns()}"
        try:
            for controller in ("memory", "pids", "cpuacct"):
                if controller not in own:
                    raise RunError(f"cgroup-v1 {controller} controller is unavailable")
                group = Path("/sys/fs/cgroup") / controller / own[controller].lstrip("/") / label
                group.mkdir()
                self.groups[controller] = group
            (self.groups["memory"] / "memory.limit_in_bytes").write_text(str(ceiling["memory_bytes"]))
            (self.groups["pids"] / "pids.max").write_text(str(ceiling["pids"]))
        except OSError as exc:
            self._remove()
            raise RunError(f"cannot create the step cgroups: {exc}") from None
        self._procs = [str(group / "cgroup.procs") for group in self.groups.values()]

    def preexec(self) -> None:  # in the child, between fork and exec
        for path in self._procs:
            with open(path, "w") as handle:
                handle.write(str(os.getpid()))
        os.sched_setaffinity(0, self.cpus)
        os.setsid()

    def _read(self, controller: str, name: str) -> int:
        return int((self.groups[controller] / name).read_text().strip())

    def kill(self) -> None:
        for controller in ("pids", "memory"):
            procs = self.groups.get(controller)
            if not procs:
                continue
            for pid in (procs / "cgroup.procs").read_text().split():
                try:
                    os.kill(int(pid), signal.SIGKILL)
                except ProcessLookupError:
                    pass

    def _remove(self) -> None:
        for group in self.groups.values():
            if group.exists():
                group.rmdir()

    def close(self) -> dict[str, Any]:
        try:
            leftovers = ["x"]
            for _ in range(500):
                leftovers = [(g / "cgroup.procs").read_text().strip() for g in self.groups.values()]
                if not any(leftovers):
                    break
                self.kill()
                time.sleep(0.01)
            peak = self._read("memory", "memory.max_usage_in_bytes")
            oom = (self.groups["memory"] / "memory.oom_control").read_text()
            cpu_ns = self._read("cpuacct", "cpuacct.usage")
            peak_path = self.groups["pids"] / "pids.peak"
            processes = int(peak_path.read_text().strip()) if peak_path.exists() else None
            events = (self.groups["pids"] / "pids.events").read_text() if (self.groups["pids"] / "pids.events").exists() else ""
        except (OSError, ValueError) as exc:
            raise RunError(f"resource meter could not be read: {exc}") from None
        if any(leftovers):
            raise RunError("processes outlived the step")
        self._remove()
        if peak <= 0:
            raise RunError("resource meter reported no memory use")
        pid_hits = int(m.group(1)) if (m := re.search(r"max (\d+)", events)) else 0
        oom_kills = int(m.group(1)) if (m := re.search(r"oom_kill (\d+)", oom)) else 0
        # The cgroup peak charges page cache too; the largest process's peak
        # resident set (from wait4) is recorded beside it.
        return {"cgroup_peak_bytes": peak, "memory_limit_hit": oom_kills > 0, "cpu_ms": cpu_ns // 1_000_000,
                "process_peak": processes, "pid_limit_hit": pid_hits > 0}


def dir_bytes(root: Path) -> int:
    total = 0
    if not root.exists():
        return 0
    for directory, _, files in os.walk(root):
        for name in files:
            try:
                total += os.lstat(os.path.join(directory, name)).st_size
            except FileNotFoundError:
                pass
    return total


@dataclass
class Sandbox:
    """How one step may touch the filesystem."""

    read_only: list[str]
    read_write: list[str]


@dataclass
class Launcher:
    binary: Path
    identity: dict[str, Any]
    base_ro: list[str] = field(default_factory=lambda: ["/usr", "/proc", "/sys/devices/system/cpu", "/dev/urandom"])
    on_step: Callable[[dict[str, Any]], None] | None = None

    @staticmethod
    def resolve(command: list[str], env: dict[str, str], cwd: Path | None) -> list[str]:
        """The sandbox takes an absolute program path: search the step's PATH as execvp would."""

        program = command[0]
        if program.startswith("/"):
            return command
        if "/" in program:
            return [str((cwd or Path("/")) / program), *command[1:]]
        directories = [d for d in env.get("PATH", "").split(":") if d.startswith("/")]
        found = shutil.which(program, path=":".join(directories))
        # Not found: the first PATH entry, so the step fails with ENOENT as execvp's would.
        return [found or f"{(directories or ['/usr/bin'])[0]}/{program}", *command[1:]]

    def argv(self, command: list[str], env: dict[str, str], box: Sandbox, cwd: Path | None = None) -> list[str]:
        command = self.resolve(command, env, cwd)
        rules = ["--dir", "/"]
        for root in [*self.base_ro, *box.read_only]:
            rules += ["--ro", root]
        for root in [*box.read_write, "/dev/null"]:
            rules += ["--rw", root]
        environment = [f"{k}={v}" for k, v in sorted(env.items())]
        # env clears the environment and execs the sandbox, which execs the command with execv: a command
        # the host cannot execute fails with its errno instead of being retried as a shell script.
        return [*IDENTITY, *NAMESPACE, *PRIVILEGES, *INIT, "/usr/bin/env", "-i", *environment, str(self.binary), *rules, "--", *command]

    def run(self, command: list[str], cwd: Path, env: dict[str, str], ceiling: dict[str, int],
            box: Sandbox, cpus: tuple[int, ...], temp: Path, watch: Callable[[], bool] | None = None) -> dict[str, Any]:
        """Run one process tree; ``watch`` returning True kills it (fault D6-F09)."""

        meter = CgroupMeter(ceiling, cpus)
        started = time.monotonic_ns()
        try:
            process = subprocess.Popen(self.argv(command, env, box, cwd), cwd=cwd, env={"PATH": "/usr/bin:/bin"},
                                       stdin=subprocess.DEVNULL, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                                       preexec_fn=meter.preexec)
        except OSError as exc:
            meter.close()
            raise RunError(f"launcher could not start: {exc}") from None
        buffers = {"stdout": bytearray(), "stderr": bytearray()}
        timed_out = oversized = killed = False
        total = 0
        cap = ceiling["output_bytes"]
        deadline = time.monotonic() + ceiling["wall_seconds"]
        with selectors.DefaultSelector() as selector:
            selector.register(process.stdout, selectors.EVENT_READ, "stdout")
            selector.register(process.stderr, selectors.EVENT_READ, "stderr")
            while selector.get_map():
                remaining = deadline - time.monotonic()
                if remaining <= 0:
                    timed_out = True
                    break
                if watch is not None and watch():
                    killed = True
                    break
                for key, _ in selector.select(min(remaining, 0.05 if watch else remaining)):
                    chunk = os.read(key.fd, 65536)
                    if not chunk:
                        selector.unregister(key.fileobj)
                        continue
                    room = cap - total
                    if len(chunk) > room:
                        buffers[key.data] += chunk[:max(room, 0)]
                        oversized = True
                        break
                    buffers[key.data] += chunk
                    total += len(chunk)
                if oversized:
                    break
        if timed_out or oversized or killed:
            meter.kill()
        _, status, usage = os.wait4(process.pid, 0)
        code = os.waitstatus_to_exitcode(status)
        process.stdout.close()
        process.stderr.close()
        wall = (time.monotonic_ns() - started) // 1_000_000
        metered = meter.close()
        stdout, stderr = bytes(buffers["stdout"]), bytes(buffers["stderr"])
        if stderr.startswith(LAUNCH_FAILURES) and not stderr.startswith(COMMAND_EXEC_FAILURE):
            raise RunError(f"launcher failure: {stderr[:300].decode('utf-8', 'replace')}")
        temp_bytes = dir_bytes(temp)
        state = classify_exit(code, timed_out, oversized, killed, metered, temp_bytes > ceiling["temp_bytes"])
        return {"argv": command, "exit_status": code, "stdout": stdout, "stderr": stderr,
                "timed_out": timed_out, "oversized": oversized, "killed": killed,
                "wall_ms": wall, "temp_bytes": temp_bytes, "state": state,
                "peak_rss_bytes": int(usage.ru_maxrss) * 1024, **metered}


def classify_exit(code: int, timed_out: bool, oversized: bool, killed: bool, metered: dict[str, Any], temp_over: bool) -> dict[str, Any]:
    signal_name = None
    exit_code: int | None = code
    number = 0
    if code < 0:
        number, exit_code = -code, None
    elif code > 128 and code - 128 < signal.NSIG:
        number, exit_code = code - 128, None
    if number:
        try:
            signal_name = signal.Signals(number).name
        except ValueError:
            signal_name = f"SIG{number}"
    if timed_out:
        kind = "timeout"
    elif killed:
        kind = "killed_by_runner"
    elif oversized:
        kind = "oversized_output"
    elif metered["memory_limit_hit"] or metered["pid_limit_hit"] or temp_over or signal_name in ("SIGXCPU", "SIGXFSZ"):
        kind = "resource_exhaustion"
    elif exit_code == 0:
        kind = "completed"
    elif exit_code is not None:
        kind = "failed"
    else:
        kind = "crash"
    return {"kind": kind, "exit_code": exit_code, "signal": signal_name}


# ---------------------------------------------------------------------------
# Toolchain archives (content-addressed) and workspaces


def deterministic_tar(source: Path, target: Path, top: str) -> None:
    """A tar of an installed tree with sorted entries, zero owners and times."""

    def reset(info: tarfile.TarInfo) -> tarfile.TarInfo:
        info.uid = info.gid = 0
        info.uname = info.gname = ""
        info.mtime = 0
        return info

    entries = sorted(source.rglob("*"), key=lambda p: p.relative_to(source).as_posix())
    with tarfile.open(target, "w", format=tarfile.GNU_FORMAT) as archive:
        archive.add(source, arcname=top, recursive=False, filter=reset)
        for path in entries:
            archive.add(path, arcname=f"{top}/{path.relative_to(source).as_posix()}", recursive=False, filter=reset)


# Every toolchain component a candidate unpacks, by candidate: (tool ids,
# source, archive kind, top-level directory, placeholder it fills).
UNPACKS = {
    "C-01": [
        {"tools": ["TC-02", "TC-03"], "source": "/opt/d006/rocq-9.2.0", "kind": "tree", "top": "rocq-9.2.0", "placeholder": "toolchain"},
        {"tools": ["TC-10"], "source": "/opt/d006/ocaml-4.14.1-aarch64", "kind": "tree", "top": "ocaml-4.14.1-aarch64", "placeholder": "toolchain_aarch64"},
    ],
    "C-02": [
        {"tools": ["TC-01"], "source": "/opt/d006/lean-4.34.1-linux.tar.zst", "kind": "tar.zst", "top": "lean-4.34.1-linux", "placeholder": "toolchain"},
        {"tools": ["TC-11"], "source": "/opt/d006/lean-4.34.1-linux_aarch64.tar.zst", "kind": "tar.zst", "top": "lean-4.34.1-linux_aarch64", "placeholder": "toolchain_aarch64"},
    ],
}
SOLVER = {"tools": ["TC-04"], "source": "/opt/d006/cadical-src/build/cadical"}


def prepare_archives(archive: Path) -> dict[str, Any]:
    """Write each toolchain component once into archive/toolchains/, named by digest."""

    store = archive / "toolchains"
    store.mkdir(parents=True, exist_ok=True)
    rows: dict[str, Any] = {}
    for cand, parts in UNPACKS.items():
        rows[cand] = []
        for part in parts:
            source = Path(part["source"])
            if part["kind"] == "tree":
                staging = store / f"staging-{part['top']}.tar"
                deterministic_tar(source, staging, part["top"])
                name = f"{file_sha256(staging)}.tar"
                staging.rename(store / name)
            else:
                name = f"{file_sha256(source)}.tar.zst"
                if not (store / name).exists():
                    shutil.copyfile(source, store / name)
            data = store / name
            data.chmod(0o444)
            rows[cand].append({**{k: part[k] for k in ("tools", "kind", "top", "placeholder")},
                               "archive": f"toolchains/{name}", "sha256": name.split(".")[0], "bytes": data.stat().st_size})
    solver = store / f"{file_sha256(Path(SOLVER['source']))}.bin"
    shutil.copyfile(SOLVER["source"], solver)
    solver.chmod(0o555)
    rows["solver"] = {"tools": SOLVER["tools"], "archive": f"toolchains/{solver.name}", "sha256": solver.name.split(".")[0], "bytes": solver.stat().st_size}
    return rows


def lab_owned(path: Path) -> Path:
    path.mkdir(parents=True, exist_ok=True)
    os.chown(path, LAB_UID, LAB_UID)
    return path


def chown_tree(root: Path) -> None:
    for directory, dirs, files in os.walk(root):
        os.chown(directory, LAB_UID, LAB_UID)
        for name in files:
            os.lchown(os.path.join(directory, name), LAB_UID, LAB_UID)


@dataclass
class Workspace:
    name: str
    root: Path

    @property
    def checkout(self) -> Path:
        return self.root / "checkout"

    @property
    def toolchains(self) -> Path:
        return self.root / "toolchains"

    @property
    def out(self) -> Path:
        return self.root / "out"


# ---------------------------------------------------------------------------
# Records


PATH_TOKENS = ("$RUN", "$TOOLCHAIN", "$WORKSPACE", "$ARCHIVE")


class Recorder:
    """Collects step records for one run and archives their logs by digest."""

    def __init__(self, archive: Path, tokens: dict[str, str]) -> None:
        self.archive = archive
        self.tokens = tokens
        self.steps: list[dict[str, Any]] = []

    def normalize(self, text: str) -> str:
        for token, value in sorted(self.tokens.items(), key=lambda kv: -len(kv[1])):
            if value:
                text = text.replace(value, token)
        return text

    def log(self, data: bytes) -> str:
        name = sha256(data)
        target = self.archive / "logs" / name
        if not target.exists():
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_bytes(data)
            target.chmod(0o444)
        return name

    def add(self, label: str, launched: dict[str, Any], ceiling_class: str, cpus: tuple[int, ...], env: dict[str, str], cwd: Path) -> dict[str, Any]:
        row = {
            "ordinal": len(self.steps) + 1,
            "label": label,
            "argv": [self.normalize(a) for a in launched["argv"]],
            "cwd": self.normalize(str(cwd)),
            "environment": {k: self.normalize(v) for k, v in sorted(env.items())},
            "ceiling": ceiling_class,
            "cpus": list(cpus),
            "state": launched["state"],
            "measured": {k: launched[k] for k in ("wall_ms", "cpu_ms", "peak_rss_bytes", "cgroup_peak_bytes", "process_peak", "temp_bytes")},
            "stdout": {"sha256": self.log(launched["stdout"]), "bytes": len(launched["stdout"])},
            "stderr": {"sha256": self.log(launched["stderr"]), "bytes": len(launched["stderr"])},
        }
        self.steps.append(row)
        return row


def outcome_row(outcome: Any, normalize: Callable[[str], str]) -> dict[str, Any]:
    return {
        "id": outcome.ident, "kind": outcome.kind, "passed": bool(outcome.passed),
        "category": outcome.category, "location": outcome.location,
        "diagnostic_conforms": outcome.diagnostic_conforms, "detail": normalize(outcome.detail or ""),
    }


# ---------------------------------------------------------------------------
# One candidate run: the sandboxed harness runner


@dataclass
class RunContext:
    epoch: "Epoch"
    cand: str
    toolchains: dict[str, str]        # placeholder -> unpacked root
    run_root: Path
    cpus: tuple[int, ...]
    recorder: Recorder
    wall_override: int | None = None  # D6-F08
    extra_ro: list[str] = field(default_factory=list)  # D6-F04's read-only home
    env_override: dict[str, str] = field(default_factory=dict)  # D6-F06
    label: str = "step"
    fresh_text: str | None = None
    checkers: dict[str, Any] = field(default_factory=dict)
    corpus_items: list[dict[str, Any]] = field(default_factory=list)
    corpus: Path | None = None

    def sandbox(self, extra_ro: Iterable[str] = ()) -> Sandbox:
        ro = [*self.toolchains.values(), *self.epoch.adapters[self.cand].get("host_files", []), *self.extra_ro, *extra_ro]
        return Sandbox(ro, [str(self.run_root)])

    def ceiling(self, harness_ceiling: dict[str, int]) -> tuple[str, dict[str, int]]:
        kind = "negative_case" if harness_ceiling.get("wall_seconds", 0) <= CEILINGS["negative_case"]["wall_seconds"] else "measured_step"
        ceiling = dict(CEILINGS[kind])
        if self.wall_override is not None:
            ceiling["wall_seconds"] = self.wall_override
        return kind, ceiling

    def runner(self, argv: list[str], cwd: Path, env: dict[str, str], harness_ceiling: dict[str, int]) -> H.Launched:
        kind, ceiling = self.ceiling(harness_ceiling)
        env = {**env, **self.env_override}
        chown_tree(self.run_root)  # the runner writes check files as root; steps run as the lab user
        launched = self.epoch.launcher.run(argv, cwd, env, ceiling, self.sandbox(), self.cpus, Path(env.get("TMPDIR", str(self.run_root))))
        self.recorder.add(self.label, launched, kind, self.cpus, env, cwd)
        return to_harness(launched)


def to_harness(launched: dict[str, Any]) -> H.Launched:
    state = launched["state"]["kind"]
    return H.Launched(
        launched["argv"], None if state == "timeout" else launched["exit_status"],
        launched["stdout"], launched["stderr"], state == "timeout", state == "oversized_output",
        launched["wall_ms"], state == "resource_exhaustion" or bool(OUT_OF_MEMORY.search((launched["stdout"] + launched["stderr"]).decode("utf-8", "replace"))),
    )


OUT_OF_MEMORY = re.compile(r"Out of memory|out of memory|std::bad_alloc|Cannot allocate memory")


# ---------------------------------------------------------------------------
# The epoch


def load_adapter_from(root: Path) -> dict[str, Any]:
    return H.load_adapter(root)


@dataclass
class Epoch:
    repo: Path
    archive: Path
    packet: dict[str, Any]
    launcher: Launcher
    adapters: dict[str, dict[str, Any]] = field(default_factory=dict)
    ordinal: int = 0
    reference: dict[str, dict[str, Any]] = field(default_factory=dict)  # cand -> deterministic projection for faults
    checkout_revision: str = ""  # the candidates' revision: the packet's, or a correction's (AM-04)

    @property
    def epoch_id(self) -> str:
        return self.packet["epoch"]

    def write_record(self, record: dict[str, Any]) -> dict[str, Any]:
        self.ordinal += 1
        record = {"schema_version": RECORD_SCHEMA, "suite_version": SUITE_VERSION, "epoch": self.epoch_id, "ordinal": self.ordinal, **record}
        name = f"records/{self.ordinal:04d}-{record['profile']}-{record.get('candidate', 'lab')}.json"
        target = self.archive / name
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_bytes(canonical_file(record))
        target.chmod(0o444)
        return record

    def tokens(self, ws: Workspace, run_root: Path, toolchains: dict[str, str]) -> dict[str, str]:
        tokens = {"$RUN": str(run_root), "$WORKSPACE": str(ws.root), "$ARCHIVE": str(self.archive)}
        for key, value in toolchains.items():
            tokens["$" + key.upper()] = value
        return tokens


def git(repo: Path, *args: str) -> str:
    return subprocess.run(["git", "-C", str(repo), *args], capture_output=True, check=True, text=True).stdout.strip()


def git_bytes(repo: Path, *args: str) -> bytes:
    result = subprocess.run(["git", "-C", str(repo), *args], capture_output=True, check=False)
    if result.returncode != 0:
        raise RunError(f"git {' '.join(args)} failed: {result.stderr[:200]!r}")
    return result.stdout


BUILD_PRODUCTS = (".lake", "*.vo", "*.vok", "*.vos", "*.glob", ".*.aux", ".lia.cache", "_build")


def source_rows(root: Path) -> list[tuple[str, str]]:
    """A candidate's sources: the path and content of every file, build products aside."""

    def built(rel: str) -> bool:
        return any(fnmatch.fnmatch(part, pattern) for part in rel.split("/") for pattern in BUILD_PRODUCTS)

    return [(row["path"], row["sha256"]) for row in tree_manifest(root, exclude=built)]


def provision(epoch: Epoch, name: str, candidates: Iterable[str]) -> tuple[Workspace, dict[str, Any]]:
    """A separately provisioned workspace: its own checkout, toolchain unpacks and output root."""

    ws = Workspace(name, epoch.archive / "work" / name)
    if ws.root.exists():
        raise RunError(f"workspace {name} already exists")
    ws.root.mkdir(parents=True)
    os.chmod(ws.root, 0o755)
    ws.checkout.mkdir()
    revision = epoch.checkout_revision or epoch.packet["revision"]
    archive = subprocess.run(["git", "-C", str(epoch.repo), "archive", "--format=tar", revision], capture_output=True, check=True).stdout
    subprocess.run(["tar", "-x", "-C", str(ws.checkout)], input=archive, check=True)
    for cand, source in epoch.packet.get("dev_candidates", {}).items():
        target = ws.checkout / CANDIDATE_DIRS[cand]
        shutil.rmtree(target, ignore_errors=True)
        shutil.copytree(source, target, ignore=shutil.ignore_patterns(*BUILD_PRODUCTS))
    for sub in ("toolchains", "out"):
        lab_owned(ws.root / sub)
    unpacked: dict[str, Any] = {}
    for cand in candidates:
        rec = Recorder(epoch.archive, {"$WORKSPACE": str(ws.root)})
        roots = unpack(epoch, cand, lab_owned(ws.toolchains / cand), rec, CPUS["declared_parallel"])
        unpacked[cand] = roots
        epoch.adapters.setdefault(cand, load_adapter_from(ws.checkout / CANDIDATE_DIRS[cand]))
        epoch.write_record({"profile": "provision", "workspace": name, "candidate": cand, "toolchains": {k: rec.normalize(v) for k, v in roots.items()}, "steps": rec.steps,
                            "checkout": {"revision": revision, "candidate_tree": tree_manifest(ws.checkout / CANDIDATE_DIRS[cand])}})
    return ws, unpacked


def unpack(epoch: Epoch, cand: str, dest: Path, rec: Recorder, cpus: tuple[int, ...]) -> dict[str, str]:
    roots = {}
    store = epoch.archive / "toolchains"
    for part in epoch.packet["archives"][cand]:
        path = epoch.archive / part["archive"]
        if file_sha256(path) != part["sha256"]:
            raise RunError(f"toolchain archive {part['archive']} differs from the packet; refusing to unpack")
        # The unpack keeps modes but not the archive's owners: the lab user owns its unpack.
        argv = ["/usr/bin/tar", "--no-same-owner", *(["--zstd"] if part["kind"] == "tar.zst" else []), "-xf", str(path), "-C", str(dest)]
        scratch = lab_owned(dest / ".unpack-tmp")
        env = {"PATH": "/usr/bin:/bin", "LANG": "C", "LC_ALL": "C", "TZ": "UTC", "HOME": str(scratch), "TMPDIR": str(scratch)}
        launched = epoch.launcher.run(argv, dest, env, CEILINGS["measured_step"], Sandbox([str(store)], [str(dest)]), cpus, scratch)
        rec.add(f"unpack {part['top']}", launched, "measured_step", cpus, env, dest)
        shutil.rmtree(scratch)
        if launched["state"]["kind"] != "completed":
            raise RunError(f"unpack of {part['top']} failed: {launched['stderr'][:300]!r}")
        roots[part["placeholder"]] = str(dest / part["top"])
    return roots


def make_candidate(epoch: Epoch, ws: Workspace, cand: str, toolchains: dict[str, str], run_root: Path,
                   cpus: tuple[int, ...], label: str, **context: Any) -> tuple[H.Candidate, RunContext]:
    rec = Recorder(epoch.archive, epoch.tokens(ws, run_root, toolchains))
    ctx = RunContext(epoch, cand, toolchains, run_root, cpus, rec, label=label, **context)
    source = ws.checkout / CANDIDATE_DIRS[cand]
    extra = {k: v for k, v in toolchains.items() if k != "toolchain"}
    candidate = H.Candidate(source, epoch.adapters[cand], toolchains["toolchain"], run_root / "c", ctx.runner, extra)
    return candidate, ctx


def fresh_root(parent: Path, name: str) -> Path:
    root = parent / name
    if root.exists():
        raise RunError(f"run root {root} already exists")
    return lab_owned(root)


# ---------------------------------------------------------------------------
# Cases inside one run


def check_items(cand: H.Candidate, case: str, directory: str, observations: list[dict[str, Any]]) -> list[Any]:
    return H.check_observations(cand, case, cand.work / "checks" / directory, observations, H.MEASURED_CEILING)


def solve(epoch: Epoch, ctx: RunContext, ident: str, key: str, obligation: str, zero_wall: bool = False) -> tuple[dict[str, Any], dict[str, Any], Path]:
    """One run of the pinned solver in the sandbox on an obligation's shared CNF."""

    solver = epoch.archive / epoch.packet["archives"]["solver"]["archive"]
    cnf = epoch.repo / SHARED_DIR / H.SOLVER_CNF[obligation]
    out = lab_owned(ctx.run_root / "solver" / ident)
    certificate = out / "certificate.lrat"
    argv = H.solver_argv(str(solver), key, str(cnf), str(certificate))
    env = {"PATH": "/usr/bin:/bin", "LANG": "C", "LC_ALL": "C", "TZ": "UTC", "HOME": str(out), "TMPDIR": str(out)}
    ceiling = "solver_zero_wall" if zero_wall else "measured_step"
    launched = epoch.launcher.run(argv, out, env, CEILINGS[ceiling], Sandbox([str(solver), str(cnf.parent)], [str(out)]), ctx.cpus, out)
    row = ctx.recorder.add(f"solver {ident}", launched, ceiling, ctx.cpus, env, out)
    return launched, row, certificate


def fresh_certificate(epoch: Epoch, ctx: RunContext) -> tuple[str | None, dict[str, Any]]:
    """The run's own certificate (D4-R01): the pinned solver's LRAT for the carry-save obligation."""

    launched, row, certificate = solve(epoch, ctx, "D4-R01", "argv", "B-C01")
    claim = H.solver_claim(launched["state"]["kind"], launched["exit_status"], certificate.is_file())
    text = certificate.read_text(encoding="ascii") if claim == "certificate" else None
    ctx.fresh_text = text
    return text, {"step": row["ordinal"], "claim": claim, "sha256": sha256(text.encode("ascii")) if text else None, "bytes": len(text) if text else None}


def run_time(epoch: Epoch, cand: H.Candidate, ctx: RunContext) -> list[Any]:
    """D4-R02 to D4-R05 with the sandboxed solver and the candidate's own checks."""

    def sandboxed(ident: str, key: str, obligation: str, zero_wall: bool) -> tuple[str, int | None, bytes, Path]:
        ctx.label = f"DS-04 {ident}"
        launched, _, certificate = solve(epoch, ctx, ident, key, obligation, zero_wall)
        return launched["state"]["kind"], launched["exit_status"], launched["stdout"], certificate

    return H.run_time_cases(sandboxed, lambda rows: check_items(cand, "DS-04", "DS-04-run-time", rows))


def run_cases(epoch: Epoch, cand: H.Candidate, ctx: RunContext, cases: Iterable[str], negatives: bool = True) -> dict[str, Any]:
    normalize = ctx.recorder.normalize
    result: dict[str, Any] = {}
    for case in cases:
        if case not in PROOF_CASES:
            continue
        ctx.label = f"{case} positive"
        positives = [outcome_row(o, normalize) for o in H.run_positive(cand, case)]
        row: dict[str, Any] = {"positives": positives}
        if case == "DS-04":
            ctx.label = "DS-04 fresh certificate"
            text, meta = fresh_certificate(epoch, ctx)
            row["fresh_certificate"] = meta
            if text is None:
                row["fresh"] = [{"id": "D4-F", "kind": "observation", "passed": False, "category": None, "location": None,
                                 "diagnostic_conforms": None, "detail": "the pinned solver produced no certificate"}]
            else:
                observations = H.certificate_observations("D4-F", text)
                row["fresh"] = [outcome_row(o, normalize) for o in check_items(cand, case, "DS-04-fresh", observations)]
        if case == "DS-04" and negatives:
            expected = {"D4-R02": "disproved_obligation", "D4-R03": "unknown", "D4-R04": "timeout", "D4-R05": "failed_certificate"}
            row["run_time"] = [{**outcome_row(o, normalize), "expected": [expected[o.ident]]} for o in run_time(epoch, cand, ctx)]
        if negatives:
            rows = []
            for negative in H.case_material(case)["negatives"]:
                ctx.label = f"{case} {negative['id']}"
                outcome = H.run_negative(cand, case, negative)
                rows.append({**outcome_row(outcome, normalize), "expected": list(negative["expected"])})
            row["negatives"] = rows
        result[case] = row
    return result


def artifact_manifest(cand: H.Candidate) -> list[dict[str, Any]]:
    patterns = cand.adapter.get("artifacts", {}).get("deterministic", [])
    return tree_manifest(cand.src, patterns)


def build_state(built: list[H.Launched]) -> dict[str, Any]:
    ok = bool(built) and all(b.exit_status == 0 for b in built)
    return {"completed": ok, "steps": len(built)}


def totals(steps: list[dict[str, Any]]) -> dict[str, int]:
    return {
        "wall_ms": sum(s["measured"]["wall_ms"] for s in steps),
        "cpu_ms": sum(s["measured"]["cpu_ms"] for s in steps),
        "peak_rss_bytes": max((s["measured"]["peak_rss_bytes"] for s in steps), default=0),
        "cgroup_peak_bytes": max((s["measured"]["cgroup_peak_bytes"] for s in steps), default=0),
        "temp_bytes": max((s["measured"]["temp_bytes"] for s in steps), default=0),
    }


def deterministic_projection(cases: dict[str, Any], artifacts: list[dict[str, Any]], build: dict[str, Any], standalone: dict[str, Any] | None) -> dict[str, Any]:
    return {"build": build, "cases": cases, "artifacts": artifacts, "standalone": standalone}


def standalone_placeholder() -> dict[str, Any]:
    return {"status": "not_run"}


# ---------------------------------------------------------------------------
# Profiles


@dataclass
class Plan:
    """What an execution covers; the epoch plan is the preregistered protocol."""

    candidates: tuple[str, ...] = ("C-01", "C-02")
    cases: tuple[str, ...] = PROOF_CASES
    cold_runs: int = 5
    replay_runs: int = 3
    timed_pairs: int = 30
    timed_cases: tuple[str, ...] = TIMED_CASES
    faults: tuple[str, ...] = tuple(f"D6-F{i:02d}" for i in range(1, 10))
    second_workspace: bool = True
    negatives: bool = True
    standalone: bool = True
    revision: str = ""  # a correction round's candidate revision; empty means the packet's

    def order(self, run: int) -> tuple[str, ...]:
        """Odd runs C-01 first, even runs C-02 first (overlay execution_order)."""
        if len(self.candidates) == 1:
            return self.candidates
        return ("C-01", "C-02") if run % 2 == 1 else ("C-02", "C-01")


def deterministic_run(epoch: Epoch, plan: Plan, ws: Workspace, toolchains: dict[str, str], cand_id: str, mode: str, run: int) -> dict[str, Any]:
    run_root = fresh_root(ws.out, f"replay-{mode}-{run}-{cand_id}")
    cand, ctx = make_candidate(epoch, ws, cand_id, toolchains, run_root, CPUS[mode], "build")
    cand.prepare()
    built = cand.build(mode)
    build = build_state(built)
    cases = run_cases(epoch, cand, ctx, plan.cases, plan.negatives) if build["completed"] else {}
    standalone = run_standalone(epoch, cand, ctx, cases, ctx.fresh_text) if plan.standalone and build["completed"] else standalone_placeholder()
    artifacts = artifact_manifest(cand)
    projection = deterministic_projection(cases, artifacts, build, standalone_projection(standalone))
    record = epoch.write_record({
        "profile": "deterministic_replay", "workspace": ws.name, "candidate": cand_id, "mode": mode, "run": run,
        "candidate_tree": tree_manifest(cand.root), "steps": ctx.recorder.steps, "totals": totals(ctx.recorder.steps),
        "projection": projection, "projection_sha256": digest(projection),
        "positive_projection_sha256": digest(positive_projection(projection)), "standalone": standalone,
    })
    return {"record": record, "run_root": run_root, "candidate": cand, "context": ctx}


def positive_projection(projection: dict[str, Any]) -> dict[str, Any]:
    """What fault runs recreate: build state, positive outcomes and the artifact manifest."""

    cases = {case: {"positives": row["positives"], **({"fresh": row["fresh"]} if "fresh" in row else {})} for case, row in projection["cases"].items()}
    return {"build": projection["build"], "cases": cases, "artifacts": projection["artifacts"]}


def cold_bootstrap(epoch: Epoch, plan: Plan, ws: Workspace, cand_id: str, run: int) -> dict[str, Any]:
    run_root = fresh_root(ws.out, f"cold-{run}-{cand_id}")
    rec = Recorder(epoch.archive, {"$RUN": str(run_root), "$WORKSPACE": str(ws.root), "$ARCHIVE": str(epoch.archive)})
    mode = "declared_parallel"
    toolchains = unpack(epoch, cand_id, lab_owned(run_root / "tc"), rec, CPUS[mode])
    cand, ctx = make_candidate(epoch, ws, cand_id, toolchains, run_root, CPUS[mode], "build")
    ctx.recorder.steps = rec.steps
    ctx.recorder.tokens.update(rec.tokens)
    cand.prepare()
    built = cand.build(mode)
    checkers = {}
    if plan.standalone and build_state(built)["completed"]:
        for host, spec in cand.adapter.get("standalone", {}).get("hosts", {}).items():
            _, checkers[host] = build_checker(epoch, cand, ctx, host, spec)
    artifacts = artifact_manifest(cand)
    record = epoch.write_record({
        "profile": "cold_bootstrap", "workspace": ws.name, "candidate": cand_id, "mode": mode, "run": run,
        "candidate_tree": tree_manifest(cand.root), "steps": ctx.recorder.steps, "totals": totals(ctx.recorder.steps),
        "build": build_state(built), "artifacts": artifacts, "artifacts_sha256": digest(artifacts),
        "standalone": checkers, "workspace_bytes": dir_bytes(run_root),
    })
    shutil.rmtree(run_root)
    return record


STANDALONE_TIMED = {"DS-03": ("D5-R-F",), "DS-04": ("D5-CNF-", "D5-G-", "D5-F-"), "DS-05": ("D5-",)}


def timed_steps(epoch: Epoch, ws: Workspace, cand_id: str, subject: dict[str, Any], case: str, toolchains: dict[str, str], cpus: tuple[int, ...], label: str) -> list[dict[str, Any]]:
    """One timed replay of a case: the kernel re-check of its compiled proofs, then its standalone corpus."""

    adapter = epoch.adapters[cand_id]
    built: H.Candidate = subject["candidate"]
    scratch = lab_owned(ws.out / "timed-scratch")
    shutil.rmtree(scratch)
    scratch = lab_owned(scratch)
    for sub in ("home", "tmp"):
        lab_owned(scratch / sub)
    rec = Recorder(epoch.archive, epoch.tokens(ws, built.work.parent, toolchains))
    env = built.environment({"HOME": str(scratch / "home"), "TMPDIR": str(scratch / "tmp")})
    box = Sandbox([*toolchains.values(), *adapter.get("host_files", []), str(built.src)], [str(scratch)])
    for argv in adapter.get("recheck", {}).get(case, []):
        filled = [built.fill(a, jobs="1") for a in argv]
        launched = epoch.launcher.run(filled, built.src, env, CEILINGS["timed_replay_step"], box, cpus, scratch / "tmp")
        rec.add(f"{label} {case} recheck", launched, "timed_replay_step", cpus, env, built.src)
    source: RunContext = subject["context"]
    checker = source.checkers.get("H-01")
    prefixes = STANDALONE_TIMED.get(case)
    if checker and prefixes and source.corpus is not None:
        timed_ctx = RunContext(epoch, cand_id, toolchains, scratch, cpus, rec, label=label)
        for item in source.corpus_items:
            if item["id"].startswith(prefixes):
                run_checker(epoch, timed_ctx, checker, item["args"], source.corpus, f"{label} {case} {item['id']}")
    return rec.steps


def timed_replay(epoch: Epoch, plan: Plan, ws: Workspace, subjects: dict[str, dict[str, Any]], unpacked: dict[str, dict[str, str]]) -> None:
    cpus = CPUS["serial"]
    for case in plan.timed_cases:
        for cand_id in plan.candidates:
            steps = timed_steps(epoch, ws, cand_id, subjects[cand_id], case, unpacked[cand_id], cpus, "warmup")
            epoch.write_record({"profile": "timed_replay", "workspace": ws.name, "candidate": cand_id, "case": case, "pair": 0,
                                "warmup": True, "steps": steps, "totals": totals(steps)})
        for pair in range(1, plan.timed_pairs + 1):
            for cand_id in plan.order(pair):
                steps = timed_steps(epoch, ws, cand_id, subjects[cand_id], case, unpacked[cand_id], cpus, "timed")
                epoch.write_record({"profile": "timed_replay", "workspace": ws.name, "candidate": cand_id, "case": case, "pair": pair,
                                    "warmup": False, "steps": steps, "totals": totals(steps),
                                    "completed": bool(steps) and all(s["state"]["kind"] == "completed" for s in steps)})


# ---------------------------------------------------------------------------
# Fault injections (DS-06 D6-F01 to D6-F09)


def fault_run(epoch: Epoch, plan: Plan, ws: Workspace, cand_id: str, fault: str, toolchains: dict[str, str]) -> dict[str, Any]:
    adapter = epoch.adapters[cand_id]
    faults = adapter.get("faults", {})
    mode = "declared_parallel"
    reference = epoch.reference.get(cand_id)
    run_root = fresh_root(ws.out, f"fault-{fault}-{cand_id}")
    evidence: dict[str, Any] = {}
    context: dict[str, Any] = {}
    cpus = CPUS[mode]
    tcs = dict(toolchains)
    pre_steps: list[dict[str, Any]] = []

    if fault == "D6-F03":
        rec = Recorder(epoch.archive, {"$RUN": str(run_root)})
        tcs = unpack(epoch, cand_id, lab_owned(run_root / "tc"), rec, cpus)
        pre_steps = rec.steps
        removed = []
        for rel in faults.get("stdlib", []):
            target = Path(tcs["toolchain"]) / rel
            if target.is_dir():
                shutil.rmtree(target)
                removed.append(rel)
            elif target.exists():
                target.unlink()
                removed.append(rel)
        evidence["removed"] = removed
    elif fault == "D6-F05":
        moved = lab_owned(run_root / "moved" / "to" / "another" / "absolute" / "path")
        tcs = {}
        for key, value in toolchains.items():
            target = moved / "tc" / Path(value).name
            target.parent.mkdir(parents=True, exist_ok=True)
            subprocess.run(["cp", "-al", value, str(target)], check=True)
            tcs[key] = str(target)
        run_root = moved
    elif fault == "D6-F04":
        # A root-owned, read-only home outside every writable root.
        home = ws.out / f"readonly-home-{cand_id}"
        home.mkdir(mode=0o555)
        context["extra_ro"] = [str(home)]
        context["env_override"] = {"HOME": str(home)}
    elif fault == "D6-F06":
        context["env_override"] = {"LANG": "C.UTF-8", "LC_ALL": "C.UTF-8", "TZ": "Pacific/Kiritimati", "SOURCE_DATE_EPOCH": "1700000000"}
    elif fault == "D6-F07":
        cpus = CPUS["serial"]
        mode = "serial"
    elif fault == "D6-F08":
        context["wall_override"] = 1

    cand, ctx = make_candidate(epoch, ws, cand_id, tcs, run_root, cpus, f"{fault} build", **context)
    ctx.recorder.steps = pre_steps + ctx.recorder.steps
    cand.prepare()

    if fault == "D6-F02":
        # A consistent stale build of a different source replaces one compiled
        # dependency (with its build metadata) in an otherwise complete cache.
        stale = fresh_root(run_root, "stale")
        stale_cand, stale_ctx = make_candidate(epoch, ws, cand_id, tcs, stale, cpus, f"{fault} stale build")
        stale_ctx.recorder = ctx.recorder
        stale_cand.prepare()
        edit = faults.get("stale", {})
        target = stale_cand.src / edit.get("file", "")
        if edit and target.is_file():
            target.write_text(target.read_text() + edit["append"])
        stale_cand.build(mode)
        warm = fresh_root(run_root, "warm")
        warm_cand, warm_ctx = make_candidate(epoch, ws, cand_id, tcs, warm, cpus, f"{fault} cache build")
        warm_ctx.recorder = ctx.recorder
        warm_cand.prepare()
        warm_cand.build(mode)
        cache = [p for pattern in faults.get("cache", []) for p in stale_cand.src.glob(pattern) if p.is_file()]
        for path in sorted(p for p in warm_cand.src.rglob("*") if p.is_file()):
            rel = path.relative_to(warm_cand.src)
            (cand.src / rel).parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(path, cand.src / rel)
        for path in cache:
            rel = path.relative_to(stale_cand.src)
            shutil.copy2(path, cand.src / rel)
        evidence["poisoned"] = sorted(str(p.relative_to(stale_cand.src)) for p in cache)
        shutil.rmtree(stale)
        shutil.rmtree(warm)

    if fault == "D6-F04":
        before = tree_manifest(home)

    if fault == "D6-F09":
        patterns = adapter.get("artifacts", {}).get("deterministic", [])
        seen = {"hit": False}

        def first_artifact() -> bool:
            if seen["hit"]:
                return False
            if any(p.is_file() for pattern in patterns for p in cand.src.glob(pattern)):
                seen["hit"] = True
                return True
            return False

        original = epoch.launcher.run

        def watched(*args: Any, **kwargs: Any) -> dict[str, Any]:
            return original(*args, watch=first_artifact, **kwargs)

        epoch.launcher.run = watched  # type: ignore[method-assign]
        try:
            first = cand.build(mode)
        finally:
            epoch.launcher.run = original  # type: ignore[method-assign]
        evidence["killed_after_first_artifact"] = seen["hit"]
        evidence["first_attempt"] = build_state(first)

    built = cand.build(mode)
    build = build_state(built)
    projection = None
    if fault in ("D6-F02", "D6-F05", "D6-F06", "D6-F09") and build["completed"]:
        cases = run_cases(epoch, cand, ctx, plan.cases, negatives=False)
        projection = positive_projection(deterministic_projection(cases, artifact_manifest(cand), build, None))
    last = ctx.recorder.steps[-1]["state"] if ctx.recorder.steps else None
    output = b""
    if built:
        output = built[-1].stdout + built[-1].stderr
    verdict = fault_verdict(fault, build, last, projection, reference, evidence, faults, output)
    if fault == "D6-F04":
        evidence["home_unchanged"] = tree_manifest(home) == before
        verdict = verdict and evidence["home_unchanged"]
        home.rmdir()
    record = epoch.write_record({
        "profile": "fault", "workspace": ws.name, "candidate": cand_id, "fault": fault, "mode": mode,
        "steps": ctx.recorder.steps, "totals": totals(ctx.recorder.steps), "build": build,
        "evidence": evidence, "projection_sha256": digest(projection) if projection else None,
        "reference_sha256": digest(reference) if reference else None, "met": verdict,
    })
    shutil.rmtree(ws.out / f"fault-{fault}-{cand_id}")
    return record


def fault_verdict(fault: str, build: dict[str, Any], last: dict[str, Any] | None, projection: dict[str, Any] | None,
                  reference: dict[str, Any] | None, evidence: dict[str, Any], faults: dict[str, Any], output: bytes) -> bool:
    kind = last["kind"] if last else None
    if fault == "D6-F01":
        return build["completed"]
    if fault == "D6-F07":  # completes within the ceilings or fails attributably
        return build["completed"] or kind in ("failed", "timeout", "resource_exhaustion")
    if fault in ("D6-F02", "D6-F05", "D6-F06", "D6-F09"):
        if fault == "D6-F09" and not evidence.get("killed_after_first_artifact"):
            return False
        if build["completed"]:
            return projection is not None and reference is not None and projection == reference
        return kind in ("failed", "crash")  # refused, never a stale success
    if fault == "D6-F03":
        names = faults.get("stdlib_names", [])
        text = output.decode("utf-8", "replace")
        return (not build["completed"]) and kind == "failed" and bool(evidence.get("removed")) and any(n in text for n in names)
    if fault == "D6-F04":  # and never writes outside the workspace
        return evidence.get("home_unchanged") is True and (build["completed"] or kind == "failed")
    if fault == "D6-F08":
        return kind == "timeout" and not build["completed"]
    return False


# ---------------------------------------------------------------------------
# prepare / execute


def host_capture() -> dict[str, Any]:
    def first(path: str, pattern: str) -> str | None:
        try:
            for line in Path(path).read_text().splitlines():
                if re.match(pattern, line):
                    return line.split(":", 1)[1].strip() if ":" in line else line
        except OSError:
            return None
        return None

    tools = {}
    for path in ("/usr/bin/unshare", "/usr/bin/setpriv", "/usr/bin/tar", "/usr/bin/zstd", "/usr/bin/qemu-aarch64-static", "/usr/bin/patch", "/bin/sh", "/usr/bin/env"):
        if os.path.exists(path):
            real = os.path.realpath(path)
            tools[path] = {"realpath": real, "sha256": file_sha256(Path(real))}
    release = {}
    for line in Path("/etc/os-release").read_text().splitlines():
        if "=" in line:
            key, value = line.split("=", 1)
            release[key] = value.strip('"')
    return {
        "system": os.uname().sysname, "kernel": os.uname().release, "machine": os.uname().machine,
        "os": release.get("PRETTY_NAME"), "cpu_model": first("/proc/cpuinfo", r"model name"),
        "cpus": os.cpu_count(), "memory": first("/proc/meminfo", r"MemTotal"),
        "python": ".".join(str(p) for p in sys.version_info[:3]), "tools": tools,
    }


def source_of(repo: Path, path: str) -> Path:
    """A bound file: from the repository, or (development only) beside this runner."""

    candidate = repo / path
    if candidate.exists() or not path.startswith("tools/"):
        return candidate
    return HERE / Path(path).name


IDENTITY_KEYS = ("suite_version", "bindings", "archives", "input_manifest_sha256", "ceilings",
                 "sandbox_caps", "lab_uid", "meter", "dev")


def epoch_name(identity: dict[str, Any]) -> str:
    """An epoch is named by the hash of everything it binds; any change opens a new epoch."""

    return "d006-e-" + digest(identity)[:20]


def bootstrap_seed(identity: dict[str, Any]) -> int:
    return int(digest(identity)[:16], 16)


def bound_inputs(repo: Path) -> list[dict[str, str]]:
    paths = [f"{SHARED_DIR}/manifest.json", OVERLAY_PATH, TOOLCHAINS_PATH, *BOUND_TOOLS]
    return [{"path": p, "sha256": file_sha256(source_of(repo, p))} for p in paths]


def command_prepare(repo: Path, root: Path, dev_candidates: dict[str, str] | None = None) -> Path:
    revision = git(repo, "rev-parse", "HEAD")
    if git(repo, "status", "--porcelain") and not dev_candidates:
        raise RunError("the repository has uncommitted changes; an epoch binds a clean revision")
    overlay = json.loads((repo / OVERLAY_PATH).read_text())
    manifest_digest = json.loads((repo / SHARED_DIR / "manifest.json").read_text())["input_manifest_sha256"]
    if overlay["shared_inputs"]["input_manifest_sha256"] != manifest_digest:
        raise RunError("the shared-input manifest does not match the overlay's digest")
    # The reference regenerates every laboratory file byte for byte.
    reference = subprocess.run([sys.executable, "-S", "-B", str(repo / "tools/d006_shared.py"), "check"], capture_output=True, check=False)
    if reference.returncode != 0:
        raise RunError(f"shared inputs differ from their reference derivation: {reference.stderr[:300]!r}")
    staging = root / "staging"
    shutil.rmtree(staging, ignore_errors=True)
    staging.mkdir(parents=True)
    archives = prepare_archives(staging)
    bindings = bound_inputs(repo)
    identity = {"suite_version": SUITE_VERSION, "bindings": bindings, "archives": archives,
                "input_manifest_sha256": manifest_digest, "ceilings": CEILINGS, "sandbox_caps": SANDBOX_CAPS,
                "lab_uid": LAB_UID, "meter": CgroupMeter.kind, "dev": bool(dev_candidates)}
    epoch = epoch_name(identity)
    archive = root / epoch
    if archive.exists():
        raise RunError(f"{archive} already exists")
    staging.rename(archive)
    os.chmod(archive, 0o755)
    sandbox, sandbox_identity = build_sandbox(repo, archive)
    os.chmod(sandbox, 0o755)
    packet = {"schema_version": "d006-v0.3-packet-1", "epoch": epoch, "revision": revision, **identity,
              "host": host_capture(), "sandbox": sandbox_identity,
              "bootstrap_seed": bootstrap_seed(identity)}
    if dev_candidates:
        packet["dev_candidates"] = dev_candidates
    (archive / "packet.json").write_bytes(canonical_file(packet))
    return archive


def load_epoch(repo: Path, archive: Path) -> Epoch:
    packet = json.loads((archive / "packet.json").read_text())
    if bound_inputs(repo) != packet["bindings"]:
        raise RunError("bound inputs changed since prepare; a change opens a new epoch")
    binary = archive / "fs-sandbox"
    if file_sha256(binary) != packet["sandbox"]["binary_sha256"]:
        raise RunError("sandbox binary changed")
    H.REPO = repo
    H.SHARED = repo / SHARED_DIR
    existing = len(list((archive / "records").glob("*.json"))) if (archive / "records").is_dir() else 0
    return Epoch(repo, archive, packet, Launcher(binary, packet["sandbox"]), ordinal=existing)


def command_execute(repo: Path, archive: Path, plan: Plan) -> None:
    epoch = load_epoch(repo, archive)
    attempts = [r for r in load_records(archive) if r.get("profile") == "execution"] if epoch.ordinal else []
    revision = epoch.packet["revision"]
    if plan.revision:
        # A correction round (overlay correction_window, AM-04): candidate artifacts may change,
        # every bound input may not, and the corrected revision descends from the epoch's.
        revision = git(repo, "rev-parse", plan.revision)
        ancestor = subprocess.run(["git", "-C", str(repo), "merge-base", "--is-ancestor", epoch.packet["revision"], revision], check=False)
        if ancestor.returncode != 0:
            raise RunError("a correction's revision must descend from the epoch's revision")
        for row in epoch.packet["bindings"]:
            if sha256(git_bytes(repo, "show", f"{revision}:{row['path']}")) != row["sha256"]:
                raise RunError(f"{row['path']} differs at {revision[:12]}; a shared change opens a new epoch")
    epoch.checkout_revision = revision
    epoch.write_record({"profile": "execution", "attempt": len(attempts) + 1, "revision": revision,
                        "plan": {k: list(v) if isinstance(v, tuple) else v for k, v in asdict(plan).items()}})
    ws, unpacked = provision(epoch, "W1", plan.candidates)
    for run in range(1, plan.cold_runs + 1):
        for cand_id in plan.order(run):
            cold_bootstrap(epoch, plan, ws, cand_id, run)
    subjects: dict[str, dict[str, Any]] = {}
    for mode in ("serial", "declared_parallel"):
        for run in range(1, plan.replay_runs + 1):
            for cand_id in plan.order(run):
                done = deterministic_run(epoch, plan, ws, unpacked[cand_id], cand_id, mode, run)
                if mode == "serial" and run == 1:
                    epoch.reference[cand_id] = positive_projection(done["record"]["projection"])
                previous = subjects.get(cand_id)
                if previous and previous["run_root"] != done["run_root"]:
                    shutil.rmtree(previous["run_root"])
                subjects[cand_id] = done
    timed_replay(epoch, plan, ws, subjects, unpacked)
    for index, fault in enumerate(plan.faults, 1):
        for cand_id in plan.order(index):
            fault_run(epoch, plan, ws, cand_id, fault, unpacked[cand_id])
    shutil.rmtree(ws.root)
    if plan.second_workspace:
        ws2, unpacked2 = provision(epoch, "W2", plan.candidates)
        for mode in ("serial", "declared_parallel"):
            for cand_id in plan.order(1):
                done = deterministic_run(epoch, plan, ws2, unpacked2[cand_id], cand_id, mode, 1)
                shutil.rmtree(done["run_root"])
        shutil.rmtree(ws2.root)
    write_manifest(archive)


def write_manifest(archive: Path) -> None:
    rows = tree_manifest(archive, exclude=lambda rel: rel.startswith("work/") or rel == "manifest.json")
    (archive / "manifest.json").write_bytes(canonical_file({"schema_version": "d006-v0.3-archive-manifest-1", "files": rows}))


# ---------------------------------------------------------------------------
# Entry point


USAGE = """usage:
  d006_run.py prepare [--root DIR] [--dev-candidates DIR]
  d006_run.py execute ARCHIVE [--smoke] [--candidates C-01,C-02] [--cases DS-01,...] [--faults F,...]
                      [--no-negatives] [--revision REV]
  d006_run.py summarize ARCHIVE
  d006_run.py export ARCHIVE DEST
  d006_run.py verify ARCHIVE_OR_EXPORT"""


def option(argv: list[str], name: str) -> str | None:
    return argv[argv.index(name) + 1] if name in argv else None


def main(argv: list[str]) -> int:
    repo = Path(os.environ.get("ORANGE_REPO", HERE.parent)).resolve()
    if not argv:
        print(USAGE, file=sys.stderr)
        return 2
    try:
        if argv[0] == "prepare":
            root = Path(option(argv, "--root") or ARCHIVE_ROOT)
            dev = option(argv, "--dev-candidates")
            dev_candidates = {cand: str(Path(dev) / Path(path).name) for cand, path in CANDIDATE_DIRS.items()} if dev else None
            print(command_prepare(repo, root, dev_candidates))
            return 0
        if argv[0] == "execute":
            plan = Plan()
            if "--smoke" in argv:
                plan = Plan(cold_runs=1, replay_runs=1, timed_pairs=2, second_workspace=False)
            if option(argv, "--candidates"):
                plan.candidates = tuple(option(argv, "--candidates").split(","))
            if option(argv, "--cases"):
                plan.cases = tuple(c for c in option(argv, "--cases").split(",") if c in PROOF_CASES)
                plan.timed_cases = tuple(option(argv, "--cases").split(","))
                plan.standalone = "DS-05" in plan.timed_cases
            if option(argv, "--faults") is not None:
                plan.faults = tuple(f for f in option(argv, "--faults").split(",") if f)
            if "--no-negatives" in argv:
                plan.negatives = False
            if option(argv, "--revision"):
                plan.revision = option(argv, "--revision")
            command_execute(repo, Path(argv[1]), plan)
            return 0
        if argv[0] == "summarize":
            summary = command_summarize(repo, Path(argv[1]))
            print(json.dumps({"conclusion": summary["conclusion"], "reasons": summary["conclusion_reasons"]}, indent=2))
            return 0
        if argv[0] == "export" and len(argv) == 3:
            print(command_export(repo, Path(argv[1]), Path(argv[2])))
            return 0
        if argv[0] == "verify":
            problems = command_verify(repo, Path(argv[1]))
            for problem in problems:
                print(problem)
            print("archive verified" if not problems else f"{len(problems)} problem(s)")
            return 1 if problems else 0
    except RunError as exc:
        print(f"d006 run error: {exc}", file=sys.stderr)
        return 1
    print(USAGE, file=sys.stderr)
    return 2



# ---------------------------------------------------------------------------
# summarize: metrics, hard gates, materiality labels and the conclusion


def median(values: list[float]) -> float:
    return float(statistics.median(values))


def p95(values: list[float]) -> float:
    ordered = sorted(values)
    return float(ordered[max(0, math.ceil(0.95 * len(ordered)) - 1)])


def mad(values: list[float]) -> float:
    centre = median(values)
    return median([abs(v - centre) for v in values])


def bootstrap_ci(values: list[float], seed: int, resamples: int = 10000) -> list[float] | None:
    if len(values) < 2:
        return None
    rng = random.Random(seed)
    n = len(values)
    medians = sorted(median([values[rng.randrange(n)] for _ in range(n)]) for _ in range(resamples))
    return [medians[int(0.025 * resamples)], medians[int(0.975 * resamples) - 1]]


def describe(values: list[float], seed: int) -> dict[str, Any]:
    if not values:
        return {"n": 0, "raw": []}
    return {"n": len(values), "raw": values, "median": median(values), "mad": mad(values), "p95": p95(values),
            "ci95_median": bootstrap_ci(values, seed)}


def paired_ratio(pairs: list[tuple[float, float]], seed: int, resamples: int = 10000) -> dict[str, Any]:
    """r = median(C-01) / median(C-02) with its paired bootstrap 95% interval."""

    if len(pairs) < 2 or any(b <= 0 for _, b in pairs):
        return {"r": None, "lo": None, "hi": None, "pairs": len(pairs)}
    rng = random.Random(seed)
    n = len(pairs)
    r = median([a for a, _ in pairs]) / median([b for _, b in pairs])
    ratios = []
    for _ in range(resamples):
        sample = [pairs[rng.randrange(n)] for _ in range(n)]
        ratios.append(median([a for a, _ in sample]) / median([b for _, b in sample]))
    ratios.sort()
    return {"r": r, "lo": ratios[int(0.025 * resamples)], "hi": ratios[int(0.975 * resamples) - 1], "pairs": n}


def band_label(kind: str, measure: dict[str, Any]) -> str:
    """The overlay's materiality bands, tested in its fixed order."""

    r, lo, hi = measure.get("r"), measure.get("lo"), measure.get("hi")
    if kind == "interval":
        if r is None or lo is None or hi is None:
            return "inconclusive"
        if hi < 1 and r <= 0.9:
            return "rocq_better"
        if lo > 1 and r >= 1 / 0.9:
            return "lean_better"
        if 0.9 < lo and hi < 1 / 0.9:
            return "practically_equivalent"
        return "inconclusive"
    threshold = 0.9 if kind == "exact" else 0.75
    if r is None:
        return "inconclusive"
    if r <= threshold:
        return "rocq_better"
    if r >= 1 / threshold:
        return "lean_better"
    return "practically_equivalent"


# Record fields an export stores once by digest: replays of one candidate repeat them unchanged.
SHARED_FIELDS = ("projection", "standalone", "candidate_tree")


def load_objects(archive: Path) -> dict[str, Any]:
    return {row["sha256"]: row["value"] for p in sorted(archive.glob("objects-*.jsonl")) for row in map(json.loads, p.read_text().splitlines())}


def pack_record(record: dict[str, Any], objects: dict[str, Any]) -> dict[str, Any]:
    packed = dict(record)
    for name in SHARED_FIELDS:
        if isinstance(packed.get(name), (dict, list)):
            key = digest(packed[name])
            objects[key] = packed[name]
            packed[name] = {"$object": key}
    return packed


def unpack_record(record: dict[str, Any], objects: dict[str, Any]) -> dict[str, Any]:
    for name in SHARED_FIELDS:
        ref = record.get(name)
        if isinstance(ref, dict) and set(ref) == {"$object"}:
            if ref["$object"] not in objects:
                raise RunError(f"record {record.get('ordinal')} names a missing {name} object")
            record[name] = objects[ref["$object"]]
    return record


def load_records(archive: Path) -> list[dict[str, Any]]:
    """Records of a live archive (records/) or of its export (records-NN.jsonl with objects-NN.jsonl)."""

    if (archive / "records").is_dir():
        return [json.loads(p.read_text()) for p in sorted((archive / "records").glob("*.json"))]
    objects = load_objects(archive)
    return [unpack_record(json.loads(line), objects) for p in sorted(archive.glob("records-*.jsonl")) for line in p.read_text().splitlines()]


def load_logs(archive: Path) -> dict[str, bytes]:
    if (archive / "logs").is_dir():
        return {p.name: p.read_bytes() for p in (archive / "logs").iterdir()}
    logs = {}
    for path in sorted(archive.glob("logs-*.jsonl")):
        for line in path.read_text().splitlines():
            row = json.loads(line)
            logs[row["sha256"]] = row["text"].encode("utf-8") if "text" in row else base64.b64decode(row["base64"])
    return logs


def gate(passed: bool | None) -> str:
    return "unresolved" if passed is None else ("pass" if passed else "fail")


def candidate_summary(cand: str, records: list[dict[str, Any]], packet: dict[str, Any], seed: int) -> dict[str, Any]:
    mine = [r for r in records if r.get("candidate") == cand]
    replays = [r for r in mine if r["profile"] == "deterministic_replay"]
    w1 = [r for r in replays if r["workspace"] == "W1"]
    w2 = [r for r in replays if r["workspace"] == "W2"]
    colds = [r for r in mine if r["profile"] == "cold_bootstrap"]
    faults = [r for r in mine if r["profile"] == "fault"]
    timed = [r for r in mine if r["profile"] == "timed_replay" and not r.get("warmup")]

    positives = negatives = 0
    positives_total = negatives_total = 0
    conforming = diagnostics_total = 0
    by_category: dict[str, list[int]] = {}
    undeclared: set[str] = set()
    case_rows: dict[str, dict[str, Any]] = {}
    for record in replays:
        for case, row in record["projection"]["cases"].items():
            items = row["positives"] + row.get("fresh", [])
            state = case_rows.setdefault(case, {"runs": 0, "clean_runs": 0})
            state["runs"] += 1
            clean = all(o["passed"] for o in items) and all(n["passed"] for n in row.get("negatives", []) + row.get("run_time", []))
            state["clean_runs"] += clean
            for o in items:
                if o["category"] == "undeclared_trust":
                    undeclared.add(f"{o['id']}: {o['detail']}")
    # Rates are reported from the first serial replay; every other replay must reproduce it (M-04).
    first = next((r for r in w1 if r["mode"] == "serial" and r["run"] == 1), None)
    if first:
        for case, row in first["projection"]["cases"].items():
            items = row["positives"] + row.get("fresh", [])
            positives += sum(o["passed"] for o in items)
            positives_total += len(items)
            # DS-04's run-time cases count as negatives (M-03); their category is the runner's claim, not a
            # candidate diagnostic, so M-15 leaves them out.
            for n in row.get("negatives", []) + row.get("run_time", []):
                negatives_total += 1
                negatives += n["passed"]
                if n in row.get("negatives", []):
                    diagnostics_total += 1
                    conforming += bool(n["diagnostic_conforms"])
                for category in n["expected"]:
                    slot = by_category.setdefault(category, [0, 0])
                    slot[1] += 1
                    slot[0] += n["passed"] and n["category"] == category

    def agreement(mode: str) -> dict[str, Any]:
        digests = [r["projection_sha256"] for r in w1 if r["mode"] == mode]
        common = max((digests.count(d) for d in digests), default=0)
        return {"runs": len(digests), "matching": common, "digests": digests}

    serial, parallel = agreement("serial"), agreement("declared_parallel")
    m04 = serial["runs"] == 3 and parallel["runs"] == 3 and serial["matching"] == 3 and parallel["matching"] == 3
    w2_match = None
    if w2:
        reference = {r["mode"]: r["projection_sha256"] for r in w1 if r["run"] == 1}
        w2_match = all(reference.get(r["mode"]) == r["projection_sha256"] for r in w2)

    def cold_values(key: str) -> list[float]:
        return [r["totals"][key] for r in sorted(colds, key=lambda r: r["run"])]

    timed_by_case: dict[str, dict[str, Any]] = {}
    for case in TIMED_CASES:
        rows = sorted((r for r in timed if r["case"] == case), key=lambda r: r["pair"])
        if not rows:
            continue
        timed_by_case[case] = {
            "wall_ms": describe([r["totals"]["wall_ms"] for r in rows], seed),
            "cpu_ms": describe([r["totals"]["cpu_ms"] for r in rows], seed),
            "peak_rss_bytes": describe([r["totals"]["peak_rss_bytes"] for r in rows], seed),
            "completed": sum(bool(r.get("completed")) for r in rows),
        }

    fault_rows = {r["fault"]: r["met"] for r in faults}
    for case in PROOF_CASES:
        row = case_rows.get(case)
        if row is None:
            continue
        row["state"] = "pass" if row["runs"] and row["clean_runs"] == row["runs"] else "fail"
    ds06_ok = (len(colds) == 5 and all(r["build"]["completed"] for r in colds) and m04 and w2_match is True
               and len(fault_rows) == 9 and all(fault_rows.values())
               and all(v["completed"] == v["wall_ms"]["n"] for v in timed_by_case.values()))
    standalone_runs = [r["standalone"] for r in w1 + w2 if r.get("standalone", {}).get("status") == "run"]
    agreement_rows = {"matching": 0, "total": 0}
    host_pass: dict[str, bool] = {}
    ds05_negatives: dict[str, bool] = {}
    sizes_row: dict[str, Any] = {}
    for run in standalone_runs:
        for host, row in run["hosts"].items():
            agreement_rows["matching"] += row["agreement"]
            agreement_rows["total"] += row["total"]
            host_pass[host] = host_pass.get(host, True) and row["build"].get("built", False) and row["agreement"] == row["total"] and row["total"] > 0
            sizes_row.setdefault(host, {k: row["build"].get(k) for k in ("unstripped_bytes", "stripped_bytes", "elf", "closure", "runtime")})
        for n in run["negatives"]:
            key = n["id"] + (f" {n['host']}" if "host" in n else "")
            ds05_negatives[key] = ds05_negatives.get(key, True) and bool(n["passed"])
    if standalone_runs:
        ds05_ok = all(host_pass.get(h, False) for h in ("H-01", "H-02")) and all(ds05_negatives.values())
        case_rows["DS-05"] = {"state": "pass" if ds05_ok else "fail", "runs": len(standalone_runs), "hosts": host_pass, "negatives": ds05_negatives}
    else:
        case_rows["DS-05"] = {"state": "unresolved", "reason": "the standalone checker was not run"}
    case_rows["DS-06"] = {"state": "pass" if ds06_ok else "fail", "faults": fault_rows}
    case_rows["DS-07"] = {"state": "unresolved", "reason": "owner tasks (AM-02); no contributor rehearsal satisfies M-16"}
    passed_cases = sum(1 for c in CASES if case_rows.get(c, {}).get("state") == "pass")
    metrics = {
        "M-01": {"value": f"{passed_cases}/7", "gate": gate(False if any(case_rows.get(c, {}).get("state") == "fail" for c in CASES) else (True if passed_cases == 7 else None))},
        "M-02": {"passed": positives, "total": positives_total, "gate": gate(positives == positives_total and positives_total > 0)},
        "M-03": {"passed": negatives, "total": negatives_total, "by_category": by_category, "gate": gate(negatives == negatives_total and negatives_total > 0)},
        "M-04": {"serial": serial, "declared_parallel": parallel, "second_workspace_matches": w2_match, "gate": gate(m04 and w2_match is True)},
        "M-05": {**agreement_rows, "gate": gate(agreement_rows["matching"] == agreement_rows["total"] and agreement_rows["total"] > 0) if standalone_runs else "unresolved"},
        "M-06": {"count": len(undeclared), "items": sorted(undeclared), "gate": gate(not undeclared)},
        "M-07": {"runs": len(colds), "wall_ms": describe(cold_values("wall_ms"), seed), "cpu_ms": describe(cold_values("cpu_ms"), seed),
                 "peak_rss_bytes": describe(cold_values("peak_rss_bytes"), seed), "temp_bytes": describe(cold_values("temp_bytes"), seed),
                 "workspace_bytes": describe([r.get("workspace_bytes", 0) for r in colds], seed)},
        "M-08": {case: {"wall_ms": v["wall_ms"], "cpu_ms": v["cpu_ms"]} for case, v in timed_by_case.items()},
        "M-09": {case: v["peak_rss_bytes"] for case, v in timed_by_case.items()},
        "M-10": sizes_row or {"value": None, "reason": "DS-05 not run"},
        "M-14": {"hosts": host_pass, "required": ["H-01", "H-02"], "gate": gate(all(host_pass.get(h, False) for h in ("H-01", "H-02"))) if standalone_runs else "unresolved"},
        "M-15": {"conforming": conforming, "total": diagnostics_total, "gate": gate(conforming == diagnostics_total and diagnostics_total > 0)},
        "M-16": {"gate": "unresolved", "reason": "DS-07 is owner-performed (AM-02)"},
        "M-17": {"logic_kernel": "unavailable", "extraction_distribution": "unavailable", "comparative_decision": "unavailable"},
        "M-18": {"value": None, "reason": "same-owner maintenance tasks are owner-performed"},
    }
    return {"cases": case_rows, "metrics": metrics, "faults": fault_rows}


# ---------------------------------------------------------------------------
# DS-05: the standalone checker, its corpus and its negatives


EMULATOR = "/usr/bin/qemu-aarch64-static"
STRIP = {"H-01": "/usr/bin/strip", "H-02": "/usr/bin/aarch64-linux-gnu-strip"}
NO_TARGET = re.compile(r"Exec format error|cannot execute binary file|Could not open '|error while loading shared libraries|No such file or directory|not found|failed at execute \(errno (2|8|13)\)")


def closure(binary: Path, host: str) -> list[dict[str, Any]]:
    """The shared objects a dynamically linked checker loads."""

    rows = []
    if host == "H-01":
        done = subprocess.run(["/usr/bin/ldd", str(binary)], capture_output=True, text=True, check=False)
        for line in done.stdout.splitlines():
            match = re.search(r"(\S+) => (/\S+)|^\s*(/\S+) \(", line)
            if match:
                path = match.group(2) or match.group(3)
                rows.append({"name": match.group(1) or Path(path).name, "path": path, "sha256": file_sha256(Path(os.path.realpath(path)))})
        return rows
    done = subprocess.run(["/usr/bin/readelf", "-d", "-l", str(binary)], capture_output=True, text=True, check=False)
    for match in re.finditer(r"\(NEEDED\)\s+Shared library: \[([^\]]+)\]|program interpreter: ([^\]]+)\]", done.stdout):
        rows.append({"name": match.group(1) or match.group(2)})
    return rows


def sizes(binary: Path, host: str, scratch: Path) -> dict[str, Any]:
    copy = scratch / f"strip-{host}"
    shutil.copyfile(binary, copy)
    done = subprocess.run([STRIP[host], str(copy)], capture_output=True, check=False)
    stripped = copy.stat().st_size if done.returncode == 0 else None
    copy.unlink()
    return {"unstripped_bytes": binary.stat().st_size, "stripped_bytes": stripped, "elf": binary.read_bytes()[:4] == b"\x7fELF"}


@dataclass
class Checker:
    host: str
    binary: Path
    sha256: str
    launch: list[str]
    runtime: list[str]
    environment: dict[str, str]


def run_checker(epoch: Epoch, ctx: RunContext, checker: Checker, args: list[str], corpus: Path, label: str,
                verify: bool = True, launch: list[str] | None = None, runtime: list[str] | None = None) -> dict[str, Any]:
    """Launch the checker as distributed: digest-checked, with only its declared runtime readable."""

    if verify and file_sha256(checker.binary) != checker.sha256:
        return {"refused": "failed_certificate", "detail": "checker digest differs from the bootstrap manifest", "stdout": b"", "stderr": b"", "exit_status": None, "state": {"kind": "refused"}}
    scratch = lab_owned(ctx.run_root / "checker-tmp")
    env = {"PATH": "/usr/bin:/bin", "LANG": "C", "LC_ALL": "C", "TZ": "UTC", "HOME": str(scratch), "TMPDIR": str(scratch), **checker.environment}
    argv = [a.replace("{binary}", str(checker.binary)) for a in (launch if launch is not None else checker.launch)] + args
    box = Sandbox([str(checker.binary), *(runtime if runtime is not None else checker.runtime), str(corpus)], [str(scratch)])
    launched = epoch.launcher.run(argv, scratch, env, CEILINGS["measured_step"], box, ctx.cpus, scratch)
    ctx.recorder.add(label, launched, "measured_step", ctx.cpus, env, scratch)
    return launched


def build_checker(epoch: Epoch, cand: H.Candidate, ctx: RunContext, host: str, spec: dict[str, Any]) -> tuple[Checker | None, dict[str, Any]]:
    ctx.label = f"DS-05 {host} build"
    ok = True
    for step in spec.get("build", []):
        argv = [cand.fill(a, jobs=JOBS["declared_parallel" if len(ctx.cpus) > 1 else "serial"]) for a in step]
        launched = ctx.runner(argv, cand.src, cand.environment(spec.get("build_environment")), H.MEASURED_CEILING)
        if launched.exit_status != 0:
            ok = False
            break
    binary = cand.src / spec.get("binary", "")
    if not ok or not binary.is_file():
        return None, {"built": False}
    digest_value = file_sha256(binary)
    runtime = [cand.fill(r) for r in spec.get("runtime", [])]
    environment = {k: cand.fill(v) for k, v in spec.get("environment", {}).items()}
    checker = Checker(host, binary, digest_value, [cand.fill(a) for a in spec["launch"]], runtime, environment)
    return checker, {"built": True, "sha256": digest_value, **sizes(binary, host, ctx.run_root), "closure": closure(binary, host),
                     "runtime": [ctx.recorder.normalize(r) for r in runtime]}


def run_standalone(epoch: Epoch, cand: H.Candidate, ctx: RunContext, cases: dict[str, Any], fresh: str | None) -> dict[str, Any]:
    spec = cand.adapter.get("standalone")
    if not spec:
        return {"status": "not_provided"}
    in_prover = {}
    for row in cases.values():
        for o in row.get("positives", []) + row.get("fresh", []):
            in_prover[o["id"]] = o["passed"]
    corpus = ctx.run_root / "corpus"
    items = H.standalone_corpus(corpus, fresh)
    source_rows = tree_manifest(cand.src, spec.get("sources", []))
    hosts: dict[str, Any] = {}
    checkers: dict[str, Checker] = {}
    for host, host_spec in spec.get("hosts", {}).items():
        checker, build = build_checker(epoch, cand, ctx, host, host_spec)
        rows = []
        if checker:
            checkers[host] = checker
            for item in items:
                launched = run_checker(epoch, ctx, checker, item["args"], corpus, f"DS-05 {host} {item['id']}")
                out = launched["stdout"].decode("utf-8", "replace")
                got = out.rstrip("\n") if item.get("multiline") else out.strip()
                rows.append({"id": item["id"], "expected": item["expected"] if not item.get("multiline") else sha256(item["expected"].encode()),
                             "got": got if not item.get("multiline") else sha256(got.encode()), "exit": launched["exit_status"],
                             "matches_reference": launched["exit_status"] == 0 and got == item["expected"],
                             "in_prover": in_prover.get(item["in_prover"])})
            usage = run_checker(epoch, ctx, checker, [], corpus, f"DS-05 {host} usage")
            rows.append({"id": "D5-USAGE", "exit": usage["exit_status"], "matches_reference": usage["exit_status"] == 2 and bool(usage["stderr"].strip()) and not usage["stdout"], "in_prover": None})
        hosts[host] = {"build": {k: v for k, v in build.items() if k != "sha256"}, "binary_sha256": build.get("sha256"), "items": rows,
                       "agreement": sum(1 for r in rows if r["matches_reference"] and r["in_prover"] is not False),
                       "total": len(rows)}
    negatives = standalone_negatives(epoch, cand, ctx, spec, checkers, items, corpus, hosts)
    ctx.checkers, ctx.corpus_items, ctx.corpus = checkers, items, corpus
    return {"status": "run", "sources": source_rows, "hosts": hosts, "negatives": negatives}


def standalone_negatives(epoch: Epoch, cand: H.Candidate, ctx: RunContext, spec: dict[str, Any], checkers: dict[str, Checker],
                         items: list[dict[str, Any]], corpus: Path, hosts: dict[str, Any]) -> list[dict[str, Any]]:
    rows = []
    records_item = next(i for i in items if i["id"] == "D5-R-F01")

    def launch_failure(launched: dict[str, Any]) -> str | None:
        text = (launched["stdout"] + launched["stderr"]).decode("utf-8", "replace")
        if launched["exit_status"] in (126, 127) or (launched["exit_status"] != 0 and NO_TARGET.search(text)):
            return "unmet_target_assumption"
        return None

    # D5-N01: the AArch64 checker launched on the x86-64 host without its emulator.
    arm = checkers.get("H-02")
    if arm:
        launch = list(arm.launch)
        if launch and launch[0] == EMULATOR:
            launch = launch[1:]
            while launch and launch[0].startswith("-"):
                launch = launch[2:] if launch[0] in ("-L", "-E", "-U", "-cpu") else launch[1:]
        launched = run_checker(epoch, ctx, arm, records_item["args"], corpus, "D5-N01", launch=launch)
        category = launch_failure(launched)
        rows.append({"id": "D5-N01", "expected": ["unmet_target_assumption"], "category": category, "passed": category == "unmet_target_assumption"})
        # D5-N02: the declared runtime closure withheld (no sysroot, no runtime files).
        empty = lab_owned(ctx.run_root / "empty-sysroot")
        launch = [(str(empty) if i > 0 and arm.launch[i - 1] == "-L" else a) for i, a in enumerate(arm.launch)]
        launched = run_checker(epoch, ctx, arm, records_item["args"], corpus, "D5-N02", launch=launch, runtime=[])
        category = launch_failure(launched) or ("parse_failure" if launched["exit_status"] not in (0, None) else None)
        rows.append({"id": "D5-N02", "expected": ["unmet_target_assumption", "parse_failure"], "category": category,
                     "passed": category in ("unmet_target_assumption", "parse_failure"), "declared_runtime": [ctx.recorder.normalize(r) for r in arm.runtime]})
    else:
        rows += [{"id": "D5-N01", "category": None, "passed": False, "detail": "no H-02 checker"},
                 {"id": "D5-N02", "category": None, "passed": False, "detail": "no H-02 checker"}]
    native = checkers.get("H-01")
    # D5-N03: one byte of the checker changes; the digest check refuses it.
    if native:
        altered = ctx.run_root / "altered-checker"
        data = bytearray(native.binary.read_bytes())
        data[len(data) // 2] ^= 0x01
        altered.write_bytes(bytes(data))
        altered.chmod(0o755)
        launched = run_checker(epoch, ctx, Checker(native.host, altered, native.sha256, native.launch, native.runtime, native.environment), records_item["args"], corpus, "D5-N03")
        rows.append({"id": "D5-N03", "expected": ["failed_certificate"], "category": launched.get("refused"), "passed": launched.get("refused") == "failed_certificate"})
    # D5-N04 and D5-N05 are corpus items: R-F22 and every V-02.
    for host, row in hosts.items():
        by_id = {r["id"]: r for r in row["items"]}
        n04 = by_id.get("D5-R-F22")
        rows.append({"id": "D5-N04", "host": host, "expected_verdict": "reject unknown_version header",
                     "passed": bool(n04 and n04["got"] == "reject unknown_version header")})
        v02 = [by_id[k] for k in ("D5-G-V-02", "D5-F-V-02") if k in by_id]
        rows.append({"id": "D5-N05", "host": host, "expected_verdict": "the reference verdict for V-02",
                     "passed": bool(v02) and all(r["matches_reference"] for r in v02)})
    # D5-N06: the H-01 build with its declared dependency removed from a copy of the toolchain.
    dependency = spec.get("dependency")
    host_spec = spec.get("hosts", {}).get("H-01")
    if dependency and host_spec:
        mirror = lab_owned(ctx.run_root / "n06")
        toolchain_copy = mirror / "toolchain"
        subprocess.run(["cp", "-al", cand.toolchain, str(toolchain_copy)], check=True)
        removed = toolchain_copy / dependency
        if removed.is_dir():
            shutil.rmtree(removed)
        elif removed.exists():
            removed.unlink()
        copy = H.Candidate(cand.root, cand.adapter, str(toolchain_copy), mirror / "c", ctx.runner, cand.extra)
        shutil.copytree(cand.work, copy.work, symlinks=True)
        saved = ctx.toolchains
        ctx.toolchains = {**ctx.toolchains, "toolchain": str(toolchain_copy)}
        try:
            ctx.label = "D5-N06"
            launched = None
            for step in host_spec.get("build", []):
                argv = [copy.fill(a, jobs="1") for a in step]
                launched = ctx.runner(argv, copy.src, copy.environment(host_spec.get("build_environment")), H.MEASURED_CEILING)
                if launched.exit_status != 0:
                    break
        finally:
            ctx.toolchains = saved
        category = None
        if launched is not None and launched.exit_status != 0:
            message = (H.errors(cand.lang.name, launched) or [H.Message("error", None, None, launched.output[:4000])])[0]
            category = H.classify(cand.adapter, launched, message, "build")
        rows.append({"id": "D5-N06", "expected": ["parse_failure", "unsupported_feature"], "category": category,
                     "passed": category in ("parse_failure", "unsupported_feature"), "removed": dependency})
        shutil.rmtree(mirror)
    else:
        rows.append({"id": "D5-N06", "category": None, "passed": False, "detail": "no declared dependency"})
    # D5-N07: a toolchain archive whose digest differs from the manifest is refused before unpacking.
    part = epoch.packet["archives"][ctx.cand][0]
    altered_digest = sha256_with_flip(epoch.archive / part["archive"], part["bytes"] // 2)
    refused = altered_digest != part["sha256"]
    rows.append({"id": "D5-N07", "expected": ["failed_certificate"], "category": "failed_certificate" if refused else None, "passed": refused,
                 "detail": "the verified unpack compares the archive's SHA-256 with the packet before extracting"})
    return rows


def sha256_with_flip(path: Path, offset: int) -> str:
    """The digest of an archive whose byte at ``offset`` is altered, streamed without a copy."""

    h = hashlib.sha256()
    position = 0
    with open(path, "rb") as handle:
        for block in iter(lambda: handle.read(1 << 20), b""):
            if position <= offset < position + len(block):
                block = bytearray(block)
                block[offset - position] ^= 0x01
                block = bytes(block)
            h.update(block)
            position += len(block)
    return h.hexdigest()


def standalone_projection(standalone: dict[str, Any]) -> dict[str, Any]:
    if standalone.get("status") != "run":
        return standalone
    return {"hosts": {host: row["items"] for host, row in standalone["hosts"].items()}, "negatives": standalone["negatives"]}


def source_surface(root: Path, adapter: dict[str, Any]) -> dict[str, Any]:
    """M-11: UTF-8 lines and bytes by role; audit planning only, no better-or-worse label."""

    roles = adapter.get("surface") or {
        "proof": ["theories/*.v", "D006/*.lean"],
        # DS-05: entry points, extraction or compilation commands and the driver of the standalone checker
        "checker": ["extraction/*", "Checker/**/*"],
        "trusted": ["Loader/*"],
        "test": [],
        "generated": [],
        "glue": ["adapter.json", "adapter.d/*.json", "patches/*"],
    }
    rows: dict[str, Any] = {}
    seen: set[Path] = set()
    for role, patterns in roles.items():
        files = sorted({p for pattern in patterns for p in root.glob(pattern) if p.is_file()} - seen)
        seen |= set(files)
        data = [p.read_bytes() for p in files]
        rows[role] = {"files": len(files), "lines": sum(d.count(b"\n") for d in data), "bytes": sum(len(d) for d in data),
                      "paths": [p.relative_to(root).as_posix() for p in files]}
    return rows


def dpkg_size(package: str) -> int | None:
    done = subprocess.run(["dpkg-query", "-W", "-f=${Installed-Size}", package], capture_output=True, text=True, check=False)
    return int(done.stdout) * 1024 if done.returncode == 0 and done.stdout.strip().isdigit() else None


def dependency_closure(cand: str, packet: dict[str, Any], toolchains: dict[str, Any], overlay: dict[str, Any], adapter: dict[str, Any]) -> dict[str, Any]:
    """M-12 and M-13: every component, its role, bytes, retrievability and terms."""

    tools = {t["id"]: t for t in toolchains["tools"]}
    wanted = next(c["toolchain"] for c in overlay["candidates"] if c["id"] == cand)
    archived = {tid: part for part in packet["archives"][cand] for tid in part["tools"]}
    components = []
    for tid in wanted:
        tool = tools[tid]
        acquisition = tool["acquisition"]
        row: dict[str, Any] = {"id": tid, "name": tool["name"], "role": tool.get("role", "prover and kernel"), "terms": tool.get("terms")}
        if tid in archived:
            part = archived[tid]
            shared_with = [t for t in part["tools"] if t != tid]
            row.update(archive=part["archive"], sha256=part["sha256"], bytes=0 if shared_with and shared_with[0] < tid else part["bytes"],
                       retrievable=True, retrieval="content-addressed archive in the epoch")
        elif acquisition.get("kind") == "distribution_package":
            sizes_ = {name: dpkg_size(name) for name in acquisition["packages"]}
            row.update(packages=acquisition["packages"], bytes=sum(v or 0 for v in sizes_.values()),
                       retrievable=all(v is not None for v in sizes_.values()), retrieval="Ubuntu archive, exact versions")
        else:
            row.update(bytes=None, retrievable=False)
        components.append(row)
    for path in adapter.get("host_files", []):
        target = Path(path)
        components.append({"id": path, "name": path, "role": "host configuration", "terms": "distribution file",
                           "bytes": target.stat().st_size if target.exists() else None, "sha256": file_sha256(target) if target.exists() else None,
                           "retrievable": target.exists(), "retrieval": "hashed in the epoch"})
    by_role: dict[str, dict[str, int]] = {}
    for row in components:
        slot = by_role.setdefault(row["role"], {"components": 0, "bytes": 0})
        slot["components"] += 1
        slot["bytes"] += row["bytes"] or 0
    resolved = [row for row in components if row.get("terms")]
    return {
        "M-12": {"components": components, "by_role": by_role, "total_bytes": sum(r["bytes"] or 0 for r in components),
                 "gate": gate(all(r["retrievable"] for r in components))},
        "M-13": {"resolved": len(resolved), "total": len(components), "d018_admission": "absent (owner disposition)",
                 "gate": gate(len(resolved) == len(components))},
    }


def comparative(summaries: dict[str, Any], records: list[dict[str, Any]], seed: int) -> dict[str, Any]:
    def pairs(profile: str, key: str, case: str | None = None, field_: str = "wall_ms") -> list[tuple[float, float]]:
        rows = [r for r in records if r["profile"] == profile and not r.get("warmup") and (case is None or r.get("case") == case)]
        index = "run" if profile == "cold_bootstrap" else "pair"
        a = {r[index]: r["totals"][field_] for r in rows if r["candidate"] == "C-01"}
        b = {r[index]: r["totals"][field_] for r in rows if r["candidate"] == "C-02"}
        return [(a[k], b[k]) for k in sorted(set(a) & set(b))]

    out: dict[str, Any] = {}
    m07 = paired_ratio(pairs("cold_bootstrap", "run"), seed)
    out["M-07"] = {**m07, "measure": "cold bootstrap wall time", "label": band_label("interval", m07)}
    out["M-08"] = {}
    out["M-09"] = {}
    for case in TIMED_CASES:
        wall = paired_ratio(pairs("timed_replay", "pair", case), seed)
        if wall["pairs"]:
            out["M-08"][case] = {**wall, "label": band_label("interval", wall)}
        rss = paired_ratio(pairs("timed_replay", "pair", case, "peak_rss_bytes"), seed)
        if rss["pairs"]:
            out["M-09"][case] = {**rss, "label": band_label("interval", rss)}

    def exact(value_a: float | None, value_b: float | None) -> dict[str, Any]:
        r = value_a / value_b if value_a and value_b else None
        return {"r": r, "label": band_label("exact", {"r": r})}

    size = lambda c: (summaries[c]["metrics"]["M-10"].get("H-01") or {}).get("unstripped_bytes") if c in summaries else None  # noqa: E731
    out["M-10"] = {**exact(size("C-01"), size("C-02")), "measure": "H-01 checker bytes, unstripped"}
    closure_bytes = lambda c: summaries[c]["metrics"]["M-12"]["total_bytes"] if c in summaries else None  # noqa: E731
    out["M-12"] = {**exact(closure_bytes("C-01"), closure_bytes("C-02")), "measure": "dependency closure bytes"}
    out["M-18"] = {"r": None, "label": "inconclusive", "reason": "a value is missing: same-owner maintenance is owner-performed"}
    return out


HARD_GATES = (
    ("HG-1", "DS-01 to DS-07 complete for both candidates", ("M-01",)),
    ("HG-2", "every positive observation and canonical result agrees", ("M-02", "M-05")),
    ("HG-3", "every negative rejects in its expected category", ("M-03", "M-15")),
    ("HG-4", "no omitted axiom, feature, evaluator, runtime or tool", ("M-06",)),
    ("HG-5", "clean bootstrap and replay deterministic in two workspaces (level 2)", ("M-04",)),
    ("HG-6", "standalone checker agrees on the corpus and supports every host", ("M-05", "M-14")),
    ("HG-7", "bytes, sources, tools, dependencies, licenses and provenance inventoried", ("M-12", "M-13")),
    ("HG-8", "owner review scopes and DS-07 complete", ("M-16",)),
)


def combine(states: list[str]) -> str:
    if "fail" in states:
        return "fail"
    if "unresolved" in states or not states:
        return "unresolved"
    return "pass"


def command_summarize(repo: Path, archive: Path) -> dict[str, Any]:
    summary = build_summary(repo, archive)
    (archive / "summary.json").write_bytes(canonical_file(summary))
    return summary


def comparative_rows(table: dict[str, Any]) -> list[dict[str, Any]]:
    """Every labelled comparative row, flattening per-case metrics."""

    rows = []
    for value in table.values():
        if "label" in value:
            rows.append(value)
        else:
            rows.extend(v for v in value.values() if isinstance(v, dict) and "label" in v)
    return rows


def full_plan() -> dict[str, Any]:
    return {k: list(v) if isinstance(v, tuple) else v for k, v in asdict(Plan()).items() if k != "revision"}


def build_summary(repo: Path, archive: Path) -> dict[str, Any]:
    packet = json.loads((archive / "packet.json").read_text())
    H.REPO, H.SHARED = repo, repo / SHARED_DIR
    every = load_records(archive)
    starts = [i for i, r in enumerate(every) if r.get("profile") == "execution"]
    records = every[starts[-1]:] if starts else every
    attempts = [{"attempt": every[i]["attempt"], "revision": every[i]["revision"], "plan": every[i]["plan"],
                 "records": (starts[n + 1] if n + 1 < len(starts) else len(every)) - i} for n, i in enumerate(starts)]
    seed = packet["bootstrap_seed"]
    overlay = json.loads((repo / OVERLAY_PATH).read_text())
    toolchains = json.loads((repo / TOOLCHAINS_PATH).read_text())
    candidates = sorted({r["candidate"] for r in records if r.get("candidate") in LANGUAGE})
    summaries: dict[str, Any] = {}
    for cand in candidates:
        root = repo / CANDIDATE_DIRS[cand]
        if not root.exists() and packet.get("dev_candidates"):
            root = Path(packet["dev_candidates"][cand])
        # Candidate trees are bound per run (AM-04): summarize reads the tree the latest run provisioned.
        provisioned = [r for r in records if r.get("profile") == "provision" and r.get("candidate") == cand]
        if provisioned and source_rows(root) != [(row["path"], row["sha256"]) for row in provisioned[-1]["checkout"]["candidate_tree"]]:
            raise RunError(f"{cand}'s tree at {root} is not the tree its latest run provisioned")
        adapter = H.load_adapter(root)
        row = candidate_summary(cand, records, packet, seed)
        row["metrics"].update(dependency_closure(cand, packet, toolchains, overlay, adapter))
        row["metrics"]["M-11"] = {"roles": source_surface(root, adapter), "label": "none: audit planning only"}
        row["hard_gates"] = [{"id": gid, "rule": text, "metrics": list(ids),
                              "state": combine([row["metrics"][m]["gate"] for m in ids if "gate" in row["metrics"].get(m, {})])}
                             for gid, text, ids in HARD_GATES]
        row["eligible"] = all(g["state"] == "pass" for g in row["hard_gates"])
        summaries[cand] = row
    table = comparative(summaries, records, seed) if len(summaries) == 2 else {}
    reasons = []
    for cand, row in summaries.items():
        for g in row["hard_gates"]:
            if g["state"] != "pass":
                reasons.append(f"{cand} {g['id']} {g['state']}")
    full = bool(attempts) and {k: v for k, v in attempts[-1]["plan"].items() if k != "revision"} == full_plan()
    if not full:
        reasons.append("the latest attempt did not run the full preregistered plan")
    reasons += ["D-004 and D-005 are not Accepted", "no D-018 owner admission exists for any tool",
                "R-01 to R-09 owner reviews are absent", "M-17 independent review is unavailable"]
    both_eligible = full and len(summaries) == 2 and all(r["eligible"] for r in summaries.values())
    # Section 8: tie only when complete eligible evidence does not distinguish the candidates; a
    # recommendation needs the owner's per-axis rationale, which no runner writes.
    labels = [row["label"] for row in comparative_rows(table)]
    conclusion = "tie" if both_eligible and labels and all(label == "practically_equivalent" for label in labels) else "inconclusive"
    if both_eligible and conclusion == "inconclusive":
        reasons.append("the owner's per-axis rationale (section 8, step 5) is absent")
    summary = {
        "schema_version": "d006-v0.3-summary-1", "suite_version": SUITE_VERSION, "epoch": packet["epoch"],
        "revision": packet["revision"], "packet_sha256": file_sha256(archive / "packet.json"),
        "records": len(records), "attempts": attempts, "dev": packet.get("dev", False),
        "full_protocol": full,
        "candidates": summaries, "comparative": table,
        "conclusion": conclusion,
        "conclusion_reasons": reasons,
        "owner_labels": {"producer_label": "contributor_produced", "review_label": "unreviewed", "independent_review": "unavailable"},
        "nonclaims": [
            "no proof foundation is selected, preferred or recommended",
            "D-006 stays investigate; only an Accepted OEP can close it",
            "same-workspace and same-owner evidence is never independent reproduction",
            "H-02 timings are emulated and never compared with H-01's",
        ],
    }
    return summary


EXPORT_SCHEMA = "d006-v0.3-export-1"
EXPORT_CHUNK = 384 * 1024


def chunked(lines: list[bytes], stem: str) -> dict[str, bytes]:
    """JSON lines split into files of at most EXPORT_CHUNK bytes each, in order."""

    files: dict[str, bytes] = {}
    current: list[bytes] = []
    size = 0
    for line in lines:
        if current and size + len(line) > EXPORT_CHUNK:
            files[f"{stem}-{len(files) + 1:02d}.jsonl"] = b"".join(current)
            current, size = [], 0
        current.append(line)
        size += len(line)
    if current:
        files[f"{stem}-{len(files) + 1:02d}.jsonl"] = b"".join(current)
    return files


def command_export(repo: Path, archive: Path, dest: Path) -> Path:
    """The committed form of a verified archive: packet, records, logs and summary, without toolchains."""

    problems = command_verify(repo, archive)
    if problems:
        raise RunError(f"the archive does not verify: {problems[0]}")
    if dest.exists():
        raise RunError(f"{dest} already exists")
    files = {"packet.json": (archive / "packet.json").read_bytes()}
    objects: dict[str, Any] = {}
    files.update(chunked([canonical(pack_record(r, objects)) + b"\n" for r in load_records(archive)], "records"))
    files.update(chunked([canonical({"sha256": key, "value": value}) + b"\n" for key, value in sorted(objects.items())], "objects"))
    rows = []
    for name, data in sorted(load_logs(archive).items()):
        try:
            rows.append(canonical({"sha256": name, "bytes": len(data), "text": data.decode("utf-8")}) + b"\n")
        except UnicodeDecodeError:
            rows.append(canonical({"sha256": name, "bytes": len(data), "base64": base64.b64encode(data).decode("ascii")}) + b"\n")
    files.update(chunked(rows, "logs"))
    files["summary.json"] = canonical_file(build_summary(repo, archive))
    dest.mkdir(parents=True)
    for name, data in files.items():
        (dest / name).write_bytes(data)
    manifest = {"schema_version": EXPORT_SCHEMA, "epoch": json.loads(files["packet.json"])["epoch"],
                "archive_manifest_sha256": file_sha256(archive / "manifest.json"),
                "files": [{"path": name, "size": len(data), "sha256": sha256(data)} for name, data in sorted(files.items())]}
    (dest / "manifest.json").write_bytes(canonical_file(manifest))
    return dest


def command_verify(repo: Path, archive: Path) -> list[str]:
    problems = []
    packet = json.loads((archive / "packet.json").read_text())
    identity = {key: packet[key] for key in IDENTITY_KEYS if key in packet}
    if epoch_name(identity) != packet["epoch"] or bootstrap_seed(identity) != packet["bootstrap_seed"]:
        problems.append("the packet's epoch name or bootstrap seed is not the hash of what it binds")
    known = subprocess.run(["git", "-C", str(repo), "cat-file", "-e", f"{packet['revision']}^{{commit}}"], capture_output=True, check=False)
    if not packet.get("dev") and known.returncode == 0:
        for row in packet["bindings"]:
            bound = subprocess.run(["git", "-C", str(repo), "show", f"{packet['revision']}:{row['path']}"], capture_output=True, check=False)
            if bound.returncode != 0 or sha256(bound.stdout) != row["sha256"]:
                problems.append(f"{row['path']} at {packet['revision'][:12]} is not the bound file")
    manifest = json.loads((archive / "manifest.json").read_text())
    listed = set()
    for row in manifest["files"]:
        listed.add(row["path"])
        path = archive / row["path"]
        if not path.is_file() or file_sha256(path) != row["sha256"] or path.stat().st_size != row["size"]:
            problems.append(f"{row['path']} differs from the archive manifest")
    if manifest.get("schema_version") == EXPORT_SCHEMA:
        extra = sorted(p.name for p in archive.iterdir() if p.name not in listed | {"manifest.json"})
        problems += [f"{name} is not in the export manifest" for name in extra]
    logs = load_logs(archive)
    for name, data in logs.items():
        if sha256(data) != name:
            problems.append(f"log {name[:12]} does not match its digest")
    for name, value in load_objects(archive).items():
        if digest(value) != name:
            problems.append(f"object {name[:12]} does not match its digest")
    try:
        records = load_records(archive)
    except RunError as exc:
        return problems + [str(exc)]
    for record in records:
        for step in record.get("steps", []):
            for stream in ("stdout", "stderr"):
                if step[stream]["sha256"] not in logs:
                    problems.append(f"record {record.get('ordinal')} step {step['ordinal']} {stream} log is missing")
        if record.get("schema_version") != RECORD_SCHEMA or record.get("epoch") != json.loads((archive / "packet.json").read_text())["epoch"]:
            problems.append(f"record {record.get('ordinal')} is not a {RECORD_SCHEMA} record of this epoch")
        if record.get("profile") == "deterministic_replay" and digest(record["projection"]) != record["projection_sha256"]:
            problems.append(f"record {record['ordinal']} projection digest mismatch")
    if (archive / "summary.json").exists():
        try:
            if canonical_file(build_summary(repo, archive)) != (archive / "summary.json").read_bytes():
                problems.append("summary.json does not regenerate byte for byte from the records")
        except RunError as exc:
            problems.append(f"summary.json cannot be regenerated: {exc}")
    return problems


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
