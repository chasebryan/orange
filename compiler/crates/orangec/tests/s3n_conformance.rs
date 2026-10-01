//! External conformance evidence for the proposed Orange 2026 S3n slice.
//!
//! The rule index lives in `docs/ORDER_2026.md`. This runner requires that
//! index to agree exactly with the evidence map below, runs every fixture and
//! every generated program through the real `orangec` binary twice, and
//! checks that every named unit test is declared once in its source's test
//! module. S3m compatibility is also observed by the S2 through S3m runners,
//! which run their fixtures through the same binary.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const ORDER_SPECIFICATION: &str = include_str!("../../../../docs/ORDER_2026.md");
const S3N_CONFORMANCE_SOURCE: &str = include_str!("s3n_conformance.rs");
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

/// Every S3n rule, in the order of the specification's index.
const RULES: [&str; 8] = [
    "S3N-SYNTAX-01",
    "S3N-TYPE-01",
    "S3N-VALUE-01",
    "S3N-CORE-01",
    "S3N-EVAL-01",
    "S3N-RES-01",
    "S3N-COMPAT-01",
    "S3N-DETERMINISM-01",
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
    "S3N-SYNTAX-01",
    "S3N-TYPE-01",
    "S3N-VALUE-01",
    "S3N-CORE-01",
    "S3N-EVAL-01",
    "S3N-DETERMINISM-01",
];

