//! External conformance evidence for the accepted Orange 2026 S3a slice.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

const SEMANTICS_SPECIFICATION: &str = include_str!("../../../../docs/SEMANTICS_2026.md");
const S3A_CONFORMANCE_SOURCE: &str = include_str!("s3a_conformance.rs");
const CORE_SOURCE: &str = include_str!("../../orange-compiler/src/core.rs");
const EVAL_SOURCE: &str = include_str!("../../orange-compiler/src/eval.rs");
const PARSER_SOURCE: &str = include_str!("../../orange-compiler/src/parser.rs");
const SEMANTICS_SOURCE: &str = include_str!("../../orange-compiler/src/semantics.rs");
const SOURCE_SOURCE: &str = include_str!("../../orange-compiler/src/source.rs");
const ORANGEC_MAIN_SOURCE: &str = include_str!("../src/main.rs");
const CLI_TEST_SOURCE: &str = include_str!("cli.rs");

const CLI_EVIDENCE: u16 = 1 << 0;
const GENERATED_CLI_EVIDENCE: u16 = 1 << 1;
const UNIT_EVIDENCE: u16 = 1 << 2;
const PARSER_UNIT_EVIDENCE: u16 = 1 << 3;
const INJECTED_WRITER_EVIDENCE: u16 = 1 << 4;
const INJECTED_LIMIT_EVIDENCE: u16 = 1 << 5;
const IO_FAULT_EVIDENCE: u16 = 1 << 6;
const ALLOCATION_FAULT_EVIDENCE: u16 = 1 << 7;
const HOST_BOUNDARY_EVIDENCE: u16 = 1 << 8;

#[derive(Clone, Copy)]
struct RuleRequirement {
    id: &'static str,
    layers: u16,
}

const RULES: [RuleRequirement; 30] = [
    RuleRequirement {
        id: "S3A-PHASE-01",
        layers: CLI_EVIDENCE | UNIT_EVIDENCE,
    },
    RuleRequirement {
        id: "S3A-GRAMMAR-01",
        layers: CLI_EVIDENCE | PARSER_UNIT_EVIDENCE,
    },
    RuleRequirement {
        id: "S3A-DECL-01",
        layers: CLI_EVIDENCE | UNIT_EVIDENCE,
    },
    RuleRequirement {
        id: "S3A-TYPE-INT-01",
        layers: CLI_EVIDENCE | UNIT_EVIDENCE,
    },
    RuleRequirement {
        id: "S3A-TYPE-WORD8-01",
        layers: CLI_EVIDENCE | UNIT_EVIDENCE,
    },
    RuleRequirement {
        id: "S3A-TYPE-REJECT-01",
        layers: CLI_EVIDENCE | UNIT_EVIDENCE,
    },
    RuleRequirement {
        id: "S3A-LIT-DECODE-01",
        layers: CLI_EVIDENCE | UNIT_EVIDENCE,
    },
    RuleRequirement {
        id: "S3A-LIT-ZEROES-01",
        layers: GENERATED_CLI_EVIDENCE | UNIT_EVIDENCE,
    },
    RuleRequirement {
        id: "S3A-LIT-BITS-01",
        layers: GENERATED_CLI_EVIDENCE | UNIT_EVIDENCE,
    },
    RuleRequirement {
        id: "S3A-INT-01",
        layers: CLI_EVIDENCE | UNIT_EVIDENCE,
    },
    RuleRequirement {
        id: "S3A-WORD-SIGN-01",
        layers: CLI_EVIDENCE | UNIT_EVIDENCE,
    },
    RuleRequirement {
        id: "S3A-WORD-RANGE-01",
        layers: CLI_EVIDENCE | UNIT_EVIDENCE,
    },
    RuleRequirement {
        id: "S3A-DIAG-01",
        layers: CLI_EVIDENCE | UNIT_EVIDENCE,
    },
    RuleRequirement {
        id: "S3A-ATOMIC-01",
        layers: CLI_EVIDENCE | UNIT_EVIDENCE,
    },
    RuleRequirement {
        id: "S3A-CORE-MEMBERSHIP-01",
        layers: CLI_EVIDENCE | UNIT_EVIDENCE,
    },
    RuleRequirement {
        id: "S3A-CORE-ORDER-01",
        layers: CLI_EVIDENCE | UNIT_EVIDENCE,
    },
    RuleRequirement {
        id: "S3A-CORE-CONTENT-01",
        layers: CLI_EVIDENCE | UNIT_EVIDENCE,
    },
    RuleRequirement {
        id: "S3A-CLI-EVAL-01",
        layers: CLI_EVIDENCE,
    },
    RuleRequirement {
        id: "S3A-EVAL-LINE-01",
        layers: CLI_EVIDENCE | UNIT_EVIDENCE,
    },
    RuleRequirement {
        id: "S3A-EVAL-INT-01",
        layers: CLI_EVIDENCE | UNIT_EVIDENCE,
    },
    RuleRequirement {
        id: "S3A-EVAL-WORD8-01",
        layers: CLI_EVIDENCE | UNIT_EVIDENCE,
    },
    RuleRequirement {
        id: "S3A-EVAL-EMPTY-01",
        layers: CLI_EVIDENCE | UNIT_EVIDENCE,
    },
    RuleRequirement {
        id: "S3A-EVAL-OUTPUT-FAIL-01",
        layers: UNIT_EVIDENCE | INJECTED_WRITER_EVIDENCE,
    },
    RuleRequirement {
        id: "S3A-RES-DIAG-01",
        layers: GENERATED_CLI_EVIDENCE | UNIT_EVIDENCE,
    },
    RuleRequirement {
        id: "S3A-RES-CORE-01",
        layers: UNIT_EVIDENCE | INJECTED_LIMIT_EVIDENCE,
    },
    RuleRequirement {
        id: "S3A-RES-EVENT-01",
        layers: UNIT_EVIDENCE | INJECTED_LIMIT_EVIDENCE,
    },
    RuleRequirement {
        id: "S3A-RES-EVAL-01",
        layers: UNIT_EVIDENCE | INJECTED_LIMIT_EVIDENCE,
    },
    RuleRequirement {
        id: "S3A-RES-FAIL-01",
        layers: UNIT_EVIDENCE | INJECTED_LIMIT_EVIDENCE,
    },
    RuleRequirement {
        id: "S3A-HOST-FAIL-01",
        layers: UNIT_EVIDENCE
            | IO_FAULT_EVIDENCE
            | ALLOCATION_FAULT_EVIDENCE
            | HOST_BOUNDARY_EVIDENCE,
    },
    RuleRequirement {
        id: "S3A-DETERMINISM-01",
        layers: CLI_EVIDENCE | UNIT_EVIDENCE,
    },
];

#[derive(Clone, Copy)]
enum Expectation {
    Success(&'static str),
    Failure {
        diagnostic_codes: &'static [&'static str],
        messages: &'static [&'static str],
        primary_locations: &'static [&'static str],
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

