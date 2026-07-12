//! `orangec`, the pre-alpha Orange compiler command-line frontend.

use std::env;
use std::ffi::{OsStr, OsString};
use std::fs::File;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use orange_compiler::{
    Edition, Lexed, MAX_SOURCE_BYTES, SourceError, SourceFile, SourceMap, lex, render_diagnostics,
};

const SUCCESS: u8 = 0;
const COMPILATION_ERROR: u8 = 1;
const USAGE_ERROR: u8 = 2;
const MAX_SOURCES_PER_INVOCATION: usize = 256;
const USAGE: &str = "Usage: orangec [OPTIONS] <check|lex> <FILE>...\n\
\n\
Commands:\n\
  check    Perform lexical validation\n\
  lex      Print the deterministic token stream\n\
\n\
Options:\n\
      --edition <YEAR>  Select the Orange edition [default: 2026]\n\
  -h, --help            Print help\n\
  -V, --version         Print version\n\
\n\
Use `-` as a file name to read UTF-8 source from standard input.\n";

fn main() -> ExitCode {
    let arguments = env::args_os().skip(1);
    let mut standard_input = io::stdin().lock();
    let mut standard_output = io::stdout().lock();
    let mut standard_error = io::stderr().lock();
    ExitCode::from(run(
        arguments,
        &mut standard_input,
        &mut standard_output,
        &mut standard_error,
    ))
}

fn run(
    arguments: impl IntoIterator<Item = OsString>,
    standard_input: &mut impl Read,
    standard_output: &mut impl Write,
    standard_error: &mut impl Write,
) -> u8 {
    let action = match parse_arguments(arguments) {
        Ok(action) => action,
        Err(message) => {
            let _ = writeln!(standard_error, "orangec: {message}\n\n{USAGE}");
            return USAGE_ERROR;
        }
    };

    match action {
        Action::Help => {
            if write!(standard_output, "{USAGE}").is_err() {
                return COMPILATION_ERROR;
            }
            SUCCESS
        }
        Action::Version => {
            if writeln!(
                standard_output,
                "orangec {} (Orange edition {})",
                env!("CARGO_PKG_VERSION"),
                Edition::CURRENT
            )
            .is_err()
            {
                return COMPILATION_ERROR;
            }
            SUCCESS
        }
        Action::Compile(options) => {
            compile(&options, standard_input, standard_output, standard_error)
        }
    }
}

