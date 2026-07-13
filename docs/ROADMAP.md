# Dependency-ordered solo roadmap

Status: directed active roadmap under D-023 and OEP-0001

Snapshot: 2026-07-13

Orange is developed by one owner. This roadmap assumes no contributors,
independent reviewers, auditors, laboratories, partner organizations, or
separate operational roles. It contains no staffing-based calendar promise.
Outside participation may be incorporated only after it becomes real and the
owner explicitly changes the operating model.

## 1. How to read this roadmap

Orange still follows the no-disposable-prototype rule. Every merged compiler
component belongs to the intended production lineage and therefore needs stable
boundaries, deterministic behavior, diagnostics, tests, documentation, and a
clear migration story.

The former aggregate Gate 0 blocked all implementation until the entire 1.0
institutional plan was staffed and externally reviewed. D-023 supersedes that
barrier at its honest 0/7 state. Work now advances through incremental
capability gates:

- an unresolved decision blocks only the component or claim that depends on it;
- a test result does not become proof;
- owner review is never described as independent review;
- unavailable external evidence limits claims rather than unrelated work; and
- a partial system is always labeled pre-alpha and never marketed as complete.

## 2. Product direction

The long-term goal remains a language and toolchain for specifying,
implementing, and verifying cryptography. The intended system includes:

- an editioned Orange language and normative semantics;
- a deterministic frontend, reference evaluator, compiler, and developer tools;
- explicit, artifact-scoped claim records rather than a generic `verified`
  label;
- canonical proof and evidence formats with offline replay;
- a small published trusted computing base for each claim;
- native artifacts and a stable foreign-function boundary; and
- standards-sourced cryptography packages with precise provenance.

These are product directions, not descriptions of current features. The project
does not promise a date, LTS window, certification, external audit, independent
rebuild, or multi-person governance.

## 3. Solo workstreams

One owner performs the work, but the boundaries remain distinct:

| ID | Workstream | Permanent responsibility |
| --- | --- | --- |
| W0 | Product and decisions | Scope, decisions, naming, licensing, support, claim wording |
| W1 | Language and semantics | Grammar, types, effects, memory, erasure, leakage semantics |
| W2 | Proof and metatheory | Proof IR, checker, certificates, metatheory, trust reporting |
| W3 | Frontend and tools | Source model, lexer, parser, diagnostics, formatter, evaluator, LSP |
| W4 | Compiler and targets | IRs, lowering, validation, object paths, ABI, bootstrap |
| W5 | Cryptography corpus | Standards provenance, specifications, implementations, tests, proofs |
| W6 | Package and release | Manifests, locks, evidence bundles, builds, provenance, updates |
| W7 | Assurance and conformance | Threat model, adversarial tests, fuzzing, claim validation |
| W8 | Documentation and adoption | References, tutorials, examples, migrations, usability notes |

Separating workstreams prevents one successful test from leaking assurance into
another boundary. It does not imply separate people or independent review.

## 4. Dependency rules

```text
source files and diagnostics
           |
           v
editioned grammar and parser
           |
           v
name resolution and typed semantic core
       |                 |
       v                 v
reference evaluator   canonical Core
                         |
              +----------+----------+
              |                     |
              v                     v
         proof boundary         compiler IRs
              |                     |
              +----------+----------+
                         |
                         v
               target artifacts and claims
                         |
                         v
             cryptography corpus and releases
```

Critical ordering rules:

- Syntax may evolve during pre-alpha, but every accepted construct needs a
  documented grammar and deterministic parse.
- Type checking and evaluation require explicit arithmetic, failure, and name
  resolution semantics.
- Proof-bearing work requires D-006 and the canonical Core boundary to be
  selected; proof-neutral frontend work does not.
- Constant-time or leakage claims require D-012 and a target model; ordinary
  lexing and parsing do not.
- ABI or native-object claims require memory, layout, target, and foreign-boundary
  decisions.
- Cryptographic claims require exact standards and errata provenance, vectors,
  negative cases, and complete assumptions.
- Release claims never inherit from development checks.

## 5. Capability stages

### S0 — Repository foundation

Status: complete enough to support implementation