const CASES: [Case; 10] = [
    Case {
        fixture: "valid-empty-mixed.or",
        expectation: Expectation::Success(""),
        rules: &[
            "S3A-PHASE-01",
            "S3A-GRAMMAR-01",
            "S3A-DECL-01",
            "S3A-CORE-MEMBERSHIP-01",
            "S3A-EVAL-EMPTY-01",
            "S3A-DETERMINISM-01",
        ],
    },
    Case {
        fixture: "valid-int-radices.or",
        expectation: Expectation::Success(concat!(
            "s3a_int_radices::decimal_positive: Int = 1234567890\n",
            "s3a_int_radices::decimal_zero: Int = 0\n",
            "s3a_int_radices::decimal_negative: Int = -10\n",
            "s3a_int_radices::decimal_negative_zero: Int = 0\n",
            "s3a_int_radices::binary_positive: Int = 165\n",
            "s3a_int_radices::binary_zero: Int = 0\n",
            "s3a_int_radices::binary_negative: Int = -165\n",
            "s3a_int_radices::binary_negative_zero: Int = 0\n",
            "s3a_int_radices::hexadecimal_positive: Int = 3735928559\n",
            "s3a_int_radices::hexadecimal_zero: Int = 0\n",
            "s3a_int_radices::hexadecimal_negative: Int = -42\n",
            "s3a_int_radices::hexadecimal_negative_zero: Int = 0\n",
        )),
        rules: &[
            "S3A-TYPE-INT-01",
            "S3A-LIT-DECODE-01",
            "S3A-INT-01",
            "S3A-CORE-CONTENT-01",
            "S3A-EVAL-LINE-01",
            "S3A-EVAL-INT-01",
            "S3A-DETERMINISM-01",
        ],
    },
    Case {
        fixture: "valid-word8-boundaries.or",
        expectation: Expectation::Success(concat!(
            "s3a_word8_boundaries::zero: Word[8] = 0x00\n",
            "s3a_word8_boundaries::one: Word[8] = 0x01\n",
            "s3a_word8_boundaries::below_high: Word[8] = 0xfe\n",
            "s3a_word8_boundaries::high: Word[8] = 0xff\n",
        )),
        rules: &[
            "S3A-TYPE-WORD8-01",
            "S3A-WORD-RANGE-01",
            "S3A-CORE-MEMBERSHIP-01",
            "S3A-CORE-ORDER-01",
            "S3A-CORE-CONTENT-01",
            "S3A-EVAL-LINE-01",
            "S3A-EVAL-WORD8-01",
            "S3A-DETERMINISM-01",
        ],
    },
    Case {
        fixture: "invalid-typed-impl.or",
        expectation: Expectation::Failure {
            diagnostic_codes: &["ORC0101"],
            messages: &[
                "typed literal bodies are allowed only on `spec` functions",
                "until implementation semantics are defined",
            ],
            primary_locations: &["4:16"],
        },
        rules: &[
            "S3A-PHASE-01",
            "S3A-GRAMMAR-01",
            "S3A-DIAG-01",
            "S3A-ATOMIC-01",
            "S3A-DETERMINISM-01",
        ],
    },
    Case {
        fixture: "invalid-duplicate-spec.or",
        expectation: Expectation::Failure {
            diagnostic_codes: &["ORC0201", "ORC0201"],
            messages: &[
                "duplicate spec function `same_spec`",
                "duplicate impl function `same_impl`",
                "separate declaration namespaces",
            ],
            primary_locations: &["5:8", "7:8"],
        },
        rules: &[
            "S3A-DECL-01",
            "S3A-DIAG-01",
            "S3A-ATOMIC-01",
            "S3A-DETERMINISM-01",
        ],
    },
    Case {
        fixture: "invalid-unsupported-type.or",
        expectation: Expectation::Failure {
            diagnostic_codes: &["ORC0203", "ORC0203"],
            messages: &[
                "unsupported result type `Integer`",
                "unsupported result type `Int`",
            ],
            primary_locations: &["4:30", "5:23"],
        },
        rules: &[
            "S3A-TYPE-REJECT-01",
            "S3A-DIAG-01",
            "S3A-ATOMIC-01",
            "S3A-DETERMINISM-01",
        ],
    },
    Case {
        fixture: "invalid-word-width.or",
        expectation: Expectation::Failure {
            diagnostic_codes: &["ORC0204", "ORC0204", "ORC0204", "ORC0204", "ORC0204"],
            messages: &[
                "`Word` requires the exact width `[8]`",
                "only the exact type `Word[8]` is supported",
                "word widths do not coerce, truncate, or wrap",
            ],
            primary_locations: &["4:27", "5:31", "6:36", "7:34", "8:30"],
        },
        rules: &[
            "S3A-TYPE-WORD8-01",
            "S3A-TYPE-REJECT-01",
            "S3A-DIAG-01",
            "S3A-ATOMIC-01",
            "S3A-DETERMINISM-01",
        ],
    },
    Case {
        fixture: "invalid-int-magnitude.or",
        expectation: Expectation::Failure {
            diagnostic_codes: &["ORC0205"],
            messages: &["integer magnitude exceeds the 16384-significant-bit limit"],
            primary_locations: &["4:25"],
        },
        rules: &[
            "S3A-LIT-BITS-01",
            "S3A-DIAG-01",
            "S3A-ATOMIC-01",
            "S3A-DETERMINISM-01",
        ],
    },
    Case {
        fixture: "invalid-negative-word.or",
        expectation: Expectation::Failure {
            diagnostic_codes: &["ORC0206", "ORC0206"],
            messages: &[
                "`Word[8]` literals cannot be negative",
                "fixed-width words do not wrap or coerce negative integers",
            ],
            primary_locations: &["4:36", "5:37"],
        },
        rules: &[
            "S3A-WORD-SIGN-01",
            "S3A-DIAG-01",
            "S3A-ATOMIC-01",
            "S3A-DETERMINISM-01",
        ],
    },
    Case {
        fixture: "invalid-word-range.or",
        expectation: Expectation::Failure {
            diagnostic_codes: &["ORC0207", "ORC0207", "ORC0207"],
            messages: &[
                "literal is outside the range of `Word[8]`",
                "expected a value from 0 through 255",
                "fixed-width words do not truncate or wrap",
            ],
            primary_locations: &["5:31", "6:35", "7:30"],
        },
        rules: &[
            "S3A-WORD-RANGE-01",
            "S3A-DIAG-01",
            "S3A-ATOMIC-01",
            "S3A-DETERMINISM-01",
        ],
    },
];

const GENERATED_EVIDENCE: &[TestEvidence] = &[
    TestEvidence {
        source_path: "compiler/crates/orangec/tests/s3a_conformance.rs",
        test: "s3a_cli_conformance_corpus_is_exact_and_repeatable",
        rules: &["S3A-PHASE-01", "S3A-DETERMINISM-01"],
    },
    TestEvidence {
        source_path: "compiler/crates/orangec/tests/s3a_conformance.rs",
        test: "s3a_mixed_semantic_diagnostics_are_source_ordered_and_repeatable",
        rules: &["S3A-DIAG-01", "S3A-ATOMIC-01", "S3A-DETERMINISM-01"],
    },
    TestEvidence {
        source_path: "compiler/crates/orangec/tests/s3a_conformance.rs",
        test: "s3a_significant_bit_boundary_is_exact_and_repeatable",
        rules: &[
            "S3A-LIT-BITS-01",
            "S3A-DIAG-01",
            "S3A-ATOMIC-01",
            "S3A-DETERMINISM-01",
        ],
    },
    TestEvidence {
        source_path: "compiler/crates/orangec/tests/s3a_conformance.rs",
        test: "s3a_semantic_diagnostic_budget_is_exact_and_repeatable",
        rules: &[
            "S3A-DIAG-01",
            "S3A-ATOMIC-01",
            "S3A-RES-DIAG-01",
            "S3A-DETERMINISM-01",
        ],
    },
    TestEvidence {
        source_path: "compiler/crates/orangec/tests/s3a_conformance.rs",
        test: "s3a_leading_zeroes_are_budget_neutral_and_repeatable",
        rules: &["S3A-LIT-ZEROES-01", "S3A-EVAL-INT-01", "S3A-DETERMINISM-01"],
    },
    TestEvidence {
        source_path: "compiler/crates/orangec/tests/s3a_conformance.rs",
        test: "s3a_case_sensitive_spelling_is_exact_and_repeatable",
        rules: &[
            "S3A-DECL-01",
            "S3A-TYPE-REJECT-01",
            "S3A-DIAG-01",
            "S3A-ATOMIC-01",
            "S3A-CORE-CONTENT-01",
            "S3A-EVAL-LINE-01",
            "S3A-DETERMINISM-01",
        ],
    },
    TestEvidence {
        source_path: "compiler/crates/orangec/tests/s3a_conformance.rs",
        test: "s3a_reports_every_later_same_kind_duplicate_repeatably",
        rules: &[
            "S3A-DECL-01",
            "S3A-DIAG-01",
            "S3A-ATOMIC-01",
            "S3A-DETERMINISM-01",
        ],
    },
];

