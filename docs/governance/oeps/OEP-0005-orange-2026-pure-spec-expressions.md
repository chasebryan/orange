---
number: OEP-0005
title: Orange 2026 pure specification expressions
authors:
  - Chase Bryan
champion: Chase Bryan
status: Draft
type: Standards
created: 2026-09-28
updated: 2026-09-28
discussion: owner-direction-2026-09-28-s3b
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
supersedes: []
superseded-by: null
review-authorities:
  - Orange Project Owner
decision-date: null
decision-revision: null
approval-records: []
---

# OEP-0005: Orange 2026 pure specification expressions

## Abstract

Orange 2026 `spec` functions gain parameters, calls, and pure expressions over
mathematical integers and fixed-width words. The accepted word types become
`Word[8]`, `Word[16]`, `Word[32]`, and `Word[64]`, each denoting the ring of
integers modulo 2^n. Words support modular addition, subtraction, and
multiplication; bitwise and, or, exclusive or, and complement; logical shifts;
and rotations. Integers support exact addition, subtraction, multiplication,
and negation.

The aim is that a specification reads like the clause of the standard it
transcribes. With this slice, the SHA-256 functions of FIPS 180-4 section 4.1.2
can be written as:

```orange
edition 2026;
module sha256 {
  spec ch(x: Word[32], y: Word[32], z: Word[32]) -> Word[32] {
    (x & y) ^ (~x & z)
  }
  spec maj(x: Word[32], y: Word[32], z: Word[32]) -> Word[32] {
    (x & y) ^ (x & z) ^ (y & z)
  }
  spec big_sigma0(x: Word[32]) -> Word[32] { (x >>> 2) ^ (x >>> 13) ^ (x >>> 22) }
  spec big_sigma1(x: Word[32]) -> Word[32] { (x >>> 6) ^ (x >>> 11) ^ (x >>> 25) }
  spec small_sigma0(x: Word[32]) -> Word[32] { (x >>> 7) ^ (x >>> 18) ^ (x >> 3) }
  spec small_sigma1(x: Word[32]) -> Word[32] { (x >>> 17) ^ (x >>> 19) ^ (x >> 10) }
  spec sample() -> Word[32] { big_sigma0(0x6a09_e667) }
}
```

`orangec eval` prints `sha256::sample: Word[32] = 0xce20b47e`.

This proposal is a **Draft**. It records the proposed S3b surface so that it
can be reviewed, tested, and implemented as one bounded slice. It accepts no
D-004 candidate and gives the Typed Reference Core no canonical or proof role.

## Motivation

S3a established a typed, deterministic, fail-closed semantic foothold, but it
cannot express a single cryptographic function. Every flagship corpus family
begins with word arithmetic: SHA-2 with 32- and 64-bit rotations, shifts, and
modular addition; ChaCha20 with add-rotate-xor rounds; Keccak with 64-bit lanes.
Until those operations exist in the specification stratum, no corpus
definition can be written, evaluated against official vectors, and kept as a
permanent fixture.

The operations chosen here are the ones those standards print. Their meaning
is the one the standards assume: a `w`-bit word is an element of the integers
modulo 2^w, `+` is addition modulo 2^w, `ROTR^n` is rotation, and `SHR^n` is a
logical shift. Orange adopts that reading rather than the overflow-checked or
undefined-behavior readings of general-purpose languages, because in a
specification the ring is the point.

Pure, total, closed evaluation keeps the slice proof-neutral. Every expression
denotes a value, every call terminates, and nothing observes time, memory, or
secrets. A pure specification fragment of this kind is expected to be needed
under every D-004 strata candidate, which is why this slice can proceed before
D-004 is accepted without deciding it.

## Scope and non-goals

This proposal defines parameters, calls, pure expressions, the four word
widths, operator typing and grouping, literal typing, call-graph acyclicity,
Typed Reference Core expressions, reference evaluation, output formats,
diagnostics, and resource limits.

It does not define local bindings, tuples, booleans, comparisons, conditionals,
loops, division or remainder, conversions between types, variable shift or
rotation amounts, recursion, typed `impl` declarations, contracts, effects,
secrecy labels, failure values, imports, multiple modules, proofs, claims,
games, canonical Core serialization, code generation, targets, ABI, layout,
leakage, packages, cryptographic claims, releases, or support. Each is left
for a later bounded slice.

