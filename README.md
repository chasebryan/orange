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
and Orange makes no timing claim until it generates code. This slice, S3f, is
implemented and tested; its specification is in review as
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
the same whoever uses it. This slice, S3h, is implemented and tested; its
specification is in review as
[OEP-0011](docs/governance/oeps/OEP-0011-orange-2026-modules.md).

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
| Typed Reference Core and reference evaluator (`orangec eval`) | Working |
| Mixed-type tuples, a type of integers modulo a prime, imports of names into scope | Not yet |
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
cargo run --manifest-path compiler/Cargo.toml -p orangec -- eval compiler/fixtures/s3h/valid-vectors.or
cargo run --manifest-path compiler/Cargo.toml -p orangec -- eval compiler/fixtures/s3g/valid-aes128.or
cargo run --manifest-path compiler/Cargo.toml -p orangec -- eval compiler/fixtures/s3f/valid-x25519.or
cargo run --manifest-path compiler/Cargo.toml -p orangec -- eval compiler/fixtures/s3b/valid-sha256-functions.or
cargo run --manifest-path compiler/Cargo.toml -p orangec -- check compiler/fixtures/hello.or
cargo run --manifest-path compiler/Cargo.toml -p orangec -- lex compiler/fixtures/hello.or

# Run the compiler test suite
cargo test --manifest-path compiler/Cargo.toml --workspace

# Run the local repository gate: policy checks and sandboxed compiler checks
scripts/ci/check-repository
```

The repository gate runs on Linux and needs a C compiler, Python 3, user
namespaces, and Landlock ABI 3 or newer; the [policy guide](policy/README.md)
explains the sandbox. Markdown lint, workflow audits, and link checks run only
in CI.

`orangec` reads a file path, or `-` for standard input:

```text
Usage: orangec [OPTIONS] <check|eval|lex> <FILE>...

Commands:
  check    Perform lexical, syntactic, and semantic validation
  eval     Reference-evaluate one source after complete validation
  lex      Print the deterministic token stream
```

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
| S3 | Name resolution, types, expressions, typed Core, reference evaluator | In progress: typed literals done; pure expressions, bindings, conversions, arrays, loops, conditions, lookups, and modules in review |
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
  [lookups keyed by data](docs/LOOKUPS_2026.md), and
  [programs of more than one module](docs/MODULES_2026.md): the definition of
  what the compiler accepts today.
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
