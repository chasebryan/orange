//! External conformance evidence for the proposed Orange 2026 S3u slice.
//!
//! The exact fixture inventory and generated programs run through the real
//! `orangec` binary twice. The rule index in `docs/DIMENSIONS_2026.md`
//! must agree with this evidence map. Earlier slice runners observe the
//! compatibility behavior of the same binary.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const SPECIFICATION: &str = include_str!("../../../../docs/DIMENSIONS_2026.md");
const CONFORMANCE_SOURCE: &str = include_str!("s3u_conformance.rs");
const CLI: u8 = 1;
const GENERATED_CLI: u8 = 2;

const RULES: [&str; 12] = [
    "S3U-01", "S3U-02", "S3U-03", "S3U-04", "S3U-05", "S3U-06", "S3U-07", "S3U-08", "S3U-09",
    "S3U-10", "S3U-11", "S3U-12",
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

fn shapes_tests() -> String {
    test_report(&[
        "a rank-3 literal keeps every axis",
        "a rank-4 value selects a cube, a plane, a row, and a scalar",
        "a fill repeats a plane",
        "a path of three indices replaces one scalar",
        "a path means the nested updates it abbreviates",
        "a path of four indices reaches a scalar of a rank-4 array",
        "a shorter path replaces a whole row or plane",
        "a two-index path updates a matrix element",
        "loops fill a cube through paths",
        "slices and joins work at every level",
        "slice updates replace whole planes",
        "residue leaves keep least-residue meaning in a rank-3 array",
        "a type parameter admits rank-3 and rank-4 aliases",
        "a size parameter stacks planes into a cube",
        "a tuple carries a rank-4 array and a rank-3 array",
    ])
}

fn shapes_values() -> String {
    String::from(concat!(
        "shapes::cube: ((Word[8]^2)^3)^2 = [[[0x01, 0x02], [0x03, 0x04], [0x05, 0x06]], ",
        "[[0x07, 0x08], [0x09, 0x0a], [0x0b, 0x0c]]]\n",
        "shapes::hyper: (((Word[8]^2)^3)^2)^2 = [[[[0x01, 0x02], [0x03, 0x04], [0x05, 0x06]], ",
        "[[0x07, 0x08], [0x09, 0x0a], [0x0b, 0x0c]]], [[[0x00, 0x00], [0x00, 0x00], ",
        "[0x00, 0x00]], [[0x00, 0x00], [0x00, 0x00], [0x00, 0x00]]]]\n",
        "shapes::residues: ((Mod[7]^4)^2)^2 = [[[1, 2, 3, 4], [5, 6, 0, 1]], ",
        "[[6, 6, 6, 6], [0, 0, 0, 0]]]\n",
    ))
}

fn aes_tests() -> String {
    test_report(&[
        "Section 5.1.1: S(0x53) = 0xed",
        "Table 5: the round constants",
        "Appendix A.1: the first and last words of the expanded key",
        "Appendix B: the cipher example",
        "Appendix C.1: AES-128",
        "Appendix C.1: the inverse cipher returns the plaintext",
    ])
}

fn keccak_tests() -> String {
    test_report(&[
        "FIPS 202 example: SHA3-256 of abc",
        "FIPS 202 example: SHA3-256 of the 448-bit message",
        "Keccak-f[1600] of the zero state",
        "the round constants of iota",
    ])
}

fn mlkem_tests() -> String {
    test_report(&[
        "Appendix A: the first zetas of NTT and of MultiplyNTTs",
        "NTT^-1 undoes NTT",
        "NTT reduces modulo each quadratic factor",
        "row 1 of A o s is a row times a column in Z_q[X]/(X^256 + 1)",
    ])
}

const CLEAN_CHECK: Run = Run {
    arguments: &["check"],
    expectation: Expectation::Exact { stdout: nothing },
};

const SHAPES: Expectation = Expectation::Failure {
    codes: &["ORC0203", "ORC0221", "ORC0221", "ORC0221", "ORC0203"],
    locations: &[
        "invalid-shapes.or:7:15",
        "invalid-shapes.or:8:15",
        "invalid-shapes.or:9:15",
        "invalid-shapes.or:10:39",
        "invalid-shapes.or:11:37",
    ],
    messages: &[
        "`Hyper` already has 4 array dimensions",
        "arrays have at most 4 dimensions",
        "this length would add a fifth dimension",
        "an array shape has 65792 scalar elements, exceeding 65536",
        "an array shape has 69632 scalar elements, exceeding 65536",
        "every axis is positive and the product of the axes is at most 65536",
        "in the instance `stack[17]`",
        "in the instance `more[1]`",
    ],
};

const PATHS: Expectation = Expectation::Failure {
    codes: &[
        "ORC0224", "ORC0223", "ORC0223", "ORC0214", "ORC0222", "ORC0214", "ORC0214",
    ],
    locations: &[
        "invalid-paths.or:8:49",
        "invalid-paths.or:9:42",
        "invalid-paths.or:10:56",
        "invalid-paths.or:11:49",
        "invalid-paths.or:12:49",
        "invalid-paths.or:13:51",
        "invalid-paths.or:14:35",
    ],
    messages: &[
        "only an array can be indexed, but this selects within `Word[8]`",
        "this array has fewer dimensions",
        "an update names one index per dimension it reaches; these 4 indices reach past the \
         array's scalars",
        "index 16 is out of range for `(Word[8]^16)^16`",
        "this index runs from 0 through 255, out of range for `(Word[8]^16)^16`",
        "an integer literal cannot have type `Word[8]^16`",
        "this array has 2 elements, but `Word[8]^16` has 16",
        "`true` has type `Bool`, but `Word[8]` is required here",
        "`c` has type `((Word[8]^16)^16)^16`, but `(Word[8]^16)^16` is required here",
    ],
};

const PATH_SYNTAX: Expectation = Expectation::Failure {
    codes: &["ORC0101", "ORC0101", "ORC0101"],
    locations: &[
        "invalid-path-syntax.or:6:53",
        "invalid-path-syntax.or:7:43",
        "invalid-path-syntax.or:8:43",
    ],
    messages: &[
        "expected `=` after the updated index",
        "expected `]` after the index",
        "an element of a row is updated with `x with [i][j] = v`, one index per dimension; a run \
         of a row is updated as `x with [i] = (x[i] with [a..b] = v)`",
        "expected an expression",
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

const CASES: [Case; 7] = [
    Case {
        fixture: "valid-shapes.or",
        runs: &[
            CLEAN_CHECK,
            Run {
                arguments: &["test"],
                expectation: Expectation::Exact {
                    stdout: shapes_tests,
                },
            },
            Run {
                arguments: &[
                    "eval", "--spec", "cube", "--spec", "hyper", "--spec", "residues",
                ],
                expectation: Expectation::Exact {
                    stdout: shapes_values,
                },
            },
        ],
        rules: &[
            "S3U-01", "S3U-03", "S3U-04", "S3U-05", "S3U-06", "S3U-07", "S3U-08", "S3U-10",
            "S3U-12",
        ],
    },
    Case {
        fixture: "valid-aes-state.or",
        runs: &[
            CLEAN_CHECK,
            Run {
                arguments: &["test"],
                expectation: Expectation::Exact { stdout: aes_tests },
            },
        ],
        rules: &["S3U-04", "S3U-05", "S3U-06", "S3U-07", "S3U-08", "S3U-12"],
    },
    Case {
        fixture: "valid-keccak-state.or",
        runs: &[
            CLEAN_CHECK,
            Run {
                arguments: &["test"],
                expectation: Expectation::Exact {
                    stdout: keccak_tests,
                },
            },
        ],
        rules: &["S3U-04", "S3U-06", "S3U-07", "S3U-08", "S3U-12"],
    },
    Case {
        fixture: "valid-mlkem-matrix.or",
        runs: &[
            CLEAN_CHECK,
            Run {
                arguments: &["test", "--steps", "16777216"],
                expectation: Expectation::Exact {
                    stdout: mlkem_tests,
                },
            },
        ],
        rules: &[
            "S3U-01", "S3U-03", "S3U-04", "S3U-05", "S3U-06", "S3U-07", "S3U-08", "S3U-12",
        ],
    },
    Case {
        fixture: "invalid-shapes.or",
        runs: &rejected(SHAPES),
        rules: &["S3U-01", "S3U-02", "S3U-10", "S3U-12"],
    },
    Case {
        fixture: "invalid-paths.or",
        runs: &rejected(PATHS),
        rules: &["S3U-04", "S3U-07", "S3U-12"],
    },
    Case {
        fixture: "invalid-path-syntax.or",
        runs: &rejected(PATH_SYNTAX),
        rules: &["S3U-06", "S3U-12"],
    },
];

const GENERATED_EVIDENCE: &[(&str, &[&str])] = &[
    (
        "s3u_rank_and_scalar_limits_are_checked_before_evaluation",
        &["S3U-01", "S3U-02", "S3U-03", "S3U-12"],
    ),
    (
        "s3u_each_path_index_checks_its_own_axis",
        &["S3U-04", "S3U-07", "S3U-12"],
    ),
    (
        "s3u_paths_agree_with_nested_updates_at_every_position",
        &["S3U-06", "S3U-07", "S3U-08", "S3U-12"],
    ),
    (
        "s3u_path_cost_is_the_sum_of_the_copied_levels",
        &["S3U-09", "S3U-12"],
    ),
    (
        "s3u_equality_cost_is_independent_of_difference_position",
        &["S3U-03", "S3U-12"],
    ),
    (
        "s3u_rank_three_and_four_witnesses_replay",
        &["S3U-10", "S3U-12"],
    ),
    (
        "s3u_unchanged_boundaries_stay_refused",
        &["S3U-05", "S3U-11", "S3U-12"],
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

fn steps(output: &Output, name: &str, context: &str) -> u64 {
    assert_eq!(
        output.status.code(),
        Some(0),
        "{context}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let prefix = format!("costs::{name}: ");
    String::from_utf8_lossy(&output.stderr)
        .lines()
        .find_map(|line| {
            line.strip_prefix(&prefix)?
                .strip_suffix(" steps")?
                .parse()
                .ok()
        })
        .unwrap_or_else(|| panic!("{context}: no step count for {name}"))
}

fn scratch_directory(name: &str) -> PathBuf {
    let directory = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("orangec-s3u-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&directory);
    fs::create_dir(&directory).unwrap();
    directory
}

#[test]
fn s3u_rule_index_is_exact_and_covered() {
    let documented: Vec<_> = SPECIFICATION
        .lines()
        .filter_map(|line| {
            let (rule, _) = line.strip_prefix("| `")?.split_once("` |")?;
            if !rule.starts_with("S3U-") {
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
            _ => panic!("unknown S3u evidence label {label:?}"),
        };
        assert_eq!(
            observed.get(rule).copied().unwrap_or_default() & required,
            required,
            "{rule} requires {label}"
        );
    }
}

#[test]
fn s3u_cli_conformance_corpus_is_exact_and_repeatable() {
    let directory = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/s3u");
    let mut observed: Vec<_> = fs::read_dir(&directory)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().into_string().unwrap())
        .collect();
    observed.sort();
    let mut expected: Vec<_> = CASES.iter().map(|case| case.fixture).collect();
    expected.sort_unstable();
    assert_eq!(observed, expected, "unexpected S3u fixture inventory");
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
fn s3u_rank_and_scalar_limits_are_checked_before_evaluation() {
    let directory = scratch_directory("limits");
    let path = directory.join("limits.or");
    // Every shape at or under the scalar limit, from the innermost axis out.
    for axes in [
        [16_u32, 16, 16, 16],
        [65536, 1, 1, 1],
        [1, 1, 1, 65536],
        [2, 2, 2, 8192],
        [256, 16, 16, 1],
    ] {
        let [first, second, third, fourth] = axes;
        let corner = axes
            .iter()
            .rev()
            .map(|axis| (axis - 1).to_string())
            .collect::<Vec<_>>()
            .join("][");
        fs::write(&path, format!("edition 2026;\nmodule limits {{\n  type Row = Word[8]^{first};\n  type Plane = Row^{second};\n  type Cube = Plane^{third};\n  type Hyper = Cube^{fourth};\n  spec corner() -> Word[8] {{ let h: Hyper = [[[[7; {first}]; {second}]; {third}]; {fourth}]; h[{corner}] }}\n}}\n")).unwrap();
        let context = format!("{axes:?}");
        let output = run_twice(&["check"], &path, &context);
        assert_exact(&output, 0, "", "", &context);
        let output = run_twice(&["eval"], &path, &context);
        assert_exact(&output, 0, "limits::corner: Word[8] = 0x07\n", "", &context);
    }
    // A cube displays and exports every scalar at the limit.
    fs::write(&path, "edition 2026;\nmodule limits { type Row = Word[8]^64; type Plane = Row^32; type Cube = Plane^32; spec all() -> Cube { [[[7; 64]; 32]; 32] } }\n").unwrap();
    let row = format!("[{}]", vec!["0x07"; 64].join(", "));
    let plane = format!("[{}]", vec![row; 32].join(", "));
    let cube = format!("[{}]", vec![plane; 32].join(", "));
    let output = run_twice(&["eval"], &path, "a full cube");
    assert_exact(
        &output,
        0,
        &format!("limits::all: ((Word[8]^64)^32)^32 = {cube}\n"),
        "",
        "a full cube",
    );
    for (source, code, message) in [
        (
            "type Row = Word[8]^16; type Plane = Row^16; type Cube = Plane^16; \
             type Hyper = Cube^17;",
            "ORC0221",
            "an array shape has 69632 scalar elements, exceeding 65536",
        ),
        (
            "type Row = Word[8]^2; type Plane = Row^2; type Cube = Plane^16385;",
            "ORC0221",
            "an array shape has 65540 scalar elements, exceeding 65536",
        ),
        (
            "type Row = Word[8]^1; type Plane = Row^1; type Cube = Plane^0;",
            "ORC0221",
            "an array length must be a decimal integer from 1 through 65536",
        ),
        (
            "type Row = Word[8]^1; type Plane = Row^1; type Cube = Plane^1; type Hyper = Cube^1; \
             spec f(x: Hyper^1) -> Int { 0 }",
            "ORC0203",
            "`Hyper` already has 4 array dimensions",
        ),
        (
            "type Row = Word[8]^1; type Plane = Row^1; type Cube = Plane^1; type Hyper = Cube^1; \
             spec f() -> (Hyper^1, Bool) { 0 }",
            "ORC0203",
            "`Hyper` already has 4 array dimensions",
        ),
        (
            "type Row = Word[8]^16; type Plane = Row^16; \
             spec f[n in 256..258](p: Plane) -> Plane^n { [p; n] }",
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
fn s3u_each_path_index_checks_its_own_axis() {
    let directory = scratch_directory("indices");
    let path = directory.join("indices.or");
    fs::write(&path, "edition 2026;\nmodule indices {\n  type Row = Word[8]^256;\n  type Plane = Row^16;\n  type Cube = Plane^16;\n  spec put(c: Cube, i: Word[8], j: Word[8], k: Word[8]) -> Cube { c with [i % 16][j % 16][k] = 9 }\n  spec corners() -> Word[8]^3 { let c: Cube = put(put([[[7; 256]; 16]; 16], 0, 0, 0), 255, 255, 255); [c[0][0][0], c[15][15][255], c[15][15][254]] }\n}\n").unwrap();
    let output = run_twice(
        &["eval", "--spec", "corners"],
        &path,
        "computed indices on every full axis",
    );
    assert_exact(
        &output,
        0,
        "indices::corners: Word[8]^3 = [0x09, 0x09, 0x07]\n",
        "",
        "computed indices on every full axis",
    );
    for (path_text, code, message) in [
        (
            "[16][0][0]",
            "ORC0223",
            "index 16 is out of range for `((Word[8]^256)^16)^16`",
        ),
        (
            "[0][16][0]",
            "ORC0223",
            "index 16 is out of range for `(Word[8]^256)^16`",
        ),
        (
            "[0][0][256]",
            "ORC0223",
            "index 256 is out of range for `Word[8]^256`",
        ),
        (
            "[i][0][0]",
            "ORC0223",
            "this index runs from 0 through 255, out of range for `((Word[8]^256)^16)^16`",
        ),
        (
            "[0][i % 17][0]",
            "ORC0223",
            "this index runs from 0 through 16, out of range for `(Word[8]^256)^16`",
        ),
        (
            "[0][0][0][0]",
            "ORC0224",
            "these 4 indices reach past the array's scalars",
        ),
        ("[0][0][-1]", "ORC0223", "out of range for `Word[8]^256`"),
    ] {
        fs::write(&path, format!("edition 2026;\nmodule indices {{ type Row = Word[8]^256; type Plane = Row^16; type Cube = Plane^16; spec bad(c: Cube, i: Word[8]) -> Cube {{ c with {path_text} = 1 }} }}\n")).unwrap();
        let output = run_twice(&["check"], &path, path_text);
        assert_rejected(&output, code, message, path_text);
    }
    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn s3u_paths_agree_with_nested_updates_at_every_position() {
    let directory = scratch_directory("nested");
    let path = directory.join("nested.or");
    // A hypercube of distinct bytes, updated at every position by paths of
    // two, three, and four indices, each compared with its nested form.
    fs::write(&path, concat!(
        "edition 2026;\nmodule nested {\n",
        "  type Row = Word[8]^3; type Plane = Row^2; type Cube = Plane^3; type Hyper = Cube^2;\n",
        "  spec start() -> Hyper {\n",
        "    for a in 0..2 with h: Hyper = [[[[0; 3]; 2]; 3]; 2] {\n",
        "      for b in 0..3 with s: Hyper = h {\n",
        "        for c in 0..2 with t: Hyper = s {\n",
        "          for d in 0..3 with u: Hyper = t {\n",
        "            u with [a][b][c][d] = ((18 * a) + (6 * b) + (3 * c) + d + 1) as Word[8]\n",
        "          }\n        }\n      }\n    }\n  }\n",
        "  test \"four indices\" {\n",
        "    let h: Hyper = start();\n",
        "    for a in 0..2 with ok: Bool = true {\n",
        "      for b in 0..3 with p: Bool = ok {\n",
        "        for c in 0..2 with q: Bool = p {\n",
        "          for d in 0..3 with r: Bool = q {\n",
        "            r && ((h with [a][b][c][d] = 0) == (h with [a] = (h[a] with [b] = (h[a][b] with [c] = (h[a][b][c] with [d] = 0)))))\n",
        "              && (h[a][b][c][d] == (((18 * a) + (6 * b) + (3 * c) + d + 1) as Word[8]))\n",
        "          }\n        }\n      }\n    }\n  }\n",
        "  test \"three indices\" {\n",
        "    let h: Hyper = start();\n",
        "    for a in 0..2 with ok: Bool = true {\n",
        "      for b in 0..3 with p: Bool = ok {\n",
        "        for c in 0..2 with q: Bool = p {\n",
        "          q && ((h with [a][b][c] = [0, 0, 0]) == (h with [a] = (h[a] with [b] = (h[a][b] with [c] = [0, 0, 0]))))\n",
        "        }\n      }\n    }\n  }\n",
        "  test \"two indices\" {\n",
        "    let h: Hyper = start();\n",
        "    let zero: Plane = [[0; 3]; 2];\n",
        "    for a in 0..2 with ok: Bool = true {\n",
        "      for b in 0..3 with p: Bool = ok {\n",
        "        p && ((h with [a][b] = zero) == (h with [a] = (h[a] with [b] = zero)))\n",
        "      }\n    }\n  }\n",
        "}\n",
    )).unwrap();
    let output = run_twice(&["test"], &path, "paths at every position");
    assert_exact(
        &output,
        0,
        &test_report(&["four indices", "three indices", "two indices"]),
        "",
        "paths at every position",
    );
    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn s3u_path_cost_is_the_sum_of_the_copied_levels() {
    let directory = scratch_directory("costs");
    let path = directory.join("costs.or");
    let bulk = |length: u64| length.div_ceil(64).max(1);
    for axes in [
        [2_u64, 2, 2],
        [65, 2, 2],
        [2, 65, 2],
        [2, 2, 65],
        [128, 4, 128],
        [16, 16, 256],
        [1, 1, 65536],
    ] {
        let [planes, rows, columns] = axes;
        let fill = format!("[[[0; {columns}]; {rows}]; {planes}]");
        let last = format!("[{}][{}][{}]", planes - 1, rows - 1, columns - 1);
        fs::write(&path, format!("edition 2026;\nmodule costs {{ type Row = Word[8]^{columns}; type Plane = Row^{rows}; type Cube = Plane^{planes}; spec base() -> Cube {{ {fill} }} spec path() -> Cube {{ {fill} with {last} = 7 }} }}\n")).unwrap();
        let context = format!("{axes:?}");
        let base = steps(
            &run_twice(&["eval", "--stats", "--spec", "base"], &path, &context),
            "base",
            &context,
        );
        let output = run_twice(&["eval", "--stats", "--spec", "path"], &path, &context);
        let updated = steps(&output, "path", &context);
        // Three index literals and the value cost a step each; the path
        // copies one array at each level it passes through.
        let copied: u64 = axes.into_iter().map(bulk).sum();
        assert_eq!(updated, base + 4 + copied, "{context}");
        // Exactly that budget suffices; one step fewer stops atomically.
        let exact = updated.to_string();
        let output = run_twice(
            &["eval", "--spec", "path", "--steps", &exact],
            &path,
            &context,
        );
        assert_eq!(output.status.code(), Some(0), "{context}");
        let short = (updated - 1).to_string();
        let output = run_twice(
            &["eval", "--spec", "path", "--steps", &short],
            &path,
            &context,
        );
        assert_rejected(
            &output,
            "ORC0301",
            "reference evaluation step limit exceeded",
            &context,
        );
    }
    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn s3u_equality_cost_is_independent_of_difference_position() {
    let directory = scratch_directory("equality");
    let path = directory.join("equality.or");
    let mut costs = BTreeSet::new();
    for position in ["[0][0][0]", "[0][15][255]", "[7][8][128]", "[15][15][255]"] {
        for (value, equal) in [(0, true), (1, false)] {
            fs::write(&path, format!("edition 2026;\nmodule costs {{ type Row = Word[8]^256; type Plane = Row^16; type Cube = Plane^16; spec compared() -> Bool {{ let x: Cube = [[[0; 256]; 16]; 16]; x == (x with {position} = {value}) }} }}\n")).unwrap();
            let context = format!("{position} = {value}");
            let output = run_twice(&["eval", "--stats"], &path, &context);
            assert_eq!(
                String::from_utf8_lossy(&output.stdout),
                format!("costs::compared: Bool = {equal}\n"),
                "{context}"
            );
            costs.insert(steps(&output, "compared", &context));
        }
    }
    assert_eq!(
        costs.len(),
        1,
        "equality cost depends on position: {costs:?}"
    );
    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn s3u_rank_three_and_four_witnesses_replay() {
    let directory = scratch_directory("replay");
    let source = directory.join("replay.or");
    let witness = directory.join("witness.values");
    fs::write(&source, "edition 2026;\nmodule m {\n  type Row = Word[8]^2; type Plane = Row^3; type Cube = Plane^2; type Hyper = Cube^2;\n  spec p(c: Cube, t: (Hyper, Bool)) -> Bool { (c[1][2][0] == 11) && (t.0[1][0][2][1] == 5) && t.1 }\n}\n").unwrap();
    let cube =
        "[[[0x01, 0x02], [0x03, 0x04], [0x05, 0x06]], [[0x07, 0x08], [0x09, 0x0a], [0x0b, 0x0c]]]";
    let zero =
        "[[[0x00, 0x00], [0x00, 0x00], [0x00, 0x00]], [[0x00, 0x00], [0x00, 0x00], [0x00, 0x00]]]";
    let five =
        "[[[0x00, 0x00], [0x00, 0x00], [0x00, 0x05]], [[0x00, 0x00], [0x00, 0x00], [0x00, 0x00]]]";
    let types = "parameter_types: [((Word[8]^2)^3)^2, ((((Word[8]^2)^3)^2)^2, Bool)]\n";
    for (hyper, outcome) in [
        (format!("[{zero}, {five}]"), "holds_for_this_witness"),
        (format!("[{five}, {zero}]"), "falsified"),
    ] {
        let arguments = format!("[{cube}, ({hyper}, true)]");
        fs::write(&witness, format!("{arguments}\n")).unwrap();
        let output = Command::new(env!("CARGO_BIN_EXE_orangec"))
            .args(["replay", "--function", "m::p", "--witness"])
            .arg(&witness)
            .arg(&source)
            .output()
            .unwrap();
        assert_exact(
            &output,
            0,
            &format!("m::p[]: {outcome}\n{types}arguments: {arguments}\n"),
            "",
            outcome,
        );
    }
    // A witness missing one scalar of the innermost axis is not coerced.
    let short = cube.replacen("[0x0b, 0x0c]", "[0x0b]", 1);
    fs::write(&witness, format!("[{short}, ([{zero}, {five}], true)]\n")).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_orangec"))
        .args(["replay", "--function", "m::p", "--witness"])
        .arg(&witness)
        .arg(&source)
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("ORC"));
    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn s3u_unchanged_boundaries_stay_refused() {
    let directory = scratch_directory("boundaries");
    let path = directory.join("boundaries.or");
    for (body, code, message) in [
        (
            "spec f(c: Word[8]^2^2^2) -> Int { 0 }",
            "ORC0101",
            "repeated `^` dimensions are not type syntax",
        ),
        (
            "spec f(c: Cube) -> Word[64] { c as little Word[64] }",
            "ORC0215",
            "`as little` does not convert `((Word[8]^2)^2)^2`",
        ),
        (
            "spec f(c: Cube) -> Cube { c as Cube }",
            "ORC0215",
            "`as` does not convert to the array type `((Word[8]^2)^2)^2`",
        ),
        (
            "type Pair = (Cube, Bool); type Pairs = Pair^2;",
            "ORC0203",
            "`Pair` is a tuple type, so this is an array of tuples",
        ),
        (
            "spec f(c: Cube) -> Cube { c with [0][0..1] = [[1, 2]] }",
            "ORC0101",
            "expected `]` after the index",
        ),
        (
            "spec f(c: Cube) -> Plane^1 { let p: Plane^2 = c[0..2]; p[0][0..1] }",
            "ORC0214",
            "this slice is an array of `Word[8]^2`, but `((Word[8]^2)^2)^1` is required here",
        ),
    ] {
        fs::write(&path, format!("edition 2026;\nmodule boundaries {{ type Row = Word[8]^2; type Plane = Row^2; type Cube = Plane^2; {body} }}\n")).unwrap();
        let output = run_twice(&["check"], &path, body);
        assert_rejected(&output, code, message, body);
    }
    // A slice of an outer level keeps the element type, and a slice of a
    // selected row is a row.
    fs::write(&path, "edition 2026;\nmodule boundaries { type Row = Word[8]^2; type Plane = Row^2; type Cube = Plane^2; spec f() -> Bool { let c: Cube = [[[1, 2], [3, 4]], [[5, 6], [7, 8]]]; let p: Plane^1 = c[1..2]; let r: Row = c[1][0]; (p[0] == c[1]) && (r[1..2] == [6]) } }\n").unwrap();
    let output = run_twice(&["eval"], &path, "slices at every level");
    assert_exact(
        &output,
        0,
        "boundaries::f: Bool = true\n",
        "",
        "slices at every level",
    );
    fs::remove_dir_all(directory).unwrap();
}
