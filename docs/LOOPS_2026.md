# Orange 2026 loops specification

Status: proposed S3e semantics under OEP-0008, in owner review; not accepted

Edition: `2026`

Snapshot: 2026-09-28

This document defines slice S3e of Orange 2026: bounded loops over literal
ranges, indices computed from loop indices, updates that replace one element
of an array, and fill literals. It is a delta over the proposed S3d rules in
[`ARRAYS_2026.md`](ARRAYS_2026.md), which are a delta over
[`BINDINGS_2026.md`](BINDINGS_2026.md) and
[`EXPRESSIONS_2026.md`](EXPRESSIONS_2026.md). Everything those documents
define and this one does not mention is unchanged.

It is proposed, not accepted. It becomes normative only when the owner accepts
[OEP-0008](governance/oeps/OEP-0008-orange-2026-bounded-loops.md), which
requires OEP-0007. At that point it replaces the S3d clauses listed in section
12. Until then, the compiler behavior it describes exists so that the proposal
can be reviewed against running code, and it establishes no accepted language
meaning. It accepts no D-004 candidate.

> [!NOTE]
> [`CONDITIONS_2026.md`](CONDITIONS_2026.md), proposed under OEP-0009, extends
> this document with `Bool`, comparisons, Euclidean division, and
> conditionals, and lets a static index divide a loop index.
> [`LOOKUPS_2026.md`](LOOKUPS_2026.md), proposed under OEP-0010, lifts the
> static-index limit of section 13: an index may depend on data, and is proved
> in range from its type. [`MODULES_2026.md`](MODULES_2026.md), proposed under
> OEP-0011, lets a module use others. Every source this document accepts keeps
> its meaning under all three.

The terms **must**, **must not**, and **may** are normative in this document.

## 1. The idea

Standards describe iteration the way mathematicians do. FIPS 180-4 prepares
the SHA-256 message schedule "for t = 16 to 63" and applies 64 rounds "for
t = 0 to 63"; RFC 8439 runs "10 iterations of the double round". Each is a
fold: a state s, a first value, and a step that turns s into its next value
once for each index in a fixed range.

S3e writes exactly that:

```orange
spec schedule(m: Word[32]^16) -> Word[32]^64 {
  let head: Word[32]^64 = for t in 0..16 with w: Word[32]^64 = [0; 64] { w with [t] = m[t] };
  for t in 16..64 with w: Word[32]^64 = head {
    w with [t] = small_sigma1(w[t - 2]) + w[t - 7] + small_sigma0(w[t - 15]) + w[t - 16]
  }
}
```

`for t in 16..64 with w: Word[32]^64 = head { step }` is read "for t from 16
up to 64, with w starting at head, replace w by step". Its value is w after
the last step: the fold s_(k+1) = step(k, s_k) over k = 16, ..., 63. With S3e,
the whole SHA-256 compression function and the whole ChaCha20 encryption of
RFC 8439 section 2.4 are short Orange modules, and `orangec eval` reproduces
the published digests and ciphertext byte for byte.

Four commitments shape every rule below.

- **Every loop is bounded by literals.** A loop's range is two integer
  literals, so its number of steps is known by reading it, and no loop can
  run forever.
- **Every index is proved in range before anything runs.** An index may use
  only integer literals and loop indices with `+`, `-`, and `*`. The checker
  computes the least and greatest value it can take over its loops, and
  rejects the program unless every one selects an element. Evaluation never
  meets an index out of range.
- **Arrays stay values.** `w with [t] = v` is a new array equal to w except at
  position t. Nothing is mutated, so nothing is aliased.
- **Nothing is hidden.** A loop states its index, its accumulator, the
  accumulator's type, and its first value. There is no implicit state and no
  early exit.

## 2. Scope and phase boundary

The phase boundary of `EXPRESSIONS_2026.md` section 2 is unchanged. The
accepted surface grows to: loops with literal bounds; indices that are
expressions over literals and loop indices; updates of one element; and fill
literals `[e; n]`. Every S3d form keeps its meaning. Loops over computed
ranges, early exit, indices that depend on data, and arrays of arrays are not
part of this slice.

## 3. Grammar

