# Orange 2026 cryptanalysis contract

Status: pre-alpha reference tooling under owner direction; no semantic
acceptance, security claim, proof or release is recorded here

Edition: `2026`

Snapshot: 2026-10-06

`orangec analyze` evaluates one checked function at every input and reports
the exact properties a cryptanalyst first asks of an S-box or a Boolean
function: how differences propagate, how far it is from every linear
function, its algebraic degree, the boomerang connectivity of a permutation,
the implicit equations its graph satisfies, and its cycle structure. With
`--linear` it reads the matrix of a linear layer back from the function and
reports its branch numbers, whether it is maximum distance separable, and the
field its blocks multiply in. The function is the Orange source itself, so
the numbers describe the function the program computes, not a table copied
beside it.

Every property of an S-box or Boolean function is computed by complete
enumeration of the function's values, and every property of a layer from its
exact matrix, so every reported number is exact for that function. None is
sampled, estimated or bounded, and none is a claim about the security of a
cipher that uses the function. The one statement that is not complete is
named as such: a layer of more than 16 bits is checked to be affine only up
to its terms of degree 2. This tool leaves the implemented language marker
unchanged, adds no syntax, and changes no meaning of any program.

## Command

```text
orangec analyze --function MODULE::NAME [--instance N[,N...]] [--bits N[,M]] [--table TABLE] [--steps N] [--stats] SOURCE|-
orangec analyze --function MODULE::NAME [--instance N[,N...]] --linear [--word W] [--table matrix] [--steps N] [--stats] SOURCE|-
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
ordinary meanings. `--bits`, `--linear`, `--word` and `--table` belong to
`analyze` alone, and each of them, like `--function`, `--instance` and
`--steps`, may appear at most once. `--bits` does not combine with `--linear`,
`--word` requires it, and with it `--table` takes only `matrix`, which in turn
requires it. A malformed, misplaced or conflicting option is a usage error
with exit status 2.

## Analyzed functions

Without `--linear`, a function can be analyzed when it has exactly one
parameter, of type `Word[8]`, `Word[16]`, `Word[32]` or `Word[64]`, and its
result is a word of one of those widths or `Bool`. Any other shape is
`ORC1016`, "analysis requires one word parameter and a word or Bool result".

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

## Linear layers

`--linear` analyzes the selected function as a linear layer: a map over
GF(2) from n bits to the same n bits.

### Analyzed layers

A function can be analyzed as a layer when it has exactly one parameter, of a
word type or a one-dimensional array of words, and its result has the same
type. Any other shape is `ORC1016`, "linear analysis requires one parameter
of a word or array type and a result of the same type". n is the number of
bits of that type, at most 128; a wider type is `ORC1017`. Element i of an
array of w0-bit words holds bits i w0 through i w0 + w0 - 1, its bit 0 being
bit i w0 of the layer.

The n bits are grouped into k = n / w words of w bits, word c holding bits
c w through c w + w - 1. `--word W` sets w to 1, 2, 4, 8, 16, 32 or 64,
written in canonical decimal. Without it, w is the width of an array's
elements, or 8 for a single word. A w that does not divide n is `ORC1017`. So
AES MixColumns on `Word[8]^4` is four words of 8 bits, PRESENT's pLayer on a
`Word[64]` is analyzed over its sixteen nibbles with `--word 4`, and
`--word 1` counts single bits.

### Reading the matrix

The layer is evaluated at 0 and at each e_j, the input whose only set bit is
bit j. Its constant is c = F(0), and column j of its matrix M is
F(e_j) + c, so M x is the sum of the columns of the bits set in x. F is
affine exactly when F(x) = c + M x at every input x, and analysis checks
that, in increasing order of x:

- at every input, when n is at most 16;
- otherwise at every input of exactly two set bits. F(e_i + e_j) + F(e_i) +
  F(e_j) + F(0) is the coefficient of x_i x_j in the algebraic normal form of
  F, so this shows exactly that F has no term of degree 2, and nothing about
  terms of degree 3 or more. The summary says so.

The first input at which F differs from c + M x is `ORC1017`, naming the
input, F's value there and c + M x, each written as a value of the layer's
type, an array as its words in index order. A call that exhausts its steps
is `ORC0301`, with the same notes as above.

### Layer properties

Below, wt(x) is the number of nonzero words of x, and x + y adds over GF(2),
bit by bit.

- **checked**: `all N inputs` with N = 2^n, or `the N inputs of at most 2
  bits: no term of degree 2, higher degrees unchecked` with
  N = 1 + n + n(n - 1) / 2.
- **form**: `linear` when c = 0, otherwise `affine` and the constant c in
  hexadecimal.
- **rank**: the rank of M over GF(2), and whether M is invertible or
  singular.
- **fixed points**: the number of x with F(x) = x, the solutions of
  (M + I) x = c: `none`, `1`, or `2^d`.
- **involution**: whether F(F(x)) = x for every x, that is M M = I and
  M c = c.
- **xor count, row by row**: the sum over the n rows of M of one less than
  the row's weight, a zero row counting 0: the XOR gates of computing each
  output bit on its own, the naive count of the literature. Implementations
  that share terms between rows need fewer.
- **differential branch**: the least wt(x) + wt(M x) over x ≠ 0, the fewest
  active words on both sides of the layer. It is at most k + 1, and a layer
  that reaches k + 1 is maximum distance separable, marked `(MDS)`.
- **linear branch**: the least wt(b) + wt(M^T b) over b ≠ 0. Since
  b · (M x) = (M^T b) · x, these are the output mask b and input mask M^T b
  of the linear approximations of the layer.
- **field**, for 2 ≤ w ≤ 8: each irreducible polynomial p of degree w over
  GF(2), written with bit i the coefficient of x^i, such that every w x w
  block of M, the map from input word c to output word r, commutes with
  multiplication by x modulo p. Such a block is multiplication by a constant
  of GF(2^w), the polynomials over GF(2) modulo p: its image of 1. The line
  reads `GF(2^w) modulo p`, or `modulo each of` several, `every GF(2^w): each
  block is 0 or 1` when every constant is 0 or 1, or `none` when no p fits.
  The **field matrix** follows: the k x k constants in hexadecimal, row r
  and column c. For w over 8 the line reads `not searched for words of more
  than 8 bits`; for w = 1 it is left out, M being itself the matrix over
  GF(2).

The summary starts with `MODULE::NAME[INSTANCE]  n bits as k words of w bits`
and prints the first six properties, the two branch numbers and the field
lines in groups separated by one blank line. `--table matrix` prints M
instead: one line per output bit i, `yi`, then the coefficient of each input
bit x0, x1, … as `0` or `1`, a space before each word.

### Branch number search

Inputs are searched by their number t of nonzero words, t = 1, 2, …, each
word taking every nonzero value. For an invertible M, the pairs (x, M x) are
also found from the output side, by searching M^-1 at the same t: after
weights 1 through t on both sides, every pair not yet seen has more than t
nonzero words in x and in M x, so the search stops once 2 (t + 1) reaches
the least sum found. For a singular M it stops once t + 1 does. An MDS
layer of k words is thereby settled by inputs of at most k / 2 nonzero words.
The linear branch searches M^T and its inverse the same way.

Each side costs k 2^w operations for the tables of word images and
C(k, t) (2^w - 1)^t for the inputs of weight t. A branch number whose search
would exceed 2^32 operations is reported as not computed, with its cost, as
above. So a 128-bit layer of 16 bytes is searched to weight 2 in about 2^24
operations, while words of 32 or more bits are out of reach.

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
| AES MixColumns, one column | `--linear --function aes::mix_column` | branch number 5, maximum distance separable, the circulant matrix 02 03 01 01 over GF(2^8) modulo 0x11b (Daemen and Rijmen); naive XOR count 152 (Kranz, Leander, Stoffelen and Wiemer, ToSC 2017) |
| AES ShiftRows then MixColumns | `--linear --function aes::linear` | branch number 5 over the 16 bytes of the state, the bound of 5 active S-boxes in any two rounds (Daemen and Rijmen) |
| PRESENT pLayer | `--linear --word 4 --function present::player` | a permutation of bits (Bogdanov et al.): no XOR gate, branch number 2 over the S-boxes' nibbles |
| Midori64 MixColumn | `--linear --word 4 --function midori::mix_column` | an involutive binary matrix, almost MDS with branch number 4 (Banik et al., ASIACRYPT 2015) |

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

Slice A1 covered S-boxes and Boolean functions, and slice A2 linear layers.
Later slices may add diffusion and avalanche counts of whole primitives,
trail bounds for small substitution-permutation networks, and checks of the
properties claimed in [`algorithms/`](../algorithms). Each will extend this
contract and keep its rule: exact numbers from complete enumeration, or a
reported limit.