const INTERNAL_EVIDENCE: &[TestEvidence] = &[
    TestEvidence {
        source_path: "compiler/crates/orange-compiler/src/parser.rs",
        test: "builds_typed_literal_spec_nodes_with_exact_spans",
        rules: &["S3A-PHASE-01", "S3A-GRAMMAR-01"],
    },
    TestEvidence {
        source_path: "compiler/crates/orange-compiler/src/parser.rs",
        test: "diagnoses_every_malformed_production_without_an_ast",
        rules: &["S3A-PHASE-01", "S3A-GRAMMAR-01", "S3A-ATOMIC-01"],
    },
    TestEvidence {
        source_path: "compiler/crates/orange-compiler/src/semantics.rs",
        test: "production_limits_match_the_s3a_specification",
        rules: &[
            "S3A-LIT-BITS-01",
            "S3A-RES-DIAG-01",
            "S3A-RES-CORE-01",
            "S3A-RES-EVENT-01",
        ],
    },
    TestEvidence {
        source_path: "compiler/crates/orange-compiler/src/semantics.rs",
        test: "namespace_uniqueness_is_per_kind_and_cites_the_first_declaration",
        rules: &["S3A-DECL-01", "S3A-DIAG-01"],
    },
    TestEvidence {
        source_path: "compiler/crates/orange-compiler/src/semantics.rs",
        test: "independent_semantic_errors_preserve_source_order_and_responsible_spans",
        rules: &["S3A-DIAG-01", "S3A-ATOMIC-01", "S3A-DETERMINISM-01"],
    },
    TestEvidence {
        source_path: "compiler/crates/orange-compiler/src/semantics.rs",
        test: "compounded_declaration_failures_follow_semantic_traversal_order",
        rules: &["S3A-DIAG-01", "S3A-ATOMIC-01", "S3A-DETERMINISM-01"],
    },
    TestEvidence {
        source_path: "compiler/crates/orange-compiler/src/semantics.rs",
        test: "exact_ints_accept_every_sign_class_in_every_radix",
        rules: &[
            "S3A-TYPE-INT-01",
            "S3A-LIT-DECODE-01",
            "S3A-INT-01",
            "S3A-CORE-CONTENT-01",
        ],
    },
    TestEvidence {
        source_path: "compiler/crates/orange-compiler/src/semantics.rs",
        test: "integer_decoding_matches_a_deterministic_u128_reference_corpus",
        rules: &[
            "S3A-LIT-DECODE-01",
            "S3A-INT-01",
            "S3A-CORE-CONTENT-01",
            "S3A-DETERMINISM-01",
        ],
    },
    TestEvidence {
        source_path: "compiler/crates/orange-compiler/src/semantics.rs",
        test: "large_integer_rendering_matches_decimal_doubling_reference",
        rules: &[
            "S3A-LIT-DECODE-01",
            "S3A-LIT-BITS-01",
            "S3A-INT-01",
            "S3A-EVAL-INT-01",
            "S3A-DETERMINISM-01",
        ],
    },
    TestEvidence {
        source_path: "compiler/crates/orange-compiler/src/semantics.rs",
        test: "word_boundaries_are_exact_and_stably_formatted",
        rules: &[
            "S3A-TYPE-WORD8-01",
            "S3A-WORD-RANGE-01",
            "S3A-EVAL-WORD8-01",
        ],
    },
    TestEvidence {
        source_path: "compiler/crates/orange-compiler/src/semantics.rs",
        test: "every_word8_value_decodes_exactly_in_every_radix",
        rules: &[
            "S3A-TYPE-WORD8-01",
            "S3A-LIT-DECODE-01",
            "S3A-WORD-RANGE-01",
            "S3A-CORE-CONTENT-01",
            "S3A-DETERMINISM-01",
        ],
    },
    TestEvidence {
        source_path: "compiler/crates/orange-compiler/src/semantics.rs",
        test: "typed_impls_and_unadmitted_types_fail_closed",
        rules: &["S3A-GRAMMAR-01", "S3A-TYPE-REJECT-01", "S3A-ATOMIC-01"],
    },
    TestEvidence {
        source_path: "compiler/crates/orange-compiler/src/semantics.rs",
        test: "integer_at_significant_bit_limit_is_exact",
        rules: &["S3A-LIT-BITS-01"],
    },
    TestEvidence {
        source_path: "compiler/crates/orange-compiler/src/semantics.rs",
        test: "integer_over_significant_bit_limit_is_rejected_without_core",
        rules: &["S3A-LIT-BITS-01", "S3A-ATOMIC-01"],
    },
    TestEvidence {
        source_path: "compiler/crates/orange-compiler/src/semantics.rs",
        test: "leading_zeroes_consume_no_significant_bit_or_event_budget",
        rules: &["S3A-LIT-ZEROES-01", "S3A-INT-01", "S3A-RES-EVENT-01"],
    },
    TestEvidence {
        source_path: "compiler/crates/orange-compiler/src/semantics.rs",
        test: "words_reject_negative_and_out_of_range_values_without_coercion",
        rules: &["S3A-WORD-SIGN-01", "S3A-WORD-RANGE-01", "S3A-ATOMIC-01"],
    },
    TestEvidence {
        source_path: "compiler/crates/orange-compiler/src/semantics.rs",
        test: "core_ids_follow_only_typed_specs_in_source_order",
        rules: &[
            "S3A-CORE-MEMBERSHIP-01",
            "S3A-CORE-ORDER-01",
            "S3A-CORE-CONTENT-01",
        ],
    },
    TestEvidence {
        source_path: "compiler/crates/orange-compiler/src/semantics.rs",
        test: "diagnostic_and_resource_limits_fail_closed",
        rules: &[
            "S3A-DIAG-01",
            "S3A-ATOMIC-01",
            "S3A-RES-DIAG-01",
            "S3A-RES-CORE-01",
            "S3A-RES-EVENT-01",
            "S3A-RES-FAIL-01",
        ],
    },
    TestEvidence {
        source_path: "compiler/crates/orange-compiler/src/semantics.rs",
        test: "injected_limits_match_normative_event_and_core_node_accounting",
        rules: &["S3A-RES-CORE-01", "S3A-RES-EVENT-01", "S3A-RES-FAIL-01"],
    },
    TestEvidence {
        source_path: "compiler/crates/orange-compiler/src/semantics.rs",
        test: "analysis_is_repeatable_for_typed_success_and_failure",
        rules: &["S3A-DETERMINISM-01"],
    },
    TestEvidence {
        source_path: "compiler/crates/orange-compiler/src/semantics.rs",
        test: "mutated_s3a_sources_preserve_phase_gates_and_repeatability",
        rules: &[
            "S3A-PHASE-01",
            "S3A-DIAG-01",
            "S3A-ATOMIC-01",
            "S3A-DETERMINISM-01",
        ],
    },
    TestEvidence {
        source_path: "compiler/crates/orange-compiler/src/core.rs",
        test: "core_accessors_preserve_source_order_and_derive_value_types",
        rules: &["S3A-CORE-ORDER-01", "S3A-CORE-CONTENT-01"],
    },
    TestEvidence {
        source_path: "compiler/crates/orange-compiler/src/core.rs",
        test: "negative_zero_is_canonical_zero",
        rules: &["S3A-INT-01", "S3A-EVAL-INT-01"],
    },
    TestEvidence {
        source_path: "compiler/crates/orange-compiler/src/core.rs",
        test: "word_display_is_fixed_width_lowercase_hexadecimal",
        rules: &["S3A-EVAL-WORD8-01"],
    },
    TestEvidence {
        source_path: "compiler/crates/orange-compiler/src/eval.rs",
        test: "production_evaluation_limit_matches_the_s3a_specification",
        rules: &["S3A-RES-EVAL-01"],
    },
    TestEvidence {
        source_path: "compiler/crates/orange-compiler/src/eval.rs",
        test: "evaluates_all_values_in_source_order_with_stable_display",
        rules: &[
            "S3A-CORE-ORDER-01",
            "S3A-EVAL-LINE-01",
            "S3A-EVAL-INT-01",
            "S3A-EVAL-WORD8-01",
        ],
    },
    TestEvidence {
        source_path: "compiler/crates/orange-compiler/src/eval.rs",
        test: "empty_core_evaluates_to_an_empty_value_set",
        rules: &["S3A-EVAL-EMPTY-01"],
    },
    TestEvidence {
        source_path: "compiler/crates/orange-compiler/src/eval.rs",
        test: "evaluation_limit_fails_without_partial_values",
        rules: &["S3A-ATOMIC-01", "S3A-RES-EVAL-01", "S3A-RES-FAIL-01"],
    },
    TestEvidence {
        source_path: "compiler/crates/orange-compiler/src/eval.rs",
        test: "evaluation_is_repeatable",
        rules: &["S3A-DETERMINISM-01"],
    },
    TestEvidence {
        source_path: "compiler/crates/orangec/tests/cli.rs",
        test: "evaluation_requires_exactly_one_source_before_reading",
        rules: &["S3A-CLI-EVAL-01"],
    },
    TestEvidence {
        source_path: "compiler/crates/orangec/tests/cli.rs",
        test: "evaluation_of_an_empty_core_succeeds_without_output",
        rules: &["S3A-CLI-EVAL-01", "S3A-EVAL-EMPTY-01"],
    },
    TestEvidence {
        source_path: "compiler/crates/orangec/tests/cli.rs",
        test: "rejects_oversized_standard_input_with_a_stable_diagnostic",
        rules: &["S3A-CLI-EVAL-01", "S3A-HOST-FAIL-01"],
    },
    TestEvidence {
        source_path: "compiler/crates/orangec/tests/cli.rs",
        test: "closed_operating_system_output_pipe_is_a_quiet_failure",
        rules: &["S3A-HOST-FAIL-01"],
    },
    TestEvidence {
        source_path: "compiler/crates/orangec/tests/cli.rs",
        test: "evaluation_emits_no_partial_values_after_a_semantic_error",
        rules: &["S3A-PHASE-01", "S3A-ATOMIC-01", "S3A-DETERMINISM-01"],
    },
    TestEvidence {
        source_path: "compiler/crates/orangec/tests/cli.rs",
        test: "evaluation_emits_no_partial_values_after_a_parser_error",
        rules: &["S3A-PHASE-01", "S3A-ATOMIC-01", "S3A-DETERMINISM-01"],
    },
    TestEvidence {
        source_path: "compiler/crates/orangec/tests/cli.rs",
        test: "reports_cross_phase_file_errors_in_argument_order_repeatably",
        rules: &[
            "S3A-PHASE-01",
            "S3A-DIAG-01",
            "S3A-ATOMIC-01",
            "S3A-DETERMINISM-01",
        ],
    },
    TestEvidence {
        source_path: "compiler/crates/orangec/tests/cli.rs",
        test: "lexical_errors_prevent_parser_cascades",
        rules: &["S3A-PHASE-01", "S3A-ATOMIC-01", "S3A-DETERMINISM-01"],
    },
    TestEvidence {
        source_path: "compiler/crates/orangec/src/main.rs",
        test: "reference_evaluation_requires_exactly_one_source",
        rules: &["S3A-CLI-EVAL-01"],
    },
    TestEvidence {
        source_path: "compiler/crates/orangec/src/main.rs",
        test: "bounded_read_preserves_a_non_interrupted_probe_error",
        rules: &["S3A-HOST-FAIL-01"],
    },
    TestEvidence {
        source_path: "compiler/crates/orangec/src/main.rs",
        test: "evaluation_input_failure_has_a_stable_status_and_diagnostic",
        rules: &["S3A-HOST-FAIL-01"],
    },
    TestEvidence {
        source_path: "compiler/crates/orangec/src/main.rs",
        test: "evaluation_allocation_failure_has_a_stable_status_and_diagnostic",
        rules: &["S3A-HOST-FAIL-01"],
    },
    TestEvidence {
        source_path: "compiler/crates/orange-compiler/src/source.rs",
        test: "checkpoint_reservation_failure_rejects_the_source_without_partial_state",
        rules: &["S3A-HOST-FAIL-01"],
    },
    TestEvidence {
        source_path: "compiler/crates/orangec/src/main.rs",
        test: "evaluation_non_regular_source_has_a_stable_status_and_diagnostic",
        rules: &["S3A-HOST-FAIL-01"],
    },
    TestEvidence {
        source_path: "compiler/crates/orangec/src/main.rs",
        test: "evaluation_output_failure_has_a_stable_status_and_diagnostic",
        rules: &["S3A-EVAL-OUTPUT-FAIL-01", "S3A-HOST-FAIL-01"],
    },
    TestEvidence {
        source_path: "compiler/crates/orangec/src/main.rs",
        test: "evaluation_output_partial_write_failure_preserves_only_the_accepted_prefix",
        rules: &["S3A-EVAL-OUTPUT-FAIL-01", "S3A-HOST-FAIL-01"],
    },
    TestEvidence {
        source_path: "compiler/crates/orangec/src/main.rs",
        test: "evaluation_output_failure_is_not_retried_during_teardown",
        rules: &["S3A-EVAL-OUTPUT-FAIL-01", "S3A-HOST-FAIL-01"],
    },
    TestEvidence {
        source_path: "compiler/crates/orangec/src/main.rs",
        test: "evaluation_output_partial_broken_pipe_is_quiet_and_not_retried",
        rules: &["S3A-EVAL-OUTPUT-FAIL-01", "S3A-HOST-FAIL-01"],
    },
    TestEvidence {
        source_path: "compiler/crates/orangec/src/main.rs",
        test: "evaluation_output_flush_failure_reports_failure_after_complete_bytes",
        rules: &["S3A-EVAL-OUTPUT-FAIL-01", "S3A-HOST-FAIL-01"],
    },
    TestEvidence {
        source_path: "compiler/crates/orangec/src/main.rs",
        test: "evaluation_broken_pipe_is_a_quiet_failure",
        rules: &["S3A-EVAL-OUTPUT-FAIL-01", "S3A-HOST-FAIL-01"],
    },
];

