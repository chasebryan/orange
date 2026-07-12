# Repository policy

`gate0-repository-policy.json` retains its historical filename but now enforces
the solo-bootstrap repository stage created by D-023 and OEP-0001. Historical
policy versions remain available in Git.

The policy admits the exact Rust compiler inventory through the first bounded
parser and continues to fail closed on unknown source, unexpected binaries,
untracked executable paths, unratified license files, unapproved dependencies,
and workflow drift. The validator parses every admitted Cargo manifest and the
lock graph; only the workspace-local `orangec` to `orange-compiler` path
dependency is allowed. Product implementation is allowed; product releases and
third-party pull requests are not.

Run `make check` to execute Rust formatting, linting, and tests plus the
foundation unit/adversarial tests and repository validator. GitHub required CI
executes the same compiler checks and policy boundary.

The tree remains closed by default. Permanent files and conformance instances
use an exact static inventory; correctly named OEP and ADR records may be added
outside it and receive structural validation. The S2 inventory explicitly adds
`compiler/crates/orange-compiler/src/parser.rs` and the normative
`docs/LANGUAGE_2026.md`; the latter is digest protected. Adding another compiler
source or normative language file requires an intentional policy and validator
inventory update.

Security-sensitive workflows, templates, ownership rules, CI scripts, schemas,
fixtures, tests, the normative Orange 2026 syntax, and selected policy documents
retain reviewed SHA-256 identities in both the validator and policy record.
Changing one requires an intentional update to both enforcement sources. Solo
owner review is a change-control record, not independent review.

Official binary brand assets remain closed by exact path, role, provenance, and
SHA-256 digest. Their inventory under [`assets/brand/`](../assets/brand/) records
working project identity; it does not claim trademark clearance or grant a
repository-wide license.

The validator and policy JSON cannot safely contain their own digests without a
self-reference cycle. Their integrity depends on exact path and executable
contracts plus Git history and required checks.
