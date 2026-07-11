# Gate 0 CI dependency inventory

Status: current direct-dependency inventory and gap record; not a product
dependency admission, reproducible-build claim, or legal approval

Snapshot: 2026-07-11

## 1. Scope and claim boundary

This document records the third-party Actions, downloaded executables, hosted
services, and ambient runner tools used by the five workflows in
`.github/workflows/`. It also separates repeatable validation methods from
reproducible evidence. The inventory is evidence for reviewing the current
repository automation; it does not satisfy the full admission record required
by [`DEPENDENCY_POLICY.md`](../../DEPENDENCY_POLICY.md).

A full Git commit SHA fixes the selected upstream Action revision under
GitHub's object model. A container digest fixes the selected OCI manifest. Each
identity still depends on its registry or service for retrieval and does not,
by itself, fix the hosted runner, service APIs, network responses, provenance,
or transitive dependency closure. A release-archive checksum fixes the bytes
accepted by an installer, but does not archive those bytes or establish their
publisher identity. Those distinctions are explicit below.

No component in this file is an Orange product dependency or part of a future
logical trusted computing base. D-018 remains open, so recording an upstream
license is factual provenance only and does not authorize incorporation,
redistribution, or a license for this repository.

## 2. Workflow map

| Workflow | Trigger and role | Direct external execution dependencies | Network or hosted-state boundary |
| --- | --- | --- | --- |
| `ci.yml` | Required pull-request, merge-queue, and `main` repository checks | Checkout, markdownlint, actionlint, and zizmor | GitHub resolves Actions; actionlint is downloaded; the digest-pinned zizmor image is pulled from GHCR |
| `dependency-review.yml` | Pull-request and merge-queue dependency-policy signal | Checkout and Dependency Review | Depends on GitHub's dependency graph, API, and event comparison state |
| `external-links.yml` | `main`, scheduled, and manual link observation | Checkout and lychee | Downloads lychee and queries every non-excluded external endpoint at run time |
| `scorecard.yml` | `main` and scheduled OpenSSF posture observation | Checkout, Scorecard, artifact upload, and CodeQL SARIF upload | Uses GitHub, GHCR, artifact, and code-scanning services; public Scorecard publication and OIDC are disabled |
| `workflow-online-audit.yml` | `main`, scheduled, and manual upstream-metadata observation | Checkout and zizmor | Pulls the digest-pinned zizmor image and intentionally queries current GitHub metadata |

`ci.yml` and `dependency-review.yml` supply the two required merge checks bound
to GitHub Actions by ruleset `18810248`; effective rules still require separate
readback during drift review. The other three
workflows are informational. Dependency, link, workflow-metadata, and
repository-posture results can change while the checked-out repository bytes
remain fixed.

## 3. Pinned Action inventory

Every direct repository Action `uses:` reference currently names a full
40-character commit SHA. Scorecard instead runs through an explicit Docker CLI
invocation of the separately admitted OCI image recorded in section 4, selected
directly by digest. Version comments are review aids, not the enforced
identities.