fn orangec() -> Command {
    Command::new(env!("CARGO_BIN_EXE_orangec"))
}

fn fixture_directory() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/s3a")
}

fn run_fixture(command: &str, path: &Path) -> Output {
    orangec().arg(command).arg(path).output().unwrap()
}

fn run_stdin(command: &str, source: &[u8]) -> Output {
    let mut child = orangec()
        .arg(command)
        .arg("-")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(source).unwrap();
    child.wait_with_output().unwrap()
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

fn assert_repeatable(first: &Output, second: &Output, context: &str) {
    assert_eq!(
        first.status.code(),
        second.status.code(),
        "{context} changed exit status"
    );
    assert_eq!(first.stdout, second.stdout, "{context} changed stdout");
    assert_eq!(first.stderr, second.stderr, "{context} changed stderr");
}

fn assert_silent_success(output: &Output, context: &str) {
    assert_eq!(output.status.code(), Some(0), "{context} failed");
    assert_eq!(output.stdout, b"", "{context} stdout");
    assert_eq!(output.stderr, b"", "{context} stderr");
}

fn assert_failure(
    output: &Output,
    expected_codes: &[&str],
    expected_messages: &[&str],
    expected_locations: Option<&[&str]>,
    context: &str,
) {
    assert_eq!(output.status.code(), Some(1), "{context} status");
    assert_eq!(output.stdout, b"", "{context} emitted partial output");
    let stderr = std::str::from_utf8(&output.stderr).unwrap();
    assert_eq!(
        diagnostic_codes(stderr),
        expected_codes,
        "{context} diagnostic set:\n{stderr}"
    );
    if let Some(locations) = expected_locations {
        assert_eq!(
            primary_locations(stderr),
            locations,
            "{context} primary locations:\n{stderr}"
        );
    }
    for message in expected_messages {
        assert!(
            stderr.contains(message),
            "{context} missing {message:?}:\n{stderr}"
        );
    }
}

fn documented_rule_ids() -> Vec<&'static str> {
    SEMANTICS_SPECIFICATION
        .lines()
        .filter_map(|line| {
            let row = line.strip_prefix("| `")?;
            let (rule, _) = row.split_once("` |")?;
            rule.starts_with("S3A-").then_some(rule)
        })
        .collect()
}

fn documented_evidence_layers() -> BTreeMap<&'static str, &'static str> {
    SEMANTICS_SPECIFICATION
        .lines()
        .filter_map(|line| {
            let row = line.strip_prefix("| `")?;
            let (rule, _) = row.split_once("` |")?;
            if !rule.starts_with("S3A-") {
                return None;
            }
            let evidence = line.strip_suffix('|')?.rsplit('|').next()?.trim();
            Some((rule, evidence))
        })
        .collect()
}

