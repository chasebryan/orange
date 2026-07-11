# Release policy

Status: Gate 0 target policy. Orange has no software or product release.

A signed planning snapshot, if one is ever created, is evidence of a dated
Gate 0 record only. It is not a product, compiler, cryptography, certification,
or assurance release.

No product release may occur until the naming, licensing, governance, PSIRT,
release-authority, support, and applicable technical gate criteria close. The
current sole-steward repository cannot perform the required independent rebuild
or separation-of-duties ceremony.

## Release identity

Each future release binds all version axes defined by the roadmap:

- language edition;
- Core and evidence format edition;
- toolchain release;
- cryptography profile release; and
- target, ABI, and leakage profile release.

It has one immutable identifier, exact source and artifact digests, support
dates, changed claims, TCB and assumption deltas, and known limitations. A
single version number must never be presented as a generic assurance level.

## Release-candidate gate

A release candidate requires:

- a frozen, content-addressed dependency graph;
- a network-disabled build from completely declared inputs;
- the complete formal, target, vector, conformance, documentation, and audit
  suites required by its claim matrix;
- two independently administered bit-for-bit rebuilds;
- SPDX SBOM and CycloneDX SBOM/CBOM;
- SLSA/in-toto provenance and signature/transparency evidence;
- TUF-style update, rollback, freeze, revocation, and recovery metadata;
- TCB, axiom, external-evidence, and unresolved-finding inventories;
- every stop-ship condition evaluated; and
- a recorded, multi-role release ceremony.

Source acceptance, building, signing/publishing, registry operation, and
offline-root recovery may not collapse to one principal. Release or root keys
must not be ordinary GitHub repository secrets.

## GitHub publication rules

Before the first eligible product release, enable release immutability and a
release-tag ruleset. Use signed annotated tags, prohibit force updates,
deletion, and tag reuse, and preserve source plus independent verification
instructions. Create a draft, attach and verify the complete asset set, and
publish once. Never edit or replace a published release; issue a new version,
advisory, withdrawal, or revocation record.
