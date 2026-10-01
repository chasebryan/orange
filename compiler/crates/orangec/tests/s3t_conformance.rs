//! External conformance evidence for proposed Orange 2026 S3t static moduli.
//! Every fixture and generated program runs through the real compiler twice.
//! Expected large integers use an independent decimal arithmetic model.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

const SPECIFICATION: &str = include_str!("../../../../docs/STATIC_MODULI_2026.md");
const RULES: [&str; 10] = [
    "S3T-01", "S3T-02", "S3T-03", "S3T-04", "S3T-05", "S3T-06", "S3T-07", "S3T-08", "S3T-09",
    "S3T-10",
];
const CLI: u8 = 1;
const GENERATED: u8 = 2;

struct Case {
    fixture: &'static str,
    rules: &'static [&'static str],
    value: fn() -> String,
    tests: &'static [&'static str],
    errors: &'static [&'static str],
}

fn empty() -> String {
    String::new()
}
fn rings() -> String {
    "rings::result: (Mod[3], Mod[4], Mod[7]) = (1, 3, 0)\n".into()
}
fn large() -> String {
    format!(
        "large::result: Mod[(1 << 255) - 19] = {}\n",
        power_minus(255, 20)
    )
}
fn aggregates() -> String {
    "aggregates::result: (Mod[3], Mod[4]^2) = (0, [3, 1])\n".into()
}
fn calls() -> String {
    "calls::result: (Mod[3], Mod[4]) = (2, 0)\n".into()
}
fn program() -> String {
    "program::result: (Mod[7], Mod[11]) = (0, 10)\n".into()
}

const CASES: &[Case] = &[
    Case {
        fixture: "valid-rings.or",
        rules: &["S3T-01", "S3T-03", "S3T-04", "S3T-05", "S3T-07", "S3T-10"],
        value: rings,
        tests: &[
            "one arithmetic definition retains each modulus",
            "negative integers become least residues in each instance",
            "composite rings keep total inverse semantics",
            "expected results and typed arguments select exact instances",
        ],
        errors: &[],
    },
    Case {
        fixture: "valid-large.or",
        rules: &["S3T-01", "S3T-02", "S3T-03", "S3T-04", "S3T-07", "S3T-10"],
        value: large,
        tests: &[
            "large moduli are exact at all three static sizes",
            "the same written conversion is specialized separately",
        ],
        errors: &[],
    },
    Case {
        fixture: "valid-aggregates.or",
        rules: &["S3T-03", "S3T-04", "S3T-05", "S3T-06", "S3T-07", "S3T-10"],
        value: aggregates,
        tests: &[
            "array elements and loop bindings retain the instance domain",
            "tuples retain scalar and array modular types",
            "slice updates preserve specialized residue types",
        ],
        errors: &[],
    },
    Case {
        fixture: "valid-calls.or",
        rules: &["S3T-03", "S3T-04", "S3T-05", "S3T-07", "S3T-10"],
        value: calls,
        tests: &[
            "explicit type arguments use the caller's static sizes",
            "branch annotations are resolved in their own instance",
        ],
        errors: &[],
    },
    Case {
        fixture: "static_ring.or",
        rules: &["S3T-01", "S3T-08", "S3T-10"],
        value: empty,
        tests: &[],
        errors: &[],
    },
    Case {
        fixture: "valid-program.or",
        rules: &["S3T-03", "S3T-04", "S3T-05", "S3T-08", "S3T-10"],
        value: program,
        tests: &[
            "imported signatures use the callee's selected modulus",
            "argument fitting across modules preserves exact domains",
        ],
        errors: &[],
    },
    Case {
        fixture: "invalid-scope.or",
        rules: &["S3T-01", "S3T-02", "S3T-10"],
        value: empty,
        tests: &[],
        errors: &["ORC0232", "ORC0232", "ORC0232", "ORC0232"],
    },
    Case {
        fixture: "invalid-bounds.or",
        rules: &["S3T-02", "S3T-03", "S3T-09", "S3T-10"],
        value: empty,
        tests: &[],
        errors: &["ORC0232", "ORC0232", "ORC0232"],
    },
    Case {
        fixture: "invalid-domains.or",
        rules: &["S3T-04", "S3T-05", "S3T-10"],
        value: empty,
        tests: &[],
        errors: &["ORC0214", "ORC0214", "ORC0214"],
    },
    Case {
        fixture: "invalid-constants.or",
        rules: &["S3T-01", "S3T-02", "S3T-03", "S3T-10"],
        value: empty,
        tests: &[],
        errors: &["ORC0232", "ORC0232", "ORC0232", "ORC0232"],
    },
];

