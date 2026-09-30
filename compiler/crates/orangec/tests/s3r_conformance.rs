//! External conformance evidence for the proposed Orange 2026 S3r slice.
//!
//! The rule index lives in `docs/AMOUNTS_2026.md`. This runner requires that
//! index to agree exactly with the evidence map below, runs every fixture and
//! every generated program through the real `orangec` binary twice, and
//! checks that every named unit test is declared once in its source's test
//! module. S3q compatibility is also observed by the S2 through S3q runners,
//! which run their fixtures through the same binary.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const AMOUNTS_SPECIFICATION: &str = include_str!("../../../../docs/AMOUNTS_2026.md");
const S3R_CONFORMANCE_SOURCE: &str = include_str!("s3r_conformance.rs");
const SEMANTICS_SOURCE: &str = include_str!("../../orange-compiler/src/semantics.rs");
const SEMANTICS_TESTS_SOURCE: &str = include_str!("../../orange-compiler/src/semantics/tests.rs");
const CORE_SOURCE: &str = include_str!("../../orange-compiler/src/core.rs");
const EVAL_SOURCE: &str = include_str!("../../orange-compiler/src/eval.rs");

const CLI: u8 = 1 << 0;
const GENERATED_CLI: u8 = 1 << 1;
const UNIT: u8 = 1 << 2;

