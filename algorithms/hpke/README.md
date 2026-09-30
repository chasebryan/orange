# HPKE (RFC 9180)

Hybrid Public Key Encryption is public-key encryption built from three
parts: a key encapsulation mechanism that makes a fresh shared secret and its
encapsulation under the recipient's public key, a key derivation function that
turns the shared secret into an AEAD key, a base nonce and an exporter secret,
and an AEAD that encrypts any number of messages under that key. Richard
Barnes, Karthikeyan Bhargavan, Benjamin Lipp and Christopher Wood designed it
in the IRTF Crypto Forum Research Group between 2019 and 2022, and
[RFC 9180](https://www.rfc-editor.org/rfc/rfc9180), "Hybrid Public Key
Encryption" (February 2022), standardizes it: the KDF and the KEM interface in
section 4, DHKEM in section 4.1, the key schedule in section 5.1, the
context's `Seal` and `Open` in section 5.2, `Export` in section 5.3, the
algorithm identifiers in section 7, and the test vectors in Appendix A. This
entry is the base mode with the ciphersuite of Appendix A.2:
DHKEM(X25519, HKDF-SHA256) (kem_id 0x0020), HKDF-SHA256 (kdf_id 0x0001) and
ChaCha20-Poly1305 (aead_id 0x0003). HPKE is the encryption of TLS Encrypted
Client Hello, of the Messaging Layer Security protocol (RFC 9420), of
Oblivious HTTP (RFC 9458) and Oblivious DNS over HTTPS (RFC 9230), and of the
rate-limited token issuance drafted for Privacy Pass. It is a current
standard.

## Analysis

### Structure

The RFC composes its three parts through one small function pair,
`LabeledExtract` and `LabeledExpand` (section 4), which are HKDF's `Extract`
and `Expand` (RFC 5869) with every input prefixed by the string `"HPKE-v1"`, a
`suite_id` and a label. The Orange file is ordered as the RFC is: the
primitives it needs, then section 4, then 5.1, 5.2 and 5.3.

**The primitives.** Orange has no imports, so each file carries SHA-256
(FIPS 180-4, as the repository's S3e fixture writes it, with `hash_blocks`
generalizing the fixture's fixed blocks to a message of any length up to 119
bytes after the 64-byte HMAC key block, which covers every message HPKE hashes
with this suite), HMAC-SHA256 (RFC 2104, `key_block`, `outer_block`,
`hmac_sha256`), HKDF-SHA256 (RFC 5869, `extract` and `expand`), X25519
(RFC 7748, as the S3f fixture and the `x25519` entry write it: field
arithmetic in `Int`, a Montgomery ladder whose conditional swap is a
conditional), and ChaCha20-Poly1305 (RFC 8439, as the S3f AEAD fixture writes
it, sized to the 29-byte plaintext and the 7- or 9-byte `aad` of Appendix A.2).

**Section 4, the labeled KDF.** `LabeledExtract(salt, label, ikm)` is
`Extract(salt, "HPKE-v1" || suite_id || label || ikm)` and
`LabeledExpand(prk, label, info, L)` is
`Expand(prk, I2OSP(L, 2) || "HPKE-v1" || suite_id || label || info, L)`.
Orange has no strings and no variable-length arrays, so each concatenation
the RFC forms is its own spec, a 128-byte buffer whose used length is passed
beside it at the call: `labeled_ikm_eae_prk` and `labeled_info_shared_secret`
for the KEM, `labeled_ikm_dkp_prk` and `labeled_info_sk` for `DeriveKeyPair`,
`labeled_ikm_psk_id_hash`, `labeled_ikm_info_hash`, `labeled_ikm_secret`,
`labeled_info_key`, `labeled_info_base_nonce` and `labeled_info_exp` for the
key schedule, and `labeled_info_sec` for `Export`. Every label's ASCII stands
in a comment beside its bytes. The KEM's `suite_id` is `"KEM" || 0x0020` and
the rest of HPKE's is `"HPKE" || 0x0020 || 0x0001 || 0x0003`. Because every
`L` this suite asks for is at most `Nh = 32`, `expand` is HKDF's `T(1)` alone.

**Section 4.1, DHKEM.** `dh` is X25519; `kem_context` is `enc || pkRm`;
`extract_and_expand` is `LabeledExtract("", "eae_prk", dh)` followed by
`LabeledExpand(eae_prk, "shared_secret", kem_context, Nsecret)`; and `encap`
is the RFC's `Encap(pkR)` with the ephemeral pair `(skE, pkE)` as inputs
rather than drawn from `GenerateKeyPair`, since a specification takes its
randomness as an argument. `SerializePublicKey` is the identity for X25519
(section 7.1.1), so `enc = pkE`. Section 7.1.3's `DeriveKeyPair` for X25519 is
`derive_key_pair_sk` (`dkp_prk = LabeledExtract("", "dkp_prk", ikm)`,
`sk = LabeledExpand(dkp_prk, "sk", "", Nsk)`) and `pk`, which is
`X25519(sk, 9)`; there are no tuples, so the pair is two specs.

**Section 5.1, the key schedule.** For `mode_base`, `psk` and `psk_id` are
empty: `key_schedule_context_base` forms `mode || psk_id_hash || info_hash`
(65 bytes) from the two `LabeledExtract`s, `secret` is
`LabeledExtract(shared_secret, "secret", "")`, and `key`, `base_nonce` and
`exporter_secret` are the three `LabeledExpand`s with `Nk = 32`, `Nn = 12`
and `Nh = 32`. `key_schedule_base` returns the context
`key || base_nonce || exporter_secret` as 76 bytes, again because there are
no records.

**Sections 5.2 and 5.3, the context.** `compute_nonce` is
`xor(base_nonce, I2OSP(seq, Nn))`; `context_seal` is `ContextS.Seal(aad, pt)`
at a given sequence number, which calls the AEAD's `seal`; `export` is
`LabeledExpand(exporter_secret, "sec", exporter_context, L)`. The RFC's
context is stateful and `IncrementSeq` advances `seq` after each `Seal`;
Orange has no mutation, so `seq` is a parameter, and a `Seal` at `seq = n`
is the (n + 1)-th `Seal` of the context.

| RFC 9180 section | Orange spec |
| --- | --- |
| 4, `Extract`, `Expand` (HKDF, RFC 5869) | `extract`, `expand` over `hmac_sha256` |
| 4, `LabeledExtract`, `LabeledExpand` | the `labeled_ikm_*` and `labeled_info_*` buffers, one per use |
| 4.1, `DH` | `dh` (`x25519`) |
| 4.1, `ExtractAndExpand` | `extract_and_expand`, `kem_context` |
| 4.1, `Encap` | `encap` |
| 5.1, `psk_id_hash`, `info_hash`, `key_schedule_context` | `key_schedule_context_base`, `key_schedule_context` |
| 5.1, `secret`, `key`, `base_nonce`, `exporter_secret` | `secret`, `key`, `base_nonce`, `exporter_secret` |
| 5.1, `KeySchedule` (mode_base) | `key_schedule_base` |
| 5.2, `ComputeNonce` | `compute_nonce` |
| 5.2, `ContextS.Seal` | `context_seal`, over `seal` (RFC 8439) |
| 5.3, `Export` | `export`, `labeled_info_sec` |
| 7.1.3, `DeriveKeyPair` (X25519) | `derive_key_pair_sk`, `pk` |

### Security status

HPKE is a KEM/DEM hybrid in the sense of Cramer and Shoup (2001), who showed
such a composition IND-CCA2 secure when the KEM and the DEM are, and of
Herranz, Hofheinz and Kiltz (2006), who showed both conditions necessary.
What HPKE adds is the key schedule between the two: the KEM's shared secret
is not the AEAD key but passes through `LabeledExtract` and `LabeledExpand`,
with the mode, the hash of `psk_id` and the hash of `info` bound into
`key_schedule_context`, and with `enc || pkRm` bound into the KEM's own
derivation, so that the ciphertext is tied to every public key in play; the
RFC notes (section 9.1) that this removes the benign malleability of the
ECIES-family standards it replaces. The labeling gives domain separation
(section 9.6): the `suite_id` prefixes `"KEM..."` and `"HPKE..."` are
prefix-free, so the KDF calls inside DHKEM and in the rest of HPKE can be
modeled as independent functions even when they are one HKDF, and
`"HPKE-v1"` binds every derived secret to the scheme and its version. The RFC
defines four modes: `mode_base` (0x00), where only the recipient holds a key;
`mode_psk` (0x01), which mixes a pre-shared key into `secret`; `mode_auth`
(0x02), where the sender's static key pair enters the KEM through
`AuthEncap`; and `mode_auth_psk` (0x03), both. This entry writes and
evaluates the base mode only; the authenticated modes and the PSK mode
change `AuthEncap`/`AuthDecap` and the `secret` and `key_schedule_context`
inputs and are not written here.

Two analyses stand behind the RFC and it cites both. Lipp's "An Analysis of
Hybrid Public Key Encryption" (2020) is a CryptoVerif model of all four modes
that gives asymptotic message secrecy and export-key secrecy, and sender
authentication for the three authenticated modes, when the KEM is DHKEM in
a group where the gap Diffie-Hellman problem is hard, `Extract` is a random
oracle, `Expand` is a PRF and the AEAD is IND-CPA and INT-CTXT secure, for a
sender that sends one message and exports two secrets. Alwen, Blanchet,
Hauck, Kiltz, Lipp and Riepel, "Analysing the HPKE Standard" (2020;
Eurocrypt 2021), give exact bounds for DHKEM's authenticated interface and
composition theorems, mechanized in CryptoVerif, for the single-shot API of
the auth mode, in the outsider/insider vocabulary of signcryption: it is
Outsider-CCA, Outsider-Auth and Insider-CCA secure, and it is not
Insider-Auth secure, that is, the auth mode is subject to key-compromise
impersonation (section 9.1.1): whoever holds the recipient's private key can
forge messages to that recipient from any sender. The composition theorems
make no random-oracle assumption and so carry over to a post-quantum KEM;
every analysis is classical otherwise, and DHKEM itself falls to Shor's
algorithm. IETF work on post-quantum and hybrid KEMs for HPKE was in
progress when this was written and was not checked from this machine.

The nonce is the point where an application can go wrong. The context's
`key` and `base_nonce` are fixed for its lifetime, and the nonce of message
`seq` is `base_nonce` xor `I2OSP(seq, 12)`, so the nonces of one context
are distinct as long as `seq` does not repeat; `IncrementSeq` must fail at
2^96 - 1 (`MessageLimitReachedError`), the sender's context must never
decrypt nor the recipient's encrypt (section 5.2), and the two sides must
keep their counters in step, because ChaCha20-Poly1305 under a repeated
key-nonce pair leaks the xor of the plaintexts and the Poly1305 key, so
forgeries follow. Across contexts, freshness rests on the KEM's randomness:
section 9.7.5 says that bad ephemeral randomness in base mode can lose
confidentiality completely, and that a repeated KEM shared secret repeats
`key` and `base_nonce` and so the AEAD's key-nonce pairs. The vectors show
the xor: with `base_nonce` ending `...547f`, the nonce at `seq = 255` ends
`...5480` and at `seq = 256` ends `...557f`. The other non-goals of section
9.7 are the application's: no forward secrecy against compromise of the
recipient's key (`skR` decrypts every past message to it, in base and auth
modes), no ordering, replay or downgrade protection, no hiding of the
plaintext length, and no sender authentication at all in base mode. For
X25519, section 7.1.4 requires the recipient to reject an all-zero
Diffie-Hellman output; this entry computes the sender's side and does not
write that check.

