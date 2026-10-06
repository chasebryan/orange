//! External conformance evidence for the proposed Orange 2026 S3y slice.
//!
//! The exact fixture inventory and generated programs run through the real
//! `orangec` binary twice. The rule index in `docs/COMPUTED_POSITIONS_2026.md` must
//! agree with this evidence map. Earlier slice runners observe the
//! compatibility behavior of the same binary.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const SPECIFICATION: &str = include_str!("../../../../docs/COMPUTED_POSITIONS_2026.md");
const CONFORMANCE_SOURCE: &str = include_str!("s3y_conformance.rs");
const CLI: u8 = 1;
const GENERATED_CLI: u8 = 2;

const RULES: [&str; 11] = [
    "S3Y-01", "S3Y-02", "S3Y-03", "S3Y-04", "S3Y-05", "S3Y-06", "S3Y-07", "S3Y-08", "S3Y-09",
    "S3Y-10", "S3Y-11",
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

fn positions_tests() -> String {
    test_report(&[
        "any Int modulo 8 selects an element",
        "a negative divisor bounds by its magnitude",
        "a named position is the same in every use",
        "a range follows a binding through arithmetic",
        "a word binding keeps the narrowed range of its value",
        "a binding in a loop step ranges over every step",
        "each branch has its own ranged bindings",
        "a window slides to any place it fits",
        "a rotation by an amount from data",
        "a window update at a computed place",
        "a binding with one value is that value",
        "loop indices and ranged bindings move a window together",
    ])
}

fn positions_values() -> String {
    String::from("positions::row: Word[8]^8 = [0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f, 0x10, 0x11]\n")
}

fn sample_tests() -> String {
    test_report(&[
        "A[0, 1]: every coefficient",
        "A[0, 0]: the first eight and the last four coefficients",
        "A[0, 1]: the last coefficient comes from bytes 465 through 467",
        "a stream of rejected candidates leaves the count short",
    ])
}

fn ball_tests() -> String {
    test_report(&[
        "SampleInBall of SHAKE256(0x00..0x1f), tau = 39",
        "tau coefficients are 1 or -1, from 48 bytes of the stream",
        "a stream of rejected bytes places nothing",
    ])
}

const CLEAN_CHECK: Run = Run {
    arguments: &["check"],
    expectation: Expectation::Exact { stdout: nothing },
};

const POSITIONS: Expectation = Expectation::Failure {
    codes: &[
        "ORC0226", "ORC0226", "ORC0226", "ORC0226", "ORC0226", "ORC0226", "ORC0226", "ORC0223",
        "ORC0223", "ORC0223", "ORC0226",
    ],
    locations: &[
        "invalid-positions.or:9:55",
        "invalid-positions.or:10:73",
        "invalid-positions.or:11:89",
        "invalid-positions.or:12:76",
        "invalid-positions.or:13:78",
        "invalid-positions.or:14:50",
        "invalid-positions.or:15:68",
        "invalid-positions.or:16:70",
        "invalid-positions.or:17:73",
        "invalid-positions.or:18:78",
        "invalid-positions.or:19:81",
    ],
    messages: &[
        "an `Int` index may use only integer literals, loop indices, words converted with `as \
         Int`, and ranged bindings",
        "this `Int` has no bound",
        "invalid-positions.or:10:55",
        "invalid-positions.or:11:71",
        "this binding's value has no range",
        "invalid-positions.or:12:48",
        "a name of a tuple pattern has no range",
        "a `let` gives its name its value's range, and `x % 16` lies from 0 through 15 whatever \
         x is",
        "this index runs from 0 through 4, out of range for `Word[8]^4`",
        "this index runs from 1 through 4, out of range for `Word[8]^4`",
        "this index runs from 3 through 255, out of range for `Word[8]^4`",
        "every value an index can take, over every loop index, word, and ranged binding in it, \
         must select an element",
    ],
};

const WINDOWS: Expectation = Expectation::Failure {
    codes: &[
        "ORC0236", "ORC0236", "ORC0223", "ORC0226", "ORC0226", "ORC0222", "ORC0223", "ORC0226",
    ],
    locations: &[
        "invalid-windows.or:8:75",
        "invalid-windows.or:9:93",
        "invalid-windows.or:10:74",
        "invalid-windows.or:11:71",
        "invalid-windows.or:14:49",
        "invalid-windows.or:16:72",
        "invalid-windows.or:17:80",
        "invalid-windows.or:18:55",
    ],
    messages: &[
        "the length of this slice changes with its ranged bindings",
        "its bounds must differ by the same number for every value",
        "b - a must be the same positive number for every value its ranged bindings can take, as \
         in `x[at..at + 4]`",
        "this slice reaches elements 0 through 8, out of range for `Word[8]^8`",
        "every element a slice can take, over every loop index and ranged binding in its bounds, \
         must be an element of the array",
        "a slice's bounds may use only integer literals, loop indices, and ranged bindings",
        "invalid-windows.or:11:57",
        "this binding's value has no range",
        "a slice's bound may multiply a loop index or a ranged binding only by a constant",
        "both operands of this `*` vary",
        "this slice has 4 elements, but `Word[8]^3` has 3",
        "this has a range, but is not a name",
        "a computed position enters a slice's bounds through a name: bind it with `let`, as in \
         `let at: Int = n % 4;`, and write both bounds with `at`",
        "a slice's length never depends on data",
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

const CASES: [Case; 5] = [
    Case {
        fixture: "valid-positions.or",
        runs: &[
            CLEAN_CHECK,
            Run {
                arguments: &["test"],
                expectation: Expectation::Exact {
                    stdout: positions_tests,
                },
            },
            Run {
                arguments: &["eval", "--spec", "row"],
                expectation: Expectation::Exact {
                    stdout: positions_values,
                },
            },
        ],
        rules: &[
            "S3Y-01", "S3Y-02", "S3Y-03", "S3Y-05", "S3Y-06", "S3Y-07", "S3Y-08", "S3Y-11",
        ],
    },
    Case {
        fixture: "valid-mlkem-sample.or",
        runs: &[
            CLEAN_CHECK,
            Run {
                arguments: &["test"],
                expectation: Expectation::Exact {
                    stdout: sample_tests,
                },
            },
        ],
        rules: &["S3Y-01", "S3Y-02", "S3Y-11"],
    },
    Case {
        fixture: "valid-mldsa-ball.or",
        runs: &[
            CLEAN_CHECK,
            Run {
                arguments: &["test"],
                expectation: Expectation::Exact { stdout: ball_tests },
            },
        ],
        rules: &["S3Y-01", "S3Y-02", "S3Y-03", "S3Y-11"],
    },
    Case {
        fixture: "invalid-positions.or",
        runs: &rejected(POSITIONS),
        rules: &["S3Y-01", "S3Y-02", "S3Y-03", "S3Y-04", "S3Y-10", "S3Y-11"],
    },
    Case {
        fixture: "invalid-windows.or",
        runs: &rejected(WINDOWS),
        rules: &[
            "S3Y-04", "S3Y-05", "S3Y-06", "S3Y-07", "S3Y-08", "S3Y-10", "S3Y-11",
        ],
    },
];

const GENERATED_EVIDENCE: &[(&str, &[&str])] = &[
    ("s3y_remainders_bound_every_int", &["S3Y-01", "S3Y-11"]),
    (
        "s3y_ranged_bindings_select_what_their_values_select",
        &["S3Y-02", "S3Y-03", "S3Y-09", "S3Y-11"],
    ),
    (
        "s3y_names_without_a_range_point_at_their_binding",
        &["S3Y-04", "S3Y-11"],
    ),
    (
        "s3y_windows_slide_to_every_place_they_fit",
        &["S3Y-05", "S3Y-07", "S3Y-08", "S3Y-09", "S3Y-11"],
    ),
    (
        "s3y_window_lengths_never_depend_on_data",
        &["S3Y-05", "S3Y-06", "S3Y-11"],
    ),
    (
        "s3y_unchanged_boundaries_stay_refused",
        &["S3Y-10", "S3Y-11"],
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
        .join(format!("orangec-s3y-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&directory);
    fs::create_dir(&directory).unwrap();
    directory
}

/// Returns the file name, line, and column of every location in `stderr`
/// that follows `marker`: ` --> ` for primary labels and ` ::: ` for
/// secondary labels.
fn labelled_locations(stderr: &str, marker: &str) -> Vec<String> {
    stderr
        .lines()
        .filter_map(|line| line.strip_prefix(marker))
        .map(|location| {
            Path::new(location)
                .file_name()
                .unwrap()
                .to_string_lossy()
                .into_owned()
        })
        .collect()
}

/// Returns the one-based column of the first occurrence of `needle` in
/// `line`, moved `offset` characters to the right.
fn column(line: &str, needle: &str, offset: usize) -> usize {
    line.find(needle)
        .unwrap_or_else(|| panic!("{needle:?} is not in {line:?}"))
        + offset
        + 1
}

/// A model of the element a form selects, from `n` and `w`.
type Model = fn(i128, u8) -> i128;

fn list<T: ToString>(values: impl IntoIterator<Item = T>) -> String {
    let values: Vec<_> = values.into_iter().map(|value| value.to_string()).collect();
    format!("[{}]", values.join(", "))
}

#[test]
fn s3y_rule_index_is_exact_and_covered() {
    let documented: Vec<_> = SPECIFICATION
        .lines()
        .filter_map(|line| {
            let (rule, _) = line.strip_prefix("| `")?.split_once("` |")?;
            if !rule.starts_with("S3Y-") {
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
            _ => panic!("unknown S3y evidence label {label:?}"),
        };
        assert_eq!(
            observed.get(rule).copied().unwrap_or_default() & required,
            required,
            "{rule} requires {label}"
        );
    }
}

#[test]
fn s3y_cli_conformance_corpus_is_exact_and_repeatable() {
    let directory = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/s3y");
    let mut observed: Vec<_> = fs::read_dir(&directory)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().into_string().unwrap())
        .collect();
    observed.sort();
    let mut expected: Vec<_> = CASES.iter().map(|case| case.fixture).collect();
    expected.sort_unstable();
    assert_eq!(observed, expected, "unexpected S3y fixture inventory");
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
fn s3y_remainders_bound_every_int() {
    let directory = scratch_directory("remainders");
    let path = directory.join("remainders.or");
    let dividends: Vec<i128> = (-20..=20)
        .chain([
            255,
            256,
            -255,
            -256,
            1000,
            -1001,
            i128::from(i64::MIN),
            i128::from(i64::MAX),
            i128::MIN + 1,
            i128::MAX,
        ])
        .collect();
    for divisor in [1_i128, 2, 3, 7, 8, -8, 13, -16, 100, 256] {
        let size = divisor.unsigned_abs();
        let table = list(0..size);
        // Every dividend selects the element its Euclidean remainder names.
        let mut program = format!(
            "edition 2026;\nmodule remainders {{\n  spec at(n: Int) -> Int {{ let t: Int^{size} \
             = {table}; t[n % {divisor}] }}\n"
        );
        let mut titles = Vec::new();
        for n in &dividends {
            let title = format!("{n} % {divisor}");
            program.push_str(&format!(
                "  test \"{title}\" {{ at({n}) == {} }}\n",
                n.rem_euclid(divisor)
            ));
            titles.push(title);
        }
        program.push_str("}\n");
        fs::write(&path, &program).unwrap();
        let context = format!("remainders by {divisor}");
        let titles: Vec<_> = titles.iter().map(String::as_str).collect();
        let output = run_twice(&["test"], &path, &context);
        assert_exact(&output, 0, &test_report(&titles), "", &context);
        // The bound is tight: one element fewer leaves the greatest
        // remainder outside the array.
        if size > 1 {
            let short = size - 1;
            fs::write(
                &path,
                format!(
                    "edition 2026;\nmodule remainders {{ spec at(t: Int^{short}, n: Int) -> Int \
                     {{ t[n % {divisor}] }} }}\n"
                ),
            )
            .unwrap();
            let output = run_twice(&["check"], &path, &context);
            assert_rejected(
                &output,
                "ORC0223",
                &format!("this index runs from 0 through {short}, out of range for `Int^{short}`"),
                &context,
            );
        }
    }
    // A divisor computed from data bounds the remainder by its greatest
    // magnitude, on either side of zero.
    let table = list(0..256);
    let mut program = format!(
        "edition 2026;\nmodule remainders {{\n  spec above(n: Int, d: Word[8]) -> Int {{ let t: \
         Int^256 = {table}; t[n % ((d as Int) + 1)] }}\n  spec below(n: Int, d: Word[8]) -> Int \
         {{ let t: Int^256 = {table}; t[n % ((d as Int) - 256)] }}\n"
    );
    let mut titles = Vec::new();
    for n in [-70_000_i128, -257, -1, 0, 1, 255, 256, 65_537] {
        for d in [0_i128, 1, 2, 127, 254, 255] {
            let title = format!("{n} by {d}");
            program.push_str(&format!(
                "  test \"{title}\" {{ (above({n}, {d}) == {}) && (below({n}, {d}) == {}) }}\n",
                n.rem_euclid(d + 1),
                n.rem_euclid(d - 256)
            ));
            titles.push(title);
        }
    }
    program.push_str("}\n");
    fs::write(&path, &program).unwrap();
    let titles: Vec<_> = titles.iter().map(String::as_str).collect();
    let output = run_twice(&["test"], &path, "divisors from data");
    assert_exact(&output, 0, &test_report(&titles), "", "divisors from data");
    // A divisor that may be zero bounds nothing.
    for divisor in ["0", "(d as Int)", "((d as Int) - 3)", "((d as Int) * 2)"] {
        fs::write(
            &path,
            format!(
                "edition 2026;\nmodule remainders {{ spec at(t: Int^256, n: Int, d: Word[8]) -> \
                 Int {{ t[n % {divisor}] }} }}\n"
            ),
        )
        .unwrap();
        let output = run_twice(&["check"], &path, divisor);
        assert_rejected(&output, "ORC0226", "this `Int` has no bound", divisor);
    }
    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn s3y_ranged_bindings_select_what_their_values_select() {
    let directory = scratch_directory("bindings");
    let path = directory.join("bindings.or");
    // Each form names a position with `let` and writes the same position in
    // place; both select the element the Rust model computes.
    let forms: [(&str, &str, &str, Model); 11] = [
        ("body", "let k: Int = n % 16; t[k]", "t[n % 16]", |n, _| {
            n.rem_euclid(16)
        }),
        (
            "negative divisor",
            "let k: Int = n % -16; t[k]",
            "t[n % -16]",
            |n, _| n.rem_euclid(-16),
        ),
        (
            "chain",
            "let a: Int = n % 8; let b: Int = a + 8; t[b]",
            "t[(n % 8) + 8]",
            |n, _| n.rem_euclid(8) + 8,
        ),
        (
            "scaled",
            "let a: Int = n % 8; let b: Int = 2 * a; t[b + 1]",
            "t[(2 * (n % 8)) + 1]",
            |n, _| 2 * n.rem_euclid(8) + 1,
        ),
        (
            "quotient",
            "let a: Int = n % 64; let b: Int = a / 4; t[b]",
            "t[(n % 64) / 4]",
            |n, _| n.rem_euclid(64) / 4,
        ),
        (
            "from a word",
            "let k: Int = (w as Int) % 16; t[k]",
            "t[(w as Int) % 16]",
            |_, w| i128::from(w % 16),
        ),
        (
            "word mask",
            "let low: Word[8] = w & 15; t[low]",
            "t[w & 15]",
            |_, w| i128::from(w & 15),
        ),
        (
            "word shift",
            "let high: Word[8] = w >> 4; t[high]",
            "t[w >> 4]",
            |_, w| i128::from(w >> 4),
        ),
        (
            "loop step",
            "for i in 0..4 with s: Int = 0 { let k: Int = (n + i) % 16; s + t[k] }",
            "for i in 0..4 with s: Int = 0 { s + t[(n + i) % 16] }",
            |n, _| (0..4).map(|i| 100 + (n + i).rem_euclid(16)).sum::<i128>() - 100,
        ),
        (
            "branch",
            "if w < 128 { let k: Int = n % 8; t[k] } else { let k: Int = 8 + (n % 8); t[k] }",
            "if w < 128 { t[n % 8] } else { t[8 + (n % 8)] }",
            |n, w| {
                if w < 128 {
                    n.rem_euclid(8)
                } else {
                    8 + n.rem_euclid(8)
                }
            },
        ),
        (
            "one value",
            "let k: Int = n % 1; t[k + 5]",
            "t[5]",
            |_, _| 5,
        ),
    ];
    let table = list((0..16).map(|index| 100 + index));
    let mut program = String::from("edition 2026;\nmodule bindings {\n");
    let mut titles = Vec::new();
    for (index, (name, bound, inline, model)) in forms.iter().enumerate() {
        program.push_str(&format!(
            "  // {name}\n  spec bound_{index}(n: Int, w: Word[8]) -> Int {{ let t: Int^16 = \
             {table}; {bound} }}\n  spec inline_{index}(n: Int, w: Word[8]) -> Int {{ let t: \
             Int^16 = {table}; {inline} }}\n"
        ));
        for n in [-1000_i128, -17, -16, -1, 0, 1, 7, 8, 15, 16, 31, 1_000_003] {
            for w in [0_u8, 15, 16, 127, 128, 255] {
                let title = format!("{name}: n = {n}, w = {w}");
                program.push_str(&format!(
                    "  test \"{title}\" {{ (bound_{index}({n}, {w}) == inline_{index}({n}, {w})) \
                     && (bound_{index}({n}, {w}) == {}) }}\n",
                    100 + model(n, w)
                ));
                titles.push(title);
            }
        }
    }
    program.push_str("}\n");
    fs::write(&path, &program).unwrap();
    let titles: Vec<_> = titles.iter().map(String::as_str).collect();
    let output = run_twice(&["test"], &path, "ranged bindings");
    assert_exact(&output, 0, &test_report(&titles), "", "ranged bindings");
    // A ranged binding must select in range for every value it can take,
    // exactly as its value written in place must.
    for (bound, inline, message) in [
        (
            "let k: Int = n % 17; t[k]",
            "t[n % 17]",
            "this index runs from 0 through 16, out of range for `Int^16`",
        ),
        (
            "let k: Int = n % 16; t[k + 1]",
            "t[(n % 16) + 1]",
            "this index runs from 1 through 16, out of range for `Int^16`",
        ),
        (
            "let k: Word[8] = w | 1; t[k]",
            "t[w | 1]",
            "this index runs from 1 through 255, out of range for `Int^16`",
        ),
    ] {
        for body in [bound, inline] {
            fs::write(
                &path,
                format!(
                    "edition 2026;\nmodule bindings {{ spec f(t: Int^16, n: Int, w: Word[8]) -> \
                     Int {{ {body} }} }}\n"
                ),
            )
            .unwrap();
            let output = run_twice(&["check"], &path, body);
            assert_rejected(&output, "ORC0223", message, body);
        }
    }
    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn s3y_names_without_a_range_point_at_their_binding() {
    let directory = scratch_directory("names");
    let path = directory.join("names.or");
    let has_none = "this binding's value has no range";
    // A spec, the text its diagnostic points at with the offset of the
    // position in it, and the binding its secondary label points at.
    for (spec, (used, offset), binding) in [
        (
            "spec f(x: Word[8]^8, n: Int) -> Word[8] { x[n] }",
            ("x[n]", 2),
            None,
        ),
        (
            "spec f(x: Word[8]^8, n: Int) -> Word[8] { let k: Int = n * 2; x[k] }",
            ("x[k]", 2),
            Some(("k: Int", has_none)),
        ),
        (
            "spec f(x: Word[8]^8, n: Int) -> Word[8] { let k: Int = n / 4; x[k] }",
            ("x[k]", 2),
            Some(("k: Int", has_none)),
        ),
        (
            "spec f(x: Word[8]^8, n: Int) -> Word[8] { let k: Int = n; let j: Int = k + 1; x[j] }",
            ("x[j]", 2),
            Some(("j: Int", has_none)),
        ),
        (
            "spec f(x: Word[8]^8) -> Word[8] { let (k: Int, j: Int) = (1, 2); x[j] }",
            ("x[j]", 2),
            Some(("j: Int", "a name of a tuple pattern has no range")),
        ),
        (
            "spec f(x: Word[8]^8, n: Int) -> Word[8] { for i in 0..4 with s: Word[8] = 0 { let \
             k: Int = n + i; s ^ x[k] } }",
            ("x[k]", 2),
            Some(("k: Int", has_none)),
        ),
        (
            "spec f(x: Word[8]^8, n: Int, b: Bool) -> Word[8] { if b { let k: Int = n - 1; x[k] \
             } else { 0 } }",
            ("x[k]", 2),
            Some(("k: Int", has_none)),
        ),
        (
            "spec f(x: Word[8]^8) -> Int { for i in 0..4 with k: Int = 0 { k + (x[k] as Int) } }",
            ("x[k]", 2),
            None,
        ),
        (
            "spec f(x: Word[8]^8, n: Int) -> Word[8]^2 { let k: Int = n; x[k..k + 2] }",
            ("x[k..", 2),
            Some(("k: Int", has_none)),
        ),
    ] {
        let line = format!("  {spec}");
        fs::write(
            &path,
            format!("edition 2026;\nmodule names {{\n{line}\n}}\n"),
        )
        .unwrap();
        let output = run_twice(&["check"], &path, spec);
        assert_eq!(output.status.code(), Some(1), "{spec} status");
        let stderr = std::str::from_utf8(&output.stderr).unwrap();
        assert_eq!(diagnostic_codes(stderr), ["ORC0226"], "{spec}:\n{stderr}");
        assert_eq!(
            labelled_locations(stderr, " --> "),
            [format!("names.or:3:{}", column(&line, used, offset))],
            "{spec}:\n{stderr}"
        );
        let secondary: Vec<_> = binding
            .iter()
            .map(|(name, _)| format!("names.or:3:{}", column(&line, name, 0)))
            .collect();
        assert_eq!(
            labelled_locations(stderr, " ::: "),
            secondary,
            "{spec}:\n{stderr}"
        );
        if let Some((_, label)) = binding {
            assert!(
                stderr.contains(label),
                "{spec} missing {label:?}:\n{stderr}"
            );
        }
    }
    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn s3y_windows_slide_to_every_place_they_fit() {
    let directory = scratch_directory("windows");
    let path = directory.join("windows.or");
    for (size, length) in [(8_i128, 4_i128), (8, 1), (8, 8), (16, 3), (32, 31)] {
        let places = size - length + 1;
        let row = list((0..size).map(|index| 100 + index));
        let fresh = list((0..length).map(|index| 200 + index));
        let mut program = format!(
            "edition 2026;\nmodule windows {{\n  spec window(x: Int^{size}, n: Int) -> \
             Int^{length} {{ let at: Int = n % {places}; x[at..at + {length}] }}\n  spec \
             stamp(x: Int^{size}, n: Int, v: Int^{length}) -> Int^{size} {{ let at: Int = n % \
             {places}; x with [at..at + {length}] = v }}\n"
        );
        let mut titles = Vec::new();
        for n in (-places - 1)..=(2 * places + 1) {
            let at = n.rem_euclid(places);
            let window = list((at..at + length).map(|index| 100 + index));
            let stamped = list((0..size).map(|index| {
                if (at..at + length).contains(&index) {
                    200 + index - at
                } else {
                    100 + index
                }
            }));
            let title = format!("{length} of {size} at {n}");
            program.push_str(&format!(
                "  test \"{title}\" {{ (window({row}, {n}) == {window}) && (stamp({row}, {n}, \
                 {fresh}) == {stamped}) }}\n"
            ));
            titles.push(title);
        }
        program.push_str("}\n");
        fs::write(&path, &program).unwrap();
        let context = format!("{length} of {size}");
        let titles: Vec<_> = titles.iter().map(String::as_str).collect();
        let output = run_twice(&["test"], &path, &context);
        assert_exact(&output, 0, &test_report(&titles), "", &context);
        // One more place reaches past the end, for reading and for update.
        let beyond = places + 1;
        let window = format!(
            "  spec window(x: Int^{size}, n: Int) -> Int^{length} {{ let at: Int = n % {beyond}; \
             x[at..at + {length}] }}"
        );
        let stamp = format!(
            "  spec stamp(x: Int^{size}, n: Int, v: Int^{length}) -> Int^{size} {{ let at: Int = \
             n % {beyond}; x with [at..at + {length}] = v }}"
        );
        fs::write(
            &path,
            format!("edition 2026;\nmodule windows {{\n{window}\n{stamp}\n}}\n"),
        )
        .unwrap();
        let output = run_twice(&["check"], &path, &context);
        let locations = [
            format!("windows.or:3:{}", column(&window, "[at..", 1)),
            format!("windows.or:4:{}", column(&stamp, "[at..", 1)),
        ];
        let locations: Vec<_> = locations.iter().map(String::as_str).collect();
        assert_failure(
            &output,
            &["ORC0223", "ORC0223"],
            &locations,
            &[&format!(
                "this slice reaches elements 0 through {size}, out of range for `Int^{size}`"
            )],
            &context,
        );
    }
    // Windows over several ranged bindings and loop indices, with constant
    // factors, move together and select what the model selects.
    let row = list((0..16).map(|index| 100 + index));
    let mut program = String::from(
        "edition 2026;\nmodule windows {\n  spec odd(x: Int^16, n: Int) -> Int^2 { let at: Int = \
         n % 7; x[2 * at + 1..2 * at + 3] }\n  spec two(x: Int^16, n: Int, w: Word[8]) -> Int^4 \
         { let a: Int = n % 5; let b: Int = (w as Int) % 8; x[a + b..a + b + 4] }\n  spec \
         sum(x: Int^16, n: Int) -> Int^3 { let at: Int = n % 6; for i in 0..8 with s: Int^3 = \
         [0, 0, 0] { let w: Int^3 = x[at + i..at + i + 3]; [s[0] + w[0], s[1] + w[1], s[2] + \
         w[2]] } }\n",
    );
    let mut titles = Vec::new();
    for n in [-9_i128, -1, 0, 1, 4, 5, 6, 13, 1_000_000] {
        for w in [0_i128, 3, 7, 200] {
            let odd = 2 * n.rem_euclid(7) + 1;
            let two = n.rem_euclid(5) + w.rem_euclid(8);
            let at = n.rem_euclid(6);
            let sum = list((0..3).map(|j| (0..8).map(|i| 100 + at + i + j).sum::<i128>()));
            let title = format!("n = {n}, w = {w}");
            program.push_str(&format!(
                "  test \"{title}\" {{ ((odd({row}, {n}) == {}) && (two({row}, {n}, {w}) == {})) \
                 && (sum({row}, {n}) == {sum}) }}\n",
                list([100 + odd, 101 + odd]),
                list((two..two + 4).map(|index| 100 + index)),
            ));
            titles.push(title);
        }
    }
    program.push_str("}\n");
    fs::write(&path, &program).unwrap();
    let titles: Vec<_> = titles.iter().map(String::as_str).collect();
    let output = run_twice(&["test"], &path, "windows together");
    assert_exact(&output, 0, &test_report(&titles), "", "windows together");
    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn s3y_window_lengths_never_depend_on_data() {
    let directory = scratch_directory("lengths");
    let path = directory.join("lengths.or");
    // Bounds whose difference cancels to a constant are accepted and select
    // what the model selects.
    let row = list((0..16).map(|index| 100 + index));
    let mut program = String::from(
        "edition 2026;\nmodule lengths {\n  spec shifted(x: Int^16, n: Int) -> Int^2 { let k: \
         Int = n % 4; x[k + 1..k + 3] }\n  spec doubled(x: Int^16, n: Int) -> Int^2 { let k: \
         Int = n % 3; x[2 * k..2 * k + 2] }\n  spec cancelled(x: Int^16, n: Int) -> Int^2 { \
         let k: Int = n % 5; x[(3 * k) - k..(k + k) + 2] }\n  spec negated(x: Int^16, n: Int) \
         -> Int^3 { let k: Int = n % 4; x[10 - k..13 - k] }\n",
    );
    let mut titles = Vec::new();
    for n in -6_i128..=12 {
        let window = |start: i128, length: i128| list((start..start + length).map(|i| 100 + i));
        let title = format!("n = {n}");
        program.push_str(&format!(
            "  test \"{title}\" {{ ((shifted({row}, {n}) == {}) && (doubled({row}, {n}) == {})) \
             && ((cancelled({row}, {n}) == {}) && (negated({row}, {n}) == {})) }}\n",
            window(n.rem_euclid(4) + 1, 2),
            window(2 * n.rem_euclid(3), 2),
            window(2 * n.rem_euclid(5), 2),
            window(10 - n.rem_euclid(4), 3),
        ));
        titles.push(title);
    }
    program.push_str("}\n");
    fs::write(&path, &program).unwrap();
    let titles: Vec<_> = titles.iter().map(String::as_str).collect();
    let output = run_twice(&["test"], &path, "constant lengths");
    assert_exact(&output, 0, &test_report(&titles), "", "constant lengths");
    // Each kind of changing length has its own message, and a product of
    // two variables is no affine bound.
    for (body, code, message) in [
        (
            "let k: Int = n % 2; x[k..2]",
            "ORC0236",
            "the length of this slice changes with its ranged bindings",
        ),
        (
            "let a: Int = n % 4; let b: Int = m % 4; x[a..b + 2]",
            "ORC0236",
            "its bounds must differ by the same number for every value",
        ),
        (
            "for i in 0..2 with s: Int^2 = [0, 0] { x[0..i + 2] }",
            "ORC0236",
            "the length of this slice changes from step to step",
        ),
        (
            "let k: Int = n % 2; for i in 0..2 with s: Int^2 = [0, 0] { x[i..k + 2] }",
            "ORC0236",
            "the length of this slice changes from step to step and with its ranged bindings",
        ),
        (
            "let k: Int = n % 4; x[k + 2..k + 2]",
            "ORC0236",
            "this slice is empty: its bounds are equal",
        ),
        (
            "let k: Int = n % 4; x[k + 3..k + 1]",
            "ORC0236",
            "this slice ends 2 elements before it starts",
        ),
        (
            "let k: Int = n % 2; for i in 0..2 with s: Int^2 = [0, 0] { x[i * k..i * k + 2] }",
            "ORC0226",
            "both operands of this `*` vary",
        ),
        (
            "let k: Int = n % 2; x[k * k..k * k + 2]",
            "ORC0226",
            "both operands of this `*` vary",
        ),
    ] {
        fs::write(
            &path,
            format!(
                "edition 2026;\nmodule lengths {{ spec f(x: Int^16, n: Int, m: Int) -> Int^2 {{ \
                 {body} }} }}\n"
            ),
        )
        .unwrap();
        let output = run_twice(&["check"], &path, body);
        assert_rejected(&output, code, message, body);
    }
    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn s3y_unchanged_boundaries_stay_refused() {
    let directory = scratch_directory("boundaries");
    let path = directory.join("boundaries.or");
    for (body, code, message) in [
        // A condition does not narrow a range.
        (
            "if (n >= 0) && (n < 8) { x[n] as Int } else { 0 }",
            "ORC0226",
            "this `Int` has no bound",
        ),
        // A parameter has no range.
        ("x[n] as Int", "ORC0226", "this `Int` has no bound"),
        // Nor has an accumulator, or a binding of a loop's value.
        (
            "for i in 0..4 with k: Int = 0 { k + (x[k] as Int) }",
            "ORC0226",
            "this `Int` has no bound",
        ),
        (
            "let t: Int = for i in 0..4 with k: Int = 0 { k + 1 }; x[t] as Int",
            "ORC0226",
            "this binding's value has no range",
        ),
        // A quotient of a value without a range has none.
        (
            "let k: Int = n / 1000; x[k] as Int",
            "ORC0226",
            "this binding's value has no range",
        ),
        // Nor has a product with one.
        (
            "let k: Int = (n % 8) * n; x[k] as Int",
            "ORC0226",
            "this binding's value has no range",
        ),
        // A position computed in a slice's bounds must be named.
        (
            "let w: Word[8]^2 = x[n % 4..(n % 4) + 2]; w[0] as Int",
            "ORC0226",
            "this has a range, but is not a name",
        ),
        (
            "let w: Word[8]^2 = x[(b as Int) % 3..((b as Int) % 3) + 2]; w[0] as Int",
            "ORC0226",
            "this has a range, but is not a name",
        ),
    ] {
        fs::write(
            &path,
            format!(
                "edition 2026;\nmodule boundaries {{ spec f(x: Word[8]^8, n: Int, b: Word[8]) -> \
                 Int {{ {body} }} }}\n"
            ),
        )
        .unwrap();
        let output = run_twice(&["check"], &path, body);
        assert_rejected(&output, code, message, body);
    }
    fs::remove_dir_all(directory).unwrap();
}
