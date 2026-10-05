//! Execute the Orange Book's actual fenced listings, not copied fixtures.
//! Educational regression evidence only; not a compiler or security proof.

use std::io::Write;
use std::process::{Command, Output, Stdio};

const CHAPTERS: &str = include_str!("../../../../docs/book/NOVICE_PROGRAMMING.md");
const N7: &str = include_str!("../../../../docs/book/NOVICE_N7_NAME_THE_INTERMEDIATE_STEP.md");
const N8: &str = include_str!("../../../../docs/book/NOVICE_N8_READ_AND_REPAIR.md");

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

fn run(command: &str, source: &str) -> Output {
    run_with(&[command, "-"], source)
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

#[test]
fn all_seven_positive_book_listings_check_and_evaluate_repeatably() {
    let sources = fences(CHAPTERS, "orange");
    let outputs = fences(CHAPTERS, "text");
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
        let sources = fences(CHAPTERS, "orange");
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
    let sources = fences(CHAPTERS, "orange");
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

fn n7_sources() -> Vec<&'static str> {
    fences(N7, "orange")
}

#[test]
fn n7_positive_listings_check_and_evaluate_repeatably() {
    let sources = n7_sources();
    let outputs = fences(N7, "text");
    let rejected = ["mixed_conversion", "past_end", "slipped", "no_range"];
    let mut checked = 0;
    for source in &sources {
        let name = module_name(source);
        if rejected.contains(&name) {
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
    assert_eq!(checked, 8, "named_round through bounded");
    assert_eq!(sources.len(), 12);
}

#[test]
fn n7_rejected_listings_print_no_value() {
    for (name, marker) in [
        ("mixed_conversion", "ORC0108"),
        ("past_end", "ORC0223"),
        ("slipped", "ORC0223"),
        ("no_range", "ORC0226"),
    ] {
        let source = n7_sources()
            .into_iter()
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
    let mixed = n7_sources()
        .into_iter()
        .find(|source| module_name(source) == "mixed_conversion")
        .expect("mixed conversion listing");
    let diagnostic = String::from_utf8(run("check", mixed).stderr).expect("UTF-8");
    assert!(
        diagnostic.contains("`as` follows `+` without grouping parentheses"),
        "{diagnostic}"
    );
    let slipped = n7_sources()
        .into_iter()
        .find(|source| module_name(source) == "slipped")
        .expect("slipped index listing");
    let slipped_diagnostic = String::from_utf8(run("check", slipped).stderr).expect("UTF-8");
    assert!(
        slipped_diagnostic
            .contains("this index runs from 1 through 4, out of range for `Word[8]^4`"),
        "{slipped_diagnostic}"
    );
    assert!(
        slipped_diagnostic.contains(
            "every value an index can take, over every loop index and word in it, \
             must select an element"
        ),
        "{slipped_diagnostic}"
    );
    let past_end = n7_sources()
        .into_iter()
        .find(|source| module_name(source) == "past_end")
        .expect("past-end listing");
    let past_end_diagnostic = String::from_utf8(run("check", past_end).stderr).expect("UTF-8");
    assert!(
        past_end_diagnostic.contains("index `4` is out of range for `Word[32]^4`"),
        "{past_end_diagnostic}"
    );
    assert!(
        past_end_diagnostic.contains("a literal index must be less than the array's length"),
        "{past_end_diagnostic}"
    );
    let no_range = n7_sources()
        .into_iter()
        .find(|source| module_name(source) == "no_range")
        .expect("unbounded Int index listing");
    let no_range_diagnostic = String::from_utf8(run("check", no_range).stderr).expect("UTF-8");
    assert!(
        no_range_diagnostic.contains(
            "an `Int` index may use only integer literals, loop indices, and words \
             converted with `as Int`"
        ),
        "{no_range_diagnostic}"
    );
    assert!(
        no_range_diagnostic.contains("this `Int` has no bound"),
        "{no_range_diagnostic}"
    );
    assert!(
        no_range_diagnostic.contains("every index is proved in range when the program is checked"),
        "{no_range_diagnostic}"
    );
}

fn n8_sources() -> Vec<&'static str> {
    fences(N8, "orange")
}

fn n8_text() -> Vec<&'static str> {
    fences(N8, "text")
}

fn n8_source(name: &str) -> &'static str {
    n8_sources()
        .into_iter()
        .find(|source| module_name(source) == name)
        .unwrap_or_else(|| panic!("missing N8 listing {name}"))
}

fn one_text(predicate: impl Fn(&str) -> bool, label: &str) -> &'static str {
    let matches: Vec<_> = n8_text()
        .into_iter()
        .filter(|text| predicate(text))
        .collect();
    assert_eq!(matches.len(), 1, "{label}");
    matches[0]
}