Status as of September 2026: RFC 9180 (February 2022) is the current
specification, published as an IRTF CFRG informational RFC with IANA
registries for its KEM, KDF and AEAD identifiers. It is the encryption inside
TLS Encrypted Client Hello (`draft-ietf-tls-esni`; its publication status was
not checked from this machine), MLS (RFC 9420, 2023), Oblivious HTTP
(RFC 9458, 2024), Oblivious DoH (RFC 9230, 2022) and the rate-limited
Privacy Pass issuance draft. Its components are current standards too:
X25519 (RFC 7748), HKDF (RFC 5869) and ChaCha20-Poly1305 (RFC 8439), each
with its own entry or fixture in this repository.

### What the Orange rendering shows

Almost nothing in HPKE depends on data. The only data-dependent choice in the
four files is the conditional swap of the X25519 ladder, a conditional on a
scalar bit, exactly as in the `x25519` entry; every KDF call, every
concatenation and the whole key schedule are straight-line. The RFC's
variable-length inputs, `ikm`, `info`, `exporter_context` and the labeled
concatenations, become fixed 128-byte buffers with their lengths written as
literals at the calls (`51`, `91`, `28`, `46`, `23`, `87`, `94`, `22 + len`),
and SHA-256's padding is computed by `hash_blocks` from that length with
comparisons rather than assumed as in the fixtures, so the file shows that
the message lengths of this suite are fixed by the suite and the inputs, and
which they are. `expand` is HKDF's first block only; the general
`T(1) || T(2) || ...` loop is not written because no `L` here exceeds 32,
and the file says so. Poly1305 and X25519 are `Int` arithmetic reduced by
`%`, as the fixtures write them. The context is a 76-byte array and the
sequence number a parameter, because there are no records and no mutation:
the stateful part of the RFC, `IncrementSeq` and the message limit, is
described in a comment and not in code. `Encap` takes `skE` and `pkE` as
inputs and `DeriveKeyPair` returns its two values through two specs.

