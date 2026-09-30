//! Sealing files: `orangec keygen`, `enc`, `dec`, and `schemes`.
//!
//! Every byte of cryptography here is an Orange program run by the reference
//! evaluator. This module only moves bytes: it reads keys and operating-system
//! randomness, lays out the sealed-file format, and calls a scheme's `seal`,
//! `authentic`, and `open` specs one chunk at a time. The format is specified
//! in `compiler/schemes/README.md`.

use std::ffi::{OsStr, OsString};
use std::fmt::Write as _;
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::mpsc;

use orange_compiler::{
    ArrayType, CoreArray, CoreFunction, CoreModule, CoreType, CoreValue, Edition, Evaluator,
    RenderedSourceName, SourceMap, analyze_program, lex, parse, render_diagnostics,
};

use super::{
    CliDiagnosticCode, CompilerCommand, PhaseResult, classify_phase_result, escape_display_text,
    io_error_reason, open_source_file, read_retry_interrupted, read_source, render_cli_error,
    render_read_source_error,
};

/// The scheme `keygen` uses when no `--scheme` is given.
pub(crate) const DEFAULT_SCHEME: &str = "xchacha20_poly1305";

/// Evaluation steps allowed for one call of a scheme's spec.
const STEPS_PER_CALL: usize = 1 << 24;

/// A sealed file begins with this magic, a zero byte, and the format version.
const MAGIC: &[u8; 6] = b"orange";
const FORMAT_VERSION: u8 = 1;
/// The header is also every chunk's associated data.
const HEADER_BYTES: usize = 64;
/// Header field offsets.
const NAME_FIELD: std::ops::Range<usize> = 16..40;
const PREFIX_FIELD: std::ops::Range<usize> = 40..64;
/// A chunk's nonce ends with a 32-bit big-endian counter and a final-chunk flag.
const COUNTER_AND_FLAG_BYTES: usize = 5;
/// The final chunk's plaintext ends with this byte and then zeros.
const PADDING_MARK: u8 = 0x80;

/// Scheme shape bounds, in bytes.
const MIN_KEY_BYTES: usize = 16;
const MAX_KEY_BYTES: usize = 64;
const MIN_NONCE_BYTES: usize = 12;
const MIN_TAG_BYTES: usize = 16;
const MIN_CHUNK_BYTES: usize = 16;
const MAX_SCHEME_NAME_BYTES: usize = 24;

/// A key file is small text; anything larger is not one.
const MAX_KEY_FILE_BYTES: u64 = 4_096;
const KEY_FILE_TAG: &str = "orange-key";
const KEY_FILE_VERSION: &str = "1";

const IO_BUFFER_BYTES: usize = 64 * 1024;

/// One scheme compiled into `orangec`.
struct Builtin {
    name: &'static str,
    path: &'static str,
    text: &'static str,
    standard: &'static str,
}

const BUILTINS: &[Builtin] = &[
    Builtin {
        name: "xchacha20_poly1305",
        path: "compiler/schemes/xchacha20_poly1305.or",
        text: include_str!("../../../schemes/xchacha20_poly1305.or"),
        standard: "draft-irtf-cfrg-xchacha-03 over RFC 8439",
    },
    Builtin {
        name: "chacha20_poly1305",
        path: "compiler/schemes/chacha20_poly1305.or",
        text: include_str!("../../../schemes/chacha20_poly1305.or"),
        standard: "RFC 8439",
    },
    Builtin {
        name: "ascon_aead128",
        path: "compiler/schemes/ascon_aead128.or",
        text: include_str!("../../../schemes/ascon_aead128.or"),
        standard: "NIST SP 800-232",
    },
];

/// Options of the sealing commands.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct SealOptions {
    pub(crate) command: CompilerCommand,
    pub(crate) edition: Edition,
    pub(crate) scheme: Option<OsString>,
    pub(crate) key: Option<PathBuf>,
    pub(crate) output: Option<PathBuf>,
    pub(crate) inputs: Vec<PathBuf>,
}

/// The sizes a scheme's specs declare, in bytes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Shape {
    key: usize,
    nonce: usize,
    tag: usize,
    chunk: usize,
}

impl Shape {
    fn sealed(self) -> Option<usize> {
        self.chunk.checked_add(self.tag)
    }

    fn prefix(self) -> Option<usize> {
        self.nonce.checked_sub(COUNTER_AND_FLAG_BYTES)
    }
}

/// A compiled scheme program and the shape of its interface.
struct Scheme {
    name: String,
    origin: String,
    sources: SourceMap,
    core: CoreModule,
    shape: Shape,
}

/// A secret key and the scheme it belongs to.
struct Key {
    scheme: String,
    bytes: Vec<u8>,
}

/// Runs a sealing command and returns the process status.
pub(crate) fn run(
    options: &SealOptions,
    standard_output: &mut impl Write,
    standard_error: &mut impl Write,
) -> u8 {
    let result = match options.command {
        CompilerCommand::Keygen => keygen(options, standard_output),
        CompilerCommand::Enc => encrypt(options),
        CompilerCommand::Dec => decrypt(options),
        CompilerCommand::Schemes => schemes(options, standard_output),
        CompilerCommand::Check | CompilerCommand::Eval | CompilerCommand::Lex => {
            Err(render_cli_error(
                CliDiagnosticCode::MissingPhaseArtifact,
                "a compiler command reached the sealing commands",
                "this is an internal compiler failure",
            ))
        }
    };
    match result {
        Ok(()) => {
            if super::flush_retry_interrupted(standard_output).is_err() {
                super::COMPILATION_ERROR
            } else {
                super::SUCCESS
            }
        }
        Err(group) => {
            let _ = standard_error
                .write_all(group.as_bytes())
                .and_then(|()| super::flush_retry_interrupted(standard_error));
            super::COMPILATION_ERROR
        }
    }
}

// ---- Commands --------------------------------------------------------------

fn keygen(options: &SealOptions, standard_output: &mut impl Write) -> Result<(), String> {
    let scheme = resolve_scheme(
        options
            .scheme
            .as_deref()
            .unwrap_or_else(|| OsStr::new(DEFAULT_SCHEME)),
        options.edition,
    )?;
    let (path, default) = match &options.output {
        Some(path) => (path.clone(), false),
        None => (default_key_path()?, true),
    };
    let shown = shown_path(&path);
    if path.symlink_metadata().is_ok() {
        return Err(render_cli_error(
            CliDiagnosticCode::KeyFile,
            format_args!("a file already exists at `{shown}`"),
            "orangec never replaces a key; remove it first or name another path with -o",
        ));
    }
    if default && let Some(parent) = path.parent() {
        create_private_directory(parent).map_err(|error| {
            render_cli_error(
                CliDiagnosticCode::KeyFile,
                format_args!("could not create the key directory for `{shown}`"),
                io_error_reason(&error),
            )
        })?;
    }
    let bytes = random_bytes(scheme.shape.key)?;
    let text = key_file_text(&scheme.name, &bytes);
    let written = create_new_private(&path).and_then(|mut file| {
        let written = file
            .write_all(text.as_bytes())
            .and_then(|()| file.sync_all());
        if written.is_err() {
            // The file is this command's own: leave neither a truncated key
            // nor a name that would refuse the next attempt.
            drop(file);
            let _ = fs::remove_file(&path);
        }
        written
    });
    if let Err(error) = written {
        return Err(render_cli_error(
            CliDiagnosticCode::KeyFile,
            format_args!("could not write the key file `{shown}`"),
            reason_with_existing(&error),
        ));
    }
    let bits = scheme.shape.key.saturating_mul(8);
    writeln!(
        standard_output,
        "wrote a {bits}-bit {} key to `{shown}`",
        scheme.name
    )
    .map_err(|_| String::from("orangec: could not write command output\n"))
}

fn encrypt(options: &SealOptions) -> Result<(), String> {
    let input = single_input(options)?;
    let key = read_key(&key_path(options)?)?;
    let scheme = scheme_for_key(options, &key)?;
    let output = match &options.output {
        Some(output) => output.clone(),
        None => {
            let mut name = input.as_os_str().to_owned();
            name.push(".orange");
            PathBuf::from(name)
        }
    };
    let mut reader = open_input(input)?;
    let prefix_bytes = scheme.shape.prefix().ok_or_else(inconsistent_shape)?;
    let prefix = random_bytes(prefix_bytes)?;
    let header = header(&scheme, &prefix)?;
    let mut pending = PendingOutput::create(&output, false)?;
    let shown_input = shown_path(input);
    seal_stream(
        &scheme,
        &key.bytes,
        &header,
        &prefix,
        &mut reader,
        &mut pending,
        &shown_input,
    )?;
    pending.publish()
}

