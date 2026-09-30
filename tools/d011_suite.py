"""D-011 native target envelope: the suite packet, its checker, and the laboratory.

Usage:
  python3 tools/d011_suite.py generate
  python3 tools/d011_suite.py check
  python3 tools/d011_suite.py owner-template
  python3 tools/d011_suite.py run --profile dev|measured [--owner-input NAME]
  python3 tools/d011_suite.py export EPOCH
  python3 tools/d011_suite.py verify EPOCH

`generate` writes research/decisions/D-011/d011-v0.1/suite-packet.json from the
tables below and the bound input files; `check` requires the committed packet
to equal that output byte for byte and to pass the structural checks.
`owner-template` prints an empty owner input on standard output. `run` builds
the stand-in kernels for every target tuple, runs the eight cases, and writes a
content-addressed epoch archive with a summary under ARCHIVE_ROOT. `export`
writes what the repository keeps of a verified archive to
research/decisions/D-011/d011-v0.1/run/EPOCH, and `verify` re-hashes an archive
or, when no archive of that name exists, its export, and recomputes the summary.

Command-line arguments never become filesystem paths or command elements. An
epoch or owner input is named by an entry that already exists under
ARCHIVE_ROOT (owner inputs under ARCHIVE_ROOT/owner-input, exports under the
run directory above), orangec is read from compiler/target/release/orangec in
this repository, and working files live under ARCHIVE_ROOT/work.

What the laboratory measures is target feasibility, not Orange code
generation. The kernels under research/decisions/D-011/d011-v0.1/kernels are
contributor-written portable C reference kernels standing in for the code
Orange does not yet generate. Everything this tool writes is
contributor-produced and unreviewed; no record it writes is an owner decision,
an owner review or a freeze. Emulated timings are never compared with native
timings.

The generator and checker use only the standard library and never run a
subprocess. The laboratory runs every compiler, linker, emulator and kernel
binary inside the repository's unchanged filesystem sandbox (tools/fs_sandbox.c)
under fresh user, mount, PID, IPC, UTS and network namespaces with no
capabilities.
"""

from __future__ import annotations

import ctypes
import gzip
import hashlib
import hmac
import json
import os
import platform
import pwd
import re
import selectors
import shutil
import signal
import struct
import subprocess
import sys
import time
from pathlib import Path
from typing import Any, Callable, Iterable

REPOSITORY_ROOT = Path(__file__).resolve().parents[1]
SUITE_VERSION = "d011-v0.1"
LAB = "research/decisions/D-011/d011-v0.1"
PACKET_PATH = f"{LAB}/suite-packet.json"
KERNEL_DIR = f"{LAB}/kernels"
KERNEL_FILES = ("d011_kernels.h", "kernels.c", "accel.c", "runtime.c", "abi_probe.rs")
SUITE_DOC = "docs/NATIVE_TARGET_DECISION_SUITE.md"
BOUND_INPUTS = tuple(f"{KERNEL_DIR}/{name}" for name in KERNEL_FILES) + (SUITE_DOC,)
BASE_REVISION = "59caa344e176bc6d8f4e5429a00f3d82eeb2799a"
PACKET_SCHEMA = "d011-v0.1-suite-packet-1"
OWNER_SCHEMA = "d011-v0.1-owner-input-1"
RECORD_SCHEMA = "d011-v0.1-record-1"
SUMMARY_SCHEMA = "d011-v0.1-summary-1"
LABEL = "contributor-produced, unreviewed"
MAX_SAFE = (1 << 53) - 1
ARCHIVE_ROOT = Path("/tmp/orange-d011")
OWNER_INPUT_DIR = ARCHIVE_ROOT / "owner-input"
WORK_DIR = ARCHIVE_ROOT / "work"
ORANGEC = REPOSITORY_ROOT / "compiler" / "target" / "release" / "orangec"
MANIFEST_NAME = "archive-manifest.json"


class SuiteError(Exception):
    """The packet or its bound inputs are not what the suite requires."""


class RunError(Exception):
    """A runner or environment defect: the epoch is invalid, not a candidate result."""


# ---------------------------------------------------------------------------
# Canonical JSON and digests


def canonical(value: Any) -> bytes:
    return json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=False).encode("utf-8")


def canonical_file(value: Any) -> bytes:
    return canonical(value) + b"\n"


def gate0_numbers(value: Any) -> Any:
    """Gate 0's JSON profile has no non-integers and no integers beyond 2**53 - 1; records carry those as
    decimal strings (floats by their shortest round-trip form)."""

    if isinstance(value, float):
        return repr(value)
    if isinstance(value, int) and not isinstance(value, bool) and abs(value) > MAX_SAFE:
        return str(value)
    if isinstance(value, dict):
        return {key: gate0_numbers(item) for key, item in value.items()}
    if isinstance(value, list):
        return [gate0_numbers(item) for item in value]
    return value


def sha256_hex(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def file_sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1 << 20), b""):
            digest.update(chunk)
    return digest.hexdigest()


def gate0_json_errors(value: Any, where: str = "$") -> list[str]:
    """Committed JSON holds no floats and no integers beyond 2**53 - 1."""

    if isinstance(value, bool) or value is None or isinstance(value, str):
        return []
    if isinstance(value, float):
        return [f"{where}: float"]
    if isinstance(value, int):
        return [] if -MAX_SAFE <= value <= MAX_SAFE else [f"{where}: integer out of range"]
    if isinstance(value, list):
        return [e for i, item in enumerate(value) for e in gate0_json_errors(item, f"{where}[{i}]")]
    if isinstance(value, dict):
        return [e for k, item in value.items() for e in gate0_json_errors(item, f"{where}.{k}")]
    return [f"{where}: {type(value).__name__}"]


# ---------------------------------------------------------------------------
# The decision boundary: candidates, target tuples, profiles and toolchains

CANDIDATES = (
    ("TE-01", "x86-64 and AArch64", ("T-X64", "T-A64"),
     "The envelope docs/DECISIONS.md currently recommends for D-011. Being the current recommendation "
     "carries no weight in this suite."),
    ("TE-02", "x86-64 only", ("T-X64",),
     "The single most common server and desktop tuple; one claim-bearing native target."),
    ("TE-03", "AArch64 only", ("T-A64",),
     "The single most common mobile and growing server tuple; one claim-bearing native target."),
    ("TE-04", "Portable C interoperability only", (),
     "No claim-bearing native target at 1.0: Orange emits portable C behind the stable C ABI, the "
     "user's compiler owns machine code, and every native constant-time claim is unsupported."),
    ("TE-05", "x86-64, AArch64 and RV64GC", ("T-X64", "T-A64", "T-RV64"),
     "TE-01 plus RV64GC, the base of the RVA20 and RVA22 Linux profiles, with scalar-crypto "
     "extensions for the crypto profile. Kept as briefed: RV64GC is an open ISA with a public golden "
     "reference model (sail-riscv) and a ratified data-independent-timing extension (Zkt), as recorded "
     "from knowledge and not checked from this machine."),
)

TUPLES = (
    {
        "id": "T-X64", "name": "x86-64 Linux", "arch": "x86_64", "triple": "x86_64-unknown-linux-gnu",
        "abi": "System V AMD64 psABI, LP64", "object_format": "ELF64 little-endian",
        "isa_base": "x86-64 baseline (x86-64-v1)", "isa_crypto": "x86-64 baseline plus AES-NI and PCLMULQDQ",
        "toolchains": ["TC-01", "TC-04"], "rust_target": "x86_64-unknown-linux-gnu",
        "emulator": "/usr/bin/qemu-x86_64-static", "emulator_cpu": "max", "feature_negative_cpu": "qemu64",
        "dit_mechanism": "Intel DOITM (IA32_UARCH_MISC_CTL); AMD publishes separate guidance",
    },
    {
        "id": "T-A64", "name": "AArch64 Linux", "arch": "aarch64", "triple": "aarch64-unknown-linux-gnu",
        "abi": "AAPCS64, LP64", "object_format": "ELF64 little-endian",
        "isa_base": "Armv8-A", "isa_crypto": "Armv8-A plus FEAT_AES and FEAT_PMULL",
        "toolchains": ["TC-02", "TC-04"], "rust_target": "aarch64-unknown-linux-gnu",
        "emulator": "/usr/bin/qemu-aarch64-static", "emulator_cpu": "max", "feature_negative_cpu": None,
        "dit_mechanism": "FEAT_DIT (PSTATE.DIT, Armv8.4-A)",
    },
    {
        "id": "T-RV64", "name": "RV64GC Linux", "arch": "riscv64", "triple": "riscv64-unknown-linux-gnu",
        "abi": "RISC-V ELF psABI, LP64D", "object_format": "ELF64 little-endian",
        "isa_base": "RV64GC (RVA20U64 base)", "isa_crypto": "RV64GC plus Zkne and Zbkc",
        "toolchains": ["TC-03", "TC-04"], "rust_target": "riscv64gc-unknown-linux-gnu",
        "emulator": "/usr/bin/qemu-riscv64-static", "emulator_cpu": "max", "feature_negative_cpu": "rv64",
        "dit_mechanism": "Zkt (data-independent execution latency, scalar cryptography v1.0)",
    },
)
TUPLE_IDS = tuple(t["id"] for t in TUPLES)
TUPLE_BY_ID = {t["id"]: t for t in TUPLES}
TUPLE_BY_ARCH = {t["arch"]: t["id"] for t in TUPLES}

PORTABLE_PATH = {
    "id": "C-PORTABLE",
    "name": "Portable C behind the stable C ABI",
    "shared_by": [c[0] for c in CANDIDATES],
    "evidence": "The same kernels built by every available C toolchain for the laboratory host's own tuple "
                "and called from Rust through extern \"C\" (the Rust ABI probe). Every candidate includes this "
                "path, so its results are identical for all five.",
    "machine_code_claims": False,
}

PROFILES = (
    {"id": "P-BASE", "name": "Baseline ISA", "defines": [], "objects": "kernels and runtime",
     "summary": "The baseline ISA of the tuple; portable kernels only; accelerated operations answer "
                "unsupported."},
    {"id": "P-CRYPTO", "name": "Crypto extension", "defines": ["D011_CRYPTO_PROFILE"],
     "objects": "kernels, crypto glue, target instructions and runtime",
     "summary": "The baseline plus the tuple's AES and carry-less-multiply extension; adds the accelerated "
                "AES and AES-GCM operations."},
)
PROFILE_IDS = tuple(p["id"] for p in PROFILES)

# Toolchains, as found on the contributor's laboratory host. "present" is what the
# contributor observed when authoring; every run records what it actually finds.
TOOLCHAINS = (
    {"id": "TC-01", "name": "GNU C 13.3 for x86-64 with GNU binutils 2.42", "role": "C compiler and linker for T-X64",
     "path": "/usr/bin/x86_64-linux-gnu-gcc-13", "kind": "gcc",
     "packages": ["gcc-13 13.3.0-6ubuntu2~24.04.1", "binutils-x86-64-linux-gnu 2.42-4ubuntu2.10"],
     "terms": "GPL-3.0-or-later with the GCC Runtime Library Exception; binutils GPL-3.0-or-later",
     "present": True, "acquisition": "distribution package"},
    {"id": "TC-02", "name": "GNU C 13.3 cross for AArch64 with GNU binutils 2.42", "role": "C compiler and linker for T-A64",
     "path": "/usr/bin/aarch64-linux-gnu-gcc-13", "kind": "gcc",
     "packages": ["gcc-13-aarch64-linux-gnu 13.3.0-6ubuntu2~24.04.1cross1", "binutils-aarch64-linux-gnu 2.42-4ubuntu2.10"],
     "terms": "GPL-3.0-or-later with the GCC Runtime Library Exception; binutils GPL-3.0-or-later",
     "present": True, "acquisition": "distribution package"},
    {"id": "TC-03", "name": "GNU C 13 cross for RV64", "role": "C compiler and linker for T-RV64",
     "path": "/usr/bin/riscv64-linux-gnu-gcc-13", "kind": "gcc",
     "packages": ["gcc-13-riscv64-linux-gnu (not installed)"],
     "terms": "GPL-3.0-or-later with the GCC Runtime Library Exception",
     "present": False, "acquisition": "distribution package gcc-13-riscv64-linux-gnu; not installed on the laboratory host"},
    {"id": "TC-04", "name": "Clang and LLD 18.1.3", "role": "C compiler and linker for every tuple",
     "path": "/usr/bin/clang-18", "kind": "clang",
     "packages": ["clang-18 1:18.1.3-1ubuntu1", "lld-18 1:18.1.3-1ubuntu1"],
     "terms": "Apache-2.0 WITH LLVM-exception", "present": True, "acquisition": "distribution package"},
    {"id": "TC-05", "name": "LLVM binary tools 18.1.3 (llvm-objdump, llvm-readelf)", "role": "instruction inventory and relocations",
     "path": "/usr/bin/llvm-objdump-18", "kind": "tool",
     "packages": ["llvm-18 1:18.1.3-1ubuntu1"], "terms": "Apache-2.0 WITH LLVM-exception",
     "present": True, "acquisition": "distribution package"},
    {"id": "TC-06", "name": "QEMU user-mode emulators 8.2.2", "role": "emulated execution and control-flow traces",
     "path": "/usr/bin/qemu-x86_64-static", "kind": "tool",
     "packages": ["qemu-user-static 1:8.2.2+ds-0ubuntu1.18"], "terms": "GPL-2.0-only (with parts under other terms)",
     "present": True, "acquisition": "distribution package"},
    {"id": "TC-07", "name": "Rust 1.96.1 (x86_64-unknown-linux-gnu standard library only)", "role": "Rust ABI probe",
     "path": "rustc (rustup toolchain 1.96.1)", "kind": "rustc",
     "packages": ["rustup toolchain 1.96.1-x86_64-unknown-linux-gnu"], "terms": "MIT OR Apache-2.0",
     "present": True, "acquisition": "rustup; aarch64-unknown-linux-gnu and riscv64gc-unknown-linux-gnu standard "
                                      "libraries are not installed"},
    {"id": "TC-08", "name": "Python 3 (hashlib, hmac)", "role": "laboratory runner and library oracle",
     "path": "the running interpreter", "kind": "tool", "packages": ["python3.11 or python3.12"],
     "terms": "PSF-2.0", "present": True, "acquisition": "distribution package"},
    {"id": "TC-09", "name": "OpenSSL libcrypto 3.0.13", "role": "library oracle (ChaCha20, Poly1305, AEADs, AES, X25519)",
     "path": "libcrypto.so.3", "kind": "tool", "packages": ["libssl3t64 3.0.13-0ubuntu3.7"], "terms": "Apache-2.0",
     "present": True, "acquisition": "distribution package"},
    {"id": "TC-10", "name": "orangec built from the base revision", "role": "Orange reference-evaluation oracle",
     "path": "compiler/target/release/orangec", "kind": "tool", "packages": ["this repository at the base revision"],
     "terms": "the repository's own terms", "present": True, "acquisition": "cargo build --release -p orangec"},
    {"id": "TC-11", "name": "fs-sandbox from tools/fs_sandbox.c", "role": "filesystem sandbox for every step",
     "path": "built per epoch by /usr/bin/cc", "kind": "tool", "packages": ["this repository"],
     "terms": "the repository's own terms", "present": True, "acquisition": "built per epoch"},
    {"id": "TC-12", "name": "util-linux 2.39.3 unshare and setpriv", "role": "namespaces and privilege drop",
     "path": "/usr/bin/unshare", "kind": "tool", "packages": ["util-linux 2.39.3-9ubuntu6.5"],
     "terms": "GPL-2.0-or-later", "present": True, "acquisition": "distribution package"},
)
TOOLCHAIN_BY_ID = {t["id"]: t for t in TOOLCHAINS}

KERNEL_PARTS = ("SHA2", "HMAC_HKDF", "CHACHA_POLY", "X25519", "AES_GCM", "KECCAK", "CT")
ACCEL_PARTS = ("GCM_GLUE", "ACCEL")
RUNTIME_PARTS = ("RT", "MEM", "DRIVER", "CONTROL")
FAMILY_OF_PART = {"SHA2": "sha2", "HMAC_HKDF": "hmac_hkdf", "CHACHA_POLY": "chacha_poly", "X25519": "x25519",
                  "AES_GCM": "aes_gcm", "KECCAK": "keccak", "CT": "ct", "GCM_GLUE": "accel_glue",
                  "ACCEL": "accel_target"}

BUILD_FLAGS = {
    "common": ["-std=c11", "-ffreestanding", "-fno-builtin", "-fno-stack-protector",
               "-fno-asynchronous-unwind-tables", "-fno-unwind-tables", "-fPIE", "-Wall", "-Wextra", "-I."],
    "gcc_extra": ["-fno-tree-loop-distribute-patterns"],
    "link": {"gcc": ["-nostdlib", "-static", "-no-pie"], "clang": ["-nostdlib", "-static", "-fuse-ld=lld"]},
    "targets": {
        "T-X64": {"gcc": [], "clang": ["--target=x86_64-linux-gnu"],
                  "P-BASE": ["-march=x86-64"], "P-CRYPTO": ["-march=x86-64", "-maes", "-mpclmul"], "link_extra": []},
        "T-A64": {"gcc": [], "clang": ["--target=aarch64-linux-gnu"],
                  "P-BASE": ["-march=armv8-a"], "P-CRYPTO": ["-march=armv8-a+aes"], "link_extra": []},
        "T-RV64": {"gcc": ["-mno-relax"], "clang": ["--target=riscv64-linux-gnu", "-mno-relax"],
                   "P-BASE": ["-march=rv64gc", "-mabi=lp64d"], "P-CRYPTO": ["-march=rv64gc_zbkc_zkne", "-mabi=lp64d"],
                   "link_extra": ["-Wl,--no-relax"]},
    },
}

# ---------------------------------------------------------------------------
# Operations of the laboratory driver (runtime.c, D011_PART_DRIVER)

OPS = {
    1: ("sha256", "sha2", 1), 2: ("sha256_probe", "sha2", 1), 3: ("sha512", "sha2", 1),
    4: ("hmac_sha256", "hmac_hkdf", 2), 5: ("hkdf_sha256", "hmac_hkdf", 4),
    6: ("chacha20_block", "chacha_poly", 3), 7: ("chacha20_xor", "chacha_poly", 4),
    8: ("poly1305", "chacha_poly", 2), 9: ("aead_seal", "chacha_poly", 4), 10: ("aead_open", "chacha_poly", 4),
    11: ("x25519", "x25519", 2), 12: ("aes_block", "aes_gcm", 2), 13: ("aes_gcm_seal", "aes_gcm", 4),
    14: ("aes_gcm_open", "aes_gcm", 4), 15: ("sha3_256", "keccak", 1), 16: ("shake128", "keccak", 2),
    17: ("accel_aes_block", "accel", 2), 18: ("accel_aes_gcm_seal", "accel", 4),
    19: ("accel_aes_gcm_open", "accel", 4), 20: ("control_first_difference", "control", 2),
    21: ("control_divide", "control", 2),
}
ACCEL_OF = {12: 17, 13: 18, 14: 19}
STATUS = {0: "ok", 1: "rejected", 2: "unsupported", 3: "malformed"}
STATUS_CODE = {v: k for k, v in STATUS.items()}
PORTABLE_OPS = tuple(range(1, 17))

# ---------------------------------------------------------------------------
# Instruction classes for the static inventory (case NT-02)

INSTRUCTION_CLASSES = {
    "A": {"name": "division and square root", "gate": "HG-04",
          "why": "latency depends on operand values on common implementations"},
    "B": {"name": "conditional branches", "gate": None,
          "why": "informational; whether a branch depends on a secret is what NT-03 measures"},
    "C": {"name": "indirect jumps and calls", "gate": None, "why": "informational"},
    "D": {"name": "multiplications", "gate": None,
          "why": "data-independent latency is promised only under DOITM, FEAT_DIT or Zkt"},
    "E": {"name": "crypto-extension instructions", "gate": None, "why": "informational"},
}
_X86_A = re.compile(r"^(div|idiv|v?divs[sd]|v?divp[sd]|v?sqrt[sp][sd]|fdivr?p?|fidivr?|fsqrt)$")
_X86_D = re.compile(r"^(mul|imul|mulx|v?pmul\w*|v?mul[sp][sd]|fmul\w*|fimul)$")
_X86_E = re.compile(r"^(v?aes\w*|v?pclmul\w*|sha1\w+|sha256\w+|gf2p8\w+)$")
_A64_A = {"udiv", "sdiv", "fdiv", "fsqrt"}
_A64_B = re.compile(r"^(b\.\w+|cbn?z|tbn?z)$")
_A64_C = {"br", "blr", "braa", "brab", "blraa", "blrab", "braaz", "brabz", "blraaz", "blrabz"}
_A64_D = {"mul", "madd", "msub", "mneg", "smull", "umull", "smulh", "umulh", "smaddl", "umaddl", "smsubl",
          "umsubl", "smnegl", "umnegl", "fmul"}
_A64_E = re.compile(r"^(aes\w+|pmull2?|sha1\w+|sha256\w+|sha512\w+|sm3\w+|sm4\w+|eor3|rax1|xar|bcax)$")
_RV_A = re.compile(r"^(divu?w?|remu?w?|fdiv\.[sdhq]|fsqrt\.[sdhq])$")
_RV_B = re.compile(r"^(c\.)?(beqz?|bnez?|blt[uz]?|bge[uz]?|bgtu?|bleu?|bgtz|blez)$")
_RV_D = re.compile(r"^(mulh?s?u?w?|mulhsu|fmul\.[sdhq])$")
_RV_E = re.compile(r"^(aes64\w+|aes32\w+|clmulh?|clmulr|sha256\w+|sha512\w+|sm3\w+|sm4\w+|xperm\w+)$")


def classify(arch: str, mnemonic: str, operands: str) -> str | None:
    """The instruction class of one disassembled instruction, or None."""

    m = mnemonic.strip().lower()
    ops = operands.strip().lower()
    for prefix in ("notrack ", "lock ", "rep ", "bnd "):
        if m.startswith(prefix):
            m = m[len(prefix):]
    if arch == "x86_64":
        if _X86_A.match(m):
            return "A"
        if (m.startswith("j") and m not in ("jmp", "jmpq")) or m in ("loop", "loope", "loopne"):
            return "B"
        if m in ("jmp", "jmpq", "call", "callq") and not re.match(r"^0x[0-9a-f]+(\s|$)", ops):
            return "C"
        if _X86_D.match(m):
            return "D"
        if _X86_E.match(m):
            return "E"
        return None
    if arch == "aarch64":
        if m in _A64_A:
            return "A"
        if _A64_B.match(m):
            return "B"
        if m in _A64_C:
            return "C"
        if m in _A64_D:
            return "D"
        if _A64_E.match(m):
            return "E"
        return None
    if arch == "riscv64":
        if _RV_A.match(m):
            return "A"
        if _RV_B.match(m):
            return "B"
        if m in ("jr", "jalr", "c.jr", "c.jalr"):
            return "C"
        if _RV_D.match(m):
            return "D"
        if _RV_E.match(m):
            return "E"
        return None
    raise ValueError(f"unknown architecture {arch}")


def classify_sequence(arch: str, instructions: list[tuple[str, str]]) -> list[str | None]:
    """Classes for one function's instructions. On RV64 an `auipc` followed by `jalr` or `jr` is the direct call or
    tail call the assembler expands `call` and `tail` into, not an indirect transfer."""

    classes: list[str | None] = []
    previous = ""
    for mnemonic, operands in instructions:
        cls = classify(arch, mnemonic, operands)
        if arch == "riscv64" and cls == "C" and previous == "auipc":
            cls = None
        classes.append(cls)
        previous = mnemonic.strip().lower()
    return classes


def parse_objdump(text: str) -> dict[str, list[tuple[str, str]]]:
    """Map each symbol of `llvm-objdump -d --no-show-raw-insn --no-leading-addr` output to its instructions."""

    functions: dict[str, list[tuple[str, str]]] = {}
    current: list[tuple[str, str]] | None = None
    for line in text.splitlines():
        header = re.match(r"^(?:[0-9a-f]+ )?<([^>]+)>:$", line)
        if header:
            current = functions.setdefault(header.group(1), [])
            continue
        if current is None or not line.startswith((" ", "\t")):
            continue
        body = line.strip()
        if not body or body == "...":
            continue
        fields = body.split("\t", 1)
        current.append((fields[0].strip(), fields[1].strip() if len(fields) > 1 else ""))
    return functions


# ---------------------------------------------------------------------------
# Cases, metrics, gates, axes, rules and owner review scopes

CASES = (
    {"id": "NT-01", "name": "Known-answer agreement",
     "acceptance_evidence": "flagship-corpus feasibility",
     "measures": "Every known-answer subject on every build, emulated for every tuple and native on the "
                 "laboratory host's own tuple, returns the expected status and bytes. Precondition: the "
                 "subject's oracles agree (the published value carried in the Orange source, the Orange "
                 "reference evaluation, and a library or Python reference).",
     "metrics": ["M-01", "M-02", "M-03"], "gates": ["HG-02"]},
    {"id": "NT-02", "name": "Static instruction inventory",
     "acceptance_evidence": "flagship-corpus feasibility",
     "measures": "llvm-objdump disassembles every kernel and crypto-profile object; each instruction is "
                 "classified (A division and square root, B conditional branches, C indirect transfers, D "
                 "multiplications, E crypto-extension). Class A must be absent from kernel code; the division "
                 "control must show class A, so the detector is proven on every build.",
     "metrics": ["M-05", "M-06", "M-07", "M-08", "M-09", "M-11"], "gates": ["HG-04"]},
    {"id": "NT-03", "name": "Control-flow trace equivalence",
     "acceptance_evidence": "flagship-corpus feasibility",
     "measures": "For each trace group, QEMU's translation-block trace (guest program counters, `-d "
                 "nochain,exec`) is hashed for several secret variants of one public shape; the hashes must be "
                 "equal. The first-difference control must produce different traces. This is emulated control "
                 "flow only: it sees no cache, memory-address or timing channel.",
     "metrics": ["M-10", "M-11"], "gates": ["HG-05"]},
    {"id": "NT-04", "name": "Negative and fail-closed behaviour",
     "acceptance_evidence": "flagship-corpus feasibility",
     "measures": "Tampered AEAD and AES-GCM inputs are rejected with no plaintext; malformed requests are "
                 "refused; truncated input ends the process with status 2; accelerated operations answer "
                 "unsupported in the baseline profile; a crypto-profile binary run on a CPU model without the "
                 "extension stops with SIGILL rather than a wrong answer; and a binary with one flipped bit in "
                 "the SHA-256 constants is caught by the known-answer check.",
     "metrics": ["M-04"], "gates": ["HG-03"]},
    {"id": "NT-05", "name": "C ABI agreement",
     "acceptance_evidence": "ISA and ABI model availability",
     "measures": "Kernel objects from one C toolchain linked with the runtime from the other agree with the "
                 "known answers; the Rust ABI probe calls the kernels through extern \"C\" and agrees; kernel "
                 "objects import nothing beyond memcpy, memmove, memset, memcmp and d011_ symbols; relocation "
                 "types are recorded.",
     "metrics": ["M-12", "M-13", "M-14"], "gates": ["HG-06"]},
    {"id": "NT-06", "name": "ISA and ABI model inventory",
     "acceptance_evidence": "ISA and ABI model availability",
     "measures": "The packet lists, per tuple, the ISA reference, the psABI, the formal or machine-readable "
                 "model and the data-independent-timing mechanism, as the contributor recorded them from "
                 "knowledge and without checking from the laboratory host. A row counts only when the owner "
                 "verifies it in the owner input.",
     "metrics": ["M-15"], "gates": ["HG-07"]},
    {"id": "NT-07", "name": "Owner-accessible hardware",
     "acceptance_evidence": "owner-accessible hardware evidence",
     "measures": "The laboratory records its own host (a contributor machine, never owner hardware). The "
                 "owner input names, per tuple, the devices the owner can run natively, and records the SHA-256 of "
                 "the output of an archived driver run natively on one of them, named, with the archived request "
                 "file; the laboratory compares it with the emulated output. Absent owner input leaves the gate "
                 "unresolved.",
     "metrics": ["M-16"], "gates": ["HG-08"]},
    {"id": "NT-08", "name": "Resource estimate",
     "acceptance_evidence": "resource estimate per target",
     "measures": "Per tuple: toolchains needed and missing, build CPU and wall time, deterministic rebuild, "
                 "kernel code bytes per family, target-specific source lines, distinct mnemonics, relocation "
                 "types, known-answer wall time (native only where the host is the tuple; emulated otherwise, "
                 "reported separately and never compared), trace sizes, a CI-time projection, and the number "
                 "of bounded target slices under E-SOLO.",
     "metrics": ["M-17", "M-18", "M-19", "M-20", "M-21", "M-22"], "gates": ["HG-09", "HG-01"]},
)
CASE_IDS = tuple(c["id"] for c in CASES)