The repository has governance records, threat and assurance models, pinned CI,
dependency controls, provisional evidence schemas, adversarial policy fixtures,
and deterministic repository validation. The legacy Gate 0 institutional exit
criteria were not completed; D-023 retired them as an aggregate implementation
barrier.

### S1 — Compiler foundation

Status: complete at merged revision
`469bdec6037f20c8d099d61a09a3d19a55c88231`

Scope:

- pinned Rust edition and toolchain;
- no third-party Rust crates;
- source identity and UTF-8 byte spans;
- deterministic lexer with comments, identifiers, literals, punctuation, and
  reserved words;
- structured diagnostics with stable codes and source locations;
- `orangec` command-line input, output, and exit-code contract; and
- formatting, lint, unit, integration, malformed-input, and repeatability tests.

Exit test: the exact source inventory passes offline locked Rust checks and the
repository policy suite; malformed source produces bounded diagnostics rather
than a panic; repeated runs are byte-identical. Passing S1 makes no grammar,
semantic, proof, code-generation, cryptographic, or production claim.

### S2 — Editioned grammar and parser

Status: complete at merged revision
`52a3460853636f7cbaa27f3e27d86e032e3c82d4` under D-025 and accepted OEP-0002

Permanent outcomes:

- a mandatory exact `edition 2026;` marker;
- the normative lexical and grammar specification in
  [`LANGUAGE_2026.md`](LANGUAGE_2026.md);
- exactly one module containing empty `spec` or `impl` functions;
- a precisely source-mapped syntax tree;
- bounded error recovery and stable parse diagnostics;
- positive, malformed, ambiguity, duplicate-name, Unicode, line-ending,
  resource-limit, and repeatability cases; and
- exact source and policy inventory.

The directed grammar contains no imports, multiple modules, parameters, types,
expressions, non-empty bodies, semantics, proofs, targets, ABI, leakage, code
generation, packaging, or release behavior. `game`, `proof`, and `claim` remain
lexical reservations only.

Exit test: every accepted form maps to one syntax tree; every rejected form has
a stable error category; resource exhaustion fails closed; and mutation and
repeated parsing reveal no unexplained acceptance, panic, hang, or
nondeterminism. Required hosted checks and local offline checks pass at the
exact merged revision.

### S3 — Semantic core and reference evaluator

Status: active after completed S3a under D-026 and accepted OEP-0003

Permanent outcomes:

- module and name resolution;
- explicit types for mathematical integers and fixed-width words;
- function, binding, control-flow, and failure semantics;
- a typed Core boundary;
- deterministic reference evaluation; and
- one conformance fixture per normative rule.

Exit test: the specification, type checker, evaluator, diagnostics, and
conformance cases agree for every supported construct. No proof or native-code
claim is implied.

#### S3a — Typed literal specifications

Status: completed under D-026 and accepted OEP-0003 at merged revision
`6c0bd3021cf2df603e08808e4660724ca1e2b2a5`

The first bounded semantic slice preserves legacy empty `spec` and `impl`
declarations and adds only `spec NAME() -> TYPE { SIGNED_INTEGER }`. Semantic
acceptance recognizes mathematical `Int` and unsigned `Word[8]`, enforces
same-kind declaration-name uniqueness, lowers typed specifications to a
source-ordered Typed Reference Core, and evaluates those closed literal values
deterministically.

The slice has exact semantic diagnostic, Core-node, integer-input, semantic-
event, and evaluation-step budgets. It defines no operators, calls, parameters,
bindings, control flow, dynamic failure values, typed implementations, canonical
Core encoding, proof identity, refinement, code generation, ABI, leakage,
package, release, or cryptographic behavior. S3a does not complete S3.

Closure evidence: PR #9 merged at `2026-07-13T00:42:10Z` after Required CI run
`29215790064`, Dependency Review run `29215790110`, and CodeQL run
`29215789258` passed. At exact merged revision
`6c0bd3021cf2df603e08808e4660724ca1e2b2a5`, Required CI run `29215877872`,
Workflow Online Audit run `29215877891`, External Links run `29215877874`,
OpenSSF Scorecard run `29215877875`, and dynamic CodeQL run `29215877437` also
completed successfully. The merged slice passed 89 Rust tests, including the
documentation test, 95 Python policy tests, and policy version 0.2.3 with zero
findings.