No token and no reserved word is added. `for`, `in`, and `with` are recognized
by position, as `let` and `as` are, so a program that uses them as names keeps
its meaning. The S3d productions for primary expressions and indices become:

```text
primary         = IDENTIFIER index? | call index? | "(" expression ")"
                | array | fill | loop ;
index           = "[" INTEGER "]" | "[" expression "]" ;
fill            = "[" expression ";" INTEGER "]" ;
loop            = "for" IDENTIFIER "in" INTEGER ".." INTEGER
                  "with" IDENTIFIER ":" declared_type "=" expression
                  "{" expression "}" ;
update          = operand "with" "[" expression "]" "=" expression ;
```

`operand` is an S3c operand: a prefix expression or a primary. An update
joins an operand as a conversion does.

- `for` starts a loop only when an identifier follows it; otherwise it is a
  name. `in` and `with` are read as words only in a loop's header.
- `with` starts an update only when `[` follows it; otherwise it is a name.
- An update's value extends as far as an expression can, so an update ends
  its operator chain. An update that is itself an operand must be
  parenthesized: `x ^ y with [0] = 1` is the S3c `ORC0108`, whose note says to
  parenthesize the update or the expression it updates. `x with [0] = 1 ^ y`
  replaces element 0 by `1 ^ y`.
- An index that is exactly one `INTEGER` token is an S3d literal index. Any
  other index is an expression.
- A fill literal's length is one `INTEGER` token, spelled as an array type's
  length is.
- A loop's bounds are single `INTEGER` tokens. A name, a sign, or an
  expression in their place is a syntax error.

Each malformed form is `ORC0101` with a note that shows the form's shape. The
syntax tree gains one loop node per loop, holding the spans of `for`, the
index, both bounds, the accumulator, its declared type, the first value, and
the step; one update node per update, holding its base, the span of `with`,
its index, and its value; and one fill node per fill literal, holding its
element and the span of its length. An index node now holds an expression. A
loop's span runs from `for` through `}`; an update's from its base through its
value; a fill's from `[` through `]`.

## 4. Loops

A loop `for i in a..b with s: T = e { f }` is checked against an expected
type `E`:

1. The bounds a and b are decoded as S3a magnitudes, in any S3a spelling. They
   must satisfy 0 ≤ a < b ≤ 65536; otherwise that is `ORC0225`, at the first
   bound above 65536, or else at b.
2. The index name i and then the accumulator name s must each differ from
   every parameter, every binding in scope, every loop index and accumulator
   of an enclosing loop, and, for s, from i. A repeated name is `ORC0219`,
   `duplicate name`, with a secondary span at the earlier declaration.
3. T is resolved as a declared type of `ARRAYS_2026.md` section 4. If it does
   not resolve, its own error is reported and nothing inside the loop is
   checked.
4. If T differs from E, that is `ORC0214` at the loop.
5. The first value e is checked against T. It sees the names in scope where
   the loop stands, but not i or s.
6. If the bounds are valid, the step f is checked against T with i in scope as
   an `Int` and s in scope with type T. Neither name is in scope anywhere
   else.

Loops nest in either position. A loop in a step sees every enclosing loop's
index and accumulator; a loop in a first value sees those of the loops that
enclose it, which do not include the loop whose first value it is. Two loops
whose scopes do not overlap may reuse names.

**Meaning.** A loop evaluates e once, giving s_0. Then, for each k from a up
to b - 1 in increasing order, it evaluates f with i = k and s = s_(k-a),
giving s_(k-a+1). Its value is s_(b-a). A loop always takes at least one
step.

## 5. Indices

An index `b[x]` whose index is one integer literal is the S3d literal index,
with the S3d rules. Any other index is checked against an expected type `E`
as follows:

1. The base's type, and then its kind, are examined exactly as in
   `ARRAYS_2026.md` section 5: an untyped base reports its own error, a
   scalar base is `ORC0224`, and an element type other than `E` is `ORC0214`
   at the index expression. The base is then checked against its own type.
2. The index x is checked as an `Int` expression, with the S3b and S3c rules.
3. The index must be **static**: built only from integer literals, indices of
   enclosing loops, parentheses, prefix `-`, and binary `+`, `-`, and `*`.
   The first part that is anything else (a parameter, a binding, an
   accumulator, a call, a conversion, or another operator) is `ORC0226`,
   reported at that part.