fn compile(
    options: &Options,
    standard_input: &mut impl Read,
    standard_output: &mut impl Write,
    standard_error: &mut impl Write,
) -> u8 {
    let mut standard_input_seen = false;
    let mut compilation_failed = false;
    let mut output_failed = false;
    let mut standard_error_available = true;
    let mut error_group_written = false;
    let mut token_output_available = true;
    let mut token_source_written = false;
    let show_headers = options.paths.len() > 1;
    let mut token_output = io::BufWriter::new(standard_output);

    for path in &options.paths {
        if path == Path::new("-") {
            if standard_input_seen {
                compilation_failed = true;
                emit_error_group(
                    standard_error,
                    &mut standard_error_available,
                    &mut error_group_written,
                    &mut output_failed,
                    &render_cli_error(
                        "ORC1004",
                        "standard input was named more than once",
                        "use `-` at most once per invocation",
                    ),
                );
                continue;
            }
            standard_input_seen = true;
        }

        let display_name = stable_source_name(path);
        let bytes = match read_source(path, standard_input) {
            Ok(bytes) => bytes,
            Err(ReadSourceError::Io(error)) => {
                compilation_failed = true;
                emit_error_group(
                    standard_error,
                    &mut standard_error_available,
                    &mut error_group_written,
                    &mut output_failed,
                    &render_cli_error(
                        "ORC1001",
                        &format!("could not read source file `{display_name}`"),
                        io_error_reason(error.kind()),
                    ),
                );
                continue;
            }
            Err(ReadSourceError::NotRegular) => {
                compilation_failed = true;
                emit_error_group(
                    standard_error,
                    &mut standard_error_available,
                    &mut error_group_written,
                    &mut output_failed,
                    &render_cli_error(
                        "ORC1001",
                        &format!("could not read source file `{display_name}`"),
                        "path does not name a regular file",
                    ),
                );
                continue;
            }
            Err(ReadSourceError::TooLarge) => {
                compilation_failed = true;
                emit_error_group(
                    standard_error,
                    &mut standard_error_available,
                    &mut error_group_written,
                    &mut output_failed,
                    &render_cli_error(
                        "ORC1003",
                        &format!(
                            "source file `{display_name}` exceeds the {MAX_SOURCE_BYTES}-byte input limit"
                        ),
                        "the pre-alpha compiler accepts at most 16 MiB per source",
                    ),
                );
                continue;
            }
        };
        let text = match String::from_utf8(bytes) {
            Ok(text) => text,
            Err(error) => {
                compilation_failed = true;
                emit_error_group(
                    standard_error,
                    &mut standard_error_available,
                    &mut error_group_written,
                    &mut output_failed,
                    &render_cli_error(
                        "ORC1002",
                        &format!("source file `{display_name}` is not valid UTF-8"),
                        &format!(
                            "invalid byte sequence begins at byte offset {}",
                            error.utf8_error().valid_up_to()
                        ),
                    ),
                );
                continue;
            }
        };
        let mut sources = SourceMap::new();
        let id = match sources.add(display_name, text) {
            Ok(id) => id,
            Err(error) => {
                compilation_failed = true;
                emit_error_group(
                    standard_error,
                    &mut standard_error_available,
                    &mut error_group_written,
                    &mut output_failed,
                    &source_limit_error(path, error),
                );
                continue;
            }
        };
        let source = sources
            .get(id)
            .expect("a source must be available immediately after insertion");
        let result = lex(source, options.edition);

        if options.command == CompilerCommand::Lex && token_output_available {
            match write_tokens(
                &mut token_output,
                source,
                &result,
                show_headers,
                token_source_written,
            ) {
                Ok(()) => token_source_written = true,
                Err(error) => {
                    token_output_available = false;
                    if error.kind() != io::ErrorKind::BrokenPipe {
                        output_failed = true;
                        emit_error_group(
                            standard_error,
                            &mut standard_error_available,
                            &mut error_group_written,
                            &mut output_failed,
                            "orangec: could not write token output\n",
                        );
                    }
                }
            }
        }

        if result.has_errors() {
            compilation_failed = true;
            emit_error_group(
                standard_error,
                &mut standard_error_available,
                &mut error_group_written,
                &mut output_failed,
                &render_diagnostics(&sources, &result.diagnostics),
            );
        }
    }

    if token_output_available
        && let Err(error) = token_output.flush()
        && error.kind() != io::ErrorKind::BrokenPipe
    {
        output_failed = true;
        emit_error_group(
            standard_error,
            &mut standard_error_available,
            &mut error_group_written,
            &mut output_failed,
            "orangec: could not write token output\n",
        );
    }

    if output_failed || compilation_failed {
        COMPILATION_ERROR
    } else {
        SUCCESS
    }
}

fn read_source(path: &Path, standard_input: &mut impl Read) -> Result<Vec<u8>, ReadSourceError> {
    if path == Path::new("-") {
        read_bounded(standard_input)
    } else {
        let metadata = path.metadata().map_err(ReadSourceError::Io)?;
        if !metadata.is_file() {
            return Err(ReadSourceError::NotRegular);
        }
        if metadata.len() > u64::try_from(MAX_SOURCE_BYTES).unwrap_or(u64::MAX) {
            return Err(ReadSourceError::TooLarge);
        }

        let file = File::open(path).map_err(ReadSourceError::Io)?;
        let metadata = file.metadata().map_err(ReadSourceError::Io)?;
        if !metadata.is_file() {
            return Err(ReadSourceError::NotRegular);
        }
        if metadata.len() > u64::try_from(MAX_SOURCE_BYTES).unwrap_or(u64::MAX) {
            return Err(ReadSourceError::TooLarge);
        }
        read_bounded(file)
    }
}

