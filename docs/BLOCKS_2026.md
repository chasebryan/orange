# Orange 2026 blocks specification

Status: proposed S3j semantics under OEP-0013, in owner review; not accepted

Edition: `2026`

Snapshot: 2026-09-30

This document defines slice S3j of Orange 2026: `let` bindings at the start
of a loop's step and of each branch of a conditional, which this document
calls **blocks**. It is a delta over the proposed S3i rules in
[`MODULAR_2026.md`](MODULAR_2026.md), which are a delta over
[`MODULES_2026.md`](MODULES_2026.md) and the documents it extends.
Everything those documents define and this one does not mention is
unchanged.

It is proposed, not accepted. It becomes normative only when the owner accepts
[OEP-0013](governance/oeps/OEP-0013-orange-2026-blocks.md), which requires
OEP-0012. At that point it replaces the S3i clauses listed in section 11.
Until then, the compiler behavior it describes exists so that the proposal can
be reviewed against running code, and it establishes no accepted language
meaning. It accepts no D-004 candidate.

> [!NOTE]
> [`TUPLES_2026.md`](TUPLES_2026.md), proposed under OEP-0014, extends this
> document with tuples and tuple patterns, which lift the exclusion of loops
> with more than one accumulator and of destructuring in section 12: a block's
> binding may name each element of a tuple, and a loop may carry several
> accumulators, and [`BYTES_2026.md`](BYTES_2026.md), proposed under OEP-0015,
> adds byte strings, `++`, and slices, so that a step may take one block of a
> message as `m[16 * j..16 * j + 16]`, and [`SIZES_2026.md`](SIZES_2026.md),
> proposed under OEP-0016, lets a loop's bounds be sizes, as `for b in
> 0..blocks`, and [`ORDER_2026.md`](ORDER_2026.md), proposed under OEP-0017,
> lets a step read a block of a message as words in one conversion, as `m[16 *
> j..16 * j + 16] as little Word[64]^2`. Every source this document accepts
> keeps its meaning under all four.

The terms **must**, **must not**, and **may** are normative in this document.

## 1. The idea

A standard writes a round as a short list of named values. FIPS 180-4
section 6.2.2 names the working variables a through h and the temporary words
T1 and T2 inside each round of SHA-256; RFC 7748 section 5 names A, AA, B,
BB, E, C, D, DA, and CB inside each step of the Montgomery ladder. Through
S3i an Orange `let` could stand only at the start of a function's body, so a
round's names had to move into a helper function that took the round's state
as parameters, or the round had to repeat its subexpressions.

S3j lets a loop's step and each branch of a conditional begin with `let`
bindings, exactly as a body does, so the round reads as its standard does:

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

A step's bindings are evaluated afresh at every step, and a branch's only when
the branch is chosen. Each is in scope for the bindings after it and for its
block's value, and nowhere else. Orange still has no shadowing: a block's
binding repeats no name in scope.

## 2. Scope and phase boundary

The phase boundary of `EXPRESSIONS_2026.md` section 2, as S3i extends it, is
unchanged. Every S3i source keeps its meaning (section 11).

## 3. Grammar

No token and no reserved word is added. The loop and conditional productions
of `LOOPS_2026.md` and `CONDITIONS_2026.md` section 3 become:

```text
loop        = "for" IDENTIFIER "in" INTEGER ".." INTEGER
              "with" IDENTIFIER ":" declared_type "=" expression block ;
