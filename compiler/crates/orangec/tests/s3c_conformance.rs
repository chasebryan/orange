//! External conformance evidence for the proposed Orange 2026 S3c slice.
//!
//! The rule index lives in `docs/BINDINGS_2026.md`. This runner requires that
//! index to agree exactly with the evidence map below, runs every fixture and
//! generated source through the real `orangec` binary twice, and checks that
//! every named unit test is declared once in its source's test module.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

const BINDINGS_SPECIFICATION: &str = include_str!("../../../../docs/BINDINGS_2026.md");
const S3C_CONFORMANCE_SOURCE: &str = include_str!("s3c_conformance.rs");
const LEXER_SOURCE: &str = include_str!("../../orange-compiler/src/lexer.rs");
const PARSER_SOURCE: &str = include_str!("../../orange-compiler/src/parser.rs");
const SEMANTICS_SOURCE: &str = include_str!("../../orange-compiler/src/semantics.rs");
const CORE_SOURCE: &str = include_str!("../../orange-compiler/src/core.rs");
const EVAL_SOURCE: &str = include_str!("../../orange-compiler/src/eval.rs");
const DIAGNOSTIC_SOURCE: &str = include_str!("../../orange-compiler/src/diagnostic.rs");

const CLI: u8 = 1 << 0;
const GENERATED_CLI: u8 = 1 << 1;
const UNIT: u8 = 1 << 2;
const PARSER_UNIT: u8 = 1 << 3;