fn decrypt(options: &SealOptions) -> Result<(), String> {
    let input = single_input(options)?;
    let shown_input = shown_path(input);
    let output = match &options.output {
        Some(output) => output.clone(),
        None => {
            let stripped = input
                .as_os_str()
                .as_encoded_bytes()
                .strip_suffix(b".orange")
                .filter(|stem| !stem.is_empty() && !stem.ends_with(b"/"));
            if stripped.is_none() {
                return Err(render_cli_error(
                    CliDiagnosticCode::CryptOutput,
                    format_args!("`{shown_input}` does not end in `.orange`"),
                    "name the opened file with -o",
                ));
            }
            let mut path = input.to_path_buf();
            path.set_extension("");
            path
        }
    };
    let key = read_key(&key_path(options)?)?;
    let mut reader = open_input(input)?;
    let mut header = [0; HEADER_BYTES];
    let read =
        read_full(&mut reader, &mut header).map_err(|error| read_error(&shown_input, &error))?;
    if read != HEADER_BYTES {
        return Err(render_cli_error(
            CliDiagnosticCode::SealedFile,
            format_args!("`{shown_input}` is not an Orange sealed file"),
            "it is shorter than the 64-byte header",
        ));
    }
    let parsed =
        parse_header(&header).map_err(|problem| sealed_file_error(&shown_input, problem))?;
    if parsed.scheme != key.scheme {
        return Err(render_cli_error(
            CliDiagnosticCode::KeyFile,
            format_args!(
                "`{shown_input}` was sealed with the scheme `{}`, but the key is for `{}`",
                parsed.scheme, key.scheme
            ),
            "open it with a key made for its scheme",
        ));
    }
    let scheme = scheme_for_key(options, &key)?;
    if scheme.shape != parsed.shape {
        return Err(sealed_file_error(
            &shown_input,
            format!(
                "its header's sizes differ from those of the scheme `{}` at {}",
                scheme.name, scheme.origin
            ),
        ));
    }
    let mut pending = PendingOutput::create(&output, true)?;
    open_stream(
        &scheme,
        &key.bytes,
        &header,
        &parsed.prefix,
        &mut reader,
        &mut pending,
        &shown_input,
    )?;
    pending.publish()
}

fn schemes(options: &SealOptions, standard_output: &mut impl Write) -> Result<(), String> {
    let mut rows = Vec::new();
    if options.inputs.is_empty() {
        for builtin in BUILTINS {
            let scheme = compile_builtin(builtin, options.edition)?;
            let mut standard = String::from(builtin.standard);
            if builtin.name == DEFAULT_SCHEME {
                standard.push_str(" (default)");
            }
            rows.push((scheme, standard));
        }
    } else {
        for input in &options.inputs {
            let scheme = resolve_scheme(input.as_os_str(), options.edition)?;
            let origin = scheme.origin.clone();
            rows.push((scheme, origin));
        }
    }
    let mut table = String::from("scheme");
    let width = rows
        .iter()
        .map(|(scheme, _)| scheme.name.len())
        .max()
        .unwrap_or(0)
        .max(6);
    let left = |text: &str, width: usize| format!("{text:<width$}");
    let _ = writeln!(
        table,
        "{}  key  nonce  tag  chunk  seal steps  source",
        left("", width.saturating_sub(6))
    );
    for (scheme, source) in &rows {
        let steps = measure_seal(scheme)?;
        let _ = writeln!(
            table,
            "{}  {:>3}  {:>5}  {:>3}  {:>5}  {:>10}  {source}",
            left(&scheme.name, width),
            scheme.shape.key.saturating_mul(8),
            scheme.shape.nonce.saturating_mul(8),
            scheme.shape.tag.saturating_mul(8),
            scheme.shape.chunk,
            steps,
        );
    }
    table.push_str(
        "\nKey, nonce, and tag sizes are in bits; a chunk is that many bytes of plaintext.\n",
    );
    standard_output
        .write_all(table.as_bytes())
        .map_err(|_| String::from("orangec: could not write command output\n"))
}

// ---- The format ------------------------------------------------------------

/// The 64-byte header: magic, version, sizes, scheme name, nonce prefix.
fn header(scheme: &Scheme, prefix: &[u8]) -> Result<[u8; HEADER_BYTES], String> {
    let mut header = [0; HEADER_BYTES];
    let shape = scheme.shape;
    let fields = [
        (0..6, MAGIC.as_slice()),
        (7..8, [FORMAT_VERSION].as_slice()),
    ];
    for (range, bytes) in fields {
        copy_into(&mut header, range, bytes)?;
    }
    let small = |value: usize| u8::try_from(value).map_err(|_| inconsistent_shape());
    copy_into(
        &mut header,
        8..11,
        &[small(shape.key)?, small(shape.nonce)?, small(shape.tag)?],
    )?;
    let chunk = u32::try_from(shape.chunk).map_err(|_| inconsistent_shape())?;
    copy_into(&mut header, 12..16, &chunk.to_be_bytes())?;
    let name_end = NAME_FIELD
        .start
        .checked_add(scheme.name.len())
        .ok_or_else(inconsistent_shape)?;
    copy_into(
        &mut header,
        NAME_FIELD.start..name_end,
        scheme.name.as_bytes(),
    )?;
    let prefix_end = PREFIX_FIELD
        .start
        .checked_add(prefix.len())
        .ok_or_else(inconsistent_shape)?;
    copy_into(&mut header, PREFIX_FIELD.start..prefix_end, prefix)?;
    Ok(header)
}

fn copy_into(target: &mut [u8], range: std::ops::Range<usize>, bytes: &[u8]) -> Result<(), String> {
    target
        .get_mut(range)
        .filter(|slot| slot.len() == bytes.len())
        .map(|slot| slot.copy_from_slice(bytes))
        .ok_or_else(inconsistent_shape)
}

struct ParsedHeader {
    scheme: String,
    shape: Shape,
    prefix: Vec<u8>,
}

fn parse_header(header: &[u8; HEADER_BYTES]) -> Result<ParsedHeader, String> {
    let byte = |index: usize| header.get(index).copied().unwrap_or(0);
    if header.get(0..6) != Some(MAGIC.as_slice()) || byte(6) != 0 {
        return Err(String::from("it does not begin with the `orange` magic"));
    }
    if byte(7) != FORMAT_VERSION {
        return Err(format!(
            "it uses format version {}, and this orangec reads version {FORMAT_VERSION}",
            byte(7)
        ));
    }
    if byte(11) != 0 {
        return Err(String::from("its reserved header byte is not zero"));
    }
    let chunk = header
        .get(12..16)
        .and_then(|bytes| <[u8; 4]>::try_from(bytes).ok())
        .map(u32::from_be_bytes)
        .and_then(|chunk| usize::try_from(chunk).ok())
        .unwrap_or(0);
    let shape = Shape {
        key: usize::from(byte(8)),
        nonce: usize::from(byte(9)),
        tag: usize::from(byte(10)),
        chunk,
    };
    check_shape(shape)
        .map_err(|problem| format!("its header names impossible sizes: {problem}"))?;
    let field = header.get(NAME_FIELD).unwrap_or_default();
    let length = field
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(field.len());
    let (name, rest) = field.split_at(length);
    let name = std::str::from_utf8(name)
        .ok()
        .filter(|name| valid_scheme_name(name) && rest.iter().all(|byte| *byte == 0))
        .ok_or_else(|| String::from("its header does not name a scheme"))?;
    let prefix_length = shape.prefix().unwrap_or(0);
    let field = header.get(PREFIX_FIELD).unwrap_or_default();
    let (prefix, rest) = field.split_at(prefix_length.min(field.len()));
    if rest.iter().any(|byte| *byte != 0) {
        return Err(String::from("its nonce-prefix field is not zero-padded"));
    }
    Ok(ParsedHeader {
        scheme: String::from(name),
        shape,
        prefix: prefix.to_vec(),
    })
}

