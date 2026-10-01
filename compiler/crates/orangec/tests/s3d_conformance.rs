//! External conformance evidence for the proposed Orange 2026 S3d slice.
//!
//! The rule index lives in `docs/ARRAYS_2026.md`. This runner requires that
//! index to agree exactly with the evidence map below, runs every fixture and
//! generated source through the real `orangec` binary twice, and checks that
//! every named unit test is declared once in its source's test module.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

const ARRAYS_SPECIFICATION: &str = include_str!("../../../../docs/ARRAYS_2026.md");
const S3D_CONFORMANCE_SOURCE: &str = include_str!("s3d_conformance.rs");
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

/// Every S3d rule, in the order of the specification's index.
const RULES: [&str; 17] = [
    "S3D-GRAMMAR-01",
    "S3D-TYPE-01",
    "S3D-LITERAL-01",
    "S3D-INDEX-01",
    "S3D-OPERATOR-01",
    "S3D-EVAL-01",
    "S3D-DISPLAY-01",
    "S3D-DIAG-01",
    "S3D-CORE-01",
    "S3D-RES-ELEM-01",
    "S3D-RES-NEST-01",
    "S3D-RES-EVENT-01",
    "S3D-RES-STEP-01",
    "S3D-RES-STACK-01",
    "S3D-RES-FAIL-01",
    "S3D-COMPAT-01",
    "S3D-DETERMINISM-01",
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

const CASES: [Case; 8] = [
    Case {
        fixture: "valid-arrays.or",
        expectation: Expectation::Success(concat!(
            "arrays::bytes: Word[8]^4 = [0x01, 0x20, 0xff, 0x07]\n",
            "arrays::last_byte: Word[8] = 0x07\n",
            "arrays::exact: Int^3 = [-1, 0, 340282366920938463463374607431768211456]\n",
            "arrays::reverse_twice: Word[16]^3 = [0x0001, 0x0002, 0xbeef]\n",
            "arrays::lanes: Word[64] = 0x0000000000000001\n",
            "arrays::little_endian: Word[16] = 0x1234\n",
            "arrays::single: Word[32]^1 = [0x6a09e667]\n",
            "arrays::indexed_binding: Int = 25\n",
        )),
        rules: &[
            "S3D-GRAMMAR-01",
            "S3D-TYPE-01",
            "S3D-LITERAL-01",
            "S3D-INDEX-01",
            "S3D-OPERATOR-01",
            "S3D-EVAL-01",
            "S3D-DISPLAY-01",
            "S3D-DETERMINISM-01",
        ],
    },
    Case {
        fixture: "valid-chacha20-block.or",
        expectation: Expectation::Success(concat!(
            "chacha20::test_vector: Word[32]^16 = [",
            "0xe4e7f110, 0x15593bd1, 0x1fdd0f50, 0xc47120a3, ",
            "0xc7f4d1c7, 0x0368c033, 0x9aaa2204, 0x4e6cd4c3, ",
            "0x466482d2, 0x09aa9f07, 0x05d7c214, 0xa2028bd9, ",
            "0xd19c12b5, 0xb94e16de, 0xe883d0cb, 0x4e3c50a2]\n",
        )),
        rules: &[
            "S3D-GRAMMAR-01",
            "S3D-TYPE-01",
            "S3D-LITERAL-01",
            "S3D-INDEX-01",
            "S3D-EVAL-01",
            "S3D-DISPLAY-01",
            "S3D-CORE-01",
            "S3D-COMPAT-01",
            "S3D-DETERMINISM-01",
        ],
    },
    Case {
        fixture: "valid-sha256-rounds.or",
        expectation: Expectation::Success(concat!(
            "sha256::initial_hash: Word[32]^8 = [",
            "0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, ",
            "0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19]\n",
            "sha256::abc_block: Word[32]^16 = [",
            "0x61626380, 0x00000000, 0x00000000, 0x00000000, ",
            "0x00000000, 0x00000000, 0x00000000, 0x00000000, ",
            "0x00000000, 0x00000000, 0x00000000, 0x00000000, ",
            "0x00000000, 0x00000000, 0x00000000, 0x00000018]\n",
            "sha256::w16_w17: Word[32]^2 = [0x61626380, 0x000f0000]\n",
            "sha256::after_round0: Word[32]^8 = [",
            "0x5d6aebcd, 0x6a09e667, 0xbb67ae85, 0x3c6ef372, ",
            "0xfa2a4622, 0x510e527f, 0x9b05688c, 0x1f83d9ab]\n",
            "sha256::after_round1: Word[32]^8 = [",
            "0x5a6ad9ad, 0x5d6aebcd, 0x6a09e667, 0xbb67ae85, ",
            "0x78ce7989, 0xfa2a4622, 0x510e527f, 0x9b05688c]\n",
        )),
        rules: &[
            "S3D-GRAMMAR-01",
            "S3D-LITERAL-01",
            "S3D-INDEX-01",
            "S3D-EVAL-01",
            "S3D-DISPLAY-01",
            "S3D-COMPAT-01",
            "S3D-DETERMINISM-01",
        ],
    },
    Case {
        fixture: "invalid-array-syntax.or",
        expectation: Expectation::Failure {
            codes: &["ORC0101", "ORC0101", "ORC0101"],
            locations: &["6:32", "7:45", "8:29"],
            messages: &[
                "expected an array element",
                "Orange 2026 has no empty arrays",
                "expected an operator or the end of the expression",
                "expected the end of the type after its array length",
                "arrays of arrays are not part of Orange 2026",
            ],
        },
        rules: &["S3D-GRAMMAR-01", "S3D-DIAG-01", "S3D-DETERMINISM-01"],
    },
    Case {
        fixture: "invalid-array-types.or",
        expectation: Expectation::Failure {
            codes: &["ORC0221", "ORC0221", "ORC0221", "ORC0221", "ORC0204"],
            locations: &["5:26", "6:29", "7:42", "8:26", "9:21"],
            messages: &[
                "an array length must be a decimal integer from 1 through 65536",
                "`Word` requires an exact width of 8, 16, 32, or 64",
            ],
        },
        rules: &["S3D-TYPE-01", "S3D-DIAG-01", "S3D-DETERMINISM-01"],
    },
    Case {
        fixture: "invalid-array-literals.or",
        expectation: Expectation::Failure {
            codes: &["ORC0222", "ORC0222", "ORC0207", "ORC0214", "ORC0214"],
            locations: &["6:41", "7:30", "8:37", "9:42", "10:36"],
            messages: &[
                "this array has 2 elements, but `Word[8]^3` has 3",
                "this array has 2 elements, but `Word[8]^1` has 1",
                "literal is outside the range of `Word[8]`",
                "an integer literal cannot have type `Word[8]^2`",
                "an array literal cannot have type `Int`",
            ],
        },
        rules: &["S3D-LITERAL-01", "S3D-DIAG-01", "S3D-DETERMINISM-01"],
    },
    Case {
        fixture: "invalid-indices.or",
        expectation: Expectation::Failure {
            codes: &["ORC0223", "ORC0224", "ORC0214", "ORC0214", "ORC0211"],
            locations: &["5:49", "6:42", "7:49", "8:47", "9:27"],
            messages: &[
                "index `16` is out of range for `Word[32]^16`",
                "indices run from 0 through 15",
                "only an array can be indexed, but this has type `Word[32]`",
                "this element has type `Word[8]`, but `Word[32]` is required here",
                "`x` has type `Word[8]^4`, but `Word[8]` is required here",
                "select one element with an index, such as `x[0]`",
                "`y` is not a parameter of `unknown`",
            ],
        },
        rules: &["S3D-INDEX-01", "S3D-DIAG-01", "S3D-DETERMINISM-01"],
    },
    Case {
        fixture: "invalid-array-operators.or",
        expectation: Expectation::Failure {
            codes: &["ORC0215", "ORC0215", "ORC0215", "ORC0215"],
            locations: &["5:60", "6:48", "7:48", "8:46"],
            messages: &[
                "`+` is not defined for `Word[32]^4`",
                "`<<<` is not defined for `Word[32]^4`",
                "prefix `~` is not defined for `Word[8]^2`",
                "`as` is not defined for `Word[8]^4`",
                "apply them to elements, such as `x[0]`",
                "convert each element, such as `x[0] as Int`",
            ],
        },
        rules: &["S3D-OPERATOR-01", "S3D-DIAG-01", "S3D-DETERMINISM-01"],
    },
];

/// Generated command-line cases in this file, each run twice.
const GENERATED_EVIDENCE: &[(&str, &[&str])] = &[(
    "s3d_element_and_length_limits_are_exact",
    &["S3D-RES-ELEM-01", "S3D-DETERMINISM-01"],
)];

const UNIT_EVIDENCE: &[TestEvidence] = &[
    TestEvidence {
        source_path: "src/parser.rs",
        test: "builds_array_types_literals_and_indices_with_exact_spans",
        rules: &["S3D-GRAMMAR-01", "S3D-RES-NEST-01"],
    },
    TestEvidence {
        source_path: "src/parser.rs",
        test: "array_types_parse_only_where_a_type_is_declared",
        rules: &["S3D-GRAMMAR-01", "S3D-COMPAT-01"],
    },
    TestEvidence {
        source_path: "src/parser.rs",
        test: "rejects_malformed_arrays_with_exact_messages",
        rules: &["S3D-GRAMMAR-01", "S3D-DIAG-01"],
    },
    TestEvidence {
        source_path: "src/parser.rs",
        test: "bounds_elements_per_array_literal",
        rules: &["S3D-RES-ELEM-01"],
    },
    TestEvidence {
        source_path: "src/parser.rs",
        test: "bounds_expression_nesting_for_every_opener",
        rules: &["S3D-RES-NEST-01", "S3D-COMPAT-01"],
    },
    TestEvidence {
        source_path: "src/parser.rs",
        test: "array_reservation_failure_returns_no_partial_ast",
        rules: &["S3D-RES-FAIL-01"],
    },
    TestEvidence {
        source_path: "src/parser.rs",
        test: "expression_parsing_is_repeatable_and_malformed_expressions_never_panic",
        rules: &["S3D-GRAMMAR-01", "S3D-DETERMINISM-01"],
    },
    TestEvidence {
        source_path: "src/semantics/tests.rs",
        test: "arrays_and_indices_build_typed_core_in_postorder",
        rules: &[
            "S3D-LITERAL-01",
            "S3D-INDEX-01",
            "S3D-EVAL-01",
            "S3D-CORE-01",
        ],
    },
    TestEvidence {
        source_path: "src/semantics/tests.rs",
        test: "array_lengths_resolve_only_as_exact_decimals_from_1_through_65536",
        rules: &["S3D-TYPE-01", "S3D-DIAG-01"],
    },
    TestEvidence {
        source_path: "src/semantics/tests.rs",
        test: "array_errors_are_reported_once_in_checking_order",
        rules: &[
            "S3D-LITERAL-01",
            "S3D-INDEX-01",
            "S3D-DIAG-01",
            "S3D-DETERMINISM-01",
        ],
    },
    TestEvidence {
        source_path: "src/semantics/tests.rs",
        test: "operators_and_conversions_apply_to_elements_not_arrays",
        rules: &["S3D-OPERATOR-01", "S3D-DIAG-01"],
    },
    TestEvidence {
        source_path: "src/semantics/tests.rs",
        test: "array_events_and_core_nodes_follow_the_normative_accounting",
        rules: &["S3D-RES-EVENT-01"],
    },
    TestEvidence {
        source_path: "src/semantics/tests.rs",
        test: "array_storage_failures_return_no_partial_core",
        rules: &["S3D-RES-FAIL-01"],
    },
    TestEvidence {
        source_path: "src/semantics/tests.rs",
        test: "rejects_foreign_spans_in_arrays_and_indices",
        rules: &["S3D-RES-FAIL-01"],
    },
    TestEvidence {
        source_path: "src/semantics/tests.rs",
        test: "let_and_as_are_ordinary_names_in_semantics",
        rules: &["S3D-COMPAT-01"],
    },
    TestEvidence {
        source_path: "src/core.rs",
        test: "array_types_hold_one_to_65536_scalars_and_display_as_powers",
        rules: &["S3D-TYPE-01", "S3D-DISPLAY-01"],
    },
    TestEvidence {
        source_path: "src/core.rs",
        test: "array_values_require_their_exact_length_and_element_type",
        rules: &["S3D-CORE-01", "S3D-DISPLAY-01"],
    },
    TestEvidence {
        source_path: "src/core.rs",
        test: "core_type_inventory_and_display_are_exact",
        rules: &["S3D-DISPLAY-01", "S3D-COMPAT-01"],
    },
    TestEvidence {
        source_path: "src/eval.rs",
        test: "arrays_evaluate_in_index_order_and_display_every_element",
        rules: &["S3D-EVAL-01", "S3D-DISPLAY-01", "S3D-DETERMINISM-01"],
    },
    TestEvidence {
        source_path: "src/eval.rs",
        test: "chacha20_quarter_round_on_an_array_matches_rfc_8439",
        rules: &["S3D-EVAL-01"],
    },
    TestEvidence {
        source_path: "src/eval.rs",
        test: "steps_follow_the_normative_cost_table",
        rules: &["S3D-RES-STEP-01"],
    },
    TestEvidence {
        source_path: "src/eval.rs",
        test: "deepest_accepted_sources_fit_in_one_mebibyte_of_stack",
        rules: &["S3D-RES-STACK-01"],
    },
    TestEvidence {
        source_path: "src/eval.rs",
        test: "deeply_nested_rejected_arrays_fit_in_one_mebibyte_of_stack",
        rules: &["S3D-RES-STACK-01"],
    },
    TestEvidence {
        source_path: "src/eval.rs",
        test: "inconsistent_arrays_and_indices_fail_closed",
        rules: &["S3D-RES-FAIL-01"],
    },
    TestEvidence {
        source_path: "src/eval.rs",
        test: "array_storage_reservation_failures_return_no_values",
        rules: &["S3D-RES-FAIL-01"],
    },
    TestEvidence {
        source_path: "src/diagnostic.rs",
        test: "diagnostic_code_inventory_is_exact_ordered_and_unique",
        rules: &["S3D-DIAG-01"],
    },
];

fn orangec() -> Command {
    Command::new(env!("CARGO_BIN_EXE_orangec"))
}

fn fixture_directory() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/s3d")
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
    ARRAYS_SPECIFICATION
        .lines()
        .filter_map(|line| {
            let row = line.strip_prefix("| `")?;
            let (rule, _) = row.split_once("` |")?;
            if !rule.starts_with("S3D-") {
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
        _ => panic!("unknown S3d evidence layer {label:?}"),
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
        _ => panic!("unmapped S3d evidence source {source_path}"),
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
        S3D_CONFORMANCE_SOURCE.matches(&declaration).count(),
        1,
        "generated evidence {test} must be declared exactly once at file root"
    );
}

#[test]
fn s3d_rule_index_is_exact_and_covered() {
    let documented = documented_rules();
    let documented_ids: Vec<_> = documented.iter().map(|(rule, _)| *rule).collect();
    assert_eq!(
        documented_ids, RULES,
        "docs/ARRAYS_2026.md rule index drifted from the S3d runner"
    );
    let known: BTreeSet<_> = RULES.iter().copied().collect();
    assert_eq!(known.len(), RULES.len(), "duplicate S3d rule ID");

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
fn s3d_cli_conformance_corpus_is_exact_and_repeatable() {
    let directory = fixture_directory();
    let mut observed: Vec<_> = fs::read_dir(&directory)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().into_string().unwrap())
        .collect();
    observed.sort();
    let mut expected: Vec<_> = CASES.iter().map(|case| case.fixture).collect();
    expected.sort_unstable();
    assert_eq!(observed, expected, "unexpected S3d fixture inventory");

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
fn s3d_element_and_length_limits_are_exact() {
    let literal = |length: usize, count: usize| {
        let elements = (0..count)
            .map(|index| format!("      {},\n", index % 256))
            .collect::<String>();
        module(&format!(
            "  spec many() -> Word[8]^{length} {{\n    [\n{elements}    ]\n  }}\n"
        ))
    };
    let accepted = run_twice("eval", &literal(65_536, 65_536), "65536 elements");
    let values = (0..65_536)
        .map(|index| format!("0x{:02x}", index % 256))
        .collect::<Vec<_>>()
        .join(", ");
    assert_success(
        &accepted,
        &format!("limits::many: Word[8]^65536 = [{values}]\n"),
        "65536 elements",
    );

    let rejected = run_twice("eval", &literal(65_536, 65_537), "65537 elements");
    assert_eq!(rejected.status.code(), Some(1), "65537 elements status");
    assert_eq!(rejected.stdout, b"", "65537 elements emitted output");
    let stderr = String::from_utf8_lossy(&rejected.stderr);
    assert_eq!(diagnostic_codes(&stderr), ["ORC0106"], "{stderr}");
    assert!(
        stderr.contains("array literal has more than 65536 elements"),
        "{stderr}"
    );
    assert_eq!(primary_locations(&stderr), ["65541:7"], "{stderr}");

    let long = run_twice("eval", &literal(65_537, 65_536), "length 65537");
    assert_eq!(long.status.code(), Some(1), "length 65537 status");
    assert_eq!(long.stdout, b"", "length 65537 emitted output");
    let stderr = String::from_utf8_lossy(&long.stderr);
    assert_eq!(diagnostic_codes(&stderr), ["ORC0221"], "{stderr}");
    assert!(
        stderr.contains("an array length must be a decimal integer from 1 through 65536"),
        "{stderr}"
    );
    assert_eq!(primary_locations(&stderr), ["3:26"], "{stderr}");
}