/// Every S3r rule, in the order of the specification's index.
const RULES: [&str; 10] = [
    "S3R-FORM-01",
    "S3R-TYPE-01",
    "S3R-SHIFT-01",
    "S3R-ROTATE-01",
    "S3R-RANGE-01",
    "S3R-CORE-01",
    "S3R-COST-01",
    "S3R-RES-01",
    "S3R-COMPAT-01",
    "S3R-DETERMINISM-01",
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

/// The operators whose amounts S3r computes, with the names generated
/// programs give them.
const OPERATORS: [(&str, &str); 4] = [
    ("<<", "shl"),
    (">>", "shr"),
    ("<<<", "rotl"),
    (">>>", "rotr"),
];

const WIDTHS: [u32; 4] = [8, 16, 32, 64];

/// The words the generated programs shift: the high bits of one pattern, so
/// that each width has its top and bottom bits set and a mixed middle.
fn pattern(bits: u32) -> u64 {
    0x9b3c_5a1e_d2f0_8765_u64 >> (64 - bits)
}

fn mask(bits: u32) -> u64 {
    u64::MAX >> (64 - bits)
}

/// The definition of `a OP k` for a word `a` of `bits` bits: `<<` is
/// floor(a * 2^k) modulo 2^n and `>>` is floor(a * 2^-k) modulo 2^n, and a
/// rotation turns by k modulo n, `>>>` by k as `<<<` by -k.
fn reference(operator: &str, bits: u32, value: u64, amount: i128) -> u64 {
    let n = i128::from(bits);
    let left = |k: i128| {
        if k >= n { 0 } else { (value << k) & mask(bits) }
    };
    let right = |k: i128| if k >= n { 0 } else { value >> k };
    let rotate = |k: i128| {
        let turn = u32::try_from(k.rem_euclid(n)).unwrap();
        if turn == 0 {
            value
        } else {
            ((value << turn) | (value >> (bits - turn))) & mask(bits)
        }
    };
    match operator {
        "<<" if amount >= 0 => left(amount),
        "<<" => right(-amount),
        ">>" if amount >= 0 => right(amount),
        ">>" => left(-amount),
        "<<<" => rotate(amount),
        ">>>" => rotate(-amount),
        _ => unreachable!("{operator}"),
    }
}

/// A word as `orangec` displays one of `bits` bits.
fn word(bits: u32, value: u64) -> String {
    let digits = usize::try_from(bits / 4).unwrap();
    format!("0x{value:0digits$x}")
}

fn words(bits: u32, values: impl IntoIterator<Item = u64>) -> String {
    let shown = values
        .into_iter()
        .map(|value| word(bits, value))
        .collect::<Vec<_>>();
    format!("[{}]", shown.join(", "))
}

fn nothing() -> String {
    String::new()
}

fn amounts_eval_stdout() -> String {
    // Every shift and rotation of 0x96 by -9 through 9, as `table` builds
    // them, and the reflections of four bytes.
    let row = |operator| words(8, (-9..=9).map(|k| reference(operator, 8, 0x96, k)));
    let table = format!(
        "({}, {}, {}, {})",
        row("<<"),
        row(">>"),
        row("<<<"),
        row(">>>")
    );
    let ty = "(Word[8]^19, Word[8]^19, Word[8]^19, Word[8]^19)";
    let reflected = words(
        8,
        [0x01_u8, 0x96, 0xe0, 0xff].map(|x| u64::from(x.reverse_bits())),
    );
    format!(
        "amounts::table: {ty} = {table}\namounts::shifts: {ty} = {table}\n\
         amounts::reflected: Word[8]^4 = {reflected}\n"
    )
}

fn amounts_test_stdout() -> String {
    String::from(concat!(
        "test \"within the width, computed amounts agree with literals\" ... ok\n",
        "test \"a shift by the width or more gives 0\" ... ok\n",
        "test \"a negative amount shifts the other way\" ... ok\n",
        "test \"a rotation turns by the amount modulo the width\" ... ok\n",
        "test \"an amount of any size\" ... ok\n",
        "test \"reflect reverses the bits of a byte\" ... ok\n",
        "test \"the key of RFC 7748 section 5.2's first vector has 114 set bits\" ... ok\n",
        "test \"a nibble at a computed position selects from a table\" ... ok\n",
        "test \"a byte turns a 64-bit word by its value modulo 64\" ... ok\n",
        "test \"the top n bits of a byte\" ... ok\n",
        "10 tests: 10 passed, 0 failed\n",
    ))
}

fn rc6_eval_stdout() -> String {
    String::from(concat!(
        "rc6::zero_key_zero_block: Word[8]^16 = [0x8f, 0xc3, 0xa5, 0x36, 0x56, 0xb1, 0xf7, ",
        "0x78, 0xc1, 0x29, 0xdf, 0x4e, 0x98, 0x48, 0xa4, 0x1e]\n",
    ))
}

fn rc6_test_stdout() -> String {
    String::from(concat!(
        "test \"RC6 paper, 128-bit key 1: encryption\" ... ok\n",
        "test \"RC6 paper, 128-bit key 1: decryption\" ... ok\n",
        "test \"RC6 paper, 128-bit key 2: encryption\" ... ok\n",
        "test \"RC6 paper, 128-bit key 2: decryption\" ... ok\n",
        "4 tests: 4 passed, 0 failed\n",
    ))
}

/// The round constants of Keccak-p[1600, 24] as FIPS 202's references
/// print them, and the SHA3-256 digest of "abc".
fn sha3_eval_stdout() -> String {
    const ROUND_CONSTANTS: [u64; 24] = [
        0x0000_0000_0000_0001,
        0x0000_0000_0000_8082,
        0x8000_0000_0000_808a,
        0x8000_0000_8000_8000,
        0x0000_0000_0000_808b,
        0x0000_0000_8000_0001,
        0x8000_0000_8000_8081,
        0x8000_0000_0000_8009,
        0x0000_0000_0000_008a,
        0x0000_0000_0000_0088,
        0x0000_0000_8000_8009,
        0x0000_0000_8000_000a,
        0x0000_0000_8000_808b,
        0x8000_0000_0000_008b,
        0x8000_0000_0000_8089,
        0x8000_0000_0000_8003,
        0x8000_0000_0000_8002,
        0x8000_0000_0000_0080,
        0x0000_0000_0000_800a,
        0x8000_0000_8000_000a,
        0x8000_0000_8000_8081,
        0x8000_0000_0000_8080,
        0x0000_0000_8000_0001,
        0x8000_0000_8000_8008,
    ];
    const ABC: [u8; 32] = [
        0x3a, 0x98, 0x5d, 0xa7, 0x4f, 0xe2, 0x25, 0xb2, 0x04, 0x5c, 0x17, 0x2d, 0x6b, 0xd3, 0x90,
        0xbd, 0x85, 0x5f, 0x08, 0x6e, 0x3e, 0x9d, 0x52, 0x5b, 0x46, 0xbf, 0xe2, 0x45, 0x11, 0x43,
        0x15, 0x32,
    ];
    format!(
        "sha3::round_constants: Word[64]^24 = {}\nsha3::abc: Word[8]^32 = {}\n",
        words(64, ROUND_CONSTANTS),
        words(8, ABC.map(u64::from)),
    )
}

fn sha3_test_stdout() -> String {
    String::from(concat!(
        "test \"FIPS 202 example: SHA3-256 of abc\" ... ok\n",
        "test \"FIPS 202 example: SHA3-256 of the 448-bit message\" ... ok\n",
        "test \"133 bytes of a, the longest message of one block\" ... ok\n",
        "test \"the round constants of iota\" ... ok\n",
        "4 tests: 4 passed, 0 failed\n",
    ))
}

/// ML-KEM's zetas, derived here as FIPS 203 section 4.3 defines them:
/// 17^BitRev7(i) and 17^(2 BitRev7(i) + 1) modulo 3329.
fn zetas_eval_stdout() -> String {
    let power = |exponent: u32| (0..exponent).fold(1_u32, |product, _| product * 17 % 3329);
    let reversed = |i: u8| u32::from(i.reverse_bits() >> 1);
    let residues = |values: Vec<u32>| {
        let shown = values
            .iter()
            .map(u32::to_string)
            .collect::<Vec<_>>()
            .join(", ");
        format!("[{shown}]")
    };
    let ntt = (0..128_u8).map(|i| power(reversed(i))).collect::<Vec<_>>();
    let base = (0..128_u8)
        .map(|i| power(2 * reversed(i) + 1))
        .collect::<Vec<_>>();
    format!(
        "zetas::ntt_zetas: Mod[3329]^128 = {}\nzetas::base_zetas: Mod[3329]^128 = {}\n\
         zetas::first_row: Mod[3329]^16 = {}\n",
        residues(ntt.clone()),
        residues(base),
        residues(ntt[..16].to_vec()),
    )
}

fn zetas_test_stdout() -> String {
    String::from(concat!(
        "test \"BitRev7 turns 1 into 64 and 0b0000011 into 0b1100000\" ... ok\n",
        "test \"Appendix A: the first sixteen of the NTT's zetas\" ... ok\n",
        "test \"Appendix A: the first eight of MultiplyNTTs' zetas\" ... ok\n",
        "test \"zeta is a primitive 256th root of unity\" ... ok\n",
        "4 tests: 4 passed, 0 failed\n",
    ))
}

const CLEAN_CHECK: Run = Run {
    arguments: &["check"],
    expectation: Expectation::Exact {
        status: 0,
        stdout: nothing,
        stderr: "",
    },
};

const INVALID_AMOUNTS: Expectation = Expectation::Failure {
    codes: &[
        "ORC0216", "ORC0216", "ORC0214", "ORC0214", "ORC0214", "ORC0214", "ORC0215", "ORC0211",
        "ORC0223",
    ],
    locations: &[
        "invalid-amounts.or:7:48",
        "invalid-amounts.or:8:48",
        "invalid-amounts.or:9:55",
        "invalid-amounts.or:10:60",
        "invalid-amounts.or:11:66",
        "invalid-amounts.or:12:59",
        "invalid-amounts.or:13:42",
        "invalid-amounts.or:14:49",
        "invalid-amounts.or:15:63",
    ],
    messages: &[
        "error[ORC0216]: `<<` on `Word[32]` needs an amount from 0 through 31\n",
        "error[ORC0216]: `>>>` on `Word[32]` needs an amount from 0 through 31\n",
        "^^ a literal amount is from 0 through 31\n",
        "= note: an amount written as one integer literal is from 0 through n - 1; any other \
         amount is computed, an `Int` or a word, such as `x <<< r` or `x >> (i % 8)`\n",
        "error[ORC0214]: `b` has type `Bool`, but `Int` is required here\n",
        "error[ORC0214]: `r` has type `Mod[5]`, but `Int` is required here\n",
        "error[ORC0214]: `a` has type `Word[8]^4`, but `Int` is required here\n",
        "error[ORC0214]: a comparison gives `Bool`, but `Int` is required here\n",
        "error[ORC0215]: `<<` is not defined for `Int`\n",
        "= note: shifts and rotations apply only to `Word[n]` values\n",
        "error[ORC0211]: `y` is not a parameter of `unknown`\n",
        "error[ORC0223]: this index runs from 0 through 255, out of range for `Word[8]^16`\n",
    ],
};

const INVALID_AMOUNT_GROUPING: Expectation = Expectation::Failure {
    codes: &["ORC0108"],
    locations: &["invalid-amount-grouping.or:6:55"],
    messages: &["error[ORC0108]: `+` follows `<<` without grouping parentheses\n"],
};

const CASES: [Case; 6] = [
    Case {
        fixture: "valid-amounts.or",
        runs: &[
            CLEAN_CHECK,
            Run {
                arguments: &["test"],
                expectation: Expectation::Exact {
                    status: 0,
                    stdout: amounts_test_stdout,
                    stderr: "",
                },
            },
            Run {
                arguments: &["eval"],
                expectation: Expectation::Exact {
                    status: 0,
                    stdout: amounts_eval_stdout,
                    stderr: "",
                },
            },
        ],
        rules: &[
            "S3R-FORM-01",
            "S3R-TYPE-01",
            "S3R-SHIFT-01",
            "S3R-ROTATE-01",
            "S3R-RANGE-01",
            "S3R-COMPAT-01",
            "S3R-DETERMINISM-01",
        ],
    },
    Case {
        fixture: "valid-rc6.or",
        runs: &[
            CLEAN_CHECK,
            Run {
                arguments: &["test", "--stats"],
                expectation: Expectation::Exact {
                    status: 0,
                    stdout: rc6_test_stdout,
                    stderr: concat!(
                        "test \"RC6 paper, 128-bit key 1: encryption\": 8910 steps\n",
                        "test \"RC6 paper, 128-bit key 1: decryption\": 8968 steps\n",
                        "test \"RC6 paper, 128-bit key 2: encryption\": 8908 steps\n",
                        "test \"RC6 paper, 128-bit key 2: decryption\": 8966 steps\n",
                        "total: 35752 of 1048576 steps\n",
                    ),
                },
            },
            Run {
                arguments: &["eval"],
                expectation: Expectation::Exact {
                    status: 0,
                    stdout: rc6_eval_stdout,
                    stderr: "",
                },
            },
        ],
        rules: &[
            "S3R-TYPE-01",
            "S3R-ROTATE-01",
            "S3R-COST-01",
            "S3R-DETERMINISM-01",
        ],
    },
    Case {
        fixture: "valid-sha3.or",
        runs: &[
            CLEAN_CHECK,
            Run {
                arguments: &["test", "--stats"],
                expectation: Expectation::Exact {
                    status: 0,
                    stdout: sha3_test_stdout,
                    stderr: concat!(
                        "test \"FIPS 202 example: SHA3-256 of abc\": 109318 steps\n",
                        "test \"FIPS 202 example: SHA3-256 of the 448-bit message\": 109317 \
                         steps\n",
                        "test \"133 bytes of a, the longest message of one block\": 109321 \
                         steps\n",
                        "test \"the round constants of iota\": 6924 steps\n",
                        "total: 334880 of 1048576 steps\n",
                    ),
                },
            },
            Run {
                arguments: &["eval"],
                expectation: Expectation::Exact {
                    status: 0,
                    stdout: sha3_eval_stdout,
                    stderr: "",
                },
            },
        ],
        rules: &[
            "S3R-TYPE-01",
            "S3R-SHIFT-01",
            "S3R-ROTATE-01",
            "S3R-COST-01",
            "S3R-DETERMINISM-01",
        ],
    },
    Case {
        fixture: "valid-zetas.or",
        runs: &[
            CLEAN_CHECK,
            Run {
                arguments: &["test"],
                expectation: Expectation::Exact {
                    status: 0,
                    stdout: zetas_test_stdout,
                    stderr: "",
                },
            },
            Run {
                arguments: &["eval"],
                expectation: Expectation::Exact {
                    status: 0,
                    stdout: zetas_eval_stdout,
                    stderr: "",
                },
            },
        ],
        rules: &["S3R-TYPE-01", "S3R-SHIFT-01", "S3R-DETERMINISM-01"],
    },
    Case {
        fixture: "invalid-amounts.or",
        runs: &[
            Run {
                arguments: &["check"],
                expectation: INVALID_AMOUNTS,
            },
            Run {
                arguments: &["test"],
                expectation: INVALID_AMOUNTS,
            },
            Run {
                arguments: &["eval"],
                expectation: INVALID_AMOUNTS,
            },
        ],
        rules: &[
            "S3R-FORM-01",
            "S3R-TYPE-01",
            "S3R-RANGE-01",
            "S3R-COMPAT-01",
            "S3R-DETERMINISM-01",
        ],
    },
    Case {
        fixture: "invalid-amount-grouping.or",
        runs: &[
            Run {
                arguments: &["check"],
                expectation: INVALID_AMOUNT_GROUPING,
            },
            Run {
                arguments: &["test"],
                expectation: INVALID_AMOUNT_GROUPING,
            },
        ],
        rules: &["S3R-FORM-01", "S3R-COMPAT-01", "S3R-DETERMINISM-01"],
    },
];

/// Generated command-line cases in this file, each run twice.
const GENERATED_EVIDENCE: &[(&str, &[&str])] = &[
    (
        "s3r_amounts_agree_with_the_definition_at_every_width",
        &[
            "S3R-TYPE-01",
            "S3R-SHIFT-01",
            "S3R-ROTATE-01",
            "S3R-COMPAT-01",
            "S3R-DETERMINISM-01",
        ],
    ),
    (
        "s3r_an_amount_costs_one_step_whatever_its_size",
        &["S3R-COST-01", "S3R-RES-01", "S3R-DETERMINISM-01"],
    ),
    (
        "s3r_literal_amounts_stay_below_every_width",
        &["S3R-FORM-01", "S3R-COMPAT-01", "S3R-DETERMINISM-01"],
    ),
];

const UNIT_EVIDENCE: &[TestEvidence] = &[
    TestEvidence {
        source_path: "src/semantics/tests.rs",
        test: "literal_shift_and_rotation_amounts_are_below_the_width",
        rules: &["S3R-FORM-01", "S3R-COMPAT-01"],
    },
    TestEvidence {
        source_path: "src/semantics/tests.rs",
        test: "computed_amounts_take_the_type_of_their_first_typed_leaf",
        rules: &[
            "S3R-FORM-01",
            "S3R-TYPE-01",
            "S3R-CORE-01",
            "S3R-DETERMINISM-01",
        ],
    },
    TestEvidence {
        source_path: "src/semantics/tests.rs",
        test: "shifts_by_computed_amounts_range_over_their_whole_type",
        rules: &["S3R-RANGE-01"],
    },
    TestEvidence {
        source_path: "src/core.rs",
        test: "core_accessors_preserve_source_order_and_derive_value_types",
        rules: &["S3R-CORE-01"],
    },
    TestEvidence {
        source_path: "src/eval.rs",
        test: "computed_shifts_and_rotations_follow_their_definition_at_every_width",
        rules: &["S3R-SHIFT-01", "S3R-ROTATE-01", "S3R-RES-01"],
    },
    TestEvidence {
        source_path: "src/eval.rs",
        test: "a_computed_amount_costs_one_step_whatever_its_size",
        rules: &["S3R-COST-01", "S3R-RES-01"],
    },
    TestEvidence {
        source_path: "src/eval.rs",
        test: "computed_amount_nodes_fail_closed_on_inconsistent_core",
        rules: &["S3R-CORE-01", "S3R-RES-01"],
    },
    TestEvidence {
        source_path: "src/eval.rs",
        test: "steps_follow_the_normative_cost_table",
        rules: &["S3R-COST-01", "S3R-COMPAT-01"],
    },
];

fn orangec() -> Command {
    Command::new(env!("CARGO_BIN_EXE_orangec"))
}

fn fixture_directory() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/s3r")
}