### S4 — Proof and claim boundary

Status: pending D-006 and dependent decisions

Permanent outcomes:

- a selected proof foundation or an explicitly smaller initial proof scope;
- canonical Core and Proof IR identities;
- authoritative checking rules and an implementation-diverse checker where
  useful;
- fail-closed certificate and solver policy;
- axiom, assumption, and trust inventories; and
- mixed-status claim records.

Exit test: malformed proofs and missing certificates fail closed; solver
timeouts and unknowns never satisfy a claim; repeated offline replay agrees.
All evidence is labeled solo-produced unless the operating model changes.

### S5 — Compiler IRs and one output path

Status: pending S3

Begin with one bounded output path selected by a decision record. Do not create
a target matrix before one path is correct and inspectable.

Permanent outcomes include semantic IRs, deterministic lowering, validation,
artifact inspection, exact target assumptions, and differential tests against
the reference evaluator. A portable C path may be selected for interoperability;
native assurance requires the stronger target and final-byte obligations.

### S6 — Memory, leakage, ABI, and native targets

Status: pending semantic and target decisions

Add ownership, buffers, layout, erasure, leakage traces, target feature models,
one stable foreign boundary, and one native tuple at a time. Each advertised
claim names the exact target, ABI, feature profile, object bytes, assumptions,
and unsupported cases.

### S7 — Cryptography corpus

Status: pending S3 through S6 as applicable

Admit one standards-sourced primitive at a time. Each package retains exact
standards and errata provenance, rights notes, specification, implementation,
vectors, negative cases, interoperability results, performance observations,
and an explicit claim matrix. No local test is called certification.

### S8 — Packages, developer tools, and preview releases

Status: pending usable language behavior

Add immutable resolution, manifests and locks, offline bundles, formatter, LSP,
documentation generator, evidence inspector, and source archives. A solo preview
release requires an explicit release decision, exact source and artifact
digests, reproducible owner build instructions, known limitations, and support
dates. It cannot claim independent rebuild or multi-party release controls.

## 6. Immediate sequence

S3a evidence closure is complete at exact merged revision
`6c0bd3021cf2df603e08808e4660724ca1e2b2a5`, with OEP-0003 accepted and its
local and hosted evidence recorded.

Before S3b expands the Typed Reference Core with pure expressions or calls:

1. decide D-003, the product form that constrains the language and evaluator;
2. decide D-004, the complete semantic strata and Core relationships; and
3. authorize the bounded S3b surface through an OEP with explicit conformance,
   resource, compatibility, threat, and non-claim boundaries.

Only one slice is stabilized at a time. Research may run ahead, but code for a
dependent stage does not claim completion before its inputs are explicit.
The unapproved
[D-003 product-form decision packet](PRODUCT_FORM_DECISION_PACKET.md) and the
conditional
[D-004 semantic-strata decision suite](SEMANTIC_STRATA_DECISION_SUITE.md)
define the owner-executable research that may run ahead. Neither packet decides
its subject or authorizes S3b; the D-004 suite retains a zero-evidence baseline.

## 7. Quality and claim metrics

The solo project tracks evidence it can actually produce:

- deterministic test and fixture pass rate;
- malformed-input rejection, panic, hang, and resource-limit results;
- diagnostic stability and source-span accuracy;
- conformance coverage per implemented rule;
- differential mismatches between implemented paths;
- dependency and trusted-component count;
- offline build and replay success from a clean local environment;
- unresolved semantic, security, and compatibility questions; and
- exact claim/non-claim coverage for each artifact.

Contributor count, independent reviews, external pilots, certifications, and
laboratory results are unavailable metrics in solo mode and are not schedule
dependencies.

## 8. Definition of progress

Progress means a permanent boundary became more complete, deterministic,
documented, tested, and honest about its limitations. Lines of code, screenshots,
syntax breadth, or passing tests outside a stated boundary do not close a gate.

The roadmap changes when evidence or owner direction changes. A future
collaborative mode may add review and operational capabilities, but the current
roadmap remains executable by one person without waiting for that event.
