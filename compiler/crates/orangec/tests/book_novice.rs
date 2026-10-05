//! Execute the Orange Book's actual fenced listings, not copied fixtures.
//! Educational regression evidence only; not a compiler or security proof.

use std::io::Write;
use std::process::{Command, Output, Stdio};

const CHAPTERS: &str = include_str!("../../../../docs/book/NOVICE_PROGRAMMING.md");

fn fences(language: &str) -> Vec<&str> {
    let start = format!("```{language}\n");
    let mut remaining = CHAPTERS;
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

fn run(command: &str, source: &str) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_orangec"))
        .args([command, "-"])
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

#[test]
fn all_seven_positive_book_listings_check_and_evaluate_repeatably() {
    let sources = fences("orange");
    let outputs = fences("text");
    assert_eq!(sources.len(), 9, "update coverage when adding listings");
    let mut checked = 0;
    for source in sources {
        let name = module_name(source);
        if matches!(name, "too_large" | "ungrouped") {
            continue;
        }
        let prefix = format!("{name}::");
        let expected: Vec<_> = outputs
            .iter()
            .filter(|output| output.starts_with(&prefix))
            .collect();
        assert_eq!(expected.len(), 1, "one expected output for {name}");
        let check = run("check", source);
        assert!(
            check.status.success(),
            "{name}: {}",
            String::from_utf8_lossy(&check.stderr)
        );
        assert!(check.stdout.is_empty(), "{name}: check printed a value");
        assert!(check.stderr.is_empty(), "{name}: check diagnostics");
        let first = run("eval", source);
        assert!(
            first.status.success(),
            "{name}: {}",
            String::from_utf8_lossy(&first.stderr)
        );
        assert_eq!(first.stdout, format!("{}\n", expected[0]).as_bytes());
        assert!(first.stderr.is_empty(), "{name}: eval diagnostics");
        let second = run("eval", source);
        assert_eq!(first.status.code(), second.status.code());
        assert_eq!(first.stdout, second.stdout);
        assert_eq!(first.stderr, second.stderr);
        checked += 1;
    }
    assert_eq!(checked, 7);
}

#[test]
fn both_deliberately_invalid_listings_are_rejected_without_values() {
    for (name, marker) in [("too_large", "256"), ("ungrouped", "ORC0108")] {
        let sources = fences("orange");
        let source = sources
            .iter()
            .find(|source| module_name(source) == name)
            .expect("negative listing exists");
        for command in ["check", "eval"] {
            let result = run(command, source);
            assert_eq!(result.status.code(), Some(1), "{name}: {command}");
            assert!(result.stdout.is_empty(), "no partial values for {name}");
            let diagnostic = String::from_utf8(result.stderr).expect("UTF-8 diagnostic");
            assert!(diagnostic.contains(marker), "{name}: {diagnostic}");
        }
    }
}

#[test]
fn removing_computed_amount_groups_exposes_the_literal_guard() {
    let sources = fences("orange");
    let source = sources
        .iter()
        .find(|source| module_name(source) == "boundary_moves")
        .expect("boundary listing exists");
    let changed = source.replace("(8)", "8").replace("(9)", "9");
    assert_ne!(*source, changed);
    let result = run("check", &changed);
    assert_eq!(result.status.code(), Some(1));
    assert!(result.stdout.is_empty());
    assert!(String::from_utf8_lossy(&result.stderr).contains("ORC0216"));
}
