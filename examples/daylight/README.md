# Daylight Horizon v17 in Orange

This is `chasebryan/-wuci-ji`'s **Daylight Horizon v17** seal, written in
Orange. `daylight-horizon.or` is one program: SHA-256, HMAC, HKDF, ChaCha20,
Poly1305, their AEAD, and the Horizon frame, each written the way its standard
writes it, with `seal`, `open`, and `authentic` on top. It reproduces the
upstream key derivation and complete sealed bytes, and `daylight.py` runs its
specifications beneath the upstream vault, whose evidence and policy checks
stay in force.

## Run the standalone program

With Orange's `orangec` on your PATH:

```sh
orangec check daylight-horizon.or
orangec eval daylight-horizon.or
```

`orangec eval` prints every specification that takes no parameters: the named
constants of the standards, the four inputs of the public interoperability
vector, then `example`, the 219-byte frame that `seal` computes for them,
`pinned_frame`, the same bytes as the pinned upstream source produces, written
into the program as a literal, and `recovered`, the plaintext that `open` gets
back from the pinned frame. The frame's final 48 bytes, the ciphertext and the
tag, are:

```text
52a6747ccae6ef1548b86f1eabbfa9569f815f01841a4b7f17384facd1772fb5dbff1531ab89bb4161a5afca9ea0717c
```

## What the program says

The program follows the pinned byte contract of `horizon_crypto.py`:

```text
header_bytes = canonical JSON(header)
salt         = SHA256(magic || header_bytes)
info         = "DAYLIGHT-HORIZON-ALPHA-AEAD-KEY:" || magic || ":" || authorization_tag
key          = HKDF-SHA256(root_key, salt, info, 32)
aad          = magic || LE32(length(header_bytes)) || header_bytes
sealed       = aad || ChaCha20-Poly1305(key, header.nonce, aad, plaintext)
```

Its sections are the standards, in the order the contract uses them:

| Section | Specifications | Standard |
| --- | --- | --- |
| Bytes and words | `load_be32`, `hex_32`, `append`, ... | |
| SHA-256 | `schedule`, `round`, `compress`, `sha256_tail`, `sha256` | FIPS 180-4 |
| HMAC and HKDF | `keyed_block`, `hmac`, `hkdf_32` | RFC 2104, RFC 5869 |
| ChaCha20 | `quarter_round`, `double_round`, `block`, `keystream`, `encrypt_32`, `one_time_key` | RFC 8439 section 2 |
| Poly1305 | `absorb`, `poly_r`, `poly_s`, `block_value`, `tag`, `same_tag` | RFC 8439 section 2.5 |
| Daylight Horizon framing | `header`, `salt`, `info`, `derive_key`, `authenticator`, `seal`, `authentic`, `open` | RFC 8439 section 2.8 and the contract above |

Rounds and schedules are loops over literal ranges, every index is proved in
range before anything runs, and arrays are values, so nothing is mutated. The
Poly1305 accumulator is an exact integer reduced modulo 2^130 - 5 after every
block, as RFC 8439 defines it. `open` compares the tag it recomputes with the
tag the frame carries and chooses the plaintext or zeros; only the chosen branch
is evaluated, so no plaintext is computed for a frame that does not
authenticate, and `authentic` gives the comparison itself:

```orange
spec seal(
  root: Word[8]^32,
  nonce: Word[8]^12,
  authorization_tag: Word[8]^32,
  plaintext: Word[8]^32,
) -> Word[8]^219 {
  let h: Word[8]^160 = header(nonce, authorization_tag);
  let key: Word[8]^32 = derive_key(root, h);
  let f: Word[8]^219 = framed(h, encrypt_32(key, nonce, plaintext));
  with_tag(f, authenticator(key, nonce, f))
}

spec open(root: Word[8]^32, sealed: Word[8]^219) -> Word[8]^32 {
  let h: Word[8]^160 = frame_header(sealed);
  let key: Word[8]^32 = derive_key(root, h);
  let nonce: Word[8]^12 = header_nonce(h);
  if same_tag(authenticator(key, nonce, sealed), frame_tag(sealed)) {
    encrypt_32(key, nonce, frame_ciphertext(sealed))
  } else {
    [0; 32]
  }
}
```

The program uses the S3e loops of
[OEP-0008](../../docs/governance/oeps/OEP-0008-orange-2026-bounded-loops.md) and
the S3f truth values, comparisons, division, and conditionals of
[OEP-0009](../../docs/governance/oeps/OEP-0009-orange-2026-conditions.md), both
implemented and in owner review.

