# Orange 2026 nested arrays specification

Status: proposed S3s semantics under OEP-0023, in owner review; not accepted

Edition: `2026`

Snapshot: 2026-10-01

This document defines S3s: fixed rectangular arrays of scalar rows. It is a
delta over proposed S3r in [`AMOUNTS_2026.md`](AMOUNTS_2026.md) and the
documents it extends. Clauses not changed here retain their earlier meaning.
The implementation is provisional evidence for review. This text becomes
normative only when the owner accepts
[OEP-0023](governance/oeps/OEP-0023-orange-2026-nested-arrays.md); it accepts no
D-004 candidate, proof foundation, backend, or target.

The terms **must**, **must not**, and **may** are normative in this document.

## 1. Rank, axes, and scalar limits

An array must have rank one or two. Its scalar leaf type is `Int`, `Bool`,
`Word[8]`, `Word[16]`, `Word[32]`, `Word[64]`, or an admitted `Mod[m]`.
A rank-one array holds scalars; a rank-two array holds rank-one arrays of
one identical type and length. Every axis must have length from 1 through
65,536, and the total number of scalar leaves must be at most 65,536.
For a matrix of r rows and c columns, this means r × c ≤ 65,536.

Arrays must not hold tuples or matrices. Rank three, ragged rows, empty
axes, and excessive scalar counts are rejected before evaluation. An
invalid declared axis or a declared product exceeding the scalar limit is
`ORC0221`; an array of tuples or an attempted third axis is `ORC0203`.
Shape checks must use bounded arithmetic; a multiplication overflow must not
admit a shape.

An attempted third axis reports "`Grid` already has two array dimensions"
for a matrix alias `Grid`, labeled "arrays have at most two dimensions",
with a secondary label "this length would add a third dimension" at the
outer length and the note "a row holds scalars; a matrix holds rows of the
same type". An excessive declared product reports "an array shape has N scalar
elements, exceeding 65536", labeled "array shape exceeds the scalar element
limit", with a secondary label "outer axis length" and the note "both axes
are positive and their product is at most 65536".

The budgets of S3r remain. Rank adds no unbounded recursive type storage or
new nesting allowance. The source, parser, semantic-event, Core-node,
specialization, integer, and evaluation budgets continue to apply.

## 2. Alias construction and exact types

S3s adds no token or type grammar. Existing transparent type aliases build
the second axis:

```orange
type Row = Word[32]^4;
type Matrix = Row^4;
```

`Row^4` is four rows, each of exactly `Word[32]^4`. Aliases retain their
existing declaration-order and module-scope rules. The name of a scalar-row
alias, or a type parameter specialized to a scalar row, may precede `^n`.
A name specialized to a matrix may not receive another axis. Literal and
size-parameter lengths retain their prior checks, followed by the scalar
product limit.

Types are structural: different aliases of the same scalar type and both
lengths name the same type. Different row widths, outer lengths, ranks,
word widths, or residue moduli name different types. A matrix has no
implicit conversion to a flat array with the same scalar count.

The diagnostic and evaluation display spells a matrix as
`(Word[32]^4)^4`, with parentheses separating the axes. That display is not
new source syntax. Source uses aliases: `Word[32]^4^4` and a parenthesized
array type followed by `^4` remain parser errors.

## 3. Nested literals and fills

An array literal is checked against the expected array type. Its number of
elements must equal the outer length, and each element must have the exact
element type. For a matrix, each element is a scalar row:

```orange
type Row = Word[8]^2;
type Matrix = Row^3;
spec literal() -> Matrix { [[1, 2], [3, 4], [5, 6]] }
spec zero() -> Matrix { [[0; 2]; 3] }
```

Each row is checked independently in source order. A row of another length
is `ORC0222`, and a scalar or a value of another type where a row is required
is `ORC0214`. Literals do not infer a ragged or flattened type.

A fill `[v; n]` evaluates v once and repeats its value n times. At matrix
type, v must be one row and n must equal the outer length. Inner fills obey
the same rule at scalar-row type. Values are immutable: shared storage for
repeated rows must not make a later functional update change another row.

