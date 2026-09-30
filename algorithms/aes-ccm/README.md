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
section 5.1), so the Orange file carries the cipher of FIPS 197 and not its
inverse.

The first pass is the CBC-MAC of section 6.1, steps 1 to 4, over the blocks
`B_0 || B_1 || ... || B_r` that the formatting function of Appendix A.2
makes from `N`, `A` and `P`. `B_0` (A.2.1) is `Flags || N || Q`, where the
flags byte `0 || Adata || [(t - 2) / 2]_3 || [q - 1]_3` records whether
associated data is present, the tag length `t` in bytes, and the size `q` of
the length field, and `Q = [Plen / 8]_8q` is the payload length in octets in
the `q = 15 - n` bytes left by the nonce. The associated data follows (A.2.2) with its
byte length in front, `[a]_16 || A` for `a < 2^16 - 2^8`, and the payload
(A.2.3) after it, each padded with zeros to whole blocks. Then
`Y_0 = CIPH_K(B_0)`, `Y_i = CIPH_K(B_i ^ Y_{i-1})` and `T = MSB_Tlen(Y_r)`.
The second pass (steps 5 to 7) is counter mode over the counter blocks of
Appendix A.3, `Ctr_i = Flags || N || [i]_8q`, with `S_j = CIPH_K(Ctr_j)`:
`Ctr_1, Ctr_2, ...` encrypt the payload and `Ctr_0` masks the tag, so that
step 8 returns `(P ^ MSB_Plen(S_1 || ... || S_m)) || (T ^ MSB_Tlen(S_0))`.
Decryption-verification (section 6.2) runs the counter pass first to
recover `P` and the received tag, then the CBC-MAC pass over `N`, `A` and
the recovered `P`, and accepts only when the two tags agree.

| Standard section | Orange spec |
| --- | --- |
| FIPS 197 section 5.1.1, SubBytes and the S-box (Table 4) | `s_box`, `lookup`, `byte_at`, `sub_bytes` |
| FIPS 197 section 5.1.2 to 5.1.4 | `shift_rows`, `mix_column`, `mix_columns`, `add_round_key` |
| FIPS 197 section 5.2, KeyExpansion with `Nk = 4` | `rot_word`, `sub_word`, `rcon`, `key_expansion` |
| FIPS 197 section 5.1, Cipher, the `CIPH_K` of SP 800-38C section 5.1 | `cipher` |
| SP 800-38C Appendix A.1, the layout Flags, N, `[x]_8q` with `n + q = 15` shared by `B_0` and `Ctr_i` | `flags_nonce_integer`, `byte_weight` |
| Appendix A.2.1, `B_0` (Table 1) | `b_0` |
| Appendix A.2.2, the associated-data blocks | `associated_data_blocks`, `associated_data_block_count` |
| Appendix A.2.3, the payload blocks | `payload_blocks`, `payload_block_count` |
| Appendix A.2, the formatting function | `formatting`, `formatted_block_count`, `byte_of` |
| Appendix A.3, the counter blocks (Table 2) | `ctr` |
| Section 6.1, steps 2 to 4, the CBC-MAC | `cbc_mac` |
| Section 6.1, steps 5 to 7, counter mode | `counter_mode`, `xor_block`, `mask_tag` |
| Section 6.1, generation-encryption | `generation_encryption`, `block_byte` |
| Section 6.2, steps 2 to 6, the recovered payload and received tag | `recovered_payload`, `received_tag` |
| Section 6.2, decryption-verification | `decryption_verification` |

The nonce travels in a 13-byte buffer with its length `n` beside it, the
associated data and the payload in 32-byte buffers with `alen` and `plen`,
and the ciphertext in a 48-byte buffer with `clen`, because an Orange array
has one length and CCM's own parameters vary: the three examples of
Appendix C use nonces of 7, 8 and 12 bytes and tags of 4, 6 and 8 bytes, and
the Wycheproof case a 12-byte nonce and a 16-byte tag. The 6.1 output is
`C` in the standard's layout, the tag starting at byte `Plen`, with zeros
past `Clen`. Section 6.2 returns either `INVALID` or `P`; Orange has no sum
type, so `decryption_verification` is the verdict of step 10 as a `Bool`
and `recovered_payload` is the `P` of step 5, to be taken only when the
verdict is `true`.

### Security status

Jonsson (2002) proved CCM secure as an authenticated-encryption scheme when
the block cipher is a pseudorandom permutation and nonces do not repeat: the
advantage of an adversary is bounded by a birthday term, of the order of
`sigma^2 / 2^128` for `sigma` blocks processed under one key, plus
`q_v / 2^Tlen` for `q_v` forgery attempts. No attack below these bounds is
known. SP 800-38C Appendix B draws the practical limits from them: at most
`2^61` block cipher invocations under one key, and a tag of at least 64 bits
unless the number of forgery attempts an attacker can make is itself
limited.

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
review of the SP 800-38 series (NIST IR 8459, initial public draft, 2023)
proposed no change to CCM as far as this author recalls (not checked from
this machine); CCM is the mandatory cipher of WPA2 and of WPA3-Personal, the
cipher of Zigbee, Thread and Bluetooth LE link encryption, and optional in
TLS 1.3.

