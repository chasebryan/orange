import D006.Core

/-!
# D-006 v0.3, candidate C-02 (Lean 4): DS-03 canonical records (OCR1)

Every shared R- symbol of DS-03 maps to one declaration here (see
`adapter.d/ds03.json`). Bytes are `List (Word 8)` from `D006.Core`, the
failure path is a `String`, and every function is structurally recursive,
so a closed observation evaluates in the kernel under `decide +kernel`.

The decoder is a small parser over the remaining bytes. `records_valid` is
written from the validity conditions of semantics.md, section 4, and does not
call the decoder; the theorems at the end connect the two through the
encoder.
-/

namespace D006.Records

/-! ## Records and failures (R-T01, R-T02, R-T03) -/

/-- R-T01: a definition (tag 1), a theorem (tag 2) or a claim (tag 3). -/
inductive Record : Type where
  /-- R-C17: a definition with its digest. -/
  | rdef (name : List (Word 8)) (digest : List (Word 8))
  /-- R-C18: a theorem with its fingerprint and references to earlier records. -/
  | rthm (name : List (Word 8)) (fingerprint : List (Word 8)) (refs : List Nat)
  /-- R-C19: a claim on an earlier theorem, with a level. -/
  | rclaim (name : List (Word 8)) (ref : Nat) (level : Nat)
  deriving DecidableEq, Repr

/-- R-T02: the decoder's failure codes (R-C01 to R-C16). -/
inductive ErrorCode : Type where
  | oversized
  | bad_magic
  | unknown_version
  | truncated
  | malformed_number
  | noncanonical_number
  | unknown_field
  | invalid_name
  | invalid_utf8
  | duplicate_name
  | noncanonical_order
  | reference_escape
  | cyclic_reference
  | reference_kind
  | invalid_level
  | trailing_data
  deriving DecidableEq, Repr

/-- R-T03: the first failure, as a code and a path such as `records/1/name`. -/
inductive Failure : Type where
  /-- R-C20. -/
  | failure (code : ErrorCode) (path : String)
  deriving DecidableEq, Repr

/-- A record's name. -/
def Record.name : Record → List (Word 8)
  | .rdef name _ | .rthm name _ _ | .rclaim name _ _ => name

/-- A record's tag byte. -/
def Record.tag : Record → Nat
  | .rdef .. => 1
  | .rthm .. => 2
  | .rclaim .. => 3

/-! ## Well-formed UTF-8 (R-F04) -/

/-- The ranges the continuation bytes after lead byte `lead` must fall in,
following the table of RFC 3629, section 4 (`none`: not a lead byte). -/
def utf8_ranges (lead : Nat) : Option (List (Nat × Nat)) :=
  let tail := (0x80, 0xBF)
  if lead ≤ 0x7F then some []
  else if 0xC2 ≤ lead ∧ lead ≤ 0xDF then some [tail]
  else if lead = 0xE0 then some [(0xA0, 0xBF), tail]
  else if 0xE1 ≤ lead ∧ lead ≤ 0xEC then some [tail, tail]
  else if lead = 0xED then some [(0x80, 0x9F), tail]
  else if 0xEE ≤ lead ∧ lead ≤ 0xEF then some [tail, tail]
  else if lead = 0xF0 then some [(0x90, 0xBF), tail, tail]
  else if 0xF1 ≤ lead ∧ lead ≤ 0xF3 then some [tail, tail, tail]
  else if lead = 0xF4 then some [(0x80, 0x8F), tail, tail]
  else none

/-- Scans the bytes left to right; `pending` holds the ranges of the
continuation bytes still owed by the current character. -/
def utf8_scan : List (Nat × Nat) → List (Word 8) → Bool
  | [], [] => true
  | _ :: _, [] => false
  | (lo, hi) :: pending, b :: bs => lo ≤ b.toNat && b.toNat ≤ hi && utf8_scan pending bs
  | [], b :: bs =>
    match utf8_ranges b.toNat with
    | some pending => utf8_scan pending bs
    | none => false

/-- R-F04: the bytes are well-formed UTF-8 (no overlong forms, no surrogates,
nothing above U+10FFFF). -/
def utf8_valid (bytes : List (Word 8)) : Bool := utf8_scan [] bytes

/-! ## The encoder (R-F02) -/

/-- A number below `2 ^ 14` in one byte (below `0x80`) or two:
`0x80 + n % 128`, then `n / 128`. A valid record list only has numbers up to
200 (name lengths), so larger ones never matter. -/
def uvar_encode (n : Nat) : List (Word 8) :=
  if n < 0x80 then [word_of_nat 8 n]
  else [word_of_nat 8 (0x80 + n % 128), word_of_nat 8 (n / 128)]

/-- The fields after a record's name. -/
def encode_fields : Record → List (Word 8)
  | .rdef _ digest => digest
  | .rthm _ fingerprint refs =>
    fingerprint ++ uvar_encode refs.length ++ refs.flatMap uvar_encode
  | .rclaim _ ref level => uvar_encode ref ++ [word_of_nat 8 level]

/-- One record: the tag byte, the name's length and bytes, then its fields. -/
def encode_record (r : Record) : List (Word 8) :=
  word_of_nat 8 r.tag :: (uvar_encode r.name.length ++ r.name ++ encode_fields r)

/-- The magic `OCR` and the version byte `1`. -/
def header : List (Word 8) := [0x4F, 0x43, 0x52, 0x01]

/-- R-F02: the header, the record count, then each record. -/
def encode_records (l : List Record) : List (Word 8) :=
  header ++ uvar_encode l.length ++ l.flatMap encode_record

/-! ## Validity (R-F03), from the stated conditions -/