fn run(arguments: &[&str], path: &Path) -> Output {
    orangec().args(arguments).arg(path).output().unwrap()
}

/// Runs `orangec` with `arguments` on `path` twice and returns the first
/// output after requiring the second to be byte-identical.
fn run_twice(arguments: &[&str], path: &Path, context: &str) -> Output {
    let first = run(arguments, path);
    let second = run(arguments, path);
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

/// A fresh directory for one generated program.
fn scratch_directory(name: &str) -> PathBuf {
    let directory = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("orangec-s3r-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&directory);
    fs::create_dir(&directory).unwrap();
    directory
}

/// The rule index rows of the specification: rule ID and evidence label.
fn documented_rules() -> Vec<(&'static str, &'static str)> {
    AMOUNTS_SPECIFICATION
        .lines()
        .filter_map(|line| {
            let row = line.strip_prefix("| `")?;
            let (rule, _) = row.split_once("` |")?;
            if !rule.starts_with("S3R-") {
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
        "Generated CLI and unit" => GENERATED_CLI | UNIT,
        "CLI, generated CLI, and unit" => CLI | GENERATED_CLI | UNIT,
        "Unit" => UNIT,
        _ => panic!("unknown S3r evidence layer {label:?}"),
    }
}

fn unit_source(source_path: &str) -> &'static str {
    match source_path {
        "src/semantics.rs" => SEMANTICS_SOURCE,
        "src/semantics/tests.rs" => SEMANTICS_TESTS_SOURCE,
        "src/core.rs" => CORE_SOURCE,
        "src/eval.rs" => EVAL_SOURCE,
        _ => panic!("unmapped S3r evidence source {source_path}"),
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
        S3R_CONFORMANCE_SOURCE.matches(&declaration).count(),
        1,
        "generated evidence {test} must be declared exactly once at file root"
    );
}

#[test]
fn s3r_rule_index_is_exact_and_covered() {
    let documented = documented_rules();
    let documented_ids: Vec<_> = documented.iter().map(|(rule, _)| *rule).collect();
    assert_eq!(
        documented_ids, RULES,
        "docs/AMOUNTS_2026.md rule index drifted from the S3r runner"
    );
    let known: BTreeSet<_> = RULES.iter().copied().collect();
    assert_eq!(known.len(), RULES.len(), "duplicate S3r rule ID");

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
        for rule in evidence.rules {
            record(rule, UNIT, evidence.test);
        }
    }

    for (rule, label) in documented {
        let required = required_layers(label);
        let actual = observed.get(rule).copied().unwrap_or_default();
        assert_eq!(
            actual & required,
            required,
            "{rule} requires {label}, but its evidence map provides layers {actual:#05b}"
        );
    }
}

#[test]
fn s3r_cli_conformance_corpus_is_exact_and_repeatable() {
    let directory = fixture_directory();
    let mut observed: Vec<_> = fs::read_dir(&directory)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().into_string().unwrap())
        .collect();
    observed.sort();
    let mut expected: Vec<_> = CASES.iter().map(|case| case.fixture).collect();
    expected.sort_unstable();
    assert_eq!(observed, expected, "unexpected S3r fixture inventory");

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

/// Amounts far outside every width, each given with and without a sign:
/// around 2^63 and 2^64, where an amount leaves 64 bits, and two that no
/// word of 64 bits holds.
const LARGE_AMOUNTS: [i128; 6] = [
    1 << 63,
    (1 << 64) - 1,
    1 << 64,
    (1 << 64) + 1,
    (1 << 126) + 3,
    i128::MAX,
];

#[test]
fn s3r_amounts_agree_with_the_definition_at_every_width() {
    let directory = scratch_directory("definition");
    let mut program = String::from("edition 2026;\nmodule amounts {\n");
    let mut expected = String::new();
    for bits in WIDTHS {
        let ty = format!("Word[{bits}]");
        let value = pattern(bits);
        let shown = word(bits, value);
        for (operator, name) in OPERATORS {
            // The operator as a function of an `Int` amount and of an
            // amount of each word width.
            writeln!(
                program,
                "  spec {name}{bits}(a: {ty}, k: Int) -> {ty} {{ a {operator} k }}"
            )
            .unwrap();
            for amount_bits in WIDTHS {
                writeln!(
                    program,
                    "  spec {name}{bits}_w{amount_bits}(a: {ty}, k: Word[{amount_bits}]) -> {ty} \
                     {{ a {operator} k }}"
                )
                .unwrap();
            }

            // Every `Int` amount from -(2n + 1) through 2n + 1.
            let reach = i128::from(2 * bits + 1);
            let count = 2 * reach + 1;
            writeln!(
                program,
                "  spec int_{name}{bits}() -> {ty}^{count} {{\n    for i in 0..{count} with \
                 r: {ty}^{count} = [0; {count}] {{ r with [i] = {name}{bits}({shown}, i - \
                 {reach}) }}\n  }}"
            )
            .unwrap();
            let values = (-reach..=reach).map(|k| reference(operator, bits, value, k));
            writeln!(
                expected,
                "amounts::int_{name}{bits}: {ty}^{count} = {}",
                words(bits, values)
            )
            .unwrap();

            // Amounts past every width, as arguments written with and
            // without a sign.
            let arguments = LARGE_AMOUNTS
                .iter()
                .flat_map(|amount| [format!("{amount:#x}"), format!("-{amount:#x}")])
                .map(|amount| format!("{name}{bits}({shown}, {amount})"))
                .collect::<Vec<_>>();
            let large = 2 * LARGE_AMOUNTS.len();
            writeln!(
                program,
                "  spec large_{name}{bits}() -> {ty}^{large} {{ [{}] }}",
                arguments.join(", ")
            )
            .unwrap();
            let values = LARGE_AMOUNTS
                .iter()
                .flat_map(|amount| [*amount, -amount])
                .map(|k| reference(operator, bits, value, k));
            writeln!(
                expected,
                "amounts::large_{name}{bits}: {ty}^{large} = {}",
                words(bits, values)
            )
            .unwrap();

            // Word amounts 0 through 2n + 1, and their negations modulo
            // 2^m, which are the largest amounts of each width.
            let reach = 2 * bits + 2;
            for amount_bits in WIDTHS {
                for (direction, negated) in [("up", false), ("down", true)] {
                    let amount = if negated {
                        format!("0 - (i as Word[{amount_bits}])")
                    } else {
                        format!("i as Word[{amount_bits}]")
                    };
                    writeln!(
                        program,
                        "  spec {direction}_{name}{bits}_w{amount_bits}() -> {ty}^{reach} {{\n    \
                         for i in 0..{reach} with r: {ty}^{reach} = [0; {reach}] {{ r with [i] = \
                         {name}{bits}_w{amount_bits}({shown}, {amount}) }}\n  }}"
                    )
                    .unwrap();
                    let modulus = 1_i128 << amount_bits;
                    let values = (0..i128::from(reach)).map(|i| {
                        let k = if negated { (modulus - i) % modulus } else { i };
                        reference(operator, bits, value, k)
                    });
                    writeln!(
                        expected,
                        "amounts::{direction}_{name}{bits}_w{amount_bits}: {ty}^{reach} = {}",
                        words(bits, values)
                    )
                    .unwrap();
                }
            }

            // Each amount a literal may be, 0 through n - 1, agrees with
            // the literal form.
            let literals = (0..bits)
                .map(|k| format!("({shown} {operator} {k}) == {name}{bits}({shown}, {k})"))
                .collect::<Vec<_>>();
            writeln!(
                program,
                "  spec literal_{name}{bits}() -> Bool^{bits} {{ [{}] }}",
                literals.join(", ")
            )
            .unwrap();
            let truths = vec!["true"; usize::try_from(bits).unwrap()];
            writeln!(
                expected,
                "amounts::literal_{name}{bits}: Bool^{bits} = [{}]",
                truths.join(", ")
            )
            .unwrap();
        }
    }
    program.push_str("}\n");
    let path = directory.join("amounts.or");
    fs::write(&path, &program).unwrap();
    let output = run_twice(&["eval"], &path, "every width");
    assert_exact(&output, 0, &expected, "", "every width");
    fs::remove_dir_all(&directory).unwrap();
}

#[test]
fn s3r_an_amount_costs_one_step_whatever_its_size() {
    let directory = scratch_directory("cost");
    // Amounts of 2 bits, 65 bits, 201 bits, and 16,384 bits, the most an
    // `Int` holds, each with and without a sign.
    let amounts = [
        String::from("3"),
        format!("{:#x}", (1_u128 << 64) + 3),
        format!("0x1{}3", "0".repeat(49)),
        format!("0x8{}3", "0".repeat(4094)),
    ];
    let mut program = String::from(concat!(
        "edition 2026;\n",
        "module cost {\n",
        "  spec shl(x: Word[64], k: Int) -> Word[64] { x << k }\n",
        "  spec rotr(x: Word[64], k: Word[64]) -> Word[64] { x >>> k }\n",
        "  spec turn(x: Word[64], k: Int) -> Word[64] { x <<< k }\n",
    ));
    let mut values = String::new();
    let mut steps = String::new();
    let mut total = 0;
    for (index, amount) in amounts.iter().enumerate() {
        for (sign, negative) in [("", false), ("-", true)] {
            let name = format!("a{index}{}", if negative { "_negative" } else { "" });
            writeln!(
                program,
                "  spec {name}() -> (Word[64], Word[64]) {{ (shl(0x9b, {sign}{amount}), \
                 turn(0x9b, {sign}{amount})) }}"
            )
            .unwrap();
            // Every amount here is 3 modulo 64, and all but the first have
            // at least 64 bits.
            let (shifted, turned) = match (index, negative) {
                (0, false) => (0x9b << 3, 0x9b_u64.rotate_left(3)),
                (0, true) => (0x9b >> 3, 0x9b_u64.rotate_right(3)),
                (_, false) => (0, 0x9b_u64.rotate_left(3)),
                (_, true) => (0, 0x9b_u64.rotate_right(3)),
            };
            writeln!(
                values,
                "cost::{name}: (Word[64], Word[64]) = ({}, {})",
                word(64, shifted),
                word(64, turned)
            )
            .unwrap();
            // Two calls and a tuple cost the same steps whatever the amount.
            writeln!(steps, "cost::{name}: 14 steps").unwrap();
            total += 14;
        }
    }
    // A word amount costs what an `Int` amount costs.
    program.push_str(
        "  spec word_amount() -> Word[64] { rotr(0x9b, 0xffffffffffffffff) }\n\
         \x20 spec int_amount() -> Word[64] { turn(0x9b, 0xffffffffffffffff) }\n}\n",
    );
    writeln!(
        values,
        "cost::word_amount: Word[64] = {}\ncost::int_amount: Word[64] = {}",
        word(64, 0x9b_u64.rotate_left(1)),
        word(64, 0x9b_u64.rotate_left(63))
    )
    .unwrap();
    write!(
        steps,
        "cost::word_amount: 6 steps\ncost::int_amount: 6 steps\ntotal: {} of 1048576 steps\n",
        total + 12
    )
    .unwrap();
    let path = directory.join("cost.or");
    fs::write(&path, &program).unwrap();
    let output = run_twice(&["eval", "--stats"], &path, "amounts of every size");
    assert_exact(&output, 0, &values, &steps, "amounts of every size");
    fs::remove_dir_all(&directory).unwrap();
}

#[test]
fn s3r_literal_amounts_stay_below_every_width() {
    let directory = scratch_directory("literals");
    let path = directory.join("literals.or");
    // At each width, each operator's amount written as the width, and as
    // a literal with a sign, is refused where it is written.
    let mut program = String::from("edition 2026;\nmodule literals {\n");
    let mut codes = Vec::new();
    let mut locations = Vec::new();
    let mut messages = Vec::new();
    let mut line = 2;
    for bits in WIDTHS {
        for (operator, name) in OPERATORS {
            for (suffix, amount) in [("width", bits.to_string()), ("signed", String::from("-0"))] {
                let prefix = format!(
                    "  spec {name}{bits}_{suffix}(x: Word[{bits}]) -> Word[{bits}] {{ x {operator} "
                );
                writeln!(program, "{prefix}{amount} }}").unwrap();
                line += 1;
                codes.push("ORC0216");
                locations.push(format!("literals.or:{line}:{}", prefix.len() + 1));
                messages.push(format!(
                    "error[ORC0216]: `{operator}` on `Word[{bits}]` needs an amount from 0 through \
                     {}\n",
                    bits - 1
                ));
            }
        }
    }
    program.push_str("}\n");
    fs::write(&path, &program).unwrap();
    let rejected = run_twice(&["check"], &path, "literal amounts");
    let messages = messages.iter().map(String::as_str).collect::<Vec<_>>();
    let locations = locations.iter().map(String::as_str).collect::<Vec<_>>();
    assert_failure(&rejected, &codes, &locations, &messages, "literal amounts");

    // Written as anything other than one literal, the same amounts are
    // computed: grouped, the width shifts every bit out and -0 is 0.
    let mut program = String::from("edition 2026;\nmodule computed {\n");
    let mut expected = String::new();
    for bits in WIDTHS {
        let value = pattern(bits);
        let shown = word(bits, value);
        let results = OPERATORS
            .iter()
            .flat_map(|(operator, _)| {
                [
                    format!("{shown} {operator} ({bits})"),
                    format!("{shown} {operator} (-0)"),
                ]
            })
            .collect::<Vec<_>>();
        writeln!(
            program,
            "  spec grouped{bits}() -> Word[{bits}]^8 {{ [{}] }}",
            results.join(", ")
        )
        .unwrap();
        let values = OPERATORS.iter().flat_map(|(operator, _)| {
            [
                reference(operator, bits, value, i128::from(bits)),
                reference(operator, bits, value, 0),
            ]
        });
        writeln!(
            expected,
            "computed::grouped{bits}: Word[{bits}]^8 = {}",
            words(bits, values)
        )
        .unwrap();
    }
    program.push_str("}\n");
    fs::write(&path, &program).unwrap();
    let computed = run_twice(&["eval"], &path, "grouped amounts");
    assert_exact(&computed, 0, &expected, "", "grouped amounts");
    fs::remove_dir_all(&directory).unwrap();
}
