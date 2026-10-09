# Orange 2026 array dimensions specification

Status: proposed S3u semantics under OEP-0025, in owner review; not accepted

Edition: `2026`

Snapshot: 2026-10-04

This document defines S3u: arrays of three and four dimensions, and update
paths that replace one element several dimensions deep. It is a delta over
proposed S3t in [`STATIC_MODULI_2026.md`](STATIC_MODULI_2026.md) and the
nested-array rules of S3s in [`NESTED_ARRAYS_2026.md`](NESTED_ARRAYS_2026.md).
Clauses not changed here retain their earlier meaning. The implementation is
provisional evidence for review. This text becomes normative only when the
owner accepts
[OEP-0025](governance/oeps/OEP-0025-orange-2026-array-dimensions.md); it
accepts no D-004 candidate, proof foundation, backend, or target.

The terms **must**, **must not**, and **may** are normative in this document.

## 1. Rank

An array must have rank one, two, three, or four. A rank-one array holds
scalars; an array of rank r + 1 holds arrays of rank r, all of one identical
type. The scalar leaf types are those of S3s, with S3t's residue domains.

Cryptographic objects reach these ranks as the standards draw them. AES
keeps a 4 × 4 state of bytes and Keccak a 5 × 5 state of lanes, both rank
two; an ML-KEM matrix is a k × k array of polynomials of 256 coefficients,
rank three; a batch of such matrices is rank four.

A fifth dimension is rejected before evaluation with `ORC0203`. For an alias
`Hyper` of rank four, the message is "`Hyper` already has 4 array
dimensions", labeled "arrays have at most 4 dimensions", with a secondary
label "this length would add a fifth dimension" at the outer length and the
note "a row holds scalars, and each `^LENGTH` after a named array type adds
a dimension of its rows". The same rejection applies in aliases, parameter
and result types, tuple fields, and every instance of a size or type
parameter.

## 2. Axes and scalar limit

Every axis must have length from 1 through 65,536, and the product of all
axes, the number of scalar leaves, must be at most 65,536. For a cube of
p planes, r rows and c columns, this means p × r × c ≤ 65,536; a 16 × 16 ×
16 × 16 array is admitted, and so is 1 × 1 × 1 × 65,536. A declared product
over the limit is `ORC0221`, reporting "an array shape has N scalar elements,
exceeding 65536", labeled "array shape exceeds the scalar element limit",
with a secondary label "outer axis length" and the note "every axis is
positive and the product of the axes is at most 65536". Each instance of a
size parameter is checked as if written out, and the first erroneous
instance is named. Shape checks must use bounded arithmetic; an overflow
must not admit a shape.

## 3. Construction, values, and display

S3u adds no type syntax. Each dimension is named by a `type` declaration
over the one before it:

```orange
type Zq = Mod[3329];
type Poly = Zq^256;
type Vector = Poly^2;
type Matrix = Vector^2;
```

`Word[8]^2^2^2` and a parenthesized array type followed by `^n` remain
parser errors, as in S3s. Repeated powers read as a tower of exponents in
mathematics; the alias names the object the standard names instead.

Types are structural: aliases of the same leaf type and the same axes, in
the same order, name the same type. Literals and fills nest to every rank,
and each level is checked against its own exact element type and length;
a ragged level is `ORC0222`. A fill evaluates its element once and repeats
it. The diagnostic and evaluation display parenthesizes each level from the
innermost out, as `((Mod[3329]^256)^2)^2`; that display is not source syntax.

`==` and `!=` compare every scalar pair at every rank, whether or not an
earlier pair differs, at the sum of the innermost rows' S3q comparison
costs. The position of the first difference never changes that cost.

## 4. Selection

`a[i]` selects one element of the outermost dimension; up to four successive
indices reach a scalar, as `m[i][j][k]`. Each index is checked against its
own axis by the rules of S3d, S3e, and S3g, and a possibly out-of-range
index is `ORC0223`. A further index after a scalar is `ORC0224`. Every
selection returns its exact element type and never flattens. Each costs one
step beyond its operands.

## 5. Slices and joins

A slice `a[x..y]` of any array selects consecutive elements of its outermost
dimension and keeps their exact element type; a slice of a selected row or
plane does the same one level in. `++` joins the outer elements of two arrays
of one exact element type, and the joined shape must satisfy section 2. A
slice update `a with [x..y] = b` replaces a run of outer elements. S3s's
grammar, costs, and restrictions remain: a selection does not follow a
slice, and no operation selects a rectangular window across several axes.

## 6. Update paths

An update may name one index per dimension it reaches:

```orange
c with [i][j][k] = v
```

The grammar of an update target becomes:

```text
update_target = "[" expression "]" ( "[" expression "]" ){0,3}
              | "[" range "]" ;
```

A path has two to four indices, and each is a single index. A slice inside
a path, an empty index, or a fifth index is `ORC0101`; the parser's note
says that an element of a row is updated with `x with [i][j] = v`, and a
run of a row as `x with [i] = (x[i] with [a..b] = v)`. Paths apply at
rank two as well; the AES and Keccak fixtures write their state updates as
`u with [r][c] = ...`, as the standards write them.

## 7. Path typing

The first index selects within the base, and each further index within the
element the one before it reached. Every index is checked against its own
axis by the rules of section 4. The value must have the exact type at the
end of the path, a scalar or a shorter array, and the update has the base's
type. A path whose indices reach past the scalars is `ORC0224`: "only an
array can be indexed, but this selects within `T`", labeled "`T` has no
elements", with a secondary label "this array has fewer dimensions" at the
base and a note counting the indices. A value of the wrong type or length
keeps its existing diagnostic, `ORC0214` or `ORC0222`. When the base itself
has the wrong type, the path is not checked against the required type.

## 8. Path meaning

