//! External conformance evidence for the proposed Orange 2026 S3p slice.
//!
//! The rule index lives in `docs/LENGTHS_2026.md`. This runner requires that
//! index to agree exactly with the evidence map below, runs every fixture and
//! every generated program through the real `orangec` binary twice, and
//! checks that every named unit test is declared once in its source's test
//! module. S3o compatibility is also observed by the S2 through S3o runners,
//! which run their fixtures through the same binary.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const LENGTHS_SPECIFICATION: &str = include_str!("../../../../docs/LENGTHS_2026.md");
const S3P_CONFORMANCE_SOURCE: &str = include_str!("s3p_conformance.rs");
const PARSER_SOURCE: &str = include_str!("../../orange-compiler/src/parser.rs");
const SEMANTICS_SOURCE: &str = include_str!("../../orange-compiler/src/semantics.rs");
const SEMANTICS_TESTS_SOURCE: &str = include_str!("../../orange-compiler/src/semantics/tests.rs");
const CORE_SOURCE: &str = include_str!("../../orange-compiler/src/core.rs");
const EVAL_SOURCE: &str = include_str!("../../orange-compiler/src/eval.rs");
const ORANGEC_SOURCE: &str = include_str!("../src/main.rs");
const CRYPT_SOURCE: &str = include_str!("../src/crypt.rs");

const CLI: u8 = 1 << 0;
const GENERATED_CLI: u8 = 1 << 1;
const UNIT: u8 = 1 << 2;
const PARSER_UNIT: u8 = 1 << 3;

/// Every S3p rule, in the order of the specification's index.
const RULES: [&str; 11] = [
    "S3P-LENGTH-01",
    "S3P-LITERAL-01",
    "S3P-INDEX-01",
    "S3P-ORDER-01",
    "S3P-COST-01",
    "S3P-STEPS-01",
    "S3P-SPEC-01",
    "S3P-STATS-01",
    "S3P-RES-01",
    "S3P-COMPAT-01",
    "S3P-DETERMINISM-01",
];

#[derive(Clone, Copy)]
enum Expectation {
    /// The exact standard output of `eval`, and its exact standard error.
    Success {
        stdout: fn() -> String,
        stderr: &'static str,
    },
    Failure {
        codes: &'static [&'static str],
        locations: &'static [&'static str],
        messages: &'static [&'static str],
    },
}

#[derive(Clone, Copy)]
struct Case {
    fixture: &'static str,
    /// Options given to `eval` before the fixture.
    eval_options: &'static [&'static str],
    expectation: Expectation,
    rules: &'static [&'static str],
}

#[derive(Clone, Copy)]
struct TestEvidence {
    source_path: &'static str,
    test: &'static str,
    rules: &'static [&'static str],
}

