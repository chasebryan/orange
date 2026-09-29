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
through 46. An index that depends on data, the classic source of cache-timing
leaks in table-driven code, cannot be written at all. The
[ChaCha20 fixture](compiler/fixtures/s3e/valid-chacha20.or) loads the key and
nonce with loops, runs the ten double rounds as one loop, and encrypts the
"sunscreen" plaintext of RFC 8439 section 2.4.2 to the RFC's ciphertext, byte
for byte. This slice, S3e, is implemented and tested; its specification is in
review as
[OEP-0008](docs/governance/oeps/OEP-0008-orange-2026-bounded-loops.md).

### Daylight Horizon example

[`examples/daylight/`](examples/daylight/README.md) contains an owner-directed
Orange port of Daylight Horizon v17's SHA-256, HKDF, ChaCha20 and Poly1305
computations. It includes a standalone framed-encryption vector, a host adapter
that preserves Horizon's existing evidence-policy checks, and interoperability
tests. This is executable reference code, not verified production cryptography.

## What works today

| Area | Status |
| --- | --- |
| Source model, UTF-8 byte spans, stable diagnostic codes | Working |
| Deterministic lexer (`orangec lex`) | Working |
| Orange 2026 grammar: one edition, one module, `spec` and `impl` declarations | Working |
| Typed `spec` functions: parameters, calls, `Int`, and `Word[8]` through `Word[64]` | Working; specification in review ([OEP-0005](docs/governance/oeps/OEP-0005-orange-2026-pure-spec-expressions.md)) |
| Operators: exact `Int` arithmetic, word ring arithmetic, and, or, xor, not, shifts, rotations | Working; specification in review |
| Typed `let` bindings and explicit `as` conversions | Working; specification in review ([OEP-0006](docs/governance/oeps/OEP-0006-orange-2026-bindings-and-conversions.md)) |
| Fixed-length arrays `T^n`, array literals, and literal indices | Working; specification in review ([OEP-0007](docs/governance/oeps/OEP-0007-orange-2026-fixed-length-arrays.md)) |
| Bounded loops, indices proved in range, updates, and fill literals | Working; specification in review ([OEP-0008](docs/governance/oeps/OEP-0008-orange-2026-bounded-loops.md)) |
| Typed Reference Core and reference evaluator (`orangec eval`) | Working |
| Comparisons, conditionals, data-dependent indices, mixed-type tuples | Not yet |
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
| S3 | Name resolution, types, expressions, typed Core, reference evaluator | In progress: typed literals done; pure expressions, bindings, conversions, arrays, and loops in review |
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
  [fixed-length arrays](docs/ARRAYS_2026.md), and
  [bounded loops](docs/LOOPS_2026.md): the definition of what the compiler
  accepts today.
- [Compiler guide](compiler/README.md): commands, diagnostics, and tests.
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
| [`docs/`](docs/) | The Orange Book, language specification, architecture, assurance, roadmap, and decisions |
| [`research/decisions/`](research/decisions/) | Decision laboratories that compare design candidates |
| [`schemas/`](schemas/README.md) and [`conformance/`](conformance/foundation/README.md) | Provisional evidence schemas and their test fixtures |
| [`policy/`](policy/README.md) and [`tools/`](tools/) | Repository policy and the Python checks that enforce it |
| [`assets/brand/`](assets/brand/README.md) | Orange emblem, wordmark, and banners |

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
