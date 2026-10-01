//! External conformance evidence for the proposed Orange 2026 S3g slice.
//!
//! The rule index lives in `docs/LOOKUPS_2026.md`. This runner requires that
//! index to agree exactly with the evidence map below, runs every fixture and
//! generated source through the real `orangec` binary twice, and checks that
//! every named unit test is declared once in its source's test module.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

const LOOKUPS_SPECIFICATION: &str = include_str!("../../../../docs/LOOKUPS_2026.md");
const S3G_CONFORMANCE_SOURCE: &str = include_str!("s3g_conformance.rs");
const PARSER_SOURCE: &str = include_str!("../../orange-compiler/src/parser.rs");
const SEMANTICS_SOURCE: &str = include_str!("../../orange-compiler/src/semantics.rs");
const SEMANTICS_TESTS_SOURCE: &str = include_str!("../../orange-compiler/src/semantics/tests.rs");
const EVAL_SOURCE: &str = include_str!("../../orange-compiler/src/eval.rs");
const DIAGNOSTIC_SOURCE: &str = include_str!("../../orange-compiler/src/diagnostic.rs");

const CLI: u8 = 1 << 0;
const GENERATED_CLI: u8 = 1 << 1;
const UNIT: u8 = 1 << 2;
const PARSER_UNIT: u8 = 1 << 3;

/// Every S3g rule, in the order of the specification's index.
const RULES: [&str; 10] = [
    "S3G-TYPE-01",
    "S3G-WORD-01",
    "S3G-INT-01",
    "S3G-EVAL-01",
    "S3G-DIAG-01",
    "S3G-CORE-01",
    "S3G-RES-EVENT-01",
    "S3G-RES-STEP-01",
    "S3G-COMPAT-01",
    "S3G-DETERMINISM-01",
];

#[derive(Clone, Copy)]
enum Expectation {
    Success(&'static str),
    Failure {
        codes: &'static [&'static str],
        locations: &'static [&'static str],
        messages: &'static [&'static str],
    },
}

#[derive(Clone, Copy)]
struct Case {
    fixture: &'static str,
    expectation: Expectation,
    rules: &'static [&'static str],
}

#[derive(Clone, Copy)]
struct TestEvidence {
    source_path: &'static str,
    test: &'static str,
    rules: &'static [&'static str],
}

const RANGE_NOTE: &str = "every value an index can take, over every loop index and word in it, \
                          must select an element";