4. The **range** of x is computed by interval arithmetic: a literal k has range
   k..k, a loop index over a..b has range a..b-1, and each operator combines
   its operands' least and greatest values. Each bound is computed
   separately, so `i - i` over 1..5 has range -3..3 although its value is
   always 0. Bounds are exact integers, as `Int` values are, so
   `x[9223372036854775808 - 9223372036854775808]` has range 0..0. If the
   least value is negative or the greatest is not below the array's length
   n, that is `ORC0223` at the index, naming the range. A bound whose
   magnitude would exceed 16,384 significant bits, the limit of an `Int`
   value, is also `ORC0223` at the index.

A static index is therefore in range for every value of every loop index it
uses, and selecting it never fails at run time.

**Meaning.** `b[x]` evaluates b, then x, and selects the element at the
position x.

## 6. Updates

An update `b with [x] = v` is checked against an expected type `E`:

1. The type of the base's first typed leaf is found without reporting, as a
   conversion operand's is. If it is a scalar type, that is `ORC0224` at the
   base, the base is then checked against its own type, and the rest of the
   update is not checked.
2. If E is not an array type, that is `ORC0214` at the update, and its parts
   are not checked.
3. The base b is checked against E.
4. The index x is checked as in section 5, steps 2 through 4, against E.
5. The value v is checked against E's element type.

**Meaning.** An update evaluates b, then x, then v. Its value is the array of
type E equal to b at every position except x, where it holds v. b itself is
unchanged.

## 7. Fill literals

A fill literal `[e; n]` is checked against an expected type `E`:

1. If E is not an array type, that is `ORC0214` at the literal, and its parts
   are not checked.
2. The length n must be a decimal integer from 1 through 256, spelled as an
   array type's length is; otherwise that is `ORC0221` at n. If it is valid
   and differs from E's length, that is `ORC0222` at the literal.
3. The element e is checked against E's element type.

**Meaning.** A fill literal evaluates e once. Its value holds n copies of it.

## 8. Diagnostics

The examination order of `ARRAYS_2026.md` section 7 is unchanged, and the new
forms are examined in the orders of sections 4 through 7. A loop's bounds come
before its names, its names before its type, its first value before its step.
An update's base comes before its index, and its index before its value. Calls
inside first values, steps, indices, and updated values are call edges like
any other, and call cycles are still reported after every function.

The new and widened categories are:

| Code | Phase | Meaning |
| --- | --- | --- |
| `ORC0101` | parse | also a malformed loop, update, or fill literal, or an empty index |
| `ORC0108` | parse | also an update that is an operand without parentheses |
| `ORC0214` | semantic | also a loop, update, or fill literal whose type is not the required one |
| `ORC0219` | semantic | also a loop index or accumulator that repeats a name in scope |
| `ORC0221` | semantic | also a fill literal's length that is not a decimal integer from 1 through 256 |
| `ORC0222` | semantic | also a fill literal whose length differs from its type's |
| `ORC0223` | semantic | also a static index whose range leaves the array or has a bound beyond 16,384 significant bits |
| `ORC0224` | semantic | also an update of a value that is not an array |
| `ORC0225` | semantic | a loop's bounds are not a nonempty range within 0 through 65536 |
| `ORC0226` | semantic | an index uses something other than integer literals and loop indices |

Every other code keeps its meaning.

## 9. Typed Reference Core

The Core of `ARRAYS_2026.md` section 8 gains a loop table per function and
six node kinds:

```text
core_function  = ... loops ;
core_loop      = index_name accumulator_name type start end
                 visible_locals scope step ;
node_kind      = ...
               | select | update | fill
               | fold loop | loop_index loop | accumulator loop ;
```

- Loops are numbered from 0 within each function in the source order of their
  `for` keywords. A loop's `scope` lists the numbers of the loops that enclose
  it, outermost first, and ends with its own. `visible_locals` is the number
  of the function's bindings in scope in its step. Its `step` is a Core
  expression of its own, in postorder.
- A `fold` node consumes one preceding subtree, the first value, and has its
  loop's type. A `loop_index` node has type `Int`, and an `accumulator` node
  its loop's type; each reads its loop, which must enclose the node.
