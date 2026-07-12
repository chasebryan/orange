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
fn reports_mixed_file_errors_in_argument_order() {
    let invalid =
        std::env::temp_dir().join(format!("orangec-ordered-invalid-{}.or", std::process::id()));
    let missing =
        std::env::temp_dir().join(format!("orangec-ordered-missing-{}.or", std::process::id()));
    fs::write(&invalid, b"@").unwrap();
    let _ = fs::remove_file(&missing);

    let output = orangec()
        .arg("check")
        .arg(&invalid)
        .arg(&missing)
        .output()
        .unwrap();
    fs::remove_file(&invalid).unwrap();

    assert_eq!(output.status.code(), Some(1));
    assert_eq!(output.stdout, b"");
    let stderr = String::from_utf8(output.stderr).unwrap();
    let lexical_error = stderr.find("error[ORC0001]").unwrap();
    let input_error = stderr.find("error[ORC1001]").unwrap();
    assert!(lexical_error < input_error, "{stderr}");
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
fn rejects_non_utf8_source_before_lexing() {
    let path = std::env::temp_dir().join(format!("orangec-invalid-utf8-{}.or", std::process::id()));
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
    let path = std::env::temp_dir().join(format!(
        "orangec-oversized-source-{}.or",
        std::process::id()
    ));
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
