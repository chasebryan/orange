# Secrets lifecycle and incident response

Status: enforced solo-bootstrap repository policy; no product or release
secrets exist

Owner: Orange Project Owner; no separate PSIRT or Release Engineering role
exists in solo mode

This document governs credentials and sensitive incident material used to
operate `chasebryan/orange`. It complements `SECURITY.md`; it does not create a
staffed PSIRT, authorize a release, or prove account-level controls that the
repository cannot observe.

## Current and future secret classes

| Class | Current state | Approved custody | Prohibited custody |
| --- | --- | --- | --- |
| GitHub account authentication and recovery | Account-bound; repository API cannot verify factors | Phishing-resistant passkey/security key, protected password manager where needed, offline recovery copy | Repository files, issues, CI variables, logs, shared accounts |
| Local GitHub/API credential | Used only for authorized repository administration | OS credential store or fine-grained, expiring credential with minimum repository scope | Source, shell history, command output, issue/PR text, long-lived broad token |
| Workflow `GITHUB_TOKEN` | Ephemeral per job | GitHub-issued job token with repository default `read` and explicit minimum job permissions | Persisted checkout credential, artifact/cache, PR output, cross-job reuse |
| Vulnerability reports and incident evidence | Possible now | GitHub private advisory with access limited to triage principals; encrypted protected evidence store when needed | Public issue/PR/discussion, ordinary CI artifact, unencrypted shared location |
| Product service, registry, signing, transparency, and online update keys | None authorized | Future narrowly scoped credentials in protected stores with audit and rotation, only under an explicit release design; sole-owner control must remain disclosed | Introduction before an accepted design and owner authority |
| Offline release/root/recovery keys | None authorized | Future separated recovery material under an explicit release design; threshold custody requires a changed operating model and is unavailable now | GitHub secrets, a networked build runner, unrecorded one-person custody, ordinary backup |

No current workflow requires a stored repository secret. Adding any secret,
environment, OIDC audience, cloud trust, deploy key, webhook credential, or
self-hosted runner is a threat-model and policy trigger.

## Admission and inventory

Before issuing a credential, record non-secret metadata:

- stable credential ID and class;
- purpose, issuer, owner, backup/recovery owner, and approving authority;
- exact repository/resource scope, permissions, consumers, and trust boundary;
- approved storage class and whether export is possible;
- issuance, last-rotation, expiry, and mandatory review dates;
- revocation, replacement, and recovery procedure; and
- affected workflows, claims, release roles, and incident notification path.

Use a GitHub App or job-scoped `GITHUB_TOKEN` before a personal token. Use OIDC
before a stored cloud credential. Credentials are single-purpose, least-scope,
time-bounded, and never shared. Inventory metadata must not include the secret,
recovery code, private key, or a value from which it can be derived.

The project owner reviews current access and inventory at least quarterly and
on every collaborator, workflow, permission, secret, environment, webhook,
deployment, or recovery change. Independent review and separation of source,
build, sign/publish, registry, and root-recovery principals are unavailable in
solo mode. They remain disclosed evidence or conformance gaps and limit any
future claim that requires them; they are not prerequisites for unrelated
pre-alpha development.

## Rotation and revocation

Rotate at the shortest issuer-supported period appropriate to the class and
immediately on suspected exposure, owner/role change, device loss, unexpected
use, scope expansion, or upstream compromise. Expiry is not a substitute for
revocation. Remove the old consumer and verify rejection before closing the
rotation record.

Deleting a secret from the latest commit, log, or artifact does not contain an
incident. Treat the value as compromised, revoke it first, inspect every place
it could have propagated, replace it only after the path is corrected, and
preserve non-secret evidence of those actions.

## Incident severity and authority

Active exploitation, proof unsoundness, widespread silent miscompilation or
leakage, registry compromise, or release/root-key compromise enters immediate
incident mode under `SECURITY.md`. Any suspected credential disclosure begins
private triage even if a scanner did not alert.

