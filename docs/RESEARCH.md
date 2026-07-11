# Research and landscape analysis

Status: research baseline for architecture decisions

Snapshot date: 2026-07-11

## 1. Method and limits

This analysis uses official project documentation, standards bodies, primary
repositories, and peer-reviewed or author-published papers. It asks a product
question rather than trying to rank theorem provers in the abstract:

> What must Orange own, and what should it reuse, to connect a standards-pinned
> cryptographic specification to optimized shipped bytes and accurately scoped
> assurance evidence?

The repository contained only the original two-line README at the start of the
analysis. There was no implementation, issue history, license, or architecture
to infer. All design conclusions below are recommendations, not descriptions of
existing Orange behavior.

The landscape moves quickly. Version-specific integrations must be rechecked at
each architecture and release gate. In particular, supported architectures,
proof-certificate formats, standards errata, and validation protocols are not
safe to freeze from this document alone.

## 2. Executive finding

There are excellent systems for individual spans of the problem:

- Cryptol is strong at executable, bit-precise specifications, while SAW proves
  relationships to C, Rust, Java, and LLVM-level implementations.
- hacspec and hax use Rust familiarity to write specifications and translate
  Rust subsets into several proof environments.
- F*, HACL*, Vale, and EverCrypt demonstrate verified high- and low-level
  cryptographic software deployed through a C-facing ecosystem.
- Jasmin joins predictable low-level programming, a verified compiler, and
  EasyCrypt-based functional and side-channel reasoning.
- Fiat-Crypto synthesizes field-arithmetic implementations correct by
  construction from mathematical parameters.
- EasyCrypt and SSProve address computational, game-based security arguments.
- Rocq and Lean provide small-kernel foundations for mechanized semantics and
  checked proof terms.

The important gap is not “nobody verifies cryptography.” It is that the normal
end-to-end story still crosses languages, semantic models, compilers, artifact
formats, and manually maintained glue. The 2024 *Last Yard* work explicitly
frames a missing unified foundational end-to-end framework and connects
hacspec, Jasmin, and SSProve in Rocq for an AES case study.

Orange should therefore not compete by adding one more isolated specification
syntax or one more solver wrapper. Its useful identity is:

> A claim-oriented language and build system that emits a proof-carrying
> vertical artifact: standards provenance, executable specification,
> implementation, target-indexed leakage model, checked transformations,
> object code, ABI metadata, validation evidence, and explicit assumptions.

This is a synthesis of the evidence below, not a claim that every individual
mechanism is novel.

## 3. Landscape comparison

| System | Primary strength | Specification | Efficient implementation | Main evidence path | Lesson for Orange |
| --- | --- | --- | --- | --- | --- |
| Cryptol + SAW | Bit-precise specs and equivalence checking | Native, executable, size-polymorphic crypto DSL | Verifies existing C, Rust, Java, LLVM and related code through symbolic execution | SAT/SMT-backed SAW proofs and scripts | Bit vectors, sequences, parameterized widths, counterexamples, and equivalence workflows are table stakes; proof/build scripting must not become an untyped second language. |
| hacspec + hax | Familiar Rust-shaped specs and multi-prover translation | Functional Rust subset | hax translates a large safe-Rust subset; it is not itself a native crypto optimizer | F\*, Lean, Rocq, SSProve, and experimental protocol backends | Familiar syntax lowers adoption cost, but every translation boundary needs a stated and checked semantic relationship. |
| F\* + HACL\* + EverCrypt | Proof-oriented high-assurance code with deployed C output | Refinement/dependent types and effects | Low\* and KaRaMeL C extraction; Vale supplies verified assembly | F\* verification with heavy Z3 automation; project-specific proofs | Refinement types and effect tracking work in real crypto. Orange should improve evidence portability and avoid accepting an opaque solver “yes” as its native proof format. |
| Vale | Verified, high-performance assembly | Contracts around low-level code | x86/x64/ARM assembly families | Dafny or F* verification | Target-specific assembly and explicit machine models are sometimes necessary; low-level escape hatches must carry proofs, not waive them. |
| Jasmin + EasyCrypt | Predictable high-speed crypto and verified lowering | Imperative source with formal semantics | Verified compiler to native assembly, including target intrinsics | Static safety checks, constant-time analysis, EasyCrypt extraction/proofs, compiler theorems | A crypto compiler needs predictable code generation and property preservation, not only semantic equivalence at a high IR. |
| Fiat-Crypto | Correct-by-construction field arithmetic | Mathematical field/modulus parameters in Rocq | Generates C and other language outputs through proven synthesis | Rocq proofs over synthesis and rewriters | Synthesis is valuable for structured subdomains. Orange should expose verified generators as packages rather than force every optimized routine to be handwritten. |
| EasyCrypt | Computational security proofs | Probabilistic games, adversaries, oracles, assumptions | Not primarily a native-code compiler | Interactive relational and game-based proofs | “Implementation equals spec” is not a cryptographic security theorem. Orange needs a separate game stratum or a first-class checked bridge to one. |
| SSProve | Foundational modular cryptographic proofs | Probabilistic programs and package composition in Rocq | Connected to implementations by verified translations | Rocq kernel-checked proofs | A shared foundational semantics can close otherwise informal tool seams, but the user-facing language should hide prover-specific plumbing. |
| Lean 4 | Extensible theorem proving and programming with a small kernel | General dependent type theory | Efficient tooling/code generation, not a dedicated crypto backend | Kernel-checked proof terms and rich tactics | Strong option for a checker and metatheory; less direct reuse of the established verified-crypto compiler stack than Rocq today. |
| Rocq + CompCert ecosystem | Mechanized language semantics and verified compilation | General constructive type theory | Extraction and verified C compiler research/production paths | Small-kernel checked developments | Strongest current fit for Orange’s compiler metatheory and authoritative extracted checker, though user experience must not expose Rocq as the Orange language. |

