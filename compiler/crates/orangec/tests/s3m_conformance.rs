//! External conformance evidence for the proposed Orange 2026 S3m slice.
//!
//! The rule index lives in `docs/SIZES_2026.md`. This runner requires that
//! index to agree exactly with the evidence map below, runs every fixture and
//! every generated program through the real `orangec` binary twice, and
//! checks that every named unit test is declared once in its source's test
//! module. S3l compatibility is also observed by the S2 through S3l runners,
//! which run their fixtures through the same binary.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const SIZES_SPECIFICATION: &str = include_str!("../../../../docs/SIZES_2026.md");
const S3M_CONFORMANCE_SOURCE: &str = include_str!("s3m_conformance.rs");
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

/// Every S3m rule, in the order of the specification's index.
const RULES: [&str; 10] = [
    "S3M-SYNTAX-01",
    "S3M-SIZE-01",
    "S3M-VALUE-01",
    "S3M-INSTANCE-01",
    "S3M-CALL-01",
    "S3M-CORE-01",
    "S3M-EVAL-01",
    "S3M-RES-01",
    "S3M-COMPAT-01",
    "S3M-DETERMINISM-01",
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
    "S3M-SYNTAX-01",
    "S3M-SIZE-01",
    "S3M-VALUE-01",
    "S3M-INSTANCE-01",
    "S3M-CALL-01",
    "S3M-CORE-01",
    "S3M-EVAL-01",
    "S3M-DETERMINISM-01",
];

