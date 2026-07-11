# Gate 0 threat model

Status: living Gate 0 security evidence; not a product-security certification

Evidence snapshot: 2026-07-11

Hosted-control snapshot: `snapshot_date=2026-07-11 review_due_date=2026-10-11 ruleset_id=18810248`

Required-check binding: `context="Required CI / docs-policy-workflows" integration_id=15368`

Required-check binding: `context="Dependency Review / policy" integration_id=15368`

Owner: Bootstrap Repository Steward (`@chasebryan`)

Next scheduled review: 2026-10-11, or earlier on any mandatory trigger below

## Executive summary

Orange is presently a public planning and repository-control project. It has no
compiler, proof checker, package client, registry, cryptographic implementation,
product dependency, or product release. The immediate risks are therefore
repository and governance integrity: compromise of the sole maintainer,
misconfiguration or privileged weakening of protected-branch controls, unsafe
CI evolution, credential disclosure, and planning text that overstates controls
which do not yet exist. The intended product will add much higher-consequence proof,
compiler, leakage, package, release, and update boundaries. Those future threats
are recorded now as design requirements, not as claims that mitigations have
been implemented.

This document refines, but does not ratify or replace, the proposed security
constitution in [`docs/ASSURANCE.md`](../ASSURANCE.md). A threat marked
`future-blocking` is not currently exploitable through Orange software because
that software does not exist. It becomes a release blocker as soon as its entry
point or asset is introduced.

## Scope and assumptions

### Current in-scope system