const GENERATED_EVIDENCE: &[(&str, &[&str])] = &[
    (
        "s3t_large_moduli_and_boundaries_are_exact",
        &[
            "S3T-01", "S3T-02", "S3T-03", "S3T-04", "S3T-07", "S3T-09", "S3T-10",
        ],
    ),
    (
        "s3t_fitting_and_aggregate_domains_are_exact",
        &["S3T-03", "S3T-04", "S3T-05", "S3T-06", "S3T-07", "S3T-10"],
    ),
    (
        "s3t_scopes_operators_and_invalid_instances_fail_closed",
        &["S3T-01", "S3T-02", "S3T-03", "S3T-05", "S3T-09", "S3T-10"],
    ),
    (
        "s3t_step_and_instance_limits_remain_bounded",
        &["S3T-03", "S3T-04", "S3T-09", "S3T-10"],
    ),
];

fn run_twice(arguments: &[&str], path: &Path) -> Output {
    let run = || {
        Command::new(env!("CARGO_BIN_EXE_orangec"))
            .args(arguments)
            .arg(path)
            .output()
            .unwrap()
    };
    let first = run();
    let second = run();
    assert_eq!(first.status.code(), second.status.code(), "{path:?}");
    assert_eq!(first.stdout, second.stdout, "{path:?}");
    assert_eq!(first.stderr, second.stderr, "{path:?}");
    first
}

fn source_twice(arguments: &[&str], source: &str) -> Output {
    let run = || {
        let mut child = Command::new(env!("CARGO_BIN_EXE_orangec"))
            .args(arguments)
            .arg("-")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        child
            .stdin
            .take()
            .unwrap()
            .write_all(source.as_bytes())
            .unwrap();
        child.wait_with_output().unwrap()
    };
    let first = run();
    let second = run();
    assert_eq!(first.status.code(), second.status.code(), "{source}");
    assert_eq!(first.stdout, second.stdout, "{source}");
    assert_eq!(first.stderr, second.stderr, "{source}");
    first
}

fn success(output: &Output, expected: &str) {
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.stderr, b"");
    assert_eq!(String::from_utf8_lossy(&output.stdout), expected);
}

fn codes(output: &Output) -> Vec<&str> {
    std::str::from_utf8(&output.stderr)
        .unwrap()
        .lines()
        .filter_map(|line| {
            line.strip_prefix("error[")?
                .split_once(']')
                .map(|(code, _)| code)
        })
        .collect()
}

