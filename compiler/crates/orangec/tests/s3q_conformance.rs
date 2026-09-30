//! External conformance evidence for the proposed Orange 2026 S3q slice.
//!
//! The rule index lives in `docs/TESTS_2026.md`. This runner requires that
//! index to agree exactly with the evidence map below, runs every fixture and
//! every generated program through the real `orangec` binary twice, and
//! checks that every named unit test is declared once in its source's test
//! module. S3p compatibility is also observed by the S2 through S3p runners,
//! which run their fixtures through the same binary.

use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const TESTS_SPECIFICATION: &str = include_str!("../../../../docs/TESTS_2026.md");
const S3Q_CONFORMANCE_SOURCE: &str = include_str!("s3q_conformance.rs");
const PARSER_SOURCE: &str = include_str!("../../orange-compiler/src/parser.rs");
const SEMANTICS_SOURCE: &str = include_str!("../../orange-compiler/src/semantics.rs");
const SEMANTICS_TESTS_SOURCE: &str = include_str!("../../orange-compiler/src/semantics/tests.rs");
const CORE_SOURCE: &str = include_str!("../../orange-compiler/src/core.rs");
const DIAGNOSTIC_SOURCE: &str = include_str!("../../orange-compiler/src/diagnostic.rs");
const EVAL_SOURCE: &str = include_str!("../../orange-compiler/src/eval.rs");
const ORANGEC_SOURCE: &str = include_str!("../src/main.rs");

const CLI: u8 = 1 << 0;
const GENERATED_CLI: u8 = 1 << 1;
const UNIT: u8 = 1 << 2;
const PARSER_UNIT: u8 = 1 << 3;

/// Every S3q rule, in the order of the specification's index.
const RULES: [&str; 12] = [
    "S3Q-SYNTAX-01",
    "S3Q-TITLE-01",
    "S3Q-CHECK-01",
    "S3Q-ROOT-01",
    "S3Q-EQUAL-01",
    "S3Q-COST-01",
    "S3Q-RUN-01",
    "S3Q-STOP-01",
    "S3Q-OPTIONS-01",
    "S3Q-RES-01",
    "S3Q-COMPAT-01",
    "S3Q-DETERMINISM-01",
];

#[derive(Clone, Copy)]
enum Expectation {
    /// The exact exit status, standard output, and standard error.
    Exact {
        status: i32,
        stdout: fn() -> String,
        stderr: &'static str,
    },
    /// Diagnostics: status 1, no output, and the codes, primary locations,
    /// and messages given.
    Failure {
        codes: &'static [&'static str],
        locations: &'static [&'static str],
        messages: &'static [&'static str],
    },
}

/// One command run on a fixture: the arguments before its path.
#[derive(Clone, Copy)]
struct Run {
    arguments: &'static [&'static str],
    expectation: Expectation,
}

#[derive(Clone, Copy)]
struct Case {
    fixture: &'static str,
    runs: &'static [Run],
    rules: &'static [&'static str],
}

#[derive(Clone, Copy)]
struct TestEvidence {
    source_path: &'static str,
    test: &'static str,
    rules: &'static [&'static str],
}

fn nothing() -> String {
    String::new()
}

/// RFC 8439 appendix A.1 test vector 1: the key stream of the zero key and
/// nonce at block 0, which `eval` prints.
const A1_1_KEY_STREAM: &str = concat!(
    "76b8e0ada0f13d90405d6ae55386bd28bdd219b8a08ded1aa836efcc8b770dc7",
    "da41597c5157488d7724e03fb8d84a376a43b8f41518a11cc387b669b2ee6586",
);

fn rfc8439_eval_stdout() -> String {
    let bytes = (0..A1_1_KEY_STREAM.len())
        .step_by(2)
        .map(|at| format!("0x{}", &A1_1_KEY_STREAM[at..at + 2]))
        .collect::<Vec<_>>();
    format!(
        "rfc8439::zero_key_stream: Word[8]^64 = [{}]\n",
        bytes.join(", ")
    )
}

fn rfc8439_test_stdout() -> String {
    String::from(concat!(
        "test \"2.1.1: the quarter round\" ... ok\n",
        "test \"2.3.2: the block function\" ... ok\n",
        "test \"A.1 #1: the zero key's key stream, block 0\" ... ok\n",
        "test \"A.1 #2: the zero key's key stream, block 1\" ... ok\n",
        "test \"the nonce changes every block\" ... ok\n",
        "test \"2.5.2: Poly1305 of the Forum's name\" ... ok\n",
        "test \"A.3 #1: Poly1305 of zeros under the zero key\" ... ok\n",
        "7 tests: 7 passed, 0 failed\n",
    ))
}

