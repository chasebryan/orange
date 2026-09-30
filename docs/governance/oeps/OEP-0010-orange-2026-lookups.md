---
number: OEP-0010
title: Orange 2026 lookups keyed by data
authors:
  - Chase Bryan
champion: Chase Bryan
status: Review
type: Standards
created: 2026-09-30
updated: 2026-09-30
discussion: owner-direction-2026-09-29-s3g
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
supersedes: []
superseded-by: null
review-authorities:
  - Orange Project Owner
decision-date: null
decision-revision: null
approval-records: []
---

# OEP-0010: Orange 2026 lookups keyed by data

## Abstract

An index may depend on data. An index whose first typed leaf is a word is
checked as that word type, and its range is its type's, narrowed by `&`, `|`,
`^`, `~`, `/`, `%`, shifts by literals, and `+`, `-`, and `*` where they
cannot wrap. An `Int` index may convert words with `as Int` and choose with
conditionals. Every index is still proved in range before anything runs. An
update or fill of n elements costs ⌈n/64⌉ evaluation steps instead of n.

With this slice, AES reads as FIPS 197 writes it:

```orange
spec sub_bytes(s: Word[8]^256, a: Word[8]^16) -> Word[8]^16 {
  for i in 0..16 with b: Word[8]^16 = a { b with [i] = s[a[i]] }
}

spec sub_word(s: Word[8]^256, w: Word[32]) -> Word[32] {
  ((s[w >> 24] as Word[32]) << 24)
    | ((s[(w >> 16) & 0xff] as Word[32]) << 16)
    | ((s[(w >> 8) & 0xff] as Word[32]) << 8)
    | (s[w & 0xff] as Word[32])
}
```

The normative text is [`docs/LOOKUPS_2026.md`](../../LOOKUPS_2026.md). An
implementation, 4 fixtures, and a 10-rule conformance runner accompany it so
that the proposal can be reviewed against running code. This proposal is in
**Review** and requires OEP-0009, which is also in review. It accepts no D-004
candidate and gives the Typed Reference Core no canonical or proof role.

## Motivation

Tables are how much of symmetric cryptography is written. AES's SubBytes is
the S-box of FIPS 197 Figure 7, DES has eight S-boxes, Camellia, ARIA, SM4,
Twofish, and Serpent are specified with tables, and the table-driven CRC and
many hash constructions read one entry per input byte. Through S3f an index
could depend only on literals and loop indices, so each of these had to be
written as a scan of the whole table, 256 comparisons for one lookup, which is
neither how the standards read nor affordable within the step budget.

The rule that made data-dependent indices inexpressible served two purposes:
no index could be out of range at run time, and no lookup keyed by a secret
could appear in the source. S3g keeps the first by proving every index in
range from its type. It gives up the second in the specification stratum,
where it never carried a guarantee: static indices were a property of the
source language, not of any compiled code, and a specification that cannot
state AES as FIPS 197 does is one a reviewer must translate before checking.
Where lookups keyed by secrets must not leak is a question for code
generation, and this proposal leaves it there explicitly.

Updates were also too expensive. S3e charged one step per element for every
update, so a loop that changed one entry of a 256-entry table on each of its
iterations spent most of the per-source budget on copying. A step is meant to
stay about as much work as one limb operation; copying 64 elements is that
much work.

## Scope and non-goals

This proposal defines the type of an index, the ranges of word indices, the
widened static `Int` indices, their Core form, the cost of updates and fills,
and the new messages of `ORC0223` and `ORC0226`. It carries one correction to
S3f: `if` before the word `as`, or `with [`, starts a conditional when the
conditional can complete.

It does not define indices built from unbounded `Int` values, ranges narrowed
by conditions, static index parameters, tables of more than 256 entries,
signed words, or any exclusion of OEP-0009 that it does not lift. It makes no
timing, secrecy, or leakage claim.

### Strata assumption

As for S3b through S3f, S3g assumes only what every D-004 candidate gives the
Specification role: pure, total, deterministic meaning. A lookup proved in
range is a total function of its table and its index, so under `ST-REL`,
`ST-UNI`, `ST-DUAL`, `ST-MIRROR`, and `ST-HOST` alike the source surface needs
no change. What an implementation stratum must do with a lookup whose index
may be secret is D-004's and D-011's question, and section 11 of the
specification records it as open.

## Specification

[`docs/LOOKUPS_2026.md`](../../LOOKUPS_2026.md) is the complete normative
text. In summary:

- **Types.** An index that is not one literal has the type of its first typed
  leaf when that is a word type, and is an `Int` otherwise, in selections and
  updates alike.
- **Word ranges.** A word index ranges over its type. `a & b` is at most
  either operand; `a | b` and `a ^ b` are at most the all-ones value covering
  both; `~a` reflects a's range; `a >> k` and `a << k` with a literal k shift
  it, the latter only without overflow; `+`, `-`, and `*` combine bounds where
  they cannot wrap; `a / b` and `a % b` follow the total rules; a conditional
  joins its values; a conversion from a narrower word keeps its range.
  Anything else ranges over the whole type.
- **`Int` ranges.** A static index may contain `a as Int`, with the range of
  its word or `Int` operand, and conditionals, joined over their values. Any
  other part, such as an `Int` parameter, call, or element, is `ORC0226`.
- **Core.** A word index is followed by one `convert` node to `Int`, so every
  position in Core is an `Int` and the evaluator is unchanged.
- **Limits.** Finding an index's type and range consumes no events; the
  conversion is one node and one step. An update or fill of n elements costs
  ⌈n/64⌉ steps.

## Alternatives

Keeping static indices and writing lookups as scans was rejected. A scan of a
256-entry table costs 256 comparisons per lookup, and it hides the standard's
own description behind an implementation technique. A scan is what a
constant-time backend might emit; it is not the meaning.