/// The nonce of chunk `index`: prefix, big-endian counter, final flag.
fn chunk_nonce(prefix: &[u8], index: u32, last: bool) -> Vec<u8> {
    let mut nonce = Vec::with_capacity(prefix.len().saturating_add(COUNTER_AND_FLAG_BYTES));
    nonce.extend_from_slice(prefix);
    nonce.extend_from_slice(&index.to_be_bytes());
    nonce.push(u8::from(last));
    nonce
}

/// Pads the final chunk's `data` to `chunk` bytes: 0x80, then zeros.
fn pad(data: &[u8], chunk: usize) -> Option<Vec<u8>> {
    if data.len() >= chunk {
        return None;
    }
    let mut padded = Vec::with_capacity(chunk);
    padded.extend_from_slice(data);
    padded.push(PADDING_MARK);
    padded.resize(chunk, 0);
    Some(padded)
}

/// Removes the final chunk's padding, or returns `None` if it has none.
fn unpad(padded: &[u8]) -> Option<&[u8]> {
    let mark = padded.iter().rposition(|byte| *byte != 0)?;
    (padded.get(mark) == Some(&PADDING_MARK)).then(|| padded.get(..mark))?
}

fn seal_stream(
    scheme: &Scheme,
    key: &[u8],
    header: &[u8; HEADER_BYTES],
    prefix: &[u8],
    reader: &mut impl Read,
    pending: &mut PendingOutput,
    shown_input: &str,
) -> Result<(), String> {
    pending.write(header)?;
    let chunk = scheme.shape.chunk;
    let mut index: u32 = 0;
    let mut finished = false;
    let next = || {
        if finished {
            return Ok(None);
        }
        let mut data = vec![0; chunk];
        let read = read_full(reader, &mut data).map_err(|error| read_error(shown_input, &error))?;
        let last = read < chunk;
        if last {
            data = data
                .get(..read)
                .and_then(|read| pad(read, chunk))
                .ok_or_else(inconsistent_shape)?;
            finished = true;
        }
        let job = Job {
            index,
            nonce: chunk_nonce(prefix, index, last),
            last,
            data,
        };
        if !last {
            index = index.checked_add(1).ok_or_else(|| {
                render_cli_error(
                    CliDiagnosticCode::CryptInput,
                    format_args!("`{shown_input}` is too large to seal"),
                    "a sealed file holds fewer than 2^32 chunks",
                )
            })?;
        }
        Ok(Some(job))
    };
    let seal = |calls: &mut Calls<'_>, job: &Job| calls.seal(&job.nonce, &job.data);
    run_chunks(scheme, key, header, next, &seal, |_, sealed| {
        pending.write(&sealed)
    })
}

fn open_stream(
    scheme: &Scheme,
    key: &[u8],
    header: &[u8; HEADER_BYTES],
    prefix: &[u8],
    reader: &mut impl Read,
    pending: &mut PendingOutput,
    shown_input: &str,
) -> Result<(), String> {
    let sealed_bytes = scheme.shape.sealed().ok_or_else(inconsistent_shape)?;
    let mut current = vec![0; sealed_bytes];
    let read = read_full(reader, &mut current).map_err(|error| read_error(shown_input, &error))?;
    if read != sealed_bytes {
        return Err(sealed_file_error(
            shown_input,
            String::from("it ends before its first chunk is complete"),
        ));
    }
    let mut index: u32 = 0;
    let mut finished = false;
    // A chunk is the final one when nothing follows it, so each read looks
    // one chunk ahead.
    let next = || {
        if finished {
            return Ok(None);
        }
        let mut following = vec![0; sealed_bytes];
        let read =
            read_full(reader, &mut following).map_err(|error| read_error(shown_input, &error))?;
        let last = read == 0;
        if !last && read != sealed_bytes {
            return Err(sealed_file_error(
                shown_input,
                format!(
                    "it ends inside chunk {}",
                    u64::from(index).saturating_add(1)
                ),
            ));
        }
        let job = Job {
            index,
            nonce: chunk_nonce(prefix, index, last),
            last,
            data: std::mem::replace(&mut current, following),
        };
        if last {
            finished = true;
        } else {
            index = index.checked_add(1).ok_or_else(|| {
                sealed_file_error(shown_input, String::from("it holds more than 2^32 chunks"))
            })?;
        }
        Ok(Some(job))
    };
    let open = |calls: &mut Calls<'_>, job: &Job| {
        if calls.authentic(&job.nonce, &job.data)? {
            calls.open(&job.nonce, &job.data).map(Some)
        } else {
            Ok(None)
        }
    };
    run_chunks(scheme, key, header, next, &open, |job, opened| {
        let Some(plaintext) = opened else {
            return Err(render_cli_error(
                CliDiagnosticCode::NotAuthentic,
                format_args!("chunk {} of `{shown_input}` is not authentic", job.index),
                "the file was altered, cut short, or sealed with another key; nothing was written",
            ));
        };
        if job.last {
            let data = unpad(&plaintext).ok_or_else(|| {
                sealed_file_error(
                    shown_input,
                    String::from("its authentic final chunk is not padded"),
                )
            })?;
            pending.write(data)
        } else {
            pending.write(&plaintext)
        }
    })
}

/// One chunk to seal or open.
struct Job {
    index: u32,
    nonce: Vec<u8>,
    last: bool,
    data: Vec<u8>,
}

/// Chunks one worker evaluates per round.
const CHUNKS_PER_WORKER: usize = 16;
/// The most workers a stream uses, whatever the machine offers.
const MAX_WORKERS: usize = 64;

/// Where a worker takes its rounds of jobs and returns them with their
/// results.
struct Lane<T> {
    jobs: mpsc::SyncSender<Vec<Job>>,
    results: mpsc::Receiver<Result<Vec<(Job, T)>, String>>,
}

