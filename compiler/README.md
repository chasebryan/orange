# Orange compiler

Status: production-lineage, pre-alpha foundation

This workspace contains the first executable slice of the Orange compiler. It
is intentionally small, but its source identities, byte spans, language-edition
boundary, diagnostic codes, and deterministic token stream are permanent
interfaces to extend rather than a disposable prototype.

Nothing here makes a verification, correctness, constant-time, or production
readiness claim. In this slice, `orangec check` performs **lexical validation
only**. Parsing, type checking, proof checking, and code generation do not exist
yet.

## Run

The workspace requires the pinned Rust 1.96.1 toolchain and has no third-party
Rust dependencies. This pre-alpha slice does not declare or test a lower MSRV.

```sh
cargo test --manifest-path compiler/Cargo.toml --workspace
cargo run --manifest-path compiler/Cargo.toml -p orangec -- check compiler/fixtures/hello.or
cargo run --manifest-path compiler/Cargo.toml -p orangec -- lex compiler/fixtures/hello.or
```

`orangec` accepts up to 256 source inputs in argument order. Regular files are
processed incrementally; `-` is the only stream input and reads standard input
at most once. Successful `check` commands are silent. Diagnostics go to standard
error and use exit status 1; command-line usage errors use status 2. File and
standard-input reads stop at a deterministic 16 MiB per-source limit. Larger
inputs fail with `ORC1003` before lexing and are never buffered without a bound.

## Frozen lexical boundary

The only supported language edition is Orange 2026. Its current lexical rules
are deliberately conservative:

- each source is at most 16 MiB of UTF-8, and spans are half-open UTF-8 byte
  ranges;
- identifiers use ASCII letters, digits, and `_` (the first character cannot be
  a digit);
- `module`, `spec`, `impl`, `game`, `proof`, and `claim` are reserved;
- decimal, `0b` binary, and `0x` hexadecimal integers allow single underscores
  between digits;
- quoted strings have a small, validated escape set and cannot cross lines;
- `//` comments and nested `/* ... */` comments are trivia; and
- punctuation is tokenized without assigning it a grammar or semantics yet.

Adding syntax requires an edition-aware decision. Token names and `ORCxxxx`
diagnostic meanings are stable automation surfaces; wording and source excerpts
may improve without reusing a code for a different error.

## Layout

- `crates/orange-compiler`: reusable source, span, diagnostic, edition, and lexer
  library;
- `crates/orangec`: thin file/stdin CLI with deterministic `check` and `lex`
  behavior; and
- `fixtures/hello.or`: permanent positive lexical fixture.
