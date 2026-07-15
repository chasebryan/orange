//! `orangec`, the pre-alpha Orange compiler command-line frontend.

use std::borrow::Cow;
use std::env;
use std::ffi::{OsStr, OsString};
use std::fmt::{self, Write as _};
use std::fs::{File, Metadata};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use orange_compiler::{
    Edition, Lexed, MAX_SOURCE_BYTES, RenderedSourceName, SourceError, SourceFile, SourceMap,
    analyze, evaluate, lex, parse, render_diagnostics,
};

const SUCCESS: u8 = 0;
const COMPILATION_ERROR: u8 = 1;
const USAGE_ERROR: u8 = 2;
const MAX_SOURCES_PER_INVOCATION: usize = 256;
const MAX_ARGUMENT_BYTES_PER_INVOCATION: usize = 4 * 1024 * 1024;
const MAX_SOURCE_BYTES_PER_INVOCATION: usize = 64 * 1024 * 1024;
const MAX_STANDARD_OUTPUT_BYTES: usize = 64 * 1024 * 1024;
const MAX_STANDARD_ERROR_BYTES: usize = 64 * 1024 * 1024;
const SOURCE_READ_BUFFER_BYTES: usize = 8 * 1024;
const TOKEN_ESCAPE_BUFFER_BYTES: usize = 4 * 1024;
const USAGE: &str = concat!(
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
);

macro_rules! define_cli_diagnostic_codes {
    ($($variant:ident => $code:literal,)+) => {
        #[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
        enum CliDiagnosticCode {
            $($variant,)+
        }

        impl CliDiagnosticCode {
            const fn as_str(self) -> &'static str {
                match self {
                    $(Self::$variant => $code,)+
                }
            }

            #[cfg(test)]
            const ALL: &'static [Self] = &[$(Self::$variant,)+];
        }
    };
}

define_cli_diagnostic_codes! {
    ReadSource => "ORC1001",
    InvalidUtf8 => "ORC1002",
    SourceTooLarge => "ORC1003",
    DuplicateStandardInput => "ORC1004",
    SourceRepresentation => "ORC1005",
    MissingPhaseArtifact => "ORC1006",
    OutputTooLarge => "ORC1007",
    InvocationSourceTooLarge => "ORC1008",
}

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

struct CountCheckedWriter<W> {
    inner: W,
}

impl<W> CountCheckedWriter<W> {
    const fn new(inner: W) -> Self {
        Self { inner }
    }
}

impl<W: Write> Write for CountCheckedWriter<W> {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        let written = self.inner.write(buffer)?;
        if written > buffer.len() {
            return Err(invalid_data_error());
        }
        Ok(written)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

struct OutputLimitedWriter<W> {
    inner: W,
    remaining: usize,
}

#[derive(Debug)]
struct OutputLimitExceeded;

impl fmt::Display for OutputLimitExceeded {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("standard output limit exceeded")
    }
}

impl std::error::Error for OutputLimitExceeded {}

fn output_limit_error() -> io::Error {
    io::Error::new(io::ErrorKind::FileTooLarge, OutputLimitExceeded)
}

impl<W> OutputLimitedWriter<W> {
    const fn new(inner: W, limit: usize) -> Self {
        Self {
            inner,
            remaining: limit,
        }
    }

    fn into_inner(self) -> W {
        self.inner
    }
}

impl<W: Write> Write for OutputLimitedWriter<W> {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        if buffer.is_empty() {
            return Ok(0);
        }
        let allowed = buffer.len().min(self.remaining);
        if allowed == 0 {
            return Err(output_limit_error());
        }
        let buffer = buffer.get(..allowed).ok_or_else(invalid_data_error)?;
        let written = self.inner.write(buffer)?;
        if written > buffer.len() {
            return Err(invalid_data_error());
        }
        self.remaining = self
            .remaining
            .checked_sub(written)
            .ok_or_else(invalid_data_error)?;
        Ok(written)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

fn invalid_data_error() -> io::Error {
    io::Error::from(io::ErrorKind::InvalidData)
}

fn flush_retry_interrupted(output: &mut impl Write) -> io::Result<()> {
    loop {
        match output.flush() {
            Err(error) if error.kind() == io::ErrorKind::Interrupted => {}
            result => return result,
        }
    }
}

fn run(
    arguments: impl IntoIterator<Item = OsString>,
    standard_input: &mut impl Read,
    standard_output: &mut impl Write,
    standard_error: &mut impl Write,
) -> u8 {
    run_with_standard_error_limit(
        arguments,
        standard_input,
        standard_output,
        standard_error,
        MAX_STANDARD_ERROR_BYTES,
    )
}

fn run_with_standard_error_limit(
    arguments: impl IntoIterator<Item = OsString>,
    standard_input: &mut impl Read,
    standard_output: &mut impl Write,
    standard_error: &mut impl Write,
    standard_error_limit: usize,
) -> u8 {
    let mut standard_output = CountCheckedWriter::new(standard_output);
    let standard_error = CountCheckedWriter::new(standard_error);
    let mut standard_error = OutputLimitedWriter::new(standard_error, standard_error_limit);
    let action = match parse_arguments(arguments) {
        Ok(action) => action,
        Err(message) => {
            let result = write!(standard_error, "orangec: {message}\n\n{USAGE}")
                .and_then(|()| flush_retry_interrupted(&mut standard_error));
            return if result.is_err() {
                COMPILATION_ERROR
            } else {
                USAGE_ERROR
            };
        }
    };

    match action {
        Action::Help => {
            if write!(standard_output, "{USAGE}")
                .and_then(|()| flush_retry_interrupted(&mut standard_output))
                .is_err()
            {
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
            .and_then(|()| flush_retry_interrupted(&mut standard_output))
            .is_err()
            {
                return COMPILATION_ERROR;
            }
            SUCCESS
        }
        Action::Compile(options) => compile(
            &options,
            standard_input,
            &mut standard_output,
            &mut standard_error,
        ),
    }
}

fn compile(
    options: &Options,
    standard_input: &mut impl Read,
    standard_output: &mut impl Write,
    standard_error: &mut impl Write,
) -> u8 {
    compile_with_limits(
        options,
        standard_input,
        standard_output,
        standard_error,
        MAX_SOURCE_BYTES_PER_INVOCATION,
        MAX_STANDARD_OUTPUT_BYTES,
    )
}

fn compile_with_limits(
    options: &Options,
    standard_input: &mut impl Read,
    standard_output: &mut impl Write,
    standard_error: &mut impl Write,
    source_limit: usize,
    output_limit: usize,
) -> u8 {
    let mut standard_input_seen = false;
    let mut compilation_failed = false;
    let mut output_failed = false;
    let mut standard_error_available = true;
    let mut standard_error_flushed = false;
    let mut error_group_written = false;
    let mut standard_output_available = true;
    let mut standard_output_written = false;
    let mut token_source_written = false;
    let mut remaining_source_bytes = source_limit;
    let show_headers = options.paths.len() > 1;
    let buffered_output = io::BufWriter::new(standard_output);
    let mut buffered_output = OutputLimitedWriter::new(buffered_output, output_limit);

    for path in &options.paths {
        // A failed result stream makes status 1 unavoidable and prevents any
        // further diagnostics or command output from being reliable. Avoid
        // spending resources on source operands that can no longer affect the
        // result observed by the caller.
        if output_failed {
            break;
        }

        if path == Path::new("-") {
            if standard_input_seen {
                compilation_failed = true;
                emit_error_group(
                    standard_error,
                    &mut standard_error_available,
                    &mut error_group_written,
                    &mut output_failed,
                    &render_cli_error(
                        CliDiagnosticCode::DuplicateStandardInput,
                        "standard input was named more than once",
                        "use `-` at most once per invocation",
                    ),
                );
                continue;
            }
            standard_input_seen = true;
        }

        let display_name = match stable_source_name(path) {
            Ok(display_name) => display_name,
            Err(error) => {
                compilation_failed = true;
                emit_error_group(
                    standard_error,
                    &mut standard_error_available,
                    &mut error_group_written,
                    &mut output_failed,
                    &source_name_error(error),
                );
                continue;
            }
        };
        let bytes = match read_source(path, standard_input, &mut remaining_source_bytes) {
            Ok(bytes) => bytes,
            Err(error) => {
                compilation_failed = true;
                emit_error_group(
                    standard_error,
                    &mut standard_error_available,
                    &mut error_group_written,
                    &mut output_failed,
                    &render_read_source_error(&display_name, error),
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
                        CliDiagnosticCode::InvalidUtf8,
                        format_args!("source file `{display_name}` is not valid UTF-8"),
                        format_args!(
                            "invalid byte sequence begins at byte offset {}",
                            error.utf8_error().valid_up_to()
                        ),
                    ),
                );
                continue;
            }
        };
        let mut sources = match SourceMap::try_new() {
            Ok(sources) => sources,
            Err(error) => {
                compilation_failed = true;
                emit_error_group(
                    standard_error,
                    &mut standard_error_available,
                    &mut error_group_written,
                    &mut output_failed,
                    &source_limit_error(&display_name, error),
                );
                continue;
            }
        };
        let id = match sources.add_with_rendered_name(display_name, text) {
            Ok(id) => id,
            Err(error) => {
                compilation_failed = true;
                emit_error_group(
                    standard_error,
                    &mut standard_error_available,
                    &mut error_group_written,
                    &mut output_failed,
                    &source_limit_error_without_name(error),
                );
                continue;
            }
        };
        let Some(source) = sources.get(id) else {
            compilation_failed = true;
            emit_error_group(
                standard_error,
                &mut standard_error_available,
                &mut error_group_written,
                &mut output_failed,
                &render_cli_error(
                    CliDiagnosticCode::MissingPhaseArtifact,
                    "source insertion succeeded without a retrievable source",
                    "this is an internal compiler failure",
                ),
            );
            continue;
        };
        let result = lex(source, options.edition);

        if options.command == CompilerCommand::Lex && standard_output_available {
            match write_tokens(
                &mut buffered_output,
                source,
                &result,
                show_headers,
                token_source_written,
            ) {
                Ok(()) => {
                    standard_output_written = true;
                    token_source_written = true;
                }
                Err(error) => {
                    standard_output_available = false;
                    output_failed = true;
                    if let Some(group) = output_failure_group(options.command, &error) {
                        emit_error_group(
                            standard_error,
                            &mut standard_error_available,
                            &mut error_group_written,
                            &mut output_failed,
                            &group,
                        );
                    }
                }
            }
        }

        if result.has_errors() {
            compilation_failed = true;
            let rendered = if result.diagnostics().is_empty() {
                render_cli_error(
                    CliDiagnosticCode::MissingPhaseArtifact,
                    "lexical analysis failed without a diagnostic",
                    "this is an internal compiler resource failure",
                )
            } else {
                render_diagnostics(&sources, result.diagnostics())
            };
            emit_error_group(
                standard_error,
                &mut standard_error_available,
                &mut error_group_written,
                &mut output_failed,
                &rendered,
            );
        } else if matches!(
            options.command,
            CompilerCommand::Check | CompilerCommand::Eval
        ) {
            let parsed = parse(source, &result);
            let ast = match classify_phase_result(parsed.ast(), parsed.diagnostics()) {
                PhaseResult::Complete(ast) => ast,
                PhaseResult::Diagnosed(diagnostics) => {
                    compilation_failed = true;
                    emit_error_group(
                        standard_error,
                        &mut standard_error_available,
                        &mut error_group_written,
                        &mut output_failed,
                        &render_diagnostics(&sources, diagnostics),
                    );
                    continue;
                }
                PhaseResult::Missing => {
                    compilation_failed = true;
                    emit_error_group(
                        standard_error,
                        &mut standard_error_available,
                        &mut error_group_written,
                        &mut output_failed,
                        &render_cli_error(
                            CliDiagnosticCode::MissingPhaseArtifact,
                            "parser returned neither a complete syntax tree nor a diagnostic",
                            "this is an internal compiler or resource failure",
                        ),
                    );
                    continue;
                }
            };
            let analyzed = analyze(source, ast);
            let core = match classify_phase_result(analyzed.core(), analyzed.diagnostics()) {
                PhaseResult::Complete(core) => core,
                PhaseResult::Diagnosed(diagnostics) => {
                    compilation_failed = true;
                    emit_error_group(
                        standard_error,
                        &mut standard_error_available,
                        &mut error_group_written,
                        &mut output_failed,
                        &render_diagnostics(&sources, diagnostics),
                    );
                    continue;
                }
                PhaseResult::Missing => {
                    compilation_failed = true;
                    emit_error_group(
                        standard_error,
                        &mut standard_error_available,
                        &mut error_group_written,
                        &mut output_failed,
                        &render_cli_error(
                            CliDiagnosticCode::MissingPhaseArtifact,
                            "semantic analysis returned neither Typed Reference Core nor a diagnostic",
                            "this is an internal compiler or resource failure",
                        ),
                    );
                    continue;
                }
            };

            if options.command == CompilerCommand::Eval {
                let evaluated = evaluate(core);
                let values = match classify_phase_result(
                    evaluated.values(),
                    evaluated.diagnostics(),
                ) {
                    PhaseResult::Complete(values) => values,
                    PhaseResult::Diagnosed(diagnostics) => {
                        compilation_failed = true;
                        emit_error_group(
                            standard_error,
                            &mut standard_error_available,
                            &mut error_group_written,
                            &mut output_failed,
                            &render_diagnostics(&sources, diagnostics),
                        );
                        continue;
                    }
                    PhaseResult::Missing => {
                        compilation_failed = true;
                        emit_error_group(
                            standard_error,
                            &mut standard_error_available,
                            &mut error_group_written,
                            &mut output_failed,
                            &render_cli_error(
                                CliDiagnosticCode::MissingPhaseArtifact,
                                "reference evaluation returned neither a complete value set nor a diagnostic",
                                "this is an internal compiler or resource failure",
                            ),
                        );
                        continue;
                    }
                };
                // Argument validation guarantees exactly one `eval` source, so
                // no later source can invalidate output after this point.
                if standard_output_available {
                    for value in values {
                        match writeln!(buffered_output, "{value}") {
                            Ok(()) => standard_output_written = true,
                            Err(error) => {
                                standard_output_available = false;
                                output_failed = true;
                                if let Some(group) = output_failure_group(options.command, &error) {
                                    emit_error_group(
                                        standard_error,
                                        &mut standard_error_available,
                                        &mut error_group_written,
                                        &mut output_failed,
                                        &group,
                                    );
                                }
                                break;
                            }
                        }
                    }
                }
            }
        }
    }

    // When diagnostics and buffered token output are both pending, commit the
    // diagnostic stream first. A diagnostic flush failure can then discard
    // token bytes that have not yet escaped the process.
    if standard_error_available
        && error_group_written
        && standard_output_available
        && standard_output_written
    {
        if flush_retry_interrupted(standard_error).is_err() {
            standard_error_available = false;
            output_failed = true;
        } else {
            standard_error_flushed = true;
        }
    }

    if !output_failed
        && standard_output_available
        && standard_output_written
        && let Err(error) = flush_retry_interrupted(&mut buffered_output)
    {
        output_failed = true;
        if let Some(group) = output_failure_group(options.command, &error) {
            standard_error_flushed = false;
            emit_error_group(
                standard_error,
                &mut standard_error_available,
                &mut error_group_written,
                &mut output_failed,
                &group,
            );
        }
    }
    // Do not let `BufWriter`'s best-effort drop path retry retained bytes after
    // an output failure. A successful explicit flush leaves nothing to discard.
    let buffered_output = buffered_output.into_inner();
    let (_standard_output, _unwritten_output) = buffered_output.into_parts();

    if standard_error_available
        && error_group_written
        && !standard_error_flushed
        && flush_retry_interrupted(standard_error).is_err()
    {
        output_failed = true;
    }

    if output_failed || compilation_failed {
        COMPILATION_ERROR
    } else {
        SUCCESS
    }
}

fn output_failure_group(command: CompilerCommand, error: &io::Error) -> Option<Cow<'static, str>> {
    if error
        .get_ref()
        .is_some_and(|cause| cause.is::<OutputLimitExceeded>())
    {
        return Some(Cow::Owned(render_cli_error(
            CliDiagnosticCode::OutputTooLarge,
            format_args!(
                "standard output exceeds the {MAX_STANDARD_OUTPUT_BYTES}-byte invocation limit"
            ),
            "orangec writes at most 64 MiB to standard output per invocation",
        )));
    }
    match error.kind() {
        io::ErrorKind::BrokenPipe => None,
        _ => Some(Cow::Borrowed(if command == CompilerCommand::Eval {
            "orangec: could not write evaluation output\n"
        } else {
            "orangec: could not write token output\n"
        })),
    }
}