const CASES: [Case; 6] = [
    Case {
        fixture: "sha256.or",
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
            "sha256::abc: Word[8]^32 = [0xba, 0x78, 0x16, 0xbf, 0x8f, 0x01, 0xcf, 0xea, 0x41, ",
            "0x41, 0x40, 0xde, 0x5d, 0xae, 0x22, 0x23, 0xb0, 0x03, 0x61, 0xa3, 0x96, 0x17, ",
            "0x7a, 0x9c, 0xb4, 0x10, 0xff, 0x61, 0xf2, 0x00, 0x15, 0xad]\n",
            "sha256::two_blocks: Word[8]^32 = [0x24, 0x8d, 0x6a, 0x61, 0xd2, 0x06, 0x38, 0xb8, ",
            "0xe5, 0xc0, 0x26, 0x93, 0x0c, 0x3e, 0x60, 0x39, 0xa3, 0x3c, 0xe4, 0x59, 0x64, ",
            "0xff, 0x21, 0x67, 0xf6, 0xec, 0xed, 0xd4, 0x19, 0xdb, 0x06, 0xc1]\n",
        )),
        rules: VALID_RULES,
    },
    Case {
        fixture: "valid-hmac.or",
        expectation: Expectation::Success(concat!(
            "hmac::case1: Word[8]^32 = [0xb0, 0x34, 0x4c, 0x61, 0xd8, 0xdb, 0x38, 0x53, 0x5c, ",
            "0xa8, 0xaf, 0xce, 0xaf, 0x0b, 0xf1, 0x2b, 0x88, 0x1d, 0xc2, 0x00, 0xc9, 0x83, ",
            "0x3d, 0xa7, 0x26, 0xe9, 0x37, 0x6c, 0x2e, 0x32, 0xcf, 0xf7]\n",
            "hmac::case2: Word[8]^32 = [0x5b, 0xdc, 0xc1, 0x46, 0xbf, 0x60, 0x75, 0x4e, 0x6a, ",
            "0x04, 0x24, 0x26, 0x08, 0x95, 0x75, 0xc7, 0x5a, 0x00, 0x3f, 0x08, 0x9d, 0x27, ",
            "0x39, 0x83, 0x9d, 0xec, 0x58, 0xb9, 0x64, 0xec, 0x38, 0x43]\n",
        )),
        rules: VALID_RULES,
    },
    Case {
        fixture: "valid-poly1305.or",
        expectation: Expectation::Success(concat!(
            "poly1305::example: Word[8]^16 = [0xa8, 0x06, 0x1d, 0xc1, 0x30, 0x51, 0x36, 0xc6, ",
            "0xc2, 0x2b, 0x8b, 0xaf, 0x0c, 0x01, 0x27, 0xa9]\n",
        )),
        rules: VALID_RULES,
    },
    Case {
        fixture: "valid-sizes.or",
        expectation: Expectation::Success(concat!(
            "sizes::zeros[1]: Word[8]^1 = [0x00]\n",
            "sizes::zeros[2]: Word[8]^2 = [0x00, 0x00]\n",
            "sizes::zeros[3]: Word[8]^3 = [0x00, 0x00, 0x00]\n",
            "sizes::code[1, 7]: Int = 17\n",
            "sizes::code[1, 8]: Int = 18\n",
            "sizes::code[2, 7]: Int = 27\n",
            "sizes::code[2, 8]: Int = 28\n",
            "sizes::blocks[1]: Int = 3\n",
            "sizes::blocks[2]: Int = 1\n",
            "sizes::blocks[3]: Int = 2\n",
            "sizes::total: Int = 16\n",
            "sizes::pair: Word[8]^4 = [0xab, 0xcd, 0xab, 0xcd]\n",
            "sizes::second: Word[8]^3 = [0x64, 0x65, 0x66]\n",
            "sizes::last: Word[8]^2 = [0x6c, 0x6f]\n",
        )),
        rules: VALID_RULES,
    },
    Case {
        fixture: "invalid-sizes-syntax.or",
        expectation: Expectation::Failure {
            codes: &[
                "ORC0101", "ORC0101", "ORC0101", "ORC0101", "ORC0101", "ORC0101", "ORC0101",
            ],
            locations: &[
                "invalid-sizes-syntax.or:6:16",
                "invalid-sizes-syntax.or:7:28",
                "invalid-sizes-syntax.or:8:57",
                "invalid-sizes-syntax.or:9:42",
                "invalid-sizes-syntax.or:10:53",
                "invalid-sizes-syntax.or:11:50",
                "invalid-sizes-syntax.or:12:47",
            ],
            messages: &[
                "expected `in` after the size's name",
                "expected the size's second bound",
                "a function has at most 4 size parameters",
                "one size parameter too many",
                "a sized function is written `spec f[n in 1..5](x: Word[8]^n) -> Type { ... }` \
                 and checked once for each n from 1 up to, but not including, 5",
                "expected the end of the type after its array length",
                "an array length computed from sizes is written in parentheses, as in \
                 `Word[8]^(2 * n)`",
                "expected `]` after the array length",
                "a length computed from sizes is written in parentheses, as in `[0; (2 * n)]`",
                "expected `with` and the loop's accumulator",
                "a bound computed from sizes is written in parentheses, as in \
                 `for i in 0..(n - 1) with s: Type = start { step }`",
                "expected `(` after the sizes",
                "a sized function is called with its sizes in brackets before its arguments, \
                 as in `sha256[2](m)`",
            ],
        },
        rules: &["S3M-SYNTAX-01", "S3M-COMPAT-01", "S3M-DETERMINISM-01"],
    },
    Case {
        fixture: "invalid-sizes.or",
        expectation: Expectation::Failure {
            codes: &[
                "ORC0238", "ORC0238", "ORC0238", "ORC0218", "ORC0218", "ORC0237", "ORC0223",
                "ORC0221", "ORC0238", "ORC0239", "ORC0239", "ORC0238", "ORC0239", "ORC0217",
            ],
            locations: &[
                "invalid-sizes.or:9:22",
                "invalid-sizes.or:10:20",
                "invalid-sizes.or:11:13",
                "invalid-sizes.or:12:25",
                "invalid-sizes.or:13:26",
                "invalid-sizes.or:14:33",
                "invalid-sizes.or:15:53",
                "invalid-sizes.or:16:37",
                "invalid-sizes.or:19:37",
                "invalid-sizes.or:20:29",
                "invalid-sizes.or:21:27",
                "invalid-sizes.or:22:29",
                "invalid-sizes.or:23:27",
                "invalid-sizes.or:24:35",
            ],
            messages: &[
                "the size range 3..3 is empty",
                "a size's bound must be at most 65536",
                "`many` has 361 instances, but a function has at most 256",
                "a size parameter `n in a..b` takes each value from a up to, but not including, \
                 b, with a < b <= 65536, and a function has at most 256 instances",
                "this name is already a size parameter",
                "size parameters and parameters share one namespace, and each name is unique",
                "a size may use only integer literals and size parameters",
                "index `3` is out of range for `Word[8]^1`",
                "in the instance `last[1]`, the first of `last` in error: a sized function is \
                 checked once for each value of its sizes",
                "this array length is 0, but an array has 1 through 256 elements",
                "in the instance `none[0]`, the first of `none` in error",
                "`first` is defined for `n` in 1..4",
                "this size is 4",
                "`first` takes 1 size, but this call gives 2",
                "`outside` has no size parameters, but this call gives 1 size",
                "no instance of `first` takes arguments of these lengths",
                "an array of length 9 is given",
                "this call fits more than one instance of `count`, among them `count[1]` and \
                 `count[2]`",
                "call cycle `swap[1]` -> `swap[2]` -> `swap[1]`",
            ],
        },
        rules: &[
            "S3M-SIZE-01",
            "S3M-VALUE-01",
            "S3M-INSTANCE-01",
            "S3M-CALL-01",
            "S3M-DETERMINISM-01",
        ],
    },
];