/// Evaluates chunks in rounds on worker threads, each with its own
/// evaluator, and hands every result to `take` in chunk order.
///
/// `next` yields the chunks in order and `None` after the final one. A round
/// gives each worker up to [`CHUNKS_PER_WORKER`] chunks. When `next` fails,
/// the chunks read before the failure are still evaluated and taken first,
/// so errors are reported in the order a sequential reader would meet them.
fn run_chunks<T: Send, W>(
    scheme: &Scheme,
    key: &[u8],
    header: &[u8; HEADER_BYTES],
    mut next: impl FnMut() -> Result<Option<Job>, String>,
    work: &W,
    mut take: impl FnMut(&Job, T) -> Result<(), String>,
) -> Result<(), String>
where
    W: Fn(&mut Calls<'_>, &Job) -> Result<T, String> + Sync,
{
    let workers = std::thread::available_parallelism()
        .map_or(1, std::num::NonZeroUsize::get)
        .clamp(1, MAX_WORKERS);
    std::thread::scope(|scope| {
        let mut lanes = Vec::with_capacity(workers);
        for _ in 0..workers {
            let (jobs, incoming) = mpsc::sync_channel::<Vec<Job>>(1);
            let (outgoing, results) = mpsc::sync_channel(1);
            let spawned = std::thread::Builder::new().spawn_scoped(scope, move || {
                chunk_worker(scheme, key, header, work, &incoming, &outgoing);
            });
            if spawned.is_err() {
                break;
            }
            lanes.push(Lane { jobs, results });
        }
        if lanes.is_empty() {
            return Err(render_cli_error(
                CliDiagnosticCode::CryptInput,
                "could not start a thread to evaluate the scheme",
                "the operating system refused to create one",
            ));
        }
        loop {
            let mut stop = None;
            let mut round = Vec::with_capacity(lanes.len());
            for lane in &lanes {
                let mut batch = Vec::with_capacity(CHUNKS_PER_WORKER);
                while stop.is_none() && batch.len() < CHUNKS_PER_WORKER {
                    match next() {
                        Ok(Some(job)) => batch.push(job),
                        Ok(None) => stop = Some(Ok(())),
                        Err(error) => stop = Some(Err(error)),
                    }
                }
                if batch.is_empty() {
                    break;
                }
                lane.jobs.send(batch).map_err(|_| worker_failure())?;
                round.push(lane);
                if stop.is_some() {
                    break;
                }
            }
            for lane in round {
                for (job, result) in lane.results.recv().map_err(|_| worker_failure())?? {
                    take(&job, result)?;
                }
            }
            if let Some(stop) = stop {
                return stop;
            }
        }
    })
}

/// A worker's loop: it prepares its evaluator on its first round and
/// evaluates every round it receives until the stream ends.
fn chunk_worker<T, W>(
    scheme: &Scheme,
    key: &[u8],
    header: &[u8; HEADER_BYTES],
    work: &W,
    incoming: &mpsc::Receiver<Vec<Job>>,
    outgoing: &mpsc::SyncSender<Result<Vec<(Job, T)>, String>>,
) where
    W: Fn(&mut Calls<'_>, &Job) -> Result<T, String>,
{
    let mut calls = None;
    for batch in incoming {
        let prepared = match calls.take() {
            Some(prepared) => Ok(prepared),
            None => Calls::new(scheme, key, header),
        };
        let outcome = match prepared {
            Ok(mut prepared) => {
                let outcome = batch
                    .into_iter()
                    .map(|job| work(&mut prepared, &job).map(|result| (job, result)))
                    .collect();
                calls = Some(prepared);
                outcome
            }
            Err(error) => Err(error),
        };
        if outgoing.send(outcome).is_err() {
            return;
        }
    }
}

fn worker_failure() -> String {
    render_cli_error(
        CliDiagnosticCode::MissingPhaseArtifact,
        "a thread evaluating the scheme stopped unexpectedly",
        "this is an internal compiler failure",
    )
}

/// A scheme's three specs, prepared for repeated calls under one key.
struct Calls<'scheme> {
    scheme: &'scheme Scheme,
    evaluator: Evaluator<'scheme>,
    seal: &'scheme CoreFunction,
    open: &'scheme CoreFunction,
    authentic: &'scheme CoreFunction,
    key: CoreValue,
    header: CoreValue,
}

impl<'scheme> Calls<'scheme> {
    fn new(scheme: &'scheme Scheme, key: &[u8], header: &[u8]) -> Result<Self, String> {
        let evaluator = Evaluator::new(&scheme.core).ok_or_else(|| {
            render_cli_error(
                CliDiagnosticCode::Scheme,
                format_args!("could not prepare the scheme `{}`", scheme.name),
                "its literal storage could not be reserved",
            )
        })?;
        let find = |name: &str| evaluator.function(name).ok_or_else(inconsistent_shape);
        Ok(Self {
            scheme,
            seal: find("seal")?,
            open: find("open")?,
            authentic: find("authentic")?,
            key: bytes_value(key).ok_or_else(inconsistent_shape)?,
            header: bytes_value(header).ok_or_else(inconsistent_shape)?,
            evaluator,
        })
    }

    fn call(
        &mut self,
        function: &'scheme CoreFunction,
        nonce: &[u8],
        data: &[u8],
    ) -> Result<CoreValue, String> {
        let arguments = [
            self.key.clone(),
            bytes_value(nonce).ok_or_else(inconsistent_shape)?,
            self.header.clone(),
            bytes_value(data).ok_or_else(inconsistent_shape)?,
        ];
        let result = self
            .evaluator
            .call(function, &arguments, STEPS_PER_CALL)
            .ok_or_else(inconsistent_shape)?;
        if result.has_errors() {
            return Err(render_diagnostics(
                &self.scheme.sources,
                result.diagnostics(),
            ));
        }
        result.into_value().ok_or_else(inconsistent_shape)
    }

    fn seal(&mut self, nonce: &[u8], plaintext: &[u8]) -> Result<Vec<u8>, String> {
        let value = self.call(self.seal, nonce, plaintext)?;
        value_bytes(&value).ok_or_else(inconsistent_shape)
    }

    fn open(&mut self, nonce: &[u8], sealed: &[u8]) -> Result<Vec<u8>, String> {
        let value = self.call(self.open, nonce, sealed)?;
        value_bytes(&value).ok_or_else(inconsistent_shape)
    }

    fn authentic(&mut self, nonce: &[u8], sealed: &[u8]) -> Result<bool, String> {
        match self.call(self.authentic, nonce, sealed)? {
            CoreValue::Bool(value) => Ok(value),
            _ => Err(inconsistent_shape()),
        }
    }
}

fn bytes_value(bytes: &[u8]) -> Option<CoreValue> {
    let ty = ArrayType::new(CoreType::Word8, u32::try_from(bytes.len()).ok()?)?;
    let elements = bytes.iter().copied().map(CoreValue::Word8).collect();
    CoreArray::new(ty, elements).map(CoreValue::Array)
}

fn value_bytes(value: &CoreValue) -> Option<Vec<u8>> {
    let CoreValue::Array(array) = value else {
        return None;
    };
    array
        .elements()
        .iter()
        .map(|element| match element {
            CoreValue::Word8(byte) => Some(*byte),
            _ => None,
        })
        .collect()
}

/// Seals one zero chunk and returns the steps it took.
fn measure_seal(scheme: &Scheme) -> Result<usize, String> {
    let zeros = |length: usize| vec![0; length];
    let mut calls = Calls::new(scheme, &zeros(scheme.shape.key), &zeros(HEADER_BYTES))?;
    let arguments = [
        calls.key.clone(),
        bytes_value(&zeros(scheme.shape.nonce)).ok_or_else(inconsistent_shape)?,
        calls.header.clone(),
        bytes_value(&zeros(scheme.shape.chunk)).ok_or_else(inconsistent_shape)?,
    ];
    let seal = calls.seal;
    let result = calls
        .evaluator
        .call(seal, &arguments, STEPS_PER_CALL)
        .ok_or_else(inconsistent_shape)?;
    if result.has_errors() {
        return Err(render_diagnostics(&scheme.sources, result.diagnostics()));
    }
    Ok(result.steps())
}

// ---- Schemes ---------------------------------------------------------------

fn scheme_for_key(options: &SealOptions, key: &Key) -> Result<Scheme, String> {
    let scheme = match &options.scheme {
        Some(scheme) => resolve_scheme(scheme, options.edition)?,
        None => match BUILTINS.iter().find(|builtin| builtin.name == key.scheme) {
            Some(builtin) => compile_builtin(builtin, options.edition)?,
            None => {
                return Err(render_cli_error(
                    CliDiagnosticCode::Scheme,
                    format_args!(
                        "the key is for the scheme `{}`, which is not built in",
                        key.scheme
                    ),
                    "name its Orange program with --scheme PATH",
                ));
            }
        },
    };
    if scheme.name != key.scheme {
        return Err(render_cli_error(
            CliDiagnosticCode::KeyFile,
            format_args!(
                "the key is for the scheme `{}`, not `{}`",
                key.scheme, scheme.name
            ),
            "each key belongs to one scheme; make one for this scheme with `orangec keygen --scheme`",
        ));
    }
    if scheme.shape.key != key.bytes.len() {
        return Err(render_cli_error(
            CliDiagnosticCode::KeyFile,
            format_args!(
                "the key holds {} bytes, but the scheme `{}` takes {}",
                key.bytes.len(),
                scheme.name,
                scheme.shape.key
            ),
            "make a new key with `orangec keygen --scheme`",
        ));
    }
    Ok(scheme)
}

/// Resolves a built-in scheme name or the path of an Orange program.
fn resolve_scheme(spelling: &OsStr, edition: Edition) -> Result<Scheme, String> {
    if let Some(builtin) = spelling
        .to_str()
        .and_then(|name| BUILTINS.iter().find(|builtin| builtin.name == name))
    {
        return compile_builtin(builtin, edition);
    }
    let bytes = spelling.as_encoded_bytes();
    let looks_like_path = bytes.contains(&b'/') || bytes.ends_with(b".or");
    if !looks_like_path {
        let shown = RenderedSourceName::try_from_os_str(spelling)
            .map(|name| name.to_string())
            .unwrap_or_default();
        return Err(render_cli_error(
            CliDiagnosticCode::Scheme,
            format_args!("unknown scheme `{shown}`"),
            format_args!(
                "the built-in schemes are {}; a path to an Orange program names any other",
                BUILTINS
                    .iter()
                    .map(|builtin| builtin.name)
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        ));
    }
    let mut remaining = super::MAX_SOURCE_BYTES_PER_INVOCATION;
    compile_path_scheme(Path::new(spelling), edition, &mut remaining)
}

/// Reads and compiles the scheme program at `path` with the modules it uses.
/// The program and its modules share one invocation's source budget,
/// `remaining_source_bytes`, as the sources of `orangec check` do.
fn compile_path_scheme(
    path: &Path,
    edition: Edition,
    remaining_source_bytes: &mut usize,
) -> Result<Scheme, String> {
    let display =
        RenderedSourceName::try_from_os_str(path.as_os_str()).map_err(super::source_name_error)?;
    let bytes = read_source(path, &mut io::empty(), remaining_source_bytes)
        .map_err(|error| render_read_source_error(&display, error))?;
    let text = String::from_utf8(bytes).map_err(|error| {
        render_cli_error(
            CliDiagnosticCode::InvalidUtf8,
            format_args!("source file `{display}` is not valid UTF-8"),
            format_args!(
                "invalid byte sequence begins at byte offset {}",
                error.utf8_error().valid_up_to()
            ),
        )
    })?;
    let origin = display.to_string();
    compile_scheme(
        display,
        text,
        edition,
        origin,
        Some((path, remaining_source_bytes)),
    )
}

fn compile_builtin(builtin: &Builtin, edition: Edition) -> Result<Scheme, String> {
    let display =
        RenderedSourceName::try_from_text(builtin.path).map_err(super::source_name_error)?;
    let scheme = compile_scheme(
        display,
        String::from(builtin.text),
        edition,
        String::from(builtin.path),
        None,
    )?;
    if scheme.name != builtin.name {
        return Err(inconsistent_shape());
    }
    Ok(scheme)
}

/// Compiles a scheme program. A program read from a path may use modules,
/// which are read from beside it as `orangec check` reads them, under what
/// remains of the budget its own bytes were charged to; a built-in scheme is
/// one module.
fn compile_scheme(
    display: RenderedSourceName,
    text: String,
    edition: Edition,
    origin: String,
    path: Option<(&Path, &mut usize)>,
) -> Result<Scheme, String> {
    let mut sources = SourceMap::try_new().map_err(super::source_name_error)?;
    let id = sources
        .add_with_rendered_name(display, text)
        .map_err(super::source_limit_error_without_name)?;
    let source = sources.get(id).ok_or_else(inconsistent_shape)?;
    let lexed = lex(source, edition);
    if lexed.has_errors() {
        return Err(render_diagnostics(&sources, lexed.diagnostics()));
    }
    let parsed = parse(source, &lexed);
    let ast = match classify_phase_result(parsed.ast(), parsed.diagnostics()) {
        PhaseResult::Complete(ast) => ast,
        PhaseResult::Diagnosed(diagnostics) => {
            return Err(render_diagnostics(&sources, diagnostics));
        }
        PhaseResult::Missing => return Err(inconsistent_shape()),
    };
    let modules = match path {
        Some((path, remaining_source_bytes)) if !ast.module().uses().is_empty() => {
            super::load_used_modules(path, ast, &mut sources, edition, remaining_source_bytes)?
        }
        _ => Vec::new(),
    };
    let program = modules
        .iter()
        .filter_map(|(id, ast)| Some((sources.get(*id)?, ast)))
        .collect::<Vec<_>>();
    let source = sources
        .get(id)
        .filter(|_| program.len() == modules.len())
        .ok_or_else(inconsistent_shape)?;
    let analyzed = analyze_program((source, ast), &program);
    if !analyzed.diagnostics().is_empty() {
        return Err(render_diagnostics(&sources, analyzed.diagnostics()));
    }
    let core = analyzed.into_core().ok_or_else(inconsistent_shape)?;
    let name = String::from(core.name());
    let shape = interface_shape(&core).map_err(|problem| {
        render_cli_error(
            CliDiagnosticCode::Scheme,
            format_args!(
                "`{}` at {origin} does not implement the sealing interface",
                escape_display_text(&name)
            ),
            problem,
        )
    })?;
    Ok(Scheme {
        name,
        origin,
        sources,
        core,
        shape,
    })
}

/// Reads the interface's sizes from the types of `seal`, `open`, and
/// `authentic`, or explains how they differ from the interface.
fn interface_shape(core: &CoreModule) -> Result<Shape, String> {
    if !valid_scheme_name(core.name()) {
        return Err(format!(
            "a scheme's module name is 1 to {MAX_SCHEME_NAME_BYTES} ASCII letters, digits, or underscores"
        ));
    }
    let function = |name: &str| {
        core.entry_functions()
            .iter()
            .find(|function| function.name() == name)
            .ok_or_else(|| format!("it has no spec named `{name}`"))
    };
    let bytes = |ty: CoreType| {
        ty.as_array()
            .filter(|array| array.element() == CoreType::Word8)
            .and_then(|array| usize::try_from(array.length()).ok())
    };
    let seal = function("seal")?;
    let seal_signature = "spec seal(key: Word[8]^K, nonce: Word[8]^N, ad: Word[8]^64, plaintext: Word[8]^C) -> Word[8]^S";
    let [key, nonce, ad, chunk] = byte_parameters(seal).ok_or_else(|| expected(seal_signature))?;
    let sealed = bytes(seal.result_type()).ok_or_else(|| expected(seal_signature))?;
    if ad != HEADER_BYTES {
        return Err(expected(seal_signature));
    }
    let tag = sealed
        .checked_sub(chunk)
        .filter(|tag| *tag > 0)
        .ok_or_else(|| {
            String::from("`seal` must return more bytes than it takes: the chunk, then the tag")
        })?;
    let shape = Shape {
        key,
        nonce,
        tag,
        chunk,
    };
    let open = function("open")?;
    let open_signature = format!(
        "spec open(key: Word[8]^{key}, nonce: Word[8]^{nonce}, ad: Word[8]^64, sealed: Word[8]^{sealed}) -> Word[8]^{chunk}"
    );
    if byte_parameters(open) != Some([key, nonce, HEADER_BYTES, sealed])
        || bytes(open.result_type()) != Some(chunk)
    {
        return Err(expected(&open_signature));
    }
    let authentic = function("authentic")?;
    let authentic_signature = format!(
        "spec authentic(key: Word[8]^{key}, nonce: Word[8]^{nonce}, ad: Word[8]^64, sealed: Word[8]^{sealed}) -> Bool"
    );
    if byte_parameters(authentic) != Some([key, nonce, HEADER_BYTES, sealed])
        || authentic.result_type() != CoreType::Bool
    {
        return Err(expected(&authentic_signature));
    }
    check_shape(shape)?;
    Ok(shape)
}

fn byte_parameters(function: &CoreFunction) -> Option<[usize; 4]> {
    let mut lengths = [0; 4];
    let parameters = function.parameters();
    if parameters.len() != lengths.len() {
        return None;
    }
    for (slot, ty) in lengths.iter_mut().zip(parameters) {
        let array = ty
            .as_array()
            .filter(|array| array.element() == CoreType::Word8)?;
        *slot = usize::try_from(array.length()).ok()?;
    }
    Some(lengths)
}

fn expected(signature: &str) -> String {
    format!("expected `{signature}`")
}

fn check_shape(shape: Shape) -> Result<(), String> {
    let max_nonce = PREFIX_FIELD.len().saturating_add(COUNTER_AND_FLAG_BYTES);
    let problem = if !(MIN_KEY_BYTES..=MAX_KEY_BYTES).contains(&shape.key) {
        format!("keys are {MIN_KEY_BYTES} to {MAX_KEY_BYTES} bytes")
    } else if !(MIN_NONCE_BYTES..=max_nonce).contains(&shape.nonce) {
        format!("nonces are {MIN_NONCE_BYTES} to {max_nonce} bytes")
    } else if shape.tag < MIN_TAG_BYTES {
        format!("tags are at least {MIN_TAG_BYTES} bytes")
    } else if shape.chunk < MIN_CHUNK_BYTES {
        format!("chunks are at least {MIN_CHUNK_BYTES} bytes")
    } else if shape
        .sealed()
        .and_then(|sealed| u32::try_from(sealed).ok())
        .is_none_or(|sealed| sealed > orange_compiler::MAX_ARRAY_LENGTH)
    {
        format!(
            "a sealed chunk is at most {} bytes",
            orange_compiler::MAX_ARRAY_LENGTH
        )
    } else {
        return Ok(());
    };
    Err(problem)
}

fn valid_scheme_name(name: &str) -> bool {
    (1..=MAX_SCHEME_NAME_BYTES).contains(&name.len())
        && name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
}

// ---- Keys --------------------------------------------------------------------

fn key_path(options: &SealOptions) -> Result<PathBuf, String> {
    match &options.key {
        Some(path) => Ok(path.clone()),
        None => default_key_path(),
    }
}

/// `$XDG_CONFIG_HOME/orange/key`, or `$HOME/.config/orange/key`.
fn default_key_path() -> Result<PathBuf, String> {
    let absolute = |name: &str| {
        std::env::var_os(name)
            .map(PathBuf::from)
            .filter(|path| path.is_absolute())
    };
    if let Some(config) = absolute("XDG_CONFIG_HOME") {
        return Ok(config.join("orange").join("key"));
    }
    if let Some(home) = absolute("HOME") {
        return Ok(home.join(".config").join("orange").join("key"));
    }
    Err(render_cli_error(
        CliDiagnosticCode::KeyFile,
        "there is no default key path",
        "set HOME or XDG_CONFIG_HOME, or name a key file with --key",
    ))
}

fn key_file_text(scheme: &str, bytes: &[u8]) -> String {
    let mut text = format!(
        "# Orange secret key for the scheme {scheme}.\n# Whoever holds this file can open and forge everything sealed with it.\n{KEY_FILE_TAG} {KEY_FILE_VERSION} {scheme} "
    );
    for byte in bytes {
        let _ = write!(text, "{byte:02x}");
    }
    text.push('\n');
    text
}

fn read_key(path: &Path) -> Result<Key, String> {
    let shown = shown_path(path);
    let key_error =
        |message: String, note: &str| render_cli_error(CliDiagnosticCode::KeyFile, message, note);
    let file = open_source_file(path).map_err(|error| {
        if error.kind() == io::ErrorKind::NotFound {
            key_error(
                format!("there is no key at `{shown}`"),
                "make one with `orangec keygen`, or name one with --key",
            )
        } else {
            key_error(
                format!("could not read the key file `{shown}`"),
                io_error_reason(&error),
            )
        }
    })?;
    let metadata = file.metadata().map_err(|error| {
        key_error(
            format!("could not read the key file `{shown}`"),
            io_error_reason(&error),
        )
    })?;
    if !metadata.is_file() {
        return Err(key_error(
            format!("could not read the key file `{shown}`"),
            "path does not name a regular file",
        ));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        if metadata.permissions().mode() & 0o077 != 0 {
            return Err(key_error(
                format!("the key file `{shown}` can be read or written by other users"),
                "restrict it to its owner with `chmod 600`",
            ));
        }
    }
    let mut text = String::new();
    file.take(MAX_KEY_FILE_BYTES.saturating_add(1))
        .read_to_string(&mut text)
        .map_err(|_| {
            key_error(
                format!("`{shown}` is not an Orange key file"),
                "it is not UTF-8 text",
            )
        })?;
    if u64::try_from(text.len()).unwrap_or(u64::MAX) > MAX_KEY_FILE_BYTES {
        return Err(key_error(
            format!("`{shown}` is not an Orange key file"),
            "a key file is at most 4096 bytes",
        ));
    }
    parse_key(&text).map_err(|note| key_error(format!("`{shown}` is not an Orange key file"), note))
}

fn parse_key(text: &str) -> Result<Key, &'static str> {
    let mut lines = text
        .lines()
        .filter(|line| !line.is_empty() && !line.starts_with('#'));
    let line = lines.next().ok_or("it holds no key line")?;
    if lines.next().is_some() {
        return Err("it holds more than one key line");
    }
    let fields = line.split(' ').collect::<Vec<_>>();
    let [tag, version, scheme, hex] = fields.as_slice() else {
        return Err("its key line is not `orange-key 1 SCHEME HEX`");
    };
    if *tag != KEY_FILE_TAG || *version != KEY_FILE_VERSION || !valid_scheme_name(scheme) {
        return Err("its key line is not `orange-key 1 SCHEME HEX`");
    }
    let bytes = decode_hex(hex).ok_or("its key is not lowercase hexadecimal")?;
    if !(MIN_KEY_BYTES..=MAX_KEY_BYTES).contains(&bytes.len()) {
        return Err("its key is not 16 to 64 bytes long");
    }
    Ok(Key {
        scheme: String::from(*scheme),
        bytes,
    })
}

fn decode_hex(hex: &str) -> Option<Vec<u8>> {
    let digit = |byte: u8| match byte {
        b'0'..=b'9' => byte.checked_sub(b'0'),
        b'a'..=b'f' => byte.checked_sub(b'a')?.checked_add(10),
        _ => None,
    };
    let pairs = hex.as_bytes().chunks_exact(2);
    if !pairs.remainder().is_empty() {
        return None;
    }
    pairs
        .map(|pair| match pair {
            [high, low] => digit(*high)?.checked_mul(16)?.checked_add(digit(*low)?),
            _ => None,
        })
        .collect()
}

// ---- Randomness and files ----------------------------------------------------

/// Reads `count` bytes from the operating system's random-number generator.
fn random_bytes(count: usize) -> Result<Vec<u8>, String> {
    let failure = |note: &str| {
        render_cli_error(
            CliDiagnosticCode::Randomness,
            "could not read operating-system randomness",
            note,
        )
    };
    #[cfg(unix)]
    {
        use std::os::unix::fs::FileTypeExt as _;
        let mut source =
            File::open("/dev/urandom").map_err(|error| failure(io_error_reason(&error)))?;
        let is_device = source
            .metadata()
            .is_ok_and(|metadata| metadata.file_type().is_char_device());
        if !is_device {
            return Err(failure("`/dev/urandom` is not a character device"));
        }
        let mut bytes = vec![0; count];
        let read =
            read_full(&mut source, &mut bytes).map_err(|error| failure(io_error_reason(&error)))?;
        if read != count {
            return Err(failure("`/dev/urandom` ended early"));
        }
        Ok(bytes)
    }
    #[cfg(not(unix))]
    {
        let _ = count;
        Err(failure("this platform has no supported randomness source"))
    }
}

/// Fills `buffer` from `reader` and returns how many bytes it read: fewer
/// than its length only at the end of the input.
fn read_full(reader: &mut impl Read, buffer: &mut [u8]) -> io::Result<usize> {
    let mut filled = 0;
    while let Some(rest) = buffer.get_mut(filled..).filter(|rest| !rest.is_empty()) {
        let read = read_retry_interrupted(reader, rest)?;
        if read == 0 {
            break;
        }
        filled = filled
            .checked_add(read)
            .filter(|filled| *filled <= buffer.len())
            .ok_or_else(|| io::Error::from(io::ErrorKind::InvalidData))?;
    }
    Ok(filled)
}

fn open_input(path: &Path) -> Result<BufReader<File>, String> {
    let shown = shown_path(path);
    let file = open_source_file(path).map_err(|error| read_error(&shown, &error))?;
    let regular = file
        .metadata()
        .map_err(|error| read_error(&shown, &error))?
        .is_file();
    if !regular {
        return Err(render_cli_error(
            CliDiagnosticCode::CryptInput,
            format_args!("could not read `{shown}`"),
            "path does not name a regular file",
        ));
    }
    Ok(BufReader::with_capacity(IO_BUFFER_BYTES, file))
}

fn read_error(shown: &str, error: &io::Error) -> String {
    // `open_source_file` refuses a final symbolic link with ELOOP on Linux.
    const ELOOP: i32 = 40;
    let note = if cfg!(target_os = "linux") && error.raw_os_error() == Some(ELOOP) {
        "path names a symbolic link; name the file it points to"
    } else {
        io_error_reason(error)
    };
    render_cli_error(
        CliDiagnosticCode::CryptInput,
        format_args!("could not read `{shown}`"),
        note,
    )
}

fn reason_with_existing(error: &io::Error) -> &'static str {
    if error.kind() == io::ErrorKind::AlreadyExists {
        "a file already exists there"
    } else {
        io_error_reason(error)
    }
}

