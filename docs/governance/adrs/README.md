# Architecture Decision Record process

Architecture Decision Records (ADRs) capture implementation-local choices
inside an already accepted OEP and normative boundary. They preserve context
that would otherwise be lost in code review.

An ADR cannot select project-wide architecture, close an unresolved capability
gate, change language semantics or public claims, expand the TCB or axiom set,
weaken a threat control, change a supported target/leakage/ABI profile,
establish licensing terms, or overrule an OEP. If analysis uncovers any such
effect, stop the ADR and escalate the question to an OEP.

## Lifecycle

ADRs use `Proposed`, `Accepted`, `Rejected`, or `Superseded`. Files are named
`ADR-NNNN-short-title.md`; assigned numbers are never reused and old records are
not deleted. A superseding ADR links both directions.

1. Link the accepted OEP or decision that grants the local design boundary.
2. Record constraints, credible options, security/assurance effects, and
   verification evidence before implementation is considered complete.
3. Record owner review for the affected path. During solo mode the owner and
   reviewer may be the same person, and the result is labeled `solo-reviewed`.
4. Accept or reject with rationale; keep rejected options and revisit triggers.
5. Update the ADR when evidence contradicts an assumption, using supersession
   for a different decision rather than rewriting history.

An Accepted ADR records the exact reviewed commit in `decision-revision` and
lists immutable approval references in `approval-records`. During solo mode,
owner approval grants project authority but not independent evidence. Its
related OEP or directed decision must grant the implementation boundary. A merge
event or repository permission is not itself approval.

`ADR-0000-template.md` is not a numbered decision. Mechanical validation checks
metadata and references; it cannot replace technical review.
