# Orange 2026 documentation generator contract

Status: permanent pre-alpha W3 frontend tooling under owner direction;
no semantic acceptance, S8 closure, or release is recorded here

Edition: `2026`

Snapshot: 2026-10-01

The generator produces an offline reference for the declarations written in
one parsed source. This is a tool boundary in
[`ARCHITECTURE.md`](ARCHITECTURE.md) and [`ROADMAP.md`](ROADMAP.md), separate
from the proposed S3 semantics. It leaves the implemented language marker unchanged.

## Command and source boundary

```text
orangec doc FILE|-
```

Exactly one UTF-8 source is required. The complete standalone HTML document
is printed on standard output; the command changes no file. `-` reads standard
input. `--edition 2026` and `--` retain their ordinary meanings, including
paths beginning with a dash and platform paths that are not UTF-8.
There is no check mode, write mode, output-file option or evaluation option.

Lexing and parsing must succeed before documentation is generated. Existing
lexical and syntax diagnostics are retained and produce no document. The
generator loads no imported module and performs no semantic analysis or
evaluation. A missing import, unknown type or duplicate declaration name may
therefore appear in a parsed source reference. Written types are not resolved
types; finite domains are not instantiated. Legacy empty `spec` and `impl`
declarations have no inferred type or execution meaning. Test entries report
their written titles and source, without running tests or reporting a pass.

## Document structure and rendering

The document includes the module, imports, aliases, `spec`, `impl` and test
declarations in source order. Entries retain written headers and finite
parameter domains, with source line/column locations and ordinal anchors that
remain distinct when names repeat. The full source listing includes comments;
comments receive no new documentation syntax or claim authority.
Function and test headers omit their bodies; the complete source listing
includes them.

Every source-derived fragment is text. The five HTML-sensitive characters
`<`, `>`, `&`, single quote and double quote are escaped. Ordinary Unicode,
tabs and source line endings are retained in serialized text. Other control
characters (C0, C1 and DEL), plus U+061C, U+200E–U+200F,
U+2028–U+202E and U+2066–U+2069, are displayed visibly as lowercase ASCII
`\u{hex}`. Other Unicode is retained without normalization. The listing is a
complete source display, not a byte-recovery
format: HTML viewers may normalize line endings or render characters differently.

The standalone document has no scripts, styles, external assets or
source-controlled links. Its fixed content-security policy disables external
loads, base URLs and form submissions. Internal links use generated anchors.
Generated metadata contains no input filename, host path or current date;
source-authored text is still displayed. Equal source contents and
edition produce byte-identical HTML regardless of filesystem location.

Generated HTML is neither canonical Core nor checked proof/evidence. Source
locations describe the input; formatting or other edits change them. The
document does not preserve or migrate source-bound proof identities, prove a
declaration correct, or give source prose a checked claim status.

## Bounds and failures

The existing 16 MiB source limit and lexical/parser budgets apply. The
generator admits at most 1,048,576 documentation work items and
16 MiB (`16 * 1024 * 1024` HTML bytes). Escaping or page structure may push
an admitted source beyond the output limit. Rendering uses checked work and
allocation, and completes before any document bytes are written.

| Diagnostic | Meaning |
| --- | --- |
| `ORC0260` | Documentation resource or output-size limit. |
| `ORC0261` | Inconsistent source or documentation construction. |

Input, UTF-8, lexical, syntax and host-output faults retain their existing
diagnostics. Exit status is 0 for a complete generated document, 1 for a
diagnosed source, resource, consistency or transport failure, and 2 for
invalid usage. A detected host write/flush failure returns status 1; a prefix
already accepted by the host is not a successful document.

## Regression evidence

The [CLI harness](../compiler/crates/orangec/tests/documentation.rs) covers
complete deterministic output, source escaping and visible controls, ordered
duplicate-safe anchors, syntax-only behavior, omitted ambient metadata,
unchanged inputs, option/path handling and atomic resource rejection. Named
cases include `doc_emits_complete_deterministic_offline_html`,
`doc_declarations_have_unique_resolved_local_anchors_in_source_order`,
`doc_uses_parsed_syntax_without_import_loading_or_semantic_claims`, and
`doc_escaped_output_limit_rejects_atomically`. CLI unit cases additionally
check complete stdout-budget preflight and accepted-prefix host failures.

The [engine regressions](../compiler/crates/orange-compiler/src/documentation.rs)
include `declaration_order_headers_domains_and_legacy_comments_are_preserved`,
`controls_are_visible_without_normalizing_other_source_text`,
`exact_output_and_work_limits_return_no_partial_document`, and
`repository_parsed_sources_generate_complete_references`. Separate cases cover
Unicode/CRLF locations, stale spans, maximum syntax height and original frontend
errors. These are tool-contract tests, not product proof or semantic acceptance.

## Product status

This permanent, dependency-free generator supplies the current parsed-source
documentation boundary. The complete 1.0 product still requires documentation
of resolved interfaces, ABI artifacts, complete claim matrices and assumptions
from their actual accepted compiler/checker artifacts, as required by
[`ASSURANCE.md`](ASSURANCE.md) and [`USER_JOURNEYS.md`](USER_JOURNEYS.md).
Those claims are not filled from source text or tests. LSP, package/evidence
tools and complete journeys remain open in
[`RELEASE_1_0_EXECUTION.md`](RELEASE_1_0_EXECUTION.md).
