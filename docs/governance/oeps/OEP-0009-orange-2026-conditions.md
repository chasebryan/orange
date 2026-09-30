---
number: OEP-0009
title: Orange 2026 conditions, comparisons, and Euclidean division
authors:
  - Chase Bryan
champion: Chase Bryan
status: Review
type: Standards
created: 2026-09-29
updated: 2026-09-29
discussion: owner-direction-2026-09-29-s3f
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
  - OEP-0008
supersedes: []
superseded-by: null
review-authorities:
  - Orange Project Owner
decision-date: null
decision-revision: null
approval-records: []
---

# OEP-0009: Orange 2026 conditions, comparisons, and Euclidean division

## Abstract

`Bool` is a type with the values `true` and `false` and the operators `!`,
`&&`, `||`, `==`, and `!=`. The comparisons `==`, `!=`, `<`, `<=`, `>`, and
`>=` give a `Bool`. `a / b` and `a % b` are Euclidean division and remainder,
total because `x / 0` is 0 and `x % 0` is x. `if c { a } else { b }` chooses
one of two values of the same type and evaluates only the one it chooses.
Static indices may divide, and remain proved in range.

With this slice, X25519 reads as RFC 7748 writes it:

```orange
spec x25519(scalar: Word[8]^32, u: Word[8]^32) -> Word[8]^32 {
  let k: Word[8]^32 = clamp(scalar);
  let masks: Word[8]^8 = [1, 2, 4, 8, 16, 32, 64, 128];
  let x1: Int = decode_u(u);
  let s: Int^4 = for i in 0..255 with s: Int^4 = [1, 0, x1, 1] {
    rung(x1, s, (k[(254 - i) / 8] & masks[(254 - i) % 8]) != 0)
  };
  encode((s[0] * power(s[1], prime() - 2)) % prime())
}
```

The normative text is [`docs/CONDITIONS_2026.md`](../../CONDITIONS_2026.md).
An implementation, 8 fixtures, and an 18-rule conformance runner accompany it
so that the proposal can be reviewed against running code. This proposal is
in **Review** and requires OEP-0008, which is also in review. It accepts no
D-004 candidate and gives the Typed Reference Core no canonical or proof role.

## Motivation

S3e can iterate over a cipher's state, but it cannot reduce modulo a prime,
test a bit, or choose between two values. Every public-key standard needs all
three. RFC 7748 defines X25519 in GF(2^255 - 19) with a Montgomery ladder
that swaps two points on each bit of the scalar; RFC 8439 defines Poly1305 in
GF(2^130 - 5) and seals an AEAD message by combining it with ChaCha20. The
field arithmetic is exact integer arithmetic followed by a remainder, and the
ladder is a choice on each bit. Without a remainder and a choice, Orange could
state symmetric primitives only.

Two properties keep these forms as easy to check as the standards' own text.
Division is total and its remainder is never negative, so "mod p" in Orange
is the canonical representative a cryptographer writes, and no division can
fail. And a conditional always has both branches, of one type, so a reader
never meets a missing value.

## Scope and non-goals

This proposal defines the type `Bool`, its literals and operators, the six
comparisons, Euclidean division and remainder with their total rules,
conditionals with `else if` chains, division in static indices, their Core
form, their evaluation, diagnostic `ORC0227`, and widened uses of `ORC0101`,
`ORC0108`, `ORC0214`, `ORC0215`, `ORC0223`, and `ORC0226`.

It does not define short-circuit evaluation, conditionals without `else`,
pattern matching, signed words, conversions to or from `Bool`, comparison of
whole arrays, a modular-integer type, or any exclusion of OEP-0008 that it
does not lift.

### Strata assumption

As for S3b through S3e, S3f assumes only what every D-004 candidate gives the
Specification role: pure, total, deterministic meaning. A comparison is a
total relation, Euclidean division with its zero rules is a total function,
and a conditional is a total choice between two values, so under `ST-REL`,
`ST-UNI`, `ST-DUAL`, `ST-MIRROR`, and `ST-HOST` alike the source surface needs
no change. Placement of the Core remains D-004's decision.

## Specification

[`docs/CONDITIONS_2026.md`](../../CONDITIONS_2026.md) is the complete
normative text. In summary:

- **Grammar.** `prefix = ( "-" | "~" | "!" ) operand`,
  `conditional = "if" expression "{" expression "}" "else" alternative`, and
  `alternative = "{" expression "}" | conditional`. Four operator groups are
  added: comparisons and divisions take exactly two operands, and `&&` and
  `||` each chain to the left. Groups still have no relative precedence, so a
  mixed expression is parenthesized (`ORC0108`). `if`, `else`, `true`, and
  `false` are recognized by position or scope, so no word is reserved.