### What the table does not imply

- A project with a verified compiler has not automatically proved an algorithm
  secure.
- A source-level constant-time checker has not automatically established a
  target-binary or microarchitectural claim.
- Passing NIST vectors establishes conformance on those tests, not functional
  correctness for all inputs or FIPS 140 module validation.
- Exporting to a proof assistant is not sufficient unless the translation and
  imported assumptions are visible.
- A small proof kernel does not make the parser, semantics, ISA model, assembler,
  FFI caller, or hardware disappear from every trusted base.

## 4. Findings that shape the design

### 4.1 The useful unit is a claim, not “verified software”

The tools address different properties because the properties are genuinely
different. Orange must model at least:

- standards and vector conformance;
- functional refinement or equivalence;
- termination, memory safety, initialization, and arithmetic safety;
- leakage noninterference under a named observation model;
- preservation through each compiler transformation;
- correspondence between final encoded object bytes and the machine IR;
- cryptographic security in a computational or other stated model;
- empirical evidence such as differential, fuzz, and timing tests.

A package can have some without the others. A report needs a matrix, not a
single level whose higher number accidentally implies unrelated properties.

### 4.2 Constant-time is target- and model-indexed

Jasmin describes the conventional control-flow and memory-address leakage model
as a noninterference property: two executions with equal public inputs must
produce equal leakage traces even if secret inputs differ. That is an essential
baseline, but not a universal side-channel theorem.

Compiler optimizations can alter branches and accesses. Research on a modified
CompCert shows that preserving cryptographic constant-time is a distinct
compiler proof, not a free consequence of ordinary semantic preservation.
Speculation, variable-latency instructions, caches beyond address traces,
power, electromagnetic emanations, faults, and operating-system effects need
separate models or explicit exclusions.

Orange should attach leakage claims to:

1. a semantic observation model;
2. a target ISA and feature set;
3. a compiler pipeline digest;
4. a hardware/platform assumption set; and
5. the exact entry points and secret/public partition.

### 4.3 The solver should search, not legislate

F\* demonstrates that SMT automation can remove a large burden from routine
refinement proofs, while its own documentation is candid that the F\* and Z3
combination is trusted for those results. Lean demonstrates the alternative of
checking explicit proof terms with a small kernel. Modern cvc5 and SAT solvers
can produce proof artifacts for useful fragments.

Orange should use a portfolio:

- kernel-checked Orange proof terms for general interactive arguments;
- LRAT-style certificates for bit-blasted SAT/equivalence obligations;
- checked Alethe or another ratified certificate format for supported SMT
  fragments;
- untrusted solvers for counterexample search and proof discovery;
- explicitly labeled external proof evidence for EasyCrypt/SSProve adapters.

Timeout, unknown, unsupported certificate production, and proof-check failure
must never become success.

### 4.4 A familiar surface is useful; semantic strata matter more

hacspec benefits from a Rust-shaped subset. Cryptol benefits from concise
sequence and width polymorphism. Jasmin benefits from making low-level choices
visible. Orange should borrow those usability lessons without pretending one
set of operational rules fits every layer.

