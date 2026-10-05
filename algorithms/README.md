# Analysis and Dissemination

This folder holds the major encryption algorithms written in Orange, one
folder per algorithm, each written from its standard and evaluated against the
vectors the standard publishes. Explicitly labeled mathematical representation
definitions also carry hand-derived boundary answers. The title names the two things an entry is
for. *Analysis*: the algorithm is laid out the way its designers laid it out,
so that a cryptographer, cryptologist or cryptanalyst can read the structure,
follow every constant back to its source, and see plainly which steps depend
on data. *Dissemination*: the same text is a program that `orangec eval`
runs, so a reader can reproduce the standard's own examples, change an input,
and watch what happens, with nothing between the specification and the
result.

Every entry is a reference evaluation of a specification. None of them is a
corpus entry in the sense of [The Orange Book](../docs/THE_ORANGE_BOOK.md),
chapter 12: no entry makes a constant-time, side-channel, performance,
interoperability or certification claim, and the pure Orange 2026 extensions
they use are proposed, not accepted. Serpent's key schedule uses a
block, so a round key's S-box is named once inside its loop. What an entry
does claim is narrow and checked by machine: the recorded vectors, taken from
the sources each README names, are reproduced byte for byte. Mathematical
boundary answers are identified separately and do not become published
cryptographic vectors or refinement proofs.

## The entries

Twenty entries, 240 published cryptographic vectors and twenty-one mathematical
answer pairs (seven P2 representation pairs and fourteen partial P4 field
operation pairs), for 261 recorded answer pairs. Each row links the
entry's README; its vector count covers the published cryptographic pairs.
The X25519 source count also includes the five-limb source: its seven P2
representation pairs and fourteen partial P4 mathematical field-operation pairs
are separate from the published cryptographic vectors. Standing is the entry's
own summary of the record as of September 2026.

### Block ciphers

| Entry | Standard | Sources | Vectors | Standing |
| --- | --- | --- | ---: | --- |
| [AES](aes/README.md) | FIPS 197; SP 800-38A modes | 2 | 13 | Current; no practical attack on the full cipher |
| [Camellia](camellia/README.md) | RFC 3713 | 1 | 8 | Current; unbroken |
| [ARIA](aria/README.md) | KS X 1213; RFC 5794 | 1 | 8 | Current; unbroken |
| [SM4](sm4/README.md) | GB/T 32907-2016 | 1 | 6 | Current; unbroken |
| [Serpent](serpent/README.md) | AES submission (1998) | 1 | 12 | AES finalist; unbroken |
| [Twofish](twofish/README.md) | AES submission (1998) | 3 | 7 | AES finalist; unbroken |
| [RC6](rc6/README.md) | AES submission, v1.1 (1998) | 1 | 12 | AES finalist; unbroken |
| [DES and Triple DES](des/README.md) | FIPS 46-3; SP 800-67 rev. 2 | 2 | 13 | DES withdrawn and broken by key search; TDEA encryption disallowed after 2023 |

### Authenticated encryption

| Entry | Standard | Sources | Vectors | Standing |
| --- | --- | --- | ---: | --- |
| [AES-GCM](aes-gcm/README.md) | SP 800-38D | 4 | 10 | Current |
| [AES-CCM](aes-ccm/README.md) | SP 800-38C; RFC 3610 | 3 | 7 | Current |
| [ChaCha20-Poly1305 and XChaCha20-Poly1305](chacha20-poly1305/README.md) | RFC 8439; draft-irtf-cfrg-xchacha | 1 | 25 | Current |
| [Ascon-AEAD128 and Ascon-Hash256](ascon/README.md) | SP 800-232 (2025) | 2 | 17 | Current |

### Stream ciphers

| Entry | Standard | Sources | Vectors | Standing |
| --- | --- | --- | ---: | --- |
| [ChaCha20 and XChaCha20](chacha20/README.md) | RFC 8439; draft-irtf-cfrg-xchacha | 1 | 20 | Current |
| [Salsa20 and XSalsa20](salsa20/README.md) | Bernstein's specification; eSTREAM | 1 | 19 | Unbroken; Salsa20/12 in the eSTREAM portfolio |
| [RC4](rc4/README.md) | None; RFC 6229 vectors | 2 | 5 | Broken; prohibited in TLS by RFC 7465 |

### Hash functions and key derivation

| Entry | Standard | Sources | Vectors | Standing |
| --- | --- | --- | ---: | --- |
| [SHA-256 and SHA-512](sha2/README.md) | FIPS 180-4 | 1 | 13 | Current |
| [SHA-3 and SHAKE](sha3/README.md) | FIPS 202 | 1 | 8 | Current |
| [HMAC-SHA-256 and HKDF](hmac-hkdf/README.md) | FIPS 198-1; RFC 4231; RFC 5869 | 2 | 16 | Current |

### Public-key

| Entry | Standard | Sources | Vectors | Standing |
| --- | --- | --- | ---: | --- |
| [X25519](x25519/README.md) | RFC 7748; separate mathematical limb definitions | 5 | 4 | Current |
| [HPKE](hpke/README.md) | RFC 9180 | 4 | 17 | Current |

RSA-OAEP and ML-KEM-512 are being written; each joins the index when its
README is complete and its vectors reproduce within the evaluation budget.

## How an entry is built

