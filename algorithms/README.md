# Analysis and Dissemination

This folder holds the major encryption algorithms written in Orange, one
folder per algorithm, each written from its standard and checked against the
vectors the standard publishes. Explicitly labeled mathematical
representation definitions also carry hand-derived boundary answers. The
title names the two things an entry is for. *Analysis*: the algorithm is laid
out the way its designers laid it out, so that a cryptographer, cryptologist
or cryptanalyst can read the structure, follow every constant back to its
source, and see plainly which steps depend on data. *Dissemination*: the same
text is a program that `orangec test` runs, so a reader can reproduce the
standard's own examples, change an input, and watch what happens, with
nothing between the specification and the result.

Every entry is a reference evaluation of a specification. None of them is a
corpus entry in the sense of [The Orange Book](../docs/THE_ORANGE_BOOK.md),
chapter 12: no entry makes a constant-time, side-channel, performance,
interoperability or certification claim, and the pure Orange 2026 extensions
they use are proposed, not accepted. What an entry does claim is narrow and
checked by machine. A claim is a test whose expected value is the published
one, taken from the sources the README names, and the claim holds when the
entry reproduces that value byte for byte. Mathematical boundary answers are
identified separately and do not become published cryptographic vectors or
refinement proofs.

## The entries

The folder holds twenty-two entries and 268 published cryptographic vectors.
Twenty of the entries state their vectors as tests, 244 in all. Two keep the
older form, a pair of specs for each vector: ChaCha20 (20 pairs), which is
waiting for a rewrite of its own, and X25519 (4 pairs), which is maintained
separately. The X25519 entry also holds 21 mathematical answer pairs (seven
P2 representation pairs and fourteen partial P4 field operation pairs) and 15
tests about the limb representation, all in `field25519-limbs.or`. The
expected values of those 21 pairs are hand-derived; neither they nor the 15
tests are published vectors, and none is counted among the 268.
`python3 algorithms/verify.py` over everything prints
`259 tests passed, 45 vector pairs reproduced, 0 findings`.

Each row links the entry's README. Its vector count is the number of its
tests, or of its published pairs for ChaCha20 and X25519. Where a standard
publishes no example, the value comes from the oracle that the README's
vectors table names, and a few tests state a value derived from a published
one, such as a rejected tag. The X25519 source count also includes the limb
source. Standing is the entry's own summary of the record as of October 2026.

### Block ciphers

| Entry | Standard | Sources | Vectors | Standing |
| --- | --- | --- | ---: | --- |
| [AES](aes/README.md) | FIPS 197; SP 800-38A modes | 2 | 19 | Current; no practical attack on the full cipher |
| [Camellia](camellia/README.md) | RFC 3713 | 1 | 8 | Current; unbroken |
| [ARIA](aria/README.md) | KS X 1213; RFC 5794 | 1 | 8 | Current; unbroken |
| [SM4](sm4/README.md) | GB/T 32907-2016 | 1 | 6 | Current; unbroken |
| [Serpent](serpent/README.md) | AES submission (1998) | 1 | 12 | AES finalist; unbroken |
| [Twofish](twofish/README.md) | AES submission (1998) | 1 | 7 | AES finalist; unbroken |
| [RC6](rc6/README.md) | AES submission, v1.1 (1998) | 1 | 12 | AES finalist; unbroken |
| [DES and Triple DES](des/README.md) | FIPS 46-3; SP 800-67 rev. 2 | 1 | 13 | DES withdrawn and broken by key search; TDEA encryption disallowed after 2023 |

### Authenticated encryption

| Entry | Standard | Sources | Vectors | Standing |
| --- | --- | --- | ---: | --- |
| [AES-GCM](aes-gcm/README.md) | SP 800-38D | 2 | 10 | Current |
| [AES-CCM](aes-ccm/README.md) | SP 800-38C; RFC 3610 | 2 | 6 | Current |
| [ChaCha20-Poly1305 and XChaCha20-Poly1305](chacha20-poly1305/README.md) | RFC 8439; draft-irtf-cfrg-xchacha | 1 | 22 | Current |
| [Ascon-AEAD128 and Ascon-Hash256](ascon/README.md) | SP 800-232 (2025) | 3 | 18 | Current |

### Stream ciphers

