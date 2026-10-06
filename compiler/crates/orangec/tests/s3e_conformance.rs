//! External conformance evidence for the proposed Orange 2026 S3e slice.
//!
//! The rule index lives in `docs/LOOPS_2026.md`. This runner requires that
//! index to agree exactly with the evidence map below, runs every fixture and
//! generated source through the real `orangec` binary twice, and checks that
//! every named unit test is declared once in its source's test module.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

const LOOPS_SPECIFICATION: &str = include_str!("../../../../docs/LOOPS_2026.md");
const S3E_CONFORMANCE_SOURCE: &str = include_str!("s3e_conformance.rs");
const LEXER_SOURCE: &str = include_str!("../../orange-compiler/src/lexer.rs");
const PARSER_SOURCE: &str = include_str!("../../orange-compiler/src/parser.rs");
const SEMANTICS_SOURCE: &str = include_str!("../../orange-compiler/src/semantics.rs");
const SEMANTICS_TESTS_SOURCE: &str = include_str!("../../orange-compiler/src/semantics/tests.rs");
const CORE_SOURCE: &str = include_str!("../../orange-compiler/src/core.rs");
const EVAL_SOURCE: &str = include_str!("../../orange-compiler/src/eval.rs");
const DIAGNOSTIC_SOURCE: &str = include_str!("../../orange-compiler/src/diagnostic.rs");

const CLI: u8 = 1 << 0;
const GENERATED_CLI: u8 = 1 << 1;
const UNIT: u8 = 1 << 2;
const PARSER_UNIT: u8 = 1 << 3;

