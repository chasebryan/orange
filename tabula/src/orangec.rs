//! Running `orangec`, the Orange compiler, through its command-line interface.
//!
//! Tabula never reimplements Orange. It sends the editor's text to
//! `orangec check -`, `orangec eval -`, or `orangec lex -` on standard input
//! and reads back what the compiler printed: the token stream and value lines
//! on standard output, and `ORCxxxx` diagnostics on standard error. Parsing is
//! tolerant: anything it does not recognize is still returned as raw text.

use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use crate::json::Json;

/// How long one compiler run may take before Tabula stops it.
pub const RUN_TIMEOUT: Duration = Duration::from_secs(20);

/// Most bytes kept from each compiler output stream.
pub const MAX_CAPTURE_BYTES: usize = 8 * 1024 * 1024;

/// A compiler command Tabula runs.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Action {
    /// `orangec check -`
    Check,
    /// `orangec eval -`
    Eval,
    /// `orangec lex -`
    Lex,
}

impl Action {
    /// The command-line word.
    #[must_use]
    pub const fn word(self) -> &'static str {
        match self {
            Self::Check => "check",
            Self::Eval => "eval",
            Self::Lex => "lex",
        }
    }
}

/// A located compiler.
#[derive(Clone, Debug)]
pub struct Orangec {
    path: PathBuf,
    version: String,
}

/// The raw result of one compiler run.
#[derive(Clone, Debug, Default)]
pub struct RunOutput {
    /// The exit status, if the process exited normally.
    pub status: Option<i32>,
    /// Standard output (lossily decoded).
    pub stdout: String,
    /// Standard error (lossily decoded).
    pub stderr: String,
    /// Whether Tabula stopped the run at [`RUN_TIMEOUT`].
    pub timed_out: bool,
    /// Whether either stream exceeded [`MAX_CAPTURE_BYTES`].
    pub truncated: bool,
    /// Wall-clock milliseconds.
    pub millis: u64,
}

impl Orangec {
    /// Probes `path` with `--version` and keeps it if it answers like orangec.
    #[must_use]
    pub fn probe(path: &Path) -> Option<Self> {
        let output = Command::new(path)
            .arg("--version")
            .env_clear()
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .output()
            .ok()?;
        let version = String::from_utf8(output.stdout).ok()?;
        let version = version.trim();
        (output.status.success() && version.starts_with("orangec ") && version.len() < 200).then(
            || Self {
                path: path.to_path_buf(),
                version: version.to_owned(),
            },
        )
    }

    /// Finds orangec: an explicit path, then `$ORANGEC`, then a sibling of the
    /// running executable, then the Orange checkout's own build outputs, then
    /// each directory on `$PATH`.
    #[must_use]
    pub fn locate(explicit: Option<&Path>, checkout: Option<&Path>) -> Option<Self> {
        if let Some(path) = explicit {
            return Self::probe(path);
        }
        let name = if cfg!(windows) {
            "orangec.exe"
        } else {
            "orangec"
        };
        let mut candidates: Vec<PathBuf> = Vec::new();
        if let Some(value) = std::env::var_os("ORANGEC") {
            candidates.push(PathBuf::from(value));
        }
        if let Some(parent) = std::env::current_exe()
            .ok()
            .as_deref()
            .and_then(Path::parent)
        {
            candidates.push(parent.join(name));
        }
        if let Some(root) = checkout {
            for profile in ["release", "debug"] {
                candidates.push(
                    root.join("compiler")
                        .join("target")
                        .join(profile)
                        .join(name),
                );
            }
        }
        if let Some(path) = std::env::var_os("PATH") {
            candidates.extend(std::env::split_paths(&path).map(|dir| dir.join(name)));
        }
        candidates
            .into_iter()
            .filter(|candidate| candidate.is_file())
            .find_map(|candidate| Self::probe(&candidate))
    }

    /// The executable path.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// The `--version` line, such as `orangec 0.0.1 (Orange edition 2026)`.
    #[must_use]
    pub fn version(&self) -> &str {
        &self.version
    }