// RFC 8439 as printed, from the D-011 suite's pinned vectors
// (`tools/d011_suite.py`): the text of appendix A.2 test vector 2, its
// ciphertext (K-CHACHA-09), the tags of A.3 test vectors 2 and 3 (K-POLY-03
// and K-POLY-04), and the inputs, ciphertext, tag and plaintext of A.5
// (K-AEAD-03).
const IETF_TEXT: &str = concat!(
    "Any submission to the IETF intended by the Contributor for publication as all or part of ",
    "an IETF Internet-Draft or RFC and any statement made within the context of an IETF ",
    "activity is considered an \"IETF Contribution\". Such statements include oral statements ",
    "in IETF sessions, as well as written and electronic communications made at any time or ",
    "place, which are addressed to",
);
const A2_2_CIPHERTEXT: &str = concat!(
    "a3fbf07df3fa2fde4f376ca23e82737041605d9f4f4f57bd8cff2c1d4b7955ec2a97948bd3722915c8f3d337f7",
    "d370050e9e96d647b7c39f56e031ca5eb6250d4042e02785ececfa4b4bb5e8ead0440e20b6e8db09d881a7c613",
    "2f420e52795042bdfa7773d8a9051447b3291ce1411c680465552aa6c405b7764d5e87bea85ad00f8449ed8f72",
    "d0d662ab052691ca66424bc86d2df80ea41f43abf937d3259dc4b2d0dfb48a6c9139ddd7f76966e928e635553b",
    "a76c5c879d7b35d49eb2e62b0871cdac638939e25e8a1e0ef9d5280fa8ca328b351c3c765989cbcf3daa8b6ccc",
    "3aaf9f3979c92b3720fc88dc95ed84a1be059c6499b9fda236e7e818b04b0bc39c1e876b193bfe5569753f8812",
    "8cc08aaa9b63d1a16f80ef2554d7189c411f5869ca52c5b83fa36ff216b9c1d30062bebcfd2dc5bce0911934fd",
    "a79a86f6e698ced759c3ff9b6477338f3da4f9cd8514ea9982ccafb341b2384dd902f3d1ab7ac61dd29c6f21ba",
    "5b862f3730e37cfdc4fd806c22f221",
);
const A3_2_TAG: &str = "36e5f6b5c5e06070f0efca96227a863e";
const A3_3_TAG: &str = "f3477e7cd95417af89a6b8794c310cf0";
const A5_KEY: &str = "1c9240a5eb55d38af333888604f6b5f0473917c1402b80099dca5cbc207075c0";
const A5_NONCE: &str = "000000000102030405060708";
const A5_AAD: &str = "f33388860000000000004e91";
const A5_CIPHERTEXT: &str = concat!(
    "64a0861575861af460f062c79be643bd5e805cfd345cf389f108670ac76c8cb24c6cfc18755d43eea09ee94e38",
    "2d26b0bdb7b73c321b0100d4f03b7f355894cf332f830e710b97ce98c8a84abd0b948114ad176e008d33bd60f9",
    "82b1ff37c8559797a06ef4f0ef61c186324e2b3506383606907b6a7c02b0f9f6157b53c867e4b9166c767b804d",
    "46a59b5216cde7a4e99040c5a40433225ee282a1b0a06c523eaf4534d7f83fa1155b0047718cbc546a0d072b04",
    "b3564eea1b422273f548271a0bb2316053fa76991955ebd63159434ecebb4e466dae5a1073a6727627097a1049",
    "e617d91d361094fa68f0ff77987130305beaba2eda04df997b714d6c6f2c29a6ad5cb4022b02709b",
);
const A5_TAG: &str = "eead9d67890cbb22392336fea1851f38";
const A5_PLAINTEXT: &str = concat!(
    "496e7465726e65742d4472616674732061726520647261667420646f63756d656e74732076616c696420666f72",
    "2061206d6178696d756d206f6620736978206d6f6e74687320616e64206d617920626520757064617465642c20",
    "7265706c616365642c206f72206f62736f6c65746564206279206f7468657220646f63756d656e747320617420",
    "616e792074696d652e20497420697320696e617070726f70726961746520746f2075736520496e7465726e6574",
    "2d447261667473206173207265666572656e6365206d6174657269616c206f7220746f2063697465207468656d",
    "206f74686572207468616e206173202fe2809c776f726b20696e2070726f67726573732e2fe2809d",
);

/// Bytes spelled as hex digit pairs, as `orangec` prints a `Word[8]` array.
fn bytes(hex: &str) -> String {
    assert_eq!(hex.len() % 2, 0, "odd hex length");
    let pairs = (0..hex.len())
        .step_by(2)
        .map(|at| format!("0x{}", &hex[at..at + 2]))
        .collect::<Vec<_>>();
    format!("[{}]", pairs.join(", "))
}

fn hex_of(text: &[u8]) -> String {
    text.iter().fold(String::new(), |mut hex, byte| {
        write!(hex, "{byte:02x}").unwrap();
        hex
    })
}

/// The line `eval` prints for a function of `Word[8]^n` given in hex.
fn byte_line(name: &str, hex: &str) -> String {
    format!(
        "rfc8439::{name}: Word[8]^{} = {}\n",
        hex.len() / 2,
        bytes(hex)
    )
}

