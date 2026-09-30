# Orange 2026 tuples specification

Status: proposed S3k semantics under OEP-0014, in owner review; not accepted

Edition: `2026`

Snapshot: 2026-09-30

This document defines slice S3k of Orange 2026: **tuples**, values made of a
fixed number of elements of possibly different types; the selection of an
element by its position; and **tuple patterns**, which name each element of a
tuple where a binding or a loop's accumulator is declared, so that a loop may
carry several accumulators. It is a delta over the proposed S3j rules in
[`BLOCKS_2026.md`](BLOCKS_2026.md), which are a delta over
[`MODULAR_2026.md`](MODULAR_2026.md) and the documents it extends.
Everything those documents define and this one does not mention is
unchanged.

It is proposed, not accepted. It becomes normative only when the owner accepts
[OEP-0014](governance/oeps/OEP-0014-orange-2026-tuples.md), which requires
OEP-0013. At that point it replaces the S3j clauses listed in section 11.
Until then, the compiler behavior it describes exists so that the proposal can
be reviewed against running code, and it establishes no accepted language
meaning. It accepts no D-004 candidate.

> [!NOTE]
> [`BYTES_2026.md`](BYTES_2026.md), proposed under OEP-0015, extends this
> document with byte strings, the concatenation `++` of arrays, and slices,
> which may follow a tuple's element as `p.0[..4]`. A tuple is not joined with
> `++`. [`SIZES_2026.md`](SIZES_2026.md), proposed under OEP-0016, adds size
> parameters, whose instances may take and give tuples.
> [`ORDER_2026.md`](ORDER_2026.md), proposed under OEP-0017, adds conversions in
> a byte order, which convert no tuple, and
> [`TYPE_PARAMETERS_2026.md`](TYPE_PARAMETERS_2026.md), proposed under
> OEP-0018, lets a tuple's elements be a function's type parameter, as
> `(K, K)`, and lists tuple types among a function's types. Every source this
> document accepts keeps its meaning under all four.

The terms **must**, **must not**, and **may** are normative in this document.

## 1. The idea

A standard's round keeps several values at once. FIPS 180-4 section 6.2.2
carries the eight working variables a through h from one round of SHA-256 to
the next; RFC 8439 section 2.1 writes the ChaCha20 quarter round as a function
of four words that gives four words; the extended Euclidean algorithm carries
three pairs of values. Through S3j a loop had one accumulator and a function
one result, so such state had to be packed into an array, as `v[0]` through
`v[7]`, and a function that computes two values, such as a sum and its carry,
had to be written twice.

S3k adds tuples. A tuple type lists its element types in parentheses, a tuple
lists its elements the same way, and `.k` selects element k, counted from
zero. A tuple pattern names each element where a `let` binding or a loop's
accumulator is declared:

```orange
spec add_carry(a: Limb, b: Limb, carry: Limb) -> (Limb, Limb) {
  let s: Limb = a + b;
  let t: Limb = s + carry;
  let out: Limb = if s < a { 1 } else { 0 };
  (t, if t < s { out + 1 } else { out })
}

spec add256(x: Limb^4, y: Limb^4) -> (Limb^4, Limb) {
  for i in 0..4 with (sum: Limb^4, carry: Limb) = ([0; 4], 0) {
    let (limb: Limb, out: Limb) = add_carry(x[i], y[i], carry);
    (sum with [i] = limb, out)
  }
}
```

With a pattern on its accumulator, a loop carries several named values, and a
round of SHA-256 reads as FIPS 180-4 writes it:

```orange
for t in 0..64 with (a: Word[32], b: Word[32], c: Word[32], d: Word[32],
                     e: Word[32], f: Word[32], g: Word[32], h: Word[32]) =
  (hash[0], hash[1], hash[2], hash[3], hash[4], hash[5], hash[6], hash[7]) {
  let t1: Word[32] = h + big_sigma1(e) + ch(e, f, g) + k[t] + w[t];
  let t2: Word[32] = big_sigma0(a) + maj(a, b, c);
  (t1 + t2, a, b, c, d + t1, e, f, g)
}
```

A tuple is a value, like an array: it has no identity, it is never changed in
place, and the names of a pattern are names for its elements, not variables.
A tuple's elements are scalars and arrays; a tuple holds no tuple, and an
array holds no tuple.

