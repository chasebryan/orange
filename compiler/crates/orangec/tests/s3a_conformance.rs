use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

#[derive(Clone, Copy)]
enum Expectation {
    Success(&'static str),
    Failure {
        diagnostic_codes: &'static [&'static str],
        messages: &'static [&'static str],
        primary_locations: &'static [&'static str],
    },
}

#[derive(Clone, Copy)]
struct Case {
    fixture: &'static str,
    expectation: Expectation,
}

const CASES: [Case; 10] = [
    Case {
        fixture: "valid-empty-mixed.or",
        expectation: Expectation::Success(""),
    },
    Case {
        fixture: "valid-int-radices.or",
        expectation: Expectation::Success(concat!(
            "s3a_int_radices::decimal_positive: Int = 1234567890\n",
            "s3a_int_radices::decimal_zero: Int = 0\n",
            "s3a_int_radices::decimal_negative: Int = -10\n",
            "s3a_int_radices::decimal_negative_zero: Int = 0\n",
            "s3a_int_radices::binary_positive: Int = 165\n",
            "s3a_int_radices::binary_zero: Int = 0\n",
            "s3a_int_radices::binary_negative: Int = -165\n",
            "s3a_int_radices::binary_negative_zero: Int = 0\n",
            "s3a_int_radices::hexadecimal_positive: Int = 3735928559\n",
            "s3a_int_radices::hexadecimal_zero: Int = 0\n",
            "s3a_int_radices::hexadecimal_negative: Int = -42\n",
            "s3a_int_radices::hexadecimal_negative_zero: Int = 0\n",
        )),
    },
    Case {
        fixture: "valid-word8-boundaries.or",
        expectation: Expectation::Success(concat!(
            "s3a_word8_boundaries::zero: Word[8] = 0x00\n",
            "s3a_word8_boundaries::one: Word[8] = 0x01\n",
            "s3a_word8_boundaries::below_high: Word[8] = 0xfe\n",
            "s3a_word8_boundaries::high: Word[8] = 0xff\n",
        )),
    },
    Case {
        fixture: "invalid-typed-impl.or",
        expectation: Expectation::Failure {
            diagnostic_codes: &["ORC0101"],
            messages: &[
                "typed literal bodies are allowed only on `spec` functions",
                "until implementation semantics are defined",
            ],
            primary_locations: &["4:16"],
        },
    },
    Case {
        fixture: "invalid-duplicate-spec.or",
        expectation: Expectation::Failure {
            diagnostic_codes: &["ORC0201", "ORC0201"],
            messages: &[
                "duplicate spec function `same_spec`",
                "duplicate impl function `same_impl`",
                "separate declaration namespaces",
            ],
            primary_locations: &["5:8", "7:8"],
        },
    },
    Case {
        fixture: "invalid-unsupported-type.or",
        expectation: Expectation::Failure {
            diagnostic_codes: &["ORC0203", "ORC0203"],
            messages: &[
                "unsupported result type `Integer`",
                "unsupported result type `Int`",
            ],
            primary_locations: &["4:30", "5:23"],
        },
    },
    Case {
        fixture: "invalid-word-width.or",
        expectation: Expectation::Failure {
            diagnostic_codes: &["ORC0204", "ORC0204", "ORC0204", "ORC0204", "ORC0204"],
            messages: &[
                "`Word` requires the exact width `[8]`",
                "only the exact type `Word[8]` is supported",
                "word widths do not coerce, truncate, or wrap",
            ],
            primary_locations: &["4:27", "5:31", "6:36", "7:34", "8:30"],
        },
    },
    Case {
        fixture: "invalid-int-magnitude.or",
        expectation: Expectation::Failure {
            diagnostic_codes: &["ORC0205"],
            messages: &["integer magnitude exceeds the 16384-significant-bit limit"],
            primary_locations: &["4:25"],
        },
    },
    Case {
        fixture: "invalid-negative-word.or",
        expectation: Expectation::Failure {
            diagnostic_codes: &["ORC0206", "ORC0206"],
            messages: &[
                "`Word[8]` literals cannot be negative",
                "fixed-width words do not wrap or coerce negative integers",
            ],
            primary_locations: &["4:36", "5:37"],
        },
    },
    Case {
        fixture: "invalid-word-range.or",
        expectation: Expectation::Failure {
            diagnostic_codes: &["ORC0207", "ORC0207", "ORC0207"],
            messages: &[
                "literal is outside the range of `Word[8]`",
                "expected a value from 0 through 255",
                "fixed-width words do not truncate or wrap",
            ],
            primary_locations: &["5:31", "6:35", "7:30"],
        },
    },
];

