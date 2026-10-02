//! Independent contracts for syntax-only documentation through the CLI.
//! Scratch roots are fixed when Cargo builds this test; directories and files
//! are created exclusively, without using runtime temporary-path variables.

use std::ffi::OsStr;
use std::fs::{self, OpenOptions};
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

use orange_compiler::MAX_SOURCE_BYTES;

struct Scratch(PathBuf);

impl Scratch {
    fn new(name: &str) -> Self {
        let root = Path::new(env!("CARGO_TARGET_TMPDIR"));
        fs::create_dir_all(root).unwrap();
        let path = root.join(format!("documentation-{name}-{}", std::process::id()));
        fs::create_dir(&path).unwrap();
        Self(path)
    }

    fn write(&self, name: impl AsRef<Path>, bytes: &[u8]) -> PathBuf {
        let path = self.0.join(name);
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .unwrap();
        file.write_all(bytes).unwrap();
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
    if let Err(error) = child.stdin.take().unwrap().write_all(bytes) {
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
    assert!(output.stderr.is_empty());
    std::str::from_utf8(&output.stdout).unwrap()
}

fn failure(output: &Output, status: i32, diagnostic: &str) {
    assert_eq!(output.status.code(), Some(status));
    assert!(output.stdout.is_empty());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains(diagnostic),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

// This decoder is independent of the documentation engine. Source text must
// use only the five ordinary text entities, with no raw element delimiters.
fn source_text(html: &str) -> String {
    let (_, rest) = html.split_once("<pre id=\"source\"><code>").unwrap();
    let (escaped, _) = rest.split_once("</code></pre>").unwrap();
    let mut text = String::new();
    let mut rest = escaped;
    while !rest.is_empty() {
        if rest.starts_with('&') {
            let (entity, suffix) = rest.split_once(';').unwrap();
            text.push(match entity {
                "&amp" => '&',
                "&lt" => '<',
                "&gt" => '>',
                "&quot" => '"',
                "&#39" | "&apos" => '\'',
                _ => panic!("unexpected source text entity {entity}"),
            });
            rest = suffix;
        } else {
            let character = rest.chars().next().unwrap();
            assert!(!matches!(character, '<' | '>'));
            text.push(character);
            rest = &rest[character.len_utf8()..];
        }
    }
    text
}

fn attributes<'a>(html: &'a str, marker: &str) -> Vec<&'a str> {
    html.match_indices(marker)
        .map(|(index, _)| html[index + marker.len()..].split_once('"').unwrap().0)
        .collect()
}

#[test]
fn doc_emits_complete_deterministic_offline_html() {
    let source = b"edition 2026;module empty{}";
    let first = stdin(&["doc", "-"], source);
    let html = succeeds(&first);
    assert_eq!(
        html,
        concat!(
            "<!doctype html>\n<html lang=\"en\">\n<head>\n",
            "<meta charset=\"utf-8\">\n",
            "<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n",
            "<meta http-equiv=\"Content-Security-Policy\" content=\"default-src 'none'; base-uri 'none'; form-action 'none'\">\n",
            "<title>Orange 2026 declaration reference</title>\n</head>\n<body>\n<main>\n",
            "<h1>Orange 2026 declaration reference</h1>\n",
            "<p>This reference describes parsed source syntax. Imports and types are not checked, tests are not run, and no proof or assurance is established.</p>\n",
            "<nav aria-label=\"Declarations\">\n<ul>\n",
            "<li><a href=\"#declaration-1\">module empty</a></li>\n",
            "<li><a href=\"#source\">Complete source</a></li>\n</ul>\n</nav>\n",
            "<h2>Declarations</h2>\n<section id=\"declaration-1\">\n",
            "<h3>module empty</h3>\n<p>Line 1, column 14. Module declaration.</p>\n",
            "<pre><code>module empty</code></pre>\n</section>\n",
            "<h2>Complete source</h2>\n",
            "<p>Source display; not a byte-recovery or proof-identity format. Control characters are shown as visible ASCII escapes.</p>\n",
            "<pre id=\"source\"><code>edition 2026;module empty{}</code></pre>\n",
            "</main>\n</body>\n</html>\n",
        )
    );
    assert!(html.ends_with("</html>\n"));
    assert_eq!(html.matches("<html").count(), 1);
    assert_eq!(html.matches("</html>").count(), 1);
    assert!(html.contains("<meta charset=\"utf-8\">"));
    assert_eq!(source_text(html).as_bytes(), source);
    for active in ["<script", "<iframe", "<object", "<embed", "<link", "<img"] {
        assert!(!html.to_ascii_lowercase().contains(active));
    }
    assert!(
        attributes(html, "href=\"")
            .iter()
            .all(|href| href.starts_with('#'))
    );
    assert!(attributes(html, "src=\"").is_empty());
    let again = stdin(&["--edition=2026", "doc", "-"], source);
    succeeds(&again);
    assert_eq!(first.stdout, again.stdout);
    assert_eq!(first.stderr, again.stderr);
}

#[test]
fn doc_escapes_hostile_source_without_changing_safe_source_text() {
    let source = concat!(
        "// <script src=\"https://invalid.example/x\"> & é\r\n",
        "edition 2026;module hostile{\r\n",
        "/* </code></pre><img src=x onerror='alert(1)'> & \" */\r\n",
        "spec value()->Word[8]^1{\"<script>&'\"}\r\n",
        "test \"<img src=x onerror=alert(1)>\" { unknown() == 0 }\r\n",
        "} // </body></html>"
    );
    let output = stdin(&["doc", "-"], source.as_bytes());
    let html = succeeds(&output);
    assert_eq!(source_text(html), source);
    assert!(html.contains("&lt;script"));
    assert!(html.contains("&lt;/code&gt;&lt;/pre&gt;"));
    assert!(html.contains("&amp;"));
    assert!(html.contains("&quot;"));
    assert!(html.contains("&#39;") || html.contains("&apos;"));
    for active in ["<script", "<img", "<iframe", "<object", "<embed"] {
        assert!(!html.to_ascii_lowercase().contains(active));
    }
    assert_eq!(html.matches("<pre id=\"source\"><code>").count(), 1);
    assert_eq!(html.matches("</body>").count(), 1);
    assert_eq!(html.matches("</html>").count(), 1);
}

#[test]
fn doc_displays_exceptional_controls_without_emitting_active_source_bytes() {
    let source = "edition 2026;module m{/*\0\u{1b}\u{7f}\u{85}\u{202e}\u{2066}*/}";
    let output = stdin(&["doc", "-"], source.as_bytes());
    let html = succeeds(&output);
    let displayed = source_text(html);
    for escape in [
        "\\u{0}",
        "\\u{1b}",
        "\\u{7f}",
        "\\u{85}",
        "\\u{202e}",
        "\\u{2066}",
    ] {
        assert!(displayed.contains(escape));
    }
    for control in ['\0', '\u{1b}', '\u{7f}', '\u{85}', '\u{202e}', '\u{2066}'] {
        assert!(!html.contains(control));
    }
    assert_ne!(displayed, source);
}

#[test]
fn doc_declarations_have_unique_resolved_local_anchors_in_source_order() {
    let source = concat!(
        "edition 2026;module entries{",
        "use absent;",
        "type Row=Word[8]^2;",
        "spec first(x:Row)->Row{x}",
        "spec first()->Int{2}",
        "test \"middle declaration\" { unknown() == 0 }",
        "impl last(){}",
        "}"
    );
    let output = stdin(&["doc", "-"], source.as_bytes());
    let html = succeeds(&output);
    assert_eq!(source_text(html), source);
    let ids = attributes(html, "id=\"");
    assert_eq!(ids.len(), 8); // Seven written declarations and complete source.
    let mut unique = ids.clone();
    unique.sort_unstable();
    unique.dedup();
    assert_eq!(ids.len(), unique.len());
    for href in attributes(html, "href=\"") {
        assert!(href.starts_with('#'));
        assert!(ids.contains(&href.strip_prefix('#').unwrap()));
    }
    let (declarations, _) = html.split_once("<pre id=\"source\"><code>").unwrap();
    assert!(declarations.contains("spec first(x:Row)-&gt;Row"));
    assert!(declarations.contains("spec first()-&gt;Int"));
    assert!(!declarations.contains("unknown()"));
    let mut prior = 0;
    for spelling in ["absent", "Row", "first", "middle declaration", "last"] {
        let index = declarations.find(spelling).unwrap();
        assert!(index > prior);
        prior = index;
    }
}

#[test]
fn doc_uses_parsed_syntax_without_import_loading_or_semantic_claims() {
    let source =
        b"edition 2026;module m{use absent;type Bad=Word[7];spec value()->Unknown{unknown()}}";
    let output = stdin(&["doc", "-"], source);
    let html = succeeds(&output);
    assert_eq!(source_text(html).as_bytes(), source);
    assert!(html.contains("parsed source syntax"));
    assert!(!html.contains("class=\"passed\""));
    assert!(!html.contains("Proof verified"));
    let semantic = stdin(&["check", "-"], source);
    failure(&semantic, 1, "error[ORC1001]");
}

#[test]
fn doc_output_omits_ambient_filenames_and_leaves_sources_unchanged() {
    let scratch = Scratch::new("identity");
    let source = b"edition 2026;module same{spec x()->Int{1}}";
    scratch.write("a.or", source);
    scratch.write("other-name.or", source);
    let a = scratch.run(&["doc", "a.or"]);
    let b = scratch.run(&["doc", "other-name.or"]);
    let input = stdin(&["doc", "-"], source);
    succeeds(&a);
    succeeds(&b);
    succeeds(&input);
    assert_eq!(a.stdout, b.stdout);
    assert_eq!(a.stdout, input.stdout);
    assert_eq!(fs::read(scratch.0.join("a.or")).unwrap(), source);
    assert_eq!(fs::read(scratch.0.join("other-name.or")).unwrap(), source);
}

#[test]
fn doc_input_and_syntax_failures_return_no_html() {
    for source in [
        b"@ edition 2026;module m{}".as_slice(),
        b"edition 2026;module m{".as_slice(),
        b"edition 2026;module m{} /* unterminated".as_slice(),
        b"edition 2027;module m{}".as_slice(),
    ] {
        failure(&stdin(&["doc", "-"], source), 1, "error[ORC");
    }
    let scratch = Scratch::new("invalid-input");
    failure(&scratch.run(&["doc", "absent.or"]), 1, "error[ORC1001]");
    scratch.write("invalid-utf8.or", b"edition 2026;module m{}\xff");
    failure(
        &scratch.run(&["doc", "invalid-utf8.or"]),
        1,
        "error[ORC1002]",
    );
    failure(&stdin(&["doc", "-"], b"\xff"), 1, "error[ORC1002]");
}

#[test]
fn doc_usage_errors_are_rejected_before_source_access() {
    let scratch = Scratch::new("usage");
    for arguments in [
        &["doc"][..],
        &["doc", "a.or", "b.or"],
        &["doc", "-", "-"],
        &["doc", "--check", "absent.or"],
        &["doc", "--stats", "absent.or"],
        &["doc", "--steps=1", "absent.or"],
        &["doc", "--spec", "x", "absent.or"],
        &["doc", "-o", "out", "absent.or"],
        &["doc", "--scheme", "x", "absent.or"],
        &["doc", "--key", "key", "absent.or"],
        &["doc", "--edition", "2025", "absent.or"],
        &["doc", "--write", "absent.or"],
    ] {
        let output = scratch.run(arguments);
        failure(&output, 2, "Usage: orangec");
        assert!(!String::from_utf8_lossy(&output.stderr).contains("ORC1001"));
    }
    let operands: Vec<&str> = std::iter::once("doc")
        .chain(std::iter::repeat_n("absent.or", 257))
        .collect();
    failure(&scratch.run(&operands), 2, "at most 256 source inputs");
    assert!(!scratch.0.join("out").exists());
}

#[test]
fn doc_option_marker_and_runtime_temporary_paths_preserve_file_identity() {
    let scratch = Scratch::new("path-marker");
    let source = b"edition 2026;module m{}";
    let path = scratch.write("--check", source);
    let marker = scratch.run(&["doc", "--", "--check"]);
    assert_eq!(source_text(succeeds(&marker)).as_bytes(), source);
    let output = Command::new(env!("CARGO_BIN_EXE_orangec"))
        .args([OsStr::new("doc"), path.as_os_str()])
        .env("TMPDIR", scratch.0.join("absent/tmp"))
        .env("TEMP", scratch.0.join("absent/temp"))
        .env("TMP", scratch.0.join("absent/tmp2"))
        .output()
        .unwrap();
    succeeds(&output);
    assert_eq!(marker.stdout, output.stdout);
    assert!(!scratch.0.join("absent").exists());
    assert_eq!(fs::read(path).unwrap(), source);
}

#[cfg(target_os = "linux")]
#[test]
fn doc_preserves_non_utf8_dash_paths_in_source_diagnostics() {
    use std::os::unix::ffi::OsStringExt as _;
    let scratch = Scratch::new("raw-path");
    let name = std::ffi::OsString::from_vec(b"-\x80.or".to_vec());
    scratch.write(&name, b"edition 2026;module m{");
    let output = Command::new(env!("CARGO_BIN_EXE_orangec"))
        .current_dir(&scratch.0)
        .args([OsStr::new("doc"), OsStr::new("--"), &name])
        .output()
        .unwrap();
    failure(&output, 1, "error[ORC");
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .contains("-\\x80.or:")
    );
    assert_eq!(
        fs::read(scratch.0.join(name)).unwrap(),
        b"edition 2026;module m{"
    );
}

