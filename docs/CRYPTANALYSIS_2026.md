# Orange 2026 cryptanalysis contract

Status: pre-alpha reference tooling under owner direction; no semantic
acceptance, security claim, proof or release is recorded here

Edition: `2026`

Snapshot: 2026-10-06

`orangec analyze` evaluates one checked function at every input and reports
the exact properties a cryptanalyst first asks of an S-box or a Boolean
function: how differences propagate, how far it is from every linear
function, its algebraic degree, the boomerang connectivity of a permutation,
the implicit equations its graph satisfies, and its cycle structure. The
function is the Orange source itself, so the numbers describe the function
the program computes, not a table copied beside it.

Every property is computed by complete enumeration of the function's values,
so every reported number is exact for that function. None is sampled,
estimated or bounded, and none is a claim about the security of a cipher
that uses the function. The implemented language marker remains S3u; this
tool adds no syntax and changes no meaning of any program.

## Command

```text
orangec analyze --function MODULE::NAME [--instance N[,N...]] [--bits N[,M]] [--table TABLE] [--steps N] [--stats] SOURCE|-
```

The source and its imported modules must pass complete lexical, syntactic
and semantic validation first. `--function` and `--instance` select one
function exactly as they do for [`orangec replay`](WITNESS_REPLAY_2026.md):
a qualified name, and the complete numeric vector of its size values and
zero-based type-domain positions, empty when it has none. A function that
does not exist at that instance is `ORC1016`. Named tests are never selected.

`--steps N` is the step budget of each call, from 1 to 1,073,741,824
(default 1,048,576); each input is a fresh call with the whole budget.
`--stats` writes the number of calls, the total steps and the steps of the
largest call to standard error. `--edition 2026` and `--` keep their
ordinary meanings. `--bits` and `--table` belong to `analyze` alone, and each
of them, like `--function`, `--instance` and `--steps`, may appear at most
once; a malformed or misplaced option is a usage error with exit status 2.

## Analyzed functions

A function can be analyzed when it has exactly one parameter, of type
`Word[8]`, `Word[16]`, `Word[32]` or `Word[64]`, and its result is a word of
one of those widths or `Bool`. Any other shape is `ORC1016`, "analysis
requires one word parameter and a word or Bool result".

The function is read as F from n input bits to m output bits.

- Without `--bits`, n is the width of the parameter and m the width of the
  result, 1 for `Bool`.
- `--bits N` sets n = N and m = N, or m = 1 for a `Bool` result.
- `--bits N,M` sets n = N and m = M.

N and M are canonical decimal numbers from 1 through 16: no sign, no leading
zero, no space. An n or m wider than its type, or n or m above 16, is
`ORC1017`. So a 4-bit S-box held in the low bits of a byte is analyzed with
`--bits 4`, DES's 6-bit to 4-bit S1 with `--bits 6,4`, and a `Word[32]`
function needs `--bits` to choose a slice of at most 16 bits.

Input x, for every x from 0 through 2^n - 1, is passed as the word whose
value is x. Bit i of x and of F(x) is the bit of weight 2^i; `Bool` `true`
is 1. Every result must be below 2^m. The first input whose result is not,
in increasing order of inputs, is `ORC1017`, naming the input and the
result in hexadecimal: analysis never drops bits silently. A call that
exhausts its steps is the evaluator's `ORC0301`, with notes naming the
`--steps` option and the input at which analysis stopped. Nothing is
printed to standard output unless every call succeeds.

## Properties

Below, x and a range over n-bit values, y and b over m-bit values,
a · x is the parity of the bits of x selected by a, and wt(a) is the number
of bits set in a. N = 2^n.

### Values

- **bijective** (n = m): F is a permutation.
- **image** (n = m and not bijective): the number of distinct values F
  takes, of 2^m.
