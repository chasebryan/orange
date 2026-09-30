# DES and Triple DES (TDEA)

DES, the Data Encryption Standard, is a 64-bit block cipher with a 56-bit key
designed at IBM in 1973 and 1974 by a group around Horst Feistel, Walter
Tuchman and Don Coppersmith, from the earlier cipher Lucifer, in answer to a
call by the National Bureau of Standards; it was reviewed with the NSA,
published as FIPS 46 in January 1977 and reaffirmed as FIPS 46-1 (1988),
46-2 (1993) and [FIPS 46-3](https://csrc.nist.gov/pubs/fips/46-3/final)
(1999), which also endorsed Triple DES. NIST withdrew FIPS 46-3 in May 2005.
Triple DES, the Triple Data Encryption Algorithm, applies the DES engine three
times, encrypt-decrypt-encrypt, under two or three keys; NIST specifies it in
[SP 800-67 rev. 2](https://doi.org/10.6028/NIST.SP.800-67r2) (2017). DES was
the cipher of banking, of the first Internet security protocols and of a
generation of hardware; Triple DES outlived it in TLS, in Kerberos and above
all in payment systems. Their status today: single DES has been breakable by
exhaustive key search since 1998; Triple DES was deprecated by NIST in 2017,
its use for encryption was disallowed after 2023 by
[SP 800-131A rev. 2](https://doi.org/10.6028/NIST.SP.800-131Ar2), and TLS
1.3 carries no cipher suite for it.

## Analysis

### Structure

DES is a sixteen-round Feistel cipher. FIPS 46-3 numbers the bits of a block
1 through 64 from the left and writes every step as a table of those numbers.
The input block passes through the initial permutation IP and is split into a
left half L0 and a right half R0 of 32 bits. Iteration n computes
L_n = R_{n-1} and R_n = L_{n-1} xor f(R_{n-1}, K_n), with K_n the 48-bit
subkey of that iteration. After the sixteenth iteration the halves are
exchanged once to form the preoutput block R16 L16, and the inverse initial
permutation IP^-1 gives the output. Deciphering is the same procedure with the
subkeys in the reverse order, K16 first.

The cipher function f(R, K) expands R from 32 to 48 bits by the bit-selection
table E (each row of four bits borrows the bit on either side), adds K by
xor, cuts the 48 bits into eight six-bit blocks B1 through B8, and passes each
through its selection function S1 through S8: the outer bits b1 b6 of a block
name a row and the inner bits b2 b3 b4 b5 a column of a 4 x 16 table of
four-bit entries. The 32 bits S1(B1) ... S8(B8) go through the permutation P.
The S-boxes are the only non-linear part of the cipher; E, P, IP, IP^-1 and
the key schedule are wirings.

The key schedule KS takes the 64-bit key, of which bits 8, 16, ..., 64 are
parity bits, through permuted choice PC-1 to two 28-bit blocks C0 and D0. At
iteration n both blocks are rotated left by one place (n = 1, 2, 9, 16) or two
places (otherwise), and K_n is the permuted choice PC-2 of the 56 bits C_n
D_n. The sixteen subkeys are therefore sixteen selections of 48 of the 56 key
bits, and the schedule is linear.

TDEA, in SP 800-67 rev. 2 section 3, is the forward transformation
O = E_K3(D_K2(E_K1(I))) and the inverse transformation
O = D_K1(E_K2(D_K3(I))) on one 64-bit block under a key bundle (K1, K2, K3).
Keying option 1 takes three independent keys; keying option 2 sets K3 = K1.
With K1 = K2 = K3 the transformation collapses to single DES, which is how
Triple DES hardware interoperated with DES.

The Orange file follows the standard's own layout. A block, a half block, an
expanded half block and a key in flight are arrays of bits, `Word[8]^65`,
`Word[8]^33`, `Word[8]^49` and `Word[8]^57`, each holding 0 or 1 with position
0 unused, so that the standard's tables can be transcribed in the standard's
1-based numbering: the first row of IP is `b[58], b[50], b[42], ...` in the
file exactly as it is in the standard. Each S-box is four `Word[64]` words, one
per row, sixteen nibbles per word with column 0 in the most significant
nibble, so that `0xe4d12fb83a6c5907` reads as the first row of S1, `14 4 13 1
2 15 11 8 3 10 6 12 5 9 0 7`. The sixteen subkeys are more bits than one array
may hold (768 of at most 256), so the schedule delivers them as `Word[64]^16`
and each iteration unpacks its subkey back to bits. FIPS 46-3 does not number
the sections of its body; the table uses the standard's own headings.

| Standard section | Orange spec |
| --- | --- |
| FIPS 46-3, Enciphering: IP, the sixteen iterations, the preoutput block, IP^-1 | `initial_permutation`, `iteration`, `right_half`, `preoutput`, `inverse_initial_permutation`, `dea` |
| FIPS 46-3, The Cipher Function f: E, S1 through S8, P | `e_bit_selection`, `s1` through `s8`, `selection`, `nibble_at`, `selection_bits`, `permutation`, `f` |
| FIPS 46-3, Key Schedule Calculation: PC-1, the left shifts, PC-2, KS | `permuted_choice_1`, `left_shifts`, `left_shift`, `permuted_choice_2`, `schedule_iteration`, `key_schedule`, `pack_c`, `pack_d`, `cd_bits`, `pack48`, `subkey_bits` |
| FIPS 46-3, Deciphering: the subkeys in reverse order | `reversed_schedule`, `decrypt` (and `encrypt`) |
| FIPS 46-3, the tables of the body and of Appendix 1 (Primitive Functions for the Data Encryption Algorithm): IP, IP^-1, E, S1 to S8, P, PC-1, PC-2 and the shift schedule | the array literals of the specs above |
| SP 800-67 rev. 2, section 3: the forward and inverse transformations, keying options 1 and 2 | `tdea_encrypt`, `tdea_decrypt` |
| SP 800-67 rev. 2, Appendix B: the example, three blocks in ECB | `tdea_encrypt_blocks`, `sp800_67_b_tdea` (in `des.or`) |

The bit and byte helpers are `bits64` and `bytes64`; bit 1 of the standard is
the most significant bit of the first byte, the convention of every DES
implementation and vector file.

### Security status

**The key.** The 56-bit key was the controversy of DES from the start: Diffie
and Hellman argued in 1977 that a purpose-built machine costing about twenty
million dollars could find a key in a day, and Wiener's 1993 design brought
the estimate to a million dollars and hours. The estimates became machines.
The DESCHALL project found an RSA challenge key by distributed search in 96
days in 1997; the Electronic Frontier Foundation's Deep Crack, 1,856 custom
chips built for about 250,000 dollars, found a key in 56 hours in July 1998
and, together with distributed.net, in 22 hours in January 1999. COPACOBANA
(Kumar, Paar, Pelzl, Pfeiffer and Schimmler, 2006) did the same with 120
commodity FPGAs for under 10,000 dollars in about a week on average. As of
2026 a full search of the 2^56 keys is a matter of hours to a day on a rack
of FPGAs or on rented GPUs, at a cost of hundreds to a few thousand dollars;
the exact figures of the commercial services that offer it were not checked
from this machine. Single DES protects nothing against a determined
adversary and has not since 1998.

**Differential and linear cryptanalysis.** Biham and Shamir introduced
differential cryptanalysis on reduced DES in 1990 and broke the full sixteen
rounds in 1992 with 2^47 chosen plaintexts, the first attack faster than
exhaustive search. Coppersmith disclosed in 1994 that IBM had known the
technique in 1974 and had chosen the S-boxes and P to resist it, keeping the
criteria secret at the NSA's request; the NSA's part in the design, and the
reduction of Lucifer's key to 56 bits, were investigated by a United States
Senate committee in 1978, which reported no evidence that the S-boxes had
been weakened. Matsui's linear cryptanalysis (1993, refined in 1994) needs
2^43 known plaintexts and was carried out in practice in 1994 on twelve
workstations in fifty days; it remains the best known shortcut attack on DES,
and it is not practical against a cipher that changes keys, since 2^43
blocks are 64 terabytes of known plaintext under one key. Davies' attack, as
improved by Biham and Biryukov (1997), needs about 2^50 known plaintexts. The
design's margin against these attacks is thin but real: the attacks work
because DES has only sixteen rounds, and they are why every later cipher was
designed with published resistance criteria.

**Structural properties.** DES has the complementation property
DES_{~K}(~P) = ~DES_K(P), since complementing both the key and the block
complements E(R) and K_n alike and leaves their xor unchanged; it halves an
exhaustive search to 2^55 when the attacker can obtain the encryptions of a
plaintext and its complement (`botan_des_case_2_complement` reproduces the
property on a published vector). Four keys are weak: their C0 and D0 are each
all zeros or all ones, so every subkey is the same and encryption is an
involution, E_K(E_K(P)) = P. Twelve keys are semi-weak, in six pairs: their
C0 and D0 are each constant or alternating, so a key has only two distinct
subkeys and its partner has the same subkeys in the reverse order, and
E_K2(E_K1(P)) = P. With odd parity, as FIPS 46-3 requires, they are:

```text
weak keys
  0101010101010101   1f1f1f1f0e0e0e0e   e0e0e0e0f1f1f1f1   fefefefefefefefe

semi-weak key pairs
  011f011f010e010e / 1f011f010e010e01     01e001e001f101f1 / e001e001f101f101
  01fe01fe01fe01fe / fe01fe01fe01fe01     1fe01fe00ef10ef1 / e01fe01ff10ef10e
  1ffe1ffe0efe0efe / fe1ffe1ffe0efe0e     e0fee0fef1fef1fe / fee0fee0fef1fef1
```

The list was derived from the key schedule by script (the sixteen keys whose
C0 and D0 are each constant or alternating), checked for the involution
properties with the Python reference and with pycryptodome, and agrees with
the list of FIPS 74 (1981) and SP 800-67 rev. 2, which forbids these keys in
a TDEA key bundle. `weak_key_0101_involution` reproduces the involution under
the first of them. The weak keys are a property of the schedule, not an
attack: a random key is weak with probability 2^-52.

**Triple DES.** Double encryption is no answer to the short key, because the
meet-in-the-middle attack of Diffie and Hellman (1977) breaks it in about
2^57 operations with 2^56 memory; hence triple encryption. Three-key TDEA has
a 168-bit key but at most 112 bits of security, the meet-in-the-middle bound
between the first and the last two encryptions (with a few known plaintexts
and 2^56 memory), and Lucks (1998) brought the time slightly below 2^112 with
a large table; NIST rates it at 112 bits. Two-key TDEA has a 112-bit key but
less security: Merkle and Hellman (1981) break it with 2^56 chosen
plaintexts, and van Oorschot and Wiener (1990) with 2^t known plaintexts and
about 2^{120-t} work, which is why NIST rated it at 80 bits and disallowed
it for encryption after 2015. The 64-bit block is the other limit: any mode
of operation leaks after about 2^32 blocks under one key (the birthday
bound), and Sweet32 (Bhargavan and Leurent, 2016) turned this into the
recovery of an HTTP session cookie from a long-lived Triple DES TLS
connection with about 785 GB of traffic. In response SP 800-67 rev. 2 (2017)
limited a key bundle to 2^20 blocks, and browsers and servers removed the
Triple DES cipher suites.

**Standing.** FIPS 46-3 was withdrawn on 19 May 2005, after AES (FIPS 197,
2001) replaced it. NIST proposed the deprecation of TDEA in 2017 and fixed it
in SP 800-131A rev. 2 (March 2019): TDEA encryption is deprecated through
2023 and disallowed after 31 December 2023, and decryption is allowed only
for legacy use; NIST has since announced the withdrawal of SP 800-67 rev. 2
itself (the date was not checked from this machine). TLS 1.3 (RFC 8446,
2018) defines no cipher suite with Triple DES, and the IETF's TLS
recommendations (BCP 195) advise against the TLS 1.2 suites that use it.
Triple DES survives where hardware and formats are slow to change, notably in
the payment card industry, which is migrating its key blocks and PIN
encryption to AES. Nothing here is a claim about a particular deployment;
this entry evaluates the algorithm as specified.

### What the Orange rendering shows

The rendering makes the structure of DES legible in a way a word-oriented
implementation does not. Every table of the standard is an array literal of
its bit numbers, so IP, E, P, PC-1 and PC-2 read in the file exactly
as they read in the standard, and a reader can see that they are wirings: no
arithmetic, no key, no data decides where a bit goes. The one data-dependent
step of the whole cipher is the S-box lookup, and it is the one place where
the file departs from a table: an index in Orange must be static, so the row
is a four-arm conditional on b1 b6 and the column a sixteen-arm conditional
on b2 b3 b4 b5 (`selection`, `nibble_at`), the row and column addressing of
the standard written out. The key schedule is visibly linear: PC-1, a
rotation by a count read from the shift table, PC-2, and no other operation.
Decryption is the encryption spec with `reversed_schedule`, and TDEA is three
calls.

Two departures from the standard's text are forced by the language and said
where they happen. The sixteen subkeys cannot travel as one array of bits
(768 entries, above the limit of 256), so `pack48` and `subkey_bits` move
each subkey through a `Word[64]`, and the schedule's loop carries C, D and
the subkeys found so far in one `Word[64]^18`, with C and D as 28-bit words
whose left shifts are 28-bit rotations; a loop's step is a single expression
that may not store at an index it computes, so each new subkey is appended at
the end of that array rather than written at position n. Neither changes a
value; both are stated in comments.

Measured under `orangec eval`, one call of `dea` (sixteen iterations, each
about 2,200 steps, of which f is about 1,800 and the eight S-box selections
about half of that) costs about 36,000 steps, a key schedule about 14,000,
and so an `encrypt` or `decrypt` about 50,000. A single-block TDEA
transformation, three schedules and three DEA calls, costs about 150,000, and
the Appendix B example, three schedules and nine DEA calls through
`tdea_encrypt_blocks`, about 375,000. The thirteen vector pairs of the entry
come to about 1.5 million steps, more than the 1,048,576 of one file, so they
are split across two files: `des.or` holds six pairs, about 845,000 steps,
with room for four more encryptions, and `des-vectors.or` seven pairs, about
690,000 steps, with room for seven. Nothing was dropped. Not expressed:
constant-time behaviour (the selection idiom specifies a lookup, it does not
claim anything about leakage); the parity check of the key, which FIPS 46-3
describes and no vector exercises; and the modes of operation of SP 800-38A
beyond the three-block ECB of the Appendix B example.

## Dissemination

### Files

- `des.or`: the Data Encryption Algorithm (IP, E, S1 to S8, P, PC-1, PC-2,
  the shift schedule, KS, the sixteen iterations, IP^-1), encryption and
  decryption, the TDEA forward and inverse transformations, the three-block
  ECB helper for the Appendix B example, and six vector pairs: the SP 800-67
  rev. 2 Appendix B example, two single-DES cases of Botan (one decrypted),
  one of OpenSSL, a two-key TDEA case and a three-key TDEA decryption.
- `des-vectors.or`: the same algorithm, spec for spec, without the
  three-block helper, and seven vector pairs that do not fit in the first
  file's step budget: two single-DES cases of OpenSSL, the complementation
  property, a weak key, a three-key TDEA case of Botan, and the DES-EDE3-ECB
  and DES-EDE-ECB cases of OpenSSL (the latter decrypted).

### Running

```console
orangec eval algorithms/des/des.or
orangec eval algorithms/des/des-vectors.or
python3 algorithms/verify.py algorithms/des
```

`eval` also prints the eight S-box tables and the shift table, since they
are parameterless specs.

### Vectors

| Spec | File | Source | Case |
| --- | --- | --- | --- |
| `sp800_67_b_tdea` | `des.or` | SP 800-67 rev. 2, Appendix B; value from pycryptodome `DES3` (ECB), confirmed by `cryptography` `TripleDES` | three-key TDEA of `The qufck brown fox jump` (the standard's spelling) under 0123456789abcdef, 23456789abcdef01, 456789abcdef0123: a826fd8ce53b855f cce21c8112256fe6 68d5c05dd9b6b900 |
| `botan_des_case_2` | `des.or` | Botan `src/tests/data/block/des.vec`, `[DES]`, second case (lines 6 to 8) | key 0123456789abcdef, block `Now is t` = 4e6f772069732074, ciphertext 3fa40e8a984d4815 (the first block of the FIPS 81 example) |
| `botan_des_case_1_decrypt` | `des.or` | Botan `des.vec`, `[DES]`, first case (lines 2 to 4) | decryption: key 0113b970fd34f2ce, ciphertext 86a560f10ec6d85b gives 059b5e0851cf143a |
| `openssl_des_ecb_7` | `des.or` | OpenSSL `test/recipes/30-test_evp_data/evpciph_des.txt`, seventh `DES-ECB` case (lines 48 to 51) | key fedcba9876543210, plaintext 0123456789abcdef, ciphertext ed39d950fa74bcc4 |
| `botan_tripledes_case_1` | `des.or` | Botan `des.vec`, `[TripleDES]`, first case (lines 1036 to 1038) | two-key TDEA (keying option 2): K1 = 0123456789abcdef, K2 = fedcba9876543210, block 0123456789abcde7, ciphertext 7f1d0a77826b8aff |
| `botan_tripledes_case_20_decrypt` | `des.or` | Botan `des.vec`, `[TripleDES]`, case 20 (lines 1112 to 1114) | three-key TDEA decryption: K1 = 0123456789abcdef, K2 = 5555555555555555, K3 = fedcba9876543210, ciphertext 18d748e563620572 gives `somedata` = 736f6d6564617461 |
| `openssl_des_ecb_1` | `des-vectors.or` | OpenSSL `evpciph_des.txt`, `DES-ECB` case 1 (lines 12 to 15) | key 0000000000000000, plaintext 0000000000000000, ciphertext 8ca64de9c1b123a7 |
| `openssl_des_ecb_2` | `des-vectors.or` | OpenSSL `evpciph_des.txt`, `DES-ECB` case 2 (lines 18 to 21) | key ffffffffffffffff, plaintext ffffffffffffffff, ciphertext 7359b2163e4edc58 |
| `botan_des_case_2_complement` | `des-vectors.or` | the complement of Botan `des.vec`, `[DES]`, second case (lines 6 to 8); confirmed by pycryptodome `DES` | key fedcba9876543210 and block b19088df968cdf8b (the case's key and block complemented), ciphertext c05bf17567b2b7ea (the case's output complemented) |
| `weak_key_0101_involution` | `des-vectors.or` | the weak key 0101010101010101 on the block of Botan `des.vec`, `[DES]`, second case; confirmed by pycryptodome `DES` | E_K(E_K(P)) = P for P = 4e6f772069732074 |
| `botan_tripledes_case_20` | `des-vectors.or` | Botan `des.vec`, `[TripleDES]`, case 20 (lines 1112 to 1114) | three-key TDEA: K1 = 0123456789abcdef, K2 = 5555555555555555, K3 = fedcba9876543210, block `somedata` = 736f6d6564617461, ciphertext 18d748e563620572 |
| `openssl_des_ede3_ecb_block_1` | `des-vectors.or` | OpenSSL `evpciph_des3_common.txt`, `DES-EDE3-ECB` (lines 29 to 32), first of four blocks | three-key TDEA: key 0123456789abcdef f1e0d3c2b5a49786 fedcba9876543210, block "7654321 " = 3736353433323120, ciphertext 62c10cc9efbf15aa |
| `openssl_des_ede_ecb_block_1_decrypt` | `des-vectors.or` | OpenSSL `evpciph_des3_common.txt`, `DES-EDE-ECB` (lines 36 to 39), first of four blocks | two-key TDEA decryption: key 0123456789abcdef fedcba9876543210, ciphertext 4d1332e49f380e23 gives "7654321 " = 3736353433323120 |

The Appendix B ciphertext is the only value that comes from an oracle rather
than from a fetched file: SP 800-67 rev. 2 could not be read from the machine
this entry was written on, so its plaintext and keys were taken from the task
description and the ciphertext computed with pycryptodome `DES3` and
confirmed with the `cryptography` package; the writer's recollection of the
standard's printed value agrees with it, and a reader with the standard should
compare. Every other expected value is copied from the cited line of the
cited file and was re-checked against pycryptodome before it was written.
Botan's TripleDES section holds the vectors with 16-byte keys as two-key
bundles; Botan's `3des.vec` does not exist (the name returns 404), and the
Triple DES cases are in `des.vec`. "Now is the time for all " under
0123456789abcdef, the example of FIPS 81 (1980), encrypts in ECB to
3fa40e8a984d4815 6a271787ab8883f9 893d51ec4b563b53 with the Python
reference; Botan's second and third `[DES]` cases hold its first block and
its last two blocks (the third case lists them in the order "for all ",
"he time "), and the entry reproduces the first block.

### Provenance and claims

Neither FIPS 46-3 nor SP 800-67 rev. 2 could be read from the machine this
entry was written on, so every table was taken from fetched reference
implementations and cross-checked by script, never transcribed by eye or from
memory:

- IP, IP^-1, E, P, PC-1, PC-2, the shift schedule and the eight S-boxes were
  extracted from `pyDes.py` (Todd Whiteman's pure-Python DES, which holds the
  tables 0-based) and from Go's `crypto/des/const.go` (which holds the
  permutations in its own reversed numbering and the S-boxes in the
  standard's 4 x 16 layout), converted to the standard's 1-based numbering,
  and required to agree; the script also checks that IP^-1 inverts IP and
  that every S-box row is a permutation of 0 through 15.
- A Python reference in the standard's vocabulary (bit vectors with position
  0 unused, IP, E, S_i, P, PC-1, PC-2, LS_n, KS, the preoutput block, and the
  TDEA forward and inverse transformations), built from those tables,
  reproduces all 258 DES and 57 TripleDES cases of Botan's `des.vec` in both
  directions, the ten ECB cases of OpenSSL's `evpciph_des.txt` and
  `evpciph_des3_common.txt`, and agrees with pycryptodome `DES` and `DES3`
  and with `cryptography`'s `TripleDES` on 200 random keys.
- The Orange array literals were generated by script from the extracted
  tables (the S-box words by packing each row's sixteen entries), and the
  vector specs by a second script that cites each case's file and lines and
  checks each expected value against pycryptodome before writing it. The
  thirteen vectors matched on the first evaluation.
- The weak and semi-weak keys were derived from the schedule and checked as
  described above.
- The section names in the files' comments are FIPS 46-3's own headings
  (Enciphering; The Cipher Function f; Key Schedule Calculation; Deciphering;
  Appendix 1, Primitive Functions for the Data Encryption Algorithm) and
  SP 800-67 rev. 2's section 3 and Appendix B, as recalled; a reader with
  the standards should check them against the text.

The extraction, generation, measurement and weak-key scripts were kept with
the work record and are not part of the repository.

This entry is a reference evaluation of FIPS 46-3 and SP 800-67 rev. 2 under
`orangec eval`. It makes no constant-time, side-channel, performance or
certification claim, and it is not a corpus entry in the sense of The Orange
Book chapter 12.

## Gaps

None that prevented anything. Three features of the language shaped the
files:

- Indices are static, so a bit permutation cannot be a loop over a position
  table; each table is an array literal of literal-index selections, which
  reads like the standard but costs one step per bit plus the literal, and
  the S-box lookup is a four-arm and a sixteen-arm conditional (about 110
  steps per S-box, about 900 of the 1,800 steps of f).
- An array holds at most 256 scalars, so the sixteen 48-bit subkeys are
  packed into `Word[64]` words and unpacked in every iteration (about 240
  steps of the 2,200 of an iteration).
- A loop's step is one expression without bindings and cannot store at a
  computed index, so the schedule appends each subkey to a rotating array
  rather than writing K_n at position n.

The step budget of one file (1,048,576) holds about twenty DES encryptions
with their key schedules, so the thirteen vectors are split across `des.or`
and `des-vectors.or` as recorded above; the second file repeats the whole
algorithm because a module has no imports.
