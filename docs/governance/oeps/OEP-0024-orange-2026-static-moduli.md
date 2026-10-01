---
number: OEP-0024
title: Orange 2026 static moduli
authors:
  - Chase Bryan
champion: Chase Bryan
status: Review
type: Standards
created: 2026-10-01
updated: 2026-10-01
discussion: owner-direction-2026-10-01-complete-1-0
related-decisions:
  - D-002
  - D-004
  - D-023
  - D-025
  - D-026
related-adrs: []
requires:
  - OEP-0001
  - OEP-0002
  - OEP-0003
  - OEP-0004
  - OEP-0005
  - OEP-0006
  - OEP-0007
  - OEP-0008
  - OEP-0009
  - OEP-0010
  - OEP-0011
  - OEP-0012
  - OEP-0013
  - OEP-0014
  - OEP-0015
  - OEP-0016
  - OEP-0017
  - OEP-0018
  - OEP-0019
  - OEP-0020
  - OEP-0021
  - OEP-0023
supersedes: []
superseded-by: null
review-authorities:
  - Orange Project Owner
decision-date: null
decision-revision: null
approval-records: []
---

# OEP-0024: Orange 2026 static moduli

## Abstract

S3t lets a pure specification's finite size parameters occur in modulus
expressions, such as `Mod[(1 << bits) - 19]`. Each declared instance is checked
eagerly with its concrete modulus, including instances not called by source.
Exact residue-domain typing, bounded integer arithmetic, specialization,
reference Core and evaluation costs retain their existing meanings.

[`STATIC_MODULI_2026.md`](../../STATIC_MODULI_2026.md) is the complete
normative proposal with ten bounded conformance rules. This proposal is in
**Review**, follows S3s and OEP-0023, and accompanies a provisional
implementation. Integration is not semantic acceptance.

## Motivation

[OEP-0022](OEP-0022-crypto-language-development-plan.md) P1 calls for
modulus-generic arithmetic. Finite type lists can name several concrete
residue domains, but they cannot express a domain computed from a selected
size. Existing finite size specialization already supplies a bounded,
deterministic checking boundary. Evaluating the modulus within that boundary
supports reusable modular specifications without introducing symbolic
runtime domains or universally quantified typechecking.

The [complete 1.0 execution record](../../RELEASE_1_0_EXECUTION.md) retains
the full product scope. S3t advances pure language engineering while the
dependent proof and compiler decisions remain open.

## Scope and non-goals

S3t admits a function's own finite size names in the existing constant-modulus
expression vocabulary, throughout signatures and body type positions,
conversions, and direct explicit `Mod[...]` type arguments. It changes no
parser production, token, reserved word, command, public Core representation,
evaluation cost or diagnostic code. Its bounded call/index disambiguation
recognizes complete direct `Mod[e]` arguments, including shift expressions.

Global aliases remain concrete; finite type-parameter lists remain independent
of size parameters. General dependent types, parameterized aliases, symbolic
moduli in Core, runtime modulus selection, primality checking, unbounded
generic checking, and refinement contracts are outside this slice.

### Strata assumption

The proposal assumes only total, deterministic finite specialization and the
existing mathematical residue operations. Every instance contains concrete
types and ordinary pure values. It neither selects a D-004 candidate nor
defines an implementation or proof stratum.

## Specification

The companion specification defines vocabulary and scope; arithmetic bounds;
eager instances and caches; exact domain identity; call fitting; aggregates;
body type positions; module scope; resource failure; and compatibility.
Every evaluated modulus is from 2 through 2^521 − 1. Intermediate magnitudes
remain bounded by the existing integer limit, and the existing combined
instance count remains at most 256. An invalid instance rejects the definition
before evaluation; ordinary argument or expected-result fitting must select a
unique concrete instance.

## Alternatives

Concrete finite type lists remain useful when the domains have no shared size
formula. Requiring a separate listed type for every computed domain duplicates
the source relationship rather than checking it. Runtime moduli would alter
value representation, arithmetic and domain identity. Universal dependent
typing would add proof obligations and a different resource boundary. S3t
uses the existing finite checking mechanism and defers both larger changes.

## Compatibility and migration

