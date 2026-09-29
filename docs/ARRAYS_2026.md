# Orange 2026 arrays specification

Status: proposed S3d semantics under OEP-0007, in owner review; not accepted

Edition: `2026`

Snapshot: 2026-09-28

This document defines slice S3d of Orange 2026: fixed-length arrays of `Int`
or word values, array literals, and selection of one element by a literal
index. It is a delta over the proposed S3c rules in
[`BINDINGS_2026.md`](BINDINGS_2026.md), which are a delta over
[`EXPRESSIONS_2026.md`](EXPRESSIONS_2026.md). Everything those documents
define and this one does not mention is unchanged.

It is proposed, not accepted. It becomes normative only when the owner accepts
[OEP-0007](governance/oeps/OEP-0007-orange-2026-fixed-length-arrays.md), which
requires OEP-0006. At that point it replaces the S3c clauses listed in section
11. Until then, the compiler behavior it describes exists so that the proposal
can be reviewed against running code, and it establishes no accepted language
meaning. It accepts no D-004 candidate.

> [!NOTE]
> [`LOOPS_2026.md`](LOOPS_2026.md), proposed under OEP-0008, extends this
> document with bounded loops, indices computed from loop indices, updates of
> one element, and fill literals, which lift the absence of loops described in
> section 12. Every source this document accepts keeps its meaning under it.

The terms **must**, **must not**, and **may** are normative in this document.

## 1. The idea

A cipher's state is not four loose words. RFC 8439 describes the ChaCha20
state as a 4 by 4 matrix of 32-bit words, FIPS 180-4 gives SHA-256 eight
working variables and a sixteen-word message block, and a key is a string of
bytes. S3d lets a specification hold such a state as one value, pass it
through functions, and name each position the way the standard does:

```orange
edition 2026;
module chacha20 {
  spec quarter_round(a: Word[32], b: Word[32], c: Word[32], d: Word[32]) -> Word[32]^4 {
    let a1: Word[32] = a + b;
    let d1: Word[32] = (d ^ a1) <<< 16;
    let c1: Word[32] = c + d1;
    let b1: Word[32] = (b ^ c1) <<< 12;
    let a2: Word[32] = a1 + b1;
    let d2: Word[32] = (d1 ^ a2) <<< 8;
    let c2: Word[32] = c1 + d2;
    let b2: Word[32] = (b1 ^ c2) <<< 7;
    [a2, b2, c2, d2]
  }

  spec column_round(x: Word[32]^16) -> Word[32]^4 {
    quarter_round(x[0], x[4], x[8], x[12])
  }
}
```

`Word[32]^4` is read "four 32-bit words". It is the notation of the
mathematics: a value of `Word[32]^4` is an element of (Z/2^32 Z)^4, a tuple of
four ring elements. With arrays, the whole ChaCha20 block function of RFC 8439
section 2.3 is one short Orange module, and `orangec eval` reproduces the
serialized block of section 2.3.2 word for word.

Three commitments shape every rule below.

- **Every length is written.** An array type states its length, an array
  literal lists every element, and no length is inferred or padded.
- **Every position is visible.** An index is a literal, so every element a
  specification reads can be found by reading it. There is no variable index,
  and no index can be out of range at run time.
- **Operators act on elements.** `x ^ y` on two arrays is not defined. A
  reader who sees an operator knows it applies to one `Int` or word value.

## 2. Scope and phase boundary

The phase boundary of `EXPRESSIONS_2026.md` section 2 is unchanged. The
accepted surface grows to: array types `T^n` for parameters, results, and
bindings; array literals; and literal indices of names and calls. Every S3c
form keeps its meaning. Arrays of arrays, empty arrays, and variable indices
are not part of this slice.

## 3. Grammar

No token and no reserved word is added. The S3c productions for declared types
and primary expressions become:

```text
typed_tail      = "->" declared_type "{" binding* expression "}" ;
binding         = "let" IDENTIFIER ":" declared_type "=" expression ";" ;
parameter       = IDENTIFIER ":" declared_type ;
declared_type   = parsed_type ("^" INTEGER)? ;

primary         = IDENTIFIER index? | call index? | "(" expression ")"
                | array ;
index           = "[" INTEGER "]" ;
array           = "[" expression ("," expression)* ","? "]" ;
```

Every other production is unchanged. In particular a conversion's target is
still a `parsed_type`: `x as Word[8] ^ y` is the S3c `ORC0108`, because `^`
after a conversion is the exclusive-or operator, and a conversion never
produces an array.