fn failing_test_stdout() -> String {
    String::from(concat!(
        "test \"a word, rotated\" ... ok\n",
        "test \"a word, rotated the wrong way\" ... FAILED\n",
        "    left:  0x00000080\n",
        "    right: 0x00000100\n",
        "test \"an array\" ... FAILED\n",
        "    left:  [0x01, 0x02, 0x03, 0x04]\n",
        "    right: [0x01, 0x02, 0x09, 0x04]\n",
        "    first difference at [2]\n",
        "test \"a tuple holding an array\" ... FAILED\n",
        "    left:  (0x01, [0x02, 0x03, 0x04])\n",
        "    right: (0x01, [0x02, 0x03, 0x05])\n",
        "    first difference at .1[2]\n",
        "test \"a residue\" ... FAILED\n",
        "    left:  1\n",
        "    right: 2\n",
        "test \"two claims at once\" ... FAILED\n",
        "test \"unequal, as claimed\" ... ok\n",
        "7 tests: 2 passed, 5 failed\n",
    ))
}

fn equality_eval_stdout() -> String {
    String::from(concat!(
        "equality::words: (Bool, Bool, Bool) = (true, false, false)\n",
        "equality::truths: Bool = true\n",
        "equality::residues: Bool = true\n",
        "equality::states: (Bool, Bool) = (false, true)\n",
        "equality::long: (Bool, Bool) = (false, true)\n",
        "equality::compared_first: Bool = false\n",
        "equality::compared_last: Bool = false\n",
    ))
}

fn no_tests_stdout() -> String {
    String::from("0 tests: 0 passed, 0 failed\n")
}

const CLEAN_CHECK: Run = Run {
    arguments: &["check"],
    expectation: Expectation::Exact {
        status: 0,
        stdout: nothing,
        stderr: "",
    },
};

const INVALID_TESTS: Expectation = Expectation::Failure {
    codes: &[
        "ORC0242", "ORC0242", "ORC0242", "ORC0242", "ORC0214", "ORC0215", "ORC0215", "ORC0227",
        "ORC0212",
    ],
    locations: &[
        "invalid-tests.or:8:8",
        "invalid-tests.or:9:8",
        "invalid-tests.or:10:8",
        "invalid-tests.or:12:8",
        "invalid-tests.or:13:21",
        "invalid-tests.or:14:33",
        "invalid-tests.or:15:36",
        "invalid-tests.or:16:25",
        "invalid-tests.or:17:32",
    ],
    messages: &[
        "error[ORC0242]: this test's title is empty\n",
        "^^ a title names the test in every report\n",
        "error[ORC0242]: a test's title holds U+00E9, which is not printable ASCII\n",
        "^^^^^^^^^^^^^^^^^^^ at byte 3 of the title\n",
        "error[ORC0242]: a test's title holds no backslash\n",
        "^^^^^^^^^^^^^^^ titles have no escapes\n",
        "error[ORC0242]: two tests of this module share a title\n",
        "^^^^^^^ this title repeats an earlier test's\n",
        "------- first test is here\n",
        "= note: a test's title is 1 through 128 printable ASCII characters, with no \
         backslash, and no two tests of a module share one\n",
        "error[ORC0214]: an integer literal cannot have type `Bool`\n",
        "error[ORC0215]: `<` is not defined for `(Int, Int)`\n",
        "= note: tuples are compared whole with `==` and `!=`; they have no order, so compare \
         elements, such as `p.0 < q.0`\n",
        "error[ORC0215]: `<=` is not defined for arrays and tuples\n",
        "^^ both operands are written out as arrays or tuples\n",
        "= note: arrays and tuples are compared whole with `==` and `!=`; they have no order, \
         so compare elements\n",
        "error[ORC0227]: the operands of `==` have no type of their own\n",
        "an array or tuple written out takes its type from where it is used\n",
        "error[ORC0212]: no typed `spec` function named `missing` in this module\n",
    ],
};

const INVALID_TEST_SYNTAX: Expectation = Expectation::Failure {
    codes: &["ORC0101"],
    locations: &["invalid-test-syntax.or:4:8"],
    messages: &[
        "error[ORC0101]: expected a quoted title after `test`\n",
        "= note: a test is written `test \"TITLE\" { EXPRESSION }`, its expression a `Bool`\n",
    ],
};

