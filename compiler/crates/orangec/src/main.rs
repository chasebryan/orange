//! `orangec`, the pre-alpha Orange compiler command-line frontend.

use std::borrow::Cow;
use std::env;
use std::ffi::{OsStr, OsString};
use std::fmt::{self, Write as _};
use std::fs::{File, Metadata};
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use orange_compiler::{
    Diagnostic, DiagnosticCode, Edition, Lexed, MAX_EVALUATION_STEPS_PER_SOURCE,
    MAX_MODULES_PER_PROGRAM, MAX_SOURCE_BYTES, RenderedSourceName, SourceError, SourceFile,
    SourceId, SourceMap, SyntaxTree, analyze_program, evaluate_selected, lex, parse,
    render_diagnostics,
};

mod crypt;

const SUCCESS: u8 = 0;
const COMPILATION_ERROR: u8 = 1;
const USAGE_ERROR: u8 = 2;
const MAX_SOURCES_PER_INVOCATION: usize = 256;
const MAX_ARGUMENT_BYTES_PER_INVOCATION: usize = 4 * 1024 * 1024;
const MAX_SOURCE_BYTES_PER_INVOCATION: usize = 64 * 1024 * 1024;
const MAX_STANDARD_OUTPUT_BYTES: usize = 64 * 1024 * 1024;
const MAX_STANDARD_ERROR_BYTES: usize = 64 * 1024 * 1024;
const MAX_CONSECUTIVE_INTERRUPTED_IO_ATTEMPTS: usize = 1_024;
const SOURCE_READ_BUFFER_BYTES: usize = 8 * 1024;
/// The most steps `eval --steps` admits: 1,024 times the default budget.
const MAX_EVALUATION_STEP_LIMIT: usize = 1 << 30;
/// The most functions one `eval` names with `--spec`.
const MAX_SELECTED_SPECS: usize = 64;
const TOKEN_ESCAPE_BUFFER_BYTES: usize = 4 * 1024;
const USAGE: &str = concat!(
    "Usage: orangec [OPTIONS] <check|eval|lex> <FILE>...\n",
    "       orangec eval [--steps <N>] [--spec <NAME>]... [--stats] <FILE>\n",
    "       orangec keygen [--scheme <NAME>] [-o <FILE>]\n",
    "       orangec <enc|dec> [--key <FILE>] [--scheme <NAME>] [-o <FILE>] <FILE>\n",
    "       orangec schemes [<NAME>...]\n",
    "\n",
    "Commands:\n",
    "  check    Perform lexical, syntactic, and semantic validation\n",
    "  eval     Reference-evaluate one source after complete validation\n",
    "  lex      Print the deterministic token stream\n",
    "  keygen   Make a secret key for a scheme [default: xchacha20_poly1305]\n",
    "  enc      Seal a file with the scheme its key belongs to\n",
    "  dec      Open a sealed file, writing nothing unless all of it is authentic\n",
    "  schemes  List the built-in sealing schemes, or describe the named ones\n",
    "\n",
    "Options:\n",
    "      --edition <YEAR>  Select the Orange edition [default: 2026; at most once]\n",
    "      --steps <N>       Evaluation step budget, 1 to 1073741824 [default: 1048576]\n",
    "      --spec <NAME>     Evaluate only this function without parameters; repeatable\n",
    "      --stats           Report the steps each evaluated function used, on stderr\n",
    "      --scheme <NAME>   Scheme: a built-in name or an Orange program's path\n",
    "      --key <FILE>      Key file [default: $XDG_CONFIG_HOME/orange/key]\n",
    "  -o, --output <FILE>   Output path [default: FILE.orange; dec strips .orange]\n",
    "      --                End option parsing\n",
    "  -h, --help            Print help\n",
    "  -V, --version         Print version\n",
    "\n",
    "Use `-` as a file name to read UTF-8 source from standard input. Sealing runs\n",
    "Orange programs on the reference evaluator, which is not constant-time; the\n",
    "schemes are reference code and are not verified.\n",
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
    KeyFile => "ORC1009",
    Scheme => "ORC1010",
    CryptInput => "ORC1011",
    CryptOutput => "ORC1012",
    SealedFile => "ORC1013",
    NotAuthentic => "ORC1014",
    Randomness => "ORC1015",
    EntryPoint => "ORC1016",
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
    interrupted_writes: usize,
}

impl<W> CountCheckedWriter<W> {
    const fn new(inner: W) -> Self {
        Self {
            inner,
            interrupted_writes: 0,
        }
    }
}

impl<W: Write> Write for CountCheckedWriter<W> {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        match self.inner.write(buffer) {
            Err(error) if error.kind() == io::ErrorKind::Interrupted => {
                if record_interrupted_attempt(&mut self.interrupted_writes) {
                    Err(interrupted_io_limit_error())
                } else {
                    Err(error)
                }
            }
            Err(error) => {
                self.interrupted_writes = 0;
                Err(error)
            }
            Ok(written) => {
                self.interrupted_writes = 0;
                if written > buffer.len() {
                    Err(invalid_data_error())
                } else {
                    Ok(written)
                }
            }
        }
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

#[derive(Debug)]
struct InterruptedIoLimitExceeded;

impl fmt::Display for OutputLimitExceeded {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("standard output limit exceeded")
    }
}

impl std::error::Error for OutputLimitExceeded {}

impl fmt::Display for InterruptedIoLimitExceeded {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("consecutive interrupted I/O attempt limit exceeded")
    }
}

impl std::error::Error for InterruptedIoLimitExceeded {}

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

fn interrupted_io_limit_error() -> io::Error {
    io::Error::other(InterruptedIoLimitExceeded)
}

fn record_interrupted_attempt(attempts: &mut usize) -> bool {
    *attempts = attempts.saturating_add(1);
    *attempts >= MAX_CONSECUTIVE_INTERRUPTED_IO_ATTEMPTS
}

