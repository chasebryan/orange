//! External conformance evidence for the proposed Orange 2026 S3f slice.
//!
//! The rule index lives in `docs/CONDITIONS_2026.md`. This runner requires
//! that index to agree exactly with the evidence map below, runs every fixture
//! and generated source through the real `orangec` binary twice, and checks
//! that every named unit test is declared once in its source's test module.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

const CONDITIONS_SPECIFICATION: &str = include_str!("../../../../docs/CONDITIONS_2026.md");
const S3F_CONFORMANCE_SOURCE: &str = include_str!("s3f_conformance.rs");
const LEXER_SOURCE: &str = include_str!("../../orange-compiler/src/lexer.rs");
const PARSER_SOURCE: &str = include_str!("../../orange-compiler/src/parser.rs");
const SEMANTICS_SOURCE: &str = include_str!("../../orange-compiler/src/semantics.rs");
const CORE_SOURCE: &str = include_str!("../../orange-compiler/src/core.rs");
const EVAL_SOURCE: &str = include_str!("../../orange-compiler/src/eval.rs");
const DIAGNOSTIC_SOURCE: &str = include_str!("../../orange-compiler/src/diagnostic.rs");

const CLI: u8 = 1 << 0;
const GENERATED_CLI: u8 = 1 << 1;
const UNIT: u8 = 1 << 2;
const PARSER_UNIT: u8 = 1 << 3;

