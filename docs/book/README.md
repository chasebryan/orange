# The Orange Book

By Chase Bryan

Three-part study structure, first manuscript increment — 2026-10-05.

[Begin reading: A word before we begin](NOVICE_OPENING.md#a-word-before-we-begin)

This directory is part of *The Orange Book*, not a second book. It supplies
new foundational teaching and a three-part reading structure for the existing
manuscript. The original seventeen chapters, four appendices, manuscript map,
and source disclosure remain in [THE_ORANGE_BOOK.md](../THE_ORANGE_BOOK.md).
They are linked below rather than copied, so corrections continue to have
one source of truth.

The baseline inspected for this increment is manuscript 0.26, snapshot
2026-10-02, file blob `78cf89362dd3bab2639f756e2815d4a0e4582ffb` at repository
commit `c3f529ef568a46ccb3e95dff0c41bc7f06c8792e`. This baseline identifies what
was read; it is not a release or assurance claim.

The part names below are fixed by the owner. Placement of existing chapters
is an editorial integration draft. Existing chapter numbers and anchors
remain unchanged during migration. The new opening has Chapters 1–3 of the
novice sequence; a link to an original chapter retains that manuscript's
number until the complete sequence is ready to renumber consistently.

## Part 1, The Novice

Begin without assumed programming or cryptography knowledge. Establish the
meaning of a term before relying on it, and build enough mathematics to
understand each operation rather than memorize its notation.

### Written in this increment

- [A word before we begin](NOVICE_OPENING.md#a-word-before-we-begin): the
  author-to-reader teaching relationship, study method, and scope of the book.
- [Chapter 1: Before You Hide Anything](NOVICE_OPENING.md#chapter-1-before-you-hide-anything):
  messages, confidentiality, adversary assumptions, public rules and keys,
  specification and implementation, examples and general arguments, integrity
  and authenticity. Six exercises with worked answers.
- [Chapter 2: Two Marks, Many Possibilities](NOVICE_OPENING.md#chapter-2-two-marks-many-possibilities):
  quantity and notation, arithmetic symbols, place value, binary, bits,
  finite strings, counting, powers, bytes, hexadecimal, and interpretation.
  Nine exercises with worked answers.
- [Chapter 3: A Rule You Can Undo](NOVICE_OPENING.md#chapter-3-a-rule-you-can-undo):
  Boolean values, truth tables, AND/OR/XOR/NOT, bitwise operations, masks,
  a complete cancellation argument, loss and retention of information,
  grouping, parity, and the scope of exhaustive checks. Ten exercises with
  worked answers.

### Remaining teaching sequence

This is a curriculum specification, not a claim that these lessons have been
written. Extend the manuscript in the following dependency order:

1. **Tell the Machine Exactly.** Programs and evaluation; values and types;
   functions, parameters and results; a complete, supported Orange source
   file explained character by character. Introduce mathematical functions
   before relying on function notation.
2. **A Place to Work.** Files, extensions, directories, paths, plain text,
   terminals, commands, arguments, exit status, installation and version
   checks. Provide platform-specific instructions only after testing them.
3. **Words Have Edges.** Fixed width, nonnegative and negative integers,
   wrapping arithmetic, congruence, remainders, shifts, rotations, and the
   difference between a value's mathematical meaning and its representation.
   Use paper models freely; executable examples must use admitted Orange
   widths, not invented types such as `Word[4]`.
4. **Name the Intermediate Step.** Expressions, grouping, bindings, explicit
   conversions, arrays, indexing, bounds, conditions, tuples and bounded
   iteration, introduced in dependency order with supported examples.
5. **Read and Repair a Program.** Parsing, type checking, evaluation,
   diagnostics, minimal counterexamples, known-answer tests and reference
   calculations. Explain each command before asking the reader to run it.
6. **Say What You Mean.** Sets, relations, functions, quantifiers, implication,
   equivalence, proof by cases, contradiction, induction, invariants, and
   finite exhaustive arguments. Build on the earlier concrete proofs without
   copying their explanations.
7. **Count What You Do Not Know.** Fractions, ratios, probability,
   conditional knowledge, independent choices, powers and logarithms;
   distinguish key length, distribution and adversary uncertainty.
8. **Protect More Than Appearance.** Explicitly educational classical
   constructions; encoding, encryption, hashing, authentication; keys,
   randomness, nonces and counters; the conditions of the one-time pad.
   Do not introduce a practical primitive before its prerequisites.
9. **The First Complete Study.** A supported small Orange construction,
   a hand-derived expected result, documented tests, a deliberate error,
   a repair, and a precisely scoped explanation of what was established.

The readiness check for Part 2 is demonstrated reasoning, not a certificate
or an assertion that reading alone confers competence. The reader should be
able to interpret a complete small Orange program, derive its elementary
operations, explain the domain of a test, and locate each assumption in a
claim.

## Part 2, The Journeyman

Work from mathematical definitions and authoritative standards to complete
constructions, then investigate representation, composition and implementation
hazards. Establish algorithm prerequisites before presenting the algorithm.
Orange is the working language, not decoration around prose.

### Existing manuscript integrated here

| Existing chapter | Role in the new progression |
| --- | --- |
| [From Surface Text to Meaning](../THE_ORANGE_BOOK.md#chapter-4-from-surface-text-to-meaning) | Develops the novice's reading of source into a disciplined account of meaning. |
| [Orange 2026: The Smallest Honest Slice](../THE_ORANGE_BOOK.md#chapter-8-orange-2026-the-smallest-honest-slice) | Connects exercises to implemented capability and its limits. Retain slice-status qualifications. |
| [Standards as Versioned Inputs](../THE_ORANGE_BOOK.md#chapter-11-standards-as-versioned-inputs) | Turns reading a standard into a reproducible specification task. |
| [The Corpus as Acceptance Test](../THE_ORANGE_BOOK.md#chapter-12-the-corpus-as-acceptance-test) | Connects worked constructions to coverage, test provenance and acceptance boundaries. |
| [Secrets Are a Semantic Concern](../THE_ORANGE_BOOK.md#chapter-6-secrets-are-a-semantic-concern) | Develops secrecy from a message-level goal into a property of computation and observation. |
| [Interoperability and External Validation](../THE_ORANGE_BOOK.md#chapter-13-interoperability-and-external-validation) | Tests whether agreement survives contact with independent implementations and explicit validation scopes. |

### Teaching that must connect those chapters

Develop modular arithmetic into rings and finite fields; probability into
entropy and security parameters; functions into state transformations; and
program evaluation into resource use and observation. Teach byte order,
serialization and format boundaries before comparing standard test vectors.

Derive representative stream and block constructions, hashes, MACs, key
derivation and authenticated encryption. ChaCha20, SHA-256, HMAC, Poly1305 and
AES are candidate running studies tied to exact standards and the repository's
supported examples. Public-key study must first establish its number theory,
then groups and fields, followed by representative encryption, key exchange,
elliptic-curve and signature constructions. Distinguish mathematical
correctness, security models, implementation quality and safe composition.

Address randomness generation, nonce requirements, key lifecycle, side
channels, failure handling, misuse and interoperability where the relevant
construction makes the question concrete. Teach the necessary algebra,
complexity and probability before treating modern lattice- and hash-based
post-quantum constructions. Pin standards and parameter choices when those
chapters are actually written; this index selects no current recommendation.

Each substantial study needs a definition, derivation, supported Orange
expression, provenance of expected results, checked examples, diagnostic
exercise, and explicit non-claims. A name in this curriculum grants no
compiler feature and no production-security endorsement.

## Part 3, The Master

Investigate the meaning and justification of a claim about an exact artifact.
This is where the original book's assurance and architectural core receives
its full treatment, supported by the earlier mathematics and practice.

### Existing manuscript integrated here

| Existing chapter | Role in the new progression |
| --- | --- |
| [The Seams Are the System](../THE_ORANGE_BOOK.md#chapter-1-the-seams-are-the-system) | Unifies the boundaries the reader has encountered in earlier implementations. |
| [Claims, Not Labels](../THE_ORANGE_BOOK.md#chapter-2-claims-not-labels) | Gives precise subjects, properties, assumptions, evidence and outcomes. |
| [One Language, Several Semantic Worlds](../THE_ORANGE_BOOK.md#chapter-3-one-language-several-semantic-worlds) | Relates distinct mathematical and computational interpretations without conflating them. |
| [Proof Search Is Not Proof Checking](../THE_ORANGE_BOOK.md#chapter-5-proof-search-is-not-proof-checking) | Distinguishes finding an argument from accepting a checkable argument. |
| [No Disposable Prototype](../THE_ORANGE_BOOK.md#chapter-7-no-disposable-prototype) | Connects production-lineage implementation choices to durable assurance boundaries. |
| [From Core to Native Bytes](../THE_ORANGE_BOOK.md#chapter-9-from-core-to-native-bytes) | Investigates which properties survive translation to executable artifacts. |
| [The Foreign Boundary](../THE_ORANGE_BOOK.md#chapter-10-the-foreign-boundary) | Makes assumptions at interfaces explicit rather than inheriting them silently. |
| [Evidence That Survives the Build](../THE_ORANGE_BOOK.md#chapter-14-evidence-that-survives-the-build) | Relates claims and evidence to the bytes that are actually produced. |
| [Offline Replay and Trust Budgets](../THE_ORANGE_BOOK.md#chapter-15-offline-replay-and-trust-budgets) | Makes checking and the components it trusts inspectable. |
| [Solo Work Through Incremental Gates](../THE_ORANGE_BOOK.md#chapter-16-solo-work-through-incremental-gates) | Preserves the project's actual development and review constraints. |
| [Releases, Updates, and Failure](../THE_ORANGE_BOOK.md#chapter-17-releases-updates-and-failure) | Carries assurance reasoning through change, replacement and failure. |

### Teaching that must connect those chapters

Develop formal operational and denotational accounts where each is needed;
syntax, judgments and derivations; invariants, refinement and equivalence;
security games, adversary resources and reductions; leakage models and
constant-time claims; proof objects, checking, counterexamples and solver
boundaries. State prerequisites locally and point to their canonical
introduction rather than relying on prestige vocabulary.

Then treat intermediate representations, compiler preservation, translation
validation, machine models, ABIs, memory and foreign-language obligations,
artifact identity, reproducibility, provenance and claim composition. Follow
each assertion to its evidence, assumptions and trusted components. Separate
source-level properties from properties of compiled bytes and physical
execution.

The capstone is an auditable claim dossier: an identified artifact, exact
property and domain, explicit assumptions, reproducible supporting evidence,
a failure or counterexample analysis, and a defensible statement of what
remains unestablished. It must not be called verified merely because the
reader completed it.

## Reference matter retained

- [Existing preface and scope distinctions](../THE_ORANGE_BOOK.md#preface).
- [Appendix A: Current Grammar and CLI](../THE_ORANGE_BOOK.md#appendix-a-current-grammar-and-cli).
- [Appendix B: Decision Ledger](../THE_ORANGE_BOOK.md#appendix-b-decision-ledger).
- [Appendix C: Claim Vocabulary](../THE_ORANGE_BOOK.md#appendix-c-claim-vocabulary).
- [Appendix D: Source Notes](../THE_ORANGE_BOOK.md#appendix-d-source-notes).
- [Existing manuscript map](../THE_ORANGE_BOOK.md#manuscript-map) and
  [sources and drafting disclosure](../THE_ORANGE_BOOK.md#sources-and-drafting-disclosure).
- [New worked answers](NOVICE_OPENING.md#answers-and-worked-reasoning) and
  [epigraph record](NOVICE_OPENING.md#source-notes-and-epigraph-record).

Expand the final reference matter with a notation register, topic and
algorithm indexes, symbol-to-language mapping, prerequisites, exercise
solutions and a standards bibliography. Generate indexes from actual
manuscript anchors; do not publish empty indexes as completed content.

## Editorial rules for continuing the manuscript

Speak as the author teaching one reader. Use direct address to invite a
prediction, locate a mistake, or demand a reason. Do not invent personal
experience, credentials, historical recollections or endorsements. Let
philosophical insight emerge from a technical distinction the reader has
just earned; do not insert a slogan on every page.

Give each concept one canonical definition. Revisit it only to apply it,
qualify it, connect it, or increase its depth. Necessary retrieval practice is
not redundant explanation. Brevity must not remove a prerequisite, a proof
step, an accessibility aid, an important exception or an answer's reasoning.

Introduce symbols and technical terms before asking the reader to use them.
Mark deliberate previews as previews. Separate definition, proposition,
example, implementation observation and proposed capability. A truth table
can establish a finite result; a sample of successful runs cannot silently
establish a universal result.

Open each newly completed chapter with a brief, relevant quotation from a
cryptographer, verified against a primary publication or the speaker's own
record. Record exact wording, context, page or section, consulted version,
translation status and verification date. Do not invent an epigraph for an
unfinished chapter or alter an attributed quotation to improve its prose.
The existing chapters still require their own sourced epigraph and voice
revision; this increment has not represented that work as done.

The names `Current`, `Directed`, `Proposed`, and `Future` retain their existing
meanings. The manuscript does not ratify a semantic proposal, settle D-018,
claim trademark clearance, or qualify a release. Any later renumbering must
preserve or redirect incoming anchors and update all internal references.

## Validation and review boundary

Run the reference checks from the repository root:

```sh
python3 tools/test_book_foundations.py
```

These checks validate the new worked examples and elementary finite models.
They do not execute Orange, establish a cryptographic security claim, or
constitute independent review. The index preserves all seventeen original
chapter links without changing the original file. Check repository-wide
links and documentation policy in CI before any merge.

New drafting and integration are AI-assisted with ChatGPT (GPT-6 Astra Pro),
2026-10-05, at the owner's direction. Owner review is pending. The working
names, legal boundaries and source disclosures of the original manuscript
continue to apply.