/// Every S3c rule, in the order of the specification's index.
const RULES: [&str; 17] = [
    "S3C-GRAMMAR-01",
    "S3C-CONTEXT-01",
    "S3C-GROUP-01",
    "S3C-SCOPE-01",
    "S3C-BIND-01",
    "S3C-CONV-TYPE-01",
    "S3C-CONV-01",
    "S3C-EVAL-01",
    "S3C-DIAG-01",
    "S3C-CORE-01",
    "S3C-RES-BIND-01",
    "S3C-RES-EVENT-01",
    "S3C-RES-STEP-01",
    "S3C-RES-STACK-01",
    "S3C-RES-FAIL-01",
    "S3C-COMPAT-01",
    "S3C-DETERMINISM-01",
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

const CASES: [Case; 10] = [
    Case {
        fixture: "valid-byte-order.or",
        expectation: Expectation::Success(concat!(
            "bytes::key_word0: Word[32] = 0x03020100\n",
            "bytes::abc_word0: Word[32] = 0x61626380\n",
            "bytes::low_byte: Word[8] = 0x80\n",
            "bytes::high_byte: Word[8] = 0x61\n",
            "bytes::lane: Word[64] = 0xdeadbeefcafef00d\n",
            "bytes::round_trip: Word[32] = 0x9b8d6f43\n",
        )),
        rules: &[
            "S3C-GRAMMAR-01",
            "S3C-GROUP-01",
            "S3C-CONV-TYPE-01",
            "S3C-CONV-01",
            "S3C-EVAL-01",
            "S3C-DETERMINISM-01",
        ],
    },
    Case {
        fixture: "valid-chacha20-quarter-round.or",
        expectation: Expectation::Success(concat!(
            "chacha20::a: Word[32] = 0xea2a92f4\n",
            "chacha20::b: Word[32] = 0xcb1cf8ce\n",
            "chacha20::c: Word[32] = 0x4581472e\n",
            "chacha20::d: Word[32] = 0x5881c4bb\n",
        )),
        rules: &[
            "S3C-GRAMMAR-01",
            "S3C-SCOPE-01",
            "S3C-BIND-01",
            "S3C-EVAL-01",
            "S3C-CORE-01",
            "S3C-DETERMINISM-01",
        ],
    },
    Case {
        fixture: "valid-conversions.or",
        expectation: Expectation::Success(concat!(
            "conversions::wrap_negative: Word[8] = 0xff\n",
            "conversions::wrap_large: Word[16] = 0x0001\n",
            "conversions::wrap_far_negative: Word[64] = 0xffffffffffffffff\n",
            "conversions::unsigned_value: Int = 18446744073709551615\n",
            "conversions::exact_after_conversion: Int = ",
            "340282366920938463426481119284349108225\n",
            "conversions::widen: Word[64] = 0x00000000000000ff\n",
            "conversions::narrow: Word[16] = 0xcdef\n",
            "conversions::identity: Word[32] = 0x6a09e667\n",
            "conversions::carry: Int = 256\n",
            "conversions::ring_sum: Word[8] = 0x00\n",
            "conversions::literal_takes_the_leaf_type: Int = 16\n",
        )),
        rules: &[
            "S3C-BIND-01",
            "S3C-CONV-TYPE-01",
            "S3C-CONV-01",
            "S3C-EVAL-01",
            "S3C-CORE-01",
            "S3C-DETERMINISM-01",
        ],
    },
    Case {
        fixture: "valid-let-and-as-names.or",
        expectation: Expectation::Success(concat!(
            "names::value: Int = 22\n",
            "names::byte: Int = 254\n",
        )),
        rules: &[
            "S3C-CONTEXT-01",
            "S3C-SCOPE-01",
            "S3C-COMPAT-01",
            "S3C-DETERMINISM-01",
        ],
    },
    Case {
        fixture: "valid-sha256-round.or",
        expectation: Expectation::Success(concat!(
            "sha256::w0: Word[32] = 0x61626380\n",
            "sha256::w15: Word[32] = 0x00000018\n",
            "sha256::w17: Word[32] = 0x000f0000\n",
            "sha256::round0_a: Word[32] = 0x5d6aebcd\n",
            "sha256::round0_e: Word[32] = 0xfa2a4622\n",
        )),
        rules: &[
            "S3C-GRAMMAR-01",
            "S3C-SCOPE-01",
            "S3C-BIND-01",
            "S3C-CONV-TYPE-01",
            "S3C-CONV-01",
            "S3C-EVAL-01",
            "S3C-DETERMINISM-01",
        ],
    },
    Case {
        fixture: "invalid-binding-names.or",
        expectation: Expectation::Failure {
            codes: &["ORC0219", "ORC0219", "ORC0211", "ORC0211", "ORC0211"],
            locations: &["5:46", "6:54", "7:42", "8:47", "9:49"],
            messages: &[
                "duplicate binding `x`",
                "the parameter is here",
                "duplicate binding `t`",
                "the first binding is here",
                "`b` is used before it is bound",
                "`n` is used before it is bound",
                "`b` is not a parameter or binding of `unknown`",
                "Orange has no shadowing",
            ],
        },
        rules: &["S3C-SCOPE-01", "S3C-DIAG-01", "S3C-DETERMINISM-01"],
    },
    Case {
        fixture: "invalid-binding-syntax.or",
        expectation: Expectation::Failure {
            codes: &["ORC0101", "ORC0101", "ORC0101"],
            locations: &["5:33", "6:47", "7:45"],
            messages: &[
                "expected `:` and the binding's type",
                "expected `;` after the bound expression",
                "expected a result expression after the last binding",
            ],
        },
        rules: &["S3C-GRAMMAR-01", "S3C-DIAG-01", "S3C-DETERMINISM-01"],
    },
    Case {
        fixture: "invalid-binding-types.or",
        expectation: Expectation::Failure {
            codes: &["ORC0203", "ORC0214", "ORC0207", "ORC0204"],
            locations: &["5:41", "6:51", "7:53", "8:40"],
            messages: &[
                "unsupported binding type `Bool`",
                "`x` has type `Word[8]`, but `Int` is required here",
                "literal is outside the range of `Word[8]`",
                "`Word` requires an exact width of 8, 16, 32, or 64",
            ],
        },
        rules: &["S3C-BIND-01", "S3C-DIAG-01", "S3C-DETERMINISM-01"],
    },
    Case {
        fixture: "invalid-conversions.or",
        expectation: Expectation::Failure {
            codes: &["ORC0220", "ORC0214", "ORC0204", "ORC0214", "ORC0211"],
            locations: &["5:31", "6:52", "7:54", "8:64", "9:36"],
            messages: &[
                "the operand of `as` has no type of its own",
                "this conversion gives `Word[16]`, but `Word[8]` is required here",
                "`Word` width must be exactly 8, 16, 32, or 64",
                "`x` has type `Word[8]`, but `Word[32]` is required here",
                "`z` is not a parameter of `unknown_operand`",
            ],
        },
        rules: &[
            "S3C-BIND-01",
            "S3C-CONV-TYPE-01",
            "S3C-DIAG-01",
            "S3C-DETERMINISM-01",
        ],
    },
    Case {
        fixture: "invalid-ungrouped-conversions.or",
        expectation: Expectation::Failure {
            codes: &["ORC0108", "ORC0108", "ORC0108"],
            locations: &["5:59", "6:54", "7:51"],
            messages: &[
                "`as` follows `+` without grouping parentheses",
                "`<<` follows `as` without grouping parentheses",
                "`as` follows `as` without grouping parentheses",
                "`as` converts exactly one operand",
            ],
        },
        rules: &["S3C-GROUP-01", "S3C-DIAG-01", "S3C-DETERMINISM-01"],
    },
];

/// Generated command-line cases in this file, each run twice.
const GENERATED_EVIDENCE: &[(&str, &[&str])] = &[(
    "s3c_binding_limit_is_exact",
    &["S3C-RES-BIND-01", "S3C-DETERMINISM-01"],
)];

const UNIT_EVIDENCE: &[TestEvidence] = &[
    TestEvidence {
        source_path: "src/parser.rs",
        test: "builds_bindings_and_conversions_with_exact_spans",
        rules: &["S3C-GRAMMAR-01"],
    },
    TestEvidence {
        source_path: "src/parser.rs",
        test: "rejects_malformed_bindings_with_exact_messages",
        rules: &["S3C-GRAMMAR-01", "S3C-DIAG-01"],
    },
    TestEvidence {
        source_path: "src/parser.rs",
        test: "let_and_as_remain_ordinary_names",
        rules: &["S3C-CONTEXT-01", "S3C-COMPAT-01"],
    },
    TestEvidence {
        source_path: "src/parser.rs",
        test: "conversions_apply_to_one_complete_operand",
        rules: &["S3C-GROUP-01"],
    },
    TestEvidence {
        source_path: "src/parser.rs",
        test: "requires_parentheses_around_conversions_with_one_diagnostic",
        rules: &["S3C-GROUP-01", "S3C-DIAG-01"],
    },
    TestEvidence {
        source_path: "src/parser.rs",
        test: "bounds_bindings_per_body",
        rules: &["S3C-RES-BIND-01"],
    },
    TestEvidence {
        source_path: "src/parser.rs",
        test: "binding_reservation_failure_returns_no_partial_ast",
        rules: &["S3C-RES-FAIL-01"],
    },
    TestEvidence {
        source_path: "src/parser.rs",
        test: "expression_parsing_is_repeatable_and_malformed_expressions_never_panic",
        rules: &["S3C-GROUP-01", "S3C-DETERMINISM-01"],
    },
    TestEvidence {
        source_path: "src/semantics.rs",
        test: "bindings_and_conversions_build_typed_core_in_source_order",
        rules: &["S3C-BIND-01", "S3C-CORE-01"],
    },
    TestEvidence {
        source_path: "src/semantics.rs",
        test: "a_conversion_operand_has_the_type_of_its_first_typed_leaf",
        rules: &["S3C-CONV-TYPE-01"],
    },
    TestEvidence {
        source_path: "src/semantics.rs",
        test: "binding_names_are_unique_and_in_scope_only_after_their_binding",
        rules: &["S3C-SCOPE-01", "S3C-DIAG-01"],
    },
    TestEvidence {
        source_path: "src/semantics.rs",
        test: "conversion_errors_are_reported_once_in_checking_order",
        rules: &[
            "S3C-BIND-01",
            "S3C-CONV-TYPE-01",
            "S3C-DIAG-01",
            "S3C-DETERMINISM-01",
        ],
    },
    TestEvidence {
        source_path: "src/semantics.rs",
        test: "calls_inside_bindings_and_conversions_join_the_call_graph",
        rules: &["S3C-EVAL-01", "S3C-DIAG-01"],
    },
    TestEvidence {
        source_path: "src/semantics.rs",
        test: "binding_and_conversion_events_and_core_nodes_follow_the_normative_accounting",
        rules: &["S3C-RES-EVENT-01"],
    },
    TestEvidence {
        source_path: "src/semantics.rs",
        test: "binding_storage_failures_return_no_partial_core",
        rules: &["S3C-RES-FAIL-01"],
    },
    TestEvidence {
        source_path: "src/semantics.rs",
        test: "rejects_foreign_spans_in_bindings_and_conversions",
        rules: &["S3C-RES-FAIL-01"],
    },
    TestEvidence {
        source_path: "src/semantics.rs",
        test: "let_and_as_are_ordinary_names_in_semantics",
        rules: &["S3C-CONTEXT-01", "S3C-COMPAT-01"],
    },
    TestEvidence {
        source_path: "src/semantics.rs",
        test: "names_and_calls_resolve_only_to_parameters_and_typed_specs",
        rules: &["S3C-COMPAT-01"],
    },
    TestEvidence {
        source_path: "src/core.rs",
        test: "core_accessors_preserve_source_order_and_derive_value_types",
        rules: &["S3C-CORE-01"],
    },
    TestEvidence {
        source_path: "src/core.rs",
        test: "word_conversions_match_an_i128_reference",
        rules: &["S3C-CONV-01"],
    },
    TestEvidence {
        source_path: "src/eval.rs",
        test: "conversions_match_a_wide_reference_for_every_type_pair",
        rules: &["S3C-CONV-01"],
    },
    TestEvidence {
        source_path: "src/eval.rs",
        test: "bindings_are_evaluated_once_in_order_and_read_from_their_slots",
        rules: &["S3C-EVAL-01", "S3C-DETERMINISM-01"],
    },
    TestEvidence {
        source_path: "src/eval.rs",
        test: "steps_follow_the_normative_cost_table",
        rules: &["S3C-RES-STEP-01"],
    },
    TestEvidence {
        source_path: "src/eval.rs",
        test: "deepest_accepted_sources_fit_in_one_mebibyte_of_stack",
        rules: &["S3C-RES-STACK-01"],
    },
    TestEvidence {
        source_path: "src/eval.rs",
        test: "inconsistent_locals_and_conversions_fail_closed",
        rules: &["S3C-RES-FAIL-01"],
    },
    TestEvidence {
        source_path: "src/eval.rs",
        test: "word_to_int_conversion_allocation_failure_returns_no_values",
        rules: &["S3C-RES-FAIL-01"],
    },
    TestEvidence {
        source_path: "src/diagnostic.rs",
        test: "diagnostic_code_inventory_is_exact_ordered_and_unique",
        rules: &["S3C-DIAG-01"],
    },
];

fn orangec() -> Command {
    Command::new(env!("CARGO_BIN_EXE_orangec"))
}

fn fixture_directory() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/s3c")
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
    BINDINGS_SPECIFICATION
        .lines()
        .filter_map(|line| {
            let row = line.strip_prefix("| `")?;
            let (rule, _) = row.split_once("` |")?;
            if !rule.starts_with("S3C-") {
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
        _ => panic!("unknown S3c evidence layer {label:?}"),
    }
}

fn unit_source(source_path: &str) -> &'static str {
    match source_path {
        "src/lexer.rs" => LEXER_SOURCE,
        "src/parser.rs" => PARSER_SOURCE,
        "src/semantics.rs" => SEMANTICS_SOURCE,
        "src/core.rs" => CORE_SOURCE,
        "src/eval.rs" => EVAL_SOURCE,
        "src/diagnostic.rs" => DIAGNOSTIC_SOURCE,
        _ => panic!("unmapped S3c evidence source {source_path}"),
    }
}

/// Requires `test` to be declared exactly once, as a `#[test]` function
/// directly inside the source's single `#[cfg(test)] mod tests` module.
fn assert_unit_test_declared(source_path: &str, test: &str) {
    let source = unit_source(source_path);
    let marker = "\n#[cfg(test)]\nmod tests {\n";
    assert_eq!(
        source.matches(marker).count(),
        1,
        "{source_path} must have exactly one unconditional test module"
    );
    let (_, tests) = source.split_once(marker).unwrap();
    let declaration = format!("\n    #[test]\n    fn {test}() {{\n");
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
        S3C_CONFORMANCE_SOURCE.matches(&declaration).count(),
        1,
        "generated evidence {test} must be declared exactly once at file root"
    );
}