fn create_new_private(path: &Path) -> io::Result<File> {
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt as _;
        options.mode(0o600);
    }
    options.open(path)
}

fn create_new_output(path: &Path, private: bool) -> io::Result<File> {
    if private {
        create_new_private(path)
    } else {
        OpenOptions::new().write(true).create_new(true).open(path)
    }
}

fn create_private_directory(path: &Path) -> io::Result<()> {
    let mut builder = fs::DirBuilder::new();
    builder.recursive(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt as _;
        builder.mode(0o700);
    }
    builder.create(path)
}

/// An output written beside its destination and published only when complete.
///
/// Until [`Self::publish`] succeeds the bytes live in `DESTINATION.partial`,
/// which is removed on failure, so no partial file ever carries the
/// destination's name. Publishing never replaces an existing file.
struct PendingOutput {
    destination: PathBuf,
    shown: String,
    partial: PathBuf,
    private: bool,
    writer: Option<BufWriter<File>>,
}

impl PendingOutput {
    fn create(destination: &Path, private: bool) -> Result<Self, String> {
        let shown = shown_path(destination);
        let exists = |shown: &str| {
            render_cli_error(
                CliDiagnosticCode::CryptOutput,
                format_args!("`{shown}` already exists"),
                "orangec never replaces a file; remove it or name another with -o",
            )
        };
        if destination.symlink_metadata().is_ok() {
            return Err(exists(&shown));
        }
        let mut partial = destination.as_os_str().to_owned();
        partial.push(".partial");
        let partial = PathBuf::from(partial);
        let file = create_new_output(&partial, private).map_err(|error| {
            render_cli_error(
                CliDiagnosticCode::CryptOutput,
                format_args!("could not create `{}`", shown_path(&partial)),
                reason_with_existing(&error),
            )
        })?;
        Ok(Self {
            destination: destination.to_path_buf(),
            shown,
            partial,
            private,
            writer: Some(BufWriter::with_capacity(IO_BUFFER_BYTES, file)),
        })
    }