- A `select` node consumes an array subtree and an `Int` subtree and has the
  array's element type. An `update` node consumes an array subtree, an `Int`
  subtree, and an element subtree, and has the array's type. A `fill` node
  consumes one element subtree and has an array type.
- A literal index is still an S3d `index` node.

The Core is still internal and noncanonical, with no encoding, digest, or
proof role, and a function without the S3e forms has exactly its S3d Core.

## 10. Resource limits and failure

The S3d budgets remain. S3e adds or refines the following.

**Parsing.** A loop opens one nesting level for its first value and its step;
an update opens one for its index and its value; an index that is not one
integer literal opens one for its expression; a fill literal is an array
literal and opens one. All draw on the same 64-level budget, and the nesting
message names groups, calls, arrays, indices, loops, updates, and prefix
operators. Each form adds one to the height of its tallest part.

**Loop bounds.** A bound is at most 65536, so one loop takes at most 65536
steps. Loops in steps multiply, and the evaluation step budget of
`EXPRESSIONS_2026.md` bounds the product.

**Semantic events.** A loop is one event, and each bound costs the S3a
decoding events: one for its base prefix and one per significant digit. Each
loop name's uniqueness check is one event, and the accumulator's declared type
costs the events of a declared type. An update is one event. A fill literal is
two: one for the literal and one for its length. An index that is not one
literal is one event, plus the events of its expression. The walk that
computes an index's range consumes no events.

**Core nodes.** A loop is three nodes: its `fold` node, its loop-table entry,
and its accumulator type. Its step's nodes count as any others. Each
`select`, `update`, and `fill` node is one node.

**Evaluation.** A loop costs one step and one more per iteration, beyond its
first value's and every step's. A `loop_index` or `accumulator` read costs
one step, and so does a `select`. An `update` or `fill` of an array of n
elements costs ⌈n/64⌉ steps, as [`LOOKUPS_2026.md`](LOOKUPS_2026.md)
section 8 amends this rule (OEP-0008 first charged n). A loop's steps run
within the call of their function and add no call depth.

Exhausting any budget, and any allocation failure, yields one resource
diagnostic, no Core, and no value line. The deepest sources the limits admit,
including 64 loops nested in steps, 64 loops nested in first values, 32
updates nested in calls, and a 63-level index, must parse, analyze, and
evaluate within 1 MiB of native stack. Inconsistent Core, such as a loop with
no iterations, an index read outside its loop, or an update that stores a
value that does not fit, must stop evaluation with no value.

## 11. Determinism and conformance

The determinism requirement of `EXPRESSIONS_2026.md` section 15 applies
unchanged. The S3e conformance runner
(`compiler/crates/orangec/tests/s3e_conformance.rs`) parses this index and
requires exact agreement with its evidence map, under the same rules as the
S3b, S3c, and S3d runners: every rule needs command-line evidence that runs
twice with identical results, except where the index says unit evidence
alone, and every rule names unit tests declared exactly once in their
sources' test modules.

### S3e conformance rule index

