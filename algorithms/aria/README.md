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

| RFC 5794 section | Orange spec |
| --- | --- |
| 2.2, W0 through W3 | `feistel_w`, with `c1`, `c2`, `c3` |
| 2.2, ek1 through ek17 | `encryption_round_keys`, `rotate_right_19` and the other four rotations |
| 2.2, dk1 through dk(n+1) | `decryption_round_keys`, `decryption_round_key` |
| 2.2, KL, KR and CK1 through CK3 per key size | `key_schedule_128`, `key_schedule_192`, `key_schedule_256` |
| 2.3.1 and 2.3.2, the n-round process | `data_randomizing`; `encrypt_128` through `decrypt_256` |
| 2.4.1, FO and FE | `fo`, `fe` |
| 2.4.2, SB1 through SB4 and SL1, SL2 | `sb1` through `sb4`, `lookup`, `sl1`, `sl2` |
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
table-driven implementation, and the selection loops in this file are one,
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
substitution layer. Orange has no data-dependent index, so each lookup is the
packed-table idiom: a 256-entry table is 32 words of eight entries, the word
is chosen by comparing the index's high five bits against a loop index, and
the byte by a conditional on the low three. That is about 300 steps per
lookup and about 5,000 per round, and it is where nearly the whole cost of a
block goes: measured with the loop-and-binary-search method of the brief, one
encryption costs about 87,000 steps with a 128-bit key, 95,000 with 192 and
117,000 with 256 (12, 14 and 16 rounds plus the three Feistel rounds of the
key schedule, which cost about 18,000 on their own), and one decryption about
105,000, 117,000 and 131,000, the difference being the diffusion layer
applied to n - 1 round keys and the selection of each of them. Everything
else is cheap: the diffusion layer is 112 xors, and each 128-bit rotation of
the key schedule is four shifts and two ors on a pair of 64-bit words, one
spec per amount because a shift amount is a literal.

Two shapes in the file answer limits of the language rather than the RFC.
The round-key schedule is a `Word[64]^34` (seventeen pairs of words).
Seventeen 16-byte keys are 272 bytes, which fit in one array: the length
bound is 65,536, not 256. The source keeps the pairs it was written with,
and a round converts its pair back to bytes with `bytes_of`. And the
number of rounds is not a loop bound, since bounds are literals: the
`data_randomizing` loop runs sixteen times for every key size and compares
the round index with `n`, and `decryption_round_key` selects `ek(n+1-i)` by
comparison because `n - i` is not a static index. All eight vectors, three
encryptions, three decryptions and two Botan blocks, evaluate in one file at
about 856,000 of the 1,048,576 steps; nothing was moved or dropped.

## Dissemination

### Files

- `aria.or`: the whole cipher, sections 2.2 through 2.4 of RFC 5794 for all
  three key sizes, encryption and decryption, with the eight vector pairs.

### Running

    orangec eval algorithms/aria/aria.or
    python3 algorithms/verify.py algorithms/aria

### Vectors

| Spec | Source | Case |
| --- | --- | --- |
| `rfc5794_a1_128` | RFC 5794, Appendix A.1 | ARIA-128, key 000102...0f, plaintext 00112233...ff |
| `rfc5794_a2_192` | RFC 5794, Appendix A.2 | ARIA-192, key 000102...17, same plaintext |
| `rfc5794_a3_256` | RFC 5794, Appendix A.3 | ARIA-256, key 000102...1f, same plaintext |
| `rfc5794_a1_128_decrypt` | RFC 5794, Appendix A.1 | the A.1 ciphertext decrypted to the plaintext |
| `rfc5794_a2_192_decrypt` | RFC 5794, Appendix A.2 | the A.2 ciphertext decrypted to the plaintext |
| `rfc5794_a3_256_decrypt` | RFC 5794, Appendix A.3 | the A.3 ciphertext decrypted to the plaintext |
| `botan_aria_128_pdf_block_1` | Botan `src/tests/data/block/aria.vec`, `[ARIA-128]`, case "ARIA Test Vector PDF" | first block: key 00112233...ff, plaintext 11111111aaaaaaaa11111111bbbbbbbb |
| `botan_aria_256_pdf_block_1` | Botan `aria.vec`, `[ARIA-256]`, case "ARIA Test Vector PDF" | first block, key 00112233...ff repeated twice, same plaintext |

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
literals and the `diffusion` body in `aria.or` were generated by a script
from those checked tables, not typed.

There is no ARIA in the Python libraries available here, so a plain Python
reference written from the RFC's structure served as the intermediate
oracle; a second transcription of Botan's word-level code was written
independently, and both agree with every block of the 234 blocks in Botan's
`aria.vec` (all nine cases, including the multi-block ECB inputs) and with
the three examples of Appendix A. The Orange file matched all eight vectors
on its first evaluation, so the oracles were not needed round by round.

This entry is a reference evaluation of RFC 5794 under `orangec eval`. It
makes no constant-time, side-channel, performance or certification claim,
and it is not a corpus entry in the sense of The Orange Book chapter 12.

## Gaps

None that prevented any planned vector. The language limits met, and their
cost, are the ones described above: no data-dependent index, so each S-box
lookup is a selection of about 300 steps and a block costs 87,000 to 131,000
steps; the 17 round keys travel as `Word[64]^34` and are converted to bytes
per round, even though 272 bytes fit in one array (the bound is 65,536);
literal loop bounds, so the round count is a comparison inside a
16-iteration loop and the decryption round keys are selected rather than
indexed.