const CASES: [Case; 5] = [
    Case {
        fixture: "valid-rfc8439-tests.or",
        runs: &[
            CLEAN_CHECK,
            Run {
                arguments: &["test", "--stats"],
                expectation: Expectation::Exact {
                    status: 0,
                    stdout: rfc8439_test_stdout,
                    stderr: concat!(
                        "test \"2.1.1: the quarter round\": 53 steps\n",
                        "test \"2.3.2: the block function\": 4404 steps\n",
                        "test \"A.1 #1: the zero key's key stream, block 0\": 4405 steps\n",
                        "test \"A.1 #2: the zero key's key stream, block 1\": 4405 steps\n",
                        "test \"the nonce changes every block\": 8792 steps\n",
                        "test \"2.5.2: Poly1305 of the Forum's name\": 2499 steps\n",
                        "test \"A.3 #1: Poly1305 of zeros under the zero key\": 4191 steps\n",
                        "total: 28749 of 1048576 steps\n",
                    ),
                },
            },
            // Evaluation runs the one function without parameters and none
            // of the tests.
            Run {
                arguments: &["eval"],
                expectation: Expectation::Exact {
                    status: 0,
                    stdout: rfc8439_eval_stdout,
                    stderr: "",
                },
            },
        ],
        rules: &[
            "S3Q-SYNTAX-01",
            "S3Q-TITLE-01",
            "S3Q-CHECK-01",
            "S3Q-EQUAL-01",
            "S3Q-RUN-01",
            "S3Q-OPTIONS-01",
            "S3Q-DETERMINISM-01",
        ],
    },
    Case {
        fixture: "failing-tests.or",
        runs: &[
            CLEAN_CHECK,
            Run {
                arguments: &["test"],
                expectation: Expectation::Exact {
                    status: 1,
                    stdout: failing_test_stdout,
                    stderr: "",
                },
            },
            Run {
                arguments: &["eval"],
                expectation: Expectation::Exact {
                    status: 0,
                    stdout: nothing,
                    stderr: "",
                },
            },
        ],
        rules: &[
            "S3Q-CHECK-01",
            "S3Q-EQUAL-01",
            "S3Q-RUN-01",
            "S3Q-DETERMINISM-01",
        ],
    },
    Case {
        fixture: "valid-equality.or",
        runs: &[
            CLEAN_CHECK,
            Run {
                arguments: &["eval", "--stats"],
                expectation: Expectation::Exact {
                    status: 0,
                    stdout: equality_eval_stdout,
                    stderr: concat!(
                        "equality::words: 41 steps\n",
                        "equality::truths: 14 steps\n",
                        "equality::residues: 13 steps\n",
                        "equality::states: 44 steps\n",
                        "equality::long: 4106 steps\n",
                        "equality::compared_first: 17 steps\n",
                        "equality::compared_last: 17 steps\n",
                        "total: 4252 of 1048576 steps\n",
                    ),
                },
            },
            Run {
                arguments: &["test"],
                expectation: Expectation::Exact {
                    status: 0,
                    stdout: no_tests_stdout,
                    stderr: "",
                },
            },
        ],
        rules: &[
            "S3Q-EQUAL-01",
            "S3Q-COST-01",
            "S3Q-RUN-01",
            "S3Q-COMPAT-01",
            "S3Q-DETERMINISM-01",
        ],
    },
    Case {
        fixture: "invalid-tests.or",
        runs: &[
            Run {
                arguments: &["check"],
                expectation: INVALID_TESTS,
            },
            Run {
                arguments: &["test"],
                expectation: INVALID_TESTS,
            },
            Run {
                arguments: &["eval"],
                expectation: INVALID_TESTS,
            },
        ],
        rules: &[
            "S3Q-TITLE-01",
            "S3Q-CHECK-01",
            "S3Q-EQUAL-01",
            "S3Q-COMPAT-01",
            "S3Q-DETERMINISM-01",
        ],
    },
    Case {
        fixture: "invalid-test-syntax.or",
        runs: &[
            Run {
                arguments: &["check"],
                expectation: INVALID_TEST_SYNTAX,
            },
            Run {
                arguments: &["test"],
                expectation: INVALID_TEST_SYNTAX,
            },
        ],
        rules: &["S3Q-SYNTAX-01", "S3Q-DETERMINISM-01"],
    },
];

/// Generated command-line cases in this file, each run twice.
const GENERATED_EVIDENCE: &[(&str, &[&str])] = &[
    (
        "s3q_titles_hold_up_to_128_printable_ascii_bytes",
        &["S3Q-TITLE-01", "S3Q-RUN-01", "S3Q-DETERMINISM-01"],
    ),
    (
        "s3q_only_the_root_modules_tests_are_checked_and_run",
        &[
            "S3Q-SYNTAX-01",
            "S3Q-CHECK-01",
            "S3Q-ROOT-01",
            "S3Q-RUN-01",
            "S3Q-DETERMINISM-01",
        ],
    ),
    (
        "s3q_a_test_that_stops_ends_the_run_with_no_report",
        &[
            "S3Q-RUN-01",
            "S3Q-STOP-01",
            "S3Q-OPTIONS-01",
            "S3Q-DETERMINISM-01",
        ],
    ),
    (
        "s3q_test_takes_one_source_and_the_step_options_but_not_spec",
        &["S3Q-OPTIONS-01", "S3Q-DETERMINISM-01"],
    ),
    (
        "s3q_comparisons_cost_the_same_wherever_the_operands_differ",
        &["S3Q-EQUAL-01", "S3Q-COST-01", "S3Q-DETERMINISM-01"],
    ),
];