- The length of an array type is one `INTEGER` token directly after `^`. A
  second `^` after it is a syntax error: arrays of arrays are not part of
  Orange 2026.
- An index follows a name or a call directly. It is one `INTEGER` token
  between brackets, with no sign and no expression. A second index, as in
  `x[0][1]`, is a syntax error, because an element is never an array.
- An array literal holds at least one element. `[]` is a syntax error. The
  list may end with one comma.
- A group, a literal, or an array literal cannot be indexed: `(x)[0]` and
  `[a, b][0]` are syntax errors at the `[`.

Each is `ORC0101` with the note that explains the rule. The syntax tree gains
an optional length span on each declared type, one array node per literal,
holding its elements in order, and one index node per index, holding its base
and the span of its integer token. An array's span runs from `[` through `]`;
an index's span runs from its base through `]`.

## 4. Array types

For each admitted scalar type `T` of `EXPRESSIONS_2026.md` section 7 and each
length n from 1 through 256, `T^n` is an admitted type:

| Source form | Values |
| --- | --- |
| `Int^n` | n-tuples of mathematical integers |
| `Word[w]^n` | n-tuples of integers modulo 2^w, for w in 8, 16, 32, 64 |

The length must be a decimal spelling with no base prefix, separator, sign, or
leading zero, exactly as a word width is. `Word[32]^16` is admitted;
`Word[32]^0`, `Word[32]^257`, `Word[32]^0x10`, `Word[32]^016`, and
`Word[32]^1_6` are `ORC0221`, reported at the length. The element type is
resolved first: `Word^4` is `ORC0204` and `Float^4` is `ORC0203`, and neither
is reported again for its length.

Two array types are equal exactly when their element types and lengths are
equal. There is no subtyping between lengths and no conversion between array
types.

## 5. Typing

The expected-type rules of `EXPRESSIONS_2026.md` section 8 and
`BINDINGS_2026.md` section 6 extend as follows.

**Array literals.** An array literal is checked against an expected type
`T^n`. If it lists m elements and m differs from n, that is `ORC0222` at the
literal. Each element is then checked, in order, against `T`, whether or not
the count matched. An array literal where a scalar type is expected is
`ORC0214`, and its elements are not checked.

**Indices.** An index `b[k]` is checked against an expected type `E`:

1. The base's own type is found without reporting: the declared type of a
   name, or the result type of a called function.
2. If the base has no type, because the name or function is unknown or its
   declared type did not resolve, the base's own error is reported.
3. If the base's type is a scalar, that is `ORC0224` at the base, and the base
   is then checked against its own type.
4. Otherwise the base has type `T^n`. The index literal is decoded as an S3a
   magnitude, including the 16,384-significant-bit limit (`ORC0205`), in any
   S3a spelling. An index k with k ≥ n is `ORC0223` at the index.
5. If `T` differs from `E`, that is `ORC0214` at the index expression.
6. The base is checked against its own type, so errors inside a call's
   arguments are still reported.

**Names and literals of the wrong kind.** A name of array type where a scalar
is expected, and a scalar name where an array is expected, are the S3b
`ORC0214`; when the name's element type is the expected type, the diagnostic
suggests an index. An integer literal where an array is expected is `ORC0214`.

**Operators and conversions.** The operator table of `EXPRESSIONS_2026.md`
section 8 is unchanged, and no operator is defined on an array type: an
operator whose expected type is an array is `ORC0215` at the operator. The
first typed leaf of a conversion operand (`BINDINGS_2026.md` section 6) may now
be an index, whose type is its element type, so `x[0] as Int` is an ordinary
conversion. A conversion whose first typed leaf is an array literal or has an
array type is `ORC0215` at `as`. Like every undefined operator
(`EXPRESSIONS_2026.md` section 10), it stops there: its operand is not
checked, so a call inside it is not examined and adds no call graph edge.

Arrays pass through calls and bindings like any other value: a parameter,
result, or binding may have an array type, and the S3b and S3c rules for
arguments, results, and names apply unchanged.

## 6. Meaning

An array value of type `T^n` is a sequence of exactly n values of type `T`,
with positions 0 through n - 1.

- An array literal evaluates its elements left to right, as call arguments
  are, and its value holds them in that order.
- An index `b[k]` evaluates `b` and selects the element at position k. Section
  5 guarantees that k is in range, so selection never fails.
- Arrays are values. Passing an array to a function or binding it to a name
  shares nothing observable; there is no mutation, so no aliasing can be seen.

