# Orange 2026 conditions specification

Status: proposed S3f semantics under OEP-0009, in owner review; not accepted

Edition: `2026`

Snapshot: 2026-09-29

This document defines slice S3f of Orange 2026: the type `Bool` and its
values, comparisons, the logical operators, Euclidean division and remainder,
and conditionals that choose one of two values. It is a delta over the proposed
S3e rules in [`LOOPS_2026.md`](LOOPS_2026.md), which are a delta over
[`ARRAYS_2026.md`](ARRAYS_2026.md), [`BINDINGS_2026.md`](BINDINGS_2026.md),
and [`EXPRESSIONS_2026.md`](EXPRESSIONS_2026.md). Everything those documents
define and this one does not mention is unchanged.

It is proposed, not accepted. It becomes normative only when the owner accepts
[OEP-0009](governance/oeps/OEP-0009-orange-2026-conditions.md), which
requires OEP-0008. At that point it replaces the S3e clauses listed in section
14. Until then, the compiler behavior it describes exists so that the proposal
can be reviewed against running code, and it establishes no accepted language
meaning. It accepts no D-004 candidate.

The terms **must**, **must not**, and **may** are normative in this document.

## 1. The idea

Public-key cryptography lives in the integers modulo a prime. RFC 7748 defines
X25519 over 2^255 - 19 and RFC 8439 defines Poly1305 over 2^130 - 5; each
reduces "mod p" after every product, reads the bits of a scalar with
"k_t = (k >> t) & 1", and swaps two points "if swap". A specification of
either needs three things S3e lacks: a remainder, a truth value, and a choice.

S3f writes them the way the standards do:

```orange
spec rung(x1: Int, s: Int^4, set: Bool) -> Int^4 {
  if set { swap(ladder(x1, swap(s))) } else { ladder(x1, s) }
}

spec absorb(a: Int, r: Int, block: Int) -> Int { ((a + block) * r) % prime() }
```

With S3f, X25519 of RFC 7748, Poly1305 of RFC 8439 section 2.5, and the
ChaCha20-Poly1305 AEAD of section 2.8 are short Orange modules, and
`orangec eval` reproduces the published test vector, tag, and sealed
ciphertext byte for byte.

Four commitments shape every rule below.

- **Truth is its own type.** `Bool` is not a number and not a word. It has two
  values, `true` and `false`, and five operators: `!`, `&&`, `||`, `==`, and
  `!=`. Nothing converts to or from it; a number becomes a truth value only
  by a comparison, and a truth value becomes a number only by a conditional.
- **Division is total and Euclidean.** `a / b` and `a % b` always have a
  value. For integers the remainder is never negative, so `a % p` is always
  the canonical representative a cryptographer writes. Dividing by zero is
  defined, not an error: `x / 0` is 0 and `x % 0` is x.
- **A conditional always has a value.** Every `if` has an `else`, both
  branches have the same type, and exactly one of them is evaluated.
- **Every index is still proved in range.** An index may now divide by
  literals and loop indices, and the proof of section 9 follows Euclidean
  division exactly, so `k[(254 - i) / 8]` over 0..255 is checked before
  anything runs.

## 2. Scope and phase boundary

The phase boundary of `EXPRESSIONS_2026.md` section 2 is unchanged. The
accepted surface grows to: the type `Bool` and arrays of it; the literals
`true` and `false`; the comparisons `==`, `!=`, `<`, `<=`, `>`, and `>=`; the
logical operators `!`, `&&`, and `||`; division `/` and remainder `%`;
conditionals with `else if` chains; and division in static indices. Every S3e
form keeps its meaning. Short-circuit evaluation, conditionals without
`else`, pattern matching, signed words, and conversions to or from `Bool` are
not part of this slice.

## 3. Grammar

No token and no reserved word is added. The S2 lexer already produces `==`,
`!=`, `<`, `<=`, `>`, `>=`, `&&`, `||`, `!`, `/`, and `%`, and no earlier
slice accepted them in an expression. `if`, `else`, `true`, and `false` are
recognized by position or by scope, as `let`, `as`, `for`, `in`, and `with`
are, so a program that uses them as names keeps its meaning. The S3e
productions for prefix operators and primary expressions become:

```text
prefix          = ( "-" | "~" | "!" ) operand ;
primary         = IDENTIFIER index? | call index? | "(" expression ")"
                | array | fill | loop | conditional ;
conditional     = "if" expression "{" expression "}" "else" alternative ;
alternative     = "{" expression "}" | conditional ;
```