| Component and use | Enforced Action revision | Upstream license and provenance | Runtime and unresolved closure |
| --- | --- | --- | --- |
| [`actions/checkout`](https://github.com/actions/checkout/tree/9c091bb21b7c1c1d1991bb908d89e4e9dddfe3e0), used by all workflows | `9c091bb21b7c1c1d1991bb908d89e4e9dddfe3e0` (`v7.0.0`) | [MIT at the selected revision](https://github.com/actions/checkout/blob/9c091bb21b7c1c1d1991bb908d89e4e9dddfe3e0/LICENSE); upstream Git repository is the provenance locator | Bundled JavaScript runs on GitHub-provided Node 24 and invokes ambient Git; neither runtime is fixed here |
| [`DavidAnson/markdownlint-cli2-action`](https://github.com/DavidAnson/markdownlint-cli2-action/tree/8de2aa07cae85fd17c0b35642db70cf5495f1d25), used by required CI | `8de2aa07cae85fd17c0b35642db70cf5495f1d25` (`v24.0.0`) | [MIT at the selected revision](https://github.com/DavidAnson/markdownlint-cli2-action/blob/8de2aa07cae85fd17c0b35642db70cf5495f1d25/LICENSE); its [exact package manifest](https://github.com/DavidAnson/markdownlint-cli2-action/blob/8de2aa07cae85fd17c0b35642db70cf5495f1d25/package.json) names `@actions/core` 3.0.1 and `markdownlint-cli2` 0.23.0 | Bundled JavaScript runs on GitHub-provided Node 24; the repository does not independently hash, archive, or inventory the bundled transitive graph |
| [`zizmorcore/zizmor-action`](https://github.com/zizmorcore/zizmor-action/tree/192e21d79ab29983730a13d1382995c2307fbcaa), used by required CI and the online audit | `192e21d79ab29983730a13d1382995c2307fbcaa` (`v0.5.7`) | [MIT at the selected revision](https://github.com/zizmorcore/zizmor-action/blob/192e21d79ab29983730a13d1382995c2307fbcaa/LICENSE); the selected revision's [version map](https://github.com/zizmorcore/zizmor-action/blob/192e21d79ab29983730a13d1382995c2307fbcaa/support/versions) supplies the runtime image digest | Composite Bash Action requiring ambient Docker; Orange selects zizmor 1.26.1, whose image digest is recorded in section 4 |
| [`actions/dependency-review-action`](https://github.com/actions/dependency-review-action/tree/a1d282b36b6f3519aa1f3fc636f609c47dddb294), used by dependency review | `a1d282b36b6f3519aa1f3fc636f609c47dddb294` (`v5.0.0`) | [MIT at the selected revision](https://github.com/actions/dependency-review-action/blob/a1d282b36b6f3519aa1f3fc636f609c47dddb294/LICENSE); upstream Git repository is the provenance locator | Bundled JavaScript runs on GitHub-provided Node 24 and consumes current GitHub dependency data; neither the bundle closure nor API response is archived here |
| [`actions/upload-artifact`](https://github.com/actions/upload-artifact/tree/043fb46d1a93c77aae656e7c1c64a875d1fc6a0a), used by Scorecard | `043fb46d1a93c77aae656e7c1c64a875d1fc6a0a` (`v7.0.1`) | [MIT at the selected revision](https://github.com/actions/upload-artifact/blob/043fb46d1a93c77aae656e7c1c64a875d1fc6a0a/LICENSE); upstream Git repository is the provenance locator | Bundled JavaScript runs on GitHub-provided Node 24 and writes to the mutable hosted artifact service; service implementation and storage are not reproducible inputs |
| [`github/codeql-action/upload-sarif`](https://github.com/github/codeql-action/tree/99df26d4f13ea111d4ec1a7dddef6063f76b97e9), used by Scorecard | `99df26d4f13ea111d4ec1a7dddef6063f76b97e9` (`v4.37.0`) | [MIT at the selected revision](https://github.com/github/codeql-action/blob/99df26d4f13ea111d4ec1a7dddef6063f76b97e9/LICENSE); upstream Git repository is the provenance locator | Bundled JavaScript runs on GitHub-provided Node 24 and writes to the hosted code-scanning service; neither service behavior nor the transitive bundle is fixed here |

The pinned zizmor composite Action also declares
`github/codeql-action/upload-sarif` at revision
`8aad20d150bbac5944a9f9d289da16a4b0d87c1e` (`v4.36.2`). Both Orange usages set
`advanced-security: false`, so that conditional step is not executed. It remains
part of the upstream descriptor and therefore part of the source-review surface.

## 4. Downloaded executable and container inventory

| Tool | Selected artifact identity | Verification performed by current automation | Upstream license and provenance | Remaining limitation |
| --- | --- | --- | --- | --- |
| actionlint 1.7.12 | Linux AMD64 `sha256:8aca8db96f1b94770f1b0d72b6dddcb1ebb8123cb3712530b08cc387b349a3d8`; Linux ARM64 `sha256:325e971b6ba9bfa504672e29be93c24981eeb1c07576d730e9f7c8805afff0c6` | [`scripts/ci/install-actionlint`](../../scripts/ci/install-actionlint) downloads the named release archive over HTTPS and checks the architecture-specific SHA-256 before extraction | [MIT at release `v1.7.12`](https://github.com/rhysd/actionlint/blob/v1.7.12/LICENSE.txt); [upstream release](https://github.com/rhysd/actionlint/releases/tag/v1.7.12) is the provenance locator | Archive bytes and upstream release metadata are not mirrored or signed into this repository; installer trust still includes DNS, TLS, GitHub availability, and ambient `curl`, `sha256sum`, `tar`, and `install` |
| lychee 0.24.2 | Linux x86-64 GNU archive `sha256:1f4e0ef7f6554a6ed33dd7ac144fb2e1bbed98598e7af973042fc5cd43951c9a` | [`scripts/ci/install-lychee`](../../scripts/ci/install-lychee) downloads the named release archive over HTTPS and checks SHA-256 before extraction | Dual [Apache-2.0](https://github.com/lycheeverse/lychee/blob/lychee-v0.24.2/LICENSE-APACHE) or [MIT](https://github.com/lycheeverse/lychee/blob/lychee-v0.24.2/LICENSE-MIT) at release `lychee-v0.24.2`; [upstream release](https://github.com/lycheeverse/lychee/releases/tag/lychee-v0.24.2) is the provenance locator | Only Linux x86-64 is admitted by the installer; archive and release metadata are not mirrored, and link results depend on live remote endpoints |
| zizmor 1.26.1 container | `ghcr.io/zizmorcore/zizmor:1.26.1@sha256:d1117e5dbd9ee4970644067b534ab6ab50371f3c6f7f4d05446eb603a6e78f48` | The exact pinned composite Action rejects an unknown version and constructs the image reference from its committed version-to-digest map | [MIT at release `v1.26.1`](https://github.com/zizmorcore/zizmor/blob/v1.26.1/LICENSE); [upstream release](https://github.com/zizmorcore/zizmor/releases/tag/v1.26.1) and GHCR are the provenance locators | The image is content-selected but not mirrored; execution still depends on the registry, ambient Docker daemon/kernel, runner CPU, and network. Online-audit output additionally depends on current GitHub state |
| Scorecard 2.4.3 container | `ghcr.io/ossf/scorecard-action@sha256:2dd6a6d60100f78ef24e14a47941d0087a524b4d3642041558239b1c6097c941` | [`scorecard.yml`](../../.github/workflows/scorecard.yml) invokes `docker run` with the exact digest; repository validation admits that image and enforces the command, read-only event mount, workspace mount, environment forwarding, dropped capabilities, and no-new-privileges | [Apache-2.0](https://github.com/ossf/scorecard-action/blob/4eaacf0543bb3f2c246792bd56e8cdeffafb205a/LICENSE); the image's upstream OCI revision label is [`4eaacf0543bb3f2c246792bd56e8cdeffafb205a`](https://github.com/ossf/scorecard-action/tree/4eaacf0543bb3f2c246792bd56e8cdeffafb205a), and GHCR is the runtime provenance locator | The runtime is content-selected but not mirrored; execution still depends on GHCR availability and integrity, correct publisher provenance and digest admission, and the ambient Docker daemon, kernel, runner CPU, and network. The container root filesystem is read-only, while the workspace remains writable for `results.sarif`. The hosted result is not hermetic or independently reproducible evidence |

Orange deliberately sets `INPUT_PUBLISH_RESULTS` to `false`. OpenSSF's public
publication API requires the official `ossf/scorecard-action` step identity,
while that exact Action descriptor selects its runtime through a mutable image
tag. Gate 0 chooses the content-addressed runtime and retains the SARIF artifact
and GitHub code-scanning upload instead. The job therefore requests no OIDC
token and makes no Orange-authenticated publication to `api.scorecard.dev`.

Checksums above are copied from the executable installer scripts or the exact
selected Action version map. A reviewer must compare this table with those
authoritative repository bytes after every change; this prose is not an
enforcement mechanism.

## 5. First-party methods and ambient dependencies

The required invariant check invokes repository-owned Bash and Python files:

- [`scripts/ci/check-repository`](../../scripts/ci/check-repository) sets
  locale, timezone, hash seed, UTF-8, bytecode, and source-date variables, then
  runs the standard-library `unittest` suite and
  [`tools/validate_foundation.py`](../../tools/validate_foundation.py);
- the actionlint and lychee installers enforce the archive identities recorded
  above; and
- [`scripts/ci/check-external-links`](../../scripts/ci/check-external-links)
  defines the live link-check method and its documented IACR exclusion.

These are first-party repository methods, not third-party dependencies. The
repository has no selected license while D-018 is blocked. This inventory does
not grant third parties permission to reuse those files.

All hosted jobs use the moving `ubuntu-24.04` runner label. The workflow does
not record an immutable runner-image release, package manifest, filesystem
digest, kernel, CPU, locale implementation, or firmware identity. The following
tools and services are ambient rather than admitted, fixed inputs:

| Ambient input | Used for | Version, license, and provenance status |
| --- | --- | --- |
| GitHub Actions runner and `ubuntu-24.04` image | Job orchestration and base environment | Mutable hosted image; exact version and heterogeneous package licenses are not captured by the workflow |
| GitHub Actions resolver and GitHub API | Action retrieval, event data, dependency review, online audits, and repository reads | Hosted control plane; no client-visible immutable service version or content digest is recorded |
| Node 24 | Checkout, markdownlint, dependency review, artifact upload, and SARIF upload | GitHub-provided runtime; exact patch, binary digest, build provenance, and effective license bundle are not captured |
| Bash and Python 3 standard library | First-party checks and composite Actions | Runner-provided executables; exact versions and binary/package digests are not captured. No PyPI packages are installed by first-party checks |
| Git | Checkout implementation | Runner-provided executable; exact version, configuration closure, and binary digest are not captured |
| Docker daemon, kernel, and CPU | zizmor and Scorecard containers | Runner-provided execution boundary; versions, configuration, and host identity are not captured |
| `curl`, `sha256sum`, `tar`, `install`, `mktemp`, `rm`, and `uname` | Download, verify, extract, install, and clean temporary tools | Runner-provided system tools; exact versions, provenance, package licenses, and binary digests are not captured |
| `make` | Optional local `make check` entry point | Not invoked by required hosted CI; local version and provenance are operator-controlled and unrecorded |
| Artifact, code-scanning, GHCR, and external web services | Storage, analysis upload, image retrieval, and link observations | Mutable hosted services; terms and service behavior are external assumptions, not repository-pinned software inputs |

The usual upstream licenses of a tool family are not a substitute for the
license metadata of the exact runner package. Until an immutable runner
manifest is captured and reviewed, the effective ambient license inventory is
incomplete.

## 6. Deterministic methods are not reproducible evidence

Orange uses **deterministic method validation** to mean that a named procedure
fixes its repository inputs and relevant process variables, avoids undeclared
network access, and is expected to return the same pass/fail classification on
compatible implementations. It uses **reproducible evidence** to mean that an
independent party can reacquire every exact input, recreate the execution
environment, rerun the method, and bind the result to the same subject with
recorded identities and digests.

| Result | What is currently fixed | What the result actually establishes | Why it is not reproducible evidence yet |
| --- | --- | --- | --- |
| `make check` or `scripts/ci/check-repository` | Repository bytes and explicit process variables; no network is used by the method | The scoped repository invariants and synthetic/adversarial fixture expectations pass under the executing Python and OS | Python, Bash, kernel, filesystem, and host tools are not content-fixed; no signed execution manifest or independently replayable environment is emitted |
| Required hosted CI | Repository revision, direct Action SHAs, actionlint archive digest, and zizmor image digest | The configured repository, Markdown, workflow, and workflow-security methods passed in one hosted run | Runner image, Node, Git, Docker host, Action bundles, and service behavior are not fully archived or fixed |
| Dependency Review | Event base/head identities and the selected Action revision | GitHub reported no configured dependency-policy violation for its then-current dependency data | Dependency graph/API state and service implementation are external, time-indexed inputs |
| External Links | lychee archive digest, link-check flags, and repository locators | The non-excluded endpoints produced accepted responses during the run | Remote content, DNS, TLS, routing, rate limits, and server policy change independently of repository bytes |
| Offline zizmor step | Repository revision, Action revision, tool version, and container digest | The selected workflow-analysis rules returned the recorded result with online audits disabled | Docker host, kernel, runner image, and output-retention path are not fixed or bundled |
| Online zizmor and Scorecard | Action/tool identities, both container digests, and the repository revision | A time-specific observation of GitHub metadata or repository posture | Live platform state, APIs, registry availability, runner and container host, artifact/code-scanning endpoints, and other hosted services remain mutable |

A green check, log URL, annotation, SARIF upload, or 14-day artifact is an
execution observation. It is not a signed, thick, offline-replayable evidence
bundle and must not be used to imply future Orange assurance claims.

## 7. Closure requirements

Before repository automation can support an offline-reproducibility or
product-assurance claim, an accepted change must:

1. select an immutable runner image or capture and verify the exact hosted image
   release, package/license manifest, tool versions, kernel, and architecture;
2. archive or independently reacquire every Action source/bundle, release
   archive, and OCI image by digest, with direct and transitive SBOM and license
   review;
3. record the exact execution manifest, environment, command line, subject
   revision, inputs, outputs, timestamps, and failure state;
4. model GitHub and other hosted services as named external oracles, preserve
   permitted request/response evidence, and avoid claims that require their
   implementation to be reproducible;
5. create the full admission, update, rollback, compromise, and end-of-life
   records required by `DEPENDENCY_POLICY.md`; and
6. sign and package claim-relevant results for independent, network-denied
   replay where the applicable claim policy requires it.

These gaps do not invalidate the current defense-in-depth repository checks.
They limit what those checks and their hosted results are permitted to claim.

## 8. Maintenance

The Bootstrap Steward updates this inventory in the same change that alters a
workflow dependency, installer identity, execution mode, or runner label. Each
update must compare the workflow, installer, exact upstream revision, license,
runtime descriptor, and transitive closure. Independent dependency and legal
review remain unavailable in the current single-owner bootstrap state and must
not be implied by an updated table.
