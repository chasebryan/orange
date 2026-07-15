use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use orange_compiler::{
    CoreType, CoreValue, Diagnostic, DiagnosticCode, Edition, FunctionBody, FunctionKind,
    MAX_LEXICAL_DIAGNOSTICS_PER_SOURCE, MAX_SOURCE_BYTES, MAX_TOKENS_PER_SOURCE, Severity,
    SourceMap, analyze, evaluate, lex, parse,
};

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
fn public_compiler_accessors_preserve_checked_structure() {
    assert_eq!(MAX_TOKENS_PER_SOURCE, 262_144);
    assert_eq!(MAX_LEXICAL_DIAGNOSTICS_PER_SOURCE, 100);

    let text = concat!(
        "edition 2026; module values {\n",
        "  spec byte() -> Word[8] { 8 }\n",
        "  impl empty() {}\n",
        "}\n",
    );
    let mut sources = SourceMap::new();
    let id = sources.add("public-api.or", text).unwrap();
    let source = sources.get(id).unwrap();
    let lexed = lex(source, Edition::E2026);
    let parsed = parse(source, &lexed);
    assert_eq!(parsed.diagnostics(), []);
    let ast = parsed.ast().unwrap();
    assert_eq!(parsed.clone().into_ast().as_ref(), Some(ast));

    assert_eq!(source.slice(ast.span()), Some(text.trim_end()));
    assert_eq!(ast.edition().edition(), Edition::E2026);
    assert_eq!(ast.edition().edition().as_str(), "2026");
    assert_eq!(source.slice(ast.edition().span()), Some("edition 2026;"));
    assert_eq!(source.slice(ast.edition().value_span()), Some("2026"));
    assert_eq!(ast.module().name().text(), "values");
    assert_eq!(source.slice(ast.module().name().span()), Some("values"));
    assert!(
        source
            .slice(ast.module().span())
            .unwrap()
            .starts_with("module")
    );

    let functions = ast.module().functions();
    assert_eq!(functions.len(), 2);
    assert_eq!(functions[0].kind(), FunctionKind::Spec);
    assert_eq!(functions[0].kind().as_str(), "spec");
    assert_eq!(functions[0].name().text(), "byte");
    assert_eq!(source.slice(functions[0].name().span()), Some("byte"));
    assert!(
        source
            .slice(functions[0].span())
            .unwrap()
            .starts_with("spec")
    );
    let FunctionBody::TypedLiteral(body) = functions[0].body() else {
        panic!("expected the public typed-literal body");
    };
    assert!(source.slice(body.span()).unwrap().starts_with("->"));
    assert_eq!(source.slice(body.result_type().span()), Some("Word[8]"));
    assert_eq!(body.result_type().name().text(), "Word");
    assert_eq!(source.slice(body.result_type().name().span()), Some("Word"));
    assert_eq!(
        source.slice(body.result_type().width_span().unwrap()),
        Some("8")
    );
    assert_eq!(source.slice(body.literal().span()), Some("8"));
    assert_eq!(source.slice(body.literal().magnitude_span()), Some("8"));
    assert!(!body.literal().is_negative());
    assert_eq!(functions[1].kind(), FunctionKind::Impl);
    assert_eq!(functions[1].body(), &FunctionBody::Empty);

    let analyzed = analyze(source, ast);
    assert_eq!(analyzed.diagnostics(), []);
    let core = analyzed.core().unwrap();
    assert_eq!(analyzed.clone().into_core().as_ref(), Some(core));
    assert_eq!(source.slice(core.span()), source.slice(ast.module().span()));
    assert_eq!(core.name(), "values");
    assert_eq!(core.functions().len(), 1);
    let function = &core.functions()[0];
    assert_eq!(function.id().index(), 0);
    assert_eq!(
        source.slice(function.span()),
        source.slice(functions[0].span())
    );
    assert_eq!(function.name(), "byte");
    assert_eq!(source.slice(function.name_span()), Some("byte"));
    assert_eq!(function.result_type(), CoreType::Word8);
    assert_eq!(function.result_type().as_str(), "Word[8]");
    assert_eq!(function.value(), &CoreValue::Word8(8));

    let diagnostic = Diagnostic::error(
        DiagnosticCode::UnsupportedType,
        "unsupported test type",
        function.name_span(),
    )
    .with_label("primary label")
    .with_secondary_span(function.span(), "secondary label")
    .with_note("first note");
    assert_eq!(diagnostic.severity(), Severity::Error);
    assert_eq!(diagnostic.severity().as_str(), "error");
    assert_eq!(diagnostic.severity().to_string(), "error");
    assert_eq!(diagnostic.code(), DiagnosticCode::UnsupportedType);
    assert_eq!(diagnostic.code().as_str(), "ORC0203");
    assert_eq!(diagnostic.code().to_string(), "ORC0203");
    assert_eq!(diagnostic.message(), "unsupported test type");
    assert_eq!(diagnostic.primary_span(), function.name_span());
    assert_eq!(diagnostic.label(), "primary label");
    assert_eq!(diagnostic.secondary_spans().len(), 1);
    assert_eq!(diagnostic.secondary_spans()[0].span(), function.span());
    assert_eq!(diagnostic.secondary_spans()[0].label(), "secondary label");
    assert_eq!(diagnostic.notes(), ["first note"]);

    let evaluated = evaluate(core);
    assert_eq!(evaluated.diagnostics(), []);
    assert!(!evaluated.has_errors());
    let values = evaluated.values().unwrap();
    assert_eq!(values.len(), 1);
    let value = &values[0];
    assert_eq!(value.id().index(), 0);
    assert_eq!(value.module(), "values");
    assert_eq!(value.name(), "byte");
    assert_eq!(value.result_type(), CoreType::Word8);
    assert_eq!(value.value(), &CoreValue::Word8(8));
    assert_eq!(value.to_string(), "values::byte: Word[8] = 0x08");
    assert_eq!(evaluated.clone().into_values().as_deref(), Some(values));
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
    for (context, source) in [
        (
            "error after a valid declaration",
            concat!(
                "edition 2026; module demo {\n",
                "  spec valid() -> Int { 42 }\n",
                "  spec invalid() -> Word[8] { 256 }\n",
                "}\n",
            ),
        ),
        (
            "error before a valid declaration",
            concat!(
                "edition 2026; module demo {\n",
                "  spec invalid() -> Word[8] { 256 }\n",
                "  spec valid() -> Int { 42 }\n",
                "}\n",
            ),
        ),
        (
            "error between valid declarations",
            concat!(
                "edition 2026; module demo {\n",
                "  spec first() -> Int { 1 }\n",
                "  spec invalid() -> Word[8] { 256 }\n",
                "  spec last() -> Int { 3 }\n",
                "}\n",
            ),
        ),
    ] {
        let first = run_with_stdin(&["eval", "-"], source.as_bytes());
        let second = run_with_stdin(&["eval", "-"], source.as_bytes());

        assert_eq!(first.status.code(), Some(1), "{context}");
        assert_eq!(first.status.code(), second.status.code(), "{context}");
        assert_eq!(first.stdout, b"", "{context}");
        assert_eq!(first.stdout, second.stdout, "{context}");
        assert_eq!(first.stderr, second.stderr, "{context}");
        let stderr = String::from_utf8(first.stderr).unwrap();
        assert_eq!(stderr.matches("error[ORC0207]").count(), 1, "{context}");
        assert!(
            stderr.contains("outside the range of `Word[8]`"),
            "{context}: {stderr}"
        );
    }
}

