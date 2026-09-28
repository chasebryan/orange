---
number: OEP-0003
title: Orange 2026 typed literal specifications
authors:
  - Chase Bryan
champion: Chase Bryan
status: Accepted
type: Standards
created: 2026-07-12
updated: 2026-07-12
discussion: owner-direction-2026-07-12-s3a
related-decisions:
  - D-002
  - D-003
  - D-004
  - D-006
  - D-023
  - D-025
  - D-026
related-adrs: []
requires:
  - OEP-0001
  - OEP-0002
supersedes: []
superseded-by: null
review-authorities:
  - Orange Project Owner
decision-date: 2026-07-12
decision-revision: 6c0bd3021cf2df603e08808e4660724ca1e2b2a5
approval-records:
  - solo-reviewed owner acceptance at merged revision 6c0bd3021cf2df603e08808e4660724ca1e2b2a5
---

# OEP-0003: Orange 2026 typed literal specifications

## Abstract

Orange 2026 gains one proof-neutral semantic foothold: a `spec` function may
declare an explicit result type and contain exactly one signed integer literal.
The only semantically accepted result types are mathematical `Int` and unsigned
`Word[8]`. Existing empty `spec` and `impl` declarations remain valid.

Successful semantic analysis produces a source-ordered Typed Reference Core for
typed specifications. `orangec eval FILE` prints every such value in source
order. The Core is an internal typed boundary without a canonical encoding,
proof identity, refinement relation, target meaning, or release claim.

The owner direction recorded here authorized bounded S3a implementation under
OEP-0001 and D-023. This proposal is accepted at the exact merged S3a revision
recorded below. Acceptance is pre-alpha semantic authority for this bounded
slice, not release authority.

## Motivation

S2 deliberately stopped after deterministic syntax. S3 needs an equally narrow
semantic step that can exercise parsed type syntax, namespace checks, literal
decoding, typed elaboration, deterministic evaluation, and fail-closed resource
behavior without deciding Orange's proof system or complete semantic strata.

A literal-only slice makes the arithmetic-domain distinction observable while
keeping every accepted value closed and deterministic. It also lets Orange
preserve its existing parser foothold: empty declarations remain useful syntax
fixtures and are not assigned invented runtime behavior.

## Scope and non-goals

This proposal defines the additive typed-`spec` grammar, declaration namespace
uniqueness, the exact `Int` and `Word[8]` acceptance rules, signed-literal
decoding, Typed Reference Core construction, reference evaluation, output
format, diagnostics, resource limits, and conformance boundary in
[`docs/SEMANTICS_2026.md`](../../SEMANTICS_2026.md).

It does not define typed `impl` declarations, parameters, calls, operators,
bindings, statements, control flow, dynamic failure values, recursion,
conversions, inference, contracts, effects, imports, multiple modules, proofs,
claims, games, canonical Core serialization, code generation, targets, ABI,
layout, leakage, packages, cryptographic behavior, releases, or support.

In particular, this slice does not accept D-003 or D-004. It does not decide
whether the Typed Reference Core later becomes part of Spec Core, Impl Core, a
shared pure fragment, or another formally related representation.

## Specification

### Surface grammar

The complete S2 grammar remains valid. One typed specification alternative is
added by factoring the existing `spec` declaration tail:

```text
function_decl   = "spec" IDENTIFIER "(" ")" spec_tail
                | "impl" IDENTIFIER "(" ")" empty_body ;
spec_tail       = empty_body
                | "->" parsed_type "{" signed_integer "}" ;
empty_body      = "{" "}" ;
parsed_type     = IDENTIFIER ("[" INTEGER "]")? ;
signed_integer  = "-"? INTEGER ;
```

The accepted S2 edition, source, lexical, token, syntax-tree, diagnostic, and
parser-resource rules remain unchanged. `parsed_type` is deliberately broader
than semantic acceptance so a syntactically valid unsupported type receives a
semantic diagnostic rather than being silently assigned meaning. A typed
`impl`, a nonliteral expression, or any additional body token remains a syntax
error.