It does not accept D-004 and does not decide whether the Typed Reference Core
becomes part of Spec Core, a shared pure fragment, or another representation.
`impl` declarations keep their S3a meaning exactly.

## Specification

### Lexical additions

Four punctuation tokens are added, with longest-match lexing:

```text
<<  >>  <<<  >>>
```

`<<<` is preferred to `<<` and `<`; `>>>` is preferred to `>>` and `>`. No other
token, reserved word, or trivia rule changes. A source that previously lexed
`<<` as two `<` tokens now lexes it as one `<<` token; no such source was
syntactically valid before this slice.

### Surface grammar

The S3a grammar remains valid. The `spec` declaration gains parameters and an
expression body:

```text
function_decl  = "spec" IDENTIFIER "(" ")" spec_tail
               | "spec" IDENTIFIER "(" parameter_list ")" typed_tail
               | "impl" IDENTIFIER "(" ")" empty_body ;
spec_tail      = empty_body | typed_tail ;
typed_tail     = "->" parsed_type "{" expression "}" ;
empty_body     = "{" "}" ;
parameter_list = parameter ("," parameter)* ;
parameter      = IDENTIFIER ":" parsed_type ;

expression     = sum | bit_chain | shift ;
sum            = product (("+" | "-") product)* ;
product        = unary ("*" unary)* ;
bit_chain      = unary ("&" unary)+
               | unary ("|" unary)+
               | unary ("^" unary)+ ;
shift          = unary ("<<" | ">>" | "<<<" | ">>>") unary ;
unary          = ("-" | "~") unary | primary ;
primary        = INTEGER | IDENTIFIER | call | "(" expression ")" ;
call           = IDENTIFIER "(" (expression ("," expression)*)? ")" ;
```

The grammar is LL(1) after its first operand: the first binary operator
selects the expression form. It deliberately has no precedence between
operator groups. Arithmetic uses the usual rule that `*` binds more tightly
than `+` and `-`, and both associate to the left. Each of `&`, `|`, and `^`
may be chained with itself, but two different bitwise operators, or a bitwise
and an arithmetic operator, cannot share a level without parentheses. A shift
or rotation takes exactly two unary operands and is not associative. Unary
operators bind more tightly than every binary operator.

So `(x >>> 2) ^ (x >>> 13)` is accepted and `x >>> 2 ^ x >>> 13` is a syntax
error, as is `a ^ b & c` or `a + b ^ c`. The rule exists for readers: every
mixed expression in Orange shows its grouping, as it does in the standards.

A typed body contains exactly one expression. The S3a body `{ -42 }` remains
valid and means what it meant: unary minus applied to an integer literal.

### Names

Function names keep their two S3a namespaces keyed by `(kind, name)`. Parameter
names are compared by exact ASCII spelling and must be unique within their
function. In an expression, a bare identifier names a parameter of the
enclosing function; an identifier followed by `(` names a typed `spec` in the
same module. A call may name a `spec` declared earlier or later in the source.
An empty `spec` has no value and cannot be called. An `impl` cannot be called.

The call graph among typed specifications must be acyclic. A function that
calls itself, directly or through other functions, is a semantic error. Every
accepted program therefore terminates.

### Types

The accepted parsed types are exactly:

| Source form | Core type | Values |
| --- | --- | --- |
| `Int` | `Int` | all mathematical integers |
| `Word[8]` | `Word8` | integers modulo 2^8 |
| `Word[16]` | `Word16` | integers modulo 2^16 |
| `Word[32]` | `Word32` | integers modulo 2^32 |
| `Word[64]` | `Word64` | integers modulo 2^64 |

The width must be spelled in decimal without a prefix, separator, or leading
zero. Every other parsed type, including `Int[8]`, bare `Word`, `Word[08]`,
`Word[12]`, and `Word[128]`, is rejected. There is no inference, alias,
subtyping, overloading, coercion, or implicit conversion.

### Typing

Every expression is checked against an expected type, so no expression needs
inference:

- a function body is checked against the declared result type;
- a call's arguments are checked against the callee's parameter types, and
  the call's result type must equal the expected type;
- a parameter reference must have exactly the expected type;
- both operands of `+`, `-`, `*`, `&`, `|`, and `^`, and the operand of a unary
  operator, are checked against the expected type;
- the left operand of a shift or rotation is checked against the expected
  type; and
- a parenthesized expression is checked against the expected type.

