//! Independent CLI contracts for the permanent formatter. Scratch paths are
//! fixed at Cargo build time, so hostile runtime temporary-path variables do
//! not redirect test artifacts or compiler source reads.

use std::ffi::OsStr;
use std::fs;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

use orange_compiler::{Edition, MAX_SOURCE_BYTES, SourceMap, TokenKind, lex};

struct Scratch(PathBuf);

impl Scratch {
    fn new(name: &str) -> Self {
        let path = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
            .join(format!("formatting-{name}-{}", std::process::id()));
        fs::create_dir_all(&path).unwrap();
        Self(path)
    }

    fn write(&self, name: impl AsRef<Path>, bytes: &[u8]) -> PathBuf {
        let path = self.0.join(name);
        fs::write(&path, bytes).unwrap();
        path
    }

    fn run(&self, arguments: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_orangec"))
            .current_dir(&self.0)
            .args(arguments)
            .output()
            .unwrap()
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

fn stdin(arguments: &[&str], bytes: &[u8]) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_orangec"))
        .args(arguments)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    // An oversized or malformed source may cause an early bounded rejection.
    let write = child.stdin.take().unwrap().write_all(bytes);
    if let Err(error) = write {
        assert_eq!(error.kind(), std::io::ErrorKind::BrokenPipe);
    }
    child.wait_with_output().unwrap()
}

fn succeeds(output: &Output) -> &str {
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        output.stderr.is_empty(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    std::str::from_utf8(&output.stdout).unwrap()
}

fn failure(output: &Output, status: i32, expected: &str) {
    assert_eq!(output.status.code(), Some(status));
    assert!(output.stdout.is_empty());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains(expected),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn tokens(text: &str) -> Vec<(TokenKind, String)> {
    let mut sources = SourceMap::new();
    let id = sources.add("tokens.or", text).unwrap();
    let source = sources.get(id).unwrap();
    let lexed = lex(source, Edition::E2026);
    assert!(!lexed.has_errors());
    lexed
        .tokens()
        .iter()
        .map(|token| (token.kind, token.lexeme(source).unwrap().to_owned()))
        .collect()
}

// A comment is anchored by the ordinal of the next non-trivia token. Each
// known comment must retain its complete bytes and token-gap location.
fn comment_anchors(text: &str, comments: &[&str]) -> Vec<usize> {
    let mut sources = SourceMap::new();
    let id = sources.add("comments.or", text).unwrap();
    let source = sources.get(id).unwrap();
    let lexed = lex(source, Edition::E2026);
    comments
        .iter()
        .map(|comment| {
            assert_eq!(text.matches(comment).count(), 1, "{comment}");
            let end = text.find(comment).unwrap() + comment.len();
            lexed
                .tokens()
                .iter()
                .position(|token| token.span.start().bytes() as usize >= end)
                .unwrap()
        })
        .collect()
}

#[test]
fn fmt_emits_exact_canonical_source_and_is_idempotent() {
    let output = stdin(&["fmt", "-"], b"edition 2026;module m{}\r\n");
    let formatted = succeeds(&output);
    assert_eq!(formatted, "edition 2026;\nmodule m {\n}\n");
    let second = stdin(&["fmt", "-"], formatted.as_bytes());
    assert_eq!(output.stdout, second.stdout);
    succeeds(&second);
    let checked = stdin(
        &["--check", "fmt", "--edition=2026", "-"],
        formatted.as_bytes(),
    );
    assert!(succeeds(&checked).is_empty());

    let function = stdin(&["fmt", "-"], b"edition 2026;module m{spec x()->Int{1}}");
    assert_eq!(
        succeeds(&function),
        "edition 2026;\nmodule m {\n  spec x() -> Int {\n    1\n  }\n}\n"
    );
    let again = stdin(&["fmt", "-"], &function.stdout);
    assert_eq!(function.stdout, again.stdout);
    succeeds(&again);
}

#[test]
fn fmt_preserves_token_spellings_comment_bytes_and_gap_anchors() {
    let source = concat!(
        "// header é\r\nedition 2026;module sample{/* module */\n",
        "use missing;type Row=Mod[(1 << 5) - 19]^2;\n",
        "/// function docs\nspec value(x:Word[8])->Word[8]{\n",
        "let a:Word[8]=0x0A; /* outer\n  /* inner */\n end */\n",
        "if x==0 { a } else { x } // branch tail\n}\n",
        "test \"literal // text\"{hex\"0a 0B\"==hex\"0A 0b\"}\n",
        "impl empty(){} } // eof tail"
    );
    let comments = [
        "// header é",
        "/* module */",
        "/// function docs",
        "/* outer\n  /* inner */\n end */",
        "// branch tail",
        "// eof tail",
    ];
    let output = stdin(&["fmt", "-"], source.as_bytes());
    let formatted = succeeds(&output);
    assert_eq!(tokens(source), tokens(formatted));
    assert_eq!(
        comment_anchors(source, &comments),
        comment_anchors(formatted, &comments)
    );
    assert!(formatted.ends_with('\n'));
    let again = stdin(&["fmt", "-"], formatted.as_bytes());
    assert_eq!(output.stdout, again.stdout);
    succeeds(&again);
}

#[test]
fn fmt_check_reports_all_sources_in_order_without_mutating_them() {
    let scratch = Scratch::new("ordered");
    let first = b"edition 2026;module first{}";
    let invalid = b"edition 2026;module invalid{";
    let last = b"edition 2026;module last{}";
    scratch.write("first.or", first);
    scratch.write("invalid.or", invalid);
    scratch.write("last.or", last);
    let output = scratch.run(&["fmt", "--check", "first.or", "invalid.or", "last.or"]);
    failure(&output, 1, "error[ORC0252]");
    let diagnostics = String::from_utf8(output.stderr.clone()).unwrap();
    let a = diagnostics.find("first.or:").unwrap();
    let b = diagnostics.find("invalid.or:").unwrap();
    let c = diagnostics.find("last.or:").unwrap();
    assert!(a < b && b < c);
    assert_eq!(diagnostics.matches("error[ORC0252]").count(), 2);
    assert!(diagnostics.contains("formatting differs here"));
    assert!(diagnostics.contains("run `orangec fmt FILE`"));
    let second = scratch.run(&["fmt", "--check", "first.or", "invalid.or", "last.or"]);
    assert_eq!(output.stdout, second.stdout);
    assert_eq!(output.stderr, second.stderr);
    for (name, original) in [
        ("first.or", first.as_slice()),
        ("invalid.or", invalid.as_slice()),
        ("last.or", last.as_slice()),
    ] {
        assert_eq!(fs::read(scratch.0.join(name)).unwrap(), original);
    }
    let formatted = scratch.run(&["fmt", "first.or"]);
    succeeds(&formatted);
    assert_eq!(fs::read(scratch.0.join("first.or")).unwrap(), first);
    scratch.write("canonical.or", &formatted.stdout);
    let checked = scratch.run(&["fmt", "--check", "canonical.or", "canonical.or"]);
    assert!(succeeds(&checked).is_empty());
}

#[test]
fn fmt_check_reports_a_missing_final_newline_at_eof() {
    let output = stdin(&["fmt", "-"], b"edition 2026;module m{}");
    let formatted = succeeds(&output);
    let without_newline = formatted.strip_suffix('\n').unwrap();
    let checked = stdin(&["fmt", "--check", "-"], without_newline.as_bytes());
    failure(&checked, 1, "error[ORC0252]");
    let line = without_newline.lines().count();
    let column = without_newline.lines().last().unwrap().chars().count() + 1;
    assert!(
        String::from_utf8(checked.stderr)
            .unwrap()
            .contains(&format!("<stdin>:{line}:{column}"))
    );
}

#[test]
fn fmt_invalid_sources_and_read_failures_emit_no_formatted_output() {
    for source in [
        b"@ edition 2026;module m{}".as_slice(),
        b"edition 2026;module m{".as_slice(),
        b"edition 2026;module m{} /* unterminated".as_slice(),
    ] {
        for arguments in [["fmt", "-"].as_slice(), ["fmt", "--check", "-"].as_slice()] {
            let output = stdin(arguments, source);
            failure(&output, 1, "error[ORC");
            assert!(!String::from_utf8_lossy(&output.stderr).contains("ORC0252"));
        }
    }
    let scratch = Scratch::new("unreadable");
    let output = scratch.run(&["fmt", "absent.or"]);
    failure(&output, 1, "error[ORC1001]");
    let output = stdin(&["fmt", "-"], b"edition 2026;module m{}\xff");
    failure(&output, 1, "error[ORC1002]");
}

#[test]
fn fmt_does_not_load_imports_or_analyze_semantics() {
    let source = b"edition 2026;module m{use absent;spec value()->Int{unknown()}}";
    let output = stdin(&["fmt", "-"], source);
    let formatted = succeeds(&output);
    let checked = stdin(&["fmt", "--check", "-"], formatted.as_bytes());
    assert!(succeeds(&checked).is_empty());
    let semantic = stdin(&["check", "-"], formatted.as_bytes());
    failure(&semantic, 1, "error[ORC1001]");
}

#[test]
fn fmt_usage_errors_and_option_marker_keep_the_command_contract() {
    let scratch = Scratch::new("usage");
    for arguments in [
        &["fmt"][..],
        &["fmt", "a.or", "b.or"],
        &["fmt", "--check", "--check", "a.or"],
        &["check", "--check", "a.or"],
        &["fmt", "--write", "a.or"],
        &["fmt", "--steps", "1", "a.or"],
        &["fmt", "--stats", "a.or"],
        &["fmt", "--spec", "x", "a.or"],
        &["fmt", "-o", "out", "a.or"],
        &["fmt", "--edition", "2025", "a.or"],
    ] {
        failure(&scratch.run(arguments), 2, "Usage: orangec");
    }
    scratch.write("--check", b"edition 2026;module m{}");
    let marker = scratch.run(&["fmt", "--", "--check"]);
    succeeds(&marker);
    let marker_check = scratch.run(&["fmt", "--check", "--", "--check"]);
    failure(&marker_check, 1, "error[ORC0252]");
    let duplicate = stdin(&["fmt", "--check", "-", "-"], &marker.stdout);
    failure(&duplicate, 1, "error[ORC1004]");
    let operands: Vec<&str> = std::iter::once("fmt")
        .chain(std::iter::once("--check"))
        .chain(std::iter::repeat_n("a.or", 257))
        .collect();
    failure(&scratch.run(&operands), 2, "at most 256 source inputs");
}

#[test]
fn fmt_uses_compile_time_scratch_despite_runtime_temporary_path_variables() {
    let scratch = Scratch::new("runtime-temp");
    let path = scratch.write("source.or", b"edition 2026;module m{}");
    let output = Command::new(env!("CARGO_BIN_EXE_orangec"))
        .args([OsStr::new("fmt"), path.as_os_str()])
        .env("TMPDIR", scratch.0.join("absent/tmp"))
        .env("TEMP", scratch.0.join("absent/temp"))
        .env("TMP", scratch.0.join("absent/tmp2"))
        .output()
        .unwrap();
    succeeds(&output);
    assert!(!scratch.0.join("absent").exists());
}

#[cfg(target_os = "linux")]
#[test]
fn fmt_preserves_non_utf8_dash_path_identity() {
    use std::os::unix::ffi::OsStringExt as _;
    let scratch = Scratch::new("raw-path");
    let name = std::ffi::OsString::from_vec(b"-\x80.or".to_vec());
    scratch.write(&name, b"edition 2026;module m{}");
    let output = Command::new(env!("CARGO_BIN_EXE_orangec"))
        .current_dir(&scratch.0)
        .args([OsStr::new("fmt"), OsStr::new("--"), &name])
        .output()
        .unwrap();
    succeeds(&output);
    let output = Command::new(env!("CARGO_BIN_EXE_orangec"))
        .current_dir(&scratch.0)
        .args([
            OsStr::new("fmt"),
            OsStr::new("--check"),
            OsStr::new("--"),
            &name,
        ])
        .output()
        .unwrap();
    failure(&output, 1, "error[ORC0252]");
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .contains("-\\x80.or:")
    );
    assert_eq!(
        fs::read(scratch.0.join(&name)).unwrap(),
        b"edition 2026;module m{}"
    );
}

#[test]
fn fmt_source_byte_limits_reject_before_any_output() {
    let scratch = Scratch::new("source-limit");
    let mut file = fs::File::create(scratch.0.join("oversized.or")).unwrap();
    file.write_all(b"edition 2026;module m{}").unwrap();
    file.set_len(MAX_SOURCE_BYTES as u64 + 1).unwrap();
    drop(file);
    failure(&scratch.run(&["fmt", "oversized.or"]), 1, "error[ORC1003]");
    let mut source = b"edition 2026;module m{}".to_vec();
    source.resize(MAX_SOURCE_BYTES + 1, b' ');
    failure(&stdin(&["fmt", "-"], &source), 1, "error[ORC1003]");
}

#[test]
fn fmt_expanded_output_limit_is_atomic() {
    let scratch = Scratch::new("formatted-limit");
    let prefix = b"edition 2026;module m{/*";
    let suffix = b"*/}";
    let mut source = prefix.to_vec();
    source.resize(MAX_SOURCE_BYTES - suffix.len(), b'x');
    source.extend_from_slice(suffix);
    assert_eq!(source.len(), MAX_SOURCE_BYTES);
    let path = scratch.write("boundary.or", &source);
    for arguments in [
        ["fmt", "boundary.or"].as_slice(),
        ["fmt", "--check", "boundary.or"].as_slice(),
    ] {
        let output = scratch.run(arguments);
        failure(&output, 1, "error[ORC0250]");
        assert!(!String::from_utf8_lossy(&output.stderr).contains("ORC0252"));
    }
    assert_eq!(fs::read(path).unwrap(), source);
}