| Rule ID | Clause | Executable obligation | Evidence layer |
| --- | --- | --- | --- |
| `S3E-GRAMMAR-01` | Section 3 | Loops, updates, fill literals, and expression indices parse with exact spans; malformed forms are `ORC0101`, and an ungrouped update is `ORC0108`. | CLI and parser unit |
| `S3E-WORDS-01` | Section 3 | `for`, `in`, and `with` are words only in their positions and remain ordinary names elsewhere. | CLI and parser unit |
| `S3E-LOOP-01` | Section 4 | Loop bounds form a nonempty range within 0 through 65536, and a loop has its accumulator's type; otherwise `ORC0225` or `ORC0214`. | CLI and unit |
| `S3E-SCOPE-01` | Section 4 | Loop names are new, and the index and accumulator are in scope only in the step; otherwise `ORC0219` or `ORC0211`. | CLI and unit |
| `S3E-INDEX-01` | Section 5 | An index is static and its interval range selects an element for every loop index; otherwise `ORC0226` or `ORC0223`. | CLI and unit |
| `S3E-UPDATE-01` | Section 6 | An update applies to an array of the required type, with a static index and a value of the element type. | CLI and unit |
| `S3E-FILL-01` | Section 7 | A fill literal states its type's length and repeats a value of the element type. | CLI and unit |
| `S3E-EVAL-01` | Sections 4 to 7 | Loops fold in increasing index order, indices select, updates replace one element, and fills repeat; SHA-256 and ChaCha20 match their standards. | CLI and unit |
| `S3E-DIAG-01` | Section 8 | Diagnostic order, spans, and non-cascading behavior are exactly as specified. | CLI and unit |
| `S3E-CORE-01` | Section 9 | Core carries the loop table and `fold`, `loop_index`, `accumulator`, `select`, `update`, and `fill` nodes in postorder. | Unit and CLI observation |
| `S3E-RES-BOUND-01` | Section 10 | A loop over 0..65536 is accepted and evaluates; a bound of 65537 is `ORC0225`; nested loops stop at the step budget. | Generated CLI and unit |
| `S3E-RES-NEST-01` | Section 10 | Loops, updates, and expression indices share the 64-level nesting budget and add exactly one to tree height. | Unit |
| `S3E-RES-EVENT-01` | Section 10 | Semantic events and Core nodes for loops, updates, fills, and expression indices are counted exactly. | Unit |
| `S3E-RES-STEP-01` | Section 10 | Loops, reads, selections, updates, and fills cost exactly their steps, and loops add no call depth. | Unit |
| `S3E-RES-STACK-01` | Section 10 | The deepest admitted loops, updates, and indices fit in 1 MiB of native stack. | Unit |
| `S3E-RES-FAIL-01` | Section 10 | Allocation failure, foreign syntax trees, and inconsistent Core yield no partial tree, Core, or value. | Unit |
| `S3E-COMPAT-01` | Section 12 | S3d programs keep their meaning and messages, except as section 12 lists. | CLI and unit |
| `S3E-DETERMINISM-01` | Section 11 | Repeated identical inputs produce identical status, diagnostics, and output bytes. | CLI and unit |

## 12. Relationship to S3d

When OEP-0008 is accepted, this document replaces these clauses of
`ARRAYS_2026.md`:

- the `primary` and `index` productions and the bullets on indices, by
  section 3;
- the index rules of section 5, extended by section 5 here;
- the meaning of section 6, extended by sections 4 through 7 here;
- the diagnostic, Core, and resource sections, extended by sections 8, 9, and
  10; and
- the exclusion of loops, variable indices, and updates from its non-claims.

Every source that S3d accepts is accepted by S3e with the same Core values
and the same output bytes. `for` before an identifier, `with` before `[`, and
`;` inside an array literal were never valid. A source that S3d rejects gets
the same diagnostics, with these exceptions:

- an index that is not one integer literal, such as `x[i]`, `x[-1]`, or
  `x[0 + 1]`, now parses, and analysis reports it by section 5;
- an empty index `x[]` now reads "expected an index after `[`";
- the note on an out-of-range literal index now reads "a literal index must be
  less than the array's length";
- the note on a missing expression now lists loops among the forms an
  expression can take; and
- the expression-nesting message now names indices, loops, and updates.

## 13. Explicit non-claims and future work

This slice defines no loops over computed or unbounded ranges, no early exit,
no loop with more than one accumulator other than through an array, no
indices that depend on data, no slices, no arrays of arrays, no mutation, and
none of the exclusions of `ARRAYS_2026.md` section 12 that this document does
not lift.

Static indices are a deliberate limit, not an accident. An index that depends
on data is the classic source of a timing channel in table-driven
cryptography, and in Orange it is simply not expressible. The ChaCha20
quarter round still names its four positions literally, because the
positions of a quarter round are parameters of the round, and an index built
from parameters is not static. A later slice may add index parameters that are
themselves static, such as a quarter round over positions known at each call.

A fixture that reproduces a standard's example value is not thereby a verified
transcription of that standard. No statement here says whether a loop, an
index, or any expression would run in constant time on any machine; that a
data-dependent index cannot be written is a property of the source language,
not of any compiled code. Tests establish the tested behavior of one
implementation at one revision; they do not prove semantic soundness,
completeness, or implementation independence.

Adding any excluded behavior requires a later accepted decision, a normative
specification, positive and negative conformance cases, resource analysis, and
migration review.