Existing S3s constant expressions and source behavior retain their meanings.
Original constant-modulus and invalid-type-argument diagnostic notes retain
their exact text. The new static-scope note applies to dependent-modulus faults
and direct `Mod[e]` type-argument faults only.
Previously rejected own size names become valid only at the specified static
positions and only when every instance is valid. Exact moduli remain part of
type identity, so mixed domains continue to reject. Rollback must revert the
size-aware type resolution, fitting, fixtures and coupled proposal documents
together; source using the new positions then returns to its prior rejection.
No stable source, binary, ABI, package or release promise is introduced.

## Semantic and claim effects

The delta is finite size-dependent construction of existing residue domains.
The supported observation is bounded typechecking and reference evaluation at
an identified implementation revision. It establishes no universal theorem,
primality, representation refinement, cryptographic conformance, native
preservation, constant-time behavior or production readiness.

## TCB, axiom, and proof effects

The analyzer and evaluator remain engineering trust dependencies. Instance-local
static environments, modulus bounds, type fitting, cached type identity and
module separation are extended trusted paths. No theorem, axiom, solver,
certificate, authoritative checker or product proof dependency is added.

## Threat, abuse, and leakage effects

An attacker-controlled expression must not evade integer or semantic budgets,
make an uncalled invalid instance successful, borrow another instance's type,
or replace a runtime value with static authority. Domain mismatches and
ambiguous fitting fail closed. Static specialization does not classify secrets
or prove source or target leakage; the existing threat and assurance models
remain in force.

## Target and ABI effects

None. Concrete residue types are reference values, with no selected native
layout, machine instruction, serialization format or foreign contract.

## Standards, errata, and provenance

This is a language proposal. Mathematical ring fixtures exercise computed
domains, including composite moduli; they do not establish a field theorem or
standard conformance. OEP-0022 supplies the broader development context.

## Dependencies, licenses, and IP

No dependency or license is added. D-018 admissions and the publication rules
remain unchanged.

## Conformance, tests, and evidence

`compiler/crates/orangec/tests/s3t_conformance.rs` binds the exact ten
`S3T-01` through `S3T-10` rules. Its repeated CLI corpus covers valid rings,
calls, aggregates, large domains and imported modules, and invalid constants,
domains, bounds and scope. Generated cases exercise arithmetic boundaries,
invalid uncalled instances, exact domain fitting, body annotations and resource
failure. Existing S2 through S3s conformance and the mathematical algorithm
corpus remain compatibility obligations. Test success is implementation
evidence and does not accept this specification or prove a generic theorem.

The exact runner entry points are `s3t_rule_index_is_exact_and_covered`,
`s3t_fixture_inventory_and_outputs_are_exact`,
`s3t_large_moduli_and_boundaries_are_exact`,
`s3t_fitting_and_aggregate_domains_are_exact`,
`s3t_scopes_operators_and_invalid_instances_fail_closed`, and
`s3t_step_and_instance_limits_remain_bounded`. Imported-module rule S3T-08
is covered by the CLI corpus; the other rules also have generated CLI cases.

## Operations, release, and recovery

No service, deployment, key or release mechanism is added. The implementation,
normative rule index, source inventory and coupled documents are validated
together. Merge does not close S3, S4, a complete 1.0 journey or release gates.

## Support and deprecation

The fragment remains pre-alpha and best effort under D-022. It introduces no
support window or compatibility commitment. Owner review is `solo-reviewed`
and never independent review.

## Unresolved questions

- General modulus parameters, parameter constraints and symbolic universal
  reasoning require separate semantics and proof/resource evidence.
- Parameterized aliases and dependent finite type lists require independent
  scope, cycle, fitting and specialization rules.
- Checked representation contracts, wide arithmetic, canonical product Core,
  proof automation and native lowering retain OEP-0022's S4–S7 prerequisites.

## Decision record

On 2026-10-01 the owner directed continued development toward the complete
1.0 product, with the earlier instruction to push and merge when all checks
are green. This bounded S3t implementation advances OEP-0022 P1 and the
permanent frontend. That direction authorizes development and integration;
it does not record semantic or foundational acceptance.

Acceptance requires the owner's exact reviewed revision, decision date and
`solo-reviewed` approval record under the OEP process. Those fields remain
empty. Codex using GPT-6.1 assisted drafting and implementation under the
owner's requested model direction; the owner remains decision authority.
AI output is not technical proof or independent review.