Mathematical integers must not silently wrap. Secret values must not silently
control branches or addresses. Probabilistic sampling must not appear in a pure
deterministic specification. Vector intrinsics must name their target feature
and semantics. These are separate strata with checked crossings.

### 4.5 End-to-end proof still needs interoperability

Even a self-contained Orange toolchain must meet existing systems at its edges:

- standards text, errata, and official vectors;
- C ABI consumers and Rust packages;
- object formats, linkers, operating systems, and CPU feature discovery;
- NIST ACVP vector exchanges;
- SBOM/CBOM and supply-chain attestations;
- established proof libraries and independently checked exports.

Interop artifacts should be generated from the same definitions and included in
the claim graph. Hand-maintained duplicate headers, vector converters, or
assumption lists recreate the gap Orange is meant to close.

### 4.6 Standards are versioned inputs, not timeless citations

FIPS 203 and FIPS 204 already publish planning notes and errata after final
publication. ACVP’s supported algorithm and schema set evolves. An Orange spec
therefore needs machine-readable provenance containing at least the publication
identifier, edition/date, incorporated errata digest, clause anchors, vector-set
digests, and any deliberate deviation.

“Implements ML-KEM” without those fields is not an auditable conformance claim.

### 4.7 Validation is external

NIST’s CAVP/ACVP process is black-box algorithm testing, and NIST explicitly
distinguishes algorithm validation from FIPS 140 cryptographic-module
validation. Orange should generate and consume ACVP-compatible evidence and
make laboratory workflows easier. It must not suggest that formal proofs or
locally replayed ACVP vectors grant a government validation certificate.

### 4.8 The name has collision risk

“Orange” is already the name of a long-running data-mining/visual-programming
product, and an earlier `orange-lang/orange` repository describes a systems
programming language. The term is also heavily used commercially.

This is not a legal conclusion, but it is enough to require a pre-code naming
gate covering trademark review, command names, package namespaces, domains,
searchability, and migration cost. The repository may retain Orange as a
codename until that gate closes.

### 4.9 Additional constant-time and arithmetic tools confirm the layered model

The broader tool landscape reinforces that no one check covers the pipeline:

- `ct-verif` applies relational verification to optimized LLVM and is valuable
  for existing C-facing APIs, but the native-lowering and hardware model remain
  separate concerns.
- FaCT uses secrecy-aware source constructs and transformations to improve
  readable timing-sensitive programming, while ordinary backend and hardware
  assumptions still need accounting.
- CT-Wasm required a stricter typed WebAssembly dialect because ordinary Wasm
  safety is not a timing guarantee.
- `dudect` performs statistical timing tests on real targets. It can find leaks
  outside a formal model; a clean result cannot prove their absence.
- CryptoLine and CoqCryptoLine are effective at algebraic and range reasoning
  for modeled low-level arithmetic, especially hand-optimized and post-quantum
  kernels, but do not replace whole-API, compiler, leakage, or security-game
  proofs.

Orange should integrate equivalent evidence classes rather than presenting one
as a substitute for the others. Search-based arithmetic or machine-code
optimization is acceptable outside the TCB when every accepted result has a
checked equivalence and range certificate.

### 4.10 Primitive correctness is not protocol interoperability

A primitive standard defines mathematics and core algorithms; deployment
profiles separately define bytes, identifiers, negotiation, error handling, and
state. Current post-quantum work illustrates the difference:

- FIPS 203 defines ML-KEM.
- RFC 9935 specifies X.509 use and key encodings.
- RFC 9936 specifies CMS integration and warns about compatibility boundaries
  with pre-standard Kyber.
- Other protocol profiles may remain Internet-Drafts and must not be represented
  as finalized standards.

Orange therefore needs nominally distinct standard editions and protocol
profiles, plus separately proved serialization/adapter modules. A proof of the
ML-KEM arithmetic cannot silently imply correct OIDs, ASN.1/DER, TLS
negotiation, failure behavior, or host API layout.

## 5. Architecture alternatives considered

### A. Orchestrator over existing languages

Orange could be a manifest and CLI that connects Cryptol/hacspec, Jasmin,
EasyCrypt, and existing compilers.

Advantages: fastest access to mature capabilities; lower metatheory burden.

Disadvantages: the central product promise remains dependent on multiple
surface languages and partly trusted translations; diagnostics and versioning
are fragmented; the manifest risks becoming the only Orange language.

Decision: useful as import/export compatibility, not sufficient as the product.

