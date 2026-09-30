//! External conformance evidence for the proposed Orange 2026 S3b slice.
//!
//! The rule index lives in `docs/EXPRESSIONS_2026.md`. This runner requires
//! that index to agree exactly with the evidence map below, runs every fixture
//! and generated source through the real `orangec` binary twice, and checks
//! that every named unit test is declared once in its source's test module.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

const EXPRESSIONS_SPECIFICATION: &str = include_str!("../../../../docs/EXPRESSIONS_2026.md");
const S3B_CONFORMANCE_SOURCE: &str = include_str!("s3b_conformance.rs");
const LEXER_SOURCE: &str = include_str!("../../orange-compiler/src/lexer.rs");
const PARSER_SOURCE: &str = include_str!("../../orange-compiler/src/parser.rs");
const SEMANTICS_SOURCE: &str = include_str!("../../orange-compiler/src/semantics.rs");
const SEMANTICS_TESTS_SOURCE: &str = include_str!("../../orange-compiler/src/semantics/tests.rs");
const CORE_SOURCE: &str = include_str!("../../orange-compiler/src/core.rs");
const EVAL_SOURCE: &str = include_str!("../../orange-compiler/src/eval.rs");

const CLI: u8 = 1 << 0;
const GENERATED_CLI: u8 = 1 << 1;
const UNIT: u8 = 1 << 2;
const PARSER_UNIT: u8 = 1 << 3;

