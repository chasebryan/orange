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
`<<<` rotate, `>>` and `<<` shift, and every amount is a literal checked
against the width. `Int` is the type of mathematical integers, with no
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
different values. This slice, S3c, is implemented and tested; its
specification is in review as
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
ladder in one loop, with the RFC's names inside it. This slice, S3i, is
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

A tuple's elements are scalars and arrays, never tuples, and no operator
applies to a whole tuple: `p == q` is an error, and `p.0 == q.0` says which
element is compared. A pattern's names follow the rules of every other name,
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
| Orange 2026 grammar: one edition, one module per file, `spec` and `impl` declarations | Working |
| Typed `spec` functions: parameters, calls, `Int`, and `Word[8]` through `Word[64]` | Working; specification in review ([OEP-0005](docs/governance/oeps/OEP-0005-orange-2026-pure-spec-expressions.md)) |
| Operators: exact `Int` arithmetic, word ring arithmetic, and, or, xor, not, shifts, rotations | Working; specification in review |
| Typed `let` bindings and explicit `as` conversions | Working; specification in review ([OEP-0006](docs/governance/oeps/OEP-0006-orange-2026-bindings-and-conversions.md)) |
| Fixed-length arrays `T^n`, array literals, and literal indices | Working; specification in review ([OEP-0007](docs/governance/oeps/OEP-0007-orange-2026-fixed-length-arrays.md)) |
| Bounded loops, indices proved in range, updates, and fill literals | Working; specification in review ([OEP-0008](docs/governance/oeps/OEP-0008-orange-2026-bounded-loops.md)) |
| `Bool`, comparisons, Euclidean division, and conditionals | Working; specification in review ([OEP-0009](docs/governance/oeps/OEP-0009-orange-2026-conditions.md)) |
| Indices keyed by data, proved in range from their types | Working; specification in review ([OEP-0010](docs/governance/oeps/OEP-0010-orange-2026-lookups.md)) |
| Programs of more than one module, each in its own file, with calls qualified by module | Working; specification in review ([OEP-0011](docs/governance/oeps/OEP-0011-orange-2026-modules.md)) |
| Integers modulo a constant, `Mod[m]`, with total division, and `type` declarations | Working; specification in review ([OEP-0012](docs/governance/oeps/OEP-0012-orange-2026-modular-arithmetic.md)) |
| `let` bindings inside a loop's step and each branch of a conditional | Working; specification in review ([OEP-0013](docs/governance/oeps/OEP-0013-orange-2026-blocks.md)) |
| Tuples, `.k`, and tuple patterns, so that a function gives several values and a loop carries several accumulators | Working; specification in review ([OEP-0014](docs/governance/oeps/OEP-0014-orange-2026-tuples.md)) |
| Byte strings `"..."` and `hex"..."`, `++` joins, and slices at bounds proved in range | Working; specification in review ([OEP-0015](docs/governance/oeps/OEP-0015-orange-2026-bytes.md)) |
| Size parameters: one `spec` for every length in a range, each instance checked before anything runs | Working; specification in review ([OEP-0016](docs/governance/oeps/OEP-0016-orange-2026-sizes.md)) |
| Typed Reference Core and reference evaluator (`orangec eval`) | Working |
| Functions generic over a modulus, sizes checked once for all values, imports of names into scope | Not yet |
| Typed `impl` bodies and refinement between `spec` and `impl` | Not yet |
| Proof checking, claim reports, evidence bundles | Proposed; decisions open (D-005, D-006, D-007); not built |
| Code generation, native targets, C ABI | Proposed; strategy under investigation (D-010, D-011, D-013); not built |
| Cryptography corpus (hashes, AEADs, signatures, KEMs) | Planned |
| Packages and releases | Planned; no release exists |

## Quick start

You need [rustup](https://rustup.rs). The repository pins Rust 1.96.1 in
[`rust-toolchain.toml`](rust-toolchain.toml), so rustup selects it
automatically. The compiler has no third-party dependencies.

```sh
git clone https://github.com/chasebryan/orange.git
cd orange

# Build and try the compiler
cargo run --manifest-path compiler/Cargo.toml -p orangec -- eval compiler/fixtures/s3i/valid-x25519.or
cargo run --manifest-path compiler/Cargo.toml -p orangec -- eval compiler/fixtures/s3h/valid-vectors.or
cargo run --manifest-path compiler/Cargo.toml -p orangec -- eval compiler/fixtures/s3g/valid-aes128.or
cargo run --manifest-path compiler/Cargo.toml -p orangec -- eval compiler/fixtures/s3b/valid-sha256-functions.or
cargo run --manifest-path compiler/Cargo.toml -p orangec -- check compiler/fixtures/hello.or
cargo run --manifest-path compiler/Cargo.toml -p orangec -- lex compiler/fixtures/hello.or

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
       orangec keygen [--scheme <NAME>] [-o <FILE>]
       orangec <enc|dec> [--key <FILE>] [--scheme <NAME>] [-o <FILE>] <FILE>
       orangec schemes [<NAME>...]

Commands:
  check    Perform lexical, syntactic, and semantic validation
  eval     Reference-evaluate one source after complete validation
  lex      Print the deterministic token stream
  keygen   Make a secret key for a scheme [default: xchacha20_poly1305]
  enc      Seal a file with the scheme its key belongs to
  dec      Open a sealed file, writing nothing unless all of it is authentic
  schemes  List the built-in sealing schemes, or describe the named ones
```

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
| S3 | Name resolution, types, expressions, typed Core, reference evaluator | In progress: typed literals done; pure expressions, bindings, conversions, arrays, loops, conditions, lookups, modules, modular arithmetic, blocks, tuples, bytes, and sizes in review |
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
  [bytes](docs/BYTES_2026.md), and [sizes](docs/SIZES_2026.md): the
  definition of what the compiler accepts today.
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
