---
number: OEP-0001
title: Solo development and incremental capability gates
authors:
  - Chase Bryan
champion: Chase Bryan
status: Accepted
type: Process
created: 2026-07-12
updated: 2026-07-12
discussion: owner-direction-2026-07-12
related-decisions:
  - D-002
  - D-006
  - D-008
  - D-017
  - D-018
  - D-019
  - D-022
  - D-023
  - D-024
related-adrs: []
requires: []
supersedes: []
superseded-by: null
review-authorities:
  - Orange Project Owner
decision-date: 2026-07-12
decision-revision: 469bdec6037f20c8d099d61a09a3d19a55c88231
approval-records:
  - solo-reviewed owner acceptance at merged revision 469bdec6037f20c8d099d61a09a3d19a55c88231
---

# OEP-0001: Solo development and incremental capability gates

## Abstract

Orange operates as a solo project under the project owner until an explicit
later decision changes that fact. Plans must not depend on contributors,
independent reviewers, auditors, laboratories, partner organizations, or other
outside participants being available. Their absence cannot block ordinary
development.

The former all-or-nothing Gate 0 implementation barrier is replaced by
incremental capability gates. Permanent production-lineage implementation may
begin when the owner records the bounded component and its non-claims. A missing
proof, audit, certification, or independent review blocks only the corresponding
claim or capability; it does not block unrelated compiler work.

## Motivation

The original program plan assumed a staffed institution and mandatory outside
validation. No such participants are available. Keeping those assumptions as
entry conditions would prevent Orange from ever acquiring the implementation
that could attract users or demonstrate its ideas.

This proposal preserves the project's assurance discipline by separating work
from claims. Solo-authored code can be tested, reproducible, and suitable for
the permanent product lineage without being described as independently
reviewed, formally verified, certified, production-ready, or cryptographically
assured.

## Scope and non-goals

This proposal governs repository work, decision authority, implementation
sequencing, dependency admission, and public claim wording during solo mode. It
applies indefinitely until the project owner records a superseding decision.

It does not select a proof foundation, finalize Orange semantics, grant a source
license, clear the Orange name, authorize third-party contributions, authorize a
product release, or claim any external validation. Public standards and other
primary materials may still be used as technical sources; the constraint is
that the plan cannot require outside people or organizations to perform work.

## Specification

The project owner is the sole decision, implementation, review, security, and
repository authority during solo mode. Owner review is recorded as
`solo-reviewed`, never as `independent-review` or external approval.

Development uses incremental capability gates:

1. Each component begins with a recorded purpose, boundary, deterministic test
   strategy, and explicit non-claims.
2. An unresolved decision gates only work that would make that decision
   irreversible or would depend on its result.
3. A component may ship only the claims supported by its current evidence.
4. Missing independent or external evidence is reported as unavailable or not
   claimed; it is not silently replaced by a second run from the same owner.
5. No future schedule, roadmap, or release plan may require outside
   participation unless the owner first records that participation as actually
   available and amends this proposal.

The first authorized product slice is the Rust compiler foundation defined by
D-024: source identity and spans, deterministic lexing, structured diagnostics,
and the `orangec` command-line boundary. It uses the pinned Rust toolchain and
the Rust standard library only. Parsing, type checking, proof checking,
optimization, native code generation, cryptographic correctness, and leakage
claims remain outside this slice.

## Alternatives

Continuing the former Gate 0 plan was rejected because its completion depended
on unavailable people and organizations. Describing owner repetition as
independent review was rejected because it would make the evidence misleading.
A disposable compiler prototype was rejected because D-002 still requires each
implementation component to belong to the intended production lineage.

Waiting without implementation was also rejected. It would not reduce any
technical uncertainty and would leave the project permanently blocked by an
organizational assumption that the owner has explicitly withdrawn.

## Compatibility and migration

Existing Gate 0 schemas, fixtures, threat identifiers, and research documents
remain historical inputs. Their external-review fields retain their literal
meaning. Records created in solo mode must not populate those fields with the
owner under another label.

Documents that describe the former staffed reference program remain useful as
long-range design material, but OEP-0001 controls whenever they conflict with
the solo operating model. Future collaborators may be added only through an
explicit governance update; their arrival does not retroactively make earlier
work independent.

## Semantic and claim effects

