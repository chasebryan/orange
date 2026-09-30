//! External conformance evidence for the proposed Orange 2026 S3o slice.
//!
//! The rule index lives in `docs/TYPE_PARAMETERS_2026.md`. This runner
//! requires that index to agree exactly with the evidence map below, runs
//! every fixture and every generated program through the real `orangec`
//! binary twice, and checks that every named unit test is declared once in
//! its source's test module. S3n compatibility is also observed by the S2
//! through S3n runners, which run their fixtures through the same binary.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const TYPE_PARAMETERS_SPECIFICATION: &str =
    include_str!("../../../../docs/TYPE_PARAMETERS_2026.md");
const S3O_CONFORMANCE_SOURCE: &str = include_str!("s3o_conformance.rs");
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

/// Every S3o rule, in the order of the specification's index.
const RULES: [&str; 10] = [
    "S3O-SYNTAX-01",
    "S3O-DECL-01",
    "S3O-INSTANCE-01",
    "S3O-CALL-01",
    "S3O-FIT-01",
    "S3O-CORE-01",
    "S3O-EVAL-01",
    "S3O-RES-01",
    "S3O-COMPAT-01",
    "S3O-DETERMINISM-01",
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
    "S3O-SYNTAX-01",
    "S3O-DECL-01",
    "S3O-INSTANCE-01",
    "S3O-CALL-01",
    "S3O-FIT-01",
    "S3O-CORE-01",
    "S3O-EVAL-01",
    "S3O-DETERMINISM-01",
];