A path means the nested updates it abbreviates:

```orange
c with [i][j][k] = v
// is
c with [i] = (c[i] with [j] = (c[i][j] with [k] = v))
```

Every element off the path keeps its value, and the base keeps its own.
Evaluation visits the base, then the indices in order, then the value; each
index is evaluated once. Core lowers a path to one `update_path` node that
records its number of indices; a single index keeps the `update` node.

## 9. Path cost

A path copies one array at each level it passes through. Beyond its
operands, it costs ceil(n / 64) steps, and at least one, for each level,
where n is the outer length of the array copied at that level:

| Operation | Steps beyond operands |
| --- | --- |
| update path through arrays of outer lengths n1, …, nk | ceil(n1 / 64) + … + ceil(nk / 64), each at least 1 |

At rank three, `c with [i][j][k] = v` on a 16 × 16 × 256 cube costs
1 + 1 + 4 steps beyond its base, three indices, and value. The nested form
also selects `c[i]` and `c[i][j]` and so costs more. The step budget
remains a deterministic evaluation budget, not a timing or constant-time
guarantee.

## 10. Parameters, tuples, and witnesses

Arrays of rank three and four may be parameters, results, `let` bindings,
loop accumulators, and tuple fields. A finite type parameter may list rank
three and four aliases, and a size parameter may supply any axis. Each
instance is checked by sections 1 and 2. The witness decoder of
`orangec replay` reads values of every rank in their canonical nested
spelling and never coerces a missing or extra element.

The Rust `ArrayType` remains `Copy`. It records the scalar leaf type, the
lengths of up to three inner dimensions, and the outer length, rather than
an unbounded recursive type. `element()` returns the exact element type,
`dimensions()` the rank, and `scalar_length()` the number of scalars.
`ArrayType::new` must reject a fifth dimension and an excessive product.

## 11. Unchanged boundaries

S3u changes no conversion: `as`, `as big`, and `as little` reject an array of
arrays as source or target, as in S3s. Arrays of tuples, direct repeated
powers, and type arguments spelled as repeated powers remain rejected.
No command, option, token, or reserved word is added; `orangec lex` is
unchanged.

## 12. Compatibility, determinism, and claims

Every S3t source retains its types, values, output bytes, and evaluation
steps. S3u admits a third and fourth dimension, which S3s rejected, and
update paths, which the parser rejected; diagnostics change for those forms
and for the dimension limit. The same source and evaluation controls must
produce the same status, diagnostics, Core, and output bytes, and `check`,
`eval`, and `test` report identical diagnostics for a rejected source.

This implements a bounded shape component of the [roadmap](ROADMAP.md) and
[OEP-0022 development plan](governance/oeps/OEP-0022-crypto-language-development-plan.md).
The fixtures reproduce FIPS 197's key expansion and cipher examples, FIPS
202's SHA3-256 examples, and FIPS 203's Appendix A zetas, and check that
ML-KEM-512's NTT is inverted by NTT⁻¹, reduces modulo each of its 128
quadratic factors, and computes A ∘ s as multiplication in
Z_q[X]/(X²⁵⁶ + 1). They test representation and arithmetic, not full
ML-KEM conformance. The slice makes no proof, soundness, compilation, ABI,
timing, secrecy, cryptographic-security, independent-review, or
production-readiness claim.

Removing S3u requires reverting its constructor and analyzer behavior, the
parser's path targets, the evaluator's path node, fixtures, conformance
runner, and coupled documents together; newly admitted sources then regain
their earlier rejection.

### S3u conformance rule index

The runner `compiler/crates/orangec/tests/s3u_conformance.rs` parses this
index and requires exact agreement with its executable evidence map.

| Rule ID | Clause | Executable obligation | Evidence layer |
| --- | --- | --- | --- |
| `S3U-01` | Section 1 | Arrays have rank one through four; a fifth dimension is rejected before evaluation in aliases, signatures, tuples, and every size instance. | CLI and generated CLI |
| `S3U-02` | Section 2 | Every axis is from 1 through 65,536 and the product of all axes is at most 65,536, checked at every rank and for every size instance. | CLI and generated CLI |
| `S3U-03` | Section 3 | Aliases build exact structural types at every rank; literals and fills nest exactly, display parenthesizes each level, and equality cost is independent of the first difference. | CLI and generated CLI |
| `S3U-04` | Section 4 | Successive indices reach a scalar at every rank, each proved in range against its own axis, and an index past the scalars is rejected. | CLI and generated CLI |
| `S3U-05` | Section 5 | Slices, joins, and slice updates act on the outermost dimension at every rank and keep the exact element type. | CLI and generated CLI |
| `S3U-06` | Section 6 | Update paths of two to four single indices parse; slices, empty indices, and a fifth index in a path are rejected. | CLI and generated CLI |
| `S3U-07` | Section 7 | Each path index is checked against its own axis, the value has the exact type at the path's end, and a path past the scalars is rejected. | CLI and generated CLI |
| `S3U-08` | Section 8 | A path gives the value of the nested updates it abbreviates at every position, preserving every other element. | CLI and generated CLI |
| `S3U-09` | Section 9 | A path costs the sum of ceil(n / 64), at least one, over the levels it copies, and an exact budget succeeds where one step fewer stops. | Generated CLI |
| `S3U-10` | Section 10 | Arrays of rank three and four serve as parameters, results, tuple fields, and type and size instances, and replay witnesses decode them exactly. | CLI and generated CLI |
| `S3U-11` | Section 11 | Conversions of arrays of arrays, arrays of tuples, and repeated power syntax remain rejected. | Generated CLI |
| `S3U-12` | Section 12 | Repeated identical valid or rejected inputs have identical status, diagnostics, and output bytes across commands. | CLI and generated CLI |
