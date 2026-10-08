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
`suite_id` and a label. The entry is five modules: one for each primitive
the suite names, and the root `hpke.or`, which `use`s three of them (`hkdf`
`use`s the fourth, `sha256`) and is ordered as the RFC is: section 4, then
5.1, 5.2 and 5.3, with the identifiers of section 7 at the top.

**The primitives.** `sha256.or` is SHA-256 (FIPS 180-4) as the `sha2` entry
writes it, with a `padding[len]` and a `digest[len]` for every message of 1
through 256 bytes, which covers every message HPKE hashes with this suite.
`hkdf.or` uses it for HMAC-SHA256 (RFC 2104, `hmac[n]`, keys of 32 bytes and
texts of 1 through 192 bytes) and HKDF-SHA256 (RFC 5869, `extract[n]` and
`expand[n]`). `curve25519.or` is X25519 (RFC 7748), written from the
section 5 pseudocode over `Mod[2^255 - 19]`: `decode_u_coordinate`,
`decode_scalar`, `cswap`, `ladder_step` and `x25519`, with the ladder's swap
carried from bit to bit as the RFC carries it. `chacha20poly1305.or` is the
AEAD of RFC 8439 section 2.8 over the block function, the encryption,
Poly1305 over `Mod[2^130 - 5]` and the one-time key of sections 2.3 to 2.6,
for additional data of 7 through 9 bytes and plaintexts of 1 through 64
bytes; its `seal[a, n]` is RFC 9180's `Seal(key, nonce, aad, pt)`.

**Section 4, the labeled KDF.** `LabeledExtract(salt, label, ikm)` is
`Extract(salt, "HPKE-v1" || suite_id || label || ikm)` and
`LabeledExpand(prk, label, info, L)` is
`Expand(prk, I2OSP(L, 2) || "HPKE-v1" || suite_id || label || info, L)`.
Both are single specs, `labeled_extract` and `labeled_expand`, sized by the
length of their last argument and typed by the `suite_id`, which is the
KEM's `"KEM" || I2OSP(kem_id, 2)` (5 bytes, `kem_suite_id`) inside DHKEM and
`"HPKE" || I2OSP(kem_id, 2) || I2OSP(kdf_id, 2) || I2OSP(aead_id, 2)` (10
bytes, `hpke_suite_id`) elsewhere, both built from the identifiers of
section 7. The label and the `ikm` or `info` are one argument, written
`"eae_prk" ++ dh` at the call, because the RFC passes empty strings for
`psk_id`, `psk`, the `info` of `DeriveKeyPair` and an export context, and an
empty string is no array: those calls pass the label alone. `I2OSP(L, 2)` is
`l as big Word[8]^2`. Because every `L` this suite asks for is at most
`Nh = 32`, `expand` is HKDF's `T(1)` alone, and `base_nonce` keeps its first
`Nn = 12` bytes.

**Section 4.1, DHKEM.** `dh` is X25519; `pk` is section 3's `pk(skX)`,
`X25519(sk, 9)`;
`extract_and_expand` is `LabeledExtract("", "eae_prk", dh)` followed by
`LabeledExpand(eae_prk, "shared_secret", kem_context, Nsecret)`; and `encap`
is the RFC's `Encap(pkR)`, returning the pair `(shared_secret, enc)`, with
`kem_context = enc || pkRm` and with the ephemeral pair `(skE, pkE)` as
inputs rather than drawn from `GenerateKeyPair`, since a specification takes
its randomness as an argument. `SerializePublicKey` is the identity for
X25519 (section 7.1.1), so `enc = pkE`. Section 7.1.3's `DeriveKeyPair` for
X25519 is `derive_key_pair`, returning `(sk, pk(sk))` with
`dkp_prk = LabeledExtract("", "dkp_prk", ikm)` and
`sk = LabeledExpand(dkp_prk, "sk", "", Nsk)`.

**Section 5.1, the key schedule.** For `mode_base`, `psk` and `psk_id` are
empty: `key_schedule[n]` forms `psk_id_hash` and `info_hash` by
`LabeledExtract`, `key_schedule_context = mode || psk_id_hash || info_hash`
(65 bytes), `secret = LabeledExtract(shared_secret, "secret", "")`, and `key`,
`base_nonce` and `exporter_secret` by the three `LabeledExpand`s with
`Nk = 32`, `Nn = 12` and `Nh = 32`, for an `info` of 1 through 64 bytes. It
returns the RFC's `Context(key, base_nonce, 0, exporter_secret)` as a value
of the tuple type `Context`.