fn rfc8439_stdout() -> String {
    assert_eq!(IETF_TEXT.len(), 375);
    assert_eq!(A5_CIPHERTEXT.len(), 2 * 265);
    assert_eq!(A5_PLAINTEXT.len(), 2 * 265);
    // Section 2.8: the additional data and the ciphertext, each padded with
    // zeros to a multiple of 16 bytes, and their lengths as little-endian
    // 64-bit words.
    let mac_data = format!(
        "{A5_AAD}{}{A5_CIPHERTEXT}{}{}{}",
        "00".repeat(4),
        "00".repeat(7),
        hex_of(&12_u64.to_le_bytes()),
        hex_of(&265_u64.to_le_bytes()),
    );
    [
        byte_line("ietf", &hex_of(IETF_TEXT.as_bytes())),
        byte_line("a2_2", A2_2_CIPHERTEXT),
        byte_line("a3_2", A3_2_TAG),
        byte_line("a3_3", A3_3_TAG),
        byte_line("a5_key", A5_KEY),
        byte_line("a5_nonce", A5_NONCE),
        byte_line("a5_aad", A5_AAD),
        byte_line("a5_ciphertext", A5_CIPHERTEXT),
        byte_line("a5_tag", A5_TAG),
        byte_line("a5_mac_data", &mac_data),
        String::from("rfc8439::a5_authentic: Bool = true\n"),
        byte_line("a5_plaintext", A5_PLAINTEXT),
        String::from("rfc8439::a5_opens_to_text: Bool = true\n"),
    ]
    .concat()
}

fn lengths_stdout() -> String {
    String::from(concat!(
        "lengths::pepin: (Mod[65537], Mod[65537], Bool) = (65536, 21846, true)\n",
        "lengths::halves: Word[8]^2 = [0xff, 0xa5]\n",
        "lengths::words: (Word[8]^8, Word[64]) = ([0x08, 0x07, 0x06, 0x05, 0x04, 0x03, 0x02, ",
        "0x01], 0x0807060504030201)\n",
        "lengths::widest: Int = 0\n",
    ))
}

const CASES: [Case; 3] = [
    Case {
        fixture: "valid-rfc8439.or",
        eval_options: &[],
        expectation: Expectation::Success {
            stdout: rfc8439_stdout,
            stderr: "",
        },
        rules: &[
            "S3P-LENGTH-01",
            "S3P-LITERAL-01",
            "S3P-INDEX-01",
            "S3P-ORDER-01",
            "S3P-COST-01",
            "S3P-COMPAT-01",
            "S3P-DETERMINISM-01",
        ],
    },
    Case {
        fixture: "valid-lengths.or",
        eval_options: &["--steps", "2097152", "--stats"],
        expectation: Expectation::Success {
            stdout: lengths_stdout,
            stderr: concat!(
                "lengths::pepin: 1452583 steps\n",
                "lengths::halves: 4629 steps\n",
                "lengths::words: 16523 steps\n",
                "lengths::widest: 804 steps\n",
                "total: 1474539 of 2097152 steps\n",
            ),
        },
        rules: &[
            "S3P-LENGTH-01",
            "S3P-LITERAL-01",
            "S3P-INDEX-01",
            "S3P-ORDER-01",
            "S3P-COST-01",
            "S3P-STEPS-01",
            "S3P-STATS-01",
            "S3P-DETERMINISM-01",
        ],
    },
    Case {
        fixture: "invalid-lengths.or",
        eval_options: &[],
        expectation: Expectation::Failure {
            codes: &[
                "ORC0221", "ORC0221", "ORC0222", "ORC0223", "ORC0223", "ORC0223", "ORC0221",
            ],
            locations: &[
                "invalid-lengths.or:8:24",
                "invalid-lengths.or:9:38",
                "invalid-lengths.or:10:45",
                "invalid-lengths.or:11:53",
                "invalid-lengths.or:12:60",
                "invalid-lengths.or:13:59",
                "invalid-lengths.or:15:20",
            ],
            messages: &[
                "an array length must be a decimal integer from 1 through 65536",
                "`++` joins 65536 and 1 elements, 65537 in all, but `Word[8]^65536` has 65536",
                "this slice reaches elements 1 through 65536, out of range for `Word[8]^65536`",
                "this index runs from 0 through 65535, out of range for `Word[8]^65535`",
                "this index runs from 0 through 4294967295, out of range for `Word[8]^65536`",
            ],
        },
        rules: &[
            "S3P-LENGTH-01",
            "S3P-INDEX-01",
            "S3P-ORDER-01",
            "S3P-COMPAT-01",
            "S3P-DETERMINISM-01",
        ],
    },
];

