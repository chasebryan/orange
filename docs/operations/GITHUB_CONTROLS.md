# GitHub repository controls

Status: operating record for `chasebryan/orange`; Gate 0 bootstrap state

Snapshot date: 2026-07-11

Hosted-control snapshot: `snapshot_date=2026-07-11 review_due_date=2026-10-11 ruleset_id=18810248`

Hosted-execution refresh: 2026-07-12 at exact S1 `main` revision
`469bdec6037f20c8d099d61a09a3d19a55c88231`; this refresh did not re-read
repository settings

Required-check binding: `context="Required CI / docs-policy-workflows" integration_id=15368`

Required-check binding: `context="Dependency Review / policy" integration_id=15368`

This runbook records GitHub-hosted controls that are not fully represented by a
Git commit. It applies only to `https://github.com/chasebryan/orange`. A setting
can drift independently of the repository, so this file is evidence only when
paired with current API readback and the effective rules response.
The repository validator deliberately expires this snapshot on its review-due
date; a fresh readback and coordinated evidence update are then required.

## Current verified control plane

| Control | Verified state | Current limitation |
| --- | --- | --- |
| Visibility and default branch | Public; `main` | The working name remains blocked by D-017. |
| Private vulnerability reporting | Enabled | One bootstrap steward; no independent PSIRT continuity. |
| Dependabot alerts and security updates | Enabled | No product package manifest exists. |
| Secret scanning | Provider-pattern scanning and push protection enabled | Non-provider patterns and validity checks are plan-ineligible and read back disabled. |
| CodeQL default setup | 2026-07-11 readback: configured with `extended` query suite, standard runner, `remote_and_local` threat model, weekly schedule, and detected languages `actions` and `python` | A separate 2026-07-12 execution at exact S1 `main` also analyzed Rust. Run `29186965847` completed without analysis errors or warnings: Actions `1468413067` reported `0/23`, Python `1468413065` reported `0/50`, and Rust `1468413062` reported `7/27`. The seven Rust results remain open high `rust/path-injection` alerts #11-#17, all classified `test` in `compiler/crates/orangec/tests/cli.rs`. This execution is not a fresh settings readback, proof of vulnerability absence, independent coverage, or merge blocking. |
| Actions source policy | Exactly six repositories: `actions/checkout`, `actions/dependency-review-action`, `actions/upload-artifact`, `github/codeql-action`, `DavidAnson/markdownlint-cli2-action`, and `zizmorcore/zizmor-action`; plus one separately machine-enforced Scorecard image used by an explicit Docker CLI step | Repository wildcards are paired with full-SHA enforcement; reusable workflows still require repository validation. The Scorecard image and command are admitted only at the exact OCI digest. All other GitHub-owned and verified-publisher Actions are denied. |
| Execution immutability | Repository Actions require full 40-character commit SHAs; the explicit Scorecard Docker invocation requires its exact OCI digest | Content addressing does not establish publisher trust, mirror the selected bytes, or fix the registry, hosted runner, container host, network, and service inputs. |
| Default workflow token | `read`; cannot approve pull-request reviews | Job permissions remain authoritative and are checked in source. |
| External fork workflows | Approval required for all external contributors | Dependabot receives fork-like token restrictions and may require approval. |
| Merge methods | Squash only; auto-merge and branch update enabled; merged branches deleted | This does not itself require a pull request. The default-branch ruleset remains the enforcement gate. |
| Discussions | Disabled | Planning decisions remain in versioned OEPs, ADRs, issues, and pull requests. |
| Wiki | Disabled | Authoritative documentation remains versioned in Git. |
| Immutable releases | Enabled for future published releases | No Orange release is authorized; this setting does not make a planning snapshot a product release. |
| Default-branch rules | Active `Protect main` ruleset, ID `18810248`; no bypass actors; pull request, resolved conversations, squash-only linear history, deletion and non-fast-forward protection; exact required contexts `Required CI / docs-policy-workflows` and `Dependency Review / policy`, each bound to GitHub Actions integration ID `15368` | Bootstrap uses zero required approvals until a second qualified maintainer exists. CodeQL is not a required ruleset context, and no CodeQL threshold or negative blocking test has been established. |
| Commit signatures | Not required | Account signing key, verified commit, and Vigilant Mode require account-bound confirmation before enforcement. |
| Web commit sign-off | Disabled | D-018 has not selected DCO/CLA or contribution terms. |

## Source-controlled workflow boundary

The required CI and dependency-review workflows use `pull_request`, `push` to
`main`, and `merge_group` as applicable. They never use `pull_request_target`,
receive no configured repository or environment secret, start from empty
workflow permissions, use only a job-scoped `GITHUB_TOKEN` limited to
`contents: read`, declare timeouts, disable checkout credential persistence, and pin
every repository Action to a full commit SHA and admit the explicit Scorecard
Docker CLI runtime only at its exact digest, with reviewable version comments.
Required workflows have no manual dispatch because required-check identity does
not distinguish the triggering event; allowing one could let an operator attach
a same-named manual result to an unintended ref.
The required-check integration ID binds a context to the GitHub Actions app,
not to one workflow file. Workflow and event identity are not part of the
context, so a same-named job in another workflow could satisfy it. The exact
source-controlled workflow inventory and digests are defense in depth, while
the server-side approval gate for all external contributors and review of every
workflow change remain essential.