/-- `a` sorts strictly before `b` bytewise; a proper prefix sorts first. -/
def bytes_lt : List (Word 8) → List (Word 8) → Bool
  | [], [] => false
  | [], _ :: _ => true
  | _ :: _, [] => false
  | a :: as, b :: bs => a.toNat < b.toNat || (a.toNat = b.toNat && bytes_lt as bs)

/-- The list strictly increases. -/
def increasing : List Nat → Bool
  | a :: b :: rest => a < b && increasing (b :: rest)
  | _ => true

/-- A name is 1 to 200 bytes of well-formed UTF-8. -/
def name_valid (name : List (Word 8)) : Bool :=
  1 ≤ name.length && name.length ≤ 200 && utf8_valid name

/-- The name sorts strictly after the previous record's name, if any. -/
def sorts_after : Option (List (Word 8)) → List (Word 8) → Bool
  | none, _ => true
  | some previous, name => bytes_lt previous name

/-- Record `i` of `before` is a theorem. -/
def is_theorem (before : List Record) (i : Nat) : Bool :=
  match before[i]? with
  | some (.rthm ..) => true
  | _ => false

/-- The conditions on a record's fields, given the records before it (so
`before.length` is its own index). -/
def fields_ok (before : List Record) : Record → Bool
  | .rdef _ digest => digest.length = 32
  | .rthm _ fingerprint refs =>
    fingerprint.length = 32 && refs.length ≤ 64 && increasing refs &&
      refs.all (· < before.length)
  | .rclaim _ ref level => ref < before.length && is_theorem before ref && level ≤ 3

/-- The conditions on one record, given the records before it: a valid name
that sorts after the previous record's name (so the names strictly
increase), and valid fields. -/
def record_ok (before : List Record) (r : Record) : Bool :=
  name_valid r.name && sorts_after (before.getLast?.map Record.name) r.name && fields_ok before r

/-- Every record satisfies `record_ok` given the records before it. -/
def records_ok_after (before : List Record) : List Record → Bool
  | [] => true
  | r :: rest => record_ok before r && records_ok_after (before ++ [r]) rest

/-- R-F03: at most 64 records, every name 1 to 200 bytes of well-formed
UTF-8, names strictly increasing, 32-byte digests and fingerprints, at most 64
strictly increasing references per theorem each below its own index, every
claim citing an earlier theorem at a level of at most 3, and an encoding of
at most 4096 bytes. -/
def records_valid (l : List Record) : Bool :=
  l.length ≤ 64 && records_ok_after [] l && (encode_records l).length ≤ 4096

/-! ## The decoder (R-F01) -/

/-- A decoding step: reads a prefix of the bytes and returns a value and the
rest, or fails. -/
structure Parser (α : Type) : Type where
  /-- Runs the step on the remaining bytes. -/
  run : List (Word 8) → Result (α × List (Word 8)) Failure

namespace Parser

/-- Returns `a` and reads nothing. -/
def pure {α : Type} (a : α) : Parser α := ⟨fun bs => .ok (a, bs)⟩

/-- Runs `p`, then `f` on its value and the rest; the first failure stops. -/
def bind {α β : Type} (p : Parser α) (f : α → Parser β) : Parser β :=
  ⟨fun bs => match p.run bs with
    | .ok (a, rest) => (f a).run rest
    | .err e => .err e⟩

instance : Monad Parser where
  pure := Parser.pure
  bind := Parser.bind

end Parser

/-- Fails with `code` at `path` unless `ok` holds. -/
def require (ok : Bool) (code : ErrorCode) (path : String) : Parser Unit :=
  ⟨fun bs => if ok then .ok ((), bs) else .err (.failure code path)⟩

/-- Reads one byte; `truncated` at `path` if none is left. -/
def readByte (path : String) : Parser (Word 8) :=
  ⟨fun
    | b :: rest => .ok (b, rest)
    | [] => .err (.failure .truncated path)⟩

/-- Reads `n` bytes; `truncated` at `path` if fewer are left. -/
def readBytes (n : Nat) (path : String) : Parser (List (Word 8)) :=
  ⟨fun bs => if n ≤ bs.length then .ok (bs.take n, bs.drop n) else .err (.failure .truncated path)⟩

/-- Reads a one- or two-byte number. A second byte of `0x80` or more is
malformed; a second byte of zero is noncanonical. -/
def readNumber (path : String) : Parser Nat := do
  let b0 ← readByte path
  if b0.toNat < 0x80 then pure b0.toNat
  else do
    let b1 ← readByte path
    require (b1.toNat < 0x80) .malformed_number path
    require (b1.toNat ≠ 0) .noncanonical_number path
    pure (b0.toNat - 0x80 + 128 * b1.toNat)

/-- Reads one byte of the magic: `truncated` or `bad_magic` at `header`. -/
def readMagic (expected : Nat) : Parser Unit :=
  ⟨fun
    | b :: rest =>
      if b.toNat = expected then .ok ((), rest) else .err (.failure .bad_magic "header")
    | [] => .err (.failure .truncated "header")⟩

/-- Reads a name (its length, then its bytes) and checks it, in this order:
length 1 to 200, well-formed UTF-8, different from the previous record's name,
and not sorting before it. -/
def readName (path : String) (previous : Option (List (Word 8))) : Parser (List (Word 8)) := do
  let length ← readNumber path
  require (length ≠ 0 && length ≤ 200) .invalid_name path
  let name ← readBytes length path
  require (utf8_valid name) .invalid_utf8 path
  match previous with
  | none => pure name
  | some p => do
    require (name ≠ p) .duplicate_name path
    require (!bytes_lt name p) .noncanonical_order path
    pure name

/-- Checks a reference of the record at `index` in a set of `count` records. -/
def checkRef (count index ref : Nat) (path : String) : Parser Unit := do
  require (ref < count) .reference_escape path
  require (ref < index) .cyclic_reference path

