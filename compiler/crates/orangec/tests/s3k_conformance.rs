//! External conformance evidence for the proposed Orange 2026 S3k slice.
//!
//! The rule index lives in `docs/TUPLES_2026.md`. This runner requires that
//! index to agree exactly with the evidence map below, runs every fixture and
//! every generated program through the real `orangec` binary twice, and
//! checks that every named unit test is declared once in its source's test
//! module. S3j compatibility is also observed by the S2 through S3j runners,
//! which run their fixtures through the same binary.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const TUPLES_SPECIFICATION: &str = include_str!("../../../../docs/TUPLES_2026.md");
const S3K_CONFORMANCE_SOURCE: &str = include_str!("s3k_conformance.rs");
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

/// Every S3k rule, in the order of the specification's index.
const RULES: [&str; 8] = [
    "S3K-SYNTAX-01",
    "S3K-SCOPE-01",
    "S3K-TYPE-01",
    "S3K-CORE-01",
    "S3K-EVAL-01",
    "S3K-RES-01",
    "S3K-COMPAT-01",
    "S3K-DETERMINISM-01",
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
        fixture: "valid-sha256.or",
        expectation: Expectation::Success(concat!(
            "sha256::round_constants: Word[32]^64 = [0x428a2f98, 0x71374491, 0xb5c0fbcf, ",
            "0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5, 0xd807aa98, 0x12835b01, ",
            "0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174, 0xe49b69c1, ",
            "0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da, ",
            "0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, ",
            "0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, ",
            "0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, ",
            "0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, ",
            "0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3, 0x748f82ee, 0x78a5636f, 0x84c87814, ",
            "0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2]\n",
            "sha256::initial_hash: Word[32]^8 = [0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, ",
            "0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19]\n",
            "sha256::abc_block: Word[32]^16 = [0x61626380, 0x00000000, 0x00000000, 0x00000000, ",
            "0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, ",
            "0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000018]\n",
            "sha256::abc_digest: Word[32]^8 = [0xba7816bf, 0x8f01cfea, 0x414140de, 0x5dae2223, ",
            "0xb00361a3, 0x96177a9c, 0xb410ff61, 0xf20015ad]\n",
            "sha256::long_first_block: Word[32]^16 = [0x61626364, 0x62636465, 0x63646566, ",
            "0x64656667, 0x65666768, 0x66676869, 0x6768696a, 0x68696a6b, 0x696a6b6c, 0x6a6b6c6d, ",
            "0x6b6c6d6e, 0x6c6d6e6f, 0x6d6e6f70, 0x6e6f7071, 0x80000000, 0x00000000]\n",
            "sha256::long_second_block: Word[32]^16 = [0x00000000, 0x00000000, 0x00000000, ",
            "0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, ",
            "0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x000001c0]\n",
            "sha256::long_digest: Word[32]^8 = [0x248d6a61, 0xd20638b8, 0xe5c02693, 0x0c3e6039, ",
            "0xa33ce459, 0x64ff2167, 0xf6ecedd4, 0x19db06c1]\n",
        )),
        rules: &[
            "S3K-SYNTAX-01",
            "S3K-SCOPE-01",
            "S3K-TYPE-01",
            "S3K-CORE-01",
            "S3K-EVAL-01",
            "S3K-DETERMINISM-01",
        ],
    },
    Case {
        fixture: "valid-chacha20.or",
        expectation: Expectation::Success(concat!(
            "chacha20::quarter_round_vector: (Word[32], Word[32], Word[32], ",
            "Word[32]) = (0xea2a92f4, 0xcb1cf8ce, 0x4581472e, 0x5881c4bb)\n",
            "chacha20::test_key: Word[32]^8 = [0x03020100, 0x07060504, 0x0b0a0908, 0x0f0e0d0c, ",
            "0x13121110, 0x17161514, 0x1b1a1918, 0x1f1e1d1c]\n",
            "chacha20::test_nonce: Word[32]^3 = [0x09000000, 0x4a000000, 0x00000000]\n",
            "chacha20::block_vector: Word[32]^16 = [0xe4e7f110, 0x15593bd1, 0x1fdd0f50, ",
            "0xc47120a3, 0xc7f4d1c7, 0x0368c033, 0x9aaa2204, 0x4e6cd4c3, 0x466482d2, 0x09aa9f07, ",
            "0x05d7c214, 0xa2028bd9, 0xd19c12b5, 0xb94e16de, 0xe883d0cb, 0x4e3c50a2]\n",
        )),
        rules: &[
            "S3K-SYNTAX-01",
            "S3K-SCOPE-01",
            "S3K-TYPE-01",
            "S3K-CORE-01",
            "S3K-EVAL-01",
            "S3K-DETERMINISM-01",
        ],
    },
    Case {
        fixture: "valid-ascon.or",
        expectation: Expectation::Success(concat!(
            "ascon::round_constants: Word[64]^12 = [0x00000000000000f0, 0x00000000000000e1, ",
            "0x00000000000000d2, 0x00000000000000c3, 0x00000000000000b4, 0x00000000000000a5, ",
            "0x0000000000000096, 0x0000000000000087, 0x0000000000000078, 0x0000000000000069, ",
            "0x000000000000005a, 0x000000000000004b]\n",
            "ascon::iv: Word[64] = 0x0000080100cc0002\n",
            "ascon::kat_count_1: Word[8]^32 = [0x0b, 0x3b, 0xe5, 0x85, 0x0f, 0x2f, 0x6b, 0x98, ",
            "0xca, 0xf2, 0x9f, 0x8f, 0xde, 0xa8, 0x9b, 0x64, 0xa1, 0xfa, 0x70, 0xaa, 0x24, 0x9b, ",
            "0x8f, 0x83, 0x9b, 0xd5, 0x3b, 0xaa, 0x30, 0x4d, 0x92, 0xb2]\n",
            "ascon::kat_count_2: Word[8]^32 = [0x07, 0x28, 0x62, 0x10, 0x35, 0xaf, 0x3e, 0xd2, ",
            "0xbc, 0xa0, 0x3b, 0xf6, 0xfd, 0xe9, 0x00, 0xf9, 0x45, 0x6f, 0x53, 0x30, 0xe4, 0xb5, ",
            "0xee, 0x23, 0xe7, 0xf6, 0xa1, 0xe7, 0x02, 0x91, 0xbc, 0x80]\n",
            "ascon::kat_count_9: Word[8]^32 = [0xb8, 0x8e, 0x49, 0x7a, 0xe8, 0xe6, 0xfb, 0x64, ",
            "0x1b, 0x87, 0xef, 0x62, 0x2e, 0xb8, 0xf2, 0xfc, 0xa0, 0xed, 0x95, 0x38, 0x3f, 0x7f, ",
            "0xfe, 0xbe, 0x16, 0x7a, 0xcf, 0x10, 0x99, 0xba, 0x76, 0x4f]\n",
        )),
        rules: &[
            "S3K-SYNTAX-01",
            "S3K-SCOPE-01",
            "S3K-TYPE-01",
            "S3K-CORE-01",
            "S3K-EVAL-01",
            "S3K-DETERMINISM-01",
        ],
    },
    Case {
        fixture: "valid-tuples.or",
        expectation: Expectation::Success(concat!(
            "tuples::wraps: (Word[64]^4, Word[64]) = ([0x0000000000000000, 0x0000000000000000, ",
            "0x0000000000000000, 0x0000000000000000], 0x0000000000000001)\n",
            "tuples::top_limb: Word[64] = 0x0000000000000001\n",
            "tuples::fibonacci: Int = 2880067194370816120\n",
            "tuples::inverse: Int = 2753\n",
            "tuples::round_trip: Int^2 = [-5, 5]\n",
        )),
        rules: &[
            "S3K-SYNTAX-01",
            "S3K-SCOPE-01",
            "S3K-TYPE-01",
            "S3K-CORE-01",
            "S3K-EVAL-01",
            "S3K-DETERMINISM-01",
        ],
    },
    Case {
        fixture: "invalid-tuple-syntax.or",
        expectation: Expectation::Failure {
            codes: &[
                "ORC0101", "ORC0101", "ORC0101", "ORC0101", "ORC0101", "ORC0101", "ORC0101",
                "ORC0101", "ORC0101", "ORC0101",
            ],
            locations: &[
                "invalid-tuple-syntax.or:5:21",
                "invalid-tuple-syntax.or:6:19",
                "invalid-tuple-syntax.or:7:27",
                "invalid-tuple-syntax.or:8:36",
                "invalid-tuple-syntax.or:9:42",
                "invalid-tuple-syntax.or:10:41",
                "invalid-tuple-syntax.or:11:39",
                "invalid-tuple-syntax.or:12:46",
                "invalid-tuple-syntax.or:13:37",
                "invalid-tuple-syntax.or:14:60",
            ],
            messages: &[
                "expected `,` and another element type",
                "a tuple type is written `(T, U)` with two through 16 element types, each `Int`, \
                 `Bool`, a word, a residue, or an array of one",
                "expected an element type",
                "a tuple's elements are `Int`, `Bool`, words, residues, and arrays of them; a \
                 tuple holds no tuple",
                "expected the end of the type after the tuple",
                "arrays of tuples are not part of Orange 2026",
                "expected another element after `,`",
                "a tuple is written `(a, b)` with two through 16 elements; `(a)` without a comma \
                 is a group",
                "expected an element's position in decimal",
                "a tuple's element is selected by its position, counted from zero and written in \
                 decimal, as in `pair.0` or `pair.1`",
                "a tuple's elements are not tuples, so an element is selected once",
                "an array's elements are not tuples, so an element has no `.k`",
                "expected `:` and the type of the name",
                "a tuple pattern names two through 16 values, each with its type, as in `let (sum: \
                 Word[64], carry: Word[64]) = add(x, y, c);`",
                "expected `,` and another name in the pattern",
            ],
        },
        rules: &["S3K-SYNTAX-01", "S3K-COMPAT-01", "S3K-DETERMINISM-01"],
    },
    Case {
        fixture: "invalid-tuple-names.or",
        expectation: Expectation::Failure {
            codes: &[
                "ORC0219", "ORC0219", "ORC0219", "ORC0219", "ORC0211", "ORC0211",
            ],
            locations: &[
                "invalid-tuple-names.or:6:39",
                "invalid-tuple-names.or:7:40",
                "invalid-tuple-names.or:8:56",
                "invalid-tuple-names.or:9:65",
                "invalid-tuple-names.or:10:53",
                "invalid-tuple-names.or:11:102",
            ],
            messages: &[
                "duplicate binding `a`",
                "the first name is here",
                "the parameter is here",
                "duplicate binding `b`",
                "the first binding is here",
                "duplicate name `i`",
                "the loop index is here",
                "`a` is used before it is bound",
                "`a` is not a parameter or binding of `after`",
            ],
        },
        rules: &["S3K-SCOPE-01", "S3K-DETERMINISM-01"],
    },
    Case {
        fixture: "invalid-tuple-types.or",
        expectation: Expectation::Failure {
            codes: &[
                "ORC0203", "ORC0203", "ORC0214", "ORC0214", "ORC0214", "ORC0214", "ORC0234",
                "ORC0223", "ORC0214", "ORC0224", "ORC0215", "ORC0215", "ORC0215", "ORC0214",
                "ORC0203", "ORC0215", "ORC0215",
            ],
            locations: &[
                "invalid-tuple-types.or:9:19",
                "invalid-tuple-types.or:10:17",
                "invalid-tuple-types.or:11:26",
                "invalid-tuple-types.or:12:26",
                "invalid-tuple-types.or:13:32",
                "invalid-tuple-types.or:14:32",
                "invalid-tuple-types.or:15:37",
                "invalid-tuple-types.or:16:37",
                "invalid-tuple-types.or:17:36",
                "invalid-tuple-types.or:18:34",
                "invalid-tuple-types.or:19:44",
                "invalid-tuple-types.or:20:44",
                "invalid-tuple-types.or:21:38",
                "invalid-tuple-types.or:22:55",
                "invalid-tuple-types.or:23:38",
                "invalid-tuple-types.or:24:42",
                "invalid-tuple-types.or:25:49",
            ],
            messages: &[
                "`Pair` is a tuple type, so this is a tuple of tuples",
                "`Pair` is a tuple type, so this is an array of tuples",
                "this tuple has 3 elements, but `(Int, Int)` has 2",
                "a tuple cannot have type `Int`",
                "`true` has type `Bool`, but `Int` is required here",
                "`p` has type `(Int, Int)`, but `Int` is required here",
                "select one element by its position, such as `p.0`",
                "only a tuple has elements selected by position, but this has type `Int^2`",
                "`(Int, Int)` has no element 2",
                "this element has type `Int`, but `Bool` is required here",
                "only an array can be indexed, but this has type `(Int, Int)`",
                "`==` is not defined for `(Int, Int)`",
                "compare elements, such as `p.0 == q.0`",
                "`+` is not defined for `(Int, Int)`",
                "`as` is not defined for `(Int, Int)`",
                "an integer literal cannot have type `Bool`",
                "unsupported binding type `Wide`",
                "`==` is not defined for a tuple",
                "`!=` is not defined for an array",
            ],
        },
        rules: &["S3K-TYPE-01", "S3K-COMPAT-01", "S3K-DETERMINISM-01"],
    },
];