/// Generated command-line cases in this file, each run twice.
const GENERATED_EVIDENCE: &[(&str, &[&str])] = &[
    (
        "s3p_literals_and_byte_strings_reach_65536_elements_exactly",
        &[
            "S3P-LENGTH-01",
            "S3P-LITERAL-01",
            "S3P-INDEX-01",
            "S3P-DETERMINISM-01",
        ],
    ),
    (
        "s3p_steps_set_the_budget_of_the_whole_evaluation",
        &["S3P-STEPS-01", "S3P-STATS-01", "S3P-DETERMINISM-01"],
    ),
    (
        "s3p_spec_evaluates_only_the_named_functions",
        &["S3P-SPEC-01", "S3P-STATS-01", "S3P-DETERMINISM-01"],
    ),
    (
        "s3p_stats_follow_the_values_and_apply_only_to_eval",
        &[
            "S3P-STEPS-01",
            "S3P-SPEC-01",
            "S3P-STATS-01",
            "S3P-DETERMINISM-01",
        ],
    ),
];

const UNIT_EVIDENCE: &[TestEvidence] = &[
    TestEvidence {
        source_path: "src/parser.rs",
        test: "bounds_elements_per_array_literal",
        rules: &["S3P-LITERAL-01", "S3P-RES-01"],
    },
    TestEvidence {
        source_path: "src/semantics/tests.rs",
        test: "array_lengths_resolve_only_as_exact_decimals_from_1_through_65536",
        rules: &["S3P-LENGTH-01", "S3P-COMPAT-01"],
    },
    TestEvidence {
        source_path: "src/semantics/tests.rs",
        test: "arrays_of_65536_elements_are_indexed_joined_and_sliced_at_the_limit",
        rules: &["S3P-LENGTH-01", "S3P-INDEX-01", "S3P-DETERMINISM-01"],
    },
    TestEvidence {
        source_path: "src/semantics/tests.rs",
        test: "byte_strings_hold_one_through_65536_printable_bytes",
        rules: &["S3P-LITERAL-01", "S3P-RES-01"],
    },
    TestEvidence {
        source_path: "src/core.rs",
        test: "array_types_hold_one_to_65536_scalars_and_display_as_powers",
        rules: &["S3P-LENGTH-01"],
    },
    TestEvidence {
        source_path: "src/eval.rs",
        test: "words_convert_to_numbers_only_within_the_exact_integer_limit",
        rules: &["S3P-ORDER-01"],
    },
    TestEvidence {
        source_path: "src/eval.rs",
        test: "long_arrays_cost_one_step_for_each_64_elements_they_make",
        rules: &["S3P-COST-01", "S3P-RES-01"],
    },
    TestEvidence {
        source_path: "src/eval.rs",
        test: "selected_functions_are_evaluated_in_source_order_with_their_steps",
        rules: &[
            "S3P-STEPS-01",
            "S3P-SPEC-01",
            "S3P-STATS-01",
            "S3P-COMPAT-01",
            "S3P-DETERMINISM-01",
        ],
    },
    TestEvidence {
        source_path: "src/eval.rs",
        test: "late_allocation_failures_discard_completed_values",
        rules: &["S3P-RES-01"],
    },
    TestEvidence {
        source_path: "orangec/src/main.rs",
        test: "evaluation_options_parse_with_exact_bounds_and_messages",
        rules: &["S3P-STEPS-01", "S3P-SPEC-01", "S3P-STATS-01", "S3P-RES-01"],
    },
    TestEvidence {
        source_path: "orangec/src/main.rs",
        test: "step_limit_diagnostics_name_the_option_only_below_the_most_admitted",
        rules: &["S3P-STEPS-01"],
    },
    TestEvidence {
        source_path: "orangec/src/main.rs",
        test: "step_report_follows_committed_values_and_is_not_written_without_them",
        rules: &["S3P-STATS-01", "S3P-RES-01"],
    },
    TestEvidence {
        source_path: "orangec/src/main.rs",
        test: "cli_diagnostic_code_inventory_is_exact_ordered_and_unique",
        rules: &["S3P-SPEC-01"],
    },
    TestEvidence {
        source_path: "orangec/src/crypt.rs",
        test: "headers_round_trip_and_parse_strictly",
        rules: &["S3P-COMPAT-01"],
    },
];

fn orangec() -> Command {
    Command::new(env!("CARGO_BIN_EXE_orangec"))
}

fn fixture_directory() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/s3p")
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

