//! External conformance evidence for the proposed Orange 2026 S3h slice.
//!
//! The rule index lives in `docs/MODULES_2026.md`. This runner requires that
//! index to agree exactly with the evidence map below, runs every program and
//! every module of the fixture directory, and every generated program, through
//! the real `orangec` binary twice, and checks that every named unit test is
//! declared once in its source's test module. S3g compatibility is also
//! observed by the S2 through S3g runners, which run their unchanged fixtures
//! through the same binary.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

const MODULES_SPECIFICATION: &str = include_str!("../../../../docs/MODULES_2026.md");
const S3H_CONFORMANCE_SOURCE: &str = include_str!("s3h_conformance.rs");
const PARSER_SOURCE: &str = include_str!("../../orange-compiler/src/parser.rs");
const SEMANTICS_SOURCE: &str = include_str!("../../orange-compiler/src/semantics.rs");
const CORE_SOURCE: &str = include_str!("../../orange-compiler/src/core.rs");
const DIAGNOSTIC_SOURCE: &str = include_str!("../../orange-compiler/src/diagnostic.rs");
const CLI_SOURCE: &str = include_str!("../src/main.rs");

const CLI: u8 = 1 << 0;
const GENERATED_CLI: u8 = 1 << 1;
const UNIT: u8 = 1 << 2;
const PARSER_UNIT: u8 = 1 << 3;

/// Every S3h rule, in the order of the specification's index.
const RULES: [&str; 10] = [
    "S3H-SYNTAX-01",
    "S3H-GRAPH-01",
    "S3H-ORDER-01",
    "S3H-CALL-01",
    "S3H-CORE-01",
    "S3H-EVAL-01",
    "S3H-CLI-01",
    "S3H-RES-01",
    "S3H-COMPAT-01",
    "S3H-DETERMINISM-01",
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

/// A program: a root file of the fixture directory and what `check` and
/// `eval` give for it.
#[derive(Clone, Copy)]
struct Case {
    fixture: &'static str,
    expectation: Expectation,
    rules: &'static [&'static str],
}

/// A module file of the fixture directory, which the programs use, and what
/// `check` gives for it as a root of its own.
#[derive(Clone, Copy)]
struct Module {
    file: &'static str,
    codes: &'static [&'static str],
}

#[derive(Clone, Copy)]
struct TestEvidence {
    source_path: &'static str,
    test: &'static str,
    rules: &'static [&'static str],
}

