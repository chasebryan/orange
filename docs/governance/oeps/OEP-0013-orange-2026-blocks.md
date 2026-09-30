---
number: OEP-0013
title: Orange 2026 blocks
authors:
  - Chase Bryan
champion: Chase Bryan
status: Review
type: Standards
created: 2026-09-30
updated: 2026-09-30
discussion: owner-direction-2026-09-30-s3j
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
supersedes: []
superseded-by: null
review-authorities:
  - Orange Project Owner
decision-date: null
decision-revision: null
approval-records: []
---

# OEP-0013: Orange 2026 blocks

## Abstract

A loop's step and each branch of a conditional may begin with typed `let`
bindings, exactly as a function's body does, and end with the value they
give. This document calls them **blocks**. A step's bindings are evaluated
afresh at every step, and a branch's only when the branch is chosen. Each
binding is in scope for the bindings after it and for its block's value, and
nowhere else; Orange still has no shadowing.

With this slice, a round of SHA-256 names its working variables and its two
temporary words as FIPS 180-4 does, inside the loop that runs it:

```orange
let v: Word[32]^8 = for t in 0..64 with v: Word[32]^8 = hash {
  let a: Word[32] = v[0];
  // ... b through g ...
  let h: Word[32] = v[7];
  let t1: Word[32] = h + big_sigma1(e) + ch(e, f, g) + k[t] + w[t];
  let t2: Word[32] = big_sigma0(a) + maj(a, b, c);
  [t1 + t2, a, b, c, d + t1, e, f, g]
};
```

The normative text is [`docs/BLOCKS_2026.md`](../../BLOCKS_2026.md). An
implementation, six programs, and an 8-rule conformance runner accompany it
so that the proposal can be reviewed against running code. This proposal is
in **Review** and requires OEP-0012, which is also in review. It accepts no
D-004 candidate and gives the Typed Reference Core no canonical or proof
role.

## Motivation

A standard writes a round as a short list of named values. FIPS 180-4
section 6.2.2 names the working variables a through h and the temporary words
T1 and T2 inside each round of SHA-256. RFC 7748 section 5 names A, AA, B,
BB, E, C, D, DA, and CB inside each step of the Montgomery ladder. The
ChaCha20 quarter round, the AES round, and the Keccak step mappings are
written the same way.

Through S3i an Orange `let` could stand only at the start of a function's
body. A round's names therefore had to move into a helper function that took
the round's state as parameters, or the round had to repeat its
subexpressions. S3h's SHA-256 module and S3i's X25519 both take the first
way. SHA-256's loop calls a `round` function with the state, the round's
constant, and its message word, and reads a through h as `v[0]` through
`v[7]`. X25519's loop calls `rung`, which calls `swap` and `ladder`: three
functions where RFC 7748 writes one loop. Both are correct, and both are
harder to compare with their standard than the standard is to read. The
encryption lane asked for `let` inside a loop step for the same reason.

S3j lets the round stand where it runs, in the words of its standard. The
SHA-256 fixture of this slice writes the compression function as one loop
whose step names a through h, T1, and T2; the X25519 fixture writes the
ladder as one loop whose step names every value RFC 7748 names. Both
reproduce their published values.

## Scope and non-goals

This proposal defines the block syntax of a loop's step and of each branch,
the scope, uniqueness, and resolution of block bindings, their typing and
its order, the meaning of a step's and a branch's bindings, their record in
the Typed Reference Core and the evaluator's step costs, and their limits.
It adds no diagnostic code, no token, and no reserved word.

It does not define a block as an expression of its own outside a step or a
branch, shadowing, assignment or mutation, loops with more than one
accumulator, tuples or destructuring, inference of a binding's type, or a
static range for an `Int` binding. It makes no timing, secrecy, or leakage
claim.

### Strata assumption

