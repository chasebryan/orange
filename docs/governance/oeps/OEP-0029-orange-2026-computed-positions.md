---
number: OEP-0029
title: Orange 2026 computed positions
authors:
  - Chase Bryan
champion: Chase Bryan
status: Review
type: Standards
created: 2026-10-06
updated: 2026-10-06
discussion: owner-direction-2026-10-06-s3y
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
  - OEP-0021
  - OEP-0023
  - OEP-0024
  - OEP-0025
supersedes: []
superseded-by: null
review-authorities:
  - Orange Project Owner
decision-date: null
decision-revision: null
approval-records: []
---

# OEP-0029: Orange 2026 computed positions

## Abstract

S3y lets a position be **computed from data** and still be proved in range
before the program runs. A remainder by a divisor that is never zero bounds
any `Int`, so `n % 256` lies from 0 through 255 whatever `n` is; a `let` of
one `Int` or word name gives the name its value's range, so a position is
named once and used wherever it selects; and a slice's bounds may name such
bindings, so a window of fixed length slides to a place the data choose.

```orange
// FIPS 203 Algorithm 7, SampleNTT: j counts accepted coefficients.
a with [j % 256] = d1 as Zq

// FIPS 204 Algorithm 29, SampleInBall: i counts placed coefficients.
let at: Int = i % 256;
let moved: Poly = c with [at] = c[j];

// A rotation by an amount from data: a window of a row joined to itself.
let by: Int = (k as Int) % 8;
let twice: Word[8]^16 = r ++ r;
twice[by..by + 8]
```

The normative text is
[`docs/COMPUTED_POSITIONS_2026.md`](../../COMPUTED_POSITIONS_2026.md). An
implementation, five programs, and an 11-rule conformance runner accompany it
so that the proposal can be reviewed against running code. This proposal is
in **Review** and requires OEP-0025, which is also in review. It adds no
syntax, Core node, or evaluator rule, accepts no D-004 candidate, and gives
the Typed Reference Core no canonical or proof role.

## Motivation

Through S3u every `Int` index had to be built from literals, loop indices,
and words converted with `as Int`, and every slice bound from literals and
loop indices. That proves every index in range, but it leaves out the
positions cryptography computes. ML-KEM's SampleNTT and ML-DSA's
SampleInBall write to a position that counts the candidates accepted so far,
which no loop index describes. A rotation by an amount from data reads a
window whose start the data choose. An index named once and used several
times had to be written out each time, because a `let` forgot the range of
its value.

The remainder already had the right meaning for this. S3f defines `%` as
Euclidean, so `e % d` is never negative and is less than the magnitude of
`d` whenever `d` is not zero, whatever `e` is. A position written modulo 256
is therefore in range for a table of 256 whatever it counts, and the checker
can prove that from the remainder alone.

## Scope and non-goals

This proposal defines the range of a remainder whose divisor's range does
not hold zero, the ranges of `let` bindings of one `Int` or word name, the
diagnostics for names without a range, the affine slice bounds over loop
indices, sizes, and ranged bindings, the constant length and the range of
such a window, and the window update.

It does not infer ranges for parameters or accumulators, narrow a range
from a condition, add refinement or range types, bound a quotient of a value
without a range, admit a remainder or quotient written in place in a slice's
bounds, or admit a window whose length depends on data. It adds no token,
reserved word, grammar production, diagnostic code, Core node, command, or
option. It makes no timing, secrecy, or leakage claim.

### Strata assumption

As for S3b through S3u, S3y assumes only what every D-004 candidate gives
the Specification role: pure, total, deterministic meaning. S3y changes what
the checker proves, not what a program means; every admitted program
evaluates with S3u's rules. S3y therefore has the same meaning under
`ST-REL`, `ST-UNI`, `ST-DUAL`, `ST-MIRROR`, and `ST-HOST`. Under a stratum
that separates specification from implementation, a position that depends on
a secret is what an implementation must be shown to access without leaking,
as section 9 of the specification records.

## Specification

[`docs/COMPUTED_POSITIONS_2026.md`](../../COMPUTED_POSITIONS_2026.md) is the
complete normative text. In summary:

- **Remainders.** `e % d` lies from 0 through one less than the greatest
  magnitude of `d` whenever the range of `d` excludes zero, whatever `e` is.
  A divisor that may be zero bounds nothing, because `x % 0` is `x`.
- **Ranged bindings.** A `let` of one `Int` or word name gives the name its
  value's range, in bodies, loop steps, and branches, and through chains of
  bindings. An index built from it is proved in range for every value or
  rejected.
- **Names without a range.** Parameters, accumulators, names of tuple
  patterns, and bindings of values without a range have none. The
  diagnostic points at the binding with a secondary label that says why.
- **Windows.** Slice bounds are affine over literals, loop indices, sizes,
  and ranged bindings, with `*` by a constant. A one-valued binding is its
  value, a product of two variables is rejected, and a position computed in
  place must be named first, with a hint that says so.
- **Length and range.** A window's length is the same for every value of its
  loop indices and bindings, and each kind of changing length has its own
  message. Both ends lie in the array for every value, or the window is
  rejected. Window updates follow the same rules.
- **Meaning and cost.** Unchanged. A ranged name selects exactly what its
  value written in place selects, at S3u's cost.

## Alternatives

Narrowing ranges from conditions, so that `if n < 16 { t[n] }` checks, was
rejected for this slice: it makes a range depend on the path to an
expression, which every later analysis and backend would have to follow,
while a remainder states the bound in the expression that selects.

