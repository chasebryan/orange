//! External conformance evidence for the proposed Orange 2026 S3w slice.
//! Every fixture runs through the real compiler twice.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const SPECIFICATION: &str = include_str!("../../../../docs/POSITIONS_2026.md");
const RULES: [&str; 6] = ["S3W-01", "S3W-02", "S3W-03", "S3W-04", "S3W-05", "S3W-06"];

struct Case {
    fixture: &'static str,
    rules: &'static [&'static str],
    value: &'static str,
    tests: &'static [&'static str],
    errors: &'static [&'static str],
}

const CASES: &[Case] = &[
    Case {
        fixture: "valid-quarter.or",
        rules: &["S3W-01", "S3W-02", "S3W-03", "S3W-06"],
        value: "quarter::result: Word[32]^4 = [0xea2a92f4, 0xcb1cf8ce, 0x4581472e, 0x5881c4bb]\n",
        tests: &["RFC 8439 section 2.1.1 at positions 0, 4, 8, 12"],
        errors: &[],
    },
    Case {
        fixture: "valid-mixed.or",
        rules: &["S3W-01", "S3W-02", "S3W-03"],
        value: "mixed::result: Int = 34\n",
        tests: &["a size and a position are chosen together"],
        errors: &[],
    },
    Case {
        fixture: "invalid-declaration.or",
        rules: &["S3W-02"],
        value: "",
        tests: &[],
        errors: &["ORC0243", "ORC0243"],
    },
    Case {
        fixture: "invalid-calls.or",
        rules: &["S3W-04"],
        value: "",
        tests: &[],
        errors: &["ORC0239", "ORC0243", "ORC0243"],
    },
    Case {
        fixture: "invalid-not-index.or",
        rules: &["S3W-03", "S3W-05"],
        value: "",
        tests: &[],
        errors: &["ORC0237", "ORC0226", "ORC0226", "ORC0237"],
    },
    Case {
        fixture: "invalid-syntax.or",
        rules: &["S3W-01"],
        value: "",
        tests: &[],
        errors: &["ORC0101"],
    },
];

fn orangec(arguments: &[&str], source: &Path) -> Output {
    let output = Command::new(env!("CARGO_BIN_EXE_orangec"))
        .args(arguments)
        .arg(source)
        .output()
        .unwrap();
    let again = Command::new(env!("CARGO_BIN_EXE_orangec"))
        .args(arguments)
        .arg(source)
        .output()
        .unwrap();
    assert_eq!(output.status, again.status);
    assert_eq!(output.stdout, again.stdout);
    assert_eq!(output.stderr, again.stderr);
    output
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
    assert!(output.stdout.is_empty());
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
fn s3w_rule_index_is_exact_and_covered() {
    let documented: Vec<_> = SPECIFICATION
        .lines()
        .filter_map(|line| {
            let first = line
                .strip_prefix("| ")?
                .split('|')
                .next()?
                .trim()
                .trim_matches(char::from(96));
            first.starts_with("S3W-").then_some(first)
        })
        .collect();
    assert_eq!(documented, RULES);
    let known: BTreeSet<_> = RULES.into_iter().collect();
    assert_eq!(known.len(), RULES.len());
    let mut covered = BTreeMap::<&str, u8>::new();
    for case in CASES {
        for rule in case.rules {
            assert!(known.contains(rule), "{rule}");
            *covered.entry(rule).or_default() |= 1;
        }
    }
    assert_eq!(covered.keys().copied().collect::<BTreeSet<_>>(), known);
    for line in SPECIFICATION
        .lines()
        .filter(|line| line.starts_with("| ") && line.contains("S3W-"))
    {
        let layer = line.split('|').rev().nth(1).unwrap().trim();
        assert_eq!(layer, "CLI");
    }
}

#[test]
fn s3w_fixture_inventory_and_outputs_are_exact() {
    let directory = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/s3w");
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
    for case in CASES {
        let path = directory.join(case.fixture);
        for command in ["check", "eval", "test"] {
            let arguments: Vec<&str> = if command == "eval" && case.errors.is_empty() {
                vec![command, "--spec", "result"]
            } else {
                vec![command]
            };
            let output = orangec(&arguments, &path);
            if !case.errors.is_empty() {
                failure(&output, case.errors);
            } else {
                let expected = match command {
                    "eval" => case.value.to_string(),
                    "test" => report(case.tests),
                    _ => String::new(),
                };
                success(&output, &expected);
            }
        }
    }
}