#[test]
fn evaluation_emits_no_partial_values_after_a_parser_error() {
    let source = concat!(
        "edition 2026; module demo {\n",
        "  spec first() -> Int { 1 }\n",
        "  impl invalid() -> Int { 2 }\n",
        "  spec last() -> Int { 3 }\n",
        "}\n",
    );
    let first = run_with_stdin(&["eval", "-"], source.as_bytes());
    let second = run_with_stdin(&["eval", "-"], source.as_bytes());

    assert_eq!(first.status.code(), Some(1));
    assert_eq!(first.status.code(), second.status.code());
    assert_eq!(first.stdout, b"");
    assert_eq!(first.stdout, second.stdout);
    assert_eq!(first.stderr, second.stderr);
    let stderr = String::from_utf8(first.stderr).unwrap();
    assert_eq!(stderr.matches("error[ORC0101]").count(), 1, "{stderr}");
    assert!(
        stderr.contains("typed literal bodies are allowed only on `spec` functions"),
        "{stderr}"
    );
    assert!(!stderr.contains("error[ORC02"), "{stderr}");
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
fn option_marker_addresses_a_dash_prefixed_source_path() {
    let process_id = std::process::id();
    let directory = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("orangec-option-marker-{process_id}"));
    let path = directory.join("--generated.or");
    let _ = fs::remove_dir_all(&directory);
    fs::create_dir(&directory).unwrap();
    fs::write(&path, b"edition 2026; module generated {}\n").unwrap();

    let rejected = orangec()
        .current_dir(&directory)
        .args(["check", "--generated.or"])
        .output()
        .unwrap();
    let accepted = orangec()
        .current_dir(&directory)
        .args(["check", "--", "--generated.or"])
        .output()
        .unwrap();
    fs::remove_dir_all(&directory).unwrap();

    assert_eq!(rejected.status.code(), Some(2));
    assert_eq!(rejected.stdout, b"");
    assert!(
        String::from_utf8(rejected.stderr)
            .unwrap()
            .contains("unknown option `--generated.or`")
    );
    assert_eq!(accepted.status.code(), Some(0));
    assert_eq!(accepted.stdout, b"");
    assert_eq!(accepted.stderr, b"");
}

