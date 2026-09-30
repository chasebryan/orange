import D006.Core

/-!
# D-006 v0.3, candidate C-02 (Lean 4): DS-04, replaying an LRAT certificate

Every DS-04 symbol maps to a declaration here (see `adapter.d/ds04.json`).

* The obligations are bit-vector identities over the DS-01 words
  (`Obligation`, `holds`), written as `Term`s so that the meaning (`Term.eval`)
  and the canonical bit-blast (`Term.blast`, `blast`) read the same syntax.
* `blast` is the canonical Tseitin encoding of the shared rules (gate
  numbering, clause order, miter); `cnf_text` prints it.
* `lrat_verdict` is an LRAT checker over the certificate's UTF-8 bytes: it
  compares the CNF text, refuses certificates above 4 MiB, then checks each
  line in the shared order (`parse`, `id_order`, `var_range`, `lemma_form`,
  `rat_unsupported`, `unknown_hint`/`rup`, `unknown_deletion`, `trailing`).
  Active clauses and the propagation assignment live in binary tries keyed by
  natural numbers (`Trie`).
* `accepted_certificate_proves` (D4-TH04) is proved once for every
  obligation and certificate: an accepting run only ever adds clauses that are
  true under every assignment satisfying the CNF (reverse unit propagation is
  sound), so an accepted empty clause makes the CNF unsatisfiable; and when an
  identity fails for some `x` and `y`, evaluating the circuit on them satisfies
  every clause (`blast_satisfiable`), because the circuit computes the words'
  bits (ripple-carry addition included).
* `carry_save_identity` (D4-TH01) is D4-TH04 applied to the golden
  certificate, whose acceptance (`golden_certificate_accepted`) is the only
  fact in this file established by `native_decide`: the kernel cannot evaluate
  a checker over a 62 KB string literal within the laboratory's limits (see
  NOTES.md). Everything else is checked by the kernel.
-/

namespace D006.Lrat

/-! ## Obligations, verdicts and their meaning (B-T01, B-T02, B-F03) -/

/-- B-T01: the two shared obligations. -/
inductive Obligation : Type where
  /-- B-C01: `x + y = (x ^^^ y) + ((x &&& y) <<< 1)` at width 32. -/
  | carry_save
  /-- B-C02: the same identity without the carry shift (false). -/
  | carry_save_unshifted
  deriving DecidableEq, Repr

/-- B-T02: the checker's verdict, B-C03 `accept` or B-C04 `reject code line`. -/
inductive Verdict : Type where
  | accept
  | reject (code : String) (line : Nat)
  deriving DecidableEq, Repr

/-- Bit-vector terms over the two 32-bit inputs. -/
inductive Term : Type where
  | x
  | y
  | add (a b : Term)
  | xor (a b : Term)
  | and (a b : Term)
  | shl (a : Term) (k : Nat)

/-- The word a term denotes, through the DS-01 word operations. -/
def Term.eval : Term → Word 32 → Word 32 → Word 32
  | .x, u, _ => u
  | .y, _, v => v
  | .add a b, u, v => word_add (a.eval u v) (b.eval u v)
  | .xor a b, u, v => word_xor (a.eval u v) (b.eval u v)
  | .and a b, u, v => word_and (a.eval u v) (b.eval u v)
  | .shl a k, u, v => word_shl (a.eval u v) k

/-- The left side of both obligations, `x + y`. -/
def Obligation.lhs (_ : Obligation) : Term := .add .x .y

/-- The right side of each obligation. -/
def Obligation.rhs : Obligation → Term
  | .carry_save => .add (.xor .x .y) (.shl (.and .x .y) 1)
  | .carry_save_unshifted => .add (.xor .x .y) (.and .x .y)

/-- B-F03: the obligation's identity holds for all 32-bit `x` and `y`. -/
def holds (o : Obligation) : Prop := ∀ x y : Word 32, o.lhs.eval x y = o.rhs.eval x y

/-- D4-TH02: the meaning of B-C01. -/
theorem holds_carry_save :
    holds .carry_save ↔
      ∀ x y : Word 32, word_add x y = word_add (word_xor x y) (word_shl (word_and x y) 1) :=
  Iff.rfl

/-- D4-TH03: the meaning of B-C02. -/
theorem holds_carry_save_unshifted :
    holds .carry_save_unshifted ↔
      ∀ x y : Word 32, word_add x y = word_add (word_xor x y) (word_and x y) :=
  Iff.rfl

/-- D4-TH05: B-C02 is false; `x = y = 1` gives `2` on the left and `1` on the right. -/
theorem carry_save_unshifted_refuted : ¬ holds .carry_save_unshifted := fun h =>
  absurd (holds_carry_save_unshifted.mp h 1 1) (by decide)

/-! ## The canonical bit-blast (shared rules 1 to 5) -/

/-- The three gate kinds. -/
inductive Kind : Type where
  | and
  | xor
  | or

/-- What a gate of each kind computes. -/
def Kind.eval : Kind → Bool → Bool → Bool
  | .and, a, b => a && b
  | .xor, a, b => a ^^ b
  | .or, a, b => a || b

/-- A gate: variable `out` is `kind` of variables `a` and `b`. -/
structure Gate : Type where
  out : Nat
  kind : Kind
  a : Nat
  b : Nat

/-- The Tseitin clauses of a gate, in the order of rule 2. -/
def Gate.clauses (g : Gate) : List (List Int) :=
  let o : Int := g.out
  let a : Int := g.a
  let b : Int := g.b
  match g.kind with
  | .and => [[-o, a], [-o, b], [o, -a, -b]]
  | .xor => [[-o, a, b], [-o, -a, -b], [o, -a, b], [o, a, -b]]
  | .or => [[o, -a], [o, -b], [-o, a, b]]

/-- The blaster's state: the last variable used and the gates so far, newest first. -/
structure Circuit : Type where
  top : Nat
  gates : List Gate

/-- Adds a gate as the next variable. -/
def Circuit.gate (c : Circuit) (k : Kind) (a b : Nat) : Circuit :=
  ⟨c.top + 1, ⟨c.top + 1, k, a, b⟩ :: c.gates⟩

@[simp] theorem Circuit.gate_top (c : Circuit) (k : Kind) (a b : Nat) :
    (c.gate k a b).top = c.top + 1 := rfl

@[simp] theorem Circuit.gate_gates (c : Circuit) (k : Kind) (a b : Nat) :
    (c.gate k a b).gates = ⟨c.top + 1, k, a, b⟩ :: c.gates := rfl

/-- One gate per bit position, least significant bit first. -/
def zipGates (k : Kind) : List Nat → List Nat → Circuit → List Nat × Circuit
  | a :: as, b :: bs, c =>
    let c₁ := c.gate k a b
    let r := zipGates k as bs c₁
    (c₁.top :: r.1, r.2)
  | _, _, c => ([], c)

/-- The ripple-carry adder of rule 3: per bit `t = a xor b`, `s = t xor carry`,
`u = a and b`, `v = carry and t`, `carry' = u or v`. -/
def ripple (carry : Nat) : List Nat → List Nat → Circuit → List Nat × Circuit
  | a :: as, b :: bs, c =>
    let t := c.gate .xor a b
    let s := t.gate .xor t.top carry
    let u := s.gate .and a b
    let v := u.gate .and carry t.top
    let c' := v.gate .or u.top v.top
    let r := ripple c'.top as bs c'
    (s.top :: r.1, r.2)
  | _, _, c => ([], c)

/-- The 32 input variables from `first` (x from 2, y from 34). -/
def inputBits (first : Nat) : List Nat := (List.range 32).map (first + ·)

/-- The literals of a term's 32 bits, left operand blasted first; variable 1 is
constant false and is what a shift moves in. -/
def Term.blast : Term → Circuit → List Nat × Circuit
  | .x, c => (inputBits 2, c)
  | .y, c => (inputBits 34, c)
  | .shl a k, c =>
    let r := a.blast c
    ((List.replicate k 1 ++ r.1).take 32, r.2)
  | .xor a b, c =>
    let l := a.blast c
    let r := b.blast l.2
    zipGates .xor l.1 r.1 r.2
  | .and a b, c =>
    let l := a.blast c
    let r := b.blast l.2
    zipGates .and l.1 r.1 r.2
  | .add a b, c =>
    let l := a.blast c
    let r := b.blast l.2
    ripple 1 l.1 r.1 r.2

/-- `o = OR(o, d)` for each remaining difference. -/
def orChain : Nat → List Nat → Circuit → Nat × Circuit
  | o, [], c => (o, c)
  | o, d :: ds, c =>
    let c₁ := c.gate .or o d
    orChain c₁.top ds c₁

/-- The miter of rule 4: `d_i = XOR(lhs_i, rhs_i)`, then the OR chain. -/
def miter (l r : List Nat) (c : Circuit) : Nat × Circuit :=
  let d := zipGates .xor l r c
  match d.1 with
  | [] => (1, d.2)
  | d₀ :: ds => orChain d₀ ds d.2

/-- The canonical CNF of an obligation: the number of variables and the clauses
(`-1`, every gate's clauses in creation order, the miter output). -/
def blast (o : Obligation) : Nat × List (List Int) :=
  let l := o.lhs.blast ⟨65, []⟩
  let r := o.rhs.blast l.2
  let m := miter l.1 r.1 r.2
  (m.2.top, [-1] :: (m.2.gates.reverse.flatMap Gate.clauses ++ [[(m.1 : Int)]]))

/-- One clause line of the DIMACS text. -/
def clauseLine (c : List Int) : String :=
  " ".intercalate (c.map toString) ++ " 0\n"

/-- B-F01: the canonical CNF text (rule 5). -/
def cnf_text (o : Obligation) : String :=
  let b := blast o
  s!"p cnf {b.1} {b.2.length}\n" ++ String.join (b.2.map clauseLine)

/-! ## Binary tries keyed by naturals -/

/-- A map from naturals, least significant bit first: key 0 is the root, a
nonzero key `k` goes to the `k % 2` child with key `k / 2`. -/
inductive Trie (α : Type) : Type where
  | leaf
  | node (root : Option α) (zero one : Trie α)

namespace Trie

variable {α : Type}

/-- The value at the root. -/
def root : Trie α → Option α
  | leaf => none
  | node r _ _ => r

/-- The child for even keys. -/
def zero : Trie α → Trie α
  | leaf => leaf
  | node _ z _ => z

/-- The child for odd keys. -/
def one : Trie α → Trie α
  | leaf => leaf
  | node _ _ o => o

/-- The value at key `k`. -/
def get? (t : Trie α) (k : Nat) : Option α :=
  if k = 0 then t.root
  else if k % 2 = 0 then t.zero.get? (k / 2) else t.one.get? (k / 2)
termination_by k
decreasing_by all_goals omega

/-- Sets (or, with `none`, clears) the value at key `k`. -/
def set (t : Trie α) (k : Nat) (v : Option α) : Trie α :=
  if k = 0 then node v t.zero t.one
  else if k % 2 = 0 then node t.root (t.zero.set (k / 2) v) t.one
  else node t.root t.zero (t.one.set (k / 2) v)
termination_by k
decreasing_by all_goals omega

theorem get?_eq (t : Trie α) (k : Nat) :
    t.get? k = if k = 0 then t.root
      else if k % 2 = 0 then t.zero.get? (k / 2) else t.one.get? (k / 2) := by
  rw [get?]

theorem get?_leaf (k : Nat) : (leaf : Trie α).get? k = none := by
  induction k using Nat.strongRecOn with
  | ind k ih =>
    rw [get?_eq]
    by_cases h0 : k = 0
    · rw [ite_eq_left h0]; rfl
    · rw [ite_eq_right h0]
      have hk : k / 2 < k := by omega
      split
      · exact ih _ hk
      · exact ih _ hk

theorem get?_set (t : Trie α) (k : Nat) (v : Option α) (j : Nat) :
    (t.set k v).get? j = if j = k then v else t.get? j := by
  induction k using Nat.strongRecOn generalizing t j with
  | ind k ih =>
    by_cases hk : k = 0
    · have hs : t.set k v = node v t.zero t.one := by rw [set, ite_eq_left hk]
      rw [hs, get?_eq, get?_eq t j]
      by_cases hj : j = 0
      · rw [ite_eq_left hj, ite_eq_left hj, ite_eq_left (hj.trans hk.symm)]; rfl
      · have hjk : ¬ j = k := by omega
        rw [ite_eq_right hj, ite_eq_right hj, ite_eq_right hjk]; rfl
    · by_cases he : k % 2 = 0
      · have hs : t.set k v = node t.root (t.zero.set (k / 2) v) t.one := by
          rw [set, ite_eq_right hk, ite_eq_left he]
        rw [hs, get?_eq, get?_eq t j]
        by_cases hj : j = 0
        · have hjk : ¬ j = k := by omega
          rw [ite_eq_left hj, ite_eq_right hjk, ite_eq_left hj]; rfl
        · rw [ite_eq_right hj, ite_eq_right hj]
          by_cases hje : j % 2 = 0
          · rw [ite_eq_left hje, ite_eq_left hje]
            show (t.zero.set (k / 2) v).get? (j / 2) = _
            rw [ih (k / 2) (by omega)]
            by_cases hjk : j = k
            · have : j / 2 = k / 2 := by omega
              rw [ite_eq_left this, ite_eq_left hjk]
            · have : ¬ j / 2 = k / 2 := by omega
              rw [ite_eq_right this, ite_eq_right hjk]
          · have hjk : ¬ j = k := by omega
            rw [ite_eq_right hje, ite_eq_right hje, ite_eq_right hjk]; rfl
      · have hs : t.set k v = node t.root t.zero (t.one.set (k / 2) v) := by
          rw [set, ite_eq_right hk, ite_eq_right he]
        rw [hs, get?_eq, get?_eq t j]
        by_cases hj : j = 0
        · have hjk : ¬ j = k := by omega
          rw [ite_eq_left hj, ite_eq_right hjk, ite_eq_left hj]; rfl
        · rw [ite_eq_right hj, ite_eq_right hj]
          by_cases hje : j % 2 = 0
          · have hjk : ¬ j = k := by omega
            rw [ite_eq_left hje, ite_eq_left hje, ite_eq_right hjk]; rfl
          · rw [ite_eq_right hje, ite_eq_right hje]
            show (t.one.set (k / 2) v).get? (j / 2) = _
            rw [ih (k / 2) (by omega)]
            by_cases hjk : j = k
            · have : j / 2 = k / 2 := by omega
              rw [ite_eq_left this, ite_eq_left hjk]
            · have : ¬ j / 2 = k / 2 := by omega
              rw [ite_eq_right this, ite_eq_right hjk]

end Trie

/-! ## The checker -/

/-- A set of literals, kept as a partial map from variables to polarities
(`true` for the positive literal); the checker never holds both polarities of
one variable. -/
abbrev Assignment : Type := Trie Bool

/-- Whether literal `l` is in the set. -/
def has (a : Assignment) (l : Int) : Bool :=
  decide (a.get? l.natAbs = some (decide (0 < l)))

/-- Adds literal `l`. -/
def assign (a : Assignment) (l : Int) : Assignment :=
  a.set l.natAbs (some (decide (0 < l)))

/-- The negations of a lemma's literals, where reverse unit propagation starts. -/
def refute (lemma : List Int) : Assignment :=
  lemma.foldl (fun a l => assign a (-l)) .leaf

/-- No literal is repeated and none occurs with its complement (literal `0`
counts as its own complement, as `-0 = 0`). -/
def lemmaFormOk : Assignment → List Int → Bool
  | _, [] => true
  | s, l :: ls => if l = 0 ∨ has s l = true ∨ has s (-l) = true then false else lemmaFormOk (assign s l) ls

/-- Reverse unit propagation along the hints (rule 7): every hint but the last
is unit and assigns its open literal, the last is falsified. `none` when the
lemma is justified, otherwise the rejection code. -/
def propagate (db : Trie (List Int)) : Assignment → List Nat → Option String
  | _, [] => some "rup"
  | a, h :: hs =>
    match db.get? h with
    | none => some "unknown_hint"
    | some c =>
      if c.any (has a) then some "rup"
      else
        match c.filter (fun l => !has a (-l)) with
        | [] => if hs.isEmpty then none else some "rup"
        | [u] => propagate db (assign a u) hs
        | _ => some "rup"

/-- Python's `split` on one separator byte: every piece, empty ones included. -/
def splitOn (sep : UInt8) (bytes : List UInt8) : List (List UInt8) :=
  go [] bytes
where
  /-- `cur` is the current piece, reversed. -/
  go (cur : List UInt8) : List UInt8 → List (List UInt8)
    | [] => [cur.reverse]
    | b :: bs => if b = sep then cur.reverse :: go [] bs else go (b :: cur) bs

/-- The certificate's lines, each with whether a line feed ended it (only a
last, unterminated line has `false`). `cur` is the current line, reversed. -/
def lines (cur : List UInt8) : List UInt8 → List (List UInt8 × Bool)
  | [] => if cur.isEmpty then [] else [(cur.reverse, false)]
  | b :: bs => if b = 10 then (cur.reverse, true) :: lines [] bs else lines (b :: cur) bs

/-- An ASCII decimal digit. -/
def isDigit (b : UInt8) : Bool := 48 ≤ b && b ≤ 57

/-- The value of a string of digits after `n`. -/
def digitsValue (n : Nat) : List UInt8 → Nat
  | [] => n
  | b :: bs => digitsValue (10 * n + (b.toNat - 48)) bs

/-- A canonical decimal natural: digits only, no leading zero. -/
def decNat (t : List UInt8) : Option Nat :=
  match t with
  | [] => none
  | b :: bs => if t.all isDigit && (b != 48 || bs.isEmpty) then some (digitsValue 0 t) else none

/-- A canonical decimal integer: a natural, or `-` and a nonzero natural. -/
def decInt (t : List UInt8) : Option Int :=
  match t with
  | [] => none
  | b :: body =>
    if b = 45 then
      match decNat body with
      | some (n + 1) => some (-((n + 1 : Nat) : Int))
      | _ => none
    else (decNat t).map Int.ofNat

/-- A parsed certificate line. -/
inductive Line : Type where
  /-- `ID d IDS 0` -/
  | delete (ids : List Nat)
  /-- `ID LITERALS 0 HINTS 0` -/
  | add (id : Nat) (lemma : List Int) (hints : List Int)

/-- The fields after `ID d`: nonzero identifiers, then `0`. -/
def parseDeletion (rest : List (List UInt8)) : Option Line :=
  match rest.reverse with
  | last :: middle =>
    if last = [48] then
      match middle.reverse.mapM decNat with
      | some ids => if ids.any (· = 0) then none else some (.delete ids)
      | none => none
    else none
  | [] => none

/-- The fields after `ID` of an addition: literals, `0`, hints, `0`. -/
def parseAddition (id : Nat) (fields : List (List UInt8)) : Option Line :=
  match fields.mapM decInt with
  | none => none
  | some values =>
    let lemma := values.takeWhile (· ≠ 0)
    match values.drop lemma.length with
    | [] => none
    | _ :: after =>
      match after.reverse with
      | last :: hints => if last = 0 ∧ hints.all (· ≠ 0) then some (.add id lemma hints.reverse) else none
      | [] => none

/-- A line's syntax (rule 6): at least two nonempty space-separated fields and a
nonzero canonical identifier. -/
def parseLine (tokens : List (List UInt8)) : Option Line :=
  match tokens with
  | t₀ :: t₁ :: rest =>
    if tokens.any List.isEmpty then none
    else
      match decNat t₀ with
      | none | some 0 => none
      | some id => if t₁ = [100] then parseDeletion rest else parseAddition id (t₁ :: rest)
  | _ => none

/-- Removes clauses; `none` if one is not active. -/
def deleteAll (db : Trie (List Int)) : List Nat → Option (Trie (List Int))
  | [] => some db
  | t :: ts =>
    match db.get? t with
    | none => none
    | some _ => deleteAll (db.set t none) ts

/-- The replay state: active clauses, the last added identifier, and whether
the empty clause has been derived. -/
structure Replay : Type where
  db : Trie (List Int)
  last : Nat
  done : Bool

/-- Checks one line in the order of semantics.md section 5. -/
def checkLine (vars : Nat) (st : Replay) (line : List UInt8) : Except String Replay :=
  match parseLine (splitOn 32 line) with
  | none => .error "parse"
  | some (.delete ids) =>
    match deleteAll st.db ids with
    | none => .error "unknown_deletion"
    | some db => .ok { st with db := db }
  | some (.add id lemma hints) =>
    if id ≤ st.last then .error "id_order"
    else if lemma.any (fun l => vars < l.natAbs) then .error "var_range"
    else if !lemmaFormOk .leaf lemma then .error "lemma_form"
    else if hints.any (· < 0) then .error "rat_unsupported"
    else
      match propagate st.db (refute lemma) (hints.map Int.toNat) with
      | some code => .error code
      | none => .ok { db := st.db.set id (some lemma), last := id, done := lemma.isEmpty }