### What the Orange rendering shows

Every data-dependent choice of the mode is a table lookup inside AES: 160
S-box lookups per block and 40 in the key schedule, each a 32-way selection
over the packed table at 291 steps (measured), which makes a block cipher
call cost about 69,900 steps and `key_expansion` about 15,200. Nothing in
CCM itself branches on the data. What it does branch on are its lengths, and
that is where the Orange form departs from the standard's text: an index
must be static, so every position that is a value is written as a
selection. `flags_nonce_integer` lays the nonce and the length field over
the same sixteen bytes under guards `i < n` and `i < q`; the payload blocks
begin at block `1 + ceil((2 + a) / 16)`, a value, so `formatting` places
each payload byte with `byte_of`, a selection over the 32-byte buffer; the
masked tag starts at byte `Plen` of `C` and is placed with `block_byte`; and
`received_tag` selects the last `Tlen` bytes of a ciphertext of `Clen`
bytes. The whole formatting function for the C.3 sizes costs about 19,000
steps, the placement selections most of it, and a counter block about
1,100: the formatting is under three percent of a CCM call, the block
cipher the rest. The cost of one call is `(r + 1) + (m + 1)` block cipher
calls, which the reader can count from the sizes: five for C.1 (one block
each of `A` and `P`, `S_0`, one counter block), six for C.2 (the 2-byte
length pushes 16 bytes of `A` into two blocks), eight for C.3.

The budget sized the files. Measured with a filler spec, the pairs cost
about 366,000 steps (C.1), 436,000 (C.2), 571,000 (C.3), 371,000
(Wycheproof 12), 86,000 (the C.1 payload recovery), 367,000 (the C.1
verdict) and 311,000 (the Wycheproof 52 rejection), out of 1,048,576 steps
per file; so `aes-ccm.or` holds C.1 and C.2 (about 802,000), `aes-ccm-c3.or`
C.3 and the Wycheproof case (about 941,000) and `aes-ccm-decrypt.or` the
three decryption pairs (about 763,000). Nothing planned was dropped.

Not expressed: associated data of `2^16 - 2^8` bytes or more, whose 6- and
10-byte length encodings of A.2.2 need a buffer no Orange array can hold;
payloads over 32 bytes and associated data over 32 bytes, since an array's
length is part of its type (a second set of specs over `Word[8]^64` would be
the same text); AES-192 and AES-256 as the block cipher (the Appendix C
examples are all `Klen = 128`); and the validity requirements on `N`, `A`,
`P` and `Tlen` of section 5.3 and Appendix A.1, which section 6.1 takes as
prerequisites and section 6.2 step 7 checks, and which are stated in
comments here rather than computed.

## Dissemination

### Files

- `aes-ccm.or`: module `aes_ccm`. AES-128 (the forward cipher, its packed
  S-box and key expansion), the formatting function of Appendix A.2 as
  `b_0`, `associated_data_blocks`, `payload_blocks` and `formatting`, the
  counter blocks of A.3, `cbc_mac`, `counter_mode`, `mask_tag`,
  `generation_encryption`, and the examples C.1 and C.2 of Appendix C.
- `aes-ccm-c3.or`: the same algorithm, with the example C.3 of Appendix C
  and test case 12 of Wycheproof's `aes_ccm_test.json`.
- `aes-ccm-decrypt.or`: the same algorithm plus `received_tag`,
  `recovered_payload` and `decryption_verification` of section 6.2, with
  the decryption of C.1 (payload and verdict) and the rejection of
  Wycheproof's test case 52.

The three files are one file split by the step budget; their algorithm part
is identical text, generated from one source (see Provenance).

### Running

```console
orangec eval algorithms/aes-ccm/aes-ccm.or
orangec eval algorithms/aes-ccm/aes-ccm-c3.or
orangec eval algorithms/aes-ccm/aes-ccm-decrypt.or
python3 algorithms/verify.py algorithms/aes-ccm
```

`eval` prints every parameterless spec, including `s_box` and `rcon`; the
pairs below are the vectors. `verify.py` reports seven vectors reproduced.

### Vectors

