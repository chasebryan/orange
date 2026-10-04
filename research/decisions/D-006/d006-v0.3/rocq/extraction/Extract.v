(** D-006 v0.3, candidate C-01 (Rocq): DS-05, extraction of the standalone
    checker to OCaml.

    Primitive strings and 63-bit integers use the mappings Rocq ships,
    [ExtrOCamlPString] and [ExtrOCamlInt63] (which also map [bool] and
    [prod] to OCaml's own). They name the kernel's [Pstring] and [Uint63]
    modules: the build compiles the kernel's own [pstring.ml] from the
    toolchain and [uint63.ml] here, the subset of the kernel's module that
    the code uses. No other [Extract] directive is used, so binary
    naturals, integers, lists and maps keep their Rocq definitions. *)

From D006 Require Lrat.
From D006.Extraction Require Checker.
From Stdlib Require Extraction ExtrOCamlPString.

(** Run from the source root; writes [extraction/extracted.ml] and its
    interface. *)
Set Extraction Output Directory "extraction".

Extraction "extracted.ml"
  Checker.records_line Lrat.Obligation Lrat.cnf_text Checker.lrat_line.