`orangec eval` writes an array value as its elements in order, each in its
scalar display form, separated by a comma and a space and enclosed in brackets.
The type is written `T^n`:

```text
chacha20::test_vector: Word[32]^16 = [0xe4e7f110, 0x15593bd1, ..., 0x4e3c50a2]
arrays::exact: Int^3 = [-1, 0, 340282366920938463463374607431768211456]
```

The output above is shortened; a value line is never shortened.

## 7. Diagnostics

The examination order of `BINDINGS_2026.md` section 8 is unchanged, and a
declared type's length is examined after its element type. Within an
expression, an array literal is examined before its elements: its kind, then
its length, then each element. An index is examined as section 5 lists: the
base's kind, the index's range, the element type, then the base. Call cycles
are still reported after every function, and calls inside array elements and
index bases are edges like any other.

The new and widened categories are:

| Code | Phase | Meaning |
| --- | --- | --- |
| `ORC0101` | parse | also an empty array, a non-literal or repeated index, or an array of arrays |
| `ORC0214` | semantic | also an array literal where a scalar is expected, a literal where an array is expected, or an element of the wrong type |
| `ORC0215` | semantic | also an operator or conversion applied to an array |
| `ORC0221` | semantic | an array length is not a decimal integer from 1 through 256 |
| `ORC0222` | semantic | an array literal lists a different number of elements than its type |
| `ORC0223` | semantic | an index is not below the array's length |
| `ORC0224` | semantic | an index is applied to a value that is not an array |

`ORC0106` also reports the element limit of section 9. Every other code keeps
its meaning.

## 8. Typed Reference Core

The Core of `BINDINGS_2026.md` section 9 gains array types and two node kinds:

```text
core_type      = Int | Word8 | Word16 | Word32 | Word64
               | Array core_scalar_type length ;
node_kind      = ...
               | array element_count
               | index position ;
```

An `array` node has an array type and consumes the element count of immediately
preceding complete subtrees, in position order; its count equals its type's
length. An `index` node consumes one preceding subtree of array type and has
that array's element type; its position is below the array's length. A Core
literal is never an array. The Core is still internal and noncanonical, with no
encoding, digest, or proof role, and a function without arrays has exactly its
S3c Core.

## 9. Resource limits and failure

The S3c budgets remain. S3d adds or refines the following.

**Parsing.** An array literal holds at most 256 elements; a 257th is a parser
resource limit (`ORC0106`) at that element. An array literal opens one nesting
level, as a group or a call does, and draws on the same 64-level budget; the
nesting message names groups, calls, arrays, and prefix operators. An array
adds one to the height of its tallest element; an index adds one to the height
of its base and opens no nesting level.

**Semantic events.** A declared type's length token counts as a parsed-type
component (one event). S3d adds one event per array literal and one per index.
An index's literal then costs the S3a decoding events: one for its base prefix
and one per significant digit. The search for an index base's type consumes no
events, as the conversion operand search does not.

**Core nodes.** Each array literal and each index is one Core node.

**Evaluation.** An `array` node of n elements costs n steps; an `index` node
costs one step. Arrays occupy the frame of their function and add no call
depth.

Exhausting any budget, and any allocation failure, yields one resource
diagnostic, no Core, and no value line. The deepest sources the limits admit,
including 64 levels of arrays and indexed calls nested inside one another,
must parse, analyze, and evaluate within 1 MiB of native stack. A source that
nests 64 array literals is rejected at the second, and it too must be analyzed
within that bound.

## 10. Determinism and conformance

The determinism requirement of `EXPRESSIONS_2026.md` section 15 applies
unchanged. The S3d conformance runner
(`compiler/crates/orangec/tests/s3d_conformance.rs`) parses this index and
requires exact agreement with its evidence map, under the same rules as the S3b
and S3c runners: every rule needs command-line evidence that runs twice with
identical results, except where the index says unit evidence alone, and every
rule names unit tests declared exactly once in their sources' test modules.

### S3d conformance rule index