/// Every S3f rule, in the order of the specification's index.
const RULES: [&str; 18] = [
    "S3F-GRAMMAR-01",
    "S3F-WORDS-01",
    "S3F-BOOL-01",
    "S3F-COMPARE-01",
    "S3F-LOGIC-01",
    "S3F-DIVISION-01",
    "S3F-COND-01",
    "S3F-INDEX-01",
    "S3F-EVAL-01",
    "S3F-DIAG-01",
    "S3F-CORE-01",
    "S3F-RES-NEST-01",
    "S3F-RES-EVENT-01",
    "S3F-RES-STEP-01",
    "S3F-RES-STACK-01",
    "S3F-RES-FAIL-01",
    "S3F-COMPAT-01",
    "S3F-DETERMINISM-01",
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

const CASES: [Case; 8] = [
    Case {
        fixture: "valid-conditions.or",
        expectation: Expectation::Success(concat!(
            "conditions::truth: Bool^2 = [true, false]\n",
            "conditions::logic: Bool^4 = [false, true, false, true]\n",
            "conditions::order: Bool^4 = [true, true, true, true]\n",
            "conditions::euclid: Int^8 = [3, 1, -4, 1, -3, 1, 4, 1]\n",
            "conditions::by_zero: Int^2 = [0, 42]\n",
            "conditions::words: Word[8]^4 = [0x1c, 0x04, 0x00, 0xc8]\n",
            "conditions::signs: Int^3 = [-1, 0, 1]\n",
            "conditions::lazy: Int = 7\n",
            "conditions::rotated: Word[8]^5 = [0x02, 0x03, 0x04, 0x05, 0x01]\n",
            "conditions::evens: Int = 5\n",
            "conditions::names: Int = 6\n",
        )),
        rules: &[
            "S3F-GRAMMAR-01",
            "S3F-WORDS-01",
            "S3F-BOOL-01",
            "S3F-COMPARE-01",
            "S3F-LOGIC-01",
            "S3F-DIVISION-01",
            "S3F-COND-01",
            "S3F-INDEX-01",
            "S3F-EVAL-01",
            "S3F-RES-STEP-01",
            "S3F-COMPAT-01",
            "S3F-DETERMINISM-01",
        ],
    },
    Case {
        fixture: "valid-poly1305.or",
        expectation: Expectation::Success(concat!(
            "poly1305::prime: Int = 1361129467683753853853498429727072845819\n",
            "poly1305::example: Word[8]^16 = [",
            "0xa8, 0x06, 0x1d, 0xc1, 0x30, 0x51, 0x36, 0xc6, ",
            "0xc2, 0x2b, 0x8b, 0xaf, 0x0c, 0x01, 0x27, 0xa9]\n",
        )),
        rules: &[
            "S3F-GRAMMAR-01",
            "S3F-DIVISION-01",
            "S3F-COND-01",
            "S3F-EVAL-01",
            "S3F-CORE-01",
            "S3F-DETERMINISM-01",
        ],
    },
    Case {
        fixture: "valid-aead.or",
        expectation: Expectation::Success(concat!(
            "aead::prime: Int = 1361129467683753853853498429727072845819\n",
            "aead::sunscreen: Word[8]^114 = [",
            "0x4c, 0x61, 0x64, 0x69, 0x65, 0x73, 0x20, 0x61, ",
            "0x6e, 0x64, 0x20, 0x47, 0x65, 0x6e, 0x74, 0x6c, ",
            "0x65, 0x6d, 0x65, 0x6e, 0x20, 0x6f, 0x66, 0x20, ",
            "0x74, 0x68, 0x65, 0x20, 0x63, 0x6c, 0x61, 0x73, ",
            "0x73, 0x20, 0x6f, 0x66, 0x20, 0x27, 0x39, 0x39, ",
            "0x3a, 0x20, 0x49, 0x66, 0x20, 0x49, 0x20, 0x63, ",
            "0x6f, 0x75, 0x6c, 0x64, 0x20, 0x6f, 0x66, 0x66, ",
            "0x65, 0x72, 0x20, 0x79, 0x6f, 0x75, 0x20, 0x6f, ",
            "0x6e, 0x6c, 0x79, 0x20, 0x6f, 0x6e, 0x65, 0x20, ",
            "0x74, 0x69, 0x70, 0x20, 0x66, 0x6f, 0x72, 0x20, ",
            "0x74, 0x68, 0x65, 0x20, 0x66, 0x75, 0x74, 0x75, ",
            "0x72, 0x65, 0x2c, 0x20, 0x73, 0x75, 0x6e, 0x73, ",
            "0x63, 0x72, 0x65, 0x65, 0x6e, 0x20, 0x77, 0x6f, ",
            "0x75, 0x6c, 0x64, 0x20, 0x62, 0x65, 0x20, 0x69, ",
            "0x74, 0x2e]\n",
            "aead::sealed_sunscreen: Word[8]^130 = [",
            "0xd3, 0x1a, 0x8d, 0x34, 0x64, 0x8e, 0x60, 0xdb, ",
            "0x7b, 0x86, 0xaf, 0xbc, 0x53, 0xef, 0x7e, 0xc2, ",
            "0xa4, 0xad, 0xed, 0x51, 0x29, 0x6e, 0x08, 0xfe, ",
            "0xa9, 0xe2, 0xb5, 0xa7, 0x36, 0xee, 0x62, 0xd6, ",
            "0x3d, 0xbe, 0xa4, 0x5e, 0x8c, 0xa9, 0x67, 0x12, ",
            "0x82, 0xfa, 0xfb, 0x69, 0xda, 0x92, 0x72, 0x8b, ",
            "0x1a, 0x71, 0xde, 0x0a, 0x9e, 0x06, 0x0b, 0x29, ",
            "0x05, 0xd6, 0xa5, 0xb6, 0x7e, 0xcd, 0x3b, 0x36, ",
            "0x92, 0xdd, 0xbd, 0x7f, 0x2d, 0x77, 0x8b, 0x8c, ",
            "0x98, 0x03, 0xae, 0xe3, 0x28, 0x09, 0x1b, 0x58, ",
            "0xfa, 0xb3, 0x24, 0xe4, 0xfa, 0xd6, 0x75, 0x94, ",
            "0x55, 0x85, 0x80, 0x8b, 0x48, 0x31, 0xd7, 0xbc, ",
            "0x3f, 0xf4, 0xde, 0xf0, 0x8e, 0x4b, 0x7a, 0x9d, ",
            "0xe5, 0x76, 0xd2, 0x65, 0x86, 0xce, 0xc6, 0x4b, ",
            "0x61, 0x16, 0x1a, 0xe1, 0x0b, 0x59, 0x4f, 0x09, ",
            "0xe2, 0x6a, 0x7e, 0x90, 0x2e, 0xcb, 0xd0, 0x60, ",
            "0x06, 0x91]\n",
        )),
        rules: &[
            "S3F-GRAMMAR-01",
            "S3F-DIVISION-01",
            "S3F-COND-01",
            "S3F-EVAL-01",
            "S3F-CORE-01",
            "S3F-COMPAT-01",
            "S3F-DETERMINISM-01",
        ],
    },
    Case {
        fixture: "valid-x25519.or",
        expectation: Expectation::Success(concat!(
            "x25519::prime: Int = 57896044618658097711785492504343953926634992332820282019728792003956564819949\n",
            "x25519::test_vector: Word[8]^32 = [",
            "0xc3, 0xda, 0x55, 0x37, 0x9d, 0xe9, 0xc6, 0x90, ",
            "0x8e, 0x94, 0xea, 0x4d, 0xf2, 0x8d, 0x08, 0x4f, ",
            "0x32, 0xec, 0xcf, 0x03, 0x49, 0x1c, 0x71, 0xf7, ",
            "0x54, 0xb4, 0x07, 0x55, 0x77, 0xa2, 0x85, 0x52]\n",
        )),
        rules: &[
            "S3F-GRAMMAR-01",
            "S3F-BOOL-01",
            "S3F-COMPARE-01",
            "S3F-DIVISION-01",
            "S3F-COND-01",
            "S3F-INDEX-01",
            "S3F-EVAL-01",
            "S3F-CORE-01",
            "S3F-DETERMINISM-01",
        ],
    },
    Case {
        fixture: "invalid-condition-syntax.or",
        expectation: Expectation::Failure {
            codes: &[
                "ORC0101", "ORC0101", "ORC0101", "ORC0108", "ORC0108", "ORC0108", "ORC0108",
            ],
            locations: &["6:45", "7:52", "8:45", "9:56", "10:55", "11:58", "12:51"],
            messages: &[
                "expected `else` and the value when the condition is false",
                "every `if` has an `else`, so that a conditional always has a value",
                "expected `{` or `if` after `else`",
                "expected `}` after the value",
                "each branch of a conditional is one expression",
                "`<` follows `<` without grouping parentheses",
                "join two comparisons with `&&` or `||`",
                "`/` follows `/` without grouping parentheses",
                "`/` and `%` take exactly two operands; parenthesize one of them",
                "`||` follows `&&` without grouping parentheses",
                "`<` follows `+` without grouping parentheses",
            ],
        },
        rules: &["S3F-GRAMMAR-01", "S3F-DIAG-01", "S3F-DETERMINISM-01"],
    },
    Case {
        fixture: "invalid-conditions.or",
        expectation: Expectation::Failure {
            codes: &[
                "ORC0214", "ORC0214", "ORC0214", "ORC0215", "ORC0215", "ORC0215", "ORC0215",
                "ORC0215", "ORC0215", "ORC0215",
            ],
            locations: &[
                "6:38", "7:53", "8:28", "9:46", "10:44", "11:46", "12:55", "13:41", "14:35",
                "15:37",
            ],
            messages: &[
                "`x` has type `Int`, but `Bool` is required here",
                "`true` has type `Bool`, but `Int` is required here",
                "an integer literal cannot have type `Bool`",
                "the `Bool` values are written `true` and `false`",
                "`<` is not defined for `Bool`",
                "`+` is not defined for `Bool`",
                "`&` is not defined for `Bool`",
                "the operators on `Bool` are `!`, `&&`, `||`, `==`, and `!=`",
                "`&&` is not defined for `Word[8]`",
                "prefix `!` is not defined for `Word[8]`",
                "`as` does not convert to or from `Bool`",
            ],
        },
        rules: &[
            "S3F-BOOL-01",
            "S3F-LOGIC-01",
            "S3F-COND-01",
            "S3F-DIAG-01",
            "S3F-DETERMINISM-01",
        ],
    },
    Case {
        fixture: "invalid-comparisons.or",
        expectation: Expectation::Failure {
            codes: &["ORC0214", "ORC0227", "ORC0214", "ORC0215", "ORC0211"],
            locations: &["6:31", "7:28", "8:49", "9:55", "10:28"],
            messages: &[
                "a comparison gives `Bool`, but `Int` is required here",
                "the operands of `<` have no type of their own",
                "`y` has type `Int`, but `Word[8]` is required here",
                "`==` is not defined for `Word[8]^2`",
                "compare elements, such as `x[0] == y[0]`",
                "`z` is not a parameter of `unknown`",
            ],
        },
        rules: &["S3F-COMPARE-01", "S3F-DIAG-01", "S3F-DETERMINISM-01"],
    },
    Case {
        fixture: "invalid-division-indices.or",
        expectation: Expectation::Failure {
            codes: &["ORC0223", "ORC0223", "ORC0223", "ORC0223", "ORC0226"],
            locations: &["6:84", "7:86", "8:85", "9:86", "10:59"],
            messages: &[
                "this index runs from 0 through 4, out of range for `Word[8]^4`",
                "this index runs from 0 through 7, out of range for `Word[8]^4`",
                "this index runs from -3 through 0, out of range for `Word[8]^4`",
                "an `Int` index may use only integer literals, loop indices, and words converted",
                "using `+`, `-`, `*`, `/`, `%`, and conditionals",
            ],
        },
        rules: &["S3F-INDEX-01", "S3F-DIAG-01", "S3F-DETERMINISM-01"],
    },
];

/// Generated command-line cases in this file, each run twice.
const GENERATED_EVIDENCE: &[(&str, &[&str])] = &[(
    "s3f_long_chains_and_untaken_branches_are_exact",
    &["S3F-COND-01", "S3F-RES-STEP-01", "S3F-DETERMINISM-01"],
)];

const UNIT_EVIDENCE: &[TestEvidence] = &[
    TestEvidence {
        source_path: "src/parser.rs",
        test: "builds_conditionals_comparisons_and_divisions_with_exact_spans",
        rules: &["S3F-GRAMMAR-01", "S3F-RES-NEST-01"],
    },
    TestEvidence {
        source_path: "src/parser.rs",
        test: "conditional_words_are_recognized_only_by_position",
        rules: &["S3F-GRAMMAR-01", "S3F-WORDS-01", "S3F-COMPAT-01"],
    },
    TestEvidence {
        source_path: "src/parser.rs",
        test: "rejects_malformed_conditionals_with_exact_messages",
        rules: &["S3F-GRAMMAR-01", "S3F-DIAG-01", "S3F-COMPAT-01"],
    },
    TestEvidence {
        source_path: "src/parser.rs",
        test: "errors_inside_branches_and_steps_recover_to_the_next_function",
        rules: &["S3F-GRAMMAR-01", "S3F-DIAG-01", "S3F-COMPAT-01"],
    },
    TestEvidence {
        source_path: "src/parser.rs",
        test: "bounds_expression_nesting_for_every_opener",
        rules: &["S3F-RES-NEST-01", "S3F-COMPAT-01"],
    },
    TestEvidence {
        source_path: "src/parser.rs",
        test: "expression_parsing_is_repeatable_and_malformed_expressions_never_panic",
        rules: &["S3F-GRAMMAR-01", "S3F-DETERMINISM-01"],
    },
    TestEvidence {
        source_path: "src/semantics.rs",
        test: "conditions_comparisons_and_divisions_build_typed_core_in_postorder",
        rules: &[
            "S3F-BOOL-01",
            "S3F-COMPARE-01",
            "S3F-LOGIC-01",
            "S3F-DIVISION-01",
            "S3F-COND-01",
            "S3F-CORE-01",
        ],
    },
    TestEvidence {
        source_path: "src/semantics.rs",
        test: "conditionals_are_numbered_in_source_order_of_their_if_keywords",
        rules: &["S3F-COND-01", "S3F-CORE-01"],
    },
    TestEvidence {
        source_path: "src/semantics.rs",
        test: "bool_literals_resolve_only_where_no_name_of_their_spelling_is_in_scope",
        rules: &["S3F-WORDS-01", "S3F-BOOL-01", "S3F-COMPAT-01"],
    },
    TestEvidence {
        source_path: "src/semantics.rs",
        test: "condition_and_comparison_errors_are_reported_once_in_checking_order",
        rules: &[
            "S3F-BOOL-01",
            "S3F-COMPARE-01",
            "S3F-LOGIC-01",
            "S3F-COND-01",
            "S3F-DIAG-01",
            "S3F-DETERMINISM-01",
        ],
    },
    TestEvidence {
        source_path: "src/semantics.rs",
        test: "index_ranges_follow_euclidean_division_and_its_total_rules",
        rules: &["S3F-INDEX-01", "S3F-DIAG-01"],
    },
    TestEvidence {
        source_path: "src/semantics.rs",
        test: "condition_events_and_core_nodes_follow_the_normative_accounting",
        rules: &["S3F-RES-EVENT-01"],
    },
    TestEvidence {
        source_path: "src/semantics.rs",
        test: "branch_storage_failures_return_no_partial_core",
        rules: &["S3F-RES-FAIL-01"],
    },
    TestEvidence {
        source_path: "src/semantics.rs",
        test: "rejects_foreign_spans_in_conditionals",
        rules: &["S3F-RES-FAIL-01"],
    },
    TestEvidence {
        source_path: "src/core.rs",
        test: "exact_comparison_and_euclidean_division_match_an_i128_reference",
        rules: &["S3F-COMPARE-01", "S3F-DIVISION-01"],
    },
    TestEvidence {
        source_path: "src/core.rs",
        test: "multi_limb_euclidean_division_satisfies_its_defining_identity",
        rules: &["S3F-DIVISION-01"],
    },
    TestEvidence {
        source_path: "src/eval.rs",
        test: "int_division_and_comparisons_match_an_i128_reference",
        rules: &["S3F-COMPARE-01", "S3F-DIVISION-01", "S3F-EVAL-01"],
    },
    TestEvidence {
        source_path: "src/eval.rs",
        test: "word_division_and_comparisons_are_unsigned_at_every_width",
        rules: &["S3F-COMPARE-01", "S3F-DIVISION-01", "S3F-EVAL-01"],
    },
    TestEvidence {
        source_path: "src/eval.rs",
        test: "conditions_choose_exactly_one_branch_in_source_order",
        rules: &[
            "S3F-LOGIC-01",
            "S3F-COND-01",
            "S3F-EVAL-01",
            "S3F-DETERMINISM-01",
        ],
    },
    TestEvidence {
        source_path: "src/eval.rs",
        test: "only_the_chosen_branch_is_evaluated",
        rules: &["S3F-COND-01", "S3F-RES-STEP-01"],
    },
    TestEvidence {
        source_path: "src/eval.rs",
        test: "steps_follow_the_normative_cost_table",
        rules: &["S3F-RES-STEP-01"],
    },
    TestEvidence {
        source_path: "src/eval.rs",
        test: "branch_frames_do_not_count_toward_the_call_depth",
        rules: &["S3F-RES-STEP-01"],
    },
    TestEvidence {
        source_path: "src/eval.rs",
        test: "deepest_accepted_sources_fit_in_one_mebibyte_of_stack",
        rules: &["S3F-RES-STACK-01"],
    },
    TestEvidence {
        source_path: "src/eval.rs",
        test: "inconsistent_conditions_and_comparisons_fail_closed",
        rules: &["S3F-CORE-01", "S3F-RES-FAIL-01"],
    },
    TestEvidence {
        source_path: "src/eval.rs",
        test: "branch_and_division_reservation_failures_return_no_values",
        rules: &["S3F-RES-FAIL-01"],
    },
    TestEvidence {
        source_path: "src/diagnostic.rs",
        test: "diagnostic_code_inventory_is_exact_ordered_and_unique",
        rules: &["S3F-DIAG-01"],
    },
];

fn orangec() -> Command {
    Command::new(env!("CARGO_BIN_EXE_orangec"))
}

fn fixture_directory() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/s3f")
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
    CONDITIONS_SPECIFICATION
        .lines()
        .filter_map(|line| {
            let row = line.strip_prefix("| `")?;
            let (rule, _) = row.split_once("` |")?;
            if !rule.starts_with("S3F-") {
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
        _ => panic!("unknown S3f evidence layer {label:?}"),
    }
}

fn unit_source(source_path: &str) -> &'static str {
    match source_path {
        "src/lexer.rs" => LEXER_SOURCE,
        "src/parser.rs" => PARSER_SOURCE,
        "src/semantics.rs" => SEMANTICS_SOURCE,
        "src/core.rs" => CORE_SOURCE,
        "src/eval.rs" => EVAL_SOURCE,
        "src/diagnostic.rs" => DIAGNOSTIC_SOURCE,
        _ => panic!("unmapped S3f evidence source {source_path}"),
    }
}

/// Requires `test` to be declared exactly once, as a `#[test]` function
/// directly inside the source's single `#[cfg(test)] mod tests` module.
fn assert_unit_test_declared(source_path: &str, test: &str) {
    let source = unit_source(source_path);
    let marker = "\n#[cfg(test)]\nmod tests {\n";
    assert_eq!(
        source.matches(marker).count(),
        1,
        "{source_path} must have exactly one unconditional test module"
    );
    let (_, tests) = source.split_once(marker).unwrap();
    let declaration = format!("\n    #[test]\n    fn {test}() {{\n");
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
        S3F_CONFORMANCE_SOURCE.matches(&declaration).count(),
        1,
        "generated evidence {test} must be declared exactly once at file root"
    );
}