Each folder holds a `README.md` and one or more `.or` sources. The README has
the same shape everywhere: *Analysis* (the structure of the algorithm mapped
to the standard's sections and the file's specs; the cryptanalytic record a
reader should know; what the Orange rendering shows and does not express),
*Dissemination* (the files, how to run them, the table of reproduced vectors
with their sources, provenance and the claim boundary) and *Gaps* (what the
language or the evaluator could not express or evaluate, precisely).

A source is one Orange module. Its header comment names the standard, the
sections transcribed and the vectors reproduced; every section of the file
carries a comment naming the section of the standard it follows, and every
departure from the standard's text is explained where it happens. Specs and
parameters use the standard's own names. The byte and word helpers are named
alike across entries (`load_le32`, `le_bytes`, `load_be32`, `be_bytes`, and
their 64-bit forms), so a reader who has followed one entry can follow the
next.

### Vectors

A reproduced vector is a pair of parameterless specs. `<name>` computes a
value with the algorithm; `<name>_expected` states the value published with
the source, as a literal. `orangec eval` prints both:

```console
$ orangec eval algorithms/chacha20/chacha20.or
chacha20::rfc8439_2_4_2: Word[8]^114 = [0x6e, 0x2e, 0x35, 0x9a, ...]
chacha20::rfc8439_2_4_2_expected: Word[8]^114 = [0x6e, 0x2e, 0x35, 0x9a, ...]
```

The same pair convention records seven P2 representation boundaries and
fourteen partial P4 mathematical field-operation boundaries in
[`field25519-limbs.or`](x25519/field25519-limbs.or). Those expected values
are hand-derived from the documented radix, prime and a24, rather than
imported from a standard's cryptographic vector corpus.

Two checks pair them and require equality of type and value, and require
every source to record at least one pair:

```sh
python3 algorithms/verify.py            # every entry, or name a folder
cargo test --manifest-path compiler/Cargo.toml -p orangec --test algorithms
```

The cargo test runs in the repository gate, so a change to the compiler that
alters any recorded value fails CI. An `_expected` value is never adjusted to
match a computation: each README's vectors table says where every value came
from, and, when a standard publishes no example, which oracle produced it
(`hashlib`, the `cryptography` package, `pycryptodome`, or a plain Python
reference written for the entry and checked against a published set).

## What the entries show about Orange 2026

The entries were written against the S3f compiler, and the *Gaps* section of
each describes the limits of that compiler. Writing them made three limits
concrete; they are listed here once. Two have since been eased by slices now
on `main` (proposed, like the rest, and in owner review). The entries keep
their S3f forms, and every one evaluates unchanged under the current
compiler.

**Indices were static.** Under S3f an index may use only literals and loop
indices (`LOOPS_2026.md` section 5, `CONDITIONS_2026.md` section 9), so
`sbox[x]` with a data byte `x` does not check. Table ciphers write the table
as 32 words of 64 bits, eight entries to a word so that each literal reads
like a row of the standard's table, and select an entry by a 32-step loop and
an 8-arm conditional, at about 290 evaluation steps per lookup. DES packs its
6-to-4-bit boxes into four words each; RC4 keeps its permutation packed and
writes it the same way; Serpent needs no lookup because its designers wrote
it bitsliced. S3g ([`LOOKUPS_2026.md`](../docs/LOOKUPS_2026.md)) admits an
index that depends on data when its type bounds it, so a `Word[8]` value
indexes a 256-element array and each selection can be written back as
`t[x]`. Rewriting the entries that way is later work.

**The evaluation budget is per program.** All parameterless specs of one
source share 1,048,576 evaluation steps (`EXPRESSIONS_2026.md` section 14).
Under S3f an update of an n-element array cost n steps; S3g charges ⌈n/64⌉
(`LOOKUPS_2026.md` section 8). Measured on the S3f compiler, an AES block
with table lookups cost about 60,000 steps, a SHA-256 compression about
3,000, a Keccak-f[1600] permutation about 10,000, and one X25519 about
600,000. Entries whose vectors do not fit together put each in its own
complete file: X25519 has one vector per file, and HPKE takes the
encapsulated key from its vector and reproduces its derivation in the X25519
entry. The budget itself is unchanged, so these splits stand.

**There were no imports.** Each source is self-contained, so HPKE carries its
own SHA-256, HMAC, HKDF, X25519, ChaCha20 and Poly1305, and every AES-GCM
file its own AES. S3h ([`MODULES_2026.md`](../docs/MODULES_2026.md)) lets a
source use the modules beside it (`use aes;` reads `aes.or` from the same
folder, and `aes::cipher(...)` calls it), so the files of one entry could
share one copy of their cipher. A module in another folder is still out of
reach, so an entry that needs another entry's algorithm keeps its own copy.

**An array held 256 elements.** Through S3o an array held at most 256
elements. S3p ([`LENGTHS_2026.md`](../docs/LENGTHS_2026.md)) admits 1 through
65,536. Where a note gave that old bound as the reason a message was split
or a schedule packed, the source still has that shape and the language no
longer requires it. Two recorded cases stay beyond one array: Crypto++'s
131,072-byte Salsa20 digests, and CCM's 10-byte associated-data length
encoding, which is for `a >= 2^32`.

## Sources reachable from the build

The READMEs cite standards by their canonical locations (the RFC Editor,
NIST's DOIs). Test vectors were taken from the standards where they publish
them, and otherwise from the designers' reference sets, the Botan and
Wycheproof vector files, or the IETF's machine-readable vector files, as each
vectors table states.
