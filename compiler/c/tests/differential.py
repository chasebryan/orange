#!/usr/bin/env python3
"""Compare the standalone C compiler with the Rust frontend on the S3a–S3o fixtures."""

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
    "s3g/valid-aes128.or",
    "s3g/valid-lookups.or",
    "s3h/valid-vectors.or",
    "s3i/valid-fields.or",
    "s3i/valid-poly1305.or",
    "s3i/valid-x25519.or",
    "s3j/valid-blocks.or",
    "s3j/valid-sha256.or",
    "s3j/valid-x25519.or",
    "s3k/valid-ascon.or",
    "s3k/valid-chacha20.or",
    "s3k/valid-sha256.or",
    "s3k/valid-tuples.or",
    "s3l/valid-aead.or",
    "s3l/valid-hmac.or",
    "s3l/valid-bytes.or",
    "s3m/valid-poly1305.or",
    "s3m/valid-hmac.or",
    "s3m/valid-sizes.or",
    "s3m/sha256.or",
    "s3n/valid-order.or",
    "s3n/valid-sha256.or",
    "s3n/valid-sha512.or",
    "s3n/valid-chacha20.or",
    "s3n/valid-poly1305.or",
    "s3n/valid-x25519.or",
    "s3o/valid-types.or",
    "s3o/valid-fields.or",
    "s3o/valid-sha2.or",
    "s3o/valid-nested.or",
    "s3o/valid-tuple.or",
    "s3p/valid-rfc8439.or",
    "s3p/valid-share.or",
    "s3p/valid-rank2.or",
    "s3q/valid-equality.or",
    "s3q/valid-rfc8439-tests.or",
    "s3q/failing-tests.or",
    "s3r/valid-amounts.or",
    "s3r/valid-rc6.or",
    "s3r/valid-sha3.or",
    "s3r/valid-shift-probes.or",
    "s3r/valid-zetas.or",
]
# Pepin's test does not finish in the default 1048576 steps. Rust's
# conformance run uses this budget and prints --stats.
STEPPED = [
    ("s3p/valid-lengths.or", ["--steps", "2097152", "--stats"]),
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
    "s3a/invalid-duplicate-spec.or",
    "s3a/invalid-int-magnitude.or",
    "s3a/invalid-negative-word.or",
    "s3a/invalid-typed-impl.or",
    "s3a/invalid-unsupported-type.or",
    "s3a/invalid-word-range.or",
    "s3a/invalid-word-width.or",
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
    "s3g/invalid-int-indices.or",
    "s3g/invalid-word-indices.or",
    "s3h/invalid-calls.or",
    "s3h/invalid-graph.or",
    "s3h/invalid-missing.or",
    "s3i/invalid-moduli.or",
    "s3i/invalid-residues.or",
    "s3i/invalid-type-syntax.or",
    "s3i/invalid-types.or",
    "s3j/invalid-block-names.or",
    "s3j/invalid-block-syntax.or",
    "s3j/invalid-block-types.or",
    "s3k/invalid-tuple-names.or",
    "s3k/invalid-tuple-syntax.or",
    "s3k/invalid-tuple-types.or",
    "s3m/invalid-alias-once.or",
    "s3m/invalid-alias-through.or",
    "s3m/invalid-alias-tuple.or",
    "s3m/invalid-alias-twice.or",
    "s3m/invalid-alias-types.or",
    "s3m/invalid-alias-unused.or",
    "s3l/invalid-bytes-lexical.or",
    "s3l/invalid-bytes-syntax.or",
    "s3l/invalid-bytes-types.or",
    "s3m/invalid-sizes.or",
    "s3m/invalid-sizes-syntax.or",
    "s3n/invalid-order.or",
    "s3n/invalid-order-syntax.or",
    "s3o/invalid-alias-listed.or",
    "s3o/invalid-types.or",
    "s3o/invalid-types-syntax.or",
    "s3o/invalid-nested.or",
    "s3o/invalid-project.or",
    "s3p/invalid-lengths.or",
    "s3p/invalid-rank2.or",
    "s3q/invalid-test-missing-body.or",
    "s3q/invalid-test-missing-brace.or",
    "s3q/invalid-test-expr-brace.or",
    "s3q/invalid-test-unclosed-body.or",
    "s3q/invalid-test-syntax.or",
    "s3q/invalid-tests.or",
    "s3r/invalid-amount-grouping.or",
    "s3r/invalid-amounts.or",
    "s3r/invalid-shift-probes.or",
    "s3r/invalid-word-width-array.or",
    "s3r/invalid-word-width-tuple.or",
]

