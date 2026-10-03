# Orange

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="assets/identity/orange-readme-banner-dark.svg">
  <source media="(prefers-color-scheme: light)" srcset="assets/identity/orange-readme-banner-light.svg">
  <img src="assets/identity/orange-readme-banner.svg" width="830" alt="Orange: cryptography you can check.">
</picture>

**Orange is a language and toolchain for cryptography you can check.** You
write the mathematical specification, connect it to a fast implementation, say
exactly which properties you claim, and ship the native code together with the
evidence for each claim.

Orange is made for cryptographers, cryptologists, and cryptanalysts: people who
read mathematics for a living. Its aim is to be exact and beautiful at once, so
that Orange source reads like the definition in a standard or a paper, while
every step from that definition to machine code stays precise enough to check.

> [!IMPORTANT]
> Orange is **pre-alpha** and built by one person. Today the compiler checks
> and evaluates a small typed fragment of the language. It does not yet
> generate native code or check proofs, and nothing in
> this repository has been independently reviewed or formally verified.

## Why Orange exists

A serious cryptographic library carries several meanings at once: the
mathematics it is meant to compute, the code that runs on real machines, the
security properties it claims, and the evidence behind those claims. Today
those meanings live in different tools: a notation for the specification, C or
Rust or assembly for speed, a proof assistant for correctness, a separate
analyzer for constant-time behavior, and test vectors and build logs around the
outside. Each tool can be excellent. The trouble is at the crossings, where a
proof about one definition gets attached to a different binary, or a
source-level guarantee quietly fails to survive the compiler.

Orange aims to make those crossings part of the product:

- **One language, several semantic worlds.** Mathematical specifications,
  executable implementations, leakage-aware machine code, security games, and
  proofs live in one module system, each with semantics suited to its job.
- **Claims, not labels.** Instead of a single "verified" badge, every artifact
  carries narrowly worded claims (conformance, functional refinement, memory
  safety, leakage, compiler preservation, ABI agreement, and more), each with
  its own subject, assumptions, evidence, and outcome.
- **Evidence you can replay.** Proofs, certificates, and build records are
  machine-readable and content-addressed, so a release can be rechecked
  offline.
- **A small, published trusted base per claim.** Each claim names exactly which
  components it trusts, instead of inheriting one project-wide trust list.
- **Real native output.** The end goal is production native code with a stable
  C ABI, deterministic builds, and signed release provenance.

These are design directions, not current features. The
[Orange Book](docs/THE_ORANGE_BOOK.md) explains them in depth.

## A first look

This is Orange 2026 source that the current compiler accepts: three of the
SHA-256 functions of FIPS 180-4, written the way the standard writes them.

```orange
edition 2026;
module sha256 {
  spec choose(x: Word[32], y: Word[32], z: Word[32]) -> Word[32] {
    (x & y) ^ (~x & z)
  }
  spec majority(x: Word[32], y: Word[32], z: Word[32]) -> Word[32] {
    (x & y) ^ (x & z) ^ (y & z)
  }
  spec big_sigma0(x: Word[32]) -> Word[32] {
    (x >>> 2) ^ (x >>> 13) ^ (x >>> 22)
  }

  // Values from round 0 of the FIPS 180-4 "abc" example.
  spec sigma0_of_h0() -> Word[32] { big_sigma0(0x6a09_e667) }
  spec majority_of_h() -> Word[32] { majority(0x6a09_e667, 0xbb67_ae85, 0x3c6e_f372) }
}
```

`Word[32]` is the ring of integers modulo 2^32, so `+`, `-`, and `*` on words
are the ring operations: wrapping is the meaning, never an accident. `>>>` and
`<<<` rotate, `>>` and `<<` shift, and an amount written as a literal is
checked against the width. `Int` is the type of mathematical integers, with no
overflow. A literal must fit its type exactly, so `256` is an error as a
`Word[8]`, not a silent zero. Saved as `sha256.or`, the module checks and
evaluates:

```console
$ orangec eval sha256.or
sha256::sigma0_of_h0: Word[32] = 0xce20b47e
sha256::majority_of_h: Word[32] = 0x3a6fe667
```

The full [SHA-256 fixture](compiler/fixtures/s3b/valid-sha256-functions.or)
carries these functions through round 0 and reproduces NIST's published value
of `a`, `0x5d6aebcd`; the
[ChaCha20 fixture](compiler/fixtures/s3b/valid-chacha20-quarter-round.or)
reproduces the quarter-round test vector of RFC 8439.

Orange's whole precedence table fits in one line: prefix operators first, then
`*` before `+` and `-`. Operators from different families never share a level
without parentheses, so every expression reads exactly as it groups. For a
file `mix.or` whose function body is `a + b ^ b <<< 7`:

```console
$ orangec check mix.or
error[ORC0108]: `^` follows `+` without grouping parentheses
 --> mix.or:4:11
  |
4 |     a + b ^ b <<< 7
  |           ^ ungrouped operator
  = note: operators from different groups have no relative precedence in Orange; parenthesize the part that applies first
```

### Named steps and explicit conversions

A standard names its intermediate values, and so can Orange. A body may begin
with `let` bindings, each with a stated type, and a value changes type only
through a written `as`. The ChaCha20 quarter round of RFC 8439 then reads the
way the RFC prints it, and bytes become a word in the order the standard
names:

```orange
edition 2026;
module chacha20 {
  spec quarter_a(a: Word[32], b: Word[32], c: Word[32], d: Word[32]) -> Word[32] {
    let a1: Word[32] = a + b;
    let d1: Word[32] = (d ^ a1) <<< 16;
    let c1: Word[32] = c + d1;
    let b1: Word[32] = (b ^ c1) <<< 12;
    a1 + b1
  }
  spec load_le32(b0: Word[8], b1: Word[8], b2: Word[8], b3: Word[8]) -> Word[32] {
    (b0 as Word[32]) | ((b1 as Word[32]) << 8) | ((b2 as Word[32]) << 16)
      | ((b3 as Word[32]) << 24)
  }

  // RFC 8439: the quarter-round test vector of section 2.1.1 and the first
  // key word of section 2.3.2.
  spec a() -> Word[32] { quarter_a(0x11111111, 0x01020304, 0x9b8d6f43, 0x01234567) }
  spec key_word0() -> Word[32] { load_le32(0x00, 0x01, 0x02, 0x03) }
}
```

```console
$ orangec eval chacha20.or
chacha20::a: Word[32] = 0xea2a92f4
chacha20::key_word0: Word[32] = 0x03020100
```