The project owner may contain a current solo-bootstrap repository incident and,
if a future product exists, withdraw artifacts/profiles, quarantine packages,
coordinate disclosure, and control build/publish recovery. No separate PSIRT or
Release Engineering authority exists in solo mode. No incident responder may
use urgency to widen public claims, bypass an assurance stop-ship condition, or
publish with unverified replacement credentials.

## Common playbooks

### Secret in source, issue, log, cache, or artifact

1. Avoid copying the value. Privately record locator, time, reporter, and
   affected class.
2. Revoke or disable the credential through its issuer immediately.
3. Restrict the exposed issue/artifact/log where the platform permits without
   destroying required evidence.
4. Inspect Git history, forks, workflow logs, artifacts, caches, mirrors, local
   clones, and audit events for propagation and use.
5. Correct the injection path and narrow scope before issuing a replacement.
6. Verify the old value is rejected and the replacement works only in its
   intended boundary.
7. Notify affected principals and publish an advisory when impact crosses a
   public product or claim boundary.

History rewriting is exceptional: it cannot revoke a credential, can destroy
review evidence, and requires an incident record and downstream coordination.

### GitHub account or repository-administration compromise

Use a clean device to secure the account, revoke sessions/tokens/keys, rotate
recovery factors, inspect audit/security logs, collaborators, rulesets,
workflows, webhooks, deploy keys, environments, releases, tags, and security
settings, and compare Git refs with independently retained public history. Stop
merges and publication until authoritative state and ownership are recovered.

### Workflow, runner, Action, or OIDC compromise

Disable affected workflows or Actions, cancel runs, revoke related tokens and
cloud trust, preserve run IDs/log and artifact digests, determine which source
revision and event ran, inspect writes and attestations, and invalidate any
result that crossed the compromised boundary. A green check from a compromised
runner is not evidence. Re-enable only after pin/admission review and a clean
negative/positive exercise.

### Private-report confidentiality failure

Restrict access, preserve the original private chronology, notify the reporter,
identify every recipient/copy, rotate included credentials, adjust disclosure
timing to actual risk, and avoid amplifying exploit details. Conduct/privacy
concerns are handled separately unless they also create a vulnerability.

### Future release, registry, or root-key compromise

Stop ship and updates; freeze or quarantine affected channels; activate the
ratified TUF-style revocation/recovery roles; identify every signed artifact,
claim, package, and downstream; rebuild from independently verified source and
inputs; issue new identities and advisories; and rehearse rollback/freeze
recovery. No such key may exist while the corresponding authority and recovery
design remain unresolved.

## Evidence handling and communications

Record actions in UTC with actor, source, command or API endpoint, non-secret
result, and content digest where useful. Preserve original evidence read-only,
minimize personal data, separate observation from inference, and restrict
embargoed details to the smallest triage group. Never paste a credential into a
hashing command merely to create evidence; issuer-side identity and revocation
records are preferred.

Public communication states affected version/commit/artifact/claim tuples,
impact, remediation, invalidated prior claims, and verification steps without
publishing an active secret. Legal, privacy, contractual, and downstream notice
requirements need responsible review; this document is not legal advice.

## Exercises and exit evidence

At least quarterly during active solo-bootstrap automation, and before any
release-capable stage, exercise with synthetic credentials only:

- push-protection behavior using only a GitHub/provider-documented, non-live
  test string in a disposable ref when such a test mechanism is available;
- workflow token/permission and untrusted-event boundaries;
- disposable least-scope test-token revocation and access readback; real owner
  session/recovery-factor recovery remains tabletop and account-bound unless
  the owner explicitly approves a live drill;
- private evidence containment and notification; and
- future key, registry, rollback, freeze, and disaster recovery once designed.

An exercise records scenario, synthetic identifiers, participants, expected and
observed results, gaps, owners, deadlines, and retest. Never invent a
provider-shaped value or commit a live credential merely to test scanning; use
only an issuer-documented inert test mechanism.