An integer literal takes the expected type. For `Int`, its magnitude may have at
most 16,384 significant bits. For `Word[n]`, its value must lie in 0 through
2^n - 1, and a minus sign written directly before a word literal remains the
S3a error, including for `-0`.

Operators are defined only on these types:

| Operator | `Int` | `Word[n]` |
| --- | --- | --- |
| `a + b`, `a - b`, `a * b` | exact | modulo 2^n |
| `-a` | exact negation | not defined |
| `a & b`, `a \| b`, `a ^ b` | not defined | bitwise and, or, exclusive or |
| `~a` | not defined | bitwise complement |
| `a << k`, `a >> k` | not defined | logical shift left or right by `k` |
| `a <<< k`, `a >>> k` | not defined | rotation left or right by `k` |

Modular negation of a word is written `0 - a`. The amount `k` of a shift or
rotation must be an unsigned integer literal with value 0 through n - 1. It is
not an expression and has no type of its own.

### Meaning

Evaluation is call-by-value and left to right. For `Word[n]`, arithmetic
results are reduced modulo 2^n. `a << k` is `a * 2^k` modulo 2^n; `a >> k` is
the floor of `a / 2^k`; `a <<< k` is `(a << k) | (a >> (n - k))` when
0 < k < n, and `a` when k is 0; `a >>> k` is `a <<< ((n - k) mod n)`. Bitwise
operators act on the n-bit binary representation. `~a` is `2^n - 1 - a`.

For `Int`, operators have their exact mathematical meaning. `Int` remains an
unbounded domain; the evaluator's magnitude limit below is a resource
boundary, not a finite width.

### Typed Reference Core

The Typed Reference Core gains parameters and expressions. Each Core function
has its S3a ID, name, and result type, a list of parameter types in order, and
one expression tree whose nodes are typed literals, parameter references by
index, unary operations, binary operations, shifts and rotations with a
literal amount, and calls by function ID. The S3a literal-only function is the
special case with no parameters and a literal body.

The Core still has no canonical encoding, digest, proof identity, refinement
relation, target meaning, or public interchange compatibility.

### Reference evaluation and CLI

`orangec check` is unchanged except that it now validates the larger
fragment. `orangec eval FILE` evaluates every typed `spec` that has no
parameters, in source order, and prints one line for each, as in S3a.
Parameterized functions are checked but not printed; they are evaluated only
through calls.

A `Word[n]` value prints as `0x` followed by exactly n/4 lowercase hexadecimal
digits, so `Word[8]` output is unchanged and a `Word[32]` value prints as, for
example, `0x6a09e667`. `Int` output is unchanged.

### Diagnostics

Existing codes keep their meanings. `ORC0204` now names the admitted widths.
The following codes are added:

| Code | Phase | Meaning |
| --- | --- | --- |
| `ORC0108` | Parsing | Operators from different groups, or chained shifts, need parentheses |
| `ORC0211` | Semantics | A bare identifier is not a parameter of the enclosing function |
| `ORC0212` | Semantics | A call names no typed `spec` in the module |
| `ORC0213` | Semantics | A call has the wrong number of arguments |
| `ORC0214` | Semantics | An expression's type differs from the expected type |
| `ORC0215` | Semantics | An operator is not defined for the expected type |
| `ORC0216` | Semantics | A shift or rotation amount is not a literal from 0 through n - 1 |
| `ORC0217` | Semantics | A call cycle makes a function depend on itself |
| `ORC0218` | Semantics | A parameter name repeats within one function |

### Failure and resource behavior

All S3a budgets remain. The parser adds a nesting limit of 256 for unary
operators, parentheses, and call arguments together, reported as `ORC0106`.
Semantic analysis adds a limit of 64 parameters per function and 256 arguments
per call. Evaluation keeps its limit of 1,048,576 steps, counting one step for
each Core node evaluated, and adds a call-depth limit of 256 frames. An `Int`
result whose magnitude would exceed 16,384 significant bits fails closed with
`ORC0301`. An acyclic program can still request exponential work; the step
budget bounds it, and exhausting any budget fails closed without accepted
output.

For identical source bytes, edition, compiler revision, and command, every
diagnostic, Core, value, and output byte is deterministic.

## Alternatives

A conventional precedence table, as in C or Rust, was rejected. C's ordering of
`&` below `==` is a well-known source of bugs, and any table asks the reader to
remember an order that the standards never rely on. Requiring parentheses
across groups costs a few characters and makes every expression's structure
visible.