fn required_evidence_label(rule: &str) -> &'static str {
    match rule {
        "S3A-PHASE-01"
        | "S3A-DECL-01"
        | "S3A-TYPE-INT-01"
        | "S3A-TYPE-WORD8-01"
        | "S3A-TYPE-REJECT-01"
        | "S3A-LIT-DECODE-01"
        | "S3A-INT-01"
        | "S3A-WORD-SIGN-01"
        | "S3A-WORD-RANGE-01"
        | "S3A-DIAG-01"
        | "S3A-ATOMIC-01"
        | "S3A-CORE-MEMBERSHIP-01"
        | "S3A-EVAL-LINE-01"
        | "S3A-EVAL-INT-01"
        | "S3A-EVAL-WORD8-01"
        | "S3A-EVAL-EMPTY-01"
        | "S3A-DETERMINISM-01" => "CLI and unit",
        "S3A-GRAMMAR-01" => "CLI and parser unit",
        "S3A-LIT-ZEROES-01" | "S3A-LIT-BITS-01" | "S3A-RES-DIAG-01" => "Generated CLI and unit",
        "S3A-CORE-ORDER-01" | "S3A-CORE-CONTENT-01" => "Unit and CLI observation",
        "S3A-CLI-EVAL-01" => "CLI",
        "S3A-EVAL-OUTPUT-FAIL-01" => "Injected writer unit",
        "S3A-RES-CORE-01" | "S3A-RES-EVENT-01" | "S3A-RES-EVAL-01" | "S3A-RES-FAIL-01" => {
            "Injected-limit unit"
        }
        "S3A-HOST-FAIL-01" => "Representative fault-injection unit",
        _ => panic!("unknown S3a rule {rule}"),
    }
}

fn required_layers_for_label(label: &str) -> u16 {
    match label {
        "CLI and unit" | "Unit and CLI observation" => CLI_EVIDENCE | UNIT_EVIDENCE,
        "CLI and parser unit" => CLI_EVIDENCE | PARSER_UNIT_EVIDENCE,
        "Generated CLI and unit" => GENERATED_CLI_EVIDENCE | UNIT_EVIDENCE,
        "CLI" => CLI_EVIDENCE,
        "Injected writer unit" => UNIT_EVIDENCE | INJECTED_WRITER_EVIDENCE,
        "Injected-limit unit" => UNIT_EVIDENCE | INJECTED_LIMIT_EVIDENCE,
        "Representative fault-injection unit" => {
            UNIT_EVIDENCE | IO_FAULT_EVIDENCE | ALLOCATION_FAULT_EVIDENCE | HOST_BOUNDARY_EVIDENCE
        }
        _ => panic!("unknown S3a evidence layer {label}"),
    }
}

fn evidence_source(source_path: &str) -> Option<&'static str> {
    match source_path {
        "compiler/crates/orange-compiler/src/core.rs" => Some(CORE_SOURCE),
        "compiler/crates/orange-compiler/src/eval.rs" => Some(EVAL_SOURCE),
        "compiler/crates/orange-compiler/src/parser.rs" => Some(PARSER_SOURCE),
        "compiler/crates/orange-compiler/src/semantics.rs" => Some(SEMANTICS_SOURCE),
        "compiler/crates/orange-compiler/src/source.rs" => Some(SOURCE_SOURCE),
        "compiler/crates/orangec/src/main.rs" => Some(ORANGEC_MAIN_SOURCE),
        "compiler/crates/orangec/tests/cli.rs" => Some(CLI_TEST_SOURCE),
        "compiler/crates/orangec/tests/s3a_conformance.rs" => Some(S3A_CONFORMANCE_SOURCE),
        _ => None,
    }
}

fn evidence_test_indentation(source_path: &str) -> &'static str {
    match source_path {
        "compiler/crates/orangec/tests/cli.rs"
        | "compiler/crates/orangec/tests/s3a_conformance.rs" => "",
        _ => "    ",
    }
}

fn expected_evidence_test_brace_stack(source: &str, source_path: &str) -> Option<Vec<usize>> {
    match source_path {
        "compiler/crates/orangec/tests/cli.rs"
        | "compiler/crates/orangec/tests/s3a_conformance.rs" => Some(Vec::new()),
        _ => {
            const TEST_MODULE: &str = "#[cfg(test)]\nmod tests {";
            let openings: Vec<_> = source
                .match_indices(TEST_MODULE)
                .map(|(offset, _)| offset + TEST_MODULE.len() - 1)
                .collect();
            let [opening] = openings.as_slice() else {
                return None;
            };
            (rust_code_brace_stack_at(source, *opening) == Some(Vec::new()))
                .then_some(vec![*opening])
        }
    }
}

fn exact_test_declaration(source_path: &str, test: &str) -> String {
    let indentation = evidence_test_indentation(source_path);
    format!("{indentation}#[test]\n{indentation}fn {test}(")
}

fn raw_string_start(bytes: &[u8], offset: usize) -> Option<(usize, usize)> {
    let r_offset = match (bytes.get(offset), bytes.get(offset + 1)) {
        (Some(b'r'), _) => offset,
        (Some(b'b' | b'c'), Some(b'r')) => offset + 1,
        _ => return None,
    };
    let mut cursor = r_offset + 1;
    while bytes.get(cursor) == Some(&b'#') {
        cursor += 1;
    }
    (bytes.get(cursor) == Some(&b'"')).then_some((cursor - r_offset - 1, cursor + 1))
}

fn char_literal_start(source: &str, offset: usize) -> bool {
    let tail = &source[offset + 1..];
    if tail.starts_with('\\') {
        return true;
    }
    let Some(character) = tail.chars().next() else {
        return false;
    };
    tail[character.len_utf8()..].starts_with('\'')
}

fn rust_code_brace_stack_at(source: &str, offset: usize) -> Option<Vec<usize>> {
    let bytes = source.as_bytes();
    if offset > bytes.len() || !source.is_char_boundary(offset) {
        return None;
    }

    let mut cursor = 0;
    let mut brace_stack = Vec::new();
    let mut block_comment_depth = 0_usize;
    let mut line_comment = false;
    let mut string = false;
    let mut string_escape = false;
    let mut character = false;
    let mut character_escape = false;
    let mut raw_string_hashes = None;

    while cursor < offset {
        if line_comment {
            if bytes[cursor] == b'\n' || bytes[cursor] == b'\r' {
                line_comment = false;
            }
            cursor += 1;
            continue;
        }
        if block_comment_depth != 0 {
            if bytes.get(cursor..cursor + 2) == Some(b"/*") {
                block_comment_depth += 1;
                cursor += 2;
            } else if bytes.get(cursor..cursor + 2) == Some(b"*/") {
                block_comment_depth -= 1;
                cursor += 2;
            } else {
                cursor += 1;
            }
            continue;
        }
        if let Some(hashes) = raw_string_hashes {
            let terminator_end = cursor + 1 + hashes;
            if bytes[cursor] == b'"'
                && terminator_end <= bytes.len()
                && bytes[cursor + 1..terminator_end]
                    .iter()
                    .all(|byte| *byte == b'#')
            {
                raw_string_hashes = None;
                cursor += hashes + 1;
            } else {
                cursor += 1;
            }
            continue;
        }
        if string {
            if string_escape {
                string_escape = false;
            } else if bytes[cursor] == b'\\' {
                string_escape = true;
            } else if bytes[cursor] == b'"' {
                string = false;
            }
            cursor += 1;
            continue;
        }
        if character {
            if character_escape {
                character_escape = false;
            } else if bytes[cursor] == b'\\' {
                character_escape = true;
            } else if bytes[cursor] == b'\'' {
                character = false;
            }
            cursor += 1;
            continue;
        }

        if bytes.get(cursor..cursor + 2) == Some(b"//") {
            line_comment = true;
            cursor += 2;
        } else if bytes.get(cursor..cursor + 2) == Some(b"/*") {
            block_comment_depth = 1;
            cursor += 2;
        } else if let Some((hashes, after_opening)) = raw_string_start(bytes, cursor) {
            raw_string_hashes = Some(hashes);
            cursor = after_opening;
        } else if bytes[cursor] == b'"' {
            string = true;
            cursor += 1;
        } else if bytes[cursor] == b'\'' && char_literal_start(source, cursor) {
            character = true;
            cursor += 1;
        } else {
            match bytes[cursor] {
                b'{' => brace_stack.push(cursor),
                b'}' => {
                    brace_stack.pop()?;
                }
                _ => {}
            }
            cursor += 1;
        }
    }

    (!line_comment
        && block_comment_depth == 0
        && raw_string_hashes.is_none()
        && !string
        && !character)
        .then_some(brace_stack)
}

