# Orange 2026 universal word specification

Status: proposed S3v semantics under OEP-0026, in owner review; not accepted

Edition: `2026`

Snapshot: 2026-10-05

This document defines S3v: one function checked at every word width, with its
ordinary size parameters checked for every affine value. It is a delta over
proposed S3t in [`STATIC_MODULI_2026.md`](STATIC_MODULI_2026.md), the size
rules in [`SIZES_2026.md`](SIZES_2026.md), and the listed-type rules in
[`TYPE_PARAMETERS_2026.md`](TYPE_PARAMETERS_2026.md). Clauses not changed here
retain their earlier meaning. Implementation supplies provisional evidence for
owner review; it does not accept
[OEP-0026](governance/oeps/OEP-0026-orange-2026-universal-words.md) or choose a
semantic stratum, proof foundation, or target.

The terms **must**, **must not**, and **may** are normative in this document.

## 1. Word parameters

A `spec` may declare one parameter `W: Word` among its bracket parameters:

```orange
spec inc[W: Word](x: W) -> W { x + 1 }
```

`W` stands for `Word[8]`, `Word[16]`, `Word[32]`, and `Word[64]`, in that
order. A call names one of those widths, as in `inc[Word[64]](256)`. The name
`Word` in that position is the parameter form; it is not a value and not a
finite type list. Size parameters keep `in`, and a listed type parameter keeps
`in { ... }`.

A function has at most one word parameter. That parameter is not combined with
a listed type parameter `K in { ... }`. Its name is not a built-in type and not
a declared `type` name. A second word parameter, or a word parameter beside a
listed type parameter, is `ORC0243`. The function may also declare ordinary
size parameters, at most four bracket parameters in total, as before.

`in` remains required for a size range and for a finite type list. S3v adds
the colon form only for `Word`.

## 2. Checked at every width

The body is checked at each of the four widths. A literal word value must fit
the narrowest width, `Word[8]`, so `x + 256` in a result of type `W` is
`ORC0207` even when every call writes `Word[64]`. A literal shift amount is
checked the same way: an amount written as the integer 8 does not fit a
rotation of `Word[8]`. An amount computed from data keeps the S3r rule, so a
shift by the width or more yields 0 and a rotation is modulo the width.

An uncalled definition receives the same width checks as a called definition.
The checker does not build a Core function for every width in advance. It
records a specialization when a call in the same module names that width and
those sizes, and it lowers only the specializations that calls name. One
module builds at most 256 such specializations. A call in another module may
use a specialization the declaring module already built; it must not ask that
finished module to build a new one (`ORC0243`).

## 3. Sizes checked for every affine value

A size parameter beside `W: Word` is a range `n in a..b`, excluding `b`, as
in S3m. The checker does not build one instance for every size. It checks the
body at each width and at the corners of the size box: each size at its first
value and at its last value. An affine use of those sizes attains its extrema
at those corners, so the corner check decides the range.

An index is affine when it is built from literals, loop indices, and the
function's sizes using `+`, `-`, `*`, and parentheses, and a product has one
factor that names no size. Division or remainder that names a size is not
affine. A non-affine index is `ORC0243`. An index that is affine and still
falls outside its array at a corner is the existing `ORC0223`.

Loop indices keep their existing interval proof. A loop `for i in 0..n` is
therefore checked at `n`'s high corner as well as its low corner.

## 4. Lengths stay invariant

An array length, a fill length, and any other length that must be one integer
for the whole function must not name a size of a word-parameter function.
`W^n` and `[x; n]` are `ORC0243`. A length written as an integer, such as
`W^25`, is the same array at every width and every size. A modulus expression
that names such a size is likewise rejected: a residue domain has one modulus.

## 5. Calls

A call writes one bracket entry for each bracket parameter, in order. The word
entry is one of `Word[8]`, `Word[16]`, `Word[32]`, and `Word[64]`. Any other
type is `ORC0241`. A size entry is an integer in that parameter's range; a
value outside the range is `ORC0238`. The wrong number of entries, including a
call that writes no brackets, is `ORC0239`.