As for S3b through S3i, S3j assumes only what every D-004 candidate gives the
Specification role: pure, total, deterministic meaning. A block binding names
a value and has no effect, so a block means exactly what its value means with
each binding's value written in place. Under `ST-REL`, `ST-UNI`, `ST-DUAL`,
`ST-MIRROR`, and `ST-HOST` alike the source surface needs no change.

## Specification

[`docs/BLOCKS_2026.md`](../../BLOCKS_2026.md) is the complete normative
text. In summary:

- **Syntax.** `block = "{" binding* expression "}"` replaces the one
  expression of a loop's step and of each branch. `let` is recognized by
  position. A block declares at most 256 bindings (`ORC0106`); its bindings
  are parsed at the nesting level of its value and count toward the height of
  their loop or conditional. A block that ends with a binding, or a binding
  missing a part, is `ORC0101` with a note that describes the block.
- **Scope.** A block's binding is in scope from the end of its `;` to the end
  of its block. It repeats no name in scope (`ORC0219`); names whose scopes
  do not overlap may repeat. A name used before its binding's `;` is "used
  before it is bound", and one used outside its block is "not in scope here",
  citing the binding (`ORC0211`).
- **Typing.** Each binding's value has its declared type, and parts are
  checked in source order, every part even after an error. Where a
  conditional's type is needed before its branches are checked, a branch
  whose typed leaf names one of its own bindings gives no type (`ORC0220`,
  `ORC0227`).
- **Meaning.** A step's bindings are evaluated at every step, in order, and
  the step's value becomes the next accumulator. Only the chosen branch's
  bindings are evaluated.
- **Core and evaluation.** A Core loop records its step's bindings, and a
  Core conditional those of its `then` and `else` branches, each with the
  offset `end` just after its value's subtree. Reads are `step_binding` and
  `branch_binding` nodes and cost one step; a binding costs the steps of its
  value each time its block is evaluated.

## Alternatives

Helper functions, as S3h's SHA-256 and every S3e through S3i round use, were
the status quo. They stay valid and are the right form when a round is
reused. As the only form, they force a round's state into parameters and
separate the round from the loop that runs it.

Loops with more than one accumulator and tuples were considered first. They
would let SHA-256 carry a through h as eight accumulators instead of an
array. They are a larger change, to the type system and to Core, and they
answer a different question: what a loop carries from one step to the next,
not what a step names within itself. Blocks are useful with or without them,
and they remain on the roadmap.

Blocks as expressions everywhere, as in Rust, were deferred. A general
`{ let ...; value }` expression would let bindings stand inside any operand.
It would also allow a binding deep inside an index or an argument, where it
is harder to read, and it raises questions of evaluation order that steps
and branches, which are already delimited by braces, do not. Every S3j
program stays valid if general blocks are added later.

Shadowing, so that a step could rebind the accumulator's name, was rejected
for the reason S3c gives: in a transcription of a standard, a name should
mean one thing wherever it is in scope.

Evaluating a branch's bindings whether or not the branch is chosen was
rejected. It would make a branch's cost and its failures depend on a branch
that does not run, and it would give an unchosen branch's bindings a meaning
that its value, which is not evaluated, does not have.

## Compatibility and migration

Every source that S3i accepts has no binding in a step or branch, so S3j
accepts it with the same Core values, messages, and output bytes. A source
that S3i rejects gets the same diagnostics, except that a `let` binding at the
start of a loop's step or of a branch, which was `ORC0101`, is now accepted or
reported by the S3j rules, and the notes of the S3e and S3f diagnostics for a
malformed step or branch now describe a block.

The public Rust API gains `CoreBinding`, exported from the crate root, with
`span`, `name`, `name_span`, `ty`, and `end`; `CoreLoop::bindings`,
`CoreConditional::then_bindings` and `else_bindings`,
`LoopExpression::step_bindings`, `ConditionalArm::bindings`, and
`ConditionalExpression::otherwise_bindings`; and the `CoreNodeKind` variants
`StepBinding` and `BranchBinding`. Code that matches every `CoreNodeKind`
variant must handle the two new ones. Functions and values that never meet a
block behave as before.

