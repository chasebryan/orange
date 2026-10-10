//! Execute J5's fenced listings. Educational regression evidence only.
//! A passing test is a Match on the inputs the listing wrote.

use std::io::Write;
use std::process::{Command, Output, Stdio};

const J5: &str =
    include_str!("../../../../docs/book/JOURNEYMAN_J5_SHA256_AS_FIPS_180_4_WRITES_IT.md");

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
    fences(J5, "orange")
}

fn text_blocks() -> Vec<&'static str> {
    fences(J5, "text")
}

fn source(name: &str) -> &'static str {
    sources()
        .into_iter()
        .find(|source| module_name(source) == name)
        .unwrap_or_else(|| panic!("missing J5 listing {name}"))
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

#[test]
fn j5_listings_are_the_nine_fenced_programs() {
    let names: Vec<_> = sources().into_iter().map(module_name).collect();
    assert_eq!(
        names,
        [
            "bare_length",
            "indexed_as",
            "pad",
            "greek",
            "functions",
            "schedule",
            "round0",
            "sha256",
            "shr_for_rotr",
        ]
    );
    assert!(J5.contains("orangec 0.0.1 (Orange edition 2026; implemented slice S3u)"));
    assert!(J5.contains("The locked label is J5."));
    assert!(!J5.contains("\n## Chapter "));
    assert!(!J5.contains("\n### Chapter "));
    assert!(J5.contains("It is not a manuscript"));
    assert!(J5.contains("Do not call that Match verified."));
    assert!(J5.contains("A Match is not called verified."));
    assert!(!J5.to_ascii_lowercase().contains("constant-time"));
    assert!(!J5.to_ascii_lowercase().contains("constant time"));
    assert!(!J5.to_ascii_lowercase().contains("production security"));
    let greek = source("greek");
    assert!(greek.contains('σ'));
    assert!(source("shr_for_rotr").contains("(x >> 17) ^ (x >>> 19) ^ (x >> 10)"));
    assert!(source("schedule").contains("(x >>> 17) ^ (x >>> 19) ^ (x >> 10)"));
}

#[test]
fn j5_rejected_listings_match_the_printed_diagnostics() {
    assert_rejected("bare_length", "error[ORC0220]", "<stdin>:4:37");
    assert_rejected("indexed_as", "error[ORC0101]", "<stdin>:5:36");
    assert_rejected("greek", "error[ORC0001]", "<stdin>:3:8");
}

#[test]
fn j5_padding_schedule_round_and_digests_match_the_printed_runs() {
    assert_silent("pad");
    assert_eval("pad", "abc_words");
    assert_test_report(
        "pad",
        |text| {
            text.starts_with("test \"FIPS 180-4 5.1.1 abc block\"")
                && text.contains("3 tests: 3 passed, 0 failed")
        },
        0,
    );

    assert_silent("functions");
    assert_eval("functions", "ch_round0");
    assert_test_report(
        "functions",
        |text| {
            text.starts_with("test \"FIPS 180-4 eq 4.2, Ch on H4 H5 H6\"")
                && text.contains("4 tests: 4 passed, 0 failed")
        },
        0,
    );

    assert_silent("schedule");
    assert_eval("schedule", "w16");
    assert_eval("schedule", "w17");
    assert_test_report(
        "schedule",
        |text| {
            text.starts_with("test \"FIPS 180-4 6.2.2 abc W16\"")
                && text.contains("2 tests: 2 passed, 0 failed")
        },
        0,
    );

    assert_silent("round0");
    assert_eval("round0", "t1");
    assert_eval("round0", "t2");
    assert_eval("round0", "after");
    assert_test_report(
        "round0",
        |text| {
            text.starts_with("test \"FIPS 180-4 6.2.2 abc T1\"")
                && text.contains("3 tests: 3 passed, 0 failed")
        },
        0,
    );

    assert_silent("sha256");
    assert_eval("sha256", "abc");
    assert_eval("sha256", "two_h1");
    assert_eval("sha256", "two");
    assert_test_report(
        "sha256",
        |text| {
            text.starts_with("test \"examples file abc digest\"")
                && text.contains("3 tests: 3 passed, 0 failed")
        },
        0,
    );
}

#[test]
fn j5_shift_for_rotation_fails_the_printed_word() {
    assert_silent("shr_for_rotr");
    assert_eval("shr_for_rotr", "w17");
    assert_test_report(
        "shr_for_rotr",
        |text| {
            text.contains("test \"FIPS 180-4 6.2.2 abc W17\" ... FAILED")
                && text.contains("left:  0x00030000")
                && text.contains("right: 0x000f0000")
                && text.contains("2 tests: 1 passed, 1 failed")
        },
        1,
    );
}