This process decision changes no Orange program semantics. It authorizes
implementation of proof-neutral compiler infrastructure before D-006 and D-012
are resolved.

All compiler results initially carry pre-alpha, solo-reviewed status. The
following remain unclaimed: language soundness, proof soundness, semantic
preservation, constant-time behavior, cryptographic correctness, ABI stability,
release reproducibility by another principal, certification, and production
readiness.

## TCB, axiom, and proof effects

The Rust toolchain, standard library, operating system, hardware, and sole owner
are engineering trust dependencies for the initial compiler. No Rust component
is admitted to a logical proof TCB by this proposal. No axiom, proof rule, proof
kernel, solver certificate, or theorem is introduced.

Separate implementations written by the same owner may provide differential
testing, but they are implementation diversity rather than organizationally
independent evidence.

## Threat, abuse, and leakage effects

Solo authority increases key-person, self-review, credential, malicious-change,
and mistake risks. Mitigations are narrow changes, deterministic tests,
fail-closed diagnostics, protected history, exact dependency inventory, and
plain disclosure of the single-author boundary.

No leakage or constant-time property is established. Source text is untrusted
input, so the compiler must reject malformed input without panics, path escape,
or ambiguous success. Security-critical claims remain unavailable until their
specific technical evidence exists.

## Target and ABI effects

The initial compiler is a host command-line program only. It defines no Orange
target, object format, calling convention, generated C ABI, Rust wrapper, CPU
feature profile, or native assurance tuple.

Host support is limited to environments on which the pinned Rust toolchain and
repository tests run. Observed host success is not a support promise.

## Standards, errata, and provenance

No cryptographic standard, test vector, or external proof is implemented by
this proposal. The initial compiler's source and tool identities are recorded
in the repository. Later standards work must retain exact provenance even
though a second human reviewer is unavailable.

## Dependencies, licenses, and IP

The initial compiler admits no third-party Rust crates. The pinned Rust
toolchain and standard library are build dependencies; their redistribution
terms must be reviewed before Orange redistributes them.

Orange source remains without a repository-wide license. Owner-authored
development is permitted, but no permission for third-party copying,
modification, redistribution, or contribution is implied. The working Orange
name is permitted for repository development without claiming trademark
clearance.

## Conformance, tests, and evidence

The repository must run formatting, linting, unit tests, integration tests, and
the existing foundation-policy suite. Lexer tests cover valid tokens, byte
spans, Unicode input, malformed input, comments, integer boundaries, stable
diagnostics, and deterministic repeated execution.

Passing tests establish only the tested behavior at the recorded revision.
They are not proof, independent review, a security audit, or release evidence.

## Operations, release, and recovery

No product release is authorized by this proposal. Generated build output stays
out of version control. The owner may recover from a bad development change by
reverting it while preserving history and its decision record.

The repository continues to use required CI and private vulnerability
reporting. Single-owner credentials and recovery remain a disclosed operational
risk rather than a hidden multi-party control.

## Support and deprecation

Compiler development is pre-alpha and best effort. There is no SLA, LTS window,
compatibility promise, or migration service. Every implemented behavior may
change until a later decision explicitly stabilizes it.

Permanent-lineage means components are engineered to survive; it does not make
unfinished interfaces stable.

## Unresolved questions

The proof foundation, core semantics, license, final product name, target
profiles, leakage semantics, corpus, ABI, package model, release process, and
support window remain unresolved. Each is decided when it becomes necessary for
the next bounded component or claim.

No unresolved question in this list blocks the authorized compiler-foundation
slice unless the implementation would silently answer it.

## Decision record

On 2026-07-12 the project owner explicitly directed that Orange proceed as a
solo project, that outside participation be treated as unavailable for all
current planning, and that compiler work begin. This proposal records that
direction and the resulting claim boundary.

On 2026-07-12 the Orange Project Owner reviewed and accepted this record against
exact merged revision
`469bdec6037f20c8d099d61a09a3d19a55c88231`. That revision contains the S1
compiler foundation, coupled policy, dependency boundary, tests, and the honest
solo-governance wording authorized by this proposal.

The approval record is literally `solo-reviewed`. The author and decision
authority are the same sole owner. There was no second principal, independent
approval, audit, proof, or external validation, and this Accepted status must
never be used to imply one.