**Sections 5.2 and 5.3, the context.** `compute_nonce` is
`xor(base_nonce, I2OSP(seq, Nn))`, with `I2OSP` written
`seq as big Word[8]^12`; `seal[a, n]` is `ContextS.Seal(aad, pt)` on the
context's `seq`, which calls the AEAD's `seal`; `increment_seq` is
`IncrementSeq`, returning the context with `seq + 1`; `export[n]` is
`Export(exporter_context, L) = LabeledExpand(exporter_secret, "sec",
exporter_context, L)` and `export_empty` the same with the empty
`exporter_context`. The RFC's context is stateful and its `Seal` calls
`IncrementSeq`; Orange has no mutation and a tuple holds no tuple, so `seal`
returns the ciphertext alone and the context for the next `Seal` is
`increment_seq(context)`; a context whose `seq` is `n` makes the
(n + 1)-th `Seal`.

| RFC 9180 section | Orange spec |
| --- | --- |
| 3, `pk(skX)` | `pk` |
| 4, `Extract`, `Expand` (HKDF, RFC 5869) | `hkdf::extract[n]`, `hkdf::expand[n]` over `hkdf::hmac[n]` and `sha256::digest[len]` |
| 4, `LabeledExtract`, `LabeledExpand` | `labeled_extract`, `labeled_expand`, with `kem_suite_id` or `hpke_suite_id` |
| 4.1, `DH` | `dh` (`curve25519::x25519`) |
| 4.1, `ExtractAndExpand` | `extract_and_expand` |
| 4.1, `Encap` | `encap` |
| 5, `mode_base` | `mode_base` |
| 5.1, `KeySchedule` (mode_base) | `key_schedule[n]`, returning a `Context` |
| 5.2, `ComputeNonce` | `compute_nonce` |
| 5.2, `IncrementSeq` | `increment_seq` |
| 5.2, `ContextS.Seal` | `seal[a, n]`, over `chacha20poly1305::seal[a, n]` (RFC 8439) |
| 5.3, `Export` | `export[n]`, `export_empty` |
| 7, `kem_id`, `kdf_id`, `aead_id`, `Nsecret`, `Nsk`, `Nh`, `Nk`, `Nn` | `kem_id`, `kdf_id`, `aead_id`, `n_secret`, `n_sk`, `n_h`, `n_k`, `n_n` |
| 7.1.3, `DeriveKeyPair` (X25519) | `derive_key_pair` |

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
five files is `cswap` in the X25519 ladder, a conditional on the xor of two
scalar bits; every KDF call, every concatenation and the whole key schedule
are straight-line, and every index is a literal or a loop index, so the
checker proves every access in range before evaluation. The bits of the
clamped scalar are read as the RFC numbers them, bit `t % 8` of byte `t / 8`
for `t` from 254 down to 0.

The RFC's strings are Orange's: labels, `info`, the plaintext and the `aad`
are string literals (`"eae_prk"`, `"Ode on a Grecian Urn"`, `"Count-255"`),
keys and expected values are `hex"..."`, and every concatenation the RFC
writes with `concat` is `++`, so `"HPKE-v1" ++ suite_id ++ label_ikm` is the
text of section 4. Byte orders are written where the standards fix them:
`I2OSP` is `as big` (the identifiers, `L`, the sequence number), X25519's
coordinates and Poly1305's blocks and lengths are `as little`, and SHA-256
parses its blocks with `as big`. Lengths are sizes: the checker computes the
length of each concatenation from the lengths of its parts, picks the
instance of `labeled_extract`, `hkdf::extract`, `hkdf::hmac` and
`sha256::digest` that takes it, and so fixes, for this suite and these
inputs, which message lengths are hashed, without a length being written at
any call. The field arithmetic is `Mod[2^255 - 19]` for X25519 and
`Mod[2^130 - 5]` for Poly1305, so the formulas are the RFCs' with no
reduction written, and X25519's final `x_2 * z_2^(p - 2)` is one division.
Tuples carry what the RFC returns in pairs (`Encap`, `DeriveKeyPair`) and the
context (`key`, `base_nonce`, `seq`, `exporter_secret`). `Decap` and `Open`
are not written: `Decap` is `extract_and_expand(dh(skR, enc), enc || pkRm)`
and `Open` is `Seal` with the tag compared, outside the scope of this entry
as before.

Measured costs under `orangec test --stats`: one SHA-256 compression costs
8,934 steps; one HMAC-SHA256 of a 51-byte text (four compressions) 36,999
and of a 92-byte text (five) 45,937; one X25519 380,835, of which each of
the 255 ladder steps is about 1,480 (1,290 of it the ten multiplications
of `ladder_step`, at 129 each) and the final division about 4,100; `extract_and_expand`
82,979; a ChaCha20 block 4,324 and a Poly1305 of four blocks 485. The tests
cost 454,868 for each `DeriveKeyPair` (one X25519 for `pk(sk)` and two
HMACs), 463,875 for `Encap`, 248,916 for each test through the key schedule
(six HMACs), 258,547 for the encryption through the key schedule, about
9,640 for each `Seal` from the printed context, and about 37,040 for each
`Export`. The seventeen tests together use 3,002,767 steps.