/// Every S3b rule, in the order of the specification's index.
const RULES: [&str; 28] = [
    "S3B-LEX-01",
    "S3B-GRAMMAR-01",
    "S3B-SIGN-01",
    "S3B-GROUP-01",
    "S3B-NAME-01",
    "S3B-CALL-01",
    "S3B-CYCLE-01",
    "S3B-TYPE-01",
    "S3B-CHECK-01",
    "S3B-LIT-01",
    "S3B-OP-01",
    "S3B-SHIFT-01",
    "S3B-WORD-01",
    "S3B-INT-01",
    "S3B-DIAG-01",
    "S3B-CORE-01",
    "S3B-EVAL-01",
    "S3B-EVAL-WORD-01",
    "S3B-RES-NEST-01",
    "S3B-RES-HEIGHT-01",
    "S3B-RES-LIST-01",
    "S3B-RES-EVENT-01",
    "S3B-RES-DEPTH-01",
    "S3B-RES-STEP-01",
    "S3B-RES-BITS-01",
    "S3B-RES-STACK-01",
    "S3B-RES-FAIL-01",
    "S3B-DETERMINISM-01",
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

const CASES: [Case; 14] = [
    Case {
        fixture: "valid-calls-and-grouping.or",
        expectation: Expectation::Success(concat!(
            "calls::before: Word[16] = 0x1400\n",
            "calls::after: Word[16] = 0x0000\n",
            "calls::diamond: Int = 35\n",
            "calls::left: Int = 8\n",
            "calls::right: Int = 27\n",
        )),
        rules: &[
            "S3B-GRAMMAR-01",
            "S3B-GROUP-01",
            "S3B-NAME-01",
            "S3B-CALL-01",
            "S3B-TYPE-01",
            "S3B-CHECK-01",
            "S3B-WORD-01",
            "S3B-INT-01",
            "S3B-CORE-01",
            "S3B-EVAL-01",
            "S3B-EVAL-WORD-01",
            "S3B-DETERMINISM-01",
        ],
    },
    Case {
        fixture: "valid-chacha20-quarter-round.or",
        expectation: Expectation::Success(concat!(
            "chacha20::a: Word[32] = 0xea2a92f4\n",
            "chacha20::b: Word[32] = 0xcb1cf8ce\n",
            "chacha20::c: Word[32] = 0x4581472e\n",
            "chacha20::d: Word[32] = 0x5881c4bb\n",
        )),
        rules: &[
            "S3B-GRAMMAR-01",
            "S3B-CALL-01",
            "S3B-OP-01",
            "S3B-SHIFT-01",
            "S3B-WORD-01",
            "S3B-EVAL-01",
            "S3B-EVAL-WORD-01",
            "S3B-DETERMINISM-01",
        ],
    },
    Case {
        fixture: "valid-int-arithmetic.or",
        expectation: Expectation::Success(concat!(
            "integers::two_to_the_128: Int = 340282366920938463463374607431768211456\n",
            "integers::signs: Int = -28\n",
            "integers::precedence: Int = 9\n",
            "integers::grouping: Int = -5\n",
            "integers::crossing_zero: Int = -18446744073709551616\n",
            "integers::negative_zero: Int = 0\n",
            "integers::p25519: Int = ",
            "57896044618658097711785492504343953926634992332820282019728792003956564819949\n",
        )),
        rules: &[
            "S3B-SIGN-01",
            "S3B-GROUP-01",
            "S3B-OP-01",
            "S3B-INT-01",
            "S3B-EVAL-01",
            "S3B-DETERMINISM-01",
        ],
    },
    Case {
        fixture: "valid-sha256-functions.or",
        expectation: Expectation::Success(concat!(
            "sha256::round0_t1: Word[32] = 0x54da50e8\n",
            "sha256::round0_a: Word[32] = 0x5d6aebcd\n",
            "sha256::round0_e: Word[32] = 0xfa2a4622\n",
            "sha256::sigma0_of_h0: Word[32] = 0xce20b47e\n",
            "sha256::schedule_sigma0: Word[32] = 0x940e90ef\n",
            "sha256::schedule_sigma1: Word[32] = 0x7da86405\n",
        )),
        rules: &[
            "S3B-GRAMMAR-01",
            "S3B-GROUP-01",
            "S3B-CALL-01",
            "S3B-OP-01",
            "S3B-SHIFT-01",
            "S3B-WORD-01",
            "S3B-EVAL-01",
            "S3B-EVAL-WORD-01",
            "S3B-DETERMINISM-01",
        ],
    },
    Case {
        fixture: "valid-word-arithmetic.or",
        expectation: Expectation::Success(concat!(
            "words::add_wraps: Word[8] = 0x2c\n",
            "words::subtract_wraps: Word[8] = 0xff\n",
            "words::multiply_wraps: Word[16] = 0x0001\n",
            "words::negation_is_subtraction: Word[32] = 0x80000000\n",
            "words::precedence: Word[32] = 0x00000003\n",
            "words::complement: Word[16] = 0xff00\n",
            "words::mask: Word[64] = 0xdeadbeef00000000\n",
            "words::merge: Word[64] = 0xdeadbeefcafef00d\n",
            "words::toggle: Word[8] = 0x5a\n",
            "words::shift_out: Word[8] = 0x02\n",
            "words::shift_in: Word[8] = 0x01\n",
            "words::rotate_left: Word[8] = 0x03\n",
            "words::rotate_right: Word[64] = 0x8000000000000000\n",
            "words::rotate_zero: Word[16] = 0xabcd\n",
            "words::largest_amount: Word[64] = 0xc000000000000000\n",
        )),
        rules: &[
            "S3B-GROUP-01",
            "S3B-TYPE-01",
            "S3B-LIT-01",
            "S3B-OP-01",
            "S3B-SHIFT-01",
            "S3B-WORD-01",
            "S3B-EVAL-01",
            "S3B-EVAL-WORD-01",
            "S3B-DETERMINISM-01",
        ],
    },
    Case {
        fixture: "invalid-call-cycles.or",
        expectation: Expectation::Failure {
            codes: &["ORC0217", "ORC0217", "ORC0217"],
            locations: &["5:26", "7:24", "10:25"],
            messages: &[
                "`itself` calls itself",
                "call cycle `ping` -> `pong` -> `ping`",
                "call cycle `first` -> `second` -> `third` -> `first`",
                "this call closes the cycle",
                "recursion is not part of Orange 2026",
            ],
        },
        rules: &["S3B-CYCLE-01", "S3B-DIAG-01", "S3B-DETERMINISM-01"],
    },
    Case {
        fixture: "invalid-diagnostic-order.or",
        expectation: Expectation::Failure {
            codes: &[
                "ORC0217", "ORC0214", "ORC0207", "ORC0204", "ORC0204", "ORC0215", "ORC0211",
            ],
            locations: &["7:26", "9:51", "9:56", "10:38", "12:37", "13:43", "13:56"],
            messages: &[
                "call cycle `loop_a` -> `loop_b` -> `loop_a`",
                "`byte` returns `Word[8]`, but `Int` is required here",
                "literal is outside the range of `Word[8]`",
                "`Word` width must be exactly 8, 16, 32, or 64",
                "`&` is not defined for `Int`",
                "`unknown` is not a parameter of `operator_stops`",
            ],
        },
        rules: &[
            "S3B-CYCLE-01",
            "S3B-TYPE-01",
            "S3B-CHECK-01",
            "S3B-DIAG-01",
            "S3B-DETERMINISM-01",
        ],
    },
    Case {
        fixture: "invalid-names-and-calls.or",
        expectation: Expectation::Failure {
            codes: &[
                "ORC0211", "ORC0211", "ORC0212", "ORC0212", "ORC0212", "ORC0213", "ORC0213",
                "ORC0218",
            ],
            locations: &[
                "8:38", "9:36", "10:32", "11:32", "12:29", "13:27", "14:28", "15:25",
            ],
            messages: &[
                "`y` is not a parameter of `unknown_name`",
                "to call the function `helper`, write `helper()` with its arguments",
                "no typed `spec` function named `missing` in this module",
                "`spec` function `legacy` has no typed body and cannot be called",
                "`impl` functions have no semantics yet and cannot be called",
                "`helper` takes 1 argument but 0 were supplied",
                "`helper` takes 1 argument but 2 were supplied",
                "duplicate parameter `x`",
            ],
        },
        rules: &[
            "S3B-NAME-01",
            "S3B-CALL-01",
            "S3B-DIAG-01",
            "S3B-DETERMINISM-01",
        ],
    },
    Case {
        fixture: "invalid-parameter-syntax.or",
        expectation: Expectation::Failure {
            codes: &["ORC0101", "ORC0101", "ORC0101"],
            locations: &["5:24", "6:23", "7:39"],
            messages: &[
                "expected `:` after the parameter name",
                "`impl` functions have an empty parameter list",
                "expected `->` after the parameter list",
            ],
        },
        rules: &["S3B-GRAMMAR-01", "S3B-DIAG-01", "S3B-DETERMINISM-01"],
    },
    Case {
        fixture: "invalid-shift-amounts.or",
        expectation: Expectation::Failure {
            codes: &["ORC0216", "ORC0216", "ORC0216", "ORC0216", "ORC0216"],
            locations: &["5:49", "6:59", "7:49", "8:63", "9:50"],
            messages: &[
                "`<<` on `Word[8]` needs an amount from 0 through 7",
                "`>>>` on `Word[32]` needs an amount from 0 through 31",
                "`>>` on `Word[16]` needs an amount from 0 through 15",
                "`<<<` on `Word[64]` needs an amount from 0 through 63",
                "amount must be an unsigned integer literal",
            ],
        },
        rules: &["S3B-SHIFT-01", "S3B-DIAG-01", "S3B-DETERMINISM-01"],
    },
    Case {
        fixture: "invalid-types-and-operators.or",
        expectation: Expectation::Failure {
            codes: &[
                "ORC0214", "ORC0214", "ORC0214", "ORC0215", "ORC0215", "ORC0215", "ORC0215",
            ],
            locations: &["7:40", "8:36", "9:54", "10:45", "11:40", "12:39", "13:38"],
            messages: &[
                "`x` has type `Word[16]`, but `Word[8]` is required here",
                "`integer` returns `Int`, but `Word[8]` is required here",
                "`x` has type `Word[32]`, but `Word[8]` is required here",
                "prefix `-` is not defined for `Word[8]`",
                "write `0 - x` for negation modulo 2^8",
                "prefix `~` is not defined for `Int`",
                "`&` is not defined for `Int`",
                "`<<<` is not defined for `Int`",
                "Orange has no implicit conversions between types",
            ],
        },
        rules: &[
            "S3B-SIGN-01",
            "S3B-CHECK-01",
            "S3B-OP-01",
            "S3B-DIAG-01",
            "S3B-DETERMINISM-01",
        ],
    },
    Case {
        fixture: "invalid-ungrouped-operators.or",
        expectation: Expectation::Failure {
            codes: &["ORC0108", "ORC0108", "ORC0108"],
            locations: &["5:65", "6:75", "7:54"],
            messages: &[
                "`&` follows `^` without grouping parentheses",
                "`^` follows `+` without grouping parentheses",
                "`<<` follows `<<` without grouping parentheses",
                "a shift or rotation takes exactly two operands",
            ],
        },
        rules: &["S3B-GROUP-01", "S3B-DIAG-01", "S3B-DETERMINISM-01"],
    },
    Case {
        fixture: "invalid-word-literals.or",
        expectation: Expectation::Failure {
            codes: &["ORC0207", "ORC0207", "ORC0207", "ORC0206"],
            locations: &["6:36", "7:46", "8:46", "9:50"],
            messages: &[
                "expected a value from 0 through 65535",
                "expected a value from 0 through 4294967295",
                "expected a value from 0 through 18446744073709551615",
                "`Word[32]` literals cannot be negative",
            ],
        },
        rules: &["S3B-LIT-01", "S3B-DIAG-01", "S3B-DETERMINISM-01"],
    },
    Case {
        fixture: "invalid-word-widths.or",
        expectation: Expectation::Failure {
            codes: &["ORC0204", "ORC0204", "ORC0204", "ORC0204"],
            locations: &["4:23", "5:23", "6:28", "7:29"],
            messages: &[
                "`Word` width must be exactly 8, 16, 32, or 64",
                "word widths do not coerce, truncate, or wrap",
            ],
        },
        rules: &["S3B-TYPE-01", "S3B-DIAG-01", "S3B-DETERMINISM-01"],
    },
];

/// Generated command-line cases in this file, each run twice.
const GENERATED_EVIDENCE: &[(&str, &[&str])] = &[
    (
        "s3b_shift_and_rotation_tokens_lex_by_longest_match",
        &["S3B-LEX-01", "S3B-DETERMINISM-01"],
    ),
    (
        "s3b_expression_nesting_limit_is_exact_for_every_opener",
        &["S3B-RES-NEST-01", "S3B-DETERMINISM-01"],
    ),
    (
        "s3b_expression_height_limit_is_exact",
        &["S3B-RES-HEIGHT-01", "S3B-DETERMINISM-01"],
    ),
    (
        "s3b_parameter_and_argument_limits_are_exact",
        &["S3B-RES-LIST-01", "S3B-DETERMINISM-01"],
    ),
    (
        "s3b_call_depth_limit_is_exact",
        &["S3B-RES-DEPTH-01", "S3B-DETERMINISM-01"],
    ),
    (
        "s3b_evaluation_step_limit_is_exact",
        &["S3B-RES-STEP-01", "S3B-DETERMINISM-01"],
    ),
    (
        "s3b_integer_result_bit_limit_is_exact",
        &["S3B-RES-BITS-01", "S3B-DETERMINISM-01"],
    ),
];

const UNIT_EVIDENCE: &[TestEvidence] = &[
    TestEvidence {
        source_path: "src/lexer.rs",
        test: "shift_and_rotation_tokens_use_longest_match",
        rules: &["S3B-LEX-01"],
    },
    TestEvidence {
        source_path: "src/parser.rs",
        test: "operator_inventories_spellings_and_tokens_are_exact",
        rules: &["S3B-LEX-01", "S3B-GROUP-01"],
    },
    TestEvidence {
        source_path: "src/parser.rs",
        test: "builds_parameters_calls_and_operators_with_exact_spans",
        rules: &["S3B-GRAMMAR-01"],
    },
    TestEvidence {
        source_path: "src/parser.rs",
        test: "rejects_malformed_parameters_calls_and_expressions",
        rules: &["S3B-GRAMMAR-01"],
    },
    TestEvidence {
        source_path: "src/parser.rs",
        test: "parameters_are_rejected_on_impl_and_untyped_spec_forms",
        rules: &["S3B-GRAMMAR-01"],
    },
    TestEvidence {
        source_path: "src/parser.rs",
        test: "groups_arithmetic_by_precedence_and_every_operator_leftward",
        rules: &["S3B-SIGN-01", "S3B-GROUP-01"],
    },
    TestEvidence {
        source_path: "src/parser.rs",
        test: "requires_parentheses_between_operator_groups_with_one_diagnostic",
        rules: &["S3B-GROUP-01"],
    },
    TestEvidence {
        source_path: "src/parser.rs",
        test: "ungrouped_operators_do_not_cascade_across_functions",
        rules: &["S3B-GROUP-01"],
    },
    TestEvidence {
        source_path: "src/parser.rs",
        test: "bounds_expression_nesting_for_every_opener",
        rules: &["S3B-RES-NEST-01"],
    },
    TestEvidence {
        source_path: "src/parser.rs",
        test: "bounds_expression_tree_height_for_operator_chains",
        rules: &["S3B-RES-HEIGHT-01"],
    },
    TestEvidence {
        source_path: "src/parser.rs",
        test: "bounds_parameters_per_function_and_arguments_per_call",
        rules: &["S3B-RES-LIST-01"],
    },
    TestEvidence {
        source_path: "src/parser.rs",
        test: "parameter_and_argument_reservation_failures_return_no_partial_ast",
        rules: &["S3B-RES-FAIL-01"],
    },
    TestEvidence {
        source_path: "src/parser.rs",
        test: "expression_parsing_is_repeatable_and_malformed_expressions_never_panic",
        rules: &["S3B-DETERMINISM-01"],
    },
    TestEvidence {
        source_path: "src/semantics/tests.rs",
        test: "typed_core_is_postorder_with_exact_types_spans_and_operations",
        rules: &["S3B-CHECK-01", "S3B-CORE-01"],
    },
    TestEvidence {
        source_path: "src/semantics/tests.rs",
        test: "calls_resolve_in_any_order_and_acyclic_graphs_are_accepted",
        rules: &["S3B-CALL-01", "S3B-CORE-01"],
    },
    TestEvidence {
        source_path: "src/semantics/tests.rs",
        test: "call_cycles_are_reported_once_at_the_closing_call",
        rules: &["S3B-CYCLE-01"],
    },
    TestEvidence {
        source_path: "src/semantics/tests.rs",
        test: "call_cycles_are_reported_through_calls_with_other_errors",
        rules: &["S3B-CYCLE-01"],
    },
    TestEvidence {
        source_path: "src/semantics/tests.rs",
        test: "long_call_cycles_have_bounded_messages",
        rules: &["S3B-CYCLE-01"],
    },
    TestEvidence {
        source_path: "src/semantics/tests.rs",
        test: "names_and_calls_resolve_only_to_parameters_and_typed_specs",
        rules: &["S3B-NAME-01", "S3B-CALL-01"],
    },
    TestEvidence {
        source_path: "src/semantics/tests.rs",
        test: "calls_check_arity_argument_types_and_result_types",
        rules: &["S3B-CALL-01", "S3B-CHECK-01", "S3B-DIAG-01"],
    },
    TestEvidence {
        source_path: "src/semantics/tests.rs",
        test: "operators_are_defined_only_for_their_types",
        rules: &["S3B-OP-01"],
    },
    TestEvidence {
        source_path: "src/semantics/tests.rs",
        test: "shift_and_rotation_amounts_are_literals_below_the_width",
        rules: &["S3B-SHIFT-01"],
    },
    TestEvidence {
        source_path: "src/semantics/tests.rs",
        test: "every_word_width_has_exact_literal_bounds",
        rules: &["S3B-TYPE-01", "S3B-LIT-01"],
    },
    TestEvidence {
        source_path: "src/semantics/tests.rs",
        test: "only_exact_decimal_word_widths_resolve",
        rules: &["S3B-TYPE-01"],
    },
    TestEvidence {
        source_path: "src/semantics/tests.rs",
        test: "unresolved_signatures_are_reported_once_without_cascades",
        rules: &["S3B-DIAG-01"],
    },
    TestEvidence {
        source_path: "src/semantics/tests.rs",
        test: "body_errors_precede_call_graph_errors_and_all_errors_are_ordered",
        rules: &["S3B-CYCLE-01", "S3B-DIAG-01", "S3B-DETERMINISM-01"],
    },
    TestEvidence {
        source_path: "src/semantics/tests.rs",
        test: "expression_events_and_core_nodes_follow_the_normative_accounting",
        rules: &["S3B-CORE-01", "S3B-RES-EVENT-01"],
    },
    TestEvidence {
        source_path: "src/semantics/tests.rs",
        test: "expression_storage_failures_return_no_partial_core",
        rules: &["S3B-RES-FAIL-01"],
    },
    TestEvidence {
        source_path: "src/core.rs",
        test: "exact_integer_arithmetic_matches_an_i128_reference",
        rules: &["S3B-INT-01"],
    },
    TestEvidence {
        source_path: "src/core.rs",
        test: "multi_limb_arithmetic_is_exact",
        rules: &["S3B-INT-01"],
    },
    TestEvidence {
        source_path: "src/core.rs",
        test: "exact_arithmetic_storage_failures_return_none",
        rules: &["S3B-RES-FAIL-01"],
    },
    TestEvidence {
        source_path: "src/eval.rs",
        test: "word_operators_match_a_wide_reference_at_every_width",
        rules: &["S3B-WORD-01", "S3B-EVAL-WORD-01"],
    },
    TestEvidence {
        source_path: "src/eval.rs",
        test: "shifts_and_rotations_match_the_reference_for_every_amount",
        rules: &["S3B-SHIFT-01", "S3B-WORD-01", "S3B-EVAL-WORD-01"],
    },
    TestEvidence {
        source_path: "src/eval.rs",
        test: "sha256_round_zero_matches_the_fips_example",
        rules: &["S3B-WORD-01"],
    },
    TestEvidence {
        source_path: "src/eval.rs",
        test: "chacha20_quarter_round_matches_rfc_8439",
        rules: &["S3B-WORD-01"],
    },
    TestEvidence {
        source_path: "src/eval.rs",
        test: "int_arithmetic_is_exact_through_calls",
        rules: &["S3B-INT-01", "S3B-EVAL-01"],
    },
    TestEvidence {
        source_path: "src/eval.rs",
        test: "int_results_are_bounded_by_the_significant_bit_limit",
        rules: &["S3B-RES-BITS-01"],
    },
    TestEvidence {
        source_path: "src/eval.rs",
        test: "only_parameterless_functions_are_evaluated_and_reported",
        rules: &["S3B-EVAL-01"],
    },
    TestEvidence {
        source_path: "src/eval.rs",
        test: "call_depth_counts_the_evaluated_function_as_the_first_frame",
        rules: &["S3B-RES-DEPTH-01"],
    },
    TestEvidence {
        source_path: "src/eval.rs",
        test: "steps_follow_the_normative_cost_table",
        rules: &["S3B-SIGN-01", "S3B-RES-STEP-01"],
    },
    TestEvidence {
        source_path: "src/eval.rs",
        test: "exponential_call_trees_stop_at_the_step_limit",
        rules: &["S3B-RES-STEP-01"],
    },
    TestEvidence {
        source_path: "src/eval.rs",
        test: "value_and_call_stack_reservation_failures_return_no_values",
        rules: &["S3B-RES-FAIL-01"],
    },
    TestEvidence {
        source_path: "src/eval.rs",
        test: "deepest_accepted_sources_fit_in_one_mebibyte_of_stack",
        rules: &["S3B-RES-STACK-01"],
    },
];

fn orangec() -> Command {
    Command::new(env!("CARGO_BIN_EXE_orangec"))
}

fn fixture_directory() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/s3b")
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
    EXPRESSIONS_SPECIFICATION
        .lines()
        .filter_map(|line| {
            let row = line.strip_prefix("| `")?;
            let (rule, _) = row.split_once("` |")?;
            if !rule.starts_with("S3B-") {
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
        _ => panic!("unknown S3b evidence layer {label:?}"),
    }
}

fn unit_source(source_path: &str) -> &'static str {
    match source_path {
        "src/lexer.rs" => LEXER_SOURCE,
        "src/parser.rs" => PARSER_SOURCE,
        "src/semantics.rs" => SEMANTICS_SOURCE,
        "src/semantics/tests.rs" => SEMANTICS_TESTS_SOURCE,
        "src/core.rs" => CORE_SOURCE,
        "src/eval.rs" => EVAL_SOURCE,
        _ => panic!("unmapped S3b evidence source {source_path}"),
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
        S3B_CONFORMANCE_SOURCE.matches(&declaration).count(),
        1,
        "generated evidence {test} must be declared exactly once at file root"
    );
}

#[test]
fn s3b_rule_index_is_exact_and_covered() {
    let documented = documented_rules();
    let documented_ids: Vec<_> = documented.iter().map(|(rule, _)| *rule).collect();
    assert_eq!(
        documented_ids, RULES,
        "docs/EXPRESSIONS_2026.md rule index drifted from the S3b runner"
    );
    let known: BTreeSet<_> = RULES.iter().copied().collect();
    assert_eq!(known.len(), RULES.len(), "duplicate S3b rule ID");

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
fn s3b_cli_conformance_corpus_is_exact_and_repeatable() {
    let directory = fixture_directory();
    let mut observed: Vec<_> = fs::read_dir(&directory)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().into_string().unwrap())
        .collect();
    observed.sort();
    let mut expected: Vec<_> = CASES.iter().map(|case| case.fixture).collect();
    expected.sort_unstable();
    assert_eq!(observed, expected, "unexpected S3b fixture inventory");

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
fn s3b_shift_and_rotation_tokens_lex_by_longest_match() {
    let output = run_twice("lex", "edition 2026;\nx<<<<y>>>>>z<<=w\n", "shift tokens");
    assert_success(
        &output,
        concat!(
            "0..7\tKW_EDITION\t\"edition\"\n",
            "8..12\tINTEGER\t\"2026\"\n",
            "12..13\tSEMICOLON\t\";\"\n",
            "14..15\tIDENTIFIER\t\"x\"\n",
            "15..18\tLESS_LESS_LESS\t\"<<<\"\n",
            "18..19\tLESS\t\"<\"\n",
            "19..20\tIDENTIFIER\t\"y\"\n",
            "20..23\tGREATER_GREATER_GREATER\t\">>>\"\n",
            "23..25\tGREATER_GREATER\t\">>\"\n",
            "25..26\tIDENTIFIER\t\"z\"\n",
            "26..28\tLESS_LESS\t\"<<\"\n",
            "28..29\tEQUAL\t\"=\"\n",
            "29..30\tIDENTIFIER\t\"w\"\n",
            "31..31\tEOF\t\"\"\n",
        ),
        "shift tokens",
    );
}

/// A name, a result type, and the expression nested `depth` levels deep.
type Opener = (&'static str, &'static str, fn(usize) -> String);

#[test]
fn s3b_expression_nesting_limit_is_exact_for_every_opener() {
    let openers: [Opener; 3] = [
        ("groups", "Int", |depth| {
            format!("{}1{}", "(".repeat(depth), ")".repeat(depth))
        }),
        ("calls", "Int", |depth| {
            format!("{}1{}", "id(".repeat(depth), ")".repeat(depth))
        }),
        ("complements", "Word[8]", |depth| {
            format!("{}1", "~".repeat(depth))
        }),
    ];
    for (name, ty, expression) in openers {
        let source = |depth| {
            module(&format!(
                "  spec id(x: Int) -> Int {{ x }}\n  spec deep() -> {ty} {{ {} }}\n",
                expression(depth)
            ))
        };
        let accepted = run_twice("eval", &source(64), &format!("{name} at 64"));
        let value = if ty == "Int" { "1" } else { "0x01" };
        assert_success(
            &accepted,
            &format!("limits::deep: {ty} = {value}\n"),
            &format!("{name} at 64"),
        );
        let rejected = run_twice("eval", &source(65), &format!("{name} at 65"));
        let stderr = String::from_utf8_lossy(&rejected.stderr);
        assert_eq!(rejected.status.code(), Some(1), "{name} at 65");
        assert_eq!(rejected.stdout, b"", "{name} at 65");
        assert_eq!(diagnostic_codes(&stderr), ["ORC0106"], "{name}:\n{stderr}");
        assert!(
            stderr.contains(
                "expression nesting exceeds the 64-level limit \
                 for groups, calls, arrays, indices, loops, conditionals, updates, moduli, \
                 and prefix operators"
            ),
            "{name}:\n{stderr}"
        );
    }
}

#[test]
fn s3b_expression_height_limit_is_exact() {
    // A left-associated chain of k additions has height k + 1.
    let source = |additions: usize| {
        module(&format!(
            "  spec tall() -> Int {{ 1{} }}\n",
            " + 1".repeat(additions)
        ))
    };
    let accepted = run_twice("eval", &source(255), "height 256");
    assert_success(&accepted, "limits::tall: Int = 256\n", "height 256");
    let rejected = run_twice("eval", &source(256), "height 257");
    assert_failure(
        &rejected,
        &["ORC0106"],
        // The 256th `+` would make the tree 257 levels tall.
        &["3:1046"],
        &["expression tree height exceeds the 256-level limit"],
        "height 257",
    );
}

#[test]
fn s3b_parameter_and_argument_limits_are_exact() {
    let parameters = |count: usize| {
        let list = (0..count)
            .map(|index| format!("p{index}: Int"))
            .collect::<Vec<_>>()
            .join(", ");
        module(&format!("  spec wide({list}) -> Int {{ p0 }}\n"))
    };
    let accepted = run_twice("check", &parameters(64), "64 parameters");
    assert_success(&accepted, "", "64 parameters");
    let rejected = run_twice("check", &parameters(65), "65 parameters");
    let stderr = String::from_utf8_lossy(&rejected.stderr);
    assert_eq!(diagnostic_codes(&stderr), ["ORC0106"], "{stderr}");
    assert!(
        stderr.contains("function declares more than 64 parameters"),
        "{stderr}"
    );

    // No function can take 256 parameters, so a call that the parser accepts
    // with 256 arguments reaches semantic analysis and fails on arity.
    let arguments = |count: usize| {
        let list = vec!["1"; count].join(", ");
        module(&format!(
            "  spec one(x: Int) -> Int {{ x }}\n  spec call() -> Int {{ one({list}) }}\n"
        ))
    };
    let parsed = run_twice("check", &arguments(256), "256 arguments");
    let stderr = String::from_utf8_lossy(&parsed.stderr);
    assert_eq!(diagnostic_codes(&stderr), ["ORC0213"], "{stderr}");
    assert!(
        stderr.contains("`one` takes 1 argument but 256 were supplied"),
        "{stderr}"
    );
    let rejected = run_twice("check", &arguments(257), "257 arguments");
    let stderr = String::from_utf8_lossy(&rejected.stderr);
    assert_eq!(diagnostic_codes(&stderr), ["ORC0106"], "{stderr}");
    assert!(
        stderr.contains("call supplies more than 256 arguments"),
        "{stderr}"
    );
}

#[test]
fn s3b_call_depth_limit_is_exact() {
    // `f0` is the first frame; `f0` through `f{last}` make last + 1 frames.
    let chain = |last: usize| {
        let mut members = String::new();
        for index in 0..last {
            members.push_str(&format!(
                "  spec f{index}() -> Int {{ f{}() }}\n",
                index + 1
            ));
        }
        members.push_str(&format!("  spec f{last}() -> Int {{ 7 }}\n"));
        module(&members)
    };
    let accepted = run_twice("eval", &chain(255), "256 frames");
    let expected: String = (0..=255)
        .map(|index| format!("limits::f{index}: Int = 7\n"))
        .collect();
    assert_success(&accepted, &expected, "256 frames");

    let rejected = run_twice("eval", &chain(256), "257 frames");
    assert_failure(
        &rejected,
        &["ORC0301"],
        &["258:24"],
        &[
            "reference evaluation call depth limit exceeded",
            "this call exceeds the depth limit",
            "at most 256 nested calls are permitted",
            "no partial value set is returned",
        ],
        "257 frames",
    );
}

#[test]
fn s3b_evaluation_step_limit_is_exact() {
    // `d0(x)` costs 5 steps at its call site (argument, call, `x`, `x`, `+`),
    // and each `dk(x)` costs 2 * cost(d(k-1)) + 3, so `d17(1)` costs
    // 8 * 2^17 - 3 = 1,048,573. `^ 0` adds two steps and `last` one or two.
    let source = |last: &str| {
        let mut members = String::from("  spec d0(x: Word[8]) -> Word[8] { x + x }\n");
        for level in 1..=17 {
            members.push_str(&format!(
                "  spec d{level}(x: Word[8]) -> Word[8] {{ d{0}(x) ^ d{0}(x) }}\n",
                level - 1
            ));
        }
        members.push_str("  spec root() -> Word[8] { d17(1) ^ 0 }\n");
        members.push_str(&format!("  spec last() -> Word[8] {{ {last} }}\n"));
        module(&members)
    };
    let accepted = run_twice("eval", &source("1"), "1,048,576 steps");
    assert_success(
        &accepted,
        "limits::root: Word[8] = 0x00\nlimits::last: Word[8] = 0x01\n",
        "1,048,576 steps",
    );
    let rejected = run_twice("eval", &source("1 + 0"), "1,048,577 steps");
    assert_failure(
        &rejected,
        &["ORC0301"],
        &["22:8"],
        &[
            "reference evaluation step limit exceeded",
            "evaluation stopped while evaluating this function",
            "at most 1048576 evaluation steps are permitted",
        ],
        "1,048,577 steps",
    );
}

#[test]
fn s3b_integer_result_bit_limit_is_exact() {
    let source = |body: &str| {
        module(&format!(
            concat!(
                "  spec square(x: Int) -> Int {{ x * x }}\n",
                "  spec power(x: Int) -> Int {{ square(square(square(square(x)))) }}\n",
                "  spec two_to_the_8192() -> Int {{ ",
                "power(power(power(square(2)))) }}\n",
                "  spec widest(x: Int) -> Int {{ (x - 1) * (x + 1) }}\n",
                "  spec edge() -> Int {{ {} }}\n",
            ),
            body
        ))
    };
    // (2^8192 - 1)(2^8192 + 1) = 2^16384 - 1 has exactly 16,384 bits.
    let accepted = run_twice("eval", &source("widest(two_to_the_8192())"), "16,384 bits");
    assert_eq!(accepted.status.code(), Some(0), "16,384 bits");
    assert_eq!(accepted.stderr, b"", "16,384 bits");
    let stdout = String::from_utf8(accepted.stdout).unwrap();
    let lines: Vec<_> = stdout.lines().collect();
    assert_eq!(lines.len(), 2, "{stdout}");
    let power = lines[0]
        .strip_prefix("limits::two_to_the_8192: Int = ")
        .unwrap();
    assert_eq!(power.len(), 2467);
    assert!(power.starts_with("10907481356194159294629842447337"));
    assert!(power.ends_with("50587661997186505665475715792896"));
    let edge = lines[1].strip_prefix("limits::edge: Int = ").unwrap();
    assert_eq!(edge.len(), 4933);
    assert!(edge.starts_with("11897314953572317650857593266280"));
    assert!(edge.ends_with("69943152460447027290669964066815"));

    // 2^16384 needs 16,385 bits.
    let rejected = run_twice("eval", &source("square(two_to_the_8192())"), "16,385 bits");
    assert_failure(
        &rejected,
        &["ORC0301"],
        &["3:32"],
        &[
            "exact integer result exceeds the 16384-significant-bit limit",
            "`Int` is unbounded; this is a resource limit, not a finite width",
            "no partial value set is returned",
        ],
        "16,385 bits",
    );
}