    /// Runs `orangec <command> -` with `source` on standard input.
    ///
    /// # Errors
    ///
    /// Returns an error when the process cannot be started.
    pub fn run(&self, command: Action, source: &str) -> io::Result<RunOutput> {
        let started = Instant::now();
        let mut child = Command::new(&self.path)
            .arg(command.word())
            .arg("-")
            .env_clear()
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        let input = child.stdin.take();
        let owned = source.to_owned();
        let writer = thread::spawn(move || {
            if let Some(mut stdin) = input {
                let _ = stdin.write_all(owned.as_bytes());
            }
        });
        let stdout = capture(child.stdout.take());
        let stderr = capture(child.stderr.take());
        let (status, timed_out) = wait_with_deadline(&mut child, started, RUN_TIMEOUT);
        let _ = writer.join();
        let (stdout, stdout_truncated) = stdout.recv().unwrap_or_default();
        let (stderr, stderr_truncated) = stderr.recv().unwrap_or_default();
        Ok(RunOutput {
            status,
            stdout: String::from_utf8_lossy(&stdout).into_owned(),
            stderr: String::from_utf8_lossy(&stderr).into_owned(),
            timed_out,
            truncated: stdout_truncated || stderr_truncated,
            millis: u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX),
        })
    }
}

fn capture(stream: Option<impl Read + Send + 'static>) -> mpsc::Receiver<(Vec<u8>, bool)> {
    let (sender, receiver) = mpsc::channel();
    thread::spawn(move || {
        let mut kept = Vec::new();
        let mut truncated = false;
        if let Some(mut stream) = stream {
            let mut buffer = [0_u8; 8192];
            loop {
                match stream.read(&mut buffer) {
                    Ok(0) => break,
                    Ok(count) => {
                        let room = MAX_CAPTURE_BYTES.saturating_sub(kept.len());
                        let chunk = buffer.get(..count.min(room)).unwrap_or(&[]);
                        kept.extend_from_slice(chunk);
                        if count > room {
                            truncated = true;
                        }
                    }
                    Err(error) if error.kind() == io::ErrorKind::Interrupted => {}
                    Err(_) => break,
                }
            }
        }
        let _ = sender.send((kept, truncated));
    });
    receiver
}

fn wait_with_deadline(child: &mut Child, started: Instant, limit: Duration) -> (Option<i32>, bool) {
    loop {
        match child.try_wait() {
            Ok(Some(status)) => return (status.code(), false),
            Ok(None) if started.elapsed() >= limit => {
                let _ = child.kill();
                let _ = child.wait();
                return (None, true);
            }
            Ok(None) => thread::sleep(Duration::from_millis(5)),
            Err(_) => {
                let _ = child.kill();
                return (None, false);
            }
        }
    }
}

/// One `ORCxxxx` diagnostic parsed from orangec's standard error.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Diagnostic {
    /// `error` or `warning`.
    pub severity: String,
    /// The stable code, such as `ORC0203`.
    pub code: String,
    /// The headline message.
    pub message: String,
    /// 1-based line, if a location was printed.
    pub line: Option<usize>,
    /// 1-based column in Unicode scalar values, if a location was printed.
    pub column: Option<usize>,
    /// Width of the underline in scalar values, when it can be trusted.
    pub width: Option<usize>,
    /// The label printed under the primary underline.
    pub label: String,
    /// `= note:` lines.
    pub notes: Vec<String>,
    /// Secondary locations as `(line, column, width, label)`.
    pub related: Vec<(usize, usize, Option<usize>, String)>,
    /// The diagnostic exactly as orangec printed it.
    pub raw: String,
}

impl Diagnostic {
    /// The phase that reported this diagnostic, from its code's hundreds
    /// digit: lexical (`ORC00xx`), syntax (`ORC01xx`), semantic (`ORC02xx`),
    /// evaluation (`ORC03xx`), or host input/output (`ORC1xxx`).
    #[must_use]
    pub fn phase(&self) -> &'static str {
        let digits = self.code.strip_prefix("ORC").unwrap_or("");
        match digits.as_bytes() {
            [b'0', b'0', _, _] => "lexical",
            [b'0', b'1', _, _] => "syntax",
            [b'0', b'2', _, _] => "semantic",
            [b'0', b'3', _, _] => "evaluation",
            [b'1', _, _, _] => "host",
            _ => "unknown",
        }
    }

    /// The JSON form sent to the page.
    #[must_use]
    pub fn to_json(&self) -> Json {
        Json::object([
            ("severity", Json::str(self.severity.clone())),
            ("code", Json::str(self.code.clone())),
            ("phase", Json::str(self.phase())),
            ("message", Json::str(self.message.clone())),
            ("line", self.line.map_or(Json::Null, Json::usize)),
            ("column", self.column.map_or(Json::Null, Json::usize)),
            ("width", self.width.map_or(Json::Null, Json::usize)),
            ("label", Json::str(self.label.clone())),
            (
                "notes",
                Json::Array(
                    self.notes
                        .iter()
                        .map(|note| Json::str(note.clone()))
                        .collect(),
                ),
            ),
            (
                "related",
                Json::Array(
                    self.related
                        .iter()
                        .map(|(line, column, width, label)| {
                            Json::object([
                                ("line", Json::usize(*line)),
                                ("column", Json::usize(*column)),
                                ("width", width.map_or(Json::Null, Json::usize)),
                                ("label", Json::str(label.clone())),
                            ])
                        })
                        .collect(),
                ),
            ),
            ("raw", Json::str(self.raw.clone())),
        ])
    }
}