### B. Embedded DSL inside F*, Lean, or Rocq

Advantages: reuse a proof kernel, tactics, package ecosystem, and meta-language;
semantic definitions are close to proofs.

Disadvantages: users inherit the host language and its upgrades; low-level
compiler and IDE experience are host-dependent; proof-engine choices leak into
ordinary Orange code; changing foundations becomes difficult.

Decision: use a proof assistant to mechanize and implement the authoritative
checker, but keep Orange standalone and its serialized core stable.

### C. Standalone language with an opaque SMT backend

Advantages: familiar compiler UX and strong automation.

Disadvantages: solver answers and encodings expand the trusted base; replay is
fragile across solver versions; game proofs and compiler preservation do not
fit a single SMT query.

Decision: reject opaque success. Accept only checked certificates for native
claims or disclose an external checker as part of the claim’s trusted base.

### D. Standalone language, proof-carrying core, verified compiler

Advantages: matches the product thesis; allows independent checking; makes
crossings and assumptions explicit; supports stable tooling around a small
normative core.

Disadvantages: largest engineering and formalization cost; requires deliberate
scope control and years of compiler/proof work.

Decision: recommended. The user explicitly chose the long route to the end
product rather than a disposable prototype, so the plan should budget this cost
instead of hiding it.

## 6. Recommended differentiation

Orange should compete on the following combined capabilities:

1. **One claim graph.** Standards source, spec, implementation, proof, compiler,
   target, object, ABI, vectors, and release provenance are nodes in one
   content-addressed graph.
2. **Assurance coverage, not branding.** Reports separate the claim outcome
   from its basis, such as a kernel proof, checked certificate, external proof,
   assumption, test run, audit, or validation.
3. **Standards plus errata provenance.** A reviewer can tell exactly which
   normative text and corrections were implemented.
4. **Target-indexed leakage.** Constant-time and future leakage models name the
   observation trace, compiler, target features, and platform assumptions.
5. **Certificate-first automation.** Solvers improve ergonomics without becoming
   invisible authorities.
6. **Checked last mile.** The supported path continues through optimized Machine
   IR and object encoding, instead of stopping at generated C.
7. **Generated interoperability.** C headers, Rust bindings, ACVP adapters,
   vectors, documentation, and CBOM entries derive from the same package.
8. **Offline replay.** A release proof bundle contains or content-addresses all
   required inputs under a documented archival policy.
9. **A flagship living corpus.** Real symmetric, curve, and post-quantum
   implementations drive language and compiler acceptance from the beginning.

## 7. Recommended technical foundation

Subject to the ratification gates in [DECISIONS.md](DECISIONS.md):

- Use Rust for the untrusted but production-quality driver, parser services,
  package tooling, diagnostics, LSP, and host integration.
- Use Rocq for the normative core semantics, metatheory, claim checker, and
  verified compiler transformations; extract the authoritative checker and
  verified passes for distribution.
- Define a canonical, deterministic, versioned Orange Core serialization so an
  independent checker never needs to trust transient compiler memory objects.
- Use proof-producing SAT/SMT only for formats Orange checks. Keep external
  EasyCrypt/SSProve bridges first-class and disclose their exact trusted base.
- Build Orange’s own formally specified Machine IR and native lowering for the
  1.0 targets. Use Jasmin, SAW, CompCert research variants, and mature crypto
  libraries as differential oracles and interoperability paths, not as an
  undocumented permanent backend.
- Emit a reference C representation for review and integration, explicitly
  without inheriting the full native assurance claim unless a separate checked
  path establishes it.

The choice of Rocq is pragmatic rather than ideological: the closest verified
crypto compiler and synthesis work already lives in that ecosystem. Orange’s
surface language and proof artifacts remain its own, so the foundation can be
reassessed before the core-format stability gate.

## 8. Source index

All links were checked during the 2026-07-11 research pass.

### Languages and verification systems