#[test]
fn s3f_rule_index_is_exact_and_covered() {
    let documented = documented_rules();
    let documented_ids: Vec<_> = documented.iter().map(|(rule, _)| *rule).collect();
    assert_eq!(
        documented_ids, RULES,
        "docs/CONDITIONS_2026.md rule index drifted from the S3f runner"
    );
    let known: BTreeSet<_> = RULES.iter().copied().collect();
    assert_eq!(known.len(), RULES.len(), "duplicate S3f rule ID");

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
fn s3f_cli_conformance_corpus_is_exact_and_repeatable() {
    let directory = fixture_directory();
    let mut observed: Vec<_> = fs::read_dir(&directory)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().into_string().unwrap())
        .collect();
    observed.sort();
    let mut expected: Vec<_> = CASES.iter().map(|case| case.fixture).collect();
    expected.sort_unstable();
    assert_eq!(observed, expected, "unexpected S3f fixture inventory");

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
fn s3f_long_chains_and_untaken_branches_are_exact() {
    // A chain of 4096 arms is one conditional per arm, each the `else` value
    // of the one before it; the last condition that holds chooses its value.
    let chain = (0..4096)
        .map(|arm| format!("if x == {arm} {{ {arm} }} else "))
        .collect::<String>();
    let chosen = module(&format!(
        "  spec pick(x: Int) -> Int {{ {chain}{{ 4096 }} }}\n  spec chosen() -> Int {{ pick(4095) }}\n"
    ));
    let accepted = run_twice("eval", &chosen, "4096 arms");
    assert_success(&accepted, "limits::chosen: Int = 4095\n", "4096 arms");

    // Only the chosen branch is evaluated: a branch whose loops the step
    // budget could never finish costs nothing unless it is chosen.
    let heavy = concat!(
        "  spec heavy(x: Int) -> Int {\n",
        "    for i in 0..65536 with s: Int = x { for j in 0..65536 with t: Int = s { t } }\n",
        "  }\n",
    );
    let skipped = module(&format!(
        "{heavy}  spec skipped() -> Int {{ if false {{ heavy(0) }} else {{ 1 }} }}\n"
    ));
    let untaken = run_twice("eval", &skipped, "untaken branch");
    assert_success(&untaken, "limits::skipped: Int = 1\n", "untaken branch");

    let taken = module(&format!(
        "{heavy}  spec taken() -> Int {{ if true {{ heavy(0) }} else {{ 1 }} }}\n"
    ));
    let stopped = run_twice("eval", &taken, "taken branch");
    assert_failure(
        &stopped,
        &["ORC0301"],
        &["6:8"],
        &["reference evaluation step limit exceeded"],
        "taken branch",
    );
}
