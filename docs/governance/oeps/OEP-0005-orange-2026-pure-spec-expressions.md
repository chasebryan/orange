---
number: OEP-0005
title: Orange 2026 pure specification expressions
authors:
  - Chase Bryan
champion: Chase Bryan
status: Review
type: Standards
created: 2026-09-28
updated: 2026-09-28
discussion: owner-direction-2026-09-28-s3b
related-decisions:
  - D-002
  - D-004
  - D-023
  - D-025
  - D-026
related-adrs: []
requires:
  - OEP-0001
  - OEP-0002
  - OEP-0003
  - OEP-0004
supersedes: []
superseded-by: null
review-authorities:
  - Orange Project Owner
decision-date: null
decision-revision: null
approval-records: []
---

# OEP-0005: Orange 2026 pure specification expressions

## Abstract

Orange 2026 `spec` functions gain parameters, calls, and pure expressions over
mathematical integers and fixed-width words. The admitted word types become
`Word[8]`, `Word[16]`, `Word[32]`, and `Word[64]`, each the ring of integers
modulo 2^n. Words support modular addition, subtraction, and multiplication;
bitwise and, or, exclusive or, and complement; logical shifts; and rotations.
Integers support exact addition, subtraction, multiplication, and negation.
Operators from different families never share a level without parentheses.

The aim is that a specification reads like the clause of the standard it
transcribes. With this slice, the SHA-256 functions of FIPS 180-4 section
4.1.2 are written as the standard prints them:

```orange
edition 2026;
module sha256 {
  spec choose(x: Word[32], y: Word[32], z: Word[32]) -> Word[32] {
    (x & y) ^ (~x & z)
  }
  spec majority(x: Word[32], y: Word[32], z: Word[32]) -> Word[32] {
    (x & y) ^ (x & z) ^ (y & z)
  }
  spec big_sigma0(x: Word[32]) -> Word[32] { (x >>> 2) ^ (x >>> 13) ^ (x >>> 22) }
  spec big_sigma1(x: Word[32]) -> Word[32] { (x >>> 6) ^ (x >>> 11) ^ (x >>> 25) }
  spec small_sigma0(x: Word[32]) -> Word[32] { (x >>> 7) ^ (x >>> 18) ^ (x >> 3) }
  spec small_sigma1(x: Word[32]) -> Word[32] { (x >>> 17) ^ (x >>> 19) ^ (x >> 10) }
  spec sample() -> Word[32] { big_sigma0(0x6a09_e667) }
}
```

`orangec eval` prints `sha256::sample: Word[32] = 0xce20b47e`.

The normative text is [`docs/EXPRESSIONS_2026.md`](../../EXPRESSIONS_2026.md).
An implementation, 14 fixtures, and a 28-rule conformance runner accompany it
so that the proposal can be reviewed against running code. This proposal is in
**Review**. It accepts no D-004 candidate and gives the Typed Reference Core
no canonical or proof role.

## Motivation

S3a established a typed, deterministic, fail-closed semantic foothold, but it
cannot express a single cryptographic function. Every flagship corpus family
begins with word arithmetic: SHA-2 with 32- and 64-bit rotations, shifts, and
modular addition; ChaCha20 with add-rotate-xor rounds; Keccak with 64-bit
lanes. Until those operations exist in the specification stratum, no corpus
definition can be written, evaluated against official examples, and kept as a
permanent fixture.

The operations chosen are the ones those standards print, with the meaning the
standards assume: a `w`-bit word is an element of the integers modulo 2^w, `+`
is addition modulo 2^w, `ROTR^n` is rotation, and `SHR^n` is a logical shift.
Orange adopts that reading rather than the overflow-checked or undefined
readings of general-purpose languages, because in a specification the ring is
the point.

Pure, total, closed evaluation keeps the slice proof-neutral. Every expression
denotes a value, every call terminates, and nothing observes time, memory, or
secrets.

## Scope and non-goals

This proposal defines parameters, calls, pure expressions, the four word
widths, operator grouping and typing, literal typing, call-graph acyclicity,
Typed Reference Core expressions, reference evaluation, output formats,
diagnostics, and resource limits.

It does not define local bindings, tuples, booleans, comparisons,
conditionals, loops, division or remainder, conversions between types,
variable shift or rotation amounts, recursion, typed `impl` declarations,
contracts, effects, secrecy labels, failure values, imports, multiple modules,
proofs, claims, games, canonical Core serialization, code generation, targets,
ABI, layout, leakage, packages, cryptographic claims, releases, or support.
Each is left for a later bounded slice. `impl` declarations keep their S3a
meaning exactly.

