---
number: OEP-0021
title: Orange 2026 computed shift and rotation amounts
authors:
  - Chase Bryan
champion: Chase Bryan
status: Review
type: Standards
created: 2026-09-30
updated: 2026-09-30
discussion: owner-direction-2026-09-30-s3r
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
  - OEP-0012
  - OEP-0013
  - OEP-0014
  - OEP-0015
  - OEP-0016
  - OEP-0017
  - OEP-0018
  - OEP-0019
  - OEP-0020
supersedes: []
superseded-by: null
review-authorities:
  - Orange Project Owner
decision-date: null
decision-revision: null
approval-records: []
---

# OEP-0021: Orange 2026 computed shift and rotation amounts

## Abstract

The amount of a shift or rotation may be **computed**: any expression of
type `Int` or a word, such as `x <<< r` or `x >> (i % 8)`, with a meaning at
every amount. A shift is multiplication or division by a power of two kept to
the word, so a shift by the width or more gives 0 and a negative amount
shifts the other way; a rotation turns by its amount modulo the width. An
amount written as one integer literal keeps S3b's rule, from 0 through n − 1.

```orange
// RC6 key schedule: B = L[j] = (L[j] + A + B) <<< (A + B).
let b1: Word[32] = (l[k % 4] + a1 + b) <<< (a1 + b);

// FIPS 202 Algorithm 2, rho: turn lane (x, y) by (t + 1)(t + 2)/2.
a[(x as Int) + 5 * (y as Int)] <<< (((t + 1) * (t + 2)) / 2)
```

The normative text is [`docs/AMOUNTS_2026.md`](../../AMOUNTS_2026.md). An
implementation, six programs, and a 10-rule conformance runner accompany it
so that the proposal can be reviewed against running code. This proposal is
in **Review** and requires OEP-0020, which is also in review. It accepts no
D-004 candidate and gives the Typed Reference Core no canonical or proof
role.

## Motivation

Cryptography turns words by amounts it computes. RC5 and RC6 are built on
rotations by amounts their data choose. SHA-3's rho step turns each lane by
(t + 1)(t + 2)/2 bits modulo the lane size, and its round constants place
bits at positions 2^j − 1. A Montgomery ladder reads bit t of its scalar,
reflected CRCs and GHASH read the bits of a byte in the opposite order, and
ML-KEM orders its transform's constants by reversing the bits of an index.
Through S3q every amount was a literal, so each of these was a table or a
chain of conditionals where the standard prints one line, and a reader had
to check the table against the line.

A computed amount also needs a meaning at amounts past the width, which
machines disagree about: C leaves them undefined, and Java and x86 reduce a
32-bit shift's amount modulo 32, so there `x << 32` is `x`. A language for
specifications cannot inherit either.

## Scope and non-goals

This proposal defines literal and computed amounts, the type of a computed
amount, the value of a shift or rotation at every amount, its index range,
its Core node, and its cost. It adds no token, reserved word, grammar
production, diagnostic code, or command; `ORC0216` keeps its message for
literal amounts with a new label and note.

It does not define shifts of `Int`, residues, arrays, or tuples, arithmetic
shifts, funnel shifts, bit-field extraction, amounts of type `Mod[m]` without
`as Int`, or any change to the literal amount's rule. It makes no timing,
secrecy, or leakage claim.

### Strata assumption

As for S3b through S3q, S3r assumes only what every D-004 candidate gives the
Specification role: pure, total, deterministic meaning. A computed amount is
an ordinary expression, and a shift or rotation is a total function of a
word and an integer. S3r therefore has the same meaning under `ST-REL`,
`ST-UNI`, `ST-DUAL`, `ST-MIRROR`, and `ST-HOST`, and the source surface needs
no change. Under a stratum that separates specification from implementation,
the meaning at every amount is what an implementation's shift must be shown
to compute.

## Specification

[`docs/AMOUNTS_2026.md`](../../AMOUNTS_2026.md) is the complete normative
text. In summary:

- **Literal amounts.** An amount written as one integer literal, with or
  without a sign, is a literal amount: unsigned and from 0 through n − 1, or
  `ORC0216` at the amount. Any other amount, a group included, is computed.
- **Types.** A computed amount has the type of its first typed leaf when that
  is a word, and `Int` otherwise, and is checked as that type; its width is
  independent of the shifted word's.
- **Meaning.** For a word a of n bits, `a << k` is floor(a · 2^k) modulo 2^n
  and `a >> k` is floor(a · 2^−k) modulo 2^n; `a <<< k` turns left by k mod n
  and `a >>> k` turns right. Below the width each agrees with the literal.
- **Ranges.** As an index, a shift by a computed amount ranges over its whole
  type.
- **Core and cost.** One `shift-by` node recording its operator and amount
  type, costing one step whatever the amount's size.

## Alternatives

Masking the amount to the low bits of the width, as Java and x86 do, was
rejected: it makes `x << 32` on a 32-bit word equal to `x`, a value no
reading of "shift left by 32" gives, and it would make a shift by n differ
from two shifts by n/2.

Leaving amounts past the width undefined, as C does, was rejected: Orange
has no undefined behavior, and a specification whose value depends on the
machine is not a specification.

Refusing computed amounts past the width at run time was rejected: it would
give shifts a run-time failure, which no other word operation has, and every
amount has an obvious mathematical value.

