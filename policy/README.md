# Repository policy

`gate0-repository-policy.json` is the machine-readable policy enforced while
Orange remains in Gate 0. It complements the human governance and assurance
documents; it does not ratify any decision that those documents leave proposed,
investigative, or blocked.

The policy deliberately fails if product implementation or a repository-wide
license appears before its decision gate closes. When Gate 0 is ratified, its
replacement must arrive through an accepted OEP with migration, threat, TCB,
license, contribution, and conformance effects. Historical policy versions
remain available in Git.

Run `make check` to execute unit/adversarial tests and then validate the current
tree. GitHub required CI invokes the same entry point.

The Gate 0 tree is closed by default. Permanent files and conformance instances
use an exact static inventory; only correctly named OEP and ADR records may be
added outside it, and those records receive their own structural validation.
Security-sensitive workflows, templates, ownership rules, CI scripts, schemas,
fixtures, tests, and policy documents also have reviewed SHA-256 identities in
both the validator and policy record. Changing one requires an intentional,
reviewable update to both enforcement sources.

Official binary brand assets are closed by the same inventory and are admitted
only with an exact path, role, provenance statement, and SHA-256 digest. Their
human and machine-readable inventory lives under [`assets/brand/`](../assets/brand/).
This admission records project identity; it does not close the pending name,
trademark, or license decisions.

The validator and `gate0-repository-policy.json` cannot safely contain their own
digests without a self-reference cycle. Their integrity therefore depends on
the exact path/executable contracts plus GitHub review, required-check, and
protected-branch controls. Local hashes are defense in depth and do not replace
server-side separation of acceptance from authoring.
