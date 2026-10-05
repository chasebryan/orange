//! External conformance evidence for proposed Orange 2026 S3v word parameters.
//! Every fixture and generated program runs through the real compiler twice.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

const SPECIFICATION: &str = include_str!("../../../../docs/UNIVERSAL_2026.md");
const RULES: [&str; 7] = [
    "S3V-01", "S3V-02", "S3V-03", "S3V-04", "S3V-05", "S3V-06", "S3V-07",
];
const CLI: u8 = 1;
const GENERATED: u8 = 2;

struct Case {
    fixture: &'static str,
    rules: &'static [&'static str],
    specs: &'static [&'static str],
    value: fn() -> String,
    tests: &'static [&'static str],
    errors: &'static [&'static str],
}

fn empty() -> String {
    String::new()
}
fn compat() -> String {
    "compat::result: (Word[8], Mod[3], Word[64]) = (0x02, 0, 0x0000000000000004)\n".into()
}
fn keccak() -> String {
    "keccak::sha3_256_abc: Word[8]^32 = [0x3a, 0x98, 0x5d, 0xa7, 0x4f, 0xe2, 0x25, 0xb2, 0x04, \
     0x5c, 0x17, 0x2d, 0x6b, 0xd3, 0x90, 0xbd, 0x85, 0x5f, 0x08, 0x6e, 0x3e, 0x9d, 0x52, 0x5b, \
     0x46, 0xbf, 0xe2, 0x45, 0x11, 0x43, 0x15, 0x32]\n\
     keccak::keccak_f200: Word[8]^25 = [0x3c, 0x28, 0x26, 0x84, 0x1c, 0xb3, 0x5c, 0x17, 0x1e, \
     0xaa, 0xe9, 0xb8, 0x11, 0x13, 0x4c, 0xea, 0xa3, 0x85, 0x2c, 0x69, 0xd2, 0xc5, 0xab, 0xaf, \
     0xea]\n"
        .into()
}

const CASES: &[Case] = &[
    Case {
        fixture: "valid-keccak.or",
        rules: &["S3V-01", "S3V-03", "S3V-05", "S3V-06", "S3V-07"],
        specs: &["sha3_256_abc", "keccak_f200"],
        value: keccak,
        tests: &[
            "FIPS 202 example: SHA3-256 of abc",
            "XKCP Keccak-f[200] of the all-zero state",
        ],
        errors: &[],
    },
    Case {
        fixture: "valid-compat.or",
        rules: &["S3V-01", "S3V-05", "S3V-07"],
        specs: &["result"],
        value: compat,
        tests: &["older forms keep their values beside a word parameter"],
        errors: &[],
    },
    Case {
        fixture: "invalid-vocabulary.or",
        rules: &["S3V-01"],
        specs: &[],
        value: empty,
        tests: &[],
        errors: &["ORC0243"],
    },
    Case {
        fixture: "invalid-literal.or",
        rules: &["S3V-02"],
        specs: &[],
        value: empty,
        tests: &[],
        errors: &["ORC0207"],
    },
    Case {
        fixture: "invalid-affine.or",
        rules: &["S3V-03"],
        specs: &[],
        value: empty,
        tests: &[],
        errors: &["ORC0243"],
    },
    Case {
        fixture: "invalid-corner.or",
        rules: &["S3V-03"],
        specs: &[],
        value: empty,
        tests: &[],
        errors: &["ORC0223"],
    },
    Case {
        fixture: "invalid-length.or",
        rules: &["S3V-04"],
        specs: &[],
        value: empty,
        tests: &[],
        errors: &["ORC0243"],
    },
    Case {
        fixture: "invalid-call.or",
        rules: &["S3V-05"],
        specs: &[],
        value: empty,
        tests: &[],
        errors: &["ORC0238"],
    },
    Case {
        fixture: "invalid-type.or",
        rules: &["S3V-05"],
        specs: &[],
        value: empty,
        tests: &[],
        errors: &["ORC0241"],
    },
];

