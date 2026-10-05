# The Orange Reference Manual

<img src="images/orange-reference-manual-cover.png" width="400" alt="The Orange Reference Manual cover: orange emblem and oversized ORANGE lettering on black.">

By Chase Bryan

Status: Current reference for the Orange 2026 `spec` stratum through implemented slice S3u (Parts I–VII). Parts VIII–XVI are Proposed.

Snapshot: 2026-10-05

Edition: `2026`

---

> Parts I–VII are the reference for Orange Edition `2026` as implemented by
> `orangec` 0.0.1 through slice S3u (OEP-0025): the `spec` stratum, its types,
> its operational semantics, and the cryptographic transcriptions in Part VII.
> Parts VIII–XVI record the Proposed 1.0 architecture (`impl`, `machine impl`,
> `game`, `proof`, `claim`, the foreign interface, and the toolchain narrative).
> They are not the behavior of the Current compiler. A diagnostic code that
> slice S3u already emits stays Current wherever a Proposed section names it.

---

## Contents

- [Preface](#preface)
  - [§1. Scope of This Manual](#1-scope-of-this-manual)
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
  - [§16. Import Dependency Graphs and Cycle Rejection](#16-import-dependency-graphs-and-cycle-rejection)
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
  - [§27. Byte Arrays (`Word[8]^n`)](#27-byte-arrays-word8n)
  - [§28. Explicit Value Conversions (`expr as T`) and Typing Judgments](#28-explicit-value-conversions-expr-as-t-and-typing-judgments)
  - [§29. Endianness Homomorphisms and Bit-Preserving Packing (`as big T`, `as little T`)](#29-endianness-homomorphisms-and-bit-preserving-packing-as-big-t-as-little-t)
  - [§30. Dependent Finite Size Parameters and Monomorphization](#30-dependent-finite-size-parameters-and-monomorphization)
  - [§31. Finite Type Parameter Domains ($[K \in \{T_1, \dots, T_m\}]$)](#31-finite-type-parameter-domains-k-in-t_1-dots-t_m)

- [Part IV: Static Semantics (Typing Rules & Judgments)](#part-iv-static-semantics-typing-rules--judgments)
  - [§32. Typing Contexts: Signature ($\Sigma$), Size ($\Theta$), Type ($\Delta$), and Variable ($\Gamma$) Environments](#32-typing-contexts-signature-sigma-size-theta-type-delta-and-variable-gamma-environments)
  - [§33. Subtyping, Coercion Freedom, and Structural Type Equality](#33-subtyping-coercion-freedom-and-structural-type-equality)
  - [§34. Expression Typing Rules and Formal Judgments](#34-expression-typing-rules-and-formal-judgments)
  - [§35. Expression Grouping Envelopes and Syntactic Ambiguity Rejection](#35-expression-grouping-envelopes-and-syntactic-ambiguity-rejection)
  - [§36. Static Index Ranges](#36-static-index-ranges)
  - [§37. Data-Dependent Indexing, Range Narrowing, and S-Box Lookups](#37-data-dependent-indexing-range-narrowing-and-s-box-lookups)

- [Part V: Dynamic Semantics (Operational & Reduction Rules)](#part-v-dynamic-semantics-operational--reduction-rules)
  - [§38. Abstract Syntax and Typed Reference Core Lowering](#38-abstract-syntax-and-typed-reference-core-lowering)
  - [§39. Evaluation Environments, Value Stores, and Step Budgets](#39-evaluation-environments-value-stores-and-step-budgets)
  - [§40. Small-Step Operational Semantics (SOS) and Big-Step Reduction](#40-small-step-operational-semantics-sos-and-big-step-reduction)
  - [§41. Operational Reduction of Variable Shifts, Rotations, and Inversions](#41-operational-reduction-of-variable-shifts-rotations-and-inversions)
  - [§42. Bounded Iteration Semantics and Step-Cost Accounting](#42-bounded-iteration-semantics-and-step-cost-accounting)
  - [§43. Known-Answer Specification Tests (`test`) and Whole-Aggregate Equality](#43-known-answer-specification-tests-test-and-whole-aggregate-equality)

- [Part VI: Formal Metatheory of the Specification Stratum](#part-vi-formal-metatheory-of-the-specification-stratum)
  - [§44. Evaluation Bounds](#44-evaluation-bounds)
  - [§45. Checking Before Evaluation](#45-checking-before-evaluation)
  - [§46. Determinism of the Reference Evaluator](#46-determinism-of-the-reference-evaluator)
  - [§47. Byte-Order Conversions at Equal Width](#47-byte-order-conversions-at-equal-width)

- [Part VII: Specification Corpus & Reference Cryptographic Standards](#part-vii-specification-corpus--reference-cryptographic-standards)
  - [§48. Mathematical Transcription Methodology and Traceability](#48-mathematical-transcription-methodology-and-traceability)
  - [§49. Complete Reference Specification: FIPS 180-4 SHA-256](#49-complete-reference-specification-fips-180-4-sha-256)
  - [§50. Complete Reference Specification: RFC 8439 ChaCha20](#50-complete-reference-specification-rfc-8439-chacha20)
  - [§51. Complete Reference Specification: Curve25519 / X25519 (RFC 7748)](#51-complete-reference-specification-curve25519--x25519-rfc-7748)
  - [§52. Complete Reference Specification: Poly1305 Field MAC (RFC 8439)](#52-complete-reference-specification-poly1305-field-mac-rfc-8439)
  - [AES, FIPS 197](#aes-fips-197)
  - [HMAC, FIPS 198-1](#hmac-fips-198-1)
  - [HKDF, RFC 5869](#hkdf-rfc-5869)
  - [AEAD, RFC 8439](#aead-rfc-8439)
  - [SHA-3 and SHAKE, FIPS 202](#sha-3-and-shake-fips-202)

- [Part VIII: Implementation Stratum (`impl`) & Memory Model](#part-viii-implementation-stratum-impl--memory-model)
  - [§53. Imperative Execution Semantics and Place Logic](#53-imperative-execution-semantics-and-place-logic)
  - [§54. Structured Memory Model: Heap-Freedom, Regions, and Stack Layout](#54-structured-memory-model-heap-freedom-regions-and-stack-layout)
  - [§55. Affine Ownership, Move Semantics, and Capability Borrowing (`&T`, `&mut T`)](#55-affine-ownership-move-semantics-and-capability-borrowing-t-mut-t)
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
  - [§77. Functional Refinement Relations](#77-functional-refinement-relations)
  - [§78. Proof IR: Canonical Encoding, De Bruijn Terms, and Cryptographic Fingerprints](#78-proof-ir-canonical-encoding-de-bruijn-terms-and-cryptographic-fingerprints)
  - [§79. Weakest Precondition Calculus and Verification Conditions](#79-weakest-precondition-calculus-and-verification-conditions)
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
  - [§103. Proposed Surface Grammar](#103-proposed-surface-grammar)

---

## Preface

### §1. Scope of This Manual

The Current language, through implemented slice S3u, is the `spec` stratum:
pure functions over `Int`, `Bool`, `Word[n]`, `Mod[m]`, arrays, and tuples, plus
`test`. An empty `impl` may be declared and is not evaluated. A typed `impl`
on the surface is `ORC0101` (§55, §96). `ORC0202` is the semantic gate for a
typed body whose kind is not `spec`; the parser does not build that body.
`game`, `proof`, and `claim` are reserved words. A member that begins with
one of them is `ORC0103`. Using one as a function name is `ORC0101`, note
`` reserved words cannot be used as names ``.

The five-stratum architecture (`impl`, `machine impl`, `game`, `proof`, `claim`)
is Proposed. Parts VIII–XVI record it. Those parts are not conformance
requirements for `orangec` 0.0.1.

- **Claims, not labels** remains the Proposed assurance rule (D-005): a claim
  would bind a subject, a relation, assumptions, and evidence. The Current
  compiler does not check claims.

### §2. Conformance, Formality, and Normative Terminology

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHALL**, **SHALL NOT**,
**SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY**, and **OPTIONAL** in this
document are to be interpreted as described in BCP 14 (RFC 2119 / RFC 8174).

- A **conforming Orange source file** is a valid UTF-8 sequence conforming to the
  lexical and syntactic grammar defined herein, beginning with an edition declaration.
- A **conforming Orange compiler** for this manual's Current text is an
  executable that enforces the lexical, syntactic, and semantic rules of Parts
  I–V, evaluates by Part V under the step budget of §39, and reports the codes
  in Part XV. Parts VIII–XVI are not part of that conformance claim.
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

| Slice / Stratum | Grammar Status | Static Type Rules | Operational Rules | Compiler Status (`orangec 0.0.1`) | Record |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **S1 / S2**: Lexical & Syntax Foundation | Normative | Normative | N/A (Syntax Only) | Accepted | D-025, OEP-0002 |
| **S3a**: Typed `spec` Literals (`Int`, `Word[8]`) | Normative | Normative | Reference Core Eval | Accepted | D-026, OEP-0003 |
| **S3b**: Pure Expressions, Word Rings, Operators | In review | In review | Small-Step Reductions | Implemented | OEP-0005 |
| **S3c**: Typed `let` Bindings & `as` Conversions | In review | In review | Environment Substitution | Implemented | OEP-0006 |
| **S3d**: Fixed-Length Arrays `T^n` & Indexing | In review | In review | Exact Bound Array Eval | Implemented | OEP-0007 |
| **S3e**: Bounded Loops `for..in..with` & Updates | In review | In review | Deterministic Iteration | Implemented | OEP-0008 |
| **S3f**: `Bool`, Conditionals `if/else`, Division | In review | In review | Exhaustive Branch Eval | Implemented | OEP-0009 |
| **S3g**: Data-Dependent Lookups & S-Boxes | In review | In review | Range-Proved Lookups | Implemented | OEP-0010 |
| **S3h**: Multi-Module Programs & `use` Imports | In review | In review | Acyclic Module Eval | Implemented | OEP-0011 |
| **S3i**: Residue Fields `Mod[m]` & `type` Aliases | In review | In review | Modular Reduction Eval | Implemented | OEP-0012 |
| **S3j**: Local `let` in Loop & Branch Blocks | In review | In review | Nested Scope Eval | Implemented | OEP-0013 |
| **S3k**: Tuple Types `(T, U)` & Patterns | In review | In review | Product Destruction | Implemented | OEP-0014 |
| **S3l**: Byte Strings `"..."`, `hex"..."`, Slices | In review | In review | Byte Vector Operations | Implemented | OEP-0015 |
| **S3m**: Finite Size Parameters `[n in low..high]` | In review | In review | Eager Monomorphization | Implemented | OEP-0016 |
| **S3n**: Explicit Byte Ordering (`big`, `little`) | In review | In review | Bit-Preserving Packing | Implemented | OEP-0017 |
| **S3o**: Finite Type Parameters `[K in {T...}]` | In review | In review | Eager Monomorphization | Implemented | OEP-0018 |
| **S3p**: Array Lengths to 65,536 & Eval Limits | In review | In review | Bounded Step Counter | Implemented | OEP-0019 |
| **S3q**: Known-Answer Specification `test` Blocks | In review | In review | Automated Test Runner | Implemented | OEP-0020 |
| **S3r**: Variable Shift & Rotation Amounts | In review | In review | Ring Turn Reduction | Implemented | OEP-0021 |
| **S3s**: Bounded Rectangular Nested Arrays | In review | In review | Chained Indexing | Implemented | OEP-0023 |
| **S3t**: Static Moduli with Own Size Names | In review | In review | Size-Dependent Moduli | Implemented | OEP-0024 |
| **S3u**: Rank 3 and 4 Arrays, Update Paths | In review | In review | Nested Path Update | Implemented | OEP-0025 |
| **Implementation Stratum (`impl`)** | Proposed | Proposed | Imperative Place Semantics | Not implemented | D-004, ST-REL |
| **Machine Stratum (`machine impl`)** | Proposed | Proposed | Target ISA Simulation | Not implemented | D-004, D-011 |
| **Game Stratum (`game`)** | Proposed | Proposed | Probabilistic Sampling | Not implemented | D-004, ST-REL |
| **Proof Stratum (`proof`)** | Proposed | Proposed | Proof IR Deduction | Not implemented | D-006, D-007 |
| **Assurance & Claims (`claim`)** | Proposed | Proposed | Content-Addressed Graph | Not implemented | D-005, AM-01 |

`Implemented` in the table means the post-#235 compiler, whose `orangec --version`
reports `implemented slice S3u`. The rows below S3u are Proposed and are not that
compiler. This documentation branch does not contain the S3u compiler commit; the
judgments are those of OEP-0025 and `docs/DIMENSIONS_2026.md` on that tip.

---

## Part I: Lexical & Concrete Syntactic Grammar

### §5. Byte-Level Encoding, Normalization Invariants, and Source Limits

1. **Source Encoding:** An Orange source compilation unit MUST consist of a contiguous
   sequence of octets representing valid UTF-8 according to RFC 3629 / Unicode Standard.
2. **U+FEFF:**
   U+FEFF has no lexical production, at offset 0 or elsewhere. The lexer reports
   it as an unexpected character, diagnostic `ORC0001`. There is no separate BOM
   check.
3. **File Size Ceiling:**
   $$|\text{Source}| \le 16,777,216 \text{ bytes} \ (16 \text{ MiB})$$
   A source longer than 16 MiB is rejected by the driver before lexing, diagnostic
   `ORC1003`. `ORC0008` is not that rejection. `ORC0008` means the lexer could not
   reserve its bounded token stream.
4. **Line Endings:**
   A logical line ends at U+000A, at U+000D, or at U+000D U+000A. A bare carriage
   return is a line ending. It is not `ORC0001`.
5. **Bidirectional Control Character Defense:**
   To guarantee protection against source spoofing and Trojan Source attacks (CVE-2021-42574),
   the compiler strictly rejects any code point in the following sets outside of
   string literals:
   - C0 control characters (U+0000 through U+001F), except U+0009, U+000A, and U+000D, which are whitespace.
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
   - Carriage Return (`0x0D`, U+000D), including a bare CR
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
   The compiler represents integer magnitudes exactly. A value whose magnitude
   has more than **16,384 significant bits** ($|x| \ge 2^{16384}$) emits
   diagnostic `ORC0205`. A value with 16,384 significant bits is admitted.
   The check is `magnitude_bits`, not a decimal-digit budget.

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
   - Maximum non-trivia tokens per source, excluding EOF: **262,144**.
     Exceeding this budget emits diagnostic `ORC0006`
     (`source exceeds the 262144-token lexical limit`).
   - Ordinary lexical errors: **100**, then one suppression diagnostic `ORC0007`.
     The same count of 100, then one suppression, applies to parse errors
     (`ORC0105`) and semantic errors (`ORC0208`).
   - `ORC0008` means the lexer could not reserve its bounded token stream, or
     the source cursor was not on a UTF-8 boundary. It is not the 16 MiB file
     limit (`ORC1003`) and it is not the token-count limit (`ORC0006`).

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
       // 3. Members: spec, impl, test
   }
   ```

2. Any tokens appearing after the closing brace `}` of the module declaration
   emit diagnostic `ORC0104`.
3. **Top-Level Member Ordering Rules:**
   - All `use` declarations precede every `type` declaration and every function.
   - All `type` declarations precede every function and every `test`.
   - A `use` after a `type`, or a `use` or `type` after a function, is `ORC0103`.
   - Members after that are `spec`, `impl`, and `test`, in any order among themselves.
   - `game`, `proof`, and `claim` are reserved words. Using one where a member
     was expected is `ORC0103` (`expected a spec or impl function declaration`).
     They are not grammar productions of the Current language.

### §15. Hermetic Module Resolution and Filesystem Path Mapping

1. **Hermeticity:** Orange modules are resolved hermetically without ambient
   network queries or implicit system paths.
2. **File Mapping:**
   - An import `use m;` names the file `m.or`.
   - `orangec` reads that file from the directory of the **root** source, or
     from the current directory when the root is standard input. It does not
     search the directory of the importing file, and it does not read
     `Orange.toml`. There is no package-root configuration.
   - A name that is not a module of the program is diagnostic `ORC0228`.
3. **Qualified Identifiers:**
   - Imported definitions are referenced using the module prefix: `m::symbol`.
   - Calling `m::symbol` when `use m;` was not declared in the importing module
     emits diagnostic `ORC0229`.
   - Unqualified imports (wildcards like `use m::*;`) do not exist in Orange.

### §16. Import Dependency Graphs and Cycle Rejection

1. **Dependency Graph Invariant:**
   Let $G = (V, E)$ be the module dependency graph where $V$ is the set of all
   modules and $(u, v) \in E \iff u \text{ contains } \texttt{use } v;$.
   $G$ MUST be a Directed Acyclic Graph (DAG).
2. **Cycle Rejection:**
   The compiler walks uses depth-first from the root and reports a cycle at the
   `use` that closes it, diagnostic `ORC0230`. It does not run Tarjan's
   algorithm, and the diagnostic does not name an SCC.
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
   - A type alias identifier MUST NOT name a built-in type. The built-in types
     are `Int`, `Bool`, `Word[n]`, and `Mod[m]`. `Byte` is not one of them.
     `type Byte = Word[8];` is a legal alias. `type Int = Word[64];` emits
     `ORC0233` (`Int` is a built-in type).
   - Declaring duplicate type alias names within the same module emits `ORC0233`.

### §18. Multi-Strata Architectural Separation and Disjoint Namespaces

1. **Current declarations.** `spec` and `impl` are separate declaration
   namespaces (`ORC0201`). The same module may declare `spec sha256` and
   `impl sha256`. A call names a typed `spec` function. There is no stratum
   selector at the call. A typed `impl` on the surface is two `ORC0101`
   diagnostics (§55) and does not emit `ORC0202`. An empty `impl` declares a
   name and does not evaluate. `game`, `proof`, and `claim` are not
   declaration forms (§14, §103).
2. **Proposed strata.** Affine ownership, machine intrinsics, games, proof
   terms, and claims are Parts VIII–XIII. They are not the Current evaluator.

---

## Part III: Formal Type System & Algebraic Foundations

### §19. Semantic Value Domains and Mathematical Universes

We define the universe of semantic values $\mathbb{V}$ as the disjoint union of
concrete value domains:

$$\mathbb{V} = \mathbb{V}_{\text{Int}} \uplus \mathbb{V}_{\text{Word}} \uplus \mathbb{V}_{\text{Mod}} \uplus \mathbb{V}_{\text{Bool}} \uplus \mathbb{V}_{\text{Array}} \uplus \mathbb{V}_{\text{Tuple}}$$

Where:

- $\mathbb{V}_{\text{Int}} = \{ x \in \mathbb{Z} \mid |x| < 2^{16384} \}$
- $\mathbb{V}_{\text{Word}} = \biguplus_{W \in \{8, 16, 32, 64\}} (\mathbb{Z} / 2^W \mathbb{Z})$
- $\mathbb{V}_{\text{Mod}} = \biguplus_{m \in [2, 2^{521}-1]} (\mathbb{Z} / m \mathbb{Z})$
- $\mathbb{V}_{\text{Bool}} = \{\text{true}, \text{false}\}$
- $\mathbb{V}_{\text{Array}} = \biguplus_{\tau, n} \mathbb{V}_\tau^n$ ($1 \le n \le 65,536$)
- $\mathbb{V}_{\text{Tuple}} = \biguplus_{k \in [2, 16]} (\mathbb{V}_{\tau_0} \times \dots \times \mathbb{V}_{\tau_{k-1}})$

### §20. Mathematical Integers ($\mathbb{Z}$) and the `Int` Type

1. The `Int` type models the algebraic ring of mathematical integers:
   $$(\mathbb{Z}, +, \cdot, -, 0, 1)$$
2. **Exact Arithmetic:** Operations on `Int` never overflow or wrap.
3. **Safety Bound:** An integer with more than 16,384 significant bits
   ($|x| \ge 2^{16384}$) emits diagnostic `ORC0205`. Shifts and rotations are
   not defined on `Int` (`ORC0215`); `1 << 4096` is that diagnostic, not
   `ORC0205`.
4. **Absence of Coercion:** An `Int` cannot be passed where a `Word[W]` or
   `Mod[m]` is required without an explicit `as` cast. The cast reduces; it
   does not assert that the value already lies in the destination (§28).

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

`Bool` has two values. The operators the S3t checker defines for an expected
type `Bool` are `!`, `&&`, `||`, `==`, and `!=`. That list is
`BOOL_OPERATOR_NOTE` in `compiler/crates/orange-compiler/src/semantics.rs`.
Slice S3f in §4 is the register row: grammar in review, compiler implemented.
These judgments are that checker. Secrecy labels are §67.

`true` and `false` are identifiers. Section 32 resolves an unbound spelling
as the `Bool` literal. A binding of that spelling hides the literal.

**Negation.** `!` is defined only when the expected type is `Bool`.

$$
\frac{\Gamma \vdash e : \mathrm{Bool}}{\Gamma \vdash {!}\,e : \mathrm{Bool}}
$$

`!x` where `x: Int` and `Int` is required is `ORC0215`, message
`` prefix `!` is not defined for `Int` ``, note
`` `!` negates a `Bool`; `-` negates an `Int` or a residue ``.

**Conjunction and disjunction.** Both operands have type `Bool`, and the
result is `Bool`.

$$
\frac{\Gamma \vdash a : \mathrm{Bool} \quad \Gamma \vdash b : \mathrm{Bool}}{\Gamma \vdash a \mathbin{\&\&} b : \mathrm{Bool}}
\qquad
\frac{\Gamma \vdash a : \mathrm{Bool} \quad \Gamma \vdash b : \mathrm{Bool}}{\Gamma \vdash a \mathbin{\vert\vert} b : \mathrm{Bool}}
$$

`x && x` where `x: Int` and the result type is `Int` is `ORC0215`, message
`` `&&` is not defined for `Int` ``, note
`` `&&` and `||` apply to `Bool` values; `&` and `|` are the bitwise operators on words ``.

**Strict evaluation.** A `Bool` binary node evaluates both operands, then the
operator. The comment in `compiler/crates/orange-compiler/src/eval.rs` on that
arm is "Both operands are evaluated". On this tree, `orangec eval --stats`
reports 19 steps for `false && (costly() == 0)` and 19 steps for
`true || (costly() == 0)`, where `costly` is `1 + 1 + 1 + 1 + 1` (13 steps)
and `false` alone is 1 step. Both calls pay for `costly`. Short-circuit
control flow for `impl` is Proposed (Part VIII).

**Equality.** `==` and `!=` are defined for every type `check_comparison`
assigns to the operands, including `Bool`, `Mod[m]`, arrays, and tuples. Both
operands are checked at that one type. The comparison's type is `Bool`.

$$
\frac{\Gamma \vdash a : \tau \quad \Gamma \vdash b : \tau}{\Gamma \vdash (a \mathbin{==} b) : \mathrm{Bool}}
\qquad
\frac{\Gamma \vdash a : \tau \quad \Gamma \vdash b : \tau}{\Gamma \vdash (a \mathbin{!=} b) : \mathrm{Bool}}
$$

**Order.** `<`, `<=`, `>`, and `>=` require `CoreType::is_ordered`
(`compiler/crates/orange-compiler/src/core.rs`): the type is `Int` or
`Word[W]`. `Bool`, `Mod[m]`, an array, and a tuple fail that predicate.

$$
\frac{\Gamma \vdash a : \tau \quad \Gamma \vdash b : \tau \quad \mathrm{ordered}(\tau)}{\Gamma \vdash a \mathbin{\mathrm{ord}} b : \mathrm{Bool}}
$$

`x < y` at `Mod[7]` is `ORC0215`, message `` `<` is not defined for `Mod[7]` ``,
note that residues are compared with `==` and `!=` and have no order.
`a < b` at `Bool` is the same code, note
`` `Bool` values are compared with `==` and `!=`; they have no order ``.

**Result context.** A comparison whose context is some type other than `Bool`
is `ORC0214`. `x == 0` where `Int` is required is the message
`` a comparison gives `Bool`, but `Int` is required here ``. Two operands with
no typed leaf are `ORC0227`.

**Conditional.** Every condition of `if` / `else` is checked as `Bool`, and
every arm is checked against the expected type (`check_conditional`).
Evaluation enters one arm: `begin_branch` pops the condition and selects that
part. On this tree, `if true { 1 + 1 + 1 } else { 2 }` is 9 steps and
`if false { 1 + 1 + 1 } else { 2 }` is 3 steps. Both arms are type-checked.
The unselected arm is not stepped.

**Worked derivation.** `orangec check` accepts this program:

```orange
edition 2026;
module m {
    spec both(x: Int) -> Bool { (x < 0) && (0 < x) }
}
```

`x` is the parameter, of type `Int`. `Int` is ordered. Each comparison's
context is `Bool`, because `&&` requires `Bool`, so `x < 0 : Bool` and
`0 < x : Bool`. `&&` is defined for that expected type, so the body has type
`Bool`, the declared result.

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

Current through slice S3u (OEP-0025), over the nested-array rules of S3s. An array
of rank $r + 1$ holds arrays of rank $r$, all of one type. The scalar leaves are
the S3s leaves, including S3t residue domains.

#### 1. Rank

The admitted ranks are 1, 2, 3, and 4. A fifth dimension is rejected before
evaluation with `ORC0203`. For an alias `Hyper` of rank 4 the message is
`` `Hyper` already has 4 array dimensions ``, labeled "arrays have at most 4
dimensions", with the secondary label "this length would add a fifth dimension"
at the outer length. The same rejection applies in aliases, parameter and result
types, tuple fields, and every instance of a size or type parameter.

S3u adds no type syntax. Each dimension is a `type` alias over the one before it:

```orange
type Zq = Mod[3329];
type Poly = Zq^256;
type Vector = Poly^2;
type Matrix = Vector^2;
```

`Word[8]^2^2`, and a parenthesized array type followed by `^n`, remain parser
errors, as in S3s. Repeated powers are not source syntax. The diagnostic and
evaluation display parenthesizes each level from the innermost out, as
`((Mod[3329]^256)^2)^2`; that display is not source syntax. Aliases of the same
leaf and the same axes, in the same order, are the same type.

AES keeps a $4 \times 4$ state of bytes and Keccak a $5 \times 5$ state of lanes,
both rank 2. An ML-KEM matrix is a $k \times k$ array of polynomials of 256
coefficients, rank 3. A batch of such matrices is rank 4.

#### 2. Axes and the Scalar Limit

Every axis length $n$ satisfies $1 \le n \le 65,536$. The product of the axes,
the number of scalar leaves, is at most 65,536. A cube of $p$ planes, $r$ rows,
and $c$ columns requires $p \times r \times c \le 65,536$. A $16 \times 16 \times
16 \times 16$ array is admitted, and so is $1 \times 1 \times 1 \times 65,536$.
A declared product over the limit is `ORC0221`, message "an array shape has N
scalar elements, exceeding 65536", labeled "array shape exceeds the scalar
element limit". Each size-parameter instance is checked as if written out, and
the first erroneous instance is named. Shape checks use bounded arithmetic; an
overflow does not admit a shape.

Literals and fills nest to every rank. Each level is checked against its own
element type and length. A ragged level is `ORC0222`. A fill evaluates its
element once and repeats it.

#### 3. Selection

`a[i]` selects one element of the outermost dimension. Up to four successive
indices reach a scalar, as `m[i][j][k]`. Each index is checked against its own
axis by the rules of S3d, S3e, and S3g. A possibly out-of-range index is
`ORC0223`. A further index after a scalar is `ORC0224`. Selection returns the
exact element type and does not flatten. Each index costs one step beyond its
operands.

#### 4. Slices and Joins

A slice `a[x..y]` selects consecutive elements of the outermost dimension and
keeps their element type. A slice of a selected row or plane does the same one
level in. `++` joins the outer elements of two arrays of one exact element type,
and the joined shape must satisfy the scalar limit. A slice update
`a with [x..y] = b` replaces a run of outer elements. A selection does not
follow a slice, and no operation selects a rectangular window across several axes.

#### 5. Update Paths

An update may name one index per dimension it reaches:

```orange
c with [i][j][k] = v
```

The grammar of an update target is a single index, two to four successive
single indices, or one range. A slice inside a path, an empty index, or a fifth
index is `ORC0101`. The parser note says that an element of a row is updated
with `x with [i][j] = v`, and a run of a row as `x with [i] = (x[i] with [a..b] = v)`.
Paths apply at rank 2 as well.

The first index selects within the base, and each further index within the
element the one before it reached. The value must have the exact type at the
end of the path, a scalar or a shorter array, and the update has the base's
type. A path whose indices reach past the scalars is `ORC0224`, message "only
an array can be indexed, but this selects within `T`", with the note that the
indices reach past the array's scalars. A value of the wrong type or length is
`ORC0214` or `ORC0222`.

#### 6. Path Meaning and Cost

A path denotes the nested updates it abbreviates, and every element off the
path keeps its value:

```orange
c with [i][j][k] = v
// denotes
c with [i] = (c[i] with [j] = (c[i][j] with [k] = v))
```

Evaluation visits the base, then the indices in order, then the value. Each
index is evaluated once. The value equals the nested updates. The step cost
does not: beyond its operands, a path costs $\lceil n / 64 \rceil$ steps, and
at least one, for each level copied, where $n$ is that level's outer length.
On a $16 \times 16 \times 256$ cube, `c with [i][j][k] = v` costs $1 + 1 + 4$
steps beyond its base, three indices, and value. The nested spelling also
selects `c[i]` and `c[i][j]`, so it costs more. The budget is an evaluation
budget, not a timing or constant-time guarantee.

#### 7. What S3u Does Not Add

Arrays of rank 3 and 4 may be parameters, results, `let` bindings, loop
accumulators, and tuple fields. A finite type parameter may list them, and a
size parameter may supply any axis. `as`, `as big`, and `as little` still
reject an array of arrays as source or target. Arrays of tuples, and type
arguments spelled as repeated powers, stay rejected. No command, option, token,
or reserved word is added. Every S3t source keeps its types, values, output
bytes, and evaluation steps.

### §26. Heterogeneous Product Types: Tuples ($(T_0, \dots, T_{k-1})$ for $2 \le k \le 16$)

1. A tuple type $(T_0, \dots, T_{k-1})$ represents the heterogeneous product:

$$
\prod_{i=0}^{k-1} T_i = T_0 \times T_1 \times \dots \times T_{k-1}
$$

2. **Arity.** A tuple value and a tuple type have two through 16 elements
   (`MAX_TUPLE_ELEMENTS` in `compiler/crates/orange-compiler/src/parser.rs`).
   The 17th element is a parser resource limit, `ORC0106`, message
   `` a tuple has more than 16 elements `` or
   `` a tuple type has more than 16 elements ``, label
   `` deterministic parser resource limit reached ``. It is not a semantic code.
   One parenthesized expression without a comma is a group.
   `spec bad() -> Int { (1) }` checks. `(1,)` is `ORC0101`, message
   `` expected another element after `,` ``, note
   `` a tuple is written `(a, b)` with two through 16 elements; `(a)` without a comma is a group ``.
   A one-element type `(Int)` is `ORC0101`, message
   `` expected `,` and another element type ``.

3. **Elements are not tuples.** An element type written as a tuple is
   `ORC0101`, message `` expected an element type ``, with the note that a
   tuple holds no tuple. A `type` alias that denotes a tuple passes that
   parse and fails in semantics. `type Pair = (Int, Int);` and a parameter
   of type `(Pair, Bool)` are `ORC0203`, message
   `` `Pair` is a tuple type, so this is a tuple of tuples ``, label
   `` a tuple holds no tuple `` (`tuple_of` in
   `compiler/crates/orange-compiler/src/semantics/tuples.rs`).

4. **Projection.** The position is a decimal integer counted from zero.

$$
\frac{\Gamma \vdash t : (T_0, \dots, T_{k-1}) \quad 0 \le j < k}{\Gamma \vdash t.j : T_j}
$$

`orangec check` accepts `spec second(p: (Int, Bool)) -> Bool { p.1 }`.
`p.2` on that type is `ORC0223`, message `` `(Int, Bool)` has no element 2 ``,
label `` its elements are numbered 0 through 1 ``. A non-tuple is `ORC0234`.
On `Int` the note is
`` `.k` selects element k of a value of a tuple type `(T, U, ...)` ``.
On `Int^2` the note is
`` an array's element is selected by an index, such as `x[0]` ``.

### §27. Byte Arrays (`Word[8]^n`)

1. There is no type `Byte`. A byte is a `Word[8]`. Naming `Byte` as a type,
   without a `type` alias, is `ORC0203`. `type Byte = Word[8];` is allowed,
   because `Byte` is not built in (§17).
2. String literals `"..."` and hexadecimal literals `hex"..."` have type
   `Word[8]^n`, where $n$ is the number of bytes.
3. **Concatenation Typing:**
   $$\frac{\Gamma \vdash A : T^a \quad \Gamma \vdash B : T^b \quad a + b \le 65,536}{\Gamma \vdash (A \mathbin{+\!+} B) : T^{a+b}}$$

### §28. Explicit Value Conversions (`expr as T`) and Typing Judgments

Orange strictly rejects implicit type coercions. Conversions MUST be explicit:

$$\frac{\Gamma \vdash e : \tau_{\text{src}} \quad \text{AdmissibleCast}(\tau_{\text{src}}, \tau_{\text{dst}})}{\Gamma \vdash (e \text{ as } \tau_{\text{dst}}) : \tau_{\text{dst}}}$$

#### Admissible Cast Table

| Source Type ($\tau_{\text{src}}$) | Target Type ($\tau_{\text{dst}}$) | Operational Semantic Meaning |
| :--- | :--- | :--- |
| `Word[W]` | `Int` | The residue in $[0, 2^W-1]$ as an `Int`. |
| `Int` | `Word[W]` | The residue of $x$ modulo $2^W$. Negative values wrap. The cast does not require $0 \le x < 2^W$. |
| `Word[W1]` | `Word[W2]` ($W_1 < W_2$) | Zero-extends. |
| `Word[W1]` | `Word[W2]` ($W_1 > W_2$) | Keeps the low $W_2$ bits. |
| `Int` | `Mod[m]` | Least residue of $x$ modulo $m$. |
| `Word[W]` | `Mod[m]` | Least residue of the word's unsigned value modulo $m$. |
| `Mod[m]` | `Int` | The least residue in $[0, m-1]$. |
| `Mod[m]` | `Word[W]` | The least residue, then reduced modulo $2^W$. |
| `Bool` | any scalar | Not a cast. `ORC0215`: `as` does not convert to or from `Bool`. |

Applying `as` to an untyped literal (e.g. `(42) as Word[32]`) emits diagnostic `ORC0220`.

### §29. Endianness Homomorphisms and Bit-Preserving Packing (`as big T`, `as little T`)

In cryptographic specifications, byte arrays are routinely repacked into words:

$$\frac{\Gamma \vdash e : \tau_{\text{src}} \quad \text{TotalBits}(\tau_{\text{src}}) = \text{TotalBits}(\tau_{\text{dst}})}{\Gamma \vdash (e \text{ as big } \tau_{\text{dst}}) : \tau_{\text{dst}}}$$

$$\frac{\Gamma \vdash e : \tau_{\text{src}} \quad \text{TotalBits}(\tau_{\text{src}}) = \text{TotalBits}(\tau_{\text{dst}})}{\Gamma \vdash (e \text{ as little } \tau_{\text{dst}}) : \tau_{\text{dst}}}$$

1. **Bit Invariance Requirement:**
   $$\text{TotalBits}(\tau_{\text{src}}) \ne \text{TotalBits}(\tau_{\text{dst}}) \implies \text{Diagnostic}(\text{ORC0240})$$
   An array of arrays is not a source or a target of `as`, `as big`, or `as little`,
   at any rank. The conversion stays a map on words and on rank-1 arrays of words.
2. **Packing Equations:**
   Let $B : \text{Word}[8]^k$ be an array of bytes converted to $W : \text{Word}[8k]$:
   - **Big-Endian:**
     $$W = \sum_{i=0}^{k-1} B[i] \cdot 2^{8(k - 1 - i)}$$
   - **Little-Endian:**
     $$W = \sum_{i=0}^{k-1} B[i] \cdot 2^{8i}$$

### §30. Dependent Finite Size Parameters and Monomorphization

1. A specification function may declare finite size parameters:

   ```orange
   spec pad[len in 1..64](msg: Word[8]^len) -> Word[8]^64 { ... }
   ```

2. **Finite Domain Invariant:**
   The range `low..high` is half-open. The bounds are integers satisfying
   $0 \le \text{low} < \text{high} \le 65,536$. The instance count
   $\text{high} - \text{low}$ is at least 1 and at most 256. An empty range,
   a bound above 65,536, or a count above 256 emits `ORC0238`.
3. **Eager Monomorphization:**
   The compiler specializes the function for every integer $k$ with
   $\text{low} \le k < \text{high}$. The spelling `n in 1..4` is the three
   instances $n = 1, 2, 3$, not $n = 4$. The example `len in 1..64` above is
   the lengths 1 through 63. Each instance is type-checked on its own.
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
   Maps static size parameter identifiers to half-open ranges `low..high` (§30).
3. **Type Parameter Context ($\Delta$):**
   $$\Delta : \text{TypeIdent} \to \mathcal{P}(\text{Types})$$
   Maps type parameter variables to their finite, explicitly admitted type candidate sets.
4. **Local Typing Context ($\Gamma$):**
   $$\Gamma : \text{VarIdent} \to \tau$$
   Maps local variable bindings and function parameters to their concrete types.

The four names are the manual's. The checker in
`compiler/crates/orange-compiler/src/semantics.rs` keeps them as separate
tables. A body is checked only after its function's instances exist.

#### Name Resolution

`BodyContext::resolve` tries a bare name in this order, and stops at the
first hit. Later tables are not consulted.

| Order | Table | Resolution | Type |
| :--- | :--- | :--- | :--- |
| 1 | the instance's size parameters ($\Theta$) | `Size` | `Int` |
| 2 | the function's parameters | `Parameter` | the parameter's type |
| 3 | `let` bindings of the body whose `;` has been passed, source order | `Binding` | the binding's type, or one element of a tuple pattern |
| 4 | bindings of the blocks being checked, innermost block first | `BlockBinding` | the same |
| 5 | loop indices and accumulators in scope, innermost loop first; the index is tried before the accumulator | `LoopIndex` or `Accumulator` | the index is `Int`; the accumulator has the `with` type |
| 6 | the spellings `true` and `false`, and only when no earlier table bound them | `BoolLiteral` | `Bool` |

A binding that exists later in the same body, but whose `;` has not been
passed, is `LaterBinding`. The diagnostic is `ORC0211`, message
`` `{name}` is used before it is bound ``, with the note that a binding is
in scope after its own `;`. A name that matches nothing is also `ORC0211`.
If the name is a finished block's binding, the message is
`` `{name}` is not in scope here ``. Otherwise it is
`` `{name}` is not a parameter or binding of `{function}` ``, or, when the
body has no bindings,
`` `{name}` is not a parameter of `{function}` ``. A type parameter used
where a value is required is the same code, with a note that the name is a
type. A `spec` name used without a call is the same code, with a note to
write the call.

#### No Shadowing

A parameter that repeats a name is `ORC0218`. A binding that repeats a size
parameter, a parameter, an earlier binding, or an earlier name of the same
pattern is `ORC0219`. The note on `ORC0219` is: each parameter and binding
of a function has its own name; Orange has no shadowing. A type parameter
names a type, not a value, so a parameter or a binding may use that
spelling. That exception is the comment on the binding check, not a second
value binding.

#### Signature Lookup

An unqualified call looks up a `spec` in the calling module. A call
`NAME::f` looks up `f` in a module this module `use`s. A qualifier that is
not such a module is `ORC0229`. A name that is not a typed `spec` is
`ORC0212`. Two `spec` declarations of one name in one module are `ORC0201`.
`spec` and `impl` are separate declaration namespaces, which is the note on
`ORC0201`. A typed body on `impl` does not reach that lookup: the parser
rejects the `->` with `ORC0101` (§55).

#### Worked Lookup

Take the instance $n = 1$ of

```orange
spec f[n in 1..3](x: Word[8]^n) -> Int {
  let y: Int = n;
  y
}
```

$\Theta$ holds $n \mapsto 1$. $\Gamma$ holds $x : \mathrm{Word}[8]^{1}$ before
the binding, and also $y : \mathrm{Int}$ after `y`'s `;`. The occurrence of
`n` in `let y` hits row 1 and has type `Int`. The occurrence of `y` in the
result hits row 3. An occurrence of `y` in the right-hand side of its own
binding is `LaterBinding` and is `ORC0211`. An occurrence of `n` as a type,
as in the parameter, is the size parameter of the instance, not a value
lookup.

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

$$\frac{n \in \mathbb{Z} \quad |n| < 2^{16384}}{\mathcal{C} \vdash n : \text{Int}} \quad (\text{T-Int-Lit})$$

$$\frac{|n| \ge 2^{16384}}{\mathcal{C} \vdash n : \text{Error}(\text{ORC0205})} \quad (\text{T-Int-Overflow})$$

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

$$\frac{\mathcal{C} \vdash a : \tau \quad \mathcal{C} \vdash b : \tau \quad \tau \in \{\text{Int}, \text{Word}[W]\}}{\mathcal{C} \vdash a \mathbin{\text{cmp}} b : \text{Bool}} \quad (\text{cmp} \in \{==, !=, <, <=, >, >=\}) \quad (\text{T-Rel})$$

$$\frac{\mathcal{C} \vdash a : \text{Mod}[m] \quad \mathcal{C} \vdash b : \text{Mod}[m]}{\mathcal{C} \vdash a \mathbin{\text{cmp}} b : \text{Bool}} \quad (\text{cmp} \in \{==, !=\}) \quad (\text{T-Rel-Mod})$$

An order comparison of residues (`<`, `<=`, `>`, `>=`) is `ORC0215`. Residues have no order. Compare least residues, as in `(x as Int) < (y as Int)`.

$$\frac{\mathcal{C} \vdash a : \tau_1 \quad \mathcal{C} \vdash b : \tau_2 \quad \tau_1 \not\equiv \tau_2}{\mathcal{C} \vdash a \mathbin{\text{cmp}} b : \text{Error}(\text{ORC0214})} \quad (\text{T-Rel-Mismatch})$$

$$\frac{a, b \text{ are untyped integer literals}}{\mathcal{C} \vdash a \mathbin{\text{cmp}} b : \text{Error}(\text{ORC0227})} \quad (\text{T-Rel-Untyped})$$

#### 7. Logical Connectives

$$\frac{\mathcal{C} \vdash a : \text{Bool} \quad \mathcal{C} \vdash b : \text{Bool}}{\mathcal{C} \vdash a \mathbin{\text{op}} b : \text{Bool}} \quad (\text{op} \in \{\&\&, ||\}) \quad (\text{T-Logic})$$

$$\frac{\mathcal{C} \vdash a : \text{Bool}}{\mathcal{C} \vdash !a : \text{Bool}} \quad (\text{T-Logic-Not})$$

#### 8. Local Bindings and Pattern Destructuring (§S3c)

$$\frac{\mathcal{C} \vdash e : \tau \quad x \notin \Gamma \quad \langle \Sigma, \Theta, \Delta, (\Gamma, x : \tau) \rangle \vdash \text{body} : \tau_{\text{body}}}{\mathcal{C} \vdash (\text{let } x : \tau = e; \ \text{body}) : \tau_{\text{body}}} \quad (\text{T-Let})$$

$$\frac{\mathcal{C} \vdash e : (T_0, \dots, T_{k-1}) \quad (\forall i.\ x_i \text{ is written } x_i : T_i)}{\mathcal{C} \vdash (\text{let } (x_0 : T_0, \dots, x_{k-1} : T_{k-1}) = e; \ \text{body}) : \tau_{\text{body}}} \quad (\text{T-Let-Tuple})$$

Every name in a `let` pattern carries a type, as in `let (sum: Word[64], carry: Word[64]) = add(x, y, c);`. A pattern that omits a type is a syntax error, not an inferred binding. An array is not a tuple: `let (a, b) = state` on a `Word[32]^8` is `ORC0214`.

$$\frac{x \in \Gamma}{\mathcal{C} \vdash (\text{let } x : \tau = e; \ \text{body}) : \text{Error}(\text{ORC0219})} \quad (\text{T-Let-Shadow})$$

#### 9. Conditionals (§S3f)

$$\frac{\mathcal{C} \vdash c : \text{Bool} \quad \mathcal{C} \vdash e_1 : \tau \quad \mathcal{C} \vdash e_2 : \tau}{\mathcal{C} \vdash (\text{if } c \ \{ e_1 \} \ \text{else } \{ e_2 \}) : \tau} \quad (\text{T-If})$$

$$\frac{\mathcal{C} \vdash c : \text{Bool} \quad \mathcal{C} \vdash e_1 : \tau_1 \quad \mathcal{C} \vdash e_2 : \tau_2 \quad \tau_1 \not\equiv \tau_2}{\mathcal{C} \vdash (\text{if } c \ \{ e_1 \} \ \text{else } \{ e_2 \}) : \text{Error}(\text{ORC0214})} \quad (\text{T-If-Mismatch})$$

#### 10. Array Construction, Indexing, Slicing, and Functional Update (§S3d, §S3e, §S3g)

$$\frac{\forall i \in [0, n-1].\ \mathcal{C} \vdash e_i : T \quad 1 \le n \le 65,536}{\mathcal{C} \vdash [e_0, e_1, \dots, e_{n-1}] : T^n} \quad (\text{T-Array-Lit})$$

$$\frac{\mathcal{C} \vdash v : T \quad 1 \le n \le 65,536}{\mathcal{C} \vdash [v; n] : T^n} \quad (\text{T-Array-Fill})$$

$$\frac{\mathcal{C} \vdash A : T^n \quad \mathcal{C} \vdash i : \text{Index}(n)}{\mathcal{C} \vdash A[i] : T} \quad (\text{T-Index})$$

$$\frac{\mathcal{C} \vdash A : T^n \quad \mathcal{C} \vdash i : \text{Index}(n) \quad \mathcal{C} \vdash v : T}{\mathcal{C} \vdash (A \text{ with } [i] = v) : T^n} \quad (\text{T-Update})$$

$$\frac{\operatorname{rank}(\tau) = r \in \{2, 3, 4\} \quad \mathcal{C} \vdash A : \tau \quad \forall j \in [1, r].\ \mathcal{C} \vdash i_j : \operatorname{Index}(\operatorname{axis}_j(\tau)) \quad \mathcal{C} \vdash v : \operatorname{leaf}(\tau)}{\mathcal{C} \vdash (A \text{ with } [i_1][i_2] \dots [i_r] = v) : \tau} \quad (\text{T-Update-Path})$$

A path with more indices than $\operatorname{rank}(\tau)$ is `ORC0224` (§25.5).
The value's type is the type at the end of the path, which is the leaf when
the path names every axis.

$$\frac{\mathcal{C} \vdash A : T^n \quad 0 \le l \le u \le n}{\mathcal{C} \vdash A[l..u] : T^{u - l}} \quad (\text{T-Slice})$$

$$\frac{\mathcal{C} \vdash A : T^n \quad 0 \le l \le u \le n \quad \mathcal{C} \vdash B : T^{u - l}}{\mathcal{C} \vdash (A \text{ with } [l..u] = B) : T^n} \quad (\text{T-Slice-Update})$$

$$\frac{\mathcal{C} \vdash A : T^a \quad \mathcal{C} \vdash B : T^b \quad a + b \le 65,536}{\mathcal{C} \vdash (A \mathbin{+\!+} B) : T^{a+b}} \quad (\text{T-Concat})$$

$$\frac{\mathcal{C} \vdash X : \tau \quad \tau \not\equiv T^n}{\mathcal{C} \vdash X[i] : \text{Error}(\text{ORC0224})} \quad (\text{T-Not-Array})$$

#### 11. Bounded Iteration Loops (§S3e)

$$\frac{\mathcal{C} \vdash \text{init} : \tau_{\text{acc}} \quad 0 \le \text{low} \le \text{high} \le 65,536 \quad \langle \Sigma, \Theta, \Delta, (\Gamma, i : \text{Int}, a : \tau_{\text{acc}}) \rangle \vdash \text{body} : \tau_{\text{acc}}}{\mathcal{C} \vdash (\text{for } i \text{ in } \text{low}..\text{high} \text{ with } a = \text{init} \ \{ \text{body} \}) : \tau_{\text{acc}}} \quad (\text{T-For})$$

$$\frac{\text{low} < 0 \lor \text{low} > \text{high} \lor \text{high} > 65,536}{\mathcal{C} \vdash (\text{for } i \text{ in } \text{low}..\text{high} \dots) : \text{Error}(\text{ORC0225})} \quad (\text{T-For-Range-Err})$$

#### 12. Function Application and Monomorphization

Let $f$ have declared signature $[n_1 \in \Theta_1, \dots][K_1 \in \Delta_1, \dots]\,(p_0 : \tau_0, \dots, p_{k-1} : \tau_{k-1}) \to \tau_{\text{ret}} \in \Sigma$:

$$\frac{\forall j.\ k_j \in \Theta_j \quad \forall m.\ U_m \in \Delta_m \quad \sigma = [\vec{n} \mapsto \vec{k}, \vec{K} \mapsto \vec{U}] \quad \forall i \in [0, k-1].\ \mathcal{C} \vdash e_i : \sigma(\tau_i)}{\mathcal{C} \vdash f[\vec{k}][\vec{U}]\,(e_0, \dots, e_{k-1}) : \sigma(\tau_{\text{ret}})} \quad (\text{T-App})$$

### §35. Expression Grouping Envelopes and Syntactic Ambiguity Rejection

The parser constructs expression ASTs according to the grouping envelope grammar:

$$\text{Group}(op_1) \ne \text{Group}(op_2) \implies \text{ParenRequirement}(op_1, op_2)$$

1. **What may share an expression without parentheses:**
   - `+` and `-` chain, left to right.
   - `*` chains, and it binds tighter than `+` and `-`, so `a + b * c` is one expression.
   - `&` chains only with `&`. `|` chains only with `|`. `^` chains only with `^`.
   - `&&` chains only with `&&`. `||` chains only with `||`.
   - `++` chains only with `++`.
2. **What takes exactly two operands:**
   Each of `/`, `%`, `<<`, `>>`, `<<<`, `>>>`, `==`, `!=`, `<`, `<=`, `>`, and `>=`
   takes two operands. A second one, or one of them beside an operator from
   another group, is `ORC0108`. In particular `a / b / c`, `a << 1 << 2`, and
   `a == b == c` are rejected, and `*` does not share a group with `/` or `%`.
3. **`as` and `with`:**
   A cast or an update is a postfix of its operand. Mixing one with a binary
   operator from another group, without parentheses, is `ORC0108`.

### §36. Static Index Ranges

Every index is proved in range before evaluation. The proof is the range of
the index expression, not a general interval lattice and not a dataflow
analysis over $\pm\infty$.

`docs/LOOKUPS_2026.md` is the lookup. The rules a reader applies are:

1. A `Word[n]` index ranges over its type, narrowed by the operator table in
   that document: `&` takes an upper bound from a mask, a conversion from a
   narrower word keeps a range that still fits, and an operator that can wrap
   ranges over the whole type.
2. An `Int` index is a static index: a literal, a loop index, a size name, or
   `a as Int` whose operand is a word, plus `+`, `-`, and parentheses. Its
   range is the range of that expression. `(x as Int)` for `x: Word[8]` is
   $[0, 255]$, so it indexes `Word[8]^256`.
3. A conditional's range runs from the least lower bound to the greatest upper
   bound of its branches. A condition does not narrow the index. `if x < 16`
   does not prove `t[x]` in range.
4. An index that is not inside $[0, n)$ is `ORC0223`, reported with the range
   the index would have had. There is no runtime bounds check in the `spec`
   evaluator.

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
3. If an index expression is not proved inside $[0, n)$, compilation rejects
   it with `ORC0223`. The `spec` evaluator does not check bounds at run time.

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

$$
\begin{array}{rcll}
v & ::= & c_{\text{Int}} & (\text{integers } n \in \mathbb{Z}) \\
  & \mid & c_{\text{Word}[W]} & (\text{ring elements } r \in [0, 2^W - 1]) \\
  & \mid & c_{\text{Mod}[m]} & (\text{residue elements } r \in [0, m - 1]) \\
  & \mid & \text{true} \mid \text{false} & (\text{booleans}) \\
  & \mid & [v_0, v_1, \dots, v_{n-1}] & (\text{arrays of length } n) \\
  & \mid & (v_0, v_1, \dots, v_{k-1}) & (\text{tuples of arity } k)
\end{array}
$$

#### 2. Evaluation Environment ($\rho$)

$$\rho \in \text{Env} = \text{Ident} \rightharpoonup \mathbb{V}$$

An immutable association mapping variable identifiers to semantic values.

#### 3. Deterministic Step Budget ($K$)

Dynamic execution is guarded by a step budget. The default is $K_0 = 1,048,576$.
`orangec eval --steps N` and `orangec test --steps N` admit $N$ from 1 through
1,073,741,824. Exhausting the budget emits `ORC0301`.

A step is not one primitive. An array update, fill, join, or slice of $n$
elements costs $\lceil n / 64 \rceil$ steps. A byte-order conversion costs one
step for each 64 bits. A shift or a rotation costs one step at any amount.
`docs/LENGTHS_2026.md`, `docs/ORDER_2026.md`, and `docs/AMOUNTS_2026.md` are
the cost tables.

### §40. Small-Step Operational Semantics (SOS) and Big-Step Reduction

We define the small-step evaluation relation over expressions:

$$e \longrightarrow e'$$

using evaluation contexts $E[\cdot]$ that formalize strict left-to-right evaluation order:

$$
\begin{array}{rcl}
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
\end{array}
$$

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
   A path denotes the nested updates of §25.6 and yields their value. Its step
   cost is the path cost of that section, not the cost of the selections in the
   expanded spelling. $\quad (\text{R-Update-Path})$
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
   - If $v = \text{true}$, `orangec test` prints `test "Title" ... ok`.
   - If $v = \text{false}$, it prints `test "Title" ... FAILED` and exits nonzero.
   - The expression must have type `Bool`. A failed step budget emits `ORC0301`
     and reports no test outcome.
2. **Whole-Aggregate Structural Equality (`==`):**
   Structural equality recursively compares compound elements:
   - For arrays $A, B : T^n$:
     $$A == B \iff \bigwedge_{i=0}^{n-1} (A[i] == B[i])$$
   - For tuples $A, B : (T_0, \dots, T_{k-1})$:
     $$A == B \iff \bigwedge_{j=0}^{k-1} (A.j == B.j)$$

---

## Part VI: Formal Metatheory of the Specification Stratum

This part states what the implemented `spec` evaluator guarantees. It is not a
machine-checked metatheory. There is no proof of strong normalization, subject
reduction, or an endianness isomorphism in this repository, and `orangec` does
not check one.

### §44. Evaluation Bounds

A closed, well-typed `spec` expression is evaluated under the step budget of
§39. The evaluator returns a value, or it stops with `ORC0301` when the budget
is exhausted, or it stops with `ORC0205` when an integer magnitude exceeds
16,384 significant bits.

Calls among typed `spec` functions are acyclic. A cycle is `ORC0217` and is
not evaluated. Module uses are acyclic (`ORC0230`, §16).

Those two budgets are the termination argument. They are not a proof that
every well-typed expression normalizes in unbounded arithmetic.

### §45. Checking Before Evaluation

`orangec` does not evaluate a program that has an error. A program that fails
a judgment in Parts III or IV produces diagnostics and no value. Progress and
subject reduction are not theorems of this manual.

The driver classifies each phase with `classify_phase_result` in
`compiler/crates/orangec/src/main.rs`. A nonempty diagnostic slice is
`Diagnosed`. An empty slice with an artifact is `Complete`. An empty slice
with no artifact is `Missing`, reported as `ORC1006`, and is not a value.
Semantic analysis is that classification of `analyze_program`. `Diagnosed`
sets the compilation failure and moves to the next input. The evaluator, the
test runner, and witness replay are reached only from `Complete`. A file that
failed lexing, parsing, or module loading never reaches analysis. One file's
diagnostics are not a value for that file, and they are not copied into
another file's result.

### §46. Determinism of the Reference Evaluator

The reference evaluator is a function of the source text, the step budget, and,
for `replay`, the witness file. The same inputs produce the same values, the
same diagnostics, and the same step count. `orangec` does not sample, does not
read a clock, and does not consult a package file.

`replay` prints `holds_for_this_witness` or `falsified`.

### §47. Byte-Order Conversions at Equal Width

`as big T` and `as little T` require the source and the target to have the
same number of bits. A mismatch is `ORC0240`. An array of arrays is neither a
source nor a target, at any rank. At equal width the two orders are the two
layouts of those bits, and each is the inverse of the other. The equations are
those of §29 and `docs/ORDER_2026.md`.

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

SHA-256 as FIPS 180-4 (NIST, August 2015, DOI 10.6028/NIST.FIPS.180-4) writes
it: the word operations of section 3.2, the functions of section 4.1.2, the
constants of section 4.2.2, the padding of section 5.1.1, the parsing of
section 5.2.1, the initial hash value of section 5.3.3, and the hash
computation of section 6.2.2. SHA-224 (sections 5.3.2 and 6.2.1), SHA-384,
SHA-512, SHA-512/224, and SHA-512/256 (sections 4.1.3, 4.2.3, 5.1.2, 5.2.2,
5.3.4 through 5.3.7, and 6.3 through 6.7) are other widths, rotation
distances, constant tables, and initial values. This section does not
restate them. `algorithms/sha2/sha2.or` writes that family and declares no
`test` member; `orangec test` on that file reports zero tests. The checked
text is the listing in this section.

#### Status

**Current.** Built `orangec` 0.0.1 from this tree reports `implemented slice
S3t` (`IMPLEMENTED_SLICE` in `compiler/crates/orangec/src/main.rs`).
`orangec test` on the listing accepts it: six tests, zero failures. The
listing uses `Word[32]`, fixed arrays, typed `let`, `for`/`with`, tuples,
byte strings, `hex"..."`, size parameters, and `as big`. Those are slices
S3b through S3n and S3q. It does not use rank-3 or rank-4 arrays. The
manual banner's "slice S3u" names OEP-0025, which is not a file in this
checkout and is not the slice this binary reports. Acceptance below is the
S3t compiler that was run.

| Text | Status | What is missing |
| :--- | :--- | :--- |
| The listing in this section, messages of 1 through 119 bytes, the empty message, and the worked words of section 6 | Current | Checked, as above |
| A message whose bit length is not a multiple of 8 | Not transcribed | FIPS 180-4 section 5.1.1 allows any $l < 2^{64}$. The listing's input type is `Word[8]^len`, so $l = 8 \cdot \mathrm{len}$. No compiler feature is missing; the domain is bytes |
| `len` of 120 bytes or more | Same algorithm, outside this size domain | `len in 1..120` is 119 instances. A size parameter has at most 256 instances (`ORC0238`, §30). A larger finite range is the same `pad` and `absorb` |
| SHA-224, SHA-384, SHA-512, SHA-512/224, SHA-512/256 | Not this section | Other FIPS sections, listed above. Not labeled Current here |

#### 1. Parameters (FIPS 180-4, sections 1 and 6.2)

| Quantity | SHA-256 |
| :--- | :--- |
| Word size | 32 bits, `Word[32]` |
| Block | 512 bits, sixteen words, `Word[8]^64` |
| Digest | 256 bits, eight words, `Word[8]^32` |
| Schedule length | 64 words |
| Length field | 64 bits, big-endian, and $l < 2^{64}$ |
| Round count | 64 |

#### 2. Logical Functions (sections 3.2 and 4.1.2)

Section 3.2, for a word $x$ of $w = 32$ bits and an integer $n$ with
$0 \le n < w$:

$$\mathrm{ROTR}^{n}(x) = (x \gg n) \lor (x \ll (w - n))$$

$$\mathrm{SHR}^{n}(x) = x \gg n$$

`>>>` is $\mathrm{ROTR}$ and `>>` is $\mathrm{SHR}$ (§48). The amounts in
section 4.1.2 are constants in that range, so they are the constant shifts
of slice S3b. Section 4.1.2, for $x, y, z \in \mathbb{Z}/2^{32}\mathbb{Z}$,
with $\land$ bitwise AND, $\lor$ bitwise OR, $\oplus$ exclusive-or, and
$\lnot$ bitwise complement:

$$\mathrm{Ch}(x, y, z) = (x \land y) \oplus (\lnot x \land z)$$

$$\mathrm{Maj}(x, y, z) = (x \land y) \oplus (x \land z) \oplus (y \land z)$$

$$\Sigma_0^{\{256\}}(x) = \mathrm{ROTR}^{2}(x) \oplus \mathrm{ROTR}^{13}(x) \oplus \mathrm{ROTR}^{22}(x)$$

$$\Sigma_1^{\{256\}}(x) = \mathrm{ROTR}^{6}(x) \oplus \mathrm{ROTR}^{11}(x) \oplus \mathrm{ROTR}^{25}(x)$$

$$\sigma_0^{\{256\}}(x) = \mathrm{ROTR}^{7}(x) \oplus \mathrm{ROTR}^{18}(x) \oplus \mathrm{SHR}^{3}(x)$$

$$\sigma_1^{\{256\}}(x) = \mathrm{ROTR}^{17}(x) \oplus \mathrm{ROTR}^{19}(x) \oplus \mathrm{SHR}^{10}(x)$$

| FIPS name | Orange | Amounts |
| :--- | :--- | :--- |
| $\mathrm{Ch}$ | `ch` | — |
| $\mathrm{Maj}$ | `maj` | — |
| $\Sigma_0^{\{256\}}$ | `big_sigma0` | 2, 13, 22 |
| $\Sigma_1^{\{256\}}$ | `big_sigma1` | 6, 11, 25 |
| $\sigma_0^{\{256\}}$ | `small_sigma0` | 7, 18, and a shift of 3 |
| $\sigma_1^{\{256\}}$ | `small_sigma1` | 17, 19, and a shift of 10 |

Addition in the schedule and in the round is addition in `Word[32]`, the
standard's $\bmod 2^{32}$. `+` on `Word[32]` is that operation. There is no
separate modulo in the listing.

#### 3. Constants and the Initial Hash Value (sections 4.2.2 and 5.3.3)

$K_0^{\{256\}}$ through $K_{63}^{\{256\}}$ are the first 32 bits of the
fractional parts of the cube roots of the first sixty-four primes, in the
order section 4.2.2 prints them. `round_constants` is that sequence.
$H^{(0)}$ is the first 32 bits of the fractional parts of the square roots of
the first eight primes. `initial_hash` is that sequence:

| $i$ | $H_i^{(0)}$ |
| :--- | :--- |
| 0 | `0x6a09e667` |
| 1 | `0xbb67ae85` |
| 2 | `0x3c6ef372` |
| 3 | `0xa54ff53a` |
| 4 | `0x510e527f` |
| 5 | `0x9b05688c` |
| 6 | `0x1f83d9ab` |
| 7 | `0x5be0cd19` |

Section 4.2.2 prints $K_t^{\{256\}}$ for $t = 0$ through $t = 63$ in that
order. `round_constants()[t]` is $K_t^{\{256\}}$. The words are the first 32
bits of the fractional parts of the cube roots of the first sixty-four
primes, which the standard lists as these values:

| $t$ | $K_t^{\{256\}}$ | $t$ | $K_t^{\{256\}}$ | $t$ | $K_t^{\{256\}}$ | $t$ | $K_t^{\{256\}}$ |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| 0 | `0x428a2f98` | 1 | `0x71374491` | 2 | `0xb5c0fbcf` | 3 | `0xe9b5dba5` |
| 4 | `0x3956c25b` | 5 | `0x59f111f1` | 6 | `0x923f82a4` | 7 | `0xab1c5ed5` |
| 8 | `0xd807aa98` | 9 | `0x12835b01` | 10 | `0x243185be` | 11 | `0x550c7dc3` |
| 12 | `0x72be5d74` | 13 | `0x80deb1fe` | 14 | `0x9bdc06a7` | 15 | `0xc19bf174` |
| 16 | `0xe49b69c1` | 17 | `0xefbe4786` | 18 | `0x0fc19dc6` | 19 | `0x240ca1cc` |
| 20 | `0x2de92c6f` | 21 | `0x4a7484aa` | 22 | `0x5cb0a9dc` | 23 | `0x76f988da` |
| 24 | `0x983e5152` | 25 | `0xa831c66d` | 26 | `0xb00327c8` | 27 | `0xbf597fc7` |
| 28 | `0xc6e00bf3` | 29 | `0xd5a79147` | 30 | `0x06ca6351` | 31 | `0x14292967` |
| 32 | `0x27b70a85` | 33 | `0x2e1b2138` | 34 | `0x4d2c6dfc` | 35 | `0x53380d13` |
| 36 | `0x650a7354` | 37 | `0x766a0abb` | 38 | `0x81c2c92e` | 39 | `0x92722c85` |
| 40 | `0xa2bfe8a1` | 41 | `0xa81a664b` | 42 | `0xc24b8b70` | 43 | `0xc76c51a3` |
| 44 | `0xd192e819` | 45 | `0xd6990624` | 46 | `0xf40e3585` | 47 | `0x106aa070` |
| 48 | `0x19a4c116` | 49 | `0x1e376c08` | 50 | `0x2748774c` | 51 | `0x34b0bcb5` |
| 52 | `0x391c0cb3` | 53 | `0x4ed8aa4a` | 54 | `0x5b9cca4f` | 55 | `0x682e6ff3` |
| 56 | `0x748f82ee` | 57 | `0x78a5636f` | 58 | `0x84c87814` | 59 | `0x8cc70208` |
| 60 | `0x90befffa` | 61 | `0xa4506ceb` | 62 | `0xbef9a3f7` | 63 | `0xc67178f2` |

#### 4. Padding, Parsing, and Byte Order (sections 3.1, 5.1.1, and 5.2.1)

Section 3.1 reads a hex digit as four bits, most significant bit first, and a
word as the integer whose rightmost hex digit is the least significant four
bits. Section 5.2.1 then splits each 512-bit block into sixteen 32-bit words.
The first 32 bits of block $i$ are $M_0^{(i)}$ and the last 32 bits are
$M_{15}^{(i)}$. That is big-endian packing (§29):

$$M_t^{(i)} = \sum_{j=0}^{3} B[64(i-1) + 4t + j] \cdot 2^{8(3-j)}$$

where $B$ is the padded message as bytes and block numbers in the standard
start at 1. `block as big Word[32]^16` is those sixteen words. The digest
uses the same map in the other direction: `hash as big Word[8]^32` is
$H_0^{(N)} \mathbin{\Vert} \cdots \mathbin{\Vert} H_7^{(N)}$.

For a message of $l$ bits, $l < 2^{64}$, section 5.1.1 appends a single 1
bit, then the least $k \ge 0$ zero bits such that

$$l + 1 + k \equiv 448 \pmod{512},$$

then the 64-bit big-endian encoding of $l$. The padded length is a multiple
of 512. The listing takes whole bytes, $l = 8 \cdot \mathrm{len}$. The bit 1
is then the byte `0x80`, because the first bit of a byte is its most
significant bit. The 64-bit length is `(8 * len) as big Word[8]^8`, eight
bytes, most significant byte first. `length_bytes` is the same map written
as shifts, used for the empty message where there is no `len` to cast.

For every $\mathrm{len}$ from 0 through 119, the FIPS block count equals
$((\mathrm{len} + 8) / 64) + 1$ under truncating integer division, and `pad`
returns that many blocks. `len in 1..120` is the half-open range of §30, so
the instances are lengths 1 through 119. Those lengths use one block for
$\mathrm{len} \le 55$ and two blocks for $56 \le \mathrm{len} \le 119$.
`absorb`'s parameter `blocks in 1..4` is the three instances 1, 2, and 3, which
covers both. Length 0 is `sha256_empty`: an array type `Word[8]^n` requires
$n \ge 1$, so there is no `Word[8]^0` to pass to `pad`. The empty block is
the byte `0x80`, 55 zero bytes, and a 64-bit length of zero. A message whose
length is not a multiple of 8 bits, and a message of 120 bytes or more, are
outside this listing, as the status table records.

#### 5. The Hash Computation (section 6.2.2)

For each block $i = 1, \ldots, N$, with working variables $(a, b, c, d, e, f, g, h)$:

1. $W_t = M_t^{(i)}$ for $0 \le t \le 15$, and for $16 \le t \le 63$

$$W_t = \sigma_1^{\{256\}}(W_{t-2}) + W_{t-7} + \sigma_0^{\{256\}}(W_{t-15}) + W_{t-16} \pmod{2^{32}}.$$

2. Initialize $(a, \ldots, h)$ from $H^{(i-1)}$.
3. For $t = 0$ to $63$,

$$T_1 = h + \Sigma_1^{\{256\}}(e) + \mathrm{Ch}(e, f, g) + K_t^{\{256\}} + W_t,$$

$$T_2 = \Sigma_0^{\{256\}}(a) + \mathrm{Maj}(a, b, c),$$

and $(a, b, c, d, e, f, g, h) \leftarrow (T_1 + T_2,\ a,\ b,\ c,\ d + T_1,\ e,\ f,\ g)$,
each sum modulo $2^{32}$.

4. $H_j^{(i)} = a_j + H_j^{(i-1)}$, where $a_0, \ldots, a_7$ are the final
working variables. The digest is $H_0^{(N)} \mathbin{\Vert} \cdots \mathbin{\Vert} H_7^{(N)}$,
big-endian.

`schedule` is step 1, in that addend order. `round` is step 3. `compress` is
steps 2 through 4. `absorb` is the loop "for $i = 1$ to $N$" and the final
big-endian bytes. The standard's step 3 assigns the eight working variables
one after another, each right-hand side using the values from before the
round. The listing returns those eight values at once. The correspondence
is:

| FIPS 180-4 section 6.2.2 | Orange |
| :--- | :--- |
| "For $i = 1$ to $N$" | `absorb`: `for b in 0..blocks`, and block $i$ is `p[64*(i-1) .. 64*(i-1)+64]` |
| Step 1, $0 \le t \le 15$, $W_t = M_t^{(i)}$ | `schedule`: `let head = block as big Word[32]^16`, then `head ++ [0; 48]` |
| Step 1, $16 \le t \le 63$ | the loop `for t in 16..64`, index `t`, sum `small_sigma1(w[t - 2]) + w[t - 7] + small_sigma0(w[t - 15]) + w[t - 16]` |
| Step 2, $a = H_0^{(i-1)}, \ldots, h = H_7^{(i-1)}$ | the `with` tuple of `compress`, `(hash[0], ..., hash[7])` |
| Step 3, $T_1 = h + \Sigma_1^{\{256\}}(e) + \mathrm{Ch}(e, f, g) + K_t^{\{256\}} + W_t$ | `t1` in `round`, addend order unchanged, `k` is `round_constants()[t]`, `w` is `schedule(block)[t]` |
| Step 3, $T_2 = \Sigma_0^{\{256\}}(a) + \mathrm{Maj}(a, b, c)$ | `t2` |
| Step 3, $a \leftarrow T_1 + T_2$ | result component 0 |
| Step 3, $b \leftarrow a$, $c \leftarrow b$, $d \leftarrow c$ | result components 1, 2, 3, the values from before the round |
| Step 3, $e \leftarrow d + T_1$ | result component 4 |
| Step 3, $f \leftarrow e$, $g \leftarrow f$, $h \leftarrow g$ | result components 5, 6, 7 |
| Step 3, $t = 0$ to $63$ | `for t in 0..64` |
| Step 4, $H_j^{(i)} = a_j + H_j^{(i-1)}$ | `[a + hash[0], ..., h + hash[7]]`, where `hash` is $H^{(i-1)}$ and was not updated inside the round loop |
| The digest $H_0^{(N)} \mathbin{\Vert} \cdots \mathbin{\Vert} H_7^{(N)}$ | `absorb`'s `hash as big Word[8]^32` |

The range `0..64` is half-open (§30), so the body runs for $t = 0, 1, \ldots, 63$.
The range `16..64` is $t = 16, \ldots, 63$.

#### 6. Worked Values for the NIST Examples

The one-block example is the message `abc` ($l = 24$). Its padded block begins
`61 62 63 80` and ends with the length word `0x00000018`. So $W_0 = \mathtt{0x61626380}$
and $W_{15} = \mathtt{0x00000018}$. Round $t = 0$ starts from $H^{(0)}$ with
$K_0 = \mathtt{0x428a2f98}$.

| Name | Value |
| :--- | :--- |
| $\Sigma_1(e)$ | `0x3587272b` |
| $\mathrm{Ch}(e, f, g)$ | `0x1f85c98c` |
| $T_1$ | `0x54da50e8` |
| $\Sigma_0(a)$ | `0xce20b47e` |
| $\mathrm{Maj}(a, b, c)$ | `0x3a6fe667` |
| $T_2$ | `0x08909ae5` |
| $a$ after the round | `0x5d6aebcd` |
| $e$ after the round | `0xfa2a4622` |

$b, c, d$ after the round are the old $a, b, c$, and $f, g, h$ are the old
$e, f, g$. The test `abc round 0` checks $T_1$, $T_2$, and that 8-tuple.

$W_{16} = W_0$ on this message, because $W_1$ through $W_{14}$ are zero.
$W_{17} = \sigma_1(W_{15}) = \sigma_1(\mathtt{0x00000018})$. The three rotations
and the shift are $\mathrm{ROTR}^{17} = \mathtt{0x000c0000}$,
$\mathrm{ROTR}^{19} = \mathtt{0x00030000}$, $\mathrm{SHR}^{10} = 0$, and their
exclusive-or is $\mathtt{0x000f0000}$. The test `abc schedule` checks that word.

The 56-byte example is

`abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq`

($l = 448$). The bit 1 falls in the first block and the length falls in the
second, whose last word is `0x000001c0`. After the first block,

$$H^{(1)} = \mathtt{85e655d6\ 417a1795\ 3363376a\ 624cde5c\ 76e09589\ cac5f811\ cc4b32c1\ f20e533a}.$$

Its schedule word $W_{16}$ is $\mathtt{0xeb8012ad}$, the sum modulo $2^{32}$ of
$\sigma_1(W_{14}) = \mathtt{0x00205000}$, $W_9 = \mathtt{0x6a6b6c6d}$,
$\sigma_0(W_1) = \mathtt{0x1f91f2dc}$, and $W_0 = \mathtt{0x61626364}$.

#### 7. Known Answers

| Message | Digest |
| :--- | :--- |
| empty, $l = 0$ | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| `abc` | `ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad` |
| the 56-byte message | `248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1` |

These are the digests of the NIST examples that accompany FIPS 180-4 (the
one-block and two-block messages) and of the $l = 0$ padding of section 5.1.1.
The listing checks them as `Word[8]^32`.

#### 8. Compiler-Checked Transcription

```orange
// FIPS 180-4 SHA-256, sections 4.1.2, 4.2.2, 5.1.1, 5.2.1, 5.3.3, and 6.2.2,
// as the standard writes them. Messages of 1 through 119 bytes are one size
// parameter. The empty message is a separate block: a Word[8] array cannot
// have length 0. A message whose bit length is not a multiple of 8 is not
// represented.
edition 2026;
module sha256_spec {
  spec ch(x: Word[32], y: Word[32], z: Word[32]) -> Word[32] { (x & y) ^ (~x & z) }
  spec maj(x: Word[32], y: Word[32], z: Word[32]) -> Word[32] { (x & y) ^ (x & z) ^ (y & z) }
  spec big_sigma0(x: Word[32]) -> Word[32] { (x >>> 2) ^ (x >>> 13) ^ (x >>> 22) }
  spec big_sigma1(x: Word[32]) -> Word[32] { (x >>> 6) ^ (x >>> 11) ^ (x >>> 25) }
  spec small_sigma0(x: Word[32]) -> Word[32] { (x >>> 7) ^ (x >>> 18) ^ (x >> 3) }
  spec small_sigma1(x: Word[32]) -> Word[32] { (x >>> 17) ^ (x >>> 19) ^ (x >> 10) }

  // Section 4.2.2: the first 32 bits of the fractional parts of the cube
  // roots of the first 64 primes, in the order the standard prints them.
  spec round_constants() -> Word[32]^64 {
    [
      0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
      0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
      0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
      0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
      0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
      0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
      0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
      0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
    ]
  }

  // Section 5.3.3.
  spec initial_hash() -> Word[32]^8 {
    [
      0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a,
      0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
    ]
  }

  // Section 6.2.2, step 1. The sum is the standard's order:
  // sigma1(W[t-2]) + W[t-7] + sigma0(W[t-15]) + W[t-16].
  spec schedule(block: Word[8]^64) -> Word[32]^64 {
    let head: Word[32]^16 = block as big Word[32]^16;
    for t in 16..64 with w: Word[32]^64 = head ++ [0; 48] {
      w with [t] = small_sigma1(w[t - 2]) + w[t - 7] + small_sigma0(w[t - 15]) + w[t - 16]
    }
  }

  // Section 6.2.2, step 3, with the standard's names a through h, T1, and T2.
  spec round(
    a: Word[32], b: Word[32], c: Word[32], d: Word[32],
    e: Word[32], f: Word[32], g: Word[32], h: Word[32],
    k: Word[32], w: Word[32],
  ) -> (Word[32], Word[32], Word[32], Word[32], Word[32], Word[32], Word[32], Word[32]) {
    let t1: Word[32] = h + big_sigma1(e) + ch(e, f, g) + k + w;
    let t2: Word[32] = big_sigma0(a) + maj(a, b, c);
    (t1 + t2, a, b, c, d + t1, e, f, g)
  }

  // Section 6.2.2, steps 2 through 4.
  spec compress(hash: Word[32]^8, block: Word[8]^64) -> Word[32]^8 {
    let w: Word[32]^64 = schedule(block);
    let k: Word[32]^64 = round_constants();
    let (a: Word[32], b: Word[32], c: Word[32], d: Word[32],
         e: Word[32], f: Word[32], g: Word[32], h: Word[32]) =
      for t in 0..64 with (a: Word[32], b: Word[32], c: Word[32], d: Word[32],
                           e: Word[32], f: Word[32], g: Word[32], h: Word[32]) =
        (hash[0], hash[1], hash[2], hash[3], hash[4], hash[5], hash[6], hash[7]) {
        round(a, b, c, d, e, f, g, h, k[t], w[t])
      };
    [
      a + hash[0], b + hash[1], c + hash[2], d + hash[3],
      e + hash[4], f + hash[5], g + hash[6], h + hash[7],
    ]
  }

  spec absorb[blocks in 1..4](p: Word[8]^(64 * blocks)) -> Word[8]^32 {
    let hash: Word[32]^8 = for b in 0..blocks with h: Word[32]^8 = initial_hash() {
      compress(h, p[64 * b..64 * b + 64])
    };
    hash as big Word[8]^32
  }

  // Section 5.1.1 for a whole-byte message of `len` bytes, 1 through 119.
  // The block count ((len + 8) / 64) + 1 is the smallest number of 512-bit
  // blocks whose last 64 bits can hold the length after the bit 1.
  spec pad[len in 1..120](m: Word[8]^len) -> Word[8]^(64 * (((len + 8) / 64) + 1)) {
    m ++ ([0; ((64 * (((len + 8) / 64) + 1)) - len - 8)] with [0] = 0x80)
      ++ ((8 * len) as big Word[8]^8)
  }

  spec sha256[len in 1..120](m: Word[8]^len) -> Word[8]^32 { absorb(pad(m)) }

  // Section 5.1.1 for l = 0. The block is the byte 0x80, 55 zero bytes, and
  // a 64-bit length of zero. There is no Word[8]^0 to pass to `pad`.
  spec length_bytes(bits: Word[64]) -> Word[8]^8 {
    [
      (bits >> 56) as Word[8], (bits >> 48) as Word[8], (bits >> 40) as Word[8], (bits >> 32) as Word[8],
      (bits >> 24) as Word[8], (bits >> 16) as Word[8], (bits >> 8) as Word[8], bits as Word[8],
    ]
  }

  spec sha256_empty() -> Word[8]^32 {
    let head: Word[8]^56 = [0; 56];
    let block: Word[8]^64 = (head with [0] = 0x80) ++ length_bytes(0);
    compress(initial_hash(), block) as big Word[8]^32
  }

  spec two_block_message() -> Word[8]^56 {
    "abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq"
  }

  // Round t = 0 of the one-block example "abc": H(0), K0, and W0 = 0x61626380.
  spec abc_round0() -> (Word[32], Word[32], Word[32], Word[32], Word[32], Word[32], Word[32], Word[32]) {
    let h: Word[32]^8 = initial_hash();
    let w0: Word[32] = schedule(pad("abc"))[0];
    round(h[0], h[1], h[2], h[3], h[4], h[5], h[6], h[7], round_constants()[0], w0)
  }

  spec abc_t1() -> Word[32] {
    let h: Word[32]^8 = initial_hash();
    h[7] + big_sigma1(h[4]) + ch(h[4], h[5], h[6]) + round_constants()[0] + schedule(pad("abc"))[0]
  }

  spec abc_t2() -> Word[32] {
    let h: Word[32]^8 = initial_hash();
    big_sigma0(h[0]) + maj(h[0], h[1], h[2])
  }

  // W17 of "abc" is sigma1 of the length word 0x00000018. The other three
  // summands are zero.
  spec abc_w17() -> Word[32] { schedule(pad("abc"))[17] }

  // The first block of the 56-byte example, after its 64 rounds, before the
  // second block is absorbed. Section 6.2.2's H(1).
  spec two_block_h1() -> Word[32]^8 {
    let padded: Word[8]^128 = pad(two_block_message());
    compress(initial_hash(), padded[0..64])
  }

  spec two_block_w16() -> Word[32] {
    let padded: Word[8]^128 = pad(two_block_message());
    schedule(padded[0..64])[16]
  }

  test "NIST example: SHA-256 of abc" {
    sha256("abc") == hex"ba7816bf 8f01cfea 414140de 5dae2223 b00361a3 96177a9c b410ff61 f20015ad"
  }

  test "NIST example: SHA-256 of the 56-byte message" {
    sha256(two_block_message())
      == hex"248d6a61 d20638b8 e5c02693 0c3e6039 a33ce459 64ff2167 f6ecedd4 19db06c1"
  }

  test "section 5.1.1: SHA-256 of the empty message" {
    sha256_empty() == hex"e3b0c442 98fc1c14 9afbf4c8 996fb924 27ae41e4 649b934c a495991b 7852b855"
  }

  test "abc round 0: T1, T2, and the working variables" {
    let h: Word[32]^8 = initial_hash();
    (abc_t1() == 0x54da50e8) && (abc_t2() == 0x08909ae5)
      && (abc_round0() == (
        0x5d6aebcd, h[0], h[1], h[2], 0xfa2a4622, h[4], h[5], h[6]
      ))
  }

  test "abc schedule: W17 is sigma1 of the length word" {
    (abc_w17() == 0x000f0000) && (abc_w17() == small_sigma1(0x00000018))
  }

  test "56-byte example: H1 and W16 of the first block" {
    (two_block_h1() == [
      0x85e655d6, 0x417a1795, 0x3363376a, 0x624cde5c,
      0x76e09589, 0xcac5f811, 0xcc4b32c1, 0xf20e533a,
    ]) && (two_block_w16() == 0xeb8012ad)
  }
}
```

### §50. Complete Reference Specification: RFC 8439 ChaCha20

ChaCha20 as RFC 8439 (2018) writes it: the quarter round of section 2.1, the
quarter round on the state of section 2.2, the block function of section 2.3,
and the encryption function of section 2.4. Bernstein's original ChaCha, with
a 64-bit counter and a 64-bit nonce, is not this RFC. XChaCha20
(draft-irtf-cfrg-xchacha) is not this section.

#### Status

**Current** for the listing in this section. `orangec test` on that listing,
with the S3t binary described in §49, accepts four tests and fails none:
section 2.1.1 with its eight intermediate words, section 2.3.2, appendix A.1
item 1, and the section 2.4 identity that a 64-byte block of zero plaintext
equals the key stream. The listing uses `Word[32]`, tuples, fixed arrays,
`for`/`with`, byte strings, `hex"..."`, and `as little`. It does not use a
size parameter. A message whose length is not 64 bytes is not a parameter of
`encrypt_block`.

`algorithms/chacha20/chacha20.or` is the same quarter round and block
function, plus encryption at the concrete lengths the RFC's examples use.
That file declares no `test` member: `orangec test` on it reports zero tests.
`orangec eval` of each `rfc8439_*` spec below equals the matching
`rfc8439_*_expected` spec. Fourteen pairs, zero mismatches. The XChaCha20
specs in that file were not part of the comparison and are not Current here.

| Text | Status | What is missing |
| :--- | :--- | :--- |
| The listing: quarter round, one block, 64-byte XOR | Current | The four tests above |
| Section 2.2.1, appendix A.1 items 2 through 5, appendix A.2, section 2.4.2 | Checked by `orangec eval` of `algorithms/chacha20/chacha20.or` | Not a `test` in this listing. The bytes are that file's `*_expected` specs, not copied here |
| A length other than the lengths that file defines | Same algorithm, another array length | Each length is its own `Word[8]^n`. The file defines 64, 114, 119, 127, and 256 |
| XChaCha20 | Not this section | A different draft. Not labeled Current here |

#### 1. The Quarter Round (section 2.1)

On four words $a, b, c, d \in \mathbb{Z}/2^{32}\mathbb{Z}$ the RFC assigns, in
order:

$$a \leftarrow a + b,\quad d \leftarrow (d \oplus a) \lll 16,$$

$$c \leftarrow c + d,\quad b \leftarrow (b \oplus c) \lll 12,$$

$$a \leftarrow a + b,\quad d \leftarrow (d \oplus a) \lll 8,$$

$$c \leftarrow c + d,\quad b \leftarrow (b \oplus c) \lll 7.$$

Addition is `Word[32]` addition. `<<<` is the left rotation. The listing names
the eight results $a_1, d_1, c_1, b_1, a_2, d_2, c_2, b_2$ and returns
$(a_2, b_2, c_2, d_2)$.

Section 2.1.1 starts from $a = \mathtt{0x11111111}$, $b = \mathtt{0x01020304}$,
$c = \mathtt{0x9b8d6f43}$, $d = \mathtt{0x01234567}$.

| Name | Value |
| :--- | :--- |
| $a_1$ | `0x12131415` |
| $d_1$ | `0x51721330` |
| $c_1$ | `0xecff8273` |
| $b_1$ | `0xd8177edf` |
| $a_2$ | `0xea2a92f4` |
| $d_2$ | `0x5881c4bb` |
| $c_2$ | `0x4581472e` |
| $b_2$ | `0xcb1cf8ce` |

The returned quarter round is $(a_2, b_2, c_2, d_2) = (\mathtt{0xea2a92f4}, \mathtt{0xcb1cf8ce}, \mathtt{0x4581472e}, \mathtt{0x5881c4bb})$, the vector section 2.1.1 prints.

The RFC writes the quarter round as eight statements. Each right-hand side
uses the value the previous statement stored. The listing binds those eight
results and returns the final four, in the order $(a, b, c, d)$:

| RFC 8439 section 2.1 | Orange in `quarter_round` |
| :--- | :--- |
| `a += b` | `a1 = a + b` |
| `d ^= a; d <<<= 16` | `d1 = (d ^ a1) <<< 16` |
| `c += d` | `c1 = c + d1` |
| `b ^= c; b <<<= 12` | `b1 = (b ^ c1) <<< 12` |
| `a += b` | `a2 = a1 + b1` |
| `d ^= a; d <<<= 8` | `d2 = (d1 ^ a2) <<< 8` |
| `c += d` | `c2 = c1 + d2` |
| `b ^= c; b <<<= 7` | `b2 = (b1 ^ c2) <<< 7` |
| the four final words | `(a2, b2, c2, d2)` |

`+` is addition in `Word[32]`, the RFC's 32-bit wrap. `<<<` is the RFC's
`<<<=`. `^` is the RFC's `^=`. `example_211_steps` is the same eight bindings
on the section 2.1.1 inputs, so the test checks the intermediate words and
not only the returned tuple. Section 2.2.1 applies one quarter round,
`QUARTERROUND(2, 7, 8, 13)`, to a sample state and changes only those four
positions. That state is `rfc8439_2_2_1` in `algorithms/chacha20/chacha20.or`.
The four results the RFC prints are `0xbdb886dc`, `0xcfacafd2`, `0xe46bea80`,
and `0xccc07c79`, written back at indices 2, 7, 8, and 13.

#### 2. The Block Function (sections 2.2 and 2.3)

The 16-word state is laid out as the RFC's matrix. Words 0 through 3 are the
constants for the string "expand 32-byte k":

| Index | Word |
| :--- | :--- |
| 0 | `0x61707865` |
| 1 | `0x3320646e` |
| 2 | `0x79622d32` |
| 3 | `0x6b206574` |

Words 4 through 11 are the 256-bit key as eight little-endian words. Word 12
is the 32-bit block counter. Words 13 through 15 are the 96-bit nonce as three
little-endian words. Bernstein's ChaCha kept a 64-bit counter and a 64-bit
nonce; RFC 8439 is the 32-bit counter and the 96-bit nonce.

One inner block is a column round and then a diagonal round. The quarter-round
index tuples are:

| Round | Index tuples |
| :--- | :--- |
| Column | $(0, 4, 8, 12)$, $(1, 5, 9, 13)$, $(2, 6, 10, 14)$, $(3, 7, 11, 15)$ |
| Diagonal | $(0, 5, 10, 15)$, $(1, 6, 11, 12)$, $(2, 7, 8, 13)$, $(3, 4, 9, 14)$ |

The block function runs that pair ten times (20 rounds), adds the initial
state word by word, and serializes the 16 words as little-endian bytes.
`block` is that function. A little-endian word of four bytes
$B[0], B[1], B[2], B[3]$ is the §29 sum

$$W = B[0] + 256\,B[1] + 256^{2}\,B[2] + 256^{3}\,B[3].$$

The listing builds the 64-byte input and loads it with one cast:

`"expand 32-byte k" ++ key ++ (counter as little Word[8]^4) ++ nonce`,

then `as little Word[32]^16`. The 16 ASCII bytes of `"expand 32-byte k"` are
the four constants: little-endian loads of `expa`, `nd 3`, `2-by`, and `te k`
are `0x61707865`, `0x3320646e`, `0x79622d32`, and `0x6b206574`. The key's
first four bytes are word 4, the next four are word 5, and so on through
word 11. The counter is word 12. The nonce's three little-endian words are
words 13, 14, and 15.

| RFC 8439 section 2.3 | Orange in `block` |
| :--- | :--- |
| `state = constants \| key \| counter \| nonce` | the `initial` array, loaded as above |
| `working_state = state` | the `with` tuple of the `for round in 0..10` loop |
| `for i = 1 upto 10` | `for round in 0..10`, ten iterations |
| column `QUARTERROUND` on $(0,4,8,12)$, $(1,5,9,13)$, $(2,6,10,14)$, $(3,7,11,15)$ | the four `quarter_round` calls bound to `c0`..`c15` |
| diagonal `QUARTERROUND` on $(0,5,10,15)$, $(1,6,11,12)$, $(2,7,8,13)$, $(3,4,9,14)$ | the four calls bound to `d0`..`d15` |
| `inner_block` returns the state in index order | the 16-tuple `(d0, d1, ..., d15)` |
| `state[i] += working_state[i]` for each $i$ | `for i in 0..16`, `out[i] + initial[i]`, where `initial` was not modified |
| `serialize`, little-endian | `state as little Word[8]^64` |

Section 2.3.2's key, counter 1, and 12-byte nonce, and appendix A.1 item 1
(the all-zero key, counter, and nonce at block 0), are the tests in the
listing. After the ten double-rounds and the word-wise sum, section 2.3.2's
state words are the array `rfc8439_2_3_2_state_expected` in
`algorithms/chacha20/chacha20.or`, beginning `0xe4e7f110`, `0x15593bd1`,
`0x1fdd0f50`, `0xc47120a3`. The serialized block is the 64 bytes in the
listing's section 2.3.2 test.

#### 3. Encryption (section 2.4)

Section 2.4's `chacha20_encrypt` walks the plaintext in 64-byte blocks.
For each $j$ from 0 through $\lfloor \mathrm{len}/64 \rfloor - 1$, the
keystream block is `chacha20_block(key, counter + j, nonce)` and the
ciphertext block is the plaintext block XOR that keystream. A leftover of
$r = \mathrm{len} \bmod 64$ bytes, $r \ne 0$, uses the next block at counter
$+ \lfloor \mathrm{len}/64 \rfloor$ and keeps the first $r$ ciphertext bytes.
`+` on the counter is `Word[32]` addition. The RFC requires the caller not to
reuse a block counter under one key and nonce; the listing does not add a
separate check.

`encrypt_block` is that XOR for one 64-byte block, the $j = 0$ case when
$\mathrm{len} = 64$. The listing checks it on a zero plaintext, which equals
the key stream. The other example lengths live in
`algorithms/chacha20/chacha20.or` and were the `orangec eval` pairs in the
status table:

| RFC 8439 | Inputs | Spec |
| :--- | :--- | :--- |
| A.1 item 1 | key 0, counter 0, nonce 0 | `rfc8439_a1_1`, also the listing |
| A.1 item 2 | key 0, counter 1, nonce 0 | `rfc8439_a1_2` |
| A.1 item 3 | key byte 31 is 1, counter 1, nonce 0 | `rfc8439_a1_3` |
| A.1 item 4 | key byte 1 is `0xff`, counter 2, nonce 0 | `rfc8439_a1_4` |
| A.1 item 5 | key 0, counter 0, nonce byte 11 is 2 | `rfc8439_a1_5` |
| A.2 item 1 | 64 zero bytes, key 0, counter 0, nonce 0 | `rfc8439_a2_1` |
| 2.4.2 | 114-byte sunscreen text, key `00..1f`, counter 1, nonce `000000000000004a00000000` | `rfc8439_2_4_2` via `encrypt_114` |
| A.2 item 2 | 375-byte text, key byte 31 is 1, counter 1, nonce byte 11 is 2 | `rfc8439_a2_2_head` (bytes 0..255, counters 1..4) and `rfc8439_a2_2_tail` (bytes 256..374, counter 5) |
| A.2 item 3 | 127-byte text, the key printed in A.2, counter 42, nonce byte 11 is 2 | `rfc8439_a2_3` via `encrypt_127` |

The 375-byte message is two calls because that file's `encrypt` is written
at length 256 and the tail at length 119. The language array bound is
65,536 (§24), so the split is the file's, not a type-system limit. The
serialized keystream and ciphertext bytes are the `*_expected` specs in that
file.

#### 4. Compiler-Checked Transcription

```orange
// RFC 8439 ChaCha20: the quarter round (section 2.1), the block function
// (section 2.3), and the keystream tests of sections 2.1.1, 2.3.2, and
// appendix A.1 item 1. Encryption (section 2.4) is the XOR of that keystream;
// a message of several blocks is the same block at counter + j, which a
// single array length does not range over here.
edition 2026;
module chacha20_spec {
  type Quad = (Word[32], Word[32], Word[32], Word[32]);

  // Section 2.1. The names are the RFC's successive assignments:
  // a += b; d ^= a; d <<<= 16; c += d; b ^= c; b <<<= 12;
  // a += b; d ^= a; d <<<= 8;  c += d; b ^= c; b <<<= 7.
  spec quarter_round(a: Word[32], b: Word[32], c: Word[32], d: Word[32]) -> Quad {
    let a1: Word[32] = a + b;
    let d1: Word[32] = (d ^ a1) <<< 16;
    let c1: Word[32] = c + d1;
    let b1: Word[32] = (b ^ c1) <<< 12;
    let a2: Word[32] = a1 + b1;
    let d2: Word[32] = (d1 ^ a2) <<< 8;
    let c2: Word[32] = c1 + d2;
    let b2: Word[32] = (b1 ^ c2) <<< 7;
    (a2, b2, c2, d2)
  }

  // Section 2.1.1, the eight words the example assigns, in order.
  spec example_211_steps() -> (Word[32], Word[32], Word[32], Word[32], Word[32], Word[32], Word[32], Word[32]) {
    let a: Word[32] = 0x11111111;
    let b: Word[32] = 0x01020304;
    let c: Word[32] = 0x9b8d6f43;
    let d: Word[32] = 0x01234567;
    let a1: Word[32] = a + b;
    let d1: Word[32] = (d ^ a1) <<< 16;
    let c1: Word[32] = c + d1;
    let b1: Word[32] = (b ^ c1) <<< 12;
    let a2: Word[32] = a1 + b1;
    let d2: Word[32] = (d1 ^ a2) <<< 8;
    let c2: Word[32] = c1 + d2;
    let b2: Word[32] = (b1 ^ c2) <<< 7;
    (a1, d1, c1, b1, a2, d2, c2, b2)
  }

  // Section 2.3. Ten iterations of a column round and a diagonal round, then
  // the word-wise sum with the initial state, serialized little-endian.
  // Column indices are (0, 4, 8, 12), (1, 5, 9, 13), (2, 6, 10, 14),
  // (3, 7, 11, 15). Diagonal indices are (0, 5, 10, 15), (1, 6, 11, 12),
  // (2, 7, 8, 13), (3, 4, 9, 14).
  spec block(key: Word[8]^32, counter: Word[32], nonce: Word[8]^12) -> Word[8]^64 {
    let initial: Word[32]^16 =
      ("expand 32-byte k" ++ key ++ (counter as little Word[8]^4) ++ nonce) as little Word[32]^16;
    let (x0: Word[32], x1: Word[32], x2: Word[32], x3: Word[32],
         x4: Word[32], x5: Word[32], x6: Word[32], x7: Word[32],
         x8: Word[32], x9: Word[32], x10: Word[32], x11: Word[32],
         x12: Word[32], x13: Word[32], x14: Word[32], x15: Word[32]) =
      for round in 0..10 with (
        s0: Word[32], s1: Word[32], s2: Word[32], s3: Word[32],
        s4: Word[32], s5: Word[32], s6: Word[32], s7: Word[32],
        s8: Word[32], s9: Word[32], s10: Word[32], s11: Word[32],
        s12: Word[32], s13: Word[32], s14: Word[32], s15: Word[32]
      ) = (
        initial[0], initial[1], initial[2], initial[3],
        initial[4], initial[5], initial[6], initial[7],
        initial[8], initial[9], initial[10], initial[11],
        initial[12], initial[13], initial[14], initial[15]
      ) {
        let (c0: Word[32], c4: Word[32], c8: Word[32], c12: Word[32]) = quarter_round(s0, s4, s8, s12);
        let (c1: Word[32], c5: Word[32], c9: Word[32], c13: Word[32]) = quarter_round(s1, s5, s9, s13);
        let (c2: Word[32], c6: Word[32], c10: Word[32], c14: Word[32]) = quarter_round(s2, s6, s10, s14);
        let (c3: Word[32], c7: Word[32], c11: Word[32], c15: Word[32]) = quarter_round(s3, s7, s11, s15);
        let (d0: Word[32], d5: Word[32], d10: Word[32], d15: Word[32]) = quarter_round(c0, c5, c10, c15);
        let (d1: Word[32], d6: Word[32], d11: Word[32], d12: Word[32]) = quarter_round(c1, c6, c11, c12);
        let (d2: Word[32], d7: Word[32], d8: Word[32], d13: Word[32]) = quarter_round(c2, c7, c8, c13);
        let (d3: Word[32], d4: Word[32], d9: Word[32], d14: Word[32]) = quarter_round(c3, c4, c9, c14);
        (d0, d1, d2, d3, d4, d5, d6, d7, d8, d9, d10, d11, d12, d13, d14, d15)
      };
    let working: Word[32]^16 = [
      x0, x1, x2, x3, x4, x5, x6, x7, x8, x9, x10, x11, x12, x13, x14, x15,
    ];
    let state: Word[32]^16 =
      for i in 0..16 with out: Word[32]^16 = working { out with [i] = out[i] + initial[i] };
    state as little Word[8]^64
  }

  // Section 2.4 for one 64-byte block: ciphertext byte i is plaintext byte i
  // XOR keystream byte i. A longer message repeats `block` at counter + j.
  spec encrypt_block(key: Word[8]^32, counter: Word[32], nonce: Word[8]^12, plaintext: Word[8]^64) -> Word[8]^64 {
    let key_stream: Word[8]^64 = block(key, counter, nonce);
    for i in 0..64 with c: Word[8]^64 = plaintext { c with [i] = plaintext[i] ^ key_stream[i] }
  }

  test "RFC 8439 section 2.1.1 quarter round" {
    let steps: (Word[32], Word[32], Word[32], Word[32], Word[32], Word[32], Word[32], Word[32]) =
      example_211_steps();
    (quarter_round(0x11111111, 0x01020304, 0x9b8d6f43, 0x01234567)
      == (0xea2a92f4, 0xcb1cf8ce, 0x4581472e, 0x5881c4bb))
      && (steps == (
        0x12131415, 0x51721330, 0xecff8273, 0xd8177edf,
        0xea2a92f4, 0x5881c4bb, 0x4581472e, 0xcb1cf8ce
      ))
  }

  test "RFC 8439 section 2.3.2 block function" {
    let key: Word[8]^32 =
      hex"00010203 04050607 08090a0b 0c0d0e0f 10111213 14151617 18191a1b 1c1d1e1f";
    let serialized: Word[8]^64 =
      hex"10 f1 e7 e4 d1 3b 59 15 50 0f dd 1f a3 20 71 c4" ++
        hex"c7 d1 f4 c7 33 c0 68 03 04 22 aa 9a c3 d4 6c 4e" ++
        hex"d2 82 64 46 07 9f aa 09 14 c2 d7 05 d9 8b 02 a2" ++
        hex"b5 12 9c d1 de 16 4e b9 cb d0 83 e8 a2 50 3c 4e";
    block(key, 1, hex"00 00 00 09 00 00 00 4a 00 00 00 00") == serialized
  }

  test "RFC 8439 appendix A.1 item 1 zero key stream" {
    block([0; 32], 0, [0; 12]) == (
      hex"76 b8 e0 ad a0 f1 3d 90 40 5d 6a e5 53 86 bd 28" ++
        hex"bd d2 19 b8 a0 8d ed 1a a8 36 ef cc 8b 77 0d c7" ++
        hex"da 41 59 7c 51 57 48 8d 77 24 e0 3f b8 d8 4a 37" ++
        hex"6a 43 b8 f4 15 18 a1 1c c3 87 b6 69 b2 ee 65 86"
    )
  }

  test "section 2.4: one block of zeros is the key stream" {
    encrypt_block([0; 32], 0, [0; 12], [0; 64]) == block([0; 32], 0, [0; 12])
  }
}
```

### §51. Complete Reference Specification: Curve25519 / X25519 (RFC 7748)

X25519 as RFC 7748 (2016) section 5 writes it, with the Diffie-Hellman
functions of section 6.1. The field is the integers modulo

$$p = 2^{255} - 19,$$

which is the integer `prime()` prints,
$57896044618658097711785492504343953926634992332820282019728792003956564819949$.
Section 4.1 sets $A = 486662$. Section 5 sets $a_{24} = (A - 2) / 4 = 121665$.
The listing uses exact `Int`. A product or a square is reduced by `%` at
once. `Int` remainder is Euclidean (`divide_euclid` in
`compiler/crates/orange-compiler/src/eval.rs`), so `x % prime()` is the
residue in $[0, p - 1]$ when `x` is negative. Sums and differences inside a
product are not reduced on their own; the product's `%` is the field
operation.

#### Status

**Current** for the listing in this section. `orangec test` on that listing,
with the S3t binary of §49, accepts the first vector of section 5.2 and fails
none. One `X25519` evaluation costs about 565,000 of the 1,048,576 steps in
`MAX_EVALUATION_STEPS_PER_SOURCE` (`compiler/crates/orange-compiler/src/eval.rs`).
A file that evaluates two of them exceeds that budget and the evaluator
reports `ORC0301`. That is why each further vector is its own file.
`algorithms/x25519/x25519.or` is the same text as the listing, without a
`test` member. `orangec check` accepts that file. The other three files are
the same algorithm with a different vector spec.

| Text | Status | Check |
| :--- | :--- | :--- |
| Listing, section 5.2 vector 1 | Current | `orangec test`: 1 passed |
| `algorithms/x25519/x25519-second-vector.or`, section 5.2 vector 2 | Checked by `orangec eval` | `rfc7748_5_2_vector_2` equals `rfc7748_5_2_vector_2_expected`, 565,070 steps |
| `algorithms/x25519/x25519-diffie-hellman.or`, section 6.1 shared secret | Checked by `orangec eval` | `rfc7748_6_1_shared_secret` equals its `_expected` spec, 565,257 steps. This run does not also evaluate $K_A = \mathrm{X25519}(a, 9)$ |
| `algorithms/x25519/x25519-wycheproof.or`, Wycheproof tcId 1 | Checked by `orangec eval` | `wycheproof_x25519_tc_1` equals its `_expected` spec, 564,808 steps. Not an RFC 7748 vector |
| `all_zero` on a computed shared secret | Not evaluated | A second X25519 in the same file exceeds 1,048,576 steps (`ORC0301`) |
| Section 5.2 iterated test: after 1, after 1,000, and after 1,000,000 iterations, starting from the 32-byte string whose first byte is 9 | Not transcribed | The one-iteration output is one X25519 call and would fit the budget. 1,000 and 1,000,000 calls do not, under the default 1,048,576 steps. No listing in the tree evaluates them |
| X448 (sections 5 and 6.2) | Not this section | 56-byte scalars, $p = 2^{448} - 2^{224} - 1$, $a_{24} = 39081$. Not labeled Current here |
| The RFC's arithmetic `cswap` mask | Not this listing | The listing swaps with `if`. A specification has no timing. An `impl` mask is Proposed (Part VIII) |

#### 1. Decoding (section 5)

`decode_little_endian` is the RFC's `decodeLittleEndian` on 32 bytes.
The RFC sums $b[i] \cdot 256^{i}$. The listing folds from the high byte,

$$n \leftarrow 256 \cdot n + b[31 - i],$$

for $i = 0, \ldots, 31$, which is the same integer. `decode_u_coordinate` is
`decodeUCoordinate` for 255 bits: the unused bit is the most significant bit
of the last byte, cleared by `u[31] & 127`, which is the RFC's mask
$(1 \ll (255 \bmod 8)) - 1$. `decode_scalar_25519` is `decodeScalar25519`'s
clamping, kept as bytes rather than returned as an integer: byte 0 becomes
`k[0] & 248` (clear bits 0, 1, and 2), and byte 31 becomes
`(k[31] & 127) | 64` (clear bit 255, set bit 254). The ladder reads that
integer one bit at a time. Bit $t$ is bit $t \bmod 8$ of byte
$\lfloor t / 8 \rfloor$, tested by `(scalar[t / 8] & bit[t % 8]) != 0` with
`bit = [1, 2, 4, 8, 16, 32, 64, 128]`. `encode_u_coordinate` is
`encodeUCoordinate`: the base-256 digits of $u \bmod p$, least significant
byte first.

#### 2. The Ladder Step (section 5)

With $s = (x_2, z_2, x_3, z_3)$ and $x_1$ the u-coordinate, the RFC's names
in lowercase are:

$$A = x_2 + z_2,\ AA = A^2,\ B = x_2 - z_2,\ BB = B^2,\ E = AA - BB,$$

$$C = x_3 + z_3,\ D = x_3 - z_3,\ DA = D \cdot A,\ CB = C \cdot B,$$

$$x_2' = AA \cdot BB,\quad z_2' = E \cdot (AA + a_{24} \cdot E),$$

$$x_3' = (DA + CB)^2,\quad z_3' = x_1 \cdot (DA - CB)^2.$$

Each product is reduced modulo $p$. `ladder_step` returns $(x_2', z_2', x_3', z_3')$.

The RFC's `cswap` uses an arithmetic mask. A specification has no timing, so
`cswap` is the conditional that exchanges $(x_2, z_2)$ with $(x_3, z_3)$. The
permutation is the RFC's. The mask is not what this listing executes. The
RFC's loop, for $t = 254, 253, \ldots, 0$, is `for i in 0..255` with bit
index $254 - i$ (255 iterations; `0..255` is half-open). The initial state is
$(x_2, z_2, x_3, z_3) = (1, 0, x_1, 1)$, the `with` value of that loop.

| RFC 7748 section 5 | Orange |
| :--- | :--- |
| `k_t = (k >> t) & 1` | `(scalar[(254 - i) / 8] & bit[(254 - i) % 8]) != 0` |
| `swap ^= k_t` before the step, `swap = k_t` after it, and the `cswap` after the loop | `rung`: `cswap(k_t, ladder_step(x_1, cswap(k_t, s)))`. The file states that those three RFC swaps amount to swapping the two pairs around the step whenever $k_t$ is set. The section 5.2 vectors check the output of that form |
| `A = x_2 + z_2`, `AA = A^2` | `a`, `aa = square(a)` |
| `B = x_2 - z_2`, `BB = B^2`, `E = AA - BB` | `b`, `bb`, `e` |
| `C = x_3 + z_3`, `D = x_3 - z_3` | `c`, `d` |
| `DA = D * A`, `CB = C * B` | `da`, `cb` |
| `x_2 = AA * BB` | result index 0 |
| `z_2 = E * (AA + a24 * E)` | result index 1 |
| `x_3 = (DA + CB)^2` | result index 2 |
| `z_3 = x_1 * (DA - CB)^2` | result index 3 |
| `encodeUCoordinate(x_2 * (z_2^(p - 2)), 255)` | `encode_u_coordinate(multiply(s[0], invert(s[1])))` |

#### 3. The Inverse and the Output

The RFC writes the output as $x_2 \cdot z_2^{(p - 2)}$ and leaves the power
open. `invert` is Bernstein's curve25519 addition chain (2006): 254 squarings
and 11 multiplications. It is not a second algorithm in the RFC; it is one
way to compute the power the RFC names. The names record the power.
`square_times(x, n)` is $x^{(2^{n})}$ for $0 \le n \le 100$, by $n$ squarings
inside a loop of 100 steps (the extra iterations keep the value). The chain's
exponents, with $z_{2^{k}-1}$ abbreviated $z(2^{k}-1)$, are:

| Binding | Exponent of $z$ |
| :--- | :--- |
| `z_2` | $2$ |
| `z_9` | $9$ |
| `z_11` | $11$ |
| `z_2_5_0` | $2^{5} - 1$ |
| `z_2_10_0` | $2^{10} - 1$ |
| `z_2_20_0` | $2^{20} - 1$ |
| `z_2_40_0` | $2^{40} - 1$ |
| `z_2_50_0` | $2^{50} - 1$ |
| `z_2_100_0` | $2^{100} - 1$ |
| `z_2_200_0` | $2^{200} - 1$ |
| `z_2_250_0` | $2^{250} - 1$ |
| `multiply(square_times(z_2_250_0, 5), z_11)` | $(2^{250} - 1) \cdot 2^{5} + 11 = 2^{255} - 21$ |

$p - 2 = 2^{255} - 21$, so the last binding is $z^{(p-2)}$.

`x25519` decodes the scalar and the u-coordinate, runs the ladder from bit
254 down to bit 0 starting at $(x_2 : z_2) = (1 : 0)$ and $(x_3 : z_3) = (x_1 : 1)$,
and encodes $x_2 \cdot z_2^{(p-2)}$ as 32 little-endian bytes.
`encode_u_coordinate` is those base-256 digits. `public_key` and
`shared_secret` are section 6.1: $K_A = \mathrm{X25519}(a, 9)$ and
$K = \mathrm{X25519}(a, K_B)$. The base point is the byte 9 followed by 31
zero bytes. `all_zero` is the optional all-zero check of section 6.1. The
test does not evaluate it on a computed shared secret, because that would be
a second X25519 evaluation in the same file.

#### 4. Section 5.2, First Vector

The input scalar, the input u-coordinate, and the output u-coordinate, each
as the RFC prints the 32 bytes:

| Role | Bytes |
| :--- | :--- |
| Scalar | `a546e36bf0527c9d3b16154b82465edd62144c0ac1fc5a18506a2244ba449ac4` |
| u-coordinate | `e6db6867583030db3594c1a424b15f7c726624ec26b3353b10a903a6d0ab1c4c` |
| Output | `c3da55379de9c6908e94ea4df28d084f32eccf03491c71f754b4075577a28552` |

Section 5.2's second vector, checked by `orangec eval` of
`algorithms/x25519/x25519-second-vector.or` and not by this listing's `test`:

| Role | Bytes |
| :--- | :--- |
| Scalar | `4b66e9d4d1b4673c5ad22691957d6af5c11b6421e0ea01d42ca4169e7918ba0d` |
| u-coordinate | `e5210f12786811d3f4b7959d0538ae2c31dbe7106fc03c3efc4cd549c715a493` |
| Output | `95cbde9476e8907d7aade45cb4b873f88b595a68799fa152e6f8f7647aac7957` |

Section 6.1's shared secret, checked the same way on
`algorithms/x25519/x25519-diffie-hellman.or`. The inputs are Alice's secret
$a$ and Bob's public key $K_B$ as that file records them. The output is $K$.

| Role | Bytes |
| :--- | :--- |
| $a$ | `77076d0a7318a57d3c16c17251b26645df4c2f87ebc0992ab177fba51db92c2a` |
| $K_B$ | `de9edb7d7b7dc1b4d35b61c2ece435373f8343c85b78674dadfc7e146f882b4f` |
| $K$ | `4a5d9d5ba4ce2de1728e3bf480350f25e07e21c947d19e3376f09b3c1e161742` |

Section 6.1 also prints Alice's public key $\mathrm{X25519}(a, 9)$ as
`8520f0098930a754748b7ddcb43ef75a0dbf3a0d26381af4eba4a98eaa9b4e6a`, Bob's
secret $b$ as
`5dab087e624a8a4b79e17f8b83800ee66f3bb1292618b6fd1c2f8b27ff88e0eb`, and
states $K_B = \mathrm{X25519}(b, 9)$. Those three calls were not evaluated
in this run. The clamping of section 5 makes the decoded scalar
$2^{254} + 8 \cdot n$ for an integer $n$ with $0 \le n \le 2^{251} - 1$.

#### 5. Compiler-Checked Transcription

```orange
// X25519 of RFC 7748, 'Elliptic Curves for Security' (2016), section 5,
// with the Diffie-Hellman functions of section 6.1. The field is the
// integers modulo p = 2^255 - 19, written with exact `Int` arithmetic and
// `%` after every product; the scalar's bits are read through indices that
// divide a loop index; each rung of the Montgomery ladder is a conditional
// swap around the RFC's formulas; and the inverse z_2^(p - 2) is computed
// by the addition chain of Bernstein's curve25519 reference.
// https://www.rfc-editor.org/rfc/rfc7748
//
// This file reproduces the first test vector of section 5.2.
// One X25519 evaluation costs about 568,000 of the 1,048,576 steps of a file,
// so each vector file holds one vector. The algorithm part of x25519.or,
// x25519-second-vector.or, x25519-diffie-hellman.or and
// x25519-wycheproof.or is the same text; only the vector specs differ.
edition 2026;
module x25519_spec {
  // Section 4.1: p = 2^255 - 19 and A = 486662; section 5: a24 = (A - 2) / 4.
  spec prime() -> Int {
    57896044618658097711785492504343953926634992332820282019728792003956564819949
  }

  spec a24() -> Int { 121665 }

  // The field GF(p). A product is reduced by `%` at once, so every field
  // element stays below p between the steps of the ladder.
  spec multiply(x: Int, y: Int) -> Int { (x * y) % prime() }

  spec square(x: Int) -> Int { (x * x) % prime() }

  // Section 5: decodeLittleEndian(b, 255).
  spec decode_little_endian(b: Word[8]^32) -> Int {
    for i in 0..32 with n: Int = 0 { n * 256 + (b[31 - i] as Int) }
  }

  // Section 5: decodeUCoordinate. The most significant bit of the final
  // byte is masked, as the RFC requires of every implementation.
  spec decode_u_coordinate(u: Word[8]^32) -> Int {
    decode_little_endian(u with [31] = u[31] & 127)
  }

  // Section 5: decodeScalar25519, the clamping. The three low bits are
  // cleared, bit 255 is cleared, and bit 254 is set. The clamped bytes are
  // kept as bytes, because the ladder reads the scalar one bit at a time.
  spec decode_scalar_25519(k: Word[8]^32) -> Word[8]^32 {
    (k with [0] = k[0] & 248) with [31] = (k[31] & 127) | 64
  }

  // Section 5: encodeUCoordinate, the 32 little-endian bytes of u mod p.
  // The bytes are the base-256 digits of u, least significant first; the
  // accumulator carries the value still to be split at index 0 and the
  // digits found so far after it, all as `Int`, and a second loop makes
  // them bytes.
  spec encode_u_coordinate(u: Int) -> Word[8]^32 {
    let digits: Int^33 = for i in 0..32 with d: Int^33 = [0; 33] with [0] = u % prime() {
      (d with [1 + i] = d[0] % 256) with [0] = d[0] / 256
    };
    for i in 0..32 with b: Word[8]^32 = [0; 32] { b with [i] = digits[1 + i] as Word[8] }
  }

  // Section 5: the body of the ladder's loop, with s = [x_2, z_2, x_3, z_3]
  // and the RFC's names A, AA, B, BB, E, C, D, DA, CB in lowercase. It
  // doubles (x_2 : z_2) and adds it to (x_3 : z_3), whose difference is
  // (x_1 : 1).
  spec ladder_step(x_1: Int, s: Int^4) -> Int^4 {
    let a: Int = s[0] + s[1];
    let aa: Int = square(a);
    let b: Int = s[0] - s[1];
    let bb: Int = square(b);
    let e: Int = aa - bb;
    let c: Int = s[2] + s[3];
    let d: Int = s[2] - s[3];
    let da: Int = multiply(d, a);
    let cb: Int = multiply(c, b);
    [
      multiply(aa, bb),
      multiply(e, aa + a24() * e),
      square(da + cb),
      multiply(x_1, square(da - cb)),
    ]
  }

  // Section 5: cswap(swap, x_2, x_3) and cswap(swap, z_2, z_3) together.
  // The RFC swaps in constant time through a mask; a specification has no
  // timing, so the swap is a conditional between the two arrangements.
  spec cswap(swap: Bool, s: Int^4) -> Int^4 {
    if swap { [s[2], s[3], s[0], s[1]] } else { s }
  }

  // Section 5: one iteration of the ladder for the scalar bit k_t. The
  // RFC's `swap ^= k_t` before the step and `swap = k_t` after it, with the
  // final cswap after the loop, amount to swapping the two points around
  // the step whenever k_t is set.
  spec rung(x_1: Int, s: Int^4, k_t: Bool) -> Int^4 {
    cswap(k_t, ladder_step(x_1, cswap(k_t, s)))
  }

  // x^(2^n) modulo p for 0 <= n <= 100: n successive squarings.
  spec square_times(x: Int, n: Int) -> Int {
    for i in 0..100 with y: Int = x { if i < n { square(y) } else { y } }
  }

  // Section 5: z_2^(p - 2), the inverse of z_2 by Fermat's little theorem.
  // The RFC writes the power and leaves its computation open; this is the
  // addition chain of Bernstein's curve25519 reference implementation
  // (2006), 254 squarings and 11 multiplications, which reaches
  // p - 2 = 2^255 - 21 at about half the cost of square-and-multiply over
  // the 255 bits of p - 2 (59,000 steps against 107,000). Each name says
  // which power of z it holds: z_2_10_0 is z^(2^10 - 2^0).
  spec invert(z: Int) -> Int {
    let z_2: Int = square(z);
    let z_9: Int = multiply(square_times(z_2, 2), z);
    let z_11: Int = multiply(z_9, z_2);
    let z_2_5_0: Int = multiply(square(z_11), z_9);
    let z_2_10_0: Int = multiply(square_times(z_2_5_0, 5), z_2_5_0);
    let z_2_20_0: Int = multiply(square_times(z_2_10_0, 10), z_2_10_0);
    let z_2_40_0: Int = multiply(square_times(z_2_20_0, 20), z_2_20_0);
    let z_2_50_0: Int = multiply(square_times(z_2_40_0, 10), z_2_10_0);
    let z_2_100_0: Int = multiply(square_times(z_2_50_0, 50), z_2_50_0);
    let z_2_200_0: Int = multiply(square_times(z_2_100_0, 100), z_2_100_0);
    let z_2_250_0: Int = multiply(square_times(z_2_200_0, 50), z_2_50_0);
    multiply(square_times(z_2_250_0, 5), z_11)
  }

  // Section 5: X25519(k, u). The clamped scalar's bits k_t, for t = 254
  // down to 0, drive the ladder from (x_2 : z_2) = (1 : 0) and
  // (x_3 : z_3) = (x_1 : 1); the result is x_2 * z_2^(p - 2), encoded.
  // Bit t of the scalar is bit t % 8 of byte t / 8.
  spec x25519(k: Word[8]^32, u: Word[8]^32) -> Word[8]^32 {
    let scalar: Word[8]^32 = decode_scalar_25519(k);
    let bit: Word[8]^8 = [1, 2, 4, 8, 16, 32, 64, 128];
    let x_1: Int = decode_u_coordinate(u);
    let s: Int^4 = for i in 0..255 with s: Int^4 = [1, 0, x_1, 1] {
      rung(x_1, s, (scalar[(254 - i) / 8] & bit[(254 - i) % 8]) != 0)
    };
    encode_u_coordinate(multiply(s[0], invert(s[1])))
  }

  // Section 6.1: the base point, u = 9, as the 32-byte string the RFC
  // gives, 9 followed by all zeros.
  spec base_point() -> Word[8]^32 {
    [
      0x09, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
      0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    ]
  }

  // Section 6.1: Alice's public key K_A = X25519(a, 9) from her secret key
  // a, 32 random bytes; Bob's K_B = X25519(b, 9) likewise.
  spec public_key(a: Word[8]^32) -> Word[8]^32 { x25519(a, base_point()) }

  // Section 6.1: the shared secret K = X25519(a, K_B) = X25519(b, K_A).
  spec shared_secret(a: Word[8]^32, k_b: Word[8]^32) -> Word[8]^32 { x25519(a, k_b) }

  // Section 6.1: both parties MAY check whether K is the all-zero value and
  // abort if so, which rejects a public key of small order. The check is a
  // Bool over the 32 bytes; none of the vector files evaluates it on a
  // computed K, because that would cost a second X25519 evaluation.
  spec all_zero(k: Word[8]^32) -> Bool {
    for i in 0..32 with zero: Bool = true { zero && (k[i] == 0) }
  }

  // RFC 7748 section 5.2, the first test vector: input scalar, input
  // u-coordinate, output u-coordinate.
  spec rfc7748_5_2_vector_1() -> Word[8]^32 {
    x25519(
      [
        0xa5, 0x46, 0xe3, 0x6b, 0xf0, 0x52, 0x7c, 0x9d, 0x3b, 0x16, 0x15, 0x4b, 0x82, 0x46, 0x5e, 0xdd,
        0x62, 0x14, 0x4c, 0x0a, 0xc1, 0xfc, 0x5a, 0x18, 0x50, 0x6a, 0x22, 0x44, 0xba, 0x44, 0x9a, 0xc4,
      ],
      [
        0xe6, 0xdb, 0x68, 0x67, 0x58, 0x30, 0x30, 0xdb, 0x35, 0x94, 0xc1, 0xa4, 0x24, 0xb1, 0x5f, 0x7c,
        0x72, 0x66, 0x24, 0xec, 0x26, 0xb3, 0x35, 0x3b, 0x10, 0xa9, 0x03, 0xa6, 0xd0, 0xab, 0x1c, 0x4c,
      ],
    )
  }

  spec rfc7748_5_2_vector_1_expected() -> Word[8]^32 {
    [
      0xc3, 0xda, 0x55, 0x37, 0x9d, 0xe9, 0xc6, 0x90, 0x8e, 0x94, 0xea, 0x4d, 0xf2, 0x8d, 0x08, 0x4f,
      0x32, 0xec, 0xcf, 0x03, 0x49, 0x1c, 0x71, 0xf7, 0x54, 0xb4, 0x07, 0x55, 0x77, 0xa2, 0x85, 0x52,
    ]
  }

  test "RFC 7748 section 5.2 first vector" {
    rfc7748_5_2_vector_1() == rfc7748_5_2_vector_1_expected()
  }
}
```

### §52. Complete Reference Specification: Poly1305 Field MAC (RFC 8439)

Poly1305 as RFC 8439 section 2.5 writes it. Section 2.8's AEAD construction,
which feeds Poly1305 a padded string of additional data, ciphertext, and
lengths, is the same `mac` on that string. The AEAD section transcribes that
construction. This section is the field MAC.

#### Status

**Current** for the listing in this section. `orangec test` on that listing,
with the S3t binary of §49, accepts three tests and fails none: section 2.5.2
(the tag of "Cryptographic Forum Research Group"), appendix A.3 item 1 (64
zero bytes under the zero key), and the clamped $r$ of the section 2.5.2 key.
The listing uses `Mod[(1 << 130) - 5]`, size parameters, slices, and
`as little`.

`algorithms/chacha20-poly1305/chacha20-poly1305.or` writes the same MAC with
`Int` and `%` instead of `Mod`, and declares no `test` member.
`orangec eval` of `rfc8439_2_5_2` and of `rfc8439_a3_1` through
`rfc8439_a3_11` each equals its `_expected` spec. Twelve pairs, zero
mismatches. The AEAD and XChaCha20 specs in that file are not this section.

| Text | Status | What is missing |
| :--- | :--- | :--- |
| The listing: 1 through 5 blocks, at most 80 message bytes | Current | The three tests above |
| Appendix A.3 items 2 through 11, including the 375-byte text | Checked by `orangec eval` of `algorithms/chacha20-poly1305/chacha20-poly1305.or` | Not a `test` in this listing. The tags are that file's `rfc8439_a3_*_expected` specs |
| An empty message | Not in this listing | `blocks in 1..5` has no zero-block instance. The RFC's loop does not run, and the tag is the 16 little-endian bytes of $s$ |
| A message longer than 80 bytes in this listing | Same algorithm, a larger `blocks` range | At most 256 instances (§30). The file's `poly1305_mac` takes a 256-byte buffer and a length, and `poly1305_mac_long` takes a second buffer |

#### 1. The Field and the Key (section 2.5)

$$p = 2^{130} - 5 = 1361129467683753853853498429727072845819,$$

and $\mathbb{F}_p = \mathbb{Z}/p\mathbb{Z}$. `Mod[(1 << 130) - 5]` is that
field. `+` and `*` on it are the field operations, reduced into $[0, p-1]$
(§22). The 32-byte key splits into $r$, the
first 16 bytes as a little-endian integer, and $s$, the second 16 bytes as a
little-endian integer. $r$ is clamped by

$$r \leftarrow r \land \mathtt{0x0ffffffc0ffffffc0ffffffc0fffffff}.$$

On the bytes, that clears the top four bits of bytes 3, 7, 11, and 15
(`byte & 0x0f`) and the bottom two bits of bytes 4, 8, and 12
(`byte & 0xfc`). The other thirteen bytes are unchanged. $s$ is not clamped.
`clamped_r`
and `s_word` are those two values. The mask in the listing is the same
integer split into little-endian 64-bit halves, `0x0ffffffc0fffffff` and
`0x0ffffffc0ffffffc`.

#### 2. The Accumulator (section 2.5.1)

The message is read in 16-byte blocks. Block $j$ becomes the integer $n$:
its bytes, little-endian, with a `0x01` byte immediately above the bytes that
belong to the message. A full block has the `0x01` at $2^{128}$. A final
block of $k$ bytes, $1 \le k \le 16$, has it at $2^{8k}$. `weight(k)` is
$256^k$, which is that place. The accumulator starts at 0 and each block
updates it by

$$a \leftarrow r \cdot (a + n) \bmod p.$$

After the last block the tag is the 16 little-endian bytes of

$$(a + s) \bmod 2^{128}.$$

`mac` is that function for 1 through 5 blocks (a message of at most 80
bytes). The caller pads the array with zeros out to a whole number of blocks
and passes `held`, the number of message bytes in the last block. A longer
message is the same function with a larger finite `blocks` range. One `spec`
does not cover every length: a size parameter has at most 256 instances
(§30).

The RFC's `poly1305_mac` is that loop. The names in the listing:

| RFC 8439 section 2.5 | Orange in the listing |
| :--- | :--- |
| $p = (1 \ll 130) - 5$ | `type P = Mod[(1 << 130) - 5]` |
| `r = le_bytes_to_num(key[0..15])` then `clamp(r)` | `clamped_r`: `key[..16] as little Word[64]^2`, mask the halves, `as little P` |
| the mask `0x0ffffffc0ffffffc0ffffffc0fffffff` | low half `0x0ffffffc0fffffff`, high half `0x0ffffffc0ffffffc` |
| `s = le_bytes_to_num(key[16..31])` | `s_word`: `key[16..] as little Int` |
| `a = 0` | the `with a: P = 0` of the loop |
| one block, `n = le_bytes_to_num(block \|\| [0x01])` | `(m[16*j .. 16*j+16] as little P) + weight(k)`, and `weight(k)` is $256^{k}$ |
| `a += n; a = (r * a) % p` | `(a + n) * r` on `P` |
| `for i = 1 upto ceil(len/16)` | `for j in 0..blocks`, with `k = 16` on every block but the last |
| `a += s` | `(a as Int) + s_word(key)` |
| `num_to_16_le_bytes(a)`, the low 128 bits | `as little Word[8]^16`. A number converts to the words of its residue modulo $2^{\mathrm{width}}$ (`pack` in `eval.rs`), here width 128 |

#### 3. The Section 2.5.2 Key, Worked

The key's first 16 bytes, before clamping, are

`85 d6 be 78 57 55 6d 33 7f 44 52 fe 42 d5 06 a8`.

After the mask they are

`85 d6 be 08 54 55 6d 03 7c 44 52 0e 40 d5 06 08`.

The message "Cryptographic Forum Research Group" is 34 bytes, so two full
blocks and a final block of 2 bytes. Its tag is

`a8 06 1d c1 30 51 36 c6 c2 2b 8b af 0c 01 27 a9`.

Appendix A.3 item 1 is the tag of 64 zero bytes under the zero key, which is
16 zero bytes: $r = 0$, so the accumulator stays 0, and $s = 0$.

#### 4. Compiler-Checked Transcription

```orange
// RFC 8439 Poly1305, section 2.5. The prime is 2^130 - 5. r is the first 16
// key bytes, little-endian, clamped by the RFC's mask. Each block is a
// little-endian number with the byte 0x01 immediately above the bytes that
// belong to the message, then a = r * (a + n) mod p. s, the second 16 key
// bytes, is added once, and the sum is reduced modulo 2^128.
edition 2026;
module poly1305_spec {
  type P = Mod[(1 << 130) - 5];

  // 256^k for 0 <= k <= 16: the place of the byte 0x01 above k message bytes.
  spec weight(k: Int) -> P {
    for i in 0..16 with w: P = 1 { if i < k { w * 256 } else { w } }
  }

  // Section 2.5: clamp(r). The mask 0x0ffffffc0ffffffc0ffffffc0fffffff, split
  // into little-endian 64-bit halves.
  spec clamped_r(key: Word[8]^32) -> P {
    let half: Word[64]^2 = key[..16] as little Word[64]^2;
    [half[0] & 0x0ffffffc0fffffff, half[1] & 0x0ffffffc0ffffffc] as little P
  }

  spec s_word(key: Word[8]^32) -> Int { key[16..] as little Int }

  // Section 2.5.1 for a message padded with zeros out to `blocks` blocks of
  // 16 bytes. The last block holds `held` message bytes; every earlier block
  // holds 16. `held` is 16 when the message fills its last block.
  spec mac[blocks in 1..5](key: Word[8]^32, m: Word[8]^(16 * blocks), held: Int) -> Word[8]^16 {
    let r: P = clamped_r(key);
    let a: P = for j in 0..blocks with a: P = 0 {
      let k: Int = if j == (blocks - 1) { held } else { 16 };
      (a + (m[16 * j..16 * j + 16] as little P) + weight(k)) * r
    };
    ((a as Int) + s_word(key)) as little Word[8]^16
  }

  spec forum_key() -> Word[8]^32 {
    hex"85 d6 be 78 57 55 6d 33 7f 44 52 fe 42 d5 06 a8" ++
      hex"01 03 80 8a fb 0d b2 fd 4a bf f6 af 41 49 f5 1b"
  }

  test "RFC 8439 section 2.5.2 Poly1305 of the Forum name" {
    let message: Word[8]^34 = "Cryptographic Forum Research Group";
    mac(forum_key(), message ++ [0; 14], 2)
      == hex"a8 06 1d c1 30 51 36 c6 c2 2b 8b af 0c 01 27 a9"
  }

  test "RFC 8439 appendix A.3 item 1 Poly1305 of zeros" {
    mac([0; 32], [0; 64], 16) == [0; 16]
  }

  test "section 2.5: the Forum r keeps the clamped bits" {
    clamped_r(forum_key()) == (
      hex"85 d6 be 08 54 55 6d 03 7c 44 52 0e 40 d5 06 08" as little P
    )
  }
}
```

### AES, FIPS 197

AES as FIPS 197 (2001; update 1, 2023, DOI 10.6028/NIST.FIPS.197-upd1) writes
it: the column-major state of section 3.4, the four transformations of
section 5.1 and their inverses in section 5.3, Key Expansion of section 5.2
for $N_k \in \{4, 6, 8\}$, and Cipher and InvCipher. The listing is accepted
by `orangec test` on Appendix B and on Appendix C.1, C.2, and C.3, each
cipher and its inverse. The state in this listing is `Word[8]^16` in column
order, index $4c + r$ for row $r$ and column $c$. A rank-2 spelling of the
same state is what slice S3u also admits; this transcription is the
column-major vector `algorithms/aes/aes.or` checks. The modes of SP 800-38A
are `algorithms/aes/aes-modes.or` and are not restated here.

#### 1. Parameters (sections 2 and 3)

$N_b = 4$ is the number of columns. The round count $N_r$ is the standard's
table:

| $N_k$ | Key bits | $N_r$ |
| :--- | :--- | :--- |
| 4 | 128 | 10 |
| 6 | 192 | 12 |
| 8 | 256 | 14 |

#### 2. The Round (section 5.1)

SubBytes is the S-box of section 5.1.1. The listing packs that table eight
entries to a `Word[64]`, because the lookup index is the data byte: `lookup`
selects the word whose index is $x \gg 3$ and then the byte $x \land 7$. The
entries are the FIPS table. Entry $x$ is the affine map of section 5.1.1
applied to the inverse of $x$ in $\mathrm{GF}(2^8)$ modulo
$x^8 + x^4 + x^3 + x + 1$, and the inverse of 0 is 0.

ShiftRows moves row $r$ left by $r$ places: $s'[r, c] = s[r, (c + r) \bmod 4]$.
`shift_rows` is that reindexing on the column-major vector.

MixColumns multiplies each column by $a(x) = \{03\}x^3 + \{01\}x^2 + \{01\}x + \{02\}$
in $\mathrm{GF}(2^8)$. `xtime` is multiplication by $x$: a left shift, then
XOR with $\mathtt{0x1b}$ when the high bit was set. $\{03\}b = \mathrm{xtime}(b) + b$.

AddRoundKey XORs the four words of the round key into the state.
`rot_word` is a left rotation by one byte. `rcon` is the ten round constants
of section 5.2, from `0x01000000` through `0x36000000`. Key expansion follows
Algorithm 2: period $N_k$, RotWord and SubWord and Rcon when the index is a
multiple of $N_k$, and the extra SubWord when $N_k = 8$ and the index modulo
$N_k$ is 4. The expanded key is stored in a `Word[32]^60` buffer. AES-128
uses the first 44 words, AES-192 the first 52, AES-256 all 60. Words past
$N_b(N_r + 1)$ are unused.

Cipher is the standard's loop: AddRoundKey, then $N_r - 1$ rounds of
SubBytes, ShiftRows, MixColumns, AddRoundKey, then a final round without
MixColumns. InvCipher is section 5.3 in the matching order.

#### 3. Known Answers

Plaintext `00112233445566778899aabbccddeeff` under the Appendix C keys, and
the Appendix B example. Each inverse test returns that plaintext.

| Example | Ciphertext |
| :--- | :--- |
| C.1, AES-128, key `000102030405060708090a0b0c0d0e0f` | `69c4e0d86a7b0430d8cdb78070b4c55a` |
| C.2, AES-192, key `000102030405060708090a0b0c0d0e0f1011121314151617` | `dda97ca4864cdfe06eaf70a0ec0d7191` |
| C.3, AES-256, key through `1f` | `8ea2b7ca516745bfeafc49904b496089` |
| B, key `2b7e151628aed2a6abf7158809cf4f3c`, input `3243f6a8885a308d313198a2e0370734` | `3925841d02dc09fbdc118597196a0b32` |

#### 4. Compiler-Checked Transcription

```orange
// AES, the Advanced Encryption Standard of FIPS 197 (2001; update 1, 2023),
// https://doi.org/10.6028/NIST.FIPS.197-upd1, written as the standard writes
// it: the state is the 4 x 4 byte array of section 3.4 kept column by column,
// the four transformations of section 5.1 and their inverses of section 5.3
// act on it, KeyExpansion of section 5.2 makes the round keys for Nk = 4, 6
// and 8, and Cipher and InvCipher take the number of rounds Nr as the
// standard's algorithms do. The S-boxes are packed eight entries to a word.
// The file reproduces the cipher examples of Appendix C.1, C.2 and C.3 and
// their inverses, and the round-by-round example of Appendix B. The modes of
// operation of SP 800-38A are in aes-modes.or.
edition 2026;
module aes_spec {
  // Section 5.1.1: SubBytes is a table. Indices in Orange are static, so a
  // lookup selects the word holding the entry and then the byte in it.
  // The byte at position k of a word holding eight table entries.
  spec byte_at(w: Word[64], k: Word[8]) -> Word[8] {
    if k == 0 { (w >> 56) as Word[8] }
    else if k == 1 { (w >> 48) as Word[8] }
    else if k == 2 { (w >> 40) as Word[8] }
    else if k == 3 { (w >> 32) as Word[8] }
    else if k == 4 { (w >> 24) as Word[8] }
    else if k == 5 { (w >> 16) as Word[8] }
    else if k == 6 { (w >> 8) as Word[8] }
    else { w as Word[8] }
  }

  // Entry x of a 256-entry table packed eight entries to a word.
  spec lookup(t: Word[64]^32, x: Word[8]) -> Word[8] {
    let w: Word[64] = for i in 0..32 with acc: Word[64] = 0 {
      if (x >> 3) == (i as Word[8]) { t[i] } else { acc }
    };
    byte_at(w, x & 7)
  }

  // Section 5.1.1, Table 4 (Figure 7 of the 2001 text): the S-box. Entry x
  // is the affine map b'_i = b_i + b_(i+4) + b_(i+5) + b_(i+6) + b_(i+7) + c_i
  // over GF(2), the indices taken modulo 8 and c = 0x63, applied to the
  // multiplicative inverse of x in GF(2^8) modulo x^8 + x^4 + x^3 + x + 1,
  // the inverse of 0 being 0. Each word below is one half-row of the table
  // read left to right: 0x637c777bf26b6fc5 holds the entries
  // 63 7c 77 7b f2 6b 6f c5 for x = 00 through 07.
  spec sbox() -> Word[64]^32 {
    [
      0x637c777bf26b6fc5, 0x3001672bfed7ab76, 0xca82c97dfa5947f0, 0xadd4a2af9ca472c0,
      0xb7fd9326363ff7cc, 0x34a5e5f171d83115, 0x04c723c31896059a, 0x071280e2eb27b275,
      0x09832c1a1b6e5aa0, 0x523bd6b329e32f84, 0x53d100ed20fcb15b, 0x6acbbe394a4c58cf,
      0xd0efaafb434d3385, 0x45f9027f503c9fa8, 0x51a3408f929d38f5, 0xbcb6da2110fff3d2,
      0xcd0c13ec5f974417, 0xc4a77e3d645d1973, 0x60814fdc222a9088, 0x46eeb814de5e0bdb,
      0xe0323a0a4906245c, 0xc2d3ac629195e479, 0xe7c8376d8dd54ea9, 0x6c56f4ea657aae08,
      0xba78252e1ca6b4c6, 0xe8dd741f4bbd8b8a, 0x703eb5664803f60e, 0x613557b986c11d9e,
      0xe1f8981169d98e94, 0x9b1e87e9ce5528df, 0x8ca1890dbfe64268, 0x41992d0fb054bb16,
    ]
  }

  // Section 5.3.2, Table 6 (Figure 14 of the 2001 text): the inverse S-box,
  // packed the same way.
  spec inv_sbox() -> Word[64]^32 {
    [
      0x52096ad53036a538, 0xbf40a39e81f3d7fb, 0x7ce339829b2fff87, 0x348e4344c4dee9cb,
      0x547b9432a6c2233d, 0xee4c950b42fac34e, 0x082ea16628d924b2, 0x765ba2496d8bd125,
      0x72f8f66486689816, 0xd4a45ccc5d65b692, 0x6c704850fdedb9da, 0x5e154657a78d9d84,
      0x90d8ab008cbcd30a, 0xf7e45805b8b34506, 0xd02c1e8fca3f0f02, 0xc1afbd0301138a6b,
      0x3a9111414f67dcea, 0x97f2cfcef0b4e673, 0x96ac7422e7ad3585, 0xe2f937e81c75df6e,
      0x47f11a711d29c589, 0x6fb7620eaa18be1b, 0xfc563e4bc6d27920, 0x9adbc0fe78cd5af4,
      0x1fdda8338807c731, 0xb11210592780ec5f, 0x60517fa919b54a0d, 0x2de57a9f93c99cef,
      0xa0e03b4dae2af5b0, 0xc8ebbb3c83539961, 0x172b047eba77d626, 0xe169146355210c7d,
    ]
  }

  // Section 3.4: the state is s[r, c] = in[r + 4c], so a block of sixteen
  // input bytes is the state read column by column, and the sixteen bytes of
  // a round key are the four words w[4 round + c] read byte by byte.
  spec load_be32(b0: Word[8], b1: Word[8], b2: Word[8], b3: Word[8]) -> Word[32] {
    ((b0 as Word[32]) << 24) | ((b1 as Word[32]) << 16) | ((b2 as Word[32]) << 8) | (b3 as Word[32])
  }

  spec be_bytes(x: Word[32]) -> Word[8]^4 {
    [(x >> 24) as Word[8], (x >> 16) as Word[8], (x >> 8) as Word[8], x as Word[8]]
  }

  // Section 5.1.1: SubBytes, the S-box applied to every byte of the state.
  // The table s is passed in so that a caller loads it once per block.
  spec sub_bytes(state: Word[8]^16, s: Word[64]^32) -> Word[8]^16 {
    [
      lookup(s, state[0]), lookup(s, state[1]), lookup(s, state[2]), lookup(s, state[3]),
      lookup(s, state[4]), lookup(s, state[5]), lookup(s, state[6]), lookup(s, state[7]),
      lookup(s, state[8]), lookup(s, state[9]), lookup(s, state[10]), lookup(s, state[11]),
      lookup(s, state[12]), lookup(s, state[13]), lookup(s, state[14]), lookup(s, state[15]),
    ]
  }

  // Section 5.1.2: ShiftRows moves row r left by r, s'[r, c] = s[r, c + r mod 4];
  // in column-major order that is the fixed reindexing below.
  spec shift_rows(s: Word[8]^16) -> Word[8]^16 {
    [
      s[0], s[5], s[10], s[15],
      s[4], s[9], s[14], s[3],
      s[8], s[13], s[2], s[7],
      s[12], s[1], s[6], s[11],
    ]
  }

  // Section 4.2: xtime, multiplication by x in GF(2^8) modulo x^8 + x^4 + x^3 + x + 1.
  spec xtime(a: Word[8]) -> Word[8] {
    (a << 1) ^ (if (a & 0x80) != 0 { 0x1b } else { 0 })
  }

  // Section 5.1.3: MixColumns multiplies each column by the fixed polynomial
  // a(x) = {03}x^3 + {01}x^2 + {01}x + {02}, where {03}b = xtime(b) + b.
  spec mix_column(s0: Word[8], s1: Word[8], s2: Word[8], s3: Word[8]) -> Word[8]^4 {
    [
      xtime(s0) ^ (xtime(s1) ^ s1) ^ s2 ^ s3,
      s0 ^ xtime(s1) ^ (xtime(s2) ^ s2) ^ s3,
      s0 ^ s1 ^ xtime(s2) ^ (xtime(s3) ^ s3),
      (xtime(s0) ^ s0) ^ s1 ^ s2 ^ xtime(s3),
    ]
  }

  spec mix_columns(s: Word[8]^16) -> Word[8]^16 {
    let c0: Word[8]^4 = mix_column(s[0], s[1], s[2], s[3]);
    let c1: Word[8]^4 = mix_column(s[4], s[5], s[6], s[7]);
    let c2: Word[8]^4 = mix_column(s[8], s[9], s[10], s[11]);
    let c3: Word[8]^4 = mix_column(s[12], s[13], s[14], s[15]);
    [
      c0[0], c0[1], c0[2], c0[3],
      c1[0], c1[1], c1[2], c1[3],
      c2[0], c2[1], c2[2], c2[3],
      c3[0], c3[1], c3[2], c3[3],
    ]
  }

  // Section 5.1.4: AddRoundKey, the four words of the round key added to the
  // four columns.
  spec add_round_key(s: Word[8]^16, k: Word[32]^4) -> Word[8]^16 {
    let k0: Word[8]^4 = be_bytes(k[0]);
    let k1: Word[8]^4 = be_bytes(k[1]);
    let k2: Word[8]^4 = be_bytes(k[2]);
    let k3: Word[8]^4 = be_bytes(k[3]);
    [
      s[0] ^ k0[0], s[1] ^ k0[1], s[2] ^ k0[2], s[3] ^ k0[3],
      s[4] ^ k1[0], s[5] ^ k1[1], s[6] ^ k1[2], s[7] ^ k1[3],
      s[8] ^ k2[0], s[9] ^ k2[1], s[10] ^ k2[2], s[11] ^ k2[3],
      s[12] ^ k3[0], s[13] ^ k3[1], s[14] ^ k3[2], s[15] ^ k3[3],
    ]
  }

  // Section 5.3.1: InvShiftRows, s'[r, c] = s[r, c - r mod 4].
  spec inv_shift_rows(s: Word[8]^16) -> Word[8]^16 {
    [
      s[0], s[13], s[10], s[7],
      s[4], s[1], s[14], s[11],
      s[8], s[5], s[2], s[15],
      s[12], s[9], s[6], s[3],
    ]
  }

  // Section 5.3.2: InvSubBytes, with the inverse S-box passed in as s.
  spec inv_sub_bytes(state: Word[8]^16, s: Word[64]^32) -> Word[8]^16 {
    sub_bytes(state, s)
  }

  // Section 5.3.3: InvMixColumns multiplies each column by
  // a^-1(x) = {0b}x^3 + {0d}x^2 + {09}x + {0e}. The constants are sums of
  // b, {02}b, {04}b and {08}b, three applications of xtime.
  spec inv_mix_column(s0: Word[8], s1: Word[8], s2: Word[8], s3: Word[8]) -> Word[8]^4 {
    let x2: Word[8]^4 = [xtime(s0), xtime(s1), xtime(s2), xtime(s3)];
    let x4: Word[8]^4 = [xtime(x2[0]), xtime(x2[1]), xtime(x2[2]), xtime(x2[3])];
    let x8: Word[8]^4 = [xtime(x4[0]), xtime(x4[1]), xtime(x4[2]), xtime(x4[3])];
    // {09}b = {08}b + b, {0b}b = {08}b + {02}b + b, {0d}b = {08}b + {04}b + b,
    // {0e}b = {08}b + {04}b + {02}b.
    let m9: Word[8]^4 = [x8[0] ^ s0, x8[1] ^ s1, x8[2] ^ s2, x8[3] ^ s3];
    let mb: Word[8]^4 = [m9[0] ^ x2[0], m9[1] ^ x2[1], m9[2] ^ x2[2], m9[3] ^ x2[3]];
    let md: Word[8]^4 = [m9[0] ^ x4[0], m9[1] ^ x4[1], m9[2] ^ x4[2], m9[3] ^ x4[3]];
    let me: Word[8]^4 = [
      x8[0] ^ x4[0] ^ x2[0], x8[1] ^ x4[1] ^ x2[1], x8[2] ^ x4[2] ^ x2[2], x8[3] ^ x4[3] ^ x2[3],
    ];
    [
      me[0] ^ mb[1] ^ md[2] ^ m9[3],
      m9[0] ^ me[1] ^ mb[2] ^ md[3],
      md[0] ^ m9[1] ^ me[2] ^ mb[3],
      mb[0] ^ md[1] ^ m9[2] ^ me[3],
    ]
  }

  spec inv_mix_columns(s: Word[8]^16) -> Word[8]^16 {
    let c0: Word[8]^4 = inv_mix_column(s[0], s[1], s[2], s[3]);
    let c1: Word[8]^4 = inv_mix_column(s[4], s[5], s[6], s[7]);
    let c2: Word[8]^4 = inv_mix_column(s[8], s[9], s[10], s[11]);
    let c3: Word[8]^4 = inv_mix_column(s[12], s[13], s[14], s[15]);
    [
      c0[0], c0[1], c0[2], c0[3],
      c1[0], c1[1], c1[2], c1[3],
      c2[0], c2[1], c2[2], c2[3],
      c3[0], c3[1], c3[2], c3[3],
    ]
  }

  // Section 5.2: the words of the key schedule. RotWord is a cyclic shift of
  // the four bytes, SubWord the S-box on each of them, and Rcon[j] is the
  // word [x^(j-1), 00, 00, 00] with the powers of x taken in GF(2^8)
  // (Table 5). The schedule w has 4(Nr + 1) words; the array is sized for
  // AES-256, and AES-128 and AES-192 leave its tail at zero.
  spec rot_word(w: Word[32]) -> Word[32] { w <<< 8 }

  spec sub_word(w: Word[32], s: Word[64]^32) -> Word[32] {
    load_be32(
      lookup(s, (w >> 24) as Word[8]),
      lookup(s, (w >> 16) as Word[8]),
      lookup(s, (w >> 8) as Word[8]),
      lookup(s, w as Word[8]),
    )
  }

  spec rcon() -> Word[32]^10 {
    [
      0x01000000, 0x02000000, 0x04000000, 0x08000000, 0x10000000, 0x20000000, 0x40000000, 0x80000000,
      0x1b000000, 0x36000000,
    ]
  }

  // Section 5.2, Algorithm 2 with Nk = 4: w[i] = w[i - 4] + temp, where temp
  // is w[i - 1], transformed by RotWord, SubWord and Rcon[i / 4] when i is a
  // multiple of 4.
  spec key_expansion_128(key: Word[8]^16) -> Word[32]^60 {
    let s: Word[64]^32 = sbox();
    let r: Word[32]^10 = rcon();
    let first: Word[32]^60 = for i in 0..4 with w: Word[32]^60 = [0; 60] {
      w with [i] = load_be32(key[4 * i], key[4 * i + 1], key[4 * i + 2], key[4 * i + 3])
    };
    for i in 4..44 with w: Word[32]^60 = first {
      w with [i] = w[i - 4] ^ (
        if (i % 4) == 0 { sub_word(rot_word(w[i - 1]), s) ^ r[(i / 4) - 1] } else { w[i - 1] }
      )
    }
  }

  // Algorithm 2 with Nk = 6: the same recurrence with period 6.
  spec key_expansion_192(key: Word[8]^24) -> Word[32]^60 {
    let s: Word[64]^32 = sbox();
    let r: Word[32]^10 = rcon();
    let first: Word[32]^60 = for i in 0..6 with w: Word[32]^60 = [0; 60] {
      w with [i] = load_be32(key[4 * i], key[4 * i + 1], key[4 * i + 2], key[4 * i + 3])
    };
    for i in 6..52 with w: Word[32]^60 = first {
      w with [i] = w[i - 6] ^ (
        if (i % 6) == 0 { sub_word(rot_word(w[i - 1]), s) ^ r[(i / 6) - 1] } else { w[i - 1] }
      )
    }
  }

  // Algorithm 2 with Nk = 8: period 8, and SubWord alone when i is 4 more
  // than a multiple of 8.
  spec key_expansion_256(key: Word[8]^32) -> Word[32]^60 {
    let s: Word[64]^32 = sbox();
    let r: Word[32]^10 = rcon();
    let first: Word[32]^60 = for i in 0..8 with w: Word[32]^60 = [0; 60] {
      w with [i] = load_be32(key[4 * i], key[4 * i + 1], key[4 * i + 2], key[4 * i + 3])
    };
    for i in 8..60 with w: Word[32]^60 = first {
      w with [i] = w[i - 8] ^ (
        if (i % 8) == 0 { sub_word(rot_word(w[i - 1]), s) ^ r[(i / 8) - 1] }
        else if (i % 8) == 4 { sub_word(w[i - 1], s) }
        else { w[i - 1] }
      )
    }
  }

  // Section 5.1, Algorithm 1: Cipher(in, Nr, w). Round 0 adds the first round
  // key; rounds 1 through Nr - 1 apply SubBytes, ShiftRows, MixColumns and
  // AddRoundKey; round Nr omits MixColumns. The loop runs to the 14 rounds of
  // AES-256 and the rounds above Nr leave the state alone, since the round
  // index must be static to select the round key.
  spec cipher(input: Word[8]^16, nr: Int, w: Word[32]^60) -> Word[8]^16 {
    let s: Word[64]^32 = sbox();
    let start: Word[8]^16 = add_round_key(input, [w[0], w[1], w[2], w[3]]);
    for round in 1..15 with state: Word[8]^16 = start {
      if round < nr {
        add_round_key(
          mix_columns(shift_rows(sub_bytes(state, s))),
          [w[4 * round], w[4 * round + 1], w[4 * round + 2], w[4 * round + 3]],
        )
      } else if round == nr {
        add_round_key(
          shift_rows(sub_bytes(state, s)),
          [w[4 * round], w[4 * round + 1], w[4 * round + 2], w[4 * round + 3]],
        )
      } else { state }
    }
  }

  // Section 5.3, Algorithm 3: InvCipher(in, Nr, w). The rounds run from Nr
  // down to 1, so the loop counts j up and takes round = 14 - j: round Nr
  // adds the last round key, rounds Nr - 1 through 1 apply InvShiftRows,
  // InvSubBytes, AddRoundKey and InvMixColumns, and the first round key is
  // added after the loop.
  spec inv_cipher(input: Word[8]^16, nr: Int, w: Word[32]^60) -> Word[8]^16 {
    let s: Word[64]^32 = inv_sbox();
    let last: Word[8]^16 = for j in 0..14 with state: Word[8]^16 = input {
      if (14 - j) == nr {
        add_round_key(
          state,
          [w[4 * (14 - j)], w[4 * (14 - j) + 1], w[4 * (14 - j) + 2], w[4 * (14 - j) + 3]],
        )
      } else if (14 - j) < nr {
        inv_mix_columns(add_round_key(
          inv_sub_bytes(inv_shift_rows(state), s),
          [w[4 * (14 - j)], w[4 * (14 - j) + 1], w[4 * (14 - j) + 2], w[4 * (14 - j) + 3]],
        ))
      } else { state }
    };
    add_round_key(inv_sub_bytes(inv_shift_rows(last), s), [w[0], w[1], w[2], w[3]])
  }

  // Section 5, as the 2023 update names them: AES-128, AES-192 and AES-256
  // are Cipher with Nr = 10, 12 and 14 under the schedule of their key, and
  // their inverses are InvCipher under the same schedule.
  spec aes128(key: Word[8]^16, input: Word[8]^16) -> Word[8]^16 {
    cipher(input, 10, key_expansion_128(key))
  }

  spec aes192(key: Word[8]^24, input: Word[8]^16) -> Word[8]^16 {
    cipher(input, 12, key_expansion_192(key))
  }

  spec aes256(key: Word[8]^32, input: Word[8]^16) -> Word[8]^16 {
    cipher(input, 14, key_expansion_256(key))
  }

  spec aes128_inverse(key: Word[8]^16, input: Word[8]^16) -> Word[8]^16 {
    inv_cipher(input, 10, key_expansion_128(key))
  }

  spec aes192_inverse(key: Word[8]^24, input: Word[8]^16) -> Word[8]^16 {
    inv_cipher(input, 12, key_expansion_192(key))
  }

  spec aes256_inverse(key: Word[8]^32, input: Word[8]^16) -> Word[8]^16 {
    inv_cipher(input, 14, key_expansion_256(key))
  }

  // FIPS 197, Appendix C.1: AES-128, plaintext 00112233...eeff under the key 000102...0f.
  spec fips197_c1_aes128() -> Word[8]^16 {
    aes128(
      [
        0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f,
      ],
      [
        0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff,
      ],
    )
  }

  spec fips197_c1_aes128_expected() -> Word[8]^16 {
    [
      0x69, 0xc4, 0xe0, 0xd8, 0x6a, 0x7b, 0x04, 0x30, 0xd8, 0xcd, 0xb7, 0x80, 0x70, 0xb4, 0xc5, 0x5a,
    ]
  }

  // FIPS 197, Appendix C.1, INVERSE CIPHER: the ciphertext back to the
  // plaintext.
  spec fips197_c1_aes128_inverse() -> Word[8]^16 {
    aes128_inverse(
      [
        0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f,
      ],
      [
        0x69, 0xc4, 0xe0, 0xd8, 0x6a, 0x7b, 0x04, 0x30, 0xd8, 0xcd, 0xb7, 0x80, 0x70, 0xb4, 0xc5, 0x5a,
      ],
    )
  }

  spec fips197_c1_aes128_inverse_expected() -> Word[8]^16 {
    [
      0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff,
    ]
  }

  // FIPS 197, Appendix C.2: AES-192, plaintext 00112233...eeff under the key 000102...17.
  spec fips197_c2_aes192() -> Word[8]^16 {
    aes192(
      [
        0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f,
        0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17,
      ],
      [
        0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff,
      ],
    )
  }

  spec fips197_c2_aes192_expected() -> Word[8]^16 {
    [
      0xdd, 0xa9, 0x7c, 0xa4, 0x86, 0x4c, 0xdf, 0xe0, 0x6e, 0xaf, 0x70, 0xa0, 0xec, 0x0d, 0x71, 0x91,
    ]
  }

  // FIPS 197, Appendix C.2, INVERSE CIPHER.
  spec fips197_c2_aes192_inverse() -> Word[8]^16 {
    aes192_inverse(
      [
        0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f,
        0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17,
      ],
      [
        0xdd, 0xa9, 0x7c, 0xa4, 0x86, 0x4c, 0xdf, 0xe0, 0x6e, 0xaf, 0x70, 0xa0, 0xec, 0x0d, 0x71, 0x91,
      ],
    )
  }

  spec fips197_c2_aes192_inverse_expected() -> Word[8]^16 {
    [
      0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff,
    ]
  }

  // FIPS 197, Appendix C.3: AES-256, plaintext 00112233...eeff under the key 000102...1f.
  spec fips197_c3_aes256() -> Word[8]^16 {
    aes256(
      [
        0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f,
        0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e, 0x1f,
      ],
      [
        0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff,
      ],
    )
  }

  spec fips197_c3_aes256_expected() -> Word[8]^16 {
    [
      0x8e, 0xa2, 0xb7, 0xca, 0x51, 0x67, 0x45, 0xbf, 0xea, 0xfc, 0x49, 0x90, 0x4b, 0x49, 0x60, 0x89,
    ]
  }

  // FIPS 197, Appendix C.3, INVERSE CIPHER.
  spec fips197_c3_aes256_inverse() -> Word[8]^16 {
    aes256_inverse(
      [
        0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f,
        0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e, 0x1f,
      ],
      [
        0x8e, 0xa2, 0xb7, 0xca, 0x51, 0x67, 0x45, 0xbf, 0xea, 0xfc, 0x49, 0x90, 0x4b, 0x49, 0x60, 0x89,
      ],
    )
  }

  spec fips197_c3_aes256_inverse_expected() -> Word[8]^16 {
    [
      0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff,
    ]
  }

  // FIPS 197, Appendix B: the round-by-round example, input 3243f6a8... under
  // the key 2b7e1516...; the output 3925841d... is the value the Python
  // cryptography package gives for it, since the vector file omits this case.
  spec fips197_b_aes128() -> Word[8]^16 {
    aes128(
      [
        0x2b, 0x7e, 0x15, 0x16, 0x28, 0xae, 0xd2, 0xa6, 0xab, 0xf7, 0x15, 0x88, 0x09, 0xcf, 0x4f, 0x3c,
      ],
      [
        0x32, 0x43, 0xf6, 0xa8, 0x88, 0x5a, 0x30, 0x8d, 0x31, 0x31, 0x98, 0xa2, 0xe0, 0x37, 0x07, 0x34,
      ],
    )
  }

  spec fips197_b_aes128_expected() -> Word[8]^16 {
    [
      0x39, 0x25, 0x84, 0x1d, 0x02, 0xdc, 0x09, 0xfb, 0xdc, 0x11, 0x85, 0x97, 0x19, 0x6a, 0x0b, 0x32,
    ]
  }

  test "FIPS 197 Appendix C.1 AES-128" {
    fips197_c1_aes128() == fips197_c1_aes128_expected()
  }

  test "FIPS 197 Appendix C.1 inverse returns the plaintext" {
    fips197_c1_aes128_inverse() == fips197_c1_aes128_inverse_expected()
  }

  test "FIPS 197 Appendix B round-by-round example" {
    fips197_b_aes128() == fips197_b_aes128_expected()
  }

  test "FIPS 197 Appendix C.2 AES-192" {
    (fips197_c2_aes192() == fips197_c2_aes192_expected())
      && (fips197_c2_aes192_inverse() == fips197_c2_aes192_inverse_expected())
  }

  test "FIPS 197 Appendix C.3 AES-256" {
    (fips197_c3_aes256() == fips197_c3_aes256_expected())
      && (fips197_c3_aes256_inverse() == fips197_c3_aes256_inverse_expected())
  }
}
```

### HMAC, FIPS 198-1

HMAC-SHA-256 as FIPS 198-1 (2008, DOI 10.6028/NIST.FIPS.198-1) sections 4 and
5 write it, and as RFC 2104 first defined it. The hash $H$ is SHA-256 of §49.
The listing restates that hash, because a module has no imports. The buffer
is not the byte array of §49. Each byte string is a length together with a
`Word[32]^64`: the big-endian words of FIPS 180-4 section 5.2.1, zero after
the last byte. One `sha256` and one `hmac_sha256` then serve every length in
the tests. `orangec test` accepts the ten tests: the pad words of RFC 4231
case 2, $K_0$ of case 6, the seven HMAC-SHA-256 cases of RFC 4231 section 4
(case 5 also truncated to 128 bits), and Wycheproof `hmac_sha256` tcId 171.

#### 1. Parameters (FIPS 198-1, sections 3 and 4)

$B = 64$ is the SHA-256 block in bytes. $L = 32$ is the digest length. The
pads are the bytes `0x36` and `0x5c`, each repeated $B$ times. As words they
are `0x36363636` and `0x5c5c5c5c`.

| Step | $K_0$, the key brought to $B$ bytes |
| :--- | :--- |
| Key length $= B$ | the key |
| Key length $> B$ | $H(\mathrm{key})$, then zeros through byte $B$ |
| Key length $< B$ | the key, then zeros |

The buffer is already zero past the key, so a key of at most $B$ bytes
contributes its first sixteen words unchanged. A longer key is hashed only
in the taken branch of `k0`.

#### 2. The MAC (section 4, steps 4 through 9)

$$\mathrm{HMAC}(K, \mathrm{text}) = H((K_0 \oplus \mathrm{opad}) \parallel H((K_0 \oplus \mathrm{ipad}) \parallel \mathrm{text})).$$

`xor_pad` XORs each of the sixteen words of $K_0$ with the pad word. The
inner hash is SHA-256 of the 64-byte inner pad followed by the text. The
outer hash is SHA-256 of the 64-byte outer pad followed by the 32-byte inner
digest, a message of 96 bytes. Truncation keeps the leftmost $t$ bits of
that digest. `leftmost_128` is $t = 128$, RFC 2104 section 5 and the
HMAC-SHA-256-128 value RFC 4231 prints for case 5.

RFC 4231 case 2 has key "Jefe", the word `0x4a656665`, shorter than the
block. $K_0 \oplus \mathrm{ipad}$ begins `0x7c535053` and every later word is
`0x36363636`. $K_0 \oplus \mathrm{opad}$ begins `0x16393a39` and every later
word is `0x5c5c5c5c`.

RFC 4231 case 6 has key `0xaa` repeated 131 times, longer than $B$. $K_0$ is
SHA-256 of that key,

`45ad4b37c6e2fc0a2cfcc1b5da524132ec707615c2cae1dbbc43c97aa521db81`,

followed by 32 zero bytes.

#### 3. Known Answers (RFC 4231, section 4)

| Case | Key and text | MAC |
| :--- | :--- | :--- |
| 4.2 | `0x0b` repeated 20 times, "Hi There" | `b0344c61d8db38535ca8afceaf0bf12b881dc200c9833da726e9376c2e32cff7` |
| 4.3 | "Jefe", "what do ya want for nothing?" | `5bdcc146bf60754e6a042426089575c75a003f089d2739839dec58b964ec3843` |
| 4.4 | `0xaa` 20 times, `0xdd` 50 times | `773ea91e36800e46854db8ebd09181a72959098b3ef8c122d9635514ced565fe` |
| 4.5 | bytes `01` through `19`, `0xcd` 50 times | `82558a389a443c0ea4cc819899f2083a85f0faa3e578f8077a2e3ff46729665b` |
| 4.6 | `0x0c` 20 times, "Test With Truncation" | `a3b6167473100ee06e0c796c2955552b` in the first 128 bits |
| 4.7 | `0xaa` 131 times, the 54-byte block-size sentence | `60e431591ee0b67f0d8a26aacbf5b77f8e0bc6213728c5140546040f0ee37f54` |
| 4.8 | the same key, the 152-byte sentence | `9b09ffa71b942fcb27635fbcd5b0e944bfdc63644f0713938a7f51535c3a35e2` |

RFC 4231 prints case 5 only after truncation to 128 bits. The listing also
checks the full 32-byte MAC
`a3b6167473100ee06e0c796c2955552bfa6f7c0a6a8aef8b93f860aab0cd20c5`.
Wycheproof `hmac_sha256_test.json` tcId 171 is a 65-byte key, one byte over
$B$, and a 32-byte message. Its tag is
`e542ac8ac8f364bae4b7da8b7a0777df350f001de4e8cfa2d9ef0b15019496ec`.

The inner message is $64 + n$ bytes. SHA-256 padding needs nine bytes past
a whole-byte message, and the buffer is 256 bytes, so the text in this
listing is at most 183 bytes. A key that is hashed first is at most 247
bytes. Case 7, at 152 bytes of text and 131 bytes of key, sits inside both
bounds. A longer string is the same functions on a larger finite buffer.
HKDF-Extract and HKDF-Expand are the next section. They are not in this
listing: one evaluation budget is $1\,048\,576$ steps, and the three RFC
5869 appendix A cases are a separate file in `algorithms/hmac-hkdf/`.

#### 4. Compiler-Checked Transcription

```orange
// HMAC-SHA-256 as FIPS 198-1 (2008, DOI 10.6028/NIST.FIPS.198-1) sections 4
// and 5 write it, first published as RFC 2104. The hash is SHA-256 of FIPS
// 180-4, restated here because a module has no imports. Every byte string is
// its length and a Word[32]^64 of big-endian words, zero past the last byte,
// so one sha256 and one hmac_sha256 serve the RFC 4231 lengths. The seven
// HMAC-SHA-256 cases of RFC 4231 section 4 are the tests, case 5 also
// truncated to 128 bits, plus the 65-byte key of Wycheproof hmac_sha256
// tcId 171. HKDF is a separate transcription.
edition 2026;
module hmac_spec {
  // A word to its bytes, big-endian (FIPS 180-4 section 3.1); the inputs
  // are already words, so only the output direction is needed.
  spec be_bytes(x: Word[32]) -> Word[8]^4 {
    [(x >> 24) as Word[8], (x >> 16) as Word[8], (x >> 8) as Word[8], x as Word[8]]
  }

  // FIPS 180-4 section 4.1.2: the SHA-256 functions.
  spec ch(x: Word[32], y: Word[32], z: Word[32]) -> Word[32] { (x & y) ^ (~x & z) }
  spec maj(x: Word[32], y: Word[32], z: Word[32]) -> Word[32] { (x & y) ^ (x & z) ^ (y & z) }
  spec big_sigma0(x: Word[32]) -> Word[32] { (x >>> 2) ^ (x >>> 13) ^ (x >>> 22) }
  spec big_sigma1(x: Word[32]) -> Word[32] { (x >>> 6) ^ (x >>> 11) ^ (x >>> 25) }
  spec small_sigma0(x: Word[32]) -> Word[32] { (x >>> 7) ^ (x >>> 18) ^ (x >> 3) }
  spec small_sigma1(x: Word[32]) -> Word[32] { (x >>> 17) ^ (x >>> 19) ^ (x >> 10) }

  // FIPS 180-4 section 4.2.2: K{256}, the first 32 bits of the fractional
  // parts of the cube roots of the first 64 primes.
  spec round_constants() -> Word[32]^64 {
    [
      0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
      0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
      0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
      0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
      0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
      0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
      0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
      0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
    ]
  }

  // FIPS 180-4 section 5.3.3: H(0), the first 32 bits of the fractional parts
  // of the square roots of the first eight primes.
  spec initial_hash() -> Word[32]^8 {
    [
      0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a,
      0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
    ]
  }

  // FIPS 180-4 section 6.2.2, step 1: the message schedule W_0 through W_63.
  spec schedule(m: Word[32]^16) -> Word[32]^64 {
    let head: Word[32]^64 = for t in 0..16 with w: Word[32]^64 = [0; 64] { w with [t] = m[t] };
    for t in 16..64 with w: Word[32]^64 = head {
      w with [t] = small_sigma1(w[t - 2]) + w[t - 7] + small_sigma0(w[t - 15]) + w[t - 16]
    }
  }

  // FIPS 180-4 section 6.2.2, step 3: one round on the working variables a
  // through h, carried as v = [a, b, c, d, e, f, g, h].
  spec round(v: Word[32]^8, k: Word[32], w: Word[32]) -> Word[32]^8 {
    let t1: Word[32] = v[7] + big_sigma1(v[4]) + ch(v[4], v[5], v[6]) + k + w;
    let t2: Word[32] = big_sigma0(v[0]) + maj(v[0], v[1], v[2]);
    [t1 + t2, v[0], v[1], v[2], v[3] + t1, v[4], v[5], v[6]]
  }

  // FIPS 180-4 section 6.2.2, steps 1 through 4: one block, H(i-1) to H(i).
  spec compress(h: Word[32]^8, m: Word[32]^16) -> Word[32]^8 {
    let w: Word[32]^64 = schedule(m);
    let k: Word[32]^64 = round_constants();
    let v: Word[32]^8 = for t in 0..64 with v: Word[32]^8 = h { round(v, k[t], w[t]) };
    for i in 0..8 with out: Word[32]^8 = v { out with [i] = out[i] + h[i] }
  }

  // Byte p of the buffer, 0 <= p < 256, set to v by or: byte p lives in
  // word p / 4, at the shift that p % 4 selects. The shift amount must be a
  // literal, so the four positions are a conditional. This writes the
  // padding byte 0x80 of SHA-256, the byte whose position depends on a
  // length.
  spec put_byte(m: Word[32]^64, p: Int, v: Word[8]) -> Word[32]^64 {
    let shifted: Word[32] = if (p % 4) == 0 { (v as Word[32]) << 24 }
      else if (p % 4) == 1 { (v as Word[32]) << 16 }
      else if (p % 4) == 2 { (v as Word[32]) << 8 }
      else { v as Word[32] };
    for i in 0..64 with out: Word[32]^64 = m {
      if i == (p / 4) { out with [i] = out[i] | shifted } else { out }
    }
  }

  // FIPS 180-4 section 5.1.1 for a message of n bytes, n <= 247: the bit 1
  // (the byte 0x80, since every message is whole bytes) after the message,
  // k zero bits, and the length l = 8n as a 64-bit big-endian integer at
  // the end of block N, N = (n + 72) / 64 = ceiling((n + 9) / 64). The high
  // word of l is zero for every n here, so only the last word is written.
  spec pad(m: Word[32]^64, n: Int) -> Word[32]^64 {
    let marked: Word[32]^64 = put_byte(m, n, 0x80);
    let blocks: Int = (n + 72) / 64;
    for i in 0..4 with w: Word[32]^64 = marked {
      if (i + 1) == blocks { w with [16 * i + 15] = (8 * n) as Word[32] } else { w }
    }
  }

  // FIPS 180-4 section 6.2.2, "for i = 1 to N": the N blocks of the padded
  // message in order, each sixteen words of the buffer, from H(0).
  spec sha256(m: Word[32]^64, n: Int) -> Word[32]^8 {
    let w: Word[32]^64 = pad(m, n);
    let blocks: Int = (n + 72) / 64;
    for i in 0..4 with h: Word[32]^8 = initial_hash() {
      if i < blocks {
        compress(h, [
          w[16 * i], w[16 * i + 1], w[16 * i + 2], w[16 * i + 3],
          w[16 * i + 4], w[16 * i + 5], w[16 * i + 6], w[16 * i + 7],
          w[16 * i + 8], w[16 * i + 9], w[16 * i + 10], w[16 * i + 11],
          w[16 * i + 12], w[16 * i + 13], w[16 * i + 14], w[16 * i + 15],
        ])
      } else { h }
    }
  }

  // FIPS 198-1 section 4: B = 64, the block size of SHA-256 in bytes, and
  // the pads ipad = 0x36 and opad = 0x5c repeated B times, here as words.
  spec ipad() -> Word[32] { 0x36363636 }
  spec opad() -> Word[32] { 0x5c5c5c5c }

  // FIPS 198-1 section 4, steps 1 to 3: K0, the key brought to B bytes. A
  // key of B bytes is used as it is; a longer key is hashed to L = 32 bytes
  // and zeros are appended; a shorter key has zeros appended. The buffer
  // is zero beyond the key, so its first sixteen words are the padded key.
  spec k0(key: Word[32]^64, key_len: Int) -> Word[32]^16 {
    let hashed: Word[32]^8 = if key_len > 64 { sha256(key, key_len) } else { [0; 8] };
    if key_len > 64 {
      [hashed[0], hashed[1], hashed[2], hashed[3], hashed[4], hashed[5], hashed[6], hashed[7],
       0, 0, 0, 0, 0, 0, 0, 0]
    } else {
      [key[0], key[1], key[2], key[3], key[4], key[5], key[6], key[7],
       key[8], key[9], key[10], key[11], key[12], key[13], key[14], key[15]]
    }
  }

  // Steps 4 and 7: K0 xor ipad and K0 xor opad.
  spec xor_pad(k: Word[32]^16, pad_word: Word[32]) -> Word[32]^16 {
    for i in 0..16 with out: Word[32]^16 = k { out with [i] = k[i] ^ pad_word }
  }

  // FIPS 198-1 section 4, steps 4 to 9, and section 5:
  // HMAC(K, text) = H((K0 xor opad) || H((K0 xor ipad) || text)).
  // Step 5 appends the text, of n <= 183 bytes, to the 64-byte inner pad
  // (so the inner message fits four blocks); step 8 appends the 32-byte
  // inner hash to the outer pad, a 96-byte message. Truncation to t bytes
  // is left to the caller (`leftmost_128`); the full L = 32 bytes are
  // returned as the eight words of the hash.
  spec hmac_sha256(key: Word[32]^64, key_len: Int, text: Word[32]^64, n: Int) -> Word[32]^8 {
    let k: Word[32]^16 = k0(key, key_len);
    let ki: Word[32]^16 = xor_pad(k, ipad());
    let ko: Word[32]^16 = xor_pad(k, opad());
    let inner: Word[32]^8 = sha256([
      ki[0], ki[1], ki[2], ki[3], ki[4], ki[5], ki[6], ki[7],
      ki[8], ki[9], ki[10], ki[11], ki[12], ki[13], ki[14], ki[15],
      text[0], text[1], text[2], text[3], text[4], text[5], text[6], text[7],
      text[8], text[9], text[10], text[11], text[12], text[13], text[14], text[15],
      text[16], text[17], text[18], text[19], text[20], text[21], text[22], text[23],
      text[24], text[25], text[26], text[27], text[28], text[29], text[30], text[31],
      text[32], text[33], text[34], text[35], text[36], text[37], text[38], text[39],
      text[40], text[41], text[42], text[43], text[44], text[45], text[46], text[47],
    ], 64 + n);
    sha256([
      ko[0], ko[1], ko[2], ko[3], ko[4], ko[5], ko[6], ko[7],
      ko[8], ko[9], ko[10], ko[11], ko[12], ko[13], ko[14], ko[15],
      inner[0], inner[1], inner[2], inner[3], inner[4], inner[5], inner[6], inner[7],
      0, 0, 0, 0, 0, 0, 0, 0,
      0, 0, 0, 0, 0, 0, 0, 0,
      0, 0, 0, 0, 0, 0, 0, 0,
      0, 0, 0, 0, 0, 0, 0, 0,
      0, 0, 0, 0, 0, 0, 0, 0,
    ], 96)
  }

  // The truncated output of FIPS 198-1 and RFC 2104 section 5: the
  // leftmost t = 128 bits of the MAC, RFC 4231's HMAC-SHA-256-128.
  spec leftmost_128(mac: Word[32]^8) -> Word[8]^16 {
    for i in 0..16 with out: Word[8]^16 = [0; 16] { out with [i] = be_bytes(mac[i / 4])[i % 4] }
  }
  spec words_8(w: Word[32]^8) -> Word[32]^64 {
    for i in 0..8 with b: Word[32]^64 = [0; 64] { b with [i] = w[i] }
  }

  spec words_1(w: Word[32]^1) -> Word[32]^64 { [0; 64] with [0] = w[0] }
  spec words_2(w: Word[32]^2) -> Word[32]^64 {
    for i in 0..2 with b: Word[32]^64 = [0; 64] { b with [i] = w[i] }
  }
  spec words_5(w: Word[32]^5) -> Word[32]^64 {
    for i in 0..5 with b: Word[32]^64 = [0; 64] { b with [i] = w[i] }
  }
  spec words_7(w: Word[32]^7) -> Word[32]^64 {
    for i in 0..7 with b: Word[32]^64 = [0; 64] { b with [i] = w[i] }
  }
  spec words_13(w: Word[32]^13) -> Word[32]^64 {
    for i in 0..13 with b: Word[32]^64 = [0; 64] { b with [i] = w[i] }
  }
  spec words_14(w: Word[32]^14) -> Word[32]^64 {
    for i in 0..14 with b: Word[32]^64 = [0; 64] { b with [i] = w[i] }
  }
  spec words_16(w: Word[32]^16) -> Word[32]^64 {
    for i in 0..16 with b: Word[32]^64 = [0; 64] { b with [i] = w[i] }
  }
  spec words_17(w: Word[32]^17) -> Word[32]^64 {
    for i in 0..17 with b: Word[32]^64 = [0; 64] { b with [i] = w[i] }
  }
  spec words_33(w: Word[32]^33) -> Word[32]^64 {
    for i in 0..33 with b: Word[32]^64 = [0; 64] { b with [i] = w[i] }
  }
  spec words_38(w: Word[32]^38) -> Word[32]^64 {
    for i in 0..38 with b: Word[32]^64 = [0; 64] { b with [i] = w[i] }
  }
  // RFC 4231 section 4.2, test case 1: key 0x0b repeated 20 times, data
  // "Hi There". The value as printed in Go's crypto/hmac/hmac_test.go,
  // which copies the RFC's SHA-256 cases (also Python's hmac).
  spec rfc4231_case_1() -> Word[32]^8 {
    hmac_sha256(
      words_5([0x0b0b0b0b, 0x0b0b0b0b, 0x0b0b0b0b, 0x0b0b0b0b, 0x0b0b0b0b]), 20,
      words_2([0x48692054, 0x68657265]), 8,
    )
  }
  spec rfc4231_case_1_expected() -> Word[32]^8 {
    [
      0xb0344c61, 0xd8db3853, 0x5ca8afce, 0xaf0bf12b, 0x881dc200, 0xc9833da7, 0x26e9376c, 0x2e32cff7,
    ]
  }

  // RFC 4231 section 4.3, test case 2: key "Jefe", data "what do ya want
  // for nothing?"; a key shorter than the block. The value as printed in
  // Botan's hmac.vec, [HMAC(SHA-256)], and Go's hmac_test.go (also hmac).
  spec rfc4231_case_2() -> Word[32]^8 {
    hmac_sha256(
      words_1([0x4a656665]), 4,
      words_7([
        0x77686174, 0x20646f20, 0x79612077, 0x616e7420, 0x666f7220, 0x6e6f7468, 0x696e673f,
      ]), 28,
    )
  }
  spec rfc4231_case_2_expected() -> Word[32]^8 {
    [
      0x5bdcc146, 0xbf60754e, 0x6a042426, 0x089575c7, 0x5a003f08, 0x9d273983, 0x9dec58b9, 0x64ec3843,
    ]
  }

  // RFC 4231 section 4.4, test case 3: key 0xaa repeated 20 times, data
  // 0xdd repeated 50 times. The value as printed in Go's hmac_test.go
  // (also hmac).
  spec rfc4231_case_3() -> Word[32]^8 {
    hmac_sha256(
      words_5([0xaaaaaaaa, 0xaaaaaaaa, 0xaaaaaaaa, 0xaaaaaaaa, 0xaaaaaaaa]), 20,
      words_13([
        0xdddddddd, 0xdddddddd, 0xdddddddd, 0xdddddddd, 0xdddddddd, 0xdddddddd, 0xdddddddd, 0xdddddddd,
        0xdddddddd, 0xdddddddd, 0xdddddddd, 0xdddddddd, 0xdddd0000,
      ]), 50,
    )
  }
  spec rfc4231_case_3_expected() -> Word[32]^8 {
    [
      0x773ea91e, 0x36800e46, 0x854db8eb, 0xd09181a7, 0x2959098b, 0x3ef8c122, 0xd9635514, 0xced565fe,
    ]
  }

  // RFC 4231 section 4.5, test case 4: key 0x01, 0x02, ..., 0x19 (25
  // bytes), data 0xcd repeated 50 times. The value as printed in Go's
  // hmac_test.go (also hmac).
  spec rfc4231_case_4() -> Word[32]^8 {
    hmac_sha256(
      words_7([
        0x01020304, 0x05060708, 0x090a0b0c, 0x0d0e0f10, 0x11121314, 0x15161718, 0x19000000,
      ]), 25,
      words_13([
        0xcdcdcdcd, 0xcdcdcdcd, 0xcdcdcdcd, 0xcdcdcdcd, 0xcdcdcdcd, 0xcdcdcdcd, 0xcdcdcdcd, 0xcdcdcdcd,
        0xcdcdcdcd, 0xcdcdcdcd, 0xcdcdcdcd, 0xcdcdcdcd, 0xcdcd0000,
      ]), 50,
    )
  }
  spec rfc4231_case_4_expected() -> Word[32]^8 {
    [
      0x82558a38, 0x9a443c0e, 0xa4cc8198, 0x99f2083a, 0x85f0faa3, 0xe578f807, 0x7a2e3ff4, 0x6729665b,
    ]
  }

  // RFC 4231 section 4.6, test case 5: key 0x0c repeated 20 times, data
  // "Test With Truncation"; the RFC prints only the MAC truncated to 128
  // bits. No fetched file carries this case, so both values are from
  // Python's hmac; the RFC's printed value is the 16 bytes.
  spec rfc4231_case_5() -> Word[32]^8 {
    hmac_sha256(
      words_5([0x0c0c0c0c, 0x0c0c0c0c, 0x0c0c0c0c, 0x0c0c0c0c, 0x0c0c0c0c]), 20,
      words_5([0x54657374, 0x20576974, 0x68205472, 0x756e6361, 0x74696f6e]), 20,
    )
  }
  spec rfc4231_case_5_expected() -> Word[32]^8 {
    [
      0xa3b61674, 0x73100ee0, 0x6e0c796c, 0x2955552b, 0xfa6f7c0a, 0x6a8aef8b, 0x93f860aa, 0xb0cd20c5,
    ]
  }
  spec rfc4231_case_5_truncated() -> Word[8]^16 { leftmost_128(rfc4231_case_5()) }
  spec rfc4231_case_5_truncated_expected() -> Word[8]^16 {
    [
      0xa3, 0xb6, 0x16, 0x74, 0x73, 0x10, 0x0e, 0xe0, 0x6e, 0x0c, 0x79, 0x6c, 0x29, 0x55, 0x55, 0x2b,
    ]
  }

  // RFC 4231 section 4.7, test case 6: key 0xaa repeated 131 times, longer
  // than the block, so it is hashed first; data "Test Using Larger Than
  // Block-Size Key - Hash Key First". The value as printed in Botan's
  // hmac.vec and Go's hmac_test.go (also hmac).
  spec rfc4231_case_6() -> Word[32]^8 {
    hmac_sha256(
      words_33([
        0xaaaaaaaa, 0xaaaaaaaa, 0xaaaaaaaa, 0xaaaaaaaa, 0xaaaaaaaa, 0xaaaaaaaa, 0xaaaaaaaa, 0xaaaaaaaa,
        0xaaaaaaaa, 0xaaaaaaaa, 0xaaaaaaaa, 0xaaaaaaaa, 0xaaaaaaaa, 0xaaaaaaaa, 0xaaaaaaaa, 0xaaaaaaaa,
        0xaaaaaaaa, 0xaaaaaaaa, 0xaaaaaaaa, 0xaaaaaaaa, 0xaaaaaaaa, 0xaaaaaaaa, 0xaaaaaaaa, 0xaaaaaaaa,
        0xaaaaaaaa, 0xaaaaaaaa, 0xaaaaaaaa, 0xaaaaaaaa, 0xaaaaaaaa, 0xaaaaaaaa, 0xaaaaaaaa, 0xaaaaaaaa,
        0xaaaaaa00,
      ]), 131,
      words_14([
        0x54657374, 0x20557369, 0x6e67204c, 0x61726765, 0x72205468, 0x616e2042, 0x6c6f636b, 0x2d53697a,
        0x65204b65, 0x79202d20, 0x48617368, 0x204b6579, 0x20466972, 0x73740000,
      ]), 54,
    )
  }
  spec rfc4231_case_6_expected() -> Word[32]^8 {
    [
      0x60e43159, 0x1ee0b67f, 0x0d8a26aa, 0xcbf5b77f, 0x8e0bc621, 0x3728c514, 0x0546040f, 0x0ee37f54,
    ]
  }

  // RFC 4231 section 4.8, test case 7: the same 131-byte key and 152 bytes
  // of data, "This is a test using a larger than block-size key and a
  // larger than block-size data. The key needs to be hashed before being
  // used by the HMAC algorithm." The value as printed in Botan's hmac.vec
  // and Go's hmac_test.go (also hmac).
  spec rfc4231_case_7() -> Word[32]^8 {
    hmac_sha256(
      words_33([
        0xaaaaaaaa, 0xaaaaaaaa, 0xaaaaaaaa, 0xaaaaaaaa, 0xaaaaaaaa, 0xaaaaaaaa, 0xaaaaaaaa, 0xaaaaaaaa,
        0xaaaaaaaa, 0xaaaaaaaa, 0xaaaaaaaa, 0xaaaaaaaa, 0xaaaaaaaa, 0xaaaaaaaa, 0xaaaaaaaa, 0xaaaaaaaa,
        0xaaaaaaaa, 0xaaaaaaaa, 0xaaaaaaaa, 0xaaaaaaaa, 0xaaaaaaaa, 0xaaaaaaaa, 0xaaaaaaaa, 0xaaaaaaaa,
        0xaaaaaaaa, 0xaaaaaaaa, 0xaaaaaaaa, 0xaaaaaaaa, 0xaaaaaaaa, 0xaaaaaaaa, 0xaaaaaaaa, 0xaaaaaaaa,
        0xaaaaaa00,
      ]), 131,
      words_38([
        0x54686973, 0x20697320, 0x61207465, 0x73742075, 0x73696e67, 0x2061206c, 0x61726765, 0x72207468,
        0x616e2062, 0x6c6f636b, 0x2d73697a, 0x65206b65, 0x7920616e, 0x64206120, 0x6c617267, 0x65722074,
        0x68616e20, 0x626c6f63, 0x6b2d7369, 0x7a652064, 0x6174612e, 0x20546865, 0x206b6579, 0x206e6565,
        0x64732074, 0x6f206265, 0x20686173, 0x68656420, 0x6265666f, 0x72652062, 0x65696e67, 0x20757365,
        0x64206279, 0x20746865, 0x20484d41, 0x4320616c, 0x676f7269, 0x74686d2e,
      ]), 152,
    )
  }
  spec rfc4231_case_7_expected() -> Word[32]^8 {
    [
      0x9b09ffa7, 0x1b942fcb, 0x27635fbc, 0xd5b0e944, 0xbfdc6364, 0x4f071393, 0x8a7f5153, 0x5c3a35e2,
    ]
  }

  // Project Wycheproof, testvectors_v1/hmac_sha256_test.json, tcId 171,
  // "long key": a 65-byte key, one byte over the block, and a 32-byte
  // message; the tag as printed there (also hmac).
  spec wycheproof_hmac_tc_171() -> Word[32]^8 {
    hmac_sha256(
      words_17([
        0x21178e26, 0xbc28ffc2, 0x7c06f762, 0xba190a62, 0x7075856d, 0x7ca6feab, 0x79ac6314, 0x9b17126e,
        0x34fd9e55, 0x90e0e90a, 0xac801df0, 0x9505d8af, 0x2dd0a270, 0x3b352c57, 0x3ac9d2cb, 0x063927f2,
        0xaf000000,
      ]), 65,
      words_8([
        0x7d5f1d6b, 0x993452b1, 0xb53a4375, 0x760d10a2, 0x0d46a0ab, 0x9ec3943f, 0xc4b07a2c, 0xe735e731,
      ]), 32,
    )
  }
  spec wycheproof_hmac_tc_171_expected() -> Word[32]^8 {
    [
      0xe542ac8a, 0xc8f364ba, 0xe4b7da8b, 0x7a0777df, 0x350f001d, 0xe4e8cfa2, 0xd9ef0b15, 0x019496ec,
    ]
  }

  // FIPS 198-1 steps 4 and 7 on RFC 4231 case 2. Key "Jefe" is shorter than
  // the block, so K0 is that key and then zeros. The first word of K0 xor
  // ipad is 0x4a656665 xor 0x36363636.
  spec rfc4231_case_2_inner_pad() -> Word[32]^16 {
    xor_pad(k0(words_1([0x4a656665]), 4), ipad())
  }
  spec rfc4231_case_2_inner_pad_expected() -> Word[32]^16 {
    [
      0x7c535053, 0x36363636, 0x36363636, 0x36363636, 0x36363636, 0x36363636, 0x36363636, 0x36363636,
      0x36363636, 0x36363636, 0x36363636, 0x36363636, 0x36363636, 0x36363636, 0x36363636, 0x36363636,
    ]
  }
  spec rfc4231_case_2_outer_pad() -> Word[32]^16 {
    xor_pad(k0(words_1([0x4a656665]), 4), opad())
  }
  spec rfc4231_case_2_outer_pad_expected() -> Word[32]^16 {
    [
      0x16393a39, 0x5c5c5c5c, 0x5c5c5c5c, 0x5c5c5c5c, 0x5c5c5c5c, 0x5c5c5c5c, 0x5c5c5c5c, 0x5c5c5c5c,
      0x5c5c5c5c, 0x5c5c5c5c, 0x5c5c5c5c, 0x5c5c5c5c, 0x5c5c5c5c, 0x5c5c5c5c, 0x5c5c5c5c, 0x5c5c5c5c,
    ]
  }

  // FIPS 198-1 steps 1 to 3 on RFC 4231 case 6: the key is 131 bytes, longer
  // than B, so K0 is SHA-256(key) and then eight zero words.
  spec rfc4231_case_6_k0() -> Word[32]^16 {
    k0(
      words_33([
        0xaaaaaaaa, 0xaaaaaaaa, 0xaaaaaaaa, 0xaaaaaaaa, 0xaaaaaaaa, 0xaaaaaaaa, 0xaaaaaaaa, 0xaaaaaaaa,
        0xaaaaaaaa, 0xaaaaaaaa, 0xaaaaaaaa, 0xaaaaaaaa, 0xaaaaaaaa, 0xaaaaaaaa, 0xaaaaaaaa, 0xaaaaaaaa,
        0xaaaaaaaa, 0xaaaaaaaa, 0xaaaaaaaa, 0xaaaaaaaa, 0xaaaaaaaa, 0xaaaaaaaa, 0xaaaaaaaa, 0xaaaaaaaa,
        0xaaaaaaaa, 0xaaaaaaaa, 0xaaaaaaaa, 0xaaaaaaaa, 0xaaaaaaaa, 0xaaaaaaaa, 0xaaaaaaaa, 0xaaaaaaaa,
        0xaaaaaa00,
      ]),
      131,
    )
  }
  spec rfc4231_case_6_k0_expected() -> Word[32]^16 {
    [
      0x45ad4b37, 0xc6e2fc0a, 0x2cfcc1b5, 0xda524132, 0xec707615, 0xc2cae1db, 0xbc43c97a, 0xa521db81,
      0, 0, 0, 0, 0, 0, 0, 0,
    ]
  }


  test "RFC 4231 case 2 inner and outer pads" {
    (rfc4231_case_2_inner_pad() == rfc4231_case_2_inner_pad_expected())
      && (rfc4231_case_2_outer_pad() == rfc4231_case_2_outer_pad_expected())
  }

  test "RFC 4231 case 6 hashes a key longer than the block" {
    rfc4231_case_6_k0() == rfc4231_case_6_k0_expected()
  }

  test "RFC 4231 section 4.2 case 1" {
    rfc4231_case_1() == rfc4231_case_1_expected()
  }

  test "RFC 4231 section 4.3 case 2" {
    rfc4231_case_2() == rfc4231_case_2_expected()
  }

  test "RFC 4231 section 4.4 case 3" {
    rfc4231_case_3() == rfc4231_case_3_expected()
  }

  test "RFC 4231 section 4.5 case 4" {
    rfc4231_case_4() == rfc4231_case_4_expected()
  }

  test "RFC 4231 section 4.6 case 5 and its 128-bit truncation" {
    (rfc4231_case_5() == rfc4231_case_5_expected())
      && (rfc4231_case_5_truncated() == rfc4231_case_5_truncated_expected())
  }

  test "RFC 4231 section 4.7 case 6" {
    rfc4231_case_6() == rfc4231_case_6_expected()
  }

  test "RFC 4231 section 4.8 case 7" {
    rfc4231_case_7() == rfc4231_case_7_expected()
  }

  test "Wycheproof hmac_sha256 tcId 171, key one byte over the block" {
    wycheproof_hmac_tc_171() == wycheproof_hmac_tc_171_expected()
  }
}
```

### HKDF, RFC 5869

HKDF as RFC 5869 (2010) sections 2.2 and 2.3 write it. The pseudorandom
function is HMAC-SHA-256 of the previous section. The listing restates that
HMAC, and the SHA-256 it calls, because a module has no imports. `orangec
test` accepts the three SHA-256 cases of appendix A, each as Extract and as
Expand. The RFC 4231 cases stay in the HMAC section. One evaluation budget
is $1\,048\,576$ steps, and the two groups do not both fit in it.
`algorithms/hmac-hkdf/hmac-hkdf-rfc5869.or` is the same split.

#### 1. Extract (section 2.2)

HashLen is 32. Extract is

$$\mathrm{PRK} = \mathrm{HMAC}(\mathrm{salt}, \mathrm{IKM}).$$

The salt is the HMAC key and the IKM is the text. A salt that is not
provided is a string of HashLen zero bytes. `hkdf_extract` treats length 0
as that string: the buffer is already zero, and the length passed to HMAC
is 32. A salt longer than the SHA-256 block is hashed first, by the same
$K_0$ rule as HMAC.

#### 2. Expand (section 2.3)

For a context string info and a length $L$ at most $255 \times 32$, with
$N = \lceil L / 32 \rceil$ and the counter $i$ a single octet,

$$T(1) = \mathrm{HMAC}(\mathrm{PRK}, \mathrm{info} \parallel \mathtt{0x01}), \qquad T(i) = \mathrm{HMAC}(\mathrm{PRK}, T(i-1) \parallel \mathrm{info} \parallel i)\ (i > 1).$$

OKM is the first $L$ octets of $T(1) \parallel \cdots \parallel T(N)$. The
listing writes two output lengths, because an array length is part of its
type. `hkdf_expand_42` is $L = 42$, so $N = 2$. `hkdf_expand_82` is
$L = 82$, so $N = 3$. $T(i-1)$ is eight words, so info starts at word 8 of
the data buffer, and `put_byte` writes the counter at byte $32 + \mathrm{len}(\mathrm{info})$.
For $i = 1$ the counter is written at byte $\mathrm{len}(\mathrm{info})$ of
info itself. An empty info is a zero buffer at length 0, so $T(1)$ is
HMAC of the single byte `0x01`.

For $i > 1$ the inner message is the 64-byte pad, the 32-byte block
$T(i-1)$, info, and one counter byte. SHA-256 padding needs nine further
bytes, and the buffer is 256 bytes, so info in this listing is at most 150
bytes. Appendix A.2, at 80 bytes of info, sits inside that bound.

#### 3. Known Answers (appendix A)

| Case | Salt, IKM, info, $L$ | PRK |
| :--- | :--- | :--- |
| A.1 | salt `000102030405060708090a0b0c`, IKM `0x0b` repeated 22 times, info `f0` through `f9`, $L = 42$ | `077709362c2e32df0ddc3f0dc47bba6390b6c73bb50f9c3122ec844ad7c2b3e5` |
| A.2 | salt `60` through `af` (80 bytes), IKM `00` through `4f` (80 bytes), info `b0` through `ff`, $L = 82$ | `06a6b88c5853361a06104c9ceb35b45cef760014904671014a193f40c15fc244` |
| A.3 | salt empty, the IKM of A.1, info empty, $L = 42$ | `19ef24a32c717b167f33a91d6f648bdf96596776afdb6377ac434c1c293ccb04` |

| Case | OKM |
| :--- | :--- |
| A.1 | `3cb25f25faacd57a90434f64d0362f2a2d2d0a90cf1a5a4c5db02d56ecc4c5bf34007208d5b887185865` |
| A.2 | `b11e398dc80327a1c8e7f78c596a49344f012eda2d4efad8a050cc4c19afa97c59045a99cac7827271cb41c65e590e09da3275600c2f09b8367793a9aca3db71cc30c58179ec3e87c14c01d5c1f3434f1d87` |
| A.3 | `8da4e775a563c18f715f802a063c5a31b8a11f5c5ee1879ec3454e5f3c738d2d9d201395faa4b61a96c8` |

A.2's salt is longer than the block, so Extract hashes it before the inner
pad. A.3's salt length is 0, so Extract uses 32 zero bytes.

#### 4. Compiler-Checked Transcription

```orange
// HKDF-SHA-256 as RFC 5869 sections 2.2 and 2.3 write it, with HMAC-SHA-256
// as the pseudorandom function. HMAC and SHA-256 are restated because a
// module has no imports. The tests are the PRK and OKM of RFC 5869 appendix
// A. The RFC 4231 HMAC cases are a separate transcription: one file's
// 1,048,576-step budget does not hold both.
edition 2026;
module hkdf_spec {
  // A word to its bytes, big-endian (FIPS 180-4 section 3.1); the inputs
  // are already words, so only the output direction is needed.
  spec be_bytes(x: Word[32]) -> Word[8]^4 {
    [(x >> 24) as Word[8], (x >> 16) as Word[8], (x >> 8) as Word[8], x as Word[8]]
  }

  // FIPS 180-4 section 4.1.2: the SHA-256 functions.
  spec ch(x: Word[32], y: Word[32], z: Word[32]) -> Word[32] { (x & y) ^ (~x & z) }
  spec maj(x: Word[32], y: Word[32], z: Word[32]) -> Word[32] { (x & y) ^ (x & z) ^ (y & z) }
  spec big_sigma0(x: Word[32]) -> Word[32] { (x >>> 2) ^ (x >>> 13) ^ (x >>> 22) }
  spec big_sigma1(x: Word[32]) -> Word[32] { (x >>> 6) ^ (x >>> 11) ^ (x >>> 25) }
  spec small_sigma0(x: Word[32]) -> Word[32] { (x >>> 7) ^ (x >>> 18) ^ (x >> 3) }
  spec small_sigma1(x: Word[32]) -> Word[32] { (x >>> 17) ^ (x >>> 19) ^ (x >> 10) }

  // FIPS 180-4 section 4.2.2: K{256}, the first 32 bits of the fractional
  // parts of the cube roots of the first 64 primes.
  spec round_constants() -> Word[32]^64 {
    [
      0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
      0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
      0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
      0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
      0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
      0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
      0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
      0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
    ]
  }

  // FIPS 180-4 section 5.3.3: H(0), the first 32 bits of the fractional parts
  // of the square roots of the first eight primes.
  spec initial_hash() -> Word[32]^8 {
    [
      0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a,
      0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
    ]
  }

  // FIPS 180-4 section 6.2.2, step 1: the message schedule W_0 through W_63.
  spec schedule(m: Word[32]^16) -> Word[32]^64 {
    let head: Word[32]^64 = for t in 0..16 with w: Word[32]^64 = [0; 64] { w with [t] = m[t] };
    for t in 16..64 with w: Word[32]^64 = head {
      w with [t] = small_sigma1(w[t - 2]) + w[t - 7] + small_sigma0(w[t - 15]) + w[t - 16]
    }
  }

  // FIPS 180-4 section 6.2.2, step 3: one round on the working variables a
  // through h, carried as v = [a, b, c, d, e, f, g, h].
  spec round(v: Word[32]^8, k: Word[32], w: Word[32]) -> Word[32]^8 {
    let t1: Word[32] = v[7] + big_sigma1(v[4]) + ch(v[4], v[5], v[6]) + k + w;
    let t2: Word[32] = big_sigma0(v[0]) + maj(v[0], v[1], v[2]);
    [t1 + t2, v[0], v[1], v[2], v[3] + t1, v[4], v[5], v[6]]
  }

  // FIPS 180-4 section 6.2.2, steps 1 through 4: one block, H(i-1) to H(i).
  spec compress(h: Word[32]^8, m: Word[32]^16) -> Word[32]^8 {
    let w: Word[32]^64 = schedule(m);
    let k: Word[32]^64 = round_constants();
    let v: Word[32]^8 = for t in 0..64 with v: Word[32]^8 = h { round(v, k[t], w[t]) };
    for i in 0..8 with out: Word[32]^8 = v { out with [i] = out[i] + h[i] }
  }

  // Byte p of the buffer, 0 <= p < 256, set to v by or: byte p lives in
  // word p / 4, at the shift that p % 4 selects. The shift amount must be a
  // literal, so the four positions are a conditional. This writes the
  // padding bit of SHA-256 and the counter octet of HKDF-Expand, the two
  // bytes whose position depends on a length.
  spec put_byte(m: Word[32]^64, p: Int, v: Word[8]) -> Word[32]^64 {
    let shifted: Word[32] = if (p % 4) == 0 { (v as Word[32]) << 24 }
      else if (p % 4) == 1 { (v as Word[32]) << 16 }
      else if (p % 4) == 2 { (v as Word[32]) << 8 }
      else { v as Word[32] };
    for i in 0..64 with out: Word[32]^64 = m {
      if i == (p / 4) { out with [i] = out[i] | shifted } else { out }
    }
  }

  // FIPS 180-4 section 5.1.1 for a message of n bytes, n <= 247: the bit 1
  // (the byte 0x80, since every message is whole bytes) after the message,
  // k zero bits, and the length l = 8n as a 64-bit big-endian integer at
  // the end of block N, N = (n + 72) / 64 = ceiling((n + 9) / 64). The high
  // word of l is zero for every n here, so only the last word is written.
  spec pad(m: Word[32]^64, n: Int) -> Word[32]^64 {
    let marked: Word[32]^64 = put_byte(m, n, 0x80);
    let blocks: Int = (n + 72) / 64;
    for i in 0..4 with w: Word[32]^64 = marked {
      if (i + 1) == blocks { w with [16 * i + 15] = (8 * n) as Word[32] } else { w }
    }
  }

  // FIPS 180-4 section 6.2.2, "for i = 1 to N": the N blocks of the padded
  // message in order, each sixteen words of the buffer, from H(0).
  spec sha256(m: Word[32]^64, n: Int) -> Word[32]^8 {
    let w: Word[32]^64 = pad(m, n);
    let blocks: Int = (n + 72) / 64;
    for i in 0..4 with h: Word[32]^8 = initial_hash() {
      if i < blocks {
        compress(h, [
          w[16 * i], w[16 * i + 1], w[16 * i + 2], w[16 * i + 3],
          w[16 * i + 4], w[16 * i + 5], w[16 * i + 6], w[16 * i + 7],
          w[16 * i + 8], w[16 * i + 9], w[16 * i + 10], w[16 * i + 11],
          w[16 * i + 12], w[16 * i + 13], w[16 * i + 14], w[16 * i + 15],
        ])
      } else { h }
    }
  }

  // FIPS 198-1 section 4: B = 64, the block size of SHA-256 in bytes, and
  // the pads ipad = 0x36 and opad = 0x5c repeated B times, here as words.
  spec ipad() -> Word[32] { 0x36363636 }
  spec opad() -> Word[32] { 0x5c5c5c5c }

  // FIPS 198-1 section 4, steps 1 to 3: K0, the key brought to B bytes. A
  // key of B bytes is used as it is; a longer key is hashed to L = 32 bytes
  // and zeros are appended; a shorter key has zeros appended. The buffer
  // is zero beyond the key, so its first sixteen words are the padded key.
  spec k0(key: Word[32]^64, key_len: Int) -> Word[32]^16 {
    let hashed: Word[32]^8 = if key_len > 64 { sha256(key, key_len) } else { [0; 8] };
    if key_len > 64 {
      [hashed[0], hashed[1], hashed[2], hashed[3], hashed[4], hashed[5], hashed[6], hashed[7],
       0, 0, 0, 0, 0, 0, 0, 0]
    } else {
      [key[0], key[1], key[2], key[3], key[4], key[5], key[6], key[7],
       key[8], key[9], key[10], key[11], key[12], key[13], key[14], key[15]]
    }
  }

  // Steps 4 and 7: K0 xor ipad and K0 xor opad.
  spec xor_pad(k: Word[32]^16, pad_word: Word[32]) -> Word[32]^16 {
    for i in 0..16 with out: Word[32]^16 = k { out with [i] = k[i] ^ pad_word }
  }

  // FIPS 198-1 section 4, steps 4 to 9, and section 5:
  // HMAC(K, text) = H((K0 xor opad) || H((K0 xor ipad) || text)).
  // Step 5 appends the text, of n <= 183 bytes, to the 64-byte inner pad
  // (so the inner message fits four blocks); step 8 appends the 32-byte
  // inner hash to the outer pad, a 96-byte message. Truncation to t bytes
  // is left to the caller (`leftmost_128`); the full L = 32 bytes are
  // returned as the eight words of the hash.
  spec hmac_sha256(key: Word[32]^64, key_len: Int, text: Word[32]^64, n: Int) -> Word[32]^8 {
    let k: Word[32]^16 = k0(key, key_len);
    let ki: Word[32]^16 = xor_pad(k, ipad());
    let ko: Word[32]^16 = xor_pad(k, opad());
    let inner: Word[32]^8 = sha256([
      ki[0], ki[1], ki[2], ki[3], ki[4], ki[5], ki[6], ki[7],
      ki[8], ki[9], ki[10], ki[11], ki[12], ki[13], ki[14], ki[15],
      text[0], text[1], text[2], text[3], text[4], text[5], text[6], text[7],
      text[8], text[9], text[10], text[11], text[12], text[13], text[14], text[15],
      text[16], text[17], text[18], text[19], text[20], text[21], text[22], text[23],
      text[24], text[25], text[26], text[27], text[28], text[29], text[30], text[31],
      text[32], text[33], text[34], text[35], text[36], text[37], text[38], text[39],
      text[40], text[41], text[42], text[43], text[44], text[45], text[46], text[47],
    ], 64 + n);
    sha256([
      ko[0], ko[1], ko[2], ko[3], ko[4], ko[5], ko[6], ko[7],
      ko[8], ko[9], ko[10], ko[11], ko[12], ko[13], ko[14], ko[15],
      inner[0], inner[1], inner[2], inner[3], inner[4], inner[5], inner[6], inner[7],
      0, 0, 0, 0, 0, 0, 0, 0,
      0, 0, 0, 0, 0, 0, 0, 0,
      0, 0, 0, 0, 0, 0, 0, 0,
      0, 0, 0, 0, 0, 0, 0, 0,
      0, 0, 0, 0, 0, 0, 0, 0,
    ], 96)
  }

  // The truncated output of FIPS 198-1 and RFC 2104 section 5: the
  // leftmost t = 128 bits of the MAC, RFC 4231's HMAC-SHA-256-128.
  spec leftmost_128(mac: Word[32]^8) -> Word[8]^16 {
    for i in 0..16 with out: Word[8]^16 = [0; 16] { out with [i] = be_bytes(mac[i / 4])[i % 4] }
  }

  // RFC 5869 section 2.2: PRK = HMAC-Hash(salt, IKM), HashLen = 32. A salt
  // that is not provided is a string of HashLen zeros; the empty salt's
  // buffer is all zeros, so it is that string at length 32.
  spec hkdf_extract(salt: Word[32]^64, salt_len: Int, ikm: Word[32]^64, ikm_len: Int) -> Word[32]^8 {
    let salt_length: Int = if salt_len == 0 { 32 } else { salt_len };
    hmac_sha256(salt, salt_length, ikm, ikm_len)
  }

  // RFC 5869 section 2.3: T(1) = HMAC-Hash(PRK, info || 0x01) and
  // T(i) = HMAC-Hash(PRK, T(i-1) || info || i) for i > 1, i a single octet.
  // T(i-1) is eight whole words, so info starts at word 8 of the data and
  // only the counter octet, at byte 32 + len(info), falls inside a word.
  spec t_i(prk: Word[32]^64, previous: Word[32]^8, info: Word[32]^64, info_len: Int, i: Int) -> Word[32]^8 {
    let data: Word[32]^64 = if i == 1 { info } else {
      [
        previous[0], previous[1], previous[2], previous[3],
        previous[4], previous[5], previous[6], previous[7],
        info[0], info[1], info[2], info[3], info[4], info[5], info[6], info[7],
        info[8], info[9], info[10], info[11], info[12], info[13], info[14], info[15],
        info[16], info[17], info[18], info[19], info[20], info[21], info[22], info[23],
        info[24], info[25], info[26], info[27], info[28], info[29], info[30], info[31],
        info[32], info[33], info[34], info[35], info[36], info[37], info[38], info[39],
        0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0,
      ]
    };
    let data_len: Int = if i == 1 { info_len } else { 32 + info_len };
    hmac_sha256(prk, 32, put_byte(data, data_len, i as Word[8]), data_len + 1)
  }

  // RFC 5869 section 2.3 with L = 42: N = ceiling(42 / 32) = 2, and OKM is
  // the first 42 octets of T(1) || T(2).
  spec hkdf_expand_42(prk: Word[32]^8, info: Word[32]^64, info_len: Int) -> Word[8]^42 {
    let key: Word[32]^64 = words_8(prk);
    let t1: Word[32]^8 = t_i(key, [0; 8], info, info_len, 1);
    let t2: Word[32]^8 = t_i(key, t1, info, info_len, 2);
    let okm: Word[32]^16 = [
      t1[0], t1[1], t1[2], t1[3], t1[4], t1[5], t1[6], t1[7],
      t2[0], t2[1], t2[2], t2[3], t2[4], t2[5], t2[6], t2[7],
    ];
    for i in 0..42 with out: Word[8]^42 = [0; 42] { out with [i] = be_bytes(okm[i / 4])[i % 4] }
  }

  // RFC 5869 section 2.3 with L = 82: N = 3, OKM the first 82 octets of
  // T(1) || T(2) || T(3).
  spec hkdf_expand_82(prk: Word[32]^8, info: Word[32]^64, info_len: Int) -> Word[8]^82 {
    let key: Word[32]^64 = words_8(prk);
    let t1: Word[32]^8 = t_i(key, [0; 8], info, info_len, 1);
    let t2: Word[32]^8 = t_i(key, t1, info, info_len, 2);
    let t3: Word[32]^8 = t_i(key, t2, info, info_len, 3);
    let okm: Word[32]^24 = [
      t1[0], t1[1], t1[2], t1[3], t1[4], t1[5], t1[6], t1[7],
      t2[0], t2[1], t2[2], t2[3], t2[4], t2[5], t2[6], t2[7],
      t3[0], t3[1], t3[2], t3[3], t3[4], t3[5], t3[6], t3[7],
    ];
    for i in 0..82 with out: Word[8]^82 = [0; 82] { out with [i] = be_bytes(okm[i / 4])[i % 4] }
  }

  // The words of a byte string placed at the start of an otherwise zero
  // buffer, one spec per word count in use, since an array's length is
  // part of its type. A string of n bytes is ceiling(n / 4) words, the last
  // one zero-padded, and travels with n. HKDF-Expand keys HMAC with the
  // eight words of the PRK through `words_8`; the rest serve the vectors.
  spec words_8(w: Word[32]^8) -> Word[32]^64 {
    for i in 0..8 with b: Word[32]^64 = [0; 64] { b with [i] = w[i] }
  }
  spec words_3(w: Word[32]^3) -> Word[32]^64 {
    for i in 0..3 with b: Word[32]^64 = [0; 64] { b with [i] = w[i] }
  }
  spec words_4(w: Word[32]^4) -> Word[32]^64 {
    for i in 0..4 with b: Word[32]^64 = [0; 64] { b with [i] = w[i] }
  }
  spec words_6(w: Word[32]^6) -> Word[32]^64 {
    for i in 0..6 with b: Word[32]^64 = [0; 64] { b with [i] = w[i] }
  }
  spec words_20(w: Word[32]^20) -> Word[32]^64 {
    for i in 0..20 with b: Word[32]^64 = [0; 64] { b with [i] = w[i] }
  }

  // RFC 5869 appendix A.1, test case 1: IKM 0x0b repeated 22 times, salt
  // 0x00 to 0x0c, info 0xf0 to 0xf9, L = 42. PRK as printed in Botan's
  // hkdf.vec, [HKDF-Extract(HMAC(SHA-256))], OpenSSL's evpkdf_hkdf.txt and
  // pyca/cryptography's rfc-5869-HKDF-SHA256.txt; OKM as printed in the
  // same three files, Botan's under [HKDF(HMAC(SHA-256))], and in
  // Wycheproof's hkdf_sha256_test.json tcId 1 (also cryptography's HKDF).
  spec rfc5869_a_1_prk() -> Word[32]^8 {
    hkdf_extract(
      words_4([0x00010203, 0x04050607, 0x08090a0b, 0x0c000000]), 13,
      words_6([0x0b0b0b0b, 0x0b0b0b0b, 0x0b0b0b0b, 0x0b0b0b0b, 0x0b0b0b0b, 0x0b0b0000]), 22,
    )
  }
  spec rfc5869_a_1_prk_expected() -> Word[32]^8 {
    [
      0x07770936, 0x2c2e32df, 0x0ddc3f0d, 0xc47bba63, 0x90b6c73b, 0xb50f9c31, 0x22ec844a, 0xd7c2b3e5,
    ]
  }
  spec rfc5869_a_1_okm() -> Word[8]^42 {
    hkdf_expand_42(rfc5869_a_1_prk(), words_3([0xf0f1f2f3, 0xf4f5f6f7, 0xf8f90000]), 10)
  }
  spec rfc5869_a_1_okm_expected() -> Word[8]^42 {
    [
      0x3c, 0xb2, 0x5f, 0x25, 0xfa, 0xac, 0xd5, 0x7a, 0x90, 0x43, 0x4f, 0x64, 0xd0, 0x36, 0x2f, 0x2a,
      0x2d, 0x2d, 0x0a, 0x90, 0xcf, 0x1a, 0x5a, 0x4c, 0x5d, 0xb0, 0x2d, 0x56, 0xec, 0xc4, 0xc5, 0xbf,
      0x34, 0x00, 0x72, 0x08, 0xd5, 0xb8, 0x87, 0x18, 0x58, 0x65,
    ]
  }

  // RFC 5869 appendix A.2, test case 2: IKM 0x00 to 0x4f, salt 0x60 to
  // 0xaf and info 0xb0 to 0xff, 80 bytes each, L = 82. The salt is longer
  // than the block, so HMAC hashes it first, and N = 3. PRK and OKM from
  // the same files as A.1 (Wycheproof tcId 3).
  spec rfc5869_a_2_prk() -> Word[32]^8 {
    hkdf_extract(
      words_20([
        0x60616263, 0x64656667, 0x68696a6b, 0x6c6d6e6f, 0x70717273, 0x74757677, 0x78797a7b, 0x7c7d7e7f,
        0x80818283, 0x84858687, 0x88898a8b, 0x8c8d8e8f, 0x90919293, 0x94959697, 0x98999a9b, 0x9c9d9e9f,
        0xa0a1a2a3, 0xa4a5a6a7, 0xa8a9aaab, 0xacadaeaf,
      ]), 80,
      words_20([
        0x00010203, 0x04050607, 0x08090a0b, 0x0c0d0e0f, 0x10111213, 0x14151617, 0x18191a1b, 0x1c1d1e1f,
        0x20212223, 0x24252627, 0x28292a2b, 0x2c2d2e2f, 0x30313233, 0x34353637, 0x38393a3b, 0x3c3d3e3f,
        0x40414243, 0x44454647, 0x48494a4b, 0x4c4d4e4f,
      ]), 80,
    )
  }
  spec rfc5869_a_2_prk_expected() -> Word[32]^8 {
    [
      0x06a6b88c, 0x5853361a, 0x06104c9c, 0xeb35b45c, 0xef760014, 0x90467101, 0x4a193f40, 0xc15fc244,
    ]
  }
  spec rfc5869_a_2_okm() -> Word[8]^82 {
    hkdf_expand_82(
      rfc5869_a_2_prk(),
      words_20([
        0xb0b1b2b3, 0xb4b5b6b7, 0xb8b9babb, 0xbcbdbebf, 0xc0c1c2c3, 0xc4c5c6c7, 0xc8c9cacb, 0xcccdcecf,
        0xd0d1d2d3, 0xd4d5d6d7, 0xd8d9dadb, 0xdcdddedf, 0xe0e1e2e3, 0xe4e5e6e7, 0xe8e9eaeb, 0xecedeeef,
        0xf0f1f2f3, 0xf4f5f6f7, 0xf8f9fafb, 0xfcfdfeff,
      ]),
      80,
    )
  }
  spec rfc5869_a_2_okm_expected() -> Word[8]^82 {
    [
      0xb1, 0x1e, 0x39, 0x8d, 0xc8, 0x03, 0x27, 0xa1, 0xc8, 0xe7, 0xf7, 0x8c, 0x59, 0x6a, 0x49, 0x34,
      0x4f, 0x01, 0x2e, 0xda, 0x2d, 0x4e, 0xfa, 0xd8, 0xa0, 0x50, 0xcc, 0x4c, 0x19, 0xaf, 0xa9, 0x7c,
      0x59, 0x04, 0x5a, 0x99, 0xca, 0xc7, 0x82, 0x72, 0x71, 0xcb, 0x41, 0xc6, 0x5e, 0x59, 0x0e, 0x09,
      0xda, 0x32, 0x75, 0x60, 0x0c, 0x2f, 0x09, 0xb8, 0x36, 0x77, 0x93, 0xa9, 0xac, 0xa3, 0xdb, 0x71,
      0xcc, 0x30, 0xc5, 0x81, 0x79, 0xec, 0x3e, 0x87, 0xc1, 0x4c, 0x01, 0xd5, 0xc1, 0xf3, 0x43, 0x4f,
      0x1d, 0x87,
    ]
  }

  // RFC 5869 appendix A.3, test case 3: the IKM of A.1 with the salt and
  // info both empty, L = 42; the salt becomes 32 zero bytes. PRK and OKM
  // from the same files as A.1 (Wycheproof tcId 2).
  spec rfc5869_a_3_prk() -> Word[32]^8 {
    hkdf_extract(
      [0; 64], 0,
      words_6([0x0b0b0b0b, 0x0b0b0b0b, 0x0b0b0b0b, 0x0b0b0b0b, 0x0b0b0b0b, 0x0b0b0000]), 22,
    )
  }
  spec rfc5869_a_3_prk_expected() -> Word[32]^8 {
    [
      0x19ef24a3, 0x2c717b16, 0x7f33a91d, 0x6f648bdf, 0x96596776, 0xafdb6377, 0xac434c1c, 0x293ccb04,
    ]
  }
  spec rfc5869_a_3_okm() -> Word[8]^42 { hkdf_expand_42(rfc5869_a_3_prk(), [0; 64], 0) }
  spec rfc5869_a_3_okm_expected() -> Word[8]^42 {
    [
      0x8d, 0xa4, 0xe7, 0x75, 0xa5, 0x63, 0xc1, 0x8f, 0x71, 0x5f, 0x80, 0x2a, 0x06, 0x3c, 0x5a, 0x31,
      0xb8, 0xa1, 0x1f, 0x5c, 0x5e, 0xe1, 0x87, 0x9e, 0xc3, 0x45, 0x4e, 0x5f, 0x3c, 0x73, 0x8d, 0x2d,
      0x9d, 0x20, 0x13, 0x95, 0xfa, 0xa4, 0xb6, 0x1a, 0x96, 0xc8,
    ]
  }

  test "RFC 5869 A.1 extract and expand" {
    (rfc5869_a_1_prk() == rfc5869_a_1_prk_expected())
      && (rfc5869_a_1_okm() == rfc5869_a_1_okm_expected())
  }

  test "RFC 5869 A.2 extract and expand, L = 82" {
    (rfc5869_a_2_prk() == rfc5869_a_2_prk_expected())
      && (rfc5869_a_2_okm() == rfc5869_a_2_okm_expected())
  }

  test "RFC 5869 A.3 empty salt and empty info" {
    (rfc5869_a_3_prk() == rfc5869_a_3_prk_expected())
      && (rfc5869_a_3_okm() == rfc5869_a_3_okm_expected())
  }
}
```

### AEAD, RFC 8439

AEAD_CHACHA20_POLY1305 as RFC 8439 section 2.8 writes it. ChaCha20 is §50
and Poly1305 is §52. The listing restates both, because a module has no
imports. Poly1305 here is arithmetic on `Int` modulo $2^{130} - 5$, with
`%` after the product: the field of §52. `orangec test` accepts six tests:
the one-time keys of sections 2.6.2 and 2.8.2, the section 2.8.2 seal, its
opening, a tag whose last bit is flipped, and appendix A.5.

#### 1. The One-Time Key (section 2.6)

`poly1305_key_gen` keeps the first 32 bytes of ChaCha20 block 0 and discards
the rest. The ciphertext uses the key stream from counter 1 upward. Block 0
is not mixed into the ciphertext.

Section 2.6.2 uses the section 2.8.2 key and the nonce
`000000000001020304050607`. Its one-time key is
`8ad5a08b905f81cc815040274ab29471a833b637e3fd0da508dbb8e2fdd1a646`.

Section 2.8.2 uses the nonce `070000004041424344454647`. Its one-time key is
`7bac2b252db447af09b67a55a4e955840ae1d6731075d9eb2a9375783ed553ff`.

#### 2. The MAC Input (section 2.8)

$\mathrm{pad16}$ appends zeros until the length is a multiple of 16. The
MAC input is

$$\mathrm{aad} \parallel \mathrm{pad16}(\mathrm{aad}) \parallel C \parallel \mathrm{pad16}(C) \parallel \mathrm{le64}(|\mathrm{aad}|) \parallel \mathrm{le64}(|C|).$$

The tag is Poly1305 of that string under the one-time key. The listing
absorbs each 16-byte block as it is formed, rather than allocating one array
for the whole padded string. The lengths it writes are 12 bytes of
additional data with 114 bytes of ciphertext (section 2.8.2), 12 bytes with
265 bytes (appendix A.5: a 256-byte head under counters 1 through 4, then a
9-byte tail under counter 5), and 8 bytes with 47 bytes. The 47-byte
function is the length a Wycheproof vector uses. That vector is not a test
in this section.

#### 3. Seal and Open

The seal is the ciphertext followed by the 16-byte tag. Opening recomputes
the tag and compares it with the received tag, one byte at a time. The
plaintext is released only when every byte matches. Otherwise the result is
zeros of the plaintext's length. The comparison is a `Bool` fold. The spec
stratum has no timing, so the fold is the accept-or-reject check the tests
run, and it is not a claim about a constant-time comparison.

#### 4. Known Answers

Section 2.8.2 encrypts the 114-byte sentence that begins "Ladies and
Gentlemen of the class of '99", under key `80818283` through `9f` and
additional data `50515253c0c1c2c3c4c5c6c7`. The ciphertext begins
`d31a8d34648e60db7b86afbc53ef7ec2`. The tag is
`1ae10b594f09e26a7e902ecbd0600691`. Replacing the last tag byte `0x91` by
`0x90` makes verification false, and the opened result is 114 zero bytes.

Appendix A.5 uses key `1c9240a5` through `75c0`, nonce
`000000000102030405060708`, and additional data
`f33388860000000000004e91`. The tag is
`eead9d67890cbb22392336fea1851f38`. The plaintext begins "Internet-Drafts
are draft documents". Its last nine bytes are `726573732e2fe2809d`.

#### 5. Compiler-Checked Transcription

AEAD_XCHACHA20_POLY1305 is in
`algorithms/chacha20-poly1305/chacha20-poly1305.or`. This listing is
AEAD_CHACHA20_POLY1305.

```orange
// AEAD_CHACHA20_POLY1305 as RFC 8439 section 2.8 writes it. ChaCha20
// (sections 2.1 through 2.4) and Poly1305 (section 2.5) are restated because
// a module has no imports. The tests are the section 2.6.2 one-time key, the
// section 2.8.2 seal, open, and rejected tag, and appendix A.5.
edition 2026;
module aead_spec {
  // Section 2.1: the quarter round on four 32-bit words a, b, c, d.
  spec quarter_round(a: Word[32], b: Word[32], c: Word[32], d: Word[32]) -> Word[32]^4 {
    let a1: Word[32] = a + b;
    let d1: Word[32] = (d ^ a1) <<< 16;
    let c1: Word[32] = c + d1;
    let b1: Word[32] = (b ^ c1) <<< 12;
    let a2: Word[32] = a1 + b1;
    let d2: Word[32] = (d1 ^ a2) <<< 8;
    let c2: Word[32] = c1 + d2;
    let b2: Word[32] = (b1 ^ c2) <<< 7;
    [a2, b2, c2, d2]
  }

  // Section 2.3: inner_block, a column round QUARTERROUND(0, 4, 8, 12)
  // through (3, 7, 11, 15) followed by a diagonal round QUARTERROUND(0, 5,
  // 10, 15) through (3, 4, 9, 14). Each quarter round returns its four words,
  // and the state is rebuilt as one literal.
  spec inner_block(x: Word[32]^16) -> Word[32]^16 {
    let q0: Word[32]^4 = quarter_round(x[0], x[4], x[8], x[12]);
    let q1: Word[32]^4 = quarter_round(x[1], x[5], x[9], x[13]);
    let q2: Word[32]^4 = quarter_round(x[2], x[6], x[10], x[14]);
    let q3: Word[32]^4 = quarter_round(x[3], x[7], x[11], x[15]);
    let d0: Word[32]^4 = quarter_round(q0[0], q1[1], q2[2], q3[3]);
    let d1: Word[32]^4 = quarter_round(q1[0], q2[1], q3[2], q0[3]);
    let d2: Word[32]^4 = quarter_round(q2[0], q3[1], q0[2], q1[3]);
    let d3: Word[32]^4 = quarter_round(q3[0], q0[1], q1[2], q2[3]);
    [
      d0[0], d1[0], d2[0], d3[0],
      d3[1], d0[1], d1[1], d2[1],
      d2[2], d3[2], d0[2], d1[2],
      d1[3], d2[3], d3[3], d0[3],
    ]
  }

  spec load_le32(b0: Word[8], b1: Word[8], b2: Word[8], b3: Word[8]) -> Word[32] {
    (b0 as Word[32]) | ((b1 as Word[32]) << 8) | ((b2 as Word[32]) << 16)
      | ((b3 as Word[32]) << 24)
  }

  spec le_bytes(x: Word[32]) -> Word[8]^4 {
    [x as Word[8], (x >> 8) as Word[8], (x >> 16) as Word[8], (x >> 24) as Word[8]]
  }

  // Section 2.8: num_to_8_le_bytes, a length as eight little-endian bytes.
  spec num_to_8_le_bytes(x: Word[64]) -> Word[8]^8 {
    [
      x as Word[8], (x >> 8) as Word[8], (x >> 16) as Word[8], (x >> 24) as Word[8],
      (x >> 32) as Word[8], (x >> 40) as Word[8], (x >> 48) as Word[8], (x >> 56) as Word[8],
    ]
  }

  // Section 2.3: the initial state. Words 0 through 3 are the constants
  // "expand 32-byte k", words 4 through 11 the key, word 12 the block
  // counter, and words 13 through 15 the nonce, each read little-endian.
  spec initial_state(key: Word[8]^32, counter: Word[32], nonce: Word[8]^12) -> Word[32]^16 {
    let constants: Word[32]^16 = [
      0x61707865, 0x3320646e, 0x79622d32, 0x6b206574,
      0, 0, 0, 0,
      0, 0, 0, 0,
      counter, 0, 0, 0,
    ];
    let keyed: Word[32]^16 = for i in 0..8 with s: Word[32]^16 = constants {
      s with [4 + i] = load_le32(key[4 * i], key[4 * i + 1], key[4 * i + 2], key[4 * i + 3])
    };
    for i in 0..3 with s: Word[32]^16 = keyed {
      s with [13 + i] = load_le32(nonce[4 * i], nonce[4 * i + 1], nonce[4 * i + 2], nonce[4 * i + 3])
    }
  }

  // Section 2.3: the sixteen words serialized as 64 little-endian bytes,
  // written as one literal: an update of a 64-byte array costs 64 steps,
  // a literal element one.
  spec serialize(s: Word[32]^16) -> Word[8]^64 {
    let b0: Word[8]^4 = le_bytes(s[0]);
    let b1: Word[8]^4 = le_bytes(s[1]);
    let b2: Word[8]^4 = le_bytes(s[2]);
    let b3: Word[8]^4 = le_bytes(s[3]);
    let b4: Word[8]^4 = le_bytes(s[4]);
    let b5: Word[8]^4 = le_bytes(s[5]);
    let b6: Word[8]^4 = le_bytes(s[6]);
    let b7: Word[8]^4 = le_bytes(s[7]);
    let b8: Word[8]^4 = le_bytes(s[8]);
    let b9: Word[8]^4 = le_bytes(s[9]);
    let b10: Word[8]^4 = le_bytes(s[10]);
    let b11: Word[8]^4 = le_bytes(s[11]);
    let b12: Word[8]^4 = le_bytes(s[12]);
    let b13: Word[8]^4 = le_bytes(s[13]);
    let b14: Word[8]^4 = le_bytes(s[14]);
    let b15: Word[8]^4 = le_bytes(s[15]);
    [
      b0[0], b0[1], b0[2], b0[3], b1[0], b1[1], b1[2], b1[3],
      b2[0], b2[1], b2[2], b2[3], b3[0], b3[1], b3[2], b3[3],
      b4[0], b4[1], b4[2], b4[3], b5[0], b5[1], b5[2], b5[3],
      b6[0], b6[1], b6[2], b6[3], b7[0], b7[1], b7[2], b7[3],
      b8[0], b8[1], b8[2], b8[3], b9[0], b9[1], b9[2], b9[3],
      b10[0], b10[1], b10[2], b10[3], b11[0], b11[1], b11[2], b11[3],
      b12[0], b12[1], b12[2], b12[3], b13[0], b13[1], b13[2], b13[3],
      b14[0], b14[1], b14[2], b14[3], b15[0], b15[1], b15[2], b15[3],
    ]
  }

  // Section 2.3: chacha20_block. Twenty rounds, that is ten inner blocks,
  // then the initial state is added word by word and the result serialized.
  spec chacha20_block(key: Word[8]^32, counter: Word[32], nonce: Word[8]^12) -> Word[8]^64 {
    let initial: Word[32]^16 = initial_state(key, counter, nonce);
    let mixed: Word[32]^16 = for i in 0..10 with s: Word[32]^16 = initial { inner_block(s) };
    serialize(for i in 0..16 with s: Word[32]^16 = mixed { s with [i] = s[i] + initial[i] })
  }

  // Section 2.4: chacha20_encrypt on a 114-byte message, the length of the
  // section 2.8.2 example, from block `counter` on: the first 64 bytes use
  // that block and the remaining 50 the next. A message's length is part of
  // its array type, so each length the vectors need has its own spec.
  // Decryption is the same function applied to the ciphertext.
  spec chacha20_encrypt(
    key: Word[8]^32,
    counter: Word[32],
    nonce: Word[8]^12,
    plaintext: Word[8]^114,
  ) -> Word[8]^114 {
    let first: Word[8]^64 = chacha20_block(key, counter, nonce);
    let second: Word[8]^64 = chacha20_block(key, counter + 1, nonce);
    let head: Word[8]^114 = for n in 0..64 with c: Word[8]^114 = plaintext {
      c with [n] = plaintext[n] ^ first[n]
    };
    for n in 64..114 with c: Word[8]^114 = head { c with [n] = plaintext[n] ^ second[n - 64] }
  }

  // Section 2.4 for a 47-byte message, the length of Wycheproof tcId 71:
  // one block.
  spec chacha20_encrypt_47(
    key: Word[8]^32,
    counter: Word[32],
    nonce: Word[8]^12,
    plaintext: Word[8]^47,
  ) -> Word[8]^47 {
    let first: Word[8]^64 = chacha20_block(key, counter, nonce);
    for n in 0..47 with c: Word[8]^47 = plaintext { c with [n] = plaintext[n] ^ first[n] }
  }

  // Section 2.4 for four whole blocks: the first 256 bytes of the 265-byte
  // ciphertext of appendix A.5, under block counters counter through
  // counter + 3.
  spec chacha20_encrypt_256(
    key: Word[8]^32,
    counter: Word[32],
    nonce: Word[8]^12,
    plaintext: Word[8]^256,
  ) -> Word[8]^256 {
    let first: Word[8]^64 = chacha20_block(key, counter, nonce);
    let second: Word[8]^64 = chacha20_block(key, counter + 1, nonce);
    let third: Word[8]^64 = chacha20_block(key, counter + 2, nonce);
    let fourth: Word[8]^64 = chacha20_block(key, counter + 3, nonce);
    let c1: Word[8]^256 = for n in 0..64 with c: Word[8]^256 = plaintext {
      c with [n] = plaintext[n] ^ first[n]
    };
    let c2: Word[8]^256 = for n in 64..128 with c: Word[8]^256 = c1 {
      c with [n] = plaintext[n] ^ second[n - 64]
    };
    let c3: Word[8]^256 = for n in 128..192 with c: Word[8]^256 = c2 {
      c with [n] = plaintext[n] ^ third[n - 128]
    };
    for n in 192..256 with c: Word[8]^256 = c3 { c with [n] = plaintext[n] ^ fourth[n - 192] }
  }

  // Section 2.4 for the 9 bytes that follow those four blocks, under the
  // block counter of the fifth.
  spec chacha20_encrypt_9(
    key: Word[8]^32,
    counter: Word[32],
    nonce: Word[8]^12,
    plaintext: Word[8]^9,
  ) -> Word[8]^9 {
    let first: Word[8]^64 = chacha20_block(key, counter, nonce);
    for n in 0..9 with c: Word[8]^9 = plaintext { c with [n] = plaintext[n] ^ first[n] }
  }

  // Section 2.5: Poly1305 works modulo the prime p = 2^130 - 5.
  spec prime() -> Int { 1361129467683753853853498429727072845819 }

  // Section 2.5: clamp(r) clears the top four bits of bytes 3, 7, 11 and 15
  // and the bottom two bits of bytes 4, 8 and 12. `Int` has no bitwise and,
  // so the mask 0ffffffc0ffffffc0ffffffc0fffffff is applied to r's bytes.
  spec clamp(r: Word[8]^16) -> Word[8]^16 {
    let mask: Word[8]^16 = [
      0xff, 0xff, 0xff, 0x0f, 0xfc, 0xff, 0xff, 0x0f,
      0xfc, 0xff, 0xff, 0x0f, 0xfc, 0xff, 0xff, 0x0f,
    ];
    for i in 0..16 with c: Word[8]^16 = r { c with [i] = r[i] & mask[i] }
  }

  // Section 2.5.1: le_bytes_to_num on sixteen bytes.
  spec le_bytes_to_num(b: Word[8]^16) -> Int {
    for i in 0..16 with n: Int = 0 { n * 256 + (b[15 - i] as Int) }
  }

  // Section 2.5.1: r is the first half of the key, clamped; s the second.
  spec poly1305_r(key: Word[8]^32) -> Int {
    le_bytes_to_num(clamp(for i in 0..16 with b: Word[8]^16 = [0; 16] { b with [i] = key[i] }))
  }

  spec poly1305_s(key: Word[8]^32) -> Int {
    le_bytes_to_num(for i in 0..16 with b: Word[8]^16 = [0; 16] { b with [i] = key[16 + i] })
  }

  // Section 2.5.1: one block enters the accumulator as a = (r * (a + n)) mod p.
  spec absorb(a: Int, r: Int, n: Int) -> Int { ((a + n) * r) % prime() }

  // Section 2.5.1: the number n of a block is its bytes, little-endian, with
  // the byte 0x01 immediately above them: above byte 15 of a whole block,
  // and above the last byte present of a partial final block. Folding a
  // block's positions from 15 down to 0, a position inside the message
  // contributes its byte, the position equal to the length contributes the
  // 0x01, and positions beyond the length contribute nothing.
  spec block_byte(n: Int, byte: Word[8], position: Int, len: Int) -> Int {
    if position < len { n * 256 + (byte as Int) }
    else if position == len { n * 256 + 1 }
    else { n }
  }

  // Section 2.5.1: the blocks of one 256-byte segment of a message of `len`
  // bytes; the segment holds message bytes `first` through `first + 255`.
  // Block j is absorbed when it begins inside the message; its number starts
  // from 1 (the 0x01 above byte 15) when the block is whole and from 0, to
  // receive the 0x01 at the message's length, when it is partial. A message
  // of at most 256 bytes is one segment.
  spec poly1305_blocks(a: Int, r: Int, m: Word[8]^256, first: Int, len: Int) -> Int {
    for j in 0..16 with acc: Int = a {
      if (first + 16 * j) < len {
        absorb(acc, r, for i in 0..16 with n: Int = if (first + 16 * j + 16) <= len { 1 } else { 0 } {
          block_byte(n, m[16 * j + 15 - i], first + 16 * j + 15 - i, len)
        })
      } else { acc }
    }
  }

  // 256^i for 0 <= i < 16.
  spec byte_weights() -> Int^16 {
    [
      1, 256,
      65536, 16777216,
      4294967296, 1099511627776,
      281474976710656, 72057594037927936,
      18446744073709551616, 4722366482869645213696,
      1208925819614629174706176, 309485009821345068724781056,
      79228162514264337593543950336, 20282409603651670423947251286016,
      5192296858534827628530496329220096, 1329227995784915872903807060280344576,
    ]
  }

  // Section 2.5.1: num_to_16_le_bytes. Byte i is the residue of x / 256^i
  // modulo 256, so the bytes above the sixteenth are dropped: the
  // standard's truncation of a + s to 128 bits.
  spec num_to_16_le_bytes(x: Int) -> Word[8]^16 {
    let w: Int^16 = byte_weights();
    for i in 0..16 with t: Word[8]^16 = [0; 16] { t with [i] = (x / w[i]) as Word[8] }
  }

  // Section 2.5.1: poly1305_mac(msg, key) for a message of `len` bytes,
  // 0 <= len <= 256, carried in the first `len` bytes of `msg`.
  spec poly1305_mac(msg: Word[8]^256, len: Int, key: Word[8]^32) -> Word[8]^16 {
    num_to_16_le_bytes(poly1305_blocks(0, poly1305_r(key), msg, 0, len) + poly1305_s(key))
  }

  // Section 2.5.1 for a message of 256 < len <= 512 bytes in two segments.
  spec poly1305_mac_long(
    head: Word[8]^256,
    tail: Word[8]^256,
    len: Int,
    key: Word[8]^32,
  ) -> Word[8]^16 {
    let r: Int = poly1305_r(key);
    num_to_16_le_bytes(
      poly1305_blocks(poly1305_blocks(0, r, head, 0, len), r, tail, 256, len) + poly1305_s(key),
    )
  }

  // Section 2.6: poly1305_key_gen, the first 32 bytes of ChaCha20 block 0
  // under the key and nonce; the other 32 are discarded.
  spec poly1305_key_gen(key: Word[8]^32, nonce: Word[8]^12) -> Word[8]^32 {
    let block: Word[8]^64 = chacha20_block(key, 0, nonce);
    for i in 0..32 with k: Word[8]^32 = [0; 32] { k with [i] = block[i] }
  }

  // Section 2.8: the input of the MAC is aad || pad16(aad) || ciphertext ||
  // pad16(ciphertext) || num_to_8_le_bytes(|aad|) ||
  // num_to_8_le_bytes(|ciphertext|). Every block of it is whole, so it is
  // absorbed part by part, sixteen bytes at a time, instead of being laid
  // out in one array: the padded aad, the ciphertext with its last block
  // padded, and the block of the two lengths.
  spec whole_block(b: Word[8]^16) -> Int {
    for i in 0..16 with n: Int = 1 { n * 256 + (b[15 - i] as Int) }
  }

  // aad || pad16(aad) for 12 bytes of aad: one block.
  spec absorb_aad_12(a: Int, r: Int, aad: Word[8]^12) -> Int {
    absorb(a, r, whole_block([
      aad[0], aad[1], aad[2], aad[3], aad[4], aad[5], aad[6], aad[7],
      aad[8], aad[9], aad[10], aad[11], 0, 0, 0, 0,
    ]))
  }

  // aad || pad16(aad) for 8 bytes of aad.
  spec absorb_aad_8(a: Int, r: Int, aad: Word[8]^8) -> Int {
    absorb(a, r, whole_block([
      aad[0], aad[1], aad[2], aad[3], aad[4], aad[5], aad[6], aad[7],
      0, 0, 0, 0, 0, 0, 0, 0,
    ]))
  }

  // ciphertext || pad16(ciphertext) for 114 bytes: seven whole blocks and
  // two bytes padded to an eighth.
  spec absorb_ciphertext_114(a: Int, r: Int, c: Word[8]^114) -> Int {
    let whole: Int = for j in 0..7 with acc: Int = a {
      absorb(acc, r, whole_block(
        for i in 0..16 with b: Word[8]^16 = [0; 16] { b with [i] = c[16 * j + i] },
      ))
    };
    absorb(whole, r, whole_block(
      for i in 0..2 with b: Word[8]^16 = [0; 16] { b with [i] = c[112 + i] },
    ))
  }

  // ciphertext || pad16(ciphertext) for 47 bytes: two whole blocks and
  // fifteen bytes padded to a third.
  spec absorb_ciphertext_47(a: Int, r: Int, c: Word[8]^47) -> Int {
    let whole: Int = for j in 0..2 with acc: Int = a {
      absorb(acc, r, whole_block(
        for i in 0..16 with b: Word[8]^16 = [0; 16] { b with [i] = c[16 * j + i] },
      ))
    };
    absorb(whole, r, whole_block(
      for i in 0..15 with b: Word[8]^16 = [0; 16] { b with [i] = c[32 + i] },
    ))
  }

  // ciphertext || pad16(ciphertext) for the 265 bytes of appendix A.5, a
  // 256-byte head and a 9-byte tail: sixteen whole blocks, then the nine
  // bytes padded to a seventeenth.
  spec absorb_ciphertext_265(a: Int, r: Int, head: Word[8]^256, tail: Word[8]^9) -> Int {
    let whole: Int = for j in 0..16 with acc: Int = a {
      absorb(acc, r, whole_block(
        for i in 0..16 with b: Word[8]^16 = [0; 16] { b with [i] = head[16 * j + i] },
      ))
    };
    absorb(whole, r, whole_block(
      for i in 0..9 with b: Word[8]^16 = [0; 16] { b with [i] = tail[i] },
    ))
  }

  // num_to_8_le_bytes(|aad|) || num_to_8_le_bytes(|ciphertext|): the last
  // block.
  spec absorb_lengths(a: Int, r: Int, aad_len: Word[64], ciphertext_len: Word[64]) -> Int {
    let al: Word[8]^8 = num_to_8_le_bytes(aad_len);
    let cl: Word[8]^8 = num_to_8_le_bytes(ciphertext_len);
    absorb(a, r, whole_block([
      al[0], al[1], al[2], al[3], al[4], al[5], al[6], al[7],
      cl[0], cl[1], cl[2], cl[3], cl[4], cl[5], cl[6], cl[7],
    ]))
  }

  // Section 2.8: the tag, poly1305_mac of the MAC input under the one-time
  // key, for 12 bytes of aad and 114 of ciphertext.
  spec aead_tag(otk: Word[8]^32, aad: Word[8]^12, ciphertext: Word[8]^114) -> Word[8]^16 {
    let r: Int = poly1305_r(otk);
    let a: Int = absorb_ciphertext_114(absorb_aad_12(0, r, aad), r, ciphertext);
    num_to_16_le_bytes(absorb_lengths(a, r, 12, 114) + poly1305_s(otk))
  }

  // Section 2.8 for 8 bytes of aad and 47 of ciphertext.
  spec aead_tag_8_47(otk: Word[8]^32, aad: Word[8]^8, ciphertext: Word[8]^47) -> Word[8]^16 {
    let r: Int = poly1305_r(otk);
    let a: Int = absorb_ciphertext_47(absorb_aad_8(0, r, aad), r, ciphertext);
    num_to_16_le_bytes(absorb_lengths(a, r, 8, 47) + poly1305_s(otk))
  }

  // Section 2.8 for 12 bytes of aad and 265 of ciphertext.
  spec aead_tag_12_265(
    otk: Word[8]^32,
    aad: Word[8]^12,
    head: Word[8]^256,
    tail: Word[8]^9,
  ) -> Word[8]^16 {
    let r: Int = poly1305_r(otk);
    let a: Int = absorb_ciphertext_265(absorb_aad_12(0, r, aad), r, head, tail);
    num_to_16_le_bytes(absorb_lengths(a, r, 12, 265) + poly1305_s(otk))
  }

  // Section 2.8: chacha20_aead_encrypt. The 96-bit nonce is the constant
  // and the IV of the standard already joined. The one-time key comes from
  // block 0, the ciphertext from block 1 on, and the result is the
  // ciphertext followed by the 16-byte tag.
  spec chacha20_aead_encrypt(
    aad: Word[8]^12,
    key: Word[8]^32,
    nonce: Word[8]^12,
    plaintext: Word[8]^114,
  ) -> Word[8]^130 {
    let otk: Word[8]^32 = poly1305_key_gen(key, nonce);
    let ciphertext: Word[8]^114 = chacha20_encrypt(key, 1, nonce, plaintext);
    let tag: Word[8]^16 = aead_tag(otk, aad, ciphertext);
    let sealed: Word[8]^130 = for i in 0..114 with out: Word[8]^130 = [0; 130] {
      out with [i] = ciphertext[i]
    };
    for i in 0..16 with out: Word[8]^130 = sealed { out with [114 + i] = tag[i] }
  }

  // Section 2.8 for 8 bytes of aad and a 47-byte plaintext.
  spec chacha20_aead_encrypt_8_47(
    aad: Word[8]^8,
    key: Word[8]^32,
    nonce: Word[8]^12,
    plaintext: Word[8]^47,
  ) -> Word[8]^63 {
    let otk: Word[8]^32 = poly1305_key_gen(key, nonce);
    let ciphertext: Word[8]^47 = chacha20_encrypt_47(key, 1, nonce, plaintext);
    let tag: Word[8]^16 = aead_tag_8_47(otk, aad, ciphertext);
    let sealed: Word[8]^63 = for i in 0..47 with out: Word[8]^63 = [0; 63] {
      out with [i] = ciphertext[i]
    };
    for i in 0..16 with out: Word[8]^63 = sealed { out with [47 + i] = tag[i] }
  }

  // Section 2.8: decryption recomputes the tag over the received ciphertext
  // and compares it with the received tag, byte by byte.
  spec chacha20_aead_verify(
    aad: Word[8]^12,
    key: Word[8]^32,
    nonce: Word[8]^12,
    sealed: Word[8]^130,
  ) -> Bool {
    let otk: Word[8]^32 = poly1305_key_gen(key, nonce);
    let ciphertext: Word[8]^114 = for i in 0..114 with c: Word[8]^114 = [0; 114] {
      c with [i] = sealed[i]
    };
    let tag: Word[8]^16 = aead_tag(otk, aad, ciphertext);
    for i in 0..16 with same: Bool = true { same && (tag[i] == sealed[114 + i]) }
  }

  // Section 2.8: chacha20_aead_decrypt. The plaintext is released only when
  // the tag verifies; otherwise the result is all zeros, and the verdict of
  // chacha20_aead_verify is false.
  spec chacha20_aead_decrypt(
    aad: Word[8]^12,
    key: Word[8]^32,
    nonce: Word[8]^12,
    sealed: Word[8]^130,
  ) -> Word[8]^114 {
    let ciphertext: Word[8]^114 = for i in 0..114 with c: Word[8]^114 = [0; 114] {
      c with [i] = sealed[i]
    };
    if chacha20_aead_verify(aad, key, nonce, sealed) {
      chacha20_encrypt(key, 1, nonce, ciphertext)
    } else {
      [0; 114]
    }
  }

  // Section 2.8 for the 265-byte ciphertext of appendix A.5, passed as a
  // 256-byte head and a 9-byte tail with its tag apart. The tag check is
  // one spec and the two parts of the plaintext are two, each released only
  // when the tag verifies.
  spec chacha20_aead_verify_265(
    aad: Word[8]^12,
    key: Word[8]^32,
    nonce: Word[8]^12,
    head: Word[8]^256,
    tail: Word[8]^9,
    tag: Word[8]^16,
  ) -> Bool {
    let otk: Word[8]^32 = poly1305_key_gen(key, nonce);
    let computed: Word[8]^16 = aead_tag_12_265(otk, aad, head, tail);
    for i in 0..16 with same: Bool = true { same && (computed[i] == tag[i]) }
  }

  spec chacha20_aead_decrypt_265_head(
    aad: Word[8]^12,
    key: Word[8]^32,
    nonce: Word[8]^12,
    head: Word[8]^256,
    tail: Word[8]^9,
    tag: Word[8]^16,
  ) -> Word[8]^256 {
    if chacha20_aead_verify_265(aad, key, nonce, head, tail, tag) {
      chacha20_encrypt_256(key, 1, nonce, head)
    } else {
      [0; 256]
    }
  }

  spec chacha20_aead_decrypt_265_tail(
    aad: Word[8]^12,
    key: Word[8]^32,
    nonce: Word[8]^12,
    head: Word[8]^256,
    tail: Word[8]^9,
    tag: Word[8]^16,
  ) -> Word[8]^9 {
    if chacha20_aead_verify_265(aad, key, nonce, head, tail, tag) {
      chacha20_encrypt_9(key, 5, nonce, tail)
    } else {
      [0; 9]
    }
  }

  spec rfc8439_2_8_2_key() -> Word[8]^32 {
    [
      0x80, 0x81, 0x82, 0x83, 0x84, 0x85, 0x86, 0x87, 0x88, 0x89, 0x8a, 0x8b, 0x8c, 0x8d, 0x8e, 0x8f,
      0x90, 0x91, 0x92, 0x93, 0x94, 0x95, 0x96, 0x97, 0x98, 0x99, 0x9a, 0x9b, 0x9c, 0x9d, 0x9e, 0x9f,
    ]
  }

  spec rfc8439_2_8_2_nonce() -> Word[8]^12 { [
      0x07, 0x00, 0x00, 0x00, 0x40, 0x41, 0x42, 0x43, 0x44, 0x45, 0x46, 0x47,
    ] }

  spec rfc8439_2_8_2_aad() -> Word[8]^12 { [
      0x50, 0x51, 0x52, 0x53, 0xc0, 0xc1, 0xc2, 0xc3, 0xc4, 0xc5, 0xc6, 0xc7,
    ] }

  spec sunscreen() -> Word[8]^114 {
    [
      0x4c, 0x61, 0x64, 0x69, 0x65, 0x73, 0x20, 0x61, 0x6e, 0x64, 0x20, 0x47,
      0x65, 0x6e, 0x74, 0x6c, 0x65, 0x6d, 0x65, 0x6e, 0x20, 0x6f, 0x66, 0x20,
      0x74, 0x68, 0x65, 0x20, 0x63, 0x6c, 0x61, 0x73, 0x73, 0x20, 0x6f, 0x66,
      0x20, 0x27, 0x39, 0x39, 0x3a, 0x20, 0x49, 0x66, 0x20, 0x49, 0x20, 0x63,
      0x6f, 0x75, 0x6c, 0x64, 0x20, 0x6f, 0x66, 0x66, 0x65, 0x72, 0x20, 0x79,
      0x6f, 0x75, 0x20, 0x6f, 0x6e, 0x6c, 0x79, 0x20, 0x6f, 0x6e, 0x65, 0x20,
      0x74, 0x69, 0x70, 0x20, 0x66, 0x6f, 0x72, 0x20, 0x74, 0x68, 0x65, 0x20,
      0x66, 0x75, 0x74, 0x75, 0x72, 0x65, 0x2c, 0x20, 0x73, 0x75, 0x6e, 0x73,
      0x63, 0x72, 0x65, 0x65, 0x6e, 0x20, 0x77, 0x6f, 0x75, 0x6c, 0x64, 0x20,
      0x62, 0x65, 0x20, 0x69, 0x74, 0x2e,
    ]
  }

  // RFC 8439 section 2.6.2: the one-time key under the key 80 81 ... 9f and
  // the nonce 00 00 00 00 00 01 02 03 04 05 06 07.
  spec rfc8439_2_6_2() -> Word[8]^32 {
    poly1305_key_gen(rfc8439_2_8_2_key(), [
      0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07,
    ])
  }

  spec rfc8439_2_6_2_expected() -> Word[8]^32 {
    [
      0x8a, 0xd5, 0xa0, 0x8b, 0x90, 0x5f, 0x81, 0xcc, 0x81, 0x50, 0x40, 0x27, 0x4a, 0xb2, 0x94, 0x71,
      0xa8, 0x33, 0xb6, 0x37, 0xe3, 0xfd, 0x0d, 0xa5, 0x08, 0xdb, 0xb8, 0xe2, 0xfd, 0xd1, 0xa6, 0x46,
    ]
  }

  // RFC 8439 section 2.8.2: sealing the 114-byte sunscreen text; the
  // ciphertext followed by the tag 1a e1 0b 59 ... 06 91.
  spec rfc8439_2_8_2() -> Word[8]^130 {
    chacha20_aead_encrypt(rfc8439_2_8_2_aad(), rfc8439_2_8_2_key(), rfc8439_2_8_2_nonce(), sunscreen())
  }

  spec rfc8439_2_8_2_expected() -> Word[8]^130 {
    [
      0xd3, 0x1a, 0x8d, 0x34, 0x64, 0x8e, 0x60, 0xdb, 0x7b, 0x86, 0xaf, 0xbc, 0x53, 0xef, 0x7e, 0xc2,
      0xa4, 0xad, 0xed, 0x51, 0x29, 0x6e, 0x08, 0xfe, 0xa9, 0xe2, 0xb5, 0xa7, 0x36, 0xee, 0x62, 0xd6,
      0x3d, 0xbe, 0xa4, 0x5e, 0x8c, 0xa9, 0x67, 0x12, 0x82, 0xfa, 0xfb, 0x69, 0xda, 0x92, 0x72, 0x8b,
      0x1a, 0x71, 0xde, 0x0a, 0x9e, 0x06, 0x0b, 0x29, 0x05, 0xd6, 0xa5, 0xb6, 0x7e, 0xcd, 0x3b, 0x36,
      0x92, 0xdd, 0xbd, 0x7f, 0x2d, 0x77, 0x8b, 0x8c, 0x98, 0x03, 0xae, 0xe3, 0x28, 0x09, 0x1b, 0x58,
      0xfa, 0xb3, 0x24, 0xe4, 0xfa, 0xd6, 0x75, 0x94, 0x55, 0x85, 0x80, 0x8b, 0x48, 0x31, 0xd7, 0xbc,
      0x3f, 0xf4, 0xde, 0xf0, 0x8e, 0x4b, 0x7a, 0x9d, 0xe5, 0x76, 0xd2, 0x65, 0x86, 0xce, 0xc6, 0x4b,
      0x61, 0x16, 0x1a, 0xe1, 0x0b, 0x59, 0x4f, 0x09, 0xe2, 0x6a, 0x7e, 0x90, 0x2e, 0xcb, 0xd0, 0x60,
      0x06, 0x91,
    ]
  }

  // RFC 8439 section 2.8.2 opened: the tag of the published ciphertext
  // verifies, and the decryption is the sunscreen text.
  spec rfc8439_2_8_2_verify() -> Bool {
    chacha20_aead_verify(
      rfc8439_2_8_2_aad(), rfc8439_2_8_2_key(), rfc8439_2_8_2_nonce(), rfc8439_2_8_2_expected(),
    )
  }

  spec rfc8439_2_8_2_verify_expected() -> Bool { true }

  spec rfc8439_2_8_2_open() -> Word[8]^114 {
    chacha20_aead_decrypt(
      rfc8439_2_8_2_aad(), rfc8439_2_8_2_key(), rfc8439_2_8_2_nonce(), rfc8439_2_8_2_expected(),
    )
  }

  spec rfc8439_2_8_2_open_expected() -> Word[8]^114 { sunscreen() }

  // The published ciphertext with the last bit of its tag flipped: the
  // verdict is false and the decryption withholds the plaintext. A check of
  // the tag comparison, not a published vector.
  spec rfc8439_2_8_2_tampered_verify() -> Bool {
    chacha20_aead_verify(
      rfc8439_2_8_2_aad(), rfc8439_2_8_2_key(), rfc8439_2_8_2_nonce(),
      rfc8439_2_8_2_expected() with [129] = 0x90,
    )
  }

  spec rfc8439_2_8_2_tampered_verify_expected() -> Bool { false }

  // RFC 8439 appendix A.5: the 265-byte ciphertext, its 12-byte aad and
  // its tag under the key 1c 92 40 a5 ... and the nonce 00 00 00 00 01 02
  // ... 08. The ciphertext is a 256-byte head and a 9-byte tail.
  spec rfc8439_a5_key() -> Word[8]^32 {
    [
      0x1c, 0x92, 0x40, 0xa5, 0xeb, 0x55, 0xd3, 0x8a, 0xf3, 0x33, 0x88, 0x86, 0x04, 0xf6, 0xb5, 0xf0,
      0x47, 0x39, 0x17, 0xc1, 0x40, 0x2b, 0x80, 0x09, 0x9d, 0xca, 0x5c, 0xbc, 0x20, 0x70, 0x75, 0xc0,
    ]
  }

  spec rfc8439_a5_nonce() -> Word[8]^12 { [
      0x00, 0x00, 0x00, 0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08,
    ] }

  spec rfc8439_a5_aad() -> Word[8]^12 { [
      0xf3, 0x33, 0x88, 0x86, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x4e, 0x91,
    ] }

  spec rfc8439_a5_ciphertext_head() -> Word[8]^256 {
    [
      0x64, 0xa0, 0x86, 0x15, 0x75, 0x86, 0x1a, 0xf4, 0x60, 0xf0, 0x62, 0xc7, 0x9b, 0xe6, 0x43, 0xbd,
      0x5e, 0x80, 0x5c, 0xfd, 0x34, 0x5c, 0xf3, 0x89, 0xf1, 0x08, 0x67, 0x0a, 0xc7, 0x6c, 0x8c, 0xb2,
      0x4c, 0x6c, 0xfc, 0x18, 0x75, 0x5d, 0x43, 0xee, 0xa0, 0x9e, 0xe9, 0x4e, 0x38, 0x2d, 0x26, 0xb0,
      0xbd, 0xb7, 0xb7, 0x3c, 0x32, 0x1b, 0x01, 0x00, 0xd4, 0xf0, 0x3b, 0x7f, 0x35, 0x58, 0x94, 0xcf,
      0x33, 0x2f, 0x83, 0x0e, 0x71, 0x0b, 0x97, 0xce, 0x98, 0xc8, 0xa8, 0x4a, 0xbd, 0x0b, 0x94, 0x81,
      0x14, 0xad, 0x17, 0x6e, 0x00, 0x8d, 0x33, 0xbd, 0x60, 0xf9, 0x82, 0xb1, 0xff, 0x37, 0xc8, 0x55,
      0x97, 0x97, 0xa0, 0x6e, 0xf4, 0xf0, 0xef, 0x61, 0xc1, 0x86, 0x32, 0x4e, 0x2b, 0x35, 0x06, 0x38,
      0x36, 0x06, 0x90, 0x7b, 0x6a, 0x7c, 0x02, 0xb0, 0xf9, 0xf6, 0x15, 0x7b, 0x53, 0xc8, 0x67, 0xe4,
      0xb9, 0x16, 0x6c, 0x76, 0x7b, 0x80, 0x4d, 0x46, 0xa5, 0x9b, 0x52, 0x16, 0xcd, 0xe7, 0xa4, 0xe9,
      0x90, 0x40, 0xc5, 0xa4, 0x04, 0x33, 0x22, 0x5e, 0xe2, 0x82, 0xa1, 0xb0, 0xa0, 0x6c, 0x52, 0x3e,
      0xaf, 0x45, 0x34, 0xd7, 0xf8, 0x3f, 0xa1, 0x15, 0x5b, 0x00, 0x47, 0x71, 0x8c, 0xbc, 0x54, 0x6a,
      0x0d, 0x07, 0x2b, 0x04, 0xb3, 0x56, 0x4e, 0xea, 0x1b, 0x42, 0x22, 0x73, 0xf5, 0x48, 0x27, 0x1a,
      0x0b, 0xb2, 0x31, 0x60, 0x53, 0xfa, 0x76, 0x99, 0x19, 0x55, 0xeb, 0xd6, 0x31, 0x59, 0x43, 0x4e,
      0xce, 0xbb, 0x4e, 0x46, 0x6d, 0xae, 0x5a, 0x10, 0x73, 0xa6, 0x72, 0x76, 0x27, 0x09, 0x7a, 0x10,
      0x49, 0xe6, 0x17, 0xd9, 0x1d, 0x36, 0x10, 0x94, 0xfa, 0x68, 0xf0, 0xff, 0x77, 0x98, 0x71, 0x30,
      0x30, 0x5b, 0xea, 0xba, 0x2e, 0xda, 0x04, 0xdf, 0x99, 0x7b, 0x71, 0x4d, 0x6c, 0x6f, 0x2c, 0x29,
    ]
  }

  spec rfc8439_a5_ciphertext_tail() -> Word[8]^9 { [
      0xa6, 0xad, 0x5c, 0xb4, 0x02, 0x2b, 0x02, 0x70, 0x9b,
    ] }

  spec rfc8439_a5_tag() -> Word[8]^16 {
    [
      0xee, 0xad, 0x9d, 0x67, 0x89, 0x0c, 0xbb, 0x22, 0x39, 0x23, 0x36, 0xfe, 0xa1, 0x85, 0x1f, 0x38,
    ]
  }

  spec rfc8439_a5_verify() -> Bool {
    chacha20_aead_verify_265(
      rfc8439_a5_aad(), rfc8439_a5_key(), rfc8439_a5_nonce(),
      rfc8439_a5_ciphertext_head(), rfc8439_a5_ciphertext_tail(), rfc8439_a5_tag(),
    )
  }

  spec rfc8439_a5_verify_expected() -> Bool { true }

  // The plaintext of appendix A.5, the Internet-Drafts boilerplate, released
  // by the decryption in the same two parts.
  spec rfc8439_a5_head() -> Word[8]^256 {
    chacha20_aead_decrypt_265_head(
      rfc8439_a5_aad(), rfc8439_a5_key(), rfc8439_a5_nonce(),
      rfc8439_a5_ciphertext_head(), rfc8439_a5_ciphertext_tail(), rfc8439_a5_tag(),
    )
  }

  spec rfc8439_a5_head_expected() -> Word[8]^256 {
    [
      0x49, 0x6e, 0x74, 0x65, 0x72, 0x6e, 0x65, 0x74, 0x2d, 0x44, 0x72, 0x61, 0x66, 0x74, 0x73, 0x20,
      0x61, 0x72, 0x65, 0x20, 0x64, 0x72, 0x61, 0x66, 0x74, 0x20, 0x64, 0x6f, 0x63, 0x75, 0x6d, 0x65,
      0x6e, 0x74, 0x73, 0x20, 0x76, 0x61, 0x6c, 0x69, 0x64, 0x20, 0x66, 0x6f, 0x72, 0x20, 0x61, 0x20,
      0x6d, 0x61, 0x78, 0x69, 0x6d, 0x75, 0x6d, 0x20, 0x6f, 0x66, 0x20, 0x73, 0x69, 0x78, 0x20, 0x6d,
      0x6f, 0x6e, 0x74, 0x68, 0x73, 0x20, 0x61, 0x6e, 0x64, 0x20, 0x6d, 0x61, 0x79, 0x20, 0x62, 0x65,
      0x20, 0x75, 0x70, 0x64, 0x61, 0x74, 0x65, 0x64, 0x2c, 0x20, 0x72, 0x65, 0x70, 0x6c, 0x61, 0x63,
      0x65, 0x64, 0x2c, 0x20, 0x6f, 0x72, 0x20, 0x6f, 0x62, 0x73, 0x6f, 0x6c, 0x65, 0x74, 0x65, 0x64,
      0x20, 0x62, 0x79, 0x20, 0x6f, 0x74, 0x68, 0x65, 0x72, 0x20, 0x64, 0x6f, 0x63, 0x75, 0x6d, 0x65,
      0x6e, 0x74, 0x73, 0x20, 0x61, 0x74, 0x20, 0x61, 0x6e, 0x79, 0x20, 0x74, 0x69, 0x6d, 0x65, 0x2e,
      0x20, 0x49, 0x74, 0x20, 0x69, 0x73, 0x20, 0x69, 0x6e, 0x61, 0x70, 0x70, 0x72, 0x6f, 0x70, 0x72,
      0x69, 0x61, 0x74, 0x65, 0x20, 0x74, 0x6f, 0x20, 0x75, 0x73, 0x65, 0x20, 0x49, 0x6e, 0x74, 0x65,
      0x72, 0x6e, 0x65, 0x74, 0x2d, 0x44, 0x72, 0x61, 0x66, 0x74, 0x73, 0x20, 0x61, 0x73, 0x20, 0x72,
      0x65, 0x66, 0x65, 0x72, 0x65, 0x6e, 0x63, 0x65, 0x20, 0x6d, 0x61, 0x74, 0x65, 0x72, 0x69, 0x61,
      0x6c, 0x20, 0x6f, 0x72, 0x20, 0x74, 0x6f, 0x20, 0x63, 0x69, 0x74, 0x65, 0x20, 0x74, 0x68, 0x65,
      0x6d, 0x20, 0x6f, 0x74, 0x68, 0x65, 0x72, 0x20, 0x74, 0x68, 0x61, 0x6e, 0x20, 0x61, 0x73, 0x20,
      0x2f, 0xe2, 0x80, 0x9c, 0x77, 0x6f, 0x72, 0x6b, 0x20, 0x69, 0x6e, 0x20, 0x70, 0x72, 0x6f, 0x67,
    ]
  }

  spec rfc8439_a5_tail() -> Word[8]^9 {
    chacha20_aead_decrypt_265_tail(
      rfc8439_a5_aad(), rfc8439_a5_key(), rfc8439_a5_nonce(),
      rfc8439_a5_ciphertext_head(), rfc8439_a5_ciphertext_tail(), rfc8439_a5_tag(),
    )
  }

  spec rfc8439_a5_tail_expected() -> Word[8]^9 { [
      0x72, 0x65, 0x73, 0x73, 0x2e, 0x2f, 0xe2, 0x80, 0x9d,
    ] }

  // RFC 8439 section 2.8.2: the one-time key is the first 32 bytes of
  // ChaCha20 block 0 under that section's key and nonce.
  spec rfc8439_2_8_2_otk() -> Word[8]^32 {
    poly1305_key_gen(rfc8439_2_8_2_key(), rfc8439_2_8_2_nonce())
  }

  spec rfc8439_2_8_2_otk_expected() -> Word[8]^32 {
    [
      0x7b, 0xac, 0x2b, 0x25, 0x2d, 0xb4, 0x47, 0xaf, 0x09, 0xb6, 0x7a, 0x55, 0xa4, 0xe9, 0x55, 0x84,
      0x0a, 0xe1, 0xd6, 0x73, 0x10, 0x75, 0xd9, 0xeb, 0x2a, 0x93, 0x75, 0x78, 0x3e, 0xd5, 0x53, 0xff,
    ]
  }

  // The same ciphertext with the last tag byte 0x91 replaced by 0x90.
  // Decryption withholds the plaintext.
  spec rfc8439_2_8_2_tampered_open() -> Word[8]^114 {
    chacha20_aead_decrypt(
      rfc8439_2_8_2_aad(), rfc8439_2_8_2_key(), rfc8439_2_8_2_nonce(),
      rfc8439_2_8_2_expected() with [129] = 0x90,
    )
  }

  spec rfc8439_2_8_2_tampered_open_expected() -> Word[8]^114 { [0; 114] }


  test "RFC 8439 section 2.6.2 one-time key" {
    rfc8439_2_6_2() == rfc8439_2_6_2_expected()
  }

  test "RFC 8439 section 2.8.2 one-time key" {
    rfc8439_2_8_2_otk() == rfc8439_2_8_2_otk_expected()
  }

  test "RFC 8439 section 2.8.2 seal" {
    rfc8439_2_8_2() == rfc8439_2_8_2_expected()
  }

  test "RFC 8439 section 2.8.2 open" {
    rfc8439_2_8_2_verify() && (rfc8439_2_8_2_open() == rfc8439_2_8_2_open_expected())
  }

  test "RFC 8439 section 2.8.2 rejects a flipped tag bit" {
    (rfc8439_2_8_2_tampered_verify() == false)
      && (rfc8439_2_8_2_tampered_open() == rfc8439_2_8_2_tampered_open_expected())
  }

  test "RFC 8439 appendix A.5" {
    rfc8439_a5_verify()
      && (rfc8439_a5_head() == rfc8439_a5_head_expected())
      && (rfc8439_a5_tail() == rfc8439_a5_tail_expected())
  }
}
```

### SHA-3 and SHAKE, FIPS 202

SHA-3 and SHAKE as FIPS 202 (August 2015, DOI 10.6028/NIST.FIPS.202) write
them: Keccak-p[1600, 24] of section 3, the sponge of section 4, pad10*1 of
section 5.1 with the domain byte of section 6, and SHA3-256, SHA3-512,
SHAKE128, and SHAKE256. SHA3-224 and SHA3-384 are that sponge at the other
two capacities. This listing does not restate them. Finite type parameters
are slice S3o and are already Current. These four functions do not use one:
each rate and each output length is its own spec, because an array length is
part of its type. `orangec test` accepts the eight tests below.

The state in this listing is `Word[64]^25`, with lane $(x, y)$ at index
$x + 5y$, little-endian, which is the byte placement of section 3.1.2. A
rank-2 spelling, five sheets of five lanes indexed $A[x][y]$, is what slice
S3u also admits. This transcription is the lane vector
`algorithms/sha3/sha3.or` checks. This documentation branch does not contain
the S3u compiler commit.

#### 1. The Permutation (section 3)

One round is $\theta$, then $\rho$, then $\pi$, then $\chi$, then $\iota$.
Keccak-p[1600, 24] is 24 rounds. `round_constants` is the 24 constants of
section 3.2.5.

$\theta$ forms the five column parities and adds
$D[x] = C[x - 1] \oplus \mathrm{rot}(C[x + 1], 1)$ to every lane of column
$x$. $\rho$ rotates each lane by the offset in Table 2. $\pi$ moves lane
$(x, y)$ to $(y, 2x + 3y)$. $\chi$ is the row map
$A[x] \oplus (\lnot A[x + 1] \land A[x + 2])$. $\iota$ XORs the round
constant into lane $(0, 0)$.

#### 2. The Sponge (sections 4, 5, and 6)

| Function | Capacity $c$ | Rate, bytes | Domain byte | Output in this listing |
| :--- | :--- | :--- | :--- | :--- |
| SHA3-256 | 512 | 136 | `0x06` | 32 bytes |
| SHA3-512 | 1024 | 72 | `0x06` | 64 bytes |
| SHAKE128 | 256 | 168 | `0x1f` | 32 bytes, or 168 bytes |
| SHAKE256 | 512 | 136 | `0x1f` | 64 bytes |

The domain byte is the suffix of section 6 (`01` for SHA-3, `1111` for
SHAKE) together with the first 1 of pad10*1, in the byte form of Appendix
B.2. `pad` XORs that byte at position $n$ and XORs `0x80` at byte $r - 1$,
the last byte of the rate. When $n = r - 1$ the two XORs land on one byte.
None of the eight tests uses that boundary.

Absorb XORs the block into the state and applies the permutation. Every
output here has length at most one rate, so one squeeze is enough and the
extra permutation of Algorithm 8 step 10 is not reached.

The message buffer is `Word[8]^200`. A one-block call takes a length $n$
strictly below the rate. Length 0 is a zero buffer: the domain byte is
written at byte 0. There is no `Word[8]^0`. The two-block SHA3-256 call
takes exactly 200 bytes, `00` through `c7`. The first block is the first
136 bytes, absorbed without padding. The second is the remaining 64 bytes,
then pad10*1 at rate 136.

#### 3. Known Answers

| Message | Result |
| :--- | :--- |
| SHA3-256, empty | `a7ffc6f8bf1ed76651c14756a061d662f580ff4de43b49fa82d80a4b80f8434a` |
| SHA3-256, "abc" | `3a985da74fe225b2045c172d6bd390bd855f086e3e9d525b46bfe24511431532` |
| SHA3-256, 200 bytes `00` through `c7` | `5f728f63bf5ee48c77f453c0490398fa645b8d4c4e56be9a41cfec344d6ca899` |
| SHA3-512, empty | `a69f73cca23a9ac5c8b567dc185a756e97c982164fe25859e0d1dcc1475c80a615b2123af1f5f94c11e3e9402c3ac558f500199d95b6d3e301758586281dcd26` |
| SHA3-512, "abc" | `b751850b1a57168a5693cd924b6b096e08f621827444f70d884f5d0240d2712e10e116e9192af3c91a7ec57647e3934057340b4cf408d5a56592f8274eec53f0` |
| SHAKE128, empty, 32 bytes | `7f9c2ba4e88f827d616045507605853ed73b8093f6efbc88eb1a6eacfa66ef26` |
| SHAKE256, "abc", 64 bytes | `483366601360a8771c6863080cc4114d8db44530f8f1e1ee4f94ea37e78b5739d5a15bef186a5386c75744c0527e1faa9f8726e462a12a4feb06bd8801e751e4` |
| SHAKE128, "abc", 168 bytes | begins `5881092dd818bf5cf8a3ddb793fbcba7`; the listing checks all 168 |

#### 4. Compiler-Checked Transcription

```orange
// SHA-3 and SHAKE of FIPS 202 (2015), "SHA-3 Standard: Permutation-Based
// Hash and Extendable-Output Functions", https://doi.org/10.6028/NIST.FIPS.202.
// Keccak-p[1600, 24] (section 3) acts on the state array as 25 lanes of 64
// bits, its step mappings theta, rho, pi, chi and iota (3.2) are one spec
// each, the sponge construction (4) with pad10*1 (5.1) absorbs one or two
// blocks, and SHA3-256, SHA3-512 (6.1), SHAKE128 and SHAKE256 (6.2)
// reproduce the digests of the empty message from Botan's sha3.vec and
// shake.vec (the latter selected from the NIST CAVS file), and the digests of
// "abc", of a 200-byte two-block message, and 168 bytes of SHAKE128 output
// from hashlib.
edition 2026;
module sha3_spec {
  spec load_le64(
    b0: Word[8], b1: Word[8], b2: Word[8], b3: Word[8],
    b4: Word[8], b5: Word[8], b6: Word[8], b7: Word[8],
  ) -> Word[64] {
    (b0 as Word[64]) | ((b1 as Word[64]) << 8) | ((b2 as Word[64]) << 16)
      | ((b3 as Word[64]) << 24) | ((b4 as Word[64]) << 32) | ((b5 as Word[64]) << 40)
      | ((b6 as Word[64]) << 48) | ((b7 as Word[64]) << 56)
  }

  spec le_bytes64(x: Word[64]) -> Word[8]^8 {
    [
      x as Word[8], (x >> 8) as Word[8], (x >> 16) as Word[8], (x >> 24) as Word[8],
      (x >> 32) as Word[8], (x >> 40) as Word[8], (x >> 48) as Word[8], (x >> 56) as Word[8],
    ]
  }

  // Section 3.1.2: the state array from a string of b = 1600 bits, here 200
  // bytes. Lane (x, y) is the 64-bit word Lane(x, y) = S[64(5y + x) ..],
  // read little-endian, and lives at index x + 5y of the array.
  spec lanes(s: Word[8]^200) -> Word[64]^25 {
    for i in 0..25 with a: Word[64]^25 = [0; 25] {
      a with [i] = load_le64(
        s[8 * i], s[8 * i + 1], s[8 * i + 2], s[8 * i + 3],
        s[8 * i + 4], s[8 * i + 5], s[8 * i + 6], s[8 * i + 7],
      )
    }
  }

  // Section 3.2.1, theta: C[x] is the parity of column x, D[x] the parity of
  // the two neighbouring columns with one rotated by 1, and each lane of
  // plane y is A[x, y] ^ D[x]; the 25 lanes are written out, one line per y.
  spec theta(a: Word[64]^25) -> Word[64]^25 {
    let c: Word[64]^5 = for x in 0..5 with c: Word[64]^5 = [0; 5] {
      c with [x] = a[x] ^ a[x + 5] ^ a[x + 10] ^ a[x + 15] ^ a[x + 20]
    };
    let d: Word[64]^5 = for x in 0..5 with d: Word[64]^5 = [0; 5] {
      d with [x] = c[(x + 4) % 5] ^ (c[(x + 1) % 5] <<< 1)
    };
    [
      a[0] ^ d[0], a[1] ^ d[1], a[2] ^ d[2], a[3] ^ d[3], a[4] ^ d[4],
      a[5] ^ d[0], a[6] ^ d[1], a[7] ^ d[2], a[8] ^ d[3], a[9] ^ d[4],
      a[10] ^ d[0], a[11] ^ d[1], a[12] ^ d[2], a[13] ^ d[3], a[14] ^ d[4],
      a[15] ^ d[0], a[16] ^ d[1], a[17] ^ d[2], a[18] ^ d[3], a[19] ^ d[4],
      a[20] ^ d[0], a[21] ^ d[1], a[22] ^ d[2], a[23] ^ d[3], a[24] ^ d[4],
    ]
  }

  // Section 3.2.2, rho: each lane rotated by its offset of Table 2, one line
  // per y. A rotation amount must be a literal, so the table appears as the
  // amounts rather than as an array the loop would read.
  spec rho(a: Word[64]^25) -> Word[64]^25 {
    [
      a[0] <<< 0, a[1] <<< 1, a[2] <<< 62, a[3] <<< 28, a[4] <<< 27,
      a[5] <<< 36, a[6] <<< 44, a[7] <<< 6, a[8] <<< 55, a[9] <<< 20,
      a[10] <<< 3, a[11] <<< 10, a[12] <<< 43, a[13] <<< 25, a[14] <<< 39,
      a[15] <<< 41, a[16] <<< 45, a[17] <<< 15, a[18] <<< 21, a[19] <<< 8,
      a[20] <<< 18, a[21] <<< 2, a[22] <<< 61, a[23] <<< 56, a[24] <<< 14,
    ]
  }

  // Section 3.2.3, pi: A'[x, y] = A[(x + 3y) mod 5, x], so position x + 5y
  // takes the lane at ((x + 3y) mod 5) + 5x; one line per y.
  spec pi(a: Word[64]^25) -> Word[64]^25 {
    [
      a[0], a[6], a[12], a[18], a[24],
      a[3], a[9], a[10], a[16], a[22],
      a[1], a[7], a[13], a[19], a[20],
      a[4], a[5], a[11], a[17], a[23],
      a[2], a[8], a[14], a[15], a[21],
    ]
  }

  // Section 3.2.4, chi: A'[x, y] = A[x, y] ^ (~A[(x + 1) mod 5, y] &
  // A[(x + 2) mod 5, y]), the only non-linear step; one line per row y.
  spec chi(a: Word[64]^25) -> Word[64]^25 {
    [
      a[0] ^ (~a[1] & a[2]), a[1] ^ (~a[2] & a[3]), a[2] ^ (~a[3] & a[4]),
      a[3] ^ (~a[4] & a[0]), a[4] ^ (~a[0] & a[1]),
      a[5] ^ (~a[6] & a[7]), a[6] ^ (~a[7] & a[8]), a[7] ^ (~a[8] & a[9]),
      a[8] ^ (~a[9] & a[5]), a[9] ^ (~a[5] & a[6]),
      a[10] ^ (~a[11] & a[12]), a[11] ^ (~a[12] & a[13]), a[12] ^ (~a[13] & a[14]),
      a[13] ^ (~a[14] & a[10]), a[14] ^ (~a[10] & a[11]),
      a[15] ^ (~a[16] & a[17]), a[16] ^ (~a[17] & a[18]), a[17] ^ (~a[18] & a[19]),
      a[18] ^ (~a[19] & a[15]), a[19] ^ (~a[15] & a[16]),
      a[20] ^ (~a[21] & a[22]), a[21] ^ (~a[22] & a[23]), a[22] ^ (~a[23] & a[24]),
      a[23] ^ (~a[24] & a[20]), a[24] ^ (~a[20] & a[21]),
    ]
  }

  // Section 3.2.5: the round constants RC[ir] for ir = 0 through 23. Bit
  // 2^j - 1 of RC[ir] is rc(j + 7 ir), the output of the LFSR of Algorithm 5
  // with polynomial x^8 + x^6 + x^5 + x^4 + 1; the table was generated from
  // that LFSR and agrees with the Keccak team's CompactFIPS202.py.
  spec round_constants() -> Word[64]^24 {
    [
      0x0000000000000001, 0x0000000000008082, 0x800000000000808a, 0x8000000080008000,
      0x000000000000808b, 0x0000000080000001, 0x8000000080008081, 0x8000000000008009,
      0x000000000000008a, 0x0000000000000088, 0x0000000080008009, 0x000000008000000a,
      0x000000008000808b, 0x800000000000008b, 0x8000000000008089, 0x8000000000008003,
      0x8000000000008002, 0x8000000000000080, 0x000000000000800a, 0x800000008000000a,
      0x8000000080008081, 0x8000000000008080, 0x0000000080000001, 0x8000000080008008,
    ]
  }

  // Section 3.2.5, iota: the round constant enters lane (0, 0).
  spec iota(a: Word[64]^25, rc: Word[64]) -> Word[64]^25 { a with [0] = a[0] ^ rc }

  // Section 3.3: Rnd(A, ir) = iota(chi(pi(rho(theta(A)))), ir).
  spec rnd(a: Word[64]^25, rc: Word[64]) -> Word[64]^25 { iota(chi(pi(rho(theta(a)))), rc) }

  // Sections 3.3 and 3.4: Keccak-p[1600, 24], the 24 rounds ir = 0 through
  // 23, which is Keccak-f[1600].
  spec keccak_p(a: Word[64]^25) -> Word[64]^25 {
    let rc: Word[64]^24 = round_constants();
    for ir in 0..24 with s: Word[64]^25 = a { rnd(s, rc[ir]) }
  }

  // Section 5.1, pad10*1, in the byte form of Appendix B.2 together with
  // the domain suffix of section 6: a message of n bytes sits in the first n
  // bytes of a 200-byte string whose other bytes are zero; the suffix bits
  // followed by the first 1 of the padding form the byte at position n
  // (0x06 for SHA-3, 0x1f for SHAKE), and the last 1 is bit 7 of byte
  // r - 1, the last byte of the rate. When n = r - 1 the two share a byte.
  // n and r are values, so the positions are found by a comparison in a
  // loop over the string rather than by an index.
  spec pad(m: Word[8]^200, n: Int, r: Int, suffix: Word[8]) -> Word[8]^200 {
    let with_suffix: Word[8]^200 = for i in 0..200 with b: Word[8]^200 = m {
      if i == n { b with [i] = b[i] ^ suffix } else { b }
    };
    for i in 0..200 with b: Word[8]^200 = with_suffix {
      if i == (r - 1) { b with [i] = b[i] ^ 0x80 } else { b }
    }
  }

  // Section 4, Algorithm 8, step 6: S = f(S ^ (P_i || 0^c)). The block P_i
  // arrives as a 200-byte string whose bytes at and beyond r are zero, so
  // the capacity is untouched.
  spec absorb(s: Word[64]^25, p: Word[8]^200) -> Word[64]^25 {
    let block: Word[64]^25 = lanes(p);
    keccak_p(for i in 0..25 with t: Word[64]^25 = s { t with [i] = s[i] ^ block[i] })
  }

  // Section 4, Algorithm 8, steps 8 and 9: Z = Trunc_r(S), and the output
  // is Trunc_d(Z). Every output here has d <= r, so one squeeze suffices and
  // step 10, another permutation, is never reached. The state string
  // (section 3.1.3) is the lanes, little-endian, in index order.
  spec squeeze_32(s: Word[64]^25) -> Word[8]^32 {
    for i in 0..4 with z: Word[8]^32 = [0; 32] {
      for j in 0..8 with b: Word[8]^32 = z { b with [8 * i + j] = le_bytes64(s[i])[j] }
    }
  }

  spec squeeze_64(s: Word[64]^25) -> Word[8]^64 {
    for i in 0..8 with z: Word[8]^64 = [0; 64] {
      for j in 0..8 with b: Word[8]^64 = z { b with [8 * i + j] = le_bytes64(s[i])[j] }
    }
  }

  spec squeeze_168(s: Word[64]^25) -> Word[8]^168 {
    for i in 0..21 with z: Word[8]^168 = [0; 168] {
      for j in 0..8 with b: Word[8]^168 = z { b with [8 * i + j] = le_bytes64(s[i])[j] }
    }
  }

  // Section 6.1: SHA3-256(M) = KECCAK[512](M || 01, 256), where KECCAK[c]
  // is the sponge over Keccak-p[1600, 24] with pad10*1 and rate 1600 - c
  // (section 5.2): c = 512 bits, r = 1088 bits = 136 bytes, suffix byte
  // 0x06, here for a message of n < 136 bytes, one padded block.
  spec sha3_256(m: Word[8]^200, n: Int) -> Word[8]^32 {
    squeeze_32(absorb([0; 25], pad(m, n, 136, 0x06)))
  }

  // Section 6.1: SHA3-512(M) = KECCAK[1024](M || 01, 512), r = 576 bits =
  // 72 bytes, for n < 72.
  spec sha3_512(m: Word[8]^200, n: Int) -> Word[8]^64 {
    squeeze_64(absorb([0; 25], pad(m, n, 72, 0x06)))
  }

  // Section 4, Algorithm 8, steps 1 through 6, for a message of exactly 200
  // bytes under SHA3-256: the first block is its first 136 bytes, the second
  // its last 64 bytes followed by the padding.
  spec sha3_256_two_blocks(m: Word[8]^200) -> Word[8]^32 {
    let first: Word[8]^200 = for i in 0..136 with b: Word[8]^200 = [0; 200] { b with [i] = m[i] };
    let second: Word[8]^200 = for i in 136..200 with b: Word[8]^200 = [0; 200] {
      b with [i - 136] = m[i]
    };
    squeeze_32(absorb(absorb([0; 25], first), pad(second, 64, 136, 0x06)))
  }

  // Section 6.2: SHAKE128(M, d) = KECCAK[256](M || 1111, d), r = 1344 bits =
  // 168 bytes, suffix byte 0x1f, for n < 168 and d = 256 bits or d = 1344
  // bits, one full rate, the unit in which implementations of ML-KEM
  // (FIPS 203) read SHAKE128 as their XOF.
  spec shake128_32(m: Word[8]^200, n: Int) -> Word[8]^32 {
    squeeze_32(absorb([0; 25], pad(m, n, 168, 0x1f)))
  }

  spec shake128_168(m: Word[8]^200, n: Int) -> Word[8]^168 {
    squeeze_168(absorb([0; 25], pad(m, n, 168, 0x1f)))
  }

  // Section 6.2: SHAKE256(M, d) = KECCAK[512](M || 1111, d), r = 136 bytes,
  // for n < 136 and d = 512 bits.
  spec shake256_64(m: Word[8]^200, n: Int) -> Word[8]^64 {
    squeeze_64(absorb([0; 25], pad(m, n, 136, 0x1f)))
  }

  // The messages: the empty string, "abc" (61 62 63), and the 200 bytes
  // 00 01 ... c7, each in a 200-byte string.
  spec empty() -> Word[8]^200 { [0; 200] }

  spec abc() -> Word[8]^200 { (([0; 200] with [0] = 0x61) with [1] = 0x62) with [2] = 0x63 }

  spec bytes_00_to_c7() -> Word[8]^200 {
    [
      0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f,
      0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e, 0x1f,
      0x20, 0x21, 0x22, 0x23, 0x24, 0x25, 0x26, 0x27, 0x28, 0x29, 0x2a, 0x2b, 0x2c, 0x2d, 0x2e, 0x2f,
      0x30, 0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39, 0x3a, 0x3b, 0x3c, 0x3d, 0x3e, 0x3f,
      0x40, 0x41, 0x42, 0x43, 0x44, 0x45, 0x46, 0x47, 0x48, 0x49, 0x4a, 0x4b, 0x4c, 0x4d, 0x4e, 0x4f,
      0x50, 0x51, 0x52, 0x53, 0x54, 0x55, 0x56, 0x57, 0x58, 0x59, 0x5a, 0x5b, 0x5c, 0x5d, 0x5e, 0x5f,
      0x60, 0x61, 0x62, 0x63, 0x64, 0x65, 0x66, 0x67, 0x68, 0x69, 0x6a, 0x6b, 0x6c, 0x6d, 0x6e, 0x6f,
      0x70, 0x71, 0x72, 0x73, 0x74, 0x75, 0x76, 0x77, 0x78, 0x79, 0x7a, 0x7b, 0x7c, 0x7d, 0x7e, 0x7f,
      0x80, 0x81, 0x82, 0x83, 0x84, 0x85, 0x86, 0x87, 0x88, 0x89, 0x8a, 0x8b, 0x8c, 0x8d, 0x8e, 0x8f,
      0x90, 0x91, 0x92, 0x93, 0x94, 0x95, 0x96, 0x97, 0x98, 0x99, 0x9a, 0x9b, 0x9c, 0x9d, 0x9e, 0x9f,
      0xa0, 0xa1, 0xa2, 0xa3, 0xa4, 0xa5, 0xa6, 0xa7, 0xa8, 0xa9, 0xaa, 0xab, 0xac, 0xad, 0xae, 0xaf,
      0xb0, 0xb1, 0xb2, 0xb3, 0xb4, 0xb5, 0xb6, 0xb7, 0xb8, 0xb9, 0xba, 0xbb, 0xbc, 0xbd, 0xbe, 0xbf,
      0xc0, 0xc1, 0xc2, 0xc3, 0xc4, 0xc5, 0xc6, 0xc7,
    ]
  }

  // Botan sha3.vec, [SHA-3(256)], In empty; hashlib.sha3_256 agrees.
  spec botan_sha3_256_empty() -> Word[8]^32 { sha3_256(empty(), 0) }

  spec botan_sha3_256_empty_expected() -> Word[8]^32 {
    [
      0xa7, 0xff, 0xc6, 0xf8, 0xbf, 0x1e, 0xd7, 0x66, 0x51, 0xc1, 0x47, 0x56, 0xa0, 0x61, 0xd6, 0x62,
      0xf5, 0x80, 0xff, 0x4d, 0xe4, 0x3b, 0x49, 0xfa, 0x82, 0xd8, 0x0a, 0x4b, 0x80, 0xf8, 0x43, 0x4a,
    ]
  }

  // Botan sha3.vec, [SHA-3(512)], In empty; hashlib.sha3_512 agrees.
  spec botan_sha3_512_empty() -> Word[8]^64 { sha3_512(empty(), 0) }

  spec botan_sha3_512_empty_expected() -> Word[8]^64 {
    [
      0xa6, 0x9f, 0x73, 0xcc, 0xa2, 0x3a, 0x9a, 0xc5, 0xc8, 0xb5, 0x67, 0xdc, 0x18, 0x5a, 0x75, 0x6e,
      0x97, 0xc9, 0x82, 0x16, 0x4f, 0xe2, 0x58, 0x59, 0xe0, 0xd1, 0xdc, 0xc1, 0x47, 0x5c, 0x80, 0xa6,
      0x15, 0xb2, 0x12, 0x3a, 0xf1, 0xf5, 0xf9, 0x4c, 0x11, 0xe3, 0xe9, 0x40, 0x2c, 0x3a, 0xc5, 0x58,
      0xf5, 0x00, 0x19, 0x9d, 0x95, 0xb6, 0xd3, 0xe3, 0x01, 0x75, 0x85, 0x86, 0x28, 0x1d, 0xcd, 0x26,
    ]
  }

  // hashlib.sha3_256(b"abc"), the widely quoted value 3a985da7...11431532;
  // CompactFIPS202.py agrees.
  spec hashlib_sha3_256_abc() -> Word[8]^32 { sha3_256(abc(), 3) }

  spec hashlib_sha3_256_abc_expected() -> Word[8]^32 {
    [
      0x3a, 0x98, 0x5d, 0xa7, 0x4f, 0xe2, 0x25, 0xb2, 0x04, 0x5c, 0x17, 0x2d, 0x6b, 0xd3, 0x90, 0xbd,
      0x85, 0x5f, 0x08, 0x6e, 0x3e, 0x9d, 0x52, 0x5b, 0x46, 0xbf, 0xe2, 0x45, 0x11, 0x43, 0x15, 0x32,
    ]
  }

  // hashlib.sha3_512(b"abc"); CompactFIPS202.py agrees.
  spec hashlib_sha3_512_abc() -> Word[8]^64 { sha3_512(abc(), 3) }

  spec hashlib_sha3_512_abc_expected() -> Word[8]^64 {
    [
      0xb7, 0x51, 0x85, 0x0b, 0x1a, 0x57, 0x16, 0x8a, 0x56, 0x93, 0xcd, 0x92, 0x4b, 0x6b, 0x09, 0x6e,
      0x08, 0xf6, 0x21, 0x82, 0x74, 0x44, 0xf7, 0x0d, 0x88, 0x4f, 0x5d, 0x02, 0x40, 0xd2, 0x71, 0x2e,
      0x10, 0xe1, 0x16, 0xe9, 0x19, 0x2a, 0xf3, 0xc9, 0x1a, 0x7e, 0xc5, 0x76, 0x47, 0xe3, 0x93, 0x40,
      0x57, 0x34, 0x0b, 0x4c, 0xf4, 0x08, 0xd5, 0xa5, 0x65, 0x92, 0xf8, 0x27, 0x4e, 0xec, 0x53, 0xf0,
    ]
  }

  // hashlib.shake_128(b"").digest(32); Botan shake.vec, [SHAKE-128], In
  // empty, gives the first 16 bytes; CompactFIPS202.py agrees.
  spec hashlib_shake128_empty_32() -> Word[8]^32 { shake128_32(empty(), 0) }

  spec hashlib_shake128_empty_32_expected() -> Word[8]^32 {
    [
      0x7f, 0x9c, 0x2b, 0xa4, 0xe8, 0x8f, 0x82, 0x7d, 0x61, 0x60, 0x45, 0x50, 0x76, 0x05, 0x85, 0x3e,
      0xd7, 0x3b, 0x80, 0x93, 0xf6, 0xef, 0xbc, 0x88, 0xeb, 0x1a, 0x6e, 0xac, 0xfa, 0x66, 0xef, 0x26,
    ]
  }

  // hashlib.shake_256(b"abc").digest(64); CompactFIPS202.py agrees.
  spec hashlib_shake256_abc_64() -> Word[8]^64 { shake256_64(abc(), 3) }

  spec hashlib_shake256_abc_64_expected() -> Word[8]^64 {
    [
      0x48, 0x33, 0x66, 0x60, 0x13, 0x60, 0xa8, 0x77, 0x1c, 0x68, 0x63, 0x08, 0x0c, 0xc4, 0x11, 0x4d,
      0x8d, 0xb4, 0x45, 0x30, 0xf8, 0xf1, 0xe1, 0xee, 0x4f, 0x94, 0xea, 0x37, 0xe7, 0x8b, 0x57, 0x39,
      0xd5, 0xa1, 0x5b, 0xef, 0x18, 0x6a, 0x53, 0x86, 0xc7, 0x57, 0x44, 0xc0, 0x52, 0x7e, 0x1f, 0xaa,
      0x9f, 0x87, 0x26, 0xe4, 0x62, 0xa1, 0x2a, 0x4f, 0xeb, 0x06, 0xbd, 0x88, 0x01, 0xe7, 0x51, 0xe4,
    ]
  }

  // hashlib.sha3_256(bytes(range(200))): two blocks under rate 136, the
  // second holding 64 message bytes and the padding; CompactFIPS202.py
  // agrees.
  spec hashlib_sha3_256_200_bytes() -> Word[8]^32 { sha3_256_two_blocks(bytes_00_to_c7()) }

  spec hashlib_sha3_256_200_bytes_expected() -> Word[8]^32 {
    [
      0x5f, 0x72, 0x8f, 0x63, 0xbf, 0x5e, 0xe4, 0x8c, 0x77, 0xf4, 0x53, 0xc0, 0x49, 0x03, 0x98, 0xfa,
      0x64, 0x5b, 0x8d, 0x4c, 0x4e, 0x56, 0xbe, 0x9a, 0x41, 0xcf, 0xec, 0x34, 0x4d, 0x6c, 0xa8, 0x99,
    ]
  }

  // hashlib.shake_128(b"abc").digest(168), one full rate of output;
  // CompactFIPS202.py agrees.
  spec hashlib_shake128_abc_168() -> Word[8]^168 { shake128_168(abc(), 3) }

  spec hashlib_shake128_abc_168_expected() -> Word[8]^168 {
    [
      0x58, 0x81, 0x09, 0x2d, 0xd8, 0x18, 0xbf, 0x5c, 0xf8, 0xa3, 0xdd, 0xb7, 0x93, 0xfb, 0xcb, 0xa7,
      0x40, 0x97, 0xd5, 0xc5, 0x26, 0xa6, 0xd3, 0x5f, 0x97, 0xb8, 0x33, 0x51, 0x94, 0x0f, 0x2c, 0xc8,
      0x44, 0xc5, 0x0a, 0xf3, 0x2a, 0xcd, 0x3f, 0x2c, 0xdd, 0x06, 0x65, 0x68, 0x70, 0x6f, 0x50, 0x9b,
      0xc1, 0xbd, 0xde, 0x58, 0x29, 0x5d, 0xae, 0x3f, 0x89, 0x1a, 0x9a, 0x0f, 0xca, 0x57, 0x83, 0x78,
      0x9a, 0x41, 0xf8, 0x61, 0x12, 0x14, 0xce, 0x61, 0x23, 0x94, 0xdf, 0x28, 0x6a, 0x62, 0xd1, 0xa2,
      0x25, 0x2a, 0xa9, 0x4d, 0xb9, 0xc5, 0x38, 0x95, 0x6c, 0x71, 0x7d, 0xc2, 0xbe, 0xd4, 0xf2, 0x32,
      0xa0, 0x29, 0x4c, 0x85, 0x7c, 0x73, 0x0a, 0xa1, 0x60, 0x67, 0xac, 0x10, 0x62, 0xf1, 0x20, 0x1f,
      0xb0, 0xd3, 0x77, 0xcf, 0xb9, 0xcd, 0xe4, 0xc6, 0x35, 0x99, 0xb2, 0x7f, 0x34, 0x62, 0xbb, 0xa4,
      0xa0, 0xed, 0x29, 0x6c, 0x80, 0x1f, 0x9f, 0xf7, 0xf5, 0x73, 0x02, 0xbb, 0x30, 0x76, 0xee, 0x14,
      0x5f, 0x97, 0xa3, 0x2a, 0xe6, 0x8e, 0x76, 0xab, 0x66, 0xc4, 0x8d, 0x51, 0x67, 0x5b, 0xd4, 0x9a,
      0xcc, 0x29, 0x08, 0x2f, 0x56, 0x47, 0x58, 0x4e,
    ]
  }

  test "FIPS 202 SHA3-256 of the empty message" {
    botan_sha3_256_empty() == botan_sha3_256_empty_expected()
  }

  test "FIPS 202 SHA3-512 of the empty message" {
    botan_sha3_512_empty() == botan_sha3_512_empty_expected()
  }

  test "FIPS 202 SHA3-256 of abc" {
    hashlib_sha3_256_abc() == hashlib_sha3_256_abc_expected()
  }

  test "FIPS 202 SHA3-512 of abc" {
    hashlib_sha3_512_abc() == hashlib_sha3_512_abc_expected()
  }

  test "FIPS 202 SHAKE128 of the empty message, 32 bytes" {
    hashlib_shake128_empty_32() == hashlib_shake128_empty_32_expected()
  }

  test "FIPS 202 SHAKE256 of abc, 64 bytes" {
    hashlib_shake256_abc_64() == hashlib_shake256_abc_64_expected()
  }

  test "FIPS 202 SHA3-256 of 200 bytes, two blocks" {
    hashlib_sha3_256_200_bytes() == hashlib_sha3_256_200_bytes_expected()
  }

  test "FIPS 202 SHAKE128 of abc, one rate of output" {
    hashlib_shake128_abc_168() == hashlib_shake128_abc_168_expected()
  }
}
```

## Part VIII: Implementation Stratum (`impl`) & Memory Model

**Status: Proposed.** This part is not the Current `spec` stratum. `orangec` 0.0.1
through slice S3u does not implement `impl`. A diagnostic code named below is
Current only when Part XV records that slice S3u emits it.

### §53. Imperative Execution Semantics and Place Logic

The `impl` stratum formalizes effectful, terminating imperative procedures engineered
for direct compilation to native physical machine code without a runtime system or
garbage collector.

#### 1. Place Syntax and Separation

While the specification stratum computes purely over mathematical values $\mathbb{V}$,
the implementation stratum executes over **places** (memory locations, stack slots,
and register allocations):

$$
\begin{array}{rcll}
p & ::= & x & (\text{base variable place}) \\
  & \mid & p[e] & (\text{indexed array place}) \\
  & \mid & p.j & (\text{tuple field place})
\end{array}
$$

#### 2. Abstract Command Syntax

$$
\begin{array}{rcll}
s & ::= & \texttt{skip} & (\text{null operation}) \\
  & \mid & p := e & (\text{place mutation}) \\
  & \mid & s_1; \ s_2 & (\text{sequential composition}) \\
  & \mid & \text{if } b \ \{ s_1 \} \ \text{else } \{ s_2 \} & (\text{conditional branch}) \\
  & \mid & \text{while } b \ \text{inv } I \ \text{var } V \ \{ s \} & (\text{bounded iteration}) \\
  & \mid & \text{let } x : T = e; \ s & (\text{stack local allocation}) \\
  & \mid & \text{let } (\&'r \text{mut } l, \&'r \text{mut } r) = \text{split\_mut}(p, k); \ s & (\text{disjoint mutable slice}) \\
  & \mid & \text{erase } p; & (\text{mandatory memory zeroization}) \\
  & \mid & \text{return } e; & (\text{procedure return})
\end{array}
$$

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

### §55. Affine Ownership, Move Semantics, and Capability Borrowing (`&T`, `&mut T`)

**Status: Proposed** (D-004). The Current compiler does not enforce an
ownership calculus. `orangec check` accepts `impl f() {}` and does not
evaluate it. `impl f() -> Int { 1 }` is `ORC0101`, message
`` typed bodies are allowed only on `spec` functions ``, label
`` an `impl` function cannot have a typed body ``.
`impl compute(x: Word[32]) -> Word[32] { x + 1 }` is two `ORC0101`
diagnostics: `` `impl` functions have an empty parameter list ``, then the
same typed-body rejection. The semantic code `ORC0202` is the gate in
`semantics.rs` for a typed body whose kind is not `spec`. The parser does
not build that body, so `orangec check` of these programs does not emit
`ORC0202`. `&` is the bitwise operator `And` in the lexer, not a borrow.
`spec f(x: &Int)` is `ORC0101`, `expected an identifier for the parameter
type`, because the token is `AMPERSAND`. There is no type `&T` and no type
`&mut T`, and no diagnostic that rejects a use after a move, because a
`spec` binding is not a place that can be moved. The judgments below are
the proposed calculus for `impl`. They are not checks `orangec` runs.

The proposed rule is an affine capability calculus for memory safety.

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

$$
\begin{array}{rcll}
P, Q & ::= & \text{emp} & (\text{empty heap}) \\
      & \mid & p \mapsto v & (\text{place } p \text{ points to value } v) \\
      & \mid & P \ast Q & (\text{spatial separating conjunction}) \\
      & \mid & P \mathbin{-\!\!*} Q & (\text{magic wand / separating implication}) \\
      & \mid & P \land Q \mid P \lor Q \mid \neg P & (\text{classical connectives}) \\
      & \mid & \forall x.\ P \mid \exists x.\ P & (\text{first-order quantifiers})
\end{array}
$$

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

**Status: Proposed.** This part is not the Current `spec` stratum. `orangec` 0.0.1
through slice S3u does not implement `machine impl`.

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

**Status: Proposed.** This part is not the Current `spec` stratum. `orangec` 0.0.1
through slice S3u does not implement information-flow labels or the policies named
below.

### §67. The Information Flow Lattice ($\text{public} \sqsubseteq \text{secret}$)

**Status: Proposed.** No OEP in this checkout implements the lattice, and
`orangec` emits no diagnostic for it. `public` and `secret` are not reserved
words: `orangec check` accepts `spec public(secret: Int) -> Int { secret }`.
A `spec` conditional evaluates one branch (§23). That is a control-flow fact
of `begin_branch`, and it is not a secrecy check. OEP-0022, section P5, says
secrecy qualifiers are orthogonal to arithmetic representation and that a
secret-dependent branch or address cannot pass merely because arithmetic
refinement succeeds; it does not define a checker. The Book's section
"Secrecy in the type system" is the same proposal: labels on values, a type
error for a branch or an index computed from a secret, and an explicit
declassification. The two-point lattice below is that proposal. It is not a
judgment of the S3t compiler.

The proposed lattice is:

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

$$
\Sigma_{\text{leak}} = \left\{
\begin{array}{ll}
\mathbf{Fetch}(\text{pc}), & (\text{instruction fetch at address pc}) \\
\mathbf{Branch}(\text{pc}, \text{target}), & (\text{control branch taken to target}) \\
\mathbf{MemRead}(\text{addr}, \text{width}), & (\text{memory bus read at address}) \\
\mathbf{MemWrite}(\text{addr}, \text{width}), & (\text{memory bus write at address}) \\
\mathbf{ALULatency}(\text{op}, c), & (\text{ALU instruction execution taking } c \text{ cycles}) \\
\mathbf{SpecBarrier}(\text{pc}) & (\text{speculation fence executed})
\end{array}
\right\}
$$

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

**Status: Proposed.** This part is not the Current `spec` stratum. `orangec` 0.0.1
through slice S3u does not implement `game`.

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

**Status: Proposed.** This part is not the Current `spec` stratum. The
`orangec` 0.0.1 binary in this tree reports slice S3t and does not implement
`proof` or `orange-check`.

### §76. Propositions as Types and the $\text{Prop}$ Universe

**Current rejection.** `proof` is a reserved word. `proof f() {}` is
`ORC0103`, message `` expected a `spec` or `impl` function declaration ``,
label `` this token cannot begin a module member ``. `spec proof() -> Int { 1 }`
is `ORC0101`, message `` expected an identifier for the function name ``,
token `KW_PROOF`, note `` reserved words cannot be used as names ``. `claim`
(`KW_CLAIM`) and `game` (`KW_GAME`) are the same pair of diagnostics. There
is no `Prop` type and no proof term in this tree. The judgments below are
the proposal recorded for the proof stratum in §4 (D-006, D-007). They are
not checks `orangec` runs.

The proposed `proof` stratum is an intuitionistic dependent type theory:

$$\text{Propositions-as-Types} \qquad \text{Proofs-as-Terms}$$

1. **The $\text{Prop}$ Universe:**
   - Resides at the base of the universe hierarchy: $\text{Prop} : \text{Type}_1 : \text{Type}_2 \dots$
   - Any proposition $P : \text{Prop}$ is proved by constructing an elaborated term $\pi$ such that:
     $$\vdash \pi : P$$
2. **Definitional Proof Irrelevance:**
   Any two proofs of the same proposition are definitionally equal:
   $$\forall \pi_1, \pi_2 : P.\ \pi_1 \equiv \pi_2 \quad (\text{for } P : \text{Prop})$$
   Proof terms are erased during native code generation, leaving zero footprint.

### §77. Functional Refinement Relations

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

### §79. Weakest Precondition Calculus and Verification Conditions

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

**Status: Proposed.** This part is not the Current `spec` stratum. `orangec` 0.0.1
through slice S3u does not implement `claim`.

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
| **CF-10** | `test_result` | $\text{TestsPass}(P, \text{VectorSet}) \iff \bigwedge_{v \in \text{Vectors}} (P(v.\text{in}) == v.\text{out})$ | Known-answer test vectors and differential fuzzing results. |

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
| $\mathbf{unsupported}$ | $\mathbf{unsupported}$ | $\mathbf{unsupported}$ | $\mathbf{unsupported}$ | $\mathbf{not\_satisfied}$ |
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

**Status: Proposed.** This part is not the Current `spec` stratum. `orangec` 0.0.1
through slice S3u does not generate C or Rust bindings.

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

$$
\text{align}(\tau) = \begin{cases}
W / 8 & \text{if } \tau = \text{Word}[W] \quad (W \in \{8, 16, 32, 64\}) \\
1 & \text{if } \tau = \text{Bool} \\
\text{align}(T) & \text{if } \tau = T^n \\
\max_{0 \le i < k} \text{align}(T_i) & \text{if } \tau = (T_0, \dots, T_{k-1})
\end{cases}
$$

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

**Status: Current codes, through slice S3u.** The identifiers `ORC0001`
through `ORC0301` are codes the implemented compiler emits. Each entry below
is a lookup for that code. It is not a second checker, and it is not a
transcript of `orangec` output: invented rendering blocks have been removed.
Where an entry and `compiler/crates/orange-compiler/src/diagnostic.rs` differ,
the compiler source controls.

S3u's rank limit is `ORC0203`. A shape whose scalar product exceeds 65,536 is
`ORC0221`. A path that indexes past the scalars is `ORC0224`. A path with a
slice, an empty index, or a fifth index is `ORC0101`. Driver limits such as
the 16 MiB source ceiling are `ORC1003`, outside this range.

### §93. Diagnostic Philosophy, Severity Structure, and Error Budgets

1. **Permanence of Diagnostic Identifiers:**
   Diagnostic codes in Orange are immutable elements of the compiler's user-facing
   interface. Existing numeric identifiers MUST NOT be repurposed, redefined, or
   reordered across releases.
2. **Severity Hierarchy:**
   - The severity of all compile-time diagnostic failures is fixed as `error`.
   - A lexical or parse error prevents semantic analysis of that source. A
     semantic error prevents evaluation. The Current driver does not generate code.
3. **Structured Diagnostic Record:**
   Every emitted diagnostic is a record:
   $$\text{Diag} = \langle \text{Code}, \text{Severity}, \text{PrimarySpan}, \text{Label}, \text{SecondarySpans}, \text{Notes} \rangle$$
4. **Deterministic Error Suppression Budgets:**
   - Each of lexing, parsing, and semantic analysis reports at most **100**
     ordinary errors, then one suppression diagnostic: `ORC0007`, `ORC0105`, or
     `ORC0208`.

The record is the struct `Diagnostic` in
`compiler/crates/orange-compiler/src/diagnostic.rs`. Its fields are
`severity`, `code`, `message`, `primary_span`, `label`, `secondary_spans`,
and `notes`. `Severity` has one variant, `Error`, whose stable spelling is
`error`. `Diagnostic::error` builds one. A rendered diagnostic begins
`error[CODE]:` and the message. The label is the primary underline. Each
secondary span carries its own label. Notes are rendered `= note:`.

The ceilings of 100 are three constants:

- Lexing: `MAX_DIAGNOSTICS_PER_SOURCE` in `lexer.rs`, then `ORC0007`.
- Parsing: `MAX_PARSE_DIAGNOSTICS_PER_SOURCE` in `parser.rs`, then `ORC0105`.
- Semantics: `MAX_SEMANTIC_DIAGNOSTICS_PER_SOURCE` in `semantics.rs`, then
  `ORC0208`, message
  `` too many semantic errors; further errors are suppressed ``, note
  `` at most 100 ordinary semantic diagnostics are retained per source ``.

`begin_report` counts every emission attempt as one semantic event, including
an attempt the budget then drops. The event ceiling is
`MAX_SEMANTIC_EVENTS_PER_SOURCE`, 1,048,576. Exhausting it is `ORC0209`.

The codes are the `DiagnosticCode` enum in that file. Its comment says an
existing meaning must not be silently reused. Sections 94 through 100 name
those codes. Where a catalog example and a run of `orangec check` differ,
the run controls (§96, `ORC0202`).

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
  $$\text{Trigger} \iff \text{Count}(\text{NonTriviaTokens excluding EOF}) > 262{,}144$$
- **Rationale:** Protects the compiler against algorithmic complexity attacks and out-of-memory crashes.
- **Remediation:** Partition large compilation units into multiple modular source files.

#### `ORC0007` — `TooManyLexicalErrors`

- **Subsystem:** Lexical Analyzer
- **Formal Trigger Predicate:**
  $$\text{Trigger} \iff \text{Count}(\text{ordinary lexical errors}) > 100$$
- **Remediation:** Fix early lexical errors and re-run compilation.

#### `ORC0008` — `LexicalResourceLimit`

- **Subsystem:** Lexical Analyzer
- **Formal Trigger Predicate:**
  The lexer could not reserve its bounded token stream, or the cursor was not
  on a UTF-8 boundary. A source longer than 16 MiB is `ORC1003`, not `ORC0008`.
- **Remediation:** This code is a resource failure of the lexer, not a file-size
  rejection. Split a source that trips `ORC0006` before it can trip `ORC0008`.

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
  $$\text{Trigger} \iff \text{Count}(\text{ordinary parse errors}) > 100$$
- **Remediation:** Resolve initial syntactic failures.

#### `ORC0106` — `ParserResourceLimit`

- **Subsystem:** Parser
- **Formal Trigger Predicate:**
  The parser stops when it cannot stay inside its budgets: 262,144 syntax
  nodes, 1,048,576 parse events, expression nesting 64, expression height 256,
  or recovery depth 64.
- **Remediation:** Simplify or split the source. The message names the budget.

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

$$
\mathrm{Trigger}(f) \iff \mathrm{body}(f) = \mathrm{Typed} \land \mathrm{kind}(f) \ne \texttt{spec}
$$

`semantics.rs` emits this when that predicate holds. The message is
`` typed bodies are supported only on `spec` functions ``, the label is
`` this `impl` function has no semantics in the current fragment ``, and the
note is
`` use an empty `impl` body or move the typed body to a `spec` function ``.
The parser does not build a typed `impl` body, so `orangec check` of the
program below emits two `ORC0101` diagnostics and does not emit `ORC0202`:
`` `impl` functions have an empty parameter list ``, then
`` typed bodies are allowed only on `spec` functions ``.

```orange
edition 2026;
module test_impl {
    impl compute(x: Word[32]) -> Word[32] { x + 1 }
}
```

The unit test `typed_impls_and_unadmitted_types_fail_closed` reaches
`ORC0202` by parsing a typed `spec` and setting the function's kind to
`impl` before analysis.

- **Remediation:** Write the typed body as `spec`. An empty `impl name() {}`
  declares the name and has no body for this gate to reject.

#### `ORC0203` — `UnsupportedType`

- **Subsystem:** Semantic Analyzer (Type Elaborator)
- **Formal Trigger Predicate:**
  $$\text{Trigger}(\tau) \iff \tau \notin \text{Universe}(\text{Orange 2026}) \lor \operatorname{rank}(\tau) > 4$$
  where the universe is $\{\text{Int}, \text{Bool}\} \cup \{\text{Word}[W] \mid W \in \{8,16,32,64\}\} \cup \{\text{Mod}[m]\} \cup \{T^n \mid \operatorname{rank}(T^n) \le 4\} \cup \{(T_0, \dots, T_{k-1})\}$. `Byte` is not in it (§27).
- **Theoretical Rationale:** IEEE-754 floating-point numbers, unbounded dynamic pointers,
  and recursive algebraic data types introduce nondeterministic rounding, platform divergence,
  and side-channel leakages. They are strictly excluded from Orange's type universe.
  Slice S3u also uses this code for a fifth array dimension (§25.1). The message is
  `` `Hyper` already has 4 array dimensions ``, labeled "arrays have at most 4 dimensions".
- **Erroneous Example:**

  ```orange
  edition 2026;
  module bad_type {
      spec float_op(x: Float64) -> Float64 { x }
  }
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
  $$\text{Trigger}(x) \iff x \in \mathbb{Z} \land |x| \ge 2^{16384}$$
- **Rationale:** `Int` is exact up to 16,384 significant bits. A wider magnitude
  is rejected rather than truncated. `1 << 4096` is not an example: shifts are
  not defined on `Int` (`ORC0215`).
- **Trigger:** A literal or a constant integer expression whose magnitude has
  more than 16,384 significant bits. The message is
  `integer magnitude exceeds the 16384-significant-bit limit`.
  `1 << 4096` is not this code: a shift of an `Int` is `ORC0215`.


- **Remediation:** Keep the magnitude within 16,384 significant bits.

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
  $$\text{Trigger} \iff \text{Count}(\text{ordinary semantic errors}) > 100$$
- **Theoretical Rationale:** Bounds diagnostic output to prevent terminal flooding and
  cascading nonsensical error reports.
- **Remediation:** Resolve initial semantic typing failures and re-run `orangec check`.

#### `ORC0209` — `SemanticResourceLimit`

- **Subsystem:** Semantic Analyzer (Memory Monitor)
- **Formal Trigger Predicate:**
  Semantic analysis could not reserve a bounded table, or it exceeded
  262,144 Core nodes or 1,048,576 semantic events. The message is
  `semantic analysis resource limit exceeded`. There is no type-inference
  depth of 1,024 and no 64 MiB type-table budget.
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
  $$\text{Trigger}(n_1, \dots, n_r) \iff (\exists j.\ n_j < 1 \lor n_j > 65,536) \lor \prod_{j=1}^{r} n_j > 65,536$$
  with $1 \le r \le 4$. A product over the limit reports "an array shape has N scalar
  elements, exceeding 65536".
- **Theoretical Rationale:** Zero-length arrays introduce degenerate algebraic properties
  and indexing ambiguities. The scalar-leaf ceiling of 65,536 is the same bound at every
  admitted rank (§25.2).
- **Erroneous Example:**

  ```orange
  edition 2026;
  module bad_array {
      type Empty = Word[8]^0;      // Zero length is prohibited
      type Massive = Word[32]^70000; // Exceeds 65,536 ceiling
  }
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
  Slice S3u uses the same code when an update path names more indices than the rank.
  The message is "only an array can be indexed, but this selects within `T`", with the
  secondary label "this array has fewer dimensions" (§25.5).
- **Erroneous Example:**

  ```orange
  edition 2026;
  module not_arr {
      spec index_word(w: Word[32]) -> Word[8] {
          w[0] // Cannot index directly into scalar Word[32]
      }
  }
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


- **Remediation:** Put `missing_module.or` in the directory of the root file. There is no `Orange.toml` search path (§15).

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


- **Remediation:** Add `use sha256;` to the module's import header.

#### `ORC0230` — `ModuleCycle`

- **Subsystem:** Semantic Analyzer (module-use depth-first search)
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


- **Remediation:** Specify an admitted modulus $2 \le m \le 2^{521} - 1$.

#### `ORC0233` — `DuplicateTypeName`

- **Subsystem:** Semantic Analyzer (Type Alias Resolver)
- **Formal Trigger Predicate:**
  $$\text{Trigger}(T) \iff T \in \text{BuiltinTypes} \lor T \in \text{dom}(\text{DeclaredAliases})$$
- **Rationale:** A `type` declaration cannot name `Int`, `Bool`, `Word`, or `Mod`, and it cannot repeat a name. `Byte` is not built in.
- **Erroneous Example:**

  ```orange
  edition 2026;
  module dup_type {
      type Int = Word[64]; // Cannot redefine builtin type Int
  }
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
  A declared range `n in a..b` is rejected when it is empty, when `b` is
  greater than 65,536, or when `a` is not strictly less than `b`. A function
  with more than 256 instances is the same code. The note is: `a < b <= 65536`,
  and a function has at most 256 instances.
- **Rationale:** This code is about the declaration of the range, not about a
  call. A call whose size is outside the half-open range is `ORC0239`.
- **Erroneous Example:**

  ```orange
  edition 2026;
  module size_range {
      spec pad[n in 1..1](x: Word[8]^n) -> Word[8]^64 { x ++ x }
  }
  ```

  `1..1` is empty. `n in 1..64` is the lengths 1 through 63, so `pad[64]` is
  not an instance of that function.


- **Remediation:** Declare a non-empty range with `a < b <= 65536` and at most 256 instances. For `n in 1..64`, the largest instance is `pad[63]`.

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

  The message is `reference evaluation step limit exceeded`, with the note
  `at most N evaluation steps are permitted`.


- **Remediation:** Increase step budget using `--steps <COUNT>`:

  ```console
  orangec eval --steps 2097152 sha256.or
  ```

---

## Part XVI: Toolchain, Evaluator & Formal EBNF Grammar

**Status: Proposed.** The `orangec` commands and numeric limits that slice S3u
implements are Current and are the ones named in §101 and §102. The EBNF in §103,
and any behavior this part states that the S3u compiler does not exercise, are
Proposed.

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
| `lex FILE` | Prints the deterministic non-trivia token stream. | 0 on success. |
| `replay` | Replays one Boolean function instance on a witness file (`--function`, `--witness`). | Prints `holds_for_this_witness` or `falsified`. |
| `keygen` | Writes a secret key for a scheme. The default scheme is `xchacha20_poly1305`. | |
| `enc` / `dec` | Seal or open a file with the scheme its key belongs to. | Reference code. Not constant-time. |
| `schemes` | Lists the built-in sealing schemes, or describes the named ones. | |

#### Global Options

- `--edition 2026`: Explicitly specifies the source edition.
- `--steps <COUNT>`: Evaluation step budget, from 1 through 1,073,741,824. The default is 1,048,576.
- `--spec <NAME>`: Evaluate only this function without parameters. Repeatable, at most 64 names. `eval` only.
- `--stats`: Report the steps each function or test used, on stderr. It does not report memory words. It applies to `eval`, `test`, and `replay`.
- `--version`: Emits package version, edition, and latest implemented slice:

  ```console
  $ orangec --version
  orangec 0.0.1 (Orange edition 2026; implemented slice S3u)
  ```

### §102. Deterministic Resource Limits and Denial-of-Service Defense

| Parameter | Implemented limit | Diagnostic on Breach |
| :--- | :--- | :--- |
| **Maximum Source File Size** | 16,777,216 bytes (16 MiB) | `ORC1003` |
| **Maximum Non-Trivia Tokens** | 262,144, excluding EOF | `ORC0006` |
| **Maximum Reported Errors** | 100 ordinary errors per phase, then one suppression | `ORC0007`, `ORC0105`, `ORC0208` |
| **Maximum Integer Magnitude** | 16,384 significant bits ($|x| < 2^{16384}$) | `ORC0205` |
| **Admitted Word Bit Widths ($W$)** | Exactly $\{8, 16, 32, 64\}$ bits | `ORC0204` |
| **Admitted Modular Moduli ($m$)** | $2 \le m \le 2^{521} - 1$ | `ORC0232` |
| **Maximum Array Length ($n$)** | 65,536 elements ($2^{16}$) | `ORC0221` |
| **Maximum Array Rank** | 4 | `ORC0203` |
| **Maximum Nested Array Elements** | 65,536 scalar elements ($2^{16}$), product of every axis | `ORC0221` |
| **Tuple Arity Range ($k$)** | $2 \le k \le 16$ | `ORC0234` |
| **Default Evaluator Step Budget** | 1,048,576 steps ($2^{20}$) | `ORC0301` |
| **Documentation Work Items** | 1,048,576 items ($2^{20}$) | `ORC0260` |
| **Documentation Output Size** | 16,777,216 HTML bytes (16 MiB) | `ORC0260` |

### §103. Proposed Surface Grammar

This grammar is Proposed. It is not the Current concrete syntax. The
Current member forms are `spec`, `impl`, and `test` (§14). `Byte` is not a
type (§27). A bare carriage return is whitespace (§7). Where this section and
Parts I–V differ, Parts I–V control.

```text
(* Proposed surface. Not the Current grammar. *)

source_file         = byte* ; (* UTF-8, at most 16 MiB; U+FEFF is ORC0001 *)
byte                = "\x00".."\xFF" ;
any_char            = ? one Unicode scalar value, the UTF-8 decoding of §5 ? ;
any_ascii_printable = " ".."~" ;

whitespace          = " " | "\t" | "\n" | "\r" | "\r\n" ;
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
                    | test_decl ;

(* `game`, `proof`, and `claim` are not productions. A token in their
   place is ORC0103: expected a `spec` or `impl` function declaration.
   There is no game_decl, proof_decl, or claim_decl. *)

impl_decl           = "impl" identifier "(" ")" "{" "}" ;

(* Types *)
parsed_type         = scalar_type
                    | array_type
                    | tuple_type
                    | identifier ;

scalar_type         = "Int"
                    | "Bool"
                    | "Word" "[" integer_literal "]"
                    | "Mod" "[" modulus_expr "]" ;

array_type          = parsed_type "^" size_expr ;
tuple_type          = "(" parsed_type ("," parsed_type)+ ")" ;

modulus_expr        = expression ; (* the parser parses an expression inside `Mod[...]` *)
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
(* The repetition is not the Current parser. ORC0108 rejects an operator
   from another group, a second shift or rotation, a second `/` or `%`,
   and a second comparison, unless parentheses group one of them.
   `binary_op` is the infix set of `BinaryOperator` in parser.rs. *)
binary_op           = "+" | "-" | "*" | "/" | "%"
                    | "&" | "|" | "^"
                    | "<<" | ">>" | "<<<" | ">>>"
                    | "==" | "!=" | "<" | "<=" | ">" | ">="
                    | "&&" | "||"
                    | "++" ;
unary_expr          = ("-" | "~" | "!")? primary_expr ;

literal_expr        = integer_literal | string_literal | hex_string_literal ;
(* `true` and `false` are identifiers. Section 32 resolves them as
   `Bool` literals when no earlier binding uses the spelling. *)

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