fn read_bounded(mut reader: impl Read) -> Result<Vec<u8>, ReadSourceError> {
    let read_limit = u64::try_from(MAX_SOURCE_BYTES)
        .unwrap_or(u64::MAX)
        .saturating_add(1);
    let mut bytes = Vec::new();
    reader
        .by_ref()
        .take(read_limit)
        .read_to_end(&mut bytes)
        .map_err(ReadSourceError::Io)?;
    if bytes.len() > MAX_SOURCE_BYTES {
        Err(ReadSourceError::TooLarge)
    } else {
        Ok(bytes)
    }
}

fn write_tokens(
    output: &mut impl Write,
    source: &SourceFile,
    result: &Lexed,
    show_header: bool,
    separate_from_previous: bool,
) -> io::Result<()> {
    if show_header {
        if separate_from_previous {
            output.write_all(b"\n")?;
        }
        writeln!(output, "== {} ==", source.name())?;
    }
    for token in &result.tokens {
        let spelling = token
            .lexeme(source)
            .map(escape_token_spelling)
            .unwrap_or_default();
        writeln!(
            output,
            "{}..{}\t{}\t\"{}\"",
            token.span.start().bytes(),
            token.span.end().bytes(),
            token.kind.name(),
            spelling
        )?;
    }
    Ok(())
}

fn escape_token_spelling(spelling: &str) -> String {
    spelling.chars().flat_map(char::escape_default).collect()
}

fn stable_source_name(path: &Path) -> String {
    if path == Path::new("-") {
        return String::from("<stdin>");
    }
    path.as_os_str()
        .to_string_lossy()
        .chars()
        .flat_map(char::escape_default)
        .collect()
}

fn source_limit_error(path: &Path, error: SourceError) -> String {
    render_cli_error(
        "ORC1005",
        &format!(
            "could not represent source file `{}`",
            stable_source_name(path)
        ),
        &error.to_string(),
    )
}

#[derive(Debug)]
enum ReadSourceError {
    Io(io::Error),
    NotRegular,
    TooLarge,
}

fn io_error_reason(kind: io::ErrorKind) -> &'static str {
    match kind {
        io::ErrorKind::NotFound => "file was not found",
        io::ErrorKind::PermissionDenied => "permission was denied",
        io::ErrorKind::IsADirectory => "path names a directory",
        io::ErrorKind::InvalidData => "the operating system reported invalid data",
        io::ErrorKind::OutOfMemory => "the operating system could not allocate memory",
        _ => "the operating system reported an I/O error",
    }
}

fn render_cli_error(code: &str, message: &str, note: &str) -> String {
    format!("error[{code}]: {message}\n  = note: {note}\n")
}