### Names and declarations

Function identifiers are compared by exact ASCII spelling in two namespaces
keyed by `(function kind, name)`. A repeated `spec` name or repeated `impl` name
is a semantic error, including a collision between an empty declaration and a
typed declaration. A `spec` and `impl` may have the same name. No declaration
can be referenced in this slice, and module names acquire no import or linkage
meaning.

Empty declarations contribute to namespace checking but do not produce Core
functions or values. They retain no execution, typing, proof, or refinement
meaning.

### Types and literals

The only semantically accepted parsed types are the exact contextual spellings
`Int` and `Word[8]`:

- `Int` has no width argument and denotes mathematical signed integers. The
  optional source minus sign negates the decoded nonnegative magnitude, and
  negative zero normalizes to mathematical zero.
- `Word[8]` has the exact decimal width spelling `8` and denotes unsigned values
  from 0 through 255. Any source minus sign, including in `-0`, is an error. A
  magnitude greater than 255 is an error.

Integer magnitudes use the base and separator rules already fixed by Orange
2026 lexing. There is no wrapping, truncation, coercion, inference, or implicit
conversion. `Int[8]`, bare `Word`, another width, another identifier, or a
malformed contextual spelling is semantically rejected.

`Int` is an unbounded mathematical domain. The significant-input-bit limit is
a resource boundary on this compiler slice, not a finite-width definition of
the type.

### Typed Reference Core

Semantic success produces one Typed Reference Core module containing only typed
`spec` declarations. Each Core function contains:

- a zero-based function ID assigned contiguously in typed-spec source order;
- the exact module and function names;
- exactly one type, `Int` or `Word8`; and
- exactly one normalized literal value of that type.

The Core has no expression operators, calls, effects, failure values, proof
terms, axioms, target information, or implementation declarations. Function IDs
are deterministic within one successfully analyzed source and carry no
cross-revision stability promise.

This Core is a permanent typed compiler boundary for the S3a slice, but it has
no canonical byte encoding, content digest, theorem fingerprint, proof-checking
role, refinement relation, erasure map, or public interchange compatibility.

### Reference evaluation and CLI

`orangec check` runs lexical, syntactic, and semantic validation. Any diagnostic
from those phases makes the source unsuccessful and prevents Core acceptance.

`orangec eval FILE` accepts exactly one source operand, performs the same checks,
and evaluates every Typed Reference Core function in source order. Each result
is written to standard output as one line:

```text
module::name: Type = value
```

An `Int` value uses canonical base-10 notation with one leading `-` only for a
negative value. A `Word[8]` value uses exactly `0x` followed by two lowercase
hexadecimal digits. Examples are `demo::answer: Int = 42` and
`demo::mask: Word[8] = 0xff`. A successful Core with no typed specifications
writes zero bytes. No partial value output is permitted when analysis or
evaluation fails.

### Failure and resource behavior

Semantic analysis retains at most 100 ordinary diagnostics followed, when
necessary, by one stable suppression diagnostic. The suppression diagnostic
does not convert failure into success. Resource exhaustion has its own stable
diagnostic and cannot be hidden by the ordinary diagnostic limit.

One source is limited to 262,144 Typed Reference Core nodes, 1,048,576 semantic
events, 16,384 significant bits in any decoded integer magnitude, and 1,048,576
reference-evaluation steps. Leading zeroes do not increase significant bit
length; zero has zero significant bits. A semantic event is one declaration
namespace operation, parsed-type component inspection, literal component
inspection, diagnostic emission attempt, or Core-node construction. Evaluation
uses one step for each Core function whose value is visited for rendering.

Exhausting any limit fails closed without an accepted Core or evaluation
output. Implementation allocation failure inside a stated limit remains a host
failure and is not an Orange semantic value.

For identical source bytes, edition, compiler revision, and command, namespace
results, normalized values, Core function order and IDs, diagnostic categories
and order, exit status, and evaluation bytes are deterministic.

## Alternatives

