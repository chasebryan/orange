# Dependency-ordered solo roadmap

Status: directed active roadmap under D-023 and OEP-0001

Snapshot: 2026-07-12

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

Status: active

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

Status: pending

Permanent outcomes:

- an explicit pre-alpha language edition marker;
- a documented lexical and grammar specification;
- a lossless or precisely mapped syntax tree;
- bounded error recovery and stable parse diagnostics;
- positive, negative, ambiguity, Unicode, and resource-limit fixtures; and
- deterministic formatter foundations.

Exit test: every accepted form maps to one syntax tree; every rejected form has
a stable error category; mutation and repeated parsing reveal no unexplained
acceptance, panic, hang, or nondeterminism.

### S3 — Semantic core and reference evaluator

Status: pending

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

The next permanent slices are:

1. finish S1 and freeze the source/span/diagnostic contracts;
2. write the pre-alpha lexical specification from the implemented behavior;
3. decide the smallest editioned module and function grammar for S2;
4. implement a bounded parser with adversarial tests; and
5. specify the first typed expression fragment before implementing S3 checking.

Only one slice is stabilized at a time. Research may run ahead, but code for a
dependent stage does not claim completion before its inputs are explicit.

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