| Entry | Standard | Sources | Vectors | Standing |
| --- | --- | --- | ---: | --- |
| [ChaCha20 and XChaCha20](chacha20/README.md) | RFC 8439; draft-irtf-cfrg-xchacha | 1 | 20 | Current |
| [Salsa20 and XSalsa20](salsa20/README.md) | Bernstein's specification; eSTREAM | 1 | 19 | Unbroken; Salsa20/12 in the eSTREAM portfolio |
| [RC4](rc4/README.md) | None; RFC 6229 vectors | 1 | 5 | Broken; prohibited in TLS by RFC 7465 |

### Hash functions and key derivation

| Entry | Standard | Sources | Vectors | Standing |
| --- | --- | --- | ---: | --- |
| [SHA-256 and SHA-512](sha2/README.md) | FIPS 180-4 | 1 | 13 | Current |
| [SHA-3 and SHAKE](sha3/README.md) | FIPS 202 | 1 | 10 | Current |
| [HMAC-SHA-256 and HKDF](hmac-hkdf/README.md) | FIPS 198-1; RFC 4231; RFC 5869 | 2 | 16 | Current |

### Public-key

| Entry | Standard | Sources | Vectors | Standing |
| --- | --- | --- | ---: | --- |
| [X25519](x25519/README.md) | RFC 7748; separate mathematical limb definitions | 5 | 4 | Current |
| [HPKE](hpke/README.md) | RFC 9180 | 5 | 17 | Current |
| [RSAES-OAEP](rsa-oaep/README.md) | RFC 8017 (PKCS #1 v2.2); FIPS 180-4 | 3 | 11 | Current at 2048 bits and above; 1024-bit keys disallowed |
| [ML-KEM-512](ml-kem/README.md) | FIPS 203 (2024); FIPS 202 | 2 | 12 | Current; no known attack below its security category |

## How an entry is built

Each folder holds a `README.md` and one or more `.or` sources. The README has
the same shape everywhere: *Analysis* (the structure of the algorithm mapped
to the standard's sections and the file's specs; the cryptanalytic record a
reader should know; what the Orange rendering shows and does not express),
*Dissemination* (Files; Running; Vectors, a table of Test, Source and Case;
Provenance and claims) and *Gaps* (what the language or the evaluator could
not express or evaluate, precisely).

A source is one Orange module. An entry is a root source, the file whose
tests state its vectors, sometimes with modules in the same folder that the
root reads with `use`: `aes.or` for the modes that need AES, `sha256.or` for
the HMAC that needs SHA-256, `keccak.or` for ML-KEM-512. A module that holds
no tests of its own is accepted because a sibling uses it. Most entries are
one root. AES (the cipher and its modes) and Ascon (the AEAD and the hash,
over one permutation module) have two, and X25519, which keeps the older
form, has four vector files and a limb file. A source's header comment names
the standard, the sections transcribed and the vectors reproduced; every
section of the file carries a comment naming the section of the standard it
follows, and every departure from the standard's text is explained where it
happens. Specs and parameters use the standard's own names, and byte orders
are written where the standard fixes them, as `as big` and `as little`
conversions between bytes and words. The two entries that keep the older form
still use helper specs of their own for this (`load_le32` and `le_bytes` in
ChaCha20, `decode_little_endian` in X25519).

### Vectors

A reproduced vector is a test. A source states one as
`test "TITLE" { claim }` ([`TESTS_2026.md`](../docs/TESTS_2026.md)): the title
cites the document, the section or the case that publishes the value, and the
claim is that what the algorithm computes is that value, written as a
literal; a rejected tag is the claim that the verdict is false. `orangec test`
runs every test of a source under one evaluation budget and reports each:

```console
$ orangec test algorithms/sm4/sm4.or
test "GB/T 32907-2016 Appendix A, example 1: encrypts" ... ok
test "GB/T 32907-2016 Appendix A, example 1: decrypts" ... ok
test "Botan sm4.vec case 2, block 1: encrypts" ... ok
test "Botan sm4.vec case 4: encrypts" ... ok
test "Botan sm4.vec case 4: decrypts" ... ok
test "Botan sm4.vec case 5, block 1: encrypts" ... ok
6 tests: 6 passed, 0 failed
```

A test that fails prints both values and the first position at which they
differ. The README's vectors table lists the same tests with the source of
each expected value.

ChaCha20 and X25519 still state their vectors as pairs of parameterless
specs. `<name>` computes a value with the algorithm and `<name>_expected`
states the value published with the source, as a literal; `orangec eval`
prints both, and the check requires them to agree in type and value. The same
pair convention records the seven P2 representation boundaries and the
fourteen partial P4 mathematical field-operation boundaries of
[`field25519-limbs.or`](x25519/field25519-limbs.or). Those expected values
are hand-derived from the documented radix, prime and a24, rather than
imported from a standard's cryptographic vector corpus, and the file's 15
tests are claims about the same representation, not published vectors.

Two checks apply the rules, and require every source to state at least one
vector or to serve a root that does:

```sh
python3 algorithms/verify.py            # every entry, or name a folder
cargo test --manifest-path compiler/Cargo.toml -p orangec --test algorithms
```

Both run `orangec check` on every source, which must pass without
diagnostics. A source with test blocks runs `orangec test --steps 1073741824`;
a source with `_expected` specs has its pairs evaluated with `orangec eval`
and compared; a source with neither is accepted only when a sibling in the
same folder `use`s it, as the roots use `aes.or`, `sha256.or` and
`keccak.or`. `verify.py` also requires each folder's README to carry
*Analysis* and *Dissemination* sections. The cargo test runs in the repository
gate on every source, in parallel threads, so a change to the compiler that
alters any recorded value fails CI.

The default budget of `orangec test` is 1,048,576 evaluation steps per
program ([`EXPRESSIONS_2026.md`](../docs/EXPRESSIONS_2026.md), section 14), so
the entries whose tests need more run with `--steps 1073741824`, the largest
budget `orangec` admits, as the gate does. Each entry keeps its tests under
about 25 million steps, because the cargo test also runs in a debug build,
about nine times slower than a release build.

An expected value is never adjusted to match a computation: each README's
vectors table says where every value came from, and, when a standard
publishes no example, which oracle produced it (`hashlib`, the `cryptography`
package, `pycryptodome`, or a plain Python reference written for the entry
and checked against a published set).

## What the entries show about Orange 2026

Eighteen of the entries were first written in an earlier form of the language
and rewritten in the current one, and two (RSAES-OAEP and ML-KEM-512) were
written in it. The *Gaps* section of each entry lists what the language still
cannot say. These are the limits that recur, with the facts the entries
record.

**Indices follow the standard.** A table is read at the index the standard
writes, a data byte indexing the table: `sbox[x]` in AES, `s[s[i] + s[j]]` in
RC4, `sbox1()[x[0]]` in Camellia, and likewise in SM4 and ARIA. The checker
proves an index in range from its type before anything runs
([`LOOKUPS_2026.md`](../docs/LOOKUPS_2026.md)), so a `Word[8]` indexes an
array of 256 entries with no mask, and a lookup costs a few evaluation steps
once the table exists. The proof reads syntax only, so a narrower index must
say that it is narrow: a value bound by `let` ranges over its whole type, and
Twofish reduces each nibble modulo 16 and DES masks its row and column with
`& 3` and `& 15`, though neither changes a value. A rotation amount computed
from data needs no mask, because a computed rotation already turns modulo the
word width (RC6).

**A table is built at every call.** There are no module-level constants, and
a parameterless spec is evaluated again each time it is called. Camellia's
SBOX1 takes 58 steps to build and every one of the eight lookups in an
F-function builds it again, about two thirds of the cost of a block; SM4 and
ARIA build theirs at every use as well. Camellia keeps the RFC's form rather
than pass the table to its callers, which would add a parameter the RFC does
not have.

**Modules reach only their own folder.** A source uses the modules beside it
([`MODULES_2026.md`](../docs/MODULES_2026.md)). AES-GCM and AES-CCM read
`aes.or`, the AES modes read the cipher, HMAC-HKDF reads `sha256.or`,
RSAES-OAEP reads `sha1.or` and `sha256.or`, ML-KEM-512 reads `keccak.or`,
Ascon's two roots share `permutation.or`, and HPKE reads `hkdf.or` (which
reads `sha256.or`), `curve25519.or` and `chacha20poly1305.or`. A module in
another folder is still out of reach, so an entry that needs another entry's
algorithm carries its own: HMAC-HKDF, HPKE and RSAES-OAEP each have a
`sha256.or`, AES-GCM and AES-CCM each an `aes.or`, and ML-KEM-512 has a
`keccak.or` of its own, apart from the SHA-3 entry.

