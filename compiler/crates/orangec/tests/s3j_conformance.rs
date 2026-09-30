//! External conformance evidence for the proposed Orange 2026 S3j slice.
//!
//! The rule index lives in `docs/BLOCKS_2026.md`. This runner requires that
//! index to agree exactly with the evidence map below, runs every fixture and
//! every generated program through the real `orangec` binary twice, and
//! checks that every named unit test is declared once in its source's test
//! module. S3i compatibility is also observed by the S2 through S3i runners,
//! which run their fixtures through the same binary.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const BLOCKS_SPECIFICATION: &str = include_str!("../../../../docs/BLOCKS_2026.md");
const S3J_CONFORMANCE_SOURCE: &str = include_str!("s3j_conformance.rs");
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

/// Every S3j rule, in the order of the specification's index.
const RULES: [&str; 8] = [
    "S3J-SYNTAX-01",
    "S3J-SCOPE-01",
    "S3J-TYPE-01",
    "S3J-CORE-01",
    "S3J-EVAL-01",
    "S3J-RES-01",
    "S3J-COMPAT-01",
    "S3J-DETERMINISM-01",
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

const CASES: [Case; 6] = [
    Case {
        fixture: "valid-sha256.or",
        expectation: Expectation::Success(concat!(
            "sha256::round_constants: Word[32]^64 = [",
            "0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, ",
            "0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, ",
            "0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, ",
            "0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, ",
            "0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, ",
            "0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b, ",
            "0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116, ",
            "0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3, ",
            "0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, ",
            "0xc67178f2]\n",
            "sha256::initial_hash: Word[32]^8 = [",
            "0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab, ",
            "0x5be0cd19]\n",
            "sha256::abc_block: Word[32]^16 = [",
            "0x61626380, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, ",
            "0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, ",
            "0x00000000, 0x00000018]\n",
            "sha256::abc_digest: Word[32]^8 = [",
            "0xba7816bf, 0x8f01cfea, 0x414140de, 0x5dae2223, 0xb00361a3, 0x96177a9c, 0xb410ff61, ",
            "0xf20015ad]\n",
            "sha256::long_first_block: Word[32]^16 = [",
            "0x61626364, 0x62636465, 0x63646566, 0x64656667, 0x65666768, 0x66676869, 0x6768696a, ",
            "0x68696a6b, 0x696a6b6c, 0x6a6b6c6d, 0x6b6c6d6e, 0x6c6d6e6f, 0x6d6e6f70, 0x6e6f7071, ",
            "0x80000000, 0x00000000]\n",
            "sha256::long_second_block: Word[32]^16 = [",
            "0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, ",
            "0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, ",
            "0x00000000, 0x000001c0]\n",
            "sha256::long_digest: Word[32]^8 = [",
            "0x248d6a61, 0xd20638b8, 0xe5c02693, 0x0c3e6039, 0xa33ce459, 0x64ff2167, 0xf6ecedd4, ",
            "0x19db06c1]\n",
        )),
        rules: &[
            "S3J-SYNTAX-01",
            "S3J-SCOPE-01",
            "S3J-TYPE-01",
            "S3J-CORE-01",
            "S3J-EVAL-01",
            "S3J-DETERMINISM-01",
        ],
    },
    Case {
        fixture: "valid-x25519.or",
        expectation: Expectation::Success(concat!(
            "x25519::test_vector: Word[8]^32 = [",
            "0xc3, 0xda, 0x55, 0x37, 0x9d, 0xe9, 0xc6, 0x90, 0x8e, 0x94, 0xea, 0x4d, ",
            "0xf2, 0x8d, 0x08, 0x4f, 0x32, 0xec, 0xcf, 0x03, 0x49, 0x1c, 0x71, 0xf7, ",
            "0x54, 0xb4, 0x07, 0x55, 0x77, 0xa2, 0x85, 0x52]\n",
        )),
        rules: &[
            "S3J-SYNTAX-01",
            "S3J-SCOPE-01",
            "S3J-TYPE-01",
            "S3J-CORE-01",
            "S3J-EVAL-01",
            "S3J-DETERMINISM-01",
        ],
    },
    Case {
        fixture: "valid-blocks.or",
        expectation: Expectation::Success(concat!(
            "blocks::squares: Int = 285\n",
            "blocks::classified: Int^3 = [700, 0, 56]\n",
            "blocks::triangle: Int = 70\n",
            "blocks::horner: Mod[3329] = 580\n",
            "blocks::twice: Word[8] = 0x65\n",
        )),
        rules: &[
            "S3J-SYNTAX-01",
            "S3J-SCOPE-01",
            "S3J-TYPE-01",
            "S3J-CORE-01",
            "S3J-EVAL-01",
            "S3J-DETERMINISM-01",
        ],
    },
    Case {
        fixture: "invalid-block-syntax.or",
        expectation: Expectation::Failure {
            codes: &["ORC0101", "ORC0101", "ORC0101", "ORC0101"],
            locations: &[
                "invalid-block-syntax.or:5:82",
                "invalid-block-syntax.or:6:69",
                "invalid-block-syntax.or:7:65",
                "invalid-block-syntax.or:8:71",
            ],
            messages: &[
                "expected a value after the last binding",
                "a loop's step holds `let` bindings, if any, and then the expression that gives \
                 the accumulator's next value",
                "expected `;` after the bound expression",
                "each binding ends with `;`; the block's last item is its value",
                "expected `}` after the value",
                "each branch of a conditional holds `let` bindings, if any, and then its value",
                "expected `:` and the binding's type",
            ],
        },
        rules: &["S3J-SYNTAX-01", "S3J-COMPAT-01", "S3J-DETERMINISM-01"],
    },
    Case {
        fixture: "invalid-block-names.or",
        expectation: Expectation::Failure {
            codes: &[
                "ORC0219", "ORC0219", "ORC0219", "ORC0219", "ORC0211", "ORC0211", "ORC0211",
            ],
            locations: &[
                "invalid-block-names.or:5:71",
                "invalid-block-names.or:6:61",
                "invalid-block-names.or:7:59",
                "invalid-block-names.or:8:58",
                "invalid-block-names.or:9:70",
                "invalid-block-names.or:10:68",
                "invalid-block-names.or:11:91",
            ],
            messages: &[
                "duplicate name `x`",
                "the parameter is here",
                "duplicate name `i`",
                "the loop index is here",
                "duplicate name `t`",
                "the binding is here",
                "each parameter, binding, loop index, and accumulator in scope has its own \
                 name; Orange has no shadowing",
                "`b` is used before it is bound",
                "a binding is in scope after its own `;`, for the bindings that follow it and \
                 the value of its step or branch",
                "`t` is not in scope here",
                "a binding of this name is here",
                "a binding of a loop's step or a branch is in scope only within that step or \
                 branch",
            ],
        },
        rules: &["S3J-SCOPE-01", "S3J-DETERMINISM-01"],
    },
    Case {
        fixture: "invalid-block-types.or",
        expectation: Expectation::Failure {
            codes: &["ORC0214", "ORC0214", "ORC0203", "ORC0220"],
            locations: &[
                "invalid-block-types.or:5:94",
                "invalid-block-types.or:6:69",
                "invalid-block-types.or:7:72",
                "invalid-block-types.or:9:5",
            ],
            messages: &[
                "`s` has type `Word[8]`, but `Word[32]` is required here",
                "`t` has type `Word[8]`, but `Word[32]` is required here",
                "unsupported binding type `Wide`",
                "the operand of `as` has no type of its own",
                "a branch's own bindings are not in scope outside it",
                "bind the conditional's value with a typed `let` first, or convert within each \
                 branch",
            ],
        },
        rules: &["S3J-TYPE-01", "S3J-DETERMINISM-01"],
    },
];

/// Generated command-line cases in this file, each run twice.
const GENERATED_EVIDENCE: &[(&str, &[&str])] = &[(
    "s3j_bindings_per_block_are_exact_at_the_limit",
    &[
        "S3J-SYNTAX-01",
        "S3J-SCOPE-01",
        "S3J-EVAL-01",
        "S3J-RES-01",
        "S3J-DETERMINISM-01",
    ],
)];

const UNIT_EVIDENCE: &[TestEvidence] = &[
    TestEvidence {
        source_path: "src/parser.rs",
        test: "builds_blocks_in_steps_and_branches_with_exact_spans",
        rules: &["S3J-SYNTAX-01"],
    },
    TestEvidence {
        source_path: "src/parser.rs",
        test: "rejects_malformed_blocks_with_exact_messages",
        rules: &["S3J-SYNTAX-01"],
    },
    TestEvidence {
        source_path: "src/parser.rs",
        test: "bounds_bindings_per_block",
        rules: &["S3J-SYNTAX-01", "S3J-RES-01"],
    },
    TestEvidence {
        source_path: "src/parser.rs",
        test: "block_bindings_count_toward_the_height_of_their_expression",
        rules: &["S3J-SYNTAX-01", "S3J-RES-01"],
    },
    TestEvidence {
        source_path: "src/parser.rs",
        test: "rejects_malformed_loops_updates_and_fills_with_exact_messages",
        rules: &["S3J-COMPAT-01"],
    },
    TestEvidence {
        source_path: "src/parser.rs",
        test: "rejects_malformed_conditionals_with_exact_messages",
        rules: &["S3J-COMPAT-01"],
    },
    TestEvidence {
        source_path: "src/semantics/tests.rs",
        test: "blocks_build_typed_core_with_each_binding_before_its_value",
        rules: &["S3J-SCOPE-01", "S3J-TYPE-01", "S3J-CORE-01"],
    },
    TestEvidence {
        source_path: "src/semantics/tests.rs",
        test: "block_names_are_unique_and_in_scope_only_within_their_block",
        rules: &["S3J-SCOPE-01", "S3J-DETERMINISM-01"],
    },
    TestEvidence {
        source_path: "src/semantics/tests.rs",
        test: "a_branch_binding_gives_no_type_where_the_branch_is_a_leaf",
        rules: &["S3J-TYPE-01", "S3J-COMPAT-01", "S3J-DETERMINISM-01"],
    },
    TestEvidence {
        source_path: "src/semantics/tests.rs",
        test: "unresolved_block_binding_types_are_reported_once_without_cascades",
        rules: &["S3J-TYPE-01", "S3J-COMPAT-01"],
    },
    TestEvidence {
        source_path: "src/semantics/tests.rs",
        test: "condition_and_comparison_errors_are_reported_once_in_checking_order",
        rules: &["S3J-COMPAT-01"],
    },
    TestEvidence {
        source_path: "src/semantics/tests.rs",
        test: "block_events_and_core_nodes_follow_the_normative_accounting",
        rules: &["S3J-RES-01"],
    },
    TestEvidence {
        source_path: "src/semantics/tests.rs",
        test: "rejects_foreign_spans_in_blocks",
        rules: &["S3J-RES-01"],
    },
    TestEvidence {
        source_path: "src/core.rs",
        test: "block_bindings_and_their_reads_keep_their_positions",
        rules: &["S3J-CORE-01"],
    },
    TestEvidence {
        source_path: "src/eval.rs",
        test: "block_bindings_are_evaluated_at_every_step_and_read_from_their_slots",
        rules: &["S3J-EVAL-01"],
    },
    TestEvidence {
        source_path: "src/eval.rs",
        test: "only_the_chosen_branch_binds_its_names",
        rules: &["S3J-EVAL-01"],
    },
    TestEvidence {
        source_path: "src/eval.rs",
        test: "steps_follow_the_normative_cost_table",
        rules: &["S3J-EVAL-01"],
    },
    TestEvidence {
        source_path: "src/eval.rs",
        test: "inconsistent_blocks_fail_closed",
        rules: &["S3J-CORE-01", "S3J-RES-01"],
    },
    TestEvidence {
        source_path: "src/eval.rs",
        test: "deepest_accepted_sources_fit_in_one_mebibyte_of_stack",
        rules: &["S3J-RES-01"],
    },
];

fn orangec() -> Command {
    Command::new(env!("CARGO_BIN_EXE_orangec"))
}

fn fixture_directory() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/s3j")
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
        .join(format!("orangec-s3j-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&directory);
    fs::create_dir(&directory).unwrap();
    directory
}

/// The rule index rows of the specification: rule ID and evidence label.
fn documented_rules() -> Vec<(&'static str, &'static str)> {
    BLOCKS_SPECIFICATION
        .lines()
        .filter_map(|line| {
            let row = line.strip_prefix("| `")?;
            let (rule, _) = row.split_once("` |")?;
            if !rule.starts_with("S3J-") {
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
        _ => panic!("unknown S3j evidence layer {label:?}"),
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
        _ => panic!("unmapped S3j evidence source {source_path}"),
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
        S3J_CONFORMANCE_SOURCE.matches(&declaration).count(),
        1,
        "generated evidence {test} must be declared exactly once at file root"
    );
}

#[test]
fn s3j_rule_index_is_exact_and_covered() {
    let documented = documented_rules();
    let documented_ids: Vec<_> = documented.iter().map(|(rule, _)| *rule).collect();
    assert_eq!(
        documented_ids, RULES,
        "docs/BLOCKS_2026.md rule index drifted from the S3j runner"
    );
    let known: BTreeSet<_> = RULES.iter().copied().collect();
    assert_eq!(known.len(), RULES.len(), "duplicate S3j rule ID");

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
fn s3j_cli_conformance_corpus_is_exact_and_repeatable() {
    let directory = fixture_directory();
    let mut observed: Vec<_> = fs::read_dir(&directory)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().into_string().unwrap())
        .collect();
    observed.sort();
    let mut expected: Vec<_> = CASES.iter().map(|case| case.fixture).collect();
    expected.sort_unstable();
    assert_eq!(observed, expected, "unexpected S3j fixture inventory");

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
fn s3j_bindings_per_block_are_exact_at_the_limit() {
    let directory = scratch_directory("limits");

    // A step and a branch may each hold 256 bindings, each in scope for the
    // bindings after it; a 257th is a parser resource limit at its binding.
    let bindings = |count: usize, first: &str| {
        (0..count)
            .map(|index| {
                if index == 0 {
                    format!("      let v0: Int = {first};\n")
                } else {
                    format!("      let v{index}: Int = v{} + 1;\n", index - 1)
                }
            })
            .collect::<String>()
    };
    let program = |step: usize, branch: usize| {
        format!(
            "edition 2026;\nmodule limits {{\n  spec step() -> Int {{\n    \
             for i in 0..2 with s: Int = 0 {{\n{}      v{} + 1\n    }}\n  }}\n  \
             spec branch() -> Int {{\n    if true {{\n{}      v{}\n    }} else {{\n      0\n    \
             }}\n  }}\n}}\n",
            bindings(step, "s"),
            step - 1,
            bindings(branch, "0"),
            branch - 1,
        )
    };
    let path = directory.join("limits.or");
    fs::write(&path, program(256, 256)).unwrap();
    let most = run_twice("eval", &path, "256 bindings per block");
    assert_success(
        &most,
        "limits::step: Int = 512\nlimits::branch: Int = 255\n",
        "256 bindings per block",
    );
    for (step, branch, location) in [(257, 256, "limits.or:261:7"), (256, 257, "limits.or:522:7")] {
        fs::write(&path, program(step, branch)).unwrap();
        let context = format!("{step} and {branch} bindings");
        let over = run_twice("check", &path, &context);
        assert_failure(
            &over,
            &["ORC0106"],
            &[location],
            &["a block declares more than 256 bindings"],
            &context,
        );
    }
    fs::remove_dir_all(&directory).unwrap();
}