const CASES: [Case; 5] = [
    Case {
        fixture: "valid-fields.or",
        expectation: Expectation::Success(concat!(
            "fields::modulus[F]: Int = ",
            "57896044618658097711785492504343953926634992332820282019728792003956564819949\n",
            "fields::modulus[L]: Int = ",
            "7237005577332262213973186563042994240857116359379907606001950938285454250989\n",
            "fields::modulus[P]: Int = 1361129467683753853853498429727072845819\n",
            "fields::modulus[Q]: Int = 3329\n",
            "fields::modulus[D]: Int = 8380417\n",
            "fields::inverts[F]: Bool = true\n",
            "fields::inverts[L]: Bool = true\n",
            "fields::inverts[P]: Bool = true\n",
            "fields::inverts[Q]: Bool = true\n",
            "fields::inverts[D]: Bool = true\n",
            "fields::two_is_square[F]: Int = -1\n",
            "fields::two_is_square[L]: Int = -1\n",
            "fields::two_is_square[P]: Int = -1\n",
            "fields::two_is_square[Q]: Int = 1\n",
            "fields::two_is_square[D]: Int = 1\n",
            "fields::sqrt_minus_one: Mod[(1 << 255) - 19] = ",
            "19681161376707505956807079304988542015446066515923890162744021073123829784752\n",
            "fields::squares_to_minus_one: Bool = true\n",
            "fields::kem_root: Mod[3329] = 3328\n",
            "fields::dsa_root: Mod[8380417] = 8380416\n",
            "fields::coefficients: (Mod[3329], Mod[8380417]) = (2342, 6478332)\n",
        )),
        rules: VALID_RULES,
    },
    Case {
        fixture: "valid-sha2.or",
        expectation: Expectation::Success(concat!(
            "sha2::round_constants256: Word[32]^64 = [0x428a2f98, 0x71374491, ",
            "0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5, ",
            "0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, ",
            "0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, ",
            "0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, ",
            "0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, ",
            "0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, ",
            "0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, ",
            "0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116, 0x1e376c08, ",
            "0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3, ",
            "0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, ",
            "0xbef9a3f7, 0xc67178f2]\n",
            "sha2::initial_hash256: Word[32]^8 = [0x6a09e667, 0xbb67ae85, 0x3c6ef372, ",
            "0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19]\n",
            "sha2::round_constants512: Word[64]^80 = [0x428a2f98d728ae22, ",
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
            "sha2::initial_hash512: Word[64]^8 = [0x6a09e667f3bcc908, ",
            "0xbb67ae8584caa73b, 0x3c6ef372fe94f82b, 0xa54ff53a5f1d36f1, ",
            "0x510e527fade682d1, 0x9b05688c2b3e6c1f, 0x1f83d9abfb41bd6b, ",
            "0x5be0cd19137e2179]\n",
            "sha2::sha256_abc: Word[8]^32 = [0xba, 0x78, 0x16, 0xbf, 0x8f, 0x01, 0xcf, ",
            "0xea, 0x41, 0x41, 0x40, 0xde, 0x5d, 0xae, 0x22, 0x23, 0xb0, 0x03, 0x61, ",
            "0xa3, 0x96, 0x17, 0x7a, 0x9c, 0xb4, 0x10, 0xff, 0x61, 0xf2, 0x00, 0x15, ",
            "0xad]\n",
            "sha2::sha256_two_blocks: Word[8]^32 = [0x24, 0x8d, 0x6a, 0x61, 0xd2, ",
            "0x06, 0x38, 0xb8, 0xe5, 0xc0, 0x26, 0x93, 0x0c, 0x3e, 0x60, 0x39, 0xa3, ",
            "0x3c, 0xe4, 0x59, 0x64, 0xff, 0x21, 0x67, 0xf6, 0xec, 0xed, 0xd4, 0x19, ",
            "0xdb, 0x06, 0xc1]\n",
            "sha2::sha512_abc: Word[8]^64 = [0xdd, 0xaf, 0x35, 0xa1, 0x93, 0x61, 0x7a, ",
            "0xba, 0xcc, 0x41, 0x73, 0x49, 0xae, 0x20, 0x41, 0x31, 0x12, 0xe6, 0xfa, ",
            "0x4e, 0x89, 0xa9, 0x7e, 0xa2, 0x0a, 0x9e, 0xee, 0xe6, 0x4b, 0x55, 0xd3, ",
            "0x9a, 0x21, 0x92, 0x99, 0x2a, 0x27, 0x4f, 0xc1, 0xa8, 0x36, 0xba, 0x3c, ",
            "0x23, 0xa3, 0xfe, 0xeb, 0xbd, 0x45, 0x4d, 0x44, 0x23, 0x64, 0x3c, 0xe8, ",
            "0x0e, 0x2a, 0x9a, 0xc9, 0x4f, 0xa5, 0x4c, 0xa4, 0x9f]\n",
            "sha2::sha512_two_blocks: Word[8]^64 = [0x8e, 0x95, 0x9b, 0x75, 0xda, ",
            "0xe3, 0x13, 0xda, 0x8c, 0xf4, 0xf7, 0x28, 0x14, 0xfc, 0x14, 0x3f, 0x8f, ",
            "0x77, 0x79, 0xc6, 0xeb, 0x9f, 0x7f, 0xa1, 0x72, 0x99, 0xae, 0xad, 0xb6, ",
            "0x88, 0x90, 0x18, 0x50, 0x1d, 0x28, 0x9e, 0x49, 0x00, 0xf7, 0xe4, 0x33, ",
            "0x1b, 0x99, 0xde, 0xc4, 0xb5, 0x43, 0x3a, 0xc7, 0xd3, 0x29, 0xee, 0xb6, ",
            "0xdd, 0x26, 0x54, 0x5e, 0x96, 0xe5, 0x5b, 0x87, 0x4b, 0xe9, 0x09]\n",
        )),
        rules: VALID_RULES,
    },
    Case {
        fixture: "valid-types.or",
        expectation: Expectation::Success(concat!(
            "types::seven[Int]: Int = 7\n",
            "types::seven[Word[16]]: Word[16] = 0x0007\n",
            "types::seven[P]: Mod[65521] = 7\n",
            "types::minus_one[Word[16]]: Word[16] = 0xffff\n",
            "types::minus_one[P]: Mod[65521] = 65520\n",
            "types::zero[Int]: Int = 0\n",
            "types::zero[Word[16]]: Word[16] = 0x0000\n",
            "types::zero[P]: Mod[65521] = 0\n",
            "types::chosen: (Word[16], Mod[65521], Int) = (0x3880, 54479, 6)\n",
            "types::named: (Mod[65521], Word[16], Int) = (7, 0x0007, 7)\n",
            "types::word_squares: (Word[16], Word[16]) = (0x012c, 0x5f90)\n",
            "types::residue_squares: (Mod[65521], Mod[65521]) = (300, 24479)\n",
            "types::swapped: (Bool, Bool) = (false, true)\n",
            "types::swapped_numbers: (Int, Int) = (2, 1)\n",
            "types::same_bytes: Word[8]^4 = [0x00, 0x01, 0x02, 0x03]\n",
            "types::same_bytes_named: Word[8]^4 = [0x00, 0x01, 0x02, 0x03]\n",
        )),
        rules: VALID_RULES,
    },
    Case {
        fixture: "invalid-types.or",
        expectation: Expectation::Failure {
            codes: &[
                "ORC0241", "ORC0233", "ORC0233", "ORC0218", "ORC0218", "ORC0204", "ORC0232",
                "ORC0203", "ORC0238", "ORC0211", "ORC0215", "ORC0241", "ORC0241", "ORC0239",
                "ORC0239", "ORC0241", "ORC0214",
            ],
            locations: &[
                "invalid-types.or:13:26",
                "invalid-types.or:14:16",
                "invalid-types.or:15:17",
                "invalid-types.or:16:27",
                "invalid-types.or:17:25",
                "invalid-types.or:18:27",
                "invalid-types.or:18:35",
                "invalid-types.or:18:39",
                "invalid-types.or:19:13",
                "invalid-types.or:20:40",
                "invalid-types.or:21:52",
                "invalid-types.or:24:35",
                "invalid-types.or:25:39",
                "invalid-types.or:26:23",
                "invalid-types.or:27:27",
                "invalid-types.or:28:41",
                "invalid-types.or:29:28",
            ],
            messages: &[
                "`K` lists the type `Mod[3329]` twice",
                "`Int` is a built-in type",
                "duplicate type name `F`",
                "duplicate parameter `K`",
                "duplicate parameter `n`",
                "`Word` width must be exactly 8, 16, 32, or 64",
                "a modulus must be a constant from 2 through 2^521 - 1",
                "unsupported listed type `H`",
                "`many` has 400 instances, but a function has at most 256",
                "`K` is not a parameter of `value`",
                "`%` is not defined for `Mod[3329]`",
                "`square` is defined for `K` in {F, Q}",
                "`square` takes a type for `K` here",
                "`square` takes 1 type in brackets, but this call gives 2",
                "this call fits more than one instance of `square`, among them `square[F]` \
                 and `square[Q]`",
                "no instance of `square` takes arguments of these types",
                "`square` returns `Mod[(1 << 255) - 19]`, but `Mod[3329]` is required here",
                "in the instance `remainder[Q]`, the first of `remainder` in error: a \
                 function is checked once for each type of its type parameters",
                "`K` is a type parameter: it names a type, not a value, so it is written \
                 where a type is, as in `let x: K = 0;`",
            ],
        },
        rules: &[
            "S3O-DECL-01",
            "S3O-INSTANCE-01",
            "S3O-CALL-01",
            "S3O-FIT-01",
            "S3O-DETERMINISM-01",
        ],
    },
    Case {
        fixture: "invalid-types-syntax.or",
        expectation: Expectation::Failure {
            codes: &["ORC0101", "ORC0101", "ORC0101", "ORC0101", "ORC0101"],
            locations: &[
                "invalid-types-syntax.or:6:20",
                "invalid-types-syntax.or:7:27",
                "invalid-types-syntax.or:8:30",
                "invalid-types-syntax.or:9:32",
                "invalid-types-syntax.or:10:61",
            ],
            messages: &[
                "expected a listed type after `{`",
                "expected a listed type after `,`",
                "expected `,` or `}` after the listed type",
                "expected `,` or `}` after the listed type",
                "a function has at most 4 size and type parameters",
                "a type parameter is written `K in {F, L}` and names each type its function \
                 is checked for, as in `spec square[K in {F, L}](x: K) -> K { x * x }`",
            ],
        },
        rules: &["S3O-SYNTAX-01", "S3O-COMPAT-01", "S3O-DETERMINISM-01"],
    },
];