const CASES: [Case; 4] = [
    Case {
        fixture: "valid-lookups.or",
        expectation: Expectation::Success(concat!(
            "lookups::digits: Word[8]^2 = [0x61, 0x37]\n",
            "lookups::counts: Int^16 = [1, 4, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 2]\n",
            "lookups::swapped: Word[8]^4 = [0x01, 0x03, 0x02, 0x04]\n",
            "lookups::narrowed: Word[8]^3 = [0xff, 0x28, 0x0b]\n",
            "lookups::check: Word[32] = 0xcbf43926\n",
        )),
        rules: &[
            "S3G-TYPE-01",
            "S3G-WORD-01",
            "S3G-INT-01",
            "S3G-EVAL-01",
            "S3G-CORE-01",
            "S3G-RES-STEP-01",
            "S3G-COMPAT-01",
            "S3G-DETERMINISM-01",
        ],
    },
    Case {
        fixture: "valid-aes128.or",
        expectation: Expectation::Success(concat!(
            "aes::sbox: Word[8]^256 = [",
            "0x63, 0x7c, 0x77, 0x7b, 0xf2, 0x6b, 0x6f, 0xc5, ",
            "0x30, 0x01, 0x67, 0x2b, 0xfe, 0xd7, 0xab, 0x76, ",
            "0xca, 0x82, 0xc9, 0x7d, 0xfa, 0x59, 0x47, 0xf0, ",
            "0xad, 0xd4, 0xa2, 0xaf, 0x9c, 0xa4, 0x72, 0xc0, ",
            "0xb7, 0xfd, 0x93, 0x26, 0x36, 0x3f, 0xf7, 0xcc, ",
            "0x34, 0xa5, 0xe5, 0xf1, 0x71, 0xd8, 0x31, 0x15, ",
            "0x04, 0xc7, 0x23, 0xc3, 0x18, 0x96, 0x05, 0x9a, ",
            "0x07, 0x12, 0x80, 0xe2, 0xeb, 0x27, 0xb2, 0x75, ",
            "0x09, 0x83, 0x2c, 0x1a, 0x1b, 0x6e, 0x5a, 0xa0, ",
            "0x52, 0x3b, 0xd6, 0xb3, 0x29, 0xe3, 0x2f, 0x84, ",
            "0x53, 0xd1, 0x00, 0xed, 0x20, 0xfc, 0xb1, 0x5b, ",
            "0x6a, 0xcb, 0xbe, 0x39, 0x4a, 0x4c, 0x58, 0xcf, ",
            "0xd0, 0xef, 0xaa, 0xfb, 0x43, 0x4d, 0x33, 0x85, ",
            "0x45, 0xf9, 0x02, 0x7f, 0x50, 0x3c, 0x9f, 0xa8, ",
            "0x51, 0xa3, 0x40, 0x8f, 0x92, 0x9d, 0x38, 0xf5, ",
            "0xbc, 0xb6, 0xda, 0x21, 0x10, 0xff, 0xf3, 0xd2, ",
            "0xcd, 0x0c, 0x13, 0xec, 0x5f, 0x97, 0x44, 0x17, ",
            "0xc4, 0xa7, 0x7e, 0x3d, 0x64, 0x5d, 0x19, 0x73, ",
            "0x60, 0x81, 0x4f, 0xdc, 0x22, 0x2a, 0x90, 0x88, ",
            "0x46, 0xee, 0xb8, 0x14, 0xde, 0x5e, 0x0b, 0xdb, ",
            "0xe0, 0x32, 0x3a, 0x0a, 0x49, 0x06, 0x24, 0x5c, ",
            "0xc2, 0xd3, 0xac, 0x62, 0x91, 0x95, 0xe4, 0x79, ",
            "0xe7, 0xc8, 0x37, 0x6d, 0x8d, 0xd5, 0x4e, 0xa9, ",
            "0x6c, 0x56, 0xf4, 0xea, 0x65, 0x7a, 0xae, 0x08, ",
            "0xba, 0x78, 0x25, 0x2e, 0x1c, 0xa6, 0xb4, 0xc6, ",
            "0xe8, 0xdd, 0x74, 0x1f, 0x4b, 0xbd, 0x8b, 0x8a, ",
            "0x70, 0x3e, 0xb5, 0x66, 0x48, 0x03, 0xf6, 0x0e, ",
            "0x61, 0x35, 0x57, 0xb9, 0x86, 0xc1, 0x1d, 0x9e, ",
            "0xe1, 0xf8, 0x98, 0x11, 0x69, 0xd9, 0x8e, 0x94, ",
            "0x9b, 0x1e, 0x87, 0xe9, 0xce, 0x55, 0x28, 0xdf, ",
            "0x8c, 0xa1, 0x89, 0x0d, 0xbf, 0xe6, 0x42, 0x68, ",
            "0x41, 0x99, 0x2d, 0x0f, 0xb0, 0x54, 0xbb, 0x16]\n",
            "aes::example_substitution: Word[8] = 0xed\n",
            "aes::example_b: Word[8]^16 = [",
            "0x39, 0x25, 0x84, 0x1d, 0x02, 0xdc, 0x09, 0xfb, ",
            "0xdc, 0x11, 0x85, 0x97, 0x19, 0x6a, 0x0b, 0x32]\n",
            "aes::c1_key: Word[8]^16 = [",
            "0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, ",
            "0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f]\n",
            "aes::example_c1: Word[8]^16 = [",
            "0x69, 0xc4, 0xe0, 0xd8, 0x6a, 0x7b, 0x04, 0x30, ",
            "0xd8, 0xcd, 0xb7, 0x80, 0x70, 0xb4, 0xc5, 0x5a]\n",
            "aes::example_c1_inverse: Word[8]^16 = [",
            "0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, ",
            "0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff]\n",
        )),
        rules: &[
            "S3G-TYPE-01",
            "S3G-WORD-01",
            "S3G-INT-01",
            "S3G-EVAL-01",
            "S3G-CORE-01",
            "S3G-RES-STEP-01",
            "S3G-DETERMINISM-01",
        ],
    },
    Case {
        fixture: "invalid-word-indices.or",
        expectation: Expectation::Failure {
            codes: &[
                "ORC0223", "ORC0223", "ORC0223", "ORC0223", "ORC0223", "ORC0223", "ORC0223",
                "ORC0223",
            ],
            locations: &[
                "6:56", "7:57", "8:55", "9:56", "10:56", "11:58", "12:66", "13:66",
            ],
            messages: &[
                "this index runs from 0 through 255, out of range for `Word[8]^16`",
                "this index runs from 0 through 65535, out of range for `Word[8]^256`",
                "this index runs from 0 through 31, out of range for `Word[8]^16`",
                "this index runs from 1 through 16, out of range for `Word[8]^16`",
                "this index runs from 0 through 16, out of range for `Word[8]^16`",
                "indices run from 0 through 15",
                RANGE_NOTE,
            ],
        },
        rules: &[
            "S3G-TYPE-01",
            "S3G-WORD-01",
            "S3G-DIAG-01",
            "S3G-DETERMINISM-01",
        ],
    },
    Case {
        fixture: "invalid-int-indices.or",
        expectation: Expectation::Failure {
            codes: &[
                "ORC0226", "ORC0226", "ORC0226", "ORC0226", "ORC0223", "ORC0223",
            ],
            locations: &["6:55", "7:77", "8:55", "10:42", "11:57", "12:57"],
            messages: &[
                "an `Int` index may use only integer literals, loop indices, and words converted \
                 with `as Int`",
                "this `Int` has no bound",
                "a word index ranges over its type",
                "this index runs from 1 through 256, out of range for `Word[8]^256`",
                "this index runs from -1 through 254, out of range for `Word[8]^256`",
                RANGE_NOTE,
            ],
        },
        rules: &["S3G-INT-01", "S3G-DIAG-01", "S3G-DETERMINISM-01"],
    },
];

