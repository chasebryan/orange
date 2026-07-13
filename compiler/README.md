# Orange compiler

Status: production-lineage, pre-alpha S3a under accepted OEP-0003

This workspace contains the first executable slice of the Orange compiler. It
is intentionally small, but its source identities, byte spans, language-edition
boundary, diagnostic codes, and deterministic token stream are permanent
interfaces to extend rather than a disposable prototype.

Nothing here makes a verification, correctness, constant-time, or production
readiness claim. `orangec check` performs lexical, syntactic, and bounded
semantic validation. The accepted S3a slice assigns meaning only to closed
typed `spec` literals, lowers them to a noncanonical Typed Reference Core, and
reference-evaluates them. General expressions, typed `impl`, proof checking,
verified lowering, and code generation do not exist.

This boundary was merged by
[PR #9](https://github.com/chasebryan/orange/pull/9) as commit
`6c0bd3021cf2df603e08808e4660724ca1e2b2a5`. Orange remains pre-alpha; that
merge creates no stable public compatibility promise.
Later S3 semantics remain incomplete, and D-003 and D-004 remain unresolved.

## Run

The workspace requires the pinned Rust 1.96.1 toolchain and has no third-party
Rust dependencies. This pre-alpha slice does not declare or test a lower MSRV.

```sh
cargo test --manifest-path compiler/Cargo.toml --workspace
cargo run --manifest-path compiler/Cargo.toml -p orangec -- check compiler/fixtures/hello.or
cargo run --manifest-path compiler/Cargo.toml -p orangec -- check compiler/fixtures/typed-answer.or
cargo run --manifest-path compiler/Cargo.toml -p orangec -- eval compiler/fixtures/typed-answer.or
cargo run --manifest-path compiler/Cargo.toml -p orangec -- lex compiler/fixtures/hello.or
```

`orangec` accepts up to 256 source inputs in argument order. Regular files are
processed incrementally; `-` is the only stream input and reads standard input
at most once. `eval` accepts exactly one source and emits no partial result after
an error. Successful `check` commands are silent. Diagnostics go to standard
error and use exit status 1; command-line usage errors use status 2. A source
with lexical errors is not parsed, and a source with syntax errors is not
analyzed. File and standard-input reads stop at a deterministic 16 MiB
per-source limit. Larger inputs fail with `ORC1003` before lexing and are never
buffered without a bound.

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

## Orange 2026 grammar

The parser accepts exactly one edition declaration followed by exactly one
module. Legacy empty `spec` and `impl` functions remain valid. A `spec` may also
declare one parsed result type and one signed integer literal:

```text
source_file   = edition_decl module_decl EOF ;
edition_decl  = "edition" "2026" ";" ;
module_decl   = "module" IDENTIFIER "{" function_decl* "}" ;
function_decl = "spec" IDENTIFIER "(" ")" spec_tail
              | "impl" IDENTIFIER "(" ")" empty_body ;
spec_tail     = empty_body | "->" parsed_type "{" signed_integer "}" ;
parsed_type   = IDENTIFIER ("[" INTEGER "]")? ;
signed_integer = "-"? INTEGER ;
empty_body    = "{" "}" ;
```

For example:

```orange
edition 2026;
module demo {
  spec identity() {}
  impl rounds() {}
  spec answer() -> Int { 42 }
  spec mask() -> Word[8] { 0xff }
}
```

The parser accepts generic type syntax so unsupported forms receive semantic
diagnostics. Semantics accepts only exact `Int` and exact `Word[8]` on typed
`spec` declarations. `Int` is mathematical within the bounded accepted source
representation and does not silently wrap;
`Word[8]` accepts only 0 through 255 and does not coerce, truncate, or wrap.
Duplicate names are syntactically valid, then semantic analysis rejects a
duplicate within the same declaration-kind namespace. Empty declarations have
no value, and a typed `impl` remains a syntax error.

`orangec eval` prints every typed specification in source order:

```text
demo::answer: Int = 42
demo::mask: Word[8] = 0xff
```

The complete accepted rules and non-claims are in
[`docs/SEMANTICS_2026.md`](../docs/SEMANTICS_2026.md). This slice defines no
operators, calls, parameters, bindings, effects, proof meaning, implementation
refinement, target behavior, ABI, leakage property, output code, package or
release behavior, or cryptographic construction.

## Layout

- `crates/orange-compiler`: reusable source, span, diagnostic, edition, lexer,
  syntax-tree, parser, semantic, Core, and evaluator library;
- `crates/orangec`: thin file/stdin CLI with deterministic `check`, `eval`, and
  `lex` behavior;
- `fixtures/hello.or`: permanent legacy syntax fixture; and
- `fixtures/typed-answer.or`: permanent typed-literal evaluation fixture.
