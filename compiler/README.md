# Orange compiler

Status: production-lineage, pre-alpha foundation

This workspace contains the first executable slice of the Orange compiler. It
is intentionally small, but its source identities, byte spans, language-edition
boundary, diagnostic codes, and deterministic token stream are permanent
interfaces to extend rather than a disposable prototype.

Nothing here makes a verification, correctness, constant-time, or production
readiness claim. In this slice, `orangec check` performs lexical and syntactic
validation for one deliberately small grammar. Type checking, name resolution,
proof checking, semantic analysis, lowering, and code generation do not exist
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
error and use exit status 1; command-line usage errors use status 2. A source
with lexical errors is not parsed, which avoids misleading syntax cascades.
File and standard-input reads stop at a deterministic 16 MiB per-source limit.
Larger inputs fail with `ORC1003` before lexing and are never buffered without a
bound.

## Frozen lexical boundary

The only supported language edition is Orange 2026. Its current lexical rules
are deliberately conservative:

- each source is at most 16 MiB of UTF-8, and spans are half-open UTF-8 byte
  ranges;
- whitespace is limited to tab, line feed, carriage return, and space; line
  feed, CRLF, and bare carriage return each form one logical line ending;
- identifiers use ASCII letters, digits, and `_` (the first character cannot be
  a digit);
- `edition`, `module`, `spec`, `impl`, `game`, `proof`, and `claim` are reserved;
- decimal, `0b` binary, and `0x` hexadecimal integers allow single underscores
  between digits;
- quoted strings have a small, validated escape set and cannot cross lines;
- `//` comments and nested `/* ... */` comments are trivia; and
- punctuation outside the minimal grammar is still tokenized but has no
  accepted syntactic or semantic role.

Adding syntax requires an edition-aware decision. Token names and `ORCxxxx`
diagnostic meanings are stable automation surfaces; wording and source excerpts
may improve without reusing a code for a different error.

## Minimal Orange 2026 grammar

The parser accepts exactly one edition declaration followed by exactly one
module. A module contains zero or more empty `spec` or `impl` function
declarations:

```text
source_file   = edition_decl module_decl EOF ;
edition_decl  = "edition" "2026" ";" ;
module_decl   = "module" IDENTIFIER "{" function_decl* "}" ;
function_decl = function_kind IDENTIFIER "(" ")" empty_body ;
function_kind = "spec" | "impl" ;
empty_body    = "{" "}" ;
```

For example:

```orange
edition 2026;
module demo {
  spec identity() {}
  impl rounds() {}
}
```

This grammar does not define imports, nested modules, parameters, return types,
statements, expressions, attributes, generic arguments, contracts, proofs, or
non-empty bodies. Duplicate names are syntactically valid because name binding
is not a parser responsibility. The parser assigns no execution, proof, type,
ABI, or other semantic meaning to accepted text.

## Layout

- `crates/orange-compiler`: reusable source, span, diagnostic, edition, lexer,
  syntax-tree, and parser library;
- `crates/orangec`: thin file/stdin CLI with deterministic `check` and `lex`
  behavior; and
- `fixtures/hello.or`: permanent positive syntax fixture.
