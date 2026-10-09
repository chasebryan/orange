# ARIA

ARIA is a 128-bit block cipher with 128-, 192- and 256-bit keys, designed by a
group of Korean cryptographers (Kwon, Kim, Park, Sung and their colleagues) and
presented at ICISC 2003; the revised version with 12, 14 and 16 rounds became the Korean
national standard KS X 1213 in 2004 and is described for the IETF in
[RFC 5794](https://www.rfc-editor.org/rfc/rfc5794) (2010), whose TLS cipher
suites are in [RFC 6209](https://www.rfc-editor.org/rfc/rfc6209). It is an
involutional substitution-permutation network that shares the AES S-box, and
it is used in Korean government and public-sector systems and shipped by
OpenSSL and Botan. It is a current standard: no attack on the full cipher is
published, and the best reduced-round results stop at 7 or 8 rounds.

## Analysis

### Structure

The state is sixteen bytes. A round xors a 128-bit round key into the state,
sends each byte through one of four S-boxes, and mixes the sixteen bytes with
a fixed binary matrix. The two round functions of RFC 5794 section 2.4.1,
`FO(D, RK) = A(SL1(D ^ RK))` for odd rounds and `FE(D, RK) = A(SL2(D ^ RK))`
for even rounds, differ only in the order of the S-boxes: SL1 applies SB1,
SB2, SB3, SB4 across each group of four bytes and SL2 applies SB3, SB4, SB1,
SB2. SB1 is the AES S-box (inversion in GF(2^8) followed by AES's affine
map), SB2 is x^247 in the same field followed by another affine map, and SB3
and SB4 are their inverses, so SL2 is the inverse of SL1. The diffusion layer
A is a 16 x 16 matrix over GF(2) with seven ones in each row, given in the RFC
as sixteen xor equations; it is an involution with branch number 8. Because A
is an involution and the substitution layers alternate with their inverses,
decryption is the encryption process run with the round keys reversed and
passed through A (section 2.2), and the file has one `data_randomizing`
spec for both directions.

The key schedule (section 2.2) splits the master key into `KL || KR` with
`KR` zero-padded, runs three rounds of a 256-bit Feistel cipher on it with
the constants C1, C2, C3 (the first 384 bits of the fractional part of 1/pi,
assigned to CK1, CK2, CK3 in an order that depends on the key size) to get
W0 through W3, and forms the round keys ek1 through ek17 by xoring pairs of
the W values with 128-bit rotations by 19, 31, 61, 31 and 19 bits. A 128-bit
key uses ek1 through ek13 for 12 rounds, a 192-bit key ek1 through ek15 for
14, a 256-bit key all 17 for 16; the last round replaces the diffusion layer
by an extra key addition.

Every 128-bit string of the RFC (a data block, a round key, KL, KR, W0
through W3, C1 through C3) is a `Block`, sixteen bytes with the leftmost
first; each S-box is an `SBox` of 256 bytes; the seventeen encryption round
keys are `RoundKeys`, an array of seventeen blocks, with ek(i+1) at index i.
Two size parameters carry the key sizes. The master key of `key_schedule`,
`encrypt` and `decrypt` is a `Word[8]^(8 * k)` for k in 2 through 4, that is
16, 24 or 32 bytes, so each is one spec over the three key sizes. The round
count n = 2h is a size of `data_randomizing` and `decryption_keys`, for h in
6 through 8, which take the n + 1 round keys rk1 through rk(n+1) as a
`Block^(2 * h + 1)`; the round count follows from the number of keys passed,
and only the RFC's 12, 14 and 16 rounds have an instance.

| RFC 5794 section | Orange spec |
| --- | --- |
| 2.1, the 128-bit operations ^, >>> and <<< | `xor`, `rotate_right`, `rotate_left` |
| 2.2, C1, C2, C3 | `c1`, `c2`, `c3` |
| 2.2, W0 through W3 and ek1 through ek17 | `round_keys` |
| 2.2, KL, KR and CK1 through CK3 per key size | `key_schedule[k]` |
| 2.2, dk1 through dk(n+1) | `decryption_keys[h]` |
| 2.3.1 and 2.3.2, the n-round process | `data_randomizing[h]`; `encrypt[k]`, `decrypt[k]` |
| 2.4.1, FO and FE | `fo`, `fe` |
| 2.4.2, SB1 through SB4 and SL1, SL2 | `sb1` through `sb4`, `sl1`, `sl2` |
| 2.4.3, A | `diffusion` |

### Security status

ARIA was evaluated at the request of the Korean standards body before its
adoption; the report of Biryukov, De Canniere, Lano, Ors and Preneel (2004)
found dedicated linear and truncated differential attacks on up to 7 rounds
of the ICISC 2003 version and led to the S-box arrangement and the round
counts of the 2004 standard, which RFC 5794 describes. Since then the
published attacks have stayed in the reduced-round setting: impossible
differentials on 6 rounds (Wu, Zhang and Feng, 2007) and later 7 rounds,
boomerang attacks on 5 and 6 rounds (Fleischmann, Gorski and Lucks, 2009),
integral attacks on 6 and 7 rounds (Li, Wu and Zhang, 2010), and
meet-in-the-middle attacks on 7 rounds (Tang, Sun, Li, Qu and Zheng, 2011)
and on 8 rounds of ARIA-256 (Akshima, Chang, Ghosh, Goel and Sanadhya,
2015), the last with time within a small factor of exhaustive search of the
256-bit key. No published attack known to the author of this entry reaches
8 rounds of ARIA-128. The full cipher therefore keeps a margin of at least
four rounds for every key size, similar to the margin
of AES-128 against its best reduced-round attacks.

The comparison with AES is the natural one: both are 128-bit byte-oriented
SPNs and ARIA's SB1 is the AES S-box, but ARIA replaces the MDS MixColumns
by a binary involutory matrix with branch number 8 over all sixteen bytes,
which costs it more rounds (12 against 10) and buys an involutional structure
in which encryption and decryption share one circuit. As with AES, a
table-driven implementation, and the table lookups in this file are one,
leaks through cache timing; the RFC makes no constant-time provision.

Standing: KS X 1213-1 remains the Korean block-cipher standard; RFC 5794 is
Informational and has not been obsoleted; the TLS 1.2 suites of RFC 6209 are
registered but are not among the TLS 1.3 suites, and no NIST or IETF
deprecation applies to ARIA itself (this paragraph is current as of September
2026). The pitfalls are those of any 128-bit block cipher: modes, nonces and
padding are the caller's responsibility, and the birthday bound of 2^64
blocks per key applies to CBC and CTR use.

### What the Orange rendering shows

The only data-dependent choices in ARIA are the sixteen S-box lookups of each
substitution layer, and the file writes them as the RFC does: `sl1` is the
array `[s1[x[0]], s2[x[1]], s3[x[2]], s4[x[3]], ...]`, each `s[x]` indexing a
256-byte table by a data byte, which the checker proves in range because a
`Word[8]` index cannot leave 0 through 255. The tables are `hex"..."` rows of
sixteen bytes, laid out as section 2.4.2 prints them. Everything else is
fixed by the RFC's text: the diffusion layer is its sixteen equations over
literal indices, the round keys are the seventeen equations of section 2.2,
and every loop runs over a count known when the file is checked, so the
only conditionals are the alternation of FO and FE by round parity and the
choice, in `decryption_keys`, of the two round keys that bypass A.

Byte order is written where the RFC fixes it. A 128-bit string's leftmost
byte is its most significant, so the two 128-bit operations of section 2.1
read a block `as big Word[64]^2`: xor is two 64-bit xors, and a rotation by
n moves n bits between the two halves, written once for 0 <= n <= 64 and
called with the amounts the RFC prints (`rotate_right(w1, 19)`,
`rotate_left(w1, 61)`). The key size is a size: `key_schedule` takes a key
of 8k bytes, builds KL || KR as the RFC defines it, the key followed by
zeros, with a join and two slices (`mk ++ [0; 16]`, then bytes 0 through 15
and 16 through 31), and chooses CK1 through CK3 from the RFC's table with
`if k == 2`, `else if k == 3`, `else`. The round count is a size too:
`encrypt` passes ek1 through ek(2k+9) as `key_schedule(mk)[..(2 * k + 9)]`,
and `data_randomizing` runs twelve rounds for a 128-bit key because it was
given thirteen keys. The tests call `encrypt` and `decrypt` without
brackets; the length of the key picks k.

Measured costs under `orangec test --stats`: building one S-box table costs
about 55 steps, and each substitution layer builds all four again, so a
layer costs about 320 (four tables and sixteen lookups); the diffusion
layer costs about 335, a 128-bit xor or rotation 22 to 32, and one round
about 680. The key schedule costs about 3,100 steps for any key size, two
thirds of it the three Feistel rounds; the decryption round keys add about
4,000 for 12 rounds, almost all of it A on the inner keys. One encryption,
key schedule included, costs 11,151 steps with a 128-bit key, 12,546 with
192 and 13,935 with 256; one decryption 15,211, 17,334 and 19,451. The
eight tests together use 114,714 steps.

## Dissemination

### Files

- `aria.or`: the whole cipher, sections 2.1 through 2.4 of RFC 5794 for all
  three key sizes, encryption and decryption, with the eight tests below.

### Running

    orangec test algorithms/aria/aria.or
    python3 algorithms/verify.py algorithms/aria

### Vectors

Each row is a `test` block in `aria.or`, comparing a block with the
published value.

| Test | Source | Case |
| --- | --- | --- |
| `RFC 5794 A.1: ARIA-128 encrypts` | RFC 5794, Appendix A.1 | ARIA-128, key 000102...0f, plaintext 00112233...ff |
| `RFC 5794 A.2: ARIA-192 encrypts` | RFC 5794, Appendix A.2 | ARIA-192, key 000102...17, same plaintext |
| `RFC 5794 A.3: ARIA-256 encrypts` | RFC 5794, Appendix A.3 | ARIA-256, key 000102...1f, same plaintext |
| `RFC 5794 A.1: ARIA-128 decrypts` | RFC 5794, Appendix A.1 | the A.1 ciphertext decrypted to the plaintext |
| `RFC 5794 A.2: ARIA-192 decrypts` | RFC 5794, Appendix A.2 | the A.2 ciphertext decrypted to the plaintext |
| `RFC 5794 A.3: ARIA-256 decrypts` | RFC 5794, Appendix A.3 | the A.3 ciphertext decrypted to the plaintext |
| `Botan aria.vec [ARIA-128] ARIA Test Vector PDF: block 1` | Botan `src/tests/data/block/aria.vec`, `[ARIA-128]`, case "ARIA Test Vector PDF" | first block: key 00112233...ff, plaintext 11111111aaaaaaaa11111111bbbbbbbb |
| `Botan aria.vec [ARIA-256] ARIA Test Vector PDF: block 1` | Botan `aria.vec`, `[ARIA-256]`, case "ARIA Test Vector PDF" | first block, key 00112233...ff repeated twice, same plaintext |

Every expected value is copied from the named source; no row was produced
by an oracle.

### Provenance and claims

The four S-box tables were extracted by script from Botan's
`src/lib/block/aria/aria.cpp` and checked, also by script, to be
permutations with SB3 and SB4 the inverses of SB1 and SB2, to equal the
tables printed in section 2.4.2 of a plain-text copy of RFC 5794, and to
have the algebraic form the RFC gives: SB1 was recomputed as the AES S-box
and matched, and SB2 was verified to be an affine image of x^247 over
GF(2^8) modulo x^8 + x^4 + x^3 + x + 1 by solving for the affine map. The
sixteen equations of the diffusion layer were taken from the RFC's text and
checked against the linear part of Botan's word-oriented FO and FE on the
unit vectors. The constants C1, C2, C3 are the RFC's. The packed `Word[64]`
literals and the `diffusion` body of the first form of `aria.or` were
generated by a script from those checked tables, not typed.

There is no ARIA in the Python libraries available here, so a plain Python
reference written from the RFC's structure served as the intermediate
oracle; a second transcription of Botan's word-level code was written
independently, and both agree with every block of the 234 blocks in Botan's
`aria.vec` (all nine cases, including the multi-block ECB inputs) and with
the three examples of Appendix A. The Orange file matched all eight vectors
on its first evaluation, so the oracles were not needed round by round.

The entry was then rewritten in the current language. Every expected value
is carried over byte for byte from the first form, where each was a
`<name>_expected` spec of sixteen bytes; the new tests state the same bytes
as `hex"..."` literals, and the keys and inputs are those of the first form.
No vector was added or dropped. The `hex"..."` rows of the S-box tables
were printed by a script from the first form's packed `Word[64]` literals,
and the script checked again that each table is a permutation, that SB3 and
SB4 invert SB1 and SB2, and that SB1 equals the AES S-box recomputed from
its definition. The diffusion equations and the constants C1, C2, C3 are
unchanged. The key schedule, encryption and decryption, which the first
form wrote as three specs each, one per key size, are now one spec each
over a size parameter.

This entry is a reference evaluation of RFC 5794 under `orangec test`. It
makes no constant-time, side-channel, performance or certification claim,
and it is not a corpus entry in the sense of The Orange Book chapter 12.

## Gaps

None that prevented any planned vector. What the language still shapes:

- There is no 128-bit word, so the 128-bit rotations of section 2.2 are
  written on two 64-bit halves, for amounts up to 64; `rotate_left` serves
  the RFC's `<<< 61`, `<<< 31` and `<<< 19` and `rotate_right` its `>>> 19`
  and `>>> 31`.
- Only single blocks are expressed; the multi-block ECB inputs of Botan's
  `aria.vec` and modes of operation are outside this entry.
