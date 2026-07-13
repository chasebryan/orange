# OpenSSF OSPS Baseline evidence matrix

Status: solo-bootstrap repository evidence; no conformance or maturity claim

Pinned baseline: [OpenSSF OSPS Baseline v2026.02.19](https://baseline.openssf.org/versions/2026-02-19.html)

Assessment snapshot: 2026-07-11

Hosted-control snapshot: `snapshot_date=2026-07-11 review_due_date=2026-10-11 ruleset_id=18810248`

Solo/pre-alpha assessment amendments and hosted-execution refreshes: 2026-07-12
through 2026-07-13. The hosted-control settings remain bound to the 2026-07-11
readback; the S2, S3a acceptance, and post-acceptance conformance/control
evidence below is separate run/result evidence, not a fresh settings observation.

Required-check binding: `context="Required CI / docs-policy-workflows" integration_id=15368`

Required-check binding: `context="Dependency Review / policy" integration_id=15368`

Owner: Orange Project Owner (`@chasebryan`)

Next scheduled review: 2026-10-11, or earlier on a trigger below

## Interpretation and scope

This matrix records directly observed operating evidence, merged repository
controls, documented intent, conditional scope, and remaining gaps. It is not an
OpenSSF certification, self-attestation, badge, or claim that Orange meets
Level 1, 2, or 3. A row is not satisfied merely because source merged, a single
workflow run was green, or policy says a future control will exist.

Version `2026.02.19` is pinned because the baseline directs downstream users to
assess a specific version, and its maturity levels have different scopes:

- Level 1 applies to any code or non-code project with any number of maintainers
  or users. It is the only level whose project scope currently matches Orange.
- Level 2 applies to a code project with at least two maintainers and consistent
  users. Orange now has a pre-alpha Rust compiler foundation, but it has only one
  maintainer and no demonstrated consistent user population.
- Level 3 applies to a code project with a large number of consistent users.
  Orange has neither. Level 3 remains the release-bearing target proposed by
  [`docs/ASSURANCE.md`](../ASSURANCE.md), not current state.

Authoritative S3a `main` contains the Orange lexer/CLI foundation, normative
Orange 2026 grammar, bounded parser, declaration-name and type analysis, a
noncanonical Typed Reference Core, deterministic literal evaluation, and the
pinned Rust toolchain dependency. It has no proof checker, canonical Core,
code generator, cryptography package, distribution channel, or product release.
Conditional release rows receive no compliance credit: `not triggered`
proves only that the triggering asset does not exist. The unresolved license and
absent non-author review remain explicit gaps, but D-023 makes them
claim/distribution limits rather than blockers to owner-authored development.

## Status vocabulary

| Status | Meaning |
| --- | --- |
| `Observed` | Direct repository or GitHub API evidence supports the current, triggered requirement at the snapshot. This is still not a level claim. |
| `Candidate` | Control source is proposed or present, but it is not authoritative operating evidence for the requirement or event under assessment. Candidate receives no observed credit. |
| `Documented` | Policy or design text exists, but operating or automated evidence is absent. |
| `Partial` | Some material control exists, but the requirement or its intended scope is not fully evidenced. |
| `Conditional` | The baseline's trigger is absent, such as no release or package manager. This is not a pass. |
| `Gap` | Required control is absent, disabled, unenforced, or contradicted by evidence. |
| `Unverified` | The state could not be independently observed with available repository-level evidence. |

## Evidence basis

### Snapshot boundary

The inspection used the public repository and authenticated GitHub REST
readback for `chasebryan/orange` on 2026-07-11. PR #1 merged as
`85b60b0f12cc566b199c54d87cc05c4879323e1f`, PR #2 merged as
`f6682072ec3149c4301dde25732d2ab4d790aa75`, and PR #3 head
`8e26785f87c3866cc12915d7037820c608d6708d` was merged by `chasebryan`
as `9f458c04542c512a8c04b00cb7ce4ef6bacd1a79` at
`2026-07-11T23:08:36Z`. The exact post-merge `main` revision reviewed is
`9f458c04542c512a8c04b00cb7ce4ef6bacd1a79`. Hosted settings and results are
operating evidence only for their exact snapshot, revision, language, and event.
GitHub settings can drift without a Git commit, so every API observation expires
at the next mandatory review.

A separate 2026-07-12 execution refresh is bound to exact S2 `main` revision
`52a3460853636f7cbaa27f3e27d86e032e3c82d4`. It records PR #6 and post-merge
run results only; it does not refresh any repository setting stated above.

A later 2026-07-12 execution refresh is bound to exact S3a `main` revision
`6c0bd3021cf2df603e08808e4660724ca1e2b2a5`. It records PR #9 and post-merge
run results only; it likewise does not refresh repository settings or establish
semantic proof, independent review, vulnerability absence, or OSPS conformance.

A 2026-07-13 post-acceptance conformance/control refresh is bound to exact
`main` revision `23352bcde976b86890db28ea4d375a31e6354bca`. It records PR #11,
post-merge runs, exact CodeQL result counts, and code-scanning alert state only.
It does not refresh settings, alter the accepted S3a revision, accept draft
D-003/D-004, authorize S3b, or establish semantic proof, independent review,
vulnerability absence, or OSPS conformance.

Evidence aliases used below:

| Evidence ID | Evidence |
| --- | --- |
| EV-GH-01 | Read-only GitHub repository API: public personal-account repository `chasebryan/orange`; default branch `main`; one collaborator, `chasebryan`, with admin; Issues enabled; Discussions disabled. |
| EV-GH-02 | GitHub security APIs: Dependabot alerts and security updates enabled; secret scanning and push protection enabled; non-provider pattern scanning and validity checks disabled; private vulnerability reporting enabled. |
| EV-GH-03 | GitHub Actions APIs: Actions enabled; sources restricted to the exact six repository Action identities used by merged workflows; broad GitHub-owned and verified-publisher allowances disabled; full-SHA pinning required; default workflow token permission `read`; workflows cannot approve pull-request reviews; all external fork runs require approval. The direct Scorecard OCI digest is enforced by merged repository validation, not the GitHub Action-source setting. |
| EV-GH-04 | GitHub rules APIs: active `Protect main` ruleset `18810248` targets the default branch with no bypass actor; requires a pull request, strict `Required CI / docs-policy-workflows` and `Dependency Review / policy` contexts from GitHub Actions integration `15368`, resolved conversations, squash-only linear history, and blocks deletion and non-fast-forward updates. Zero approvals are required during sole stewardship. Repository setting `web_commit_signoff_required` is false. |
| EV-GH-05 | GitHub code-scanning execution refresh: CodeQL run `29188111040` completed at exact S2 `main` revision `52a3460853636f7cbaa27f3e27d86e032e3c82d4` without analysis errors or warnings. GitHub Actions analysis `1468459678` reported `0/23`, Python analysis `1468459893` reported `0/50`, and Rust analysis `1468460793` reported `0/27`. Alerts #11-#17 all read back fixed at `2026-07-12T09:51:03Z`, with null dismissal time/reason, leaving no open CodeQL alerts. Open code-scanning alerts #4-#10 remain Scorecard posture findings. This refresh proves only the named execution, analyses, rule counts, revision, and alert state; it is not a fresh settings readback, all-code-scanning-clear claim, CodeQL threshold, merge block, independent analysis, or vulnerability-absence proof. Scorecard posture SARIF is not SAST or reproducible-build assurance. |
| EV-GH-06 | Repository APIs: squash-only merge, auto-merge and branch update enabled, merged branches deleted, wiki disabled, and immutable future releases enabled. No product release exists or is authorized. |
| EV-GH-07 | S3a code-scanning execution refresh: CodeQL run `29215789258` succeeded for PR #9 head `8c48a85997b756cf65d64110ebc869bb26e49079`, and post-merge dynamic CodeQL run `29215877437` succeeded for exact merged revision `6c0bd3021cf2df603e08808e4660724ca1e2b2a5`. These successes prove only that the configured analyses completed for their named revision and event; no result-count or alert-state readback was performed, and no vulnerability-absence, CodeQL threshold, independent-analysis, or fresh-settings claim is created. |
| EV-GH-08 | Post-acceptance code-scanning execution refresh: dynamic CodeQL run `29292740478` succeeded at exact revision `23352bcde976b86890db28ea4d375a31e6354bca` without analysis warnings. GitHub Actions analysis `1474500928` reported `0/23`, Python analysis `1474500920` reported `0/50`, and Rust analysis `1474500915` reported `0/27`. The open-alert readback contained Scorecard posture alerts #4-#10 and no open CodeQL alerts. This proves only the named execution, analyses, rule counts, revision, and alert state; it is not a fresh settings readback, all-code-scanning-clear claim, CodeQL threshold, merge block, independent analysis, or vulnerability-absence proof. |
| EV-REP-01 | Public Git URL and Git history identify source changes and authors. `git ls-files` plus file-type inspection found no generated executable. The steward-designated working brand images are the only tracked binary assets; each has an exact path, role, provenance statement, and SHA-256 admission in repository policy and [`assets/brand/manifest.json`](../../assets/brand/manifest.json). |
| EV-POL-01 | [`GOVERNANCE.md`](../../GOVERNANCE.md), [`CONTRIBUTING.md`](../../CONTRIBUTING.md), and [`.github/CODEOWNERS`](../../.github/CODEOWNERS) define current authority, branch workflow, ownership routing, and the solo-owner limitation. |
| EV-POL-02 | [`SECURITY.md`](../../SECURITY.md), [`SUPPORT.md`](../../SUPPORT.md), and the [issue configuration](../../.github/ISSUE_TEMPLATE/config.yml) define private reporting, response targets, public defect routing, and the lack of a staffed PSIRT. |
| EV-POL-03 | [`DEPENDENCY_POLICY.md`](../../DEPENDENCY_POLICY.md) and [`RELEASE_POLICY.md`](../../RELEASE_POLICY.md) define dependency admission, immutable references, release prohibition, future artifacts, separation, signing, and recovery. |
| EV-POL-04 | [`docs/ASSURANCE.md`](../ASSURANCE.md), [`docs/ARCHITECTURE.md`](../ARCHITECTURE.md), and [`THREAT_MODEL.md`](THREAT_MODEL.md) define proposed assurance, future actors/interfaces, stop-ship conditions, and current/future threats. |
| EV-POL-05 | [`docs/DECISIONS.md`](../DECISIONS.md#d-018--licenses) leaves licenses and inbound terms blocked; there is no `LICENSE`, `COPYING`, accepted DCO, or contributor agreement. |
| EV-OPS-01 | Merged workflows in [`.github/workflows/`](../../.github/workflows/) use full action SHAs, top-level empty permissions, explicit job permissions, timeouts, concurrency, non-persistent checkout credentials, and no `pull_request_target`. Exact S2 `main` revision `52a3460853636f7cbaa27f3e27d86e032e3c82d4` produced successful push runs Required CI `29188111313`, Workflow Online Audit `29188111278`, External Links `29188111303` attempt 2, OpenSSF Scorecard `29188111302`, and CodeQL `29188111040`. External Links attempt 1 encountered a transient `slsa.dev` connection failure; attempt 2 succeeded. These are exact push/trusted-main results; scheduled triggers remain unproven, and they do not refresh settings. |
| EV-OPS-02 | Merged [Dependabot configuration](../../.github/dependabot.yml), [dependency-review policy](../../.github/dependency-review-config.yml), repository validator, schemas, conformance fixtures, and parser boundary are authoritative on S2 `main`. Solo-bootstrap policy `0.2.1` has 88 passing Python tests, and post-merge Required CI `29188111313` passed at the exact S2 revision. PR #6 head `73416f1ee8b613f0f6244f8dcd2d30281e6e91f2` passed final Required CI `29188056038`, Dependency Review `29188056060`, and CodeQL `29188055399` before merge; the dependency result remains a required producer-bound PR check. Dependabot operation and rejection of a known-vulnerable, malicious, or otherwise untrusted dependency have not been demonstrated. Scorecard `29188111302` passed on S2 `main`, but it reports project posture rather than SAST or reproducible-build assurance; public Scorecard publication and OIDC remain disabled. |
| EV-OPS-03 | S3a is authoritative at exact `main` revision `6c0bd3021cf2df603e08808e4660724ca1e2b2a5`. PR #9 head `8c48a85997b756cf65d64110ebc869bb26e49079` passed Required CI `29215790064`, Dependency Review `29215790110`, and CodeQL `29215789258` before squash merge. The exact merged revision then passed Required CI `29215877872`, Workflow Online Audit `29215877891`, External Links `29215877874`, OpenSSF Scorecard `29215877875`, and dynamic CodeQL `29215877437`. Required CI covered policy `0.2.3`, 89 Rust tests including the documentation test, 95 Python tests, and zero foundation-validator findings. These are exact run observations, not a settings readback, semantic proof, independent review, release evidence, vulnerability-absence claim, or OSPS conformance claim. |
| EV-OPS-04 | Post-acceptance PR #11 head `7d54594349cc7afe0cacf60ebc9f1d8f5e913fee` passed Required CI `29292600483`, Dependency Review `29292600471`, and CodeQL `29292598799` before squash merge at `2026-07-13T23:20:22Z` as exact `main` revision `23352bcde976b86890db28ea4d375a31e6354bca`. The exact merged revision then passed Required CI `29292740885`, Workflow Online Audit `29292740874`, External Links `29292740884`, OpenSSF Scorecard `29292740941`, and dynamic CodeQL `29292740478`. Required CI covered policy `0.2.6`, 92 Rust tests including the documentation test, 103 Python tests, and zero foundation-validator findings. This revision adds the permanent external S3a black-box corpus and draft D-003/D-004 research protocols; it does not change S3a acceptance, accept either draft decision, or authorize S3b. These are exact run observations, not a settings readback, semantic proof, independent review, release evidence, vulnerability-absence claim, or OSPS conformance claim. |

### Current GitHub control-plane facts

| Setting | Observed value | Interpretation |
| --- | --- | --- |
| Visibility and ownership | Public personal-account repository; owner `chasebryan`; default `main` | Public source/history is available. Personal-account ownership has one ultimate owner and less granular collaborator roles than an organization. |
| Current access | Only `chasebryan`, admin | Exact current list, but also a bus-factor and independent-review gap. |
| Default-branch protection | Active ruleset `18810248`; no bypass actor; pull request, strict checks, resolved conversations, linear history, deletion and non-fast-forward protection | Platform API and effective-rule readback show enforcement. Zero approvals and one administrator remain an independence and privileged-control-plane risk; a safe direct-update negative test remains pending. |
| Vulnerability intake | Private vulnerability reporting enabled | Private repository-advisory intake exists; continuity depends on one steward. |
| Dependency security | Dependabot alerts and security updates enabled; Dependabot and dependency-review configuration merged | Configuration and a required PR dependency-review context exist. The compiler Cargo manifests contain only workspace packages. Dependabot operation and rejection of a known-vulnerable, malicious, or otherwise untrusted dependency remain unverified. |
| Secret protection | Secret scanning and push protection enabled | Supported provider patterns are covered. Non-provider patterns and validity checks are disabled, so coverage is not complete. |
| Actions policy | Enabled at the 2026-07-11 readback; exactly six repository Action identities selected; broad GitHub-owned and verified-publisher allowances disabled; full-SHA pinning required; all external fork runs need approval | Source identities and mutable references are restricted. EV-OPS-04 records the latest green PR #11 required checks and exact trusted-main push runs. This is an execution refresh, not a settings readback; scheduled trigger execution remains unproven. |
| Workflow tokens | Default `read`; cannot approve PR reviews | Merged workflows start with no permissions and grant explicit job permissions. Exact trusted-main push jobs completed successfully. Scorecard alone receives `security-events: write` for SARIF upload; public Scorecard publication and OIDC are disabled. Run success does not prove scheduled or manual trigger behavior or future permission stability. |
| CodeQL | Successful configured analyses for Python, GitHub Actions, and Rust; latest exact result-count and alert-state readback is bound to revision `23352bcde976b86890db28ea4d375a31e6354bca` | EV-GH-08 records `0/23`, `0/50`, and `0/27` results plus no open CodeQL alerts; open alerts #4-#10 are Scorecard posture findings. No CodeQL threshold/ruleset or failing-alert merge-block proof exists, and the execution refresh is not a fresh settings readback or vulnerability-absence claim. |
| Merge and release settings | Squash-only; auto-update/auto-merge enabled; merged branches deleted; immutable future releases enabled; active ruleset `18810248` | The ruleset protects `main`, but zero approvals do not provide independent review. No release is authorized. |
| Web sign-off | Disabled | No GitHub web-commit DCO/sign-off enforcement. Legal contribution terms remain unresolved. |

## Level 1 matrix

Level 1 is the applicable current project scope. The combination of gaps and
unverified controls below means Orange makes no Level 1 conformance claim.

| Control | Status | Exact current evidence | Gap and next evidence required |
| --- | --- | --- | --- |
| OSPS-AC-01.01 | `Unverified` | EV-GH-01 shows the only privileged principal, but repository APIs do not disclose that account's MFA state. EV-POL-01 requires phishing-resistant MFA where available. | A personal repository cannot impose an organization-wide collaborator MFA policy. Record verifiable platform enforcement or move to an organization that requires MFA; never publish recovery factors. |
| OSPS-AC-02.01 | `Observed` | EV-GH-01 shows only the owner. GitHub personal repositories add collaborators through an explicit invitation, satisfying the manual-assignment branch of the requirement. | Future collaborator admission needs a dated access record and review. Personal repositories give collaborators broad write access; an organization is needed for more granular roles. |
| OSPS-AC-03.01 | `Observed` with verification residual | EV-GH-04 records an active no-bypass `main` ruleset requiring a pull request and strict current checks. Effective-rule readback matched the configured rules. | Preserve the rule and perform a safe direct-update rejection test. Zero approvals provide no independent-review evidence. |
| OSPS-AC-03.02 | `Observed` with verification residual | EV-GH-04 records active deletion and non-fast-forward restrictions, confirmed by effective-rule API readback. | Preserve both rules and perform only a safe non-destructive negative test; do not test branch deletion against authoritative `main`. |
| OSPS-BR-01.01 | `Observed` with trigger-scope residual | EV-OPS-01 is merged and does not interpolate event metadata into shell commands; workflow expressions used for concurrency are not executed as shell source. EV-OPS-04 records successful PR #11 and trusted-main push executions at exact current revision `23352bcde976b86890db28ea4d375a31e6354bca`. | Scheduled and manual trigger execution remains unproven. Any future manual or event metadata reaching an interpreter must be allow-listed or safely passed through environment variables or arguments. |
| OSPS-BR-01.03 | `Observed` with event-scope residual | EV-OPS-01 is merged. PR workflows receive no configured repository or environment secrets, begin with no permissions, grant only job-scoped `contents: read`, avoid `pull_request_target`, and disable persisted checkout credentials. Required PR jobs previously succeeded; exact trusted-main push workflows are now green. Privileged Scorecard permissions occur only on trusted events. | Scheduled trigger execution remains unproven. Reassess on every secret, environment, self-hosted runner, event, or permission change. |
| OSPS-BR-03.01 | `Observed` | EV-REP-01 and repository link inspection show official project channels use HTTPS. Git and GitHub links identify `https://github.com/chasebryan/orange`. | Preserve HTTPS-only link validation. A future domain, registry, chat, package, or documentation channel must be inventoried before being called official. |
| OSPS-BR-03.02 | `Conditional` | No official distribution channel or software release exists; EV-POL-03 prohibits publication. | Before distribution, require authenticated HTTPS and signed, verifiable artifact/update metadata; demonstrate downgrade and adversary-in-the-middle resistance. |
| OSPS-BR-07.01 | `Partial` | EV-GH-02 enables secret scanning and push protection; merged ignore policy excludes common local credential files. | Non-provider patterns and validity checks are disabled. Define a secrets lifecycle, add complementary scanning where justified, and test block/revocation handling without committing a real secret. |
| OSPS-DO-01.01 | `Conditional` | No release or basic product functionality exists. EV-POL-03 forbids a product release. | Complete, tested user guidance for every released basic function is a pre-release requirement. Planning docs are not user guidance. |
| OSPS-DO-02.01 | `Conditional` | No release exists. EV-POL-02 already routes public planning defects and private vulnerabilities. | Before release, provide tested defect-report instructions for product versions, logs, reproductions, supported tuples, and sensitive-data handling. |
| OSPS-GV-02.01 | `Observed` | EV-GH-01 has public Issues enabled; structured issue forms accept planning defects, evidence, and OEP proposals. | Discussions are disabled, which is acceptable because the requirement needs one mechanism. Preserve public decision links and do not route vulnerabilities publicly. |
| OSPS-GV-03.01 | `Observed` | EV-POL-01 explains scope, workflow, evidence, review, definition of done, AI-assisted material, and the current third-party contribution prohibition. | Update immediately when licensing or contribution authority changes. |
| OSPS-LE-02.01 | `Gap` | Product source now exists, but EV-POL-05 records no selected OSI/FSF-compliant source license. | D-018 must select exact terms before third-party contributions or distribution. Solo owner-authored development does not close the licensing gap. |
| OSPS-LE-02.02 | `Conditional` and `Gap` | No released software asset exists, and EV-POL-05 records no selected release license. | Select compatible release-asset and generated-output terms before any release. Never infer them from a proposed recommendation. |
| OSPS-LE-03.01 | `Gap` | EV-POL-05: no `LICENSE`, `COPYING`, or license directory exists. | After legal selection, add the exact license text and machine-readable metadata in the same reviewed change. |
| OSPS-LE-03.02 | `Conditional` and `Gap` | No release exists and no release license is selected. | Include the ratified license alongside source and release assets, then verify packaged contents. |
| OSPS-QA-01.01 | `Observed` | EV-GH-01 and EV-REP-01: the source repository is publicly readable at a static GitHub URL. | Reassess on visibility or repository transfer. The owner's repo-only operating boundary must be preserved unless explicitly changed. |
| OSPS-QA-01.02 | `Observed` | EV-REP-01 records public Git history; EV-GH-04 blocks deletion and non-fast-forward updates to `main`. | Preserve archival continuity and monitor rule drift. Git authorship is not proof of legal authorization or cryptographic identity. |
| OSPS-QA-02.01 | `Partial` | The compiler workspace commits `Cargo.toml` and `Cargo.lock` and currently has no third-party crates. EV-OPS-02 also covers repository automation and GitHub Actions configuration. | Keep the direct/transitive graph exact for every admitted ecosystem; toolchain provenance, offline bytes, and future dependency admission remain additional Orange requirements. |
| OSPS-QA-04.01 | `Conditional` | Orange currently uses one repository, explicitly bounded to `chasebryan/orange` by EV-POL-01. | If a second codebase becomes part of Orange, add a canonical repository inventory before use; do not operate or publish elsewhere without owner direction. |
| OSPS-QA-05.01 | `Observed` | EV-REP-01 file-type inspection found no generated executable artifact. EV-POL-01 prohibits them, and EV-OPS-02 shows repository validation operating in required CI at the exact merged revision. | Preserve the validator in required CI and inspect Git LFS/releases when introduced; a source scan does not cover external assets. |
| OSPS-QA-05.02 | `Observed` with limitation | EV-REP-01 identifies eight non-executable working brand images. Required CI closes their paths and verifies exact SHA-256 admissions; the local README and manifest record roles and owner-supplied provenance. Two originals retain C2PA claims identifying OpenAI-generated media. | The C2PA signatures were not independently verified, binary review does not prove copyright or trademark rights, the D-017 working name lacks public-name clearance, and D-018 outbound terms remain open. Require the same closed admission for every future binary corpus or generated artifact. |
| OSPS-VM-02.01 | `Observed` with limitation | EV-POL-02 provides the private advisory contact path and identifies the project owner through governance. | Exercise solo continuity and recovery before public packages. One owner is discoverable but not resilient; independent PSIRT contacts are unavailable. |

## Level 2 target matrix

Level 2 does not currently apply because Orange is not a code project with two
maintainers. Existing evidence is recorded to expose rather than hide the work
remaining to become collaborative. Missing independent people or roles are
OSPS conformance gaps, not prerequisites for current pre-alpha development.

| Control | Status | Exact current evidence | Gap and next evidence required |
| --- | --- | --- | --- |
| OSPS-AC-04.01 | `Observed` with trigger-scope residual | EV-GH-03 sets default workflow permissions to `read`. EV-OPS-01 is merged, begins every workflow with `permissions: {}`, grants explicit job permissions, and has green exact trusted-main push runs. | Scheduled and manual trigger execution remains unproven. Continuously lint workflows and reverify the setting after repository transfer or GitHub policy change. |
| OSPS-BR-02.01 | `Conditional` | No official release exists. EV-POL-03 requires one immutable identifier spanning all relevant version axes. | Define and validate the release identifier format before the first candidate. |
| OSPS-BR-04.01 | `Conditional` | No release exists. EV-POL-03 requires changed claims, TCB/assumption deltas, security changes, and limitations. | Generate and review a functional/security changelog bound to each immutable release. |
| OSPS-BR-05.01 | `Conditional` | No build/release pipeline ingests product dependencies. Merged repository CI uses standard Actions plus checksum-verified downloads under EV-OPS-01. | Ratify standardized package/build tooling, immutable inputs, and offline archives for each admitted ecosystem. |
| OSPS-BR-06.01 | `Conditional` | No release or signing authority exists. EV-POL-03 requires signatures or signed manifests and exact asset digests. | Independent signing roles are unavailable in solo mode. Scope credentials and record identity, transparency/archival evidence, and a tested verification command before any publication; any OSPS requirement for independent roles remains unmet and is not claimed. |
| OSPS-DO-06.01 | `Conditional` with documented target | No release exists. EV-POL-03 documents selection, provenance, pinning, update, rollback, and removal requirements. | Replace target prose with the actual release dependency graph, acquisition procedure, and maintenance owners. |
| OSPS-DO-07.01 | `Partial` for pre-alpha compiler | On authoritative S3a `main`, `compiler/README.md` documents the dependency-free Rust lexer/parser/semantic/Core/evaluator/CLI workspace, and `make check` runs locked offline formatting, lint, documentation, unit, CLI, malformed, Unicode, line-ending, resource, and repeatability tests. No proof checker, canonical Core, code generator, or cryptography implementation exists. | Add clean-platform observations and update instructions with every merged capability; repository/compiler checks are not release assurance. |
| OSPS-GV-01.01 | `Observed` current scope; structurally weak | EV-GH-01 and this snapshot list the sole sensitive-resource holder, `chasebryan`, with admin access. EV-POL-01 requires a maintainer record when another principal exists. | Inventory all repository, security-alert, CI, domain, registry, key, and release access before those resources exist; publish role-safe details without secrets. |
| OSPS-GV-01.02 | `Observed` current solo role | EV-POL-01 and D-019/D-023 define the owner as the sole technical, assurance, release-policy, and security-response authority. | Preserve the single-owner limitation and never claim collaborative governance. Record an explicit transition only if real participants later exist. |
| OSPS-GV-03.02 | `Documented` but legally blocked | EV-POL-01 defines acceptable scope, workflow, evidence, review, and contribution quality. It rejects third-party merge until D-018 closes. | Ratify licenses and DCO/CLA terms, then test the guide with an eligible external contribution. |
| OSPS-LE-01.01 | `Gap` | EV-GH-04 has web sign-off disabled; EV-POL-05 has no accepted DCO or contributor agreement. Third-party merges are prohibited rather than falsely treated as authorized. | Select legal provenance terms, require assertion on every code commit, and add enforcement/validation with counsel-approved wording. |
| OSPS-QA-03.01 | `Observed` with negative-test residual | EV-GH-04 binds exact successful `Required CI / docs-policy-workflows` and `Dependency Review / policy` contexts to GitHub Actions integration `15368` under a strict no-bypass ruleset. | Preserve producer binding and prove a qualifying failed check blocks merge without weakening or bypassing the ruleset. |
| OSPS-QA-06.01 | `Observed` for repository-policy scope | EV-OPS-01/02/03 retain historical policy evidence; EV-OPS-04 records policy `0.2.6`, 103 passing Python tests, zero foundation-validator findings, PR #11 Required CI `29292600483`, and post-merge Required CI `29292740885` at exact revision `23352bcde976b86890db28ea4d375a31e6354bca`. Negative local mutations demonstrate fail-closed validator behavior. | Retain the required check and expand the suite with every product component. This is repository-policy evidence, not product implementation assurance. |
| OSPS-SA-01.01 | `Conditional` with design evidence | No release exists. EV-POL-04 documents proposed actors, components, and flows, including this stable-ID threat model. | Update from intended architecture to the exact deployed system and all human/service actors before release. |
| OSPS-SA-02.01 | `Conditional` | No released external software interface exists. The current `orangec` CLI is documented as pre-alpha; later CLI/LSP/ABI/registry surfaces remain planned rather than released specifications. | Inventory and document every actual external interface, protocol, parser, error contract, privilege, and version before release. |
| OSPS-SA-03.01 | `Conditional` with early assessment | No release exists. EV-POL-04 covers repository controls and the merged pre-alpha lexer/parser/semantic/Core/evaluator/CLI boundary, not a full product or deployment assessment. | Repeat against executable code, dependencies, deployments, findings, and test results before each release-bearing capability. |
| OSPS-VM-01.01 | `Observed` policy | EV-POL-02 defines coordinated disclosure, safe handling, one-business-day acknowledgement and three-business-day assessment targets. | The owner must exercise the process; targets are not an SLA and one-person continuity remains a disclosed gap. External staffing is unavailable, not a current development prerequisite. |
| OSPS-VM-03.01 | `Observed` | EV-GH-02 verifies private vulnerability reporting; EV-POL-02 links directly to it and prohibits public reports. | Exercise owner notifications and continuity without submitting a fake vulnerability. An independent private contact is unavailable, remains an OSPS continuity gap, and is not claimed. |
| OSPS-VM-04.01 | `Conditional` | No Orange software vulnerability or release advisory exists. EV-POL-02 promises published advisories identifying affected versions and invalidated claims. | Define the advisory/Vulnerability Disclosure Report publication record and exercise it during an incident simulation. |

## Level 3 target matrix

Level 3 is a future release-bearing target only. None of these rows should be
used to imply present maturity or a large user base.

| Control | Status | Exact current evidence | Gap and next evidence required |
| --- | --- | --- | --- |
| OSPS-AC-04.02 | `Observed` with independence and trigger residuals | EV-OPS-01 grants read-only contents to validation jobs; only trusted-event Scorecard has `security-events: write` for SARIF upload. EV-OPS-04 records Scorecard run `29292740941` passing at exact revision `23352bcde976b86890db28ea4d375a31e6354bca`. Public Scorecard publication and OIDC are disabled. | Scheduled execution remains unproven. One owner and zero required approvals do not independently protect workflow permission changes. Document why the remaining write is necessary and re-review every permission delta. |
| OSPS-BR-01.04 | `Partial` | Merged manual workflow definitions accept no user-defined inputs, grant only `contents: read`, and do not interpolate collaborator input into shell. Their jobs have green push-path evidence. The privileged Scorecard job has no manual trigger and is hard-gated to `main`. | `workflow_dispatch` execution itself remains unproven. Any future dispatch input or write-capable manual job must have an allow-list, length/type constraints, trusted-ref gate, safe interpreter boundary, and negative tests. |
| OSPS-BR-02.02 | `Conditional` | No release asset exists. EV-POL-03 requires each asset and claim to bind to immutable identities/digests. | Generate an asset manifest and prove every archive, SBOM, evidence bundle, signature, and binary maps to the release ID. |
| OSPS-BR-07.02 | `Documented` and `Partial` | [`SECRETS_AND_INCIDENTS.md`](SECRETS_AND_INCIDENTS.md) defines current/future classes, owners, stores, issuance, rotation, expiry, audit, revocation, recovery, and synthetic exercises. | Verify account controls, run owner-executable exercises, and instantiate inventories for every future credential. Independent response/release roles are unavailable and remain a disclosed conformance gap. |
| OSPS-DO-03.01 | `Conditional` | No release. EV-POL-03 plans hashes, signatures, provenance, and update metadata. | Publish tested offline and online integrity/authenticity verification instructions for exact assets. |
| OSPS-DO-03.02 | `Conditional` | No release author or signing identity exists. | Document trusted release identities, keyless/threshold verification, rotation, compromise, and historical verification without exposing private keys. |
| OSPS-DO-04.01 | `Conditional` | No release. [`SUPPORT.md`](../../SUPPORT.md) explicitly says none is supported and labels future duration as proposed. | Publish funded scope and exact start/end dates for every released support tuple. |
| OSPS-DO-05.01 | `Conditional` | No version receives updates today. Support and withdrawal behavior is only proposed. | Publish EOL and security-update cessation statements and downstream notices for each release. |
| OSPS-GV-04.01 | `Documented` and `Partial` | EV-POL-01 requires least privilege, role criteria, conflict handling, prompt offboarding, and quarterly access review. | Ratify objective privilege-escalation criteria, record reviewer/decision/effective date, and technically restrict roles. Solo owner cannot independently review own escalation. |
| OSPS-QA-02.02 | `Conditional` | No compiled release exists. EV-POL-03 requires SPDX SBOM and CycloneDX SBOM/CBOM. | Generate, validate, sign/bind, and ship complete component inventories for actual release assets. |
| OSPS-QA-04.02 | `Conditional` | One repository and no release. | If a release spans repositories, inventory them and prove each enforces equivalent or stronger source, review, CI, dependency, and release controls. |
| OSPS-QA-06.02 | `Partial` | EV-POL-04 documents PR, merge, nightly, weekly, and release-candidate test classes. EV-OPS-04 records merged repository-check commands, 92 Rust tests including the documentation test, 103 policy tests, successful PR #11 checks, and exact green trusted-main push results. The current compiler suite covers formatting, lint, documentation, unit, CLI, malformed-input, parser, semantic, Core, evaluator, Unicode, line-ending, resource, repeatability, and the permanent external S3a black-box corpus. | Scheduled cadence is not established by repository configuration alone. Bind each accepted compiler change to its exact required-CI result and publish exact environments, expected outputs, retention, and failure evidence as each future test class becomes operating. |
| OSPS-QA-06.03 | `Documented` | EV-POL-01 requires tests or replayable evidence where applicable and coupled updates for major effects. | Ratify explicit major-change test obligations and enforce them through review, coverage, and conformance policy as code evolves. |
| OSPS-QA-07.01 | `Gap` | EV-GH-01/04 and EV-POL-01: one owner, no non-author collaborator, no required approval. CODEOWNERS names only the author/owner. | A qualified independent maintainer and non-author approval are unavailable in solo mode, so this OSPS control remains unmet. If the operating model changes, require the approval, prevent self-approval/bypass, and verify a negative merge test; this gap is not a pre-alpha development prerequisite and cannot be solved by a bot or the author's second account. |
| OSPS-SA-03.02 | `Conditional` with pre-alpha model | [`THREAT_MODEL.md`](THREAT_MODEL.md) covers the current repository and compiler foundation plus future boundaries with stable IDs, abuse paths, controls, residual risk, owners, and triggers. No released critical code path exists. | Re-perform threat and attack-surface analysis against the exact code and deployment before release and on every mandatory trigger. |
| OSPS-VM-04.02 | `Conditional` | No product component vulnerability or non-exploitability decision exists; no VEX process has run. | Define CycloneDX or CSAF VEX generation, review, expiry, evidence, and correction procedures before suppressing any component finding. |
| OSPS-VM-05.01 | `Gap` | Merged dependency review is configured to fail at `moderate` vulnerability severity, but EV-POL-03 has no complete SCA vulnerability/license remediation deadlines or risk thresholds. | Ratify severity, exploitability, malicious-package, license, SLA, exception, VEX, and expiry thresholds for every dependency class. |
| OSPS-VM-05.02 | `Documented` and `Partial` | EV-POL-03 says security policy failures fail closed and cannot waive assurance gates. EV-OPS-02 is merged; Dependency Review previously succeeded and EV-GH-04 makes its producer-bound context required. | Exercise violation and exception paths, demonstrate rejection of an untrusted dependency, and block every release on unresolved policy violations. No release-block proof exists yet. |
| OSPS-VM-05.03 | `Partial` | EV-OPS-02 supplies merged required PR dependency review plus Dependabot configuration and vulnerability/license checks. EV-OPS-04 records green PR #11 Dependency Review `29292600471` and trusted-main Scorecard `29292740941`. | Dependabot operation and rejection of a known-vulnerable, malicious, or otherwise untrusted dependency remain unverified. Add explicit malicious-dependency policy and ecosystem coverage. Scorecard posture is not SAST or reproducible-build assurance. |
| OSPS-VM-06.01 | `Gap` | No SAST remediation threshold or exception/expiry policy exists. | Define severity, confidence, scope, remediation time, false-positive evidence, owner, expiry, and release-blocking behavior before SAST is authoritative. |
| OSPS-VM-06.02 | `Partial` | EV-GH-08 records successful dynamic CodeQL run `29292740478` at exact revision `23352bcde976b86890db28ea4d375a31e6354bca`: Actions `0/23`, Python `0/50`, and Rust `0/27`, with no analysis warnings or open CodeQL alerts. Open alerts #4-#10 are Scorecard posture findings. | Evidence covers only the named analyses, languages, rules, revision, and event. Define an authoritative threshold and exception/expiry policy, bind an appropriate CodeQL result to the ruleset, and demonstrate that a qualifying alert blocks merge. Zero configured-query results and no open CodeQL alerts are not vulnerability-absence proof. Scorecard is posture data, not source-code weakness analysis or reproducible-build assurance. |

## Priority gaps and gates

The following must remain visible despite merged source and green named `main` runs:

1. **Protected-main verification residual:** active no-bypass ruleset `18810248`
   requires a PR, strict producer-bound checks, resolved conversations, linear
   history, and blocks deletion and non-fast-forward updates. Safe negative
   tests still need to prove direct-update and failed-check rejection without
   weakening authoritative `main` (OSPS-AC-03.01/02, OSPS-QA-03.01).
2. **Independent-review gap:** one owner cannot provide non-author approval. A
   bot, CODEOWNERS request, self-review, or second account controlled by the
   author does not close it. D-023 records this as unavailable evidence, not an
   active dependency.
3. **Licensing gap:** D-018 has no outbound terms, license file, or inbound
   assertion. Third-party merges and distribution remain prohibited, while
   owner-authored product code is permitted
   (OSPS-LE-01.01, OSPS-LE-02.01/02, OSPS-LE-03.01/02).
4. **MFA evidence gap:** the repository review could not independently verify
   account MFA or impose an organization requirement (OSPS-AC-01.01).
5. **Code-scanning enforcement gap:** EV-GH-08 records exact zero-result Actions,
   Python, and Rust analyses at revision
   `23352bcde976b86890db28ea4d375a31e6354bca`, no analysis warnings, and no
   open CodeQL alerts; open alerts #4-#10 are Scorecard posture findings. No SAST
   remediation threshold, CodeQL-required ruleset context, exception/expiry
   policy, or failing-alert merge-block proof exists (OSPS-VM-06.01/02). The
   evidence does not extend beyond the named languages, rules, revision, and
   event, and it does not prove vulnerability absence.
   Scorecard is project posture, not SAST or reproducible-build assurance.
6. **Secrets and supply-chain gaps:** generic/non-provider and validity secret
   checks are disabled; the documented lifecycle is not independently exercised;
   selected Actions still depend on reviewed upstream identities; no complete
   SCA, VEX, or malicious-dependency lifecycle exists.
7. **Operational-evidence residual:** S3a acceptance remains bound to exact
   revision `6c0bd3021cf2df603e08808e4660724ca1e2b2a5`. Post-acceptance PR #11
   head `7d54594349cc7afe0cacf60ebc9f1d8f5e913fee` passed Required CI
   `29292600483`, Dependency Review `29292600471`, and CodeQL `29292598799`.
   At exact current `main` revision
   `23352bcde976b86890db28ea4d375a31e6354bca`, trusted-main push runs then
   succeeded for Required CI `29292740885`, Workflow Online Audit `29292740874`,
   External Links `29292740884`, OpenSSF Scorecard `29292740941`, and dynamic
   CodeQL `29292740478`. This refresh did not re-read settings. The exact CodeQL
   result counts and alert state are separately bounded in EV-GH-08.
   Scheduled triggers remain unproven. Dependabot operation and rejection of a
   known-vulnerable, malicious, or otherwise untrusted dependency remain
   unverified. Green Scorecard execution does not establish SAST or reproducible
   assurance; public Scorecard publication and OIDC remain disabled. Run success
   alone does not prove failure-path enforcement or retained artifact contents.
8. **Release-capability gap:** no release decision, keys, artifacts, SBOM/CBOM,
   provenance, registry, support capacity, or recovery drill exists. Independent
   PSIRT, builders, and release roles are unavailable and not claimed. The
   current release prohibition is correct.

## Next-review protocol

### Mandatory triggers

Review the complete matrix, not only one row, when any of these occurs:

- a workflow, Action, runner, token permission, environment, secret, webhook,
  GitHub App, deployment, or external integration changes;
- repository ownership, visibility, collaborator access, MFA requirement,
  branch/tag rule, merge policy, security feature, or recovery setting changes;
- a merged workflow first executes through a scheduled or manual trigger,
  Dependabot first operates, or a required check/ruleset is changed;
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

The next scheduled full assessment is 2026-10-11. The first scheduled workflow
results, first demonstrated Dependabot operation, first safe direct-update or
failed-required-check negative test, any CodeQL threshold/ruleset change, and
any protected-main configuration change are earlier mandatory triggers.

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