const CASES: [Case; 8] = [
    Case {
        fixture: "valid-sha256.or",
        expectation: Expectation::Success(concat!(
            "sha256::round_constants: Word[32]^64 = [0x428a2f98, 0x71374491, 0xb5c0fbcf, ",
            "0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5, 0xd807aa98, ",
            "0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, ",
            "0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, ",
            "0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, ",
            "0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, ",
            "0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, ",
            "0x92722c85, 0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, ",
            "0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116, 0x1e376c08, 0x2748774c, ",
            "0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3, 0x748f82ee, ",
            "0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, ",
            "0xc67178f2]\n",
            "sha256::initial_hash: Word[32]^8 = [0x6a09e667, 0xbb67ae85, 0x3c6ef372, ",
            "0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19]\n",
            "sha256::abc: Word[8]^32 = [0xba, 0x78, 0x16, 0xbf, 0x8f, 0x01, 0xcf, 0xea, ",
            "0x41, 0x41, 0x40, 0xde, 0x5d, 0xae, 0x22, 0x23, 0xb0, 0x03, 0x61, 0xa3, 0x96, ",
            "0x17, 0x7a, 0x9c, 0xb4, 0x10, 0xff, 0x61, 0xf2, 0x00, 0x15, 0xad]\n",
            "sha256::two_blocks: Word[8]^32 = [0x24, 0x8d, 0x6a, 0x61, 0xd2, 0x06, 0x38, ",
            "0xb8, 0xe5, 0xc0, 0x26, 0x93, 0x0c, 0x3e, 0x60, 0x39, 0xa3, 0x3c, 0xe4, 0x59, ",
            "0x64, 0xff, 0x21, 0x67, 0xf6, 0xec, 0xed, 0xd4, 0x19, 0xdb, 0x06, 0xc1]\n",
            "sha256::full_block: Word[8]^32 = [0xaa, 0x35, 0x3e, 0x00, 0x9e, 0xdb, 0xae, ",
            "0xbf, 0xc6, 0xe4, 0x94, 0xc8, 0xd8, 0x47, 0x69, 0x68, 0x96, 0xcb, 0x8b, 0x39, ",
            "0x8e, 0x01, 0x73, 0xa4, 0xb5, 0xc1, 0xb6, 0x36, 0x29, 0x2d, 0x87, 0xc7]\n",
            "sha256::longest: Word[8]^32 = [0x31, 0xeb, 0xa5, 0x1c, 0x31, 0x3a, 0x5c, ",
            "0x08, 0x22, 0x6a, 0xdf, 0x18, 0xd4, 0xa3, 0x59, 0xcf, 0xdf, 0xd8, 0xd2, 0xe8, ",
            "0x16, 0xb1, 0x3f, 0x4a, 0xf9, 0x52, 0xf7, 0xea, 0x65, 0x84, 0xdc, 0xfb]\n",
        )),
        rules: VALID_RULES,
    },
    Case {
        fixture: "valid-sha512.or",
        expectation: Expectation::Success(concat!(
            "sha512::round_constants: Word[64]^80 = [0x428a2f98d728ae22, ",
            "0x7137449123ef65cd, 0xb5c0fbcfec4d3b2f, 0xe9b5dba58189dbbc, ",
            "0x3956c25bf348b538, 0x59f111f1b605d019, 0x923f82a4af194f9b, ",
            "0xab1c5ed5da6d8118, 0xd807aa98a3030242, 0x12835b0145706fbe, ",
            "0x243185be4ee4b28c, 0x550c7dc3d5ffb4e2, 0x72be5d74f27b896f, ",
            "0x80deb1fe3b1696b1, 0x9bdc06a725c71235, 0xc19bf174cf692694, ",
            "0xe49b69c19ef14ad2, 0xefbe4786384f25e3, 0x0fc19dc68b8cd5b5, ",
            "0x240ca1cc77ac9c65, 0x2de92c6f592b0275, 0x4a7484aa6ea6e483, ",
            "0x5cb0a9dcbd41fbd4, 0x76f988da831153b5, 0x983e5152ee66dfab, ",
            "0xa831c66d2db43210, 0xb00327c898fb213f, 0xbf597fc7beef0ee4, ",
            "0xc6e00bf33da88fc2, 0xd5a79147930aa725, 0x06ca6351e003826f, ",
            "0x142929670a0e6e70, 0x27b70a8546d22ffc, 0x2e1b21385c26c926, ",
            "0x4d2c6dfc5ac42aed, 0x53380d139d95b3df, 0x650a73548baf63de, ",
            "0x766a0abb3c77b2a8, 0x81c2c92e47edaee6, 0x92722c851482353b, ",
            "0xa2bfe8a14cf10364, 0xa81a664bbc423001, 0xc24b8b70d0f89791, ",
            "0xc76c51a30654be30, 0xd192e819d6ef5218, 0xd69906245565a910, ",
            "0xf40e35855771202a, 0x106aa07032bbd1b8, 0x19a4c116b8d2d0c8, ",
            "0x1e376c085141ab53, 0x2748774cdf8eeb99, 0x34b0bcb5e19b48a8, ",
            "0x391c0cb3c5c95a63, 0x4ed8aa4ae3418acb, 0x5b9cca4f7763e373, ",
            "0x682e6ff3d6b2b8a3, 0x748f82ee5defb2fc, 0x78a5636f43172f60, ",
            "0x84c87814a1f0ab72, 0x8cc702081a6439ec, 0x90befffa23631e28, ",
            "0xa4506cebde82bde9, 0xbef9a3f7b2c67915, 0xc67178f2e372532b, ",
            "0xca273eceea26619c, 0xd186b8c721c0c207, 0xeada7dd6cde0eb1e, ",
            "0xf57d4f7fee6ed178, 0x06f067aa72176fba, 0x0a637dc5a2c898a6, ",
            "0x113f9804bef90dae, 0x1b710b35131c471b, 0x28db77f523047d84, ",
            "0x32caab7b40c72493, 0x3c9ebe0a15c9bebc, 0x431d67c49c100d4c, ",
            "0x4cc5d4becb3e42b6, 0x597f299cfc657e2a, 0x5fcb6fab3ad6faec, ",
            "0x6c44198c4a475817]\n",
            "sha512::initial_hash: Word[64]^8 = [0x6a09e667f3bcc908, 0xbb67ae8584caa73b, ",
            "0x3c6ef372fe94f82b, 0xa54ff53a5f1d36f1, 0x510e527fade682d1, ",
            "0x9b05688c2b3e6c1f, 0x1f83d9abfb41bd6b, 0x5be0cd19137e2179]\n",
            "sha512::abc: Word[8]^64 = [0xdd, 0xaf, 0x35, 0xa1, 0x93, 0x61, 0x7a, 0xba, ",
            "0xcc, 0x41, 0x73, 0x49, 0xae, 0x20, 0x41, 0x31, 0x12, 0xe6, 0xfa, 0x4e, 0x89, ",
            "0xa9, 0x7e, 0xa2, 0x0a, 0x9e, 0xee, 0xe6, 0x4b, 0x55, 0xd3, 0x9a, 0x21, 0x92, ",
            "0x99, 0x2a, 0x27, 0x4f, 0xc1, 0xa8, 0x36, 0xba, 0x3c, 0x23, 0xa3, 0xfe, 0xeb, ",
            "0xbd, 0x45, 0x4d, 0x44, 0x23, 0x64, 0x3c, 0xe8, 0x0e, 0x2a, 0x9a, 0xc9, 0x4f, ",
            "0xa5, 0x4c, 0xa4, 0x9f]\n",
            "sha512::two_blocks: Word[8]^64 = [0x8e, 0x95, 0x9b, 0x75, 0xda, 0xe3, 0x13, ",
            "0xda, 0x8c, 0xf4, 0xf7, 0x28, 0x14, 0xfc, 0x14, 0x3f, 0x8f, 0x77, 0x79, 0xc6, ",
            "0xeb, 0x9f, 0x7f, 0xa1, 0x72, 0x99, 0xae, 0xad, 0xb6, 0x88, 0x90, 0x18, 0x50, ",
            "0x1d, 0x28, 0x9e, 0x49, 0x00, 0xf7, 0xe4, 0x33, 0x1b, 0x99, 0xde, 0xc4, 0xb5, ",
            "0x43, 0x3a, 0xc7, 0xd3, 0x29, 0xee, 0xb6, 0xdd, 0x26, 0x54, 0x5e, 0x96, 0xe5, ",
            "0x5b, 0x87, 0x4b, 0xe9, 0x09]\n",
            "sha512::full_block: Word[8]^64 = [0xfa, 0x91, 0x21, 0xc7, 0xb3, 0x2b, 0x9e, ",
            "0x01, 0x73, 0x3d, 0x03, 0x4c, 0xfc, 0x78, 0xcb, 0xf6, 0x7f, 0x92, 0x6c, 0x7e, ",
            "0xd8, 0x3e, 0x82, 0x20, 0x0e, 0xf8, 0x68, 0x18, 0x19, 0x69, 0x21, 0x76, 0x0b, ",
            "0x4b, 0xef, 0xf4, 0x84, 0x04, 0xdf, 0x81, 0x1b, 0x95, 0x38, 0x28, 0x27, 0x44, ",
            "0x61, 0x67, 0x3c, 0x68, 0xd0, 0x4e, 0x29, 0x7b, 0x0e, 0xb7, 0xb2, 0xb4, 0xd6, ",
            "0x0f, 0xc6, 0xb5, 0x66, 0xa2]\n",
            "sha512::longest: Word[8]^64 = [0x52, 0xc8, 0x53, 0xcb, 0x8d, 0x90, 0x7f, ",
            "0x3d, 0x4d, 0x6b, 0x88, 0x9b, 0xeb, 0x02, 0x79, 0x85, 0xd7, 0xc2, 0x73, 0x48, ",
            "0x6d, 0x75, 0xf8, 0xba, 0xf2, 0x6f, 0x80, 0xd2, 0x4e, 0x90, 0xc7, 0x4c, 0x6c, ",
            "0x3d, 0xe3, 0xe2, 0x21, 0x31, 0x58, 0x23, 0x80, 0xa7, 0xd1, 0x4d, 0x43, 0xf2, ",
            "0x94, 0x1a, 0x31, 0x38, 0x54, 0x39, 0xcd, 0x6d, 0xdc, 0x46, 0x9f, 0x62, 0x80, ",
            "0x15, 0xe5, 0x0b, 0xf2, 0x86]\n",
        )),
        rules: VALID_RULES,
    },
    Case {
        fixture: "valid-chacha20.or",
        expectation: Expectation::Success(concat!(
            "chacha20::block_vector: Word[8]^64 = [0x10, 0xf1, 0xe7, 0xe4, 0xd1, 0x3b, ",
            "0x59, 0x15, 0x50, 0x0f, 0xdd, 0x1f, 0xa3, 0x20, 0x71, 0xc4, 0xc7, 0xd1, 0xf4, ",
            "0xc7, 0x33, 0xc0, 0x68, 0x03, 0x04, 0x22, 0xaa, 0x9a, 0xc3, 0xd4, 0x6c, 0x4e, ",
            "0xd2, 0x82, 0x64, 0x46, 0x07, 0x9f, 0xaa, 0x09, 0x14, 0xc2, 0xd7, 0x05, 0xd9, ",
            "0x8b, 0x02, 0xa2, 0xb5, 0x12, 0x9c, 0xd1, 0xde, 0x16, 0x4e, 0xb9, 0xcb, 0xd0, ",
            "0x83, 0xe8, 0xa2, 0x50, 0x3c, 0x4e]\n",
            "chacha20::ciphertext_vector: Word[8]^114 = [0x6e, 0x2e, 0x35, 0x9a, 0x25, ",
            "0x68, 0xf9, 0x80, 0x41, 0xba, 0x07, 0x28, 0xdd, 0x0d, 0x69, 0x81, 0xe9, 0x7e, ",
            "0x7a, 0xec, 0x1d, 0x43, 0x60, 0xc2, 0x0a, 0x27, 0xaf, 0xcc, 0xfd, 0x9f, 0xae, ",
            "0x0b, 0xf9, 0x1b, 0x65, 0xc5, 0x52, 0x47, 0x33, 0xab, 0x8f, 0x59, 0x3d, 0xab, ",
            "0xcd, 0x62, 0xb3, 0x57, 0x16, 0x39, 0xd6, 0x24, 0xe6, 0x51, 0x52, 0xab, 0x8f, ",
            "0x53, 0x0c, 0x35, 0x9f, 0x08, 0x61, 0xd8, 0x07, 0xca, 0x0d, 0xbf, 0x50, 0x0d, ",
            "0x6a, 0x61, 0x56, 0xa3, 0x8e, 0x08, 0x8a, 0x22, 0xb6, 0x5e, 0x52, 0xbc, 0x51, ",
            "0x4d, 0x16, 0xcc, 0xf8, 0x06, 0x81, 0x8c, 0xe9, 0x1a, 0xb7, 0x79, 0x37, 0x36, ",
            "0x5a, 0xf9, 0x0b, 0xbf, 0x74, 0xa3, 0x5b, 0xe6, 0xb4, 0x0b, 0x8e, 0xed, 0xf2, ",
            "0x78, 0x5e, 0x42, 0x87, 0x4d]\n",
        )),
        rules: VALID_RULES,
    },
    Case {
        fixture: "valid-poly1305.or",
        expectation: Expectation::Success(concat!(
            "poly1305::example: Word[8]^16 = [0xa8, 0x06, 0x1d, 0xc1, 0x30, 0x51, 0x36, ",
            "0xc6, 0xc2, 0x2b, 0x8b, 0xaf, 0x0c, 0x01, 0x27, 0xa9]\n",
            "poly1305::whole_block: Word[8]^16 = [0x03, 0x00, 0x00, 0x00, 0x00, 0x00, ",
            "0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]\n",
            "poly1305::wraps: Word[8]^16 = [0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, ",
            "0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]\n",
        )),
        rules: VALID_RULES,
    },
    Case {
        fixture: "valid-x25519.or",
        expectation: Expectation::Success(concat!(
            "x25519::test_vector: Word[8]^32 = [0xc3, 0xda, 0x55, 0x37, 0x9d, 0xe9, 0xc6, ",
            "0x90, 0x8e, 0x94, 0xea, 0x4d, 0xf2, 0x8d, 0x08, 0x4f, 0x32, 0xec, 0xcf, 0x03, ",
            "0x49, 0x1c, 0x71, 0xf7, 0x54, 0xb4, 0x07, 0x55, 0x77, 0xa2, 0x85, 0x52]\n",
        )),
        rules: VALID_RULES,
    },
    Case {
        fixture: "valid-order.or",
        expectation: Expectation::Success(concat!(
            "order::big_word: Word[32] = 0x01020304\n",
            "order::little_word: Word[32] = 0x04030201\n",
            "order::text: Word[32] = 0x61626364\n",
            "order::bytes: Word[8]^4 = [0xde, 0xad, 0xbe, 0xef]\n",
            "order::round_trip: Word[32] = 0xdeadbeef\n",
            "order::joined: Word[64] = 0x0123456789abcdef\n",
            "order::quarters: Word[16]^4 = [0xcdef, 0x89ab, 0x4567, 0x0123]\n",
            "order::number: Int = 256\n",
            "order::little_number: Int = 1\n",
            "order::minus_one: Word[8]^4 = [0xff, 0xff, 0xff, 0xff]\n",
            "order::wraps: Word[8]^2 = [0x00, 0x03]\n",
            "order::residue: Mod[251] = 5\n",
            "order::residue_bytes: Word[8]^2 = [0xff, 0xf0]\n",
            "order::bits[1]: Word[8]^2 = [0x00, 0x08]\n",
            "order::bits[2]: Word[8]^2 = [0x00, 0x10]\n",
        )),
        rules: VALID_RULES,
    },
    Case {
        fixture: "invalid-order.or",
        expectation: Expectation::Failure {
            codes: &[
                "ORC0240", "ORC0240", "ORC0215", "ORC0215", "ORC0215", "ORC0215", "ORC0215",
                "ORC0220", "ORC0214", "ORC0215", "ORC0215",
            ],
            locations: &[
                "invalid-order.or:8:51",
                "invalid-order.or:9:54",
                "invalid-order.or:10:41",
                "invalid-order.or:11:38",
                "invalid-order.or:12:53",
                "invalid-order.or:13:54",
                "invalid-order.or:14:38",
                "invalid-order.or:15:32",
                "invalid-order.or:16:52",
                "invalid-order.or:17:43",
                "invalid-order.or:18:44",
            ],
            messages: &[
                "`Word[8]^3` and `Word[32]` have different widths",
                "`Word[32]` has 32 bits",
                "`Word[8]^3` has 24 bits",
                "a byte order keeps every bit of the words it converts, so words convert only \
                 to words of the same number of bits",
                "`Word[64]` and `Word[16]^2` have different widths",
                "`big` orders words, but this converts `Int` to `Mod[7]`",
                "a number converts to another without a byte order, as `x as Mod[7]`",
                "`as big` does not convert `Bool`",
                "`as big` and `as little` convert a word or an array of words to words of \
                 another width, to `Int`, or to `Mod[m]`, and back",
                "`as little` does not convert to `Bool`",
                "`as big` does not convert to `Mod[7]^2`",
                "`as big` does not convert `Int^2`",
                "the operand of `as` has no type of its own",
                "this conversion gives `Word[32]`, but `Word[64]` is required here",
                "`as` is not defined for `Word[8]^4`",
                "name a byte order to read the words as one number or as words of another \
                 width, as in `x as big Int`, or convert each element, such as `x[0] as Int`",
                "`as` does not convert to the array type `Word[32]^2`",
                "name a byte order to write words as `Word[32]^2`, as in `as big Word[32]^2`, \
                 or build the array from its elements",
            ],
        },
        rules: &["S3N-TYPE-01", "S3N-COMPAT-01", "S3N-DETERMINISM-01"],
    },
    Case {
        fixture: "invalid-order-syntax.or",
        expectation: Expectation::Failure {
            codes: &["ORC0108", "ORC0108"],
            locations: &[
                "invalid-order-syntax.or:6:54",
                "invalid-order-syntax.or:7:46",
            ],
            messages: &[
                "`^` follows `as` without grouping parentheses",
                "For an array type, name a byte order first, as in `as big Word[32]^16`",
                "`as` follows `as` without grouping parentheses",
            ],
        },
        rules: &["S3N-SYNTAX-01", "S3N-COMPAT-01", "S3N-DETERMINISM-01"],
    },
];

