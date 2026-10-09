//! Execute J3's fenced listings. Educational regression evidence only.
//! A passing test is a Match of the Bool the listing writes. It is not a
//! cryptographic security claim, and it does not transcribe FIPS 180-4 §6.2.2.
//!
//! `wrong_table` and `wrong_flip` each pass the thin corpus and fail the
//! RFC 4231 §4.4 key byte, with different left values. `wrong_rest` passes
//! every test in `corpus` and still denotes a different byte at `0x01`.

use std::io::Write;
use std::process::{Command, Output, Stdio};

const J3: &str =
    include_str!("../../../../docs/book/JOURNEYMAN_J3_THE_CORPUS_AS_ACCEPTANCE_TEST.md");

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

fn j3_sources() -> Vec<&'static str> {
    fences(J3, "orange")
}

fn j3_text() -> Vec<&'static str> {
    fences(J3, "text")
}

fn j3_source(name: &str) -> &'static str {
    j3_sources()
        .into_iter()
        .find(|source| module_name(source) == name)
        .unwrap_or_else(|| panic!("missing J3 listing {name}"))
}

fn one_text(predicate: impl Fn(&str) -> bool, label: &str) -> &'static str {
    let matches: Vec<_> = j3_text()
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
fn j3_listings_accept_a_corpus_and_separate_wrong_functions() {
    let sources = j3_sources();
    let names: Vec<_> = sources.iter().copied().map(module_name).collect();
    assert_eq!(
        names,
        vec![
            "corpus",
            "wrong_rest",
            "mislabelled",
            "wrong_table",
            "wrong_flip",
        ]
    );
    assert!(!J3.contains("\n## Chapter "));
    assert!(!J3.contains("Chapter 12"));
    assert!(!J3.contains("S3u"));
    assert!(J3.contains("implemented slice S3t"));
    for forbidden in [
        "spec compress(",
        "spec schedule(",
        "small_sigma0",
        "0x428a2f98",
        "spec pack(",
        "Word[32]",
    ] {
        assert!(
            !sources.iter().any(|source| source.contains(forbidden)),
            "{forbidden}"
        );
    }

    let corpus_report = one_text(
        |text| text.starts_with("test \"RFC 4231 4.2 test case 1 key byte\" ... ok"),
        "corpus test",
    );
    assert!(corpus_report.contains("6 tests: 6 passed, 0 failed"));

    let corpus = j3_source("corpus");
    assert_silent_check("corpus", corpus);
    assert_stdout(
        "corpus eval",
        &["eval", "-"],
        corpus,
        one_text(
            |text| text.starts_with("corpus::case2_inner:"),
            "corpus eval",
        ),
        0,
    );
    assert_stdout("corpus test", &["test", "-"], corpus, corpus_report, 0);

    let wrong_rest = j3_source("wrong_rest");
    assert!(wrong_rest.contains("inner(0x01)"));
    assert_silent_check("wrong_rest", wrong_rest);
    assert_stdout(
        "wrong_rest eval",
        &["eval", "-"],
        wrong_rest,
        one_text(|text| text.starts_with("wrong_rest::off:"), "wrong_rest eval"),
        0,
    );
    assert_stdout(
        "wrong_rest test",
        &["test", "-"],
        wrong_rest,
        corpus_report,
        0,
    );

    let mislabelled = j3_source("mislabelled");
    assert!(mislabelled.contains("case2_inner() == 0x3d"));
    assert_silent_check("mislabelled", mislabelled);
    assert_stdout(
        "mislabelled eval",
        &["eval", "-"],
        mislabelled,
        one_text(
            |text| text.starts_with("mislabelled::case2_inner:"),
            "mislabelled eval",
        ),
        0,
    );
    assert_stdout(
        "mislabelled test",
        &["test", "-"],
        mislabelled,
        one_text(
            |text| {
                text.starts_with("test \"RFC 4231 4.3 test case 2 key byte J\" ... FAILED")
                    && text.contains("left:  0x7c")
                    && text.contains("right: 0x3d")
            },
            "mislabelled test",
        ),
        1,
    );

    let wrong_table = j3_source("wrong_table");
    assert!(wrong_table.contains("inner(0x00) == 0x36"));
    assert!(wrong_table.contains("inner(0x0b) == 0x3d"));
    assert!(wrong_table.contains("inner(0x4a) == 0x7c"));
    assert!(wrong_table.contains("case3_inner() == 0x9c"));
    assert_silent_check("wrong_table", wrong_table);
    assert_stdout(
        "wrong_table eval",
        &["eval", "-"],
        wrong_table,
        one_text(
            |text| text.starts_with("wrong_table::case3_inner:"),
            "wrong_table eval",
        ),
        0,
    );
    assert_stdout(
        "wrong_table test",
        &["test", "-"],
        wrong_table,
        one_text(
            |text| {
                text.contains("test \"RFC 2104 2 zero fill\" ... ok")
                    && text.contains("test \"RFC 4231 4.2 test case 1 key byte\" ... ok")
                    && text.contains("test \"RFC 4231 4.3 test case 2 key byte J\" ... ok")
                    && text.contains("left:  0x00")
                    && text.contains("right: 0x9c")
                    && text.contains("4 tests: 3 passed, 1 failed")
            },
            "wrong_table test",
        ),
        1,
    );

    let wrong_flip = j3_source("wrong_flip");
    assert!(wrong_flip.contains("k ^ 0x36"));
    assert!(wrong_flip.contains("x ^ 0x01"));
    assert_silent_check("wrong_flip", wrong_flip);
    assert_stdout(
        "wrong_flip eval",
        &["eval", "-"],
        wrong_flip,
        one_text(
            |text| text.starts_with("wrong_flip::case3_inner:"),
            "wrong_flip eval",
        ),
        0,
    );
    assert_stdout(
        "wrong_flip test",
        &["test", "-"],
        wrong_flip,
        one_text(
            |text| {
                text.contains("test \"RFC 2104 2 zero fill\" ... ok")
                    && text.contains("test \"RFC 4231 4.2 test case 1 key byte\" ... ok")
                    && text.contains("test \"RFC 4231 4.3 test case 2 key byte J\" ... ok")
                    && text.contains("left:  0x9d")
                    && text.contains("right: 0x9c")
                    && text.contains("4 tests: 3 passed, 1 failed")
            },
            "wrong_flip test",
        ),
        1,
    );
}