Scorecard runs its content-addressed image through an explicit Docker CLI step
only on trusted `main` push or schedule and hard-gates its job to the `main`
ref; it has no manual dispatch
because that could select an unreviewed branch while holding SARIF write
permission. Public Scorecard publication is deliberately disabled because the
publication service requires the official outer Action identity, whose selected
descriptor delegates to a mutable image tag. The digest-pinned run instead
retains its SARIF artifact and uploads it to GitHub code scanning without an
OIDC permission. External-link
and online workflow-metadata audits may be manually dispatched because they
hold only `contents: read`. All three are informational and must not be required
merge checks. Required CI runs zizmor with online audits disabled, so mutable
network metadata cannot decide whether a pull request is mergeable.
Scorecard alone is not SAST, a CodeQL result, an audit, or an OSPS conformance
claim. External-link availability is nondeterministic, so local relative-link
and anchor validation remains in required CI.

### Hosted execution observations

The original observation is bound to the 2026-07-11 readback and exact `main`
revision
`9f458c04542c512a8c04b00cb7ce4ef6bacd1a79`. Pull request #3 head
`8e26785f87c3866cc12915d7037820c608d6708d` was merged by `chasebryan` as that
exact `main` commit after its checks were green. The active ruleset still
requires the exact GitHub-Actions-bound Required CI and Dependency Review
contexts recorded above.

| Hosted observation | Exact evidence | Claim boundary |
| --- | --- | --- |
| Required CI on `main` | Run `29171653266` succeeded; repository policy `0.1.4` and its 65-test suite passed | One hosted execution under mutable runner and service inputs; not independent or reproducible proof |
| Workflow Online Audit on `main` | Run `29171653264` succeeded | Live upstream metadata is time-dependent; this does not prove that the scheduled trigger has executed |
| External Links on `main` | Run `29171653282` succeeded | Remote availability is time-dependent; this does not prove that the scheduled trigger has executed |
| OpenSSF Scorecard on `main` | Run `29171653261` succeeded | Posture observation only; Scorecard is not SAST, CodeQL, an audit, a merge gate, or an assurance claim, and scheduled-event execution remains unproven |
| CodeQL default setup on `main` | Run `29171652948` succeeded for `actions` and `python`; analyses `1467719309` and `1467719573` completed without errors or warnings | Zero results do not prove vulnerability absence; no CodeQL ruleset threshold or negative blocking behavior has been proven |

CodeQL alerts #1 through #3, all `py/path-injection`, read back as fixed rather
than dismissed at `2026-07-11T23:09:26Z`. That disposition records remediation
state in GitHub for this snapshot; it is not evidence that all injection paths
or future revisions are safe. The run IDs, analysis IDs, rule counts, statuses,
and alert dispositions above are hosted observations only. They do not capture
an immutable runner, service implementation, complete toolchain, signed result
bundle, or independently replayable environment.

On 2026-07-12, hosted execution was refreshed at the exact S1 merge revision
`469bdec6037f20c8d099d61a09a3d19a55c88231`. This was a run/result refresh,
not a readback of the settings table above. PR #5's final Required CI run
`29186933765` and Dependency Review run `29186933780` succeeded before the
squash merge. The resulting `main` push produced:

| Hosted observation | Exact S1 evidence | Claim boundary |
| --- | --- | --- |
| Required CI on `main` | Run `29186961911` succeeded; solo-bootstrap policy `0.2.0` and all 78 Python tests passed with the S1 Rust checks | One hosted execution under mutable runner and service inputs; not independent or reproducible proof |
| Workflow Online Audit on `main` | Run `29186961920` succeeded | Live upstream metadata is time-dependent; this push result does not prove the scheduled trigger |
| External Links on `main` | Run `29186961966` succeeded | Remote availability is time-dependent; this push result does not prove the scheduled trigger |
| OpenSSF Scorecard on `main` | Run `29186961902` succeeded | Posture observation only; not SAST, CodeQL, an audit, a merge gate, or an assurance claim |
| CodeQL default setup on `main` | Run `29186965847` succeeded without analysis errors or warnings; Actions `1468413067`=`0/23`, Python `1468413065`=`0/50`, Rust `1468413062`=`7/27` | Successful analysis does not mean alert-free analysis or prove a CodeQL merge threshold |

The seven Rust results are open high `rust/path-injection` alerts numbered 11
through 17, all classified `test` at paths in
`compiler/crates/orangec/tests/cli.rs`. S2 contains candidate remediation that
moves the affected tests off environment-derived temporary paths, but neither
closure nor non-exploitability is claimed until the S2 revision merges and a
post-merge CodeQL result and alert readback confirm disposition.

