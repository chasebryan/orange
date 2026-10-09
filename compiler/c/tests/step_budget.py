#!/usr/bin/env python3
"""Compare reference-evaluation step budgets with the Rust frontend.

Each completing case is also run one step under that budget. A charge that
would pass the limit is not recorded, so the two sides must agree on the
exit status, stdout, and the full ORC0301 text. Cases the C frontend cannot
run yet are listed in SKIPS with a reason; the list is printed and must not
be silent. S3p removes the `[0; 65536]` entry and runs that case.
"""

import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
FIXTURES = ROOT / "compiler" / "fixtures"
RUST_DEFAULT = ROOT / "compiler" / "target" / "debug" / "orangec"

# S3p runs every case, including `[0; 65536]`. A later skip needs a reason,
# and the list must stay empty: this slice is not done while an entry remains.
SKIPS = []


def run(compiler: Path, args: list[str]) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        [str(compiler), *args],
        cwd=ROOT,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
    )


def fail(message: str) -> None:
    print(f"FAIL {message}")


def first_line(text: str) -> str:
    return text.splitlines()[0] if text else ""


def compare(name: str, rust: subprocess.CompletedProcess[str], c_result: subprocess.CompletedProcess[str], *, first_only: bool = False) -> bool:
    rust_err = first_line(rust.stderr) if first_only else rust.stderr
    c_err = first_line(c_result.stderr) if first_only else c_result.stderr
    if rust.returncode != c_result.returncode or rust.stdout != c_result.stdout or rust_err != c_err:
        fail(name)
        print(f"  rust exit {rust.returncode} c exit {c_result.returncode}")
        if rust.stdout != c_result.stdout:
            print("  stdout mismatch")
            print("  rust:", rust.stdout[:400])
            print("  c:   ", c_result.stdout[:400])
        if rust_err != c_err:
            print("  stderr mismatch")
            print("  rust:", rust_err[:800])
            print("  c:   ", c_err[:800])
        return False
    print(f"ok   {name}")
    return True


def eval_case(rust_bin: Path, c_bin: Path, name: str, source: Path, steps: int | None, *, stats: bool = False) -> bool:
    args = ["eval"]
    if steps is not None:
        args.extend(["--steps", str(steps)])
    if stats:
        args.append("--stats")
    args.append(str(source))
    return compare(name, run(rust_bin, args), run(c_bin, args))


def write_source(directory: Path, name: str, body: str) -> Path:
    path = directory / name
    path.write_text(body, encoding="utf-8")
    return path


def array_literal(length: int) -> str:
    elements = ", ".join(["1"] * length)
    return (
        "edition 2026;\n"
        "module m {\n"
        f"  spec a() -> Word[8]^{length} {{ [{elements}] }}\n"
        "}\n"
    )


def array_fill(length: int) -> str:
    return (
        "edition 2026;\n"
        "module m {\n"
        f"  spec a() -> Word[8]^{length} {{ [0; {length}] }}\n"
        "}\n"
    )


