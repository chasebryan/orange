# OpenSSF OSPS Baseline evidence matrix

Status: Gate 0 readiness evidence; no conformance or maturity claim

Pinned baseline: [OpenSSF OSPS Baseline v2026.02.19](https://baseline.openssf.org/versions/2026-02-19.html)

Assessment snapshot: 2026-07-11

Owner: Bootstrap Repository Steward (`@chasebryan`)

Next scheduled review: 2026-10-11, or earlier on a trigger below

## Interpretation and scope

This matrix records what was observed, what exists only in the current
foundation change, what is merely documented, and what is missing. It is not an
OpenSSF certification, self-attestation, badge, or claim that Orange meets
Level 1, 2, or 3. A row is not satisfied merely because a policy says a future
control will exist.

Version `2026.02.19` is pinned because the baseline directs downstream users to
assess a specific version, and its maturity levels have different scopes:

- Level 1 applies to any code or non-code project with any number of maintainers
  or users. It is the only level whose project scope currently matches Orange.
- Level 2 applies to a code project with at least two maintainers and consistent
  users. Orange has repository-policy tooling but no product implementation,
  only one maintainer, and no demonstrated consistent user population.
- Level 3 applies to a code project with a large number of consistent users.
  Orange has neither. Level 3 remains the release-bearing target proposed by
  [`docs/ASSURANCE.md`](../ASSURANCE.md), not current state.

There is no Orange compiler, checker, cryptography package, product dependency,
distribution channel, or product release. Conditional release rows therefore
receive no compliance credit: `not triggered` proves only that the triggering
asset does not exist. The unresolved license decision and absent non-author
reviewer are explicit blockers, not administrative cleanup.

## Status vocabulary

| Status | Meaning |
| --- | --- |
| `Observed` | Direct repository or GitHub API evidence supports the current, triggered requirement at the snapshot. This is still not a level claim. |
| `Candidate` | Control source exists only in the unmerged foundation change, or has not yet produced a successful hosted result. It is not enforced evidence. |
| `Documented` | Policy or design text exists, but operating or automated evidence is absent. |
| `Partial` | Some material control exists, but the requirement or its intended scope is not fully evidenced. |
| `Conditional` | The baseline's trigger is absent, such as no release or package manager. This is not a pass. |
| `Gap` | Required control is absent, disabled, unenforced, or contradicted by evidence. |
| `Unverified` | The state could not be independently observed with available repository-level evidence. |

## Evidence basis

### Snapshot boundary

The inspection used the public repository and authenticated GitHub REST
readback for `chasebryan/orange` on 2026-07-11. The committed branch base
was `f50d35227a04831d459b3358db65b80093e8123a`; candidate files in this
foundation change were inspected separately and are never labeled enforced.
GitHub settings can drift without a Git commit, so every API observation expires
at the next mandatory review.

Evidence aliases used below:

| Evidence ID | Evidence |
| --- | --- |
| EV-GH-01 | Read-only GitHub repository API: public personal-account repository `chasebryan/orange`; default branch `main`; one collaborator, `chasebryan`, with admin; Issues enabled; Discussions disabled. |
| EV-GH-02 | GitHub security APIs: Dependabot alerts and security updates enabled; secret scanning and push protection enabled; non-provider pattern scanning and validity checks disabled; private vulnerability reporting enabled. |
| EV-GH-03 | GitHub Actions APIs: Actions enabled; sources restricted to the exact seven repositories used by candidate workflows; broad GitHub-owned and verified-publisher allowances disabled; full-SHA pinning required; default workflow token permission `read`; workflows cannot approve pull-request reviews; all external fork runs require approval. |
| EV-GH-04 | GitHub rules APIs: no repository ruleset and no protection for `main`. Repository setting `web_commit_signoff_required` is false. |
| EV-GH-05 | GitHub code-scanning API: CodeQL default setup `configured`, no language currently detected, `extended` query suite, standard runner, and `remote_and_local` threat model. Configuration without a language/result is not source coverage; Scorecard SARIF is not CodeQL analysis. |
| EV-GH-06 | Repository APIs: squash-only merge, auto-merge and branch update enabled, merged branches deleted, wiki disabled, and immutable future releases enabled. No product release exists or is authorized. |
| EV-REP-01 | Public Git URL and Git history identify source changes and authors. `git ls-files` plus file-type inspection found no generated executable or unreviewable binary at the snapshot. |
| EV-POL-01 | [`GOVERNANCE.md`](../../GOVERNANCE.md), [`CONTRIBUTING.md`](../../CONTRIBUTING.md), and [`.github/CODEOWNERS`](../../.github/CODEOWNERS) define current authority, branch workflow, ownership routing, and the solo-owner limitation. |
| EV-POL-02 | [`SECURITY.md`](../../SECURITY.md), [`SUPPORT.md`](../../SUPPORT.md), and the [issue configuration](../../.github/ISSUE_TEMPLATE/config.yml) define private reporting, response targets, public defect routing, and the lack of a staffed PSIRT. |
| EV-POL-03 | [`DEPENDENCY_POLICY.md`](../../DEPENDENCY_POLICY.md) and [`RELEASE_POLICY.md`](../../RELEASE_POLICY.md) define dependency admission, immutable references, release prohibition, future artifacts, separation, signing, and recovery. |
| EV-POL-04 | [`docs/ASSURANCE.md`](../ASSURANCE.md), [`docs/ARCHITECTURE.md`](../ARCHITECTURE.md), and [`THREAT_MODEL.md`](THREAT_MODEL.md) define proposed assurance, future actors/interfaces, stop-ship conditions, and current/future threats. |
| EV-POL-05 | [`docs/DECISIONS.md`](../DECISIONS.md#d-018--licenses) leaves licenses and inbound terms blocked; there is no `LICENSE`, `COPYING`, accepted DCO, or contributor agreement. |
| EV-CAND-01 | Candidate workflows in [`.github/workflows/`](../../.github/workflows/) use full action SHAs, top-level empty permissions, per-job minimum permissions, timeouts, concurrency, non-persistent checkout credentials, and no `pull_request_target`. No hosted result or required-check enforcement existed at the snapshot. |
| EV-CAND-02 | Candidate [Dependabot configuration](../../.github/dependabot.yml), [dependency-review policy](../../.github/dependency-review-config.yml), repository validators, schemas, and conformance fixtures exist in the working change; merge and successful hosted execution remain pending. |

### Current GitHub control-plane facts

| Setting | Observed value | Interpretation |
| --- | --- | --- |
| Visibility and ownership | Public personal-account repository; owner `chasebryan`; default `main` | Public source/history is available. Personal-account ownership has one ultimate owner and less granular collaborator roles than an organization. |
| Current access | Only `chasebryan`, admin | Exact current list, but also a bus-factor and independent-review gap. |
| Default-branch protection | No branch protection and no ruleset | Direct update, force-push/deletion prevention, required PR, required checks, and conversation-resolution rules are not enforced. |
| Vulnerability intake | Private vulnerability reporting enabled | Private repository-advisory intake exists; continuity depends on one steward. |
| Dependency security | Dependabot alerts and security updates enabled | Alerts and automatic security-fix support exist; product manifests do not. |
| Secret protection | Secret scanning and push protection enabled | Supported provider patterns are covered. Non-provider patterns and validity checks are disabled, so coverage is not complete. |
| Actions policy | Enabled; exactly seven Action repositories selected; broad GitHub-owned and verified-publisher allowances disabled; full-SHA pinning required; all external fork runs need approval | Source identities and mutable references are restricted. Candidate workflows have not yet completed a hosted run. |
| Workflow tokens | Default `read`; cannot approve PR reviews | Safe default. Candidate workflows further reduce permissions, but are not yet authoritative or run. |
| CodeQL | Default setup configured with extended queries and automatic detection; no language/result yet | The control is configured but dormant while `main` has no supported source. Do not claim coverage or require a CodeQL rule until a successful result exists. |
| Merge and release settings | Squash-only; auto-update/auto-merge enabled; merged branches deleted; immutable future releases enabled | Merge settings do not protect `main` without a ruleset. No release is authorized. |
| Web sign-off | Disabled | No GitHub web-commit DCO/sign-off enforcement. Legal contribution terms remain unresolved. |

## Level 1 matrix

Level 1 is the applicable current project scope. The combination of gaps and
unverified controls below means Orange makes no Level 1 conformance claim.

| Control | Status | Exact current evidence | Gap and next evidence required |
| --- | --- | --- | --- |
| OSPS-AC-01.01 | `Unverified` | EV-GH-01 shows the only privileged principal, but repository APIs do not disclose that account's MFA state. EV-POL-01 requires phishing-resistant MFA where available. | A personal repository cannot impose an organization-wide collaborator MFA policy. Record verifiable platform enforcement or move to an organization that requires MFA; never publish recovery factors. |
| OSPS-AC-02.01 | `Observed` | EV-GH-01 shows only the owner. GitHub personal repositories add collaborators through an explicit invitation, satisfying the manual-assignment branch of the requirement. | Future collaborator admission needs a dated access record and review. Personal repositories give collaborators broad write access; an organization is needed for more granular roles. |
| OSPS-AC-03.01 | `Gap` | EV-GH-04: `main` has no protection or ruleset. EV-POL-01 documents PR use but cannot prevent direct commit. | Add an active `main` ruleset requiring a PR, with no bypass, after candidate check contexts exist; test a direct push rejection. |
| OSPS-AC-03.02 | `Gap` | EV-GH-04: no rule prevents `main` deletion. | Add an active rule restricting deletion and block force pushes; verify the effective rules API and a safe negative test. |
| OSPS-BR-01.01 | `Candidate` | EV-CAND-01 does not interpolate event metadata into shell commands; workflow expressions used for concurrency are not executed as shell source. | Merge, run, and statically audit every workflow. Any future manual/event metadata reaching an interpreter must be allow-listed or safely passed through environment/arguments. |
| OSPS-BR-01.03 | `Candidate` | EV-CAND-01 PR workflows have no secrets, begin with no permissions, grant only `contents: read`, avoid `pull_request_target`, and disable persisted checkout credentials. Privileged Scorecard permissions occur only on trusted events. | Obtain successful hosted runs and protect the workflow paths/checks. Reassess on every secret, environment, self-hosted runner, or event change. |
| OSPS-BR-03.01 | `Observed` | EV-REP-01 and repository link inspection show official project channels use HTTPS. Git and GitHub links identify `https://github.com/chasebryan/orange`. | Preserve HTTPS-only link validation. A future domain, registry, chat, package, or documentation channel must be inventoried before being called official. |
| OSPS-BR-03.02 | `Conditional` | No official distribution channel or software release exists; EV-POL-03 prohibits publication. | Before distribution, require authenticated HTTPS and signed, verifiable artifact/update metadata; demonstrate downgrade and adversary-in-the-middle resistance. |
| OSPS-BR-07.01 | `Partial` | EV-GH-02 enables secret scanning and push protection; candidate ignore policy excludes common local credential files. | Non-provider patterns and validity checks are disabled. Define a secrets lifecycle, add complementary scanning where justified, and test block/revocation handling without committing a real secret. |
| OSPS-DO-01.01 | `Conditional` | No release or basic product functionality exists. EV-POL-03 forbids a product release. | Complete, tested user guidance for every released basic function is a pre-release requirement. Planning docs are not user guidance. |
| OSPS-DO-02.01 | `Conditional` | No release exists. EV-POL-02 already routes public planning defects and private vulnerabilities. | Before release, provide tested defect-report instructions for product versions, logs, reproductions, supported tuples, and sensitive-data handling. |
| OSPS-GV-02.01 | `Observed` | EV-GH-01 has public Issues enabled; structured issue forms accept planning defects, evidence, and OEP proposals. | Discussions are disabled, which is acceptable because the requirement needs one mechanism. Preserve public decision links and do not route vulnerabilities publicly. |
| OSPS-GV-03.01 | `Observed` | EV-POL-01 explains scope, workflow, evidence, review, definition of done, AI-assisted material, and the current third-party contribution prohibition. | Update immediately when licensing or contribution authority changes. |
| OSPS-LE-02.01 | `Gap` | EV-POL-05: no product source code exists, but the active project has not selected an OSI/FSF-compliant source license. | Owner/legal decision D-018 must select exact terms before product code or third-party contributions. Absence of code does not close the licensing gate. |
| OSPS-LE-02.02 | `Conditional` and `Gap` | No released software asset exists, and EV-POL-05 records no selected release license. | Select compatible release-asset and generated-output terms before any release. Never infer them from a proposed recommendation. |
| OSPS-LE-03.01 | `Gap` | EV-POL-05: no `LICENSE`, `COPYING`, or license directory exists. | After legal selection, add the exact license text and machine-readable metadata in the same reviewed change. |
| OSPS-LE-03.02 | `Conditional` and `Gap` | No release exists and no release license is selected. | Include the ratified license alongside source and release assets, then verify packaged contents. |
| OSPS-QA-01.01 | `Observed` | EV-GH-01 and EV-REP-01: the source repository is publicly readable at a static GitHub URL. | Reassess on visibility or repository transfer. The owner's repo-only operating boundary must be preserved unless explicitly changed. |
| OSPS-QA-01.02 | `Observed` | EV-REP-01: public Git history records commits, authors, and timestamps. | Protect history from force-push/deletion and preserve archival continuity. Git authorship is not proof of legal authorization or cryptographic identity. |
| OSPS-QA-02.01 | `Conditional` | There is no product package-management system or language dependency manifest. EV-CAND-02 tracks only GitHub Actions candidates. | Every admitted ecosystem must commit its direct dependency manifest and lock/inventory; offline bytes and transitive provenance remain additional Orange requirements. |
| OSPS-QA-04.01 | `Conditional` | Orange currently uses one repository, explicitly bounded to `chasebryan/orange` by EV-POL-01. | If a second codebase becomes part of Orange, add a canonical repository inventory before use; do not operate or publish elsewhere without owner direction. |
| OSPS-QA-05.01 | `Observed` | EV-REP-01 file-type inspection found no generated executable artifact. EV-POL-01 prohibits them. | Add the repository validator to required CI and inspect Git LFS/releases when introduced; a source scan does not cover external assets. |
| OSPS-QA-05.02 | `Observed` | EV-REP-01 found no unreviewable binary artifact. The README image is an external HTTPS attachment, not a tracked executable. | Define binary-fixture admission, source/provenance, review, and size policy before adding any necessary corpus. |
| OSPS-VM-02.01 | `Observed` with limitation | EV-POL-02 provides the private advisory contact path and identifies the Bootstrap Repository Steward through governance. | Add independent PSIRT contacts and continuity before public packages. One owner is discoverable but not resilient. |

## Level 2 target matrix

Level 2 does not currently apply because Orange is not a code project with two
maintainers. Existing evidence is recorded to expose rather than hide the work
remaining to become collaborative.

| Control | Status | Exact current evidence | Gap and next evidence required |
| --- | --- | --- | --- |
| OSPS-AC-04.01 | `Observed` platform default; `Candidate` source | EV-GH-03 sets default workflow permissions to `read`. EV-CAND-01 begins every workflow with `permissions: {}` and grants explicit job permissions. | Merge, run, and continuously lint workflows. Reverify the setting after repository transfer or GitHub policy change. |
| OSPS-BR-02.01 | `Conditional` | No official release exists. EV-POL-03 requires one immutable identifier spanning all relevant version axes. | Define and validate the release identifier format before the first candidate. |
| OSPS-BR-04.01 | `Conditional` | No release exists. EV-POL-03 requires changed claims, TCB/assumption deltas, security changes, and limitations. | Generate and review a functional/security changelog bound to each immutable release. |
| OSPS-BR-05.01 | `Conditional` | No build/release pipeline ingests product dependencies. Candidate CI uses standard Actions plus checksum-verified downloads under EV-CAND-01. | Ratify standardized package/build tooling, immutable inputs, and offline archives for each admitted ecosystem. |
| OSPS-BR-06.01 | `Conditional` | No release or signing authority exists. EV-POL-03 requires signatures or signed manifests and exact asset digests. | Establish independent signing roles, identity verification, transparency/archival evidence, and a tested verification command before publication. |
| OSPS-DO-06.01 | `Conditional` with documented target | No release exists. EV-POL-03 documents selection, provenance, pinning, update, rollback, and removal requirements. | Replace target prose with the actual release dependency graph, acquisition procedure, and maintenance owners. |
| OSPS-DO-07.01 | `Gap` for future product | Repository-policy Python/shell tooling runs through `make check`, but no Orange language, checker, compiler, or cryptography implementation exists. Repository checks are not product build instructions. | When the first permanent product implementation is authorized, add pinned, clean-environment, network/offline, and platform-specific build/replay instructions with tests. |
| OSPS-GV-01.01 | `Observed` current scope; structurally weak | EV-GH-01 and this snapshot list the sole sensitive-resource holder, `chasebryan`, with admin access. EV-POL-01 requires a maintainer record when another principal exists. | Inventory all repository, security-alert, CI, domain, registry, key, and release access before those resources exist; publish role-safe details without secrets. |
| OSPS-GV-01.02 | `Observed` current role; future target documented | EV-POL-01 defines the Bootstrap Repository Steward and proposed technical, assurance, release, and PSIRT authorities. | Ratify D-019, publish exact responsibilities/terms/recusal/succession, and assign real people before claiming collaborative governance. |
| OSPS-GV-03.02 | `Documented` but legally blocked | EV-POL-01 defines acceptable scope, workflow, evidence, review, and contribution quality. It rejects third-party merge until D-018 closes. | Ratify licenses and DCO/CLA terms, then test the guide with an eligible external contribution. |
| OSPS-LE-01.01 | `Gap` | EV-GH-04 has web sign-off disabled; EV-POL-05 has no accepted DCO or contributor agreement. Third-party merges are prohibited rather than falsely treated as authorized. | Select legal provenance terms, require assertion on every code commit, and add enforcement/validation with counsel-approved wording. |
| OSPS-QA-03.01 | `Gap` | EV-GH-04: no required status checks or protected branch. Candidate checks have not run and cannot block merge. | After successful check discovery, require `Required CI / docs-policy-workflows` and `Dependency Review / policy` on `main`; verify failure blocks merge and bypass policy is explicit. |
| OSPS-QA-06.01 | `Candidate` | EV-CAND-01 defines repository policy tests before acceptance; candidate validators and conformance checks are under development in the same foundation change. | Run locally and on GitHub, require the check, prove a negative mutation fails, and expand the suite with every product component. |
| OSPS-SA-01.01 | `Conditional` with design evidence | No release exists. EV-POL-04 documents proposed actors, components, and flows, including this stable-ID threat model. | Update from intended architecture to the exact deployed system and all human/service actors before release. |
| OSPS-SA-02.01 | `Conditional` | No released external software interface exists. Architecture discusses planned CLI/LSP/ABI/registry surfaces but they are not specifications. | Inventory and document every actual external interface, protocol, parser, error contract, privilege, and version before release. |
| OSPS-SA-03.01 | `Conditional` with early assessment | No release exists. EV-POL-04 supplies a Gate 0 security assessment, not an implementation assessment. | Repeat against executable code, dependencies, deployments, findings, and test results before each release-bearing gate. |
| OSPS-VM-01.01 | `Observed` policy | EV-POL-02 defines coordinated disclosure, safe handling, one-business-day acknowledgement and three-business-day assessment targets. | Staff and exercise the process; targets are not an SLA and one-person continuity is weak. |
| OSPS-VM-03.01 | `Observed` | EV-GH-02 verifies private vulnerability reporting; EV-POL-02 links directly to it and prohibits public reports. | Test notifications and continuity without submitting a fake vulnerability. Add an independent private contact before public packages. |
| OSPS-VM-04.01 | `Conditional` | No Orange software vulnerability or release advisory exists. EV-POL-02 promises published advisories identifying affected versions and invalidated claims. | Define the advisory/Vulnerability Disclosure Report publication record and exercise it during an incident simulation. |

## Level 3 target matrix

Level 3 is a future release-bearing target only. None of these rows should be
used to imply present maturity or a large user base.

| Control | Status | Exact current evidence | Gap and next evidence required |
| --- | --- | --- | --- |
| OSPS-AC-04.02 | `Candidate` | EV-CAND-01 grants read-only contents to validation jobs; only trusted-event Scorecard has `security-events: write` for SARIF upload. Public Scorecard publication and OIDC are disabled. | Complete hosted main validation, protect workflow changes, document why the remaining write is necessary, and re-review every permission delta. |
| OSPS-BR-01.04 | `Candidate` | Candidate manual workflows accept no user-defined inputs, grant only `contents: read`, and do not interpolate collaborator input into shell. The privileged Scorecard job has no manual trigger and is hard-gated to `main`. | Any future dispatch input or write-capable manual job must have an allow-list, length/type constraints, trusted-ref gate, safe interpreter boundary, and negative tests. |
| OSPS-BR-02.02 | `Conditional` | No release asset exists. EV-POL-03 requires each asset and claim to bind to immutable identities/digests. | Generate an asset manifest and prove every archive, SBOM, evidence bundle, signature, and binary maps to the release ID. |
| OSPS-BR-07.02 | `Documented` and `Partial` | [`SECRETS_AND_INCIDENTS.md`](SECRETS_AND_INCIDENTS.md) defines current/future classes, owners, stores, issuance, rotation, expiry, audit, revocation, recovery, and synthetic exercises. | Verify account controls, run exercises, staff independent response/release roles, and instantiate inventories for every future credential. |
| OSPS-DO-03.01 | `Conditional` | No release. EV-POL-03 plans hashes, signatures, provenance, and update metadata. | Publish tested offline and online integrity/authenticity verification instructions for exact assets. |
| OSPS-DO-03.02 | `Conditional` | No release author or signing identity exists. | Document trusted release identities, keyless/threshold verification, rotation, compromise, and historical verification without exposing private keys. |
| OSPS-DO-04.01 | `Conditional` | No release. [`SUPPORT.md`](../../SUPPORT.md) explicitly says none is supported and labels future duration as proposed. | Publish funded scope and exact start/end dates for every released support tuple. |
| OSPS-DO-05.01 | `Conditional` | No version receives updates today. Support and withdrawal behavior is only proposed. | Publish EOL and security-update cessation statements and downstream notices for each release. |
| OSPS-GV-04.01 | `Documented` and `Partial` | EV-POL-01 requires least privilege, role criteria, conflict handling, prompt offboarding, and quarterly access review. | Ratify objective privilege-escalation criteria, record reviewer/decision/effective date, and technically restrict roles. Solo owner cannot independently review own escalation. |
| OSPS-QA-02.02 | `Conditional` | No compiled release exists. EV-POL-03 requires SPDX SBOM and CycloneDX SBOM/CBOM. | Generate, validate, sign/bind, and ship complete component inventories for actual release assets. |
| OSPS-QA-04.02 | `Conditional` | One repository and no release. | If a release spans repositories, inventory them and prove each enforces equivalent or stronger source, review, CI, dependency, and release controls. |
| OSPS-QA-06.02 | `Documented` and `Candidate` | EV-POL-04 documents PR, merge, nightly, weekly, and release-candidate test classes. EV-CAND-01 names current repository checks. | Publish exact executable commands, environments, cadence, expected outputs, and retained evidence after the candidate validators stabilize. |
| OSPS-QA-06.03 | `Documented` | EV-POL-01 requires tests or replayable evidence where applicable and coupled updates for major effects. | Ratify explicit major-change test obligations and enforce them through review/coverage/conformance policy once code exists. |
| OSPS-QA-07.01 | `Gap` | EV-GH-01/04 and EV-POL-01: one owner, no non-author collaborator, no required approval. CODEOWNERS names only the author/owner. | Add a qualified independent maintainer, require at least one non-author human approval on `main`, prevent self-approval/bypass, and verify a negative merge test. This cannot be solved by a bot or the author's second account. |
| OSPS-SA-03.02 | `Conditional` with Gate 0 model | [`THREAT_MODEL.md`](THREAT_MODEL.md) covers current and future boundaries with stable IDs, abuse paths, controls, residual risk, owners, and triggers. No released critical code path exists. | Re-perform threat and attack-surface analysis against real code/deployments before release and on every mandatory trigger. |
| OSPS-VM-04.02 | `Conditional` | No product component vulnerability or non-exploitability decision exists; no VEX process has run. | Define CycloneDX or CSAF VEX generation, review, expiry, evidence, and correction procedures before suppressing any component finding. |
| OSPS-VM-05.01 | `Gap` | Candidate dependency review fails at `moderate` vulnerability severity, but EV-POL-03 has no complete SCA vulnerability/license remediation deadlines or risk thresholds. | Ratify severity, exploitability, malicious-package, license, SLA, exception, VEX, and expiry thresholds for every dependency class. |
| OSPS-VM-05.02 | `Documented` and `Candidate` | EV-POL-03 says security policy failures fail closed and cannot waive assurance gates; EV-CAND-02 proposes dependency review. | Make the check required, exercise violation/exception paths, and block every release on unresolved policy violations. |
| OSPS-VM-05.03 | `Candidate` | EV-CAND-02 proposes PR dependency review, Dependabot surveillance, vulnerability/license checks, and Scorecard signals. | Merge and require the check; add explicit malicious-dependency policy and ecosystem coverage; demonstrate a known-vulnerable dependency mutation is blocked. |
| OSPS-VM-06.01 | `Gap` | No SAST remediation threshold or exception/expiry policy exists. | Define severity, confidence, scope, remediation time, false-positive evidence, owner, expiry, and release-blocking behavior before SAST is authoritative. |
| OSPS-VM-06.02 | `Partial` | EV-GH-05: CodeQL default setup is configured with extended queries, but no supported language or scan result exists. Scorecard SARIF is posture data, not source-code weakness analysis. | Confirm automatic detection after supported source reaches `main`; obtain a successful baseline, then require an appropriate CodeQL threshold and test that a qualifying alert blocks merge. |

## Priority gaps and gates

The following must remain visible even if all candidate checks pass:

1. **Protected-main gap:** no PR, required-check, force-push, deletion,
   conversation-resolution, or bypass restriction is active
   (OSPS-AC-03.01, OSPS-AC-03.02, OSPS-QA-03.01).
2. **Independent-review gap:** one owner cannot provide the non-author approval
   required by OSPS-QA-07.01 or Orange's mature governance. A bot, CODEOWNERS
   request, self-review, or second account controlled by the author does not
   close it.
3. **Licensing gap:** D-018 is blocked, no license file or inbound assertion
   exists, and third-party merges/product code remain prohibited
   (OSPS-LE-01.01, OSPS-LE-02.01/02, OSPS-LE-03.01/02).
4. **MFA evidence gap:** the repository review could not independently verify
   account MFA or impose an organization requirement (OSPS-AC-01.01).
5. **Code scanning execution gap:** CodeQL default setup is configured, but no
   supported language, scan result, or SAST threshold exists
   (OSPS-VM-06.01/02). Scorecard does not substitute for SAST.
6. **Secrets and supply-chain gaps:** generic/non-provider and validity secret
   checks are disabled; the documented lifecycle is not independently exercised;
   selected Actions still depend on reviewed upstream identities; no complete
   SCA, VEX, or malicious-dependency lifecycle exists.
7. **Operational-evidence gap:** branch-candidate workflows have not yet run on
   GitHub and are not required checks. Local/static success cannot establish
   GitHub event, token, permission, SARIF, or branch-rule behavior.
8. **Release-capability gap:** no independent PSIRT, release roles, keys,
   builders, SBOM/CBOM, provenance, registry, support capacity, or recovery
   drill exists. The release prohibition is correct current behavior.

## Next-review protocol

### Mandatory triggers

Review the complete matrix, not only one row, when any of these occurs:

- a workflow, Action, runner, token permission, environment, secret, webhook,
  GitHub App, deployment, or external integration changes;
- repository ownership, visibility, collaborator access, MFA requirement,
  branch/tag rule, merge policy, security feature, or recovery setting changes;
- the candidate foundation change merges, its first hosted checks finish, or a
  required check/ruleset is enabled;
- product code, package manifests, dependencies, build instructions, interfaces,
  user data, registry, package, tag, asset, signing key, or release is added;
- D-018, D-019, or another decision changes license, contribution, governance,
  assurance, target, dependency, or release scope;
- an alert, vulnerability, incident, exception, VEX statement, failed control,
  or recovery exercise occurs; or
- OpenSSF publishes a new current OSPS Baseline version or materially corrects
  `v2026.02.19`.

### Evidence procedure

1. Pin the exact OSPS version; never silently follow `current` or `latest`.
2. Record repository revision, assessment date, assessor, scope, and every
   authenticated API endpoint used without storing credentials or private
   report content.
3. Re-read effective repository access, security/analysis, Actions, workflow
   permission, ruleset/branch, code-scanning, release, environment, webhook, and
   deployment state. A settings screenshot alone is insufficient.
4. Run repository validators and negative fixtures; obtain hosted runs for event
   and permission claims; verify required rules with safe rejection tests.
5. For each baseline row, preserve the exact ID and change status only when the
   requirement's full trigger and scope have direct evidence. `Conditional` is
   never promoted because the trigger is absent.
6. Link exceptions to owner, rationale, compensating control, expiry, and
   approval. No exception can waive an Orange assurance stop-ship condition.
7. Record review independence honestly. Until another qualified principal
   participates, label reviews steward-authored, not independent.

The next scheduled full assessment is 2026-10-11. The foundation merge, first
hosted workflow results, first CodeQL result, and protected-main configuration are
earlier mandatory triggers.

## Primary sources

- [OpenSSF OSPS Baseline v2026.02.19](https://baseline.openssf.org/versions/2026-02-19.html)
- [OSPS Baseline versioning and current-version policy](https://baseline.openssf.org/)
- [GitHub repository security and analysis settings](https://docs.github.com/en/repositories/managing-your-repositorys-settings-and-features/enabling-features-for-your-repository/managing-security-and-analysis-settings-for-your-repository)
- [GitHub Actions repository settings and full-SHA enforcement](https://docs.github.com/en/repositories/managing-your-repositorys-settings-and-features/enabling-features-for-your-repository/managing-github-actions-settings-for-a-repository)
- [GitHub Actions permissions REST API](https://docs.github.com/en/rest/actions/permissions?apiVersion=2026-03-10)
- [GitHub protected branches](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-protected-branches/about-protected-branches)
- [GitHub repository rulesets](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-rulesets/available-rules-for-rulesets)
- [GitHub private vulnerability reporting](https://docs.github.com/en/code-security/how-tos/report-and-fix-vulnerabilities/configure-vulnerability-reporting)
- [GitHub CodeQL default setup](https://docs.github.com/en/code-security/how-tos/find-and-fix-code-vulnerabilities/configure-code-scanning/configure-code-scanning)
- [GitHub personal-account repository permissions](https://docs.github.com/en/repositories/managing-your-repositorys-settings-and-features/repository-access-and-collaboration/permission-levels-for-a-personal-account-repository)
- [GitHub two-factor authentication](https://docs.github.com/en/authentication/securing-your-account-with-two-factor-authentication-2fa/about-two-factor-authentication)
- [OpenSSF Scorecard checks](https://github.com/ossf/scorecard/blob/main/docs/checks.md)