fn eval_fence(name: &str) -> &'static str {
    let prefix = format!("{name}::");
    one_text(
        |text| text.starts_with(&prefix) && text.contains(" = "),
        name,
    )
}

fn test_fence(name: &str) -> &'static str {
    let prefix = format!("test \"{name}");
    one_text(
        |text| {
            text.starts_with(&prefix) && (text.contains("... ok") || text.contains("... FAILED"))
        },
        name,
    )
}

fn diagnostic_fence(marker: &str) -> &'static str {
    one_text(
        |text| text.starts_with("error[") && text.contains(marker),
        marker,
    )
}

fn assert_silent_check(name: &str, source: &str) {
    let check = run("check", source);
    assert!(
        check.status.success(),
        "{name}: {}",
        String::from_utf8_lossy(&check.stderr)
    );
    assert!(check.stdout.is_empty(), "{name}: check printed a value");
    assert!(check.stderr.is_empty(), "{name}: check diagnostics");
}

fn assert_eval_matches(name: &str, source: &str) {
    let expected = format!("{}\n", eval_fence(name));
    let first = run("eval", source);
    assert!(
        first.status.success(),
        "{name}: {}",
        String::from_utf8_lossy(&first.stderr)
    );
    assert_eq!(first.stdout, expected.as_bytes(), "{name}");
    assert!(first.stderr.is_empty(), "{name}: eval diagnostics");
    let second = run("eval", source);
    assert_eq!(first.status.code(), second.status.code());
    assert_eq!(first.stdout, second.stdout);
    assert_eq!(first.stderr, second.stderr);
}

#[test]
fn n8_repaired_listings_check_and_evaluate_repeatably() {
    let sources = n8_sources();
    assert_eq!(sources.len(), 18);
    assert!(!N8.contains("Chapter 8"));
    for name in [
        "silent_wrong",
        "heard_round",
        "grouped_mix",
        "added_widen",
        "last_lane",
        "reversed_bytes",
        "wrong_claim",
        "right_claim",
        "two_specs",
    ] {
        let source = n8_source(name);
        assert_silent_check(name, source);
        assert_eval_matches(name, source);
    }
}

#[test]
fn n8_rejected_listings_match_the_printed_diagnostics() {
    for (name, marker) in [
        ("bare_mix", "a + b ^ a"),
        ("bare_widen", "x + y as Word[32]"),
        ("past_lane", "words[4]"),
        ("slipped_copy", "i + 1"),
        ("torn_round", "d ^ a1 <<< 16"),
        ("bad_field", "quad.4"),
    ] {
        let source = n8_source(name);
        let expected = format!("{}\n", diagnostic_fence(marker));
        for command in ["check", "eval"] {
            let result = run(command, source);
            assert_eq!(result.status.code(), Some(1), "{name}: {command}");
            assert!(result.stdout.is_empty(), "no partial values for {name}");
            assert_eq!(
                String::from_utf8(result.stderr).expect("UTF-8 diagnostic"),
                expected,
                "{name}: {command}"
            );
        }
    }
    let torn = n8_source("torn_round");
    let test_result = run("test", torn);
    assert_eq!(test_result.status.code(), Some(1));
    assert!(
        test_result.stdout.is_empty(),
        "no test report before acceptance"
    );
    assert_eq!(test_result.stderr, run("check", torn).stderr);
}