/-- `r` is above the previous reference, if any. -/
def above : Option Nat → Nat → Bool
  | none, _ => true
  | some p, r => p < r

/-- Reads `n` more references of the theorem at `index`, starting at slot
`slot`; `previous` is the reference in the slot before. -/
def readRefs (count index : Nat) (base : String) (slot : Nat) (previous : Option Nat) :
    Nat → Parser (List Nat)
  | 0 => pure []
  | n + 1 => do
    let path := base ++ "/refs/" ++ toString slot
    let ref ← readNumber path
    checkRef count index ref path
    require (above previous ref) .noncanonical_order path
    let refs ← readRefs count index base (slot + 1) (some ref) n
    pure (ref :: refs)

/-- Reads the fields of a record with tag `tag` and name `name`; `before` are
the records read so far. `readRecord` has already refused other tags, so the
last case is never reached from there. -/
def readFields (count : Nat) (before : List Record) (base : String) (name : List (Word 8)) :
    Nat → Parser Record
  | 1 => do
    let digest ← readBytes 32 (base ++ "/digest")
    pure (.rdef name digest)
  | 2 => do
    let fingerprint ← readBytes 32 (base ++ "/fingerprint")
    let total ← readNumber (base ++ "/refs")
    require (total ≤ 64) .oversized (base ++ "/refs")
    let refs ← readRefs count before.length base 0 none total
    pure (.rthm name fingerprint refs)
  | 3 => do
    let ref ← readNumber (base ++ "/ref")
    checkRef count before.length ref (base ++ "/ref")
    require (is_theorem before ref) .reference_kind (base ++ "/ref")
    let level ← readByte (base ++ "/level")
    require (level.toNat ≤ 3) .invalid_level (base ++ "/level")
    pure (.rclaim name ref level.toNat)
  | _ => ⟨fun _ => .err (.failure .unknown_field (base ++ "/tag"))⟩

/-- Reads record `before.length` of `count`. -/
def readRecord (count : Nat) (before : List Record) : Parser Record := do
  let base := "records/" ++ toString before.length
  let tag ← readByte (base ++ "/tag")
  require (1 ≤ tag.toNat && tag.toNat ≤ 3) .unknown_field (base ++ "/tag")
  let name ← readName (base ++ "/name") (before.getLast?.map Record.name)
  readFields count before base name tag.toNat

/-- Reads `n` more records of `count`; `before` are the records read so far. -/
def readRecords (count : Nat) (before : List Record) : Nat → Parser (List Record)
  | 0 => pure []
  | n + 1 => do
    let r ← readRecord count before
    let rest ← readRecords count (before ++ [r]) n
    pure (r :: rest)

/-- Reads the header, the record count (at most 64) and the records. -/
def readFile : Parser (List Record) := do
  readMagic 0x4F
  readMagic 0x43
  readMagic 0x52
  let version ← readByte "header"
  require (version.toNat = 1) .unknown_version "header"
  let count ← readNumber "count"
  require (count ≤ 64) .oversized "count"
  readRecords count [] count

/-- R-F01: refuses inputs over 4096 bytes, reads the file, then refuses
trailing bytes. -/
def decode_records (b : List (Word 8)) : Result (List Record) Failure :=
  if 4096 < b.length then .err (.failure .oversized "input")
  else
    match readFile.run b with
    | .ok (l, []) => .ok l
    | .ok (_, _ :: _) => .err (.failure .trailing_data "trailing")
    | .err e => .err e

/-! ## Proofs

The decoder and the encoder are connected step by step. For each reading
step there are two lemmas:

* `*_ok` (backwards): if the step succeeds, the bytes it read are exactly the
  encoding of the value it returned, and the value meets the conditions the
  step checks;
* `*_encode` or `*_bind` (forwards): on the encoding of a value that meets
  those conditions, the step succeeds with that value and the rest.

D3-TH02 and D3-TH03 come from the backwards lemma for the whole input
(`readFile_ok`), D3-TH01 from the forwards one (`readFile_encode`). -/

/-! ### Parser steps -/

namespace Parser

theorem bind_run {α β : Type} (p : Parser α) (f : α → Parser β) (bs : List (Word 8)) :
    (p >>= f).run bs =
      (match p.run bs with
        | .ok (a, rest) => (f a).run rest
        | .err e => .err e) := rfl

/-- A sequence succeeds only if its first step does. -/
theorem bind_eq_ok {α β : Type} {p : Parser α} {f : α → Parser β} {bs : List (Word 8)} {b : β}
    {rest : List (Word 8)} (h : (p >>= f).run bs = .ok (b, rest)) :
    ∃ a mid, p.run bs = .ok (a, mid) ∧ (f a).run mid = .ok (b, rest) := by
  rw [bind_run] at h
  split at h
  · exact ⟨_, _, by assumption, h⟩
  · cases h

theorem bind_of_ok {α β : Type} {p : Parser α} {f : α → Parser β} {bs : List (Word 8)} {a : α}
    {mid : List (Word 8)} (h : p.run bs = .ok (a, mid)) : (p >>= f).run bs = (f a).run mid := by
  rw [bind_run, h]

theorem pure_eq_ok {α : Type} {a b : α} {bs rest : List (Word 8)}
    (h : (pure a : Parser α).run bs = .ok (b, rest)) : a = b ∧ bs = rest := by
  cases h; exact ⟨rfl, rfl⟩

end Parser

open Parser

theorem require_ok {ok : Bool} {code : ErrorCode} {path : String} {bs rest : List (Word 8)}
    {u : Unit} (h : (require ok code path).run bs = .ok (u, rest)) : ok = true ∧ rest = bs := by
  unfold require at h
  split at h
  · cases h; exact ⟨by assumption, rfl⟩
  · cases h