fn orangec() -> Command {
    Command::new(env!("CARGO_BIN_EXE_orangec"))
}

fn fixture_directory() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/s3a")
}

fn run_fixture(command: &str, path: &Path) -> Output {
    orangec().arg(command).arg(path).output().unwrap()
}

fn run_stdin(command: &str, source: &[u8]) -> Output {
    let mut child = orangec()
        .arg(command)
        .arg("-")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(source).unwrap();
    child.wait_with_output().unwrap()
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

fn assert_repeatable(first: &Output, second: &Output, context: &str) {
    assert_eq!(
        first.status.code(),
        second.status.code(),
        "{context} changed exit status"
    );
    assert_eq!(first.stdout, second.stdout, "{context} changed stdout");
    assert_eq!(first.stderr, second.stderr, "{context} changed stderr");
}

fn assert_silent_success(output: &Output, context: &str) {
    assert_eq!(output.status.code(), Some(0), "{context} failed");
    assert_eq!(output.stdout, b"", "{context} stdout");
    assert_eq!(output.stderr, b"", "{context} stderr");
}

fn assert_failure(
    output: &Output,
    expected_codes: &[&str],
    expected_messages: &[&str],
    expected_locations: Option<&[&str]>,
    context: &str,
) {
    assert_eq!(output.status.code(), Some(1), "{context} status");
    assert_eq!(output.stdout, b"", "{context} emitted partial output");
    let stderr = std::str::from_utf8(&output.stderr).unwrap();
    assert_eq!(
        diagnostic_codes(stderr),
        expected_codes,
        "{context} diagnostic set:\n{stderr}"
    );
    if let Some(locations) = expected_locations {
        assert_eq!(
            primary_locations(stderr),
            locations,
            "{context} primary locations:\n{stderr}"
        );
    }
    for message in expected_messages {
        assert!(
            stderr.contains(message),
            "{context} missing {message:?}:\n{stderr}"
        );
    }
}

#[test]
fn s3a_cli_conformance_corpus_is_exact_and_repeatable() {
    let directory = fixture_directory();
    let mut observed: Vec<_> = fs::read_dir(&directory)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().into_string().unwrap())
        .collect();
    observed.sort();
    let mut expected: Vec<_> = CASES.iter().map(|case| case.fixture).collect();
    expected.sort_unstable();
    assert_eq!(observed, expected, "unexpected S3a fixture inventory");

    for case in CASES {
        let path = directory.join(case.fixture);
        let first_check = run_fixture("check", &path);
        let second_check = run_fixture("check", &path);
        let first_eval = run_fixture("eval", &path);
        let second_eval = run_fixture("eval", &path);

        assert_repeatable(
            &first_check,
            &second_check,
            &format!("{} check", case.fixture),
        );
        assert_repeatable(&first_eval, &second_eval, &format!("{} eval", case.fixture));

        match case.expectation {
            Expectation::Success(expected_stdout) => {
                assert_silent_success(&first_check, &format!("{} check", case.fixture));
                assert_eq!(first_eval.status.code(), Some(0), "{} eval", case.fixture);
                assert_eq!(
                    first_eval.stdout,
                    expected_stdout.as_bytes(),
                    "{} eval stdout",
                    case.fixture
                );
                assert_eq!(first_eval.stderr, b"", "{} eval stderr", case.fixture);
            }
            Expectation::Failure {
                diagnostic_codes,
                messages,
                primary_locations,
            } => {
                assert_failure(
                    &first_check,
                    diagnostic_codes,
                    messages,
                    Some(primary_locations),
                    &format!("{} check", case.fixture),
                );
                assert_failure(
                    &first_eval,
                    diagnostic_codes,
                    messages,
                    Some(primary_locations),
                    &format!("{} eval", case.fixture),
                );
                assert_eq!(
                    first_check.stderr, first_eval.stderr,
                    "{} check/eval diagnostic mismatch",
                    case.fixture
                );
            }
        }
    }
}