# Rust evaluates these matrices. This slice rejects the value and pins C's
# diagnostic so it cannot drift. The reason is printed and is not a step-budget
# skip: step_budget.py's SKIPS stays empty.
RANK2_REASON = "rank-2 arrays: nested-array slice"
RANK2 = [
    (
        "s3p/rank2-id.or",
        """\
error[ORC0203]: a value of type `(Word[8]^2)^2` is a matrix, which this compiler does not evaluate
 --> {path}:6:14
  |
6 |   spec id(a: Mat) -> Mat {{ a }}
  |              ^^^ rank-2 arrays: nested-array slice
  = note: a row holds scalars; a matrix holds rows of the same type

error[ORC0203]: a value of type `(Word[8]^2)^2` is a matrix, which this compiler does not evaluate
 --> {path}:6:22
  |
6 |   spec id(a: Mat) -> Mat {{ a }}
  |                      ^^^ rank-2 arrays: nested-array slice
  = note: a row holds scalars; a matrix holds rows of the same type

error[ORC0203]: a value of type `(Word[8]^2)^2` is a matrix, which this compiler does not evaluate
 --> {path}:7:16
  |
7 |   spec pass(a: Mat) -> Int {{ 0 }}
  |                ^^^ rank-2 arrays: nested-array slice
  = note: a row holds scalars; a matrix holds rows of the same type
""",
    ),
    (
        "s3p/rank2-index.or",
        """\
error[ORC0203]: a value of type `(Word[8]^2)^2` is a matrix, which this compiler does not evaluate
 --> {path}:6:29
  |
6 |   spec row(a: Mat) -> Row {{ a[0] }}
  |                             ^^^^ rank-2 arrays: nested-array slice
  = note: a row holds scalars; a matrix holds rows of the same type
""",
    ),
    (
        "s3p/rank2-listed.or",
        """\
error[ORC0203]: a value of type `(Word[8]^2)^2` is a matrix, which this compiler does not evaluate
 --> {path}:6:21
  |
6 |   spec f[K in {{Row, Mat}}](x: K) -> K {{ x }}
  |                     ^^^ rank-2 arrays: nested-array slice
  = note: a row holds scalars; a matrix holds rows of the same type
""",
    ),
    (
        "s3p/rank2-nested.or",
        """\
error[ORC0203]: a value of type `(Word[8]^2)^2` is a matrix, which this compiler does not evaluate
 --> {path}:6:26
  |
6 |   spec nested() -> Mat {{ [[1, 2], [3, 4]] }}
  |                          ^^^^^^^^^^^^^^^^ rank-2 arrays: nested-array slice
  = note: a row holds scalars; a matrix holds rows of the same type
""",
    ),
    (
        "s3p/rank2-update.or",
        """\
error[ORC0203]: a value of type `(Word[8]^3)^3` is a matrix, which this compiler does not evaluate
 --> {path}:5:34
  |
5 |   spec cell(x: Row^3) -> Row^3 {{ x with [1] = (x[1] with [2] = 9) }}
  |                                  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ rank-2 arrays: nested-array slice
  = note: a row holds scalars; a matrix holds rows of the same type

error[ORC0203]: a value of type `(Word[8]^3)^3` is a matrix, which this compiler does not evaluate
 --> {path}:7:20
  |
7 |     let x: Row^3 = [[1, 2, 3], [4, 5, 6], [7, 8, 9]];
  |                    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ rank-2 arrays: nested-array slice
  = note: a row holds scalars; a matrix holds rows of the same type

error[ORC0203]: a value of type `(Word[8]^3)^3` is a matrix, which this compiler does not evaluate
 --> {path}:8:5
  |
8 |     x with [1] = (x[1] with [2] = 9)
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ rank-2 arrays: nested-array slice
  = note: a row holds scalars; a matrix holds rows of the same type
""",
    ),
    (
        "s3p/rank2-uses.or",
        """\
error[ORC0203]: a value of type `(Word[8]^2)^2` is a matrix, which this compiler does not evaluate
 --> {path}:6:16
  |
6 |   type Alias = Mat;
  |                ^^^ rank-2 arrays: nested-array slice
  = note: a row holds scalars; a matrix holds rows of the same type

error[ORC0203]: a value of type `(Word[8]^2)^2` is a matrix, which this compiler does not evaluate
 --> {path}:7:16
  |
7 |   type Pair = (Mat, Int);
  |                ^^^ rank-2 arrays: nested-array slice
  = note: a row holds scalars; a matrix holds rows of the same type

error[ORC0203]: a value of type `(Word[8]^2)^2` is a matrix, which this compiler does not evaluate
 --> {path}:9:18
  |
9 |   spec inline(x: Row^2) -> Row^2 {{ x }}
  |                  ^^^^^ rank-2 arrays: nested-array slice
  = note: a row holds scalars; a matrix holds rows of the same type

error[ORC0203]: a value of type `(Word[8]^2)^2` is a matrix, which this compiler does not evaluate
 --> {path}:9:28
  |
9 |   spec inline(x: Row^2) -> Row^2 {{ x }}
  |                            ^^^^^ rank-2 arrays: nested-array slice
  = note: a row holds scalars; a matrix holds rows of the same type

error[ORC0203]: a value of type `(Word[8]^2)^2` is a matrix, which this compiler does not evaluate
 --> {path}:10:16
   |
10 |   spec kept(a: Mat) -> Int {{ let x: Mat = a; 0 }}
   |                ^^^ rank-2 arrays: nested-array slice
  = note: a row holds scalars; a matrix holds rows of the same type

error[ORC0203]: a value of type `(Word[8]^2)^2` is a matrix, which this compiler does not evaluate
 --> {path}:10:37
   |
10 |   spec kept(a: Mat) -> Int {{ let x: Mat = a; 0 }}
   |                                     ^^^ rank-2 arrays: nested-array slice
  = note: a row holds scalars; a matrix holds rows of the same type

error[ORC0203]: a value of type `(Word[8]^2)^2` is a matrix, which this compiler does not evaluate
 --> {path}:11:18
   |
11 |   spec walked(a: Mat) -> Int {{ let n: Mat = for i in 0..1 with s: Mat = a {{ s }}; 0 }}
   |                  ^^^ rank-2 arrays: nested-array slice
  = note: a row holds scalars; a matrix holds rows of the same type

error[ORC0203]: a value of type `(Word[8]^2)^2` is a matrix, which this compiler does not evaluate
 --> {path}:11:39
   |
11 |   spec walked(a: Mat) -> Int {{ let n: Mat = for i in 0..1 with s: Mat = a {{ s }}; 0 }}
   |                                       ^^^ rank-2 arrays: nested-array slice
  = note: a row holds scalars; a matrix holds rows of the same type

error[ORC0203]: a value of type `(Word[8]^2)^2` is a matrix, which this compiler does not evaluate
 --> {path}:11:67
   |
11 | ... nt {{ let n: Mat = for i in 0..1 with s: Mat = a {{ s }}; 0 }}
   |                                             ^^^ rank-2 arrays: nested-array slice
  = note: a row holds scalars; a matrix holds rows of the same type
""",
    ),
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


def loop_source(lower: str, upper: str) -> str:
    return (
        "edition 2026;\n"
        "module bounds {\n"
        f"  spec f() -> Int {{ for i in {lower}..{upper} with s: Int = 0 {{ s }} }}\n"
        "}\n"
    )


def compare_codes(rust_compiler: Path, c_compiler: Path, name: str, source: str) -> bool:
    with tempfile.TemporaryDirectory() as directory:
        path = str(Path(directory) / f"{name}.or")
        Path(path).write_text(source)
        rust = run(rust_compiler, ["check", path])
        c_result = run(c_compiler, ["check", path])
    rust_codes = codes(rust.stderr)
    c_codes = codes(c_result.stderr)
    if rust.returncode == 0 or c_result.returncode == 0 or rust_codes != c_codes:
        print(f"FAIL check {name}")
        print(f"  rust {rust_codes}")
        print(f"  c    {c_codes}")
        return False
    print(f"ok   check {name}")
    return True


def index_source(index: str) -> str:
    return (
        "edition 2026;\n"
        "module bounds {\n"
        f"  spec f(x: Word[8]^4) -> Word[8] {{ x[{index}] }}\n"
        "}\n"
    )


def loop_bound_magnitude(rust_compiler: Path, c_compiler: Path) -> int:
    """An oversized literal is the magnitude limit, not a range error."""
    over = "0x" + ("f" * 4097)
    fits = "0x" + ("f" * 4096)
    half = "0x4" + ("0" * 4095)
    full = "0x8" + ("0" * 4095)
    wide = (
        "edition 2026;\n"
        "module bounds {\n"
        "  spec f(x: Word[8]^4) -> Word[8] { for i in 1..5 with s: Word[8] = 0 "
        f"{{ s ^ x[{full} + {full} - {full} - {full} + i - 1] }} }}\n"
        "}\n"
    )
    cases = [
        ("loop-bound-magnitude-upper", loop_source("0", over), ["ORC0205"]),
        ("loop-bound-magnitude-lower", loop_source(over, "1"), ["ORC0205"]),
        ("loop-bound-magnitude-before-range", loop_source("70000", over), ["ORC0205"]),
        ("loop-bound-magnitude-both", loop_source(over, over), ["ORC0205"]),
        ("loop-bound-fits-but-too-wide", loop_source("0", fits), ["ORC0225"]),
        ("loop-bound-65537", loop_source("0", "65537"), ["ORC0225"]),
        ("index-magnitude-grouped", index_source(f"({over})"), ["ORC0205"]),
        ("index-magnitude-sum", index_source(f"1 + {over}"), ["ORC0205"]),
        ("index-range-overflow", wide, ["ORC0223"]),
    ]
    failures = 0
    for name, source, expected in cases:
        if not compare_codes(rust_compiler, c_compiler, name, source):
            failures += 1
            continue
        with tempfile.TemporaryDirectory() as directory:
            path = str(Path(directory) / f"{name}.or")
            Path(path).write_text(source)
            observed = codes(run(c_compiler, ["check", path]).stderr)
        if observed != expected:
            failures += 1
            print(f"FAIL check {name} expected {expected} got {observed}")
    admitted = (
        "edition 2026;\n"
        "module bounds {\n"
        "  spec witness(x: Word[8]^4) -> Word[8] { x[0] }\n"
        "  spec f(x: Word[8]^4) -> Word[8] { for i in 1..5 with s: Word[8] = 0 "
        f"{{ s ^ x[{half} + {half} - {half} - {half} + i - 1] }} }}\n"
        "}\n"
    )
    with tempfile.TemporaryDirectory() as directory:
        path = str(Path(directory) / "index-range-admitted.or")
        Path(path).write_text(admitted)
        rust = run(rust_compiler, ["check", path])
        c_result = run(c_compiler, ["check", path])
    if rust.returncode != 0 or c_result.returncode != 0 or rust.stderr != "" or c_result.stderr != "":
        failures += 1
        print("FAIL check index-range-admitted")
        print(f"  rust exit {rust.returncode} {codes(rust.stderr)}")
        print(f"  c    exit {c_result.returncode} {codes(c_result.stderr)}")
    else:
        print("ok   check index-range-admitted")
    return failures


def index_chain_and_loop_recovery(rust_compiler: Path, c_compiler: Path) -> int:
    """A second index of a scalar is ORC0224, and a failed loop step is not ORC0104."""
    chain = (
        "edition 2026;\n"
        "module m {\n"
        "  spec f() -> Word[8] { let t: Word[8]^3 = [1, 2, 3]; t[0][1] }\n"
        "}\n"
    )
    chain3 = (
        "edition 2026;\n"
        "module m {\n"
        "  spec f() -> Word[8] { let t: Word[8]^3 = [1, 2, 3]; t[0][1][2] }\n"
        "}\n"
    )
    past = (
        "edition 2026;\n"
        "module m {\n"
        "  spec f() -> Word[8] { let t: Word[8]^4 = [1, 2, 3, 4]; t[4][0] }\n"
        "}\n"
    )
    unbound = (
        "edition 2026;\n"
        "module m {\n"
        "  spec f() -> Word[8] { let t: Word[8]^4 = [1, 2, 3, 4]; let k: Int = 0; t[k][0] }\n"
        "}\n"
    )
    called = (
        "edition 2026;\n"
        "module m {\n"
        "  spec g() -> Word[8]^3 { [1, 2, 3] }\n"
        "  spec f() -> Word[8] { g()[0][1] }\n"
        "}\n"
    )
    scalar = (
        "edition 2026;\n"
        "module m {\n"
        "  spec f() -> Word[8] { let x: Word[8] = 1; x[0][1] }\n"
        "}\n"
    )
    loop_chain = (
        "edition 2026;\n"
        "module m {\n"
        "  spec f() -> Word[8] { for i in 0..3 with r: Word[8]^3 = [1, 2, 3] { r[i][0] } }\n"
        "}\n"
    )
    step_only = (
        "edition 2026;\n"
        "module m {\n"
        "  spec f() -> Word[8]^3 { for i in 0..3 with r: Word[8]^3 = [1, 2, 3] { r[i][0] } }\n"
        "}\n"
    )
    two_exprs = (
        "edition 2026;\n"
        "module m {\n"
        "  spec f() -> Int { for i in 0..2 with s: Int = 0 { 1 2 } }\n"
        "}\n"
    )
    fill_index = (
        "edition 2026;\n"
        "module m {\n"
        "  spec f() -> Word[8]^3 { for i in 0..3 with r: Word[8]^3 = [0; 3] { r with [i] = [7; 3][i] } }\n"
        "}\n"
    )
    unclosed = (
        "edition 2026;\n"
        "module m {\n"
        "  spec f() -> Int { for i in 0..2 with s: Int = 0 { (1 } }\n"
        "}\n"
    )
    empty = (
        "edition 2026;\n"
        "module m {\n"
        "  spec f() -> Word[8] { let t: Word[8]^3 = [1, 2, 3]; t[0][] }\n"
        "}\n"
    )
    cases = [
        ("index-chain", chain),
        ("index-chain-three", chain3),
        ("index-chain-past-end", past),
        ("index-chain-unbounded", unbound),
        ("index-chain-call", called),
        ("index-chain-scalar", scalar),
        ("index-chain-loop", loop_chain),
        ("index-chain-step", step_only),
        ("loop-step-two-exprs", two_exprs),
        ("loop-step-fill-index", fill_index),
        ("loop-step-unclosed", unclosed),
        ("index-chain-empty", empty),
    ]
    failures = 0
    for name, source in cases:
        if not compare_codes(rust_compiler, c_compiler, name, source):
            failures += 1
    return failures

def module(body: str) -> str:
    return "edition 2026;\nmodule conv {\n" + body + "\n}\n"


def conversion_targets(rust_compiler: Path, c_compiler: Path) -> int:
    """An invalid `as` target does not hide the operand, or the outer conversion."""
    cases = [
        ("conv-lit-bad-width", module("  spec f() -> Word[8] { 1 as Word[7] }"), ["ORC0220", "ORC0204"]),
        ("conv-lit-missing-width", module("  spec f() -> Word[8] { 1 as Word }"), ["ORC0220", "ORC0204"]),
        ("conv-lit-float", module("  spec f() -> Word[8] { 1 as Float }"), ["ORC0220", "ORC0203"]),
        ("conv-bool-bad-width", module("  spec f(b: Bool) -> Word[8] { b as Word[7] }"), ["ORC0215", "ORC0204"]),
        ("conv-missing-name", module("  spec f() -> Word[8] { missing as Word[7] }"), ["ORC0211", "ORC0204"]),
        ("conv-missing-call", module("  spec f() -> Word[8] { missing() as Word[7] }"), ["ORC0212", "ORC0204"]),
        (
            "conv-array-bad-width",
            module("  spec f(t: Word[8]^4) -> Word[8] { t as Word[7] }"),
            ["ORC0215", "ORC0204"],
        ),
        (
            "conv-compare-bad-width",
            module("  spec f(x: Word[8]) -> Word[8] { (x == 1) as Word[7] }"),
            ["ORC0215", "ORC0204"],
        ),
        (
            "conv-conditional-untyped",
            module("  spec f(c: Bool) -> Word[8] { (if c { 1 } else { 2 }) as Word[7] }"),
            ["ORC0220", "ORC0204"],
        ),
        (
            "conv-nested-untyped",
            module("  spec f() -> Word[8] { (1 as Word[7]) as Word[8] }"),
            ["ORC0220", "ORC0204"],
        ),
        (
            "conv-nested-word",
            module("  spec f(x: Word[8]) -> Word[16] { (x as Word[7]) as Word[16] }"),
            ["ORC0204"],
        ),
        (
            "conv-nested-bool",
            module("  spec f(b: Bool) -> Int { (b as Word[99]) as Int }"),
            ["ORC0215", "ORC0204"],
        ),
        (
            "conv-typed-only-width",
            module("  spec f(x: Word[8]) -> Word[8] { x as Word[7] }"),
            ["ORC0204"],
        ),
        (
            "conv-compare-operand",
            module("  spec f(x: Word[8]) -> Bool { (x as Word[7]) == 1 }"),
            ["ORC0204"],
        ),
        (
            "conv-index-operand",
            module("  spec f(t: Word[8]^4, x: Word[8]) -> Word[8] { t[x as Word[7]] }"),
            ["ORC0204"],
        ),
        (
            "conv-leading-zero-width",
            module("  spec f() -> Word[8] { 1 as Word[08] }"),
            ["ORC0220", "ORC0204"],
        ),
    ]
    failures = 0
    for name, source, expected in cases:
        if not compare_codes(rust_compiler, c_compiler, name, source):
            failures += 1
            continue
        with tempfile.TemporaryDirectory() as directory:
            path = str(Path(directory) / f"{name}.or")
            Path(path).write_text(source)
            observed = codes(run(c_compiler, ["check", path]).stderr)
        if observed != expected:
            failures += 1
            print(f"FAIL check {name} expected {expected} got {observed}")
    admitted = module(
        "  spec f(x: Word[8]) -> Word[8] { (x as Word[16]) as Word[8] }\n"
        "  spec g() -> Word[8] { f(0x2c) }\n"
    )
    with tempfile.TemporaryDirectory() as directory:
        path = str(Path(directory) / "conv-nested-admitted.or")
        Path(path).write_text(admitted)
        rust = run(rust_compiler, ["eval", path])
        c_result = run(c_compiler, ["eval", path])
    if rust.returncode != 0 or c_result.returncode != 0 or rust.stdout != c_result.stdout:
        failures += 1
        print("FAIL eval conv-nested-admitted")
        print(f"  rust {rust.returncode} {rust.stdout!r} {codes(rust.stderr)}")
        print(f"  c    {c_result.returncode} {c_result.stdout!r} {codes(c_result.stderr)}")
    else:
        print("ok   eval conv-nested-admitted")
    return failures


def logical_operators(rust_compiler: Path, c_compiler: Path) -> int:
    """A rejected `!`, `&&`, or `||` is only ORC0215; a `Bool` operator still checks operands."""
    cases = [
        ("logic-bang-missing", module("  spec f() -> Word[8] { !missing }"), ["ORC0215"]),
        ("logic-bang-true", module("  spec f() -> Word[8] { !true }"), ["ORC0215"]),
        ("logic-bang-compare", module("  spec f() -> Word[8] { !(1 == 1) }"), ["ORC0215"]),
        ("logic-bang-int", module("  spec f() -> Int { !(missing && also) }"), ["ORC0215"]),
        ("logic-and-literals", module("  spec f() -> Int { true && false }"), ["ORC0215"]),
        ("logic-and-missing", module("  spec f() -> Word[8] { missing && also }"), ["ORC0215"]),
        ("logic-or-missing", module("  spec f() -> Int { missing || 1 }"), ["ORC0215"]),
        ("logic-and-array", module("  spec f() -> Bool^2 { true && false }"), ["ORC0215"]),
        ("logic-or-array", module("  spec f() -> Word[8]^4 { true || false }"), ["ORC0215"]),
        ("logic-bang-array", module("  spec f(t: Bool^2) -> Bool^2 { !t }"), ["ORC0215"]),
        ("logic-defined-missing", module("  spec f() -> Bool { missing || also }"), ["ORC0211", "ORC0211"]),
        ("logic-defined-and-one", module("  spec f(b: Bool) -> Bool { missing && b }"), ["ORC0211"]),
        ("logic-defined-literal", module("  spec f() -> Bool { true && 1 }"), ["ORC0214"]),
        ("logic-defined-bang", module("  spec f() -> Bool { !1 }"), ["ORC0214"]),
        (
            "logic-defined-words",
            module("  spec f(x: Word[8], y: Word[8]) -> Bool { x && y }"),
            ["ORC0214", "ORC0214"],
        ),
        (
            "logic-defined-condition",
            module("  spec f(c: Bool) -> Int { if missing && c { 1 } else { 2 } }"),
            ["ORC0211"],
        ),
    ]
    failures = 0
    for name, source, expected in cases:
        if not compare_codes(rust_compiler, c_compiler, name, source):
            failures += 1
            continue
        with tempfile.TemporaryDirectory() as directory:
            path = str(Path(directory) / f"{name}.or")
            Path(path).write_text(source)
            observed = codes(run(c_compiler, ["check", path]).stderr)
        if observed != expected:
            failures += 1
            print(f"FAIL check {name} expected {expected} got {observed}")
    admitted = module(
        "  spec f() -> Bool { !(true && false) || true }\n"
        "  spec g(b: Bool) -> Bool { b && !b }\n"
        "  spec t() -> Bool { g(true) }\n"
        "  spec u() -> Bool { g(false) }\n"
    )
    with tempfile.TemporaryDirectory() as directory:
        path = str(Path(directory) / "logic-admitted.or")
        Path(path).write_text(admitted)
        rust = run(rust_compiler, ["eval", path])
        c_result = run(c_compiler, ["eval", path])
    if rust.returncode != 0 or c_result.returncode != 0 or rust.stdout != c_result.stdout:
        failures += 1
        print("FAIL eval logic-admitted")
        print(f"  rust {rust.returncode} {rust.stdout!r} {codes(rust.stderr)}")
        print(f"  c    {c_result.returncode} {c_result.stdout!r} {codes(c_result.stderr)}")
    else:
        print("ok   eval logic-admitted")
    return failures


def residue_modules(rust_compiler: Path, c_compiler: Path) -> int:
    """Moduli are values, not per-module table indexes. A rejected result stops the body."""

    def program(root: str, other: str) -> dict[str, str]:
        return {"a.or": root, "b.or": other}

    root = "edition 2026;\nmodule m {\n%s\n}\n"
    other = "edition 2026;\nmodule b {\n%s\n}\n"
    checks = [
        (
            "mod-cross-mismatch",
            program(
                root % "  use b;\n  spec f() -> Mod[7] { b::g() }",
                other % "  spec g() -> Mod[11] { 4 }",
            ),
            ["ORC0214"],
        ),
        (
            "mod-cross-alias",
            program(
                root % "  use b;\n  type F = Mod[7];\n  spec f() -> F { b::g() }",
                other % "  type G = Mod[11];\n  spec g() -> G { 4 }",
            ),
            ["ORC0214"],
        ),
        (
            "mod-arg-mismatch",
            program(
                root % "  use b;\n  spec f(x: Mod[7]) -> Mod[11] { b::g(x) }",
                other % "  spec g(x: Mod[11]) -> Mod[11] { x }",
            ),
            ["ORC0214"],
        ),
        (
            "mod-index-wide",
            program(
                root % "  use b;\n  spec pad() -> Mod[4] { 1 }\n  spec f(t: Int^4) -> Int { t[b::g() as Int] }",
                other % "  spec g() -> Mod[100] { 3 }",
            ),
            ["ORC0223"],
        ),
        (
            "mod-elem-mismatch",
            program(
                root % "  use b;\n  spec f() -> Mod[7] { b::row()[0] }",
                other % "  spec row() -> Mod[11]^3 { [1, 2, 3] }",
            ),
            ["ORC0214"],
        ),
        (
            "mod-result-float",
            {"a.or": root % "  spec f() -> Float { let x: Int = missing; 1 }"},
            ["ORC0203"],
        ),
        (
            "mod-result-forward",
            {"a.or": root % "  type F = G;\n  type G = Mod[7];\n  spec f() -> F { missing }"},
            ["ORC0203"],
        ),
        (
            "mod-result-modulus",
            {"a.or": root % "  spec f() -> Mod[1] { missing }"},
            ["ORC0232"],
        ),
    ]
    evals = [
        (
            "mod-cross-same",
            program(
                root % "  use b;\n  spec other() -> Mod[7] { 1 }\n  spec f() -> Mod[11] { b::g() + 1 }",
                other % "  spec g() -> Mod[11] { 4 }",
            ),
        ),
        (
            "mod-cross-arg",
            program(
                root % "  use b;\n  spec pad() -> Mod[5] { 1 }\n  spec f() -> Mod[11] { b::g(4) + 2 }",
                other % "  spec g(x: Mod[11]) -> Mod[11] { x * 3 }",
            ),
        ),
        (
            "mod-cross-array",
            program(
                root % "  use b;\n  spec pad() -> Mod[3] { 1 }\n  spec f() -> Mod[11] { b::row()[1] + 1 }",
                other % "  spec row() -> Mod[11]^2 { [1, 2] }",
            ),
        ),
        (
            "mod-cross-index",
            program(
                root
                % "  use b;\n  spec pad() -> Mod[100] { 1 }\n  spec f(t: Int^4) -> Int { t[b::g() as Int] }\n  spec a() -> Int { f([9, 8, 7, 6]) }",
                other % "  spec g() -> Mod[4] { 3 }",
            ),
        ),
    ]
    failures = 0
    for name, files, expected in checks:
        with tempfile.TemporaryDirectory() as directory:
            for filename, source in files.items():
                Path(directory, filename).write_text(source)
            path = str(Path(directory) / "a.or")
            rust = run(rust_compiler, ["check", path])
            c_result = run(c_compiler, ["check", path])
        rust_codes = codes(rust.stderr)
        c_codes = codes(c_result.stderr)
        if rust.returncode == 0 or c_result.returncode == 0 or rust_codes != c_codes or c_codes != expected:
            failures += 1
            print(f"FAIL check {name}")
            print(f"  rust {rust_codes}")
            print(f"  c    {c_codes}")
        else:
            print(f"ok   check {name}")
    for name, files in evals:
        with tempfile.TemporaryDirectory() as directory:
            for filename, source in files.items():
                Path(directory, filename).write_text(source)
            path = str(Path(directory) / "a.or")
            rust = run(rust_compiler, ["eval", path])
            c_result = run(c_compiler, ["eval", path])
        if rust.returncode != 0 or c_result.returncode != 0 or rust.stdout != c_result.stdout or codes(rust.stderr) or codes(c_result.stderr):
            failures += 1
            print(f"FAIL eval {name}")
            print(f"  rust {rust.returncode} {rust.stdout!r} {codes(rust.stderr)}")
            print(f"  c    {c_result.returncode} {c_result.stdout!r} {codes(c_result.stderr)}")
        else:
            print(f"ok   eval {name}")
    return failures


def rank2_pins(rust_compiler: Path, c_compiler: Path) -> int:
    """Skip Rust's matrix evaluation and pin this slice's diagnostic."""
    failures = 0
    print(f"skip {RANK2_REASON}")
    if not RANK2_REASON.strip():
        print("FAIL rank-2 skip reason is empty")
        return 1
    for relative, pin in RANK2:
        path = str(FIXTURES / relative)
        expected = pin.format(path=path)
        rust_check = run(rust_compiler, ["check", path])
        rust_eval = run(rust_compiler, ["eval", path])
        c_check = run(c_compiler, ["check", path])
        c_eval = run(c_compiler, ["eval", path])
        if (
            rust_check.returncode != 0
            or rust_eval.returncode != 0
            or c_check.returncode == 0
            or c_eval.returncode == 0
            or c_check.stderr != expected
            or c_eval.stderr != expected
            or c_eval.stdout != ""
        ):
            failures += 1
            print(f"FAIL rank2 {relative}")
            print(f"  rust check {rust_check.returncode} eval {rust_eval.returncode}")
            print(f"  c check {c_check.returncode} eval {c_eval.returncode}")
            if c_check.stderr != expected:
                print("  c check stderr:", c_check.stderr)
                print("  pinned:", expected)
            if c_eval.stderr != expected:
                print("  c eval stderr:", c_eval.stderr)
            if c_eval.stdout:
                print("  c eval stdout:", c_eval.stdout)
        else:
            print(f"ok   rank2 {relative}")
    return failures


def main() -> int:
    c_compiler = C_COMPILER
    rust_compiler = RUST
    if len(sys.argv) > 1:
        given = Path(sys.argv[1])
        c_compiler = given if given.is_absolute() else (C_DIR / given).resolve()
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

    for relative, extra in STEPPED:
        path = str(FIXTURES / relative)
        args = ["eval", *extra, path]
        rust = run(rust_compiler, args)
        c_result = run(c_compiler, args)
        if rust.returncode != 0 or c_result.returncode != 0 or rust.stdout != c_result.stdout or rust.stderr != c_result.stderr:
            failures += 1
            print(f"FAIL eval {relative}")
            print(f"  rust exit {rust.returncode} c exit {c_result.returncode}")
            if rust.stdout != c_result.stdout:
                print("  stdout mismatch")
                print("  rust:", rust.stdout)
                print("  c:   ", c_result.stdout)
            if rust.stderr != c_result.stderr:
                print("  stderr mismatch")
                print("  rust:", rust.stderr)
                print("  c:   ", c_result.stderr)
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
        if rust.returncode == 0 or c_result.returncode == 0 or rust.stderr != c_result.stderr:
            failures += 1
            print(f"FAIL check {relative}")
            print(f"  rust {codes(rust.stderr)}")
            print(f"  c    {codes(c_result.stderr)}")
            if rust.stderr != c_result.stderr:
                print("  rust stderr:", rust.stderr)
                print("  c stderr:", c_result.stderr)
        else:
            print(f"ok   check {relative}")

    failures += rank2_pins(rust_compiler, c_compiler)

    for relative, expect in (
        ("s3q/valid-rfc8439-tests.or", 0),
        ("s3q/failing-tests.or", 1),
        ("s3r/valid-sha3.or", 0),
        ("s3r/valid-rc6.or", 0),
        ("s3r/valid-zetas.or", 0),
        ("s3r/valid-amounts.or", 0),
        ("s3r/valid-shift-probes.or", 0),
    ):
        path = str(FIXTURES / relative)
        rust = run(rust_compiler, ["test", path])
        c_result = run(c_compiler, ["test", path])
        check = run(c_compiler, ["check", path])
        if (
            rust.returncode != expect
            or c_result.returncode != expect
            or rust.stdout != c_result.stdout
            or rust.stderr != c_result.stderr
            or check.returncode != 0
            or check.stderr != ""
        ):
            failures += 1
            print(f"FAIL test {relative}")
            print(f"  rust exit {rust.returncode} c exit {c_result.returncode} check {check.returncode}")
            if rust.stdout != c_result.stdout:
                print("  stdout mismatch")
                print("  rust:", rust.stdout)
                print("  c:   ", c_result.stdout)
            if rust.stderr != c_result.stderr:
                print("  stderr mismatch")
                print("  rust:", rust.stderr)
                print("  c:   ", c_result.stderr)
        else:
            print(f"ok   test {relative}")

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

    failures += loop_bound_magnitude(rust_compiler, c_compiler)
    failures += index_chain_and_loop_recovery(rust_compiler, c_compiler)
    failures += conversion_targets(rust_compiler, c_compiler)
    failures += logical_operators(rust_compiler, c_compiler)
    failures += residue_modules(rust_compiler, c_compiler)
    failures += alias_targets(rust_compiler, c_compiler)
    failures += alias_uses(rust_compiler, c_compiler)

    if failures:
        print(f"{failures} failure(s)")
        return 1
    print("differential comparison passed")
    return 0


def _skip_space(source: str, index: int) -> int:
    while index < len(source) and source[index] in " \t\r\n":
        index += 1
    return index


def _ground_end(source: str, index: int):
    """End index of a ground type at `index`, or None when it names a local type."""
    index = _skip_space(source, index)
    if index >= len(source):
        return None
    if source[index] == "(":
        cursor = index + 1
        while True:
            end = _ground_end(source, cursor)
            if end is None:
                return None
            cursor = _skip_space(source, end)
            if cursor < len(source) and source[cursor] == ",":
                cursor += 1
                continue
            if cursor < len(source) and source[cursor] == ")":
                return cursor + 1
            return None
    for word in ("Word", "Int", "Bool", "Mod"):
        if not source.startswith(word, index):
            continue
        after = index + len(word)
        if after < len(source) and (source[after].isalnum() or source[after] == "_"):
            continue
        cursor = after
        if word in ("Word", "Mod") and cursor < len(source) and source[cursor] == "[":
            close = source.find("]", cursor + 1)
            if close < 0:
                return None
            inner = source[cursor + 1 : close]
            if word == "Word" and re.fullmatch(r"[0-9A-Za-z_]*", inner) is None:
                return None
            if word == "Mod" and re.fullmatch(r"[0-9A-Za-z_+\-*/<>() \t]*", inner) is None:
                return None
            cursor = close + 1
        if cursor < len(source) and source[cursor] == "^":
            cursor += 1
            if cursor < len(source) and source[cursor] == "(":
                depth = 1
                cursor += 1
                while cursor < len(source) and depth:
                    if source[cursor] == "(":
                        depth += 1
                    elif source[cursor] == ")":
                        depth -= 1
                    cursor += 1
            else:
                start = cursor
                while cursor < len(source) and (source[cursor].isalnum() or source[cursor] == "_"):
                    cursor += 1
                if cursor == start:
                    return None
        return cursor
    return None


def alias_targets(rust_compiler: Path, c_compiler: Path) -> int:
    """Every ground type in a rejecting invalid is also checked as `type T = ...`."""
    position = re.compile(r"(?:->|type\s+[A-Za-z_][A-Za-z0-9_]*\s*=|:)\s*")
    found = []
    failures = 0
    for relative in INVALID:
        text = (FIXTURES / relative).read_text(encoding="utf-8")
        for match in position.finditer(text):
            end = _ground_end(text, match.end())
            if end is None:
                continue
            expr = text[match.end() : end].strip()
            if expr and expr not in found:
                found.append(expr)
    for expr in found:
        source = f"edition 2026;\nmodule aliaspath {{\n  type T = {expr};\n}}\n"
        path = None
        try:
            with tempfile.NamedTemporaryFile("w", suffix=".or", delete=False, encoding="utf-8") as handle:
                handle.write(source)
                path = handle.name
            rust = run(rust_compiler, ["check", path])
            if rust.returncode == 0:
                continue
            c_result = run(c_compiler, ["check", path])
            if c_result.returncode == 0 or rust.stderr != c_result.stderr:
                failures += 1
                print(f"FAIL alias type T = {expr}")
                print(f"  rust {codes(rust.stderr)} exit {rust.returncode}")
                print(f"  c    {codes(c_result.stderr)} exit {c_result.returncode}")
                if rust.stderr != c_result.stderr:
                    print("  rust stderr:", rust.stderr)
                    print("  c stderr:", c_result.stderr)
            else:
                print(f"ok   alias type T = {expr}")
        finally:
            if path is not None:
                Path(path).unlink(missing_ok=True)
    return failures


# Codes that reject the type itself. ORC0221 is also raised for fills and
# byte strings, so a length inside a type is recognized from the source.
_ALIAS_TYPE_CODES = {"ORC0203", "ORC0204", "ORC0232", "ORC0237"}
_ALIAS_LENGTH = re.compile(r"\^(?:0(?!\d)|65537|65544|\()")
_ALIAS_BUILTINS = {"Int", "Bool", "Word", "Mod"}
_ALIAS_KEYWORDS = {
    "spec", "module", "edition", "let", "for", "if", "with", "test", "else", "use", "type", "in",
    "as", "return", "little", "big", "true", "false",
}
_ALIAS_IDENT = re.compile(r"[A-Za-z_][A-Za-z0-9_]*")
_ALIAS_DECL = re.compile(r"^[ \t]*type\s+([A-Za-z_][A-Za-z0-9_]*)\s*=\s*([^;]*);", re.M)
_ALIAS_PARAM = re.compile(r"\b([A-Za-z_][A-Za-z0-9_]*)\s+in\s*\{")


def _blank_comments(source: str) -> str:
    out = []
    index = 0
    while index < len(source):
        if source.startswith("//", index):
            end = source.find("\n", index)
            if end < 0:
                end = len(source)
            out.append(" " * (end - index))
            index = end
            continue
        out.append(source[index])
        index += 1
    return "".join(out)


def _match_delim(source: str, index: int, open_c: str, close_c: str):
    depth = 0
    while index < len(source):
        if source[index] == open_c:
            depth += 1
        elif source[index] == close_c:
            depth -= 1
            if depth == 0:
                return index + 1
        index += 1
    return None


def _parse_alias_type(source: str, index: int):
    """End index of a type expression at `index`, or None."""
    index = _skip_space(source, index)
    if index >= len(source):
        return None
    if source[index] == "(":
        end = _match_delim(source, index, "(", ")")
        if end is None:
            return None
        inner = index + 1
        saw = False
        while True:
            inner = _skip_space(source, inner)
            if inner < len(source) and source[inner] == ")":
                return end if saw else None
            nxt = _parse_alias_type(source, inner)
            if nxt is None or nxt > end:
                return None
            saw = True
            inner = _skip_space(source, nxt)
            if inner < len(source) and source[inner] == ",":
                inner += 1
                continue
            if inner < len(source) and source[inner] == ")":
                return end
            return None
    match = _ALIAS_IDENT.match(source, index)
    if match is None or match.group() in _ALIAS_KEYWORDS:
        return None
    cursor = match.end()
    if cursor < len(source) and source[cursor] == "[":
        end = _match_delim(source, cursor, "[", "]")
        if end is None:
            return None
        cursor = end
    caret = _skip_space(source, cursor)
    if caret < len(source) and source[caret] == "^":
        start = _skip_space(source, caret + 1)
        if start < len(source) and source[start] == "(":
            end = _match_delim(source, start, "(", ")")
            if end is None:
                return None
            cursor = end
        elif start < len(source) and source[start].isdigit():
            cursor = start + 1
            if source[start] == "0" and cursor < len(source) and source[cursor] in "xX":
                cursor += 1
                hexed = cursor
                while cursor < len(source) and source[cursor] in "0123456789abcdefABCDEF":
                    cursor += 1
                if cursor == hexed:
                    return None
            else:
                while cursor < len(source) and source[cursor].isdigit():
                    cursor += 1
        else:
            length = _ALIAS_IDENT.match(source, start)
            if length is None or length.group() in _ALIAS_KEYWORDS:
                return None
            cursor = length.end()
    return cursor


def _alias_type_exprs(source: str) -> list[str]:
    found = []

    def add(start: int, end: int) -> None:
        expr = re.sub(r"\s+", " ", source[start:end].strip())
        if expr and expr not in found:
            found.append(expr)

    for match in re.finditer(r"type\s+[A-Za-z_][A-Za-z0-9_]*\s*=\s*", source):
        end = _parse_alias_type(source, match.end())
        if end:
            add(match.end(), end)
    for match in re.finditer(r"->\s*", source):
        end = _parse_alias_type(source, match.end())
        if end:
            add(match.end(), end)
    for match in re.finditer(r"(?<![:\w])as\s+(?:little\s+|big\s+)?", source):
        end = _parse_alias_type(source, match.end())
        if end:
            add(match.end(), end)
    for match in re.finditer(r"(?<!:):(?!:)", source):
        end = _parse_alias_type(source, match.end())
        if end:
            add(match.end(), end)
    for match in re.finditer(r"\bin\s*\{", source):
        cursor = match.end()
        while True:
            cursor = _skip_space(source, cursor)
            if cursor < len(source) and source[cursor] == "}":
                break
            end = _parse_alias_type(source, cursor)
            if end is None:
                break
            add(cursor, end)
            cursor = _skip_space(source, end)
            if cursor < len(source) and source[cursor] == ",":
                cursor += 1
                continue
            break
    return found


def _alias_decls(source: str):
    return [(match.group(1), match.group(0).strip(), match.group(2)) for match in _ALIAS_DECL.finditer(source)]


def _alias_deps(expr: str, decls) -> list[str]:
    by_name = {name: (line, rhs) for name, line, rhs in decls}
    wanted = set()
    stack = [ident for ident in _ALIAS_IDENT.findall(expr) if ident not in _ALIAS_BUILTINS and ident in by_name]
    while stack:
        name = stack.pop()
        if name in wanted:
            continue
        wanted.add(name)
        for ident in _ALIAS_IDENT.findall(by_name[name][1]):
            if ident not in _ALIAS_BUILTINS and ident in by_name and ident not in wanted:
                stack.append(ident)
    return [by_name[name][0] for name, _, _ in decls if name in wanted]


def _alias_program(deps: list[str], expr: str) -> str:
    lines = [line.strip() for line in deps]
    lines.append(f"type AliasTarget = {expr};")
    lines.append("spec use_alias(x: AliasTarget) -> AliasTarget { x }")
    body = "\n".join("  " + line for line in lines)
    return f"edition 2026;\nmodule aliasuse {{\n{body}\n}}\n"


def _alias_must_cover(codes: list[str], source: str) -> bool:
    found = set(codes)
    if found & _ALIAS_TYPE_CODES:
        return True
    return "ORC0221" in found and _ALIAS_LENGTH.search(source) is not None


def alias_uses(rust_compiler: Path, c_compiler: Path) -> int:
    """Every rejected type in an invalid fixture is also used behind an alias.

    The direct path is the fixture itself, compared in `main`: a silent accept
    there fails. This path writes `type AliasTarget = <type>;` and a use of
    `AliasTarget`, then requires Rust's full stderr. A silent accept of the
    alias fails. Type parameters stay in the fixture; a module-level alias
    cannot see them. Declared names the type mentions are copied first.
    """
    failures = 0
    seen = {}
    rejected = 0
    for relative in INVALID:
        text = _blank_comments((FIXTURES / relative).read_text(encoding="utf-8"))
        rust_fixture = run(rust_compiler, ["check", str(FIXTURES / relative)])
        fixture_codes = codes(rust_fixture.stderr)
        params = set(_ALIAS_PARAM.findall(text))
        decls = _alias_decls(text)
        covered = False
        for expr in _alias_type_exprs(text):
            if any(ident in params for ident in _ALIAS_IDENT.findall(expr)):
                continue
            source = _alias_program(_alias_deps(expr, decls), expr)
            if source in seen:
                if seen[source]:
                    covered = True
                continue
            path = None
            try:
                with tempfile.NamedTemporaryFile("w", suffix=".or", delete=False, encoding="utf-8") as handle:
                    handle.write(source)
                    path = handle.name
                rust = run(rust_compiler, ["check", path])
                rust_err = rust.stderr.replace(path, "FILE")
                rejects = rust.returncode != 0
                seen[source] = rejects
                if not rejects:
                    continue
                covered = True
                rejected += 1
                c_result = run(c_compiler, ["check", path])
                c_err = c_result.stderr.replace(path, "FILE")
                if c_result.returncode == 0 or rust_err != c_err:
                    failures += 1
                    print(f"FAIL alias use {relative}: {expr}")
                    print(f"  rust {codes(rust_err)} exit {rust.returncode}")
                    print(f"  c    {codes(c_err)} exit {c_result.returncode}")
                    if rust_err != c_err:
                        print("  rust stderr:", rust_err)
                        print("  c stderr:", c_err)
            finally:
                if path is not None:
                    Path(path).unlink(missing_ok=True)
        if _alias_must_cover(fixture_codes, text) and not covered:
            failures += 1
            print(f"FAIL alias coverage {relative}")
    if rejected == 0:
        failures += 1
        print("FAIL alias uses found no rejected type")
    elif failures == 0:
        print(f"ok   alias uses: {rejected} rejected types match Rust")
    return failures


if __name__ == "__main__":
    raise SystemExit(main())