fn assert_success(output: &Output, stdout: &str, stderr: &str, context: &str) {
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
        .join(format!("orangec-s3p-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&directory);
    fs::create_dir(&directory).unwrap();
    directory
}

/// The rule index rows of the specification: rule ID and evidence label.
fn documented_rules() -> Vec<(&'static str, &'static str)> {
    LENGTHS_SPECIFICATION
        .lines()
        .filter_map(|line| {
            let row = line.strip_prefix("| `")?;
            let (rule, _) = row.split_once("` |")?;
            if !rule.starts_with("S3P-") {
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
        _ => panic!("unknown S3p evidence layer {label:?}"),
    }
}

fn unit_source(source_path: &str) -> &'static str {
    match source_path {
        "src/parser.rs" => PARSER_SOURCE,
        "src/semantics.rs" => SEMANTICS_SOURCE,
        "src/semantics/tests.rs" => SEMANTICS_TESTS_SOURCE,
        "src/core.rs" => CORE_SOURCE,
        "src/eval.rs" => EVAL_SOURCE,
        "orangec/src/main.rs" => ORANGEC_SOURCE,
        "orangec/src/crypt.rs" => CRYPT_SOURCE,
        _ => panic!("unmapped S3p evidence source {source_path}"),
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
        S3P_CONFORMANCE_SOURCE.matches(&declaration).count(),
        1,
        "generated evidence {test} must be declared exactly once at file root"
    );
}

#[test]
fn s3p_rule_index_is_exact_and_covered() {
    let documented = documented_rules();
    let documented_ids: Vec<_> = documented.iter().map(|(rule, _)| *rule).collect();
    assert_eq!(
        documented_ids, RULES,
        "docs/LENGTHS_2026.md rule index drifted from the S3p runner"
    );
    let known: BTreeSet<_> = RULES.iter().copied().collect();
    assert_eq!(known.len(), RULES.len(), "duplicate S3p rule ID");

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
fn s3p_cli_conformance_corpus_is_exact_and_repeatable() {
    let directory = fixture_directory();
    let mut observed: Vec<_> = fs::read_dir(&directory)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().into_string().unwrap())
        .collect();
    observed.sort();
    let mut expected: Vec<_> = CASES.iter().map(|case| case.fixture).collect();
    expected.sort_unstable();
    assert_eq!(observed, expected, "unexpected S3p fixture inventory");

    for case in CASES {
        let path = directory.join(case.fixture);
        let check = format!("{} check", case.fixture);
        let eval = format!("{} eval", case.fixture);
        let eval_arguments = [&["eval"][..], case.eval_options].concat();
        let first_check = run_twice(&["check"], &path, &check);
        let first_eval = run_twice(&eval_arguments, &path, &eval);

        match case.expectation {
            Expectation::Success { stdout, stderr } => {
                assert_success(&first_check, "", "", &check);
                assert_success(&first_eval, &stdout(), stderr, &eval);
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
fn s3p_literals_and_byte_strings_reach_65536_elements_exactly() {
    let directory = scratch_directory("literals");
    // A literal of every 16-bit word in order, and a byte string of 65,536
    // characters. Only the functions that read them are evaluated, so the
    // table itself is not printed.
    let listed = |count: u32| {
        (0..count)
            .map(|value| value.to_string())
            .collect::<Vec<_>>()
            .join(", ")
    };
    let text = "ab".repeat(32_768);
    let program = format!(
        "edition 2026;\nmodule literal {{\n  spec table() -> Word[16]^65536 {{\n    [{}]\n  }}\n  \
         spec read() -> (Word[16], Word[16]) {{\n    let t: Word[16]^65536 = table();\n    \
         (t[0x1234], t[t[65535]])\n  }}\n  \
         spec text() -> Word[8]^2 {{\n    let s: Word[8]^65536 = \"{text}\";\n    \
         [s[0], s[65535]]\n  }}\n}}\n",
        listed(65_536)
    );
    let path = directory.join("literal.or");
    fs::write(&path, &program).unwrap();
    let read = run_twice(
        &["eval", "--spec", "read", "--spec", "text"],
        &path,
        "65,536 elements",
    );
    assert_success(
        &read,
        concat!(
            "literal::read: (Word[16], Word[16]) = (0x1234, 0xffff)\n",
            "literal::text: Word[8]^2 = [0x61, 0x62]\n",
        ),
        "",
        "65,536 elements",
    );

    // One element more is the parser's error at that element.
    let prefix = "    [";
    let over = format!(
        "edition 2026;\nmodule literal {{\n  spec table() -> Word[32]^65536 {{\n{prefix}{}]\n  }}\n}}\n",
        listed(65_537)
    );
    let column = prefix.len() + listed(65_536).len() + ", ".len() + 1;
    let path = directory.join("over.or");
    fs::write(&path, &over).unwrap();
    let rejected = run_twice(&["check"], &path, "65,537 elements");
    assert_failure(
        &rejected,
        &["ORC0106"],
        &[format!("over.or:4:{column}").as_str()],
        &["array literal has more than 65536 elements"],
        "65,537 elements",
    );

    // A byte string of 65,537 bytes is reported at the string.
    let long = format!(
        "edition 2026;\nmodule literal {{\n  spec text() -> Word[8]^65536 {{ \"{}\" }}\n}}\n",
        "a".repeat(65_537)
    );
    let path = directory.join("long.or");
    fs::write(&path, &long).unwrap();
    let rejected = run_twice(&["check"], &path, "65,537 bytes");
    assert_failure(
        &rejected,
        &["ORC0221"],
        &["long.or:3:34"],
        &[
            "a byte string holds at most 65536 bytes",
            "a byte string is an array `Word[8]^n` of 1 through 65536 bytes; join longer runs \
             with `++`",
        ],
        "65,537 bytes",
    );
    fs::remove_dir_all(&directory).unwrap();
}

const BUDGET_PROGRAM: &str = concat!(
    "edition 2026;\n",
    "module budget {\n",
    "  spec first() -> Int { for i in 0..100 with s: Int = 0 { s + i } }\n",
    "  spec second() -> Int { for i in 0..50 with s: Int = 0 { s + 1 } }\n",
    "}\n",
);

#[test]
fn s3p_steps_set_the_budget_of_the_whole_evaluation() {
    let directory = scratch_directory("steps");
    let path = directory.join("budget.or");
    fs::write(&path, BUDGET_PROGRAM).unwrap();
    let values = "budget::first: Int = 4950\nbudget::second: Int = 50\n";
    let report = |budget: u32| {
        format!(
            "budget::first: 501 steps\nbudget::second: 252 steps\ntotal: 753 of {budget} steps\n"
        )
    };

    // The default budget, exactly the steps needed, and the most admitted.
    for (options, budget) in [
        (&["--stats"][..], 1_048_576),
        (&["--steps", "753", "--stats"][..], 753),
        (&["--stats", "--steps", "1073741824"][..], 1_073_741_824),
    ] {
        let arguments = [&["eval"][..], options].concat();
        let output = run_twice(&arguments, &path, &format!("{options:?}"));
        assert_success(&output, values, &report(budget), &format!("{options:?}"));
    }

    // The budget is shared, so one step fewer stops the second function,
    // and fewer than the first needs stops the first.
    for (budget, line) in [("752", 4), ("500", 3)] {
        let output = run_twice(
            &["eval", "--steps", budget, "--stats"],
            &path,
            &format!("--steps {budget}"),
        );
        assert_failure(
            &output,
            &["ORC0301"],
            &[format!("budget.or:{line}:8").as_str()],
            &[
                "reference evaluation step limit exceeded",
                &format!("= note: at most {budget} evaluation steps are permitted\n"),
                "= note: no partial value set is returned\n",
                "= note: `orangec eval --steps N` sets the budget, up to 1073741824 steps\n",
            ],
            &format!("--steps {budget}"),
        );
        assert!(
            !String::from_utf8_lossy(&output.stderr).contains("total:"),
            "a failed evaluation reports no steps"
        );
    }

    let invalid = "option `--steps` takes a number of steps from 1 through 1073741824";
    for value in [
        "0",
        "01",
        "+5",
        "-5",
        "5x",
        "1_000",
        "1073741825",
        "18446744073709551617",
        "",
    ] {
        let output = run(&["eval", "--steps", value], &path);
        assert_usage_error(&output, invalid, &format!("--steps {value:?}"));
    }
    let output = run(&["eval", "--steps", "5", "--steps", "6"], &path);
    assert_usage_error(
        &output,
        "option `--steps` may be specified at most once",
        "--steps twice",
    );
    fs::remove_dir_all(&directory).unwrap();
}

const CHOSEN_PROGRAM: &str = concat!(
    "edition 2026;\n",
    "module chosen {\n",
    "  spec a() -> Int { 1 }\n",
    "  spec b() -> Int { 2 }\n",
    "  spec c[n in 1..3]() -> Int { n }\n",
    "  spec d(x: Int) -> Int { x }\n",
    "}\n",
);

#[test]
fn s3p_spec_evaluates_only_the_named_functions() {
    let directory = scratch_directory("spec");
    let path = directory.join("chosen.or");
    fs::write(&path, CHOSEN_PROGRAM).unwrap();

    // Functions are evaluated in source order, every instance of a sized
    // one, and a repeated name counts once.
    let output = run_twice(
        &[
            "eval", "--spec", "c", "--spec", "a", "--spec", "c", "--stats",
        ],
        &path,
        "--spec c a c",
    );
    assert_success(
        &output,
        "chosen::a: Int = 1\nchosen::c[1]: Int = 1\nchosen::c[2]: Int = 2\n",
        concat!(
            "chosen::a: 1 step\n",
            "chosen::c[1]: 1 step\n",
            "chosen::c[2]: 1 step\n",
            "total: 3 of 1048576 steps\n",
        ),
        "--spec c a c",
    );
    let output = run_twice(&["eval", "--spec=b"], &path, "--spec=b");
    assert_success(&output, "chosen::b: Int = 2\n", "", "--spec=b");

    // A name that matches no function without parameters evaluates nothing.
    for name in ["d", "nope", "chosen"] {
        let output = run_twice(
            &["eval", "--spec", "a", "--spec", name],
            &path,
            &format!("--spec {name}"),
        );
        assert_eq!(output.status.code(), Some(1), "--spec {name}");
        assert_eq!(output.stdout, b"", "--spec {name}");
        assert_eq!(
            String::from_utf8_lossy(&output.stderr),
            format!(
                "error[ORC1016]: module `chosen` has no function `{name}` without parameters\n  \
                 = note: `--spec` names a function of the evaluated module that takes no \
                 parameters; a function with sizes is evaluated in every instance\n"
            ),
            "--spec {name}"
        );
    }

    for name in ["", "9a", "a-b", "chosen::a", "c[1]", "a b"] {
        let output = run(&["eval", "--spec", name], &path);
        assert_usage_error(
            &output,
            "option `--spec` takes the name of a function",
            &format!("--spec {name:?}"),
        );
    }
    let names = (0..65).map(|index| format!("f{index}")).collect::<Vec<_>>();
    let mut arguments = vec!["eval"];
    for name in &names {
        arguments.extend(["--spec", name.as_str()]);
    }
    let output = run(&arguments, &path);
    assert_usage_error(
        &output,
        "option `--spec` names at most 64 functions",
        "65 names",
    );
    fs::remove_dir_all(&directory).unwrap();
}

#[test]
fn s3p_stats_follow_the_values_and_apply_only_to_eval() {
    let directory = scratch_directory("stats");
    let path = directory.join("chosen.or");
    fs::write(&path, CHOSEN_PROGRAM).unwrap();

    // With both streams in one file, the report follows the values.
    let transcript = |name: &str| {
        let merged = directory.join(name);
        let file = File::create(&merged).unwrap();
        let status = orangec()
            .args(["eval", "--stats", "--steps", "3", "--spec", "c"])
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
            "chosen::c[1]: Int = 1\n",
            "chosen::c[2]: Int = 2\n",
            "chosen::c[1]: 1 step\n",
            "chosen::c[2]: 1 step\n",
            "total: 2 of 3 steps\n",
        )
    );

    // Each option with a command that does not evaluate is a usage error,
    // before or after the command, and even at the default budget. Since
    // S3q, `test` takes `--steps` and `--stats` too, but not `--spec`.
    for (arguments, message) in [
        (
            &["check", "--steps", "5"][..],
            "`--steps` applies only to eval, test, and replay",
        ),
        (
            &["check", "--steps", "1048576"][..],
            "`--steps` applies only to eval, test, and replay",
        ),
        (&["--spec", "a", "lex"][..], "`--spec` applies only to eval"),
        (
            &["check", "--stats"][..],
            "`--stats` applies only to eval, test, and replay",
        ),
        (
            &["lex", "--stats", "--spec", "a", "--steps", "9"][..],
            "`--steps` applies only to eval, test, and replay",
        ),
        (
            &["enc", "--stats"][..],
            "`--stats` applies only to eval, test, and replay",
        ),
        (
            &["test", "--spec", "a"][..],
            "`--spec` applies only to eval",
        ),
    ] {
        let output = run(arguments, &path);
        assert_usage_error(
            &output,
            &format!("option {message}"),
            &format!("{arguments:?}"),
        );
    }
    fs::remove_dir_all(&directory).unwrap();
}