**A function has at most 256 instances.** A size parameter makes one spec
stand for a family with one instance for each value of its sizes, and a type
parameter does the same for a list of types
([`SIZES_2026.md`](../docs/SIZES_2026.md),
[`TYPE_PARAMETERS_2026.md`](../docs/TYPE_PARAMETERS_2026.md)). A function has
at most 256 instances in all, counting every combination, so a length that
varies is a bounded range: the SHA-256 modules of HMAC-HKDF and HPKE hash 1
through 256 bytes. An AEAD with two independent lengths cannot range over
both, so AES-GCM, AES-CCM, Ascon and ChaCha20-Poly1305 list the lengths of
their vectors as type parameters, and a vector at another length needs its
length added to a list. ChaCha20-Poly1305 has two Poly1305 MAC specs for the
same reason: the tag's input does not fall among the instances of the first,
so the second runs the same loop over whole blocks. A size that no array
argument carries is written at the call, as `padding[3, 1]()` and
`keystream[32]`.

**An array is never empty.** Every array holds at least one element, so the
empty string is not a value, and empty associated data, an empty `info` or an
empty message needs a spec of its own. AES-CCM decrypts without associated
data in a second spec, HKDF has `hkdf_expand_no_info`, Ascon has
`encrypt_empty_a`, `encrypt_empty_p` and `encrypt_empty`, and the SHA entries
pad by returning only what is appended, for the test to join to its message.