Replacing empty bodies was rejected because it would discard the permanent S2
fixtures without a semantic need. Adding typed `impl` declarations was deferred
because even a literal-only implementation declaration would invite premature
execution and refinement interpretations.

Operators, calls, parameters, local bindings, more word widths, type inference,
and explicit conversion syntax were deferred. Each would add rules that are not
needed to establish the first typed boundary. Treating every parsed type as a
syntax error was rejected because a generic parsed type shape preserves precise
semantic rejection and a useful future parser boundary.

Making the Core canonical or assigning it immediately to the complete Spec/Impl
Core family was rejected for this slice because D-003, D-004, and D-006 remain
open. Such a choice requires broader representative programs and proof-boundary
evidence.

## Compatibility and migration

Every source accepted by the S2 grammar remains syntactically valid. Empty
declarations remain valid, but same-kind duplicate names that previously had no
binding consequence now fail semantic checking. A same-named `spec` and `impl`
continues to succeed because the namespaces are separate.

The change extends `orangec check` from lexical and syntactic validation to
semantic validation. Automation that relied on same-kind duplicate acceptance
must rename or remove a declaration. Typed sources are new pre-alpha syntax and
have no prior compatibility promise.

Rollback reverts the typed grammar, semantics, Core, evaluator, tests, and
normative documents together. Typed specifications then become syntax errors;
the legacy S2 empty grammar remains. No binary, package, evidence, target, ABI,
or canonical-format compatibility surface is created.

## Semantic and claim effects

This proposal gives exact meanings only to closed typed specification literals,
the two accepted types, namespace uniqueness, their Typed Reference Core, and
their deterministic displayed values. An empty declaration still has no value.

The supported claim is limited to deterministic bounded analysis and evaluation
of the documented fragment at a recorded implementation revision. Test success
does not establish language soundness, semantic completeness, proof soundness,
implementation refinement, compilation correctness, cryptographic correctness,
constant-time behavior, compatibility, independent review, or production
readiness.

## TCB, axiom, and proof effects

The Rust lexer, parser, semantic analyzer, integer decoder, Core constructor,
evaluator, diagnostic renderer, standard library, pinned toolchain, host, and
sole owner become engineering trust dependencies for the S3a results. They do
not enter a logical proof TCB because no proof judgment exists.

No axiom, theorem, proof rule, certificate, proof format, proof checker, solver,
canonical theorem identity, or mechanized soundness claim is introduced. Core
typing and evaluator tests are implementation evidence only.

## Threat, abuse, and leakage effects

This slice expands TB-008 and the TM-014 hostile-frontend surface to names,
contextual types, large integer decoding, semantic diagnostic floods, Core
construction, and evaluator output. Fixed semantic, Core, integer, and
evaluation budgets; exact contextual spellings; fail-closed analysis; stable
diagnostics; and no partial output constrain those paths.

TM-005 and TM-010 evidence-confusion risks also apply: a Typed Reference Core or
evaluation result could be misrepresented as canonical, proved, refining an
implementation, or cryptographically meaningful. Explicit non-claims and the
absence of proof/target fields limit that confusion but do not replace external
review. No secrecy label, leakage trace, constant-time property, or physical
side-channel property is defined.

Residual risks include allocation failure inside a budget, compiler or host
defects, algorithmic mistakes in integer decoding or formatting, diagnostic
drift, and a shared mistake in the solo-authored specification and
implementation.

## Target and ABI effects

The analyzer and evaluator are host-side Rust code. They emit text only and
select no Orange execution target, object format, layout, CPU feature, calling
convention, foreign boundary, stable symbol, or ABI. `Word[8]` is a mathematical
eight-bit value domain, not a selection of a machine representation.

Observed host execution remains pre-alpha evidence and is not a host support
promise.

## Standards, errata, and provenance

No external language, arithmetic, cryptographic, ABI, or encoding standard is
incorporated normatively. The grammar delta, type names, literal rules, Core,
and evaluator format are owner-directed Orange project material.

No standard, erratum, test vector, or external proof gains authority through
this proposal.

## Dependencies, licenses, and IP

