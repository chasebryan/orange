use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use orange_compiler::MAX_SOURCE_BYTES;

fn orangec() -> Command {
    Command::new(env!("CARGO_BIN_EXE_orangec"))
}

fn fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/hello.or")
}

fn typed_fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/typed-answer.or")
}

fn run_with_stdin(arguments: &[&str], input: &[u8]) -> std::process::Output {
    let mut child = orangec()
        .args(arguments)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(input).unwrap();
    child.wait_with_output().unwrap()
}

#[test]
fn checks_the_permanent_fixture_without_output() {
    let output = orangec().arg("check").arg(fixture()).output().unwrap();

    assert!(output.status.success());
    assert_eq!(output.stdout, b"");
    assert_eq!(output.stderr, b"");
}

#[test]
fn checks_multiple_files_in_argument_order() {
    let fixture = fixture();
    let output = orangec()
        .arg("check")
        .arg(&fixture)
        .arg(&fixture)
        .output()
        .unwrap();

    assert!(output.status.success());
    assert_eq!(output.stdout, b"");
    assert_eq!(output.stderr, b"");
}

#[test]
fn checks_and_evaluates_the_typed_fixture_repeatably() {
    let fixture = typed_fixture();
    let checked = orangec().arg("check").arg(&fixture).output().unwrap();
    let first = orangec().arg("eval").arg(&fixture).output().unwrap();
    let second = orangec().arg("eval").arg(&fixture).output().unwrap();

    assert!(checked.status.success());
    assert_eq!(checked.stdout, b"");
    assert_eq!(checked.stderr, b"");
    assert!(first.status.success());
    assert_eq!(first.stderr, b"");
    assert_eq!(first.status.code(), second.status.code());
    assert_eq!(first.stdout, second.stdout);
    assert_eq!(first.stderr, second.stderr);
    assert_eq!(
        String::from_utf8(first.stdout).unwrap(),
        concat!(
            "demo::answer: Int = 42\n",
            "demo::negative: Int = -42\n",
            "demo::mask: Word[8] = 0xff\n",
        )
    );
}

#[test]
fn evaluation_emits_no_partial_values_after_a_semantic_error() {
    let source = concat!(
        "edition 2026; module demo {\n",
        "  spec valid() -> Int { 42 }\n",
        "  spec invalid() -> Word[8] { 256 }\n",
        "}\n",
    );
    let output = run_with_stdin(&["eval", "-"], source.as_bytes());

    assert_eq!(output.status.code(), Some(1));
    assert_eq!(output.stdout, b"");
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("error[ORC0207]"), "{stderr}");
    assert!(
        stderr.contains("outside the range of `Word[8]`"),
        "{stderr}"
    );
}

#[test]
fn evaluation_requires_exactly_one_source_before_reading() {
    let output = orangec()
        .arg("eval")
        .arg("first.or")
        .arg("second.or")
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(2));
    assert_eq!(output.stdout, b"");
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        stderr.starts_with("orangec: command `eval` requires exactly one source file\n\nUsage:")
    );
    assert!(!stderr.contains("ORC1001"));
}

#[test]
fn evaluation_of_an_empty_core_succeeds_without_output() {
    let source = "edition 2026; module demo { spec empty() {} impl empty() {} }\n";
    let output = run_with_stdin(&["eval", "-"], source.as_bytes());

    assert!(output.status.success());
    assert_eq!(output.stdout, b"");
    assert_eq!(output.stderr, b"");
}

#[test]
fn accepts_the_minimal_program_from_standard_input_repeatably() {
    let source = concat!(
        "edition 2026;\n",
        "module demo {\n",
        "  spec identity() {}\n",
        "  impl rounds() {}\n",
        "}\n",
    );
    let first = run_with_stdin(&["check", "-"], source.as_bytes());
    let second = run_with_stdin(&["check", "-"], source.as_bytes());

    assert!(first.status.success());
    assert_eq!(first.stdout, b"");
    assert_eq!(first.stderr, b"");
    assert_eq!(first.stdout, second.stdout);
    assert_eq!(first.stderr, second.stderr);
    assert_eq!(first.status.code(), second.status.code());
}