#[test]
fn s3a_significant_bit_boundary_is_exact_and_repeatable() {
    const INTEGER_BITS: usize = 16_384;

    let accepted_magnitude = format!("8{}", "0".repeat((INTEGER_BITS - 1) / 4));
    let accepted_source = format!(
        "edition 2026; module bit_limit {{ spec exact() -> Int {{ 0x{accepted_magnitude} }} }}\n"
    );
    let first_accepted = run_stdin("check", accepted_source.as_bytes());
    let second_accepted = run_stdin("check", accepted_source.as_bytes());
    assert_repeatable(
        &first_accepted,
        &second_accepted,
        "exact significant-bit boundary",
    );
    assert_silent_success(&first_accepted, "exact significant-bit boundary");

    let rejected_magnitude = format!("1{}", "0".repeat(INTEGER_BITS / 4));
    let rejected_source = format!(
        "edition 2026; module bit_limit {{ spec over() -> Int {{ 0x{rejected_magnitude} }} }}\n"
    );
    let first_check = run_stdin("check", rejected_source.as_bytes());
    let second_check = run_stdin("check", rejected_source.as_bytes());
    let first_eval = run_stdin("eval", rejected_source.as_bytes());
    let second_eval = run_stdin("eval", rejected_source.as_bytes());
    assert_repeatable(&first_check, &second_check, "over-limit check");
    assert_repeatable(&first_eval, &second_eval, "over-limit eval");
    for (context, output) in [
        ("over-limit check", &first_check),
        ("over-limit eval", &first_eval),
    ] {
        assert_failure(
            output,
            &["ORC0205"],
            &["integer magnitude exceeds the 16384-significant-bit limit"],
            None,
            context,
        );
    }
    let expected_location = format!("1:{}", rejected_source.find("0x").unwrap() + 1);
    for output in [&first_check, &first_eval] {
        let stderr = std::str::from_utf8(&output.stderr).unwrap();
        assert_eq!(
            primary_locations(stderr),
            std::slice::from_ref(&expected_location)
        );
    }
    assert_eq!(
        first_check.stderr, first_eval.stderr,
        "over-limit check/eval diagnostic mismatch"
    );
}

#[test]
fn s3a_semantic_diagnostic_budget_is_exact_and_repeatable() {
    let mut source = String::from("edition 2026; module diagnostic_limit {\n");
    let mut expected_locations = Vec::new();
    for index in 0..101 {
        let declaration = format!("  spec bad_{index}() -> Nope {{ {index} }}\n");
        expected_locations.push(format!(
            "{}:{}",
            index + 2,
            declaration.find("Nope").unwrap() + 1
        ));
        source.push_str(&declaration);
    }
    source.push_str("}\n");

    let first_check = run_stdin("check", source.as_bytes());
    let second_check = run_stdin("check", source.as_bytes());
    let first_eval = run_stdin("eval", source.as_bytes());
    let second_eval = run_stdin("eval", source.as_bytes());
    assert_repeatable(&first_check, &second_check, "diagnostic-budget check");
    assert_repeatable(&first_eval, &second_eval, "diagnostic-budget eval");

    let mut expected_codes = vec!["ORC0203"; 100];
    expected_codes.push("ORC0208");
    for (context, output) in [
        ("diagnostic-budget check", &first_check),
        ("diagnostic-budget eval", &first_eval),
    ] {
        assert_failure(
            output,
            &expected_codes,
            &[
                "unsupported result type `Nope`",
                "too many semantic errors; further errors are suppressed",
            ],
            None,
            context,
        );
        let stderr = std::str::from_utf8(&output.stderr).unwrap();
        assert_eq!(
            primary_locations(stderr),
            expected_locations,
            "{context} primary locations"
        );
    }
    assert_eq!(
        first_check.stderr, first_eval.stderr,
        "diagnostic-budget check/eval mismatch"
    );
}