- **`Bool`.** An admitted scalar and element type. `true` and `false` denote
  its values where no parameter, binding, loop index, or accumulator of that
  spelling is in scope. Its operators are `!`, `&&`, `||`, `==`, and `!=`
  (`ORC0215` otherwise); it takes no integer literal (`ORC0214`); `as` does
  not convert to or from it (`ORC0215`).
- **Comparisons.** A comparison gives `Bool` (`ORC0214` elsewhere). Its
  operands take the type of the first typed leaf of the left operand, else of
  the right (`ORC0227` if neither has one). Equality applies to every scalar;
  order applies to `Int` and words, which compare as unsigned integers.
- **Logic.** `!`, `&&`, and `||` apply to `Bool` and evaluate every operand.
- **Division.** For `Int`, a = b · (a / b) + a % b with 0 ≤ a % b < |b|. For
  words, unsigned. For every type, `x / 0` is 0 and `x % 0` is x.
- **Conditionals.** Conditions are `Bool`, values have the required type, a
  chain is one conditional per arm, and only the chosen branch is evaluated.
- **Indices.** A static index may use `/` and `%`, and its range follows the
  Euclidean rules, considering positive, zero, and negative divisors
  separately.
- **Core.** `Bool` is a Core type; each function gains a conditional table;
  Core gains `compare` and `choose` nodes. A function without the new forms
  has exactly its S3e Core.
- **Limits.** A conditional opens one nesting level for all its parts and a
  chain opens no more. Each arm is one semantic event and one table node.
  Comparisons of integers cost 1 + max(d(a), d(b)) steps and integer division
  1 + d(a) · max(d(b), 1); every other new operation costs one step; an
  untaken branch costs nothing and branches add no call depth. The deepest
  admitted sources, and a chain of 4096 arms, fit in 1 MiB of stack.

## Alternatives

Truncating division, as in C and Rust's `/` and `%`, was rejected. Its
remainder takes the sign of the dividend, so `-7 % 2` is -1, and every
reduction modulo p in a field would need a correction step that the
standards never write. Floored division was also considered; it agrees with
Euclidean division for positive divisors, which is every modulus in scope,
and differs only for negative divisors, where Euclidean division keeps the
remainder non-negative and the specification simpler to state.

Rejecting division by zero was rejected. A run-time failure is a value Orange
does not have, and a static proof that a divisor is never zero is out of
reach for divisors that depend on data. The total rules `x / 0 = 0` and
`x % 0 = x` keep the identity a = b · (a / b) + a % b true for every a and b,
and the fixtures never divide by zero.

Short-circuit `&&` and `||` were rejected. They would make the logical
operators a hidden choice, when Orange already has an explicit one. With
strict operators, the only way to skip work is a conditional, whose two
branches are visible.

A conditional without `else` was rejected, because it has no value when its
condition is false.

A comparison chain such as `a < b < c` was rejected in favor of
`(a < b) && (b < c)`. A chain reads differently in mathematics and in most
programming languages, and Orange admits neither reading silently.

Converting `Bool` to and from numbers with `as` was rejected. A truth value
is not a number, and the two directions are one comparison and one
conditional, both of which state the encoding they use.

A type of integers modulo a declared prime was considered and deferred. It
would reduce automatically and keep values in the field, but it needs a way to
declare the modulus, a decision on how values of different moduli interact,
and a rule for division by zero in the field. Exact `Int` arithmetic with `%`
states each reduction explicitly and is enough for every standard in scope.

## Compatibility and migration

Every source that S3e accepts is accepted by S3f with the same Core values and
output bytes: the new operators, `if` before an identifier, an integer
literal, `!`, or `~`, and `if` before a brace group followed by `else` were
never valid. A source that S3e rejects gets the same diagnostics, except that
`Bool` is now admitted, `true` and `false` are values where no name of their
spelling is in scope, the new forms parse and are reported by the new rules,
the notes on unsupported types, array operators, static indices, and a
missing expression are reworded, the expression-nesting message names
conditionals, and parsing recovers past the braces still open in a body after
a syntax error inside a loop's step or a conditional's value. The S3b runner's
nesting message, one S3c test and fixture line that used `Bool` as the
example of an unsupported type (now `Float`), and one S3d document example are
updated accordingly.