fn read_source(
    path: &Path,
    standard_input: &mut impl Read,
    remaining_source_bytes: &mut usize,
) -> Result<Vec<u8>, ReadSourceError> {
    read_source_with_post_read(path, standard_input, remaining_source_bytes, || {})
}

fn read_source_with_post_read(
    path: &Path,
    standard_input: &mut impl Read,
    remaining_source_bytes: &mut usize,
    post_read: impl FnOnce(),
) -> Result<Vec<u8>, ReadSourceError> {
    if path == Path::new("-") {
        read_bounded_for_invocation(standard_input, remaining_source_bytes)
    } else {
        let path_metadata = path.symlink_metadata().map_err(ReadSourceError::Io)?;
        if !path_metadata.is_file() {
            return Err(ReadSourceError::NotRegular);
        }
        if path_metadata.len() > u64::try_from(MAX_SOURCE_BYTES).unwrap_or(u64::MAX) {
            return Err(ReadSourceError::TooLarge);
        }
        if path_metadata.len() > u64::try_from(*remaining_source_bytes).unwrap_or(u64::MAX) {
            return Err(ReadSourceError::InvocationTooLarge);
        }

        let mut file = open_source_file(path).map_err(ReadSourceError::Io)?;
        let opened_metadata = file.metadata().map_err(ReadSourceError::Io)?;
        if !opened_metadata.is_file() {
            return Err(ReadSourceError::NotRegular);
        }
        if !opened_file_matches_path_metadata(&path_metadata, &opened_metadata) {
            return Err(ReadSourceError::ChangedDuringOpen);
        }
        if opened_metadata.len() > u64::try_from(MAX_SOURCE_BYTES).unwrap_or(u64::MAX) {
            return Err(ReadSourceError::TooLarge);
        }
        if opened_metadata.len() > u64::try_from(*remaining_source_bytes).unwrap_or(u64::MAX) {
            return Err(ReadSourceError::InvocationTooLarge);
        }
        let bytes = read_bounded_for_invocation(&mut file, remaining_source_bytes)?;
        post_read();
        let closed_metadata = file.metadata().map_err(ReadSourceError::Io)?;
        let final_path_metadata = path
            .symlink_metadata()
            .map_err(|_| ReadSourceError::ChangedDuringRead)?;
        if !final_path_metadata.is_file()
            || !source_read_length_matches_metadata(bytes.len(), opened_metadata.len())
            || !opened_file_metadata_unchanged(&opened_metadata, &closed_metadata)
            || !opened_file_matches_path_metadata(&final_path_metadata, &closed_metadata)
        {
            return Err(ReadSourceError::ChangedDuringRead);
        }
        Ok(bytes)
    }
}

fn open_source_file(path: &Path) -> io::Result<File> {
    #[cfg(all(
        target_os = "linux",
        any(target_arch = "x86_64", target_arch = "aarch64")
    ))]
    {
        use std::os::unix::fs::OpenOptionsExt as _;

        // Stable Linux UAPI values on the admitted x86-64 and AArch64 hosts.
        const O_NONBLOCK: i32 = 0o004_000;
        const O_NOFOLLOW: i32 = 0o400_000;

        std::fs::OpenOptions::new()
            .read(true)
            .custom_flags(O_NONBLOCK | O_NOFOLLOW)
            .open(path)
    }
    #[cfg(not(all(
        target_os = "linux",
        any(target_arch = "x86_64", target_arch = "aarch64")
    )))]
    {
        File::open(path)
    }
}

fn opened_file_matches_path_metadata(path_metadata: &Metadata, opened_metadata: &Metadata) -> bool {
    opened_file_metadata_unchanged(path_metadata, opened_metadata)
}

fn opened_file_metadata_unchanged(opened_metadata: &Metadata, closed_metadata: &Metadata) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt as _;

        opened_metadata.dev() == closed_metadata.dev()
            && opened_metadata.ino() == closed_metadata.ino()
            && opened_metadata.mode() == closed_metadata.mode()
            && opened_metadata.uid() == closed_metadata.uid()
            && opened_metadata.gid() == closed_metadata.gid()
            && opened_metadata.nlink() == closed_metadata.nlink()
            && opened_metadata.len() == closed_metadata.len()
            && opened_metadata.mtime() == closed_metadata.mtime()
            && opened_metadata.mtime_nsec() == closed_metadata.mtime_nsec()
            && opened_metadata.ctime() == closed_metadata.ctime()
            && opened_metadata.ctime_nsec() == closed_metadata.ctime_nsec()
    }
    #[cfg(not(unix))]
    {
        opened_metadata.len() == closed_metadata.len()
            && opened_metadata
                .modified()
                .ok()
                .is_some_and(|modified| closed_metadata.modified().ok() == Some(modified))
    }
}

fn source_read_length_matches_metadata(read_length: usize, metadata_length: u64) -> bool {
    u64::try_from(read_length).ok() == Some(metadata_length)
}

#[cfg(test)]
fn read_bounded(reader: impl Read) -> Result<Vec<u8>, ReadSourceError> {
    let mut remaining_source_bytes = MAX_SOURCE_BYTES;
    read_bounded_with_limit_and_reservation(
        reader,
        MAX_SOURCE_BYTES,
        ReadSourceError::TooLarge,
        &mut remaining_source_bytes,
        reserve_bounded_source_capacity,
    )
}

fn read_bounded_for_invocation(
    reader: impl Read,
    remaining_source_bytes: &mut usize,
) -> Result<Vec<u8>, ReadSourceError> {
    let limit = MAX_SOURCE_BYTES.min(*remaining_source_bytes);
    let exceeded = if *remaining_source_bytes < MAX_SOURCE_BYTES {
        ReadSourceError::InvocationTooLarge
    } else {
        ReadSourceError::TooLarge
    };
    read_bounded_with_limit_and_reservation(
        reader,
        limit,
        exceeded,
        remaining_source_bytes,
        reserve_bounded_source_capacity,
    )
}

#[cfg(test)]
fn read_bounded_with_reservation(
    mut reader: impl Read,
    mut reserve: impl FnMut(&mut Vec<u8>, usize) -> Result<(), ReadSourceError>,
) -> Result<Vec<u8>, ReadSourceError> {
    let mut remaining_source_bytes = MAX_SOURCE_BYTES;
    read_bounded_with_limit_and_reservation(
        &mut reader,
        MAX_SOURCE_BYTES,
        ReadSourceError::TooLarge,
        &mut remaining_source_bytes,
        &mut reserve,
    )
}

fn read_bounded_with_limit_and_reservation(
    mut reader: impl Read,
    limit: usize,
    exceeded: ReadSourceError,
    remaining_source_bytes: &mut usize,
    mut reserve: impl FnMut(&mut Vec<u8>, usize) -> Result<(), ReadSourceError>,
) -> Result<Vec<u8>, ReadSourceError> {
    let mut bytes = Vec::new();
    let mut buffer = [0_u8; SOURCE_READ_BUFFER_BYTES];

    while bytes.len() < limit {
        let remaining = limit
            .checked_sub(bytes.len())
            .ok_or(ReadSourceError::TooLarge)?;
        let buffer_length = remaining.min(buffer.len());
        let buffer = buffer
            .get_mut(..buffer_length)
            .ok_or_else(|| ReadSourceError::Io(invalid_data_error()))?;
        let read = loop {
            match reader.read(buffer) {
                Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
                result => break result.map_err(ReadSourceError::Io)?,
            }
        };
        if read == 0 {
            return Ok(bytes);
        }

        let chunk = buffer
            .get(..read)
            .ok_or_else(|| ReadSourceError::Io(invalid_data_error()))?;
        *remaining_source_bytes = remaining_source_bytes
            .checked_sub(chunk.len())
            .ok_or(ReadSourceError::InvocationTooLarge)?;
        reserve(&mut bytes, chunk.len())?;
        bytes.extend_from_slice(chunk);
    }

    let mut probe = [0_u8; 1];
    loop {
        match reader.read(&mut probe) {
            Ok(0) => return Ok(bytes),
            Ok(1) => {
                if let Some(remaining) = remaining_source_bytes.checked_sub(1) {
                    *remaining_source_bytes = remaining;
                }
                return Err(exceeded);
            }
            Ok(_) => return Err(ReadSourceError::Io(invalid_data_error())),
            Err(error) if error.kind() == io::ErrorKind::Interrupted => {}
            Err(error) => return Err(ReadSourceError::Io(error)),
        }
    }
}

fn reserve_bounded_source_capacity(
    bytes: &mut Vec<u8>,
    additional: usize,
) -> Result<(), ReadSourceError> {
    let required = bytes
        .len()
        .checked_add(additional)
        .filter(|&required| required <= MAX_SOURCE_BYTES)
        .ok_or(ReadSourceError::TooLarge)?;
    if required <= bytes.capacity() {
        return Ok(());
    }

    let next_capacity = bytes
        .capacity()
        .saturating_mul(2)
        .max(SOURCE_READ_BUFFER_BYTES)
        .max(required)
        .min(MAX_SOURCE_BYTES);
    let additional_capacity = next_capacity
        .checked_sub(bytes.len())
        .ok_or(ReadSourceError::TooLarge)?;
    bytes
        .try_reserve_exact(additional_capacity)
        .map_err(|_| ReadSourceError::Io(io::Error::from(io::ErrorKind::OutOfMemory)))
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
    for token in result.tokens() {
        write!(
            output,
            "{}..{}\t{}\t\"",
            token.span.start().bytes(),
            token.span.end().bytes(),
            token.kind.name()
        )?;
        if let Some(spelling) = token.lexeme(source) {
            write_escaped_token_spelling(output, spelling)?;
        }
        output.write_all(b"\"\n")?;
    }
    Ok(())
}

