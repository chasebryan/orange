//! Execute J6's fenced listings. Educational regression evidence only.
//! A passing test is a Match on the inputs the listing wrote.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

const J6: &str = include_str!("../../../../docs/book/JOURNEYMAN_J6_SHA512_AND_THE_SHA2_FAMILY.md");

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
    fences(J6, "orange")
}

fn text_blocks() -> Vec<&'static str> {
    fences(J6, "text")
}

fn source(name: &str) -> &'static str {
    sources()
        .into_iter()
        .find(|source| module_name(source) == name)
        .unwrap_or_else(|| panic!("missing J6 listing {name}"))
}

fn sha2_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../algorithms/sha2")
}

fn run_at(directory: &Path, arguments: &[&str], source: Option<&str>) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_orangec"))
        .args(arguments)
        .current_dir(directory)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("start orangec");
    if let Some(source) = source {
        child
            .stdin
            .take()
            .expect("piped stdin")
            .write_all(source.as_bytes())
            .expect("write the listing");
    }
    child.wait_with_output().expect("wait for orangec")
}

fn run(source: &str, arguments: &[&str]) -> Output {
    let directory = if source.contains("use sha2;") {
        sha2_dir()
    } else {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    };
    run_at(&directory, arguments, Some(source))
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
fn j6_listings_are_the_twenty_one_fenced_programs() {
    let names: Vec<_> = sources().into_iter().map(module_name).collect();
    assert_eq!(
        names,
        [
            "pad",
            "functions",
            "schedule",
            "round0",
            "sha512",
            "bad_sigma",
            "carried",
            "short_field",
            "missing80",
            "truncate",
            "word128",
            "rot64",
            "rot_computed",
            "wrap64",
            "no_shift",
            "wide_int",
            "past_range",
            "n_bound",
            "too_many",
            "budget",
            "bare_length",
        ]
    );
    assert!(J6.contains("orangec 0.0.1 (Orange edition 2026; implemented slice S3u)"));
    assert!(J6.contains("The locked label is J6."));
    assert!(J6.contains("It is not a manuscript"));
    assert!(!J6.contains("\n## Chapter "));
    assert!(!J6.contains("\n### Chapter "));
    assert!(J6.contains("Do not call that Match verified."));
    assert!(J6.contains("A Match is not called verified."));
    assert!(!J6.to_ascii_lowercase().contains("constant-time"));
    assert!(!J6.to_ascii_lowercase().contains("constant time"));
    assert!(!J6.to_ascii_lowercase().contains("production security"));
    assert!(source("sha512").contains("hex\"ddaf35a193617aba"));
    assert!(source("bad_sigma").contains("(x >>> 6) ^ (x >>> 11) ^ (x >>> 25)"));
    assert!(source("functions").contains("(x >>> 14) ^ (x >>> 18) ^ (x >>> 41)"));
    assert!(source("carried").contains("padding[3, 1]()"));
    assert!(source("short_field").contains("use sha2;"));
    assert!(source("truncate").contains("[..48]"));
    assert!(source("word128").contains("Word[128]"));
    assert!(source("rot64").contains(">>> 64"));
    assert!(source("no_shift").contains("1 << 310"));
    assert!(source("too_many").contains("len in 0..200"));
    assert!(source("budget").contains("sha512_abc_len_bits_24"));
}

#[test]
fn j6_printed_runs_match_the_compiler() {
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
            assert_eq!(hits.len(), 1, "{name}: diagnostic fence");
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

    let sha512_stats = run(source("sha512"), &["test", "--stats", "-"]);
    assert!(sha512_stats.status.success());
    mark_stats(
        &blocks,
        &mut matched_stats,
        &String::from_utf8(sha512_stats.stderr).expect("UTF-8 stats"),
        "sha512 stats",
    );

    let budget_pass = run(
        source("budget"),
        &["test", "--steps", "11595", "--stats", "-"],
    );
    assert!(
        budget_pass.status.success(),
        "{}",
        String::from_utf8_lossy(&budget_pass.stderr)
    );
    mark_stats(
        &blocks,
        &mut matched_stats,
        &String::from_utf8(budget_pass.stderr).expect("UTF-8 stats"),
        "budget 11595",
    );
    let budget_fail = run(
        source("budget"),
        &["test", "--steps", "11594", "--stats", "-"],
    );
    assert_eq!(budget_fail.status.code(), Some(1));
    assert!(budget_fail.stdout.is_empty());
    let diagnostic = String::from_utf8(budget_fail.stderr).expect("UTF-8 diagnostic");
    let hits: Vec<_> = blocks
        .iter()
        .enumerate()
        .filter(|(_, block)| block_equal(block) == diagnostic)
        .map(|(index, _)| index)
        .collect();
    assert_eq!(hits.len(), 1, "budget 11594 diagnostic");
    matched_errors[hits[0]] = true;

    let sha2 = sha2_dir().join("sha2.or");
    let file_stats = run_at(
        &sha2_dir(),
        &["test", "--stats", sha2.to_str().expect("utf-8 path")],
        None,
    );
    assert!(
        file_stats.status.success(),
        "{}",
        String::from_utf8_lossy(&file_stats.stderr)
    );
    let file_stderr = String::from_utf8(file_stats.stderr).expect("UTF-8 stats");
    assert!(file_stderr.contains("NIST example: SHA-512 of abc\": 11595 steps"));
    assert!(file_stderr.contains("NIST example: SHA-512 of the two-block message\": 23145 steps"));
    assert!(file_stderr.contains("total: 196481 of 1048576 steps"));
    mark_stats(&blocks, &mut matched_stats, &file_stderr, "sha2.or stats");

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

    let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let sha2_path = repo.join("algorithms/sha2/sha2.or");
    let sha2_arg = sha2_path.to_str().expect("utf-8 path");
    for (arguments, first_line) in [
        (
            &["check", "--steps", "10", sha2_arg][..],
            "orangec: option `--steps` applies only to eval, test, replay, and analyze\n",
        ),
        (
            &["test", "--steps", "0", sha2_arg][..],
            "orangec: option `--steps` takes a number of steps from 1 through 1073741824\n",
        ),
        (
            &["test", "--steps", "1073741825", sha2_arg][..],
            "orangec: option `--steps` takes a number of steps from 1 through 1073741824\n",
        ),
    ] {
        let output = run_at(&repo, arguments, None);
        assert_eq!(output.status.code(), Some(2), "{arguments:?}");
        let stderr = String::from_utf8(output.stderr).expect("UTF-8 usage");
        assert!(stderr.starts_with(first_line), "{stderr}");
        assert!(J6.contains(first_line.trim_end()), "{first_line}");
    }
}

fn mark_stats(blocks: &[&str], matched: &mut [bool], stderr: &str, label: &str) {
    let hits: Vec<_> = blocks
        .iter()
        .enumerate()
        .filter(|(_, block)| block_equal(block) == stderr)
        .map(|(index, _)| index)
        .collect();
    assert_eq!(hits.len(), 1, "{label}");
    matched[hits[0]] = true;
}
