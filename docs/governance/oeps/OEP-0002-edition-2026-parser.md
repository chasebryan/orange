---
number: OEP-0002
title: Edition 2026 minimal grammar and bounded parser
authors:
  - Chase Bryan
champion: Chase Bryan
status: Provisional
type: Standards
created: 2026-07-12
updated: 2026-07-12
discussion: owner-direction-2026-07-12-s2
related-decisions:
  - D-002
  - D-003
  - D-023
  - D-024
  - D-025
related-adrs: []
requires:
  - OEP-0001
supersedes: []
superseded-by: null
review-authorities:
  - Orange Project Owner
decision-date: null
decision-revision: null
approval-records:
  - Project-owner direction recorded on 2026-07-12; solo-reviewed and provisional
---

# OEP-0002: Edition 2026 minimal grammar and bounded parser

## Abstract

Orange 2026 gains one normative, deliberately minimal source grammar and a
bounded deterministic parser. A source contains the exact declaration
`edition 2026;`, one named module, and zero or more empty `spec` or `impl`
functions. The coupled lexical specification fixes UTF-8, whitespace, line
ending, identifier, comment, literal, punctuation, keyword, and resource rules.

The owner direction recorded here authorizes implementation immediately under
OEP-0001. This proposal remains Provisional until the exact merged S2 revision
can be recorded and reviewed. Provisional status is not release authority.

## Motivation

The S1 compiler foundation intentionally stopped at tokens. S2 needs a
normative boundary before a parser can distinguish intentional syntax from an
implementation accident. A very small complete grammar supplies permanent
editioning, tree, diagnostic, and resource-limit interfaces without silently
choosing types, evaluation, proofs, targets, or code generation.

The slice also tests Orange's incremental-capability process: one exact syntax
surface can advance while decisions that have no bearing on parsing remain
open.

## Scope and non-goals

This proposal defines Orange 2026 source representation and lexing, the exact
S2 grammar, syntax-tree mapping, deterministic parser behavior, resource
budgets, diagnostics, and conformance boundaries.

It does not define imports, multiple or nested modules, parameters, return
types, expressions, statements, non-empty bodies, attributes, contracts,
types, name resolution, evaluation, Core semantics, proof rules, targets, ABI,
leakage, lowering, code generation, packaging, linking, or releases. `game`,
`proof`, and `claim` remain reserved words with no grammar or semantics.

## Specification

[`docs/LANGUAGE_2026.md`](../../LANGUAGE_2026.md) is the normative lexical and
grammar specification for this proposal. The complete parser grammar is:

```text
source_file   = edition_decl module_decl EOF ;
edition_decl  = "edition" "2026" ";" ;
module_decl   = "module" IDENTIFIER "{" function_decl* "}" ;
function_decl = function_kind IDENTIFIER "(" ")" empty_body ;
function_kind = "spec" | "impl" ;
empty_body    = "{" "}" ;
```

Sources are valid UTF-8 of at most 16 MiB. Whitespace is exactly U+0009,
U+000A, U+000D, or U+0020. Line feed, carriage-return line feed, and bare
carriage return each form one logical line ending. Identifiers are ASCII only.
The exact decimal token `2026` is mandatory in the edition declaration.

The parser is deterministic LL(1). It preserves source order and exact token
spans. Duplicate function names are syntactically valid. A recovered tree with
diagnostics is never an accepted program.

The resource budgets are 262,144 non-trivia lexical tokens, 262,144 syntax
nodes, 1,048,576 parser events or equivalent elements, 100 ordinary parser
diagnostics plus one suppression marker, and recovery delimiter depth 64.
Exhaustion fails closed.

## Alternatives

A lexer-only S2 was rejected because it would provide no grammar boundary. A
larger Rust-like grammar was rejected because parameters, types, expressions,
and bodies would force semantic and migration choices that S2 cannot justify.
Parsing the long-range five-stratum design was rejected for the same reason.

An external parser generator was rejected for this slice because the grammar is
LL(1), a small standard-library-only implementation is reviewable, and D-024's
zero-third-party-Rust-dependency boundary remains useful. A trivia-preserving
green tree may be reconsidered with formatter or IDE requirements; it is not
needed to make this exact grammar deterministic and source mapped.

## Compatibility and migration

S1 did not accept programs syntactically, so this proposal does not invalidate a
previous accepted grammar. It does change `orangec check` from lexical-only
success to lexical-and-syntactic success and reserves `edition` as a keyword.
Pre-alpha source that omitted the declaration or used reserved tokens as names
must migrate to the exact grammar.

The rollback is one repository revert of the S2 implementation and normative
documents. That would restore the S1 lexer boundary and must not be described as
support for another grammar. A later syntax extension requires an edition-aware
decision and explicit migration analysis.