#[test]
fn reports_mixed_file_errors_in_argument_order() {
    let process_id = std::process::id();
    let invalid = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!(
        "orangec-reports-mixed-file-errors-invalid-{process_id}.or"
    ));
    let missing = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!(
        "orangec-reports-mixed-file-errors-missing-{process_id}.or"
    ));
    let _ = fs::remove_file(&invalid);
    let _ = fs::remove_file(&missing);
    fs::write(&invalid, b"@").unwrap();

    let output = orangec()
        .arg("check")
        .arg(&invalid)
        .arg(&missing)
        .output()
        .unwrap();
    fs::remove_file(&invalid).unwrap();
    let _ = fs::remove_file(&missing);

    assert_eq!(output.status.code(), Some(1));
    assert_eq!(output.stdout, b"");
    let stderr = String::from_utf8(output.stderr).unwrap();
    let lexical_error = stderr.find("error[ORC0001]").unwrap();
    let input_error = stderr.find("error[ORC1001]").unwrap();
    assert!(lexical_error < input_error, "{stderr}");
}

#[test]
fn reports_parser_errors_for_multiple_files_in_argument_order() {
    let process_id = std::process::id();
    let missing_semicolon = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!(
        "orangec-parser-order-missing-semicolon-{process_id}.or"
    ));
    let trailing_module = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!(
        "orangec-parser-order-trailing-module-{process_id}.or"
    ));
    let _ = fs::remove_file(&missing_semicolon);
    let _ = fs::remove_file(&trailing_module);
    fs::write(&missing_semicolon, b"edition 2026\nmodule first {}\n").unwrap();
    fs::write(
        &trailing_module,
        b"edition 2026; module second {} module extra {}\n",
    )
    .unwrap();

    let output = orangec()
        .arg("check")
        .arg(&missing_semicolon)
        .arg(&trailing_module)
        .output()
        .unwrap();
    fs::remove_file(&missing_semicolon).unwrap();
    fs::remove_file(&trailing_module).unwrap();

    assert_eq!(output.status.code(), Some(1));
    assert_eq!(output.stdout, b"");
    let stderr = String::from_utf8(output.stderr).unwrap();
    let first = stderr.find("error[ORC0101]").unwrap();
    let second = stderr.find("error[ORC0104]").unwrap();
    assert!(first < second, "{stderr}");
}

#[test]
fn rejects_more_than_256_source_inputs_before_reading() {
    let mut command = orangec();
    command.arg("check");
    for _ in 0..257 {
        command.arg("missing-source.or");
    }
    let output = command.output().unwrap();

    assert_eq!(output.status.code(), Some(2));
    assert_eq!(output.stdout, b"");
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        stderr.starts_with(
            "orangec: at most 256 source inputs are accepted per invocation\n\nUsage:"
        )
    );
    assert!(!stderr.contains("ORC1001"));
}

#[cfg(unix)]
#[test]
fn rejects_non_regular_unix_source_paths() {
    let output = orangec().arg("check").arg("/dev/null").output().unwrap();

    assert_eq!(output.status.code(), Some(1));
    assert_eq!(output.stdout, b"");
    assert_eq!(
        String::from_utf8(output.stderr).unwrap(),
        concat!(
            "error[ORC1001]: could not read source file `/dev/null`\n",
            "  = note: path does not name a regular file\n",
        )
    );
}

#[test]
fn lex_output_is_exact_and_repeatable() {
    let source = b"module m { claim ok = 0x2a; }\n";
    let first = run_with_stdin(&["--edition", "2026", "lex", "-"], source);
    let second = run_with_stdin(&["lex", "-"], source);

    assert!(first.status.success());
    assert_eq!(first.stderr, b"");
    assert_eq!(first.stdout, second.stdout);
    assert_eq!(
        String::from_utf8(first.stdout).unwrap(),
        concat!(
            "0..6\tKW_MODULE\t\"module\"\n",
            "7..8\tIDENTIFIER\t\"m\"\n",
            "9..10\tLEFT_BRACE\t\"{\"\n",
            "11..16\tKW_CLAIM\t\"claim\"\n",
            "17..19\tIDENTIFIER\t\"ok\"\n",
            "20..21\tEQUAL\t\"=\"\n",
            "22..26\tINTEGER\t\"0x2a\"\n",
            "26..27\tSEMICOLON\t\";\"\n",
            "28..29\tRIGHT_BRACE\t\"}\"\n",
            "30..30\tEOF\t\"\"\n",
        )
    );
}

