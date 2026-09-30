---
number: OEP-0012
title: Orange 2026 integers modulo a constant and type declarations
authors:
  - Chase Bryan
champion: Chase Bryan
status: Review
type: Standards
created: 2026-09-30
updated: 2026-09-30
discussion: owner-direction-2026-09-30-s3i
related-decisions:
  - D-002
  - D-004
  - D-011
  - D-023
  - D-025
  - D-026
related-adrs: []
requires:
  - OEP-0001
  - OEP-0002
  - OEP-0003
  - OEP-0004
  - OEP-0005
  - OEP-0006
  - OEP-0007
  - OEP-0008
  - OEP-0009
  - OEP-0010
  - OEP-0011
supersedes: []
superseded-by: null
review-authorities:
  - Orange Project Owner
decision-date: null
decision-revision: null
approval-records: []
---

# OEP-0012: Orange 2026 integers modulo a constant and type declarations

## Abstract

The type `Mod[m]` holds the integers modulo a constant m, from 2 through
2^521 - 1. Its values are least residues; `+`, `-`, `*`, and prefix `-`
reduce by themselves, `/` multiplies by the inverse and gives 0 for a
non-unit, and `==` and `!=` compare. A modulus is a constant expression of
literals, `+`, `-`, `*`, and `<<`, so a field is written as its standard
writes it. A `type` declaration names a type for the rest of its module.
`as` converts among `Int`, words, and residues by least residues, and
nothing else crosses from one ring to another.

With this slice, Poly1305 is written over the field its RFC names, with no
reduction in sight:

```orange
module poly1305 {
  type P = Mod[(1 << 130) - 5];

  // RFC 8439 section 2.5.1: add one block to the accumulator and multiply by r.
  spec absorb(a: P, r: P, block: P) -> P { (a + block) * r }
}
```

The normative text is [`docs/MODULAR_2026.md`](../../MODULAR_2026.md). An
implementation, seven programs, and a 13-rule conformance runner accompany it
so that the proposal can be reviewed against running code. This proposal is
in **Review** and requires OEP-0011, which is also in review. It accepts no
D-004 candidate and gives the Typed Reference Core no canonical or proof
role.

## Motivation

Much of public-key cryptography, and some symmetric cryptography, is
arithmetic in the integers modulo a fixed number: X25519 and Ed25519 in the
field of 2^255 - 19 elements, Poly1305 in that of 2^130 - 5, the NIST curves
in their prime fields, and ML-KEM and ML-DSA in the integers modulo 3329 and
8380417. The standards write that arithmetic without reductions: RFC 7748
writes `AA = A^2` and means the square in its field, and FIPS 203 writes
`zeta^2` and means it modulo q.

Through S3h an Orange program wrote the same arithmetic with exact `Int`
values and a `% p` after every product; S3f's X25519 fixture reduces by
hand twelve times. A transcription that forgets one is still a valid program; it is merely
wrong, and only a test vector notices. A reviewer comparing the program with
its RFC must check every reduction as well as every formula.

S3i puts the ring in the type. A value of `Mod[p]` is always reduced, so a
reduction cannot be forgotten, and two rings cannot be mixed without an `as`.
The program then reads as the RFC reads, line for line, and the reviewer
checks formulas only.

`type` declarations follow from the same goal. A field whose modulus is
written in full at every parameter is as hard to review as a missing
reduction, and a standard names its field once: "let p = 2^255 - 19".

## Scope and non-goals

This proposal defines the type `Mod[m]` and its constant moduli, their
identity and display, residue literals, operators, total division,
conversions, indexing through least residues, `type` declarations and their
errors, residue types and values in Core, the evaluator's step costs for
residues, and their limits. It adds the diagnostic codes `ORC0232` and
`ORC0233`.

It does not define generic moduli or type parameters, moduli computed at run
time, named constants, extension fields, an exponentiation operator, square
roots or Legendre symbols, Montgomery or any other representation, an order
or bits on residues, implicit conversions, a primality check, new types by
declaration, or type names shared between modules. Moduli of more than 521
bits, such as RSA's, have no type. It makes no timing, secrecy, or leakage
claim.

### Strata assumption

As for S3b through S3h, S3i assumes only what every D-004 candidate gives the
Specification role: pure, total, deterministic meaning. Every residue
operation is a total function of least residues, and division is total by
the stated convention, so under `ST-REL`, `ST-UNI`, `ST-DUAL`, `ST-MIRROR`,
and `ST-HOST` alike the source surface needs no change. Which representation
an implementation stratum refines a residue to, and how a proof relates the
two, is D-004's question and D-006's, and is left open.