fn rust_code_brace_depth_at(source: &str, offset: usize) -> Option<usize> {
    rust_code_brace_stack_at(source, offset).map(|stack| stack.len())
}

fn internal_evidence_layers(source_path: &str, test: &str) -> u16 {
    let broad_layer = match source_path {
        "compiler/crates/orangec/tests/cli.rs" => CLI_EVIDENCE,
        "compiler/crates/orange-compiler/src/parser.rs" => UNIT_EVIDENCE | PARSER_UNIT_EVIDENCE,
        _ => UNIT_EVIDENCE,
    };
    let specialized_layer = match (source_path, test) {
        (
            "compiler/crates/orange-compiler/src/semantics.rs",
            "diagnostic_and_resource_limits_fail_closed"
            | "injected_limits_match_normative_event_and_core_node_accounting",
        )
        | (
            "compiler/crates/orange-compiler/src/eval.rs",
            "evaluation_limit_fails_without_partial_values",
        ) => INJECTED_LIMIT_EVIDENCE,
        (
            "compiler/crates/orangec/src/main.rs",
            "evaluation_broken_pipe_is_a_quiet_failure"
            | "evaluation_output_failure_has_a_stable_status_and_diagnostic"
            | "evaluation_output_failure_is_not_retried_during_teardown"
            | "evaluation_output_flush_failure_reports_failure_after_complete_bytes"
            | "evaluation_output_partial_broken_pipe_is_quiet_and_not_retried"
            | "evaluation_output_partial_write_failure_preserves_only_the_accepted_prefix",
        ) => INJECTED_WRITER_EVIDENCE | IO_FAULT_EVIDENCE,
        (
            "compiler/crates/orangec/src/main.rs",
            "bounded_read_preserves_a_non_interrupted_probe_error"
            | "evaluation_input_failure_has_a_stable_status_and_diagnostic",
        ) => IO_FAULT_EVIDENCE,
        (
            "compiler/crates/orangec/src/main.rs",
            "evaluation_allocation_failure_has_a_stable_status_and_diagnostic",
        )
        | (
            "compiler/crates/orange-compiler/src/source.rs",
            "checkpoint_reservation_failure_rejects_the_source_without_partial_state",
        ) => ALLOCATION_FAULT_EVIDENCE,
        (
            "compiler/crates/orangec/src/main.rs",
            "evaluation_non_regular_source_has_a_stable_status_and_diagnostic",
        )
        | (
            "compiler/crates/orangec/tests/cli.rs",
            "closed_operating_system_output_pipe_is_a_quiet_failure",
        ) => HOST_BOUNDARY_EVIDENCE,
        _ => 0,
    };
    broad_layer | specialized_layer
}

fn layer_names(layers: u16) -> Vec<&'static str> {
    [
        (CLI_EVIDENCE, "CLI"),
        (GENERATED_CLI_EVIDENCE, "generated CLI"),
        (UNIT_EVIDENCE, "unit"),
        (PARSER_UNIT_EVIDENCE, "parser unit"),
        (INJECTED_WRITER_EVIDENCE, "injected writer"),
        (INJECTED_LIMIT_EVIDENCE, "injected limit"),
        (IO_FAULT_EVIDENCE, "I/O fault"),
        (ALLOCATION_FAULT_EVIDENCE, "allocation fault"),
        (HOST_BOUNDARY_EVIDENCE, "host boundary"),
    ]
    .into_iter()
    .filter_map(|(layer, name)| (layers & layer != 0).then_some(name))
    .collect()
}

#[test]
fn named_evidence_depth_rejects_noncode_and_nested_test_lookalikes() {
    const UNIT_PATH: &str = "compiler/crates/orange-compiler/src/core.rs";

    let top_level = "#[test]\nfn real() {}\n";
    assert_eq!(rust_code_brace_depth_at(top_level, 0), Some(0));

    let module_level = "#[cfg(test)]\nmod tests {\n    #[test]\n    fn real() {}\n}\n";
    let module_test = module_level.find("    #[test]").unwrap();
    assert_eq!(rust_code_brace_depth_at(module_level, module_test), Some(1));
    assert_eq!(
        rust_code_brace_stack_at(module_level, module_test),
        expected_evidence_test_brace_stack(module_level, UNIT_PATH)
    );

    let disabled_module = "#[cfg(any())]\nmod tests {\n    #[test]\n    fn fake() {}\n}\n";
    let disabled_test = disabled_module.find("    #[test]").unwrap();
    assert_eq!(
        rust_code_brace_depth_at(disabled_module, disabled_test),
        Some(1)
    );
    assert_eq!(
        expected_evidence_test_brace_stack(disabled_module, UNIT_PATH),
        None
    );

    let nested = r#"mod tests {
    fn wrapper<'a>() { let _ = '{'; let _ = "}";
    #[test]
    fn fake() {}
    }
}
"#;
    let nested_test = nested.find("    #[test]").unwrap();
    assert_eq!(rust_code_brace_depth_at(nested, nested_test), Some(2));

    let block_comment = r#"mod tests {
    /* nested /* comment */
    #[test]
    fn fake() {}
    */
}
"#;
    let block_test = block_comment.find("    #[test]").unwrap();
    assert_eq!(rust_code_brace_depth_at(block_comment, block_test), None);

    let normal_string = r#"mod tests {
    let _ = "{
    #[test]
    fn fake() {}
    }";
}
"#;
    let normal_test = normal_string.find("    #[test]").unwrap();
    assert_eq!(rust_code_brace_depth_at(normal_string, normal_test), None);

    let raw_string = r###"mod tests {
    let _ = br##"{ " quoted
    #[test]
    fn fake() {}
    " }"##;
}
"###;
    let raw_test = raw_string.find("    #[test]").unwrap();
    assert_eq!(rust_code_brace_depth_at(raw_string, raw_test), None);
}