#[test]
fn doc_source_byte_limits_reject_before_output() {
    let scratch = Scratch::new("source-limit");
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(scratch.0.join("oversized.or"))
        .unwrap();
    file.write_all(b"edition 2026;module m{}").unwrap();
    file.set_len(MAX_SOURCE_BYTES as u64 + 1).unwrap();
    drop(file);
    failure(&scratch.run(&["doc", "oversized.or"]), 1, "error[ORC1003]");
    let mut source = b"edition 2026;module m{}".to_vec();
    source.resize(MAX_SOURCE_BYTES + 1, b' ');
    failure(&stdin(&["doc", "-"], &source), 1, "error[ORC1003]");
}

#[test]
fn doc_escaped_output_limit_rejects_atomically() {
    let scratch = Scratch::new("html-limit");
    let mut source = b"edition 2026;module m{/*".to_vec();
    source.resize(MAX_SOURCE_BYTES / 4, b'&');
    source.extend_from_slice(b"*/}");
    assert!(source.len() < MAX_SOURCE_BYTES);
    let path = scratch.write("expansion.or", &source);
    failure(&scratch.run(&["doc", "expansion.or"]), 1, "error[ORC0260]");
    failure(&stdin(&["doc", "-"], &source), 1, "error[ORC0260]");
    assert_eq!(fs::read(path).unwrap(), source);
}