The evaluation cost was measured with a filler spec sharing each file's
budget (`budget.py` and `budget2.py` in the scratch directory; 67 steps per
filler iteration, so the figures are about that precise). One SHA-256
compression costs about 12,100 steps; one HMAC-SHA256 of a 51-byte text
(four compressions) about 51,700 and of a 91-byte text (five) about 66,100;
one X25519 about 621,000; `extract_and_expand` about 117,600, so `encap`,
one X25519 and two HMACs, about 737,700; `derive_key_pair_sk` about
103,200; each of `key`, `base_nonce` and `exporter_secret` from
`shared_secret`, which recomputes `psk_id_hash`, `info_hash` and `secret`
because parameterless specs share nothing, about 218,600; the whole
`key_schedule_base` about 350,200; the AEAD's `seal` about 39,100 and
`context_seal` about 44,000; one `export` about 55,700. Against the budget of
1,048,576 steps per file: `hpke.or` uses about 840,200 (`shared_secret` and
`skRm`) and has 208,000 left, less than the 218,600 of the smallest
key-schedule pair, which is why the key schedule and the seal are in
`hpke-key-schedule.or` with `shared_secret` as a literal, as the brief's
contingency foresaw; `hpke-key-schedule.or` uses about 1,045,800 and has
2,800 left; `hpke-encapsulated-key.or` uses about 792,400;
`hpke-context.or` about 420,500. `Decap` and `Open` are not written: `Decap`
is `extract_and_expand(dh(skR, enc), kem_context(enc, pkRm))`, a second
X25519 that no file could evaluate, and `Open` is `Seal` with the tag
compared, outside the scope of this entry.