## 4. Indexing one axis at a time

`m[i]` selects one row; `m[i][j]` selects one scalar from that row.
S3s extends the existing suffix grammar to admit successive index
selections after a name, call, or tuple projection. Each suffix retains its
exact source spans and increases the expression-tree height within the
existing nesting budget; parsing a chain must use bounded stack space.
The type checker rejects a further selection after a scalar is reached.

The earlier suffix production becomes:

```text
suffix = projection? index* slice? ;
```

A suffix contains at least one projection, index, or slice. A terminal slice
may follow the indices; no projection or selection follows that slice.
Bind a slice with `let` before selecting from it. Groups and array literals
retain their earlier restriction: bind them before indexing or slicing.
The index of each selection is checked against that selection's own axis.
The outer index ranges from 0 through r − 1; the inner index ranges from
0 through c − 1. A known scalar total must not widen either range.

Literal, loop-derived, and data-derived indices retain S3d, S3e, and S3g's
rules. Every admitted index is proved in range before evaluation, and a
possibly out-of-range index remains `ORC0223`. Masking or arithmetic may
establish a range as before. Selecting a row returns its exact array type;
selecting its element returns its exact scalar type. No indexing operation
implicitly flattens a matrix.

Each selection evaluates its base then its index. It costs one step beyond
its operands, so `m[i][j]` contains two selections with their existing costs.

## 5. Element and row updates

`m with [i] = row` replaces one outer element, which is one complete row of
the matrix's exact element type. The base retains its type and value except
at that row. `row with [j] = scalar` replaces one scalar within a row.
To update one matrix scalar, write the two functional operations:

```orange
spec put(m: Matrix, x: Word[8]) -> Matrix {
  m with [1] = (m[1] with [0] = x)
}
```

Each index is checked against its own base's length. A replacement of the
wrong scalar domain or row width is rejected by the existing type or length
diagnostic. The base, index, and replacement evaluate in that order. An
update returns a new value; other rows and the original matrix retain their
values. There is no new multi-axis update syntax.

## 6. Slices and slice updates

A matrix slice `m[a..b]` selects consecutive complete rows. Its type has
the same exact row type and outer length b − a. An inner slice, as
`m[i][a..b]`, selects consecutive scalars from one row. Each bound is checked
against its own axis by S3l's existing static range and fixed-length rules.
Empty slices remain unsupported, and omitted bounds retain their meanings.

`m with [a..b] = rows` replaces a run of rows. The replacement must have
the same exact row type and outer length b − a; the result has the base's
type. A slice update of an individual row retains the scalar-row rule.
Mismatched row widths or scalar domains are rejected even when the total
scalar counts happen to match. The existing `ORC0222`, `ORC0223`, and type
diagnostics apply to the selected axis.

Evaluation order remains base, start, end, and replacement when present.
Slices copy values and functional slice updates preserve all other values.
There is no syntax selecting a rectangular window across several columns
of several rows in one operation.

## 7. Concatenation

`left ++ right` concatenates the outer elements of two arrays. For matrices,
their exact row types must agree: leaf domain and column count are equal.
The result's outer length is the sum of their outer lengths, and its scalar
total must satisfy section 1. A join of scalar rows retains S3l's meaning.

A matrix join does not join columns or flatten rows. A rank-one operand and
a rank-two operand cannot be joined, and equal scalar totals do not repair
different row widths. The existing required-type, found-length, operand
checking order, and diagnostics remain. Evaluation visits the left operand
before the right and preserves their row order.

## 8. Matrices within tuples

A tuple may contain a matrix wherever it could contain an array. Tuple
construction, projection, patterns, function arguments and results, and
loop accumulator state retain S3k's rules and exact structural types.
For example, `(Matrix, Word[8])` holds one matrix and one byte.
The existing tuple limit and prohibition on tuples holding tuples remain.
An array of such a tuple is still rejected; containing a matrix does not
make a tuple an admissible array element.

## 9. Size and type specialization

An outer length may use an existing size parameter. Each concrete instance
is checked against every axis and the scalar total, as if written out:

```orange
type Row = Word[8]^2;
spec zeros[n in 1..4]() -> Row^n { [[0; 2]; n] }
```

A finite type parameter may list a row or matrix alias. A parameter
specialized to a row may be used as an array element; one specialized to a
matrix cannot create a third axis. Concrete row and matrix types participate
in explicit calls, argument fitting, expected-result fitting, and qualified
calls through existing structural type equality. A wrong inner dimension
must not fit an instance merely because its outer length matches.

The rules for instance order, the first erroneous instance, ambiguity,
limits of four parameters and 256 instances, and per-source budgets remain.
S3s adds no parameter domain, shape variable, dependent type, type-declaration
parameter, or new inference rule.

## 10. Core, equality, and deterministic costs

Core keeps its existing node kinds. An array literal contains scalar or row
subtrees; `fill`, `index`, `select`, `update`, `concat`, `slice`, and
`slice_update` record exact element types by their existing rules. Selecting
an outer element has row type. The Core stays internal and noncanonical,
with no encoding, digest, certificate, or proof role.

The Rust `ArrayType` remains `Copy`. It records one scalar leaf type, an
optional row width, and an outer length, rather than an unbounded recursive
type. `element()` returns the exact scalar or row type, `length()` returns
the outer length, and `scalar_length()` returns the total scalar count.
`ArrayType::new` must reject invalid axes, excessive products, tuples, and
rank three. Core array values must have exactly the recorded number and
type of elements. Inconsistent Core stops evaluation with `ORC0301` and
no partial value set.

`==` and `!=` compare matrices structurally, recursively visiting every row
and every scalar pair, whether or not an earlier pair differs. They require
the same exact type. Matrix order comparisons remain unsupported. Tuple
equality applies the same rule to any matrix parts.

The equality cost is the sum of the rows' existing S3q comparison costs.
A matrix of r rows and c words or truth values costs r × ceil(c / 64)
comparison steps. An `Int` matrix sums one plus the 32-bit digits of the
longer value for each scalar pair; a `Mod[m]` matrix sums one plus the
32-bit digits of m for each scalar pair. Operand evaluation costs are added
as before. The location of the first difference never changes this cost.

All other evaluation costs retain the earlier operation rules. An array
literal costs one step per outer element beyond its element subtrees; each
nested row literal therefore contributes one step per scalar element in
addition to the scalars' own evaluation. A fill evaluates its row once
before repeating it.

| Operation | Steps beyond operands |
| --- | --- |
| array literal of n outer elements | n |
| `index` or `select`, at either axis | 1 |
| fill, element update, or slice update of n outer elements | ceil(n / 64) |
| concatenation with n outer elements in the result | ceil(n / 64) |
| slice with L outer elements in the result | ceil(L / 64) |

At matrix type, an outer copy operation copies immutable row references,
so its cost counts rows rather than all scalar leaves. A copy of a selected
scalar row counts that row's scalars. This preserves all rank-one costs and
charges each separately evaluated inner operation. Matrix equality, which
visits every leaf, follows the recursive comparison cost above. The step
budget remains a deterministic evaluation budget, not a timing or
constant-time guarantee.

## 11. Conversions preserve rank

S3s introduces no conversion. Plain `as` retains its existing scalar rules.
`as big` and `as little` accept a word or rank-one array of words and their
existing numeric targets; they must reject a matrix as either their source
or target. They must not infer a flattening order, concatenate rows, or
reinterpret the matrix's scalar count as a word sequence.

Converting one selected word row retains S3n and S3p's meanings, exact-bit
checks, and integer resource limit. Converting several rows requires source
code that explicitly defines their order. A matrix whose leaves are words
is not itself a rank-one array of words.

## 12. Compatibility, determinism, and claims

Every S3r source retains its types, values, output bytes, and evaluation
steps. S3s admits previously rejected scalar-row aliases followed by an
outer length and successive index suffixes; source diagnostics change for
those forms and for the now more specific rank and scalar-product rejections.
Tuples as array elements, new type syntax, and implicit array conversions
remain rejected. No command, option, token, or reserved word is added;
`orangec lex` is unchanged.