fn write_escaped_token_spelling(output: &mut impl Write, spelling: &str) -> io::Result<()> {
    let mut buffer = [0_u8; TOKEN_ESCAPE_BUFFER_BYTES];
    let mut used = 0_usize;
    for character in spelling.chars().flat_map(char::escape_default) {
        let mut encoded = [0_u8; 4];
        let encoded = character.encode_utf8(&mut encoded).as_bytes();
        let mut end = used
            .checked_add(encoded.len())
            .ok_or_else(invalid_data_error)?;
        if end > buffer.len() {
            let pending = buffer.get(..used).ok_or_else(invalid_data_error)?;
            output.write_all(pending)?;
            used = 0;
            end = encoded.len();
        }
        let destination = buffer.get_mut(used..end).ok_or_else(invalid_data_error)?;
        destination.copy_from_slice(encoded);
        used = end;
    }
    let pending = buffer.get(..used).ok_or_else(invalid_data_error)?;
    output.write_all(pending)
}

fn stable_source_name(path: &Path) -> Result<RenderedSourceName, SourceError> {
    if path == Path::new("-") {
        return RenderedSourceName::try_from_text("<stdin>");
    }
    RenderedSourceName::try_from_os_str(path.as_os_str())
}

fn render_read_source_error(display_name: &RenderedSourceName, error: ReadSourceError) -> String {
    match error {
        ReadSourceError::Io(error) => render_cli_error(
            CliDiagnosticCode::ReadSource,
            format_args!("could not read source file `{display_name}`"),
            io_error_reason(error.kind()),
        ),
        ReadSourceError::NotRegular => render_cli_error(
            CliDiagnosticCode::ReadSource,
            format_args!("could not read source file `{display_name}`"),
            "path does not name a regular file",
        ),
        ReadSourceError::ChangedDuringOpen => render_cli_error(
            CliDiagnosticCode::ReadSource,
            format_args!("could not read source file `{display_name}`"),
            "path changed while the source file was being opened",
        ),
        ReadSourceError::ChangedDuringRead => render_cli_error(
            CliDiagnosticCode::ReadSource,
            format_args!("could not read source file `{display_name}`"),
            "source file changed while it was being read",
        ),
        ReadSourceError::InvocationTooLarge => render_cli_error(
            CliDiagnosticCode::InvocationSourceTooLarge,
            format_args!("source input `{display_name}` exceeds the remaining invocation budget"),
            "orangec buffers at most 64 MiB of source bytes per invocation",
        ),
        ReadSourceError::TooLarge => render_cli_error(
            CliDiagnosticCode::SourceTooLarge,
            format_args!(
                "source file `{display_name}` exceeds the {MAX_SOURCE_BYTES}-byte input limit"
            ),
            "the pre-alpha compiler accepts at most 16 MiB per source",
        ),
    }
}

struct EscapedDisplayText<'text>(&'text str);

impl fmt::Display for EscapedDisplayText<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        for character in self.0.chars().flat_map(char::escape_default) {
            formatter.write_char(character)?;
        }
        Ok(())
    }
}

const fn escape_display_text(text: &str) -> EscapedDisplayText<'_> {
    EscapedDisplayText(text)
}

fn source_name_error(error: SourceError) -> String {
    render_cli_error(
        CliDiagnosticCode::SourceRepresentation,
        "could not represent source file name",
        error,
    )
}

fn source_limit_error_without_name(error: SourceError) -> String {
    render_cli_error(
        CliDiagnosticCode::SourceRepresentation,
        "could not represent source file",
        error,
    )
}

fn source_limit_error(display_name: &RenderedSourceName, error: SourceError) -> String {
    render_cli_error(
        CliDiagnosticCode::SourceRepresentation,
        format_args!("could not represent source file `{display_name}`"),
        error,
    )
}

#[derive(Debug)]
enum ReadSourceError {
    Io(io::Error),
    NotRegular,
    ChangedDuringOpen,
    ChangedDuringRead,
    InvocationTooLarge,
    TooLarge,
}

const fn io_error_reason(kind: io::ErrorKind) -> &'static str {
    match kind {
        io::ErrorKind::NotFound => "file was not found",
        io::ErrorKind::PermissionDenied => "permission was denied",
        io::ErrorKind::IsADirectory => "path names a directory",
        io::ErrorKind::InvalidData => "the operating system reported invalid data",
        io::ErrorKind::OutOfMemory => "the operating system could not allocate memory",
        _ => "the operating system reported an I/O error",
    }
}

fn render_cli_error(
    code: CliDiagnosticCode,
    message: impl std::fmt::Display,
    note: impl std::fmt::Display,
) -> String {
    // The final owned error string is part of the documented process-level
    // allocator-exhaustion residual; dynamic fields stream into this one value.
    format!("error[{}]: {message}\n  = note: {note}\n", code.as_str())
}

#[derive(Debug, Eq, PartialEq)]
enum PhaseResult<'a, T, D> {
    Complete(T),
    Diagnosed(&'a [D]),
    Missing,
}

fn classify_phase_result<'a, T, D>(
    artifact: Option<T>,
    diagnostics: &'a [D],
) -> PhaseResult<'a, T, D> {
    if !diagnostics.is_empty() {
        PhaseResult::Diagnosed(diagnostics)
    } else if let Some(artifact) = artifact {
        PhaseResult::Complete(artifact)
    } else {
        PhaseResult::Missing
    }
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

macro_rules! define_compiler_commands {
    ($($variant:ident => $name:literal,)+) => {
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        enum CompilerCommand {
            $($variant,)+
        }

        impl CompilerCommand {
            const fn as_str(self) -> &'static str {
                match self {
                    $(Self::$variant => $name,)+
                }
            }

            fn parse(value: &str) -> Option<Self> {
                match value {
                    $($name => Some(Self::$variant),)+
                    _ => None,
                }
            }

            #[cfg(test)]
            const ALL: &'static [Self] = &[$(Self::$variant,)+];
        }
    };
}

define_compiler_commands! {
    Check => "check",
    Eval => "eval",
    Lex => "lex",
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
    // Usage errors intentionally cross the same documented final-String
    // allocation boundary as compilation diagnostics.
    parse_arguments_with_path_reservation(arguments, MAX_ARGUMENT_BYTES_PER_INVOCATION, |paths| {
        paths.try_reserve(1).is_ok()
    })
}

fn parse_arguments_with_path_reservation(
    arguments: impl IntoIterator<Item = OsString>,
    argument_limit: usize,
    mut reserve_path: impl FnMut(&mut Vec<PathBuf>) -> bool,
) -> Result<Action, String> {
    let mut arguments = arguments.into_iter();
    let mut command = None;
    let mut edition = Edition::default();
    let mut edition_seen = false;
    let mut paths = Vec::new();
    let mut options_enabled = true;
    let mut remaining_argument_bytes = argument_limit;

    while let Some(argument) = arguments.next() {
        charge_argument_bytes(&mut remaining_argument_bytes, &argument)?;
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
                    mark_edition_option(&mut edition_seen)?;
                    let value = arguments
                        .next()
                        .ok_or_else(|| String::from("option `--edition` requires a value"))?;
                    charge_argument_bytes(&mut remaining_argument_bytes, &value)?;
                    edition = parse_edition(&value)?;
                    continue;
                }
                Some(value) => {
                    if let Some(value) = value.strip_prefix("--edition=") {
                        mark_edition_option(&mut edition_seen)?;
                        edition = value.parse().map_err(
                            |error: orange_compiler::ParseEditionError| error.to_string(),
                        )?;
                        continue;
                    }
                    if value.starts_with('-') && value != "-" {
                        return Err(format!("unknown option `{}`", escape_display_text(value)));
                    }
                }
                None if argument.as_encoded_bytes().starts_with(b"--edition=") => {
                    mark_edition_option(&mut edition_seen)?;
                    return Err(String::from("edition name is not valid UTF-8"));
                }
                None if argument.as_encoded_bytes().first() == Some(&b'-') => {
                    let argument = RenderedSourceName::try_from_os_str(&argument)
                        .map_err(|_| String::from("could not allocate option display text"))?;
                    return Err(format!("unknown option `{}`", argument));
                }
                _ => {}
            }
        }

        if command.is_none() {
            command =
                match utf8 {
                    Some(value) => Some(CompilerCommand::parse(value).ok_or_else(|| {
                        format!("unknown command `{}`", escape_display_text(value))
                    })?),
                    None => return Err(String::from("command is not valid UTF-8")),
                };
        } else {
            if paths.len() >= MAX_SOURCES_PER_INVOCATION {
                return Err(format!(
                    "at most {MAX_SOURCES_PER_INVOCATION} source inputs are accepted per invocation"
                ));
            }
            if !reserve_path(&mut paths) {
                return Err(String::from("could not allocate source input list"));
            }
            paths.push(PathBuf::from(argument));
        }
    }

    let command = command.ok_or_else(|| String::from("missing command"))?;
    if paths.is_empty() {
        return Err(format!(
            "command `{}` requires at least one source file",
            command.as_str()
        ));
    }
    if command == CompilerCommand::Eval && paths.len() != 1 {
        return Err(String::from(
            "command `eval` requires exactly one source file",
        ));
    }
    Ok(Action::Compile(Options {
        command,
        edition,
        paths,
    }))
}

fn charge_argument_bytes(remaining: &mut usize, argument: &OsStr) -> Result<(), String> {
    let Some(next) = remaining.checked_sub(argument.as_encoded_bytes().len()) else {
        return Err(format!(
            "command-line arguments exceed the {MAX_ARGUMENT_BYTES_PER_INVOCATION}-byte invocation limit"
        ));
    };
    *remaining = next;
    Ok(())
}

