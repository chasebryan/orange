# Gate 0 schema conformance fixtures

Status: provisional Gate 0 test data; non-product and non-authoritative

[`manifest.json`](manifest.json) is the machine-readable case list. Every case
names a repository-root-relative instance, its schema, the expected validity,
and, for a rejected instance, the expected error code, schema keyword, and
JSON Pointer. Case order is stable: schema family, then valid case before its
invalid mutation.

The `valid` records are representative shapes only. Their digests, times,
versions, sizes, paths, observations, review identities, approvals, and
independent-reproduction records may be intentionally synthetic. They are not
actual claim evidence, standards archives, legal review, GitHub audit, or
release provenance. Copying one into another directory does not make it
authoritative.

The `invalid` records are near-valid adversarial mutations:

| Invalid fixture | Required rejection |
| --- | --- |
| `claim-record-assumption-only.json` | A `satisfied` outcome cannot be supported only by an assumption; a checked, unexpired non-assumption basis is required |
| `evidence-manifest-network-enabled.json` | Replay network mode is not `denied` |
| `evidence-manifest-path-escape.json` | A repository path contains a parent-directory segment |
| `evidence-manifest-independent-without-review.json` | The manifest claims independent reproduction without an identified independent replay record |
| `repository-control-missing-explanation.json` | A disabled or unavailable control omits the explanation needed to distinguish state from coverage |
| `repository-control-selected-actions-empty.json` | Selected-Action mode identifies no allowed Action repository |
| `standards-provenance-bad-digest.json` | The purported SHA-256 value is not 64 lowercase hexadecimal characters |
| `standards-provenance-reviewed-without-reference.json` | A completed rights review has no accountable review reference |
| `trust-inventory-missing-identity.json` | An authoritative component has no immutable identity digest |

The assumption-only claim is a cross-record policy violation that JSON Schema
cannot express within the deterministic validator subset. The repository
validator must report `CLAIM_SATISFIED_WITHOUT_CHECKED_BASIS` at `/basis` after
shape validation. All other invalid cases exercise schema constraints. The
valid evidence fixture deliberately includes an empty non-command argument,
an additional allowlisted environment variable, a serial concurrency record,
separate matched observation, trust-inventory reference, synthetic independent
reproduction, and synthetic supersession digest so those fields do not remain
unexercised.

A complete run must prove both halves: all positive cases are accepted and all
negative cases are rejected for the expected reason. Merely parsing every JSON
file, or merely observing that a negative case failed somehow, is insufficient.
