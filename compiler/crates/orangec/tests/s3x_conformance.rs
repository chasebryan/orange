//! External conformance evidence for the proposed Orange 2026 S3x slice.
//!
//! The exact fixture inventory and generated programs run through the real
//! `orangec` binary twice. The rule index in `docs/TYPE_LISTS_2026.md` must
//! agree with this evidence map. Earlier slice runners observe the
//! compatibility behavior of the same binary.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const SPECIFICATION: &str = include_str!("../../../../docs/TYPE_LISTS_2026.md");
const CONFORMANCE_SOURCE: &str = include_str!("s3x_conformance.rs");
const CLI: u8 = 1;
const GENERATED_CLI: u8 = 2;

const RULES: [&str; 8] = [
    "S3X-01", "S3X-02", "S3X-03", "S3X-04", "S3X-05", "S3X-06", "S3X-07", "S3X-08",
];

const LISTS_CODES: [&str; 8] = [
    "ORC0244", "ORC0244", "ORC0241", "ORC0244", "ORC0244", "ORC0244", "ORC0244", "ORC0244",
];

const LISTS_MESSAGES: [&str; 8] = [
    "`Int` is a built-in type",
    "duplicate type name `P`",
    "`Rings` lists the type `Mod[7]` twice",
    "duplicate type name `Again`",
    "no type list named `Gone`",
    "`P` names a type, not a list of types",
    "`Fields` names a list of types, not a type",
    "`Fields` names a list of types",
];

const SYNTAX_CODES: [&str; 4] = ["ORC0101", "ORC0101", "ORC0101", "ORC0103"];

const SYNTAX_MESSAGES: [&str; 4] = [
    "expected a listed type after `{`",
    "expected a listed type after `,`",
    "expected `{` before the listed types",
    "a `types` declaration cannot follow a function",
];

#[derive(Clone, Copy)]
enum Expectation {
    Exact {
        stdout: fn() -> String,
    },
    Failure {
        codes: &'static [&'static str],
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

fn lists_tests() -> String {
    String::from(concat!(
        "test \"P adds to 1\" ... ok\n",
        "test \"Q adds to 1\" ... ok\n",
        "test \"the place chooses Q\" ... ok\n",
        "test \"Ch is all ones on a 32-bit word\" ... ok\n",
        "test \"Ch is all ones on a 64-bit word\" ... ok\n",
        "test \"Maj keeps the shared bits\" ... ok\n",
        "test \"an inline list is unchanged\" ... ok\n",
        "7 tests: 7 passed, 0 failed\n",
    ))
}

fn lists_values() -> String {
    String::from(concat!(
        "lists::two[P]: Mod[65537] = 2\n",
        "lists::two[Q]: Mod[3329] = 2\n",
        "lists::pair: (Mod[65537], Mod[3329]) = (4001, 4)\n",
        "lists::row: Mod[65537]^2 = [7, 7]\n",
        "lists::ch32: Word[32] = 0xffffffff\n",
        "lists::ch64: Word[64] = 0xffffffffffffffff\n",
        "lists::maj32: Word[32] = 0xffffffff\n",
        "lists::kept: Int = 6\n",
    ))
}

fn caller_tests() -> String {
    String::from(concat!(
        "test \"twice three in the first ring is six\" ... ok\n",
        "test \"four in the second ring is eight\" ... ok\n",
        "2 tests: 2 passed, 0 failed\n",
    ))
}

fn caller_values() -> String {
    String::from(concat!(
        "caller::go: Mod[7] = 6\n",
        "caller::both: (Mod[7], Mod[11]) = (6, 8)\n",
    ))
}

fn ring_values() -> String {
    String::from(concat!(
        "ring::zero[P]: Mod[7] = 0\n",
        "ring::zero[Q]: Mod[11] = 0\n",
    ))
}

const fn exact(stdout: fn() -> String) -> Run {
    Run {
        arguments: &["eval"],
        expectation: Expectation::Exact { stdout },
    }
}

const LISTS_RUNS: &[Run] = &[
    Run {
        arguments: &["check"],
        expectation: Expectation::Exact { stdout: nothing },
    },
    Run {
        arguments: &["eval"],
        expectation: Expectation::Exact {
            stdout: lists_values,
        },
    },
    Run {
        arguments: &["test"],
        expectation: Expectation::Exact {
            stdout: lists_tests,
        },
    },
    Run {
        arguments: &["fmt", "--check"],
        expectation: Expectation::Exact { stdout: nothing },
    },
];

const CALLER_RUNS: &[Run] = &[
    Run {
        arguments: &["check"],
        expectation: Expectation::Exact { stdout: nothing },
    },
    exact(caller_values),
    Run {
        arguments: &["test"],
        expectation: Expectation::Exact {
            stdout: caller_tests,
        },
    },
];

const RING_RUNS: &[Run] = &[
    Run {
        arguments: &["check"],
        expectation: Expectation::Exact { stdout: nothing },
    },
    exact(ring_values),
];

const fn rejected(codes: &'static [&'static str], messages: &'static [&'static str]) -> Run {
    Run {
        arguments: &["check"],
        expectation: Expectation::Failure { codes, messages },
    }
}