| Rule ID | Clause | Executable obligation | Evidence layer |
| --- | --- | --- | --- |
| `S3D-GRAMMAR-01` | Section 3 | Array types, literals, and indices parse with exact spans; an empty array, a non-literal or repeated index, and an array of arrays are `ORC0101`. | CLI and parser unit |
| `S3D-TYPE-01` | Section 4 | `T^n` resolves for an admitted scalar `T` and a decimal n from 1 through 256; any other length is `ORC0221`. | CLI and unit |
| `S3D-LITERAL-01` | Section 5 | An array literal lists exactly its type's length of elements, each of the element type; otherwise `ORC0222` or `ORC0214`. | CLI and unit |
| `S3D-INDEX-01` | Section 5 | An index applies to an array, is below its length, and has the element type; otherwise `ORC0224`, `ORC0223`, or `ORC0214`. | CLI and unit |
| `S3D-OPERATOR-01` | Section 5 | Operators and conversions apply to elements; applied to an array they are `ORC0215`. | CLI and unit |
| `S3D-EVAL-01` | Section 6 | Array literals evaluate their elements in order, indices select them, and arrays pass through calls and bindings. | CLI and unit |
| `S3D-DISPLAY-01` | Section 6 | An array value prints as `T^n = [e0, e1, ...]` with every element in its scalar form. | CLI and unit |
| `S3D-DIAG-01` | Section 7 | Diagnostic order, spans, and non-cascading behavior are exactly as specified. | CLI and unit |
| `S3D-CORE-01` | Section 8 | Core carries array types and `array` and `index` nodes in postorder. | Unit and CLI observation |
| `S3D-RES-ELEM-01` | Section 9 | 256 elements are accepted and a 257th fails with `ORC0106`; a length of 257 is `ORC0221`. | Generated CLI and parser unit |
| `S3D-RES-NEST-01` | Section 9 | Array literals share the 64-level nesting budget, and arrays and indices add exactly one to tree height. | Unit |
| `S3D-RES-EVENT-01` | Section 9 | Semantic events and Core nodes for array types, literals, and indices are counted exactly. | Unit |
| `S3D-RES-STEP-01` | Section 9 | An array of n elements costs n steps and an index one. | Unit |
| `S3D-RES-STACK-01` | Section 9 | The deepest admitted sources with arrays and indices, and 64 nested array literals, fit in 1 MiB of native stack. | Unit |
| `S3D-RES-FAIL-01` | Section 9 | Allocation failure, foreign syntax trees, and inconsistent Core yield no partial tree, Core, or value. | Unit |
| `S3D-COMPAT-01` | Section 11 | S3c programs keep their meaning and messages, except the nesting-limit message, which names arrays. | CLI and unit |
| `S3D-DETERMINISM-01` | Section 10 | Repeated identical inputs produce identical status, diagnostics, and output bytes. | CLI and unit |

## 11. Relationship to S3c

When OEP-0007 is accepted, this document replaces these clauses of
`BINDINGS_2026.md` and `EXPRESSIONS_2026.md`:

- the `typed_tail`, `binding`, `parameter`, and `primary` productions, by
  section 3;
- `EXPRESSIONS_2026.md` section 7, whose table of admitted types gains the
  array types of section 4;
- `EXPRESSIONS_2026.md` section 8 and `BINDINGS_2026.md` section 6, extended
  by section 5;
- `EXPRESSIONS_2026.md` section 12, whose display gains arrays by section 6;
- the diagnostic, Core, and resource sections of both, extended by sections 7,
  8, and 9; and
- the exclusion of arrays from both documents' non-claims.

Every source that S3c accepts is accepted by S3d with the same Core values and
the same output bytes. `^` after a declared type was never valid, `[` after a
name, a call, or at the start of an operand was never valid, and `^` after a
conversion target keeps its meaning. A source that S3c rejects gets the same
diagnostics, with one exception: the expression-nesting message now names
arrays among the forms that open a level.

## 12. Explicit non-claims and future work

This slice defines no arrays of arrays, empty arrays, variable or computed
indices, slices, array concatenation, array comparison, elementwise operators,
loops, comprehensions, length polymorphism, tuples or records of mixed types,
byte-string literals, mutation or update in place, or any of the exclusions of
`BINDINGS_2026.md` section 13.

The absence of loops is visible in the ChaCha20 fixture: the ten double rounds
are ten bindings. A bounded loop, or a fold over a literal range, is the
natural next step. So is a way to update one element and keep the rest, which
would let a quarter round act on a whole state.

A fixture that reproduces a standard's example value is not thereby a verified
transcription of that standard. No statement here says whether indexing or any
expression would run in constant time on any machine. Tests establish the
tested behavior of one implementation at one revision; they do not prove
semantic soundness, completeness, or implementation independence.

Adding any excluded behavior requires a later accepted decision, a normative
specification, positive and negative conformance cases, resource analysis, and
migration review.
