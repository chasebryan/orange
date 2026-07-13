# Repository policy

`gate0-repository-policy.json` retains its historical filename but now enforces
the solo-bootstrap repository stage created by D-023 and OEP-0001. Historical
policy versions remain available in Git.

The policy admits the exact Rust compiler inventory through the first bounded
typed-reference evaluator and continues to fail closed on unknown source,
unexpected binaries, untracked executable paths, unratified license files,
unapproved dependencies, and workflow drift. The validator parses every
admitted Cargo manifest and the lock graph; only the workspace-local `orangec`
to `orange-compiler` path dependency is allowed. Product implementation is
allowed; product releases and third-party pull requests are not.

Run `make check` to execute Rust formatting, linting, and tests plus the
foundation unit/adversarial tests and repository validator. GitHub required CI
executes the same compiler checks and policy boundary.

The validator always binds filesystem scope to the checkout containing
`tools/validate_foundation.py`. Its optional `--root PATH` flag is an
assertion-only compatibility interface: the path is resolved and must identify
that same checkout, after which the trusted script-owned path is used. It cannot
redirect validation to another tree. Omitting `--root` has identical scope, and
caller-selected policy paths remain unsupported.

The tree remains closed by default. Permanent files and conformance instances
use an exact static inventory; correctly named OEP and ADR records may be added
outside it and receive structural validation. The S3a inventory explicitly adds
the Core, semantics, and evaluator sources; the permanent typed-answer fixture;
the exact ten-file S3a CLI conformance corpus and runner; and normative
[`docs/SEMANTICS_2026.md`](../docs/SEMANTICS_2026.md). The corpus externalizes
only accepted behavior and explicitly remains incomplete against the full S3a
conformance minimum. The normative Orange 2026 syntax and semantic documents
are digest protected. Adding another compiler source, fixture, test runner, or
normative language file requires an intentional policy and validator inventory
update.

The S3a conformance runner and all ten corpus fixtures also retain reviewed
SHA-256 identities in both enforcement sources. The runner checks both
`orangec check` and `orangec eval`, exact diagnostic-code sequences, no partial
failure output, exact primary line and column, significant-integer boundaries,
semantic-diagnostic suppression, and same-revision repeatability. Those
observations are solo-produced implementation tests, not proof or independent
validation.

The admitted
[`docs/PRODUCT_FORM_DECISION_PACKET.md`](../docs/PRODUCT_FORM_DECISION_PACKET.md)
and
[`docs/SEMANTIC_STRATA_DECISION_SUITE.md`](../docs/SEMANTIC_STRATA_DECISION_SUITE.md)
are structurally validated, non-normative D-003 and D-004 research protocols.
Their presence does not record owner acceptance, allocate an OEP number, widen
the implemented semantics, or authorize S3b.

Security-sensitive workflows, templates, ownership rules, CI scripts, schemas,
fixtures, tests, the normative Orange 2026 syntax and semantics, and selected
policy documents retain reviewed SHA-256 identities in both the validator and
policy record.
Changing one requires an intentional update to both enforcement sources. Solo
owner review is a change-control record, not independent review.

Official binary brand assets remain closed by exact path, role, provenance, and
SHA-256 digest. Their inventory under [`assets/brand/`](../assets/brand/) records
working project identity; it does not claim trademark clearance or grant a
repository-wide license.

The validator and policy JSON cannot safely contain their own digests without a
self-reference cycle. Their integrity depends on exact path and executable
contracts plus Git history and required checks.