The implementation must retain the Rust standard-library-only product
dependency graph unless a separate dependency decision is accepted. This
proposal admits no crate, generator, build script, network fetch, or generated
source.

The repository-wide outbound license remains unresolved under D-018. This
proposal grants no third-party permission and makes no final-name, trademark,
patent, export, or redistribution claim.

## Conformance, tests, and evidence

Conformance requires at least one positive and one applicable negative case for
every normative grammar, namespace, type, literal, Core, output, diagnostic,
resource, and determinism rule. The permanent corpus includes legacy empty
declarations; mixed empty and typed specifications; cross-kind equal names;
same-kind duplicates; each parsed-but-unsupported type shape; `Int` positive,
zero, negative, negative-zero, and each lexical base; `Word[8]` boundaries;
negative and out-of-range words; exact Core order and IDs; empty output; exact
decimal and hexadecimal output; every resource limit; suppression; repeated
analysis; and repeated evaluation.

Repository evidence must pass formatting, linting, Rust unit and CLI tests,
offline locked dependency checks, the foundation policy and adversarial tests,
exact inventory validation, and required hosted checks. The exact merged
revision and time-indexed hosted evidence must exist before this proposal can
be Accepted.

Passing tests establish only the tested behavior at the recorded revision.
They are not proof, an independent review, a security audit, a cryptographic
validation, or release evidence.

## Operations, release, and recovery

`orangec eval` is a local pre-alpha command. This proposal adds no service,
deployment, network endpoint, registry, package, signing key, update path,
monitoring commitment, or release operation. Generated build output remains
untracked.

A defect is recovered by preserving history, reverting or narrowing the
complete typed-semantic change, and adding a regression fixture. A wrong
semantic rule requires a normative correction and migration analysis, not only
an implementation patch. No product release is authorized.

## Support and deprecation

The S3a fragment is pre-alpha and best effort under D-022. There is no SLA, LTS
window, compatibility promise, migration service, or production-support
commitment. The exact type and output forms may change only through an explicit
edition-aware semantic decision with migration notes.

Permanent-lineage means the analyzer, Core boundary, evaluator, diagnostics,
and tests are extended in place; it does not make this accepted pre-alpha
fragment a stable public interface.

## Unresolved questions

The final product form, complete semantic strata, proof foundation, canonical
Core identities, additional types and word widths, operators, calls, parameters,
bindings, control flow, dynamic failure, contracts, effects, implementation
semantics, refinement, targets, ABI, leakage, packages, cryptographic corpus,
release process, and support window remain unresolved.

None is silently answered by the literal-only Typed Reference Core. Further S3
slices require later bounded decisions and conformance cases.

## Decision record

On 2026-07-12 the project owner directed the exact typed-spec grammar, contextual
types, literal rules, namespace boundary, Typed Reference Core, evaluator
format, resource budgets, non-goals, and solo claim boundary recorded here.
Under OEP-0001 and D-023, that direction is immediately effective and permits
the bounded S3a implementation to proceed.

On 2026-07-12 the Orange Project Owner reviewed and accepted this proposal at
exact S3a merge revision
`6c0bd3021cf2df603e08808e4660724ca1e2b2a5`. PR #9 head
`8c48a85997b756cf65d64110ebc869bb26e49079` passed Required CI run
`29215790064`, Dependency Review run `29215790110`, and CodeQL run
`29215789258` before the squash merge at `2026-07-13T00:42:10Z`. At the exact
merged revision, Required CI run `29215877872`, Workflow Online Audit run
`29215877891`, External Links run `29215877874`, OpenSSF Scorecard run
`29215877875`, and dynamic CodeQL run `29215877437` also completed
successfully. Acceptance evidence additionally includes 89 passing Rust tests,
including the documentation test, 95 passing Python policy tests, and policy
version 0.2.3 reporting zero findings.

The approval record is literally `solo-reviewed` and binds the exact decision
revision. The author and decision authority are the same sole owner. No
independent approval, proof, audit, external validation, product release, or
cryptographic, target, ABI, leakage, or code-generation claim is created by
this acceptance.
