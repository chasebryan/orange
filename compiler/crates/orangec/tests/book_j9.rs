//! Execute J9's fenced listings. Educational regression evidence only.
//! A passing test is a Match on the inputs the listing wrote.

use std::io::Write;
use std::process::{Command, Output, Stdio};

const J9: &str = include_str!("../../../../docs/book/JOURNEYMAN_J9_CHACHA20.md");

fn fences<'a>(text: &'a str, language: &str) -> Vec<&'a str> {
    let start = format!("```{language}\n");
    let mut remaining = text;
    let mut blocks = Vec::new();
    while let Some((_, after)) = remaining.split_once(&start) {
        let (body, rest) = after.split_once("\n```").expect("unclosed book fence");
        blocks.push(body);
        remaining = rest;
    }
    blocks
}

fn module_name(source: &str) -> &str {
    source
        .split_whitespace()
        .skip_while(|token| *token != "module")
        .nth(1)
        .expect("listing must name its module")
}

fn sources() -> Vec<&'static str> {
    fences(J9, "orange")
}

fn text_blocks() -> Vec<&'static str> {
    fences(J9, "text")
}

fn source(name: &str) -> &'static str {
    sources()
        .into_iter()
        .find(|source| module_name(source) == name)
        .unwrap_or_else(|| panic!("missing J9 listing {name}"))
}

fn one_text(predicate: impl Fn(&str) -> bool, label: &str) -> &'static str {
    let matches: Vec<_> = text_blocks()
        .into_iter()
        .filter(|text| predicate(text))
        .collect();
    assert_eq!(matches.len(), 1, "{label}");
    matches[0]
}

fn line_starting(prefix: &str) -> &'static str {
    let mut found = None;
    for block in text_blocks() {
        for line in block.split('\n') {
            if line.starts_with(prefix) {
                assert!(found.is_none(), "duplicate line {prefix}");
                found = Some(line);
            }
        }
    }
    found.unwrap_or_else(|| panic!("missing line {prefix}"))
}

fn run(arguments: &[&str], source: &str) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_orangec"))
        .args(arguments)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("start orangec");
    child
        .stdin
        .take()
        .expect("piped stdin")
        .write_all(source.as_bytes())
        .expect("write the listing");
    child.wait_with_output().expect("wait for orangec")
}

fn assert_same(first: &Output, second: &Output) {
    assert_eq!(first.status.code(), second.status.code());
    assert_eq!(first.stdout, second.stdout);
    assert_eq!(first.stderr, second.stderr);
}

fn assert_rejected(name: &str, code: &str, locus: &str) {
    let source = source(name);
    let expected = format!(
        "{}\n",
        one_text(|text| text.starts_with(code) && text.contains(locus), name)
    );
    for command in ["check", "eval", "test"] {
        let first = run(&[command, "-"], source);
        assert_eq!(first.status.code(), Some(1), "{name}: {command}");
        assert!(first.stdout.is_empty(), "{name}: {command} printed a value");
        assert_eq!(
            String::from_utf8(first.stderr.clone()).expect("UTF-8 diagnostic"),
            expected,
            "{name}: {command}"
        );
        let second = run(&[command, "-"], source);
        assert_same(&first, &second);
    }
}

fn assert_silent(name: &str) {
    let source = source(name);
    let first = run(&["check", "-"], source);
    assert!(
        first.status.success(),
        "{name}: {}",
        String::from_utf8_lossy(&first.stderr)
    );
    assert!(first.stdout.is_empty(), "{name}: check printed a value");
    assert!(first.stderr.is_empty(), "{name}: check diagnostics");
    let second = run(&["check", "-"], source);
    assert_same(&first, &second);
}

fn assert_eval(name: &str, spec: &str) {
    let prefix = format!("{name}::{spec}:");
    let expected = format!("{}\n", line_starting(&prefix));
    let first = run(&["eval", "--spec", spec, "-"], source(name));
    assert!(
        first.status.success(),
        "{name} {spec}: {}",
        String::from_utf8_lossy(&first.stderr)
    );
    assert_eq!(first.stdout, expected.as_bytes(), "{name} {spec}");
    assert!(first.stderr.is_empty(), "{name} {spec}: eval diagnostics");
    let second = run(&["eval", "--spec", spec, "-"], source(name));
    assert_same(&first, &second);
}

fn assert_test_report(name: &str, predicate: impl Fn(&str) -> bool, status: i32) {
    let expected = format!("{}\n", one_text(predicate, name));
    let first = run(&["test", "-"], source(name));
    assert_eq!(first.status.code(), Some(status), "{name}: test");
    assert!(
        first.stderr.is_empty(),
        "{name}: {}",
        String::from_utf8_lossy(&first.stderr)
    );
    assert_eq!(first.stdout, expected.as_bytes(), "{name}: test");
    let second = run(&["test", "-"], source(name));
    assert_same(&first, &second);
}

fn assert_passing_with_stats(name: &str, report: impl Fn(&str) -> bool, total: &str) {
    let expected_report = format!("{}\n", one_text(report, &format!("{name} report")));
    let expected_stats = format!(
        "{}\n",
        one_text(
            |text| text.starts_with("test \"") && text.contains(total),
            &format!("{name} stats"),
        )
    );
    let first = run(&["test", "--stats", "-"], source(name));
    assert_eq!(
        first.status.code(),
        Some(0),
        "{name}: test --stats\n{}",
        String::from_utf8_lossy(&first.stderr)
    );
    assert_eq!(first.stdout, expected_report.as_bytes(), "{name}: report");
    assert_eq!(
        String::from_utf8(first.stderr.clone()).expect("UTF-8 stats"),
        expected_stats,
        "{name}: stats"
    );
    let second = run(&["test", "--stats", "-"], source(name));
    assert_same(&first, &second);
}