const CASES: [Case; 4] = [
    Case {
        fixture: "valid-vectors.or",
        expectation: Expectation::Success(concat!(
            "vectors::abc: Word[8]^32 = [",
            "0xba, 0x78, 0x16, 0xbf, 0x8f, 0x01, 0xcf, 0xea, 0x41, 0x41, 0x40, 0xde, ",
            "0x5d, 0xae, 0x22, 0x23, 0xb0, 0x03, 0x61, 0xa3, 0x96, 0x17, 0x7a, 0x9c, ",
            "0xb4, 0x10, 0xff, 0x61, 0xf2, 0x00, 0x15, 0xad]\n",
            "vectors::hi_there: Word[8]^32 = [",
            "0xb0, 0x34, 0x4c, 0x61, 0xd8, 0xdb, 0x38, 0x53, 0x5c, 0xa8, 0xaf, 0xce, ",
            "0xaf, 0x0b, 0xf1, 0x2b, 0x88, 0x1d, 0xc2, 0x00, 0xc9, 0x83, 0x3d, 0xa7, ",
            "0x26, 0xe9, 0x37, 0x6c, 0x2e, 0x32, 0xcf, 0xf7]\n",
            "vectors::jefe: Word[8]^32 = [",
            "0x5b, 0xdc, 0xc1, 0x46, 0xbf, 0x60, 0x75, 0x4e, 0x6a, 0x04, 0x24, 0x26, ",
            "0x08, 0x95, 0x75, 0xc7, 0x5a, 0x00, 0x3f, 0x08, 0x9d, 0x27, 0x39, 0x83, ",
            "0x9d, 0xec, 0x58, 0xb9, 0x64, 0xec, 0x38, 0x43]\n",
            "vectors::prk: Word[8]^32 = [",
            "0x07, 0x77, 0x09, 0x36, 0x2c, 0x2e, 0x32, 0xdf, 0x0d, 0xdc, 0x3f, 0x0d, ",
            "0xc4, 0x7b, 0xba, 0x63, 0x90, 0xb6, 0xc7, 0x3b, 0xb5, 0x0f, 0x9c, 0x31, ",
            "0x22, 0xec, 0x84, 0x4a, 0xd7, 0xc2, 0xb3, 0xe5]\n",
            "vectors::okm: Word[8]^42 = [",
            "0x3c, 0xb2, 0x5f, 0x25, 0xfa, 0xac, 0xd5, 0x7a, 0x90, 0x43, 0x4f, 0x64, ",
            "0xd0, 0x36, 0x2f, 0x2a, 0x2d, 0x2d, 0x0a, 0x90, 0xcf, 0x1a, 0x5a, 0x4c, ",
            "0x5d, 0xb0, 0x2d, 0x56, 0xec, 0xc4, 0xc5, 0xbf, 0x34, 0x00, 0x72, 0x08, ",
            "0xd5, 0xb8, 0x87, 0x18, 0x58, 0x65]\n",
        )),
        rules: &[
            "S3H-SYNTAX-01",
            "S3H-ORDER-01",
            "S3H-CALL-01",
            "S3H-CORE-01",
            "S3H-EVAL-01",
            "S3H-CLI-01",
            "S3H-COMPAT-01",
            "S3H-DETERMINISM-01",
        ],
    },
    Case {
        fixture: "invalid-graph.or",
        expectation: Expectation::Failure {
            codes: &["ORC0230", "ORC0231", "ORC0228", "ORC0230"],
            locations: &[
                "invalid-graph.or:7:3",
                "invalid-graph.or:9:3",
                "invalid-graph.or:10:7",
                "ring_b.or:4:3",
            ],
            messages: &[
                "module `graph` uses itself",
                "module `sha256` is used twice",
                "no module named `misnamed` in this program",
                "module cycle `ring_a` -> `ring_b` -> `ring_a`",
                "this `use` closes the cycle",
                "`orangec` reads the module `NAME` from the file `NAME.or` beside the file \
                 that uses it",
            ],
        },
        rules: &["S3H-GRAPH-01", "S3H-CLI-01", "S3H-DETERMINISM-01"],
    },
    Case {
        fixture: "invalid-calls.or",
        expectation: Expectation::Failure {
            codes: &[
                "ORC0229", "ORC0229", "ORC0212", "ORC0212", "ORC0213", "ORC0214",
            ],
            locations: &[
                "invalid-calls.or:6:33",
                "invalid-calls.or:7:33",
                "invalid-calls.or:8:42",
                "invalid-calls.or:9:38",
                "invalid-calls.or:10:32",
                "invalid-calls.or:11:35",
            ],
            messages: &[
                "module `hmac` is not used by `calls`",
                "declare `use hmac;` at the head of the module to call its functions",
                "`calls` is the calling module",
                "call a function of the same module without a module name, as in `f(x)`",
                "no typed `spec` function named `final` in module `sha256`",
                "a qualified call names a typed `spec` of the used module",
                "the used module `sha256` declares `initial`; call it as \
                 `sha256::initial(...)`",
                "`compress` takes 2 arguments but 1 was supplied",
                "`initial` returns `Word[32]^8`, but `Word[8]^32` is required here",
            ],
        },
        rules: &["S3H-CALL-01", "S3H-DETERMINISM-01"],
    },
    Case {
        fixture: "invalid-missing.or",
        expectation: Expectation::Failure {
            codes: &["ORC1001"],
            locations: &[],
            messages: &[
                "absent.or`",
                "`use absent;` in module `missing` reads the module `absent` from this file",
            ],
        },
        rules: &["S3H-CLI-01", "S3H-DETERMINISM-01"],
    },
];

const MODULES: [Module; 6] = [
    Module {
        file: "hkdf.or",
        codes: &[],
    },
    Module {
        file: "hmac.or",
        codes: &[],
    },
    Module {
        file: "misnamed.or",
        codes: &[],
    },
    Module {
        file: "ring_a.or",
        codes: &["ORC0230"],
    },
    Module {
        file: "ring_b.or",
        codes: &["ORC0230"],
    },
    Module {
        file: "sha256.or",
        codes: &[],
    },
];