Rollback reverts the parser, analyzer, Core, evaluator, tests, fixtures, and
normative documents together.

## Semantic and claim effects

This proposal gives exact meaning to bindings within a loop's step and a
branch. The supported claim remains deterministic, bounded analysis and
evaluation of the documented fragment at a recorded implementation revision.
It establishes no soundness, proof, refinement, compilation, cryptographic
correctness, constant-time behavior, compatibility, independent review, or
production readiness.

## TCB, axiom, and proof effects

The parser, analyzer, Core constructor, evaluator, and command-line interface
remain engineering trust dependencies. The block parser, the scope of block
bindings and the lookup of finished blocks for diagnostics, the `end` offsets
in Core, and the evaluator's binding slots are new trusted code. Unit tests
check exact spans, scope and uniqueness in nested blocks, the order of
checking, node and event accounting, the Core layout of bindings, evaluation
at every step and only in the chosen branch, step costs, the deepest sources
the limits admit on a 1 MiB stack, eleven kinds of inconsistent Core, and
spans that do not belong to their source. No axiom, theorem, proof rule,
certificate, checker, or solver is introduced.

## Threat, abuse, and leakage effects

A block multiplies neither the work nor the size of a program: each binding
costs the semantic events, Core nodes, and steps its value would cost in
place, one node for itself and one for its type, and one step per read. A
block declares at most 256 bindings, as a body does, and blocks nest within
the existing 64-level limit and height bound, so the analyzer's and
evaluator's memory and native stack stay bounded; the deepest admitted
sources run within 1 MiB of stack. The step budget of 1,048,576 is
unchanged. Core whose bindings are inconsistent with their expression stops
evaluation with no values, and an allocation failure gives no Core.

A branch's bindings are evaluated only when it is chosen, as its value is.
The reference evaluator is not constant-time, and no secrecy label or
leakage property is defined.

## Target and ABI effects

None. A block binding is a name for a value of the specification; how a
compiled program holds it is left to D-011 and later slices.

## Standards, errata, and provenance

FIPS 180-4 section 6.2.2 and RFC 7748 section 5 motivate the slice. The
fixtures check the SHA-256 digests of "abc" and of the 448-bit message of
FIPS 180-4's examples, and the first X25519 test vector of RFC 7748 section
5.2. No standard gains normative authority through this proposal.

## Dependencies, licenses, and IP

No dependency is added.

## Conformance, tests, and evidence

`compiler/crates/orangec/tests/s3j_conformance.rs` binds the 8 rules of the
specification's index to evidence and fails on any drift. Six programs run
through `orangec check` and `eval` twice each, and a generated program checks
a step and a branch of 256 bindings and of 257. Unit tests cover the parser,
scope and names, typing, Core construction, evaluation and its step costs,
the deepest admitted sources, inconsistent Core, allocation failure, and
foreign spans. The S2 through S3i runners continue to pass, with only the
S3f note that now describes a block updated.

## Operations, release, and recovery

No service, package, key, or release is added.

## Support and deprecation

The fragment is pre-alpha and best effort under D-022, with no compatibility
promise.

## Unresolved questions

- Whether a block should become an expression of its own, usable in any
  operand.
- Whether loops should carry more than one accumulator, or tuples be added,
  so that a round's state need not be an array.
- Whether an `Int` binding should carry a static range from its value, so
  that it can index a table.

## Decision record

On 2026-09-28 the owner directed that Orange continue its implementation goals
and that development not freeze unless the owner asks, and on 2026-09-29 that
development of Orange continue. `let` inside a loop's step was the first
language request of the encryption lane and was pending after S3i. This
proposal records the S3j surface built under that direction and is presented
for the owner's review. It is not accepted. Acceptance is the owner's decision
alone; until it is recorded here with a decision date, reviewed revision, and
`solo-reviewed` approval record, this proposal authorizes nothing by itself.