#[test]
fn check_reports_a_stable_source_diagnostic_and_failure_status() {
    let output = run_with_stdin(&["check", "-"], "module café {}\n".as_bytes());

    assert_eq!(output.status.code(), Some(1));
    assert_eq!(output.stdout, b"");
    assert_eq!(
        String::from_utf8(output.stderr).unwrap(),
        concat!(
            "error[ORC0001]: unexpected character U+00E9\n",
            " --> <stdin>:1:11\n",
            "  |\n",
            "1 | module café {}\n",
            "  |           ^ character is not part of Orange 2026\n",
            "  = note: identifiers are ASCII in this pre-alpha edition\n",
        )
    );
}

#[test]
fn check_reports_an_exact_repeatable_parser_diagnostic() {
    let source = b"edition 2026\nmodule demo {}\n";
    let first = run_with_stdin(&["check", "-"], source);
    let second = run_with_stdin(&["check", "-"], source);

    assert_eq!(first.status.code(), Some(1));
    assert_eq!(first.stdout, b"");
    assert_eq!(first.status.code(), second.status.code());
    assert_eq!(first.stdout, second.stdout);
    assert_eq!(first.stderr, second.stderr);
    assert_eq!(
        String::from_utf8(first.stderr).unwrap(),
        concat!(
            "error[ORC0101]: expected `;` after the edition\n",
            " --> <stdin>:2:1\n",
            "  |\n",
            "2 | module demo {}\n",
            "  | ^^^^^^ found KW_MODULE\n",
            "  = note: write `edition 2026;`\n",
        )
    );
}

#[test]
fn lexical_errors_prevent_parser_cascades() {
    let source = b"@ edition 2026; module { spec broken( {}\n";
    let output = run_with_stdin(&["check", "-"], source);

    assert_eq!(output.status.code(), Some(1));
    assert_eq!(output.stdout, b"");
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("error[ORC0001]"), "{stderr}");
    assert!(!stderr.contains("error[ORC01"), "{stderr}");
}

#[test]
fn rejects_non_utf8_source_before_lexing() {
    let process_id = std::process::id();
    let path = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("orangec-rejects-non-utf8-source-{process_id}.or"));
    let _ = fs::remove_file(&path);
    fs::write(&path, [b'm', 0xff]).unwrap();
    let output = orangec().arg("check").arg(&path).output().unwrap();
    fs::remove_file(&path).unwrap();

    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("error[ORC1002]:"));
    assert!(stderr.contains("invalid byte sequence begins at byte offset 1"));
}

#[test]
fn rejects_an_oversized_file_without_reading_it() {
    let process_id = std::process::id();
    let path = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("orangec-rejects-oversized-source-{process_id}.or"));
    let _ = fs::remove_file(&path);
    let file = File::create(&path).unwrap();
    file.set_len(u64::try_from(MAX_SOURCE_BYTES).unwrap() + 1)
        .unwrap();
    drop(file);

    let output = orangec().arg("check").arg(&path).output().unwrap();
    fs::remove_file(&path).unwrap();

    assert_eq!(output.status.code(), Some(1));
    assert_eq!(output.stdout, b"");
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("error[ORC1003]:"));
    assert!(stderr.contains("exceeds the 16777216-byte input limit"));
    assert!(stderr.contains("accepts at most 16 MiB per source"));
}

#[test]
fn rejects_oversized_standard_input_with_a_stable_diagnostic() {
    let input = vec![b' '; MAX_SOURCE_BYTES + 1];
    let output = run_with_stdin(&["check", "-"], &input);

    assert_eq!(output.status.code(), Some(1));
    assert_eq!(output.stdout, b"");
    assert_eq!(
        String::from_utf8(output.stderr).unwrap(),
        concat!(
            "error[ORC1003]: source file `<stdin>` exceeds the 16777216-byte input limit\n",
            "  = note: the pre-alpha compiler accepts at most 16 MiB per source\n",
        )
    );
}

#[test]
fn usage_errors_have_a_distinct_exit_status() {
    let output = orangec().arg("compile").arg("file.or").output().unwrap();

    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.starts_with("orangec: unknown command `compile`\n\nUsage:"));
}
