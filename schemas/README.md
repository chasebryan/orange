# Gate 0 evidence schemas

Status: provisional Gate 0 architecture; non-product and non-normative

This directory contains version `0.1` drafts for the records needed to review
Orange's evidence architecture before product implementation begins. They are
not the future language, package, proof, compiler, registry, or release
formats. Acceptance of a fixture proves only that the fixture has the shape of
a Gate 0 architecture record. It does not prove a technical claim.

The schemas use JSON Schema draft 2020-12. Their `$id` values are
`urn:orange:gate0:...` identifiers: they are stable local identifiers, not
published network locations and not promises of a future namespace. A future
ratified format must receive a new identifier and a documented migration; it
must not silently reinterpret these records.

## Drafts

| Schema | Boundary represented |
| --- | --- |
| `gate0/claim-record-v0.1.schema.json` | One precisely scoped claim, its outcome, assumptions, exclusions, review policy, and typed evidence bases |
| `gate0/evidence-manifest-v0.1.schema.json` | Content-addressed Gate 0 decision inputs, outputs, external sources, reproducibility level, observations, trust references, replay profile, independent reproductions, and supersession |
| `gate0/trust-inventory-v0.1.schema.json` | TCB components, axioms, trusted models, external contracts, compromise effects, and change from a prior inventory |
| `gate0/standards-provenance-v0.1.schema.json` | Exact standards, errata, clauses, vector sources, archival state, accountable rights/legal review, and reviewer-backed transcription review |
| `gate0/repository-control-snapshot-v0.1.schema.json` | Point-in-time evidence about GitHub security, selected Actions, default-branch and merge controls, response retention, and expiring exceptions |

## Cross-record rules

JSON Schema enforces local shape. A repository validator must additionally
enforce relationships that draft 2020-12 cannot express conveniently:

- identifiers are unique across the relevant inventory;
- every `*_ref` resolves to a record in the same evidence closure;
- every referenced path passes the schema's lexical safety rule and, where it
  identifies repository content, exists, stays beneath the repository root
  after symlink resolution, is NFC, and has the recorded digest and size;
- collection ordering follows
  [`docs/REPRODUCIBILITY.md`](../docs/REPRODUCIBILITY.md);
- a `satisfied` claim has a checked, unexpired non-assumption basis permitted by
  the policy for that claim kind;
- an assumption is never rewritten as successful evidence;
- trust-inventory closure references name existing components, axioms, models,
  and contracts;
- repository-control observations distinguish `disabled`, `unavailable`, and
  `unverified`; and
- source archives and transcriptions comply with their recorded rights terms.

Schema `format` keywords are assertions in this repository. Validators must
check them rather than treating them as annotations.

The schemas reject absolute/drive-prefixed paths, backslashes, controls, empty
segments, `.` and `..` segments, and trailing `/`. That lexical check is not a
filesystem containment proof. Resolution, digest/size comparison, cross-file
reference closure, expiry evaluation, and rights-policy evaluation remain
mandatory second-pass checks.

## Conformance material

Representative records and deliberately invalid mutations live in
[`conformance/foundation`](../conformance/foundation/README.md). The examples
use synthetic hashes and evidence identifiers. They must never be cited as
actual Orange assurance, standards acquisition, or repository-control
observations.
