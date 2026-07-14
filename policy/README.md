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

Run `scripts/ci/check-repository` for the hardened standard gate. Its POSIX
privileged shell mode suppresses inherited interpreter startup files before the
script executes. It resolves its script directory and repository root to their
physical paths before entering the checkout, so a symlink alias cannot change
the policy scope. It then removes inherited Make control and shell-startup
variables before Make parses any file and validates the closed repository tree
before foundation unit/adversarial tests and Rust formatting, linting, and
tests. The Make entrypoint serializes that order even under parallel execution,
and its fixed privileged recipe shell suppresses inherited Bash functions and
startup settings. The compiler target runs all tests in both debug and
optimized release profiles. The release profile retains debug assertions and
integer overflow checks, and a runtime test verifies both settings. GitHub
required CI uses the same policy-first order before repository-controlled test
discovery or Cargo execution. Direct `make check` remains a convenience for an
already trusted caller environment.

Canonical Python invocations start from an allowlisted environment with a fixed
hash seed, skip `site` initialization, exclude unsafe path injection, suppress
bytecode writes, and force UTF-8 mode. The validator therefore imports only the
interpreter's standard library before it accepts the closed tree; foundation
tests add the already validated checkout to `sys.path` only after that startup
and redirect bytecode lookup to a fresh temporary root.

Compiler checks likewise run through a protected Make recipe with a fresh,
canonical absolute Cargo home, fresh target tree, and allowlisted environment.
It invokes Cargo from the filesystem root with the exact selected toolchain,
preventing caller wrapper
variables, flags, target runners, home or ancestor Cargo configuration, and
ignored prior build artifacts from steering the build after policy validation.

The validator always binds filesystem scope to the checkout containing
`tools/validate_foundation.py`. Its optional `--root PATH` flag is an
assertion-only compatibility interface: the path is resolved and must identify
that same checkout, after which the trusted script-owned path is used. It cannot
redirect validation to another tree. Omitting `--root` has identical scope, and
caller-selected policy paths remain unsupported.

Repository discovery is likewise host-configuration independent. The bounded
Git inventory passes only the caller's tool-search path plus fixed Git and
locale controls, disables system and user configuration, disables
repository-configured filesystem monitors, and applies only checkout
`.gitignore` files when excluding untracked content. One
30-second deadline covers the complete inventory stream and process exit. If
Git is unavailable for an exported tree with no `.git` entry, the bounded
filesystem fallback preserves the same fail-closed resource checks. Each queued
fallback directory is reopened from the trusted root one component at a time
with no-follow flags, so a concurrent directory-to-symlink replacement cannot
redirect discovery. Git failure is fatal when repository metadata is present,
and a discovery ceiling prevents an exported tree from inheriting a parent
repository's index. Inventory paths remain raw bytes through record and
resource-limit checks, then must decode as UTF-8 before any path is admitted;
NFC is enforced later with the other tree format checks. Every Git inventory
record is checked for worktree presence one component at a time without
following symlinks, so a stale tracked deletion cannot disappear from the exact
path inventory. When Git metadata is present, the bounded file and stage-zero
path sets must be exactly equal; nonignored untracked content is rejected even
if its name would otherwise be admitted.
Schema, fixture, workflow, OEP, and ADR directory queries, plus every required,
forbidden, and optional artifact-presence query, are selected lexically from
that same bounded inventory. Validation does not launch a second filesystem
glob or ordinary path-status probe that could traverse ignored or concurrently
replaced directories. Markdown fragment validation computes each target's
anchor set once per run, so repeated links cannot multiply target scans. Local
link percent escapes must be complete and decode as strict UTF-8, preventing
lossy filename aliases.

Content reads require POSIX component-relative open support. Every directory
and final file component is opened with no-follow flags; the final open is also
nonblocking before its descriptor metadata is compared with the preflight
snapshot. Preflight rejects hardlinked files and uses `SEEK_HOLE` to reject
sparse files before parsing policy content. A host or filesystem without these
primitives receives `resource.unsupported_host` instead of a weaker validation
result. The validator and its intermediate schema and record-metadata checkers
each retain at most 4,096 detailed findings. Final finding messages retain at
most 4,096 characters, and the report adds one deterministic suppression
record, preventing bounded repository bytes from amplifying into an unbounded
diagnostic object or output stream.

The tree remains closed by default. Permanent files and conformance instances
use an exact static inventory; correctly named OEP and ADR records may be added
outside it and receive structural validation. The S3a inventory explicitly adds
the Core, semantics, and evaluator sources; the permanent typed-answer fixture;
the exact ten-file S3a CLI conformance corpus and runner; and normative
[`docs/SEMANTICS_2026.md`](../docs/SEMANTICS_2026.md). The corpus externalizes
only accepted behavior and is one layer of the indexed S3a evidence set. The
runner requires an exact stable 30-rule ID inventory and named evidence mapping;
it also binds each normative evidence-layer label and requires the corresponding
CLI, generated-CLI, parser-unit, or unit observation. Specialized labels also
require a named injected-writer or injected-limit test; host-failure coverage
separately requires I/O, allocation, and non-regular host-boundary failures.
Every named test must have exactly one unconditional declaration at its expected
harness location: integration tests at file root and unit tests directly inside
the source's unique `#[cfg(test)] mod tests` container. Declarations inside
comments, strings, nested functions, or alternate or disabled modules do not
qualify. That traceability is not proof that a named test exhausts its rule.
Production constants remain specification-bound by policy validation, and
internal injected-limit tests exercise accounting where a maximum cannot be
reached through valid public source before an earlier bound. The normative
Orange 2026 syntax and semantic documents are digest protected. Adding another
compiler source, fixture, test runner, or normative language file requires an
intentional policy and validator inventory update.

The S3a conformance runner and all ten corpus fixtures also retain reviewed
SHA-256 identities in both enforcement sources. The runner checks both
`orangec check` and `orangec eval`, exact diagnostic-code sequences, no partial
failure output, exact primary line and column, significant-integer boundaries,
leading-zero neutrality, semantic-diagnostic suppression, case-sensitive
spelling, every later same-kind duplicate, named internal resource evidence,
and same-revision repeatability. Those observations are solo-produced
implementation tests, not proof or independent validation.

The admitted
[`docs/PRODUCT_FORM_DECISION_PACKET.md`](../docs/PRODUCT_FORM_DECISION_PACKET.md)
and
[`docs/SEMANTIC_STRATA_DECISION_SUITE.md`](../docs/SEMANTIC_STRATA_DECISION_SUITE.md)
are structurally validated, non-normative D-003 and D-004 research protocols.
Their presence does not record owner acceptance, allocate an OEP number, widen
the implemented semantics, or authorize S3b.

The living [Orange Book](../docs/THE_ORANGE_BOOK.md) v0.2 remains a
non-normative reader guide. Its validator binds the exact manuscript version,
byline, status, ISO-date snapshot, ordered section and contents inventory,
minimum length of each drafted chapter, non-normative boundary, and drafting
disclosure. Chapter 2 explains the proposed claim model; neither the chapter nor
its structural validation accepts D-005 or creates a product claim format.

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