const UNIT_EVIDENCE: &[TestEvidence] = &[
    TestEvidence {
        source_path: "src/parser.rs",
        test: "parses_test_declarations_among_functions_with_exact_spans",
        rules: &["S3Q-SYNTAX-01"],
    },
    TestEvidence {
        source_path: "src/parser.rs",
        test: "test_titles_are_kept_as_written_for_the_checker",
        rules: &["S3Q-SYNTAX-01", "S3Q-TITLE-01", "S3Q-RES-01"],
    },
    TestEvidence {
        source_path: "src/parser.rs",
        test: "rejects_malformed_test_declarations_with_exact_messages",
        rules: &["S3Q-SYNTAX-01", "S3Q-DETERMINISM-01"],
    },
    TestEvidence {
        source_path: "src/diagnostic.rs",
        test: "diagnostic_code_inventory_is_exact_ordered_and_unique",
        rules: &["S3Q-TITLE-01"],
    },
    TestEvidence {
        source_path: "src/semantics/tests.rs",
        test: "test_titles_are_checked_once_in_source_order",
        rules: &["S3Q-TITLE-01", "S3Q-DETERMINISM-01"],
    },
    TestEvidence {
        source_path: "src/semantics/tests.rs",
        test: "tests_are_checked_as_truth_valued_functions_after_the_modules_own",
        rules: &["S3Q-CHECK-01"],
    },
    TestEvidence {
        source_path: "src/semantics/tests.rs",
        test: "a_tests_moduli_and_types_are_resolved_with_the_modules",
        rules: &["S3Q-CHECK-01"],
    },
    TestEvidence {
        source_path: "src/semantics/tests.rs",
        test: "a_test_is_checked_only_in_the_module_it_is_run_from",
        rules: &["S3Q-CHECK-01", "S3Q-ROOT-01"],
    },
    TestEvidence {
        source_path: "src/semantics/tests.rs",
        test: "whole_values_of_every_type_compare_with_equal_and_not_equal",
        rules: &["S3Q-EQUAL-01"],
    },
    TestEvidence {
        source_path: "src/semantics/tests.rs",
        test: "condition_and_comparison_errors_are_reported_once_in_checking_order",
        rules: &["S3Q-EQUAL-01", "S3Q-COMPAT-01"],
    },
    TestEvidence {
        source_path: "src/semantics/tests.rs",
        test: "tuple_types_and_selections_are_checked_once_in_order",
        rules: &["S3Q-EQUAL-01", "S3Q-COMPAT-01"],
    },
    TestEvidence {
        source_path: "src/core.rs",
        test: "core_accessors_preserve_source_order_and_derive_value_types",
        rules: &["S3Q-CHECK-01", "S3Q-COMPAT-01"],
    },
    TestEvidence {
        source_path: "src/eval.rs",
        test: "whole_values_compare_every_part_at_the_cost_of_their_parts",
        rules: &["S3Q-EQUAL-01", "S3Q-COST-01", "S3Q-DETERMINISM-01"],
    },
    TestEvidence {
        source_path: "src/eval.rs",
        test: "tests_run_in_source_order_and_keep_what_a_failed_equality_compared",
        rules: &[
            "S3Q-CHECK-01",
            "S3Q-RUN-01",
            "S3Q-STOP-01",
            "S3Q-DETERMINISM-01",
        ],
    },
    TestEvidence {
        source_path: "src/eval.rs",
        test: "every_limit_that_stops_a_test_is_reported_at_its_title",
        rules: &["S3Q-STOP-01"],
    },
    TestEvidence {
        source_path: "src/eval.rs",
        test: "a_module_without_tests_passes_them_all",
        rules: &["S3Q-RUN-01", "S3Q-COMPAT-01"],
    },
    TestEvidence {
        source_path: "src/eval.rs",
        test: "test_storage_failures_report_no_outcome",
        rules: &["S3Q-STOP-01", "S3Q-RES-01"],
    },
    TestEvidence {
        source_path: "orangec/src/main.rs",
        test: "test_report_names_each_test_and_where_a_failed_equality_first_differs",
        rules: &["S3Q-RUN-01"],
    },
    TestEvidence {
        source_path: "orangec/src/main.rs",
        test: "first_differences_name_an_element_or_a_part_and_its_element",
        rules: &["S3Q-RUN-01"],
    },
    TestEvidence {
        source_path: "orangec/src/main.rs",
        test: "step_limit_diagnostics_name_the_option_only_below_the_most_admitted",
        rules: &["S3Q-STOP-01"],
    },
    TestEvidence {
        source_path: "orangec/src/main.rs",
        test: "evaluation_options_parse_with_exact_bounds_and_messages",
        rules: &["S3Q-OPTIONS-01", "S3Q-COMPAT-01"],
    },
    TestEvidence {
        source_path: "orangec/src/main.rs",
        test: "cli_command_inventory_matches_the_help_rows",
        rules: &["S3Q-OPTIONS-01"],
    },
    TestEvidence {
        source_path: "orangec/src/main.rs",
        test: "step_report_follows_committed_values_and_is_not_written_without_them",
        rules: &["S3Q-OPTIONS-01", "S3Q-RES-01"],
    },
];