What the language fixes today, the program states rather than hides:

- **The header shape is fixed.** `seal` takes the root key, the nonce, the
  authorization tag, and a 32-byte plaintext, and writes the public
  interoperability header of `test_daylight.py`, canonical JSON of 160 bytes
  with the tag and the nonce as hexadecimal digits. A vault header carries more
  fields and is longer than an Orange array, which holds at most 256 elements;
  the bridge below serves the vault.
- **Lengths are literal.** `sha256` hashes a message of up to 183 bytes held in
  a three-block buffer, `hmac` a text of up to 119 bytes, and `hkdf_32` an
  `info` of up to 118 bytes, which is what the contract needs. Orange has no
  strings, so every text is its bytes with the text in a comment.
- **Evaluation is bounded.** The program's parameterless specifications share
  the reference evaluator's budget of 1,048,576 steps and use about 870,000 of
  them, most in key derivation; `seal` alone costs about 460,000 and `open`
  about 390,000.
- **Nothing here claims timing.** A conditional is a choice between two values
  and `&&` evaluates both sides; whether any of it runs in constant time on a
  machine is not a property the source states.

## Files

| File | Purpose |
| --- | --- |
| `daylight-horizon.or` | The Orange program: primitives, framing, `seal`, `open`, `authentic`, and the pinned example. |
| `daylight.py` | Bounded interpreter bridge implementing Horizon's private AEAD backend interface over the program. |
| `test_daylight.py` | Standards, upstream agreement, boundary, tamper, and policy tests, with a pure-Python reference. |
| `validation.txt` | Captured local test results. |
| `LICENSE`, `NOTICE` | Apache-2.0 terms and upstream attribution. |

## General inputs and Horizon integration

Orange 2026 has fixed-length arrays, literal loop bounds, and no data-dependent
indices, JSON parser, native code generation, or foreign-function interface.
The bridge loads the program up to its `Examples` line, since `orangec eval`
prints every specification without parameters, appends one entry point, and
runs it through `orangec eval -`. Python performs byte encoding, padding,
batching, framing, and control flow; hashing, HMAC and HKDF, the cipher, the
authenticator, and the tag comparison are the program's specifications. The
runtime bridge imports no Python cryptographic library.

For a public AEAD example:

```sh
python3 daylight.py --orangec /absolute/path/to/orange/compiler/target/release/orangec
```

To use it beneath the existing Horizon vault, put this directory on Python's
module search path along with the upstream v17 package:

```python
from daylight import OrangeBackend
from src import horizon_crypto, horizon_vault

backend = OrangeBackend('/absolute/path/to/orangec')
vault = horizon_vault.HorizonVault('/path/to/existing/research-vault')

with backend.horizon(horizon_crypto):
    sealed = vault.seal_bytes(
        name='message.txt',
        plaintext=b'Daylight running in Orange.',
        state_path='/path/to/-wuci-ji/daylight/v17-singularity/examples/state.current.json',
    )
    opened = vault.open_bytes(
        sealed=sealed,
        state_path='/path/to/-wuci-ji/daylight/v17-singularity/examples/state.current.json',
    )
```

The context manager temporarily substitutes the upstream module's private
`_AEAD` backend, restoring it on exit. Use it only in a single-threaded research
process; the substitution is process-global. It targets the exact upstream
revision below, not an asserted stable public API.

The original vault still constructs and verifies its scorecard, policy digest,
authorization tag, object version, and evidence restrictions. This port does
not replace those checks with caller-supplied approval flags. Authentication
failure returns `None` before the bridge computes any plaintext.

The bridge caps individual byte inputs at 1 MiB and the combined padded AEAD
MAC input at 1 MiB. It rejects ChaCha counter wrap and invalid key, nonce, and
tag lengths. It hashes in batches of sixteen blocks, ciphers one block per
call, and authenticates in batches of sixty-four blocks, passing the exact
Poly1305 accumulator between calls. The tests exercise batch boundaries; they
are not a throughput or maximum-size performance qualification.

## Validation and provenance

Validated locally on Linux with Python 3.11.15, Rust 1.96.1, and the actual
Orange reference evaluator, on the S3f head of pull request #203 with this
folder, against a clean checkout of the pinned upstream revision.

