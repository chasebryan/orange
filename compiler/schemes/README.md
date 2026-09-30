# Orange sealing schemes

`orangec enc` seals a file with an authenticated cipher written in Orange, and
`orangec dec` opens it again. The programs in this folder are the built-in
schemes. Any other Orange program with the [interface](#the-scheme-interface)
below is a scheme too: name its path with `--scheme`.

Every byte of cryptography is computed by an Orange program on the reference
evaluator. `orangec` itself only reads keys and operating-system randomness,
lays out the [sealed-file format](#sealed-file-format-1), and calls the
program's `seal`, `authentic`, and `open` specifications once per chunk.

This is reference code. The evaluator is not constant-time, nothing here is
verified, and keys are stored unencrypted; read
[Security notes](#security-notes) before sealing anything that matters.

## Use

```sh
orangec keygen                         # a key for the default scheme, in ~/.config/orange/key
orangec enc report.pdf                 # writes report.pdf.orange
orangec dec report.pdf.orange          # writes report.pdf, and nothing unless all of it is authentic
orangec schemes                        # lists the built-in schemes

orangec keygen --scheme ascon_aead128 -o ascon.key
orangec enc --key ascon.key -o report.sealed report.pdf
orangec dec --key ascon.key -o report.pdf report.sealed

orangec keygen --scheme ./my_cipher.or -o my.key
orangec enc --key my.key --scheme ./my_cipher.or data.bin
orangec dec --key my.key --scheme ./my_cipher.or data.bin.orange
```

A key belongs to one scheme, and `enc` seals with the scheme its key names.
`dec` reads the scheme from the sealed file and refuses a key made for another.
A scheme that is not built in is named by its path on every command; a
`--scheme` argument that contains `/` or ends in `.or` is a path.

The default key is `$XDG_CONFIG_HOME/orange/key`, or `$HOME/.config/orange/key`
when `XDG_CONFIG_HOME` is not set. `-o` names the output: `enc` writes
`FILE.orange` by default and `dec` writes its input's name without `.orange`.
Neither command ever replaces an existing file.

## Built-in schemes

| Scheme | Standard | Key | Nonce | Tag | Chunk |
| --- | --- | --- | --- | --- | --- |
| `xchacha20_poly1305` (default) | draft-irtf-cfrg-xchacha-03 over RFC 8439 | 256 bits | 192 bits | 128 bits | 240 bytes |
| `chacha20_poly1305` | RFC 8439 | 256 bits | 96 bits | 128 bits | 240 bytes |
| `ascon_aead128` | NIST SP 800-232 | 128 bits | 128 bits | 128 bits | 240 bytes |

Each program ends with its known answers, specifications without parameters
that `orangec eval` evaluates and that must all be `true`:

- `xchacha20_poly1305.or`: the ChaCha20 block of RFC 8439 section 2.3.2, the
  HChaCha20 vector of the draft's section 2.2.1, and a whole sealed chunk;
- `chacha20_poly1305.or`: the ChaCha20 block of section 2.3.2, the one-time
  Poly1305 key of section 2.6.2, and a whole sealed chunk; and
- `ascon_aead128.or`: counts 1 and 545 of `LWC_AEAD_KAT_128_128.txt`, the
  known-answer file of the reference implementation ascon-c, and a whole sealed
  chunk.

Each whole-chunk answer seals 240 bytes of plaintext with 64 bytes of
associated data and compares the result with a value computed by an independent
implementation (pycryptodome 3.23.0 for the ChaCha schemes, and pyascon, the
Python reference implementation for SP 800-232, for Ascon); it also checks that
`authentic` accepts that value and rejects it with one bit flipped, and that
`open` recovers the plaintext. During development each scheme also matched the
independent implementation on 42 random and edge-case chunks, and every
single-bit change to the ciphertext, tag, associated data, or nonce tried was
rejected. `crates/orangec/tests/crypt.rs` opens files sealed by an independent
Python implementation of format 1 for every built-in scheme.

The evaluator runs about 110,000 to 135,000 steps to seal one chunk (`orangec
schemes` prints the exact count). `orangec` evaluates chunks on every core of
the machine, one evaluator per core; on four cores, sealing or opening a
megabyte takes about two seconds.

## The scheme interface

```text
edition 2026;
module NAME {
  spec seal(key: Word[8]^K, nonce: Word[8]^N, ad: Word[8]^64, plaintext: Word[8]^C) -> Word[8]^S
  spec open(key: Word[8]^K, nonce: Word[8]^N, ad: Word[8]^64, sealed: Word[8]^S) -> Word[8]^C
  spec authentic(key: Word[8]^K, nonce: Word[8]^N, ad: Word[8]^64, sealed: Word[8]^S) -> Bool
}
```

- The module's name is the scheme's name: 1 to 24 ASCII letters, digits, or
  underscores. It is written into every sealed file.
- `orangec` reads the sizes from these signatures. `S = C + T`, where `T` is
  the tag length. Keys are 16 to 64 bytes, nonces 12 to 29 bytes, tags and
  chunks at least 16 bytes, and a sealed chunk `S` at most 256 bytes, the
  longest array Orange has. The associated data is always the 64-byte header.
- `seal` is authenticated encryption: `authentic(k, n, a, seal(k, n, a, p))`
  is `true`, and `open` of it returns `p`. `authentic` must be `true` only for
  what `seal` produces; `orangec` calls `open` only on a chunk that
  `authentic` accepted, so `open` need not check the tag.
- Each call may take at most 16,777,216 evaluation steps.
- The parameter names above are the convention; `orangec` checks the types in
  order.

`orangec schemes PATH` compiles a program, checks its interface, seals one
chunk of zeros, and prints its sizes and step count. A program that differs
from the interface is refused with the signature `orangec` expected.

## Sealed-file format 1

A sealed file is a 64-byte header followed by one or more sealed chunks.

| Bytes | Field |
| --- | --- |
| 0 to 5 | the magic `orange` |
| 6 | zero |
| 7 | the format version, 1 |
| 8 | key length `K` in bytes |
| 9 | nonce length `N` in bytes |
| 10 | tag length `T` in bytes |
| 11 | zero |
| 12 to 15 | chunk length `C` in bytes, big-endian |
| 16 to 39 | the scheme's name in ASCII, padded with zeros |
| 40 to 63 | the nonce prefix: `N - 5` random bytes, padded with zeros |

Chunk `i`, counting from 0, is `seal(key, nonce_i, header, plaintext_i)`,
`C + T` bytes, where

- `nonce_i` is the prefix, then `i` as a 32-bit big-endian integer, then one
  byte that is 1 for the final chunk and 0 for every other: the STREAM
  construction of Hoang, Reyhanitabar, Rogaway, and Vizár, "Online
  Authenticated-Encryption and its Nonce-Reuse Misuse-Resistance" (CRYPTO
  2015);
- every chunk but the final one holds exactly `C` bytes of plaintext, and the
  final chunk holds the remaining 0 to `C - 1` bytes followed by the byte 0x80
  and zeros up to `C` bytes (the padding of ISO/IEC 7816-4), so every file,
  even an empty one, ends with a final chunk; and
- the whole header is every chunk's associated data.

`enc` draws a fresh prefix from the operating system for every file. A file
holds fewer than 2^32 chunks.

`dec` parses the header strictly: any other magic, version, or reserved byte,
impossible sizes, a malformed name, or nonzero padding is refused before any
evaluation. The scheme the header names must be the key's scheme, and its sizes
must be that scheme's sizes. `dec` then checks every chunk with `authentic`,
knowing it is final only when nothing follows it, and opens it with `open`.
Changing any byte of the header or a chunk, dropping, reordering, or repeating
chunks, cutting the file short, appending to it, or splicing chunks from
another file all make `dec` fail: the header is refused, or some chunk fails
`authentic`. The opened bytes go to
`OUTPUT.partial`, which becomes `OUTPUT` only when every chunk is authentic and
the final padding is well formed; on any failure it is removed, and nothing is
written.

## Keys

A key file is UTF-8 text with mode 0600. Lines that begin with `#` are
comments, and exactly one other line holds the key:

```text
orange-key 1 SCHEME HEX
```

`HEX` is the key in lowercase hexadecimal, 16 to 64 bytes. `keygen` reads the
key from `/dev/urandom`, writes it with mode 0600, and creates the default
key's folder with mode 0700. `enc` and `dec` refuse a key file that other users
can read or write.

## Security notes

- The Orange reference evaluator is not constant-time: how long a call takes
  and which memory it touches can depend on the key and the data. Do not seal
  on a machine where others can measure `orangec` while it runs.
- Nothing here is verified. The schemes are checked against published vectors
  and independent implementations, not proved correct, and the Rust code
  around them has tests, not proofs.
- Keys are stored unencrypted: whoever reads a key file can open and forge
  everything sealed with it. There are no passphrases yet; a password-based
  key derivation strong enough to use is too slow on the reference evaluator
  today.
- A random prefix bounds how many files one key may seal. The prefix is 19
  bytes for XChaCha20-Poly1305 and 11 bytes for Ascon-AEAD128, but only 7 bytes
  (56 bits) for ChaCha20-Poly1305: after 2^20 files under one
  ChaCha20-Poly1305 key, two share a prefix with probability about 2^-17, and
  files that share a prefix share nonces, which breaks both their secrecy and
  their integrity. That is why XChaCha20-Poly1305 is the default. Replace a
  ChaCha20-Poly1305 key long before that.
- Sealing hides the content, not the size: a sealed file shows its
  plaintext's length to within 239 bytes, and its header names the scheme.
- If `dec` is killed before it finishes, `OUTPUT.partial` can remain. It holds
  the plaintext of chunks that were each authentic, not a checked file.