/// Generated command-line cases in this file, each run twice.
const GENERATED_EVIDENCE: &[(&str, &[&str])] = &[(
    "s3m_instances_are_exact_at_the_limit",
    &[
        "S3M-SIZE-01",
        "S3M-INSTANCE-01",
        "S3M-EVAL-01",
        "S3M-RES-01",
        "S3M-DETERMINISM-01",
    ],
)];

const UNIT_EVIDENCE: &[TestEvidence] = &[
    TestEvidence {
        source_path: "src/parser.rs",
        test: "parses_size_parameters_sized_types_and_sized_calls_with_exact_spans",
        rules: &["S3M-SYNTAX-01", "S3M-COMPAT-01"],
    },
    TestEvidence {
        source_path: "src/parser.rs",
        test: "a_name_before_brackets_is_indexed_unless_a_call_follows",
        rules: &["S3M-SYNTAX-01", "S3M-COMPAT-01"],
    },
    TestEvidence {
        source_path: "src/parser.rs",
        test: "rejects_malformed_sizes_with_exact_messages",
        rules: &["S3M-SYNTAX-01", "S3M-RES-01"],
    },
    TestEvidence {
        source_path: "src/parser.rs",
        test: "sizes_count_toward_the_height_of_their_expression",
        rules: &["S3M-SYNTAX-01", "S3M-RES-01"],
    },
    TestEvidence {
        source_path: "src/semantics/tests.rs",
        test: "sized_functions_take_one_core_function_per_instance_in_ascending_order",
        rules: &["S3M-INSTANCE-01", "S3M-CORE-01", "S3M-DETERMINISM-01"],
    },
    TestEvidence {
        source_path: "src/semantics/tests.rs",
        test: "only_the_first_failing_instance_of_a_function_is_reported_and_named",
        rules: &["S3M-INSTANCE-01", "S3M-DETERMINISM-01"],
    },
    TestEvidence {
        source_path: "src/semantics/tests.rs",
        test: "size_parameters_have_bounded_ranges_and_unique_names",
        rules: &["S3M-SIZE-01", "S3M-RES-01"],
    },
    TestEvidence {
        source_path: "src/semantics/tests.rs",
        test: "sizes_are_built_from_literals_and_size_parameters_only",
        rules: &["S3M-VALUE-01"],
    },
    TestEvidence {
        source_path: "src/semantics/tests.rs",
        test: "sizes_are_exact_and_held_to_the_integer_limit",
        rules: &["S3M-VALUE-01", "S3M-RES-01"],
    },
    TestEvidence {
        source_path: "src/semantics/tests.rs",
        test: "calls_name_one_instance_by_their_sizes_or_by_their_arguments",
        rules: &["S3M-CALL-01"],
    },
    TestEvidence {
        source_path: "src/semantics/tests.rs",
        test: "calls_that_name_no_instance_are_reported_at_the_call",
        rules: &["S3M-CALL-01"],
    },
    TestEvidence {
        source_path: "src/semantics/tests.rs",
        test: "sizes_are_constants_in_indices_slices_and_values",
        rules: &["S3M-VALUE-01", "S3M-CORE-01"],
    },
    TestEvidence {
        source_path: "src/semantics/tests.rs",
        test: "sized_functions_of_used_modules_are_called_by_their_instances",
        rules: &["S3M-CALL-01", "S3M-CORE-01"],
    },
    TestEvidence {
        source_path: "src/semantics/tests.rs",
        test: "call_cycles_are_found_between_instances",
        rules: &["S3M-CALL-01", "S3M-DETERMINISM-01"],
    },
    TestEvidence {
        source_path: "src/semantics/tests.rs",
        test: "every_part_of_a_size_is_one_semantic_event",
        rules: &["S3M-RES-01"],
    },
    TestEvidence {
        source_path: "src/semantics/tests.rs",
        test: "rejects_foreign_size_spans",
        rules: &["S3M-RES-01"],
    },
    TestEvidence {
        source_path: "src/eval.rs",
        test: "every_instance_of_a_sized_root_is_evaluated_and_named_by_its_sizes",
        rules: &["S3M-CORE-01", "S3M-EVAL-01"],
    },
    TestEvidence {
        source_path: "src/eval.rs",
        test: "deepest_accepted_sources_fit_in_one_mebibyte_of_stack",
        rules: &["S3M-RES-01"],
    },
    TestEvidence {
        source_path: "src/diagnostic.rs",
        test: "diagnostic_code_inventory_is_exact_ordered_and_unique",
        rules: &["S3M-SIZE-01", "S3M-VALUE-01", "S3M-CALL-01"],
    },
];