/// Generated command-line cases in this file, each run twice.
const GENERATED_EVIDENCE: &[(&str, &[&str])] = &[(
    "s3o_instances_reach_the_limit_of_256_exactly",
    &[
        "S3O-DECL-01",
        "S3O-CALL-01",
        "S3O-EVAL-01",
        "S3O-RES-01",
        "S3O-DETERMINISM-01",
    ],
)];

const UNIT_EVIDENCE: &[TestEvidence] = &[
    TestEvidence {
        source_path: "src/parser.rs",
        test: "parses_type_parameters_as_lists_of_types_with_exact_spans",
        rules: &["S3O-SYNTAX-01"],
    },
    TestEvidence {
        source_path: "src/parser.rs",
        test: "rejects_malformed_type_parameters_with_exact_messages",
        rules: &["S3O-SYNTAX-01"],
    },
    TestEvidence {
        source_path: "src/parser.rs",
        test: "a_name_before_brackets_is_indexed_unless_a_call_follows",
        rules: &["S3O-SYNTAX-01", "S3O-COMPAT-01"],
    },
    TestEvidence {
        source_path: "src/semantics/tests.rs",
        test: "type_parameters_take_one_core_function_per_listed_type",
        rules: &["S3O-DECL-01", "S3O-INSTANCE-01", "S3O-CORE-01"],
    },
    TestEvidence {
        source_path: "src/semantics/tests.rs",
        test: "each_instance_is_checked_with_its_type_and_the_first_in_error_is_named",
        rules: &["S3O-INSTANCE-01", "S3O-DETERMINISM-01"],
    },
    TestEvidence {
        source_path: "src/semantics/tests.rs",
        test: "type_parameters_list_distinct_types_under_names_of_their_own",
        rules: &["S3O-DECL-01", "S3O-DETERMINISM-01"],
    },
    TestEvidence {
        source_path: "src/semantics/tests.rs",
        test: "calls_name_an_instance_by_its_types_or_fit_one_by_argument_and_result_types",
        rules: &["S3O-CALL-01", "S3O-FIT-01", "S3O-CORE-01"],
    },
    TestEvidence {
        source_path: "src/semantics/tests.rs",
        test: "calls_that_give_no_listed_type_are_reported_at_the_call",
        rules: &["S3O-CALL-01", "S3O-FIT-01", "S3O-DETERMINISM-01"],
    },
    TestEvidence {
        source_path: "src/semantics/tests.rs",
        test: "a_call_whose_type_only_its_place_would_choose_is_reported_where_its_type_is_needed",
        rules: &["S3O-FIT-01", "S3O-DETERMINISM-01"],
    },
    TestEvidence {
        source_path: "src/semantics/tests.rs",
        test: "typed_functions_of_used_modules_are_called_by_their_instances",
        rules: &["S3O-CALL-01", "S3O-FIT-01"],
    },
    TestEvidence {
        source_path: "src/semantics/tests.rs",
        test: "rejects_foreign_listed_type_spans",
        rules: &["S3O-RES-01"],
    },
    TestEvidence {
        source_path: "src/semantics/tests.rs",
        test: "calls_nested_in_arguments_and_branches_are_fitted_once_per_level",
        rules: &["S3O-RES-01", "S3O-COMPAT-01"],
    },
    TestEvidence {
        source_path: "src/eval.rs",
        test: "every_instance_of_a_typed_root_is_evaluated_and_named_by_its_types",
        rules: &["S3O-CORE-01", "S3O-EVAL-01"],
    },
    TestEvidence {
        source_path: "src/eval.rs",
        test: "typed_functions_are_found_one_instance_at_a_time_by_type_position",
        rules: &["S3O-CORE-01", "S3O-EVAL-01"],
    },
    TestEvidence {
        source_path: "src/eval.rs",
        test: "late_allocation_failures_discard_completed_values",
        rules: &["S3O-RES-01"],
    },
    TestEvidence {
        source_path: "src/eval.rs",
        test: "deepest_accepted_sources_fit_in_one_mebibyte_of_stack",
        rules: &["S3O-RES-01"],
    },
    TestEvidence {
        source_path: "src/diagnostic.rs",
        test: "diagnostic_code_inventory_is_exact_ordered_and_unique",
        rules: &["S3O-DECL-01", "S3O-CALL-01"],
    },
];