Checking indices at run time, with a failure when one leaves its array, was
rejected. Orange has no failing expressions, and a specification whose meaning
includes an out-of-range failure is not total.

Narrowing ranges by conditions, so that `if x < 16 { t[x] } else { 0 }` is
accepted for a table of 16, was deferred. It needs flow-sensitive typing, and
the same effect is written `t[x & 15]` or with a remainder, whose range a
reader can see.

Clamping or reducing an out-of-range index silently, as `t[x % n]` would, was
rejected as an implicit rule. A program that means a remainder writes one.

A dedicated lookup operator, or a table type distinct from arrays, was
considered. It would mark lookups for a later constant-time analysis, but S3f
static indices are already recognizable by their syntax, so every other
index is a lookup by definition, and a second array type would double the
surface for no gain in meaning.

Charging updates one step per element, or a flat one step, was rejected. The
first made table-driven code unaffordable; the second would let a program
copy large arrays without cost. One step per 64 elements keeps a step close
to one limb operation, and never charges more than S3e did.

## Compatibility and migration

Every source that S3f accepts is accepted by S3g with the same Core and output
bytes, because none of its indices is a word index, and its evaluation costs
no more steps. A source that S3f rejects gets the same diagnostics, except
that an index whose first typed leaf is a word, or that converts to `Int` or
contains a conditional, is now accepted or reported with its range, and the
`ORC0226` message, label, and note and the `ORC0223` note are reworded. The
S3e and S3f runners' `ORC0226` message checks, the update and fill rows of the
step-cost unit test, and `LOOPS_2026.md` section 10 are updated accordingly.

The S3f parser correction changes two sources that S3f rejected,
`if as { ... } else { ... }` and `if with[i] { ... } else { ... }` on names
spelled `as` and `with`, into conditionals. `CONDITIONS_2026.md` sections 3
and 12 state the corrected rule.

The public Rust API is unchanged in shape: Core gains no node kind, and a word
index uses the existing `Convert` node.

Rollback reverts the analyzer, evaluator, tests, fixtures, and normative
documents together.

## Semantic and claim effects

This proposal gives exact meaning to indices keyed by data. The supported
claim remains deterministic, bounded analysis and evaluation of the
documented fragment at a recorded implementation revision. It establishes no
soundness, proof, refinement, compilation, cryptographic correctness,
constant-time behavior, compatibility, independent review, or production
readiness.

## TCB, axiom, and proof effects

The parser, analyzer, Core constructor, and evaluator remain engineering trust
dependencies. The word-range rules are new trusted code, computed in 128-bit
integers that cannot overflow for words of at most 64 bits. Unit tests
accept an index for every row of the specification's table and reject
indices whose operators could wrap or whose ranges are too wide, and the
evaluator does not rely on the rules: every selection and update checks its
position again and fails closed. No axiom, theorem, proof rule, certificate,
checker, or solver is introduced.

## Threat, abuse, and leakage effects

The hostile-frontend surface is unchanged in shape. The range computation
walks an index once, bounded by the parser's expression height, and allocates
nothing for words. Cheaper updates let a program perform more updates within
the same budget, still bounded by it.

This is the first slice in which a source may state a lookup keyed by data,
and so a lookup keyed by a secret. No secrecy label or leakage property is
defined, and nothing here claims that any lookup would take the same time on
every input in any compiled code. A later decision on code generation must
treat such lookups explicitly, for example by compiling them to scans of the
whole table or rejecting them under a constant-time profile. Until then, a
reviewer who needs to find lookups can: every index that is not built from
literals and loop indices is one.

## Target and ABI effects

None. A lookup is a selection between values, not a memory access.

## Standards, errata, and provenance

FIPS 197 sections 4.2, 5.1, 5.2, and 5.3 and Appendices B and C.1 motivate the
forms. The AES fixture derives the S-box from section 5.1.1's definition and
checks section 5.1.1's example S-box value, the cipher example of Appendix B,
the AES-128 example of Appendix C.1, and its inverse cipher. The lookups
fixture checks the CRC-32 check value 0xcbf43926 of "123456789". No standard
gains normative authority through this proposal.

## Dependencies, licenses, and IP

No dependency is added.

## Conformance, tests, and evidence

`compiler/crates/orangec/tests/s3g_conformance.rs` binds the 10 rules of the
specification's index to evidence and fails on any drift. Four fixtures and
one generated case run through `orangec check` and `eval` twice each. Unit
tests cover every word-range row, operators that could wrap, the Core conversion
of a word index, selection and update by value, exact event, node, and step
accounting, and the parser correction. The S2 through S3f runners continue to
pass, with the updates described above.

## Operations, release, and recovery

No service, package, key, or release is added.

## Support and deprecation

The fragment is pre-alpha and best effort under D-022, with no compatibility
promise.

## Unresolved questions

- What code generation must do with a lookup whose index may be secret, and
  whether a constant-time profile rejects it or compiles it to a scan.
- Whether ranges should be narrowed by conditions, and at what cost to the
  simplicity of the rule.
- Whether tables of more than 256 entries, or static index parameters, should
  follow.
- Whether the compiler should report which indices depend on data, for
  review.

## Decision record

On 2026-09-28 the owner directed that Orange continue its implementation goals
and that development not freeze unless the owner asks, and on 2026-09-29 that
development of Orange continue. The reversal of the static-index rule was put
to the owner as a decision on 2026-09-29, with bounded lookups recommended,
and work proceeded on that recommendation. This proposal records the S3g
surface built under that direction and is presented for the owner's review.
It is not accepted. Acceptance is the owner's decision alone; until it is
recorded here with a decision date, reviewed revision, and `solo-reviewed`
approval record, this proposal authorizes nothing by itself.