#[cfg(unix)]
#[test]
fn option_marker_preserves_a_non_utf8_dash_prefixed_path() {
    use std::os::unix::ffi::OsStringExt as _;

    let process_id = std::process::id();
    let directory = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("orangec-raw-option-marker-{process_id}"));
    let file_name = std::ffi::OsString::from_vec(b"-\x80.or".to_vec());
    let path = directory.join(&file_name);
    let _ = fs::remove_dir_all(&directory);
    fs::create_dir(&directory).unwrap();
    fs::write(&path, b"edition 2026; module raw_path {}\n").unwrap();

    let rejected = orangec()
        .current_dir(&directory)
        .args([std::ffi::OsString::from("check"), file_name.clone()])
        .output()
        .unwrap();
    let accepted = orangec()
        .current_dir(&directory)
        .args([
            std::ffi::OsString::from("check"),
            std::ffi::OsString::from("--"),
            file_name,
        ])
        .output()
        .unwrap();
    fs::remove_dir_all(&directory).unwrap();

    assert_eq!(rejected.status.code(), Some(2));
    assert_eq!(rejected.stdout, b"");
    assert!(
        String::from_utf8(rejected.stderr)
            .unwrap()
            .contains("unknown option `-\\x80.or`")
    );
    assert_eq!(accepted.status.code(), Some(0));
    assert_eq!(accepted.stdout, b"");
    assert_eq!(accepted.stderr, b"");
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

    let run = || {
        orangec()
            .arg("check")
            .arg(&invalid)
            .arg(&missing)
            .output()
            .unwrap()
    };
    let first = run();
    let second = run();
    fs::remove_file(&invalid).unwrap();
    let _ = fs::remove_file(&missing);

    assert_eq!(first.status.code(), Some(1));
    assert_eq!(first.status.code(), second.status.code());
    assert_eq!(first.stdout, b"");
    assert_eq!(first.stdout, second.stdout);
    assert_eq!(first.stderr, second.stderr);
    let stderr = String::from_utf8(first.stderr).unwrap();
    assert!(!stderr.starts_with('\n'), "{stderr:?}");
    assert!(stderr.ends_with('\n'), "{stderr:?}");
    assert!(!stderr.ends_with("\n\n"), "{stderr:?}");
    let groups: Vec<_> = stderr.trim_end_matches('\n').split("\n\n").collect();
    assert_eq!(groups.len(), 2, "{stderr}");
    assert!(groups[0].starts_with("error[ORC0001]"), "{stderr}");
    assert!(groups[1].starts_with("error[ORC1001]"), "{stderr}");
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
fn reports_cross_phase_file_errors_in_argument_order_repeatably() {
    let process_id = std::process::id();
    let directory = PathBuf::from(env!("CARGO_TARGET_TMPDIR"));
    let semantic = directory.join(format!("orangec-cross-phase-semantic-{process_id}.or"));
    let valid = directory.join(format!("orangec-cross-phase-valid-{process_id}.or"));
    let lexical = directory.join(format!("orangec-cross-phase-lexical-{process_id}.or"));
    let parser = directory.join(format!("orangec-cross-phase-parser-{process_id}.or"));
    for path in [&semantic, &valid, &lexical, &parser] {
        let _ = fs::remove_file(path);
    }
    fs::write(
        &semantic,
        b"edition 2026; module semantic { spec bad() -> Word[8] { 256 } }\n",
    )
    .unwrap();
    fs::write(
        &valid,
        b"edition 2026; module valid { spec answer() -> Int { 42 } }\n",
    )
    .unwrap();
    fs::write(&lexical, b"@").unwrap();
    fs::write(&parser, b"edition 2026\nmodule parser {}\n").unwrap();

    let run = || {
        orangec()
            .arg("check")
            .arg(&semantic)
            .arg(&valid)
            .arg(&lexical)
            .arg(&parser)
            .output()
            .unwrap()
    };
    let first = run();
    let second = run();
    for path in [&semantic, &valid, &lexical, &parser] {
        fs::remove_file(path).unwrap();
    }

    assert_eq!(first.status.code(), Some(1));
    assert_eq!(first.status.code(), second.status.code());
    assert_eq!(first.stdout, b"");
    assert_eq!(first.stdout, second.stdout);
    assert_eq!(first.stderr, second.stderr);
    let stderr = String::from_utf8(first.stderr).unwrap();
    let codes: Vec<_> = stderr
        .lines()
        .filter_map(|line| {
            line.strip_prefix("error[")
                .and_then(|suffix| suffix.split_once(']'))
                .map(|(code, _)| code)
        })
        .collect();
    assert_eq!(codes, ["ORC0207", "ORC0001", "ORC0101"], "{stderr}");
}

