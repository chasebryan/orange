# Solo-bootstrap CI and compiler-toolchain dependency inventory

Status: current direct-dependency inventory and gap record; not a reproducible-
build claim or legal approval

Inventory amendment: 2026-07-14

Hosted execution snapshot: 2026-07-11

## 1. Scope and claim boundary

This document records the Rust compiler toolchain plus third-party Actions,
downloaded executables, hosted services, and ambient runner tools used by the
five workflows in `.github/workflows/`. It also separates repeatable validation
methods from reproducible evidence. The inventory does not by itself satisfy
every admission or release record required by
[`DEPENDENCY_POLICY.md`](../../DEPENDENCY_POLICY.md).

A full Git commit SHA fixes the selected upstream Action revision under
GitHub's object model. A container digest fixes the selected OCI manifest. Each
identity still depends on its registry or service for retrieval and does not,
by itself, fix the hosted runner, service APIs, network responses, provenance,
or transitive dependency closure. A release-archive checksum fixes the bytes
accepted by an installer, but does not archive those bytes or establish their
publisher identity. Those distinctions are explicit below.

The pinned Rust toolchain and standard library are admitted build/bootstrap
dependencies for D-024 but are not a logical proof TCB. CI Actions and
repository tools remain repository/build dependencies. D-018 remains open, so
recording upstream terms is factual provenance only and does not authorize
redistribution or grant a license for this repository.

## 2. Compiler toolchain

| Component | Identity and role | Current closure | Known gap |
| --- | --- | --- | --- |
| Rust toolchain | `rustc`/Cargo 1.96.1 selected by `rust-toolchain.toml`; compiles and tests the Rust 2024 workspace | Exact release version and required `rustfmt`/`clippy` components are selected; initial Cargo graph has no third-party crates | Platform archives, installer, standard-library bytes, signatures, licenses, and transitive host inputs are not vendored or digest-bound here |
| Cargo workspace | `compiler/Cargo.toml` and `compiler/Cargo.lock`; dependency resolution and build orchestration | `--locked --offline` is required; lock graph contains only workspace packages; the protected gate builds optimized `orangec` twice in distinct fresh target trees and requires byte equality | Cargo and rustc remain toolchain trust; a lock file cannot archive the toolchain; same-host repeatability is not an independent or cross-platform rebuild |
| Rust standard library | Runtime/build interface used by `orange-compiler` and `orangec` | Supplied by the selected toolchain; no additional crate registry input | Target-specific standard-library and OS behavior are trusted; redistribution review remains open |

These records authorize local owner development only. They do not establish a
hermetic build, toolchain redistribution right, compiler correctness, or release
provenance.

The byte-comparison check fixes the process environment, toolchain selection,
locale, timezone, and source-date epoch, then rebuilds the same checkout twice
with separate target trees. Both builds still share one host, source path,
toolchain installation, Cargo home, owner, and trust domain. The result is a
regression check for same-host nondeterminism, not independently reproduced
release evidence.

## 3. Workflow map

| Workflow | Trigger and role | Direct external execution dependencies | Network or hosted-state boundary |
| --- | --- | --- | --- |
| `ci.yml` | Required pull-request, merge-queue, and `main` repository and compiler checks | Rust toolchain, Checkout, markdownlint, actionlint, and zizmor | An allowlisted environment runs rustup to install the pinned minimal toolchain with Clippy and rustfmt; GitHub resolves Actions; actionlint is downloaded; the digest-pinned zizmor image is pulled from GHCR |
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

### Observed post-merge execution snapshot

The following is a time-indexed GitHub observation for exact `main`
`9f458c04542c512a8c04b00cb7ce4ef6bacd1a79`, not an additional dependency
admission or reproducibility claim. Pull request #3 head
`8e26785f87c3866cc12915d7037820c608d6708d` merged green through the exact
Required CI and Dependency Review contexts bound by ruleset `18810248` to
GitHub Actions integration ID `15368`.