No binary, schema, package, evidence, target, or ABI compatibility surface is
created.

## Semantic and claim effects

The only new language effect is syntactic acceptance under the Orange 2026
edition. `spec` and `impl` distinguish syntax-node kinds but have no execution,
typing, proof, or refinement meaning. Empty bodies contain no implied value or
operation. Duplicate names have no binding consequence because binding does not
exist.

The supported claim is limited to deterministic, bounded recognition of the
documented grammar at a recorded revision. Parser success is not a correctness,
soundness, cryptographic, constant-time, compatibility, or production claim.

## TCB, axiom, and proof effects

The Rust parser, lexer, source model, diagnostics, standard library, toolchain,
host, and sole owner become engineering trust dependencies for the new syntax
result. They do not enter a logical proof TCB because no proof judgment exists.

This proposal adds no axiom, theorem, proof rule, certificate, proof format, or
checker. Parser tests and repeated execution are implementation evidence, not a
proof of parser correctness or organizationally independent checking.

## Threat, abuse, and leakage effects

The parser expands TB-008 and CTL-020 in the threat model. Hostile inputs can
target recovery loops, nesting, allocation, diagnostic floods, ambiguous
acceptance, Unicode confusion, and lexer/parser disagreement. Fixed source,
token, node, event, diagnostic, and recovery-depth limits; progress-guaranteed
recovery; exact ASCII rules; negative cases; and repeatability tests constrain
those paths.

Residual risks include allocation failure within a budget, Rust or host defects,
untested platform behavior, and shared mistakes in the specification and
solo-authored implementation. The parser establishes no leakage model and no
constant-time property.

## Target and ABI effects

The parser is host-side Rust code and emits no Orange target artifact. This
proposal selects no CPU, operating-system support promise, object format,
calling convention, layout, foreign interface, or ABI. Observed host test
success remains pre-alpha evidence only.

## Standards, errata, and provenance

No external cryptographic or language standard is incorporated normatively.
The grammar and lexical rules are owner-directed Orange project material.
Calendar year `2026` is an edition identifier, not a claim of standards
publication or stability.

## Dependencies, licenses, and IP

The implementation retains the S1 Rust standard-library-only dependency graph
and introduces no crate, parser generator, build script, network fetch, or
generated source. The pinned Rust toolchain remains a build dependency.

The repository-wide outbound license remains unresolved under D-018. This
proposal grants no third-party permission and makes no final-name or trademark
claim.

## Conformance, tests, and evidence

Conformance includes accepted empty and multi-function modules; exact
syntax-tree mapping; all three logical line-ending forms; malformed cases at
each grammar boundary; lexical-only reserved words in declaration and name
positions; duplicate-name acceptance; Unicode whitespace and identifier
rejection; trailing-token rejection; all resource budgets; progress under
recovery; and deterministic repeated parsing.

Repository evidence must pass formatting, linting, Rust unit and CLI tests,
offline locked dependency checks, foundation policy tests, exact inventory, and
required hosted checks. The S2 merge revision will be recorded only after that
evidence exists.

## Operations, release, and recovery

No service, deployment, registry, key, package, or release operation is added.
`orangec check` becomes the local entry point for lexical and syntax validation;
lexically invalid sources are not parsed. Generated build artifacts remain
untracked.

A parser defect is recovered by reverting the S2 change with history preserved,
narrowing the accepted grammar if necessary, and adding a regression source.
No product release is authorized.

## Support and deprecation

Orange 2026 syntax is pre-alpha and best effort. There is no stability, SLA,
LTS, migration-service, or production-support promise. Permanent-lineage
implementation means the boundaries are intended to evolve in place; it does
not freeze this grammar.

A future change must document whether it extends Orange 2026 or introduces a
new edition and must provide an explicit source migration boundary.

## Unresolved questions

Tree trivia retention, formatter needs, later module structure, names, types,
expressions, semantics, proofs, targets, ABI, leakage, code generation, package
shape, and release behavior remain unresolved. None is answered by accepting
this grammar.

OEP-0002 also remains mechanically unaccepted until the exact merged S2
revision and its hosted evidence exist.

## Decision record

On 2026-07-12 the project owner directed the exact grammar, lexical choices,
resource budgets, non-goals, and bounded implementation recorded here. Under
OEP-0001 and D-023, that owner direction is immediately effective and permits
the S2 implementation to proceed.

The record remains Provisional with null `decision-date` and
`decision-revision`. It must not be marked Accepted until the implementation,
normative document, tests, policy inventory, and required hosted checks are
present at an exact merged revision. The work is solo-reviewed; no independent
approval, audit, proof, or external validation is claimed.