fn orangec() -> Command {
    Command::new(env!("CARGO_BIN_EXE_orangec"))
}

fn fixture_directory() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/s3o")
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
        .join(format!("orangec-s3o-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&directory);
    fs::create_dir(&directory).unwrap();
    directory
}

/// The rule index rows of the specification: rule ID and evidence label.
fn documented_rules() -> Vec<(&'static str, &'static str)> {
    TYPE_PARAMETERS_SPECIFICATION
        .lines()
        .filter_map(|line| {
            let row = line.strip_prefix("| `")?;
            let (rule, _) = row.split_once("` |")?;
            if !rule.starts_with("S3O-") {
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
        _ => panic!("unknown S3o evidence layer {label:?}"),
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
        _ => panic!("unmapped S3o evidence source {source_path}"),
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
        S3O_CONFORMANCE_SOURCE.matches(&declaration).count(),
        1,
        "generated evidence {test} must be declared exactly once at file root"
    );
}

#[test]
fn s3o_rule_index_is_exact_and_covered() {
    let documented = documented_rules();
    let documented_ids: Vec<_> = documented.iter().map(|(rule, _)| *rule).collect();
    assert_eq!(
        documented_ids, RULES,
        "docs/TYPE_PARAMETERS_2026.md rule index drifted from the S3o runner"
    );
    let known: BTreeSet<_> = RULES.iter().copied().collect();
    assert_eq!(known.len(), RULES.len(), "duplicate S3o rule ID");

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
fn s3o_cli_conformance_corpus_is_exact_and_repeatable() {
    let directory = fixture_directory();
    let mut observed: Vec<_> = fs::read_dir(&directory)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().into_string().unwrap())
        .collect();
    observed.sort();
    let mut expected: Vec<_> = CASES.iter().map(|case| case.fixture).collect();
    expected.sort_unstable();
    assert_eq!(observed, expected, "unexpected S3o fixture inventory");

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
fn s3o_instances_reach_the_limit_of_256_exactly() {
    let directory = scratch_directory("limits");
    // Sixty-four residue types, `Mod[2]` through `Mod[65]`, listed by one
    // type parameter beside a size of four values: 256 instances, the most
    // a function has. Each gives the least residue of -1 times its size.
    let types = (2..66_u32)
        .map(|modulus| format!("  type T{modulus} = Mod[{modulus}];\n"))
        .collect::<String>();
    let listed = |last: u32| {
        (2..=last)
            .map(|modulus| format!("T{modulus}"))
            .collect::<Vec<_>>()
            .join(", ")
    };
    let program = format!(
        "edition 2026;\nmodule limits {{\n{types}  \
         spec most[K in {{{}}}, n in 1..5]() -> Int {{\n    \
         let minus_one: K = 0 - 1;\n    (minus_one as Int) * n\n  }}\n  \
         spec last() -> Int {{ most[T65, 4]() }}\n}}\n",
        listed(65)
    );
    let mut expected = String::new();
    for modulus in 2..66_u32 {
        for size in 1..5_u32 {
            writeln!(
                expected,
                "limits::most[T{modulus}, {size}]: Int = {}",
                (modulus - 1) * size
            )
            .unwrap();
        }
    }
    expected.push_str("limits::last: Int = 256\n");
    let path = directory.join("most.or");
    fs::write(&path, &program).unwrap();
    let most = run_twice("eval", &path, "256 instances");
    assert_success(&most, &expected, "256 instances");

    // One type more, `Int` beside the 64 declared types a module may have,
    // is 260 instances, reported from the first parameter in brackets
    // through the last.
    let over = format!(
        "edition 2026;\nmodule limits {{\n{types}  \
         spec over[K in {{{}, Int}}, n in 1..5]() -> Int {{ n }}\n}}\n",
        listed(65)
    );
    let line = over
        .lines()
        .position(|text| text.contains("spec over"))
        .unwrap()
        + 1;
    let path = directory.join("over.or");
    fs::write(&path, &over).unwrap();
    let rejected = run_twice("check", &path, "260 instances");
    assert_failure(
        &rejected,
        &["ORC0238"],
        &[format!("over.or:{line}:13").as_str()],
        &[
            "`over` has 260 instances, but a function has at most 256",
            "a function has one instance for each combination of its sizes' values and its type \
             parameters' types, at most 256 in all",
        ],
        "260 instances",
    );
    fs::remove_dir_all(&directory).unwrap();
}
