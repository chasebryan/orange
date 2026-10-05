#!/usr/bin/env python3
"""Compare the standalone C compiler with the Rust frontend on the S3a–S3k fixtures."""

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

    failures += loop_bound_magnitude(rust_compiler, c_compiler)
    failures += index_chain_and_loop_recovery(rust_compiler, c_compiler)
    failures += conversion_targets(rust_compiler, c_compiler)
    failures += logical_operators(rust_compiler, c_compiler)
    failures += residue_modules(rust_compiler, c_compiler)

    if failures:
        print(f"{failures} failure(s)")
        return 1
    print("differential comparison passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