#[test]
fn reports_file_stdin_file_errors_in_argument_order_repeatably() {
    let process_id = std::process::id();
    let directory = PathBuf::from(env!("CARGO_TARGET_TMPDIR"));
    let semantic = directory.join(format!("orangec-stream-order-semantic-{process_id}.or"));
    let lexical = directory.join(format!("orangec-stream-order-lexical-{process_id}.or"));
    for path in [&semantic, &lexical] {
        let _ = fs::remove_file(path);
    }
    fs::write(
        &semantic,
        b"edition 2026; module semantic { spec bad() -> Word[8] { 256 } }\n",
    )
    .unwrap();
    fs::write(&lexical, b"@").unwrap();
    let stdin_source = b"edition 2026\nmodule parser {}\n";

    let run = || {
        let mut child = orangec()
            .arg("check")
            .arg(&semantic)
            .arg("-")
            .arg(&lexical)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        child.stdin.take().unwrap().write_all(stdin_source).unwrap();
        child.wait_with_output().unwrap()
    };
    let first = run();
    let second = run();
    for path in [&semantic, &lexical] {
        fs::remove_file(path).unwrap();
    }

    assert_eq!(first.status.code(), Some(1));
    assert_eq!(first.status.code(), second.status.code());
    assert_eq!(first.stdout, b"");
    assert_eq!(first.stdout, second.stdout);
    assert_eq!(first.stderr, second.stderr);
    let stderr = String::from_utf8(first.stderr).unwrap();
    assert!(!stderr.starts_with('\n'), "{stderr:?}");
    assert!(stderr.ends_with('\n'), "{stderr:?}");
    assert!(!stderr.ends_with("\n\n"), "{stderr:?}");
    let groups: Vec<_> = stderr.trim_end_matches('\n').split("\n\n").collect();
    assert_eq!(groups.len(), 3, "{stderr}");
    assert!(groups[0].starts_with("error[ORC0207]"), "{stderr}");
    assert!(
        groups[0].contains(&semantic.to_string_lossy()[..]),
        "{stderr}"
    );
    assert!(groups[1].starts_with("error[ORC0101]"), "{stderr}");
    assert!(groups[1].contains(" --> <stdin>:2:1\n"), "{stderr}");
    assert!(groups[2].starts_with("error[ORC0001]"), "{stderr}");
    assert!(
        groups[2].contains(&lexical.to_string_lossy()[..]),
        "{stderr}"
    );
}