| Execution boundary | Observed result | Unclosed input or proof gap |
| --- | --- | --- |
| Required CI | `29171653266` succeeded; policy `0.1.4` and 65 tests passed | Runner image, Python, Node, Git, Docker host, Action bundles, and GitHub service behavior are not fully fixed or archived |
| Dependency Review | Pull request #3's required check was green and app-bound by the active ruleset | Dependency graph and API state are hosted, time-indexed inputs; this workflow has no `main` push execution to claim |
| Workflow Online Audit | `29171653264` succeeded on `main` | Current GitHub metadata is mutable, and this observation does not prove scheduled-event execution |
| External Links | `29171653282` succeeded on `main` | DNS, TLS, routing, rate limits, remote content, and server policy remain mutable; scheduled-event execution is unproven |
| OpenSSF Scorecard | `29171653261` succeeded on `main` | Registry, runner, Docker host, GitHub APIs, artifact and code-scanning services remain external; Scorecard is posture data, not SAST or CodeQL; scheduled-event execution is unproven |
| CodeQL default setup | `29171652948` succeeded on `main`; Python analysis `1467719573` returned 0 results/50 rules and Actions analysis `1467719309` returned 0 results/23 rules, without errors or warnings | GitHub's analyzer, queries, runner, extraction, service behavior, and result storage are hosted inputs; zero results are not proof of absence and no CodeQL blocking threshold has been tested |

Alerts #1-#3 (`py/path-injection`) were fixed, not dismissed, at
`2026-07-11T23:09:26Z`. This records the hosted remediation disposition for the
snapshot only. It is not a signed result bundle, independent retest, or proof
that other paths and future revisions are safe.

Scorecard run `29171653261`, job `86593727305`, executed the admitted image
digest with a read-only root filesystem, bounded `tmpfs`, dropped capabilities
except `DAC_OVERRIDE`, `no-new-privileges`, and a 256-process limit. Public
publication and OIDC remained disabled. Artifact `8253693735` was unexpired at
readback: its 16,361-byte archive had GitHub digest
`sha256:404535706b75a2c3468e914e08c11cb9b537fd5e29fc82808bb330e5df58e7fe`;
the extracted 74,492-byte `results.sarif` had SHA-256
`b88e7044a7580230177f39cccd6d5e42a968b62a6f1f8cf054b26d31de8d69f5`,
three SARIF runs, 18 rules, and seven results. Code-scanning accepted those
results as analyses `1467719019`, `1467719022`, and `1467719027` without errors
or warnings. A local pattern scan of all 190,363 log bytes and the retained
SARIF found no GitHub token, classic PAT, bearer credential, or unmasked
authorization-value pattern; the two token-value log fields were masked. These
checks reduce accidental-disclosure uncertainty for this run only and do not
make the mutable hosted execution hermetic or reproducible.

## 4. Pinned Action inventory

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

## 5. Downloaded executable and container inventory