/// Generated command-line cases in this file, each run twice.
const GENERATED_EVIDENCE: &[(&str, &[&str])] = &[(
    "s3g_update_costs_and_whole_byte_lookups_are_exact",
    &[
        "S3G-TYPE-01",
        "S3G-WORD-01",
        "S3G-EVAL-01",
        "S3G-RES-STEP-01",
        "S3G-DETERMINISM-01",
    ],
)];

const UNIT_EVIDENCE: &[TestEvidence] = &[
    TestEvidence {
        source_path: "src/semantics/tests.rs",
        test: "word_indices_range_over_their_types_and_narrow_through_operators",
        rules: &[
            "S3G-TYPE-01",
            "S3G-WORD-01",
            "S3G-INT-01",
            "S3G-DIAG-01",
            "S3G-CORE-01",
            "S3G-DETERMINISM-01",
        ],
    },
    TestEvidence {
        source_path: "src/semantics/tests.rs",
        test: "word_index_events_and_core_nodes_follow_the_normative_accounting",
        rules: &["S3G-CORE-01", "S3G-RES-EVENT-01"],
    },
    TestEvidence {
        source_path: "src/semantics/tests.rs",
        test: "index_ranges_follow_euclidean_division_and_its_total_rules",
        rules: &["S3G-INT-01", "S3G-DIAG-01", "S3G-COMPAT-01"],
    },
    TestEvidence {
        source_path: "src/parser.rs",
        test: "conditional_words_are_recognized_only_by_position",
        rules: &["S3G-COMPAT-01"],
    },
    TestEvidence {
        source_path: "src/eval.rs",
        test: "word_indices_select_and_update_by_value",
        rules: &["S3G-TYPE-01", "S3G-EVAL-01", "S3G-DETERMINISM-01"],
    },
    TestEvidence {
        source_path: "src/eval.rs",
        test: "steps_follow_the_normative_cost_table",
        rules: &["S3G-RES-STEP-01", "S3G-COMPAT-01"],
    },
    TestEvidence {
        source_path: "src/eval.rs",
        test: "loops_updates_and_fills_evaluate_in_index_order",
        rules: &["S3G-COMPAT-01"],
    },
    TestEvidence {
        source_path: "src/diagnostic.rs",
        test: "diagnostic_code_inventory_is_exact_ordered_and_unique",
        rules: &["S3G-DIAG-01"],
    },
];

fn orangec() -> Command {
    Command::new(env!("CARGO_BIN_EXE_orangec"))
}

fn fixture_directory() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/s3g")
}

fn run_fixture(command: &str, path: &Path) -> Output {
    orangec().arg(command).arg(path).output().unwrap()
}

