import D006.Core

/-!
# D-006 v0.3, candidate C-02 (Lean 4): DS-02 Sieve

Sieve is the suite's small constant-time language (semantics.md, section 3):
8-bit words and booleans, variables and fixed-size arrays indexed by
naturals, bounded loops, public branches and one release construct,
`declassify`. Every shared S- symbol maps to one declaration here (see
`adapter.d/ds02.json`).

Every function is structurally recursive, so a closed observation evaluates
in the kernel under `decide +kernel`. The metatheory rests on two lemmas
about two well-formed, publicly equivalent states:

* `eval_agree`: a typed expression makes the same observations in both, and
  either fails in both or succeeds in both with values of its type, equal
  when its label is public;
* `step_agree`: a well-typed command takes the same kind of step in both
  (`StepAgree`), keeping typing and well-formedness, with the same
  observations and publicly equivalent states unless it releases two
  different values.

D2-TH01 `progress` and D2-TH02 `preservation` are `step_agree` with the two
states equal, D2-TH03 `lockstep_noninterference` is `step_agree` itself, and
D2-TH04 `run_noninterference` follows by induction on the fuel. D2-TH05
applies D2-TH04 to the suite's positive program; D2-TH06 evaluates the
negative one.
-/

namespace D006.Sieve

/-! ## Syntax (S-T01 to S-T12) -/

/-- S-T01: security labels. -/
inductive Label : Type where
  | «public»
  | secret
  deriving DecidableEq, Repr

/-- S-T02: value types. -/
inductive Ty : Type where
  | ty_word
  | ty_bool
  deriving DecidableEq, Repr

/-- S-T03: values, 8-bit words and booleans. -/
inductive Val : Type where
  | vword (w : Word 8)
  | vbool (b : Bool)
  deriving DecidableEq, Repr

/-- S-T04: expressions. `var x` is the `x`-th declared variable, `get a i`
reads element `i` of the `a`-th declared array. -/
inductive Expr : Type where
  | lit (v : Val)
  | var (x : Nat)
  | add (e1 e2 : Expr)
  | xor (e1 e2 : Expr)
  | band (e1 e2 : Expr)
  | eq (e1 e2 : Expr)
  | get (a : Nat) (i : Expr)
  deriving DecidableEq, Repr

/-- S-T05: commands. `set a i v` writes `v` at index `i` of array `a`;
`loop n c` runs `c` exactly `n` times. -/
inductive Cmd : Type where
  | skip
  | assign (x : Nat) (e : Expr)
  | set (a : Nat) (i v : Expr)
  | seq (c1 c2 : Cmd)
  | cond (e : Expr) (c1 c2 : Cmd)
  | loop (n : Nat) (c : Cmd)
  | declassify (x : Nat) (e : Expr)
  deriving DecidableEq, Repr

/-- S-T06: what a step reveals: the array addresses it reads and writes, each
branch outcome, each released value and each out-of-bounds address. -/
inductive Obs : Type where
  | read (a n : Nat)
  | write (a n : Nat)
  | branch (b : Bool)
  | release (v : Val)
  | oob (a n : Nat)
  deriving DecidableEq, Repr

/-- S-T07: a declared variable's type and label. -/
structure VarDecl : Type where
  vdecl ::
  ty : Ty
  label : Label
  deriving DecidableEq, Repr

/-- S-T08: a declared array's size and label. -/
structure ArrDecl : Type where
  adecl ::
  size : Nat
  label : Label
  deriving DecidableEq, Repr

/-- S-T09: the declared variables and arrays, in index order. -/
structure Env : Type where
  env ::
  vars : List VarDecl
  arrs : List ArrDecl
  deriving DecidableEq, Repr

/-- S-T10: the variables' values and the arrays' contents, in index order. -/
structure State : Type where
  state ::
  vars : List Val
  arrs : List (List (Word 8))
  deriving DecidableEq, Repr

/-- S-T11: the result of one step. -/
inductive StepResult : Type where
  | done
  | next (c : Cmd) (s : State) (t : List Obs)
  | fail (t : List Obs)
  | stuck
  deriving DecidableEq, Repr

/-- S-T12: how a run with bounded fuel ends. -/
inductive Outcome : Type where
  | halted (s : State)
  | failed
  | out_of_fuel (c : Cmd) (s : State)
  | wedged
  deriving DecidableEq, Repr

/-! ## Typing (S-F01) -/

/-- The join of two labels: `secret` when either is. -/
def Label.join : Label → Label → Label
  | .public, .public => .public
  | _, _ => .secret

/-- A value labelled `l` may be stored in a place labelled `l'`: `l` is
`public` or `l'` is `secret`. -/
def Label.flows : Label → Label → Bool
  | .public, _ => true
  | .secret, .secret => true
  | .secret, .public => false

/-- The type of a value. -/
def Val.ty : Val → Ty
  | .vword _ => .ty_word
  | .vbool _ => .ty_bool

/-- The typing rule shared by the four operators: two words give a result of
type `t` whose label is the join of the operands' labels. -/
def wordOpType (t : Ty) : Option (Ty × Label) → Option (Ty × Label) → Option (Ty × Label)
  | some (.ty_word, l1), some (.ty_word, l2) => some (t, l1.join l2)
  | _, _ => none

/-- Expression typing: a type and a label, or `none`. -/
def typeOf (g : Env) : Expr → Option (Ty × Label)
  | .lit v => some (v.ty, .public)
  | .var x => g.vars[x]?.map fun d => (d.ty, d.label)
  | .add e1 e2 => wordOpType .ty_word (typeOf g e1) (typeOf g e2)
  | .xor e1 e2 => wordOpType .ty_word (typeOf g e1) (typeOf g e2)
  | .band e1 e2 => wordOpType .ty_word (typeOf g e1) (typeOf g e2)
  | .eq e1 e2 => wordOpType .ty_bool (typeOf g e1) (typeOf g e2)
  | .get a i =>
    match g.arrs[a]?, typeOf g i with
    | some d, some (.ty_word, .public) => some (.ty_word, d.label)
    | _, _ => none