## Dissemination

### Files

- `hpke.or`: RFC 9180 base mode for this suite, sections 4 through 5.3 and
  the identifiers of section 7, and the seventeen tests below.
- `sha256.or`: SHA-256 (FIPS 180-4) for messages of 1 through 256 bytes;
  module `sha256`, used by `hkdf.or`.
- `hkdf.or`: HMAC-SHA256 (RFC 2104) and HKDF-SHA256 (RFC 5869) `Extract`
  and the first block of `Expand`; module `hkdf`, used by `hpke.or`.
- `curve25519.or`: X25519 (RFC 7748) over `Mod[2^255 - 19]`; module
  `curve25519`, used by `hpke.or`.
- `chacha20poly1305.or`: the ChaCha20-Poly1305 AEAD (RFC 8439) for the
  lengths HPKE uses here; module `chacha20poly1305`, used by `hpke.or`.

The four modules have no tests of their own; the gate checks them as
modules of `hpke.or`, whose tests run through all of them.

### Running

```console
orangec test algorithms/hpke/hpke.or
python3 algorithms/verify.py algorithms/hpke
```

`orangec eval algorithms/hpke/hpke.or` prints the parameterless specs of the
root, the identifiers, the `N` constants, the two `suite_id`s and
`mode_base`, which are not vectors.

### Vectors

All rows are RFC 9180 Appendix A.2, "DHKEM(X25519, HKDF-SHA256), HKDF-SHA256,
ChaCha20Poly1305", "Base Setup Information": `mode` 0, `kem_id` 32, `kdf_id`
1, `aead_id` 3, `info` "Ode on a Grecian Urn", plaintext "Beauty is truth,
truth beauty", `aad` "Count-n". Every test is in `hpke.or`.

| Test | Source | Case |
| --- | --- | --- |
| `RFC 9180 A.2: DeriveKeyPair(ikmR) gives skRm` | A.2 Base Setup, `skRm` | `DeriveKeyPair(ikmR)`, `ikmR` 1ac01f18...; 8057991e... |
| `RFC 9180 A.2: DeriveKeyPair(ikmE) gives skEm` | A.2 Base Setup, `skEm` | `DeriveKeyPair(ikmE)`, `ikmE` 909a9b35...; f4ec9b33... |
| `RFC 9180 A.2: DeriveKeyPair(ikmE) gives pkEm, the enc` | A.2 Base Setup, `pkEm` | `pk(skEm)` = X25519(`skEm`, 9); 1afa08d3... |
| `RFC 9180 A.2: Encap gives shared_secret` | A.2 Base Setup, `shared_secret` | `Encap` from `skEm` f4ec9b33..., `pkEm` (= `enc`) 1afa08d3..., `pkRm` 4310ee97...; 0bbe7849... |
| `RFC 9180 A.2: KeySchedule gives key` | A.2 Base Setup, `key` | key schedule from `shared_secret`; ad2744de... |
| `RFC 9180 A.2: KeySchedule gives base_nonce` | A.2 Base Setup, `base_nonce` | 5c4d98150661b848853b547f |
| `RFC 9180 A.2: KeySchedule gives exporter_secret` | A.2 Base Setup, `exporter_secret` | a3b010d4... |
| `RFC 9180 A.2: Seal at sequence number 0 from KeySchedule` | A.2 Encryptions, sequence number 0 | through `key_schedule`; `aad` "Count-0"; `ct` 1c5250d8... |
| `RFC 9180 A.2: Seal at sequence number 0` | A.2 Encryptions, sequence number 0 | from the printed context; nonce ...547f; `ct` 1c5250d8... |
| `RFC 9180 A.2: Seal at sequence number 1, after IncrementSeq` | A.2 Encryptions, sequence number 1 | the context at `seq` 0 advanced by `increment_seq`; nonce ...547e; `ct` 6b53c051... |
| `RFC 9180 A.2: Seal at sequence number 2` | A.2 Encryptions, sequence number 2 | nonce ...547d; `ct` 71146bd6... |
| `RFC 9180 A.2: Seal at sequence number 4` | A.2 Encryptions, sequence number 4 | nonce ...547b; `ct` 63357a2a... |
| `RFC 9180 A.2: Seal at sequence number 255` | A.2 Encryptions, sequence number 255 | `aad` "Count-255"; nonce ...5480; `ct` 18ab939d... |
| `RFC 9180 A.2: Seal at sequence number 256` | A.2 Encryptions, sequence number 256 | `aad` "Count-256"; nonce ...557f; `ct` 7a4a13e9... |
| `RFC 9180 A.2: Export with the empty exporter_context` | A.2 Exported Values, first | `exporter_context` "", L = 32; 4bbd6243... |
| `RFC 9180 A.2: Export with exporter_context 00` | A.2 Exported Values, second | `exporter_context` 0x00, L = 32; 8c1df147... |
| `RFC 9180 A.2: Export with exporter_context TestContext` | A.2 Exported Values, third | `exporter_context` "TestContext", L = 32; 5acb0921... |

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
first form's sources decoded back to the ASCII its comment names, with the
stated concatenation lengths recomputed (`check_constants.py`). The first
form's byte literals were rendered from the fetched JSON and the labels'
ASCII by `build.py`, never typed by hand, and its four files were
byte-identical to that script's output.

