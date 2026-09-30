---
number: OEP-0016
title: Orange 2026 sizes
authors:
  - Chase Bryan
champion: Chase Bryan
status: Review
type: Standards
created: 2026-09-30
updated: 2026-09-30
discussion: owner-direction-2026-09-30-s3m
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
supersedes: []
superseded-by: null
review-authorities:
  - Orange Project Owner
decision-date: null
decision-revision: null
approval-records: []
---

# OEP-0016: Orange 2026 sizes

## Abstract

A `spec` may declare **size parameters**, `spec pad[len in 1..120](m:
Word[8]^len) -> ...`, each with a finite range. The function then stands for
one **instance** for each value of its sizes, `pad[1]` through `pad[119]`,
and every instance is checked as if it were written out by hand. A **size**,
built from integer literals and size parameters, writes an array length, a
fill length, or a loop bound, and a size parameter's name is an `Int`
constant in expressions. A call names its instance by its sizes, `pad[3](m)`,
or by the lengths of its arguments, `sha256("abc")`:

```orange
spec sha256[len in 1..120](m: Word[8]^len) -> Word[8]^32 { absorb(pad(m)) }

spec abc() -> Word[8]^32 { sha256("abc") }
```

The normative text is [`docs/SIZES_2026.md`](../../SIZES_2026.md). An
implementation, six programs, and a 10-rule conformance runner accompany it
so that the proposal can be reviewed against running code. This proposal is
in **Review** and requires OEP-0015, which is also in review. It accepts no
D-004 candidate and gives the Typed Reference Core no canonical or proof
role.

## Motivation

A standard defines its algorithms for inputs of many lengths. FIPS 180-4
hashes a message of any length, padding it to whole 64-byte blocks; RFC 2104
keys HMAC with any key of up to one block and signs any message; RFC 8439
section 2.5 authenticates any message with Poly1305, sixteen bytes at a time,
the last block holding what is left.

Through S3l every array had a length written as an integer. The S3l HMAC
fixture padded each message by hand, with its zero bytes and its length in
bits worked out for "abc", for the 72-byte inner input of test case 1, for
the 92-byte one of test case 2, and for the 96-byte outer input, and it had
one hash for one block and another for two. The ChaCha20-Poly1305 fixture
fixed its plaintext at 114 bytes, and the schemes of `orangec enc` seal
chunks of exactly 240 bytes. A program for a message of 3 bytes and one for
56 bytes were two programs, although the standard gives one algorithm.
Static size parameters were the next language candidate on the roadmap after
S3l for this reason.

With S3m, SHA-256 is written once for every message of 1 through 119 bytes,
HMAC-SHA-256 once for every key of 1 through 63 bytes and every message of 1
through 55, and Poly1305 once for every message of 1 through 255 bytes. The
fixtures reproduce FIPS 180-4's digests of "abc" and of its 56-byte two-block
message, RFC 4231's test cases 1 and 2, and RFC 8439's section 2.5.2 tag, and
every instance of each function, 119 of `sha256` and 255 of `mac`, is checked
before any of them runs.

## Scope and non-goals

This proposal defines size parameters on a `spec`, the sizes built from them,
their use in array lengths, fill lengths, loop bounds, and expressions, the
checking of a function once for each of its instances, calls that name an
instance by their sizes or by their arguments' lengths, the record of
instances in the Typed Reference Core, and their display by `orangec eval`.
It adds no token and reserves no word, and it adds three diagnostic codes:
`ORC0237` for a size built from anything but integer literals and size
parameters, `ORC0238` for a size or a range outside its bounds, and `ORC0239`
for a call that gives the wrong number of sizes or names no single instance.

It does not define symbolic reasoning over sizes, sizes inferred from
anything but arguments' lengths, size parameters on `type` declarations or
moduli, sizes as run-time values, sets of values other than ranges, more than
256 instances of a function, empty arrays, or recursion over sizes. It makes
no timing, secrecy, or leakage claim and fixes no layout, calling convention,
or ABI.

### Strata assumption

As for S3b through S3l, S3m assumes only what every D-004 candidate gives the
Specification role: pure, total, deterministic meaning. A sized function is a
finite family of functions, each with the meaning of the function written out
with its sizes' values, and a size is an integer computed before the program
runs. Each instance therefore has the same meaning under `ST-REL`, `ST-UNI`,
`ST-DUAL`, `ST-MIRROR`, and `ST-HOST`, and the source surface needs no
change. Because every size is fixed before a program runs, a size reveals
nothing about data, and no length, index, or slice position becomes
data-dependent.