## Specification

[`docs/MODULAR_2026.md`](../../MODULAR_2026.md) is the complete normative
text. In summary:

- **Syntax.** `type NAME = TYPE;` declarations follow a module's `use`
  declarations and precede its functions, at most 64 of them; `type` is a
  word only there. `Mod[...]` takes an expression, which opens one nesting
  level and counts toward the height of the expression holding the type.
- **Moduli.** A modulus is a constant built from integer literals with `+`,
  `-`, `*`, `<<`, and parentheses, from 2 through 2^521 - 1, evaluated once
  per module before any type is resolved (`ORC0232`, `ORC0205`). Two moduli
  are one type when their values are equal, and a type displays its modulus
  canonically, as `Mod[(1 << 255) - 19]`.
- **Names.** Declarations resolve in source order to their types; a name is
  not built in and not repeated (`ORC0233`), uses only earlier names, and
  names no array of arrays (`ORC0203`). A declared name is another spelling
  of its type and stays in its module.
- **Residues.** A literal of `Mod[m]` lies strictly between -m and m, and a
  negative one stands for m less its magnitude (`ORC0207`). `+`, `-`, `*`,
  `/`, prefix `-`, `==`, and `!=` apply to residues of one modulus; the other
  operators are `ORC0215`. `x / y` is x times the inverse of y when y is a
  unit and 0 otherwise. `as` converts among `Int`, words, and residues by
  least residues, and a residue indexes only through `as`.
- **Core and evaluation.** Core records residue types and values exactly,
  with declared names replaced by their types. Each operation costs steps in
  proportion to the 32-bit digits of its modulus: `+` 1 + d, `*` 1 + 2d^2,
  `/` 1 + 64d^2.

## Alternatives

Reducing by hand with `%`, as S3f through S3h require, was the status quo.
It is kept for `Int` and words, and it is what S3i removes for residues: a
missing reduction is invisible until a test vector fails.

Fixing a set of named fields, such as `F25519` or `Fq3329`, was rejected. It
admits only the fields someone foresaw, gives the compiler a table of
constants to trust, and hides the modulus from the reader. A constant
modulus written as its standard writes it covers every field and every ring.

Accepting only prime moduli was rejected. Checking primality of a 521-bit
constant in the compiler is either expensive or probabilistic, and rings
modulo a composite, as `Mod[256]` or `Mod[1 << 32]`, are useful. `Mod[m]` is a ring for every m, and `/` is total
for every m by returning 0 for a non-unit, the convention RFC 7748 and RFC
9380 use for inversion.

Making division by a non-unit an error was rejected. It would make `/` the
only partial operator in Orange 2026, which S3f made total, and every use
would need a proof that its divisor is a unit.

Allowing an order on residues was rejected. The standards compare least
residues, as RFC 8032 does when it checks that a scalar is less than L, and
an order on the ring itself would invite comparisons of values whose
representatives were never meant to be compared. `(x as Int) < (y as Int)` says
what such a check means.

Letting `type` declare a new, distinct type was deferred. It would let a
point coordinate and a scalar of one modulus be told apart, but it needs
rules for conversions and literals that S3i does not need yet. Every S3i
program stays valid if distinct types are added under another keyword.

Parameterizing a function by its modulus was deferred to the static
parameters on the roadmap. It is the natural next step, and S3i's rule that
moduli are equal by value is what such parameters will need.

## Compatibility and migration

Every source that S3h accepts declares no type and writes no `Mod[...]`, so
S3i accepts it with the same Core values, messages, and output bytes. A
source that S3h rejects gets the same diagnostics, except that a `type`
declaration at the head of a module, which was `ORC0103`, and `Mod[m]`, which
was `ORC0203` or `ORC0101`, are now accepted or reported by the S3i rules;
the label listing the admitted types, the labels of `as` diagnostics, the
note of `!` on a non-`Bool`, the notes on arrays, and the nesting-limit
message now name residues or moduli.

The public Rust API gains `Modulus`, `Residue`, `TypeDeclaration`,
`MAX_MODULUS_BITS`, and `MAX_TYPES_PER_MODULE`; `CoreType` and `CoreValue`
gain a residue variant, `TypeSyntax::modulus` returns the modulus of
`Mod[...]`, and `ModuleDeclaration::types` returns a module's
declarations. Functions and values that
never meet a residue behave as before.

