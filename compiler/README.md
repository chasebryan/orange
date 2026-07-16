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
scripts/ci/check-repository
cargo test --manifest-path compiler/Cargo.toml --workspace
cargo run --manifest-path compiler/Cargo.toml -p orangec -- check compiler/fixtures/hello.or
cargo run --manifest-path compiler/Cargo.toml -p orangec -- check compiler/fixtures/typed-answer.or
cargo run --manifest-path compiler/Cargo.toml -p orangec -- eval compiler/fixtures/typed-answer.or
cargo run --manifest-path compiler/Cargo.toml -p orangec -- lex compiler/fixtures/hello.or
cargo test --manifest-path compiler/Cargo.toml -p orangec --test s3a_conformance --locked --offline
```

The protected repository gate runs all Rust targets in both debug and optimized
release profiles. The release profile retains debug assertions and integer
overflow checks, so optimization cannot silently weaken internal invariants;
the individual commands are useful for focused development. A separate
production-only Clippy pass denies unchecked arithmetic, silent `as`
conversions, UTF-8 string slicing, indexing, unwrap/expect, and explicit panic
sites while leaving test assertions available to state fixture invariants.
The same isolated gate fixes a private file-creation mask and captures one source
archive before Cargo runs. A sanitized, NUL-delimited Git index inventory admits
exactly tracked paths, while archive bytes come from the working tree so tracked
local edits are tested; untracked and ignored local state cannot enter. The
archive format, path order, timestamps, numeric owner/group fields, and file
modes are fixed: ordinary files are `0644`, admitted executables are `0755`,
and hard links are archived as independent files. Before Cargo runs, every
tracked path in the working tree and first extraction must be a regular,
non-symlinked file, their executable classifications and bytes must match, and a
fresh Git inventory must match the original path list, rejecting observed type,
executable-mode, content, or membership edits during capture. The copied
validator opens the archive and inventory through read-only descriptors, unlinks
their filesystem names, and closes those descriptors before any copied Python
or Rust code executes. Every copied command then runs as PID 1 with a private
user, mount, PID, `/proc`, and network namespace. The private process view keeps
the trusted parent shell and its descriptors outside copied-code visibility,
while the empty network namespace prevents new external connections. The
namespace supervisor kills its child if supervision is interrupted. Trusted gate
operations alone retain the descriptors used for later extraction and identity
checks. The copied validator first policy-checks that exact exported tree before
its foundation test modules import.
After those tests, it policy-checks the tree again before Cargo, so Python-test
drift cannot reach Rust execution. Formatting, linting, documentation, and Rust
tests use the same extracted check root. A third policy check runs after all Rust
commands. The gate then verifies that the original archive and path inventory
retained their captured identities, extracts a fresh reference, compares the
NUL-safe sorted non-directory membership of all three compiler input roots, and
compares every tracked file's type, complete mode, and bytes with the reference.
These exact comparisons reject added source entries and policy-valid
tracked-source drift before the gate can pass. Optimized
`orangec` builds use independently created temporary ancestors, relocated
source roots, separate Cargo homes, and separate target trees whose names differ
in bytes, length, and directory depth. Both artifacts must be regular
non-symlink files with identical complete modes and bytes. This is
source-relocated same-host reproducibility evidence, not a cross-platform or
independently rebuilt claim.

`orangec` accepts up to 256 source inputs in argument order. Argument parsing
inspects at most 4 MiB (`4 * 1024 * 1024` bytes) of encoded command-line
arguments per invocation, charged before each argument is interpreted.
Exceeding the byte allowance is a usage error before any source read. Regular
files are processed incrementally; `-` is the only stream input and reads
standard input at most once. Integration coverage requires exactly 256 valid
inputs to succeed silently and 257 operands to fail as a usage error before any
source read. It
also interleaves file, standard-input, and file failures in exact operand order;
a repeated `-` emits exactly one `ORC1004` group and still processes a later
operand. The global `--edition` option may appear before or after the command
but at most once; a repeated split or inline form is a usage error before any
source read. `--` ends option parsing so dash-prefixed source paths remain
addressable.
The portable regular-file boundary checks path-entry metadata without following
a final symlink before opening and after reading, rejecting an observed symlink
as non-regular. It checks descriptor metadata after opening and again after
reading, and requires the final path entry to remain a regular file. Linux
x86-64 and AArch64 opens also request `O_NOFOLLOW | O_NONBLOCK`, so a final
symlink swap fails at the descriptor open and a swapped FIFO cannot wait for a
peer. On Unix, the opened descriptor's device, inode, mode, owner, group, link
count, length, modification time, and change time must match both path
snapshots and remain stable through the read. The completed byte snapshot must
also have exactly the descriptor's reported length. Before the final metadata
comparison, `orangec` seeks the same opened descriptor to offset zero and
requires a second bounded read to match every retained byte plus exact EOF. The
verification read allocates no second source snapshot and does not charge the
invocation's buffered-source allowance twice. Other hosts compare length and
modification time at each boundary. This remains short of race-free path
confinement: portable-host opens can still block on a swapped special file,
parent components are not confined, a path can change away and back between
snapshots, and coordinated mutation or unusual filesystem semantics that
reproduce the same bytes and metadata across both reads can evade the
comparison. Compile untrusted filesystem trees from a stable copied file or
standard input inside an appropriate host sandbox; full path confinement is not
claimed.
`eval` accepts exactly one source and begins output only after complete
validation and evaluation. A host output failure can leave an
already-written prefix, but returns status 1; a broken pipe remains quiet and
is never reported as successful evaluation. Once a standard-output or
standard-error write failure is observed, later source operands are not read or
compiled, and a partially accepted diagnostic prefix is not retried. Ordinary
source failures still aggregate diagnostics across later inputs. Successful
`check` commands are silent. Diagnostics go to standard
error and use exit status 1; distinct compiler or host error groups have exactly
one blank separator with no leading or extra trailing blank group.
Every parser, semantic-analysis, and evaluation result is classified
fail-closed: diagnostics take precedence, an artifact is accepted only without
diagnostics, and an absent artifact without diagnostics emits `ORC1006` as an
internal compiler or resource failure.
Command-line usage errors use status 2 when their diagnostic is written; a
detected usage-output failure uses status 1 without reading source input. A
usage diagnostic has one blank separator before the exact help text and one
trailing newline, while help and version output failures follow the same status
1 transport rule. All three paths flush explicitly and treat a detected flush
failure as status 1. Transient `Interrupted` results from source reads,
verification seeks, output writes, and explicit output flushes are retried
without duplicating accepted bytes. They fail closed
after 1,024 consecutive attempts for one operation.
Source reads that reach this boundary retain `ORC1001` and identify the exact
retry limit instead of attributing the local limit to the operating system.
Every output adapter rejects an impossible write count larger than the
offered byte slice. Compilation diagnostics are also explicitly flushed after
their final error group. When diagnostics and buffered token output are both
pending, the diagnostic stream is flushed first so a detected diagnostic-flush
failure discards token bytes that have not escaped the process. `orangec` caps standard error at 64 MiB (`64 * 1024 * 1024` bytes)
per invocation. Reaching the cap returns status 1 and stops before later source
operands; because the diagnostic channel itself is exhausted, an already
accepted prefix can end without a final limit notice. After any detected stream
failure, retained buffered standard output is discarded instead of being
flushed as later command output.
Compilation standard output is explicitly flushed only after successful token
or evaluation bytes have been queued; untouched output and diagnostic streams
are not flushed for a silent `check` or empty `eval`. A source with lexical
errors is not parsed, and a source with syntax errors is not analyzed. File and
standard-input reads stop at a deterministic 16 MiB per-source limit. Larger
individual inputs fail with `ORC1003` before lexing. `orangec` buffers at most
64 MiB (`64 * 1024 * 1024` bytes) across all source operands per invocation;
the first operand that would exceed the remaining total budget fails with
`ORC1008`. Bytes consume that shared budget as soon as they are read into the
bounded input buffer, even when the operand is later rejected. The one-byte
probe used to diagnose per-source overflow is also charged whenever aggregate
budget remains, so a rejected oversized operand cannot donate that byte to a
later operand. Once no aggregate budget remains, a reader may consume one
unbuffered probe byte only to distinguish end of input from overflow. Source
bytes are never buffered without a bound. CLI-derived rendered source names
reserve their complete escaped representation before encoding.
Source-map slots, borrowed
source-name and source-text copies, and derived line/column indexes also use
checked reservations; an allocation failure rejects the source through
`ORC1005` without exposing partial source state or consuming an insertion ID.
Already owned `String` inputs move into the map without an additional
source-data copy.
Lexing uses bounded amortized fallible growth while preserving one allocated EOF
slot and never requesting speculative capacity beyond the complete token-stream
limit. It fallibly reserves the complete 102-record diagnostic-vector bound
before scanning. Failure to reserve that vector exposes only the allocation-free
EOF fallback and is classified by the CLI as a fail-closed `ORC1006` internal
resource failure. A token-storage reservation failure emits `ORC0008`, discards all
ordinary tokens, and cannot expose a parser-acceptable partial stream; failure
to reserve even the initial heap slot uses an allocation-free inline EOF
fallback, so the public token stream still contains exactly one final EOF. An
impossible internal UTF-8 cursor mismatch follows the same atomic rejection
boundary: it emits `ORC0008`, discards partial tokens, and exposes only EOF.
Parsing reserves every owned identifier copy and each module-function slot
before installing them, and fallibly pre-reserves its complete 102-record
diagnostic-vector bound. Identifier or declaration reservation failure emits
`ORC0106`; diagnostic-vector reservation failure returns no AST or diagnostic
and is classified by the CLI as `ORC1006`.
Semantic analysis reserves and deterministically sorts the complete declaration
namespace index, checks
exact-integer limb growth, owned Core-name copies, and each pending
typed-function slot, then reserves the complete Core function table before
installing its first entry. It also fallibly pre-reserves its complete
102-record diagnostic-vector bound. Ordinary representation failures emit
`ORC0209`; diagnostic-vector reservation failure returns no Core or diagnostic
and is classified by the CLI as `ORC1006`. Identifier spellings echoed by
semantic diagnostics are capped at 64 bytes plus a deterministic total-length
suffix.
Lexical, parser, and semantic reporting admit an ordinary diagnostic before
constructing its owned message, label, note, or secondary-span fields.
Post-limit attempts create at most the one suppression record and construct no
discarded ordinary diagnostic.
Reference evaluation reserves the complete value-set vector before evaluating
the first function and checks every copied function name and exact-integer limb
vector. It also fallibly reserves its single possible diagnostic slot before
evaluating. Ordinary reservation failures emit `ORC0301` and expose no partial
value set; diagnostic-slot reservation failure returns no values or diagnostic
and is classified by the CLI as `ORC1006`. The shared module-name `Arc` control
block still uses the standard infallible allocator API because stable Rust does
not provide a fallible `Arc` constructor.
These checked container reservations do not make the entire diagnostic path
out-of-memory recoverable. The infallible `RenderedSourceName::from_text` and
`RenderedSourceName::from_os_str` convenience constructors, owned diagnostic
messages/labels/notes, owned CLI usage and error strings, rendered diagnostic
output, and the shared `Arc` control block still use standard infallible Rust
allocation APIs. The CLI uses the fallible rendered-name constructors. The
input and output bounds limit amplification, but process-level allocator
exhaustion can still abort instead of producing an Orange diagnostic.
Within that residual boundary, diagnostic messages, labels, notes, and source
names are escaped directly into the final rendered output instead of first
materializing expanded copies. Excerpt escaping writes into only the bounded
40-before/80-after window described below.
Exact-integer decimal display uses fixed stack arrays sized from the normative
16,384-bit limit, then writes base-1,000,000,000 limbs directly to the
destination without heap scratch or a materialized decimal output string.

Diagnostic source excerpts include at most 40 Unicode scalars before and 80
from the responsible position. Unit coverage renders the full 100-diagnostic
frontend budget beside a 1 MiB line and requires the complete deterministic
output to remain below 64 KiB, preventing line length from multiplying output
memory per diagnostic.

CLI-derived source names, diagnostic excerpts, and echoed command or option
text use an ASCII-safe escape representation for backslashes, controls, and
non-ASCII scalars. Invalid encoded path bytes use `\xNN` identities rather than
lossy replacement, and literal escape-looking path input remains
distinguishable from the byte it resembles. Library-provided raw source names
are escaped injectively during diagnostic rendering; the CLI passes a tagged
canonical name so its already encoded OS path is not escaped a second time.

`orangec lex` streams token records and escapes each spelling through a fixed
4 KiB scratch buffer instead of materializing an expanded token string. Lex
output for multiple sources uses an exact `== SOURCE ==` header and one blank
separator in argument order. A lexical error returns status 1 but does not
suppress that source's bounded token stream or later sources. Lex output still
can be larger than the accepted source because of escape notation and per-token
metadata. `orangec` caps standard output at 64 MiB (`64 * 1024 * 1024` bytes)
per invocation. Reaching that limit returns status 1, emits `ORC1007`, stops
before reading later sources, and can leave an already-accepted output prefix;
callers processing untrusted input should also cap time.

Accepted S3a assigns no separate semantic budget to evaluation-output bytes.
Each successful output line repeats the module name, so a source with a long
module name and many typed specifications can request output much larger than
its input. The CLI shares the evaluated module identity and streams values
through a fixed buffer, while the operational 64 MiB standard-output ceiling
above bounds the bytes actually accepted per invocation. Exceeding it is an
unsuccessful output operation rather than an accepted partial evaluation.
Apply caller-side time limits before using `orangec eval` on untrusted sources.

## Frozen lexical boundary

The only supported language edition is Orange 2026. Its current lexical rules
are deliberately conservative:

- each source is at most 16 MiB of UTF-8, and spans are half-open UTF-8 byte
  ranges;
- each source map receives a unique nonzero process-local identity; exhausting
  the 64-bit identity space is a sticky failure and can never wrap into an
  earlier map's span ownership; the CLI uses the fallible constructor and
  reports exhaustion as a source-representation error;
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

The reusable syntax-tree and Typed Reference Core nodes, together with parser,
analysis, and evaluation result envelopes, are read-only outside the compiler
crate. Callers can inspect parsed, checked, and evaluated spans, names,
source-ordered declarations, identities, types, and values through accessors,
but cannot mutate compiler-established structure or replace a checked or
evaluated value.
Parsing rejects lexer output paired with a different source as `ORC0107`, even
when that lexer output already contains errors. Semantic analysis rejects a
syntax tree paired with a different source as `ORC0210`. A Core function's
reported type is derived from its value, so a type/value mismatch is not
representable at the public Core boundary.

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

## S3a CLI conformance corpus

`fixtures/s3a/` contains an exact ten-file black-box corpus for already accepted
S3a behavior. Three fixtures must evaluate successfully and seven must fail
closed. The corpus covers:

- empty declarations, mixed empty and typed declarations, and cross-kind equal
  names;
- positive, zero, negative, and negative-zero `Int` observations in decimal,
  binary, and hexadecimal source forms;
- exact `Word[8]` observations at 0, 1, 254, and 255;
- typed-`impl` syntax rejection; and
- the stable duplicate, unsupported-type, word-width, integer-magnitude,
  negative-word, and word-range diagnostic categories.

`crates/orangec/tests/s3a_conformance.rs` checks the directory inventory rather
than accepting an extra fixture implicitly. It invokes both `orangec check` and
`orangec eval` twice per fixture and requires identical status, standard output,
and standard error. Accepted cases have silent checking, exact evaluation
output, and no diagnostic. Rejected cases have status 1, no partial output, the
exact ordered diagnostic-code sequence, the expected diagnostic meaning, and
the exact primary line and column for every diagnostic. Check and evaluation
rejection bytes must agree.

Six generated black-box cases exercise boundaries and combinations that are
impractical as ordinary expected-output fixtures. They cover the exact
significant-bit boundary, leading-zero neutrality, the exact semantic diagnostic
budget with additional post-suppression attempts, mixed-category diagnostic
source ordering, case-sensitive names and types, and every later same-kind
duplicate. Every generated command runs twice. The rejected boundary and
diagnostic-budget sources run through both commands; the accepted 16,384-bit
boundary uses silent checking only so the test does not intentionally capture
an enormous decimal evaluation result.

CLI integration coverage places lexical, parser, and semantic failures before,
after, and between otherwise valid typed declarations. `eval` remains
repeatably unsuccessful with zero value bytes in every such ordering, and an
earlier phase diagnostic prevents later-phase cascades. Multi-file `check`
coverage interleaves semantic, valid, lexical, and parser inputs and requires
repeatable diagnostics in argument order rather than phase or code order.

Semantic unit coverage also aggregates independent duplicate, unsupported-type,
word-width, negative-word, and word-range failures in one source. It requires
the raw and rendered diagnostic sequences to remain in source order, checks the
exact responsible source slice for every primary span, and checks that a
duplicate's secondary span names the first declaration. A duplicate typed
declaration is still type-checked in semantic traversal order; focused limit
tests pin the exact event at which its second failure becomes diagnostic
suppression or resource exhaustion.

A deterministic unit mutation corpus deletes, replaces, and inserts characters
at every boundary of an accepted mixed S3a module, then adds bounded sequences
of grammar fragments, comments, line endings, malformed characters, and
Unicode. The resulting set contains more than 2,500 unique sources. Every
mutant runs twice through the same gated lexical, parser, semantic, and
evaluation pipeline; phase results must be structurally equal, rendered
diagnostic bytes must match, success must remain atomic, and every primary and
secondary diagnostic span must belong to the mutated source. The corpus must
reach lexical, parser, and semantic rejection as well as successful evaluation.

Two frontend byte corpora exercise boundaries outside valid source grammar. On
Unix, 512 generated raw argument strings are classified twice in command,
option, operand, and post-`--` positions; every error is ASCII and contains no
control byte. A platform-independent corpus sends 518 fixed and generated raw
source byte strings—including every possible one-byte input—through `check`,
`eval`, and `lex` twice each. Status and output bytes must repeat exactly,
invalid UTF-8 must reach `ORC1002`, diagnostics may contain only ASCII plus line
feeds, and token output may additionally use its canonical tab separators.

The ten-file external corpus alone is not the full S3a evidence set. The
conformance runner parses the stable 30-rule index in
`docs/SEMANTICS_2026.md`, rejects missing or unknown rule IDs, and binds every
rule to named external or internal tests. It also binds the exact evidence-layer
declaration and requires the corresponding CLI, generated-CLI, parser-unit, or
unit observation for every rule. Specialized labels additionally require a
named test classified as an injected writer or injected limit; host-failure
coverage separately requires I/O, allocation, and non-regular host-boundary
failures. Each named test must have exactly one unconditional declaration at the
expected harness location: integration tests at file root and unit tests
directly inside the source's unique `#[cfg(test)] mod tests` container.
Declarations inside comments, strings, nested functions, or alternate or
disabled modules do not qualify. This is an exact evidence map, not a claim that
a named test exhausts its rule. Policy validation binds the production
constants to the specification, while injected-limit unit tests exercise exact
semantic-event, Core-node, and evaluation accounting and fail-closed behavior
at reachable boundaries. This indexed mapping does not complete S3 and adds no source
construct, semantic rule, canonical Core identity, proof, target, claim, or S3b
authority.

## Layout

- `crates/orange-compiler`: reusable source, span, diagnostic, edition, lexer,
  syntax-tree, parser, semantic, Core, and evaluator library;
- `crates/orangec`: thin file/stdin CLI with deterministic `check`, `eval`, and
  `lex` behavior;
- `crates/orangec/tests/s3a_conformance.rs`: exact repeatable black-box S3a
  corpus runner;
- `fixtures/hello.or`: permanent legacy syntax fixture; and
- `fixtures/typed-answer.or`: permanent typed-literal evaluation fixture; and
- `fixtures/s3a/`: exact three-positive/seven-negative S3a CLI fixture corpus.
