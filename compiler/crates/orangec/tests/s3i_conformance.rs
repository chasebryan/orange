//! External conformance evidence for the proposed Orange 2026 S3i slice.
//!
//! The rule index lives in `docs/MODULAR_2026.md`. This runner requires that
//! index to agree exactly with the evidence map below, runs every fixture and
//! every generated program through the real `orangec` binary twice, and
//! checks that every named unit test is declared once in its source's test
//! module. S3h compatibility is also observed by the S2 through S3h runners,
//! which run their unchanged fixtures through the same binary.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const MODULAR_SPECIFICATION: &str = include_str!("../../../../docs/MODULAR_2026.md");
const S3I_CONFORMANCE_SOURCE: &str = include_str!("s3i_conformance.rs");
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

/// Every S3i rule, in the order of the specification's index.
const RULES: [&str; 13] = [
    "S3I-SYNTAX-01",
    "S3I-MODULUS-01",
    "S3I-NAMES-01",
    "S3I-LITERAL-01",
    "S3I-OPERATOR-01",
    "S3I-DIVISION-01",
    "S3I-CONVERT-01",
    "S3I-INDEX-01",
    "S3I-CORE-01",
    "S3I-EVAL-01",
    "S3I-RES-01",
    "S3I-COMPAT-01",
    "S3I-DETERMINISM-01",
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

const CASES: [Case; 7] = [
    Case {
        fixture: "valid-x25519.or",
        expectation: Expectation::Success(concat!(
            "x25519::test_vector: Word[8]^32 = [",
            "0xc3, 0xda, 0x55, 0x37, 0x9d, 0xe9, 0xc6, 0x90, 0x8e, 0x94, 0xea, 0x4d, ",
            "0xf2, 0x8d, 0x08, 0x4f, 0x32, 0xec, 0xcf, 0x03, 0x49, 0x1c, 0x71, 0xf7, ",
            "0x54, 0xb4, 0x07, 0x55, 0x77, 0xa2, 0x85, 0x52]\n",
        )),
        rules: &[
            "S3I-SYNTAX-01",
            "S3I-MODULUS-01",
            "S3I-NAMES-01",
            "S3I-OPERATOR-01",
            "S3I-DIVISION-01",
            "S3I-CONVERT-01",
            "S3I-CORE-01",
            "S3I-EVAL-01",
            "S3I-COMPAT-01",
            "S3I-DETERMINISM-01",
        ],
    },
    Case {
        fixture: "valid-poly1305.or",
        expectation: Expectation::Success(concat!(
            "poly1305::example: Word[8]^16 = [",
            "0xa8, 0x06, 0x1d, 0xc1, 0x30, 0x51, 0x36, 0xc6, ",
            "0xc2, 0x2b, 0x8b, 0xaf, 0x0c, 0x01, 0x27, 0xa9]\n",
        )),
        rules: &[
            "S3I-SYNTAX-01",
            "S3I-MODULUS-01",
            "S3I-NAMES-01",
            "S3I-OPERATOR-01",
            "S3I-CONVERT-01",
            "S3I-CORE-01",
            "S3I-EVAL-01",
            "S3I-COMPAT-01",
            "S3I-DETERMINISM-01",
        ],
    },
    Case {
        fixture: "valid-fields.or",
        expectation: Expectation::Success(concat!(
            "fields::zeta_128: Mod[3329] = 3328\n",
            "fields::scale: Mod[3329] = 3303\n",
            "fields::d: Mod[(1 << 255) - 19] = ",
            "37095705934669439343138083508754565189542113879843219016388785533085940283555\n",
            "fields::sqrt_m1: Mod[(1 << 255) - 19] = ",
            "19681161376707505956807079304988542015446066515923890162744021073123829784752\n",
            "fields::sqrt_m1_squared: Mod[(1 << 255) - 19] = ",
            "57896044618658097711785492504343953926634992332820282019728792003956564819948\n",
            "fields::p256_generator_on_curve: Bool = true\n",
            "fields::p256_minus_three: ",
            "Mod[0xffffffff00000001000000000000000000000000ffffffffffffffffffffffff] = ",
            "115792089210356248762697446949407573530086143415290314195533631308867097853948\n",
            "fields::bytes: Mod[256]^4 = [0, 1, 0, 171]\n",
            "fields::negatives: Mod[7]^4 = [6, 1, 0, 4]\n",
            "fields::conversions: Int^4 = [4, 3, 6, 6]\n",
            "fields::wrapped: Word[8] = 0x00\n",
            "fields::selected: Word[8]^2 = [0x16, 0x10]\n",
        )),
        rules: &[
            "S3I-SYNTAX-01",
            "S3I-MODULUS-01",
            "S3I-NAMES-01",
            "S3I-LITERAL-01",
            "S3I-OPERATOR-01",
            "S3I-DIVISION-01",
            "S3I-CONVERT-01",
            "S3I-INDEX-01",
            "S3I-CORE-01",
            "S3I-EVAL-01",
            "S3I-DETERMINISM-01",
        ],
    },
    Case {
        fixture: "invalid-type-syntax.or",
        expectation: Expectation::Failure {
            codes: &["ORC0103", "ORC0103", "ORC0101"],
            locations: &[
                "invalid-type-syntax.or:7:3",
                "invalid-type-syntax.or:9:3",
                "invalid-type-syntax.or:10:18",
            ],
            messages: &[
                "a `use` declaration cannot follow a `type` declaration",
                "a module's `use` declarations come first, then its `type` declarations, then \
                 its functions",
                "a `type` declaration cannot follow a function",
                "`type` declarations come before a module's functions",
                "expected `]` after the modulus",
            ],
        },
        rules: &["S3I-SYNTAX-01", "S3I-DETERMINISM-01"],
    },
    Case {
        fixture: "invalid-moduli.or",
        expectation: Expectation::Failure {
            codes: &[
                "ORC0232", "ORC0232", "ORC0232", "ORC0232", "ORC0232", "ORC0232", "ORC0232",
                "ORC0205", "ORC0232",
            ],
            locations: &[
                "invalid-moduli.or:5:19",
                "invalid-moduli.or:6:24",
                "invalid-moduli.or:7:20",
                "invalid-moduli.or:8:21",
                "invalid-moduli.or:9:23",
                "invalid-moduli.or:10:16",
                "invalid-moduli.or:11:24",
                "invalid-moduli.or:12:21",
                "invalid-moduli.or:13:49",
            ],
            messages: &[
                "a modulus must be a constant from 2 through 2^521 - 1",
                "this modulus is 1",
                "this modulus is -5",
                "this modulus has 522 bits",
                "not a constant integer expression",
                "`Mod` requires a modulus",
                "missing modulus",
                "a shift amount in a modulus is from 0 through 16384",
                "this value of the modulus is too large",
                "a modulus is a constant built from integer literals with `+`, `-`, `*`, `<<`, \
                 and parentheses, as in `Mod[(1 << 255) - 19]`",
            ],
        },
        rules: &["S3I-MODULUS-01", "S3I-RES-01", "S3I-DETERMINISM-01"],
    },
    Case {
        fixture: "invalid-types.or",
        expectation: Expectation::Failure {
            codes: &[
                "ORC0233", "ORC0233", "ORC0233", "ORC0203", "ORC0221", "ORC0203", "ORC0203",
            ],
            locations: &[
                "invalid-types.or:6:8",
                "invalid-types.or:7:8",
                "invalid-types.or:9:8",
                "invalid-types.or:10:12",
                "invalid-types.or:13:17",
                "invalid-types.or:15:13",
                "invalid-types.or:16:13",
            ],
            messages: &[
                "`Int` is a built-in type",
                "`Mod` is a built-in type",
                "a `type` declaration cannot name a built-in type",
                "the built-in types are `Int`, `Bool`, `Word[n]`, and `Mod[m]`",
                "duplicate type name `K`",
                "first declaration is here",
                "each `type` declaration of a module names a different type",
                "`M` is declared by a later `type` declaration; a `type` declaration uses only \
                 the names declared before it",
                "an array holds at most 65536 scalars in all, but this one would hold 131072",
                "this length multiplies the rows",
                "an array has at most 4 dimensions",
                "this length would add a fifth dimension",
                "the admitted types are `Int`, `Bool`, `Word[8]`, `Word[16]`, `Word[32]`, \
                 `Word[64]`, `Mod[m]`, and the names of earlier `type` declarations",
            ],
        },
        rules: &["S3I-NAMES-01", "S3I-COMPAT-01", "S3I-DETERMINISM-01"],
    },
    Case {
        fixture: "invalid-residues.or",
        expectation: Expectation::Failure {
            codes: &[
                "ORC0215", "ORC0215", "ORC0215", "ORC0215", "ORC0215", "ORC0207", "ORC0207",
                "ORC0214", "ORC0214", "ORC0215", "ORC0215", "ORC0214",
            ],
            locations: &[
                "invalid-residues.or:10:38",
                "invalid-residues.or:11:33",
                "invalid-residues.or:12:32",
                "invalid-residues.or:13:28",
                "invalid-residues.or:14:29",
                "invalid-residues.or:15:23",
                "invalid-residues.or:16:27",
                "invalid-residues.or:17:43",
                "invalid-residues.or:18:44",
                "invalid-residues.or:19:32",
                "invalid-residues.or:20:35",
                "invalid-residues.or:23:7",
            ],
            messages: &[
                "`<` is not defined for `Mod[7]`",
                "residues are compared with `==` and `!=`; they have no order, so compare least \
                 residues, such as `(x as Int) < (y as Int)`",
                "a residue is already reduced; `%` applies to `Int` and word values, such as \
                 `(x as Int) % 16`",
                "literal is outside the range of `Mod[7]`",
                "the literal's magnitude is not less than the modulus",
                "`y` has type `Mod[11]`, but `Mod[7]` is required here",
                "`as` converts one `Int`, word, or residue value",
                "`as` does not convert to the array type `Mod[7]^2`",
                "`as` gives one `Int`, word, or residue value",
                "`x` has type `Mod[7]`, but `Int` is required here",
            ],
        },
        rules: &[
            "S3I-LITERAL-01",
            "S3I-OPERATOR-01",
            "S3I-CONVERT-01",
            "S3I-INDEX-01",
            "S3I-COMPAT-01",
            "S3I-DETERMINISM-01",
        ],
    },
];

/// Generated command-line cases in this file, each run twice.
const GENERATED_EVIDENCE: &[(&str, &[&str])] = &[
    (
        "s3i_type_declaration_and_modulus_limits_are_exact",
        &[
            "S3I-SYNTAX-01",
            "S3I-MODULUS-01",
            "S3I-NAMES-01",
            "S3I-DIVISION-01",
            "S3I-EVAL-01",
            "S3I-RES-01",
            "S3I-DETERMINISM-01",
        ],
    ),
    (
        "s3i_literals_and_indices_are_exact_at_their_edges",
        &[
            "S3I-LITERAL-01",
            "S3I-CONVERT-01",
            "S3I-INDEX-01",
            "S3I-EVAL-01",
            "S3I-DETERMINISM-01",
        ],
    ),
];

const UNIT_EVIDENCE: &[TestEvidence] = &[
    TestEvidence {
        source_path: "src/parser.rs",
        test: "builds_type_declarations_and_modulus_types_with_exact_spans",
        rules: &["S3I-SYNTAX-01", "S3I-COMPAT-01"],
    },
    TestEvidence {
        source_path: "src/parser.rs",
        test: "rejects_malformed_type_declarations_and_moduli",
        rules: &["S3I-SYNTAX-01"],
    },
    TestEvidence {
        source_path: "src/parser.rs",
        test: "bounds_type_declarations_per_module",
        rules: &["S3I-SYNTAX-01", "S3I-RES-01"],
    },
    TestEvidence {
        source_path: "src/parser.rs",
        test: "a_modulus_counts_toward_nesting_and_the_height_of_its_expression",
        rules: &["S3I-SYNTAX-01", "S3I-RES-01"],
    },
    TestEvidence {
        source_path: "src/parser.rs",
        test: "bounds_expression_nesting_for_every_opener",
        rules: &["S3I-SYNTAX-01", "S3I-COMPAT-01"],
    },
    TestEvidence {
        source_path: "src/semantics/tests.rs",
        test: "residues_build_typed_core_in_postorder",
        rules: &["S3I-LITERAL-01", "S3I-OPERATOR-01", "S3I-CORE-01"],
    },
    TestEvidence {
        source_path: "src/semantics/tests.rs",
        test: "moduli_are_constants_from_two_through_two_to_the_521_minus_one",
        rules: &["S3I-MODULUS-01", "S3I-DETERMINISM-01"],
    },
    TestEvidence {
        source_path: "src/semantics/tests.rs",
        test: "moduli_written_within_a_modulus_are_evaluated_too",
        rules: &["S3I-MODULUS-01"],
    },
    TestEvidence {
        source_path: "src/semantics/tests.rs",
        test: "type_names_resolve_in_declaration_order_within_their_module",
        rules: &["S3I-NAMES-01", "S3I-CORE-01"],
    },
    TestEvidence {
        source_path: "src/semantics/tests.rs",
        test: "residue_literals_lie_strictly_between_minus_the_modulus_and_the_modulus",
        rules: &["S3I-LITERAL-01", "S3I-DETERMINISM-01"],
    },
    TestEvidence {
        source_path: "src/semantics/tests.rs",
        test: "residues_have_ring_operators_and_equality_but_no_order_remainder_or_bits",
        rules: &["S3I-OPERATOR-01", "S3I-DETERMINISM-01"],
    },
    TestEvidence {
        source_path: "src/semantics/tests.rs",
        test: "residues_convert_to_and_from_numbers_and_index_through_least_residues",
        rules: &["S3I-CONVERT-01", "S3I-INDEX-01"],
    },
    TestEvidence {
        source_path: "src/semantics/tests.rs",
        test: "residue_types_cross_modules_by_value_and_type_names_stay_in_their_module",
        rules: &["S3I-NAMES-01", "S3I-CORE-01", "S3I-COMPAT-01"],
    },
    TestEvidence {
        source_path: "src/semantics/tests.rs",
        test: "modulus_events_follow_the_normative_accounting",
        rules: &["S3I-MODULUS-01", "S3I-RES-01"],
    },
    TestEvidence {
        source_path: "src/semantics/tests.rs",
        test: "modulus_storage_failures_return_no_partial_core",
        rules: &["S3I-RES-01"],
    },
    TestEvidence {
        source_path: "src/semantics/tests.rs",
        test: "rejects_foreign_spans_in_type_declarations_and_moduli",
        rules: &["S3I-RES-01"],
    },
    TestEvidence {
        source_path: "src/core.rs",
        test: "moduli_range_from_two_through_two_to_the_521_minus_one",
        rules: &["S3I-MODULUS-01"],
    },
    TestEvidence {
        source_path: "src/core.rs",
        test: "moduli_display_as_their_standards_write_them",
        rules: &["S3I-MODULUS-01"],
    },
    TestEvidence {
        source_path: "src/core.rs",
        test: "residues_reduce_and_invert_exactly",
        rules: &["S3I-DIVISION-01", "S3I-EVAL-01"],
    },
    TestEvidence {
        source_path: "src/core.rs",
        test: "residue_types_and_values_display_their_modulus_and_least_residue",
        rules: &["S3I-CORE-01", "S3I-EVAL-01"],
    },
    TestEvidence {
        source_path: "src/eval.rs",
        test: "residue_arithmetic_matches_a_u128_reference",
        rules: &["S3I-OPERATOR-01", "S3I-DIVISION-01", "S3I-EVAL-01"],
    },
    TestEvidence {
        source_path: "src/eval.rs",
        test: "conversions_reduce_into_residues_and_give_least_residues_out",
        rules: &["S3I-CONVERT-01", "S3I-EVAL-01"],
    },
    TestEvidence {
        source_path: "src/eval.rs",
        test: "residue_steps_follow_the_normative_cost_table",
        rules: &["S3I-EVAL-01"],
    },
    TestEvidence {
        source_path: "src/eval.rs",
        test: "residues_cross_the_call_interface_with_exact_types",
        rules: &["S3I-CORE-01", "S3I-EVAL-01"],
    },
    TestEvidence {
        source_path: "src/eval.rs",
        test: "deepest_accepted_sources_fit_in_one_mebibyte_of_stack",
        rules: &["S3I-RES-01"],
    },
    TestEvidence {
        source_path: "src/eval.rs",
        test: "deeply_nested_rejected_moduli_fit_in_one_mebibyte_of_stack",
        rules: &["S3I-MODULUS-01", "S3I-RES-01"],
    },
    TestEvidence {
        source_path: "src/diagnostic.rs",
        test: "diagnostic_code_inventory_is_exact_ordered_and_unique",
        rules: &["S3I-MODULUS-01", "S3I-NAMES-01"],
    },
];

fn orangec() -> Command {
    Command::new(env!("CARGO_BIN_EXE_orangec"))
}

fn fixture_directory() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/s3i")
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
        .join(format!("orangec-s3i-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&directory);
    fs::create_dir(&directory).unwrap();
    directory
}

/// The rule index rows of the specification: rule ID and evidence label.
fn documented_rules() -> Vec<(&'static str, &'static str)> {
    MODULAR_SPECIFICATION
        .lines()
        .filter_map(|line| {
            let row = line.strip_prefix("| `")?;
            let (rule, _) = row.split_once("` |")?;
            if !rule.starts_with("S3I-") {
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
        _ => panic!("unknown S3i evidence layer {label:?}"),
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
        _ => panic!("unmapped S3i evidence source {source_path}"),
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
        S3I_CONFORMANCE_SOURCE.matches(&declaration).count(),
        1,
        "generated evidence {test} must be declared exactly once at file root"
    );
}

#[test]
fn s3i_rule_index_is_exact_and_covered() {
    let documented = documented_rules();
    let documented_ids: Vec<_> = documented.iter().map(|(rule, _)| *rule).collect();
    assert_eq!(
        documented_ids, RULES,
        "docs/MODULAR_2026.md rule index drifted from the S3i runner"
    );
    let known: BTreeSet<_> = RULES.iter().copied().collect();
    assert_eq!(known.len(), RULES.len(), "duplicate S3i rule ID");

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
fn s3i_cli_conformance_corpus_is_exact_and_repeatable() {
    let directory = fixture_directory();
    let mut observed: Vec<_> = fs::read_dir(&directory)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().into_string().unwrap())
        .collect();
    observed.sort();
    let mut expected: Vec<_> = CASES.iter().map(|case| case.fixture).collect();
    expected.sort_unstable();
    assert_eq!(observed, expected, "unexpected S3i fixture inventory");

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
fn s3i_type_declaration_and_modulus_limits_are_exact() {
    let directory = scratch_directory("limits");

    // A module has at most 64 `type` declarations, and a declared name may
    // be used by every function of its module.
    let declarations = |count: usize| {
        let types: String = (0..count)
            .map(|index| format!("  type T{index} = Mod[{}];\n", index + 2))
            .collect();
        format!(
            "edition 2026;\nmodule limits {{\n{types}  spec v() -> T{} {{ -1 }}\n}}\n",
            count - 1
        )
    };
    let path = directory.join("limits.or");
    fs::write(&path, declarations(64)).unwrap();
    let most = run_twice("eval", &path, "64 type declarations");
    assert_success(&most, "limits::v: Mod[65] = 64\n", "64 type declarations");
    fs::write(&path, declarations(65)).unwrap();
    let over = run_twice("check", &path, "65 type declarations");
    assert_failure(
        &over,
        &["ORC0106"],
        &["limits.or:67:3"],
        &["module has more than 64 `type` declarations"],
        "65 type declarations",
    );

    // The widest modulus is 2^521 - 1, the prime of P-521, and the smallest
    // is 2; 2^521 has 522 bits.
    let path = directory.join("widest.or");
    fs::write(
        &path,
        "edition 2026;\nmodule widest {\n  type P = Mod[(1 << 521) - 1];\n  \
         spec top() -> P { -1 }\n  spec half() -> P { 1 / 2 }\n  \
         spec two() -> Mod[2] { -1 }\n}\n",
    )
    .unwrap();
    let widest = run_twice("eval", &path, "widest modulus");
    assert_success(
        &widest,
        concat!(
            "widest::top: Mod[(1 << 521) - 1] = ",
            "686479766013060971498190079908139321726943530014330540939446345918554318339765",
            "6052122559640661454554977296311391480858037121987999716643812574028291115057150\n",
            "widest::half: Mod[(1 << 521) - 1] = ",
            "343239883006530485749095039954069660863471765007165270469723172959277159169882",
            "8026061279820330727277488648155695740429018560993999858321906287014145557528576\n",
            "widest::two: Mod[2] = 1\n",
        ),
        "widest modulus",
    );
    fs::write(
        &path,
        "edition 2026;\nmodule widest {\n  type P = Mod[1 << 521];\n  \
         type Q = Mod[(1 << 521) - 1 + 1];\n}\n",
    )
    .unwrap();
    let wider = run_twice("check", &path, "wider modulus");
    assert_failure(
        &wider,
        &["ORC0232", "ORC0232"],
        &["widest.or:3:16", "widest.or:4:16"],
        &["this modulus has 522 bits"],
        "wider modulus",
    );
    fs::remove_dir_all(&directory).unwrap();
}

#[test]
fn s3i_literals_and_indices_are_exact_at_their_edges() {
    let directory = scratch_directory("edges");

    // A literal of `Mod[3329]` has a magnitude of at most 3328; `x as Int`
    // for x of `Mod[7]` ranges over 0 through 6, and so does `x as Word[8]`,
    // while a residue of `Mod[300]` as `Word[8]` ranges over the whole word.
    let path = directory.join("edges.or");
    fs::write(
        &path,
        concat!(
            "edition 2026;\nmodule edges {\n",
            "  type Zq = Mod[3329];\n",
            "  spec high() -> Zq { 3328 }\n",
            "  spec low() -> Zq { -3328 }\n",
            "  spec at(x: Mod[7]) -> Int {\n",
            "    let t: Int^7 = [0, 1, 2, 3, 4, 5, 6];\n",
            "    t[x as Int]\n",
            "  }\n",
            "  spec small(x: Mod[7]) -> Int {\n",
            "    let t: Int^7 = [0, 1, 2, 3, 4, 5, 6];\n",
            "    t[x as Word[8]]\n",
            "  }\n",
            "  spec byte(x: Mod[300]) -> Int {\n",
            "    let t: Int^256 = [7; 256];\n",
            "    t[x as Word[8]]\n",
            "  }\n",
            "  spec picked() -> Int^3 { [at(-1), small(6), byte(299)] }\n",
            "}\n",
        ),
    )
    .unwrap();
    let edges = run_twice("eval", &path, "edges");
    assert_success(
        &edges,
        concat!(
            "edges::high: Mod[3329] = 3328\n",
            "edges::low: Mod[3329] = 1\n",
            "edges::picked: Int^3 = [6, 6, 7]\n",
        ),
        "edges",
    );

    // One more in each direction is refused before anything is evaluated.
    let path = directory.join("over.or");
    fs::write(
        &path,
        concat!(
            "edition 2026;\nmodule over {\n",
            "  type Zq = Mod[3329];\n",
            "  spec high() -> Zq { 3329 }\n",
            "  spec low() -> Zq { -3329 }\n",
            "  spec at(x: Mod[7]) -> Int {\n",
            "    let t: Int^6 = [0, 1, 2, 3, 4, 5];\n",
            "    t[x as Int]\n",
            "  }\n",
            "  spec small(x: Mod[7]) -> Int {\n",
            "    let t: Int^6 = [0, 1, 2, 3, 4, 5];\n",
            "    t[x as Word[8]]\n",
            "  }\n",
            "  spec byte(x: Mod[257]) -> Int {\n",
            "    let t: Int^255 = [7; 255];\n",
            "    t[x as Word[8]]\n",
            "  }\n",
            "}\n",
        ),
    )
    .unwrap();
    let over = run_twice("eval", &path, "over the edges");
    assert_failure(
        &over,
        &["ORC0207", "ORC0207", "ORC0223", "ORC0223", "ORC0223"],
        &[
            "over.or:4:23",
            "over.or:5:23",
            "over.or:8:7",
            "over.or:12:7",
            "over.or:16:7",
        ],
        &[
            "literal is outside the range of `Mod[3329]`",
            "this index runs from 0 through 6, out of range for `Int^6`",
            "this index runs from 0 through 255, out of range for `Int^255`",
        ],
        "over the edges",
    );
    fs::remove_dir_all(&directory).unwrap();
}
