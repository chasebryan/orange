(** D-006 v0.3, candidate C-01 (Rocq): DS-04, checking an LRAT certificate
    for a bit-vector obligation.

    Every shared B- symbol maps to one declaration here (see
    adapter.d/ds04.json). The module has four parts:

    - the two obligations, their meaning [holds] as equations between
      32-bit word terms of DS-01, and the canonical bit-blast of each into
      Tseitin clauses, printed by [cnf_text];
    - [lrat_verdict], a checker that reads a certificate byte by byte from a
      primitive string, one byte per step of a small machine, and checks
      every addition by reverse unit propagation against the blasted
      clauses, reporting the first failure with its code and line exactly as
      semantics.md, section 5, and the numbered rules fix it;
    - soundness: an accepted certificate makes the blasted clauses
      unsatisfiable (only the clause database invariant matters; the parser
      needs no proof), and the blast is correct: a pair of words that breaks
      the equation gives an assignment satisfying every clause;
    - the golden certificate, accepted by [vm_compute], which gives D4-TH01
      through the checker.

    Computation is by the bytecode VM ([vm_compute]); the checker uses
    primitive strings and 63-bit integers for the input and binary
    naturals, integers and [PositiveMap] tries for everything else. *)

From D006 Require Import Core.
From Stdlib Require Import NArith ZArith Bool List Lia.
From Stdlib Require Import FSets.FMapPositive.
From Stdlib Require PrimString.
From Corelib Require PrimInt63.
Import ListNotations PrimString.PStringNotations.

Local Open Scope N_scope.
Local Open Scope pstring_scope.

Local Abbreviation string := PrimString.string.
Local Abbreviation int := PrimInt63.int.

(** * Obligations and verdicts (B-T01, B-T02) *)

Inductive Obligation : Type := carry_save | carry_save_unshifted.

Inductive Verdict : Type :=
| accept
| reject (code : PrimString.string) (line : N).

(** * Meaning (B-F03) *)

(** Terms over two words [x] and [y] of 32 bits. *)
Inductive BV : Type :=
| Bx | By
| Badd (l r : BV) | Bxor (l r : BV) | Band (l r : BV)
| Bshl (e : BV) (k : N).

Fixpoint eval_bv (e : BV) (x y : Word 32) : Word 32 :=
  match e with
  | Bx => x
  | By => y
  | Badd l r => word_add 32 (eval_bv l x y) (eval_bv r x y)
  | Bxor l r => word_xor 32 (eval_bv l x y) (eval_bv r x y)
  | Band l r => word_and 32 (eval_bv l x y) (eval_bv r x y)
  | Bshl e k => word_shl 32 (eval_bv e x y) k
  end.

Definition lhs (o : Obligation) : BV := Badd Bx By.

Definition rhs (o : Obligation) : BV :=
  match o with
  | carry_save => Badd (Bxor Bx By) (Bshl (Band Bx By) 1)
  | carry_save_unshifted => Badd (Bxor Bx By) (Band Bx By)
  end.

Definition holds (o : Obligation) : Prop :=
  forall x y : Word 32, eval_bv (lhs o) x y = eval_bv (rhs o) x y.

(** * The canonical bit-blast *)

(** Variable 1 is constant false; bit [i] of [x] is variable [2 + i] and of
    [y] is variable [34 + i]; gates are numbered from 66 on. *)
Definition W : nat := 32.

Inductive GateKind : Type := Gand | Gxor | Gor.

Record Gate : Type := gate { gout : positive ; gkind : GateKind ; ga : positive ; gb : positive }.

Definition gate_fn (k : GateKind) (a b : bool) : bool :=
  match k with Gand => a && b | Gxor => xorb a b | Gor => a || b end.

(** The Tseitin clauses of a gate, in the order of the shared rules. *)
Definition gate_clauses (g : Gate) : list (list Z) :=
  let o := Zpos (gout g) in
  let a := Zpos (ga g) in
  let b := Zpos (gb g) in
  match gkind g with
  | Gand => [[-o; a]; [-o; b]; [o; -a; -b]]
  | Gxor => [[-o; a; b]; [-o; -a; -b]; [o; -a; b]; [o; a; -b]]
  | Gor => [[o; -a]; [o; -b]; [-o; a; b]]
  end%Z.

(** Blasting state: the last variable used and the gates made so far,
    newest first. *)
Record St : Type := st { top : positive ; gates : list Gate }.

Definition mk (k : GateKind) (a b : positive) (s : St) : positive * St :=
  let g := Pos.succ (top s) in (g, st g (gate g k a b :: gates s)).
Arguments mk : simpl never.

Fixpoint range (p : positive) (n : nat) : list positive :=
  match n with O => [] | S n' => p :: range (Pos.succ p) n' end.

(** One gate per bit, least significant first. *)
Fixpoint bitwise (k : GateKind) (a b : list positive) (s : St) : list positive * St :=
  match a, b with
  | a0 :: a', b0 :: b' =>
      let (g, s1) := mk k a0 b0 s in
      let (rest, s2) := bitwise k a' b' s1 in
      (g :: rest, s2)
  | _, _ => ([], s)
  end.

(** Ripple-carry addition: per bit t = XOR(a, b), s = XOR(t, c),
    u = AND(a, b), v = AND(c, t), c = OR(u, v). *)
