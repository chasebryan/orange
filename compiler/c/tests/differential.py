#!/usr/bin/env python3
"""Compare the standalone C compiler with the Rust frontend on the S3a–S3f fixtures."""

import re
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
C_DIR = Path(__file__).resolve().parents[1]
FIXTURES = ROOT / "compiler" / "fixtures"
RUST = ROOT / "compiler" / "target" / "debug" / "orangec"
C_COMPILER = C_DIR / "out" / "orangec-asan"
CODE = re.compile(r"^error\[(ORC[0-9]+)\]", re.MULTILINE)

VALID = [
    "s3a/valid-empty-mixed.or",
    "s3a/valid-int-radices.or",
    "s3a/valid-word8-boundaries.or",
    "s3b/valid-calls-and-grouping.or",
    "s3b/valid-chacha20-quarter-round.or",
    "s3b/valid-int-arithmetic.or",
    "s3b/valid-sha256-functions.or",
    "s3b/valid-word-arithmetic.or",
    "s3c/valid-byte-order.or",
    "s3c/valid-chacha20-quarter-round.or",
    "s3c/valid-conversions.or",
    "s3c/valid-let-and-as-names.or",
    "s3c/valid-sha256-round.or",
    "s3d/valid-arrays.or",
    "s3d/valid-chacha20-block.or",
    "s3d/valid-sha256-rounds.or",
    "s3e/valid-chacha20.or",
    "s3e/valid-loops.or",
    "s3e/valid-sha256.or",
    "s3f/valid-aead.or",
    "s3f/valid-conditions.or",
    "s3f/valid-poly1305.or",
    "s3f/valid-x25519.or",
]
# Admitted by S3e. Kept inline so this check does not add a Gate 0 path.
# large-int-array: Int^2 of 2^16384-1 does not fit in an 8192-byte value buffer.
# array-slots: 1025 arrays of 256 words overflow a store that is never released.
EXTRA = {
    "large-int-array.or": """\
edition 2026;
module bigprint {
  spec wide() -> Int {
    let a: Int = for i in 0..13 with n: Int = 2 { n * n };
    (a - 1) * (a + 1)
  }
  spec pair() -> Int^2 { [wide(), wide()] }
  spec one() -> Int { wide() }
}
""",
    "array-slots.or": """\
edition 2026;
module slots {
  spec block() -> Word[8]^256 { [0; 256] }
  spec many() -> Word[8]^256 {
    for i in 0..1025 with s: Word[8]^256 = block() { block() }
  }
  spec bumped() -> Word[8]^256 {
    for i in 0..1025 with s: Word[8]^256 = block() { s with [0] = 1 }
  }
  spec id(x: Word[8]^256) -> Word[8]^256 { x }
  spec passed() -> Word[8]^256 {
    for i in 0..1025 with s: Word[8]^256 = id(block()) { id(s) }
  }
}
""",
}
INVALID = [
    "s3b/invalid-call-cycles.or",
    "s3b/invalid-diagnostic-order.or",
    "s3b/invalid-names-and-calls.or",
    "s3b/invalid-parameter-syntax.or",
    "s3b/invalid-shift-amounts.or",
    "s3b/invalid-types-and-operators.or",
    "s3b/invalid-ungrouped-operators.or",
    "s3b/invalid-word-literals.or",
    "s3b/invalid-word-widths.or",
    "s3c/invalid-binding-names.or",
    "s3c/invalid-binding-syntax.or",
    "s3c/invalid-binding-types.or",
    "s3c/invalid-conversions.or",
    "s3c/invalid-ungrouped-conversions.or",
    "s3d/invalid-array-literals.or",
    "s3d/invalid-array-operators.or",
    "s3d/invalid-array-syntax.or",
    "s3d/invalid-array-types.or",
    "s3d/invalid-indices.or",
    "s3e/invalid-indices.or",
    "s3e/invalid-loop-syntax.or",
    "s3e/invalid-loops.or",
    "s3e/invalid-updates.or",
    "s3f/invalid-comparisons.or",
    "s3f/invalid-condition-syntax.or",
    "s3f/invalid-conditions.or",
    "s3f/invalid-division-indices.or",
]