#[test]
fn n8_tests_separate_a_false_claim_from_a_passing_one() {
    for name in ["plain_false", "recorded_and", "mended_round"] {
        let source = n8_source(name);
        assert_silent_check(name, source);
        let evaluation = run("eval", source);
        assert!(evaluation.status.success(), "{name}");
        assert!(
            evaluation.stdout.is_empty(),
            "{name}: eval had no spec to print"
        );
        assert!(evaluation.stderr.is_empty(), "{name}");
    }
    for (name, status) in [
        ("plain_false", 1),
        ("wrong_claim", 1),
        ("recorded_and", 0),
        ("right_claim", 0),
        ("mended_round", 0),
    ] {
        let source = n8_source(name);
        let expected = format!("{}\n", test_fence(name));
        let first = run("test", source);
        assert_eq!(first.status.code(), Some(status), "{name}");
        assert!(
            first.stderr.is_empty(),
            "{name}: a test report is not a diagnostic"
        );
        assert_eq!(first.stdout, expected.as_bytes(), "{name}");
        let second = run("test", source);
        assert_eq!(first.status.code(), second.status.code());
        assert_eq!(first.stdout, second.stdout);
        assert_eq!(first.stderr, second.stderr);
    }
}

#[test]
fn n8_spec_and_step_budget_match_the_lesson() {
    let grouped = n8_source("grouped_mix");
    let stats = run_with(&["eval", "--stats", "-"], grouped);
    assert!(stats.status.success());
    assert_eq!(
        stats.stdout,
        format!("{}\n", eval_fence("grouped_mix")).as_bytes()
    );
    assert_eq!(
        String::from_utf8(stats.stderr).expect("UTF-8 stats"),
        format!(
            "{}\n",
            one_text(|text| text.contains("8 steps"), "step count")
        )
    );

    let stopped = run_with(&["eval", "--steps", "7", "-"], grouped);
    assert_eq!(stopped.status.code(), Some(1));
    assert!(stopped.stdout.is_empty());
    assert_eq!(
        String::from_utf8(stopped.stderr).expect("UTF-8 budget diagnostic"),
        format!("{}\n", diagnostic_fence("at most 7 evaluation steps"))
    );

    let finished = run_with(&["eval", "--steps", "8", "-"], grouped);
    assert!(finished.status.success());
    assert_eq!(finished.stdout, stats.stdout);
    assert!(finished.stderr.is_empty());

    let zero = run_with(&["eval", "--steps", "0", "-"], grouped);
    assert_eq!(zero.status.code(), Some(2));
    assert!(zero.stdout.is_empty());
    assert!(String::from_utf8_lossy(&zero.stderr).contains("from 1 through 1073741824"));

    let two = n8_source("two_specs");
    let selected = run_with(&["eval", "--spec", "first", "-"], two);
    assert!(selected.status.success());
    assert_eq!(
        selected.stdout,
        b"two_specs::first: Word[32] = 0x11111111\n"
    );
    assert!(selected.stderr.is_empty());

    let parameterized = run_with(
        &["eval", "--spec", "quarter_round", "-"],
        n8_source("wrong_claim"),
    );
    assert_eq!(parameterized.status.code(), Some(1));
    assert!(parameterized.stdout.is_empty());
    let diagnostic = String::from_utf8(parameterized.stderr).expect("UTF-8");
    assert!(diagnostic.contains("ORC1016"), "{diagnostic}");
    assert!(
        diagnostic.contains("no function `quarter_round` without parameters"),
        "{diagnostic}"
    );

    let misuse = run_with(&["test", "--spec", "first", "-"], two);
    assert_eq!(misuse.status.code(), Some(2));
    assert!(misuse.stdout.is_empty());
    assert!(
        String::from_utf8_lossy(&misuse.stderr).contains("option `--spec` applies only to eval")
    );
}

const N11: &str = include_str!("../../../../docs/book/NOVICE_PROTECT.md");