/// Generated command-line cases in this file, each run twice.
const GENERATED_EVIDENCE: &[(&str, &[&str])] = &[(
    "s3k_tuple_elements_are_exact_at_the_limit",
    &[
        "S3K-SYNTAX-01",
        "S3K-SCOPE-01",
        "S3K-EVAL-01",
        "S3K-RES-01",
        "S3K-DETERMINISM-01",
    ],
)];

const UNIT_EVIDENCE: &[TestEvidence] = &[
    TestEvidence {
        source_path: "src/parser.rs",
        test: "builds_tuples_projections_and_patterns_with_exact_spans",
        rules: &["S3K-SYNTAX-01", "S3K-COMPAT-01"],
    },
    TestEvidence {
        source_path: "src/parser.rs",
        test: "rejects_malformed_tuples_with_exact_messages",
        rules: &["S3K-SYNTAX-01"],
    },
    TestEvidence {
        source_path: "src/parser.rs",
        test: "bounds_tuple_elements",
        rules: &["S3K-SYNTAX-01", "S3K-RES-01"],
    },
    TestEvidence {
        source_path: "src/parser.rs",
        test: "tuples_and_projections_count_toward_expression_height",
        rules: &["S3K-SYNTAX-01", "S3K-RES-01"],
    },
    TestEvidence {
        source_path: "src/parser.rs",
        test: "tuple_reservation_failures_return_no_partial_ast",
        rules: &["S3K-RES-01"],
    },
    TestEvidence {
        source_path: "src/semantics/tests.rs",
        test: "tuples_build_typed_core_with_elements_in_order",
        rules: &["S3K-TYPE-01", "S3K-CORE-01"],
    },
    TestEvidence {
        source_path: "src/semantics/tests.rs",
        test: "tuple_pattern_names_are_unique_and_scoped_like_bindings",
        rules: &["S3K-SCOPE-01", "S3K-DETERMINISM-01"],
    },
    TestEvidence {
        source_path: "src/semantics/tests.rs",
        test: "tuple_types_and_selections_are_checked_once_in_order",
        rules: &["S3K-TYPE-01", "S3K-COMPAT-01", "S3K-DETERMINISM-01"],
    },
    TestEvidence {
        source_path: "src/semantics/tests.rs",
        test: "unresolved_pattern_types_are_reported_once_without_cascades",
        rules: &["S3K-TYPE-01"],
    },
    TestEvidence {
        source_path: "src/semantics/tests.rs",
        test: "tuple_events_and_core_nodes_follow_the_normative_accounting",
        rules: &["S3K-RES-01"],
    },
    TestEvidence {
        source_path: "src/semantics/tests.rs",
        test: "rejects_foreign_spans_in_tuples",
        rules: &["S3K-RES-01"],
    },
    TestEvidence {
        source_path: "src/core.rs",
        test: "tuple_types_hold_two_through_sixteen_flat_elements",
        rules: &["S3K-CORE-01", "S3K-RES-01"],
    },
    TestEvidence {
        source_path: "src/core.rs",
        test: "tuple_values_hold_one_value_of_each_element_type",
        rules: &["S3K-CORE-01"],
    },
    TestEvidence {
        source_path: "src/eval.rs",
        test: "tuples_hold_their_elements_in_order_and_select_them_by_position",
        rules: &["S3K-EVAL-01"],
    },
    TestEvidence {
        source_path: "src/eval.rs",
        test: "steps_follow_the_normative_cost_table",
        rules: &["S3K-EVAL-01"],
    },
    TestEvidence {
        source_path: "src/eval.rs",
        test: "tuple_storage_reservation_failures_return_no_values",
        rules: &["S3K-RES-01"],
    },
    TestEvidence {
        source_path: "src/eval.rs",
        test: "inconsistent_tuples_fail_closed",
        rules: &["S3K-CORE-01", "S3K-RES-01"],
    },
    TestEvidence {
        source_path: "src/eval.rs",
        test: "deepest_accepted_sources_fit_in_one_mebibyte_of_stack",
        rules: &["S3K-RES-01"],
    },
    TestEvidence {
        source_path: "src/diagnostic.rs",
        test: "diagnostic_code_inventory_is_exact_ordered_and_unique",
        rules: &["S3K-TYPE-01"],
    },
];