METRICS = (
    ("M-01", "Known-answer agreement", "subjects matching / subjects run, per build and execution mode"),
    ("M-02", "Oracle agreement", "subjects whose available oracles all agree / subjects"),
    ("M-03", "Native and emulated agreement", "host-tuple builds whose native and emulated responses are identical"),
    ("M-04", "Negative checks", "negative checks passing / negative checks applicable, per build"),
    ("M-05", "Class A in kernel code", "division and square-root instructions in kernel objects"),
    ("M-06", "Class B in kernel code", "conditional branches in kernel objects (informational)"),
    ("M-07", "Class C in kernel code", "indirect jumps and calls in kernel objects (informational)"),
    ("M-08", "Class D in kernel code", "multiplications in kernel objects (informational; DIT assumption)"),
    ("M-09", "Class E", "crypto-extension instructions in crypto-profile objects (informational)"),
    ("M-10", "Trace equivalence", "trace groups with one distinct trace / trace groups run"),
    ("M-11", "Detector controls", "division control found by the inventory; first-difference control traces differ"),
    ("M-12", "Cross-toolchain agreement", "cross-linked builds agreeing with the known answers / cross-linked builds"),
    ("M-13", "Rust probe agreement", "probe subjects matching / probe subjects run"),
    ("M-14", "Symbol surface", "undefined symbols in kernel objects outside the allowed set"),
    ("M-15", "Inventory verification", "required inventory rows the owner verified / required rows"),
    ("M-16", "Owner-attested devices", "devices per tuple in the owner input"),
    ("M-17", "Build cost", "compile and link CPU and wall milliseconds per build"),
    ("M-18", "Kernel code bytes", "executable bytes per kernel family and per crypto-profile object"),
    ("M-19", "Known-answer wall time", "native and emulated medians, reported separately and never compared"),
    ("M-20", "Target surface", "target-specific source lines, distinct mnemonics, relocation types"),
    ("M-21", "Independent review", "unavailable under E-SOLO; recorded, never scored"),
    ("M-22", "Solo slices", "bounded target slices the candidate needs (E-SOLO S-05: one at a time)"),
)

GATES = (
    ("HG-01", "Matrix completeness", "NT-08",
     "Every planned build of the tuple compiled, linked and ran every case. A build failure fails; a toolchain "
     "that is absent (an acquisition) leaves it unresolved."),
    ("HG-02", "Known answers", "NT-01",
     "Every subject matches on every build and execution mode. A mismatch with agreeing oracles fails; an oracle "
     "conflict leaves the subject unresolved."),
    ("HG-03", "Fail-closed", "NT-04",
     "Every applicable negative check passes. A check the laboratory cannot perform on the tuple (no emulated CPU "
     "model without the extension) is unresolved until the owner attests, for that build's archived driver, the "
     "SIGILL on a named device without the extension."),
    ("HG-04", "No division in kernel code", "NT-02",
     "No class A instruction in any kernel or crypto-profile object, and the division control is detected on "
     "every build (otherwise unresolved)."),
    ("HG-05", "Trace equivalence", "NT-03",
     "Every kernel trace group has one distinct trace on every build, and the first-difference control differs "
     "(otherwise unresolved)."),
    ("HG-06", "C ABI agreement", "NT-05",
     "Cross-toolchain links and the Rust probe agree and the symbol surface holds. A missing second toolchain or "
     "missing Rust standard library leaves it unresolved."),
    ("HG-07", "ISA and ABI inventory verified", "NT-06",
     "The owner verified every required inventory row of the tuple; a rejected row fails; otherwise unresolved."),
    ("HG-08", "Owner hardware", "NT-07",
     "The owner attests a device for the tuple and records a native run of an archived driver on that device whose "
     "output digest equals the emulated one. A declared absence or a disagreeing run fails; otherwise unresolved."),
    ("HG-09", "Resource estimate", "NT-08",
     "Every resource field is present and every build rebuilt bit for bit; a nondeterministic build fails."),
)
GATE_IDS = tuple(g[0] for g in GATES)
GATE_STATES = ("pass", "fail", "unresolved", "unsupported")
PRECEDENCE = {"unsupported": 3, "fail": 2, "unresolved": 1, "pass": 0}
# The portable path makes no machine-code claim, so these gates are vacuous for it.
PORTABLE_VACUOUS = ("HG-04", "HG-05", "HG-07", "HG-08")

AXES = (
    ("AX-01", "Claim coverage", "claim-bearing native tuples", "higher", "any difference"),
    ("AX-02", "Evidence closure", "non-vacuous gate evaluations passing", "higher", "any difference"),
    ("AX-03", "Leakage-risk surface", "class D instructions in kernel code needing a DIT assumption", "lower",
     "a difference of at least 10 percent"),
    ("AX-04", "Build and test cost", "projected CI milliseconds per run on the laboratory host", "lower",
     "a difference of at least 20 percent"),
    ("AX-05", "ISA and ABI surface", "distinct kernel mnemonics plus target-specific source lines", "lower",
     "a difference of at least 10 percent"),
    ("AX-06", "Toolchain dependencies", "distinct toolchains needed plus missing acquisitions", "lower",
     "any difference"),
    ("AX-07", "Solo capacity", "bounded target slices needed under E-SOLO S-05", "lower", "any difference"),
)
AXIS_IDS = tuple(a[0] for a in AXES)

RULES = (
    ("DR-1", "Coverage first within capacity",
     "Among eligible candidates whose AX-07 slices fit the owner's declared solo_slice_capacity, prefer the "
     "highest AX-01; equal coverage is a tie."),
    ("DR-2", "Solo first", "Prefer the fewest AX-07 slices, then the highest AX-01; otherwise a tie."),
    ("DR-3", "Cost first", "Prefer the lowest AX-04, then the lowest AX-06, by the materiality bands; otherwise a tie."),
    ("DR-4", "Dominance only",
     "Recommend a candidate only if it is no worse on every axis and materially better on at least one than "
     "every other eligible candidate; otherwise inconclusive."),
)

REVIEW_SCOPES = (
    ("NR-01", "Custody and candidate parity", "the packet, the candidate set and its symmetry"),
    ("NR-02", "Flagship corpus and oracles", "subjects, oracle sources, named gaps"),
    ("NR-03", "x86-64 evidence", "T-X64 builds, inventory, traces, negatives"),
    ("NR-04", "AArch64 evidence", "T-A64 builds, inventory, traces, negatives"),
    ("NR-05", "RV64GC evidence", "T-RV64 builds, inventory, traces, negatives"),
    ("NR-06", "Portable C path", "C-PORTABLE builds and the Rust probe"),
    ("NR-07", "ISA and ABI inventory", "every NT-06 row, verified or rejected"),
    ("NR-08", "Hardware attestation", "the owner's own devices per tuple"),
    ("NR-09", "Resource and solo capacity", "NT-08 estimates against E-SOLO"),
    ("NR-10", "Comparative disposition", "the distinguishing rule and the final choice"),
)

RUN_PROFILES = {
    "dev": {"optimizations": ["O2"], "build_repetitions": 2, "native_timed_runs": 1, "native_warmups": 0,
            "emulated_timed_runs": 1, "timing_batch_repeats": 2, "trace_variants": 3, "concurrency": "serial",
            "purpose": "exercise every stage once; not a result"},
    "measured": {"optimizations": ["O2", "O3", "Os"], "build_repetitions": 3, "native_timed_runs": 30,
                 "native_warmups": 1, "emulated_timed_runs": 5, "timing_batch_repeats": 20, "trace_variants": 8,
                 "concurrency": "serial",
                 "purpose": "the evidence run the owner reviews"},
}

DECISION_PROCEDURE = (
    "Evaluate every gate per tuple and for the portable path, then per candidate by the precedence unsupported > "
    "fail > unresolved > pass over the portable path and the candidate's tuples.",
    "A dev-profile epoch concludes inconclusive: it exercises the laboratory and is not a result.",
    "Without owner input covering every review scope, the epoch concludes inconclusive.",
    "A candidate is eligible when every gate passes. None eligible: inconclusive. One eligible: recommend it.",
    "Several eligible: the candidates are nested (TE-02 and TE-03 inside TE-01 inside TE-05, TE-04 the empty "
    "envelope), so gates cannot separate them. Apply the owner's distinguishing rule (DR-1 to DR-4); with none "
    "recorded the epoch concludes inconclusive; a rule that leaves several concludes tie.",
    "No weighted score is computed. Axes are reported for the owner, never summed.",
    "The conclusion is advice to the owner. The choice, and any freeze, is the owner's alone and is not "
    "recorded by this tool.",
)

NONCLAIMS = (
    "The kernels are contributor-written C standing in for code Orange does not yet generate; nothing here "
    "measures Orange code generation.",
    "No build is claimed constant-time. Trace equivalence is emulated control flow; it sees no cache, "
    "memory-address, power or timing channel.",
    "Emulated timings are never compared with native timings; emulated time is a CI-cost proxy, not a "
    "performance figure.",
    "ISA and ABI inventory rows are recorded from the contributor's knowledge, not checked from the "
    "laboratory host, and count only when the owner verifies them.",
    "The laboratory host is a contributor machine; it is never evidence of owner-accessible hardware.",
    "Every record is contributor-produced and unreviewed; no owner decision, review or freeze is recorded.",
)

# ---------------------------------------------------------------------------
# Oracle sources and the known-answer subjects (the flagship corpus slice)

ORACLE_SOURCES = (
    ("OS-SHA2", "algorithms/sha2/sha2.or", "e57e32d80a06d30fdb381371e11226501c7eea01c4c4a7ff081dc1e43cd189db"),
    ("OS-SHA3", "algorithms/sha3/sha3.or", "687947c96d4090e478caf062b35dac4b65d9a851af7072559f966469deaef14e"),
    ("OS-HMAC", "algorithms/hmac-hkdf/hmac-hkdf.or", "2cff7fbc89366fe90df5acc5e024a0b0342e8f733a5c7e9cdbaa33aa126701eb"),
    ("OS-HKDF", "algorithms/hmac-hkdf/hmac-hkdf-rfc5869.or",
     "6c56017c937602a018e03d569749504fd6afcc19420a0933cfff56b66a0c2b84"),
    ("OS-CHACHA", "algorithms/chacha20/chacha20.or", "28673361293e18f3350ea42f86d701ef07bbe0819c8ce41850e12e9339258e0a"),
    ("OS-AEAD", "algorithms/chacha20-poly1305/chacha20-poly1305.or",
     "56cfb47a2794af840a513611aa20e8778083dcf26c85f801f5ff44953f03d4a2"),
    ("OS-AES", "algorithms/aes/aes.or", "4a4ba27158e2e61a92510891a82d7227b5c1ad87667ecc69cd0daf53c1546603"),
    ("OS-GCM", "algorithms/aes-gcm/aes-gcm.or", "68b9414e794443d5e71833fb8595d86d035fff52a8540bf0b7ea6cab321a9bd5"),
    ("OS-GCM256", "algorithms/aes-gcm/aes-gcm-256.or", "35e585e5f5d13d4c33d77843c6fd74aa45f6461a89de35558e164f3d21d43be4"),
    ("OS-GCMAD", "algorithms/aes-gcm/aes-gcm-ad.or", "a55ef74a131f6fa1490cae68fc9b545b03ae9130b33e04eaafc9dec5d6decd60"),
    ("OS-X25519", "algorithms/x25519/x25519.or", "347376bcb4d491d1ac562df78485610998e783a6d1de16607564901cfdf2b991"),
    ("OS-X25519-2", "algorithms/x25519/x25519-second-vector.or",
     "e165cc20cce58d14810780a1dea0f906da12df92c8f1b91104db8000dc99db6b"),
    ("OS-X25519-DH", "algorithms/x25519/x25519-diffie-hellman.or",
     "3fb6cb45867d6334cd35d894616048f645b270ada925fe53221aa43392cb4de0"),
    ("OS-X25519-WP", "algorithms/x25519/x25519-wycheproof.or",
     "d3b1140269dbf4d9292fc4aaa412c4e43835dacca4416e35a6ca3f0c9b13d3f9"),
    ("OS-S3D", "compiler/fixtures/s3d/valid-sha256-rounds.or",
     "107a7a3ed248cdd5120be49270e126c2409f04f12a27287ec328534bb0972446"),
)
PUBLISHED_CARRIERS = (
    ("PC-S3D", "compiler/crates/orangec/tests/s3d_conformance.rs",
     "61b2030796d00211cb26bba51fa42054ba48ce91df6a06e498d16859d9f67791",
     "carries the FIPS 180-4 worked-example values for K-SHA256-05"),
)
ORACLE_PATH = {sid: path for sid, path, _ in ORACLE_SOURCES}


def _t(text: str) -> str:
    return text.encode("ascii").hex()


def _r(byte: int, count: int) -> str:
    return (bytes([byte]) * count).hex()


def _seq(start: int, count: int) -> str:
    return bytes((start + i) & 0xFF for i in range(count)).hex()


def _le32(value: int) -> str:
    return value.to_bytes(4, "little").hex()


SUNSCREEN = _t("Ladies and Gentlemen of the class of '99: If I could offer you only one tip for the future, "
               "sunscreen would be it.")
IETF_TEXT = _t(
    "Any submission to the IETF intended by the Contributor for publication as all or part of an IETF "
    "Internet-Draft or RFC and any statement made within the context of an IETF activity is considered an "
    "\"IETF Contribution\". Such statements include oral statements in IETF sessions, as well as written and "
    "electronic communications made at any time or place, which are addressed to")
JABBERWOCKY = _t("'Twas brillig, and the slithy toves\nDid gyre and gimble in the wabe:\nAll mimsy were the "
                 "borogoves,\nAnd the mome raths outgrabe.")
A5_KEY = "1c9240a5eb55d38af333888604f6b5f0473917c1402b80099dca5cbc207075c0"
AEAD_KEY = _seq(0x80, 32)
GCM_K = "feffe9928665731c6d6a8f9467308308"
GCM_IV = "cafebabefacedbaddecaf888"
GCM_A = "feedfacedeadbeeffeedfacedeadbeefabaddad2"
GCM_P = ("d9313225f88406e5a55909c5aff5269a86a7a9531534f7da2e4c303d8a318a721c3c0c95956809532fcf0e2449a6b525b16aed"
         "f5aa0de657ba637b39")

# SUBJECTS_BEGIN
A5_CT = ("64a0861575861af460f062c79be643bd5e805cfd345cf389f108670ac76c8cb24c6cfc18755d43eea09ee94e382d26b0"
         "bdb7b73c321b0100d4f03b7f355894cf332f830e710b97ce98c8a84abd0b948114ad176e008d33bd60f982b1ff37c8559797a0"
         "6ef4f0ef61c186324e2b3506383606907b6a7c02b0f9f6157b53c867e4b9166c767b804d46a59b5216cde7a4e99040c5a4"
         "0433225ee282a1b0a06c523eaf4534d7f83fa1155b0047718cbc546a0d072b04b3564eea1b422273f548271a0bb2316053"
         "fa76991955ebd63159434ecebb4e466dae5a1073a6727627097a1049e617d91d361094fa68f0ff77987130305beaba2eda"
         "04df997b714d6c6f2c29a6ad5cb4022b02709beead9d67890cbb22392336fea1851f38")
BOTAN_64 = ("3b47876f88f7c4e7ce09f8bb35240915391cd5d335f12b40b3da5e30eb504a196b484f864929c89793f93691ad4c062e"
            "e4861e811ef2c0c047266752ba43524d")