/// Every S3e rule, in the order of the specification's index.
const RULES: [&str; 18] = [
    "S3E-GRAMMAR-01",
    "S3E-WORDS-01",
    "S3E-LOOP-01",
    "S3E-SCOPE-01",
    "S3E-INDEX-01",
    "S3E-UPDATE-01",
    "S3E-FILL-01",
    "S3E-EVAL-01",
    "S3E-DIAG-01",
    "S3E-CORE-01",
    "S3E-RES-BOUND-01",
    "S3E-RES-NEST-01",
    "S3E-RES-EVENT-01",
    "S3E-RES-STEP-01",
    "S3E-RES-STACK-01",
    "S3E-RES-FAIL-01",
    "S3E-COMPAT-01",
    "S3E-DETERMINISM-01",
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

const CASES: [Case; 7] = [
    Case {
        fixture: "valid-loops.or",
        expectation: Expectation::Success(concat!(
            "loops::triangle: Int = 5050\n",
            "loops::power: Int = 18446744073709551616\n",
            "loops::odds: Word[8]^4 = [0x01, 0x03, 0x05, 0x07]\n",
            "loops::reversed: Word[8]^4 = [0x04, 0x03, 0x02, 0x01]\n",
            "loops::nested: Int = 36\n",
            "loops::words: Word[8] = 0x32\n",
            "loops::updated: Word[8]^3 = [0x01, 0x09, 0x03]\n",
            "loops::filled: Word[16]^4 = [0xbeef, 0xbeef, 0xbeef, 0xbeef]\n",
            "loops::longest: Word[32] = 0x00010000\n",
            "loops::names: Int = 6\n",
        )),
        rules: &[
            "S3E-GRAMMAR-01",
            "S3E-WORDS-01",
            "S3E-LOOP-01",
            "S3E-SCOPE-01",
            "S3E-INDEX-01",
            "S3E-UPDATE-01",
            "S3E-FILL-01",
            "S3E-EVAL-01",
            "S3E-RES-BOUND-01",
            "S3E-COMPAT-01",
            "S3E-DETERMINISM-01",
        ],
    },
    Case {
        fixture: "valid-sha256.or",
        expectation: Expectation::Success(concat!(
            "sha256::round_constants: Word[32]^64 = [",
            "0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, ",
            "0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5, ",
            "0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, ",
            "0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174, ",
            "0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, ",
            "0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da, ",
            "0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, ",
            "0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, ",
            "0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, ",
            "0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, ",
            "0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, ",
            "0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, ",
            "0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, ",
            "0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3, ",
            "0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, ",
            "0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2]\n",
            "sha256::initial_hash: Word[32]^8 = [",
            "0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, ",
            "0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19]\n",
            "sha256::abc_block: Word[32]^16 = [",
            "0x61626380, 0x00000000, 0x00000000, 0x00000000, ",
            "0x00000000, 0x00000000, 0x00000000, 0x00000000, ",
            "0x00000000, 0x00000000, 0x00000000, 0x00000000, ",
            "0x00000000, 0x00000000, 0x00000000, 0x00000018]\n",
            "sha256::abc_digest: Word[32]^8 = [",
            "0xba7816bf, 0x8f01cfea, 0x414140de, 0x5dae2223, ",
            "0xb00361a3, 0x96177a9c, 0xb410ff61, 0xf20015ad]\n",
            "sha256::long_first_block: Word[32]^16 = [",
            "0x61626364, 0x62636465, 0x63646566, 0x64656667, ",
            "0x65666768, 0x66676869, 0x6768696a, 0x68696a6b, ",
            "0x696a6b6c, 0x6a6b6c6d, 0x6b6c6d6e, 0x6c6d6e6f, ",
            "0x6d6e6f70, 0x6e6f7071, 0x80000000, 0x00000000]\n",
            "sha256::long_second_block: Word[32]^16 = [",
            "0x00000000, 0x00000000, 0x00000000, 0x00000000, ",
            "0x00000000, 0x00000000, 0x00000000, 0x00000000, ",
            "0x00000000, 0x00000000, 0x00000000, 0x00000000, ",
            "0x00000000, 0x00000000, 0x00000000, 0x000001c0]\n",
            "sha256::long_digest: Word[32]^8 = [",
            "0x248d6a61, 0xd20638b8, 0xe5c02693, 0x0c3e6039, ",
            "0xa33ce459, 0x64ff2167, 0xf6ecedd4, 0x19db06c1]\n",
        )),
        rules: &[
            "S3E-GRAMMAR-01",
            "S3E-LOOP-01",
            "S3E-INDEX-01",
            "S3E-UPDATE-01",
            "S3E-FILL-01",
            "S3E-EVAL-01",
            "S3E-CORE-01",
            "S3E-DETERMINISM-01",
        ],
    },
    Case {
        fixture: "valid-chacha20.or",
        expectation: Expectation::Success(concat!(
            "chacha20::key: Word[8]^32 = [",
            "0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, ",
            "0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f, ",
            "0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, ",
            "0x18, 0x19, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e, 0x1f]\n",
            "chacha20::block_test_vector: Word[32]^16 = [",
            "0xe4e7f110, 0x15593bd1, 0x1fdd0f50, 0xc47120a3, ",
            "0xc7f4d1c7, 0x0368c033, 0x9aaa2204, 0x4e6cd4c3, ",
            "0x466482d2, 0x09aa9f07, 0x05d7c214, 0xa2028bd9, ",
            "0xd19c12b5, 0xb94e16de, 0xe883d0cb, 0x4e3c50a2]\n",
            "chacha20::sunscreen: Word[8]^114 = [",
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
            "chacha20::sunscreen_ciphertext: Word[8]^114 = [",
            "0x6e, 0x2e, 0x35, 0x9a, 0x25, 0x68, 0xf9, 0x80, ",
            "0x41, 0xba, 0x07, 0x28, 0xdd, 0x0d, 0x69, 0x81, ",
            "0xe9, 0x7e, 0x7a, 0xec, 0x1d, 0x43, 0x60, 0xc2, ",
            "0x0a, 0x27, 0xaf, 0xcc, 0xfd, 0x9f, 0xae, 0x0b, ",
            "0xf9, 0x1b, 0x65, 0xc5, 0x52, 0x47, 0x33, 0xab, ",
            "0x8f, 0x59, 0x3d, 0xab, 0xcd, 0x62, 0xb3, 0x57, ",
            "0x16, 0x39, 0xd6, 0x24, 0xe6, 0x51, 0x52, 0xab, ",
            "0x8f, 0x53, 0x0c, 0x35, 0x9f, 0x08, 0x61, 0xd8, ",
            "0x07, 0xca, 0x0d, 0xbf, 0x50, 0x0d, 0x6a, 0x61, ",
            "0x56, 0xa3, 0x8e, 0x08, 0x8a, 0x22, 0xb6, 0x5e, ",
            "0x52, 0xbc, 0x51, 0x4d, 0x16, 0xcc, 0xf8, 0x06, ",
            "0x81, 0x8c, 0xe9, 0x1a, 0xb7, 0x79, 0x37, 0x36, ",
            "0x5a, 0xf9, 0x0b, 0xbf, 0x74, 0xa3, 0x5b, 0xe6, ",
            "0xb4, 0x0b, 0x8e, 0xed, 0xf2, 0x78, 0x5e, 0x42, ",
            "0x87, 0x4d]\n",
        )),
        rules: &[
            "S3E-GRAMMAR-01",
            "S3E-LOOP-01",
            "S3E-INDEX-01",
            "S3E-UPDATE-01",
            "S3E-FILL-01",
            "S3E-EVAL-01",
            "S3E-CORE-01",
            "S3E-COMPAT-01",
            "S3E-DETERMINISM-01",
        ],
    },
    Case {
        fixture: "invalid-loop-syntax.or",
        expectation: Expectation::Failure {
            codes: &[
                "ORC0101", "ORC0101", "ORC0101", "ORC0101", "ORC0108", "ORC0101", "ORC0101",
            ],
            locations: &["6:48", "7:52", "8:48", "9:58", "10:67", "11:47", "12:46"],
            messages: &[
                "expected `with` and the loop's accumulator",
                "expected the loop's second bound",
                "a loop is written `for i in 0..n with s: Type = start { step }`",
                "expected `:` and the accumulator's type",
                "expected `=` after the updated index",
                "`with` follows `^` without grouping parentheses",
                "parenthesize the update or the expression it updates",
                "expected an array length after `;`",
                "expected an index after `[`",
            ],
        },
        rules: &["S3E-GRAMMAR-01", "S3E-DIAG-01", "S3E-DETERMINISM-01"],
    },
    Case {
        fixture: "invalid-loops.or",
        expectation: Expectation::Failure {
            codes: &[
                "ORC0225", "ORC0225", "ORC0225", "ORC0219", "ORC0219", "ORC0214", "ORC0211",
                "ORC0211", "ORC0215",
            ],
            locations: &[
                "6:37", "7:41", "8:40", "9:34", "10:43", "11:29", "12:54", "13:79", "14:83",
            ],
            messages: &[
                "the loop range 3..3 is empty",
                "the loop range 4..2 is empty",
                "a loop bound must be at most 65536",
                "duplicate name `x`",
                "the parameter is here",
                "duplicate name `i`",
                "the loop index is here",
                "this loop has type `Int`, but `Word[8]` is required here",
                "`i` is not a parameter of `before`",
                "`s` is not a parameter or binding of `after`",
                "`+` is not defined for `Word[8]^2`",
            ],
        },
        rules: &[
            "S3E-LOOP-01",
            "S3E-SCOPE-01",
            "S3E-DIAG-01",
            "S3E-RES-BOUND-01",
            "S3E-DETERMINISM-01",
        ],
    },
    Case {
        fixture: "invalid-indices.or",
        expectation: Expectation::Failure {
            codes: &[
                "ORC0226", "ORC0226", "ORC0226", "ORC0226", "ORC0223", "ORC0223", "ORC0223",
                "ORC0223",
            ],
            locations: &[
                "7:55", "8:69", "9:42", "11:82", "12:86", "13:90", "14:83", "15:46",
            ],
            messages: &[
                "an `Int` index may use only integer literals, loop indices, words converted with",
                "this `Int` has no bound",
                "this binding's value has no range",
                "this index runs from 0 through 4, out of range for `Word[8]^4`",
                "this index runs from -1 through 2, out of range for `Word[8]^4`",
                "this index runs from -3 through 3, out of range for `Word[8]^4`",
                "index 4 is out of range for `Word[8]^4`",
                "indices run from 0 through 3",
            ],
        },
        rules: &["S3E-INDEX-01", "S3E-DIAG-01", "S3E-DETERMINISM-01"],
    },
    Case {
        fixture: "invalid-updates.or",
        expectation: Expectation::Failure {
            codes: &[
                "ORC0224", "ORC0214", "ORC0223", "ORC0207", "ORC0222", "ORC0221", "ORC0214",
                "ORC0207",
            ],
            locations: &[
                "5:40", "6:41", "8:51", "10:58", "11:31", "12:34", "13:35", "14:39",
            ],
            messages: &[
                "only an array can be updated, but this has type `Word[8]`",
                "an update gives an array, but `Word[8]` is required here",
                "this index runs from 0 through 2, out of range for `Word[8]^2`",
                "literal is outside the range of `Word[8]`",
                "this array has 3 elements, but `Word[8]^4` has 4",
                "an array length must be a decimal integer from 1 through 65536",
                "an array literal cannot have type `Word[8]`",
            ],
        },
        rules: &[
            "S3E-UPDATE-01",
            "S3E-FILL-01",
            "S3E-DIAG-01",
            "S3E-DETERMINISM-01",
        ],
    },
];

/// Generated command-line cases in this file, each run twice.
const GENERATED_EVIDENCE: &[(&str, &[&str])] = &[(
    "s3e_loop_bounds_and_the_step_budget_are_exact",
    &["S3E-RES-BOUND-01", "S3E-DETERMINISM-01"],
)];

const UNIT_EVIDENCE: &[TestEvidence] = &[
    TestEvidence {
        source_path: "src/parser.rs",
        test: "builds_loops_updates_fills_and_expression_indices_with_exact_spans",
        rules: &["S3E-GRAMMAR-01", "S3E-RES-NEST-01"],
    },
    TestEvidence {
        source_path: "src/parser.rs",
        test: "loop_and_update_words_are_recognized_only_by_position",
        rules: &["S3E-GRAMMAR-01", "S3E-WORDS-01", "S3E-COMPAT-01"],
    },
    TestEvidence {
        source_path: "src/parser.rs",
        test: "rejects_malformed_loops_updates_and_fills_with_exact_messages",
        rules: &["S3E-GRAMMAR-01", "S3E-DIAG-01", "S3E-COMPAT-01"],
    },
    TestEvidence {
        source_path: "src/parser.rs",
        test: "bounds_expression_nesting_for_every_opener",
        rules: &["S3E-RES-NEST-01", "S3E-COMPAT-01"],
    },
    TestEvidence {
        source_path: "src/parser.rs",
        test: "expression_parsing_is_repeatable_and_malformed_expressions_never_panic",
        rules: &["S3E-GRAMMAR-01", "S3E-DETERMINISM-01"],
    },
    TestEvidence {
        source_path: "src/semantics/tests.rs",
        test: "loops_updates_fills_and_selections_build_typed_core_in_postorder",
        rules: &[
            "S3E-LOOP-01",
            "S3E-SCOPE-01",
            "S3E-INDEX-01",
            "S3E-UPDATE-01",
            "S3E-FILL-01",
            "S3E-CORE-01",
        ],
    },
    TestEvidence {
        source_path: "src/semantics/tests.rs",
        test: "index_ranges_follow_interval_arithmetic_over_loop_ranges",
        rules: &["S3E-INDEX-01", "S3E-DIAG-01"],
    },
    TestEvidence {
        source_path: "src/semantics/tests.rs",
        test: "loop_update_and_fill_errors_are_reported_once_in_checking_order",
        rules: &[
            "S3E-LOOP-01",
            "S3E-SCOPE-01",
            "S3E-INDEX-01",
            "S3E-UPDATE-01",
            "S3E-FILL-01",
            "S3E-DIAG-01",
            "S3E-RES-BOUND-01",
            "S3E-DETERMINISM-01",
        ],
    },
    TestEvidence {
        source_path: "src/semantics/tests.rs",
        test: "loop_events_and_core_nodes_follow_the_normative_accounting",
        rules: &["S3E-RES-EVENT-01"],
    },
    TestEvidence {
        source_path: "src/semantics/tests.rs",
        test: "loop_step_storage_failures_return_no_partial_core",
        rules: &["S3E-RES-FAIL-01"],
    },
    TestEvidence {
        source_path: "src/semantics/tests.rs",
        test: "rejects_foreign_spans_in_loops_updates_and_fills",
        rules: &["S3E-RES-FAIL-01"],
    },
    TestEvidence {
        source_path: "src/core.rs",
        test: "exact_integers_convert_to_i64_only_within_63_bits",
        rules: &["S3E-CORE-01", "S3E-RES-BOUND-01"],
    },
    TestEvidence {
        source_path: "src/eval.rs",
        test: "loops_updates_and_fills_evaluate_in_index_order",
        rules: &["S3E-EVAL-01", "S3E-RES-BOUND-01", "S3E-DETERMINISM-01"],
    },
    TestEvidence {
        source_path: "src/eval.rs",
        test: "steps_follow_the_normative_cost_table",
        rules: &["S3E-RES-STEP-01"],
    },
    TestEvidence {
        source_path: "src/eval.rs",
        test: "loop_frames_do_not_count_toward_the_call_depth",
        rules: &["S3E-RES-STEP-01"],
    },
    TestEvidence {
        source_path: "src/eval.rs",
        test: "deepest_accepted_sources_fit_in_one_mebibyte_of_stack",
        rules: &["S3E-RES-STACK-01"],
    },
    TestEvidence {
        source_path: "src/eval.rs",
        test: "inconsistent_loops_updates_and_fills_fail_closed",
        rules: &["S3E-CORE-01", "S3E-RES-FAIL-01"],
    },
    TestEvidence {
        source_path: "src/eval.rs",
        test: "loop_update_and_fill_reservation_failures_return_no_values",
        rules: &["S3E-RES-FAIL-01"],
    },
    TestEvidence {
        source_path: "src/diagnostic.rs",
        test: "diagnostic_code_inventory_is_exact_ordered_and_unique",
        rules: &["S3E-DIAG-01"],
    },
];

fn orangec() -> Command {
    Command::new(env!("CARGO_BIN_EXE_orangec"))
}

fn fixture_directory() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/s3e")
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
    LOOPS_SPECIFICATION
        .lines()
        .filter_map(|line| {
            let row = line.strip_prefix("| `")?;
            let (rule, _) = row.split_once("` |")?;
            if !rule.starts_with("S3E-") {
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
        _ => panic!("unknown S3e evidence layer {label:?}"),
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
        "src/diagnostic.rs" => DIAGNOSTIC_SOURCE,
        _ => panic!("unmapped S3e evidence source {source_path}"),
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
        S3E_CONFORMANCE_SOURCE.matches(&declaration).count(),
        1,
        "generated evidence {test} must be declared exactly once at file root"
    );
}

#[test]
fn s3e_rule_index_is_exact_and_covered() {
    let documented = documented_rules();
    let documented_ids: Vec<_> = documented.iter().map(|(rule, _)| *rule).collect();
    assert_eq!(
        documented_ids, RULES,
        "docs/LOOPS_2026.md rule index drifted from the S3e runner"
    );
    let known: BTreeSet<_> = RULES.iter().copied().collect();
    assert_eq!(known.len(), RULES.len(), "duplicate S3e rule ID");

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
fn s3e_cli_conformance_corpus_is_exact_and_repeatable() {
    let directory = fixture_directory();
    let mut observed: Vec<_> = fs::read_dir(&directory)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().into_string().unwrap())
        .collect();
    observed.sort();
    let mut expected: Vec<_> = CASES.iter().map(|case| case.fixture).collect();
    expected.sort_unstable();
    assert_eq!(observed, expected, "unexpected S3e fixture inventory");

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
fn s3e_loop_bounds_and_the_step_budget_are_exact() {
    let longest = module(
        "  spec longest() -> Word[32] { for i in 0..65536 with s: Word[32] = 0 { s + 1 } }\n",
    );
    let accepted = run_twice("eval", &longest, "65536 steps");
    assert_success(
        &accepted,
        "limits::longest: Word[32] = 0x00010000\n",
        "65536 steps",
    );

    let too_long = module("  spec too_long() -> Int { for i in 0..65537 with s: Int = 0 { s } }\n");
    let rejected = run_twice("eval", &too_long, "bound 65537");
    assert_failure(
        &rejected,
        &["ORC0225"],
        &["3:40"],
        &["a loop bound must be at most 65536"],
        "bound 65537",
    );

    // Two nested loops of 65536 steps each are admitted by analysis and
    // stopped by the evaluation step budget, with no partial value.
    let nested = module(concat!(
        "  spec nested() -> Int {\n",
        "    for i in 0..65536 with s: Int = 0 { for j in 0..65536 with t: Int = s { t } }\n",
        "  }\n",
    ));
    let checked = run_twice("check", &nested, "nested loops check");
    assert_success(&checked, "", "nested loops check");
    let stopped = run_twice("eval", &nested, "nested loops eval");
    assert_failure(
        &stopped,
        &["ORC0301"],
        &["3:8"],
        &["reference evaluation step limit exceeded"],
        "nested loops eval",
    );
}
