//! External conformance evidence for the proposed Orange 2026 S3l slice.
//!
//! The rule index lives in `docs/BYTES_2026.md`. This runner requires that
//! index to agree exactly with the evidence map below, runs every fixture and
//! every generated program through the real `orangec` binary twice, and
//! checks that every named unit test is declared once in its source's test
//! module. S3k compatibility is also observed by the S2 through S3k runners,
//! which run their fixtures through the same binary.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const BYTES_SPECIFICATION: &str = include_str!("../../../../docs/BYTES_2026.md");
const S3L_CONFORMANCE_SOURCE: &str = include_str!("s3l_conformance.rs");
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

/// Every S3l rule, in the order of the specification's index.
const RULES: [&str; 10] = [
    "S3L-LEX-01",
    "S3L-SYNTAX-01",
    "S3L-BYTES-01",
    "S3L-TYPE-01",
    "S3L-STATIC-01",
    "S3L-CORE-01",
    "S3L-EVAL-01",
    "S3L-RES-01",
    "S3L-COMPAT-01",
    "S3L-DETERMINISM-01",
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

/// The rules a program that runs to its published values gives evidence of.
const VALID_RULES: &[&str] = &[
    "S3L-LEX-01",
    "S3L-SYNTAX-01",
    "S3L-BYTES-01",
    "S3L-TYPE-01",
    "S3L-STATIC-01",
    "S3L-CORE-01",
    "S3L-EVAL-01",
    "S3L-DETERMINISM-01",
];

const CASES: [Case; 6] = [
    Case {
        fixture: "valid-bytes.or",
        expectation: Expectation::Success(concat!(
            "bytes::greeting: Word[8]^8 = [0x48, 0x69, 0x20, 0x54, 0x68, 0x65, 0x72, 0x65]\n",
            "bytes::escapes: Word[8]^7 = [0x22, 0x5c, 0x0a, 0x0d, 0x09, 0x00, 0x7f]\n",
            "bytes::nonce: Word[8]^12 = [0x07, 0x00, 0x00, 0x00, 0x40, 0x41, 0x42, 0x43, 0x44, ",
            "0x45, 0x46, 0x47]\n",
            "bytes::mixed_case: Word[8]^4 = [0xde, 0xad, 0xbe, 0xef]\n",
            "bytes::padded: Word[8]^16 = [0x61, 0x62, 0x63, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, ",
            "0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x18]\n",
            "bytes::joined_words: Word[32]^5 = [0x00000001, 0x00000002, 0x00000003, 0x00000004, ",
            "0x00000005]\n",
            "bytes::joined_flags: Bool^3 = [true, false, true]\n",
            "bytes::constant: Word[8]^4 = [0x07, 0x00, 0x00, 0x00]\n",
            "bytes::iv: Word[8]^8 = [0x40, 0x41, 0x42, 0x43, 0x44, 0x45, 0x46, 0x47]\n",
            "bytes::middle: Word[8]^2 = [0x54, 0x68]\n",
            "bytes::rotated: Word[8]^8 = [0x77, 0x78, 0x79, 0x7a, 0x41, 0x42, 0x43, 0x44]\n",
            "bytes::key_words: Word[32]^4 = [0x00010203, 0x04050607, 0x08090a0b, 0x0c0d0e0f]\n",
            "bytes::reversed_iv: Word[8]^8 = [0x47, 0x46, 0x45, 0x44, 0x43, 0x42, 0x41, 0x40]\n",
            "bytes::spliced: Word[8]^8 = [0x48, 0x69, 0x20, 0x54, 0x77, 0x78, 0x79, 0x7a]\n",
            "bytes::overwritten: Word[8]^8 = [0x48, 0x69, 0x20, 0x65, 0x21, 0x21, 0x21, 0x21]\n",
        )),
        rules: VALID_RULES,
    },
    Case {
        fixture: "valid-hmac.or",
        expectation: Expectation::Success(concat!(
            "hmac::round_constants: Word[32]^64 = [0x428a2f98, 0x71374491, 0xb5c0fbcf, ",
            "0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5, 0xd807aa98, 0x12835b01, ",
            "0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174, 0xe49b69c1, ",
            "0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da, ",
            "0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, ",
            "0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, ",
            "0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, ",
            "0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, ",
            "0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3, 0x748f82ee, 0x78a5636f, 0x84c87814, ",
            "0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2]\n",
            "hmac::initial_hash: Word[32]^8 = [0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, ",
            "0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19]\n",
            "hmac::abc: Word[8]^32 = [0xba, 0x78, 0x16, 0xbf, 0x8f, 0x01, 0xcf, 0xea, 0x41, ",
            "0x41, 0x40, 0xde, 0x5d, 0xae, 0x22, 0x23, 0xb0, 0x03, 0x61, 0xa3, 0x96, 0x17, 0x7a, ",
            "0x9c, 0xb4, 0x10, 0xff, 0x61, 0xf2, 0x00, 0x15, 0xad]\n",
            "hmac::case1: Word[8]^32 = [0xb0, 0x34, 0x4c, 0x61, 0xd8, 0xdb, 0x38, 0x53, 0x5c, ",
            "0xa8, 0xaf, 0xce, 0xaf, 0x0b, 0xf1, 0x2b, 0x88, 0x1d, 0xc2, 0x00, 0xc9, 0x83, 0x3d, ",
            "0xa7, 0x26, 0xe9, 0x37, 0x6c, 0x2e, 0x32, 0xcf, 0xf7]\n",
            "hmac::case2: Word[8]^32 = [0x5b, 0xdc, 0xc1, 0x46, 0xbf, 0x60, 0x75, 0x4e, 0x6a, ",
            "0x04, 0x24, 0x26, 0x08, 0x95, 0x75, 0xc7, 0x5a, 0x00, 0x3f, 0x08, 0x9d, 0x27, 0x39, ",
            "0x83, 0x9d, 0xec, 0x58, 0xb9, 0x64, 0xec, 0x38, 0x43]\n",
        )),
        rules: VALID_RULES,
    },
    Case {
        fixture: "valid-aead.or",
        expectation: Expectation::Success(concat!(
            "aead::key: Word[8]^32 = [0x80, 0x81, 0x82, 0x83, 0x84, 0x85, 0x86, 0x87, 0x88, ",
            "0x89, 0x8a, 0x8b, 0x8c, 0x8d, 0x8e, 0x8f, 0x90, 0x91, 0x92, 0x93, 0x94, 0x95, 0x96, ",
            "0x97, 0x98, 0x99, 0x9a, 0x9b, 0x9c, 0x9d, 0x9e, 0x9f]\n",
            "aead::nonce: Word[8]^12 = [0x07, 0x00, 0x00, 0x00, 0x40, 0x41, 0x42, 0x43, 0x44, ",
            "0x45, 0x46, 0x47]\n",
            "aead::aad: Word[8]^12 = [0x50, 0x51, 0x52, 0x53, 0xc0, 0xc1, 0xc2, 0xc3, 0xc4, ",
            "0xc5, 0xc6, 0xc7]\n",
            "aead::sunscreen: Word[8]^114 = [0x4c, 0x61, 0x64, 0x69, 0x65, 0x73, 0x20, 0x61, ",
            "0x6e, 0x64, 0x20, 0x47, 0x65, 0x6e, 0x74, 0x6c, 0x65, 0x6d, 0x65, 0x6e, 0x20, 0x6f, ",
            "0x66, 0x20, 0x74, 0x68, 0x65, 0x20, 0x63, 0x6c, 0x61, 0x73, 0x73, 0x20, 0x6f, 0x66, ",
            "0x20, 0x27, 0x39, 0x39, 0x3a, 0x20, 0x49, 0x66, 0x20, 0x49, 0x20, 0x63, 0x6f, 0x75, ",
            "0x6c, 0x64, 0x20, 0x6f, 0x66, 0x66, 0x65, 0x72, 0x20, 0x79, 0x6f, 0x75, 0x20, 0x6f, ",
            "0x6e, 0x6c, 0x79, 0x20, 0x6f, 0x6e, 0x65, 0x20, 0x74, 0x69, 0x70, 0x20, 0x66, 0x6f, ",
            "0x72, 0x20, 0x74, 0x68, 0x65, 0x20, 0x66, 0x75, 0x74, 0x75, 0x72, 0x65, 0x2c, 0x20, ",
            "0x73, 0x75, 0x6e, 0x73, 0x63, 0x72, 0x65, 0x65, 0x6e, 0x20, 0x77, 0x6f, 0x75, 0x6c, ",
            "0x64, 0x20, 0x62, 0x65, 0x20, 0x69, 0x74, 0x2e]\n",
            "aead::sealed: Word[8]^130 = [0xd3, 0x1a, 0x8d, 0x34, 0x64, 0x8e, 0x60, 0xdb, 0x7b, ",
            "0x86, 0xaf, 0xbc, 0x53, 0xef, 0x7e, 0xc2, 0xa4, 0xad, 0xed, 0x51, 0x29, 0x6e, 0x08, ",
            "0xfe, 0xa9, 0xe2, 0xb5, 0xa7, 0x36, 0xee, 0x62, 0xd6, 0x3d, 0xbe, 0xa4, 0x5e, 0x8c, ",
            "0xa9, 0x67, 0x12, 0x82, 0xfa, 0xfb, 0x69, 0xda, 0x92, 0x72, 0x8b, 0x1a, 0x71, 0xde, ",
            "0x0a, 0x9e, 0x06, 0x0b, 0x29, 0x05, 0xd6, 0xa5, 0xb6, 0x7e, 0xcd, 0x3b, 0x36, 0x92, ",
            "0xdd, 0xbd, 0x7f, 0x2d, 0x77, 0x8b, 0x8c, 0x98, 0x03, 0xae, 0xe3, 0x28, 0x09, 0x1b, ",
            "0x58, 0xfa, 0xb3, 0x24, 0xe4, 0xfa, 0xd6, 0x75, 0x94, 0x55, 0x85, 0x80, 0x8b, 0x48, ",
            "0x31, 0xd7, 0xbc, 0x3f, 0xf4, 0xde, 0xf0, 0x8e, 0x4b, 0x7a, 0x9d, 0xe5, 0x76, 0xd2, ",
            "0x65, 0x86, 0xce, 0xc6, 0x4b, 0x61, 0x16, 0x1a, 0xe1, 0x0b, 0x59, 0x4f, 0x09, 0xe2, ",
            "0x6a, 0x7e, 0x90, 0x2e, 0xcb, 0xd0, 0x60, 0x06, 0x91]\n",
            "aead::verified: Bool = true\n",
        )),
        rules: VALID_RULES,
    },
    Case {
        fixture: "invalid-bytes-lexical.or",
        expectation: Expectation::Failure {
            codes: &[
                "ORC0009", "ORC0009", "ORC0009", "ORC0009", "ORC0009", "ORC0009", "ORC0003",
            ],
            locations: &[
                "invalid-bytes-lexical.or:7:40",
                "invalid-bytes-lexical.or:8:39",
                "invalid-bytes-lexical.or:9:42",
                "invalid-bytes-lexical.or:10:38",
                "invalid-bytes-lexical.or:11:35",
                "invalid-bytes-lexical.or:12:37",
                "invalid-bytes-lexical.or:13:38",
            ],
            messages: &[
                "'g' cannot appear in a hex string",
                "not a hex digit or a space",
                "hex digit '2' has no partner",
                "a byte is written as two hex digits",
                "a hex string has no escapes",
                "a hex string holds bytes written as pairs of hex digits, as in `hex\"00 1f a0\"`; \
                 spaces may separate bytes but not split one",
                "unterminated hex string",
            ],
        },
        rules: &["S3L-LEX-01", "S3L-COMPAT-01", "S3L-DETERMINISM-01"],
    },
    Case {
        fixture: "invalid-bytes-syntax.or",
        expectation: Expectation::Failure {
            codes: &[
                "ORC0101", "ORC0101", "ORC0101", "ORC0101", "ORC0108", "ORC0108", "ORC0101",
                "ORC0101",
            ],
            locations: &[
                "invalid-bytes-syntax.or:5:36",
                "invalid-bytes-syntax.or:6:47",
                "invalid-bytes-syntax.or:7:50",
                "invalid-bytes-syntax.or:8:38",
                "invalid-bytes-syntax.or:9:67",
                "invalid-bytes-syntax.or:10:63",
                "invalid-bytes-syntax.or:11:52",
                "invalid-bytes-syntax.or:12:51",
            ],
            messages: &[
                "a hex string's quote follows `hex` directly, with no space, as in `hex\"00 1f a0\"`",
                "expected a bound of the slice after `..`",
                "a slice is taken once, from a name, a call, or a tuple's element; bind it with \
                 `let` to select from it",
                "a byte string is not indexed or sliced where it is written; bind it with `let` to \
                 select from it",
                "`+` follows `++` without grouping parentheses",
                "`++` follows `+` without grouping parentheses",
                "a slice update is written `x with [a..b] = values`",
                "expected `]` after the slice",
            ],
        },
        rules: &["S3L-SYNTAX-01", "S3L-COMPAT-01", "S3L-DETERMINISM-01"],
    },
    Case {
        fixture: "invalid-bytes-types.or",
        expectation: Expectation::Failure {
            codes: &[
                "ORC0222", "ORC0214", "ORC0235", "ORC0235", "ORC0221", "ORC0222", "ORC0214",
                "ORC0224", "ORC0214", "ORC0223", "ORC0236", "ORC0236", "ORC0226", "ORC0236",
                "ORC0226", "ORC0223", "ORC0222", "ORC0214", "ORC0224", "ORC0222", "ORC0224",
                "ORC0222",
            ],
            locations: &[
                "invalid-bytes-types.or:8:31",
                "invalid-bytes-types.or:9:32",
                "invalid-bytes-types.or:10:35",
                "invalid-bytes-types.or:11:34",
                "invalid-bytes-types.or:12:31",
                "invalid-bytes-types.or:13:38",
                "invalid-bytes-types.or:14:36",
                "invalid-bytes-types.or:15:41",
                "invalid-bytes-types.or:16:55",
                "invalid-bytes-types.or:17:46",
                "invalid-bytes-types.or:18:45",
                "invalid-bytes-types.or:19:48",
                "invalid-bytes-types.or:20:52",
                "invalid-bytes-types.or:22:51",
                "invalid-bytes-types.or:25:52",
                "invalid-bytes-types.or:28:51",
                "invalid-bytes-types.or:30:44",
                "invalid-bytes-types.or:31:46",
                "invalid-bytes-types.or:32:40",
                "invalid-bytes-types.or:33:59",
                "invalid-bytes-types.or:34:40",
                "invalid-bytes-types.or:35:42",
            ],
            messages: &[
                "this byte string holds 3 bytes, but `Word[8]^4` has 4",
                "this byte string has type `Word[8]^3`, but `Word[32]^3` is required here",
                "U+00E9 is not a printable ASCII character",
                "its UTF-8 bytes are written `hex\"c3 a9\"`",
                "its UTF-8 bytes are written `hex\"c2 b7\"`",
                "a byte string holds at least one byte",
                "`++` joins 3 and 2 elements, 5 in all, but `Word[8]^8` has 8",
                "`++` joins arrays, but `Word[32]` is required here",
                "only arrays can be joined, but this has type `Word[32]`",
                "`x` has type `Word[32]^2`, but `Word[8]^2` is required here",
                "this slice reaches elements 6 through 9, out of range for `Word[8]^8`",
                "this slice is empty: its bounds are equal",
                "this slice ends 3 elements before it starts",
                "a slice's bounds may use only integer literals and loop indices",
                "the length of this slice changes from step to step",
                "a slice's bound may multiply a loop index only by a constant",
                "this slice reaches elements 1 through 8, out of range for `Word[8]^8`",
                "this slice has 4 elements, but `Word[8]^3` has 3",
                "this slice is an array of `Word[32]`, but `Word[8]^4` is required here",
                "only an array can be sliced, but this has type `(Int, Int)`",
                "this byte string holds 3 bytes, but `Word[8]^2` has 2",
                "only an array can be updated, but this has type `Word[32]`",
                "`++` joins 200 and 100 elements, 300 in all, but `Word[8]^256` has 256",
            ],
        },
        rules: &[
            "S3L-BYTES-01",
            "S3L-TYPE-01",
            "S3L-STATIC-01",
            "S3L-DETERMINISM-01",
        ],
    },
];

/// Generated command-line cases in this file, each run twice.
const GENERATED_EVIDENCE: &[(&str, &[&str])] = &[
    (
        "s3l_byte_strings_are_exact_at_the_limit",
        &[
            "S3L-LEX-01",
            "S3L-BYTES-01",
            "S3L-EVAL-01",
            "S3L-RES-01",
            "S3L-DETERMINISM-01",
        ],
    ),
    (
        "s3l_control_characters_in_byte_strings_are_named_by_their_byte",
        &["S3L-BYTES-01", "S3L-DETERMINISM-01"],
    ),
];

const UNIT_EVIDENCE: &[TestEvidence] = &[
    TestEvidence {
        source_path: "src/lexer.rs",
        test: "lexes_hex_strings_and_concatenation_by_longest_match",
        rules: &["S3L-LEX-01", "S3L-COMPAT-01"],
    },
    TestEvidence {
        source_path: "src/lexer.rs",
        test: "rejects_malformed_hex_strings_at_the_first_offense",
        rules: &["S3L-LEX-01", "S3L-DETERMINISM-01"],
    },
    TestEvidence {
        source_path: "src/lexer.rs",
        test: "unterminated_hex_string_is_reported_at_its_opening_alone",
        rules: &["S3L-LEX-01"],
    },
    TestEvidence {
        source_path: "src/lexer.rs",
        test: "reserved_and_punctuation_spellings_are_exact",
        rules: &["S3L-LEX-01", "S3L-COMPAT-01"],
    },
    TestEvidence {
        source_path: "src/parser.rs",
        test: "builds_byte_strings_joins_and_slices_with_exact_spans",
        rules: &["S3L-SYNTAX-01", "S3L-COMPAT-01"],
    },
    TestEvidence {
        source_path: "src/parser.rs",
        test: "rejects_malformed_bytes_and_slices_with_exact_messages",
        rules: &["S3L-SYNTAX-01"],
    },
    TestEvidence {
        source_path: "src/parser.rs",
        test: "slices_and_joins_count_toward_expression_height",
        rules: &["S3L-SYNTAX-01", "S3L-RES-01"],
    },
    TestEvidence {
        source_path: "src/semantics/tests.rs",
        test: "byte_strings_joins_and_slices_build_typed_core_in_postorder",
        rules: &[
            "S3L-BYTES-01",
            "S3L-TYPE-01",
            "S3L-STATIC-01",
            "S3L-CORE-01",
        ],
    },
    TestEvidence {
        source_path: "src/semantics/tests.rs",
        test: "byte_strings_hold_one_through_256_printable_bytes",
        rules: &["S3L-BYTES-01", "S3L-RES-01", "S3L-DETERMINISM-01"],
    },
    TestEvidence {
        source_path: "src/semantics/tests.rs",
        test: "slice_bounds_are_static_in_range_and_a_fixed_length_apart",
        rules: &["S3L-STATIC-01", "S3L-DETERMINISM-01"],
    },
    TestEvidence {
        source_path: "src/semantics/tests.rs",
        test: "slice_bounds_are_held_to_the_significant_bit_limit_of_int",
        rules: &["S3L-STATIC-01", "S3L-RES-01"],
    },
    TestEvidence {
        source_path: "src/semantics/tests.rs",
        test: "joins_are_checked_once_in_order",
        rules: &["S3L-TYPE-01", "S3L-DETERMINISM-01"],
    },
    TestEvidence {
        source_path: "src/semantics/tests.rs",
        test: "a_conditional_joined_takes_its_length_from_a_branch_without_bindings",
        rules: &["S3L-TYPE-01"],
    },
    TestEvidence {
        source_path: "src/semantics/tests.rs",
        test: "slices_and_slice_updates_are_typed_against_the_required_array",
        rules: &["S3L-TYPE-01"],
    },
    TestEvidence {
        source_path: "src/semantics/tests.rs",
        test: "byte_string_and_slice_events_and_core_nodes_follow_the_normative_accounting",
        rules: &["S3L-RES-01"],
    },
    TestEvidence {
        source_path: "src/semantics/tests.rs",
        test: "slice_bound_storage_failures_return_no_partial_core",
        rules: &["S3L-RES-01"],
    },
    TestEvidence {
        source_path: "src/semantics/tests.rs",
        test: "rejects_foreign_spans_in_byte_strings_joins_and_slices",
        rules: &["S3L-RES-01"],
    },
    TestEvidence {
        source_path: "src/eval.rs",
        test: "byte_strings_joins_and_slices_evaluate_in_index_order",
        rules: &["S3L-CORE-01", "S3L-EVAL-01"],
    },
    TestEvidence {
        source_path: "src/eval.rs",
        test: "steps_follow_the_normative_cost_table",
        rules: &["S3L-EVAL-01", "S3L-COMPAT-01"],
    },
    TestEvidence {
        source_path: "src/eval.rs",
        test: "join_and_slice_reservation_failures_return_no_values",
        rules: &["S3L-RES-01"],
    },
    TestEvidence {
        source_path: "src/eval.rs",
        test: "inconsistent_joins_and_slices_fail_closed",
        rules: &["S3L-CORE-01", "S3L-RES-01"],
    },
    TestEvidence {
        source_path: "src/eval.rs",
        test: "deepest_accepted_sources_fit_in_one_mebibyte_of_stack",
        rules: &["S3L-RES-01"],
    },
    TestEvidence {
        source_path: "src/eval.rs",
        test: "long_rejected_joins_fit_in_one_mebibyte_of_stack",
        rules: &["S3L-RES-01"],
    },
    TestEvidence {
        source_path: "src/diagnostic.rs",
        test: "diagnostic_code_inventory_is_exact_ordered_and_unique",
        rules: &["S3L-LEX-01", "S3L-BYTES-01", "S3L-STATIC-01"],
    },
];

fn orangec() -> Command {
    Command::new(env!("CARGO_BIN_EXE_orangec"))
}

fn fixture_directory() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/s3l")
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
        .join(format!("orangec-s3l-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&directory);
    fs::create_dir(&directory).unwrap();
    directory
}

/// The rule index rows of the specification: rule ID and evidence label.
fn documented_rules() -> Vec<(&'static str, &'static str)> {
    BYTES_SPECIFICATION
        .lines()
        .filter_map(|line| {
            let row = line.strip_prefix("| `")?;
            let (rule, _) = row.split_once("` |")?;
            if !rule.starts_with("S3L-") {
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
        _ => panic!("unknown S3l evidence layer {label:?}"),
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
        _ => panic!("unmapped S3l evidence source {source_path}"),
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
        S3L_CONFORMANCE_SOURCE.matches(&declaration).count(),
        1,
        "generated evidence {test} must be declared exactly once at file root"
    );
}

#[test]
fn s3l_rule_index_is_exact_and_covered() {
    let documented = documented_rules();
    let documented_ids: Vec<_> = documented.iter().map(|(rule, _)| *rule).collect();
    assert_eq!(
        documented_ids, RULES,
        "docs/BYTES_2026.md rule index drifted from the S3l runner"
    );
    let known: BTreeSet<_> = RULES.iter().copied().collect();
    assert_eq!(known.len(), RULES.len(), "duplicate S3l rule ID");

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
fn s3l_cli_conformance_corpus_is_exact_and_repeatable() {
    let directory = fixture_directory();
    let mut observed: Vec<_> = fs::read_dir(&directory)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().into_string().unwrap())
        .collect();
    observed.sort();
    let mut expected: Vec<_> = CASES.iter().map(|case| case.fixture).collect();
    expected.sort_unstable();
    assert_eq!(observed, expected, "unexpected S3l fixture inventory");

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
fn s3l_byte_strings_are_exact_at_the_limit() {
    let directory = scratch_directory("limits");

    // A byte string holds 1 through 256 bytes, written as characters or as
    // hex digit pairs; a 257th byte is reported at the string, before any
    // later character is read.
    let program = |text: usize, hex: usize| {
        let mut source =
            String::from("edition 2026;\nmodule limits {\n  spec text() -> Word[8]^256 {\n    \"");
        source.push_str(&"~".repeat(text));
        source.push_str("\"\n  }\n  spec hex() -> Word[8]^256 {\n    hex\"");
        for byte in 0..hex {
            write!(source, "{:02x} ", byte % 256).unwrap();
        }
        source.push_str("\"\n  }\n}\n");
        source
    };
    let path = directory.join("limits.or");
    fs::write(&path, program(256, 256)).unwrap();
    let most = run_twice("eval", &path, "256 bytes");
    let mut expected = String::from("limits::text: Word[8]^256 = [");
    expected.push_str(&vec!["0x7e"; 256].join(", "));
    expected.push_str("]\nlimits::hex: Word[8]^256 = [");
    let bytes: Vec<_> = (0..256).map(|byte| format!("0x{byte:02x}")).collect();
    expected.push_str(&bytes.join(", "));
    expected.push_str("]\n");
    assert_success(&most, &expected, "256 bytes");
    for (text, hex, location) in [(257, 256, "limits.or:4:5"), (256, 257, "limits.or:7:5")] {
        fs::write(&path, program(text, hex)).unwrap();
        let context = format!("{text} and {hex} bytes");
        let over = run_twice("check", &path, &context);
        assert_failure(
            &over,
            &["ORC0221"],
            &[location],
            &["a byte string holds at most 256 bytes"],
            &context,
        );
    }
    fs::remove_dir_all(&directory).unwrap();
}

#[test]
fn s3l_control_characters_in_byte_strings_are_named_by_their_byte() {
    let directory = scratch_directory("controls");

    // A tab or a delete written into a byte string is not printable ASCII;
    // each is reported where it stands, with the hex byte to write instead.
    // The repository keeps tabs out of its sources, so this one is generated.
    let path = directory.join("controls.or");
    fs::write(
        &path,
        "edition 2026;\nmodule controls {\n  spec tab() -> Word[8]^3 { \"a\tb\" }\n  \
         spec delete() -> Word[8]^3 { \"a\u{7f}b\" }\n}\n",
    )
    .unwrap();
    let rejected = run_twice("check", &path, "control characters");
    assert_failure(
        &rejected,
        &["ORC0235", "ORC0235"],
        &["controls.or:3:31", "controls.or:4:34"],
        &[
            "U+0009 is not a printable ASCII character",
            "its byte is written `hex\"09\"`",
            "U+007F is not a printable ASCII character",
            "its byte is written `hex\"7f\"`",
        ],
        "control characters",
    );
    fs::remove_dir_all(&directory).unwrap();
}