fn failure(output: &Output, expected_codes: &[&str]) {
    assert_eq!(
        output.status.code(),
        Some(1),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        output.stdout.is_empty(),
        "a rejected program returned partial values"
    );
    assert_eq!(
        codes(output),
        expected_codes,
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn report(titles: &[&str]) -> String {
    let mut result = String::new();
    for title in titles {
        result.push_str(&format!("test \"{title}\" ... ok\n"));
    }
    match titles.len() {
        0 => result.push_str("0 tests: 0 passed, 0 failed\n"),
        1 => result.push_str("1 test: 1 passed, 0 failed\n"),
        n => result.push_str(&format!("{n} tests: {n} passed, 0 failed\n")),
    }
    result
}

/// Decimal arithmetic intentionally does not share the compiler's binary limbs.
fn power_minus(bits: usize, subtract: u32) -> String {
    let mut digits = vec![1_u8];
    for _ in 0..bits {
        let mut carry = 0_u16;
        for digit in &mut digits {
            let value = u16::from(*digit) * 2 + carry;
            *digit = u8::try_from(value % 10).unwrap();
            carry = value / 10;
        }
        if carry != 0 {
            digits.push(u8::try_from(carry).unwrap());
        }
    }
    let mut remaining = subtract;
    let mut position = 0;
    let mut borrow = 0_i32;
    while remaining != 0 || borrow != 0 {
        let amount = i32::try_from(remaining % 10).unwrap() + borrow;
        let digit = &mut digits[position];
        let value = i32::from(*digit) - amount;
        *digit = u8::try_from(value.rem_euclid(10)).unwrap();
        borrow = i32::from(value < 0);
        remaining /= 10;
        position += 1;
    }
    while digits.len() > 1 && digits.last() == Some(&0) {
        digits.pop();
    }
    digits
        .iter()
        .rev()
        .map(|digit| char::from(b'0' + digit))
        .collect()
}

#[test]
fn s3t_rule_index_is_exact_and_covered() {
    let documented: Vec<_> = SPECIFICATION
        .lines()
        .filter_map(|line| {
            let first = line
                .strip_prefix("| ")?
                .split('|')
                .next()?
                .trim()
                .trim_matches(char::from(96));
            first.starts_with("S3T-").then_some(first)
        })
        .collect();
    assert_eq!(documented, RULES);
    let known: BTreeSet<_> = RULES.into_iter().collect();
    assert_eq!(known.len(), RULES.len());
    let mut covered = BTreeMap::<&str, u8>::new();
    for case in CASES {
        for rule in case.rules {
            assert!(known.contains(rule));
            *covered.entry(rule).or_default() |= CLI;
        }
    }
    let source = include_str!("s3t_conformance.rs");
    for (name, rules) in GENERATED_EVIDENCE {
        assert_eq!(source.matches(&format!("fn {name}(")).count(), 1);
        for rule in *rules {
            assert!(known.contains(rule));
            *covered.entry(rule).or_default() |= GENERATED;
        }
    }
    assert_eq!(covered.keys().copied().collect::<BTreeSet<_>>(), known);
    for line in SPECIFICATION
        .lines()
        .filter(|line| line.starts_with("| ") && line.contains("S3T-"))
    {
        let rule = line
            .split('|')
            .nth(1)
            .unwrap()
            .trim()
            .trim_matches(char::from(96));
        let layer = line.split('|').rev().nth(1).unwrap().trim();
        let expected = match layer {
            "CLI" => CLI,
            "CLI and generated CLI" => CLI | GENERATED,
            other => panic!("unknown evidence layer {other}"),
        };
        assert_eq!(covered[rule], expected, "{rule}");
    }
}

#[test]
fn s3t_fixture_inventory_and_outputs_are_exact() {
    let directory = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/s3t");
    let actual: BTreeSet<_> = fs::read_dir(&directory)
        .unwrap()
        .map(|entry| {
            let entry = entry.unwrap();
            assert!(entry.file_type().unwrap().is_file());
            entry.file_name().into_string().unwrap()
        })
        .collect();
    let expected: BTreeSet<_> = CASES.iter().map(|case| case.fixture.to_string()).collect();
    assert_eq!(actual, expected);
    assert_eq!(actual.len(), 10);
    for case in CASES {
        let path = directory.join(case.fixture);
        for command in ["check", "eval", "test"] {
            let arguments = if command == "eval" && !case.tests.is_empty() {
                vec![command, "--spec", "result"]
            } else {
                vec![command]
            };
            let output = run_twice(&arguments, &path);
            if !case.errors.is_empty() {
                failure(&output, case.errors);
            } else {
                let expected = match command {
                    "eval" => (case.value)(),
                    "test" => report(case.tests),
                    _ => String::new(),
                };
                success(&output, &expected);
            }
        }
    }
}

#[test]
fn s3t_large_moduli_and_boundaries_are_exact() {
    assert_eq!(power_minus(8, 1), "255");
    assert_eq!(power_minus(64, 19), "18446744073709551597");
    for bits in [2, 3, 8, 32, 64, 65, 127, 255, 256, 521] {
        let source = format!(
            "edition 2026; module generated {{ \
             spec prior[n in {bits}..{}]() -> Mod[(1 << n) - 1] {{ \
               let value: Int = -1; value as Mod[(1 << n) - 1] \
             }} }}",
            bits + 1
        );
        let modulus = if bits <= 64 {
            power_minus(bits, 1)
        } else {
            format!("(1 << {bits}) - 1")
        };
        let expected = format!(
            "generated::prior[{bits}]: Mod[{modulus}] = {}\n",
            power_minus(bits, 2)
        );
        success(&source_twice(&["eval"], &source), &expected);
    }
    for (modulus, subtract) in [("(1 << n)", 0), ("(1 << n) - 1", 1)] {
        let source = format!(
            "edition 2026; module edge {{ \
             spec prior[n in 521..522]() -> Mod[{modulus}] {{ \
               let x: Int = -1; x as Mod[{modulus}] \
             }} }}"
        );
        if subtract == 0 {
            failure(&source_twice(&["eval"], &source), &["ORC0232"]);
        } else {
            let expected = format!(
                "edge::prior[521]: Mod[(1 << 521) - 1] = {}\n",
                power_minus(521, 2)
            );
            success(&source_twice(&["eval"], &source), &expected);
        }
    }
    let unused = "edition 2026; module bad { \
        spec good() -> Int { 42 } \
        spec unused[n in 521..523]() -> Mod[(1 << n) - 1] { 0 } }";
    let output = source_twice(&["eval", "--spec", "good"], unused);
    failure(&output, &["ORC0232"]);
    assert!(String::from_utf8_lossy(&output.stderr).contains("unused[522]"));
}

#[test]
fn s3t_fitting_and_aggregate_domains_are_exact() {
    let source = "edition 2026; module fitting { \
        spec id[n in 2..6](x: Mod[n]) -> Mod[n] { x } \
        spec zero[n in 2..6]() -> Mod[n] { 0 } \
        spec rows[n in 2..6](x: Mod[n]^2) -> Mod[n]^2 { x } \
        spec explicit[T in {Mod[3]^2, Mod[4]^2}](x: T) -> T { x } \
        spec forward[n in 3..5](x: Mod[n]^2) -> Mod[n]^2 { explicit[Mod[n]^2](x) } \
        spec matrix[T in {Mod[3]^2, Mod[4]^2}](x: T) -> T^2 { [x; 2] } \
        spec rowcheck[n in 3..5]() -> Bool { \
          let row: Mod[n]^2 = [2, 1]; matrix[Mod[n]^2](row) == [row, row] \
        } \
        spec value() -> (Mod[3], Mod[4]^2) { \
          let scalar: Mod[3] = zero(); let array: Mod[4]^2 = [3, 2]; \
          (id(scalar), rows(forward[4](array))) \
        } \
        test \"size-specialized matrix row types\" { rowcheck[3]() && rowcheck[4]() } }";
    success(
        &source_twice(&["eval", "--spec", "value"], source),
        "fitting::value: (Mod[3], Mod[4]^2) = (0, [3, 2])\n",
    );
    success(
        &source_twice(&["test"], source),
        &report(&["size-specialized matrix row types"]),
    );
    let ambiguous = "edition 2026; module fitting { \
        spec zero[n in 2..4]() -> Mod[3 + (0 * n)] { 0 } \
        spec value() -> Mod[3] { zero() } }";
    failure(&source_twice(&["eval"], ambiguous), &["ORC0239"]);
    let mismatch = "edition 2026; module fitting { \
        spec id[n in 2..6](x: Mod[n]) -> Mod[n] { x } \
        spec value(x: Mod[7]) -> Mod[7] { id(x) } }";
    failure(&source_twice(&["check"], mismatch), &["ORC0238"]);
    let domains = "edition 2026; module fitting { \
        spec combine[n in 2..4](x: Mod[n], y: Mod[n + 1]) -> Mod[n] { x + y } }";
    failure(&source_twice(&["check"], domains), &["ORC0214"]);
}

#[test]
fn s3t_scopes_operators_and_invalid_instances_fail_closed() {
    for declaration in [
        "type Invalid = Mod[n];",
        "spec invalid(x: Int) -> Mod[x] { 0 }",
        "spec invalid[n in 2..4, T in {Mod[n]}]() -> T { 0 }",
        "spec invalid[n in 2..4]() -> Mod[missing + n] { 0 }",
        "spec invalid[n in 2..4]() -> Mod[n / 2] { 0 }",
        "spec invalid[n in 2..4]() -> Mod[n % 2] { 0 }",
        "spec invalid[n in 2..4]() -> Mod[16 >> n] { 0 }",
        "spec invalid[n in 2..4]() -> Mod[(1 << (0 - n)) + 2] { 0 }",
    ] {
        let source = format!("edition 2026; module scopes {{ {declaration} }}");
        let output = source_twice(&["eval"], &source);
        failure(&output, &["ORC0232"]);
    }
    let nonconstant = "edition 2026; module scopes { \
        spec identity[T in {Mod[3]}](x: T) -> T { x } \
        spec invalid(x: Int) -> Mod[3] { identity[Mod[x]](0) } }";
    let output = source_twice(&["check"], nonconstant);
    failure(&output, &["ORC0232"]);
}

#[test]
fn s3t_step_and_instance_limits_remain_bounded() {
    let source = "edition 2026; module budget { spec zeros[n in 2..258]() -> Mod[n] { 0 } }";
    let mut expected = String::new();
    for n in 2..258 {
        expected.push_str(&format!("budget::zeros[{n}]: Mod[{n}] = 0\n"));
    }
    success(
        &source_twice(&["eval", "--steps", "256"], source),
        &expected,
    );
    let exhausted = source_twice(&["eval", "--steps", "255"], source);
    failure(&exhausted, &["ORC0301"]);
    let too_many = "edition 2026; module budget { spec zeros[n in 2..259]() -> Mod[n] { 0 } }";
    failure(&source_twice(&["check"], too_many), &["ORC0238"]);
    let old = "edition 2026; module old { type P = Mod[7]; spec result() -> P { 3 * 5 } }";
    let new = "edition 2026; module new { spec result[n in 7..8]() -> Mod[n] { 3 * 5 } }";
    let old_output = source_twice(&["eval", "--stats"], old);
    let new_output = source_twice(&["eval", "--stats"], new);
    assert!(old_output.status.success());
    assert!(new_output.status.success());
    assert_eq!(old_output.stdout, b"old::result: Mod[7] = 1\n");
    assert_eq!(new_output.stdout, b"new::result[7]: Mod[7] = 1\n");
    let costs = |output: &Output| {
        String::from_utf8_lossy(&output.stderr)
            .split(": ")
            .last()
            .unwrap()
            .to_string()
    };
    assert_eq!(costs(&old_output), costs(&new_output));
}
