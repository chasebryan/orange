---
number: OEP-0018
title: Orange 2026 type parameters
authors:
  - Chase Bryan
champion: Chase Bryan
status: Review
type: Standards
created: 2026-09-30
updated: 2026-09-30
discussion: owner-direction-2026-09-30-s3o
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
supersedes: []
superseded-by: null
review-authorities:
  - Orange Project Owner
decision-date: null
decision-revision: null
approval-records: []
---

# OEP-0018: Orange 2026 type parameters

## Abstract

A function may declare **type parameters** in its brackets, beside or instead
of sizes: `K in {F, P, Q}` lists the types the function is written for, and
the function is checked, and runs, once for each. A call names its instance
by its types, `pow[F](x, e)`, or lets its arguments' types choose it, and
where they do not decide, the type its place expects:

```orange
spec pow[K in {F, L, P, Q, D}](x: K, e: Int) -> K { ... }
spec inverse[K in {F, L, P, Q, D}](x: K) -> K { pow(x, modulus[K]() - 2) }
spec ch[W in {Word[32], Word[64]}](x: W, y: W, z: W) -> W { (x & y) ^ (~x & z) }
```

The normative text is
[`docs/TYPE_PARAMETERS_2026.md`](../../TYPE_PARAMETERS_2026.md). An
implementation, five programs, and a 10-rule conformance runner accompany it
so that the proposal can be reviewed against running code. This proposal is
in **Review** and requires OEP-0017, which is also in review. It accepts no
D-004 candidate and gives the Typed Reference Core no canonical or proof
role.

## Motivation

Cryptography computes the same way in many types. Square-and-multiply,
Fermat inversion, and Euler's criterion are the same in the field of
Curve25519 (RFC 7748), the order of its subgroup (RFC 8032), the field of
Poly1305 (RFC 8439), and the moduli of ML-KEM (FIPS 203) and ML-DSA
(FIPS 204). FIPS 180-4 defines Ch, Maj, and the round of SHA-256 and SHA-512
by the same formulas on 32-bit and 64-bit words. Through S3n each had to be
written once for each type, and the copies had to be checked against each
other by eye.

S3m let one function stand for many lengths. S3o lets one function stand for
many types, by the same mechanism: a finite list, one instance for each
entry, every instance checked before anything runs. The fixtures write field
arithmetic once for five prime fields and reproduce values the standards
state (the square root of −1 that RFC 8032 decodes points with, and the
primitive roots of unity that FIPS 203 and FIPS 204 build their
number-theoretic transforms on), and write SHA-256 and SHA-512 with one Ch,
one Maj, one round, and one final addition, reproducing FIPS 180-4's digests.

## Scope and non-goals

This proposal defines type parameters in a function's brackets, the listed
types, the instances over sizes and types together, type entries in calls'
brackets, the fitting of a call without brackets by its arguments' types and
its place's type, the Core record of an instance, and the costs of all of
them. It adds no token and reserves no word, and it adds one diagnostic code,
`ORC0241`, for a type listed twice, a type entry that is not listed or not a
type, and a call that fits no instance by its arguments' types.

It does not define type variables, bounds or classes of types, reasoning
over all types, type parameters on `type` declarations, types computed from
sizes or values, residue or tuple types written in a call's brackets except
through a `type` declaration's name, or inference from anything but a call's
arguments and its place. It makes no timing, secrecy, or leakage claim and
fixes no layout, calling convention, or ABI.

### Strata assumption

As for S3b through S3n, S3o assumes only what every D-004 candidate gives the
Specification role: pure, total, deterministic meaning. An instance means
what the function written out with its types means, so a type parameter
introduces no new meaning, only a finite family of functions each with the
meaning the earlier slices give it. It therefore has the same meaning under
`ST-REL`, `ST-UNI`, `ST-DUAL`, `ST-MIRROR`, and `ST-HOST`, and the source
surface needs no change.

## Specification

[`docs/TYPE_PARAMETERS_2026.md`](../../TYPE_PARAMETERS_2026.md) is the
complete normative text. In summary:

- **Syntax.** `size_param = IDENTIFIER "in" (INTEGER ".." INTEGER |
  "{" listed_type ("," listed_type)* "}")`, at most four in a function's
  brackets, sizes and types together. A call's brackets hold one entry for
  each, parsed as expressions; the call scan admits `^` and a name's `[n]` so
  that `ch[Word[32]](e, f, g)` is a call.
- **Declarations.** Listed types resolve once, outside every instance, each
  once and without sizes. A type parameter's name is a type in the function's
  signature and body, not a value, and is no built-in type's, no `type`
  declaration's, and no other bracket parameter's.
- **Instances.** One for each combination of sizes' values and types, the
  first parameter slowest, at most 256. Each is checked as the function
  written out with its types; the first in error is reported.
- **Calls.** Entries name one instance, each type matched by equality with a
  listed type. Without brackets, the instance whose parameters have the
  arguments' types, or among several, the one whose result has the type the
  call's place expects.
- **Core and evaluation.** Each instance is one Core function recording its
  parameters' values and its name; types cost nothing at run time.

## Alternatives

Copies of each function, one per type, were the only way through S3n. They
are what the owner and reviewers would otherwise have to compare line by
line, and what drifts.