## Specification

[`docs/SIZES_2026.md`](../../SIZES_2026.md) is the complete normative text.
In summary:

- **Syntax.** `spec f[n in a..b, ...](...)` declares at most four size
  parameters with integer bounds. A size is an integer token, a name, or a
  parenthesized expression, and stands where an array length, a fill length,
  or a loop bound was written. A call gives sizes in brackets before its
  arguments, `f[2](x)` or `m::f[2](x)`; a name followed by brackets that hold
  only size tokens and then `(` is a sized call, and any other brackets are an
  index or a slice. Every malformed form is `ORC0101` with a note that shows
  the form.
- **Size parameters.** A range is nonempty with bounds at most 65536, and a
  function has at most 256 instances (`ORC0238`); size parameters share one
  namespace with parameters and bindings (`ORC0218`, `ORC0219`).
- **Sizes.** A size is built from integer literals and size parameters with
  `+`, `-`, `*`, `/`, `%`, prefix `-`, and parentheses (`ORC0237`), computed
  exactly within the significant-bit limit of `Int` (`ORC0205`), with `/` and
  `%` Euclidean and total. Every length it writes is from 1 through 256 in
  every instance (`ORC0221`), and every loop bound obeys the loop rules
  (`ORC0225`). A size parameter's name is an `Int` constant, and counts as an
  integer literal in indices and slice bounds.
- **Instances.** Every instance is checked in order, the first size changing
  slowest, as the function written out with its sizes' values; only the first
  instance of a function in error is reported, named in a note.
- **Calls.** A call's sizes are counted (`ORC0239`) and each must lie in its
  range (`ORC0238`). A call that writes none calls the one instance whose
  array parameters have its arguments' lengths; two or more (`ORC0239`) or
  none (`ORC0238`) is an error. Call cycles are found between instances
  (`ORC0217`).
- **Core and evaluation.** Each instance is one Core function recording its
  sizes; a size's name is an `Int` literal node. `orangec eval` evaluates
  every instance of each root `spec` without value parameters and names it as
  a call does, `sizes::zeros[2]`.

## Alternatives

Symbolic size parameters, checked once for all values, were considered.
Proving that `x[64 * b..64 * b + 64]` lies within `Word[8]^(64 * blocks)` for
every `blocks` needs arithmetic over unknowns: linear facts, and products of
unknowns in lengths such as `64 * (((len + 8) / 64) + 1)`, whose division
takes it outside linear arithmetic. That would be a new decision procedure in
the trusted base, and its errors would name no value. Checking each instance
of a finite range reuses every existing rule, reports the first value at
which a function fails, and costs work proportional to the number of
instances, which the 256 limit and the semantic budgets bound.

Inferring sizes from the arguments' types by unification was tried first.
Lengths such as `64 * blocks` are not invertible in general, and the error for
a failed inference was hard to state. Fitting the instance whose parameters
have the arguments' found lengths needs no inverse, and when it is ambiguous
the call writes its sizes.

Sizes written in angle brackets, `f<n>`, were rejected: `<` is already a
comparison, and square brackets already write `Word[32]` and `Mod[m]`, the
parameters of a type.

## Compatibility and migration

Every source that S3l accepts has no size parameter and writes every length
and loop bound as an integer token, so S3m accepts it with the same Core
values, messages, and output bytes. A source that S3l rejects gets the same
diagnostics, except that brackets after a function's name in a declaration,
a name or a parenthesized expression as a length or a loop bound, and a name
followed by bracketed size tokens and `(`, which were `ORC0101`, are now
size parameters, sizes, and sized calls, accepted or reported by the S3m
rules, and "expected an integer length after `^`" is now "expected a length
after `^`". `orangec lex` is unchanged.

The public Rust API gains, in the `parser` module, `Size`, `SizeParameter`,
and `CallQualifiers`, `FunctionDeclaration::sizes`, a `Size` in place of the
length span of `TypeSyntax`, `FillExpression`, and the bounds of
`LoopExpression`, and `CallExpression::module()` and `sizes()` in place of
its `module` field; `CoreFunction::sizes` and `EvaluatedFunction::sizes`; and
the `DiagnosticCode` variants `NonStaticSize`, `SizeRange`, and `SizeCount`.
Code that matches every variant of these enums, or reads the fields replaced,
must follow them.