## Dissemination

### Files

- `hpke.or`: the whole algorithm, from SHA-256 through `export`, and the
  vectors that need X25519 once: the KEM shared secret of `Encap` from
  `skEm`, `pkEm` and `pkRm`, and `skRm` from `ikmR` by `DeriveKeyPair`.
- `hpke-encapsulated-key.or`: the KEM part only (through `DeriveKeyPair`),
  and the derivation of the ephemeral pair from `ikmE`: `skEm`, and
  `pkEm = X25519(skEm, 9)`, which is the encapsulated key `enc` that
  `hpke.or` takes as a literal.
- `hpke-key-schedule.or`: the parts after the KEM (no X25519), and the key
  schedule from `shared_secret` as a literal: `key`, `base_nonce`,
  `exporter_secret`, and the sequence-number-0 encryption through
  `key_schedule_base` and `context_seal`.
- `hpke-context.or`: the context's operations only (no X25519, no key
  schedule), and, from the context as a literal, the six encryptions and
  the three exports of Appendix A.2.

The four files are one file split by the step budget. Each is a subset of
the same sections, assembled from one text by `build.py` in the scratch
directory, so the algorithm text two files share is identical byte for
byte; only the header and the vector specs differ. A file keeps each of its
sections whole, so a few specs of a section are not reached by that file's
vectors: `hpke.or` writes the whole algorithm and evaluates the part its
budget holds (`key_schedule_base`, `context_seal` and `export` are evaluated
in the other files), `hpke-encapsulated-key.or` carries `encap` beside the
`DeriveKeyPair` it evaluates, `hpke-key-schedule.or` carries `export`, and
`hpke-context.or` carries HKDF's `extract` beside the `expand` it uses.

### Running