A binding never shadows another name, and `as` converts exactly one operand.
`x + y as Word[32]` is an error, because for bytes `x` and `y` the two
readings, `(x + y) as Word[32]` and `(x as Word[32]) + (y as Word[32])`, are
different values. Since [S3n](#words-in-either-byte-order), the four bytes
of `load_le32` are one conversion, `b as little Word[32]` for `b: Word[8]^4`.
This slice, S3c, is implemented and tested; its specification is in review as
[OEP-0006](docs/governance/oeps/OEP-0006-orange-2026-bindings-and-conversions.md).

### A whole state as one value

A cipher works on a state, and in Orange a state is one value. `Word[32]^8` is
eight 32-bit words, an element of (Z/2^32 Z)^8. An array literal lists every
element, and `s[4]` reads one at a literal index that the compiler checks
against the length. One SHA-256 round of FIPS 180-4 section 6.2.2 is then one
function from state to state:

```orange
spec round(s: Word[32]^8, k: Word[32], w: Word[32]) -> Word[32]^8 {
  let t1: Word[32] = s[7] + big_sigma1(s[4]) + choose(s[4], s[5], s[6]) + k + w;
  let t2: Word[32] = big_sigma0(s[0]) + majority(s[0], s[1], s[2]);
  [t1 + t2, s[0], s[1], s[2], s[3] + t1, s[4], s[5], s[6]]
}
```

Applied to the initial hash value and the first word of the padded "abc"
block, the [SHA-256 fixture](compiler/fixtures/s3d/valid-sha256-rounds.or)
gives exactly the working variables NIST publishes for round 0:

```text
sha256::after_round0: Word[32]^8 = [0x5d6aebcd, 0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xfa2a4622, 0x510e527f, 0x9b05688c, 0x1f83d9ab]
```

The [ChaCha20 fixture](compiler/fixtures/s3d/valid-chacha20-block.or) holds
the whole block function of RFC 8439, ten double rounds over a `Word[32]^16`
state, and reproduces the serialized block of section 2.3.2 word for word.
Arrays have no operators of their own, so every operator still acts on one
ring element, and every index is a literal, so every position a specification
reads is visible and in range. This slice, S3d, is implemented and tested; its
specification is in review as
[OEP-0007](docs/governance/oeps/OEP-0007-orange-2026-fixed-length-arrays.md).

### Loops the way standards write them

FIPS 180-4 prepares the SHA-256 message schedule "for t = 16 to 63", and Orange
writes exactly that. A loop runs over a range given by two literals, carries
one accumulator of a stated type, and has that accumulator's value after its
last step: a fold, with its count in plain sight. `w with [t] = v` is the array
`w` with element `t` replaced, and `[0; 64]` is sixty-four zeros.

```orange
spec schedule(m: Word[32]^16) -> Word[32]^64 {
  let head: Word[32]^64 = for t in 0..16 with w: Word[32]^64 = [0; 64] { w with [t] = m[t] };
  for t in 16..64 with w: Word[32]^64 = head {
    w with [t] = small_sigma1(w[t - 2]) + w[t - 7] + small_sigma0(w[t - 15]) + w[t - 16]
  }
}

spec compress(h: Word[32]^8, m: Word[32]^16) -> Word[32]^8 {
  let w: Word[32]^64 = schedule(m);
  let k: Word[32]^64 = round_constants();
  let v: Word[32]^8 = for t in 0..64 with v: Word[32]^8 = h { round(v, k[t], w[t]) };
  for i in 0..8 with out: Word[32]^8 = v { out with [i] = out[i] + h[i] }
}
```

The [SHA-256 fixture](compiler/fixtures/s3e/valid-sha256.or) hashes "abc" to
the digest FIPS 180-4 publishes, and the two-block NIST example too:

```text
sha256::abc_digest: Word[32]^8 = [0xba7816bf, 0x8f01cfea, 0x414140de, 0x5dae2223, 0xb00361a3, 0x96177a9c, 0xb410ff61, 0xf20015ad]
```

An index such as `w[t - 15]` may use only literals and loop indices, and the
compiler proves, before anything runs, that it stays in range for every `t`
from 16 to 63; `w[t - 17]` is rejected with the range it would take, -1
through 46. In this slice an index may use only literals and loop indices;
[S3g](#tables-keyed-by-data) lets it depend on data, still proved in range. The
[ChaCha20 fixture](compiler/fixtures/s3e/valid-chacha20.or) loads the key and
nonce with loops, runs the ten double rounds as one loop, and encrypts the
"sunscreen" plaintext of RFC 8439 section 2.4.2 to the RFC's ciphertext, byte
for byte. This slice, S3e, is implemented and tested; its specification is in
review as
[OEP-0008](docs/governance/oeps/OEP-0008-orange-2026-bounded-loops.md).

### Prime fields and choices

RFC 7748 defines X25519 in the integers modulo 2^255 − 19: reduce after every
product, read one bit of the scalar per rung of the Montgomery ladder, and
swap two points when the bit is set. Orange writes each step the way the RFC
does. `%` is Euclidean, so `a % p` is always the canonical residue from 0
through p − 1; a comparison gives a `Bool`; and `if c { a } else { b }` chooses
one of two values of the same type and evaluates only the one it chooses.

```orange
spec rung(x1: Int, s: Int^4, set: Bool) -> Int^4 {
  if set { swap(ladder(x1, swap(s))) } else { ladder(x1, s) }
}

spec x25519(scalar: Word[8]^32, u: Word[8]^32) -> Word[8]^32 {
  let k: Word[8]^32 = clamp(scalar);
  let masks: Word[8]^8 = [1, 2, 4, 8, 16, 32, 64, 128];
  let x1: Int = decode_u(u);
  let s: Int^4 = for i in 0..255 with s: Int^4 = [1, 0, x1, 1] {
    rung(x1, s, (k[(254 - i) / 8] & masks[(254 - i) % 8]) != 0)
  };
  encode((s[0] * power(s[1], prime() - 2)) % prime())
}
```

The [X25519 fixture](compiler/fixtures/s3f/valid-x25519.or) computes the first
test vector of RFC 7748 section 5.2, byte for byte:

```text
x25519::test_vector: Word[8]^32 = [0xc3, 0xda, 0x55, 0x37, 0x9d, 0xe9, 0xc6, 0x90, 0x8e, 0x94, 0xea, 0x4d, 0xf2, 0x8d, 0x08, 0x4f, 0x32, 0xec, 0xcf, 0x03, 0x49, 0x1c, 0x71, 0xf7, 0x54, 0xb4, 0x07, 0x55, 0x77, 0xa2, 0x85, 0x52]
```

The index `k[(254 - i) / 8]` divides a loop index, and the compiler still
proves it in range, 0 through 31, before anything runs. The
[Poly1305 fixture](compiler/fixtures/s3f/valid-poly1305.or) reproduces the tag
of RFC 8439 section 2.5.2, and the
[AEAD fixture](compiler/fixtures/s3f/valid-aead.or) seals the section 2.8.2
"sunscreen" message with ChaCha20-Poly1305 to the RFC's ciphertext and tag.
Division by zero is defined (`x / 0` is 0 and `x % 0` is x), so nothing fails
at run time, and `Bool` is not a number: it converts to nothing and has only
`!`, `&&`, `||`, `==`, and `!=`. A conditional is a choice between values, not
a claim about how a machine branches; RFC 7748 asks for a constant-time swap,
and Orange makes no timing claim until it generates code.
[S3i](#fields-as-types) puts the field in the type, so the ladder writes no
reduction at all. This slice, S3f, is implemented and tested; its specification is in review as
[OEP-0009](docs/governance/oeps/OEP-0009-orange-2026-conditions.md).

### Tables keyed by data

FIPS 197 defines AES's SubBytes as a table: each byte of the state selects one
of the 256 entries of the S-box. An Orange index may depend on data, and the
compiler still proves it in range before anything runs. A byte runs from 0
through 255, so it may index any table of 256 entries, and `x & 15` or
`x >> 4` may index a table of 16:

```orange
spec sub_bytes(s: Word[8]^256, a: Word[8]^16) -> Word[8]^16 {
  for i in 0..16 with b: Word[8]^16 = a { b with [i] = s[a[i]] }
}

spec sub_word(s: Word[8]^256, w: Word[32]) -> Word[32] {
  ((s[w >> 24] as Word[32]) << 24)
    | ((s[(w >> 16) & 0xff] as Word[32]) << 16)
    | ((s[(w >> 8) & 0xff] as Word[32]) << 8)
    | (s[w & 0xff] as Word[32])
}
```

The [AES-128 fixture](compiler/fixtures/s3g/valid-aes128.or) does not copy the
S-box; it derives it as FIPS 197 section 5.1.1 defines it, from inverses in
GF(2^8) read off a table of logarithms, which is itself built by updates keyed
by the table's own values. It then encrypts the examples of Appendix B and
Appendix C.1 to the published ciphertexts and decrypts C.1 back to its
plaintext:

```text
aes::example_c1: Word[8]^16 = [0x69, 0xc4, 0xe0, 0xd8, 0x6a, 0x7b, 0x04, 0x30, 0xd8, 0xcd, 0xb7, 0x80, 0x70, 0xb4, 0xc5, 0x5a]
```

Each operator narrows a range by one rule a reader can apply: `x & 15` runs
from 0 through 15, `(x & 15) + 16` from 16 through 31, and `(x & 15) - 1`
over its whole type, because it wraps. `s[x]` for a byte `x` and a table of 255
entries is rejected with the range it would take, 0 through 255. A lookup
keyed by a secret is the classic cache-timing leak of software AES; Orange
states the lookup the standard states, makes no timing claim about it, and
leaves how such a lookup is compiled to a later code-generation decision.
Updates also cost less: changing one entry of a 256-entry table costs 4
evaluation steps, not 256. This slice, S3g, is implemented and tested; its
specification is in review as
[OEP-0010](docs/governance/oeps/OEP-0010-orange-2026-lookups.md).

### Standards built on standards

Cryptography is specified in layers: HMAC is defined over a hash function, and
HKDF over HMAC. An Orange module names the modules it uses at its head and
calls their functions by module name, so each standard is written once, in its
own file, and read against its own text. This is HMAC as RFC 2104 defines it,
over the SHA-256 of another file:

```orange
module hmac {
  use sha256;

  spec keyed(key: Word[8]^64, pad: Word[8]) -> Word[8]^64 {
    for i in 0..64 with b: Word[8]^64 = key { b with [i] = key[i] ^ pad }
  }

  spec block(d: Word[8]^32) -> Word[8]^64 {
    for i in 0..32 with b: Word[8]^64 = [0; 64] { b with [i] = d[i] }
  }

  spec mac(key: Word[8]^64, m: Word[8]^64, length: Int) -> Word[8]^32 {
    let inner: Word[32]^8 = sha256::compress(sha256::initial(), keyed(key, 0x36));
    let outer: Word[32]^8 = sha256::compress(sha256::initial(), keyed(key, 0x5c));
    let text: Word[8]^32 =
      sha256::digest(sha256::compress(inner, sha256::last_block(m, length, 64 + length)));
    sha256::digest(sha256::compress(outer, sha256::last_block(block(text), 32, 96)))
  }
}
```

`orangec eval` reads `sha256.or` for `use sha256;` from beside the file that
names it, checks every module once, after the modules it uses, and prints only
the values of the program it was given. The
[module fixtures](compiler/fixtures/s3h/) write SHA-256, HMAC, and HKDF as
three modules and reproduce the SHA-256 example of FIPS 180-4, test cases 1
and 2 of RFC 4231, and test case 1 of RFC 5869:

```text
vectors::okm: Word[8]^42 = [0x3c, 0xb2, 0x5f, 0x25, 0xfa, 0xac, 0xd5, 0x7a, 0x90, 0x43, 0x4f, 0x64, 0xd0, 0x36, 0x2f, 0x2a, 0x2d, 0x2d, 0x0a, 0x90, 0xcf, 0x1a, 0x5a, 0x4c, 0x5d, 0xb0, 0x2d, 0x56, 0xec, 0xc4, 0xc5, 0xbf, 0x34, 0x00, 0x72, 0x08, 0xd5, 0xb8, 0x87, 0x18, 0x58, 0x65]
```

Nothing is imported into scope: a call into another module always names it,
and a module declares every module it uses, so a reader sees where each
function comes from. Modules may not use each other in a cycle, and each means
the same whoever uses it. [S3l](#bytes-as-the-standards-print-them) writes
the test cases' keys and messages as the RFC prints them, and
[S3m](#one-algorithm-for-every-length) writes HMAC once for every key and
message length. This slice, S3h, is
implemented and tested; its specification is in review as
[OEP-0011](docs/governance/oeps/OEP-0011-orange-2026-modules.md).

### Fields as types

RFC 7748 writes `AA = A^2` and means the square in the field of 2^255 − 19
elements. In Orange that field is a type. `Mod[m]` holds the integers modulo
m, its `+`, `-`, and `*` reduce by themselves, and the modulus is a constant
written the way the standard writes it; `type` names it once for the rest of
the module. The rung of the Montgomery ladder is then the RFC's formulas, line
for line, with no reduction in sight:

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

Division multiplies by the inverse, and gives 0 when there is none, which is
exactly what the RFC's `x_2 * (z_2^(p - 2))` computes, so the ladder ends in
`s[0] / s[1]`. The [X25519 fixture](compiler/fixtures/s3i/valid-x25519.or)
reproduces the RFC's test vector with no `%` anywhere, the
[Poly1305 fixture](compiler/fixtures/s3i/valid-poly1305.or) keeps its
accumulator in `Mod[(1 << 130) - 5]`, and the
[field fixture](compiler/fixtures/s3i/valid-fields.or) computes constants in
the rings their standards define, from ML-KEM's modulo 3329 to the field of
P-256:

```text
fields::d: Mod[(1 << 255) - 19] = 37095705934669439343138083508754565189542113879843219016388785533085940283555
fields::p256_generator_on_curve: Bool = true
```

Two moduli are two types, so a residue modulo 7 never meets one modulo 11
without an `as`, and `as` also turns a residue into its least residue as an
`Int` or a word. Residues have no order, no remainder, and no bits: a standard
that compares field elements compares least residues, and
`(x as Int) < (y as Int)` says so. A modulus may be as wide as 2^521 − 1, the
prime of P-521. [S3j](#rounds-in-the-words-of-their-standard) puts the whole
ladder in one loop, with the RFC's names inside it, and
[S3o](#one-function-for-several-types) writes one exponentiation and one
inverse for every field a module names. This slice, S3i, is
implemented and tested; its specification is in review as
[OEP-0012](docs/governance/oeps/OEP-0012-orange-2026-modular-arithmetic.md).

### Rounds in the words of their standard

FIPS 180-4 writes a round of SHA-256 as a short list of named values: the
working variables a through h, and the temporary words T1 and T2. A loop's
step and each branch of a conditional may begin with `let` bindings, exactly
as a function's body does, so the round stands inside the loop that runs it
and reads as the standard reads:

```orange
spec compress(hash: Word[32]^8, m: Word[32]^16) -> Word[32]^8 {
  let w: Word[32]^64 = schedule(m);
  let k: Word[32]^64 = round_constants();
  let v: Word[32]^8 = for t in 0..64 with v: Word[32]^8 = hash {
    let a: Word[32] = v[0];
    let b: Word[32] = v[1];
    let c: Word[32] = v[2];
    let d: Word[32] = v[3];
    let e: Word[32] = v[4];
    let f: Word[32] = v[5];
    let g: Word[32] = v[6];
    let h: Word[32] = v[7];
    let t1: Word[32] = h + big_sigma1(e) + ch(e, f, g) + k[t] + w[t];
    let t2: Word[32] = big_sigma0(a) + maj(a, b, c);
    [t1 + t2, a, b, c, d + t1, e, f, g]
  };
  for i in 0..8 with out: Word[32]^8 = v { out with [i] = out[i] + hash[i] }
}
```

The [SHA-256 fixture](compiler/fixtures/s3j/valid-sha256.or) hashes the
examples of FIPS 180-4 to their published digests:

```text
sha256::abc_digest: Word[32]^8 = [0xba7816bf, 0x8f01cfea, 0x414140de, 0x5dae2223, 0xb00361a3, 0x96177a9c, 0xb410ff61, 0xf20015ad]
```

The [X25519 fixture](compiler/fixtures/s3j/valid-x25519.or) writes the
Montgomery ladder of RFC 7748 as one loop whose step names k_t, swap, A, AA,
B, BB, E, C, D, DA, and CB, as the RFC does. A step's bindings are evaluated
afresh at every step, and a branch's only when the branch is chosen. Each is
in scope for the bindings after it and for its block's value, and nowhere
else, and none may reuse a name already in scope: Orange has no shadowing, so
a name means one thing wherever a reader meets it.
[S3k](#several-values-at-once) lets the loop carry a through h themselves.
This slice, S3j, is implemented and tested; its specification is in review as
[OEP-0013](docs/governance/oeps/OEP-0013-orange-2026-blocks.md).

### Several values at once

A round keeps several values at once, and its standard names each of them. A
**tuple** holds a fixed number of values of possibly different types: its type
is written `(Word[32], Word[32])`, the tuple itself `(a, b)`, and `.k` selects
element k, counted from zero. A **tuple pattern** names every element where a
binding or a loop's accumulator is declared, so a function can give several
values and a loop can carry several accumulators. SHA-256's compression
function carries the working variables a through h from round to round by
name, as FIPS 180-4 section 6.2.2 does:

```orange
spec compress(hash: Word[32]^8, m: Word[32]^16) -> Word[32]^8 {
  let w: Word[32]^64 = schedule(m);
  let k: Word[32]^64 = round_constants();
  let (a: Word[32], b: Word[32], c: Word[32], d: Word[32],
       e: Word[32], f: Word[32], g: Word[32], h: Word[32]) =
    for t in 0..64 with (a: Word[32], b: Word[32], c: Word[32], d: Word[32],
                         e: Word[32], f: Word[32], g: Word[32], h: Word[32]) =
      (hash[0], hash[1], hash[2], hash[3], hash[4], hash[5], hash[6], hash[7]) {
      let t1: Word[32] = h + big_sigma1(e) + ch(e, f, g) + k[t] + w[t];
      let t2: Word[32] = big_sigma0(a) + maj(a, b, c);
      (t1 + t2, a, b, c, d + t1, e, f, g)
    };
  [
    a + hash[0], b + hash[1], c + hash[2], d + hash[3],
    e + hash[4], f + hash[5], g + hash[6], h + hash[7],
  ]
}
```

The quarter round of ChaCha20 takes four words and gives four, as RFC 8439
section 2.1 writes it:

```orange
type Quad = (Word[32], Word[32], Word[32], Word[32]);

spec quarter_round(a: Word[32], b: Word[32], c: Word[32], d: Word[32]) -> Quad {
  let a1: Word[32] = a + b;
  let d1: Word[32] = (d ^ a1) <<< 16;
  let c1: Word[32] = c + d1;
  let b1: Word[32] = (b ^ c1) <<< 12;
  let a2: Word[32] = a1 + b1;
  let d2: Word[32] = (d1 ^ a2) <<< 8;
  let c2: Word[32] = c1 + d2;
  let b2: Word[32] = (b1 ^ c2) <<< 7;
  (a2, b2, c2, d2)
}
```

The [SHA-256](compiler/fixtures/s3k/valid-sha256.or) and
[ChaCha20](compiler/fixtures/s3k/valid-chacha20.or) fixtures reproduce FIPS
180-4's digests and RFC 8439's vectors, and the
[Ascon-Hash256 fixture](compiler/fixtures/s3k/valid-ascon.or) of NIST SP
800-232 carries its state as five named words, x0 through x4 as the Ascon
designers name them, and reproduces the designers' known answers:

```text
chacha20::quarter_round_vector: (Word[32], Word[32], Word[32], Word[32]) = (0xea2a92f4, 0xcb1cf8ce, 0x4581472e, 0x5881c4bb)
```

A tuple's elements are scalars and arrays, never tuples, and no arithmetic
applies to a whole tuple: `p + q` is an error, and `p.0 + q.0` says which
elements are added. Since S3q, `p == q` compares two tuples whole. A pattern's names follow the rules of every other name,
with no shadowing. This slice, S3k, is implemented and tested; its
specification is in review as
[OEP-0014](docs/governance/oeps/OEP-0014-orange-2026-tuples.md).

### Bytes as the standards print them

A standard prints its test inputs as text and hex, and its algorithms move
runs of bytes. A **byte string** `"Hi There"` is the array `Word[8]^8` of the
ASCII bytes of its text, with the escapes `\"`, `\\`, `\n`, `\r`, `\t`,
`\0`, and `\xNN`; a **hex string** `hex"0c00000000000000"` is the array of
its hex digit pairs, spaced wherever the reader likes between bytes. `a ++ b`
joins two arrays, `x[a..b]` is the run of elements of `x` from index a up to,
but not including, index b, and `x with [a..b] = v` replaces that run. SHA-256
pads a message as FIPS 180-4 section 5.1.1 says, and reads each block's words
as four-byte slices:

```orange
spec schedule(block: Word[8]^64) -> Word[32]^64 {
  let head: Word[32]^64 = for t in 0..16 with w: Word[32]^64 = [0; 64] {
    w with [t] = word(block[4 * t..4 * t + 4])
  };
  for t in 16..64 with w: Word[32]^64 = head {
    w with [t] = small_sigma1(w[t - 2]) + w[t - 7] + small_sigma0(w[t - 15]) + w[t - 16]
  }
}

spec abc() -> Word[8]^32 { hash64("abc" ++ hex"80" ++ [0; 52] ++ hex"00000000 00000018") }
```

```text
hmac::abc: Word[8]^32 = [0xba, 0x78, 0x16, 0xbf, 0x8f, 0x01, 0xcf, 0xea, 0x41, 0x41, 0x40, 0xde, 0x5d, 0xae, 0x22, 0x23, 0xb0, 0x03, 0x61, 0xa3, 0x96, 0x17, 0x7a, 0x9c, 0xb4, 0x10, 0xff, 0x61, 0xf2, 0x00, 0x15, 0xad]
```

ChaCha20-Poly1305 reads as RFC 8439 section 2.8 writes it: the one-time key is
the first 32 bytes of block 0, and Poly1305 reads the additional data, the
ciphertext, their padding, and both lengths joined into one message:

```orange
spec mac_data(aad: Word[8]^12, ciphertext: Word[8]^114) -> Word[8]^160 {
  aad ++ [0; 4] ++ ciphertext ++ [0; 14] ++ hex"0c00000000000000" ++ hex"7200000000000000"
}

spec seal(key: Word[8]^32, nonce: Word[8]^12, aad: Word[8]^12, plaintext: Word[8]^114)
  -> Word[8]^130 {
  let ciphertext: Word[8]^114 = encrypt(key, nonce, plaintext);
  ciphertext ++ mac(block(key, 0, nonce)[..32], mac_data(aad, ciphertext))
}
```

The [HMAC fixture](compiler/fixtures/s3l/valid-hmac.or) reproduces FIPS
180-4's digest of "abc" and RFC 4231's test cases 1 and 2, keyed by twenty
bytes 0b and by "Jefe", and the
[AEAD fixture](compiler/fixtures/s3l/valid-aead.or) seals RFC 8439's sunscreen
sentence, written as text, into the RFC's ciphertext and tag, and verifies the
tag. A slice's bounds are integer literals and loop indices, so its length is
the same at every step and every element it takes is proved to exist before
the program runs; `x[n..n + 4]` with `n` a parameter is an error. A byte
string holds printable ASCII: `"é"` is an error whose label writes its UTF-8
bytes as `hex"c3 a9"`. This slice, S3l, is implemented and tested; its
specification is in review as
[OEP-0015](docs/governance/oeps/OEP-0015-orange-2026-bytes.md).

### One algorithm for every length

A standard gives one algorithm for inputs of many lengths, and Orange writes
it once. A `spec` may declare **size parameters** in square brackets, each with
a finite range, and use them wherever a length or a loop bound is written.
SHA-256 pads a message of any length from 1 through 119 bytes to whole blocks
and absorbs them one at a time:

```orange
// FIPS 180-4 section 5.1.1: the message, the byte 80, zeros, and the
// message's length in bits fill ((len + 8) / 64) + 1 blocks.
spec pad[len in 1..120](m: Word[8]^len) -> Word[8]^(64 * (((len + 8) / 64) + 1)) {
  m ++ hex"80" ++ [0; ((64 * (((len + 8) / 64) + 1)) - len - 3)]
    ++ [((8 * len) / 256) as Word[8], (8 * len) as Word[8]]
}

// Section 6.2.2: each block absorbed in turn.
spec absorb[blocks in 1..4](p: Word[8]^(64 * blocks)) -> Word[8]^32 {
  digest(for b in 0..blocks with h: Word[32]^8 = initial_hash() {
    compress(h, p[64 * b..64 * b + 64])
  })
}

spec sha256[len in 1..120](m: Word[8]^len) -> Word[8]^32 { absorb(pad(m)) }

spec abc() -> Word[8]^32 { sha256("abc") }

spec two_blocks() -> Word[8]^32 {
  sha256("abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq")
}
```

```text
sha256::abc: Word[8]^32 = [0xba, 0x78, 0x16, 0xbf, 0x8f, 0x01, 0xcf, 0xea, 0x41, 0x41, 0x40, 0xde, 0x5d, 0xae, 0x22, 0x23, 0xb0, 0x03, 0x61, 0xa3, 0x96, 0x17, 0x7a, 0x9c, 0xb4, 0x10, 0xff, 0x61, 0xf2, 0x00, 0x15, 0xad]
sha256::two_blocks: Word[8]^32 = [0x24, 0x8d, 0x6a, 0x61, 0xd2, 0x06, 0x38, 0xb8, 0xe5, 0xc0, 0x26, 0x93, 0x0c, 0x3e, 0x60, 0x39, 0xa3, 0x3c, 0xe4, 0x59, 0x64, 0xff, 0x21, 0x67, 0xf6, 0xec, 0xed, 0xd4, 0x19, 0xdb, 0x06, 0xc1]
```

`pad` stands for 119 functions, `pad[1]` through `pad[119]`, one for each
value of `len`, and the compiler checks every one before anything runs,
exactly as it would check each written out by hand: every length is computed,
every slice of `absorb` is proved in range for each number of blocks, and
were one instance in error, the first would be reported by its name, such as
`pad[56]`. Nothing is symbolic. A size is a number, different in each instance, and what
is proved of `sha256` is proved of each of its 119 instances.

A call names its instance by its sizes, as `pad[3](m)`, or by the lengths of
its arguments: `sha256("abc")` calls `sha256[3]`, and `absorb(pad(m))` calls
the instance of `absorb` that takes `pad`'s result. HMAC is then written once
for every key of 1 through 63 bytes and every message of 1 through 55, over
that SHA-256 in its own module:

```orange
spec padded[klen in 1..64](key: Word[8]^klen) -> Word[8]^64 { key ++ [0; (64 - klen)] }

spec hmac[len in 1..56](k0: Word[8]^64, m: Word[8]^len) -> Word[8]^32 {
  sha256::sha256(keyed(k0, 0x5c) ++ sha256::sha256(keyed(k0, 0x36) ++ m))
}

spec case2() -> Word[8]^32 { hmac(padded("Jefe"), "what do ya want for nothing?") }
```

The [SHA-256 fixture](compiler/fixtures/s3m/sha256.or) reproduces FIPS
180-4's digests of "abc" and of its 56-byte message, whose padding takes a
second block; the [HMAC fixture](compiler/fixtures/s3m/valid-hmac.or)
reproduces RFC 4231's test cases 1 and 2; and the
[Poly1305 fixture](compiler/fixtures/s3m/valid-poly1305.or), written once for
every message of 1 through 255 bytes, reproduces the tag of RFC 8439 section
2.5.2. A size is built from integer literals and size parameters, so it never
depends on data, and a function has at most 256 instances. This slice, S3m,
is implemented and tested; its specification is in review as
[OEP-0016](docs/governance/oeps/OEP-0016-orange-2026-sizes.md).

### Words in either byte order

A standard prints bytes and computes on words, and it says in a few words how
one becomes the other: SHA-256 reads each 64-byte block as sixteen big-endian
32-bit words, and ChaCha20 reads its key and nonce as little-endian ones.
Orange says it the same way. A conversion may name a **byte order**, `as big`
or `as little`, and then reads a word or an array of words as the words of
another width, as an `Int`, or as a residue, and writes a number as words,
the first word most significant with `big` and least significant with
`little`:

```orange
edition 2026;
module order {
  spec big_word() -> Word[32] { hex"01020304" as big Word[32] }
  spec little_word() -> Word[32] { hex"01020304" as little Word[32] }
  spec text() -> Word[32] { "abcd" as big Word[32] }
  spec bytes() -> Word[8]^4 { let w: Word[32] = 0xdeadbeef; w as big Word[8]^4 }
  spec number() -> Int { hex"0100" as big Int }
  spec minus_one() -> Word[8]^4 { let n: Int = -1; n as big Word[8]^4 }
}
```

```console
$ orangec eval order.or
order::big_word: Word[32] = 0x01020304
order::little_word: Word[32] = 0x04030201
order::text: Word[32] = 0x61626364
order::bytes: Word[8]^4 = [0xde, 0xad, 0xbe, 0xef]
order::number: Int = 256
order::minus_one: Word[8]^4 = [0xff, 0xff, 0xff, 0xff]
```

SHA-256 then reads its blocks and writes its padding's length exactly as FIPS
180-4 describes them, and Poly1305 reads its key and each block of the message
as RFC 8439 section 2.5 does, as little-endian numbers, the block straight into
the field of integers modulo 2^130 − 5:

```orange
// Section 5.1.1: the message, the bit 1 and then zeros, as a byte 80 and
// zero bytes, and the message's length in bits as a big-endian 64-bit
// number fill ((len + 8) / 64) + 1 blocks.
spec pad[len in 1..120](m: Word[8]^len) -> Word[8]^(64 * (((len + 8) / 64) + 1)) {
  m ++ ([0; ((64 * (((len + 8) / 64) + 1)) - len - 8)] with [0] = 0x80)
    ++ ((8 * len) as big Word[8]^8)
}

// Section 6.2.2, step 1: the message schedule of one 64-byte block. Its
// first sixteen words are the block itself, read as big-endian words.
spec schedule(block: Word[8]^64) -> Word[32]^64 {
  let head: Word[32]^16 = block as big Word[32]^16;
  for t in 16..64 with w: Word[32]^64 = head ++ [0; 48] {
    w with [t] = small_sigma1(w[t - 2]) + w[t - 7] + small_sigma0(w[t - 15]) + w[t - 16]
  }
}
```

```orange
type P = Mod[(1 << 130) - 5];

spec mac[len in 1..256](key: Word[8]^32, m: Word[8]^len) -> Word[8]^16 {
  // r &= 0x0ffffffc0ffffffc0ffffffc0fffffff, on its two 64-bit halves.
  let half: Word[64]^2 = key[..16] as little Word[64]^2;
  let r: P = [half[0] & 0x0ffffffc0fffffff, half[1] & 0x0ffffffc0ffffffc] as little P;
  let s: Int = key[16..] as little Int;
  let padded: Word[8]^(16 * ((len / 16) + 1)) = m ++ [0; (16 - (len % 16))];
  let a: P = for j in 0..((len + 15) / 16) with a: P = 0 {
    let held: Int = if j == (((len + 15) / 16) - 1) { len - (16 * j) } else { 16 };
    (a + (padded[16 * j..16 * j + 16] as little P) + weight(held)) * r
  };
  ((a as Int) + s) as little Word[8]^16
}
```

```text
poly1305::example: Word[8]^16 = [0xa8, 0x06, 0x1d, 0xc1, 0x30, 0x51, 0x36, 0xc6, 0xc2, 0x2b, 0x8b, 0xaf, 0x0c, 0x01, 0x27, 0xa9]
```

The tag is the low 128 bits of the accumulator plus s, which is exactly what
writing that sum as sixteen bytes keeps: a number becomes words by its residue,
as `as Word[32]` already wraps one word. Words convert only to words of the
same number of bits, so `Word[8]^3 as big Word[32]` is an error that counts
both sides' bits, and `big` and `little` are words only directly after `as`
and before a type, so no program that used them as names changes meaning. The
fixtures of [SHA-256](compiler/fixtures/s3n/valid-sha256.or),
[SHA-512](compiler/fixtures/s3n/valid-sha512.or),
[ChaCha20](compiler/fixtures/s3n/valid-chacha20.or),
[Poly1305](compiler/fixtures/s3n/valid-poly1305.or), and
[X25519](compiler/fixtures/s3n/valid-x25519.or) reproduce FIPS 180-4's, RFC
8439's, and RFC 7748's published values. The three schemes of `orangec enc`
read and write their words the same way, and seal a megabyte in about a third
of the time they took, with the same bytes. This slice, S3n, is implemented
and tested; its specification is in review as
[OEP-0017](docs/governance/oeps/OEP-0017-orange-2026-byte-order.md).

### One function for several types

Square-and-multiply raises an element to a power the same way in every field,
and Fermat's little theorem inverts in every field of prime order. RFC 7748
computes modulo 2^255 − 19, RFC 8439 modulo 2^130 − 5, and FIPS 203 modulo
3329, and a program that needed all three wrote the same function three
times. A `spec` may declare a **type parameter** in its brackets, beside or
instead of sizes, listing the types it is written for, `K in {F, P, Q}`, and
use its name wherever a type is written:

```orange
edition 2026;
module fields {
  type F = Mod[(1 << 255) - 19];
  type P = Mod[(1 << 130) - 5];
  type Q = Mod[3329];

  // The modulus m: one more than the least residue of -1.
  spec modulus[K in {F, P, Q}]() -> Int {
    let minus_one: K = 0 - 1;
    (minus_one as Int) + 1
  }

  // x^e for 0 <= e < 2^256, squaring x once for each bit of e.
  spec pow[K in {F, P, Q}](x: K, e: Int) -> K {
    let (square: K, power: K, rest: Int) =
      for i in 0..256 with (square: K, power: K, rest: Int) = (x, 1, e) {
        (square * square, if (rest % 2) == 1 { power * square } else { power }, rest / 2)
      };
    power
  }

  // Fermat's little theorem: x^(m - 2) inverts x in every field of prime order.
  spec inverse[K in {F, P, Q}](x: K) -> K { pow(x, modulus[K]() - 2) }

  spec inverts[K in {F, P, Q}]() -> Bool {
    let x: K = 1234;
    (x * inverse(x)) == 1
  }

  // FIPS 203 builds ML-KEM's transform on 17, a primitive 256th root of unity.
  spec kem_root() -> Q { pow(17, 128) }
}
```

```console
$ orangec eval fields.or
fields::modulus[F]: Int = 57896044618658097711785492504343953926634992332820282019728792003956564819949
fields::modulus[P]: Int = 1361129467683753853853498429727072845819
fields::modulus[Q]: Int = 3329
fields::inverts[F]: Bool = true
fields::inverts[P]: Bool = true
fields::inverts[Q]: Bool = true
fields::kem_root: Mod[3329] = 3328
```

`pow` stands for three functions, `pow[F]`, `pow[P]`, and `pow[Q]`, and the
compiler checks each before anything runs, exactly as it would check it
written out with that field, so every operator, literal, and call is checked
in every field. A call names its instance by its types, as `modulus[K]()`, or
lets its arguments' types choose, as `inverse(x)` does with `x: K`; where they
do not decide, the type the call's place expects does, so `pow(17, 128)` in
`kem_root` is `pow[Q]`, and 17^128 is −1 modulo 3329. A function that is
wrong for one of its types is reported in that instance:

```orange
edition 2026;
module half {
  type Q = Mod[3329];

  spec half[K in {Word[32], Q}](x: K) -> K { x >> 1 }
}
```

```console
$ orangec check half.or
error[ORC0215]: `>>` is not defined for `Mod[3329]`
 --> half.or:5:48
  |
5 | ... half[K in {Word[32], Q}](x: K) -> K { x >> 1 }
  |                                             ^^ `Mod[3329]` is required here
  = note: shifts and rotations apply only to `Word[n]` values
  = note: in the instance `half[Q]`, the first of `half` in error: a function is checked once for each type of its type parameters
```

FIPS 180-4 defines Ch, Maj, and the round of SHA-256 and SHA-512 by the same
formulas on words of 32 and of 64 bits. One module writes them once for both,
and each hash's compression function calls them without brackets, on its own
words:

```orange
// Sections 4.1.2 and 4.1.3: the same Ch and Maj on words of either width.
spec ch[W in {Word[32], Word[64]}](x: W, y: W, z: W) -> W { (x & y) ^ (~x & z) }
spec maj[W in {Word[32], Word[64]}](x: W, y: W, z: W) -> W { (x & y) ^ (x & z) ^ (y & z) }

// Step 3 of sections 6.2.2 and 6.4.2: one round on the working variables,
// given the round's two Sigma values and its constant plus schedule word.
spec round[W in {Word[32], Word[64]}](
  v: (W, W, W, W, W, W, W, W),
  sigma0: W,
  sigma1: W,
  kw: W,
) -> (W, W, W, W, W, W, W, W) {
  let (a: W, b: W, c: W, d: W, e: W, f: W, g: W, h: W) = v;
  let t1: W = h + sigma1 + ch(e, f, g) + kw;
  let t2: W = sigma0 + maj(a, b, c);
  (t1 + t2, a, b, c, d + t1, e, f, g)
}
```

```text
sha2::sha512_abc: Word[8]^64 = [0xdd, 0xaf, 0x35, 0xa1, 0x93, 0x61, 0x7a, 0xba, 0xcc, 0x41, 0x73, 0x49, 0xae, 0x20, 0x41, 0x31, 0x12, 0xe6, 0xfa, 0x4e, 0x89, 0xa9, 0x7e, 0xa2, 0x0a, 0x9e, 0xee, 0xe6, 0x4b, 0x55, 0xd3, 0x9a, 0x21, 0x92, 0x99, 0x2a, 0x27, 0x4f, 0xc1, 0xa8, 0x36, 0xba, 0x3c, 0x23, 0xa3, 0xfe, 0xeb, 0xbd, 0x45, 0x4d, 0x44, 0x23, 0x64, 0x3c, 0xe8, 0x0e, 0x2a, 0x9a, 0xc9, 0x4f, 0xa5, 0x4c, 0xa4, 0x9f]
```

Nothing is generic at run time: a type parameter is a type, different in each
instance, and the list says in the source which types a function was checked
for. A listed type is the same for every instance, so its lengths are written
without sizes, and a function has at most four parameters in brackets and
256 instances, sizes and types together. The
[field fixture](compiler/fixtures/s3o/valid-fields.or) writes its arithmetic
once for five prime fields, those above and the order of the Curve25519
subgroup and ML-DSA's modulus 8380417, and reproduces the square root of −1
that RFC 8032 decodes points with and the roots of unity of FIPS 203 and FIPS
204; the [SHA-2 fixture](compiler/fixtures/s3o/valid-sha2.or) reproduces FIPS
180-4's digests of "abc" and of the two-block messages for SHA-256 and
SHA-512. This slice, S3o, is implemented and tested; its specification is in
review as
[OEP-0018](docs/governance/oeps/OEP-0018-orange-2026-type-parameters.md).

### Vectors at full length

The objects of cryptography are long. An ML-KEM-512 ciphertext is 768 bytes,
an ML-DSA-44 signature 2,420, and an RSA-4096 block 512; RFC 8439 prints test
vectors of 375 and 265 bytes. An Orange array, array literal, or byte string
holds up to **65,536** elements: as many as a loop visits, and exactly the
values of a 16-bit word, so a `Word[16]` indexes the longest array with no
check at run time. RFC 8439's appendix A.5 is written as the RFC prints it,
and ChaCha20 is written once for messages of 1 through 256 whole blocks:

```orange
spec a5_ciphertext() -> Word[8]^265 {
  hex"64 a0 86 15 75 86 1a f4 60 f0 62 c7 9b e6 43 bd" ++
    hex"5e 80 5c fd 34 5c f3 89 f1 08 67 0a c7 6c 8c b2" ++
    ...
    hex"a6 ad 5c b4 02 2b 02 70 9b"
}

spec encrypt[blocks in 1..257](
  key: Word[8]^32, counter: Word[32], nonce: Word[8]^12, plaintext: Word[8]^(64 * blocks),
) -> Word[8]^(64 * blocks) {
  for j in 0..blocks with c: Word[8]^(64 * blocks) = plaintext {
    c with [64 * j..64 * j + 64] =
      xor64(plaintext[64 * j..64 * j + 64], block(key, counter + (j as Word[32]), nonce))
  }
}

// The plaintext is the ciphertext exclusive-ored with the key stream from
// block 1, padded to five blocks and cut back to its 265 bytes.
spec a5_plaintext() -> Word[8]^265 {
  encrypt(a5_key(), 1, a5_nonce(), a5_ciphertext() ++ [0; 55])[..265]
}
```

Nothing about costs changes: making an array costs one step for each 64 of
its elements, so a long table is built in rows placed with slice updates, and
an evaluation's memory stays bounded by its steps. `orangec eval` takes
three options for longer work: `--steps N` sets the step budget, up to
1,073,741,824; `--spec NAME` evaluates only the functions it names; and
`--stats` reports on standard error, after the values, the steps each used:

```console
$ orangec eval --spec a5_authentic --spec a5_opens_to_text --stats compiler/fixtures/s3p/valid-rfc8439.or
rfc8439::a5_authentic: Bool = true
rfc8439::a5_opens_to_text: Bool = true
rfc8439::a5_authentic: 24802 steps
rfc8439::a5_opens_to_text: 28442 steps
total: 53244 of 1048576 steps
```

The [RFC 8439 fixture](compiler/fixtures/s3p/valid-rfc8439.or) reproduces the
375-byte ciphertext of appendix A.2 test vector 2, the tags of appendix A.3
test vectors 2 and 3, and appendix A.5's authentication and plaintext, and the
[lengths fixture](compiler/fixtures/s3p/valid-lengths.or) builds the 65,536
powers of 3 modulo the Fermat prime 2^16 + 1 and reads them by 16-bit words
for Pepin's test. A conversion of words to a number still stops at the
evaluator's 16,384-bit limit, now at run time, since a long array can spell a
longer number. This slice, S3p, is implemented and tested; its specification
is in review as
[OEP-0019](docs/governance/oeps/OEP-0019-orange-2026-lengths.md).

### Known answers beside the algorithm

Every standard ends in numbers: a key, a nonce, and the bytes an
implementation must give from them. In Orange those numbers live in the
program, as tests beside the functions they check, each titled with where its
claim comes from:

```orange
test "2.1.1: the quarter round" {
  quarter_round(0x11111111, 0x01020304, 0x9b8d6f43, 0x01234567)
    == (0xea2a92f4, 0xcb1cf8ce, 0x4581472e, 0x5881c4bb)
}

test "2.5.2: Poly1305 of the Forum's name" {
  let key: Word[8]^32 =
    hex"85 d6 be 78 57 55 6d 33 7f 44 52 fe 42 d5 06 a8" ++
      hex"01 03 80 8a fb 0d b2 fd 4a bf f6 af 41 49 f5 1b";
  let message: Word[8]^34 = "Cryptographic Forum Research Group";
  mac(key, message ++ [0; 14], 2) == hex"a8 06 1d c1 30 51 36 c6 c2 2b 8b af 0c 01 27 a9"
}
```

A test is a title and a claim: a `Bool` expression, with its own `let`
bindings, over the module's functions. `==` and `!=` compare arrays and tuples
whole, and every element is compared wherever they differ, so what a
comparison costs says nothing about where. `orangec test` checks the program
and runs its tests in order:

```console
$ orangec test compiler/fixtures/s3q/valid-rfc8439-tests.or
test "2.1.1: the quarter round" ... ok
test "2.3.2: the block function" ... ok
test "A.1 #1: the zero key's key stream, block 0" ... ok
test "A.1 #2: the zero key's key stream, block 1" ... ok
test "the nonce changes every block" ... ok
test "2.5.2: Poly1305 of the Forum's name" ... ok
test "A.3 #1: Poly1305 of zeros under the zero key" ... ok
7 tests: 7 passed, 0 failed
```

A claim that fails shows both sides and where they part:

```text
test "a tuple holding an array" ... FAILED
    left:  (0x01, [0x02, 0x03, 0x04])
    right: (0x01, [0x02, 0x03, 0x05])
    first difference at .1[2]
```

`orangec test` exits with status 1 when any test fails, so a claim that stops
holding stops the build. Only the tests of the file given are run, and
`orangec eval` runs none. This slice, S3q, is implemented and tested; its
specification is in review as
[OEP-0020](docs/governance/oeps/OEP-0020-orange-2026-tests.md).

### Amounts computed from data

RC6 rotates its words by amounts its data choose, and SHA-3 turns each lane of
its state by an amount it computes. A shift or rotation takes any `Int` or
word as its amount, written the way the standard writes it:

```orange
// "The RC6 Block Cipher": twenty rounds of
//   t = f(B); u = f(D)
//   A = ((A ^ t) <<< u) + S[2i]; C = ((C ^ u) <<< t) + S[2i + 1]
//   (A, B, C, D) = (B, C, D, A)
spec rounds(s: Word[32]^44, x: Word[32]^4) -> Registers {
  for i in 1..21 with (a: Word[32], b: Word[32], c: Word[32], d: Word[32]) =
    (x[0], x[1] + s[0], x[2], x[3] + s[1]) {
    let t: Word[32] = f(b);
    let u: Word[32] = f(d);
    (b, ((c ^ u) <<< t) + s[2 * i + 1], d, ((a ^ t) <<< u) + s[2 * i])
  }
}
```

Every amount has the value the mathematics gives it. A shift by the width or
more gives 0 and a negative amount shifts the other way, since `a << k` is
floor(a · 2^k) kept to the word; a rotation turns by its amount modulo the
width, so RC6's "least significant lg w bits" need no mask. Nothing is left to
the machine: `x << 32` on a 32-bit word computed from data is 0 everywhere,
where C leaves it undefined and x86 gives `x`. An amount written as one
literal is still a bit position from 0 through n − 1, and a computed amount
costs one step whatever its size.

```console
$ orangec test compiler/fixtures/s3r/valid-rc6.or
test "RC6 paper, 128-bit key 1: encryption" ... ok
test "RC6 paper, 128-bit key 1: decryption" ... ok
test "RC6 paper, 128-bit key 2: encryption" ... ok
test "RC6 paper, 128-bit key 2: decryption" ... ok
4 tests: 4 passed, 0 failed
```

The [SHA3-256 fixture](compiler/fixtures/s3r/valid-sha3.or) computes rho's
offsets and iota's round constants as FIPS 202 defines them and reproduces
NIST's examples, and the [zetas fixture](compiler/fixtures/s3r/valid-zetas.or)
derives ML-KEM's constants by reversing the bits of an index, as FIPS 203
does. This slice, S3r, is implemented and tested; its specification is in
review as
[OEP-0021](docs/governance/oeps/OEP-0021-orange-2026-computed-amounts.md).

### States and polynomial vectors as rows

A matrix is an array of scalar rows, with both dimensions in its type:

```orange
edition 2026;
module rows {
  type Row = Word[32]^4;
  type Matrix = Row^4;
  spec diagonal(m: Matrix) -> Row {
    for i in 0..4 with out: Row = [0; 4] { out with [i] = m[i][i] }
  }
  test "diagonal" { diagonal([[1, 2, 3, 4]; 4]) == [1, 2, 3, 4] }
}
```

Each index is checked against its own axis. Rows must have identical types,
and a matrix holds at most 65,536 scalar elements. Updates, slices, joins,
tuples, and finite specialization retain the whole shape; a byte-order
conversion requires an explicitly selected row. S3s is implemented and
tested, with its specification in review as
[OEP-0023](docs/governance/oeps/OEP-0023-orange-2026-nested-arrays.md).

### Moduli from finite sizes

One pure definition can use a different residue domain in each size instance:

```orange
edition 2026;
module rings {
  spec add[m in 2..8](a: Mod[m], b: Mod[m]) -> Mod[m] { a + b }
  spec result() -> (Mod[3], Mod[4]) { (add[3](2, 2), add[4](2, 2)) }
  test "exact domains" { (add[3](2, 2) == 1) && (add[4](2, 2) == 0) }
}
```

Every declared instance is checked before anything runs. Modulus expressions
can use the function's own finite size names, and each result retains its exact
domain. Module aliases and finite type lists remain concrete. S3t is implemented
with its [specification](docs/STATIC_MODULI_2026.md) and
[OEP-0024](docs/governance/oeps/OEP-0024-orange-2026-static-moduli.md) in review.

The [five-limb field definitions](algorithms/x25519/field25519-limbs.or)
also give executable reconstruction, abstraction, tight/loose/canonical
predicates, addition, carrying and canonicalization for p = 2^255 − 19.
Their mathematical boundary tests do not establish a refinement proof or
verified machine arithmetic.

### Daylight Horizon example

[`examples/daylight/`](examples/daylight/README.md) is Daylight Horizon v17's
seal written as one Orange program: SHA-256, HMAC, HKDF, ChaCha20, Poly1305,
and their AEAD, each as its standard writes it, with `seal`, `open`, and
`authentic` on top. `orangec eval` reproduces the upstream frame byte for byte,
and a bridge runs the upstream vault on the same specifications beneath its
evidence checks. It is executable reference code, not verified production
cryptography.

## What works today

| Area | Status |
| --- | --- |
| Source model, UTF-8 byte spans, stable diagnostic codes | Working |
| Deterministic lexer (`orangec lex`) | Working |
| Syntax-only formatter (`orangec fmt` and `fmt --check`), preserving token spellings and comments | Working; [tool contract](docs/FORMATTER_2026.md) |
| Syntax-only offline documentation (`orangec doc`), with written declarations and escaped source | Working; [tool contract](docs/DOCUMENTATION_2026.md) |
| Orange 2026 grammar: one edition, one module per file, `spec` and `impl` declarations | Working |
| Typed `spec` functions: parameters, calls, `Int`, and `Word[8]` through `Word[64]` | Working; specification in review ([OEP-0005](docs/governance/oeps/OEP-0005-orange-2026-pure-spec-expressions.md)) |
| Operators: exact `Int` arithmetic, word ring arithmetic, and, or, xor, not, shifts, rotations | Working; specification in review |
| Typed `let` bindings and explicit `as` conversions | Working; specification in review ([OEP-0006](docs/governance/oeps/OEP-0006-orange-2026-bindings-and-conversions.md)) |
| Fixed-length arrays `T^n`, array literals, and literal indices | Working; specification in review ([OEP-0007](docs/governance/oeps/OEP-0007-orange-2026-fixed-length-arrays.md)) |
| Rectangular arrays of scalar rows, with both axes checked and a bounded scalar product | Working; specification in review ([OEP-0023](docs/governance/oeps/OEP-0023-orange-2026-nested-arrays.md)) |
| Bounded loops, indices proved in range, updates, and fill literals | Working; specification in review ([OEP-0008](docs/governance/oeps/OEP-0008-orange-2026-bounded-loops.md)) |
| `Bool`, comparisons, Euclidean division, and conditionals | Working; specification in review ([OEP-0009](docs/governance/oeps/OEP-0009-orange-2026-conditions.md)) |
| Indices keyed by data, proved in range from their types | Working; specification in review ([OEP-0010](docs/governance/oeps/OEP-0010-orange-2026-lookups.md)) |
| Programs of more than one module, each in its own file, with calls qualified by module | Working; specification in review ([OEP-0011](docs/governance/oeps/OEP-0011-orange-2026-modules.md)) |
| Integers modulo a constant, `Mod[m]`, with total division, and `type` declarations | Working; specification in review ([OEP-0012](docs/governance/oeps/OEP-0012-orange-2026-modular-arithmetic.md)) |
| Modulus expressions over own finite size parameters, with eagerly checked exact domains | Working; specification in review ([OEP-0024](docs/governance/oeps/OEP-0024-orange-2026-static-moduli.md)) |
| `let` bindings inside a loop's step and each branch of a conditional | Working; specification in review ([OEP-0013](docs/governance/oeps/OEP-0013-orange-2026-blocks.md)) |
| Tuples, `.k`, and tuple patterns, so that a function gives several values and a loop carries several accumulators | Working; specification in review ([OEP-0014](docs/governance/oeps/OEP-0014-orange-2026-tuples.md)) |
| Byte strings `"..."` and `hex"..."`, `++` joins, and slices at bounds proved in range | Working; specification in review ([OEP-0015](docs/governance/oeps/OEP-0015-orange-2026-bytes.md)) |
| Size parameters: one `spec` for every length in a range, each instance checked before anything runs | Working; specification in review ([OEP-0016](docs/governance/oeps/OEP-0016-orange-2026-sizes.md)) |
| Byte orders: `as big` and `as little` read words as words of another width, a number, or a residue, and write numbers as words | Working; specification in review ([OEP-0017](docs/governance/oeps/OEP-0017-orange-2026-byte-order.md)) |
| Type parameters: one `spec` for a list of types, such as several fields or word widths, each instance checked before anything runs | Working; specification in review ([OEP-0018](docs/governance/oeps/OEP-0018-orange-2026-type-parameters.md)) |
| Arrays, literals, and byte strings of up to 65,536 elements, and `orangec eval --steps`, `--spec`, and `--stats` | Working; specification in review ([OEP-0019](docs/governance/oeps/OEP-0019-orange-2026-lengths.md)) |
| Known-answer tests `test "TITLE" { claim }` beside the functions, `==` on whole arrays and tuples, and `orangec test` | Working; specification in review ([OEP-0020](docs/governance/oeps/OEP-0020-orange-2026-tests.md)) |
| Shift and rotation amounts computed from data, `x <<< r` or `x >> (i % 8)`, with a value at every amount | Working; specification in review ([OEP-0021](docs/governance/oeps/OEP-0021-orange-2026-computed-amounts.md)) |
| Typed Reference Core and reference evaluator (`orangec eval`) | Working |
| Functions over every type rather than a listed few, sizes checked once for all values, imports of names into scope | Not yet |
| Typed `impl` bodies and refinement between `spec` and `impl` | Not yet |
| Proof checking, claim reports, evidence bundles | Proposed; decisions open (D-005, D-006, D-007); not built |
| Code generation, native targets, C ABI | Proposed; strategy under investigation (D-010, D-011, D-013); not built |
| Cryptography corpus (hashes, AEADs, signatures, KEMs) | Planned |
| Packages and releases | Planned; no release exists |

The target remains the complete 1.0 product. The
[execution record](docs/RELEASE_1_0_EXECUTION.md) maps all charter requirements
and eight journeys to engineering and actual owner decisions. The current
language and representation work supplies part of that path; no complete
release or journey is claimed.

## Quick start

You need [rustup](https://rustup.rs). The repository pins Rust 1.96.1 in
[`rust-toolchain.toml`](rust-toolchain.toml), so rustup selects it
automatically. The compiler has no third-party dependencies.

```sh
git clone https://github.com/chasebryan/orange.git
cd orange

# Build and try the compiler
cargo run --manifest-path compiler/Cargo.toml -p orangec -- test compiler/fixtures/s3q/valid-rfc8439-tests.or
cargo run --manifest-path compiler/Cargo.toml -p orangec -- test compiler/fixtures/s3r/valid-sha3.or
cargo run --manifest-path compiler/Cargo.toml -p orangec -- eval compiler/fixtures/s3i/valid-x25519.or
cargo run --manifest-path compiler/Cargo.toml -p orangec -- eval --stats compiler/fixtures/s3p/valid-rfc8439.or
cargo run --manifest-path compiler/Cargo.toml -p orangec -- eval compiler/fixtures/s3h/valid-vectors.or
cargo run --manifest-path compiler/Cargo.toml -p orangec -- eval compiler/fixtures/s3g/valid-aes128.or
cargo run --manifest-path compiler/Cargo.toml -p orangec -- eval compiler/fixtures/s3b/valid-sha256-functions.or
cargo run --manifest-path compiler/Cargo.toml -p orangec -- check compiler/fixtures/hello.or
cargo run --manifest-path compiler/Cargo.toml -p orangec -- lex compiler/fixtures/hello.or
cargo run --manifest-path compiler/Cargo.toml -p orangec -- fmt compiler/fixtures/hello.or
cargo run --manifest-path compiler/Cargo.toml -p orangec -- doc compiler/fixtures/hello.or

# Run the compiler test suite
cargo test --manifest-path compiler/Cargo.toml --workspace

# Run the local repository gate: policy checks and sandboxed compiler checks
scripts/ci/check-repository
```

The repository gate runs on Linux 6.2 or newer (Landlock ABI 3) and needs a C
compiler and Python 3 (on Ubuntu, `sudo apt install build-essential`) plus
rustup's pinned toolchain with its components (`rustup toolchain install
1.96.1 --component clippy,rustfmt`). It builds its sandbox from user
namespaces; where the host blocks unprivileged ones, as Ubuntu 23.10 and newer
do, the gate (`make`, which `scripts/ci/check-repository` runs) asks for your
sudo password once and builds the same sandbox through sudo. The [policy guide](policy/README.md) explains the sandbox.
Markdown lint, workflow audits, and link checks run only in CI.

`orangec` reads a file path, or `-` for standard input:

```text
Usage: orangec [OPTIONS] <check|eval|lex> <FILE>...
       orangec eval [--steps <N>] [--spec <NAME>]... [--stats] <FILE>
       orangec test [--steps <N>] [--stats] <FILE>
       orangec fmt <FILE>
       orangec fmt --check <FILE>...
       orangec doc <FILE>
       orangec keygen [--scheme <NAME>] [-o <FILE>]
       orangec <enc|dec> [--key <FILE>] [--scheme <NAME>] [-o <FILE>] <FILE>
       orangec schemes [<NAME>...]

Commands:
  check    Perform lexical, syntactic, and semantic validation
  eval     Reference-evaluate one source after complete validation
  lex      Print the deterministic token stream
  test     Run one source's known-answer tests after complete validation
  fmt      Format one source, or check source formatting with --check
  doc      Document one parsed source as standalone HTML
  keygen   Make a secret key for a scheme [default: xchacha20_poly1305]
  enc      Seal a file with the scheme its key belongs to
  dec      Open a sealed file, writing nothing unless all of it is authentic
  schemes  List the built-in sealing schemes, or describe the named ones
```

`orangec fmt FILE` prints one complete formatted source. `orangec fmt --check
FILE...` reports sources that differ from that format and changes no files.
Both accept `-` for standard input and validate syntax without loading imports
or checking types. The [formatter contract](docs/FORMATTER_2026.md) describes
comment preservation, bounds and diagnostics. This is permanent frontend
tooling; it does not close S8 or authorize a release.

`orangec doc FILE` prints a standalone offline HTML reference for one parsed
source, with written declarations and a full escaped source listing. It loads
no imports, checks no types and runs no tests. The
[documentation contract](docs/DOCUMENTATION_2026.md) defines its bounded
rendering and source-only scope; proof and ABI documentation remain later
product obligations.

`orangec enc FILE` seals any file with an authenticated cipher written in
Orange, and `orangec dec FILE.orange` opens it again. XChaCha20-Poly1305 (the
default), ChaCha20-Poly1305, and Ascon-AEAD128 are built in, and any Orange
program with `seal`, `open`, and `authentic` specifications is a scheme too,
including one that uses other modules. The
[scheme guide](compiler/schemes/README.md) specifies the file format and
states its limits: the evaluator is not constant-time, nothing is verified,
and keys are stored unencrypted.

The [compiler guide](compiler/README.md) covers the grammar, diagnostics, and
test corpora in detail.

## Roadmap

Orange is built in dependency order. Each stage adds permanent components to
the production compiler; there is no throwaway prototype.

| Stage | Delivers | Status |
| --- | --- | --- |
| S0 | Repository foundation: governance, CI, policy checks | Done |
| S1 | Compiler foundation: source model, spans, diagnostics, lexer, CLI | Done |
| S2 | Editioned grammar and bounded parser | Done |
| S3 | Name resolution, types, expressions, typed Core, reference evaluator | In progress: typed literals done; pure expressions, bindings, conversions, arrays, loops, conditions, lookups, modules, modular arithmetic, blocks, tuples, bytes, sizes, byte orders, type parameters, long arrays, known-answer tests, and computed amounts in review |
| S4 | Proof and claim boundary | Research underway |
| S5 | Compiler IRs and one output path | Open |
| S6 | Memory, leakage, ABI, and native targets | Open |
| S7 | Cryptography corpus | Open |
| S8 | Packages, developer tools, and preview releases | Open |
| 1.0 | Stable release | Open |

Three of the ten gates are closed. That counts finished stages, not effort or
time remaining. The [roadmap](docs/ROADMAP.md) has the details, and the
[decision register](docs/DECISIONS.md) tracks every open design choice.

## Read more

- **[The Orange Book](docs/THE_ORANGE_BOOK.md)**: the reader's guide to why
  Orange exists, how it is designed, and what has been built. Start here.
- [Orange 2026 language specification](docs/LANGUAGE_2026.md),
  [typed-literal semantics](docs/SEMANTICS_2026.md), and the proposed
  [pure expression semantics](docs/EXPRESSIONS_2026.md),
  [bindings and conversions](docs/BINDINGS_2026.md),
  [fixed-length arrays](docs/ARRAYS_2026.md),
  [bounded loops](docs/LOOPS_2026.md),
  [conditions and division](docs/CONDITIONS_2026.md),
  [lookups keyed by data](docs/LOOKUPS_2026.md),
  [programs of more than one module](docs/MODULES_2026.md),
  [integers modulo a constant](docs/MODULAR_2026.md),
  [blocks](docs/BLOCKS_2026.md), [tuples](docs/TUPLES_2026.md),
  [bytes](docs/BYTES_2026.md), [sizes](docs/SIZES_2026.md),
  [byte order](docs/ORDER_2026.md),
  [type parameters](docs/TYPE_PARAMETERS_2026.md),
  [lengths and evaluation controls](docs/LENGTHS_2026.md),
  [known-answer tests](docs/TESTS_2026.md), and
  [computed amounts](docs/AMOUNTS_2026.md): the definition of what the
  compiler accepts today.
- [Compiler guide](compiler/README.md): commands, diagnostics, and tests.
- [Tabula](tabula/README.md): a local workbench for writing Orange, with the
  compiler's results and this documentation beside the editor. It is a
  separate tool, not part of the language.
- [Architecture](docs/ARCHITECTURE.md) and
  [assurance model](docs/ASSURANCE.md): the intended end state.
- [Roadmap](docs/ROADMAP.md), [decision register](docs/DECISIONS.md), and
  [project charter](docs/PROJECT_CHARTER.md): scope, sequence, and open
  questions.
- [Research and landscape](docs/RESEARCH.md): how Orange relates to existing
  verified-cryptography work.
- [Governance](GOVERNANCE.md) and
  [Orange Enhancement Proposals](docs/governance/oeps/README.md): how changes
  are decided.

## Repository layout

| Path | Contents |
| --- | --- |
| [`compiler/`](compiler/README.md) | The Rust workspace: the `orange-compiler` library and the `orangec` CLI |
| [`tabula/`](tabula/README.md) | A local workbench for writing Orange; a separate tool, not part of the language |
| [`docs/`](docs/) | The Orange Book, language specification, architecture, assurance, roadmap, and decisions |
| [`research/decisions/`](research/decisions/) | Decision laboratories that compare design candidates |
| [`schemas/`](schemas/README.md) and [`conformance/`](conformance/foundation/README.md) | Provisional evidence schemas and their test fixtures |
| [`policy/`](policy/README.md) and [`tools/`](tools/) | Repository policy and the Python checks that enforce it |
| [`assets/identity/`](assets/identity/README.md) and [`assets/brand/`](assets/brand/README.md) | The Orange emblem, wordmark, README banner, and book covers, and the original brand assets |

## Project status

- **Solo, pre-alpha.** One owner, Chase Bryan, designs, builds, and reviews
  Orange. Owner review is not independent review, and passing tests show only
  that the implemented slice behaves as tested.
- **No license yet.** An outbound license has not been chosen
  ([D-018](docs/DECISIONS.md#d-018--licenses)), so no right to use, copy, or
  redistribute is granted. For the same reason, outside pull requests can't be
  merged yet; issues with facts, sources, and questions are welcome. See
  [CONTRIBUTING.md](CONTRIBUTING.md).
- **Security reports stay private.** Use the process in
  [SECURITY.md](SECURITY.md), never a public issue.
- **Working name.** "Orange" is a working name until naming and trademark
  questions are settled. Other software, including an earlier systems
  language, already uses the name.