/// Generated command-line cases in this file, each run twice.
const GENERATED_EVIDENCE: &[(&str, &[&str])] = &[
    (
        "s3h_modules_are_read_beside_the_root_once_each",
        &[
            "S3H-ORDER-01",
            "S3H-EVAL-01",
            "S3H-CLI-01",
            "S3H-DETERMINISM-01",
        ],
    ),
    (
        "s3h_module_and_use_limits_are_exact",
        &[
            "S3H-SYNTAX-01",
            "S3H-EVAL-01",
            "S3H-RES-01",
            "S3H-DETERMINISM-01",
        ],
    ),
];

const UNIT_EVIDENCE: &[TestEvidence] = &[
    TestEvidence {
        source_path: "src/parser.rs",
        test: "builds_use_declarations_and_qualified_calls_with_exact_spans",
        rules: &["S3H-SYNTAX-01", "S3H-COMPAT-01"],
    },
    TestEvidence {
        source_path: "src/parser.rs",
        test: "rejects_malformed_use_declarations_and_qualified_names",
        rules: &["S3H-SYNTAX-01"],
    },
    TestEvidence {
        source_path: "src/parser.rs",
        test: "bounds_use_declarations_per_module",
        rules: &["S3H-SYNTAX-01", "S3H-RES-01"],
    },
    TestEvidence {
        source_path: "src/semantics.rs",
        test: "links_used_modules_in_dependency_order_with_dense_identities",
        rules: &[
            "S3H-ORDER-01",
            "S3H-CALL-01",
            "S3H-CORE-01",
            "S3H-EVAL-01",
            "S3H-DETERMINISM-01",
        ],
    },
    TestEvidence {
        source_path: "src/semantics.rs",
        test: "a_chain_of_sixty_four_modules_links_and_evaluates",
        rules: &["S3H-ORDER-01", "S3H-EVAL-01", "S3H-RES-01"],
    },
    TestEvidence {
        source_path: "src/semantics.rs",
        test: "module_graph_errors_name_the_use_that_causes_them",
        rules: &["S3H-GRAPH-01"],
    },
    TestEvidence {
        source_path: "src/semantics.rs",
        test: "qualified_calls_resolve_only_in_used_modules",
        rules: &["S3H-CALL-01"],
    },
    TestEvidence {
        source_path: "src/semantics.rs",
        test: "each_module_is_checked_even_when_a_module_it_uses_has_errors",
        rules: &["S3H-ORDER-01", "S3H-CALL-01"],
    },
    TestEvidence {
        source_path: "src/semantics.rs",
        test: "rejects_foreign_use_and_qualifier_spans_and_foreign_modules",
        rules: &["S3H-RES-01"],
    },
    TestEvidence {
        source_path: "src/semantics.rs",
        test: "names_and_calls_resolve_only_to_parameters_and_typed_specs",
        rules: &["S3H-COMPAT-01"],
    },
    TestEvidence {
        source_path: "src/core.rs",
        test: "core_accessors_preserve_source_order_and_derive_value_types",
        rules: &["S3H-CORE-01"],
    },
    TestEvidence {
        source_path: "src/diagnostic.rs",
        test: "diagnostic_code_inventory_is_exact_ordered_and_unique",
        rules: &["S3H-GRAPH-01", "S3H-CALL-01"],
    },
    TestEvidence {
        source_path: "orangec/src/main.rs",
        test: "used_modules_are_read_once_from_the_root_directory",
        rules: &["S3H-CLI-01"],
    },
];

fn orangec() -> Command {
    Command::new(env!("CARGO_BIN_EXE_orangec"))
}

fn fixture_directory() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/s3h")
}

fn run(command: &str, path: &Path) -> Output {
    orangec().arg(command).arg(path).output().unwrap()
}