const GENERATED_EVIDENCE: &[(&str, &[&str])] = &[
    (
        "s3v_widths_and_vocabulary_are_exact",
        &["S3V-01", "S3V-02", "S3V-07"],
    ),
    (
        "s3v_sizes_lengths_and_calls_are_exact",
        &["S3V-03", "S3V-04", "S3V-05"],
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

#[test]
fn s3v_rule_index_is_exact_and_covered() {
    let documented: Vec<_> = SPECIFICATION
        .lines()
        .filter_map(|line| {
            let first = line
                .strip_prefix("| ")?
                .split('|')
                .next()?
                .trim()
                .trim_matches(char::from(96));
            first.starts_with("S3V-").then_some(first)
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
    let source = include_str!("s3v_conformance.rs");
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
        .filter(|line| line.starts_with("| ") && line.contains("S3V-"))
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
fn s3v_fixture_inventory_and_outputs_are_exact() {
    let directory = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/s3v");
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
    assert_eq!(actual.len(), 9);
    for case in CASES {
        let path = directory.join(case.fixture);
        for command in ["check", "eval", "test"] {
            let mut arguments = vec![command];
            if command == "eval" {
                for spec in case.specs {
                    arguments.push("--spec");
                    arguments.push(spec);
                }
            }
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
fn s3v_widths_and_vocabulary_are_exact() {
    let widths = "edition 2026; module widths { \
        spec inc[W: Word](x: W) -> W { x + 1 } \
        spec go() -> (Word[8], Word[16], Word[32], Word[64]) { \
          (inc[Word[8]](254), inc[Word[16]](255), inc[Word[32]](256), inc[Word[64]](256)) \
        } }";
    success(
        &source_twice(&["eval", "--spec", "go"], widths),
        "widths::go: (Word[8], Word[16], Word[32], Word[64]) = (0xff, 0x0100, 0x00000101, \
         0x0000000000000101)\n",
    );
    let second = "edition 2026; module widths { \
        spec two[A: Word, B: Word](x: A) -> A { x } }";
    failure(&source_twice(&["check"], second), &["ORC0243"]);
    let mixed = "edition 2026; module widths { \
        spec mixed[W: Word, K in {Word[8]}](x: W) -> W { x } }";
    failure(&source_twice(&["check"], mixed), &["ORC0243"]);
    let literal = "edition 2026; module widths { \
        spec bad[W: Word](x: W) -> W { x + 256 } \
        spec kept() -> Int { 1 } }";
    failure(
        &source_twice(&["eval", "--spec", "kept"], literal),
        &["ORC0207"],
    );
    let retained = "edition 2026; module old { \
        spec add[n in 2..4](x: Mod[n]) -> Mod[n] { x + 1 } \
        spec id[K in {Word[8], Word[64]}](x: K) -> K { x + 1 } \
        spec go() -> (Mod[3], Word[8], Word[64]) { \
          (add[3](2), id[Word[8]](7), id[Word[64]](8)) \
        } }";
    success(
        &source_twice(&["eval", "--spec", "go"], retained),
        "old::go: (Mod[3], Word[8], Word[64]) = (0, 0x08, 0x0000000000000009)\n",
    );
}

#[test]
fn s3v_sizes_lengths_and_calls_are_exact() {
    let corner = "edition 2026; module sizes { \
        spec pick[W: Word, n in 1..5](a: W^4) -> W { \
          for i in 0..n with s: W = 0 { s ^ a[i] } \
        } \
        spec go() -> Word[8] { pick[Word[8], 3]([1, 2, 3, 4]) } }";
    success(
        &source_twice(&["eval", "--spec", "go"], corner),
        "sizes::go: Word[8] = 0x00\n",
    );
    let divided = "edition 2026; module sizes { \
        spec quot[W: Word, n in 2..8](a: W^4) -> W { a[n / 2] } }";
    failure(&source_twice(&["check"], divided), &["ORC0243"]);
    let product = "edition 2026; module sizes { \
        spec bad[W: Word, n in 1..3, m in 1..3](a: W^4) -> W { a[n * m] } }";
    failure(&source_twice(&["check"], product), &["ORC0243"]);
    let wide = "edition 2026; module sizes { \
        spec pick[W: Word, n in 1..6](a: W^4) -> W { \
          for i in 0..n with s: W = 0 { s ^ a[i] } \
        } }";
    failure(&source_twice(&["check"], wide), &["ORC0223"]);
    let length = "edition 2026; module sizes { \
        spec fill[W: Word, n in 1..4](x: W) -> W^n { [x; n] } }";
    failure(&source_twice(&["check"], length), &["ORC0243"]);
    let outside = "edition 2026; module sizes { \
        spec idw[W: Word, n in 1..4](x: W) -> W { x } \
        spec use() -> Word[8] { idw[Word[8], 8](1) } }";
    failure(&source_twice(&["check"], outside), &["ORC0238"]);
    let foreign = "edition 2026; module sizes { \
        spec idw[W: Word](x: W) -> W { x } \
        spec use() -> Word[8] { idw[Int](1) } }";
    failure(&source_twice(&["check"], foreign), &["ORC0241"]);
    let brackets = "edition 2026; module sizes { \
        spec idw[W: Word](x: W) -> W { x } \
        spec use(x: Word[8]) -> Word[8] { idw(x) } }";
    failure(&source_twice(&["check"], brackets), &["ORC0239"]);
}