/-- Replays the lines from line number `n`. -/
def replay (vars : Nat) : Replay → Nat → List (List UInt8 × Bool) → Verdict
  | st, _, [] => if st.done then .accept else .reject "no_empty_clause" 0
  | st, n, (line, ended) :: rest =>
    if st.done then .reject "trailing" n
    else if !ended then .reject "parse" n
    else
      match checkLine vars st line with
      | .error code => .reject code n
      | .ok st' => replay vars st' (n + 1) rest

/-- The CNF's clauses as active clauses `1, 2, ...`. -/
def initDb (clauses : List (List Int)) : Trie (List Int) :=
  go 1 .leaf clauses
where
  go (i : Nat) (t : Trie (List Int)) : List (List Int) → Trie (List Int)
    | [] => t
    | c :: cs => go (i + 1) (t.set i (some c)) cs

/-- The largest certificate accepted for checking, 4 MiB. -/
def maxCertificate : Nat := 4 * 1024 * 1024

/-- B-F02: checks certificate `certificate` against obligation `o` whose CNF text
is claimed to be `cnf`. -/
def lrat_verdict (o : Obligation) (cnf certificate : String) : Verdict :=
  if cnf ≠ cnf_text o then .reject "cnf_mismatch" 0
  else if maxCertificate < certificate.utf8ByteSize then .reject "oversized" 0
  else
    let b := blast o
    replay b.1 ⟨initDb b.2, b.2.length, false⟩ 1 (lines [] certificate.toByteArray.data.toList)

/-! ## Soundness of the checker -/

/-- A literal under a total assignment of the variables; `0` is never true. -/
def litTrue (α : Nat → Bool) (l : Int) : Bool :=
  if 0 < l then α l.natAbs else if l < 0 then !α l.natAbs else false

/-- A clause under a total assignment. -/
def clauseTrue (α : Nat → Bool) (c : List Int) : Bool := c.any (litTrue α)

theorem litTrue_ne_zero {α : Nat → Bool} {l : Int} (h : litTrue α l = true) : l ≠ 0 := by
  intro h0
  subst h0
  simp [litTrue] at h

theorem litTrue_neg {α : Nat → Bool} {l : Int} (h : litTrue α (-l) = true) : litTrue α l = false := by
  unfold litTrue at h ⊢
  rw [Int.natAbs_neg] at h
  by_cases hp : 0 < l
  · have h1 : ¬ 0 < -l := by omega
    have h2 : -l < 0 := by omega
    rw [ite_eq_right h1, ite_eq_left h2] at h
    rw [ite_eq_left hp]
    simpa using h
  · by_cases hn : l < 0
    · have h1 : 0 < -l := by omega
      rw [ite_eq_left h1] at h
      rw [ite_eq_right hp, ite_eq_left hn, h]
      rfl
    · rw [ite_eq_right hp, ite_eq_right hn]

theorem litTrue_neg_of_false {α : Nat → Bool} {l : Int} (h0 : l ≠ 0) (h : litTrue α l = false) :
    litTrue α (-l) = true := by
  unfold litTrue at h ⊢
  rw [Int.natAbs_neg]
  by_cases hp : 0 < l
  · have h1 : ¬ 0 < -l := by omega
    have h2 : -l < 0 := by omega
    rw [ite_eq_left hp] at h
    rw [ite_eq_right h1, ite_eq_left h2, h]
    rfl
  · have hn : l < 0 := by omega
    have h1 : 0 < -l := by omega
    rw [ite_eq_right hp, ite_eq_left hn] at h
    rw [ite_eq_left h1]
    simpa using h

theorem litTrue_ofNat {α : Nat → Bool} {n : Nat} (h : 1 ≤ n) : litTrue α (n : Int) = α n := by
  have h1 : (0 : Int) < n := by omega
  unfold litTrue
  rw [ite_eq_left h1, Int.natAbs_natCast]

theorem litTrue_negOfNat {α : Nat → Bool} {n : Nat} (h : 1 ≤ n) :
    litTrue α (-(n : Int)) = !α n := by
  have h1 : ¬ (0 : Int) < -(n : Int) := by omega
  have h2 : -(n : Int) < 0 := by omega
  unfold litTrue
  rw [ite_eq_right h1, ite_eq_left h2, Int.natAbs_neg, Int.natAbs_natCast]

/-- Every literal in the set is true under `α`. -/
def Agrees (α : Nat → Bool) (a : Assignment) : Prop := ∀ l, has a l = true → litTrue α l = true

theorem agrees_leaf (α : Nat → Bool) : Agrees α .leaf := by
  intro l h
  simp [has, Trie.get?_leaf] at h

theorem agrees_assign {α : Nat → Bool} {a : Assignment} {u : Int} (ha : Agrees α a)
    (hu : litTrue α u = true) : Agrees α (assign a u) := by
  intro l hl
  unfold has assign at hl
  rw [Trie.get?_set] at hl
  by_cases habs : l.natAbs = u.natAbs
  · rw [ite_eq_left habs] at hl
    have hsign : decide (0 < u) = decide (0 < l) := Option.some.inj (of_decide_eq_true hl)
    rcases Int.natAbs_eq_natAbs_iff.mp habs with h | h
    · rw [h]
      exact hu
    · exfalso
      have hu0 := litTrue_ne_zero hu
      subst h
      by_cases hpos : 0 < u
      · have h1 : ¬ 0 < -u := by omega
        rw [decide_eq_true hpos, decide_eq_false h1] at hsign
        exact Bool.noConfusion hsign
      · have h1 : 0 < -u := by omega
        rw [decide_eq_false hpos, decide_eq_true h1] at hsign
        exact Bool.noConfusion hsign
  · rw [ite_eq_right habs] at hl
    exact ha l hl

theorem agrees_refute {α : Nat → Bool} : ∀ (lemma : List Int) (a : Assignment), Agrees α a →
    (∀ m ∈ lemma, m ≠ 0 ∧ litTrue α m = false) →
    Agrees α (lemma.foldl (fun a l => assign a (-l)) a)
  | [], _, ha, _ => ha
  | m :: ms, a, ha, hm => by
    rw [List.foldl_cons]
    apply agrees_refute ms
    · obtain ⟨h0, hf⟩ := hm m (by simp)
      exact agrees_assign ha (litTrue_neg_of_false h0 hf)
    · intro m' h'
      exact hm m' (by simp [h'])

/-- Reverse unit propagation is sound: if every active clause and every
assumed literal is true under `α`, propagation cannot reach a conflict. -/
theorem propagate_sound {α : Nat → Bool} {db : Trie (List Int)}
    (hdb : ∀ k c, db.get? k = some c → clauseTrue α c = true) :
    ∀ (hs : List Nat) (a : Assignment), Agrees α a → propagate db a hs ≠ none
  | [], _, _ => by simp [propagate]
  | h :: hs, a, ha => by
    unfold propagate
    split
    · simp
    · rename_i c hc
      obtain ⟨l, hl, htrue⟩ := List.any_eq_true.mp (hdb h c hc)
      have hopen : has a (-l) = false := by
        cases hneg : has a (-l)
        · rfl
        · have := litTrue_neg (ha _ hneg)
          rw [this] at htrue
          exact absurd htrue (by simp)
      split
      · simp
      · split
        · rename_i hnil
          have hmem : l ∈ c.filter (fun l => !has a (-l)) := List.mem_filter.mpr ⟨hl, by simp [hopen]⟩
          rw [hnil] at hmem
          exact absurd hmem List.not_mem_nil
        · rename_i u hu
          have hmem : l ∈ c.filter (fun l => !has a (-l)) := List.mem_filter.mpr ⟨hl, by simp [hopen]⟩
          rw [hu, List.mem_singleton] at hmem
          subst hmem
          exact propagate_sound hdb hs (assign a l) (agrees_assign ha htrue)
        · simp

theorem lemmaFormOk_nonzero : ∀ (s : Assignment) (lemma : List Int), lemmaFormOk s lemma = true →
    ∀ m ∈ lemma, m ≠ 0
  | _, [], _ => by simp
  | s, l :: ls, h => by
    unfold lemmaFormOk at h
    split at h
    · contradiction
    · rename_i hc
      intro m hm
      rcases List.mem_cons.mp hm with rfl | hm'
      · intro h0
        exact hc (Or.inl h0)
      · exact lemmaFormOk_nonzero _ ls h m hm'

