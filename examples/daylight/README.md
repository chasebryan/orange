# Daylight Horizon v17 in Orange

This is a working Orange port of the cryptographic computations used by
`chasebryan/-wuci-ji`'s **Daylight Horizon v17**. It reproduces the upstream
Horizon key derivation and complete sealed bytes, and interoperates with the
upstream vault while retaining its evidence and policy checks.

## Run the standalone Orange program

With Orange's `orangec` on your PATH:

```sh
orangec check daylight-horizon.or
orangec eval daylight-horizon.or
```

`daylight-horizon.or` contains the complete cryptographic calculation for a
public, fixed-context Horizon vector: SHA-256, HKDF-SHA256, ChaCha20 encryption,
Poly1305 authentication, and framing. Its `example()` returns 219 framed bytes.
It does not call Python. Its final 48 bytes, the ciphertext and authentication
tag, are:

```text
52a6747ccae6ef1548b86f1eabbfa9569f815f01841a4b7f17384facd1772fb5dbff1531ab89bb4161a5afca9ea0717c
```

`seal_fixture` accepts a root key and 32-byte plaintext, but deliberately fixes
the header and nonce to this public interoperability fixture. It is a test
entry point, not a general vault API. The fixture header tests Horizon's crypto
framing; it does not carry a vault evidence authorization. Use the integration
below for the real vault and its policy enforcement.

## Files

| File | Purpose |
| --- | --- |
| `daylight.or` | Reusable Orange ChaCha20, Poly1305, SHA-256 and tag-comparison primitives. |
| `daylight-horizon.or` | Self-contained Orange Horizon seal vector, including key derivation and framing. |
| `daylight.py` | Bounded interpreter bridge implementing Horizon's private AEAD backend interface. |
| `emit_example.py` | Deterministically regenerates the standalone Orange vector from the core. |
| `test_daylight.py` | Standards, upstream agreement, boundary, tamper and policy tests. |
| `validation.txt` | Captured local test results. |
| `LICENSE`, `NOTICE` | Apache-2.0 terms and upstream attribution. |

## General inputs and Horizon integration

Orange 2026 currently has fixed arrays, literal indices and no general loops,
conditionals, JSON parser, native code generation or foreign-function interface.
The bridge emits bounded Orange expressions and runs them through `orangec eval
-`. Python performs byte encoding, padding, batching, framing/control flow and
invocation; hash, HMAC/HKDF, cipher, MAC and tag-comparison arithmetic runs in
Orange. The runtime bridge imports no Python cryptographic library.

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

The byte contract is preserved:

```text
header_bytes = canonical JSON(header)
salt         = SHA256(magic || header_bytes)
info         = "DAYLIGHT-HORIZON-ALPHA-AEAD-KEY:" || magic || ":" || authorization_tag
key          = HKDF-SHA256(root_key, salt, info, 32)
aad          = magic || LE32(length(header_bytes)) || header_bytes
sealed       = aad || ChaCha20-Poly1305(key, header.nonce, aad, plaintext)
```

The bridge caps individual byte inputs at 1 MiB and the combined padded AEAD
MAC input at 1 MiB. It rejects ChaCha counter wrap and invalid key, nonce and
tag lengths. Internal batches stay within Orange's array, binding and evaluator
limits. The tests exercise batch boundaries; they are not a throughput or
maximum-size performance qualification.

## Validation and provenance

Validated locally on Linux with Python 3.12.14, Rust 1.96.1 and the actual Orange
reference evaluator. The initial standalone validation used clean upstream checkouts on `main`;
Wuci-Ji's unrelated `third_party/zp1` submodule was left uninitialized. This
folder is the owner-directed Orange repository integration of that port.

| Input | Exact revision / source |
| --- | --- |
| Orange | `f92b2f5d719b36d98591d43a2a0222b61ab4955e` |
| Wuci-Ji | `fec91dc6618d477908790520118bae7e25909c43` |
| Horizon framing/key derivation | `daylight/v17-singularity/src/horizon_crypto.py` |
| Horizon evidence boundary | `daylight/v17-singularity/src/horizon_vault.py` |
| Upstream primitive reference | `daylight/v15-meridian/src/aead.py` |
| AEAD and Poly1305 | [RFC 8439](https://www.rfc-editor.org/rfc/rfc8439) |
| HKDF | [RFC 5869](https://www.rfc-editor.org/rfc/rfc5869) |
| SHA-256 | [FIPS 180-4](https://doi.org/10.6028/NIST.FIPS.180-4) |

The implementation follows the pinned
[Horizon crypto source](https://github.com/chasebryan/-wuci-ji/blob/fec91dc6618d477908790520118bae7e25909c43/daylight/v17-singularity/src/horizon_crypto.py).

Build the pinned Orange checkout:

```sh
cargo build --release --locked --offline --manifest-path /path/to/orange/compiler/Cargo.toml -p orangec
```

From this bundle, run:

```sh
python3 test_daylight.py \
  --orangec /path/to/orange/compiler/target/release/orangec \
  --upstream /path/to/-wuci-ji
```

**Result: 12 tests passed**, including multiple subcases:

- RFC 8439 ChaCha block, Poly1305, and AEAD vectors; RFC 5869 HKDF vector.
- SHA-256 and HMAC comparison with Python's independent standard library.
- Poly1305 partial blocks, carry/reduction edges, zero/all-one inputs and batching.
- AEAD empty, partial and multiple-block lengths, compared with the upstream reference.
- Actual Horizon vault derived-key and framed-byte equality, plus opening in both directions.
- Modified key, nonce, AAD, ciphertext and tag rejected before plaintext computation.
- Modified Horizon header/authorization, stricter declaration policy, and disallowed fixture evidence rejected.
- Standalone Orange fixture agreement and deterministic source regeneration.

The upstream six-test AEAD vector suite also passed:

```sh
PYTHONPATH=/path/to/-wuci-ji/daylight/v15-meridian python3 -m unittest discover \
  -s /path/to/-wuci-ji/daylight/v15-meridian/tests -p test_aead_vectors.py
```

This is a new port, not a repair to upstream behavior. Wuci-Ji's assembly,
release, website, OS, and external-evidence lanes are unchanged and their full
suites were not run for this standalone artifact. The owner subsequently requested this folder be committed and merged into
Orange. No deployment or product release is part of this integration.

This remains a locally validated research reference. Orange proof checking and
native code generation are unavailable; there is no formal verification,
constant-time, secret zeroization, external audit, or production-readiness
claim. Keys and inputs exist in interpreter source and subprocess memory, even
though the bridge sends that source through stdin rather than writing it to disk.
Do not use the public vector's fixed key/nonce as a real encryption configuration.

The Orange translation, bridge, generator, tests and documentation were
AI-assisted with OpenAI Codex. Test success is local evidence of the named
behaviors, not independent review.

## Orange repository integration

From the repository root, run the permanent compiler checks for this example:

```sh
cargo test --manifest-path compiler/Cargo.toml -p orangec --test cli daylight
```

These tests execute the full standalone frame vector twice and exercise the
Python adapter's public RFC 8439 vector through the actual compiler binary.
They run in the existing debug and release compiler CI lanes without network
access or a Wuci-Ji checkout. The 12-test upstream compatibility suite above
remains a separate, explicitly pinned local integration check.

The scoped Apache-2.0 `LICENSE` and `NOTICE` apply to this Daylight example.
Their admission under [D-018](../../docs/DECISIONS.md#d-018--licenses) does not
select an outbound license for the rest of Orange. OEP-0007's existing S3d
arrays suffice; this adds no language feature, compiler dependency, proof
claim or production release.
