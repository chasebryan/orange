//! Execute J7's fenced listings. Educational regression evidence only.
//! A passing test is a Match on the inputs the listing wrote.

use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};

const J7: &str = include_str!("../../../../docs/book/JOURNEYMAN_J7_KECCAK_P_AND_SHA3.md");

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
    fences(J7, "orange")
}

fn text_blocks() -> Vec<&'static str> {
    fences(J7, "text")
}

fn source(name: &str) -> &'static str {
    sources()
        .into_iter()
        .find(|source| module_name(source) == name)
        .unwrap_or_else(|| panic!("missing J7 listing {name}"))
}

fn run(source: &str, arguments: &[&str]) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_orangec"))
        .args(arguments)
        .current_dir(PathBuf::from(env!("CARGO_MANIFEST_DIR")))
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

fn block_equal(body: &str) -> String {
    format!("{body}\n")
}

#[test]
fn j7_listings_are_the_seven_fenced_programs() {
    let names: Vec<_> = sources().into_iter().map(module_name).collect();
    assert_eq!(
        names,
        [
            "wordw", "nested", "paired", "word1", "rot8", "facts", "keccak"
        ]
    );
    assert!(J7.contains("orangec 0.0.1 (Orange edition 2026; implemented slice S3u)"));
    assert!(J7.contains("The locked label is J7."));
    assert!(J7.contains("It is not a manuscript"));
    assert!(!J7.contains("\n## Chapter "));
    assert!(!J7.contains("\n### Chapter "));
    assert!(J7.contains("Do not call that Match verified."));
    assert!(J7.contains("A Match is not called verified."));
    assert!(!J7.to_ascii_lowercase().contains("constant-time"));
    assert!(!J7.to_ascii_lowercase().contains("constant time"));
    assert!(!J7.to_ascii_lowercase().contains("production security"));
    assert!(source("keccak").contains("hex\"3c2826841cb35c171eaae9b811134ceaa3852c69d2c5abafea\""));
    assert!(
        source("keccak")
            .contains("hex\"3a985da74fe225b2045c172d6bd390bd855f086e3e9d525b46bfe24511431532\"")
    );
    assert!(
        source("keccak")
            .contains("hex\"a7ffc6f8bf1ed76651c14756a061d662f580ff4de43b49fa82d80a4b80f8434a\"")
    );
    assert!(source("wordw").contains("Word[w]"));
    assert!(source("nested").contains("W^5^5"));
    assert!(source("word1").contains("Word[1]"));
    assert!(source("rot8").contains("<<< 8"));
    assert!(source("facts").contains("chi_roundtrip"));
}

#[test]
fn j7_printed_runs_match_the_compiler() {
    let blocks = text_blocks();
    let mut matched_errors = vec![false; blocks.len()];
    let mut matched_reports = vec![false; blocks.len()];
    let mut matched_stats = vec![false; blocks.len()];

    for source in sources() {
        let name = module_name(source);
        let check = run(source, &["check", "-"]);
        let check_again = run(source, &["check", "-"]);
        assert_same(&check, &check_again);
        if check.status.success() {
            assert!(check.stdout.is_empty(), "{name}: check printed a value");
            assert!(check.stderr.is_empty(), "{name}: check diagnostics");
            let test = run(source, &["test", "-"]);
            let test_again = run(source, &["test", "-"]);
            assert_same(&test, &test_again);
            assert!(
                test.stderr.is_empty(),
                "{name}: {}",
                String::from_utf8_lossy(&test.stderr)
            );
            if name == "keccak" {
                assert_eq!(test.status.code(), Some(1), "{name}: test");
            } else {
                assert!(test.status.success(), "{name}: test");
            }
            let report = String::from_utf8(test.stdout.clone()).expect("UTF-8 report");
            let hits: Vec<_> = blocks
                .iter()
                .enumerate()
                .filter(|(_, block)| block_equal(block) == report)
                .map(|(index, _)| index)
                .collect();
            assert_eq!(hits.len(), 1, "{name}: test report fence");
            matched_reports[hits[0]] = true;
        } else {
            assert_eq!(check.status.code(), Some(1), "{name}: check");
            assert!(check.stdout.is_empty(), "{name}: check printed a value");
            let diagnostic = String::from_utf8(check.stderr.clone()).expect("UTF-8 diagnostic");
            let hits: Vec<_> = blocks
                .iter()
                .enumerate()
                .filter(|(_, block)| block_equal(block) == diagnostic)
                .map(|(index, _)| index)
                .collect();
            assert_eq!(hits.len(), 1, "{name}: diagnostic fence\n{diagnostic}");
            matched_errors[hits[0]] = true;
            for command in ["eval", "test"] {
                let output = run(source, &[command, "-"]);
                assert_eq!(output.status.code(), Some(1), "{name}: {command}");
                assert!(
                    output.stdout.is_empty(),
                    "{name}: {command} printed a value"
                );
                assert_eq!(output.stderr, check.stderr, "{name}: {command}");
            }
        }
    }

    for block in &blocks {
        for line in block.split('\n') {
            let Some((name, rest)) = line.split_once("::") else {
                continue;
            };
            if !name
                .chars()
                .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '_')
            {
                continue;
            }
            let Some(spec) = rest.split(':').next() else {
                continue;
            };
            if spec.is_empty()
                || !spec
                    .chars()
                    .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '_')
            {
                continue;
            }
            if !line.starts_with(&format!("{name}::{spec}:")) {
                continue;
            }
            let output = run(source(name), &["eval", "--spec", spec, "-"]);
            assert!(
                output.status.success(),
                "{name} {spec}: {}",
                String::from_utf8_lossy(&output.stderr)
            );
            assert!(output.stderr.is_empty(), "{name} {spec}");
            assert_eq!(
                String::from_utf8(output.stdout).expect("UTF-8 value"),
                format!("{line}\n"),
                "{name} {spec}"
            );
        }
    }

    for (name, label) in [("facts", "facts stats"), ("keccak", "keccak stats")] {
        let stats = run(source(name), &["test", "--stats", "-"]);
        if name == "keccak" {
            assert_eq!(stats.status.code(), Some(1), "{label}");
        } else {
            assert!(stats.status.success(), "{label}");
        }
        mark_stats(
            &blocks,
            &mut matched_stats,
            &String::from_utf8(stats.stderr).expect("UTF-8 stats"),
            label,
        );
    }

    for (index, block) in blocks.iter().enumerate() {
        if block.starts_with("error[") {
            assert!(matched_errors[index], "unmatched diagnostic\n{block}");
        }
        if block.starts_with("test \"")
            && (block.contains("... ok") || block.contains("... FAILED"))
        {
            assert!(matched_reports[index], "unmatched test report\n{block}");
        }
        if block.starts_with("test \"") && block.contains(" steps") {
            assert!(matched_stats[index], "unmatched stats\n{block}");
        }
    }
}

fn mark_stats(blocks: &[&str], matched: &mut [bool], stderr: &str, label: &str) {
    let hits: Vec<_> = blocks
        .iter()
        .enumerate()
        .filter(|(_, block)| block_equal(block) == stderr)
        .map(|(index, _)| index)
        .collect();
    assert_eq!(hits.len(), 1, "{label}\n{stderr}");
    matched[hits[0]] = true;
}