/// Generated command-line cases in this file, each run twice.
const GENERATED_EVIDENCE: &[(&str, &[&str])] = &[(
    "s3n_orders_are_exact_for_every_width_and_at_the_limit",
    &[
        "S3N-TYPE-01",
        "S3N-VALUE-01",
        "S3N-EVAL-01",
        "S3N-RES-01",
        "S3N-DETERMINISM-01",
    ],
)];

const UNIT_EVIDENCE: &[TestEvidence] = &[
    TestEvidence {
        source_path: "src/parser.rs",
        test: "operator_inventories_spellings_and_tokens_are_exact",
        rules: &["S3N-SYNTAX-01"],
    },
    TestEvidence {
        source_path: "src/parser.rs",
        test: "byte_orders_follow_as_when_a_type_follows_them",
        rules: &["S3N-SYNTAX-01", "S3N-COMPAT-01"],
    },
    TestEvidence {
        source_path: "src/parser.rs",
        test: "an_array_type_after_as_without_a_byte_order_is_ungrouped",
        rules: &["S3N-SYNTAX-01", "S3N-COMPAT-01"],
    },
    TestEvidence {
        source_path: "src/semantics/tests.rs",
        test: "byte_orders_pack_words_into_words_and_numbers_and_back",
        rules: &["S3N-TYPE-01", "S3N-CORE-01", "S3N-DETERMINISM-01"],
    },
    TestEvidence {
        source_path: "src/semantics/tests.rs",
        test: "byte_orders_convert_words_of_one_width_or_a_number",
        rules: &["S3N-TYPE-01"],
    },
    TestEvidence {
        source_path: "src/semantics/tests.rs",
        test: "operators_and_conversions_apply_to_elements_not_arrays",
        rules: &["S3N-TYPE-01", "S3N-COMPAT-01"],
    },
    TestEvidence {
        source_path: "src/semantics/tests.rs",
        test: "a_byte_order_is_one_semantic_event",
        rules: &["S3N-RES-01"],
    },
    TestEvidence {
        source_path: "src/semantics/tests.rs",
        test: "rejects_a_foreign_byte_order_span",
        rules: &["S3N-RES-01"],
    },
    TestEvidence {
        source_path: "src/core.rs",
        test: "core_accessors_preserve_source_order_and_derive_value_types",
        rules: &["S3N-CORE-01"],
    },
    TestEvidence {
        source_path: "src/eval.rs",
        test: "byte_orders_match_a_wide_reference_for_every_pair_of_widths",
        rules: &["S3N-VALUE-01", "S3N-EVAL-01"],
    },
    TestEvidence {
        source_path: "src/eval.rs",
        test: "byte_orders_write_numbers_as_their_residues_and_read_residues",
        rules: &["S3N-VALUE-01", "S3N-EVAL-01"],
    },
    TestEvidence {
        source_path: "src/eval.rs",
        test: "byte_orders_reach_the_widest_array_and_integer",
        rules: &["S3N-EVAL-01", "S3N-RES-01"],
    },
    TestEvidence {
        source_path: "src/eval.rs",
        test: "byte_order_limb_reservation_failure_returns_no_values",
        rules: &["S3N-RES-01"],
    },
    TestEvidence {
        source_path: "src/eval.rs",
        test: "steps_follow_the_normative_cost_table",
        rules: &["S3N-EVAL-01"],
    },
    TestEvidence {
        source_path: "src/eval.rs",
        test: "deepest_accepted_sources_fit_in_one_mebibyte_of_stack",
        rules: &["S3N-RES-01"],
    },
    TestEvidence {
        source_path: "src/diagnostic.rs",
        test: "diagnostic_code_inventory_is_exact_ordered_and_unique",
        rules: &["S3N-TYPE-01"],
    },
];