#[test]
fn s3a_rule_index_is_exact_and_covered() {
    let expected: BTreeSet<_> = RULES.iter().map(|rule| rule.id).collect();
    assert_eq!(expected.len(), RULES.len(), "duplicate expected rule ID");

    let documented = documented_rule_ids();
    let documented_set: BTreeSet<_> = documented.iter().copied().collect();
    assert_eq!(
        documented_set.len(),
        documented.len(),
        "duplicate rule ID in docs/SEMANTICS_2026.md"
    );
    assert_eq!(
        documented_set, expected,
        "S3a specification rule index drifted"
    );
    let documented_layers = documented_evidence_layers();
    assert_eq!(
        documented_layers.len(),
        RULES.len(),
        "S3a evidence-layer rows drifted"
    );
    for requirement in RULES {
        let expected_label = required_evidence_label(requirement.id);
        assert_eq!(
            documented_layers.get(requirement.id).copied(),
            Some(expected_label),
            "{} evidence-layer declaration drifted",
            requirement.id
        );
        assert_eq!(
            requirement.layers,
            required_layers_for_label(expected_label),
            "{} evidence-layer capability mask drifted",
            requirement.id
        );
    }
    let mut covered = BTreeSet::new();
    let mut observed_layers = BTreeMap::<&str, u16>::new();
    let mut fixtures = BTreeSet::new();
    for case in CASES {
        assert!(
            fixtures.insert(case.fixture),
            "duplicate conformance fixture {}",
            case.fixture
        );
        assert!(!case.rules.is_empty(), "{} has no rule IDs", case.fixture);
        let mut case_rules = BTreeSet::new();
        for rule in case.rules {
            assert!(
                expected.contains(rule),
                "unknown rule {rule} on {}",
                case.fixture
            );
            assert!(
                case_rules.insert(*rule),
                "duplicate rule {rule} on {}",
                case.fixture
            );
            covered.insert(*rule);
            *observed_layers.entry(*rule).or_default() |= CLI_EVIDENCE;
        }
    }
    let mut named_evidence = BTreeSet::new();
    let evidence = GENERATED_EVIDENCE
        .iter()
        .map(|evidence| (evidence, GENERATED_CLI_EVIDENCE | CLI_EVIDENCE))
        .chain(INTERNAL_EVIDENCE.iter().map(|evidence| {
            (
                evidence,
                internal_evidence_layers(evidence.source_path, evidence.test),
            )
        }));
    for (evidence, layers) in evidence {
        assert!(
            named_evidence.insert((evidence.source_path, evidence.test)),
            "duplicate named evidence {}::{}",
            evidence.source_path,
            evidence.test
        );
        let Some(source) = evidence_source(evidence.source_path) else {
            panic!("unknown evidence source path {}", evidence.source_path);
        };
        let declaration = exact_test_declaration(evidence.source_path, evidence.test);
        let offsets: Vec<_> = source
            .match_indices(&declaration)
            .map(|(offset, _)| offset)
            .collect();
        assert_eq!(
            offsets.len(),
            1,
            "{} must contain exactly one unconditional test declaration for {}; found {}",
            evidence.source_path,
            evidence.test,
            offsets.len()
        );
        let expected_brace_stack = expected_evidence_test_brace_stack(source, evidence.source_path);
        assert!(
            expected_brace_stack.is_some(),
            "{} has no unique executable test-harness container",
            evidence.source_path
        );
        assert_eq!(
            rust_code_brace_stack_at(source, offsets[0]),
            expected_brace_stack,
            "{} test {} is not executable in the expected test-harness container",
            evidence.source_path,
            evidence.test
        );
        let indentation = evidence_test_indentation(evidence.source_path);
        let previous_item_boundary = format!("{indentation}}}");
        let mut found_previous_item = false;
        let mut controlling_attribute = None;
        for line in source[..offsets[0]].lines().rev() {
            if line == previous_item_boundary {
                found_previous_item = true;
                break;
            }
            let line = line.trim();
            if controlling_attribute.is_none() && line.starts_with("#[") {
                controlling_attribute = Some(line);
            }
        }
        assert!(
            found_previous_item,
            "{} test {} has no preceding item boundary at its expected module depth",
            evidence.source_path, evidence.test
        );
        assert!(
            controlling_attribute.is_none(),
            "{} test {} has an additional controlling attribute {:?}",
            evidence.source_path,
            evidence.test,
            controlling_attribute
        );
        assert!(
            !evidence.rules.is_empty(),
            "{} has no rule IDs",
            evidence.test
        );
        let mut evidence_rules = BTreeSet::new();
        for rule in evidence.rules {
            assert!(
                expected.contains(rule),
                "unknown rule {rule} on {}",
                evidence.test
            );
            assert!(
                evidence_rules.insert(*rule),
                "duplicate rule {rule} on {}::{}",
                evidence.source_path,
                evidence.test
            );
            covered.insert(*rule);
            *observed_layers.entry(*rule).or_default() |= layers;
        }
    }
    assert_eq!(
        covered, expected,
        "S3a rule IDs without executable evidence"
    );
    for requirement in RULES {
        let observed = observed_layers.get(requirement.id).copied().unwrap_or(0);
        let missing = requirement.layers & !observed;
        assert_eq!(
            missing,
            0,
            "{} is missing required evidence layers {:?}; observed {:?}",
            requirement.id,
            layer_names(missing),
            layer_names(observed)
        );
    }
}

#[test]
fn s3a_cli_conformance_corpus_is_exact_and_repeatable() {
    let directory = fixture_directory();
    let mut observed: Vec<_> = fs::read_dir(&directory)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().into_string().unwrap())
        .collect();
    observed.sort();
    let mut expected: Vec<_> = CASES.iter().map(|case| case.fixture).collect();
    expected.sort_unstable();
    assert_eq!(observed, expected, "unexpected S3a fixture inventory");

    for case in CASES {
        let path = directory.join(case.fixture);
        let first_check = run_fixture("check", &path);
        let second_check = run_fixture("check", &path);
        let first_eval = run_fixture("eval", &path);
        let second_eval = run_fixture("eval", &path);

        assert_repeatable(
            &first_check,
            &second_check,
            &format!("{} check", case.fixture),
        );
        assert_repeatable(&first_eval, &second_eval, &format!("{} eval", case.fixture));

        match case.expectation {
            Expectation::Success(expected_stdout) => {
                assert_silent_success(&first_check, &format!("{} check", case.fixture));
                assert_eq!(first_eval.status.code(), Some(0), "{} eval", case.fixture);
                assert_eq!(
                    first_eval.stdout,
                    expected_stdout.as_bytes(),
                    "{} eval stdout",
                    case.fixture
                );
                assert_eq!(first_eval.stderr, b"", "{} eval stderr", case.fixture);
            }
            Expectation::Failure {
                diagnostic_codes,
                messages,
                primary_locations,
            } => {
                assert_failure(
                    &first_check,
                    diagnostic_codes,
                    messages,
                    Some(primary_locations),
                    &format!("{} check", case.fixture),
                );
                assert_failure(
                    &first_eval,
                    diagnostic_codes,
                    messages,
                    Some(primary_locations),
                    &format!("{} eval", case.fixture),
                );
                assert_eq!(
                    first_check.stderr, first_eval.stderr,
                    "{} check/eval diagnostic mismatch",
                    case.fixture
                );
            }
        }
    }
}

#[test]
fn s3a_mixed_semantic_diagnostics_are_source_ordered_and_repeatable() {
    let source = concat!(
        "edition 2026; module mixed_errors {\n",
        "  spec repeated() {}\n",
        "  spec repeated() -> Word[16] { 1 }\n",
        "  spec unsupported() -> Integer { 1 }\n",
        "  spec negative() -> Word[8] { -1 }\n",
        "  spec valid_between() -> Int { 42 }\n",
        "  spec out_of_range() -> Word[8] { 256 }\n",
        "}\n",
    );
    let first_check = run_stdin("check", source.as_bytes());
    let second_check = run_stdin("check", source.as_bytes());
    let first_eval = run_stdin("eval", source.as_bytes());
    let second_eval = run_stdin("eval", source.as_bytes());

    assert_repeatable(&first_check, &second_check, "mixed semantic check");
    assert_repeatable(&first_eval, &second_eval, "mixed semantic eval");
    for (context, output) in [
        ("mixed semantic check", &first_check),
        ("mixed semantic eval", &first_eval),
    ] {
        assert_failure(
            output,
            &["ORC0201", "ORC0204", "ORC0203", "ORC0206", "ORC0207"],
            &[
                "duplicate spec function `repeated`",
                "only the exact type `Word[8]` is supported",
                "unsupported result type `Integer`",
                "`Word[8]` literals cannot be negative",
                "literal is outside the range of `Word[8]`",
            ],
            Some(&["3:8", "3:27", "4:25", "5:32", "7:36"]),
            context,
        );
    }
    assert_eq!(
        first_check.stderr, first_eval.stderr,
        "mixed semantic check/eval diagnostic mismatch"
    );
}