The binary operators gain four groups beside those of `EXPRESSIONS_2026.md`
section 5:

| Group | Operators | Shape |
| --- | --- | --- |
| Comparison | `==` `!=` `<` `<=` `>` `>=` | exactly two operands, not associative |
| Logical and | `&&` | chains of `&&` associate to the left |
| Logical or | `\|\|` | chains of `\|\|` associate to the left |
| Division | `/` `%` | exactly two operands, not associative |

The grouping rule is unchanged: a binary operator from a different group at
the same level is `ORC0108` at that operator, and so is a second comparison
or a second division. There is still no precedence between groups, so
`a + b < c`, `a < b && c < d`, `a * b / c`, and `a && b || c` are rejected,
and `(a + b) < c`, `(a < b) && (c < d)`, `(a * b) / c`, and
`(a && b) || c` are accepted. Each `ORC0108` note says what to do: a second
comparison is joined with `&&` or `||`, a second division parenthesizes one
operand, and operators of different groups parenthesize the part that
applies first.

- `if` starts a conditional only where its next token could begin a
  condition and the conditional can complete:
  - before an identifier, unless the identifier is the word `as`, or the word
    `with` followed by `[`;
  - before an integer literal, `!`, or `~`; and
  - before `(`, `-`, or `[`, only when a `}` that returns to the depth of that
    token is followed directly by `else`, before a `;` or `,` at that depth or
    a closing delimiter that leaves it.

  Everywhere else `if` is a name, so `if(x)`, `if - x`, `if[0]`, `if as Int`,
  and `if with [0] = 1` keep their S3e meaning.
- `else` is read as a word only after the `}` that closes a conditional's
  value. After `else`, `{` begins the last value and `if` begins another arm;
  anything else is `ORC0101`, "expected `{` or `if` after `else`".
- A missing `else` is `ORC0101`, "expected `else` and the value when the
  condition is false", with the note "every `if` has an `else`, so that a
  conditional always has a value". A value that is not one expression is
  `ORC0101`, "expected `}` after the value". Before `(`, an `if` whose brace
  group is not followed by `else` is read as a call, so `if (a) { b }` is the
  body error "expected `}` after the body expression".
- `true` and `false` are identifiers; section 4 gives them their meaning.

The syntax tree gains one conditional node per `if ... else` chain, holding
one arm per `if` (the span of `if`, the condition, and the value) and the
final value; one prefix node for `!`; and binary nodes for the new operators.
A conditional's span runs from its first `if` through its last `}`.

## 4. The type `Bool`

`Bool` is a scalar type. It may be written wherever a type may: as a
parameter, result, binding, or accumulator type, and as an array's element
type, such as `Bool^4`. The admitted scalar types are now `Int`, `Bool`,
`Word[8]`, `Word[16]`, `Word[32]`, and `Word[64]`.

A name `true` or `false` is resolved as a name is in S3e: first the
parameters, then the bindings in scope, then the indices and accumulators of
the enclosing loops. Only if none of them has that spelling does the name
denote the `Bool` value of its spelling. A binding later in the body does not
hide it, because a binding's scope has not started there. Where a function
declares a parameter or binding named `true`, the name means that parameter
or binding.

Typing:

- A `Bool` value where another type is required is `ORC0214`, and so is any
  other typed value where `Bool` is required: "`x` has type `Int`, but `Bool`
  is required here".
- An integer literal cannot have type `Bool` (`ORC0214`), with the note "the
  `Bool` values are written `true` and `false`".
- The operators defined for `Bool` are `!`, `&&`, `||`, `==`, and `!=`. Any
  other operator whose operands are `Bool` is `ORC0215` at the operator, such
  as "`+` is not defined for `Bool`" or "prefix `-` is not defined for
  `Bool`", with a note that lists the five.
- `as` does not convert to or from `Bool` (`ORC0215` at `as`), with the note
  "choose a number with a conditional, such as `if b { 1 } else { 0 }`, or
  compare a number, such as `x != 0`".

**Meaning.** `Bool` has exactly the two values true and false. Its values are
displayed as `true` and `false`.

## 5. Comparisons

A comparison `a op b` is checked against an expected type `E`:

1. If E is not `Bool`, that is `ORC0214` at the comparison, "a comparison
   gives `Bool`, but `E` is required here", with the note "a conditional
   `if c { a } else { b }` chooses a value by a `Bool`". Its operands are
   still checked, so their own errors are reported.
