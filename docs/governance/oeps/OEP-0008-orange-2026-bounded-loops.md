---
number: OEP-0008
title: Orange 2026 bounded loops, static indices, and updates
authors:
  - Chase Bryan
champion: Chase Bryan
status: Review
type: Standards
created: 2026-09-28
updated: 2026-09-28
discussion: owner-direction-2026-09-28-s3e
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
  - OEP-0005
  - OEP-0006
  - OEP-0007
supersedes: []
superseded-by: null
review-authorities:
  - Orange Project Owner
decision-date: null
decision-revision: null
approval-records: []
---

# OEP-0008: Orange 2026 bounded loops, static indices, and updates

## Abstract

A loop `for i in a..b with s: T = e { f }` folds the step f over the literal
range a..b, starting from e, and has the value of s after the last step. An
index may be an expression over integer literals and loop indices, which the
analyzer proves in range for every value of every loop index before anything
runs. `x with [i] = v` is the array x with element i replaced, and `[e; n]` is
n copies of e.

With this slice, SHA-256 reads as FIPS 180-4 writes it:

```orange
spec compress(h: Word[32]^8, m: Word[32]^16) -> Word[32]^8 {
  let w: Word[32]^64 = schedule(m);
  let k: Word[32]^64 = round_constants();
  let v: Word[32]^8 = for t in 0..64 with v: Word[32]^8 = h { round(v, k[t], w[t]) };
  for i in 0..8 with out: Word[32]^8 = v { out with [i] = out[i] + h[i] }
}
```

The normative text is [`docs/LOOPS_2026.md`](../../LOOPS_2026.md). An
implementation, 7 fixtures, and an 18-rule conformance runner accompany it so
that the proposal can be reviewed against running code. This proposal is in
**Review** and requires OEP-0007, which is also in review. It accepts no D-004
candidate and gives the Typed Reference Core no canonical or proof role.

## Motivation

S3d can hold a cipher's state, but it cannot iterate over it. Its ChaCha20
fixture writes the ten double rounds as ten bindings and its SHA-256 fixture
stops after two of the 64 rounds, because each round is another line of
source. Every standard in scope is written as iteration: FIPS 180-4 prepares
the schedule "for t = 16 to 63" and runs 64 rounds, RFC 8439 runs ten double
rounds and serializes sixteen words, AES runs ten to fourteen rounds, and
Keccak runs 24. A specification language for cryptography needs the loop the
standard writes, and it needs that loop to be as easy to check as the
standard's own text.

Two properties make that possible. The range of a loop is written in its
source, so its number of steps is visible. And an index into a state is
proved in range for every step when the program is checked, so no reader has
to reason about a run-time failure that Orange cannot express.

## Scope and non-goals

This proposal defines loops over literal ranges from 0 through 65536, indices
built from integer literals and loop indices with `+`, `-`, and `*`, functional
updates of one element, fill literals, their Core form, their evaluation,
diagnostics `ORC0225` and `ORC0226`, and widened uses of `ORC0214`, `ORC0219`,
and `ORC0221` through `ORC0224`.

It does not define loops over computed or unbounded ranges, early exit,
indices that depend on data, slices, arrays of arrays, mutation, or any
exclusion of OEP-0007 that it does not lift.

### Strata assumption

As for S3b through S3d, S3e assumes only what every D-004 candidate gives the
Specification role: pure, total, deterministic meaning. A loop over a literal
range is a finite composition of its step, an index proved in range is a
projection, and an update is a tuple with one component replaced, so under
`ST-REL`, `ST-UNI`, `ST-DUAL`, `ST-MIRROR`, and `ST-HOST` alike the source
surface needs no change. Placement of the Core remains D-004's decision.

## Specification

[`docs/LOOPS_2026.md`](../../LOOPS_2026.md) is the complete normative text. In
summary:

- **Grammar.** `loop = "for" IDENTIFIER "in" INTEGER ".." INTEGER "with"
  IDENTIFIER ":" declared_type "=" expression "{" expression "}"`,
  `index = "[" INTEGER "]" | "[" expression "]"`,
  `update = operand "with" "[" expression "]" "=" expression`, and
  `fill = "[" expression ";" INTEGER "]"`. `for`, `in`, and `with` are
  recognized by position, so no word is reserved. An update that is an operand
  is parenthesized (`ORC0108` otherwise).
- **Loops.** Bounds satisfy 0 ≤ a < b ≤ 65536 (`ORC0225`). The index and the
  accumulator are new names (`ORC0219`), seen only in the step. The loop's type
  is its accumulator's, and must be the type required where it stands
  (`ORC0214`).
- **Indices.** A non-literal index is static, built from literals and loop
  indices (`ORC0226` otherwise), and its range by interval arithmetic lies
  within the array (`ORC0223` otherwise).
- **Updates and fills.** An update applies to an array (`ORC0224`) of the
  required type (`ORC0214`), with a static index and a value of the element
  type. A fill literal states its type's length (`ORC0221`, `ORC0222`).
- **Meaning.** A loop evaluates its first value once, then its step for each
  index in increasing order. An update copies its base with one element
  replaced. A fill repeats one value.
- **Core.** Each function gains a loop table, and Core gains `fold`,
  `loop_index`, `accumulator`, `select`, `update`, and `fill` nodes. A
  function without the new forms has exactly its S3d Core.
- **Limits.** Loops, updates, and expression indices share the 64-level
  nesting budget. A loop is one semantic event plus its bounds' decoding
  events and two name checks; a loop costs one evaluation step plus one per
  iteration; an update or fill of n elements costs n steps. The step budget
  bounds nested loops. The deepest admitted sources still fit in 1 MiB of
  stack.

Amended 2026-10-09: the evaluator now charges ceil(n / 64) steps per update
or fill (CTL-028); see `bulk_cost` in
`compiler/crates/orange-compiler/src/eval.rs`.

## Alternatives

A general `while` loop was rejected. Its number of steps is not visible, it
can fail to terminate, and no standard in scope needs it: every iteration in
FIPS 180-4, RFC 8439, FIPS 197, and FIPS 202 has a fixed count.

Recursion was rejected as the way to iterate. It is already excluded by the
call-cycle rule of S3b, and a fold states the count and the state directly
where recursion would hide them in a base case.

Checking indices at run time was rejected. A run-time failure is a value
Orange does not have, and a static proof keeps every access visible when the
program is checked. Interval arithmetic is the simplest proof that covers
every index the fixtures need, such as `w[t - 15]` over 16..64 and
`b[4 * i + j]` over 0..16 and 0..4; it can reject an index that is always in
range, such as `x[i - i]`, and the error names the computed range so the
author can rewrite it.

Indices that depend on data, such as an S-box lookup `sbox[x]`, were rejected
for this slice. They need a type of values below n, and they are the classic
source of cache-timing leaks in table-driven implementations. A specification
may still describe such a table later, under a decision that treats its
secrecy explicitly.

Mutation in place, such as `w[t] = v`, was rejected. It would make an array a
location rather than a value, and every later reader would need to know which
names share it. `x with [i] = v` gives the same algorithmic shape with no
aliasing.

A loop with several accumulators, such as the eight SHA-256 working
variables, is written with one array accumulator. A tuple accumulator would
need tuples, which belong to a later slice.

## Compatibility and migration

Every source that S3d accepts is accepted by S3e with the same Core values and
output bytes: `for` before an identifier, `with` before `[`, and `;` inside an
array literal were never valid. A source that S3d rejects gets the same
diagnostics, except that a non-literal index now parses and is reported by the
index rules, an empty index reads "expected an index after `[`", the note on
an out-of-range literal index and the note on a missing expression are
reworded, and the expression-nesting message names indices, loops, and
updates. The S3b runner's nesting message and one S3d fixture line, whose
variable index now reaches analysis, are updated accordingly.