# Expected driver outputs: the published literal of each subject, sliced to the driver's layout.
EXPECTED: dict[str, str] = {
    'K-SHA256-01': 'ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad',
    'K-SHA256-02': '248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1',
    'K-SHA256-03': 'e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855',
    'K-SHA256-04': '14609c054e038a8cb4f0886de99b31307f2e707a072c674abfa646161b6bff63',
    'K-SHA256-05': (
        '61626380000f00005d6aebcd6a09e667bb67ae853c6ef372fa2a4622510e527f9b05688c1f83d9ab5a6ad9ad5d6aebcd6a09'
        'e667bb67ae8578ce7989fa2a4622510e527f9b05688c'
    ),
    'K-SHA512-01': (
        'ddaf35a193617abacc417349ae20413112e6fa4e89a97ea20a9eeee64b55d39a2192992a274fc1a836ba3c23a3feebbd454d'
        '4423643ce80e2a9ac94fa54ca49f'
    ),
    'K-SHA512-02': (
        '8e959b75dae313da8cf4f72814fc143f8f7779c6eb9f7fa17299aeadb6889018501d289e4900f7e4331b99dec4b5433ac7d3'
        '29eeb6dd26545e96e55b874be909'
    ),
    'K-SHA512-03': (
        'cf83e1357eefb8bdf1542850d66d8007d620e4050b5715dc83f4a921d36ce9ce47d0d13c5d85f2b0ff8318d2877eec2f63b9'
        '31bd47417a81a538327af927da3e'
    ),
    'K-HMAC-01': 'b0344c61d8db38535ca8afceaf0bf12b881dc200c9833da726e9376c2e32cff7',
    'K-HMAC-02': '5bdcc146bf60754e6a042426089575c75a003f089d2739839dec58b964ec3843',
    'K-HMAC-03': '773ea91e36800e46854db8ebd09181a72959098b3ef8c122d9635514ced565fe',
    'K-HMAC-04': '82558a389a443c0ea4cc819899f2083a85f0faa3e578f8077a2e3ff46729665b',
    'K-HMAC-05': '60e431591ee0b67f0d8a26aacbf5b77f8e0bc6213728c5140546040f0ee37f54',
    'K-HMAC-06': '9b09ffa71b942fcb27635fbcd5b0e944bfdc63644f0713938a7f51535c3a35e2',
    'K-HMAC-07': 'e542ac8ac8f364bae4b7da8b7a0777df350f001de4e8cfa2d9ef0b15019496ec',
    'K-HKDF-01': (
        '3cb25f25faacd57a90434f64d0362f2a2d2d0a90cf1a5a4c5db02d56ecc4c5bf34007208d5b887185865'
    ),
    'K-HKDF-02': (
        'b11e398dc80327a1c8e7f78c596a49344f012eda2d4efad8a050cc4c19afa97c59045a99cac7827271cb41c65e590e09da32'
        '75600c2f09b8367793a9aca3db71cc30c58179ec3e87c14c01d5c1f3434f1d87'
    ),
    'K-HKDF-03': (
        '8da4e775a563c18f715f802a063c5a31b8a11f5c5ee1879ec3454e5f3c738d2d9d201395faa4b61a96c8'
    ),
    'K-CHACHA-01': (
        '10f1e7e4d13b5915500fdd1fa32071c4c7d1f4c733c068030422aa9ac3d46c4ed2826446079faa0914c2d705d98b02a2b512'
        '9cd1de164eb9cbd083e8a2503c4e'
    ),
    'K-CHACHA-02': (
        '76b8e0ada0f13d90405d6ae55386bd28bdd219b8a08ded1aa836efcc8b770dc7da41597c5157488d7724e03fb8d84a376a43'
        'b8f41518a11cc387b669b2ee6586'
    ),
    'K-CHACHA-03': (
        '9f07e7be5551387a98ba977c732d080dcb0f29a048e3656912c6533e32ee7aed29b721769ce64e43d57133b074d839d531ed'
        '1f28510afb45ace10a1f4b794d6f'
    ),
    'K-CHACHA-04': (
        '3aeb5224ecf849929b9d828db1ced4dd832025e8018b8160b82284f3c949aa5a8eca00bbb4a73bdad192b5c42f73f2fd4e27'
        '3644c8b36125a64addeb006c13a0'
    ),
    'K-CHACHA-05': (
        '72d54dfbf12ec44b362692df94137f328fea8da73990265ec1bbbea1ae9af0ca13b25aa26cb4a648cb9b9d1be65b2c0924a6'
        '6c54d545ec1b7374f4872e99f096'
    ),
    'K-CHACHA-06': (
        'c2c64d378cd536374ae204b9ef933fcd1a8b2288b3dfa49672ab765b54ee27c78a970e0e955c14f3a88e741b97c286f75f8f'
        'c299e8148362fa198a39531bed6d'
    ),
    'K-CHACHA-07': (
        '6e2e359a2568f98041ba0728dd0d6981e97e7aec1d4360c20a27afccfd9fae0bf91b65c5524733ab8f593dabcd62b3571639'
        'd624e65152ab8f530c359f0861d807ca0dbf500d6a6156a38e088a22b65e52bc514d16ccf806818ce91ab77937365af90bbf'
        '74a35be6b40b8eedf2785e42874d'
    ),
    'K-CHACHA-08': (
        '76b8e0ada0f13d90405d6ae55386bd28bdd219b8a08ded1aa836efcc8b770dc7da41597c5157488d7724e03fb8d84a376a43'
        'b8f41518a11cc387b669b2ee6586'
    ),
    'K-CHACHA-09': (
        'a3fbf07df3fa2fde4f376ca23e82737041605d9f4f4f57bd8cff2c1d4b7955ec2a97948bd3722915c8f3d337f7d370050e9e'
        '96d647b7c39f56e031ca5eb6250d4042e02785ececfa4b4bb5e8ead0440e20b6e8db09d881a7c6132f420e52795042bdfa77'
        '73d8a9051447b3291ce1411c680465552aa6c405b7764d5e87bea85ad00f8449ed8f72d0d662ab052691ca66424bc86d2df8'
        '0ea41f43abf937d3259dc4b2d0dfb48a6c9139ddd7f76966e928e635553ba76c5c879d7b35d49eb2e62b0871cdac638939e2'
        '5e8a1e0ef9d5280fa8ca328b351c3c765989cbcf3daa8b6ccc3aaf9f3979c92b3720fc88dc95ed84a1be059c6499b9fda236'
        'e7e818b04b0bc39c1e876b193bfe5569753f88128cc08aaa9b63d1a16f80ef2554d7189c411f5869ca52c5b83fa36ff216b9'
        'c1d30062bebcfd2dc5bce0911934fda79a86f6e698ced759c3ff9b6477338f3da4f9cd8514ea9982ccafb341b2384dd902f3'
        'd1ab7ac61dd29c6f21ba5b862f3730e37cfdc4fd806c22f221'
    ),
    'K-CHACHA-10': (
        '62e6347f95ed87a45ffae7426f27a1df5fb69110044c0d73118effa95b01e5cf166d3df2d721caf9b21e5fb14c616871fd84'
        'c54f9d65b283196c7fe4f60553ebf39c6402c42234e32a356b3e764312a61a5532055716ead6962568f87d3f3f7704c6a8d1'
        'bcd1bf4d50d6154b6da731b187b58dfd728afa36757a797ac188d1'
    ),
    'K-POLY-01': 'a8061dc1305136c6c22b8baf0c0127a9',
    'K-POLY-02': '00000000000000000000000000000000',
    'K-POLY-03': '36e5f6b5c5e06070f0efca96227a863e',
    'K-POLY-04': 'f3477e7cd95417af89a6b8794c310cf0',
    'K-POLY-05': '4541669a7eaaee61e708dc7cbcc5eb62',
    'K-POLY-06': '03000000000000000000000000000000',
    'K-POLY-07': '03000000000000000000000000000000',
    'K-POLY-08': '05000000000000000000000000000000',
    'K-POLY-09': '00000000000000000000000000000000',
    'K-POLY-10': 'faffffffffffffffffffffffffffffff',
    'K-POLY-11': '14000000000000005500000000000000',
    'K-POLY-12': '13000000000000000000000000000000',
    'K-POLY-13': '8ad5a08b905f81cc815040274ab29471a833b637e3fd0da508dbb8e2fdd1a646',
    'K-POLY-14': '965e3bc6f9ec7ed9560808f4d229f94b137ff275ca9b3fcbdd59deaad23310ae',
    'K-AEAD-01': (
        'd31a8d34648e60db7b86afbc53ef7ec2a4aded51296e08fea9e2b5a736ee62d63dbea45e8ca9671282fafb69da92728b1a71'
        'de0a9e060b2905d6a5b67ecd3b3692ddbd7f2d778b8c9803aee328091b58fab324e4fad675945585808b4831d7bc3ff4def0'
        '8e4b7a9de576d26586cec64b61161ae10b594f09e26a7e902ecbd0600691'
    ),
    'K-AEAD-02': (
        '4c616469657320616e642047656e746c656d656e206f662074686520636c617373206f66202739393a204966204920636f75'
        '6c64206f6666657220796f75206f6e6c79206f6e652074697020666f7220746865206675747572652c2073756e7363726565'
        '6e20776f756c642062652069742e'
    ),
    'K-AEAD-03': (
        '496e7465726e65742d4472616674732061726520647261667420646f63756d656e74732076616c696420666f722061206d61'
        '78696d756d206f6620736978206d6f6e74687320616e64206d617920626520757064617465642c207265706c616365642c20'
        '6f72206f62736f6c65746564206279206f7468657220646f63756d656e747320617420616e792074696d652e204974206973'
        '20696e617070726f70726961746520746f2075736520496e7465726e65742d447261667473206173207265666572656e6365'
        '206d6174657269616c206f7220746f2063697465207468656d206f74686572207468616e206173202fe2809c776f726b2069'
        '6e2070726f67726573732e2fe2809d'
    ),
    'K-AEAD-04': (
        'a1ffed80761829ecce242e0e88b138049016bca018da2b6e19986b3e318cae8d806198fb4c527cc39350ebddeac573c4cbf0'
        'befda0b70242c640d7cd02d7a3'
    ),
    'K-X25519-01': 'c3da55379de9c6908e94ea4df28d084f32eccf03491c71f754b4075577a28552',
    'K-X25519-02': '95cbde9476e8907d7aade45cb4b873f88b595a68799fa152e6f8f7647aac7957',
    'K-X25519-03': '4a5d9d5ba4ce2de1728e3bf480350f25e07e21c947d19e3376f09b3c1e161742',
    'K-X25519-04': '436a2c040cf45fea9b29a0cb81b1f41458f863d0d61b453d0a982720d6d61320',
    'K-AES-01': '69c4e0d86a7b0430d8cdb78070b4c55a',
    'K-AES-02': '8ea2b7ca516745bfeafc49904b496089',
    'K-AES-03': '3925841d02dc09fbdc118597196a0b32',
    'K-GCM-01': '58e2fccefa7e3061367f1d57a4e7455a',
    'K-GCM-02': '0388dace60b6a392f328c2b971b2fe78ab6e47d42cec13bdf53a67b21257bddf',
    'K-GCM-03': (
        '42831ec2217774244b7221b784d0d49ce3aa212f2c02a4e035c17e2329aca12e21d514b25466931c7d8f6a5aac84aa051ba3'
        '0b396a0aac973d58e0915bc94fbc3221a5db94fae95ae7121a47'
    ),
    'K-GCM-04': '530f8afbc74536b9a963b4f1c4cb738b',
    'K-GCM-05': (
        '522dc1f099567d07f47f37a32a84427d643a8cdcbfe5c0c97598a2bd2555d1aa8cb08e48590dbb3da7b08b1056828838c5f6'
        '1e6393ba7a0abcc9f66276fc6ece0f4e1768cddf8853bb2d551b'
    ),
    'K-GCM-06': (
        'd9313225f88406e5a55909c5aff5269a86a7a9531534f7da2e4c303d8a318a721c3c0c95956809532fcf0e2449a6b525b16a'
        'edf5aa0de657ba637b39'
    ),
    'K-SHA3-01': 'a7ffc6f8bf1ed76651c14756a061d662f580ff4de43b49fa82d80a4b80f8434a',
    'K-SHA3-02': '3a985da74fe225b2045c172d6bd390bd855f086e3e9d525b46bfe24511431532',
    'K-SHA3-03': '5f728f63bf5ee48c77f453c0490398fa645b8d4c4e56be9a41cfec344d6ca899',
    'K-SHAKE-01': '7f9c2ba4e88f827d616045507605853ed73b8093f6efbc88eb1a6eacfa66ef26',
    'K-SHAKE-02': (
        '5881092dd818bf5cf8a3ddb793fbcba74097d5c526a6d35f97b83351940f2cc844c50af32acd3f2cdd066568706f509bc1bd'
        'de58295dae3f891a9a0fca5783789a41f8611214ce612394df286a62d1a2252aa94db9c538956c717dc2bed4f232a0294c85'
        '7c730aa16067ac1062f1201fb0d377cfb9cde4c63599b27f3462bba4a0ed296c801f9ff7f57302bb3076ee145f97a32ae68e'
        '76ab66c48d51675bd49acc29082f5647584e'
    ),
}
_POLY11 = "e33594d7505e43b9" + _r(0, 8) + "3394d7505e4379cd01" + _r(0, 7) + _r(0, 16)
_A3_10_KEY = "01" + _r(0, 7) + "04" + _r(0, 23)
# (id, op, args, status, oracle source, Orange computation, published literal, Orange slices, driver slices,
#  Orange status spec, provenance kind, citation)
SUBJECTS: tuple[tuple[Any, ...], ...] = (
    ("K-SHA256-01", 1, [_t("abc")], "ok", "OS-SHA2", ["nist_sha256_abc"], ["nist_sha256_abc_expected"], None, None,
     None, "published_standard", "FIPS 180-4 SHA-256 example \"abc\" (NIST CSRC examples)"),
    ("K-SHA256-02", 1, [_t("abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq")], "ok", "OS-SHA2",
     ["nist_sha256_two_block"], ["nist_sha256_two_block_expected"], None, None, None, "published_standard",
     "FIPS 180-4 SHA-256 two-block example (NIST CSRC examples)"),
    ("K-SHA256-03", 1, [""], "ok", "OS-SHA2", ["botan_sha256_empty"], ["botan_sha256_empty_expected"], None, None,
     None, "published_test_suite", "Botan sha2_32.vec, SHA-256 of the empty string"),
    ("K-SHA256-04", 1, [BOTAN_64], "ok", "OS-SHA2", ["botan_sha256_64_bytes"], ["botan_sha256_64_bytes_expected"],
     None, None, None, "published_test_suite", "Botan sha2_32.vec, SHA-256 of a 64-byte message"),
    ("K-SHA256-05", 2, ["61626380" + _r(0, 56) + "00000018"], "ok", "OS-S3D", ["w16_w17", "after_round0",
     "after_round1"], [], None, None, None, "published_standard",
     "FIPS 180-4 SHA-256 \"abc\" worked example (NIST CSRC): W16, W17 and the working variables after rounds 0 "
     "and 1; the published values are carried in compiler/crates/orangec/tests/s3d_conformance.rs"),
    ("K-SHA512-01", 3, [_t("abc")], "ok", "OS-SHA2", ["nist_sha512_abc"], ["nist_sha512_abc_expected"], None, None,
     None, "published_standard", "FIPS 180-4 SHA-512 example \"abc\" (NIST CSRC examples)"),
    ("K-SHA512-02", 3, [_t("abcdefghbcdefghicdefghijdefghijkefghijklfghijklmghijklmnhijklmnoijklmnopjklmnopqklmnopqr"
                           "lmnopqrsmnopqrstnopqrstu")], "ok", "OS-SHA2", ["nist_sha512_two_block"],
     ["nist_sha512_two_block_expected"], None, None, None, "published_standard",
     "FIPS 180-4 SHA-512 two-block example (NIST CSRC examples)"),
    ("K-SHA512-03", 3, [""], "ok", "OS-SHA2", ["hashlib_sha512_empty"], ["hashlib_sha512_empty_expected"], None,
     None, None, "library_produced", "SHA-512 of the empty string as algorithms/sha2 recorded it from Python hashlib"),
    ("K-HMAC-01", 4, [_r(0x0B, 20), _t("Hi There")], "ok", "OS-HMAC", ["rfc4231_case_1"], ["rfc4231_case_1_expected"],
     None, None, None, "published_standard", "RFC 4231 section 4.2, test case 1"),
    ("K-HMAC-02", 4, [_t("Jefe"), _t("what do ya want for nothing?")], "ok", "OS-HMAC", ["rfc4231_case_2"],
     ["rfc4231_case_2_expected"], None, None, None, "published_standard", "RFC 4231 section 4.3, test case 2"),
    ("K-HMAC-03", 4, [_r(0xAA, 20), _r(0xDD, 50)], "ok", "OS-HMAC", ["rfc4231_case_3"], ["rfc4231_case_3_expected"],
     None, None, None, "published_standard", "RFC 4231 section 4.4, test case 3"),
    ("K-HMAC-04", 4, [_seq(1, 25), _r(0xCD, 50)], "ok", "OS-HMAC", ["rfc4231_case_4"], ["rfc4231_case_4_expected"],
     None, None, None, "published_standard", "RFC 4231 section 4.5, test case 4"),
    ("K-HMAC-05", 4, [_r(0xAA, 131), _t("Test Using Larger Than Block-Size Key - Hash Key First")], "ok", "OS-HMAC",
     ["rfc4231_case_6"], ["rfc4231_case_6_expected"], None, None, None, "published_standard",
     "RFC 4231 section 4.7, test case 6"),
    ("K-HMAC-06", 4, [_r(0xAA, 131), _t("This is a test using a larger than block-size key and a larger than "
                                         "block-size data. The key needs to be hashed before being used by the "
                                         "HMAC algorithm.")], "ok", "OS-HMAC", ["rfc4231_case_7"],
     ["rfc4231_case_7_expected"], None, None, None, "published_standard", "RFC 4231 section 4.8, test case 7"),
    ("K-HMAC-07", 4, ["21178e26bc28ffc27c06f762ba190a627075856d7ca6feab79ac63149b17126e34fd9e5590e0e90aac801df095"
                      "05d8af2dd0a2703b352c573ac9d2cb063927f2af",
                      "7d5f1d6b993452b1b53a4375760d10a20d46a0ab9ec3943fc4b07a2ce735e731"], "ok", "OS-HMAC",
     ["wycheproof_hmac_tc_171"], ["wycheproof_hmac_tc_171_expected"], None, None, None, "published_test_suite",
     "Project Wycheproof hmac_sha256_test.json, tcId 171 (a 65-byte key)"),
    ("K-HKDF-01", 5, [_r(0x0B, 22), _seq(0, 13), _seq(0xF0, 10), _le32(42)], "ok", "OS-HKDF", ["rfc5869_a_1_okm"],
     ["rfc5869_a_1_okm_expected"], None, None, None, "published_standard", "RFC 5869 appendix A.1"),
    ("K-HKDF-02", 5, [_seq(0, 80), _seq(0x60, 80), _seq(0xB0, 80), _le32(82)], "ok", "OS-HKDF", ["rfc5869_a_2_okm"],
     ["rfc5869_a_2_okm_expected"], None, None, None, "published_standard", "RFC 5869 appendix A.2"),
    ("K-HKDF-03", 5, [_r(0x0B, 22), "", "", _le32(42)], "ok", "OS-HKDF", ["rfc5869_a_3_okm"],
     ["rfc5869_a_3_okm_expected"], None, None, None, "published_standard", "RFC 5869 appendix A.3"),
    ("K-CHACHA-01", 6, [_seq(0, 32), _le32(1), "000000090000004a00000000"], "ok", "OS-CHACHA", ["rfc8439_2_3_2"],
     ["rfc8439_2_3_2_expected"], None, None, None, "published_standard", "RFC 8439 section 2.3.2"),
    ("K-CHACHA-02", 6, [_r(0, 32), _le32(0), _r(0, 12)], "ok", "OS-CHACHA", ["rfc8439_a1_1"],
     ["rfc8439_a1_1_expected"], None, None, None, "published_standard", "RFC 8439 appendix A.1, test vector 1"),
    ("K-CHACHA-03", 6, [_r(0, 32), _le32(1), _r(0, 12)], "ok", "OS-CHACHA", ["rfc8439_a1_2"],
     ["rfc8439_a1_2_expected"], None, None, None, "published_standard", "RFC 8439 appendix A.1, test vector 2"),
    ("K-CHACHA-04", 6, [_r(0, 31) + "01", _le32(1), _r(0, 12)], "ok", "OS-CHACHA", ["rfc8439_a1_3"],
     ["rfc8439_a1_3_expected"], None, None, None, "published_standard", "RFC 8439 appendix A.1, test vector 3"),
    ("K-CHACHA-05", 6, ["00ff" + _r(0, 30), _le32(2), _r(0, 12)], "ok", "OS-CHACHA", ["rfc8439_a1_4"],
     ["rfc8439_a1_4_expected"], None, None, None, "published_standard", "RFC 8439 appendix A.1, test vector 4"),
    ("K-CHACHA-06", 6, [_r(0, 32), _le32(0), _r(0, 11) + "02"], "ok", "OS-CHACHA", ["rfc8439_a1_5"],
     ["rfc8439_a1_5_expected"], None, None, None, "published_standard", "RFC 8439 appendix A.1, test vector 5"),
    ("K-CHACHA-07", 7, [_seq(0, 32), _le32(1), "000000000000004a00000000", SUNSCREEN], "ok", "OS-CHACHA",
     ["rfc8439_2_4_2"], ["rfc8439_2_4_2_expected"], None, None, None, "published_standard", "RFC 8439 section 2.4.2"),
    ("K-CHACHA-08", 7, [_r(0, 32), _le32(0), _r(0, 12), _r(0, 64)], "ok", "OS-CHACHA", ["rfc8439_a2_1"],
     ["rfc8439_a2_1_expected"], None, None, None, "published_standard", "RFC 8439 appendix A.2, test vector 1"),
    ("K-CHACHA-09", 7, [_r(0, 31) + "01", _le32(1), _r(0, 11) + "02", IETF_TEXT], "ok", "OS-CHACHA",
     ["rfc8439_a2_2_head", "rfc8439_a2_2_tail"], ["rfc8439_a2_2_head_expected", "rfc8439_a2_2_tail_expected"], None,
     None, None, "published_standard", "RFC 8439 appendix A.2, test vector 2"),
    ("K-CHACHA-10", 7, [A5_KEY, _le32(42), _r(0, 11) + "02", JABBERWOCKY], "ok", "OS-CHACHA", ["rfc8439_a2_3"],
     ["rfc8439_a2_3_expected"], None, None, None, "published_standard", "RFC 8439 appendix A.2, test vector 3"),
    ("K-POLY-01", 8, ["85d6be7857556d337f4452fe42d506a80103808afb0db2fd4abff6af4149f51b",
                      _t("Cryptographic Forum Research Group")], "ok", "OS-AEAD", ["rfc8439_2_5_2"],
     ["rfc8439_2_5_2_expected"], None, None, None, "published_standard", "RFC 8439 section 2.5.2"),
    ("K-POLY-02", 8, [_r(0, 32), _r(0, 64)], "ok", "OS-AEAD", ["rfc8439_a3_1"], ["rfc8439_a3_1_expected"], None,
     None, None, "published_standard", "RFC 8439 appendix A.3, test vector 1"),
    ("K-POLY-03", 8, [_r(0, 16) + "36e5f6b5c5e06070f0efca96227a863e", IETF_TEXT], "ok", "OS-AEAD", ["rfc8439_a3_2"],
     ["rfc8439_a3_2_expected"], None, None, None, "published_standard", "RFC 8439 appendix A.3, test vector 2"),
    ("K-POLY-04", 8, ["36e5f6b5c5e06070f0efca96227a863e" + _r(0, 16), IETF_TEXT], "ok", "OS-AEAD", ["rfc8439_a3_3"],
     ["rfc8439_a3_3_expected"], None, None, None, "published_standard", "RFC 8439 appendix A.3, test vector 3"),
    ("K-POLY-05", 8, [A5_KEY, JABBERWOCKY], "ok", "OS-AEAD", ["rfc8439_a3_4"], ["rfc8439_a3_4_expected"], None, None,
     None, "published_standard", "RFC 8439 appendix A.3, test vector 4"),
    ("K-POLY-06", 8, ["02" + _r(0, 31), _r(0xFF, 16)], "ok", "OS-AEAD", ["rfc8439_a3_5"], ["rfc8439_a3_5_expected"],
     None, None, None, "published_standard", "RFC 8439 appendix A.3, test vector 5"),
    ("K-POLY-07", 8, ["02" + _r(0, 15) + _r(0xFF, 16), "02" + _r(0, 15)], "ok", "OS-AEAD", ["rfc8439_a3_6"],
     ["rfc8439_a3_6_expected"], None, None, None, "published_standard", "RFC 8439 appendix A.3, test vector 6"),
    ("K-POLY-08", 8, ["01" + _r(0, 31), _r(0xFF, 16) + "f0" + _r(0xFF, 15) + "11" + _r(0, 15)], "ok", "OS-AEAD",
     ["rfc8439_a3_7"], ["rfc8439_a3_7_expected"], None, None, None, "published_standard",
     "RFC 8439 appendix A.3, test vector 7"),
    ("K-POLY-09", 8, ["01" + _r(0, 31), _r(0xFF, 16) + "fb" + _r(0xFE, 15) + _r(0x01, 16)], "ok", "OS-AEAD",
     ["rfc8439_a3_8"], ["rfc8439_a3_8_expected"], None, None, None, "published_standard",
     "RFC 8439 appendix A.3, test vector 8"),
    ("K-POLY-10", 8, ["02" + _r(0, 31), "fd" + _r(0xFF, 15)], "ok", "OS-AEAD", ["rfc8439_a3_9"],
     ["rfc8439_a3_9_expected"], None, None, None, "published_standard", "RFC 8439 appendix A.3, test vector 9"),
    ("K-POLY-11", 8, [_A3_10_KEY, _POLY11 + "01" + _r(0, 15)], "ok", "OS-AEAD", ["rfc8439_a3_10"],
     ["rfc8439_a3_10_expected"], None, None, None, "published_standard", "RFC 8439 appendix A.3, test vector 10"),
    ("K-POLY-12", 8, [_A3_10_KEY, _POLY11], "ok", "OS-AEAD", ["rfc8439_a3_11"], ["rfc8439_a3_11_expected"], None,
     None, None, "published_standard", "RFC 8439 appendix A.3, test vector 11"),
    ("K-POLY-13", 6, [AEAD_KEY, _le32(0), "000000000001020304050607"], "ok", "OS-AEAD", ["rfc8439_2_6_2"],
     ["rfc8439_2_6_2_expected"], None, [[0, 32]], None, "published_standard",
     "RFC 8439 section 2.6.2: the Poly1305 one-time key is the first 32 bytes of the ChaCha20 block, counter 0"),
    ("K-POLY-14", 6, [A5_KEY, _le32(0), _r(0, 11) + "02"], "ok", "OS-AEAD", ["rfc8439_a4_3"],
     ["rfc8439_a4_3_expected"], None, [[0, 32]], None, "published_standard",
     "RFC 8439 appendix A.4, test vector 3 (the first 32 bytes of the block)"),
    ("K-AEAD-01", 9, [AEAD_KEY, "070000004041424344454647", "50515253c0c1c2c3c4c5c6c7", SUNSCREEN], "ok", "OS-AEAD",
     ["rfc8439_2_8_2"], ["rfc8439_2_8_2_expected"], None, None, None, "published_standard", "RFC 8439 section 2.8.2"),
    ("K-AEAD-02", 10, [AEAD_KEY, "070000004041424344454647", "50515253c0c1c2c3c4c5c6c7", "@K-AEAD-01"], "ok",
     "OS-AEAD", ["rfc8439_2_8_2_open"], ["rfc8439_2_8_2_open_expected"], None, None, "rfc8439_2_8_2_verify",
     "published_standard", "RFC 8439 section 2.8.2, opened"),
    ("K-AEAD-03", 10, [A5_KEY, "000000000102030405060708", "f33388860000000000004e91", A5_CT], "ok", "OS-AEAD",
     ["rfc8439_a5_head", "rfc8439_a5_tail"], ["rfc8439_a5_head_expected", "rfc8439_a5_tail_expected"], None, None,
     "rfc8439_a5_verify", "published_standard", "RFC 8439 appendix A.5 (authenticated decryption)"),
    ("K-AEAD-04", 9, ["6cbfd71c645d184cf5d23c402bdb0d25ec54898c8a0273d42eb5be109fdcb2ac", "d4d807341683825b31cd4d95",
                      "b3e4064683b02d84", "a98995504df16f748bfb7785ff91eeb3b660ea9ed3450c3d5e7b0e79ef653659a9978d7554"
                                          "2ef91c456762215640b9"], "ok", "OS-AEAD",
     ["wycheproof_chacha20_poly1305_tc_71"], ["wycheproof_chacha20_poly1305_tc_71_expected"], None, None, None,
     "published_test_suite", "Project Wycheproof chacha20_poly1305_test.json, tcId 71"),
    ("K-X25519-01", 11, ["a546e36bf0527c9d3b16154b82465edd62144c0ac1fc5a18506a2244ba449ac4",
                         "e6db6867583030db3594c1a424b15f7c726624ec26b3353b10a903a6d0ab1c4c"], "ok", "OS-X25519",
     ["rfc7748_5_2_vector_1"], ["rfc7748_5_2_vector_1_expected"], None, None, None, "published_standard",
     "RFC 7748 section 5.2, first test vector"),
    ("K-X25519-02", 11, ["4b66e9d4d1b4673c5ad22691957d6af5c11b6421e0ea01d42ca4169e7918ba0d",
                         "e5210f12786811d3f4b7959d0538ae2c31dbe7106fc03c3efc4cd549c715a493"], "ok", "OS-X25519-2",
     ["rfc7748_5_2_vector_2"], ["rfc7748_5_2_vector_2_expected"], None, None, None, "published_standard",
     "RFC 7748 section 5.2, second test vector"),
    ("K-X25519-03", 11, ["77076d0a7318a57d3c16c17251b26645df4c2f87ebc0992ab177fba51db92c2a",
                         "de9edb7d7b7dc1b4d35b61c2ece435373f8343c85b78674dadfc7e146f882b4f"], "ok", "OS-X25519-DH",
     ["rfc7748_6_1_shared_secret"], ["rfc7748_6_1_shared_secret_expected"], None, None, None, "published_standard",
     "RFC 7748 section 6.1, the shared secret from Alice's private key and Bob's public key"),
    ("K-X25519-04", 11, ["c8a9d5a91091ad851c668b0736c1c9a02936c0d3ad62670858088047ba057475",
                         "504a36999f489cd2fdbc08baff3d88fa00569ba986cba22548ffde80f9806829"], "ok", "OS-X25519-WP",
     ["wycheproof_x25519_tc_1"], ["wycheproof_x25519_tc_1_expected"], None, None, None, "published_test_suite",
     "Project Wycheproof x25519_test.json, tcId 1"),
    ("K-AES-01", 12, [_seq(0, 16), "00112233445566778899aabbccddeeff"], "ok", "OS-AES", ["fips197_c1_aes128"],
     ["fips197_c1_aes128_expected"], None, None, None, "published_standard", "FIPS 197 appendix C.1 (AES-128)"),
    ("K-AES-02", 12, [_seq(0, 32), "00112233445566778899aabbccddeeff"], "ok", "OS-AES", ["fips197_c3_aes256"],
     ["fips197_c3_aes256_expected"], None, None, None, "published_standard", "FIPS 197 appendix C.3 (AES-256)"),
    ("K-AES-03", 12, ["2b7e151628aed2a6abf7158809cf4f3c", "3243f6a8885a308d313198a2e0370734"], "ok", "OS-AES",
     ["fips197_b_aes128"], ["fips197_b_aes128_expected"], None, None, None, "published_standard",
     "FIPS 197 appendix B (the cipher example)"),
    ("K-GCM-01", 13, [_r(0, 16), _r(0, 12), "", ""], "ok", "OS-GCM", ["nist_gcm_test_case_1"],
     ["nist_gcm_test_case_1_expected"], [[64, 80]], None, None, "published_standard",
     "GCM specification (McGrew and Viega), test case 1"),
    ("K-GCM-02", 13, [_r(0, 16), _r(0, 12), "", _r(0, 16)], "ok", "OS-GCM", ["nist_gcm_test_case_2"],
     ["nist_gcm_test_case_2_expected"], [[0, 16], [64, 80]], None, None, "published_standard",
     "GCM specification, test case 2"),
    ("K-GCM-03", 13, [GCM_K, GCM_IV, GCM_A, GCM_P], "ok", "OS-GCM", ["nist_gcm_test_case_4"],
     ["nist_gcm_test_case_4_expected"], [[0, 60], [64, 80]], None, None, "published_standard",
     "GCM specification, test case 4"),
    ("K-GCM-04", 13, [_r(0, 32), _r(0, 12), "", ""], "ok", "OS-GCM256", ["nist_gcm_test_case_13"],
     ["nist_gcm_test_case_13_expected"], [[64, 80]], None, None, "published_standard",
     "GCM specification, test case 13 (AES-256)"),
    ("K-GCM-05", 13, [GCM_K + GCM_K, GCM_IV, GCM_A, GCM_P], "ok", "OS-GCM256", ["nist_gcm_test_case_16"],
     ["nist_gcm_test_case_16_expected"], [[0, 60], [64, 80]], None, None, "published_standard",
     "GCM specification, test case 16 (AES-256)"),
    ("K-GCM-06", 14, [GCM_K, GCM_IV, GCM_A, "@K-GCM-03"], "ok", "OS-GCMAD", ["nist_gcm_test_case_4_plaintext"],
     ["nist_gcm_test_case_4_plaintext_expected"], [[0, 60]], None, "nist_gcm_test_case_4_verify",
     "published_standard", "GCM specification, test case 4, authenticated decryption"),
    ("K-SHA3-01", 15, [""], "ok", "OS-SHA3", ["botan_sha3_256_empty"], ["botan_sha3_256_empty_expected"], None,
     None, None, "published_test_suite", "Botan sha3.vec, SHA3-256 of the empty string"),
    ("K-SHA3-02", 15, [_t("abc")], "ok", "OS-SHA3", ["hashlib_sha3_256_abc"], ["hashlib_sha3_256_abc_expected"],
     None, None, None, "library_produced", "SHA3-256 of \"abc\" as algorithms/sha3 recorded it from Python hashlib"),
    ("K-SHA3-03", 15, [_seq(0, 200)], "ok", "OS-SHA3", ["hashlib_sha3_256_200_bytes"],
     ["hashlib_sha3_256_200_bytes_expected"], None, None, None, "library_produced",
     "SHA3-256 of the bytes 00 to c7 as algorithms/sha3 recorded it from Python hashlib"),
    ("K-SHAKE-01", 16, ["", _le32(32)], "ok", "OS-SHA3", ["hashlib_shake128_empty_32"],
     ["hashlib_shake128_empty_32_expected"], None, None, None, "library_produced",
     "SHAKE128 of the empty string, 32 bytes, as algorithms/sha3 recorded it from Python hashlib"),
    ("K-SHAKE-02", 16, [_t("abc"), _le32(168)], "ok", "OS-SHA3", ["hashlib_shake128_abc_168"],
     ["hashlib_shake128_abc_168_expected"], None, None, None, "library_produced",
     "SHAKE128 of \"abc\", 168 bytes, as algorithms/sha3 recorded it from Python hashlib"),
)
# SUBJECTS_END


def _resolve(arg: str) -> str:
    """An argument "@K-..." is that earlier subject's expected output."""

    return EXPECTED[arg[1:]] if arg.startswith("@") else arg


def subject_rows() -> list[dict[str, Any]]:
    rows = []
    for (sid, op, args, status, source, compute, literal, slices, native_slices, status_spec, kind,
         citation) in SUBJECTS:
        rows.append({
            "id": sid, "op": op, "op_name": OPS[op][0], "family": OPS[op][1], "args": [_resolve(a) for a in args],
            "expect": {"status": status, "output": EXPECTED[sid]}, "driver_slices": native_slices,
            "accel_op": ACCEL_OF.get(op),
            "provenance": {"kind": kind, "citation": citation},
            "orange": {"source": source, "compute": list(compute), "literal": list(literal),
                       "slices": slices, "status_spec": status_spec},
            "library": LIBRARY_ORACLE[op],
        })
    return rows


LIBRARY_ORACLE = {
    1: "python hashlib.sha256", 2: "python reference (FIPS 180-4 section 6.2.2)", 3: "python hashlib.sha512",
    4: "python hmac", 5: "python hmac composed as RFC 5869", 6: "OpenSSL EVP_chacha20", 7: "OpenSSL EVP_chacha20",
    8: "OpenSSL EVP_MAC POLY1305", 9: "OpenSSL EVP_chacha20_poly1305", 10: "OpenSSL EVP_chacha20_poly1305",
    11: "OpenSSL EVP_PKEY X25519", 12: "OpenSSL EVP_aes_*_ecb", 13: "OpenSSL EVP_aes_*_gcm",
    14: "OpenSSL EVP_aes_*_gcm", 15: "python hashlib.sha3_256", 16: "python hashlib.shake_128",
}