fn orangec() -> Command {
    Command::new(env!("CARGO_BIN_EXE_orangec"))
}

fn fixture_directory() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/s3n")
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
        .join(format!("orangec-s3n-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&directory);
    fs::create_dir(&directory).unwrap();
    directory
}

/// The rule index rows of the specification: rule ID and evidence label.
fn documented_rules() -> Vec<(&'static str, &'static str)> {
    ORDER_SPECIFICATION
        .lines()
        .filter_map(|line| {
            let row = line.strip_prefix("| `")?;
            let (rule, _) = row.split_once("` |")?;
            if !rule.starts_with("S3N-") {
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
        _ => panic!("unknown S3n evidence layer {label:?}"),
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
        _ => panic!("unmapped S3n evidence source {source_path}"),
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
        S3N_CONFORMANCE_SOURCE.matches(&declaration).count(),
        1,
        "generated evidence {test} must be declared exactly once at file root"
    );
}

#[test]
fn s3n_rule_index_is_exact_and_covered() {
    let documented = documented_rules();
    let documented_ids: Vec<_> = documented.iter().map(|(rule, _)| *rule).collect();
    assert_eq!(
        documented_ids, RULES,
        "docs/ORDER_2026.md rule index drifted from the S3n runner"
    );
    let known: BTreeSet<_> = RULES.iter().copied().collect();
    assert_eq!(known.len(), RULES.len(), "duplicate S3n rule ID");

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
fn s3n_cli_conformance_corpus_is_exact_and_repeatable() {
    let directory = fixture_directory();
    let mut observed: Vec<_> = fs::read_dir(&directory)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().into_string().unwrap())
        .collect();
    observed.sort();
    let mut expected: Vec<_> = CASES.iter().map(|case| case.fixture).collect();
    expected.sort_unstable();
    assert_eq!(observed, expected, "unexpected S3n fixture inventory");

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
fn s3n_orders_are_exact_for_every_width_and_at_the_limit() {
    let directory = scratch_directory("orders");
    const WIDTHS: [u32; 4] = [8, 16, 32, 64];
    const ORDERS: [&str; 2] = ["big", "little"];
    // The sixteen bytes 00 01 ... 0f, read big-endian.
    let bytes: u128 = 0x0001_0203_0405_0607_0809_0a0b_0c0d_0e0f;
    let chunk = |value: u128, bits: u32, count: u32, index: u32, order: &str| {
        let place = if order == "big" {
            count - 1 - index
        } else {
            index
        };
        (value >> (bits * place)) & ((1_u128 << bits) - 1)
    };
    // The value that words of `bits` bits, read big-endian from `bytes`,
    // spell in `order`.
    let spelled = |bits: u32, order: &str| {
        let count = 128 / bits;
        (0..count).fold(0_u128, |value, index| {
            let word = chunk(bytes, bits, count, index, "big");
            let place = if order == "big" {
                count - 1 - index
            } else {
                index
            };
            value | (word << (bits * place))
        })
    };

    // Words of every width, read big-endian from the same sixteen bytes,
    // convert in each order to words of every width and to `Int`. An array
    // of 256 of the widest words is 16384 bits, the most an `Int` holds, and
    // converts to `Int` and back exactly.
    let mut program = String::from("edition 2026;\nmodule orders {\n");
    let mut expected = String::new();
    for from in WIDTHS {
        let from_count = 128 / from;
        for order in ORDERS {
            for to in WIDTHS {
                let count = 128 / to;
                write!(
                    program,
                    "  spec w{from}_{to}_{order}() -> Word[{to}]^{count} {{\n    \
                     let x: Word[{from}]^{from_count} = \
                     hex\"000102030405060708090a0b0c0d0e0f\" as big Word[{from}]^{from_count};\n    \
                     x as {order} Word[{to}]^{count}\n  }}\n"
                )
                .unwrap();
                let value = spelled(from, order);
                let words = (0..count)
                    .map(|index| {
                        let digits = usize::try_from(to / 4 + 2).unwrap();
                        format!("{:#0digits$x}", chunk(value, to, count, index, order))
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                writeln!(
                    expected,
                    "orders::w{from}_{to}_{order}: Word[{to}]^{count} = [{words}]"
                )
                .unwrap();
            }
            write!(
                program,
                "  spec n{from}_{order}() -> Int {{\n    \
                 let x: Word[{from}]^{from_count} = \
                 hex\"000102030405060708090a0b0c0d0e0f\" as big Word[{from}]^{from_count};\n    \
                 x as {order} Int\n  }}\n"
            )
            .unwrap();
            writeln!(
                expected,
                "orders::n{from}_{order}: Int = {}",
                spelled(from, order)
            )
            .unwrap();
        }
    }
    program.push_str(
        "  spec widest() -> Word[64]^256 {\n    \
         let x: Word[64]^256 = for i in 0..256 with w: Word[64]^256 = [0; 256] {\n      \
         w with [i] = i as Word[64]\n    };\n    \
         (x as big Int) as little Word[64]^256\n  }\n}\n",
    );
    let reversed = (0..256_u32)
        .rev()
        .map(|index| format!("{index:#018x}"))
        .collect::<Vec<_>>()
        .join(", ");
    writeln!(expected, "orders::widest: Word[64]^256 = [{reversed}]").unwrap();
    let path = directory.join("orders.or");
    fs::write(&path, &program).unwrap();
    let orders = run_twice("eval", &path, "every width");
    assert_success(&orders, &expected, "every width");

    // Words of more than 65536 elements are no type, so no conversion
    // reaches them.
    let over = directory.join("over.or");
    fs::write(
        &over,
        "edition 2026;\nmodule limits {\n  \
         spec over(x: Word[64]^65536) -> Word[8]^524288 { x as little Word[8]^524288 }\n  \
         spec inner(x: Word[64]^65536) -> Int { (x as little Word[16]^262144) as big Int }\n}\n",
    )
    .unwrap();
    let rejected = run_twice("check", &over, "one element more");
    assert_failure(
        &rejected,
        &["ORC0221", "ORC0221"],
        &["over.or:3:43", "over.or:4:64"],
        &["an array length must be a decimal integer from 1 through 65536"],
        "one element more",
    );
    fs::remove_dir_all(&directory).unwrap();
}