/// Runs `command` on standard input in `directory`.
fn run_stdin_in(directory: &Path, command: &str, source: &str) -> Output {
    let mut child = orangec()
        .current_dir(directory)
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
        .join(format!("orangec-s3h-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&directory);
    fs::create_dir(&directory).unwrap();
    directory
}

/// The rule index rows of the specification: rule ID and evidence label.
fn documented_rules() -> Vec<(&'static str, &'static str)> {
    MODULES_SPECIFICATION
        .lines()
        .filter_map(|line| {
            let row = line.strip_prefix("| `")?;
            let (rule, _) = row.split_once("` |")?;
            if !rule.starts_with("S3H-") {
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
        _ => panic!("unknown S3h evidence layer {label:?}"),
    }
}

fn unit_source(source_path: &str) -> &'static str {
    match source_path {
        "src/parser.rs" => PARSER_SOURCE,
        "src/semantics.rs" => SEMANTICS_SOURCE,
        "src/core.rs" => CORE_SOURCE,
        "src/diagnostic.rs" => DIAGNOSTIC_SOURCE,
        "orangec/src/main.rs" => CLI_SOURCE,
        _ => panic!("unmapped S3h evidence source {source_path}"),
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
        S3H_CONFORMANCE_SOURCE.matches(&declaration).count(),
        1,
        "generated evidence {test} must be declared exactly once at file root"
    );
}

#[test]
fn s3h_rule_index_is_exact_and_covered() {
    let documented = documented_rules();
    let documented_ids: Vec<_> = documented.iter().map(|(rule, _)| *rule).collect();
    assert_eq!(
        documented_ids, RULES,
        "docs/MODULES_2026.md rule index drifted from the S3h runner"
    );
    let known: BTreeSet<_> = RULES.iter().copied().collect();
    assert_eq!(known.len(), RULES.len(), "duplicate S3h rule ID");

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
fn s3h_cli_conformance_corpus_is_exact_and_repeatable() {
    let directory = fixture_directory();
    let mut observed: Vec<_> = fs::read_dir(&directory)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().into_string().unwrap())
        .collect();
    observed.sort();
    let mut expected: Vec<_> = CASES
        .iter()
        .map(|case| case.fixture)
        .chain(MODULES.iter().map(|module| module.file))
        .collect();
    expected.sort_unstable();
    assert_eq!(observed, expected, "unexpected S3h fixture inventory");

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

    // A module that other programs use is a program of its own when it is
    // named as the root.
    for module in MODULES {
        let context = format!("{} check", module.file);
        let output = run_twice("check", &directory.join(module.file), &context);
        let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
        assert_eq!(
            diagnostic_codes(&stderr),
            module.codes,
            "{context}:\n{stderr}"
        );
        let status = if module.codes.is_empty() { 0 } else { 1 };
        assert_eq!(output.status.code(), Some(status), "{context}");
        assert_eq!(output.stdout, b"", "{context}");
    }
}

#[test]
fn s3h_modules_are_read_beside_the_root_once_each() {
    let directory = scratch_directory("diamond");
    let files = [
        (
            "base.or",
            "edition 2026;\nmodule base {\n  spec one() -> Int { 1 }\n  \
             spec bad() -> Int { nope }\n}\n",
        ),
        (
            "left.or",
            "edition 2026;\nmodule left {\n  use base;\n  \
             spec two() -> Int { base::one() + 1 }\n}\n",
        ),
        (
            "right.or",
            "edition 2026;\nmodule right {\n  use base;\n  use left;\n  \
             spec three() -> Int { left::two() + base::one() }\n}\n",
        ),
    ];
    for (name, text) in files {
        fs::write(directory.join(name), text).unwrap();
    }
    let root = "edition 2026;\nmodule top {\n  use right;\n  use left;\n  \
                spec six() -> Int { right::three() + left::two() + 1 }\n}\n";
    fs::write(directory.join("top.or"), root).unwrap();

    // `base` is used by three modules and read once: its one error is
    // reported once, before the modules that use it, which are still checked.
    let broken = run_twice("eval", &directory.join("top.or"), "diamond with an error");
    assert_failure(
        &broken,
        &["ORC0211"],
        &["base.or:4:23"],
        &["`nope` is not a parameter of `bad`"],
        "diamond with an error",
    );

    fs::write(
        directory.join("base.or"),
        "edition 2026;\nmodule base {\n  spec one() -> Int { 1 }\n}\n",
    )
    .unwrap();
    let path = directory.join("top.or");
    let program = run_twice("eval", &path, "diamond");
    assert_success(&program, "top::six: Int = 6\n", "diamond");

    // From standard input, modules are read from the current directory, and
    // only the root's values are printed.
    let first = run_stdin_in(&directory, "eval", root);
    let second = run_stdin_in(&directory, "eval", root);
    assert_repeatable(&first, &second, "diamond from standard input");
    assert_success(&first, "top::six: Int = 6\n", "diamond from standard input");
    let module = run_twice("eval", &directory.join("right.or"), "right as the root");
    assert_success(&module, "right::three: Int = 3\n", "right as the root");

    // The file must name the module it holds.
    fs::write(
        directory.join("left.or"),
        "edition 2026;\nmodule other {\n  use base;\n}\n",
    )
    .unwrap();
    let misnamed = run_twice("check", &path, "misnamed module");
    assert_failure(
        &misnamed,
        &["ORC0228", "ORC0228"],
        &["top.or:4:7", "right.or:4:7"],
        &["no module named `left` in this program"],
        "misnamed module",
    );
    fs::remove_dir_all(&directory).unwrap();
}

#[test]
fn s3h_module_and_use_limits_are_exact() {
    // A chain of 64 modules, each adding one to the next, is the largest
    // program; one more module is refused before any module is checked.
    let directory = scratch_directory("chain");
    let write_chain = |count: usize| {
        for index in 0..count {
            let text = if index + 1 == count {
                format!("edition 2026;\nmodule m{index} {{\n  spec v() -> Int {{ 1 }}\n}}\n")
            } else {
                let next = index + 1;
                format!(
                    "edition 2026;\nmodule m{index} {{\n  use m{next};\n  \
                     spec v() -> Int {{ m{next}::v() + 1 }}\n}}\n"
                )
            };
            fs::write(directory.join(format!("m{index}.or")), text).unwrap();
        }
    };
    write_chain(64);
    let root = directory.join("m0.or");
    let chain = run_twice("eval", &root, "chain of 64");
    assert_success(&chain, "m0::v: Int = 64\n", "chain of 64");
    write_chain(65);
    let over = run_twice("eval", &root, "chain of 65");
    assert_failure(
        &over,
        &["ORC0209"],
        &["m0.or:2:1"],
        &["program supplies more than 64 modules"],
        "chain of 65",
    );

    // A module has at most 64 `use` declarations.
    let uses = |count: usize| {
        (1..=count)
            .map(|index| format!("  use m{index};\n"))
            .collect::<String>()
    };
    fs::write(
        directory.join("many.or"),
        format!("edition 2026;\nmodule many {{\n{}}}\n", uses(65)),
    )
    .unwrap();
    let many = run_twice("check", &directory.join("many.or"), "65 uses");
    assert_failure(
        &many,
        &["ORC0106"],
        &["many.or:67:3"],
        &["module has more than 64 `use` declarations"],
        "65 uses",
    );

    // One step budget covers the whole program, whichever module the steps
    // run in: each call of `spin` costs 131,074 steps beyond the call: the loop
    // 1, its initial value 1, and each of its 65,536 iterations 2.
    fs::write(
        directory.join("spin.or"),
        "edition 2026;\nmodule spin {\n  \
         spec spin() -> Int { for i in 0..65536 with s: Int = 0 { s } }\n}\n",
    )
    .unwrap();
    let calls = |count: usize| {
        let sum = vec!["spin::spin()"; count].join(" + ");
        format!(
            "edition 2026;\nmodule busy {{\n  use spin;\n  spec total() -> Int {{ {sum} }}\n}}\n"
        )
    };
    fs::write(directory.join("busy.or"), calls(7)).unwrap();
    let under = run_twice("eval", &directory.join("busy.or"), "seven calls");
    assert_success(&under, "busy::total: Int = 0\n", "seven calls");
    fs::write(directory.join("busy.or"), calls(8)).unwrap();
    let over = run_twice("eval", &directory.join("busy.or"), "eight calls");
    assert_failure(
        &over,
        &["ORC0301"],
        &["busy.or:4:8"],
        &["reference evaluation step limit exceeded"],
        "eight calls",
    );
    fs::remove_dir_all(&directory).unwrap();
}