#[test]
fn s3a_significant_bit_boundary_is_exact_and_repeatable() {
    const INTEGER_BITS: usize = 16_384;

    let accepted_magnitude = format!("8{}", "0".repeat((INTEGER_BITS - 1) / 4));
    let accepted_source = format!(
        "edition 2026; module bit_limit {{ spec exact() -> Int {{ 0x{accepted_magnitude} }} }}\n"
    );
    let first_accepted = run_stdin("check", accepted_source.as_bytes());
    let second_accepted = run_stdin("check", accepted_source.as_bytes());
    assert_repeatable(
        &first_accepted,
        &second_accepted,
        "exact significant-bit boundary",
    );
    assert_silent_success(&first_accepted, "exact significant-bit boundary");

    let rejected_magnitude = format!("1{}", "0".repeat(INTEGER_BITS / 4));
    let rejected_source = format!(
        "edition 2026; module bit_limit {{ spec over() -> Int {{ 0x{rejected_magnitude} }} }}\n"
    );
    let first_check = run_stdin("check", rejected_source.as_bytes());
    let second_check = run_stdin("check", rejected_source.as_bytes());
    let first_eval = run_stdin("eval", rejected_source.as_bytes());
    let second_eval = run_stdin("eval", rejected_source.as_bytes());
    assert_repeatable(&first_check, &second_check, "over-limit check");
    assert_repeatable(&first_eval, &second_eval, "over-limit eval");
    for (context, output) in [
        ("over-limit check", &first_check),
        ("over-limit eval", &first_eval),
    ] {
        assert_failure(
            output,
            &["ORC0205"],
            &["integer magnitude exceeds the 16384-significant-bit limit"],
            None,
            context,
        );
    }
    let expected_location = format!("1:{}", rejected_source.find("0x").unwrap() + 1);
    for output in [&first_check, &first_eval] {
        let stderr = std::str::from_utf8(&output.stderr).unwrap();
        assert_eq!(
            primary_locations(stderr),
            std::slice::from_ref(&expected_location)
        );
    }
    assert_eq!(
        first_check.stderr, first_eval.stderr,
        "over-limit check/eval diagnostic mismatch"
    );
}

#[test]
fn s3a_semantic_diagnostic_budget_is_exact_and_repeatable() {
    let mut source = String::from("edition 2026; module diagnostic_limit {\n");
    let mut expected_locations = Vec::new();
    for index in 0..103 {
        let declaration = format!("  spec bad_{index}() -> Nope {{ {index} }}\n");
        if index < 101 {
            expected_locations.push(format!(
                "{}:{}",
                index + 2,
                declaration.find("Nope").unwrap() + 1
            ));
        }
        source.push_str(&declaration);
    }
    source.push_str("}\n");

    let first_check = run_stdin("check", source.as_bytes());
    let second_check = run_stdin("check", source.as_bytes());
    let first_eval = run_stdin("eval", source.as_bytes());
    let second_eval = run_stdin("eval", source.as_bytes());
    assert_repeatable(&first_check, &second_check, "diagnostic-budget check");
    assert_repeatable(&first_eval, &second_eval, "diagnostic-budget eval");

    let mut expected_codes = vec!["ORC0203"; 100];
    expected_codes.push("ORC0208");
    for (context, output) in [
        ("diagnostic-budget check", &first_check),
        ("diagnostic-budget eval", &first_eval),
    ] {
        assert_failure(
            output,
            &expected_codes,
            &[
                "unsupported result type `Nope`",
                "too many semantic errors; further errors are suppressed",
            ],
            None,
            context,
        );
        let stderr = std::str::from_utf8(&output.stderr).unwrap();
        assert_eq!(
            primary_locations(stderr),
            expected_locations,
            "{context} primary locations"
        );
    }
    assert_eq!(
        first_check.stderr, first_eval.stderr,
        "diagnostic-budget check/eval mismatch"
    );
}

#[test]
fn s3a_leading_zeroes_are_budget_neutral_and_repeatable() {
    const INTEGER_BITS: usize = 16_384;

    let zeroes = "0".repeat(INTEGER_BITS + 1);
    let source = format!(
        "edition 2026; module leading_zeroes {{ spec value() -> Int {{ 0x{zeroes}2a }} }}\n"
    );
    let first_check = run_stdin("check", source.as_bytes());
    let second_check = run_stdin("check", source.as_bytes());
    let first_eval = run_stdin("eval", source.as_bytes());
    let second_eval = run_stdin("eval", source.as_bytes());

    assert_repeatable(&first_check, &second_check, "leading-zero check");
    assert_repeatable(&first_eval, &second_eval, "leading-zero eval");
    assert_silent_success(&first_check, "leading-zero check");
    assert_eq!(first_eval.status.code(), Some(0), "leading-zero eval");
    assert_eq!(
        first_eval.stdout, b"leading_zeroes::value: Int = 42\n",
        "leading-zero eval stdout"
    );
    assert_eq!(first_eval.stderr, b"", "leading-zero eval stderr");
}

#[test]
fn s3a_case_sensitive_spelling_is_exact_and_repeatable() {
    let accepted = concat!(
        "edition 2026; module case_sensitive_names {\n",
        "  spec value() -> Int { 1 }\n",
        "  spec Value() -> Word[8] { 2 }\n",
        "  impl value() {}\n",
        "  impl Value() {}\n",
        "}\n",
    );
    let first_check = run_stdin("check", accepted.as_bytes());
    let second_check = run_stdin("check", accepted.as_bytes());
    let first_eval = run_stdin("eval", accepted.as_bytes());
    let second_eval = run_stdin("eval", accepted.as_bytes());
    assert_repeatable(&first_check, &second_check, "case-sensitive-name check");
    assert_repeatable(&first_eval, &second_eval, "case-sensitive-name eval");
    assert_silent_success(&first_check, "case-sensitive-name check");
    assert_eq!(
        first_eval.status.code(),
        Some(0),
        "case-sensitive-name eval"
    );
    assert_eq!(
        first_eval.stdout,
        concat!(
            "case_sensitive_names::value: Int = 1\n",
            "case_sensitive_names::Value: Word[8] = 0x02\n",
        )
        .as_bytes(),
        "case-sensitive-name eval stdout"
    );
    assert_eq!(first_eval.stderr, b"", "case-sensitive-name eval stderr");

    let rejected = concat!(
        "edition 2026; module case_sensitive_types {\n",
        "  spec lower_int() -> int { 1 }\n",
        "  spec upper_word() -> WORD[8] { 2 }\n",
        "}\n",
    );
    let first_check = run_stdin("check", rejected.as_bytes());
    let second_check = run_stdin("check", rejected.as_bytes());
    let first_eval = run_stdin("eval", rejected.as_bytes());
    let second_eval = run_stdin("eval", rejected.as_bytes());
    assert_repeatable(&first_check, &second_check, "case-sensitive-type check");
    assert_repeatable(&first_eval, &second_eval, "case-sensitive-type eval");
    for (context, output) in [
        ("case-sensitive-type check", &first_check),
        ("case-sensitive-type eval", &first_eval),
    ] {
        assert_failure(
            output,
            &["ORC0203", "ORC0203"],
            &[
                "unsupported result type `int`",
                "unsupported result type `WORD`",
            ],
            Some(&["2:23", "3:24"]),
            context,
        );
    }
    assert_eq!(
        first_check.stderr, first_eval.stderr,
        "case-sensitive-type check/eval mismatch"
    );
}

#[test]
fn s3a_reports_every_later_same_kind_duplicate_repeatably() {
    let source = concat!(
        "edition 2026; module duplicate_later {\n",
        "  spec same() {}\n",
        "  spec same() {}\n",
        "  impl same() {}\n",
        "  spec same() {}\n",
        "  impl same() {}\n",
        "  impl same() {}\n",
        "}\n",
    );
    let first_check = run_stdin("check", source.as_bytes());
    let second_check = run_stdin("check", source.as_bytes());
    let first_eval = run_stdin("eval", source.as_bytes());
    let second_eval = run_stdin("eval", source.as_bytes());
    assert_repeatable(&first_check, &second_check, "every-later-duplicate check");
    assert_repeatable(&first_eval, &second_eval, "every-later-duplicate eval");

    for (context, output) in [
        ("every-later-duplicate check", &first_check),
        ("every-later-duplicate eval", &first_eval),
    ] {
        assert_failure(
            output,
            &["ORC0201", "ORC0201", "ORC0201", "ORC0201"],
            &[
                "duplicate spec function `same`",
                "duplicate impl function `same`",
                "separate declaration namespaces",
            ],
            Some(&["3:8", "5:8", "6:8", "7:8"]),
            context,
        );
        let stderr = std::str::from_utf8(&output.stderr).unwrap();
        assert_eq!(stderr.matches("duplicate spec function `same`").count(), 2);
        assert_eq!(stderr.matches("duplicate impl function `same`").count(), 2);
        assert_eq!(stderr.matches(" ::: <stdin>:2:8\n").count(), 2);
        assert_eq!(stderr.matches(" ::: <stdin>:4:8\n").count(), 2);
    }
    assert_eq!(
        first_check.stderr, first_eval.stderr,
        "every-later-duplicate check/eval mismatch"
    );
}
