---
number: OEP-0014
title: Orange 2026 tuples
authors:
  - Chase Bryan
champion: Chase Bryan
status: Review
type: Standards
created: 2026-09-30
updated: 2026-09-30
discussion: owner-direction-2026-09-30-s3k
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
supersedes: []
superseded-by: null
review-authorities:
  - Orange Project Owner
decision-date: null
decision-revision: null
approval-records: []
---

# OEP-0014: Orange 2026 tuples

## Abstract

A tuple is a value of two through 16 elements of possibly different types,
written `(a, b)`, of a tuple type written `(T, U)`. `.k` selects element k,
counted from zero. A tuple pattern names each element where a `let` binding
or a loop's accumulator is declared, so that a function can give several
values and a loop can carry several accumulators:

```orange
spec add256(x: Limb^4, y: Limb^4) -> (Limb^4, Limb) {
  for i in 0..4 with (sum: Limb^4, carry: Limb) = ([0; 4], 0) {
    let (limb: Limb, out: Limb) = add_carry(x[i], y[i], carry);
    (sum with [i] = limb, out)
  }
}
```

A tuple's elements are scalars and arrays; a tuple holds no tuple, and an
array holds no tuple. No operator applies to a whole tuple.

The normative text is [`docs/TUPLES_2026.md`](../../TUPLES_2026.md). An
implementation, seven programs, and an 8-rule conformance runner accompany it
so that the proposal can be reviewed against running code. This proposal is
in **Review** and requires OEP-0013, which is also in review. It accepts no
D-004 candidate and gives the Typed Reference Core no canonical or proof
role.

## Motivation

A standard's round keeps several values at once. FIPS 180-4 section 6.2.2
carries the working variables a through h of SHA-256 from round to round. RFC
8439 section 2.1 writes the ChaCha20 quarter round as a function of four
words that gives four words. NIST SP 800-232 writes Ascon's state as five
64-bit words x0 through x4. A multi-precision addition gives a sum and a
carry, and the extended Euclidean algorithm carries three pairs of values.

Through S3j a loop had one accumulator and a function one result. Such state
had to be packed into an array and read back as `v[0]` through `v[7]`, which
hides the standard's names and forces every value of the state to one type,
or a function that computes two values had to be written twice. The
encryption lane asked for loops with more than one accumulator for this
reason: Ascon would otherwise need a record of 35 words.

With S3k the SHA-256 fixture carries a through h as the loop's eight named
accumulators, the ChaCha20 fixture writes the quarter round as RFC 8439 does
and names all sixteen words of the state through each double round, and the
Ascon-Hash256 fixture carries its state as five named words. All three
reproduce their published values.

## Scope and non-goals

This proposal defines tuple types, tuples, the selection of an element by its
position, and tuple patterns in bindings and in loops' accumulators: their
syntax and limits, the scope and uniqueness of a pattern's names, typing, the
meaning of each form, their record in the Typed Reference Core, and the
evaluator's step costs. It adds one diagnostic code, `ORC0234` for a
selection by position from a value that is not a tuple, and no token or
reserved word.

It does not define tuples of tuples, arrays of tuples, tuples of one element
or none, equality, order, or any other operator on whole tuples, an update of
one element of a tuple, nested or partial patterns, patterns in parameters,
selection from an expression other than a name or a call, records with named
fields, or inference of a pattern's types. It makes no timing, secrecy, or
leakage claim and fixes no layout or ABI.

### Strata assumption

As for S3b through S3j, S3k assumes only what every D-004 candidate gives the
Specification role: pure, total, deterministic meaning. A tuple is a finite
product of values, `.k` is a projection, and a pattern names the projections
of its value, so each has the same meaning under `ST-REL`, `ST-UNI`,
`ST-DUAL`, `ST-MIRROR`, and `ST-HOST`, and the source surface needs no
change.

## Specification

[`docs/TUPLES_2026.md`](../../TUPLES_2026.md) is the complete normative
text. In summary:

- **Syntax.** `(T0, T1, ...)` is a tuple type and `(e0, e1, ...)` a tuple,
  each of two through 16 parts with an optional trailing comma; `(e)` stays a
  group. `.k` follows a name or a call, with k in decimal, and may be followed
  by one index. A binding or an accumulator declares either one typed name or
  a tuple pattern `(a: T, b: U, ...)`. `let` is recognized by position. More
  than 16 parts is `ORC0106`; every malformed form is `ORC0101` with a note
  that shows the form.
- **Scope.** A pattern's names are the kind of name they stand in for, a
  binding's or an accumulator's, with the same scope. Each is unique among the
  names in scope and the pattern's other names (`ORC0219`).
- **Typing.** A tuple is checked against a tuple type of as many elements. A
  tuple's element is `Int`, `Bool`, a word, a residue, or an array of one; a
  tuple of tuples and an array of tuples are `ORC0203`. `.k` selects an
  element that exists (`ORC0223`) of a tuple (`ORC0234`). No arithmetic,
  comparison, conversion, index, or update applies to a whole tuple. As for a
  binding of one name, a pattern whose type does not resolve is reported once
  and its names are not reported again.
- **Meaning.** A tuple's elements are evaluated left to right; `.k` is
  element k; a pattern names each element of its value in order. A loop whose
  accumulator is a pattern carries one tuple from step to step.
- **Core and evaluation.** Core gains a tuple type and `tuple` and `project`
  nodes. A pattern is one binding, local, or accumulator of tuple type, and a
  read of one of its names is the read of the whole followed by `project`. A
  tuple of n elements costs its elements' steps and n, and `.k` one step.

## Alternatives

Several accumulators without tuples, as `with a: T = x, b: U = y`, were
considered first. They would serve loops alone, and the loop's value would
still need a type: either a tuple, which this proposal adds anyway, or only
one of the accumulators, which loses the others. A function that gives a sum
and a carry needs the same thing. Tuples answer both with one construct.

Records with named fields would let `s.x0` name an element. They need
declarations, field namespaces, and rules for field names across modules.
Patterns give the names where they are used, as a standard's text does, and
the positions of `.k` match the order in which standards list their values.
Records remain possible later, and every S3k program would stay valid.

Nested tuples and arrays of tuples were deferred. They are not needed by the
algorithms in hand, they make the analysis of an index and the display of
values recursive, and they raise the question of a tuple's layout in an array
before D-011 chooses a target.

Equality of whole tuples was deferred with the other operators. A comparison
of all elements at once hides which element differs and invites comparisons
of secret state that a backend would have to compile element by element. An
elementwise comparison is short to write.

Tuple patterns without types, with the types inferred from the value, were
rejected for the reason S3c gives for bindings: every name states its type
where it is introduced.

## Compatibility and migration

Every source that S3j accepts has no tuple type, tuple, projection, or
pattern, so S3k accepts it with the same Core values, messages, and output
bytes. A source that S3j rejects gets the same diagnostics, except that the
forms S3k defines, which were `ORC0101`, are now accepted or reported by the
S3k rules, and a comparison whose first operand is an array written out, as
`[1, 2] == x`, is now `ORC0215` "`==` is not defined for an array" at the
operator, where it was `ORC0214` at the array.

The public Rust API gains `TupleType`, `CoreTuple`, and `MAX_TUPLE_ELEMENTS`
from the crate root; `CoreType::Tuple`, `CoreValue::Tuple`, and the
`CoreNodeKind` variants `Tuple` and `Project`; `CoreType::as_tuple`; and in
the syntax tree `Pattern`, `TuplePattern`, `TypedName`, `TupleExpression`,
`ProjectExpression`, the `ExpressionKind` variants `Tuple` and `Project`, and
`TypeSyntax::elements` and `is_tuple`. Because a tuple type holds its element
list, `CoreType` is no longer `Copy`: its query methods take `&self`, the
`ty` and `result_type` accessors are no longer `const`, and `ArrayType::new`
takes `&CoreType`. `Binding::name` and `ty` give way to `Binding::pattern`,
and `LoopExpression::accumulator` returns a `Pattern` in place of the name
and type accessors. Code that matches every variant of these enums must
handle the new ones.

Rollback reverts the parser, analyzer, Core, evaluator, tests, fixtures, and
normative documents together.