def run(compiler: Path, args: list[str]) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        [str(compiler), *args],
        cwd=ROOT,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
    )


def codes(stderr: str) -> list[str]:
    return CODE.findall(stderr)


def main() -> int:
    c_compiler = C_COMPILER
    rust_compiler = RUST
    if not c_compiler.is_file() or not rust_compiler.is_file():
        print("missing compiler binary", file=sys.stderr)
        return 2

    failures = 0
    for relative in VALID:
        path = str(FIXTURES / relative)
        rust = run(rust_compiler, ["eval", path])
        c_result = run(c_compiler, ["eval", path])
        again = run(c_compiler, ["eval", path])
        if rust.returncode != 0 or c_result.returncode != 0 or rust.stdout != c_result.stdout or c_result.stdout != again.stdout:
            failures += 1
            print(f"FAIL eval {relative}")
            print(f"  rust exit {rust.returncode} c exit {c_result.returncode}")
            if rust.stdout != c_result.stdout:
                print("  stdout mismatch")
                print("  rust:", rust.stdout)
                print("  c:   ", c_result.stdout)
            if c_result.stderr:
                print("  c stderr:", c_result.stderr)
        else:
            print(f"ok   eval {relative}")

    with tempfile.TemporaryDirectory(prefix="orangec-c-extra-") as temporary:
        extra_paths = []
        for name, source in EXTRA.items():
            path = Path(temporary) / name
            path.write_text(source, encoding="utf-8")
            extra_paths.append(path)
        for path in extra_paths:
            rust = run(rust_compiler, ["eval", str(path)])
            c_result = run(c_compiler, ["eval", str(path)])
            again = run(c_compiler, ["eval", str(path)])
            label = path.name
            if rust.returncode != 0 or c_result.returncode != 0 or rust.stdout != c_result.stdout or c_result.stdout != again.stdout:
                failures += 1
                print(f"FAIL eval {label}")
                print(f"  rust exit {rust.returncode} c exit {c_result.returncode}")
                if rust.stdout != c_result.stdout:
                    print("  stdout mismatch")
                    print(f"  rust bytes {len(rust.stdout)} c bytes {len(c_result.stdout)}")
                if c_result.stderr:
                    print("  c stderr:", c_result.stderr)
                if rust.stderr:
                    print("  rust stderr:", rust.stderr)
            else:
                print(f"ok   eval {label}")

    for relative in INVALID:
        path = str(FIXTURES / relative)
        rust = run(rust_compiler, ["check", path])
        c_result = run(c_compiler, ["check", path])
        rust_codes = codes(rust.stderr)
        c_codes = codes(c_result.stderr)
        if rust.returncode == 0 or c_result.returncode == 0 or rust_codes != c_codes:
            failures += 1
            print(f"FAIL check {relative}")
            print(f"  rust {rust_codes}")
            print(f"  c    {c_codes}")
            if rust_codes != c_codes:
                print("  c stderr:", c_result.stderr)
        else:
            print(f"ok   check {relative}")

    sample = FIXTURES / "s3b" / "valid-int-arithmetic.or"
    first = run(c_compiler, ["lex", str(sample)])
    second = run(c_compiler, ["lex", str(sample)])
    rust_lex = run(rust_compiler, ["lex", str(sample)])
    if first.returncode != 0 or first.stdout != second.stdout or first.stdout != rust_lex.stdout:
        failures += 1
        print("FAIL lex determinism or Rust token stream")
        if first.stdout != rust_lex.stdout:
            print("  rust:", rust_lex.stdout.splitlines()[:8])
            print("  c:   ", first.stdout.splitlines()[:8])
    else:
        print("ok   lex valid-int-arithmetic.or")

    if failures:
        print(f"{failures} failure(s)")
        return 1
    print("differential comparison passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