- [Cryptol documentation](https://tools.galois.com/cryptol/get-started/documentation)
- [SAW overview](https://tools.galois.com/saw)
- [SAW tutorial](https://saw.galois.com/intro/)
- [hacspec](https://hacspec.org/)
- [hax repository and backend status](https://github.com/cryspen/hax)
- [F\* proof-oriented programming introduction](https://fstar-lang.org/tutorial/book/intro.html)
- [HACL\* and EverCrypt manual](https://hacl-star.github.io/)
- [Vale repository](https://github.com/project-everest/vale)
- [Jasmin documentation](https://jasmin-lang.readthedocs.io/en/stable/)
- [Jasmin constant-time methodology](https://jasmin-lang.readthedocs.io/en/stable/tools/ct.html)
- [Jasmin compiler passes](https://jasmin-lang.readthedocs.io/en/stable/compiler/passes/index.html)
- [EasyCrypt](https://easycrypt.gitlab.io/easycrypt-web/)
- [Fiat-Crypto](https://github.com/mit-plv/fiat-crypto)
- [Lean language reference](https://lean-lang.org/doc/reference/latest/)
- [Rocq extraction documentation](https://docs.rocq-prover.org/master/refman/addendum/extraction.html)
- [CompCert documentation](https://compcert.org/doc/)

### End-to-end and compiler research

- [The Last Yard, CPP 2024](https://popl24.sigplan.org/details/CPP-2024-papers/2/The-Last-Yard-Foundational-End-to-End-Verification-of-High-Speed-Cryptography)
- [Last Yard preprint](https://eprint.iacr.org/2023/185)
- [The Last Mile](https://eprint.iacr.org/2019/160)
- [EverCrypt paper](https://eprint.iacr.org/2019/757)
- [Constant-time-preserving C compiler](https://eprint.iacr.org/2019/926)
- [SoK: Computer-Aided Cryptography](https://eprint.iacr.org/2019/1393)
- [ct-verif](https://www.usenix.org/conference/usenixsecurity16/technical-sessions/presentation/almeida)
- [FaCT](https://pldi19.sigplan.org/details/pldi-2019-papers/47/FaCT-A-DSL-for-Timing-Sensitive-Computation)
- [CT-Wasm](https://popl19.sigplan.org/details/POPL-2019-Research-Papers/59/CT-Wasm-Type-Driven-Secure-Cryptography-for-the-Web-Ecosystem)
- [dudect](https://github.com/oreparaz/dudect)
- [CoqCryptoLine](https://link.springer.com/chapter/10.1007/978-3-031-37703-7_11)

### Standards and validation

- [NIST CAVP](https://csrc.nist.gov/projects/cryptographic-algorithm-validation-program)
- [NIST ACVP documentation](https://pages.nist.gov/ACVP/)
- [FIPS 203, ML-KEM](https://csrc.nist.gov/pubs/fips/203/final)
- [FIPS 204, ML-DSA](https://csrc.nist.gov/pubs/fips/204/final)
- [FIPS 205, SLH-DSA](https://csrc.nist.gov/pubs/fips/205/final)
- [NIST automated module validation work](https://pages.nist.gov/ACMVPDocs/)
- [RFC 9935, ML-KEM in X.509](https://www.rfc-editor.org/rfc/rfc9935.html)
- [RFC 9936, ML-KEM in CMS](https://www.rfc-editor.org/rfc/rfc9936.html)
- [RFC 9180, HPKE and detailed vectors](https://www.rfc-editor.org/rfc/rfc9180.html)
- [Project Wycheproof](https://github.com/C2SP/wycheproof)

### Release and supply-chain evidence

- [NIST SSDF 1.1](https://csrc.nist.gov/pubs/sp/800/218/final)
- [SLSA 1.2](https://slsa.dev/spec/v1.2/)
- [OpenSSF OSPS Baseline](https://baseline.openssf.org/)
- [Reproducible Builds definition](https://reproducible-builds.org/docs/definition/)
- [`SOURCE_DATE_EPOCH` specification](https://reproducible-builds.org/specs/source-date-epoch/)
- [Sigstore signing model](https://docs.sigstore.dev/cosign/signing/overview/)
- [CycloneDX CBOM](https://cyclonedx.org/capabilities/cbom/)

### Naming evidence

- [Orange data-mining project](https://orangedatamining.com/)
- [Earlier Orange systems language](https://github.com/orange-lang/orange)

## 9. Research to refresh at later gates

- Confirm the current proof-output coverage and checker maturity of cvc5/Alethe
  for Orange’s actual obligation fragments.
- Re-evaluate Lean, Rocq, F*, and emerging verified Rust compilers using a
  written, reproducible decision suite before freezing the proof foundation.
- Confirm current Jasmin architectures and formal statements before using it as
  a differential oracle.
- Track speculative constant-time, hardware leakage, and verified object-format
  research before freezing the native leakage claim.
- Re-read every normative cryptographic standard and errata sheet when importing
  it; this research index is not a substitute for provenance in the package.
- Perform a professional naming/trademark and license review before public
  branding or package publication.