- **balanced** (n > m): every output value is taken 2^(n-m) times.
- **injective** (n < m): no value is taken twice.
- **weight** (m = 1): the number of x with F(x) = 1, of N.
- **fixed points** (n = m): the number of x with F(x) = x.
- **cycle type** (permutations): the lengths of the cycles of F, longest
  first, a length k repeated r times written k^r.

### Differences

The difference distribution table is DDT(a, b) = #{x : F(x) ⊕ F(x ⊕ a) = b}.

- **differential uniformity**: the largest DDT(a, b) with a ≠ 0. It is
  followed by the probability DDT(a, b) / N of its best differential, in
  lowest terms with a power-of-two denominator, and the number of pairs
  (a, b) that reach it.
- **differential spectrum**: how many entries DDT(a, b) with a ≠ 0, b
  anything, take each value, zeros included.
- **differential branch** (m > 1): the least wt(a) + wt(b) over a ≠ 0 with
  DDT(a, b) > 0.
- **absolute indicator** (m = 1): the largest |N - 2 · DDT(a, 1)| over
  a ≠ 0, the largest autocorrelation of a Boolean function. It is 0 exactly
  for bent functions.

### Correlations

The Walsh coefficient is W(a, b) = Σ_x (-1)^(b · F(x) ⊕ a · x), and the
linear approximation table holds LAT(a, b) = W(a, b) / 2, the number of x
with b · F(x) = a · x less N / 2.

- **linearity**: the largest |W(a, b)| with b ≠ 0, followed by the
  **nonlinearity** N / 2 - linearity / 2, the distance to the nearest affine
  function of any component, and the **correlation** linearity / N of the
  best linear approximation.
- **walsh spectrum**: how many coefficients W(a, b) with b ≠ 0, a anything,
  take each absolute value.
- **linear branch** (m > 1): the least wt(a) + wt(b) over b ≠ 0 with
  W(a, b) ≠ 0. Mask a = 0 counts; it is reached only by an unbalanced
  component.
- **correlation immunity** (m = 1): one less than the least wt(a) over
  a ≠ 0 with W(a, 1) ≠ 0, or n when there is none.

### Algebra

Each output bit y_j has a unique algebraic normal form, a sum over GF(2) of
monomials x_u = Π_{i in u} x_i. Its degree is the largest wt(u) in that sum,
and 0 for a constant.

- **algebraic degree**: the largest degree of an output bit. For m > 1 it is
  followed by "every component" when every component b · F, b ≠ 0, has that
  degree, or by "components d to D" giving the least and the largest degree
  over the components.
- **inverse degree** (permutations): the algebraic degree of F⁻¹.
- **quadratic equations**: the dimension of the space of equations of
  degree at most 2 in the n + m variables x_0 … x_(n-1), y_0 … y_(m-1) that
  every pair (x, F(x)) satisfies: the number of such monomials,
  1 + v + v(v - 1) / 2 for v = n + m, less the rank over GF(2) of their
  values at the N points. It is followed by the dimension of the
  **bi-affine** equations, those using only 1, x_i, y_j and x_i y_j.
- **boomerang uniformity** (permutations): the largest entry of the
  boomerang connectivity table
  BCT(a, b) = #{x : F⁻¹(F(x) ⊕ b) ⊕ F⁻¹(F(x ⊕ a) ⊕ b) = a} with a ≠ 0 and
  b ≠ 0.

## Output

Standard output starts with the qualified name, the instance and the
shape, `MODULE::NAME[INSTANCE]  n bits to m bits`, then a blank line and the
properties in four groups (values, differences, correlations, algebra), one
per line, each label padded to 26 columns, groups separated by one blank
line. A spectrum lists each value with its count, `value: count`, separated
by two spaces in increasing order of value; a spectrum of more than 16
distinct values is summarized as `K distinct values from LOW to HIGH`. The
exact output of each fixture is pinned in
[`analyze.rs`](../compiler/crates/orangec/tests/analyze.rs).

`--table` prints one complete table instead of the summary:

- `values`: F(x) in hexadecimal, 16 values to a line after the first input
  of the line;
- `ddt`, `lat`, `bct`: one row per a and one column per b, both in
  hexadecimal, entries right-aligned; `bct` requires a permutation and is
  `ORC1017` otherwise;
- `anf`: one line per output bit, `y0 = 1 + x0 + x1x2`, monomials in
  increasing degree and, within a degree, increasing mask.

Tables are produced for n and m of at most 10 bits; above that `--table` is
`ORC1017` and the summary remains available.

## Limits

n and m are at most 16. Each summary property is computed only when its
cost, counted in elementary operations, is at most 2^32; otherwise its line
reads `not computed: about 2^k operations, over the limit of 2^32`, with k
rounded up, and the other properties are still reported. The costs are:

| Property | Elementary operations |
| --- | --- |
| differences | 2^(2n) |
| correlations | 2^m · 2^n · (n + 1) |
| algebra | 2^m · ⌈2^n / 64⌉ · (n + 1) + 2m · 2^n · n |
| quadratic equations | M² · ⌈2^n / 64⌉ with M = 1 + v + v(v - 1) / 2 |
| boomerang uniformity | 2^(3n) |

So every property of an 8-bit S-box is computed, a 12-bit permutation is
summarized without its boomerang uniformity, and a 16-bit function is
summarized without its correlations either. Evaluation is bounded by
`--steps` per call and by the 2^16 calls at most that one analysis makes.

## Published values

The fixtures in [`compiler/fixtures/analyze`](../compiler/fixtures/analyze)
compute each S-box from its standard, and the summaries agree with the
values their designers and analysts published.

| Fixture | Command | Reported, as published |
| --- | --- | --- |
| AES S-box, from GF(2^8) inversion and the affine map | `--function aes::sbox` | differential uniformity 4, nonlinearity 112, degree 7 (Daemen and Rijmen, *The Design of Rijndael*); boomerang uniformity 6 (Cid et al., EUROCRYPT 2018); 39 quadratic equations, 23 bi-affine (Courtois and Pieprzyk, ASIACRYPT 2002) |
| PRESENT S-box | `--function present::sbox --bits 4` | differential uniformity 4, no single-bit differential (branch 3), best correlation 2^-1 (Bogdanov et al., CHES 2007); 21 quadratic equations |
| Ascon S-box | `--function ascon::sbox --bits 5` | degree 2, differential and linear branch 3 (Dobraunig, Eichlseder, Mendel and Schläffer, Journal of Cryptology 2021) |
| Keccak χ on 5 bits | `--function ascon::chi --bits 5` | the same spectra as Ascon's S-box, branches 2 |
| DES S1 | `--function des::s1 --bits 6,4` | balanced; differential uniformity 16, the entry DDT(0x34, 0x2) of Biham and Shamir's pairs table (CRYPTO 1990) |
| Bent function x0x1 + x2x3 | `--function boolean::bent --bits 4` | nonlinearity 6, the largest of any 4-bit Boolean function, and absolute indicator 0 |

Each number in the tests was also recomputed, while the tool was built, by
an independent implementation of the definitions above that shares no code
with `orangec`.

## What a result means

A result is exact for the function as the reference evaluator computes it,
and only for the selected instance and bit widths. It depends on the
evaluator and on this tool's arithmetic, neither of which is verified. It
says nothing about timing, side channels, a cipher's other layers, or any
attack. Low differential uniformity or high nonlinearity is a property a
designer asks for, not evidence that a design is secure.

## Later slices

This is the first slice of a cryptanalysis track. Later slices may add the
branch number and maximum-distance-separable check of a linear layer over
words, diffusion and avalanche counts, trail bounds for small
substitution-permutation networks, and checks of the properties claimed in
[`algorithms/`](../algorithms). Each will extend this contract and keep its
rule: exact numbers from complete enumeration, or a reported limit.