- The authoritative public repository is
  [`chasebryan/orange`](https://github.com/chasebryan/orange), with `main` as its
  default branch.
- Tracked content consists of planning, governance, security policy, issue and
  pull-request templates, provisional Gate 0 schemas, conformance material, and
  repository automation. The schemas are architecture evidence, not product
  formats or proof evidence.
- GitHub is the hosted identity, repository, issue, private-vulnerability, and
  Actions control plane. Orange assesses its configuration and use; testing or
  threat modeling GitHub's internal implementation is out of scope.
- The current foundation change contains candidate GitHub Actions workflows for
  repository policy, dependency review, link checking, workflow-metadata audit,
  and OpenSSF Scorecard.
  Required CI and dependency review completed successfully for exact PR head
  `eac77fe1383361775a4f2256aaada4c8d02b345d`; active ruleset `18810248`
  requires both exact GitHub Actions check contexts. The workflow source remains
  unmerged, and trusted-`main` or scheduled behavior is not operating evidence
  until the corresponding hosted runs complete after merge.
- One person, `@chasebryan`, is the repository owner and only current
  collaborator. [`GOVERNANCE.md`](../../GOVERNANCE.md) explicitly bars this
  bootstrap state from making mature-governance, independent-review,
  external-certification, or product-release claims.

### Future design scope

The following components and flows are in scope as requirements because the
project documents commit the end product to them:

- source language, parser, elaborator, semantic cores, proof search, canonical
  Proof IR, and the authoritative offline checker;
- compilation through CT IR and Machine IR to object bytes and generated foreign
  interfaces;
- standards, errata, vectors, cryptographic packages, claims, evidence bundles,
  and independently replayable validation;
- package resolution, immutable local storage, registry, build, signing,
  provenance, release, update, rollback, revocation, and recovery; and
- host, target, ABI, operating-system, CPU, accelerator, entropy, and leakage
  assumptions.

These elements are described in [`docs/ARCHITECTURE.md`](../ARCHITECTURE.md) and
[`docs/ASSURANCE.md`](../ASSURANCE.md). Their existence and controls are not
asserted here.

### Material assumptions and open questions

- GitHub correctly enforces the repository settings returned by its APIs and
  keeps authentication, secret-scanning, and Actions isolation within its
  documented service boundary.
- The owner protects the GitHub account with phishing-resistant MFA and secure
  recovery material. Account-level MFA state was not independently observable
  during this review and remains `unverified`.
- No production credential, signing key, embargoed vulnerability, or private
  cryptographic material belongs in this repository or an untrusted workflow.
- Naming, licensing, proof-foundation, governance, target, leakage, and release
  decisions remain open. Their resolution can materially change likelihood,
  impact, ownership, and trust boundaries.
- Deployment scale, multi-tenancy, public service topology, package-registry
  operation, supported targets, and data sensitivity are not yet selected.
  Threat ranks for those surfaces are deliberately conditional.

Before Gate 0 closes, the project must answer who independently owns the TCB,
cryptography, release, and PSIRT controls; which services are Internet-facing;
which sensitive data each service may retain; and which exact target and
leakage profiles a release promises.

## System model

### Primary components

| Component ID | Component | State | Security role and evidence |
| --- | --- | --- | --- |
| CMP-001 | GitHub repository and control plane | Current, external | Authoritative source, history, issues, reviews, security settings, private reports, and workflow execution. Configuration evidence is maintained in [`OSPS_BASELINE.md`](OSPS_BASELINE.md). |
| CMP-002 | Gate 0 policy and evidence records | Current | Planning, decisions, threat/control records, provisional schemas, and conformance fixtures. See [`README.md`](../../README.md), [`docs/DECISIONS.md`](../DECISIONS.md), and [`schemas/README.md`](../../schemas/README.md). |
| CMP-003 | Repository CI | Branch candidate with hosted PR evidence | Proposes policy, dependency-review, link, workflow-metadata-audit, and Scorecard workflows under GitHub Actions. Workflow source is under [`.github/workflows/`](../../.github/workflows/); required CI and dependency review succeeded at the reviewed PR head and are required by ruleset `18810248`. Main-only and scheduled execution remains pending. |
| CMP-004 | Orange driver and language services | Future | Planned parser, elaborator, build coordinator, CLI, LSP, and host integration. No implementation exists. |
| CMP-005 | Orange semantic and evidence system | Future | Planned Core family, claims, Proof IR, proof search, and authoritative offline checker. No implementation exists. |
| CMP-006 | Orange compiler and native boundary | Future | Planned CT IR, Machine IR, compiler, object encoding, linker validation, C ABI, and target execution. No implementation exists. |
| CMP-007 | Package, registry, build, and release system | Future | Planned immutable dependency resolution, registry, hermetic builds, provenance, signing, publication, updates, and recovery. No implementation exists. |
| CMP-008 | Standards and cryptography corpus | Future | Planned standards/errata provenance, vectors, packages, proofs, tests, and external-validation records. No implementation exists. |

### Data flows and trust boundaries

Boundary IDs are permanent. A removed boundary keeps its ID and receives a
tombstone rather than being renumbered.

| Boundary ID | Source to destination | Data and channel | Current guarantees and validation | State |
| --- | --- | --- | --- | --- |
| TB-001 | Public user or contributor to CMP-001 | Issues, pull requests, Git refs, comments, titles, branch names, and uploaded text over GitHub HTTPS | Structured issue forms; blank issues disabled; security reports directed to a private channel; contribution scope restricted. GitHub authentication applies to writes. Content remains untrusted. | Current |
| TB-002 | Repository owner to CMP-001 | Credentials and privileged settings through GitHub HTTPS/API/SSH or Git credential transport | GitHub authentication is assumed; account-level phishing-resistant MFA is required by policy but unverified here. One-person custody remains a high residual risk. | Current |
| TB-003 | Proposed Git change to authoritative `main` | Git commits and pull-request metadata | Active ruleset `18810248` requires a branch and pull request, strict `Required CI / docs-policy-workflows` and `Dependency Review / policy` checks from GitHub Actions integration `15368`, resolved conversations, squash-only linear history, and blocks deletion and non-fast-forward updates without a bypass actor. Zero approvals and one administrator still allow unilateral reviewed-PR merge and privileged control-plane weakening or recovery. | Current control with solo-owner residual risk |
| TB-004 | Untrusted pull-request snapshot to CMP-003 | Repository files executed or parsed by hosted Actions runners | Candidate PR workflows declare no ambient permissions, grant only `contents: read` where needed, avoid privileged secrets and `pull_request_target`, disable persisted checkout credentials, and set timeouts. Required CI and dependency review succeeded for the exact PR head and are required by ruleset `18810248`. | Branch candidate with hosted PR evidence |
| TB-005 | CMP-003 to action/tool publishers and network services | Pinned Actions, digest-selected containers, downloaded tools, release archives, checksums, SARIF, and HTTPS requests | Repository settings require full action SHAs and restrict sources to the exact six Action repositories used by the candidate workflows; broad GitHub-owned and verified-publisher allowances are disabled. Scorecard executes directly at one separately machine-enforced OCI digest, and downloaded binaries use pinned versions and SHA-256 checks. Publisher, selected-action, selected-image, registry, hosted-runner, or provenance compromise remains possible. | Current setting; required PR workflows demonstrated; trusted-event jobs pending |
| TB-006 | Researcher to private security triage | Vulnerability report and attachments through GitHub private vulnerability reporting | Private reporting is enabled and [`SECURITY.md`](../../SECURITY.md) defines handling targets and disclosure constraints. Only one bootstrap steward receives and triages reports; no independent PSIRT exists. | Current |
| TB-007 | Human standards intent to CMP-008 formal specification | Standards, errata, clauses, vectors, interpretations, and transcription records | Exact provenance and independent review are required by policy. No admitted standard package or transcription exists. | Future-blocking |
| TB-008 | Surface source to CMP-005 canonical Core and claims | Source text, parsed syntax, elaboration, types, assumptions, and canonical serialization | Planned independent parsing, normative semantics, checked formats, and conformance cases. No implementation or accepted format exists. | Future-blocking |
| TB-009 | Proof search and automation to authoritative proof checking | Candidate proof objects, solver certificates, limits, and errors | Planned untrusted search with deterministic, resource-bounded independent checking and fail-closed outcomes. No checker exists. | Future-blocking |
| TB-010 | Each CMP-006 compiler stage to the next stage and final bytes | IR, certificates, transformations, target model, relocations, objects, and link results | Planned verified passes or checked per-artifact certificates, executable semantics, differential tests, and final-byte validation. No compiler exists. | Future-blocking |
| TB-011 | Generated artifact or foreign interface to its integrator, OS, CPU, accelerator, and entropy provider | ABI calls, buffers, errors, target features, entropy, runtime observations, and leakage | Planned explicit contracts, target profiles, named leakage models, misuse-resistant APIs, and empirical defense in depth. Exact platforms and profiles remain undecided. | Future-blocking |
| TB-012 | Authoritative source to CMP-007 builders, registry, and update client | Source, dependencies, build inputs, packages, provenance, keys, artifacts, and update metadata | Planned hermetic inputs, independent reproducible builds, least-privilege roles, signed provenance, transparency evidence, and TUF-style recovery. No release system or keys exist. | Future-blocking |

#### Diagram

```mermaid
flowchart LR
  Public["Public contributor or researcher"] -->|TB 001| GitHub["GitHub control plane"]
  Owner["Bootstrap steward"] -->|TB 002| GitHub
  GitHub -->|TB 003| Main["Authoritative main"]
  GitHub -->|TB 004| CI["Repository CI"]
  CI -->|TB 005| Tools["Pinned actions and tools"]
  Public -->|TB 006| Triage["Private security triage"]
  Standards["Standards and errata"] -->|TB 007| Spec["Formal specification"]
  Source["Orange source"] -->|TB 008| Core["Core and claims"]
  Search["Proof search"] -->|TB 009| Checker["Offline checker"]
  Core -->|TB 010| Native["Native artifact"]
  Native -->|TB 011| Platform["Integrator and platform"]
  Main -->|TB 012| Release["Build registry and update"]
```

## Assets and security objectives

| Asset ID | Asset | Why it matters | Objective |
| --- | --- | --- | --- |
| AS-001 | Authoritative repository, history, settings, and `main` | Unauthorized or erased changes can corrupt every downstream decision and future release. | Integrity, availability |
| AS-002 | Gate 0 decisions, policies, source provenance, and research evidence | Hidden edits or fabricated evidence can silently choose architecture, licensing, or assurance boundaries. | Integrity, authenticity, availability |
| AS-003 | Maintainer identity, credentials, recovery factors, and privileged settings | The sole current principal can change source, settings, reports, and future publication paths. | Confidentiality, integrity, availability |
| AS-004 | Workflow definitions, Actions tokens, runner isolation, and security results | CI can become an execution and credential boundary and can create false evidence if compromised. | Confidentiality, integrity, availability |
| AS-005 | Private vulnerability reports and incident records | Premature disclosure can enable exploitation and harm reporters or downstream users. | Confidentiality, integrity, availability |
| AS-006 | Future semantic truth, axioms, Core formats, claims, proofs, and checker | Unsound acceptance defeats the central assurance promise. | Integrity, authenticity, availability |
| AS-007 | Future standards, errata, vectors, and cryptographic source intent | Wrong or stale intent can yield internally consistent but unsafe cryptography. | Integrity, authenticity, availability |
| AS-008 | Future compiler stages, target models, objects, ABIs, and leakage evidence | A last-mile mismatch can invalidate functional, safety, or confidentiality claims. | Integrity, confidentiality, availability |
| AS-009 | Future packages, dependency graph, build inputs, release artifacts, and provenance | Substitution or rollback can deliver bytes different from reviewed source and evidence. | Integrity, authenticity, availability |
| AS-010 | Future signing, registry, update, revocation, and recovery keys | Key compromise can authorize malicious artifacts or prevent safe recovery. | Confidentiality, integrity, availability |
| AS-011 | Public assurance language and project trust | Overclaiming can cause unsafe adoption even when repository bytes are unchanged. | Integrity, authenticity |
| AS-012 | Official working emblem, wordmark, lockups, and their provenance record | Substitution, malformed bytes, or false rights/provenance claims can misrepresent project identity and expose image consumers. | Integrity, authenticity, availability |

## Attacker model

| Adversary ID | Capabilities | Important non-capabilities in the current stage |
| --- | --- | --- |
| ADV-001 | A public contributor controls fork content, PR/issue text, branch names, commits, and other public metadata; may submit pathological files or social-engineering content. | Has no repository write/admin permission and receives no repository secrets from an ordinary fork PR by design. |
| ADV-002 | An attacker compromises the sole owner or a future collaborator account, Git credential, session, recovery path, or local workstation. | Does not automatically compromise an offline key or independent reviewer; neither exists for a product today. |
| ADV-003 | A dependency, Action, tool, publisher, release archive, registry, mirror, runner, or network path is malicious or compromised. | Cannot change a referenced full commit SHA without changing workflow source, but can compromise the content already at that identity or a downloaded artifact whose digest was incorrectly admitted. |
| ADV-004 | A privileged insider or captured governance authority intentionally bypasses review, weakens a model, conceals an assumption, or publishes a misleading claim. | Cannot produce valid independent evidence merely by changing a status word if authoritative checking and threshold controls are implemented as planned. Those controls do not exist yet. |
| ADV-005 | A future malicious source, proof, certificate, package, object, or evidence author targets parsers, semantics, resource limits, claim binding, and compiler transitions. | Has no such Orange parser or checker to attack in the current docs-only stage. |
| ADV-006 | A future remote, local co-resident, physical-profile, or well-intentioned integrating party chooses inputs, observes outputs/leakage, violates API preconditions, or runs outside the declared target model. | Physical resistance and behavior outside a named target/leakage profile are not implied claims. |
| ADV-007 | The hosting platform, operating system, compiler, linker, CPU, firmware, accelerator, or entropy provider behaves maliciously or outside its model. | Is not made trustworthy by an Orange proof; impact must remain an explicit assumption or be reduced by independent checking and diversity. |
| ADV-008 | A well-intentioned maintainer makes a review, configuration, transcription, release, or recovery mistake. | Cannot waive a documented assurance stop-ship condition by labeling the mistake operational. |

## Control register

Control IDs are stable and state exactly what exists. `Target` controls are not
compliance evidence.

| Control ID | Control and evidence | Enforcement state | Known gap or residual risk |
| --- | --- | --- | --- |
| CTL-001 | Public Git history, structured issue forms, pull-request template, CODEOWNERS in [`.github/`](../../.github/), and active `Protect main` ruleset `18810248` | Current and platform-enforced | Rules require pull requests, strict checks, resolved conversations, squash-only linear history, and no bypass actor, but zero approvals and sole ownership cannot provide independent review. |
| CTL-002 | Private vulnerability reporting plus triage and disclosure policy in [`SECURITY.md`](../../SECURITY.md) | Current, platform-enabled and documented | One steward; no staffed independent PSIRT, encrypted alternative channel, or tested continuity path. |
| CTL-003 | GitHub secret scanning and push protection; candidate credential exclusions in [`.gitignore`](../../.gitignore) | Current platform control; branch-candidate repository rule | Non-provider patterns and validity checks were disabled at the snapshot; scanners cannot guarantee absence and cannot undo exposure. |
| CTL-004 | Full-commit-SHA and selected-source requirements in repository settings and [`DEPENDENCY_POLICY.md`](../../DEPENDENCY_POLICY.md); candidate workflows pin repository Actions to 40-character SHAs and execute Scorecard directly at `sha256:2dd6a6d60100f78ef24e14a47941d0087a524b4d3642041558239b1c6097c941` | Current platform settings and policy; branch-candidate source with required PR execution | The six Action repositories and separately admitted Scorecard image remain upstream trust. Content addressing does not make their code trustworthy, mirror it, or eliminate publisher, registry, provenance, runner, and host compromise. |
| CTL-005 | Repository workflow token default is read-only; each candidate workflow begins with `permissions: {}` and grants per-job minimums | Current platform default; branch-candidate source with required PR execution | Scorecard grants only `security-events: write` for SARIF upload; public publication and OIDC are disabled, and the write-capable job remains off untrusted events. |
| CTL-006 | PR workflows use `pull_request`, no `pull_request_target`, no configured repository or environment secrets, only a job-scoped `GITHUB_TOKEN` limited to `contents: read`, bounded timeouts, concurrency, and checkout with `persist-credentials: false` | Current branch source; hosted PR execution demonstrated and exact check contexts required by ruleset `18810248` | PR content includes executable repository scripts; runner and Action compromise remain external assumptions. |
| CTL-007 | [`GOVERNANCE.md`](../../GOVERNANCE.md) discloses sole stewardship, forbids independent/mature claims, and requires future separation of duties | Current documented constraint | No second maintainer or technically enforced non-author review; governance D-019 is not ratified. |
| CTL-008 | [`CONTRIBUTING.md`](../../CONTRIBUTING.md) and Required CI block third-party merge until licensing terms close; ruleset `18810248` requires a branch, pull request, checks, and resolved conversations | Current documented and platform-enforced constraint | Legal decision D-018 remains blocked, and zero approvals cannot supply independent review. |
| CTL-009 | [`docs/ASSURANCE.md`](../ASSURANCE.md) defines fail-closed claim outcomes, explicit assumptions/non-claims, and non-waivable stop-ship conditions | Proposed constitution, not ratified implementation | No checker, claim registry, release gate, or independent assurance authority exists. |
| CTL-010 | [`DEPENDENCY_POLICY.md`](../../DEPENDENCY_POLICY.md), candidate Dependabot/dependency-review configuration, and candidate Scorecard workflow define admission and surveillance | Current policy; required dependency review succeeded at the PR head; Dependabot and trusted-event Scorecard execution pending | No product dependency graph exists; future manifests, SCA exceptions, VEX, and admission records remain to be implemented. |
| CTL-011 | [`RELEASE_POLICY.md`](../../RELEASE_POLICY.md) forbids product release and specifies identity, reproducibility, signing, provenance, and recovery gates | Current documented prohibition; future target | No release authority, keys, builders, signatures, registry, or drills exist. |
| CTL-012 | Provisional schemas and negative/positive conformance fixtures keep Gate 0 claims, trust, provenance, and repository-control observations explicit | Current architecture evidence | Passing schema checks proves shape only, not truth, soundness, provenance, or control operation. |
| CTL-013 | Planned authoritative checker, independent checker, canonical formats, resource limits, and adversarial corpora | Target only | No implementation, proof, fuzzing result, or independent review exists. |
| CTL-014 | Planned verified/certificate-checked compiler transitions and final-object validation | Target only | No IR, compiler, target model, artifact, or preservation evidence exists. |
| CTL-015 | Planned hermetic builds, two independent rebuilds, signed provenance, transparency evidence, TUF-style roles, SBOM/CBOM, and recovery drills | Target only | No build/release infrastructure or independent principals exist. |
| CTL-016 | Planned secrecy typing, named target leakage profiles, binary inspection, differential testing, and laboratory evidence | Target only | No leakage semantics, implementation, target choice, or measurement evidence exists. |
| CTL-017 | [`SECRETS_AND_INCIDENTS.md`](SECRETS_AND_INCIDENTS.md) inventories current/future credential classes and defines least-scope custody, rotation, revocation, containment, recovery, evidence, communication, and synthetic exercises | Current documented control | Account factors are unverified; exercises, independent PSIRT continuity, and future key stores/roles do not yet exist. |
| CTL-018 | The exact [`assets/brand/`](../../assets/brand/) inventory, owner-specific CODEOWNERS route, byte-level manifest, binary Git attributes, and repository-policy SHA-256 admissions protect the steward-designated working identity assets | Current branch source; local validation required before merge | D-017 and D-018 remain blocked; C2PA claims are preserved but not independently verified, and content addressing does not prove rights or safe decoder behavior. |

## Entry points and attack surfaces

| Surface | How reached | Boundary | Notes and evidence |
| --- | --- | --- | --- |
| Public issues and PR metadata | GitHub web/API | TB-001 | Untrusted text and links reach maintainers and some automation. Forms live in [`.github/ISSUE_TEMPLATE/`](../../.github/ISSUE_TEMPLATE/). |
| Git commits and repository files | Fork/branch/PR or privileged push | TB-001, TB-003 | Repository scripts and workflow definitions are security-sensitive even before product code. |
| Tracked brand images | Git checkout, GitHub rendering, README clients, or downstream reuse | TB-001, TB-003 | Eight PNG/JPEG files are inert to repository tooling but reach external image decoders; exact digest admission and provenance records do not make every decoder safe. |
| GitHub administration and owner recovery | Authenticated GitHub UI/API/credential transport | TB-002 | Sole-owner compromise has broad blast radius; current access is recorded in [`OSPS_BASELINE.md`](OSPS_BASELINE.md). |
| GitHub Actions PR runs | `pull_request` and `merge_group` events | TB-004 | Treat fork content, repository scripts, and parsed documents as attacker controlled. |
| Trusted Actions runs | Push, schedule, or manual dispatch | TB-005 | Scorecard can upload SARIF but cannot request OIDC; event restrictions and minimum permissions remain critical. |
| Private vulnerability intake | GitHub security advisory form | TB-006 | Reports may contain embargoed exploit information. See [`SECURITY.md`](../../SECURITY.md). |
| Future parser, package client, checker, and LSP | Files, packages, proof objects, editor input | TB-008, TB-009 | Must reject malformed, cyclic, oversized, ambiguous, and resource-exhausting inputs. |
| Future compiler, linker, and foreign ABI | Source/Core/IR/object input and caller buffers | TB-010, TB-011 | Must bind claims to exact bytes, targets, ABI contracts, and failure behavior. |
| Future registry and update client | Package publication/resolution and update metadata | TB-012 | Must resist namespace takeover, downgrade, freeze, rollback, key compromise, and malicious packages. |

## Top abuse paths

1. **TM-001 — corrupt the authoritative plan:** ADV-002 compromises the sole
   owner, crosses TB-002, and uses administrator authority to weaken TB-003 or
   merge a zero-approval pull request that changes a decision or security rule
   without independent review. Consumers mistake the altered repository for
   authorized project direction.
2. **TM-002 — turn validation into privileged code execution:** ADV-001 changes a
   workflow or repository script in a PR. A future configuration accidentally
   exposes a secret or write token on TB-004. The attacker executes the changed
   code on a runner and exfiltrates the credential or modifies project state.
3. **TM-003 — compromise an admitted tool:** ADV-003 compromises an Action or
   downloadable release already pinned by CTL-004. CI executes the malicious
   bytes across TB-005, falsifying a check or stealing the narrowly scoped token.
4. **TM-004 — disclose a credential:** ADV-008 commits a credential or prints it
   in a log. CTL-003 misses an unsupported pattern or is bypassed. ADV-001 uses
   the credential before revocation and history/log cleanup.
5. **TM-005 — substitute evidence for another subject:** ADV-005 supplies a valid
   proof, test, or certificate for one source/target tuple but binds it to
   different artifact bytes across TB-008 through TB-010. A release makes a
   false assurance claim without forging the original evidence.
6. **TM-006 — forge or disable proof checking:** ADV-005 triggers parser
   disagreement, an unsound axiom, cyclic expansion, resource exhaustion, or a
   checker bug across TB-009. A false claim is accepted or verification is made
   unavailable at scale.
7. **TM-007 — exploit the last mile:** ADV-003 or ADV-007 changes a compiler pass,
   assembler/linker result, target model, or object after a valid source proof.
   TB-010 emits bytes whose functional or leakage behavior is not covered by the
   advertised claim.
8. **TM-008 — corrupt standards intent:** ADV-004 or ADV-008 omits an erratum,
   misreads a clause, substitutes vectors, or suppresses dissent at TB-007. The
   internally verified implementation is nevertheless nonconformant or unsafe.
9. **TM-009 — publish a forged or rollback release:** ADV-002 or ADV-003 controls
   source acceptance, a builder, signing identity, registry, or update role at
   TB-012. Users receive malicious or old bytes with misleading provenance.
10. **TM-010 — manufacture maturity:** ADV-004 uses sole-owner authority to label
    proposed controls, schema-valid fixtures, or green but incomplete CI as a
    certification. AS-011 is damaged and adopters rely on guarantees Orange has
    not established.
11. **TM-011 — expose an embargoed report:** ADV-008 routes a vulnerability into
    a public issue, PR, commit, or CI log instead of TB-006, or sole-steward
    unavailability prevents timely triage. Exploit detail becomes public before
    containment.
12. **TM-012 — violate the cryptographic deployment model:** ADV-006 supplies
    overlapping buffers, reuses a nonce, selects an unsupported target, observes
    unmodeled leakage, or receives ambiguous authentication failure at TB-011.
    Correct primitive mathematics fails to protect real users.

## Threat register

Likelihood assesses the named stage. `Future` means the entry point is absent
today; the impact rank describes the intended product if introduced without the
required control. Reviews must replace conditional ranks with deployment facts.

| Threat ID | Source, boundaries, assets | Threat action and impact | Existing controls and evidence | Required treatment | Likelihood | Impact | Priority | Residual risk | Owner, review, status |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| TM-001 | ADV-002/004/008; TB-002/003; AS-001/002/003/011 | Compromise or unilateral authority changes or erases authoritative decisions, settings, or history. | CTL-001, CTL-007, CTL-008; ruleset `18810248` constrains ordinary Git updates, and public history aids detection. | Preserve the active no-bypass ruleset and exact required checks; test safe direct-update rejection. Add a non-author maintainer before claiming independent review; establish recovery and access review. | Medium-high: one administrator and zero required approvals, despite protected Git updates | High | High | Owner or platform compromise can weaken settings or abuse admin recovery; zero approvals do not create independent review. | Bootstrap Steward; each rules/access change and quarterly; `open-current` |
| TM-002 | ADV-001/002; TB-001/004; AS-003/004 | Attacker-controlled workflow, script, or metadata reaches a privileged runner and steals credentials or changes state. | CTL-005/006; required PR jobs demonstrated read-only source access, no configured repository or environment secrets, only a job-scoped read-only `GITHUB_TOKEN`, timeouts, and no `pull_request_target`. | Policy-check every workflow diff; keep privileged jobs off PR events; validate metadata before shell use; require CI and security review on workflow paths. | Low now: demonstrated PR jobs have read-only permissions; reassess every permission or event change | High | Medium | Actions/runner isolation and future permission edits remain trusted. | CI/Release authority, currently Bootstrap Steward; every workflow change; `candidate-control` |
| TM-003 | ADV-003; TB-005/012; AS-004/009 | Compromised action, tool, image, registry, archive, or dependency falsifies results or executes malicious code. | CTL-004/010; current selected-action/full-SHA settings plus candidate full SHAs, the content-addressed Scorecard runtime, download digests, and successful required PR dependency execution. | Preserve the minimum allowlist and exact container admission; retain provenance/admission records; archive dependencies; verify signatures where available; use hermetic release inputs and independent rebuilds. | Medium: admitted third-party CI already executes with narrow PR permissions | High | High | A selected full SHA or digest identifies malicious bytes as faithfully as good bytes; admission, publisher provenance, registry availability, runner integrity, and monitoring remain human or external judgments. | Dependency and CI authority, currently Bootstrap Steward; each admission/update and weekly surveillance; `candidate-control` |
| TM-004 | ADV-001/002/008; TB-001/002/004/005; AS-003/004/005/010 | Secret enters source, artifact, log, cache, or untrusted job and is used before revocation. | CTL-003/005/006/017; secret scanning and push protection enabled, minimal workflow permissions, no product keys, and a fail-closed lifecycle/playbook is documented. | Exercise synthetic leak/revocation paths; enable broader scanning if available; keep release/root keys out of GitHub; rotate immediately and treat history deletion as insufficient. | Medium: humans and tooling can leak unsupported patterns | High | High | Detection is not prevention for every secret; account custody and incident execution are not independently verified. | Security authority, currently Bootstrap Steward; every alert/credential event and quarterly; `open-current` |
| TM-005 | ADV-004/005; TB-008/009/010/012; AS-006/008/009/011 | Valid evidence is rebound, omitted, downgraded, or confused across source, target, artifact, or claim context. | CTL-009/012 specify explicit subjects, digests, contexts, assumptions, and fail-closed outcomes. | Ratify canonical schemas; bind complete claim closure to exact bytes and versions; independently check bundle traversal; add substitution, omission, downgrade, and cross-target negative tests. | Future | High | High | Schema validity cannot prove truthful binding; human standards intent remains an assumption. | Assurance/TCB authority; every schema/claim change and release; `future-blocking` |
| TM-006 | ADV-005/007; TB-008/009; AS-006 | Malformed or adversarial proofs exploit unsoundness, parser differential, resource exhaustion, or hidden axioms. | CTL-009/013 target small deterministic checker, axiom ledger, resource bounds, independent implementation, and adversarial corpus. | Prove the checked relation sound; fuzz and mutate accepted objects; test malformed/cyclic/oversized inputs; enforce canonical decoding and budgets; obtain independent logic/implementation audit. | Future | High | Critical | A shared semantic error can survive multiple implementations; audit and diverse formulations remain necessary. | Assurance/TCB Board; every checker/format/axiom change and release; `future-stop-ship` |
| TM-007 | ADV-003/004/007/008; TB-010/011; AS-006/008/009 | Compiler, encoder, linker, ABI, or target behavior diverges from proved source or promised leakage behavior. | CTL-009/014/016 require checked transitions, target-indexed claims, differential testing, and final-byte inspection. | Give each stable IR executable semantics; verify or certificate-check each pass; bind object bytes and ABI; test real hardware and emulators; forbid silent target fallback. | Future | High | Critical | OS, firmware, CPU, toolchain, and unmodeled microarchitecture remain explicit assumptions. | Compiler and Assurance authorities; every pass/target/profile change and release; `future-stop-ship` |
| TM-008 | ADV-004/008; TB-007; AS-002/007/011 | Wrong, stale, or selectively interpreted standard/erratum/vector becomes authoritative source intent. | CTL-009/012 require exact provenance, rights, clause links, errata, vectors, and independent transcription review. | Pin publication/errata/vector digests; archive permitted inputs; obtain independent cryptographer review; run mature independent implementations and official vectors as separate evidence. | Medium now for planning; High once packages exist | High | High | Formal proof can preserve a human transcription mistake perfectly. | Standards and Cryptography authorities; each upstream change and package admission; `open-design` |
| TM-009 | ADV-002/003/004/007; TB-003/005/012; AS-003/004/009/010 | Source, builder, signer, registry, or update role is compromised, enabling forgery, rollback, freeze, or unrecoverable loss. | CTL-007/011/015 prohibit current release and require separation, independent rebuilds, provenance, threshold roots, and drills. | Separate source/build/sign/publish/root principals; use hermetic inputs, signed provenance, TUF-style roles, release/tag rules, immutable publication, monitoring, revocation, and rehearsed recovery. | Future | High | Critical | Coordinated principal or platform compromise and dependency on external transparency remain residual. | Release Engineering and PSIRT; every release/key/registry change and drill; `future-stop-ship` |
| TM-010 | ADV-004/008; TB-002/003; AS-002/011 | Proposed, partial, or synthetic evidence is presented as mature assurance or compliance. | CTL-007/009/012; repository explicitly distinguishes proposed, target, current, and unsupported states. | Require machine-readable claim status and evidence; independent approval for critical assertions; release-facing claim review; block words such as certified unless exact external scope is supplied. | High: sole owner and public planning | High | High | Readers can ignore qualifications; governance independence is unavailable today. | Project owner and future Assurance Board; every public claim and release; `open-current` |
| TM-011 | ADV-001/002/008; TB-001/006; AS-005/011 | Vulnerability details are disclosed publicly, mishandled, or left untriaged. | CTL-002/017; private reporting enabled, public issue redirection, response targets, evidence handling, containment, and notification are documented. | Staff an independent PSIRT; create continuity; exercise intake with synthetic data; minimize attachments/access; publish advisories only after coordinated remediation. | Medium: private path exists but one-person availability | Medium | Medium | Reporter error, GitHub outage, owner unavailability, or an unexercised playbook can still expose or delay a case. | Bootstrap Steward then PSIRT; each report and quarterly drill; `open-current` |
| TM-012 | ADV-006/007/008; TB-011; AS-007/008/011 | Misuse, target mismatch, unmodeled leakage, entropy failure, or ambiguous failure behavior defeats real cryptographic security. | CTL-009/016 define independent claim dimensions, non-claims, explicit contracts, named leakage models, and layered evidence. | Ratify finite profiles; design misuse-resistant APIs; type and test buffer/nonce/state rules; bind entropy and platform contracts; add target binary and specialist lab evidence where claimed. | Future | High | Critical | Cryptographic hardness, foreign callers, hardware, and behavior outside named profiles remain assumptions/non-claims. | Cryptography, Target, and Assurance authorities; each API/target/profile/standard change and release; `future-stop-ship` |
| TM-013 | ADV-001/002/004/008; TB-001/003; AS-011/012 | A substituted, malformed, deceptively derived, or falsely attributed image corrupts project identity, strips provenance, overstates rights, or targets a viewer's decoder. | CTL-001/018 close the official binary inventory to exact paths and digests, route ownership, preserve supplied bytes, and state the D-017/D-018 boundary. | Keep originals immutable; review decoded content and metadata; verify C2PA independently before making a signed-provenance claim; add derived assets rather than overwriting sources; reassess every image format or rendering path. | Low: only the steward can merge and the bytes are digest-bound | Medium | Low | A trusted admitted file can still be legally encumbered, misleading, or dangerous to a vulnerable external decoder; sole stewardship provides no independent visual or rights review. | Bootstrap Steward; every brand-asset or identity change; `open-current` |

## Criticality calibration

- **Critical:** a condition that can make Orange accept false proofs, ship
  incorrect cryptography, violate a promised confidentiality profile, or
  authorize malicious/rollback release bytes with broad downstream trust.
  Examples: exploitable checker unsoundness (TM-006), unchecked native
  miscompilation (TM-007), or release-root compromise without recovery (TM-009).
- **High:** compromise of a central integrity or confidentiality asset with a
  plausible path, even if no product release exists yet. Examples: direct
  unauthorized change to `main` (TM-001), CI supply-chain compromise (TM-003),
  or a misleading assurance claim that drives unsafe adoption (TM-010).
- **Medium:** bounded compromise, delay, or exposure with available containment
  and no demonstrated product-wide false assurance. Examples: an unprivileged
  PR runner attack under current permissions (TM-002), delayed private-report
  triage (TM-011), or a security result that fails closed without release impact.
- **Low:** low-sensitivity disclosure or transient availability loss with no
  claim, release, credential, or durable-record impact. Examples: spam blocked
  by issue templates, a scheduled external-link audit delay, or loss of a
  regenerable non-authoritative CI artifact.

No current issue is downgraded merely because the project is young. Conversely,
a future critical impact is not evidence that an exploitable Orange product
exists today.

## Assurance stop-ship linkage

The following links the stable threats to the non-waivable conditions in
[`docs/ASSURANCE.md`](../ASSURANCE.md#8-stop-ship-conditions). It does not change
those conditions.

| Assurance condition | Principal threat IDs | Required disposition before release |
| --- | --- | --- |
| Proof-soundness flaw | TM-005, TM-006 | Fix and independently revalidate the checker, formats, affected proof closure, and every dependent claim. |
| Incorrect cryptographic output | TM-007, TM-008, TM-012 | Correct source/semantics/compiler/package as coupled artifacts and rerun the complete affected evidence set. |
| Secret-dependent behavior within a promised profile | TM-007, TM-012 | Withdraw or narrow the profile, fix the source-to-binary path, and repeat formal, binary, hardware, and review evidence. |
| Undocumented axiom, TCB expansion, foreign boundary, or claim downgrade | TM-005, TM-006, TM-007, TM-010 | Restore explicit closure and review; a wording change alone cannot cure missing evidence. |
| Meaning-changing semantic ambiguity | TM-005, TM-006, TM-007 | Resolve normatively, add independent parsing/semantics evidence and migration analysis, then recheck dependents. |
| Failed reproducibility, signature, provenance, update, or rollback protection | TM-003, TM-009 | Stop publication, repair the complete release path, rehearse recovery, and issue new immutable identities. |
| Unresolved critical/high security or assurance finding | Any applicable threat | Understand scope and impact, remediate, retest, and obtain required independent review; no risk acceptance can waive it. |
| Relevant unreviewed standards erratum | TM-008, TM-012 | Complete provenance and cryptographer review, update affected packages/claims, and notify downstreams. |
| Audit finding whose impact is unknown | Any applicable threat | Keep release blocked until impact and claim closure are known. |

At Gate 0, release is already prohibited by governance, licensing, staffing, and
technical prerequisites. A green threat table does not override any other gate.

## Mandatory update and review protocol

### Update triggers

The same pull request must update this document when it:

1. adds or changes any workflow, Action permission, secret, runner, environment,
   deployment, release, registry, webhook, or external service;
2. changes repository visibility, ownership, collaborators, MFA policy, branch
   or tag rules, merge policy, security feature, or recovery arrangement;
3. adds product code, an executable parser, schema consumer, dependency or
   package manifest, network endpoint, stored data, package namespace, or
   user-controlled file format;
4. ratifies or changes a semantic Core, proof/checker boundary, axiom, TCB
   component, compiler pass, object format, target, ABI, leakage model, entropy
   contract, foreign boundary, or public claim;
5. admits a standard, erratum, vector, cryptographic package, toolchain,
   dependency, or externally validated artifact;
6. creates a tag, package, build, artifact, provenance record, signing/update
   key, release candidate, withdrawal, revocation, or support commitment;
7. discovers an incident, vulnerability, new abuse path, control failure,
   critical/high finding, standards change, or invalidated assumption; or
8. changes the applicable OSPS Baseline, NIST SSDF, SLSA, disclosure, or other
   pinned external security baseline.

### Review mechanics

- Never renumber an existing AS, ADV, TB, CTL, or TM identifier. Retire it with
  a dated tombstone and replacement link.
- For each changed boundary, update entry points, abuse paths, threat rank,
  current evidence, owner, residual risk, status, and stop-ship mapping.
- Evidence must identify the repository revision or API observation date. A
  policy, planned workflow, schema-valid fixture, or passing unrelated check is
  not operating evidence.
- The pull-request threat-impact row must name affected IDs. `No change` needs a
  reason tied to inspected boundaries.
- Until a second qualified maintainer exists, the Bootstrap Steward records the
  review but must not label it independent. TCB, cryptography, release, and
  assurance-critical reviews require the authorities in
  [`GOVERNANCE.md`](../../GOVERNANCE.md) before a product release.
- Perform a complete review at least quarterly, at every program gate, before
  any release candidate, and after every incident or recovery exercise.

## Focus paths for security review

| Path | Why it matters | Related threats |
| --- | --- | --- |
| `.github/workflows/` | Defines untrusted/trusted event separation, executable dependencies, token permissions, and security result uploads. | TM-002, TM-003, TM-004, TM-009 |
| `.github/CODEOWNERS` | Routes critical review but currently demonstrates the solo-owner independence gap. | TM-001, TM-010 |
| `SECURITY.md` | Controls private intake, response, disclosure, and explicit lack of a staffed PSIRT. | TM-004, TM-011 |
| `GOVERNANCE.md` | Defines authority, separation, succession, and non-waivable assurance gates. | TM-001, TM-009, TM-010 |
| `DEPENDENCY_POLICY.md` | Governs action, tool, product dependency, provenance, and exception admission. | TM-003, TM-009 |
| `RELEASE_POLICY.md` | Prohibits current product release and defines future source/build/sign/update separation. | TM-009, TM-010 |
| `docs/ASSURANCE.md` | Defines adversaries, claim dimensions, TCB, stop-ship conditions, and non-claims. | TM-005 through TM-012 |
| `docs/ARCHITECTURE.md` | Defines the future parser, checker, compiler, ABI, package, registry, and evidence boundaries. | TM-005 through TM-012 |
| `docs/DECISIONS.md` | Records unresolved choices whose resolution changes the attack surface and authority model. | TM-001, TM-008, TM-009, TM-010, TM-012 |
| `schemas/gate0/` | Encodes provisional claim, evidence, trust, standards, and repository-control record shapes; shape must not be confused with truth. | TM-005, TM-008, TM-010 |
| `scripts/` and `tools/` | Repository-owned code executes in CI and validates evidence/policy; changes can weaken or bypass controls. | TM-002, TM-003, TM-010 |

## Quality check

- Current and future runtime, CI/development, and release surfaces are separated.
- Every discovered current entry point maps to at least one trust boundary and
  threat.
- Every TB identifier appears in the system model and at least one threat or
  explicit future abuse path.
- Attacker-controlled, operator-controlled, and platform-controlled inputs are
  distinguished.
- Existing controls cite repository or observed platform evidence; target
  controls are labeled as targets.
- Open service context, ownership, licensing, and deployment assumptions are
  explicit rather than silently resolved.
- Stop-ship conditions and update triggers are mechanically reviewable by
  stable IDs.