#[test]
fn rejects_duplicate_standard_input_once_then_processes_later_input_repeatably() {
    let source = b"edition 2026; module stream { spec ok() -> Int { 1 } }\n";
    let run = || {
        let mut child = orangec()
            .arg("check")
            .arg("-")
            .arg("-")
            .arg(".")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        child.stdin.take().unwrap().write_all(source).unwrap();
        child.wait_with_output().unwrap()
    };
    let first = run();
    let second = run();

    assert_eq!(first.status.code(), Some(1));
    assert_eq!(first.status.code(), second.status.code());
    assert_eq!(first.stdout, b"");
    assert_eq!(first.stdout, second.stdout);
    assert_eq!(first.stderr, second.stderr);
    assert_eq!(
        first.stderr,
        concat!(
            "error[ORC1004]: standard input was named more than once\n",
            "  = note: use `-` at most once per invocation\n",
            "\n",
            "error[ORC1001]: could not read source file `.`\n",
            "  = note: path does not name a regular file\n",
        )
        .as_bytes()
    );
}

#[test]
fn accepts_exactly_256_source_inputs_repeatably() {
    let fixture = fixture();
    let run = || {
        let mut command = orangec();
        command.arg("check");
        for _ in 0..256 {
            command.arg(&fixture);
        }
        command.output().unwrap()
    };
    let first = run();
    let second = run();

    assert_eq!(first.status.code(), Some(0));
    assert_eq!(first.status.code(), second.status.code());
    assert_eq!(first.stdout, b"");
    assert_eq!(first.stdout, second.stdout);
    assert_eq!(first.stderr, b"");
    assert_eq!(first.stderr, second.stderr);
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
fn closed_operating_system_output_pipe_is_a_quiet_failure() {
    let source = b"edition 2026; module values { spec answer() -> Int { 42 } }\n";
    let mut child = orangec()
        .arg("lex")
        .arg("-")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    // Close the only reader before releasing the complete input. The child is
    // still blocked in its bounded input read, so its first output attempt must
    // observe the real operating-system broken-pipe boundary.
    drop(child.stdout.take().unwrap());
    child.stdin.take().unwrap().write_all(source).unwrap();
    let output = child.wait_with_output().unwrap();

    assert_eq!(output.status.code(), Some(1));
    assert_eq!(output.stdout, b"");
    assert_eq!(output.stderr, b"");
}

#[test]
fn closed_operating_system_error_pipe_is_a_failure() {
    let mut child = orangec()
        .arg("check")
        .arg("-")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    // Keep the child blocked on input until its only diagnostic reader has
    // closed, making the first error emission cross the real broken pipe.
    drop(child.stderr.take().unwrap());
    child.stdin.take().unwrap().write_all(b"@").unwrap();
    let output = child.wait_with_output().unwrap();

    assert_eq!(output.status.code(), Some(1));
    assert_eq!(output.stdout, b"");
    assert_eq!(output.stderr, b"");
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
fn lexes_multiple_sources_with_exact_framing_and_error_order() {
    let process_id = std::process::id();
    let directory = PathBuf::from(env!("CARGO_TARGET_TMPDIR"));
    let first = directory.join(format!("orangec-multi-lex-first-{process_id}.or"));
    let invalid = directory.join(format!("orangec-multi-lex-invalid-{process_id}.or"));
    let last = directory.join(format!("orangec-multi-lex-last-{process_id}.or"));
    for path in [&first, &invalid, &last] {
        let _ = fs::remove_file(path);
    }
    fs::write(&first, b"a\n").unwrap();
    fs::write(&invalid, b"@").unwrap();
    fs::write(&last, b"0").unwrap();

    let run = || {
        orangec()
            .arg("lex")
            .arg(&first)
            .arg(&invalid)
            .arg(&last)
            .output()
            .unwrap()
    };
    let first_output = run();
    let second_output = run();
    for path in [&first, &invalid, &last] {
        fs::remove_file(path).unwrap();
    }

    assert_eq!(first_output.status.code(), Some(1));
    assert_eq!(first_output.status.code(), second_output.status.code());
    assert_eq!(first_output.stdout, second_output.stdout);
    assert_eq!(first_output.stderr, second_output.stderr);
    assert_eq!(
        String::from_utf8(first_output.stdout).unwrap(),
        format!(
            concat!(
                "== {} ==\n",
                "0..1\tIDENTIFIER\t\"a\"\n",
                "2..2\tEOF\t\"\"\n",
                "\n",
                "== {} ==\n",
                "1..1\tEOF\t\"\"\n",
                "\n",
                "== {} ==\n",
                "0..1\tINTEGER\t\"0\"\n",
                "1..1\tEOF\t\"\"\n",
            ),
            first.display(),
            invalid.display(),
            last.display(),
        )
    );
    let stderr = String::from_utf8(first_output.stderr).unwrap();
    assert_eq!(stderr.matches("error[ORC0001]").count(), 1, "{stderr}");
    assert!(
        stderr.contains(&format!(" --> {}:1:1", invalid.display())),
        "{stderr}"
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
            "1 | module caf\\u{e9} {}\n",
            "  |           ^^^^^^ character is not part of Orange 2026\n",
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
    for (context, source) in [
        (
            "fault before grammar",
            b"@ edition 2026; module { spec broken( {}\n".as_slice(),
        ),
        (
            "fault after a valid typed declaration",
            concat!(
                "edition 2026; module demo {\n",
                "  spec valid() -> Int { 42 }\n",
                "  @\n",
                "  spec later() -> Int { 7 }\n",
                "}\n",
            )
            .as_bytes(),
        ),
    ] {
        let first_check = run_with_stdin(&["check", "-"], source);
        let second_check = run_with_stdin(&["check", "-"], source);
        let first_eval = run_with_stdin(&["eval", "-"], source);
        let second_eval = run_with_stdin(&["eval", "-"], source);

        for (command, first, second) in [
            ("check", &first_check, &second_check),
            ("eval", &first_eval, &second_eval),
        ] {
            assert_eq!(first.status.code(), Some(1), "{context} {command} status");
            assert_eq!(first.stdout, b"", "{context} {command} stdout");
            assert_eq!(
                first.status.code(),
                second.status.code(),
                "{context} {command} status drift"
            );
            assert_eq!(
                first.stdout, second.stdout,
                "{context} {command} stdout drift"
            );
            assert_eq!(
                first.stderr, second.stderr,
                "{context} {command} stderr drift"
            );
            let stderr = String::from_utf8(first.stderr.clone()).unwrap();
            assert!(stderr.contains("error[ORC0001]"), "{context}: {stderr}");
            assert!(!stderr.contains("error[ORC01"), "{context}: {stderr}");
        }
        assert_eq!(
            first_check.stderr, first_eval.stderr,
            "{context} check/eval drift"
        );
    }
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
fn accepts_exact_source_limit_from_standard_input() {
    let mut input = b"edition 2026; module boundary { spec answer() -> Int { 42 } }\n".to_vec();
    input.resize(MAX_SOURCE_BYTES, b' ');

    let output = run_with_stdin(&["check", "-"], &input);

    assert_eq!(input.len(), MAX_SOURCE_BYTES);
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(output.stdout, b"");
    assert_eq!(output.stderr, b"");
}

#[test]
fn rejects_oversized_standard_input_with_a_stable_diagnostic() {
    let input = vec![b' '; MAX_SOURCE_BYTES + 1];
    for command in ["check", "eval"] {
        let output = run_with_stdin(&[command, "-"], &input);

        assert_eq!(output.status.code(), Some(1), "{command} status");
        assert_eq!(output.stdout, b"", "{command} stdout");
        assert_eq!(
            String::from_utf8(output.stderr).unwrap(),
            concat!(
                "error[ORC1003]: source file `<stdin>` exceeds the 16777216-byte input limit\n",
                "  = note: the pre-alpha compiler accepts at most 16 MiB per source\n",
            ),
            "{command} stderr"
        );
    }
}

#[test]
fn usage_errors_have_a_distinct_exit_status() {
    let help_first = orangec().arg("--help").output().unwrap();
    let help_second = orangec().arg("-h").output().unwrap();
    assert_eq!(help_first.status.code(), Some(0));
    assert_eq!(help_first.status.code(), help_second.status.code());
    assert_eq!(help_first.stderr, b"");
    assert_eq!(help_first.stderr, help_second.stderr);
    assert_eq!(help_first.stdout, help_second.stdout);
    let help = String::from_utf8(help_first.stdout).unwrap();
    assert_eq!(
        help,
        concat!(
            "Usage: orangec [OPTIONS] <check|eval|lex> <FILE>...\n",
            "\n",
            "Commands:\n",
            "  check    Perform lexical, syntactic, and semantic validation\n",
            "  eval     Reference-evaluate one source after complete validation\n",
            "  lex      Print the deterministic token stream\n",
            "\n",
            "Options:\n",
            "      --edition <YEAR>  Select the Orange edition [default: 2026; at most once]\n",
            "      --                End option parsing\n",
            "  -h, --help            Print help\n",
            "  -V, --version         Print version\n",
            "\n",
            "Use `-` as a file name to read UTF-8 source from standard input.\n",
        )
    );

    let version_first = orangec().arg("--version").output().unwrap();
    let version_second = orangec().arg("-V").output().unwrap();
    assert_eq!(version_first.status.code(), Some(0));
    assert_eq!(version_first.status.code(), version_second.status.code());
    assert_eq!(version_first.stderr, b"");
    assert_eq!(version_first.stderr, version_second.stderr);
    assert_eq!(version_first.stdout, version_second.stdout);
    assert_eq!(
        String::from_utf8(version_first.stdout).unwrap(),
        format!(
            "orangec {} (Orange edition 2026)\n",
            env!("CARGO_PKG_VERSION")
        )
    );

    let run_usage_error = || orangec().arg("compile").arg("file.or").output().unwrap();
    let first = run_usage_error();
    let second = run_usage_error();

    assert_eq!(first.status.code(), Some(2));
    assert_eq!(first.status.code(), second.status.code());
    assert_eq!(first.stdout, b"");
    assert_eq!(first.stdout, second.stdout);
    assert_eq!(first.stderr, second.stderr);
    assert_eq!(
        String::from_utf8(first.stderr).unwrap(),
        format!("orangec: unknown command `compile`\n\n{help}")
    );

    let repeated_edition = orangec()
        .args(["--edition=2026", "check", "--edition", "2026", "missing.or"])
        .output()
        .unwrap();
    assert_eq!(repeated_edition.status.code(), Some(2));
    assert_eq!(repeated_edition.stdout, b"");
    assert_eq!(
        String::from_utf8(repeated_edition.stderr).unwrap(),
        format!("orangec: option `--edition` may be specified at most once\n\n{help}")
    );
}
