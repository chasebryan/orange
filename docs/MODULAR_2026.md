# Orange 2026 modular arithmetic specification

Status: proposed S3i semantics under OEP-0012, in owner review; not accepted

Edition: `2026`

Snapshot: 2026-09-30

This document defines slice S3i of Orange 2026: the type `Mod[m]` of the
integers modulo a constant m, and `type` declarations, which name a type for
the rest of a module, as in `type F = Mod[(1 << 255) - 19];`. It is a delta
over the proposed S3h rules in [`MODULES_2026.md`](MODULES_2026.md), which are
a delta over [`LOOKUPS_2026.md`](LOOKUPS_2026.md) and the documents it
extends. Everything those documents define and this one does not mention is
unchanged.

It is proposed, not accepted. It becomes normative only when the owner accepts
[OEP-0012](governance/oeps/OEP-0012-orange-2026-modular-arithmetic.md), which
requires OEP-0011. At that point it replaces the S3h clauses listed in section
12. Until then, the compiler behavior it describes exists so that the proposal
can be reviewed against running code, and it establishes no accepted language
meaning. It accepts no D-004 candidate.

> [!NOTE]
> [`BLOCKS_2026.md`](BLOCKS_2026.md), proposed under OEP-0013, extends this
> document with `let` bindings at the start of a loop's step and of each branch
> of a conditional, so that the Montgomery ladder of X25519 names A, AA, B, BB,
> E, C, D, DA, and CB inside each step, as RFC 7748 does.
> [`TUPLES_2026.md`](TUPLES_2026.md), proposed under OEP-0014, adds tuples,
> which a `type` declaration may name, as `type Pair = (Int, Int);`, and whose
> elements may be residues. [`BYTES_2026.md`](BYTES_2026.md), proposed under
> OEP-0015, adds byte strings and slices, so that a Poly1305 key is written as
> RFC 8439 prints it and its message read sixteen bytes at a time.
> [`SIZES_2026.md`](SIZES_2026.md), proposed under OEP-0016, adds size
> parameters, so that Poly1305 is written once for every message length; a
> modulus is not a size. [`ORDER_2026.md`](ORDER_2026.md), proposed under
> OEP-0017, adds conversions in a byte order, so that a Poly1305 block or an
> X25519 coordinate is read as a residue in one conversion, as `(b ++ hex"01")
> as little P`, and a residue is written as bytes.
> [`TYPE_PARAMETERS_2026.md`](TYPE_PARAMETERS_2026.md), proposed under
> OEP-0018, adds type parameters, so that one function serves several moduli,
> as `spec pow[K in {F, P, Q}](x: K, e: Int) -> K`, each instance checked in its
> own ring; a modulus is still a constant. Every source this document accepts
> keeps its meaning under all six.

The terms **must**, **must not**, and **may** are normative in this document.

## 1. The idea

Much of public-key cryptography, and some symmetric cryptography, is
arithmetic in the integers modulo a fixed number. X25519 and Ed25519 compute
in the field of 2^255 - 19 elements, Poly1305 in the field of 2^130 - 5
elements, P-256 in a field of 2^256 - 2^224 + 2^192 + 2^96 - 1 elements, and
ML-KEM in the integers modulo 3329. The standards write that arithmetic
without reductions in sight: RFC 7748 writes `AA = A^2` and means the square
in its field.

Through S3h an Orange program wrote the same arithmetic with exact `Int`
values and a `% p` after every product. A transcription that forgets one is
still a valid program; it is merely wrong, and only a test vector notices.
S3i puts the ring in the type:

```orange
module x25519 {
  // RFC 7748 section 4.1: the field of p = 2^255 - 19 elements.
  type F = Mod[(1 << 255) - 19];
  // [x_2, z_2, x_3, z_3].
  type Ladder = F^4;

  spec ladder(x1: F, s: Ladder) -> Ladder {
    let a: F = s[0] + s[1];
    let aa: F = a * a;
    let b: F = s[0] - s[1];
    let bb: F = b * b;
    let e: F = aa - bb;
    let c: F = s[2] + s[3];
    let d: F = s[2] - s[3];
    let da: F = d * a;
    let cb: F = c * b;
    [aa * bb, e * (aa + 121665 * e), (da + cb) * (da + cb), x1 * ((da - cb) * (da - cb))]
  }
}
```