fn run_stdin(command: &str, source: &str) -> Output {
    let mut child = orangec()
        .arg(command)
        .arg("-")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(source.as_bytes())
        .unwrap();
    child.wait_with_output().unwrap()
}

/// Runs `command` twice on `source` and returns the first output after
/// requiring the second to be byte-identical.
fn run_twice(command: &str, source: &str, context: &str) -> Output {
    let first = run_stdin(command, source);
    let second = run_stdin(command, source);
    assert_repeatable(&first, &second, context);
    first
}

fn assert_repeatable(first: &Output, second: &Output, context: &str) {
    assert_eq!(
        first.status.code(),
        second.status.code(),
        "{context} changed exit status"
    );
    assert_eq!(first.stdout, second.stdout, "{context} changed stdout");
    assert_eq!(first.stderr, second.stderr, "{context} changed stderr");
}

fn diagnostic_codes(stderr: &str) -> Vec<&str> {
    stderr
        .lines()
        .filter_map(|line| {
            line.strip_prefix("error[")
                .and_then(|suffix| suffix.split_once(']'))
                .map(|(code, _)| code)
        })
        .collect()
}

fn primary_locations(stderr: &str) -> Vec<String> {
    stderr
        .lines()
        .filter_map(|line| line.strip_prefix(" --> "))
        .filter_map(|location| {
            let (prefix, column) = location.rsplit_once(':')?;
            let (_, line) = prefix.rsplit_once(':')?;
            Some(format!("{line}:{column}"))
        })
        .collect()
}

fn assert_success(output: &Output, stdout: &str, context: &str) {
    assert_eq!(
        output.status.code(),
        Some(0),
        "{context} failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        stdout,
        "{context} stdout"
    );
    assert_eq!(output.stderr, b"", "{context} stderr");
}

fn assert_failure(
    output: &Output,
    codes: &[&str],
    locations: &[&str],
    messages: &[&str],
    context: &str,
) {
    assert_eq!(output.status.code(), Some(1), "{context} status");
    assert_eq!(output.stdout, b"", "{context} emitted partial output");
    let stderr = std::str::from_utf8(&output.stderr).unwrap();
    assert_eq!(
        diagnostic_codes(stderr),
        codes,
        "{context} codes:\n{stderr}"
    );
    assert_eq!(
        primary_locations(stderr),
        locations,
        "{context} primary locations:\n{stderr}"
    );
    for message in messages {
        assert!(
            stderr.contains(message),
            "{context} missing {message:?}:\n{stderr}"
        );
    }
}

fn module(members: &str) -> String {
    format!("edition 2026;\nmodule limits {{\n{members}}}\n")
}

/// The rule index rows of the specification: rule ID and evidence label.
fn documented_rules() -> Vec<(&'static str, &'static str)> {
    LOOKUPS_SPECIFICATION
        .lines()
        .filter_map(|line| {
            let row = line.strip_prefix("| `")?;
            let (rule, _) = row.split_once("` |")?;
            if !rule.starts_with("S3G-") {
                return None;
            }
            let evidence = line.strip_suffix('|')?.rsplit('|').next()?.trim();
            Some((rule, evidence))
        })
        .collect()
}

fn required_layers(label: &str) -> u8 {
    match label {
        "CLI and unit" | "Unit and CLI observation" => CLI | UNIT,
        "CLI and parser unit" => CLI | UNIT | PARSER_UNIT,
        "Generated CLI and unit" => GENERATED_CLI | UNIT,
        "Generated CLI and parser unit" => GENERATED_CLI | UNIT | PARSER_UNIT,
        "Unit" => UNIT,
        _ => panic!("unknown S3g evidence layer {label:?}"),
    }
}

fn unit_source(source_path: &str) -> &'static str {
    match source_path {
        "src/parser.rs" => PARSER_SOURCE,
        "src/semantics.rs" => SEMANTICS_SOURCE,
        "src/semantics/tests.rs" => SEMANTICS_TESTS_SOURCE,
        "src/eval.rs" => EVAL_SOURCE,
        "src/diagnostic.rs" => DIAGNOSTIC_SOURCE,
        _ => panic!("unmapped S3g evidence source {source_path}"),
    }
}