fn orangec() -> Command {
    Command::new(env!("CARGO_BIN_EXE_orangec"))
}

fn fixture_directory() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/s3q")
}

fn run(arguments: &[&str], paths: &[&Path]) -> Output {
    orangec().args(arguments).args(paths).output().unwrap()
}

/// Runs `orangec` with `arguments` on `path` twice and returns the first
/// output after requiring the second to be byte-identical.
fn run_twice(arguments: &[&str], path: &Path, context: &str) -> Output {
    let first = run(arguments, &[path]);
    let second = run(arguments, &[path]);
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

/// The primary location of each diagnostic as `file:line:column`, with the
/// file's directory removed.
fn primary_locations(stderr: &str) -> Vec<String> {
    stderr
        .lines()
        .filter_map(|line| line.strip_prefix(" --> "))
        .map(|location| {
            Path::new(location)
                .file_name()
                .unwrap()
                .to_string_lossy()
                .into_owned()
        })
        .collect()
}

fn assert_exact(output: &Output, status: i32, stdout: &str, stderr: &str, context: &str) {
    assert_eq!(
        output.status.code(),
        Some(status),
        "{context} status:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        stdout,
        "{context} stdout"
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stderr),
        stderr,
        "{context} stderr"
    );
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

/// A usage error: status 2, no output, and `message` first on stderr.
fn assert_usage_error(output: &Output, message: &str, context: &str) {
    assert_eq!(output.status.code(), Some(2), "{context} status");
    assert_eq!(output.stdout, b"", "{context} stdout");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.starts_with(&format!("orangec: {message}\n\nUsage: ")),
        "{context} stderr:\n{stderr}"
    );
}

/// A fresh directory for one generated program.
fn scratch_directory(name: &str) -> PathBuf {
    let directory = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("orangec-s3q-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&directory);
    fs::create_dir(&directory).unwrap();
    directory
}

/// The rule index rows of the specification: rule ID and evidence label.
fn documented_rules() -> Vec<(&'static str, &'static str)> {
    TESTS_SPECIFICATION
        .lines()
        .filter_map(|line| {
            let row = line.strip_prefix("| `")?;
            let (rule, _) = row.split_once("` |")?;
            if !rule.starts_with("S3Q-") {
                return None;
            }
            let evidence = line.strip_suffix('|')?.rsplit('|').next()?.trim();
            Some((rule, evidence))
        })
        .collect()
}

fn required_layers(label: &str) -> u8 {
    match label {
        "CLI and unit" => CLI | UNIT,
        "CLI and parser unit" => CLI | UNIT | PARSER_UNIT,
        "Generated CLI and unit" => GENERATED_CLI | UNIT,
        "Unit" => UNIT,
        _ => panic!("unknown S3q evidence layer {label:?}"),
    }
}

fn unit_source(source_path: &str) -> &'static str {
    match source_path {
        "src/parser.rs" => PARSER_SOURCE,
        "src/semantics.rs" => SEMANTICS_SOURCE,
        "src/semantics/tests.rs" => SEMANTICS_TESTS_SOURCE,
        "src/core.rs" => CORE_SOURCE,
        "src/diagnostic.rs" => DIAGNOSTIC_SOURCE,
        "src/eval.rs" => EVAL_SOURCE,
        "orangec/src/main.rs" => ORANGEC_SOURCE,
        _ => panic!("unmapped S3q evidence source {source_path}"),
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
        S3Q_CONFORMANCE_SOURCE.matches(&declaration).count(),
        1,
        "generated evidence {test} must be declared exactly once at file root"
    );
}