With S3i, `compiler/fixtures/s3i/` holds X25519 and Poly1305 written over
their fields, reproducing the first test vector of RFC 7748 section 5.2 and the
tag of RFC 8439 section 2.5.2 byte for byte, and the constants of ML-KEM,
Ed25519, and P-256 computed in the rings their standards define.

Three commitments shape every rule below.

- **The ring is in the type.** Every operation on `Mod[m]` gives a least
  residue, from 0 through m - 1, so no value leaves its ring and no program can
  forget a reduction.
- **Every operation is total.** Division multiplies by the inverse when there
  is one and gives 0 when there is none, which is the convention RFC 7748 and
  RFC 9380 use for field inversion, so `x_2 / z_2` is exactly the RFC's
  `x_2 * z_2^(p - 2)`.
- **Nothing changes type silently.** Two moduli are two types, a literal is
  written within its modulus, and a residue becomes a number or another
  residue only through `as`.

## 2. Scope and phase boundary

The phase boundary of `EXPRESSIONS_2026.md` section 2, as S3h extends it, is
unchanged. Every S3h source keeps its meaning (section 12).

## 3. Grammar

The module production gains `type` declarations after its `use` declarations
and before its functions, and a type may be `Mod[...]` or a declared name.

```text
Module     = "module" Identifier "{" UseDecl* TypeDecl* Function* "}" ;
TypeDecl   = "type" Identifier "=" Type ";" ;
Type       = ScalarType ( "^" INTEGER )? ;
ScalarType = "Int" | "Bool" | "Word" "[" INTEGER "]"
           | "Mod" "[" Expression "]" | Identifier ;
```

`type` is recognized by position, as `use` is: it begins a declaration only
at the head of a module, after its `use` declarations and before its first
function. Elsewhere it is an ordinary identifier. `Mod` followed by `[` takes
an expression, the **modulus**; `Mod` without `[` is an ordinary type name,
which section 4 rejects. A conversion target is never an array, as in S3g, but
it may be `Mod[...]` or a declared name.

A `use` declaration after a `type` declaration is `ORC0103` at `use`, "expected
a `type` declaration or a function", labeled "a `use` declaration cannot
follow a `type` declaration", with the note "a module's `use` declarations
come first, then its `type` declarations, then its functions". A `type`
declaration after a function is `ORC0103` at `type`, labeled "a `type`
declaration cannot follow a function", with the note "`type` declarations
come before a module's functions". A malformed declaration or modulus is
`ORC0101`; a modulus is closed by `]`, "`]` after the modulus". A module may
have at most 64 `type` declarations; the 65th is `ORC0106`, "module has more
than 64 `type` declarations".

A modulus is parsed one nesting level deeper than the type that holds it, and
its expression tree counts toward the height of the expression that holds the
type: a conversion or loop is one level above the taller of its operands and
the modulus of its type. The nesting-limit message now names moduli: "for
groups, calls, arrays, indices, loops, conditionals, updates, moduli, and
prefix operators".

## 4. Moduli

A modulus is a **constant**: an integer literal; a constant in parentheses;
or two constants joined by `+`, `-`, `*`, or `<<`. The value of `a << b` is
a * 2^b, and b must be from 0 through 16384. The value of every constant must
have at most 16384 significant bits, as an `Int` literal must. The modulus
must be from 2 through 2^521 - 1, so that every standardized prime field, that
of P-521 included, has a type.

- Anything else in a modulus, such as a name, a call, `/`, or a prefix `-`,
  is `ORC0232` at that part, "a modulus must be a constant from 2 through
  2^521 - 1", labeled "not a constant integer expression", with the note "a
  modulus is a constant built from integer literals with `+`, `-`, `*`, `<<`,
  and parentheses, as in `Mod[(1 << 255) - 19]`".
