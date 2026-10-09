//! External conformance evidence for the proposed Orange 2026 S3s slice.
//!
//! The exact fixture inventory and generated programs run through the real
//! `orangec` binary twice. The rule index in `docs/NESTED_ARRAYS_2026.md`
//! must agree with this evidence map. Earlier slice runners observe the
//! compatibility behavior of the same binary.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const SPECIFICATION: &str = include_str!("../../../../docs/NESTED_ARRAYS_2026.md");
const CONFORMANCE_SOURCE: &str = include_str!("s3s_conformance.rs");
const CLI: u8 = 1;
const GENERATED_CLI: u8 = 2;

const RULES: [&str; 12] = [
    "S3S-01", "S3S-02", "S3S-03", "S3S-04", "S3S-05", "S3S-06", "S3S-07", "S3S-08", "S3S-09",
    "S3S-10", "S3S-11", "S3S-12",
];

#[derive(Clone, Copy)]
enum Expectation {
    Exact {
        stdout: fn() -> String,
    },
    Failure {
        codes: &'static [&'static str],
        locations: &'static [&'static str],
        messages: &'static [&'static str],
    },
}

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

fn nothing() -> String {
    String::new()
}

fn test_report(titles: &[&str]) -> String {
    let mut report = String::new();
    for title in titles {
        report.push_str(&format!("test \"{title}\" ... ok\n"));
    }
    let count = titles.len();
    report.push_str(&format!("{count} tests: {count} passed, 0 failed\n"));
    report
}

fn matrices_tests() -> String {
    test_report(&[
        "nested literals preserve row order",
        "a fill evaluates one row then repeats it",
        "selection checks each axis",
        "a row update preserves the other row",
        "inner slices and joins preserve scalar order",
        "an outer slice is still an array of rows",
        "outer concatenation reconstructs the matrix",
        "slice replacement updates complete rows",
        "tuples retain complete matrix values",
    ])
}

fn matrices_value() -> String {
    String::from(
        "matrices::literal: (Word[16]^3)^2 = [[0x0001, 0x0002, 0x0003], \
         [0x0004, 0x0005, 0x0006]]\n",
    )
}

fn parameters_tests() -> String {
    test_report(&[
        "an explicit row type yields a matrix",
        "a matrix argument selects its finite type instance",
        "size specialization checks nested loop indices",
        "another scalar domain retains its row type",
        "typed row literals select a matrix instance",
        "typed row fills select a matrix instance",
        "a typed scalar leaf retains both axes during inference",
        "an explicit array type selects a matrix instance",
    ])
}

fn parameters_values() -> String {
    String::from(concat!(
        "parameters::rows: (Word[8]^3)^2 = [[0x01, 0x02, 0x03], [0x01, 0x02, 0x03]]\n",
        "parameters::inferred: (Word[8]^3)^2 = [[0x01, 0x02, 0x03], [0x01, 0x02, 0x03]]\n",
        "parameters::selected: Int^3 = [4, 5, 6]\n",
        "parameters::sum: Int = 12\n",
    ))
}

fn quadratic_tests() -> String {
    test_report(&[
        "two quadratic-factor products, hand-derived",
        "quadratic-pair multiplication is commutative in this example",
        "the multiplicative identity has one in every constant coefficient",
    ])
}

fn quadratic_value() -> String {
    String::from("quadratic::products: (Mod[3329]^2)^2 = [[139, 10], [15, 3]]\n")
}

fn domains_tests() -> String {
    test_report(&[
        "integer leaves retain signs",
        "truth leaves compare recursively",
        "residue leaves retain least-residue meaning",
        "a tuple can contain matrices from different domains",
    ])
}

fn domains_value() -> String {
    String::from(
        "domains::packaged: ((Int^2)^2, (Bool^2)^2, (Mod[7]^2)^2) = \
         ([[-1, 2], [3, -4]], [[true, false], [false, true]], [[3, 4], [0, 1]])\n",
    )
}

