//! Execute J2's fenced listings. Educational regression evidence only.
//! A passing test is a Match of the Bool the listing writes. It is not a
//! cryptographic security claim and it does not transcribe FIPS 180-4 §6.2.2.

use std::io::Write;
use std::process::{Command, Output, Stdio};

const J2: &str =
    include_str!("../../../../docs/book/JOURNEYMAN_J2_STANDARDS_AS_VERSIONED_INPUTS.md");

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
        .expect("complete listing must name its module")
}

fn run_with(arguments: &[&str], source: &str) -> Output {
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
        .expect("write the complete listing");
    child.wait_with_output().expect("wait for orangec")
}

fn j2_sources() -> Vec<&'static str> {
    fences(J2, "orange")
}

fn j2_text() -> Vec<&'static str> {
    fences(J2, "text")
}

fn j2_source(name: &str) -> &'static str {
    j2_sources()
        .into_iter()
        .find(|source| module_name(source) == name)
        .unwrap_or_else(|| panic!("missing J2 listing {name}"))
}

fn one_text(predicate: impl Fn(&str) -> bool, label: &str) -> &'static str {
    let matches: Vec<_> = j2_text()
        .into_iter()
        .filter(|text| predicate(text))
        .collect();
    assert_eq!(matches.len(), 1, "{label}");
    matches[0]
}

fn assert_silent_check(name: &str, source: &str) {
    let check = run_with(&["check", "-"], source);
    assert!(
        check.status.success(),
        "{name}: {}",
        String::from_utf8_lossy(&check.stderr)
    );
    assert!(check.stdout.is_empty(), "{name}: check printed a value");
    assert!(check.stderr.is_empty(), "{name}: check diagnostics");
}

fn assert_stdout(name: &str, arguments: &[&str], source: &str, body: &str, status: i32) {
    let expected = format!("{body}\n");
    let first = run_with(arguments, source);
    assert_eq!(first.status.code(), Some(status), "{name}: {arguments:?}");
    assert!(first.stderr.is_empty(), "{name}: {arguments:?} diagnostics");
    assert_eq!(
        String::from_utf8(first.stdout.clone()).expect("UTF-8"),
        expected,
        "{name}: {arguments:?}"
    );
    let second = run_with(arguments, source);
    assert_eq!(first.status.code(), second.status.code());
    assert_eq!(first.stdout, second.stdout);
    assert_eq!(first.stderr, second.stderr);
}

#[test]
fn j2_listings_pin_an_edition_and_reject_a_mismatched_one() {
    let sources = j2_sources();
    let names: Vec<_> = sources.iter().copied().map(module_name).collect();
    assert_eq!(names, vec!["byte_limit", "wrong_edition"]);
    assert!(!J2.contains("\n## Chapter "));
    assert!(!J2.contains("Chapter 11"));
    for forbidden in [
        "spec compress(",
        "spec schedule(",
        "small_sigma0",
        "0x428a2f98",
    ] {
        assert!(!J2.contains(forbidden), "{forbidden}");
    }

    let byte_limit = j2_source("byte_limit");
    assert_silent_check("byte_limit", byte_limit);
    assert_stdout(
        "byte_limit eval",
        &["eval", "-"],
        byte_limit,
        one_text(
            |text| text.starts_with("byte_limit::blocks:"),
            "byte_limit eval",
        ),
        0,
    );
    assert_stdout(
        "byte_limit test",
        &["test", "-"],
        byte_limit,
        one_text(
            |text| text.starts_with("test \"RFC 8439 2.8 P_MAX\""),
            "byte_limit test",
        ),
        0,
    );

    let wrong_edition = j2_source("wrong_edition");
    assert_silent_check("wrong_edition", wrong_edition);
    assert_stdout(
        "wrong_edition eval",
        &["eval", "-"],
        wrong_edition,
        one_text(
            |text| text.starts_with("wrong_edition::blocks:"),
            "wrong_edition eval",
        ),
        0,
    );
    assert_stdout(
        "wrong_edition test",
        &["test", "-"],
        wrong_edition,
        one_text(
            |text| text.starts_with("test \"RFC 8439 2.8 copied from RFC 7539\""),
            "wrong_edition test",
        ),
        1,
    );
}