#[test]
fn s3q_rule_index_is_exact_and_covered() {
    let documented = documented_rules();
    let documented_ids: Vec<_> = documented.iter().map(|(rule, _)| *rule).collect();
    assert_eq!(
        documented_ids, RULES,
        "docs/TESTS_2026.md rule index drifted from the S3q runner"
    );
    let known: BTreeSet<_> = RULES.iter().copied().collect();
    assert_eq!(known.len(), RULES.len(), "duplicate S3q rule ID");

    let mut observed = BTreeMap::<&str, u8>::new();
    let mut record = |rule: &'static str, layers: u8, context: &str| {
        assert!(known.contains(rule), "unknown rule {rule} on {context}");
        *observed.entry(rule).or_default() |= layers;
    };

    let mut fixtures = BTreeSet::new();
    for case in CASES {
        assert!(fixtures.insert(case.fixture), "duplicate {}", case.fixture);
        assert!(!case.rules.is_empty(), "{} has no rules", case.fixture);
        assert!(!case.runs.is_empty(), "{} has no runs", case.fixture);
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
fn s3q_cli_conformance_corpus_is_exact_and_repeatable() {
    let directory = fixture_directory();
    let mut observed: Vec<_> = fs::read_dir(&directory)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().into_string().unwrap())
        .collect();
    observed.sort();
    let mut expected: Vec<_> = CASES.iter().map(|case| case.fixture).collect();
    expected.sort_unstable();
    assert_eq!(observed, expected, "unexpected S3q fixture inventory");

    for case in CASES {
        let path = directory.join(case.fixture);
        let mut diagnostics = None;
        for Run {
            arguments,
            expectation,
        } in case.runs
        {
            let context = format!("{} {}", case.fixture, arguments.join(" "));
            let output = run_twice(arguments, &path, &context);
            match *expectation {
                Expectation::Exact {
                    status,
                    stdout,
                    stderr,
                } => assert_exact(&output, status, &stdout(), stderr, &context),
                Expectation::Failure {
                    codes,
                    locations,
                    messages,
                } => {
                    assert_failure(&output, codes, locations, messages, &context);
                    // Every command reports a program's diagnostics alike.
                    let first = diagnostics.get_or_insert_with(|| output.stderr.clone());
                    assert_eq!(
                        *first, output.stderr,
                        "{context} diagnostics differ from the first command's"
                    );
                }
            }
        }
    }
}

#[test]
fn s3q_titles_hold_up_to_128_printable_ascii_bytes() {
    let directory = scratch_directory("titles");
    let path = directory.join("titles.or");
    // Titles of 1 and 128 bytes, and one of spaces and punctuation, are
    // admitted and printed as written.
    let longest = "a".repeat(128);
    fs::write(
        &path,
        format!(
            "edition 2026;\nmodule titles {{\n  test \"x\" {{ true }}\n  test \"{longest}\" {{ \
             true }}\n  test \" #1: 'a' ~ z! \" {{ true }}\n}}\n"
        ),
    )
    .unwrap();
    let output = run_twice(&["test"], &path, "admitted titles");
    assert_exact(
        &output,
        0,
        &format!(
            "test \"x\" ... ok\ntest \"{longest}\" ... ok\ntest \" #1: 'a' ~ z! \" ... ok\n\
             3 tests: 3 passed, 0 failed\n"
        ),
        "",
        "admitted titles",
    );

    // A raw tab and a delete, which the repository keeps out of its
    // sources, and a title one byte too long.
    fs::write(
        &path,
        format!(
            "edition 2026;\nmodule titles {{\n  test \"tab\tinside\" {{ true }}\n  test \
             \"del\x7f\" {{ true }}\n  test \"{longest}b\" {{ true }}\n}}\n"
        ),
    )
    .unwrap();
    let output = run_twice(&["test"], &path, "rejected titles");
    assert_failure(
        &output,
        &["ORC0242", "ORC0242", "ORC0242"],
        &["titles.or:3:8", "titles.or:4:8", "titles.or:5:8"],
        &[
            "error[ORC0242]: a test's title holds U+0009, which is not printable ASCII\n",
            "error[ORC0242]: a test's title holds U+007F, which is not printable ASCII\n",
            " at byte 3 of the title\n",
            "error[ORC0242]: this test's title is 129 bytes long\n",
            " a title holds at most 128 bytes\n",
        ],
        "rejected titles",
    );
    fs::remove_dir_all(&directory).unwrap();
}

const HELPER_PROGRAM: &str = concat!(
    "edition 2026;\n",
    "module helper {\n",
    "  spec one() -> Int { 1 }\n",
    "  test \"one is two\" { one() == 2 }\n",
    "  test \"a number\" { 3 }\n",
    "}\n",
);

const ROOT_PROGRAM: &str = concat!(
    "edition 2026;\n",
    "module main {\n",
    "  use helper;\n",
    "  spec test() -> Int { helper::one() + 6 }\n",
    "  test \"seven, through the helper\" { test() == 7 }\n",
    "}\n",
);