fn flush_retry_interrupted(output: &mut impl Write) -> io::Result<()> {
    let mut interrupted_attempts = 0;
    loop {
        match output.flush() {
            Err(error) if error.kind() == io::ErrorKind::Interrupted => {
                if record_interrupted_attempt(&mut interrupted_attempts) {
                    return Err(interrupted_io_limit_error());
                }
            }
            result => return result,
        }
    }
}

fn seek_start_retry_interrupted(stream: &mut impl Seek) -> io::Result<u64> {
    let mut interrupted_attempts = 0;
    loop {
        match stream.seek(SeekFrom::Start(0)) {
            Err(error) if error.kind() == io::ErrorKind::Interrupted => {
                if record_interrupted_attempt(&mut interrupted_attempts) {
                    return Err(interrupted_io_limit_error());
                }
            }
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
        Action::Seal(options) => crypt::run(&options, &mut standard_output, &mut standard_error),
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
            let modules = if ast.module().uses().is_empty() {
                Vec::new()
            } else {
                match load_used_modules(
                    path,
                    ast,
                    &mut sources,
                    options.edition,
                    &mut remaining_source_bytes,
                ) {
                    Ok(modules) => modules,
                    Err(group) => {
                        compilation_failed = true;
                        emit_error_group(
                            standard_error,
                            &mut standard_error_available,
                            &mut error_group_written,
                            &mut output_failed,
                            &group,
                        );
                        continue;
                    }
                }
            };
            let program = modules
                .iter()
                .filter_map(|(id, ast)| Some((sources.get(*id)?, ast)))
                .collect::<Vec<_>>();
            let Some(source) = sources.get(id).filter(|_| program.len() == modules.len()) else {
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
            let analyzed = analyze_program((source, ast), &program);
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
                let evaluation = &options.evaluation;
                if let Some(missing) = unmatched_spec(core, &evaluation.specs) {
                    compilation_failed = true;
                    emit_error_group(
                        standard_error,
                        &mut standard_error_available,
                        &mut error_group_written,
                        &mut output_failed,
                        &render_cli_error(
                            CliDiagnosticCode::EntryPoint,
                            format_args!(
                                "module `{}` has no function `{missing}` without parameters",
                                core.name()
                            ),
                            "`--spec` names a function of the evaluated module that takes no \
                             parameters; a function with sizes is evaluated in every instance",
                        ),
                    );
                    continue;
                }
                let evaluated = evaluate_selected(core, evaluation.steps, |function| {
                    evaluation.specs.is_empty()
                        || evaluation
                            .specs
                            .iter()
                            .any(|spec| spec.as_str() == function.name())
                });
                let values = match classify_phase_result(
                    evaluated.values(),
                    evaluated.diagnostics(),
                ) {
                    PhaseResult::Complete(values) => values,
                    PhaseResult::Diagnosed(diagnostics) => {
                        compilation_failed = true;
                        let hinted = with_budget_hint(diagnostics, evaluation.steps);
                        emit_error_group(
                            standard_error,
                            &mut standard_error_available,
                            &mut error_group_written,
                            &mut output_failed,
                            &render_diagnostics(&sources, hinted.as_deref().unwrap_or(diagnostics)),
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
                // The report follows the values: they are committed first, so
                // that a terminal shows them in that order, and a failure to
                // commit them writes no report.
                if evaluation.stats
                    && standard_output_available
                    && standard_output_written
                    && let Err(error) = flush_retry_interrupted(&mut buffered_output)
                {
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
                if evaluation.stats && standard_output_available {
                    emit_error_group(
                        standard_error,
                        &mut standard_error_available,
                        &mut error_group_written,
                        &mut output_failed,
                        &render_steps(values, evaluation.steps),
                    );
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

/// Adds to each step-limit diagnostic the option that raises the budget,
/// while `budget` is below the most `--steps` admits; `None` when the copy
/// cannot be allocated.
fn with_budget_hint(diagnostics: &[Diagnostic], budget: usize) -> Option<Vec<Diagnostic>> {
    let mut hinted = Vec::new();
    hinted.try_reserve_exact(diagnostics.len()).ok()?;
    for diagnostic in diagnostics {
        let steps = diagnostic.code() == DiagnosticCode::EvaluationResourceLimit
            && diagnostic.message() == "reference evaluation step limit exceeded";
        hinted.push(if steps && budget < MAX_EVALUATION_STEP_LIMIT {
            diagnostic.clone().with_note(format!(
                "`orangec eval --steps N` sets the budget, up to {MAX_EVALUATION_STEP_LIMIT} steps"
            ))
        } else {
            diagnostic.clone()
        });
    }
    Some(hinted)
}

/// Returns the first of `specs` that names no function of `core`'s root
/// module without parameters.
fn unmatched_spec<'spec>(
    core: &orange_compiler::CoreModule,
    specs: &'spec [String],
) -> Option<&'spec str> {
    specs
        .iter()
        .find(|spec| {
            !core.entry_functions().iter().any(|function| {
                function.parameters().is_empty() && function.name() == spec.as_str()
            })
        })
        .map(String::as_str)
}

/// Reports the steps each evaluated function used, one line each in the
/// order printed, and their total against the budget.
fn render_steps(values: &[orange_compiler::EvaluatedFunction], budget: usize) -> String {
    let mut report = String::new();
    let mut total = 0_usize;
    for value in values {
        total = total.saturating_add(value.steps());
        let unit = if value.steps() == 1 { "step" } else { "steps" };
        let _ = writeln!(
            report,
            "{}::{}{}: {} {unit}",
            value.module(),
            value.name(),
            value.instance(),
            value.steps()
        );
    }
    let _ = writeln!(report, "total: {total} of {budget} steps");
    report
}

/// Reads, lexes, and parses the modules that `root` uses, directly or
/// through other modules.
///
/// The module `NAME` of a `use NAME;` declaration is read from the file
/// `NAME.or` in the directory of the root file, or the current directory for
/// standard input; a module name is an ASCII identifier, so it names a file
/// in that directory and nothing outside it. Each module is read once, in the
/// order in which a `use` first names it, and every read is charged to the
/// invocation's source budget. The uses of a file that declares a module of
/// another name are not followed. At most one module more than a program may
/// hold is read, so that semantic analysis reports the limit. A module that
/// cannot be read, lexed, or parsed stops the program with its diagnostics.
fn load_used_modules(
    root_path: &Path,
    root: &SyntaxTree,
    sources: &mut SourceMap,
    edition: Edition,
    remaining_source_bytes: &mut usize,
) -> Result<Vec<(SourceId, SyntaxTree)>, String> {
    let directory = if root_path == Path::new("-") {
        Path::new("")
    } else {
        root_path.parent().unwrap_or_else(|| Path::new(""))
    };
    let root_name = root.module().name().text();
    let mut requested: Vec<String> = Vec::new();
    let mut loaded: Vec<(SourceId, SyntaxTree)> = Vec::new();
    // Module 0 is the root; module `n` is `loaded[n - 1]`.
    let mut next_module = 0_usize;
    loop {
        let uses = if next_module == 0 {
            Some(root.module().uses())
        } else {
            // A file that declares a module of another name stays loaded, so
            // that the module graph reports the `use` that read it, but its
            // own uses name no module of this program and are not followed.
            next_module
                .checked_sub(1)
                .and_then(|index| loaded.get(index).zip(requested.get(index)))
                .map(|((_, ast), requested)| {
                    if ast.module().name().text() == requested {
                        ast.module().uses()
                    } else {
                        &[]
                    }
                })
        };
        let Some(uses) = uses else {
            return Ok(loaded);
        };
        let user = if next_module == 0 {
            root_name
        } else {
            next_module
                .checked_sub(1)
                .and_then(|index| loaded.get(index))
                .map_or("", |(_, ast)| ast.module().name().text())
        }
        .to_owned();
        let names = uses
            .iter()
            .map(|declaration| declaration.name().text())
            .filter(|name| *name != root_name && !requested.iter().any(|seen| seen == name))
            .map(str::to_owned)
            .collect::<Vec<_>>();
        next_module = next_module.saturating_add(1);
        for name in names {
            if requested.contains(&name) {
                continue;
            }
            if loaded.len() >= MAX_MODULES_PER_PROGRAM {
                return Ok(loaded);
            }
            let path = directory.join(format!("{name}.or"));
            let module = load_module(&path, sources, edition, remaining_source_bytes).map_err(
                |mut group| {
                    if group.starts_with("error[ORC1") {
                        let _ = writeln!(
                            group,
                            "  = note: `use {name};` in module `{user}` reads the module `{name}` \
                             from this file"
                        );
                    }
                    group
                },
            )?;
            requested.push(name);
            loaded.push(module);
        }
    }
}

/// Reads, lexes, and parses one used module.
fn load_module(
    path: &Path,
    sources: &mut SourceMap,
    edition: Edition,
    remaining_source_bytes: &mut usize,
) -> Result<(SourceId, SyntaxTree), String> {
    let display_name = stable_source_name(path).map_err(source_name_error)?;
    // A used module is always a file, never standard input.
    let bytes = read_source(path, &mut io::empty(), remaining_source_bytes)
        .map_err(|error| render_read_source_error(&display_name, error))?;
    let text = String::from_utf8(bytes).map_err(|error| {
        render_cli_error(
            CliDiagnosticCode::InvalidUtf8,
            format_args!("source file `{display_name}` is not valid UTF-8"),
            format_args!(
                "invalid byte sequence begins at byte offset {}",
                error.utf8_error().valid_up_to()
            ),
        )
    })?;
    let id = sources
        .add_with_rendered_name(display_name, text)
        .map_err(source_limit_error_without_name)?;
    let missing = || {
        render_cli_error(
            CliDiagnosticCode::MissingPhaseArtifact,
            "source insertion succeeded without a retrievable source",
            "this is an internal compiler failure",
        )
    };
    let source = sources.get(id).ok_or_else(missing)?;
    let lexed = lex(source, edition);
    if lexed.has_errors() {
        return Err(if lexed.diagnostics().is_empty() {
            render_cli_error(
                CliDiagnosticCode::MissingPhaseArtifact,
                "lexical analysis failed without a diagnostic",
                "this is an internal compiler resource failure",
            )
        } else {
            render_diagnostics(sources, lexed.diagnostics())
        });
    }
    let parsed = parse(source, &lexed);
    if !parsed.diagnostics().is_empty() {
        return Err(render_diagnostics(sources, parsed.diagnostics()));
    }
    let ast = parsed.into_ast().ok_or_else(|| {
        render_cli_error(
            CliDiagnosticCode::MissingPhaseArtifact,
            "parser returned neither a complete syntax tree nor a diagnostic",
            "this is an internal compiler or resource failure",
        )
    })?;
    Ok((id, ast))
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
        if !source_descriptor_matches_snapshot(&mut file, &bytes).map_err(ReadSourceError::Io)? {
            return Err(ReadSourceError::ChangedDuringRead);
        }
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

fn source_descriptor_matches_snapshot(
    reader: &mut (impl Read + Seek),
    expected: &[u8],
) -> io::Result<bool> {
    if seek_start_retry_interrupted(reader)? != 0 {
        return Err(invalid_data_error());
    }

    let mut buffer = [0_u8; SOURCE_READ_BUFFER_BYTES];
    let mut verified = 0_usize;
    while verified < expected.len() {
        let remaining = expected
            .len()
            .checked_sub(verified)
            .ok_or_else(invalid_data_error)?;
        let buffer_length = remaining.min(buffer.len());
        let destination = buffer
            .get_mut(..buffer_length)
            .ok_or_else(invalid_data_error)?;
        let read = read_retry_interrupted(reader, destination)?;
        if read == 0 {
            return Ok(false);
        }
        let chunk = destination.get(..read).ok_or_else(invalid_data_error)?;
        let end = verified
            .checked_add(chunk.len())
            .ok_or_else(invalid_data_error)?;
        let expected_chunk = expected.get(verified..end).ok_or_else(invalid_data_error)?;
        if chunk != expected_chunk {
            return Ok(false);
        }
        verified = end;
    }

    let mut probe = [0_u8; 1];
    match read_retry_interrupted(reader, &mut probe)? {
        0 => Ok(true),
        1 => Ok(false),
        _ => Err(invalid_data_error()),
    }
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
        let read = read_retry_interrupted(&mut reader, buffer).map_err(ReadSourceError::Io)?;
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
    match read_retry_interrupted(&mut reader, &mut probe) {
        Ok(0) => Ok(bytes),
        Ok(1) => {
            if let Some(remaining) = remaining_source_bytes.checked_sub(1) {
                *remaining_source_bytes = remaining;
            }
            Err(exceeded)
        }
        Ok(_) => Err(ReadSourceError::Io(invalid_data_error())),
        Err(error) => Err(ReadSourceError::Io(error)),
    }
}

fn read_retry_interrupted(reader: &mut impl Read, buffer: &mut [u8]) -> io::Result<usize> {
    let mut interrupted_attempts = 0;
    loop {
        match reader.read(buffer) {
            Err(error) if error.kind() == io::ErrorKind::Interrupted => {
                if record_interrupted_attempt(&mut interrupted_attempts) {
                    return Err(interrupted_io_limit_error());
                }
            }
            result => return result,
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
            io_error_reason(&error),
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

fn io_error_reason(error: &io::Error) -> &'static str {
    if error
        .get_ref()
        .is_some_and(|cause| cause.is::<InterruptedIoLimitExceeded>())
    {
        return "source stream remained interrupted for 1,024 consecutive attempts";
    }
    match error.kind() {
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
    Keygen => "keygen",
    Enc => "enc",
    Dec => "dec",
    Schemes => "schemes",
}

impl CompilerCommand {
    /// Returns whether this is one of the sealing commands in [`crypt`].
    const fn seals(self) -> bool {
        matches!(self, Self::Keygen | Self::Enc | Self::Dec | Self::Schemes)
    }
}

#[derive(Debug, Eq, PartialEq)]
struct Options {
    command: CompilerCommand,
    edition: Edition,
    paths: Vec<PathBuf>,
    evaluation: Evaluation,
}

/// How `eval` evaluates: its step budget, the functions it names, and
/// whether it reports the steps each used.
#[derive(Debug, Eq, PartialEq)]
struct Evaluation {
    /// The step budget of the whole evaluation.
    steps: usize,
    /// The functions without parameters to evaluate, each named once, in
    /// the order given; empty for every one.
    specs: Vec<String>,
    /// Whether to report each function's steps and the total on stderr.
    stats: bool,
}

impl Default for Evaluation {
    fn default() -> Self {
        Self {
            steps: MAX_EVALUATION_STEPS_PER_SOURCE,
            specs: Vec::new(),
            stats: false,
        }
    }
}

impl Evaluation {
    /// Returns whether any option set this evaluation away from the default.
    fn is_default(&self) -> bool {
        *self == Self::default()
    }
}

#[derive(Debug, Eq, PartialEq)]
enum Action {
    Help,
    Version,
    Compile(Options),
    Seal(crypt::SealOptions),
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
    let mut scheme = None;
    let mut key = None;
    let mut output = None;
    let mut evaluation = Evaluation::default();
    let mut steps_seen = false;
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
                Some("--stats") => {
                    evaluation.stats = true;
                    continue;
                }
                Some(name @ ("--steps" | "--spec")) => {
                    let value = arguments
                        .next()
                        .ok_or_else(|| format!("option `{name}` requires a value"))?;
                    charge_argument_bytes(&mut remaining_argument_bytes, &value)?;
                    set_evaluation_option(name, &value, &mut evaluation, &mut steps_seen)?;
                    continue;
                }
                Some(name @ ("--scheme" | "--key" | "-o" | "--output")) => {
                    let value = arguments
                        .next()
                        .ok_or_else(|| format!("option `{name}` requires a value"))?;
                    charge_argument_bytes(&mut remaining_argument_bytes, &value)?;
                    set_sealing_option(name, value, &mut scheme, &mut key, &mut output)?;
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
                    if let Some((name @ ("--steps" | "--spec"), value)) = value.split_once('=') {
                        set_evaluation_option(
                            name,
                            OsStr::new(value),
                            &mut evaluation,
                            &mut steps_seen,
                        )?;
                        continue;
                    }
                    if let Some((name @ ("--scheme" | "--key" | "--output"), value)) =
                        value.split_once('=')
                    {
                        set_sealing_option(
                            name,
                            OsString::from(value),
                            &mut scheme,
                            &mut key,
                            &mut output,
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
                None if [
                    &b"--scheme="[..],
                    b"--key=",
                    b"--output=",
                    b"--steps=",
                    b"--spec=",
                ]
                .iter()
                .any(|prefix| argument.as_encoded_bytes().starts_with(prefix)) =>
                {
                    return Err(String::from(
                        "an option value that is not valid UTF-8 must be a separate argument",
                    ));
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
    if command != CompilerCommand::Eval && (steps_seen || !evaluation.is_default()) {
        let name = if steps_seen {
            "--steps"
        } else if evaluation.specs.is_empty() {
            "--stats"
        } else {
            "--spec"
        };
        return Err(format!("option `{name}` applies only to eval"));
    }
    if command.seals() {
        return sealing_action(command, edition, scheme, key, output, paths);
    }
    for (present, name) in [
        (scheme.is_some(), "--scheme"),
        (key.is_some(), "--key"),
        (output.is_some(), "--output"),
    ] {
        if present {
            return Err(format!(
                "option `{name}` applies only to keygen, enc, dec, and schemes"
            ));
        }
    }
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
        evaluation,
    }))
}

/// Records `--steps`, at most once, or one more `--spec` name.
fn set_evaluation_option(
    name: &str,
    value: &OsStr,
    evaluation: &mut Evaluation,
    steps_seen: &mut bool,
) -> Result<(), String> {
    if name == "--steps" {
        if *steps_seen {
            return Err(String::from(
                "option `--steps` may be specified at most once",
            ));
        }
        *steps_seen = true;
        evaluation.steps = parse_steps(value)?;
        return Ok(());
    }
    let spec = parse_spec_name(value)?;
    if evaluation.specs.contains(&spec) {
        return Ok(());
    }
    if evaluation.specs.len() >= MAX_SELECTED_SPECS {
        return Err(format!(
            "option `--spec` names at most {MAX_SELECTED_SPECS} functions"
        ));
    }
    if evaluation.specs.try_reserve(1).is_err() {
        return Err(String::from(
            "could not allocate the list of `--spec` names",
        ));
    }
    evaluation.specs.push(spec);
    Ok(())
}

/// Reads a step budget: a decimal number from 1 through
/// [`MAX_EVALUATION_STEP_LIMIT`], without sign or leading zeros.
fn parse_steps(value: &OsStr) -> Result<usize, String> {
    let invalid = || {
        format!(
            "option `--steps` takes a number of steps from 1 through {MAX_EVALUATION_STEP_LIMIT}"
        )
    };
    let text = value.to_str().ok_or_else(invalid)?;
    if text.starts_with('0') || !text.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(invalid());
    }
    text.parse::<usize>()
        .ok()
        .filter(|steps| (1..=MAX_EVALUATION_STEP_LIMIT).contains(steps))
        .ok_or_else(invalid)
}

/// Reads a function name: an Orange identifier, an ASCII letter or `_`
/// followed by ASCII letters, digits, and `_`.
fn parse_spec_name(value: &OsStr) -> Result<String, String> {
    let invalid = || String::from("option `--spec` takes the name of a function");
    let text = value.to_str().ok_or_else(invalid)?;
    let mut bytes = text.bytes();
    let starts = bytes
        .next()
        .is_some_and(|byte| byte.is_ascii_alphabetic() || byte == b'_');
    if !starts || !bytes.all(|byte| byte.is_ascii_alphanumeric() || byte == b'_') {
        return Err(invalid());
    }
    let mut name = String::new();
    if name.try_reserve_exact(text.len()).is_err() {
        return Err(String::from(
            "could not allocate the list of `--spec` names",
        ));
    }
    name.push_str(text);
    Ok(name)
}

/// Records one sealing option, each at most once.
fn set_sealing_option(
    name: &str,
    value: OsString,
    scheme: &mut Option<OsString>,
    key: &mut Option<PathBuf>,
    output: &mut Option<PathBuf>,
) -> Result<(), String> {
    let (canonical, taken) = match name {
        "--scheme" => ("--scheme", scheme.replace(value).is_some()),
        "--key" => ("--key", key.replace(PathBuf::from(value)).is_some()),
        _ => ("--output", output.replace(PathBuf::from(value)).is_some()),
    };
    if taken {
        return Err(format!(
            "option `{canonical}` may be specified at most once"
        ));
    }
    Ok(())
}

/// Checks a sealing command's operands and options.
fn sealing_action(
    command: CompilerCommand,
    edition: Edition,
    scheme: Option<OsString>,
    key: Option<PathBuf>,
    output: Option<PathBuf>,
    inputs: Vec<PathBuf>,
) -> Result<Action, String> {
    let name = command.as_str();
    match command {
        CompilerCommand::Keygen => {
            if !inputs.is_empty() {
                return Err(String::from(
                    "command `keygen` takes no file; name the new key file with -o",
                ));
            }
            if key.is_some() {
                return Err(String::from(
                    "command `keygen` writes its key to -o, not --key",
                ));
            }
        }
        CompilerCommand::Schemes => {
            if scheme.is_some() || key.is_some() || output.is_some() {
                return Err(String::from(
                    "command `schemes` takes scheme names or paths as operands and no options",
                ));
            }
        }
        _ => {
            if inputs.len() != 1 {
                return Err(format!("command `{name}` requires exactly one file"));
            }
            if inputs.iter().any(|input| input == Path::new("-")) {
                return Err(format!("command `{name}` reads a file, not standard input"));
            }
        }
    }
    Ok(Action::Seal(crypt::SealOptions {
        command,
        edition,
        scheme,
        key,
        output,
        inputs,
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

    #[cfg(unix)]
    #[test]
    fn used_modules_are_read_once_from_the_root_directory() {
        let directory = unix_test_root().join(format!("orangec-modules-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&directory);
        std::fs::create_dir(&directory).unwrap();
        let files = [
            (
                "base.or",
                "edition 2026; module base { spec one() -> Int { 1 } }\n",
            ),
            ("left.or", "edition 2026; module left { use base; }\n"),
            (
                "right.or",
                "edition 2026; module right { use base; use left; }\n",
            ),
        ];
        for (name, text) in files {
            std::fs::write(directory.join(name), text).unwrap();
        }
        let root_path = directory.join("root.or");
        let load = |root_text: &str| {
            let mut sources = SourceMap::new();
            let id = sources.add("root.or", root_text).unwrap();
            let source = sources.get(id).unwrap();
            let lexed = lex(source, Edition::default());
            let ast = parse(source, &lexed).into_ast().unwrap();
            let mut remaining = MAX_SOURCE_BYTES_PER_INVOCATION;
            let loaded = load_used_modules(
                &root_path,
                &ast,
                &mut sources,
                Edition::default(),
                &mut remaining,
            );
            let loaded = loaded.map(|modules| {
                modules
                    .iter()
                    .map(|(id, ast)| {
                        (
                            sources.get(*id).unwrap().name().to_owned(),
                            ast.module().name().text().to_owned(),
                        )
                    })
                    .collect::<Vec<_>>()
            });
            (loaded, MAX_SOURCE_BYTES_PER_INVOCATION - remaining)
        };

        // Each module is read once, in the order a `use` first names it; the
        // root's own name is not read.
        let (loaded, charged) =
            load("edition 2026; module root { use right; use root; use base; use right; }");
        let path_of = |name: &str| directory.join(name).display().to_string();
        assert_eq!(
            loaded.unwrap(),
            [
                (path_of("right.or"), String::from("right")),
                (path_of("base.or"), String::from("base")),
                (path_of("left.or"), String::from("left")),
            ]
        );
        assert_eq!(
            charged,
            files.iter().map(|(_, text)| text.len()).sum::<usize>()
        );

        let (missing, _) = load("edition 2026; module root { use left; use gone; }");
        let group = missing.unwrap_err();
        assert!(group.starts_with("error[ORC1001]: could not read source file `"));
        assert!(group.contains("gone.or`"));
        assert!(group.ends_with(
            "  = note: `use gone;` in module `root` reads the module `gone` from this file\n"
        ));
        std::fs::remove_dir_all(&directory).unwrap();
    }

    #[test]
    fn cli_diagnostic_code_inventory_is_exact_ordered_and_unique() {
        let actual = CliDiagnosticCode::ALL
            .iter()
            .map(|code| code.as_str())
            .collect::<Vec<_>>();
        let expected = [
            "ORC1001", "ORC1002", "ORC1003", "ORC1004", "ORC1005", "ORC1006", "ORC1007", "ORC1008",
            "ORC1009", "ORC1010", "ORC1011", "ORC1012", "ORC1013", "ORC1014", "ORC1015", "ORC1016",
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
            evaluation: Evaluation::default(),
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
            evaluation: Evaluation::default(),
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
            evaluation: Evaluation::default(),
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
        assert_eq!(
            names,
            ["check", "eval", "lex", "keygen", "enc", "dec", "schemes"]
        );
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

    #[derive(Default)]
    struct InterruptEveryWrite {
        attempts: usize,
    }

    impl Write for InterruptEveryWrite {
        fn write(&mut self, _buffer: &[u8]) -> io::Result<usize> {
            self.attempts += 1;
            Err(io::Error::from(io::ErrorKind::Interrupted))
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    #[derive(Default)]
    struct InterruptEveryFlush {
        attempts: usize,
        bytes: Vec<u8>,
    }

    impl Write for InterruptEveryFlush {
        fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
            self.bytes.extend_from_slice(buffer);
            Ok(buffer.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            self.attempts += 1;
            Err(io::Error::from(io::ErrorKind::Interrupted))
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

    struct InterruptingSeekReader {
        inner: io::Cursor<Vec<u8>>,
        interruptions_remaining: usize,
        seek_attempts: usize,
    }

    impl InterruptingSeekReader {
        fn new(bytes: &[u8], interruptions: usize) -> Self {
            Self {
                inner: io::Cursor::new(bytes.to_vec()),
                interruptions_remaining: interruptions,
                seek_attempts: 0,
            }
        }
    }

    impl Read for InterruptingSeekReader {
        fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
            self.inner.read(buffer)
        }
    }

    impl Seek for InterruptingSeekReader {
        fn seek(&mut self, position: SeekFrom) -> io::Result<u64> {
            self.seek_attempts += 1;
            if self.interruptions_remaining != 0 {
                self.interruptions_remaining -= 1;
                Err(io::Error::from(io::ErrorKind::Interrupted))
            } else {
                self.inner.seek(position)
            }
        }
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
    fn persistent_interrupted_writes_are_bounded_across_output_classes() {
        let mut input = RejectReads::default();
        let mut output = InterruptEveryWrite::default();
        let mut error = Vec::new();
        let status = run(
            os_arguments(&["--help"]),
            &mut input,
            &mut output,
            &mut error,
        );
        assert_eq!(status, COMPILATION_ERROR);
        assert_eq!(input.attempts, 0);
        assert_eq!(output.attempts, MAX_CONSECUTIVE_INTERRUPTED_IO_ATTEMPTS);
        assert_eq!(error, b"");

        let mut input = RejectReads::default();
        let mut output = Vec::new();
        let mut error = InterruptEveryWrite::default();
        let status = run(
            os_arguments(&["unknown", "source.or"]),
            &mut input,
            &mut output,
            &mut error,
        );
        assert_eq!(status, COMPILATION_ERROR);
        assert_eq!(input.attempts, 0);
        assert_eq!(output, b"");
        assert_eq!(error.attempts, MAX_CONSECUTIVE_INTERRUPTED_IO_ATTEMPTS);
    }

    #[test]
    fn persistent_interrupted_flushes_are_bounded_across_output_classes() {
        let mut input = RejectReads::default();
        let mut output = InterruptEveryFlush::default();
        let mut error = Vec::new();
        let status = run(
            os_arguments(&["--help"]),
            &mut input,
            &mut output,
            &mut error,
        );
        assert_eq!(status, COMPILATION_ERROR);
        assert_eq!(input.attempts, 0);
        assert_eq!(output.attempts, MAX_CONSECUTIVE_INTERRUPTED_IO_ATTEMPTS);
        assert_eq!(output.bytes, USAGE.as_bytes());
        assert_eq!(error, b"");

        let mut input = RejectReads::default();
        let mut output = Vec::new();
        let mut error = InterruptEveryFlush::default();
        let status = run(
            os_arguments(&["unknown", "source.or"]),
            &mut input,
            &mut output,
            &mut error,
        );
        assert_eq!(status, COMPILATION_ERROR);
        assert_eq!(input.attempts, 0);
        assert_eq!(output, b"");
        assert_eq!(error.attempts, MAX_CONSECUTIVE_INTERRUPTED_IO_ATTEMPTS);
        assert!(error.bytes.starts_with(b"orangec: unknown command"));
    }

    #[test]
    fn persistent_interrupted_source_reads_fail_closed() {
        let mut input = RejectReads {
            attempts: 0,
            error_kind: io::ErrorKind::Interrupted,
        };
        let mut output = Vec::new();
        let mut error = Vec::new();
        let status = run(
            os_arguments(&["check", "-"]),
            &mut input,
            &mut output,
            &mut error,
        );

        assert_eq!(status, COMPILATION_ERROR);
        assert_eq!(input.attempts, MAX_CONSECUTIVE_INTERRUPTED_IO_ATTEMPTS);
        assert_eq!(output, b"");
        assert_eq!(
            error,
            concat!(
                "error[ORC1001]: could not read source file `<stdin>`\n",
                "  = note: source stream remained interrupted for 1,024 consecutive attempts\n",
            )
            .as_bytes()
        );
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
                evaluation: Evaluation::default(),
            }))
        );
        assert_eq!(
            parse_arguments(os_arguments(&["check", "--edition=2026", "one.or"])),
            Ok(Action::Compile(Options {
                command: CompilerCommand::Check,
                edition: Edition::E2026,
                paths: vec![PathBuf::from("one.or")],
                evaluation: Evaluation::default(),
            }))
        );
        assert_eq!(
            parse_arguments(os_arguments(&["eval", "--edition=2026", "one.or"])),
            Ok(Action::Compile(Options {
                command: CompilerCommand::Eval,
                edition: Edition::E2026,
                paths: vec![PathBuf::from("one.or")],
                evaluation: Evaluation::default(),
            }))
        );
    }

    #[test]
    fn evaluation_options_parse_with_exact_bounds_and_messages() {
        use std::os::unix::ffi::OsStringExt as _;

        let evaluation = |arguments: &[&str]| match parse_arguments(os_arguments(arguments)) {
            Ok(Action::Compile(options)) => Ok(options.evaluation),
            Ok(_) => panic!("{arguments:?} is not a compilation"),
            Err(error) => Err(error),
        };
        assert_eq!(
            evaluation(&[
                "eval",
                "--steps",
                "5",
                "--spec",
                "a",
                "--spec=b_2",
                "--stats",
                "--spec",
                "a",
                "one.or",
            ]),
            Ok(Evaluation {
                steps: 5,
                specs: vec![String::from("a"), String::from("b_2")],
                stats: true,
            })
        );
        assert_eq!(
            evaluation(&["--steps=1073741824", "eval", "one.or"]).map(|value| value.steps),
            Ok(MAX_EVALUATION_STEP_LIMIT)
        );
        assert_eq!(
            evaluation(&["eval", "one.or"]),
            Ok(Evaluation {
                steps: MAX_EVALUATION_STEPS_PER_SOURCE,
                specs: Vec::new(),
                stats: false,
            })
        );

        let steps = Err(String::from(
            "option `--steps` takes a number of steps from 1 through 1073741824",
        ));
        for value in [
            "0",
            "01",
            "1073741825",
            "-1",
            "+5",
            "1e3",
            "1_000",
            " 5",
            "",
            "99999999999999999999999",
        ] {
            assert_eq!(
                evaluation(&["eval", "--steps", value, "one.or"]),
                steps,
                "{value}"
            );
        }
        assert_eq!(
            evaluation(&["eval", "--steps", "5", "--steps=6", "one.or"]),
            Err(String::from(
                "option `--steps` may be specified at most once"
            ))
        );
        assert_eq!(
            evaluation(&["eval", "one.or", "--steps"]),
            Err(String::from("option `--steps` requires a value"))
        );

        let spec = Err(String::from("option `--spec` takes the name of a function"));
        for value in ["", "1a", "a-b", "m::f", "f()", "é", "a b"] {
            assert_eq!(
                evaluation(&["eval", "--spec", value, "one.or"]),
                spec,
                "{value}"
            );
        }
        let names = (0..65).map(|index| format!("f{index}")).collect::<Vec<_>>();
        let mut arguments = vec!["eval"];
        for name in names.iter().take(64) {
            arguments.extend(["--spec", name.as_str()]);
        }
        arguments.push("one.or");
        assert_eq!(
            evaluation(&arguments).map(|value| value.specs),
            Ok(names[..64].to_vec())
        );
        arguments.pop();
        arguments.extend(["--spec", "f64", "one.or"]);
        assert_eq!(
            evaluation(&arguments),
            Err(String::from("option `--spec` names at most 64 functions"))
        );

        for (arguments, name) in [
            (&["check", "--steps", "5", "one.or"][..], "--steps"),
            (&["check", "--steps", "1048576", "one.or"][..], "--steps"),
            (&["lex", "--spec", "f", "one.or"][..], "--spec"),
            (&["check", "--stats", "one.or"][..], "--stats"),
            (
                &["--steps=5", "--spec=f", "--stats", "check", "one.or"][..],
                "--steps",
            ),
            (&["--spec=f", "--stats", "check", "one.or"][..], "--spec"),
            (&["enc", "--stats", "one.or"][..], "--stats"),
        ] {
            assert_eq!(
                parse_arguments(os_arguments(arguments)),
                Err(format!("option `{name}` applies only to eval")),
                "{arguments:?}"
            );
        }

        assert_eq!(
            parse_arguments([
                OsString::from_vec(b"--steps=\x80".to_vec()),
                OsString::from("eval"),
                OsString::from("one.or"),
            ]),
            Err(String::from(
                "an option value that is not valid UTF-8 must be a separate argument"
            ))
        );
        assert_eq!(
            parse_arguments([
                OsString::from("eval"),
                OsString::from("--spec"),
                OsString::from_vec(vec![0x80]),
                OsString::from("one.or"),
            ]),
            Err(String::from("option `--spec` takes the name of a function"))
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
    fn source_snapshot_requires_an_exact_second_descriptor_read() {
        let cases: &[(&[u8], &[u8], bool)] = &[
            (b"", b"", true),
            (b"abc", b"abc", true),
            (b"abd", b"abc", false),
            (b"ab", b"abc", false),
            (b"abcd", b"abc", false),
        ];

        for &(descriptor, expected, matches) in cases {
            let mut reader = io::Cursor::new(descriptor);
            reader.set_position(u64::try_from(descriptor.len()).unwrap());

            assert_eq!(
                source_descriptor_matches_snapshot(&mut reader, expected).unwrap(),
                matches,
                "descriptor={descriptor:?}, expected={expected:?}",
            );
        }
    }

    #[test]
    fn source_snapshot_seek_retries_are_transient_and_bounded() {
        let mut transient = InterruptingSeekReader::new(b"abc", 1);
        transient.inner.set_position(3);
        assert!(source_descriptor_matches_snapshot(&mut transient, b"abc").unwrap());
        assert_eq!(transient.seek_attempts, 2);

        let mut persistent =
            InterruptingSeekReader::new(b"abc", MAX_CONSECUTIVE_INTERRUPTED_IO_ATTEMPTS);
        persistent.inner.set_position(3);
        let error = source_descriptor_matches_snapshot(&mut persistent, b"abc").unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::Other);
        assert!(
            error
                .get_ref()
                .is_some_and(|cause| cause.is::<InterruptedIoLimitExceeded>())
        );
        assert_eq!(
            persistent.seek_attempts,
            MAX_CONSECUTIVE_INTERRUPTED_IO_ATTEMPTS
        );
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

    /// Both streams of one run, in the order their bytes were written.
    #[derive(Clone, Default)]
    struct Transcript(std::rc::Rc<std::cell::RefCell<Vec<u8>>>);

    impl Write for Transcript {
        fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
            self.0.borrow_mut().extend_from_slice(buffer);
            Ok(buffer.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn step_report_follows_committed_values_and_is_not_written_without_them() {
        let source =
            b"edition 2026; module values { spec answer() -> Int { 42 } spec one() -> Word[8] { 1 } }\n";
        let transcript = Transcript::default();
        let mut input = source.as_slice();
        let status = run(
            os_arguments(&["eval", "--stats", "-"]),
            &mut input,
            &mut transcript.clone(),
            &mut transcript.clone(),
        );
        assert_eq!(status, SUCCESS);
        assert_eq!(
            String::from_utf8_lossy(&transcript.0.borrow()),
            concat!(
                "values::answer: Int = 42\n",
                "values::one: Word[8] = 0x01\n",
                "values::answer: 1 step\n",
                "values::one: 1 step\n",
                "total: 2 of 1048576 steps\n",
            )
        );

        let mut input = source.as_slice();
        let mut output = FailFlush::default();
        let mut error = Vec::new();
        let status = run(
            os_arguments(&["eval", "--stats", "-"]),
            &mut input,
            &mut output,
            &mut error,
        );
        assert_eq!(status, COMPILATION_ERROR);
        assert_eq!(
            output.bytes,
            b"values::answer: Int = 42\nvalues::one: Word[8] = 0x01\n"
        );
        assert_eq!(output.flush_attempts, 1);
        assert_eq!(error, b"orangec: could not write evaluation output\n");
    }

    #[test]
    fn step_limit_diagnostics_name_the_option_only_below_the_most_admitted() {
        let mut sources = SourceMap::new();
        let id = sources
            .add(
                "budget.or",
                "edition 2026; module budget { spec two() -> Int { 1 + 1 } }\n",
            )
            .unwrap();
        let source = sources.get(id).unwrap();
        let lexed = lex(source, Edition::default());
        let ast = parse(source, &lexed).into_ast().unwrap();
        let analysis = analyze_program((source, &ast), &[]);
        let stopped = evaluate_selected(analysis.core().unwrap(), 1, |_| true);
        let diagnostics = stopped.diagnostics();
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(
            diagnostics[0].message(),
            "reference evaluation step limit exceeded"
        );
        let hint = "`orangec eval --steps N` sets the budget, up to 1073741824 steps";
        for (budget, hinted) in [
            (1, true),
            (MAX_EVALUATION_STEPS_PER_SOURCE, true),
            (MAX_EVALUATION_STEP_LIMIT - 1, true),
            (MAX_EVALUATION_STEP_LIMIT, false),
        ] {
            let copied = with_budget_hint(diagnostics, budget).unwrap();
            let mut expected = diagnostics[0].clone();
            if hinted {
                expected = expected.with_note(hint);
            }
            assert_eq!(copied, [expected], "{budget}");
        }

        // Every other diagnostic is copied unchanged.
        let other = Diagnostic::error(
            DiagnosticCode::EvaluationResourceLimit,
            "reference evaluation result allocation failed",
            diagnostics[0].primary_span(),
        );
        assert_eq!(
            with_budget_hint(std::slice::from_ref(&other), 1).unwrap(),
            [other]
        );
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
                Self::Help | Self::Version | Self::Seal(_) => panic!("expected compile action"),
            }
        }
    }
}