    fn write(&mut self, bytes: &[u8]) -> Result<(), String> {
        let written = match &mut self.writer {
            Some(writer) => writer.write_all(bytes),
            None => Err(io::Error::from(io::ErrorKind::BrokenPipe)),
        };
        written.map_err(|error| self.write_error(&error))
    }

    fn write_error(&self, error: &io::Error) -> String {
        render_cli_error(
            CliDiagnosticCode::CryptOutput,
            format_args!("could not write `{}`", shown_path(&self.partial)),
            io_error_reason(error),
        )
    }

    fn publish(mut self) -> Result<(), String> {
        self.finish()?;
        // A hard link publishes without replacing; where links are not
        // supported, a copy does, into a file created only if none exists.
        let published = match fs::hard_link(&self.partial, &self.destination) {
            Ok(()) => fs::remove_file(&self.partial),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => Err(error),
            Err(_) => self.copy_to_destination(),
        };
        published.map_err(|error| {
            render_cli_error(
                CliDiagnosticCode::CryptOutput,
                format_args!("could not create `{}`", self.shown),
                reason_with_existing(&error),
            )
        })
    }

    /// Flushes and syncs the partial file and closes it.
    fn finish(&mut self) -> Result<(), String> {
        let Some(writer) = self.writer.take() else {
            return Err(inconsistent_shape());
        };
        let file = writer
            .into_inner()
            .map_err(|error| self.write_error(error.error()))?;
        file.sync_all().map_err(|error| self.write_error(&error))
    }

