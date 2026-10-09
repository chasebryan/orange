//! Execute J4's fenced listings. Educational regression evidence only.
//! A passing test is a Match of the Bool the listing writes. It is not a
//! cryptographic security claim, and it does not transcribe FIPS 180-4 §6.2.2.
//!
//! `orders` reads one four-byte group both ways. `wrong_order` is the
//! big-endian load of those bytes compared with the RFC 8439 §2.3 word.
//! `length_field` checks the FIPS 180-4 §5.1.1 length and the first message word.

use std::io::Write;
use std::process::{Command, Output, Stdio};

const J4: &str =
    include_str!("../../../../docs/book/JOURNEYMAN_J4_BYTE_ORDER_AND_FORMAT_BOUNDARIES.md");

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

fn j4_sources() -> Vec<&'static str> {
    fences(J4, "orange")
}

fn j4_text() -> Vec<&'static str> {
    fences(J4, "text")
}

fn j4_source(name: &str) -> &'static str {
    j4_sources()
        .into_iter()
        .find(|source| module_name(source) == name)
        .unwrap_or_else(|| panic!("missing J4 listing {name}"))
}

fn one_text(predicate: impl Fn(&str) -> bool, label: &str) -> &'static str {
    let matches: Vec<_> = j4_text()
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
fn j4_listings_read_both_orders_and_fail_the_swapped_word() {
    let sources = j4_sources();
    let names: Vec<_> = sources.iter().copied().map(module_name).collect();
    assert_eq!(names, vec!["orders", "wrong_order", "length_field"]);
    assert!(!J4.contains("\n## Chapter "));
    assert!(!J4.contains("this chapter"));
    assert!(!J4.contains("S3u"));
    assert!(J4.contains("implemented slice S3t"));
    assert!(J4.contains("The prediction is left `0x65787061` and right `0x61707865`."));
    for forbidden in [
        "spec compress(",
        "spec schedule(",
        "small_sigma0",
        "0x428a2f98",
    ] {
        assert!(!J4.contains(forbidden), "{forbidden}");
    }

    let orders = j4_source("orders");
    assert!(orders.contains("as little Word[32]"));
    assert!(orders.contains("as big Word[32]"));
    assert_silent_check("orders", orders);
    assert_stdout(
        "orders eval",
        &["eval", "-"],
        orders,
        one_text(|text| text.starts_with("orders::first_little:"), "orders eval"),
        0,
    );
    assert_stdout(
        "orders test",
        &["test", "-"],
        orders,
        one_text(
            |text| text.starts_with("test \"RFC 8439 2.3 first constant word\" ... ok"),
            "orders test",
        ),
        0,
    );

    let wrong = j4_source("wrong_order");
    assert!(wrong.contains("as big Word[32]"));
    assert!(!wrong.contains("as little"));
    assert_silent_check("wrong_order", wrong);
    assert_stdout(
        "wrong_order eval",
        &["eval", "-"],
        wrong,
        one_text(
            |text| text.starts_with("wrong_order::first:"),
            "wrong_order eval",
        ),
        0,
    );
    assert_stdout(
        "wrong_order test",
        &["test", "-"],
        wrong,
        one_text(
            |text| {
                text.starts_with("test \"RFC 8439 2.3 first constant word\" ... FAILED")
                    && text.contains("left:  0x65787061")
                    && text.contains("right: 0x61707865")
                    && text.contains("1 test: 0 passed, 1 failed")
            },
            "wrong_order test",
        ),
        1,
    );

    let length = j4_source("length_field");
    assert!(length.contains("length_be() == 24"));
    assert!(length.contains("word0() == 0x61626380"));
    assert_silent_check("length_field", length);
    assert_stdout(
        "length_field eval",
        &["eval", "-"],
        length,
        one_text(
            |text| text.starts_with("length_field::bits:"),
            "length_field eval",
        ),
        0,
    );
    assert_stdout(
        "length_field test",
        &["test", "-"],
        length,
        one_text(
            |text| {
                text.contains("test \"FIPS 180-4 5.1.1 length field is the bit length\" ... ok")
                    && text.contains("3 tests: 3 passed, 0 failed")
            },
            "length_field test",
        ),
        0,
    );
}