The public Rust API gains `CoreConditional`, `ConditionalExpression`,
`ConditionalArm`, the `Bool` variant of `CoreType`, a `Bool` literal value,
`Compare` and `Choose` node kinds, ten `BinaryOperator` variants, a `Not`
`UnaryOperator` variant, a `Conditional` `ExpressionKind` variant, the
`UntypedComparison` diagnostic code, and a conditional table on
`CoreFunction`.

Rollback reverts the parser, semantics, Core, evaluator, tests, fixtures, and
normative documents together.

## Semantic and claim effects

This proposal gives exact meaning to `Bool`, comparisons, logical operators,
Euclidean division, and conditionals. The supported claim remains
deterministic, bounded analysis and evaluation of the documented fragment at
a recorded implementation revision. It establishes no soundness, proof,
refinement, compilation, cryptographic correctness, constant-time behavior,
compatibility, independent review, or production readiness.

## TCB, axiom, and proof effects

The parser, analyzer, Core constructor, and evaluator remain engineering trust
dependencies. Multi-limb Euclidean division and comparison of exact integers
are new trusted code; unit tests check them against 128-bit references and
against the defining identity on multi-limb operands. The interval rules for
division in indices are new trusted code, and the evaluator does not rely on
them: every selection and update checks its position again. The evaluator
fails closed on any Core whose conditionals, scopes, or operand types
disagree. No axiom, theorem, proof rule, certificate, checker, or solver is
introduced.

## Threat, abuse, and leakage effects

The hostile-frontend surface grows by conditionals and the new operators.
Chains are parsed and checked by iteration, branch frames are held on the
evaluator's explicit stack, and a 1 MiB native-stack bound is tested with 64
conditionals nested in each position and a chain of 4096 arms. Integer
division is bounded by the step budget and never enlarges its operands.
Because only the chosen branch is evaluated, a branch that would exhaust a
budget is harmless unless it is chosen.

No secrecy label or leakage property is defined. A conditional is a
mathematical choice, not a machine branch, and nothing here claims that a
choice, comparison, or division would take the same time on every input in
any compiled code. That matters most for X25519, whose standard requires a
constant-time swap; a later decision on code generation must treat it
explicitly.

## Target and ABI effects

None. A conditional is a choice between values, not a machine branch.

## Standards, errata, and provenance

RFC 7748 sections 4.1, 5, and 5.2 and RFC 8439 sections 2.5, 2.5.2, 2.6, 2.8,
and 2.8.2 motivate the forms. The fixtures check the first X25519 test vector
of RFC 7748 section 5.2, the Poly1305 tag of RFC 8439 section 2.5.2, and the
ciphertext and tag of the AEAD example of section 2.8.2. No standard gains
normative authority through this proposal.

## Dependencies, licenses, and IP

No dependency is added.

## Conformance, tests, and evidence

`compiler/crates/orangec/tests/s3f_conformance.rs` binds the 18 rules of the
specification's index to evidence and fails on any drift. Eight fixtures and
one generated case run through `orangec check` and `eval` twice each. Unit
tests cover spans, positional words, `Bool` literal resolution, grammar
errors and recovery, comparison typing, Euclidean division against a 128-bit
reference and its defining identity, unsigned word division at every width,
index ranges under division, conditional numbering and evaluation order,
exact event, node, and step accounting, call depth, allocation and
foreign-input failure, inconsistent Core, and the 1 MiB stack bound. The S2
through S3e runners continue to pass, with the updates described above.

## Operations, release, and recovery

No service, package, key, or release is added.

## Support and deprecation

The fragment is pre-alpha and best effort under D-022, with no compatibility
promise.

## Unresolved questions

- Whether Orange should add a type of integers modulo a declared prime, so
  that field arithmetic reduces automatically and cannot leave the field.
- Whether signed comparison and arithmetic shift of words should be added,
  and under what spelling.
- How a later code-generation decision should treat conditionals whose
  condition depends on secret data, such as the X25519 swap.
- Whether the interval rules for division should be strengthened, for example
  to see that `i - (i % 4)` is never negative.

## Decision record

On 2026-09-28 the owner directed that Orange continue its implementation goals
and that development not freeze unless the owner asks, and on 2026-09-29 that
development of Orange continue. This proposal records the S3f surface built
under that direction and is presented for the owner's review. It is not
accepted. Acceptance is the owner's decision alone; until it is recorded here
with a decision date, reviewed revision, and `solo-reviewed` approval record,
this proposal authorizes nothing by itself.