#[test]
fn j9_listings_are_the_eleven_fenced_programs() {
    let names: Vec<_> = sources().into_iter().map(module_name).collect();
    assert_eq!(
        names,
        [
            "quarter",
            "on_state",
            "block",
            "rot15",
            "big_load",
            "diag_first",
            "encrypt",
            "counter_off",
            "nonce_reuse",
            "too_wide",
            "too_many",
        ]
    );
    assert!(J9.contains("orangec 0.0.1 (Orange edition 2026; implemented slice S3u)"));
    assert!(J9.contains("The locked label is J9."));
    assert!(J9.contains("It is not a manuscript"));
    assert!(!J9.contains("\n## Chapter "));
    assert!(!J9.contains("\n### Chapter "));
    assert!(J9.contains("A Match is not called verified."));
    assert!(J9.contains("Do not call that Match verified."));
    let lowered = J9.to_ascii_lowercase();
    assert!(!lowered.contains("constant-time"));
    assert!(!lowered.contains("constant time"));
    assert!(!lowered.contains("production security"));
    assert!(source("rot15").contains("<<< 15"));
    assert!(source("quarter").contains("<<< 16"));
    assert!(source("big_load").contains("as big"));
    assert!(source("diag_first").contains("quarter_round(s[0], s[5], s[10], s[15])"));
    assert!(source("counter_off").contains("block(key32(), 0, nonce242())"));
    assert!(source("encrypt").contains("blocks in 1..7"));
    assert!(source("encrypt").contains("full[..114]"));
    assert!(source("encrypt").contains("full[..375]"));
    assert!(source("block").contains("for i in 1..11"));
    assert!(J9.contains("76 b8 e0 ad"));
    assert!(J9.contains("6e 2e 35 9a"));
}

#[test]
fn j9_quarter_round_block_and_encrypt_match_rfc_8439() {
    for name in ["quarter", "on_state", "block", "encrypt"] {
        assert_silent(name);
    }
    for (name, spec) in [
        ("quarter", "a1"),
        ("quarter", "d1"),
        ("quarter", "c2"),
        ("quarter", "vector"),
        ("quarter", "recovered"),
        ("on_state", "mid_c"),
        ("on_state", "word2"),
        ("block", "column_a"),
        ("block", "column_c"),
    ] {
        assert_eval(name, spec);
    }
    assert_passing_with_stats(
        "quarter",
        |text| {
            text.contains("test \"RFC 8439 2.1.1\" ... ok")
                && text.contains("test \"inverse of RFC 8439 2.1.1\" ... ok")
                && text.contains("2 tests: 2 passed, 0 failed")
        },
        "total: 108 of 1048576 steps",
    );
    assert_passing_with_stats(
        "on_state",
        |text| text.contains("6 tests: 6 passed, 0 failed"),
        "total: 561 of 1048576 steps",
    );
    assert_passing_with_stats(
        "block",
        |text| {
            text.contains("test \"RFC 8439 A.1 vector 5\" ... ok")
                && text.contains("15 tests: 15 passed, 0 failed")
        },
        "total: 39679 of 1048576 steps",
    );
    assert_passing_with_stats(
        "encrypt",
        |text| {
            text.contains("test \"RFC 8439 2.4.2\" ... ok")
                && text.contains("test \"RFC 8439 A.2 vector 2\" ... ok")
                && text.contains("5 tests: 5 passed, 0 failed")
        },
        "total: 79698 of 1048576 steps",
    );
}

#[test]
fn j9_wrong_programs_fail_and_two_forms_are_rejected() {
    for name in [
        "rot15",
        "big_load",
        "diag_first",
        "counter_off",
        "nonce_reuse",
    ] {
        assert_silent(name);
    }
    for (name, spec) in [
        ("rot15", "a1"),
        ("rot15", "d_after_roll"),
        ("rot15", "a_final"),
        ("big_load", "key_word"),
        ("big_load", "counter_word"),
        ("big_load", "nonce_word"),
        ("diag_first", "first_a"),
        ("counter_off", "first_byte"),
        ("nonce_reuse", "cipher_xor"),
    ] {
        assert_eval(name, spec);
    }
    assert_test_report(
        "rot15",
        |text| {
            text.contains("test \"a before the roll\" ... ok")
                && text.contains("left:  0x59d11069")
                && text.contains("right: 0xea2a92f4")
                && text.contains("2 tests: 1 passed, 1 failed")
        },
        1,
    );
    assert_test_report(
        "big_load",
        |text| {
            text.contains("left:  0x00010203")
                && text.contains("left:  0x01000000")
                && text.contains("left:  0x00000009")
                && text.contains("3 tests: 0 passed, 3 failed")
        },
        1,
    );
    assert_test_report(
        "diag_first",
        |text| {
            text.contains("left:  0xc0bf2761")
                && text.contains("right: 0xabbaa25c")
                && text.contains("1 test: 0 passed, 1 failed")
        },
        1,
    );
    assert_test_report(
        "counter_off",
        |text| {
            text.contains("left:  0xe3")
                && text.contains("right: 0x6e")
                && text.contains("1 test: 0 passed, 1 failed")
        },
        1,
    );
    assert_test_report(
        "nonce_reuse",
        |text| {
            text.contains("test \"ciphertext XOR equals plaintext XOR\" ... ok")
                && text.contains("left:  [0x01, 0x02, 0x03, 0x04]")
                && text.contains("2 tests: 1 passed, 1 failed")
        },
        1,
    );
    assert_rejected("too_wide", "error[ORC0221]", "<stdin>:3:25");
    assert_rejected("too_many", "error[ORC0238]", "<stdin>:3:16");
}