#[test]
fn s3q_only_the_root_modules_tests_are_checked_and_run() {
    let directory = scratch_directory("root");
    let helper = directory.join("helper.or");
    let root = directory.join("main.or");
    fs::write(&helper, HELPER_PROGRAM).unwrap();
    fs::write(&root, ROOT_PROGRAM).unwrap();

    // The helper's tests are neither checked, though one is not a `Bool`,
    // nor run, though the other fails. A function may be named `test`.
    let check = run_twice(&["check"], &root, "main check");
    assert_exact(&check, 0, "", "", "main check");
    let eval = run_twice(&["eval"], &root, "main eval");
    assert_exact(&eval, 0, "main::test: Int = 7\n", "", "main eval");
    let tests = run_twice(&["test", "--stats"], &root, "main test");
    assert_exact(
        &tests,
        0,
        "test \"seven, through the helper\" ... ok\n1 test: 1 passed, 0 failed\n",
        "test \"seven, through the helper\": 9 steps\ntotal: 9 of 1048576 steps\n",
        "main test",
    );

    // As the root, the helper's tests are checked, by every command.
    let mut first = None;
    for command in ["check", "eval", "test"] {
        let output = run_twice(&[command], &helper, &format!("helper {command}"));
        assert_failure(
            &output,
            &["ORC0214"],
            &["helper.or:5:21"],
            &["an integer literal cannot have type `Bool`"],
            &format!("helper {command}"),
        );
        let first = first.get_or_insert_with(|| output.stderr.clone());
        assert_eq!(*first, output.stderr, "helper {command} diagnostics");
    }

    // With the test that is not a `Bool` removed, the helper's own failing
    // test runs.
    fs::write(
        &helper,
        HELPER_PROGRAM.replace("  test \"a number\" { 3 }\n", ""),
    )
    .unwrap();
    let tests = run_twice(&["test"], &helper, "helper test");
    assert_exact(
        &tests,
        1,
        concat!(
            "test \"one is two\" ... FAILED\n",
            "    left:  1\n",
            "    right: 2\n",
            "1 test: 0 passed, 1 failed\n",
        ),
        "",
        "helper test",
    );
    fs::remove_dir_all(&directory).unwrap();
}

const STOPPING_PROGRAM: &str = concat!(
    "edition 2026;\n",
    "module stop {\n",
    "  spec sum(n: Int) -> Int {\n",
    "    for i in 0..100 with s: Int = 0 { if i < n { s + i } else { s } }\n",
    "  }\n",
    "  test \"a short sum\" { sum(10) == 45 }\n",
    "  test \"a long sum\" { sum(100) == 4950 }\n",
    "}\n",
);

#[test]
fn s3q_a_test_that_stops_ends_the_run_with_no_report() {
    let directory = scratch_directory("stop");
    let path = directory.join("stop.or");
    fs::write(&path, STOPPING_PROGRAM).unwrap();
    let report =
        "test \"a short sum\" ... ok\ntest \"a long sum\" ... ok\n2 tests: 2 passed, 0 failed\n";
    let steps = |budget: u32| {
        format!(
            "test \"a short sum\": 736 steps\ntest \"a long sum\": 1006 steps\n\
             total: 1742 of {budget} steps\n"
        )
    };

    // The default budget, exactly the steps needed, and the most admitted.
    for (options, budget) in [
        (&["--stats"][..], 1_048_576),
        (&["--steps", "1742", "--stats"][..], 1_742),
        (&["--stats", "--steps", "1073741824"][..], 1_073_741_824),
    ] {
        let arguments = [&["test"][..], options].concat();
        let output = run_twice(&arguments, &path, &format!("{options:?}"));
        assert_exact(&output, 0, report, &steps(budget), &format!("{options:?}"));
    }

    // The tests share the budget: one step fewer stops the second test, and
    // fewer than the first needs stops the first. Either way no test's
    // outcome is written.
    for (budget, line, column) in [("1741", 7, 8), ("735", 6, 8)] {
        let output = run_twice(
            &["test", "--steps", budget, "--stats"],
            &path,
            &format!("--steps {budget}"),
        );
        assert_failure(
            &output,
            &["ORC0301"],
            &[format!("stop.or:{line}:{column}").as_str()],
            &[
                "error[ORC0301]: reference evaluation step limit exceeded\n",
                " evaluation stopped while evaluating this test\n",
                &format!("= note: at most {budget} evaluation steps are permitted\n"),
                "= note: no test outcome is reported\n",
                "= note: `orangec test --steps N` sets the budget, up to 1073741824 steps\n",
            ],
            &format!("--steps {budget}"),
        );
        assert!(
            !String::from_utf8_lossy(&output.stderr).contains("total:"),
            "a stopped run reports no steps"
        );
    }

    // Any other limit stops the run at the test's title too, with the place
    // the limit was reached as a secondary label.
    let grow = directory.join("grow.or");
    fs::write(&grow, GROWING_PROGRAM).unwrap();
    let output = run_twice(&["test", "--stats"], &grow, "grow");
    assert_failure(
        &output,
        &["ORC0301"],
        &["grow.or:4:8"],
        &[
            "error[ORC0301]: exact integer result exceeds the 16384-significant-bit limit\n",
            " evaluation stopped while evaluating this test\n",
            " result is too large for the reference evaluator\n",
            "= note: `Int` is unbounded; this is a resource limit, not a finite width\n",
            "= note: no test outcome is reported\n",
        ],
        "grow",
    );
    assert!(
        !String::from_utf8_lossy(&output.stderr).contains("--steps"),
        "a limit other than the budget names no option"
    );
    fs::remove_dir_all(&directory).unwrap();
}