| Input | Exact revision / source |
| --- | --- |
| Orange | the S3f head `4a730dcea9844ad7226131e57ab6523446885300` plus this folder |
| Wuci-Ji | `fec91dc6618d477908790520118bae7e25909c43` |
| Horizon framing/key derivation | `daylight/v17-singularity/src/horizon_crypto.py` |
| Horizon evidence boundary | `daylight/v17-singularity/src/horizon_vault.py` |
| Upstream primitive reference | `daylight/v15-meridian/src/aead.py` |
| AEAD and Poly1305 | [RFC 8439](https://www.rfc-editor.org/rfc/rfc8439) |
| HKDF | [RFC 5869](https://www.rfc-editor.org/rfc/rfc5869) |
| HMAC | [RFC 2104](https://www.rfc-editor.org/rfc/rfc2104) |
| SHA-256 | [FIPS 180-4](https://doi.org/10.6028/NIST.FIPS.180-4) |

The implementation follows the pinned
[Horizon crypto source](https://github.com/chasebryan/-wuci-ji/blob/fec91dc6618d477908790520118bae7e25909c43/daylight/v17-singularity/src/horizon_crypto.py).

Build the Orange compiler:

```sh
cargo build --release --locked --offline --manifest-path /path/to/orange/compiler/Cargo.toml -p orangec
```

From this folder, run:

```sh
python3 test_daylight.py --orangec /path/to/orange/compiler/target/release/orangec
python3 test_daylight.py \
  --orangec /path/to/orange/compiler/target/release/orangec \
  --upstream /path/to/-wuci-ji
```

Without `--upstream` the primitives and the standalone program are checked
against RFC and FIPS vectors, Python's `hashlib` and `hmac`, and a pure-Python
reference of RFC 8439 and the Horizon frame adapted from Daylight v15 Meridian.
With it, the frame is also compared with the Horizon source and the vault tests
run beneath its evidence checks.

**Result: 15 tests passed** with `--upstream` (14 and one skipped without it),
including multiple subcases:

- RFC 8439 ChaCha block, encryption, Poly1305, and AEAD vectors; RFC 5869 HKDF
  vector; FIPS 180-4 examples hashed by the program's own `sha256`.
- SHA-256 and HMAC comparison with Python's independent standard library, and
  the program's `hmac` and `hkdf_32` at their longest inputs.
- Poly1305 partial blocks, carry and reduction edges, tag truncation, zero and
  all-one inputs, and batching.
- AEAD empty, partial, and multiple-block lengths, compared with the reference.
- `seal` against the reference frame for random inputs; `open` recovering the
  plaintext; `authentic` false, and `open` zeros, for a changed root key,
  magic, header digit, ciphertext byte, or tag byte.
- Actual Horizon vault derived-key and framed-byte equality, plus opening in
  both directions; modified header or authorization, stricter declaration
  policy, and disallowed fixture evidence rejected.
- The standalone program's printed frame equal to its pinned literal, to the
  reference, and to `horizon_crypto.seal_framed`.

This is a port, not a repair to upstream behavior. Wuci-Ji's assembly, release,
website, OS, and external-evidence lanes are unchanged and their full suites
were not run. No deployment or product release is part of this integration.

This remains a locally validated research reference. Orange proof checking and
native code generation are unavailable; there is no formal verification,
constant-time, secret zeroization, external audit, or production-readiness
claim. Keys and inputs exist in interpreter source and subprocess memory, even
though the bridge sends that source through stdin rather than writing it to
disk. Do not use the public vector's fixed key and nonce as a real encryption
configuration.

The original translation, bridge, generator, and tests were AI-assisted with
OpenAI Codex. This rewrite in the language as it stands, its bridge, tests, and
documentation were AI-assisted with Claude Code at the owner's direction. Test
success is local evidence of the named behaviors, not independent review.

## Orange repository integration

From the repository root, run the permanent compiler checks for this example:

```sh
cargo test --manifest-path compiler/Cargo.toml -p orangec --test cli daylight
```

The first test checks the program and evaluates it twice, requiring the exact
printed inputs, the frame, its pinned literal, and the recovered plaintext,
byte for byte. The second runs the bridge's public RFC 8439 vector through the
actual compiler binary. Both run in the existing debug and release compiler CI
lanes without network access or a Wuci-Ji checkout. The 15-test suite above,
with `--upstream`, remains a separate, explicitly pinned local integration
check.

The scoped Apache-2.0 `LICENSE` and `NOTICE` apply to this Daylight example.
Their admission under [D-018](../../docs/DECISIONS.md#d-018--licenses) does not
select an outbound license for the rest of Orange. The program uses the
proposed S3e and S3f slices as implemented; this adds no language feature,
compiler dependency, proof claim, or production release.