Rollback reverts the parser, analyzer, Core, evaluator, tests, fixtures, and
normative documents together.

## Semantic and claim effects

This proposal gives exact meaning to size parameters, sizes, instances, and
sized calls. The supported claim remains deterministic, bounded analysis and
evaluation of the documented fragment at a recorded implementation revision.
What is checked of a sized function is checked for each of its instances; no
claim is made for a value outside its sizes' ranges. It establishes no
soundness, proof, refinement, compilation, cryptographic correctness,
constant-time behavior, compatibility, independent review, or production
readiness.

## TCB, axiom, and proof effects

The lexer, parser, analyzer, Core constructor, evaluator, and command-line
interface remain engineering trust dependencies. The parsing of size
parameters, sizes, and sized calls, the lookahead that tells a sized call from
an index, the size evaluator, the checking of each instance, the fitting of
calls without sizes, the search for cycles between instances, and the
record and display of instances are new trusted code. Unit tests check exact
spans and messages, the lookahead, heights, ranges and names, the size
evaluator and its checking order and limit, instance order and reporting,
calls with and without sizes across modules, cycles, events, the deepest
sources the limits admit on a 1 MiB stack, and spans that do not belong to
their source. No axiom, theorem, proof rule, certificate, checker, or solver
is introduced.

## Threat, abuse, and leakage effects

A function has at most 4 size parameters and 256 instances, and each bound is
at most 65536, so a declaration cannot ask for unbounded work. Every instance
is checked in full, and every part of every size, in every instance, costs a
semantic event, so the per-source budgets of 1,048,576 semantic events and
262,144 Core nodes bound all instances together; a source that asks for more
is stopped with a resource diagnostic and no Core. A size is computed exactly
and within the significant-bit limit of `Int`, so no size overflows or wraps.
The lookahead that recognizes a sized call reads each token at most once, and
the deepest admitted sources run within 1 MiB of stack. The step budget of
1,048,576 is unchanged and covers every instance evaluated.

The reference evaluator is not constant-time, and no secrecy label or leakage
property is defined. A size is fixed before a program runs, so the choice of
instance depends only on the program's text and the lengths of its values,
never on their contents.

## Target and ABI effects

None. How a backend would compile a family of instances, as copies or as one
function taking a length, is left to D-011 and later slices.

## Standards, errata, and provenance

FIPS 180-4 sections 5.1.1 and 6.2.2, RFC 2104, RFC 4231 section 4, and RFC
8439 section 2.5 motivate the slice. The fixtures check FIPS 180-4's SHA-256
digests of "abc" and of the 56-byte message
"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq", the HMAC-SHA-256
values of RFC 4231 test cases 1 and 2, and the Poly1305 tag of RFC 8439
section 2.5.2. No standard gains normative authority through this proposal.

## Dependencies, licenses, and IP

No dependency is added.

## Conformance, tests, and evidence

`compiler/crates/orangec/tests/s3m_conformance.rs` binds the 10 rules of the
specification's index to evidence and fails on any drift. Six programs run
through `orangec check` and `eval` twice each, and a generated program checks
a function of four sizes with 256 instances, a function with every array
length from 1 through 256, and a size bound of 65536, and one instance or
one bound more. Unit tests cover the parser, sizes, instances, calls, Core
construction, evaluation, events, the deepest admitted sources, and foreign
spans. The S2 through S3l runners and the algorithms corpus continue to pass.

## Operations, release, and recovery

No service, package, key, or release is added.

## Support and deprecation

The fragment is pre-alpha and best effort under D-022, with no compatibility
promise.

## Unresolved questions

- Whether sizes should be checked symbolically, once for all values, as a
  later, separately trusted analysis alongside the per-instance one.
- Whether the 256-instance limit should rise, and whether a function's
  instances should be checked lazily, only where they are called.
- Whether `type` declarations should take size parameters, as `type
  Block[n] = Word[8]^(16 * n)`.
- Whether conversions between bytes and words should be written with sizes,
  as `le[n](b)`, or built in.

## Decision record

On 2026-09-28 the owner directed that Orange continue its implementation goals
and that development not freeze unless the owner asks, and on 2026-09-29 that
development of Orange continue. Static size parameters were the next language
candidate on the roadmap after S3l. This proposal records
the S3m surface built under that direction and is presented for the owner's
review. It is not accepted. Acceptance is the owner's decision alone; until it
is recorded here with a decision date, reviewed revision, and `solo-reviewed`
approval record, this proposal authorizes nothing by itself.