```console
orangec eval algorithms/hpke/hpke.or
orangec eval algorithms/hpke/hpke-encapsulated-key.or
orangec eval algorithms/hpke/hpke-key-schedule.or
orangec eval algorithms/hpke/hpke-context.or
python3 algorithms/verify.py algorithms/hpke
```

`eval` also prints the input constants of Appendix A.2 (`a2_skem`,
`a2_pkem`, `a2_pkrm`, `a2_ikmr`, `a2_ikme`, `a2_shared_secret`, `a2_info`,
`a2_context`, `a2_pt`, the `a2_aad_*`) and the constant specs of the
algorithm (`round_constants`, `initial_hash`, `empty_salt`, `prime_25519`,
`prime_1305`, `base_point`, the parameterless `labeled_*` buffers), which
have no `_expected` twin and are not vectors.

### Vectors

All rows are RFC 9180 Appendix A.2, "DHKEM(X25519, HKDF-SHA256), HKDF-SHA256,
ChaCha20Poly1305", "Base Setup Information": `mode` 0, `kem_id` 32, `kdf_id`
1, `aead_id` 3, `info` "Ode on a Grecian Urn", plaintext "Beauty is truth,
truth beauty", `aad` "Count-n".

| Spec | Source | Case |
| --- | --- | --- |
| `rfc9180_a2_shared_secret` (`hpke.or`) | A.2 Base Setup, `shared_secret` | `Encap` from `skEm` f4ec9b33..., `pkEm` (= `enc`) 1afa08d3..., `pkRm` 4310ee97...; 0bbe7849... |
| `rfc9180_a2_sk_rm` (`hpke.or`) | A.2 Base Setup, `skRm` | `DeriveKeyPair(ikmR)`, `ikmR` 1ac01f18...; 8057991e... |
| `rfc9180_a2_sk_em` (`hpke-encapsulated-key.or`) | A.2 Base Setup, `skEm` | `DeriveKeyPair(ikmE)`, `ikmE` 909a9b35...; f4ec9b33... |
| `rfc9180_a2_pk_em` (`hpke-encapsulated-key.or`) | A.2 Base Setup, `pkEm` | `pk(skEm)` = X25519(`skEm`, 9); 1afa08d3... |
| `rfc9180_a2_key` (`hpke-key-schedule.or`) | A.2 Base Setup, `key` | key schedule from `shared_secret`; ad2744de... |
| `rfc9180_a2_base_nonce` (`hpke-key-schedule.or`) | A.2 Base Setup, `base_nonce` | 5c4d98150661b848853b547f |
| `rfc9180_a2_exporter_secret` (`hpke-key-schedule.or`) | A.2 Base Setup, `exporter_secret` | a3b010d4... |
| `rfc9180_a2_encryption_0` (`hpke-key-schedule.or`) | A.2 Encryptions, sequence number 0 | through `key_schedule_base`; `aad` "Count-0"; `ct` 1c5250d8... |
| `rfc9180_a2_encryption_0` (`hpke-context.or`) | A.2 Encryptions, sequence number 0 | from the context as a literal; nonce ...547f; `ct` 1c5250d8... |
| `rfc9180_a2_encryption_1` (`hpke-context.or`) | A.2 Encryptions, sequence number 1 | nonce ...547e; `ct` 6b53c051... |
| `rfc9180_a2_encryption_2` (`hpke-context.or`) | A.2 Encryptions, sequence number 2 | nonce ...547d; `ct` 71146bd6... |
| `rfc9180_a2_encryption_4` (`hpke-context.or`) | A.2 Encryptions, sequence number 4 | nonce ...547b; `ct` 63357a2a... |
| `rfc9180_a2_encryption_255` (`hpke-context.or`) | A.2 Encryptions, sequence number 255 | `aad` "Count-255"; nonce ...5480; `ct` 18ab939d... |
| `rfc9180_a2_encryption_256` (`hpke-context.or`) | A.2 Encryptions, sequence number 256 | `aad` "Count-256"; nonce ...557f; `ct` 7a4a13e9... |
| `rfc9180_a2_export_empty_context` (`hpke-context.or`) | A.2 Exported Values, first | `exporter_context` "", L = 32; 4bbd6243... |
| `rfc9180_a2_export_zero_context` (`hpke-context.or`) | A.2 Exported Values, second | `exporter_context` 0x00, L = 32; 8c1df147... |
| `rfc9180_a2_export_test_context` (`hpke-context.or`) | A.2 Exported Values, third | `exporter_context` "TestContext", L = 32; 5acb0921... |