/// Requires `test` to be declared exactly once, as a `#[test]` function
/// directly inside the source's single `#[cfg(test)] mod tests` module. The
/// module is written inline, or in its own `tests.rs` file that its parent
/// declares once as `#[cfg(test)] mod tests;` with no other attribute.
fn assert_unit_test_declared(source_path: &str, test: &str) {
    let source = unit_source(source_path);
    let (tests, declaration) = match source_path.strip_suffix("/tests.rs") {
        Some(parent) => {
            let parent_path = format!("{parent}.rs");
            let parent_source = unit_source(&parent_path);
            assert_eq!(
                parent_source.matches("mod tests").count(),
                1,
                "{parent_path} must declare exactly one test module"
            );
            assert_eq!(
                parent_source
                    .matches("\n#[cfg(test)]\nmod tests;\n")
                    .count(),
                1,
                "{parent_path} must declare its test module unconditionally"
            );
            assert!(
                !parent_source.contains("]\n#[cfg(test)]\nmod tests;"),
                "{parent_path} must not add an attribute to its test module"
            );
            assert!(
                !source.contains("#!["),
                "{source_path} must not carry an inner attribute"
            );
            (source, format!("\n#[test]\nfn {test}() {{\n"))
        }
        None => {
            let marker = "\n#[cfg(test)]\nmod tests {\n";
            assert_eq!(
                source.matches(marker).count(),
                1,
                "{source_path} must have exactly one unconditional test module"
            );
            let (_, tests) = source.split_once(marker).unwrap();
            (tests, format!("\n    #[test]\n    fn {test}() {{\n"))
        }
    };
    assert_eq!(
        tests.matches(&declaration).count(),
        1,
        "{source_path} must declare #[test] fn {test} exactly once in its test module"
    );
    assert_eq!(
        source.matches(&format!("fn {test}(")).count(),
        1,
        "{source_path} declares {test} more than once"
    );
}

fn assert_generated_test_declared(test: &str) {
    let declaration = format!("\n#[test]\nfn {test}() {{\n");
    assert_eq!(
        S3G_CONFORMANCE_SOURCE.matches(&declaration).count(),
        1,
        "generated evidence {test} must be declared exactly once at file root"
    );
}

#[test]
fn s3g_rule_index_is_exact_and_covered() {
    let documented = documented_rules();
    let documented_ids: Vec<_> = documented.iter().map(|(rule, _)| *rule).collect();
    assert_eq!(
        documented_ids, RULES,
        "docs/LOOKUPS_2026.md rule index drifted from the S3g runner"
    );
    let known: BTreeSet<_> = RULES.iter().copied().collect();
    assert_eq!(known.len(), RULES.len(), "duplicate S3g rule ID");

    let mut observed = BTreeMap::<&str, u8>::new();
    let mut record = |rule: &'static str, layers: u8, context: &str| {
        assert!(known.contains(rule), "unknown rule {rule} on {context}");
        *observed.entry(rule).or_default() |= layers;
    };

    let mut fixtures = BTreeSet::new();
    for case in CASES {
        assert!(fixtures.insert(case.fixture), "duplicate {}", case.fixture);
        assert!(!case.rules.is_empty(), "{} has no rules", case.fixture);
        for rule in case.rules {
            record(rule, CLI, case.fixture);
        }
    }

    let mut generated = BTreeSet::new();
    for (test, rules) in GENERATED_EVIDENCE {
        assert!(generated.insert(*test), "duplicate generated test {test}");
        assert_generated_test_declared(test);
        for rule in *rules {
            record(rule, GENERATED_CLI, test);
        }
    }

    let mut unit_tests = BTreeSet::new();
    for evidence in UNIT_EVIDENCE {
        assert!(
            unit_tests.insert((evidence.source_path, evidence.test)),
            "duplicate unit evidence {}",
            evidence.test
        );
        assert!(!evidence.rules.is_empty(), "{} has no rules", evidence.test);
        assert_unit_test_declared(evidence.source_path, evidence.test);
        let layers = if evidence.source_path == "src/parser.rs" {
            UNIT | PARSER_UNIT
        } else {
            UNIT
        };
        for rule in evidence.rules {
            record(rule, layers, evidence.test);
        }
    }

    for (rule, label) in documented {
        let required = required_layers(label);
        let actual = observed.get(rule).copied().unwrap_or_default();
        assert_eq!(
            actual & required,
            required,
            "{rule} requires {label}, but its evidence map provides layers {actual:#06b}"
        );
    }
}