## 2. Scope and phase boundary

The phase boundary of `EXPRESSIONS_2026.md` section 2, as S3j extends it, is
unchanged. Every S3j source keeps its meaning (section 11).

## 3. Grammar

No token and no reserved word is added: `(`, `)`, `,`, and `.` are existing
tokens. The type, binding, loop, and primary productions of the earlier
documents become:

```text
Type         = ScalarType ( "^" INTEGER )? | TupleType ;
TupleType    = "(" ElementType "," ElementType ( "," ElementType )* ","? ")" ;
ElementType  = ScalarType ( "^" INTEGER )? ;

binding      = "let" pattern "=" expression ";" ;
loop         = "for" IDENTIFIER "in" INTEGER ".." INTEGER
               "with" pattern "=" expression block ;
pattern      = IDENTIFIER ":" Type
             | "(" typed_name "," typed_name ( "," typed_name )* ","? ")" ;
typed_name   = IDENTIFIER ":" Type ;

primary      = IDENTIFIER suffix? | call suffix? | "(" expression ")"
             | tuple | array | fill | loop | conditional | ... ;
tuple        = "(" expression "," expression ( "," expression )* ","? ")" ;
suffix       = projection index? | index ;
projection   = "." INTEGER ;
```

A **tuple type** has two through 16 element types; a **tuple** has two
through 16 elements; a **tuple pattern** names two through 16 values. A
trailing comma may follow the last element of each. `(e)` without a comma is
a group, as before, and `(T)` is not a type. A tuple type may appear wherever
a type may: as a parameter's, result's, binding's, or accumulator's type, in
a `type` declaration, and, parsed but always rejected (section 5), as the
target of `as`.

A **projection** `.k` follows a name or a call, optionally followed by one
index, as in `pair.1[i]`. Its position `k` is one `INTEGER` token written in
decimal, with no sign, prefix, `_`, or leading zero (`0` itself is allowed).
A position too large for 32 bits is kept as 2^32 - 1, which no tuple has. A
projection follows nothing else: not a group, a tuple, an array, a loop, or
a conditional, and not another projection or an index.

`let` is recognized by position, as in `BINDINGS_2026.md` section 3 and
`BLOCKS_2026.md` section 3: it starts a binding when it is the first token of
a body or block item and the next tokens are an identifier; `(` and then an
identifier and `:`; or `(`, a list of at most 17 identifiers separated by
commas with an optional trailing comma, `)`, and `=`. The last form is always
an error, because its names have no types, and is parsed as a pattern so that
the missing type is reported. Anywhere else `let` is a name, as before.

The parse errors are `ORC0101`, each with a note that shows the form:

| Form | Message | Note |
| --- | --- | --- |
| `(Int)` | "expected `,` and another element type" | "a tuple type is written `(T, U)` with two through 16 element types, each `Int`, `Bool`, a word, a residue, or an array of one" |
| `()`, `(Int,)`, `(Int Int)` | "expected an element type" or "expected `,` or `)` after the element type" | the same |
| `((Int, Int), Int)` | "expected an element type", at the inner `(` | "a tuple's elements are `Int`, `Bool`, words, residues, and arrays of them; a tuple holds no tuple" |
| `(Int, Int)^2` | "expected the end of the type after the tuple", at `^` | "an array's elements are `Int`, `Bool`, words, or residues; arrays of tuples are not part of Orange 2026" |
| `(1,)`, `(1, 2` | "expected another element after `,`" or "expected `,` or `)` after the tuple's element" | "a tuple is written `(a, b)` with two through 16 elements; `(a)` without a comma is a group" |
| `p.`, `p.01`, `p.0x1` | "expected an element's position after `.`" or "expected an element's position in decimal" | "a tuple's element is selected by its position, counted from zero and written in decimal, as in `pair.0` or `pair.1`" |
| `p.0.1` | "expected an operator or the end of the expression", at the second `.` | "a tuple's elements are not tuples, so an element is selected once" |
| `x[0].1` | the same, at `.` | "an array's elements are not tuples, so an element has no `.k`" |
| `let (x, y) = p;`, `with (s: Int, t) = ...` | "expected `:` and the type of the name" | "a tuple pattern names two through 16 values, each with its type, as in `let (sum: Word[64], carry: Word[64]) = add(x, y, c);`" |
| `let (x: Int) = 1;` | "expected `,` and another name in the pattern" | the same |
| `let (x: Int, 1) = p;` | "expected a name for the pattern's next value" or "expected `,` or `)` after the pattern's element" | the same |