Every expected value is the published value, taken from the two copies
named below, which agree. The `cryptography` package (`X25519PrivateKey`,
`HKDF`, `ChaCha20Poly1305`) and `hmac`/`hashlib` (HMAC-SHA256) confirmed
every row through the Python reference described next; no row's expected
value comes from an oracle alone.

### Provenance and claims

The RFC is unreachable from the build machine, so its values came from the
CFRG's own repository: the machine-readable
`cfrg/draft-irtf-cfrg-hpke/master/test-vectors.json` (SHA-256
`61fc662f01996cd06d713dacf5e133167bd309a1f329442d53f1e21a47b3ede6`,
re-fetched and found unchanged), from which the one case with `mode` 0,
`kem_id` 32, `kdf_id` 1 and `aead_id` 3 was selected (`a2_base.json` in the
scratch directory), and the appendix text of `draft-irtf-cfrg-hpke.md` in the
same repository, whose "Base Setup Information", "Encryptions" (sequence
numbers 0, 1, 2, 4, 255, 256) and "Exported Values" for this suite agree with
the JSON field for field (`check_appendix.py`). A Python reference of RFC 9180
base mode (`oracle.py` in the scratch directory: HKDF over `hmac`/`hashlib`,
the labeled functions, DHKEM, the key schedule, `Seal` and `Export`)
reproduces `dh`, `eae_prk`, `shared_secret`, `psk_id_hash`, `info_hash`,
`key_schedule_context`, `secret`, `key`, `base_nonce`, `exporter_secret`, all
257 encryptions and the three exports of the case, and agrees with the
`cryptography` package on X25519 (both directions of the exchange and both
public keys), HKDF and ChaCha20-Poly1305; the intermediates it saved
(`intermediates.json`) were the values compared against when an Orange value
was being debugged. `DeriveKeyPair` was checked against both `ikmE`/`skEm`
and `ikmR`/`skRm`.

The SHA-256 round constants and initial hash were recomputed from the cube
and square roots of the first primes, the ChaCha20 constants decoded back to
"expand 32-byte k", the Poly1305 clamp and prime, 2^255 - 19, a24 and the
base point compared with Python, and every labeled byte string in the
sources decoded back to the ASCII its comment names, with the stated
concatenation lengths recomputed (`check_constants.py`). The byte literals
of the four files were rendered from the fetched JSON and the labels' ASCII
by `build.py`, never typed by hand; the repository files are byte-identical
to its output.

This entry is a reference evaluation of RFC 9180's base mode under
`orangec eval`. It makes no constant-time, side-channel, performance or
certification claim; the X25519 swap is a specification of a choice, not a
constant-time swap, and the stateful nonce discipline of section 5.2 is
described, not enforced. It is not a corpus entry in the sense of The Orange
Book chapter 12.

## Gaps

- The step budget of 1,048,576 steps per file holds one X25519 (about
  621,000 steps) and little more, and parameterless specs share no
  computation, so the entry is four files with an identical algorithm part
  rather than one: `hpke.or` cannot carry even the smallest key-schedule
  pair beyond `shared_secret`, the key-schedule file is full to within 2,800
  steps, `pkEm` is derived in a file of its own, and `Decap`, which would be
  a second X25519 beside `Encap`, is not written.
- There are no strings and no variable-length arrays, so `LabeledExtract`
  and `LabeledExpand` cannot be two specs taking a label and an input; the
  eleven concatenations the RFC forms for this mode are eleven specs, each a
  128-byte buffer with its used length passed as an `Int`, and SHA-256's
  padding is computed from that length at every hash.
- There is no mutation, so the context's sequence number is a parameter of
  `context_seal` and `IncrementSeq` with its `MessageLimitReachedError`
  is a comment; there are no records or tuples, so the context is a 76-byte
  array with three accessor specs, `Encap` takes `pkE` as an input, and
  `DeriveKeyPair` is two specs.
- Loop bounds are literals and there is no data-dependent index, so
  `hash_blocks` always runs its two-block loop and skips the second block
  with a comparison, and `expand` covers HKDF's first output block only,
  which is all this suite uses.