fn n11_sources() -> Vec<&'static str> {
    fences(N11, "orange")
}

fn n11_text() -> Vec<&'static str> {
    fences(N11, "text")
}

fn n11_source(name: &str) -> &'static str {
    n11_sources()
        .into_iter()
        .find(|source| module_name(source) == name)
        .unwrap_or_else(|| panic!("missing N11 listing {name}"))
}

fn n11_one(predicate: impl Fn(&str) -> bool, label: &str) -> &'static str {
    let matches: Vec<_> = n11_text()
        .into_iter()
        .filter(|text| predicate(text))
        .collect();
    assert_eq!(matches.len(), 1, "{label}");
    matches[0]
}

fn n11_eval_fence(name: &str) -> &'static str {
    let prefix = format!("{name}::");
    n11_one(
        |text| text.starts_with(&prefix) && text.contains(" = "),
        name,
    )
}

fn n11_test_fence(title: &str) -> &'static str {
    let prefix = format!("test \"{title}");
    n11_one(
        |text| text.starts_with(&prefix) && text.contains("... ok"),
        title,
    )
}

fn n11_diagnostic_fence(marker: &str) -> &'static str {
    n11_one(
        |text| text.starts_with("error[") && text.contains(marker),
        marker,
    )
}

#[test]
fn n11_listings_check_and_evaluate_repeatably() {
    let sources = n11_sources();
    assert_eq!(sources.len(), 12, "shift through pad");
    assert!(!N11.contains("\n## Chapter 11"));
    assert!(!N11.contains("\n# Chapter 11"));
    let rejected = ["bad_letter", "wide_key", "residue_xor"];
    let mut checked = 0;
    for source in &sources {
        let name = module_name(source);
        if rejected.contains(&name) {
            continue;
        }
        let expected = format!("{}\n", n11_eval_fence(name));
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
        assert_eq!(first.stdout, expected.as_bytes(), "{name}");
        assert!(first.stderr.is_empty(), "{name}: eval diagnostics");
        let second = run("eval", source);
        assert_eq!(first.status.code(), second.status.code());
        assert_eq!(first.stdout, second.stdout);
        assert_eq!(first.stderr, second.stderr);
        checked += 1;
    }
    assert_eq!(checked, 9);
}

#[test]
fn n11_rejected_listings_match_the_printed_diagnostics() {
    for (name, marker) in [
        ("bad_letter", "ORC0207"),
        ("wide_key", "ORC0223"),
        ("residue_xor", "ORC0215"),
    ] {
        let source = n11_source(name);
        let expected = format!("{}\n", n11_diagnostic_fence(marker));
        for command in ["check", "eval"] {
            let result = run(command, source);
            assert_eq!(result.status.code(), Some(1), "{name}: {command}");
            assert!(result.stdout.is_empty(), "no partial values for {name}");
            assert_eq!(
                String::from_utf8(result.stderr).expect("UTF-8 diagnostic"),
                expected,
                "{name}: {command}"
            );
        }
    }
}

#[test]
fn n11_known_answer_tests_pass() {
    for (name, title) in [
        ("shift", "shift HELLO round trip"),
        ("affine", "affine round trip at 7"),
        ("substitution", "keyword substitution round trip"),
        ("pad", "two-time pad cancels the key"),
    ] {
        let source = n11_source(name);
        let expected = format!("{}\n", n11_test_fence(title));
        let first = run("test", source);
        assert_eq!(first.status.code(), Some(0), "{name}");
        assert!(first.stderr.is_empty(), "{name}: a test report is not a diagnostic");
        assert_eq!(first.stdout, expected.as_bytes(), "{name}");
        let second = run("test", source);
        assert_eq!(first.stdout, second.stdout);
        assert_eq!(first.stderr, second.stderr);
    }
}

const N12: &str = include_str!("../../../../docs/book/NOVICE_N12_THE_FIRST_COMPLETE_STUDY.md");

fn n12_sources() -> Vec<&'static str> {
    fences(N12, "orange")
}

