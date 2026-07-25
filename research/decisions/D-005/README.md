# D-005 historical negative inputs

Status: historical non-product research inputs; `draft_unfrozen`

These files preserve five dangerous records accepted by the provisional
[`claim-record-v0.1`](../../../schemas/gate0/claim-record-v0.1.schema.json)
shape and its current second-pass checks. They are inputs to the symmetric
D-005 comparison, not evidence that any public-assurance-model candidate has
run or passed.

The `epochs/0001/` component reserves the suite's eventual archive topology.
It does not freeze or create evidence epoch 0001. No candidate implementation,
candidate output, result, recommendation, owner review, or acceptance record is
present. Canonical D-005 execution evidence remains 0/32 candidate-case
executions.

The Rust laboratory now includes a draft-only adapter transport/capture
boundary. It constructs digest-bound requests and validates synthetic captured
bytes, but deliberately contains no subprocess launcher or candidate payload
validator. A transport-valid response exposes only an unvalidated object; it is
not a candidate result or decision evidence. The research inventory therefore
adds no adapter output, candidate directory, replay attestation, owner review,
or disposition artifact.

The machine-readable index is
[`legacy-v0.1-mutations.json`](d005-v0.1/epochs/0001/shared-inputs/legacy-v0.1-mutations.json).
The checked-in draft protocol is
[`epoch.json`](d005-v0.1/epochs/0001/protocol/epoch.json). Its strict canonical
JSON SHA-256 is
`731428229b4f77cd7e684e2a5cae51bdfd277898aaab60852b843d3183dbc194`;
the Rust laboratory parses those checked-in bytes and binds that digest before
it prepares any replay plan. The packet also binds the exact decision-suite
document, historical claim-record schema, and legacy-input manifest by raw-byte
SHA-256. That manifest in turn binds every historical fixture and comparison
base. Its `mutation_manifest_sha256` binds the ordered ID, case, and description
of all 50 suite mutations under the same strict canonical JSON profile.
The legacy index's `mutations` array contains exactly five historical negative
inputs. The
separate `subject-reuse-original.json` record is a comparison reference, not a
sixth mutation. Its paired mutation changes only the subject path and subject
digest while reusing the same claim and evidence identities. The index binds
every fixture and comparison base by its exact SHA-256 digest.

Every claim-shaped JSON file deliberately retains the schema-required
`record_status: provisional_gate0` value. Its `notes` mark the record as
historical, non-product, and `draft_unfrozen`; `non_product` is also the
schema-enforced Boolean `true`. Schema acceptance demonstrates only the defect
under test and grants no semantic, proof, validation, or release authority.