The entry was then rewritten in the current language. The first form was
four files, one text split by the step budget of the time; the rewrite is
one root, `hpke.or`, that holds every vector of the four, and four modules
that it `use`s. Every expected value is carried over byte for byte from the
first form's `<name>_expected` specs: a script read the bytes of each old
spec from `origin/main` and compared them with the `hex"..."` literal of the
test that carries it (seventeen pairs, `rfc9180_a2_encryption_0` twice, once
through the key schedule and once from the printed context), and the inputs
(`ikmR`, `ikmE`, `skEm`, `pkEm`, `pkRm`, `shared_secret` and the printed
context) were printed by the same script from the first form's literals. No
vector was added or dropped. The labels, `info`, the plaintext and the `aad`
are now string literals; the SHA-256 round constants are `hex"..."` rows
read with `as big`, as in the `sha2` entry, and were recomputed from the
cube roots of the first 64 primes; the moduli are `(1 << 255) - 19` and
`(1 << 130) - 5`; and the Poly1305 clamp is written as the two
little-endian 64-bit halves of `0x0ffffffc0ffffffc0ffffffc0fffffff`. Beyond
the seventeen vectors, a scratch program outside the entry (`modcheck/gen.py`
in the scratch directory) ran the four modules on sixty generated cases,
none of them a vector of this entry: `sha256::digest` at nineteen lengths
from 1 to 256 bytes against `hashlib`, `hkdf::hmac`, `hkdf::extract` and
`hkdf::expand` against `hmac`/`hashlib` and `cryptography`'s `HKDFExpand`,
`chacha20poly1305::seal` against `cryptography`'s `ChaCha20Poly1305` for
additional data of 7, 8 and 9 bytes and plaintexts of 1, 15, 16, 17, 29,
33 and 64 bytes,
and `curve25519::x25519` against the first test vector of RFC 7748 section
5.2 and `cryptography`'s `X25519PrivateKey` on random keys; all agreed.

This entry is a reference evaluation of RFC 9180's base mode under
`orangec test`. It makes no constant-time, side-channel, performance or
certification claim; the X25519 swap is a specification of a choice, not a
constant-time swap, and the stateful nonce discipline of section 5.2 is
described, not enforced. It is not a corpus entry in the sense of The Orange
Book chapter 12.

## Gaps

- An array has at least one element, so an empty string is not a value:
  `labeled_extract` and `labeled_expand` take the label and the `ikm` or
  `info` as one string, and the calls with an empty `psk_id`, `psk`, `info`
  of `DeriveKeyPair` or `exporter_context` pass the label alone;
  `export_empty` is a second spec for the empty `exporter_context`, and
  `key_schedule` takes an `info` of at least one byte.
- A function has at most 256 instances, counting every combination of its
  sizes and types, so each length is a range: SHA-256 messages of 1 through
  256 bytes, HMAC texts of 1 through 192, labeled strings of 1 through 128,
  `info` and `exporter_context` of 1 through 64, and an AEAD `seal` over
  additional data of 7 through 9 bytes and plaintexts of 1 through 64 bytes
  (3 x 64 instances), which is where the two independent lengths meet.
- `use` reads a module only from the folder of the file that uses it, so
  `sha256.or`, `hkdf.or` and `chacha20poly1305.or` are copies written for
  this entry of what the `sha2`, `hmac-hkdf` and `chacha20-poly1305`
  entries hold, and `curve25519.or` is written here beside the `x25519`
  entry; the entries cannot share one module.
- A tuple holds no tuple and nothing is mutable, so `Seal` returns the
  ciphertext and `increment_seq` gives the context of the next `Seal`; there
  are no errors, so `IncrementSeq`'s `MessageLimitReachedError` and the
  all-zero check of section 7.1.4 are comments, not code.
- Not written, as a choice of scope and not for want of the language:
  HKDF's `Expand` past its first block (every `L` here is at most
  `Nh = 32`), Poly1305 over a partial last block (the AEAD pads every input
  to whole blocks), `Decap`, `Open`, and the PSK and authenticated modes.