const CASES: &[Case] = &[
    Case {
        fixture: "valid-lists.or",
        runs: LISTS_RUNS,
        rules: &["S3X-01", "S3X-02", "S3X-06", "S3X-07", "S3X-08"],
    },
    Case {
        fixture: "caller.or",
        runs: CALLER_RUNS,
        rules: &["S3X-03", "S3X-08"],
    },
    Case {
        fixture: "ring.or",
        runs: RING_RUNS,
        rules: &["S3X-01", "S3X-03", "S3X-08"],
    },
    Case {
        fixture: "invalid-lists.or",
        runs: &[rejected(&LISTS_CODES, &LISTS_MESSAGES)],
        rules: &["S3X-04", "S3X-05", "S3X-08"],
    },
    Case {
        fixture: "invalid-lists-syntax.or",
        runs: &[rejected(&SYNTAX_CODES, &SYNTAX_MESSAGES)],
        rules: &["S3X-04", "S3X-08"],
    },
];

const GENERATED_EVIDENCE: &[(&str, &[&str])] = &[
    (
        "s3x_a_list_of_256_types_is_one_instance_each_and_257_is_rejected",
        &["S3X-05", "S3X-08"],
    ),
    (
        "s3x_inline_lists_and_use_order_keep_their_meaning",
        &["S3X-04", "S3X-06", "S3X-08"],
    ),
    ("s3x_documentation_names_the_list", &["S3X-07", "S3X-08"]),
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

fn assert_exact(output: &Output, status: i32, stdout: &str, context: &str) {
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
    assert!(
        output.stderr.is_empty(),
        "{context} stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
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

fn assert_failure(output: &Output, codes: &[&str], messages: &[&str], context: &str) {
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
    let fixture = context.split_whitespace().next().unwrap_or(context);
    assert!(
        observed.len() == codes.len()
            && observed
                .iter()
                .all(|name| name.split(':').next() == Some(fixture)),
        "{context} locations {observed:?}:\n{stderr}"
    );
    for message in messages {
        assert!(
            stderr.contains(message),
            "{context} missing {message:?}:\n{stderr}"
        );
    }
}

fn scratch_directory(name: &str) -> PathBuf {
    let directory = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("orangec-s3x-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&directory);
    fs::create_dir(&directory).unwrap();
    directory
}

fn write_module(directory: &Path, name: &str, body: &str) -> PathBuf {
    let path = directory.join(format!("{name}.or"));
    fs::write(
        &path,
        format!("edition 2026;\nmodule {name} {{\n{body}\n}}\n"),
    )
    .unwrap();
    path
}

#[test]
fn s3x_rule_index_is_exact_and_covered() {
    let documented: Vec<_> = SPECIFICATION
        .lines()
        .filter_map(|line| {
            let (rule, _) = line.strip_prefix("| `")?.split_once("` |")?;
            if !rule.starts_with("S3X-") {
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
            _ => panic!("unknown S3x evidence label {label:?}"),
        };
        assert_eq!(
            observed.get(rule).copied().unwrap_or_default() & required,
            required,
            "{rule} requires {label}"
        );
    }
}

#[test]
fn s3x_cli_conformance_corpus_is_exact_and_repeatable() {
    let directory = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/s3x");
    let mut observed: Vec<_> = fs::read_dir(&directory)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().into_string().unwrap())
        .collect();
    observed.sort();
    let mut expected: Vec<_> = CASES.iter().map(|case| case.fixture).collect();
    expected.sort_unstable();
    assert_eq!(observed, expected, "unexpected S3x fixture inventory");
    for case in CASES {
        let path = directory.join(case.fixture);
        for run in case.runs {
            let context = format!("{} {}", case.fixture, run.arguments.join(" "));
            let output = run_twice(run.arguments, &path, &context);
            match run.expectation {
                Expectation::Exact { stdout } => assert_exact(&output, 0, &stdout(), &context),
                Expectation::Failure { codes, messages } => {
                    assert_failure(&output, codes, messages, &context);
                }
            }
        }
    }
}

#[test]
fn s3x_a_list_of_256_types_is_one_instance_each_and_257_is_rejected() {
    let directory = scratch_directory("length");
    let admitted: String = (2..258)
        .map(|modulus| format!("Mod[{modulus}]"))
        .collect::<Vec<_>>()
        .join(", ");
    let path = write_module(
        &directory,
        "length",
        &format!(
            "  types Many = {{{admitted}}};\n  spec zero[K in Many](x: K) -> K {{ x }}\n  \
             spec first() -> Mod[2] {{ zero[Mod[2]](0) }}\n  spec last() -> Mod[257] {{ zero[Mod[257]](0) }}\n"
        ),
    );
    let output = run_twice(&["eval"], &path, "256 types");
    assert_exact(
        &output,
        0,
        "length::first: Mod[2] = 0\nlength::last: Mod[257] = 0\n",
        "256 types",
    );
    let rejected: String = (2..259)
        .map(|modulus| format!("Mod[{modulus}]"))
        .collect::<Vec<_>>()
        .join(", ");
    let path = write_module(
        &directory,
        "length",
        &format!("  types Many = {{{rejected}}};\n  spec zero[K in Many]() -> K {{ 0 }}\n"),
    );
    let output = run_twice(&["check"], &path, "257 types");
    assert_failure(
        &output,
        &["ORC0244"],
        &["`Many` lists 257 types, but a type list has at most 256"],
        "length.or check",
    );
    let path = write_module(
        &directory,
        "length",
        &format!(
            "  types Many = {{{admitted}}};\n  spec wide[K in Many, n in 1..3]() -> K^n {{ [0; n] }}\n"
        ),
    );
    let output = run_twice(&["check"], &path, "256 types and a size");
    assert_failure(
        &output,
        &["ORC0238"],
        &["`wide` has 512 instances, but a function has at most 256"],
        "length.or check",
    );
}

#[test]
fn s3x_inline_lists_and_use_order_keep_their_meaning() {
    let directory = scratch_directory("compat");
    let path = write_module(
        &directory,
        "old",
        "  spec f[K in {Int, Word[8]}](x: K) -> K { x + x }\n  spec g() -> Word[8] { f(3) }\n",
    );
    let output = run_twice(&["eval"], &path, "inline list");
    assert_exact(&output, 0, "old::g: Word[8] = 0x06\n", "inline list");
    let path = write_module(
        &directory,
        "order",
        "  types Fields = {Int, Bool};\n  use other;\n  spec f() -> Int { 1 }\n",
    );
    let output = run_twice(&["check"], &path, "use after types");
    assert_failure(
        &output,
        &["ORC0103"],
        &["a `use` declaration cannot follow a `types` declaration"],
        "order.or check",
    );
}

#[test]
fn s3x_documentation_names_the_list() {
    let directory =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/s3x/valid-lists.or");
    let output = run_twice(&["doc"], &directory, "documentation");
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let html = String::from_utf8(output.stdout).unwrap();
    assert!(html.contains("types Fields"), "{html}");
    assert!(html.contains("types Widths"), "{html}");
    assert!(output.stderr.is_empty());
}
