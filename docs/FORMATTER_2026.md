# Orange 2026 formatter contract

Status: permanent pre-alpha W3 frontend tooling under owner direction;
no semantic acceptance, S8 closure, or release is recorded here

Edition: `2026`

Snapshot: 2026-10-01

The formatter provides deterministic source layout for the currently parsed
Orange 2026 syntax. It is a tool boundary in
[`ARCHITECTURE.md`](ARCHITECTURE.md) and [`ROADMAP.md`](ROADMAP.md), separate
from the proposed S3 semantics. It leaves the implemented language marker unchanged.

## Command boundary

```text
orangec fmt FILE|-
orangec fmt --check FILE...
```

Without `--check`, exactly one source is required and its complete formatted
text is printed on standard output. With `--check`, one through 256 sources
are checked in argument order and no formatted text is printed. Neither mode
changes any file. Standard input, spelled `-`, may appear at most once.

`--check` may precede or follow `fmt`, is allowed only for that command, and
may appear only once. The global `--edition 2026` option remains available;
`--` ends option parsing and permits paths beginning with a dash. Platform
paths need not themselves be UTF-8. Source contents must be UTF-8.
There is no write mode or output-file option.

Formatting requires successful lexing and parsing. It performs no semantic
analysis, import loading or evaluation: a missing imported module, unknown
name or invalid type does not prevent formatting otherwise valid syntax.
Lexical and syntax errors retain their existing diagnostics and produce no
formatted source.

## Preservation and validation

The formatter uses the parsed syntax to select whitespace for the gaps between
tokens. Every token's exact source spelling is retained, including integer
bases, digit separators, byte strings and hex-string spacing. Each complete
line or nested block comment retains its bytes and order. Its anchor between
the same neighboring tokens, or at the beginning or end of the source, is
retained; whitespace around it may change.

Layout uses two spaces per real block level and one final LF. Module members
are separated by a blank line, while the edition and module declarations use
one newline. Binary operators receive spaces; prefixes and array-type carets
remain tight. Calls, indices, projections, qualification and finite type
domains remain compact. Lists and expressions are not wrapped to a line width.
Existing parentheses are retained. A trailing `//` comment keeps its same-line
affinity; standalone comments remain separate. Internal block-comment layout
is never reindented.

The formatted source is re-lexed and re-parsed before it is returned. Token
categories and spellings and preserved comments must agree with the input.
A formatting inconsistency fails closed instead of returning suspect text.
Formatting a successfully formatted result must return byte-identical text:
the format is idempotent. Before success, the formatter also re-renders the
validated result with the same bounded role plan and checks that equality.
This uses a linear renderer pass rather than recursive formatting. Determinism does not depend on the host filesystem,
imports, locale, terminal width or a semantic type.

The preserved token/comment identities and successful reparse retain parsed
behavior. This source-text boundary is not a canonical Core encoding or a
semantic-preservation proof. Formatting changes source bytes, byte spans and
file digests; it neither preserves nor migrates source-bound proof or evidence
identities and introduces no product proof claim. Generated layout whitespace
uses LF; line endings inside retained tokens and comments keep their bytes.

## Bounds and failures

The existing 16 MiB source limit, token and parser budgets apply. Formatted
output is limited to 16 MiB (`16 * 1024 * 1024` output bytes). The iterative
syntax planner admits at most 1,048,576 formatting work items. Comments are
scanned within their source gaps under the byte limit rather than a separate
comment-item allowance. A source within its input limit can fail formatting
when the selected layout would exceed that output limit. The complete result
is validated before stdout writing begins.

| Diagnostic | Meaning |
| --- | --- |
| `ORC0250` | Formatter resource or output-size limit. |
| `ORC0251` | The formatter could not establish a consistent result. |
| `ORC0252` | `--check` found a source different from its formatted result. |

`ORC0252` identifies the first differing character boundary in the original
source, including its end when only a final newline is missing. Existing
input, UTF-8, syntax and host-output diagnostics retain their meanings.

Exit status is 0 when one source is formatted successfully or every checked
source is canonical; 1 for a formatting difference or diagnosed input,
syntax, resource, validation or output failure; and 2 for invalid usage.
Ordinary source failures allow later check operands to be checked. A detected
output-stream failure stops further work and returns status 1. A host may
already have accepted a prefix before its write or flush fails; that prefix
is not reported as a successful result.

## Regression evidence

The permanent CLI harness is
[`formatting.rs`](../compiler/crates/orangec/tests/formatting.rs). Its cases
cover exact output and repeatability, token/comment identities and anchors,
check-mode ordering and unchanged files, EOF and UTF-8 difference spans,
syntax/input failures, syntax-only scope, option and path handling, and source
bounds. The relevant named checks include
`fmt_emits_exact_canonical_source_and_is_idempotent`,
`fmt_preserves_token_spellings_comment_bytes_and_gap_anchors`,
`fmt_check_reports_all_sources_in_order_without_mutating_them`, and
`fmt_does_not_load_imports_or_analyze_semantics`.

CLI unit tests additionally check full-output preflight and that check mode
does not consume the stdout budget. These tests concern the formatter tool
contract; they do not supply product proof or semantic acceptance evidence.

The [engine regressions](../compiler/crates/orange-compiler/src/formatter.rs)
include `comments_at_every_token_boundary_are_idempotent`,
`preservation_checker_rejects_token_and_comment_changes`,
`exact_output_and_event_limits_fail_closed`, and
`repository_parsed_sources_are_complete_and_idempotent`. They also exercise
every current expression variant, streamed comment-heavy input, the longest
array and maximum syntax height, and unchanged observed reference values and
evaluation costs. The repository corpus case covers parseable sources; syntax
and resource rejection have separate negative cases.

## Product status

The formatter is permanent frontend tooling, with no third-party dependency.
It supplies one part of the developer-tool path in
[`RELEASE_1_0_EXECUTION.md`](RELEASE_1_0_EXECUTION.md). Language-server,
documentation-generator, package/evidence and complete journey obligations
remain open. Repository tests and merge do not accept semantic OEPs, close
foundational decisions or authorize publication.