#[test]
fn s3c_rule_index_is_exact_and_covered() {
    let documented = documented_rules();
    let documented_ids: Vec<_> = documented.iter().map(|(rule, _)| *rule).collect();
    assert_eq!(
        documented_ids, RULES,
        "docs/BINDINGS_2026.md rule index drifted from the S3c runner"
    );
    let known: BTreeSet<_> = RULES.iter().copied().collect();
    assert_eq!(known.len(), RULES.len(), "duplicate S3c rule ID");

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
fn s3c_cli_conformance_corpus_is_exact_and_repeatable() {
    let directory = fixture_directory();
    let mut observed: Vec<_> = fs::read_dir(&directory)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().into_string().unwrap())
        .collect();
    observed.sort();
    let mut expected: Vec<_> = CASES.iter().map(|case| case.fixture).collect();
    expected.sort_unstable();
    assert_eq!(observed, expected, "unexpected S3c fixture inventory");

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
fn s3c_binding_limit_is_exact() {
    let body = |count: usize| {
        let bindings = (0..count)
            .map(|index| format!("    let v{index}: Int = {index};\n"))
            .collect::<String>();
        module(&format!("  spec many() -> Int {{\n{bindings}    0\n  }}\n"))
    };
    let accepted = run_twice("eval", &body(256), "256 bindings");
    assert_success(&accepted, "limits::many: Int = 0\n", "256 bindings");
    let rejected = run_twice("eval", &body(257), "257 bindings");
    assert_eq!(rejected.status.code(), Some(1), "257 bindings status");
    assert_eq!(rejected.stdout, b"", "257 bindings emitted output");
    let stderr = String::from_utf8_lossy(&rejected.stderr);
    assert_eq!(diagnostic_codes(&stderr), ["ORC0106"], "{stderr}");
    assert!(
        stderr.contains("typed body declares more than 256 bindings"),
        "{stderr}"
    );
    assert_eq!(primary_locations(&stderr), ["260:5"], "{stderr}");
}