Fixpoint adder (c : positive) (a b : list positive) (s : St) : list positive * St :=
  match a, b with
  | a0 :: a', b0 :: b' =>
      let (t, s1) := mk Gxor a0 b0 s in
      let (sum, s2) := mk Gxor t c s1 in
      let (u, s3) := mk Gand a0 b0 s2 in
      let (v, s4) := mk Gand c t s3 in
      let (c', s5) := mk Gor u v s4 in
      let (rest, s6) := adder c' a' b' s5 in
      (sum :: rest, s6)
  | _, _ => ([], s)
  end.

(** The bits of a term; the left operand is blasted before the right, and a
    shift makes no gates and shifts in variable 1. *)
Fixpoint bits (e : BV) (s : St) : list positive * St :=
  match e with
  | Bx => (range 2 W, s)
  | By => (range 34 W, s)
  | Badd l r =>
      let (a, s1) := bits l s in let (b, s2) := bits r s1 in adder 1 a b s2
  | Bxor l r =>
      let (a, s1) := bits l s in let (b, s2) := bits r s1 in bitwise Gxor a b s2
  | Band l r =>
      let (a, s1) := bits l s in let (b, s2) := bits r s1 in bitwise Gand a b s2
  | Bshl e k =>
      let (a, s1) := bits e s in (firstn W (repeat 1%positive (N.to_nat k) ++ a), s1)
  end.

Fixpoint or_chain (acc : positive) (ds : list positive) (s : St) : positive * St :=
  match ds with
  | [] => (acc, s)
  | d :: ds' => let (g, s1) := mk Gor acc d s in or_chain g ds' s1
  end.

(** The miter: d_i = XOR(lhs_i, rhs_i), then o = d_0 and o = OR(o, d_i). *)
Definition miter (l r : list positive) (s : St) : positive * St :=
  let (ds, s1) := bitwise Gxor l r s in
  match ds with
  | [] => (1%positive, s1)
  | d :: ds' => or_chain d ds' s1
  end.

(** The output variable and the final state. The counter starts at
    [1 + 2 * 32 = 65]. *)
Definition blast (o : Obligation) : positive * St :=
  let (l, s1) := bits (lhs o) (st 65 []) in
  let (r, s2) := bits (rhs o) s1 in
  miter l r s2.

Definition cnf_vars (o : Obligation) : positive := top (snd (blast o)).

(** Clause 1 is the negative unit of variable 1; the last clause is the
    unit of the miter's output. *)
Definition cnf_clauses (o : Obligation) : list (list Z) :=
  let (out, s) := blast o in
  [-1]%Z :: flat_map gate_clauses (rev (gates s)) ++ [[Zpos out]].

(** * Bytes as primitive integers *)

Module Chars.
  Import PrimInt63.

  Definition newline : int := 10.
  Definition space : int := 32.
  Definition minus : int := 45.
  Definition zero : int := 48.
  Definition nine : int := 57.
  Definition letter_d : int := 100.
  Definition one : int := 1.
  Definition start : int := 0.

  (** Certificates above 4 MiB are refused before parsing. *)
  Definition max_certificate : int := 4194304.

  Definition is_digit (c : int) : bool := leb zero c && leb c nine.

  (** The value of a decimal digit byte. *)
  Definition digit_value (c : int) : N :=
    if eqb c 48 then 0 else if eqb c 49 then 1 else if eqb c 50 then 2
    else if eqb c 51 then 3 else if eqb c 52 then 4 else if eqb c 53 then 5
    else if eqb c 54 then 6 else if eqb c 55 then 7 else if eqb c 56 then 8
    else 9.

  (** A positive as a primitive integer (used below 10 only). *)
  Fixpoint int_of_pos (p : positive) : int :=
    match p with
    | xH => 1
    | xO q => let i := int_of_pos q in add i i
    | xI q => let i := int_of_pos q in add (add i i) 1
    end.

  (** The byte of a decimal digit [d < 10]. *)
  Definition digit_byte (d : N) : int :=
    match d with N0 => zero | Npos p => add zero (int_of_pos p) end.
End Chars.

(** * The CNF text (B-F01) *)

Definition byte_string (c : int) : string := PrimString.make Chars.one c.

Definition nl : string := byte_string Chars.newline.

Fixpoint decimal_acc (fuel : nat) (n : N) (acc : string) : string :=
  match fuel with
  | O => acc
  | S fuel' =>
      let acc' := PrimString.cat (byte_string (Chars.digit_byte (n mod 10))) acc in
      if n <? 10 then acc' else decimal_acc fuel' (n / 10) acc'
  end.

(** A natural in decimal; it has at most as many digits as bits. *)
Definition decimal (n : N) : string := decimal_acc (S (N.size_nat n)) n "".

Definition literal_text (l : Z) : string :=
  match l with
  | Zneg p => PrimString.cat "-" (decimal (Npos p))
  | _ => decimal (Z.to_N l)
  end.

(** Literals separated by single spaces, then a space, 0 and a line feed. *)
Definition clause_line (c : list Z) : string :=
  match c with
  | [] => PrimString.cat " 0" nl
  | _ => fold_right (fun l acc => PrimString.cat (PrimString.cat (literal_text l) " ") acc)
                    (PrimString.cat "0" nl) c
  end.

(** Concatenation in balanced rounds, so no byte is copied more than
    [log2] of the number of pieces times. *)
Fixpoint pairs (l : list string) : list string :=
  match l with
  | a :: b :: rest => PrimString.cat a b :: pairs rest
  | _ => l
  end.

Fixpoint concat_rounds (n : nat) (l : list string) : string :=
  match l with
  | [] => ""
  | [a] => a
  | _ => match n with O => fold_right PrimString.cat "" l | S n' => concat_rounds n' (pairs l) end
  end.

Definition cnf_text (o : Obligation) : PrimString.string :=
  let clauses := cnf_clauses o in
  let header :=
    PrimString.cat (PrimString.cat (PrimString.cat (PrimString.cat "p cnf " (decimal (Npos (cnf_vars o)))) " ")
                                   (decimal (N.of_nat (length clauses)))) nl in
  concat_rounds 64 (header :: map clause_line clauses).

(** * Reverse unit propagation *)

Local Abbreviation Db := (PositiveMap.t (list Z)).

(** A partial assignment maps a variable to the value that makes it true. *)
Local Abbreviation Assign := (PositiveMap.t bool).

(** [Some true]: the literal is assigned true; [Some false]: its negation
    is; [None]: it is open. *)
Definition lit_status (A : Assign) (l : Z) : option bool :=
  match l with
  | Zpos p => PositiveMap.find p A
  | Zneg p => option_map negb (PositiveMap.find p A)
  | Z0 => Some false
  end.

Definition satisfied (A : Assign) (c : list Z) : bool :=
  existsb (fun l => match lit_status A l with Some true => true | _ => false end) c.

Definition open_lits (A : Assign) (c : list Z) : list Z :=
  filter (fun l => match lit_status A l with None => true | _ => false end) c.

Definition assume (A : Assign) (l : Z) : Assign :=
  match l with
  | Zpos p => PositiveMap.add p true A
  | Zneg p => PositiveMap.add p false A
  | Z0 => A
  end.

(** The assignment falsifying a lemma, or [None] when a variable occurs
    twice (a repeated or complementary literal). *)
Fixpoint falsify (lemma : list Z) (A : Assign) : option Assign :=
  match lemma with
  | [] => Some A
  | Zpos p :: rest =>
      match PositiveMap.find p A with Some _ => None | None => falsify rest (PositiveMap.add p false A) end
  | Zneg p :: rest =>
      match PositiveMap.find p A with Some _ => None | None => falsify rest (PositiveMap.add p true A) end
  | Z0 :: _ => None
  end.

Inductive Rup : Type := Rok | Rfail (code : string).

(** Every hint but the last must be unit and assigns its open literal; the
    last must be falsified; a satisfied hint fails. *)
Fixpoint rup (db : Db) (A : Assign) (hints : list positive) : Rup :=
  match hints with
  | [] => Rfail "rup"
  | h :: rest =>
      match PositiveMap.find h db with
      | None => Rfail "unknown_hint"
      | Some c =>
          if satisfied A c then Rfail "rup"
          else match open_lits A c with
               | [] => match rest with [] => Rok | _ => Rfail "rup" end
               | [u] => rup db (assume A u) rest
               | _ => Rfail "rup"
               end
      end
  end.

(** * Certificate lines *)

(** The active clauses by identifier and the last addition's identifier. *)
Record Checker : Type := checker { active : Db ; last_id : N }.

Inductive Token : Type := Knum (z : Z) | Kd.

Inductive LineResult : Type :=
| Lcont (c : Checker)
| Lempty
| Lreject (code : string).

(** Deletion targets: nonzero identifiers, then a final 0. *)
Fixpoint del_targets (ts : list Token) : option (list positive) :=
  match ts with
  | Knum Z0 :: [] => Some []
  | Knum (Zpos p) :: rest => option_map (cons p) (del_targets rest)
  | _ => None
  end.

Fixpoint delete_all (db : Db) (ts : list positive) : option Db :=
  match ts with
  | [] => Some db
  | t :: ts' =>
      match PositiveMap.find t db with
      | None => None
      | Some _ => delete_all (PositiveMap.remove t db) ts'
      end
  end.

Fixpoint all_nums (ts : list Token) : option (list Z) :=
  match ts with
  | [] => Some []
  | Knum z :: rest => option_map (cons z) (all_nums rest)
  | Kd :: _ => None
  end.

(** The lemma before the first 0 and the rest. *)
Fixpoint split_zero (l : list Z) : option (list Z * list Z) :=
  match l with
  | [] => None
  | z :: rest =>
      if Z.eqb z 0 then Some ([], rest)
      else option_map (fun '(a, b) => (z :: a, b)) (split_zero rest)
  end.

(** Hints: nonzero numbers, then a final 0. *)
Fixpoint hint_list (l : list Z) : option (list Z) :=
  match l with
  | [] => None
  | z :: rest =>
      if Z.eqb z 0 then match rest with [] => Some [] | _ => None end
      else option_map (cons z) (hint_list rest)
  end.

(** An addition, after its syntax: identifier order, variable range,
    lemma form, hint signs, then propagation. *)
Definition add_lemma (vars : positive) (c : Checker) (id : positive) (lemma hints : list Z) : LineResult :=
  if Npos id <=? last_id c then Lreject "id_order"
  else if existsb (fun l => Z.ltb (Zpos vars) (Z.abs l)) lemma then Lreject "var_range"
  else match falsify lemma (PositiveMap.empty bool) with
       | None => Lreject "lemma_form"
       | Some A =>
           if existsb (fun h => Z.ltb h 0) hints then Lreject "rat_unsupported"
           else match rup (active c) A (map Z.to_pos hints) with
                | Rfail code => Lreject code
                | Rok =>
                    match lemma with
                    | [] => Lempty
                    | _ => Lcont (checker (PositiveMap.add id lemma (active c)) (Npos id))
                    end
                end
       end.

(** One complete line, as its tokens. *)
Definition line_step (vars : positive) (c : Checker) (ts : list Token) : LineResult :=
  match ts with
  | Knum (Zpos _) :: Kd :: rest =>
      match del_targets rest with
      | None => Lreject "parse"
      | Some targets =>
          match delete_all (active c) targets with
          | None => Lreject "unknown_deletion"
          | Some db => Lcont (checker db (last_id c))
          end
      end
  | Knum (Zpos id) :: Knum z :: rest =>
      match all_nums rest with
      | None => Lreject "parse"
      | Some zs =>
          match split_zero (z :: zs) with
          | None => Lreject "parse"
          | Some (lemma, tail) =>
              match hint_list tail with
              | None => Lreject "parse"
              | Some hints => add_lemma vars c id lemma hints
              end
          end
      end
  | _ => Lreject "parse"
  end.

(** * Reading bytes *)

(** The token being read: nothing yet, a minus sign, a zero (canonical only
    if it ends there), a number with a nonzero leading digit, or [d]. *)
Inductive Tok : Type :=
| Tstart | Tminus | Tzero (neg : bool) | Tnum (neg : bool) (n : N) | Td.

(** The next byte of a token, or [None] when no token can continue so. *)
Definition feed (t : Tok) (c : int) : option Tok :=
  if Chars.is_digit c then
    let d := Chars.digit_value c in
    match t with
    | Tstart => Some (if d =? 0 then Tzero false else Tnum false d)
    | Tminus => Some (if d =? 0 then Tzero true else Tnum true d)
    | Tnum neg n => Some (Tnum neg (10 * n + d))
    | _ => None
    end
  else if PrimInt63.eqb c Chars.minus then match t with Tstart => Some Tminus | _ => None end
  else if PrimInt63.eqb c Chars.letter_d then match t with Tstart => Some Td | _ => None end
  else None.

(** A finished token; empty tokens, a lone minus and [-0] are not tokens. *)
Definition finish (t : Tok) : option Token :=
  match t with
  | Tzero false => Some (Knum 0)
  | Tnum false n => Some (Knum (Z.of_N n))
  | Tnum true n => Some (Knum (- Z.of_N n))
  | Td => Some Kd
  | _ => None
  end.

(** The machine: the next byte, the line number, the token being read, the
    tokens of the line so far (reversed) and the checker. *)
Record Machine : Type :=
  machine { cursor : int ; lineno : N ; partial : Tok ; tokens : list Token ; state : Checker }.

(** One byte. Any syntax error in a line is a [parse] rejection of that
    line, and so is a last line without its line feed; a line after the
    empty clause is [trailing]. *)
Definition step (vars : positive) (p : string) (len : int) (m : Machine) : Machine + Verdict :=
  let i := cursor m in
  if PrimInt63.leb len i then
    inr match partial m, tokens m with
        | Tstart, [] => reject "no_empty_clause" 0
        | _, _ => reject "parse" (lineno m)
        end
  else
    let c := PrimString.get p i in
    let next := PrimInt63.add i Chars.one in
    if PrimInt63.eqb c Chars.newline then
      match finish (partial m) with
      | None => inr (reject "parse" (lineno m))
      | Some t =>
          match line_step vars (state m) (rev (t :: tokens m)) with
          | Lcont ch => inl (machine next (N.succ (lineno m)) Tstart [] ch)
          | Lempty =>
              inr (if PrimInt63.ltb next len then reject "trailing" (N.succ (lineno m)) else accept)
          | Lreject code => inr (reject code (lineno m))
          end
      end
    else if PrimInt63.eqb c Chars.space then
      match finish (partial m) with
      | None => inr (reject "parse" (lineno m))
      | Some t => inl (machine next (lineno m) Tstart (t :: tokens m) (state m))
      end
    else
      match feed (partial m) c with
      | None => inr (reject "parse" (lineno m))
      | Some t => inl (machine next (lineno m) t (tokens m) (state m))
      end.

(** Up to [2 ^ depth] steps. *)
Fixpoint drive (depth : nat) (f : Machine -> Machine + Verdict) (m : Machine) : Machine + Verdict :=
  match depth with
  | O => f m
  | S d => match drive d f m with inl m' => drive d f m' | inr v => inr v end
  end.

Fixpoint load (k : positive) (cs : list (list Z)) (db : Db) : Db :=
  match cs with
  | [] => db
  | c :: cs' => load (Pos.succ k) cs' (PositiveMap.add k c db)
  end.

Definition initial (cs : list (list Z)) : Checker :=
  checker (load 1 cs (PositiveMap.empty _)) (N.of_nat (length cs)).

(** Every step reads one byte, so a certificate of at most [2 ^ 22] bytes
    ends within [2 ^ 23] steps; the first branch is never taken. *)
Definition check_certificate (vars : positive) (cs : list (list Z)) (p : string) : Verdict :=
  match drive 23 (step vars p (PrimString.length p)) (machine Chars.start 1 Tstart [] (initial cs)) with
  | inl _ => reject "oversized" 0
  | inr v => v
  end.

(** * The checker (B-F02) *)

Definition lrat_verdict (o : Obligation) (cnf certificate : PrimString.string) : Verdict :=
  if negb (PrimString.eqb cnf (cnf_text o)) then reject "cnf_mismatch" 0
  else if PrimInt63.ltb Chars.max_certificate (PrimString.length certificate) then reject "oversized" 0
  else check_certificate (cnf_vars o) (cnf_clauses o) certificate.

(** * Soundness of the checker *)

Definition lit_true (v : positive -> bool) (l : Z) : bool :=
  match l with Z0 => false | Zpos p => v p | Zneg p => negb (v p) end.

Definition clause_true (v : positive -> bool) (c : list Z) : bool := existsb (lit_true v) c.

Definition satisfies (v : positive -> bool) (cs : list (list Z)) : Prop :=
  forall c, In c cs -> clause_true v c = true.

Definition unsat (cs : list (list Z)) : Prop := forall v, ~ satisfies v cs.

Definition db_true (v : positive -> bool) (db : Db) : Prop :=
  forall k c, PositiveMap.find k db = Some c -> clause_true v c = true.

(** Every assignment satisfying the clauses satisfies the database. *)
Definition entails (cs : list (list Z)) (db : Db) : Prop :=
  forall v, satisfies v cs -> db_true v db.

Definition agrees (v : positive -> bool) (A : Assign) : Prop :=
  forall p b, PositiveMap.find p A = Some b -> v p = b.

Lemma agrees_add v A p b : agrees v A -> v p = b -> agrees v (PositiveMap.add p b A).
Proof.
  intros H E q b' F. destruct (Pos.eq_dec q p) as [->|Hne].
  - rewrite PositiveMap.gss in F. congruence.
  - rewrite PositiveMap.gso in F by exact Hne. exact (H q b' F).
Qed.

Lemma status_true v A l b : agrees v A -> lit_status A l = Some b -> lit_true v l = b.
Proof.
  intros H S. destruct l as [|p|p]; simpl in *.
  - congruence.
  - exact (H p b S).
  - destruct (PositiveMap.find p A) as [b'|] eqn:F; simpl in S; [|discriminate].
    injection S as <-. rewrite (H p b' F). reflexivity.
Qed.

(** A clause true under [v] has a literal that is neither falsified nor
    (when the clause is not satisfied) assigned: an open one. *)
Lemma true_clause_open v A c :
  agrees v A -> clause_true v c = true -> satisfied A c = false ->
  exists l, In l (open_lits A c) /\ lit_true v l = true.
Proof.
  intros H T S. apply existsb_exists in T as (l & Hl & Tl).
  exists l. split; [|exact Tl]. apply filter_In. split; [exact Hl|].
  destruct (lit_status A l) as [[|]|] eqn:St; [| |reflexivity].
  - assert (satisfied A c = true) by (apply existsb_exists; exists l; rewrite St; auto). congruence.
  - rewrite (status_true v A l false H St) in Tl. discriminate.
Qed.

Lemma rup_sound db A hs v : rup db A hs = Rok -> db_true v db -> agrees v A -> False.
Proof.
  revert A; induction hs as [|h hs IH]; intros A R D H; simpl in R; [discriminate|].
  destruct (PositiveMap.find h db) as [c|] eqn:F; [|discriminate].
  destruct (satisfied A c) eqn:S; [discriminate|].
  destruct (true_clause_open v A c H (D h c F) S) as (l & Hl & Tl).
  destruct (open_lits A c) as [|u [|u' rest]] eqn:O.
  - destruct Hl.
  - destruct Hl as [->|[]]. apply (IH (assume A l) R D).
    destruct l as [|p|p]; simpl in Tl |- *; [discriminate| |]; apply agrees_add; auto.
    destruct (v p); [discriminate|reflexivity].
  - discriminate.
Qed.

Lemma falsify_agrees lemma A A' v :
  falsify lemma A = Some A' -> agrees v A -> clause_true v lemma = false -> agrees v A'.
Proof.
  revert A; induction lemma as [|l lemma IH]; intros A F H C; simpl in F.
  - injection F as <-. exact H.
  - unfold clause_true in C; simpl in C. apply orb_false_elim in C as [C1 C2].
    destruct l as [|p|p]; simpl in C1; [discriminate| |].
    + destruct (PositiveMap.find p A); [discriminate|].
      apply (IH _ F); [apply agrees_add; [exact H|exact C1]|exact C2].
    + destruct (PositiveMap.find p A); [discriminate|].
      apply (IH _ F); [apply agrees_add; [exact H|]|exact C2].
      destruct (v p); [reflexivity|discriminate].
Qed.

(** A successful propagation entails the lemma. *)
Lemma rup_entails db lemma A hints v :
  falsify lemma (PositiveMap.empty bool) = Some A -> rup db A hints = Rok ->
  db_true v db -> clause_true v lemma = true.
Proof.
  intros F R D. destruct (clause_true v lemma) eqn:C; [reflexivity|].
  exfalso. apply (rup_sound db A hints v R D).
  apply (falsify_agrees lemma _ A v F); [|exact C].
  intros p b E. rewrite PositiveMap.gempty in E. discriminate.
Qed.

(** What a line leaves: an entailed database, an unsatisfiable formula
    when it derives the empty clause, and nothing on a rejection. *)
Definition line_ok (cs : list (list Z)) (r : LineResult) : Prop :=
  match r with
  | Lcont c => entails cs (active c)
  | Lempty => unsat cs
  | Lreject _ => True
  end.

Lemma add_lemma_sound cs vars c id lemma hints :
  entails cs (active c) -> line_ok cs (add_lemma vars c id lemma hints).
Proof.
  intros E. unfold add_lemma.
  destruct (Npos id <=? last_id c); [exact I|].
  destruct (existsb _ lemma); [exact I|].
  destruct (falsify lemma _) as [A|] eqn:F; [|exact I].
  destruct (existsb _ hints); [exact I|].
  destruct (rup (active c) A _) as [|code] eqn:R; [|exact I].
  destruct lemma as [|l lemma'] eqn:L; simpl.
  - intros v Hv. pose proof (rup_entails _ _ _ _ v F R (E v Hv)). discriminate.
  - intros v Hv k c' K. destruct (Pos.eq_dec k id) as [->|Hne].
    + rewrite PositiveMap.gss in K. injection K as <-.
      exact (rup_entails _ _ _ _ v F R (E v Hv)).
    + rewrite PositiveMap.gso in K by exact Hne. exact (E v Hv k c' K).
Qed.

Lemma delete_all_sub db ts db' :
  delete_all db ts = Some db' -> forall k c, PositiveMap.find k db' = Some c -> PositiveMap.find k db = Some c.
Proof.
  revert db; induction ts as [|t ts IH]; intros db D k c K; simpl in D.
  - congruence.
  - destruct (PositiveMap.find t db); [|discriminate].
    pose proof (IH _ D k c K) as K'. destruct (Pos.eq_dec k t) as [->|Hne].
    + rewrite PositiveMap.grs in K'. discriminate.
    + rewrite PositiveMap.gro in K' by exact Hne. exact K'.
Qed.

Lemma line_step_sound cs vars c ts : entails cs (active c) -> line_ok cs (line_step vars c ts).
Proof.
  intros E. unfold line_step.
  destruct ts as [|[[|id|]|] [|[z|] rest]]; try exact I.
  - destruct (all_nums rest) as [zs|]; [|exact I].
    destruct (split_zero (z :: zs)) as [[lemma tail]|]; [|exact I].
    destruct (hint_list tail) as [hints|]; [|exact I].
    apply add_lemma_sound, E.
  - destruct (del_targets rest) as [targets|]; [|exact I].
    destruct (delete_all (active c) targets) as [db|] eqn:D; [|exact I].
    intros v Hv k c' K. exact (E v Hv k c' (delete_all_sub _ _ _ D k c' K)).
Qed.

(** The invariant of the machine: its database is entailed. *)
Definition outcome_ok (cs : list (list Z)) (r : Machine + Verdict) : Prop :=
  match r with
  | inl m => entails cs (active (state m))
  | inr accept => unsat cs
  | inr (reject _ _) => True
  end.

Lemma step_sound cs vars p len m :
  entails cs (active (state m)) -> outcome_ok cs (step vars p len m).
Proof.
  intros E. unfold step.
  destruct (PrimInt63.leb len (cursor m)).
  { destruct (partial m), (tokens m); exact I. }
  destruct (PrimInt63.eqb _ Chars.newline).
  - destruct (finish (partial m)) as [t|]; [|exact I].
    pose proof (line_step_sound cs vars (state m) (rev (t :: tokens m)) E) as L.
    destruct (line_step _ _ _); simpl in *; [exact L| |exact I].
    destruct (PrimInt63.ltb _ _); [exact I|exact L].
  - destruct (PrimInt63.eqb _ Chars.space).
    + destruct (finish (partial m)); exact E || exact I.
    + destruct (feed (partial m) _); exact E || exact I.
Qed.

Lemma drive_sound cs f :
  (forall m, entails cs (active (state m)) -> outcome_ok cs (f m)) ->
  forall d m, entails cs (active (state m)) -> outcome_ok cs (drive d f m).
Proof.
  intros F d; induction d as [|d IH]; intros m E; simpl; [apply F, E|].
  pose proof (IH m E) as H. destruct (drive d f m) as [m'|r]; [apply IH, H|exact H].
Qed.

Lemma load_in k cs db j c :
  PositiveMap.find j (load k cs db) = Some c -> In c cs \/ PositiveMap.find j db = Some c.
Proof.
  revert k db; induction cs as [|c0 cs IH]; intros k db H; simpl in H; [right; exact H|].
  destruct (IH _ _ H) as [I|I]; [left; right; exact I|].
  destruct (Pos.eq_dec j k) as [->|Hne].
  - rewrite PositiveMap.gss in I. injection I as ->. left; left; reflexivity.
  - rewrite PositiveMap.gso in I by exact Hne. right; exact I.
Qed.

Lemma initial_entailed cs : entails cs (active (initial cs)).
Proof.
  intros v Hv k c K. simpl in K. destruct (load_in _ _ _ _ _ K) as [I|I].
  - exact (Hv c I).
  - rewrite PositiveMap.gempty in I. discriminate.
Qed.

Lemma check_certificate_sound vars cs p : check_certificate vars cs p = accept -> unsat cs.
Proof.
  unfold check_certificate. intros H.
  pose proof (drive_sound cs (step vars p (PrimString.length p)) (fun m => step_sound cs vars p _ m) 23
                (machine Chars.start 1 Tstart [] (initial cs)) (initial_entailed cs)) as S.
  destruct (drive _ _ _) as [m|r]; [discriminate|]. subst r. exact S.
Qed.

(** * Correctness of the bit-blast *)

(** ** Bits of naturals, least significant first *)

Fixpoint to_bits (n : nat) (x : N) : list bool :=
  match n with O => [] | S n' => N.odd x :: to_bits n' (N.div2 x) end.

Fixpoint of_bits (l : list bool) : N :=
  match l with [] => 0 | b :: l' => N.b2n b + 2 * of_bits l' end.

Definition wbits (x : Word 32) : list bool := to_bits W (val x).

Lemma length_to_bits n x : length (to_bits n x) = n.
Proof. revert x; induction n; intros; simpl; auto. Qed.

Lemma to_bits_nth n x i : (i < n)%nat -> nth i (to_bits n x) false = N.testbit x (N.of_nat i).
Proof.
  revert x i; induction n as [|n IH]; intros x i H; [lia|].
  destruct i as [|i]; cbn [nth to_bits].
  - rewrite N.bit0_odd. reflexivity.
  - rewrite IH by lia. rewrite N.testbit_div2, Nnat.Nat2N.inj_succ. reflexivity.
Qed.

Lemma to_bits_zip f n z a b :
  (forall i, (i < n)%nat -> N.testbit z (N.of_nat i) = f (N.testbit a (N.of_nat i)) (N.testbit b (N.of_nat i))) ->
  to_bits n z = zip_with f (to_bits n a) (to_bits n b).
Proof.
  revert z a b; induction n as [|n IH]; intros z a b H; [reflexivity|].
  simpl. f_equal.
  - rewrite <- !N.bit0_odd. apply (H 0%nat). lia.
  - apply IH. intros i Hi. rewrite !N.testbit_div2, <- !Nnat.Nat2N.inj_succ. apply H. lia.
Qed.

Lemma of_to_bits n x : x < 2 ^ N.of_nat n -> of_bits (to_bits n x) = x.
Proof.
  revert x; induction n as [|n IH]; intros x H; cbn [to_bits of_bits].
  - change (2 ^ N.of_nat 0) with 1 in H. lia.
  - rewrite Nnat.Nat2N.inj_succ, N.pow_succ_r' in H.
    pose proof (N.div2_odd x) as E.
    rewrite IH by (destruct (N.odd x); cbn [N.b2n] in E; lia). lia.
Qed.

Lemma to_of_bits l : to_bits (length l) (of_bits l) = l.
Proof.
  induction l as [|b l IH]; [reflexivity|]. cbn [length to_bits of_bits].
  replace (N.odd (N.b2n b + 2 * of_bits l)) with b by (destruct b, (of_bits l); reflexivity).
  replace (N.div2 (N.b2n b + 2 * of_bits l)) with (of_bits l) by (destruct b, (of_bits l); reflexivity).
  rewrite IH. reflexivity.
Qed.

Lemma of_bits_bound l : of_bits l < 2 ^ N.of_nat (length l).
Proof.
  induction l as [|b l IH]; cbn [length of_bits]; [lia|].
  rewrite Nnat.Nat2N.inj_succ, N.pow_succ_r'. destruct b; cbn [N.b2n]; lia.
Qed.

Lemma wbits_injective (x y : Word 32) : wbits x = wbits y -> x = y.
Proof.
  intros E. apply word_ext.
  rewrite <- (of_to_bits W (val x)), <- (of_to_bits W (val y)) by apply val_bound.
  unfold wbits in E. rewrite E. reflexivity.
Qed.

(** ** Ripple-carry addition on booleans *)

Fixpoint add_bits (c : bool) (a b : list bool) : list bool :=
  match a, b with
  | a0 :: a', b0 :: b' => let t := xorb a0 b0 in xorb t c :: add_bits (a0 && b0 || c && t) a' b'
  | _, _ => []
  end.

Fixpoint carry_bits (c : bool) (a b : list bool) : bool :=
  match a, b with
  | a0 :: a', b0 :: b' => let t := xorb a0 b0 in carry_bits (a0 && b0 || c && t) a' b'
  | _, _ => c
  end.

Lemma length_add_bits c a b : length a = length b -> length (add_bits c a b) = length a.
Proof.
  revert c b; induction a as [|a0 a IH]; intros c [|b0 b] H; simpl in *; try lia.
  rewrite IH by lia. reflexivity.
Qed.

Lemma add_bits_spec c a b : length a = length b ->
  of_bits (add_bits c a b) + 2 ^ N.of_nat (length a) * N.b2n (carry_bits c a b) =
  of_bits a + of_bits b + N.b2n c.
Proof.
  revert c b; induction a as [|a0 a IH]; intros c [|b0 b] H;
    cbn [add_bits carry_bits of_bits length] in *; try discriminate.
  - change (2 ^ N.of_nat 0) with 1. lia.
  - injection H as H. specialize (IH (a0 && b0 || c && xorb a0 b0) b H).
    rewrite Nnat.Nat2N.inj_succ, N.pow_succ_r'.
    set (P := 2 ^ N.of_nat (length a)) in *.
    set (K := N.b2n (carry_bits (a0 && b0 || c && xorb a0 b0) a b)) in *.
    assert (Bit : N.b2n (xorb (xorb a0 b0) c) + 2 * N.b2n (a0 && b0 || c && xorb a0 b0) =
                  N.b2n a0 + N.b2n b0 + N.b2n c) by (destruct a0, b0, c; reflexivity).
    nia.
Qed.

(** ** Word operations as bits *)

Lemma wbits_xor x y : wbits (word_xor 32 x y) = zip_with xorb (wbits x) (wbits y).
Proof.
  apply to_bits_zip. intros i Hi. cbn [val word_xor word_of_nat].
  rewrite N.mod_pow2_bits_low by (unfold W in Hi; lia). apply N.lxor_spec.
Qed.

Lemma wbits_and x y : wbits (word_and 32 x y) = zip_with andb (wbits x) (wbits y).
Proof.
  apply to_bits_zip. intros i Hi. cbn [val word_and word_of_nat].
  rewrite N.mod_pow2_bits_low by (unfold W in Hi; lia). apply N.land_spec.
Qed.

Lemma wbits_add x y : wbits (word_add 32 x y) = add_bits false (wbits x) (wbits y).
Proof.
  set (a := wbits x). set (b := wbits y).
  assert (La : length a = W) by apply length_to_bits.
  assert (Lb : length b = W) by apply length_to_bits.
  assert (Ea : of_bits a = val x) by (apply of_to_bits, val_bound).
  assert (Eb : of_bits b = val y) by (apply of_to_bits, val_bound).
  pose proof (add_bits_spec false a b ltac:(congruence)) as S.
  pose proof (of_bits_bound (add_bits false a b)) as B.
  rewrite length_add_bits in B by congruence. rewrite La in S, B.
  change (2 ^ N.of_nat W) with (2 ^ 32) in S, B. change (N.b2n false) with 0 in S.
  unfold wbits at 1. cbn [val word_add word_of_nat].
  rewrite <- (N.mod_unique (val x + val y) (2 ^ 32) (N.b2n (carry_bits false a b)) (of_bits (add_bits false a b)) B)
    by lia.
  rewrite <- La, <- (length_add_bits false a b) by congruence. apply to_of_bits.
Qed.

Lemma wbits_shl x k : wbits (word_shl 32 x k) = firstn W (repeat false (N.to_nat k) ++ wbits x).
Proof.
  unfold wbits. apply nth_ext with (d := false) (d' := false).
  { rewrite length_firstn, length_app, repeat_length, !length_to_bits. lia. }
  intros i Hi. rewrite length_to_bits in Hi.
  rewrite to_bits_nth, nth_firstn by exact Hi.
  cbn [val word_shl word_of_nat]. rewrite N.mod_pow2_bits_low by (unfold W in Hi; lia).
  destruct (Nat.ltb_spec i W) as [_|]; [|lia].
  destruct (Nat.ltb_spec i (N.to_nat k)) as [Lt|Ge].
  - rewrite app_nth1 by (rewrite repeat_length; exact Lt).
    rewrite nth_repeat. apply N.shiftl_spec_low. lia.
  - rewrite app_nth2 by (rewrite repeat_length; exact Ge). rewrite repeat_length.
    rewrite to_bits_nth by lia. rewrite N.shiftl_spec_high' by lia.
    f_equal. lia.
Qed.

(** ** The blaster computes the bits *)

Definition gate_ok (v : positive -> bool) (g : Gate) : Prop :=
  v (gout g) = gate_fn (gkind g) (v (ga g)) (v (gb g)).

Definition gates_ok (v : positive -> bool) (gs : list Gate) : Prop := forall g, In g gs -> gate_ok v g.

(** A later state keeps every gate of an earlier one. *)
Definition extends (s s' : St) : Prop := exists pre, gates s' = pre ++ gates s.

Lemma extends_refl s : extends s s.
Proof. exists []; reflexivity. Qed.

Lemma extends_trans s1 s2 s3 : extends s1 s2 -> extends s2 s3 -> extends s1 s3.
Proof. intros [p E] [q F]. exists (q ++ p). rewrite F, E, app_assoc. reflexivity. Qed.

Lemma gates_ok_extends v s s' : extends s s' -> gates_ok v (gates s') -> gates_ok v (gates s).
Proof. intros [p E] H g I. apply H. rewrite E. apply in_or_app. right; exact I. Qed.

Lemma mk_spec k a b s g s' : mk k a b s = (g, s') ->
  extends s s' /\ forall v, gates_ok v (gates s') -> v g = gate_fn k (v a) (v b).
Proof.
  unfold mk. intros H. injection H as <- <-. split.
  - exists [gate (Pos.succ (top s)) k a b]. reflexivity.
  - intros v Hv. apply (Hv (gate _ k a b)). left; reflexivity.
Qed.

Lemma bitwise_spec k a b s out s' : bitwise k a b s = (out, s') ->
  extends s s' /\ forall v, gates_ok v (gates s') -> map v out = zip_with (gate_fn k) (map v a) (map v b).
Proof.
  revert b s out s'; induction a as [|a0 a IH]; intros [|b0 b] s out s' H; cbn [bitwise] in H;
    try (injection H as <- <-; split; [apply extends_refl|reflexivity]).
  destruct (mk k a0 b0 s) as [g s1] eqn:E1. destruct (bitwise k a b s1) as [rest s2] eqn:E2.
  injection H as <- <-.
  destruct (mk_spec _ _ _ _ _ _ E1) as [X1 G1]. destruct (IH _ _ _ _ E2) as [X2 G2].
  split; [exact (extends_trans _ _ _ X1 X2)|]. intros v Hv. simpl.
  rewrite (G2 v Hv), (G1 v (gates_ok_extends v _ _ X2 Hv)). reflexivity.
Qed.

Lemma adder_spec c a b s out s' : adder c a b s = (out, s') ->
  extends s s' /\ forall v, gates_ok v (gates s') -> map v out = add_bits (v c) (map v a) (map v b).
Proof.
  revert c b s out s'; induction a as [|a0 a IH]; intros c [|b0 b] s out s' H; cbn [adder] in H;
    try (injection H as <- <-; split; [apply extends_refl|reflexivity]).
  destruct (mk Gxor a0 b0 s) as [t s1] eqn:E1.
  destruct (mk Gxor t c s1) as [sum s2] eqn:E2.
  destruct (mk Gand a0 b0 s2) as [u s3] eqn:E3.
  destruct (mk Gand c t s3) as [w s4] eqn:E4.
  destruct (mk Gor u w s4) as [c' s5] eqn:E5.
  destruct (adder c' a b s5) as [rest s6] eqn:E6.
  injection H as <- <-.
  destruct (mk_spec _ _ _ _ _ _ E1) as [X1 G1]. destruct (mk_spec _ _ _ _ _ _ E2) as [X2 G2].
  destruct (mk_spec _ _ _ _ _ _ E3) as [X3 G3]. destruct (mk_spec _ _ _ _ _ _ E4) as [X4 G4].
  destruct (mk_spec _ _ _ _ _ _ E5) as [X5 G5]. destruct (IH _ _ _ _ _ E6) as [X6 G6].
  assert (X56 : extends s5 s6) by exact X6.
  assert (X46 : extends s4 s6) by (eapply extends_trans; eauto).
  assert (X36 : extends s3 s6) by (eapply extends_trans; eauto).
  assert (X26 : extends s2 s6) by (eapply extends_trans; eauto).
  assert (X16 : extends s1 s6) by (eapply extends_trans; eauto).
  split; [eapply extends_trans; eauto|]. intros v Hv. simpl.
  rewrite (G6 v Hv), (G5 v (gates_ok_extends v _ _ X56 Hv)), (G4 v (gates_ok_extends v _ _ X46 Hv)),
    (G3 v (gates_ok_extends v _ _ X36 Hv)), (G2 v (gates_ok_extends v _ _ X26 Hv)),
    (G1 v (gates_ok_extends v _ _ X16 Hv)).
  reflexivity.
Qed.

Lemma bits_spec e s out s' : bits e s = (out, s') ->
  extends s s' /\
  forall v x y, gates_ok v (gates s') -> v 1%positive = false ->
    map v (range 2 W) = wbits x -> map v (range 34 W) = wbits y ->
    map v out = wbits (eval_bv e x y).
Proof.
  revert s out s'; induction e as [| |l IHl r IHr|l IHl r IHr|l IHl r IHr|e IH k]; intros s out s' H;
    cbn [bits] in H.
  - injection H as <- <-. split; [apply extends_refl|]. intros v x y _ _ Hx _. exact Hx.
  - injection H as <- <-. split; [apply extends_refl|]. intros v x y _ _ _ Hy. exact Hy.
  - destruct (bits l s) as [a s1] eqn:E1. destruct (bits r s1) as [b s2] eqn:E2.
    destruct (IHl _ _ _ E1) as [X1 G1]. destruct (IHr _ _ _ E2) as [X2 G2].
    destruct (adder_spec _ _ _ _ _ _ H) as [X3 G3].
    split; [eauto using extends_trans|]. intros v x y Hv V1 Hx Hy.
    assert (Hv2 : gates_ok v (gates s2)) by exact (gates_ok_extends v _ _ X3 Hv).
    assert (Hv1 : gates_ok v (gates s1)) by exact (gates_ok_extends v _ _ X2 Hv2).
    rewrite (G3 v Hv), V1, (G1 v x y Hv1 V1 Hx Hy), (G2 v x y Hv2 V1 Hx Hy).
    cbn [eval_bv]. rewrite wbits_add. reflexivity.
  - destruct (bits l s) as [a s1] eqn:E1. destruct (bits r s1) as [b s2] eqn:E2.
    destruct (IHl _ _ _ E1) as [X1 G1]. destruct (IHr _ _ _ E2) as [X2 G2].
    destruct (bitwise_spec _ _ _ _ _ _ H) as [X3 G3].
    split; [eauto using extends_trans|]. intros v x y Hv V1 Hx Hy.
    assert (Hv2 : gates_ok v (gates s2)) by exact (gates_ok_extends v _ _ X3 Hv).
    assert (Hv1 : gates_ok v (gates s1)) by exact (gates_ok_extends v _ _ X2 Hv2).
    rewrite (G3 v Hv), (G1 v x y Hv1 V1 Hx Hy), (G2 v x y Hv2 V1 Hx Hy).
    cbn [eval_bv]. rewrite wbits_xor. reflexivity.
  - destruct (bits l s) as [a s1] eqn:E1. destruct (bits r s1) as [b s2] eqn:E2.
    destruct (IHl _ _ _ E1) as [X1 G1]. destruct (IHr _ _ _ E2) as [X2 G2].
    destruct (bitwise_spec _ _ _ _ _ _ H) as [X3 G3].
    split; [eauto using extends_trans|]. intros v x y Hv V1 Hx Hy.
    assert (Hv2 : gates_ok v (gates s2)) by exact (gates_ok_extends v _ _ X3 Hv).
    assert (Hv1 : gates_ok v (gates s1)) by exact (gates_ok_extends v _ _ X2 Hv2).
    rewrite (G3 v Hv), (G1 v x y Hv1 V1 Hx Hy), (G2 v x y Hv2 V1 Hx Hy).
    cbn [eval_bv]. rewrite wbits_and. reflexivity.
  - destruct (bits e s) as [a s1] eqn:E1. injection H as <- <-.
    destruct (IH _ _ _ E1) as [X1 G1]. split; [exact X1|]. intros v x y Hv V1 Hx Hy.
    change (map v (firstn W (repeat 1%positive (N.to_nat k) ++ a)) = wbits (eval_bv (Bshl e k) x y)).
    rewrite <- firstn_map, map_app, map_repeat, V1, (G1 v x y Hv V1 Hx Hy).
    cbn [eval_bv]. rewrite wbits_shl. reflexivity.
Qed.

Lemma existsb_id_map (v : positive -> bool) l : existsb (fun b => b) (map v l) = existsb v l.
Proof. induction l as [|p l IH]; simpl; [reflexivity|]. rewrite IH. reflexivity. Qed.

Lemma or_chain_spec acc ds s o s' : or_chain acc ds s = (o, s') ->
  extends s s' /\ forall v, gates_ok v (gates s') -> v o = v acc || existsb v ds.
Proof.
  revert acc s; induction ds as [|d ds IH]; intros acc s H; cbn [or_chain] in H.
  - injection H as <- <-. split; [apply extends_refl|]. intros v _. rewrite orb_false_r. reflexivity.
  - destruct (mk Gor acc d s) as [g s1] eqn:E1.
    destruct (mk_spec _ _ _ _ _ _ E1) as [X1 G1]. destruct (IH _ _ H) as [X2 G2].
    split; [eauto using extends_trans|]. intros v Hv.
    rewrite (G2 v Hv), (G1 v (gates_ok_extends v _ _ X2 Hv)). simpl. symmetry. apply orb_assoc.
Qed.

Lemma miter_spec l r s o s' : miter l r s = (o, s') ->
  extends s s' /\ forall v, gates_ok v (gates s') -> v 1%positive = false ->
    v o = existsb (fun b => b) (zip_with xorb (map v l) (map v r)).
Proof.
  unfold miter. intros H.
  destruct (bitwise Gxor l r s) as [ds s1] eqn:E1. destruct (bitwise_spec _ _ _ _ _ _ E1) as [X1 G1].
  destruct ds as [|d ds].
  - injection H as <- <-. split; [exact X1|]. intros v Hv V1.
    change xorb with (gate_fn Gxor). rewrite <- (G1 v Hv). simpl. exact V1.
  - destruct (or_chain_spec _ _ _ _ _ H) as [X2 G2].
    split; [eauto using extends_trans|]. intros v Hv V1. change xorb with (gate_fn Gxor).
    rewrite <- (G1 v (gates_ok_extends v _ _ X2 Hv)), (G2 v Hv). simpl. rewrite existsb_id_map. reflexivity.
Qed.

Lemma blast_spec o out s : blast o = (out, s) ->
  forall v x y, gates_ok v (gates s) -> v 1%positive = false ->
    map v (range 2 W) = wbits x -> map v (range 34 W) = wbits y ->
    v out = existsb (fun b => b) (zip_with xorb (wbits (eval_bv (lhs o) x y)) (wbits (eval_bv (rhs o) x y))).
Proof.
  unfold blast. intros H v x y Hv V1 Hx Hy.
  destruct (bits (lhs o) (st 65 [])) as [l s1] eqn:E1. destruct (bits (rhs o) s1) as [r s2] eqn:E2.
  destruct (bits_spec _ _ _ _ E1) as [X1 G1]. destruct (bits_spec _ _ _ _ E2) as [X2 G2].
  destruct (miter_spec _ _ _ _ _ H) as [X3 G3].
  assert (Hv2 : gates_ok v (gates s2)) by exact (gates_ok_extends v _ _ X3 Hv).
  assert (Hv1 : gates_ok v (gates s1)) by exact (gates_ok_extends v _ _ X2 Hv2).
  rewrite (G3 v Hv V1), (G1 v x y Hv1 V1 Hx Hy), (G2 v x y Hv2 V1 Hx Hy). reflexivity.
Qed.

(** ** An assignment from two words *)

(** Variable [p] of the inputs: 1 is false, then the bits of [x] and [y]. *)
Definition inputs (x y : Word 32) (p : positive) : bool :=
  nth (Pos.to_nat p) (false :: false :: wbits x ++ wbits y) false.

(** Every gate evaluated from the gates before it. *)
Fixpoint value (gs : list Gate) (b : positive -> bool) (p : positive) : bool :=
  match gs with
  | [] => b p
  | g :: rest =>
      if Pos.eqb p (gout g) then gate_fn (gkind g) (value rest b (ga g)) (value rest b (gb g))
      else value rest b p
  end.

(** Gates numbered above the inputs, decreasing, each above its inputs. *)
Fixpoint wf_gates (gs : list Gate) : bool :=
  match gs with
  | [] => true
  | g :: rest =>
      Pos.ltb 65 (gout g) && Pos.ltb (ga g) (gout g) && Pos.ltb (gb g) (gout g) &&
      forallb (fun h => Pos.ltb (gout h) (gout g)) rest && wf_gates rest
  end.

Lemma wf_gates_in gs g : wf_gates gs = true -> In g gs ->
  (65 < gout g)%positive /\ (ga g < gout g)%positive /\ (gb g < gout g)%positive.
Proof.
  induction gs as [|h gs IH]; intros W' I; [destruct I|]. simpl in W'.
  apply andb_prop in W' as [W' R]. apply andb_prop in W' as [W' _].
  apply andb_prop in W' as [W' B]. apply andb_prop in W' as [T A].
  destruct I as [<-|I]; [|exact (IH R I)].
  apply Pos.ltb_lt in T, A, B. auto.
Qed.

Lemma value_skip g rest b p : p <> gout g -> value (g :: rest) b p = value rest b p.
Proof. intros H. simpl. apply Pos.eqb_neq in H. rewrite H. reflexivity. Qed.

Lemma value_ok gs b : wf_gates gs = true -> gates_ok (value gs b) gs.
Proof.
  induction gs as [|g gs IH]; intros W' h I; [destruct I|].
  pose proof W' as W0. simpl in W'.
  apply andb_prop in W' as [W' R]. apply andb_prop in W' as [W' F].
  apply andb_prop in W' as [W' B]. apply andb_prop in W' as [_ A].
  apply Pos.ltb_lt in A, B. rewrite forallb_forall in F.
  unfold gate_ok. destruct I as [<-|I].
  - rewrite (value_skip g gs b (ga g)), (value_skip g gs b (gb g)) by lia.
    cbn [value]. rewrite Pos.eqb_refl. reflexivity.
  - pose proof (proj1 (Pos.ltb_lt _ _) (F h I)) as Hh.
    destruct (wf_gates_in gs h R I) as (_ & Ha & Hb).
    rewrite !value_skip by lia. exact (IH R h I).