theorem deleteAll_sub : ∀ (ids : List Nat) (db db' : Trie (List Int)), deleteAll db ids = some db' →
    ∀ k c, db'.get? k = some c → db.get? k = some c
  | [], db, db', h => by
    simp only [deleteAll, Option.some.injEq] at h
    subst h
    exact fun _ _ h => h
  | t :: ts, db, db', h => by
    unfold deleteAll at h
    split at h
    · contradiction
    · intro k c hk
      have := deleteAll_sub ts _ _ h k c hk
      rw [Trie.get?_set] at this
      split at this
      · contradiction
      · exact this

/-- The replay invariant under an assignment satisfying the CNF: every active
clause is true, so the empty clause has not been derived. -/
def Replay.Sat (α : Nat → Bool) (st : Replay) : Prop :=
  (∀ k c, st.db.get? k = some c → clauseTrue α c = true) ∧ st.done = false

theorem checkLine_sound {α : Nat → Bool} {vars : Nat} {st st' : Replay} {line : List UInt8}
    (hs : st.Sat α) (h : checkLine vars st line = .ok st') : st'.Sat α := by
  unfold checkLine at h
  split at h
  · contradiction
  · split at h
    · contradiction
    · rename_i db hdb
      cases h
      exact ⟨fun k c hk => hs.1 k c (deleteAll_sub _ _ _ hdb k c hk), hs.2⟩
  · rename_i id lemma hints _
    split at h
    · contradiction
    split at h
    · contradiction
    split at h
    · contradiction
    rename_i hform
    split at h
    · contradiction
    split at h
    · contradiction
    rename_i hprop
    cases h
    have hlemma : clauseTrue α lemma = true := by
      cases hc : clauseTrue α lemma
      · exfalso
        have hform' : lemmaFormOk .leaf lemma = true := by simpa using hform
        have hall : ∀ m ∈ lemma, m ≠ 0 ∧ litTrue α m = false := by
          intro m hm
          refine ⟨lemmaFormOk_nonzero _ _ hform' m hm, ?_⟩
          have := List.any_eq_false.mp hc m hm
          simpa using this
        exact propagate_sound hs.1 _ _ (agrees_refute lemma .leaf (agrees_leaf α) hall) hprop
      · rfl
    refine ⟨?_, ?_⟩
    · intro k c hk
      rw [Trie.get?_set] at hk
      split at hk
      · cases hk
        exact hlemma
      · exact hs.1 k c hk
    · cases lemma with
      | nil => simp [clauseTrue] at hlemma
      | cons => rfl

theorem replay_sound {α : Nat → Bool} {vars : Nat} :
    ∀ (ls : List (List UInt8 × Bool)) (st : Replay) (n : Nat), st.Sat α → replay vars st n ls ≠ .accept
  | [], st, _, hs => by
    simp [replay, hs.2]
  | (line, ended) :: rest, st, n, hs => by
    unfold replay
    rw [ite_eq_right (by simp [hs.2])]
    split
    · simp
    · split
      · simp
      · rename_i st' hst'
        exact replay_sound rest st' (n + 1) (checkLine_sound hs hst')

theorem initDb_go_mem : ∀ (cs : List (List Int)) (i : Nat) (t : Trie (List Int)) (k : Nat)
    (c : List Int), (initDb.go i t cs).get? k = some c → c ∈ cs ∨ t.get? k = some c
  | [], _, _, _, _, h => Or.inr h
  | d :: ds, i, t, k, c, h => by
    rcases initDb_go_mem ds (i + 1) _ k c h with hm | ht
    · exact Or.inl (List.mem_cons_of_mem _ hm)
    · rw [Trie.get?_set] at ht
      split at ht
      · cases ht
        exact Or.inl (by simp)
      · exact Or.inr ht

/-- An accepted certificate refutes the obligation's canonical CNF. -/
theorem verdict_sound {o : Obligation} {f p : String} (h : lrat_verdict o f p = .accept)
    (α : Nat → Bool) (hα : ∀ c ∈ (blast o).2, clauseTrue α c = true) : False := by
  unfold lrat_verdict at h
  split at h
  · contradiction
  · split at h
    · contradiction
    · refine replay_sound (α := α) _ _ 1 ⟨fun k c hk => ?_, rfl⟩ h
      rcases initDb_go_mem _ 1 .leaf k c hk with hm | hl
      · exact hα c hm
      · simp [Trie.get?_leaf] at hl

/-! ## The CNF is satisfiable when the identity fails -/

/-- The input variables for `x` and `y`: bit `i` of `x` is variable `2 + i`,
bit `i` of `y` is variable `34 + i`, and everything else (variable 1) is false. -/
def inputs (x y : Word 32) (v : Nat) : Bool :=
  if 2 ≤ v ∧ v < 34 then x.getLsbD (v - 2) else if 34 ≤ v ∧ v < 66 then y.getLsbD (v - 34) else false

/-- The value of variable `v` when the gates (newest first) are evaluated on `inp`. -/
def value (inp : Nat → Bool) : List Gate → Nat → Bool
  | [], v => inp v
  | g :: gs, v => if v = g.out then g.kind.eval (value inp gs g.a) (value inp gs g.b) else value inp gs v

/-- The blaster's invariant: gates are numbered above the inputs and at most
`top`, read earlier variables, and compute their kind under `value`. -/
def Circuit.Sound (inp : Nat → Bool) (c : Circuit) : Prop :=
  65 ≤ c.top ∧ ∀ g ∈ c.gates, 65 < g.out ∧ g.out ≤ c.top ∧ 1 ≤ g.a ∧ g.a ≤ c.top ∧ 1 ≤ g.b ∧
    g.b ≤ c.top ∧ value inp c.gates g.out = g.kind.eval (value inp c.gates g.a) (value inp c.gates g.b)

/-- Literal `l` of circuit `c` carries bit `b`. -/
def Den (inp : Nat → Bool) (c : Circuit) (l : Nat) (b : Bool) : Prop :=
  1 ≤ l ∧ l ≤ c.top ∧ value inp c.gates l = b

/-- The lists have the same length and are related pointwise. -/
inductive Forall2 {α β : Type} (R : α → β → Prop) : List α → List β → Prop where
  | nil : Forall2 R [] []
  | cons {a : α} {b : β} {l₁ : List α} {l₂ : List β} : R a b → Forall2 R l₁ l₂ → Forall2 R (a :: l₁) (b :: l₂)

@[simp] theorem forall2_nil {α β : Type} {R : α → β → Prop} : Forall2 R [] [] := .nil

/-- `c'` extends `c` without changing the value of any of its variables. -/
def Grows (c c' : Circuit) : Prop :=
  c.top ≤ c'.top ∧ ∀ inp v, v ≤ c.top → value inp c'.gates v = value inp c.gates v

theorem Grows.refl (c : Circuit) : Grows c c := ⟨Nat.le_refl _, fun _ _ _ => rfl⟩

theorem Grows.trans {c₁ c₂ c₃ : Circuit} (h₁ : Grows c₁ c₂) (h₂ : Grows c₂ c₃) : Grows c₁ c₃ :=
  ⟨Nat.le_trans h₁.1 h₂.1, fun inp v hv => (h₂.2 inp v (Nat.le_trans hv h₁.1)).trans (h₁.2 inp v hv)⟩

theorem Den.grow {inp : Nat → Bool} {c c' : Circuit} {l : Nat} {b : Bool} (h : Den inp c l b)
    (g : Grows c c') : Den inp c' l b :=
  ⟨h.1, Nat.le_trans h.2.1 g.1, (g.2 inp l h.2.1).trans h.2.2⟩

theorem Den.growAll {inp : Nat → Bool} {c c' : Circuit} (g : Grows c c') :
    ∀ {L : List Nat} {B : List Bool}, Forall2 (Den inp c) L B → Forall2 (Den inp c') L B
  | _, _, .nil => .nil
  | _, _, .cons h t => .cons (h.grow g) (Den.growAll g t)

theorem value_input {inp : Nat → Bool} : ∀ (gs : List Gate), (∀ g ∈ gs, 65 < g.out) →
    ∀ v, v ≤ 65 → value inp gs v = inp v
  | [], _, _, _ => rfl
  | g :: gs, h, v, hv => by
    have hne : ¬ v = g.out := by
      have := h g (by simp)
      omega
    simp only [value, ite_eq_right hne]
    exact value_input gs (fun g' hg' => h g' (by simp [hg'])) v hv

theorem gate_step {inp : Nat → Bool} {c : Circuit} {k : Kind} {a b : Nat} {va vb : Bool}
    (hs : c.Sound inp) (ha : Den inp c a va) (hb : Den inp c b vb) :
    (c.gate k a b).Sound inp ∧ Grows c (c.gate k a b) ∧
      Den inp (c.gate k a b) (c.gate k a b).top (k.eval va vb) := by
  have htop := hs.1
  have ha2 := ha.2.1
  have hb2 := hb.2.1
  have hold : ∀ v, v ≤ c.top → value inp (c.gate k a b).gates v = value inp c.gates v := by
    intro v hv
    have hne : ¬ v = c.top + 1 := by omega
    simp only [Circuit.gate_gates, value, ite_eq_right hne]
  have hnew : value inp (c.gate k a b).gates (c.top + 1) =
      k.eval (value inp c.gates a) (value inp c.gates b) := by
    simp [value]
  refine ⟨⟨by simp; omega, ?_⟩, ⟨by simp, fun inp' v hv => ?_⟩, ?_⟩
  · intro g hg
    rw [Circuit.gate_gates] at hg
    rcases List.mem_cons.mp hg with rfl | hmem
    · refine ⟨by simp; omega, by simp, ha.1, by simp; omega, hb.1, by simp; omega, ?_⟩
      rw [hnew, hold a ha.2.1, hold b hb.2.1]
    · obtain ⟨h1, h2, h3, h4, h5, h6, h7⟩ := hs.2 g hmem
      refine ⟨h1, by simp; omega, h3, by simp; omega, h5, by simp; omega, ?_⟩
      rw [hold _ h2, hold _ h4, hold _ h6]
      exact h7
  · have hne : ¬ v = c.top + 1 := by omega
    simp only [Circuit.gate_gates, value, ite_eq_right hne]
  · refine ⟨by simp, by simp, ?_⟩
    rw [Circuit.gate_top, hnew, ha.2.2, hb.2.2]

theorem zipGates_spec {inp : Nat → Bool} (k : Kind) :
    ∀ (L R : List Nat) (A B : List Bool) (c : Circuit), c.Sound inp →
      Forall2 (Den inp c) L A → Forall2 (Den inp c) R B →
      (zipGates k L R c).2.Sound inp ∧ Grows c (zipGates k L R c).2 ∧
        Forall2 (Den inp (zipGates k L R c).2) (zipGates k L R c).1 (List.zipWith k.eval A B)
  | [], R, A, B, c, hs, hL, _ => by
    cases hL
    exact ⟨hs, .refl c, by simp [zipGates]⟩
  | a :: L, [], A, B, c, hs, _, hR => by
    cases hR
    exact ⟨hs, .refl c, by simp [zipGates]⟩
  | a :: L, b :: R, A, B, c, hs, hL, hR => by
    cases hL with
    | cons ha hL =>
      cases hR with
      | cons hb hR =>
        obtain ⟨hs1, hg1, hd1⟩ := gate_step (k := k) hs ha hb
        obtain ⟨hs2, hg2, hd2⟩ :=
          zipGates_spec k L R _ _ _ hs1 (Den.growAll hg1 hL) (Den.growAll hg1 hR)
        exact ⟨hs2, hg1.trans hg2, .cons (hd1.grow hg2) hd2⟩

/-- The bits a ripple-carry adder computes from carry `c`. -/
def rippleBits : Bool → List Bool → List Bool → List Bool
  | c, a :: as, b :: bs => ((a ^^ b) ^^ c) :: rippleBits ((a && b) || (c && (a ^^ b))) as bs
  | _, _, _ => []

theorem ripple_spec {inp : Nat → Bool} :
    ∀ (L R : List Nat) (A B : List Bool) (cv : Nat) (cb : Bool) (c : Circuit), c.Sound inp →
      Den inp c cv cb → Forall2 (Den inp c) L A → Forall2 (Den inp c) R B →
      (ripple cv L R c).2.Sound inp ∧ Grows c (ripple cv L R c).2 ∧
        Forall2 (Den inp (ripple cv L R c).2) (ripple cv L R c).1 (rippleBits cb A B)
  | [], R, A, B, _, _, c, hs, _, hL, _ => by
    cases hL
    exact ⟨hs, .refl c, by simp [ripple, rippleBits]⟩
  | a :: L, [], A, B, _, _, c, hs, _, hL, hR => by
    cases hR
    cases hL
    exact ⟨hs, .refl c, by simp [ripple, rippleBits]⟩
  | a :: L, b :: R, A, B, cv, cb, c, hs, hc, hL, hR => by
    cases hL with
    | cons ha hL =>
      cases hR with
      | cons hb hR =>
        rename_i va A vb B
        obtain ⟨s1, g1, d1⟩ := gate_step (k := .xor) hs ha hb
        obtain ⟨s2, g2, d2⟩ := gate_step (k := .xor) s1 d1 (hc.grow g1)
        obtain ⟨s3, g3, d3⟩ := gate_step (k := .and) s2 ((ha.grow g1).grow g2) ((hb.grow g1).grow g2)
        obtain ⟨s4, g4, d4⟩ := gate_step (k := .and) s3 (((hc.grow g1).grow g2).grow g3)
          ((d1.grow g2).grow g3)
        obtain ⟨s5, g5, d5⟩ := gate_step (k := .or) s4 (d3.grow g4) d4
        have g15 := g1.trans (g2.trans (g3.trans (g4.trans g5)))
        obtain ⟨s6, g6, d6⟩ := ripple_spec L R A B _ _ _ s5 d5 (Den.growAll g15 hL) (Den.growAll g15 hR)
        refine ⟨s6, g15.trans g6, ?_⟩
        exact .cons ((((d2.grow g3).grow g4).grow g5).grow g6) d6

theorem orChain_spec {inp : Nat → Bool} :
    ∀ (ds : List Nat) (D : List Bool) (o : Nat) (ob : Bool) (c : Circuit), c.Sound inp →
      Den inp c o ob → Forall2 (Den inp c) ds D →
      (orChain o ds c).2.Sound inp ∧ Grows c (orChain o ds c).2 ∧
        Den inp (orChain o ds c).2 (orChain o ds c).1 (D.foldl (· || ·) ob)
  | [], _, _, _, c, hs, ho, hD => by
    cases hD
    exact ⟨hs, .refl c, ho⟩
  | d :: ds, _, o, ob, c, hs, ho, hD => by
    cases hD with
    | cons hd hD =>
      obtain ⟨s1, g1, d1⟩ := gate_step (k := .or) hs ho hd
      obtain ⟨s2, g2, d2⟩ := orChain_spec ds _ _ _ _ s1 d1 (Den.growAll g1 hD)
      exact ⟨s2, g1.trans g2, d2⟩

/-- A word's 32 bits, least significant first. -/
def bitsOf (v : Word 32) : List Bool := (List.range 32).map v.getLsbD

theorem forall₂_map {α β γ : Type} {R : α → β → Prop} {f : γ → α} {g : γ → β} :
    ∀ (L : List γ), (∀ i ∈ L, R (f i) (g i)) → Forall2 R (L.map f) (L.map g)
  | [], _ => .nil
  | i :: L, h => .cons (h i (by simp)) (forall₂_map L (fun j hj => h j (by simp [hj])))

theorem forall₂_append {α β : Type} {R : α → β → Prop} :
    ∀ {L₁ : List α} {B₁ : List β} {L₂ : List α} {B₂ : List β}, Forall2 R L₁ B₁ →
      Forall2 R L₂ B₂ → Forall2 R (L₁ ++ L₂) (B₁ ++ B₂)
  | _, _, _, _, .nil, h => h
  | _, _, _, _, .cons h t, h₂ => .cons h (forall₂_append t h₂)

theorem forall₂_take {α β : Type} {R : α → β → Prop} :
    ∀ (n : Nat) {L : List α} {B : List β}, Forall2 R L B → Forall2 R (L.take n) (B.take n)
  | 0, _, _, _ => by simp
  | _ + 1, _, _, .nil => by simp
  | n + 1, _, _, .cons h t => by simpa using .cons h (forall₂_take n t)

theorem forall₂_replicate {α β : Type} {R : α → β → Prop} {a : α} {b : β} (h : R a b) :
    ∀ n, Forall2 R (List.replicate n a) (List.replicate n b)
  | 0 => .nil
  | n + 1 => .cons h (forall₂_replicate h n)

theorem zipWith_map_map {α β : Type} (f : β → β → β) (g h : α → β) :
    ∀ (L : List α), List.zipWith f (L.map g) (L.map h) = L.map (fun i => f (g i) (h i))
  | [] => rfl
  | i :: L => by simp [zipWith_map_map f g h L]

theorem bitsOf_xor (v w : Word 32) :
    bitsOf (word_xor v w) = List.zipWith Kind.xor.eval (bitsOf v) (bitsOf w) := by
  simp only [bitsOf, zipWith_map_map]
  congr
  funext i
  simp [word_xor, Kind.eval]

theorem bitsOf_and (v w : Word 32) :
    bitsOf (word_and v w) = List.zipWith Kind.and.eval (bitsOf v) (bitsOf w) := by
  simp only [bitsOf, zipWith_map_map]
  congr
  funext i
  simp [word_and, Kind.eval]

theorem bitsOf_shl (v : Word 32) (k : Nat) :
    bitsOf (word_shl v k) = (List.replicate k false ++ bitsOf v).take 32 := by
  apply List.ext_getElem
  · simp [bitsOf]
  · intro i h₁ h₂
    simp only [bitsOf, List.length_map, List.length_range] at h₁
    simp only [bitsOf, word_shl, List.getElem_map, List.getElem_range, List.getElem_take,
      BitVec.getLsbD_shiftLeft, List.getElem_append, List.length_replicate]
    by_cases hk : i < k
    · simp [hk, h₁]
    · simp [hk, h₁]

/-- The carries of ripple-carry addition. -/
def carryAt (v w : Word 32) : Nat → Bool
  | 0 => false
  | i + 1 => (v.getLsbD i && w.getLsbD i) || (carryAt v w i && (v.getLsbD i ^^ w.getLsbD i))

theorem mod_two_pow_succ' (x i : Nat) :
    x % 2 ^ (i + 1) = 2 ^ i * (x.testBit i).toNat + x % 2 ^ i := by
  rw [Nat.toNat_testBit]
  have hp : 0 < 2 ^ i := Nat.two_pow_pos i
  have hr : x % 2 ^ i < 2 ^ i := Nat.mod_lt _ hp
  have h1 := Nat.div_add_mod x (2 ^ i)
  have h2 := Nat.div_add_mod (x / 2 ^ i) 2
  have hb := Nat.mod_two_eq_zero_or_one (x / 2 ^ i)
  generalize x / 2 ^ i / 2 = a at h2
  generalize x / 2 ^ i % 2 = b at h2 hb ⊢
  generalize x / 2 ^ i = q at h1 h2
  generalize x % 2 ^ i = r at h1 hr ⊢
  subst h2
  subst h1
  have e : 2 ^ i * (2 * a + b) + r = (2 ^ i * b + r) + 2 ^ (i + 1) * a := by
    rw [Nat.mul_add, Nat.pow_succ, ← Nat.mul_assoc, Nat.add_assoc, Nat.add_comm (2 ^ i * 2 * a)]
  rw [e, Nat.add_mul_mod_self_left, Nat.mod_eq_of_lt]
  rw [Nat.pow_succ]
  rcases hb with rfl | rfl <;> omega

/-- Core's `BitVec.carry_succ`, proved without classical reasoning. -/
theorem carry_succ' (i : Nat) (x y : Word 32) :
    BitVec.carry (i + 1) x y false =
      ((x.getLsbD i && y.getLsbD i) || (BitVec.carry i x y false && (x.getLsbD i ^^ y.getLsbD i))) := by
  have hxl : x.toNat % 2 ^ i < 2 ^ i := Nat.mod_lt _ (Nat.two_pow_pos i)
  have hyl : y.toNat % 2 ^ i < 2 ^ i := Nat.mod_lt _ (Nat.two_pow_pos i)
  have hp : 2 ^ (i + 1) = 2 * 2 ^ i := by rw [Nat.pow_succ, Nat.mul_comm]
  unfold BitVec.carry
  rw [mod_two_pow_succ' x.toNat, mod_two_pow_succ' y.toNat, hp, BitVec.testBit_toNat,
    BitVec.testBit_toNat]
  generalize x.toNat % 2 ^ i = xm at *
  generalize y.toNat % 2 ^ i = ym at *
  generalize 2 ^ i = p at *
  by_cases hq : xm + ym + false.toNat ≥ p
  · rw [decide_eq_true hq]
    cases x.getLsbD i <;> cases y.getLsbD i <;>
      simp only [Bool.toNat_false, Bool.toNat_true] at hq ⊢ <;>
      first
        | exact decide_eq_true (by omega)
        | exact decide_eq_false (by omega)
  · rw [decide_eq_false hq]
    cases x.getLsbD i <;> cases y.getLsbD i <;>
      simp only [Bool.toNat_false, Bool.toNat_true] at hq ⊢ <;>
      first
        | exact decide_eq_true (by omega)
        | exact decide_eq_false (by omega)

theorem carry_eq_carryAt (v w : Word 32) : ∀ i, BitVec.carry i v w false = carryAt v w i
  | 0 => by simp [carryAt]
  | i + 1 => by rw [carry_succ', carry_eq_carryAt v w i]; rfl

theorem getLsbD_word_add (v w : Word 32) {i : Nat} (hi : i < 32) :
    (word_add v w).getLsbD i = ((v.getLsbD i ^^ w.getLsbD i) ^^ carryAt v w i) := by
  rw [word_add, BitVec.getLsbD_add hi, carry_eq_carryAt, Bool.xor_assoc]

theorem ripple_range (v w : Word 32) : ∀ n i, i + n = 32 →
    rippleBits (carryAt v w i) ((List.range' i n).map v.getLsbD) ((List.range' i n).map w.getLsbD) =
      (List.range' i n).map (word_add v w).getLsbD
  | 0, _, _ => rfl
  | n + 1, i, h => by
    rw [List.range'_succ, List.map_cons, List.map_cons, List.map_cons,
      ← ripple_range v w n (i + 1) (by omega), getLsbD_word_add v w (by omega)]
    rfl

theorem bitsOf_add (v w : Word 32) :
    bitsOf (word_add v w) = rippleBits false (bitsOf v) (bitsOf w) := by
  simp only [bitsOf, List.range_eq_range']
  exact (ripple_range v w 32 0 rfl).symm

theorem Term.blast_spec (x y : Word 32) :
    ∀ (t : Term) (c : Circuit), c.Sound (inputs x y) →
      (t.blast c).2.Sound (inputs x y) ∧ Grows c (t.blast c).2 ∧
        Forall2 (Den (inputs x y) (t.blast c).2) (t.blast c).1 (bitsOf (t.eval x y))
  | .x, c, hs => by
    simp only [Term.blast, Term.eval, inputBits, bitsOf]
    refine ⟨hs, .refl c, forall₂_map _ fun i hi => ?_⟩
    have hi : i < 32 := List.mem_range.mp hi
    refine ⟨by omega, by have := hs.1; omega, ?_⟩
    rw [value_input _ (fun g hg => (hs.2 g hg).1) _ (by omega)]
    have h1 : 2 ≤ 2 + i ∧ 2 + i < 34 := ⟨by omega, by omega⟩
    unfold inputs
    rw [ite_eq_left h1, Nat.add_sub_cancel_left]
  | .y, c, hs => by
    simp only [Term.blast, Term.eval, inputBits, bitsOf]
    refine ⟨hs, .refl c, forall₂_map _ fun i hi => ?_⟩
    have hi : i < 32 := List.mem_range.mp hi
    refine ⟨by omega, by have := hs.1; omega, ?_⟩
    rw [value_input _ (fun g hg => (hs.2 g hg).1) _ (by omega)]
    have h1 : ¬ (2 ≤ 34 + i ∧ 34 + i < 34) := by omega
    have h2 : 34 ≤ 34 + i ∧ 34 + i < 66 := ⟨by omega, by omega⟩
    unfold inputs
    rw [ite_eq_right h1, ite_eq_left h2, Nat.add_sub_cancel_left]
  | .shl a k, c, hs => by
    obtain ⟨s1, g1, d1⟩ := Term.blast_spec x y a c hs
    refine ⟨s1, g1, ?_⟩
    have hone : Den (inputs x y) (a.blast c).2 1 false := by
      refine ⟨Nat.le_refl _, by have := s1.1; omega, ?_⟩
      rw [value_input _ (fun g hg => (s1.2 g hg).1) _ (by omega)]
      simp [inputs]
    show Forall2 _ ((List.replicate k 1 ++ (a.blast c).1).take 32) (bitsOf (word_shl (a.eval x y) k))
    rw [bitsOf_shl]
    exact forall₂_take 32 (forall₂_append (forall₂_replicate hone k) d1)
  | .xor a b, c, hs => by
    obtain ⟨s1, g1, d1⟩ := Term.blast_spec x y a c hs
    obtain ⟨s2, g2, d2⟩ := Term.blast_spec x y b _ s1
    obtain ⟨s3, g3, d3⟩ := zipGates_spec .xor _ _ _ _ _ s2 (Den.growAll g2 d1) d2
    refine ⟨s3, g1.trans (g2.trans g3), ?_⟩
    show Forall2 _ _ (bitsOf (word_xor (a.eval x y) (b.eval x y)))
    rw [bitsOf_xor]
    exact d3
  | .and a b, c, hs => by
    obtain ⟨s1, g1, d1⟩ := Term.blast_spec x y a c hs
    obtain ⟨s2, g2, d2⟩ := Term.blast_spec x y b _ s1
    obtain ⟨s3, g3, d3⟩ := zipGates_spec .and _ _ _ _ _ s2 (Den.growAll g2 d1) d2
    refine ⟨s3, g1.trans (g2.trans g3), ?_⟩
    show Forall2 _ _ (bitsOf (word_and (a.eval x y) (b.eval x y)))
    rw [bitsOf_and]
    exact d3
  | .add a b, c, hs => by
    obtain ⟨s1, g1, d1⟩ := Term.blast_spec x y a c hs
    obtain ⟨s2, g2, d2⟩ := Term.blast_spec x y b _ s1
    have hone : Den (inputs x y) (b.blast (a.blast c).2).2 1 false := by
      refine ⟨Nat.le_refl _, by have := s2.1; omega, ?_⟩
      rw [value_input _ (fun g hg => (s2.2 g hg).1) _ (by omega)]
      simp [inputs]
    obtain ⟨s3, g3, d3⟩ := ripple_spec _ _ _ _ _ _ _ s2 hone (Den.growAll g2 d1) d2
    refine ⟨s3, g1.trans (g2.trans g3), ?_⟩
    show Forall2 _ _ (bitsOf (word_add (a.eval x y) (b.eval x y)))
    rw [bitsOf_add]
    exact d3

theorem foldl_or (b : Bool) : ∀ (ds : List Bool), ds.foldl (· || ·) b = (b || ds.any id)
  | [] => by simp
  | d :: ds => by
    rw [List.foldl_cons, foldl_or (b || d) ds]
    cases b <;> cases d <;> simp

theorem miter_spec (x y : Word 32) {L R : List Nat} {A B : List Bool} {c : Circuit}
    (hs : c.Sound (inputs x y)) (hL : Forall2 (Den (inputs x y) c) L A)
    (hR : Forall2 (Den (inputs x y) c) R B) :
    (miter L R c).2.Sound (inputs x y) ∧
      Den (inputs x y) (miter L R c).2 (miter L R c).1 ((List.zipWith Kind.xor.eval A B).any id) := by
  obtain ⟨s1, _, d1⟩ := zipGates_spec .xor L R A B c hs hL hR
  simp only [miter]
  split
  · rename_i hnil
    rw [hnil] at d1
    generalize hz : List.zipWith Kind.xor.eval A B = Z at d1
    cases d1
    refine ⟨s1, Nat.le_refl _, ?_, ?_⟩
    · dsimp only
      have := s1.1
      omega
    · dsimp only
      rw [value_input _ (fun g hg => (s1.2 g hg).1) _ (by omega)]
      simp [inputs]
  · rename_i d₀ ds hcons
    rw [hcons] at d1
    generalize hz : List.zipWith Kind.xor.eval A B = Z at d1
    cases d1 with
    | cons h0 hds =>
      obtain ⟨s2, _, d2⟩ := orChain_spec ds _ d₀ _ _ s1 h0 hds
      refine ⟨s2, ?_⟩
      rw [foldl_or] at d2
      simpa using d2

theorem bitsOf_inj {v w : Word 32} (h : bitsOf v = bitsOf w) : v = w := by
  apply BitVec.eq_of_getLsbD_eq
  intro i hi
  have := congrArg (fun l => l[i]?) h
  simpa [bitsOf, hi] using this

theorem any_xor_of_ne : ∀ {A B : List Bool}, A.length = B.length → A ≠ B →
    (List.zipWith Kind.xor.eval A B).any id = true
  | [], [], _, h => absurd rfl h
  | [], _ :: _, hl, _ => nomatch hl
  | _ :: _, [], hl, _ => nomatch hl
  | a :: A, b :: B, hl, h => by
    simp only [List.zipWith_cons_cons, List.any_cons]
    have hl' : A.length = B.length := by
      rw [List.length_cons, List.length_cons] at hl
      exact Nat.succ.inj hl
    cases a <;> cases b
    · have : A ≠ B := fun e => h (by rw [e])
      rw [any_xor_of_ne hl' this]
      rfl
    · rfl
    · rfl
    · have : A ≠ B := fun e => h (by rw [e])
      rw [any_xor_of_ne hl' this]
      rfl

theorem gate_clauses_true (α : Nat → Bool) (g : Gate) (ho : 1 ≤ g.out) (ha : 1 ≤ g.a) (hb : 1 ≤ g.b)
    (h : α g.out = g.kind.eval (α g.a) (α g.b)) : ∀ c ∈ g.clauses, clauseTrue α c = true := by
  obtain ⟨o, k, a, b⟩ := g
  simp only at ho ha hb h
  cases k <;>
  · simp only [Gate.clauses, List.forall_mem_cons, clauseTrue,
      List.any_cons, List.any_nil, Bool.or_false, litTrue_ofNat ho, litTrue_ofNat ha,
      litTrue_ofNat hb, litTrue_negOfNat ho, litTrue_negOfNat ha, litTrue_negOfNat hb]
    rw [h]
    cases α a <;> cases α b <;> simp [Kind.eval]

/-- When the identity fails for `x` and `y`, evaluating the circuit on them
satisfies every clause of the canonical CNF. -/
theorem blast_satisfiable (o : Obligation) (x y : Word 32) (hne : o.lhs.eval x y ≠ o.rhs.eval x y) :
    ∃ α : Nat → Bool, ∀ c ∈ (blast o).2, clauseTrue α c = true := by
  have h0 : Circuit.Sound (inputs x y) ⟨65, []⟩ := ⟨Nat.le_refl _, by simp⟩
  obtain ⟨s1, _, d1⟩ := Term.blast_spec x y o.lhs ⟨65, []⟩ h0
  obtain ⟨s2, g2, d2⟩ := Term.blast_spec x y o.rhs _ s1
  obtain ⟨s3, d3⟩ := miter_spec x y s2 (Den.growAll g2 d1) d2
  generalize hm : miter (o.lhs.blast ⟨65, []⟩).1 (o.rhs.blast (o.lhs.blast ⟨65, []⟩).2).1
    (o.rhs.blast (o.lhs.blast ⟨65, []⟩).2).2 = m at s3 d3
  have hlen : (bitsOf (o.lhs.eval x y)).length = (bitsOf (o.rhs.eval x y)).length := by simp [bitsOf]
  have hbits : bitsOf (o.lhs.eval x y) ≠ bitsOf (o.rhs.eval x y) := fun e => hne (bitsOf_inj e)
  rw [any_xor_of_ne hlen hbits] at d3
  refine ⟨value (inputs x y) m.2.gates, fun c hc => ?_⟩
  simp only [blast, hm, List.mem_cons, List.mem_append, List.mem_flatMap, List.mem_reverse,
    List.not_mem_nil, or_false] at hc
  rcases hc with rfl | ⟨g, hg, hcg⟩ | rfl
  · have : litTrue (value (inputs x y) m.2.gates) (-((1 : Nat) : Int)) = true := by
      rw [litTrue_negOfNat (Nat.le_refl _), value_input _ (fun g hg => (s3.2 g hg).1) _ (by omega)]
      simp [inputs]
    simpa [clauseTrue] using this
  · obtain ⟨h1, _, h3, _, h5, _, h7⟩ := s3.2 g hg
    exact gate_clauses_true _ g (by omega) h3 h5 h7 c hcg
  · simp only [clauseTrue, List.any_cons, List.any_nil, Bool.or_false, litTrue_ofNat d3.1]
    exact d3.2.2

/-! ## The checker's guarantee and the carry-save identity -/

/-- D4-TH04: an accepted certificate proves its obligation, for every
obligation, claimed CNF text and certificate. -/
theorem accepted_certificate_proves :
    ∀ (o : Obligation) (f p : String), lrat_verdict o f p = .accept → holds o := by
  intro o f p h x y
  by_cases he : o.lhs.eval x y = o.rhs.eval x y
  · exact he
  · obtain ⟨α, hα⟩ := blast_satisfiable o x y he
    exact (verdict_sound h α hα).elim

/-- The golden certificate of the shared packet (`ds04-carry-save-golden.lrat`,
61850 bytes, sha256 `9b75f94c...0bc1276`), one certificate line per source
line: each line ends in `\n` and a string gap. -/
def golden_certificate : String :=
  "1535 d 7 9 14 15 17 537 538 539 540 541 542 543 544 545 767 768 769 771 773 775 777 778 780 782 783 784 785 792 794 799 800 802 1305 1306 1307 1308 1309 1310 1311 1312 1313 1316 1317 1320 1321 1325 1344 1361 1376 1377 1380 1381 1384 1389 1392 1393 1397 1400 1401 1405 1408 1409 1412 1413 1416 1417 1420 1421 1424 1425 1432 1436 1437 1442 1443 1445 1446 1448 1449 1451 1452 1454 1455 1457 1458 1460 1461 1463 1464 1466 1467 1469 1470 1472 1473 1475 1476 1478 1479 1481 1482 1484 1485 1487 1488 1490 1494 1500 1502 1503 1505 1508 1509 1511 1512 1517 1518 1520 1521 1523 1529 1530 1532 1533 0\n\
  1536 -67 66 0 1 6 0\n\
  1536 d 6 0\n\
  1537 67 -66 0 1 8 0\n\
  1537 d 8 0\n\
  1539 -70 68 0 1 13 18 0\n\
  1539 d 13 18 0\n\
  1540 -290 226 0 1 770 0\n\
  1540 d 770 0\n\
  1541 290 -226 0 1 772 0\n\
  1541 d 772 0\n\
  1542 -291 290 0 1 774 0\n\
  1542 d 774 0\n\
  1543 291 -290 0 1 776 0\n\
  1543 d 776 0\n\
  1547 -296 295 0 1 781 779 786 791 0\n\
  1547 d 791 0\n\
  1548 296 -295 0 1 781 779 786 793 0\n\
  1548 d 793 0\n\
  1550 -299 297 0 1 781 779 786 798 803 0\n\
  1550 d 781 779 786 798 803 0\n\
  1552 256 -444 0 1288 1296 1291 1292 1280 761 762 663 1271 1279 1274 1275 1263 758 759 659 1254 1262 1257 1258 1246 755 756 655 1237 1245 1240 1241 1229 752 753 651 1220 1228 1223 1224 1212 749 750 647 1203 1211 1206 1207 1195 746 747 643 1186 1194 1189 1190 1178 743 744 639 1169 1177 1172 1173 1161 740 741 635 1152 1160 1155 1156 1144 737 738 631 1135 1143 1138 1139 1127 734 735 627 1118 1126 1121 1122 1110 731 732 623 1101 1109 1104 1105 1093 728 729 619 1084 1092 1087 1088 1076 725 726 615 1067 1075 1070 1071 1059 722 723 611 1050 1058 1053 1054 1042 719 720 607 1033 1041 1036 1037 1025 716 717 603 1016 1024 1019 1020 1008 713 714 599 999 1007 1002 1003 991 710 711 595 982 990 985 986 974 707 708 591 965 973 968 969 957 704 705 587 948 956 951 952 940 701 702 583 931 939 934 935 923 698 699 579 914 922 917 918 906 695 696 575 897 905 900 901 889 692 693 571 880 888 883 884 872 689 690 567 863 871 866 867 855 686 687 563 846 854 849 850 838 683 684 559 829 837 832 833 821 680 681 555 812 820 815 816 1550 804 795 677 678 551 0\n\
  1552 d 1288 1292 0\n\
  1553 -258 -450 0 674 675 547 1540 1542 1314 1536 3 0\n\
  1553 d 547 3 0\n\
  1554 -68 258 0 10 11 676 0\n\
  1554 d 10 11 676 0\n\
  1555 -34 -450 0 1553 1554 12 5 1537 1315 1543 1541 549 0\n\
  1555 d 1553 5 549 0\n\
  1556 2 -450 0 1555 2 546 1536 1540 1314 1542 0\n\
  1556 d 2 546 1536 1540 1314 1542 0\n\
  1557 -450 0 1556 1555 4 548 1537 1541 1315 1543 0\n\
  1557 d 1556 1555 4 548 1537 1541 1315 1543 0\n\
  1559 -73 259 0 27 28 679 0\n\
  1559 d 27 28 679 0\n\
  1563 258 -71 -227 -451 0 1554 1539 789 1548 1319 25 0\n\
  1563 d 789 25 0\n\
  1564 -3 35 -451 0 21 552 1563 788 674 675 1547 12 1318 16 24 0\n\
  1565 -258 35 -451 0 674 675 1564 12 19 550 16 790 26 1548 1319 0\n\
  1566 35 -451 0 1565 1554 1539 1564 19 550 23 787 1318 1547 0\n\
  1566 d 1565 1564 0\n\
  1567 227 258 -451 0 1566 1554 553 1539 20 787 1547 1318 23 0\n\
  1567 d 787 23 0\n\
  1568 258 -451 0 1566 1567 551 22 1563 0\n\
  1568 d 1567 1563 0\n\
  1569 -227 -451 0 1568 1566 674 675 551 12 22 16 788 24 1547 1318 0\n\
  1569 d 788 24 1547 1318 0\n\
  1570 -451 0 1569 1568 674 675 12 16 1566 553 20 790 26 1548 1319 0\n\
  1570 d 1569 1568 1566 790 26 1548 1319 0\n\
  1572 -260 -76 0 680 681 37 0\n\
  1572 d 37 0\n\
  1573 -36 -76 228 0 1572 682 557 0\n\
  1573 d 557 0\n\
  1574 -76 228 0 1573 36 556 0\n\
  1574 d 1573 36 556 0\n\
  1575 228 -75 -259 299 -452 0 1574 807 43 810 1323 0\n\
  1575 d 807 0\n\
  1576 36 76 -228 0 38 554 0\n\
  1576 d 38 554 0\n\
  1577 76 -228 0 1576 555 39 0\n\
  1577 d 1576 555 39 0\n\
  1578 -75 -259 299 -452 0 1575 1577 41 1322 808 805 0\n\
  1578 d 1575 805 0\n\
  1579 227 -35 -452 0 795 553 1550 29 33 1559 1578 0\n\
  1579 d 1578 0\n\
  1580 76 -75 259 -299 -452 0 1577 43 804 1323 811 0\n\
  1580 d 811 0\n\
  1581 -75 259 -299 -452 0 1580 1574 806 809 1322 41 0\n\
  1581 d 1580 41 0\n\
  1582 -258 -35 -452 0 674 675 1579 12 797 551 16 801 677 1581 34 32 22 0\n\
  1582 d 22 0\n\
  1583 -76 -35 -452 0 1574 1582 1579 796 1554 551 1550 1539 677 30 1559 806 35 810 42 1323 0\n\
  1584 -35 -452 0 1583 1577 1582 796 1554 1550 1539 30 1579 551 677 1559 804 35 808 40 1322 0\n\
  1584 d 1583 1582 1579 551 0\n\
  1585 76 75 299 -452 0 1577 1584 678 804 40 808 1322 0\n\
  1585 d 40 808 1322 0\n\
  1586 299 75 -452 0 1584 678 1585 1574 42 806 1323 810 0\n\
  1586 d 1585 1323 810 0\n\
  1587 75 -452 0 1584 34 1586 1550 795 796 550 674 675 21 12 32 16 0\n\
  1587 d 34 1586 1550 795 796 550 674 675 21 12 32 16 0\n\
  1588 -452 0 1587 1584 678 1559 35 30 31 1539 19 1554 552 797 801 1581 0\n\
  1588 d 1587 1584 1581 0\n\
  1590 35 -75 297 0 678 1559 35 30 31 1539 1554 797 552 19 0\n\
  1590 d 552 19 0\n\
  1591 -74 -35 297 0 30 31 1539 20 1554 553 797 0\n\
  1591 d 30 31 1539 20 1554 553 797 0\n\
  1593 -37 81 -229 0 56 559 0\n\
  1593 d 559 0\n\
  1594 81 -229 0 1593 55 558 0\n\
  1594 d 1593 55 558 0\n\
  1595 -305 -80 260 -304 -453 0 826 821 1326 1594 58 0\n\
  1596 37 -81 229 0 53 560 0\n\
  1596 d 53 560 0\n\
  1597 -81 229 0 1596 54 561 0\n\
  1597 d 1596 561 0\n\
  1598 -304 -80 260 -453 0 1595 823 1597 60 1327 828 0\n\
  1598 d 1595 828 0\n\
  1599 -76 -75 -259 -453 0 1574 1572 49 51 1598 814 818 0\n\
  1599 d 814 818 0\n\
  1600 -78 260 0 44 45 682 0\n\
  1600 d 44 45 682 0\n\
  1602 229 80 260 304 -453 0 1597 821 825 1326 57 0\n\
  1602 d 57 0\n\
  1603 80 260 304 -453 0 1602 1594 823 59 827 1327 0\n\
  1603 d 1602 0\n\
  1604 260 79 304 -453 0 1600 1603 52 0\n\
  1604 d 1600 1603 52 0\n\
  1605 -229 -260 304 -453 0 1594 680 681 46 50 58 822 1326 825 0\n\
  1605 d 822 1326 825 0\n\
  1606 304 79 -453 0 1604 680 681 46 50 1605 1597 60 824 827 1327 0\n\
  1606 d 1604 1605 824 1327 0\n\
  1607 299 -75 -453 0 815 801 1590 1591 35 1559 1599 48 1577 1606 812 820 0\n\
  1607 d 801 1590 1591 35 1559 0\n\
  1608 304 -75 -453 0 1606 1607 48 819 1574 1599 817 806 0\n\
  1608 d 819 1574 1599 817 806 0\n\
  1609 -79 -304 -453 0 48 51 1572 1598 0\n\
  1609 d 1598 0\n\
  1610 -75 -453 0 1588 1607 1608 1609 49 1577 43 1324 809 816 820 812 0\n\
  1610 d 1607 1608 1609 49 43 812 0\n\
  1611 -259 73 0 677 678 29 0\n\
  1611 d 677 678 29 0\n\
  1612 -453 0 1588 1610 33 47 1611 1606 813 820 815 816 804 809 1577 1324 42 0\n\
  1612 d 1610 33 47 1611 1606 813 820 815 816 804 809 1577 1324 42 0\n\
  1614 -262 -86 0 686 687 71 0\n\
  1614 d 71 0\n\
  1615 -38 -86 230 0 1614 688 565 0\n\
  1615 d 565 0\n\
  1616 -86 230 0 1615 70 564 0\n\
  1616 d 1615 70 564 0\n\
  1617 230 -85 -261 309 -454 0 1616 841 77 844 1331 0\n\
  1617 d 841 0\n\
  1618 38 86 -230 0 72 562 0\n\
  1618 d 72 562 0\n\
  1619 86 -230 0 1618 73 563 0\n\
  1619 d 1618 563 0\n\
  1620 -85 -261 309 -454 0 1617 1619 839 75 842 1330 0\n\
  1620 d 1617 0\n\
  1621 -37 81 309 -454 0 56 63 685 67 1620 0\n\
  1621 d 56 63 685 67 1620 0\n\
  1622 86 85 261 309 -454 0 1619 838 842 1330 74 0\n\
  1622 d 74 0\n\
  1623 85 261 309 -454 0 1622 1616 76 840 1331 844 0\n\
  1623 d 1622 0\n\
  1624 37 84 309 -454 0 684 1623 69 62 0\n\
  1624 d 62 0\n\
  1625 309 84 -454 0 1624 1621 54 683 61 1623 69 0\n\
  1625 d 1624 1621 61 1623 69 0\n\
  1627 309 79 -454 0 1612 836 1625 65 1597 835 64 58 1329 831 823 827 834 0\n\
  1627 d 835 64 831 0\n\
  1631 -261 -309 0 1612 683 684 54 1594 829 837 832 833 821 826 680 681 1328 46 60 50 0\n\
  1631 d 683 684 54 0\n\
  1632 229 -308 0 1612 1597 832 833 826 1328 60 50 821 680 681 46 0\n\
  1632 d 60 821 0\n\
  1633 86 -85 -309 -454 0 1619 1631 838 77 845 1331 0\n\
  1633 d 77 845 1331 0\n\
  1634 -85 -309 -454 0 1631 1633 1616 75 840 843 1330 0\n\
  1634 d 1633 1330 0\n\
  1635 -308 84 0 1612 832 833 1632 1594 66 826 59 1328 0\n\
  1635 d 832 833 1632 826 59 1328 0\n\
  1636 -309 -454 0 1634 68 1635 837 829 830 1594 680 681 66 46 50 0\n\
  1636 d 1634 68 1635 837 829 830 1594 680 681 66 46 50 0\n\
  1637 -454 0 1612 1636 1627 1625 836 48 51 65 1572 1597 58 823 1329 827 834 0\n\
  1637 d 1636 1627 1625 836 48 51 65 1572 1597 58 823 1329 827 834 0\n\
  1641 -39 91 -231 0 90 567 0\n\
  1641 d 567 0\n\
  1642 91 -231 0 1641 89 566 0\n\
  1642 d 1641 89 566 0\n\
  1643 -315 -90 262 -314 -455 0 860 855 1334 1642 92 0\n\
  1644 39 -91 231 0 87 568 0\n\
  1644 d 87 568 0\n\
  1645 -91 231 0 1644 88 569 0\n\
  1645 d 1644 569 0\n\
  1646 -314 -90 262 -455 0 1643 857 862 1645 1335 94 0\n\
  1646 d 1643 0\n\
  1648 -88 262 0 78 79 688 0\n\
  1648 d 78 79 688 0\n\
  1649 -38 86 88 0 80 73 0\n\
  1649 d 73 0\n\
  1650 231 90 262 314 -455 0 1645 855 859 1334 91 0\n\
  1650 d 91 0\n\
  1651 90 262 314 -455 0 1650 1642 857 93 861 1335 0\n\
  1651 d 1650 0\n\
  1652 262 89 314 -455 0 1648 1651 86 0\n\
  1652 d 1648 1651 86 0\n\
  1653 -231 -262 314 -455 0 1642 686 687 80 84 92 856 1334 859 0\n\
  1653 d 856 1334 859 0\n\
  1654 314 89 -455 0 1652 686 687 80 84 1653 1645 94 858 861 1335 0\n\
  1654 d 1652 1653 858 1335 0\n\
  1655 86 313 -455 0 82 1619 1654 854 846 0\n\
  1655 d 846 0\n\
  1656 314 309 -455 0 1637 853 1654 1655 1616 852 81 848 840 75 844 1333 0\n\
  1656 d 852 81 848 75 844 1333 0\n\
  1657 309 -455 0 1637 849 1656 854 847 1655 1616 1614 1646 85 83 76 1332 839 842 0\n\
  1657 d 849 1656 1655 839 842 0\n\
  1658 -314 -455 0 1637 1657 1631 847 854 850 838 843 1619 1614 1332 76 1646 85 83 0\n\
  1658 d 847 854 850 838 843 1619 1332 76 1646 83 0\n\
  1659 -455 0 1658 1657 1654 82 853 1631 851 1616 840 0\n\
  1659 d 1658 1657 1654 853 1631 851 1616 840 0\n\
  1661 -264 -96 0 692 693 105 0\n\
  1661 d 105 0\n\
  1662 -40 -96 232 0 1661 694 573 0\n\
  1662 d 573 0\n\
  1663 -96 232 0 1662 104 572 0\n\
  1663 d 1662 104 572 0\n\
  1664 232 -95 -263 319 -456 0 1663 875 111 878 1339 0\n\
  1664 d 875 0\n\
  1665 40 96 -232 0 106 570 0\n\
  1665 d 106 570 0\n\
  1666 96 -232 0 1665 571 107 0\n\
  1666 d 1665 107 0\n\
  1667 -95 -263 319 -456 0 1664 1666 109 1338 876 873 0\n\
  1667 d 1664 873 0\n\
  1668 -39 91 319 -456 0 90 691 97 101 1667 0\n\
  1668 d 90 1667 0\n\
  1669 96 95 263 319 -456 0 1666 872 108 1338 876 0\n\
  1669 d 108 876 0\n\
  1670 95 263 319 -456 0 1669 1663 110 874 1339 878 0\n\
  1670 d 1669 0\n\
  1671 39 94 319 -456 0 96 690 103 1670 0\n\
  1672 94 319 -456 0 1671 1668 88 95 689 103 1670 0\n\
  1672 d 1671 1668 1670 0\n\
  1673 -86 314 -456 0 1659 866 1614 864 871 1672 98 99 92 1645 1337 857 861 0\n\
  1673 d 1614 0\n\
  1674 319 89 -456 0 1659 870 1672 99 1645 869 98 92 1337 865 857 868 861 0\n\
  1674 d 869 98 865 861 0\n\
  1675 -96 263 314 -456 0 1663 1673 866 874 82 1674 871 877 863 864 1338 1642 686 687 109 80 102 84 100 0\n\
  1676 263 314 -456 0 866 1673 1675 1666 872 82 1674 871 879 863 1339 1642 111 102 100 84 1649 687 864 0\n\
  1676 d 1673 1675 82 1649 864 0\n\
  1677 314 -456 0 866 1676 689 690 88 1642 99 863 1672 871 0\n\
  1677 d 866 1676 0\n\
  1678 -263 -314 -319 0 1659 689 690 88 1642 863 871 867 855 860 686 687 1336 80 94 84 0\n\
  1679 231 92 -315 0 1645 94 84 855 686 687 80 0\n\
  1679 d 94 855 0\n\
  1680 96 -95 -319 -456 0 1666 111 1677 1678 872 1339 879 0\n\
  1680 d 1339 879 0\n\
  1681 -95 -319 -456 0 1677 1678 1680 109 1663 874 877 1338 0\n\
  1681 d 1678 1680 1338 0\n\
  1682 -231 94 -314 0 1659 1642 100 84 93 1336 860 857 686 687 80 0\n\
  1682 d 100 84 93 857 686 687 80 0\n\
  1683 -319 -456 0 1659 1677 1681 102 1682 863 871 867 1679 860 1336 0\n\
  1683 d 1681 102 1682 863 871 867 1679 860 1336 0\n\
  1684 -456 0 1659 1683 1677 1674 1672 870 868 85 99 862 92 1337 0\n\
  1684 d 1683 1677 1674 1672 870 868 85 862 92 1337 0\n\
  1687 -41 101 -233 0 124 575 0\n\
  1687 d 575 0\n\
  1688 101 -233 0 1687 123 574 0\n\
  1688 d 1687 123 574 0\n\
  1689 -325 -100 264 -324 -457 0 894 889 1342 1688 126 0\n\
  1689 d 894 0\n\
  1690 41 -101 233 0 121 576 0\n\
  1690 d 121 576 0\n\
  1691 -101 233 0 1690 122 577 0\n\
  1691 d 1690 577 0\n\
  1692 -324 -100 264 -457 0 1689 891 1691 128 1343 896 0\n\
  1692 d 1689 896 0\n\
  1693 -96 -95 263 -319 -457 0 1663 1661 117 874 119 1692 885 887 0\n\
  1693 d 885 887 0\n\
  1694 -98 264 0 112 113 694 0\n\
  1694 d 112 113 694 0\n\
  1696 233 100 264 324 -457 0 1691 889 893 1342 125 0\n\
  1696 d 125 0\n\
  1697 100 264 324 -457 0 1696 1688 891 895 1343 127 0\n\
  1697 d 1696 127 0\n\
  1698 264 99 324 -457 0 1694 120 1697 0\n\
  1698 d 1697 0\n\
  1699 -233 -264 324 -457 0 1688 692 693 114 118 126 890 1342 893 0\n\
  1699 d 890 1342 893 0\n\
  1700 324 99 -457 0 1698 692 693 114 118 1699 1691 892 895 128 1343 0\n\
  1700 d 1698 1699 892 128 1343 0\n\
  1701 263 -95 -319 -457 0 881 1693 1666 116 872 1700 884 888 0\n\
  1701 d 1693 0\n\
  1702 -263 -231 0 689 690 1642 88 0\n\
  1702 d 1642 88 0\n\
  1703 -94 -95 -457 0 1684 99 1645 1702 881 1701 883 888 1700 116 1663 874 109 878 1341 0\n\
  1703 d 99 1645 1702 1701 109 878 1341 0\n\
  1704 324 -263 -457 0 1700 116 1663 882 886 0\n\
  1704 d 882 886 0\n\
  1705 -96 -95 -457 0 117 1661 119 1703 1692 103 1704 95 96 691 0\n\
  1705 d 1703 1692 103 1704 95 96 691 0\n\
  1706 -95 -457 0 1684 1705 1666 116 111 880 1700 1340 888 883 884 877 0\n\
  1706 d 1705 0\n\
  1707 -263 93 0 689 690 97 0\n\
  1707 d 689 690 97 0\n\
  1708 -457 0 1684 1706 101 115 1707 881 1700 888 883 884 872 877 1666 1340 110 0\n\
  1708 d 1706 1700 0\n\
  1710 -266 -106 0 698 699 139 0\n\
  1711 -42 -106 234 0 139 581 0\n\
  1711 d 581 0\n\
  1712 -106 234 0 1711 138 580 0\n\
  1712 d 1711 138 580 0\n\
  1713 234 -105 -265 329 -458 0 909 1712 145 912 1347 0\n\
  1713 d 909 0\n\
  1714 42 106 -234 0 140 578 0\n\
  1714 d 140 578 0\n\
  1715 106 -234 0 1714 141 579 0\n\
  1715 d 1714 579 0\n\
  1716 -105 -265 329 -458 0 1713 1715 143 1346 910 907 0\n\
  1716 d 1713 907 0\n\
  1717 -41 101 329 -458 0 124 131 135 697 1716 0\n\
  1717 d 124 697 1716 0\n\
  1718 106 105 265 329 -458 0 1715 906 910 1346 142 0\n\
  1718 d 142 0\n\
  1719 105 265 329 -458 0 1718 1712 144 908 1347 912 0\n\
  1719 d 1718 0\n\
  1720 41 104 329 -458 0 696 1719 137 130 0\n\
  1720 d 130 0\n\
  1721 329 104 -458 0 1720 1717 122 695 129 1719 137 0\n\
  1721 d 1720 1717 129 1719 137 0\n\
  1722 95 -232 -458 0 1684 101 1666 1707 1661 1694 898 115 120 132 1721 905 900 110 1340 874 877 883 888 881 0\n\
  1722 d 115 110 874 881 0\n\
  1723 329 -232 -458 0 1708 1721 1666 132 133 1661 1691 904 126 891 1345 902 895 0\n\
  1723 d 904 126 891 1345 902 895 0\n\
  1724 -106 -232 265 -458 0 1712 1666 1723 1722 908 1661 117 911 898 119 1346 905 143 901 136 889 134 1688 0\n\
  1725 265 -232 -458 0 1666 1661 898 1723 905 901 889 1688 1722 1724 1715 906 913 1347 145 136 134 117 119 0\n\
  1725 d 1723 1722 1724 117 119 0\n\
  1726 -232 -458 0 1666 1661 898 1725 695 696 122 1688 133 889 1721 901 905 0\n\
  1726 d 1666 1725 0\n\
  1727 -265 -329 0 1684 695 696 122 1688 897 905 900 901 889 692 693 1661 571 880 888 883 884 872 877 1707 1340 101 111 0\n\
  1727 d 122 901 889 1661 571 0\n\
  1728 -323 232 0 1684 1663 883 884 872 877 1707 1340 101 111 0\n\
  1728 d 883 884 872 877 1707 1340 101 111 0\n\
  1729 106 -329 -458 0 1715 1726 1727 906 1728 880 888 900 905 897 898 1688 692 693 114 118 134 136 145 913 1347 0\n\
  1729 d 913 1347 0\n\
  1730 -329 -458 0 1727 1726 1729 1712 908 911 1728 880 1346 143 888 136 900 905 897 898 1688 692 693 134 114 118 0\n\
  1730 d 1729 1728 880 1346 888 136 900 905 897 898 1688 692 693 134 114 118 0\n\
  1731 -458 0 1730 1726 1721 903 1663 132 133 116 120 1691 1694 899 0\n\
  1731 d 1730 1726 1721 903 1663 132 133 116 120 1691 1694 899 0\n\
  1733 -106 -105 265 329 0 1731 1712 908 143 912 1349 0\n\
  1733 d 143 912 1349 0\n\
  1734 -105 265 329 0 1731 1733 1715 906 145 1348 910 0\n\
  1734 d 1733 145 910 0\n\
  1735 -267 -111 0 701 702 156 0\n\
  1735 d 156 0\n\
  1736 -43 -111 235 0 585 1735 703 0\n\
  1737 -111 235 0 1736 155 584 0\n\
  1737 d 1736 155 584 0\n\
  1738 235 -110 -266 334 -459 0 1737 926 162 929 1351 0\n\
  1738 d 926 0\n\
  1739 43 111 -235 0 157 582 0\n\
  1739 d 157 582 0\n\
  1740 111 -235 0 1739 583 158 0\n\
  1740 d 1739 583 158 0\n\
  1741 -110 -266 334 -459 0 1738 1740 924 160 927 1350 0\n\
  1741 d 1738 0\n\
  1742 -42 106 334 -459 0 141 148 152 700 1741 0\n\
  1742 d 141 700 1741 0\n\
  1743 111 110 266 334 -459 0 1740 923 927 1350 159 0\n\
  1743 d 159 0\n\
  1744 110 266 334 -459 0 1743 1737 161 925 1351 929 0\n\
  1744 d 1743 0\n\
  1745 42 109 334 -459 0 699 1744 154 147 0\n\
  1745 d 147 0\n\
  1746 109 334 -459 0 1745 1742 1710 139 1744 146 154 0\n\
  1746 d 1745 1742 139 1744 146 154 0\n\
  1747 265 329 -459 0 915 917 922 1746 1734 149 0\n\
  1747 d 1734 149 0\n\
  1748 334 -265 -459 0 1746 150 1712 920 916 0\n\
  1748 d 920 916 0\n\
  1749 -111 329 -459 0 1737 1747 917 695 696 1748 131 922 135 914 1715 1710 151 925 153 928 160 1350 0\n\
  1750 329 -459 0 917 1749 1740 1747 1748 922 695 696 914 1715 1710 923 930 1351 162 153 151 131 135 0\n\
  1750 d 917 1749 1747 695 696 914 131 135 0\n\
  1751 111 -334 -459 0 1731 1740 1750 1727 915 922 918 906 911 1715 1348 1710 144 923 151 153 162 930 1351 0\n\
  1751 d 930 1351 0\n\
  1752 -334 -459 0 1731 1750 1751 1737 1727 915 922 918 906 911 1715 1348 1710 925 928 144 1350 160 151 153 0\n\
  1752 d 1751 1727 915 922 918 906 911 1715 1348 1710 144 1350 151 153 0\n\
  1753 -459 0 1752 1750 1748 1746 921 919 908 150 1712 0\n\
  1753 d 1752 1750 1748 1746 921 919 908 150 1712 0\n\
  1755 -111 -110 266 334 0 1753 1737 925 160 929 1353 0\n\
  1755 d 160 929 1353 0\n\
  1756 266 -110 334 0 1753 1755 1740 162 923 1352 927 0\n\
  1756 d 1755 0\n\
  1757 -268 -116 0 704 705 173 0\n\
  1757 d 173 0\n\
  1758 -44 -116 236 0 589 1757 706 0\n\
  1759 -116 236 0 1758 172 588 0\n\
  1759 d 1758 172 588 0\n\
  1760 236 -115 -267 339 -460 0 1759 943 179 946 1355 0\n\
  1761 44 116 -236 0 174 586 0\n\
  1761 d 174 586 0\n\
  1762 116 -236 0 1761 587 175 0\n\
  1762 d 1761 175 0\n\
  1763 -115 -267 339 -460 0 1760 1762 177 1354 944 941 0\n\
  1763 d 1760 941 0\n\
  1764 -43 111 339 -460 0 1740 585 703 165 169 1763 0\n\
  1764 d 585 1763 0\n\
  1765 116 115 267 339 -460 0 1762 940 176 1354 944 0\n\
  1765 d 176 944 0\n\
  1766 115 267 339 -460 0 1765 1759 178 942 1355 946 0\n\
  1766 d 1765 0\n\
  1767 111 338 -460 0 1740 167 931 939 1764 164 702 171 1766 0\n\
  1767 d 167 931 1764 0\n\
  1768 -116 -115 267 -339 -460 0 1759 177 942 945 1354 0\n\
  1768 d 1354 0\n\
  1769 -339 -115 267 -460 0 1768 1762 179 940 1355 947 0\n\
  1769 d 1768 1355 947 0\n\
  1770 -110 334 -460 0 934 1767 1737 1735 168 170 1769 937 933 1756 0\n\
  1770 d 1756 0\n\
  1771 334 -460 0 1753 934 1767 1737 1735 1770 161 1352 166 927 924 932 939 1766 171 163 164 703 0\n\
  1771 d 934 1770 166 927 924 932 939 1766 171 163 164 703 0\n\
  1772 339 -460 0 1771 937 938 1767 1737 936 925 933 0\n\
  1772 d 937 938 936 933 0\n\
  1773 -267 -460 0 1753 1771 1735 1740 1767 935 923 928 698 699 1352 148 162 152 0\n\
  1773 d 1735 0\n\
  1774 111 -460 0 1753 1771 1740 1767 935 928 1352 923 698 699 148 152 162 0\n\
  1774 d 1740 1767 935 923 162 0\n\
  1775 -460 0 1753 1774 1773 1772 1771 1737 1769 170 168 152 161 1352 928 925 698 699 148 0\n\
  1775 d 1774 1773 1772 1771 1737 1769 170 168 152 161 1352 928 925 698 699 148 0\n\
  1778 -269 -121 0 707 708 190 0\n\
  1778 d 190 0\n\
  1779 -45 -121 237 0 593 1778 709 0\n\
  1780 -121 237 0 1779 189 592 0\n\
  1780 d 1779 189 592 0\n\
  1781 237 -120 -268 344 -461 0 1780 960 196 963 1359 0\n\
  1782 45 121 -237 0 191 590 0\n\
  1782 d 191 590 0\n\
  1783 121 -237 0 1782 591 192 0\n\
  1783 d 1782 591 192 0\n\
  1784 -120 -268 344 -461 0 1781 1783 194 1358 961 958 0\n\
  1784 d 1781 958 0\n\
  1785 -44 116 344 -461 0 1762 589 706 182 1784 186 0\n\
  1785 d 589 0\n\
  1786 121 120 268 344 -461 0 1783 957 193 1358 961 0\n\
  1786 d 193 961 0\n\
  1787 120 268 344 -461 0 1786 1780 195 959 1359 963 0\n\
  1787 d 1786 963 0\n\
  1788 116 267 -461 0 949 1762 184 940 952 956 1785 181 705 188 1787 0\n\
  1788 d 940 952 0\n\
  1789 -121 -120 268 -344 -461 0 1780 959 962 194 1358 0\n\
  1789 d 194 1358 0\n\
  1790 -344 -120 268 -461 0 1789 1783 196 957 1359 964 0\n\
  1790 d 1789 1359 964 0\n\
  1792 -118 268 0 180 181 706 0\n\
  1792 d 180 181 706 0\n\
  1793 -120 267 -461 0 1775 1788 1759 1757 942 1792 1790 188 955 183 953 177 946 1357 0\n\
  1794 267 -461 0 1775 949 1793 187 1788 1759 1757 185 942 1787 956 951 945 1356 178 0\n\
  1794 d 949 1793 1788 178 0\n\
  1795 120 -461 0 1775 1794 701 702 165 169 187 185 179 1356 1762 948 943 945 951 956 1785 705 1787 0\n\
  1795 d 187 185 1762 1785 1787 0\n\
  1796 -268 -461 0 1775 704 705 1795 1794 1757 1784 587 701 702 943 948 165 956 169 951 179 945 1356 0\n\
  1796 d 1757 1784 587 701 702 943 948 165 956 169 951 179 945 1356 0\n\
  1797 -461 0 1796 1792 1795 188 184 1759 1794 950 954 1790 0\n\
  1797 d 1796 1795 1794 1790 0\n\
  1799 344 -119 0 1775 954 955 183 184 1759 177 950 1357 942 946 953 0\n\
  1799 d 954 955 183 184 1759 177 950 1357 942 946 953 0\n\
  1800 -270 -126 0 710 711 207 0\n\
  1800 d 710 0\n\
  1801 -46 -126 238 0 207 597 0\n\
  1801 d 597 0\n\
  1802 -126 238 0 1801 206 596 0\n\
  1802 d 1801 206 596 0\n\
  1803 238 -125 -269 349 -462 0 1802 977 213 980 1363 0\n\
  1803 d 977 0\n\
  1804 46 126 -238 0 208 594 0\n\
  1804 d 208 594 0\n\
  1805 126 -238 0 1804 209 595 0\n\
  1805 d 1804 595 0\n\
  1806 -125 -269 349 -462 0 1803 1805 211 1362 978 975 0\n\
  1806 d 1803 975 0\n\
  1807 -45 121 349 -462 0 1783 593 709 199 1806 203 0\n\
  1807 d 593 0\n\
  1808 126 125 269 349 -462 0 1805 974 210 1362 978 0\n\
  1808 d 210 978 0\n\
  1809 125 269 349 -462 0 1808 212 1802 1363 976 980 0\n\
  1809 d 1808 0\n\
  1810 121 268 -462 0 1783 201 966 957 969 973 1807 198 708 205 1809 0\n\
  1810 d 957 969 1807 0\n\
  1811 -126 -125 269 -349 -462 0 211 1802 976 979 1362 0\n\
  1811 d 1362 0\n\
  1812 -349 -125 269 -462 0 1811 1805 213 974 1363 981 0\n\
  1812 d 1811 1363 981 0\n\
  1813 268 -120 -462 0 1810 1780 1778 202 959 204 1812 1792 972 188 970 1799 0\n\
  1813 d 1792 972 188 970 1799 0\n\
  1814 349 -125 -268 -462 0 971 1806 967 1780 201 205 197 198 709 0\n\
  1814 d 971 1806 967 201 0\n\
  1815 -125 -120 -462 0 1797 1813 1814 1812 1778 1783 196 960 965 1360 962 973 968 0\n\
  1815 d 1814 1812 0\n\
  1816 -120 -462 0 1797 1815 203 1813 204 202 1783 196 1360 960 962 968 965 973 1809 707 708 199 0\n\
  1816 d 1815 1813 204 202 1783 196 960 965 0\n\
  1817 -269 123 0 707 708 199 0\n\
  1817 d 707 708 199 0\n\
  1818 -268 118 0 704 705 182 0\n\
  1818 d 704 705 182 0\n\
  1819 -462 0 1797 1816 186 200 1818 1810 966 1780 1778 195 959 1360 962 968 973 1809 205 197 198 709 0\n\
  1819 d 1816 186 200 1818 1810 966 1780 1778 195 959 1360 962 968 973 1809 205 197 198 709 0\n\
  1822 -271 -131 0 713 714 224 0\n\
  1822 d 224 0\n\
  1823 -47 -131 239 0 601 1822 715 0\n\
  1824 -131 239 0 1823 223 600 0\n\
  1824 d 1823 223 600 0\n\
  1825 239 -130 -270 354 -463 0 1824 994 230 997 1367 0\n\
  1825 d 994 0\n\
  1826 47 131 -239 0 225 598 0\n\
  1826 d 225 598 0\n\
  1827 131 -239 0 1826 599 226 0\n\
  1827 d 1826 599 226 0\n\
  1828 -130 -270 354 -463 0 1825 1827 992 228 995 1366 0\n\
  1828 d 1825 0\n\
  1829 -46 126 354 -463 0 209 216 712 220 1828 0\n\
  1829 d 209 216 712 220 1828 0\n\
  1830 131 130 270 354 -463 0 1827 991 995 1366 227 0\n\
  1830 d 227 0\n\
  1831 130 270 354 -463 0 1830 1824 229 993 1367 997 0\n\
  1831 d 1830 0\n\
  1832 126 353 -463 0 1805 982 990 1829 215 218 222 1831 711 0\n\
  1832 d 1829 218 711 0\n\
  1833 -131 -130 270 -354 -463 0 1824 228 993 996 1366 0\n\
  1833 d 1366 0\n\
  1834 -354 -130 270 -463 0 1833 1827 991 230 998 1367 0\n\
  1834 d 1833 230 998 1367 0\n\
  1836 -128 269 -463 0 214 215 207 1832 1805 986 974 0\n\
  1836 d 214 215 207 0\n\
  1837 126 269 -463 0 1832 1805 986 974 0\n\
  1838 125 -463 0 1819 203 1817 217 1837 1802 1800 976 983 1836 212 1364 979 985 990 222 1831 0\n\
  1838 d 217 1837 983 1836 212 222 1831 0\n\
  1840 -126 -463 0 1819 1800 1838 1802 211 219 1365 221 1834 988 989 984 976 987 980 0\n\
  1840 d 1802 211 219 1365 221 1834 988 989 984 976 987 980 0\n\
  1841 -463 0 1819 1840 1838 213 1364 1832 985 986 979 0\n\
  1841 d 1840 1838 1832 0\n\
  1843 -353 126 0 1819 1805 985 986 979 974 1364 1817 213 203 0\n\
  1843 d 985 986 979 974 1364 1817 213 203 0\n\
  1844 -272 -136 0 716 717 241 0\n\
  1845 -48 -136 240 0 241 605 0\n\
  1845 d 605 0\n\
  1846 -136 240 0 1845 240 604 0\n\
  1846 d 1845 240 604 0\n\
  1847 240 -135 -271 359 -464 0 1846 1011 247 1014 1371 0\n\
  1848 48 136 -240 0 242 602 0\n\
  1848 d 242 602 0\n\
  1849 136 -240 0 1848 603 243 0\n\
  1849 d 1848 603 243 0\n\
  1850 -135 -271 359 -464 0 1847 1849 245 1370 1012 1009 0\n\
  1850 d 1847 1009 0\n\
  1851 -47 131 359 -464 0 1827 601 715 233 1850 237 0\n\
  1851 d 601 0\n\
  1852 136 135 271 359 -464 0 1849 1008 244 1370 1012 0\n\
  1852 d 244 1012 0\n\
  1853 135 271 359 -464 0 1852 1846 246 1010 1371 1014 0\n\
  1853 d 1852 0\n\
  1854 131 358 -464 0 1827 235 999 1007 1851 232 714 239 1853 0\n\
  1854 d 235 999 1851 0\n\
  1855 -136 -135 271 -359 -464 0 1846 245 1010 1013 1370 0\n\
  1855 d 1370 0\n\
  1856 -359 -135 271 -464 0 1855 1849 247 1008 1371 1015 0\n\
  1856 d 1855 1371 1015 0\n\
  1857 -130 126 -464 0 1841 1805 1843 982 990 1002 1854 1824 1822 228 236 1369 238 997 1856 993 1005 1001 0\n\
  1857 d 1005 1001 0\n\
  1858 126 -464 0 1841 1805 1843 1857 234 982 990 1002 1854 1824 1822 229 1368 995 992 1000 1007 1853 239 231 232 715 0\n\
  1858 d 1805 1843 1857 982 990 995 992 0\n\
  1859 271 -135 -464 0 1841 1858 1800 1856 1006 1854 1824 993 1004 997 1369 228 234 239 231 232 715 0\n\
  1859 d 1856 1006 1854 1824 993 1004 997 1369 228 234 239 231 232 715 0\n\
  1860 -271 -464 0 713 714 1858 233 1800 237 1000 1822 1827 991 1003 1007 1850 0\n\
  1860 d 1822 1850 0\n\
  1861 -464 0 1841 1860 1858 1859 1800 238 1853 1000 1007 1002 1003 991 996 1827 1368 229 236 0\n\
  1861 d 1860 1858 1859 1800 238 1853 1000 1007 1002 1003 991 996 1827 1368 229 236 0\n\
  1891 361 -137 0 1861 1372 0\n\
  1891 d 1372 0\n\
  1892 -361 137 0 1861 1373 0\n\
  1892 d 1373 0\n\
  1893 465 -496 0 1557 1570 1444 1588 1447 1612 1450 1637 1453 1659 1456 1684 1459 1708 1462 1731 1465 1753 1468 1775 1471 1797 1474 1819 1477 1841 1480 1861 1483 1486 0\n\
  1893 d 1444 1447 1450 1453 1456 1459 1462 1465 1468 1471 1474 1477 1480 1483 1486 0\n\
  1894 511 481 0 1535 1534 0\n\
  1894 d 1534 0\n\
  1896 -274 -242 0 722 723 611 0\n\
  1896 d 611 0\n\
  1897 -275 -243 0 725 726 615 0\n\
  1897 d 615 0\n\
  1898 -276 -244 0 728 729 619 0\n\
  1898 d 619 0\n\
  1899 -277 -245 0 731 732 623 0\n\
  1899 d 623 0\n\
  1900 -278 -246 0 734 735 627 0\n\
  1900 d 627 0\n\
  1901 -279 -247 0 737 738 631 0\n\
  1901 d 631 0\n\
  1902 -280 -248 0 740 741 635 0\n\
  1902 d 635 0\n\
  1903 -281 -249 0 743 744 639 0\n\
  1903 d 639 0\n\
  1904 -282 -250 0 746 747 643 0\n\
  1904 d 643 0\n\
  1905 -283 -251 0 749 750 647 0\n\
  1905 d 647 0\n\
  1906 -284 -252 0 752 753 651 0\n\
  1906 d 651 0\n\
  1907 -285 -253 0 755 756 655 0\n\
  1907 d 756 655 0\n\
  1908 -286 -254 0 758 759 659 0\n\
  1908 d 659 0\n\
  1910 -65 221 -257 0 532 671 0\n\
  1910 d 532 671 0\n\
  1911 221 -257 0 1910 531 670 0\n\
  1911 d 1910 531 670 0\n\
  1912 -63 211 -255 0 663 498 0\n\
  1912 d 663 498 0\n\
  1913 211 -255 0 1912 662 497 0\n\
  1913 d 1912 662 497 0\n\
  1914 -50 146 -242 0 1896 724 277 0\n\
  1914 d 277 0\n\
  1915 146 -242 0 1914 610 276 0\n\
  1915 d 1914 610 276 0\n\
  1916 -51 151 -243 0 1897 727 294 0\n\
  1916 d 294 0\n\
  1917 151 -243 0 1916 293 614 0\n\
  1917 d 1916 293 614 0\n\
  1918 -52 156 -244 0 1898 730 311 0\n\
  1918 d 311 0\n\
  1919 156 -244 0 1918 310 618 0\n\
  1919 d 1918 310 618 0\n\
  1920 -53 161 -245 0 1899 733 328 0\n\
  1920 d 328 0\n\
  1921 161 -245 0 1920 327 622 0\n\
  1921 d 1920 327 622 0\n\
  1922 -54 166 -246 0 1900 736 345 0\n\
  1922 d 345 0\n\
  1923 166 -246 0 1922 626 344 0\n\
  1923 d 1922 626 344 0\n\
  1924 -55 171 -247 0 1901 739 362 0\n\
  1924 d 362 0\n\
  1925 171 -247 0 1924 361 630 0\n\
  1925 d 1924 361 630 0\n\
  1926 -56 176 -248 0 1902 742 379 0\n\
  1926 d 379 0\n\
  1927 176 -248 0 1926 634 378 0\n\
  1927 d 1926 634 378 0\n\
  1928 -57 181 -249 0 1903 745 396 0\n\
  1928 d 396 0\n\
  1929 181 -249 0 1928 395 638 0\n\
  1929 d 1928 395 638 0\n\
  1930 -58 186 -250 0 1904 748 413 0\n\
  1930 d 413 0\n\
  1931 186 -250 0 1930 412 642 0\n\
  1931 d 1930 412 642 0\n\
  1932 -59 191 -251 0 430 1905 751 0\n\
  1933 191 -251 0 1932 646 429 0\n\
  1933 d 1932 0\n\
  1934 -60 196 -252 0 1906 754 447 0\n\
  1934 d 447 0\n\
  1935 196 -252 0 1934 446 650 0\n\
  1935 d 1934 446 650 0\n\
  1936 -61 201 -253 0 1907 757 464 0\n\
  1936 d 464 0\n\
  1937 201 -253 0 1936 463 654 0\n\
  1937 d 1936 654 0\n\
  1938 -62 206 -254 0 1908 760 481 0\n\
  1938 d 481 0\n\
  1939 206 -254 0 1938 658 480 0\n\
  1939 d 1938 658 480 0\n\
  1940 -64 216 -256 0 515 667 0\n\
  1940 d 515 667 0\n\
  1941 216 -256 0 1940 514 666 0\n\
  1941 d 1940 514 666 0\n\
  1942 -288 -256 0 764 765 1941 513 0\n\
  1942 d 765 0\n\
  1944 -49 241 273 0 721 609 0\n\
  1944 d 609 0\n\
  1945 49 -145 241 0 266 608 273 257 269 0\n\
  1945 d 269 0\n\
  1946 -145 -140 146 241 242 -366 369 -497 0 281 1945 1944 719 1045 258 1048 264 1379 1375 1489 1893 0\n\
  1947 -497 -140 146 241 242 -366 369 0 1946 271 272 278 270 264 1375 1893 1489 1378 1046 1042 719 720 267 0\n\
  1947 d 1946 0\n\
  1949 -153 275 0 299 300 727 0\n\
  1949 d 299 300 727 0\n\
  1950 243 373 -379 0 1067 1075 1070 1071 1058 1059 1050 1896 0\n\
  1951 -372 -140 -146 241 -243 -366 369 -375 -498 0 1917 1050 1051 1056 719 720 1043 1064 258 267 1046 264 271 1375 279 287 1893 1378 289 1489 296 1492 1382 0\n\
  1952 -148 274 0 282 283 724 0\n\
  1952 d 282 283 724 0\n\
  1953 -243 -140 241 -242 -366 369 -372 -498 0 1896 1915 1061 1951 0\n\
  1953 d 1951 0\n\
  1955 51 -151 243 0 291 616 0\n\
  1955 d 291 616 0\n\
  1956 -151 243 0 1955 292 617 0\n\
  1956 d 1955 292 617 0\n\
  1957 -150 -140 241 -242 -366 369 -498 0 1915 1896 1952 290 285 279 1945 1944 719 1052 1043 258 1056 1046 264 1378 1375 1893 1489 1492 1953 1956 1059 298 1066 1383 0\n\
  1957 d 1896 1953 0\n\
  1958 -273 143 0 719 720 267 0\n\
  1958 d 267 0\n\
  1962 53 -161 245 0 325 624 0\n\
  1962 d 325 624 0\n\
  1963 -161 245 0 1962 326 625 0\n\
  1963 d 1962 326 625 0\n\
  1964 54 -166 246 0 342 628 0\n\
  1964 d 342 628 0\n\
  1965 246 -166 0 1964 343 629 0\n\
  1965 d 1964 343 629 0\n\
  1967 56 -176 248 0 376 636 0\n\
  1967 d 376 636 0\n\
  1968 248 -176 0 1967 377 637 0\n\
  1968 d 1967 377 637 0\n\
  1969 -173 279 0 367 368 739 0\n\
  1969 d 367 368 739 0\n\
  1970 58 -186 250 0 410 644 0\n\
  1970 d 410 644 0\n\
  1971 250 -186 0 1970 411 645 0\n\
  1971 d 1970 645 0\n\
  1972 -183 281 0 401 402 745 0\n\
  1972 d 401 402 745 0\n\
  1973 60 -196 252 0 444 652 0\n\
  1973 d 444 652 0\n\
  1974 252 -196 0 1973 445 653 0\n\
  1974 d 1973 445 653 0\n\
  1975 -193 283 0 435 751 436 0\n\
  1975 d 435 436 0\n\
  1976 -158 276 0 316 317 730 0\n\
  1976 d 316 317 730 0\n\
  1977 499 157 -162 -244 385 -500 0 1919 314 1493 1498 1491 1391 1100 1090 1091 1086 1949 1078 307 1089 302 303 1073 1074 1956 296 1069 1385 1061 1065 1072 0\n\
  1977 d 1100 0\n\
  1978 384 -155 -244 467 0 1090 1091 1086 1949 1078 307 1089 302 303 1073 1074 1956 296 1069 1385 1061 1065 1072 0\n\
  1979 -383 -381 0 1087 1088 1081 0\n\
  1979 d 1081 0\n\
  1980 498 -155 -244 -499 0 1491 1495 1919 1978 313 1386 1979 1092 1085 1897 1067 1077 1080 1950 1075 1053 1054 1071 1059 722 723 275 1915 1042 719 720 607 1033 1041 1036 1037 1025 1844 1849 1016 1024 1019 1020 1013 1008 1891 713 714 247 233 237 0\n\
  1980 d 1077 0\n\
  1981 -379 243 0 1067 1075 1950 1053 1054 1071 1059 722 723 275 1915 1042 719 720 607 1033 1041 1036 1037 1025 1844 1849 1016 1024 1019 1020 1013 1008 1891 713 714 247 233 237 0\n\
  1981 d 1067 1950 1071 0\n\
  1982 -271 133 0 713 714 233 0\n\
  1982 d 713 714 233 0\n\
  1983 -166 165 277 389 -470 0 1965 348 1112 1395 1116 0\n\
  1984 165 277 389 -470 0 1983 1923 1110 1114 1394 346 0\n\
  1984 d 1983 346 0\n\
  1985 277 245 389 -470 0 1984 1963 337 341 333 334 733 0\n\
  1986 166 -277 389 -470 0 1923 731 732 335 339 349 1395 1113 1116 0\n\
  1986 d 1113 1116 0\n\
  1987 -470 245 389 0 1985 731 732 1986 1965 1111 1114 1394 347 335 339 0\n\
  1987 d 1985 731 732 1986 335 339 0\n\
  1989 -257 -220 288 -444 -481 0 1911 1299 1302 534 1438 0\n\
  1989 d 1302 0\n\
  1990 65 -221 257 0 529 672 0\n\
  1990 d 529 672 0\n\
  1991 -221 257 0 1990 530 673 0\n\
  1991 d 1990 530 673 0\n\
  1992 -481 -220 288 -444 0 1989 1991 1297 536 1439 1304 0\n\
  1992 d 1989 1304 0\n\
  1993 -63 -210 -215 255 -256 -436 439 509 0 665 1942 1941 496 763 525 517 502 1281 1290 527 1431 1284 1294 1528 1434 1992 1531 1894 0\n\
  1994 439 -210 -215 255 -256 -436 509 0 1993 504 664 511 495 507 0\n\
  1994 d 1993 0\n\
  1998 444 -256 -439 0 1294 1295 1290 1293 1282 0\n\
  1999 -287 -255 0 761 1913 496 762 0\n\
  1999 d 762 0\n\
  2000 -208 -255 -436 478 0 486 487 760 479 1908 1264 1254 1267 1262 1257 1258 1251 1246 1428 755 1907 485 656 475 471 0\n\
  2003 254 -434 0 1254 1262 1257 1258 1246 1907 1237 1245 1240 1241 1229 1906 1220 1228 1223 1224 1212 1905 1203 1211 1206 1207 1195 1904 1186 1194 1189 1190 1178 1903 1169 1177 1172 1173 1161 1902 1152 1160 1155 1156 1144 1901 1135 1143 1138 1139 1127 1900 1118 1126 1121 1122 1110 1899 1101 1109 1104 1105 1093 1898 1084 1092 1087 1088 1981 1076 1897 0\n\
  2003 d 1254 1258 0\n\
  2004 -423 252 0 1223 1224 1212 1905 1203 1211 1206 1207 1195 1904 1186 1194 1189 1190 1178 1903 1169 1177 1172 1173 1161 1902 1152 1160 1155 1156 1144 1901 1135 1143 1138 1139 1127 1900 1118 1126 1121 1122 1110 1899 1101 1109 1104 1105 1093 1898 1084 1092 1087 1088 1981 1076 1897 0\n\
  2005 -388 245 0 1104 1105 1093 1898 1084 1092 1087 1088 1981 1076 1897 0\n\
  2005 d 1105 0\n\
  2007 -138 272 0 248 249 718 0\n\
  2007 d 248 249 718 0\n\
  2008 -363 135 0 237 1019 1020 1982 1013 1891 246 1849 1008 0\n\
  2008 d 246 1008 0\n\
  2010 -49 140 241 -274 366 369 -497 0 1944 722 723 719 1958 275 258 271 1915 261 281 1045 1374 1048 1893 1379 1489 0\n\
  2011 -497 140 241 -274 366 369 0 722 723 275 1915 2010 1945 608 720 278 257 1042 261 1046 1374 1378 1893 1489 0\n\
  2011 d 2010 0\n\
  2012 -243 -274 374 -467 0 1917 1060 722 723 1063 284 1382 288 296 0\n\
  2013 -467 -274 374 0 722 723 284 288 2012 1956 298 1383 1065 1062 0\n\
  2013 d 2012 1062 0\n\
  2016 -256 -210 -255 -434 509 0 1942 1941 2003 1999 1913 1908 1282 508 500 1265 510 1276 1268 525 517 1278 1430 527 1998 1285 1528 1992 1434 1894 1531 0\n\
  2017 257 -220 288 444 481 0 1991 1297 536 1301 1440 0\n\
  2017 d 1440 0\n\
  2018 288 -220 444 481 0 2017 1911 1299 534 1303 1441 0\n\
  2018 d 2017 1441 0\n\
  2019 481 -215 -441 444 510 0 1894 1531 1435 519 525 527 2018 764 1942 668 513 0\n\
  2019 d 519 2018 0\n\
  2020 -257 -288 444 -481 0 1911 764 1942 668 522 526 1298 1301 1438 534 0\n\
  2020 d 1298 534 0\n\
  2021 -288 444 -481 0 1942 764 668 2020 1991 522 1300 1303 1439 526 536 0\n\
  2021 d 764 2020 522 1300 526 536 0\n\
  2022 -213 287 0 503 504 763 0\n\
  2022 d 503 504 763 0\n\
  2023 -430 431 433 0 1252 1259 0\n\
  2023 d 1259 0\n\
  2024 -254 430 432 0 1248 1256 0\n\
  2024 d 1256 0\n\
  2025 -253 425 427 0 1239 1231 0\n\
  2026 -62 254 286 0 760 661 0\n\
  2026 d 661 0\n\
  2027 478 -210 -255 439 0 1277 1273 1265 1278 1276 1260 1261 1269 2000 494 489 490 483 1429 2023 2024 2026 660 478 0\n\
  2027 d 1278 1276 2000 0\n\
  2028 254 -210 286 0 2026 487 660 494 478 490 0\n\
  2028 d 490 0\n\
  2029 -198 284 0 452 453 754 0\n\
  2029 d 452 453 754 0\n\
  2030 -203 285 0 469 470 757 0\n\
  2030 d 469 470 757 0\n\
  2032 -61 203 253 0 657 471 0\n\
  2032 d 471 0\n\
  2033 61 -201 253 0 461 656 0\n\
  2034 253 -201 0 2033 462 657 0\n\
  2034 d 2033 657 0\n\
  2035 -59 193 251 0 437 649 0\n\
  2035 d 437 649 0\n\
  2036 251 -191 193 0 2035 648 427 0\n\
  2036 d 648 427 0\n\
  2037 -188 282 0 418 419 748 0\n\
  2037 d 418 419 748 0\n\
  2038 -57 183 249 0 403 641 0\n\
  2038 d 641 0\n\
  2039 249 -181 183 0 2038 393 640 0\n\
  2039 d 2038 393 640 0\n\
  2040 -178 280 0 384 385 742 0\n\
  2040 d 384 385 742 0\n\
  2041 -55 247 279 0 1969 369 633 0\n\
  2041 d 633 0\n\
  2042 247 -171 279 0 2041 359 632 0\n\
  2042 d 2041 359 632 0\n\
  2043 -168 278 0 350 351 736 0\n\
  2043 d 350 351 736 0\n\
  2044 -163 277 0 333 334 733 0\n\
  2044 d 333 334 733 0\n\
  2045 -52 244 276 0 1976 318 621 0\n\
  2045 d 621 0\n\
  2046 244 -156 276 0 2045 308 620 0\n\
  2046 d 2045 308 620 0\n\
  2047 -50 242 274 0 1952 284 613 0\n\
  2047 d 613 0\n\
  2048 242 -146 274 0 2047 274 612 0\n\
  2048 d 2047 274 612 0\n\
  2049 -143 273 0 265 266 721 0\n\
  2049 d 265 266 721 0\n\
  2050 241 -145 273 0 1944 1945 0\n\
  2050 d 1945 0\n\
  2051 394 -169 0 1125 353 354 1965 1124 1120 2044 1112 341 1123 336 1107 1108 337 1963 1103 1976 1095 324 1106 319 320 1090 1091 2046 1978 1086 1949 1078 307 1089 302 303 1073 296 1956 1069 1382 1952 1061 290 1064 285 286 1056 1057 2048 1052 1044 2049 2050 1055 273 1039 1040 268 1035 2007 1027 256 1038 251 252 1022 1023 1846 245 1018 1010 1892 1021 1014 0\n\
  2051 d 1124 1120 337 0\n\
  2054 364 -139 0 252 251 1022 1023 1846 245 1018 1010 1892 1021 1014 0\n\
  2054 d 251 1022 1023 1846 245 1018 1010 1892 1021 1014 0\n\
  2055 -156 244 0 2046 728 729 309 0\n\
  2055 d 2046 309 0\n\
  2057 -245 -276 384 -469 0 1921 728 729 318 322 330 1390 1097 1094 0\n\
  2057 d 1094 0\n\
  2058 -469 -276 384 0 728 729 318 322 2057 1963 332 1391 1099 1096 0\n\
  2058 d 2057 1096 0\n\
  2059 -252 195 283 419 -476 0 1935 1214 1218 1419 450 0\n\
  2059 d 450 0\n\
  2060 -476 195 283 419 0 2059 1974 1212 448 1216 1418 0\n\
  2060 d 2059 448 0\n\
  2062 -64 256 288 0 669 766 0\n\
  2063 257 220 444 -481 0 2021 1297 1991 1301 533 1438 0\n\
  2063 d 1297 1991 1301 533 1438 0\n\
  2064 -255 -210 286 -434 478 509 0 1913 1999 1265 508 500 510 2027 2016 1552 1280 1287 1268 1430 1528 2019 2021 2062 521 668 512 524 528 2063 1911 1299 535 1303 1439 0\n\
  2064 d 2027 2016 1268 0\n\
  2065 -481 256 0 1552 2021 2062 521 668 512 524 528 2063 1911 1299 535 1303 1439 0\n\
  2065 d 2021 2062 524 0\n\
  2066 -434 -210 -215 255 256 510 0 2003 1271 1552 2065 1527 1908 2019 1263 1275 1270 1279 1433 1286 500 1283 507 2022 511 0\n\
  2067 -210 -215 255 256 510 0 1271 2066 1552 2065 1526 1527 1260 1261 1274 2019 1524 1279 1286 1283 2022 511 507 500 1433 1269 1266 2028 1939 2024 2023 1429 483 489 494 486 487 760 0\n\
  2067 d 2066 1527 507 1433 760 0\n\
  2068 -440 441 443 0 1286 1293 0\n\
  2068 d 1293 0\n\
  2069 510 -215 256 0 2065 1526 1552 1524 1295 2019 2068 1283 2022 511 506 1287 1277 2067 1273 2028 2064 1939 1260 1261 2024 2023 1429 483 489 494 486 487 479 0\n\
  2069 d 1526 1552 1524 1295 2019 2068 1287 2067 0\n\
  2071 -254 434 -478 0 1939 1260 1261 2024 2023 1251 1247 1427 1243 1244 2030 484 477 472 473 2034 2025 1239 1242 2029 1226 1227 460 455 456 1974 1222 1214 1975 1225 443 1209 1210 438 439 2036 1205 2037 1197 426 1208 421 422 1192 1193 1971 1188 1180 1972 1191 409 1175 1176 404 405 2039 1171 2040 1163 392 1174 387 388 1158 1159 1968 1154 1969 1146 375 1157 370 371 1141 1142 2042 1137 2043 1129 358 1140 2051 0\n\
  2072 -285 203 0 755 1907 2032 656 0\n\
  2073 -249 -180 189 -414 0 1903 1929 1187 406 1194 408 423 1931 1178 1190 0\n\
  2073 d 1190 0\n\
  2074 -412 189 0 1186 1187 1931 743 744 423 403 407 0\n\
  2075 249 279 -409 0 1169 1153 1177 1172 1173 1160 1161 1156 1902 1144 0\n\
  2076 -247 -170 189 -414 0 1901 1925 2074 1194 1153 372 1189 374 2075 2073 390 391 389 1927 1144 1156 1160 1172 1177 1170 740 741 386 0\n\
  2076 d 2075 0\n\
  2077 -282 188 0 746 747 420 0\n\
  2077 d 420 0\n\
  2079 -284 198 0 752 753 454 0\n\
  2079 d 752 753 454 0\n\
  2080 -280 178 0 740 741 386 0\n\
  2080 d 740 741 386 0\n\
  2081 -398 247 0 1138 1139 1127 1900 1118 1126 1121 1122 1110 1899 2005 1101 1109 0\n\
  2081 d 1139 0\n\
  2082 -402 179 0 1152 1153 1927 737 738 389 369 373 0\n\
  2083 247 -399 0 2081 1143 1135 0\n\
  2083 d 2081 1135 0\n\
  2084 -278 168 0 734 735 352 0\n\
  2084 d 734 735 352 0\n\
  2085 245 -167 -246 -394 0 2005 1101 1109 1121 1987 1126 1119 1396 1111 1114 0\n\
  2085 d 1396 1111 1114 0\n\
  2086 -383 159 468 0 1087 1088 1979 1981 1897 1076 1919 321 314 1388 0\n\
  2086 d 1388 0\n\
  2087 -382 159 0 1084 1085 1919 725 726 321 301 305 0\n\
  2087 d 1084 1085 321 0\n\
  2088 -145 147 149 0 281 287 0\n\
  2089 147 149 -370 0 2088 271 280 1958 1915 1042 0\n\
  2089 d 2088 0\n\
  2090 -140 142 144 0 270 264 0\n\
  2090 d 264 0\n\
  2091 271 139 -364 0 1017 1024 1019 1020 1013 1891 2008 253 247 0\n\
  2091 d 1017 1020 2008 0\n\
  2092 139 -364 0 2091 1982 237 253 247 1849 1891 1011 1016 1013 1024 1019 0\n\
  2092 d 2091 1982 237 253 247 1849 1891 1011 1016 1013 1024 1019 0\n\
  2093 -49 141 -241 0 260 607 0\n\
  2093 d 260 607 0\n\
  2094 141 -241 0 2093 259 606 0\n\
  2094 d 2093 259 606 0\n\
  2095 -273 -141 0 719 720 258 0\n\
  2095 d 719 720 258 0\n\
  2096 -141 -140 149 -374 0 2095 270 1051 272 1058 287 1054 1915 1042 0\n\
  2097 -375 149 -152 -364 -379 0 2092 1981 252 255 1060 1068 1075 1070 2096 2094 1033 1844 1025 1037 1041 1053 1058 1050 1051 1915 1958 287 271 0\n\
  2097 d 1060 1844 0\n\
  2098 -274 148 0 722 723 284 0\n\
  2098 d 284 0\n\
  2099 145 368 -374 0 285 271 1958 1051 1058 1053 1041 1033 1034 2094 716 717 2096 250 254 0\n\
  2099 d 271 1958 0\n\
  2100 -383 159 0 1087 1981 1897 1917 1088 1076 1919 1979 2086 1386 313 306 304 288 289 297 2098 1068 1061 1075 1070 2097 1036 2099 287 1915 1050 1058 1053 1054 1041 1042 1033 2095 2094 0\n\
  2100 d 1087 1088 1979 2086 2097 0\n\
  2102 246 388 -394 0 1118 1126 1121 1122 1109 1110 1101 1899 0\n\
  2102 d 1118 0\n\
  2103 384 169 -394 0 1104 2102 1923 355 340 348 2085 1899 1921 1119 338 1126 322 1121 1109 1102 728 729 318 0\n\
  2103 d 1121 0\n\
  2104 -245 -160 169 -394 0 1899 1921 338 340 355 1923 1119 1110 1126 1122 0\n\
  2104 d 1119 1110 1126 1122 0\n\
  2105 245 169 -394 0 2005 2102 2085 1923 348 355 0\n\
  2105 d 2102 2085 1923 348 355 0\n\
  2106 -394 169 0 2103 2105 2104 323 2100 2087 1092 0\n\
  2106 d 2103 2105 2104 0\n\
  2107 -399 189 -414 0 2083 2076 356 357 2084 2106 1136 1138 1143 0\n\
  2107 d 2076 0\n\
  2108 -368 241 0 1036 2092 252 1037 1025 716 717 241 0\n\
  2108 d 1037 0\n\
  2109 -374 149 241 0 2108 1033 1041 1053 1058 2099 1050 1915 287 0\n\
  2109 d 2099 1050 287 0\n\
  2110 279 189 -414 0 1153 2107 2074 1155 1194 1160 1189 1172 1177 1169 1170 2073 2080 390 0\n\
  2111 180 403 -409 0 390 391 2080 2082 1160 1172 1177 1170 0\n\
  2111 d 1170 0\n\
  2112 190 -419 0 424 425 2077 2074 1204 1211 1206 2107 1194 1155 1189 2111 2073 1169 1177 1172 1173 1160 1161 1152 1902 0\n\
  2112 d 1204 0\n\
  2113 -419 194 0 2112 440 1933 1203 1211 1206 1207 1195 746 747 1904 411 1186 422 1194 2107 1189 1155 2111 2073 1169 1177 1172 1173 1160 1161 1152 1902 0\n\
  2113 d 2112 440 1933 1203 1211 1206 1207 0\n\
  2114 -286 -206 0 758 759 479 0\n\
  2115 -206 254 0 2114 2026 478 660 0\n\
  2115 d 2114 2026 478 660 0\n\
  2116 -63 213 255 0 665 505 0\n\
  2116 d 665 505 0\n\
  2117 64 -216 256 0 512 668 0\n\
  2117 d 512 668 0\n\
  2118 -216 256 0 2117 513 669 0\n\
  2118 d 2117 513 669 0\n\
  2119 256 210 255 436 440 441 509 0 2065 2118 1283 506 1894 2022 511 2116 516 664 1434 495 1531 499 1528 1430 0\n\
  2120 436 210 255 440 441 509 0 2119 1942 1941 1282 761 664 496 2116 499 509 1430 525 517 1528 527 1434 1531 1894 1992 1290 1294 0\n\
  2120 d 2119 1290 1294 0\n\
  2121 255 210 286 434 441 509 0 1271 1263 1274 1267 1279 1286 2120 0\n\
  2121 d 2120 0\n\
  2122 441 210 286 434 509 0 506 1272 1274 1279 1286 2121 1265 1913 1999 1269 501 2022 1282 1431 511 2065 2118 1528 1894 516 1531 1434 0\n\
  2122 d 2121 0\n\
  2123 -218 288 0 520 521 766 0\n\
  2123 d 520 521 766 0\n\
  2124 -255 210 286 434 509 0 1913 1999 506 2122 1272 1274 2022 1289 1279 511 1291 1284 523 1296 1280 1942 1941 2123 518 528 1435 1265 1269 501 1431 1528 1531 1894 2063 1911 1299 535 1303 1439 0\n\
  2124 d 1265 501 0\n\
  2125 -256 210 255 436 -440 -441 443 509 0 1942 1941 2123 1281 2022 1289 2116 1296 664 495 499 1430 1528 506 511 523 518 528 1435 1531 1894 2063 1911 1299 535 1303 1439 0\n\
  2125 d 1281 506 511 0\n\
  2126 436 210 255 -440 -441 443 509 0 2125 1280 761 664 496 2116 509 2069 1528 1430 499 0\n\
  2126 d 2125 499 0\n\
  2127 286 254 509 0 1272 2003 1274 1279 1291 2028 2122 2124 1284 1263 2126 1267 0\n\
  2127 d 2028 0\n\
  2128 256 -210 -436 -440 509 0 1280 761 1999 664 2116 509 2069 1528 1431 502 496 0\n\
  2128 d 1280 496 0\n\
  2129 -441 -210 255 -436 439 509 0 1284 1291 2128 1942 1941 1994 509 510 2116 508 664 502 761 1431 1528 2123 523 518 1289 528 1435 1531 1894 1296 2063 1911 1299 535 1303 1439 0\n\
  2129 d 2123 523 518 1289 528 1435 1296 2063 1911 1299 535 1303 1439 0\n\
  2130 -256 -210 255 -436 439 509 0 2129 1994 1286 1282 509 2116 664 761 0\n\
  2130 d 1994 1286 509 761 0\n\
  2131 -436 -210 255 439 509 0 2130 2129 2128 1283 2118 2065 1894 2022 2116 664 495 502 1431 1528 2069 1531 1434 516 0\n\
  2131 d 2130 2129 2128 1283 2118 2065 2022 2116 664 495 502 1431 516 0\n\
  2132 255 -286 434 509 0 758 759 1274 488 492 1271 1279 2131 1266 1269 0\n\
  2132 d 1271 1266 1269 0\n\
  2133 509 254 478 0 2003 2127 758 759 2132 488 1264 1267 1273 1913 1999 492 1277 508 500 510 1430 1528 2069 1282 1941 1942 1998 1285 525 517 527 1434 1992 1531 1894 0\n\
  2133 d 2127 758 759 2132 488 1264 1273 1913 1999 492 1277 508 500 510 1430 1528 2069 1282 1941 1942 1998 1285 525 517 527 1434 1992 1531 1894 0\n\
  2134 -61 200 284 285 424 -477 0 2030 2032 1937 1231 467 1235 1423 0\n\
  2134 d 2032 0\n\
  2135 29 61 200 284 424 -477 0 461 1937 465 1229 1422 1233 0\n\
  2135 d 461 0\n\
  2136 61 200 284 424 -477 0 2135 463 656 467 1231 1423 1235 0\n\
  2136 d 2135 463 656 467 1231 0\n\
  2137 -477 200 284 285 424 0 2136 2134 0\n\
  2137 d 2134 0\n\
  2140 -208 -254 0 1939 486 487 479 0\n\
  2140 d 486 487 479 0\n\
  2141 -155 154 -158 379 -468 0 307 1976 1949 1898 1079 2055 1082 315 1387 0\n\
  2142 -468 154 -158 379 0 1976 1898 2055 2141 305 312 1386 1080 1076 725 726 301 0\n\
  2142 d 2141 0\n\
  2143 -243 150 374 -467 0 1917 2013 1061 1065 1383 297 0\n\
  2143 d 297 0\n\
  2147 278 -279 394 -471 0 2043 737 738 1901 2051 358 360 1127 363 1131 1398 0\n\
  2148 -471 -279 394 0 737 738 1901 360 2147 2084 1130 356 1133 366 1399 0\n\
  2148 d 2147 0\n\
  2149 245 388 -470 0 1101 1109 1987 0\n\
  2149 d 1101 1987 0\n\
  2150 -470 -168 -276 388 0 2043 728 729 2149 1900 318 1921 1103 1965 322 1107 354 338 2106 340 1125 349 1123 1395 1117 0\n\
  2151 -191 251 0 2036 1975 749 750 428 0\n\
  2151 d 2036 0\n\
  2152 415 -190 282 -414 -475 0 1197 2151 434 1415 1202 0\n\
  2152 d 1202 0\n\
  2153 -475 -190 282 -414 0 2152 1208 1210 2113 439 432 1414 1200 0\n\
  2153 d 2152 1200 0\n\
  2154 -413 -180 -249 -409 504 -506 0 1193 1903 2073 421 422 425 415 1971 1904 1180 2153 1516 1513 1410 1183 0\n\
  2154 d 1183 0\n\
  2155 -189 414 0 422 1192 1971 1188 1180 1972 421 409 404 405 2039 1193 1191 1175 1176 1171 2040 1163 392 1174 387 388 1158 1159 1968 1154 1969 1146 375 1157 370 371 1141 1142 2042 1137 2043 1129 358 1140 2051 0\n\
  2155 d 421 1193 0\n\
  2156 -179 404 0 387 388 1159 1968 1158 1154 1969 1146 375 370 371 2042 1157 1141 1142 1137 2043 1129 358 1140 2051 0\n\
  2156 d 1158 1157 0\n\
  2157 -191 -190 414 -475 0 2151 2155 426 2037 432 1414 1199 1196 0\n\
  2157 d 432 1196 0\n\
  2158 -475 -190 414 0 2155 426 2037 2157 439 2113 1209 1205 434 1415 1201 1198 0\n\
  2158 d 2157 434 1198 0\n\
  2159 -470 -169 388 0 2149 353 354 1899 347 1965 2044 1112 1394 341 1115 336 1107 1103 1976 1095 324 1106 319 320 1090 1091 2055 1086 1978 1078 1949 1089 307 1073 302 303 1956 296 1069 1382 1952 1061 290 1064 285 286 1056 1057 2048 1052 2049 1044 2050 273 1055 268 1039 1040 1035 1027 2007 1038 256 2054 0\n\
  2160 243 -158 -468 0 1981 1956 2142 303 0\n\
  2161 498 -150 156 -158 244 -499 0 1491 1495 2160 1897 1917 1076 296 304 1385 306 315 1387 1083 1073 1074 1069 1061 1072 1065 0\n\
  2161 d 2160 0\n\
  2162 -149 274 372 373 0 285 286 2048 1052 2049 1044 2050 273 1055 268 1039 1040 1035 1027 2007 1038 256 2054 0\n\
  2163 -146 149 -241 274 373 -497 0 2048 2094 2095 1044 2089 279 272 270 255 263 2092 1055 1039 1048 1035 1379 1027 1489 1031 1893 1375 0\n\
  2163 d 1055 0\n\
  2164 369 146 -241 -497 0 1915 2094 2095 1042 1046 1039 1040 2049 1035 1027 2007 1038 2054 1031 256 268 263 1375 1893 1489 1378 278 273 0\n\
  2164 d 1039 1040 2049 1035 268 263 273 0\n\
  2165 140 -369 0 254 255 2092 1036 1041 1034 716 717 250 0\n\
  2165 d 1036 1034 0\n\
  2166 364 -241 -366 -369 0 1029 2054 2165 256 2007 1026 0\n\
  2166 d 1026 0\n\
  2167 -366 -241 -369 0 2166 2092 1030 252 1027 716 717 241 0\n\
  2167 d 2166 1030 252 1027 241 0\n\
  2168 -241 149 274 373 -497 0 2094 2095 2163 1915 2164 1042 2165 2167 1049 262 270 1374 272 1893 281 1489 1379 0\n\
  2168 d 2163 0\n\
  2169 140 241 -366 0 254 255 2092 1029 1025 716 717 250 0\n\
  2169 d 254 255 2092 1029 1025 716 717 250 0\n\
  2170 146 241 -366 -497 0 1915 2169 2108 1033 1041 1947 0\n\
  2170 d 1947 0\n\
  2171 -366 149 241 274 372 -497 0 2169 2170 2108 1033 2048 1041 1052 1044 2050 2089 1048 272 1379 1489 1893 2090 1375 0\n\
  2171 d 2090 1375 0\n\
  2172 364 -140 241 366 0 1031 2054 1028 256 2007 0\n\
  2172 d 1031 2054 1028 256 2007 0\n\
  2173 -364 366 368 0 1032 1038 0\n\
  2173 d 1032 1038 0\n\
  2174 -141 241 0 2095 1944 257 608 0\n\
  2174 d 1944 257 608 0\n\
  2175 -146 145 273 274 369 -466 0 2048 280 1379 1048 1044 0\n\
  2175 d 280 1044 0\n\
  2176 145 273 274 369 -466 0 2175 1915 1042 1046 1378 278 0\n\
  2176 d 2175 278 0\n\
  2177 273 241 274 -466 0 2108 1033 1041 2176 2050 0\n\
  2177 d 2176 2050 0\n\
  2179 140 141 241 -465 0 261 2169 1374 0\n\
  2180 -273 -151 274 -374 376 -498 0 2095 2094 2108 1033 2109 1041 285 286 289 2048 279 296 1043 1382 1046 1492 1378 1489 1893 2179 1957 2173 2172 0\n\
  2180 d 2179 0\n\
  2181 241 -369 0 2108 1033 1041 0\n\
  2181 d 1033 1041 0\n\
  2182 376 -151 274 -374 -498 0 2180 1051 1058 1053 1054 2181 2167 2165 1042 1047 2094 270 262 1915 2096 272 1374 279 289 1893 1378 296 1489 1382 1492 0\n\
  2182 d 2180 0\n\
  2183 274 -243 -379 -498 0 1917 1061 1068 1075 1070 2182 1064 0\n\
  2183 d 2182 1064 0\n\
  2184 -374 -274 0 722 723 275 1915 286 2109 2094 2095 1051 1042 1058 1054 0\n\
  2185 -241 146 373 -497 0 2094 2095 2164 2167 2165 262 1374 1893 1489 1915 270 1042 272 1049 1379 281 0\n\
  2185 d 2164 1915 270 1042 272 1049 281 0\n\
  2186 -274 -498 0 722 723 2184 1057 275 2013 1492 2185 2108 2181 2170 2173 2172 2011 0\n\
  2186 d 722 723 2184 275 2013 2185 2170 2011 0\n\
  2187 -497 149 241 274 372 0 2181 2108 2171 2173 2172 2174 261 1374 1893 1489 2177 1052 1045 1048 2089 1379 0\n\
  2187 d 2171 2174 2177 1052 1045 1048 2089 1379 0\n\
  2189 -140 -150 241 -242 -498 0 2181 1957 2172 2108 2173 0\n\
  2189 d 1957 2172 2108 2173 0\n\
  2190 243 -150 274 -374 -467 0 1059 1956 1066 298 1383 0\n\
  2190 d 1066 298 1383 0\n\
  2191 -243 -158 -498 -499 0 2186 1952 1061 2183 1074 1072 1056 1057 2162 290 2143 1492 2187 2168 0\n\
  2192 241 -150 -497 -498 0 2186 1952 290 285 286 279 2181 1053 2162 1051 2095 2048 1043 2189 1046 1378 1489 1893 261 1374 2169 0\n\
  2192 d 2181 2048 1043 2189 1046 261 2169 0\n\
  2193 -150 243 -374 -498 0 2186 1952 290 2190 1492 285 286 2192 2094 2095 1051 1058 1053 2165 279 1054 2167 262 1047 1374 1378 1893 1489 0\n\
  2193 d 2190 285 286 2192 279 1054 2167 262 1047 1374 1378 1893 1489 0\n\
  2194 149 -374 0 2109 2094 2096 2095 2165 1051 1053 1058 0\n\
  2194 d 2109 2094 2096 2095 2165 1051 1053 1058 0\n\
  2195 243 -374 -498 0 2194 289 2193 0\n\
  2195 d 2193 0\n\
  2196 -498 -158 -499 0 2186 1952 2191 1059 2195 1956 1063 1056 1057 2162 290 295 1382 1492 2187 2168 0\n\
  2196 d 2191 0\n\
  2197 -499 -158 0 1976 1898 2055 2196 1495 2161 288 289 302 2098 2194 1068 1070 1075 2142 0\n\
  2197 d 2196 2161 2142 0\n\
  2198 193 191 -476 0 439 2113 443 2060 750 1905 2035 0\n\
  2199 196 -195 -283 419 -476 0 1935 451 1419 1215 1218 0\n\
  2199 d 1215 1218 0\n\
  2200 -283 -195 419 -476 0 2199 1974 449 1418 1213 1216 0\n\
  2200 d 2199 1213 1216 0\n\
  2201 -476 191 0 439 2113 2198 441 1975 2200 0\n\
  2201 d 2198 2200 0\n\
  2202 -254 -285 429 508 0 1908 1939 1247 2072 1272 1250 475 491 483 493 1426 1525 2024 1260 2064 1263 1270 1275 2131 1279 0\n\
  2202 d 2024 1260 0\n\
  2203 -285 -200 284 -424 507 0 2072 1907 1238 475 2034 1229 468 1236 1241 1423 1245 1522 1257 2202 2115 1249 485 2023 1427 1525 2133 0\n\
  2204 -207 198 -200 203 206 -424 430 -478 508 0 1427 482 2079 1253 477 1238 1244 473 1242 2034 2025 0\n\
  2205 -478 -200 203 206 284 -424 430 508 0 2029 1238 2204 485 1426 476 1250 474 1245 1937 1241 1229 0\n\
  2205 d 2204 0\n\
  2206 508 -200 203 254 284 -424 430 0 2115 2205 1525 2133 0\n\
  2206 d 2205 0\n\
  2207 207 -200 206 -424 430 -477 478 0 485 1429 476 1253 474 1244 468 1242 1423 1236 0\n\
  2208 478 -200 203 206 -424 427 430 -477 0 2207 1428 482 1250 477 1245 473 1241 466 1234 1422 0\n\
  2208 d 2207 0\n\
  2209 -207 203 206 284 -424 430 -478 0 1427 482 1238 1253 477 1244 473 1242 2034 2025 0\n\
  2209 d 2025 0\n\
  2210 254 -200 284 -424 507 0 2115 1238 2203 1246 2030 2206 1522 2208 2209 485 1426 476 1250 474 1245 1937 1241 1229 0\n\
  2210 d 2206 2208 0\n\
  2211 -478 -200 -254 284 -430 432 0 2071 1238 1939 1262 1257 1251 1245 1426 1241 483 1229 476 1937 474 0\n\
  2212 -207 -200 -206 208 286 -424 -430 432 478 507 0 483 1272 476 489 474 494 468 1428 1251 1257 1262 1274 1279 1291 1244 1242 1236 1423 1522 1525 2122 2124 1284 1263 2126 1267 0\n\
  2212 d 1428 1244 1242 1236 0\n\
  2213 -252 -195 -283 507 0 1935 1906 1222 457 1238 1226 459 2210 2203 1908 2140 1939 1248 1255 2030 1272 2211 2212 1429 484 2023 1252 477 491 1261 1245 473 493 1241 466 1234 1422 1522 1525 2064 1263 1270 1275 2131 1279 0\n\
  2213 d 2211 0\n\
  2214 207 -200 203 -206 -284 286 424 -430 478 507 0 1429 484 1272 1240 2023 1252 477 491 1261 1245 473 493 1237 466 1230 1233 1422 1522 1525 2064 1263 1270 1275 2131 1279 0\n\
  2215 -200 199 -254 285 424 478 507 0 460 1908 2140 1939 1248 1255 2030 1240 2029 1272 2214 483 476 489 474 494 1937 468 1237 1232 1245 1235 1257 1423 1262 1522 1274 1525 1279 2122 2124 1291 1284 1263 2126 1267 0\n\
  2215 d 2214 0\n\
  2216 478 199 -254 285 424 507 0 1255 2030 1240 1908 2140 1272 2215 458 472 2079 477 1238 2137 1245 1522 1257 1525 1262 489 1274 1279 494 1291 2122 2124 1263 1284 1267 2126 0\n\
  2216 d 2215 489 1274 494 1291 2122 2124 1284 1267 2126 0\n\
  2217 -254 199 285 424 507 0 1939 1240 1248 1255 2216 2071 1262 1257 1251 1245 1426 1237 1238 483 1937 2079 476 458 474 0\n\
  2217 d 2216 0\n\
  2218 205 -200 206 428 430 -478 0 476 482 474 1426 1937 1250 1237 1245 0\n\
  2219 424 -200 203 206 430 -478 0 1227 1240 1226 2218 477 485 473 1427 2034 1253 1243 1239 2029 460 455 456 1974 1222 1975 1214 443 1225 438 439 1209 1210 2151 1205 2037 1197 426 1208 2155 0\n\
  2219 d 1226 2218 1253 1243 1239 0\n\
  2220 -205 203 206 -424 430 -478 0 485 2209 1906 2004 1220 1228 0\n\
  2220 d 2209 0\n\
  2221 -200 203 206 430 -478 0 2219 2220 476 482 474 1426 1937 1250 1237 1245 1241 1229 1906 2004 1220 1228 0\n\
  2221 d 2219 2220 0\n\
  2223 -422 199 0 1220 1221 1935 750 1905 457 2035 441 0\n\
  2223 d 441 0\n\
  2224 205 429 -478 0 1257 475 2072 1255 1262 2071 2115 1246 1250 1426 482 0\n\
  2224 d 475 2072 1246 482 0\n\
  2225 206 -205 430 -478 0 1939 1249 2030 477 2221 472 0\n\
  2225 d 1249 2221 472 0\n\
  2226 430 -205 -478 0 2225 2115 483 1248 1426 1907 1237 1250 1245 1240 1241 1229 1906 2004 1220 1228 0\n\
  2226 d 2225 1250 0\n\
  2227 -206 -205 -478 0 2226 2115 483 2071 1247 1255 1426 1262 1251 1257 0\n\
  2227 d 2115 483 2071 1247 1426 1262 1251 1257 0\n\
  2228 -205 -478 0 2227 1939 2226 485 2003 1261 2023 1427 0\n\
  2228 d 2227 2226 485 2003 1427 0\n\
  2229 -427 204 0 1237 1238 1937 2079 474 458 0\n\
  2230 199 -424 0 2223 1228 1223 2004 2113 1935 442 457 0\n\
  2231 -478 0 2228 476 2224 2229 1245 1240 1241 2230 456 459 1974 1906 1229 1937 474 0\n\
  2231 d 2228 476 2224 2229 474 0\n\
  2233 248 175 279 399 -472 0 1968 1144 1148 1402 380 0\n\
  2233 d 380 0\n\
  2234 -472 175 279 399 0 2233 1927 1146 382 1150 1403 0\n\
  2234 d 2233 0\n\
  2237 281 -180 249 250 -406 409 503 -505 0 1972 1178 1971 2039 1182 400 1407 1510 1513 1410 414 409 405 0\n\
  2237 d 405 0\n\
  2238 250 -180 249 -406 409 503 -505 0 1971 2237 743 744 1181 394 403 1184 400 407 1407 417 1510 1411 1513 0\n\
  2238 d 2237 0\n\
  2239 -281 -180 -250 -406 409 503 -505 0 743 744 1179 1931 394 403 1182 400 407 1407 415 1510 1410 1513 0\n\
  2240 404 -180 -405 503 -505 0 2156 1172 1167 392 2040 1162 1169 1177 1189 2238 2239 1931 1180 1972 1187 1184 2039 1194 400 2155 1407 423 1510 416 1513 1411 0\n\
  2240 d 1162 0\n\
  2241 -180 403 -405 503 -505 -506 0 2240 1174 1166 1160 1176 1152 1902 1161 1929 1903 406 398 1187 408 1406 1510 1513 2154 1194 1191 2155 1185 423 1411 417 0\n\
  2241 d 2240 0\n\
  2242 250 185 281 409 -474 0 1178 1182 1410 1971 414 0\n\
  2242 d 1178 1971 414 0\n\
  2243 -474 185 281 409 0 2242 1931 1180 1184 1411 416 0\n\
  2243 d 2242 1180 416 0\n\
  2244 404 180 -405 503 -505 0 390 404 2080 1161 1929 1903 1972 409 1159 2111 2243 1513 1510 1167 1407 399 0\n\
  2244 d 1159 1167 399 0\n\
  2245 180 -405 503 -505 0 391 1506 2082 2244 1160 1155 1156 2083 1149 1901 1404 1144 1927 382 389 0\n\
  2245 d 2244 0\n\
  2246 -405 503 -505 -506 0 2245 2241 1155 1156 2083 1901 1144 1902 2040 1161 392 1929 1903 2156 398 406 1187 408 1174 1166 1406 1510 1513 1176 2154 1191 1194 1185 2155 1411 423 417 0\n\
  2246 d 2245 2241 1174 1166 1176 0\n\
  2247 249 -180 -406 409 503 -505 0 1189 2238 1931 2239 1972 1187 2039 1194 2155 423 2243 1513 1510 400 1407 0\n\
  2247 d 2238 2239 400 1407 0\n\
  2248 503 -180 -403 -505 0 1155 1156 1514 1506 2246 2083 1149 1404 1173 1901 1144 1902 1927 1163 382 389 1169 2156 1177 1168 2247 0\n\
  2249 168 -170 -247 500 -502 0 2084 1925 364 358 1129 354 2051 1965 353 347 1132 1398 1504 1501 2159 1394 1108 2005 1115 1899 1112 0\n\
  2249 d 353 1132 0\n\
  2250 -388 166 469 -470 0 1104 1108 2005 354 1921 2106 1125 1123 1117 1395 349 340 338 323 2100 2087 1092 0\n\
  2251 -170 -247 500 -502 0 1925 1496 1497 2249 2043 1900 1965 354 2106 364 1493 1491 1128 1131 1398 1504 1501 2250 2149 1899 2044 2150 1976 1102 1095 1109 1106 1984 341 336 324 319 320 1978 2055 0\n\
  2251 d 1496 1497 2249 364 1493 1128 2150 0\n\
  2252 170 -399 0 356 357 2084 2106 1136 1138 1143 0\n\
  2252 d 1136 1138 1143 0\n\
  2253 -276 -500 0 728 729 1898 318 2055 320 2100 2087 1092 2058 1498 2197 0\n\
  2253 d 2058 2197 0\n\
  2256 159 -384 0 2100 2087 1092 0\n\
  2256 d 2100 2087 1092 0\n\
  2257 -275 -498 0 2186 1952 1897 1059 2195 1956 1063 1056 1057 2162 290 295 1382 1492 2187 2168 0\n\
  2257 d 1897 0\n\
  2258 -384 -499 0 2256 319 320 2055 1980 2186 1952 2257 1949 307 302 303 290 1956 1061 2183 1074 1072 1056 1057 2162 0\n\
  2258 d 1980 2257 0\n\
  2259 -243 378 -498 0 2186 1952 1061 1072 1056 1057 2162 290 2143 1492 2187 2168 0\n\
  2259 d 2143 0\n\
  2260 -498 378 0 2186 2259 1956 2195 1952 1059 1056 1057 1063 2162 290 295 1382 1492 2187 2168 0\n\
  2260 d 2259 1952 1059 1056 1057 1063 2162 290 295 1382 1492 2187 2168 0\n\
  2261 -244 382 -499 0 2258 1091 1086 1078 1089 1082 1074 2260 1491 1495 1978 1387 1919 314 0\n\
  2261 d 1086 1078 1919 314 0\n\
  2262 -154 379 467 0 1074 302 303 1073 296 1956 1385 1069 1061 1065 1072 0\n\
  2262 d 302 303 1073 296 1956 1385 1069 1061 1065 1072 0\n\
  2263 -155 154 244 379 -468 0 2055 315 1387 307 1082 1949 1079 0\n\
  2263 d 307 1082 1949 1079 0\n\
  2264 -499 379 382 498 0 2261 1491 1495 2055 2262 2263 305 312 1386 1080 1076 725 726 301 0\n\
  2264 d 2262 2263 305 312 1386 1080 1076 725 726 301 0\n\
  2265 161 160 276 384 -469 0 1921 1093 329 1390 1097 0\n\
  2265 d 329 1097 0\n\
  2266 160 276 384 -469 0 2265 1963 1095 331 1099 1391 0\n\
  2266 d 2265 331 1099 1391 0\n\
  2267 -500 159 499 0 2256 2253 1498 1976 324 2266 0\n\
  2267 d 2266 0\n\
  2268 -400 -399 -472 0 1149 2252 2083 1402 1901 1925 1144 372 1927 374 381 0\n\
  2269 -472 -399 0 2083 1901 1925 2252 2268 372 1151 1403 1146 1968 383 374 0\n\
  2269 d 2252 2268 372 1151 1146 374 0\n\
  2270 -505 -180 -280 404 503 0 1514 2246 1165 1164 1929 1171 1903 398 1406 1510 1513 406 1175 1187 2154 1194 2155 408 423 417 1411 1185 1191 0\n\
  2270 d 1514 1164 1929 1171 1903 398 406 1175 1187 2154 408 1185 1191 0\n\
  2272 -283 414 -475 0 749 750 1905 2158 428 424 431 2077 1414 1195 1199 0\n\
  2273 -59 414 -475 0 2158 424 2077 2272 1975 751 2035 1197 1201 1415 433 430 0\n\
  2273 d 2272 751 2035 430 0\n\
  2274 27 282 414 -475 0 2273 2158 646 2151 1195 431 1199 1414 0\n\
  2274 d 646 1195 431 1199 1414 0\n\
  2275 414 282 -475 0 2158 2273 2274 429 2151 1197 433 1201 1415 0\n\
  2275 d 2273 2274 429 433 1201 1415 0\n\
  2276 -414 282 -475 0 2153 425 2074 2107 1194 1155 1189 2111 2073 1169 1177 1172 1173 1160 1161 1152 1902 0\n\
  2276 d 2153 425 2074 2107 1152 0\n\
  2277 -475 282 0 2275 2276 0\n\
  2277 d 2275 2276 0\n\
  2279 -253 199 -200 424 507 0 2231 1907 460 2217 2029 2133 1525 1522 1230 1233 1422 466 1937 0\n\
  2279 d 1230 1937 0\n\
  2280 -200 199 424 507 0 2231 1240 2279 460 2034 1237 1245 2029 468 1232 1235 1423 1522 1525 2133 2202 2217 0\n\
  2280 d 2279 460 2034 1237 2029 468 1232 1235 1423 0\n\
  2281 195 -476 0 2201 2151 1905 442 2060 2113 0\n\
  2281 d 442 2060 2113 0\n\
  2282 -476 199 475 0 2201 2151 2281 1905 1975 443 438 457 451 1935 1419 1212 1219 1209 1210 1205 2037 1197 426 1208 2155 0\n\
  2282 d 2281 1905 457 451 1935 1419 1212 1219 0\n\
  2283 -254 199 429 507 508 0 2230 2202 2217 0\n\
  2283 d 2202 2217 0\n\
  2284 508 199 429 507 0 2231 1525 2283 2133 0\n\
  2284 d 2283 2133 0\n\
  2285 -477 200 284 424 0 2136 2137 755 1907 462 1229 465 1422 1233 0\n\
  2285 d 2136 2137 755 1907 462 1229 465 1233 0\n\
  2286 200 506 0 1515 458 459 2282 2230 1519 2079 1238 1240 1245 2284 1522 2285 0\n\
  2286 d 458 459 2079 1240 2284 2285 0\n\
  2287 -199 422 423 475 0 455 456 1974 1222 1214 1975 1225 443 1209 1210 438 439 2151 1205 2037 1197 426 2155 1208 0\n\
  2287 d 1974 1222 1214 1975 1225 443 1209 1210 438 439 2151 1205 2037 1197 426 1208 0\n\
  2288 424 422 506 0 2286 1227 1515 2287 2282 1519 2280 0\n\
  2288 d 1227 1515 2287 2282 2280 0\n\
  2289 422 506 0 2231 2288 2286 1228 2230 455 1223 1224 2004 456 1217 1906 449 1418 1519 1238 2210 2203 1908 1939 2140 1248 1255 2030 1272 2212 1429 484 2023 1252 477 491 1261 1245 473 493 1241 466 1234 1422 1522 1525 2064 1263 1270 1275 2131 1279 0\n\
  2289 d 2288 2286 1228 2230 1223 1224 2004 456 1217 1906 449 1418 1238 2210 2203 1908 1939 2140 1248 1255 2030 1272 2212 1429 484 2023 1252 477 491 1261 1245 473 493 1241 466 1234 1422 1522 1525 2064 1263 1270 1275 2131 1279 0\n\
  2290 506 0 2289 1220 1221 2223 749 750 455 428 2213 2201 1519 0\n\
  2290 d 2289 1220 1221 2223 749 750 455 428 2213 2201 1519 0\n\
  2297 -471 -279 0 737 738 1901 2083 1142 2148 2106 1140 357 1134 1399 366 360 0\n\
  2297 d 2148 360 0\n\
  2298 498 384 -499 0 1090 1091 1495 2264 2261 2055 1981 1089 1917 1083 1387 315 306 304 288 289 2098 2194 1070 1068 1075 0\n\
  2298 d 1090 1091 1495 2264 2261 1981 1089 1917 1083 1387 315 306 304 288 289 2098 2194 1068 1075 0\n\
  2299 -498 0 2260 2186 1070 1074 2195 2183 0\n\
  2299 d 2260 2186 1070 1074 2195 2183 0\n\
  2308 -245 -384 -469 0 1921 2256 320 323 2055 1898 1095 330 1098 1390 0\n\
  2308 d 330 1098 1390 0\n\
  2309 -500 -384 0 2253 2258 2256 1498 319 320 323 2308 2055 313 1093 1963 1977 332 0\n\
  2309 d 2253 2258 1498 313 1093 1963 1977 332 0\n\
  2310 245 -470 0 2149 2005 0\n\
  2310 d 2149 2005 0\n\
  2312 -388 -245 469 -470 0 1104 1899 1921 2256 1108 2250 1965 1112 323 338 340 347 1115 1394 0\n\
  2312 d 1108 2250 323 347 1115 1394 0\n\
  2313 -384 -245 388 0 2256 320 2055 1106 1898 1095 0\n\
  2313 d 2256 1106 1898 1095 0\n\
  2314 -159 384 0 2299 1491 319 320 1978 2055 0\n\
  2314 d 1491 319 320 1978 2055 0\n\
  2324 505 475 0 2290 1516 0\n\
  2324 d 1516 0\n\
  2336 503 -405 -505 0 2290 2246 0\n\
  2336 d 2246 0\n\
  2339 -499 384 0 2299 2298 0\n\
  2339 d 2298 0\n\
  2340 -276 -245 393 -470 0 1899 728 729 1103 1921 318 1107 322 1123 338 1117 1112 1965 340 1395 349 0\n\
  2340 d 728 729 1103 1921 318 1107 322 1123 338 1117 1112 340 1395 349 0\n\
  2341 -470 -245 388 0 2313 2314 1899 2159 2106 2044 1125 2340 1102 1976 1109 324 1984 336 341 0\n\
  2341 d 1899 2159 2044 1125 2340 1102 1976 1109 324 1984 336 341 0\n\
  2342 170 171 247 394 -471 0 356 2084 1127 363 1131 1398 0\n\
  2342 d 363 1131 1398 0\n\
  2343 247 279 394 -471 0 2042 2051 2342 366 358 1399 2043 1133 1130 0\n\
  2343 d 2342 1130 0\n\
  2345 -180 179 404 503 -505 0 392 2270 2040 0\n\
  2345 d 392 2270 2040 0\n\
  2346 281 180 406 409 503 -505 0 2336 1972 390 404 409 2080 1163 2243 1513 1510 1406 397 2039 0\n\
  2346 d 1972 390 404 409 2080 2243 2039 0\n\
  2347 414 -281 409 -474 0 743 744 403 407 1192 2155 1188 423 1181 417 1184 1411 0\n\
  2347 d 1192 2155 1188 423 1181 417 1184 1411 0\n\
  2349 -475 279 0 2277 746 747 2077 411 424 422 2158 2110 0\n\
  2349 d 2110 0\n\
  2350 403 279 503 0 2349 1153 2324 1160 2336 2156 2345 1165 2111 2346 1189 743 744 394 403 397 407 1406 1510 1513 2347 1194 1186 1179 1931 1182 415 1410 0\n\
  2351 503 -171 279 0 2349 2324 1506 2350 1155 1156 2248 391 1149 1144 1404 1927 382 389 0\n\
  2351 d 1506 2350 1156 2248 1149 1144 1404 1927 382 389 0\n\
  2352 -247 169 279 502 0 1925 1969 2351 1507 2269 2234 375 1141 370 1137 358 2043 0\n\
  2352 d 1141 370 1137 358 2043 0\n\
  2353 279 169 502 0 1969 2349 2324 1153 2352 2083 2042 1155 371 1160 375 2156 2234 1507 2336 2345 1165 2111 2346 1189 743 744 394 403 397 407 1406 1510 1513 2347 1194 1186 1179 1931 1182 415 1410 0\n\
  2353 d 2349 1153 2352 0\n\
  2354 179 -279 -472 0 2082 2269 737 738 1154 1968 369 1147 373 1150 383 1403 0\n\
  2354 d 737 738 1154 369 1147 373 1150 383 1403 0\n\
  2355 -472 -279 0 2269 2354 388 1968 387 381 1145 1402 1148 0\n\
  2355 d 2354 387 381 1145 1402 1148 0\n\
  2356 -505 -179 503 0 2336 388 391 1173 1968 1902 2156 1168 1163 1169 1177 2247 0\n\
  2356 d 2156 1168 1163 2247 0\n\
  2357 -179 503 0 388 391 2356 2324 2277 746 747 2077 1904 411 424 1186 422 2158 1194 2073 1189 1169 1177 1968 1173 1902 1161 0\n\
  2357 d 388 391 2356 1968 1173 1902 1161 0\n\
  2358 505 403 408 0 2324 2277 746 747 2077 1904 1186 411 424 422 2158 1194 1189 2111 2073 1177 1169 0\n\
  2358 d 2324 2277 746 747 2077 1904 411 424 422 2158 2073 1177 1169 0\n\
  2359 502 169 0 2353 2355 1901 1507 2083 2357 1155 2082 1160 1172 2358 2345 2336 2111 1165 1189 2346 743 744 394 403 397 407 1406 1510 1513 2347 1194 1186 1179 1931 1182 415 1410 0\n\
  2359 d 2353 0\n\
  2360 394 384 501 0 2051 1499 2359 1504 2297 2343 2251 1925 356 365 2084 1399 1129 1133 0\n\
  2361 -472 278 -394 0 2355 2269 2234 1969 1142 375 1140 1129 2042 371 0\n\
  2361 d 2355 2269 2234 1969 1142 375 1140 371 0\n\
  2362 406 180 409 503 -505 0 2346 1189 743 744 394 403 397 407 1406 1510 1513 2347 1194 1186 1179 1931 1182 415 1410 0\n\
  2362 d 2346 1189 743 744 394 403 397 407 1406 1510 1513 2347 1194 1186 1179 1931 1182 415 1410 0\n\
  2363 403 503 0 2357 2082 1160 1172 2358 2336 2345 1165 2111 2362 0\n\
  2363 d 2357 2082 1160 1172 2358 2336 2345 1165 2111 2362 0\n\
  2364 503 0 2363 1155 2083 1925 1901 2351 0\n\
  2364 d 2363 1155 2083 1901 2351 0\n\
  2367 388 -245 0 2364 2341 2313 2339 2314 2267 1501 2360 2106 354 357 1965 1900 2361 1507 1504 2251 2297 1127 2042 1134 366 1399 0\n\
  2367 d 2341 2313 2360 0\n\
  2368 -394 -471 500 -502 0 2297 2106 354 1965 1900 357 2251 1127 2042 1134 1399 366 0\n\
  2368 d 2106 357 1127 2042 1134 366 0\n\
  2369 -502 -245 0 2367 1104 2309 2308 2312 1501 1504 2368 2297 2343 2251 1925 356 365 2084 1399 1129 1133 0\n\
  2369 d 2367 1104 2308 2312 0\n\
  2370 502 0 2364 2359 1507 354 2051 1965 2361 1900 0\n\
  2370 d 2359 1507 354 2051 1965 2361 1900 0\n\
  2379 501 -394 0 2370 1504 1499 2368 0\n\
  2379 d 1499 2368 0\n\
  2380 -394 0 2370 2369 2310 2379 1501 2309 2339 2314 2267 0\n\
  2380 d 2379 0\n\
  2384 -500 0 2309 2339 2314 2267 0\n\
  2384 d 2309 2339 2314 2267 0\n\
  2424 0 2370 2369 2310 2380 2384 1501 1504 2297 2343 2251 1925 356 365 2084 1399 1129 1133 0\n"

/-- The golden certificate is accepted against B-C01's canonical CNF. The only
`native_decide` in the candidate's library: the kernel cannot evaluate the
checker on a 62 KB string within the laboratory's limits (NOTES.md, DS-04). -/
theorem golden_certificate_accepted :
    lrat_verdict .carry_save (cnf_text .carry_save) golden_certificate = .accept := by
  native_decide

/-- D4-TH01: the carry-save decomposition of 32-bit addition, through the
checked certificate: the golden certificate is accepted (above), so B-C01 holds
(D4-TH04), which is the identity (D4-TH02). -/
theorem carry_save_identity :
    ∀ x y : Word 32, word_add x y = word_add (word_xor x y) (word_shl (word_and x y) 1) :=
  holds_carry_save.mp
    (accepted_certificate_proves .carry_save _ _ golden_certificate_accepted)

end D006.Lrat