const CLEAN_CHECK: Run = Run {
    arguments: &["check"],
    expectation: Expectation::Exact { stdout: nothing },
};

const RAGGED: Expectation = Expectation::Failure {
    codes: &["ORC0222", "ORC0222"],
    locations: &["invalid-ragged.or:5:44", "invalid-ragged.or:6:34"],
    messages: &[
        "this array has 2 elements, but `Word[8]^3` has 3",
        "this array has 1 element, but `(Word[8]^3)^2` has 2",
    ],
};

const DIMENSIONS: Expectation = Expectation::Failure {
    codes: &["ORC0214", "ORC0214", "ORC0214"],
    locations: &[
        "invalid-dimensions.or:7:41",
        "invalid-dimensions.or:8:49",
        "invalid-dimensions.or:9:71",
    ],
    messages: &[
        "`x` has type `(Word[8]^3)^3`, but `(Word[8]^3)^2` is required here",
        "`x` has type `(Word[8]^4)^2`, but `(Word[8]^3)^2` is required here",
        "`row` has type `Word[8]^4`, but `Word[8]^3` is required here",
    ],
};

const DOMAINS: Expectation = Expectation::Failure {
    codes: &["ORC0214", "ORC0214", "ORC0215"],
    locations: &[
        "invalid-domains.or:7:49",
        "invalid-domains.or:8:69",
        "invalid-domains.or:9:50",
    ],
    messages: &[
        "`x` has type `(Mod[5]^2)^2`, but `(Mod[7]^2)^2` is required here",
        "`row` has type `Mod[5]^2`, but `Mod[7]^2` is required here",
        "`<` is not defined for `(Mod[7]^2)^2`",
    ],
};

const INDICES: Expectation = Expectation::Failure {
    codes: &["ORC0223", "ORC0223", "ORC0223", "ORC0223", "ORC0226"],
    locations: &[
        "invalid-indices.or:5:36",
        "invalid-indices.or:6:43",
        "invalid-indices.or:7:57",
        "invalid-indices.or:8:64",
        "invalid-indices.or:9:50",
    ],
    messages: &[
        "index `2` is out of range for `(Word[8]^3)^2`",
        "index `3` is out of range for `Word[8]^3`",
        "this index runs from 0 through 255, out of range for `(Word[8]^3)^2`",
        "this index runs from 0 through 255, out of range for `Word[8]^3`",
    ],
};

const TYPES: Expectation = Expectation::Failure {
    codes: &["ORC0203", "ORC0221", "ORC0203"],
    locations: &[
        "invalid-types.or:5:56",
        "invalid-types.or:6:19",
        "invalid-types.or:8:16",
    ],
    messages: &[
        "`Hyper` already has 4 array dimensions",
        "arrays have at most 4 dimensions",
        "this length would add a fifth dimension",
        "an array shape has 65792 scalar elements, exceeding 65536",
        "every axis is positive and the product of the axes is at most 65536",
        "`Pair` is a tuple type, so this is an array of tuples",
    ],
};

const CONVERSIONS: Expectation = Expectation::Failure {
    codes: &["ORC0215", "ORC0215", "ORC0215", "ORC0215"],
    locations: &[
        "invalid-conversions.or:5:44",
        "invalid-conversions.or:6:40",
        "invalid-conversions.or:7:52",
        "invalid-conversions.or:8:37",
    ],
    messages: &[
        "`as little` does not convert `(Word[8]^2)^2`",
        "`as big` does not convert `(Word[8]^2)^2`",
        "`as little` does not convert to `(Word[8]^2)^2`",
        "`as` is not defined for `(Word[8]^2)^2`",
    ],
};

const fn rejected(expectation: Expectation) -> [Run; 3] {
    [
        Run {
            arguments: &["check"],
            expectation,
        },
        Run {
            arguments: &["eval"],
            expectation,
        },
        Run {
            arguments: &["test"],
            expectation,
        },
    ]
}