2. The **operand type** T is the type of the first typed leaf of a, or, if a
   has none, of b. A typed leaf is found as a conversion operand's is in
   `BINDINGS_2026.md`, with two additions: a comparison is itself a leaf of
   type `Bool`, and a conditional's leaf is the first typed leaf among its
   values. The conditions of conditionals are not searched. If neither
   operand has a typed leaf, that is `ORC0227` at the comparison, "the
   operands of `op` have no type of their own", with the note "compare with a
   typed operand, such as a name, or give the literal a type with a `let`
   binding". If the leaf has no type, the leaf's own error is reported.
3. `==` and `!=` are defined for every scalar type. `<`, `<=`, `>`, and `>=`
   are defined for `Int` and the word types. Any other pairing, including any
   comparison of arrays, is `ORC0215` at the operator: "`<` is not defined for
   `Bool`", with the note "`Bool` values are compared with `==` and `!=`; they
   have no order", or "`==` is not defined for `Word[8]^2`", with the note
   "compare elements, such as `x[0] == y[0]`".
4. a and then b are checked against T.

**Meaning.** A comparison evaluates a, then b, and gives true exactly when the
relation holds. Integers compare by value. Words compare as the unsigned
integers they denote, so `0x80 > 0x7f` is true at `Word[8]`. `Bool` values
compare only for equality.

## 6. Logical operators

`!a` is checked against `E`, and so are both operands of `a && b` and
`a || b`. Each is defined only when E is `Bool`; otherwise that is `ORC0215`
at the operator, such as "`&&` is not defined for `Word[8]`", with the note
"`&&` and `||` apply to `Bool` values; `&` and `|` are the bitwise operators
on words", or "prefix `!` is not defined for `Word[8]`", with the note "`!`
negates a `Bool`; `~` is the bitwise complement of a word". The bitwise
operators `&`, `|`, `^`, and `~` are not defined for `Bool`.

**Meaning.** `!a` is the negation of a. `a && b` evaluates a, then b, and is
true exactly when both are. `a || b` evaluates a, then b, and is true exactly
when either is. Both operands are always evaluated. There is no
short-circuit: a program chooses to skip work only with a conditional, which
states the choice.

## 7. Division and remainder

`a / b` and `a % b` are checked against `E` like the other arithmetic
operators. They are defined for `Int` and the word types, and for no other
type (`ORC0215`).

**Meaning.** Division evaluates a, then b.

- For `Int`, with b ≠ 0, `a / b` and `a % b` are the unique integers q and r
  with a = b · q + r and 0 ≤ r < |b|. This is Euclidean division: the
  remainder is never negative, whatever the signs of a and b. So `7 / 2` is
  3 and `7 % 2` is 1; `-7 / 2` is -4 and `-7 % 2` is 1; `7 / -2` is -3 and
  `7 % -2` is 1; `-7 / -2` is 4 and `-7 % -2` is 1.
- For `Word[n]`, with b ≠ 0, `a / b` and `a % b` are the quotient rounded
  down and the remainder of the unsigned integers a and b. Both fit in n
  bits.
- For every type, `a / 0` is 0 and `a % 0` is a. With these values
  a = b · (a / b) + a % b holds for every a and b, and division never fails.

Division never enlarges an `Int`: |a / b| ≤ |a|, and a % b is below |b|, or
is a when b is 0.

## 8. Conditionals

A conditional `if c { v } else { w }` is checked against an expected type
`E`. An `else if` chain `if c0 { v0 } else if c1 { v1 } ... else { w }` is one
conditional per `if`: each later arm is the `else` value of the arm before it.
Arms are checked in source order:

1. The arm's condition is checked against `Bool`.
2. The arm's value is checked against E.

Then the final value w is checked against E. The conditional has type E. Its
conditions and values see every name in scope where it stands, and a
conditional introduces no name.

Conditionals are numbered from 0 within each function in the source order of
their `if` keywords, counting every arm of a chain and every conditional
nested in a condition or a value.

**Meaning.** A conditional evaluates its condition. If it is true, the
conditional's value is its first value; otherwise it is its `else` value.
**Only the chosen branch is evaluated.** The other has no effect: it costs
no evaluation steps and cannot exhaust a budget. A chain therefore evaluates
its conditions in order up to the first true one, and the value of that arm,
or the final value if none is true.

