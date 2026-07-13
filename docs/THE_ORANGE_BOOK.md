# The Orange Book

![Official Orange emblem and wordmark](../assets/brand/orange-banner.png)

By Chase Bryan

Status: living pre-alpha reader guide

Snapshot: 2026-07-12

Manuscript version: 0.1

> The Orange Book explains why Orange exists, what it is intended to become,
> what has actually been built, and which questions remain open. It is not a
> normative language specification, proof, assurance report, license grant, or
> release claim.

## Contents

- [Preface](#preface)
- [Chapter 1: The Seams Are the System](#chapter-1-the-seams-are-the-system)
- [Manuscript map](#manuscript-map)
- [Sources and drafting disclosure](#sources-and-drafting-disclosure)

## Preface

Orange begins with an uncomfortable observation: high-assurance cryptography is
not one problem. It is a chain of problems whose boundaries are easy to hide.
A mathematical construction, an executable implementation, a proof, a compiler,
a test suite, a binary, and the machine that runs it can each be reasonable in
isolation while the claim connecting them remains unclear.

This book is the reader's guide to that chain. It is meant for cryptographic
implementers, verification engineers, cryptographers, standards authors,
library maintainers, integrators, auditors, and curious programmers who want to
understand the project without first reading every planning record. It will
explain the ideas in ordinary technical prose, show the permanent implementation
as it grows, and keep the boundary between aspiration and evidence visible.

That boundary matters because Orange is young. The repository is in solo,
pre-alpha compiler development. It has a production-lineage Rust compiler
foundation, a deterministic lexer, a bounded parser, structured diagnostics,
and a deliberately tiny Orange 2026 grammar. The accepted S3a slice adds
bounded semantic checking and reference evaluation for closed typed
`spec` literals only. It does not add a general expression language, typed
implementations, refinement, code generation, a standard library, a proof
checker, package or release behavior, or a verified cryptographic
implementation. A passing test suite is
evidence about the implemented slice; it is not evidence that the eventual
language or compiler is sound.

The manuscript uses four kinds of statements:

- **Current** describes behavior or evidence present in the repository now.
- **Directed** describes an explicit project-owner decision that controls work.
- **Proposed** describes an architecture or policy recommended for a later
  decision gate.
- **Future** describes an intended capability whose design, implementation, or
  evidence is not complete.

The distinction is not decorative. A proposed architecture cannot become an
accepted one merely because a chapter speaks about it fluently. When this book
and a normative source disagree, the normative source, accepted Orange
Enhancement Proposal, and [decision register](DECISIONS.md) control. The book
must then be corrected.

The title **The Orange Book** and the name **Orange** are repository-local
working names. They do not assert trademark clearance or authorize publication
to a package registry, domain, or other public namespace. The manuscript is a
living part of the solo project, not a product release.

## Chapter 1: The Seams Are the System

Cryptographic software is asked to carry several different meanings at once.
It has a mathematical meaning: a function, construction, or protocol is being
described. It has an operational meaning: instructions execute on concrete
machines with finite words, memory, errors, and timing behavior. It has a
security meaning: an attacker is granted certain powers and a property is
claimed under stated assumptions. Finally, it has an evidentiary meaning: some
combination of proofs, certificates, tests, reviews, provenance, and build
records is supposed to justify what users are told.

Those meanings rarely live in one place today. A specification may be written
in a notation suited to mathematicians. A fast implementation may be C, Rust,
or assembly. Functional correctness may be argued in a proof assistant.
Constant-time behavior may be checked by a separate analysis. Game-based
security may use yet another formalism. Test vectors, compiler flags, linker
inputs, target features, and release attestations collect around the outside.

Each tool can be excellent. The difficulty is the crossings between them.

### Six questions for one artifact

Imagine receiving a native library that exports a cryptographic routine. A
serious account of that library should answer at least six questions:

1. What mathematical construction or protocol component is intended?
2. Which executable implementation is claimed to realize it?
3. Which properties are claimed, for which inputs, targets, and versions?
4. Which assumptions and leakage model limit each property?
5. Which proof object, checked certificate, test corpus, or external record
   supports each claim?
6. Which source, toolchain, dependencies, and invocation produced the shipped
   bytes?

It is possible to have good answers to several of these questions and no
reliable answer to the next one. A proof about a mathematical function does not
identify the binary loaded by an application. A correct implementation at one
intermediate representation does not establish that a later optimization kept
its behavior. Passing official vectors does not cover all inputs. Source-level
control-flow discipline does not automatically describe final machine-code
leakage. A reproducible build faithfully reproduces a bug just as readily as a
correct program.

The transition is therefore part of the claim. Serialization is not merely
plumbing when a theorem fingerprint can be attached to the wrong definition.
Foreign-function glue is not merely plumbing when buffer length, aliasing,
alignment, or error behavior can violate a proved precondition. The compiler is
not merely plumbing when the advertised property must survive to object bytes.
The release process is not merely plumbing when an attacker can replace either
the code or its evidence.

The seams are part of the system.

### The proposed vertical artifact

Orange's directed mission is to specify, implement, and verify cryptography.
The project's proposed answer to the seam problem is a claim-oriented language
and build graph. In the intended end state, a package connects standards
provenance, readable specifications, executable implementations, named target
and leakage models, checked transformations, native artifacts, foreign-interface
metadata, tests, and explicit assumptions.

The important word is *connects*. Orange is not useful merely because it can
place several kinds of text in one source file. A shared spelling is not a
semantic relationship. The system must preserve exact identities and record
which checked step supports each edge of the graph. Where a relationship has
not been established, the graph must say so.

This is why the long-term product is larger than a notation or compiler front
end. A language can make intent expressible, but an assurance claim also needs
semantics, checking rules, artifact identity, assumptions, and replay. A code
generator can produce fast bytes, but speed says nothing about whether those
bytes implement the named specification. A theorem prover can check a proof,
but the theorem may not be the property an integrator thinks it is.

Many architectural details of this vertical artifact remain proposed or under
investigation. The current Typed Reference Core for literal specifications
has no canonical encoding, proof identity, or refinement role and does not
select the complete semantic Core. Orange has also not selected its proof
foundation, proof format, solver policy, leakage baseline, native target
envelope, stable foreign boundary, package model, or flagship cryptography
corpus. This chapter describes the problem those choices must eventually solve;
it does not settle them.

### Claims, not labels

The word *verified* compresses too much. It can refer to well-formed syntax,
memory safety, functional refinement, standards conformance, termination,
constant-time behavior under a particular observation model, compiler
preservation, ABI correctness, game-based security, or simply the fact that
tests were run. These properties are related, but none is a universal substitute
for the others.

Suppose a routine passes every published vector for a standard. That is useful
conformance evidence for those cases. It is not a proof for every input, and it
does not establish memory safety. Suppose a proof establishes that a source
implementation refines a mathematical specification. That does not, by itself,
show that emitted machine code preserves the result or that its memory addresses
are independent of secret data. Suppose a binary is rebuilt byte for byte by a
second machine. That establishes a reproducibility fact, not cryptographic
correctness.

Orange therefore proposes to make the unit of assurance a scoped claim. A claim
should name its subject, property, model, target, assumptions, evidence, and
outcome. Different claims about one exported routine can have different states.
A conformance claim may be satisfied while a leakage claim is unresolved. A
platform may be unsupported even though a mathematical proof is valid. An
external validation can be recorded without being misrepresented as a theorem
checked by the Orange kernel.

The intended outcomes also need more precision than success and failure. A
claim can be satisfied, not satisfied, unresolved, or unsupported. A timeout is
not a proof failure, but it cannot become a proof success. An assumption is a
visible dependency, not evidence that proves itself. A neighboring
implementation's test result cannot silently migrate to the implementation
being shipped.

In the proposed design, this discipline would change the shape of a build.
Instead of producing a binary and then attaching a broad adjective, the build
would produce an artifact together with a graph of narrowly worded claims. Each
edge would name the authority that justifies it. Some authorities may be
machine-checked proofs. Some may be checked certificates. Some may be test runs,
owner review, or identified external records. Their differences would remain
visible.

### Trust does not disappear

Formal methods can shrink and clarify trust, but they do not make trust vanish.
A small proof kernel is still software. The statement fed to it can be wrong.
The parser can construct the wrong syntax tree. A compiler model can omit an
instruction behavior. An assembler or linker can break the connection to final
bytes. A foreign caller can violate a buffer contract. A CPU, operating system,
or entropy source can behave outside the model. A release account can be
compromised.

Orange's intended response is to publish the trusted computing base for each
kind of claim and to keep it specific. The trusted base for a parser behavior
claim is not the same as the trusted base for a native constant-time claim. A
component appears because a claim actually depends on it, not because every
claim inherits one project-wide trust list.

Tests remain important inside this approach. They find regressions, exercise
error paths, compare implementations, and expose resource failures. They can
also provide the right basis for an empirical claim. The boundary is that a
test does not change its authority when a stronger proof is missing. Honest
evidence is useful evidence precisely because its limits are recorded.

### What exists now

The current Orange implementation is deliberately narrow. The permanent Rust
compiler lineage provides source identities and UTF-8 byte spans, deterministic
lexing, stable diagnostic codes, and the `orangec` command-line boundary. The
Orange 2026 parser recognizes exactly one edition declaration followed by one
module. Legacy empty `spec` and `impl` functions remain valid, and the
accepted S3a grammar adds closed typed-literal specifications:

PR #9 merged that bounded pre-alpha implementation and its normative records as
commit `6c0bd3021cf2df603e08808e4660724ca1e2b2a5`. The larger S3 milestone and
the D-003/D-004 architecture decisions remain open.

```orange
edition 2026;
module demo {
  spec identity() {}
  impl rounds() {}
  spec answer() -> Int { 42 }
  spec mask() -> Word[8] { 0xff }
}
```

`orangec check` lexes, parses, and semantically validates that source. Function
names must be unique within separate `spec` and `impl` namespaces, so the two
kinds may share a spelling while a same-kind duplicate fails. Semantic type
acceptance is contextual and exact: `Int` denotes mathematical signed integers,
and `Word[8]` accepts only unsigned values from 0 through 255 without wrapping,
truncation, or coercion.

Successful typed specifications lower in source order to a bounded Typed
Reference Core. Running `orangec eval FILE` prints:

```text
demo::answer: Int = 42
demo::mask: Word[8] = 0xff
```

Empty declarations still have no type, value, or execution meaning. The Core is
noncanonical and carries no proof identity or relationship between a `spec` and
an `impl`. The fragment has no parameters, operators, calls, bindings, control
flow, general failure values, proof terms, targets, ABI rules, or code generation.
The reserved words `game`, `proof`, and `claim` still introduce no usable
constructs.

These absences are not disguised as a miniature finished language. The parser,
semantic analyzer, Core constructor, and evaluator are bounded components at
their intended incremental boundaries. Parse success means the source has the
recorded syntactic shape. S3a semantic and evaluation success means only that a
closed typed literal satisfied the accepted rules and produced the displayed
value. Neither result means the source is correct cryptography, a valid proof,
safe machine code, a refining implementation, or a generally executable
program.

This is the project's no-disposable-prototype rule in practice. Orange grows by
adding permanent components with explicit interfaces, deterministic behavior,
diagnostics, tests, and migration rules. The rule does not make the early system
large. It makes each small piece honest about where it belongs and what it can
show.

### The reader's habit

The central habit of this book is to ask one question whenever a strong sentence
appears: *what connects that sentence to the exact artifact under discussion?*

Sometimes the answer will be a directed project decision. Sometimes it will be
a normative rule and a conformance test. Later, it may be a proof term, a checked
translation certificate, an object-code inspection record, a standards source,
or an external validation with exact scope. Often, during pre-alpha development,
the answer will be that the connection is proposed or does not exist yet.

That last answer is not a defeat. An explicit gap is a tractable engineering
fact. A hidden gap is an unbounded trust claim.

Orange's first thesis is therefore simple: the path from intent to shipped bytes
must be part of the product. Its second thesis follows immediately: every claim
about that path must say what it covers, what supports it, and where it stops.

## Manuscript map

The map is a writing plan, not an architecture decision or delivery schedule.
Chapter names may change as the normative design changes.

| Part | Chapter | State | Governing boundary |
| --- | --- | --- | --- |
| I — Why Orange | 1. The Seams Are the System | Drafted in v0.1 | Directed mission; current limits; proposed claim-oriented graph |
| I — Why Orange | 2. Claims, Not Labels | Planned | Public claim model remains proposed |
| I — Why Orange | 3. One Language, Several Semantic Worlds | Planned | Product form and semantic strata remain proposed |
| II — Meaning and Trust | 4. From Surface Text to Meaning | Planned | Accepted typed-literal Core and evaluator exist; complete semantic Core remains open |
| II — Meaning and Trust | 5. Proof Search Is Not Proof Checking | Planned | Proof foundation and checker remain unsettled |
| II — Meaning and Trust | 6. Secrets Are a Semantic Concern | Planned | Leakage baseline and target models remain unsettled |
| III — Building the Language | 7. No Disposable Prototype | Planned | Directed production-lineage doctrine |
| III — Building the Language | 8. Orange 2026: The Smallest Honest Slice | Planned | Current parser plus accepted typed-literal semantics |
| III — Building the Language | 9. From Core to Native Bytes | Planned | Compiler strategy and targets remain proposed |
| III — Building the Language | 10. The Foreign Boundary | Planned | ABI and generated interfaces remain proposed |
| IV — Cryptography in Practice | 11. Standards as Versioned Inputs | Planned | Exact source and rights decisions are required |
| IV — Cryptography in Practice | 12. The Corpus as Acceptance Test | Planned | Flagship corpus remains proposed |
| IV — Cryptography in Practice | 13. Interoperability and External Validation | Planned | No certification or external validation is claimed |
| V — Operating Orange | 14. Evidence That Survives the Build | Planned | Package, evidence, and release formats remain proposed |
| V — Operating Orange | 15. Offline Replay and Trust Budgets | Planned | Replay is a product direction, not current behavior |
| V — Operating Orange | 16. Solo Work Through Incremental Gates | Planned | Directed solo operating model |
| V — Operating Orange | 17. Releases, Updates, and Failure | Planned | No release is currently authorized |
| Appendices | Current grammar and CLI; decision ledger; claim vocabulary; source notes | Planned | Must track the normative repository state |

## Sources and drafting disclosure

This manuscript is an explanatory synthesis of repository-local material. Its
principal sources for version 0.1 are:

- the [project charter](PROJECT_CHARTER.md) for mission, users, scope, and
  engineering doctrine;
- the [research and landscape analysis](RESEARCH.md) for the polyglot seam and
  vertical-artifact framing;
- the [assurance and security model](ASSURANCE.md) for independent claim
  dimensions, evidence bases, and trust boundaries;
- the [decision register](DECISIONS.md) for the distinction between directed,
  proposed, investigative, and unresolved choices;
- the [dependency-ordered roadmap](ROADMAP.md) for current capability status;
- the [Orange 2026 lexical and grammar specification](LANGUAGE_2026.md) for the
  normative parser boundary;
- the [accepted typed-literal semantics](SEMANTICS_2026.md) and
  [OEP-0003](governance/oeps/OEP-0003-orange-2026-typed-literals.md) for the
  bounded S3a meaning and non-claims; and
- the [compiler guide](../compiler/README.md) for implemented CLI behavior.

Initial manuscript version 0.1—the structure, preface, manuscript map, and
Chapter 1—was drafted with OpenAI Codex, based on GPT-5, under Chase Bryan's
direction on 2026-07-12. Chase Bryan is the named author and remains accountable
for review, correctness, provenance, and future revisions. AI-assisted prose is
not a primary source, proof, independent review, or license provenance.

The repository has no selected outbound documentation license under D-018. No
license or redistribution grant should be inferred from this manuscript.
