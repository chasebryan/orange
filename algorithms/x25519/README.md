# X25519 (RFC 7748)

X25519 is the Diffie-Hellman function over Curve25519, the Montgomery curve
v^2 = u^3 + 486662 u^2 + u over the field of integers modulo p = 2^255 - 19.
Daniel J. Bernstein designed the curve and the function in 2006 ("Curve25519:
new Diffie-Hellman speed records", PKC 2006), and
[RFC 7748](https://www.rfc-editor.org/rfc/rfc7748), "Elliptic Curves for
Security" (Langley, Hamburg, Turner, 2016), standardizes it: the curve in
section 4.1, the X25519 function with its decoding, clamping and ladder in
section 5, the test vectors in section 5.2, and the Diffie-Hellman protocol in
section 6.1. It is the key agreement of most TLS 1.3 handshakes (alone or
inside the hybrid group X25519MLKEM768), of SSH (`curve25519-sha256`), of
the Signal protocol, of WireGuard, and of the DHKEM inside HPKE (RFC 9180).
It is a current standard, with no known weakness below its generic security
level of about 2^126 operations.

## Analysis

### Structure

X25519 takes a 32-byte scalar k and a 32-byte u-coordinate and returns the
u-coordinate of the k-th multiple of the point, computed without ever
forming a v-coordinate. Four pieces make up the function, and the Orange file
follows RFC 7748 section 5 piece by piece, with the RFC's own names in
`snake_case`.

The field is GF(p) with p = 2^255 - 19, written as `Int` arithmetic: `prime`
holds p, and `multiply` and `square` reduce every product by `%` at once, so
a field element is always an integer below p between the steps of the
ladder. Sums and differences are left unreduced; the RFC says that all
calculations are performed modulo p, and the `%` of the next product brings
each back below p, so the residue class is the RFC's at every step.

Decoding is `decode_little_endian` (the RFC's `decodeLittleEndian`),
`decode_u_coordinate`, which masks the top bit of the last byte as the RFC
requires, and `decode_scalar_25519`, the clamping: the three low bits and bit
255 cleared, bit 254 set. The clamped scalar stays a byte array, because the
ladder reads it one bit at a time through static indices. Encoding,
`encode_u_coordinate`, peels the 32 base-256 digits of u mod p off with `/`
and `%` and makes them bytes.

The Montgomery ladder is `ladder_step`, the body of the RFC's loop, with the
RFC's intermediate names A, AA, B, BB, E, C, D, DA, CB and its constant
a24 = 121665; `cswap` swaps (x_2 : z_2) with (x_3 : z_3); and `rung` applies
the swap before and after the step whenever the scalar bit k_t is set, which
is what the RFC's running `swap ^= k_t` and the final `cswap` amount to. The
loop in `x25519` runs the 255 rungs from bit 254 down to bit 0 over the state
`[x_2, z_2, x_3, z_3]`, an `Int^4`, and finishes with x_2 * z_2^(p - 2).

The inverse is `invert`. The RFC writes z_2^(p - 2) and leaves its
computation open; the file computes that power by the addition chain of
Bernstein's curve25519 reference implementation, 254 squarings and 11
multiplications, with `square_times` for the runs of squarings. This is the
same power, and it costs about half of what square-and-multiply on the 255
bits of p - 2 costs under the evaluator (see below).

Section 6.1 is `base_point` (u = 9 as 32 bytes), `public_key`
(K_A = X25519(a, 9)), `shared_secret` (K = X25519(a, K_B)), and `all_zero`,
the check on K that the RFC allows both parties to make.

| RFC 7748 section | Orange spec |
| --- | --- |
| 4.1, p and A; 5, a24 | `prime`, `a24` |
| 5, decodeLittleEndian, decodeUCoordinate | `decode_little_endian`, `decode_u_coordinate` |
| 5, decodeScalar25519 | `decode_scalar_25519` |
| 5, encodeUCoordinate | `encode_u_coordinate` |
| 5, the ladder's loop body | `ladder_step` |
| 5, cswap | `cswap`, `rung` |
| 5, z_2^(p - 2) | `invert`, `square_times` |
| 5, X25519(k, u) | `x25519` |
| 6.1, K_A, K_B, K, the all-zero check | `base_point`, `public_key`, `shared_secret`, `all_zero` |

### Security status

Curve25519 was designed in 2006 against a written list of criteria, and the
criteria show in the function. The Montgomery ladder performs the same field
operations for every scalar, so the intended implementations have no branch
and no memory access that depends on a secret; the constant-time conditional
swap of RFC 7748 is written so that the choice is arithmetic, not a branch.
The base point generates a subgroup of prime order l, about 2^252, in a
group of order 8l; the clamping makes every scalar a multiple of 8, which
removes the order-8 component of any input point, and sets bit 254, which
gives every ladder the same length. The quadratic twist of the curve also
has a large prime-order subgroup (cofactor 4), so a u-coordinate that is not
on the curve lands on the twist and gains an attacker nothing: X25519 accepts
every 32-byte string as a public key and needs no point validation. The best
known attack on the discrete logarithm is generic, Pollard's rho, at about
sqrt(pi l / 4), that is about 2^126 curve operations; no algebraic weakness
of the curve is known, and RFC 7748 states the level as about 128 bits.

Two pitfalls are the protocol's rather than the curve's. First, X25519 is
not contributory: the points of small order on the curve and on the twist
(u = 0 and u = 1 among them, with their non-canonical encodings) give the
all-zero output for every scalar, so a party that accepts such a public key ends with
a shared secret its peer chose. RFC 7748 section 6.1 says both parties MAY
check for the all-zero output and abort; Wycheproof's `LowOrderPublic` and
`ZeroSharedSecret` cases exercise this. This entry writes the check as the
`Bool` spec `all_zero` but does not evaluate it on a computed K, because a
second X25519 evaluation does not fit the step budget of a file (see below).
Second, the encoding is not canonical: the RFC masks the top bit of u and
reduces u modulo p, so several 32-byte strings denote one point, and an
implementation that compares public keys as bytes may be surprised;
Wycheproof's `NonCanonicalPublic` cases cover it. Implementation side
channels have been found in deployed code, not in the design: Kaufmann,
Pelletier, Vaudenay and Vuagnoux (2016) showed a compiler turning
constant-time Curve25519 source into a variable-time binary, and Genkin,
Valenta and Yarom (2017) recovered keys from libgcrypt's ladder through a
microarchitectural channel. Like every discrete-logarithm scheme, X25519 is
broken by Shor's algorithm on a large quantum computer; the hybrid group
X25519MLKEM768, which pairs it with ML-KEM, is what the major browsers and
edge networks negotiate by default since 2024.

Status as of September 2026: RFC 7748 (2016) is current. TLS 1.3 (RFC 8446,
2018) makes X25519 a SHOULD-implement group and it is the group most TLS 1.3
handshakes use; RFC 8731 (2020) specifies it for SSH; the Signal protocol,
WireGuard (2017) and HPKE's DHKEM(X25519, HKDF-SHA256) (RFC 9180, 2022) build
on it. NIST SP 800-186 (2023) lists Curve25519 among its recommended
curves; OpenSSL's FIPS provider test data fetched for this entry
(`evppkey_ecx.txt`) still marks X25519 key derivation as unapproved in that
module.

### What the Orange rendering shows

One choice in the function depends on data: `cswap` on the scalar bit k_t.
In Orange it is a conditional, and only the chosen branch is evaluated, so
the rendering says which arrangement the bit selects and nothing about
timing; the RFC's masked swap is an implementation of the same choice, and a
specification does not carry it. Everything else is straight-line field
arithmetic: the `Int` type replaces the radix-2^51 or radix-2^25.5 limb
representations of the fast implementations with exact integers and a `%`
after every product, which is exactly what the RFC's own pseudocode does.
The scalar's bits are read with the static index `(254 - i) / 8` and the
mask `bit[(254 - i) % 8]`, because a data-dependent index does not exist in
the language. The inversion is an addition chain, made of `square_times`
calls whose loop bound is a literal, 100, with the idle iterations skipped by
a comparison.

The evaluation cost was measured with a filler spec sharing the file's
budget (`probe.py` in the scratch directory): one X25519 evaluation, from
the vector's literals to the encoded output, costs about 568,000 of the
1,048,576 steps of a file. One rung of the ladder costs about 1,963 steps,
of which the ten products (65 steps each for two 8-limb numbers) and their
ten reductions by `%` (129 steps each) are more than nine tenths; 255 rungs
are about 500,000 steps. The inversion by the addition chain costs about
59,000 steps, against about 107,000 for square-and-multiply on p - 2 as the
S3f fixture writes it; the digit-peeling encoding costs about 4,400 steps,
against about 12,300 for a `byte_weight` loop. Two evaluations, about
1,136,000 steps, still do not fit one file: a file carrying both section 5.2
vectors fails with ORC0301, so every vector has its own file. The iterated
test of section 5.2 (1,000 and 1,000,000 iterations of X25519) is out of
reach by three and six orders of magnitude.

## Dissemination

### Files

- `x25519.or`: the algorithm, and the first test vector of RFC 7748
  section 5.2.
- `x25519-second-vector.or`: the same algorithm, and the second test vector
  of section 5.2.
- `x25519-diffie-hellman.or`: the same algorithm, and the shared secret K of
  section 6.1 from Alice's private key and Bob's public key, through
  `shared_secret`.
- `x25519-wycheproof.or`: the same algorithm, and test case 1 of Wycheproof's
  `x25519_test.json`.

The four files are one file split by the step budget: their algorithm part,
from `edition 2026;` through `all_zero`, is the same text, generated from one
source by a script and compared byte for byte; only the header's last
sentence and the two vector specs differ.

### Running

    orangec eval algorithms/x25519/x25519.or
    orangec eval algorithms/x25519/x25519-second-vector.or
    orangec eval algorithms/x25519/x25519-diffie-hellman.or
    orangec eval algorithms/x25519/x25519-wycheproof.or
    python3 algorithms/verify.py algorithms/x25519

`eval` also prints the constant specs `prime`, `a24` and `base_point`, which
have no `_expected` twin and are not vectors.

### Vectors

| Spec | Source | Case |
| --- | --- | --- |
| `rfc7748_5_2_vector_1` | RFC 7748, section 5.2, first vector | scalar a546e36b..., u e6db6867..., output c3da5537... |
| `rfc7748_5_2_vector_2` | RFC 7748, section 5.2, second vector | scalar 4b66e9d4..., u e5210f12..., output 95cbde94... |
| `rfc7748_6_1_shared_secret` | RFC 7748, section 6.1 | K = X25519(a, K_B) with Alice's a 77076d0a... and Bob's K_B de9edb7d...; K 4a5d9d5b... |
| `wycheproof_x25519_tc_1` | Wycheproof `testvectors_v1/x25519_test.json`, tcId 1 | "normal case", flags Normal, result valid; private c8a9d5a9..., public 504a3699..., shared 436a2c04... |

Every expected value is the published value: the section 5.2 outputs and the
section 6.1 K as the RFC prints them, taken from the copies named below, and
the Wycheproof `shared` field. The `cryptography` package
(`X25519PrivateKey.from_private_bytes(...).exchange(...)` and
`public_key()`) confirmed every row and the two section 6.1 public keys.

### Provenance and claims

The RFC itself is unreachable from the build machine, so its values came
from files that copy it verbatim: RFC 7748 section 5.2 from
`pyca/cryptography`'s `vectors/.../X25519/rfc7748.txt` and BoringSSL's
`crypto/curve25519/x25519_test.cc`, which agree; section 6.1 from OpenSSL's
`test/recipes/30-test_evp_data/evppkey_ecx.txt` (the raw keys and the shared
secret) and the same BoringSSL file, which agree; and the Wycheproof case from
`C2SP/wycheproof`'s `testvectors_v1/x25519_test.json`, the only case in that
file whose flags are exactly `Normal`. The prime is 2^255 - 19 computed by
Python and compared with the literal; a24 = (486662 - 2) / 4 likewise. The
addition chain of `invert` was checked in Python against `pow(z, p - 2, p)`
on fifty pseudo-random field elements before being transcribed. A Python
reference of RFC 7748 section 5 (`reference.py` in the scratch directory)
reproduces every vector and agrees with `cryptography` on each; the Orange
byte literals were rendered from the fetched files by `literal.py`, and the
four `.or` files were assembled by `build.py`, never typed by hand.

This entry is a reference evaluation of RFC 7748 under `orangec eval`. It
makes no constant-time, side-channel, performance or certification claim; in
particular the conditional in `cswap` is a specification of a choice, not a
constant-time swap. It is not a corpus entry in the sense of The Orange Book
chapter 12.

## Gaps

- The step budget of 1,048,576 steps per file holds one X25519 evaluation
  (about 568,000 steps) and not two, so the four vectors are four files with
  an identical algorithm part instead of one file, the iterated test of
  section 5.2 is out of reach, and the all-zero check of section 6.1 is
  written (`all_zero`) but not evaluated on a computed shared secret.
- Loop bounds must be literals, so the runs of squarings in the addition
  chain cannot take their length as the bound; `square_times` always runs
  100 iterations and skips the surplus with a comparison, about 4,500 idle
  steps per inversion, the alternative being nine one-purpose specs.
- There are no tuples and no arrays of mixed type, so the ladder's four
  coordinates travel as an `Int^4`, and the encoding carries the remaining
  value and the 32 digits together as an `Int^33` and needs a second loop to
  make bytes of them.
- `^` is not defined on `Int` or `Bool`, so the RFC's running `swap ^= k_t`
  is not written as such; `rung` swaps around the step when k_t is set,
  which is the same sequence of arrangements.