A conditional is a choice between two mathematical values. It says nothing
about how a machine would make the choice, and in particular it is not a
claim that the choice takes the same time either way (section 15).

## 9. Indices

The static indices of `LOOPS_2026.md` section 5 may now also use `/` and
`%`. An index is static when it is built only from integer literals, indices
of enclosing loops, parentheses, prefix `-`, and binary `+`, `-`, `*`, `/`,
and `%`; anything else is `ORC0226`, whose note now lists the five operators.

The range of `x / y` and of `x % y` is computed from the ranges of x and y
under the rules of section 7, considering the positive, zero, and negative
divisors in y's range separately and joining their ranges:

- For positive divisors d from dl through dh, the quotient's least and
  greatest values are among x's bounds divided by dl and by dh. The
  remainder's range is exact when there is one divisor and x's range lies
  within one quotient; otherwise it is 0 through dh - 1, and at most x's
  greatest value when x cannot be negative.
- A zero divisor contributes 0 for `/` and x's range for `%`.
- For negative divisors, x / d = -(x / -d) and x % d = x % -d, so their
  ranges follow from those of the positive divisors -dh through -dl.

So `(254 - i) / 8` over 0..255 has range 0 through 31, `(254 - i) % 8` has
range 0 through 7, `(i + 1) % 5` over 0..5 has range 0 through 4, and
`i % 0` over 0..8 has range 0 through 7. As in S3e, each bound is computed
separately, so a range may be wider than the values an index takes; the
`ORC0223` error names the computed range.

## 10. Diagnostics

The examination order of `LOOPS_2026.md` section 8 is unchanged, and the new
forms are examined in the orders of sections 4 through 8. A comparison's
result type comes before its operand type, its operand type before the
operator's definition, and its left operand before its right. A conditional's
arms come in source order, each condition before its value, and the final
value last. Calls in conditions and in both branches are call edges like any
other, whichever branch evaluation would take, and call cycles are still
reported after every function.

The new and widened categories are:

| Code | Phase | Meaning |
| --- | --- | --- |
| `ORC0101` | parse | also a malformed conditional |
| `ORC0108` | parse | also a second comparison or division, or operators of different groups, including the new groups, without parentheses |
| `ORC0203` | semantic | no longer reported for `Bool`, which is admitted |
| `ORC0214` | semantic | also a condition that is not `Bool`, a comparison where `Bool` is not required, an integer literal where `Bool` is required, and `Bool` values where another type is required |
| `ORC0215` | semantic | also an operator not defined for `Bool`, a logical operator on another type, an ordering of `Bool` values, a comparison of arrays, and a conversion to or from `Bool` |
| `ORC0223` | semantic | also a static index with `/` or `%` whose range leaves the array |
| `ORC0226` | semantic | also an index that divides by something other than integer literals and loop indices |
| `ORC0227` | semantic | the operands of a comparison have no type of their own |

Every other code keeps its meaning.

## 11. Typed Reference Core

The Core of `LOOPS_2026.md` section 9 gains the type `Bool`, a conditional
table per function, and two node kinds:

```text
core_type        = ... | bool ;
core_function    = ... loops conditionals ;
core_conditional = type visible_locals scope then_branch else_branch ;
node_kind        = ...
                 | compare operator operand_type | choose conditional ;
```

- A literal node may hold a `Bool` value.
- A `compare` node consumes two subtrees of its operand type and has type
  `Bool`. The operators `!`, `&&`, `||`, `/`, and `%` are prefix and binary
  operator nodes of their operands' type.
- A `choose` node consumes one `Bool` subtree, the condition, and has its
  conditional's type. It reads its conditional's entry, whose `then_branch`
  and `else_branch` are Core expressions of their own, in postorder, of the
  conditional's type. `visible_locals` is the number of the function's
  bindings in scope in the branches, and `scope` lists the loops whose index
  and accumulator the branches may read, outermost first.
- For an `else if` chain, the first arm's condition nodes and its `choose`
  node are part of the enclosing expression, and each later arm's condition
  nodes and `choose` node form the `else_branch` of the arm before it.

The Core is still internal and noncanonical, with no encoding, digest, or
proof role, and a function without the S3f forms has exactly its S3e Core.

## 12. Resource limits and failure

The S3e budgets remain. S3f adds or refines the following.