Passing the word parameter through a call, as in `theta[W](a)`, uses the
caller's width. The callee is checked at every width on its own definition,
and this call lowers the specialization for the width the caller is checking
or evaluating.

Only those named specializations are lowered into Core. Their identities are
`[Word[8]]`, `[Word[64], 24]`, and the same form for the other widths and
sizes. Concrete finite instances of functions that have no word parameter keep
their existing identities and are still checked eagerly.

## 6. Keccak-p at every lane width

Keccak-p[b, nr] from FIPS 202 is one function of the lane type and the round
count. The state is a flat array of 25 lanes. Lane (x, y) is the element at
`x + 5y`. Theta, rho, pi, chi, and iota are each written once for `W: Word`.
The round count is `nr in 1..25`. Rho's offset `((t + 1) * (t + 2)) / 2` is a
computed rotation. Iota's round constants are the FIPS 202 LFSR, and a place
`2^j - 1` at or past the lane width shifts to 0, so one walk serves every
width.

Two known answers are required from that one source:

- SHA3-256 of the three-byte message `abc`, which is Keccak-p[1600, 24] with
  the FIPS 202 padding for a 136-byte rate, and whose digest begins
  `3a985da74fe225b2`.
- Keccak-f[200], which is Keccak-p[200, 18], applied to the all-zero state.
  The final state is the first "State after permutation" of XKCP's
  intermediate-values file
  `tests/TestVectors/KeccakF-200-IntermediateValues.txt` at commit
  `4017707cade3c1fd42f3c6fa984609db87606700` (2018-03-16):
  `3c 28 26 84 1c b3 5c 17 1e aa e9 b8 11 13 4c ea a3 85 2c 69 d2 c5 ab af ea`.
  The source URL is
  `https://github.com/XKCP/XKCP/blob/4017707cade3c1fd42f3c6fa984609db87606700/tests/TestVectors/KeccakF-200-IntermediateValues.txt`.

These two values are evidence for this implementation at those inputs. They
are not a claim that every Keccak, SHA-3, or SHAKE input is implemented, and
they are not a constant-time or native-code claim.

## 7. Compatibility and claim boundary

A function that uses only finite size ranges, or only a listed type parameter,
keeps its S3m and S3o meaning, including eager instances and the 256-instance
cap. S3t modulus expressions are unchanged for those functions. A source that
S3t accepted has the same values and the same reference-evaluation costs under
S3v.

The corner check is the decision procedure for the affine expressions this
slice admits. It is not a general proof about a non-affine program, and a
non-affine use is rejected rather than interpreted. No type class, `type`
parameter, import, typed `impl` body, rank of three or more, variable-length
array, symbolic modulus, or D-004 candidate is introduced. No theorem,
certificate, leakage result, or release follows from these checks.

## Bounded normative rule index

`compiler/crates/orangec/tests/s3v_conformance.rs` binds the following exact
rule IDs to executable CLI evidence. The index and runner must agree; the
negative corpus is part of the proposed semantic boundary.

| Rule | Normative subject | Evidence class |
| --- | --- | --- |
| S3V-01 | One `W: Word` parameter, standing for every word width, and not combined with a listed type parameter | CLI and generated CLI |
| S3V-02 | Every width checked, including an uncalled definition, and a literal that does not fit `Word[8]` rejected | CLI and generated CLI |
| S3V-03 | Sizes checked for every affine value; a non-affine index or a corner past an array rejected | CLI and generated CLI |
| S3V-04 | An array or fill length that names a size rejected | CLI and generated CLI |
| S3V-05 | A call names one specialization; a size outside the range, a type that is not a word width, or the wrong number of brackets rejected | CLI and generated CLI |
| S3V-06 | One Keccak-p source reproduces SHA3-256 of `abc` and the XKCP Keccak-f[200] zero-state permutation | CLI |
| S3V-07 | Listed types and finite sizes retain their meaning; the corner check adds no proof, constant-time, or full SHA-3 claim | CLI and generated CLI |