fn n12_text() -> Vec<&'static str> {
    fences(N12, "text")
}

fn n12_source(name: &str) -> &'static str {
    n12_sources()
        .into_iter()
        .find(|source| module_name(source) == name)
        .unwrap_or_else(|| panic!("missing N12 listing {name}"))
}

fn n12_one_text(predicate: impl Fn(&str) -> bool, label: &str) -> &'static str {
    let matches: Vec<_> = n12_text()
        .into_iter()
        .filter(|text| predicate(text))
        .collect();
    assert_eq!(matches.len(), 1, "{label}");
    matches[0]
}

fn n12_eval_fence(name: &str) -> &'static str {
    let prefix = format!("{name}::");
    n12_one_text(
        |text| text.starts_with(&prefix) && text.contains(" = "),
        name,
    )
}

fn n12_test_fence(title: &str) -> &'static str {
    let prefix = format!("test \"{title}");
    n12_one_text(
        |text| {
            text.starts_with(&prefix) && (text.contains("... ok") || text.contains("... FAILED"))
        },
        title,
    )
}

#[test]
fn n12_listings_check_and_evaluate_repeatably() {
    let sources = n12_sources();
    assert_eq!(sources.len(), 6, "update coverage when adding N12 listings");
    assert!(!N12.contains("\n## Chapter "));
    for name in [
        "sample_line",
        "quarter_vector",
        "state_quarter",
        "opened_column",
        "chacha_block",
    ] {
        let source = n12_source(name);
        assert_silent_check(name, source);
        let expected = format!("{}\n", n12_eval_fence(name));
        let first = run("eval", source);
        assert!(
            first.status.success(),
            "{name}: {}",
            String::from_utf8_lossy(&first.stderr)
        );
        assert_eq!(first.stdout, expected.as_bytes(), "{name}");
        assert!(first.stderr.is_empty(), "{name}: eval diagnostics");
        let second = run("eval", source);
        assert_eq!(first.status.code(), second.status.code());
        assert_eq!(first.stdout, second.stdout);
        assert_eq!(first.stderr, second.stderr);
    }
}

#[test]
fn n12_shifted_sum_matches_the_printed_diagnostic() {
    let source = n12_source("shifted_sum");
    let expected = format!(
        "{}\n",
        n12_one_text(
            |text| text.starts_with("error[ORC0223]") && text.contains("original[i + 1]"),
            "shifted sum diagnostic"
        )
    );
    for command in ["check", "eval", "test"] {
        let result = run(command, source);
        assert_eq!(result.status.code(), Some(1), "{command}");
        assert!(result.stdout.is_empty(), "{command} printed a value");
        assert_eq!(
            String::from_utf8(result.stderr).expect("UTF-8 diagnostic"),
            expected,
            "{command}"
        );
    }
    assert!(source.contains("original[i + 1]"));
    let repaired = n12_source("chacha_block");
    assert!(repaired.contains("worked[i] + original[i]"));
    assert!(!repaired.contains("original[i + 1]"));
}

#[test]
fn n12_known_answer_tests_match_the_printed_reports() {
    for (module, title) in [
        ("quarter_vector", "RFC 8439 2.1.1"),
        ("state_quarter", "RFC 8439 2.2.1"),
        ("opened_column", "first column of RFC 8439 2.3.2"),
        ("chacha_block", "RFC 8439 2.3.2 words"),
    ] {
        let source = n12_source(module);
        let expected = format!("{}\n", n12_test_fence(title));
        let first = run("test", source);
        assert_eq!(first.status.code(), Some(0), "{module}");
        assert!(first.stderr.is_empty(), "{module}: {}", String::from_utf8_lossy(&first.stderr));
        assert_eq!(first.stdout, expected.as_bytes(), "{module}");
        let second = run("test", source);
        assert_eq!(first.status.code(), second.status.code());
        assert_eq!(first.stdout, second.stdout);
        assert_eq!(first.stderr, second.stderr);
    }
}