**Types are fixed.** A `type` declaration cannot take a size, and an array of
arrays needs a named row type, so ML-KEM cannot write `Vector` and `Matrix`
for a parameter k; the entry is ML-KEM-512 only, with k = 2. Words are 8, 16,
32 or 64 bits wide, so ARIA and Camellia write a 128-bit value as two 64-bit
halves, DES keeps its 28-, 48- and 56-bit values in the low bits of a 64-bit
word and carries their widths by hand, and Twofish keeps a nibble in a byte.
A tuple holds no tuple, so the loop of RC4 carries its state as three
accumulators beside the output bytes. An array inside a tuple argument does
not choose a sized instance, so the decryption of AES-CCM takes the
ciphertext and the tag as two arguments rather than the pair its encryption
returns.

**Results have one type.** There is no sum type and no error value, so a
failed decryption is a value the caller must read. AES-GCM, AES-CCM,
ChaCha20-Poly1305 and Ascon return a verdict beside the plaintext, and the
caller carries the rule that the plaintext counts only when the verdict is
true; Ascon's `decrypt` returns zeros when the tag does not verify, and
RSAES-OAEP folds the RFC's errors into a `Bool`.

**Specs are not values.** No function takes a hash as a parameter and no
array holds functions, so RSAES-OAEP is written once for SHA-1 and once for
SHA-256, and Serpent picks the S-box of a round with an eight-arm conditional
on the round number modulo 8.

**A loop has a fixed count.** ML-KEM-512 squeezes a fixed four blocks of
SHAKE128 where the standard squeezes until it has 256 coefficients, and the
exponentiation of RSAES-OAEP loops over at most 2,048 exponent bits. The
bound is fixed when the file is checked, and no recorded vector needs more.

**The budget is per program.** All the tests of a source share one budget of
evaluation steps, 1,048,576 by default. AES, AES-GCM, DES, HMAC-HKDF, HPKE,
ML-KEM-512, RSAES-OAEP and Twofish need a larger one, RSAES-OAEP the most at
about 22.5 million steps, and what would not fit within about 25 million
stays out: the million-fold iteration of SM4's second example (about 3.6 x
10^9 steps, over the largest budget `orangec` admits), the iterated test of
RFC 7748 section 5.2, and the remaining ACVP cases of ML-KEM-512. The
rewritten entries no longer split one algorithm across files to fit the
budget; where an algorithm is reused, a module serves. X25519, which was not
rewritten, still holds one vector per file, because one X25519 evaluation
costs about 565,000 of the 1,048,576 steps of a source as of S3u, and two do
not fit.

**An array holds 1 through 65,536 elements.** A message of up to that length
fits one array ([`LENGTHS_2026.md`](../docs/LENGTHS_2026.md)), so the
rewritten entries do not split messages to fit one. ChaCha20, not yet
rewritten, still passes its two longest messages as two arrays, and Salsa20
states the long vectors of its sources as their first 64 to 192 bytes. The
one recorded case beyond an array is the 10-byte associated-data length
encoding of CCM, which is for `a >= 2^32` bytes.

## Sources reachable from the build

The READMEs cite standards by their canonical locations (the RFC Editor,
NIST's DOIs). Test vectors were taken from the standards where they publish
them, and otherwise from the designers' reference sets, the vector files of
Botan, OpenSSL, Wycheproof and other libraries, the IETF's machine-readable
vector files, NIST's ACVP machine-readable vectors (the ML-KEM-512 entry) or
RSA Laboratories' `oaep-vect.txt` (the RSAES-OAEP entry), as each vectors
table states. Where a standard's own text was not reachable from the build
machine, the README says which verbatim copy or library file supplied its
values and which oracle confirmed them.