const CASES: [Case; 10] = [
    Case {
        fixture: "valid-matrices.or",
        runs: &[
            CLEAN_CHECK,
            Run {
                arguments: &["test"],
                expectation: Expectation::Exact {
                    stdout: matrices_tests,
                },
            },
            Run {
                arguments: &["eval", "--spec", "literal"],
                expectation: Expectation::Exact {
                    stdout: matrices_value,
                },
            },
        ],
        rules: &[
            "S3S-02", "S3S-03", "S3S-04", "S3S-05", "S3S-06", "S3S-07", "S3S-08", "S3S-10",
            "S3S-12",
        ],
    },
    Case {
        fixture: "valid-parameters.or",
        runs: &[
            CLEAN_CHECK,
            Run {
                arguments: &["test"],
                expectation: Expectation::Exact {
                    stdout: parameters_tests,
                },
            },
            Run {
                arguments: &[
                    "eval", "--spec", "rows", "--spec", "inferred", "--spec", "selected", "--spec",
                    "sum",
                ],
                expectation: Expectation::Exact {
                    stdout: parameters_values,
                },
            },
        ],
        rules: &["S3S-03", "S3S-04", "S3S-09", "S3S-12"],
    },
    Case {
        fixture: "valid-quadratic-pairs.or",
        runs: &[
            CLEAN_CHECK,
            Run {
                arguments: &["test"],
                expectation: Expectation::Exact {
                    stdout: quadratic_tests,
                },
            },
            Run {
                arguments: &["eval"],
                expectation: Expectation::Exact {
                    stdout: quadratic_value,
                },
            },
        ],
        rules: &["S3S-02", "S3S-03", "S3S-04", "S3S-05", "S3S-10", "S3S-12"],
    },
    Case {
        fixture: "valid-domains.or",
        runs: &[
            CLEAN_CHECK,
            Run {
                arguments: &["test"],
                expectation: Expectation::Exact {
                    stdout: domains_tests,
                },
            },
            Run {
                arguments: &["eval", "--spec", "packaged"],
                expectation: Expectation::Exact {
                    stdout: domains_value,
                },
            },
        ],
        rules: &["S3S-02", "S3S-03", "S3S-08", "S3S-10", "S3S-12"],
    },
    Case {
        fixture: "invalid-ragged.or",
        runs: &rejected(RAGGED),
        rules: &["S3S-02", "S3S-03", "S3S-12"],
    },
    Case {
        fixture: "invalid-dimensions.or",
        runs: &rejected(DIMENSIONS),
        rules: &["S3S-02", "S3S-05", "S3S-12"],
    },
    Case {
        fixture: "invalid-domains.or",
        runs: &rejected(DOMAINS),
        rules: &["S3S-02", "S3S-05", "S3S-10", "S3S-12"],
    },
    Case {
        fixture: "invalid-indices.or",
        runs: &rejected(INDICES),
        rules: &["S3S-04", "S3S-12"],
    },
    Case {
        fixture: "invalid-types.or",
        runs: &rejected(TYPES),
        rules: &["S3S-01", "S3S-02", "S3S-08", "S3S-12"],
    },
    Case {
        fixture: "invalid-conversions.or",
        runs: &rejected(CONVERSIONS),
        rules: &["S3S-11", "S3S-12"],
    },
];

const GENERATED_EVIDENCE: &[(&str, &[&str])] = &[
    (
        "s3s_axis_and_scalar_limits_are_checked_before_evaluation",
        &["S3S-01", "S3S-02", "S3S-09", "S3S-12"],
    ),
    (
        "s3s_each_axis_checks_static_and_computed_indices",
        &["S3S-04", "S3S-12"],
    ),
    (
        "s3s_matrix_equality_cost_is_independent_of_difference_position",
        &["S3S-10", "S3S-12"],
    ),
    (
        "s3s_nested_copy_operations_charge_by_outer_slots",
        &["S3S-03", "S3S-05", "S3S-06", "S3S-07", "S3S-10", "S3S-12"],
    ),
    (
        "s3s_nested_literals_keep_rank_and_domains",
        &["S3S-02", "S3S-03", "S3S-09", "S3S-11", "S3S-12"],
    ),
];

