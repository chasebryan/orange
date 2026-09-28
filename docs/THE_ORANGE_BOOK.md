# The Orange Book

![Hand-drawn Orange carton emblem and wordmark](../assets/brand/orange-handdrawn-marker-banner.png)

By Chase Bryan

Status: living pre-alpha reader guide

Snapshot: 2026-09-28

Manuscript version: 0.5

> The Orange Book explains why Orange exists, what it is intended to become,
> what has actually been built, and which questions remain open. It is not a
> normative language specification, proof, assurance report, license grant, or
> release claim.

## Contents

- [Preface](#preface)

Part I: Why Orange

- [Chapter 1: The Seams Are the System](#chapter-1-the-seams-are-the-system)
- [Chapter 2: Claims, Not Labels](#chapter-2-claims-not-labels)
- [Chapter 3: One Language, Several Semantic Worlds](#chapter-3-one-language-several-semantic-worlds)

Part II: Meaning and Trust

- [Chapter 4: From Surface Text to Meaning](#chapter-4-from-surface-text-to-meaning)
- [Chapter 5: Proof Search Is Not Proof Checking](#chapter-5-proof-search-is-not-proof-checking)
- [Chapter 6: Secrets Are a Semantic Concern](#chapter-6-secrets-are-a-semantic-concern)

Part III: Building the Language

- [Chapter 7: No Disposable Prototype](#chapter-7-no-disposable-prototype)
- [Chapter 8: Orange 2026: The Smallest Honest Slice](#chapter-8-orange-2026-the-smallest-honest-slice)
- [Chapter 9: From Core to Native Bytes](#chapter-9-from-core-to-native-bytes)
- [Chapter 10: The Foreign Boundary](#chapter-10-the-foreign-boundary)

Part IV: Cryptography in Practice

- [Chapter 11: Standards as Versioned Inputs](#chapter-11-standards-as-versioned-inputs)
- [Chapter 12: The Corpus as Acceptance Test](#chapter-12-the-corpus-as-acceptance-test)
- [Chapter 13: Interoperability and External Validation](#chapter-13-interoperability-and-external-validation)

Part V: Operating Orange

- [Chapter 14: Evidence That Survives the Build](#chapter-14-evidence-that-survives-the-build)
- [Chapter 15: Offline Replay and Trust Budgets](#chapter-15-offline-replay-and-trust-budgets)
- [Chapter 16: Solo Work Through Incremental Gates](#chapter-16-solo-work-through-incremental-gates)
- [Chapter 17: Releases, Updates, and Failure](#chapter-17-releases-updates-and-failure)

Appendices

- [Appendix A: Current Grammar and CLI](#appendix-a-current-grammar-and-cli)
- [Appendix B: Decision Ledger](#appendix-b-decision-ledger)
- [Appendix C: Claim Vocabulary](#appendix-c-claim-vocabulary)
- [Appendix D: Source Notes](#appendix-d-source-notes)

- [Manuscript map](#manuscript-map)
- [Sources and drafting disclosure](#sources-and-drafting-disclosure)

## Preface

Orange begins with an uncomfortable observation: high-assurance cryptography is
not one problem. It is a chain of problems whose boundaries are easy to hide.
A mathematical construction, an executable implementation, a proof, a compiler,
a test suite, a binary, and the machine that runs it can each be reasonable in
isolation while the claim connecting them remains unclear.

This book is the reader's guide to that chain. It is meant for cryptographic
implementers, verification engineers, cryptographers, standards authors,
library maintainers, integrators, auditors, and curious programmers who want to
understand the project without first reading every planning record. It will
explain the ideas in ordinary technical prose, show the permanent implementation
as it grows, and keep the boundary between aspiration and evidence visible.

Above all, Orange is written for people who read mathematics for a living:
cryptographers, cryptologists, and cryptanalysts. Its aim is to be exact and
beautiful at once. A specification should read like the clause of the standard
it transcribes, with the same words, the same rotations, and the same modular
additions, and every statement Orange makes about a piece of code should be as
precise as the mathematics behind it. That aim shapes the language and it
shapes this book. Where the prose is careful about a qualifier, it is because
the qualifier is where the truth lives.

That boundary matters because Orange is young. The repository is in solo,
pre-alpha compiler development. It has a production-lineage Rust compiler
foundation, a deterministic lexer, a bounded parser, structured diagnostics,
and a deliberately small Orange 2026 grammar. The accepted S3a slice adds
bounded semantic checking and reference evaluation for closed typed `spec`
literals. The S3b slice, implemented and awaiting the owner's acceptance of its
specification, extends them to pure functions over integers and 8- to 64-bit
words, and the S3c slice, likewise implemented and in review, adds named
intermediate values and explicit conversions between those types. None of them
adds control flow, typed implementations, refinement, code
generation, a standard library, a proof checker, package or release behavior,
or a verified cryptographic implementation. A passing test suite is
evidence about the implemented slice; it is not evidence that the eventual
language or compiler is sound.

The manuscript uses four kinds of statements:

- **Current** describes behavior or evidence present in the repository now.
- **Directed** describes an explicit project-owner decision that controls work.
- **Proposed** describes an architecture or policy recommended for a later
  decision gate.
- **Future** describes an intended capability whose design, implementation, or
  evidence is not complete.

The distinction is not decorative. A proposed architecture cannot become an
accepted one merely because a chapter speaks about it fluently. When this book
and a normative source disagree, the normative source, accepted Orange
Enhancement Proposal, and [decision register](DECISIONS.md) control. The book
must then be corrected.

The book has five parts. Part I explains why Orange exists and what kind of
language it is meant to be. Part II covers meaning and trust: semantics,
proofs, and secrets. Part III describes the compiler, from the permanent
foundation that exists today to native code and foreign interfaces. Part IV
turns to cryptography itself: standards, the corpus, and external validation.
Part V covers evidence, replay, the solo operating model, and releases. The
appendices collect the current grammar and command line, the decision ledger,
the claim vocabulary, and source notes. A cryptographer may prefer to read
Chapters 1 through 3, 6, 11, and 12 first; an implementer, Chapters 4 and 7
through 10; an auditor, Chapters 2, 14, 15, and 17.

The title **The Orange Book** and the name **Orange** are repository-local
working names. They do not assert trademark clearance or authorize publication
to a package registry, domain, or other public namespace. The manuscript is a
living part of the solo project, not a product release.

## Chapter 1: The Seams Are the System

Cryptographic software is asked to carry several different meanings at once.
It has a mathematical meaning: a function, construction, or protocol is being
described. It has an operational meaning: instructions execute on concrete
machines with finite words, memory, errors, and timing behavior. It has a
security meaning: an attacker is granted certain powers and a property is
claimed under stated assumptions. Finally, it has an evidentiary meaning: some
combination of proofs, certificates, tests, reviews, provenance, and build
records is supposed to justify what users are told.

Those meanings rarely live in one place today. A specification may be written
in a notation suited to mathematicians. A fast implementation may be C, Rust,
or assembly. Functional correctness may be argued in a proof assistant.
Constant-time behavior may be checked by a separate analysis. Game-based
security may use yet another formalism. Test vectors, compiler flags, linker
inputs, target features, and release attestations collect around the outside.

Each tool can be excellent. The difficulty is the crossings between them.

### Six questions for one artifact

Imagine receiving a native library that exports a cryptographic routine. A
serious account of that library should answer at least six questions:

1. What mathematical construction or protocol component is intended?
2. Which executable implementation is claimed to realize it?
3. Which properties are claimed, for which inputs, targets, and versions?
4. Which assumptions and leakage model limit each property?
5. Which proof object, checked certificate, test corpus, or external record
   supports each claim?
6. Which source, toolchain, dependencies, and invocation produced the shipped
   bytes?

It is possible to have good answers to several of these questions and no
reliable answer to the next one. A proof about a mathematical function does not
identify the binary loaded by an application. A correct implementation at one
intermediate representation does not establish that a later optimization kept
its behavior. Passing official vectors does not cover all inputs. Source-level
control-flow discipline does not automatically describe final machine-code
leakage. A reproducible build faithfully reproduces a bug just as readily as a
correct program.

The transition is therefore part of the claim. Serialization is not merely
plumbing when a theorem fingerprint can be attached to the wrong definition.
Foreign-function glue is not merely plumbing when buffer length, aliasing,
alignment, or error behavior can violate a proved precondition. The compiler is
not merely plumbing when the advertised property must survive to object bytes.
The release process is not merely plumbing when an attacker can replace either
the code or its evidence.

The seams are part of the system.

### The proposed vertical artifact

Orange's directed mission is to specify, implement, and verify cryptography.
The project's proposed answer to the seam problem is a claim-oriented language
and build graph. In the intended end state, a package connects standards
provenance, readable specifications, executable implementations, named target
and leakage models, checked transformations, native artifacts, foreign-interface
metadata, tests, and explicit assumptions.

The important word is *connects*. Orange is not useful merely because it can
place several kinds of text in one source file. A shared spelling is not a
semantic relationship. The system must preserve exact identities and record
which checked step supports each edge of the graph. Where a relationship has
not been established, the graph must say so.

This is why the long-term product is larger than a notation or compiler front
end. A language can make intent expressible, but an assurance claim also needs
semantics, checking rules, artifact identity, assumptions, and replay. A code
generator can produce fast bytes, but speed says nothing about whether those
bytes implement the named specification. A theorem prover can check a proof,
but the theorem may not be the property an integrator thinks it is.

Many architectural details of this vertical artifact remain proposed or under
investigation. The current Typed Reference Core for literal specifications
has no canonical encoding, proof identity, or refinement role and does not
select the complete semantic Core. Orange has also not selected its proof
foundation, proof format, solver policy, leakage baseline, native target
envelope, stable foreign boundary, package model, or flagship cryptography
corpus. This chapter describes the problem those choices must eventually solve;
it does not settle them.

### Claims, not labels

The word *verified* compresses too much. It can refer to well-formed syntax,
memory safety, functional refinement, standards conformance, termination,
constant-time behavior under a particular observation model, compiler
preservation, ABI correctness, game-based security, or simply the fact that
tests were run. These properties are related, but none is a universal substitute
for the others.

Suppose a routine passes every published vector for a standard. That is useful
conformance evidence for those cases. It is not a proof for every input, and it
does not establish memory safety. Suppose a proof establishes that a source
implementation refines a mathematical specification. That does not, by itself,
show that emitted machine code preserves the result or that its memory addresses
are independent of secret data. Suppose a binary is rebuilt byte for byte by a
second machine. That establishes a reproducibility fact, not cryptographic
correctness.

Orange therefore proposes to make the unit of assurance a scoped claim. A claim
should name its subject, property, model, target, assumptions, evidence, and
outcome. Different claims about one exported routine can have different states.
A conformance claim may be satisfied while a leakage claim is unresolved. A
platform may be unsupported even though a mathematical proof is valid. An
external validation can be recorded without being misrepresented as a theorem
checked by the Orange kernel.

The intended outcomes also need more precision than success and failure. A
claim can be satisfied, not satisfied, unresolved, or unsupported. A timeout is
not a proof failure, but it cannot become a proof success. An assumption is a
visible dependency, not evidence that proves itself. A neighboring
implementation's test result cannot silently migrate to the implementation
being shipped.

In the proposed design, this discipline would change the shape of a build.
Instead of producing a binary and then attaching a broad adjective, the build
would produce an artifact together with a graph of narrowly worded claims. Each
edge would name the authority that justifies it. Some authorities may be
machine-checked proofs. Some may be checked certificates or test runs. An owner
review may support only an explicitly scoped owner-audit or governance record;
it cannot impersonate a technical proof, external validation, certification, or
independent review. Identified external records retain their external scope and
authority. These differences would remain visible.

### Trust does not disappear

Formal methods can shrink and clarify trust, but they do not make trust vanish.
A small proof kernel is still software. The statement fed to it can be wrong.
The parser can construct the wrong syntax tree. A compiler model can omit an
instruction behavior. An assembler or linker can break the connection to final
bytes. A foreign caller can violate a buffer contract. A CPU, operating system,
or entropy source can behave outside the model. A release account can be
compromised.

Orange's intended response is to publish the trusted computing base for each
kind of claim and to keep it specific. The trusted base for a parser behavior
claim is not the same as the trusted base for a native constant-time claim. A
component appears because a claim actually depends on it, not because every
claim inherits one project-wide trust list.

Tests remain important inside this approach. They find regressions, exercise
error paths, compare implementations, and expose resource failures. They can
also provide the right basis for an empirical claim. The boundary is that a
test does not change its authority when a stronger proof is missing. Honest
evidence is useful evidence precisely because its limits are recorded.

### What exists now

The current Orange implementation is deliberately narrow. The permanent Rust
compiler lineage provides source identities and UTF-8 byte spans, deterministic
lexing, stable diagnostic codes, and the `orangec` command-line boundary. The
Orange 2026 parser recognizes exactly one edition declaration followed by one
module. Legacy empty `spec` and `impl` functions remain valid. The accepted
S3a slice adds closed typed-literal specifications, the S3b slice, whose
specification is in the owner's review, adds pure functions over integers and
machine words, and the S3c slice, also in review, adds `let` bindings and
explicit `as` conversions:

PR #9 merged that bounded pre-alpha implementation and its normative records as
commit `6c0bd3021cf2df603e08808e4660724ca1e2b2a5`. The larger S3 milestone and
the D-004 architecture decision remain open. D-003 candidate PF-01 is accepted
through OEP-0004 at exact revision
`a82a5cec2ee4359dc2fe66171f17c93146747333`.

```orange
edition 2026;
module demo {
  spec identity() {}
  impl rounds() {}
  spec answer() -> Int { 42 }
  spec mask() -> Word[8] { 0xff }
  spec ch(x: Word[32], y: Word[32], z: Word[32]) -> Word[32] {
    (x & y) ^ (~x & z)
  }
  spec sample() -> Word[32] { ch(0x510e_527f, 0x9b05_688c, 0x1f83_d9ab) }
}
```

`orangec check` lexes, parses, and semantically validates that source. Function
names must be unique within separate `spec` and `impl` namespaces, so the two
kinds may share a spelling while a same-kind duplicate fails. Semantic type
acceptance is contextual and exact: `Int` denotes mathematical signed integers,
and `Word[n]`, for n of 8, 16, 32, or 64, denotes the integers modulo 2^n. A
word literal must already fit, with no wrapping, truncation, or coercion; word
arithmetic wraps because that is what arithmetic modulo 2^n means.

Successful typed specifications lower in source order to a bounded Typed
Reference Core. Running `orangec eval FILE` prints:

```text
demo::answer: Int = 42
demo::mask: Word[8] = 0xff
demo::sample: Word[32] = 0x1f85c98c
```

Empty declarations still have no type, value, or execution meaning, and a
function with parameters runs only when it is called. The Core is noncanonical
and carries no proof identity or relationship between a `spec` and an `impl`.
The fragment has no bindings, control flow, recursion, general failure values,
proof terms, targets, ABI rules, or code generation.
The reserved words `game`, `proof`, and `claim` still introduce no usable
constructs.

These absences are not disguised as a miniature finished language. The parser,
semantic analyzer, Core constructor, and evaluator are bounded components at
their intended incremental boundaries. Parse success means the source has the
recorded syntactic shape. Semantic and evaluation success means only that a
typed specification satisfied the rules and produced the displayed value. Neither result means the source is correct cryptography, a valid proof,
safe machine code, a refining implementation, or a generally executable
program.

This is the project's no-disposable-prototype rule in practice. Orange grows by
adding permanent components with explicit interfaces, deterministic behavior,
diagnostics, tests, and migration rules. The rule does not make the early system
large. It makes each small piece honest about where it belongs and what it can
show.

### The reader's habit

The central habit of this book is to ask one question whenever a strong sentence
appears: *what connects that sentence to the exact artifact under discussion?*

Sometimes the answer will be a directed project decision. Sometimes it will be
a normative rule and a conformance test. Later, it may be a proof term, a checked
translation certificate, an object-code inspection record, a standards source,
or an external validation with exact scope. Often, during pre-alpha development,
the answer will be that the connection is proposed or does not exist yet.

That last answer is not a defeat. An explicit gap is a tractable engineering
fact. A hidden gap is an unbounded trust claim.

Orange's first thesis is therefore simple: the path from intent to shipped bytes
must be part of the product. Its second thesis follows immediately: every claim
about that path must say what it covers, what supports it, and where it stops.

## Chapter 2: Claims, Not Labels

Security engineering has a vocabulary problem. Words such as *safe*,
*conformant*, *constant-time*, and *verified* sound like properties, but in
ordinary use they often behave like stickers. They are placed on a library,
package, or release after some valuable work has been done, then asked to carry
far more meaning than that work established.

The sticker may begin with a true statement. A team proved a functional theorem.
A laboratory ran a validation program. A test suite passed. A memory-safe
language rejected certain errors. A timing experiment found no signal. Trouble
begins when the subject, conditions, and evidence disappear, leaving only the
adjective. The reader can no longer tell whether *verified* means a parser
accepted the source, a proof kernel checked a theorem, a compiler preserved the
theorem, or a reviewer inspected the final object. The strongest available
interpretation tends to win, even when it is the least justified.

Orange's [proposed public assurance model](ASSURANCE.md#3-claim-model) replaces
that compression with a set of separately named claims.
[D-005](DECISIONS.md#d-005--public-assurance-model) has not yet been accepted,
so the complete claim taxonomy and product record format remain proposals. The
underlying discipline, however, already controls how the project describes its
present compiler: say exactly what happened, bind the statement to an artifact
and revision, name the evidence, and state what the result does not show.

### A claim is a proposition with coordinates

Consider the sentence, “the implementation is constant-time.” Before it can be
checked, almost every important noun in that sentence needs coordinates:

- Which implementation, source revision, exported symbol, and artifact bytes?
- For which inputs, preconditions, target, instruction set, and calling
  environment?
- What can the observer see: branches, addresses, instruction classes, timing,
  caches, speculation, power, or something else?
- Which compiler and transformations connect the reviewed program to the
  executed bytes?
- Which assumptions exclude behavior outside the model?
- What evidence supports the proposition, and which authority checked it?

Without those coordinates, the sentence may express an intention or a useful
engineering convention, but it does not yet identify one reviewable assurance
claim. Adding coordinates does not guarantee truth. It makes truth and error
arguable against the same subject.

This is why an Orange claim is intended to carry exact wording rather than only
a category name. The category says what kind of question is being asked. The
wording says which proposition must be supported. Its subject identifies a
definition, export, control set, or artifact by revision and digest. Its context
identifies such things as the language edition, toolchain, cryptographic
profile, target profile, and leakage model. Assumptions and exclusions mark the
edge of the statement instead of hiding beyond it.

The digest matters because names drift. A function called `encrypt` can change
while retaining its spelling. A standard profile can acquire errata. A compiler
flag can alter the generated object without altering the source. Human-friendly
names remain essential for reading, but a claim about exact bytes needs an
identity that changes when the bytes do.

### One artifact, several answers

A native cryptographic export does not have one assurance status. It presents a
matrix of questions whose answers may differ:

| Question | Example scope | Possible evidence |
| --- | --- | --- |
| Does it match a standard? | Named edition, profile, and input domain | Vectors, differential tests, or external validation |
| Does it realize a specification? | Named implementation and mathematical definition | Refinement proof or checked certificate |
| Does it execute safely? | Named faults, preconditions, and runtime model | Type argument, proof, analysis, and adversarial tests |
| Does it terminate? | Named inputs and environment assumptions | Variant or termination proof |
| What does it leak? | Named source or target observation model | Noninterference proof, translation evidence, and measurements |
| Did compilation preserve a property? | Exact passes, toolchain, target, and final bytes | Pass theorem or translation-validation certificate |
| Does the foreign boundary agree? | Named ABI, layout, alias, length, and error rules | Contract proof and adversarial caller tests |
| Was sensitive state erased? | Named object, lifecycle point, target, and architectural model | Erasure proof, binary analysis, and scoped measurements |
| Does it meet a security theorem? | Named game, advantage bound, and assumptions | Checked reduction or recorded external proof |
| What did empirical testing observe? | Named corpus, method, environment, and implementation/target | Vectors, differential tests, fuzzing, timing, or interoperability records |

The rows are related, but they are not interchangeable. Standard vectors can
expose a wrong answer without proving all answers. A source refinement theorem
can establish functional meaning without describing cache observations. A
leakage argument can hold for a faulty algorithm. An ABI wrapper can be
memory-safe while passing bytes in the wrong order. A security reduction can be
mathematically sound while the shipped implementation fails to realize the
construction it studies.

The purpose of a claim matrix is not to demand that every row be satisfied
before any work is useful. It is to prevent a result in one row from silently
coloring all the others. A small library with three narrow, well-supported
claims is easier to reason about than one broadly advertised as verified.

### Four outcomes, not one light

The proposed model gives each claim one of four outcomes: `satisfied`,
`not_satisfied`, `unresolved`, or `unsupported`. These are not grades on a
single scale.

`satisfied` means the claim has a basis that its policy permits and that basis
is valid for the recorded subject. It does not mean adjacent claims are
satisfied. A successful vector claim, for example, remains a successful vector
claim rather than becoming functional correctness for all inputs.

`not_satisfied` means the proposition was checked far enough to obtain a
negative result. A counterexample, failed certificate, mismatched vector, or
violated contract may justify this outcome. It is evidence about the claim, not
merely the absence of success.

`unresolved` means the system cannot presently decide the proposition. Proof
search may time out. A solver may return `unknown`. Required evidence may be
incomplete. An open design question may prevent the claim from being stated
precisely. Turning any of these conditions into success would confuse a search
procedure with an authority.

`unsupported` means the selected toolchain, model, target, operating mode, or
project capability does not offer the claim. This outcome is especially
important for honest partial systems. Orange currently has no native target
model, leakage semantics, ABI, proof checker, or release path. Claims that
depend on those facilities are not weakly satisfied by the frontend test suite;
they are outside the implemented support envelope.

The distinction affects action. A `not_satisfied` claim points toward a defect
or a false proposition. An `unresolved` claim may need more evidence, a smaller
statement, or a better procedure. An `unsupported` claim may require an explicit
product decision and new machinery. Collapsing all three into a red light loses
that information. Collapsing them into “not yet verified” is gentler wording but
has the same defect.

### Evidence keeps its authority

Evidence is useful because of what it can support, not because it can be counted.
A thousand passing tests do not add up to a proof for every input. A proof does
not become an external validation because two people read it. A reproducible
build does not become a correctness result because the reproduced bytes are
stable. Each basis retains its type and authority.

Orange's proposed records distinguish kernel proofs, checked certificates,
external proofs, test runs, audits, external validations, and assumptions. A
claim policy determines which kinds may close which claim and which complete
set is mandatory. More than one basis can support the same proposition: a
refinement claim may have a checked proof, tests that catch regressions, and an
external review record. A favorable optional basis cannot mask a failed,
expired, unavailable, missing, or mismatched mandatory basis, context, trust
component, or composition edge. Those bases can reinforce confidence and
diagnose different failures without pretending to be the same thing.

This separation is most visible around automation. A solver is excellent at
searching for proofs or counterexamples. If a proof-required claim depends on a
small checker, the solver's success must arrive as a certificate or proof object
that the checker accepts. A timeout is unresolved, not false. An unsupported
certificate step is unresolved, not true. The search tool can be large and
heuristic while the acceptance path remains small and explicit.

External authority also stays external. A laboratory certificate, audit, or
standards-body record may be exactly the right basis for a validation claim.
Recording its issuer, scope, subject, dates, and digest makes it auditable; it
does not transform that record into an Orange theorem. Conversely, a local
theorem does not impersonate a certification program with legal and procedural
requirements that the project has not performed.

### Assumptions are dependencies

Every serious claim rests on something it does not prove internally. A logical
kernel is trusted to implement its rules. A target model is assumed to describe
the processor behavior relevant to the property. A caller may be required to
provide nonoverlapping buffers of sufficient length. An operating system may be
trusted to supply pages with stated behavior. A cryptographic theorem may rely
on a named hardness assumption.

Calling these items assumptions should not make them vague. The useful form is
specific: what is assumed, why the claim needs it, and what happens if it is
false. This turns an assumption into a visible dependency in the claim closure.
Different claims over the same artifact can then have different trust bases. A
mathematical equality need not inherit the operating system assumptions of a
runtime erasure claim. A vector result need not pretend that a proof kernel was
involved.

Exclusions serve a related purpose. They state tempting interpretations that
the wording does not cover. A source-level address-trace result might exclude
power analysis, speculation, and the behavior of the final machine code. A
repository-control result might exclude compiler correctness and release
readiness. Exclusions do not repair an overbroad claim, but they help keep a
narrow one from expanding as it travels.

### Composition must be earned

The most important claims usually cross boundaries. To say that shipped object
bytes implement a specification, it is not enough to have a source proof and an
object file. The graph needs justified edges through lowering, optimization,
assembly, linking, and the foreign boundary as applicable. Each edge may use a
general preservation theorem, a per-artifact checked certificate, or another
explicit authority allowed by policy.

This creates a useful failure rule: a claim does not jump over a missing edge.
If source code refines a specification but the backend has no preservation
argument, the source claim remains available and the final-byte claim remains
unresolved or unsupported. Nothing has been taken away from the source theorem.
The system has simply declined to lend it to a different subject.

Composition also runs in the other direction. A broad release statement should
be decomposable into the narrower propositions on which it depends. A reader
ought to be able to ask why a property is reported, inspect the basis, enumerate
its assumptions, and follow identities back to exact artifacts. The eventual
Orange trust report is intended to print that closure, not a marketing summary.

### The provisional record and the current compiler

The repository contains a provisional Gate 0
[claim-record schema](../schemas/gate0/claim-record-v0.1.schema.json) and
[conformance fixtures](../conformance/foundation/README.md). They demonstrate
structural ideas: an exact subject, identified or inapplicable contexts,
assumptions, exclusions, evidence references, a typed basis, an outcome, and a
review policy. They are explicitly non-product records with synthetic fixture
data. D-005 remains proposed, so these files are not the stable public claim
format and do not authorize future syntax or assurance behavior.

The current compiler makes much smaller observations. At its recorded revisions,
the tests show that the implemented lexer, parser, S3a semantic analyzer, Typed
Reference Core constructor, evaluator, diagnostics, resource limits, and CLI
behave as asserted by those tests. The normative documents define the accepted
surface and typed-literal behavior. The conformance index maps each current S3a
rule to named executable evidence, while warning that a named test need not
exhaust its rule.

That is worthwhile evidence for a pre-alpha compiler slice. It is not a claim
that Orange has proved its semantics, verified the Rust implementation, produced
cryptography, preserved a property into machine code, met a leakage model,
passed independent review, or created a release. The honest statement is longer
than “verified,” but it is also more useful: a reader can see what is present,
what remains open, and which next result would actually change the picture.

The discipline of claims is therefore not paperwork added after verification.
It is the interface between technical work and public meaning. A proof, test,
build, review, and certificate become safer to reuse when none is forced to
masquerade as the others. Orange's ambition is not to make every box green. It
is to make every box precise enough that green, red, unresolved, and unsupported
each tell the truth.

## Chapter 3: One Language, Several Semantic Worlds

A cryptographer who writes down a hash function is doing several different
things at once, usually without naming them. There is a mathematical object: a
function from byte strings to fixed-length digests, defined by compression
rounds over words of a stated width. There is an algorithm that computes it,
with buffers, padding, and a streaming state. There may be a fast variant that
uses vector registers or a dedicated instruction. There is a security notion,
such as collision resistance, stated as a game against an adversary. And there
are arguments connecting these, some mechanical and some mathematical.

Each of those things has a natural way of being true. A mathematical function
is true when it is total and well defined. A stateful algorithm is correct when
every execution computes the function and never reads outside its buffers. A
vectorized routine is correct when its lanes, shuffles, and target features do
what the scalar algorithm does. A game is meaningful when its probabilities and
oracles are defined, and a reduction is sound when its advantage bound follows.
These are different kinds of meaning. Orange's language design begins by
refusing to pretend otherwise.

### Why a standalone language

The first question is whether Orange should be a language at all. Excellent
alternatives exist. A project could orchestrate existing tools: write the
specification in Cryptol or hacspec, the fast code in Jasmin, the security
argument in EasyCrypt, and connect them with a manifest. It could embed a
cryptographic domain-specific language inside a proof assistant such as F\*,
Lean, or Rocq, inheriting a mature kernel and library. It could accept a subset
of Rust with proof annotations and meet implementers where they already work.

Orange examined those options as candidates rather than dismissing them. The
[product-form decision](DECISIONS.md#d-003--product-form) compared four forms
against eight hard gates and accepted candidate PF-01, a standalone
domain-specific language with its own editioned semantics and canonical Core
formats. That decision is **current and accepted**: the project owner accepted
it on 2026-07-26 and
[OEP-0004](governance/oeps/OEP-0004-standalone-orange-product-form.md) binds it
to exact revision `a82a5cec2ee4359dc2fe66171f17c93146747333`.

The reasoning is the seam argument from Chapter 1 turned on the language
itself. If the surface language belongs to a host prover, every upgrade of that
prover becomes an upgrade of Orange's meaning, and ordinary cryptographic code
inherits proof-engine conventions it does not need. If Orange is only an
orchestrator, the manifest quietly becomes the real language, and the crossings
between tools remain the least specified part of the system. Interoperability
with those systems is still valuable, and later chapters return to it. But
delegating the language would preserve exactly the gaps Orange exists to close.

Accepting a standalone form does not accept a semantics. PF-01 says Orange owns
its meaning; it does not say what that meaning is. The rest of this chapter
describes the leading proposal and the discipline that any answer must obey.

### Five roles, one module system

The [project charter](PROJECT_CHARTER.md#4-product-thesis) proposes one module
system with several deliberately separated declaration roles:

- **Specification** for total mathematical functions and relations;
- **Implementation** for terminating, memory-safe executable procedures with
  contracts and typed failure;
- **Machine implementation** for explicit layout, vector operations, target
  intrinsics, and leakage-aware control flow;
- **Game** for probabilistic programs, adversary interfaces, and
  reduction-based security claims; and
- **Proof** for refinements, invariants, equivalences, noninterference, and
  security reductions.

The roles share names and types where sharing is sound. A specification of
SHA-256 and its implementation can live side by side in one module and be read
together. What the roles must not share is a single set of operational rules.
Mathematical integers do not overflow; machine words do. A pure function cannot
sample randomness; a game must. A proof may mention ghost values that must
never influence a running program. A vector intrinsic has meaning only on a
target that provides it.

A **claim** is conspicuously absent from that list. In the
[semantic-strata proposal](SEMANTIC_STRATA_DECISION_SUITE.md#31-source-declaration-roles),
a claim is a record that binds a subject, a relation, assumptions, and evidence.
It is not a sixth semantic world with its own execution rules. Foreign imports
and deliberate declassification are similar: they are cross-cutting boundaries
that must be declared, not annotations that switch off a stratum's rules.

All of this is **proposed**. The role map is a hypothesis under
[D-004](DECISIONS.md#d-004--semantic-strata), which remains open.

### The crossings are the design

If the roles were the whole story, Orange would be five small languages sharing
a parser. The interesting part is how meaning moves between them. The D-004
suite names fourteen required crossings, from `SR-01` through `SR-14`, and
requires every candidate architecture to express each one with a versioned
name, domain, codomain, definedness conditions, obligations, identity inputs,
trust role, failure behavior, and prohibited reverse inferences.

A few of those crossings show the shape of the problem:

| Crossing | What it must preserve |
| --- | --- |
| Specification source to Spec Core | Either one checked pure subject or failure without creating an identity |
| Implementation to specification | A named refinement obligation between explicit subjects, never inferred from equal names |
| Implementation to CT IR | Runtime meaning through ghost erasure and lowering, or invalidation of the dependent result |
| Game to game | A named reduction that preserves its exact bound expression |
| Claim record to subject and evidence | No upgrade of a failed, missing, unknown, or unsupported relation |

The suite also fixes a short list of invariants that every candidate must
respect. They read almost like a style guide for honesty:

- a shared source name never creates a refinement relation;
- sampling cannot enter a specification or implementation through a pure
  embedding;
- state, memory, target, and ambient effects cannot enter the pure Core;
- proof or ghost data cannot affect runtime behavior;
- machine-level source cannot bypass the checked low-level boundary;
- byte or format conversion is not semantic preservation; and
- a failed crossing invalidates its dependent result rather than producing a
  generic lower assurance level.

The first rule deserves attention because it is so tempting to break. A module
that contains `spec sha256` and `impl sha256` looks, to a human reader, as if
the second implements the first. Orange's proposal is that the matching names
mean nothing semantically. Refinement is a relation that must be stated,
checked, and bound to exact subjects. The reader's intuition is a good reason
to write the refinement obligation; it is not evidence that the obligation
holds.

### Five candidate architectures

The research recommendation behind the charter is a role-oriented family of
formally related Cores: a Spec Core, an Impl Core, and a Game Core as the
normative program semantics; CT IR and Machine IR as compilation boundaries;
and a proof-evidence interface that names judgments without choosing a proof
calculus. A small **Shared Pure** subset would let deterministic definitions be
reused across roles without importing state or randomness.

That recommendation is not a selection. The
[D-004 suite](SEMANTIC_STRATA_DECISION_SUITE.md#2-candidate-architectures)
compares it symmetrically with four alternatives:

| ID | Candidate | Idea |
| --- | --- | --- |
| ST-REL | Role-oriented related family | Separate Cores per role with named, checked relations |
| ST-UNI | Universal Core | One effect-parameterized calculus for everything |
| ST-DUAL | Pure/effect pair | One pure Core plus one general effect Core |
| ST-MIRROR | Five mirrored Cores | One Core per source role, joined by crossings |
| ST-HOST | Host-delegated strata | Local deterministic semantics; games, proofs, or machine meaning delegated to external systems |

The universal Core deserves a fair hearing. One calculus is easier to specify
once, easier to implement once, and avoids a family of translations. The
concern recorded in the decision register is that mathematical totality,
probabilistic games, stateful memory, target leakage, and concrete instructions
make conflicting demands, and that hiding those conflicts inside effect
annotations could make the semantics less honest rather than more. That is a
hypothesis to test against five fixed cases: SHA-like word code, mutable-buffer
refinement, a secret-dependent rejection, one vector intrinsic, and one game
with a reduction. None of the 25 candidate-case executions has been run.

### What exists in the language now

The current language shows only the outline of this design. Orange 2026
reserves `spec` and `impl` as declaration keywords and gives them separate
namespaces, so a module may contain both `spec rounds` and `impl rounds`
without a conflict while two `spec rounds` declarations are an error. The words
`game`, `proof`, and `claim` are reserved and introduce nothing. Only typed
specifications have meaning: pure `spec` functions over `Int` and `Word[8]`
through `Word[64]`, built from literals, parameters, calls, operators, `let`
bindings, and explicit conversions. An `impl` body must still be empty.

Even that small surface already follows the chapter's rules. `Int` and each
word width are distinct types, and a value moves between them only through a
written `as`, never implicitly. A same-named
`spec` and `impl` have no relation. Nothing in the Typed Reference Core
pretends to be a Spec Core, and the Core records no claim. The expression and
binding slices were built to fit inside every candidate's specification
stratum: they are pure, total, and deterministic, so the strata decision can
place them without changing a line of source.

### Beauty as a constraint

Orange is written for cryptographers, cryptologists, and cryptanalysts, people
who read mathematics for a living. That audience sets a high bar for how the
language should look. A specification written in Orange ought to read like the
definition in a good paper: rotations where the standard rotates, addition
modulo a word size where the standard adds modulo a word size, and nothing else
in the way. The separation of worlds is what makes that possible. When the
specification stratum only has to be mathematics, it can look like
mathematics. The mess of buffers, targets, and timing belongs to the strata
built to carry it, where it can be stated precisely instead of leaking into
every definition.

The goal, then, is not five languages bolted together. It is one language
whose parts are honest about the kind of truth they express, so that the
places where those truths meet can be written down, checked, and read.

## Chapter 4: From Surface Text to Meaning

Source code is text before it is anything else, and text is easy to
misunderstand. The characters on the screen are not the bytes in the file, the
bytes are not the tokens, the tokens are not the program's structure, and the
structure is not its meaning. A compiler for high-assurance cryptography has to
keep each of those steps explicit, because each one is a place where two tools,
or two readers, can disagree about what was written.

This chapter follows one small Orange program from bytes to value. Everything
in the walk-through is **current**: it describes what the `orangec` compiler in
this repository does today, under the normative
[lexical and grammar specification](LANGUAGE_2026.md), the accepted
[typed-literal semantics](SEMANTICS_2026.md), and the
[pure expression specification](EXPRESSIONS_2026.md) now in the owner's
review. The last part of the chapter
turns to what the complete semantic Core is meant to become, which remains
open.

### Bytes with an identity

An Orange 2026 source must be valid UTF-8 and at most 16 MiB. Before any
lexing, the compiler gives the source an identity and treats every later
position as a half-open byte range `[start, end)` into that source. Line and
column numbers are computed for people, with one-based columns counted in
Unicode scalar values, but the permanent coordinates are bytes.

That choice is small and deliberate. Byte spans are unambiguous across editors,
operating systems, and normalization forms. A line feed, a carriage-return line
feed, and a bare carriage return each count as exactly one logical line ending,
so a file edited on two systems does not report two different line numbers for
the same error. Only four characters are whitespace: tab, line feed, carriage
return, and space. A non-breaking space is an error, not an invisible
separator. A reviewer who sees two identical-looking programs should not have
to wonder whether one contains a character that changes where a token ends.

### Tokens

Lexing turns bytes into a deterministic sequence of tokens, each carrying its
exact span. `orangec lex` prints that sequence:

```text
78..85   KW_EDITION   "edition"
86..90   INTEGER      "2026"
90..91   SEMICOLON    ";"
92..98   KW_MODULE    "module"
99..103  IDENTIFIER   "demo"
```

The Orange 2026 lexer recognizes ASCII identifiers, seven reserved words,
decimal, binary, and hexadecimal integers with single underscores between
digits, line-bounded strings with a fixed escape set, nested block comments,
and a fixed inventory of punctuation, matched longest first so that `<<<` is
one rotation token rather than a shift and a comparison. It reserves more than
the grammar uses: strings and several punctuation tokens have no grammatical
role yet. Reservation is a promise about spelling, not about meaning.

The lexer is also bounded. It retains at most 262,144 non-trivia tokens and
emits at most 100 ordinary diagnostics before one suppression diagnostic. A
lexically invalid source is never parsed. That rule keeps error reports honest:
a malformed integer should produce a lexical diagnostic, not a cascade of
confusing parse errors downstream of a token that should never have existed.

### Structure

Parsing checks that the tokens have one of a small number of shapes. The
parser reads them in order and never backtracks: one token of lookahead decides
almost everything, and a second is consulted only to tell a call from a name
and a literal's sign from negation. A source is exactly one edition
declaration, `edition 2026;`, followed by exactly one module. A module contains
`spec` and `impl` declarations. An `impl` has an empty parameter list and an
empty body. A `spec` body may be empty, or the `spec` may declare parameters
and a result type and contain exactly one expression.

Parsing produces a syntax tree that records spelling and source structure
only. It is easy to overlook what that excludes. The grammar accepts any
identifier as a type and any integer as a width, so `spec x() -> Word[12] { 1 }`
and `spec y() -> Banana { 7 }` both parse. The parser also accepts two
declarations with the same name, because deciding whether names collide is not
a syntactic question. Parse success means only that the source has a recorded
shape. It is not validation, and the specification forbids describing it as
such.

### Meaning

Semantic analysis is where the program first acquires meaning, and in the
current slices that meaning is intentionally small. The analyzer works through
the syntax tree in source order and does five things:

1. It checks that every declaration key is unique, where a key is the pair of
   declaration kind and exact name. `spec mix` and `impl mix` are different
   keys; two `spec mix` declarations are a duplicate.
2. It resolves each typed specification's signature. Exactly five type forms
   are accepted: `Int`, with no width, and `Word[8]`, `Word[16]`, `Word[32]`,
   and `Word[64]`, with the width written as a plain decimal token.
   `Word[08]`, `Word[0x8]`, `Word[12]`, `Int[8]`, and every other form are
   errors. Parameter names must be distinct within one function.
3. It checks each body against its declared result type, from the root of the
   expression down. Every expression has an expected type and nothing is
   inferred: a literal takes the type expected of it and must fit, a name must
   be a parameter of that type, a call must name a typed `spec` whose result is
   that type, and an operator must be defined on it. Literals are decoded
   exactly, in base 2, 10, or 16.
4. It checks that the call graph is acyclic, so that every accepted program
   terminates.
5. If no semantic diagnostic occurred, it constructs the Typed Reference Core.

The types are where the language's character first shows. `Int` is the
type of mathematical integers. It has no maximum and does not overflow. The
analyzer does limit the size of a literal's magnitude to 16,384 significant
bits, but that limit is a boundary on source representation, not a secret width
for `Int`. `Word[8]` is the type of eight-bit machine words, with values from
0 through 255, and the wider words follow the same rule at their own widths. A
negative word literal is an error even when it is `-0`, and 256 is an error
rather than zero:

```text
error[ORC0207]: literal is outside the range of `Word[8]`
 --> compiler/fixtures/s3a/invalid-word-range.or:5:31
  |
5 |   spec decimal() -> Word[8] { 256 }
  |                               ^^^ expected a value from 0 through 255
  = note: fixed-width words do not truncate or wrap out-of-range integers
```

No literal wraps, truncates, saturates, or coerces, and no value ever changes
type. Every mature cryptographic codebase has at least one bug that came from
an integer silently changing its width or sign. Orange's first semantic rule is
that such changes are never silent.

Arithmetic on words is the one place where values do wrap, and there wrapping
is the meaning rather than an accident. `Word[32]` is not a bounded integer
that overflows; it is the ring of integers modulo 2^32, the structure SHA-256
and ChaCha20 are written over. `a + b` on words is addition in that ring, `~a`
is the complement, and `a <<< 7` is the rotation: the word operations of
FIPS 180-4 and RFC 8439, defined the way those documents define them. `Int` has the operators that make sense
for mathematical integers, `+`, `-`, `*`, and negation, with their exact
meaning. The bitwise operators are not defined on `Int`, and using one is an
error rather than a guess about representation.

### The Typed Reference Core

A successful analysis produces one Typed Reference Core module. Its grammar is
short enough to quote whole:

```text
core_module    = module_name core_function* ;
core_function  = function_id function_name parameter_type* core_type body ;
core_type      = Int | Word8 | Word16 | Word32 | Word64 ;
body           = core_node+ ;
core_node      = core_type node_kind ;
node_kind      = literal value
               | parameter index
               | call function_id argument_count
               | unary (negate | complement)
               | binary (add | subtract | multiply | and | or | xor)
               | shift (shl | shr | rotl | rotr) amount ;
```

Only typed specifications enter the Core. Empty `spec` and `impl` declarations
remain valid syntax but gain no type, value, or execution meaning, and they do
not appear. Functions keep source order and receive contiguous identifiers from
zero. A literal written `-0x2a` becomes the mathematical integer −42, and every
spelling of negative zero becomes zero. A body is stored in postorder, each
node after its operands and each call after its arguments, and every node
carries its type. Parentheses leave no trace, because grouping is already the
shape of the tree.

The Core is bounded in the same spirit as the lexer and parser: at most
262,144 Core nodes, 1,048,576 semantic events, and 100 ordinary semantic
diagnostics. Exhausting a budget fails closed with a stable resource
diagnostic. There is no partial Core. An error in one declaration does not
authorize the others; the analyzer may keep going to report more errors, but
the result is unsuccessful.

Evaluation is the last step. `orangec eval` visits each Core function in order
and prints one line for each function without parameters, decimal for `Int`
and fixed-width lowercase hexadecimal for words, from two digits for `Word[8]`
to sixteen for `Word[64]`:

```text
demo::answer: Int = 42
demo::negative: Int = -42
demo::mask: Word[8] = 0xff
```

That output format is precise down to its bytes, including the absence of a
plus sign and the leading zero in `0x0a`. The precision is not decoration. A
reference evaluator is useful only if another implementation can be compared
against it byte for byte.

Evaluation is bounded too. Every function of one source shares a budget of
1,048,576 steps, the call stack holds at most 256 frames, and no `Int` result
may exceed 16,384 significant bits. An acyclic program can still ask for an
exponential amount of work, a function that calls another twice, twenty levels
deep; the step budget is what stops it, with a diagnostic rather than a hang.

### What the Core is not

The Typed Reference Core is an internal compiler boundary. It has no canonical
encoding, serialization, content digest, theorem fingerprint, proof identity,
refinement relation, or promise that identifiers stay stable across
revisions. It is noncanonical on purpose. Freezing an encoding before the
semantic strata are decided would make a proof-bearing choice by accident,
which is exactly the kind of hidden decision the project forbids.

The **proposed** end state is a family of canonical Cores: a Spec Core for pure
mathematics, an Impl Core for stateful procedures with contracts and typed
failure, a Game Core for probabilistic experiments, and a Proof IR checked by
a small authoritative checker. A canonical Core would have a deterministic
encoding, so that two tools, or two revisions, can agree on exactly which
definition a theorem is about. The
[architecture](ARCHITECTURE.md#4-core-semantic-family) describes those
proposals in detail; [D-004](DECISIONS.md#d-004--semantic-strata) decides their
number and relationships.

### The next steps of meaning

The three current slices complete bounded parts of the roadmap's S3 stage:
literals first, then pure expressions with parameters, calls, and operators
over integers and words, then `let` bindings and explicit conversions. The
rest of S3 adds the remaining substance of a language: tuples or records,
comparisons and control flow, and explicit failure semantics, together with
one conformance case per normative rule. Each addition follows the same
pattern as the slices before it: a normative rule, a diagnostic for
every way to break it, a bound on the work it can cause, and a reference result
that can be printed and compared.

Meaning is where Orange's promises start to cost something. A lexer can be
made deterministic with care. A semantics has to be right about arithmetic,
about failure, and about what a name refers to, and every later proof inherits
whatever it gets wrong. That is why the first semantic slice is small, and why
each later slice is meant to be small enough to state completely.

## Chapter 5: Proof Search Is Not Proof Checking

Modern verification tools are astonishingly good at finding proofs. SAT
solvers settle bit-level equivalences over millions of clauses. SMT solvers
discharge the routine arithmetic side conditions that once filled pages of
hand-written lemmas. Tactic languages turn a two-line user hint into a large
derivation. For a language that wants cryptographers to state and prove real
properties without becoming full-time proof engineers, that automation is
indispensable.

It is also the wrong thing to trust. A search procedure is large, heuristic,
fast-moving, and optimized for finding answers rather than for being right
about them. The question Orange asks is not whether automation should be used.
It is where authority lives once automation has done its work.

### Two jobs, two sizes

Finding a proof and checking a proof are different jobs with different natural
sizes. Search explores a vast space with clever strategies, caches, and
heuristics; its code grows as it gets better. Checking confirms that a
completed derivation follows the rules of a fixed logic; its code can stay
small, because it never has to be clever.

The project's charter turns that asymmetry into a principle: *soundness before
automation*. Solvers are search engines. Their answer is accepted only through a
checked certificate or through an explicitly disclosed external-trust claim.
The [research analysis](RESEARCH.md#43-the-solver-should-search-not-legislate)
states the same idea more bluntly: the solver should search, not legislate.

F\* is a useful point of comparison because it is both successful and candid.
Its SMT automation removes a great deal of routine proof burden, and its
documentation is clear that the combination of F\* and Z3 is trusted for those
results. That is a legitimate design with a known trusted base. Lean and Rocq
take the other path: tactics may do anything they like, but the final proof
term is checked by a small kernel. Orange's **proposal** is to follow the second
path for claims that require proof, while treating any exception as a visible,
named part of the trusted base rather than a silent convenience.

### The proposed checker

The [architecture](ARCHITECTURE.md#22-orange-check) describes an authoritative
offline checker, `orange-check`, with a deliberately austere feature list. It
has no network access, no package resolution, no tactics, no plugins, and no
code generation. Its behavior is deterministic and resource bounded. It
supports one explicit set of format versions, and it can print every axiom and
trusted model in a claim's closure.

The checker would read **Orange Proof IR**, a stable evidence language of fully
elaborated terms with explicit universe and type arguments, references by
theorem fingerprint, and no tactic syntax. The proposed requirements read like
a security specification because they are one: canonical deterministic
encoding, streaming and bounded validation, stable rejection of malformed
input, no unknown fields in a security-critical version, no cyclic or
exponential expansion, and theorem fingerprints that include definitions,
axioms, semantics edition, and relevant target model.

That last requirement matters more than it first appears. A theorem about
`sha256` is only useful if it is about the `sha256` being shipped. A
fingerprint that changes whenever the definition, axioms, semantic edition, or
target model changes is what keeps a proof from quietly migrating to a
different subject.

[D-007](DECISIONS.md#d-007--orange-owned-proof-format-and-checker) proposes that
Orange own this proof format and checker rather than making a host prover's
compiled environment the permanent public artifact. The register is plain about
the risk: a custom kernel is a major soundness and schedule risk, and its logic
must be smaller than the surface language. The recommended mitigation is to
specify the checker in a proof assistant, prove it sound there, distribute an
extracted authoritative checker, and maintain an implementation-diverse checker
in safe Rust for differential testing. Because both would come from the same
owner, the second checker is described as implementation-diverse, never as
independent.

### Choosing a foundation

The logic that the checker implements has to be defined and proved sound in
something. [D-006](DECISIONS.md#d-006--proof-foundation) compares two
candidates, Rocq and Lean 4, and selects neither. Rocq brings the closest
existing ecosystem of verified compilers, cryptographic synthesis, and
extraction. Lean 4 brings an integrated implementation, kernel, and tooling
model. The register treats both strengths as hypotheses to measure, not reasons
to preselect a winner.

The [proof-foundation suite](PROOF_FOUNDATION_DECISION_SUITE.md) asks each
candidate to do the same concrete work: define and check a proposed Core
fragment, mechanize progress and preservation plus a leakage lemma, validate a
canonical serialization, produce and replay an LRAT-backed bit-vector proof,
distribute the checker on supported hosts, and survive the same seeded
maintenance tasks. The draft defines 14 candidate-case runs. None has been
executed. D-006 is marked **investigate**, and no proof toolchain is admitted.

### What counts as a checked answer

Automation produces many kinds of output, and only some of them should be able
to close a claim. The [proposed automation portfolio](ARCHITECTURE.md#7-proof-automation)
sorts them:

- **Bit-vector and finite equivalence** goes through verified bit-blasting to
  SAT, with an LRAT-family certificate required for a claim-closing success.
  Counterexamples are decoded back into source values so that a failure is
  something a cryptographer can read.
- **Ring and field identities, modular reasoning, and ranges** use
  kernel-checked reflective procedures, which compute inside the logic and
  therefore need no external trust.
- **Supported SMT fragments** use proof-producing solvers with a ratified
  certificate format such as Alethe, checked before acceptance.
- **Quantified and inductive properties** use explicit induction and user
  lemmas.
- **External proofs** from EasyCrypt or SSProve remain labeled external until
  their evidence is reconstructed in Orange's own format.

Everything else is search. A timeout, an `unknown`, resource exhaustion,
missing proof output, or a certificate that fails to check leaves the claim
**unresolved**, with the precise reason recorded. Developer profiles may
display a solver's unchecked opinion, but that opinion cannot satisfy a claim
or be cached under a status that suggests it did.

### Three policies for solver trust

How strict should that rule be? [D-009](DECISIONS.md#d-009--solver-trust) frames
the choice as three candidates, compared symmetrically in the
[solver-trust suite](SOLVER_TRUST_DECISION_SUITE.md):

| ID | Policy | Where authority lives |
| --- | --- | --- |
| SP-01 | Checked-artifact portfolio | An accepted certificate or Orange proof term |
| SP-02 | Kernel-only reconstruction | Only a kernel-accepted Orange proof term |
| SP-03 | Direct trusted-solver authority | An exact admitted solver, version, and fragment, listed in the logical trusted base |

SP-03 is not a strawman. Some organizations deliberately trust a particular
solver version for a narrow fragment because the alternative is unaffordable.
What the suite forbids is the confusion of categories: a direct solver result
may never be presented as a checked certificate or kernel proof. The frozen
matrix has 24 candidate-case runs across eight cases; none has been executed,
and no policy is selected.

### Caching without laundering

Proof replay is expensive, so real systems cache. A cache is also a place where
an old answer can quietly survive a change that should have invalidated it. The
proposed cache key for a proof result therefore includes the normalized
obligation, imported theorem fingerprints, source and core-semantics editions,
checker and decision-procedure versions, and the target and leakage policy
where relevant. A separate search cache may key on the solver binary, its
arguments, seed, and limits, but the certificate it hopes to find cannot be
part of the key for the result being searched for. The command-line tools are
meant to explain which component invalidated a cached proof, because an
unexplained cache miss teaches nothing and an unexplained hit should worry
everyone.

### Where things stand

Today Orange has no proof syntax, no Proof IR, no checker, and no admitted
solver. The word `proof` is reserved and does nothing. The current compiler's
semantic analyzer and evaluator are engineering trust dependencies. No logical
checker exists, so they are not outside any trusted base by virtue of being
checked, and their test results are implementation evidence, not proofs.

That emptiness is a feature of the order of work, not an oversight. Proof
components are gated on decisions that are still open, and the proof-neutral
frontend has been built first because it does not depend on them. When proofs
do arrive, one principle is already directed: search wherever it helps, and
accept a solver's answer only through a checked certificate or an explicitly
disclosed external-trust claim. Which checker and which solver policy carry
that authority remain open under D-006, D-007, and D-009.

## Chapter 6: Secrets Are a Semantic Concern

Side channels are the part of cryptographic engineering that most resists
being written down. A routine can compute exactly the right answer and still
betray its key through the time it takes, the cache lines it touches, or the
branch it chooses. Engineers have developed good habits in response: avoid
secret-dependent branches, avoid secret-dependent table lookups, prefer
arithmetic masks to conditionals. In most languages those habits live in
comments, code review, and the memory of whoever wrote the routine.

Orange's charter takes the opposite position in one short sentence: *secrets
are a semantic concern*. Secrecy labels are meant to affect typing, permitted
control flow and addresses, leakage traces, diagnostics, and review of the
foreign interface. They are not comments. This chapter describes what that
would mean and why the details are still open.

### What "constant time" actually says

The phrase *constant time* is itself a label of the kind Chapter 2 warned
about. Literally, it is false for almost all real code, which takes different
amounts of time on different machines and different days. What engineers mean
is narrower and more useful: what an attacker can observe does not depend on
the secret.

That can be made precise as **two-run noninterference**. Take two executions
with the same public inputs and possibly different secret inputs. Record what
an observer could see in each: the sequence of branch decisions, the memory
addresses accessed, and so on. The program satisfies the property if the two
observation traces are always equal. A difference in the secret produces no
difference the observer can detect.

The definition immediately raises the question the label hides: *what does the
observer see?* The answer is a model, and different models make different
claims. Orange's [assurance model](ASSURANCE.md#41-baseline-model) proposes a
baseline observation trace containing at least:

- branch decisions and targets;
- memory addresses, widths, and access classes;
- indirect call and return targets;
- traps, exceptions, and termination; and
- use of instructions classified as variable-latency on the target.

A claim under that model names its public-input relation and any permitted
declassification. It says nothing about power consumption, electromagnetic
emanation, faults, speculation, or unspecified microarchitectural behavior.
Those are excluded unless a separate profile models them.

### Versioned policies instead of a Boolean

Because the observation model is part of the claim, Orange proposes to replace
a single `constant_time` flag with versioned leakage policies. The
[architecture](ARCHITECTURE.md#45-ct-ir) sketches a family:

| Policy | What it constrains |
| --- | --- |
| `ct-architectural-v1` | Branches, call targets, memory addresses and widths, traps, and termination are secret-independent |
| `ct-variable-latency-v1` | Additionally keeps secret operands away from instructions the target profile classifies as variable-latency |
| `ct-speculative-v1` | Uses a named speculative-execution model or a proved hardening transformation |

Later profiles could tie claims to documented hardware modes such as
data-independent timing. Masked implementations, power, electromagnetic
emanation, and fault resistance would each need their own semantics, target
assumptions, and evidence. None extends the baseline by implication. A reader
who sees `ct-architectural-v1` satisfied learns exactly that, and nothing about
Spectre.

### Secrecy in the type system

For those claims to be checkable, secrecy has to be visible to the compiler.
The proposal is for public and secret to be semantic labels on values and
types. A branch on a secret value, or an array index computed from one, would
be a type error in a claim-bearing kernel rather than a lint. Secret values
would be non-copy by default, so every duplicate is explicit and tracked.
Formatting and debug output would refuse to print them. The foreign-interface
contract would record which arguments carry secrets.

Deliberate release of secret-derived information is sometimes necessary. A MAC
comparison eventually reveals whether the tag matched; a rejection-sampling
loop reveals how many attempts it took. The proposal is that every such release
names a declassification policy, visible in the claim graph, rather than an
annotation that switches the checker off. The difference is between a program
that says "this bit is intentionally public, under this policy" and one that
says "trust me here."

Erasure follows the same discipline. An `erase` obligation, or a zeroization
claim, concerns architecturally modeled storage and the copies the compiler
itself creates. It does not promise physical destruction, cache clearing, or
resistance to remanence unless a stronger target policy says so.

### A selection, two ways

Consider choosing between two words, `x` and `y`, according to a secret bit
`b`. The obvious code branches: if `b` is one, return `x`, otherwise return
`y`. Under the baseline observation model the branch decision is part of the
trace, so two runs that differ only in `b` produce different traces. The
property fails, and a type system that knows `b` is secret can reject the
branch where it is written.

The familiar alternative computes a mask. Negating the bit in w-bit
two's-complement arithmetic, that is modulo 2^w, gives either all ones or all
zeros, and the result is `(x AND mask) OR (y AND
NOT mask)`. Every run performs the same operations on the same addresses, the
traces agree, and the property holds at the source. This is code cryptographers
already write by hand. Orange's contribution would be to check it, and to
record that the check covered this function, under this model, at this layer,
and nothing further. The next section explains why the last clause matters.

### Why the source is not the binary

A routine can be perfectly constant-time as written and leak once compiled. An
optimizer may turn a carefully masked selection back into a branch, because a
branch is faster and, as far as ordinary semantics is concerned, equivalent.
Research on a modified CompCert showed that preserving cryptographic
constant-time is a distinct compiler proof, not a free consequence of ordinary
semantic preservation.

So a leakage claim about shipped bytes needs more than a source-level argument.
The [proposed evidence stack](ASSURANCE.md#43-evidence-stack) for a target
leakage claim runs through seven layers: source or CT IR noninterference;
pass-by-pass leakage preservation or checked translation validation;
correspondence between final object bytes and the accepted final semantics;
target instruction classification and ABI assumptions; binary static
inspection; empirical timing tests on named hardware as defense in depth; and
specialist laboratory work for release profiles that require it. While that
last kind of work is unavailable, those stronger profiles remain `unsupported`
rather than blocking everything else.

The proposed CT IR is the intermediate language built for this job. It is
first-order, monomorphized, fixed-width, and free of undefined behavior, and it
carries secret and public domains alongside an executable leakage trace. It is
where the compiler would check that its own transformations did not introduce
a leak.

### Testing finds; it does not prove

Statistical timing tools such as `dudect` run code on real hardware and look
for input-dependent timing. They can find leaks that the formal model in use
does not capture, which makes them valuable. But the absence of a detected signal is
not evidence of absence. A clean timing run is recorded as a test result about
a named corpus, method, and machine. It does not become a noninterference proof
because it came back clean, and it does not cover a different target.

Cryptanalysts will recognize the asymmetry. A distinguisher that succeeds is a
result. A distinguisher that fails is, at most, a data point about that
distinguisher.

### Where things stand

The leakage baseline is [D-012](DECISIONS.md#d-012--baseline-leakage-claim),
marked **investigate**. Its acceptance evidence includes a formal trace
semantics, a target instruction-classification process, positive and negative
examples, and a preservation plan through final bytes. The initial target
envelope is [D-011](DECISIONS.md#d-011--initial-native-target-envelope),
**proposed** as x86-64 Linux and AArch64 Linux, possibly only one of them if
solo capacity requires it.

The current compiler has no secrecy labels, no leakage semantics, no target
model, and no code generation. It therefore makes no constant-time claim of any
kind, and any such claim is outside its support envelope, which the claim model
would record as `unsupported`. What exists is the commitment that when Orange
does say something about leakage, it will say which observer, which layer,
which target, and which evidence, and it will say nothing more.

## Chapter 7: No Disposable Prototype

Most languages begin with a prototype. Someone writes a quick interpreter to
see whether an idea feels right, then a second, better implementation replaces
it once the design settles. For many projects that is exactly right. It is
cheap, it teaches quickly, and the prototype's shortcuts never matter because
nobody relies on it.

Orange rejects that path on purpose. [D-002](DECISIONS.md#d-002--no-disposable-prototype)
is a **directed** decision from the project owner: build the end product
through permanent, production-lineage components. There is no
prototype-to-rewrite phase and no minimum viable product that postpones the
core assurance claim. This chapter explains why a project about evidence makes
that choice, what it costs, and what it does not mean.

### Why prototypes are dangerous here

A prototype's defining property is that its internal decisions do not matter.
For an assurance-oriented toolchain, that property is the problem. Evidence
attaches to exact artifacts. A test run, a conformance result, or eventually a
proof says something about a particular implementation at a particular
revision. If that implementation is a prototype destined to be replaced, the
evidence goes with it, and the replacement starts with none.

Worse, prototypes tend not to die cleanly. Their conventions leak into the
successor: an error format that became familiar, a numeric shortcut that tests
came to rely on, a parse that was "temporarily" permissive. In an ordinary
product such residue is an annoyance. In a toolchain whose job is to say
precisely what was checked, residue is an undocumented semantic decision.

There is also a subtler risk. A prototype makes design choices by being
written. Once a syntax works in the prototype, it acquires momentum that has
nothing to do with whether it was the right choice. Orange's
[contribution rules](../CONTRIBUTING.md#solo-development-scope) name this
directly: do not add syntax or architecture selected merely by implementing it
first.

### What the doctrine requires

The [project charter](PROJECT_CHARTER.md#7-engineering-doctrine-build-the-end-product-directly)
turns D-002 into eight working rules. Paraphrased, they say:

1. Normative semantics precede convenience syntax and optimization.
2. Every committed component occupies its intended final boundary, with
   production error handling, deterministic output, tests, and versioned
   formats.
3. Early algorithms are permanent conformance fixtures, not demonstrations.
4. Automation may propose proofs; a proof-required claim closes only with
   replayable evidence accepted by the checker.
5. A backend cannot inherit a source property by assertion.
6. Performance work starts with the IR and cost model and stays subordinate to
   semantics and evidence.
7. Unimplemented claims fail closed.
8. No phase may create a second informal language inside build scripts,
   macros, or backend annotations.

The last rule is easy to underestimate. Many verification pipelines end up
with an unofficial language in their glue: a shell script that decides which
proof applies to which file, a macro convention that encodes a precondition, a
naming pattern that tells a tool what to trust. That glue has semantics, but
nobody specified them. Orange's rule is that anything with meaning belongs in
the language or in a checked record, not in the scaffolding around it.

### Small is not the same as temporary

The doctrine does not require the early system to be large. It requires each
small piece to be permanent. The current compiler shows what that looks like in
practice.

The first slice, under [D-024](DECISIONS.md#d-024--initial-compiler-foundation),
contained only source identities, byte spans, a deterministic lexer, structured
diagnostics, and the `orangec` command-line boundary. That is a tiny amount of
language. But each piece was built as the permanent version of itself:

- **Diagnostics have stable codes.** Lexical errors are `ORC0001` through
  `ORC0008`, parse errors `ORC0101` through `ORC0107`, semantic errors
  `ORC0201` through `ORC0210`, and evaluation resource exhaustion `ORC0301`.
  A code, once assigned, keeps its meaning, so tests, documentation, and users
  can refer to it.
- **Every phase is bounded.** Tokens, syntax nodes, parser events, Core nodes,
  semantic events, evaluation steps, and diagnostics each have a fixed ceiling,
  and exhausting one produces a stable resource diagnostic rather than a hang
  or a crash. Hostile input is a design case from the first commit.
- **Output is deterministic.** For the same bytes, edition, and compiler
  revision, the token stream, syntax tree, diagnostics, and evaluation output
  are identical, byte for byte.
- **The compiler has no third-party dependencies.** It uses the pinned Rust
  toolchain and the standard library only, so its trusted build inputs are
  short and explicit.
- **Public API boundaries are tested as boundaries.** The library's
  documentation tests include compile-fail examples showing, for instance,
  that a caller cannot rewrite a parse result or forge the type of an
  evaluated value after the fact.

The typed-literal slice showed what permanence means when a language grows.
S3a did not replace the S2 parser. It extended the same grammar with one new
tail for `spec` declarations, kept legacy empty `spec` and `impl`
declarations syntactically valid, and kept every earlier diagnostic code. Where
it did change behavior, it said so: same-kind duplicate names, which S2's
`check` accepted, now fail semantic checking with `ORC0201`, and OEP-0003
records that as a migration. The S2 conformance runner still runs beside the S3a runner. Where S3a
deliberately declined to extend the language, it said so in the grammar: a
typed `impl` is a syntax error, not a silently ignored annotation, because
implementation semantics have not been decided.

None of that makes the compiler impressive. It makes it extensible without
apology. When expressions arrive, they arrive in the same lexer, the same
parser, the same diagnostic system, and the same bounded analyzer, and the
evidence for the earlier slices still describes the code that runs.

### Research is not a parallel product

Some decisions cannot be made by argument alone. Choosing between a universal
Core and a family of related Cores, or between Rocq and Lean as a proof
foundation, benefits from running the candidates against the same cases. Orange
calls those experiments decision laboratories, and they live under
`research/decisions/` and in the compiler's integration tests.

D-002 is careful about their status. A decision case is research evidence, not
an unreviewed parallel implementation. The losing candidate never becomes a
second product, and the winning candidate's case graduates into the permanent
test suite rather than shipping as-is. The laboratories are instruments for
making a choice, not early versions of the thing chosen.

### What the doctrine costs

The costs are real, and the project does not pretend otherwise. Building
permanent components means writing error handling, budgets, and tests for
features that are still tiny. It means the language grows slower in breadth
than a prototype would, and that a reader browsing the repository finds a
great deal of rigor around a small amount of syntax.

The compensation is that nothing has to be thrown away and nothing has to be
re-earned. A diagnostic introduced in the first slice is still correct. A
conformance fixture written for the typed-literal slice is still a fixture. When
Orange eventually makes a claim about a compiled artifact, the claim will be
about code that grew continuously from the first commit, not about a successor
that inherited a prototype's reputation.

### What the doctrine does not mean

"No disposable prototype" is sometimes misread as "no change." It is not.
Pre-alpha syntax may evolve, and the roadmap says so. Components can be
refactored, generalized, and rewritten internally when their boundaries
improve. What the doctrine forbids is a change in *status*: shipping something
as a stand-in with the plan of replacing it, or letting a stand-in's behavior
become the definition by default.

Nor does it mean waiting. The project owner has been explicit that development
should keep moving: on 2026-09-28 the owner directed that Orange should not
freeze development unless the owner specifically asks for it. The permanent
lineage is a rule about quality and continuity, not a reason to pause. The
fastest honest path is to build the real thing in small, finished pieces, and
to keep building.

## Chapter 8: Orange 2026: The Smallest Honest Slice

Every language has a first edition that is embarrassingly small. Orange's is
smaller than most, and it is small on purpose. This chapter is a guided tour of
Orange 2026 as it exists: every construct it accepts, every value it can
compute, and the precise places where it stops. It is **current** throughout,
and every example in it was run against the compiler in this repository. The
normative sources are the [lexical and grammar specification](LANGUAGE_2026.md),
the accepted [typed-literal semantics](SEMANTICS_2026.md) of S3a, the
[pure expression specification](EXPRESSIONS_2026.md) of S3b, and the
[bindings and conversions specification](BINDINGS_2026.md) of S3c. S3b and S3c
are implemented and tested, but their specifications are **proposed**:
[OEP-0005](governance/oeps/OEP-0005-orange-2026-pure-spec-expressions.md) and
[OEP-0006](governance/oeps/OEP-0006-orange-2026-bindings-and-conversions.md)
are in the owner's review and have not been accepted. Where this chapter and
those documents disagree, they win.

The edition name matters. `2026` is not a version number that will be bumped
with every release. It names a language edition, in the same sense that Rust
editions do: a named interpretation of source text. During pre-alpha it
carries no stability promise and has already been extended in place, but every
change must say whether it extends 2026 or introduces a new edition and must
give an explicit source migration boundary, so meaning never changes silently.
Every Orange source begins by saying which edition it is written in, and today
exactly one exists.

### A complete program

Here is a program that touches most of what Orange 2026 accepts:

```orange
// A tour of Orange 2026.
edition 2026;
module tour {
  spec identity() {}
  impl rounds() {}

  spec answer() -> Int { 42 }
  spec negative() -> Int { -0x2a }
  spec mask() -> Word[8] { 0xff }

  spec square(n: Int) -> Int { n * n }
  spec two_to_the_64() -> Int { square(square(square(square(square(square(2)))))) }
  spec wraps() -> Word[64] { 0xffff_ffff_ffff_ffff + 1 }
  spec high_nibble() -> Word[8] { ~0x0f & mask() }
  spec rotated() -> Word[16] { 0x8001 <<< 4 }
}
```

Running `orangec check` on it prints nothing and exits with status 0. Running
`orangec eval` prints one line for each typed specification without
parameters, in source order:

```text
tour::answer: Int = 42
tour::negative: Int = -42
tour::mask: Word[8] = 0xff
tour::two_to_the_64: Int = 18446744073709551616
tour::wraps: Word[64] = 0x0000000000000000
tour::high_nibble: Word[8] = 0xf0
tour::rotated: Word[16] = 0x0018
```

The empty `identity` and `rounds` declarations are valid and print nothing:
they have no type, value, or execution meaning. `square` has a parameter, so it
prints nothing either; it runs only when another function calls it.
`two_to_the_64` shows that `Int` is not a machine integer: six nested squarings
of 2 reach 2^64 exactly, with nothing lost. `wraps` shows the other discipline.
`Word[64]` is the ring of integers modulo 2^64, and adding one to its largest
element gives zero, because that is what addition in the ring means.

### The lexical layer

Orange 2026 source is UTF-8, at most 16 MiB. Whitespace is exactly tab, line
feed, carriage return, and space. Comments come in two forms, `//` to the end
of the line and `/* ... */`, and block comments nest, so commenting out a
region that already contains a comment works as a reader expects.

Identifiers are ASCII: a letter or underscore followed by letters, digits, and
underscores. Seven words are reserved:

```text
edition  module  spec  impl  game  proof  claim
```

The first four have grammatical roles. `game`, `proof`, and `claim` are
reserved so that future editions can give them meaning without breaking
programs that used them as names; today they cannot be used at all.

Integer tokens are decimal, binary with `0b`, or hexadecimal with `0x`. A
single underscore may separate two digits, which keeps long constants
readable: `0x6a09_e667` rather than `0x6a09e667`. Leading, trailing, or doubled underscores are errors.

The expression slice gives grammatical roles to `,`, `:`, `+`, `-`, `*`, `&`,
`|`, `^`, and `~`, and adds four tokens of its own, `<<`, `>>`, `<<<`, and
`>>>`, matched longest first, so `<<<<` is `<<<` followed by `<`. String
tokens and the remaining punctuation (`%`, `&&`, and the rest) are lexically
reserved but have no grammatical role yet. `orangec lex` shows how any source
tokenizes, with exact byte spans.

### The grammar

The whole Orange 2026 grammar fits in twenty-six lines:

```text
source_file     = edition_decl module_decl EOF ;
edition_decl    = "edition" "2026" ";" ;
module_decl     = "module" IDENTIFIER "{" function_decl* "}" ;
function_decl   = "spec" IDENTIFIER "(" ")" spec_tail
                | "spec" IDENTIFIER "(" parameters ")" typed_tail
                | "impl" IDENTIFIER "(" ")" empty_body ;
spec_tail       = empty_body | typed_tail ;
typed_tail      = "->" parsed_type "{" binding* expression "}" ;
binding         = "let" IDENTIFIER ":" parsed_type "=" expression ";" ;
empty_body      = "{" "}" ;
parameters      = parameter ("," parameter)* ","? ;
parameter       = IDENTIFIER ":" parsed_type ;
parsed_type     = IDENTIFIER ("[" INTEGER "]")? ;

expression      = arithmetic | chain("&") | chain("|") | chain("^") | shift
                | conversion ;
conversion      = prefixed "as" parsed_type ;
arithmetic      = product (("+" | "-") product)* ;
product         = prefixed ("*" prefixed)* ;
chain(op)       = prefixed (op prefixed)+ ;
shift           = prefixed shift_operator prefixed ;
shift_operator  = "<<" | ">>" | "<<<" | ">>>" ;
prefixed        = literal | ("-" | "~") prefixed | primary ;
literal         = "-"? INTEGER ;
primary         = IDENTIFIER | call | "(" expression ")" ;
call            = IDENTIFIER "(" arguments? ")" ;
arguments       = expression ("," expression)* ","? ;
```

It has no implicit semicolons. The edition declaration must be first and
must spell `2026` exactly. `let` and `as` are the only contextual words: `let`
starts a binding only at the start of a body item and before a name, and `as`
converts only directly after a complete operand. Anywhere else they are
ordinary names, so no program that used them as names changed meaning when
they gained a role. One source holds one
module. A typed `impl` is a syntax error, not a feature waiting to be switched
on, and a `spec` with parameters must declare a result type and a body. A `-`
written directly before an integer is that literal's sign, so the S3a body
`{ -42 }` is still one literal and means what it always meant.

### Grouping you can see

Most languages inherit a precedence table from C, and few programmers can
recite it. In C, `a + b ^ c` means `(a + b) ^ c` and `a & b == c` means
`a & (b == c)`, and cryptographic code is exactly where those rules bite.
Orange 2026 keeps only the precedence every reader already knows: prefix
operators bind first, and `*` binds more tightly than `+` and `-`. Beyond that,
operators fall into five groups: arithmetic, `&`, `|`, `^`, and the shifts and
rotations. Two operators from different groups may not share a level without
parentheses:

```text
error[ORC0108]: `^` follows `+` without grouping parentheses
 --> <stdin>:4:11
  |
4 |     a + b ^ b <<< 7
  |           ^ ungrouped operator
  = note: operators from different groups have no relative precedence in Orange; parenthesize the part that applies first
```

The rule costs a pair of parentheses and buys an expression that means what it
looks like. It also matches the standards. FIPS 180-4 writes the choice
function as `(x ∧ y) ⊕ (¬x ∧ z)`, with its grouping visible, and the Orange
transcription is `(x & y) ^ (~x & z)`, the same shape symbol for symbol. A
shift or rotation takes exactly two operands, and its amount must be a literal
that fits the width, so `x >>> 32` on a `Word[32]` is an error rather than a
question about what some processor does.

### Five types

Five types have meaning:

| Source form | Meaning | Values |
| --- | --- | --- |
| `Int` | Mathematical integers | Every integer, positive or negative |
| `Word[8]` | The integers modulo 2^8 | 0 through 255 |
| `Word[16]` | The integers modulo 2^16 | 0 through 65,535 |
| `Word[32]` | The integers modulo 2^32 | 0 through 4,294,967,295 |
| `Word[64]` | The integers modulo 2^64 | 0 through 2^64 − 1 |

The distinction is the seed of everything Orange will later say about
arithmetic. A specification over `Int` is mathematics and does not overflow. A
specification over a word type is about machine words, and its arithmetic is
modular by definition, the way the standards write it. A literal is different:
a value outside a word's range is an error rather than a wrapped value, because
a constant that does not fit is almost always a transcription mistake. No
value changes type implicitly, and nothing is inferred. `Word` with any width
other than the exact decimal tokens `8`, `16`, `32`, and `64` is rejected, and
so is `Int` with a width.

### Naming steps and changing types

Standards are written as sequences of named steps, and they move values
between bytes, words, and integers constantly. The S3c slice gives Orange
both. A typed body may begin with `let` bindings, each with a name, a stated
type, and a value, and each ended by a semicolon; the last expression is still
the body's result. Here is the ChaCha20 quarter round of RFC 8439, as far as
its first output word:

```orange
spec quarter_a(a: Word[32], b: Word[32], c: Word[32], d: Word[32]) -> Word[32] {
  let a1: Word[32] = a + b;
  let d1: Word[32] = (d ^ a1) <<< 16;
  let c1: Word[32] = c + d1;
  let b1: Word[32] = (b ^ c1) <<< 12;
  a1 + b1
}
```

Where the RFC updates `a` in place, the Orange text writes `a1` and then `a2`.
A binding never shadows a parameter or another binding, so every name in a
body refers to exactly one thing, and a name is in scope only after its own
semicolon. Each binding is evaluated once, in order, before the result. The
stated type is not decoration. It is the one fact a reader checking a
transcription most needs, so Orange does not infer it.

A conversion, written `e as T`, is the only way a value changes type, and its
meaning is one rule: take the operand's integer value and, for `Word[n]`, its
residue modulo 2^n. Widening keeps a value, narrowing keeps the low bits, and
an `Int` holding -1 becomes `0xff` as a `Word[8]`. That single rule is enough
for byte order:

```orange
spec load_le32(b0: Word[8], b1: Word[8], b2: Word[8], b3: Word[8]) -> Word[32] {
  (b0 as Word[32]) | ((b1 as Word[32]) << 8) | ((b2 as Word[32]) << 16)
    | ((b3 as Word[32]) << 24)
}
```

Applied to the bytes `00 01 02 03`, it gives `0x03020100`, the first ChaCha20
key word of RFC 8439 section 2.3.2. The same function with its arguments
reversed reads SHA-256's big-endian message words.

A conversion applies to exactly one operand and forms a group of its own, under
the grouping rule above. `x + y as Word[32]` is `ORC0108`, because its two
readings differ: for bytes `x` and `y`, `(x + y) as Word[32]` adds modulo 2^8
and then widens, while `(x as Word[32]) + (y as Word[32])` adds modulo 2^32.
The parentheses say which one the standard means.

### From bytes to a value

It is worth following one line through the compiler, because each step is a
permanent phase rather than a shortcut. Take `spec negative() -> Int { -0x2a }`.

The lexer reads bytes and produces tokens with exact spans: the reserved word
`spec`, the identifier `negative`, two parentheses, the arrow, the identifier
`Int`, a brace, a minus sign, the integer token `0x2a`, and a closing brace. It
assigns no numeric value; `0x2a` is still only a spelling. The parser
recognizes the typed `spec` tail and builds a syntax node that records the
sign, the integer token, and the type syntax, each with its span. Semantic
analysis then does three separate things: it resolves `Int` to the
mathematical integer type, decodes the hexadecimal magnitude 42, and applies
the sign to obtain -42, checking the magnitude bound along the way. The result
is lowered into the Typed Reference Core, a small typed representation that
knows nothing about spelling. Finally the evaluator reads the Core and prints
`tour::negative: Int = -42`.

A call follows the same chain with more work in the middle. In
`square(square(2))`, the parser builds a call node around a call node around a
literal. Semantic analysis finds that `square` names a typed `spec` in the
module, checks that it takes one argument, checks that argument against `Int`,
and records an edge in the call graph, which must stay acyclic. The Core stores
the body in postorder: the literal, then the inner call, then the outer call,
each tagged with its type. The evaluator runs that sequence under a step budget
and a limit of 256 call frames.

Nothing in that chain is a stand-in. Each phase is bounded, deterministic, and
tested on its own, and each is the phase that later slices will extend.

### Diagnostics

Errors carry stable codes, precise spans, and a note explaining the rule. A
duplicate declaration points at both places:

```text
error[ORC0201]: duplicate spec function `same`
 --> <stdin>:4:8
  |
4 |   spec same() {}
  |        ^^^^ this declaration repeats a name in the same namespace
 ::: <stdin>:3:8
  |
3 |   spec same() {}
  |        ---- first declaration is here
  = note: `spec` and `impl` use separate declaration namespaces
```

The code families follow the phases. `ORC00xx` codes are lexical, `ORC01xx`
parse, `ORC02xx` semantic, and `ORC0301` evaluation resource exhaustion.
`ORC10xx` codes belong to the command line itself: unreadable files, invalid
UTF-8, oversized input, and similar. Once a code is assigned, it keeps its
meaning; wording can improve, but a code is never reused for a different error.
[Appendix A](#appendix-a-current-grammar-and-cli) lists them all.

A few rules show how the phases divide the work. `spec x() -> Word[12] { 1 }`
parses, then fails semantic analysis with `ORC0204` because only widths 8, 16,
32, and 64 are supported. `spec x() -> Word[8] { -1 }` fails with `ORC0206`, a
negative word literal. Two declarations named `spec x` fail with `ORC0201`,
while `spec x` and `impl x` together are fine. A stray `@` fails lexing with
`ORC0001` and is never parsed at all.

The expression slice adds codes in the same families: `ORC0108` for ungrouped
operators, and `ORC0211` through `ORC0218` for unknown names and functions,
wrong argument counts, type mismatches, undefined operators, bad shift and
rotation amounts, call cycles, and repeated parameter names. A cycle is
reported once, at the call that closes it:

```text
error[ORC0217]: call cycle `even` -> `odd` -> `even`
 --> <stdin>:4:29
  |
4 |   spec odd(n: Int) -> Int { even(n) }
  |                             ^^^^^^^ this call closes the cycle
  = note: a `spec` may not depend on itself; recursion is not part of Orange 2026
```

The binding slice adds `ORC0219` for a name bound twice and `ORC0220` for a
conversion whose operand has no type of its own, such as `(1 + 2) as Word[8]`,
where nothing says which ring the addition belongs to. A name used before its
binding is `ORC0211`, and the error also points at the binding that comes
too late.

One mistake is never reported twice through its consequences. A call to an
unknown function stops there, without complaints about its arguments, and a
parameter or binding whose type was already rejected is not rejected again at
each use.

### The command line

`orangec` has three commands:

```text
orangec [OPTIONS] <check|eval|lex> <FILE>...
```

- `check` performs lexical, syntactic, and semantic validation of one or more
  sources and is silent on success.
- `eval` validates exactly one source and prints the value of each typed
  `spec` without parameters.
- `lex` prints the deterministic token stream.

`-` reads UTF-8 source from standard input. `--edition 2026` selects the
edition explicitly. `--version` prints `orangec 0.0.1 (Orange edition 2026)`.
The exit status is 0 on success, 1 when compilation or I/O fails, and 2 for a
usage error. Output streams are bounded like everything else. A compiler-phase
failure makes `eval` print no values at all; if writing the output itself
fails, `eval` exits with status 1, although a prefix the stream already
accepted may remain, and that prefix is never reported as a result.

### Conformance

The specifications are backed by executable conformance. The lexical and
grammar document defines thirteen rule identifiers, `S2-SOURCE-01` through
`S2-DETERMINISM-01`, each mapped to named tests that a conformance runner
checks. The typed-literal semantics adds its own rule index and an external
black-box corpus: three valid and seven invalid sources that the runner feeds
through `check` and `eval` twice each, comparing exact codes, messages, and
locations. The expression specification does the same with 28 rule
identifiers and fourteen sources, five valid and nine invalid. Two of the
valid sources are the SHA-256 round functions and the ChaCha20 quarter round,
checked against the values published with the standards, and generated
sources pin every resource limit at its exact boundary. The binding and
conversion specification adds 17 rule identifiers and ten sources, five valid
and five invalid, including SHA-256 message words and round 0 of the "abc"
example and the ChaCha20 quarter round written with named steps. The complete
test suite covers the lexer, parser, semantic analyzer, Core, evaluator,
diagnostics, resource limits, and command-line behavior.

The documents are careful about what those tests mean. A named test is evidence
for the recorded implementation revision; it does not prove that a rule is
complete or that the parser is correct. That caution is not modesty. It is the
same discipline Chapter 2 applied to claims, applied to the project's own
compiler.

### What Orange 2026 does not have

The list of absences is long, and it is printed in the specifications rather
than hidden: imports, multiple modules, attributes, visibility, generic
arguments, contracts, effects, statements other than `let`, mutation,
shadowing, type inference, tuples, arrays, booleans, comparisons,
conditionals, loops, division, remainder, signed words, variable shift and
rotation amounts, recursion, typed implementations,
failure values, secrecy labels, proof terms, claims, games, targets, layout,
ABI, leakage behavior, lowering, optimization, code generation, packaging, and
releases.

That is not a finished language in miniature, and it does not pretend to be.
It is the smallest language whose every behavior is specified, bounded,
tested, and deterministic, built as the permanent foundation that later slices
extend. S3b is its first step past literals. It was built before the semantic
strata decision described in
[Chapter 3](#chapter-3-one-language-several-semantic-worlds), and it assumes
only what every candidate gives the specification stratum: pure, total,
deterministic meaning over mathematical values. Accepting it is the owner's
decision, through OEP-0005, and S3c's, which builds on it, through OEP-0006.
Orange 2026 is pre-alpha and makes no compatibility promise, but any change to
what the programs in this chapter mean has to arrive with an explicit,
documented migration. Both migrations so far are small: every source that S3a
accepted still has the same values and prints the same bytes under S3b, and
every source S3b accepted does the same under S3c.

## Chapter 9: From Core to Native Bytes

A specification that cannot run fast is a reference, not a product. The
cryptography that protects real systems runs on real processors, often inside
tight loops that have been tuned instruction by instruction. If Orange is to
matter to the engineers who write that code, it has to produce native output
that competes with hand-tuned C and assembly. And if Orange is to matter to
everyone else, the properties established upstream have to survive the trip.

This chapter describes that trip. Almost all of it is **proposed**. Orange
currently generates no code, has selected no compiler strategy, and supports no
target. What exists is a precise account of what any strategy would have to
prove, and a comparison designed to choose among five of them.

### Every arrow needs a reason

The [architecture](ARCHITECTURE.md#1-architecture-objective) draws the
direct-native path as a chain: source modules elaborate into Spec, Impl, and
Game Cores; the implementation lowers into CT IR; verified or validated passes
transform CT IR; the result lowers into Machine IR; checked encoding produces
an object or library together with a generated C interface. Beside that chain
runs a second one: every Core and every transition relation feeds a claim graph,
whose obligations are discharged by automation that produces certificates,
checked by an authoritative checker, and packaged into an evidence bundle.

The most important sentence in that part of the architecture is a rule about
arrows. Every formal-preservation arrow is justified in exactly one of four
ways:

1. it is part of the normative semantics;
2. it is justified by a kernel-checked theorem;
3. it is accompanied by a checked per-artifact certificate; or
4. it is recorded as an explicit external assumption.

There is no fifth category called "obvious glue." Tests, audits, and external
validations remain valuable, but they attach to scoped claims as separate
evidence. They do not close a formal preservation obligation. A compiler that
cannot say which of the four justifies a given step has not preserved anything;
it has merely produced output.

### Two ways to trust a compiler

There are two classic answers to the question of how a compiler can preserve a
property. The first is to prove the compiler correct once: each pass carries a
theorem saying that, for every input, its output behaves as its input did.
CompCert is the famous example. The cost is a large proof, but every later
compilation inherits it for free.

The second is **translation validation**: let the compiler do whatever it likes,
and check each particular output against its input. The pass can be a heuristic
optimizer, a scheduler, or a register allocator that nobody would want to
verify. What gets checked is the certificate it emits for this one artifact.
The cost moves from the compiler's authors to every build, but the trusted base
shrinks to the checker.

Orange's proposals use both. A stable structural pass might carry a reusable
theorem. An aggressive optimization might emit a per-artifact certificate
covering both functional behavior and leakage. The charter's rule is that
evidence survives optimization or the optimization does not run: a pass that
can offer neither a mechanized preservation theorem nor a checked certificate is
excluded from an assurance-preserving build.

### The claim frontier

Not every strategy has to carry its claims all the way to machine code. The
useful concept is the **claim frontier**: the exact point in the pipeline where
a compiler claim stops. A strategy that emits portable C can make honest claims
about the C it emits. It cannot claim anything about what a C compiler then does
with that output unless a checked relation for that C toolchain is separately
accepted.

Seen this way, "Orange compiles to C" and "Orange compiles to native code" are
not two implementations of the same promise. They are two different promises,
and a report must show which one was made.

### Five candidate strategies

[D-010](DECISIONS.md#d-010--compiler-strategy) compares five compiler strategies
symmetrically in the
[compiler-strategy suite](COMPILER_STRATEGY_DECISION_SUITE.md):

| ID | Strategy | Where the claim frontier ends |
| --- | --- | --- |
| CP-01 | Theorem and certificate hybrid, direct to native code | Final bytes, with per-artifact certificates for optimization |
| CP-02 | Mechanized proof for every pass, direct to native code | Final bytes, with a reusable theorem per pass |
| CP-03 | Versioned Jasmin backend boundary | Exact Jasmin input, unless a downstream relation is accepted |
| CP-04 | Portable C11 interoperability boundary | Deterministic C11 output, unless a C-toolchain relation is accepted |
| CP-05 | Versioned LLVM IR interoperability boundary | Exact LLVM IR, unless an LLVM-pipeline relation is accepted |

The suite insists that CP-04 and CP-05 are separate candidates. A C source
boundary and an LLVM IR boundary have different semantics, different undefined
behavior, different toolchains, and different target assumptions. Results for
one may not be reused for the other, and "C or LLVM" is not a strategy.

Each candidate must work through the same eight cases: freezing the pipeline
and its authorities; preserving functional meaning through structural lowering;
handling optimization, scheduling, vectorization, and allocation; preserving
one named leakage model; exercising the endpoint and, when claimed, the final
object; failing closed under corruption, substitution, failure, and fallback;
binding replay identities and comparing against the reference semantics; and
measuring what one owner can actually audit and maintain. That makes 40
candidate-case runs. None has been executed, and D-010 cannot be accepted before
the product-form, semantic-strata, assurance-model, proof-foundation, and
solver-trust decisions it depends on.

One constraint in the suite is aimed squarely at wishful thinking. A
direct-native candidate may not shrink its promised frontier to relabel missing
final-byte evidence as unsupported. If a strategy promises native code, the
last mile is part of its exam.

### The last mile

The last mile is where many end-to-end stories quietly stop. A compiler can be
proved correct down to an assembly-like representation and still hand its
output to an assembler, a linker, and a loader whose behavior is merely
assumed. For a direct-native candidate that carries a claim to final objects,
Orange's architecture lists what must be bound and validated: instruction bytes
against the final internal semantics, section placement and permissions,
constants and tables, relocations and symbol bindings, stack and call behavior
at the ABI boundary, CPU-feature dispatch, and final exported-symbol digests.
Any tool that remains in that path either appears in the claim closure or is
covered by an accepted checked relation.

### Several implementations, one specification

Real libraries ship more than one implementation of the same primitive: a
portable version and one or more accelerated versions for particular
instruction-set extensions. Orange's proposal treats each as a separate
implementation with its own claims, and treats the dispatcher that chooses
among them as an implementation too. The dispatcher needs its own proof: that
feature detection is correct under the platform's contract, that it selects
only an implementation whose preconditions hold, that every candidate refines
the same specification, and that fallback never silently lowers assurance.

### The reference evaluator's role

Today's reference evaluator runs pure functions over integers and words. It is
small, but its role in this chapter is permanent. The architecture keeps the reference semantics as a
common differential oracle, independent of whichever strategy D-010 selects.
Every future output path can be run against it on the same inputs. A mismatch
is not a proof of anything, but it is a cheap, early, and very loud alarm.

### Where things stand

Orange has no intermediate representation beyond the Typed Reference Core, no
lowering, no optimization, no code generation, no object output, and no target.
The initial target envelope is proposed under
[D-011](DECISIONS.md#d-011--initial-native-target-envelope) as x86-64 and
AArch64 on Linux, with host tools on Linux, macOS, and Windows. None of it is
implemented.

That is the honest state of a project that decided to settle meaning before
speed. The payoff is that when Orange does emit its first byte of machine code,
the question "what does this byte have to do with the specification?" will
already have a required answer, and a place in the evidence to record it.

## Chapter 10: The Foreign Boundary

Most cryptographic code is ultimately called by code that is not
cryptographic. A key
exchange is called by a TLS stack; a signature routine is called by a package
manager; a hash is called by a database, a file system, or a browser. The
caller is ordinary software, written in C, Rust, Go, Python, or something else,
by people who have never seen the proofs behind the routine and should not have
to.

That makes the foreign interface the place where assurance is most often lost.
A proof that a routine is memory-safe assumes its buffers are as long as the
proof says. A proof that it is correct assumes its inputs do not overlap its
outputs, or does not, and the difference matters. A constant-time argument
assumes that the caller has not copied the secret somewhere the routine cannot
see. None of those assumptions is visible in a C prototype such as
`int encrypt(unsigned char *out, const unsigned char *in, size_t len)`. The
declaration says what types flow across the boundary. It says almost nothing
about what must be true of them.

This chapter describes Orange's **proposed** answer. The foreign boundary is
[D-013](DECISIONS.md#d-013--stable-foreign-boundary), and nothing in it is
implemented.

### A header is not a contract

A C header is a remarkably thin description of a function. It gives the
number and types of arguments, the return type, and sometimes a comment. It
does not state that `out` must have room for `len` bytes, whether `out` may
equal `in`, whether either may be null when `len` is zero, what alignment is
required, which error values are possible and what the output buffer contains
after an error, whether the function allocates, whether it may panic or abort,
whether it reads thread-local state or a global random-number generator, and
whether it leaves secret material behind in memory it does not own.

Every one of those questions has a correct answer for a particular routine, and
every one is a place where an integrator can go wrong. Mature libraries answer
them in documentation. Orange's proposal is to answer them in a machine-readable
contract that is generated from the same definition as the code, checked
against the implementation, and carried in the evidence.

### The proposed contract

The [architecture's foreign-interface section](ARCHITECTURE.md#13-foreign-interface)
lists what the stable integration boundary would state for every export:

- exact scalar and aggregate layout;
- buffer lengths, alignment, overlap, mutability, and initialization;
- typed error and failure behavior;
- no hidden allocator, exception, panic, thread-local state, or random-number
  generator;
- a stable symbol and version policy;
- explicit zeroization and ownership-transfer rules; and
- target-feature and dispatcher requirements.

The boundary itself is a generated C ABI, because C is the lingua franca that
every other language can call. Rust wrappers sit above it and enforce whatever
Rust's type system can represent: lengths tied to slices, exclusive borrows
where overlap is forbidden, and typed errors instead of integer codes. Whatever
the wrapper cannot express, it checks at run time. Bindings for other languages
would sit on the same C contract.

The single most important property is that the header, the wrapper, the
contract, and the object all derive from one definition. Hand-maintained
headers are a quiet source of disagreement between what a library does and
what its users believe it does. Generating them removes the disagreement at
its source.

Consider authenticated decryption. Its most dangerous question is what the
output buffer holds when the tag does not verify. A routine that decrypts in
place before checking the tag can leave unauthenticated plaintext in the
caller's memory, and a caller that ignores the error code will use it. A
contract would state the answer, for example that the output is left unwritten
or is overwritten before the error is returned, and the adversarial callers
described later in this chapter would check that the implementation keeps the
promise.

### Imports are assumptions

The boundary runs in both directions. Sometimes Orange code must call out: to
an operating-system entropy source, to a platform's secure-memory facility, or
to an existing library that nobody is going to rewrite. The proposal is simple
and strict. Every foreign import names its ABI, preconditions, effects, alias
rules, failure behavior, and any claims it is assumed to satisfy. Until those
claims are separately proved, the import is an assumption, and the assumption is
attached to every dependent claim.

That rule changes how a claim report reads. A routine that is proved correct
except for one imported call does not report "proved." It reports its claim
together with the named assumption about that call, and a reader can decide
whether that assumption is acceptable in their setting. The trust budget of
[Chapter 15](#chapter-15-offline-replay-and-trust-budgets) is where those
assumptions accumulate and become visible.

### Secrets across the line

The foreign boundary is also where secrecy leaves Orange's type system. Inside
a claim-bearing kernel, the compiler can track which values are secret and
where they flow. Outside, the caller holds the key in whatever memory it
chooses. The proposed contract therefore records which arguments carry secret
material, what the routine guarantees about erasing its own copies, and what it
cannot guarantee about the caller's. An erasure claim covers architecturally
modeled storage that the routine controls. It never promises that the caller's
buffers, swap space, or crash dumps are clean.

The claim model gives these properties a home. An `abi` claim asks whether the
object and its wrapper satisfy a named calling, layout, alias, and error
contract. An `erases` claim asks whether named secret storage is overwritten
under a stated machine model. Both are separate from functional correctness and
from leakage, and each can be satisfied, not satisfied, unresolved, or
unsupported on its own.

### Testing the boundary as an adversary would

A contract is only as good as the evidence that the implementation honors it.
D-013's acceptance evidence calls for an ABI model and adversarial callers for
each supported target tuple. In practice that means callers written to break
the rules: for example, callers that pass short buffers, overlapping buffers,
misaligned pointers, null pointers with zero lengths, or maximal lengths. Each
such case should either be rejected in the way the contract says or be shown to
fall outside what the contract promises, never silently accepted.

### Primitives are not protocols

One more boundary deserves mention because it is so easy to blur. A primitive
standard defines mathematics and core algorithms. A deployment profile defines
bytes on the wire: identifiers, encodings, negotiation, error handling, and
state. The [research analysis](RESEARCH.md#410-primitive-correctness-is-not-protocol-interoperability)
uses post-quantum key encapsulation as the example. FIPS 203 defines ML-KEM;
RFC 9935 specifies its use and key encodings in X.509; RFC 9936 specifies CMS
integration and warns about compatibility boundaries with pre-standard Kyber.

A proof about ML-KEM's arithmetic does not imply correct object identifiers,
correct ASN.1 encoding, correct TLS negotiation, or a correct host API. In
Orange's proposal, those are separately specified and separately proved
serialization and adapter modules, each with its own claims. The boundary
between a primitive and its protocol profile is a foreign boundary of a
different kind, and it receives the same treatment.

### Where things stand

Orange has no foreign interface today. It emits no objects, generates no
headers or wrappers, and makes no ABI or erasure claim. The current `orangec`
command line is a boundary of a humbler kind: it reads UTF-8 source, bounds its
input and output, and reports failures through stable codes and exit statuses.
That discipline, small as it is, is the same one the foreign boundary will
need. Every input is hostile until checked, every limit is explicit, and every
failure is reported rather than absorbed.

## Chapter 11: Standards as Versioned Inputs

Cryptographers cite standards the way mathematicians cite theorems: by name,
as if the name fixed the content forever. "Implements ML-KEM." "Conforms to
FIPS 180-4." "RFC 8439 ChaCha20-Poly1305." In conversation that is fine. In an
assurance claim it is not, because standards change after publication, and the
change is often exactly the part that matters.

This chapter explains why Orange treats a standard as a versioned input with
provenance rather than a timeless citation. The discipline is **directed** for
any future cryptography package and **proposed** in its exact record format.
No standard has yet been imported into Orange.

### Standards move

The [research analysis](RESEARCH.md#46-standards-are-versioned-inputs-not-timeless-citations)
gives current examples. FIPS 203 and FIPS 204, the post-quantum key
encapsulation and signature standards, published planning notes and errata
after their final publication. The set of algorithms and schemas supported by
NIST's Automated Cryptographic Validation Protocol evolves. Protocol profiles
that build on a primitive may remain Internet-Drafts, as
[section 4.10](RESEARCH.md#410-primitive-correctness-is-not-protocol-interoperability)
notes, and must not be represented as finalized standards.

None of that reflects badly on the standards process. It is what careful
standardization looks like. But it means that "implements ML-KEM" is an
incomplete sentence. Which publication? Which errata were applied? Which
clauses does this definition transcribe? Which vector set was it tested
against? Did the implementer deliberately deviate anywhere, and why? Without
those answers the claim cannot be audited, only believed.

### What provenance records

For every standard that becomes a decision input, Orange's provisional
[reproducibility contract](REPRODUCIBILITY.md#6-external-source-and-standards-capture)
requires capturing the following, and the roadmap requires the same exactness
before any cryptographic claim:

- the issuing organization, exact document identifier, edition, and date;
- the primary publisher or an authorized mirror;
- the retrieval time and an exact byte digest;
- an archive path, or digest-verifying instructions for reacquiring the bytes;
- redistribution terms and their review status;
- applicable errata and the rationale for applying them;
- the clause or vector locators that the work depends on;
- the path, digest, and review state of any transcription; and
- any unresolved patent, export, certification, or access question, recorded
  without turning a technical inventory into legal advice.

A provisional
[standards-provenance schema](../schemas/gate0/standards-provenance-v0.1.schema.json)
gives those fields a concrete shape: a standard with its issuer, identifier,
edition, and publication date; a source with retrieval time, digest, and
archive state; rights, patent, export, and technical review records; errata
with applicability; normative clauses with transcriptions; and vector sources
with scope and expected interpretation. The schema is explicitly historical
and non-product. It shows the shape of the idea, not the final format.

### Clauses, not documents

The most useful granularity is not the document but the clause. A definition
of SHA-256's message schedule transcribes a specific section of FIPS 180-4. A
definition of ML-KEM's `Decaps` transcribes a specific algorithm in FIPS 203,
possibly as corrected by a specific erratum. When the transcription is linked
to its clause, a reviewer can put the two side by side. When an erratum
changes that clause, every definition that depends on it can be found.

Orange's specification stratum is meant to make those side-by-side readings
pleasant. A cryptographer comparing a standard's pseudocode with an Orange
specification should see the same structure: the same names where the
standard names things, the same word sizes, the same rotations and additions,
the same loop over the same indices. The intended style is that of a careful
transcription, not a clever re-derivation. Clever re-derivations belong in the
implementation stratum, where a refinement proof connects them back to the
plain version.

### A clause, read closely

FIPS 180-4 defines six logical functions for SHA-224 and SHA-256 in its
section 4.1.2. Four of them look almost alike:

```text
Σ0(x) = ROTR^2(x)  ⊕ ROTR^13(x) ⊕ ROTR^22(x)
Σ1(x) = ROTR^6(x)  ⊕ ROTR^11(x) ⊕ ROTR^25(x)
σ0(x) = ROTR^7(x)  ⊕ ROTR^18(x) ⊕ SHR^3(x)
σ1(x) = ROTR^17(x) ⊕ ROTR^19(x) ⊕ SHR^10(x)
```

A faithful transcription has to preserve more than it first appears. The
operands are 32-bit words. `ROTR` is a rotation and `SHR` is a shift, and the
last term of each lowercase function is a shift, not a rotation. Addition
elsewhere in the algorithm is modulo 2^32. The uppercase functions belong to
the compression function and the lowercase ones to the message schedule. A
transcription that swapped one rotation amount, or wrote a rotation where the
standard has a shift, would still produce a plausible-looking hash function.
It would fail the official vectors, which is why vectors matter, but a
reviewer comparing the text side by side should be able to see the mistake
before running anything.

That is the standard Orange sets for its specification stratum: a reviewer
should be able to hold the clause in one hand and the definition in the other
and check them symbol by symbol. With the expression slice, Orange can write
these four lines, and they read like this:

```orange
spec big_sigma0(x: Word[32]) -> Word[32] { (x >>> 2) ^ (x >>> 13) ^ (x >>> 22) }
spec big_sigma1(x: Word[32]) -> Word[32] { (x >>> 6) ^ (x >>> 11) ^ (x >>> 25) }
spec small_sigma0(x: Word[32]) -> Word[32] { (x >>> 7) ^ (x >>> 18) ^ (x >> 3) }
spec small_sigma1(x: Word[32]) -> Word[32] { (x >>> 17) ^ (x >>> 19) ^ (x >> 10) }
```

`>>>` is `ROTR`, `>>` is `SHR`, and `^` is `⊕`. The last term of each
lowercase function is visibly a shift, and every amount is a literal that the
compiler checks against the width. These definitions live in the compiler's
conformance fixtures, where Σ0 and Σ1, with the choice and majority functions,
reproduce the working variables `a` and `e` after round 0 of NIST's "abc"
example. That
is still not a transcription in this chapter's sense. The fixture records no
exact edition, errata state, or provenance, and a function that evaluates to a
published value is not thereby a verified reading of the standard.

### The intent boundary

There is a limit that no amount of formalism removes. A proof checker can
establish that an implementation refines an Orange specification. It cannot
establish that the Orange specification says what the standard's authors meant.
That link runs through human reading, and the architecture says so plainly:
the checker can prove a formal specification, while humans, standards
provenance, independent implementations, and test vectors establish that the
formal specification is the intended algorithm.

Orange's proposed response is to make that human link as small, explicit, and
checkable as possible. Transcriptions would be linked to clauses. Official
vectors would be imported with their provenance and run against the executable
specification itself, not only against the fast implementation. Differential
tests would compare against mature implementations of the same standard. And
the transcription's review status would be recorded honestly. In solo mode that means
the record says the owner cross-checked it and that external cryptographer
review is unavailable. A claim whose policy requires external review remains
unsupported rather than silently satisfied.

### Vectors are versioned too

Test vectors deserve the same treatment as the prose. A vector set has a
source, a version, and an interpretation: which fields are inputs, which are
expected outputs, and which cases are expected to fail. Wycheproof-style
adversarial suites include deliberately invalid inputs whose correct outcome is
rejection. If the interpretation is wrong, a passing test proves nothing, or
worse, proves the wrong thing. The provenance schema therefore records each
vector source with its scope and expected interpretation, not just its bytes.

A conformance claim then becomes precise. It names the standard edition and
errata, the profile, the vector set and its digest, and the implementation and
target it was run against. That claim can be satisfied while a functional
refinement claim about all inputs remains unresolved, and the report will show
both.

### Rights are an input

Standards and vectors come with terms. Some may be redistributed freely; some
may be quoted but not copied wholesale; some must be fetched from their
publisher. The reproducibility contract archives exact bytes only when the terms
permit, and otherwise records instructions for reacquiring and verifying them.
A screenshot is never a normative input, and a generated transcription never
replaces its source.

Orange's own licensing is part of this picture.
[D-018](DECISIONS.md#d-018--licenses) leaves the repository's outbound license
open, which is one reason no cryptographic package has been published. The
working recommendation preserves vector and standards provenance according to
each source's terms, whatever license Orange eventually adopts.

### Where things stand

Orange has imported no standard and transcribed no clause with recorded
provenance. Its compiler fixtures evaluate the SHA-256 round functions and the
ChaCha20 quarter round against values published with the standards, but those
fixtures test the compiler; they are not corpus entries and make no claim
about the standards. The first cryptography package will require exact standards and errata
provenance, vectors, negative cases, and complete assumptions before it makes a
claim. When it arrives, the plan is that a reader will be able to point at any
line of an Orange specification and ask which sentence of which edition of
which standard it came from, and get an exact answer.

## Chapter 12: The Corpus as Acceptance Test

Every new language ships with examples. They are usually chosen to flatter it:
short programs that show off the syntax, avoid the awkward corners, and run
quickly. Orange intends something more demanding. Its first-party cryptography
packages are not examples. They are the acceptance test for the entire
toolchain.

The [project charter](PROJECT_CHARTER.md#8-product-principles) puts it as a
principle: *the standard library proves the product*. The flagship corpus is not
a marketing sample. It is the end-to-end acceptance suite for expressiveness,
proof ergonomics, generated code, interoperability, documentation, and
maintenance. If Orange cannot express, prove, compile, and ship real
cryptography with a complete claim matrix, then it has not succeeded, however
elegant its smaller programs look.

### Why real algorithms, early

It is tempting to postpone real cryptography until the compiler is finished.
Orange's architecture argues the opposite: the corpus is developed with the
compiler, not after it. Real algorithms surface requirements that toy programs
never do. SHA-256 forces precise word semantics and streaming state. AES-GCM
forces a decision about table lookups in a constant-time profile, because the
classic table-driven AES is a textbook cache-timing leak. ML-KEM forces
rejection sampling, polynomial arithmetic, and a careful account of which
failures may be observable. A language designed without those pressures tends
to discover them late, when changing the design is expensive.

The same reasoning explains the no-disposable-prototype rule of
[Chapter 7](#chapter-7-no-disposable-prototype). Early algorithms become
permanent conformance fixtures. They are not throwaway demonstrations that a
later compiler must re-earn.

### A corpus chosen for coverage

The proposed corpus is chosen for the capabilities each family exercises, not
for breadth or popularity. The
[assurance model's corpus plan](ASSURANCE.md#6-flagship-corpus-plan) pairs each
family with its architectural purpose:

| Family | What it exercises |
| --- | --- |
| SHA-256 and SHA-512 | Bit and word semantics, streaming state, vectors |
| ChaCha20-Poly1305 | Add-rotate-xor code, field arithmetic, AEAD API and failure behavior |
| AES-GCM | Hardware intrinsics, forbidden tables in a constant-time profile, dispatch |
| HKDF and HMAC | Generic modules and composition |
| X25519 and Ed25519 | Finite fields, encodings, scalar handling |
| ML-KEM | Polynomials, matrices, rejection and failure behavior, post-quantum standards |
| ML-DSA or SLH-DSA | Larger state, performance pressure, randomized signatures |

Read as a curriculum, the list is deliberate. SHA-2 is where the language first
meets fixed-width words and the rotations and additions of a compression
function. ChaCha20 adds the add-rotate-xor style that is beautiful precisely
because it is so regular, and Poly1305 adds arithmetic modulo a prime. AES-GCM
brings the first real pressure from hardware: an implementation that uses AES
and carry-less multiplication instructions must coexist with a portable one,
behind a dispatcher that is itself proved. The curve families bring field
arithmetic and canonical encodings, where many historical bugs lived. ML-KEM
and the signature family bring post-quantum standards whose errata are still
recent.

[D-015](DECISIONS.md#d-015--flagship-10-corpus) records this as a **proposed
set**. Exact membership is decided before the S7 stage admits the corpus.

### What admission requires

A corpus package is admitted only with a complete record. The
[package-admission rules](ASSURANCE.md#54-cryptography-package-admission) list
what every stable algorithm, construction, and profile needs:

- the exact normative publication, edition, errata snapshot, and source digest;
- clause-to-definition traceability and an honest transcription-review status;
- intellectual-property and transition or deprecation status;
- the mathematical specification and implementation-refinement evidence;
- a statement that separates implementation correctness from assumed hardness
  and game-based theorems;
- known-answer and intermediate-value vectors where available;
- negative, malformed, boundary, overlap, cross-endian, and
  authentication-failure cases;
- differential tests against mature implementations;
- adversarial corpora such as Wycheproof where applicable;
- source and target leakage evidence for every promised profile; and
- approved budgets for performance, memory, stack use, and proof replay.

The fifth item is easy to skip past and important not to. Proving that an
implementation of ML-KEM computes exactly what FIPS 203 specifies says nothing
about whether ML-KEM is secure. Security rests on hardness assumptions and on
game-based arguments that live in a different stratum and a different claim
family. A package report shows them separately, so that no reader mistakes a
refinement proof for a security proof.

### Failure is part of the algorithm

The corpus plan names failure behavior explicitly, and for good reason. How a
routine fails is often where its security lives. An AEAD decryption must
reject a forged tag without releasing plaintext and without revealing, through
timing, how much of the tag matched. ML-KEM goes further: under FIPS 203,
decapsulation of a correctly sized but invalid ciphertext does not return an
error at all. It
uses implicit rejection, returning a pseudorandom shared secret derived from a
secret value and the ciphertext, so that an attacker probing with invalid
ciphertexts learns nothing from the shape of the response.

A language that treats failure as an afterthought cannot express that
faithfully. Orange's corpus will force the question early: which failures are
observable, to whom, and under which leakage policy. The answer has to appear
in the specification, the implementation, the foreign contract, and the claims,
and it has to agree across all four.

### The scope rule

A solo project cannot build everything, and the corpus plan says what gives way
first. D-015's scope rule is short: under-resourcing removes a family or a
target rather than removing the proof, leakage, binary, interoperability, or
response gates while retaining the claim.

In other words, Orange may ship fewer algorithms, or support fewer processors,
but it may not ship the same algorithms with weaker evidence under the same
words. A smaller corpus with complete claims is a success. A larger corpus with
hollow claims is a failure that looks like a success, which is worse.

### Cryptanalysts as readers

The corpus has one more audience. Cryptanalysts read implementations looking
for the gap between what the standard says and what the code does: a missing
check, an unexpected branch, a buffer that holds a secret longer than it
should. Orange's corpus is meant to be pleasant for them to read. The
specification sits beside the implementation, the claims say exactly what was
proved and under which model, the assumptions are listed, and the evidence can
be replayed. A reviewer who finds a flaw should be able to say precisely which
claim it invalidates, and the system should be able to say which artifacts
depend on that claim.

### A first fixture, sketched

The smallest real piece of the corpus is probably the ChaCha20 quarter round
from RFC 8439, section 2.1. It operates on four 32-bit words:

```text
a += b; d ^= a; d <<<= 16;
c += d; b ^= c; b <<<= 12;
a += b; d ^= a; d <<<= 8;
c += d; b ^= c; b <<<= 7;
```

Here `+=` is addition modulo 2^32, `^=` is exclusive or, and `<<<=` is a left
rotation. Section 2.1.1 of the RFC gives a test vector: starting from
`a = 0x11111111`, `b = 0x01020304`, `c = 0x9b8d6f43`, and `d = 0x01234567`,
the quarter round produces `a = 0xea2a92f4`, `b = 0xcb1cf8ce`,
`c = 0x4581472e`, and `d = 0x5881c4bb`.

The compiler can already express those twelve operations, and with the
binding slice it names them the way the RFC does. Orange 2026 has no tuples
yet, so each output word is its own function. The first reads:

```orange
spec quarter_a(a: Word[32], b: Word[32], c: Word[32], d: Word[32]) -> Word[32] {
  let a1: Word[32] = a + b;
  let d1: Word[32] = (d ^ a1) <<< 16;
  let c1: Word[32] = c + d1;
  let b1: Word[32] = (b ^ c1) <<< 12;
  a1 + b1
}
```

Three more functions finish the round, four more apply it to the RFC's input
words, and `orangec eval` prints the RFC's output words exactly:

```text
chacha20::a: Word[32] = 0xea2a92f4
chacha20::b: Word[32] = 0xcb1cf8ce
chacha20::c: Word[32] = 0x4581472e
chacha20::d: Word[32] = 0x5881c4bb
```

That fixture tests the compiler. As a corpus fixture, the same twelve
operations would become an Orange specification transcribed from that clause,
with its provenance recorded. The vector would be imported with its source and
interpretation and evaluated against the specification by the reference
evaluator, not only against a fast implementation. And the fixture would stay in the conformance suite for as
long as the language exists. It would be a tiny claim, a conformance result for
one function on one vector, and it would be exactly as large as its evidence.

### Where things stand

No corpus package exists. The expression slice gave Orange the operations
SHA-2 and ChaCha20 are built from: addition modulo a word size, rotation,
shifts, exclusive or, and the other bitwise operations. The compiler's
fixtures already evaluate the SHA-256 round functions, round 0 of the "abc"
example, and the ChaCha20 quarter round against published values, and the
binding slice added named steps and the conversions that byte order needs. A
whole primitive is still out of reach. Orange 2026 has no tuples, no arrays or
byte strings, no loops, and no state, so a message schedule or sixty-four
rounds of compression cannot yet be written in any readable form. The corpus remains a set of research inputs rather than promises.

The acceptance test will run for the first time when a complete primitive can
be written in the specification stratum, admitted with its provenance, and
evaluated against its official vectors. That will be the corpus's first line.

## Chapter 13: Interoperability and External Validation

A self-contained toolchain is still surrounded by other systems. Its
specifications come from standards bodies. Its outputs are linked into C and
Rust programs. Its vectors are exchanged with validation services. Its
releases are described by software bills of materials and supply-chain
attestations. Its proofs may need to meet proofs written in other assistants.
And some of the most important statements about deployed cryptography are made
not by any tool but by accredited laboratories and certification programs.

This chapter is about those edges. Its governing boundary is simple and
**current**: Orange claims no certification and no external validation of any
kind. What it describes is how Orange intends to meet other systems without
confusing their evidence with its own.

### The edges

The [research analysis](RESEARCH.md#45-end-to-end-proof-still-needs-interoperability)
lists the places where even a complete Orange toolchain must meet existing
systems:

- standards text, errata, and official vectors;
- C ABI consumers and Rust packages;
- object formats, linkers, operating systems, and CPU feature discovery;
- NIST ACVP vector exchanges;
- SBOM, CBOM, and supply-chain attestations; and
- established proof libraries and independently checked exports.

The design principle for all of them is the same. Interoperability artifacts
are generated from the same definitions that produce the code, and they are
part of the claim graph. A hand-maintained header, a hand-written vector
converter, or a hand-copied assumption list recreates exactly the gap Orange
exists to close. Generated artifacts can still be wrong, but they are wrong in
one place, and that place is checked.

### Proofs from elsewhere

Some of the best cryptographic proofs in existence were written in other
systems. EasyCrypt and SSProve handle computational, game-based security
arguments that Orange's own game stratum will reach only gradually. Rather than
pretend those proofs do not exist, the architecture proposes first-class
adapters that import and export them.

The rule for such evidence is strict and simple: an external theorem remains
labeled external until its evidence is reconstructed into Orange's own proof
format and checked there. Its record names the external system, version,
theorem, and the assumptions that come with trusting it. A claim that rests on
it can be satisfied under a policy that permits external proofs, and a reader
sees exactly which checker was trusted. What the claim may not do is present
the external proof as if Orange's kernel had checked it.

### Validation is a different kind of evidence

Cryptographic validation programs occupy a distinct place. NIST's Cryptographic
Algorithm Validation Program tests algorithm implementations through the
Automated Cryptographic Validation Protocol, a black-box exchange of test
vectors and responses. The Cryptographic Module Validation Program, under FIPS
140-3, assesses complete cryptographic modules: a concrete boundary, build,
set of approved modes, entropy strategy, self-tests, and operational
environment, examined by an accredited laboratory.

Those are not proofs, and proofs are not those. The
[assurance model](ASSURANCE.md#11-external-validation-posture) draws the lines
carefully:

- a local ACVP-compatible test run is not an algorithm certificate;
- an algorithm certificate does not validate a complete module;
- the Orange language cannot be FIPS 140 validated in the abstract; and
- a concrete generated module, with its exact build, boundary, approved modes,
  entropy strategy, self-tests, version, platform, and environment, can be
  assessed externally.

The last point is the constructive one. Orange cannot be validated, but modules
built with Orange could be, and the toolchain should avoid choices that make
that assessment unnecessarily difficult. Clear module boundaries, reproducible
builds, precise entropy contracts, and machine-readable descriptions of approved
modes are all things a laboratory would ask for.

### Orange's posture

[D-016](DECISIONS.md#d-016--validation-and-certification-posture) records the
**proposed** posture:

- support ACVP-compatible input and output, and record validation status;
- never call local vectors or proof replay an ACVP or CAVP certificate;
- never call Orange itself FIPS 140 validated; and
- keep certificate-bearing profiles unsupported in the current solo operating
  model.

Accredited laboratory work is unavailable to a solo project. The claim model
has a precise way to say so: a claim whose policy requires a certificate is
`unsupported`, not `unresolved`, and certainly not `satisfied` on the strength
of passing vectors. Only a future explicit change to the operating model, with
a laboratory actually available, could open a certificate-bearing profile.

When external evidence does exist, the claim record keeps its authority where
it belongs. It records the issuer, scope, subject, dates, and digest of a
certificate or audit. It validates that metadata. It never turns the
laboratory's judgment into an Orange theorem, and it never extends the
certificate beyond the exact module and version it covers.

### Differential testing as everyday interoperability

Between formal proof and formal validation lies a great deal of practical
evidence, and Orange intends to use it. Differential tests compare Orange's
outputs against mature implementations of the same standard on the same
inputs. Adversarial suites such as Wycheproof feed carefully constructed
malformed and edge-case inputs whose correct result is known. Interoperability
tests check that a key encoded by Orange is accepted by other software and vice
versa.

In the claim model these are `test_run` evidence and empirical-test claims.
They are powerful at finding bugs, and they are honest about their scope: a
named corpus, a named method, a named environment, a named implementation and
target. They can satisfy an empirical claim. They cannot satisfy a claim that
requires proof, and they do not become validation because they came from a
well-known suite.

### When the answers differ

Differential testing is most informative when it fails. Suppose Orange's
output for some input disagrees with a mature implementation. The claim model
records a `not_satisfied` result for the differential claim, with the input,
both outputs, and both implementations identified. It does not decide in
advance which side is wrong.

Sometimes Orange will be wrong: a transcription error, a missed erratum, an
edge case in an encoding. Sometimes the other implementation will be wrong, or
will implement an older edition of the standard, or will be deliberately
lenient where the standard is strict. The investigation decides, and its
conclusion is recorded with its reasons. What the record never does is quietly
adjust Orange to match the majority. Agreement with other implementations is
evidence, not authority. The standard, read at its exact edition and errata,
is the authority.

### The supply-chain edge

Releases meet another set of external systems: software bills of materials in
SPDX and CycloneDX form, a cryptographic bill of materials that lists
algorithms and their parameters, SLSA and in-toto build provenance, and
signature and transparency evidence. The
[assurance model](ASSURANCE.md#91-framework-targets) proposes targets against
current frameworks such as NIST SSDF, SLSA, and the OpenSSF OSPS Baseline,
pinned to exact versions at release time.

Those frameworks are external too, and the same discipline applies. Meeting a
framework's requirements for a particular release is a claim about that
release. It is recorded with evidence and scope. It is not a property of the
project in general, and a repository control snapshot is not a certification.

### Where things stand

Orange currently exchanges nothing with any external system. It has no ACVP
adapter, no proof import or export, no SBOM generation, and no release. It
claims no certification, no validation, and no conformance to any external
framework. The repository does record its own CI and repository controls, and
it describes them as what they are: point-in-time observations about a solo
project, with the controls that require other people marked unavailable.

The interoperability story is therefore mostly a set of promises about how
evidence will be labeled. That may seem like a small thing to have settled
first. It is not. Most confusion about cryptographic assurance comes from
evidence that crossed an edge and lost its label on the way.

## Chapter 14: Evidence That Survives the Build

Most build systems produce artifacts and forget how. A compiler runs, a binary
appears, and the only record of what happened is a log that scrolls away or a
CI page that expires. When a question arises later (which source produced this
library, which compiler, which flags, which proof covered it), the answer has
to be reconstructed, if it can be reconstructed at all.

Orange proposes that a build produce two things: the artifact and the evidence
that says what the artifact is. This chapter describes the evidence: how it is
identified, what it contains, and how it stays attached to the bytes it
describes. The package, evidence, and release formats are all **proposed**.
The discipline of exact identity, however, already governs how the repository
records its own work.

### Names drift; digests do not

Chapter 2 made the point for claims: a function called `encrypt` can change
while keeping its name. The same is true of every other thing a build touches.
A package version number can be republished. A compiler binary called `orangec`
can be rebuilt with different flags. A vector file can be edited in place. A
standard's PDF can be silently corrected.

The remedy is content addressing. Every input and output is identified by a
cryptographic digest of its exact bytes. A claim binds to digests rather than
to names, so that when the bytes change, the identity changes, and any evidence
bound to the old identity no longer applies. Names stay for humans. Digests are
for the evidence.

Orange's architecture applies this throughout. Proofs bind to exact package
digests and theorem fingerprints, not to semantic-version ranges. The lock
file records each dependency's immutable identity and digest, its language and
Core-format editions, its exported theorem fingerprints and assumption
summaries, and its license and provenance metadata. A theorem fingerprint
includes the definitions, axioms, semantic edition, and target model the
theorem depends on, so a proof about one definition cannot silently attach to
another.

### One artifact, many identities

Consider a future claim that an object file implements the ChaCha20 quarter
round of RFC 8439 and satisfies a named leakage profile on one target. Each
noun in that sentence is an identity the evidence must pin. The object is a
digest. RFC 8439 is an exact document with an errata snapshot. The Orange
specification that transcribes it is a source digest and a theorem
fingerprint. The compiler is a binary digest built from a source digest with a
recorded toolchain. The leakage profile is a versioned policy name. The target
is a processor and ABI model at an exact version. The vectors used in testing
are a file digest with a recorded interpretation.

Change any one of those and the claim is about something else. A patched
compiler, an erratum applied, or a rebuilt object each produces a different
identity, and the old evidence no longer applies to the new thing until it is
re-established. That can feel strict. It is the only way a reader can know
that the evidence in front of them describes the bytes in front of them.

### Thin manifests and thick bundles

The [architecture](ARCHITECTURE.md#102-evidence-bundle) distinguishes two forms
of evidence:

- a **thin evidence manifest** content-addresses objects that may live
  elsewhere. It is convenient for development and online distribution, and it
  does not claim offline replay by itself.
- a **thick evidence bundle** contains every source, package, model, proof,
  certificate, tool, and build-critical byte needed for the replay it
  advertises. Claim-bearing releases require a thick bundle.

The proposed thick bundle, provisionally called `.orange-evidence`, contains a
canonical manifest that maps every content digest to its role, media type,
size, and replay requirement, together with a content-addressed store of the
actual bytes. Around that core it carries the claims and their
theorem-to-assumption graph; pass and translation certificates; compiler,
checker, solver, and target-model identities; standards, errata, and vector
provenance; object, header, wrapper, and ABI-contract digests; test summaries
and their machine-readable results; archival audit or validation material where
redistribution is permitted; SPDX and CycloneDX bills of materials, including a
cryptographic bill of materials; SLSA and in-toto build provenance; and
signature and transparency material.

The design goal is durability. The logical proof and build-critical chain must
remain replayable even if the registry, the transparency service, or an audit
URL disappears. A third-party certificate may still need its issuer as the
authority, and the bundle does not pretend otherwise; it preserves and
validates the metadata without pretending to machine-check an institution's
judgment.

### Determinism is part of the evidence

Evidence that cannot be reproduced is weak evidence. The proposed build rules
therefore fix the things that usually make builds vary: canonical ordering and
serialization, normalized paths, a pinned locale and timezone, declared seeds,
immutable tool digests, and `SOURCE_DATE_EPOCH` for timestamps. Search data
that affects output, such as profile-guided optimization data, becomes a
checked-in, hashed input rather than an ambient influence.

The repository already practices this at small scale. Its full check,
`scripts/ci/check-repository`, runs with an emptied environment, the `C`
locale, `TZ=UTC`, and `SOURCE_DATE_EPOCH=0`. The compiler's gate builds the
release binary twice in relocated directories and compares the results byte for
byte. Those are repeatability checks performed by one owner, and the project
labels them that way. They are not independent rebuilds, and they are not
release evidence. But they show that determinism is being engineered in from
the start, not retrofitted.

### The historical Gate 0 records

The repository contains a set of provisional evidence schemas from an earlier
planning phase: a claim record, an evidence manifest, a trust inventory, a
standards-provenance record, and a repository-control snapshot, with positive
and adversarial conformance fixtures. They demonstrate structural ideas that
this chapter describes: exact subjects, typed evidence bases, content digests
and sizes, replay profiles, trust closures, and supersession of corrected
records without mutating the originals.

They are explicitly historical and non-product. A fixture that passes one of
those schemas proves only that it has the right shape. The eventual package and
evidence formats will receive new identifiers and documented migrations; they
will not silently reinterpret those records.

### Evidence that can be corrected

Evidence is sometimes wrong. A transcription has an error, a certificate was
issued for the wrong digest, a test corpus was misinterpreted. The proposed
rule is that corrections never mutate history. A corrected record is a new
record that names the one it supersedes, and the old one remains addressable.
Claims that depended on the incorrect record are invalidated and re-derived,
and a report can show both the old conclusion and why it no longer holds.

That rule sounds bureaucratic until one considers the alternative. A system
that quietly edits its evidence cannot be audited, because an auditor can never
know whether the record they are reading is the one that was relied upon. An
append-only history with explicit supersession is what makes a later audit
meaningful.

### What the repository records today

Orange has no package format, no lock file, no evidence bundle, and no release.
What it does have is a habit. Accepted language slices are recorded with the
exact commit revision at which they were merged and the continuous-integration
runs that passed at that revision. Decisions name the revisions they bind.
Normative documents say which slice they belong to. The policy validator checks
repository invariants deterministically, and its findings name exact files and
rules.

That habit is the seed of the evidence system. When Orange begins to produce
artifacts, the question "what exactly is this, and what is known about it?" will
have been asked of every accepted change from the start, and the answer will
already have a shape.

## Chapter 15: Offline Replay and Trust Budgets

An assurance claim is only as useful as a skeptic's ability to check it. If
checking requires the vendor's servers, the vendor's build farm, or the
vendor's word, then the claim is a statement of trust in the vendor, however
many proofs stand behind it. Orange's charter lists the opposite as a product
principle: *independent replay is a feature*. A reviewer should be able to
unpack a proof bundle, inspect its manifest, and run the checker against
pinned inputs without a registry, a cloud service, or a source checkout.

This chapter describes that replay and the related idea of a trust budget.
Both are **product directions**, not current behavior. Orange has no bundle to
replay and no checker to replay it with.

### The auditor's journey

The clearest description of offline replay is the
[auditor's journey](USER_JOURNEYS.md#j-06--audit-and-replay-evidence-offline),
one of eight proposed end-to-end journeys that define Orange 1.0. An auditor
receives a thick evidence bundle and an independently obtained artifact
identity, and works in a clean environment with the network denied. The bundle's
bytes are treated as hostile until checked.

The proposed flow has six steps:

1. Verify the bundle's format and version, canonical paths, manifest identity,
   signatures, and content digests, and confirm that no undeclared or escaping
   files are present.
2. Enumerate the complete closure (sources, packages, models, proofs,
   certificates, tools, build inputs, artifacts, external evidence, axioms,
   assumptions, and trusted components) before executing anything.
3. Replay proofs and certificates with the authoritative and the
   implementation-diverse checkers, under deterministic resource bounds.
4. Rebuild or validate the advertised artifacts from declared inputs and
   compare their digests and claimed identities.
5. Re-run required tests, and inspect audit, laboratory, and external records
   for identity, scope, validity, and expiry, without presenting them as
   machine proofs.
6. Produce a machine-readable replay report and a human-readable claim and
   trust matrix listing every success, failure, non-claim, unresolved item, and
   invalidated dependent.

The fail-closed rule is the heart of it. Missing or extra bytes, an escaping
path, a digest or signature mismatch, checker disagreement, a failed proof or
build, expired external evidence, undeclared trust, resource exhaustion, or an
attempted network access prevents the affected claim from replaying
successfully. The auditor may inspect partial diagnostics, but receives no
generic green verdict.

### Replay is not reproduction by someone else

Orange is careful with the words it uses for repetition. The
[reproducibility contract](REPRODUCIBILITY.md#1-reproducibility-levels)
distinguishes four levels:

1. a **replayable method**, where inputs, tools, arguments, environment, and
   expected observations are recorded;
2. a **deterministic decision case**, where the same recorded environment and
   inputs reproduce the declared outputs;
3. an **independent decision reproduction**, where an identified reviewer
   repeats the case in an independently provisioned environment; and
4. **future release reproducibility**, where multiple independent builders
   reproduce published release artifacts.

A solo project can reach the second level on its own. The owner can run a case
twice, in two separately provisioned workspaces, and compare the results. That
is valuable evidence of determinism. It is not the third level, because the
same person provisioned both environments and chose what to compare. Orange
records such runs as same-owner replays and never relabels them as
independent. When a reviewer outside the project repeats a result, that will be
recorded as what it is, with the reviewer identified.

The distinction mirrors the rest of the book. A second run by the same person
is a better test, not a second witness.

### The trust budget

Every claim rests on something it does not prove: a checker implementation, a
set of axioms, a processor model, an ABI model, an operating system behavior, a
foreign contract. Chapter 1 argued that trust does not disappear. The trust
budget is how Orange proposes to make it visible and to keep it from growing
quietly.

The [assurance model](ASSURANCE.md#33-trust-budget) proposes that every release
report:

- the executable and source size of the authoritative checker;
- the accepted axioms, and why each is necessary;
- the modeled ISA, ABI, object, and leakage components;
- the external contracts and proof systems relied upon;
- the changes to the trusted computing base since the previous release; and
- which claims would be invalidated if a given component were compromised.

The last item turns a list into a tool. A trust budget that says "the ARM
target model is trusted" is informative. One that says "if the ARM target
model is wrong about conditional-select timing, these fourteen leakage claims
fall" is actionable. It tells a reader where to focus scrutiny and tells the
project what a single discovered flaw would cost.

The goal is not an arbitrary line-count threshold. It is a small, reviewable,
slowly changing closure with no undocumented expansion. A release whose trusted
base grew must say so, and say why.

### Trust per claim, not per project

The trust budget is computed per claim, not once for the whole project. The
architecture proposes a command, `orange trust`, that prints the closure of an
artifact or claim rather than a summary. A parser-behavior claim, a functional
refinement, and a native constant-time claim have genuinely different trusted
bases. The first depends on the parser and its specification; the last depends
on a leakage model, a target model, a compiler preservation argument, and a
processor. Folding them into one project-wide list would make the simple claims
look weaker and the hard claims look stronger than they are.

### Explaining invalidation

Replay is not only for auditors. A developer changing one definition wants to
know which proofs need to be redone and why. The proposed proof cache keys
results by the normalized obligation, imported theorem fingerprints, source and
Core-semantics editions, checker and decision-procedure versions, and the
target and leakage policy where relevant. When any of those changes, the cached
result no longer applies, and the command-line tools are meant to explain which
component invalidated it. That explanation is the same closure the trust budget
reports, seen from the other direction.

### What replay cannot tell you

Replay establishes that checking is reproducible: that the same inputs, run
through the same checkers under the same bounds, give the same verdicts. That
is a strong property, and it is not the only one a reader needs. Replay cannot
tell whether the Orange specification says what the standard's authors meant;
that is the intent boundary of
[Chapter 11](#chapter-11-standards-as-versioned-inputs). It cannot tell whether
a processor model matches the silicon it describes. It cannot tell whether a
claim's assumptions are reasonable in the reader's deployment.

What replay does is move those questions to where they can be seen. A replay
report lists every assumption, every model, and every trusted component that
the verdicts depended on. A cryptanalyst who doubts one of them does not have
to argue with the project's reputation. They can point at the exact item and
say which claims would fall with it.

### What exists now

None of this is implemented. Orange has no evidence bundle, no checker, no
`orange trust` command, and no release trust report. What exists is a
deterministic repository check, which runs in a sanitized environment with a
fixed locale, timezone, and timestamp, and a compiler whose output is
byte-for-byte repeatable. The repository's decision laboratories go one step
further in the same direction: they define replay plans and fresh-cache rules
for comparing candidates, even though no candidate has yet been executed.

The design choice underneath is that replay should be ordinary. It should not
require special access, special trust, or a special occasion. If every claim
Orange makes can be rechecked by anyone holding the bundle, then the project's
reputation is not what users rely on. The evidence is.

## Chapter 16: Solo Work Through Incremental Gates

Orange is built by one person. That sentence belongs near the end of the book
because every earlier chapter has depended on it, often explicitly. Each time
this book has said that a review is unavailable, that a rebuild is a same-owner
repetition, or that a certificate-bearing claim is unsupported, it was
describing the consequences of one fact about the project's circumstances.

This chapter describes how Orange works under that fact without letting it
distort either the engineering or the claims. The operating model is
**directed**: it is set by explicit owner decisions,
[D-023](DECISIONS.md#d-023--solo-project-operating-model) and the accepted
[OEP-0001](governance/oeps/OEP-0001-solo-development.md), and it controls the
repository today.

### The plan that assumed an institution

Orange's earliest planning described a staffed program. It had separate
language, proof, compiler, release, and security roles; a board for the trusted
computing base; independent auditors; external laboratories; and a Gate 0 that
required much of that institution to exist before permanent implementation
could begin. The documents were careful and ambitious. They were also written
for people who were not there.

On 2026-07-12 the owner withdrew the assumption. D-023 states that Orange is
developed as a solo project until the owner explicitly records otherwise, and
that no milestone may depend on contributors, independent reviewers, auditors,
laboratories, partner organizations, or separate release and incident-response
roles. The aggregate Gate 0 implementation embargo was superseded at its honest
state: zero of its seven institutional exit criteria had been met, and the
gate as a whole could not close without staff, reviewers, and organizations
that did not exist.

The easy mistake at that moment would have been to keep the old gate and wait,
or to drop the old gate and quietly keep its claims. Orange did neither.
OEP-0001 names the move precisely: it separates *work* from *claims*.
Solo-authored code can be tested, deterministic, documented, and suitable for
the permanent product lineage without being described as independently
reviewed, formally verified, certified, production-ready, or cryptographically
assured.

### Incremental capability gates

The replacement for the single barrier is a set of small ones. OEP-0001 gives
five rules:

1. Each component begins with a recorded purpose, boundary, deterministic test
   strategy, and explicit non-claims.
2. An unresolved decision gates only work that would make that decision
   irreversible or would depend on its result.
3. A component may ship only the claims supported by its current evidence.
4. Missing independent or external evidence is reported as unavailable or not
   claimed; it is not silently replaced by a second run from the same owner.
5. No future schedule, roadmap, or release plan may require outside
   participation unless the owner first records that participation as actually
   available.

The second rule does most of the work. Under the old model, an open question
about the proof foundation blocked the lexer. Under the new one, it blocks only
proof-bearing work. The lexer, the parser, the diagnostics, and the reference
evaluator have no dependency on D-006, so they may proceed. The leakage model
is open, so no constant-time claim is made, but that does not stop the compiler
from learning to read a module.

The first rule does the rest. A gate is not a date or a mood. It is a recorded
boundary with tests that say when it is closed and non-claims that say what
closing it does not mean.

### Owner review is not independent review

Solo development tempts a particular kind of dishonesty, usually unintended. A
careful owner reviews a change the next morning with fresh eyes, or runs the
same test on a second machine, or writes a second implementation to compare
against. Each of those is good practice. None of them is a second witness.

Orange's governance is explicit about this. The owner may author, review, and
approve the same change, and the record says so: every such decision is
labeled `owner-approved` or `solo-reviewed`, never `independently reviewed`.
Separate implementations written by the same owner provide differential
testing; they are implementation diversity, not organizationally independent
evidence. Two owner rebuilds are repeatability evidence, not independent
rebuilds. Existing historical schemas that have fields for external review keep
those fields' literal meaning, and solo records do not populate them with the
owner under another label.

This discipline costs very little and protects a great deal. A reader who
trusts Orange's labels can calibrate exactly how much weight to give a result.
A reader who discovered one relabeled review would have reason to doubt all of
them.

The governance record adds one more boundary: owner approval is valid
governance disposition, but it is never evidence that a technical statement is
true. Authority decides what the project will do. It cannot make a proof pass.

### The order of authority

With one person holding every role, it matters which record wins when two
disagree. [Governance](../GOVERNANCE.md#current-authority) gives the order:

1. explicit direction from the project owner;
2. directed decisions in the decision register;
3. accepted or provisional Orange Enhancement Proposals;
4. accepted or proposed architecture decision records; and
5. implementation.

Implementation is last on purpose. Code may explore a reversible local choice,
but it cannot silently stabilize semantics, widen a public claim, grant a
license, or override a higher record. When the compiler does something the
specification does not say, the answer is to fix one of them and record which,
not to let the compiler's behavior become the specification by default.

### Nine workstreams, one person

The roadmap still divides the work into distinct workstreams: product and
decisions, language and semantics, proof and metatheory, frontend and tools,
compiler and targets, the cryptography corpus, packages and releases, assurance
and conformance, and documentation. One person performs all of them. The
separation exists so that a result in one boundary cannot leak assurance into
another. A green parser test says something about parsing; it says nothing
about leakage, however many other checks happened to pass in the same run.

### The roadmap as a ladder

The [solo roadmap](ROADMAP.md#5-capability-stages) orders the work into
capability stages, each with a permanent outcome and an exit test:

| Stage | Capability | State |
| --- | --- | --- |
| S0 | Repository foundation | Closed for its solo scope |
| S1 | Compiler foundation: sources, lexer, diagnostics, CLI | Closed |
| S2 | Editioned grammar and bounded parser | Closed |
| S3 | Semantic core and reference evaluator | Active; S3a complete |
| S4 | Proof and claim boundary | Open |
| S5 | Compiler IRs and one output path | Open |
| S6 | Memory, leakage, ABI, and native targets | Open |
| S7 | Cryptography corpus | Open |
| S8 | Packages, developer tools, and preview releases | Open |
| 1.0 | Stable release gate | Open |

Progress is scored by closed gates only: three of ten, or 30 percent. Active
work earns no fractional credit, and nested slices earn no separate credit, so
the completed S3a slice advances S3 without closing it. The roadmap is explicit
that the percentage measures scope-gate closure. It is not an estimate of
remaining effort, not an assurance-strength score, and not a sign that a
release is near.

That scoring rule can look severe. It is meant to. A project that counts
partial work as partial completion will always appear to be further along than
it is, because the hardest part of any stage is usually its last part.

### Decision laboratories

Some questions are too large to answer by writing code and seeing what
happens. The semantic strata question of [Chapter 3](#chapter-3-one-language-several-semantic-worlds)
is one: choosing the wrong relationship between specification, implementation,
game, and machine semantics would be expensive to undo. For questions like
that, Orange builds decision laboratories.

A decision laboratory is an instrument for choosing, not a choice. It fixes
candidate answers, the cases that could distinguish them, the resource bounds
and replay rules for running those cases, and the result format, all before
any candidate is run. The D-003 product-form decision packet, a simpler
instrument of the same kind, has done its job: the
owner accepted a standalone Orange product form, and that decision is now
recorded at an exact revision. The D-004 laboratory is further back. Its
reviewed protocol describes five candidate graphs and a replay plan of 25
candidate-case units, each run three times, for 75 planned executions. None
has been executed, and D-004 remains proposed.

Laboratories let research run ahead of commitments without becoming
commitments. They also make the eventual decision auditable. When D-004 is
decided, the record will show which cases were run, what each candidate did,
and why one was chosen, rather than only which one won.

### Development does not freeze

The incremental model has a corollary that the owner restated on 2026-09-28:
Orange should not freeze development unless the owner specifically asks for it,
and older procedural rules are not standards for the project's current work.
An open decision blocks the capability that depends on it and nothing else. A
missing external review limits a claim and nothing else. When a check or a
record would stop unrelated work, that is a defect in the check or the record,
and the fix is to change it deliberately and say so, not to wait.

That direction does not loosen the claim discipline. It tightens the
distinction between the two things the old model conflated. Work may proceed
freely; claims may not. A slice that lands without an independent review is
recorded as solo-reviewed. A stage that closes without a proof is recorded as
closing without one. What is forbidden is not progress but mislabeling.

### Risks the model does not remove

The solo model has real costs, and Orange discloses them rather than arguing
them away. One person is a single point of failure for knowledge, credentials,
and continuity. Self-review misses what a second reviewer would catch. There is
no separation of duties between writing, accepting, and publishing a change,
and no multi-party custody of keys.

The mitigations are the ordinary ones, applied consistently: narrow changes,
deterministic tests, fail-closed diagnostics, protected history, an exact
dependency inventory, and preserved records that another maintainer could use
to recover the project. They reduce the risks. They do not remove them, and no
Orange claim will say otherwise.

If people later join the project, their participation is welcome but not
assumed. The owner would record a governance transition before granting any
decision, merge, release, security, or key-custody authority, and earlier work
would remain labeled as what it was. Arrival does not retroactively make solo
work independent.

## Chapter 17: Releases, Updates, and Failure

A release is a promise that outlives the moment it is made. Once bytes leave a
repository and enter someone else's build, the project no longer controls how
long they are used, what they are combined with, or what is later discovered
about them. For cryptographic software, the most important part of a release
process is therefore not the day of publication. It is what happens months
later, when a standard publishes an erratum, a dependency is compromised, or a
proof turns out to rest on a false lemma.

This chapter describes how Orange intends to release, update, and fail. Its
first fact is **current** and short: Orange has never released anything. No
source preview, toolchain preview, package, or binary is authorized, and a
merge, archive, CI artifact, or local build is not a release.

### What a release would be

The [release policy](../RELEASE_POLICY.md#release-classes) defines three
classes that the owner could later authorize through a recorded decision:

- a **source preview**: an immutable source snapshot for experimentation;
- a **toolchain preview**: owner-built binaries with exact provenance and
  explicit pre-alpha limitations; and
- a **stable toolchain**: a release whose complete supported behavior,
  compatibility, security, and support gates are recorded and satisfied.

No class is authorized merely because compiler code exists. Cryptographic or
proof-bearing packages add their own, stronger gates on top of whichever class
carries them.

### Identity has several axes

Most software identifies a release with a single version number. That is not
enough for Orange, because an Orange artifact's meaning depends on several
independently evolving things. The release policy binds each axis that
actually exists:

- the language edition;
- the Core and evidence format edition, where applicable;
- the toolchain version;
- the cryptography profile, where applicable; and
- the target, ABI, and leakage profile, where applicable.

The current compiler already shows the first axis. Every Orange source file
begins with `edition 2026;`, and a file without that exact marker is rejected.
The edition is not decoration. It lets the language change in a later edition
without silently changing the meaning of a program written for this one.

Each release would also carry one immutable identifier, exact source and
artifact digests, support dates, changed claims, deltas in its trusted
computing base and assumptions, known limitations, and a clear
`solo-produced` status. An axis that does not yet exist is listed as
unsupported, not omitted. A reader should never have to infer that a release
makes no leakage claim from the absence of a field.

### The solo release gate

When a preview is eventually authorized, the
[solo release gate](../RELEASE_POLICY.md#solo-release-gate) requires an explicit
owner decision and versioned scope; a frozen dependency graph and pinned
toolchain; a clean, network-disabled build where the toolchain permits it; two
separately provisioned owner rebuilds with byte comparison; every test,
conformance case, and security check that its exact claim matrix requires;
source and artifact digests, a dependency inventory, a software bill of
materials where applicable, and reproducible invocation records; a statement of
known limitations, unresolved findings, non-claims, support dates, and
vulnerability-reporting instructions; and a recorded procedure for publishing,
rolling back, withdrawing, and recovering the release.

"Frozen" in that list describes a dependency graph, not a project. It means the
exact inputs to one release are fixed so the release can be rebuilt. It does
not mean development stops.

As [Chapter 16](#chapter-16-solo-work-through-incremental-gates) explained, the
two owner rebuilds are repeatability evidence, not independent rebuilds. The
owner necessarily controls source acceptance, building, and publication, and a
solo release records that missing separation of duties as a residual risk
rather than hiding it.

### Publication never rewrites

The publication rules share a principle with the evidence rules of
[Chapter 14](#chapter-14-evidence-that-survives-the-build): what was published
stays published. Tags are annotated and immutable; force updates, deletion, and
tag reuse are prohibited. Every correction gets a new version. An artifact
already published under an identity is never replaced by different bytes under
the same identity.

Package registries follow the same rule in the proposed architecture. Published
versions are immutable. Yanking a version changes which version new resolution
selects; it does not mutate or delete the bytes that already exist, so a build
that pinned the old version can still be reproduced and audited. The
difference matters most in a crisis, when the temptation to make a bad release
quietly disappear is strongest.

Two open decisions currently block crate, package-registry, and binary
distribution. The outbound license under
[D-018](DECISIONS.md#d-018--licenses) is unselected, and the working name under
[D-017](DECISIONS.md#d-017--project-and-package-name) has no trademark
clearance. Until both are recorded for an exact release boundary, crate
publication, package-registry publication, and binary distribution are
prohibited. Local development by the owner is unaffected.

### Updates are claims too

A new version is not automatically a better one. The proposed
[update journey](USER_JOURNEYS.md#j-07--update-deprecate-withdraw-or-replace-a-profile)
treats every update, deprecation, or withdrawal as a change to a claim graph.
Its steps are to detect the event through authenticated metadata; resolve its
exact affected tuple, authority, urgency, and downstream claim impact; publish
an immutable replacement and migration path; verify signatures, thresholds,
freeze and rollback rules, and complete evidence before activating anything;
re-run the affected proof, build, conformance, ABI, and replay journeys; and
finally mark old versions supported, deprecated, yanked, withdrawn, or revoked
while preserving their historical replay material.

The fail-closed outcomes are what give the journey teeth. Unsigned metadata, an
attempted rollback or freeze, a missing dependency, an invalid migration,
unavailable evidence, ambiguous impact, or an unapproved claim downgrade
rejects the update. A withdrawn unsafe profile is never silently replaced with
a weaker one that keeps the old claims. Updating an algorithm profile never
retroactively strengthens an old evidence bundle.

### When something is wrong

Every serious project eventually discovers that something it shipped is wrong.
Orange's [assurance model](ASSURANCE.md#10-vulnerability-response) lists the
classes of failure specific to a verified cryptography toolchain:

- proof-system unsoundness;
- compiler miscompilation;
- target leakage-profile failure;
- standards nonconformance;
- an unsafe API or a misleading claim;
- build or update compromise;
- a malicious or taken-over package; and
- documentation that predictably induces cryptographic misuse.

The list is broader than most projects' definition of a vulnerability, and
deliberately so. A misleading claim is a vulnerability in a system whose
product is claims. Documentation that leads careful users to misuse an API is a
vulnerability even if every function behaves as specified.

The proposed
[response journey](USER_JOURNEYS.md#j-08--respond-to-a-vulnerability-or-invalidated-claim)
contains the report privately; reproduces it and identifies every affected
identity; stops publication and marks dependent claims invalid or unresolved
whenever the impact cannot be bounded; corrects every coupled artifact
(semantics, implementation, proof, compiler, vectors, documentation, claims,
and attestations) together; re-runs the affected evidence; and publishes an
immutable advisory that names which earlier claims are no longer valid.

One sentence from that journey deserves to be memorized: *wording changes alone
cannot repair missing or false assurance.* If a claim was wrong, softening the
adjective in a README does not fix it. The claim is invalidated, the evidence
is repaired or withdrawn, and the record shows both.

### Stop-ship

Some findings block a release outright. The
[assurance model](ASSURANCE.md#8-stop-ship-conditions) lists them: an
unresolved proof-soundness flaw; incorrect cryptographic output;
secret-dependent behavior within a promised target and leakage profile; an
undocumented axiom, trusted-base expansion, foreign boundary, or claim
downgrade; a semantic ambiguity that changes a valid program's meaning; failed
reproducibility, signature, provenance, update, or rollback protection; an
unresolved critical or high finding; an unreviewed standards erratum relevant
to a stable package; and an audit finding whose impact is not understood.

Security, soundness, and public-assurance gates cannot be waived. An exception
to an operational gate that carries no assurance meaning requires a named owner,
a rationale, a compensating control, an expiry, and disclosure in the release
notes. These conditions govern releases, not development: a known soundness
flaw stops a release, and the work to fix it proceeds.

### Support that can actually be given

[D-022](DECISIONS.md#d-022--support-policy) directs best-effort support by the
owner during pre-alpha, with no service-level agreement, long-term support
window, compatibility promise, or migration service. An earlier institutional
target of five plus two years of support is explicitly not an active
commitment. A release-specific support window may be adopted only when the
owner can actually sustain it, and every release must state its real support
dates and its single-maintainer risk.

Security response works the same way. The project targets acknowledgement of a
private report within one business day and an initial technical assessment
within three, as targets rather than a contract. There is no staffed security
team. Reports go through GitHub's private vulnerability reporting, described
in [the security policy](../SECURITY.md), and never into a public issue.

Support also attaches to the whole affected tuple rather than a single version
number: language edition, Core and evidence editions, toolchain release,
cryptography profile, target and leakage profile, package or artifact digest,
and operating environment. A security advisory that says only "versions before
1.4 are affected" is not precise enough for a system whose claims depend on
which processor model was assumed.

### The end of the beginning

The [project charter](PROJECT_CHARTER.md#9-what-end-means) says what "end"
means: not the end of maintenance, but the first stable, supportable 1.0
system. It lists ten conditions, from published and versioned semantics to an
exercised vulnerability-response process. After 1.0, new targets, leakage
models, proof automation, and algorithm packages are normal evolution. They do
not retroactively strengthen old claim bundles.

Orange is far from that gate, and this book has tried not to blur the
distance. What exists is a small compiler that checks and evaluates typed
literals, a large body of design, and a discipline for keeping the two apart.
The rest of the work is to close the gap one honest boundary at a time, so that
when Orange finally makes a promise about a piece of cryptography, every word
of it can be checked.

## Appendix A: Current Grammar and CLI

This appendix restates the implemented Orange 2026 surface for convenience.
The [lexical and grammar specification](LANGUAGE_2026.md) and the
[typed-literal semantics](SEMANTICS_2026.md) are normative, and the
[pure expression specification](EXPRESSIONS_2026.md) and the
[bindings and conversions specification](BINDINGS_2026.md) are proposed under
OEP-0005 and OEP-0006 and in the owner's review. Where this summary and those
documents differ, they control.

### Grammar

The parser accepts exactly this grammar, with at most two tokens of
lookahead:

```text
source_file     = edition_decl module_decl EOF ;
edition_decl    = "edition" "2026" ";" ;
module_decl     = "module" IDENTIFIER "{" function_decl* "}" ;
function_decl   = "spec" IDENTIFIER "(" ")" spec_tail
                | "spec" IDENTIFIER "(" parameters ")" typed_tail
                | "impl" IDENTIFIER "(" ")" empty_body ;
spec_tail       = empty_body | typed_tail ;
typed_tail      = "->" parsed_type "{" binding* expression "}" ;
binding         = "let" IDENTIFIER ":" parsed_type "=" expression ";" ;
empty_body      = "{" "}" ;
parameters      = parameter ("," parameter)* ","? ;
parameter       = IDENTIFIER ":" parsed_type ;
parsed_type     = IDENTIFIER ("[" INTEGER "]")? ;

expression      = arithmetic | chain("&") | chain("|") | chain("^") | shift
                | conversion ;
conversion      = prefixed "as" parsed_type ;
arithmetic      = product (("+" | "-") product)* ;
product         = prefixed ("*" prefixed)* ;
chain(op)       = prefixed (op prefixed)+ ;
shift           = prefixed shift_operator prefixed ;
shift_operator  = "<<" | ">>" | "<<<" | ">>>" ;
prefixed        = literal | ("-" | "~") prefixed | primary ;
literal         = "-"? INTEGER ;
primary         = IDENTIFIER | call | "(" expression ")" ;
call            = IDENTIFIER "(" arguments? ")" ;
arguments       = expression ("," expression)* ","? ;
```

Sources are valid UTF-8 of at most 16 MiB. Identifiers are ASCII. Integers
may be decimal, `0b` binary, or `0x` hexadecimal, with single underscores
between digits. `edition`, `module`, `spec`, `impl`, `game`, `proof`, and
`claim` are reserved; the last three have no grammatical role yet. `let` and
`as` are not reserved: `let` starts a binding only at the start of a body item
before a name, and `as` converts only after a complete operand. Line and
nested block comments are trivia. `<<`, `>>`, `<<<`, and `>>>` are single
tokens, matched longest first. Operators from different groups, or two shifts,
may not share a level without parentheses, and a conversion shares a level with
no operator and no other conversion. Expressions may nest at most 64 levels
deep and reach height 256; a function declares at most 64 parameters and 256
bindings, and a call supplies at most 256 arguments.

### Types and values

| Type | Values | Displayed as |
| --- | --- | --- |
| `Int` | All mathematical integers (unbounded); a literal's magnitude may use at most 16,384 significant bits | Decimal, with `-` when negative |
| `Word[8]` | The integers modulo 2^8, 0 through 255 | `0x` and 2 lowercase hex digits |
| `Word[16]` | The integers modulo 2^16 | `0x` and 4 lowercase hex digits |
| `Word[32]` | The integers modulo 2^32 | `0x` and 8 lowercase hex digits |
| `Word[64]` | The integers modulo 2^64 | `0x` and 16 lowercase hex digits |

No other type or width is accepted. Word literals are never wrapped, truncated,
saturated, or coerced, and no value changes type implicitly. `e as T` converts
between any two of these types: it takes the integer value of `e` and, for
`Word[n]`, its residue modulo 2^n. The operand's type comes from its first
name, call, or conversion, so a conversion of literals alone is an error.

### Operators

| Expression | On `Int` | On `Word[n]` |
| --- | --- | --- |
| `a + b`, `a - b`, `a * b` | Exact | Modulo 2^n |
| `-a` | Exact negation | Not defined; write `0 - a` |
| `a & b`, `a \| b`, `a ^ b` | Not defined | Bitwise and, or, exclusive or |
| `~a` | Not defined | Bitwise complement |
| `a << k`, `a >> k` | Not defined | Logical shift left, right |
| `a <<< k`, `a >>> k` | Not defined | Rotation left, right |

The amount `k` must be an unsigned integer literal from 0 through n − 1. Calls
name typed `spec` functions of the same module, pass exactly one argument per
parameter, and may not form a cycle. A `let` binding states its type, is in
scope after its semicolon, and may not reuse the name of a parameter or another
binding.

### Commands

```text
orangec [OPTIONS] <check|eval|lex> <FILE>...
```

| Command | Behavior |
| --- | --- |
| `check` | Lexical, syntactic, and semantic validation; silent on success |
| `eval` | Validate one source, then print each typed `spec` without parameters as `module::name: Type = value` |
| `lex` | Print the deterministic token stream with byte spans |

Options are `--edition <YEAR>` (only `2026`, at most once), `--` to end option
parsing, `-h` or `--help`, and `-V` or `--version`. A file name of `-` reads
UTF-8 source from standard input, once per invocation. Exit status is 0 on
success, 1 on a compile or input failure, and 2 on a usage error.

### Diagnostic families

| Codes | Phase | Examples |
| --- | --- | --- |
| `ORC0001`–`ORC0008` | Lexing | Unexpected character, unterminated comment or string, malformed integer, token budget |
| `ORC0101`–`ORC0108` | Parsing | Expected syntax, unsupported edition, trailing syntax, parser budget, ungrouped operators |
| `ORC0201`–`ORC0220` | Semantic analysis | Duplicate function, parameter, or binding, unsupported type or word width, negative or out-of-range word, magnitude limit, unknown name or function, name used before its binding, argument count, type mismatch, undefined operator, shift amount, call cycle, conversion operand without a type |
| `ORC0301` | Evaluation | Step budget, call depth, or `Int` result size exhausted |
| `ORC1001`–`ORC1008` | Command line | Unreadable or oversized input, invalid UTF-8, duplicate standard input, output limit |

Codes and their meanings are stable automation surfaces. Every resource budget
fails closed with a diagnostic rather than a panic, hang, or partial success.

## Appendix B: Decision Ledger

The [decision register](DECISIONS.md) is the authority. This ledger is a
snapshot of its statuses for readers who want the whole map at once. `directed`
means explicit owner direction with details still open; `accepted` means
ratified at an exact revision; `proposed` means a recommended answer awaiting
its gate; `investigate` means the alternatives need a reproducible comparison.

| Decision | Subject | Status |
| --- | --- | --- |
| D-001 | Mission | Directed |
| D-002 | No disposable prototype | Directed |
| D-003 | Product form | Accepted: standalone Orange (PF-01) |
| D-004 | Semantic strata | Proposed; decision laboratory prepared, not run |
| D-005 | Public assurance model | Proposed |
| D-006 | Proof foundation | Investigate |
| D-007 | Orange-owned proof format and checker | Proposed; depends on D-006 |
| D-008 | Implementation languages | Directed for the Rust bootstrap |
| D-009 | Solver trust | Proposed |
| D-010 | Compiler strategy | Investigate |
| D-011 | Initial native target envelope | Proposed |
| D-012 | Baseline leakage claim | Investigate |
| D-013 | Stable foreign boundary | Proposed |
| D-014 | Package and registry model | Proposed |
| D-015 | Flagship 1.0 corpus | Proposed set |
| D-016 | Validation and certification posture | Proposed |
| D-017 | Project and package name | Directed working codename; public name open |
| D-018 | Licenses | Directed development boundary; outbound license open |
| D-019 | Governance and release authority | Directed solo governance |
| D-020 | Supply-chain target | Proposed |
| D-021 | Self-hosting | Proposed |
| D-022 | Support policy | Directed best-effort solo support |
| D-023 | Solo project operating model | Directed |
| D-024 | Initial compiler foundation | Directed |
| D-025 | Orange 2026 minimal grammar and bounded parser | Directed |
| D-026 | Orange 2026 typed literal specifications | Directed |

Four Orange Enhancement Proposals are accepted: OEP-0001 (solo development and
incremental gates), OEP-0002 (the edition 2026 parser), OEP-0003 (typed
literals), and OEP-0004 (the standalone product form).

## Appendix C: Claim Vocabulary

These terms are used throughout the book. Most belong to the proposed public
assurance model, D-005, and will change if that decision changes.

**Statement kinds.** *Current* describes what the repository contains now.
*Directed* describes explicit owner direction. *Proposed* describes a
recommendation awaiting a decision. *Future* describes an intended capability
whose design, implementation, or evidence is incomplete.

**Claim.** A proposition about an exact subject, under identified contexts,
with stated assumptions and exclusions, supported by typed evidence and judged
by a policy. A claim is not a label attached to a project or a function name.

**Outcomes.** `satisfied`: every mandatory basis, context, identity binding, and
trust-closure element the claim policy requires is present, valid, and bound to
the same subject, and no valid decisive negative result exists.
`not_satisfied`: the proposition was checked and found false or violated.
`unresolved`: the system cannot presently decide it. `unsupported`: the
toolchain, model, target, or operating mode does not offer the claim.

**Evidence bases.** Kernel proofs, checked certificates, external proofs, test
runs, audits, external validations, and assumptions. Each keeps its own
authority. None becomes another by accumulation or relabeling.

**Assumption.** A named dependency that a claim does not prove, stated with why
it is needed and what fails if it is false.

**Exclusion.** A tempting interpretation that the claim's wording does not
cover, stated so the claim does not grow as it travels.

**Trust budget.** The per-claim closure of checkers, axioms, models, foreign
contracts, and other trusted components, including what a compromise of each
would invalidate.

**Leakage profile.** A named observation model, such as control flow and memory
addresses, together with a target model. A constant-time claim is meaningful
only relative to one.

**Solo-reviewed and owner-approved.** Labels for review or approval by the sole
owner. They are never written as independent review.

**Same-owner replay.** A repetition performed or provisioned by the owner.
Useful evidence of determinism; not independent reproduction.

**Thin manifest and thick bundle.** A manifest content-addresses evidence that
may live elsewhere; a bundle contains every byte needed for the replay it
advertises.

**Edition.** A versioned language surface, selected by `edition 2026;` today, so
that later language changes do not silently change the meaning of older
programs.

## Appendix D: Source Notes

This book is a synthesis of repository sources. The principal sources for each
part are listed here so a reader can move from explanation to authority.

- **Chapters 1 through 3:** the [project charter](PROJECT_CHARTER.md), the
  [research analysis](RESEARCH.md), the [assurance model](ASSURANCE.md), the
  [product-form decision packet](PRODUCT_FORM_DECISION_PACKET.md), and the
  [semantic strata decision suite](SEMANTIC_STRATA_DECISION_SUITE.md).
- **Chapters 4 and 8:** the [grammar specification](LANGUAGE_2026.md), the
  [typed-literal semantics](SEMANTICS_2026.md),
  [OEP-0003](governance/oeps/OEP-0003-orange-2026-typed-literals.md), the
  [compiler guide](../compiler/README.md), and the compiler's own behavior at
  the book's snapshot.
- **Chapters 5 and 6:** the [architecture](ARCHITECTURE.md), the
  [assurance model](ASSURANCE.md), the
  [proof foundation](PROOF_FOUNDATION_DECISION_SUITE.md) and
  [solver trust](SOLVER_TRUST_DECISION_SUITE.md) decision suites, and the
  [threat model](security/THREAT_MODEL.md).
- **Chapters 7 and 16:** the [decision register](DECISIONS.md),
  [OEP-0001](governance/oeps/OEP-0001-solo-development.md),
  [governance](../GOVERNANCE.md), the [roadmap](ROADMAP.md), and the owner's
  recorded directions.
- **Chapters 9 and 10:** the [architecture](ARCHITECTURE.md), the
  [compiler strategy decision suite](COMPILER_STRATEGY_DECISION_SUITE.md), and
  the [user journeys](USER_JOURNEYS.md).
- **Chapters 11 through 13:** the [research analysis](RESEARCH.md), the
  [reproducibility contract](REPRODUCIBILITY.md), and the
  [assurance model](ASSURANCE.md).
- **Chapters 14, 15, and 17:** the [architecture](ARCHITECTURE.md), the
  [reproducibility contract](REPRODUCIBILITY.md), the
  [user journeys](USER_JOURNEYS.md), the
  [release policy](../RELEASE_POLICY.md), the
  [security policy](../SECURITY.md), and the [support policy](../SUPPORT.md).

External standards, programs, and tools named in the book, such as FIPS
publications, ACVP, FIPS 140-3, Wycheproof, SLSA, SPDX, and CycloneDX, are
referenced for orientation only. The book imports none of them as normative
input, and any future use would carry the exact provenance described in
[Chapter 11](#chapter-11-standards-as-versioned-inputs).

## Manuscript map

The map records the manuscript's state. It is not an architecture decision or
delivery schedule. Every planned chapter now has a first draft; chapters will
be revised as the normative design changes, and a chapter's governing boundary
controls how far its prose may go.

| Part | Chapter | State | Governing boundary |
| --- | --- | --- | --- |
| I — Why Orange | 1. The Seams Are the System | Drafted in v0.1; revised in v0.5 | Directed mission; current limits; proposed claim-oriented graph |
| I — Why Orange | 2. Claims, Not Labels | Drafted in v0.2 | Public claim model remains proposed; current evidence boundaries are directed |
| I — Why Orange | 3. One Language, Several Semantic Worlds | Drafted in v0.3; revised in v0.5 | PF-01 product form accepted at exact revision `a82a5cec2ee4359dc2fe66171f17c93146747333`; semantic strata remain proposed |
| II — Meaning and Trust | 4. From Surface Text to Meaning | Drafted in v0.3; revised in v0.5 | Accepted typed-literal Core and evaluator exist; expression and binding slices implemented, specifications in review; complete semantic Core remains open |
| II — Meaning and Trust | 5. Proof Search Is Not Proof Checking | Drafted in v0.3 | Proof foundation and checker remain unsettled |
| II — Meaning and Trust | 6. Secrets Are a Semantic Concern | Drafted in v0.3 | Leakage baseline and target models remain unsettled |
| III — Building the Language | 7. No Disposable Prototype | Drafted in v0.3 | Directed production-lineage doctrine |
| III — Building the Language | 8. Orange 2026: The Smallest Honest Slice | Drafted in v0.3; revised in v0.5 | Current parser, accepted typed-literal semantics, and the proposed expression and binding slices |
| III — Building the Language | 9. From Core to Native Bytes | Drafted in v0.3; revised in v0.4 | Compiler strategy and targets remain proposed |
| III — Building the Language | 10. The Foreign Boundary | Drafted in v0.3 | ABI and generated interfaces remain proposed |
| IV — Cryptography in Practice | 11. Standards as Versioned Inputs | Drafted in v0.3; revised in v0.4 | Exact source and rights decisions are required |
| IV — Cryptography in Practice | 12. The Corpus as Acceptance Test | Drafted in v0.3; revised in v0.5 | Flagship corpus remains proposed |
| IV — Cryptography in Practice | 13. Interoperability and External Validation | Drafted in v0.3 | No certification or external validation is claimed |
| V — Operating Orange | 14. Evidence That Survives the Build | Drafted in v0.3 | Package, evidence, and release formats remain proposed |
| V — Operating Orange | 15. Offline Replay and Trust Budgets | Drafted in v0.3 | Replay is a product direction, not current behavior |
| V — Operating Orange | 16. Solo Work Through Incremental Gates | Drafted in v0.3 | Directed solo operating model |
| V — Operating Orange | 17. Releases, Updates, and Failure | Drafted in v0.3 | No release is currently authorized |
| Appendices | A. Current Grammar and CLI; B. Decision Ledger; C. Claim Vocabulary; D. Source Notes | Drafted in v0.3; Appendix A revised in v0.5 | Must track the normative repository state |

## Sources and drafting disclosure

This manuscript is an explanatory synthesis of repository-local material. Its
principal sources for version 0.1 are:

- the [project charter](PROJECT_CHARTER.md) for mission, users, scope, and
  engineering doctrine;
- the [research and landscape analysis](RESEARCH.md) for the polyglot seam and
  vertical-artifact framing;
- the [assurance and security model](ASSURANCE.md) for independent claim
  dimensions, evidence bases, and trust boundaries;
- the [decision register](DECISIONS.md) for the distinction between directed,
  proposed, investigative, and unresolved choices;
- the [dependency-ordered roadmap](ROADMAP.md) for current capability status;
- the [Orange 2026 lexical and grammar specification](LANGUAGE_2026.md) for the
  normative parser boundary;
- the [accepted typed-literal semantics](SEMANTICS_2026.md) and
  [OEP-0003](governance/oeps/OEP-0003-orange-2026-typed-literals.md) for the
  bounded S3a meaning and non-claims; and
- the [compiler guide](../compiler/README.md) for implemented CLI behavior.

Version 0.3 adds, among others, the
[architecture](ARCHITECTURE.md), [reproducibility contract](REPRODUCIBILITY.md),
[user journeys](USER_JOURNEYS.md), [threat model](security/THREAT_MODEL.md),
[OEP-0001](governance/oeps/OEP-0001-solo-development.md),
[governance](../GOVERNANCE.md), [release policy](../RELEASE_POLICY.md), and the
decision suites under `docs/`. Version 0.4 adds the
[pure expression specification](EXPRESSIONS_2026.md) and
[OEP-0005](governance/oeps/OEP-0005-orange-2026-pure-spec-expressions.md), and
version 0.5 adds the
[bindings and conversions specification](BINDINGS_2026.md) and
[OEP-0006](governance/oeps/OEP-0006-orange-2026-bindings-and-conversions.md).
Appendix D lists the principal sources for each chapter.

Initial manuscript version 0.1—the structure, preface, manuscript map, and
Chapter 1—was drafted with OpenAI Codex, based on GPT-5, under Chase Bryan's
direction on 2026-07-12. Chase Bryan is the named author and remains accountable
for review, correctness, provenance, and future revisions. AI-assisted prose is
not a primary source, proof, independent review, or license provenance.

Manuscript version 0.2 added Chapter 2, drafted with OpenAI Codex, based on
GPT-5, under Chase Bryan's direction on 2026-07-14. The same authorship, review,
evidence, and provenance boundaries apply.

Manuscript version 0.3 added Chapters 3 through 17 and Appendices A through D,
revised the preface and contents, and updated this map. It was drafted with
Claude Code, Anthropic's coding agent, under Chase Bryan's direction on
2026-09-28. Each new chapter was cross-checked during drafting against the
repository sources it cites; that check is AI-assisted consistency review, not
independent review. The same authorship, review, evidence, and provenance
boundaries apply.

Manuscript version 0.4 revised the preface, Chapters 1, 3, 4, 8, 9, 11, and 12,
and Appendix A for the S3b expression slice. It was drafted with Claude Code
under Chase Bryan's direction on 2026-09-28, and every Orange example it adds
was run against the compiler at the revision that introduced it. That check is
not independent review, and the same authorship, review, evidence, and
provenance boundaries apply.

Manuscript version 0.5 revised the preface, Chapters 1, 3, 4, 8, and 12, and
Appendix A for the S3c binding and conversion slice, and added the Chapter 8
section "Naming steps and changing types". It was drafted with Claude Code
under Chase Bryan's direction on 2026-09-28, and every Orange example it adds
was run against the compiler at the revision that introduced it. That check is
not independent review, and the same authorship, review, evidence, and
provenance boundaries apply.

The repository has no selected outbound documentation license under D-018. No
license or redistribution grant should be inferred from this manuscript.