/// Parses orangec's rendered diagnostics. `source` is the text that was
/// checked; it lets the parser trust an underline's width only when the
/// printed excerpt matches the source line exactly.
#[must_use]
pub fn parse_diagnostics(stderr: &str, source: &str) -> Vec<Diagnostic> {
    let source_lines: Vec<&str> = split_source_lines(source);
    let mut diagnostics: Vec<Diagnostic> = Vec::new();
    let mut current: Option<Diagnostic> = None;
    let mut pending_location: Option<(bool, usize, usize)> = None;
    let mut excerpt: Option<String> = None;

    for line in stderr.lines() {
        if let Some(header) = parse_header(line) {
            if let Some(done) = current.take() {
                diagnostics.push(done);
            }
            current = Some(header);
            pending_location = None;
            excerpt = None;
        }
        let Some(diagnostic) = current.as_mut() else {
            continue;
        };
        if !diagnostic.raw.is_empty() {
            diagnostic.raw.push('\n');
        }
        diagnostic.raw.push_str(line);

        let trimmed = line.trim_start();
        if let Some(location) = trimmed.strip_prefix("--> ") {
            if let Some((line_number, column)) = parse_location(location) {
                diagnostic.line = Some(line_number);
                diagnostic.column = Some(column);
                pending_location = Some((true, line_number, column));
            }
        } else if let Some(location) = trimmed.strip_prefix("::: ") {
            if let Some((line_number, column)) = parse_location(location) {
                pending_location = Some((false, line_number, column));
            }
        } else if let Some(note) = trimmed.strip_prefix("= note: ") {
            diagnostic.notes.push(note.to_owned());
        } else if let Some((gutter, text)) = line.split_once(" | ") {
            let gutter = gutter.trim();
            if !gutter.is_empty() && gutter.bytes().all(|byte| byte.is_ascii_digit()) {
                excerpt = Some(text.to_owned());
            } else if gutter.is_empty()
                && let Some((primary, line_number, column)) = pending_location.take()
            {
                let marker = if primary { '^' } else { '-' };
                let (width, label) = parse_underline(text, marker);
                let trusted = excerpt.as_deref().is_some_and(|printed| {
                    source_lines
                        .get(line_number.saturating_sub(1))
                        .is_some_and(|actual| *actual == printed && printed.is_ascii())
                });
                let width = width.filter(|_| trusted);
                if primary {
                    diagnostic.width = width;
                    diagnostic.label = label;
                } else {
                    diagnostic.related.push((line_number, column, width, label));
                }
            }
        } else if line.trim_end() == "|" || line.trim_end().ends_with(" |") {
            let gutter = line.trim_end().trim_end_matches('|').trim();
            if gutter.bytes().all(|byte| byte.is_ascii_digit()) && !gutter.is_empty() {
                excerpt = Some(String::new());
            }
        }
    }
    if let Some(done) = current.take() {
        diagnostics.push(done);
    }
    diagnostics
}

fn parse_header(line: &str) -> Option<Diagnostic> {
    let (severity, rest) = line.split_once('[')?;
    if !matches!(severity, "error" | "warning" | "note" | "help") {
        return None;
    }
    let (code, message) = rest.split_once("]: ")?;
    if !(code.len() == 7
        && code.starts_with("ORC")
        && code.bytes().skip(3).all(|byte| byte.is_ascii_digit()))
    {
        return None;
    }
    Some(Diagnostic {
        severity: severity.to_owned(),
        code: code.to_owned(),
        message: message.to_owned(),
        ..Diagnostic::default()
    })
}

fn parse_location(location: &str) -> Option<(usize, usize)> {
    let mut parts = location.rsplitn(3, ':');
    let column = parts.next()?.trim().parse::<usize>().ok()?;
    let line = parts.next()?.trim().parse::<usize>().ok()?;
    parts.next()?;
    (line > 0 && column > 0).then_some((line, column))
}

