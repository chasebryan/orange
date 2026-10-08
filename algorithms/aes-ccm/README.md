# AES-CCM: Counter with CBC-MAC

CCM is the authenticated-encryption mode that combines counter-mode
encryption with a CBC-MAC over the nonce, the associated data and the
plaintext, under one block cipher key. Doug Whiting, Russ Housley and Niels
Ferguson designed it in 2002 for IEEE 802.11i as a patent-free alternative
to OCB; [RFC 3610](https://www.rfc-editor.org/rfc/rfc3610)
(2003) is the original definition with its packet vectors, and NIST published
it as [SP 800-38C](https://doi.org/10.6028/NIST.SP.800-38C) (2004, with an
update in 2007), whose formatting function of Appendix A fixes the byte
layout of every block. With AES-128 it is the cipher of WPA2 (CCMP), of IEEE
802.15.4 and Zigbee (as the variant CCM\*), of Bluetooth Low Energy, of IPsec
ESP (RFC 4309), of TLS 1.2 (RFC 6655, RFC 7251) and of the optional TLS 1.3
suites `TLS_AES_128_CCM_SHA256` and `TLS_AES_128_CCM_8_SHA256` (RFC 8446). It
is a current NIST standard with no known attack below its generic,
birthday-bound security, provided nonces never repeat under a key.

## Analysis

### Structure

CCM takes a key `K`, a nonce `N` of `n` bytes with `7 <= n <= 13`,
associated data `A`, a payload `P` and a tag length `Tlen` in bits, and
returns `C = ciphertext || T` of `Plen + Tlen` bits. It is a two-pass mode,
and both passes use only the forward block cipher `CIPH_K` (SP 800-38C
section 5.1), so the entry carries the cipher of FIPS 197, in the module
`aes`, and not its inverse.

The first pass is the CBC-MAC of section 6.1, steps 1 to 4, over the blocks
`B_0 || B_1 || ... || B_r` that the formatting function of Appendix A.2
makes from `N`, `A` and `P`. `B_0` (A.2.1) is `Flags || N || Q`, where the
flags byte `0 || Adata || [(t - 2) / 2]_3 || [q - 1]_3` records whether
associated data is present, the tag length `t` in bytes, and the size `q` of
the length field, and `Q = [Plen / 8]_8q` is the payload length in octets in
the `q = 15 - n` bytes left by the nonce. The associated data follows
(A.2.2) with its byte length in front, `[a]_16 || A` for `a < 2^16 - 2^8`,
and the payload (A.2.3) after it, each padded with zeros to whole blocks.
Then `Y_0 = CIPH_K(B_0)`, `Y_i = CIPH_K(B_i ^ Y_{i-1})` and `T = MSB_Tlen(Y_r)`.
The second pass (steps 5 to 7) is counter mode over the counter blocks of
Appendix A.3, `Ctr_i = Flags || N || [i]_8q`, with `S_j = CIPH_K(Ctr_j)`:
`Ctr_1, Ctr_2, ...` encrypt the payload and `Ctr_0` masks the tag, so that
step 8 returns `(P ^ MSB_Plen(S_1 || ... || S_m)) || (T ^ MSB_Tlen(S_0))`.
Decryption-verification (section 6.2) runs the counter pass first to
recover `P` and the received tag, then the CBC-MAC pass over `N`, `A` and
the recovered `P`, and accepts only when the two tags agree.

| Standard section | Orange spec |
| --- | --- |
| FIPS 197 section 3.4, the state | `aes::state`, `aes::output` (types `Row`, `State`) |
| FIPS 197 section 4.2, multiplication by `x` | `aes::xtime` |
| FIPS 197 section 5.1.1, SubBytes and the S-box (Table 4) | `aes::sbox`, `aes::sub_bytes` |
| FIPS 197 sections 5.1.2 to 5.1.4 | `aes::shift_rows`, `aes::mix_columns`, `aes::add_round_key` |
| FIPS 197 section 5.2, KeyExpansion with `Nk = 4` (Algorithm 2, Table 5) | `aes::rot_word`, `aes::sub_word`, `aes::xor_word`, `aes::rcon`, `aes::key_expansion` |
| FIPS 197 section 5.1, Cipher (Algorithm 1), and section 5, AES-128 | `aes::cipher`, `aes::aes128` |
| SP 800-38C section 5.1, `CIPH_K` | `ciph` |
| The standard's symbols, `X ^ Y` and the length of a string in octets | `xor[l]`, `octets[l]` |
| Appendix A.1, the layout Flags, N, `[x]_8q` with `n + q = 15` shared by `B_0` and `Ctr_i` | `flags_nonce_integer[n]` |
| Appendix A.2.1, `B_0` (Tables 1 and 2) | `b_0[n]` |
| Appendix A.2.2, the associated-data blocks | `associated_data_blocks[a]` |
| Appendix A.2.3, the payload blocks | `payload_blocks[p]` |
| Appendix A.2, the formatting function | `b_0(...) ++ associated_data_blocks(assoc) ++ payload_blocks(payload)` in the processes; `b_0(...) ++ payload_blocks(payload)` in `decryption_verification_without_a` |
| Appendix A.3, the counter blocks (Tables 3 and 4) | `ctr[n]` |
| Section 6.1, steps 2 to 4, the CBC-MAC | `cbc_mac[blocks]` |
| Section 6.1, steps 5 to 7, the counter blocks and `S` | `keystream[n, m]` |
| Section 6.1, step 8 (and section 6.2, step 5), `P ^ MSB_Plen(S)` | `counter_mode[n, p]` |
| Section 6.1, generation-encryption | `generation_encryption[N, A, P, u]` |
| Section 6.2, decryption-verification | `decryption_verification[N, A, P, u]`, `decryption_verification_without_a[N, P, u]` |

Lengths are sizes. The nonce is `Word[8]^n` for `n` in 7 through 13, the
seven lengths Appendix A.1 allows, and `q = 15 - n` follows from it, so
`flags_nonce_integer` writes `Flags || N || [x]_8q` as the join of a
flags byte, the nonce and `x as big Word[8]^(15 - n)`. The associated
data and the payload are `Word[8]^a` and `Word[8]^p` for 1 through 32
bytes, and their formatted blocks have the lengths the standard gives
them, `16 ceil((2 + a) / 16)` and `16 ceil(p / 16)` bytes, written into
the result types. `cbc_mac` takes the formatted string as
`Word[8]^(16 * blocks)` and `keystream` makes `m` counter blocks, so
every step of section 6.1 has the length of its operands in its type.

The two processes take N, A and P at once, and that is more lengths than
one function can range over: a function has at most 256 instances,
counting every combination of its sizes, and ranges from the shortest to
the longest of the vectors' lengths would give 6 x 13 x 21 x 13 of them
(`n`, `a`, the payload length and the tag length), `a` and the payload
length alone 273. So `generation_encryption` and the decryption processes
take N, A and P at the lengths of the vectors, listed as types (`N in
{Word[8]^7, Word[8]^8, Word[8]^12}`, associated data of 8, 16 and 20
bytes, payloads of 4, 16 and 24), and the tag length as the size `u`,
`t = 2u` bytes for `u` in 2 through 8: the seven tag lengths of Appendix
A.1, whose flags field `[(t - 2) / 2]_3` is `u - 1`. That is 189
instances. Each instance calls the sized steps above, which pick their
own instances from the lengths of its arguments, and the formatting
function of Appendix A.2 is the join `b_0(...) ++
associated_data_blocks(assoc) ++ payload_blocks(payload)` (without the
middle term in `decryption_verification_without_a`) passed straight to
`cbc_mac`, whose block count the checker reads off that join; no
function names the formatted string's length in terms of the listed
types, which it could not. Since the type parameters carry the
standard's names `N`, `A` and `P`, the strings themselves are the
parameters `nonce`, `assoc` and `payload` (and `ciphertext` and `tag` in
decryption), and `a` and `plen` stay the lengths, as in `b_0`.

The same limit shapes the results. `C = ciphertext || T` has
`Plen / 8 + t` bytes, a length no type can write when `Plen / 8` comes
from a listed type, so `generation_encryption` returns the pair
`(ciphertext, tag)`, the two halves of `C` in order, and the
decryption processes take the same two halves as two arguments (the
length of an array inside a tuple argument does not choose an instance,
and the tag's length is what picks `u`). Section 6.2 returns either
`INVALID` or `P`; Orange has no sum type, so the decryption processes
return the pair `(verdict, P)`, `VALID` as `true`, and `P` counts only
when the verdict is `true`. The empty associated data of Wycheproof's
test case 52 is not an array, since an array has at least one element,
so `decryption_verification_without_a` is the same process with
`Adata = 0` and no associated-data blocks.

### Security status

Jonsson (2002) proved CCM secure as an authenticated-encryption scheme when
the block cipher is a pseudorandom permutation and nonces do not repeat: the
advantage of an adversary is bounded by a birthday term, of the order of
`sigma^2 / 2^128` for `sigma` blocks processed under one key, plus
`q_v / 2^Tlen` for `q_v` forgery attempts. No attack below these bounds is
known. SP 800-38C draws the practical limits from them: at most `2^61` block
cipher invocations under one key (section 5.1), and a tag of at least 64 bits
unless the number of forgery attempts an attacker can make is itself limited
(Appendix B.2).

The design has been criticized rather than broken. Rogaway and Wagner
("A Critique of CCM", 2003) listed what the two-pass structure costs. CCM
is not online: `B_0` carries `Plen`, so the CBC-MAC cannot start before the
whole message length is known, and the MAC is over the plaintext, so a
receiver must decrypt everything before it can verify anything and must
buffer the result. The associated data follows `B_0`, which contains the
nonce, so the MAC of a fixed header cannot be computed once per key. The
2-byte length prefix of `A` shifts the data off word boundaries, the
payload costs two block cipher calls per block with no parallelism in the
MAC pass, and the parameterization by `q` and `n` is more intricate than
the security it buys. The same authors' EAX (with Bellare, 2004) and later
GCM (McGrew and Viega, 2004) were designed to remove these costs, and GCM,
not CCM, became the default of TLS and IPsec; CCM kept the places where a
small code base or hardware reuse of the AES encryption core matters.

The nonce and the message length share the fifteen bytes after the flags
byte: with `n = 13` the payload is limited to `2^16 - 1` bytes, with
`n = 12` to `2^24 - 1`, and with `n = 7` the length field is eight bytes
wide. IEEE 802.11i (2004) took `n = 13` (priority, address and a 48-bit
packet number) for frames under 64 KiB, with an 8-byte MIC; Bluetooth Low
Energy uses `n = 13` with a 4-byte MIC, the low end of what the standard
allows, at a forgery probability of `2^-32` per attempt; TLS and IPsec use
`n = 12`, the implicit salt and the record sequence number, with a 16-byte
tag, or an 8-byte tag in the `CCM_8` suites. `TLS_AES_128_CCM_8_SHA256` is
in TLS 1.3 for constrained devices (RFC 7925 requires the corresponding
TLS 1.2 suite for IoT profiles) but is marked not recommended in the IANA
registry, because `2^-64` per forgery attempt is a margin that has to be
enforced by counting failures and rekeying rather than one that holds by
itself.

A repeated nonce under one key voids the proof. The counter keystream
repeats, so the exclusive-or of the two payloads is exposed, and the masks
`S_0` repeat, so the two masked tags reveal the exclusive-or of the two
CBC-MACs. This is not hypothetical: the KRACK attacks (Vanhoef and Piessens,
2017) reset WPA2's packet number by reinstalling a key, which reused CCMP
nonces and let an attacker decrypt and replay frames; against CCMP the
CBC-MAC did not give up forgeries, where against GCMP the same reuse
recovers the authentication key. Implementation pitfalls are the usual ones
for a byte-oriented mode: nonce lengths outside 7 to 13 and tag lengths
outside `{4, 6, ..., 16}` bytes leave `B_0` undefined and must be rejected
(Wycheproof's notes record memory overflows in implementations that took
nonces longer than 60 bytes, CVE-2017-18330), and a verifier must compare
the whole tag.

Status as of 2026: SP 800-38C is a current NIST recommendation, and NIST's
review of the SP 800-38 series (NIST IR 8459, September 2024) recommends
reaffirming it with possible corrections, among them one for an error in the
decryption-verification of a zero-length plaintext (section 6 of the report);
NIST announced in April 2025 that it will revise SP 800-38C. CCM is the
mandatory cipher of WPA2 and of WPA3-Personal, the cipher of Zigbee, Thread
and Bluetooth LE link encryption, and optional in TLS 1.3.

### What the Orange rendering shows

The steps that depend on the data are all inside AES. The data-indexed
reads are the S-box lookups: the S-box of FIPS 197 Table 4 is a
`hex"..."` string of 256 bytes, sixteen rows as the standard prints them,
and SubBytes and SubWord read it as `box[x]` with the byte itself as the
index, 160 lookups per block and 40 in the key schedule. The checker
proves every such index in range, because a `Word[8]` indexes a
256-entry table exactly. The one data-dependent branch is the
conditional `if (b & 0x80) != 0 { 0x1b } else { 0 }` in `xtime`, the
multiplication by `x` of FIPS 197 section 4.2, which tests the top bit of
a state byte each time MixColumns multiplies by `{02}` or `{03}`. Neither
the lookups nor the `if` carry any timing claim. The state is the 4 x 4
byte array `s[r][c]` of FIPS 197 section 3.4, and ShiftRows and
MixColumns index it with the coordinates the standard writes, modulo 4.

Nothing in CCM itself branches on the data. What it does depend on are its
lengths, and those are now sizes rather than values: the nonce and the
`q`-byte integer of A.1 are joined with `++`, the length field of B_0 and
of the counter blocks is `x as big Word[8]^(15 - n)`, the 2-byte prefix
`[a]_16` is `a as big Word[8]^2`, the associated data and the payload are
placed in their zero blocks with slice updates `with [2..a + 2]` and
`with [0..p]`, the CBC-MAC slices block `i` out of the formatted string
with `b[16 * i..16 * i + 16]`, and `MSB_Tlen` and `MSB_Plen` are the
slices `y[..2 * u]` and `s[..p]`. Every position is a literal, a loop
index or a size, so the checker proves every access in range for every
instance before evaluation, and no step selects a byte by comparing
indices. The one comparison is the tag check of section 6.2, step 10,
`t == y[..2 * u]` over the whole tag, whose cost does not depend on where
the two differ.

Measured costs under `orangec test --stats` and `orangec eval --stats`:
a call of `ciph`, `CIPH_K` as AES-128 with its key expansion, is 22,537
steps, of which the call of `key_expansion` is 2,922; `ciph` expands the
key at every call, as `CIPH_K` names the key and not a schedule, at a cost
of about 13 percent.
`B_0` costs 47 steps, a counter block 21, the associated-data blocks 17,
the payload blocks 8 and a 16-byte `xor` 181, so the formatting is under
one percent of a CCM call and the block cipher the rest. A call costs
`(r + 1) + (m + 1)` block cipher calls, which the reader can count from
the sizes: five for C.1 (one block each of `A` and `P`, `S_0`, one counter
block), six for C.2 (the 2-byte length pushes 16 bytes of `A` into two
blocks), eight for C.3, five for Wycheproof's test case 12, and four for
test case 52 (no associated data). The tests cost 113,540 (C.1), 136,424
(C.2), 181,839 (C.3), 113,804 (test case 12), 113,543 (C.1 decrypted) and
91,050 steps (test case 52), 750,200 in all.

Not written, by this entry's choice: the longer length encodings of
A.2.2. The entry encodes only `[a]_16`, for `a < 2^16 - 2^8`. The 6-byte
form `0xff 0xfe || [a]_32`, for `2^16 - 2^8 <= a < 2^32`, would fit an
array, since its shortest formatted field, the prefix and 65,280 bytes of
`A`, is 65,286 bytes, but at about 4,080 blocks of some 22,500 steps each
it would cost about 92 million steps, far over the budget of an entry.
Only the 10-byte form `0xff 0xff || [a]_64` is beyond the language, since
its `a >= 2^32` bytes cannot be an array.

Also not expressed: the processes at lengths other than the vectors' (the
steps take nonces of 7 to 13 bytes and associated data and payloads of 1
to 32, the processes only the listed lengths; adding a length is a change
to the lists in the processes' signatures, within the 256 instances);
generation-encryption with empty associated data, which no vector here
needs; AES-192 and AES-256 as the block cipher (the Appendix C examples are
all `Klen = 128`); and the validity requirements on `N`, `A`, `P` and `Tlen`
of section 5.4 and Appendix A.1, which section 6.1 takes as prerequisites
and section 6.2 step 7 checks, and which hold here by the listed lengths
rather than being computed.

## Dissemination

### Files

- `aes-ccm.or`: module `aes_ccm`, the root. `CIPH_K`, the formatting
  function of Appendix A.2 as `b_0`, `associated_data_blocks` and
  `payload_blocks`, the counter blocks of A.3, `cbc_mac`, `keystream`,
  `counter_mode`, `generation_encryption`, `decryption_verification` and
  `decryption_verification_without_a`, and the six tests below.
- `aes.or`: module `aes`, AES-128 of FIPS 197, the forward cipher with its
  S-box and key expansion. It has no tests of its own; `aes-ccm.or` uses it.

### Running

```console
orangec test algorithms/aes-ccm/aes-ccm.or
python3 algorithms/verify.py algorithms/aes-ccm
```

`verify.py` runs the six tests of `aes-ccm.or` and checks `aes.or`, which
passes as a module of the root.

### Vectors

Each row is a `test` block in `aes-ccm.or`.

| Test | Source | Case |
| --- | --- | --- |
| `SP 800-38C C.1: generation-encryption` | SP 800-38C, Appendix C.1, via Botan's `ccm.vec` ("SP 800-38C Example 1"); confirmed with the Python `cryptography` AESCCM oracle | Klen 128, Tlen 32, Nlen 56, Alen 64, Plen 32: key 404142...4f, N 10111213141516, A 0001...07, P 20212223; C = 7162015b 4dac255d |
| `SP 800-38C C.2: generation-encryption` | SP 800-38C, Appendix C.2, via Botan's `ccm.vec` ("Example 2"); confirmed with `cryptography` | Tlen 48, Nlen 64, Alen 128, Plen 128: C = d2a1f0e0...593d 1fc64fbfaccd |
| `SP 800-38C C.3: generation-encryption` | SP 800-38C, Appendix C.3, via Botan's `ccm.vec` ("Example 3"); confirmed with `cryptography` | Tlen 64, Nlen 96, Alen 160, Plen 192: C = e3b201a9...e70b 6176aad9a4428aa5 484392fbc1b09951 |
| `Wycheproof tcId 12: generation-encryption` | Wycheproof `testvectors_v1/aes_ccm_test.json`, tcId 12 | AES-128, 12-byte nonce, 8-byte aad, 16-byte msg, 16-byte tag; ct 08db327a..., tag b7c249f8... |
| `SP 800-38C C.1: decryption-verification` | SP 800-38C, Appendix C.1, via Botan's `ccm.vec`: the C.1 ciphertext back to its payload | section 6.2 on C = 7162015b4dac255d: VALID (`true`, step 10) and P = 20212223 (step 5) |
| `Wycheproof tcId 52: decryption-verification rejects` | Wycheproof `aes_ccm_test.json`, tcId 52, "Flipped bit 0 in tag", result `invalid` | AES-128, 12-byte nonce, no aad, 16-byte msg, 16-byte tag with bit 0 flipped: INVALID (`false`) |

Botan's `ccm.vec` is `src/tests/data/aead/ccm.vec` of the Botan repository,
which carries the three Appendix C examples under that name; the three
values were also checked against the `cryptography` package's `AESCCM`
with the example's tag length, so the row does not rest on one transcription.
The `_expected` literals of the first form were generated by script from the
fetched files.

### Provenance and claims

The standards' own sites are not reachable from the machine that wrote this
entry, so every constant and vector was taken from a fetched file or an
oracle and cross-checked by script, never transcribed by eye. The S-box was
computed from its definition in FIPS 197 section 5.1.1 (the inverse in
GF(2^8) modulo `x^8 + x^4 + x^3 + x + 1`, then the affine map with constant
`0x63`), compared entry by entry with the `sbox[]` table of the tiny-AES-c
reference implementation, and packed into the `Word[64]` literals by the
same script; the round constants are the powers of `x` in that field. The
CCM layout (the flags bytes of Tables 1 and 4, the length encodings, the
counter blocks) was written as a Python reference of SP 800-38C for this
entry, and that reference agrees with the `cryptography` package's `AESCCM`
and with every AES-128 case of three fetched vector files: all 34 AES-128
cases of Botan's `ccm.vec` (the RFC 3610 packet vectors, the Appendix C
examples and NIST CAVS cases; the file's one SM4 case aside), all 135
valid and 27 invalid AES-128 cases of Wycheproof's `aes_ccm_test.json` with
nonces of 7 to 13 bytes, and all 796 encryption and 159 decryption-failure
`aes-128-ccm` cases of OpenSSL's `evpciph_aes_ccm_cavs.txt`. It served as
the oracle for the block-by-block values while the Orange was written. In
that first form the three `.or` files were assembled by one script from one
hand-written algorithm text and the fetched vectors, so their algorithm
parts were identical.

The entry was then rewritten in the current language, the three files folded
into one root, `aes-ccm.or`, with the cipher in the module `aes.or`. Every
expected value is carried over byte for byte from the first form, where each
was a `<name>_expected` spec. Those specs held `C` in a 48-byte buffer, and
the recovered payload in a 32-byte one, followed by zeros because the first
form's buffers had one length; the new tests state the `Clen` bytes of `C`,
split after `Plen / 8` bytes into the ciphertext and the tag, and the 4 bytes
of `P`, and a script compared each with the prefix of the old value
evaluated by `orangec eval` from `origin/main` and checked that everything
after it was zero. The two verdicts are the same `true` and `false`. The
inputs (keys, nonces, associated data, payloads, ciphertexts and tags) were
compared by script with the old specs' literals, and every test's inputs and
expected value were run again through `cryptography`'s `AESCCM` (encryption,
the decryption of C.1, and the rejection of test case 52, whose tag with
bit 0 restored decrypts to the payload 202122...2f). A scratch run of
`decryption_verification_without_a` on that restored tag returned `true`
with the same payload, but the source prints no such vector, so no test
in the entry pins the accepting side of that spec; only the rejection of
test case 52 exercises it. The S-box, now sixteen `hex"..."` rows, was
compared by script, byte for byte, with the table computed from its
definition in FIPS 197 section 5.1.1 and with the first form's packed
`Word[64]` literals; the round constants of Table 5, now one
`hex"..."` row, are the leading bytes of the first form's `Rcon` words,
compared the same way. No vector was added or dropped.

This entry is a reference evaluation of a specification under `orangec
test`: it shows that the Orange text computes the standard's values on the
cases listed. It makes no constant-time, side-channel, performance or
certification claim, it is not an implementation anyone should deploy, and
it is not a corpus entry in the sense of The Orange Book chapter 12.

## Gaps

- A function has at most 256 instances, counting every combination of its
  sizes and listed types, and CCM's processes depend on four lengths: the
  nonce, the associated data, the payload and the tag. No function can
  range over the associated-data and payload lengths of the vectors at
  once (13 x 21 = 273), so the formatting function is a join of sized
  steps inside each process rather than one spec, and the processes take
  `N`, `A` and `P` at the vectors' lengths, listed as types, with the tag
  length as a size; a vector at another length needs its length added to
  a list.
- When a length comes from a listed type, no type can write a length
  computed from it: `C = ciphertext || T` is returned, and taken, as its
  two halves, and the formatted string is never bound to a name.
- No array has zero elements, so empty associated data (`a = 0`) is not a
  value: the decryption process for test case 52 is a second spec without
  the parameter `A`, and there is no generation-encryption without
  associated data.
- The length of an array inside a tuple argument does not choose a sized
  function's instance, so the decryption processes take the ciphertext and
  the tag as two arguments rather than the pair the generation process
  returns.
- No sum type: `INVALID` or `P` of section 6.2 is the pair `(verdict, P)`,
  and the caller carries the rule that `P` counts only when the verdict is
  `true`.
