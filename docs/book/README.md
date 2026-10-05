# The Orange Book

By Chase Bryan

Three-part study structure, opening and programming continuation — 2026-10-05.

[Begin reading: A word before we begin](NOVICE_OPENING.md#a-word-before-we-begin)

[Continue reading: Chapters 4–6](NOVICE_PROGRAMMING.md#chapter-4-a-place-to-work)

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
remain unchanged during migration. The new opening and continuation have
Chapters 1–6 of the novice sequence; a link to an original chapter retains
that manuscript's number until the complete sequence is ready to renumber
consistently.

## Part 1, The Novice

Begin without assumed programming or cryptography knowledge. Establish the
meaning of a term before relying on it, and build enough mathematics to
understand each operation rather than memorize its notation.

### Available manuscript

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

- [Chapter 4: A Place to Work](NOVICE_PROGRAMMING.md#chapter-4-a-place-to-work):
  files, directories, paths, plain text, terminals and shells, arguments,
  exit status, source revisions, compiler identity and reproducible working
  records. Seven exercises with answers. One explicit POSIX-style command
  track, not an untested claim of identical installation on every platform.
- [Chapter 5: Tell the Machine Exactly](NOVICE_PROGRAMMING.md#chapter-5-tell-the-machine-exactly):
  a first complete Orange program, every delimiter explained, functions,
  types, expressions, parameters, arguments, scope, calls, diagnostics and
  the distinction between accepted source and fulfilled intention. Eight
  exercises with answers; includes a deliberately rejected literal.
- [Chapter 6: Words Have Edges](NOVICE_PROGRAMMING.md#chapter-6-words-have-edges):
  finite words, signed mathematical integers, Euclidean division, modular
  arithmetic, wrapping, shifts, rotations, literal and computed amounts,
  explicit grouping and the inverse of a small nonsecure composition.
  Twelve exercises with answers; includes a deliberately ungrouped program.
- **N7.** [Name the Intermediate Step](NOVICE_N7_NAME_THE_INTERMEDIATE_STEP.md#n7-name-the-intermediate-step):
  bindings, explicit conversions, arrays with literal indices, and the
  ChaCha20 quarter round written with the intermediate names of RFC 8439
  §2.1. Rejected forms include `x + y as Word[32]`, a literal index past
  the array, a loop index whose proved range leaves the array, and an
  `Int` parameter used as an index. The lesson also chooses with `Bool`
  and `if`, returns the quarter round as one tuple, and repeats with a
  bounded `for` whose index is proved in range before it runs. Twelve
  exercises with worked answers. The lesson's finish line is the four
  outcomes in that section.
  N7 is not the manuscript chapter titled No Disposable Prototype.
- **N8.** [Read and Repair a Program](NOVICE_N8_READ_AND_REPAIR.md#n8-read-and-repair-a-program):
  predicting an `ORC` code and its locus, then confirming with `orangec
  check`, `eval`, and `test`. Repairs stay on the N7 surface: grouping,
  `as`, a literal index, a bounded-loop index, and a test whose expected
  word is wrong. A silent check means the source was well-formed. A passing
  test means one claim's `Bool` was true. Ten exercises with worked answers.
  N8 is not the manuscript chapter titled Orange 2026: The Smallest Honest
  Slice.
- **N9.** [Say What You Mean](NOVICE_LOGIC.md#n9-say-what-you-mean):
  sets, membership, subsets, relations, functions and inverses, quantifiers,
  implication, equivalence, negation, cases, contradiction, contrapositive,
  induction, invariants, and finite exhaustive arguments. Twenty exercises
  with worked answers. The label is lesson N9. It is not a manuscript
  chapter numeral. [Worked answers](NOVICE_LOGIC.md#worked-answers-n9).
- **N10.** [Count What You Do Not Know](NOVICE_PROBABILITY.md#n10-count-what-you-do-not-know):
  fractions, ratios, finite probability spaces, conditional knowledge,
  independence, an elementary birthday bound, expected value, and base-two
  logarithms. Key length, the distribution a key is drawn from, and an
  adversary's uncertainty are separated. Sixteen exercises with worked
  answers. No Orange listing is included. N10 is a provisional label.
  Final numbering waits on the integration plan. N10 does not use the
  formal vocabulary reserved for Say What You Mean.
- **N11.** [Protect More Than Appearance](NOVICE_PROTECT.md#n11-protect-more-than-appearance):
  encoding, encryption, hashing, and authentication as separate claims;
  the shift, affine, substitution, and Vigenère schemes, each proved
  correct and then broken under a stated rule; keys, distributions,
  nonces, and counters; the one-time pad with an elementary proof of
  perfect secrecy; the two-time pad shown by XOR. Twelve Orange listings,
  three of them intentionally rejected. Sixteen exercises with worked
  answers. N11 is a provisional label. It is not manuscript Chapter 11,
  *Standards as Versioned Inputs*.

The opening was approved by the owner before this continuation. Later
corrections to its worked reversal example, counting argument, and
hexadecimal answer are in the opening file; this index describes that
corrected source. Chapters 4–6, N7, N8, N9, N10, and N11, and the new executable checks remain
reviewable new work. Across the six chapters there are 52 exercises with
worked answers. N7 adds twelve further exercises in its own file. N8 adds
ten further exercises in its own file. N9 adds 20. N10 adds sixteen
further exercises, numbered N10.1 onward. N11 adds sixteen further
exercises, numbered N11.1 onward.

### Remaining teaching sequence

Item 1 is drafted as N7. Item 2 is drafted as N8. Item 3 is drafted as N9
and listed above. Item 4 is drafted as N10. Item 5 is drafted as N11.
Item 6 has not been written. Extend the manuscript in the following dependency order:

1. **Name the Intermediate Step.** Drafted in
   [N7](NOVICE_N7_NAME_THE_INTERMEDIATE_STEP.md#n7-name-the-intermediate-step).
   Expressions, grouping, bindings, explicit conversions, arrays, indexing,
   bounds, conditions, tuples and bounded iteration, introduced in
   dependency order with supported examples. The four outcomes in N7 are
   the finish line.
2. **Read and Repair a Program.** Drafted in
   [N8](NOVICE_N8_READ_AND_REPAIR.md#n8-read-and-repair-a-program).
   Parsing, type checking, evaluation, diagnostics, minimal repairs,
   known-answer tests, and the difference between a silent check and a
   passing test. The five outcomes in N8 are the finish line.
3. **Say What You Mean.** Drafted as
   [N9: Say What You Mean](NOVICE_LOGIC.md#n9-say-what-you-mean).
   The locked label is N9. It does not use a manuscript chapter numeral.
   N7 and N8 stay ahead of it in this order and are not part of that file.
4. **Count What You Do Not Know.** Drafted as
   [N10](NOVICE_PROBABILITY.md#n10-count-what-you-do-not-know).
   Fractions, ratios, probability, conditional knowledge, independent
   choices, powers and logarithms; distinguish key length, distribution
   and adversary uncertainty. The label is provisional. Final numbering
   waits on the integration plan.
5. **Protect More Than Appearance.** Drafted as
   [N11](NOVICE_PROTECT.md#n11-protect-more-than-appearance).
   Explicitly educational shift, affine, substitution, and Vigenère
   constructions, each with a stated break; encoding, encryption,
   hashing, and authentication; keys, distributions, nonces, and
   counters; the one-time pad and its exact conditions. The six outcomes
   in N11 are the finish line. The label is provisional. Final numbering
   waits on the integration plan. Do not introduce a practical primitive
   before its prerequisites.
6. **The First Complete Study.** A supported small Orange construction,
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
- [Opening worked answers](NOVICE_OPENING.md#answers-and-worked-reasoning) and
  [epigraph record](NOVICE_OPENING.md#source-notes-and-epigraph-record).
- [Continuation worked answers](NOVICE_PROGRAMMING.md#worked-answers-chapters-46) and
  [source record](NOVICE_PROGRAMMING.md#sources-and-epigraph-record).
- [N9 worked answers](NOVICE_LOGIC.md#worked-answers-n9) and
  [N9 epigraph record](NOVICE_LOGIC.md#sources-and-epigraph-record).

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

These checks validate the worked examples, finite models, the 52 chapter
answers, the N7 answers, the 10 lesson-N8 answers, the 20 lesson-N9
answers, the 16 lesson-N10 answers, the 16 lesson-N11 answers, and document structure. Python test
discovery through
`tools/tests/test_book_foundations.py` loads those checks and the two
printed-continuation audits.
They do not execute Orange, establish a cryptographic security claim, or
constitute independent review. The index preserves all seventeen original
chapter links without changing the original file. Check repository-wide
links and documentation policy in CI before any merge.

The Rust integration test `compiler/crates/orangec/tests/book_novice.rs`
extracts the nine actual Orange listings from the continuation and runs the
compiler against the seven expected successes and two intended rejections.
It also checks that removing the computed-amount parentheses exposes the
literal guard. The same test reads the Orange listings in N7 and checks
their printed results and intended rejections. The same test reads the
Orange listings in N8 and checks the printed values, the printed
diagnostics, the failing and passing tests, and the `--spec` and `--steps`
commands the lesson names. The same test reads the Orange listings in
N11 and checks their printed values, their printed diagnostics, and the
passing known-answer tests. Run it in a build-capable
checkout:

```sh
cargo test --manifest-path compiler/Cargo.toml -p orangec --test book_novice --locked --offline
```

The existence of this test is not a claim that a run passed. The PR and
delivery validation record identify which checks were actually executed.
No native code generation or cryptographic security claim is added.

New drafting and integration through Chapters 4–6 are AI-assisted with
ChatGPT (GPT-6 Astra Pro), 2026-10-05, at the owner's direction. Lessons N8 and N9
are AI-assisted with Grok 4.7 in Cursor, 2026-10-05, at the owner's
direction. Lesson N11 is AI-assisted with Grok 4.7 in Cursor, 2026-10-05,
at the owner's direction. The opening is owner-approved; continuation, N8, N9, and N11 review are
pending. The working names, legal boundaries and source disclosures of the
original manuscript continue to apply.