def main() -> int:
    if len(sys.argv) < 2:
        print("usage: step_budget.py <c-orangec> [rust-orangec]", file=sys.stderr)
        return 2
    c_bin = Path(sys.argv[1])
    if not c_bin.is_absolute():
        c_bin = Path.cwd() / c_bin
    rust_bin = Path(sys.argv[2]) if len(sys.argv) > 2 else RUST_DEFAULT
    if not rust_bin.is_file() or not c_bin.is_file():
        print(f"missing compiler rust={rust_bin} c={c_bin}", file=sys.stderr)
        return 2

    failures = 0
    if SKIPS or any(not reason.strip() for _, reason in SKIPS):
        fail("step-budget skip list must be empty, and any future entry needs a reason")
        return 1

    with tempfile.TemporaryDirectory(prefix="orangec-steps-") as temporary:
        directory = Path(temporary)
        edges = []
        for length in (1, 64, 65, 128, 129):
            edges.append((f"lit-{length}", write_source(directory, f"lit-{length}.or", array_literal(length))))
            edges.append((f"fill-{length}", write_source(directory, f"fill-{length}.or", array_fill(length))))
        updates = write_source(
            directory,
            "updates.or",
            "edition 2026;\n"
            "module m {\n"
            "  spec u() -> Word[8]^64 {\n"
            "    for i in 0..64 with s: Word[8]^64 = [0; 64] { s with [i] = 1 }\n"
            "  }\n"
            "}\n",
        )
        # A loop `for i in a..b` is rejected unless b <= 65536, so one
        # `0..524287` loop never evaluates. `for i in 0..N with s: Word[8] = 0 { s }`
        # costs 2 + 2N, and a `let` adds no step. Seven loops of 65536 and one of
        # 65528 are 1048576 steps; `~` is the next step and is dropped at the
        # default budget. The bulk program stops two steps earlier, then a
        # 256-element fill whose bulk charge of 4 does not fit.
        def loop_expr(bound: int) -> str:
            return f"for i in 0..{bound} with s: Word[8] = 0 {{ s }}"

        full_loops = "\n".join(
            f"    let a{index}: Word[8] = {loop_expr(65536)};" for index in range(1, 8)
        )
        tilde = write_source(
            directory,
            "tilde.or",
            "edition 2026;\n"
            "module m {\n"
            "  spec a() -> Word[8] {\n"
            f"{full_loops}\n"
            f"    ~({loop_expr(65528)})\n"
            "  }\n"
            "}\n",
        )
        bulk = write_source(
            directory,
            "bulk.or",
            "edition 2026;\n"
            "module m {\n"
            "  spec a() -> Word[8]^256 {\n"
            f"{full_loops}\n"
            f"    let unused: Word[8] = {loop_expr(65527)};\n"
            "    [0xa8; 256]\n"
            "  }\n"
            "}\n",
        )
        chacha = FIXTURES / "s3n" / "valid-chacha20.or"
        before = FIXTURES / "steps" / "limit-before.or"
        while_eval = FIXTURES / "steps" / "limit-while.or"
        int_limit = FIXTURES / "steps" / "int-limit.or"
        called = write_source(
            directory,
            "called.or",
            "edition 2026;\n"
            "module m {\n"
            "  spec s(x: Int) -> Int { x * x }\n"
            "  spec over() -> Int { s(s(s(s(s(s(s(s(s(s(s(s(s(s(2)))))))))))))) }\n"
            "}\n",
        )

        def check(ok: bool) -> None:
            nonlocal failures
            if not ok:
                failures += 1

        for name, path in edges:
            rust_stats = run(rust_bin, ["eval", "--stats", str(path)])
            total_line = [line for line in rust_stats.stderr.splitlines() if line.startswith("total:")]
            if rust_stats.returncode != 0 or len(total_line) != 1:
                fail(f"{name} rust did not finish")
                failures += 1
                continue
            total = int(total_line[0].split()[1])
            check(eval_case(rust_bin, c_bin, f"{name} at {total}", path, total, stats=True))
            check(eval_case(rust_bin, c_bin, f"{name} at {total - 1}", path, total - 1))

        wide_fill = write_source(directory, "fill-65536.or", array_fill(65536))
        check(eval_case(rust_bin, c_bin, "fill 65536 at 1025", wide_fill, 1025, stats=True))
        check(eval_case(rust_bin, c_bin, "fill 65536 at 1024", wide_fill, 1024))

        def one_under(label: str, path: Path, budget: int, names: set[str]) -> None:
            stats = run(rust_bin, ["eval", "--steps", str(budget), "--stats", str(path)])
            if stats.returncode != 0:
                fail(f"{label} rust did not finish")
                failures += 1
                return
            check(eval_case(rust_bin, c_bin, f"{label} at {budget}", path, budget, stats=True))
            spent = 0
            for line in stats.stderr.splitlines():
                if line.startswith("total:"):
                    break
                head, _, rest = line.partition(": ")
                count_text = rest.split(" ", 1)[0]
                if not count_text.isdigit():
                    continue
                count = int(count_text)
                short = head.split("::")[-1]
                if short in names:
                    check(eval_case(rust_bin, c_bin, f"{head} one under", path, spent + count - 1))
                spent += count

        one_under("lengths", FIXTURES / "s3p" / "valid-lengths.or", 2097152, {"pepin", "halves", "words"})
        one_under("rfc8439", FIXTURES / "s3p" / "valid-rfc8439.or", 1048576, {"a2_2"})
        check(eval_case(rust_bin, c_bin, "updates at 323", updates, 323, stats=True))
        check(eval_case(rust_bin, c_bin, "updates at 322", updates, 322))
        check(eval_case(rust_bin, c_bin, "chacha20 at 14595", chacha, 14595, stats=True))
        check(eval_case(rust_bin, c_bin, "chacha20 at 14594", chacha, 14594))
        check(eval_case(rust_bin, c_bin, "array at 2", while_eval, 2))
        check(eval_case(rust_bin, c_bin, "array at 5", while_eval, 5))
        check(eval_case(rust_bin, c_bin, "array at 6", while_eval, 6, stats=True))
        check(eval_case(rust_bin, c_bin, "limit 1 before second", before, 1))
        check(eval_case(rust_bin, c_bin, "limit 3 while evaluating", while_eval, 3))
        check(eval_case(rust_bin, c_bin, "int limit in a loop", int_limit, None))
        check(eval_case(rust_bin, c_bin, "int limit in a call", called, None))

        # The loops cost 1048576 steps and `~` is one more. The default budget
        # stops on that extra step and does not record it.
        check(eval_case(rust_bin, c_bin, "tilde at default 1048576", tilde, None))
        check(eval_case(rust_bin, c_bin, "tilde at 1048577", tilde, 1048577, stats=True))

        # 1048574 steps, then a literal, then a bulk charge of 4 that does not
        # fit in the one remaining step of the default budget.
        check(eval_case(rust_bin, c_bin, "bulk at default 1048576", bulk, None))
        check(eval_case(rust_bin, c_bin, "bulk at 1048578", bulk, 1048578))
        check(eval_case(rust_bin, c_bin, "bulk at 1048579", bulk, 1048579, stats=True))

    # CLI errors: the first line matches Rust. C keeps its shorter usage.
    cli = [
        ("steps 0", ["eval", "--steps", "0", "x.or"], "orangec: option `--steps` takes a number of steps from 1 through 1073741824"),
        ("steps 01", ["eval", "--steps", "01", "x.or"], "orangec: option `--steps` takes a number of steps from 1 through 1073741824"),
        ("steps high", ["eval", "--steps", "1073741825", "x.or"], "orangec: option `--steps` takes a number of steps from 1 through 1073741824"),
        ("steps missing", ["eval", "--steps"], "orangec: option `--steps` requires a value"),
        ("steps twice", ["eval", "--steps", "2", "--steps", "3", "x.or"], "orangec: option `--steps` may be specified at most once"),
        ("steps on check", ["check", "--steps", "5", "x.or"], "orangec: option `--steps` applies only to eval, test, and replay"),
        ("stats on lex", ["lex", "--stats", "x.or"], "orangec: option `--stats` applies only to eval, test, and replay"),
    ]
    for name, args, line in cli:
        rust = run(rust_bin, args)
        c_result = run(c_bin, args)
        if rust.returncode != 2 or c_result.returncode != 2 or first_line(rust.stderr) != line or first_line(c_result.stderr) != line:
            fail(name)
            print(f"  rust: {first_line(rust.stderr)!r} exit {rust.returncode}")
            print(f"  c:    {first_line(c_result.stderr)!r} exit {c_result.returncode}")
            failures += 1
        else:
            print(f"ok   {name}")

    if failures:
        print(f"{failures} step-budget comparison(s) failed")
        return 1
    print("step-budget comparison passed")
    return 0


if __name__ == "__main__":
    sys.exit(main())