After an error inside a tuple type, the parser skips past the type's closing
`)`, so that an enclosing parameter list does not read the tuple's commas as
its own. It skips at most to a `{`, `}`, `;`, `->`, `=`, `spec`, `impl`, or
the end of the source, and within the recovery-depth limit of
`EXPRESSIONS_2026.md` section 12.

**Nesting and height.** A tuple opens one nesting level, as a group does, and
its height is one more than the tallest of its elements. A projection opens
no level and adds one to the height of its base; an index after it is parsed
as any index is. A tuple type's element types are parsed at the level of the
type, and a pattern's types at the level of its binding or loop; their
heights are the tallest modulus among them, as for one type in S3i.

The syntax tree gains a tuple expression, a projection with its base, the
span of its position, and the position, tuple type syntax with its element
types, and patterns: a binding or an accumulator holds either one typed name
or a tuple pattern of typed names with the pattern's span.

## 4. Names and scope

A tuple pattern declares each of its names as the kind of name it stands in
for: the names of a body's binding are bindings of the body, the names of a
block's binding are bindings of that block, and the names of a loop's pattern
are accumulators of that loop. Each is in scope exactly where a single name
of that kind would be: a binding's names from the end of its `;`, and an
accumulator's names in the loop's step, its bindings, and its value, not in
the loop's header or first value.

Every name of a pattern must differ from the pattern's other names and from
every name a single name in its place must differ from (`BINDINGS_2026.md`
section 5, `LOOPS_2026.md` section 4, `BLOCKS_2026.md` section 4). A repeated
name is `ORC0219` with the S3c through S3j message of that kind, such as
"duplicate binding `a`" with a secondary span "the first name is here" for
the pattern's own earlier name, "the parameter is here", or "the first
binding is here", and "duplicate name `i`" with "the loop index is here" for
a loop's pattern. Names whose scopes do not overlap may repeat: in the
SHA-256 example of section 1, the body's pattern that binds the loop's result
may use the names a through h of the loop's own pattern, because the loop's
names are in scope only in its step and the body's names only after the
binding's `;`.

A name of a pattern read before its binding's `;` is `ORC0211`, "`a` is used
before it is bound", citing the name; a loop's names read after the loop are
`ORC0211` with the S3c through S3j messages, such as "`a` is not a parameter
or binding of `after`". A name resolves in the order of `BLOCKS_2026.md`
section 4, as the kind of name it stands in for.

## 5. Typing

**Tuple types.** A tuple type is the type of its element types, in order. Two
tuple types are equal exactly when they have the same number of elements and
equal element types at each position; a `type` declaration names a tuple type
as it names any other, and a tuple type crosses a module boundary by its
elements, as a residue type does. A tuple's element is `Int`, `Bool`, a word,
a residue, or an array of one of them:

- an element type that is a declared name for a tuple type, or a tuple type
  written out as a pattern's name's type, is `ORC0203`, "`Pair` is a tuple
  type, so this is a tuple of tuples", labeled "a tuple holds no tuple"; and
- an array type whose element is a declared name for a tuple type is
  `ORC0203`, "`Pair` is a tuple type, so this is an array of tuples", with a
  secondary span at the length.

A tuple type is displayed as its element types in parentheses, separated by a
comma and a space, as in `(Word[64]^4, Word[64])`, with declared names
replaced by their types.

**Tuples.** A tuple is checked against the type required where it stands,
which must be a tuple type with as many elements. Its elements are checked in
order against the element types. A tuple where another type is required is
`ORC0214`, "a tuple cannot have type `Int`", and one with the wrong number of
elements is `ORC0214`, "this tuple has 3 elements, but `(Int, Int)` has 2".

**Projections.** For `base.k`, the base's type is found without reporting, as
an indexed array's is (`ARRAYS_2026.md` section 5). The base must have a
tuple type, which has an element at position k, and that element's type must
be the type required; the base is then checked against its own type, so
errors inside it are still reported.