### Strata assumption

D-004 has not selected among its candidates. S3b assumes only what every
candidate already gives the Specification role: pure, total, deterministic
meaning over mathematical values.

- Under `ST-REL`, S3b is a fragment of Spec Core and a candidate for Shared
  Pure.
- Under `ST-UNI`, it is the fragment of the universal calculus with the empty
  effect.
- Under `ST-DUAL`, it is a fragment of the pure Core.
- Under `ST-MIRROR`, it is a fragment of the Spec Core mirror.
- Under `ST-HOST`, it is part of the deterministic semantics that stay local.

The Typed Reference Core remains internal and noncanonical, so no candidate is
chosen by this slice's representation. Whichever candidate D-004 accepts, the
source surface of S3b needs no change; only the Core's placement does, and that
placement is D-004's decision.

## Specification

[`docs/EXPRESSIONS_2026.md`](../../EXPRESSIONS_2026.md) is the complete
normative text. In summary:

- **Tokens.** `<<`, `>>`, `<<<`, and `>>>` are added with longest-match
  lexing.
- **Grammar.** A typed `spec` may declare parameters `name: Type` and has one
  expression body. Parameter and argument lists allow one trailing comma. A `-`
  directly before an integer token is the literal's sign, as in S3a; any other
  prefix `-` is negation.
- **Grouping.** `*` binds more tightly than `+` and `-`. Arithmetic, `&`, `|`,
  and `^` chains associate to the left. Operators from different groups, and
  two shifts or rotations, cannot share a level without parentheses
  (`ORC0108`, one per ungrouped expression).
- **Names.** Bare names are parameters of the enclosing function. Calls name
  typed `spec` functions in the same module, in any source order. The call
  graph must be acyclic.
- **Types.** `Int`, `Word[8]`, `Word[16]`, `Word[32]`, and `Word[64]`, with
  widths spelled in decimal. There is no inference or conversion: every
  expression is checked against an expected type.
- **Operators.** Arithmetic is exact on `Int` and modular on words. Negation is
  `Int` only (`0 - a` on words). Bitwise operators, shifts, and rotations are
  word only. A shift or rotation amount is an unsigned literal from 0 through
  n - 1.
- **Core and evaluation.** Core functions carry parameter types and a typed
  postorder body. `orangec eval` prints each parameterless typed `spec` in
  source order; a `Word[n]` value prints as `0x` and n/4 hexadecimal digits.
- **Diagnostics.** `ORC0108` and `ORC0211` through `ORC0218` are added.
  Errors do not cascade: an undefined operator or a wrong call stops at that
  node, and an unsupported type is reported once where it is written.
- **Limits.** Expression nesting 64 (groups, calls, prefix operators), tree
  height 256, 64 parameters, and 256 arguments are parser limits (`ORC0106`).
  Evaluation shares 1,048,576 steps per source under an exact cost table in
  which `Int` arithmetic pays for its size in 32-bit limbs, allows 256 call
  frames, and bounds `Int` results at 16,384 significant bits (`ORC0301`). The
  call-graph check consumes no semantic events.

The nesting limit is 64, not 256 as an earlier draft proposed. Measured
recursion at 256 nesting levels overflowed a two-mebibyte test thread in debug
builds. With nesting at 64 and operator chains parsed by iteration, the
deepest admitted sources use at most about 720 KiB of stack in debug builds
and 140 KiB in release builds, and a test holds every phase to 1 MiB.

## Alternatives

A conventional precedence table, as in C or Rust, was rejected. C's ordering of
`&` below `==` is a well-known source of bugs, and any table asks the reader to
remember an order the standards never rely on. Requiring parentheses across
groups costs a few characters and makes every expression's structure visible.

Checked or trapping word arithmetic was rejected for the specification
stratum. Cryptographic standards define word operations modulo 2^w, and a
specification that trapped on overflow would say something the standard does
not. Overflow checking belongs to implementation-stratum claims.

Arbitrary widths `Word[n]` were deferred. The four widths cover SHA-2,
ChaCha20, Keccak lanes, and byte-oriented code.

Variable shift and rotation amounts were deferred because out-of-range amounts
need dynamic failure semantics, which no slice has defined. Data-dependent
rotations, as in RC5, would also raise leakage questions that belong with
secrecy labels.

Local bindings and tuples were deferred to keep this slice to one idea. They
are the natural next step: the ChaCha20 fixture has to recompute shared
intermediate values through calls because it cannot name them.

Placing S3b behind an opt-in flag until acceptance was considered and
rejected. It would keep two grammars and two sets of messages alive in the
compiler for a change whose compatibility cost is one fixture line and a few
message strings.