Admitting any expression, a literal past the width included, was rejected: a
literal names a fixed bit position, and `x << 32` written out is far more
often a slip than a wish for 0. A group, `x << (32)`, says the amount is
computed.

A separate operator for computed amounts was rejected: the standards write
one operator for both, and the literal case is the computed case restricted.

## Compatibility and migration

Every source that S3q accepts writes each amount as one unsigned integer
literal below the width, and is accepted with the same Core values, output
bytes, and steps. A source that S3q rejects gets the same diagnostics, except
that an amount that is not one integer literal was `ORC0216` and is now
checked as a computed amount, and `ORC0216` for a literal amount has the label
"a literal amount is from 0 through 31" and the note "an amount written as
one integer literal is from 0 through n - 1; any other amount is computed, an
`Int` or a word, such as `x <<< r` or `x >> (i % 8)`" in place of "amount
must be an unsigned integer literal" and "amounts are fixed literals;
variable amounts are not part of Orange 2026".

The public Rust API gains `CoreNodeKind::ShiftBy`. `orangec` gains no command
or option, and `orangec lex` is unchanged.

Rollback reverts the analyzer, Core, evaluator, tests, fixtures, and
normative documents together.

## Semantic and claim effects

This proposal gives shifts and rotations a total meaning over every integer
amount. The supported claim remains deterministic, bounded analysis and
evaluation of the documented fragment at a recorded implementation revision.
It establishes no soundness, proof, refinement, compilation, cryptographic
correctness, constant-time behavior, compatibility, independent review, or
production readiness.

## TCB, axiom, and proof effects

The analyzer, Core constructor, and evaluator remain engineering trust
dependencies. The computed amount's typing, the `shift-by` node, and its
evaluation are new trusted code. Unit tests compare every width's shifts and
rotations against their definition for `Int` amounts from −(2n + 1) through
2n + 1, amounts around 2^64, 2^200, and 2^16383 of both signs, and every byte
amount, and check typing, Core, cost, and inconsistent Core. No axiom,
theorem, proof rule, certificate, checker, or solver is introduced.

## Threat, abuse, and leakage effects

A computed amount costs its expression's events and nodes. A `shift-by` node
costs one step, allocates nothing, and reads of an `Int` amount only its
sign, whether it exceeds 64 bits, and its low 64 bits, so an amount of 16,384
bits is read in bounded work. Inconsistent Core is refused with `ORC0301`.

A shift or rotation by a secret amount is a timing concern on processors
whose shifter takes time that grows with the amount, and data-dependent
rotations were an early target of timing analysis. The reference evaluator
is not constant-time. A backend must decide what such a shift becomes, for
example a fixed sequence of conditional rotations by powers of two, or a
refusal under a constant-time profile, as it must for S3g's secret indices.

## Target and ABI effects

None. A backend must compute Orange's value at every amount, which a machine
shift that reduces its amount gives only below the width.

## Standards, errata, and provenance

The RC6 fixture follows "The RC6 Block Cipher" (Rivest, Robshaw, Sidney, and
Yin, version 1.1, 1998) and reproduces its two 128-bit-key test vectors both
ways. The SHA3-256 fixture follows FIPS 202 (August 2015) and reproduces
NIST's examples for "abc" and the 448-bit message, and a 133-byte message
checked against an independent implementation. The zetas fixture derives
ML-KEM's constants as FIPS 203 (August 2024) section 4.3 defines them and
reproduces Appendix A. No standard gains normative authority through this
proposal.

## Dependencies, licenses, and IP

No dependency is added.

## Conformance, tests, and evidence

`compiler/crates/orangec/tests/s3r_conformance.rs` binds the 10 rules of the
specification's index to evidence and fails on any drift. Six programs run
through `orangec check`, `test`, and `eval` twice each, and generated tests
compare every operator at every width against a reference for `Int` amounts
of every sign and size and word amounts of every width, show that amounts of
2 through 16,384 bits cost the same steps, and refuse every literal amount
at the width or with a sign. The S2 through S3q runners, the `orangec enc`
tests, and the algorithms corpus continue to pass.

## Operations, release, and recovery

No service, package, key, or release is added.

## Support and deprecation

The fragment is pre-alpha and best effort under D-022, with no compatibility
promise.

## Unresolved questions

- Whether a constant-time profile should refuse, or require a particular
  lowering of, shifts and rotations by amounts derived from secrets.
- Whether shifts of `Int`, with the meaning floor(a · 2^k), should follow,
  so that bit arithmetic on large integers is written the same way.
- Whether index ranges should follow a computed shift whose amount's range
  is known, so that `t[x >> (4 * i)]` for a byte x and a loop index i in
  `1..2` fits a table of 16 without a mask.

## Decision record

On 2026-09-28 the owner directed that Orange continue its implementation goals
and that development not freeze unless the owner asks, and on 2026-09-29 that
development of Orange continue, with the vision of a technical and beautiful
language for cryptographers, cryptologists, and cryptanalysts. Shift and
rotation amounts computed from data, which RC5, RC6, SHA-3, and bit-by-bit
algorithms need, were among the absences The Orange Book listed after S3q.
This proposal records the S3r surface built under that direction and is
presented for the owner's review. It is not
accepted. Acceptance is the owner's decision alone; until it is recorded here
with a decision date, reviewed revision, and `solo-reviewed` approval record,
this proposal authorizes nothing by itself.