#[test]
fn s3g_cli_conformance_corpus_is_exact_and_repeatable() {
    let directory = fixture_directory();
    let mut observed: Vec<_> = fs::read_dir(&directory)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().into_string().unwrap())
        .collect();
    observed.sort();
    let mut expected: Vec<_> = CASES.iter().map(|case| case.fixture).collect();
    expected.sort_unstable();
    assert_eq!(observed, expected, "unexpected S3g fixture inventory");

    for case in CASES {
        let path = directory.join(case.fixture);
        let first_check = run_fixture("check", &path);
        let second_check = run_fixture("check", &path);
        let first_eval = run_fixture("eval", &path);
        let second_eval = run_fixture("eval", &path);
        let check = format!("{} check", case.fixture);
        let eval = format!("{} eval", case.fixture);
        assert_repeatable(&first_check, &second_check, &check);
        assert_repeatable(&first_eval, &second_eval, &eval);

        match case.expectation {
            Expectation::Success(stdout) => {
                assert_success(&first_check, "", &check);
                assert_success(&first_eval, stdout, &eval);
            }
            Expectation::Failure {
                codes,
                locations,
                messages,
            } => {
                assert_failure(&first_check, codes, locations, messages, &check);
                assert_failure(&first_eval, codes, locations, messages, &eval);
                assert_eq!(
                    first_check.stderr, first_eval.stderr,
                    "{} check and eval diagnostics differ",
                    case.fixture
                );
            }
        }
    }
}

#[test]
fn s3g_update_costs_and_whole_byte_lookups_are_exact() {
    // An update or fill of 256 elements costs 4 steps. Each inner iteration
    // costs 8: the iteration, the reads of `u` and `i`, the literal `1`, and
    // the update. Each outer iteration costs 3 + 8 * 256: the iteration, the
    // read of `s`, the inner loop, and its iterations. The fill `[0; 256]`
    // costs 5, the outer loop 1, `t[0]` 2, the last loop 1, and each of its
    // iterations 2. So the budget of 1,048,576 steps is spent exactly, where
    // one step per element would have spent it more than thirty times over.
    const BUDGET: u64 = 1_048_576;
    const OUTER: u64 = 511;
    let inner = 3 + 8 * 256;
    let rest = BUDGET - 9 - OUTER * inner;
    assert_eq!(rest % 2, 0);
    let churn = |extra: &str| {
        module(&format!(
            concat!(
                "  spec churn() -> Word[8] {{\n",
                "    let t: Word[8]^256 = for j in 0..{} with s: Word[8]^256 = [0; 256] {{\n",
                "      for i in 0..256 with u: Word[8]^256 = s {{ u with [i] = 1 }}\n",
                "    }};\n",
                "    {}(for r in 0..{} with w: Word[8] = t[0] {{ w }})\n",
                "  }}\n",
            ),
            OUTER,
            extra,
            rest / 2
        ))
    };
    let exact = run_twice("eval", &churn(""), "exact budget");
    assert_success(&exact, "limits::churn: Word[8] = 0x01\n", "exact budget");
    let over = run_twice("eval", &churn("~"), "one step over");
    assert_failure(
        &over,
        &["ORC0301"],
        &["3:8"],
        &[
            "reference evaluation step limit exceeded",
            "at most 1048576 evaluation steps are permitted",
        ],
        "one step over",
    );

    // Every byte selects its own element: a permutation of the 256 bytes,
    // inverted by updates keyed by its values, composes to the identity.
    let permutation = (0..256_u32)
        .map(|k| format!("{}", (k * 167 + 13) % 256))
        .collect::<Vec<_>>()
        .join(", ");
    let identity = module(&format!(
        concat!(
            "  spec invert(t: Word[8]^256) -> Word[8]^256 {{\n",
            "    for i in 0..256 with u: Word[8]^256 = [0; 256] {{ u with [t[i]] = i as Word[8] }}\n",
            "  }}\n",
            "  spec fixed() -> Int {{\n",
            "    let t: Word[8]^256 = [{}];\n",
            "    let u: Word[8]^256 = invert(t);\n",
            "    for i in 0..256 with n: Int = 0 {{ if u[t[i]] == (i as Word[8]) {{ n + 1 }} else {{ n }} }}\n",
            "  }}\n",
        ),
        permutation
    ));
    let fixed = run_twice("eval", &identity, "permutation");
    assert_success(&fixed, "limits::fixed: Int = 256\n", "permutation");
}