| Spec | Source | Case |
| --- | --- | --- |
| `sp800_38c_c1` | SP 800-38C, Appendix C.1, via Botan's `ccm.vec` ("SP 800-38C Example 1"); confirmed with the Python `cryptography` AESCCM oracle | Klen 128, Tlen 32, Nlen 56, Alen 64, Plen 32: key 404142...4f, N 10111213141516, A 0001...07, P 20212223; C = 7162015b 4dac255d |
| `sp800_38c_c2` | SP 800-38C, Appendix C.2, via Botan's `ccm.vec` ("Example 2"); confirmed with `cryptography` | Tlen 48, Nlen 64, Alen 128, Plen 128: C = d2a1f0e0...593d 1fc64fbfaccd |
| `sp800_38c_c3` | SP 800-38C, Appendix C.3, via Botan's `ccm.vec` ("Example 3"); confirmed with `cryptography` | Tlen 64, Nlen 96, Alen 160, Plen 192: C = e3b201a9...e70b 6176aad9a4428aa5 484392fbc1b09951 |
| `wycheproof_ccm_tc_12` | Wycheproof `testvectors_v1/aes_ccm_test.json`, tcId 12 | AES-128, 12-byte nonce, 8-byte aad, 16-byte msg, 16-byte tag; ct 08db327a..., tag b7c249f8... |
| `sp800_38c_c1_payload` | SP 800-38C, Appendix C.1, via Botan's `ccm.vec`: the C.1 ciphertext back to its payload | section 6.2 step 5 from C = 7162015b4dac255d: P = 20212223 |
| `sp800_38c_c1_verification` | SP 800-38C, Appendix C.1, via Botan's `ccm.vec` | section 6.2 step 10 on the C.1 ciphertext: VALID (`true`) |
| `wycheproof_ccm_tc_52_verification` | Wycheproof `aes_ccm_test.json`, tcId 52, "Flipped bit 0 in tag", result `invalid` | AES-128, 12-byte nonce, 16-byte msg, 16-byte tag with bit 0 flipped: INVALID (`false`) |

Botan's `ccm.vec` is `src/tests/data/aead/ccm.vec` of the Botan repository,
which carries the three Appendix C examples under that name; the three
values were also checked against the `cryptography` package's `AESCCM`
with the example's tag length, so the row does not rest on one transcription.
The `_expected` literals were generated by script from the fetched files.

### Provenance and claims

The standards' own sites are not reachable from the machine that wrote this
entry, so every constant and vector was taken from a fetched file or an
oracle and cross-checked by script, never transcribed by eye. The S-box was
computed from its definition in FIPS 197 section 5.1.1 (the inverse in
GF(2^8) modulo `x^8 + x^4 + x^3 + x + 1`, then the affine map with constant
`0x63`), compared entry by entry with the `sbox[]` table of the tiny-AES-c
reference implementation, and packed into the `Word[64]` literals by the
same script; the round constants are the powers of `x` in that field. The
CCM layout (the flags bytes of Tables 1 and 2, the length encodings, the
counter blocks) was written as a Python reference of SP 800-38C for this
entry, and that reference agrees with the `cryptography` package's `AESCCM`
and with every AES-128 case of three fetched vector files: all 34 AES-128
cases of Botan's `ccm.vec` (the RFC 3610 packet vectors, the Appendix C
examples and NIST CAVS cases; the file's one SM4 case aside), all 135
valid and 27 invalid AES-128 cases of Wycheproof's `aes_ccm_test.json` with
nonces of 7 to 13 bytes, and all 796 encryption and 159 decryption-failure
`aes-128-ccm` cases of OpenSSL's `evpciph_aes_ccm_cavs.txt`. It served as
the oracle for the block-by-block values while the Orange was written. The
three `.or` files were assembled by one script from one hand-written
algorithm text and the fetched vectors, so their algorithm parts are
identical.

This entry is a reference evaluation of a specification under `orangec
eval`: it shows that the Orange text computes the standard's values on the
cases listed. It makes no constant-time, side-channel, performance or
certification claim, it is not an implementation anyone should deploy, and
it is not a corpus entry in the sense of The Orange Book chapter 12.

## Gaps

- No imports: each of the three files repeats the 333 lines of AES-128 and
  CCM to add its vectors, and the step budget of 1,048,576 per file forces
  the split, since one block cipher call costs about 69,900 steps and the
  seven pairs need 35 of them.
- Static indices only: the positions that CCM's own lengths determine (the
  length field after a nonce of `n` bytes, the first payload block after
  the associated data, the tag after `Plen` bytes of ciphertext, the last
  `Tlen` bytes of `C`) are selections over a buffer instead of an index,
  `byte_of`, `block_byte` and `received_tag`; they cost about 19,000 steps
  per formatting, small next to the cipher, but they are the least
  standard-like lines of the file.
- No length polymorphism and arrays of at most 256 elements: the associated
  data and payload are 32-byte buffers with their lengths beside them, the
  nonce a 13-byte buffer with `n`, and the long associated-data encodings of
  A.2.2 (6 and 10 bytes) are out of reach.
- No sum type: `INVALID` or `P` of section 6.2 is a `Bool` verdict and a
  separate payload spec, and the caller carries the rule that the payload
  counts only when the verdict is `true`; a `Bool` result also cannot be
  paired with the payload in one array, since arrays hold one scalar type.