/-- Passing a check: the rest of the step runs on the same bytes. -/
theorem require_bind_ok {β : Type} {ok : Bool} {code : ErrorCode} {path : String}
    {f : Unit → Parser β} {bs rest : List (Word 8)} {b : β}
    (h : (require ok code path >>= f).run bs = .ok (b, rest)) :
    ok = true ∧ (f ()).run bs = .ok (b, rest) := by
  obtain ⟨u, mid, h1, h2⟩ := bind_eq_ok h
  obtain ⟨hok, rfl⟩ := require_ok h1
  exact ⟨hok, h2⟩

/-- A check that holds reads nothing. -/
theorem require_bind {β : Type} {ok : Bool} {code : ErrorCode} {path : String} {f : Unit → Parser β}
    (h : ok = true) (bs : List (Word 8)) : (require ok code path >>= f).run bs = (f ()).run bs := by
  subst h; rfl

theorem readByte_ok {path : String} {bs rest : List (Word 8)} {b : Word 8}
    (h : (readByte path).run bs = .ok (b, rest)) : bs = b :: rest := by
  match bs, h with
  | _ :: _, rfl => rfl

theorem readByte_bind {β : Type} {path : String} {f : Word 8 → Parser β} (b : Word 8)
    (bs : List (Word 8)) : (readByte path >>= f).run (b :: bs) = (f b).run bs := rfl

/-! ### Bytes and numbers -/

theorem toNat_word_of_nat {n : Nat} (h : n < 256) : (word_of_nat 8 n).toNat = n := by
  simp [word_of_nat]; omega

theorem word_of_nat_toNat (b : Word 8) : word_of_nat 8 b.toNat = b := by
  apply BitVec.eq_of_toNat_eq
  have := b.isLt
  simp [word_of_nat]

/-- A number that was read is canonically encoded: a two-byte number is at
least `0x80`, and its bytes are `0x80 + n % 128` and `n / 128`. -/
theorem readNumber_ok {path : String} {bs rest : List (Word 8)} {n : Nat}
    (h : (readNumber path).run bs = .ok (n, rest)) : bs = uvar_encode n ++ rest := by
  obtain ⟨b0, mid, h0, h⟩ := bind_eq_ok h
  rw [readByte_ok h0]
  split at h
  · obtain ⟨rfl, rfl⟩ := pure_eq_ok h
    simp [uvar_encode, *, word_of_nat_toNat]
  · obtain ⟨b1, mid2, h1, h⟩ := bind_eq_ok h
    rw [readByte_ok h1]
    obtain ⟨c2, h⟩ := require_bind_ok h
    obtain ⟨c3, h⟩ := require_bind_ok h
    obtain ⟨rfl, rfl⟩ := pure_eq_ok h
    have := b0.isLt; have := b1.isLt
    simp at c2 c3
    have hn : ¬ (b0.toNat - 128 + 128 * b1.toNat < 128) := by omega
    simp only [uvar_encode, hn, ite_false, List.cons_append, List.nil_append, List.cons.injEq,
      and_true]
    constructor <;> apply BitVec.eq_of_toNat_eq <;> rw [toNat_word_of_nat (by omega)] <;> omega

theorem readNumber_encode (path : String) {n : Nat} (hn : n < 16384) (rest : List (Word 8)) :
    (readNumber path).run (uvar_encode n ++ rest) = .ok (n, rest) := by
  unfold uvar_encode readNumber
  split
  · rw [List.singleton_append, readByte_bind, toNat_word_of_nat (by omega),
      ite_eq_left (by assumption)]
    rfl
  · rw [List.cons_append, readByte_bind, toNat_word_of_nat (by omega), ite_eq_right (by omega),
      List.singleton_append, readByte_bind, toNat_word_of_nat (by omega),
      require_bind (by simp; omega), require_bind (by simp; omega)]
    show Result.ok (128 + n % 128 - 128 + 128 * (n / 128), rest) = _
    congr 3; omega

theorem readBytes_ok {n : Nat} {path : String} {bs rest x : List (Word 8)}
    (h : (readBytes n path).run bs = .ok (x, rest)) : bs = x ++ rest ∧ x.length = n := by
  dsimp only [readBytes] at h
  split at h
  · cases h
    exact ⟨(List.take_append_drop n bs).symm, by simp; omega⟩
  · cases h

theorem readBytes_bind {β : Type} {path : String} {f : List (Word 8) → Parser β}
    (x rest : List (Word 8)) :
    (readBytes x.length path >>= f).run (x ++ rest) = (f x).run rest := by
  rw [bind_of_ok]
  unfold readBytes
  simp

theorem readMagic_bind_ok {β : Type} {e : Nat} {f : Unit → Parser β} {bs rest : List (Word 8)}
    {b : β} (h : (readMagic e >>= f).run bs = .ok (b, rest)) :
    ∃ x mid, x.toNat = e ∧ bs = x :: mid ∧ (f ()).run mid = .ok (b, rest) := by
  obtain ⟨u, mid, h1, h2⟩ := bind_eq_ok h
  match bs, h1 with
  | x :: bs, h1 =>
    simp only [readMagic] at h1
    split at h1
    · cases h1; exact ⟨x, mid, by assumption, rfl, h2⟩
    · cases h1

theorem readMagic_bind {β : Type} {e : Nat} {f : Unit → Parser β} {b : Word 8} {bs : List (Word 8)}
    (h : b.toNat = e) : (readMagic e >>= f).run (b :: bs) = (f ()).run bs := by
  rw [bind_of_ok]
  simp [readMagic, h]

/-! ### Bytewise order: strict and total -/

theorem bytes_lt_irrefl (a : List (Word 8)) : bytes_lt a a = false := by
  induction a with
  | nil => rfl
  | cons x a ih => simp [bytes_lt, ih]

