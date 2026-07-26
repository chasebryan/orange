# D-009 pre-epoch inputs

Status: input-only non-product research; `draft_unfrozen`

This directory contains only the candidate-neutral D-009 pre-epoch packet and
its eight-row case-input index. It prepares a strict, content-addressed boundary
for the owner-executable solver-trust comparison without freezing an evidence
epoch, admitting or installing a solver or checker, implementing a candidate
adapter, or creating a candidate result.

Every TC-01 through TC-08 index row records absent shared inputs, absent
candidate mappings, zero executable fixtures, unresolved coverage, and an
active freeze blocker. The packet also records that D-004 and D-005 acceptance
is absent. Tool and dependency admissions, executable identities, resources,
host and timeout policy, result/replay schema, correction window, materiality
bands, and physical execution order remain unassigned.

The standard-library Rust laboratory may strictly parse these bytes, verify the
raw bindings to the index and unchanged
[`SOLVER_TRUST_DECISION_SUITE.md`](../../../docs/SOLVER_TRUST_DECISION_SUITE.md),
and enumerate the exact 24 candidate-case identities in memory. The inventory
is case-major and candidate-minor: SP-01, SP-02, and SP-03 for TC-01, then the
same candidate order through TC-08. It is a canonical identity serialization,
not an authorized physical execution order.

The packet's strict canonical JSON SHA-256, excluding its final line feed, is
`e4b01992905589cf459f0f21d00afc92232736dfeed87571f8d1b93aa4b22598`;
its raw file SHA-256, including the final line feed, is
`7dc35a621a852b0684d198719a3df26d3a404b9e9f8b99173d9900691e7b11e1`.
The case-input index's strict canonical JSON SHA-256 is
`2e55c671771d5740b0346992c8b86b9cce0571a8fc3e5b745195b0956010470e`;
its raw file SHA-256, including the final line feed, is
`c5298d625f5392de2774ffb861fe1dc1701b379ebd385cde0584a8cbcd249859`.
The bound suite's raw SHA-256 is
`a26073e6431fb401af4aac6e57dcdfa76b27fe9451c26fb42595d7de14c2a35b`.

The laboratory launches no process, writes no archive, evaluates no solver
output, parses or checks no LRAT/Alethe/proof artifact, validates no
counterexample or claim, and records no execution or evidence. All candidate
states remain absent or `not_performed`; resources, epoch, physical order,
selection, and conclusion remain null or unfrozen.

Current D-009 execution evidence remains 0/24 candidate-case runs. No solver-
trust candidate is selected, preferred, recommended by this laboratory,
installed, or authorized for claim-bearing product work. This preparation does
not accept D-004, D-005, D-006, D-007, or D-009; add or remove a solver from the
logical TCB; close S4; authorize a release; or advance Orange beyond 30%
readiness.