Type variables with bounds, as `fn pow<K: Field>`, were considered and
rejected for this slice. They need a language of bounds, a checker that
reasons about all types satisfying one, and a story for what a backend does
with an open family. A finite list needs none of these: each instance is
checked as written out, which is exactly what S3m does for sizes, and the
list says in the source which fields a function was checked for.

A macro or textual substitution was rejected: it checks nothing until it is
expanded, reports errors in text no one wrote, and gives no name to the
instance in error.

## Compatibility and migration

Every source that S3n accepts has no type parameter and is accepted with the
same Core values and output bytes. A source that S3n rejects gets the same
diagnostics, except that braces after a size parameter's `in` are now a type
parameter, a name followed by brackets holding `^` or a name's `[n]` and then
`(` is now a call, a fifth parameter of a function with a type parameter is
"a function has at most 4 size and type parameters", and a source whose calls
nest in conditional arguments, which took time doubling with each level, is
now checked in time linear in its depth. `orangec lex` is unchanged.

The public Rust API gains `SizeParameter::types` and `is_type` in the
`parser` module, `CoreFunction::instance` in the `core` module,
`EvaluatedFunction::instance` in the `eval` module, and the `DiagnosticCode`
variant `TypeParameter`. Code that matches every variant of that enum must
follow it.

Rollback reverts the parser, analyzer, Core, evaluator, tests, fixtures, and
normative documents together.

## Semantic and claim effects

This proposal gives exact meaning to type parameters and typed calls. The
supported claim remains deterministic, bounded analysis and evaluation of the
documented fragment at a recorded implementation revision. It establishes no
soundness, proof, refinement, compilation, cryptographic correctness,
constant-time behavior, compatibility, independent review, or production
readiness.

## TCB, axiom, and proof effects

The lexer, parser, analyzer, Core constructor, evaluator, and command-line
interface remain engineering trust dependencies. The parsing of type
parameters and type entries, the resolution of listed types and type
parameters' names, and the fitting of calls by type are new trusted code.
Unit tests check exact spans and messages, every rule of declarations and
calls, fitting by arguments and by place, calls across modules, Core records,
evaluation, the deepest sources the limits admit on a 1 MiB stack, the work
of fitting nested calls, allocation failure, and spans that do not belong to
their source. No axiom, theorem, proof rule, certificate, checker, or solver
is introduced.

## Threat, abuse, and leakage effects

A function has at most four parameters in brackets and 256 instances, so a
type parameter cannot ask for more checking than the same function written
out 256 times, which the per-source budgets already bound. Finding a call's
instance reads each argument once, so calls nested in each other's arguments
cost work linear in their depth at each level; the same change removes a
doubling of work per level for sized calls whose arguments are conditionals,
present since S3m, which let a small source take hours to check.

The reference evaluator is not constant-time, and no secrecy label or leakage
property is defined.

## Target and ABI effects

None. How a backend would compile a family of instances is left to D-011 and
later slices.

## Standards, errata, and provenance

RFC 7748 section 4.1 and RFC 8032 section 5.1 (the field and group order of
Curve25519, and the square root of −1 of RFC 8032 section 5.1.3), RFC 8439
section 2.5 (the field of
Poly1305), FIPS 203 section 4.3 and FIPS 204 section 7.5 (the moduli and roots
of unity of ML-KEM and ML-DSA), and FIPS 180-4 sections 4.1, 4.2, 5.3, and 6
motivate the slice. The fixtures check each field's modulus, Fermat inversion
and Euler's criterion in each, the square root of −1 modulo 2^255 − 19, that
17^128 is −1 modulo 3329 and 1753^256 is −1 modulo 8380417, and FIPS 180-4's
SHA-256 and SHA-512 digests of "abc" and of its two-block messages. No
standard gains normative authority through this proposal.

## Dependencies, licenses, and IP

No dependency is added.

## Conformance, tests, and evidence

`compiler/crates/orangec/tests/s3o_conformance.rs` binds the 10 rules of the
specification's index to evidence and fails on any drift. Five programs run
through `orangec check` and `eval` twice each, and a generated program
evaluates the 256 instances of a function over 64 residue types and four
sizes and shows that one type more is 260 instances and an error. Unit tests
cover the parser, declarations, instances, calls, fitting, Core
construction, evaluation, the deepest admitted sources, the work of fitting
nested calls, allocation failure, and foreign spans. The S2 through S3n
runners, the `orangec enc` tests, and the algorithms corpus continue to pass.

## Operations, release, and recovery

No service, package, key, or release is added.

## Support and deprecation

The fragment is pre-alpha and best effort under D-022, with no compatibility
promise.

## Unresolved questions

- Whether a residue type should be writable in a call's brackets as
  `Mod[m]`, beside a `type` declaration's name.
- Whether `type` declarations should take type parameters, as a field element
  paired with its modulus's name.
- Whether a list of types should be nameable once and reused, as `K in
  Fields`, so that several functions stay checked for the same fields.

## Decision record

On 2026-09-28 the owner directed that Orange continue its implementation goals
and that development not freeze unless the owner asks, and on 2026-09-29 that
development of Orange continue. One function for several fields and word
widths was the next language candidate after S3n's byte orders. This proposal
records the S3o surface built under that direction and is presented for the
owner's review. It is not accepted. Acceptance is the owner's decision alone;
until it is recorded here with a decision date, reviewed revision, and
`solo-reviewed` approval record, this proposal authorizes nothing by itself.
