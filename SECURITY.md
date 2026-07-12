# Security policy

Status: solo pre-alpha development; no Orange software release exists.

## Supported versions

| Product or version | Security support |
| --- | --- |
| Orange software, toolchain, packages, or generated artifacts | Not applicable; none has been released |
| Compiler source on `main` | Best-effort owner triage; not a supported version or security guarantee |
| Research and planning documents | Reviewed for integrity, but not production software and carry no software-security assurance |

The working branch, default branch, planning documents, and future pre-release
artifacts must not be described as a supported product version.

## Report a vulnerability privately

Use [GitHub private vulnerability reporting](https://github.com/chasebryan/orange/security/advisories/new).
Private reporting is enabled for this repository. Never disclose an unpatched
vulnerability in a public issue, pull request, discussion, commit message, CI
log, or pasted proof-of-concept artifact.

Security reports may concern, among other things:

- proof-system unsoundness, forged evidence, or claim-verification bypass;
- compiler miscompilation or an incorrect connection to a native artifact;
- failure of a promised target or leakage profile;
- cryptographic standards nonconformance;
- an unsafe API or a misleading public assurance claim;
- source, dependency, build, CI, signing, registry, or update compromise;
- a malicious or taken-over dependency or package;
- documentation that predictably causes cryptographic misuse; or
- leaked credentials, signing material, or recovery material.

Do not submit secrets or private cryptographic material merely to demonstrate a
problem. If the problem is a general planning defect rather than a vulnerability,
use the public planning-defect issue form. Conduct reports follow
[CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md), not this security channel.

## What to include

Provide the smallest safe report that permits private triage:

- the affected commit, document, artifact, or digest;
- observed behavior, expected behavior, and reproducible steps or a safe proof
  of concept;
- likely impact, clearly separating observed facts from inferences;
- for a future released product, the language edition, toolchain version,
  cryptography profile, target/leakage profile, package and artifact digest,
  and relevant operating-system/CPU environment;
- any disclosure constraints and your preferred attribution; and
- whether you believe exploitation is active.

## Response and coordinated disclosure

The project targets acknowledgement within one business day and an initial
technical assessment within three business days. These are response targets,
not a contractual service-level agreement. Active exploitation, proof
unsoundness, widespread silent miscompilation or leakage, registry compromise,
or release-key compromise enters immediate incident mode.

The current repository has one owner who performs security triage and no
staffed, independent PSIRT. That is a disclosed solo-project limitation. It
does not block development; any later release must state the resulting
single-person response and continuity risk.

The project will confirm receipt, reproduce and classify the issue privately,
coordinate a disclosure date based on risk rather than impose a blanket
embargo, and fix every coupled artifact that is affected. That can include
code, semantics, proofs, vectors, documentation, claims, and attestations. A
published advisory will identify affected versions and any invalidated earlier
claims. Credit is given as requested when legally and operationally possible.

## Solo pre-release boundary

Orange currently publishes compiler source, planning material, and repository
policy, not a project-operated service or software distribution. This policy
does not grant testing authority over GitHub, another third-party system, or any
future Orange service. Counsel review is unavailable and no counsel-reviewed
safe harbor is claimed.

Researchers must avoid privacy violations, destructive testing, denial of
service, persistence, lateral movement, social engineering, and access beyond
what is necessary to demonstrate the issue. Stop when sensitive data is
encountered; minimize, protect, and delete any data obtained. Give the project
a reasonable opportunity to remediate before disclosure.

This policy does not promise a bounty, waive third-party rights, grant legal
immunity, or provide legal advice. Public source and documentation review can
still be reported privately through the path above.