fn parse_underline(text: &str, marker: char) -> (Option<usize>, String) {
    let trimmed = text.trim_start_matches(' ');
    let width = trimmed
        .chars()
        .take_while(|character| *character == marker)
        .count();
    let label = trimmed
        .chars()
        .skip(width)
        .collect::<String>()
        .trim()
        .to_owned();
    ((width > 0).then_some(width), label)
}

/// Splits source text at LF, CRLF, and bare CR, as orangec counts lines.
#[must_use]
pub fn split_source_lines(source: &str) -> Vec<&str> {
    let mut lines = Vec::new();
    let mut start = 0_usize;
    let bytes = source.as_bytes();
    let mut index = 0_usize;
    while let Some(byte) = bytes.get(index) {
        match byte {
            b'\n' => {
                lines.push(source.get(start..index).unwrap_or(""));
                index = index.saturating_add(1);
                start = index;
            }
            b'\r' => {
                lines.push(source.get(start..index).unwrap_or(""));
                index = index.saturating_add(1);
                if bytes.get(index) == Some(&b'\n') {
                    index = index.saturating_add(1);
                }
                start = index;
            }
            _ => index = index.saturating_add(1),
        }
    }
    lines.push(source.get(start..).unwrap_or(""));
    lines
}

/// One evaluated value line: `module::name: Type = value`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Value {
    /// The module path.
    pub module: String,
    /// The declaration name.
    pub name: String,
    /// The printed type.
    pub ty: String,
    /// The printed value.
    pub value: String,
}

/// Parses `orangec eval` standard output. Lines that do not match the value
/// form are skipped.
#[must_use]
pub fn parse_values(stdout: &str) -> Vec<Value> {
    stdout
        .lines()
        .filter_map(|line| {
            let (qualified, rest) = line.split_once(": ")?;
            let (module, name) = qualified.rsplit_once("::")?;
            let (ty, value) = rest.split_once(" = ")?;
            Some(Value {
                module: module.to_owned(),
                name: name.to_owned(),
                ty: ty.to_owned(),
                value: value.to_owned(),
            })
        })
        .collect()
}

/// One token from `orangec lex`, with offsets converted to UTF-16 code units
/// so the page can index its own strings directly.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Token {
    /// Start offset in UTF-16 code units.
    pub start: usize,
    /// End offset in UTF-16 code units.
    pub end: usize,
    /// The stable token-kind name, such as `KW_SPEC`.
    pub kind: String,
    /// The exact source text of the token. orangec prints an escaped copy;
    /// since the offsets are exact, the text is taken from the source itself.
    pub spelling: String,
}

/// Parses `orangec lex` standard output (`start..end<TAB>KIND<TAB>"spelling"`)
/// and converts byte offsets in `source` to UTF-16 offsets.
#[must_use]
pub fn parse_tokens(stdout: &str, source: &str) -> Vec<Token> {
    stdout
        .lines()
        .filter_map(|line| {
            let mut fields = line.splitn(3, '\t');
            let span = fields.next()?;
            let kind = fields.next()?;
            let (start, end) = span.split_once("..")?;
            let (start, end) = (start.parse::<usize>().ok()?, end.parse::<usize>().ok()?);
            Some(Token {
                spelling: source.get(start..end)?.to_owned(),
                start: utf16_offset(source, start)?,
                end: utf16_offset(source, end)?,
                kind: kind.to_owned(),
            })
        })
        .collect()
}

/// Converts a UTF-8 byte offset into a UTF-16 code-unit offset, or `None` if
/// the offset is not a character boundary within `source`.
#[must_use]
pub fn utf16_offset(source: &str, byte_offset: usize) -> Option<usize> {
    let prefix = source.get(..byte_offset)?;
    Some(
        prefix
            .chars()
            .map(char::len_utf16)
            .fold(0_usize, usize::saturating_add),
    )
}

#[cfg(test)]
mod tests {
    use super::{
        Diagnostic, parse_diagnostics, parse_tokens, parse_values, split_source_lines, utf16_offset,
    };

    const DUPLICATE_SOURCE: &str = "// dup\nedition 2026;\nmodule demo {\n  spec same_spec() {}\n  spec same_spec() -> Int { 1 }\n}\n";