Range or refinement types on parameters, such as `n: Int in 0..256`, were
rejected for this slice: they add syntax, a new obligation at every call,
and a choice of where the obligation is checked, all of which a remainder
and a binding already express inside the function that needs them.

Substituting a binding's value into a slice's bounds, so that `x[k..j]` with
`let j: Int = k + 2` has length 2, was rejected: it makes a window's length
depend on how far the checker follows names, and a reader comparing two
bounds must then do the same. Naming the start and writing both bounds from
it, as `x[at..at + 4]`, keeps the length visible.

Checking positions at run time, as most languages do, was rejected: Orange
proves every index in range when the program is checked, and a position
that may fail at run time would give selection a failure no other operation
has.

## Compatibility and migration

Every source that S3u accepts is accepted with the same types, Core, values,
output bytes, and steps. S3y admits indices and windows that S3u rejected.
A source that S3u rejects gets the same diagnostic codes, except where S3y
now admits it; the message, label, and note of `ORC0226` for an index and
for a slice bound, the notes of `ORC0223` for an index and for a slice, and
S3l's note "a slice's position never depends on data", now "a slice's
length never depends on data", change as section 11 of the specification
lists. A rejected source may also carry the new secondary label or the hint
of sections 3 and 4.

The public Rust API is unchanged. `orangec` gains no command or option, its
version line names S3y as the implemented slice, and `orangec lex` is
unchanged.

Rollback reverts the analyzer's binding ranges, remainder rule, and window
forms, the fixtures, the conformance runner, and the normative documents
together; newly admitted sources then regain their earlier rejection.

## Semantic and claim effects

This proposal enlarges the set of programs the checker proves free of
out-of-range selection. The supported claim remains deterministic, bounded
analysis and evaluation of the documented fragment at a recorded
implementation revision. It establishes no soundness, proof, refinement,
compilation, cryptographic correctness, constant-time behavior,
compatibility, independent review, or production readiness.

## TCB, axiom, and proof effects

The analyzer remains an engineering trust dependency, and its range rules
are now trusted for more programs. A wrong range would admit a selection
that the evaluator still checks and refuses as inconsistent Core, `ORC0301`,
with no partial output; it would not select a wrong element. Unit tests
check the remainder rule for divisors of both signs, ranged bindings in
bodies, loop steps, and branches, names without a range and their secondary
labels, and every window form and message. The conformance
runner compares remainders with Rust's `rem_euclid` for dividends up to
2^127 − 1 in magnitude, compares every binding with its value written in
place, and slides windows to every place they fit. No axiom, theorem, proof
rule, certificate, checker, or solver is introduced.

## Threat, abuse, and leakage effects

Range analysis of a binding costs the analysis of its value, once, and a
ranged name costs one lookup wherever it appears. Ranges are the analyzer's
existing exact integers under its existing reservation limits. No evaluator
cost changes.

Before S3y an `Int` position could depend on data only through a word
converted with `as Int`, which its spelling showed. Since S3y a name can
carry data into an index or a window's start, so whether a position depends
on data is found by following names to their values. A selection at a
position computed from a secret is the access pattern cache-timing attacks
observe. The reference evaluator is not constant-time. A later D-004 and
D-011 decision must say what code generation does with such a position, as
`LOOKUPS_2026.md` section 11 already requires of lookups.

## Target and ABI effects

None.

## Standards, errata, and provenance

The SampleNTT fixture follows FIPS 203 (August 2024) Algorithm 7 and runs it
on the first 504 bytes of SHAKE128 outputs for a fixed seed. The
SampleInBall fixture follows FIPS 204 (August 2024) Algorithm 29 with
τ = 39 on the first 136 bytes of a SHAKE256 output. The streams and
expected results come from Python's `hashlib` and an independent
transcription of each algorithm, not from published test vectors, and the
fixtures test positions and selection, not ML-KEM or ML-DSA conformance. No
standard gains normative authority through this proposal.

## Dependencies, licenses, and IP

No dependency is added.

## Conformance, tests, and evidence

`compiler/crates/orangec/tests/s3y_conformance.rs` binds the 11 rules of the
specification's index to evidence and fails on any drift. Five programs run
through `orangec check`, `test`, and `eval` twice each. Generated tests
compare remainders by divisors of both signs and from data with Rust's
Euclidean remainder and show each bound is tight, compare every ranged
binding with its value written in place, point at the binding of every name
without a range, slide windows of several lengths to every place they fit
and show one more place is refused, check every window-length message, and
keep the unchanged boundaries refused. The S2 through S3u runners, the
`orangec enc` tests, and the algorithms corpus continue to pass.

## Operations, release, and recovery

No service, package, key, or release is added.

## Support and deprecation

The fragment is pre-alpha and best effort under D-022, with no compatibility
promise.

## Unresolved questions

- Whether a report of the positions that may depend on data, per function,
  should accompany `orangec check`, so that a reviewer sees every one
  without following names by hand.
- Whether conditions should narrow ranges in a later slice, and how a
  backend would then follow them.
- Whether a constant-time profile should refuse, or require a particular
  lowering of, selections at positions derived from secrets.

## Decision record

On 2026-09-28 the owner directed that Orange continue its implementation goals
and that development not freeze unless the owner asks, and on 2026-09-29 that
development of Orange continue, with the vision of a technical and beautiful
language for cryptographers, cryptologists, and cryptanalysts. On 2026-10-06
the owner chose to build positions computed from data as S3y. This proposal
records the S3y surface built under that direction and is presented for the
owner's review. It is not accepted. Acceptance is the owner's decision alone;
until it is recorded here with a decision date, reviewed revision, and
`solo-reviewed` approval record, this proposal authorizes nothing by itself.
