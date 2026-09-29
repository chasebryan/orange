# Analysis and Dissemination

This folder holds the major encryption algorithms written in Orange, one
folder per algorithm, each written from its standard and evaluated against the
vectors the standard publishes. The title names the two things an entry is
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
interoperability or certification claim, and the Orange 2026 slices they use
(S3b through S3f) are proposed, not accepted. What an entry does claim is
narrow and checked by machine: the recorded vectors, taken from the sources
each README names, are reproduced byte for byte.

## The entries

INDEX_TABLE

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

Writing the corpus against the S3f compiler made three limits of the current
slices concrete. Each entry records how it met them; they are listed here once.

**Indices are static.** An index may use only literals and loop indices
(`LOOPS_2026.md` section 5, `CONDITIONS_2026.md` section 9), so `sbox[x]`
with a data byte `x` does not check. Table ciphers write the table as 32
words of 64 bits, eight entries to a word so that each literal reads like a
row of the standard's table, and select an entry by a 32-step loop and an
8-arm conditional, at about 290 evaluation steps per lookup. DES packs its
6-to-4-bit boxes into four words each; RC4 keeps its permutation packed and
writes it the same way; Serpent needs no lookup because its designers wrote
it bitsliced. The planned S3g slice admits indices that are *bounded* rather
than static (a `Word[8]` value indexes a 256-element array by its type), which
turns every such selection back into `t[x]`.

**The evaluation budget is per file.** All parameterless specs of one source
share 1,048,576 evaluation steps (`EXPRESSIONS_2026.md` section 14), and an
update of an n-element array costs n steps. Measured against that budget, an
AES block with table lookups costs about 60,000 steps, a SHA-256 compression
about 3,000, a Keccak-f[1600] permutation about 10,000, one X25519 about
600,000, one 256-coefficient NTT about 460,000, and a 2048-bit RSA
exponentiation with the public exponent about 250,000. Entries whose vectors
do not fit together put each in its own complete file (X25519 has one vector
per file; HPKE takes the encapsulated key from its vector and reproduces its
derivation in the X25519 entry), and ML-KEM-512 is written whole but
evaluated in parts. S3g lowers the cost of an update to 1 + n/64 steps and
adds `orangec eval --steps N`, after which the split files can be folded back
together.

**There are no imports.** Each source is self-contained, so HPKE carries its
own SHA-256, HMAC, HKDF, X25519, ChaCha20 and Poly1305, and AES-GCM its own
AES. The multi-file programs planned for S3h let an entry call another's
specs instead.

## Sources reachable from the build

The READMEs cite standards by their canonical locations (the RFC Editor,
NIST's DOIs). Test vectors were taken from the standards where they publish
them, and otherwise from the designers' reference sets, the Botan and
Wycheproof vector files, or the IETF's machine-readable vector files, as each
vectors table states.