theorem bytes_lt_asymm {a b : List (Word 8)} (h : bytes_lt a b = true) : bytes_lt b a = false := by
  induction a generalizing b with
  | nil => cases b <;> first | rfl | cases h
  | cons x a ih =>
    cases b with
    | nil => cases h
    | cons y b =>
      simp only [bytes_lt, Bool.or_eq_true, decide_eq_true_eq, Bool.and_eq_true] at h
      simp only [bytes_lt, Bool.or_eq_false_iff, decide_eq_false_iff_not, Bool.and_eq_false_iff]
      rcases h with h | ⟨h1, h2⟩
      · exact ⟨by omega, Or.inl (by omega)⟩
      · exact ⟨by omega, Or.inr (ih h2)⟩

theorem bytes_lt_total {a b : List (Word 8)} (hne : a ≠ b) (h : bytes_lt a b = false) :
    bytes_lt b a = true := by
  induction a generalizing b with
  | nil => cases b with
    | nil => exact absurd rfl hne
    | cons => cases h
  | cons x a ih =>
    cases b with
    | nil => rfl
    | cons y b =>
      simp only [bytes_lt, Bool.or_eq_false_iff, decide_eq_false_iff_not,
        Bool.and_eq_false_iff] at h
      simp only [bytes_lt, Bool.or_eq_true, decide_eq_true_eq, Bool.and_eq_true]
      by_cases hxy : x.toNat = y.toNat
      · have hx : x = y := BitVec.eq_of_toNat_eq hxy
        subst hx
        have hab : a ≠ b := fun e => hne (by rw [e])
        rcases h with ⟨_, h | h⟩
        · exact absurd rfl h
        · exact Or.inr ⟨rfl, ih hab h⟩
      · exact Or.inl (by omega)

/-! ### Names -/