Checked or trapping word arithmetic was rejected for the specification
stratum. Cryptographic standards define word operations modulo 2^w, and a
specification that trapped on overflow would say something the standard does
not. Overflow checking belongs to implementation-stratum claims, not to the
meaning of `+` on a word.

Arbitrary widths `Word[n]` were deferred. The four widths cover SHA-2,
ChaCha20, Keccak lanes, and byte-oriented code. A later slice can generalize
the width once there is a use and a representation plan for it.

Variable shift and rotation amounts were deferred because out-of-range amounts
need dynamic failure semantics, which no slice has defined. Data-dependent
rotations, as in RC5, would also raise leakage questions that belong with
secrecy labels.

Local bindings and tuples were deferred to keep this slice to one idea. They
are the natural next step, and ChaCha20's quarter round needs both.

## Compatibility and migration

Every source accepted under S3a remains accepted with the same values and
output, with one class of exception. `Word[16]`, `Word[32]`, and `Word[64]`,
which S3a rejected with `ORC0204`, are now accepted. The S3a rejection fixture
that uses `Word[16]` as its example of an unsupported width moves to an
unadmitted width such as `Word[12]`.

`orangec lex` output changes for sources containing `<<`, `>>`, `<<<`, or
`>>>`, which now lex as single tokens. No such source was syntactically valid
before.

Rollback reverts the lexer tokens, grammar, semantics, Core, evaluator, tests,
and normative documents together.

## Semantic and claim effects

This proposal gives exact meanings to parameterized typed specifications,
calls, the listed operators, and the four word types. The supported claim is
limited to deterministic, bounded analysis and evaluation of the documented
fragment at a recorded implementation revision. It establishes no language
soundness, proof, refinement, compilation, cryptographic correctness,
constant-time behavior, compatibility, independent review, or production
readiness. That a function spelled like `big_sigma0` evaluates correctly on
tested inputs is not a claim that it transcribes FIPS 180-4; any such claim
would need the provenance and conformance evidence of a future corpus package.

## TCB, axiom, and proof effects

The lexer, parser, semantic analyzer, Core constructor, evaluator, and their
host dependencies remain engineering trust dependencies. No axiom, theorem,
proof rule, certificate, checker, or solver is introduced.

## Threat, abuse, and leakage effects

The hostile-frontend surface grows to expression nesting, call graphs,
exponential call trees, large intermediate integers, and wider word output.
Fixed nesting, parameter, argument, depth, step, and magnitude budgets, cycle
rejection, and fail-closed evaluation constrain them. No secrecy label or
leakage property is defined; nothing in this slice says whether an expression
would run in constant time on any machine.

## Target and ABI effects

None. `Word[n]` is a mathematical value domain, not a machine representation.

## Standards, errata, and provenance

FIPS 180-4 and RFC 8439 motivate the operator set and appear in examples only.
No standard, erratum, or test vector gains normative authority through this
proposal.

## Dependencies, licenses, and IP

No dependency is added. The Rust standard-library-only product graph remains.
D-018 remains unresolved.

## Conformance, tests, and evidence

Conformance requires at least one positive and one negative case for every
rule above: each token, each grammar form and each mixing error, parameter and
call resolution, each type and width, each operator on each type, each
modular boundary (for example `0xffff_ffff + 1`, `0 - 1`, and `~0` at every
width), shift and rotation amounts at 0, n - 1, and n, cycle detection, every
new diagnostic, every budget, output formats at every width, and repeated
analysis and evaluation. The permanent corpus should include the six SHA-256
functions above evaluated at recorded inputs, with the expected values taken
from an independent computation and recorded as such.

## Operations, release, and recovery

No service, package, key, or release is added. A defect is recovered by a
regression fixture and a normative correction.

## Support and deprecation

The fragment is pre-alpha and best effort under D-022, with no compatibility
promise. Changes require explicit migration notes.

## Unresolved questions

- Whether bindings and tuples should arrive together in the next slice.
- Whether `Word[n]` should generalize to any width from 1 to some bound.
- Whether a later edition should reserve `let`, `if`, and `Bool`, and how that
  interacts with existing identifiers.
- How D-004's accepted strata will place this fragment.

## Decision record

On 2026-09-28 the owner directed that Orange continue its implementation goals
and that development not freeze unless the owner asks. This Draft records the
proposed S3b surface under that direction. It is not accepted and authorizes
nothing by itself.