- A base of another type is `ORC0234`, "only a tuple has elements selected by
  position, but this has type `Int^2`", with the note "an array's element is
  selected by an index, such as `x[0]`" for an array.
- A position the tuple lacks is `ORC0223`, "`(Int, Int)` has no element 2",
  labeled "its elements are numbered 0 through 1", at the position.
- An element of another type than the one required is `ORC0214`, "this
  element has type `Int`, but `Bool` is required here".

**Patterns.** A tuple pattern's type is the tuple of its names' types, and
the value is checked against it. If any of those types does not resolve, each
unresolved type is reported once where it is written, the value is not
checked, and no use of any of the pattern's names is reported, exactly as for
a binding of one name in `BINDINGS_2026.md` section 6. Each name has the type
written with it.

**Whole tuples.** A tuple is a value to be named, passed, returned, chosen by
a conditional, and carried by a loop. No operator applies to a whole tuple:

- a name of tuple type where another type is required is `ORC0214`, with the
  note "select one element by its position, such as `p.0`";
- an arithmetic, bitwise, shift, or prefix operator on a tuple is `ORC0215`,
  "`+` is not defined for `(Int, Int)`";
- a comparison of tuples is `ORC0215`, "`==` is not defined for `(Int, Int)`",
  with the note "compare elements, such as `p.0 == q.0`";
- `as` from a tuple is `ORC0215`, "`as` is not defined for `(Int, Int)`", and
  `as` to a tuple type `ORC0215`, "`as` does not convert to the tuple type
  `(Int, Int)`";
- an index into a tuple is `ORC0224`, "only an array can be indexed, but this
  has type `(Int, Int)`", with the note "a tuple's element is selected by its
  position, such as `p.0`", and an update of a tuple `ORC0224`, "only an array
  can be updated".

**Typed leaves.** A projection is a typed leaf (`BINDINGS_2026.md` section 6)
whose type is its element's type, and a name of a pattern is a typed leaf of
its own type. A tuple written out, like an array written out, has no type of
its own: as the first typed leaf of the operand of `as` it is `ORC0215`, "`as`
is not defined for a tuple", and as the first typed leaf of a comparison it is
`ORC0215`, "`==` is not defined for a tuple" (for an array or a fill written
out, "`==` is not defined for an array"), at the operator.

**Index ranges.** A projection or a name of a pattern whose type is a word
ranges over that type when it is used as an index, as a word binding does in
`LOOKUPS_2026.md`; one of type `Int` has no static range, as an `Int` binding
has none, and an index built from it is `ORC0226`.

## 6. Meaning

A tuple's elements are evaluated left to right, and the tuple is the sequence
of their values. `base.k` is element k of the tuple `base`, counted from zero.
A tuple pattern names each element of its value in order: in `let (s: T, c:
U) = e;`, `s` is element 0 of the value of `e` and `c` is element 1. A loop
whose accumulator is a pattern carries one tuple from step to step; at each
step the pattern's names are the elements of the current accumulator, and the
step's value is the next.

A tuple costs nothing to name, pass, or return beyond its elements' steps and
the steps of section 8, and the evaluator shares a tuple's value where it is
read more than once, as it shares an array's.

`orangec eval` displays a tuple as its elements in parentheses, separated by a
comma and a space, each displayed as in S3j:

```text
tuples::wraps: (Word[64]^4, Word[64]) = ([0x0000000000000000, 0x0000000000000000, 0x0000000000000000, 0x0000000000000000], 0x0000000000000001)
chacha20::quarter_round_vector: (Word[32], Word[32], Word[32], Word[32]) = (0xea2a92f4, 0xcb1cf8ce, 0x4581472e, 0x5881c4bb)
```

## 7. Typed Reference Core

The Core of the earlier documents gains a tuple type and two node kinds:

```text
core_type  = ... | Tuple core_element_type core_element_type+ ;
node_kind  = ...
           | tuple element_count
           | project position ;
```

A Core tuple type has two through 16 element types, each a scalar or an array
type, never a tuple type. A `tuple` node has a tuple type and consumes its
element count of immediately preceding complete subtrees, in position order;
its count equals its type's, and each subtree has its element's type. A
`project` node consumes one preceding subtree of tuple type and has the type
of its element at `position`, which is below the tuple's number of elements.

