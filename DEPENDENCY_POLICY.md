# Dependency policy

Status: enforced repository policy for Gate 0; product dependency choices remain
blocked by Gate 0 decisions.

No product dependency may be admitted before licensing, proof-foundation, host,
target, and trust-boundary decisions close. Gate 0 research tools must be
isolated, pinned by exact version and digest, reproducible, and accompanied by
license and provenance records. They are outside the product and logical TCB
unless a later accepted decision explicitly admits them.

## Dependency classes

Every dependency must be classified as one or more of:

- runtime;
- build or bootstrap;
- authoritative checker or other TCB;
- proof automation whose results are independently checked;
- CI Action or reusable workflow;
- test, vector, benchmark, or external oracle; or
- documentation and repository tooling.

## Admission record

An admission proposal must identify:

1. the need, rejected alternatives, owner, and removal plan;
2. exact source, version, immutable digest, and archival location;
3. the direct and transitive graph;
4. licenses, patent terms, provenance, and redistribution constraints;
5. maintainer health, release practice, and relevant vulnerability history;
6. install-time network access, native build scripts, plugins, and capabilities;
7. offline availability and reproducibility;
8. update, rollback, compromise, and end-of-life handling; and
9. effects on the TCB, axioms, claims, threat model, supported targets, and
   evidence replay.

Repository automation, Dependabot, dependency review, SBOMs, and vulnerability
alerts are defense in depth. They do not constitute admission approval.
Until D-018 closes, Dependabot pull requests are non-mergeable surveillance
suggestions. The bootstrap steward reviews the diff and independently authors
an admitted update with the required provenance record.

## Immutability and execution rules

- Language packages require a lockfile and archived package bytes. A lockfile
  alone is not an offline dependency store.
- GitHub Actions and reusable workflows use a full 40-character commit SHA and
  retain a human-readable version comment.
- Containers and images use content digests, never mutable tags.
- Toolchains and operating-system inputs use an immutable snapshot or digest.
- Release inputs contain no floating ranges, branches, tags, or latest URLs.
- A claim-bearing dependency graph does not run arbitrary native build scripts,
  compiler plugins, or generators with ambient authority.

Security policy failures are fail-closed. A non-assurance exception must name
an owner, scope, rationale, compensating control, expiry, and approval. No
exception may waive an assurance stop-ship condition.