fn orangec() -> Command {
    Command::new(env!("CARGO_BIN_EXE_orangec"))
}

fn fixture_directory() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/s3m")
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
        .join(format!("orangec-s3m-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&directory);
    fs::create_dir(&directory).unwrap();
    directory
}

/// The rule index rows of the specification: rule ID and evidence label.
fn documented_rules() -> Vec<(&'static str, &'static str)> {
    SIZES_SPECIFICATION
        .lines()
        .filter_map(|line| {
            let row = line.strip_prefix("| `")?;
            let (rule, _) = row.split_once("` |")?;
            if !rule.starts_with("S3M-") {
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
        _ => panic!("unknown S3m evidence layer {label:?}"),
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
        _ => panic!("unmapped S3m evidence source {source_path}"),
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
        S3M_CONFORMANCE_SOURCE.matches(&declaration).count(),
        1,
        "generated evidence {test} must be declared exactly once at file root"
    );
}

#[test]
fn s3m_rule_index_is_exact_and_covered() {
    let documented = documented_rules();
    let documented_ids: Vec<_> = documented.iter().map(|(rule, _)| *rule).collect();
    assert_eq!(
        documented_ids, RULES,
        "docs/SIZES_2026.md rule index drifted from the S3m runner"
    );
    let known: BTreeSet<_> = RULES.iter().copied().collect();
    assert_eq!(known.len(), RULES.len(), "duplicate S3m rule ID");

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
fn s3m_cli_conformance_corpus_is_exact_and_repeatable() {
    let directory = fixture_directory();
    let mut observed: Vec<_> = fs::read_dir(&directory)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().into_string().unwrap())
        .collect();
    observed.sort();
    let mut expected: Vec<_> = CASES.iter().map(|case| case.fixture).collect();
    expected.sort_unstable();
    assert_eq!(observed, expected, "unexpected S3m fixture inventory");

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
fn s3m_instances_are_exact_at_the_limit() {
    let directory = scratch_directory("limits");

    // A function has at most 256 instances, however many sizes give them:
    // four sizes of four values each give 256, the first changing slowest,
    // and one size from 1 through 256 gives every admitted array length. A
    // size's bound is at most 65536. One instance more, or a bound one
    // larger, is reported at the declaration and nothing is evaluated.
    let program = |last: u32, longest: u32, bound: u32| {
        format!(
            "edition 2026;\nmodule limits {{\n  \
             spec code[a in 0..4, b in 0..4, c in 0..4, d in 0..{last}]() -> Int {{\n    \
             (((((a * 4) + b) * 4) + c) * 4) + d\n  }}\n  \
             spec zeros[n in 1..{longest}]() -> Word[8]^n {{ [0; n] }}\n  \
             spec edge[n in 65535..{bound}]() -> Int {{ n }}\n}}\n"
        )
    };
    let path = directory.join("limits.or");
    fs::write(&path, program(4, 257, 65536)).unwrap();
    let most = run_twice("eval", &path, "256 instances");
    let mut expected = String::new();
    for index in 0..256_u32 {
        let (a, b, c, d) = (index / 64, (index / 16) % 4, (index / 4) % 4, index % 4);
        writeln!(expected, "limits::code[{a}, {b}, {c}, {d}]: Int = {index}").unwrap();
    }
    for length in 1..=256 {
        writeln!(
            expected,
            "limits::zeros[{length}]: Word[8]^{length} = [{}]",
            vec!["0x00"; length].join(", ")
        )
        .unwrap();
    }
    expected.push_str("limits::edge[65535]: Int = 65535\n");
    assert_success(&most, &expected, "256 instances");

    fs::write(&path, program(5, 258, 65537)).unwrap();
    let over = run_twice("check", &path, "one instance more");
    assert_failure(
        &over,
        &["ORC0238", "ORC0238", "ORC0238"],
        &["limits.or:3:13", "limits.or:6:14", "limits.or:7:25"],
        &[
            "`code` has 320 instances, but a function has at most 256",
            "`zeros` has 257 instances, but a function has at most 256",
            "a size's bound must be at most 65536",
        ],
        "one instance more",
    );
    fs::remove_dir_all(&directory).unwrap();
}