The public Rust API gains `CoreLoop`, `MAX_LOOP_BOUND`, `LoopExpression`,
`UpdateExpression`, `FillExpression`, three `ExpressionKind` variants, six
`CoreNodeKind` variants, and a loop table on `CoreFunction`. An
`IndexExpression` now holds an expression, reached through `index()` and
`index_span()`.

Rollback reverts the parser, semantics, Core, evaluator, tests, fixtures, and
normative documents together.

## Semantic and claim effects

This proposal gives exact meaning to bounded loops, static indices, updates,
and fill literals. The supported claim remains deterministic, bounded analysis
and evaluation of the documented fragment at a recorded implementation
revision. It establishes no soundness, proof, refinement, compilation,
cryptographic correctness, constant-time behavior, compatibility, independent
review, or production readiness.

## TCB, axiom, and proof effects

The parser, analyzer, Core constructor, and evaluator remain engineering trust
dependencies. The interval computation that proves indices in range is new
trusted code, and the evaluator does not rely on it: every selection and
update checks its position again and fails closed on any Core whose loops,
scopes, positions, or types disagree. No axiom, theorem, proof rule,
certificate, checker, or solver is introduced.

## Threat, abuse, and leakage effects

The hostile-frontend surface grows by loops and computed indices. Literal
bounds cap each loop at 65536 steps, the per-source evaluation step budget caps
nested loops, loop frames are held on the evaluator's explicit stack, and a
1 MiB native-stack bound is tested with 64 nested loops in both positions,
nested updates, and deep indices. No secrecy label or leakage property is
defined. That an index cannot depend on data is a property of the source
language, not a claim about any compiled code.

## Target and ABI effects

None. A loop is a fold over a mathematical range, not a machine loop.

## Standards, errata, and provenance

FIPS 180-4 sections 4.1.2, 4.2.2, 5.3.3, and 6.2.2 with the NIST "abc" and
two-block examples, and RFC 8439 sections 2.1 through 2.4.2, motivate the
forms. The fixtures check the SHA-256 digests of both NIST examples and the
ChaCha20 block of section 2.3.2 and the ciphertext of section 2.4.2. No
standard gains normative authority through this proposal.

## Dependencies, licenses, and IP

No dependency is added.

## Conformance, tests, and evidence

`compiler/crates/orangec/tests/s3e_conformance.rs` binds the 18 rules of the
specification's index to evidence and fails on any drift. Seven fixtures and
one generated boundary case run through `orangec check` and `eval` twice each.
Unit tests cover spans, positional words, grammar errors, loop bounds and
scopes, interval ranges, update and fill typing, evaluation order, exact
event, node, and step accounting, call depth, allocation and foreign-input
failure, inconsistent Core, and the 1 MiB stack bound. The S2 through S3d
runners continue to pass, with the updates described above.

## Operations, release, and recovery

No service, package, key, or release is added.

## Support and deprecation

The fragment is pre-alpha and best effort under D-022, with no compatibility
promise.

## Unresolved questions

- Whether a function may take static index parameters, so that a ChaCha20
  quarter round can act on four positions of the whole state.
- Whether a tuple accumulator should replace the array accumulator for
  working variables of different roles.
- Whether data-dependent tables, such as the AES S-box, should be admitted in
  the specification stratum with an explicit secrecy annotation, or only as
  computed functions.
- Whether the interval proof should be strengthened, for example to see that
  `i - i` is always 0.

## Decision record

On 2026-09-28 the owner directed that Orange continue its implementation goals,
including the development steps after S3b, and that development not freeze
unless the owner asks. This proposal records the S3e surface built under that
direction and is presented for the owner's review. It is not accepted.
Acceptance is the owner's decision alone; until it is recorded here with a
decision date, reviewed revision, and `solo-reviewed` approval record, this
proposal authorizes nothing by itself.
