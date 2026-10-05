# The Orange Reference Manual

<img src="images/orange-reference-manual-cover.png" width="400" alt="The Orange Reference Manual cover: orange emblem and oversized ORANGE lettering on black.">

By Chase Bryan

Status: normative Orange 2026 reference manual and 1.0 language architecture specification

Snapshot: 2026-10-05

Edition: `2026`

---

> This manual is the definitive, exhaustive, and mathematically rigorous reference
> specification for the Orange programming language, its multi-strata semantic
> architecture, formal type system, small-step and big-step operational semantics,
> deductive proof foundation, cryptographic leakage models, assurance claim graph,
> foreign interface, and diagnostic taxonomy. It authoritatively defines the normative
> grammar, static semantics, and dynamic execution behavior of Orange Edition `2026`
> as implemented in the authoritative compiler toolchain (`orangec`), alongside the
> formal 1.0 product architecture uniting pure mathematical specifications (`spec`),
> terminating imperative implementations (`impl`), target-modeled machine code
> (`machine impl`), probabilistic cryptographic games (`game`), deductive proof terms
> (`proof`), and atomic assurance contracts (`claim`).

---

## Contents

- [Preface](#preface)
  - [§1. Mission, Doctrine, and Philosophical Basis](#1-mission-doctrine-and-philosophical-basis)
  - [§2. Conformance, Formality, and Normative Terminology](#2-conformance-formality-and-normative-terminology)
  - [§3. Mathematical Notations and Typographical Conventions](#3-mathematical-notations-and-typographical-conventions)
  - [§4. Language Status Register and Slice Traceability](#4-language-status-register-and-slice-traceability)

- [Part I: Lexical & Concrete Syntactic Grammar](#part-i-lexical--concrete-syntactic-grammar)
  - [§5. Byte-Level Encoding, Normalization Invariants, and Source Limits](#5-byte-level-encoding-normalization-invariants-and-source-limits)
  - [§6. Lexical Analysis, Maximal Munch, and State-Machine Tokenization](#6-lexical-analysis-maximal-munch-and-state-machine-tokenization)
  - [§7. Whitespace, Trivia Preservation, and Formatting Idempotence](#7-whitespace-trivia-preservation-and-formatting-idempotence)
  - [§8. Nested Block and Line Comments](#8-nested-block-and-line-comments)
  - [§9. Identifiers, Reserved Words, and Contextual Keywords](#9-identifiers-reserved-words-and-contextual-keywords)
  - [§10. Numeric Literals: Radices, Magnitudes, and Underscore Grammar](#10-numeric-literals-radices-magnitudes-and-underscore-grammar)
  - [§11. Byte Sequences, Escape Grammar, and Hexadecimal Strings](#11-byte-sequences-escape-grammar-and-hexadecimal-strings)
  - [§12. Punctuation, Delimiters, Operators, and Lexical Budgets](#12-punctuation-delimiters-operators-and-lexical-budgets)

- [Part II: Compilation Units and Module Architecture](#part-ii-compilation-units-and-module-architecture)
  - [§13. Compilation Units, Edition Invariants, and Header Syntax](#13-compilation-units-edition-invariants-and-header-syntax)
  - [§14. Module Enclosures, Top-Level Member Ordering, and Syntax Trees](#14-module-enclosures-top-level-member-ordering-and-syntax-trees)
  - [§15. Hermetic Module Resolution and Filesystem Path Mapping](#15-hermetic-module-resolution-and-filesystem-path-mapping)
  - [§16. Import Dependency Graphs and Tarjan Cycle Elimination](#16-import-dependency-graphs-and-tarjan-cycle-elimination)
  - [§17. Type Aliases, Structural Equivalence, and Shadowing Restrictions](#17-type-aliases-structural-equivalence-and-shadowing-restrictions)
  - [§18. Multi-Strata Architectural Separation and Disjoint Namespaces](#18-multi-strata-architectural-separation-and-disjoint-namespaces)

- [Part III: Formal Type System & Algebraic Foundations](#part-iii-formal-type-system--algebraic-foundations)
  - [§19. Semantic Value Domains and Mathematical Universes](#19-semantic-value-domains-and-mathematical-universes)
  - [§20. Mathematical Integers ($\mathbb{Z}$) and the `Int` Type](#20-mathematical-integers-mathbbz-and-the-int-type)
  - [§21. Word Rings: Algebraic Foundations of $\mathbb{Z}/2^W\mathbb{Z}$ ($W \in \{8, 16, 32, 64\}$)](#21-word-rings-algebraic-foundations-of-mathbbz2wmathbbz-w-in-8-16-32-64)
  - [§22. Residue Fields and Modular Arithmetic ($\mathbb{Z}/m\mathbb{Z}$ for $2 \le m \le 2^{521}-1$)](#22-residue-fields-and-modular-arithmetic-mathbbzmmathbbz-for-2-le-m-le-2521-1)
  - [§23. Boolean Truth Values and Propositional Logic (`Bool`)](#23-boolean-truth-values-and-propositional-logic-bool)
  - [§24. Fixed-Length Array Spaces ($T^n$ for $1 \le n \le 65,536$)](#24-fixed-length-array-spaces-tn-for-1-le-n-le-65536)
  - [§25. Multi-Dimensional Rectangular Matrices and Nested Arrays](#25-multi-dimensional-rectangular-matrices-and-nested-arrays)
  - [§26. Heterogeneous Product Types: Tuples ($(T_0, \dots, T_{k-1})$ for $2 \le k \le 16$)](#26-heterogeneous-product-types-tuples-t_0-dots-t_k-1-for-2-le-k-le-16)
  - [§27. Byte Arrays (`Byte` and `Word[8]^n`)](#27-byte-arrays-byte-and-word8n)
  - [§28. Explicit Value Conversions (`expr as T`) and Typing Judgments](#28-explicit-value-conversions-expr-as-t-and-typing-judgments)
  - [§29. Endianness Homomorphisms and Bit-Preserving Packing (`as big T`, `as little T`)](#29-endianness-homomorphisms-and-bit-preserving-packing-as-big-t-as-little-t)
  - [§30. Dependent Finite Size Parameters ($[n \in \text{low}..\text{high}]$) and Monomorphization](#30-dependent-finite-size-parameters-n-in-textlowtextfield-and-monomorphization)
  - [§31. Finite Type Parameter Domains ($[K \in \{T_1, \dots, T_m\}]$)](#31-finite-type-parameter-domains-k-in-t_1-dots-t_m)

- [Part IV: Static Semantics (Typing Rules & Judgments)](#part-iv-static-semantics-typing-rules--judgments)
  - [§32. Typing Contexts: Signature ($\Sigma$), Size ($\Theta$), Type ($\Delta$), and Variable ($\Gamma$) Environments](#32-typing-contexts-signature-sigma-size-theta-type-delta-and-variable-gamma-environments)
  - [§33. Subtyping, Coercion Freedom, and Structural Type Equality](#33-subtyping-coercion-freedom-and-structural-type-equality)
  - [§34. Expression Typing Rules and Formal Judgments](#34-expression-typing-rules-and-formal-judgments)
  - [§35. Expression Grouping Envelopes and Syntactic Ambiguity Rejection](#35-expression-grouping-envelopes-and-syntactic-ambiguity-rejection)
  - [§36. Static Array Bounds Verification and Abstract Interval Analysis](#36-static-array-bounds-verification-and-abstract-interval-analysis)
  - [§37. Data-Dependent Indexing, Range Narrowing, and S-Box Lookups](#37-data-dependent-indexing-range-narrowing-and-s-box-lookups)

- [Part V: Dynamic Semantics (Operational & Reduction Rules)](#part-v-dynamic-semantics-operational--reduction-rules)
  - [§38. Abstract Syntax and Typed Reference Core Lowering](#38-abstract-syntax-and-typed-reference-core-lowering)
  - [§39. Evaluation Environments, Value Stores, and Step Budgets](#39-evaluation-environments-value-stores-and-step-budgets)
  - [§40. Small-Step Operational Semantics (SOS) and Big-Step Reduction](#40-small-step-operational-semantics-sos-and-big-step-reduction)
  - [§41. Operational Reduction of Variable Shifts, Rotations, and Inversions](#41-operational-reduction-of-variable-shifts-rotations-and-inversions)
  - [§42. Bounded Iteration Semantics and Step-Cost Accounting](#42-bounded-iteration-semantics-and-step-cost-accounting)
  - [§43. Known-Answer Specification Tests (`test`) and Whole-Aggregate Equality](#43-known-answer-specification-tests-test-and-whole-aggregate-equality)

- [Part VI: Formal Metatheory of the Specification Stratum](#part-vi-formal-metatheory-of-the-specification-stratum)
  - [§44. Metatheorem 1: Strong Normalization and Totality](#44-metatheorem-1-strong-normalization-and-totality)
  - [§45. Metatheorem 2: Type Safety (Progress and Subject Reduction)](#45-metatheorem-2-type-safety-progress-and-subject-reduction)
  - [§46. Metatheorem 3: Determinism and Semantic Confluence](#46-metatheorem-3-determinism-and-semantic-confluence)
  - [§47. Metatheorem 4: Endianness Packing Isomorphism](#47-metatheorem-4-endianness-packing-isomorphism)

- [Part VII: Specification Corpus & Reference Cryptographic Standards](#part-vii-specification-corpus--reference-cryptographic-standards)
  - [§48. Mathematical Transcription Methodology and Traceability](#48-mathematical-transcription-methodology-and-traceability)
  - [§49. Complete Reference Specification: FIPS 180-4 SHA-256](#49-complete-reference-specification-fips-180-4-sha-256)
  - [§50. Complete Reference Specification: RFC 8439 ChaCha20](#50-complete-reference-specification-rfc-8439-chacha20)
  - [§51. Complete Reference Specification: Curve25519 / X25519 (RFC 7748)](#51-complete-reference-specification-curve25519--x25519-rfc-7748)
  - [§52. Complete Reference Specification: Poly1305 Field MAC (RFC 8439)](#52-complete-reference-specification-poly1305-field-mac-rfc-8439)

- [Part VIII: Implementation Stratum (`impl`) & Memory Model](#part-viii-implementation-stratum-impl--memory-model)
  - [§53. Imperative Execution Semantics and Place Logic](#53-imperative-execution-semantics-and-place-logic)
  - [§54. Structured Memory Model: Heap-Freedom, Regions, and Stack Layout](#54-structured-memory-model-heap-freedom-regions-and-stack-layout)
  - [§55. Affine Ownership, Move Semantics, and Capability Borrowing ($\&T, \&\text{mut } T$)](#55-affine-ownership-move-semantics-and-capability-borrowing-t-mut-t)
  - [§56. Separation Logic Foundation and Non-Aliasing Invariants](#56-separation-logic-foundation-and-non-aliasing-invariants)
  - [§57. In-Place Mutable Slicing and Disjointness Proof Obligations](#57-in-place-mutable-slicing-and-disjointness-proof-obligations)
  - [§58. Two-State Contracts: Preconditions (`requires`), Postconditions (`ensures`), and `old(...)`](#58-two-state-contracts-preconditions-requires-postconditions-ensures-and-old)
  - [§59. Loop Invariants and Well-Founded Termination Variants](#59-loop-invariants-and-well-founded-termination-variants)
  - [§60. Typed Failure, Result Enums, and Total Panic Freedom](#60-typed-failure-result-enums-and-total-panic-freedom)
  - [§61. Storage Zeroization, Stack Scrubbing, and Memory Erasure (`erase`)](#61-storage-zeroization-stack-scrubbing-and-memory-erasure-erase)

- [Part IX: Machine Implementation Stratum (`machine impl`)](#part-ix-machine-implementation-stratum-machine-impl)
  - [§62. Target Machine Modeling and Register Capabilities](#62-target-machine-modeling-and-register-capabilities)
  - [§63. Fixed-Width Vector Types (`Vec128`, `Vec256`, `Vec512`) and SIMD Semantics](#63-fixed-width-vector-types-vec128-vec256-vec512-and-simd-semantics)
  - [§64. Hardware Cryptographic Intrinsics (AES-NI, ARMv8 Crypto, PCLMULQDQ, PMULL, SHA-NI, Zkne)](#64-hardware-cryptographic-intrinsics-aes-ni-armv8-crypto-pclmulqdq-pmull-sha-ni-zkne)
  - [§65. Target Profiles: x86-64 System V, AArch64 AAPCS64, RV64GC](#65-target-profiles-x86-64-system-v-aarch64-aapcs64-rv64gc)
  - [§66. Instruction Latency Classification and Hardware Data-Independent Timing](#66-instruction-latency-classification-and-hardware-data-independent-timing)

- [Part X: Information Flow, Secrecy & Microarchitectural Leakage](#part-x-information-flow-secrecy--microarchitectural-leakage)
  - [§67. The Information Flow Lattice ($\text{public} \sqsubseteq \text{secret}$)](#67-the-information-flow-lattice-textpublic-sqsubseteq-textsecret)
  - [§68. Microarchitectural Trace Semantics and Observational Noninterference](#68-microarchitectural-trace-semantics-and-observational-noninterference)
  - [§69. Architectural Noninterference Policy: `ct-architectural-v1`](#69-architectural-noninterference-policy-ct-architectural-v1)
  - [§70. Hardware ALU Latency Policy: `ct-variable-latency-v1` (DOITM, DIT, Zkt)](#70-hardware-alu-latency-policy-ct-variable-latency-v1-doitm-dit-zkt)
  - [§71. Speculative Noninterference Policy: `ct-speculative-v1`](#71-speculative-noninterference-policy-ct-speculative-v1)
  - [§72. Declassification Contracts, Audit Ledgers, and Flow Gates](#72-declassification-contracts-audit-ledgers-and-flow-gates)

- [Part XI: Cryptographic Game Stratum (`game`)](#part-xi-cryptographic-game-stratum-game)
  - [§73. Monadic Probabilistic Semantics and Distribution Ensembles](#73-monadic-probabilistic-semantics-and-distribution-ensembles)
  - [§74. Stateful Oracles, Adversaries, and Black-Box Encapsulation](#74-stateful-oracles-adversaries-and-black-box-encapsulation)
  - [§75. Sequence-of-Games Reductions and Concrete Advantage Bounding](#75-sequence-of-games-reductions-and-concrete-advantage-bounding)

- [Part XII: Deductive Proof System & Metatheory (`proof`)](#part-xii-deductive-proof-system--metatheory-proof)
  - [§76. Propositions as Types and the $\text{Prop}$ Universe](#76-propositions-as-types-and-the-textprop-universe)
  - [§77. Functional Refinement Relations ($\text{impl } P \sqsubseteq \text{spec } S$)](#77-functional-refinement-relations-textimpl-p-sqsubseteq-textspec-s)
  - [§78. Proof IR: Canonical Encoding, De Bruijn Terms, and Cryptographic Fingerprints](#78-proof-ir-canonical-encoding-de-bruijn-terms-and-cryptographic-fingerprints)
  - [§79. Weakest Precondition Calculus $\text{wp}(S, Q)$ and Verification Conditions](#79-weakest-precondition-calculus-textsps-q-and-verification-conditions)
  - [§80. Certificate-Producing Automation: LRAT/DRAT and LFSC/SMT-LIB Reconstruction](#80-certificate-producing-automation-lratdrat-and-lfscsmt-lib-reconstruction)
  - [§81. Authoritative Offline Checker: `orange-check` Kernel and TCB Boundary](#81-authoritative-offline-checker-orange-check-kernel-and-tcb-boundary)

- [Part XIII: Assurance Claims & Evidence Architecture (`claim`)](#part-xiii-assurance-claims--evidence-architecture-claim)
  - [§82. The Philosophy of Atomic Claims ("Claims, Not Labels")](#82-the-philosophy-of-atomic-claims-claims-not-labels)
  - [§83. The Ten Mandatory Claim Families (CF-01 through CF-10)](#83-the-ten-mandatory-claim-families-cf-01-through-cf-10)
  - [§84. Claim Record Schema, 4-Valued Outcome Algebra, and Evidentiary Bases](#84-claim-record-schema-4-valued-outcome-algebra-and-evidentiary-bases)
  - [§85. Evidence Bundles: Thin Manifests and Thick Content-Addressed `.orange-evidence`](#85-evidence-bundles-thin-manifests-and-thick-content-addressed-orange-evidence)
  - [§86. CBOM (CycloneDX 1.6), SPDX SBOM, and SLSA Level 3/4 Build Provenance](#86-cbom-cyclonedx-16-spdx-sbom-and-slsa-level-34-build-provenance)
  - [§87. The Minimal TCB Calculus (`orange trust`)](#87-the-minimal-tcb-calculus-orange-trust)

- [Part XIV: Foreign Function Interface & ABI](#part-xiv-foreign-function-interface--abi)
  - [§88. Sound Foreign Interface Principles and Import Contracts](#88-sound-foreign-interface-principles-and-import-contracts)
  - [§89. Standard C ABI Layouts, Packing, and Alignment (x86-64, AArch64, RV64)](#89-standard-c-abi-layouts-packing-and-alignment-x86-64-aarch64-rv64)
  - [§90. Register-Passing Conventions, Stack Frames, and Red Zones](#90-register-passing-conventions-stack-frames-and-red-zones)
  - [§91. Generated C11 Headers, Symbol Mangling, and Preconditions](#91-generated-c11-headers-symbol-mangling-and-preconditions)
  - [§92. Zero-Overhead Safe Rust Bindings](#92-zero-overhead-safe-rust-bindings)

- [Part XV: Complete Diagnostic Reference Catalog](#part-xv-complete-diagnostic-reference-catalog)
  - [§93. Diagnostic Philosophy, Severity Structure, and Error Budgets](#93-diagnostic-philosophy-severity-structure-and-error-budgets)
  - [§94. Lexical Diagnostics (`ORC0001`–`ORC0009`): Formal Predicates, Triggers, Examples, Fixes](#94-lexical-diagnostics-orc0001orc0009-formal-predicates-triggers-examples-fixes)
  - [§95. Syntactic Diagnostics (`ORC0101`–`ORC0108`): Formal Predicates, Triggers, Examples, Fixes](#95-syntactic-diagnostics-orc0101orc0108-formal-predicates-triggers-examples-fixes)
  - [§96. Semantic & Type Diagnostics (`ORC0201`–`ORC0242`): Formal Predicates, Triggers, Examples, Fixes](#96-semantic--type-diagnostics-orc0201orc0242-formal-predicates-triggers-examples-fixes)
  - [§97. Formatter Diagnostics (`ORC0250`–`ORC0252`): Formal Predicates, Triggers, Examples, Fixes](#97-formatter-diagnostics-orc0250orc0252-formal-predicates-triggers-examples-fixes)
  - [§98. Documentation Diagnostics (`ORC0260`–`ORC0261`): Formal Predicates, Triggers, Examples, Fixes](#98-documentation-diagnostics-orc0260orc0261-formal-predicates-triggers-examples-fixes)
  - [§99. Witness Replay Diagnostics (`ORC0270`–`ORC0274`): Formal Predicates, Triggers, Examples, Fixes](#99-witness-replay-diagnostics-orc0270orc0274-formal-predicates-triggers-examples-fixes)
  - [§100. Evaluator & Resource Diagnostics (`ORC0301`): Formal Predicates, Triggers, Examples, Fixes](#100-evaluator--resource-diagnostics-orc0301-formal-predicates-triggers-examples-fixes)

- [Part XVI: Toolchain, Evaluator & Formal EBNF Grammar](#part-xvi-toolchain-evaluator--formal-ebnf-grammar)
  - [§101. The Driver CLI: `orangec` Commands, Options, and Determinism](#101-the-driver-cli-orangec-commands-options-and-determinism)
  - [§102. Deterministic Resource Limits and Denial-of-Service Defense](#102-deterministic-resource-limits-and-denial-of-service-defense)
  - [§103. Complete Unified Formal EBNF Grammar](#103-complete-unified-formal-ebnf-grammar)

---

## Preface

### §1. Mission, Doctrine, and Philosophical Basis

High-assurance cryptographic software occupies a distinct niche in computer science:
a single algorithmic defect, memory safety violation, or microarchitectural side
channel completely destroys the confidentiality and integrity guarantees of an entire
cryptosystem. Historically, this assurance has been sought by assembling a polyglot,
disconnected pipeline:
1. Standards committees write specifications in mathematical prose, LaTeX, or pseudocode.
2. Software engineers transcribe these specifications into C, Rust, or assembly to
   achieve production execution speed.
3. Verification specialists formalize mathematical models in interactive theorem
   provers (such as Rocq, Lean, or Isabelle/HOL) to prove functional correctness.
4. Security researchers construct paper-and-pencil or EasyCrypt reduction proofs
   establishing asymptotic or concrete computational hardness bounds.
5. Timing analyzers inspect emitted binaries with heuristic statistical tests
   (e.g., dudect) to detect data-dependent execution variations.
6. Build engineers write ad-hoc shell scripts to compile, package, and link these
   disparate artifacts.

The fundamental vulnerability of this methodology resides at the **seams**. A
functional correctness theorem proved about an abstract mathematical model guarantees
nothing about a compiled machine binary if the compilation passes introduce undefined
behavior, if the C compiler eliminates memory zeroization loops, or if register
allocation creates variable-latency memory spills.

Orange addresses these seams directly under two governing principles:
- **One language, several semantic worlds:** Rather than forcing all cryptographic
  tasks into an unprincipled universal language or relying on fragile multi-language
  glue, Orange unifies five distinct semantic strata within one editioned module
  system:
  - **`spec`**: Pure, total, mathematical functions over rings and fields.
  - **`impl`**: First-order, terminating imperative procedures governed by affine
    ownership, explicit regions, and contracts.
  - **`machine impl`**: Target-aware low-level routines with register capabilities,
    SIMD intrinsics, and explicit layout.
  - **`game`**: Probabilistic execution, stateful oracles, and polynomial-time adversaries.
  - **`proof`**: Elaborated lambda terms and deductive evidence in Proof IR.
- **Claims, not labels:** Rather than marketing software under qualitative
  adjectives like "verified", Orange records atomic, machine-checkable **claims**.
  Every claim binds an exact subject to an explicit formal relation, named assumptions,
  cryptographic evidence, and an unambiguous verification outcome.

Orange is engineered under an uncompromising doctrine: **build the end product directly
through permanent, production-lineage components**. There is no disposable prototype
phase, no relaxed semantic mode, and no silent failure.

### §2. Conformance, Formality, and Normative Terminology

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHALL**, **SHALL NOT**,
**SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY**, and **OPTIONAL** in this
document are to be interpreted as described in BCP 14 (RFC 2119 / RFC 8174).

- A **conforming Orange source file** is a valid UTF-8 sequence conforming to the
  lexical and syntactic grammar defined herein, beginning with an edition declaration.
- A **conforming Orange compiler** (such as `orangec`) is an executable toolchain
  that strictly enforces all lexical, syntactic, semantic, and typing rules,
  rejecting invalid programs with stable, deterministic diagnostic identifiers
  (`ORCxxxx`), and compiling or evaluating valid programs in exact accordance with
  the operational semantics defined herein.
- An **accepted slice** is a language feature subset formally ratified under the
  project governance model and recorded as immutable normative baseline.
- An **in-review slice** is a feature subset implemented in the authoritative
  compiler that awaits formal ratification of its governing Orange Enhancement
  Proposal (OEP).

### §3. Mathematical Notations and Typographical Conventions

Throughout this manual, formal syntax and semantics are presented using standard
programming language theory conventions:

- $\mathbb{Z}$: The set of mathematical integers $\{\dots, -2, -1, 0, 1, 2, \dots\}$.
- $\mathbb{N}$: The set of non-negative integers $\{0, 1, 2, \dots\}$.
- $\mathbb{Z} / 2^W \mathbb{Z}$: The quotient ring of integers modulo $2^W$.
- $\mathbb{Z} / m \mathbb{Z}$: The ring (or finite field when $m$ is prime) of
  integers modulo $m \ge 2$.
- $\mathbb{B} = \{\text{true}, \text{false}\}$: The two-element Boolean algebra.
- $\mathbb{V}$: The universe of all closed semantic values in the language.
- $\sigma \in \text{Store}$: The structured memory store mapping physical places
  to values.
- $\rho \in \text{Env} = \text{Ident} \to \mathbb{V}$: The local evaluation environment
  mapping variable identifiers to values.
- $\Gamma \in \text{TyEnv} = \text{Ident} \to \tau$: The static typing environment
  mapping variables to types.
- $\Sigma$: The global module and signature environment.
- $\Theta$: The finite size parameter environment.
- $\Delta$: The finite type parameter environment.
- $\Gamma \vdash e : \tau$: Static typing judgment asserting that expression $e$
  possesses type $\tau$ under environment $\Gamma$.
- $\langle e, \rho \rangle \Downarrow v$: Big-step operational reduction judgment
  asserting that expression $e$ evaluates to value $v$ under environment $\rho$.
- $\langle S, \sigma \rangle \longrightarrow \langle S', \sigma' \rangle$: Small-step
  imperative transition judgment.
- $\text{wp}(S, Q)$: Weakest precondition predicate transformer.
- $a \mathbin{\Vert} b$: Bitwise concatenation of bit sequences.
- $H(x)$: Cryptographic hash function (SHA-256 unless otherwise qualified).

### §4. Language Status Register and Slice Traceability

The development of Orange follows a gated, incremental slice roadmap. The table
below records the formal status of every feature slice and architectural stratum:

| Slice / Stratum | Grammar Status | Static Type Rules | Operational Rules | Compiler Status (`orangec 0.0.1`) | Governing Ratification |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **S1 / S2**: Lexical & Syntax Foundation | Normative | Normative | N/A (Syntax Only) | Accepted | D-025, OEP-0002 |
| **S3a**: Typed `spec` Literals (`Int`, `Word[8]`) | Normative | Normative | Reference Core Eval | Accepted | D-026, OEP-0003 |
| **S3b**: Pure Expressions, Word Rings, Operators | Normative | Normative | Small-Step Reductions | Implemented | OEP-0005 |
| **S3c**: Typed `let` Bindings & `as` Conversions | Normative | Normative | Environment Substitution | Implemented | OEP-0006 |
| **S3d**: Fixed-Length Arrays `T^n` & Indexing | Normative | Normative | Exact Bound Array Eval | Implemented | OEP-0007 |
| **S3e**: Bounded Loops `for..in..with` & Updates | Normative | Normative | Deterministic Iteration | Implemented | OEP-0008 |
| **S3f**: `Bool`, Conditionals `if/else`, Division | Normative | Normative | Exhaustive Branch Eval | Implemented | OEP-0009 |
| **S3g**: Data-Dependent Lookups & S-Boxes | Normative | Normative | Range-Proved Lookups | Implemented | OEP-0010 |
| **S3h**: Multi-Module Programs & `use` Imports | Normative | Normative | Acyclic Module Eval | Implemented | OEP-0011 |
| **S3i**: Residue Fields `Mod[m]` & `type` Aliases | Normative | Normative | Modular Reduction Eval | Implemented | OEP-0012 |
| **S3j**: Local `let` in Loop & Branch Blocks | Normative | Normative | Nested Scope Eval | Implemented | OEP-0013 |
| **S3k**: Tuple Types `(T, U)` & Patterns | Normative | Normative | Product Destruction | Implemented | OEP-0014 |
| **S3l**: Byte Strings `"..."`, `hex"..."`, Slices | Normative | Normative | Byte Vector Operations | Implemented | OEP-0015 |
| **S3m**: Finite Size Parameters `[n in low..high]` | Normative | Normative | Eager Monomorphization | Implemented | OEP-0016 |
| **S3n**: Explicit Byte Ordering (`big`, `little`) | Normative | Normative | Bit-Preserving Packing | Implemented | OEP-0017 |
| **S3o**: Finite Type Parameters `[K in {T...}]` | Normative | Normative | Eager Monomorphization | Implemented | OEP-0018 |
| **S3p**: Array Lengths to 65,536 & Eval Limits | Normative | Normative | Bounded Step Counter | Implemented | OEP-0019 |
| **S3q**: Known-Answer Specification `test` Blocks | Normative | Normative | Automated Test Runner | Implemented | OEP-0020 |
| **S3r**: Variable Shift & Rotation Amounts | Normative | Normative | Ring Turn Reduction | Implemented | OEP-0021 |
| **S3s**: Bounded Rectangular Nested Arrays | Normative | Normative | Chained Indexing | Implemented | OEP-0023 |
| **S3t**: Static Moduli with Own Size Names | Normative | Normative | Size-Dependent Moduli | Implemented | OEP-0024 |
| **Implementation Stratum (`impl`)** | Normative 1.0 | Normative 1.0 | Imperative Place Semantics | Planned 1.0 Kernel | D-004, ST-REL |
| **Machine Stratum (`machine impl`)** | Normative 1.0 | Normative 1.0 | Target ISA Simulation | Planned 1.0 Kernel | D-004, D-011 |
| **Game Stratum (`game`)** | Normative 1.0 | Normative 1.0 | Probabilistic Sampling | Planned 1.0 Kernel | D-004, ST-REL |
| **Proof Stratum (`proof`)** | Normative 1.0 | Normative 1.0 | Proof IR Deduction | Planned 1.0 Kernel | D-006, D-007 |
| **Assurance & Claims (`claim`)** | Normative 1.0 | Normative 1.0 | Content-Addressed Graph | Planned 1.0 Kernel | D-005, AM-01 |

---

## Part I: Lexical & Concrete Syntactic Grammar

### §5. Byte-Level Encoding, Normalization Invariants, and Source Limits

1. **Source Encoding:** An Orange source compilation unit MUST consist of a contiguous
   sequence of octets representing valid UTF-8 according to RFC 3629 / Unicode Standard.
2. **Byte Order Mark Prohibition:**
   $$\text{Source}[0..3] = \langle \text{0xEF}, \text{0xBB}, \text{0xBF} \rangle \implies \text{Diagnostic}(\text{ORC0001})$$
   A source file MUST NOT contain the UTF-8 Byte Order Mark (U+FEFF) at byte offset 0.
   Encountering a BOM halts lexing immediately with diagnostic `ORC0001`.
3. **File Size Ceiling:**
   $$|\text{Source}| \le 16,777,216 \text{ bytes} \ (16 \text{ MiB})$$
   Source files exceeding $16 \text{ MiB}$ are rejected with diagnostic `ORC0008`.
4. **Line Endings:**
   The concrete lexical grammar permits only two line terminators:
   - Line Feed (`LF`, ASCII `0x0A`, U+000A).
   - Carriage Return followed immediately by Line Feed (`CRLF`, ASCII `0x0D 0x0A`, U+000D U+000A).
   - An isolated Carriage Return (`CR`, ASCII `0x0D`) not immediately followed by `0x0A`
     is forbidden and emits diagnostic `ORC0001`.
5. **Bidirectional Control Character Defense:**
   To guarantee protection against source spoofing and Trojan Source attacks (CVE-2021-42574),
   the compiler strictly rejects any code point in the following sets outside of
   string literals:
   - C0 control characters (U+0000 through U+001F, excluding `\t`, `\n`, `\r` in CRLF).
   - C1 control characters (U+0080 through U+009F) and DEL (U+007F).
   - U+061C (Arabic Letter Mark).
   - U+200E, U+200F (LRM and RLM).
   - U+2028, U+2029 (Line and Paragraph Separators).
   - U+202A through U+202E (Embedding and Override Controls).
   - U+2066 through U+2069 (Directional Isolate Controls).
   Encountering any of these code points emits diagnostic `ORC0001`. In offline
   HTML documentation generation (`orangec doc`), any permitted control character
   is displayed visibly as `\u{hex}`.
6. **Absence of Canonical Normalization:**
   The lexer does not apply Unicode Normalization Forms (NFC, NFD, NFKC, NFKD).
   Source text is compared strictly as raw byte sequences. Identifiers are
   restricted to ASCII, rendering multi-byte Unicode comparison outside strings
   lexically impossible.

### §6. Lexical Analysis, Maximal Munch, and State-Machine Tokenization

1. **Maximal Munch Rule:**
   The lexical analyzer operates as a deterministic finite automaton (DFA) consuming
   input characters greedily. At any scan point, the longest sequence of characters
   forming a valid lexical token is selected.
   - Example: `<<<` is scanned as the single rotation token `<<<`, never as `<` followed
     by `<<` or three `<` tokens.
   - Example: `::` is scanned as `DOUBLE_COLON`, never as two `COLON` tokens.
   - Example: `++` is scanned as `PLUS_PLUS`, never as two `PLUS` tokens.
2. **Deterministic Token Spans:**
   Every scanned token carries an exact half-open byte span:
   $$\text{Span} = [\text{byte\_start}, \text{byte\_end})$$
   alongside 1-indexed line and column numbers. Diagnostics refer strictly to
   these byte spans.

### §7. Whitespace, Trivia Preservation, and Formatting Idempotence

1. **Whitespace Definition:**
   Whitespace consists of:
   - Space (`0x20`, U+0020)
   - Horizontal Tab (`0x09`, U+0009)
   - Carriage Return (`0x0D`, U+000D, only when followed by `0x0A`)
   - Line Feed (`0x0A`, U+000A)
2. Whitespace serves solely to delimit tokens and has no syntactic significance
   except inside string literals.
3. **The Formatter Contract:**
   The compiler driver incorporates a canonical syntax formatter (`orangec fmt`).
   The formatter is governed by the following mathematical laws:
   - **Idempotence:** $\text{fmt}(\text{fmt}(\text{src})) = \text{fmt}(\text{src})$
   - **Token Identity Invariant:** Stripping trivia from $\text{fmt}(\text{src})$
     yields a token sequence identical in spellings and values to the unformatted input.
   - **Comment Preservation:** All line comments and block comments are preserved
     with byte-identical content and exact inter-token relative order.
   - **Check Mode:** Running `orangec fmt --check FILE` verifies byte-level equality.
     Any discrepancy emits diagnostic `ORC0252` and exits with code 1.
   - Formatting resource exhaustion emits `ORC0250`; internal comment preservation
     inconsistency emits `ORC0251`.

### §8. Nested Block and Line Comments

1. **Line Comments:**
   Begin with ASCII `//` and extend to the next newline (`0x0A`) or end-of-file.
   Line comments have no closing delimiter and cannot nest.
2. **Block Comments:**
   Begin with ASCII `/*` and terminate with `*/`.
   - **Arbitrary Nesting Invariant:**
     The lexer maintains an integer nesting depth counter $d \in \mathbb{N}$,
     initialized to $0$.
     - Upon encountering `/*`, $d \leftarrow d + 1$.
     - Upon encountering `*/`, $d \leftarrow d - 1$.
     - The comment terminates when $d = 0$.
   - **Unterminated Comments:**
     If EOF is reached while $d > 0$, lexing aborts with diagnostic `ORC0002`
     pointing to the primary span of the opening `/*` that lacked a closing delimiter.

### §9. Identifiers, Reserved Words, and Contextual Keywords

1. **Identifier Syntax:**
   Identifiers are ASCII-only and conform to the regular grammar:
   $$\text{identifier} = [A\text{-}Za\text{-}z\_][A\text{-}Za\text{-}z0\text{-}9\_]^*$$
   Non-ASCII Unicode characters in identifiers emit diagnostic `ORC0001`.
2. **Permanently Reserved Words:**
   The following 7 spellings are permanently reserved across all syntactic positions:
   ```text
   edition    module    spec    impl    game    proof    claim
   ```
   A reserved word cannot be used as an identifier for a function, variable,
   parameter, type, or module. Any attempt to do so emits diagnostic `ORC0101`.
3. **Contextual Keywords:**
   The following identifiers possess grammatical meaning only within designated
   syntactic productions, acting as ordinary identifiers elsewhere:
   - `use`: Module header import declaration (§15).
   - `type`: Module-level type alias declaration (§17).
   - `let`: Local immutable variable binding (§33).
   - `as`: Explicit type conversion and byte-order cast (§28, §29).
   - `for`, `in`, `with`: Bounded iteration header delimiters (§42).
   - `if`, `else`: Conditional branching introducers (§37).
   - `true`, `false`: Boolean literal constants (§23).
   - `Mod`: Modular residue domain constructor (§22).
   - `big`, `little`: Endianness specifiers (§29).
   - `test`: Known-answer specification test introducer (§43).
   - `requires`, `ensures`: Pre- and post-condition contracts in `impl` (§58).
   - `invariant`, `variant`: Invariant and termination measures in `impl` (§59).
   - `erase`: Sensitive memory zeroization statement in `impl` (§61).
   - `declassify`: Explicit information-flow release statement (§72).

### §10. Numeric Literals: Radices, Magnitudes, and Underscore Grammar

An integer literal denotes an exact non-negative integer:

```text
integer_literal = decimal_literal | hex_literal | binary_literal ;
decimal_literal = digit (digit | "_")* ;
hex_literal     = ("0x" | "0X") hex_digit (hex_digit | "_")* ;
binary_literal  = ("0b" | "0B") binary_digit (binary_digit | "_")* ;
digit           = "0".."9" ;
hex_digit       = digit | "a".."f" | "A".."F" ;
binary_digit    = "0" | "1" ;
```

1. **Radix Selection:**
   - Prefix `0x` or `0X` selects hexadecimal (base 16).
   - Prefix `0b` or `0B` selects binary (base 2).
   - Lack of prefix selects decimal (base 10).
2. **At Least One Digit:** An integer literal MUST contain at least one valid digit
   following the radix prefix. An isolated `0x` or `0b` emits diagnostic `ORC0005`.
3. **Digit Separator Rules:**
   - Underscores (`_`) are visual separators.
   - An underscore MUST be placed strictly between two valid digits of the chosen base.
   - Leading underscores (`_100` is an identifier, not an integer).
   - Underscores immediately following a prefix (`0x_ff`, `0b_10`) emit `ORC0005`.
   - Trailing underscores (`100_`, `0xff_`) emit `ORC0005`.
   - Consecutive underscores (`10__00`) emit `ORC0005`.
4. **Digit Set Conformance:** Any character consumed after a prefix that is not a
   valid digit of that base (e.g. `0b102` or `0x1g`) causes the token to fail
   with diagnostic `ORC0005`.
5. **Magnitude Representation Budget:**
   The compiler represents integer magnitudes exactly. A literal MUST NOT exceed
   the internal magnitude representation budget of **4,096 bits** ($2^{4096} - 1$,
   approximately $1,234$ decimal digits). Exceeding this magnitude emits diagnostic
   `ORC0205`.

### §11. Byte Sequences, Escape Grammar, and Hexadecimal Strings

Orange provides two concrete representations for immutable byte sequences (`Word[8]^n`):

#### 1. ASCII String Literals (`"..."`)
```text
string_literal = "\"" string_char* "\"" ;
string_char    = (ascii_printable - ("\"" | "\\")) | escape_seq ;
escape_seq     = "\\\"" | "\\\\" | "\\n" | "\\r" | "\\t" | "\\0" | "\\x" hex_digit hex_digit ;
```
- Delimited by double quotes (`0x22`).
- MUST reside on a single logical line. Unclosed strings before `\n` emit `ORC0003`.
- Internal characters MUST be printable ASCII (`0x20`–`0x7E`) or a valid escape.
  Non-printable ASCII or multi-byte UTF-8 codepoints emit `ORC0235` or `ORC0001`.
- **Admitted Escapes:**
  - `\"` (0x22), `\\` (0x5C), `\n` (0x0A), `\r` (0x0D), `\t` (0x09), `\0` (0x00).
  - `\xNN`: An exact byte value specified by two hexadecimal digits `NN` (`0x00`–`0xFF`).
- Any other escape (e.g. `\a`, `\e`, `\u`, `\U`) emits diagnostic `ORC0004`.

#### 2. Hexadecimal String Literals (`hex"..."`)
```text
hex_string_literal = "hex\"" (hex_digit hex_digit | " ")* "\"" ;
```
- Introduced by the token `hex` immediately followed by `"`.
- Consists of pairs of hexadecimal digits representing raw bytes, separated by
  optional spaces (`0x20`).
- An odd number of hex digits, non-hex characters (other than space), or unclosed
  quotes emit diagnostic `ORC0009`.

### §12. Punctuation, Delimiters, Operators, and Lexical Budgets

1. **Closed Operator & Delimiter Inventory:**
   ```text
   Delimiters:
     (   )   {   }   [   ]   ,   :   ;   .   ..   ::

   Arithmetic Operators:
     +   -   *   /   %

   Bitwise Operators:
     &   |   ^   ~

   Shift and Rotation Operators:
     <<   >>   <<<   >>>

   Relational Operators:
     ==   !=   <   <=   >   >=

   Logical Operators:
     !   &&   ||

   Structural Combinators:
     =   with   ++   ->   =>   ?
   ```
2. **Lexical Resource Limits:**
   - Maximum non-trivia tokens per source: **1,048,576 tokens** ($2^{20}$).
     Exceeding this budget emits diagnostic `ORC0006`.
   - Lexical error reporting threshold: **32 errors**. Subsequent errors are
     suppressed, emitting diagnostic `ORC0007`.
   - Lexer memory allocation exhaustion emits diagnostic `ORC0008`.

---

## Part II: Compilation Units and Module Architecture

### §13. Compilation Units, Edition Invariants, and Header Syntax

1. **Compilation Unit Invariant:**
   A compilation unit is an individual UTF-8 source file ending in the extension `.or`.
2. **Mandatory Edition Header:**
   The first non-trivia grammatical production in every Orange source file MUST
   be an edition declaration:
   ```orange
   edition 2026;
   ```
3. **Edition Validation Predicates:**
   - Let $E$ be the edition token. If $E \ne 2026$, the compiler emits diagnostic `ORC0102`.
   - Omitting the terminating semicolon emits `ORC0101`.
   - Repeating the edition declaration or placing it after any other statement
     emits `ORC0104`.
4. The edition declaration binds the source text permanently to the Orange 2026
   grammar and semantics.

### §14. Module Enclosures, Top-Level Member Ordering, and Syntax Trees

1. **Single Module Invariant:**
   Every Orange source file MUST declare exactly one module immediately following
   the edition header:
   ```orange
   edition 2026;

   module module_name {
       // Declarations in strict order:
       // 1. Imports: use ...;
       // 2. Type Aliases: type ... = ...;
       // 3. Members: spec, impl, game, proof, claim, test
   }
   ```
2. Any tokens appearing after the closing brace `}` of the module declaration
   emit diagnostic `ORC0104`.
3. **Top-Level Member Ordering Rules:**
   - All `use` declarations MUST precede all other module declarations.
   - All `type` declarations MUST precede all function, game, proof, or claim declarations.
   - Function (`spec`, `impl`), `game`, `proof`, `claim`, and `test` declarations
     may follow in arbitrary source order.

### §15. Hermetic Module Resolution and Filesystem Path Mapping

1. **Hermeticity:** Orange modules are resolved hermetically without ambient
   network queries or implicit system paths.
2. **File Mapping:**
   - An import `use m;` directs the compiler to load the file `m.or`.
   - The compiler searches:
     1. The directory containing the importing source file.
     2. Declared package source roots configured in `Orange.toml`.
   - If `m.or` cannot be found or read, the compiler halts with diagnostic `ORC0228`.
3. **Qualified Identifiers:**
   - Imported definitions are referenced using the module prefix: `m::symbol`.
   - Calling `m::symbol` when `use m;` was not declared in the importing module
     emits diagnostic `ORC0229`.
   - Unqualified imports (wildcards like `use m::*;`) do not exist in Orange.

### §16. Import Dependency Graphs and Tarjan Cycle Elimination

1. **Dependency Graph Invariant:**
   Let $G = (V, E)$ be the module dependency graph where $V$ is the set of all
   modules and $(u, v) \in E \iff u \text{ contains } \texttt{use } v;$.
   $G$ MUST be a Directed Acyclic Graph (DAG).
2. **Tarjan Cycle Elimination:**
   The compiler constructs $G$ and executes Tarjan's Strongly Connected Components
   (SCC) algorithm. If any component contains a cycle (length $\ge 1$), compilation
   halts immediately with diagnostic `ORC0230`:
   $$\exists v_0, v_1, \dots, v_k.\ (v_i, v_{i+1}) \in E \land v_k = v_0 \implies \text{Diagnostic}(\text{ORC0230})$$
3. **Duplicate Imports:**
   - Importing the same module twice in one source file emits `ORC0231`.
   - Defining two distinct files with identical module identifiers in the same
     program emits `ORC0231`.

### §17. Type Aliases, Structural Equivalence, and Shadowing Restrictions

1. A module may introduce nominal names for types using `type`:
   ```orange
   type State = Word[32]^8;
   type Fe = Mod[(1 << 255) - 19];
   ```
2. **Structural Equivalence:**
   Type aliases are transparent: during semantic analysis, every occurrence of
   an alias is expanded to its underlying type. Type equivalence in Orange is
   strictly structural:
   $$\text{type } A = T \implies A \equiv T$$
3. **Anti-Shadowing Invariants:**
   - A type alias identifier MUST NOT shadow built-in types (`Int`, `Bool`, `Byte`,
     `Word`, `Mod`). Defining `type Int = Word[64];` emits `ORC0233`.
   - Declaring duplicate type alias names within the same module emits `ORC0233`.

### §18. Multi-Strata Architectural Separation and Disjoint Namespaces

1. Orange enforces an ontological separation between distinct modes of cryptographic
   computation:
   - **`spec`**: Pure mathematical functions. Computes over algebraic rings and fields.
     Total, acyclic, and terminating.
   - **`impl`**: Imperative executable procedures. Computes over physical memory places
     and mutable buffers. Affine ownership, contracts, and zeroization.
   - **`machine impl`**: Low-level machine code with register constraints, SIMD types,
     and hardware crypto intrinsics.
   - **`game`**: Probabilistic programs, uniform sampling, stateful oracles, and
     cryptographic reductions.
   - **`proof`**: Proof IR terms establishing refinement, equivalence, and safety.
2. **Disjoint Namespace Invariant:**
   Each stratum maintains an independent symbol table within a module.
   - A module may legitimately declare both `spec sha256` and `impl sha256`.
   - They do not conflict in name resolution: call sites explicitly specify the stratum.
   - They are formally connected only when an explicit `claim` record proves a
     refinement theorem between them.

---

## Part III: Formal Type System & Algebraic Foundations

### §19. Semantic Value Domains and Mathematical Universes

We define the universe of semantic values $\mathbb{V}$ as the disjoint union of
concrete value domains:

$$\mathbb{V} = \mathbb{V}_{\text{Int}} \uplus \mathbb{V}_{\text{Word}} \uplus \mathbb{V}_{\text{Mod}} \uplus \mathbb{V}_{\text{Bool}} \uplus \mathbb{V}_{\text{Array}} \uplus \mathbb{V}_{\text{Tuple}}$$

Where:
- $\mathbb{V}_{\text{Int}} = \mathbb{Z} \cap [-(2^{4096}-1), 2^{4096}-1]$
- $\mathbb{V}_{\text{Word}} = \biguplus_{W \in \{8, 16, 32, 64\}} (\mathbb{Z} / 2^W \mathbb{Z})$
- $\mathbb{V}_{\text{Mod}} = \biguplus_{m \in [2, 2^{521}-1]} (\mathbb{Z} / m \mathbb{Z})$
- $\mathbb{V}_{\text{Bool}} = \{\text{true}, \text{false}\}$
- $\mathbb{V}_{\text{Array}} = \biguplus_{\tau, n} \mathbb{V}_\tau^n$ ($1 \le n \le 65,536$)
- $\mathbb{V}_{\text{Tuple}} = \biguplus_{k \in [2, 16]} (\mathbb{V}_{\tau_0} \times \dots \times \mathbb{V}_{\tau_{k-1}})$

### §20. Mathematical Integers ($\mathbb{Z}$) and the `Int` Type

1. The `Int` type models the algebraic ring of mathematical integers:
   $$(\mathbb{Z}, +, \cdot, -, 0, 1)$$
2. **Exact Arithmetic:** Operations on `Int` never overflow or wrap.
3. **Safety Bound:** Evaluated magnitudes are bounded by $2^{4096} - 1$. Producing
   an integer $|x| \ge 2^{4096}$ emits diagnostic `ORC0205`.
4. **Absence of Coercion:** An `Int` CANNOT be passed to a function expecting `Word[W]`
   or `Mod[m]` without an explicit `as` cast.

### §21. Word Rings: Algebraic Foundations of $\mathbb{Z}/2^W\mathbb{Z}$ ($W \in \{8, 16, 32, 64\}$)

1. The type `Word[W]` represents the quotient ring:
   $$R_W = \mathbb{Z} / 2^W \mathbb{Z}$$
2. **Width Restriction:** $W \in \{8, 16, 32, 64\}$. Any other width emits `ORC0204`.
3. **Ring Axiom Invariance:**
   For all $a, b, c \in R_W$:
   - Associativity: $(a + b) + c = a + (b + c)$ and $(a \cdot b) \cdot c = a \cdot (b \cdot c)$
   - Commutativity: $a + b = b + a$ and $a \cdot b = b \cdot a$
   - Identity: $a + 0 = a$ and $a \cdot 1 = a$
   - Additive Inverse: $a + (2^W - a) \equiv 0 \pmod{2^W}$
   - Distributivity: $a \cdot (b + c) = (a \cdot b) + (a \cdot c)$
4. **Wrapping as Primary Semantics:**
   In Orange word arithmetic, $(2^W - 1) + 1 = 0$ is not an overflow error; it is
   the rigorous algebraic definition of the ring. Fixed-width words cannot produce
   arithmetic faults.
5. **Literal Enclosure:**
   A literal $v$ checked as `Word[W]` MUST satisfy $0 \le v \le 2^W - 1$.
   $v < 0$ emits `ORC0206`; $v \ge 2^W$ emits `ORC0207`.

### §22. Residue Fields and Modular Arithmetic ($\mathbb{Z}/m\mathbb{Z}$ for $2 \le m \le 2^{521}-1$)

1. The type `Mod[m]` represents the quotient structure:
   $$R_m = \mathbb{Z} / m \mathbb{Z}$$
   When $m$ is prime, $R_m \cong \mathbb{F}_m$ (the Galois field of order $m$).
2. **Modulus Range:** $2 \le m \le 2^{521} - 1$. Moduli outside this range emit `ORC0232`.
3. **Canonical Residues:** Every element of `Mod[m]` is uniquely represented by its
   canonical integer residue $r \in [0, m - 1]$.
4. **Inversion and Division:**
   - Division $a / b$ computes $a \cdot b^{-1} \pmod m$.
   - $b^{-1}$ is computed via the Extended Euclidean Algorithm:
     $$\gcd(b, m) = 1 \implies b \cdot u + m \cdot v = 1 \implies b^{-1} \equiv u \pmod m$$
   - **Deterministic Total Behavior:** If $\gcd(b, m) \ne 1$ (including $b \equiv 0$),
     $a / b$ evaluates deterministically to $0$.

### §23. Boolean Truth Values and Propositional Logic (`Bool`)

1. The `Bool` type forms a Boolean algebra $(\mathbb{B}, \land, \lor, \neg, \text{false}, \text{true})$.
2. Relational operators produce `Bool`:
   $$\frac{\Gamma \vdash a : \tau \quad \Gamma \vdash b : \tau \quad \tau \in \{\text{Int}, \text{Word}[W], \text{Mod}[m]\}}{\Gamma \vdash a \mathbin{\text{cmp}} b : \text{Bool}} \quad (\text{cmp} \in \{==, !=, <, <=, >, >=\})$$
3. In specifications, `&&` and `||` evaluate both operands purely; in `impl`,
   they execute as short-circuit control flow.

### §24. Fixed-Length Array Spaces ($T^n$ for $1 \le n \le 65,536$)

1. An array type $T^n$ represents the $n$-ary Cartesian power:
   $$T^n = \underbrace{T \times T \times \dots \times T}_{n \text{ times}}$$
2. **Length Domain:** $1 \le n \le 65,536$. Lengths $< 1$ or $> 65,536$ emit `ORC0221`.
3. **Array Literal Typing:**
   $$\frac{\forall i \in [0, n-1].\ \Gamma \vdash e_i : T}{\Gamma \vdash [e_0, e_1, \dots, e_{n-1}] : T^n}$$
   If the literal element count $k \ne n$, the compiler emits diagnostic `ORC0222`.
4. **Fill Literal Typing:**
   $$\frac{\Gamma \vdash v : T \quad 1 \le n \le 65,536}{\Gamma \vdash [v; n] : T^n}$$

### §25. Multi-Dimensional Rectangular Matrices and Nested Arrays

1. Rectangular multi-dimensional arrays are defined by recursive array types:
   $$T^{n_1 \times n_2 \times \dots \times n_k} = (\dots((T^{n_k})\dots)^{n_2})^{n_1}$$
2. **Global Element Ceiling:**
   $$\prod_{i=1}^k n_i \le 65,536$$
   Exceeding this aggregate scalar limit emits diagnostic `ORC0221`.
3. Chained indexing $M[i][j]$ resolves row types statically without pointer dereferencing.

### §26. Heterogeneous Product Types: Tuples ($(T_0, \dots, T_{k-1})$ for $2 \le k \le 16$)

1. A tuple type $(T_0, \dots, T_{k-1})$ represents the heterogeneous product:
   $$\prod_{i=0}^{k-1} T_i = T_0 \times T_1 \times \dots \times T_{k-1}$$
2. **Arity Ceiling:** $2 \le k \le 16$. Tuples with arity $< 2$ or $> 16$ are rejected.
3. **Projection Typing:**
   $$\frac{\Gamma \vdash t : (T_0, \dots, T_{k-1}) \quad 0 \le j < k}{\Gamma \vdash t.j : T_j}$$
   Attempting field projection on a non-tuple emits `ORC0234`.

### §27. Byte Arrays (`Byte` and `Word[8]^n`)

1. The identifier `Byte` is syntactically and semantically identical to `Word[8]`.
2. String literals `"..."` and hexadecimal literals `hex"..."` are first-class values
   of type `Word[8]^n`, where $n$ is the byte length.
3. **Concatenation Typing:**
   $$\frac{\Gamma \vdash A : T^a \quad \Gamma \vdash B : T^b \quad a + b \le 65,536}{\Gamma \vdash (A \mathbin{+\!+} B) : T^{a+b}}$$

### §28. Explicit Value Conversions (`expr as T`) and Typing Judgments

Orange strictly rejects implicit type coercions. Conversions MUST be explicit:

$$\frac{\Gamma \vdash e : \tau_{\text{src}} \quad \text{AdmissibleCast}(\tau_{\text{src}}, \tau_{\text{dst}})}{\Gamma \vdash (e \text{ as } \tau_{\text{dst}}) : \tau_{\text{dst}}}$$

#### Admissible Cast Table
| Source Type ($\tau_{\text{src}}$) | Target Type ($\tau_{\text{dst}}$) | Operational Semantic Meaning |
| :--- | :--- | :--- |
| `Word[W]` | `Int` | Maps residue $x \in [0, 2^W-1]$ to exact integer $x \in \mathbb{Z}$. |
| `Int` | `Word[W]` | Asserts $0 \le x < 2^W$; maps to word residue $x \bmod 2^W$. |
| `Word[W1]` | `Word[W2]` ($W_1 < W_2$) | Zero-extends: high-order bits set to 0. |
| `Word[W1]` | `Word[W2]` ($W_1 > W_2$) | Truncates: retains low $W_2$ bits ($x \bmod 2^{W_2}$). |
| `Int` | `Mod[m]` | Canonical reduction: $x \bmod m \in [0, m-1]$. |
| `Word[W]` | `Mod[m]` | Canonical reduction: $(x \text{ as Int}) \bmod m$. |

Applying `as` to an untyped literal (e.g. `(42) as Word[32]`) emits diagnostic `ORC0220`.

### §29. Endianness Homomorphisms and Bit-Preserving Packing (`as big T`, `as little T`)

In cryptographic specifications, byte arrays are routinely repacked into words:

$$\frac{\Gamma \vdash e : \tau_{\text{src}} \quad \text{TotalBits}(\tau_{\text{src}}) = \text{TotalBits}(\tau_{\text{dst}})}{\Gamma \vdash (e \text{ as big } \tau_{\text{dst}}) : \tau_{\text{dst}}}$$

$$\frac{\Gamma \vdash e : \tau_{\text{src}} \quad \text{TotalBits}(\tau_{\text{src}}) = \text{TotalBits}(\tau_{\text{dst}})}{\Gamma \vdash (e \text{ as little } \tau_{\text{dst}}) : \tau_{\text{dst}}}$$

1. **Bit Invariance Requirement:**
   $$\text{TotalBits}(\tau_{\text{src}}) \ne \text{TotalBits}(\tau_{\text{dst}}) \implies \text{Diagnostic}(\text{ORC0240})$$
2. **Packing Equations:**
   Let $B : \text{Word}[8]^k$ be an array of bytes converted to $W : \text{Word}[8k]$:
   - **Big-Endian:**
     $$W = \sum_{i=0}^{k-1} B[i] \cdot 2^{8(k - 1 - i)}$$
   - **Little-Endian:**
     $$W = \sum_{i=0}^{k-1} B[i] \cdot 2^{8i}$$

### §30. Dependent Finite Size Parameters ($[n \in \text{low}..\text{high}]$) and Monomorphization

1. A specification function may declare finite size parameters:
   ```orange
   spec pad[len in 1..64](msg: Word[8]^len) -> Word[8]^64 { ... }
   ```
2. **Finite Domain Invariant:**
   - $\text{low}, \text{high} \in \mathbb{N}$ MUST satisfy $1 \le \text{low} \le \text{high} \le 65,536$.
   - Malformed bounds emit diagnostic `ORC0238`.
3. **Eager Monomorphization:**
   The compiler specializes the function for every integer $k \in [\text{low}, \text{high}]$.
   Type checking and verification conditions are checked independently for each instance.
4. Calling with an invalid size parameter count emits `ORC0239`.

### §31. Finite Type Parameter Domains ($[K \in \{T_1, \dots, T_m\}]$)

1. A specification function may declare finite type parameters:
   ```orange
   spec square[K in {Mod[p], Mod[q]}](x: K) -> K { x * x }
   ```
2. The domain set $\{T_1, \dots, T_m\}$ MUST be finite, non-empty, and explicitly listed.
3. Repeating a type in the set, or instantiating with a type outside the set, emits
   diagnostic `ORC0241`.


---

## Part IV: Static Semantics (Typing Rules & Judgments)

### §32. Typing Contexts: Signature ($\Sigma$), Size ($\Theta$), Type ($\Delta$), and Variable ($\Gamma$) Environments

Static semantics in the `spec` stratum operates over a four-component typing context:

$$\mathcal{C} = \langle \Sigma, \Theta, \Delta, \Gamma \rangle$$

1. **Global Signature Context ($\Sigma$):**
   $$\Sigma : \text{ModIdent} \times \text{FuncIdent} \to (\tau_0 \times \dots \times \tau_{k-1} \to \tau_{\text{ret}})$$
   Maps qualified function identifiers to their declared parameter and return signatures.
2. **Size Parameter Context ($\Theta$):**
   $$\Theta : \text{SizeIdent} \to [\text{low}, \text{high}]$$
   Maps static size parameter identifiers to their closed integer bounding intervals.
3. **Type Parameter Context ($\Delta$):**
   $$\Delta : \text{TypeIdent} \to \mathcal{P}(\text{Types})$$
   Maps type parameter variables to their finite, explicitly admitted type candidate sets.
4. **Local Typing Context ($\Gamma$):**
   $$\Gamma : \text{VarIdent} \to \tau$$
   Maps local variable bindings and function parameters to their concrete types.

### §33. Subtyping, Coercion Freedom, and Structural Type Equality

1. **Coercion Freedom:**
   The Orange type system contains no implicit coercions, subtyping, or width widening:
   $$\tau_1 \le \tau_2 \iff \tau_1 \equiv \tau_2$$
2. **Definitional Structural Type Equality:**
   Type equality $\tau_1 \equiv \tau_2$ is reflexive, symmetric, and transitive:
   - $\text{Int} \equiv \text{Int}$
   - $\text{Bool} \equiv \text{Bool}$
   - $\text{Word}[W_1] \equiv \text{Word}[W_2] \iff W_1 = W_2$
   - $\text{Mod}[m_1] \equiv \text{Mod}[m_2] \iff m_1 = m_2$
   - $T_1^{n_1} \equiv T_2^{n_2} \iff T_1 \equiv T_2 \land n_1 = n_2$
   - $(T_0, \dots, T_{k-1}) \equiv (U_0, \dots, U_{m-1}) \iff k = m \land (\forall i < k.\ T_i \equiv U_i)$

### §34. Expression Typing Rules and Formal Judgments

We formalize the static semantics of the `spec` stratum as an inductive inference system
over the four-component typing context $\mathcal{C} = \langle \Sigma, \Theta, \Delta, \Gamma \rangle$.
The primary typing judgment is written:

$$\mathcal{C} \vdash e : \tau$$

denoting that under signatures $\Sigma$, size parameters $\Theta$, type parameters $\Delta$,
and local variables $\Gamma$, expression $e$ is well-typed with unique principal type $\tau$.

#### 1. Identifiers and Variables
$$\frac{x : \tau \in \Gamma}{\langle \Sigma, \Theta, \Delta, \Gamma \rangle \vdash x : \tau} \quad (\text{T-Var})$$

$$\frac{x \notin \Gamma \quad x \notin \text{dom}(\Sigma)}{\langle \Sigma, \Theta, \Delta, \Gamma \rangle \vdash x : \text{Error}(\text{ORC0211})} \quad (\text{T-Var-Err})$$

#### 2. Literals and Constants
$$\frac{n \in \mathbb{Z} \quad -(2^{4096}-1) \le n \le 2^{4096}-1}{\mathcal{C} \vdash n : \text{Int}} \quad (\text{T-Int-Lit})$$

$$\frac{|n| \ge 2^{4096}}{\mathcal{C} \vdash n : \text{Error}(\text{ORC0205})} \quad (\text{T-Int-Overflow})$$

$$\frac{b \in \{\text{true}, \text{false}\}}{\mathcal{C} \vdash b : \text{Bool}} \quad (\text{T-Bool-Lit})$$

$$\frac{s \text{ is ASCII string of length } n \quad 1 \le n \le 65,536}{\mathcal{C} \vdash s : \text{Word}[8]^n} \quad (\text{T-String-Lit})$$

$$\frac{h \text{ is hex string of } 2n \text{ valid nibbles} \quad 1 \le n \le 65,536}{\mathcal{C} \vdash h : \text{Word}[8]^n} \quad (\text{T-Hex-Lit})$$

#### 3. Ring and Arithmetic Expressions
Arithmetic operations are typed according to their underlying algebraic domains.
Mixed-type arithmetic is strictly rejected:

$$\frac{\mathcal{C} \vdash a : \text{Int} \quad \mathcal{C} \vdash b : \text{Int}}{\mathcal{C} \vdash a \mathbin{\text{op}} b : \text{Int}} \quad (\text{op} \in \{+, -, *, /, \%\}) \quad (\text{T-Arith-Int})$$

$$\frac{\mathcal{C} \vdash a : \text{Word}[W] \quad \mathcal{C} \vdash b : \text{Word}[W]}{\mathcal{C} \vdash a \mathbin{\text{op}} b : \text{Word}[W]} \quad (\text{op} \in \{+, -, *\}) \quad (\text{T-Arith-Word})$$

$$\frac{\mathcal{C} \vdash a : \text{Mod}[m] \quad \mathcal{C} \vdash b : \text{Mod}[m]}{\mathcal{C} \vdash a \mathbin{\text{op}} b : \text{Mod}[m]} \quad (\text{op} \in \{+, -, *, /\}) \quad (\text{T-Arith-Mod})$$

$$\frac{\mathcal{C} \vdash a : \tau_1 \quad \mathcal{C} \vdash b : \tau_2 \quad \tau_1 \not\equiv \tau_2}{\mathcal{C} \vdash a \mathbin{\text{op}} b : \text{Error}(\text{ORC0214})} \quad (\text{T-Arith-Mismatch})$$

$$\frac{\mathcal{C} \vdash a : \tau \quad \text{op} \text{ undefined for } \tau}{\mathcal{C} \vdash a \mathbin{\text{op}} b : \text{Error}(\text{ORC0215})} \quad (\text{T-Arith-Undefined})$$

$$\frac{\mathcal{C} \vdash a : \text{Int}}{\mathcal{C} \vdash {-}a : \text{Int}} \quad (\text{T-Neg-Int}) \qquad \frac{\mathcal{C} \vdash a : \text{Mod}[m]}{\mathcal{C} \vdash {-}a : \text{Mod}[m]} \quad (\text{T-Neg-Mod})$$

#### 4. Bitwise Operators
Bitwise operators are strictly restricted to word rings:

$$\frac{\mathcal{C} \vdash a : \text{Word}[W] \quad \mathcal{C} \vdash b : \text{Word}[W]}{\mathcal{C} \vdash a \mathbin{\text{op}} b : \text{Word}[W]} \quad (\text{op} \in \{\&, |, \land\}) \quad (\text{T-Bitwise})$$

$$\frac{\mathcal{C} \vdash a : \text{Word}[W]}{\mathcal{C} \vdash {\sim}a : \text{Word}[W]} \quad (\text{T-Bitwise-Not})$$

$$\frac{\mathcal{C} \vdash a : \tau \quad \tau \ne \text{Word}[W]}{\mathcal{C} \vdash {\sim}a : \text{Error}(\text{ORC0215})} \quad (\text{T-Bitwise-Not-Err})$$

#### 5. Shift and Rotation Operators (§S3b, §S3r)
$$\frac{\mathcal{C} \vdash a : \text{Word}[W] \quad \mathcal{C} \vdash k : T_k \quad T_k \in \{\text{Int}, \text{Word}[U]\}}{\mathcal{C} \vdash a \mathbin{\text{op}} k : \text{Word}[W]} \quad (\text{op} \in \{<<, >>, <<<, >>>\}) \quad (\text{T-Shift})$$

$$\frac{\mathcal{C} \vdash a : \text{Word}[W] \quad k \in \text{Literals} \quad (k < 0 \lor k \ge W)}{\mathcal{C} \vdash a \mathbin{\text{op}} k : \text{Error}(\text{ORC0216})} \quad (\text{T-Shift-Lit-Range})$$

#### 6. Relational Comparisons
Relational equality and orderings operate over homogeneous scalars:

$$\frac{\mathcal{C} \vdash a : \tau \quad \mathcal{C} \vdash b : \tau \quad \tau \in \{\text{Int}, \text{Word}[W], \text{Mod}[m]\}}{\mathcal{C} \vdash a \mathbin{\text{cmp}} b : \text{Bool}} \quad (\text{cmp} \in \{==, !=, <, <=, >, >=\}) \quad (\text{T-Rel})$$

$$\frac{\mathcal{C} \vdash a : \tau_1 \quad \mathcal{C} \vdash b : \tau_2 \quad \tau_1 \not\equiv \tau_2}{\mathcal{C} \vdash a \mathbin{\text{cmp}} b : \text{Error}(\text{ORC0214})} \quad (\text{T-Rel-Mismatch})$$

$$\frac{a, b \text{ are untyped integer literals}}{\mathcal{C} \vdash a \mathbin{\text{cmp}} b : \text{Error}(\text{ORC0227})} \quad (\text{T-Rel-Untyped})$$

#### 7. Logical Connectives
$$\frac{\mathcal{C} \vdash a : \text{Bool} \quad \mathcal{C} \vdash b : \text{Bool}}{\mathcal{C} \vdash a \mathbin{\text{op}} b : \text{Bool}} \quad (\text{op} \in \{\&\&, ||\}) \quad (\text{T-Logic})$$

$$\frac{\mathcal{C} \vdash a : \text{Bool}}{\mathcal{C} \vdash !a : \text{Bool}} \quad (\text{T-Logic-Not})$$

#### 8. Local Bindings and Pattern Destructuring (§S3c)
$$\frac{\mathcal{C} \vdash e : \tau \quad x \notin \Gamma \quad \langle \Sigma, \Theta, \Delta, (\Gamma, x : \tau) \rangle \vdash \text{body} : \tau_{\text{body}}}{\mathcal{C} \vdash (\text{let } x : \tau = e; \ \text{body}) : \tau_{\text{body}}} \quad (\text{T-Let})$$

$$\frac{\mathcal{C} \vdash e : (T_0, \dots, T_{k-1}) \quad (\forall i \ne j.\ x_i \ne x_j) \quad (\forall i.\ x_i \notin \Gamma) \quad \langle \Sigma, \Theta, \Delta, (\Gamma, x_0 : T_0, \dots, x_{k-1} : T_{k-1}) \rangle \vdash \text{body} : \tau_{\text{body}}}{\mathcal{C} \vdash (\text{let } (x_0, \dots, x_{k-1}) = e; \ \text{body}) : \tau_{\text{body}}} \quad (\text{T-Let-Tuple})$$

$$\frac{x \in \Gamma}{\mathcal{C} \vdash (\text{let } x : \tau = e; \ \text{body}) : \text{Error}(\text{ORC0219})} \quad (\text{T-Let-Shadow})$$

#### 9. Conditionals (§S3f)
$$\frac{\mathcal{C} \vdash c : \text{Bool} \quad \mathcal{C} \vdash e_1 : \tau \quad \mathcal{C} \vdash e_2 : \tau}{\mathcal{C} \vdash (\text{if } c \ \{ e_1 \} \ \text{else } \{ e_2 \}) : \tau} \quad (\text{T-If})$$

$$\frac{\mathcal{C} \vdash c : \text{Bool} \quad \mathcal{C} \vdash e_1 : \tau_1 \quad \mathcal{C} \vdash e_2 : \tau_2 \quad \tau_1 \not\equiv \tau_2}{\mathcal{C} \vdash (\text{if } c \ \{ e_1 \} \ \text{else } \{ e_2 \}) : \text{Error}(\text{ORC0214})} \quad (\text{T-If-Mismatch})$$

#### 10. Array Construction, Indexing, Slicing, and Functional Update (§S3d, §S3e, §S3g)
$$\frac{\forall i \in [0, n-1].\ \mathcal{C} \vdash e_i : T \quad 1 \le n \le 65,536}{\mathcal{C} \vdash [e_0, e_1, \dots, e_{n-1}] : T^n} \quad (\text{T-Array-Lit})$$

$$\frac{\mathcal{C} \vdash v : T \quad 1 \le n \le 65,536}{\mathcal{C} \vdash [v; n] : T^n} \quad (\text{T-Array-Fill})$$

$$\frac{\mathcal{C} \vdash A : T^n \quad \mathcal{C} \vdash i : \text{Index}(n)}{\mathcal{C} \vdash A[i] : T} \quad (\text{T-Index})$$

$$\frac{\mathcal{C} \vdash A : T^n \quad \mathcal{C} \vdash i : \text{Index}(n) \quad \mathcal{C} \vdash v : T}{\mathcal{C} \vdash (A \text{ with } [i] = v) : T^n} \quad (\text{T-Update})$$

$$\frac{\mathcal{C} \vdash A : T^n \quad 0 \le l \le u \le n}{\mathcal{C} \vdash A[l..u] : T^{u - l}} \quad (\text{T-Slice})$$

$$\frac{\mathcal{C} \vdash A : T^n \quad 0 \le l \le u \le n \quad \mathcal{C} \vdash B : T^{u - l}}{\mathcal{C} \vdash (A \text{ with } [l..u] = B) : T^n} \quad (\text{T-Slice-Update})$$

$$\frac{\mathcal{C} \vdash A : T^a \quad \mathcal{C} \vdash B : T^b \quad a + b \le 65,536}{\mathcal{C} \vdash (A \mathbin{+\!+} B) : T^{a+b}} \quad (\text{T-Concat})$$

$$\frac{\mathcal{C} \vdash X : \tau \quad \tau \not\equiv T^n}{\mathcal{C} \vdash X[i] : \text{Error}(\text{ORC0224})} \quad (\text{T-Not-Array})$$

#### 11. Bounded Iteration Loops (§S3e)
$$\frac{\mathcal{C} \vdash \text{init} : \tau_{\text{acc}} \quad 0 \le \text{low} \le \text{high} \le 65,536 \quad \langle \Sigma, \Theta, \Delta, (\Gamma, i : \text{Int}, a : \tau_{\text{acc}}) \rangle \vdash \text{body} : \tau_{\text{acc}}}{\mathcal{C} \vdash (\text{for } i \text{ in } \text{low}..\text{high} \text{ with } a = \text{init} \ \{ \text{body} \}) : \tau_{\text{acc}}} \quad (\text{T-For})$$

$$\frac{\text{low} < 0 \lor \text{low} > \text{high} \lor \text{high} > 65,536}{\mathcal{C} \vdash (\text{for } i \text{ in } \text{low}..\text{high} \dots) : \text{Error}(\text{ORC0225})} \quad (\text{T-For-Range-Err})$$

#### 12. Function Application and Monomorphization
Let $f$ have declared signature $[n_1 \in \Theta_1, \dots][K_1 \in \Delta_1, \dots](p_0 : \tau_0, \dots, p_{k-1} : \tau_{k-1}) \to \tau_{\text{ret}} \in \Sigma$:

$$\frac{\forall j.\ k_j \in \Theta_j \quad \forall m.\ U_m \in \Delta_m \quad \sigma = [\vec{n} \mapsto \vec{k}, \vec{K} \mapsto \vec{U}] \quad \forall i \in [0, k-1].\ \mathcal{C} \vdash e_i : \sigma(\tau_i)}{\mathcal{C} \vdash f[\vec{k}][\vec{U}](e_0, \dots, e_{k-1}) : \sigma(\tau_{\text{ret}})} \quad (\text{T-App})$$

### §35. Expression Grouping Envelopes and Syntactic Ambiguity Rejection

The parser constructs expression ASTs according to the grouping envelope grammar:

$$\text{Group}(op_1) \ne \text{Group}(op_2) \implies \text{ParenRequirement}(op_1, op_2)$$

1. **Equi-Precedence Groups:**
   - Group 1 (Additive): `+`, `-`
   - Group 2 (Multiplicative): `*`, `/`, `%`
   - Group 3 (Bitwise AND): `&`
   - Group 4 (Bitwise OR): `|`
   - Group 5 (Bitwise XOR): `^`
   - Group 6 (Shifts / Rotates): `<<`, `>>`, `<<<`, `>>>`
   - Group 7 (Relational): `==`, `!=`, `<`, `<=`, `>`, `>=`
   - Group 8 (Logical): `&&`, `||`
2. **Ambiguity Rejection Rule:**
   If an expression contains binary operators from distinct groups without explicit
   parenthesizing sub-expressions, the syntax tree parser halts immediately with
   diagnostic `ORC0108`. Operator precedence trees cannot resolve across group boundaries.

### §36. Static Array Bounds Verification and Abstract Interval Analysis

Orange guarantees zero runtime bounds checks by verifying all array indexing statically.

#### 1. The Abstract Interval Domain Lattice ($\mathbb{I}$)
The compiler tracks values in the complete lattice:

$$\mathbb{I} = \{ [l, u] \mid l, u \in \mathbb{Z} \cup \{-\infty, +\infty\}, l \le u \} \cup \{ \bot, \top \}$$

ordered by interval inclusion: $[l_1, u_1] \sqsubseteq [l_2, u_2] \iff l_2 \le l_1 \land u_1 \le u_2$.

#### 2. Lattice Transfer Operations
- **Join ($\sqcup$):** $[l_1, u_1] \sqcup [l_2, u_2] = [\min(l_1, l_2), \max(u_1, u_2)]$
- **Meet ($\sqcap$):** $[l_1, u_1] \sqcap [l_2, u_2] = [\max(l_1, l_2), \min(u_1, u_2)]$ (or $\bot$ if $\max > \min$)
- **Addition ($+$):** $[l_1, u_1] + [l_2, u_2] = [l_1 + l_2, u_1 + u_2]$
- **Subtraction ($-$):** $[l_1, u_1] - [l_2, u_2] = [l_1 - u_2, u_1 - l_2]$
- **Bitwise AND with Mask ($\&$):** For $M = 2^k - 1$:
  $$[l, u] \mathbin{\&} [0, M] = [0, \min(u, M)] \quad (\text{if } l \ge 0)$$

#### 3. Bounds Judgment for Array Indexing
$$\text{Index}(n) \triangleq \{ e \mid \mathcal{I}(e) = [l, u] \land 0 \le l \land u < n \}$$

$$\frac{\mathcal{C} \vdash A : T^n \quad \mathcal{I}(e) = [l, u] \quad 0 \le l \quad u < n}{\mathcal{C} \vdash A[e] : T} \quad (\text{T-Index-Proven})$$

$$\frac{\mathcal{C} \vdash A : T^n \quad \mathcal{I}(e) = [l, u] \quad (l < 0 \lor u \ge n)}{\mathcal{C} \vdash A[e] : \text{Error}(\text{ORC0223})} \quad (\text{T-Index-Out-Of-Bounds})$$

### §37. Data-Dependent Indexing, Range Narrowing, and S-Box Lookups

Under slice S3g, data-dependent array lookups (such as cryptographic Substitution Boxes)
are statically admitted by range narrowing:

1. **Word-to-Int Range Narrowing:**
   For any variable $w : \text{Word}[W]$:
   $$\mathcal{I}(w \text{ as Int}) = [0, 2^W - 1]$$
   Consequently, indexing an array of length $2^W$ (e.g. `Word[8]^256` indexed by `Word[8] as Int`)
   satisfies $[0, 2^8 - 1] \subset [0, 256)$, discharging the bounds proof obligation at compile time.
2. **Masked Word Narrowing:**
   For $w : \text{Word}[W]$ and constant $k < W$:
   $$\mathcal{I}((w \mathbin{\&} (2^k - 1)) \text{ as Int}) = [0, 2^k - 1]$$
   Provably safe for indexing any array of length $N \ge 2^k$.
3. If an index expression cannot be statically bounded within $[0, n-1]$, compilation halts
   with `ORC0223`. Runtime bounds checks are formally forbidden in compiled machine kernels.

---

## Part V: Dynamic Semantics (Operational & Reduction Rules)

### §38. Abstract Syntax and Typed Reference Core Lowering

Before evaluation, the compiler lowers the typed AST into **Typed Reference Core IR** (TRC):

$$\text{Source AST} \xrightarrow{\text{Lex/Parse}} \text{Surface AST} \xrightarrow{\text{Elaborate \& Type}} \text{Core IR (TRC)}$$

TRC enforces four architectural invariants:
1. **Full Monomorphization:** All size parameters $[n \in low..high]$ and type parameters
   $[K \in \{T_1, \dots\}]$ are replaced with specialized, ground monomorphic instances.
2. **Type De-Aliasing:** All transparent `type` aliases are eliminated in favor of
   canonical structural representations.
3. **De Bruijn / Immutable Offsets:** All variable references are linked directly to
   immutable environment offsets.
4. **Normalized Envelopes:** Syntactic groupings are flattened into canonical binary trees.

### §39. Evaluation Environments, Value Stores, and Step Budgets

#### 1. Semantic Values ($\mathbb{V}$)
$$\begin{array}{rcll}
v & ::= & c_{\text{Int}} & (\text{integers } n \in \mathbb{Z}) \\
  & \mid & c_{\text{Word}[W]} & (\text{ring elements } r \in [0, 2^W - 1]) \\
  & \mid & c_{\text{Mod}[m]} & (\text{residue elements } r \in [0, m - 1]) \\
  & \mid & \text{true} \mid \text{false} & (\text{booleans}) \\
  & \mid & [v_0, v_1, \dots, v_{n-1}] & (\text{arrays of length } n) \\
  & \mid & (v_0, v_1, \dots, v_{k-1}) & (\text{tuples of arity } k)
\end{array}$$

#### 2. Evaluation Environment ($\rho$)
$$\rho \in \text{Env} = \text{Ident} \rightharpoonup \mathbb{V}$$

An immutable association mapping variable identifiers to semantic values.

#### 3. Deterministic Step Budget ($K$)
Dynamic execution is guarded by a monotonically decreasing step counter $K \in \mathbb{N}$,
initialized to $K_0 = 1,048,576$ (configurable via `--steps`). Every primitive reduction step
decrements $K$ by 1. Reaching $K = 0$ triggers diagnostic `ORC0301`, eliminating unbounded execution.

### §40. Small-Step Operational Semantics (SOS) and Big-Step Reduction

We define the small-step evaluation relation over expressions:

$$e \longrightarrow e'$$

using evaluation contexts $E[\cdot]$ that formalize strict left-to-right evaluation order:

$$\begin{array}{rcl}
E & ::= & [\cdot] \mid E \mathbin{op} e_2 \mid v_1 \mathbin{op} E \mid \mathbf{unop}\; E \\
  & \mid & \text{let } x : \tau = E; \ e_2 \\
  & \mid & \text{let } (x_0, \dots, x_{k-1}) = E; \ e_2 \\
  & \mid & \text{if } E \ \{ e_1 \} \ \text{else } \{ e_2 \} \\
  & \mid & E[e] \mid v[E] \\
  & \mid & E \text{ with } [e_1] = e_2 \mid v \text{ with } [E] = e_2 \mid v \text{ with } [v_1] = E \\
  & \mid & [v_0, \dots, v_{i-1}, E, e_{i+1}, \dots] \\
  & \mid & (v_0, \dots, v_{i-1}, E, e_{i+1}, \dots) \\
  & \mid & E.j \\
  & \mid & E \text{ as } \tau \mid E \text{ as big } \tau \mid E \text{ as little } \tau \\
  & \mid & f(v_0, \dots, v_{i-1}, E, e_{i+1}, \dots)
\end{array}$$

#### Contextual Transition Rule
$$\frac{r \longrightarrow_{\text{redex}} r'}{E[r] \longrightarrow E[r']} \quad (\text{SOS-Context})$$

#### Primitive Redex Contractions ($r \longrightarrow_{\text{redex}} r'$)
1. **Variable Lookup:**
   $$\rho(x) = v \implies x \longrightarrow_{\text{redex}} v \quad (\text{R-Var})$$
2. **Word Ring Operations ($W \in \{8, 16, 32, 64\}$):**
   $$v_1 +_{W} v_2 \longrightarrow_{\text{redex}} (v_1 + v_2) \bmod 2^W \quad (\text{R-Add-Word})$$
   $$v_1 -_{W} v_2 \longrightarrow_{\text{redex}} (v_1 - v_2 + 2^W) \bmod 2^W \quad (\text{R-Sub-Word})$$
   $$v_1 *_{W} v_2 \longrightarrow_{\text{redex}} (v_1 \cdot v_2) \bmod 2^W \quad (\text{R-Mul-Word})$$
3. **Modular Residue Operations ($m \in [2, 2^{521}-1]$):**
   $$v_1 +_{m} v_2 \longrightarrow_{\text{redex}} (v_1 + v_2) \bmod m \quad (\text{R-Add-Mod})$$
   $$v_1 -_{m} v_2 \longrightarrow_{\text{redex}} (v_1 - v_2 + m) \bmod m \quad (\text{R-Sub-Mod})$$
   $$v_1 *_{m} v_2 \longrightarrow_{\text{redex}} (v_1 \cdot v_2) \bmod m \quad (\text{R-Mul-Mod})$$
   $$\gcd(v_2, m) = 1 \implies v_1 /_{m} v_2 \longrightarrow_{\text{redex}} (v_1 \cdot v_2^{-1}) \bmod m \quad (\text{R-Div-Mod})$$
   $$\gcd(v_2, m) \ne 1 \implies v_1 /_{m} v_2 \longrightarrow_{\text{redex}} 0 \quad (\text{R-Div-Mod-Zero})$$
4. **Conditionals:**
   $$\text{if } \text{true} \ \{ e_1 \} \ \text{else } \{ e_2 \} \longrightarrow_{\text{redex}} e_1 \quad (\text{R-If-True})$$
   $$\text{if } \text{false} \ \{ e_1 \} \ \text{else } \{ e_2 \} \longrightarrow_{\text{redex}} e_2 \quad (\text{R-If-False})$$
5. **Local Binding Substitution:**
   $$\text{let } x : \tau = v; \ e_2 \longrightarrow_{\text{redex}} e_2[v / x] \quad (\text{R-Let})$$
   $$\text{let } (x_0, \dots, x_{k-1}) = (v_0, \dots, v_{k-1}); \ e_2 \longrightarrow_{\text{redex}} e_2[v_0/x_0, \dots, v_{k-1}/x_{k-1}] \quad (\text{R-Let-Tuple})$$
6. **Array Indexing and Update:**
   $$[v_0, \dots, v_{n-1}][k] \longrightarrow_{\text{redex}} v_k \quad (\text{where } 0 \le k < n) \quad (\text{R-Index})$$
   $$([v_0, \dots, v_{n-1}] \text{ with } [k] = u) \longrightarrow_{\text{redex}} [v_0, \dots, v_{k-1}, u, v_{k+1}, \dots, v_{n-1}] \quad (\text{R-Update})$$
7. **Tuple Projection:**
   $$(v_0, \dots, v_{k-1}).j \longrightarrow_{\text{redex}} v_j \quad (\text{where } 0 \le j < k) \quad (\text{R-Tuple-Proj})$$
8. **Function Invocation:**
   $$f(v_0, \dots, v_{m-1}) \longrightarrow_{\text{redex}} e_{\text{body}}[v_0/p_0, \dots, v_{m-1}/p_{m-1}] \quad (\text{R-Call})$$

### §41. Operational Reduction of Variable Shifts, Rotations, and Inversions

Let $v_x \in [0, 2^W - 1]$ be a word residue and $k \in \mathbb{Z}$ an evaluation of amount:

1. **Logical Left Shift ($v_x \ll k$):**
   $$v = \begin{cases} (v_x \cdot 2^k) \bmod 2^W & \text{if } 0 \le k < W \\ 0 & \text{if } k \ge W \\ \lfloor v_x / 2^{-k} \rfloor & \text{if } k < 0 \end{cases}$$
2. **Logical Right Shift ($v_x \gg k$):**
   $$v = \begin{cases} \lfloor v_x / 2^k \rfloor & \text{if } 0 \le k < W \\ 0 & \text{if } k \ge W \\ (v_x \cdot 2^{-k}) \bmod 2^W & \text{if } k < 0 \end{cases}$$
3. **Left Circular Rotation ($v_x \lll k$):**
   Let $r = k \bmod W$ with $0 \le r < W$:
   $$v = ((v_x \ll r) \mathbin{|} (v_x \gg (W - r))) \bmod 2^W$$
4. **Right Circular Rotation ($v_x \ggg k$):**
   Let $r = k \bmod W$ with $0 \le r < W$:
   $$v = ((v_x \gg r) \mathbin{|} (v_x \ll (W - r))) \bmod 2^W$$

### §42. Bounded Iteration Semantics and Step-Cost Accounting

Consider the bounded iteration loop:
$$L = \text{for } i \text{ in } \text{low}..\text{high} \text{ with } a = e_{\text{init}} \ \{ e_{\text{body}} \}$$

1. **Iteration Bound:** Let $N = \text{high} - \text{low} \in \mathbb{N}$. Since $0 \le \text{low} \le \text{high} \le 65,536$, $0 \le N \le 65,536$.
2. **Operational Transition Sequence:**
   - Initialize accumulator: $\langle e_{\text{init}}, \rho, K \rangle \Downarrow \langle a_0, K_0 \rangle$.
   - For each step $j = 0, 1, \dots, N-1$:
     - Construct iteration environment: $\rho_j = \rho[i \mapsto \text{low} + j, a \mapsto a_j]$.
     - Evaluate loop body: $\langle e_{\text{body}}, \rho_j, K_j \rangle \Downarrow \langle a_{j+1}, K_{j+1} \rangle$.
     - Verify step budget: $K_{j+1} > 0$. If $K_{j+1} = 0$, abort with `ORC0301`.
   - The loop expression evaluates to final accumulator $a_N$.

### §43. Known-Answer Specification Tests (`test`) and Whole-Aggregate Equality

1. **Test Execution Semantics:**
   A test `test "Title" { e }` is evaluated under the empty environment $\rho_0$:
   $$\langle e, \rho_0, K_0 \rangle \Downarrow \langle v, K' \rangle$$
   - If $v = \text{true}$, the test outcome is `PASSED`.
   - If $v = \text{false}$, the test outcome is `FAILED` with non-zero exit code.
2. **Whole-Aggregate Structural Equality (`==`):**
   Structural equality recursively compares compound elements:
   - For arrays $A, B : T^n$:
     $$A == B \iff \bigwedge_{i=0}^{n-1} (A[i] == B[i])$$
   - For tuples $A, B : (T_0, \dots, T_{k-1})$:
     $$A == B \iff \bigwedge_{j=0}^{k-1} (A.j == B.j)$$

---

## Part VI: Formal Metatheory of the Specification Stratum

### §44. Metatheorem 1: Strong Normalization and Totality

The specification stratum models terminating mathematical functions. We formalize this
via Strong Normalization: every closed, well-typed expression evaluates in a finite number
of steps to a canonical value.

#### 1. Definition of the Complexity Measure ($\mathcal{M}$)
We define a well-founded termination metric $\mathcal{M} : \text{Expr} \to \mathbb{N} \times \mathbb{N} \times \mathbb{N}$
equipped with standard lexicographic ordering $<_{\text{lex}}$:

$$\mathcal{M}(e) = \langle \mathcal{H}(e), \mathcal{L}(e), \mathcal{S}(e) \rangle$$

Where:
- $\mathcal{H}(e) \in \mathbb{N}$ is the **Call Graph DAG Height**: The maximum height of
  any function symbol appearing in $e$ within the acyclic module call graph DAG $G = (V, E)$.
  Since recursive calls are strictly rejected by diagnostic `ORC0217`, $G$ is a DAG of
  finite height $\le |V|$.
- $\mathcal{L}(e) \in \mathbb{N}$ is the **Loop Complexity Sum**: The sum of remaining iteration
  counts $(high - i)$ for all active loop constructs in $e$.
- $\mathcal{S}(e) \in \mathbb{N}$ is the **Syntactic AST Size**: The total number of nodes
  in the abstract syntax tree of $e$.

#### 2. Metatheorem 1 Statement and Proof

**Theorem 1 (Strong Normalization / Totality):**
*For every closed expression $e$ and type $\tau$ such that $\emptyset \vdash e : \tau$, there exists a unique value $v \in \mathbb{V}_\tau$ and finite step count $k \in \mathbb{N}$ such that:*
$$e \longrightarrow^k v$$

*Proof.*
We proceed by well-founded induction on the measure $\mathcal{M}(e) \in (\mathbb{N}^3, <_{\text{lex}})$:
1. **Base Cases (Values):** If $e \in \mathbb{V}$, $k = 0$, $v = e$, and normalization holds vacuously.
2. **Primitive Reductions ($r \longrightarrow r'$):**
   - *Arithmetic/Bitwise:* Computing $(v_1 + v_2) \bmod 2^W$ or $(v_1 \cdot v_2) \bmod m$ strictly
     decreases AST size $\mathcal{S}(r) > \mathcal{S}(r')$ while preserving $\mathcal{H}$ and $\mathcal{L}$.
     Thus $\mathcal{M}(r) >_{\text{lex}} \mathcal{M}(r')$.
   - *Totalization:* Division by zero ($x / 0 = 0$) and modular division with $\gcd(b, m) \ne 1$
     evaluate deterministically to $0$ without divergence or trapping.
   - *Conditionals:* Branching reduces $\text{if } \text{true} \{ e_1 \} \text{ else } \{ e_2 \}$
     to $e_1$, strictly decreasing $\mathcal{S}$.
   - *Let-bindings:* $\text{let } x = v; e_2 \longrightarrow e_2[v/x]$ eliminates the let-binder
     node, strictly reducing $\mathcal{S}$.
3. **Loop Stepping:**
   Each loop iteration reduces the remaining loop bounds from $(high - low)$ to $(high - low - 1)$,
   strictly decreasing the second component $\mathcal{L}(e)$. By lexicographic order,
   $\langle \mathcal{H}, \mathcal{L}, \mathcal{S} \rangle >_{\text{lex}} \langle \mathcal{H}, \mathcal{L} - 1, \mathcal{S}' \rangle$.
4. **Function Invocations ($f(v_0, \dots)$):**
   Let $f$ have body $e_f$. Because the call graph is an acyclic DAG (verified by `ORC0217`),
   every callee satisfies $\mathcal{H}(e_f) < \mathcal{H}(f)$. Thus, the first component
   of the metric strictly decreases:
   $$\langle \mathcal{H}(f), \mathcal{L}, \mathcal{S} \rangle >_{\text{lex}} \langle \mathcal{H}(e_f), \mathcal{L}', \mathcal{S}' \rangle$$
5. Since $(\mathbb{N}^3, <_{\text{lex}})$ contains no infinite descending chains, the reduction
   sequence MUST terminate in a finite number of steps $k \le \mathcal{M}(e)$ at a normal form $v$.
   By Progress (Theorem 3), this normal form is a value $v \in \mathbb{V}_\tau$.
$\blacksquare$

### §45. Metatheorem 2: Type Safety (Progress and Subject Reduction)

Type safety establishes that evaluation never gets stuck on malformed states or trapped operations.

#### Lemma 2.1 (Weakening Context)
*If $\Gamma \vdash e : \tau$ and $x \notin \text{dom}(\Gamma)$, then $\Gamma, x : \tau' \vdash e : \tau$.*

*Proof.* By straightforward induction on the derivation tree of $\Gamma \vdash e : \tau$.
The presence of additional disjoint bindings does not alter any typing premise. $\blacksquare$

#### Lemma 2.2 (Substitution Lemma)
*If $\Gamma, x : \tau' \vdash e : \tau$ and $\Gamma \vdash v : \tau'$, then $\Gamma \vdash e[v / x] : \tau$.*

*Proof.*
We proceed by induction on the derivation of $\Gamma, x : \tau' \vdash e : \tau$:
- **Case T-Var:**
  - If $e = x$, then $e[v/x] = v$. The derivation gave $\tau = \tau'$. We have $\Gamma \vdash v : \tau'$, so $\Gamma \vdash e[v/x] : \tau$.
  - If $e = y \ne x$, then $e[v/x] = y$. Since $y \in \Gamma$, $\Gamma \vdash y : \tau$.
- **Case T-Arith / T-Bitwise / T-Rel:**
  $e = e_1 \mathbin{op} e_2$. By induction hypothesis, $\Gamma \vdash e_1[v/x] : \tau_1$ and
  $\Gamma \vdash e_2[v/x] : \tau_2$. Applying the matching inference rule yields $\Gamma \vdash (e_1 \mathbin{op} e_2)[v/x] : \tau$.
- **Case T-Let:**
  $e = \text{let } y : \tau_y = e_1; e_2$. By $\alpha$-conversion, assume $y \ne x$ and $y \notin \text{FV}(v)$.
  By IH, $\Gamma \vdash e_1[v/x] : \tau_y$. By IH on the body, $\Gamma, y : \tau_y \vdash e_2[v/x] : \tau$.
  Applying T-Let gives $\Gamma \vdash (\text{let } y = e_1[v/x]; e_2[v/x]) : \tau$.
- **Other cases (If, Index, Update, Array, Tuple, Call):**
  Follow directly by applying the induction hypothesis to all constituent subterms.
$\blacksquare$

#### Lemma 2.3 (Canonical Forms)
*Let $v$ be a closed value such that $\emptyset \vdash v : \tau$. Then:*
1. *If $\tau = \text{Int}$, then $v = c \in \mathbb{Z} \cap [-(2^{4096}-1), 2^{4096}-1]$.*
2. *If $\tau = \text{Word}[W]$, then $v = c \in [0, 2^W - 1]$.*
3. *If $\tau = \text{Mod}[m]$, then $v = c \in [0, m - 1]$.*
4. *If $\tau = \text{Bool}$, then $v \in \{\text{true}, \text{false}\}$.*
5. *If $\tau = T^n$, then $v = [v_0, \dots, v_{n-1}]$ where $\forall i.\ \emptyset \vdash v_i : T$.*
6. *If $\tau = (T_0, \dots, T_{k-1})$, then $v = (v_0, \dots, v_{k-1})$ where $\forall j.\ \emptyset \vdash v_j : T_j$.*

*Proof.* Immediate by inspecting the value grammar $\mathbb{V}$ and literal typing rules. $\blacksquare$

#### Theorem 2 (Subject Reduction / Preservation)
*If $\Gamma \vdash e : \tau$ and $e \longrightarrow e'$, then $\Gamma \vdash e' : \tau$.*

*Proof.*
We induct on the small-step transition derivation $e \longrightarrow e'$:
- **Case Contextual Rule $E[r] \longrightarrow E[r']$:**
  By induction on the structure of evaluation context $E[\cdot]$. The typing of $E[\cdot]$
  decomposes into a typing sub-derivation for redex $r$ with some intermediate type $\tau_r$.
  Preservation of the primitive redex $r \longrightarrow r'$ ensures $\Gamma \vdash r' : \tau_r$,
  which re-composes under $E[\cdot]$ to yield $\Gamma \vdash E[r'] : \tau$.
- **Case R-Add-Word:**
  $r = v_1 +_W v_2$. By Lemma 2.3, $v_1, v_2 \in [0, 2^W - 1]$. The redex contracts to
  $r' = (v_1 + v_2) \bmod 2^W$. Since $(v_1 + v_2) \bmod 2^W \in [0, 2^W - 1]$,
  applying T-Word-Lit yields $\Gamma \vdash r' : \text{Word}[W]$.
- **Case R-Let:**
  $r = \text{let } x : \tau_x = v; e_2 \longrightarrow e_2[v/x]$.
  From T-Let, $\Gamma \vdash v : \tau_x$ and $\Gamma, x : \tau_x \vdash e_2 : \tau$.
  By the Substitution Lemma (Lemma 2.2), $\Gamma \vdash e_2[v/x] : \tau$.
- **Case R-If-True / R-If-False:**
  $\text{if } \text{true} \{ e_1 \} \text{ else } \{ e_2 \} \longrightarrow e_1$.
  From T-If, $\Gamma \vdash e_1 : \tau$ and $\Gamma \vdash e_2 : \tau$. The result $e_1$ has type $\tau$.
- **Case R-Index:**
  $[v_0, \dots, v_{n-1}][k] \longrightarrow v_k$.
  From T-Array-Lit and T-Index, $\Gamma \vdash [v_0, \dots, v_{n-1}] : T^n$ and $0 \le k < n$.
  Each element satisfies $\Gamma \vdash v_i : T$. Therefore $\Gamma \vdash v_k : T$.
All other primitive contractions follow analogously. $\blacksquare$

#### Theorem 3 (Progress)
*If $e$ is a closed, well-typed expression ($\emptyset \vdash e : \tau$), then either $e \in \mathbb{V}$ or there exists $e'$ such that $e \longrightarrow e'$.*

*Proof.*
By induction on the typing derivation $\emptyset \vdash e : \tau$:
- If $e$ is a literal, variable in empty context (impossible by well-typedness), or value, $e \in \mathbb{V}$.
- If $e = e_1 \mathbin{op} e_2$:
  By induction hypothesis, $e_1$ either steps ($e_1 \longrightarrow e_1'$, so $E[e_1] \longrightarrow E[e_1']$)
  or is a value $v_1$. If $v_1$ is a value, $e_2$ either steps or is a value $v_2$.
  If both are values, by Canonical Forms (Lemma 2.3), they match the domain of $\text{op}$.
  Word and modular arithmetic are algebraically total for all operands (including division by zero).
  Thus a primitive redex contraction rule applies.
- If $e = A[i]$:
  By IH, $A$ either steps or is a value $[v_0, \dots, v_{n-1}]$; $i$ either steps or is a value $k$.
  By bounds typing T-Index-Proven, the interval analysis guarantees $0 \le k < n$.
  Thus R-Index applies: $[v_0, \dots, v_{n-1}][k] \longrightarrow v_k$.
- If $e = \text{if } c \{ e_1 \} \text{ else } \{ e_2 \}$:
  By IH, $c$ either steps or is a boolean value. If $c = \text{true}$, R-If-True applies;
  if $c = \text{false}$, R-If-False applies.
No well-typed closed expression can be stuck. $\blacksquare$

### §46. Metatheorem 3: Determinism and Semantic Confluence

Evaluation in Orange is purely deterministic: every program execution path is unique.

#### Lemma 3.1 (Unique Decomposition)
*For every closed expression $e$, either $e \in \mathbb{V}$ or there exists a UNIQUE evaluation context $E$ and UNIQUE redex $r$ such that $e = E[r]$.*

*Proof.*
By induction on the abstract syntax tree of $e$. The grammar of evaluation contexts $E$
is deterministic: left-to-right evaluation specifies that the leftmost, innermost redex
is the unique active redex. $\blacksquare$

#### Theorem 4 (Deterministic Evaluation)
*If $e \longrightarrow e_1$ and $e \longrightarrow e_2$, then $e_1 = e_2$.*
*Furthermore, if $\langle e, \rho \rangle \Downarrow v_1$ and $\langle e, \rho \rangle \Downarrow v_2$, then $v_1 = v_2$.*

*Proof.*
By Lemma 3.1, $e$ decomposes uniquely into $E[r]$. The primitive redex contractions
$r \longrightarrow_{\text{redex}} r'$ are defined by mathematical functions:
- Modular arithmetic computes the unique mathematical remainder.
- Division by zero returns canonical zero ($0$).
- Conditionals evaluate unique truth values.
Since the decomposition is unique and the contraction is a mathematical function,
$e_1 = E[r'] = e_2$. Determinism of multi-step and big-step reduction follows immediately
by induction on step count. $\blacksquare$

### §47. Metatheorem 4: Endianness Packing Isomorphism

Cryptographic routines continually convert between sequences of bytes and integer words.
Orange formalizes this equivalence via bijective algebraic homomorphisms.

#### 1. Definition of Packing Homomorphisms
Let $W \in \{16, 32, 64\}$ and $k = W / 8$.
We define the packing maps $\beta_W^{\text{big}}, \beta_W^{\text{little}} : \text{Word}[8]^k \to \text{Word}[W]$:

$$\beta_W^{\text{big}}(B) = \sum_{i=0}^{k-1} B[i] \cdot 2^{8(k - 1 - i)} \pmod{2^W}$$

$$\beta_W^{\text{little}}(B) = \sum_{i=0}^{k-1} B[i] \cdot 2^{8i} \pmod{2^W}$$

and their inverse unpacking maps $\beta_W^{\text{big},-1}, \beta_W^{\text{little},-1} : \text{Word}[W] \to \text{Word}[8]^k$:

$$\beta_W^{\text{big},-1}(w)[i] = \lfloor w / 2^{8(k - 1 - i)} \rfloor \bmod 256$$

$$\beta_W^{\text{little},-1}(w)[i] = \lfloor w / 2^{8i} \rfloor \bmod 256$$

#### Theorem 5 (Bijective Invertibility and Bit-Level Isomorphism)
*The packing maps $\beta_W^{\text{big}}$ and $\beta_W^{\text{little}}$ are bijections between $\text{Word}[8]^k$ and $\text{Word}[W]$:*
$$\beta_W^{-1} \circ \beta_W = \text{id}_{\text{Word}[8]^k} \quad \text{and} \quad \beta_W \circ \beta_W^{-1} = \text{id}_{\text{Word}[W]}$$

*Furthermore, packing preserves exact bit-level representation:*
$$\text{bin}_W(\beta_W^{\text{big}}(B)) = \text{bin}_8(B[0]) \mathbin{\Vert} \text{bin}_8(B[1]) \mathbin{\Vert} \dots \mathbin{\Vert} \text{bin}_8(B[k-1])$$
$$\text{bin}_W(\beta_W^{\text{little}}(B)) = \text{bin}_8(B[k-1]) \mathbin{\Vert} \dots \mathbin{\Vert} \text{bin}_8(B[1]) \mathbin{\Vert} \text{bin}_8(B[0])$$

*Proof.*
We verify the identity for $\beta_W^{\text{big}}$ (the little-endian case is symmetric):
1. **Left Inverse:** For any $B \in \text{Word}[8]^k$, let $w = \beta_W^{\text{big}}(B) = \sum_{j=0}^{k-1} B[j] 2^{8(k-1-j)}$.
   For any index $i \in [0, k-1]$:
   $$\lfloor w / 2^{8(k-1-i)} \rfloor = \sum_{j=0}^i B[j] 2^{8(i-j)}$$
   Taking modulo 256 isolates the term $j = i$ (since for $j < i$, $8(i-j) \ge 8$, so $2^{8(i-j)} \equiv 0 \pmod{256}$):
   $$\lfloor w / 2^{8(k-1-i)} \rfloor \bmod 256 = B[i]$$
   Thus $\beta_W^{\text{big},-1}(\beta_W^{\text{big}}(B)) = B$.
2. **Right Inverse:** For any $w \in [0, 2^W - 1]$, by radix-256 integer representation,
   $w = \sum_{i=0}^{k-1} (\lfloor w / 2^{8(k-1-i)} \rfloor \bmod 256) 2^{8(k-1-i)} = \beta_W^{\text{big}}(\beta_W^{\text{big},-1}(w))$.
3. Since both domain and codomain have cardinality $2^W$, invertibility establishes bijection and bitwise isomorphism.
$\blacksquare$

---

## Part VII: Specification Corpus & Reference Cryptographic Standards

### §48. Mathematical Transcription Methodology and Traceability

To eliminate transcription error at the specification seam, Orange 2026 specifications
transcribe standard mathematical specifications directly into equivalent language
constructs:

| Standard Notation (e.g. FIPS 180-4 / RFC 8439) | Orange 2026 Representation | Algebraic Meaning |
| :--- | :--- | :--- |
| $x \oplus y$ | `x ^ y` | Bitwise Exclusive-OR |
| $x \land y$ | `x & y` | Bitwise Conjunction |
| $\neg x$ | `~x` | Bitwise Complement |
| $x + y \pmod{2^{32}}$ | `x + y` on `Word[32]` | Algebraic Word Ring Addition |
| $\text{ROTR}^n(x)$ | `x >>> n` | Circular Right Rotation |
| $\text{SHR}^n(x)$ | `x >> n` | Logical Right Shift |
| $A \mathbin{\Vert} B$ | `A ++ B` | Contiguous Array Concatenation |

### §49. Complete Reference Specification: FIPS 180-4 SHA-256

```orange
edition 2026;

module sha256_spec {
    // 1. Bitwise Logical Functions (FIPS 180-4 Section 4.1.2)
    spec ch(x: Word[32], y: Word[32], z: Word[32]) -> Word[32] {
        (x & y) ^ (~x & z)
    }

    spec maj(x: Word[32], y: Word[32], z: Word[32]) -> Word[32] {
        (x & y) ^ (x & z) ^ (y & z)
    }

    spec big_sigma0(x: Word[32]) -> Word[32] {
        (x >>> 2) ^ (x >>> 13) ^ (x >>> 22)
    }

    spec big_sigma1(x: Word[32]) -> Word[32] {
        (x >>> 6) ^ (x >>> 11) ^ (x >>> 25)
    }

    spec small_sigma0(x: Word[32]) -> Word[32] {
        (x >>> 7) ^ (x >>> 18) ^ (x >> 3)
    }

    spec small_sigma1(x: Word[32]) -> Word[32] {
        (x >>> 17) ^ (x >>> 19) ^ (x >> 10)
    }

    // 2. Initial Hash Values H(0) (FIPS 180-4 Section 5.3.3)
    spec initial_state() -> Word[32]^8 {
        [
            0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a,
            0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19
        ]
    }

    // 3. Message Schedule Expansion (FIPS 180-4 Section 6.2.2)
    spec expand_message(block: Word[8]^64) -> Word[32]^64 {
        let words: Word[32]^16 = block as big Word[32]^16;
        for i in 16..64 with w = words ++ [0; 48] {
            let s0 = small_sigma0(w[i - 15]);
            let s1 = small_sigma1(w[i - 2]);
            let next_w = w[i - 16] + s0 + w[i - 7] + s1;
            w with [i] = next_w
        }
    }

    // 4. SHA-256 Block Compression Round Step
    spec compress_step(
        state: Word[32]^8,
        w_i: Word[32],
        k_i: Word[32]
    ) -> Word[32]^8 {
        let (a, b, c, d, e, f, g, h) = state;
        let t1 = h + big_sigma1(e) + ch(e, f, g) + k_i + w_i;
        let t2 = big_sigma0(a) + maj(a, b, c);
        [t1 + t2, a, b, c, d + t1, e, f, g]
    }
}
```

### §50. Complete Reference Specification: RFC 8439 ChaCha20

```orange
edition 2026;

module chacha20_spec {
    // 1. ChaCha20 Quarter-Round (RFC 8439 Section 2.1)
    spec quarter_round(
        a: Word[32],
        b: Word[32],
        c: Word[32],
        d: Word[32]
    ) -> (Word[32], Word[32], Word[32], Word[32]) {
        let a1 = a + b;
        let d1 = (d ^ a1) <<< 16;
        let c1 = c + d1;
        let b1 = (b ^ c1) <<< 12;
        let a2 = a1 + b1;
        let d2 = (d1 ^ a2) <<< 8;
        let c2 = c1 + d2;
        let b2 = (b1 ^ c2) <<< 7;
        (a2, b2, c2, d2)
    }

    // 2. Executable Conformance Test Vector (RFC 8439 Section 2.1.1)
    test "RFC 8439 Section 2.1.1 ChaCha20 Quarter-Round" {
        let (a, b, c, d) = quarter_round(
            0x11111111,
            0x01020304,
            0x9b8d6f43,
            0x01234567
        );
        (a == 0xea2a92f4) && (b == 0xcb1cf8ce) && (c == 0x4581472e) && (d == 0x5881c4bb)
    }
}
```

### §51. Complete Reference Specification: Curve25519 / X25519 (RFC 7748)

```orange
edition 2026;

module curve25519_spec {
    // Prime Field of Curve25519: 2^255 - 19
    type Fe = Mod[(1 << 255) - 19];

    spec fe_add(a: Fe, b: Fe) -> Fe { a + b }
    spec fe_sub(a: Fe, b: Fe) -> Fe { a - b }
    spec fe_mul(a: Fe, b: Fe) -> Fe { a * b }
    spec fe_sq(a: Fe)  -> Fe { a * a }
    spec fe_inv(a: Fe) -> Fe { 1 / a }

    // Differential Point Addition and Doubling (RFC 7748 Section 5)
    spec montgomery_ladder_step(
        x1: Fe,
        x2: Fe,
        z2: Fe,
        x3: Fe,
        z3: Fe
    ) -> (Fe, Fe, Fe, Fe) {
        let da = (x3 - z3) * (x2 + z2);
        let cb = (x3 + z3) * (x2 - z2);
        let next_x3 = (da + cb) * (da + cb);
        let next_z3 = x1 * ((da - cb) * (da - cb));
        let aa = (x2 + z2) * (x2 + z2);
        let bb = (x2 - z2) * (x2 - z2);
        let e = aa - bb;
        let a24 = 121665 as Fe;
        let next_x2 = aa * bb;
        let next_z2 = e * (aa + (a24 * e));
        (next_x2, next_z2, next_x3, next_z3)
    }
}
```

### §52. Complete Reference Specification: Poly1305 Field MAC (RFC 8439)

```orange
edition 2026;

module poly1305_spec {
    // Prime Field of Poly1305: 2^130 - 5
    type Fe = Mod[(1 << 130) - 5];

    // Key Clamping: clears specific bits of r (RFC 8439 Section 2.5)
    spec clamp_r(r_bytes: Word[8]^16) -> Fe {
        let r_words: Word[32]^4 = r_bytes as little Word[32]^4;
        let c0 = r_words[0] & 0x0fffffff;
        let c1 = r_words[1] & 0x0ffffffc;
        let c2 = r_words[2] & 0x0ffffffc;
        let c3 = r_words[3] & 0x0ffffffc;
        let clamped: Word[8]^16 = [c0, c1, c2, c3] as little Word[8]^16;
        clamped as little Fe
    }

    // Accumulator Block Multiplication
    spec poly1305_block(acc: Fe, r: Fe, block_val: Fe) -> Fe {
        (acc + block_val) * r
    }
}
```


---

## Part VIII: Implementation Stratum (`impl`) & Memory Model

### §53. Imperative Execution Semantics and Place Logic

The `impl` stratum formalizes effectful, terminating imperative procedures engineered
for direct compilation to native physical machine code without a runtime system or
garbage collector.

#### 1. Place Syntax and Separation
While the specification stratum computes purely over mathematical values $\mathbb{V}$,
the implementation stratum executes over **places** (memory locations, stack slots,
and register allocations):

$$\begin{array}{rcll}
p & ::= & x & (\text{base variable place}) \\
  & \mid & p[e] & (\text{indexed array place}) \\
  & \mid & p.j & (\text{tuple field place})
\end{array}$$

#### 2. Abstract Command Syntax
$$\begin{array}{rcll}
s & ::= & \texttt{skip} & (\text{null operation}) \\
  & \mid & p := e & (\text{place mutation}) \\
  & \mid & s_1; \ s_2 & (\text{sequential composition}) \\
  & \mid & \text{if } b \ \{ s_1 \} \ \text{else } \{ s_2 \} & (\text{conditional branch}) \\
  & \mid & \text{while } b \ \text{inv } I \ \text{var } V \ \{ s \} & (\text{bounded iteration}) \\
  & \mid & \text{let } x : T = e; \ s & (\text{stack local allocation}) \\
  & \mid & \text{let } (\&'r \text{mut } l, \&'r \text{mut } r) = \text{split\_mut}(p, k); \ s & (\text{disjoint mutable slice}) \\
  & \mid & \text{erase } p; & (\text{mandatory memory zeroization}) \\
  & \mid & \text{return } e; & (\text{procedure return})
\end{array}$$

#### 3. Structured Machine Stores ($\sigma$)
Execution operates over a structured memory store $\sigma$:

$$\sigma : \text{Loc} \rightharpoonup \mathbb{V}$$

where $\text{Loc} = \text{RegionId} \times \mathbb{N}$ represents physical memory addresses
partitioned into lifetime regions.

#### 4. Small-Step Imperative Transition System
The operational reduction is defined as:

$$\langle s, \sigma \rangle \longrightarrow \langle s', \sigma' \rangle$$

- **Place Mutation:**
  $$\frac{\langle e, \sigma \rangle \Downarrow v \quad p \in \text{dom}(\sigma)}{\langle p := e, \sigma \rangle \longrightarrow \langle \texttt{skip}, \sigma[p \mapsto v] \rangle} \quad (\text{S-Assign})$$
- **Sequential Composition:**
  $$\frac{\langle s_1, \sigma \rangle \longrightarrow \langle s_1', \sigma' \rangle}{\langle s_1; s_2, \sigma \rangle \longrightarrow \langle s_1'; s_2, \sigma' \rangle} \quad (\text{S-Seq-Step}) \qquad \frac{}{\langle \texttt{skip}; s_2, \sigma \rangle \longrightarrow \langle s_2, \sigma \rangle} \quad (\text{S-Seq-Skip})$$
- **Conditionals:**
  $$\frac{\langle b, \sigma \rangle \Downarrow \text{true}}{\langle \text{if } b \{ s_1 \} \text{ else } \{ s_2 \}, \sigma \rangle \longrightarrow \langle s_1, \sigma \rangle} \quad (\text{S-If-True})$$
  $$\frac{\langle b, \sigma \rangle \Downarrow \text{false}}{\langle \text{if } b \{ s_1 \} \text{ else } \{ s_2 \}, \sigma \rangle \longrightarrow \langle s_2, \sigma \rangle} \quad (\text{S-If-False})$$

### §54. Structured Memory Model: Heap-Freedom, Regions, and Stack Layout

#### 1. The Heap-Freedom Invariant
Verified cryptographic kernels in Orange formally prohibit dynamic heap allocation:

$$\forall \text{procedure } P.\ \text{DynamicAllocations}(P) = 0$$

General memory allocators (`malloc`, `free`, runtime pools) are excluded from the runtime
linkage. All memory consumed during procedure execution is allocated in one of three
statically governed lifetime regions:

$$\text{Region} = \{\text{Static}\} \cup \{\text{Stack}(\ell) \mid \ell \in \mathbb{N}\} \cup \{\text{Caller}(\kappa) \mid \kappa \in \text{RegionId}\}$$

1. **Static Region ($\mathcal{R}_{\text{static}}$):** Contains read-only lookup tables,
   constants, and algorithm parameters mapped into immutable physical text/rodata sections.
2. **Stack Regions ($\mathcal{R}_{\text{stack}}(\ell)$):** Activation records created on
   procedure call at call depth $\ell$. Every buffer declared within a procedure body
   resides at a fixed, statically computed stack offset.
3. **Caller Regions ($\mathcal{R}_{\text{caller}}(\kappa)$):** Destination and source buffers
   allocated by the calling environment and passed via capability references.

#### 2. Bounded Stack Invariant
The compiler computes a verified upper bound on activation frame size:

$$\forall P.\ \text{FrameSize}(P) \le \text{MAX\_STACK\_FRAME} = 65,536 \text{ bytes}$$

Because call graphs are strictly acyclic DAGs, the total worst-case stack depth of an
entire cryptographic library is statically bounded:

$$\text{TotalStackDepth} = \sum_{P \in \text{CallPath}_{\max}} \text{FrameSize}(P) < \infty$$

Stack overflow is provably impossible at compile time.

### §55. Affine Ownership, Move Semantics, and Capability Borrowing ($\&T, \&\text{mut } T$)

Memory safety and data-race freedom are enforced statically via an affine capability calculus.

#### 1. Capability Modalities
Every binding carries one of three linear capability modalities:
- **Owned Capability ($\mathbf{Own}(T)$):** Unique, affine ownership. May be mutated
  or consumed (moved) exactly once. After a move, the source identifier is uninitialized.
- **Shared Borrow ($\&'r T$):** Non-exclusive read capability valid throughout lifetime region $'r$.
  Duplicable; forbids concurrent mutable access.
- **Unique Mutable Borrow ($\&'r \text{mut } T$):** Exclusive read/write capability valid
  throughout region $'r$. Non-duplicable; prohibits all other concurrent borrows (shared or mutable).

#### 2. Region Outlives Subtyping
Lifetimes are partially ordered by the outlives relation $\sqsubseteq$:

$$'r_1 \sqsubseteq 'r_2 \iff \text{Region } 'r_1 \text{ encompasses / outlives Region } 'r_2$$

$$\frac{'r_1 \sqsubseteq 'r_2}{\&'r_1 T \le \&'r_2 T} \quad (\text{Subtype-Shared}) \qquad \frac{'r_1 \sqsubseteq 'r_2 \quad T_1 \equiv T_2}{\&'r_1 \text{mut } T_1 \le \&'r_2 \text{mut } T_2} \quad (\text{Subtype-Mut})$$

### §56. Separation Logic Foundation and Non-Aliasing Invariants

The formal memory model is governed by an intuitionistic Separation Logic:

#### 1. Spatial Assertion Language
$$\begin{array}{rcll}
P, Q & ::= & \text{emp} & (\text{empty heap}) \\
      & \mid & p \mapsto v & (\text{place } p \text{ points to value } v) \\
      & \mid & P \ast Q & (\text{spatial separating conjunction}) \\
      & \mid & P \mathbin{-\!\!*} Q & (\text{magic wand / separating implication}) \\
      & \mid & P \land Q \mid P \lor Q \mid \neg P & (\text{classical connectives}) \\
      & \mid & \forall x.\ P \mid \exists x.\ P & (\text{first-order quantifiers})
\end{array}$$

#### 2. Semantic Satisfaction ($\sigma \models P$)
- $\sigma \models \text{emp} \iff \text{dom}(\sigma) = \emptyset$
- $\sigma \models p \mapsto v \iff \text{dom}(\sigma) = \{ p \} \land \sigma(p) = v$
- $\sigma \models P \ast Q \iff \exists \sigma_1, \sigma_2.\ \sigma = \sigma_1 \uplus \sigma_2 \land \sigma_1 \models P \land \sigma_2 \models Q$,
  where $\uplus$ denotes disjoint union of memory domains ($\text{dom}(\sigma_1) \cap \text{dom}(\sigma_2) = \emptyset$).

#### 3. Separation Logic Inference Rules
$$\frac{\{ P \} \ s \ \{ Q \}}{\{ P \ast R \} \ s \ \{ Q \ast R \}} \quad (\text{Frame Rule, with } \text{Mod}(s) \cap \text{FV}(R) = \emptyset)$$

$$\frac{}{\{ p \mapsto - \} \ p := v \ \{ p \mapsto v \}} \quad (\text{Place-Assign-Axiom})$$

#### 4. The Non-Aliasing Theorem

**Theorem 6 (Static Disjointness of Mutable Borrows):**
*Let $\Gamma$ be a well-typed environment in the `impl` stratum containing two distinct mutable bindings $x : \&'r \text{mut } T_1$ and $y : \&'r \text{mut } T_2$. Then the places referenced by $x$ and $y$ are strictly disjoint in memory:*
$$\text{dom}(\sigma_x) \cap \text{dom}(\sigma_y) = \emptyset$$

*Proof.*
By induction on the typing derivations of borrow introduction. A unique mutable borrow
$\&'r \text{mut } p$ requires consuming the exclusive capability over place $p$.
By the linearity of the capability typing rules, splitting an exclusive capability into
two distinct active mutable handles is prohibited unless guarded by a disjoint sub-place
partitioning rule (such as `split_mut`). Thus, distinct mutable identifiers reference
orthogonal memory addresses $\sigma_x \uplus \sigma_y$. Aliasing of mutable memory is
statically impossible. $\blacksquare$

### §57. In-Place Mutable Slicing and Disjointness Proof Obligations

Cryptographic routines (such as Feistel networks or round functions) require partitioning
a single continuous buffer into independently mutable sub-slices:

```orange
impl feistel_round(block: &mut Word[8]^64) {
    let (left, right) = split_mut(block, 32);
    // left:  &mut Word[8]^32
    // right: &mut Word[8]^32
    round_function(left, right);
}
```

#### 1. Slicing Axiom in Separation Logic
$$\frac{0 \le k \le n}{\{ p \mapsto (v_0 \dots v_{n-1}) \} \ \text{split\_mut}(p, k) \ \{ (p[0..k] \mapsto v_0 \dots v_{k-1}) \ast (p[k..n] \mapsto v_k \dots v_{n-1}) \}}$$

#### 2. Disjointness Obligation
The separating conjunction holds because the half-open index intervals are disjoint:
$$[0, k) \cap [k, n) = \emptyset$$
The compiler's deductive verification engine automatically proves interval disjointness
prior to emitting native code.

### §58. Two-State Contracts: Preconditions (`requires`), Postconditions (`ensures`), and `old(...)`

Procedures declare formal contracts verified by automated deductive proof:

```orange
impl sha256_compress(
    state:  &mut Word[32]^8,
    block:  &Word[8]^64
)
requires block.len == 64
ensures  state == spec::sha256_compress_block(old(state), block)
```

1. **Preconditions (`requires` $P$):**
   A state predicate over arguments and initial memory $\sigma_0$. Call sites MUST
   satisfy $\sigma_0 \models P$.
2. **Postconditions (`ensures` $Q$):**
   A relational two-state predicate evaluated over initial state $\sigma_0$ and final state $\sigma_f$.
3. **The `old(e)` Operator:**
   Evaluates an expression $e$ in the pre-state $\sigma_0$:
   $$\llbracket \text{old}(e) \rrbracket_{(\sigma_0, \sigma_f)} = \llbracket e \rrbracket_{\sigma_0}$$
   This allows contracts to state exact functional transitions ($x = \text{old}(x) + 1$).

### §59. Loop Invariants and Well-Founded Termination Variants

While loops in `impl` MUST be annotated with an inductive invariant and a termination variant:

```orange
while i < 64
invariant i <= 64 && state == spec::partial_compress(old(state), block, i)
variant   64 - i
{
    state = round_step(state, block, i);
    i = i + 1;
}
```

1. **Inductive Invariant ($I$):**
   - Initiation: $P(\sigma_0) \implies I(\sigma_0)$.
   - Preservation: $\{ I \land b \} \ s \ \{ I \}$.
   - Exit: $(I \land \neg b) \implies Q$.
2. **Well-Founded Variant ($V$):**
   - $V : \text{Store} \to \mathcal{W}$ where $(\mathcal{W}, \prec)$ is a well-founded order (typically $(\mathbb{N}, <)$).
   - Invariant: $I \land b \implies V(\sigma) \ge 0$.
   - Strictly decreasing: $\{ I \land b \land V = v_0 \} \ s \ \{ V \prec v_0 \}$.
   - Infinite execution loops are provably impossible.

### §60. Typed Failure, Result Enums, and Total Panic Freedom

1. **Zero Runtime Traps:** Division-by-zero, out-of-bounds indexing, and null pointer
   dereferences are prevented statically.
2. **Algebraic Error Handling:** Operations susceptible to runtime operational failure
   (e.g. signature verification or ciphertext authentication) MUST return an algebraic `Result[T, E]`:
   ```orange
   impl aead_decrypt(key: &Word[8]^32, ct: &Word[8]^n, tag: &Word[8]^16) -> Result[Word[8]^n, AuthError]
   ```
3. Callers MUST pattern-match both branches. Panic mechanisms, abort signals, and
   unwinding runtimes do not exist in conforming Orange implementations.

### §61. Storage Zeroization, Stack Scrubbing, and Memory Erasure (`erase`)

1. Secrets (private keys, ephemeral nonces, decrypted plaintexts) MUST be erased
   from physical memory immediately upon leaving scope:
   ```orange
   erase expanded_keys;
   ```
2. **Operational Semantics of `erase`:**
   $$\frac{}{\{ p \mapsto v \} \ \text{erase } p; \ \{ p \mapsto \mathbf{0} \}} \quad (\text{Erase-Axiom})$$
3. **Machine Code Guarantees:**
   - Emits volatile memory zero-fill instructions (e.g. `rep stosb` on x86-64 or vector zero stores).
   - Inserts hardware compiler barriers (`asm volatile("" ::: "memory")`) preventing dead-store
     elimination (DSE) optimizations from removing the zeroization writes.
   - Claim `CF-08` verifies that upon procedure exit, every stack slot or register allocated
     for the erased place contains zero bits.

---

## Part IX: Machine Implementation Stratum (`machine impl`)

### §62. Target Machine Modeling and Register Capabilities

The `machine impl` stratum allows direct assembly-level cryptographic engineering
while retaining end-to-end formal verification:

#### 1. Machine State Configuration
A physical execution state is modeled as:

$$\mathcal{M} = \langle \text{PC}, \mathcal{R}_{\text{GPR}}, \mathcal{R}_{\text{SIMD}}, \text{Flags}, \text{Mem} \rangle$$

- $\mathcal{R}_{\text{GPR}} : \text{RegId} \to \text{Word}[64]$ (general-purpose registers).
- $\mathcal{R}_{\text{SIMD}} : \text{VecRegId} \to \text{Word}[8]^{16 \text{ to } 64}$ (vector registers).
- $\text{Flags} : \{\text{CF}, \text{ZF}, \text{SF}, \text{OF}\} \to \text{Bool}$.

#### 2. Linear Register Typing
Physical registers are treated as linear resources. The register capability environment
$\Delta_{\text{reg}}$ tracks register ownership across instruction sequences:

$$\Delta_{\text{reg}} \vdash \text{instr} : \Delta_{\text{reg}}'$$

Callee-saved registers clobbered without explicit stack spill and restore emit compile-time errors.

### §63. Fixed-Width Vector Types (`Vec128`, `Vec256`, `Vec512`) and SIMD Semantics

Direct vector primitives correspond exactly to hardware vector registers:
- `Vec128[T]`: 128-bit vector register (`%xmm` on x86-64, `v` on AArch64).
- `Vec256[T]`: 256-bit vector register (`%ymm` on AVX2).
- `Vec512[T]`: 512-bit vector register (`%zmm` on AVX-512).

#### Lane Partitioning Invariant
$$\text{Lanes}(\text{Vec}W[T]) = \frac{W}{\text{Bits}(T)}$$

Lane operations are algebraically verified to execute in parallel without cross-lane interference:
$$(\vec{a} +_{\text{simd}} \vec{b})[i] = (\vec{a}[i] + \vec{b}[i]) \bmod 2^{\text{Bits}(T)}$$

### §64. Hardware Cryptographic Intrinsics (AES-NI, ARMv8 Crypto, PCLMULQDQ, PMULL, SHA-NI, Zkne)

Hardware cryptographic primitives are exposed as strongly typed intrinsic functions:

```orange
spec intrinsic_aes_enc(state: Vec128[Word[8]], round_key: Vec128[Word[8]]) -> Vec128[Word[8]]
```

#### 1. AES Encryption Round (`aes_enc`)
Formally defined over state $S \in \text{GF}(2^8)^{16}$ and key $K \in \text{GF}(2^8)^{16}$:
$$\text{intrinsic\_aes\_enc}(S, K) = \text{AddRoundKey}(\text{MixColumns}(\text{ShiftRows}(\text{SubBytes}(S))), K)$$
- Lowers to `aesenc` on x86-64.
- Lowers to `aese` followed by `aesmc` on AArch64.
- Lowers to `aes64esm` on RISC-V Zkne.

#### 2. Carryless Multiplication (`carryless_mul`)
Multiplication of two 64-bit polynomials in Galois field $\text{GF}(2)[x]$:
$$C(x) = A(x) \cdot B(x) \pmod 0 \quad (\text{producing 128-bit product})$$
- Lowers to `pclmulqdq` on x86-64.
- Lowers to `pmull` on AArch64 (FEAT_PMULL).
- Lowers to `clmul` on RISC-V Zbkc.

#### 3. SHA-256 Compression Round (`sha256_rnd`)
Computes two rounds of SHA-256 compression over vector registers in hardware:
- Lowers to `sha256rnds2` on x86-64 (SHA-NI).
- Lowers to `sha256h` / `sha256h2` on AArch64 (FEAT_SHA2).

### §65. Target Profiles: x86-64 System V, AArch64 AAPCS64, RV64GC

| Parameter | Profile `T-X64` | Profile `T-A64` | Profile `T-RV64` |
| :--- | :--- | :--- | :--- |
| **Architecture** | `x86_64-unknown-linux-gnu` | `aarch64-unknown-linux-gnu` | `riscv64-unknown-linux-gnu` |
| **ABI Standard** | System V AMD64 psABI | AAPCS64 | RISC-V ELF psABI |
| **Data Model** | LP64 | LP64 | LP64D |
| **Baseline ISA** | x86-64-v3 | ARMv8.0-A | RV64GC |
| **Crypto Extensions** | AES-NI, PCLMULQDQ, SHA-NI | FEAT_AES, FEAT_PMULL, FEAT_SHA2 | Zkne, Zknd, Zbkc, Zkt |
| **Hardware DIT** | Intel DOITM | ARMv8.4-A `msr dit, 1` | Ratified RISC-V Zkt |

### §66. Instruction Latency Classification and Hardware Data-Independent Timing

1. **The Latency Invariant:**
   Instructions executed on secret data MUST have latency independent of operand bits:
   $$\forall \text{operands } op_1, op_2.\ \text{Cycles}(\text{Instr}(op_1)) = \text{Cycles}(\text{Instr}(op_2))$$
2. **DIT Directive Injection:**
   On architectures featuring data-independent timing controls, the compiler enforces DIT:
   - On AArch64: Emits `msr dit, 1` at procedure entry and verifies invariant maintenance.
   - On x86-64: Enforces execution within processor-specific constant-time mode (DOITM).
   - Variable-latency ALU instructions (e.g. non-constant shift circuits, integer divide)
     operating on secret values trigger compile-time rejection under policy `ct-variable-latency-v1`.

---

## Part X: Information Flow, Secrecy & Microarchitectural Leakage

### §67. The Information Flow Lattice ($\text{public} \sqsubseteq \text{secret}$)

Confidentiality is formalized via a two-point security lattice:

$$\mathcal{L} = \langle \{\text{public}, \text{secret}\}, \sqsubseteq, \sqcup, \sqcap \rangle$$

where $\text{public} \sqsubseteq \text{secret}$.

1. **Flow Rule:** Information may flow from public to secret, but never from secret to public:
   $$L_1 \sqsubseteq L_2 \implies \text{Assignment } (x_{L_2} := e_{L_1}) \text{ is permitted}$$
   $$L_{\text{secret}} \not\sqsubseteq L_{\text{public}} \implies \text{Assignment } (x_{\text{public}} := e_{\text{secret}}) \text{ is REJECTED}$$
2. **Label Propagation:**
   $$\text{Label}(a \mathbin{\text{op}} b) = \text{Label}(a) \sqcup \text{Label}(b)$$
   Any computation consuming at least one secret operand yields a secret result.

### §68. Microarchitectural Trace Semantics and Observational Noninterference

We formalize side-channel security via microarchitectural observation traces.

#### 1. Microarchitectural Event Alphabet ($\Sigma_{\text{leak}}$)
$$\Sigma_{\text{leak}} = \left\{
\begin{array}{ll}
\mathbf{Fetch}(\text{pc}), & (\text{instruction fetch at address pc}) \\
\mathbf{Branch}(\text{pc}, \text{target}), & (\text{control branch taken to target}) \\
\mathbf{MemRead}(\text{addr}, \text{width}), & (\text{memory bus read at address}) \\
\mathbf{MemWrite}(\text{addr}, \text{width}), & (\text{memory bus write at address}) \\
\mathbf{ALULatency}(\text{op}, c), & (\text{ALU instruction execution taking } c \text{ cycles}) \\
\mathbf{SpecBarrier}(\text{pc}) & (\text{speculation fence executed})
\end{array}
\right\}$$

#### 2. Trace Generation Semantics
Small-step execution produces traces of leakage events:

$$\langle s, \sigma \rangle \xrightarrow{\tau} \langle s', \sigma' \rangle \quad (\tau \in \Sigma_{\text{leak}}^*)$$

#### 3. Public Memory Equivalence ($\sim_{\text{public}}$)
Two initial memory configurations $\sigma_1, \sigma_2$ are publicly equivalent:

$$\sigma_1 \sim_{\text{public}} \sigma_2 \iff \forall p \in \text{dom}(\sigma_1) \cap \text{dom}(\sigma_2).\ \text{Label}(p) = \text{public} \implies \sigma_1(p) = \sigma_2(p)$$

#### 4. The Cryptographic Observational Noninterference Theorem

**Theorem 7 (Cryptographic Noninterference):**
*Let procedure $P$ be verified under noninterference policy $\mathcal{P}$. For all initial states $\sigma_1, \sigma_2$ such that $\sigma_1 \sim_{\text{public}} \sigma_2$:*
$$\text{Trace}_{\mathcal{P}}(P, \sigma_1) = \text{Trace}_{\mathcal{P}}(P, \sigma_2)$$

*The emitted observation traces are bitwise identical. Consequently, any adversary observing instruction timing, memory bus addresses, branch target buffers, and cache lines obtains exactly zero Shannon mutual information regarding secret inputs:*
$$I(\text{Secrets}; \text{Trace}_{\mathcal{P}}) = 0$$

*Proof.*
We induct on the length of execution traces $k$:
1. **Base Case ($k = 0$):** Empty traces are identical.
2. **Instruction Fetch:** Program counters depend only on control flow. Under policy `ct-architectural-v1`,
   branch conditions MUST have label `public`. Since $\sigma_1 \sim_{\text{public}} \sigma_2$,
   branch conditions evaluate identically in both executions. Thus $\text{pc}_1 = \text{pc}_2$,
   producing identical $\mathbf{Fetch}$ and $\mathbf{Branch}$ events.
3. **Memory Accesses:** Load and store addresses are typed `public`. Since all public variables
   agree ($\sigma_1(p) = \sigma_2(p)$), calculated memory addresses are identical, producing
   identical $\mathbf{MemRead}(\text{addr})$ and $\mathbf{MemWrite}(\text{addr})$ events.
4. **ALU Operations:** Under policy `ct-variable-latency-v1`, all operations consuming secret
   data run in constant cycles independent of operand bits ($c_1 = c_2$), emitting identical
   $\mathbf{ALULatency}(\text{op}, c)$ events.
By induction, the traces are strictly identical for all execution steps. Mutual information
is identically zero. $\blacksquare$

### §69. Architectural Noninterference Policy: `ct-architectural-v1`

Enforces classical constant-time programming rules:
1. **Branch Noninterference:**
   $$\Gamma \vdash c : \text{Bool} \land \text{Label}(c) = \text{secret} \implies \text{Compilation Error}$$
2. **Address Noninterference:**
   $$\Gamma \vdash p[i] \land \text{Label}(i) = \text{secret} \implies \text{Compilation Error}$$
   Secret-dependent table lookups are strictly impossible to compile.
3. **Loop Bound Invariance:**
   Loop iteration counts must depend exclusively on public constants.

### §70. Hardware ALU Latency Policy: `ct-variable-latency-v1` (DOITM, DIT, Zkt)

Extends `ct-architectural-v1` by inspecting target CPU execution units:
1. Instructions whose hardware latency varies based on operand data (e.g. `div`, `idiv`,
   floating-point operations, bit-serial shift loops) are rejected if any input carries `secret`.
2. Verifies that CPU data-independent timing modes (DIT, DOITM, Zkt) are active.

### §71. Speculative Noninterference Policy: `ct-speculative-v1`

Extends noninterference to transient and speculative execution (Spectre-v1, Spectre-v4):
1. **Speculative Execution Model:** Branch predictors may speculatively execute up to
   $W_{\text{spec}} = 256$ instructions past an unresolved branch.
2. **Transient Leakage:** Speculative memory reads that access secret-dependent addresses
   leave persistent microarchitectural traces in CPU cache hierarchy tags.
3. **Barrier Insertion Invariant:** The compiler verifies the placement of architectural
   speculation barriers (`lfence` on x86-64, `csdb` on AArch64) before any memory dereference
   following a conditional boundary that depends on secret data.

### §72. Declassification Contracts, Audit Ledgers, and Flow Gates

1. Releasing secret information (e.g. digital signatures, ciphertexts, MAC tags) requires
   an explicit declassification gate:
   ```orange
   let pub_sig = declassify(sig, policy: "ed25519-signature-release");
   ```
2. **Audit Ledger:** Every declassification event is appended to the compilation unit's
   cryptographic evidence ledger, identifying:
   - Source place and secret type.
   - Formal declassification justification policy.
   - Hash of the governing assurance claim.

---

## Part XI: Cryptographic Game Stratum (`game`)

### §73. Monadic Probabilistic Semantics and Distribution Ensembles

The `game` stratum formalizes cryptographic games, adversary interactions, and
computational reductions.

#### 1. Discrete Probability Sub-Monad ($\mathcal{D}$)
Computations in `game` possess a monadic semantics over finite sample spaces:

$$\mathcal{D}(A) = \left\{ \mu : A \to [0, 1] \;\middle|\; \sum_{x \in \operatorname{supp}(\mu)} \mu(x) = 1 \right\}$$

- **Unit / Return ($\eta$):**
  $$\eta(a) = \lambda x.\ \begin{cases} 1 & \text{if } x = a \\ 0 & \text{otherwise} \end{cases}$$
- **Bind ($\gg=$):**
  For $\mu \in \mathcal{D}(A)$ and $f : A \to \mathcal{D}(B)$:
  $$(\mu \gg= f)(b) = \sum_{a \in A} \mu(a) \cdot f(a)(b)$$

#### 2. Uniform Sampling Operator
Sampling selects an element uniformly at random from a finite set $S$:

$$x \sample S \iff \Pr[X = s] = \frac{1}{|S|} \quad (\forall s \in S)$$

### §74. Stateful Oracles, Adversaries, and Black-Box Encapsulation

1. A **game** encapsulates private state and exposes public **oracles**:

```orange
game PRF_Game {
    var key: Word[8]^32;
    var b: Bool;
    var table: Map[Word[8]^16, Word[8]^16];

    oracle init() {
        key = sample Word[8]^32;
        b = sample Bool;
    }

    oracle eval(x: Word[8]^16) -> Word[8]^16 {
        if b {
            spec::aes128_encrypt(key, x)
        } else {
            if !table.contains(x) {
                table[x] = sample Word[8]^16;
            }
            table[x]
        }
    }
}
```

2. **Adversary Encapsulation:**
   An adversary $\mathcal{A}$ has black-box query access to public oracles under a bounded
   query budget $(q, t)$, where $q$ is the query limit and $t$ is execution time.
   Internal variables (`key`, `b`) are inaccessible to $\mathcal{A}$.

### §75. Sequence-of-Games Reductions and Concrete Advantage Bounding

Cryptographic proofs are structured as sequences of game hops:
$$G_0 \xrightarrow{\text{hop}_1} G_1 \xrightarrow{\text{hop}_2} \dots \xrightarrow{\text{hop}_k} G_k$$

#### 1. Shoup's Fundamental Difference Lemma

**Lemma 7.1 (Fundamental Difference Lemma of Game-Playing):**
*Let $G_1$ and $G_2$ be two games defined over identical random choices, identical state variables, and identical oracle interfaces, such that their execution rules are identical unless a boolean flag $\text{Bad}$ is set. For any adversary $\mathcal{A}$ and event $E$:*
$$|\Pr[G_1^{\mathcal{A}} \Rightarrow E] - \Pr[G_2^{\mathcal{A}} \Rightarrow E]| \le \Pr[G_1^{\mathcal{A}} \text{ sets Bad}] = \Pr[G_2^{\mathcal{A}} \text{ sets Bad}]$$

*Proof.*
By the Law of Total Probability, condition on the event $\text{Bad}$:
$$\Pr[G_1 \Rightarrow E] = \Pr[G_1 \Rightarrow E \land \neg\text{Bad}] + \Pr[G_1 \Rightarrow E \land \text{Bad}]$$
$$\Pr[G_2 \Rightarrow E] = \Pr[G_2 \Rightarrow E \land \neg\text{Bad}] + \Pr[G_2 \Rightarrow E \land \text{Bad}]$$
Since $G_1$ and $G_2$ proceed identically as long as $\text{Bad}$ does not occur:
$$\Pr[G_1 \Rightarrow E \land \neg\text{Bad}] = \Pr[G_2 \Rightarrow E \land \neg\text{Bad}]$$
Subtracting the two probabilities:
$$|\Pr[G_1 \Rightarrow E] - \Pr[G_2 \Rightarrow E]| = |\Pr[G_1 \Rightarrow E \land \text{Bad}] - \Pr[G_2 \Rightarrow E \land \text{Bad}]| \le \max(\Pr[G_1 \land \text{Bad}], \Pr[G_2 \land \text{Bad}]) \le \Pr[\text{Bad}]$$
$\blacksquare$

#### 2. Concrete Advantage Bounding Theorem
In Orange, game reductions produce concrete arithmetic bounds:

$$\mathbf{Adv}_{\text{Scheme}}^{\text{IND-CPA}}(\mathcal{A}) \le 2 \cdot \mathbf{Adv}_{\text{BlockCipher}}^{\text{PRF}}(\mathcal{B}) + \frac{q^2 \cdot L^2}{2^{128}}$$

where $q$ is the number of encryption queries and $L$ is block length.

---

## Part XII: Deductive Proof System & Metatheory (`proof`)

### §76. Propositions as Types and the $\text{Prop}$ Universe

The `proof` stratum implements an intuitionistic dependent type theory formalizing
the Curry-Howard correspondence:

$$\text{Propositions-as-Types} \qquad \text{Proofs-as-Terms}$$

1. **The $\text{Prop}$ Universe:**
   - Resides at the base of the universe hierarchy: $\text{Prop} : \text{Type}_1 : \text{Type}_2 \dots$
   - Any proposition $P : \text{Prop}$ is proved by constructing an elaborated term $\pi$ such that:
     $$\vdash \pi : P$$
2. **Definitional Proof Irrelevance:**
   Any two proofs of the same proposition are definitionally equal:
   $$\forall \pi_1, \pi_2 : P.\ \pi_1 \equiv \pi_2 \quad (\text{for } P : \text{Prop})$$
   Proof terms are erased during native code generation, leaving zero footprint.

### §77. Functional Refinement Relations ($\text{impl } P \sqsubseteq \text{spec } S$)

#### Definition 1 (Functional Refinement)
*An imperative procedure $P$ functionally refines a mathematical specification $S$ ($P \sqsubseteq S$) under precondition $\text{Pre}$ and postcondition $\text{Post}$ if for all initial stores $\sigma_0$ such that $\sigma_0 \models \text{Pre}$:*
1. **Totality:** $\exists \sigma_f.\ \langle P, \sigma_0 \rangle \Downarrow \sigma_f$ (execution terminates).
2. **Memory Safety:** Execution accesses only valid places in $\text{dom}(\sigma)$ without out-of-bounds or invalid borrows.
3. **Semantic Equivalence:** $\text{Observable}(\sigma_f) = S(\text{Inputs}(\sigma_0))$.

### §78. Proof IR: Canonical Encoding, De Bruijn Terms, and Cryptographic Fingerprints

1. **Proof IR Representation:**
   Proof terms are lowered into an explicit, tactic-free intermediate representation
   using de Bruijn indices for bound variables.
2. **$\beta\eta\iota$-Normalization:**
   Proof checking computes weak-head normal forms to verify term equivalence.
3. **Cryptographic Fingerprinting:**
   Every verified lemma receives an immutable content-addressed cryptographic fingerprint:
   $$\text{Fingerprint}(\text{Thm}) = \text{SHA-256}\Big(\text{Term} \mathbin{\Vert} \text{Axioms} \mathbin{\Vert} \text{Edition} \mathbin{\Vert} \text{TargetProfile}\Big)$$

### §79. Weakest Precondition Calculus $\text{wp}(S, Q)$ and Verification Conditions

Verification conditions (VCs) are generated via Dijkstra's weakest precondition transformer:

$$\text{wp}(s, Q) : \text{Store} \to \text{Prop}$$

#### 1. Inductive Equations for $\text{wp}$
- $\text{wp}(\texttt{skip}, Q) = Q$
- $\text{wp}(x := e, Q) = Q[e / x]$
- $\text{wp}(p[i] := e, Q) = Q[\sigma \oplus (p[i] \mapsto e) / \sigma]$
- $\text{wp}(s_1; s_2, Q) = \text{wp}(s_1, \text{wp}(s_2, Q))$
- $\text{wp}(\text{if } b \{ s_1 \} \text{ else } \{ s_2 \}, Q) = (b \implies \text{wp}(s_1, Q)) \land (\neg b \implies \text{wp}(s_2, Q))$
- $\text{wp}(\text{while } b \text{ inv } I \text{ var } V \{ s \}, Q) = I \land \forall \sigma.\ \Big((I \land b \implies \text{wp}(s, I \land V < \text{old}(V))) \land (I \land \neg b \implies Q)\Big)$

#### 2. Soundness of Weakest Preconditions

**Theorem 8 (Soundness of $\text{wp}$):**
*If $\text{Pre} \implies \text{wp}(s, \text{Post})$, then the Hoare triple $\{\text{Pre}\} \ s \ \{\text{Post}\}$ is valid.*

*Proof.*
By induction on the structure of statement $s$. For atomic assignments, substitution
corresponds to memory store updates. For conditionals, the conjunction covers both
branch conditions. For while loops, invariant preservation and variant decrease guarantee
correctness upon bounded termination. $\blacksquare$

### §80. Certificate-Producing Automation: LRAT/DRAT and LFSC/SMT-LIB Reconstruction

1. Automated SAT/SMT solvers are used strictly as untrusted proof search engines.
2. **Proof Reconstruction:**
   - SAT solvers MUST emit DRAT or LRAT clausal proof certificates.
   - SMT solvers MUST emit LFSC (Logical Framework with Side Conditions) or Alethe certificates.
   - The Orange proof reconstructor parses certificates and produces Proof IR lambda terms.
3. Solvers are excluded from the Trusted Computing Base (TCB).

### §81. Authoritative Offline Checker: `orange-check` Kernel and TCB Boundary

1. The ultimate arbiter of correctness is **`orange-check`**, a standalone verification kernel.
2. **TCB Invariants of `orange-check`:**
   - **Zero Network Access:** Runs air-gapped without sockets.
   - **Zero Compilers:** Invokes no C compilers, assemblers, or linkers.
   - **Formally Verified Kernel:** The core type-checking algorithm is verified in Rocq/Lean.
   - **Closure Audit:** Emits an unabridged ledger of all axioms and hardware profiles
     supporting any assurance claim.

---

## Part XIII: Assurance Claims & Evidence Architecture (`claim`)

### §82. The Philosophy of Atomic Claims ("Claims, Not Labels")

Orange rejects qualitative, coarse-grained security labels (such as "verified" or "secure")
in favor of fine-grained, mathematically verifiable **atomic assurance claims**:

```orange
claim chacha20_functional_refinement {
    subject:      impl::chacha20_block,
    relation:     refines(spec::chacha20_block),
    target:       "x86_64-unknown-linux-gnu",
    policy:       "ct-architectural-v1",
    basis:        kernel_proof(proof::chacha20_refinement_proof),
    outcome:      satisfied
}
```

Every claim is an immutable cryptographic contract binding a subject symbol to a formal
relation, target architectural profile, noninterference policy, and checkable evidentiary basis.

### §83. The Ten Mandatory Claim Families (CF-01 through CF-10)

Assurance graphs in Orange are partitioned into ten orthogonal, normative claim families:

| ID | Family | Formal Relational Predicate | Scope and Verification Artifact |
| :--- | :--- | :--- | :--- |
| **CF-01** | `conforms` | $\text{Conforms}(P, \text{Standard}, \text{Profile})$ | Standard conformance clauses, RFC/NIST test vectors. |
| **CF-02** | `refines` | $P \sqsubseteq S \iff \forall \sigma_0 \models \text{Pre}.\ \text{Obs}(\text{Exec}(P, \sigma_0)) = S(\text{In}(\sigma_0))$ | Functional refinement of mathematical specification. |
| **CF-03** | `safe` | $\text{Safe}(P) \iff \forall \sigma_0 \models \text{Pre}.\ \text{NoTraps}(P, \sigma_0) \land \text{SpatialTemporalSafety}(P)$ | Memory safety, panic freedom, absence of dangling references. |
| **CF-04** | `terminates` | $\text{Terminates}(P) \iff \exists k \le K_{\max}.\ \text{Steps}(P, \sigma_0) = k$ | Provable bounded termination within finite step budget. |
| **CF-05** | `leakage` | $\text{Noninterference}(P, \mathcal{P}) \iff \forall \sigma_1 \sim_{\text{pub}} \sigma_2.\ \text{Trace}_{\mathcal{P}}(\sigma_1) = \text{Trace}_{\mathcal{P}}(\sigma_2)$ | Observational noninterference under policy $\mathcal{P}$. |
| **CF-06** | `compiled` | $\text{CompiledCorrectly}(P, \text{Target}, \text{ObjectBytes})$ | Translation validation: compiler preserves semantics to machine code. |
| **CF-07** | `abi` | $\text{ConformsABI}(P, \text{PlatformABI}, \text{HeaderFile})$ | Layout, calling convention, alignment, and register rules. |
| **CF-08** | `erases` | $\text{Erases}(P, \text{SecretPlaces}) \iff \forall p \in \text{Secrets}.\ \text{FinalVal}(p) = \mathbf{0}$ | Mandatory storage zeroization of private keys and secrets. |
| **CF-09** | `security` | $\mathbf{Adv}_{\text{Scheme}}^{\text{Goal}}(\mathcal{A}) \le \mathcal{B}(\mathbf{Adv}_{\text{Primitive}}^{\text{Assump}}, q, t)$ | Game-based computational reduction bounding adversary advantage. |
| **CF-10** | `test_result`| $\text{TestsPass}(P, \text{VectorSet}) \iff \bigwedge_{v \in \text{Vectors}} (P(v.\text{in}) == v.\text{out})$ | Known-answer test vectors and differential fuzzing results. |

### §84. Claim Record Schema, 4-Valued Outcome Algebra, and Evidentiary Bases

#### 1. Four-Valued Outcome Bilattice ($\mathcal{B}_4$)
Claim outcomes reside in Ginsberg's 4-valued bilattice:

$$\mathcal{B}_4 = \langle \{\mathbf{satisfied}, \mathbf{not\_satisfied}, \mathbf{unresolved}, \mathbf{unsupported}\}, \le_t, \le_k \rangle$$

ordered along two orthogonal axes:
- **Truth Ordering ($\le_t$):** $\mathbf{not\_satisfied} \le_t \mathbf{unresolved}, \mathbf{unsupported} \le_t \mathbf{satisfied}$.
- **Knowledge Ordering ($\le_k$):** $\mathbf{unresolved} \le_k \mathbf{unsupported} \le_k \mathbf{satisfied}, \mathbf{not\_satisfied}$.

#### Bilattice Truth Tables

| $\land$ (Conjunction) | $\mathbf{satisfied}$ | $\mathbf{unresolved}$ | $\mathbf{unsupported}$ | $\mathbf{not\_satisfied}$ |
| :---: | :---: | :---: | :---: | :---: |
| $\mathbf{satisfied}$ | $\mathbf{satisfied}$ | $\mathbf{unresolved}$ | $\mathbf{unsupported}$ | $\mathbf{not\_satisfied}$ |
| $\mathbf{unresolved}$ | $\mathbf{unresolved}$ | $\mathbf{unresolved}$ | $\mathbf{unsupported}$ | $\mathbf{not\_satisfied}$ |
| $\mathbf{unsupported}$| $\mathbf{unsupported}$| $\mathbf{unsupported}$| $\mathbf{unsupported}$ | $\mathbf{not\_satisfied}$ |
| $\mathbf{not\_satisfied}$ | $\mathbf{not\_satisfied}$ | $\mathbf{not\_satisfied}$ | $\mathbf{not\_satisfied}$ | $\mathbf{not\_satisfied}$ |

A compound claim $C_1 \land C_2$ is $\mathbf{satisfied}$ if and only if both constituent claims
are independently $\mathbf{satisfied}$. Any falsified premise immediately collapses the outcome
to $\mathbf{not\_satisfied}$.

#### 2. Normative Claim Record Schema
```json
{
  "$schema": "https://schemas.orange-lang.org/2026/claim-record.json",
  "type": "object",
  "required": ["claim_id", "family", "subject", "relation", "context", "outcome", "bases"],
  "properties": {
    "claim_id": { "type": "string", "pattern": "^[a-z0-9_-]+$" },
    "family": { "type": "string", "enum": ["conforms", "refines", "safe", "terminates", "leakage", "compiled", "abi", "erases", "security", "test_result"] },
    "subject": {
      "type": "object",
      "required": ["symbol", "artifact_digest"],
      "properties": {
        "symbol": { "type": "string" },
        "artifact_digest": { "type": "string", "pattern": "^sha256:[0-9a-f]{64}$" }
      }
    },
    "relation": { "type": "string" },
    "context": {
      "type": "object",
      "required": ["target", "policy", "edition"],
      "properties": {
        "target": { "type": "string" },
        "policy": { "type": "string" },
        "edition": { "type": "string", "enum": ["2026"] }
      }
    },
    "outcome": { "type": "string", "enum": ["satisfied", "not_satisfied", "unresolved", "unsupported"] },
    "bases": {
      "type": "array",
      "minItems": 1,
      "items": {
        "type": "object",
        "required": ["type", "evidence_digest", "verification_state"],
        "properties": {
          "type": { "type": "string", "enum": ["kernel_proof", "checked_certificate", "external_proof", "test_run", "audit", "external_validation", "assumption"] },
          "evidence_digest": { "type": "string", "pattern": "^sha256:[0-9a-f]{64}$" },
          "verification_state": { "type": "string", "enum": ["checked", "unverified", "failed"] }
        }
      }
    },
    "axiom_closure": {
      "type": "array",
      "items": { "type": "string" }
    }
  }
}
```

### §85. Evidence Bundles: Thin Manifests and Thick Content-Addressed `.orange-evidence`

1. **Thin Evidence Manifest:**
   A high-speed developer ledger (`evidence.json`) containing cryptographic digests of
   all sources, Core IR modules, and verified claims.
2. **Thick Content-Addressed Bundle (`.orange-evidence`):**
   An immutable archival container supporting air-gapped forensic audit:
   ```text
   release.orange-evidence/
   +-- manifest.json                 (root manifest with cryptographic digests)
   +-- blobs/                        (content-addressed store: sha256/<hash>)
   |   +-- sources/                  (exact normalized UTF-8 source files)
   |   +-- core/                     (monomorphic Typed Reference Core IR modules)
   |   +-- proof_ir/                 (elaborated Proof IR lambda objects)
   |   +-- certificates/             (LRAT/DRAT and LFSC proof certificates)
   |   +-- objects/                  (native ELF/Mach-O .o binary artifacts)
   |   +-- headers/                  (generated C11 headers)
   +-- claims/                       (normative JSON claim graph records)
   +-- provenance/                   (CycloneDX 1.6 CBOM, SPDX 3.0 SBOM, SLSA attestations)
   ```

### §86. CBOM (CycloneDX 1.6), SPDX SBOM, and SLSA Level 3/4 Build Provenance

1. **CycloneDX 1.6 CBOM (Cryptography Bill of Materials):**
   Formally inventories every cryptographic asset:
   - Unique Algorithm Object Identifiers (OID).
   - Primitive classification (symmetric stream cipher, prime field arithmetic, MAC, AEAD).
   - Cryptographic parameter bounds: key length (bits), security strength $\lambda$ (classical/quantum).
   - Verification status: functional refinement claims and noninterference policies.
2. **SLSA Level 3/4 Build Provenance:**
   Cryptographically signed in-toto attestations guaranteeing that every binary was
   compiled deterministically from audited source trees in hermetic build environments.

### §87. The Minimal TCB Calculus (`orange trust`)

The Trusted Computing Base (TCB) represents the minimal set of code and assumptions
that must be trusted for a verification claim to hold.

#### 1. Formal Definition of the TCB
For any claim $C$:

$$\text{TCB}(C) = \text{Kernel}(\texttt{orange-check}) \cup \text{TargetModel}(C) \cup \bigcup_{c \in \operatorname{Ancestors}(C)} \Big(\text{Axioms}(c) \cup \text{Assumptions}(c)\Big)$$

#### 2. TCB Graph Reduction Algorithm
```text
Algorithm: ComputeMinimalTCB(claim_id, ClaimGraph)
Input:  claim_id in ClaimGraph.Nodes
Output: Minimal TCB Record (KernelDigest, Axioms, Assumptions, ForeignContracts)

1. visited := empty_set()
2. tcb_axioms := empty_set()
3. tcb_assumptions := empty_set()
4. queue := [ claim_id ]

5. while queue is not empty:
     current_id := queue.pop_front()
     if current_id in visited:
         continue
     visited.insert(current_id)
     node := ClaimGraph.get(current_id)

     // Accumulate axioms and unproved assumptions
     for ax in node.axiom_closure:
         tcb_axioms.insert(ax)
     for basis in node.bases:
         if basis.type == "assumption":
             tcb_assumptions.insert(basis)
         else if basis.type == "kernel_proof":
             assert orange_check.verify(basis.evidence_digest) == true

     // Traverse upstream claim dependencies
     for dep in node.dependencies:
         queue.push_back(dep)

6. return {
     kernel: orange_check.kernel_digest(),
     target_model: ClaimGraph.get(claim_id).context.target,
     axioms: tcb_axioms,
     assumptions: tcb_assumptions
   }
```

Components excluded from the TCB:
- The Orange compiler parser, type elaborator, and optimization pipeline.
- SMT and SAT solvers (Z3, CVC5, CaDiCaL) — only their emitted certificates are verified.
- The host build system, shell, and IDE plugins.

---

## Part XIV: Foreign Function Interface & ABI

### §88. Sound Foreign Interface Principles and Import Contracts

1. External C routines cannot be called without an explicit two-state contract:
   ```orange
   extern c fn get_random_bytes(buf: &mut Word[8]^32) -> Result[(), EntropyError]
   requires buf.len == 32
   ensures  buf.is_initialized()
   ```
2. Unverified foreign imports are classified as `assumption` evidentiary bases,
   preventing unverified foreign code from masquerading as verified kernels.

### §89. Standard C ABI Layouts, Packing, and Alignment (x86-64, AArch64, RV64)

Orange procedures compile to standard platform C ABIs:
- **x86-64:** System V AMD64 psABI (LP64).
- **AArch64:** Standard ARM 64-bit Architecture ABI (AAPCS64, LP64).
- **RISC-V:** RISC-V ELF psABI (LP64D).

#### 1. Algebraic Compound Data Layout Calculus
For any type $\tau$, its size, alignment, and field offsets are defined inductively:

$$\text{align}(\tau) = \begin{cases}
W / 8 & \text{if } \tau = \text{Word}[W] \quad (W \in \{8, 16, 32, 64\}) \\
1 & \text{if } \tau = \text{Bool} \\
\text{align}(T) & \text{if } \tau = T^n \\
\max_{0 \le i < k} \text{align}(T_i) & \text{if } \tau = (T_0, \dots, T_{k-1})
\end{cases}$$

$$\text{offset}(T_0) = 0 \qquad \text{offset}(T_{i+1}) = \text{align\_up}\Big(\text{offset}(T_i) + \text{sizeof}(T_i), \ \text{align}(T_{i+1})\Big)$$

$$\text{sizeof}(T^n) = n \cdot \text{sizeof}(T) \qquad \text{sizeof}(T_0, \dots, T_{k-1}) = \text{align\_up}\Big(\text{offset}(T_{k-1}) + \text{sizeof}(T_{k-1}), \ \text{align}(\tau)\Big)$$

where $\text{align\_up}(x, a) = (x + a - 1) \mathbin{\&} \sim(a - 1)$.

#### 2. Primitive Type ABI Mapping Table

| Orange Type | C11 Standard Equivalent | Size (Bytes) | Alignment (Bytes) | Calling Convention Register Class |
| :--- | :--- | :---: | :---: | :--- |
| `Word[8]` / `Byte` | `uint8_t` | 1 | 1 | Integer GPR (zero-extended) |
| `Word[16]` | `uint16_t` | 2 | 2 | Integer GPR (zero-extended) |
| `Word[32]` | `uint32_t` | 4 | 4 | Integer GPR (zero-extended) |
| `Word[64]` | `uint64_t` | 8 | 8 | Integer GPR |
| `Bool` | `uint8_t` (`0`=false, `1`=true) | 1 | 1 | Integer GPR (zero-extended) |
| `Word[W]^n` | Contiguous array `uintW_t[n]` | $n \times (W/8)$ | $W/8$ | Pointer passed in Integer GPR |
| `&T^n` | Pointer to const buffer `const T*` | 8 | 8 | Integer GPR |
| `&mut T^n` | Pointer to mutable buffer `T*` | 8 | 8 | Integer GPR |
| `Vec128[Word[32]]` | Vector type `__m128i` / `uint32x4_t` | 16 | 16 | Vector / SIMD register (`%xmm`, `v`) |
| `Vec256[Word[64]]` | Vector type `__m256i` | 32 | 32 | Vector register (`%ymm`) |

### §90. Register-Passing Conventions, Stack Frames, and Red Zones

#### 1. Calling Convention Register Allocations
- **System V AMD64:**
  - Argument registers (in order): `%rdi`, `%rsi`, `%rdx`, `%rcx`, `%r8`, `%r9`.
  - Vector argument registers: `%xmm0` through `%xmm7`.
  - Return registers: `%rax` (primary), `%rdx` (secondary).
  - Callee-saved registers: `%rbx`, `%rsp`, `%rbp`, `%r12`, `%r13`, `%r14`, `%r15`.
- **AAPCS64:**
  - Argument registers: `x0` through `x7`.
  - Vector argument registers: `v0` through `v7`.
  - Return registers: `x0` (primary), `x1` (secondary).
  - Callee-saved registers: `x19` through `x28`, `x29` (FP), `x30` (LR).
- **RISC-V RV64GC:**
  - Argument registers: `a0` through `a7`.
  - Return registers: `a0` (primary), `a1` (secondary).
  - Callee-saved registers: `s0` through `s11`, `sp`.

#### 2. Stack Frame Diagram and 16-Byte Alignment Invariant

```text
Higher Addresses (Caller Stack Frame)
+-------------------------------------------------------+
|  Incoming Stack Arguments (arg 7+)                    |
+-------------------------------------------------------+
|  Return Address (pushed by CALL instruction)          |  <-- Stack Top at Entry
+-------------------------------------------------------+
|  Saved Frame Pointer (%rbp / x29)                     |  <-- Frame Pointer (%rbp)
+-------------------------------------------------------+
|  Callee-Saved Registers (%r12-%r15, %rbx)             |
+-------------------------------------------------------+
|  Local Bounded Arrays and Scratch Buffers             |
|  (Fixed static size, zero-initialized on scope exit)  |
+-------------------------------------------------------+
|  Temporary Spill Slots (Allocated by compiler)        |
+-------------------------------------------------------+  <-- Stack Pointer (%rsp)
Lower Addresses (16-Byte Aligned Boundary)
```

The 128-byte System V red zone below `%rsp` is **disabled** in cryptographic procedures
(compiled with `-mno-red-zone`). This ensures that asynchronous signal handlers and hardware
interrupts cannot overwrite sensitive keys or intermediate states residing on the stack.

### §91. Generated C11 Headers, Symbol Mangling, and Preconditions

1. **Symbol Mangling Rule:**
   $$\text{Symbol} = \text{"orange\_"} \mathbin{\Vert} \text{ModuleName} \mathbin{\Vert} \text{"\_"} \mathbin{\Vert} \text{FunctionName}$$
2. **Canonical C11 Header Specification:**

```c
#ifndef ORANGE_CHACHA20_H
#define ORANGE_CHACHA20_H

#include <stdint.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

/*
 * Module: chacha20
 * Generated by orangec (Orange Edition 2026)
 * Formal Refinement: claim::chacha20_functional_refinement (SATISFIED)
 * Leakage Policy:    ct-architectural-v1 (NONINTERFERENCE VERIFIED)
 */

_Static_assert(sizeof(uint8_t) == 1, "uint8_t size mismatch");
_Static_assert(sizeof(uint32_t) == 4, "uint32_t size mismatch");

void orange_chacha20_block(
    const uint8_t key[32],
    const uint8_t nonce[12],
    uint32_t counter,
    uint8_t out[64]
) __attribute__((nonnull(1, 2, 4)));

#ifdef __cplusplus
}
#endif

#endif /* ORANGE_CHACHA20_H */
```

### §92. Zero-Overhead Safe Rust Bindings

Orange generates idiomatic, zero-overhead safe Rust bindings wrapping the C symbols:

```rust
// Auto-generated safe Rust wrapper for orange_chacha20
// Verified functional refinement against spec::chacha20
pub mod chacha20 {
    extern "C" {
        fn orange_chacha20_block(
            key: *const u8,
            nonce: *const u8,
            counter: u32,
            out: *mut u8,
        );
    }

    #[inline]
    pub fn block(
        key: &[u8; 32],
        nonce: &[u8; 12],
        counter: u32,
        out: &mut [u8; 64],
    ) {
        // Compile-time array length enforcement guarantees precondition satisfaction
        unsafe {
            orange_chacha20_block(
                key.as_ptr(),
                nonce.as_ptr(),
                counter,
                out.as_mut_ptr(),
            );
        }
    }
}
```


---

## Part XV: Complete Diagnostic Reference Catalog

### §93. Diagnostic Philosophy, Severity Structure, and Error Budgets

1. **Permanence of Diagnostic Identifiers:**
   Diagnostic codes in Orange are immutable elements of the compiler's user-facing
   interface. Existing numeric identifiers MUST NOT be repurposed, redefined, or
   reordered across releases.
2. **Severity Hierarchy:**
   - The severity of all compile-time diagnostic failures is fixed as `error`.
   - Any emitted error immediately invalidates downstream compiler phases (e.g. an
     error during parsing prevents semantic analysis; an error during semantic analysis
     prevents evaluation or code generation).
3. **Structured Diagnostic Record:**
   Every emitted diagnostic is a record:
   $$\text{Diag} = \langle \text{Code}, \text{Severity}, \text{PrimarySpan}, \text{Label}, \text{SecondarySpans}, \text{Notes} \rangle$$
4. **Deterministic Error Suppression Budgets:**
   - The compiler reports at most **32 errors per compiler phase** before halting
     diagnostic accumulation.
   - Upon encountering the 33rd error in any phase, the compiler emits a phase-specific
     suppression diagnostic (`ORC0007` for lexing, `ORC0105` for parsing, `ORC0208`
     for semantic analysis) and halts further analysis.

---

### §94. Lexical Diagnostics (`ORC0001`–`ORC0009`): Formal Predicates, Triggers, Examples, Fixes

#### `ORC0001` — `UnexpectedCharacter`
- **Subsystem:** Lexical Analyzer
- **Formal Trigger Predicate:**
  $$\text{Trigger}(c) \iff c \notin \text{Alphabet}(\text{Edition 2026}) \lor c \in \text{ForbiddenControls} \lor c = \text{BOM}$$
- **Rationale:** Prevents Trojan Source attacks, encoding ambiguities, and accidental
  syntax corruption from foreign codepoints.
- **Erroneous Example:**
  ```orange
  spec hash(m: Word[8]^32) -> Word[32] {
      let @temp = 10; // '@' has no lexical meaning
  }
  ```
- **Remediation:** Remove the character or replace it with valid ASCII tokens:
  ```orange
  spec hash(m: Word[8]^32) -> Word[32] {
      let temp: Word[32] = 10;
  }
  ```

#### `ORC0002` — `UnterminatedBlockComment`
- **Subsystem:** Lexical Analyzer
- **Formal Trigger Predicate:**
  $$\text{Trigger} \iff \text{EOF reached} \land \text{CommentNestingDepth} > 0$$
- **Rationale:** Eliminates silent swallowing of source code caused by unclosed comments.
- **Erroneous Example:**
  ```orange
  /* Outer comment
     /* Nested comment */
  spec main() -> Int { 42 }
  ```
- **Remediation:** Append `*/` to close all open comment nesting levels.

#### `ORC0003` — `UnterminatedString`
- **Subsystem:** Lexical Analyzer
- **Formal Trigger Predicate:**
  $$\text{Trigger} \iff \text{Scanned}(\texttt{"}) \land (\text{NextChar} = \texttt{\textbackslash n} \lor \text{NextChar} = \text{EOF})$$
- **Rationale:** Prevents multi-line string confusion and enforces single-line byte string literals.
- **Erroneous Example:**
  ```orange
  test "Unclosed test vector title {
      true
  }
  ```
- **Remediation:** Close the string literal with `"` on the same logical line.

#### `ORC0004` — `InvalidEscape`
- **Subsystem:** Lexical Analyzer
- **Formal Trigger Predicate:**
  $$\text{Trigger}(\texttt{\textbackslash}c) \iff c \notin \{\texttt{"}, \texttt{\textbackslash}, \texttt{n}, \texttt{r}, \texttt{t}, \texttt{0}, \texttt{x}\} \lor (c = \texttt{x} \land \neg \text{IsHexDigitPair}(\text{Next}_2))$$
- **Rationale:** Ensures byte string literals have deterministic, unambiguous binary decodings.
- **Erroneous Example:**
  ```orange
  let s = "invalid \u0041 escape"; // Unicode escapes unsupported
  let bad_hex = "\x1z";
  ```
- **Remediation:** Use only supported escapes (`\"`, `\\`, `\n`, `\r`, `\t`, `\0`, `\xNN`):
  ```orange
  let s = "valid \x41 escape";
  let good_hex = "\x1a";
  ```

#### `ORC0005` — `MalformedInteger`
- **Subsystem:** Lexical Analyzer
- **Formal Trigger Predicate:**
  $$\text{Trigger}(tok) \iff \text{InvalidDigitsForBase}(tok) \lor \text{MisplacedUnderscore}(tok) \lor \text{MissingDigitsAfterPrefix}(tok)$$
- **Rationale:** Enforces strict lexical boundaries for numeric literals, eliminating silent base misinterpretations.
- **Erroneous Example:**
  ```orange
  let a = 0x_10;    // Underscore immediately after prefix
  let b = 100_;     // Trailing underscore
  let c = 0b102;    // '2' invalid for base 2
  ```
- **Remediation:** Place underscores only between valid digits:
  ```orange
  let a = 0x10;
  let b = 100;
  let c = 0b101;
  ```

#### `ORC0006` — `LexicalTokenLimit`
- **Subsystem:** Lexical Analyzer
- **Formal Trigger Predicate:**
  $$\text{Trigger} \iff \text{Count}(\text{NonTriviaTokens}) > 1,048,576$$
- **Rationale:** Protects the compiler against algorithmic complexity attacks and out-of-memory crashes.
- **Remediation:** Partition large compilation units into multiple modular source files.

#### `ORC0007` — `TooManyLexicalErrors`
- **Subsystem:** Lexical Analyzer
- **Formal Trigger Predicate:**
  $$\text{Trigger} \iff \text{Count}(\text{LexicalErrors}) \ge 33$$
- **Remediation:** Fix early lexical errors and re-run compilation.

#### `ORC0008` — `LexicalResourceLimit`
- **Subsystem:** Lexical Analyzer
- **Formal Trigger Predicate:**
  $$\text{Trigger} \iff |\text{SourceFile}| > 16,777,216 \text{ bytes} \lor \text{MemoryAllocationFailed}()$$
- **Remediation:** Keep source files under 16 MiB.

#### `ORC0009` — `MalformedHexString`
- **Subsystem:** Lexical Analyzer
- **Formal Trigger Predicate:**
  $$\text{Trigger}(tok) \iff \text{Prefix}(tok) = \texttt{hex"} \land (\text{OddDigitCount}(tok) \lor \text{ContainsInvalidHexChars}(tok))$$
- **Rationale:** Byte sequences require exact 8-bit alignment.
- **Erroneous Example:**
  ```orange
  let key = hex"012";     // Odd number of nibbles (3 nibbles)
  let iv  = hex"01 02 gg"; // 'g' is not a hexadecimal digit
  ```
- **Remediation:** Provide exact byte pairs separated by optional single spaces:
  ```orange
  let key = hex"01 20";
  let iv  = hex"01 02 ff";
  ```

---

### §95. Syntactic Diagnostics (`ORC0101`–`ORC0108`): Formal Predicates, Triggers, Examples, Fixes

#### `ORC0101` — `ExpectedSyntax`
- **Subsystem:** Parser
- **Formal Trigger Predicate:**
  $$\text{Trigger} \iff \text{CurrentToken} \ne \text{ExpectedProductionToken}$$
- **Erroneous Example:**
  ```orange
  edition 2026 // Missing semicolon
  module sha256 { }
  ```
- **Remediation:** Supply the required syntactic delimiter: `edition 2026;`.

#### `ORC0102` — `UnsupportedSourceEdition`
- **Subsystem:** Parser
- **Formal Trigger Predicate:**
  $$\text{Trigger}(E) \iff E \ne 2026$$
- **Erroneous Example:**
  ```orange
  edition 2025;
  ```
- **Remediation:** Specify the supported language edition: `edition 2026;`.

#### `ORC0103` — `ExpectedFunctionDeclaration`
- **Subsystem:** Parser
- **Formal Trigger Predicate:**
  $$\text{Trigger}(kw) \iff kw \notin \{\texttt{spec}, \texttt{impl}, \texttt{use}, \texttt{type}, \texttt{game}, \texttt{proof}, \texttt{claim}, \texttt{test}\}$$
- **Remediation:** Begin module declarations with a valid top-level member keyword.

#### `ORC0104` — `TrailingSyntax`
- **Subsystem:** Parser
- **Formal Trigger Predicate:**
  $$\text{Trigger} \iff \text{CurrentToken} \ne \text{EOF} \text{ after } \text{ModuleClosingBrace}$$
- **Erroneous Example:**
  ```orange
  edition 2026;
  module a { }
  module b { } // Second module in same file
  ```
- **Remediation:** Place each module in its own dedicated source file.

#### `ORC0105` — `TooManySyntaxErrors`
- **Subsystem:** Parser
- **Formal Trigger Predicate:**
  $$\text{Trigger} \iff \text{Count}(\text{SyntaxErrors}) \ge 33$$
- **Remediation:** Resolve initial syntactic failures.

#### `ORC0106` — `ParserResourceLimit`
- **Subsystem:** Parser
- **Formal Trigger Predicate:**
  $$\text{Trigger} \iff \text{RecursionDepth} > 1,024 \lor \text{ASTNodeCount} > 1,048,576$$
- **Remediation:** Simplify deeply nested parenthesized expressions.

#### `ORC0107` — `InvalidParserInput`
- **Subsystem:** Parser
- **Formal Trigger Predicate:**
  $$\text{Trigger} \iff \text{TokenSourceId} \ne \text{ParserSourceId}$$
- **Remediation:** Internal compiler bug; report issue.

#### `ORC0108` — `UngroupedOperators`
- **Subsystem:** Parser
- **Formal Trigger Predicate:**
  $$\text{Trigger}(op_1, op_2) \iff \text{Group}(op_1) \ne \text{Group}(op_2) \land \neg \text{Parenthesized}(op_1, op_2)$$
- **Rationale:** Eliminates cryptographic vulnerabilities arising from unexpected operator precedence.
- **Erroneous Example:**
  ```orange
  spec f(a: Word[32], b: Word[32], c: Word[32]) -> Word[32] {
      a & b ^ c
  }
  spec g(x: Word[32]) -> Word[32] {
      x << 2 >> 1
  }
  ```
- **Remediation:** Enclose distinct operator groups in explicit parentheses:
  ```orange
  spec f(a: Word[32], b: Word[32], c: Word[32]) -> Word[32] {
      (a & b) ^ c
  }
  spec g(x: Word[32]) -> Word[32] {
      (x << 2) >> 1
  }
  ```

---

### §96. Semantic & Type Diagnostics (`ORC0201`–`ORC0242`): Formal Predicates, Triggers, Examples, Fixes

#### `ORC0201` — `DuplicateFunction`
- **Subsystem:** Semantic Analyzer (Symbol Table Resolver)
- **Formal Trigger Predicate:**
  $$\text{Trigger}(f, M) \iff \exists g \in \text{Declarations}(M).\ f \ne g \land \text{Stratum}(f) = \text{Stratum}(g) \land \text{Ident}(f) = \text{Ident}(g)$$
- **Theoretical Rationale:** Function overloading within a single semantic stratum introduces
  call-graph ambiguities and complicates automated theorem proving. Every symbol in a stratum
  must have a unique canonical denotation.
- **Erroneous Example:**
  ```orange
  edition 2026;
  module crypto {
      spec process(x: Word[32]) -> Word[32] { x + 1 }
      spec process(x: Word[64]) -> Word[64] { x + 1 } // Duplicate identifier
  }
  ```
- **Compiler Diagnostic Rendering:**
  ```text
  error[ORC0201]: duplicate function definition 'process'
   --> crypto.or:4:10
    |
  3 |     spec process(x: Word[32]) -> Word[32] { x + 1 }
    |          ------- previous definition of 'process' here
  4 |     spec process(x: Word[64]) -> Word[64] { x + 1 }
    |          ^^^^^^^ duplicate definition in module 'crypto'
  ```
- **Remediation:** Disambiguate by assigning distinct semantic function identifiers:
  ```orange
  edition 2026;
  module crypto {
      spec process32(x: Word[32]) -> Word[32] { x + 1 }
      spec process64(x: Word[64]) -> Word[64] { x + 1 }
  }
  ```

#### `ORC0202` — `UnsupportedTypedFunction`
- **Subsystem:** Semantic Analyzer (Slice Capability Gate)
- **Formal Trigger Predicate:**
  $$\text{Trigger}(f, \mathcal{S}) \iff \text{Stratum}(f) \ne \texttt{spec} \land \text{HasTypedBody}(f) \land \text{ActiveSlice}(\mathcal{S}) \prec \text{SliceRequired}(f)$$
- **Theoretical Rationale:** Language slices (S1 through S3t) gate feature stabilization.
  Declaring typed imperative `impl` or `machine impl` procedure bodies when targeting pure
  specification slices prevents undefined evaluation behavior.
- **Erroneous Example:**
  ```orange
  edition 2026;
  module test_impl {
      impl compute(x: Word[32]) -> Word[32] { x + 1 }
  }
  ```
- **Compiler Diagnostic Rendering:**
  ```text
  error[ORC0202]: unsupported typed function in active slice
   --> test_impl.or:3:5
    |
  3 |     impl compute(x: Word[32]) -> Word[32] { x + 1 }
    |     ^^^^ 'impl' procedures require slice S4 or higher; active slice is S3t
  ```
- **Remediation:** Use `spec` for pure mathematical specifications in slice S3t:
  ```orange
  edition 2026;
  module test_impl {
      spec compute(x: Word[32]) -> Word[32] { x + 1 }
  }
  ```

#### `ORC0203` — `UnsupportedType`
- **Subsystem:** Semantic Analyzer (Type Elaborator)
- **Formal Trigger Predicate:**
  $$\text{Trigger}(\tau) \iff \tau \notin \text{Universe}(\text{Orange 2026}) = \{\text{Int}, \text{Bool}, \text{Byte}\} \cup \{\text{Word}[W] \mid W \in \{8,16,32,64\}\} \cup \{\text{Mod}[m]\} \cup \{T^n\} \cup \{(T_0, \dots, T_{k-1})\}$$
- **Theoretical Rationale:** IEEE-754 floating-point numbers, unbounded dynamic pointers,
  and recursive algebraic data types introduce nondeterministic rounding, platform divergence,
  and side-channel leakages. They are strictly excluded from Orange's type universe.
- **Erroneous Example:**
  ```orange
  edition 2026;
  module bad_type {
      spec float_op(x: Float64) -> Float64 { x }
  }
  ```
- **Compiler Diagnostic Rendering:**
  ```text
  error[ORC0203]: unsupported type 'Float64'
   --> bad_type.or:3:22
    |
  3 |     spec float_op(x: Float64) -> Float64 { x }
    |                      ^^^^^^^ type 'Float64' is not part of Orange 2026
  ```
- **Remediation:** Model computations using fixed-width words, modular rings, or exact integers:
  ```orange
  edition 2026;
  module bad_type {
      spec word_op(x: Word[64]) -> Word[64] { x }
  }
  ```

#### `ORC0204` — `UnsupportedWordWidth`
- **Subsystem:** Semantic Analyzer (Type Elaborator)
- **Formal Trigger Predicate:**
  $$\text{Trigger}(W) \iff W \notin \{8, 16, 32, 64\}$$
- **Theoretical Rationale:** Word arithmetic is grounded in physical machine register rings
  $\mathbb{Z}/2^W\mathbb{Z}$. Arbitrary-width words (e.g. 24-bit or 48-bit) cannot execute
  with constant-time hardware guarantees without non-standard masking.
- **Erroneous Example:**
  ```orange
  edition 2026;
  module word_width {
      spec bad_word(x: Word[24]) -> Word[24] { x }
  }
  ```
- **Compiler Diagnostic Rendering:**
  ```text
  error[ORC0204]: unsupported word width '24'
   --> word_width.or:3:24
    |
  3 |     spec bad_word(x: Word[24]) -> Word[24] { x }
    |                        ^^ word width must be exactly 8, 16, 32, or 64
  ```
- **Remediation:** Use standard machine word widths ($8, 16, 32, 64$):
  ```orange
  edition 2026;
  module word_width {
      spec good_word(x: Word[32]) -> Word[32] { x }
  }
  ```

#### `ORC0205` — `IntegerMagnitudeLimit`
- **Subsystem:** Semantic Analyzer (Constant Evaluator)
- **Formal Trigger Predicate:**
  $$\text{Trigger}(x) \iff x \in \mathbb{Z} \land |x| \ge 2^{4096}$$
- **Theoretical Rationale:** Prevents denial-of-service and algorithmic complexity attacks
  on the compiler's big-integer arithmetic engine. $4,096$ bits is sufficient to model
  arbitrary RSA/ECC parameters (including Curve448 and E-521) while maintaining bounded memory.
- **Erroneous Example:**
  ```orange
  edition 2026;
  module limits {
      spec huge() -> Int { 1 << 4096 }
  }
  ```
- **Compiler Diagnostic Rendering:**
  ```text
  error[ORC0205]: integer magnitude limit exceeded
   --> limits.or:3:28
    |
  3 |     spec huge() -> Int { 1 << 4096 }
    |                            ^^^^^^^ evaluated magnitude >= 2^4096
  ```
- **Remediation:** Keep integer operands within the 4,096-bit representation budget.

#### `ORC0206` — `NegativeWordLiteral`
- **Subsystem:** Semantic Analyzer (Type Checker)
- **Formal Trigger Predicate:**
  $$\text{Trigger}(lit, \tau) \iff \tau = \text{Word}[W] \land lit < 0$$
- **Theoretical Rationale:** Word elements belong to $\mathbb{Z}/2^W\mathbb{Z}$ and are
  canonically represented by residues $r \in [0, 2^W - 1]$. Negative literal syntax obscures
  ring semantics; negation must be expressed via explicit arithmetic subtraction.
- **Erroneous Example:**
  ```orange
  edition 2026;
  module neg_word {
      spec mask() -> Word[8] { -1 }
  }
  ```
- **Compiler Diagnostic Rendering:**
  ```text
  error[ORC0206]: negative word literal '-1'
   --> neg_word.or:3:30
    |
  3 |     spec mask() -> Word[8] { -1 }
    |                              ^^ word literals must be non-negative in [0, 2^W - 1]
  ```
- **Remediation:** Use canonical unsigned hex literals or ring subtraction:
  ```orange
  edition 2026;
  module neg_word {
      spec mask() -> Word[8] { 0xff }
  }
  ```

#### `ORC0207` — `WordLiteralOutOfRange`
- **Subsystem:** Semantic Analyzer (Type Checker)
- **Formal Trigger Predicate:**
  $$\text{Trigger}(lit, \tau) \iff (\tau = \text{Word}[W] \land lit \ge 2^W) \lor (\tau = \text{Mod}[m] \land lit \ge m)$$
- **Theoretical Rationale:** Prevents silent truncation bugs where developers accidentally
  specify a literal exceeding the ring or residue modulus.
- **Erroneous Example:**
  ```orange
  edition 2026;
  module out_of_range {
      spec b() -> Word[8] { 256 }
      spec m() -> Mod[17] { 17 }
  }
  ```
- **Compiler Diagnostic Rendering:**
  ```text
  error[ORC0207]: word literal '256' out of range for Word[8]
   --> out_of_range.or:3:27
    |
  3 |     spec b() -> Word[8] { 256 }
    |                           ^^^ maximum value for Word[8] is 255
  ```
- **Remediation:** Specify canonical residues in $[0, 2^W - 1]$ or $[0, m - 1]$:
  ```orange
  edition 2026;
  module out_of_range {
      spec b() -> Word[8] { 255 }
      spec m() -> Mod[17] { 16 }
  }
  ```

#### `ORC0208` — `TooManySemanticErrors`
- **Subsystem:** Semantic Analyzer (Error Recovery Driver)
- **Formal Trigger Predicate:**
  $$\text{Trigger} \iff \text{Count}(\text{SemanticErrors}) \ge 33$$
- **Theoretical Rationale:** Bounds diagnostic output to prevent terminal flooding and
  cascading nonsensical error reports.
- **Remediation:** Resolve initial semantic typing failures and re-run `orangec check`.

#### `ORC0209` — `SemanticResourceLimit`
- **Subsystem:** Semantic Analyzer (Memory Monitor)
- **Formal Trigger Predicate:**
  $$\text{Trigger} \iff \text{TypeInferenceDepth} > 1,024 \lor \text{TypeTableAllocBytes} > 67,108,864$$
- **Theoretical Rationale:** Protects the compiler against pathological type structures
  and cyclic elaboration attacks.
- **Remediation:** Simplify deeply nested tuple expressions or split huge modules.

#### `ORC0210` — `InvalidSemanticInput`
- **Subsystem:** Semantic Analyzer (AST Validator)
- **Formal Trigger Predicate:**
  $$\text{Trigger} \iff \text{ASTSourceFileId} \ne \text{ContextSourceFileId}$$
- **Theoretical Rationale:** Internal assertion failure verifying AST integrity between
  parser output and type checker input.
- **Remediation:** Internal compiler consistency defect; recompile or report issue.

#### `ORC0211` — `UnknownParameter`
- **Subsystem:** Semantic Analyzer (Scope Resolution)
- **Formal Trigger Predicate:**
  $$\text{Trigger}(x, \Gamma, \Sigma) \iff x \notin \text{dom}(\Gamma) \land x \notin \text{dom}(\Sigma)$$
- **Theoretical Rationale:** Orange requires explicit lexical declarations for all identifiers.
  Ambient global variables do not exist.
- **Erroneous Example:**
  ```orange
  edition 2026;
  module scoping {
      spec add_one(x: Word[32]) -> Word[32] { x + y }
  }
  ```
- **Compiler Diagnostic Rendering:**
  ```text
  error[ORC0211]: cannot find value 'y' in this scope
   --> scoping.or:3:49
    |
  3 |     spec add_one(x: Word[32]) -> Word[32] { x + y }
    |                                                 ^ not found in this scope
  ```
- **Remediation:** Bind `y` as a function parameter or in a local `let` binding:
  ```orange
  edition 2026;
  module scoping {
      spec add_one(x: Word[32], y: Word[32]) -> Word[32] { x + y }
  }
  ```

#### `ORC0212` — `UnknownFunction`
- **Subsystem:** Semantic Analyzer (Call Graph Resolver)
- **Formal Trigger Predicate:**
  $$\text{Trigger}(f, \Sigma) \iff f \notin \text{dom}(\Sigma)$$
- **Theoretical Rationale:** Function invocations must resolve to statically known,
  typed signatures in the current module or imported dependencies.
- **Erroneous Example:**
  ```orange
  edition 2026;
  module calls {
      spec test_call() -> Word[32] { helper(42) }
  }
  ```
- **Compiler Diagnostic Rendering:**
  ```text
  error[ORC0212]: cannot find function 'helper' in module 'calls'
   --> calls.or:3:36
    |
  3 |     spec test_call() -> Word[32] { helper(42) }
    |                                    ^^^^^^ function 'helper' is not defined
  ```
- **Remediation:** Define `spec helper(...)` or import its declaring module using `use`.

#### `ORC0213` — `ArgumentCountMismatch`
- **Subsystem:** Semantic Analyzer (Call Site Type Checker)
- **Formal Trigger Predicate:**
  $$\text{Trigger}(f, \text{args}) \iff |\text{args}| \ne \text{ParamCount}(\Sigma(f))$$
- **Theoretical Rationale:** Functions have fixed arity; variadic arguments and default
  parameters are excluded to ensure deterministic call sequences.
- **Erroneous Example:**
  ```orange
  edition 2026;
  module arity {
      spec add(a: Int, b: Int) -> Int { a + b }
      spec invoke() -> Int { add(1) }
  }
  ```
- **Compiler Diagnostic Rendering:**
  ```text
  error[ORC0213]: this function takes 2 arguments but 1 was supplied
   --> arity.or:4:28
    |
  3 |     spec add(a: Int, b: Int) -> Int { a + b }
    |          --- defined here with 2 parameters
  4 |     spec invoke() -> Int { add(1) }
    |                            ^^^ expected 2 arguments, found 1
  ```
- **Remediation:** Supply all required arguments: `add(1, 2)`.

#### `ORC0214` — `TypeMismatch`
- **Subsystem:** Semantic Analyzer (Unification Engine)
- **Formal Trigger Predicate:**
  $$\text{Trigger}(\tau_{\text{actual}}, \tau_{\text{expected}}) \iff \tau_{\text{actual}} \not\equiv \tau_{\text{expected}}$$
- **Theoretical Rationale:** Orange forbids implicit type coercions and subtyping widening.
  All type mismatches indicate conceptual or structural bugs.
- **Erroneous Example:**
  ```orange
  edition 2026;
  module mismatch {
      spec assign() -> Word[32] {
          let x: Word[32] = 42 as Word[8];
          x
      }
  }
  ```
- **Compiler Diagnostic Rendering:**
  ```text
  error[ORC0214]: mismatched types: expected 'Word[32]', found 'Word[8]'
   --> mismatch.or:4:27
    |
  4 |         let x: Word[32] = 42 as Word[8];
    |                --------   ^^^^^^^^^^^^^ expected 'Word[32]', found 'Word[8]'
    |                |
    |                expected due to this type annotation
  ```
- **Remediation:** Use explicit conversion to the expected type: `42 as Word[32]`.

#### `ORC0215` — `UnsupportedOperator`
- **Subsystem:** Semantic Analyzer (Operator Resolution)
- **Formal Trigger Predicate:**
  $$\text{Trigger}(op, \tau) \iff op \notin \text{AdmittedOperators}(\tau)$$
- **Theoretical Rationale:** Prevents applying bitwise operations to non-ring types
  (such as `Int` or `Bool`) or applying arithmetic operators to tuples.
- **Erroneous Example:**
  ```orange
  edition 2026;
  module bad_op {
      spec int_xor(a: Int, b: Int) -> Int { a ^ b }
  }
  ```
- **Compiler Diagnostic Rendering:**
  ```text
  error[ORC0215]: binary operator '^' cannot be applied to type 'Int'
   --> bad_op.or:3:45
    |
  3 |     spec int_xor(a: Int, b: Int) -> Int { a ^ b }
    |                                           ^ bitwise operators require 'Word[W]'
  ```
- **Remediation:** Convert integer operands to fixed-width words before bitwise operations:
  ```orange
  edition 2026;
  module bad_op {
      spec word_xor(a: Word[32], b: Word[32]) -> Word[32] { a ^ b }
  }
  ```

#### `ORC0216` — `InvalidShiftAmount`
- **Subsystem:** Semantic Analyzer (Shift Verification)
- **Formal Trigger Predicate:**
  $$\text{Trigger}(k, W) \iff k \in \text{Literals} \land (k < 0 \lor k \ge W)$$
- **Theoretical Rationale:** Literal shifts of $k \ge W$ or $k < 0$ produce undefined
  or non-portable results across target architectures (e.g. x86 masks shift counts by 31/63
  while ARM masks by 255).
- **Erroneous Example:**
  ```orange
  edition 2026;
  module bad_shift {
      spec rot(x: Word[32]) -> Word[32] { x <<< 32 }
  }
  ```
- **Compiler Diagnostic Rendering:**
  ```text
  error[ORC0216]: shift amount '32' is out of range for 'Word[32]'
   --> bad_shift.or:3:47
    |
  3 |     spec rot(x: Word[32]) -> Word[32] { x <<< 32 }
    |                                               ^^ shift amount must be in 0..31
  ```
- **Remediation:** Specify a literal shift amount $0 \le k < W$:
  ```orange
  edition 2026;
  module bad_shift {
      spec rot(x: Word[32]) -> Word[32] { x <<< 16 }
  }
  ```

#### `ORC0217` — `CallCycle`
- **Subsystem:** Semantic Analyzer (Call Graph DAG Validator)
- **Formal Trigger Predicate:**
  $$\text{Trigger}(G_{\text{call}}) \iff \exists f.\ f \to^+ f \text{ in } G_{\text{call}}$$
- **Theoretical Rationale:** Strongly normalizes the specification stratum. Banning recursive
  cycles guarantees that all specification functions terminate in bounded time.
- **Erroneous Example:**
  ```orange
  edition 2026;
  module cycle {
      spec f(x: Int) -> Int { g(x) }
      spec g(x: Int) -> Int { f(x) }
  }
  ```
- **Compiler Diagnostic Rendering:**
  ```text
  error[ORC0217]: cycle detected in call graph: 'f' -> 'g' -> 'f'
   --> cycle.or:3:5
    |
  3 |     spec f(x: Int) -> Int { g(x) }
    |     ^^^^ recursive function call cycle is prohibited in Orange
  ```
- **Remediation:** Eliminate recursion; formulate algorithms using bounded `for` loops.

#### `ORC0218` — `DuplicateParameter`
- **Subsystem:** Semantic Analyzer (Parameter Binder)
- **Formal Trigger Predicate:**
  $$\text{Trigger}(f) \iff \exists i \ne j.\ \text{ParamName}_i(f) = \text{ParamName}_j(f)$$
- **Theoretical Rationale:** Prevents lexical shadowing within the procedure's parameter
  environment.
- **Erroneous Example:**
  ```orange
  edition 2026;
  module dup_param {
      spec f(x: Word[32], x: Word[32]) -> Word[32] { x }
  }
  ```
- **Compiler Diagnostic Rendering:**
  ```text
  error[ORC0218]: duplicate parameter name 'x'
   --> dup_param.or:3:25
    |
  3 |     spec f(x: Word[32], x: Word[32]) -> Word[32] { x }
    |            -            ^ duplicate parameter 'x'
    |            |
    |            previously declared here
  ```
- **Remediation:** Provide unique identifiers for each parameter: `spec f(a: Word[32], b: Word[32])`.

#### `ORC0219` — `DuplicateBinding`
- **Subsystem:** Semantic Analyzer (Scope Resolution)
- **Formal Trigger Predicate:**
  $$\text{Trigger}(x, \Gamma) \iff x \in \text{dom}(\Gamma)$$
- **Theoretical Rationale:** Variable shadowing in cryptographic kernels routinely conceals
  accidental overwrite bugs (e.g. shadowing an accumulator variable inside a loop).
- **Erroneous Example:**
  ```orange
  edition 2026;
  module shadow {
      spec f(x: Word[32]) -> Word[32] {
          let x: Word[32] = 10;
          x
      }
  }
  ```
- **Compiler Diagnostic Rendering:**
  ```text
  error[ORC0219]: duplicate binding 'x' shadows existing variable
   --> shadow.or:4:13
    |
  3 |     spec f(x: Word[32]) -> Word[32] {
    |            - first binding of 'x' declared here
  4 |         let x: Word[32] = 10;
    |             ^ re-declaration of 'x' in the same scope is prohibited
  ```
- **Remediation:** Assign a distinct variable name: `let x1: Word[32] = 10;`.

#### `ORC0220` — `UntypedConversionOperand`
- **Subsystem:** Semantic Analyzer (Cast Checker)
- **Formal Trigger Predicate:**
  $$\text{Trigger}(e) \iff e \text{ as } \tau \land e \text{ is an untyped literal}$$
- **Theoretical Rationale:** Untyped integer literals must be bound to a known type before
  undergoing conversion, avoiding ambiguous conversion paths.
- **Erroneous Example:**
  ```orange
  edition 2026;
  module untyped_cast {
      spec bad() -> Word[32] { (42) as Word[32] }
  }
  ```
- **Compiler Diagnostic Rendering:**
  ```text
  error[ORC0220]: untyped conversion operand: cannot cast untyped literal with 'as'
   --> untyped_cast.or:3:30
    |
  3 |     spec bad() -> Word[32] { (42) as Word[32] }
    |                              ^^^^ cannot cast untyped literal; bind to a typed variable first
  ```
- **Remediation:** Bind to a typed local variable or use typed literal syntax:
  ```orange
  edition 2026;
  module untyped_cast {
      spec good() -> Word[32] {
          let x: Word[32] = 42;
          x
      }
  }
  ```

#### `ORC0221` — `UnsupportedArrayLength`
- **Subsystem:** Semantic Analyzer (Array Type Elaborator)
- **Formal Trigger Predicate:**
  $$\text{Trigger}(n) \iff n < 1 \lor n > 65,536$$
- **Theoretical Rationale:** Zero-length arrays introduce degenerate algebraic properties
  and indexing ambiguities. Upper bounding arrays at $65,536$ elements ensures that stack
  allocation bounds $\text{StackFrameSize} \le 64\text{ KiB}$ are preserved by construction.
- **Erroneous Example:**
  ```orange
  edition 2026;
  module bad_array {
      type Empty = Word[8]^0;      // Zero length is prohibited
      type Massive = Word[32]^70000; // Exceeds 65,536 ceiling
  }
  ```
- **Compiler Diagnostic Rendering:**
  ```text
  error[ORC0221]: unsupported array length: '0'
   --> bad_array.or:3:26
    |
  3 |     type Empty = Word[8]^0;
    |                          ^ array length must be between 1 and 65,536
  ```
- **Remediation:** Declare array dimensions within the closed interval $[1, 65536]$:
  ```orange
  edition 2026;
  module bad_array {
      type Valid = Word[8]^64;
  }
  ```

#### `ORC0222` — `ArrayLengthMismatch`
- **Subsystem:** Semantic Analyzer (Literal Type Checker)
- **Formal Trigger Predicate:**
  $$\text{Trigger}(lit, n) \iff lit = [e_0, \dots, e_{k-1}] \land k \ne n$$
- **Theoretical Rationale:** Array literals are statically sized. Initializer elements must
  correspond bijectively to declared vector slots to prevent uninitialized memory slots.
- **Erroneous Example:**
  ```orange
  edition 2026;
  module arr_len {
      spec get_iv() -> Word[32]^4 {
          [0x10, 0x20, 0x30] // Only 3 elements supplied for length 4
      }
  }
  ```
- **Compiler Diagnostic Rendering:**
  ```text
  error[ORC0222]: array literal element count mismatch: expected 4, found 3
   --> arr_len.or:4:9
    |
  4 |         [0x10, 0x20, 0x30]
    |         ^^^^^^^^^^^^^^^^^^ expected 4 elements for type 'Word[32]^4', found 3
  ```
- **Remediation:** Provide exactly $n$ elements, or use the fill expression `[val; n]`:
  ```orange
  edition 2026;
  module arr_len {
      spec get_iv() -> Word[32]^4 {
          [0x10, 0x20, 0x30, 0x40]
      }
  }
  ```

#### `ORC0223` — `IndexOutOfRange`
- **Subsystem:** Semantic Analyzer (Static Interval Bounds Checker)
- **Formal Trigger Predicate:**
  $$\text{Trigger}(i, n) \iff \mathcal{I}(i) = [l, u] \land (l < 0 \lor u \ge n)$$
- **Theoretical Rationale:** Memory safety is mathematically verified at compile time.
  Any index whose evaluated interval cannot be proved strictly within $[0, n-1]$ is rejected.
- **Erroneous Example:**
  ```orange
  edition 2026;
  module bounds {
      spec pick(arr: Word[8]^16) -> Word[8] {
          arr[16] // Index 16 out of bounds for array of length 16
      }
  }
  ```
- **Compiler Diagnostic Rendering:**
  ```text
  error[ORC0223]: array index out of bounds: index 16 is out of range for length 16
   --> bounds.or:4:13
    |
  4 |         arr[16]
    |             ^^ index must be strictly less than 16 (valid range: 0..15)
  ```
- **Remediation:** Use indices within $[0, n-1]$:
  ```orange
  edition 2026;
  module bounds {
      spec pick(arr: Word[8]^16) -> Word[8] {
          arr[15]
      }
  }
  ```

#### `ORC0224` — `NotAnArray`
- **Subsystem:** Semantic Analyzer (Type Checker)
- **Formal Trigger Predicate:**
  $$\text{Trigger}(X) \iff \text{Type}(X) \not\equiv T^n$$
- **Theoretical Rationale:** Bracket indexing `X[i]` is defined exclusively over fixed-length
  Cartesian power array types. Applying brackets to scalar words or tuples is prohibited.
- **Erroneous Example:**
  ```orange
  edition 2026;
  module not_arr {
      spec index_word(w: Word[32]) -> Word[8] {
          w[0] // Cannot index directly into scalar Word[32]
      }
  }
  ```
- **Compiler Diagnostic Rendering:**
  ```text
  error[ORC0224]: cannot index into value of type 'Word[32]'
   --> not_arr.or:4:9
    |
  4 |         w[0]
    |         ^ indexing syntax '[...]' can only be applied to array types 'T^n'
  ```
- **Remediation:** Convert word to a byte array before indexing:
  ```orange
  edition 2026;
  module not_arr {
      spec index_word(w: Word[32]) -> Word[8] {
          let bytes: Word[8]^4 = w as big Word[8]^4;
          bytes[0]
      }
  }
  ```

#### `ORC0225` — `InvalidLoopRange`
- **Subsystem:** Semantic Analyzer (Loop Verification)
- **Formal Trigger Predicate:**
  $$\text{Trigger}(\text{low}, \text{high}) \iff \text{low} < 0 \lor \text{low} > \text{high} \lor \text{high} > 65,536$$
- **Theoretical Rationale:** Loop bounds in `spec` must be compile-time verifiable, finite,
  non-negative, and monotonically non-decreasing to guarantee bounded normalization.
- **Erroneous Example:**
  ```orange
  edition 2026;
  module bad_loop {
      spec rev() -> Word[32] {
          for i in 10..5 with acc = 0 { acc + i } // Decreasing range prohibited
      }
  }
  ```
- **Compiler Diagnostic Rendering:**
  ```text
  error[ORC0225]: invalid loop range: start '10' is greater than end '5'
   --> bad_loop.or:4:18
    |
  4 |         for i in 10..5 with acc = 0 { acc + i }
    |                  ^^^^ loop bounds must satisfy 0 <= low <= high <= 65536
  ```
- **Remediation:** Ensure bounds satisfy $0 \le \text{low} \le \text{high} \le 65,536$:
  ```orange
  edition 2026;
  module bad_loop {
      spec rev() -> Word[32] {
          for i in 5..10 with acc = 0 { acc + i }
      }
  }
  ```

#### `ORC0226` — `NonStaticIndex`
- **Subsystem:** Semantic Analyzer (Slice S1/S2 Enforcer)
- **Formal Trigger Predicate:**
  $$\text{Trigger}(i, \mathcal{S}) \iff \text{ActiveSlice}(\mathcal{S}) \prec \text{S3g} \land \neg \text{IsCompileTimeConstant}(i)$$
- **Theoretical Rationale:** Data-dependent indexing is restricted to slices with formal
  narrowing (S3g+). In earlier slices, indices must be static compile-time constants.
- **Erroneous Example:**
  ```orange
  edition 2026;
  module static_idx {
      spec lookup(table: Word[8]^16, dynamic_i: Int) -> Word[8] {
          table[dynamic_i]
      }
  }
  ```
- **Compiler Diagnostic Rendering:**
  ```text
  error[ORC0226]: non-static array index in slice S2
   --> static_idx.or:4:15
    |
  4 |         table[dynamic_i]
    |               ^^^^^^^^^ array index must be a compile-time constant in active slice
  ```
- **Remediation:** In S3g+, use data-dependent lookups with typed word indices narrowed to bounds.

#### `ORC0227` — `UntypedComparison`
- **Subsystem:** Semantic Analyzer (Relational Type Checker)
- **Formal Trigger Predicate:**
  $$\text{Trigger}(a \mathbin{\text{cmp}} b) \iff \text{IsUntypedLit}(a) \land \text{IsUntypedLit}(b)$$
- **Theoretical Rationale:** Relational comparisons require ground types to establish whether
  comparison follows algebraic integer order ($\mathbb{Z}$), word ring order ($\mathbb{Z}/2^W\mathbb{Z}$),
  or modular residue order.
- **Erroneous Example:**
  ```orange
  edition 2026;
  module cmp_err {
      spec test_cmp() -> Bool { (10 == 20) } // Both operands untyped literals
  }
  ```
- **Compiler Diagnostic Rendering:**
  ```text
  error[ORC0227]: untyped comparison operands
   --> cmp_err.or:3:32
    |
  3 |     spec test_cmp() -> Bool { (10 == 20) }
    |                                ^^    ^^ cannot compare two untyped literals
  ```
- **Remediation:** Bind operands to typed variables before comparing:
  ```orange
  edition 2026;
  module cmp_err {
      spec test_cmp() -> Bool {
          let a: Int = 10;
          let b: Int = 20;
          a == b
      }
  }
  ```

#### `ORC0228` — `UnknownModule`
- **Subsystem:** Semantic Analyzer (Module Import Resolver)
- **Formal Trigger Predicate:**
  $$\text{Trigger}(m) \iff \neg \text{FileExists}(m \mathbin{\Vert} \text{".or"})$$
- **Theoretical Rationale:** Hermetic module resolution requires that every imported module
  corresponds to a deterministically discoverable source file on disk.
- **Erroneous Example:**
  ```orange
  edition 2026;
  module app {
      use missing_module; // File missing_module.or does not exist
  }
  ```
- **Compiler Diagnostic Rendering:**
  ```text
  error[ORC0228]: cannot find module 'missing_module'
   --> app.or:3:9
    |
  3 |     use missing_module;
    |         ^^^^^^^^^^^^^^ file 'missing_module.or' not found in package roots
  ```
- **Remediation:** Create `missing_module.or` or verify package search paths in `Orange.toml`.

#### `ORC0229` — `ModuleNotUsed`
- **Subsystem:** Semantic Analyzer (Qualified Name Resolver)
- **Formal Trigger Predicate:**
  $$\text{Trigger}(m, \Sigma) \iff m\text{::}f \in \text{References} \land \texttt{use } m \notin \text{Imports}$$
- **Theoretical Rationale:** Prevents ambient resolution of undeclared dependencies. All module
  dependencies must be explicitly imported at the top of the compilation unit.
- **Erroneous Example:**
  ```orange
  edition 2026;
  module client {
      spec call_external() -> Word[32] {
          sha256::hash_word(0) // 'use sha256;' was not declared
      }
  }
  ```
- **Compiler Diagnostic Rendering:**
  ```text
  error[ORC0229]: module 'sha256' was not declared with 'use'
   --> client.or:4:9
    |
  4 |         sha256::hash_word(0)
    |         ^^^^^^ module 'sha256' must be imported via 'use sha256;'
  ```
- **Remediation:** Add `use sha256;` to the module's import header.

#### `ORC0230` — `ModuleCycle`
- **Subsystem:** Semantic Analyzer (Tarjan SCC Module DAG Validator)
- **Formal Trigger Predicate:**
  $$\text{Trigger}(G_{\text{mod}}) \iff \exists m.\ m \to^+ m \text{ in } G_{\text{mod}}$$
- **Theoretical Rationale:** Cyclically dependent modules create mutually referential
  compilation environments and preclude deterministic, topological order compilation.
- **Erroneous Example:**
  ```text
  // File a.or
  edition 2026;
  module a { use b; }

  // File b.or
  edition 2026;
  module b { use a; }
  ```
- **Compiler Diagnostic Rendering:**
  ```text
  error[ORC0230]: cyclic dependency detected between modules: 'a' -> 'b' -> 'a'
   --> a.or:3:5
    |
  3 |     use b;
    |     ^^^^^ module import creates a circular dependency
  ```
- **Remediation:** Factor shared declarations into a leaf module `c.or` imported by both `a` and `b`.

#### `ORC0231` — `DuplicateModule`
- **Subsystem:** Semantic Analyzer (Module Header Resolver)
- **Formal Trigger Predicate:**
  $$\text{Trigger}(m) \iff \text{Count}(\text{Imports}(m)) > 1 \lor \text{DuplicateModuleSource}(m)$$
- **Theoretical Rationale:** Modules must have distinct names across a program, and a module
  cannot be imported redundantly within a single file.
- **Remediation:** Remove duplicate `use` statements or rename colliding modules.

#### `ORC0232` — `InvalidModulus`
- **Subsystem:** Semantic Analyzer (Residue Field Elaborator)
- **Formal Trigger Predicate:**
  $$\text{Trigger}(m) \iff m < 2 \lor m > 2^{521} - 1$$
- **Theoretical Rationale:** Moduli $< 2$ do not form residue rings or fields. The upper bound
  $2^{521} - 1$ admits NIST P-521 while maintaining bounded arithmetic complexity.
- **Erroneous Example:**
  ```orange
  edition 2026;
  module bad_mod {
      type Degenerate = Mod[1]; // Modulus 1 is algebraically trivial
  }
  ```
- **Compiler Diagnostic Rendering:**
  ```text
  error[ORC0232]: invalid modulus '1'
   --> bad_mod.or:3:27
    |
  3 |     type Degenerate = Mod[1];
    |                           ^ modulus must satisfy 2 <= m <= 2^521 - 1
  ```
- **Remediation:** Specify an admitted modulus $2 \le m \le 2^{521} - 1$.

#### `ORC0233` — `DuplicateTypeName`
- **Subsystem:** Semantic Analyzer (Type Alias Resolver)
- **Formal Trigger Predicate:**
  $$\text{Trigger}(T) \iff T \in \text{BuiltinTypes} \lor T \in \text{dom}(\text{DeclaredAliases})$$
- **Theoretical Rationale:** Prevents shadowing of builtin primitive types (`Int`, `Bool`, `Word`)
  and rejects re-declarations of type aliases in the same module.
- **Erroneous Example:**
  ```orange
  edition 2026;
  module dup_type {
      type Int = Word[64]; // Cannot redefine builtin type Int
  }
  ```
- **Compiler Diagnostic Rendering:**
  ```text
  error[ORC0233]: duplicate type name 'Int'
   --> dup_type.or:3:10
    |
  3 |     type Int = Word[64];
    |          ^^^ 'Int' is a reserved builtin type
  ```
- **Remediation:** Choose an unreserved, unique type alias name.

#### `ORC0234` — `NotATuple`
- **Subsystem:** Semantic Analyzer (Projection Type Checker)
- **Formal Trigger Predicate:**
  $$\text{Trigger}(t) \iff \text{Type}(t) \not\equiv (T_0, \dots, T_{k-1})$$
- **Theoretical Rationale:** Dot-index field projections (`t.0`, `t.1`) are valid exclusively
  on heterogeneous product tuple types.
- **Erroneous Example:**
  ```orange
  edition 2026;
  module tuple_err {
      spec proj(w: Word[32]) -> Word[32] {
          w.0 // Cannot apply tuple projection to word
      }
  }
  ```
- **Compiler Diagnostic Rendering:**
  ```text
  error[ORC0234]: cannot access tuple field '.0' on non-tuple type 'Word[32]'
   --> tuple_err.or:4:10
    |
  4 |         w.0
    |          ^^ field access is only valid on tuple types '(T0, T1, ...)'
  ```
- **Remediation:** Apply `.j` projections only to tuple values: `(x, y).0`.

#### `ORC0235` — `UnprintableByteString`
- **Subsystem:** Lexical & Semantic String Checker
- **Formal Trigger Predicate:**
  $$\text{Trigger}(s) \iff \exists c \in \text{Chars}(s).\ c < 0\text{x}20 \lor c > 0\text{x}7E$$
- **Theoretical Rationale:** Prevents non-printable and invisible Unicode characters from
  obfuscating cryptographic test titles or string contents.
- **Remediation:** Use `\xNN` escape sequences or `hex"..."` literals.

#### `ORC0236` — `SliceLength`
- **Subsystem:** Semantic Analyzer (Slice Bounds Checker)
- **Formal Trigger Predicate:**
  $$\text{Trigger}(a, b, n) \iff a > b \lor b > n \lor a < 0$$
- **Theoretical Rationale:** Array slice ranges $a..b$ must define valid, monotonically
  non-decreasing sub-intervals within the parent array bounds.
- **Erroneous Example:**
  ```orange
  edition 2026;
  module slice_err {
      spec bad_slice(arr: Word[8]^16) -> Word[8]^8 {
          arr[10..5] // Inverted slice bounds
      }
  }
  ```
- **Compiler Diagnostic Rendering:**
  ```text
  error[ORC0236]: invalid slice range: start '10' is greater than end '5'
   --> slice_err.or:4:13
    |
  4 |         arr[10..5]
    |             ^^^^^ slice bounds must satisfy 0 <= a <= b <= n
  ```
- **Remediation:** Ensure $0 \le a \le b \le n$: `arr[5..10]`.

#### `ORC0237` — `NonStaticSize`
- **Subsystem:** Semantic Analyzer (Size Parameter Checker)
- **Formal Trigger Predicate:**
  $$\text{Trigger}(sz) \iff \neg \text{IsCompileTimeConstant}(sz)$$
- **Theoretical Rationale:** Array dimensions and finite size arguments must evaluate to
  static constants at compile time to support monomorphization.
- **Remediation:** Supply an integer literal or compile-time constant size expression.

#### `ORC0238` — `SizeRange`
- **Subsystem:** Semantic Analyzer (Monomorphization Engine)
- **Formal Trigger Predicate:**
  $$\text{Trigger}(k, [\text{low}, \text{high}]) \iff k < \text{low} \lor k > \text{high}$$
- **Theoretical Rationale:** Passing a size argument outside the callee's declared size
  interval $[low..high]$ violates the function's monomorphization domain contract.
- **Erroneous Example:**
  ```orange
  edition 2026;
  module size_range {
      spec pad[n in 1..64](x: Word[8]^n) -> Word[8]^64 { ... }
      spec call_pad(x: Word[8]^70) -> Word[8]^64 {
          pad[70](x) // 70 exceeds declared high bound of 64
      }
  }
  ```
- **Compiler Diagnostic Rendering:**
  ```text
  error[ORC0238]: size parameter '70' is out of range for 'pad'
   --> size_range.or:5:13
    |
  3 |     spec pad[n in 1..64](x: Word[8]^n) -> Word[8]^64 { ... }
    |              ---------- declared interval is 1..64
  4 |     spec call_pad(x: Word[8]^70) -> Word[8]^64 {
  5 |         pad[70](x)
    |             ^^ size argument must be between 1 and 64
  ```
- **Remediation:** Pass a size argument residing in the declared range: `pad[64](x)`.

#### `ORC0239` — `SizeCount`
- **Subsystem:** Semantic Analyzer (Call Site Validator)
- **Formal Trigger Predicate:**
  $$\text{Trigger}(f, \text{sizes}) \iff |\text{sizes}| \ne \text{DeclaredSizeParamCount}(f)$$
- **Theoretical Rationale:** Procedures expect an exact number of size parameters.
- **Remediation:** Supply the exact number of size parameters declared in the signature.

#### `ORC0240` — `PackedWidth`
- **Subsystem:** Semantic Analyzer (Endian Packing Checker)
- **Formal Trigger Predicate:**
  $$\text{Trigger}(\tau_1, \tau_2) \iff \text{TotalBits}(\tau_1) \ne \text{TotalBits}(\tau_2)$$
- **Theoretical Rationale:** Bit-preserving packing (`as big T`, `as little T`) requires
  that total bit widths match exactly: $\text{bits}(\tau_1) = \text{bits}(\tau_2)$.
  Packing cannot synthesize or drop bits.
- **Erroneous Example:**
  ```orange
  edition 2026;
  module packed_err {
      spec pack(bytes: Word[8]^15) -> Word[32]^4 {
          bytes as big Word[32]^4 // 120 bits != 128 bits
      }
  }
  ```
- **Compiler Diagnostic Rendering:**
  ```text
  error[ORC0240]: bit width mismatch in packing conversion: 120 bits vs 128 bits
   --> packed_err.or:4:9
    |
  4 |         bytes as big Word[32]^4
    |         ^^^^^^^^^^^^^^^^^^^^^^^ cannot pack 120-bit type 'Word[8]^15' into 128-bit type 'Word[32]^4'
  ```
- **Remediation:** Ensure both source and target types have identical total bit widths:
  ```orange
  edition 2026;
  module packed_err {
      spec pack(bytes: Word[8]^16) -> Word[32]^4 {
          bytes as big Word[32]^4
      }
  }
  ```

#### `ORC0241` — `TypeParameter`
- **Subsystem:** Semantic Analyzer (Finite Type Parameter Elaborator)
- **Formal Trigger Predicate:**
  $$\text{Trigger}(T, \mathcal{S}) \iff T \notin \mathcal{S} \lor \text{DuplicatesInSet}(\mathcal{S})$$
- **Theoretical Rationale:** Type parameters are restricted to finite, explicitly enumerated
  sets $\{T_1, \dots, T_m\}$ to ensure exhaustive monomorphization. Instantiating with an
  unlisted type is rejected.
- **Erroneous Example:**
  ```orange
  edition 2026;
  module type_param {
      spec square[K in {Mod[17], Mod[31]}](x: K) -> K { x * x }
      spec call_sq() -> Int {
          square[Int](42) // 'Int' is not in the parameter set
      }
  }
  ```
- **Compiler Diagnostic Rendering:**
  ```text
  error[ORC0241]: type argument 'Int' is not in the declared parameter set
   --> type_param.or:5:16
    |
  3 |     spec square[K in {Mod[17], Mod[31]}](x: K) -> K { x * x }
    |                      ------------------ permitted types are Mod[17], Mod[31]
  4 |     spec call_sq() -> Int {
  5 |         square[Int](42)
    |                ^^^ unpermitted type parameter
  ```
- **Remediation:** Pass one of the types listed in the parameter set: `square[Mod[17]](x)`.

#### `ORC0242` — `TestTitle`
- **Subsystem:** Semantic Analyzer (Test Validator)
- **Formal Trigger Predicate:**
  $$\text{Trigger}(title) \iff |title| = 0 \lor |title| > 128 \lor \text{DuplicateTestTitle}(title)$$
- **Theoretical Rationale:** Test titles identify verification test vectors in evidence bundles
  and TAP reports. They must be unique, non-empty, and under 128 characters.
- **Remediation:** Provide a unique ASCII title between 1 and 128 characters.

---

### §97. Formatter Diagnostics (`ORC0250`–`ORC0252`): Formal Predicates, Triggers, Examples, Fixes

#### `ORC0250` — `FormatResourceLimit`
- **Subsystem:** Syntax Formatter (`orangec fmt`)
- **Formal Trigger Predicate:**
  $$\text{Trigger} \iff \text{FormatterMemoryExhausted}() \lor \text{FormattingTokens} > 1,048,576$$
- **Theoretical Rationale:** Prevents DOS during automated source formatting.
- **Remediation:** Partition large compilation units into modular files under 16 MiB.

#### `ORC0251` — `FormattingInconsistency`
- **Subsystem:** Syntax Formatter (`orangec fmt`)
- **Formal Trigger Predicate:**
  $$\text{Trigger} \iff \text{TokenStream}(\text{fmt}(\text{src})) \ne \text{TokenStream}(\text{src})$$
- **Theoretical Rationale:** Formatter invariant assertion: formatting MUST preserve the exact
  non-trivia token stream. If the AST changes, formatting halts immediately.
- **Remediation:** Internal compiler fault; report issue.

#### `ORC0252` — `FormattingRequired`
- **Subsystem:** Syntax Formatter (`orangec fmt --check`)
- **Formal Trigger Predicate:**
  $$\text{Trigger}(\text{src}) \iff \text{SourceBytes}(\text{src}) \ne \text{FormattedBytes}(\text{src})$$
- **Theoretical Rationale:** Enforces canonical source formatting in CI pipelines.
- **Erroneous Example:** Unformatted source text evaluated under `--check`.
- **Compiler Diagnostic Rendering:**
  ```text
  error[ORC0252]: source formatting required
   --> crypto.or:1:1
    |
  1 | edition 2026;module crypto{spec a()->Int{1}}
    | ^ file is not canonically formatted
  ```
- **Remediation:** Run `orangec fmt FILE` to canonically reformat whitespace.

---

### §98. Documentation Diagnostics (`ORC0260`–`ORC0261`): Formal Predicates, Triggers, Examples, Fixes

#### `ORC0260` — `DocumentationResourceLimit`
- **Subsystem:** Offline Documentation Generator (`orangec doc`)
- **Formal Trigger Predicate:**
  $$\text{Trigger} \iff \text{DocItems} > 1,048,576 \lor \text{GeneratedBytes} > 16,777,216$$
- **Theoretical Rationale:** Limits memory footprint during offline single-page HTML documentation.
- **Remediation:** Split package across multiple smaller modules.

#### `ORC0261` — `DocumentationInconsistency`
- **Subsystem:** Offline Documentation Generator (`orangec doc`)
- **Formal Trigger Predicate:**
  $$\text{Trigger} \iff \text{DuplicateAnchorId}() \lor \text{MalformedDocSpan}()$$
- **Theoretical Rationale:** Generated documentation anchors must correspond bijectively
  to verified AST declaration nodes.
- **Remediation:** Check for conflicting declaration identifiers.

---

### §99. Witness Replay Diagnostics (`ORC0270`–`ORC0274`): Formal Predicates, Triggers, Examples, Fixes

#### `ORC0270` — `NoncanonicalArgumentValue`
- **Subsystem:** Witness Replay Driver (`orangec replay`)
- **Formal Trigger Predicate:**
  $$\text{Trigger}(arg) \iff \neg \text{IsCanonicalDecimalOrHex}(arg)$$
- **Theoretical Rationale:** Counterexample replay inputs must use canonical decimal or
  lowercase hex encodings to ensure cross-platform reproducibility.
- **Remediation:** Format argument vector entries using canonical decimal or lowercase hex (`0x...`).

#### `ORC0271` — `ArgumentValueMismatch`
- **Subsystem:** Witness Replay Driver (`orangec replay`)
- **Formal Trigger Predicate:**
  $$\text{Trigger}(args, f) \iff \text{Shape}(args) \ne \text{ParamShape}(\Sigma(f))$$
- **Theoretical Rationale:** Replay arguments must match the exact parameter types and array
  lengths of the target boolean specification function.
- **Remediation:** Supply an argument vector matching the function's parameter signature.

#### `ORC0272` — `ArgumentDecodeResourceLimit`
- **Subsystem:** Witness Replay Driver (`orangec replay`)
- **Formal Trigger Predicate:**
  $$\text{Trigger} \iff \text{ReplayArgumentBytes} > 1,048,576$$
- **Theoretical Rationale:** Bounds memory allocated when decoding counterexample witness vectors.
- **Remediation:** Partition large witness inputs.

#### `ORC0273` — `InvalidWitnessReplayBinding`
- **Subsystem:** Witness Replay Driver (`orangec replay`)
- **Formal Trigger Predicate:**
  $$\text{Trigger}(f) \iff \text{ReturnType}(f) \ne \text{Bool}$$
- **Theoretical Rationale:** Witness replay checks whether a property holds or is falsified.
  The target specification function MUST return `Bool`.
- **Erroneous Example:**
  ```orange
  edition 2026;
  module bad_replay {
      spec hash(x: Word[32]) -> Word[32] { x } // Returns Word[32], not Bool
  }
  ```
- **Compiler Diagnostic Rendering:**
  ```text
  error[ORC0273]: invalid witness replay target: function 'hash' does not return 'Bool'
   --> bad_replay.or:3:5
    |
  3 |     spec hash(x: Word[32]) -> Word[32] { x }
    |     ^^^^ replay target must be a specification function returning 'Bool'
  ```
- **Remediation:** Target a boolean predicate function `spec prop(...) -> Bool`.

#### `ORC0274` — `WitnessReplayInconsistency`
- **Subsystem:** Witness Replay Driver (`orangec replay`)
- **Formal Trigger Predicate:**
  $$\text{Trigger} \iff \text{InternalReplayBindingFailure}()$$
- **Theoretical Rationale:** Internal assertion failure during witness argument binding.
- **Remediation:** Internal compiler bug; report issue.

---

### §100. Evaluator & Resource Diagnostics (`ORC0301`): Formal Predicates, Triggers, Examples, Fixes

#### `ORC0301` — `EvaluationResourceLimit`
- **Subsystem:** Reference Evaluator (`orangec eval`)
- **Formal Trigger Predicate:**
  $$\text{Trigger}(K) \iff K \le 0 \quad (\text{where step counter starts at } K_0 = 1,048,576 \text{ or } \texttt{--steps})$$
- **Theoretical Rationale:** Guarantees that evaluation halts deterministically even when
  evaluating computationally intensive algorithms, defending against infinite evaluation loops.
- **Erroneous Example:**
  ```console
  $ orangec eval --steps 100 sha256.or
  error[ORC0301]: evaluation resource limit exceeded: step budget '100' exhausted
  ```
- **Compiler Diagnostic Rendering:**
  ```text
  error[ORC0301]: evaluation resource limit exceeded
   --> sha256.or:15:9
    |
  15|         for i in 0..64 with state = init_state { ... }
    |         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ execution exceeded allocated budget of 100 steps
  ```
- **Remediation:** Increase step budget using `--steps <COUNT>`:
  ```console
  $ orangec eval --steps 2097152 sha256.or
  ```

---

## Part XVI: Toolchain, Evaluator & Formal EBNF Grammar

### §101. The Driver CLI: `orangec` Commands, Options, and Determinism

The authoritative compiler driver `orangec` provides deterministic, offline tooling:

```text
Usage: orangec <COMMAND> [OPTIONS] <FILE>
```

#### Subcommands

| Command | Operational Semantics | Exit Code |
| :--- | :--- | :--- |
| `check FILE` | Executes lexer, parser, and semantic type analysis. | 0 on success, 1 on diagnostic error, 2 on usage error. |
| `eval FILE` | Lowers source to Core IR and evaluates `spec` functions in order. | 0 on success, 1 on step exhaustion or error. |
| `test FILE` | Runs all `test` blocks in the root module. | 0 if all tests evaluate to `true`, 1 otherwise. |
| `fmt FILE` | Reformats source whitespace canonically to stdout. | 0 on success, 1 on resource limit. |
| `fmt --check FILE...` | Verifies source byte equality against canonical formatting. | 0 if identical, 1 if reformatting required (`ORC0252`). |
| `doc FILE` | Generates self-contained, offline HTML documentation to stdout. | 0 on success, 1 on limit breach. |
| `lex FILE` | Dumps the deterministic non-trivia token stream. | 0 on success. |
| `replay FILE` | Replays a witness vector against a boolean spec function. | 0 on success, printing `HoldsForThisWitness` or `Falsified`. |

#### Global Options
- `--edition 2026`: Explicitly specifies the source edition.
- `--steps <COUNT>`: Configures the reference evaluation step budget (default: $2^{20} = 1,048,576$).
- `--spec <NAME>`: Restricts evaluation to the named `spec` function.
- `--stats`: Emits exact step counts and memory words consumed during execution.
- `--version`: Emits package version, edition, and latest implemented slice:
  ```console
  $ orangec --version
  orangec 0.0.1 (Orange edition 2026; implemented slice S3t)
  ```

### §102. Deterministic Resource Limits and Denial-of-Service Defense

| Parameter | Normative Threshold | Diagnostic on Breach |
| :--- | :--- | :--- |
| **Maximum Source File Size** | 16,777,216 bytes (16 MiB) | `ORC0008` |
| **Maximum Non-Trivia Tokens** | 1,048,576 tokens ($2^{20}$) | `ORC0006` |
| **Maximum Reported Errors** | 32 per phase before suppression | `ORC0007`, `ORC0105`, `ORC0208` |
| **Maximum Integer Magnitude** | $2^{4096} - 1$ ($\approx 1234$ decimal digits) | `ORC0205` |
| **Admitted Word Bit Widths ($W$)** | Exactly $\{8, 16, 32, 64\}$ bits | `ORC0204` |
| **Admitted Modular Moduli ($m$)** | $2 \le m \le 2^{521} - 1$ | `ORC0232` |
| **Maximum Array Length ($n$)** | 65,536 elements ($2^{16}$) | `ORC0221` |
| **Maximum Nested Array Elements** | 65,536 scalar elements ($2^{16}$) | `ORC0221` |
| **Tuple Arity Range ($k$)** | $2 \le k \le 16$ | `ORC0234` |
| **Default Evaluator Step Budget** | 1,048,576 steps ($2^{20}$) | `ORC0301` |
| **Documentation Work Items** | 1,048,576 items ($2^{20}$) | `ORC0260` |
| **Documentation Output Size** | 16,777,216 HTML bytes (16 MiB) | `ORC0260` |

### §103. Complete Unified Formal EBNF Grammar

```text
(* Orange 2026 Complete Formal EBNF Grammar *)

source_file         = byte* ; (* UTF-8 encoded, <= 16 MiB, no BOM *)

whitespace          = " " | "\t" | "\n" | "\r\n" ;
line_comment        = "//" (any_char - "\n")* ;
block_comment       = "/*" (block_comment | any_char)* "*/" ;

identifier          = identifier_start identifier_continue* ;
identifier_start    = "A".."Z" | "a".."z" | "_" ;
identifier_continue = identifier_start | "0".."9" ;

reserved_word       = "edition" | "module" | "spec" | "impl"
                    | "game"    | "proof"  | "claim" ;

integer_literal     = decimal_literal | hex_literal | binary_literal ;
decimal_literal     = digit (digit | "_")* ;
hex_literal         = ("0x" | "0X") hex_digit (hex_digit | "_")* ;
binary_literal      = ("0b" | "0B") binary_digit (binary_digit | "_")* ;
digit               = "0".."9" ;
hex_digit           = digit | "a".."f" | "A".."F" ;
binary_digit        = "0" | "1" ;

string_literal      = "\"" string_char* "\"" ;
string_char         = (any_ascii_printable - ("\"" | "\\")) | escape_seq ;
escape_seq          = "\\\"" | "\\\\" | "\\n" | "\\r" | "\\t" | "\\0" | "\\x" hex_digit hex_digit ;

hex_string_literal  = "hex\"" (hex_digit hex_digit | " ")* "\"" ;

compilation_unit    = edition_decl module_decl ;

edition_decl        = "edition" "2026" ";" ;

module_decl         = "module" identifier "{" module_body "}" ;
module_body         = use_decl* type_decl* member_decl* ;

use_decl            = "use" identifier ";" ;

type_decl           = "type" identifier "=" parsed_type ";" ;

member_decl         = spec_decl
                    | impl_decl
                    | game_decl
                    | proof_decl
                    | claim_decl
                    | test_decl ;

(* Types *)
parsed_type         = scalar_type
                    | array_type
                    | tuple_type
                    | identifier ;

scalar_type         = "Int"
                    | "Bool"
                    | "Byte"
                    | "Word" "[" integer_literal "]"
                    | "Mod" "[" modulus_expr "]" ;

array_type          = parsed_type "^" size_expr ;
tuple_type          = "(" parsed_type ("," parsed_type)+ ")" ;

modulus_expr        = shift_expr ;
size_expr           = integer_literal | identifier ;

(* Specification Stratum *)
spec_decl           = "spec" identifier [size_params] [type_params]
                      "(" [param_list] ")" ("->" parsed_type)? "{" spec_body "}" ;

size_params         = "[" identifier "in" size_expr ".." size_expr "]" ;
type_params         = "[" identifier "in" "{" parsed_type ("," parsed_type)* "}" "]" ;

param_list          = param ("," param)* ;
param               = identifier ":" parsed_type ;

spec_body           = (let_binding ";")* expression? ;
let_binding         = "let" pattern (":" parsed_type)? "=" expression ;

pattern             = identifier
                    | "(" identifier ("," identifier)+ ")" ;

(* Expressions *)
expression          = conditional_expr
                    | loop_expr
                    | binary_expr ;

conditional_expr    = "if" expression "{" block_body "}" "else" "{" block_body "}" ;
loop_expr           = "for" identifier "in" size_expr ".." size_expr
                      "with" pattern "=" expression "{" block_body "}" ;

block_body          = (let_binding ";")* expression ;

binary_expr         = unary_expr (binary_op unary_expr)* ;
unary_expr          = ("-" | "~" | "!")? primary_expr ;

primary_expr        = literal_expr
                    | qualified_ident
                    | call_expr
                    | index_or_slice_expr
                    | update_expr
                    | tuple_expr
                    | array_lit_expr
                    | fill_expr
                    | cast_expr
                    | "(" expression ")" ;

qualified_ident     = (identifier "::")? identifier ;
call_expr           = qualified_ident [size_args] [type_args] "(" [arg_list] ")" ;
size_args           = "[" size_expr ("," size_expr)* "]" ;
type_args           = "[" parsed_type ("," parsed_type)* "]" ;
arg_list            = expression ("," expression)* ;

index_or_slice_expr = primary_expr "[" (expression | slice_range) "]" ;
slice_range         = size_expr ".." size_expr ;

update_expr         = primary_expr "with" "[" (expression | slice_range) "]" "=" expression ;

tuple_expr          = "(" expression ("," expression)+ ")" ;
array_lit_expr      = "[" expression ("," expression)* "]" ;
fill_expr           = "[" expression ";" size_expr "]" ;

cast_expr           = primary_expr "as" [byte_order] parsed_type ;
byte_order          = "big" | "little" ;

(* Tests *)
test_decl           = "test" string_literal "{" expression "}" ;
```