const GROWING_PROGRAM: &str = concat!(
    "edition 2026;\n",
    "module grow {\n",
    "  spec tower() -> Int { for i in 0..15 with x: Int = 2 { x * x } }\n",
    "  test \"a tower of squares\" { tower() > 0 }\n",
    "}\n",
);

#[test]
fn s3q_test_takes_one_source_and_the_step_options_but_not_spec() {
    let directory = scratch_directory("options");
    let path = directory.join("stop.or");
    fs::write(&path, STOPPING_PROGRAM).unwrap();

    // With both streams in one file, the step report follows the report.
    let transcript = |name: &str| {
        let merged = directory.join(name);
        let file = File::create(&merged).unwrap();
        let status = orangec()
            .args(["test", "--stats", "--steps=2000"])
            .arg(&path)
            .stdout(file.try_clone().unwrap())
            .stderr(file)
            .status()
            .unwrap();
        assert_eq!(status.code(), Some(0), "{name}");
        fs::read_to_string(&merged).unwrap()
    };
    let first = transcript("first.txt");
    assert_eq!(first, transcript("second.txt"), "report changed");
    assert_eq!(
        first,
        concat!(
            "test \"a short sum\" ... ok\n",
            "test \"a long sum\" ... ok\n",
            "2 tests: 2 passed, 0 failed\n",
            "test \"a short sum\": 736 steps\n",
            "test \"a long sum\": 1006 steps\n",
            "total: 1742 of 2000 steps\n",
        )
    );

    // `--spec` selects functions to evaluate, so it applies only to eval;
    // `test` takes exactly one source; and its budget is checked as eval's.
    let other = directory.join("other.or");
    fs::write(&other, STOPPING_PROGRAM).unwrap();
    for (arguments, paths, message) in [
        (
            &["test", "--spec", "sum"][..],
            &[path.as_path()][..],
            "option `--spec` applies only to eval",
        ),
        (
            &["--spec=sum", "test"][..],
            &[path.as_path()][..],
            "option `--spec` applies only to eval",
        ),
        (
            &["test"][..],
            &[][..],
            "command `test` requires at least one source file",
        ),
        (
            &["test", "--stats"][..],
            &[path.as_path(), other.as_path()][..],
            "command `test` requires exactly one source file",
        ),
        (
            &["test", "--steps", "0"][..],
            &[path.as_path()][..],
            "option `--steps` takes a number of steps from 1 through 1073741824",
        ),
        (
            &["test", "--steps", "1", "--steps", "2"][..],
            &[path.as_path()][..],
            "option `--steps` may be specified at most once",
        ),
        (
            &["check", "--stats"][..],
            &[path.as_path()][..],
            "option `--stats` applies only to eval and test",
        ),
        (
            &["lex", "--steps", "5"][..],
            &[path.as_path()][..],
            "option `--steps` applies only to eval and test",
        ),
    ] {
        let first = run(arguments, paths);
        let second = run(arguments, paths);
        let context = format!("{arguments:?} on {} sources", paths.len());
        assert_repeatable(&first, &second, &context);
        assert_usage_error(&first, message, &context);
    }
    fs::remove_dir_all(&directory).unwrap();
}

#[test]
fn s3q_comparisons_cost_the_same_wherever_the_operands_differ() {
    let directory = scratch_directory("cost");
    let path = directory.join("place.or");
    // Two arrays of 65,536 bytes, equal or differing at one element. The
    // fill, the update, and the comparison cost 1,024 steps each, and the
    // comparison compares every element wherever the difference is.
    for at in [0, 1, 32_767, 65_535] {
        for (value, equal) in [(0, true), (1, false)] {
            fs::write(
                &path,
                format!(
                    "edition 2026;\nmodule place {{\n  spec compared() -> Bool {{\n    \
                     let x: Word[8]^65536 = [0; 65536];\n    x == (x with [{at}] = {value})\n  \
                     }}\n}}\n"
                ),
            )
            .unwrap();
            let context = format!("[{at}] = {value}");
            let output = run_twice(&["eval", "--stats"], &path, &context);
            assert_exact(
                &output,
                0,
                &format!("place::compared: Bool = {equal}\n"),
                "place::compared: 3077 steps\ntotal: 3077 of 1048576 steps\n",
                &context,
            );
        }
    }
    fs::remove_dir_all(&directory).unwrap();
}