    const DUPLICATE_STDERR: &str = "error[ORC0201]: duplicate spec function `same_spec`
 --> <stdin>:5:8
  |
5 |   spec same_spec() -> Int { 1 }
  |        ^^^^^^^^^ this declaration repeats a name in the same namespace
 ::: <stdin>:4:8
  |
4 |   spec same_spec() {}
  |        --------- first declaration is here
  = note: `spec` and `impl` use separate declaration namespaces
";

    #[test]
    fn parses_primary_and_secondary_locations() {
        let parsed = parse_diagnostics(DUPLICATE_STDERR, DUPLICATE_SOURCE);
        assert_eq!(parsed.len(), 1);
        let diagnostic = parsed.first().unwrap();
        assert_eq!(diagnostic.severity, "error");
        assert_eq!(diagnostic.code, "ORC0201");
        assert_eq!(diagnostic.phase(), "semantic");
        assert_eq!(diagnostic.message, "duplicate spec function `same_spec`");
        assert_eq!(
            (diagnostic.line, diagnostic.column, diagnostic.width),
            (Some(5), Some(8), Some(9))
        );
        assert_eq!(
            diagnostic.label,
            "this declaration repeats a name in the same namespace"
        );
        assert_eq!(
            diagnostic.notes,
            ["`spec` and `impl` use separate declaration namespaces"]
        );
        assert_eq!(
            diagnostic.related,
            [(4, 8, Some(9), "first declaration is here".to_owned())]
        );
        assert_eq!(diagnostic.raw, DUPLICATE_STDERR.trim_end());
    }

    #[test]
    fn distrusts_widths_from_escaped_or_truncated_excerpts() {
        let source = "edition 2026;\nmodule m { spec é() -> Int { 1 } }\n";
        let stderr = "error[ORC0001]: unexpected character U+00E9
 --> <stdin>:2:17
  |
2 | module m { spec \\u{e9}() -> Int { 1 } }
  |                 ^^^^^^ character is not part of Orange 2026
  = note: identifiers are ASCII in this pre-alpha edition

error[ORC0101]: expected `}` to close the module
 --> <stdin>:3:1
  |
3 |
  | ^ found EOF
  = note: close the module before end of file
";
        let parsed = parse_diagnostics(stderr, source);
        assert_eq!(parsed.len(), 2);
        let first = parsed.first().unwrap();
        assert_eq!(
            (first.line, first.column, first.width),
            (Some(2), Some(17), None)
        );
        assert_eq!(first.phase(), "lexical");
        let second = parsed.get(1).unwrap();
        assert_eq!(
            (second.line, second.column, second.width),
            (Some(3), Some(1), Some(1))
        );
        assert_eq!(second.label, "found EOF");
        assert_eq!(second.phase(), "syntax");
    }

    #[test]
    fn keeps_unrecognized_text_out_of_diagnostics() {
        assert!(parse_diagnostics("thread panicked\n", "").is_empty());
        let host = parse_diagnostics("error[ORC1003]: source exceeds the limit\n", "");
        assert_eq!(host.first().map(Diagnostic::phase), Some("host"));
        assert_eq!(host.first().and_then(|d| d.line), None);
    }

    #[test]
    fn parses_values() {
        let values = parse_values("demo::answer: Int = 42\ndemo::mask: Word[8] = 0xff\nnoise\n");
        assert_eq!(values.len(), 2);
        let mask = values.get(1).unwrap();
        assert_eq!((mask.module.as_str(), mask.name.as_str()), ("demo", "mask"));
        assert_eq!((mask.ty.as_str(), mask.value.as_str()), ("Word[8]", "0xff"));
    }

    #[test]
    fn parses_tokens_with_utf16_offsets() {
        let source = "// é𝔽\nedition 2026;";
        let tokens = parse_tokens(
            "10..17\tKW_EDITION\t\"edition\"\n18..22\tINTEGER\t\"2026\"\nbad line\n",
            source,
        );
        assert_eq!(tokens.len(), 2);
        let edition = tokens.first().unwrap();
        assert_eq!((edition.start, edition.end), (7, 14));
        assert_eq!(edition.kind, "KW_EDITION");
        assert_eq!(edition.spelling, "edition");
        assert!(parse_tokens("1..4\tSTRING\t\"x\"\n", source).is_empty());
        assert_eq!(utf16_offset(source, 4), None);
    }

    #[test]
    fn splits_lines_like_orangec() {
        assert_eq!(split_source_lines("a\nb\r\nc\rd"), ["a", "b", "c", "d"]);
        assert_eq!(split_source_lines("a\n"), ["a", ""]);
    }
}