- A shift amount outside 0 through 16384 is `ORC0232` at the amount, labeled
  "a shift amount in a modulus is from 0 through 16384".
- A value of more than 16384 bits is `ORC0205` at that part, labeled "this
  value of the modulus is too large"; an oversized literal is `ORC0205` as in
  S3a.
- A value below 2 is `ORC0232` at the modulus, labeled "this modulus is N"
  when N has at most 64 bits and "this modulus is negative" otherwise; a value
  of 2^521 or more is `ORC0232`, labeled "this modulus has B bits".
- `Mod` without a modulus is `ORC0232` at `Mod`, "`Mod` requires a modulus",
  labeled "missing modulus".

Every modulus written in a module's `type` declarations and typed `spec`
functions, in their parameter, result, binding, conversion, and loop types, is
evaluated once, in source order, before the declarations of section 5 are
resolved and before any function is checked, and its diagnostics come first.
A modulus written within another, in a conversion or loop inside a modulus
that is therefore not a constant, is evaluated too, after the modulus that
holds it.
A modulus in an `impl` function or a function without a typed body is not
evaluated, as the other types of those functions are not checked.

**Identity.** `Mod[m]` and `Mod[n]` are the same type exactly when m and n are
the same integer, however they are written: `Mod[7]`, `Mod[0b111]`, and
`Mod[3 + 4]` are one type.

**Display.** A type is displayed with its modulus in a canonical form: in
decimal when m < 2^64; otherwise as `1 << k`, `(1 << k) - c`, or `(1 << k) +
c` when m is within c < 2^64 of a power of two, with the smaller c when two
powers qualify; and otherwise in lowercase hexadecimal with `0x`. So the field
of X25519 is `Mod[(1 << 255) - 19]`, that of secp256k1 is `Mod[(1 << 256) -
4294968273]`, and that of P-256 is
`Mod[0xffffffff00000001000000000000000000000000ffffffffffffffffffffffff]`.

## 5. Type declarations

`type NAME = TYPE;` declares NAME as a name for TYPE throughout its module.
The declarations are resolved in source order, after the moduli of section 4
and before any function's signature. A declaration's type may use the names
declared before it; a function's types may use every declared name.

- NAME must not be `Int`, `Bool`, `Word`, or `Mod`: such a declaration is
  `ORC0233` at the name, "`Int` is a built-in type", labeled "a `type`
  declaration cannot name a built-in type", with the note "the built-in types
  are `Int`, `Bool`, `Word[n]`, and `Mod[m]`".
- NAME must not be declared twice in a module: the second is `ORC0233`,
  "duplicate type name `NAME`", labeled "this declaration repeats a type
  name", with the first declaration's name as a secondary span, "first
  declaration is here", and the note "each `type` declaration of a module
  names a different type".
- The type of a rejected declaration is still checked. A name whose
  declaration was rejected, or whose type did not resolve, is not reported
  again where it is used.
- A name that is neither built in nor declared is `ORC0203`, as in S3a; the
  label lists the admitted types as "`Int`, `Bool`, `Word[8]`, `Word[16]`,
  `Word[32]`, `Word[64]`, `Mod[m]`, and the names of earlier `type`
  declarations". When the module declares the name later, the note is "`NAME`
  is declared by a later `type` declaration; a `type` declaration uses only
  the names declared before it".
- A declared name stands for its whole type. When it names an array type,
  `NAME^n` would be an array of arrays and is `ORC0203` at the type, "`NAME`
  is an array type, so this is an array of arrays", labeled "arrays of arrays
  are not part of Orange 2026", with the length as a secondary span. A width
  after a declared name, as in `NAME[3]`, is `ORC0203`.

A declared name is another spelling of its type, not a new type: `type A =
Mod[7];` and `type B = Mod[7];` name one type, the same as `Mod[7]`. Type names
and value names are separate, so a parameter may share a name with a type. A
declaration belongs to its module: another module sees the type, by its value,
in the signatures of the functions it calls, but not the name.

## 6. Residues