/-- S-F01: the command typing of semantics.md 3.1. A public destination only
receives public data, indices and branch conditions are public, and
`declassify` is the one way to move data of any label into a public variable. -/
def well_typed (g : Env) : Cmd → Bool
  | .skip => true
  | .assign x e =>
    match g.vars[x]?, typeOf g e with
    | some d, some (t, l) => t == d.ty && l.flows d.label
    | _, _ => false
  | .set a i v =>
    match g.arrs[a]?, typeOf g i, typeOf g v with
    | some d, some (.ty_word, .public), some (.ty_word, l) => l.flows d.label
    | _, _, _ => false
  | .seq c1 c2 => well_typed g c1 && well_typed g c2
  | .cond e c1 c2 =>
    match typeOf g e with
    | some (.ty_bool, .public) => well_typed g c1 && well_typed g c2
    | _ => false
  | .loop _ c => well_typed g c
  | .declassify x e =>
    match g.vars[x]?, typeOf g e with
    | some d, some (t, _) => t == d.ty && d.label == .public
    | _, _ => false

/-! ## Evaluation (S-F04 to S-F06) -/

/-- The result of evaluating an expression: a value, a failure, or stuck,
with the observations made on the way. -/
inductive Eval : Type where
  | ok (v : Val) (t : List Obs)
  | fail (t : List Obs)
  | stuck

/-- Operands evaluate left to right, the first failure or stuck operand ends
evaluation (keeping the observations before it), and the operator applies
only to two words. -/
def Eval.wordOp (f : Word 8 → Word 8 → Val) : Eval → Eval → Eval
  | .ok v1 t1, .ok v2 t2 =>
    match v1, v2 with
    | .vword x, .vword y => .ok (f x y) (t1 ++ t2)
    | _, _ => .stuck
  | .ok _ t1, .fail t2 => .fail (t1 ++ t2)
  | .ok _ _, .stuck => .stuck
  | .fail t, _ => .fail t
  | .stuck, _ => .stuck

/-- Expression evaluation. A read inside the array appends `read a n`; one
outside it fails after appending `oob a n`. -/
def evalExpr (s : State) : Expr → Eval
  | .lit v => .ok v []
  | .var x =>
    match s.vars[x]? with
    | some v => .ok v []
    | none => .stuck
  | .add e1 e2 => .wordOp (fun x y => .vword (word_add x y)) (evalExpr s e1) (evalExpr s e2)
  | .xor e1 e2 => .wordOp (fun x y => .vword (word_xor x y)) (evalExpr s e1) (evalExpr s e2)
  | .band e1 e2 => .wordOp (fun x y => .vword (word_and x y)) (evalExpr s e1) (evalExpr s e2)
  | .eq e1 e2 => .wordOp (fun x y => .vbool (decide (x = y))) (evalExpr s e1) (evalExpr s e2)
  | .get a i =>
    match evalExpr s i with
    | .ok (.vword n) t =>
      match s.arrs[a]? with
      | some cells =>
        match cells[nat_of_word n]? with
        | some m => .ok (.vword m) (t ++ [.read a (nat_of_word n)])
        | none => .fail (t ++ [.oob a (nat_of_word n)])
      | none => .stuck
    | .ok (.vbool _) _ => .stuck
    | .fail t => .fail t
    | .stuck => .stuck

/-- The state with variable `x` set to `v`. -/
def State.setVar (s : State) (x : Nat) (v : Val) : State :=
  { s with vars := s.vars.set x v }

/-- The state with array `a` replaced by `cells`. -/
def State.setArr (s : State) (a : Nat) (cells : List (Word 8)) : State :=
  { s with arrs := s.arrs.set a cells }