    /// Copies the finished partial file to the destination, which must not
    /// exist: it is created with `create_new`, so a file that appeared after
    /// the check in [`PendingOutput::create`] is refused, never replaced.
    fn copy_to_destination(&self) -> io::Result<()> {
        let mut destination = create_new_output(&self.destination, self.private)?;
        let copied = File::open(&self.partial)
            .and_then(|mut partial| io::copy(&mut partial, &mut destination))
            .and_then(|_| destination.sync_all());
        match copied {
            Ok(()) => fs::remove_file(&self.partial),
            Err(error) => {
                drop(destination);
                let _ = fs::remove_file(&self.destination);
                Err(error)
            }
        }
    }
}

impl Drop for PendingOutput {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.partial);
    }
}

// ---- Helpers -----------------------------------------------------------------

fn single_input(options: &SealOptions) -> Result<&Path, String> {
    match options.inputs.as_slice() {
        [input] => Ok(input),
        _ => Err(inconsistent_shape()),
    }
}

fn shown_path(path: &Path) -> String {
    RenderedSourceName::try_from_os_str(path.as_os_str())
        .map(|name| name.to_string())
        .unwrap_or_else(|_| String::from("<path>"))
}

fn sealed_file_error(shown: &str, problem: String) -> String {
    render_cli_error(
        CliDiagnosticCode::SealedFile,
        format_args!("`{shown}` is not a sealed file this orangec can open"),
        problem,
    )
}