A binding or an accumulator declared by a tuple pattern is one Core binding,
local, or accumulator of the pattern's tuple type, exactly as one name of that
type would be. Its name is the pattern's names in parentheses, separated by a
comma and a space, as `(sum, carry)`, and its name's span is the pattern's. A read of one
of the pattern's names is the read of the whole binding, local, or accumulator
followed by a `project` node of the name's position; both nodes span the name.
The Core is still internal and noncanonical, with no encoding, digest, or
proof role, and a function without tuples has exactly its S3j Core.

## 8. Evaluation

A tuple value is its type and one value of each element type, in order, and
is shared, not copied, where it is read.

| Operation | Steps |
| --- | --- |
| a tuple of n elements | the steps of its elements, and n |
| `.k` | the steps of its base, and 1 |
| a read of a name of a pattern | 2: the read of the pattern's value and its `.k` |

The per-source budget of 1,048,576 steps is unchanged. Inconsistent Core
stops evaluation with no values: a `tuple` node whose count differs from its
type's, whose type is not a tuple type, or one of whose elements has another
type than its type says; a `project` node on a value that is not a tuple, of a
position the tuple lacks, or of another type than its element's; and a
function whose tuple result has another type than its declared type.

## 9. Resource limits and failure

The S3j budgets remain. S3k adds or refines the following.

- A tuple type has at most 16 element types, a tuple at most 16 elements,
  and a tuple pattern at most 16 names. The 17th is a parser resource limit
  (`ORC0106`) at that element: "a tuple type has more than 16 elements", "a
  tuple has more than 16 elements", or "a tuple pattern names more than 16
  values".
- A tuple opens one nesting level of the 64-level budget, and tuples,
  projections, and patterns count toward expression height (section 3).
- A tuple costs one semantic event and the events of its elements, a
  projection one event and the events of its base, and a tuple type one event
  and the events of its element types. A tuple pattern costs one event, and
  each name after its first one event for its uniqueness check and the events
  of its type; the first name's check and type are those of the binding or
  accumulator, as for one name.
- A tuple and a projection each count one Core node; a read of a name of a
  pattern counts two. A pattern's type counts one type node, as one name's
  type does.
- An allocation failure while parsing a tuple, a tuple type, or a pattern is
  `ORC0106` ("parser could not allocate tuple storage", "... type storage",
  or "... pattern storage"); while resolving a tuple type, `ORC0209`, "tuple
  type storage allocation failed"; and while evaluating, `ORC0301`,
  "reference evaluation result allocation failed", labeled "evaluation tuple
  storage could not be reserved" or "evaluated tuple storage could not be
  reserved". None gives partial output.
- A resolved tuple type's element types, at most 16 of them, are then copied
  once into one shared list, which every copy of the type reuses without
  allocating. The Rust standard library cannot yet report the failure of that
  allocation (its fallible form is unstable, and the compiler has no unsafe
  code), so a failure there aborts the process, during analysis and before
  any result is written, instead of giving `ORC0209`. It is never reported
  as success (`SEMANTICS_2026.md` section 9).

The deepest sources the limits admit, including 32 tuples nested in the
arguments of calls, each followed by a projection, and 63 loops nested in
steps whose accumulators are tuple patterns with a tuple pattern binding in
every step, must parse, analyze, and evaluate within 1 MiB of native stack. A
syntax tree whose spans, including those of every tuple, projection, tuple
type, and pattern, do not all belong to the source it is supplied with is
`ORC0210` for that source, and nothing is checked.

## 10. Determinism and conformance

The determinism requirement of `EXPRESSIONS_2026.md` section 15 applies
unchanged. The S3k conformance runner
(`compiler/crates/orangec/tests/s3k_conformance.rs`) parses this index and
requires exact agreement with its evidence map, under the same rules as the
S3b through S3j runners.

### S3k conformance rule index

