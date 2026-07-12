# Release policy

Status: solo-development policy; no Orange product release exists

Compiler source in the repository is pre-alpha development work. A merge,
archive, CI artifact, Cargo build, or planning snapshot is not an Orange product
release and carries no compatibility, support, cryptographic, or assurance
promise.

Outside rebuilders, release managers, signing principals, auditors, and PSIRT
staff are unavailable under D-023. Their absence does not block development and
must never be hidden. A later solo release may not claim independent rebuild,
separation of duties, threshold approval, or external validation.

## Release classes

The owner may later authorize one of these classes through a recorded decision:

- **source preview:** an immutable source snapshot for experimentation;
- **toolchain preview:** owner-built binaries with exact provenance and explicit
  pre-alpha limitations; or
- **stable toolchain:** a future release whose complete supported behavior,
  compatibility, security, and support gates are recorded and satisfied.

No class is authorized merely because compiler code exists. Cryptographic or
proof-bearing packages add their own stronger evidence gates.

## Release identity

Each release binds the version axes that actually exist:

- language edition;
- Core and evidence format edition where applicable;
- toolchain version;
- cryptography profile where applicable; and
- target, ABI, and leakage profile where applicable.

It has one immutable identifier, exact source and artifact digests, support
dates, changed claims, TCB and assumption deltas, known limitations, and a clear
`solo-produced` status. Unsupported axes remain explicit rather than omitted.

## Solo release gate

A preview release requires:

- an explicit owner release decision and versioned scope;
- a frozen dependency graph and pinned toolchain;
- a clean, network-disabled build where the toolchain permits it;
- two separately provisioned owner rebuilds with byte comparison;
- all tests, conformance cases, and security checks required by its exact claim
  matrix;
- source and artifact digests, dependency inventory, SBOM where applicable, and
  reproducible invocation records;
- known limitations, unresolved findings, non-claims, support dates, and
  vulnerability-reporting instructions; and
- an owner-recorded publish, rollback, withdrawal, and recovery procedure.

The two owner rebuilds are repeatability evidence, not independent rebuilds.
The owner necessarily controls source acceptance, building, and publication;
that missing separation of duties is recorded as residual risk.

## Publication rules

Before the first authorized release, configure immutable tags and release
assets. Use annotated tags, prohibit force updates, deletion, and tag reuse, and
preserve source plus verification instructions. Publish a new version for every
correction; never replace an already published artifact under the same identity.

The current unresolved license and working-name decisions prohibit crate,
package-registry, and binary distribution until their exact release boundary is
recorded. Local owner development remains permitted.