/-- The step of `seq c1 c2` for `c1` other than `skip`, from the step of `c1`:
the new command is wrapped as `seq c1' c2`, a failure or `stuck` passes
through (and `done`, which only `skip` gives, cannot occur). -/
def StepResult.inSeq (c2 : Cmd) : StepResult → StepResult
  | .next c1' s' t => .next (.seq c1' c2) s' t
  | .fail t => .fail t
  | .done => .stuck
  | .stuck => .stuck

/-- S-F04: one small step (semantics.md 3.2). -/
def step (s : State) : Cmd → StepResult
  | .skip => .done
  | .assign x e =>
    match evalExpr s e with
    | .ok v t => if x < s.vars.length then .next .skip (s.setVar x v) t else .stuck
    | .fail t => .fail t
    | .stuck => .stuck
  | .set a i v =>
    match evalExpr s i with
    | .ok (.vword n) ti =>
      match evalExpr s v with
      | .ok (.vword m) tv =>
        match s.arrs[a]? with
        | some cells =>
          if nat_of_word n < cells.length then
            .next .skip (s.setArr a (cells.set (nat_of_word n) m))
              (ti ++ tv ++ [.write a (nat_of_word n)])
          else .fail (ti ++ tv ++ [.oob a (nat_of_word n)])
        | none => .stuck
      | .ok (.vbool _) _ => .stuck
      | .fail tv => .fail (ti ++ tv)
      | .stuck => .stuck
    | .ok (.vbool _) _ => .stuck
    | .fail t => .fail t
    | .stuck => .stuck
  | .seq .skip c2 => .next c2 s []
  | .seq c1 c2 => (step s c1).inSeq c2
  | .cond e c1 c2 =>
    match evalExpr s e with
    | .ok (.vbool b) t => .next (if b then c1 else c2) s (t ++ [.branch b])
    | .ok (.vword _) _ => .stuck
    | .fail t => .fail t
    | .stuck => .stuck
  | .loop 0 _ => .next .skip s []
  | .loop (n + 1) c => .next (.seq c (.loop n c)) s []
  | .declassify x e =>
    match evalExpr s e with
    | .ok v t =>
      if x < s.vars.length then .next .skip (s.setVar x v) (t ++ [.release v]) else .stuck
    | .fail t => .fail t
    | .stuck => .stuck

/-- Run with fuel `n`: the outcome and every observation made on the way. -/
def run : Nat → State → Cmd → Outcome × List Obs
  | 0, s, c => (.out_of_fuel c s, [])
  | n + 1, s, c =>
    match step s c with
    | .done => (.halted s, [])
    | .fail t => (.failed, t)
    | .stuck => (.wedged, [])
    | .next c' s' t =>
      match run n s' c' with
      | (o, t') => (o, t ++ t')

/-- S-F05: the observations of `run`. -/
def run_trace (n : Nat) (s : State) (c : Cmd) : List Obs := (run n s c).2

/-- S-F06: the outcome of `run`. -/
def run_outcome (n : Nat) (s : State) (c : Cmd) : Outcome := (run n s c).1

/-! ## Well-formedness and public equivalence (S-F02, S-F03, S-F07) -/

section Forall₂
variable {α β : Type} {R : α → β → Prop}

/-- `Forall₂ R xs ys`: the lists have the same length and `R` relates the
elements at each position. -/
def Forall₂ (R : α → β → Prop) : List α → List β → Prop
  | [], [] => True
  | a :: as, b :: bs => R a b ∧ Forall₂ R as bs
  | _, _ => False

/-- Decided position by position (structural recursion, so `decide` evaluates it). -/
instance Forall₂.instDecidable [∀ a b, Decidable (R a b)] :
    ∀ xs ys, Decidable (Forall₂ R xs ys)
  | [], [] => isTrue trivial
  | [], _ :: _ => isFalse nofun
  | _ :: _, [] => isFalse nofun
  | a :: as, b :: bs =>
    have := Forall₂.instDecidable as bs
    inferInstanceAs (Decidable (R a b ∧ Forall₂ R as bs))

end Forall₂

/-- S-F02: the state has exactly the declared variables and arrays, each
variable's value has its declared type and each array its declared size. -/
def wf (g : Env) (s : State) : Prop :=
  Forall₂ (fun d v => v.ty = d.ty) g.vars s.vars ∧
  Forall₂ (fun d cells => cells.length = d.size) g.arrs s.arrs

/-- `wf` is decidable, so `decide` checks it on closed states (D2-TH06). -/
instance (g : Env) (s : State) : Decidable (wf g s) :=
  inferInstanceAs (Decidable (_ ∧ _))

/-- S-F03: the two states agree on every declared public variable and every
declared public array. -/
def low_eq (g : Env) (s1 s2 : State) : Prop :=
  (∀ (x : Nat) (d : VarDecl), g.vars[x]? = some d → d.label = .public →
    s1.vars[x]? = s2.vars[x]?) ∧
  (∀ (a : Nat) (d : ArrDecl), g.arrs[a]? = some d → d.label = .public →
    s1.arrs[a]? = s2.arrs[a]?)

/-- S-F07: public equivalence of outcomes: both halted in publicly equivalent
states, both failed, both out of fuel at the same command in publicly
equivalent states, or both wedged. -/
def outcome_low_eq (g : Env) : Outcome → Outcome → Prop
  | .halted s1, .halted s2 => low_eq g s1 s2
  | .failed, .failed => True
  | .out_of_fuel c1 s1, .out_of_fuel c2 s2 => c1 = c2 ∧ low_eq g s1 s2
  | .wedged, .wedged => True
  | _, _ => False

/-! ## Lemmas: lists, labels, typing inversion -/

namespace Forall₂
variable {α β : Type} {R : α → β → Prop}

theorem length_eq : ∀ {xs : List α} {ys : List β}, Forall₂ R xs ys → xs.length = ys.length
  | [], [], _ => rfl
  | _ :: _, _ :: _, h => congrArg (· + 1) (length_eq h.2)
  | [], _ :: _, h => False.elim h
  | _ :: _, [], h => False.elim h

/-- Where the left list has an element, the right one has a related one. -/
theorem getElem? : ∀ {xs : List α} {ys : List β} {i : Nat} {a : α},
    Forall₂ R xs ys → xs[i]? = some a → ∃ b, ys[i]? = some b ∧ R a b
  | [], _, _, _, _, hx => by simp at hx
  | _ :: _, [], _, _, h, _ => False.elim h
  | _ :: _, b :: _, 0, _, h, hx => by
    simp only [List.getElem?_cons_zero, Option.some.injEq] at hx
    subst hx
    exact ⟨b, rfl, h.1⟩
  | _ :: xs, _ :: ys, _ + 1, _, h, hx => getElem? (xs := xs) (ys := ys) h.2 hx

/-- Replacing a right element by one related to the left element keeps the relation. -/
theorem set : ∀ {xs : List α} {ys : List β} {i : Nat} {a : α} {b : β},
    Forall₂ R xs ys → xs[i]? = some a → R a b → Forall₂ R xs (ys.set i b)
  | [], _, _, _, _, _, hx, _ => by simp at hx
  | _ :: _, [], _, _, _, h, _, _ => False.elim h
  | _ :: _, _ :: _, 0, _, _, h, hx, hr => by
    simp only [List.getElem?_cons_zero, Option.some.injEq] at hx
    subst hx
    exact ⟨hr, h.2⟩
  | _ :: xs, _ :: ys, _ + 1, _, _, h, hx, hr => ⟨h.1, set (xs := xs) (ys := ys) h.2 hx hr⟩

end Forall₂

theorem lt_length_of_getElem? {α : Type} {l : List α} {i : Nat} {a : α} (h : l[i]? = some a) :
    i < l.length :=
  (List.getElem?_eq_some_iff.1 h).1

theorem Label.join_eq_public {l1 l2 : Label} (h : l1.join l2 = .public) :
    l1 = .public ∧ l2 = .public := by
  cases l1 <;> cases l2 <;> simp_all [Label.join]

theorem Label.eq_public_of_flows {l : Label} (h : l.flows .public = true) : l = .public := by
  cases l <;> simp_all [Label.flows]

theorem typeOf_var {g : Env} {x : Nat} {t : Ty} {l : Label} (h : typeOf g (.var x) = some (t, l)) :
    ∃ d, g.vars[x]? = some d ∧ d.ty = t ∧ d.label = l := by
  simpa [typeOf] using h

theorem wordOpType_eq_some {t t' : Ty} {l : Label} {o1 o2 : Option (Ty × Label)}
    (h : wordOpType t o1 o2 = some (t', l)) :
    ∃ l1 l2, o1 = some (.ty_word, l1) ∧ o2 = some (.ty_word, l2) ∧ t' = t ∧
      l = l1.join l2 := by
  unfold wordOpType at h
  split at h
  · next l1 l2 =>
    simp only [Option.some.injEq, Prod.mk.injEq] at h
    exact ⟨l1, l2, rfl, rfl, h.1.symm, h.2.symm⟩
  · simp at h

theorem typeOf_get {g : Env} {a : Nat} {i : Expr} {t : Ty} {l : Label}
    (h : typeOf g (.get a i) = some (t, l)) :
    ∃ d, g.arrs[a]? = some d ∧ typeOf g i = some (.ty_word, .public) ∧
      t = .ty_word ∧ l = d.label := by
  simp only [typeOf] at h
  split at h
  · next d hd hi =>
    simp only [Option.some.injEq, Prod.mk.injEq] at h
    exact ⟨d, hd, hi, h.1.symm, h.2.symm⟩
  · simp at h

section WellTyped
variable {g : Env}

theorem wt_assign {x : Nat} {e : Expr} (h : well_typed g (.assign x e) = true) :
    ∃ d t l, g.vars[x]? = some d ∧ typeOf g e = some (t, l) ∧ t = d.ty ∧
      l.flows d.label = true := by
  simp only [well_typed] at h
  split at h
  · next d t l hd he =>
    simp only [Bool.and_eq_true, beq_iff_eq] at h
    exact ⟨d, t, l, hd, he, h.1, h.2⟩
  · simp at h

theorem wt_set {a : Nat} {i v : Expr} (h : well_typed g (.set a i v) = true) :
    ∃ d l, g.arrs[a]? = some d ∧ typeOf g i = some (.ty_word, .public) ∧
      typeOf g v = some (.ty_word, l) ∧ l.flows d.label = true := by
  simp only [well_typed] at h
  split at h
  · next d l hd hi hv => exact ⟨d, l, hd, hi, hv, h⟩
  · simp at h

theorem wt_seq {c1 c2 : Cmd} (h : well_typed g (.seq c1 c2) = true) :
    well_typed g c1 = true ∧ well_typed g c2 = true := by
  simpa [well_typed] using h

theorem wt_cond {e : Expr} {c1 c2 : Cmd} (h : well_typed g (.cond e c1 c2) = true) :
    typeOf g e = some (.ty_bool, .public) ∧ well_typed g c1 = true ∧
      well_typed g c2 = true := by
  simp only [well_typed] at h
  split at h
  · next he =>
    simp only [Bool.and_eq_true] at h
    exact ⟨he, h⟩
  · simp at h

theorem wt_declassify {x : Nat} {e : Expr} (h : well_typed g (.declassify x e) = true) :
    ∃ d t l, g.vars[x]? = some d ∧ typeOf g e = some (t, l) ∧ t = d.ty ∧
      d.label = .public := by
  simp only [well_typed] at h
  split at h
  · next d t l hd he =>
    simp only [Bool.and_eq_true, beq_iff_eq] at h
    exact ⟨d, t, l, hd, he, h.1, h.2⟩
  · simp at h

end WellTyped

/-! ## Lemmas: well-formed states and public equivalence -/

section States
variable {g : Env} {s s1 s2 : State}

theorem wf.var {x : Nat} {d : VarDecl} (w : wf g s) (hd : g.vars[x]? = some d) :
    ∃ v, s.vars[x]? = some v ∧ v.ty = d.ty :=
  w.1.getElem? hd

theorem wf.arr {a : Nat} {d : ArrDecl} (w : wf g s) (hd : g.arrs[a]? = some d) :
    ∃ cells, s.arrs[a]? = some cells ∧ cells.length = d.size :=
  w.2.getElem? hd

theorem wf.setVar {x : Nat} {d : VarDecl} {v : Val} (w : wf g s) (hd : g.vars[x]? = some d)
    (hv : v.ty = d.ty) : wf g (s.setVar x v) :=
  ⟨w.1.set hd hv, w.2⟩

theorem wf.setArr {a : Nat} {d : ArrDecl} {cells : List (Word 8)} (w : wf g s)
    (hd : g.arrs[a]? = some d) (hc : cells.length = d.size) : wf g (s.setArr a cells) :=
  ⟨w.1, w.2.set hd hc⟩

theorem wf.vars_length (w1 : wf g s1) (w2 : wf g s2) : s1.vars.length = s2.vars.length :=
  w1.1.length_eq.symm.trans w2.1.length_eq

theorem wf.arrs_length (w1 : wf g s1) (w2 : wf g s2) : s1.arrs.length = s2.arrs.length :=
  w1.2.length_eq.symm.trans w2.2.length_eq

theorem low_eq_refl (g : Env) (s : State) : low_eq g s s :=
  ⟨fun _ _ _ _ => rfl, fun _ _ _ _ => rfl⟩

/-- Setting a variable keeps public equivalence when the two new values are
equal whenever the variable is public. -/
theorem low_eq.setVar {x : Nat} {d : VarDecl} {v1 v2 : Val} (h : low_eq g s1 s2)
    (hlen : s1.vars.length = s2.vars.length) (hd : g.vars[x]? = some d)
    (hv : d.label = .public → v1 = v2) : low_eq g (s1.setVar x v1) (s2.setVar x v2) := by
  refine ⟨fun y d' hy hp => ?_, h.2⟩
  simp only [State.setVar, List.getElem?_set]
  by_cases hxy : x = y
  · subst hxy
    rw [hd, Option.some.injEq] at hy
    subst hy
    simp [hv hp, hlen]
  · simp only [hxy, ite_false]
    exact h.1 y d' hy hp

/-- Replacing an array keeps public equivalence when the two new contents are
equal whenever the array is public. -/
theorem low_eq.setArr {a : Nat} {d : ArrDecl} {c1 c2 : List (Word 8)} (h : low_eq g s1 s2)
    (hlen : s1.arrs.length = s2.arrs.length) (hd : g.arrs[a]? = some d)
    (hc : d.label = .public → c1 = c2) : low_eq g (s1.setArr a c1) (s2.setArr a c2) := by
  refine ⟨h.1, fun b d' hb hp => ?_⟩
  simp only [State.setArr, List.getElem?_set]
  by_cases hab : a = b
  · subst hab
    rw [hd, Option.some.injEq] at hb
    subst hb
    simp [hc hp, hlen]
  · simp only [hab, ite_false]
    exact h.2 b d' hb hp

end States

/-! ## Expressions in two publicly equivalent states -/

/-- What typing guarantees about evaluating an expression of type `t` and
label `l` in two well-formed, publicly equivalent states: both succeed with
the same observations and values of type `t`, equal when `l` is public, or
both fail with the same observations. -/
def Agree (t : Ty) (l : Label) : Eval → Eval → Prop
  | .ok v1 o1, .ok v2 o2 => o1 = o2 ∧ v1.ty = t ∧ v2.ty = t ∧ (l = .public → v1 = v2)
  | .fail o1, .fail o2 => o1 = o2
  | _, _ => False

theorem Val.eq_vword {v : Val} (h : v.ty = .ty_word) : ∃ x, v = .vword x := by
  cases v with
  | vword x => exact ⟨x, rfl⟩
  | vbool _ => simp [Val.ty] at h

theorem Val.eq_vbool {v : Val} (h : v.ty = .ty_bool) : ∃ b, v = .vbool b := by
  cases v with
  | vword _ => simp [Val.ty] at h
  | vbool b => exact ⟨b, rfl⟩

theorem Agree.wordOp {f : Word 8 → Word 8 → Val} {t : Ty} (hf : ∀ x y, (f x y).ty = t)
    {l1 l2 : Label} {r1 r1' r2 r2' : Eval}
    (h1 : Agree .ty_word l1 r1 r1') (h2 : Agree .ty_word l2 r2 r2') :
    Agree t (l1.join l2) (Eval.wordOp f r1 r2) (Eval.wordOp f r1' r2') := by
  cases r1 <;> cases r1' <;> simp only [Agree] at h1
  case ok.ok v1 o1 v1' o1' =>
    obtain ⟨rfl, hv1, hv1', hp1⟩ := h1
    obtain ⟨x, rfl⟩ := Val.eq_vword hv1
    obtain ⟨x', rfl⟩ := Val.eq_vword hv1'
    cases r2 <;> cases r2' <;> simp only [Agree] at h2
    case ok.ok v2 o2 v2' o2' =>
      obtain ⟨rfl, hv2, hv2', hp2⟩ := h2
      obtain ⟨y, rfl⟩ := Val.eq_vword hv2
      obtain ⟨y', rfl⟩ := Val.eq_vword hv2'
      refine ⟨rfl, hf x y, hf x' y', fun hp => ?_⟩
      obtain ⟨hp1', hp2'⟩ := Label.join_eq_public hp
      cases hp1 hp1'
      cases hp2 hp2'
      rfl
    case fail.fail o2 o2' =>
      subst h2
      exact rfl
  case fail.fail o1 o1' =>
    subst h1
    exact rfl

/-- Typed expressions evaluate alike in two well-formed, publicly equivalent
states. -/
theorem eval_agree {g : Env} {s1 s2 : State} (w1 : wf g s1) (w2 : wf g s2)
    (hlow : low_eq g s1 s2) :
    ∀ {e : Expr} {t : Ty} {l : Label}, typeOf g e = some (t, l) →
      Agree t l (evalExpr s1 e) (evalExpr s2 e)
  | .lit v, t, l, h => by
    simp only [typeOf, Option.some.injEq, Prod.mk.injEq] at h
    obtain ⟨rfl, rfl⟩ := h
    exact ⟨rfl, rfl, rfl, fun _ => rfl⟩
  | .var x, t, l, h => by
    obtain ⟨d, hd, rfl, rfl⟩ := typeOf_var h
    obtain ⟨v1, hv1, ht1⟩ := w1.var hd
    obtain ⟨v2, hv2, ht2⟩ := w2.var hd
    simp only [evalExpr, hv1, hv2]
    refine ⟨rfl, ht1, ht2, fun hp => ?_⟩
    have := hlow.1 x d hd hp
    rw [hv1, hv2, Option.some.injEq] at this
    exact this
  | .add e1 e2, t, l, h => by
    obtain ⟨l1, l2, h1, h2, rfl, rfl⟩ := wordOpType_eq_some h
    exact Agree.wordOp (f := fun x y => .vword (word_add x y)) (fun _ _ => rfl)
      (eval_agree w1 w2 hlow h1) (eval_agree w1 w2 hlow h2)
  | .xor e1 e2, t, l, h => by
    obtain ⟨l1, l2, h1, h2, rfl, rfl⟩ := wordOpType_eq_some h
    exact Agree.wordOp (f := fun x y => .vword (word_xor x y)) (fun _ _ => rfl)
      (eval_agree w1 w2 hlow h1) (eval_agree w1 w2 hlow h2)
  | .band e1 e2, t, l, h => by
    obtain ⟨l1, l2, h1, h2, rfl, rfl⟩ := wordOpType_eq_some h
    exact Agree.wordOp (f := fun x y => .vword (word_and x y)) (fun _ _ => rfl)
      (eval_agree w1 w2 hlow h1) (eval_agree w1 w2 hlow h2)
  | .eq e1 e2, t, l, h => by
    obtain ⟨l1, l2, h1, h2, rfl, rfl⟩ := wordOpType_eq_some h
    exact Agree.wordOp (f := fun x y => .vbool (decide (x = y))) (fun _ _ => rfl)
      (eval_agree w1 w2 hlow h1) (eval_agree w1 w2 hlow h2)
  | .get a i, t, l, h => by
    obtain ⟨d, hd, hi, rfl, rfl⟩ := typeOf_get h
    obtain ⟨c1, hc1, hl1⟩ := w1.arr hd
    obtain ⟨c2, hc2, hl2⟩ := w2.arr hd
    have ih := eval_agree w1 w2 hlow hi
    simp only [evalExpr]
    revert ih
    cases evalExpr s1 i <;> cases evalExpr s2 i <;> intro ih <;> simp only [Agree] at ih
    case ok.ok v1 o1 v2 o2 =>
      obtain ⟨rfl, hv1, -, hp⟩ := ih
      cases hp (by simp)
      obtain ⟨n, rfl⟩ := Val.eq_vword hv1
      simp only [hc1, hc2]
      by_cases hn : nat_of_word n < d.size
      · rw [List.getElem?_eq_getElem (hl1 ▸ hn), List.getElem?_eq_getElem (hl2 ▸ hn)]
        refine ⟨rfl, rfl, rfl, fun hp => ?_⟩
        have := hlow.2 a d hd hp
        rw [hc1, hc2, Option.some.injEq] at this
        subst this
        rfl
      · rw [List.getElem?_eq_none (by omega), List.getElem?_eq_none (by omega)]
        exact rfl
    case fail.fail o1 o2 =>
      exact ih

/-! ## One step in two publicly equivalent states -/

/-- What typing guarantees about one step of a command in two well-formed,
publicly equivalent states: both are `done`; both fail with the same
observations; or both continue with the same well-typed command and
well-formed states, either with the same observations and publicly
equivalent states, or with observations that differ only in a final
release of two different values. -/
def StepAgree (g : Env) : StepResult → StepResult → Prop
  | .done, .done => True
  | .fail t1, .fail t2 => t1 = t2
  | .next c1 s1 t1, .next c2 s2 t2 =>
    c1 = c2 ∧ well_typed g c1 = true ∧ wf g s1 ∧ wf g s2 ∧
      ((t1 = t2 ∧ low_eq g s1 s2) ∨
        ∃ p v1 v2, t1 = p ++ [.release v1] ∧ t2 = p ++ [.release v2] ∧ v1 ≠ v2)
  | _, _ => False

theorem step_seq_skip (s : State) (c2 : Cmd) : step s (.seq .skip c2) = .next c2 s [] := rfl

theorem step_seq {s : State} {c1 c2 : Cmd} (h : c1 ≠ .skip) :
    step s (.seq c1 c2) = (step s c1).inSeq c2 := by
  cases c1 <;> first | exact absurd rfl h | rfl

/-- Only `skip` is `done`. -/
theorem step_ne_done {s : State} {c : Cmd} (h : c ≠ .skip) : step s c ≠ .done := by
  intro hd
  cases c with
  | skip => exact h rfl
  | seq c1 c2 =>
    by_cases h1 : c1 = .skip
    · subst h1; simp [step] at hd
    · rw [step_seq h1] at hd
      generalize step s c1 = r at hd
      cases r <;> simp [StepResult.inSeq] at hd
  | loop n c => cases n <;> simp [step] at hd
  | _ =>
    simp only [step] at hd
    repeat' split at hd
    all_goals simp at hd

theorem StepAgree.inSeq {g : Env} {c2 : Cmd} (hc2 : well_typed g c2 = true) :
    ∀ {r1 r2 : StepResult}, r1 ≠ .done → StepAgree g r1 r2 →
      StepAgree g (r1.inSeq c2) (r2.inSeq c2)
  | .next c1 _ _, .next _ _ _, _, ⟨rfl, hc1, w1, w2, h⟩ =>
    ⟨rfl, by simp [well_typed, hc1, hc2], w1, w2, h⟩
  | .fail _, .fail _, _, h => h
  | .done, _, hd, _ => absurd rfl hd

/-- The step lemma: a well-typed command steps alike in two well-formed,
publicly equivalent states. It gives progress (with two equal states),
preservation and lock-step noninterference. -/
theorem step_agree {g : Env} {s1 s2 : State} (w1 : wf g s1) (w2 : wf g s2)
    (hlow : low_eq g s1 s2) :
    ∀ {c : Cmd}, well_typed g c = true → StepAgree g (step s1 c) (step s2 c)
  | .skip, _ => trivial
  | .assign x e, hc => by
    obtain ⟨d, t, l, hd, he, rfl, hfl⟩ := wt_assign hc
    obtain ⟨_, hx1, -⟩ := w1.var hd
    obtain ⟨_, hx2, -⟩ := w2.var hd
    have ha := eval_agree w1 w2 hlow he
    simp only [step]
    revert ha
    cases evalExpr s1 e <;> cases evalExpr s2 e <;> intro ha <;> simp only [Agree] at ha
    case ok.ok v1 o1 v2 o2 =>
      obtain ⟨rfl, hv1, hv2, hp⟩ := ha
      dsimp only
      rw [ite_eq_left (lt_length_of_getElem? hx1), ite_eq_left (lt_length_of_getElem? hx2)]
      refine ⟨rfl, rfl, w1.setVar hd hv1, w2.setVar hd hv2, .inl ⟨rfl, ?_⟩⟩
      exact hlow.setVar (w1.vars_length w2) hd
        (fun hpub => hp (Label.eq_public_of_flows (hpub ▸ hfl)))
    case fail.fail o1 o2 => exact ha
  | .declassify x e, hc => by
    obtain ⟨d, t, l, hd, he, rfl, hpub⟩ := wt_declassify hc
    obtain ⟨_, hx1, -⟩ := w1.var hd
    obtain ⟨_, hx2, -⟩ := w2.var hd
    have ha := eval_agree w1 w2 hlow he
    simp only [step]
    revert ha
    cases evalExpr s1 e <;> cases evalExpr s2 e <;> intro ha <;> simp only [Agree] at ha
    case ok.ok v1 o1 v2 o2 =>
      obtain ⟨rfl, hv1, hv2, -⟩ := ha
      dsimp only
      rw [ite_eq_left (lt_length_of_getElem? hx1), ite_eq_left (lt_length_of_getElem? hx2)]
      refine ⟨rfl, rfl, w1.setVar hd hv1, w2.setVar hd hv2, ?_⟩
      by_cases hv : v1 = v2
      · subst hv
        exact .inl ⟨rfl, hlow.setVar (w1.vars_length w2) hd fun _ => rfl⟩
      · exact .inr ⟨o1, v1, v2, rfl, rfl, hv⟩
    case fail.fail o1 o2 => exact ha
  | .set a i v, hc => by
    obtain ⟨d, l, hd, hi, hv, hfl⟩ := wt_set hc
    obtain ⟨c1, hc1, hl1⟩ := w1.arr hd
    obtain ⟨c2, hc2, hl2⟩ := w2.arr hd
    have hai := eval_agree w1 w2 hlow hi
    have hav := eval_agree w1 w2 hlow hv
    simp only [step]
    revert hai
    cases evalExpr s1 i <;> cases evalExpr s2 i <;> intro hai <;> simp only [Agree] at hai
    case ok.ok vi1 oi1 vi2 oi2 =>
      obtain ⟨rfl, hvi1, -, hpi⟩ := hai
      cases hpi (by simp)
      obtain ⟨n, rfl⟩ := Val.eq_vword hvi1
      simp only
      revert hav
      cases evalExpr s1 v <;> cases evalExpr s2 v <;> intro hav <;> simp only [Agree] at hav
      case ok.ok vv1 ov1 vv2 ov2 =>
        obtain ⟨rfl, hvv1, hvv2, hpv⟩ := hav
        obtain ⟨m1, rfl⟩ := Val.eq_vword hvv1
        obtain ⟨m2, rfl⟩ := Val.eq_vword hvv2
        simp only [hc1, hc2]
        by_cases hn : nat_of_word n < d.size
        · rw [ite_eq_left (hl1 ▸ hn), ite_eq_left (hl2 ▸ hn)]
          refine ⟨rfl, rfl, w1.setArr hd (by simpa using hl1), w2.setArr hd (by simpa using hl2),
            .inl ⟨rfl, ?_⟩⟩
          refine hlow.setArr (w1.arrs_length w2) hd fun hpub => ?_
          have hcells := hlow.2 a d hd hpub
          rw [hc1, hc2, Option.some.injEq] at hcells
          cases hpv (Label.eq_public_of_flows (hpub ▸ hfl))
          rw [hcells]
        · rw [ite_eq_right (hl1 ▸ hn), ite_eq_right (hl2 ▸ hn)]
          exact rfl
      case fail.fail ov1 ov2 =>
        subst hav
        exact rfl
    case fail.fail o1 o2 => exact hai
  | .seq c1 c2, hc => by
    obtain ⟨hc1, hc2⟩ := wt_seq hc
    by_cases h1 : c1 = .skip
    · subst h1
      rw [step_seq_skip, step_seq_skip]
      exact ⟨rfl, hc2, w1, w2, .inl ⟨rfl, hlow⟩⟩
    · rw [step_seq h1, step_seq h1]
      exact StepAgree.inSeq hc2 (step_ne_done h1) (step_agree w1 w2 hlow hc1)
  | .cond e c1 c2, hc => by
    obtain ⟨he, hc1, hc2⟩ := wt_cond hc
    have ha := eval_agree w1 w2 hlow he
    simp only [step]
    revert ha
    cases evalExpr s1 e <;> cases evalExpr s2 e <;> intro ha <;> simp only [Agree] at ha
    case ok.ok v1 o1 v2 o2 =>
      obtain ⟨rfl, hv1, -, hp⟩ := ha
      cases hp (by simp)
      obtain ⟨b, rfl⟩ := Val.eq_vbool hv1
      refine ⟨rfl, ?_, w1, w2, .inl ⟨rfl, hlow⟩⟩
      cases b <;> assumption
    case fail.fail o1 o2 => exact ha
  | .loop 0 c, _ => ⟨rfl, rfl, w1, w2, .inl ⟨rfl, hlow⟩⟩
  | .loop (n + 1) c, hc => ⟨rfl, by simpa [well_typed] using hc, w1, w2, .inl ⟨rfl, hlow⟩⟩

/-! ## D2-TH01 to D2-TH03 -/

/-- D2-TH01: a well-typed command never gets stuck in a well-formed state. -/
theorem progress (g : Env) (c : Cmd) (s : State) :
    well_typed g c = true → wf g s → step s c ≠ .stuck := by
  intro hc w hstuck
  have h := step_agree w w (low_eq_refl g s) hc
  rw [hstuck] at h
  exact h

/-- D2-TH02: a step keeps the command well typed and the state well formed. -/
theorem preservation (g : Env) (c : Cmd) (s : State) (c' : Cmd) (s' : State) (t : List Obs) :
    well_typed g c = true → wf g s → step s c = .next c' s' t →
      well_typed g c' = true ∧ wf g s' := by
  intro hc w hstep
  have h := step_agree w w (low_eq_refl g s) hc
  rw [hstep] at h
  exact ⟨h.2.1, h.2.2.1⟩

/-- D2-TH03: one step of a well-typed command from two well-formed, publicly
equivalent states makes the same observations and reaches publicly
equivalent states, unless it releases two different values. -/
theorem lockstep_noninterference (g : Env) (c : Cmd) (s1 s2 : State) :
    well_typed g c = true → wf g s1 → wf g s2 → low_eq g s1 s2 →
      (step s1 c = .done ∧ step s2 c = .done) ∨
      (∃ t, step s1 c = .fail t ∧ step s2 c = .fail t) ∨
      (∃ c' s1' s2' t, step s1 c = .next c' s1' t ∧ step s2 c = .next c' s2' t ∧
        low_eq g s1' s2') ∨
      (∃ c' s1' s2' p v1 v2, step s1 c = .next c' s1' (p ++ [.release v1]) ∧
        step s2 c = .next c' s2' (p ++ [.release v2]) ∧ v1 ≠ v2) := by
  intro hc w1 w2 hlow
  have h := step_agree w1 w2 hlow hc
  revert h
  cases step s1 c <;> cases step s2 c <;> intro h <;> simp only [StepAgree] at h
  case done.done => exact .inl ⟨rfl, rfl⟩
  case fail.fail t1 t2 => exact .inr (.inl ⟨t1, rfl, h ▸ rfl⟩)
  case next.next c1 s1' t1 c2 s2' t2 =>
    obtain ⟨rfl, -, -, -, h⟩ := h
    rcases h with ⟨rfl, hl⟩ | ⟨p, v1, v2, rfl, rfl, hv⟩
    · exact .inr (.inr (.inl ⟨c1, s1', s2', t1, rfl, rfl, hl⟩))
    · exact .inr (.inr (.inr ⟨c1, s1', s2', p, v1, v2, rfl, rfl, hv⟩))

/-! ## D2-TH04: runs -/

section Runs
variable {n : Nat}

theorem run_trace_zero (s : State) (c : Cmd) : run_trace 0 s c = [] := rfl

theorem run_outcome_zero (s : State) (c : Cmd) : run_outcome 0 s c = .out_of_fuel c s := rfl

theorem run_done {s : State} {c : Cmd} (h : step s c = .done) :
    run_trace (n + 1) s c = [] ∧ run_outcome (n + 1) s c = .halted s := by
  simp [run_trace, run_outcome, run, h]

theorem run_fail {s : State} {c : Cmd} {t : List Obs} (h : step s c = .fail t) :
    run_trace (n + 1) s c = t ∧ run_outcome (n + 1) s c = .failed := by
  simp [run_trace, run_outcome, run, h]

theorem run_next {s s' : State} {c c' : Cmd} {t : List Obs} (h : step s c = .next c' s' t) :
    run_trace (n + 1) s c = t ++ run_trace n s' c' ∧
      run_outcome (n + 1) s c = run_outcome n s' c' := by
  simp [run_trace, run_outcome, run, h]

end Runs

/-- D2-TH04: two runs of a well-typed command from well-formed, publicly
equivalent states make the same observations and end in publicly equivalent
outcomes, or their observations agree up to a release of two different
values. -/
theorem run_noninterference (n : Nat) (g : Env) (c : Cmd) (s1 s2 : State) :
    well_typed g c = true → wf g s1 → wf g s2 → low_eq g s1 s2 →
      (run_trace n s1 c = run_trace n s2 c ∧
        outcome_low_eq g (run_outcome n s1 c) (run_outcome n s2 c)) ∨
      (∃ p v1 v2 r1 r2, run_trace n s1 c = p ++ .release v1 :: r1 ∧
        run_trace n s2 c = p ++ .release v2 :: r2 ∧ v1 ≠ v2) := by
  induction n generalizing c s1 s2 with
  | zero => intro _ _ _ hlow; exact .inl ⟨rfl, rfl, hlow⟩
  | succ n ih =>
    intro hc w1 w2 hlow
    have h := step_agree w1 w2 hlow hc
    revert h
    cases h1 : step s1 c <;> cases h2 : step s2 c <;> intro h <;> simp only [StepAgree] at h
    case done.done =>
      obtain ⟨e1, o1⟩ := run_done (n := n) h1
      obtain ⟨e2, o2⟩ := run_done (n := n) h2
      exact .inl ⟨by rw [e1, e2], by rw [o1, o2]; exact hlow⟩
    case fail.fail t1 t2 =>
      obtain ⟨e1, o1⟩ := run_fail (n := n) h1
      obtain ⟨e2, o2⟩ := run_fail (n := n) h2
      exact .inl ⟨by rw [e1, e2, h], by rw [o1, o2]; trivial⟩
    case next.next c1 s1' t1 c2 s2' t2 =>
      obtain ⟨rfl, hc', w1', w2', h⟩ := h
      obtain ⟨e1, o1⟩ := run_next (n := n) h1
      obtain ⟨e2, o2⟩ := run_next (n := n) h2
      rw [e1, e2, o1, o2]
      rcases h with ⟨rfl, hl⟩ | ⟨p, v1, v2, rfl, rfl, hv⟩
      · rcases ih c1 s1' s2' hc' w1' w2' hl with
          ⟨et, eo⟩ | ⟨p, v1, v2, r1, r2, e1', e2', hv⟩
        · exact .inl ⟨by rw [et], eo⟩
        · exact .inr ⟨t1 ++ p, v1, v2, r1, r2, by rw [e1', List.append_assoc],
            by rw [e2', List.append_assoc], hv⟩
      · exact .inr ⟨p, v1, v2, run_trace n s1' c1, run_trace n s2' c1, by simp, by simp, hv⟩

/-! ## D2-TH05 and D2-TH06: the suite's programs -/

/-- Γ+: `key` (secret word), `nonce` (public word), `acc` (secret word), `ok`
(public bool); `table` (4 public words), `state` (4 secret words). -/
def gamma_plus : Env :=
  .env [.vdecl .ty_word .secret, .vdecl .ty_word .public, .vdecl .ty_word .secret,
      .vdecl .ty_bool .public]
    [.adecl 4 .public, .adecl 4 .secret]

/-- The word literal `n`. -/
abbrev litw (n : Nat) : Expr := .lit (.vword (word_of_nat 8 n))

/-- P+, the positive program: mixes the key into `acc`, writes `state` at a
public index, loops twice, branches on the public nonce and releases whether
`acc` equals `key`. -/
def p_plus : Cmd :=
  .seq (.assign 2 (.xor (.var 0) (.var 1)))
    (.seq (.set 1 (.band (.var 1) (litw 3)) (.add (.var 2) (.get 0 (.band (.var 1) (litw 3)))))
      (.seq (.loop 2 (.assign 2 (.add (.var 2) (.var 0))))
        (.seq (.cond (.eq (.var 1) (litw 7)) (.assign 2 (.xor (.var 2) (.var 1))) .skip)
          (.declassify 3 (.eq (.var 2) (.var 0))))))

/-- D2-TH05: P+ is well typed, so D2-TH04 applies to it. -/
theorem positive_program_witness (n : Nat) (s1 s2 : State) :
    wf gamma_plus s1 → wf gamma_plus s2 → low_eq gamma_plus s1 s2 →
      (run_trace n s1 p_plus = run_trace n s2 p_plus ∧
        outcome_low_eq gamma_plus (run_outcome n s1 p_plus) (run_outcome n s2 p_plus)) ∨
      (∃ p v1 v2 r1 r2, run_trace n s1 p_plus = p ++ .release v1 :: r1 ∧
        run_trace n s2 p_plus = p ++ .release v2 :: r2 ∧ v1 ≠ v2) :=
  run_noninterference n gamma_plus p_plus s1 s2 (by decide)

/-- P-, the negative program: branches on the secret key. -/
def p_minus : Cmd :=
  .cond (.eq (.var 0) (litw 0)) (.assign 2 (litw 1)) (.assign 2 (litw 2))

/-- σa and σb differ only in the secret `key` (0 and 1). -/
def sigma (key : Nat) : State :=
  .state [.vword (word_of_nat 8 key), .vword (word_of_nat 8 7), .vword (word_of_nat 8 0),
      .vbool false]
    [[word_of_nat 8 16, word_of_nat 8 32, word_of_nat 8 48, word_of_nat 8 64],
      [word_of_nat 8 0, word_of_nat 8 0, word_of_nat 8 0, word_of_nat 8 0]]

theorem low_eq_sigma : low_eq gamma_plus (sigma 0) (sigma 1) := by
  refine ⟨fun x d hd hp => ?_, fun _ _ _ _ => rfl⟩
  cases x with
  | zero =>
    simp only [gamma_plus, List.getElem?_cons_zero, Option.some.injEq] at hd
    subst hd
    cases hp
  | succ x => rfl

/-- D2-TH06: P- (which `well_typed` rejects) does leak: σa and σb are
well formed and publicly equivalent, and the traces of P- from them differ
at the branch on the key. -/
theorem negative_program_leaks :
    wf gamma_plus (sigma 0) ∧ wf gamma_plus (sigma 1) ∧
      low_eq gamma_plus (sigma 0) (sigma 1) ∧
      ∃ p r1 r2, run_trace 8 (sigma 0) p_minus = p ++ .branch true :: r1 ∧
        run_trace 8 (sigma 1) p_minus = p ++ .branch false :: r2 :=
  ⟨by decide, by decide, low_eq_sigma, [], [], [], by decide, by decide⟩

end D006.Sieve
