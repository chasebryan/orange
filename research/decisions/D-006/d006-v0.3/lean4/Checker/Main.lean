import D006.Records
import D006.Lrat

/-!
# D-006 v0.3, candidate C-02 (Lean 4): DS-05, the standalone checker's driver

The standalone checker is the Lean compiler's C translation of the in-prover
definitions `D006.Records.decode_records` (DS-03), `D006.Lrat.cnf_text` and
`D006.Lrat.lrat_verdict` (DS-04), linked with the Lean runtime and the
compiled core library. Nothing here re-implements them: this module is the
hand-written driver (trusted glue). It reads the command line and the input
files, calls those three definitions, prints the verdict line and sets the
exit code.

```
d006-checker records PATH
d006-checker cnf OBLIGATION
d006-checker lrat OBLIGATION CNF_PATH CERTIFICATE_PATH
```

`OBLIGATION` is `B-C01` or `B-C02`. On success the checker prints one line
(the CNF text for `cnf`) on standard output and exits 0. Any other command
line prints the usage on standard error and exits 2. An input file that
cannot be read (or, for `lrat`, is not UTF-8 text) prints `error: ...` on
standard error and exits 1.
-/

namespace D006.Checker

open D006.Records D006.Lrat

/-! ## The DS-03 verdict line (semantics.md, section 4) -/

/-- A failure code as the shared packet spells it. -/
def codeText : ErrorCode → String
  | .oversized => "oversized"
  | .bad_magic => "bad_magic"
  | .unknown_version => "unknown_version"
  | .truncated => "truncated"
  | .malformed_number => "malformed_number"
  | .noncanonical_number => "noncanonical_number"
  | .unknown_field => "unknown_field"
  | .invalid_name => "invalid_name"
  | .invalid_utf8 => "invalid_utf8"
  | .duplicate_name => "duplicate_name"
  | .noncanonical_order => "noncanonical_order"
  | .reference_escape => "reference_escape"
  | .cyclic_reference => "cyclic_reference"
  | .reference_kind => "reference_kind"
  | .invalid_level => "invalid_level"
  | .trailing_data => "trailing_data"

/-- Bytes in lowercase hexadecimal, two digits each. -/
def hex (bytes : List (Word 8)) : String :=
  String.join (bytes.map fun b =>
    String.ofList [Nat.digitChar (b.toNat / 16), Nat.digitChar (b.toNat % 16)])

/-- One accepted record: `def:NAME:DIGEST`, `thm:NAME:FINGERPRINT:REFS` or
`claim:NAME:REF:LEVEL`, references as comma-separated decimals. -/
def recordField : Record → String
  | .rdef name digest => s!"def:{hex name}:{hex digest}"
  | .rthm name fingerprint refs =>
    s!"thm:{hex name}:{hex fingerprint}:{",".intercalate (refs.map toString)}"
  | .rclaim name ref level => s!"claim:{hex name}:{ref}:{level}"

/-- The verdict line of a decoded input: `accept` followed by one field per
record, or `reject CODE PATH`. -/
def recordsLine : Result (List Record) Failure → String
  | .ok records => " ".intercalate ("accept" :: records.map recordField)
  | .err (.failure code path) => s!"reject {codeText code} {path}"

/-! ## The DS-04 verdict line -/

/-- `accept`, or `reject CODE LINE`. -/
def verdictLine : Verdict → String
  | .accept => "accept"
  | .reject code line => s!"reject {code} {line}"

/-- The obligation with shared id `B-C01` or `B-C02`. -/
def obligation? : String → Option Obligation
  | "B-C01" => some .carry_save
  | "B-C02" => some .carry_save_unshifted
  | _ => none

/-! ## The command line -/

/-- Printed on standard error for any other command line. -/
def usage : String :=
  "usage: d006-checker records PATH\n" ++
  "       d006-checker cnf B-C01|B-C02\n" ++
  "       d006-checker lrat B-C01|B-C02 CNF_PATH CERTIFICATE_PATH"

/-- A file's bytes as DS-03 input. -/
def readBytes (path : String) : IO (List (Word 8)) := do
  return (← IO.FS.readBinFile path).data.toList.map UInt8.toBitVec

/-- The text a command prints, or `none` if the arguments are not one of the
three commands. -/
def command : List String → Option (IO String)
  | ["records", path] => some do
    return recordsLine (decode_records (← readBytes path)) ++ "\n"
  | ["cnf", name] => (obligation? name).map fun o => pure (cnf_text o)
  | ["lrat", name, cnf, certificate] => (obligation? name).map fun o => do
    return verdictLine (lrat_verdict o (← IO.FS.readFile cnf) (← IO.FS.readFile certificate)) ++ "\n"
  | _ => none

end D006.Checker

open D006.Checker in
/-- Runs one command: exit 0 with its output, 2 with the usage, 1 if a file
cannot be read. -/
def main (args : List String) : IO UInt32 := do
  match command args with
  | none =>
    IO.eprintln usage
    return 2
  | some run =>
    try
      IO.print (← run)
      return 0
    catch e =>
      IO.eprintln s!"error: {e}"
      return 1