| Rule ID | Clause | Executable obligation | Evidence layer |
| --- | --- | --- | --- |
| `S3K-SYNTAX-01` | Section 3 | Tuple types, tuples, projections, and tuple patterns have two through 16 parts, with an optional trailing comma, parse at the specified levels and heights, and malformed forms are `ORC0101` or `ORC0106` with the specified messages and notes. | CLI and parser unit |
| `S3K-SCOPE-01` | Section 4 | A pattern's names are unique among the names in scope and the pattern's other names (`ORC0219`), are in scope where a single binding or accumulator is, and are `ORC0211` elsewhere. | CLI and unit |
| `S3K-TYPE-01` | Section 5 | Tuples are checked against tuple types of as many elements, a tuple holds no tuple and is no array's element (`ORC0203`), `.k` selects an element of a tuple (`ORC0234`, `ORC0223`), no operator applies to a whole tuple, and unresolved pattern types are reported once. | CLI and unit |
| `S3K-CORE-01` | Section 7 | Core records tuple types, `tuple` and `project` nodes, and each pattern as one binding or accumulator whose names are read through `project`. | Unit and CLI observation |
| `S3K-EVAL-01` | Section 8 | Tuples are evaluated left to right and selected by position at the specified step costs; SHA-256, the ChaCha20 quarter round and block function, and Ascon-Hash256 written with tuples match FIPS 180-4, RFC 8439, and NIST SP 800-232. | CLI and unit |
| `S3K-RES-01` | Section 9 | Tuples and patterns, their events, nodes, allocations, stack use, and inconsistent Core are bounded as specified, and foreign spans are `ORC0210`. | Generated CLI and unit |
| `S3K-COMPAT-01` | Section 11 | S3j sources keep their meaning, Core values, and output bytes, with only the specified diagnostic changes. | CLI and unit |
| `S3K-DETERMINISM-01` | Section 10 | Repeated identical inputs produce identical status, diagnostics, and output bytes. | CLI and unit |

## 11. Relationship to S3j

When OEP-0014 is accepted, this document replaces these clauses of
`BLOCKS_2026.md` and the documents it extends:

- the type productions of `ARRAYS_2026.md` section 3 and `MODULAR_2026.md`
  section 3, the binding production of `BINDINGS_2026.md` section 3, the loop
  production of `BLOCKS_2026.md` section 3, and the primary production of
  `ARRAYS_2026.md` section 3 as later documents extend it, by section 3;
- the name rules for bindings and accumulators, for the names of a pattern,
  by section 4;
- the typing of names, operators, conversions, comparisons, indices, and
  typed leaves, for tuples, by section 5; and
- the Core records and the step table, by sections 7 and 8.

Every source that S3j accepts has no tuple type, tuple, projection, or
pattern, so S3k accepts it with the same Core values and the same output
bytes. A source that S3j rejects gets the same diagnostics, with these
exceptions:

- a `(` in a type position was `ORC0101`, "expected an identifier for the
  parameter type" (or the binding's, accumulator's, or result's); a `,` after
  a group's expression was `ORC0101`, "expected `)` to close the group"; a `.`
  after a name or a call was `ORC0101` at the `.`, with the message of what
  was expected after the name; `let (` was parsed as a call of a function
  named `let` and failed as that call, and `with (` was `ORC0101`, "expected
  an identifier for the accumulator". These forms are now accepted, or
  rejected by sections 3 through 5; and
- a comparison whose first typed leaf is an array or a fill written out, as
  in `[1, 2] == x`, was `ORC0214`, "an array literal cannot have type `Bool`",
  at the literal; it is now `ORC0215`, "`==` is not defined for an array", at
  the operator, as the comparison of an array name is.

## 12. Explicit non-claims and future work

This slice defines no tuple of tuples, no array of tuples, no tuple of one
element or of none, no equality, order, or other operator on whole tuples, no
update of one element of a tuple, no pattern that nests, skips an element, or
omits a type, no pattern in a parameter, no projection of an expression other
than a name or a call, no named fields or records, no inference of a
pattern's types, and none of the exclusions of `BLOCKS_2026.md` section 12
that this document does not lift.

A tuple of several values is a statement about mathematical values, not about
memory: nothing here fixes a layout, an ABI, or a calling convention for a
backend, and the reference evaluator is not constant-time
(`CONDITIONS_2026.md` section 15).

A fixture that reproduces a standard's example value is not thereby a verified
transcription of that standard. Tests establish the tested behavior of one
implementation at one revision; they do not prove semantic soundness,
completeness, or implementation independence.

Adding any excluded behavior requires a later accepted decision, a normative
specification, positive and negative conformance cases, resource analysis, and
migration review.