conditional = "if" expression block "else" alternative ;
alternative = block | conditional ;
block       = "{" binding* expression "}" ;
binding     = "let" IDENTIFIER ":" declared_type "=" expression ";" ;
```

`let` is recognized by position, as in `BINDINGS_2026.md` section 3: it
starts a binding when it is the first token of a block item and the next
token is an identifier. Anywhere else it is a name. A function's body keeps
its S3c production.

- A block that ends with a binding is `ORC0101`, "expected a value after the
  last binding", at the `}`.
- A binding without its `:` and type, its `=`, or its `;` is the S3c
  `ORC0101` for that part. The note of a missing `;` in a block is "each
  binding ends with `;`; the block's last item is its value".
- A value followed by anything but `}` is `ORC0101`, "expected `}` after the
  loop's step" or "expected `}` after the value", as in S3e and S3f.

The note of these diagnostics describes the block: for a step, "a loop's step
holds `let` bindings, if any, and then the expression that gives the
accumulator's next value"; for a branch, "each branch of a conditional holds
`let` bindings, if any, and then its value".

A block declares at most 256 bindings; a 257th is `ORC0106`, "a block
declares more than 256 bindings", at that binding. A block's bindings, their
types, and their values are parsed at the nesting level of the block's value,
one deeper than the loop or conditional, and each counts toward the height of
the loop or conditional: that is one level above the tallest of its parts,
each binding's type and value included.

The syntax tree's loop node gains the bindings of its step, each arm the
bindings of its branch, and the conditional the bindings of its final `else`
branch. A binding node is the S3c binding node; its span runs from `let`
through its `;`.

## 4. Names and scope

A block's binding is **in scope** from the end of its own `;` to the end of
its block: in the values of the block's later bindings and in the block's
value, including every loop and conditional nested in them. It is not in scope
in its own value, in its loop's first value or header, in its arm's
condition, in any other branch, or outside its loop or conditional.

A block's binding must have a name different from every parameter, every
binding of the body in scope, every binding in scope of an enclosing block,
every earlier binding of its block, and every loop index and accumulator in
scope. A loop's index and accumulator must likewise differ from the block
bindings in scope where the loop stands. A repeated name is `ORC0219`,
"duplicate name", with a secondary span at the earlier name and the note
"each parameter, binding, loop index, and accumulator in scope has its own
name; Orange has no shadowing". Names whose scopes do not overlap may repeat:
two branches of one conditional, two separate steps, or a block's binding and
a body binding after its loop may share a name.

A bare name refers, in this order, to a parameter, a binding of the body in
scope, a block binding in scope (the innermost block first), a loop index or
accumulator in scope, and then `true` or `false`. Any other name is
`ORC0211`:

- a name that matches a binding of a block being checked that is not yet in
  scope is "used before it is bound", citing that binding, with the note "a
  binding is in scope after its own `;`, for the bindings that follow it and
  the value of its step or branch";
- a name that matches a binding of a step or branch already checked in the
  function is "`t` is not in scope here", with a secondary span "a binding of
  this name is here" at the most recently checked such binding and the note
  "a binding of a loop's step or a branch is in scope only within that step
  or branch"; and
- otherwise the S3c through S3i messages apply, and a block's bindings count
  as bindings of the function for "is not a parameter or binding of".

## 5. Typing

Each binding's declared type is resolved as a declared type of
`ARRAYS_2026.md` section 4, as S3i extends it, and the binding's value is
checked against it. If the type does not resolve, the error is reported once
where the type is written, the value is not checked, and uses of the binding
are not reported again, exactly as for a body's binding in `BINDINGS_2026.md`
section 6. A step's value is checked against the loop's accumulator type, and
a branch's value against the conditional's expected type.

Parts are checked in source order. A loop checks its bounds, names, type, and
first value as in S3e, then its step's bindings in order, then the step's
value. Each arm of a conditional checks its condition, then its branch's
bindings, then its value, and the final `else` branch checks its bindings and
value last. Every part is checked even when an earlier part has an error,
except the value of a binding whose type does not resolve.

**Typed leaves.** Where the type of a conditional is needed before its
branches are checked, as for the operand of a conversion or a comparison or
the range of an index, it comes from the first branch whose first typed leaf
does not name one of that branch's own bindings, directly or through indices;
those bindings are not in scope where the type is needed. When no branch gives
a type, a conversion is `ORC0220` and a comparison `ORC0227`, as in S3c and
S3f, labeled "a branch's own bindings are not in scope outside it", with the
note "bind the conditional's value with a typed `let` first, or convert within
each branch" or "... or compare within each branch".

**Index ranges.** A block binding of type `Int` has no static range, as a
body binding has none; a binding of a word type ranges over its type, as in
`LOOKUPS_2026.md`.

## 6. Meaning

A loop evaluates its step once for each index: first the value of each of the
step's bindings in order, each seeing the bindings before it, and then the
step's value, which becomes the next accumulator. A step's bindings are
discarded when the step ends and computed afresh at the next step.

A conditional evaluates only its chosen branch, bindings included. **The
bindings of a branch that is not chosen are never evaluated**; they cost no
steps and cannot exhaust a budget.

A binding names a value. It has no effect, and it costs nothing beyond the
steps of its value and one step per read.

## 7. Typed Reference Core

A Core loop gains the bindings of its step, and a Core conditional the
bindings of its `then` branch and of its `else` branch. A block binding
records its span, its name and the name's span, its declared type, and `end`:
the offset in the step's or branch's expression just after the binding's
value.

A step's or branch's expression holds the value subtree of each of its
bindings in order and then the subtree of its value, so the bindings'
subtrees partition a prefix of the expression and each ends where its `end`
says. Two node kinds read them: `step_binding(loop, index)` and
`branch_binding(conditional, index)`, typed with the binding's type. The
bindings of a chain's final `else` branch belong to the conditional of the
chain's last arm.

## 8. Evaluation

A block's binding values lie on the evaluator's value stack above the length
at which its step or branch began. Each binding takes its slot when the
subtree of its value is complete, and the block's intermediate values lie
above the slots. When the step or branch ends, its value is taken and its
bindings' slots are removed.

| Operation | Steps |
| --- | --- |
| a `step_binding` or `branch_binding` read | 1 |
| a block's binding | the steps of its value, each time its block is evaluated |

The per-source budget of 1,048,576 steps is unchanged. Inconsistent Core,
such as a read of a binding that has not taken its slot, a binding whose value
is empty or ends past its block, a binding value or a read of another type
than the binding's, or a read of a block that is not being evaluated, stops
evaluation with no values.

## 9. Resource limits and failure

The S3i budgets remain. S3j adds or refines the following.

- A block declares at most 256 bindings (section 3).
- A block's bindings are parsed at the nesting level of its value and count
  toward the height of their loop or conditional (section 3).
- A block's binding costs one semantic event for its name's uniqueness check,
  the events of its declared type, and the events of its value, as a body's
  binding does.
- A block's binding counts one Core node for itself and one for its type; its
  value's nodes are nodes of the step or branch.
- An allocation failure while recording a block's bindings is `ORC0209`,
  "block binding storage allocation failed", and gives no Core.

The deepest sources the limits admit, including 64 loops nested in steps and
64 conditionals nested in branches with a binding in every block, a step and
a branch of 256 bindings each nested 63 levels deep, and 32 conditionals
nested in bindings' values, must parse, analyze, and evaluate within 1 MiB of
native stack. A syntax tree whose spans, including those of every block's
bindings, do not all belong to the source it is supplied with is `ORC0210`
for that source, and nothing is checked.

## 10. Determinism and conformance

The determinism requirement of `EXPRESSIONS_2026.md` section 15 applies
unchanged. The S3j conformance runner
(`compiler/crates/orangec/tests/s3j_conformance.rs`) parses this index and
requires exact agreement with its evidence map, under the same rules as the
S3b through S3i runners.

### S3j conformance rule index

| Rule ID | Clause | Executable obligation | Evidence layer |
| --- | --- | --- | --- |
| `S3J-SYNTAX-01` | Section 3 | A loop's step and each branch hold `let` bindings and then a value, at most 256 bindings per block, parsed at the block's nesting level and counted toward its height; malformed forms are `ORC0101` or `ORC0106` with the specified notes. | CLI and parser unit |
| `S3J-SCOPE-01` | Section 4 | A block's binding is in scope for its block's later bindings and value only, repeats no name in scope (`ORC0219`), and is `ORC0211` before its `;` or outside its block, with the specified messages. | CLI and unit |
| `S3J-TYPE-01` | Section 5 | A binding's value has its declared type, parts are checked in source order, and a branch whose leaf names its own binding gives the conditional no type (`ORC0220`, `ORC0227`). | CLI and unit |
| `S3J-CORE-01` | Section 7 | Core records each block's bindings with their `end` offsets, each binding's value subtree before the block's value, and reads as `step_binding` and `branch_binding` nodes. | Unit and CLI observation |
| `S3J-EVAL-01` | Section 8 | A step's bindings are evaluated at every step and a branch's only when chosen, at the specified step costs; SHA-256 and X25519 written with blocks match FIPS 180-4 and RFC 7748. | CLI and unit |
| `S3J-RES-01` | Section 9 | Blocks, their events, nodes, allocations, stack use, and inconsistent Core are bounded as specified, and foreign spans are `ORC0210`. | Generated CLI and unit |
| `S3J-COMPAT-01` | Section 11 | S3i sources keep their meaning, Core values, and output bytes, with only the specified message changes. | CLI and unit |
| `S3J-DETERMINISM-01` | Section 10 | Repeated identical inputs produce identical status, diagnostics, and output bytes. | CLI and unit |

## 11. Relationship to S3i

When OEP-0013 is accepted, this document replaces these clauses of
`MODULAR_2026.md` and the documents it extends:

- the loop production of `LOOPS_2026.md` section 3 and the conditional
  production of `CONDITIONS_2026.md` section 3, by section 3;
- the scope and name rules of `BINDINGS_2026.md` section 5, `LOOPS_2026.md`
  section 4, and `CONDITIONS_2026.md` section 8, for block bindings, by
  section 4;
- the typed-leaf rule of `BINDINGS_2026.md` section 6, as S3f extends it to
  conditionals, by section 5; and
- the Core records and the step table, by sections 7 and 8.

Every source that S3i accepts has no binding in a step or branch, so S3j
accepts it with the same Core values and the same output bytes. A source that
S3i rejects gets the same diagnostics, with these exceptions:

- a `let` binding at the start of a loop's step was `ORC0101`, "expected `}`
  after the loop's step", and one at the start of a branch `ORC0101`,
  "expected `}` after the value"; they are now accepted, or rejected by
  sections 3 through 5; and
- the notes of those two diagnostics, which were "a loop's step is one
  expression that gives the accumulator's next value" and "each branch of a
  conditional is one expression", now describe a block (section 3).

## 12. Explicit non-claims and future work

This slice defines no block as an expression of its own outside a step or a
branch, no shadowing, no assignment or mutation, no loop with more than one
accumulator, no tuples or destructuring, no inference of a binding's type,
no static range for an `Int` binding, and none of the exclusions of
`MODULAR_2026.md` section 13 that this document does not lift.

A branch's bindings are evaluated only when the branch is chosen, as its value
is. That is a statement about mathematical values, not about time: the
reference evaluator is not constant-time, and nothing here concerns code
generation or the obligations of a backend (`CONDITIONS_2026.md` section 15).

A fixture that reproduces a standard's example value is not thereby a verified
transcription of that standard. Tests establish the tested behavior of one
implementation at one revision; they do not prove semantic soundness,
completeness, or implementation independence.

Adding any excluded behavior requires a later accepted decision, a normative
specification, positive and negative conformance cases, resource analysis, and
migration review.