fn mark_edition_option(seen: &mut bool) -> Result<(), String> {
    if *seen {
        Err(String::from(
            "option `--edition` may be specified at most once",
        ))
    } else {
        *seen = true;
        Ok(())
    }
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

    #[cfg(unix)]
    fn unix_test_root() -> PathBuf {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/orangec-tests");
        std::fs::create_dir_all(&root).unwrap();
        root
    }

    #[test]
    fn cli_diagnostic_code_inventory_is_exact_ordered_and_unique() {
        let actual = CliDiagnosticCode::ALL
            .iter()
            .map(|code| code.as_str())
            .collect::<Vec<_>>();
        let expected = [
            "ORC1001", "ORC1002", "ORC1003", "ORC1004", "ORC1005", "ORC1006", "ORC1007", "ORC1008",
        ];

        assert_eq!(actual, expected);
        assert!(actual.windows(2).all(|pair| pair[0] < pair[1]));
        assert!(actual.iter().all(|code| {
            code.len() == 7
                && code.starts_with("ORC")
                && code[3..].bytes().all(|byte| byte.is_ascii_digit())
        }));
    }

    #[test]
    fn standard_output_limit_accepts_only_the_exact_prefix_and_reports_orc1007() {
        let mut bytes = Vec::new();
        {
            let mut output = OutputLimitedWriter::new(CountCheckedWriter::new(&mut bytes), 3);

            let error = output.write_all(b"abcd").unwrap_err();
            assert_eq!(error.kind(), io::ErrorKind::FileTooLarge);
            assert_eq!(output.write(&[]).unwrap(), 0);
            output.flush().unwrap();
            assert_eq!(
                output_failure_group(CompilerCommand::Lex, &error).as_deref(),
                Some(concat!(
                    "error[ORC1007]: standard output exceeds the 67108864-byte invocation limit\n",
                    "  = note: orangec writes at most 64 MiB to standard output per invocation\n",
                ))
            );
            assert_eq!(
                output_failure_group(
                    CompilerCommand::Lex,
                    &io::Error::from(io::ErrorKind::FileTooLarge),
                )
                .as_deref(),
                Some("orangec: could not write token output\n")
            );
        }

        assert_eq!(bytes, b"abc");

        let options = Options {
            command: CompilerCommand::Lex,
            edition: Edition::CURRENT,
            paths: vec![PathBuf::from("-")],
        };
        let mut input = b"edition 2026; module m {}".as_slice();
        let mut output = Vec::new();
        let mut diagnostic = Vec::new();
        let status = compile_with_limits(
            &options,
            &mut input,
            &mut output,
            &mut diagnostic,
            MAX_SOURCE_BYTES_PER_INVOCATION,
            3,
        );

        assert_eq!(status, COMPILATION_ERROR);
        assert_eq!(output, b"");
        assert_eq!(
            diagnostic,
            concat!(
                "error[ORC1007]: standard output exceeds the 67108864-byte invocation limit\n",
                "  = note: orangec writes at most 64 MiB to standard output per invocation\n",
            )
            .as_bytes()
        );
    }

    #[test]
    fn standard_error_limit_accepts_only_the_exact_prefix_before_source_access() {
        let mut input = RejectReads::default();
        let mut output = Vec::new();
        let mut error = Vec::new();

        let status = run_with_standard_error_limit(
            os_arguments(&["unknown", "source.or"]),
            &mut input,
            &mut output,
            &mut error,
            3,
        );

        assert_eq!(status, COMPILATION_ERROR);
        assert_eq!(input.attempts, 0);
        assert_eq!(output, b"");
        assert_eq!(error, b"ora");
    }

    #[test]
    fn invocation_source_limit_rejects_the_probe_byte_with_orc1008() {
        let options = Options {
            command: CompilerCommand::Check,
            edition: Edition::CURRENT,
            paths: vec![PathBuf::from("-")],
        };
        let mut input = b"abcd".as_slice();
        let mut output = Vec::new();
        let mut diagnostic = Vec::new();

        let status = compile_with_limits(
            &options,
            &mut input,
            &mut output,
            &mut diagnostic,
            3,
            MAX_STANDARD_OUTPUT_BYTES,
        );

        assert_eq!(status, COMPILATION_ERROR);
        assert_eq!(input, b"");
        assert_eq!(output, b"");
        assert_eq!(
            diagnostic,
            concat!(
                "error[ORC1008]: source input `<stdin>` exceeds the remaining invocation budget\n",
                "  = note: orangec buffers at most 64 MiB of source bytes per invocation\n",
            )
            .as_bytes()
        );
    }

    #[test]
    fn invocation_source_limit_is_consumed_across_operands() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
        let source_bytes = usize::try_from(path.metadata().unwrap().len()).unwrap();
        let source_limit = source_bytes
            .checked_mul(2)
            .and_then(|total| total.checked_sub(1))
            .unwrap();
        let options = Options {
            command: CompilerCommand::Check,
            edition: Edition::CURRENT,
            paths: vec![path.clone(), path],
        };
        let mut input = b"".as_slice();
        let mut output = Vec::new();
        let mut diagnostic = Vec::new();

        let status = compile_with_limits(
            &options,
            &mut input,
            &mut output,
            &mut diagnostic,
            source_limit,
            MAX_STANDARD_OUTPUT_BYTES,
        );
        let diagnostic = String::from_utf8(diagnostic).unwrap();

        assert_eq!(status, COMPILATION_ERROR);
        assert_eq!(output, b"");
        assert!(diagnostic.contains("error[ORC1008]: source input `"));
        assert!(diagnostic.ends_with(
            "  = note: orangec buffers at most 64 MiB of source bytes per invocation\n"
        ));
    }

    #[test]
    fn cli_help_default_tracks_the_current_edition_registry() {
        let edition_line = USAGE
            .lines()
            .find(|line| line.trim_start().starts_with("--edition "));
        assert_eq!(
            edition_line,
            Some(
                format!(
                    "      --edition <YEAR>  Select the Orange edition [default: {}; at most once]",
                    Edition::CURRENT
                )
                .as_str()
            )
        );
        assert_eq!(
            parse_edition(OsStr::new(Edition::CURRENT.as_str())),
            Ok(Edition::CURRENT)
        );
    }

    #[test]
    fn cli_command_inventory_matches_the_help_rows() {
        let names = CompilerCommand::ALL
            .iter()
            .map(|command| command.as_str())
            .collect::<Vec<_>>();
        assert_eq!(names, ["check", "eval", "lex"]);
        assert_eq!(
            CompilerCommand::ALL
                .iter()
                .map(|command| CompilerCommand::parse(command.as_str()))
                .collect::<Vec<_>>(),
            CompilerCommand::ALL
                .iter()
                .copied()
                .map(Some)
                .collect::<Vec<_>>()
        );

        let help_names = USAGE
            .split_once("Commands:\n")
            .unwrap()
            .1
            .split_once("\n\nOptions:")
            .unwrap()
            .0
            .lines()
            .map(|line| line.split_whitespace().next().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(help_names, names);
    }

    struct RejectWrites(io::ErrorKind);

    impl Write for RejectWrites {
        fn write(&mut self, _buffer: &[u8]) -> io::Result<usize> {
            Err(io::Error::from(self.0))
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    struct RejectReads {
        attempts: usize,
        error_kind: io::ErrorKind,
    }

    impl Default for RejectReads {
        fn default() -> Self {
            Self {
                attempts: 0,
                error_kind: io::ErrorKind::Other,
            }
        }
    }

    impl Read for RejectReads {
        fn read(&mut self, _buffer: &mut [u8]) -> io::Result<usize> {
            self.attempts += 1;
            Err(io::Error::from(self.error_kind))
        }
    }

    struct OverReportingReader;

    impl Read for OverReportingReader {
        fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
            Ok(buffer.len() + 1)
        }
    }

    #[derive(Default)]
    struct OverReportingWriter {
        attempts: usize,
    }

    impl Write for OverReportingWriter {
        fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
            self.attempts += 1;
            Ok(buffer.len() + 1)
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    #[derive(Default)]
    struct InterruptFirstWrite {
        attempts: usize,
        bytes: Vec<u8>,
    }

    impl Write for InterruptFirstWrite {
        fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
            self.attempts += 1;
            if self.attempts == 1 {
                return Err(io::Error::from(io::ErrorKind::Interrupted));
            }
            self.bytes.extend_from_slice(buffer);
            Ok(buffer.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    #[derive(Default)]
    struct InterruptFirstFlush {
        flush_attempts: usize,
        bytes: Vec<u8>,
    }

    impl Write for InterruptFirstFlush {
        fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
            self.bytes.extend_from_slice(buffer);
            Ok(buffer.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            self.flush_attempts += 1;
            if self.flush_attempts == 1 {
                Err(io::Error::from(io::ErrorKind::Interrupted))
            } else {
                Ok(())
            }
        }
    }

    struct InstrumentedSourceReader {
        remaining: usize,
        maximum_partial_read: usize,
        interrupt_body_once: bool,
        interrupt_probe_once: bool,
        overreport_probe_once: bool,
        probe_error: Option<io::ErrorKind>,
        requested_buffer_lengths: Vec<usize>,
    }

    impl InstrumentedSourceReader {
        fn new(remaining: usize) -> Self {
            Self {
                remaining,
                maximum_partial_read: usize::MAX,
                interrupt_body_once: false,
                interrupt_probe_once: false,
                overreport_probe_once: false,
                probe_error: None,
                requested_buffer_lengths: Vec::new(),
            }
        }
    }

    impl Read for InstrumentedSourceReader {
        fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
            self.requested_buffer_lengths.push(buffer.len());
            if buffer.len() == 1 {
                if self.interrupt_probe_once {
                    self.interrupt_probe_once = false;
                    return Err(io::Error::from(io::ErrorKind::Interrupted));
                }
                if self.overreport_probe_once {
                    self.overreport_probe_once = false;
                    return Ok(buffer.len() + 1);
                }
                if let Some(kind) = self.probe_error.take() {
                    return Err(io::Error::from(kind));
                }
            } else if self.interrupt_body_once {
                self.interrupt_body_once = false;
                return Err(io::Error::from(io::ErrorKind::Interrupted));
            }

            let read = self
                .remaining
                .min(buffer.len())
                .min(self.maximum_partial_read);
            buffer[..read].fill(b'x');
            self.remaining -= read;
            Ok(read)
        }
    }

    #[derive(Default)]
    struct FailFirstWrite {
        attempts: usize,
        bytes: Vec<u8>,
    }

    impl Write for FailFirstWrite {
        fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
            self.attempts += 1;
            if self.attempts == 1 {
                Err(io::Error::from(io::ErrorKind::Other))
            } else {
                self.bytes.extend_from_slice(buffer);
                Ok(buffer.len())
            }
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    #[derive(Default)]
    struct ZeroWrites {
        attempts: usize,
    }

    impl Write for ZeroWrites {
        fn write(&mut self, _buffer: &[u8]) -> io::Result<usize> {
            self.attempts += 1;
            Ok(0)
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    struct AcceptPrefixThenFail {
        prefix_bytes: usize,
        error_kind: io::ErrorKind,
        attempts: usize,
        bytes: Vec<u8>,
    }

    impl AcceptPrefixThenFail {
        fn new(prefix_bytes: usize, error_kind: io::ErrorKind) -> Self {
            assert!(prefix_bytes > 0);
            Self {
                prefix_bytes,
                error_kind,
                attempts: 0,
                bytes: Vec::new(),
            }
        }
    }

    impl Write for AcceptPrefixThenFail {
        fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
            self.attempts += 1;
            if self.bytes.len() == self.prefix_bytes {
                return Err(io::Error::from(self.error_kind));
            }

            let accepted = buffer.len().min(self.prefix_bytes - self.bytes.len());
            self.bytes.extend_from_slice(&buffer[..accepted]);
            Ok(accepted)
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    #[derive(Default)]
    struct FailFlush {
        flush_attempts: usize,
        bytes: Vec<u8>,
    }

    impl Write for FailFlush {
        fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
            self.bytes.extend_from_slice(buffer);
            Ok(buffer.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            self.flush_attempts += 1;
            Err(io::Error::from(io::ErrorKind::Other))
        }
    }

    #[derive(Default)]
    struct MeasureWrites {
        bytes: usize,
        largest_write: usize,
        writes: usize,
    }

    impl Write for MeasureWrites {
        fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
            self.bytes += buffer.len();
            self.largest_write = self.largest_write.max(buffer.len());
            self.writes += 1;
            Ok(buffer.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    fn os_arguments(values: &[&str]) -> Vec<OsString> {
        values.iter().map(OsString::from).collect()
    }

    fn run_with_source_bytes(command: &str, source: &[u8]) -> (u8, Vec<u8>, Vec<u8>) {
        let mut input = source;
        let mut output = Vec::new();
        let mut error = Vec::new();
        let status = run(
            os_arguments(&[command, "-"]),
            &mut input,
            &mut output,
            &mut error,
        );
        (status, output, error)
    }

    #[test]
    fn phase_result_classification_fails_closed_for_inconsistent_states() {
        let diagnostics = [()];

        assert_eq!(
            classify_phase_result(Some(7), &[] as &[()]),
            PhaseResult::Complete(7)
        );
        assert_eq!(
            classify_phase_result(Some(7), &diagnostics),
            PhaseResult::Diagnosed(&diagnostics)
        );
        assert_eq!(
            classify_phase_result(None::<u8>, &diagnostics),
            PhaseResult::Diagnosed(&diagnostics)
        );
        assert_eq!(
            classify_phase_result(None::<u8>, &[] as &[()]),
            PhaseResult::Missing
        );
    }

    #[test]
    fn source_representation_failures_have_stable_diagnostics() {
        let display_name = stable_source_name(Path::new("-")).unwrap();
        for (error, note) in [
            (
                SourceError::IdentitySpaceExhausted,
                "source-map identity space is exhausted",
            ),
            (
                SourceError::TooLarge,
                "source exceeds the 16 MiB input limit",
            ),
            (
                SourceError::TooManyFiles,
                "source map exceeds the file representation limit",
            ),
            (
                SourceError::SourceAllocationFailed,
                "could not allocate owned source data",
            ),
            (
                SourceError::IndexAllocationFailed,
                "could not allocate source indexing data",
            ),
        ] {
            assert_eq!(
                source_limit_error(&display_name, error),
                format!(
                    "error[ORC1005]: could not represent source file `<stdin>`\n  = note: {note}\n"
                )
            );
        }
        assert_eq!(
            source_name_error(SourceError::SourceAllocationFailed),
            concat!(
                "error[ORC1005]: could not represent source file name\n",
                "  = note: could not allocate owned source data\n",
            )
        );
        assert_eq!(
            source_limit_error_without_name(SourceError::IndexAllocationFailed),
            concat!(
                "error[ORC1005]: could not represent source file\n",
                "  = note: could not allocate source indexing data\n",
            )
        );
    }

    #[test]
    fn non_compilation_output_failures_return_failure_without_reading_input() {
        let mut input = RejectReads::default();
        let mut output = Vec::new();
        let mut error = RejectWrites(io::ErrorKind::Other);

        let status = run(
            os_arguments(&["unknown", "source.or"]),
            &mut input,
            &mut output,
            &mut error,
        );

        assert_eq!(status, COMPILATION_ERROR);
        assert_eq!(input.attempts, 0);
        assert_eq!(output, b"");

        for arguments in [["--help"].as_slice(), ["--version"].as_slice()] {
            let mut input = RejectReads::default();
            let mut output = RejectWrites(io::ErrorKind::Other);
            let mut error = Vec::new();

            let status = run(os_arguments(arguments), &mut input, &mut output, &mut error);

            assert_eq!(status, COMPILATION_ERROR, "{arguments:?}");
            assert_eq!(input.attempts, 0, "{arguments:?}");
            assert_eq!(error, b"", "{arguments:?}");
        }

        let mut input = RejectReads::default();
        let mut output = Vec::new();
        let mut error = FailFlush::default();
        let status = run(
            os_arguments(&["unknown", "source.or"]),
            &mut input,
            &mut output,
            &mut error,
        );
        assert_eq!(status, COMPILATION_ERROR);
        assert_eq!(input.attempts, 0);
        assert_eq!(output, b"");
        assert_eq!(error.flush_attempts, 1);
        assert!(error.bytes.starts_with(b"orangec: unknown command"));

        for arguments in [["--help"].as_slice(), ["--version"].as_slice()] {
            let mut input = RejectReads::default();
            let mut output = FailFlush::default();
            let mut error = Vec::new();
            let status = run(os_arguments(arguments), &mut input, &mut output, &mut error);
            assert_eq!(status, COMPILATION_ERROR, "{arguments:?}");
            assert_eq!(input.attempts, 0, "{arguments:?}");
            assert_eq!(output.flush_attempts, 1, "{arguments:?}");
            assert!(!output.bytes.is_empty(), "{arguments:?}");
            assert_eq!(error, b"", "{arguments:?}");
        }
    }

    #[test]
    fn overreporting_output_writers_are_rejected_without_reading_input() {
        let mut input = RejectReads::default();
        let mut output = OverReportingWriter::default();
        let mut error = Vec::new();

        let status = run(
            os_arguments(&["--help"]),
            &mut input,
            &mut output,
            &mut error,
        );

        assert_eq!(status, COMPILATION_ERROR);
        assert_eq!(input.attempts, 0);
        assert_eq!(output.attempts, 1);
        assert_eq!(error, b"");

        let mut input = RejectReads::default();
        let mut output = Vec::new();
        let mut error = OverReportingWriter::default();
        let status = run(
            os_arguments(&["unknown", "source.or"]),
            &mut input,
            &mut output,
            &mut error,
        );

        assert_eq!(status, COMPILATION_ERROR);
        assert_eq!(input.attempts, 0);
        assert_eq!(output, b"");
        assert_eq!(error.attempts, 1);
    }

    #[test]
    fn output_limit_rejects_an_inner_writer_count_overreport() {
        let mut output = OutputLimitedWriter::new(OverReportingWriter::default(), 8);

        let error = output.write(b"abc").unwrap_err();

        assert_eq!(error.kind(), io::ErrorKind::InvalidData);
        assert_eq!(output.remaining, 8);
        assert_eq!(output.into_inner().attempts, 1);
    }

    #[test]
    fn overreporting_output_writers_are_rejected_across_compilation_paths() {
        let mut input = b"module caf\xc3\xa9 {}\n".as_slice();
        let mut output = Vec::new();
        let mut error = OverReportingWriter::default();
        let status = run(
            os_arguments(&["check", "-"]),
            &mut input,
            &mut output,
            &mut error,
        );

        assert_eq!(status, COMPILATION_ERROR);
        assert_eq!(output, b"");
        assert_eq!(error.attempts, 1);

        let source = b"edition 2026; module values { spec answer() -> Int { 42 } }\n";
        let mut output = OverReportingWriter::default();
        let (status, error) = run_evaluation_with_output(source, &mut output);

        assert_eq!(status, COMPILATION_ERROR);
        assert_eq!(output.attempts, 1);
        assert_eq!(error, b"orangec: could not write evaluation output\n");

        let mut input = b"edition 2026; module values {}\n".as_slice();
        let mut output = OverReportingWriter::default();
        let mut error = Vec::new();
        let status = run(
            os_arguments(&["lex", "-"]),
            &mut input,
            &mut output,
            &mut error,
        );

        assert_eq!(status, COMPILATION_ERROR);
        assert_eq!(output.attempts, 1);
        assert_eq!(error, b"orangec: could not write token output\n");
    }

    #[test]
    fn interrupted_output_write_is_retried_across_output_classes() {
        let mut input = RejectReads::default();
        let mut output = InterruptFirstWrite::default();
        let mut error = Vec::new();

        let status = run(
            os_arguments(&["--help"]),
            &mut input,
            &mut output,
            &mut error,
        );

        assert_eq!(status, SUCCESS);
        assert_eq!(input.attempts, 0);
        assert_eq!(output.attempts, 2);
        assert_eq!(output.bytes, USAGE.as_bytes());
        assert_eq!(error, b"");

        let mut input = RejectReads::default();
        let mut output = Vec::new();
        let mut error = InterruptFirstWrite::default();
        let status = run(
            os_arguments(&["unknown", "source.or"]),
            &mut input,
            &mut output,
            &mut error,
        );

        assert_eq!(status, USAGE_ERROR);
        assert_eq!(input.attempts, 0);
        assert_eq!(output, b"");
        assert!(error.attempts >= 2);
        assert_eq!(
            error.bytes,
            format!("orangec: unknown command `unknown`\n\n{USAGE}").as_bytes()
        );

        let source = b"module caf\xc3\xa9 {}\n";
        let mut input = source.as_slice();
        let mut output = Vec::new();
        let mut error = InterruptFirstWrite::default();
        let status = run(
            os_arguments(&["check", "-"]),
            &mut input,
            &mut output,
            &mut error,
        );

        assert_eq!(status, COMPILATION_ERROR);
        assert_eq!(output, b"");
        assert!(error.attempts >= 2);
        assert_eq!(
            error.bytes,
            concat!(
                "error[ORC0001]: unexpected character U+00E9\n",
                " --> <stdin>:1:11\n",
                "  |\n",
                "1 | module caf\\u{e9} {}\n",
                "  |           ^^^^^^ character is not part of Orange 2026\n",
                "  = note: identifiers are ASCII in this pre-alpha edition\n",
            )
            .as_bytes()
        );

        let source = b"edition 2026; module values { spec answer() -> Int { 42 } }\n";
        let mut output = InterruptFirstWrite::default();
        let (status, error) = run_evaluation_with_output(source, &mut output);

        assert_eq!(status, SUCCESS);
        assert!(output.attempts >= 2);
        assert_eq!(output.bytes, b"values::answer: Int = 42\n");
        assert_eq!(error, b"");
    }

    #[test]
    fn interrupted_flush_is_retried_across_output_classes() {
        let mut input = RejectReads::default();
        let mut output = InterruptFirstFlush::default();
        let mut error = Vec::new();
        let status = run(
            os_arguments(&["--help"]),
            &mut input,
            &mut output,
            &mut error,
        );
        assert_eq!(status, SUCCESS);
        assert_eq!(input.attempts, 0);
        assert_eq!(output.flush_attempts, 2);
        assert_eq!(output.bytes, USAGE.as_bytes());
        assert_eq!(error, b"");

        let mut input = RejectReads::default();
        let mut output = Vec::new();
        let mut error = InterruptFirstFlush::default();
        let status = run(
            os_arguments(&["unknown", "source.or"]),
            &mut input,
            &mut output,
            &mut error,
        );
        assert_eq!(status, USAGE_ERROR);
        assert_eq!(input.attempts, 0);
        assert_eq!(output, b"");
        assert_eq!(error.flush_attempts, 2);
        assert_eq!(
            error.bytes,
            format!("orangec: unknown command `unknown`\n\n{USAGE}").as_bytes()
        );

        let source = b"edition 2026; module values { spec answer() -> Int { 42 } }\n";
        let mut output = InterruptFirstFlush::default();
        let (status, error) = run_evaluation_with_output(source, &mut output);
        assert_eq!(status, SUCCESS);
        assert_eq!(output.flush_attempts, 2);
        assert_eq!(output.bytes, b"values::answer: Int = 42\n");
        assert_eq!(error, b"");

        let mut input = b"module caf\xc3\xa9 {}\n".as_slice();
        let mut output = Vec::new();
        let mut error = InterruptFirstFlush::default();
        let status = run(
            os_arguments(&["check", "-"]),
            &mut input,
            &mut output,
            &mut error,
        );
        assert_eq!(status, COMPILATION_ERROR);
        assert_eq!(output, b"");
        assert_eq!(error.flush_attempts, 2);
        assert!(error.bytes.starts_with(b"error[ORC0001]:"));
    }

    #[test]
    fn compilation_flushes_only_streams_that_received_output() {
        let mut invalid_input = b"module caf\xc3\xa9 {}\n".as_slice();
        let mut output = Vec::new();
        let mut diagnostic_error = FailFlush::default();
        let invalid_status = run(
            os_arguments(&["check", "-"]),
            &mut invalid_input,
            &mut output,
            &mut diagnostic_error,
        );

        assert_eq!(invalid_status, COMPILATION_ERROR);
        assert_eq!(output, b"");
        assert_eq!(diagnostic_error.flush_attempts, 1);
        assert!(diagnostic_error.bytes.starts_with(b"error[ORC0001]:"));

        let valid_source = b"edition 2026; module valid {}\n";
        let mut valid_input = valid_source.as_slice();
        let mut untouched_output = FailFlush::default();
        let mut untouched_error = FailFlush::default();
        let valid_status = run(
            os_arguments(&["check", "-"]),
            &mut valid_input,
            &mut untouched_output,
            &mut untouched_error,
        );

        assert_eq!(valid_status, SUCCESS);
        assert_eq!(untouched_output.flush_attempts, 0);
        assert_eq!(untouched_output.bytes, b"");
        assert_eq!(untouched_error.flush_attempts, 0);
        assert_eq!(untouched_error.bytes, b"");

        let mut empty_evaluation_output = FailFlush::default();
        let (empty_status, empty_error) =
            run_evaluation_with_output(valid_source, &mut empty_evaluation_output);

        assert_eq!(empty_status, SUCCESS);
        assert_eq!(empty_evaluation_output.flush_attempts, 0);
        assert_eq!(empty_evaluation_output.bytes, b"");
        assert_eq!(empty_error, b"");
    }

    fn run_evaluation_with_output(source: &[u8], output: &mut impl Write) -> (u8, Vec<u8>) {
        let mut input = source;
        let mut error = Vec::new();
        let status = run(os_arguments(&["eval", "-"]), &mut input, output, &mut error);
        (status, error)
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
        assert_eq!(
            parse_arguments(os_arguments(&["eval", "--edition=2026", "one.or"])),
            Ok(Action::Compile(Options {
                command: CompilerCommand::Eval,
                edition: Edition::E2026,
                paths: vec![PathBuf::from("one.or")],
            }))
        );
    }

    #[test]
    fn rejects_repeated_edition_options_before_reading_input() {
        let expected = Err(String::from(
            "option `--edition` may be specified at most once",
        ));
        for arguments in [
            ["--edition", "2026", "--edition", "2026", "check", "-"].as_slice(),
            ["--edition=2026", "check", "--edition=2026", "-"].as_slice(),
            ["check", "--edition", "2026", "--edition=2026", "-"].as_slice(),
        ] {
            assert_eq!(parse_arguments(os_arguments(arguments)), expected);
        }

        let mut input = RejectReads::default();
        let mut output = Vec::new();
        let mut error = Vec::new();
        let status = run(
            os_arguments(&["--edition=2026", "check", "--edition", "2026", "-"]),
            &mut input,
            &mut output,
            &mut error,
        );

        assert_eq!(status, USAGE_ERROR);
        assert_eq!(input.attempts, 0);
        assert_eq!(output, b"");
        assert_eq!(
            error,
            format!("orangec: option `--edition` may be specified at most once\n\n{USAGE}")
                .as_bytes()
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

    #[cfg(unix)]
    #[test]
    fn non_utf8_dash_prefixed_paths_require_the_option_marker() {
        use std::os::unix::ffi::OsStringExt as _;

        let dash_prefixed = OsString::from_vec(b"-\x80.or".to_vec());
        let ordinary = OsString::from_vec(b"source-\x80.or".to_vec());

        assert_eq!(
            parse_arguments([OsString::from("check"), dash_prefixed.clone()]),
            Err(String::from("unknown option `-\\x80.or`"))
        );
        assert_eq!(
            parse_arguments([OsString::from("check"), ordinary.clone()])
                .unwrap()
                .compile_options()
                .paths,
            vec![PathBuf::from(ordinary)]
        );
        assert_eq!(
            parse_arguments([
                OsString::from("check"),
                OsString::from("--"),
                dash_prefixed.clone(),
            ])
            .unwrap()
            .compile_options()
            .paths,
            vec![PathBuf::from(dash_prefixed)]
        );
    }

    #[cfg(unix)]
    #[test]
    fn non_utf8_edition_forms_are_rejected_consistently_before_input() {
        use std::os::unix::ffi::OsStringExt as _;

        let edition = OsString::from_vec(vec![0x80]);
        let inline_edition = OsString::from_vec(b"--edition=\x80".to_vec());
        let expected = Err(String::from("edition name is not valid UTF-8"));

        assert_eq!(
            parse_arguments([
                OsString::from("--edition"),
                edition,
                OsString::from("check"),
                OsString::from("-"),
            ]),
            expected
        );
        assert_eq!(
            parse_arguments([
                inline_edition.clone(),
                OsString::from("check"),
                OsString::from("-"),
            ]),
            expected
        );
        assert_eq!(
            parse_arguments([
                OsString::from("--edition=2026"),
                inline_edition.clone(),
                OsString::from("check"),
                OsString::from("-"),
            ]),
            Err(String::from(
                "option `--edition` may be specified at most once"
            ))
        );

        let mut input = RejectReads::default();
        let mut output = Vec::new();
        let mut error = Vec::new();
        let status = run(
            [inline_edition, OsString::from("check"), OsString::from("-")],
            &mut input,
            &mut output,
            &mut error,
        );

        assert_eq!(status, USAGE_ERROR);
        assert_eq!(input.attempts, 0);
        assert_eq!(output, b"");
        assert_eq!(
            error,
            format!("orangec: edition name is not valid UTF-8\n\n{USAGE}").as_bytes()
        );
    }

    #[cfg(unix)]
    #[test]
    fn raw_argument_byte_corpus_is_repeatable_and_error_text_is_ascii_safe() {
        use std::os::unix::ffi::OsStringExt as _;

        let mut state = 0x6a09_e667_f3bc_c909_u64;
        let mut exercised_non_utf8 = false;
        let mut exercised_control = false;

        for case_index in 0..512_u32 {
            state = state
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            let length = usize::try_from(state % 33).unwrap();
            let mut bytes = Vec::with_capacity(length);
            for _ in 0..length {
                state ^= state << 13;
                state ^= state >> 7;
                state ^= state << 17;
                let byte = u8::try_from(state & u64::from(u8::MAX)).unwrap();
                exercised_non_utf8 |= byte >= 0x80;
                exercised_control |= byte.is_ascii_control();
                bytes.push(byte);
            }
            let raw = OsString::from_vec(bytes);
            let cases = [
                vec![raw.clone()],
                vec![OsString::from("check"), raw.clone()],
                vec![OsString::from("check"), OsString::from("--"), raw.clone()],
                vec![
                    OsString::from("--edition"),
                    raw,
                    OsString::from("check"),
                    OsString::from("-"),
                ],
            ];

            for arguments in cases {
                let first = parse_arguments(arguments.clone());
                let second = parse_arguments(arguments);
                assert_eq!(
                    first, second,
                    "argument case {case_index} was not repeatable"
                );
                if let Err(message) = first {
                    assert!(
                        message.is_ascii(),
                        "argument case {case_index}: {message:?}"
                    );
                    assert!(
                        !message.bytes().any(|byte| byte.is_ascii_control()),
                        "argument case {case_index}: {message:?}"
                    );
                }
            }
        }

        assert!(exercised_non_utf8);
        assert!(exercised_control);
    }

    #[test]
    fn raw_source_byte_corpus_is_repeatable_and_output_is_ascii_safe() {
        let mut corpus = vec![
            Vec::new(),
            b"edition 2026; module values { spec answer() -> Int { 42 } }\n".to_vec(),
            vec![0x80],
            vec![0xc2],
            vec![0xc0, 0x80],
            vec![b'e', b'd', 0xf0, 0x9f, 0x92],
        ];
        corpus.extend((u8::MIN..=u8::MAX).map(|byte| vec![byte]));
        let mut state = 0xbb67_ae85_84ca_a73b_u64;
        for _ in 0..256 {
            state = state
                .wrapping_mul(2_862_933_555_777_941_757)
                .wrapping_add(3_037_000_493);
            let length = usize::try_from(state % 65).unwrap();
            let mut bytes = Vec::with_capacity(length);
            for _ in 0..length {
                state ^= state << 7;
                state ^= state >> 9;
                state ^= state << 8;
                bytes.push(u8::try_from(state & u64::from(u8::MAX)).unwrap());
            }
            corpus.push(bytes);
        }

        let mut observed_success = false;
        let mut observed_utf8_rejection = false;
        let mut observed_lex_output = false;
        for (case_index, source) in corpus.iter().enumerate() {
            for command in ["check", "eval", "lex"] {
                let first = run_with_source_bytes(command, source);
                let second = run_with_source_bytes(command, source);
                assert_eq!(
                    first, second,
                    "source case {case_index} under {command} was not repeatable"
                );

                let (status, output, error) = first;
                assert!(matches!(status, SUCCESS | COMPILATION_ERROR));
                observed_success |= status == SUCCESS;
                observed_utf8_rejection |= error.starts_with(b"error[ORC1002]:");
                observed_lex_output |= command == "lex" && !output.is_empty();
                assert!(
                    output.is_ascii() && error.is_ascii(),
                    "source case {case_index} under {command}"
                );
                assert!(
                    !output
                        .iter()
                        .copied()
                        .any(|byte| { byte.is_ascii_control() && !matches!(byte, b'\n' | b'\t') }),
                    "source case {case_index} under {command}: {output:?}"
                );
                assert!(
                    !error
                        .iter()
                        .copied()
                        .any(|byte| byte.is_ascii_control() && byte != b'\n'),
                    "source case {case_index} under {command}: {error:?}"
                );
            }
        }

        assert!(observed_success);
        assert!(observed_utf8_rejection);
        assert!(observed_lex_output);
    }

    #[cfg(unix)]
    #[test]
    fn non_utf8_source_names_are_escaped_without_lossy_aliases() {
        use std::os::unix::ffi::OsStringExt as _;

        let first = PathBuf::from(OsString::from_vec(b"source-\x80.or".to_vec()));
        let second = PathBuf::from(OsString::from_vec(b"source-\x81.or".to_vec()));
        let literal_escape = PathBuf::from(r"source-\x80.or");

        let first = stable_source_name(&first).unwrap();
        let second = stable_source_name(&second).unwrap();
        let literal_escape = stable_source_name(&literal_escape).unwrap();

        assert_eq!(first.as_str(), "source-\\x80.or");
        assert_eq!(second.as_str(), "source-\\x81.or");
        assert_eq!(literal_escape.as_str(), r"source-\\x80.or");
        assert_ne!(first, second);
        assert_ne!(first, literal_escape);
    }

    #[cfg(unix)]
    #[test]
    fn non_utf8_source_diagnostics_encode_the_path_exactly_once() {
        use std::os::unix::ffi::OsStringExt as _;

        let mut path_bytes = b"source-".to_vec();
        path_bytes.extend_from_slice(b"\x80.or");
        let path = PathBuf::from(OsString::from_vec(path_bytes));
        let expected_name = stable_source_name(&path).unwrap();
        let mut sources = SourceMap::new();
        let id = sources
            .add_with_rendered_name(expected_name.clone(), "@")
            .unwrap();
        let source = sources.get(id).unwrap();
        let lexed = lex(source, Edition::E2026);

        assert!(lexed.has_errors());
        assert_eq!(
            expected_name
                .as_str()
                .chars()
                .filter(|&ch| ch == '\\')
                .count(),
            1
        );
        let rendered = render_diagnostics(&sources, lexed.diagnostics());
        let location = rendered
            .lines()
            .find(|line| line.starts_with(" --> "))
            .unwrap();
        assert_eq!(location, format!(" --> {expected_name}:1:1"));
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
    fn display_text_encoding_matches_default_escaping() {
        let ordinary = "ordinary-option";
        assert_eq!(escape_display_text(ordinary).to_string(), ordinary);

        let escaped_source = "'\"\\\né";
        assert_eq!(
            escape_display_text(escaped_source).to_string(),
            escaped_source
                .chars()
                .flat_map(char::escape_default)
                .collect::<String>()
        );
    }

    #[test]
    fn escapes_untrusted_command_and_option_text_injectively() {
        assert_eq!(
            parse_arguments(os_arguments(&["bad\ncommand", "source.or"])),
            Err(String::from("unknown command `bad\\ncommand`"))
        );
        assert_eq!(
            parse_arguments(os_arguments(&[r"bad\ncommand", "source.or"])),
            Err(String::from("unknown command `bad\\\\ncommand`"))
        );
        assert_eq!(
            parse_arguments(os_arguments(&["check", "--bad\u{1b}[31m", "source.or"])),
            Err(String::from("unknown option `--bad\\u{1b}[31m`"))
        );
        assert_eq!(
            parse_arguments(os_arguments(&["check", "--bad\u{202e}", "source.or"])),
            Err(String::from("unknown option `--bad\\u{202e}`"))
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

    #[test]
    fn bounds_aggregate_argument_bytes_before_interpretation() {
        let arguments = os_arguments(&["--edition", "2026", "check", "source.or"]);
        let exact_bytes = arguments
            .iter()
            .map(|argument| argument.as_encoded_bytes().len())
            .sum();
        assert!(
            parse_arguments_with_path_reservation(arguments.clone(), exact_bytes, |_| true).is_ok()
        );
        assert_eq!(
            parse_arguments_with_path_reservation(arguments, exact_bytes - 1, |_| true),
            Err(format!(
                "command-line arguments exceed the {MAX_ARGUMENT_BYTES_PER_INVOCATION}-byte invocation limit"
            ))
        );
    }

    #[test]
    fn source_input_list_reservation_failure_is_a_usage_error() {
        let result = parse_arguments_with_path_reservation(
            os_arguments(&["check", "source.or"]),
            MAX_ARGUMENT_BYTES_PER_INVOCATION,
            |_| false,
        );

        assert_eq!(
            result,
            Err(String::from("could not allocate source input list"))
        );
    }

    #[test]
    fn reference_evaluation_requires_exactly_one_source() {
        assert_eq!(
            parse_arguments(os_arguments(&["eval"])),
            Err(String::from(
                "command `eval` requires at least one source file"
            ))
        );
        assert_eq!(
            parse_arguments(os_arguments(&["eval", "one.or", "two.or"])),
            Err(String::from(
                "command `eval` requires exactly one source file"
            ))
        );
    }

    #[test]
    fn bounded_read_accepts_the_exact_limit_without_overallocating_for_a_probe() {
        let mut reader = InstrumentedSourceReader::new(MAX_SOURCE_BYTES);

        let bytes = read_bounded(&mut reader).unwrap();

        assert_eq!(bytes.len(), MAX_SOURCE_BYTES);
        assert_eq!(bytes.capacity(), MAX_SOURCE_BYTES);
        assert_eq!(reader.remaining, 0);
        assert_eq!(reader.requested_buffer_lengths.last(), Some(&1));
        assert!(
            reader.requested_buffer_lengths[..reader.requested_buffer_lengths.len() - 1]
                .iter()
                .all(|&length| length == SOURCE_READ_BUFFER_BYTES)
        );
    }

    #[test]
    fn bounded_read_stops_after_a_body_reservation_failure() {
        let mut reader = InstrumentedSourceReader::new(SOURCE_READ_BUFFER_BYTES * 2);
        let result = read_bounded_with_reservation(&mut reader, |bytes, additional| {
            assert!(bytes.is_empty());
            assert_eq!(additional, SOURCE_READ_BUFFER_BYTES);
            Err(ReadSourceError::Io(io::Error::from(
                io::ErrorKind::OutOfMemory,
            )))
        });

        let Err(ReadSourceError::Io(error)) = result else {
            panic!("expected the injected allocation failure");
        };
        assert_eq!(error.kind(), io::ErrorKind::OutOfMemory);
        assert_eq!(reader.remaining, SOURCE_READ_BUFFER_BYTES);
        assert_eq!(reader.requested_buffer_lengths, [SOURCE_READ_BUFFER_BYTES]);
    }

    #[test]
    fn invocation_budget_charges_bytes_before_a_body_reservation_failure() {
        let mut reader = InstrumentedSourceReader::new(SOURCE_READ_BUFFER_BYTES * 2);
        let mut remaining_source_bytes = SOURCE_READ_BUFFER_BYTES * 2;
        let result = read_bounded_with_limit_and_reservation(
            &mut reader,
            SOURCE_READ_BUFFER_BYTES * 2,
            ReadSourceError::InvocationTooLarge,
            &mut remaining_source_bytes,
            |bytes, additional| {
                assert!(bytes.is_empty());
                assert_eq!(additional, SOURCE_READ_BUFFER_BYTES);
                Err(ReadSourceError::Io(io::Error::from(
                    io::ErrorKind::OutOfMemory,
                )))
            },
        );

        let Err(ReadSourceError::Io(error)) = result else {
            panic!("expected the injected allocation failure");
        };
        assert_eq!(error.kind(), io::ErrorKind::OutOfMemory);
        assert_eq!(remaining_source_bytes, SOURCE_READ_BUFFER_BYTES);
        assert_eq!(reader.remaining, SOURCE_READ_BUFFER_BYTES);
        assert_eq!(reader.requested_buffer_lengths, [SOURCE_READ_BUFFER_BYTES]);
    }

    #[test]
    fn per_source_probe_byte_consumes_the_invocation_budget() {
        let mut reader = InstrumentedSourceReader::new(4);
        let mut remaining_source_bytes = 10;

        let result = read_bounded_with_limit_and_reservation(
            &mut reader,
            3,
            ReadSourceError::TooLarge,
            &mut remaining_source_bytes,
            reserve_bounded_source_capacity,
        );

        assert!(matches!(result, Err(ReadSourceError::TooLarge)));
        assert_eq!(remaining_source_bytes, 6);
        assert_eq!(reader.remaining, 0);
        assert_eq!(reader.requested_buffer_lengths, [3, 1]);
    }

    #[test]
    fn bounded_read_rejects_limit_plus_one_using_only_the_separate_probe() {
        let mut reader = InstrumentedSourceReader::new(MAX_SOURCE_BYTES + 1);

        let result = read_bounded(&mut reader);

        assert!(matches!(result, Err(ReadSourceError::TooLarge)));
        assert_eq!(reader.remaining, 0);
        assert_eq!(reader.requested_buffer_lengths.last(), Some(&1));
        assert!(
            reader.requested_buffer_lengths[..reader.requested_buffer_lengths.len() - 1]
                .iter()
                .all(|&length| length == SOURCE_READ_BUFFER_BYTES)
        );
    }

    #[test]
    fn bounded_read_retries_interrupted_partial_body_and_probe_reads() {
        let mut reader = InstrumentedSourceReader::new(MAX_SOURCE_BYTES);
        reader.maximum_partial_read = SOURCE_READ_BUFFER_BYTES - 1;
        reader.interrupt_body_once = true;
        reader.interrupt_probe_once = true;

        let bytes = read_bounded(&mut reader).unwrap();

        assert_eq!(bytes.len(), MAX_SOURCE_BYTES);
        assert_eq!(bytes.capacity(), MAX_SOURCE_BYTES);
        assert_eq!(reader.remaining, 0);
        assert_eq!(
            &reader.requested_buffer_lengths[reader.requested_buffer_lengths.len() - 2..],
            &[1, 1]
        );
        assert!(
            reader.requested_buffer_lengths[..reader.requested_buffer_lengths.len() - 2]
                .contains(&SOURCE_READ_BUFFER_BYTES)
        );
    }

    #[test]
    fn bounded_read_preserves_a_non_interrupted_probe_error() {
        let mut reader = InstrumentedSourceReader::new(MAX_SOURCE_BYTES);
        reader.probe_error = Some(io::ErrorKind::PermissionDenied);

        let result = read_bounded(&mut reader);

        let Err(ReadSourceError::Io(error)) = result else {
            panic!("expected the probe I/O error to be preserved");
        };
        assert_eq!(error.kind(), io::ErrorKind::PermissionDenied);
        assert_eq!(reader.requested_buffer_lengths.last(), Some(&1));
    }

    #[test]
    fn bounded_read_rejects_an_overreporting_probe_as_invalid_data() {
        let mut reader = InstrumentedSourceReader::new(MAX_SOURCE_BYTES);
        reader.overreport_probe_once = true;

        let result = read_bounded(&mut reader);

        let Err(ReadSourceError::Io(error)) = result else {
            panic!("expected the malformed probe result to be rejected");
        };
        assert_eq!(error.kind(), io::ErrorKind::InvalidData);
        assert_eq!(reader.remaining, 0);
        assert_eq!(reader.requested_buffer_lengths.last(), Some(&1));
    }

    #[test]
    fn overreporting_input_reader_is_rejected_without_output() {
        let mut input = OverReportingReader;
        let mut output = Vec::new();
        let mut error = Vec::new();

        let status = run(
            os_arguments(&["eval", "-"]),
            &mut input,
            &mut output,
            &mut error,
        );

        assert_eq!(status, COMPILATION_ERROR);
        assert_eq!(output, b"");
        assert_eq!(
            error,
            concat!(
                "error[ORC1001]: could not read source file `<stdin>`\n",
                "  = note: the operating system reported invalid data\n",
            )
            .as_bytes()
        );
    }

    #[test]
    fn evaluation_input_failure_has_a_stable_status_and_diagnostic() {
        let mut input = RejectReads::default();
        let mut output = Vec::new();
        let mut error = Vec::new();

        let status = run(
            os_arguments(&["eval", "-"]),
            &mut input,
            &mut output,
            &mut error,
        );

        assert_eq!(status, COMPILATION_ERROR);
        assert_eq!(input.attempts, 1);
        assert_eq!(output, b"");
        assert_eq!(
            error,
            concat!(
                "error[ORC1001]: could not read source file `<stdin>`\n",
                "  = note: the operating system reported an I/O error\n",
            )
            .as_bytes()
        );
    }

    #[test]
    fn evaluation_allocation_failure_has_a_stable_status_and_diagnostic() {
        let mut input = RejectReads {
            attempts: 0,
            error_kind: io::ErrorKind::OutOfMemory,
        };
        let mut output = Vec::new();
        let mut error = Vec::new();

        let status = run(
            os_arguments(&["eval", "-"]),
            &mut input,
            &mut output,
            &mut error,
        );

        assert_eq!(status, COMPILATION_ERROR);
        assert_eq!(input.attempts, 1);
        assert_eq!(output, b"");
        assert_eq!(
            error,
            concat!(
                "error[ORC1001]: could not read source file `<stdin>`\n",
                "  = note: the operating system could not allocate memory\n",
            )
            .as_bytes()
        );
    }

    #[test]
    fn source_snapshot_requires_the_exact_metadata_length() {
        let maximum = u64::try_from(MAX_SOURCE_BYTES).unwrap();

        assert!(source_read_length_matches_metadata(0, 0));
        assert!(source_read_length_matches_metadata(
            MAX_SOURCE_BYTES,
            maximum
        ));
        assert!(!source_read_length_matches_metadata(0, 1));
        assert!(!source_read_length_matches_metadata(1, 0));
    }

    #[test]
    #[cfg(unix)]
    fn unix_opened_file_identity_and_snapshot_reject_different_regular_files() {
        let crate_root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let source_path = crate_root.join("src/main.rs");
        let manifest_path = crate_root.join("Cargo.toml");
        let path_metadata = source_path.metadata().unwrap();
        let opened_metadata = File::open(&source_path).unwrap().metadata().unwrap();
        let other_metadata = File::open(manifest_path).unwrap().metadata().unwrap();

        assert!(opened_file_matches_path_metadata(
            &path_metadata,
            &opened_metadata
        ));
        assert!(!opened_file_matches_path_metadata(
            &path_metadata,
            &other_metadata
        ));
        assert!(opened_file_metadata_unchanged(
            &opened_metadata,
            &opened_metadata
        ));
        assert!(!opened_file_metadata_unchanged(
            &opened_metadata,
            &other_metadata
        ));
        assert_eq!(
            render_read_source_error(
                &RenderedSourceName::try_from_text("changed.or").unwrap(),
                ReadSourceError::ChangedDuringOpen,
            ),
            concat!(
                "error[ORC1001]: could not read source file `changed.or`\n",
                "  = note: path changed while the source file was being opened\n",
            )
        );
        assert_eq!(
            render_read_source_error(
                &RenderedSourceName::try_from_text("changed.or").unwrap(),
                ReadSourceError::ChangedDuringRead,
            ),
            concat!(
                "error[ORC1001]: could not read source file `changed.or`\n",
                "  = note: source file changed while it was being read\n",
            )
        );
    }

    #[test]
    #[cfg(all(
        target_os = "linux",
        any(target_arch = "x86_64", target_arch = "aarch64")
    ))]
    fn linux_hardened_source_open_rejects_a_final_symlink() {
        use std::os::unix::fs::symlink;

        let crate_root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let target = crate_root.join("src/main.rs");
        assert!(File::open(&target).is_ok());

        let test_root = unix_test_root();
        let mut temporary = None;
        for suffix in 0..1_024 {
            let path = test_root.join(format!(
                "orangec-source-open-symlink-{}-{suffix}.or",
                std::process::id()
            ));
            match symlink(&target, &path) {
                Ok(()) => {
                    temporary = Some(path);
                    break;
                }
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
                Err(error) => panic!("could not create source-open test symlink: {error}"),
            }
        }
        let path = temporary.expect("could not allocate a source-open test symlink name");

        let result = open_source_file(&path);

        std::fs::remove_file(path).unwrap();
        assert!(result.is_err());
    }

    #[test]
    #[cfg(unix)]
    fn unix_opened_file_snapshot_rejects_a_same_inode_size_change() {
        use std::os::unix::fs::MetadataExt as _;

        let test_root = unix_test_root();
        let mut temporary = None;
        for suffix in 0..1_024 {
            let path = test_root.join(format!(
                "orangec-source-snapshot-{}-{suffix}.or",
                std::process::id()
            ));
            match std::fs::OpenOptions::new()
                .create_new(true)
                .read(true)
                .write(true)
                .open(&path)
            {
                Ok(file) => {
                    temporary = Some((path, file));
                    break;
                }
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
                Err(error) => panic!("could not create snapshot test file: {error}"),
            }
        }
        let (path, mut file) = temporary.expect("could not allocate a snapshot test file name");
        file.write_all(b"a").unwrap();
        let opened_metadata = file.metadata().unwrap();
        file.write_all(b"b").unwrap();
        let closed_metadata = file.metadata().unwrap();

        assert_eq!(opened_metadata.ino(), closed_metadata.ino());
        assert_ne!(opened_metadata.len(), closed_metadata.len());
        assert!(!opened_file_metadata_unchanged(
            &opened_metadata,
            &closed_metadata
        ));
        assert!(!opened_file_matches_path_metadata(
            &opened_metadata,
            &closed_metadata
        ));

        drop(file);
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    #[cfg(unix)]
    fn unix_source_path_drift_after_read_is_rejected() {
        let test_root = unix_test_root();
        for mutation in ["deletion", "non_regular", "replacement"] {
            let mut temporary = None;
            for suffix in 0..1_024 {
                let path = test_root.join(format!(
                    "orangec-source-path-{}-{suffix}",
                    std::process::id()
                ));
                match std::fs::create_dir(&path) {
                    Ok(()) => {
                        temporary = Some(path);
                        break;
                    }
                    Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
                    Err(error) => panic!("could not create source path test directory: {error}"),
                }
            }
            let directory = temporary.expect("could not allocate a source path test directory");
            let source = directory.join("source.or");
            let replacement = directory.join("replacement.or");
            std::fs::write(&source, b"edition 2026; module original {}").unwrap();
            if mutation == "replacement" {
                std::fs::write(&replacement, b"edition 2026; module replaced {}").unwrap();
            }
            let mut input = &b""[..];
            let mut remaining = MAX_SOURCE_BYTES_PER_INVOCATION;

            let result = read_source_with_post_read(&source, &mut input, &mut remaining, || {
                if mutation == "replacement" {
                    std::fs::rename(&replacement, &source).unwrap();
                } else if mutation == "non_regular" {
                    std::fs::remove_file(&source).unwrap();
                    std::fs::create_dir(&source).unwrap();
                } else {
                    std::fs::remove_file(&source).unwrap();
                }
            });

            assert!(matches!(result, Err(ReadSourceError::ChangedDuringRead)));
            if mutation == "replacement" {
                std::fs::remove_file(source).unwrap();
            } else if mutation == "non_regular" {
                std::fs::remove_dir(source).unwrap();
            }
            std::fs::remove_dir(directory).unwrap();
        }
    }

    #[test]
    #[cfg(unix)]
    fn unix_source_metadata_drift_after_read_is_rejected() {
        use std::os::unix::fs::{MetadataExt as _, PermissionsExt as _};
        use std::time::SystemTime;

        let test_root = unix_test_root();
        for mutation in ["hardlink", "mode", "rewrite"] {
            let mut temporary = None;
            for suffix in 0..1_024 {
                let path = test_root.join(format!(
                    "orangec-source-metadata-{}-{suffix}",
                    std::process::id()
                ));
                match std::fs::create_dir(&path) {
                    Ok(()) => {
                        temporary = Some(path);
                        break;
                    }
                    Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
                    Err(error) => {
                        panic!("could not create source metadata test directory: {error}")
                    }
                }
            }
            let directory = temporary.expect("could not allocate a source metadata test directory");
            let source = directory.join("source.or");
            let alias = directory.join("alias.or");
            std::fs::write(&source, b"before").unwrap();
            File::open(&source)
                .unwrap()
                .set_times(std::fs::FileTimes::new().set_modified(SystemTime::UNIX_EPOCH))
                .unwrap();
            let initial_metadata = source.metadata().unwrap();
            let mut input = &b""[..];
            let mut remaining = MAX_SOURCE_BYTES_PER_INVOCATION;

            let result = read_source_with_post_read(&source, &mut input, &mut remaining, || {
                if mutation == "hardlink" {
                    std::fs::hard_link(&source, &alias).unwrap();
                } else if mutation == "mode" {
                    let mut permissions = source.metadata().unwrap().permissions();
                    permissions.set_mode(permissions.mode() ^ 0o100);
                    std::fs::set_permissions(&source, permissions).unwrap();
                } else {
                    std::fs::write(&source, b"after!").unwrap();
                }
            });
            let final_metadata = source.metadata().unwrap();

            assert_eq!(initial_metadata.ino(), final_metadata.ino());
            assert_eq!(initial_metadata.len(), final_metadata.len());
            assert!(!opened_file_matches_path_metadata(
                &initial_metadata,
                &final_metadata
            ));
            if mutation == "hardlink" {
                assert_eq!(initial_metadata.mtime(), final_metadata.mtime());
                assert_eq!(initial_metadata.mtime_nsec(), final_metadata.mtime_nsec());
                assert_ne!(initial_metadata.nlink(), final_metadata.nlink());
                std::fs::remove_file(alias).unwrap();
            } else if mutation == "mode" {
                assert_eq!(initial_metadata.mtime(), final_metadata.mtime());
                assert_eq!(initial_metadata.mtime_nsec(), final_metadata.mtime_nsec());
                assert_ne!(initial_metadata.mode(), final_metadata.mode());
            }
            assert!(matches!(result, Err(ReadSourceError::ChangedDuringRead)));
            std::fs::remove_file(source).unwrap();
            std::fs::remove_dir(directory).unwrap();
        }
    }

    #[test]
    fn evaluation_non_regular_source_has_a_stable_status_and_diagnostic() {
        let mut input = &b""[..];
        let mut output = Vec::new();
        let mut error = Vec::new();

        let status = run(
            os_arguments(&["eval", "."]),
            &mut input,
            &mut output,
            &mut error,
        );

        assert_eq!(status, COMPILATION_ERROR);
        assert_eq!(output, b"");
        assert_eq!(
            error,
            concat!(
                "error[ORC1001]: could not read source file `.`\n",
                "  = note: path does not name a regular file\n",
            )
            .as_bytes()
        );
    }

    #[test]
    fn evaluation_output_failure_has_a_stable_status_and_diagnostic() {
        let source = b"edition 2026; module values { spec answer() -> Int { 42 } }\n";
        let mut output = RejectWrites(io::ErrorKind::Other);

        let (status, error) = run_evaluation_with_output(source, &mut output);

        assert_eq!(status, COMPILATION_ERROR);
        assert_eq!(error, b"orangec: could not write evaluation output\n");
    }

    #[test]
    fn evaluation_output_failure_is_not_retried_during_teardown() {
        let source = b"edition 2026; module values { spec answer() -> Int { 42 } }\n";
        let mut output = FailFirstWrite::default();

        let (status, error) = run_evaluation_with_output(source, &mut output);

        assert_eq!(status, COMPILATION_ERROR);
        assert_eq!(error, b"orangec: could not write evaluation output\n");
        assert_eq!(output.attempts, 1);
        assert_eq!(output.bytes, b"");
    }

    #[test]
    fn zero_length_output_writes_are_rejected_across_output_classes() {
        let mut input = RejectReads::default();
        let mut output = ZeroWrites::default();
        let mut error = Vec::new();
        let status = run(
            os_arguments(&["--help"]),
            &mut input,
            &mut output,
            &mut error,
        );

        assert_eq!(status, COMPILATION_ERROR);
        assert_eq!(input.attempts, 0);
        assert_eq!(output.attempts, 1);
        assert_eq!(error, b"");

        let mut input = RejectReads::default();
        let mut output = Vec::new();
        let mut error = ZeroWrites::default();
        let status = run(
            os_arguments(&["unknown", "source.or"]),
            &mut input,
            &mut output,
            &mut error,
        );

        assert_eq!(status, COMPILATION_ERROR);
        assert_eq!(input.attempts, 0);
        assert_eq!(output, b"");
        assert_eq!(error.attempts, 1);

        let mut input = b"module caf\xc3\xa9 {}\n".as_slice();
        let mut output = Vec::new();
        let mut error = ZeroWrites::default();
        let status = run(
            os_arguments(&["check", "-"]),
            &mut input,
            &mut output,
            &mut error,
        );

        assert_eq!(status, COMPILATION_ERROR);
        assert_eq!(output, b"");
        assert_eq!(error.attempts, 1);

        let source = b"edition 2026; module values { spec answer() -> Int { 42 } }\n";
        let mut output = ZeroWrites::default();

        let (status, error) = run_evaluation_with_output(source, &mut output);

        assert_eq!(status, COMPILATION_ERROR);
        assert_eq!(error, b"orangec: could not write evaluation output\n");
        assert_eq!(output.attempts, 1);

        let mut input = b"edition 2026; module values {}\n".as_slice();
        let mut output = ZeroWrites::default();
        let mut error = Vec::new();
        let status = run(
            os_arguments(&["lex", "-"]),
            &mut input,
            &mut output,
            &mut error,
        );

        assert_eq!(status, COMPILATION_ERROR);
        assert_eq!(output.attempts, 1);
        assert_eq!(error, b"orangec: could not write token output\n");
    }

    #[test]
    fn evaluation_output_partial_write_failure_preserves_only_the_accepted_prefix() {
        let source = b"edition 2026; module values { spec answer() -> Int { 42 } }\n";
        let mut output = AcceptPrefixThenFail::new(2, io::ErrorKind::Other);

        let (status, error) = run_evaluation_with_output(source, &mut output);

        assert_eq!(status, COMPILATION_ERROR);
        assert_eq!(output.bytes, b"va");
        assert_eq!(output.attempts, 2);
        assert_eq!(error, b"orangec: could not write evaluation output\n");
    }

    #[test]
    fn evaluation_output_partial_broken_pipe_is_quiet_and_not_retried() {
        let source = b"edition 2026; module values { spec answer() -> Int { 42 } }\n";
        let mut output = AcceptPrefixThenFail::new(2, io::ErrorKind::BrokenPipe);

        let (status, error) = run_evaluation_with_output(source, &mut output);

        assert_eq!(status, COMPILATION_ERROR);
        assert_eq!(output.bytes, b"va");
        assert_eq!(output.attempts, 2);
        assert_eq!(error, b"");
    }

    #[test]
    fn evaluation_output_flush_failure_reports_failure_after_complete_bytes() {
        let source = b"edition 2026; module values { spec answer() -> Int { 42 } }\n";
        let mut output = FailFlush::default();

        let (status, error) = run_evaluation_with_output(source, &mut output);

        assert_eq!(status, COMPILATION_ERROR);
        assert_eq!(output.bytes, b"values::answer: Int = 42\n");
        assert_eq!(output.flush_attempts, 1);
        assert_eq!(error, b"orangec: could not write evaluation output\n");
    }

    #[test]
    fn token_output_flush_failure_reports_failure_after_complete_bytes() {
        let source = b"edition 2026; module values {}\n";
        let (expected_status, expected_output, expected_error) =
            run_with_source_bytes("lex", source);
        assert_eq!(expected_status, SUCCESS);
        assert_eq!(expected_error, b"");

        let mut input = source.as_slice();
        let mut output = FailFlush::default();
        let mut error = Vec::new();
        let status = run(
            os_arguments(&["lex", "-"]),
            &mut input,
            &mut output,
            &mut error,
        );

        assert_eq!(status, COMPILATION_ERROR);
        assert_eq!(output.bytes, expected_output);
        assert_eq!(output.flush_attempts, 1);
        assert_eq!(error, b"orangec: could not write token output\n");
    }

    #[test]
    fn evaluation_broken_pipe_is_a_quiet_failure() {
        let source = b"edition 2026; module values { spec answer() -> Int { 42 } }\n";
        let mut output = RejectWrites(io::ErrorKind::BrokenPipe);

        let (status, error) = run_evaluation_with_output(source, &mut output);

        assert_eq!(status, COMPILATION_ERROR);
        assert_eq!(error, b"");
    }

    #[test]
    fn token_output_broken_pipe_is_a_quiet_failure() {
        let mut input = b"edition 2026; module values {}\n".as_slice();
        let mut output = RejectWrites(io::ErrorKind::BrokenPipe);
        let mut error = Vec::new();

        let status = run(
            os_arguments(&["lex", "-"]),
            &mut input,
            &mut output,
            &mut error,
        );

        assert_eq!(status, COMPILATION_ERROR);
        assert_eq!(error, b"");
    }

    #[test]
    fn token_output_failure_stops_before_reading_later_sources() {
        // The spelling is larger than BufWriter's buffer, so the rejecting
        // destination is reached during this source rather than at final
        // flush after every operand has already been processed.
        let source = "a".repeat(16 * 1024);
        let mut input = source.as_bytes();
        let mut output = RejectWrites(io::ErrorKind::Other);
        let mut error = Vec::new();

        let status = run(
            os_arguments(&["lex", "-", "."]),
            &mut input,
            &mut output,
            &mut error,
        );

        assert_eq!(status, COMPILATION_ERROR);
        assert_eq!(error, b"orangec: could not write token output\n");
    }

    #[test]
    fn diagnostic_output_failure_stops_before_reading_later_sources() {
        let mut input = RejectReads::default();
        let mut output = Vec::new();
        let mut error = RejectWrites(io::ErrorKind::Other);

        let status = run(
            os_arguments(&["check", ".", "-"]),
            &mut input,
            &mut output,
            &mut error,
        );

        assert_eq!(status, COMPILATION_ERROR);
        assert_eq!(input.attempts, 0);
        assert_eq!(output, b"");
    }

    #[test]
    fn diagnostic_output_failure_discards_buffered_token_output() {
        let mut input = b"@".as_slice();
        let mut output = Vec::new();
        let mut error = RejectWrites(io::ErrorKind::Other);

        let status = run(
            os_arguments(&["lex", "-"]),
            &mut input,
            &mut output,
            &mut error,
        );

        assert_eq!(status, COMPILATION_ERROR);
        assert_eq!(output, b"");
    }

    #[test]
    fn diagnostic_flush_failure_discards_buffered_token_output() {
        let mut input = b"@".as_slice();
        let mut output = Vec::new();
        let mut error = FailFlush::default();

        let status = run(
            os_arguments(&["lex", "-"]),
            &mut input,
            &mut output,
            &mut error,
        );

        assert_eq!(status, COMPILATION_ERROR);
        assert_eq!(output, b"");
        assert_eq!(error.flush_attempts, 1);
        assert!(error.bytes.starts_with(b"error[ORC0001]:"));
    }

    #[test]
    fn token_flush_failure_reflushes_its_diagnostic_after_prior_diagnostics() {
        let mut input = b"@".as_slice();
        let mut output = FailFlush::default();
        let mut error = InterruptFirstFlush::default();

        let status = run(
            os_arguments(&["lex", "-"]),
            &mut input,
            &mut output,
            &mut error,
        );

        assert_eq!(status, COMPILATION_ERROR);
        assert_eq!(output.flush_attempts, 1);
        assert!(!output.bytes.is_empty());
        assert_eq!(error.flush_attempts, 3);
        let error = String::from_utf8(error.bytes).unwrap();
        assert!(error.starts_with("error[ORC0001]:"));
        assert!(error.ends_with("\n\norangec: could not write token output\n"));
    }

    #[test]
    fn partial_diagnostic_output_failure_is_not_retried_or_followed_by_input() {
        let mut input = RejectReads::default();
        let mut output = Vec::new();
        let mut error = AcceptPrefixThenFail::new(3, io::ErrorKind::Other);

        let status = run(
            os_arguments(&["check", ".", "-"]),
            &mut input,
            &mut output,
            &mut error,
        );

        assert_eq!(status, COMPILATION_ERROR);
        assert_eq!(input.attempts, 0);
        assert_eq!(output, b"");
        assert_eq!(error.bytes, b"err");
        assert_eq!(error.attempts, 2);
    }

    #[test]
    fn direct_large_evaluation_write_failure_is_reported() {
        let module = "m".repeat(16 * 1024);
        let source = format!("edition 2026; module {module} {{ spec answer() -> Int {{ 42 }} }}\n");
        let mut output = RejectWrites(io::ErrorKind::Other);

        let (status, error) = run_evaluation_with_output(source.as_bytes(), &mut output);

        assert_eq!(status, COMPILATION_ERROR);
        assert_eq!(error, b"orangec: could not write evaluation output\n");
    }

    #[test]
    fn evaluation_output_is_streamed_by_value() {
        let module = "m".repeat(16 * 1024);
        let source = format!(
            "edition 2026; module {module} {{ \
             spec first() -> Int {{ 1 }} spec second() -> Int {{ 2 }} }}\n"
        );
        let expected_bytes = format!("{module}::first: Int = 1\n{module}::second: Int = 2\n").len();
        let mut output = MeasureWrites::default();

        let (status, error) = run_evaluation_with_output(source.as_bytes(), &mut output);

        assert_eq!(status, SUCCESS);
        assert_eq!(error, b"");
        assert_eq!(output.bytes, expected_bytes);
        assert!(output.writes >= 2);
        assert!(output.largest_write < output.bytes);
    }

    #[test]
    fn escaped_token_spelling_is_streamed_in_bounded_chunks() {
        let spelling = "\u{80}".repeat(TOKEN_ESCAPE_BUFFER_BYTES);
        let escaped_character_bytes = '\u{80}'.escape_default().count();
        let mut output = MeasureWrites::default();

        write_escaped_token_spelling(&mut output, &spelling).unwrap();

        assert_eq!(
            output.bytes,
            TOKEN_ESCAPE_BUFFER_BYTES * escaped_character_bytes
        );
        assert!(output.writes > 1);
        assert!(output.largest_write <= TOKEN_ESCAPE_BUFFER_BYTES);
    }

    #[test]
    fn streamed_token_spelling_matches_canonical_escape_bytes() {
        let spelling = "\"ascii\t\\é\u{1b}\u{202e}";
        let expected: String = spelling.chars().flat_map(char::escape_default).collect();
        let mut output = Vec::new();

        write_escaped_token_spelling(&mut output, spelling).unwrap();

        assert_eq!(output, expected.as_bytes());
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