def negative_rows() -> list[dict[str, Any]]:
    """NT-04's negative checks. `request` checks run on every build unless `profiles` narrows them."""

    by_id = {row[0]: row for row in SUBJECTS}
    aead = by_id["K-AEAD-01"]
    aead_ct = bytearray(bytes.fromhex(EXPECTED["K-AEAD-01"]))
    tag_flip = bytes(aead_ct[:-1] + bytes([0x90]))  # the byte Orange's rfc8439_2_8_2_tampered_verify writes
    ct_flip = bytearray(aead_ct)
    ct_flip[0] ^= 0x01
    gcm_ct = bytearray(bytes.fromhex(EXPECTED["K-GCM-03"]))
    gcm_tag_flip = bytearray(gcm_ct)
    gcm_tag_flip[-1] ^= 0x80
    rows = [
        {"id": "N-01", "kind": "request", "name": "ChaCha20-Poly1305 open with a changed tag byte",
         "op": 10, "args": [aead[2][0], aead[2][1], aead[2][2], tag_flip.hex()], "expect": "rejected",
         "orange": {"source": "OS-AEAD", "status_spec": "rfc8439_2_8_2_tampered_verify"}, "profiles": list(PROFILE_IDS)},
        {"id": "N-02", "kind": "request", "name": "ChaCha20-Poly1305 open with a flipped ciphertext bit",
         "op": 10, "args": [aead[2][0], aead[2][1], aead[2][2], bytes(ct_flip).hex()], "expect": "rejected",
         "orange": None, "profiles": list(PROFILE_IDS)},
        {"id": "N-03", "kind": "request", "name": "AES-GCM open with a flipped tag bit",
         "op": 14, "args": [GCM_K, GCM_IV, GCM_A, bytes(gcm_tag_flip).hex()], "expect": "rejected",
         "orange": None, "profiles": list(PROFILE_IDS)},
        {"id": "N-04", "kind": "request", "name": "AES-GCM open with changed additional data",
         "op": 14, "args": [GCM_K, GCM_IV, "00" + GCM_A[2:], EXPECTED["K-GCM-03"]], "expect": "rejected",
         "orange": None, "profiles": list(PROFILE_IDS)},
        {"id": "N-05", "kind": "request", "name": "Accelerated AES-GCM open with a flipped tag bit",
         "op": 19, "args": [GCM_K, GCM_IV, GCM_A, bytes(gcm_tag_flip).hex()], "expect": "rejected",
         "orange": None, "profiles": ["P-CRYPTO"]},
        {"id": "N-06", "kind": "request", "name": "SHA-256 probe with a 63-byte block",
         "op": 2, "args": [_r(0, 63)], "expect": "malformed", "orange": None, "profiles": list(PROFILE_IDS)},
        {"id": "N-07", "kind": "request", "name": "AES with a 24-byte key (outside the kernel's key sizes)",
         "op": 12, "args": [_r(0, 24), _r(0, 16)], "expect": "malformed", "orange": None, "profiles": list(PROFILE_IDS)},
        {"id": "N-08", "kind": "request", "name": "HKDF asking for more than 255 blocks",
         "op": 5, "args": [_r(0x0b, 22), "", "", _le32(8161)], "expect": "malformed", "orange": None,
         "profiles": list(PROFILE_IDS)},
        {"id": "N-09", "kind": "request", "name": "Unknown operation", "op": 99, "args": [], "expect": "unsupported",
         "orange": None, "profiles": list(PROFILE_IDS)},
        {"id": "N-10", "kind": "request", "name": "Accelerated AES in the baseline profile",
         "op": 17, "args": [_seq(0, 16), "00112233445566778899aabbccddeeff"], "expect": "unsupported",
         "orange": None, "profiles": ["P-BASE"]},
        {"id": "N-11", "kind": "truncated", "name": "A request whose argument is cut short",
         "expect": "exit status 2 after the complete responses", "profiles": list(PROFILE_IDS)},
        {"id": "N-12", "kind": "feature", "name": "Crypto-profile binary on a CPU model without the extension",
         "expect": "SIGILL, no response", "profiles": ["P-CRYPTO"]},
        {"id": "N-13", "kind": "corruption", "name": "One flipped bit in the linked SHA-256 round constants",
         "expect": "the SHA-256 known answers fail", "profiles": ["P-BASE"]},
    ]
    return rows


# Trace groups: each variant fills the `secret` arguments with derived bytes of the stated lengths and keeps the
# public arguments fixed, so every variant of a group has one public shape.
TRACE_GROUPS = (
    ("TG-SHA256", 1, [("secret", 64)], "equal", PROFILE_IDS),
    ("TG-SHA512", 3, [("secret", 64)], "equal", PROFILE_IDS),
    ("TG-HMAC", 4, [("secret", 32), ("secret", 64)], "equal", PROFILE_IDS),
    ("TG-HKDF", 5, [("secret", 32), ("public", _seq(0, 13)), ("public", _seq(0xF0, 10)), ("public", _le32(42))],
     "equal", PROFILE_IDS),
    ("TG-CHACHA", 6, [("secret", 32), ("public", _le32(1)), ("public", "000000090000004a00000000")], "equal",
     PROFILE_IDS),
    ("TG-POLY", 8, [("secret", 32), ("secret", 64)], "equal", PROFILE_IDS),
    ("TG-AEAD-SEAL", 9, [("secret", 32), ("public", "070000004041424344454647"), ("public", "50515253c0c1c2c3c4c5c6c7"),
                         ("secret", 64)], "equal", PROFILE_IDS),
    ("TG-AEAD-OPEN", 10, [("secret", 32), ("public", "070000004041424344454647"),
                          ("public", "50515253c0c1c2c3c4c5c6c7"), ("sealed", 64)], "equal", PROFILE_IDS),
    ("TG-X25519", 11, [("secret", 32), ("public", "09" + _r(0, 31))], "equal", PROFILE_IDS),
    ("TG-AES128", 12, [("secret", 16), ("secret", 16)], "equal", PROFILE_IDS),
    ("TG-AES256", 12, [("secret", 32), ("secret", 16)], "equal", PROFILE_IDS),
    ("TG-GCM", 13, [("secret", 16), ("public", GCM_IV), ("public", GCM_A), ("secret", 64)], "equal", PROFILE_IDS),
    ("TG-SHA3", 15, [("secret", 64)], "equal", PROFILE_IDS),
    ("TG-SHAKE", 16, [("secret", 32), ("public", _le32(64))], "equal", PROFILE_IDS),
    ("TG-ACCEL-AES", 17, [("secret", 16), ("secret", 16)], "equal", ("P-CRYPTO",)),
    ("TG-ACCEL-GCM", 18, [("secret", 16), ("public", GCM_IV), ("public", GCM_A), ("secret", 64)], "equal",
     ("P-CRYPTO",)),
    ("TG-CONTROL", 20, [("differing", 32), ("differing", 32)], "differ", PROFILE_IDS),
)


def trace_group_rows() -> list[dict[str, Any]]:
    return [{"id": gid, "op": op, "op_name": OPS[op][0], "args": [{"kind": k, "value": v} for k, v in args],
             "expect": expect, "profiles": list(profiles),
             "role": "detector control" if expect == "differ" else "kernel"} for gid, op, args, expect, profiles in
            TRACE_GROUPS]


def derive(label: str, length: int) -> bytes:
    """Deterministic variant bytes: SHA-256 in counter mode over a label."""

    out = b""
    counter = 0
    while len(out) < length:
        out += hashlib.sha256(f"{SUITE_VERSION}|{label}|{counter}".encode("ascii")).digest()
        counter += 1
    return out[:length]


GAPS = (
    ("G-01", "Ed25519 signatures", "RFC 8032; proposed D-015 family. No stand-in kernel and no Orange source in "
     "algorithms/ at the base revision."),
    ("G-02", "ML-KEM arithmetic", "FIPS 203; only its SHA-3 and SHAKE core is covered (K-SHA3, K-SHAKE). "
     "algorithms/README.md lists ML-KEM-512 as being written."),
    ("G-03", "Post-quantum signatures", "FIPS 204 or FIPS 205; no stand-in kernel."),
    ("G-04", "AES-192", "FIPS 197 appendix C.2 is reproduced by algorithms/aes/aes.or, but the stand-in kernel "
     "supports 128- and 256-bit keys only; N-07 checks that a 24-byte key is refused."),
    ("G-05", "GCM with IVs other than 96 bits", "SP 800-38D section 7.1; GCM test case 6 and Wycheproof tcId 2 "
     "(algorithms/aes-gcm/aes-gcm-iv.or) need GHASH-derived J0, which the kernel does not implement."),
    ("G-06", "XChaCha20 and XChaCha20-Poly1305", "draft-irtf-cfrg-xchacha; reproduced in algorithms/ but outside "
     "the stand-in kernel."),
    ("G-07", "HMAC truncation and HKDF edge cases", "RFC 4231 test case 5 (truncated output) and Wycheproof HKDF "
     "tcId 69 are not driver subjects."),
    ("G-08", "Memory-address and timing channels", "QEMU user mode offers no memory-access trace without plugins "
     "(not built into the distribution package); NT-03 sees control flow only."),
    ("G-09", "AArch64 feature negative", "QEMU 8.2's AArch64 CPU models all implement FEAT_AES, so N-12 cannot run "
     "for T-A64 in emulation; an AArch64 device without the Cryptographic Extension closes it."),
    ("G-10", "RV64 second C toolchain and Rust standard libraries", "gcc-13-riscv64-linux-gnu and the Rust "
     "aarch64 and riscv64gc standard libraries are not installed on the laboratory host."),
)

# ---------------------------------------------------------------------------
# NT-06: the ISA and ABI inventory, from the contributor's knowledge. Nothing below was checked from the
# laboratory host; each row counts only when the owner verifies it.

INVENTORY = (
    ("IA-X64-01", "T-X64", "isa_reference", True, "Intel 64 and IA-32 Architectures Software Developer's Manual",
     "Intel", "intel.com (document 325462)", "public", "prose"),
    ("IA-X64-02", "T-X64", "isa_reference", False, "AMD64 Architecture Programmer's Manual", "AMD",
     "amd.com (publication 24592 and following)", "public", "prose"),
    ("IA-X64-03", "T-X64", "abi", True, "System V Application Binary Interface, AMD64 Architecture Processor "
     "Supplement", "x86-64 psABI maintainers", "gitlab.com/x86-psABIs/x86-64-ABI", "public", "prose"),
    ("IA-X64-04", "T-X64", "formal_model", False, "No vendor formal model; community models cover subsets "
     "(for example the ACL2 x86 ISA model and Sail x86 work)", "community", "various", "public (partial)",
     "machine-readable, partial"),
    ("IA-X64-05", "T-X64", "dit_mechanism", True, "Data Operand Independent Timing Mode (DOITM) and its instruction "
     "list", "Intel", "intel.com security guidance", "public", "prose"),
    ("IA-A64-01", "T-A64", "isa_reference", True, "Arm Architecture Reference Manual for A-profile architecture",
     "Arm", "developer.arm.com (DDI 0487)", "public (registration for some downloads)", "prose"),
    ("IA-A64-02", "T-A64", "abi", True, "Procedure Call Standard for the Arm 64-bit Architecture (AAPCS64) and ELF "
     "for the Arm 64-bit Architecture", "Arm", "github.com/ARM-software/abi-aa", "public", "prose"),
    ("IA-A64-03", "T-A64", "formal_model", False, "Arm's machine-readable A-profile specification (ASL) and its "
     "Sail translation", "Arm; REMS project", "developer.arm.com; github.com/rems-project/sail-arm", "public",
     "machine-readable"),
    ("IA-A64-04", "T-A64", "dit_mechanism", True, "FEAT_DIT: PSTATE.DIT and the instructions it covers", "Arm",
     "Arm ARM (DDI 0487)", "public", "prose"),
    ("IA-RV-01", "T-RV64", "isa_reference", True, "The RISC-V Instruction Set Manual, Volumes I and II",
     "RISC-V International", "github.com/riscv/riscv-isa-manual", "public", "prose"),
    ("IA-RV-02", "T-RV64", "abi", True, "RISC-V ELF psABI (LP64D)", "RISC-V International",
     "github.com/riscv-non-isa/riscv-elf-psabi-doc", "public", "prose"),
    ("IA-RV-03", "T-RV64", "formal_model", False, "sail-riscv, the RISC-V golden reference model",
     "RISC-V International", "github.com/riscv/sail-riscv", "public", "machine-readable"),
    ("IA-RV-04", "T-RV64", "dit_mechanism", True, "Zkt: data-independent execution latency (scalar cryptography "
     "v1.0)", "RISC-V International", "github.com/riscv/riscv-crypto", "public", "prose"),
    ("IA-RV-05", "T-RV64", "profile", False, "RVA20 and RVA22 profiles (RV64GC base)", "RISC-V International",
     "github.com/riscv/riscv-profiles", "public", "prose"),
)


def inventory_rows() -> list[dict[str, Any]]:
    return [{"id": rid, "tuple": tup, "kind": kind, "required": required, "name": name, "publisher": pub,
             "reference": ref, "availability": avail, "form": form,
             "status": "contributor_recorded_unverified", "checked_from_laboratory_host": False,
             "note": "recorded from the contributor's knowledge; not checked from this machine"}
            for rid, tup, kind, required, name, pub, ref, avail, form in INVENTORY]


def owner_template() -> dict[str, Any]:
    """The owner input's shape, empty. The laboratory never fills it in."""

    return {
        "schema": OWNER_SCHEMA,
        "suite_version": SUITE_VERSION,
        "note": "To be completed by the owner only. The laboratory reads it and records its digest; it does not "
                "verify who wrote it.",
        "supplied_by": "",
        "date": "",
        "hardware": [],
        "hardware_example": {"tuple": "T-X64", "device": "", "cpu": "", "features": [], "runs_natively": True},
        "no_hardware": [],
        "native_runs": [],
        "native_run_example": {"tuple": "T-A64", "device": "", "driver_sha256": "", "request_file": "kat-P-BASE.bin",
                               "stdout_sha256": "",
                               "how": "run products/<driver_sha256>.elf from the epoch archive natively on a device "
                                      "attested under hardware for the same tuple, with products/kat-<profile>.bin "
                                      "on standard input, and record the device and the SHA-256 of standard "
                                      "output"},
        "feature_negatives": [],
        "feature_negative_example": {"tuple": "T-A64", "device": "", "driver_sha256": "", "result": "SIGILL",
                                     "how": "for each P-CRYPTO driver whose N-12 check is unresolved, run it from "
                                            "the archive on a named device without the extension with "
                                            "products/feature-negative.bin on standard input and record how it "
                                            "ends; an entry settles only the driver it names"},
        "isa_abi_verification": [{"row": row[0], "verdict": "", "note": ""} for row in INVENTORY],
        "review_scopes": [{"scope": scope[0], "done": False, "note": ""} for scope in REVIEW_SCOPES],
        "distinguishing_rule": None,
        "solo_slice_capacity": None,
    }


# ---------------------------------------------------------------------------
# The packet


def input_manifest(root: Path) -> list[dict[str, Any]]:
    rows = []
    for rel in BOUND_INPUTS:
        path = root / rel
        if not path.is_file():
            raise SuiteError(f"bound input missing: {rel}")
        data = path.read_bytes()
        rows.append({"path": rel, "sha256": sha256_hex(data), "bytes": len(data)})
    return rows


def build_packet(root: Path = REPOSITORY_ROOT) -> dict[str, Any]:
    return {
        "schema": PACKET_SCHEMA,
        "suite_version": SUITE_VERSION,
        "decision": "D-011",
        "title": "Initial native target envelope",
        "status": "contributor_draft_unreviewed",
        "label": LABEL,
        "base_revision": BASE_REVISION,
        "owner": "Chase Bryan (the decision and every freeze are the owner's; none is recorded here)",
        "stand_in_statement": "The native code is contributor-written portable C reference kernels standing in "
                              "for Orange output. The laboratory measures target feasibility, not Orange code "
                              "generation.",
        "nonclaims": list(NONCLAIMS),
        "candidates": [{"id": cid, "name": name, "native_tuples": list(tuples), "portable_path": "C-PORTABLE",
                        "claim_bearing_native_targets": len(tuples), "why_in_set": why,
                        "cases": list(CASE_IDS), "gates": list(GATE_IDS)}
                       for cid, name, tuples, why in CANDIDATES],
        "candidate_parity": "Every candidate is the portable path plus a set of tuples; every tuple and the "
                            "portable path run the same cases, subjects, gates and budgets, and a candidate's "
                            "result is computed from its members only.",
        "tuples": [dict(t) for t in TUPLES],
        "portable_path": dict(PORTABLE_PATH),
        "profiles": [dict(p) for p in PROFILES],
        "toolchains": [dict(t) for t in TOOLCHAINS],
        "build_flags": BUILD_FLAGS,
        "parts": {"kernel": list(KERNEL_PARTS), "crypto_profile": list(ACCEL_PARTS), "runtime": list(RUNTIME_PARTS),
                  "family_of_part": FAMILY_OF_PART},
        "operations": {str(k): {"name": v[0], "family": v[1], "arguments": v[2]} for k, v in OPS.items()},
        "statuses": {str(k): v for k, v in STATUS.items()},
        "instruction_classes": INSTRUCTION_CLASSES,
        "cases": [dict(c) for c in CASES],
        "metrics": [{"id": m, "name": n, "definition": d} for m, n, d in METRICS],
        "gates": [{"id": g, "name": n, "case": c, "rule": r} for g, n, c, r in GATES],
        "gate_states": list(GATE_STATES),
        "gate_precedence": ["unsupported", "fail", "unresolved", "pass"],
        "portable_vacuous_gates": list(PORTABLE_VACUOUS),
        "axes": [{"id": a, "name": n, "measure": m, "better": b, "material": mat} for a, n, m, b, mat in AXES],
        "distinguishing_rules": [{"id": r, "name": n, "rule": d} for r, n, d in RULES],
        "owner_distinguishing_rule": None,
        "review_scopes": [{"id": s, "name": n, "covers": c, "recorded": False} for s, n, c in REVIEW_SCOPES],
        "run_profiles": RUN_PROFILES,
        "decision_procedure": list(DECISION_PROCEDURE),
        "oracle_sources": [{"id": sid, "path": path, "sha256": digest, "revision": BASE_REVISION}
                           for sid, path, digest in ORACLE_SOURCES],
        "published_carriers": [{"id": cid, "path": path, "sha256": digest, "revision": BASE_REVISION, "role": role}
                               for cid, path, digest, role in PUBLISHED_CARRIERS],
        "subjects": subject_rows(),
        "negatives": negative_rows(),
        "trace_groups": trace_group_rows(),
        "gaps": [{"id": g, "name": n, "detail": d} for g, n, d in GAPS],
        "isa_abi_inventory": inventory_rows(),
        "owner_input_template": owner_template(),
        "inputs": input_manifest(root),
    }


def validate_packet(packet: dict[str, Any]) -> list[str]:
    """Structural checks that hold for any packet this suite version accepts."""

    errors: list[str] = []

    def need(condition: bool, message: str) -> None:
        if not condition:
            errors.append(message)

    need(packet.get("schema") == PACKET_SCHEMA, "schema")
    need(packet.get("suite_version") == SUITE_VERSION, "suite_version")
    need(packet.get("status") == "contributor_draft_unreviewed", "status must stay contributor_draft_unreviewed")
    need(packet.get("owner_distinguishing_rule") is None, "the packet never records an owner rule")
    need(all(not s.get("recorded") for s in packet.get("review_scopes", [])), "the packet records no owner review")
    errors += gate0_json_errors(packet)
    candidates = packet.get("candidates", [])
    need([c["id"] for c in candidates] == ["TE-01", "TE-02", "TE-03", "TE-04", "TE-05"], "candidate set")
    tuples = {t["id"] for t in packet.get("tuples", [])}
    shapes = {tuple(sorted(c)) for c in candidates}
    need(len(shapes) == 1, "candidate parity: every candidate has the same fields")
    for cand in candidates:
        need(set(cand["native_tuples"]) <= tuples, f"{cand['id']}: unknown tuple")
        need(cand["cases"] == list(CASE_IDS) and cand["gates"] == list(GATE_IDS), f"{cand['id']}: parity of cases")
        need(cand["claim_bearing_native_targets"] == len(cand["native_tuples"]), f"{cand['id']}: target count")
    envelopes = [frozenset(c["native_tuples"]) for c in candidates]
    need(len(set(envelopes)) == len(envelopes), "candidates are distinct envelopes")
    for tup in packet.get("tuples", []):
        need(tup["id"] in packet["build_flags"]["targets"], f"{tup['id']}: build flags")
        need(all(tc in TOOLCHAIN_BY_ID for tc in tup["toolchains"]), f"{tup['id']}: toolchains")
    ids: set[str] = set()
    for group in ("subjects", "negatives", "trace_groups", "gaps", "isa_abi_inventory", "metrics", "gates", "cases"):
        for row in packet.get(group, []):
            need(row["id"] not in ids, f"duplicate id {row['id']}")
            ids.add(row["id"])
    sources = {s["id"] for s in packet.get("oracle_sources", [])}
    for subj in packet.get("subjects", []):
        op = subj["op"]
        need(op in PORTABLE_OPS, f"{subj['id']}: op")
        need(len(subj["args"]) == OPS[op][2], f"{subj['id']}: argument count")
        need(all(re.fullmatch(r"(?:[0-9a-f]{2})*", a) is not None for a in subj["args"]), f"{subj['id']}: args hex")
        need(subj["expect"]["status"] in STATUS_CODE, f"{subj['id']}: status")
        need(re.fullmatch(r"(?:[0-9a-f]{2})*", subj["expect"]["output"]) is not None, f"{subj['id']}: output hex")
        need(subj["orange"]["source"] in sources, f"{subj['id']}: oracle source")
        need(bool(subj["orange"]["compute"]), f"{subj['id']}: an Orange computation")
        need(subj["provenance"]["kind"] in ("published_standard", "published_test_suite", "library_produced"),
             f"{subj['id']}: provenance kind")
    for neg in packet.get("negatives", []):
        need(set(neg["profiles"]) <= set(PROFILE_IDS), f"{neg['id']}: profiles")
        if neg["kind"] == "request":
            need(neg["expect"] in STATUS_CODE, f"{neg['id']}: expect")
    for group in packet.get("trace_groups", []):
        need(group["expect"] in ("equal", "differ"), f"{group['id']}: expect")
        need(len(group["args"]) == OPS[group["op"]][2], f"{group['id']}: argument count")
    need(sum(1 for g in packet.get("trace_groups", []) if g["expect"] == "differ") >= 1, "a trace control group")
    for row in packet.get("isa_abi_inventory", []):
        need(row["status"] == "contributor_recorded_unverified", f"{row['id']}: status")
        need(row["tuple"] in tuples, f"{row['id']}: tuple")
    for tup in tuples:
        kinds = {r["kind"] for r in packet.get("isa_abi_inventory", []) if r["tuple"] == tup and r["required"]}
        need({"isa_reference", "abi", "dit_mechanism"} <= kinds, f"{tup}: required inventory rows")
    need([row["path"] for row in packet.get("inputs", [])] == list(BOUND_INPUTS), "bound inputs")
    return errors


def committed_packet_bytes(root: Path = REPOSITORY_ROOT) -> bytes:
    return canonical_file(build_packet(root))


def check(root: Path = REPOSITORY_ROOT) -> list[str]:
    path = root / PACKET_PATH
    if not path.is_file():
        return [f"{PACKET_PATH} is missing"]
    data = path.read_bytes()
    problems = []
    try:
        expected = committed_packet_bytes(root)
    except SuiteError as exc:
        return [str(exc)]
    if data != expected:
        problems.append(f"{PACKET_PATH} differs from the generator's output; run generate")
    packet = json.loads(data.decode("utf-8"))
    problems += validate_packet(packet)
    return problems


# ---------------------------------------------------------------------------
# The driver protocol and the Orange evaluation output


def encode_request(op: int, args: Iterable[bytes]) -> bytes:
    args = list(args)
    out = bytes([op, len(args)])
    for arg in args:
        out += struct.pack("<I", len(arg)) + arg
    return out


def parse_responses(data: bytes) -> tuple[list[tuple[str, bytes]], bool]:
    """Responses in order, and whether the stream ended exactly on a response boundary."""

    responses = []
    at = 0
    while at < len(data):
        if len(data) - at < 5:
            return responses, False
        status = data[at]
        length = struct.unpack("<I", data[at + 1:at + 5])[0]
        if len(data) - at - 5 < length:
            return responses, False
        responses.append((STATUS.get(status, f"status-{status}"), data[at + 5:at + 5 + length]))
        at += 5 + length
    return responses, True


IDENTIFIER_START = frozenset("ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz_")
IDENTIFIER_REST = IDENTIFIER_START | frozenset("0123456789")


def _identifier(text: str) -> bool:
    return bool(text) and text[0] in IDENTIFIER_START and all(c in IDENTIFIER_REST for c in text)


def parse_eval(text: str) -> dict[str, tuple[str, str]]:
    """`module::name: Type = value` lines, split at the first `: `, `::` and ` = ` with no pattern
    matching, so a crafted line costs time linear in its length."""

    values = {}
    for line in text.splitlines():
        head, colon, rest = line.partition(": ")
        module, scope, name = head.partition("::")
        type_name, equals, value = rest.partition(" = ")
        if not (colon and scope and equals and type_name and value and _identifier(module) and _identifier(name)):
            raise SuiteError(f"unrecognized evaluation line: {line[:120]}")
        values[name] = (type_name, value)
    return values


def value_bytes(type_name: str, text: str) -> bytes | bool:
    """An evaluated Orange value as the bytes the driver would return: words are big-endian."""

    if type_name == "Bool":
        if text not in ("true", "false"):
            raise SuiteError(f"bad Bool {text}")
        return text == "true"
    match = re.fullmatch(r"Word\[(8|16|32|64)\](?:\^(\d+))?", type_name)
    if match is None:
        raise SuiteError(f"unsupported type {type_name}")
    width = int(match.group(1)) // 8
    words = [int(x, 16) for x in re.findall(r"0x[0-9a-fA-F]+", text)]
    if match.group(2) is not None and len(words) != int(match.group(2)):
        raise SuiteError(f"array length mismatch for {type_name}")
    return b"".join(w.to_bytes(width, "big") for w in words)


def apply_slices(data: bytes, slices: list[list[int]] | None) -> bytes:
    if slices is None:
        return data
    return b"".join(data[a:b] for a, b in slices)


def orange_verdict(subject: dict[str, Any], values: dict[str, tuple[str, str]]) -> dict[str, Any]:
    """Compare a subject's expectation with its Orange computation and its published literal."""

    orange = subject["orange"]
    expected = bytes.fromhex(subject["expect"]["output"])
    result: dict[str, Any] = {"computed": None, "literal": None, "status": None}

    def gather(names: list[str]) -> bytes | None:
        parts = []
        for name in names:
            if name not in values:
                return None
            value = value_bytes(*values[name])
            if isinstance(value, bool):
                raise SuiteError(f"{name} is Bool where bytes are needed")
            parts.append(value)
        return apply_slices(b"".join(parts), orange["slices"])

    computed = gather(orange["compute"])
    result["computed"] = "missing" if computed is None else ("agree" if computed == expected else "conflict")
    if orange["literal"]:
        literal = gather(orange["literal"])
        result["literal"] = "missing" if literal is None else ("agree" if literal == expected else "conflict")
    if orange["status_spec"]:
        spec = orange["status_spec"]
        if spec not in values:
            result["status"] = "missing"
        else:
            verdict = value_bytes(*values[spec])
            want = subject["expect"]["status"] == "ok"
            result["status"] = "agree" if verdict is want else "conflict"
    return result


# ---------------------------------------------------------------------------
# Library oracles: Python's hashlib and hmac, and OpenSSL's libcrypto through ctypes

_SSL: Any = None