**Parsing.** A conditional opens one nesting level for all its conditions and
values, however long its `else if` chain; a `!` opens one as the other prefix
operators do. All draw on the same 64-level budget, and the nesting message
names groups, calls, arrays, indices, loops, conditionals, updates, and prefix
operators. A conditional adds one to the height of its tallest part, so a
chain of any length has the height of its tallest arm plus one. Chains are
parsed and checked by iteration, not recursion.

**Semantic events.** Each arm of a conditional, that is each `if`, is one
event. A comparison, `!`, `&&`, `||`, `/`, and `%` are operators and cost one
event each, as every operator does. `true` and `false` are names and cost one
event each. The walk that finds an operand type or computes an index range
consumes no events.

**Core nodes.** Each arm of a conditional is one node for its table entry, and
its `choose` node is another. Every node of every branch counts as any other.
Each `compare` node is one node.

**Evaluation.** The step costs are:

| Core node | Steps |
| --- | --- |
| `Bool` literal, `!a`, `a && b`, `a \|\| b` | 1 |
| comparison of `Bool` or word values | 1 |
| comparison of `Int` values | 1 + max(d(a), d(b)) |
| `a / b`, `a % b` on `Word[n]` | 1 |
| `a / b`, `a % b` on `Int` | 1 + d(a) · max(d(b), 1) |
| `choose` | 1 |

Here d(x) is the number of 32-bit limbs in the magnitude of x, as in
`EXPRESSIONS_2026.md` section 14. A conditional costs its condition's steps,
one to choose, and the chosen branch's steps. The branch runs within the call
of its function and adds no call depth.

Exhausting any budget, and any allocation failure, yields one resource
diagnostic, no Core, and no value line. The deepest sources the limits admit,
including 64 conditionals nested in values, in `else` values, and in
conditions, and an `else if` chain of 4096 arms every one of which is taken,
must parse, analyze, and evaluate within 1 MiB of native stack. Inconsistent
Core, such as a `choose` whose conditional does not exist, a branch that
reads a binding or loop outside its scope, or a comparison of operands whose
types disagree, must stop evaluation with no value.

## 13. Determinism and conformance

The determinism requirement of `EXPRESSIONS_2026.md` section 15 applies
unchanged. The S3f conformance runner
(`compiler/crates/orangec/tests/s3f_conformance.rs`) parses this index and
requires exact agreement with its evidence map, under the same rules as the
S3b through S3e runners: every rule needs command-line evidence that runs
twice with identical results, except where the index says unit evidence
alone, and every rule names unit tests declared exactly once in their
sources' test modules.

### S3f conformance rule index

| Rule ID | Clause | Executable obligation | Evidence layer |
| --- | --- | --- | --- |
| `S3F-GRAMMAR-01` | Section 3 | Conditionals, comparisons, logical operators, and divisions parse with exact spans; malformed conditionals are `ORC0101`, and a second comparison or division or a mixed group is `ORC0108`. | CLI and parser unit |
| `S3F-WORDS-01` | Sections 3 and 4 | `if` and `else` are words only in their positions, and `true` and `false` are values only where no name of their spelling is in scope. | CLI and parser unit |
| `S3F-BOOL-01` | Section 4 | `Bool` is admitted, has only its five operators, takes no integer literal, and has no conversion; otherwise `ORC0214` or `ORC0215`. | CLI and unit |
| `S3F-COMPARE-01` | Section 5 | A comparison gives `Bool`, takes its operand type from its first typed leaf, and orders only numbers; otherwise `ORC0214`, `ORC0227`, or `ORC0215`. | CLI and unit |
| `S3F-LOGIC-01` | Section 6 | `!`, `&&`, and `\|\|` apply to `Bool` and evaluate every operand. | CLI and unit |
| `S3F-DIVISION-01` | Section 7 | `Int` division is Euclidean, word division is unsigned, and division by zero gives 0 and the dividend. | CLI and unit |
| `S3F-COND-01` | Section 8 | A conditional has `Bool` conditions and values of the required type, a chain is one conditional per arm, and only the chosen branch is evaluated. | CLI and unit |
| `S3F-INDEX-01` | Section 9 | Static indices may divide, and their ranges follow Euclidean division and its total rules; otherwise `ORC0226` or `ORC0223`. | CLI and unit |
| `S3F-EVAL-01` | Sections 4 to 8 | Values are exact; Poly1305, ChaCha20-Poly1305, and X25519 match their standards. | CLI and unit |
| `S3F-DIAG-01` | Section 10 | Diagnostic order, spans, and non-cascading behavior are exactly as specified. | CLI and unit |
| `S3F-CORE-01` | Section 11 | Core carries `Bool`, the conditional table, and `compare` and `choose` nodes in postorder, numbered in source order. | Unit and CLI observation |
| `S3F-RES-NEST-01` | Section 12 | A conditional opens one nesting level for all its parts, a chain opens no more, and each adds exactly one to tree height. | Unit |
| `S3F-RES-EVENT-01` | Section 12 | Semantic events and Core nodes for conditionals and comparisons are counted exactly. | Unit |
| `S3F-RES-STEP-01` | Section 12 | Comparisons, logic, division, and choices cost exactly their steps, an untaken branch costs nothing, and branches add no call depth. | Generated CLI and unit |
| `S3F-RES-STACK-01` | Section 12 | The deepest admitted conditionals and the longest admitted chains fit in 1 MiB of native stack. | Unit |
| `S3F-RES-FAIL-01` | Section 12 | Allocation failure, foreign syntax trees, and inconsistent Core yield no partial tree, Core, or value. | Unit |
| `S3F-COMPAT-01` | Section 14 | S3e programs keep their meaning and messages, except as section 14 lists. | CLI and unit |
| `S3F-DETERMINISM-01` | Section 13 | Repeated identical inputs produce identical status, diagnostics, and output bytes. | CLI and unit |