Rollback reverts the parser, analyzer, Core, evaluator, tests, fixtures, and
normative documents together.

## Semantic and claim effects

This proposal gives exact meaning to the integers modulo a constant and to
type names. The supported claim remains deterministic, bounded analysis and
evaluation of the documented fragment at a recorded implementation revision.
It establishes no soundness, proof, refinement, compilation, cryptographic
correctness, constant-time behavior, compatibility, independent review, or
production readiness.

## TCB, axiom, and proof effects

The parser, analyzer, Core constructor, evaluator, and command-line interface
remain engineering trust dependencies. The evaluation of constant moduli, the
table of moduli per module, the resolution of type names, and residue
arithmetic, inversion by the extended Euclidean algorithm included, are new
trusted code. Unit tests check the arithmetic against a 128-bit reference for
moduli up to 2^64, reduction and inversion exactly for a prime, a composite,
and the field of 2^255 - 19, the range of moduli up to 2^521 - 1, the display of the standard fields, name resolution and its errors,
event accounting, allocation failure, and spans that do not belong to their
source. No axiom, theorem, proof rule, certificate, checker, or solver is
introduced.

## Threat, abuse, and leakage effects

A modulus is evaluated at analysis time, so a source could try to make the
analyzer compute a large constant. Every value computed for a modulus has at
most 16384 bits, as an `Int` literal does, every shift amount is at most
16384, every part of a modulus costs a semantic event, and each modulus is
evaluated once per module, not once per use. A modulus has at most 521 bits,
so every residue operation during evaluation works on at most 17 digits and
is charged steps in proportion to its work; the budget of 1,048,576 steps is
unchanged. An allocation failure gives no Core and no values.

The reference evaluator is not constant-time: its residue arithmetic takes
time that depends on the values. No secrecy label or leakage property is
defined.

## Target and ABI effects

None. A residue is a value of the specification; its representation in a
compiled program is left to D-011 and later slices.

## Standards, errata, and provenance

RFC 7748 sections 4.1, 5, and 5.2, RFC 8439 sections 2.5, 2.5.1, and
2.5.2, RFC 8032 section 5.1, FIPS 203, FIPS 186-5 and SEC 2 for P-256, and
RFC 9380 for the convention of inversion motivate the slice. The fixtures
check the first X25519 test vector of RFC 7748 section 5.2, the Poly1305 tag
of RFC 8439 section 2.5.2, zeta^128 = -1 and 128^-1 = 3303 modulo 3329 from
FIPS 203, d and the square root of -1 of RFC 8032, and that P-256's
generator lies on its curve. No standard gains normative authority through
this proposal.

## Dependencies, licenses, and IP

No dependency is added.

## Conformance, tests, and evidence

`compiler/crates/orangec/tests/s3i_conformance.rs` binds the 13 rules of the
specification's index to evidence and fails on any drift. Seven programs run
through `orangec check` and `eval` twice each, and generated programs check
64 and 65 type declarations, moduli at 2^521 - 1 and 2^521, residue literals
at the edges of their range, and a residue's least residue as an index at the
edge of its table. Unit tests cover the parser, modulus evaluation and its
events, name resolution, residue literals, operators, conversions, indices,
Core construction, arithmetic against a reference, step costs, residues
across modules, allocation failure, and foreign spans. The S2 through S3h
runners continue to pass unchanged.

## Operations, release, and recovery

No service, package, key, or release is added.

## Support and deprecation

The fragment is pre-alpha and best effort under D-022, with no compatibility
promise.

## Unresolved questions

- Whether functions should take their modulus as a static parameter, so that
  one `spec` serves every field.
- Whether `type` should also declare distinct types, so that coordinates and
  scalars of one modulus cannot be mixed.
- Whether exponentiation by a constant, square roots, or the Legendre symbol
  should be operators or library functions.
- Whether extension fields, such as GF(2^128) of GCM or the fields of
  pairings, should be types.
- Which representation a backend must use for residues, and what it must
  guarantee about the time their operations take.

## Decision record

On 2026-09-28 the owner directed that Orange continue its implementation goals
and that development not freeze unless the owner asks, and on 2026-09-29 that
development of Orange continue. Integers modulo a prime were the next slice on
the roadmap after S3h. This proposal records the S3i surface built under that
direction and is presented for the owner's review. It is not accepted.
Acceptance is the owner's decision alone; until it is recorded here with a
decision date, reviewed revision, and `solo-reviewed` approval record, this
proposal authorizes nothing by itself.