def _libcrypto() -> Any:
    global _SSL
    if _SSL is None:
        lib = ctypes.CDLL("libcrypto.so.3")
        vp, ip, cp = ctypes.c_void_p, ctypes.c_int, ctypes.c_char_p
        for name in ("EVP_chacha20", "EVP_chacha20_poly1305", "EVP_aes_128_ecb", "EVP_aes_256_ecb",
                     "EVP_aes_128_gcm", "EVP_aes_256_gcm", "EVP_CIPHER_CTX_new"):
            getattr(lib, name).restype = vp
            getattr(lib, name).argtypes = []
        lib.EVP_CIPHER_CTX_free.argtypes = [vp]
        lib.EVP_CipherInit_ex.argtypes = [vp, vp, vp, cp, cp, ip]
        lib.EVP_CipherUpdate.argtypes = [vp, cp, ctypes.POINTER(ip), cp, ip]
        lib.EVP_CipherFinal_ex.argtypes = [vp, cp, ctypes.POINTER(ip)]
        lib.EVP_CIPHER_CTX_ctrl.argtypes = [vp, ip, ip, vp]
        lib.EVP_CIPHER_CTX_set_padding.argtypes = [vp, ip]
        lib.EVP_MAC_fetch.restype = vp
        lib.EVP_MAC_fetch.argtypes = [vp, cp, cp]
        lib.EVP_MAC_CTX_new.restype = vp
        lib.EVP_MAC_CTX_new.argtypes = [vp]
        lib.EVP_MAC_init.argtypes = [vp, cp, ctypes.c_size_t, vp]
        lib.EVP_MAC_update.argtypes = [vp, cp, ctypes.c_size_t]
        lib.EVP_MAC_final.argtypes = [vp, cp, ctypes.POINTER(ctypes.c_size_t), ctypes.c_size_t]
        lib.EVP_MAC_CTX_free.argtypes = [vp]
        lib.EVP_MAC_free.argtypes = [vp]
        lib.EVP_PKEY_new_raw_private_key.restype = vp
        lib.EVP_PKEY_new_raw_private_key.argtypes = [ip, vp, cp, ctypes.c_size_t]
        lib.EVP_PKEY_new_raw_public_key.restype = vp
        lib.EVP_PKEY_new_raw_public_key.argtypes = [ip, vp, cp, ctypes.c_size_t]
        lib.EVP_PKEY_CTX_new.restype = vp
        lib.EVP_PKEY_CTX_new.argtypes = [vp, vp]
        lib.EVP_PKEY_derive_init.argtypes = [vp]
        lib.EVP_PKEY_derive_set_peer.argtypes = [vp, vp]
        lib.EVP_PKEY_derive.argtypes = [vp, cp, ctypes.POINTER(ctypes.c_size_t)]
        lib.EVP_PKEY_free.argtypes = [vp]
        lib.EVP_PKEY_CTX_free.argtypes = [vp]
        lib.OpenSSL_version.restype = cp
        lib.OpenSSL_version.argtypes = [ip]
        _SSL = lib
    return _SSL


def openssl_version() -> str:
    try:
        return _libcrypto().OpenSSL_version(0).decode("ascii")
    except OSError:
        return "unavailable"


def _cipher(kind: str, key: bytes, iv: bytes | None, data: bytes, encrypt: bool,
            aad: bytes = b"", tag: bytes | None = None) -> tuple[bytes, bytes | None, bool]:
    lib = _libcrypto()
    table = {"chacha20": lib.EVP_chacha20, "chacha20_poly1305": lib.EVP_chacha20_poly1305,
             "aes_ecb": lib.EVP_aes_128_ecb if len(key) == 16 else lib.EVP_aes_256_ecb,
             "aes_gcm": lib.EVP_aes_128_gcm if len(key) == 16 else lib.EVP_aes_256_gcm}
    ctx = lib.EVP_CIPHER_CTX_new()
    try:
        out_len = ctypes.c_int(0)
        if lib.EVP_CipherInit_ex(ctx, table[kind](), None, None, None, int(encrypt)) != 1:
            raise RunError("OpenSSL init")
        if kind == "aes_ecb":
            lib.EVP_CIPHER_CTX_set_padding(ctx, 0)
        if lib.EVP_CipherInit_ex(ctx, None, None, key, iv, int(encrypt)) != 1:
            raise RunError("OpenSSL key")
        if aad:
            if lib.EVP_CipherUpdate(ctx, None, ctypes.byref(out_len), aad, len(aad)) != 1:
                raise RunError("OpenSSL aad")
        buf = ctypes.create_string_buffer(len(data) + 32)
        if lib.EVP_CipherUpdate(ctx, buf, ctypes.byref(out_len), data, len(data)) != 1:
            raise RunError("OpenSSL update")
        produced = buf.raw[:out_len.value]
        if tag is not None:
            tag_buf = ctypes.create_string_buffer(tag, 16)
            lib.EVP_CIPHER_CTX_ctrl(ctx, 0x11, 16, ctypes.cast(tag_buf, ctypes.c_void_p))
        fin = ctypes.create_string_buffer(32)
        ok = lib.EVP_CipherFinal_ex(ctx, fin, ctypes.byref(out_len)) == 1
        produced += fin.raw[:out_len.value] if ok else b""
        got_tag = None
        if encrypt and kind in ("chacha20_poly1305", "aes_gcm"):
            tag_buf = ctypes.create_string_buffer(16)
            lib.EVP_CIPHER_CTX_ctrl(ctx, 0x10, 16, ctypes.cast(tag_buf, ctypes.c_void_p))
            got_tag = tag_buf.raw
        return produced, got_tag, ok
    finally:
        lib.EVP_CIPHER_CTX_free(ctx)


def _poly1305(key: bytes, msg: bytes) -> bytes:
    lib = _libcrypto()
    mac = lib.EVP_MAC_fetch(None, b"POLY1305", None)
    ctx = lib.EVP_MAC_CTX_new(mac)
    try:
        out = ctypes.create_string_buffer(16)
        size = ctypes.c_size_t(0)
        if lib.EVP_MAC_init(ctx, key, len(key), None) != 1 or lib.EVP_MAC_update(ctx, msg, len(msg)) != 1:
            raise RunError("OpenSSL Poly1305")
        if lib.EVP_MAC_final(ctx, out, ctypes.byref(size), 16) != 1:
            raise RunError("OpenSSL Poly1305 final")
        return out.raw[:size.value]
    finally:
        lib.EVP_MAC_CTX_free(ctx)
        lib.EVP_MAC_free(mac)


def _x25519(scalar: bytes, u: bytes) -> bytes:
    lib = _libcrypto()
    private = lib.EVP_PKEY_new_raw_private_key(1034, None, scalar, 32)
    public = lib.EVP_PKEY_new_raw_public_key(1034, None, u, 32)
    ctx = lib.EVP_PKEY_CTX_new(private, None)
    try:
        out = ctypes.create_string_buffer(32)
        size = ctypes.c_size_t(32)
        if (lib.EVP_PKEY_derive_init(ctx) != 1 or lib.EVP_PKEY_derive_set_peer(ctx, public) != 1
                or lib.EVP_PKEY_derive(ctx, out, ctypes.byref(size)) != 1):
            raise RunError("OpenSSL X25519")
        return out.raw[:size.value]
    finally:
        lib.EVP_PKEY_CTX_free(ctx)
        lib.EVP_PKEY_free(private)
        lib.EVP_PKEY_free(public)


def _hkdf(ikm: bytes, salt: bytes, info: bytes, length: int) -> bytes:
    prk = hmac.new(salt or bytes(32), ikm, "sha256").digest()
    out, block, counter = b"", b"", 1
    while len(out) < length:
        block = hmac.new(prk, block + info + bytes([counter]), "sha256").digest()
        out += block
        counter += 1
    return out[:length]


_K256 = None


def _sha256_probe(block: bytes) -> bytes:
    """FIPS 180-4 section 6.2.2 on one block: W16, W17 and the state after rounds 0 and 1."""

    global _K256
    if _K256 is None:
        primes = [p for p in range(2, 320) if all(p % d for d in range(2, int(p ** 0.5) + 1))][:64]

        def root(value: int, k: int) -> int:
            lo, hi = 0, 1
            while hi ** k <= value:
                hi *= 2
            while lo < hi - 1:
                mid = (lo + hi) // 2
                lo, hi = (mid, hi) if mid ** k <= value else (lo, mid)
            return lo

        _K256 = [root(p << 96, 3) & 0xFFFFFFFF for p in primes]
    rotr = lambda x, n: ((x >> n) | (x << (32 - n))) & 0xFFFFFFFF  # noqa: E731
    w = list(struct.unpack(">16I", block))
    for t in range(16, 18):
        s0 = rotr(w[t - 15], 7) ^ rotr(w[t - 15], 18) ^ (w[t - 15] >> 3)
        s1 = rotr(w[t - 2], 17) ^ rotr(w[t - 2], 19) ^ (w[t - 2] >> 10)
        w.append((s1 + w[t - 7] + s0 + w[t - 16]) & 0xFFFFFFFF)
    state = [0x6A09E667, 0xBB67AE85, 0x3C6EF372, 0xA54FF53A, 0x510E527F, 0x9B05688C, 0x1F83D9AB, 0x5BE0CD19]
    out = [w[16], w[17]]
    for t in range(2):
        a, b, c, d, e, f, g, h = state
        t1 = (h + (rotr(e, 6) ^ rotr(e, 11) ^ rotr(e, 25)) + ((e & f) ^ (~e & g)) + _K256[t] + w[t]) & 0xFFFFFFFF
        t2 = ((rotr(a, 2) ^ rotr(a, 13) ^ rotr(a, 22)) + ((a & b) ^ (a & c) ^ (b & c))) & 0xFFFFFFFF
        state = [(t1 + t2) & 0xFFFFFFFF, a, b, c, (d + t1) & 0xFFFFFFFF, e, f, g]
        out += state
    return struct.pack(">18I", *out)


def library_compute(op: int, args: list[bytes]) -> tuple[str, bytes]:
    """The library oracle's (status, output) for one portable operation."""

    a = args
    if op == 1:
        return "ok", hashlib.sha256(a[0]).digest()
    if op == 2:
        return "ok", _sha256_probe(a[0])
    if op == 3:
        return "ok", hashlib.sha512(a[0]).digest()
    if op == 4:
        return "ok", hmac.new(a[0], a[1], "sha256").digest()
    if op == 5:
        return "ok", _hkdf(a[0], a[1], a[2], struct.unpack("<I", a[3])[0])
    if op in (6, 7):
        data = bytes(64) if op == 6 else a[3]
        out, _, _ = _cipher("chacha20", a[0], a[1] + a[2], data, True)
        return "ok", out
    if op == 8:
        return "ok", _poly1305(a[0], a[1])
    if op == 9:
        out, tag, _ = _cipher("chacha20_poly1305", a[0], a[1], a[3], True, aad=a[2])
        return "ok", out + (tag or b"")
    if op == 10:
        out, _, ok = _cipher("chacha20_poly1305", a[0], a[1], a[3][:-16], False, aad=a[2], tag=a[3][-16:])
        return ("ok", out) if ok else ("rejected", b"")
    if op == 11:
        return "ok", _x25519(a[0], a[1])
    if op == 12:
        out, _, _ = _cipher("aes_ecb", a[0], None, a[1], True)
        return "ok", out
    if op == 13:
        out, tag, _ = _cipher("aes_gcm", a[0], a[1], a[3], True, aad=a[2])
        return "ok", out + (tag or b"")
    if op == 14:
        out, _, ok = _cipher("aes_gcm", a[0], a[1], a[3][:-16], False, aad=a[2], tag=a[3][-16:])
        return ("ok", out) if ok else ("rejected", b"")
    if op == 15:
        return "ok", hashlib.sha3_256(a[0]).digest()
    if op == 16:
        return "ok", hashlib.shake_128(a[0]).digest(struct.unpack("<I", a[1])[0])
    raise ValueError(f"no library oracle for op {op}")


# ---------------------------------------------------------------------------
# Gate arithmetic


def combine(states: Iterable[str]) -> str:
    states = list(states)
    if not states:
        return "unresolved"
    return max(states, key=lambda s: PRECEDENCE[s])


def candidate_gates(tuple_gates: dict[str, dict[str, dict[str, Any]]], members: Iterable[str]) -> dict[str, Any]:
    """Combine the portable path and each member tuple, gate by gate; vacuous entries count only if all are."""

    members = ["C-PORTABLE", *members]
    out = {}
    for gate in GATE_IDS:
        entries = [tuple_gates[m][gate] for m in members]
        live = [e for e in entries if not e.get("vacuous")]
        if not live:
            out[gate] = {"state": "pass", "vacuous": True}
        else:
            out[gate] = {"state": combine(e["state"] for e in live), "vacuous": False}
    return out


def epoch_name(identity: dict[str, Any]) -> str:
    return "d011-e-" + sha256_hex(canonical(identity))[:20]


# ---------------------------------------------------------------------------
# ELF: code sizes, undefined symbols and constant search