## 14. Relationship to S3e

When OEP-0009 is accepted, this document replaces these clauses of
`LOOPS_2026.md` and the documents it extends:

- the `prefix` and `primary` productions, by section 3;
- the operator groups of `EXPRESSIONS_2026.md` section 5, extended by the four
  groups of section 3;
- the admitted types, extended by section 4;
- the static indices of section 5, extended by section 9;
- the diagnostic, Core, and resource sections, extended by sections 10, 11,
  and 12; and
- the exclusion of booleans and conditionals from the non-claims of
  `BINDINGS_2026.md` and its successors.

Every source that S3e accepts is accepted by S3f with the same Core values and
the same output bytes. The new operators, `if` before an identifier, an
integer literal, `!`, or `~`, and `if` before a brace group followed by `else`
were never valid. A source that S3e rejects gets the same diagnostics, with
these exceptions:

- `Bool` is an admitted type, and `true` and `false` are values where no name
  of their spelling is in scope, so sources that used them as an unsupported
  type or an unknown name may now be accepted or reported by these rules;
- the new operators and conditionals parse, and analysis reports them by
  sections 4 through 9;
- the notes on unsupported types, on operators applied to arrays, and on
  static indices, and the note on a missing expression, now list `Bool`,
  `Bool` values, `/` and `%`, and conditionals;
- the expression-nesting message now names conditionals; and
- after a syntax error inside a loop's step or a conditional's value, parsing
  recovers past the braces still open in the body, so the `}` that closes the
  step is no longer taken for the end of the function, and the errors that
  followed from it, such as `ORC0104` at the next function, are no longer
  reported.

## 15. Explicit non-claims and future work

This slice defines no short-circuit operators, no conditional without `else`,
no pattern matching, no signed words or signed comparison of words, no
conversions to or from `Bool`, no ordering of `Bool` values, no comparison of
whole arrays, no modular-integer type, and none of the exclusions of
`LOOPS_2026.md` section 13 that this document does not lift.

A conditional is a mathematical choice, not a machine branch. X25519 is
specified in RFC 7748 with a constant-time conditional swap, and the fixture
writes that swap as a conditional because its meaning, one of two values, is
what the specification needs. Nothing here says that a conditional, a
comparison, or a division would run in constant time on any machine, and no
statement here concerns secrecy or leakage. Code generation, and any
constant-time property of generated code, belongs to later decisions.

Exact `Int` arithmetic with `%` is the plainest way to write a prime field,
and it is how the fixtures write GF(2^255 - 19) and GF(2^130 - 5). A later
slice may add a type of integers modulo a declared prime, whose operations
reduce automatically and whose values cannot leave the field.

A fixture that reproduces a standard's example value is not thereby a verified
transcription of that standard. Tests establish the tested behavior of one
implementation at one revision; they do not prove semantic soundness,
completeness, or implementation independence.

Adding any excluded behavior requires a later accepted decision, a normative
specification, positive and negative conformance cases, resource analysis, and
migration review.