fn orangec() -> Command {
    Command::new(env!("CARGO_BIN_EXE_orangec"))
}

fn fixture_directory() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/s3k")
}

fn run(command: &str, path: &Path) -> Output {
    orangec().arg(command).arg(path).output().unwrap()
}

/// Runs `command` twice on `path` and returns the first output after
/// requiring the second to be byte-identical.
fn run_twice(command: &str, path: &Path, context: &str) -> Output {
    let first = run(command, path);
    let second = run(command, path);
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

/// A fresh directory for one generated program.
fn scratch_directory(name: &str) -> PathBuf {
    let directory = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("orangec-s3k-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&directory);
    fs::create_dir(&directory).unwrap();
    directory
}

/// The rule index rows of the specification: rule ID and evidence label.
fn documented_rules() -> Vec<(&'static str, &'static str)> {
    TUPLES_SPECIFICATION
        .lines()
        .filter_map(|line| {
            let row = line.strip_prefix("| `")?;
            let (rule, _) = row.split_once("` |")?;
            if !rule.starts_with("S3K-") {
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
        "Unit" => UNIT,
        _ => panic!("unknown S3k evidence layer {label:?}"),
    }
}

fn unit_source(source_path: &str) -> &'static str {
    match source_path {
        "src/parser.rs" => PARSER_SOURCE,
        "src/semantics.rs" => SEMANTICS_SOURCE,
        "src/semantics/tests.rs" => SEMANTICS_TESTS_SOURCE,
        "src/core.rs" => CORE_SOURCE,
        "src/eval.rs" => EVAL_SOURCE,
        "src/diagnostic.rs" => DIAGNOSTIC_SOURCE,
        _ => panic!("unmapped S3k evidence source {source_path}"),
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
        S3K_CONFORMANCE_SOURCE.matches(&declaration).count(),
        1,
        "generated evidence {test} must be declared exactly once at file root"
    );
}

#[test]
fn s3k_rule_index_is_exact_and_covered() {
    let documented = documented_rules();
    let documented_ids: Vec<_> = documented.iter().map(|(rule, _)| *rule).collect();
    assert_eq!(
        documented_ids, RULES,
        "docs/TUPLES_2026.md rule index drifted from the S3k runner"
    );
    let known: BTreeSet<_> = RULES.iter().copied().collect();
    assert_eq!(known.len(), RULES.len(), "duplicate S3k rule ID");

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
fn s3k_cli_conformance_corpus_is_exact_and_repeatable() {
    let directory = fixture_directory();
    let mut observed: Vec<_> = fs::read_dir(&directory)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().into_string().unwrap())
        .collect();
    observed.sort();
    let mut expected: Vec<_> = CASES.iter().map(|case| case.fixture).collect();
    expected.sort_unstable();
    assert_eq!(observed, expected, "unexpected S3k fixture inventory");

    for case in CASES {
        let path = directory.join(case.fixture);
        let check = format!("{} check", case.fixture);
        let eval = format!("{} eval", case.fixture);
        let first_check = run_twice("check", &path, &check);
        let first_eval = run_twice("eval", &path, &eval);

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
fn s3k_tuple_elements_are_exact_at_the_limit() {
    let directory = scratch_directory("limits");

    // A tuple type, a tuple, and a tuple pattern may each have 16 parts,
    // one on each line with a trailing comma; a 17th is a parser resource
    // limit at that part.
    let program = |types: usize, elements: usize, names: usize| {
        let mut source = String::from("edition 2026;\nmodule limits {\n  spec wide() -> (\n");
        for _ in 0..types {
            source.push_str("    Int,\n");
        }
        source.push_str("  ) {\n    (\n");
        for element in 0..elements {
            source.push_str(&format!("      {element},\n"));
        }
        source.push_str("    )\n  }\n  spec last() -> Int {\n    let (\n");
        for name in 0..names {
            source.push_str(&format!("      v{name}: Int,\n"));
        }
        source.push_str(&format!("    ) = wide();\n    v{}\n  }}\n}}\n", names - 1));
        source
    };
    let path = directory.join("limits.or");
    fs::write(&path, program(16, 16, 16)).unwrap();
    let most = run_twice("eval", &path, "16 parts");
    assert_success(
        &most,
        concat!(
            "limits::wide: (Int, Int, Int, Int, Int, Int, Int, Int, Int, Int, Int, Int, Int, ",
            "Int, Int, Int) = (0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15)\n",
            "limits::last: Int = 15\n",
        ),
        "16 parts",
    );
    for (types, elements, names, location, message) in [
        (
            17,
            17,
            17,
            "limits.or:20:5",
            "a tuple type has more than 16 elements",
        ),
        (
            16,
            17,
            17,
            "limits.or:38:7",
            "a tuple has more than 16 elements",
        ),
        (
            16,
            16,
            17,
            "limits.or:58:7",
            "a tuple pattern names more than 16 values",
        ),
    ] {
        fs::write(&path, program(types, elements, names)).unwrap();
        let context = format!("{types}, {elements}, and {names} parts");
        let over = run_twice("check", &path, &context);
        assert_failure(&over, &["ORC0106"], &[location], &[message], &context);
    }
    fs::remove_dir_all(&directory).unwrap();
}