The same source and evaluation controls must produce the same status,
diagnostics, Core, and output bytes. `orangec eval` renders matrices as
nested arrays in row order and spells their types as section 2 requires.
`orangec test` applies its existing whole-value reports to matrix results.

This implements a bounded shape component of the
[roadmap](ROADMAP.md) and
[OEP-0022 development plan](governance/oeps/OEP-0022-crypto-language-development-plan.md).
The mathematical polynomial-pair fixture evaluates products in `Mod[3329]`
using explicit quadratic factors. For a row representing a0 + a1X modulo
X² − z, it computes `(a0b0 + za1b1, a0b1 + a1b0)`. The two row products
of `[[1, 2], [3328, 1]]` and `[[3, 4], [2, 3328]]`, at z = 17 and
z = 3312 respectively, are `[[139, 10], [15, 3]]`, all entries modulo 3329.
This tests matrix representation and domain behavior; it does not establish
full ML-KEM, NTT, or standard conformance.
The slice makes no proof, soundness, compilation, ABI, timing, secrecy,
cryptographic-security, independent-review, or production-readiness claim.

Acceptance would replace the scalar-only array-element exclusions of the
earlier array, loop, tuple, size, type-parameter, and length documents with
sections 1 through 9, extend S3q equality recursively as section 10 states,
and retain S3n's conversion boundary explicitly as section 11 states.
Removing S3s requires reverting its constructor and analyzer behavior,
evaluator changes, fixtures, conformance runner, and coupled documents
together; newly admitted sources then regain their earlier rejection.

### S3s conformance rule index

The runner `compiler/crates/orangec/tests/s3s_conformance.rs` parses this
index and requires exact agreement with its executable evidence map.

| Rule ID | Clause | Executable obligation | Evidence layer |
| --- | --- | --- | --- |
| `S3S-01` | Section 1 | Arrays have rank one or two, nonempty axes of at most 65,536, and at most 65,536 scalar leaves; rank three, tuple elements, and excessive products are rejected before evaluation. | CLI and generated CLI |
| `S3S-02` | Section 2 | Existing aliases construct exact rectangular types structurally; matrix display distinguishes both axes and the parser gains no nested type syntax. | CLI and generated CLI |
| `S3S-03` | Section 3 | Nested literals and fills require the exact row type and outer length, reject ragged or mismatched rows, and preserve immutable repeated values. | CLI and generated CLI |
| `S3S-04` | Section 4 | Successive index suffixes parse with exact spans within the existing nesting budget; literal and computed indices are proved in range separately for each axis, and outer selection returns one exact row type. | CLI and generated CLI |
| `S3S-05` | Section 5 | Outer updates replace one exact row and composed inner updates replace one scalar while preserving the base and other rows. | CLI and generated CLI |
| `S3S-06` | Section 6 | Slices and slice updates act on one axis, preserve the exact element type, require a fixed nonempty run, and reject mismatched replacement shapes. | CLI and generated CLI |
| `S3S-07` | Section 7 | Concatenation joins outer elements only, requires identical row types, and checks the resulting outer length and scalar total. | CLI and generated CLI |
| `S3S-08` | Section 8 | Matrices participate in tuple construction, projection, patterns, and loop state while arrays of tuples remain rejected. | CLI |
| `S3S-09` | Section 9 | Size and type instances check every axis and scalar total, preserve exact structural fitting, and reject rank three and mismatched inner dimensions. | CLI and generated CLI |
| `S3S-10` | Section 10 | Existing Core operations carry exact row types; equality visits every scalar pair at the specified deterministic costs, and nested construction and copy operations retain their defined costs. | CLI and generated CLI |
| `S3S-11` | Section 11 | Matrix sources and targets are rejected by byte-order conversion rather than flattened, while selected scalar word rows keep their prior conversion behavior. | CLI and generated CLI |
| `S3S-12` | Section 12 | S3r values, bytes, and steps remain unchanged, and repeated identical valid or rejected inputs have identical status, diagnostics, and output bytes. | CLI and generated CLI |