def elf_facts(data: bytes) -> dict[str, Any]:
    if data[:4] != b"\x7fELF" or data[4] != 2 or data[5] != 1:
        raise RunError("not a little-endian ELF64 file")
    shoff = struct.unpack_from("<Q", data, 0x28)[0]
    shentsize, shnum, shstrndx = struct.unpack_from("<HHH", data, 0x3A)
    sections = []
    for i in range(shnum):
        name, stype, flags, _addr, offset, size, link, _info, _align, entsize = struct.unpack_from(
            "<IIQQQQIIQQ", data, shoff + i * shentsize)
        sections.append({"name": name, "type": stype, "flags": flags, "offset": offset, "size": size,
                         "link": link, "entsize": entsize})
    strtab = sections[shstrndx]

    def cstr(table: dict[str, Any], index: int) -> str:
        start = table["offset"] + index
        return data[start:data.index(b"\0", start)].decode("ascii", "replace")

    code = rodata = rwdata = 0
    undefined: set[str] = set()
    for sec in sections:
        sec["label"] = cstr(strtab, sec["name"])
        alloc, write, execute = sec["flags"] & 0x2, sec["flags"] & 0x1, sec["flags"] & 0x4
        if alloc and execute:
            code += sec["size"]
        elif alloc and not write:
            rodata += sec["size"]
        elif alloc and write:
            rwdata += sec["size"]
        if sec["type"] == 2 and sec["entsize"] == 24:  # SHT_SYMTAB
            names = sections[sec["link"]]
            for k in range(sec["size"] // 24):
                st_name, st_info, _other, st_shndx, _value, _size = struct.unpack_from(
                    "<IBBHQQ", data, sec["offset"] + 24 * k)
                if st_shndx == 0 and st_name:
                    undefined.add(cstr(names, st_name))
    return {"code_bytes": code, "rodata_bytes": rodata, "rwdata_bytes": rwdata, "undefined": sorted(undefined)}


ALLOWED_IMPORTS = ("memcpy", "memmove", "memset", "memcmp", "_GLOBAL_OFFSET_TABLE_")


def symbol_violations(undefined: Iterable[str]) -> list[str]:
    return [s for s in undefined if s not in ALLOWED_IMPORTS and not s.startswith("d011_")]


K256_PREFIX = struct.pack("<II", 0x428A2F98, 0x71374491)


def corrupt_constants(binary: bytes) -> bytes | None:
    """Flip the lowest bit of the first SHA-256 round constant in a little-endian image."""

    at = binary.find(K256_PREFIX)
    if at < 0:
        return None
    mutated = bytearray(binary)
    mutated[at] ^= 0x01
    return bytes(mutated)


def target_specific_lines(text: str) -> dict[str, int]:
    """Lines inside `#if defined(__x86_64__)` / `#elif ...` target sections, per architecture."""

    openers = {"#if defined(__x86_64__)": "x86_64", "#elif defined(__aarch64__)": "aarch64",
               "#elif defined(__riscv) && __riscv_xlen == 64": "riscv64"}
    counts = {"x86_64": 0, "aarch64": 0, "riscv64": 0}
    current = None
    for line in text.splitlines():
        stripped = line.strip()
        if stripped in openers:
            current = openers[stripped]
            continue
        if current and (stripped.startswith("#elif") or stripped == "#else" or stripped == "#endif /* target */"):
            current = None
            continue
        if current and stripped:
            counts[current] += 1
    return counts


def trace_pc(line: bytes) -> bytes | None:
    """The guest program counter of a QEMU `-d exec` line: `Trace 0: 0x... [cs/pc/flags/cflags] sym`."""

    if not line.startswith(b"Trace "):
        return None
    start = line.find(b"[")
    end = line.find(b"]", start)
    if start < 0 or end < 0:
        return None
    fields = line[start + 1:end].split(b"/")
    return fields[1] if len(fields) >= 2 else None


# ---------------------------------------------------------------------------
# The laboratory: sandboxed execution. Namespaces and privileges follow tools/d004_run.py; the
# sandbox binary is built from the unchanged tools/fs_sandbox.c with the repository's flags.

NAMESPACE = ("/usr/bin/unshare", "--user", "--map-current-user", "--mount", "--ipc", "--uts",
             "--pid", "--fork", "--kill-child=KILL", "--mount-proc", "--net")
PRIVILEGES = ("/usr/bin/setpriv", "--bounding-set=-all", "--inh-caps=-all", "--ambient-caps=-all",
              "--no-new-privs")
INIT = ("/bin/sh", "-c", '"$@"; exit $?', "d011-init")
SANDBOX_SOURCE = "tools/fs_sandbox.c"
SANDBOX_COMPILER = "/usr/bin/cc"
SANDBOX_FLAGS = ("-std=c17", "-O2", "-D_FORTIFY_SOURCE=3", "-fPIE", "-pie", "-Wall", "-Wextra", "-Werror",
                 "-pedantic", "-Wl,-z,relro,-z,now")
LAUNCH_FAILURES = (b"orange filesystem sandbox failed", b"unshare:", b"setpriv:", b"/usr/bin/env:")
COMMAND_EXEC_FAILURE = b"orange filesystem sandbox failed at execute "
BASE_ENV = {"PATH": "/usr/bin:/bin", "LANG": "C", "LC_ALL": "C", "TZ": "UTC", "SOURCE_DATE_EPOCH": "0"}
SYSTEM_RO = ("/usr", "/etc/alternatives", "/etc/ld.so.cache", "/proc/self", "/sys/devices/system/cpu")


def build_sandbox(root: Path, work: Path) -> tuple[Path, dict[str, Any]]:
    source = root / SANDBOX_SOURCE
    binary = work / "fs-sandbox"
    done = subprocess.run([SANDBOX_COMPILER, *SANDBOX_FLAGS, str(source), "-o", str(binary)],
                          env=dict(BASE_ENV), capture_output=True, check=False)
    if done.returncode != 0 or not binary.is_file():
        raise RunError(f"sandbox build failed: {done.stderr.decode('utf-8', 'replace')}")
    return binary, {"source": SANDBOX_SOURCE, "source_sha256": file_sha256(source),
                    "compiler": os.path.realpath(SANDBOX_COMPILER), "flags": list(SANDBOX_FLAGS),
                    "binary_sha256": file_sha256(binary)}


def exit_facts(code: int) -> dict[str, Any]:
    number = 0
    if code < 0:
        number = -code
    elif 128 < code < 128 + 65:
        number = code - 128
    if number:
        try:
            name = signal.Signals(number).name
        except ValueError:
            name = f"SIG{number}"
        return {"exit_code": None, "signal": name}
    return {"exit_code": code, "signal": None}


def run_state(code: int, timed_out: bool, oversized: bool) -> str:
    """How a step ended. Only `completed` (exit status 0, no limit reached) is a success; an expected
    failure is matched on `failed` with its exit code or `crash` with its signal, never on a limit."""

    facts = exit_facts(code)
    if timed_out:
        return "timeout"
    if oversized:
        return "oversized_output"
    if facts["signal"] in ("SIGXCPU", "SIGXFSZ"):
        return "resource_exhaustion"
    if facts["exit_code"] == 0:
        return "completed"
    if facts["exit_code"] is not None:
        return "failed"
    return "crash"


class Launcher:
    """Run one process tree in fresh namespaces, without capabilities, inside fs-sandbox."""

    def __init__(self, sandbox: Path) -> None:
        self.sandbox = sandbox

    def argv(self, command: list[str], ro: Iterable[str], rw: Iterable[str], env: dict[str, str]) -> list[str]:
        rules = ["--dir", "/"]
        for path in [*SYSTEM_RO, *ro]:
            if os.path.exists(path):
                rules += ["--ro", path]
        for path in [*rw, "/dev/null"]:
            rules += ["--rw", path]
        return [*NAMESPACE, *PRIVILEGES, *INIT, "/usr/bin/env", "-i",
                *(f"{k}={v}" for k, v in sorted(env.items())), str(self.sandbox), *rules, "--", *command]

    def run(self, command: list[str], *, ro: Iterable[str] = (), rw: Iterable[str] = (), stdin: bytes = b"",
            env: dict[str, str] | None = None, cwd: Path | None = None, wall_seconds: int = 600,
            output_cap: int = 64 << 20, trace: bool = False) -> dict[str, Any]:
        environment = dict(BASE_ENV, **(env or {}))
        started = time.perf_counter_ns()
        process = subprocess.Popen(self.argv(command, ro, rw, environment), cwd=cwd, env={"PATH": "/usr/bin:/bin"},
                                   stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        out, err = bytearray(), bytearray()
        pending = memoryview(stdin)
        partial = b""
        digest = hashlib.sha256()
        blocks = 0
        timed_out = oversized = False
        deadline = time.monotonic() + wall_seconds
        with selectors.DefaultSelector() as selector:
            if pending:
                os.set_blocking(process.stdin.fileno(), False)
                selector.register(process.stdin, selectors.EVENT_WRITE, "stdin")
            else:
                process.stdin.close()
            selector.register(process.stdout, selectors.EVENT_READ, "stdout")
            selector.register(process.stderr, selectors.EVENT_READ, "stderr")
            while any(key.data != "stdin" for key in selector.get_map().values()):
                remaining = deadline - time.monotonic()
                if remaining <= 0:
                    timed_out = True
                    break
                for key, _ in selector.select(remaining):
                    if key.data == "stdin":
                        try:
                            written = os.write(key.fd, pending[:65536])
                        except BrokenPipeError:
                            written = len(pending)
                        pending = pending[written:]
                        if not pending:
                            selector.unregister(key.fileobj)
                            process.stdin.close()
                        continue
                    chunk = os.read(key.fd, 1 << 16)
                    if not chunk:
                        selector.unregister(key.fileobj)
                        if key.data == "stdin":
                            process.stdin.close()
                        continue
                    if key.data == "stdout":
                        out += chunk
                    elif trace:
                        lines = (partial + chunk).split(b"\n")
                        partial = lines.pop()
                        for line in lines:
                            pc = trace_pc(line)
                            if pc is None:
                                if len(err) < 65536:
                                    err += line + b"\n"
                            else:
                                digest.update(pc + b"\n")
                                blocks += 1
                    else:
                        err += chunk
                    if len(out) + len(err) > output_cap:
                        oversized = True
                        break
                if oversized:
                    break
            for key in list(selector.get_map().values()):
                if key.data == "stdin":
                    selector.unregister(key.fileobj)
                    process.stdin.close()
        if timed_out or oversized:
            process.kill()
        _, status, usage = os.wait4(process.pid, 0)
        process.stdout.close()
        process.stderr.close()
        wall = (time.perf_counter_ns() - started) // 1000
        code = os.waitstatus_to_exitcode(status)
        if bytes(err).startswith(LAUNCH_FAILURES) and not bytes(err).startswith(COMMAND_EXEC_FAILURE):
            raise RunError(f"launcher failure: {bytes(err)[:300].decode('utf-8', 'replace')}")
        state = run_state(code, timed_out, oversized)
        facts = exit_facts(code)
        if state in ("timeout", "oversized_output"):
            # A step stopped by a limit keeps neither its exit status nor its signal.
            code, facts = None, {"exit_code": None, "signal": None}
        result = {"exit": code, **facts, "state": state, "stdout": bytes(out), "stderr": bytes(err), "wall_us": wall,
                  "cpu_us": int((usage.ru_utime + usage.ru_stime) * 1_000_000), "max_rss_kib": int(usage.ru_maxrss),
                  "timed_out": timed_out, "oversized": oversized}
        if trace:
            result["trace"] = {"sha256": digest.hexdigest(), "blocks": blocks}
        return result


# ---------------------------------------------------------------------------
# Host and tool capture


def host_capture() -> dict[str, Any]:
    cpu_model, flags = "", set()
    try:
        for line in Path("/proc/cpuinfo").read_text(encoding="utf-8", errors="replace").splitlines():
            if line.startswith(("model name", "Model")) and not cpu_model:
                cpu_model = line.split(":", 1)[1].strip()
            if line.startswith(("flags", "Features")) and not flags:
                flags = set(line.split(":", 1)[1].split())
    except OSError:
        pass
    os_release = {}
    try:
        for line in Path("/etc/os-release").read_text(encoding="utf-8").splitlines():
            if "=" in line:
                key, value = line.split("=", 1)
                os_release[key] = value.strip('"')
    except OSError:
        pass
    wanted = ("aes", "pclmulqdq", "avx2", "sha_ni", "vaes", "vpclmulqdq", "pmull", "sha2", "asimd")
    return {"role": "contributor laboratory host; not owner hardware", "machine": platform.machine(),
            "arch": {"x86_64": "x86_64", "aarch64": "aarch64", "riscv64": "riscv64"}.get(platform.machine(), ""),
            "kernel": platform.release(), "os": os_release.get("PRETTY_NAME", ""), "cpu_model": cpu_model,
            "cpu_flags": sorted(f for f in flags if f in wanted), "cpus": os.cpu_count() or 0,
            "python": sys.version.split()[0]}


def _version(launcher: Launcher, argv: list[str], ro: Iterable[str] = ()) -> str:
    """The first line a tool prints about its version, from a run inside the sandbox like every other tool
    execution."""

    run = launcher.run(argv, ro=ro, wall_seconds=60)
    if run["state"] != "completed":
        return "unavailable"
    text = (run["stdout"] or run["stderr"]).decode("utf-8", "replace").strip().splitlines()
    return text[0] if text else "unavailable"


RUSTC_TOOLCHAIN = "1.96.1-x86_64-unknown-linux-gnu"


def find_rustc() -> Path | None:
    """The pinned rustup toolchain in the current user's home, found from the password database."""

    try:
        home = Path(pwd.getpwuid(os.getuid()).pw_dir)
    except KeyError:
        return None
    candidate = home / ".rustup" / "toolchains" / RUSTC_TOOLCHAIN / "bin" / "rustc"
    return candidate if candidate.is_file() else None


GCC_PROGRAM_ROOTS = ("/usr/libexec/gcc", "/usr/libexec/gcc-cross", "/usr/lib/gcc", "/usr/lib/gcc-cross")


def gcc_program(reported: str) -> str | None:
    """The installed GCC program a driver reports, as one of the files that already exist under GCC's program
    directories. The driver's output is only compared with those entries, so it never becomes a path."""

    for base in GCC_PROGRAM_ROOTS:
        root = Path(base)
        for candidate in sorted(root.glob("*/*/*")) if root.is_dir() else ():
            if str(candidate) == reported and candidate.is_file():
                return str(candidate)
    return None


def tool_capture(launcher: Launcher, orangec: Path | None, rustc: Path | None) -> dict[str, Any]:
    tools: dict[str, Any] = {}

    def add(key: str, path: str, argv: list[str] | None, ro: Iterable[str] = ()) -> None:
        real = os.path.realpath(path)
        if not os.path.isfile(real):
            tools[key] = {"path": path, "present": False}
            return
        tools[key] = {"path": path, "realpath": real, "sha256": file_sha256(Path(real)), "present": True,
                      "version": _version(launcher, argv, ro) if argv else ""}

    for tc in TOOLCHAINS:
        if tc["kind"] in ("gcc", "clang"):
            add(tc["id"], tc["path"], [tc["path"], "--version"])
        if tc["kind"] == "gcc" and os.path.isfile(tc["path"]):
            # The driver's own programs: the compiler proper, the assembler and the linker.
            prefix = tc["path"].rsplit("-gcc", 1)[0]
            cc1 = gcc_program(_version(launcher, [tc["path"], "-print-prog-name=cc1"]))
            if cc1 is not None:
                add(f"{tc['id']}/cc1", cc1, None)
            add(f"{tc['id']}/as", f"{prefix}-as", [f"{prefix}-as", "--version"])
            add(f"{tc['id']}/ld", f"{prefix}-ld", [f"{prefix}-ld", "--version"])
    add("TC-04/lld", "/usr/bin/ld.lld-18" if os.path.exists("/usr/bin/ld.lld-18") else "/usr/bin/ld.lld",
        ["/usr/bin/ld.lld", "--version"])
    add("TC-05/llvm-objdump", "/usr/bin/llvm-objdump-18", ["/usr/bin/llvm-objdump-18", "--version"])
    add("TC-05/llvm-readelf", "/usr/bin/llvm-readelf-18", ["/usr/bin/llvm-readelf-18", "--version"])
    for tup in TUPLES:
        add(f"TC-06/{tup['arch']}", tup["emulator"], [tup["emulator"], "--version"])
    if rustc is not None:
        add("TC-07", str(rustc), [str(rustc), "--version"], ro=[str(rustc.parents[1])])
        # rustc links the probe with the system C driver.
        add("TC-07/linker", "/usr/bin/cc", ["/usr/bin/cc", "--version"])
    else:
        tools["TC-07"] = {"path": "rustc", "present": False}
    tools["TC-08"] = {"path": sys.executable, "realpath": os.path.realpath(sys.executable),
                      "sha256": file_sha256(Path(os.path.realpath(sys.executable))), "present": True,
                      "version": sys.version.split()[0]}
    libcrypto = next((p for p in ("/usr/lib/x86_64-linux-gnu/libcrypto.so.3", "/usr/lib/aarch64-linux-gnu/libcrypto.so.3",
                                  "/usr/lib/riscv64-linux-gnu/libcrypto.so.3") if os.path.exists(p)), None)
    tools["TC-09"] = {"path": libcrypto or "libcrypto.so.3", "present": libcrypto is not None,
                      "sha256": file_sha256(Path(os.path.realpath(libcrypto))) if libcrypto else "",
                      "version": openssl_version()}
    if orangec is not None and orangec.is_file():
        add("TC-10", str(orangec), [str(orangec), "--version"], ro=[str(orangec)])
    else:
        tools["TC-10"] = {"path": str(orangec), "present": False}
    add("TC-12/unshare", "/usr/bin/unshare", ["/usr/bin/unshare", "--version"])
    add("TC-12/setpriv", "/usr/bin/setpriv", ["/usr/bin/setpriv", "--version"])
    return tools


# ---------------------------------------------------------------------------
# The epoch


def median(values: list[int]) -> int:
    ordered = sorted(values)
    if not ordered:
        return 0
    mid = len(ordered) // 2
    return ordered[mid] if len(ordered) % 2 else (ordered[mid - 1] + ordered[mid]) // 2


def build_id(tup: str, tc: str, profile: str, opt: str, rep: int) -> str:
    return f"{tup}/{tc}/{profile}/{opt}/r{rep}"


def ensure_archive_root() -> None:
    """Create ARCHIVE_ROOT private to this user, and refuse one that is a link or someone else's."""

    ARCHIVE_ROOT.mkdir(mode=0o700, exist_ok=True)
    info = os.lstat(ARCHIVE_ROOT)
    if ARCHIVE_ROOT.is_symlink() or not ARCHIVE_ROOT.is_dir() or info.st_uid != os.getuid():
        raise RunError(f"{ARCHIVE_ROOT} is not a directory owned by this user")
    OWNER_INPUT_DIR.mkdir(mode=0o700, exist_ok=True)


def existing_entry(name: str, root: Path, directories: bool) -> Path:
    """The entry under root that an argument names, by name or path. The argument is only compared with
    entries that already exist there (as D-004's and D-006's runners do), so it never becomes part of a
    filesystem path."""

    wanted = os.path.realpath(name)
    for child in sorted(root.iterdir()) if root.is_dir() else ():
        if child.is_symlink() or not (child.is_dir() if directories else child.is_file()):
            continue
        if child.name == name or os.path.realpath(child) == wanted:
            return child
    raise RunError(f"no {'epoch' if directories else 'file'} named {name!r} under {root}")


class Lab:
    def __init__(self, root: Path, profile: str, owner_input: Path | None) -> None:
        self.root = root
        self.profile_name = profile
        self.profile = RUN_PROFILES[profile]
        self.archive_root = ARCHIVE_ROOT
        self.work = WORK_DIR
        self.orangec = ORANGEC
        self.owner_input = owner_input
        self.packet_bytes = (root / PACKET_PATH).read_bytes()
        self.packet = json.loads(self.packet_bytes.decode("utf-8"))
        self.index: list[dict[str, str]] = []
        self.records: list[dict[str, Any]] = []
        self.builds: dict[str, dict[str, Any]] = {}
        self.host = host_capture()
        self.rustc = find_rustc()

    # -- records

    def record(self, stage: str, key: str, data: dict[str, Any]) -> None:
        body = {"schema": RECORD_SCHEMA, "epoch": self.name, "label": LABEL, "stage": stage, "key": key, **data}
        raw = canonical_file(gate0_numbers(body))
        digest = sha256_hex(raw)
        (self.archive / "records" / f"{digest}.json").write_bytes(raw)
        self.index.append({"stage": stage, "key": key, "sha256": digest})
        # The summary is computed from exactly what was recorded, as verify recomputes it.
        self.records.append(json.loads(raw.decode("utf-8")))
        print(f"  {stage:<12} {key}", flush=True)

    # -- preparation

    def prepare(self) -> None:
        problems = check(self.root)
        if problems:
            raise RunError("packet check failed: " + "; ".join(problems[:5]))
        ensure_archive_root()
        self.work.mkdir(mode=0o700, exist_ok=True)
        self.src = self.work / "src"
        if self.src.exists():
            shutil.rmtree(self.src)
        self.src.mkdir()
        for row in self.packet["inputs"]:
            if row["path"].startswith(KERNEL_DIR):
                target = self.src / Path(row["path"]).name
                shutil.copyfile(self.root / row["path"], target)
                if file_sha256(target) != row["sha256"]:
                    raise RunError(f"{row['path']} changed while copying")
        sandbox, sandbox_facts = build_sandbox(self.root, self.work)
        self.launcher = Launcher(sandbox)
        self.tools = tool_capture(self.launcher, self.orangec, self.rustc)
        self.tools["TC-11"] = sandbox_facts
        head, dirty = "", "unknown"
        try:
            found = subprocess.run(["/usr/bin/git", "-C", str(self.root), "rev-parse", "HEAD"], capture_output=True,
                                   text=True, check=False, env=dict(BASE_ENV))
            status = subprocess.run(["/usr/bin/git", "-C", str(self.root), "status", "--porcelain"],
                                    capture_output=True, text=True, check=False, env=dict(BASE_ENV))
            if found.returncode == 0 and status.returncode == 0:
                head, dirty = found.stdout.strip(), status.stdout.strip()
        except OSError:
            pass
        self.identity = {"suite_version": SUITE_VERSION, "packet_sha256": sha256_hex(self.packet_bytes),
                         "profile": self.profile_name, "run_profile": self.profile,
                         "base_revision": self.packet["base_revision"], "repository_head": head,
                         "working_tree_clean": not dirty, "tools": self.tools, "host": self.host,
                         "started_utc": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime())}
        self.identity = gate0_numbers(self.identity)
        self.name = epoch_name(self.identity)
        self.archive = self.archive_root / self.name
        if self.archive.exists():
            raise RunError(f"{self.archive} exists")
        (self.archive / "records").mkdir(parents=True)
        (self.archive / "products").mkdir()
        (self.archive / "packet.json").write_bytes(self.packet_bytes)
        (self.archive / "epoch.json").write_bytes(canonical_file(gate0_numbers(
            {"schema": RECORD_SCHEMA, "epoch": self.name, "label": LABEL, "identity": self.identity,
             "statement": self.packet["stand_in_statement"]})))
        print(f"epoch {self.name} ({self.profile_name})", flush=True)
        self.record("host", "host", {"host": self.host})
        self.record("tools", "tools", {"tools": self.tools})
        self.record("source_lines", "target-specific", {
            "accel": target_specific_lines((self.src / "accel.c").read_text(encoding="utf-8")),
            "runtime": target_specific_lines((self.src / "runtime.c").read_text(encoding="utf-8"))})

    # -- NT-01 precondition: oracles

    def oracles(self) -> None:
        values: dict[str, dict[str, tuple[str, str]]] = {}
        for src in self.packet["oracle_sources"]:
            path = self.root / src["path"]
            found = file_sha256(path) if path.is_file() else ""
            row: dict[str, Any] = {"source": src["id"], "path": src["path"], "sha256_expected": src["sha256"],
                                   "sha256_found": found, "evaluated": False}
            if found == src["sha256"] and self.orangec is not None and self.orangec.is_file():
                run = self.launcher.run([str(self.orangec), "eval", str(path)], ro=[str(path), str(self.orangec)],
                                        wall_seconds=300)
                row.update({"exit": run["exit"], "run_state": run["state"], "wall_us": run["wall_us"],
                            "stderr": run["stderr"][:2000].decode("utf-8", "replace")})
                if run["state"] == "completed" and not run["stderr"].strip():
                    values[src["id"]] = parse_eval(run["stdout"].decode("utf-8"))
                    row["evaluated"] = True
                    row["stdout_sha256"] = sha256_hex(run["stdout"])
            self.record("oracle_source", src["id"], row)
        carriers = []
        for car in self.packet["published_carriers"]:
            path = self.root / car["path"]
            text = path.read_text(encoding="utf-8") if path.is_file() else ""
            carriers.append({"id": car["id"], "sha256_found": sha256_hex(text.encode("utf-8")),
                             "sha256_expected": car["sha256"]})
        carrier_ok = all(c["sha256_found"] == c["sha256_expected"] for c in carriers)
        probe_text = (self.root / self.packet["published_carriers"][0]["path"]).read_text(encoding="utf-8")
        for subj in self.packet["subjects"]:
            row = {"subject": subj["id"], "provenance": subj["provenance"]["kind"]}
            src = subj["orange"]["source"]
            row["orange"] = orange_verdict(subj, values[src]) if src in values else {"computed": "missing"}
            status, output = library_compute(subj["op"], [bytes.fromhex(a) for a in subj["args"]])
            output = apply_slices(output, subj["driver_slices"])
            row["library"] = "agree" if (status == subj["expect"]["status"] and output.hex() ==
                                         subj["expect"]["output"]) else "conflict"
            if not subj["orange"]["literal"]:
                words = [subj["expect"]["output"][i:i + 8] for i in range(0, len(subj["expect"]["output"]), 8)]
                row["published_carrier"] = "agree" if carrier_ok and all(f"0x{w}" in probe_text for w in words) \
                    else "conflict"
            verdicts = [v for v in row["orange"].values() if v is not None] + [row["library"]]
            if "published_carrier" in row:
                verdicts.append(row["published_carrier"])
            row["state"] = "conflict" if "conflict" in verdicts else ("missing" if "missing" in verdicts else "agree")
            independent = 1 + (1 if row["orange"].get("computed") == "agree" else 0)
            if subj["provenance"]["kind"] != "library_produced":
                independent += 1
            row["independent_oracles"] = independent
            self.record("oracle", subj["id"], row)
        for neg in self.packet["negatives"]:
            spec = (neg.get("orange") or {}).get("status_spec")
            if spec:
                src = neg["orange"]["source"]
                verdict = value_bytes(*values[src][spec]) if src in values and spec in values[src] else None
                self.record("oracle", neg["id"], {"subject": neg["id"], "orange_status": verdict,
                                                  "state": "agree" if verdict is False else "conflict"})

    # -- builds

    def toolchain_kind(self, tc: str) -> str:
        return TOOLCHAIN_BY_ID[tc]["kind"]

    def compile_argv(self, tup: str, tc: str, profile: str, opt: str) -> list[str]:
        kind = self.toolchain_kind(tc)
        flags = BUILD_FLAGS["targets"][tup]
        argv = [TOOLCHAIN_BY_ID[tc]["path"], *flags[kind], *flags[profile], f"-{opt}", *BUILD_FLAGS["common"]]
        if kind == "gcc":
            argv += BUILD_FLAGS["gcc_extra"]
        if profile == "P-CRYPTO":
            argv.append("-DD011_CRYPTO_PROFILE")
        return argv

    def link_argv(self, tup: str, tc: str, profile: str) -> list[str]:
        kind = self.toolchain_kind(tc)
        flags = BUILD_FLAGS["targets"][tup]
        return [TOOLCHAIN_BY_ID[tc]["path"], *flags[kind], *flags[profile], *BUILD_FLAGS["link"][kind],
                *flags["link_extra"]]

    def run_build(self, tup: str, tc: str, profile: str, opt: str, rep: int) -> dict[str, Any]:
        bid = build_id(tup, tc, profile, opt, rep)
        out = self.work / "builds" / bid.replace("/", "_")
        if out.exists():
            shutil.rmtree(out)
        (out / "tmp").mkdir(parents=True)
        parts = [("kernels.c", p, "k") for p in KERNEL_PARTS]
        if profile == "P-CRYPTO":
            parts += [("accel.c", p, "a") for p in ACCEL_PARTS]
        parts += [("runtime.c", p, "r") for p in RUNTIME_PARTS]
        objects = []
        state = "built"
        base = self.compile_argv(tup, tc, profile, opt)
        for source, part, prefix in parts:
            obj = out / f"{prefix}_{part}.o"
            run = self.launcher.run([*base, f"-DD011_PART_{part}", "-c", source, "-o", str(obj)],
                                    ro=[str(self.src)], rw=[str(out)], cwd=self.src, env={"TMPDIR": str(out / "tmp")},
                                    wall_seconds=600)
            row = {"part": part, "source": source, "exit": run["exit"], "run_state": run["state"],
                   "cpu_us": run["cpu_us"], "wall_us": run["wall_us"],
                   "diagnostics": run["stderr"][:4000].decode("utf-8", "replace")}
            if run["state"] == "completed" and obj.is_file():
                data = obj.read_bytes()
                facts = elf_facts(data)
                row.update({"sha256": sha256_hex(data), "bytes": len(data), "code_bytes": facts["code_bytes"],
                            "rodata_bytes": facts["rodata_bytes"], "undefined": facts["undefined"]})
            else:
                state = "failed"
            objects.append(row)
        link: dict[str, Any] = {}
        driver = out / "driver"
        if state == "built":
            run = self.launcher.run([*self.link_argv(tup, tc, profile),
                                     *[f"{prefix}_{part}.o" for _source, part, prefix in parts], "-o", str(driver)],
                                    ro=[str(self.src)], rw=[str(out)], cwd=out, env={"TMPDIR": str(out / "tmp")},
                                    wall_seconds=600)
            link = {"exit": run["exit"], "run_state": run["state"], "cpu_us": run["cpu_us"], "wall_us": run["wall_us"],
                    "diagnostics": run["stderr"][:4000].decode("utf-8", "replace")}
            if run["state"] == "completed" and driver.is_file():
                data = driver.read_bytes()
                link.update({"sha256": sha256_hex(data), "bytes": len(data), "code_bytes": elf_facts(data)["code_bytes"]})
                if rep == 1:
                    shutil.copyfile(driver, self.archive / "products" / f"{sha256_hex(data)}.elf")
            else:
                state = "failed"
        info = {"id": bid, "tuple": tup, "toolchain": tc, "profile": profile, "opt": opt, "rep": rep,
                "state": state, "dir": out, "driver": driver}
        self.builds[bid] = info
        self.record("build", bid, {"tuple": tup, "toolchain": tc, "profile": profile, "opt": opt, "rep": rep,
                                   "state": state, "compile_argv": base, "link_argv": self.link_argv(tup, tc, profile),
                                   "objects": objects, "link": link})
        return info

    def build_all(self) -> None:
        for tup in TUPLES:
            for tc in tup["toolchains"]:
                present = os.path.isfile(TOOLCHAIN_BY_ID[tc]["path"])
                for profile in PROFILE_IDS:
                    for opt in self.profile["optimizations"]:
                        for rep in range(1, self.profile["build_repetitions"] + 1):
                            if present:
                                self.run_build(tup["id"], tc, profile, opt, rep)
                            elif rep == 1:
                                bid = build_id(tup["id"], tc, profile, opt, rep)
                                self.record("build", bid, {"tuple": tup["id"], "toolchain": tc, "profile": profile,
                                                           "opt": opt, "rep": rep, "state": "unavailable",
                                                           "reason": TOOLCHAIN_BY_ID[tc]["acquisition"]})

    def primary_builds(self) -> list[dict[str, Any]]:
        return [b for b in self.builds.values() if b["rep"] == 1 and b["state"] == "built"]

    # -- execution

    def modes(self, build: dict[str, Any]) -> list[str]:
        tup = TUPLE_BY_ID[build["tuple"]]
        modes = ["emulated"]
        if tup["arch"] == self.host["arch"]:
            needs = {"P-BASE": set(), "P-CRYPTO": {"aes", "pclmulqdq"} if tup["arch"] == "x86_64" else {"aes", "pmull"}}
            if needs[build["profile"]] <= set(self.host["cpu_flags"]):
                modes.append("native")
        return modes

    def execute(self, binary: Path, tup: str, mode: str, stdin: bytes, cpu: str | None = None,
                trace: bool = False, wall_seconds: int = 600) -> dict[str, Any]:
        info = TUPLE_BY_ID[tup]
        if mode == "native":
            command = [str(binary)]
            ro = [str(binary)]
        else:
            command = [info["emulator"], "-cpu", cpu or info["emulator_cpu"]]
            if trace:
                command += ["-d", "nochain,exec"]
            command.append(str(binary))
            ro = [str(binary)]
        return self.launcher.run(command, ro=ro, stdin=stdin, wall_seconds=wall_seconds, trace=trace)

    def kat_requests(self, profile: str) -> list[tuple[str, int, list[bytes]]]:
        requests = []
        for subj in self.packet["subjects"]:
            args = [bytes.fromhex(a) for a in subj["args"]]
            requests.append((subj["id"], subj["op"], args))
            if profile == "P-CRYPTO" and subj["accel_op"]:
                requests.append((subj["id"] + "+accel", subj["accel_op"], args))
        return requests

    def compare(self, requests: list[tuple[str, int, list[bytes]]], responses: list[tuple[str, bytes]]
                ) -> list[dict[str, Any]]:
        by_id = {s["id"]: s for s in self.packet["subjects"]}
        rows = []
        for i, (rid, op, _args) in enumerate(requests):
            subj = by_id[rid.split("+")[0]]
            if i >= len(responses):
                rows.append({"subject": rid, "op": op, "status": "missing", "match": False})
                continue
            status, output = responses[i]
            output = apply_slices(output, subj["driver_slices"])
            match = status == subj["expect"]["status"] and output.hex() == subj["expect"]["output"]
            row = {"subject": rid, "op": op, "status": status, "match": match}
            if not match:
                row["output"] = output.hex()[:512]
            rows.append(row)
        return rows

    def kat(self) -> None:
        for profile in PROFILE_IDS:
            batch = b"".join(encode_request(op, args) for _rid, op, args in self.kat_requests(profile))
            (self.archive / "products" / f"kat-{profile}.bin").write_bytes(batch)
        for build in self.primary_builds():
            requests = self.kat_requests(build["profile"])
            stdin = b"".join(encode_request(op, args) for _rid, op, args in requests)
            outputs = {}
            for mode in self.modes(build):
                run = self.execute(build["driver"], build["tuple"], mode, stdin)
                responses, whole = parse_responses(run["stdout"])
                rows = self.compare(requests, responses)
                runs = self.profile["native_timed_runs"] if mode == "native" else self.profile["emulated_timed_runs"]
                warm = self.profile["native_warmups"] if mode == "native" else 0
                repeats = self.profile["timing_batch_repeats"]
                timings, overhead = [], []
                for i in range(warm + runs):
                    timed = self.execute(build["driver"], build["tuple"], mode, stdin * repeats)
                    empty = self.execute(build["driver"], build["tuple"], mode, b"")
                    if i >= warm:
                        timings.append({"wall_us": timed["wall_us"], "cpu_us": timed["cpu_us"],
                                        "run_state": timed["state"],
                                        "same_output": timed["stdout"] == run["stdout"] * repeats})
                        overhead.append({"wall_us": empty["wall_us"], "cpu_us": empty["cpu_us"],
                                         "run_state": empty["state"]})
                outputs[mode] = sha256_hex(run["stdout"])
                self.record("kat", f"{build['id']}/{mode}", {
                    "build": build["id"], "tuple": build["tuple"], "toolchain": build["toolchain"],
                    "profile": build["profile"], "opt": build["opt"], "mode": mode, "exit": run["exit"],
                    "signal": run["signal"], "run_state": run["state"],
                    "complete": whole and len(responses) == len(requests) and run["state"] == "completed",
                    "timings_consistent": all(t["run_state"] == "completed" and t["same_output"] for t in timings)
                    and all(o["run_state"] == "completed" for o in overhead),
                    "stdout_sha256": sha256_hex(run["stdout"]), "results": rows,
                    "matched": sum(1 for r in rows if r["match"]), "total": len(rows),
                    "timing_kind": mode, "timing_batch_repeats": repeats, "timings": timings,
                    "launch_overhead": overhead,
                    "timing_median_wall_us": median([t["wall_us"] for t in timings]),
                    "timing_median_cpu_us": median([t["cpu_us"] for t in timings]),
                    "launch_overhead_median_wall_us": median([t["wall_us"] for t in overhead])})
            if len(outputs) == 2:
                self.record("native_emulated", build["id"], {"build": build["id"], "tuple": build["tuple"],
                                                             "identical": outputs["native"] == outputs["emulated"]})

    # -- NT-04

    def negatives(self) -> None:
        for build in self.primary_builds():
            tup = TUPLE_BY_ID[build["tuple"]]
            for neg in self.packet["negatives"]:
                if build["profile"] not in neg["profiles"]:
                    continue
                key = f"{build['id']}/{neg['id']}"
                base = {"build": build["id"], "tuple": build["tuple"], "profile": build["profile"],
                        "negative": neg["id"], "kind": neg["kind"]}
                if neg["kind"] == "request":
                    stdin = encode_request(neg["op"], [bytes.fromhex(a) for a in neg["args"]])
                    rows = []
                    for mode in self.modes(build):
                        run = self.execute(build["driver"], build["tuple"], mode, stdin, wall_seconds=120)
                        responses, whole = parse_responses(run["stdout"])
                        ok = (run["state"] == "completed" and whole and len(responses) == 1
                              and responses[0][0] == neg["expect"]
                              and (neg["expect"] == "ok" or responses[0][1] == b""))
                        rows.append({"mode": mode, "exit": run["exit"], "run_state": run["state"],
                                     "responses": [r[0] for r in responses],
                                     "pass": ok})
                    self.record("negative", key, {**base, "modes": rows,
                                                  "state": "pass" if all(r["pass"] for r in rows) else "fail"})
                elif neg["kind"] == "truncated":
                    stdin = encode_request(1, [b"abc"]) + encode_request(1, [b"x" * 100])[:20]
                    run = self.execute(build["driver"], build["tuple"], "emulated", stdin, wall_seconds=120)
                    responses, whole = parse_responses(run["stdout"])
                    ok = (run["state"] == "failed" and run["exit_code"] == 2 and whole and len(responses) == 1
                          and responses[0][0] == "ok")
                    self.record("negative", key, {**base, "exit": run["exit"], "run_state": run["state"],
                                                  "responses": len(responses),
                                                  "state": "pass" if ok else "fail"})
                elif neg["kind"] == "feature":
                    (self.archive / "products" / "feature-negative.bin").write_bytes(
                        encode_request(17, [bytes(16), bytes(16)]))
                    cpu = tup["feature_negative_cpu"]
                    if cpu is None:
                        self.record("negative", key, {**base, "state": "unresolved",
                                                      "reason": "the emulator has no CPU model of this tuple without "
                                                                "the extension (gap G-09)"})
                        continue
                    stdin = encode_request(17, [bytes(16), bytes(16)])
                    run = self.execute(build["driver"], build["tuple"], "emulated", stdin, cpu=cpu, wall_seconds=120)
                    responses, _ = parse_responses(run["stdout"])
                    ok = run["state"] == "crash" and run["signal"] == "SIGILL" and not responses
                    self.record("negative", key, {**base, "cpu": cpu, "exit": run["exit"], "signal": run["signal"],
                                                  "run_state": run["state"],
                                                  "responses": len(responses), "state": "pass" if ok else "fail"})
                elif neg["kind"] == "corruption":
                    image = build["driver"].read_bytes()
                    mutated = corrupt_constants(image)
                    if mutated is None:
                        self.record("negative", key, {**base, "state": "unresolved",
                                                      "reason": "the SHA-256 round constants are not stored as a table "
                                                                "in this image"})
                        continue
                    target = build["dir"] / "driver-corrupted"
                    target.write_bytes(mutated)
                    target.chmod(0o700)
                    sha = [s for s in self.packet["subjects"] if s["op"] == 1]
                    requests = [(s["id"], 1, [bytes.fromhex(a) for a in s["args"]]) for s in sha]
                    stdin = b"".join(encode_request(op, args) for _rid, op, args in requests)
                    run = self.execute(target, build["tuple"], "emulated", stdin, wall_seconds=120)
                    rows = self.compare(requests, parse_responses(run["stdout"])[0])
                    # Caught by the known answers: the corrupted driver completes and answers wrongly.
                    detected = run["state"] == "completed" and any(not r["match"] for r in rows)
                    self.record("negative", key, {**base, "run_state": run["state"],
                                                  "mismatches": sum(1 for r in rows if not r["match"]),
                                                  "state": "pass" if detected else "fail"})

    # -- NT-02 and the symbol surface of NT-05

    def inventory(self) -> None:
        objdump = "/usr/bin/llvm-objdump-18"
        readelf = "/usr/bin/llvm-readelf-18"
        for build in self.primary_builds():
            arch = TUPLE_BY_ID[build["tuple"]]["arch"]
            objects = {}
            for obj in sorted(build["dir"].glob("*.o")):
                part = obj.stem.split("_", 1)[1]
                if obj.stem.startswith("r_") and part != "CONTROL":
                    continue
                argv = [objdump, "-d", "--no-show-raw-insn", "--no-leading-addr"]
                if arch == "x86_64":
                    argv.append("--x86-asm-syntax=intel")
                run = self.launcher.run([*argv, str(obj)], ro=[str(obj)], wall_seconds=300)
                functions = parse_objdump(run["stdout"].decode("utf-8", "replace"))
                classes = {c: 0 for c in INSTRUCTION_CLASSES}
                mnemonics: set[str] = set()
                class_a = []
                for symbol, instructions in functions.items():
                    for (mnemonic, operands), cls in zip(instructions, classify_sequence(arch, instructions)):
                        mnemonics.add(mnemonic.lower())
                        if cls:
                            classes[cls] += 1
                        if cls == "A":
                            class_a.append(f"{symbol}: {mnemonic} {operands}")
                relocs = self.launcher.run([readelf, "-r", str(obj)], ro=[str(obj)], wall_seconds=300)
                types = sorted(set(re.findall(r"\bR_[A-Z0-9_]+\b", relocs["stdout"].decode("utf-8", "replace"))))
                facts = elf_facts(obj.read_bytes())
                objects[obj.stem] = {"part": part, "family": FAMILY_OF_PART.get(part, "control"),
                                     "role": "control" if part == "CONTROL" else "kernel",
                                     "classes": classes, "mnemonics": sorted(mnemonics), "class_a": class_a[:50],
                                     "relocation_types": types, "undefined": facts["undefined"],
                                     "symbol_violations": symbol_violations(facts["undefined"]),
                                     "code_bytes": facts["code_bytes"], "instructions": sum(
                                         len(v) for v in functions.values()), "objdump_exit": run["exit"],
                                     "objdump_state": run["state"], "readelf_state": relocs["state"]}
            control = objects.get("r_CONTROL", {}).get("classes", {}).get("A", 0)
            completed = bool(objects) and all(o["objdump_state"] == "completed" and o["readelf_state"] == "completed"
                                              for o in objects.values())
            self.record("inventory", build["id"], {"build": build["id"], "tuple": build["tuple"],
                                                   "toolchain": build["toolchain"], "profile": build["profile"],
                                                   "opt": build["opt"], "objects": objects, "completed": completed,
                                                   "control_detected": control > 0})

    # -- NT-03

    def group_request(self, group: dict[str, Any], variant: int) -> bytes:
        args: list[bytes] = []
        n = len(group["args"])
        for i, arg in enumerate(group["args"]):
            if arg["kind"] == "public":
                args.append(bytes.fromhex(arg["value"]))
            elif arg["kind"] == "secret":
                args.append(derive(f"{group['id']}|v{variant}|a{i}", arg["value"]))
            elif arg["kind"] == "sealed":
                pt = derive(f"{group['id']}|v{variant}|a{i}", arg["value"])
                status, sealed = library_compute(9, [args[0], args[1], args[2], pt])
                args.append(sealed)
            elif arg["kind"] == "differing":
                if i == 0:
                    args.append(derive(f"{group['id']}|base", arg["value"]))
                else:
                    other = bytearray(args[0])
                    other[(variant * 7 + 1) % arg["value"]] ^= 0x5A
                    args.append(bytes(other))
        assert len(args) == n
        return encode_request(group["op"], args)

    def traces(self) -> None:
        variants = self.profile["trace_variants"]
        for build in self.primary_builds():
            for group in self.packet["trace_groups"]:
                if build["profile"] not in group["profiles"]:
                    continue
                hashes, blocks, statuses, walls = [], [], [], []
                for v in range(variants):
                    run = self.execute(build["driver"], build["tuple"], "emulated", self.group_request(group, v),
                                       trace=True, wall_seconds=600)
                    hashes.append(run["trace"]["sha256"])
                    blocks.append(run["trace"]["blocks"])
                    responses, _ = parse_responses(run["stdout"])
                    statuses.append(responses[0][0] if responses and run["state"] == "completed"
                                    else f"{run['state']} (exit {run['exit']}, signal {run['signal']})")
                    walls.append(run["wall_us"])
                distinct = len(set(hashes))
                ran = all(s == "ok" for s in statuses) and all(b > 0 for b in blocks)
                if not ran:
                    state = "unresolved"
                elif group["expect"] == "equal":
                    state = "pass" if distinct == 1 else "fail"
                else:
                    state = "pass" if distinct == variants else "fail"
                self.record("trace", f"{build['id']}/{group['id']}", {
                    "build": build["id"], "tuple": build["tuple"], "profile": build["profile"], "group": group["id"],
                    "role": group["role"], "expect": group["expect"], "hashes": hashes, "blocks": blocks,
                    "statuses": statuses, "distinct": distinct, "wall_us": sum(walls), "state": state})

    # -- NT-05

    def cross_links(self) -> None:
        opt = self.profile["optimizations"][0]
        for tup in TUPLES:
            built = [tc for tc in tup["toolchains"]
                     if build_id(tup["id"], tc, "P-BASE", opt, 1) in self.builds
                     and self.builds[build_id(tup["id"], tc, "P-BASE", opt, 1)]["state"] == "built"]
            if len(built) < 2:
                self.record("cross_link", tup["id"], {"tuple": tup["id"], "state": "unavailable",
                                                      "toolchains_built": built,
                                                      "reason": "fewer than two C toolchains built this tuple"})
                continue
            for kernel_tc, runtime_tc in ((built[0], built[1]), (built[1], built[0])):
                kdir = self.builds[build_id(tup["id"], kernel_tc, "P-BASE", opt, 1)]["dir"]
                rdir = self.builds[build_id(tup["id"], runtime_tc, "P-BASE", opt, 1)]["dir"]
                out = self.work / "cross" / f"{tup['id']}_{kernel_tc}_{runtime_tc}"
                if out.exists():
                    shutil.rmtree(out)
                (out / "tmp").mkdir(parents=True)
                objects = [str(kdir / f"k_{p}.o") for p in KERNEL_PARTS] + [str(rdir / f"r_{p}.o") for p in RUNTIME_PARTS]
                link = self.launcher.run([*self.link_argv(tup["id"], runtime_tc, "P-BASE"), *objects, "-o",
                                          str(out / "driver")], ro=[str(kdir), str(rdir)], rw=[str(out)], cwd=out,
                                         env={"TMPDIR": str(out / "tmp")})
                key = f"{tup['id']}/{kernel_tc}+{runtime_tc}"
                if link["state"] != "completed":
                    self.record("cross_link", key, {"tuple": tup["id"], "kernels": kernel_tc, "runtime": runtime_tc,
                                                    "state": "failed", "run_state": link["state"],
                                                    "diagnostics": link["stderr"][:2000].decode(
                                                        "utf-8", "replace")})
                    continue
                requests = self.kat_requests("P-BASE")
                stdin = b"".join(encode_request(op, args) for _rid, op, args in requests)
                run = self.execute(out / "driver", tup["id"], "emulated", stdin)
                rows = self.compare(requests, parse_responses(run["stdout"])[0])
                agree = run["state"] == "completed" and all(r["match"] for r in rows)
                self.record("cross_link", key, {"tuple": tup["id"], "kernels": kernel_tc, "runtime": runtime_tc,
                                                "run_state": run["state"],
                                                "matched": sum(1 for r in rows if r["match"]), "total": len(rows),
                                                "state": "agree" if agree else "disagree"})

    def rust_probes(self) -> None:
        opt = self.profile["optimizations"][0]
        sysroot = self.rustc.parent.parent if self.rustc else None
        for tup in TUPLES:
            std = sysroot / "lib" / "rustlib" / tup["rust_target"] / "lib" if sysroot else None
            host_tuple = tup["arch"] == self.host["arch"]
            if not (std and std.is_dir()) or not host_tuple:
                reason = ("the Rust standard library for this target is not installed"
                          if not (std and std.is_dir()) else "the probe is hosted and runs only on the host tuple")
                self.record("rust_probe", tup["id"], {"tuple": tup["id"], "state": "unavailable", "reason": reason})
                continue
            for tc in tup["toolchains"]:
                bid = build_id(tup["id"], tc, "P-BASE", opt, 1)
                if bid not in self.builds or self.builds[bid]["state"] != "built":
                    continue
                kdir = self.builds[bid]["dir"]
                out = self.work / "rust" / f"{tup['id']}_{tc}"
                if out.exists():
                    shutil.rmtree(out)
                (out / "tmp").mkdir(parents=True)
                argv = [str(self.rustc), "--edition", "2024", "-O", "-C", "panic=abort", "--target", tup["rust_target"],
                        *[f"-Clink-arg={kdir / f'k_{p}.o'}" for p in KERNEL_PARTS],
                        str(self.src / "abi_probe.rs"), "-o", str(out / "probe")]
                build = self.launcher.run(argv, ro=[str(self.src), str(kdir), str(sysroot)], rw=[str(out)], cwd=out,
                                          env={"TMPDIR": str(out / "tmp")})
                key = f"{tup['id']}/{tc}"
                if build["state"] != "completed":
                    self.record("rust_probe", key, {"tuple": tup["id"], "toolchain": tc, "state": "failed",
                                                    "run_state": build["state"],
                                                    "wall_us": build["wall_us"], "cpu_us": build["cpu_us"],
                                                    "diagnostics": build["stderr"][:3000].decode("utf-8", "replace")})
                    continue
                requests = self.kat_requests("P-BASE")
                stdin = b"".join(encode_request(op, args) for _rid, op, args in requests)
                run = self.launcher.run([str(out / "probe")], ro=[str(out / "probe")], stdin=stdin)
                rows = self.compare(requests, parse_responses(run["stdout"])[0])
                negatives = []
                for neg in self.packet["negatives"]:
                    if neg["kind"] != "request" or "P-BASE" not in neg["profiles"]:
                        continue
                    nrun = self.launcher.run([str(out / "probe")], ro=[str(out / "probe")],
                                             stdin=encode_request(neg["op"], [bytes.fromhex(a) for a in neg["args"]]))
                    responses, whole = parse_responses(nrun["stdout"])
                    negatives.append({"negative": neg["id"], "run_state": nrun["state"],
                                      "pass": nrun["state"] == "completed" and whole and len(responses) == 1
                                      and responses[0][0] == neg["expect"]})
                agree = run["state"] == "completed" and all(r["match"] for r in rows)
                self.record("rust_probe", key, {
                    "tuple": tup["id"], "toolchain": tc, "rustc": self.tools.get("TC-07", {}).get("version", ""),
                    "build_wall_us": build["wall_us"], "build_cpu_us": build["cpu_us"], "run_wall_us": run["wall_us"],
                    "run_state": run["state"],
                    "matched": sum(1 for r in rows if r["match"]), "total": len(rows), "kat_agree": agree,
                    "mismatches": [r for r in rows if not r["match"]][:10], "negatives": negatives,
                    "negatives_pass": all(n["pass"] for n in negatives),
                    "state": "agree" if agree and all(n["pass"] for n in negatives) else "disagree"})

    # -- owner input

    def owner(self) -> None:
        if self.owner_input is None:
            self.record("owner_input", "owner", {"present": False})
            return
        raw = self.owner_input.read_bytes()
        content = json.loads(raw.decode("utf-8"))
        problems = owner_input_errors(content)
        self.record("owner_input", "owner", {"present": True, "sha256": sha256_hex(raw), "content": content,
                                             "problems": problems,
                                             "note": "as supplied; the laboratory does not verify who wrote it"})

    # -- the whole epoch

    def run(self) -> Path:
        self.prepare()
        self.oracles()
        self.build_all()
        self.kat()
        self.negatives()
        self.inventory()
        self.traces()
        self.cross_links()
        self.rust_probes()
        self.owner()
        summary = summarize(self.records, self.packet, self.profile_name)
        summary["epoch"] = self.name
        (self.archive / "summary.json").write_bytes(canonical_file(gate0_numbers(summary)))
        (self.archive / "index.json").write_bytes(canonical_file({"epoch": self.name, "records": self.index}))
        write_manifest(self.archive)
        return self.archive


def owner_input_errors(content: Any) -> list[str]:
    problems = []
    if not isinstance(content, dict) or content.get("schema") != OWNER_SCHEMA:
        return ["schema"]
    rows = {r[0] for r in INVENTORY}
    for entry in content.get("isa_abi_verification", []):
        if entry.get("row") not in rows or entry.get("verdict") not in ("", "verified", "rejected"):
            problems.append(f"isa_abi_verification {entry.get('row')}")
    entries: dict[str, list[dict[str, Any]]] = {}
    for group in ("hardware", "native_runs", "feature_negatives"):
        entries[group] = []
        for entry in content.get(group, []):
            if not isinstance(entry, dict) or entry.get("tuple") not in TUPLE_IDS:
                problems.append(f"{group} tuple")
                continue
            if not isinstance(entry.get("device"), str) or not entry["device"].strip():
                problems.append(f"{group} device")
                continue
            entries[group].append(entry)
    attested = attested_devices(entries["hardware"])
    for entry in entries["native_runs"]:
        if not all(re.fullmatch(r"[0-9a-f]{64}", str(entry.get(k, ""))) for k in ("driver_sha256", "stdout_sha256")):
            problems.append("native_runs digests")
        if entry["device"] not in attested.get(entry["tuple"], set()):
            problems.append("native_runs device is not an attested device of its tuple")
    for entry in entries["feature_negatives"]:
        if not re.fullmatch(r"[0-9a-f]{64}", str(entry.get("driver_sha256", ""))):
            problems.append("feature_negatives digest")
        if not isinstance(entry.get("result"), str) or not entry["result"]:
            problems.append("feature_negatives result")
    if content.get("distinguishing_rule") not in (None, *[r[0] for r in RULES]):
        problems.append("distinguishing_rule")
    capacity = content.get("solo_slice_capacity")
    if capacity is not None and (not isinstance(capacity, int) or capacity < 0):
        problems.append("solo_slice_capacity")
    return problems


def attested_devices(hardware: list[dict[str, Any]]) -> dict[str, set[str]]:
    """Per tuple, the devices the owner attests run it natively."""

    devices: dict[str, set[str]] = {}
    for entry in hardware:
        if entry.get("runs_natively") is True and isinstance(entry.get("device"), str) and entry["device"]:
            devices.setdefault(entry.get("tuple"), set()).add(entry["device"])
    return devices


def sigill_drivers(feature_negatives: list[dict[str, Any]]) -> set[str]:
    """Drivers the owner attests stop with SIGILL on a named device without the extension."""

    return {e["driver_sha256"] for e in feature_negatives
            if e.get("result") == "SIGILL" and isinstance(e.get("device"), str) and e["device"]
            and isinstance(e.get("driver_sha256"), str) and re.fullmatch(r"[0-9a-f]{64}", e["driver_sha256"])}


def matched_native_runs(runs: list[dict[str, Any]], devices: set[str], drivers: dict[str, str],
                        emulated_out: dict[str, str]) -> list[dict[str, Any]]:
    """Native runs on an attested device of an archived driver whose output equals that driver's emulated
    output."""

    return [r for r in runs if r.get("device") in devices and r.get("driver_sha256") in drivers
            and emulated_out.get(drivers[r["driver_sha256"]]) == r.get("stdout_sha256")]


def archive_files(archive: Path) -> list[str]:
    """Every regular file and every other entry (link, device, socket) under an archive, except the manifest."""

    return sorted(p.relative_to(archive).as_posix() for p in archive.rglob("*")
                  if not p.is_dir() or p.is_symlink()) if archive.is_dir() else []


def unlisted(archive: Path, listed: Iterable[str]) -> list[str]:
    """Entries in the archive that its manifest does not list."""

    known = set(listed) | {MANIFEST_NAME}
    return [name for name in archive_files(archive) if name not in known]


def write_manifest(archive: Path) -> None:
    rows = []
    for name in archive_files(archive):
        if name == MANIFEST_NAME:
            continue
        path = archive / name
        if path.is_symlink() or not path.is_file():
            raise RunError(f"{name} in the archive is not a regular file")
        rows.append({"path": name, "sha256": file_sha256(path), "bytes": path.stat().st_size})
    (archive / MANIFEST_NAME).write_bytes(canonical_file({"files": rows}))


# ---------------------------------------------------------------------------
# The summary: gates per tuple and for the portable path, then per candidate; axes; the conclusion.
# It is a pure function of the records and the packet, so `verify` recomputes it.


def _gate(state: str, reasons: list[str], vacuous: bool = False) -> dict[str, Any]:
    return {"state": state, "reasons": sorted(set(reasons))[:12], "vacuous": vacuous}


def _worst(pairs: list[tuple[str, str]]) -> dict[str, Any]:
    """pairs of (state, reason); an empty list passes."""

    if not pairs:
        return _gate("pass", [])
    state = combine(s for s, _ in pairs)
    return _gate(state, [r for s, r in pairs if s == state and r])


def _owner_view(content: dict[str, Any] | None) -> dict[str, Any]:
    if content is None:
        return {"verified": {}, "hardware": {}, "no_hardware": set(), "scopes_done": set(), "rule": None,
                "capacity": None, "native_runs": {}, "feature_negatives": {}, "devices": {}}
    by_tuple = lambda group: {t: [e for e in content.get(group, []) if e.get("tuple") == t]  # noqa: E731
                              for t in TUPLE_IDS}
    return {"verified": {e["row"]: e["verdict"] for e in content.get("isa_abi_verification", [])},
            "native_runs": by_tuple("native_runs"), "feature_negatives": by_tuple("feature_negatives"),
            "hardware": {t: [h for h in content.get("hardware", []) if h.get("tuple") == t and h.get("runs_natively")]
                         for t in TUPLE_IDS},
            "devices": attested_devices(content.get("hardware", [])),
            "no_hardware": set(content.get("no_hardware", [])),
            "scopes_done": {e["scope"] for e in content.get("review_scopes", []) if e.get("done") is True},
            "rule": content.get("distinguishing_rule"), "capacity": content.get("solo_slice_capacity")}


def summarize(records: list[dict[str, Any]], packet: dict[str, Any], profile_name: str) -> dict[str, Any]:
    stage: dict[str, list[dict[str, Any]]] = {}
    for rec in records:
        stage.setdefault(rec["stage"], []).append(rec)
    host = stage["host"][0]["host"]
    owner_rec = (stage.get("owner_input") or [{"present": False}])[0]
    content = owner_rec.get("content") if owner_rec.get("present") and not owner_rec.get("problems") else None
    owner = _owner_view(content)
    oracle = {r["key"]: r for r in stage.get("oracle", [])}
    subject_oracles = {k: v for k, v in oracle.items() if k.startswith("K-")}
    builds = stage.get("build", [])
    lines = (stage.get("source_lines") or [{"accel": {}, "runtime": {}}])[0]
    profile = packet["run_profiles"][profile_name]
    first_opt = profile["optimizations"][0]
    host_tuple = TUPLE_BY_ARCH.get(host.get("arch", ""))

    def of(name: str, tup: str) -> list[dict[str, Any]]:
        return [r for r in stage.get(name, []) if r.get("tuple") == tup]

    def reference(tup: str, prof: str) -> dict[str, Any] | None:
        for tc in TUPLE_BY_ID[tup]["toolchains"]:
            bid = build_id(tup, tc, prof, first_opt, 1)
            inv = [r for r in stage.get("inventory", []) if r["build"] == bid]
            if inv:
                return inv[0]
        return None

    def determinism(rows: list[dict[str, Any]]) -> tuple[bool | None, list[str]]:
        groups: dict[tuple[str, str, str], list[dict[str, Any]]] = {}
        for b in rows:
            if b["state"] == "built":
                groups.setdefault((b["toolchain"], b["profile"], b["opt"]), []).append(b)
        if not groups or any(len(g) < 2 for g in groups.values()):
            return None, []
        bad = []
        for key, group in groups.items():
            digests = {canonical([[o.get("sha256") for o in b["objects"]], b["link"].get("sha256")]) for b in group}
            if len(digests) != 1:
                bad.append("/".join(key))
        return not bad, bad

    tuple_results: dict[str, Any] = {}
    gates_by_member: dict[str, dict[str, dict[str, Any]]] = {}
    for tup in TUPLES:
        tid = tup["id"]
        planned = [b for b in builds if b["tuple"] == tid]
        built1 = [b for b in planned if b["rep"] == 1 and b["state"] == "built"]
        ids1 = {build_id(tid, b["toolchain"], b["profile"], b["opt"], 1) for b in built1}
        g: dict[str, dict[str, Any]] = {}
        # HG-01
        pairs = [("fail", f"{b['key']} failed") for b in planned if b["state"] == "failed"]
        pairs += [("unresolved", f"{b['toolchain']} unavailable: {b.get('reason', '')}") for b in planned
                  if b["state"] == "unavailable"]
        for name in ("kat", "inventory", "trace", "negative"):
            have = {r["build"] for r in of(name, tid)}
            pairs += [("unresolved", f"{name} missing for {bid}") for bid in sorted(ids1 - have)]
        if not built1:
            pairs.append(("unresolved", "no build of this tuple"))
        g["HG-01"] = _worst(pairs)
        # HG-02
        pairs = []
        for k in of("kat", tid):
            if not k["complete"]:
                pairs.append(("fail", f"{k['key']}: {k['run_state']} run (exit {k['exit']}, signal {k['signal']})"))
            if not k["timings_consistent"]:
                pairs.append(("fail", f"{k['key']}: a timed run did not complete with the same responses"))
            for row in k["results"]:
                if not row["match"]:
                    state = oracle.get(row["subject"].split("+")[0], {}).get("state")
                    pairs.append(("fail" if state == "agree" else "unresolved",
                                  f"{k['key']}: {row['subject']} {'mismatch' if state == 'agree' else 'oracle ' + str(state)}"))
        for r in of("native_emulated", tid):
            if not r["identical"]:
                pairs.append(("fail", f"{r['build']}: native and emulated responses differ"))
        if not of("kat", tid):
            pairs.append(("unresolved", "no known-answer run"))
        pairs += [("unresolved", f"oracle {s}: {o['state']}") for s, o in oracle.items() if o["state"] != "agree"]
        g["HG-02"] = _worst(pairs)
        # HG-03
        # An N-12 the emulator cannot run is settled only by an owner attestation naming that build's driver.
        negs = of("negative", tid)
        driver_of = {b["key"]: b["link"].get("sha256") for b in planned if b["state"] == "built"}
        attested = sigill_drivers(owner["feature_negatives"].get(tid, []))
        pairs = [(n["state"], f"{n['key']}: {n.get('reason', n['state'])}") for n in negs if n["state"] != "pass"
                 and not (n["negative"] == "N-12" and n["state"] == "unresolved"
                          and driver_of.get(n["build"]) in attested)]
        if not negs:
            pairs.append(("unresolved", "no negative checks ran"))
        g["HG-03"] = _worst(pairs)
        # HG-04
        pairs = []
        for inv in of("inventory", tid):
            for name, obj in inv["objects"].items():
                if obj["role"] == "kernel" and obj["classes"]["A"]:
                    pairs.append(("fail", f"{inv['build']}: {name} has {obj['classes']['A']} class A"))
            if not inv["completed"]:
                pairs.append(("unresolved", f"{inv['build']}: a disassembly step did not complete"))
            if not inv["control_detected"]:
                pairs.append(("unresolved", f"{inv['build']}: division control not detected"))
        if not of("inventory", tid):
            pairs.append(("unresolved", "no inventory"))
        g["HG-04"] = _worst(pairs)
        # HG-05
        pairs = []
        for tr in of("trace", tid):
            if tr["role"] == "kernel" and tr["state"] != "pass":
                pairs.append((tr["state"], f"{tr['key']}: {tr['distinct']} distinct traces"))
            if tr["role"] != "kernel" and tr["state"] != "pass":
                pairs.append(("unresolved", f"{tr['key']}: the trace control did not differ"))
        if not of("trace", tid):
            pairs.append(("unresolved", "no traces"))
        g["HG-05"] = _worst(pairs)
        # HG-06
        pairs = []
        for c in of("cross_link", tid):
            if c["state"] == "unavailable":
                pairs.append(("unresolved", f"cross-toolchain link: {c['reason']}"))
            elif c["state"] != "agree":
                pairs.append(("fail", f"{c['key']}: {c['state']}"))
        for rp in of("rust_probe", tid):
            if rp["state"] == "unavailable":
                pairs.append(("unresolved", f"Rust probe: {rp['reason']}"))
            elif rp["state"] == "failed" or not rp.get("kat_agree", False):
                pairs.append(("fail", f"{rp['key']}: Rust probe {rp['state']}"))
        for inv in of("inventory", tid):
            for name, obj in inv["objects"].items():
                if obj["role"] == "kernel" and obj["symbol_violations"]:
                    pairs.append(("fail", f"{inv['build']}: {name} imports {obj['symbol_violations']}"))
        g["HG-06"] = _worst(pairs)
        # HG-07
        required = [r for r in packet["isa_abi_inventory"] if r["tuple"] == tid and r["required"]]
        verdicts = [owner["verified"].get(r["id"], "") for r in required]
        if any(v == "rejected" for v in verdicts):
            g["HG-07"] = _gate("fail", [f"{r['id']} rejected by the owner" for r, v in zip(required, verdicts)
                                        if v == "rejected"])
        elif verdicts and all(v == "verified" for v in verdicts):
            g["HG-07"] = _gate("pass", [])
        else:
            g["HG-07"] = _gate("unresolved", ["required inventory rows not verified by the owner"])
        # HG-08: an attested device and a native run on that device that reproduces the emulated responses
        drivers = {b["link"].get("sha256"): b["key"] for b in planned if b["state"] == "built" and b["rep"] == 1}
        emulated_out = {k["build"]: k["stdout_sha256"] for k in of("kat", tid) if k["mode"] == "emulated"}
        runs = owner["native_runs"].get(tid, [])
        matched_runs = matched_native_runs(runs, owner["devices"].get(tid, set()), drivers, emulated_out)
        if tid in owner["no_hardware"]:
            g["HG-08"] = _gate("fail", ["the owner declares no device for this tuple"])
        elif runs and len(matched_runs) != len(runs):
            g["HG-08"] = _gate("fail", ["an owner native run does not reproduce the emulated responses"])
        elif owner["hardware"].get(tid) and matched_runs:
            g["HG-08"] = _gate("pass", [])
        elif owner["hardware"].get(tid):
            g["HG-08"] = _gate("unresolved", ["device attested; no native run recorded"])
        else:
            g["HG-08"] = _gate("unresolved", ["no owner hardware attestation"])
        # HG-09
        deterministic, bad = determinism(planned)
        pairs = [("fail", f"nondeterministic rebuild: {k}") for k in bad]
        if deterministic is None:
            pairs.append(("unresolved", "fewer than two repetitions of some build"))
        if not of("kat", tid):
            pairs.append(("unresolved", "no timing"))
        g["HG-09"] = _worst(pairs)
        gates_by_member[tid] = g

        # Metrics and the resource estimate (NT-08)
        kats = of("kat", tid)
        ref = {p: reference(tid, p) for p in PROFILE_IDS}
        mnemonics: set[str] = set()
        relocs: set[str] = set()
        kernel_bytes: dict[str, int] = {}
        crypto_bytes: dict[str, int] = {}
        classes = {c: 0 for c in INSTRUCTION_CLASSES}
        for p, inv in ref.items():
            if inv is None:
                continue
            for name, obj in inv["objects"].items():
                if obj["role"] != "kernel":
                    continue
                mnemonics |= set(obj["mnemonics"])
                relocs |= set(obj["relocation_types"])
                if p == "P-BASE":
                    kernel_bytes[obj["family"]] = obj["code_bytes"]
                    for c in ("A", "B", "C", "D"):
                        classes[c] += obj["classes"][c]
                elif obj["part"] in ACCEL_PARTS:
                    crypto_bytes[obj["family"]] = obj["code_bytes"]
                    classes["E"] += obj["classes"]["E"]
        rep1 = [b for b in planned if b["rep"] == 1 and b["state"] == "built"]
        build_cpu = sum(sum(o["cpu_us"] for o in b["objects"]) + b["link"].get("cpu_us", 0) for b in rep1) // 1000
        build_wall = sum(sum(o["wall_us"] for o in b["objects"]) + b["link"].get("wall_us", 0) for b in rep1) // 1000
        emulated = [k["timing_median_wall_us"] for k in kats if k["mode"] == "emulated"]
        native = [k["timing_median_wall_us"] for k in kats if k["mode"] == "native"]
        traces = of("trace", tid)
        ref_trace_build = ref["P-BASE"]["build"] if ref["P-BASE"] else None
        trace_blocks = sum(t["blocks"][0] for t in traces if t["build"] == ref_trace_build and t["blocks"])
        trace_wall = sum(t["wall_us"] for t in traces) // 1000
        acquisitions = sorted({b["toolchain"] for b in planned if b["state"] == "unavailable"})
        acquisitions += [f"Rust standard library for {tup['rust_target']}" for r in of("rust_probe", tid)
                         if r["state"] == "unavailable" and "not installed" in r.get("reason", "")]
        arch = tup["arch"]
        resource = {
            "toolchains_planned": list(tup["toolchains"]),
            "toolchains_available": sorted({b["toolchain"] for b in planned if b["state"] != "unavailable"}),
            "acquisitions_needed": acquisitions,
            "builds_planned": len(planned), "builds_built": sum(1 for b in planned if b["state"] == "built"),
            "build_cpu_ms_one_repetition": build_cpu, "build_wall_ms_one_repetition": build_wall,
            "deterministic_rebuild": deterministic,
            "kernel_code_bytes_by_family": kernel_bytes, "crypto_profile_code_bytes": crypto_bytes,
            "distinct_kernel_mnemonics": len(mnemonics), "relocation_types": sorted(relocs),
            "target_specific_source_lines": lines["accel"].get(arch, 0) + lines["runtime"].get(arch, 0),
            "timing_batch_repeats": profile["timing_batch_repeats"],
            "known_answer_emulated_median_wall_ms": median(emulated) // 1000 if emulated else None,
            "known_answer_native_median_wall_ms": median(native) // 1000 if native else None,
            "launch_overhead_emulated_median_wall_ms": median(
                [k["launch_overhead_median_wall_us"] for k in kats if k["mode"] == "emulated"]) // 1000,
            "launch_overhead_native_median_wall_ms": median(
                [k["launch_overhead_median_wall_us"] for k in kats if k["mode"] == "native"]) // 1000 if native else None,
            "timing_note": "emulated and native times are different quantities and are never compared",
            "trace_blocks_reference_build": trace_blocks, "trace_wall_ms": trace_wall,
            "ci_projection_ms": build_wall + sum(emulated) // 1000 + trace_wall,
            "ci_projection_note": "projection from the contributor's laboratory host, not a CI measurement",
            "solo_slices": 1,
        }
        negs_all = of("negative", tid)
        trace_kernel = [t for t in traces if t["role"] == "kernel"]
        metrics = {
            "M-01": [sum(k["matched"] for k in kats), sum(k["total"] for k in kats)],
            "M-02": [sum(1 for o in subject_oracles.values() if o["state"] == "agree"), len(subject_oracles)],
            "M-03": [sum(1 for r in of("native_emulated", tid) if r["identical"]), len(of("native_emulated", tid))],
            "M-04": [sum(1 for n in negs_all if n["state"] == "pass"), len(negs_all)],
            "M-05": classes["A"], "M-06": classes["B"], "M-07": classes["C"], "M-08": classes["D"],
            "M-09": classes["E"],
            "M-10": [sum(1 for t in trace_kernel if t["state"] == "pass"), len(trace_kernel)],
            "M-11": {"division_control_detected": [sum(1 for i in of("inventory", tid) if i["control_detected"]),
                                                   len(of("inventory", tid))],
                     "trace_control_differs": [sum(1 for t in traces if t["role"] != "kernel" and t["state"] == "pass"),
                                               sum(1 for t in traces if t["role"] != "kernel")]},
            "M-12": [sum(1 for c in of("cross_link", tid) if c["state"] == "agree"),
                     sum(1 for c in of("cross_link", tid) if c["state"] != "unavailable")],
            "M-13": [sum(r.get("matched", 0) for r in of("rust_probe", tid)),
                     sum(r.get("total", 0) for r in of("rust_probe", tid))],
            "M-14": sum(len(o["symbol_violations"]) for i in of("inventory", tid) for o in i["objects"].values()
                        if o["role"] == "kernel"),
            "M-15": [sum(1 for v in verdicts if v == "verified"), len(verdicts)],
            "M-16": {"devices": len(owner["hardware"].get(tid, [])), "matching_native_runs": len(matched_runs)},
            "M-21": "unavailable",
            "M-22": 1,
        }
        tuple_results[tid] = {"gates": g, "metrics": metrics, "resource": resource,
                              "reference_builds": {p: (ref[p]["build"] if ref[p] else None) for p in PROFILE_IDS},
                              "class_d_by_reference_build": classes["D"], "mnemonics": sorted(mnemonics)}

    # The portable path: the host tuple's own builds and the Rust probe.
    pg: dict[str, dict[str, Any]] = {}
    probes = [r for r in stage.get("rust_probe", []) if r.get("tuple") == host_tuple and r["state"] != "unavailable"]
    host_builds = [b for b in builds if b["tuple"] == host_tuple and b["profile"] == "P-BASE"]
    if host_tuple is None:
        for gate in GATE_IDS:
            pg[gate] = _gate("unresolved", ["the laboratory host is not one of the tuples"], gate in PORTABLE_VACUOUS)
    else:
        pairs = [("fail", f"{b['key']} failed") for b in host_builds if b["state"] == "failed"]
        pairs += [("fail", f"{r['key']}: probe build failed") for r in probes if r["state"] == "failed"]
        if not probes:
            pairs.append(("unresolved", "no Rust probe ran"))
        pg["HG-01"] = _worst(pairs)
        pairs = [("fail", f"{r['key']}: probe known answers disagree") for r in probes
                 if r["state"] != "failed" and not r["kat_agree"]]
        natives = [k for k in stage.get("kat", []) if k["tuple"] == host_tuple and k["profile"] == "P-BASE"
                   and k["mode"] == "native"]
        pairs += [("fail", f"{k['key']}: native known answers disagree") for k in natives if k["matched"] != k["total"]]
        if not probes or not natives:
            pairs.append(("unresolved", "no native known-answer run through the C ABI"))
        pg["HG-02"] = _worst(pairs)
        pairs = [("fail", f"{r['key']}: probe negative checks failed") for r in probes
                 if r["state"] != "failed" and not r["negatives_pass"]]
        if not probes:
            pairs.append(("unresolved", "no Rust probe ran"))
        pg["HG-03"] = _worst(pairs)
        pairs = [("fail", f"{r['key']}: probe {r['state']}") for r in probes if r["state"] == "failed"]
        for inv in [i for i in stage.get("inventory", []) if i["tuple"] == host_tuple and i["profile"] == "P-BASE"]:
            for name, obj in inv["objects"].items():
                if obj["role"] == "kernel" and obj["symbol_violations"]:
                    pairs.append(("fail", f"{inv['build']}: {name} imports {obj['symbol_violations']}"))
        if not probes:
            pairs.append(("unresolved", "no Rust probe ran"))
        pg["HG-06"] = _worst(pairs)
        deterministic, bad = determinism(host_builds)
        pairs = [("fail", f"nondeterministic rebuild: {k}") for k in bad]
        if deterministic is None:
            pairs.append(("unresolved", "fewer than two repetitions of some build"))
        pg["HG-09"] = _worst(pairs)
        for gate in PORTABLE_VACUOUS:
            pg[gate] = _gate("pass", ["the portable path makes no machine-code claim"], vacuous=True)
    gates_by_member["C-PORTABLE"] = pg
    portable_ms = 0
    if host_tuple:
        rep1 = [b for b in host_builds if b["rep"] == 1 and b["state"] == "built" and b["opt"] == first_opt]
        portable_ms = sum(sum(o["wall_us"] for o in b["objects"]) + b["link"].get("wall_us", 0) for b in rep1) // 1000
        portable_ms += sum(r.get("build_wall_us", 0) + r.get("run_wall_us", 0) for r in probes) // 1000
    portable = {"gates": pg, "host_tuple": host_tuple, "ci_projection_ms": portable_ms,
                "rust_probes": [{"key": r["key"], "state": r["state"], "matched": r.get("matched"),
                                 "total": r.get("total")} for r in stage.get("rust_probe", [])]}

    # Candidates
    candidates: dict[str, Any] = {}
    for cand in packet["candidates"]:
        members = cand["native_tuples"]
        cg = candidate_gates(gates_by_member, members)
        live = [(m, gate) for m in ["C-PORTABLE", *members] for gate in GATE_IDS
                if not gates_by_member[m][gate]["vacuous"]]
        toolchains = set()
        acquisitions = set()
        for m in members:
            toolchains |= set(TUPLE_BY_ID[m]["toolchains"])
            if TUPLE_BY_ID[m]["arch"] != host.get("arch"):
                toolchains.add("TC-06")
            acquisitions |= set(tuple_results[m]["resource"]["acquisitions_needed"])
        toolchains |= {"TC-07"} | set(TUPLE_BY_ID[host_tuple]["toolchains"] if host_tuple else [])
        mnemonics_union: set[str] = set()
        for m in members:
            mnemonics_union |= set(tuple_results[m]["mnemonics"])
        axes = {
            "AX-01": len(members),
            "AX-02": sum(1 for m, gate in live if gates_by_member[m][gate]["state"] == "pass"),
            "AX-03": sum(tuple_results[m]["class_d_by_reference_build"] for m in members),
            "AX-04": portable_ms + sum(tuple_results[m]["resource"]["ci_projection_ms"] for m in members),
            "AX-05": len(mnemonics_union) + sum(tuple_results[m]["resource"]["target_specific_source_lines"]
                                                for m in members),
            "AX-06": len({t for t in toolchains if TOOLCHAIN_BY_ID[t]["kind"] in ("gcc", "clang", "rustc")} |
                         ({"TC-06"} & toolchains)) + len(acquisitions),
            "AX-07": len(members),
        }
        candidates[cand["id"]] = {
            "name": cand["name"], "native_tuples": members,
            "gates": {gate: {"state": v["state"], "vacuous": v["vacuous"]} for gate, v in cg.items()},
            "eligible": all(v["state"] == "pass" for v in cg.values()),
            "axes": axes, "acquisitions_needed": sorted(acquisitions)}

    conclusion = conclude(candidates, profile_name, content, owner)
    return {
        "schema": SUMMARY_SCHEMA, "label": LABEL, "profile": profile_name,
        "profile_is_result": profile_name == "measured",
        "statement": packet["stand_in_statement"],
        "host": {"arch": host.get("arch"), "cpu_model": host.get("cpu_model"), "role": host.get("role")},
        "owner_input": {"present": bool(owner_rec.get("present")), "sha256": owner_rec.get("sha256"),
                        "problems": owner_rec.get("problems", [])},
        "oracles": {"subjects": len(subject_oracles),
                    "agree": sum(1 for o in subject_oracles.values() if o["state"] == "agree"),
                    "negative_checks_with_orange_oracle": len(oracle) - len(subject_oracles)},
        "tuples": tuple_results, "portable_path": portable, "candidates": candidates, "conclusion": conclusion,
        "nonclaims": packet["nonclaims"],
    }


def _material(axis: str, a: int, b: int) -> bool:
    if a == b:
        return False
    if axis in ("AX-03", "AX-05"):
        return 10 * abs(a - b) >= max(a, b)
    if axis == "AX-04":
        return 5 * abs(a - b) >= max(a, b)
    return True


def _better(axis: str, a: int, b: int) -> int:
    """1 when a is materially better than b on the axis, -1 when materially worse, 0 otherwise."""

    if not _material(axis, a, b):
        return 0
    higher = dict((x[0], x[3]) for x in AXES)[axis] == "higher"
    return 1 if (a > b) == higher else -1


def conclude(candidates: dict[str, Any], profile_name: str, content: dict[str, Any] | None,
             owner: dict[str, Any]) -> dict[str, Any]:
    eligible = [c for c, v in candidates.items() if v["eligible"]]
    base = {"eligible": eligible, "rule": owner["rule"],
            "note": "advice to the owner; the choice and any freeze are the owner's alone and are not recorded here"}
    if profile_name != "measured":
        return {**base, "result": "inconclusive", "reason": "dev profile: this epoch exercises the laboratory and is "
                                                            "not a result"}
    if content is None:
        return {**base, "result": "inconclusive", "reason": "no valid owner input"}
    missing = [s[0] for s in REVIEW_SCOPES if s[0] not in owner["scopes_done"]]
    if missing:
        return {**base, "result": "inconclusive", "reason": f"owner review scopes not done: {', '.join(missing)}"}
    if not eligible:
        return {**base, "result": "inconclusive", "reason": "no candidate passes every gate"}
    if len(eligible) == 1:
        return {**base, "result": f"recommend_{eligible[0].lower().replace('-', '_')}", "reason": "sole eligible"}
    rule = owner["rule"]
    if rule is None:
        return {**base, "result": "inconclusive",
                "reason": "several nested candidates are eligible and no owner distinguishing rule is recorded"}
    axes = {c: candidates[c]["axes"] for c in eligible}
    pool = list(eligible)
    if rule == "DR-1":
        capacity = owner["capacity"]
        pool = [c for c in pool if capacity is not None and axes[c]["AX-07"] <= capacity]
        top = max((axes[c]["AX-01"] for c in pool), default=None)
        pool = [c for c in pool if axes[c]["AX-01"] == top]
    elif rule == "DR-2":
        low = min(axes[c]["AX-07"] for c in pool)
        pool = [c for c in pool if axes[c]["AX-07"] == low]
        top = max(axes[c]["AX-01"] for c in pool)
        pool = [c for c in pool if axes[c]["AX-01"] == top]
    elif rule == "DR-3":
        for axis in ("AX-04", "AX-06"):
            pool = [c for c in pool if not any(_better(axis, axes[o][axis], axes[c][axis]) == 1 for o in pool)]
    elif rule == "DR-4":
        pool = [c for c in pool if all(
            all(_better(a, axes[c][a], axes[o][a]) >= 0 for a in AXIS_IDS) and
            any(_better(a, axes[c][a], axes[o][a]) == 1 for a in AXIS_IDS) for o in eligible if o != c)]
        if len(pool) != 1:
            return {**base, "result": "inconclusive", "reason": "DR-4: no candidate dominates"}
    if not pool:
        return {**base, "result": "inconclusive", "reason": f"{rule}: no eligible candidate fits"}
    if len(pool) == 1:
        return {**base, "result": f"recommend_{pool[0].lower().replace('-', '_')}", "reason": rule}
    return {**base, "result": "tie", "tied": pool, "reason": rule}


# ---------------------------------------------------------------------------
# Verification of an archive


def verify(archive: Path) -> list[str]:
    """Re-check an epoch archive: every file against its manifest, no file outside it, every record against
    its digest and this epoch, and the summary recomputed from the records alone."""

    problems: list[str] = []
    try:
        manifest = json.loads((archive / MANIFEST_NAME).read_text(encoding="utf-8"))
        index = json.loads((archive / "index.json").read_text(encoding="utf-8"))
        packet_bytes = (archive / "packet.json").read_bytes()
        epoch = json.loads((archive / "epoch.json").read_text(encoding="utf-8"))
    except (OSError, ValueError) as exc:
        return [f"archive unreadable: {exc}"]
    listed = []
    for row in manifest["files"]:
        listed.append(row["path"])
        path = archive / row["path"]
        if path.is_symlink() or not path.is_file() or file_sha256(path) != row["sha256"] \
                or path.stat().st_size != row["bytes"]:
            problems.append(f"{row['path']} differs from the archive manifest")
    problems += [f"{name} is not in the archive manifest" for name in unlisted(archive, listed)]
    raw_records: dict[str, bytes] = {}
    for entry in index["records"]:
        try:
            raw_records[entry["sha256"]] = (archive / "records" / f"{entry['sha256']}.json").read_bytes()
        except OSError:
            pass
    indexed = {f"records/{entry['sha256']}.json" for entry in index["records"]}
    problems += [f"{name} is not in the record index" for name in listed
                 if name.startswith("records/") and name not in indexed]
    try:
        summary_bytes = (archive / "summary.json").read_bytes()
    except OSError as exc:
        return problems + [f"archive unreadable: {exc}"]
    return problems + verify_epoch(archive.name, epoch, packet_bytes, index, raw_records, summary_bytes)


def verify_epoch(name: str, epoch: dict[str, Any], packet_bytes: bytes, index: dict[str, Any],
                 raw_records: dict[str, bytes], summary_bytes: bytes) -> list[str]:
    """The checks an archive and its export share: the epoch name hashes its identity, the packet is the
    one the epoch bound, every indexed record is present, matches its digest and belongs to this epoch, and
    the summary recomputes byte for byte from the records alone."""

    problems: list[str] = []
    if epoch_name(epoch["identity"]) != epoch["epoch"] or epoch["epoch"] != name:
        problems.append("the epoch name is not the hash of its identity")
    if sha256_hex(packet_bytes) != epoch["identity"]["packet_sha256"]:
        problems.append("packet.json is not the packet the epoch bound")
    records = []
    for entry in index["records"]:
        raw = raw_records.get(entry["sha256"])
        if raw is None:
            problems.append(f"record {entry['key']} is missing")
            continue
        if sha256_hex(raw) != entry["sha256"]:
            problems.append(f"record {entry['key']} does not match its digest")
        record = json.loads(raw.decode("utf-8"))
        if record.get("schema") != RECORD_SCHEMA or record.get("epoch") != epoch["epoch"] \
                or record.get("stage") != entry["stage"] or record.get("key") != entry["key"]:
            problems.append(f"record {entry['key']} is not the indexed {RECORD_SCHEMA} record of this epoch")
        records.append(record)
    try:
        summary = summarize(records, json.loads(packet_bytes.decode("utf-8")), epoch["identity"]["profile"])
    except (KeyError, IndexError, TypeError, ValueError) as exc:
        return problems + [f"the summary cannot be recomputed: {exc!r}"]
    summary["epoch"] = epoch["epoch"]
    if canonical_file(gate0_numbers(summary)) != summary_bytes:
        problems.append("summary.json does not recompute byte for byte from the records")
    return problems


# ---------------------------------------------------------------------------
# Committed epoch exports
#
# An export is what the repository keeps of an epoch: epoch.json, packet.json and summary.json as the archive
# holds them, every record as one line of gzip-compressed JSON lines, and a manifest. The archive's other rows
# (index.json and the driver ELFs under products/) are carried in the manifest by digest: the index is rebuilt
# from the record lines, and the products are rebuilt deterministically by a later epoch rather than
# committed. The rebuilt archive manifest must hash to
# the digest the export names, so the export holds exactly the archive's records and nothing else.

EXPORT_SCHEMA = "d011-v0.1-export-1"
EXPORT_MANIFEST = "manifest.json"
EXPORT_CHUNK_RAW_BYTES = 2 * 1024 * 1024
EXPORT_CHUNK_MAX_BYTES = 480 * 1024
RUN_ROOT = REPOSITORY_ROOT / LAB / "run"


def _gzip(data: bytes) -> bytes:
    return gzip.compress(data, compresslevel=9, mtime=0)


def export(archive: Path, out_root: Path) -> Path:
    """Write the export of a verified archive to out_root/EPOCH, replacing an earlier export of it."""

    problems = verify(archive)
    if problems:
        raise RunError(f"{archive.name} does not verify: {problems[0]}")
    manifest_bytes = (archive / MANIFEST_NAME).read_bytes()
    manifest = json.loads(manifest_bytes.decode("utf-8"))
    index = json.loads((archive / "index.json").read_text(encoding="utf-8"))
    out = out_root / archive.name
    out.mkdir(parents=True, exist_ok=True)
    for child in sorted(out.iterdir()):
        if child.is_symlink() or not child.is_file():
            raise RunError(f"{child} is not a regular file")
        child.unlink()
    chunks: list[bytes] = []
    pending = b""
    for entry in index["records"]:
        line = (archive / "records" / f"{entry['sha256']}.json").read_bytes()
        if pending and len(pending) + len(line) > EXPORT_CHUNK_RAW_BYTES:
            chunks.append(pending)
            pending = b""
        pending += line
    if pending:
        chunks.append(pending)
    written: dict[str, bytes] = {
        "epoch.json": (archive / "epoch.json").read_bytes(),
        "packet.json": (archive / "packet.json").read_bytes(),
        "summary.json": (archive / "summary.json").read_bytes(),
    }
    for number, chunk in enumerate(chunks, start=1):
        data = _gzip(chunk)
        if len(data) > EXPORT_CHUNK_MAX_BYTES:
            raise RunError(f"records chunk {number} compresses to {len(data)} bytes, over the export cap")
        written[f"records-{number:02d}.jsonl.gz"] = data
    for name, data in written.items():
        (out / name).write_bytes(data)
    (out / EXPORT_MANIFEST).write_bytes(canonical_file({
        "schema": EXPORT_SCHEMA, "epoch": archive.name, "label": LABEL,
        "archive_manifest_sha256": sha256_hex(manifest_bytes),
        "carried": [row for row in manifest["files"] if not row["path"].startswith("records/")
                    and row["path"] not in written],
        "files": [{"path": name, "sha256": sha256_hex(data), "bytes": len(data)}
                  for name, data in sorted(written.items())],
        "note": "index.json is rebuilt from the record lines and products/ are named by digest only",
    }))
    return out


def verify_export(out: Path) -> list[str]:
    """Re-check an export: its files against its manifest, no file outside it, the archive manifest rebuilt
    from the carried rows and the record lines, and then every check verify makes of an archive."""

    problems: list[str] = []
    try:
        manifest = json.loads((out / EXPORT_MANIFEST).read_text(encoding="utf-8"))
        epoch_bytes = (out / "epoch.json").read_bytes()
        packet_bytes = (out / "packet.json").read_bytes()
        summary_bytes = (out / "summary.json").read_bytes()
        epoch = json.loads(epoch_bytes.decode("utf-8"))
    except (OSError, ValueError) as exc:
        return [f"export unreadable: {exc}"]
    if manifest.get("schema") != EXPORT_SCHEMA or manifest.get("epoch") != out.name:
        problems.append(f"{EXPORT_MANIFEST} is not the {EXPORT_SCHEMA} manifest of this epoch")
    listed = {EXPORT_MANIFEST}
    lines: list[bytes] = []
    for row in manifest["files"]:
        chunk = re.fullmatch(r"records-[0-9]{2}\.jsonl\.gz", row["path"]) is not None
        if not chunk and row["path"] not in ("epoch.json", "packet.json", "summary.json"):
            problems.append(f"{row['path']!r} is not a file an export holds")
            continue
        listed.add(row["path"])
        path = out / row["path"]
        if path.is_symlink() or not path.is_file() or file_sha256(path) != row["sha256"] \
                or path.stat().st_size != row["bytes"]:
            problems.append(f"{row['path']} differs from the export manifest")
            continue
        if chunk:
            try:
                lines += gzip.decompress(path.read_bytes()).splitlines(keepends=True)
            except (OSError, EOFError, ValueError) as exc:
                problems.append(f"{row['path']} does not decompress: {exc}")
    problems += [f"{name} is not in the export manifest" for name in archive_files(out) if name not in listed]
    raw_records = {sha256_hex(line): line for line in lines}
    entries = []
    for digest, line in raw_records.items():
        try:
            record = json.loads(line.decode("utf-8"))
            entries.append({"key": record["key"], "sha256": digest, "stage": record["stage"]})
        except (ValueError, KeyError, TypeError):
            problems.append(f"a record line is not a record: {digest}")
    if len(raw_records) != len(lines):
        problems.append("a record line appears more than once")
    index = {"epoch": out.name, "records": entries}
    own = {"epoch.json": epoch_bytes, "summary.json": summary_bytes, "index.json": canonical_file(index),
           "packet.json": packet_bytes}
    rows = [{"path": f"records/{digest}.json", "sha256": digest, "bytes": len(line)}
            for digest, line in raw_records.items()]
    rows += [{"path": name, "sha256": sha256_hex(data), "bytes": len(data)} for name, data in own.items()]
    rows += [row for row in manifest["carried"] if row["path"] not in own]
    rebuilt = canonical_file({"files": sorted(rows, key=lambda row: row["path"])})
    if sha256_hex(rebuilt) != manifest["archive_manifest_sha256"]:
        problems.append("the carried rows and record lines do not rebuild the archive manifest")
    return problems + verify_epoch(out.name, epoch, packet_bytes, index, raw_records, summary_bytes)


# ---------------------------------------------------------------------------
# Command line

USAGE = next((part for part in (__doc__ or "").split("\n\n") if part.startswith("Usage:")), "Usage: see the module")
PROFILE_NAMES = {name: name for name in RUN_PROFILES}


def _option(argv: list[str], name: str) -> str | None:
    if name in argv:
        i = argv.index(name)
        if i + 1 >= len(argv):
            raise SystemExit(f"{name} needs a value")
        return argv[i + 1]
    return None


def main(argv: list[str]) -> int:
    if not argv:
        print(USAGE)
        return 2
    command = argv[0]
    try:
        if command == "generate" and len(argv) == 1:
            (REPOSITORY_ROOT / PACKET_PATH).write_bytes(committed_packet_bytes())
            print(f"wrote {PACKET_PATH}")
            return 0
        if command == "check" and len(argv) == 1:
            problems = check()
            for problem in problems:
                print(f"FAIL  {problem}")
            print("ok" if not problems else f"{len(problems)} problems")
            return 1 if problems else 0
        if command == "owner-template" and len(argv) == 1:
            sys.stdout.write(canonical_file(owner_template()).decode("utf-8"))
            return 0
        if command == "run":
            profile = PROFILE_NAMES.get(_option(argv, "--profile") or "dev")
            if profile is None:
                print(USAGE)
                return 2
            owner_name = _option(argv, "--owner-input")
            if owner_name is not None:
                ensure_archive_root()
            owner = existing_entry(owner_name, OWNER_INPUT_DIR, directories=False) if owner_name else None
            path = Lab(REPOSITORY_ROOT, profile, owner).run()
            summary = json.loads((path / "summary.json").read_text(encoding="utf-8"))
            print(f"archive {path}")
            print(f"conclusion {summary['conclusion']['result']}: {summary['conclusion']['reason']}")
            return 0
        if command == "export" and len(argv) == 2:
            out = export(existing_entry(argv[1], ARCHIVE_ROOT, directories=True), RUN_ROOT)
            print(f"wrote {out.relative_to(REPOSITORY_ROOT).as_posix()}")
            return 0
        if command == "verify" and len(argv) == 2:
            try:
                problems = verify(existing_entry(argv[1], ARCHIVE_ROOT, directories=True))
            except RunError:
                problems = verify_export(existing_entry(argv[1], RUN_ROOT, directories=True))
            for problem in problems:
                print(f"FAIL  {problem}")
            print("ok" if not problems else f"{len(problems)} problems")
            return 1 if problems else 0
    except RunError as exc:
        print(f"error: {exc}", file=sys.stderr)
        return 1
    print(USAGE)
    return 2


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