## Compatibility and migration

Every source S3a accepts is accepted by S3b with the same Core values and
output bytes. The change runs the other way:

- `Word[16]`, `Word[32]`, and `Word[64]`, which S3a rejected with `ORC0204`,
  are admitted. The S3a rejection fixture that uses `Word[16]` as its example
  of an unsupported width moves to `Word[12]`.
- The S3a messages that named only `Word[8]` now name the four widths, and the
  typed-`impl` message says "typed bodies" rather than "typed literal bodies".
- `orangec lex` reports `<<`, `>>`, `<<<`, and `>>>` as single tokens. No
  source containing those spellings was syntactically valid before.

The D-004 decision suite once bound those S3a files by digest. It now reads
byte-identical stored copies under `research/decisions/D-004/baseline/`, so the
live S3a files can change without changing any D-004 evidence.

Rollback reverts the lexer tokens, grammar, semantics, Core, evaluator, tests,
fixtures, and normative documents together.

## Semantic and claim effects

This proposal gives exact meanings to parameterized typed specifications,
calls, the listed operators, and the four word types. The supported claim is
limited to deterministic, bounded analysis and evaluation of the documented
fragment at a recorded implementation revision. It establishes no language
soundness, proof, refinement, compilation, cryptographic correctness,
constant-time behavior, compatibility, independent review, or production
readiness. That a function named like `big_sigma0` evaluates to the standard's
example value is not a claim that it transcribes FIPS 180-4; such a claim would
need the provenance and conformance evidence of a future corpus package.

## TCB, axiom, and proof effects

The lexer, parser, semantic analyzer, Core constructor, evaluator, and their
host dependencies remain engineering trust dependencies. The exact integer
arithmetic used by the evaluator is new trusted code and is tested against an
independent 128-bit reference and multi-limb identities. No axiom, theorem,
proof rule, certificate, checker, or solver is introduced.

## Threat, abuse, and leakage effects

The hostile-frontend surface grows to expression nesting, call graphs,
exponential call trees, large intermediate integers, and wider word output.
Fixed nesting, height, list, depth, step, and magnitude budgets, cycle
rejection, a stack bound tested at 1 MiB, and fail-closed evaluation constrain
them. No secrecy label or leakage property is defined; nothing in this slice
says whether an expression would run in constant time on any machine.

## Target and ABI effects

None. `Word[n]` is a mathematical value domain, not a machine representation.

## Standards, errata, and provenance

FIPS 180-4 and RFC 8439 motivate the operator set. The fixtures check the
SHA-256 functions against round 0 of the FIPS 180-4 "abc" example and the
ChaCha20 quarter round against the RFC 8439 section 2.1.1 test vector. No
standard, erratum, or test vector gains normative authority through this
proposal.

## Dependencies, licenses, and IP

No dependency is added. The Rust standard-library-only product graph remains.
D-018 remains unresolved.

## Conformance, tests, and evidence

`compiler/crates/orangec/tests/s3b_conformance.rs` binds the 28 rules of the
specification's index to evidence and fails on any drift. Every rule has a
command-line case that runs twice with identical results, from 14 fixtures or
from generated sources at each limit's exact boundary, except the three rules
whose index entry asks for unit evidence alone. Every rule also names unit
tests in the lexer, parser, semantic analyzer, Core arithmetic, or evaluator,
including word operators against a wide reference at every width, every shift
and rotation amount at every width, exact semantic-event and Core-node
accounting, the step cost table, allocation failure in every phase, and a
1 MiB stack bound for the deepest admitted sources. The S2 and S3a runners
continue to pass after the compatibility edits above.

## Operations, release, and recovery

No service, package, key, or release is added. A defect is recovered by a
regression fixture and a normative correction.

## Support and deprecation

The fragment is pre-alpha and best effort under D-022, with no compatibility
promise. Changes require explicit migration notes.

## Unresolved questions

- Whether bindings and tuples should arrive together in the next slice.
- Whether `Word[n]` should generalize to any width from 1 to some bound.
- Whether a later edition should reserve `let`, `if`, and `Bool`, and how that
  interacts with existing identifiers.
- How D-004's accepted strata will place this fragment.

## Decision record

On 2026-09-28 the owner directed that Orange continue its implementation goals
and that development not freeze unless the owner asks. This proposal records
the S3b surface built under that direction and is presented for the owner's
review. It is not accepted. Acceptance is the owner's decision alone; until it
is recorded here with a decision date, reviewed revision, and `solo-reviewed`
approval record, this proposal authorizes nothing by itself.