## Semantic and claim effects

This proposal gives exact meaning to tuples, the selection of their elements,
and patterns. The supported claim remains deterministic, bounded analysis
and evaluation of the documented fragment at a recorded implementation
revision. It establishes no soundness, proof, refinement, compilation,
cryptographic correctness, constant-time behavior, compatibility,
independent review, or production readiness.

## TCB, axiom, and proof effects

The parser, analyzer, Core constructor, evaluator, and command-line interface
remain engineering trust dependencies. The tuple and pattern parser and its
recovery, the analysis of tuple types, tuples, projections, and pattern
names, the Core tuple type and nodes, and the evaluator's tuple values are
new trusted code. The analyzer was split into five files so that no source
file exceeds the repository's size cap; the split moved code without
changing it. Unit tests check exact spans and messages, limits and heights,
scope and uniqueness, checking order, node and event accounting, evaluation
and step costs, the deepest sources the limits admit on a 1 MiB stack, eight
kinds of inconsistent Core, allocation failures, and spans that do not belong
to their source. No axiom, theorem, proof rule, certificate, checker, or
solver is introduced.

## Threat, abuse, and leakage effects

A tuple holds at most 16 elements and no tuple, so a tuple's size is bounded
by 16 arrays of at most 256 elements, and the display and checks of a tuple
recurse one level at most. Tuples nest within the existing 64-level limit
and height bound, and each tuple, projection, and pattern costs semantic
events, Core nodes, and steps, so the analyzer's and evaluator's memory and
native stack stay bounded; the deepest admitted sources run within 1 MiB of
stack. The step budget of 1,048,576 is unchanged. Core whose tuples are
inconsistent with their types stops evaluation with no values, and an
allocation failure gives no Core or no values.

The reference evaluator is not constant-time, and no secrecy label or leakage
property is defined.

## Target and ABI effects

None. A tuple is a value of the specification; how a compiled program lays
it out or passes it is left to D-011 and later slices.

## Standards, errata, and provenance

FIPS 180-4 section 6.2.2, RFC 8439 sections 2.1 and 2.3, and NIST SP 800-232
sections 3 and 5 motivate the slice. The fixtures check the SHA-256 digests
of "abc" and of the 448-bit message of FIPS 180-4's examples, the quarter
round vector of RFC 8439 section 2.1.1 and the block of section 2.3.2, and
entries 1, 2, and 9 of the Ascon designers' known-answer file
`LWC_HASH_KAT_128_256.txt` for Ascon-Hash256. No standard gains normative
authority through this proposal.

## Dependencies, licenses, and IP

No dependency is added.

## Conformance, tests, and evidence

`compiler/crates/orangec/tests/s3k_conformance.rs` binds the 8 rules of the
specification's index to evidence and fails on any drift. Seven programs run
through `orangec check` and `eval` twice each, and generated programs check
tuple types, tuples, and patterns of 16 parts and of 17. Unit tests cover the
parser, scope and names, typing, Core construction, evaluation and its step
costs, the deepest admitted sources, inconsistent Core, allocation failure,
and foreign spans. The S2 through S3j runners and the algorithms corpus
continue to pass unchanged.

## Operations, release, and recovery

No service, package, key, or release is added.

## Support and deprecation

The fragment is pre-alpha and best effort under D-022, with no compatibility
promise.

## Unresolved questions

- Whether tuples should nest, or arrays hold tuples, once a target's layout
  is chosen.
- Whether whole tuples should have equality, and whether a pattern may skip
  an element with a wildcard.
- Whether records with named fields should be added beside tuples.

## Decision record

On 2026-09-28 the owner directed that Orange continue its implementation goals
and that development not freeze unless the owner asks, and on 2026-09-29 that
development of Orange continue. Loops with more than one accumulator were a
language request of the encryption lane and pending after S3j. This proposal
records the S3k surface built under that direction and is presented for the
owner's review. It is not accepted. Acceptance is the owner's decision alone;
until it is recorded here with a decision date, reviewed revision, and
`solo-reviewed` approval record, this proposal authorizes nothing by itself.