The values of `Mod[m]` are the residues 0 through m - 1. Arrays of residues,
as `Mod[m]^n`, are arrays as in S3d.

**Literals.** Where a `Mod[m]` value is required, an integer literal n must
satisfy -m < n < m. It denotes n when n >= 0 and m + n when n < 0, so `-1` is
m - 1 and `-0` is 0. Any other literal is `ORC0207` at its magnitude,
"literal is outside the range of `Mod[m]`", labeled "the literal's magnitude
is not less than the modulus", with the note "a literal of `Mod[m]` has a
magnitude n less than m, and `-n` stands for m - n; residues do not reduce
out-of-range literals". A `Bool` literal where a residue is required is
`ORC0214`.

**Operators.** For residues x and y of `Mod[m]`:

| Form | Value |
| --- | --- |
| `x + y`, `x - y`, `x * y` | the least residue of the exact sum, difference, or product |
| `-x` | m - x when x is not 0, and 0 when it is |
| `x / y` | x times the inverse of y when y is a unit (gcd(y, m) = 1), and 0 otherwise |
| `x == y`, `x != y` | `Bool` equality of the residues |

Both operands of a binary operator, and the operand of prefix `-`, have the
type required of the result, as for every S3 operator, and the operands of a
comparison have the type of its first typed leaf. So two moduli never meet in
one operator; a residue of another modulus is `ORC0214`.

`%`, `<`, `<=`, `>`, `>=`, the bitwise operators, shifts, rotations, `!`,
`&&`, and `||` are not defined for residues and are `ORC0215` at the operator.
The notes are: for an ordering, "residues are compared with `==` and `!=`;
they have no order, so compare least residues, such as `(x as Int) < (y as Int)`";
for `%`, "a residue is already reduced; `%` applies to `Int` and word values,
such as `(x as Int) % 16`"; and otherwise the S3 notes, of which prefix `!` now
reads "`!` negates a `Bool`; `-` negates an `Int` or a residue".

**Division.** For a prime modulus every nonzero residue is a unit, so `x / y`
is field division and `x / 0` is 0. For a composite modulus, dividing by a
residue that shares a factor with m gives 0: `Mod[256]` gives `1 / 2 = 0` and
`1 / 3 = 171`. A program that must know whether y is a unit can test `(y *
(1 / y)) == 1`.

**Conversions.** `as` converts among `Int`, the word types, and the residue
types, in every direction:

- to `Mod[m]` from `Int` or a word, the least residue of the operand's
  integer value (a word's is unsigned);
- to `Mod[m]` from `Mod[n]`, the least residue of the operand's least residue;
- from `Mod[m]` to `Int`, the least residue itself; to a word, the least
  residue modulo 2^n, as for an `Int`.

`as` does not convert to or from `Bool` (`ORC0215`, as in S3f), and a
conversion to a declared array type is `ORC0215` at the type, "`as` does not
convert to the array type `T`", labeled "`as` gives one `Int`, word, or
residue value". The labels of the other `as` diagnostics now read "`as`
converts one `Int`, word, or residue value".

**Indices.** A residue is not an index; `t[x]` for a residue x is `ORC0214`.
Its least residue is: in an `Int` index, `x as Int` ranges over 0 through m -
1, and in a word index, `x as Word[n]` ranges over 0 through m - 1 when m - 1
fits the word and over the whole word otherwise, under the S3g range rules.
So `t[x as Int]` for x of `Mod[7]` is proved in range for a table of seven
elements, and refused (`ORC0223`) for a table of six.

## 7. Diagnostics

| Code | Phase | Meaning |
| --- | --- | --- |
| `ORC0232` | semantic | a modulus is not a constant from 2 through 2^521 - 1, or `Mod` has none |
| `ORC0233` | semantic | a `type` declaration names a built-in type or repeats a type name |

`ORC0101`, `ORC0103`, and `ORC0106` gain the uses of section 3; `ORC0203`,
`ORC0205`, `ORC0207`, `ORC0214`, `ORC0215`, and `ORC0223` gain the uses of
sections 4 through 6. Every other code keeps its meaning.

## 8. Typed Reference Core

Core gains the type `Mod[m]`, which records m exactly, and the value kind of
a residue, which records its modulus and least residue. The node kinds are
unchanged: a residue literal is a literal node, the operators are unary,
binary, and comparison nodes typed with the residue type, and a conversion is
a conversion node recording its operand's type.

A `type` declaration leaves no trace in Core: every declared name is replaced
by its type, so a module that writes `F` and one that writes `Mod[(1 << 255) -
19]` have the same Core.

## 9. Evaluation

Reference evaluation computes every residue operation exactly as section 6
defines it and prints residues in decimal, as their least residues, and their
types in the display of section 4:

```text
fields::d: Mod[(1 << 255) - 19] = 37095705934669439343138083508754565189542113879843219016388785533085940283555
```

**Steps.** Let d be the number of 32-bit digits of m, ceil(bits(m) / 32).

| Operation | Steps |
| --- | --- |
| a residue literal, or reading a residue | 1 |
| `+`, `-`, prefix `-`, `==`, `!=` | 1 + d |
| `*` | 1 + 2d^2 |
| `/` | 1 + 64d^2 |
| `as` to `Mod[m]` | 1 + d times the digits of the operand's integer value |
| `as` from `Mod[m]` | 1 |

The costs follow the work of the reference evaluator: a product is reduced by
a division of 2d digits by d, and an inverse is found by the extended
Euclidean algorithm in at most about 46d rounds of O(d) work. The per-source
budget of 1,048,576 steps is unchanged.

## 10. Resource limits and failure

The S3h budgets remain. S3i adds the following.

- A module has at most 64 `type` declarations (section 3).
- A modulus opens one nesting level, and its tree counts toward the height of
  the expression that holds its type (section 3).
- Every part of a modulus costs one semantic event, and each of its literals
  the prefix and digit events of S3a. A `type` declaration costs one event for
  its name's lookup and the events of its type. A use of a declared name costs
  what a use of `Int` costs.
- A modulus has at most 521 bits, and every value computed for one at most
  16384 (section 4).
- An allocation failure while evaluating a modulus or a residue literal is
  `ORC0209`, "exact integer storage allocation failed", and gives no Core; one
  during evaluation stops it with no values.

A syntax tree whose spans, including those of its `type` declarations and of
every modulus, do not all belong to the source it is supplied with is
`ORC0210` for that source, and nothing is checked.

## 11. Determinism and conformance

The determinism requirement of `EXPRESSIONS_2026.md` section 15 applies
unchanged. The S3i conformance runner
(`compiler/crates/orangec/tests/s3i_conformance.rs`) parses this index and
requires exact agreement with its evidence map, under the same rules as the
S3b through S3h runners.

### S3i conformance rule index

| Rule ID | Clause | Executable obligation | Evidence layer |
| --- | --- | --- | --- |
| `S3I-SYNTAX-01` | Section 3 | `type` declarations follow the `use` declarations and precede the functions, at most 64 of them; `Mod[...]` takes an expression that counts toward nesting and height; malformed forms are `ORC0101`, `ORC0103`, or `ORC0106`. | CLI and parser unit |
| `S3I-MODULUS-01` | Section 4 | A modulus is a constant of literals, `+`, `-`, `*`, `<<`, and parentheses from 2 through 2^521 - 1, evaluated once before any type is resolved; otherwise `ORC0232` or `ORC0205`. Moduli are equal by value and displayed canonically. | CLI and unit |
| `S3I-NAMES-01` | Section 5 | `type` declarations resolve in order to their types, are not built-in names or repeated (`ORC0233`), use only earlier names, and name no array of arrays (`ORC0203`). | CLI and unit |
| `S3I-LITERAL-01` | Section 6 | A literal of `Mod[m]` lies strictly between -m and m and denotes its residue; otherwise `ORC0207`. | CLI and unit |
| `S3I-OPERATOR-01` | Section 6 | Residues have `+`, `-`, `*`, `/`, prefix `-`, `==`, and `!=` of one modulus; every other operator is `ORC0215` with the specified notes, and another modulus is `ORC0214`. | CLI and unit |
| `S3I-DIVISION-01` | Section 6 | `x / y` is x times the inverse of y when y is a unit and 0 otherwise. | CLI and unit |
| `S3I-CONVERT-01` | Section 6 | `as` converts among `Int`, words, and residues by least residues, and not to `Bool` or an array type. | CLI and unit |
| `S3I-INDEX-01` | Section 6 | A least residue indexes through `as Int` over 0 through m - 1, or through a word over its range. | CLI and unit |
| `S3I-CORE-01` | Section 8 | Core records residue types and values exactly, with declared names replaced by their types. | Unit and CLI observation |
| `S3I-EVAL-01` | Section 9 | Residue arithmetic matches a reference and the specified step costs; X25519 and Poly1305 over their fields and the ML-KEM, Ed25519, and P-256 constants match their published values. | CLI and unit |
| `S3I-RES-01` | Section 10 | Type declarations, moduli, and their events and allocations are bounded as specified, and foreign spans are `ORC0210`. | Generated CLI and unit |
| `S3I-COMPAT-01` | Section 12 | S3h sources keep their meaning, Core values, and output bytes, with only the specified message changes. | CLI and unit |
| `S3I-DETERMINISM-01` | Section 11 | Repeated identical inputs produce identical status, diagnostics, and output bytes. | CLI and unit |

## 12. Relationship to S3h

When OEP-0012 is accepted, this document replaces these clauses of
`MODULES_2026.md` and the documents it extends:

- the module production of `MODULES_2026.md` section 3 and the type
  production of `EXPRESSIONS_2026.md`, as S3c and S3d extend it, by section 3;
- the admitted types of `SEMANTICS_2026.md` and the documents extending it,
  by sections 4 and 5;
- the meaning of literals, operators, comparisons, conversions, and indices,
  for residue types, by section 6; and
- the Core types and values and the step table, by sections 8 and 9.

Every source that S3h accepts is accepted by S3i with the same Core values and
the same output bytes: it declares no type and writes no `Mod[...]`. A source
that S3h rejects gets the same diagnostics, with these exceptions:

- a `type` declaration at the head of a module was `ORC0103` at `type`; it is
  now accepted, or rejected by sections 3 and 5;
- `Mod[m]` with a literal m was `ORC0203`, and with any other modulus
  `ORC0101`; it is now accepted, or rejected by section 4;
- the label of `ORC0203` for an unsupported type lists the admitted types of
  section 5;
- the labels of the `as` diagnostics, the note of `!` on a non-`Bool`, the
  note of an array operator ("operators apply to `Int`, `Bool`, word, and
  residue values; apply them to elements, such as `x[0]`"), the note of an
  array of arrays ("an array's elements are `Int`, `Bool`, words, or
  residues; arrays of arrays are not part of Orange 2026"), and the
  nesting-limit message name residues or moduli.

## 13. Explicit non-claims and future work

This slice defines no generic moduli or type parameters, no moduli computed at
run time, no named constants, no extension fields, no exponentiation
operator, no square roots or Legendre symbols, no Montgomery or other
representation, no order, bits, or remainder on residues, no implicit
conversion, no primality check, no new types by declaration (a declared name
is another spelling of its type), no types exported between modules, and none
of the exclusions of `MODULES_2026.md` section 12 that this document does not
lift. Moduli of more than 521 bits, such as an RSA modulus, have no type.

`Mod[m]` is a ring for every m. Nothing checks that m is prime, and `/` for a
composite modulus gives 0 for every non-unit, which a program must expect.

The reference evaluator is not constant-time. Its residue arithmetic runs in
time that depends on the values, and nothing here concerns code generation;
a backend's obligations for residues are left to D-011 and later slices.

A fixture that reproduces a standard's example value is not thereby a verified
transcription of that standard. Tests establish the tested behavior of one
implementation at one revision; they do not prove semantic soundness,
completeness, or implementation independence.

Adding any excluded behavior requires a later accepted decision, a normative
specification, positive and negative conformance cases, resource analysis, and
migration review.