Qed.

Lemma value_inputs gs b p : wf_gates gs = true -> (p <= 65)%positive -> value gs b p = b p.
Proof.
  induction gs as [|g gs IH]; intros W' H; [reflexivity|].
  pose proof (wf_gates_in (g :: gs) g W' (or_introl eq_refl)) as (T & _ & _).
  simpl in W'. apply andb_prop in W' as [_ R].
  rewrite value_skip by lia. exact (IH R H).
Qed.

Lemma inputs_x x y : map (inputs x y) (range 2 W) = wbits x.
Proof. reflexivity. Qed.

Lemma inputs_y x y : map (inputs x y) (range 34 W) = wbits y.
Proof. reflexivity. Qed.

Lemma range_inputs : forallb (fun p => Pos.leb p 65) (range 2 W ++ range 34 W) = true.
Proof. reflexivity. Qed.

Lemma blast_wf o : wf_gates (gates (snd (blast o))) = true.
Proof. destruct o; vm_compute; reflexivity. Qed.

Lemma gate_clauses_true v g : gate_ok v g -> forall c, In c (gate_clauses g) -> clause_true v c = true.
Proof.
  unfold gate_ok, gate_clauses. intros H c I.
  destruct g as [o k a b]; simpl in *.
  destruct k; simpl in H;
    repeat destruct I as [<-|I]; try destruct I; unfold clause_true; simpl; rewrite H;
    destruct (v a), (v b); reflexivity.
Qed.

Lemma zip_xor_differs l m : length l = length m -> l <> m -> existsb (fun b => b) (zip_with xorb l m) = true.
Proof.
  revert m; induction l as [|a l IH]; intros [|b m] L N; simpl in *; try congruence.
  destruct a, b; simpl; auto; apply IH; congruence.
Qed.

(** A pair of words breaking the equation satisfies every clause. *)
Lemma counterexample_satisfies o x y :
  eval_bv (lhs o) x y <> eval_bv (rhs o) x y -> exists v, satisfies v (cnf_clauses o).
Proof.
  intros Ne. pose proof (blast_wf o) as WF. unfold cnf_clauses.
  destruct (blast o) as [out s] eqn:E. simpl in WF.
  set (v := value (gates s) (inputs x y)).
  assert (Hv : gates_ok v (gates s)) by (apply value_ok, WF).
  assert (Base : forall p, In p (range 2 W ++ range 34 W) -> v p = inputs x y p).
  { intros p I. apply value_inputs; [exact WF|].
    pose proof range_inputs as R. rewrite forallb_forall in R. apply Pos.leb_le, R, I. }
  assert (V1 : v 1%positive = false) by (unfold v; rewrite value_inputs by (exact WF || lia); reflexivity).
  assert (Hx : map v (range 2 W) = wbits x).
  { rewrite <- (inputs_x x y). apply map_ext_in. intros p I. apply Base, in_or_app. left; exact I. }
  assert (Hy : map v (range 34 W) = wbits y).
  { rewrite <- (inputs_y x y). apply map_ext_in. intros p I. apply Base, in_or_app. right; exact I. }
  exists v. intros c I. destruct I as [<-|I].
  { unfold clause_true; simpl. rewrite V1. reflexivity. }
  apply in_app_or in I as [I|I].
  - apply in_flat_map in I as (g & Ig & Ic). apply in_rev in Ig.
    exact (gate_clauses_true v g (Hv g Ig) c Ic).
  - destruct I as [<-|[]]. unfold clause_true; simpl.
    rewrite (blast_spec o out s E v x y Hv V1 Hx Hy), orb_false_r.
    apply zip_xor_differs; [unfold wbits; rewrite !length_to_bits; reflexivity|].
    intros Eq. apply Ne, wbits_injective, Eq.
Qed.

Lemma unsat_holds o : unsat (cnf_clauses o) -> holds o.
Proof.
  intros U x y. destruct (N.eq_dec (val (eval_bv (lhs o) x y)) (val (eval_bv (rhs o) x y))) as [E|Ne].
  - apply word_ext, E.
  - exfalso. destruct (counterexample_satisfies o x y) as [v Hv].
    + intros E. apply Ne. rewrite E. reflexivity.
    + exact (U v Hv).
Qed.

(** * Theorems D4-TH02 to D4-TH05 *)

Theorem meaning_of_carry_save :
  holds carry_save <->
  forall x y : Word 32,
    word_add 32 x y = word_add 32 (word_xor 32 x y) (word_shl 32 (word_and 32 x y) 1).
Proof. apply iff_refl. Qed.

Theorem meaning_of_carry_save_unshifted :
  holds carry_save_unshifted <->
  forall x y : Word 32, word_add 32 x y = word_add 32 (word_xor 32 x y) (word_and 32 x y).
Proof. apply iff_refl. Qed.

Theorem accepted_certificates_hold :
  forall (o : Obligation) (f p : PrimString.string), lrat_verdict o f p = accept -> holds o.
Proof.
  intros o f p H. unfold lrat_verdict in H.
  destruct (negb _); [discriminate|].
  destruct (PrimInt63.ltb _ _); [discriminate|].
  exact (unsat_holds o (check_certificate_sound _ _ _ H)).
Qed.

(** The counterexample x = y = 1: 1 + 1 = 2, but (1 xor 1) + (1 and 1) = 1. *)
Theorem unshifted_is_refuted : ~ holds carry_save_unshifted.
Proof.
  intros H. specialize (H (word_of_nat 32 1) (word_of_nat 32 1)).
  apply (f_equal val) in H. vm_compute in H. discriminate.
Qed.

(** * D4-TH01, through the checker *)

(** The golden certificate of the shared inputs
    ([ds04-carry-save-golden.lrat], 61850 bytes, SHA-256
    9b75f94cee4bae8b17133fd877e8e58f2023dcf8d3871d8c1ee442faa0bc1276), produced by drat-trim from
    a CaDiCaL proof of the CNF of [carry_save]; it is untrusted data. *)
Definition golden_certificate : PrimString.string := "1535 d 7 9 14 15 17 537 538 539 540 541 542 543 544 545 767 768 769 771 773 775 777 778 780 782 783 784 785 792 794 799 800 802 1305 1306 1307 1308 1309 1310 1311 1312 1313 1316 1317 1320 1321 1325 1344 1361 1376 1377 1380 1381 1384 1389 1392 1393 1397 1400 1401 1405 1408 1409 1412 1413 1416 1417 1420 1421 1424 1425 1432 1436 1437 1442 1443 1445 1446 1448 1449 1451 1452 1454 1455 1457 1458 1460 1461 1463 1464 1466 1467 1469 1470 1472 1473 1475 1476 1478 1479 1481 1482 1484 1485 1487 1488 1490 1494 1500 1502 1503 1505 1508 1509 1511 1512 1517 1518 1520 1521 1523 1529 1530 1532 1533 0
1536 -67 66 0 1 6 0
1536 d 6 0
1537 67 -66 0 1 8 0
1537 d 8 0
1539 -70 68 0 1 13 18 0
1539 d 13 18 0
1540 -290 226 0 1 770 0
1540 d 770 0
1541 290 -226 0 1 772 0
1541 d 772 0
1542 -291 290 0 1 774 0
1542 d 774 0
1543 291 -290 0 1 776 0
1543 d 776 0
1547 -296 295 0 1 781 779 786 791 0
1547 d 791 0
1548 296 -295 0 1 781 779 786 793 0
1548 d 793 0
1550 -299 297 0 1 781 779 786 798 803 0
1550 d 781 779 786 798 803 0
1552 256 -444 0 1288 1296 1291 1292 1280 761 762 663 1271 1279 1274 1275 1263 758 759 659 1254 1262 1257 1258 1246 755 756 655 1237 1245 1240 1241 1229 752 753 651 1220 1228 1223 1224 1212 749 750 647 1203 1211 1206 1207 1195 746 747 643 1186 1194 1189 1190 1178 743 744 639 1169 1177 1172 1173 1161 740 741 635 1152 1160 1155 1156 1144 737 738 631 1135 1143 1138 1139 1127 734 735 627 1118 1126 1121 1122 1110 731 732 623 1101 1109 1104 1105 1093 728 729 619 1084 1092 1087 1088 1076 725 726 615 1067 1075 1070 1071 1059 722 723 611 1050 1058 1053 1054 1042 719 720 607 1033 1041 1036 1037 1025 716 717 603 1016 1024 1019 1020 1008 713 714 599 999 1007 1002 1003 991 710 711 595 982 990 985 986 974 707 708 591 965 973 968 969 957 704 705 587 948 956 951 952 940 701 702 583 931 939 934 935 923 698 699 579 914 922 917 918 906 695 696 575 897 905 900 901 889 692 693 571 880 888 883 884 872 689 690 567 863 871 866 867 855 686 687 563 846 854 849 850 838 683 684 559 829 837 832 833 821 680 681 555 812 820 815 816 1550 804 795 677 678 551 0
1552 d 1288 1292 0
1553 -258 -450 0 674 675 547 1540 1542 1314 1536 3 0
1553 d 547 3 0
1554 -68 258 0 10 11 676 0
1554 d 10 11 676 0
1555 -34 -450 0 1553 1554 12 5 1537 1315 1543 1541 549 0
1555 d 1553 5 549 0
1556 2 -450 0 1555 2 546 1536 1540 1314 1542 0
1556 d 2 546 1536 1540 1314 1542 0
1557 -450 0 1556 1555 4 548 1537 1541 1315 1543 0
1557 d 1556 1555 4 548 1537 1541 1315 1543 0
1559 -73 259 0 27 28 679 0
1559 d 27 28 679 0
1563 258 -71 -227 -451 0 1554 1539 789 1548 1319 25 0
1563 d 789 25 0
1564 -3 35 -451 0 21 552 1563 788 674 675 1547 12 1318 16 24 0
1565 -258 35 -451 0 674 675 1564 12 19 550 16 790 26 1548 1319 0
1566 35 -451 0 1565 1554 1539 1564 19 550 23 787 1318 1547 0
1566 d 1565 1564 0
1567 227 258 -451 0 1566 1554 553 1539 20 787 1547 1318 23 0
1567 d 787 23 0
1568 258 -451 0 1566 1567 551 22 1563 0
1568 d 1567 1563 0
1569 -227 -451 0 1568 1566 674 675 551 12 22 16 788 24 1547 1318 0
1569 d 788 24 1547 1318 0
1570 -451 0 1569 1568 674 675 12 16 1566 553 20 790 26 1548 1319 0
1570 d 1569 1568 1566 790 26 1548 1319 0
1572 -260 -76 0 680 681 37 0
1572 d 37 0
1573 -36 -76 228 0 1572 682 557 0
1573 d 557 0
1574 -76 228 0 1573 36 556 0
1574 d 1573 36 556 0
1575 228 -75 -259 299 -452 0 1574 807 43 810 1323 0
1575 d 807 0
1576 36 76 -228 0 38 554 0
1576 d 38 554 0
1577 76 -228 0 1576 555 39 0
1577 d 1576 555 39 0
1578 -75 -259 299 -452 0 1575 1577 41 1322 808 805 0
1578 d 1575 805 0
1579 227 -35 -452 0 795 553 1550 29 33 1559 1578 0
1579 d 1578 0
1580 76 -75 259 -299 -452 0 1577 43 804 1323 811 0
1580 d 811 0
1581 -75 259 -299 -452 0 1580 1574 806 809 1322 41 0
1581 d 1580 41 0
1582 -258 -35 -452 0 674 675 1579 12 797 551 16 801 677 1581 34 32 22 0
1582 d 22 0
1583 -76 -35 -452 0 1574 1582 1579 796 1554 551 1550 1539 677 30 1559 806 35 810 42 1323 0
1584 -35 -452 0 1583 1577 1582 796 1554 1550 1539 30 1579 551 677 1559 804 35 808 40 1322 0
1584 d 1583 1582 1579 551 0
1585 76 75 299 -452 0 1577 1584 678 804 40 808 1322 0
1585 d 40 808 1322 0
1586 299 75 -452 0 1584 678 1585 1574 42 806 1323 810 0
1586 d 1585 1323 810 0
1587 75 -452 0 1584 34 1586 1550 795 796 550 674 675 21 12 32 16 0
1587 d 34 1586 1550 795 796 550 674 675 21 12 32 16 0
1588 -452 0 1587 1584 678 1559 35 30 31 1539 19 1554 552 797 801 1581 0
1588 d 1587 1584 1581 0
1590 35 -75 297 0 678 1559 35 30 31 1539 1554 797 552 19 0
1590 d 552 19 0
1591 -74 -35 297 0 30 31 1539 20 1554 553 797 0
1591 d 30 31 1539 20 1554 553 797 0
1593 -37 81 -229 0 56 559 0
1593 d 559 0
1594 81 -229 0 1593 55 558 0
1594 d 1593 55 558 0
1595 -305 -80 260 -304 -453 0 826 821 1326 1594 58 0
1596 37 -81 229 0 53 560 0
1596 d 53 560 0
1597 -81 229 0 1596 54 561 0
1597 d 1596 561 0
1598 -304 -80 260 -453 0 1595 823 1597 60 1327 828 0
1598 d 1595 828 0
1599 -76 -75 -259 -453 0 1574 1572 49 51 1598 814 818 0
1599 d 814 818 0
1600 -78 260 0 44 45 682 0
1600 d 44 45 682 0
1602 229 80 260 304 -453 0 1597 821 825 1326 57 0
1602 d 57 0
1603 80 260 304 -453 0 1602 1594 823 59 827 1327 0
1603 d 1602 0
1604 260 79 304 -453 0 1600 1603 52 0
1604 d 1600 1603 52 0
1605 -229 -260 304 -453 0 1594 680 681 46 50 58 822 1326 825 0
1605 d 822 1326 825 0
1606 304 79 -453 0 1604 680 681 46 50 1605 1597 60 824 827 1327 0
1606 d 1604 1605 824 1327 0
1607 299 -75 -453 0 815 801 1590 1591 35 1559 1599 48 1577 1606 812 820 0
1607 d 801 1590 1591 35 1559 0
1608 304 -75 -453 0 1606 1607 48 819 1574 1599 817 806 0
1608 d 819 1574 1599 817 806 0
1609 -79 -304 -453 0 48 51 1572 1598 0
1609 d 1598 0
1610 -75 -453 0 1588 1607 1608 1609 49 1577 43 1324 809 816 820 812 0
1610 d 1607 1608 1609 49 43 812 0
1611 -259 73 0 677 678 29 0
1611 d 677 678 29 0
1612 -453 0 1588 1610 33 47 1611 1606 813 820 815 816 804 809 1577 1324 42 0
1612 d 1610 33 47 1611 1606 813 820 815 816 804 809 1577 1324 42 0
1614 -262 -86 0 686 687 71 0
1614 d 71 0
1615 -38 -86 230 0 1614 688 565 0
1615 d 565 0
1616 -86 230 0 1615 70 564 0
1616 d 1615 70 564 0
1617 230 -85 -261 309 -454 0 1616 841 77 844 1331 0
1617 d 841 0
1618 38 86 -230 0 72 562 0
1618 d 72 562 0
1619 86 -230 0 1618 73 563 0
1619 d 1618 563 0
1620 -85 -261 309 -454 0 1617 1619 839 75 842 1330 0
1620 d 1617 0
1621 -37 81 309 -454 0 56 63 685 67 1620 0
1621 d 56 63 685 67 1620 0
1622 86 85 261 309 -454 0 1619 838 842 1330 74 0
1622 d 74 0
1623 85 261 309 -454 0 1622 1616 76 840 1331 844 0
1623 d 1622 0
1624 37 84 309 -454 0 684 1623 69 62 0
1624 d 62 0
1625 309 84 -454 0 1624 1621 54 683 61 1623 69 0
1625 d 1624 1621 61 1623 69 0
1627 309 79 -454 0 1612 836 1625 65 1597 835 64 58 1329 831 823 827 834 0
1627 d 835 64 831 0
1631 -261 -309 0 1612 683 684 54 1594 829 837 832 833 821 826 680 681 1328 46 60 50 0
1631 d 683 684 54 0
1632 229 -308 0 1612 1597 832 833 826 1328 60 50 821 680 681 46 0
1632 d 60 821 0
1633 86 -85 -309 -454 0 1619 1631 838 77 845 1331 0
1633 d 77 845 1331 0
1634 -85 -309 -454 0 1631 1633 1616 75 840 843 1330 0
1634 d 1633 1330 0
1635 -308 84 0 1612 832 833 1632 1594 66 826 59 1328 0
1635 d 832 833 1632 826 59 1328 0
1636 -309 -454 0 1634 68 1635 837 829 830 1594 680 681 66 46 50 0
1636 d 1634 68 1635 837 829 830 1594 680 681 66 46 50 0
1637 -454 0 1612 1636 1627 1625 836 48 51 65 1572 1597 58 823 1329 827 834 0
1637 d 1636 1627 1625 836 48 51 65 1572 1597 58 823 1329 827 834 0
1641 -39 91 -231 0 90 567 0
1641 d 567 0
1642 91 -231 0 1641 89 566 0
1642 d 1641 89 566 0
1643 -315 -90 262 -314 -455 0 860 855 1334 1642 92 0
1644 39 -91 231 0 87 568 0
1644 d 87 568 0
1645 -91 231 0 1644 88 569 0
1645 d 1644 569 0
1646 -314 -90 262 -455 0 1643 857 862 1645 1335 94 0
1646 d 1643 0
1648 -88 262 0 78 79 688 0
1648 d 78 79 688 0
1649 -38 86 88 0 80 73 0
1649 d 73 0
1650 231 90 262 314 -455 0 1645 855 859 1334 91 0
1650 d 91 0
1651 90 262 314 -455 0 1650 1642 857 93 861 1335 0
1651 d 1650 0
1652 262 89 314 -455 0 1648 1651 86 0
1652 d 1648 1651 86 0
1653 -231 -262 314 -455 0 1642 686 687 80 84 92 856 1334 859 0
1653 d 856 1334 859 0
1654 314 89 -455 0 1652 686 687 80 84 1653 1645 94 858 861 1335 0
1654 d 1652 1653 858 1335 0
1655 86 313 -455 0 82 1619 1654 854 846 0
1655 d 846 0
1656 314 309 -455 0 1637 853 1654 1655 1616 852 81 848 840 75 844 1333 0
1656 d 852 81 848 75 844 1333 0
1657 309 -455 0 1637 849 1656 854 847 1655 1616 1614 1646 85 83 76 1332 839 842 0
1657 d 849 1656 1655 839 842 0
1658 -314 -455 0 1637 1657 1631 847 854 850 838 843 1619 1614 1332 76 1646 85 83 0
1658 d 847 854 850 838 843 1619 1332 76 1646 83 0
1659 -455 0 1658 1657 1654 82 853 1631 851 1616 840 0
1659 d 1658 1657 1654 853 1631 851 1616 840 0
1661 -264 -96 0 692 693 105 0
1661 d 105 0
1662 -40 -96 232 0 1661 694 573 0
1662 d 573 0
1663 -96 232 0 1662 104 572 0
1663 d 1662 104 572 0
1664 232 -95 -263 319 -456 0 1663 875 111 878 1339 0
1664 d 875 0
1665 40 96 -232 0 106 570 0
1665 d 106 570 0
1666 96 -232 0 1665 571 107 0
1666 d 1665 107 0
1667 -95 -263 319 -456 0 1664 1666 109 1338 876 873 0
1667 d 1664 873 0
1668 -39 91 319 -456 0 90 691 97 101 1667 0
1668 d 90 1667 0
1669 96 95 263 319 -456 0 1666 872 108 1338 876 0
1669 d 108 876 0
1670 95 263 319 -456 0 1669 1663 110 874 1339 878 0
1670 d 1669 0
1671 39 94 319 -456 0 96 690 103 1670 0
1672 94 319 -456 0 1671 1668 88 95 689 103 1670 0
1672 d 1671 1668 1670 0
1673 -86 314 -456 0 1659 866 1614 864 871 1672 98 99 92 1645 1337 857 861 0
1673 d 1614 0
1674 319 89 -456 0 1659 870 1672 99 1645 869 98 92 1337 865 857 868 861 0
1674 d 869 98 865 861 0
1675 -96 263 314 -456 0 1663 1673 866 874 82 1674 871 877 863 864 1338 1642 686 687 109 80 102 84 100 0
1676 263 314 -456 0 866 1673 1675 1666 872 82 1674 871 879 863 1339 1642 111 102 100 84 1649 687 864 0
1676 d 1673 1675 82 1649 864 0
1677 314 -456 0 866 1676 689 690 88 1642 99 863 1672 871 0
1677 d 866 1676 0
1678 -263 -314 -319 0 1659 689 690 88 1642 863 871 867 855 860 686 687 1336 80 94 84 0
1679 231 92 -315 0 1645 94 84 855 686 687 80 0
1679 d 94 855 0
1680 96 -95 -319 -456 0 1666 111 1677 1678 872 1339 879 0
1680 d 1339 879 0
1681 -95 -319 -456 0 1677 1678 1680 109 1663 874 877 1338 0
1681 d 1678 1680 1338 0
1682 -231 94 -314 0 1659 1642 100 84 93 1336 860 857 686 687 80 0
1682 d 100 84 93 857 686 687 80 0
1683 -319 -456 0 1659 1677 1681 102 1682 863 871 867 1679 860 1336 0
1683 d 1681 102 1682 863 871 867 1679 860 1336 0
1684 -456 0 1659 1683 1677 1674 1672 870 868 85 99 862 92 1337 0
1684 d 1683 1677 1674 1672 870 868 85 862 92 1337 0
1687 -41 101 -233 0 124 575 0
1687 d 575 0
1688 101 -233 0 1687 123 574 0
1688 d 1687 123 574 0
1689 -325 -100 264 -324 -457 0 894 889 1342 1688 126 0
1689 d 894 0
1690 41 -101 233 0 121 576 0
1690 d 121 576 0
1691 -101 233 0 1690 122 577 0
1691 d 1690 577 0
1692 -324 -100 264 -457 0 1689 891 1691 128 1343 896 0
1692 d 1689 896 0
1693 -96 -95 263 -319 -457 0 1663 1661 117 874 119 1692 885 887 0
1693 d 885 887 0
1694 -98 264 0 112 113 694 0
1694 d 112 113 694 0
1696 233 100 264 324 -457 0 1691 889 893 1342 125 0
1696 d 125 0
1697 100 264 324 -457 0 1696 1688 891 895 1343 127 0
1697 d 1696 127 0
1698 264 99 324 -457 0 1694 120 1697 0
1698 d 1697 0
1699 -233 -264 324 -457 0 1688 692 693 114 118 126 890 1342 893 0
1699 d 890 1342 893 0
1700 324 99 -457 0 1698 692 693 114 118 1699 1691 892 895 128 1343 0
1700 d 1698 1699 892 128 1343 0
1701 263 -95 -319 -457 0 881 1693 1666 116 872 1700 884 888 0
1701 d 1693 0
1702 -263 -231 0 689 690 1642 88 0
1702 d 1642 88 0
1703 -94 -95 -457 0 1684 99 1645 1702 881 1701 883 888 1700 116 1663 874 109 878 1341 0
1703 d 99 1645 1702 1701 109 878 1341 0
1704 324 -263 -457 0 1700 116 1663 882 886 0
1704 d 882 886 0
1705 -96 -95 -457 0 117 1661 119 1703 1692 103 1704 95 96 691 0
1705 d 1703 1692 103 1704 95 96 691 0
1706 -95 -457 0 1684 1705 1666 116 111 880 1700 1340 888 883 884 877 0
1706 d 1705 0
1707 -263 93 0 689 690 97 0
1707 d 689 690 97 0
1708 -457 0 1684 1706 101 115 1707 881 1700 888 883 884 872 877 1666 1340 110 0
1708 d 1706 1700 0
1710 -266 -106 0 698 699 139 0
1711 -42 -106 234 0 139 581 0
1711 d 581 0
1712 -106 234 0 1711 138 580 0
1712 d 1711 138 580 0
1713 234 -105 -265 329 -458 0 909 1712 145 912 1347 0
1713 d 909 0
1714 42 106 -234 0 140 578 0
1714 d 140 578 0
1715 106 -234 0 1714 141 579 0
1715 d 1714 579 0
1716 -105 -265 329 -458 0 1713 1715 143 1346 910 907 0
1716 d 1713 907 0
1717 -41 101 329 -458 0 124 131 135 697 1716 0
1717 d 124 697 1716 0
1718 106 105 265 329 -458 0 1715 906 910 1346 142 0
1718 d 142 0
1719 105 265 329 -458 0 1718 1712 144 908 1347 912 0
1719 d 1718 0
1720 41 104 329 -458 0 696 1719 137 130 0
1720 d 130 0
1721 329 104 -458 0 1720 1717 122 695 129 1719 137 0
1721 d 1720 1717 129 1719 137 0
1722 95 -232 -458 0 1684 101 1666 1707 1661 1694 898 115 120 132 1721 905 900 110 1340 874 877 883 888 881 0
1722 d 115 110 874 881 0
1723 329 -232 -458 0 1708 1721 1666 132 133 1661 1691 904 126 891 1345 902 895 0
1723 d 904 126 891 1345 902 895 0
1724 -106 -232 265 -458 0 1712 1666 1723 1722 908 1661 117 911 898 119 1346 905 143 901 136 889 134 1688 0
1725 265 -232 -458 0 1666 1661 898 1723 905 901 889 1688 1722 1724 1715 906 913 1347 145 136 134 117 119 0
1725 d 1723 1722 1724 117 119 0
1726 -232 -458 0 1666 1661 898 1725 695 696 122 1688 133 889 1721 901 905 0
1726 d 1666 1725 0
1727 -265 -329 0 1684 695 696 122 1688 897 905 900 901 889 692 693 1661 571 880 888 883 884 872 877 1707 1340 101 111 0
1727 d 122 901 889 1661 571 0
1728 -323 232 0 1684 1663 883 884 872 877 1707 1340 101 111 0
1728 d 883 884 872 877 1707 1340 101 111 0
1729 106 -329 -458 0 1715 1726 1727 906 1728 880 888 900 905 897 898 1688 692 693 114 118 134 136 145 913 1347 0
1729 d 913 1347 0
1730 -329 -458 0 1727 1726 1729 1712 908 911 1728 880 1346 143 888 136 900 905 897 898 1688 692 693 134 114 118 0
1730 d 1729 1728 880 1346 888 136 900 905 897 898 1688 692 693 134 114 118 0
1731 -458 0 1730 1726 1721 903 1663 132 133 116 120 1691 1694 899 0
1731 d 1730 1726 1721 903 1663 132 133 116 120 1691 1694 899 0
1733 -106 -105 265 329 0 1731 1712 908 143 912 1349 0
1733 d 143 912 1349 0
1734 -105 265 329 0 1731 1733 1715 906 145 1348 910 0
1734 d 1733 145 910 0
1735 -267 -111 0 701 702 156 0
1735 d 156 0
1736 -43 -111 235 0 585 1735 703 0
1737 -111 235 0 1736 155 584 0
1737 d 1736 155 584 0
1738 235 -110 -266 334 -459 0 1737 926 162 929 1351 0
1738 d 926 0
1739 43 111 -235 0 157 582 0
1739 d 157 582 0
1740 111 -235 0 1739 583 158 0
1740 d 1739 583 158 0
1741 -110 -266 334 -459 0 1738 1740 924 160 927 1350 0
1741 d 1738 0
1742 -42 106 334 -459 0 141 148 152 700 1741 0
1742 d 141 700 1741 0
1743 111 110 266 334 -459 0 1740 923 927 1350 159 0
1743 d 159 0
1744 110 266 334 -459 0 1743 1737 161 925 1351 929 0
1744 d 1743 0
1745 42 109 334 -459 0 699 1744 154 147 0
1745 d 147 0
1746 109 334 -459 0 1745 1742 1710 139 1744 146 154 0
1746 d 1745 1742 139 1744 146 154 0
1747 265 329 -459 0 915 917 922 1746 1734 149 0
1747 d 1734 149 0
1748 334 -265 -459 0 1746 150 1712 920 916 0
1748 d 920 916 0
1749 -111 329 -459 0 1737 1747 917 695 696 1748 131 922 135 914 1715 1710 151 925 153 928 160 1350 0
1750 329 -459 0 917 1749 1740 1747 1748 922 695 696 914 1715 1710 923 930 1351 162 153 151 131 135 0
1750 d 917 1749 1747 695 696 914 131 135 0
1751 111 -334 -459 0 1731 1740 1750 1727 915 922 918 906 911 1715 1348 1710 144 923 151 153 162 930 1351 0
1751 d 930 1351 0
1752 -334 -459 0 1731 1750 1751 1737 1727 915 922 918 906 911 1715 1348 1710 925 928 144 1350 160 151 153 0
1752 d 1751 1727 915 922 918 906 911 1715 1348 1710 144 1350 151 153 0
1753 -459 0 1752 1750 1748 1746 921 919 908 150 1712 0
1753 d 1752 1750 1748 1746 921 919 908 150 1712 0
1755 -111 -110 266 334 0 1753 1737 925 160 929 1353 0
1755 d 160 929 1353 0
1756 266 -110 334 0 1753 1755 1740 162 923 1352 927 0
1756 d 1755 0
1757 -268 -116 0 704 705 173 0
1757 d 173 0
1758 -44 -116 236 0 589 1757 706 0
1759 -116 236 0 1758 172 588 0
1759 d 1758 172 588 0
1760 236 -115 -267 339 -460 0 1759 943 179 946 1355 0
1761 44 116 -236 0 174 586 0
1761 d 174 586 0
1762 116 -236 0 1761 587 175 0
1762 d 1761 175 0
1763 -115 -267 339 -460 0 1760 1762 177 1354 944 941 0
1763 d 1760 941 0
1764 -43 111 339 -460 0 1740 585 703 165 169 1763 0
1764 d 585 1763 0
1765 116 115 267 339 -460 0 1762 940 176 1354 944 0
1765 d 176 944 0
1766 115 267 339 -460 0 1765 1759 178 942 1355 946 0
1766 d 1765 0
1767 111 338 -460 0 1740 167 931 939 1764 164 702 171 1766 0
1767 d 167 931 1764 0
1768 -116 -115 267 -339 -460 0 1759 177 942 945 1354 0
1768 d 1354 0
1769 -339 -115 267 -460 0 1768 1762 179 940 1355 947 0
1769 d 1768 1355 947 0
1770 -110 334 -460 0 934 1767 1737 1735 168 170 1769 937 933 1756 0
1770 d 1756 0
1771 334 -460 0 1753 934 1767 1737 1735 1770 161 1352 166 927 924 932 939 1766 171 163 164 703 0
1771 d 934 1770 166 927 924 932 939 1766 171 163 164 703 0
1772 339 -460 0 1771 937 938 1767 1737 936 925 933 0
1772 d 937 938 936 933 0
1773 -267 -460 0 1753 1771 1735 1740 1767 935 923 928 698 699 1352 148 162 152 0
1773 d 1735 0
1774 111 -460 0 1753 1771 1740 1767 935 928 1352 923 698 699 148 152 162 0
1774 d 1740 1767 935 923 162 0
1775 -460 0 1753 1774 1773 1772 1771 1737 1769 170 168 152 161 1352 928 925 698 699 148 0
1775 d 1774 1773 1772 1771 1737 1769 170 168 152 161 1352 928 925 698 699 148 0
1778 -269 -121 0 707 708 190 0
1778 d 190 0
1779 -45 -121 237 0 593 1778 709 0
1780 -121 237 0 1779 189 592 0
1780 d 1779 189 592 0
1781 237 -120 -268 344 -461 0 1780 960 196 963 1359 0
1782 45 121 -237 0 191 590 0
1782 d 191 590 0
1783 121 -237 0 1782 591 192 0
1783 d 1782 591 192 0
1784 -120 -268 344 -461 0 1781 1783 194 1358 961 958 0
1784 d 1781 958 0
1785 -44 116 344 -461 0 1762 589 706 182 1784 186 0
1785 d 589 0
1786 121 120 268 344 -461 0 1783 957 193 1358 961 0
1786 d 193 961 0
1787 120 268 344 -461 0 1786 1780 195 959 1359 963 0
1787 d 1786 963 0
1788 116 267 -461 0 949 1762 184 940 952 956 1785 181 705 188 1787 0
1788 d 940 952 0
1789 -121 -120 268 -344 -461 0 1780 959 962 194 1358 0
1789 d 194 1358 0
1790 -344 -120 268 -461 0 1789 1783 196 957 1359 964 0
1790 d 1789 1359 964 0
1792 -118 268 0 180 181 706 0
1792 d 180 181 706 0
1793 -120 267 -461 0 1775 1788 1759 1757 942 1792 1790 188 955 183 953 177 946 1357 0
1794 267 -461 0 1775 949 1793 187 1788 1759 1757 185 942 1787 956 951 945 1356 178 0
1794 d 949 1793 1788 178 0
1795 120 -461 0 1775 1794 701 702 165 169 187 185 179 1356 1762 948 943 945 951 956 1785 705 1787 0
1795 d 187 185 1762 1785 1787 0
1796 -268 -461 0 1775 704 705 1795 1794 1757 1784 587 701 702 943 948 165 956 169 951 179 945 1356 0
1796 d 1757 1784 587 701 702 943 948 165 956 169 951 179 945 1356 0
1797 -461 0 1796 1792 1795 188 184 1759 1794 950 954 1790 0
1797 d 1796 1795 1794 1790 0
1799 344 -119 0 1775 954 955 183 184 1759 177 950 1357 942 946 953 0
1799 d 954 955 183 184 1759 177 950 1357 942 946 953 0
1800 -270 -126 0 710 711 207 0
1800 d 710 0
1801 -46 -126 238 0 207 597 0
1801 d 597 0
1802 -126 238 0 1801 206 596 0
1802 d 1801 206 596 0
1803 238 -125 -269 349 -462 0 1802 977 213 980 1363 0
1803 d 977 0
1804 46 126 -238 0 208 594 0
1804 d 208 594 0
1805 126 -238 0 1804 209 595 0
1805 d 1804 595 0
1806 -125 -269 349 -462 0 1803 1805 211 1362 978 975 0
1806 d 1803 975 0
1807 -45 121 349 -462 0 1783 593 709 199 1806 203 0
1807 d 593 0
1808 126 125 269 349 -462 0 1805 974 210 1362 978 0
1808 d 210 978 0
1809 125 269 349 -462 0 1808 212 1802 1363 976 980 0
1809 d 1808 0
1810 121 268 -462 0 1783 201 966 957 969 973 1807 198 708 205 1809 0
1810 d 957 969 1807 0
1811 -126 -125 269 -349 -462 0 211 1802 976 979 1362 0
1811 d 1362 0
1812 -349 -125 269 -462 0 1811 1805 213 974 1363 981 0
1812 d 1811 1363 981 0
1813 268 -120 -462 0 1810 1780 1778 202 959 204 1812 1792 972 188 970 1799 0
1813 d 1792 972 188 970 1799 0
1814 349 -125 -268 -462 0 971 1806 967 1780 201 205 197 198 709 0
1814 d 971 1806 967 201 0
1815 -125 -120 -462 0 1797 1813 1814 1812 1778 1783 196 960 965 1360 962 973 968 0
1815 d 1814 1812 0
1816 -120 -462 0 1797 1815 203 1813 204 202 1783 196 1360 960 962 968 965 973 1809 707 708 199 0
1816 d 1815 1813 204 202 1783 196 960 965 0
1817 -269 123 0 707 708 199 0
1817 d 707 708 199 0
1818 -268 118 0 704 705 182 0
1818 d 704 705 182 0
1819 -462 0 1797 1816 186 200 1818 1810 966 1780 1778 195 959 1360 962 968 973 1809 205 197 198 709 0
1819 d 1816 186 200 1818 1810 966 1780 1778 195 959 1360 962 968 973 1809 205 197 198 709 0
1822 -271 -131 0 713 714 224 0
1822 d 224 0
1823 -47 -131 239 0 601 1822 715 0
1824 -131 239 0 1823 223 600 0
1824 d 1823 223 600 0
1825 239 -130 -270 354 -463 0 1824 994 230 997 1367 0
1825 d 994 0
1826 47 131 -239 0 225 598 0
1826 d 225 598 0
1827 131 -239 0 1826 599 226 0
1827 d 1826 599 226 0
1828 -130 -270 354 -463 0 1825 1827 992 228 995 1366 0
1828 d 1825 0
1829 -46 126 354 -463 0 209 216 712 220 1828 0
1829 d 209 216 712 220 1828 0
1830 131 130 270 354 -463 0 1827 991 995 1366 227 0
1830 d 227 0
1831 130 270 354 -463 0 1830 1824 229 993 1367 997 0
1831 d 1830 0
1832 126 353 -463 0 1805 982 990 1829 215 218 222 1831 711 0
1832 d 1829 218 711 0
1833 -131 -130 270 -354 -463 0 1824 228 993 996 1366 0
1833 d 1366 0
1834 -354 -130 270 -463 0 1833 1827 991 230 998 1367 0
1834 d 1833 230 998 1367 0
1836 -128 269 -463 0 214 215 207 1832 1805 986 974 0
1836 d 214 215 207 0
1837 126 269 -463 0 1832 1805 986 974 0
1838 125 -463 0 1819 203 1817 217 1837 1802 1800 976 983 1836 212 1364 979 985 990 222 1831 0
1838 d 217 1837 983 1836 212 222 1831 0
1840 -126 -463 0 1819 1800 1838 1802 211 219 1365 221 1834 988 989 984 976 987 980 0
1840 d 1802 211 219 1365 221 1834 988 989 984 976 987 980 0
1841 -463 0 1819 1840 1838 213 1364 1832 985 986 979 0
1841 d 1840 1838 1832 0
1843 -353 126 0 1819 1805 985 986 979 974 1364 1817 213 203 0
1843 d 985 986 979 974 1364 1817 213 203 0
1844 -272 -136 0 716 717 241 0
1845 -48 -136 240 0 241 605 0
1845 d 605 0
1846 -136 240 0 1845 240 604 0
1846 d 1845 240 604 0
1847 240 -135 -271 359 -464 0 1846 1011 247 1014 1371 0
1848 48 136 -240 0 242 602 0
1848 d 242 602 0
1849 136 -240 0 1848 603 243 0
1849 d 1848 603 243 0
1850 -135 -271 359 -464 0 1847 1849 245 1370 1012 1009 0
1850 d 1847 1009 0
1851 -47 131 359 -464 0 1827 601 715 233 1850 237 0
1851 d 601 0
1852 136 135 271 359 -464 0 1849 1008 244 1370 1012 0
1852 d 244 1012 0
1853 135 271 359 -464 0 1852 1846 246 1010 1371 1014 0
1853 d 1852 0
1854 131 358 -464 0 1827 235 999 1007 1851 232 714 239 1853 0
1854 d 235 999 1851 0
1855 -136 -135 271 -359 -464 0 1846 245 1010 1013 1370 0
1855 d 1370 0
1856 -359 -135 271 -464 0 1855 1849 247 1008 1371 1015 0
1856 d 1855 1371 1015 0
1857 -130 126 -464 0 1841 1805 1843 982 990 1002 1854 1824 1822 228 236 1369 238 997 1856 993 1005 1001 0
1857 d 1005 1001 0
1858 126 -464 0 1841 1805 1843 1857 234 982 990 1002 1854 1824 1822 229 1368 995 992 1000 1007 1853 239 231 232 715 0
1858 d 1805 1843 1857 982 990 995 992 0
1859 271 -135 -464 0 1841 1858 1800 1856 1006 1854 1824 993 1004 997 1369 228 234 239 231 232 715 0
1859 d 1856 1006 1854 1824 993 1004 997 1369 228 234 239 231 232 715 0
1860 -271 -464 0 713 714 1858 233 1800 237 1000 1822 1827 991 1003 1007 1850 0
1860 d 1822 1850 0
1861 -464 0 1841 1860 1858 1859 1800 238 1853 1000 1007 1002 1003 991 996 1827 1368 229 236 0
1861 d 1860 1858 1859 1800 238 1853 1000 1007 1002 1003 991 996 1827 1368 229 236 0
1891 361 -137 0 1861 1372 0
1891 d 1372 0
1892 -361 137 0 1861 1373 0
1892 d 1373 0
1893 465 -496 0 1557 1570 1444 1588 1447 1612 1450 1637 1453 1659 1456 1684 1459 1708 1462 1731 1465 1753 1468 1775 1471 1797 1474 1819 1477 1841 1480 1861 1483 1486 0
1893 d 1444 1447 1450 1453 1456 1459 1462 1465 1468 1471 1474 1477 1480 1483 1486 0
1894 511 481 0 1535 1534 0
1894 d 1534 0
1896 -274 -242 0 722 723 611 0
1896 d 611 0
1897 -275 -243 0 725 726 615 0
1897 d 615 0
1898 -276 -244 0 728 729 619 0
1898 d 619 0
1899 -277 -245 0 731 732 623 0
1899 d 623 0
1900 -278 -246 0 734 735 627 0
1900 d 627 0
1901 -279 -247 0 737 738 631 0
1901 d 631 0
1902 -280 -248 0 740 741 635 0
1902 d 635 0
1903 -281 -249 0 743 744 639 0
1903 d 639 0
1904 -282 -250 0 746 747 643 0
1904 d 643 0
1905 -283 -251 0 749 750 647 0
1905 d 647 0
1906 -284 -252 0 752 753 651 0
1906 d 651 0
1907 -285 -253 0 755 756 655 0
1907 d 756 655 0
1908 -286 -254 0 758 759 659 0
1908 d 659 0
1910 -65 221 -257 0 532 671 0
1910 d 532 671 0
1911 221 -257 0 1910 531 670 0
1911 d 1910 531 670 0
1912 -63 211 -255 0 663 498 0
1912 d 663 498 0
1913 211 -255 0 1912 662 497 0
1913 d 1912 662 497 0
1914 -50 146 -242 0 1896 724 277 0
1914 d 277 0
1915 146 -242 0 1914 610 276 0
1915 d 1914 610 276 0
1916 -51 151 -243 0 1897 727 294 0
1916 d 294 0
1917 151 -243 0 1916 293 614 0
1917 d 1916 293 614 0
1918 -52 156 -244 0 1898 730 311 0
1918 d 311 0
1919 156 -244 0 1918 310 618 0
1919 d 1918 310 618 0
1920 -53 161 -245 0 1899 733 328 0
1920 d 328 0
1921 161 -245 0 1920 327 622 0
1921 d 1920 327 622 0
1922 -54 166 -246 0 1900 736 345 0
1922 d 345 0
1923 166 -246 0 1922 626 344 0
1923 d 1922 626 344 0
1924 -55 171 -247 0 1901 739 362 0
1924 d 362 0
1925 171 -247 0 1924 361 630 0
1925 d 1924 361 630 0
1926 -56 176 -248 0 1902 742 379 0
1926 d 379 0
1927 176 -248 0 1926 634 378 0
1927 d 1926 634 378 0
1928 -57 181 -249 0 1903 745 396 0
1928 d 396 0
1929 181 -249 0 1928 395 638 0
1929 d 1928 395 638 0
1930 -58 186 -250 0 1904 748 413 0
1930 d 413 0
1931 186 -250 0 1930 412 642 0
1931 d 1930 412 642 0
1932 -59 191 -251 0 430 1905 751 0
1933 191 -251 0 1932 646 429 0
1933 d 1932 0
1934 -60 196 -252 0 1906 754 447 0
1934 d 447 0
1935 196 -252 0 1934 446 650 0
1935 d 1934 446 650 0
1936 -61 201 -253 0 1907 757 464 0
1936 d 464 0
1937 201 -253 0 1936 463 654 0
1937 d 1936 654 0
1938 -62 206 -254 0 1908 760 481 0
1938 d 481 0
1939 206 -254 0 1938 658 480 0
1939 d 1938 658 480 0
1940 -64 216 -256 0 515 667 0
1940 d 515 667 0
1941 216 -256 0 1940 514 666 0
1941 d 1940 514 666 0
1942 -288 -256 0 764 765 1941 513 0
1942 d 765 0
1944 -49 241 273 0 721 609 0
1944 d 609 0
1945 49 -145 241 0 266 608 273 257 269 0
1945 d 269 0
1946 -145 -140 146 241 242 -366 369 -497 0 281 1945 1944 719 1045 258 1048 264 1379 1375 1489 1893 0
1947 -497 -140 146 241 242 -366 369 0 1946 271 272 278 270 264 1375 1893 1489 1378 1046 1042 719 720 267 0
1947 d 1946 0
1949 -153 275 0 299 300 727 0
1949 d 299 300 727 0
1950 243 373 -379 0 1067 1075 1070 1071 1058 1059 1050 1896 0
1951 -372 -140 -146 241 -243 -366 369 -375 -498 0 1917 1050 1051 1056 719 720 1043 1064 258 267 1046 264 271 1375 279 287 1893 1378 289 1489 296 1492 1382 0
1952 -148 274 0 282 283 724 0
1952 d 282 283 724 0
1953 -243 -140 241 -242 -366 369 -372 -498 0 1896 1915 1061 1951 0
1953 d 1951 0
1955 51 -151 243 0 291 616 0
1955 d 291 616 0
1956 -151 243 0 1955 292 617 0
1956 d 1955 292 617 0
1957 -150 -140 241 -242 -366 369 -498 0 1915 1896 1952 290 285 279 1945 1944 719 1052 1043 258 1056 1046 264 1378 1375 1893 1489 1492 1953 1956 1059 298 1066 1383 0
1957 d 1896 1953 0
1958 -273 143 0 719 720 267 0
1958 d 267 0
1962 53 -161 245 0 325 624 0
1962 d 325 624 0
1963 -161 245 0 1962 326 625 0
1963 d 1962 326 625 0
1964 54 -166 246 0 342 628 0
1964 d 342 628 0
1965 246 -166 0 1964 343 629 0
1965 d 1964 343 629 0
1967 56 -176 248 0 376 636 0
1967 d 376 636 0
1968 248 -176 0 1967 377 637 0
1968 d 1967 377 637 0
1969 -173 279 0 367 368 739 0
1969 d 367 368 739 0
1970 58 -186 250 0 410 644 0
1970 d 410 644 0
1971 250 -186 0 1970 411 645 0
1971 d 1970 645 0
1972 -183 281 0 401 402 745 0
1972 d 401 402 745 0
1973 60 -196 252 0 444 652 0
1973 d 444 652 0
1974 252 -196 0 1973 445 653 0
1974 d 1973 445 653 0
1975 -193 283 0 435 751 436 0
1975 d 435 436 0
1976 -158 276 0 316 317 730 0
1976 d 316 317 730 0
1977 499 157 -162 -244 385 -500 0 1919 314 1493 1498 1491 1391 1100 1090 1091 1086 1949 1078 307 1089 302 303 1073 1074 1956 296 1069 1385 1061 1065 1072 0
1977 d 1100 0
1978 384 -155 -244 467 0 1090 1091 1086 1949 1078 307 1089 302 303 1073 1074 1956 296 1069 1385 1061 1065 1072 0
1979 -383 -381 0 1087 1088 1081 0
1979 d 1081 0
1980 498 -155 -244 -499 0 1491 1495 1919 1978 313 1386 1979 1092 1085 1897 1067 1077 1080 1950 1075 1053 1054 1071 1059 722 723 275 1915 1042 719 720 607 1033 1041 1036 1037 1025 1844 1849 1016 1024 1019 1020 1013 1008 1891 713 714 247 233 237 0
1980 d 1077 0
1981 -379 243 0 1067 1075 1950 1053 1054 1071 1059 722 723 275 1915 1042 719 720 607 1033 1041 1036 1037 1025 1844 1849 1016 1024 1019 1020 1013 1008 1891 713 714 247 233 237 0
1981 d 1067 1950 1071 0
1982 -271 133 0 713 714 233 0
1982 d 713 714 233 0
1983 -166 165 277 389 -470 0 1965 348 1112 1395 1116 0
1984 165 277 389 -470 0 1983 1923 1110 1114 1394 346 0
1984 d 1983 346 0
1985 277 245 389 -470 0 1984 1963 337 341 333 334 733 0
1986 166 -277 389 -470 0 1923 731 732 335 339 349 1395 1113 1116 0
1986 d 1113 1116 0
1987 -470 245 389 0 1985 731 732 1986 1965 1111 1114 1394 347 335 339 0
1987 d 1985 731 732 1986 335 339 0
1989 -257 -220 288 -444 -481 0 1911 1299 1302 534 1438 0
1989 d 1302 0
1990 65 -221 257 0 529 672 0
1990 d 529 672 0
1991 -221 257 0 1990 530 673 0
1991 d 1990 530 673 0
1992 -481 -220 288 -444 0 1989 1991 1297 536 1439 1304 0
1992 d 1989 1304 0
1993 -63 -210 -215 255 -256 -436 439 509 0 665 1942 1941 496 763 525 517 502 1281 1290 527 1431 1284 1294 1528 1434 1992 1531 1894 0
1994 439 -210 -215 255 -256 -436 509 0 1993 504 664 511 495 507 0
1994 d 1993 0
1998 444 -256 -439 0 1294 1295 1290 1293 1282 0
1999 -287 -255 0 761 1913 496 762 0
1999 d 762 0
2000 -208 -255 -436 478 0 486 487 760 479 1908 1264 1254 1267 1262 1257 1258 1251 1246 1428 755 1907 485 656 475 471 0
2003 254 -434 0 1254 1262 1257 1258 1246 1907 1237 1245 1240 1241 1229 1906 1220 1228 1223 1224 1212 1905 1203 1211 1206 1207 1195 1904 1186 1194 1189 1190 1178 1903 1169 1177 1172 1173 1161 1902 1152 1160 1155 1156 1144 1901 1135 1143 1138 1139 1127 1900 1118 1126 1121 1122 1110 1899 1101 1109 1104 1105 1093 1898 1084 1092 1087 1088 1981 1076 1897 0
2003 d 1254 1258 0
2004 -423 252 0 1223 1224 1212 1905 1203 1211 1206 1207 1195 1904 1186 1194 1189 1190 1178 1903 1169 1177 1172 1173 1161 1902 1152 1160 1155 1156 1144 1901 1135 1143 1138 1139 1127 1900 1118 1126 1121 1122 1110 1899 1101 1109 1104 1105 1093 1898 1084 1092 1087 1088 1981 1076 1897 0
2005 -388 245 0 1104 1105 1093 1898 1084 1092 1087 1088 1981 1076 1897 0
2005 d 1105 0
2007 -138 272 0 248 249 718 0
2007 d 248 249 718 0
2008 -363 135 0 237 1019 1020 1982 1013 1891 246 1849 1008 0
2008 d 246 1008 0
2010 -49 140 241 -274 366 369 -497 0 1944 722 723 719 1958 275 258 271 1915 261 281 1045 1374 1048 1893 1379 1489 0
2011 -497 140 241 -274 366 369 0 722 723 275 1915 2010 1945 608 720 278 257 1042 261 1046 1374 1378 1893 1489 0
2011 d 2010 0
2012 -243 -274 374 -467 0 1917 1060 722 723 1063 284 1382 288 296 0
2013 -467 -274 374 0 722 723 284 288 2012 1956 298 1383 1065 1062 0
2013 d 2012 1062 0
2016 -256 -210 -255 -434 509 0 1942 1941 2003 1999 1913 1908 1282 508 500 1265 510 1276 1268 525 517 1278 1430 527 1998 1285 1528 1992 1434 1894 1531 0
2017 257 -220 288 444 481 0 1991 1297 536 1301 1440 0
2017 d 1440 0
2018 288 -220 444 481 0 2017 1911 1299 534 1303 1441 0
2018 d 2017 1441 0
2019 481 -215 -441 444 510 0 1894 1531 1435 519 525 527 2018 764 1942 668 513 0
2019 d 519 2018 0
2020 -257 -288 444 -481 0 1911 764 1942 668 522 526 1298 1301 1438 534 0
2020 d 1298 534 0
2021 -288 444 -481 0 1942 764 668 2020 1991 522 1300 1303 1439 526 536 0
2021 d 764 2020 522 1300 526 536 0
2022 -213 287 0 503 504 763 0
2022 d 503 504 763 0
2023 -430 431 433 0 1252 1259 0
2023 d 1259 0
2024 -254 430 432 0 1248 1256 0
2024 d 1256 0
2025 -253 425 427 0 1239 1231 0
2026 -62 254 286 0 760 661 0
2026 d 661 0
2027 478 -210 -255 439 0 1277 1273 1265 1278 1276 1260 1261 1269 2000 494 489 490 483 1429 2023 2024 2026 660 478 0
2027 d 1278 1276 2000 0
2028 254 -210 286 0 2026 487 660 494 478 490 0
2028 d 490 0
2029 -198 284 0 452 453 754 0
2029 d 452 453 754 0
2030 -203 285 0 469 470 757 0
2030 d 469 470 757 0
2032 -61 203 253 0 657 471 0
2032 d 471 0
2033 61 -201 253 0 461 656 0
2034 253 -201 0 2033 462 657 0
2034 d 2033 657 0
2035 -59 193 251 0 437 649 0
2035 d 437 649 0
2036 251 -191 193 0 2035 648 427 0
2036 d 648 427 0
2037 -188 282 0 418 419 748 0
2037 d 418 419 748 0
2038 -57 183 249 0 403 641 0
2038 d 641 0
2039 249 -181 183 0 2038 393 640 0
2039 d 2038 393 640 0
2040 -178 280 0 384 385 742 0
2040 d 384 385 742 0
2041 -55 247 279 0 1969 369 633 0
2041 d 633 0
2042 247 -171 279 0 2041 359 632 0
2042 d 2041 359 632 0
2043 -168 278 0 350 351 736 0
2043 d 350 351 736 0
2044 -163 277 0 333 334 733 0
2044 d 333 334 733 0
2045 -52 244 276 0 1976 318 621 0
2045 d 621 0
2046 244 -156 276 0 2045 308 620 0
2046 d 2045 308 620 0
2047 -50 242 274 0 1952 284 613 0
2047 d 613 0
2048 242 -146 274 0 2047 274 612 0
2048 d 2047 274 612 0
2049 -143 273 0 265 266 721 0
2049 d 265 266 721 0
2050 241 -145 273 0 1944 1945 0
2050 d 1945 0
2051 394 -169 0 1125 353 354 1965 1124 1120 2044 1112 341 1123 336 1107 1108 337 1963 1103 1976 1095 324 1106 319 320 1090 1091 2046 1978 1086 1949 1078 307 1089 302 303 1073 296 1956 1069 1382 1952 1061 290 1064 285 286 1056 1057 2048 1052 1044 2049 2050 1055 273 1039 1040 268 1035 2007 1027 256 1038 251 252 1022 1023 1846 245 1018 1010 1892 1021 1014 0
2051 d 1124 1120 337 0
2054 364 -139 0 252 251 1022 1023 1846 245 1018 1010 1892 1021 1014 0
2054 d 251 1022 1023 1846 245 1018 1010 1892 1021 1014 0
2055 -156 244 0 2046 728 729 309 0
2055 d 2046 309 0
2057 -245 -276 384 -469 0 1921 728 729 318 322 330 1390 1097 1094 0
2057 d 1094 0
2058 -469 -276 384 0 728 729 318 322 2057 1963 332 1391 1099 1096 0
2058 d 2057 1096 0
2059 -252 195 283 419 -476 0 1935 1214 1218 1419 450 0
2059 d 450 0
2060 -476 195 283 419 0 2059 1974 1212 448 1216 1418 0
2060 d 2059 448 0
2062 -64 256 288 0 669 766 0
2063 257 220 444 -481 0 2021 1297 1991 1301 533 1438 0
2063 d 1297 1991 1301 533 1438 0
2064 -255 -210 286 -434 478 509 0 1913 1999 1265 508 500 510 2027 2016 1552 1280 1287 1268 1430 1528 2019 2021 2062 521 668 512 524 528 2063 1911 1299 535 1303 1439 0
2064 d 2027 2016 1268 0
2065 -481 256 0 1552 2021 2062 521 668 512 524 528 2063 1911 1299 535 1303 1439 0
2065 d 2021 2062 524 0
2066 -434 -210 -215 255 256 510 0 2003 1271 1552 2065 1527 1908 2019 1263 1275 1270 1279 1433 1286 500 1283 507 2022 511 0
2067 -210 -215 255 256 510 0 1271 2066 1552 2065 1526 1527 1260 1261 1274 2019 1524 1279 1286 1283 2022 511 507 500 1433 1269 1266 2028 1939 2024 2023 1429 483 489 494 486 487 760 0
2067 d 2066 1527 507 1433 760 0
2068 -440 441 443 0 1286 1293 0
2068 d 1293 0
2069 510 -215 256 0 2065 1526 1552 1524 1295 2019 2068 1283 2022 511 506 1287 1277 2067 1273 2028 2064 1939 1260 1261 2024 2023 1429 483 489 494 486 487 479 0
2069 d 1526 1552 1524 1295 2019 2068 1287 2067 0
2071 -254 434 -478 0 1939 1260 1261 2024 2023 1251 1247 1427 1243 1244 2030 484 477 472 473 2034 2025 1239 1242 2029 1226 1227 460 455 456 1974 1222 1214 1975 1225 443 1209 1210 438 439 2036 1205 2037 1197 426 1208 421 422 1192 1193 1971 1188 1180 1972 1191 409 1175 1176 404 405 2039 1171 2040 1163 392 1174 387 388 1158 1159 1968 1154 1969 1146 375 1157 370 371 1141 1142 2042 1137 2043 1129 358 1140 2051 0
2072 -285 203 0 755 1907 2032 656 0
2073 -249 -180 189 -414 0 1903 1929 1187 406 1194 408 423 1931 1178 1190 0
2073 d 1190 0
2074 -412 189 0 1186 1187 1931 743 744 423 403 407 0
2075 249 279 -409 0 1169 1153 1177 1172 1173 1160 1161 1156 1902 1144 0
2076 -247 -170 189 -414 0 1901 1925 2074 1194 1153 372 1189 374 2075 2073 390 391 389 1927 1144 1156 1160 1172 1177 1170 740 741 386 0
2076 d 2075 0
2077 -282 188 0 746 747 420 0
2077 d 420 0
2079 -284 198 0 752 753 454 0
2079 d 752 753 454 0
2080 -280 178 0 740 741 386 0
2080 d 740 741 386 0
2081 -398 247 0 1138 1139 1127 1900 1118 1126 1121 1122 1110 1899 2005 1101 1109 0
2081 d 1139 0
2082 -402 179 0 1152 1153 1927 737 738 389 369 373 0
2083 247 -399 0 2081 1143 1135 0
2083 d 2081 1135 0
2084 -278 168 0 734 735 352 0
2084 d 734 735 352 0
2085 245 -167 -246 -394 0 2005 1101 1109 1121 1987 1126 1119 1396 1111 1114 0
2085 d 1396 1111 1114 0
2086 -383 159 468 0 1087 1088 1979 1981 1897 1076 1919 321 314 1388 0
2086 d 1388 0
2087 -382 159 0 1084 1085 1919 725 726 321 301 305 0
2087 d 1084 1085 321 0
2088 -145 147 149 0 281 287 0
2089 147 149 -370 0 2088 271 280 1958 1915 1042 0
2089 d 2088 0
2090 -140 142 144 0 270 264 0
2090 d 264 0
2091 271 139 -364 0 1017 1024 1019 1020 1013 1891 2008 253 247 0
2091 d 1017 1020 2008 0
2092 139 -364 0 2091 1982 237 253 247 1849 1891 1011 1016 1013 1024 1019 0
2092 d 2091 1982 237 253 247 1849 1891 1011 1016 1013 1024 1019 0
2093 -49 141 -241 0 260 607 0
2093 d 260 607 0
2094 141 -241 0 2093 259 606 0
2094 d 2093 259 606 0
2095 -273 -141 0 719 720 258 0
2095 d 719 720 258 0
2096 -141 -140 149 -374 0 2095 270 1051 272 1058 287 1054 1915 1042 0
2097 -375 149 -152 -364 -379 0 2092 1981 252 255 1060 1068 1075 1070 2096 2094 1033 1844 1025 1037 1041 1053 1058 1050 1051 1915 1958 287 271 0
2097 d 1060 1844 0
2098 -274 148 0 722 723 284 0
2098 d 284 0
2099 145 368 -374 0 285 271 1958 1051 1058 1053 1041 1033 1034 2094 716 717 2096 250 254 0
2099 d 271 1958 0
2100 -383 159 0 1087 1981 1897 1917 1088 1076 1919 1979 2086 1386 313 306 304 288 289 297 2098 1068 1061 1075 1070 2097 1036 2099 287 1915 1050 1058 1053 1054 1041 1042 1033 2095 2094 0
2100 d 1087 1088 1979 2086 2097 0
2102 246 388 -394 0 1118 1126 1121 1122 1109 1110 1101 1899 0
2102 d 1118 0
2103 384 169 -394 0 1104 2102 1923 355 340 348 2085 1899 1921 1119 338 1126 322 1121 1109 1102 728 729 318 0
2103 d 1121 0
2104 -245 -160 169 -394 0 1899 1921 338 340 355 1923 1119 1110 1126 1122 0
2104 d 1119 1110 1126 1122 0
2105 245 169 -394 0 2005 2102 2085 1923 348 355 0
2105 d 2102 2085 1923 348 355 0
2106 -394 169 0 2103 2105 2104 323 2100 2087 1092 0
2106 d 2103 2105 2104 0
2107 -399 189 -414 0 2083 2076 356 357 2084 2106 1136 1138 1143 0
2107 d 2076 0
2108 -368 241 0 1036 2092 252 1037 1025 716 717 241 0
2108 d 1037 0
2109 -374 149 241 0 2108 1033 1041 1053 1058 2099 1050 1915 287 0
2109 d 2099 1050 287 0
2110 279 189 -414 0 1153 2107 2074 1155 1194 1160 1189 1172 1177 1169 1170 2073 2080 390 0
2111 180 403 -409 0 390 391 2080 2082 1160 1172 1177 1170 0
2111 d 1170 0
2112 190 -419 0 424 425 2077 2074 1204 1211 1206 2107 1194 1155 1189 2111 2073 1169 1177 1172 1173 1160 1161 1152 1902 0
2112 d 1204 0
2113 -419 194 0 2112 440 1933 1203 1211 1206 1207 1195 746 747 1904 411 1186 422 1194 2107 1189 1155 2111 2073 1169 1177 1172 1173 1160 1161 1152 1902 0
2113 d 2112 440 1933 1203 1211 1206 1207 0
2114 -286 -206 0 758 759 479 0
2115 -206 254 0 2114 2026 478 660 0
2115 d 2114 2026 478 660 0
2116 -63 213 255 0 665 505 0
2116 d 665 505 0
2117 64 -216 256 0 512 668 0
2117 d 512 668 0
2118 -216 256 0 2117 513 669 0
2118 d 2117 513 669 0
2119 256 210 255 436 440 441 509 0 2065 2118 1283 506 1894 2022 511 2116 516 664 1434 495 1531 499 1528 1430 0
2120 436 210 255 440 441 509 0 2119 1942 1941 1282 761 664 496 2116 499 509 1430 525 517 1528 527 1434 1531 1894 1992 1290 1294 0
2120 d 2119 1290 1294 0
2121 255 210 286 434 441 509 0 1271 1263 1274 1267 1279 1286 2120 0
2121 d 2120 0
2122 441 210 286 434 509 0 506 1272 1274 1279 1286 2121 1265 1913 1999 1269 501 2022 1282 1431 511 2065 2118 1528 1894 516 1531 1434 0
2122 d 2121 0
2123 -218 288 0 520 521 766 0
2123 d 520 521 766 0
2124 -255 210 286 434 509 0 1913 1999 506 2122 1272 1274 2022 1289 1279 511 1291 1284 523 1296 1280 1942 1941 2123 518 528 1435 1265 1269 501 1431 1528 1531 1894 2063 1911 1299 535 1303 1439 0
2124 d 1265 501 0
2125 -256 210 255 436 -440 -441 443 509 0 1942 1941 2123 1281 2022 1289 2116 1296 664 495 499 1430 1528 506 511 523 518 528 1435 1531 1894 2063 1911 1299 535 1303 1439 0
2125 d 1281 506 511 0
2126 436 210 255 -440 -441 443 509 0 2125 1280 761 664 496 2116 509 2069 1528 1430 499 0
2126 d 2125 499 0
2127 286 254 509 0 1272 2003 1274 1279 1291 2028 2122 2124 1284 1263 2126 1267 0
2127 d 2028 0
2128 256 -210 -436 -440 509 0 1280 761 1999 664 2116 509 2069 1528 1431 502 496 0
2128 d 1280 496 0
2129 -441 -210 255 -436 439 509 0 1284 1291 2128 1942 1941 1994 509 510 2116 508 664 502 761 1431 1528 2123 523 518 1289 528 1435 1531 1894 1296 2063 1911 1299 535 1303 1439 0
2129 d 2123 523 518 1289 528 1435 1296 2063 1911 1299 535 1303 1439 0
2130 -256 -210 255 -436 439 509 0 2129 1994 1286 1282 509 2116 664 761 0
2130 d 1994 1286 509 761 0
2131 -436 -210 255 439 509 0 2130 2129 2128 1283 2118 2065 1894 2022 2116 664 495 502 1431 1528 2069 1531 1434 516 0
2131 d 2130 2129 2128 1283 2118 2065 2022 2116 664 495 502 1431 516 0
2132 255 -286 434 509 0 758 759 1274 488 492 1271 1279 2131 1266 1269 0
2132 d 1271 1266 1269 0
2133 509 254 478 0 2003 2127 758 759 2132 488 1264 1267 1273 1913 1999 492 1277 508 500 510 1430 1528 2069 1282 1941 1942 1998 1285 525 517 527 1434 1992 1531 1894 0
2133 d 2127 758 759 2132 488 1264 1273 1913 1999 492 1277 508 500 510 1430 1528 2069 1282 1941 1942 1998 1285 525 517 527 1434 1992 1531 1894 0
2134 -61 200 284 285 424 -477 0 2030 2032 1937 1231 467 1235 1423 0
2134 d 2032 0
2135 29 61 200 284 424 -477 0 461 1937 465 1229 1422 1233 0
2135 d 461 0
2136 61 200 284 424 -477 0 2135 463 656 467 1231 1423 1235 0
2136 d 2135 463 656 467 1231 0
2137 -477 200 284 285 424 0 2136 2134 0
2137 d 2134 0
2140 -208 -254 0 1939 486 487 479 0
2140 d 486 487 479 0
2141 -155 154 -158 379 -468 0 307 1976 1949 1898 1079 2055 1082 315 1387 0
2142 -468 154 -158 379 0 1976 1898 2055 2141 305 312 1386 1080 1076 725 726 301 0
2142 d 2141 0
2143 -243 150 374 -467 0 1917 2013 1061 1065 1383 297 0
2143 d 297 0
2147 278 -279 394 -471 0 2043 737 738 1901 2051 358 360 1127 363 1131 1398 0
2148 -471 -279 394 0 737 738 1901 360 2147 2084 1130 356 1133 366 1399 0
2148 d 2147 0
2149 245 388 -470 0 1101 1109 1987 0
2149 d 1101 1987 0
2150 -470 -168 -276 388 0 2043 728 729 2149 1900 318 1921 1103 1965 322 1107 354 338 2106 340 1125 349 1123 1395 1117 0
2151 -191 251 0 2036 1975 749 750 428 0
2151 d 2036 0
2152 415 -190 282 -414 -475 0 1197 2151 434 1415 1202 0
2152 d 1202 0
2153 -475 -190 282 -414 0 2152 1208 1210 2113 439 432 1414 1200 0
2153 d 2152 1200 0
2154 -413 -180 -249 -409 504 -506 0 1193 1903 2073 421 422 425 415 1971 1904 1180 2153 1516 1513 1410 1183 0
2154 d 1183 0
2155 -189 414 0 422 1192 1971 1188 1180 1972 421 409 404 405 2039 1193 1191 1175 1176 1171 2040 1163 392 1174 387 388 1158 1159 1968 1154 1969 1146 375 1157 370 371 1141 1142 2042 1137 2043 1129 358 1140 2051 0
2155 d 421 1193 0
2156 -179 404 0 387 388 1159 1968 1158 1154 1969 1146 375 370 371 2042 1157 1141 1142 1137 2043 1129 358 1140 2051 0
2156 d 1158 1157 0
2157 -191 -190 414 -475 0 2151 2155 426 2037 432 1414 1199 1196 0
2157 d 432 1196 0
2158 -475 -190 414 0 2155 426 2037 2157 439 2113 1209 1205 434 1415 1201 1198 0
2158 d 2157 434 1198 0
2159 -470 -169 388 0 2149 353 354 1899 347 1965 2044 1112 1394 341 1115 336 1107 1103 1976 1095 324 1106 319 320 1090 1091 2055 1086 1978 1078 1949 1089 307 1073 302 303 1956 296 1069 1382 1952 1061 290 1064 285 286 1056 1057 2048 1052 2049 1044 2050 273 1055 268 1039 1040 1035 1027 2007 1038 256 2054 0
2160 243 -158 -468 0 1981 1956 2142 303 0
2161 498 -150 156 -158 244 -499 0 1491 1495 2160 1897 1917 1076 296 304 1385 306 315 1387 1083 1073 1074 1069 1061 1072 1065 0
2161 d 2160 0
2162 -149 274 372 373 0 285 286 2048 1052 2049 1044 2050 273 1055 268 1039 1040 1035 1027 2007 1038 256 2054 0
2163 -146 149 -241 274 373 -497 0 2048 2094 2095 1044 2089 279 272 270 255 263 2092 1055 1039 1048 1035 1379 1027 1489 1031 1893 1375 0
2163 d 1055 0
2164 369 146 -241 -497 0 1915 2094 2095 1042 1046 1039 1040 2049 1035 1027 2007 1038 2054 1031 256 268 263 1375 1893 1489 1378 278 273 0
2164 d 1039 1040 2049 1035 268 263 273 0
2165 140 -369 0 254 255 2092 1036 1041 1034 716 717 250 0
2165 d 1036 1034 0
2166 364 -241 -366 -369 0 1029 2054 2165 256 2007 1026 0
2166 d 1026 0
2167 -366 -241 -369 0 2166 2092 1030 252 1027 716 717 241 0
2167 d 2166 1030 252 1027 241 0
2168 -241 149 274 373 -497 0 2094 2095 2163 1915 2164 1042 2165 2167 1049 262 270 1374 272 1893 281 1489 1379 0
2168 d 2163 0
2169 140 241 -366 0 254 255 2092 1029 1025 716 717 250 0
2169 d 254 255 2092 1029 1025 716 717 250 0
2170 146 241 -366 -497 0 1915 2169 2108 1033 1041 1947 0
2170 d 1947 0
2171 -366 149 241 274 372 -497 0 2169 2170 2108 1033 2048 1041 1052 1044 2050 2089 1048 272 1379 1489 1893 2090 1375 0
2171 d 2090 1375 0
2172 364 -140 241 366 0 1031 2054 1028 256 2007 0
2172 d 1031 2054 1028 256 2007 0
2173 -364 366 368 0 1032 1038 0
2173 d 1032 1038 0
2174 -141 241 0 2095 1944 257 608 0
2174 d 1944 257 608 0
2175 -146 145 273 274 369 -466 0 2048 280 1379 1048 1044 0
2175 d 280 1044 0
2176 145 273 274 369 -466 0 2175 1915 1042 1046 1378 278 0
2176 d 2175 278 0
2177 273 241 274 -466 0 2108 1033 1041 2176 2050 0
2177 d 2176 2050 0
2179 140 141 241 -465 0 261 2169 1374 0
2180 -273 -151 274 -374 376 -498 0 2095 2094 2108 1033 2109 1041 285 286 289 2048 279 296 1043 1382 1046 1492 1378 1489 1893 2179 1957 2173 2172 0
2180 d 2179 0
2181 241 -369 0 2108 1033 1041 0
2181 d 1033 1041 0
2182 376 -151 274 -374 -498 0 2180 1051 1058 1053 1054 2181 2167 2165 1042 1047 2094 270 262 1915 2096 272 1374 279 289 1893 1378 296 1489 1382 1492 0
2182 d 2180 0
2183 274 -243 -379 -498 0 1917 1061 1068 1075 1070 2182 1064 0
2183 d 2182 1064 0
2184 -374 -274 0 722 723 275 1915 286 2109 2094 2095 1051 1042 1058 1054 0
2185 -241 146 373 -497 0 2094 2095 2164 2167 2165 262 1374 1893 1489 1915 270 1042 272 1049 1379 281 0
2185 d 2164 1915 270 1042 272 1049 281 0
2186 -274 -498 0 722 723 2184 1057 275 2013 1492 2185 2108 2181 2170 2173 2172 2011 0
2186 d 722 723 2184 275 2013 2185 2170 2011 0
2187 -497 149 241 274 372 0 2181 2108 2171 2173 2172 2174 261 1374 1893 1489 2177 1052 1045 1048 2089 1379 0
2187 d 2171 2174 2177 1052 1045 1048 2089 1379 0
2189 -140 -150 241 -242 -498 0 2181 1957 2172 2108 2173 0
2189 d 1957 2172 2108 2173 0
2190 243 -150 274 -374 -467 0 1059 1956 1066 298 1383 0
2190 d 1066 298 1383 0
2191 -243 -158 -498 -499 0 2186 1952 1061 2183 1074 1072 1056 1057 2162 290 2143 1492 2187 2168 0
2192 241 -150 -497 -498 0 2186 1952 290 285 286 279 2181 1053 2162 1051 2095 2048 1043 2189 1046 1378 1489 1893 261 1374 2169 0
2192 d 2181 2048 1043 2189 1046 261 2169 0
2193 -150 243 -374 -498 0 2186 1952 290 2190 1492 285 286 2192 2094 2095 1051 1058 1053 2165 279 1054 2167 262 1047 1374 1378 1893 1489 0
2193 d 2190 285 286 2192 279 1054 2167 262 1047 1374 1378 1893 1489 0
2194 149 -374 0 2109 2094 2096 2095 2165 1051 1053 1058 0
2194 d 2109 2094 2096 2095 2165 1051 1053 1058 0
2195 243 -374 -498 0 2194 289 2193 0
2195 d 2193 0
2196 -498 -158 -499 0 2186 1952 2191 1059 2195 1956 1063 1056 1057 2162 290 295 1382 1492 2187 2168 0
2196 d 2191 0
2197 -499 -158 0 1976 1898 2055 2196 1495 2161 288 289 302 2098 2194 1068 1070 1075 2142 0
2197 d 2196 2161 2142 0
2198 193 191 -476 0 439 2113 443 2060 750 1905 2035 0
2199 196 -195 -283 419 -476 0 1935 451 1419 1215 1218 0
2199 d 1215 1218 0
2200 -283 -195 419 -476 0 2199 1974 449 1418 1213 1216 0
2200 d 2199 1213 1216 0
2201 -476 191 0 439 2113 2198 441 1975 2200 0
2201 d 2198 2200 0
2202 -254 -285 429 508 0 1908 1939 1247 2072 1272 1250 475 491 483 493 1426 1525 2024 1260 2064 1263 1270 1275 2131 1279 0
2202 d 2024 1260 0
2203 -285 -200 284 -424 507 0 2072 1907 1238 475 2034 1229 468 1236 1241 1423 1245 1522 1257 2202 2115 1249 485 2023 1427 1525 2133 0
2204 -207 198 -200 203 206 -424 430 -478 508 0 1427 482 2079 1253 477 1238 1244 473 1242 2034 2025 0
2205 -478 -200 203 206 284 -424 430 508 0 2029 1238 2204 485 1426 476 1250 474 1245 1937 1241 1229 0
2205 d 2204 0
2206 508 -200 203 254 284 -424 430 0 2115 2205 1525 2133 0
2206 d 2205 0
2207 207 -200 206 -424 430 -477 478 0 485 1429 476 1253 474 1244 468 1242 1423 1236 0
2208 478 -200 203 206 -424 427 430 -477 0 2207 1428 482 1250 477 1245 473 1241 466 1234 1422 0
2208 d 2207 0
2209 -207 203 206 284 -424 430 -478 0 1427 482 1238 1253 477 1244 473 1242 2034 2025 0
2209 d 2025 0
2210 254 -200 284 -424 507 0 2115 1238 2203 1246 2030 2206 1522 2208 2209 485 1426 476 1250 474 1245 1937 1241 1229 0
2210 d 2206 2208 0
2211 -478 -200 -254 284 -430 432 0 2071 1238 1939 1262 1257 1251 1245 1426 1241 483 1229 476 1937 474 0
2212 -207 -200 -206 208 286 -424 -430 432 478 507 0 483 1272 476 489 474 494 468 1428 1251 1257 1262 1274 1279 1291 1244 1242 1236 1423 1522 1525 2122 2124 1284 1263 2126 1267 0
2212 d 1428 1244 1242 1236 0
2213 -252 -195 -283 507 0 1935 1906 1222 457 1238 1226 459 2210 2203 1908 2140 1939 1248 1255 2030 1272 2211 2212 1429 484 2023 1252 477 491 1261 1245 473 493 1241 466 1234 1422 1522 1525 2064 1263 1270 1275 2131 1279 0
2213 d 2211 0
2214 207 -200 203 -206 -284 286 424 -430 478 507 0 1429 484 1272 1240 2023 1252 477 491 1261 1245 473 493 1237 466 1230 1233 1422 1522 1525 2064 1263 1270 1275 2131 1279 0
2215 -200 199 -254 285 424 478 507 0 460 1908 2140 1939 1248 1255 2030 1240 2029 1272 2214 483 476 489 474 494 1937 468 1237 1232 1245 1235 1257 1423 1262 1522 1274 1525 1279 2122 2124 1291 1284 1263 2126 1267 0
2215 d 2214 0
2216 478 199 -254 285 424 507 0 1255 2030 1240 1908 2140 1272 2215 458 472 2079 477 1238 2137 1245 1522 1257 1525 1262 489 1274 1279 494 1291 2122 2124 1263 1284 1267 2126 0
2216 d 2215 489 1274 494 1291 2122 2124 1284 1267 2126 0
2217 -254 199 285 424 507 0 1939 1240 1248 1255 2216 2071 1262 1257 1251 1245 1426 1237 1238 483 1937 2079 476 458 474 0
2217 d 2216 0
2218 205 -200 206 428 430 -478 0 476 482 474 1426 1937 1250 1237 1245 0
2219 424 -200 203 206 430 -478 0 1227 1240 1226 2218 477 485 473 1427 2034 1253 1243 1239 2029 460 455 456 1974 1222 1975 1214 443 1225 438 439 1209 1210 2151 1205 2037 1197 426 1208 2155 0
2219 d 1226 2218 1253 1243 1239 0
2220 -205 203 206 -424 430 -478 0 485 2209 1906 2004 1220 1228 0
2220 d 2209 0
2221 -200 203 206 430 -478 0 2219 2220 476 482 474 1426 1937 1250 1237 1245 1241 1229 1906 2004 1220 1228 0
2221 d 2219 2220 0
2223 -422 199 0 1220 1221 1935 750 1905 457 2035 441 0
2223 d 441 0
2224 205 429 -478 0 1257 475 2072 1255 1262 2071 2115 1246 1250 1426 482 0
2224 d 475 2072 1246 482 0
2225 206 -205 430 -478 0 1939 1249 2030 477 2221 472 0
2225 d 1249 2221 472 0
2226 430 -205 -478 0 2225 2115 483 1248 1426 1907 1237 1250 1245 1240 1241 1229 1906 2004 1220 1228 0
2226 d 2225 1250 0
2227 -206 -205 -478 0 2226 2115 483 2071 1247 1255 1426 1262 1251 1257 0
2227 d 2115 483 2071 1247 1426 1262 1251 1257 0
2228 -205 -478 0 2227 1939 2226 485 2003 1261 2023 1427 0
2228 d 2227 2226 485 2003 1427 0
2229 -427 204 0 1237 1238 1937 2079 474 458 0
2230 199 -424 0 2223 1228 1223 2004 2113 1935 442 457 0
2231 -478 0 2228 476 2224 2229 1245 1240 1241 2230 456 459 1974 1906 1229 1937 474 0
2231 d 2228 476 2224 2229 474 0
2233 248 175 279 399 -472 0 1968 1144 1148 1402 380 0
2233 d 380 0
2234 -472 175 279 399 0 2233 1927 1146 382 1150 1403 0
2234 d 2233 0
2237 281 -180 249 250 -406 409 503 -505 0 1972 1178 1971 2039 1182 400 1407 1510 1513 1410 414 409 405 0
2237 d 405 0
2238 250 -180 249 -406 409 503 -505 0 1971 2237 743 744 1181 394 403 1184 400 407 1407 417 1510 1411 1513 0
2238 d 2237 0
2239 -281 -180 -250 -406 409 503 -505 0 743 744 1179 1931 394 403 1182 400 407 1407 415 1510 1410 1513 0
2240 404 -180 -405 503 -505 0 2156 1172 1167 392 2040 1162 1169 1177 1189 2238 2239 1931 1180 1972 1187 1184 2039 1194 400 2155 1407 423 1510 416 1513 1411 0
2240 d 1162 0
2241 -180 403 -405 503 -505 -506 0 2240 1174 1166 1160 1176 1152 1902 1161 1929 1903 406 398 1187 408 1406 1510 1513 2154 1194 1191 2155 1185 423 1411 417 0
2241 d 2240 0
2242 250 185 281 409 -474 0 1178 1182 1410 1971 414 0
2242 d 1178 1971 414 0
2243 -474 185 281 409 0 2242 1931 1180 1184 1411 416 0
2243 d 2242 1180 416 0
2244 404 180 -405 503 -505 0 390 404 2080 1161 1929 1903 1972 409 1159 2111 2243 1513 1510 1167 1407 399 0
2244 d 1159 1167 399 0
2245 180 -405 503 -505 0 391 1506 2082 2244 1160 1155 1156 2083 1149 1901 1404 1144 1927 382 389 0
2245 d 2244 0
2246 -405 503 -505 -506 0 2245 2241 1155 1156 2083 1901 1144 1902 2040 1161 392 1929 1903 2156 398 406 1187 408 1174 1166 1406 1510 1513 1176 2154 1191 1194 1185 2155 1411 423 417 0
2246 d 2245 2241 1174 1166 1176 0
2247 249 -180 -406 409 503 -505 0 1189 2238 1931 2239 1972 1187 2039 1194 2155 423 2243 1513 1510 400 1407 0
2247 d 2238 2239 400 1407 0
2248 503 -180 -403 -505 0 1155 1156 1514 1506 2246 2083 1149 1404 1173 1901 1144 1902 1927 1163 382 389 1169 2156 1177 1168 2247 0
2249 168 -170 -247 500 -502 0 2084 1925 364 358 1129 354 2051 1965 353 347 1132 1398 1504 1501 2159 1394 1108 2005 1115 1899 1112 0
2249 d 353 1132 0
2250 -388 166 469 -470 0 1104 1108 2005 354 1921 2106 1125 1123 1117 1395 349 340 338 323 2100 2087 1092 0
2251 -170 -247 500 -502 0 1925 1496 1497 2249 2043 1900 1965 354 2106 364 1493 1491 1128 1131 1398 1504 1501 2250 2149 1899 2044 2150 1976 1102 1095 1109 1106 1984 341 336 324 319 320 1978 2055 0
2251 d 1496 1497 2249 364 1493 1128 2150 0
2252 170 -399 0 356 357 2084 2106 1136 1138 1143 0
2252 d 1136 1138 1143 0
2253 -276 -500 0 728 729 1898 318 2055 320 2100 2087 1092 2058 1498 2197 0
2253 d 2058 2197 0
2256 159 -384 0 2100 2087 1092 0
2256 d 2100 2087 1092 0
2257 -275 -498 0 2186 1952 1897 1059 2195 1956 1063 1056 1057 2162 290 295 1382 1492 2187 2168 0
2257 d 1897 0
2258 -384 -499 0 2256 319 320 2055 1980 2186 1952 2257 1949 307 302 303 290 1956 1061 2183 1074 1072 1056 1057 2162 0
2258 d 1980 2257 0
2259 -243 378 -498 0 2186 1952 1061 1072 1056 1057 2162 290 2143 1492 2187 2168 0
2259 d 2143 0
2260 -498 378 0 2186 2259 1956 2195 1952 1059 1056 1057 1063 2162 290 295 1382 1492 2187 2168 0
2260 d 2259 1952 1059 1056 1057 1063 2162 290 295 1382 1492 2187 2168 0
2261 -244 382 -499 0 2258 1091 1086 1078 1089 1082 1074 2260 1491 1495 1978 1387 1919 314 0
2261 d 1086 1078 1919 314 0
2262 -154 379 467 0 1074 302 303 1073 296 1956 1385 1069 1061 1065 1072 0
2262 d 302 303 1073 296 1956 1385 1069 1061 1065 1072 0
2263 -155 154 244 379 -468 0 2055 315 1387 307 1082 1949 1079 0
2263 d 307 1082 1949 1079 0
2264 -499 379 382 498 0 2261 1491 1495 2055 2262 2263 305 312 1386 1080 1076 725 726 301 0
2264 d 2262 2263 305 312 1386 1080 1076 725 726 301 0
2265 161 160 276 384 -469 0 1921 1093 329 1390 1097 0
2265 d 329 1097 0
2266 160 276 384 -469 0 2265 1963 1095 331 1099 1391 0
2266 d 2265 331 1099 1391 0
2267 -500 159 499 0 2256 2253 1498 1976 324 2266 0
2267 d 2266 0
2268 -400 -399 -472 0 1149 2252 2083 1402 1901 1925 1144 372 1927 374 381 0
2269 -472 -399 0 2083 1901 1925 2252 2268 372 1151 1403 1146 1968 383 374 0
2269 d 2252 2268 372 1151 1146 374 0
2270 -505 -180 -280 404 503 0 1514 2246 1165 1164 1929 1171 1903 398 1406 1510 1513 406 1175 1187 2154 1194 2155 408 423 417 1411 1185 1191 0
2270 d 1514 1164 1929 1171 1903 398 406 1175 1187 2154 408 1185 1191 0
2272 -283 414 -475 0 749 750 1905 2158 428 424 431 2077 1414 1195 1199 0
2273 -59 414 -475 0 2158 424 2077 2272 1975 751 2035 1197 1201 1415 433 430 0
2273 d 2272 751 2035 430 0
2274 27 282 414 -475 0 2273 2158 646 2151 1195 431 1199 1414 0
2274 d 646 1195 431 1199 1414 0
2275 414 282 -475 0 2158 2273 2274 429 2151 1197 433 1201 1415 0
2275 d 2273 2274 429 433 1201 1415 0
2276 -414 282 -475 0 2153 425 2074 2107 1194 1155 1189 2111 2073 1169 1177 1172 1173 1160 1161 1152 1902 0
2276 d 2153 425 2074 2107 1152 0
2277 -475 282 0 2275 2276 0
2277 d 2275 2276 0
2279 -253 199 -200 424 507 0 2231 1907 460 2217 2029 2133 1525 1522 1230 1233 1422 466 1937 0
2279 d 1230 1937 0
2280 -200 199 424 507 0 2231 1240 2279 460 2034 1237 1245 2029 468 1232 1235 1423 1522 1525 2133 2202 2217 0
2280 d 2279 460 2034 1237 2029 468 1232 1235 1423 0
2281 195 -476 0 2201 2151 1905 442 2060 2113 0
2281 d 442 2060 2113 0
2282 -476 199 475 0 2201 2151 2281 1905 1975 443 438 457 451 1935 1419 1212 1219 1209 1210 1205 2037 1197 426 1208 2155 0
2282 d 2281 1905 457 451 1935 1419 1212 1219 0
2283 -254 199 429 507 508 0 2230 2202 2217 0
2283 d 2202 2217 0
2284 508 199 429 507 0 2231 1525 2283 2133 0
2284 d 2283 2133 0
2285 -477 200 284 424 0 2136 2137 755 1907 462 1229 465 1422 1233 0
2285 d 2136 2137 755 1907 462 1229 465 1233 0
2286 200 506 0 1515 458 459 2282 2230 1519 2079 1238 1240 1245 2284 1522 2285 0
2286 d 458 459 2079 1240 2284 2285 0
2287 -199 422 423 475 0 455 456 1974 1222 1214 1975 1225 443 1209 1210 438 439 2151 1205 2037 1197 426 2155 1208 0
2287 d 1974 1222 1214 1975 1225 443 1209 1210 438 439 2151 1205 2037 1197 426 1208 0
2288 424 422 506 0 2286 1227 1515 2287 2282 1519 2280 0
2288 d 1227 1515 2287 2282 2280 0
2289 422 506 0 2231 2288 2286 1228 2230 455 1223 1224 2004 456 1217 1906 449 1418 1519 1238 2210 2203 1908 1939 2140 1248 1255 2030 1272 2212 1429 484 2023 1252 477 491 1261 1245 473 493 1241 466 1234 1422 1522 1525 2064 1263 1270 1275 2131 1279 0
2289 d 2288 2286 1228 2230 1223 1224 2004 456 1217 1906 449 1418 1238 2210 2203 1908 1939 2140 1248 1255 2030 1272 2212 1429 484 2023 1252 477 491 1261 1245 473 493 1241 466 1234 1422 1522 1525 2064 1263 1270 1275 2131 1279 0
2290 506 0 2289 1220 1221 2223 749 750 455 428 2213 2201 1519 0
2290 d 2289 1220 1221 2223 749 750 455 428 2213 2201 1519 0
2297 -471 -279 0 737 738 1901 2083 1142 2148 2106 1140 357 1134 1399 366 360 0
2297 d 2148 360 0
2298 498 384 -499 0 1090 1091 1495 2264 2261 2055 1981 1089 1917 1083 1387 315 306 304 288 289 2098 2194 1070 1068 1075 0
2298 d 1090 1091 1495 2264 2261 1981 1089 1917 1083 1387 315 306 304 288 289 2098 2194 1068 1075 0
2299 -498 0 2260 2186 1070 1074 2195 2183 0
2299 d 2260 2186 1070 1074 2195 2183 0
2308 -245 -384 -469 0 1921 2256 320 323 2055 1898 1095 330 1098 1390 0
2308 d 330 1098 1390 0
2309 -500 -384 0 2253 2258 2256 1498 319 320 323 2308 2055 313 1093 1963 1977 332 0
2309 d 2253 2258 1498 313 1093 1963 1977 332 0
2310 245 -470 0 2149 2005 0
2310 d 2149 2005 0
2312 -388 -245 469 -470 0 1104 1899 1921 2256 1108 2250 1965 1112 323 338 340 347 1115 1394 0
2312 d 1108 2250 323 347 1115 1394 0
2313 -384 -245 388 0 2256 320 2055 1106 1898 1095 0
2313 d 2256 1106 1898 1095 0
2314 -159 384 0 2299 1491 319 320 1978 2055 0
2314 d 1491 319 320 1978 2055 0
2324 505 475 0 2290 1516 0
2324 d 1516 0
2336 503 -405 -505 0 2290 2246 0
2336 d 2246 0
2339 -499 384 0 2299 2298 0
2339 d 2298 0
2340 -276 -245 393 -470 0 1899 728 729 1103 1921 318 1107 322 1123 338 1117 1112 1965 340 1395 349 0
2340 d 728 729 1103 1921 318 1107 322 1123 338 1117 1112 340 1395 349 0
2341 -470 -245 388 0 2313 2314 1899 2159 2106 2044 1125 2340 1102 1976 1109 324 1984 336 341 0
2341 d 1899 2159 2044 1125 2340 1102 1976 1109 324 1984 336 341 0
2342 170 171 247 394 -471 0 356 2084 1127 363 1131 1398 0
2342 d 363 1131 1398 0
2343 247 279 394 -471 0 2042 2051 2342 366 358 1399 2043 1133 1130 0
2343 d 2342 1130 0
2345 -180 179 404 503 -505 0 392 2270 2040 0
2345 d 392 2270 2040 0
2346 281 180 406 409 503 -505 0 2336 1972 390 404 409 2080 1163 2243 1513 1510 1406 397 2039 0
2346 d 1972 390 404 409 2080 2243 2039 0
2347 414 -281 409 -474 0 743 744 403 407 1192 2155 1188 423 1181 417 1184 1411 0
2347 d 1192 2155 1188 423 1181 417 1184 1411 0
2349 -475 279 0 2277 746 747 2077 411 424 422 2158 2110 0
2349 d 2110 0
2350 403 279 503 0 2349 1153 2324 1160 2336 2156 2345 1165 2111 2346 1189 743 744 394 403 397 407 1406 1510 1513 2347 1194 1186 1179 1931 1182 415 1410 0
2351 503 -171 279 0 2349 2324 1506 2350 1155 1156 2248 391 1149 1144 1404 1927 382 389 0
2351 d 1506 2350 1156 2248 1149 1144 1404 1927 382 389 0
2352 -247 169 279 502 0 1925 1969 2351 1507 2269 2234 375 1141 370 1137 358 2043 0
2352 d 1141 370 1137 358 2043 0
2353 279 169 502 0 1969 2349 2324 1153 2352 2083 2042 1155 371 1160 375 2156 2234 1507 2336 2345 1165 2111 2346 1189 743 744 394 403 397 407 1406 1510 1513 2347 1194 1186 1179 1931 1182 415 1410 0
2353 d 2349 1153 2352 0
2354 179 -279 -472 0 2082 2269 737 738 1154 1968 369 1147 373 1150 383 1403 0
2354 d 737 738 1154 369 1147 373 1150 383 1403 0
2355 -472 -279 0 2269 2354 388 1968 387 381 1145 1402 1148 0
2355 d 2354 387 381 1145 1402 1148 0
2356 -505 -179 503 0 2336 388 391 1173 1968 1902 2156 1168 1163 1169 1177 2247 0
2356 d 2156 1168 1163 2247 0
2357 -179 503 0 388 391 2356 2324 2277 746 747 2077 1904 411 424 1186 422 2158 1194 2073 1189 1169 1177 1968 1173 1902 1161 0
2357 d 388 391 2356 1968 1173 1902 1161 0
2358 505 403 408 0 2324 2277 746 747 2077 1904 1186 411 424 422 2158 1194 1189 2111 2073 1177 1169 0
2358 d 2324 2277 746 747 2077 1904 411 424 422 2158 2073 1177 1169 0
2359 502 169 0 2353 2355 1901 1507 2083 2357 1155 2082 1160 1172 2358 2345 2336 2111 1165 1189 2346 743 744 394 403 397 407 1406 1510 1513 2347 1194 1186 1179 1931 1182 415 1410 0
2359 d 2353 0
2360 394 384 501 0 2051 1499 2359 1504 2297 2343 2251 1925 356 365 2084 1399 1129 1133 0
2361 -472 278 -394 0 2355 2269 2234 1969 1142 375 1140 1129 2042 371 0
2361 d 2355 2269 2234 1969 1142 375 1140 371 0
2362 406 180 409 503 -505 0 2346 1189 743 744 394 403 397 407 1406 1510 1513 2347 1194 1186 1179 1931 1182 415 1410 0
2362 d 2346 1189 743 744 394 403 397 407 1406 1510 1513 2347 1194 1186 1179 1931 1182 415 1410 0
2363 403 503 0 2357 2082 1160 1172 2358 2336 2345 1165 2111 2362 0
2363 d 2357 2082 1160 1172 2358 2336 2345 1165 2111 2362 0
2364 503 0 2363 1155 2083 1925 1901 2351 0
2364 d 2363 1155 2083 1901 2351 0
2367 388 -245 0 2364 2341 2313 2339 2314 2267 1501 2360 2106 354 357 1965 1900 2361 1507 1504 2251 2297 1127 2042 1134 366 1399 0
2367 d 2341 2313 2360 0
2368 -394 -471 500 -502 0 2297 2106 354 1965 1900 357 2251 1127 2042 1134 1399 366 0
2368 d 2106 357 1127 2042 1134 366 0
2369 -502 -245 0 2367 1104 2309 2308 2312 1501 1504 2368 2297 2343 2251 1925 356 365 2084 1399 1129 1133 0
2369 d 2367 1104 2308 2312 0
2370 502 0 2364 2359 1507 354 2051 1965 2361 1900 0
2370 d 2359 1507 354 2051 1965 2361 1900 0
2379 501 -394 0 2370 1504 1499 2368 0
2379 d 1499 2368 0
2380 -394 0 2370 2369 2310 2379 1501 2309 2339 2314 2267 0
2380 d 2379 0
2384 -500 0 2309 2339 2314 2267 0
2384 d 2309 2339 2314 2267 0
2424 0 2370 2369 2310 2380 2384 1501 1504 2297 2343 2251 1925 356 365 2084 1399 1129 1133 0
".

Theorem golden_certificate_accepted :
  lrat_verdict carry_save (cnf_text carry_save) golden_certificate = accept.
Proof. vm_compute. reflexivity. Qed.

Theorem carry_save_decomposition :
  forall x y : Word 32,
    word_add 32 x y = word_add 32 (word_xor 32 x y) (word_shl 32 (word_and 32 x y) 1).
Proof.
  apply meaning_of_carry_save.
  exact (accepted_certificates_hold carry_save _ _ golden_certificate_accepted).
Qed.