| Tool | Selected artifact identity | Verification performed by current automation | Upstream license and provenance | Remaining limitation |
| --- | --- | --- | --- | --- |
| actionlint 1.7.12 | Linux AMD64 `sha256:8aca8db96f1b94770f1b0d72b6dddcb1ebb8123cb3712530b08cc387b349a3d8`; Linux ARM64 `sha256:325e971b6ba9bfa504672e29be93c24981eeb1c07576d730e9f7c8805afff0c6` | [`scripts/ci/install-actionlint`](../../scripts/ci/install-actionlint) limits connection setup to 20 seconds and the complete HTTPS fetch to five minutes, enforces the 32 MiB archive cap in both curl and the operating system, caps the extracted member at 64 MiB, checks the architecture-specific SHA-256, extracts only `actionlint` without archive ownership or permissions, requires one nonempty, regular, non-symlinked, single-link result, and treats the requested destination as an exact file path | [MIT at release `v1.7.12`](https://github.com/rhysd/actionlint/blob/v1.7.12/LICENSE.txt); [upstream release](https://github.com/rhysd/actionlint/releases/tag/v1.7.12) is the provenance locator | Archive bytes and upstream release metadata are not mirrored or signed into this repository; installer trust still includes DNS, TLS, GitHub availability, and ambient `curl`, `sha256sum`, `tar`, `stat`, and `install` |
| lychee 0.24.2 | Linux x86-64 GNU archive `sha256:1f4e0ef7f6554a6ed33dd7ac144fb2e1bbed98598e7af973042fc5cd43951c9a` | [`scripts/ci/install-lychee`](../../scripts/ci/install-lychee) limits connection setup to 20 seconds and the complete HTTPS fetch to five minutes, enforces the 64 MiB archive cap in both curl and the operating system, caps the extracted member at 128 MiB, checks SHA-256, extracts only the expected `lychee` member without archive ownership or permissions, requires one nonempty, regular, non-symlinked, single-link result, and treats the requested destination as an exact file path | Dual [Apache-2.0](https://github.com/lycheeverse/lychee/blob/lychee-v0.24.2/LICENSE-APACHE) or [MIT](https://github.com/lycheeverse/lychee/blob/lychee-v0.24.2/LICENSE-MIT) at release `lychee-v0.24.2`; [upstream release](https://github.com/lycheeverse/lychee/releases/tag/lychee-v0.24.2) is the provenance locator | Only Linux x86-64 is admitted by the installer; archive and release metadata are not mirrored, and link results depend on live remote endpoints |
| zizmor 1.26.1 container | `ghcr.io/zizmorcore/zizmor:1.26.1@sha256:d1117e5dbd9ee4970644067b534ab6ab50371f3c6f7f4d05446eb603a6e78f48` | The exact pinned composite Action rejects an unknown version and constructs the image reference from its committed version-to-digest map | [MIT at release `v1.26.1`](https://github.com/zizmorcore/zizmor/blob/v1.26.1/LICENSE); [upstream release](https://github.com/zizmorcore/zizmor/releases/tag/v1.26.1) and GHCR are the provenance locators | The image is content-selected but not mirrored; execution still depends on the registry, ambient Docker daemon/kernel, runner CPU, and network. Online-audit output additionally depends on current GitHub state |
| Scorecard 2.4.3 container | `ghcr.io/ossf/scorecard-action@sha256:2dd6a6d60100f78ef24e14a47941d0087a524b4d3642041558239b1c6097c941` | [`scorecard.yml`](../../.github/workflows/scorecard.yml) invokes `docker run` with the exact digest; repository validation admits that image and enforces the command, read-only event mount, workspace mount, environment forwarding, dropped capabilities, and no-new-privileges | [Apache-2.0](https://github.com/ossf/scorecard-action/blob/4eaacf0543bb3f2c246792bd56e8cdeffafb205a/LICENSE); the image's upstream OCI revision label is [`4eaacf0543bb3f2c246792bd56e8cdeffafb205a`](https://github.com/ossf/scorecard-action/tree/4eaacf0543bb3f2c246792bd56e8cdeffafb205a), and GHCR is the runtime provenance locator | The runtime is content-selected but not mirrored; execution still depends on GHCR availability and integrity, correct publisher provenance and digest admission, and the ambient Docker daemon, kernel, runner CPU, and network. The container root filesystem is read-only, while the workspace remains writable for `results.sarif`. The hosted result is not hermetic or independently reproducible evidence |

Orange deliberately sets `INPUT_PUBLISH_RESULTS` to `false`. OpenSSF's public
publication API requires the official `ossf/scorecard-action` step identity,
while that exact Action descriptor selects its runtime through a mutable image
tag. Current repository policy therefore chooses the content-addressed runtime
and retains the SARIF artifact and GitHub code-scanning upload instead. The job
requests no OIDC token and makes no Orange-authenticated publication to
`api.scorecard.dev`.

Checksums above are copied from the executable installer scripts or the exact
selected Action version map. A reviewer must compare this table with those
authoritative repository bytes after every change; this prose is not an
enforcement mechanism.

## 6. First-party methods and ambient dependencies

The required invariant check invokes repository-owned Bash and Python files:

- [`scripts/ci/check-repository`](../../scripts/ci/check-repository) sets the
  locale and timezone, resolves and enters the physical repository root, fixes
  the source-date epoch to zero, then runs the closed-tree
  [`tools/validate_foundation.py`](../../tools/validate_foundation.py) gate,
  the standard-library `unittest` suite, and the Rust compiler checks in that
  serialized order; before Make starts, it removes inherited Make control and
  shell-startup variables and disables built-in rules and variables;
- each Python recipe starts from an allowlisted environment with a fixed hash
  seed, skips `site` initialization, excludes unsafe path injection, suppresses
  bytecode writes, and forces UTF-8 mode; foundation tests also redirect
  bytecode lookup to a fresh temporary root so ignored checkout caches cannot
  execute;
- the protected `check-compiler` Make recipe runs Cargo from the filesystem
  root with a fresh temporary Cargo home and target tree, the selected Rust
  1.96.1 toolchain, offline mode, and an allowlisted process environment;
- every repository Bash helper selects `/bin/bash` directly in privileged
  startup mode, suppressing inherited shell functions and `BASH_ENV` before
  the script body runs;
- every hosted `run` step likewise selects `/bin/bash -p` by absolute path and
  retains immediate-exit and pipeline-failure handling through the supported
  custom-shell command template;
- the actionlint and lychee installers use only `/usr/bin` and `/bin` for
  ambient command lookup, reject empty destination arguments, terminate
  options before caller-selected install destinations, disable curl's default
  configuration, clear inherited tar/gzip option variables, and enforce the
  archive identities recorded above; and
- [`scripts/ci/check-external-links`](../../scripts/ci/check-external-links)
  defines the live link-check method and its documented IACR exclusion, with
  a nonempty caller-selected executable path, prior option termination, and
  an allowlisted process environment that excludes ambient checker, proxy,
  logging, locale, and home-directory configuration. It recursively scans
  every nonignored repository Markdown file and explicitly includes hidden
  GitHub Markdown and YAML inputs.

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
| `curl`, `sha256sum`, `tar`, `stat`, `install`, `mktemp`, `rm`, and `uname` | Download, verify, extract, inspect, install, and clean temporary tools | Runner-provided system tools; exact versions, provenance, package licenses, and binary digests are not captured |
| GNU Make | Protected compiler recipe and optional local `make check` entry point | Required hosted CI invokes the compiler target with built-in rules and variables disabled; exact binary version, provenance, and package license are unrecorded |
| Artifact, code-scanning, GHCR, and external web services | Storage, analysis upload, image retrieval, and link observations | Mutable hosted services; terms and service behavior are external assumptions, not repository-pinned software inputs |

The usual upstream licenses of a tool family are not a substitute for the
license metadata of the exact runner package. Until an immutable runner
manifest is captured and reviewed, the effective ambient license inventory is
incomplete.

## 7. Deterministic methods are not reproducible evidence

Orange uses **deterministic method validation** to mean that a named procedure
fixes its repository inputs and relevant process variables, avoids undeclared
network access, and is expected to return the same pass/fail classification on
compatible implementations. It uses **reproducible evidence** to mean that an
independent party can reacquire every exact input, recreate the execution
environment, rerun the method, and bind the result to the same subject with
recorded identities and digests.

| Result | What is currently fixed | What the result actually establishes | Why it is not reproducible evidence yet |
| --- | --- | --- | --- |
| `scripts/ci/check-repository` | Repository bytes, Cargo lock, selected Rust version, allowlisted Python and Cargo environments, and fresh bytecode/build cache roots; Cargo runs locked/offline after the toolchain exists | Rust formatting, lint, docs, debug and optimized unit/CLI tests, repository invariants, and synthetic/adversarial fixture expectations pass under the executing toolchain and OS | Rust archives, Python, shell, Make, kernel, filesystem, and host tools are not content-fixed; no signed execution manifest or independently replayable environment is emitted |
| Required hosted CI | Repository revision, direct Action SHAs, actionlint archive digest, and zizmor image digest | The configured repository, Markdown, workflow, and workflow-security methods passed in one hosted run | Runner image, Node, Git, Docker host, Action bundles, and service behavior are not fully archived or fixed |
| Dependency Review | Event base/head identities and the selected Action revision | GitHub reported no configured dependency-policy violation for its then-current dependency data | Dependency graph/API state and service implementation are external, time-indexed inputs |
| External Links | lychee archive digest, link-check flags, and repository locators | The non-excluded endpoints produced accepted responses during the run | Remote content, DNS, TLS, routing, rate limits, and server policy change independently of repository bytes |
| Offline zizmor step | Repository revision, Action revision, tool version, and container digest | The selected workflow-analysis rules returned the recorded result with online audits disabled | Docker host, kernel, runner image, and output-retention path are not fixed or bundled |
| Online zizmor and Scorecard | Action/tool identities, both container digests, and the repository revision | A time-specific observation of GitHub metadata or repository posture | Live platform state, APIs, registry availability, runner and container host, artifact/code-scanning endpoints, and other hosted services remain mutable |
| CodeQL default setup | Exact repository revision plus hosted run, analysis, language, suite, and rule-count identifiers | GitHub reported successful `actions` and `python` analysis for the recorded snapshot and recorded three earlier `py/path-injection` alerts as fixed rather than dismissed | Analyzer/query implementation, extraction, standard runner, service state, and storage are not independently fixed or replayable; zero results do not prove absence, and no required-check threshold or negative blocking test exists |

A green check, log URL, annotation, SARIF upload, or 14-day artifact is an
execution observation. It is not a signed, thick, offline-replayable evidence
bundle and must not be used to imply future Orange assurance claims.

## 8. Closure requirements

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

## 9. Maintenance

The project owner updates this inventory in the same change that alters a
workflow dependency, installer identity, execution mode, or runner label. Each
update must compare the workflow, installer, exact upstream revision, license,
runtime descriptor, and transitive closure. Independent dependency and legal
review remain unavailable in the current single-owner bootstrap state and must
not be implied by an updated table.