The scheduled link audit excludes only `https://eprint.iacr.org/` because that
primary-source host returns HTTP 403 to automated clients. The citations remain
in `docs/RESEARCH.md` and must be reviewed manually when research is refreshed;
the audit does not globally accept HTTP 403.

## Protected-main activation sequence

The following sequence was used for ruleset `18810248` and is mandatory for any
replacement; it avoids weakening the target while also avoiding a check-name
deadlock:

1. Push the complete branch and open a pull request.
2. Require successful hosted results from these exact jobs:
   `Required CI / docs-policy-workflows` and `Dependency Review / policy`.
3. Read the check runs for the exact head SHA. Record each check name and GitHub
   App integration ID; do not accept a same-named status from another producer.
   The activation readback bound both exact contexts to GitHub Actions
   integration ID `15368`.
4. Create one active default-branch ruleset with no bypass actor. Require pull
   request, strict current checks, conversation resolution, linear history, and
   block deletion and non-fast-forward updates.
5. During the sole-steward stage, require zero approvals and do not require
   CODEOWNER or last-push approval. GitHub does not permit an author to approve
   their own pull request; pretending otherwise would deadlock, not create
   independence.
6. Merge the bootstrap pull request only after all checks pass, then read back
   the effective rules for `main` and verify a safe direct update is rejected.
7. When a second trusted human maintainer joins, raise the rule to at least one
   non-author approval, require code-owner and last-push approval, dismiss stale
   reviews, and record the governance-stage transition.

The zero-approval bootstrap is a documented gap. It does not satisfy Orange's
mature two-person rule or OpenSSF OSPS-QA-07.01.

## Controls deliberately deferred

- **Signed commits:** configure an SSH or GPG signing key for `chasebryan`, a
  verified email, and Vigilant Mode; prove an owner-authored signed PR and
  squash merge before adding the required-signature rule.
- **Release tag rules:** D-017 must select the final namespace and D-018/D-019
  must close before a release pattern or authority is claimed. Immutable
  releases are already enabled, but no tag or release is authorized.
- **Code-scanning rule:** CodeQL default setup now produces successful `actions`,
  `python`, and `rust` analyses on S1 `main`, but it is not a required ruleset
  context. Rust analysis currently has seven open high test-classified
  `rust/path-injection` results.
  Keep a CodeQL threshold deferred until the exact producer/check identity is
  selected and both proposed and target revisions pass a safe activation test,
  including a controlled failing analysis that proves the intended rule blocks
  merge without a same-named-context bypass or check-name deadlock. Successful
  runs, zero-result languages, candidate fixes, and later fixed-alert states are
  not negative blocking proof.
- **License enforcement:** dependency review reports license data but no
  allow/deny list is authoritative before D-018.
- **Advanced secret features:** non-provider patterns and validity checks are
  not available on the current personal public-repository plan. This is an
  eligibility gap, not grounds to disable basic scanning.

## Account-bound prerequisites

Repository APIs cannot safely prove the owner's authentication factors or
recovery custody. The owner must confirm a passkey or security-key-backed 2FA,
offline recovery codes, a verified signing identity, and Vigilant Mode. Use
fine-grained, expiring credentials only when a GitHub App or job-scoped
`GITHUB_TOKEN` cannot perform the task. Never put offline release/root keys in
GitHub repository secrets.

These checks require explicit account access and are not marked complete by
this repository change.

## Drift review

Review on every workflow, collaborator, visibility, security-feature, branch
or tag rule, merge policy, secret, environment, deployment, webhook, signing,
or release change, and at least quarterly.

The review must read and preserve non-secret evidence for:

- repository visibility, ownership, merge methods, wiki, and immutable-release
  state;
- security-and-analysis features, private reporting, Dependabot, and CodeQL;
- Actions policy, selected identities, token defaults, and fork approval;
- effective default-branch and tag rules, bypass actors, required checks, and
  integration IDs;
- collaborators, environments, secrets metadata, webhooks, deployments, and
  release/tag inventory; and
- drift from this runbook, the threat model, and the OSPS evidence matrix.

API success alone is not end-to-end proof. Hosted runs, effective-rule
readback, negative tests, and human/account confirmations close their respective
controls.

## Primary references

- [GitHub Actions repository settings](https://docs.github.com/en/repositories/managing-your-repositorys-settings-and-features/enabling-features-for-your-repository/managing-github-actions-settings-for-a-repository)
- [GitHub Actions permissions API](https://docs.github.com/en/rest/actions/permissions?apiVersion=2026-03-10)
- [GitHub secure-use reference](https://docs.github.com/en/actions/reference/security/secure-use)
- [GitHub repository rules](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-rulesets/available-rules-for-rulesets)
- [GitHub CodeQL default setup](https://docs.github.com/en/code-security/how-tos/find-and-fix-code-vulnerabilities/configure-code-scanning/configure-code-scanning)
- [GitHub immutable releases](https://docs.github.com/en/code-security/concepts/supply-chain-security/immutable-releases)