fn emit_error_group(
    output: &mut impl Write,
    output_available: &mut bool,
    previous_group_written: &mut bool,
    output_failed: &mut bool,
    group: &str,
) {
    if !*output_available || group.is_empty() {
        return;
    }
    let result = if *previous_group_written {
        output
            .write_all(b"\n")
            .and_then(|()| output.write_all(group.as_bytes()))
    } else {
        output.write_all(group.as_bytes())
    };
    if result.is_err() {
        *output_available = false;
        *output_failed = true;
    } else {
        *previous_group_written = true;
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CompilerCommand {
    Check,
    Lex,
}

#[derive(Debug, Eq, PartialEq)]
struct Options {
    command: CompilerCommand,
    edition: Edition,
    paths: Vec<PathBuf>,
}

#[derive(Debug, Eq, PartialEq)]
enum Action {
    Help,
    Version,
    Compile(Options),
}

fn parse_arguments(arguments: impl IntoIterator<Item = OsString>) -> Result<Action, String> {
    let mut arguments = arguments.into_iter();
    let mut command = None;
    let mut edition = Edition::default();
    let mut paths = Vec::new();
    let mut options_enabled = true;

    while let Some(argument) = arguments.next() {
        let utf8 = argument.to_str();
        if options_enabled {
            match utf8 {
                Some("-h" | "--help") => return Ok(Action::Help),
                Some("-V" | "--version") => return Ok(Action::Version),
                Some("--") => {
                    options_enabled = false;
                    continue;
                }
                Some("--edition") => {
                    let value = arguments
                        .next()
                        .ok_or_else(|| String::from("option `--edition` requires a value"))?;
                    edition = parse_edition(&value)?;
                    continue;
                }
                Some(value) if value.starts_with("--edition=") => {
                    edition = value["--edition=".len()..]
                        .parse()
                        .map_err(|error: orange_compiler::ParseEditionError| error.to_string())?;
                    continue;
                }
                Some(value) if value.starts_with('-') && value != "-" => {
                    return Err(format!("unknown option `{value}`"));
                }
                _ => {}
            }
        }

        if command.is_none() {
            command = match utf8 {
                Some("check") => Some(CompilerCommand::Check),
                Some("lex") => Some(CompilerCommand::Lex),
                Some(value) => return Err(format!("unknown command `{value}`")),
                None => return Err(String::from("command is not valid UTF-8")),
            };
        } else {
            if paths.len() >= MAX_SOURCES_PER_INVOCATION {
                return Err(format!(
                    "at most {MAX_SOURCES_PER_INVOCATION} source inputs are accepted per invocation"
                ));
            }
            paths.push(PathBuf::from(argument));
        }
    }

    let command = command.ok_or_else(|| String::from("missing command"))?;
    if paths.is_empty() {
        return Err(format!(
            "command `{}` requires at least one source file",
            match command {
                CompilerCommand::Check => "check",
                CompilerCommand::Lex => "lex",
            }
        ));
    }
    Ok(Action::Compile(Options {
        command,
        edition,
        paths,
    }))
}

fn parse_edition(value: &OsStr) -> Result<Edition, String> {
    value
        .to_str()
        .ok_or_else(|| String::from("edition name is not valid UTF-8"))?
        .parse()
        .map_err(|error: orange_compiler::ParseEditionError| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn os_arguments(values: &[&str]) -> Vec<OsString> {
        values.iter().map(OsString::from).collect()
    }

    #[test]
    fn accepts_options_before_and_after_the_command() {
        assert_eq!(
            parse_arguments(os_arguments(&["--edition", "2026", "lex", "one.or"])),
            Ok(Action::Compile(Options {
                command: CompilerCommand::Lex,
                edition: Edition::E2026,
                paths: vec![PathBuf::from("one.or")],
            }))
        );
        assert_eq!(
            parse_arguments(os_arguments(&["check", "--edition=2026", "one.or"])),
            Ok(Action::Compile(Options {
                command: CompilerCommand::Check,
                edition: Edition::E2026,
                paths: vec![PathBuf::from("one.or")],
            }))
        );
    }

    #[test]
    fn option_marker_allows_dash_prefixed_file_names() {
        assert_eq!(
            parse_arguments(os_arguments(&["check", "--", "--generated.or"]))
                .unwrap()
                .compile_options()
                .paths,
            vec![PathBuf::from("--generated.or")]
        );
    }

    #[test]
    fn reports_missing_inputs_and_unknown_editions() {
        assert_eq!(
            parse_arguments(os_arguments(&["check"])),
            Err(String::from(
                "command `check` requires at least one source file"
            ))
        );
        assert_eq!(
            parse_arguments(os_arguments(&["--edition", "1999", "check", "x.or"])),
            Err(String::from(
                "unsupported Orange edition; supported editions: 2026"
            ))
        );
    }

    #[test]
    fn bounds_source_inputs_per_invocation() {
        let mut at_limit = vec![OsString::from("check")];
        at_limit.extend((0..MAX_SOURCES_PER_INVOCATION).map(|_| OsString::from("source.or")));
        assert_eq!(
            parse_arguments(at_limit)
                .unwrap()
                .compile_options()
                .paths
                .len(),
            MAX_SOURCES_PER_INVOCATION
        );

        let mut over_limit = vec![OsString::from("check")];
        over_limit.extend((0..=MAX_SOURCES_PER_INVOCATION).map(|_| OsString::from("source.or")));
        assert_eq!(
            parse_arguments(over_limit),
            Err(format!(
                "at most {MAX_SOURCES_PER_INVOCATION} source inputs are accepted per invocation"
            ))
        );
    }

    impl Action {
        fn compile_options(self) -> Options {
            match self {
                Self::Compile(options) => options,
                Self::Help | Self::Version => panic!("expected compile action"),
            }
        }
    }
}