/-- A name that was read is valid and sorts after the previous one. -/
theorem readName_ok {path : String} {previous : Option (List (Word 8))}
    {bs rest name : List (Word 8)} (h : (readName path previous).run bs = .ok (name, rest)) :
    bs = uvar_encode name.length ++ name ++ rest ∧ name_valid name = true ∧
      sorts_after previous name = true := by
  obtain ⟨length, mid, h0, h⟩ := bind_eq_ok h
  obtain ⟨c1, h⟩ := require_bind_ok h
  obtain ⟨x, mid2, h2, h⟩ := bind_eq_ok h
  obtain ⟨hx, hlen⟩ := readBytes_ok h2
  obtain ⟨c2, h⟩ := require_bind_ok h
  have hv : name_valid x = true := by
    simp only [Bool.and_eq_true, decide_eq_true_eq] at c1
    simp only [name_valid, hlen, c2, Bool.and_eq_true, decide_eq_true_eq, and_true]
    exact ⟨by omega, c1.2⟩
  have hs : sorts_after previous x = true ∧ x = name ∧ mid2 = rest := by
    cases previous with
    | none => obtain ⟨rfl, rfl⟩ := pure_eq_ok h; exact ⟨rfl, rfl, rfl⟩
    | some p =>
      simp only at h
      obtain ⟨c3, h⟩ := require_bind_ok h
      obtain ⟨c4, h⟩ := require_bind_ok h
      obtain ⟨rfl, rfl⟩ := pure_eq_ok h
      simp only [decide_eq_true_eq, Bool.not_eq_true'] at c3 c4
      exact ⟨bytes_lt_total c3 c4, rfl, rfl⟩
  obtain ⟨hs, rfl, rfl⟩ := hs
  refine ⟨?_, hv, hs⟩
  rw [readNumber_ok h0, hx, hlen, List.append_assoc]

theorem readName_encode (path : String) {previous : Option (List (Word 8))} {name : List (Word 8)}
    (hv : name_valid name = true) (hs : sorts_after previous name = true) (rest : List (Word 8)) :
    (readName path previous).run (uvar_encode name.length ++ name ++ rest) = .ok (name, rest) := by
  simp only [name_valid, Bool.and_eq_true, decide_eq_true_eq] at hv
  obtain ⟨⟨h1, h2⟩, hu⟩ := hv
  unfold readName
  rw [List.append_assoc, bind_of_ok (readNumber_encode path (by omega) _),
    require_bind (by simp only [Bool.and_eq_true, decide_eq_true_eq]; exact ⟨by omega, h2⟩),
    readBytes_bind, require_bind hu]
  cases previous with
  | none => rfl
  | some p =>
    simp only [sorts_after] at hs
    simp only
    rw [require_bind, require_bind]
    · rfl
    · simp [bytes_lt_asymm hs]
    · simp only [decide_eq_true_eq]
      intro e; subst e; simp [bytes_lt_irrefl] at hs

/-! ### References -/

theorem checkRef_ok {count index ref : Nat} {path : String} {bs rest : List (Word 8)} {u : Unit}
    (h : (checkRef count index ref path).run bs = .ok (u, rest)) :
    ref < count ∧ ref < index ∧ rest = bs := by
  unfold checkRef at h
  obtain ⟨c1, h⟩ := require_bind_ok h
  obtain ⟨c2, rfl⟩ := require_ok h
  simp only [decide_eq_true_eq] at c1 c2
  exact ⟨c1, c2, rfl⟩

theorem checkRef_bind_ok {β : Type} {count index ref : Nat} {path : String}
    {f : Unit → Parser β} {bs rest : List (Word 8)} {b : β}
    (h : (checkRef count index ref path >>= f).run bs = .ok (b, rest)) :
    ref < count ∧ ref < index ∧ (f ()).run bs = .ok (b, rest) := by
  obtain ⟨u, mid, h1, h2⟩ := bind_eq_ok h
  obtain ⟨c1, c2, hm⟩ := checkRef_ok h1
  rw [hm] at h2
  exact ⟨c1, c2, h2⟩

theorem checkRef_bind {β : Type} {count index ref : Nat} {path : String} {f : Unit → Parser β}
    (h1 : ref < count) (h2 : ref < index) (bs : List (Word 8)) :
    (checkRef count index ref path >>= f).run bs = (f ()).run bs := by
  apply bind_of_ok
  unfold checkRef
  rw [require_bind (by simpa using h1)]
  simp [require, h2]

/-- The decoder's reference loop keeps `previous.toList ++ refs` increasing. -/
theorem increasing_cons {previous : Option Nat} {ref : Nat} {refs : List Nat}
    (ha : above previous ref = true) (h : increasing (ref :: refs) = true) :
    increasing (previous.toList ++ ref :: refs) = true := by
  cases previous with
  | none => exact h
  | some p => simp only [above, decide_eq_true_eq] at ha; simp [increasing, ha, h]

theorem above_of_increasing {previous : Option Nat} {ref : Nat} {refs : List Nat}
    (h : increasing (previous.toList ++ ref :: refs) = true) :
    above previous ref = true ∧ increasing (ref :: refs) = true := by
  cases previous with
  | none => exact ⟨rfl, h⟩
  | some p =>
    simp only [Option.toList_some, List.cons_append, List.nil_append, increasing,
      Bool.and_eq_true, decide_eq_true_eq] at h
    exact ⟨by simp [above, h.1], h.2⟩

/-- References that were read are encoded one number each, increase, and lie
below the theorem's index. -/
theorem readRefs_ok {count index : Nat} {base : String} :
    ∀ {n slot : Nat} {previous : Option Nat} {bs rest : List (Word 8)} {refs : List Nat},
      (readRefs count index base slot previous n).run bs = .ok (refs, rest) →
      bs = refs.flatMap uvar_encode ++ rest ∧ refs.length = n ∧
        increasing (previous.toList ++ refs) = true ∧ refs.all (· < index) = true
  | 0, slot, previous, bs, rest, refs, h => by
    obtain ⟨rfl, rfl⟩ := pure_eq_ok h
    cases previous <;> simp [increasing]
  | n + 1, slot, previous, bs, rest, refs, h => by
    simp only [readRefs] at h
    obtain ⟨ref, mid, h0, h⟩ := bind_eq_ok h
    obtain ⟨_, hri, h⟩ := checkRef_bind_ok h
    obtain ⟨ha, h⟩ := require_bind_ok h
    obtain ⟨tail, mid3, h2, h⟩ := bind_eq_ok h
    obtain ⟨hb, hlen, hinc, hall⟩ := readRefs_ok h2
    obtain ⟨rfl, rfl⟩ := pure_eq_ok h
    refine ⟨?_, by simp [hlen], increasing_cons ha hinc, by simp [hri, hall]⟩
    rw [readNumber_ok h0, hb, List.flatMap_cons, List.append_assoc]

theorem readRefs_encode {count index : Nat} {base : String} (hic : index ≤ count)
    (hc : count ≤ 64) :
    ∀ {refs : List Nat} {slot : Nat} {previous : Option Nat},
      increasing (previous.toList ++ refs) = true → refs.all (· < index) = true →
      ∀ rest : List (Word 8),
        (readRefs count index base slot previous refs.length).run
          (refs.flatMap uvar_encode ++ rest) = .ok (refs, rest)
  | [], _, _, _, _, _ => rfl
  | ref :: refs, slot, previous, hinc, hall, rest => by
    simp only [List.all_cons, Bool.and_eq_true, decide_eq_true_eq] at hall
    obtain ⟨ha, hinc⟩ := above_of_increasing hinc
    simp only [List.length_cons, readRefs, List.flatMap_cons, List.append_assoc]
    rw [bind_of_ok (readNumber_encode _ (by omega) _), checkRef_bind (by omega) hall.1,
      require_bind ha, bind_of_ok (readRefs_encode hic hc (previous := some ref) hinc hall.2 rest)]
    rfl

/-! ### Records -/

/-- The fields that were read are the encoding of the record, which has the
name and tag it was read with and meets `fields_ok`. -/
theorem readFields_ok {count : Nat} {before : List Record} {base : String} {name : List (Word 8)}
    {tag : Nat} {bs rest : List (Word 8)} {r : Record}
    (h : (readFields count before base name tag).run bs = .ok (r, rest)) :
    r.name = name ∧ r.tag = tag ∧ bs = encode_fields r ++ rest ∧ fields_ok before r = true := by
  unfold readFields at h
  split at h
  · obtain ⟨digest, mid, h1, h⟩ := bind_eq_ok h
    obtain ⟨hd, hlen⟩ := readBytes_ok h1
    obtain ⟨rfl, rfl⟩ := pure_eq_ok h
    exact ⟨rfl, rfl, hd, by simp [fields_ok, hlen]⟩
  · obtain ⟨fp, mid, h1, h⟩ := bind_eq_ok h
    obtain ⟨hf, hflen⟩ := readBytes_ok h1
    obtain ⟨total, mid2, h2, h⟩ := bind_eq_ok h
    obtain ⟨c, h⟩ := require_bind_ok h
    obtain ⟨refs, mid3, h3, h⟩ := bind_eq_ok h
    obtain ⟨hr, hrlen, hinc, hall⟩ := readRefs_ok h3
    obtain ⟨rfl, rfl⟩ := pure_eq_ok h
    simp only [decide_eq_true_eq] at c
    refine ⟨rfl, rfl, ?_, ?_⟩
    · rw [hf, readNumber_ok h2, hr, ← hrlen]
      simp [encode_fields]
    · simp only [Option.toList_none, List.nil_append] at hinc
      simp [fields_ok, hflen, hrlen, c, hinc, hall]
  · obtain ⟨ref, mid, h1, h⟩ := bind_eq_ok h
    obtain ⟨_, hri, h⟩ := checkRef_bind_ok h
    obtain ⟨hk, h⟩ := require_bind_ok h
    obtain ⟨level, mid3, h3, h⟩ := bind_eq_ok h
    obtain ⟨hl, h⟩ := require_bind_ok h
    obtain ⟨rfl, rfl⟩ := pure_eq_ok h
    simp only [decide_eq_true_eq] at hl
    refine ⟨rfl, rfl, ?_, by simp [fields_ok, hri, hk, hl]⟩
    rw [readNumber_ok h1, readByte_ok h3]
    simp [encode_fields, word_of_nat_toNat]
  · cases h

theorem readFields_encode {count : Nat} {before : List Record} {base : String} {r : Record}
    (hf : fields_ok before r = true) (hlt : before.length < count) (hc : count ≤ 64)
    (rest : List (Word 8)) :
    (readFields count before base r.name r.tag).run (encode_fields r ++ rest) = .ok (r, rest) := by
  cases r with
  | rdef name digest =>
    simp only [fields_ok, decide_eq_true_eq] at hf
    simp only [Record.name, Record.tag, readFields, encode_fields]
    rw [← hf, readBytes_bind]
    rfl
  | rthm name fp refs =>
    simp only [fields_ok, Bool.and_eq_true, decide_eq_true_eq] at hf
    obtain ⟨⟨⟨hfp, hlen⟩, hinc⟩, hall⟩ := hf
    simp only [Record.name, Record.tag, readFields, encode_fields, List.append_assoc]
    rw [← hfp, readBytes_bind, bind_of_ok (readNumber_encode _ (by omega) _),
      require_bind (by simpa using hlen),
      bind_of_ok (readRefs_encode (Nat.le_of_lt hlt) hc (previous := none) hinc hall rest)]
    rfl
  | rclaim name ref level =>
    simp only [fields_ok, Bool.and_eq_true, decide_eq_true_eq] at hf
    obtain ⟨⟨hri, hk⟩, hl⟩ := hf
    simp only [Record.name, Record.tag, readFields, encode_fields, List.append_assoc]
    rw [bind_of_ok (readNumber_encode _ (by omega) _), checkRef_bind (by omega) hri,
      require_bind hk,
      List.singleton_append, readByte_bind, toNat_word_of_nat (by omega),
      require_bind (by simpa using hl)]
    rfl

theorem tag_lt (r : Record) : r.tag < 256 := by cases r <;> simp [Record.tag]

/-- A record that was read is encoded as `encode_record` and meets `record_ok`. -/
theorem readRecord_ok {count : Nat} {before : List Record} {bs rest : List (Word 8)} {r : Record}
    (h : (readRecord count before).run bs = .ok (r, rest)) :
    bs = encode_record r ++ rest ∧ record_ok before r = true := by
  unfold readRecord at h
  obtain ⟨tag, mid, h0, h⟩ := bind_eq_ok h
  obtain ⟨_, h⟩ := require_bind_ok h
  obtain ⟨name, mid2, h1, h⟩ := bind_eq_ok h
  obtain ⟨hn, hv, hs⟩ := readName_ok h1
  obtain ⟨hname, htag, hf, hok⟩ := readFields_ok h
  refine ⟨?_, by simp [record_ok, hname, hv, hs, hok]⟩
  rw [readByte_ok h0, hn, hf, encode_record, htag, word_of_nat_toNat, hname]
  simp

theorem readRecord_encode {count : Nat} {before : List Record} {r : Record}
    (hok : record_ok before r = true) (hlt : before.length < count) (hc : count ≤ 64)
    (rest : List (Word 8)) :
    (readRecord count before).run (encode_record r ++ rest) = .ok (r, rest) := by
  simp only [record_ok, Bool.and_eq_true] at hok
  obtain ⟨⟨hv, hs⟩, hf⟩ := hok
  have htag : (word_of_nat 8 r.tag).toNat = r.tag := toNat_word_of_nat (tag_lt r)
  have h13 : 1 ≤ r.tag ∧ r.tag ≤ 3 := by cases r <;> simp [Record.tag]
  unfold readRecord encode_record
  simp only [List.cons_append]
  rw [readByte_bind, require_bind (by simp [htag, h13]), List.append_assoc,
    bind_of_ok (readName_encode _ hv hs _), htag]
  exact readFields_encode hf hlt hc rest

theorem readRecords_ok {count : Nat} :
    ∀ {n : Nat} {before : List Record} {bs rest : List (Word 8)} {l : List Record},
      (readRecords count before n).run bs = .ok (l, rest) →
      bs = l.flatMap encode_record ++ rest ∧ l.length = n ∧ records_ok_after before l = true
  | 0, before, bs, rest, l, h => by
    obtain ⟨rfl, rfl⟩ := pure_eq_ok h
    exact ⟨rfl, rfl, rfl⟩
  | n + 1, before, bs, rest, l, h => by
    simp only [readRecords] at h
    obtain ⟨r, mid, h0, h⟩ := bind_eq_ok h
    obtain ⟨hr, hok⟩ := readRecord_ok h0
    obtain ⟨tail, mid2, h1, h⟩ := bind_eq_ok h
    obtain ⟨ht, hlen, hoks⟩ := readRecords_ok h1
    obtain ⟨rfl, rfl⟩ := pure_eq_ok h
    refine ⟨?_, by simp [hlen], by simp [records_ok_after, hok, hoks]⟩
    rw [hr, ht, List.flatMap_cons, List.append_assoc]

theorem readRecords_encode {count : Nat} (hc : count ≤ 64) :
    ∀ {l before : List Record}, records_ok_after before l = true →
      before.length + l.length = count → ∀ rest : List (Word 8),
      (readRecords count before l.length).run (l.flatMap encode_record ++ rest) = .ok (l, rest)
  | [], _, _, _, _ => rfl
  | r :: l, before, hok, hlen, rest => by
    simp only [records_ok_after, Bool.and_eq_true] at hok
    simp only [List.length_cons] at hlen
    simp only [List.length_cons, readRecords, List.flatMap_cons, List.append_assoc]
    rw [bind_of_ok (readRecord_encode hok.1 (by omega) hc _),
      bind_of_ok (readRecords_encode hc hok.2 (by simp; omega) rest)]
    rfl

/-! ### The whole input -/

theorem header_eq (a b c d : Word 8) (ha : a.toNat = 0x4F) (hb : b.toNat = 0x43)
    (hc : c.toNat = 0x52) (hd : d.toNat = 1) : [a, b, c, d] = header := by
  have ea : a = 0x4F := BitVec.eq_of_toNat_eq ha
  have eb : b = 0x43 := BitVec.eq_of_toNat_eq hb
  have ec : c = 0x52 := BitVec.eq_of_toNat_eq hc
  have ed : d = 0x01 := BitVec.eq_of_toNat_eq hd
  rw [ea, eb, ec, ed]; rfl

/-- Backwards for the whole input: what was read is the encoding of the
records, which number at most 64 and each meet `record_ok`. -/
theorem readFile_ok {bs rest : List (Word 8)} {l : List Record}
    (h : readFile.run bs = .ok (l, rest)) :
    bs = encode_records l ++ rest ∧ l.length ≤ 64 ∧ records_ok_after [] l = true := by
  unfold readFile at h
  obtain ⟨a, m1, ha, e1, h⟩ := readMagic_bind_ok h
  obtain ⟨b, m2, hb, e2, h⟩ := readMagic_bind_ok h
  obtain ⟨c, m3, hc, e3, h⟩ := readMagic_bind_ok h
  obtain ⟨d, m4, h4, h⟩ := bind_eq_ok h
  rw [e1, e2, e3, readByte_ok h4]
  obtain ⟨hd, h⟩ := require_bind_ok h
  obtain ⟨count, m5, h5, h⟩ := bind_eq_ok h
  obtain ⟨hcount, h⟩ := require_bind_ok h
  obtain ⟨hr, hlen, hok⟩ := readRecords_ok h
  simp only [decide_eq_true_eq] at hd hcount
  refine ⟨?_, by omega, hok⟩
  rw [readNumber_ok h5, hr, encode_records, ← header_eq a b c d ha hb hc hd, hlen]
  simp

/-- Forwards for the whole input. -/
theorem readFile_encode {l : List Record} (hlen : l.length ≤ 64)
    (hok : records_ok_after [] l = true) (rest : List (Word 8)) :
    readFile.run (encode_records l ++ rest) = .ok (l, rest) := by
  unfold readFile encode_records
  simp only [header, List.cons_append, List.nil_append, List.append_assoc]
  rw [readMagic_bind (e := 0x4F) (b := 0x4F) rfl, readMagic_bind (e := 0x43) (b := 0x43) rfl,
    readMagic_bind (e := 0x52) (b := 0x52) rfl, readByte_bind,
    require_bind (ok := decide ((1 : Word 8).toNat = 1)) rfl,
    bind_of_ok (readNumber_encode _ (by omega) _), require_bind (by simpa using hlen)]
  exact readRecords_encode (count := l.length) (by omega) hok (by simp) rest

/-- An accepted input has at most 4096 bytes and is read with nothing left over. -/
theorem decode_records_ok {b : List (Word 8)} {l : List Record} (h : decode_records b = .ok l) :
    b.length ≤ 4096 ∧ readFile.run b = .ok (l, []) := by
  unfold decode_records at h
  split at h
  · cases h
  · refine ⟨by omega, ?_⟩
    split at h
    · cases h; assumption
    · cases h
    · cases h

/-! ## Theorems D3-TH01 to D3-TH05 -/

/-- D3-TH01: encoding round-trips. -/
theorem encoding_round_trips (l : List Record) (h : records_valid l = true) :
    decode_records (encode_records l) = .ok l := by
  simp only [records_valid, Bool.and_eq_true, decide_eq_true_eq] at h
  obtain ⟨⟨hlen, hok⟩, hsize⟩ := h
  have hrun := readFile_encode hlen hok []
  rw [List.append_nil] at hrun
  simp [decode_records, hrun, Nat.not_lt.mpr hsize]

/-- D3-TH02: accepted bytes are canonical. -/
theorem accepted_bytes_are_canonical (b : List (Word 8)) (l : List Record)
    (h : decode_records b = .ok l) : encode_records l = b := by
  obtain ⟨hb, _, _⟩ := readFile_ok (decode_records_ok h).2
  rw [hb, List.append_nil]

/-- D3-TH03: accepted values are valid. -/
theorem accepted_values_are_valid (b : List (Word 8)) (l : List Record)
    (h : decode_records b = .ok l) : records_valid l = true := by
  obtain ⟨hsize, hrun⟩ := decode_records_ok h
  obtain ⟨hb, hlen, hok⟩ := readFile_ok hrun
  rw [List.append_nil] at hb
  subst hb
  simp [records_valid, hlen, hok, hsize]

/-- D3-TH04: accepted inputs are bounded. -/
theorem accepted_inputs_are_bounded (b : List (Word 8)) (l : List Record)
    (h : decode_records b = .ok l) : b.length ≤ 4096 :=
  (decode_records_ok h).1

/-- D3-TH05: one value has one encoding. -/
theorem one_value_has_one_encoding (b1 b2 : List (Word 8)) (l : List Record)
    (h1 : decode_records b1 = .ok l) (h2 : decode_records b2 = .ok l) : b1 = b2 := by
  rw [← accepted_bytes_are_canonical b1 l h1, ← accepted_bytes_are_canonical b2 l h2]

end D006.Records