fn run_twice(arguments: &[&str], path: &Path, context: &str) -> Output {
    let run = || {
        Command::new(env!("CARGO_BIN_EXE_orangec"))
            .args(arguments)
            .arg(path)
            .output()
            .unwrap()
    };
    let first = run();
    let second = run();
    assert_eq!(
        first.status.code(),
        second.status.code(),
        "{context} changed exit status"
    );
    assert_eq!(first.stdout, second.stdout, "{context} changed stdout");
    assert_eq!(first.stderr, second.stderr, "{context} changed stderr");
    first
}

fn assert_exact(output: &Output, status: i32, stdout: &str, stderr: &str, context: &str) {
    assert_eq!(
        output.status.code(),
        Some(status),
        "{context}:\n{}",
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

fn diagnostic_codes(stderr: &str) -> Vec<&str> {
    stderr
        .lines()
        .filter_map(|line| {
            line.strip_prefix("error[")?
                .split_once(']')
                .map(|(code, _)| code)
        })
        .collect()
}

fn assert_failure(
    output: &Output,
    codes: &[&str],
    locations: &[&str],
    messages: &[&str],
    context: &str,
) {
    assert_eq!(output.status.code(), Some(1), "{context} status");
    assert!(output.stdout.is_empty(), "{context} emitted partial output");
    let stderr = std::str::from_utf8(&output.stderr).unwrap();
    assert_eq!(
        diagnostic_codes(stderr),
        codes,
        "{context} codes:\n{stderr}"
    );
    let observed: Vec<_> = stderr
        .lines()
        .filter_map(|line| line.strip_prefix(" --> "))
        .map(|location| {
            Path::new(location)
                .file_name()
                .unwrap()
                .to_string_lossy()
                .into_owned()
        })
        .collect();
    assert_eq!(observed, locations, "{context} locations:\n{stderr}");
    for message in messages {
        assert!(
            stderr.contains(message),
            "{context} missing {message:?}:\n{stderr}"
        );
    }
}

fn assert_rejected(output: &Output, code: &str, message: &str, context: &str) {
    assert_eq!(output.status.code(), Some(1), "{context} status");
    assert!(output.stdout.is_empty(), "{context} emitted partial output");
    let stderr = std::str::from_utf8(&output.stderr).unwrap();
    assert_eq!(
        diagnostic_codes(stderr),
        [code],
        "{context} codes:\n{stderr}"
    );
    assert!(
        stderr.contains(message),
        "{context} missing {message:?}:\n{stderr}"
    );
}

fn scratch_directory(name: &str) -> PathBuf {
    let directory = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("orangec-s3s-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&directory);
    fs::create_dir(&directory).unwrap();
    directory
}

#[test]
fn s3s_rule_index_is_exact_and_covered() {
    let documented: Vec<_> = SPECIFICATION
        .lines()
        .filter_map(|line| {
            let (rule, _) = line.strip_prefix("| `")?.split_once("` |")?;
            if !rule.starts_with("S3S-") {
                return None;
            }
            let label = line.strip_suffix('|')?.rsplit('|').next()?.trim();
            Some((rule, label))
        })
        .collect();
    assert_eq!(
        documented.iter().map(|(rule, _)| *rule).collect::<Vec<_>>(),
        RULES
    );
    let known: BTreeSet<_> = RULES.into_iter().collect();
    assert_eq!(known.len(), RULES.len());
    let mut observed = BTreeMap::<&str, u8>::new();
    let mut record = |rule: &'static str, layer: u8| {
        assert!(known.contains(rule), "unknown rule {rule}");
        *observed.entry(rule).or_default() |= layer;
    };
    let mut fixtures = BTreeSet::new();
    for case in CASES {
        assert!(
            fixtures.insert(case.fixture),
            "duplicate fixture {}",
            case.fixture
        );
        assert!(!case.runs.is_empty() && !case.rules.is_empty());
        for rule in case.rules {
            record(rule, CLI);
        }
    }
    let mut generated = BTreeSet::new();
    for (test, rules) in GENERATED_EVIDENCE {
        assert!(generated.insert(*test), "duplicate generated test {test}");
        assert_eq!(
            CONFORMANCE_SOURCE
                .matches(&format!("\n#[test]\nfn {test}() {{\n"))
                .count(),
            1
        );
        for rule in *rules {
            record(rule, GENERATED_CLI);
        }
    }
    for (rule, label) in documented {
        let required = match label {
            "CLI" => CLI,
            "Generated CLI" => GENERATED_CLI,
            "CLI and generated CLI" => CLI | GENERATED_CLI,
            _ => panic!("unknown S3s evidence label {label:?}"),
        };
        assert_eq!(
            observed.get(rule).copied().unwrap_or_default() & required,
            required,
            "{rule} requires {label}"
        );
    }
}

#[test]
fn s3s_cli_conformance_corpus_is_exact_and_repeatable() {
    let directory = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/s3s");
    let mut observed: Vec<_> = fs::read_dir(&directory)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().into_string().unwrap())
        .collect();
    observed.sort();
    let mut expected: Vec<_> = CASES.iter().map(|case| case.fixture).collect();
    expected.sort_unstable();
    assert_eq!(observed, expected, "unexpected S3s fixture inventory");
    for case in CASES {
        let path = directory.join(case.fixture);
        let mut diagnostics = None;
        for run in case.runs {
            let context = format!("{} {}", case.fixture, run.arguments.join(" "));
            let output = run_twice(run.arguments, &path, &context);
            match run.expectation {
                Expectation::Exact { stdout } => assert_exact(&output, 0, &stdout(), "", &context),
                Expectation::Failure {
                    codes,
                    locations,
                    messages,
                } => {
                    assert_failure(&output, codes, locations, messages, &context);
                    let first = diagnostics.get_or_insert_with(|| output.stderr.clone());
                    assert_eq!(
                        *first, output.stderr,
                        "{context} command changed diagnostics"
                    );
                }
            }
        }
    }
}

#[test]
fn s3s_axis_and_scalar_limits_are_checked_before_evaluation() {
    let directory = scratch_directory("limits");
    let path = directory.join("limits.or");
    for (rows, columns) in [
        (1, 1),
        (1, 65536),
        (65536, 1),
        (256, 256),
        (128, 512),
        (257, 255),
    ] {
        fs::write(&path, format!("edition 2026;\nmodule limits {{\n  type Row = Word[8]^{columns};\n  type Grid = Row^{rows};\n  spec corner() -> Word[8] {{ let m: Grid = [[7; {columns}]; {rows}]; m[{}][{}] }}\n  spec all() -> Grid {{ [[7; {columns}]; {rows}] }}\n}}\n", rows - 1, columns - 1)).unwrap();
        let context = format!("{rows} rows of {columns} columns");
        let output = run_twice(&["check"], &path, &context);
        assert_exact(&output, 0, "", "", &context);
        let output = run_twice(&["eval", "--spec", "corner"], &path, &context);
        assert_exact(&output, 0, "limits::corner: Word[8] = 0x07\n", "", &context);
        // Export every leaf at the scalar limit, including extreme axes.
        if rows * columns == 65536 {
            let row = format!(
                "[{}]",
                vec!["0x07"; usize::try_from(columns).unwrap()].join(", ")
            );
            let matrix = format!("[{}]", vec![row; usize::try_from(rows).unwrap()].join(", "));
            let output = run_twice(&["eval", "--spec", "all"], &path, &context);
            assert_exact(
                &output,
                0,
                &format!("limits::all: (Word[8]^{columns})^{rows} = {matrix}\n"),
                "",
                &context,
            );
        }
    }
    for (source, code, message) in [
        (
            "type Row = Word[8]^0;",
            "ORC0221",
            "an array length must be a decimal integer from 1 through 65536",
        ),
        (
            "type Row = Word[8]^65537;",
            "ORC0221",
            "an array length must be a decimal integer from 1 through 65536",
        ),
        (
            "type Row = Word[8]^1; type Grid = Row^0;",
            "ORC0221",
            "an array length must be a decimal integer from 1 through 65536",
        ),
        (
            "type Row = Word[8]^1; type Grid = Row^65537;",
            "ORC0221",
            "an array length must be a decimal integer from 1 through 65536",
        ),
        (
            "type Row = Word[8]^256; type Grid = Row^257;",
            "ORC0221",
            "an array shape has 65792 scalar elements, exceeding 65536",
        ),
        (
            "type Row = Word[8]^1; type Grid = Row^1; type Cube = Grid^1; type Hyper = Cube^1; \
             type Five = Hyper^1;",
            "ORC0203",
            "already has 4 array dimensions",
        ),
        (
            "type Row = Word[8]^256; spec f[n in 256..258](r: Row) -> Row^n { [r; n] }",
            "ORC0221",
            "an array shape has 65792 scalar elements, exceeding 65536",
        ),
    ] {
        fs::write(
            &path,
            format!("edition 2026;\nmodule limits {{ {source} }}\n"),
        )
        .unwrap();
        let output = run_twice(&["check"], &path, source);
        assert_rejected(&output, code, message, source);
    }
    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn s3s_each_axis_checks_static_and_computed_indices() {
    let directory = scratch_directory("indices");
    let path = directory.join("indices.or");
    fs::write(&path, "edition 2026;\nmodule indices {\n  type Row = Word[8]^256;\n  type Grid = Row^256;\n  spec pick(m: Grid, r: Word[8], c: Word[8]) -> Word[8] { m[r][c] }\n  spec corners() -> Word[8]^4 { let m: Grid = [[7; 256]; 256]; [pick(m, 0, 0), pick(m, 0, 255), pick(m, 255, 0), pick(m, 255, 255)] }\n}\n").unwrap();
    let output = run_twice(&["eval"], &path, "computed byte indices on both full axes");
    assert_exact(
        &output,
        0,
        "indices::corners: Word[8]^4 = [0x07, 0x07, 0x07, 0x07]\n",
        "",
        "computed byte indices on both full axes",
    );
    for (expression, code, message) in [
        ("m[256]", "ORC0223", "index `256` is out of range"),
        ("m[0][256]", "ORC0223", "index `256` is out of range"),
        ("m[0][0][0]", "ORC0224", "only an array can be indexed"),
        (
            "m[0][0..1][0]",
            "ORC0101",
            "bind it with `let` to select from it",
        ),
    ] {
        let result = if expression == "m[256]" {
            "Row"
        } else {
            "Word[8]"
        };
        fs::write(&path, format!("edition 2026;\nmodule indices {{ type Row = Word[8]^256; type Grid = Row^256; spec bad(m: Grid) -> {result} {{ {expression} }} }}\n")).unwrap();
        let output = run_twice(&["check"], &path, expression);
        assert_rejected(&output, code, message, expression);
    }
    // Suffixes are parsed iteratively but retain the expression-height budget.
    let chain = "[0]".repeat(257);
    fs::write(&path, format!("edition 2026;\nmodule indices {{ type Row = Word[8]^1; type Grid = Row^1; spec bad(m: Grid) -> Word[8] {{ m{chain} }} }}\n")).unwrap();
    let output = run_twice(&["check"], &path, "an excessive suffix chain");
    assert_rejected(
        &output,
        "ORC0106",
        "expression",
        "an excessive suffix chain",
    );
    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn s3s_matrix_equality_cost_is_independent_of_difference_position() {
    let directory = scratch_directory("equality");
    let path = directory.join("equality.or");
    for (row, column) in [(0, 0), (0, 255), (127, 128), (255, 0), (255, 255)] {
        for (value, equal) in [(0, true), (1, false)] {
            fs::write(&path, format!("edition 2026;\nmodule equality {{\n  type Row = Word[8]^256;\n  type Grid = Row^256;\n  spec compared() -> Bool {{ let x: Grid = [[0; 256]; 256]; x == (x with [{row}] = (x[{row}] with [{column}] = {value})) }}\n}}\n")).unwrap();
            let context = format!("[{row}][{column}] = {value}");
            let output = run_twice(&["eval", "--stats"], &path, &context);
            assert_exact(
                &output,
                0,
                &format!("equality::compared: Bool = {equal}\n"),
                "equality::compared: 1048 steps\ntotal: 1048 of 1048576 steps\n",
                &context,
            );
        }
    }
    // Exactly the declared work succeeds; one fewer step stops atomically.
    let output = run_twice(
        &["eval", "--stats", "--steps", "1048"],
        &path,
        "exact comparison budget",
    );
    assert_exact(
        &output,
        0,
        "equality::compared: Bool = false\n",
        "equality::compared: 1048 steps\ntotal: 1048 of 1048 steps\n",
        "exact comparison budget",
    );
    let output = run_twice(
        &["eval", "--stats", "--steps", "1047"],
        &path,
        "one step short",
    );
    assert_rejected(
        &output,
        "ORC0301",
        "reference evaluation step limit exceeded",
        "one step short",
    );
    // Every scalar domain retains its comparison cost at each leaf. A
    // two-digit Int costs three steps per comparison rather than two.
    for (scalar, initial, replacement, expected_cost) in [
        ("Word[16]", "1", "2", 20),
        ("Bool", "false", "true", 20),
        ("Int", "1", "2", 404),
        ("Int", "4294967296", "4294967297", 599),
        ("Mod[7]", "1", "2", 404),
    ] {
        for (row, column) in [(0, 0), (2, 64)] {
            fs::write(&path, format!("edition 2026;\nmodule equality {{ type Row = {scalar}^65; type Grid = Row^3; spec compared() -> Bool {{ let x: Grid = [[{initial}; 65]; 3]; x == (x with [{row}] = (x[{row}] with [{column}] = {replacement})) }} }}\n")).unwrap();
            let context = format!("{scalar}, [{row}][{column}]");
            let output = run_twice(&["eval", "--stats"], &path, &context);
            assert_exact(
                &output,
                0,
                "equality::compared: Bool = false\n",
                &format!(
                    "equality::compared: {expected_cost} steps\ntotal: {expected_cost} of 1048576 steps\n"
                ),
                &context,
            );
        }
    }
    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn s3s_nested_copy_operations_charge_by_outer_slots() {
    let directory = scratch_directory("costs");
    let path = directory.join("costs.or");
    for columns in [1_u32, 64, 65, 512] {
        let row_cost = columns.div_ceil(64);
        // The outer fill and each outer copy write 65 row slots, costing 2.
        // The row is evaluated once; its immutable value may then be shared.
        for (body, rows, cost) in [
            (format!("[[7; {columns}]; 65]"), 65, row_cost + 3),
            (
                format!("[[7; {columns}]; 65] with [64] = [8; {columns}]"),
                65,
                2 * row_cost + 7,
            ),
            (
                format!("[[7; {columns}]; 32] ++ [[8; {columns}]; 33]"),
                65,
                2 * row_cost + 6,
            ),
            (
                format!("let m: Row^65 = [[7; {columns}]; 65]; m[0..64]"),
                64,
                row_cost + 7,
            ),
            (
                format!("[[7; {columns}]; 65] with [0..1] = [[8; {columns}]; 1]"),
                65,
                2 * row_cost + 9,
            ),
        ] {
            fs::write(&path, format!("edition 2026;\nmodule costs {{ type Row = Word[8]^{columns}; spec matrix() -> Row^{rows} {{ {body} }} }}\n")).unwrap();
            let context = format!("{columns} columns, {body}");
            let output = run_twice(&["eval", "--stats"], &path, &context);
            assert_eq!(
                output.status.code(),
                Some(0),
                "{context}: {}",
                String::from_utf8_lossy(&output.stderr)
            );
            assert_eq!(
                String::from_utf8_lossy(&output.stderr),
                format!("costs::matrix: {cost} steps\ntotal: {cost} of 1048576 steps\n"),
                "{context}"
            );
            let stdout = String::from_utf8(output.stdout).unwrap();
            assert!(
                stdout.starts_with(&format!("costs::matrix: (Word[8]^{columns})^{rows} = [[")),
                "{context}"
            );
            assert!(stdout.ends_with("]]\n"), "{context}");
        }
    }
    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn s3s_nested_literals_keep_rank_and_domains() {
    let directory = scratch_directory("domains");
    let path = directory.join("domains.or");
    for (scalar, row, literal, expected) in [
        (
            "Word[8]",
            "Word[8]^2",
            "[[1, 2], [3, 4]]",
            "[[1, 2], [3, 4]]",
        ),
        ("Int", "Int^2", "[[-1, 2], [3, -4]]", "[[-1, 2], [3, -4]]"),
        (
            "Bool",
            "Bool^2",
            "[[true, false], [false, true]]",
            "[[true, false], [false, true]]",
        ),
        (
            "Mod[7]",
            "Mod[7]^2",
            "[[3, -3], [0, 1]]",
            "[[3, 4], [0, 1]]",
        ),
    ] {
        fs::write(&path, format!("edition 2026;\nmodule domains {{ type Row = {row}; type Grid = Row^2; spec id[T in {{Grid}}](m: T) -> T {{ m }} spec answer() -> Bool {{ let m: Grid = {literal}; id(m) == {expected} }} }}\n")).unwrap();
        let output = run_twice(&["eval"], &path, scalar);
        assert_exact(&output, 0, "domains::answer: Bool = true\n", "", scalar);
    }
    // Byte order conversions still work after explicitly selecting a row.
    fs::write(&path, "edition 2026;\nmodule domains { type Row = Word[8]^2; type Grid = Row^2; spec row() -> Word[16] { let m: Grid = [[1, 2], [3, 4]]; m[1] as little Word[16] } }\n").unwrap();
    let output = run_twice(&["eval"], &path, "an explicitly selected row converts");
    assert_exact(
        &output,
        0,
        "domains::row: Word[16] = 0x0403\n",
        "",
        "an explicitly selected row converts",
    );
    for (body, code, message) in [
        (
            "spec join(a: Row^1, b: Wide^1) -> Grid { a ++ b }",
            "ORC0214",
            "`b` has type `(Word[8]^4)^1`, but `(Word[8]^3)^1` is required here",
        ),
        (
            "spec replace(m: Grid) -> Grid { m with [0..1] = [[1, 2, 3]; 2] }",
            "ORC0222",
            "this array has 2 elements, but `(Word[8]^3)^1` has 1",
        ),
        (
            "spec fill() -> Grid { [1; 2] }",
            "ORC0214",
            "an integer literal cannot have type `Word[8]^3`",
        ),
        (
            "spec slice(m: Grid) -> Row^1 { m[1..3] }",
            "ORC0223",
            "this slice reaches elements 1 through 2, out of range for `(Word[8]^3)^2`",
        ),
    ] {
        fs::write(&path, format!("edition 2026;\nmodule domains {{ type Row = Word[8]^3; type Wide = Word[8]^4; type Grid = Row^2; {body} }}\n")).unwrap();
        let output = run_twice(&["check"], &path, body);
        assert_rejected(&output, code, message, body);
    }
    // Explicit matrix arguments use the row alias; repeated powers are
    // not an alternative type grammar inside a call's brackets.
    fs::write(&path, "edition 2026;\nmodule domains { type Row = Word[8]^2; type Grid = Row^2; spec id[T in {Grid}](m: T) -> T { m } spec answer() -> Grid { id[Word[8]^2^2]([[1, 2], [3, 4]]) } }\n").unwrap();
    let output = run_twice(&["check"], &path, "a repeated power is not a type argument");
    assert_rejected(
        &output,
        "ORC0241",
        "not a type",
        "a repeated power is not a type argument",
    );
    fs::remove_dir_all(directory).unwrap();
}