fn inconsistent_shape() -> String {
    render_cli_error(
        CliDiagnosticCode::MissingPhaseArtifact,
        "the sealing commands reached an inconsistent state",
        "this is an internal compiler failure",
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn builtin(name: &str) -> Scheme {
        let builtin = BUILTINS
            .iter()
            .find(|builtin| builtin.name == name)
            .unwrap();
        compile_builtin(builtin, Edition::E2026).unwrap()
    }

    #[cfg(unix)]
    #[test]
    fn a_path_scheme_and_its_modules_share_one_source_budget() {
        let directory = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../target/orangec-tests")
            .join(format!("orangec-scheme-budget-{}", std::process::id()));
        let _ = fs::remove_dir_all(&directory);
        fs::create_dir_all(&directory).unwrap();
        let aead = BUILTINS
            .iter()
            .find(|builtin| builtin.name == "chacha20_poly1305")
            .unwrap()
            .text
            .replace("module chacha20_poly1305 {", "module aead {");
        let layered = concat!(
            "edition 2026;\n",
            "module layered {\n",
            "  use aead;\n",
            "  spec seal(key: Word[8]^32, nonce: Word[8]^12, ad: Word[8]^64, ",
            "plaintext: Word[8]^240) -> Word[8]^256 {\n",
            "    aead::seal(key, nonce, ad, plaintext)\n",
            "  }\n",
            "  spec open(key: Word[8]^32, nonce: Word[8]^12, ad: Word[8]^64, ",
            "sealed: Word[8]^256) -> Word[8]^240 {\n",
            "    aead::open(key, nonce, ad, sealed)\n",
            "  }\n",
            "  spec authentic(key: Word[8]^32, nonce: Word[8]^12, ad: Word[8]^64, ",
            "sealed: Word[8]^256) -> Bool {\n",
            "    aead::authentic(key, nonce, ad, sealed)\n",
            "  }\n",
            "}\n",
        );
        fs::write(directory.join("aead.or"), &aead).unwrap();
        let path = directory.join("layered.or");
        fs::write(&path, layered).unwrap();
        let total = layered.len() + aead.len();

        let mut remaining = total;
        let scheme = compile_path_scheme(&path, Edition::E2026, &mut remaining).unwrap();
        assert_eq!(scheme.name, "layered");
        assert_eq!(remaining, 0);

        // The scheme's own bytes are charged first, so one byte less leaves
        // too little for the module it uses.
        let mut remaining = total - 1;
        let error = compile_path_scheme(&path, Edition::E2026, &mut remaining)
            .err()
            .unwrap();
        assert!(
            error.starts_with("error[ORC1008]: source input `"),
            "{error}"
        );
        assert!(
            error.contains("aead.or` exceeds the remaining invocation budget"),
            "{error}"
        );
        assert!(error.ends_with(
            "  = note: `use aead;` in module `layered` reads the module `aead` from this file\n"
        ));
        fs::remove_dir_all(&directory).unwrap();
    }

    #[test]
    fn builtin_schemes_declare_their_shapes() {
        let shapes = BUILTINS
            .iter()
            .map(|entry| (entry.name, builtin(entry.name).shape))
            .collect::<Vec<_>>();
        let shape = |key, nonce| Shape {
            key,
            nonce,
            tag: 16,
            chunk: 240,
        };
        assert_eq!(
            shapes,
            [
                ("xchacha20_poly1305", shape(32, 24)),
                ("chacha20_poly1305", shape(32, 12)),
                ("ascon_aead128", shape(16, 16)),
            ]
        );
        assert_eq!(
            BUILTINS.first().map(|builtin| builtin.name),
            Some(DEFAULT_SCHEME)
        );
    }

    #[test]
    fn headers_round_trip_and_parse_strictly() {
        let scheme = builtin("chacha20_poly1305");
        let prefix = [1, 2, 3, 4, 5, 6, 7];
        let header = header(&scheme, &prefix).unwrap();
        assert_eq!(
            &header[..16],
            b"orange\x00\x01\x20\x0c\x10\x00\x00\x00\x00\xf0"
        );
        assert_eq!(&header[16..40], b"chacha20_poly1305\0\0\0\0\0\0\0");
        assert_eq!(&header[40..47], &prefix);
        assert!(header[47..].iter().all(|byte| *byte == 0));
        let parsed = parse_header(&header).unwrap();
        assert_eq!(parsed.scheme, "chacha20_poly1305");
        assert_eq!(parsed.shape, scheme.shape);
        assert_eq!(parsed.prefix, prefix);

        let altered = |at: usize, byte: u8| {
            let mut altered = header;
            altered[at] = byte;
            parse_header(&altered).err().unwrap()
        };
        assert_eq!(
            altered(0, b'O'),
            "it does not begin with the `orange` magic"
        );
        assert_eq!(altered(6, 1), "it does not begin with the `orange` magic");
        assert_eq!(
            altered(7, 2),
            "it uses format version 2, and this orangec reads version 1"
        );
        assert_eq!(altered(11, 1), "its reserved header byte is not zero");
        assert_eq!(
            altered(8, 8),
            "its header names impossible sizes: keys are 16 to 64 bytes"
        );
        assert_eq!(
            altered(9, 30),
            "its header names impossible sizes: nonces are 12 to 29 bytes"
        );
        assert_eq!(
            altered(15, 0xf1),
            "its header names impossible sizes: a sealed chunk is at most 256 bytes"
        );
        assert_eq!(altered(16, 0), "its header does not name a scheme");
        assert_eq!(altered(20, b'-'), "its header does not name a scheme");
        assert_eq!(altered(39, b'x'), "its header does not name a scheme");
        assert_eq!(altered(47, 1), "its nonce-prefix field is not zero-padded");
    }

    #[test]
    fn chunk_nonces_end_in_a_counter_and_a_final_flag() {
        assert_eq!(
            chunk_nonce(&[0xaa; 7], 0x0102_0304, false),
            [0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 1, 2, 3, 4, 0]
        );
        assert_eq!(
            chunk_nonce(&[0xbb; 19], u32::MAX, true)[19..],
            [0xff, 0xff, 0xff, 0xff, 1]
        );
    }

    #[test]
    fn the_final_chunk_is_padded_with_one_mark_and_zeros() {
        assert_eq!(pad(b"", 4).unwrap(), [0x80, 0, 0, 0]);
        assert_eq!(pad(b"abc", 4).unwrap(), *b"abc\x80");
        assert_eq!(pad(b"abcd", 4), None);
        assert_eq!(unpad(&[0x80, 0, 0, 0]), Some(&b""[..]));
        assert_eq!(unpad(b"ab\x80\x00"), Some(&b"ab"[..]));
        assert_eq!(unpad(b"a\x80\x80\x00"), Some(&b"a\x80"[..]));
        assert_eq!(unpad(b"ab\x00\x00"), None);
        assert_eq!(unpad(b"ab\x81\x00"), None);
        assert_eq!(unpad(&[0, 0, 0, 0]), None);
        for length in 0..16 {
            let data = vec![0x80; length];
            assert_eq!(unpad(&pad(&data, 16).unwrap()), Some(data.as_slice()));
        }
    }

    #[test]
    fn key_files_parse_strictly() {
        let text = key_file_text("ascon_aead128", &[0xab; 16]);
        assert_eq!(
            text,
            concat!(
                "# Orange secret key for the scheme ascon_aead128.\n",
                "# Whoever holds this file can open and forge everything sealed with it.\n",
                "orange-key 1 ascon_aead128 abababababababababababababababab\n",
            )
        );
        let key = parse_key(&text).unwrap();
        assert_eq!(key.scheme, "ascon_aead128");
        assert_eq!(key.bytes, [0xab; 16]);

        let line = |line: &str| parse_key(line).err().unwrap();
        assert_eq!(line(""), "it holds no key line");
        assert_eq!(
            line("orange-key 1 a 00000000000000000000000000000000\norange-key 1 a 00\n"),
            "it holds more than one key line"
        );
        for malformed in [
            "orange-key 1 a",
            "orange-key 2 a 00000000000000000000000000000000",
            "orange-key 1 a-b 00000000000000000000000000000000",
            "orange-key  1 a 00000000000000000000000000000000",
        ] {
            assert_eq!(
                line(malformed),
                "its key line is not `orange-key 1 SCHEME HEX`",
                "{malformed}"
            );
        }
        assert_eq!(
            line("orange-key 1 a 0000000000000000000000000000000G"),
            "its key is not lowercase hexadecimal"
        );
        assert_eq!(
            line("orange-key 1 a 000000000000000000000000000000AB"),
            "its key is not lowercase hexadecimal"
        );
        assert_eq!(
            line("orange-key 1 a 0000"),
            "its key is not 16 to 64 bytes long"
        );
        assert_eq!(decode_hex("00ff7f"), Some(vec![0, 0xff, 0x7f]));
        assert_eq!(decode_hex("0"), None);
    }

    #[test]
    fn scheme_names_are_short_identifiers() {
        assert!(valid_scheme_name("ascon_aead128"));
        assert!(valid_scheme_name(&"x".repeat(24)));
        assert!(!valid_scheme_name(""));
        assert!(!valid_scheme_name(&"x".repeat(25)));
        assert!(!valid_scheme_name("a-b"));
        assert!(!valid_scheme_name("é"));
    }

    #[test]
    fn a_scheme_seals_what_it_opens() {
        let scheme = builtin("ascon_aead128");
        let mut calls = Calls::new(&scheme, &[7; 16], &[9; HEADER_BYTES]).unwrap();
        let nonce = chunk_nonce(&[5; 11], 3, true);
        let plaintext = pad(b"attack at dawn", 240).unwrap();
        let sealed = calls.seal(&nonce, &plaintext).unwrap();
        assert_eq!(sealed.len(), 256);
        assert!(calls.authentic(&nonce, &sealed).unwrap());
        assert_eq!(calls.open(&nonce, &sealed).unwrap(), plaintext);
        let other = chunk_nonce(&[5; 11], 3, false);
        assert!(!calls.authentic(&other, &sealed).unwrap());
        assert!(measure_seal(&scheme).unwrap() > 0);
    }

    #[test]
    fn publishing_never_replaces_a_file() {
        // Under the workspace's ignored `target` folder, as Tabula's tests do.
        let folder = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../target/orangec-unit-tests")
            .join(format!("publish-{}", std::process::id()));
        let _ = fs::remove_dir_all(&folder);
        fs::create_dir_all(&folder).unwrap();
        let written = |destination: &Path, private: bool| {
            let mut pending = PendingOutput::create(destination, private).unwrap();
            pending.write(b"opened").unwrap();
            pending
        };

        // The copy used where hard links are not supported.
        let copied = folder.join("copied");
        let mut pending = written(&copied, true);
        pending.finish().unwrap();
        pending.copy_to_destination().unwrap();
        assert_eq!(fs::read(&copied).unwrap(), b"opened");
        assert!(!pending.partial.exists());
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt as _;
            let mode = fs::metadata(&copied).unwrap().permissions().mode();
            assert_eq!(mode & 0o777, 0o600);
        }

        // A file that appears after `create` is kept, by either path.
        for (name, copy) in [("linked-late", false), ("copied-late", true)] {
            let late = folder.join(name);
            let mut pending = written(&late, false);
            fs::write(&late, b"theirs").unwrap();
            let partial = pending.partial.clone();
            if copy {
                pending.finish().unwrap();
                let error = pending.copy_to_destination().unwrap_err();
                assert_eq!(error.kind(), io::ErrorKind::AlreadyExists);
                drop(pending);
            } else {
                assert!(pending.publish().unwrap_err().contains("already exists"));
            }
            assert_eq!(fs::read(&late).unwrap(), b"theirs");
            assert!(!partial.exists());
        }
        fs::remove_dir_all(&folder).unwrap();
    }
}
