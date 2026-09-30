(** D-006 v0.3, candidate C-01 (Rocq): DS-02, the Sieve language.

    Sieve has 8-bit words (T-03 at width 8, from [Core]), booleans,
    variables and fixed-size word arrays named by their position, bounded
    loops, public branches and one release construct, [declassify]. Every
    shared S- symbol maps to one declaration here (see adapter.d/ds02.json).
    Expression, command and observation constructors carry a one-letter
    prefix ([E], [C], [O]) so that none of them shadows a standard name such
    as [eq]; the other constructors keep their shared names.

    The semantics follows semantics.md, section 3: [eval] and [step] are
    structural, [run] is structural in its fuel (a [nat], converted from the
    shared [N]), and everything computes under [vm_compute], the declared
    observation method. D2-TH01, D2-TH02 and D2-TH03 are proved by
    induction on the command, from two facts about expressions: a typed
    expression evaluates to a value of its type or fails ([eval_typed]), and
    in low-equivalent states it makes the same observations, with the same
    value when it is public ([eval_agree_typed]). D2-TH04 is an induction
    on the fuel from D2-TH03 and D2-TH02. The proofs use no axioms and no
    evaluator beyond the kernel's own conversion. *)

From D006 Require Import Core.
From Stdlib Require Import NArith Bool List.
Import ListNotations.

(** * Syntax (S-T01 to S-T12) *)

(** S-T01 *)
Inductive Label : Type := public | secret.

(** S-T02 *)
Inductive Ty : Type := ty_word | ty_bool.

(** S-T03 *)
Inductive Val : Type :=
| vword (w : Word 8)
| vbool (b : bool).

(** S-T04: [Evar x] is the [x]-th variable, [Eget a i] reads array [a]. *)
Inductive Expr : Type :=
| Elit (v : Val)
| Evar (x : N)
| Eadd (e1 e2 : Expr)
| Exor (e1 e2 : Expr)
| Eband (e1 e2 : Expr)
| Eeq (e1 e2 : Expr)
| Eget (a : N) (i : Expr).

(** S-T05 *)
Inductive Cmd : Type :=
| Cskip
| Cassign (x : N) (e : Expr)
| Cset (a : N) (i e : Expr)
| Cseq (c1 c2 : Cmd)
| Ccond (e : Expr) (c1 c2 : Cmd)
| Cloop (n : N) (c : Cmd)
| Cdeclassify (x : N) (e : Expr).

(** S-T06: what one step reveals. *)
Inductive Obs : Type :=
| Oread (a n : N)
| Owrite (a n : N)
| Obranch (b : bool)
| Orelease (v : Val)
| Ooob (a n : N).

(** S-T07 to S-T10 *)
Record VarDecl : Type := vdecl { vty : Ty ; vlabel : Label }.
Record ArrDecl : Type := adecl { asize : N ; alabel : Label }.
Record Env : Type := env { vars : list VarDecl ; arrs : list ArrDecl }.
Record State : Type := state { values : list Val ; arrays : list (list (Word 8)) }.

(** S-T11 *)
Inductive StepResult : Type :=
| done
| next (c : Cmd) (s : State) (t : list Obs)
| fail (t : list Obs)
| stuck.

(** S-T12 *)
Inductive Outcome : Type :=
| halted (s : State)
| failed
| out_of_fuel (c : Cmd) (s : State)
| wedged.

(** * Positions

    A variable or array name is a binary natural (T-02 is [N]); it is looked
    up, and updated, by its position in a list. [put] leaves a list without
    that position unchanged. *)

Definition lookup {A : Type} (l : list A) (i : N) : option A := nth_error l (N.to_nat i).

Fixpoint update {A : Type} (l : list A) (n : nat) (a : A) : list A :=
  match l, n with
  | [], _ => []
  | _ :: l', O => a :: l'
  | b :: l', S n' => b :: update l' n' a
  end.

Definition put {A : Type} (l : list A) (i : N) (a : A) : list A := update l (N.to_nat i) a.

Arguments lookup {A} l i : simpl never.
Arguments put {A} l i a : simpl never.

(** * Typing (S-F01) *)

(** The join of two labels. *)
Definition join (l1 l2 : Label) : Label :=
  match l1 with public => l2 | secret => secret end.

(** Data labelled [l1] may be stored under label [l2]. *)
Definition flows (l1 l2 : Label) : bool :=
  match l1, l2 with secret, public => false | _, _ => true end.

Definition is_public (l : Label) : bool :=
  match l with public => true | secret => false end.

Definition ty_eqb (t1 t2 : Ty) : bool :=
  match t1, t2 with ty_word, ty_word | ty_bool, ty_bool => true | _, _ => false end.

Definition ty_of (v : Val) : Ty :=
  match v with vword _ => ty_word | vbool _ => ty_bool end.

(** [add], [xor], [band] and [eq] take two words; the label is the join. *)
Definition type_binop (res : Ty) (a b : option (Ty * Label)) : option (Ty * Label) :=
  match a, b with
  | Some (ty_word, l1), Some (ty_word, l2) => Some (res, join l1 l2)
  | _, _ => None
  end.

(** The type and label of an expression, or [None]. *)
Fixpoint type_expr (g : Env) (e : Expr) : option (Ty * Label) :=
  match e with
  | Elit v => Some (ty_of v, public)
  | Evar x =>
      match lookup (vars g) x with
      | Some d => Some (vty d, vlabel d)
      | None => None
      end
  | Eadd e1 e2 | Exor e1 e2 | Eband e1 e2 =>
      type_binop ty_word (type_expr g e1) (type_expr g e2)
  | Eeq e1 e2 => type_binop ty_bool (type_expr g e1) (type_expr g e2)
  | Eget a i =>
      match lookup (arrs g) a, type_expr g i with
      | Some d, Some (ty_word, public) => Some (ty_word, alabel d)
      | _, _ => None
      end
  end.

Fixpoint well_typed (g : Env) (c : Cmd) : bool :=
  match c with
  | Cskip => true
  | Cassign x e =>
      match lookup (vars g) x, type_expr g e with
      | Some d, Some (t, l) => ty_eqb t (vty d) && flows l (vlabel d)
      | _, _ => false
      end
  | Cset a i e =>
      match lookup (arrs g) a, type_expr g i, type_expr g e with
      | Some d, Some (ty_word, public), Some (ty_word, l) => flows l (alabel d)
      | _, _, _ => false
      end
  | Cseq c1 c2 => well_typed g c1 && well_typed g c2
  | Ccond e c1 c2 =>
      match type_expr g e with
      | Some (ty_bool, public) => well_typed g c1 && well_typed g c2
      | _ => false
      end
  | Cloop _ c1 => well_typed g c1
  | Cdeclassify x e =>
      match lookup (vars g) x, type_expr g e with
      | Some d, Some (t, _) => ty_eqb t (vty d) && is_public (vlabel d)
      | _, _ => false
      end
  end.

(** * Evaluation (S-F04, S-F05, S-F06) *)

(** An expression gives a value and its observations, a failure after some
    observations, or stuck. *)
Inductive EvalResult : Type :=
| eok (v : Val) (t : list Obs)
| efail (t : list Obs)
| estuck.

(** A word operator: operands left to right, the first failure or stuck
    ends evaluation, and operands that are not words are stuck. *)
Definition eval_binop (f : Word 8 -> Word 8 -> Val) (r1 r2 : EvalResult) : EvalResult :=
  match r1 with
  | eok v1 t1 =>
      match r2 with
      | eok v2 t2 =>
          match v1, v2 with
          | vword x, vword y => eok (f x y) (t1 ++ t2)
          | _, _ => estuck
          end
      | efail t2 => efail (t1 ++ t2)
      | estuck => estuck
      end
  | efail t1 => efail t1
  | estuck => estuck
  end.

Fixpoint eval (s : State) (e : Expr) : EvalResult :=
  match e with
  | Elit v => eok v []
  | Evar x =>
      match lookup (values s) x with
      | Some v => eok v []
      | None => estuck
      end
  | Eadd e1 e2 => eval_binop (fun x y => vword (word_add 8 x y)) (eval s e1) (eval s e2)
  | Exor e1 e2 => eval_binop (fun x y => vword (word_xor 8 x y)) (eval s e1) (eval s e2)
  | Eband e1 e2 => eval_binop (fun x y => vword (word_and 8 x y)) (eval s e1) (eval s e2)
  | Eeq e1 e2 => eval_binop (fun x y => vbool (word_eqb x y)) (eval s e1) (eval s e2)
  | Eget a i =>
      match eval s i with
      | eok (vword n) t =>
          match lookup (arrays s) a with
          | Some cells =>
              match lookup cells (val n) with
              | Some w => eok (vword w) (t ++ [Oread a (val n)])
              | None => efail (t ++ [Ooob a (val n)])
              end
          | None => estuck
          end
      | eok (vbool _) _ => estuck
      | efail t => efail t
      | estuck => estuck
      end
  end.

(** Variable [x] of [s] set to [v]. *)
Definition assign_var (s : State) (x : N) (v : Val) : State :=
  state (put (values s) x v) (arrays s).

(** Array [a] of [s], whose cells are [cells], with cell [n] set to [w]. *)
Definition store (s : State) (a : N) (cells : list (Word 8)) (n : N) (w : Word 8) : State :=
  state (values s) (put (arrays s) a (put cells n w)).

(** A step of [c1] seen from [seq c1 c2]. *)
Definition wrap (r : StepResult) (c2 : Cmd) : StepResult :=
  match r with
  | next c1' s' t => next (Cseq c1' c2) s' t
  | fail t => fail t
  | done | stuck => stuck
  end.

Fixpoint step (s : State) (c : Cmd) : StepResult :=
  match c with
  | Cskip => done
  | Cassign x e =>
      match eval s e with
      | eok v t =>
          match lookup (values s) x with
          | Some _ => next Cskip (assign_var s x v) t
          | None => stuck
          end
      | efail t => fail t
      | estuck => stuck
      end
  | Cset a i e =>
      match eval s i with
      | eok (vword n) ti =>
          match eval s e with
          | eok (vword w) te =>
              match lookup (arrays s) a with
              | Some cells =>
                  match lookup cells (val n) with
                  | Some _ => next Cskip (store s a cells (val n) w) ((ti ++ te) ++ [Owrite a (val n)])
                  | None => fail ((ti ++ te) ++ [Ooob a (val n)])
                  end
              | None => stuck
              end
          | eok (vbool _) _ => stuck
          | efail te => fail (ti ++ te)
          | estuck => stuck
          end
      | eok (vbool _) _ => stuck
      | efail ti => fail ti
      | estuck => stuck
      end
  | Cseq c1 c2 =>
      match c1 with
      | Cskip => next c2 s []
      | _ => wrap (step s c1) c2
      end
  | Ccond e c1 c2 =>
      match eval s e with
      | eok (vbool b) t => next (if b then c1 else c2) s (t ++ [Obranch b])
      | eok (vword _) _ => stuck
      | efail t => fail t
      | estuck => stuck
      end
  | Cloop n c1 =>
      match n with
      | N0 => next Cskip s []
      | Npos _ => next (Cseq c1 (Cloop (N.pred n) c1)) s []
      end
  | Cdeclassify x e =>
      match eval s e with
      | eok v t =>
          match lookup (values s) x with
          | Some _ => next Cskip (assign_var s x v) (t ++ [Orelease v])
          | None => stuck
          end
      | efail t => fail t
      | estuck => stuck
      end
  end.

(** At most [k] steps: the outcome and every observation made. *)
Fixpoint run (k : nat) (s : State) (c : Cmd) : Outcome * list Obs :=
  match k with
  | O => (out_of_fuel c s, [])
  | S k' =>
      match step s c with
      | done => (halted s, [])
      | next c' s' t => let r := run k' s' c' in (fst r, t ++ snd r)
      | fail t => (failed, t)
      | stuck => (wedged, [])
      end
  end.

Definition run_trace (n : N) (s : State) (c : Cmd) : list Obs := snd (run (N.to_nat n) s c).
Definition run_outcome (n : N) (s : State) (c : Cmd) : Outcome := fst (run (N.to_nat n) s c).

(** * Well-formedness and public equivalence (S-F02, S-F03, S-F07) *)

(** [s] has one value of the declared type per declared variable and one
    array of the declared size per declared array. *)
Definition wf (g : Env) (s : State) : Prop :=
  Forall2 (fun d v => ty_of v = vty d) (vars g) (values s) /\
  Forall2 (fun d cells => N.of_nat (length cells) = asize d) (arrs g) (arrays s).

(** [s1] and [s2] agree on every declared public variable and array. *)
Definition low_eq (g : Env) (s1 s2 : State) : Prop :=
  (forall x d, lookup (vars g) x = Some d -> vlabel d = public ->
               lookup (values s1) x = lookup (values s2) x) /\
  (forall a d, lookup (arrs g) a = Some d -> alabel d = public ->
               lookup (arrays s1) a = lookup (arrays s2) a).

Definition outcome_low_eq (g : Env) (o1 o2 : Outcome) : Prop :=
  match o1, o2 with
  | halted s1, halted s2 => low_eq g s1 s2
  | failed, failed => True
  | out_of_fuel c1 s1, out_of_fuel c2 s2 => c1 = c2 /\ low_eq g s1 s2
  | wedged, wedged => True
  | _, _ => False
  end.

(** * The fixtures of D2-TH05 and D2-TH06 *)

(** Gamma+: key (secret word), nonce (public word), acc (secret word), ok
    (public bool); table (4 public words) and state (4 secret words). *)
Definition gamma_plus : Env :=
  env [vdecl ty_word secret; vdecl ty_word public; vdecl ty_word secret; vdecl ty_bool public]
      [adecl 4 public; adecl 4 secret].

Definition wlit (n : N) : Expr := Elit (vword (word_of_nat 8 n)).

(** The positive program P+. *)
Definition p_plus : Cmd :=
  Cseq (Cassign 2 (Exor (Evar 0) (Evar 1)))
  (Cseq (Cset 1 (Eband (Evar 1) (wlit 3)) (Eadd (Evar 2) (Eget 0 (Eband (Evar 1) (wlit 3)))))
  (Cseq (Cloop 2 (Cassign 2 (Eadd (Evar 2) (Evar 0))))
  (Cseq (Ccond (Eeq (Evar 1) (wlit 7)) (Cassign 2 (Exor (Evar 2) (Evar 1))) Cskip)
        (Cdeclassify 3 (Eeq (Evar 2) (Evar 0)))))).

(** The negative program P-: a branch on the secret key. *)
Definition p_minus : Cmd :=
  Ccond (Eeq (Evar 0) (wlit 0)) (Cassign 2 (wlit 1)) (Cassign 2 (wlit 2)).

Definition table : list (Word 8) :=
  [word_of_nat 8 16; word_of_nat 8 32; word_of_nat 8 48; word_of_nat 8 64].

Definition zeros : list (Word 8) :=
  [word_of_nat 8 0; word_of_nat 8 0; word_of_nat 8 0; word_of_nat 8 0].

(** Two states that differ only in the secret key. *)
Definition sigma_a : State :=
  state [vword (word_of_nat 8 0); vword (word_of_nat 8 7); vword (word_of_nat 8 0); vbool false]
        [table; zeros].

Definition sigma_b : State :=
  state [vword (word_of_nat 8 1); vword (word_of_nat 8 7); vword (word_of_nat 8 0); vbool false]
        [table; zeros].

(** * Lemmas on positions *)

Lemma update_length {A : Type} (l : list A) (n : nat) (a : A) :
  length (update l n a) = length l.
Proof. revert n; induction l as [|b l IH]; intros [|n]; simpl; auto. Qed.

Lemma nth_error_update_same {A : Type} (l : list A) (n : nat) (a b : A) :
  nth_error l n = Some b -> nth_error (update l n a) n = Some a.
Proof.
  revert n; induction l as [|c l IH]; intros [|n] H; simpl in *; try discriminate; auto.
Qed.

Lemma nth_error_update_other {A : Type} (l : list A) (n m : nat) (a : A) :
  n <> m -> nth_error (update l n a) m = nth_error l m.
Proof.
  revert n m; induction l as [|c l IH]; intros [|n] [|m] H; simpl; auto; congruence.
Qed.

Lemma put_length {A : Type} (l : list A) (i : N) (a : A) : length (put l i a) = length l.
Proof. apply update_length. Qed.

Lemma lookup_put_same {A : Type} (l : list A) (i : N) (a b : A) :
  lookup l i = Some b -> lookup (put l i a) i = Some a.
Proof. apply nth_error_update_same. Qed.

Lemma lookup_put_other {A : Type} (l : list A) (i j : N) (a : A) :
  i <> j -> lookup (put l i a) j = lookup l j.
Proof. intros H. apply nth_error_update_other. intros E. apply H, N2Nat.inj, E. Qed.

(** Two lists of the same length have the same positions. *)
Lemma lookup_none {A B : Type} (l1 : list A) (l2 : list B) (i : N) :
  length l1 = length l2 -> lookup l1 i = None -> lookup l2 i = None.
Proof.
  unfold lookup. intros E H. apply nth_error_None. rewrite <- E. apply nth_error_None, H.
Qed.

Section Forall2.
  Context {A B : Type} (R : A -> B -> Prop).

  Lemma Forall2_lookup_l (l1 : list A) (l2 : list B) (i : N) (a : A) :
    Forall2 R l1 l2 -> lookup l1 i = Some a -> exists b, lookup l2 i = Some b /\ R a b.
  Proof.
    unfold lookup. generalize (N.to_nat i) as n. intros n H. revert n.
    induction H as [|x y l1 l2 Hxy H IH]; intros [|n] E; simpl in *; try discriminate.
    - injection E as <-. exists y; auto.
    - apply IH, E.
  Qed.

  Lemma Forall2_lookup (l1 : list A) (l2 : list B) (i : N) (a : A) (b : B) :
    Forall2 R l1 l2 -> lookup l1 i = Some a -> lookup l2 i = Some b -> R a b.
  Proof.
    intros H E1 E2. destruct (Forall2_lookup_l l1 l2 i a H E1) as (b' & E & Hr).
    rewrite E2 in E. injection E as ->. exact Hr.
  Qed.

  Lemma Forall2_put_r (l1 : list A) (l2 : list B) (i : N) (a : A) (b : B) :
    Forall2 R l1 l2 -> lookup l1 i = Some a -> R a b -> Forall2 R l1 (put l2 i b).
  Proof.
    unfold lookup, put. generalize (N.to_nat i) as n. intros n H. revert n.
    induction H as [|x y l1 l2 Hxy H IH]; intros [|n] E Hr; simpl in *; try discriminate.
    - injection E as <-. constructor; assumption.
    - constructor; [assumption | apply (IH n E Hr)].
  Qed.
End Forall2.

(** * Well-typed expressions and well-formed states *)

Lemma join_public (l1 l2 : Label) : join l1 l2 = public -> l1 = public /\ l2 = public.
Proof. destruct l1, l2; simpl; intros H; try discriminate; auto. Qed.

Lemma flows_public (l : Label) : flows l public = true -> l = public.
Proof. destruct l; simpl; congruence. Qed.

Lemma ty_eqb_eq (t1 t2 : Ty) : ty_eqb t1 t2 = true -> t1 = t2.
Proof. destruct t1, t2; simpl; congruence. Qed.

Lemma type_binop_inv (res t : Ty) (l : Label) (a b : option (Ty * Label)) :
  type_binop res a b = Some (t, l) ->
  exists l1 l2, a = Some (ty_word, l1) /\ b = Some (ty_word, l2) /\ t = res /\ l = join l1 l2.
Proof.
  destruct a as [[[|] l1]|], b as [[[|] l2]|]; simpl; intros H; try discriminate.
  injection H as <- <-. exists l1, l2. auto.
Qed.

(** A variable declared in a well-formed state has a value of its type. *)
Lemma wf_var (g : Env) (s : State) (x : N) (d : VarDecl) :
  wf g s -> lookup (vars g) x = Some d -> exists v, lookup (values s) x = Some v /\ ty_of v = vty d.
Proof. intros [H _] E. exact (Forall2_lookup_l _ _ _ _ _ H E). Qed.

(** A declared array of a well-formed state has its declared size. *)
Lemma wf_array (g : Env) (s : State) (a : N) (d : ArrDecl) :
  wf g s -> lookup (arrs g) a = Some d ->
  exists cells, lookup (arrays s) a = Some cells /\ N.of_nat (length cells) = asize d.
Proof. intros [_ H] E. exact (Forall2_lookup_l _ _ _ _ _ H E). Qed.

Lemma wf_assign (g : Env) (s : State) (x : N) (d : VarDecl) (v : Val) :
  wf g s -> lookup (vars g) x = Some d -> ty_of v = vty d -> wf g (assign_var s x v).
Proof. intros [Hv Ha] E T. split; [eapply Forall2_put_r; eauto | exact Ha]. Qed.

Lemma wf_store (g : Env) (s : State) (a : N) (d : ArrDecl) (cells : list (Word 8)) (n : N) (w : Word 8) :
  wf g s -> lookup (arrs g) a = Some d -> lookup (arrays s) a = Some cells -> wf g (store s a cells n w).
Proof.
  intros [Hv Ha] Ed Ec. split; [exact Hv|].
  eapply Forall2_put_r; [exact Ha | exact Ed |].
  rewrite put_length. exact (Forall2_lookup _ _ _ _ _ _ Ha Ed Ec).
Qed.

(** An expression of type [t] gives a value of type [t] or fails; it is
    never stuck. *)
Definition eval_good (t : Ty) (r : EvalResult) : Prop :=
  match r with
  | eok v _ => ty_of v = t
  | efail _ => True
  | estuck => False
  end.

Lemma eval_binop_good (f : Word 8 -> Word 8 -> Val) (res : Ty) (r1 r2 : EvalResult) :
  (forall x y, ty_of (f x y) = res) ->
  eval_good ty_word r1 -> eval_good ty_word r2 -> eval_good res (eval_binop f r1 r2).
Proof.
  intros Hf G1 G2.
  destruct r1 as [[x|b] t1|t1|]; simpl in G1 |- *; try discriminate; try contradiction; [|exact I].
  destruct r2 as [[y|c] t2|t2|]; simpl in G2 |- *; try discriminate; try contradiction; [|exact I].
  apply Hf.
Qed.

Lemma eval_typed (g : Env) (s : State) (e : Expr) (t : Ty) (l : Label) :
  wf g s -> type_expr g e = Some (t, l) -> eval_good t (eval s e).
Proof.
  intros W. revert t l.
  induction e as [v|x|e1 IH1 e2 IH2|e1 IH1 e2 IH2|e1 IH1 e2 IH2|e1 IH1 e2 IH2|a i IH];
    intros t l He; cbn [type_expr] in He; cbn [eval].
  - injection He as <- _. reflexivity.
  - destruct (lookup (vars g) x) as [d|] eqn:Ed; [|discriminate].
    injection He as <- _. destruct (wf_var g s x d W Ed) as (v & -> & T). exact T.
  - apply type_binop_inv in He as (l1 & l2 & E1 & E2 & -> & _).
    apply eval_binop_good; [reflexivity | exact (IH1 _ _ E1) | exact (IH2 _ _ E2)].
  - apply type_binop_inv in He as (l1 & l2 & E1 & E2 & -> & _).
    apply eval_binop_good; [reflexivity | exact (IH1 _ _ E1) | exact (IH2 _ _ E2)].
  - apply type_binop_inv in He as (l1 & l2 & E1 & E2 & -> & _).
    apply eval_binop_good; [reflexivity | exact (IH1 _ _ E1) | exact (IH2 _ _ E2)].
  - apply type_binop_inv in He as (l1 & l2 & E1 & E2 & -> & _).
    apply eval_binop_good; [reflexivity | exact (IH1 _ _ E1) | exact (IH2 _ _ E2)].
  - destruct (lookup (arrs g) a) as [d|] eqn:Ed; [|discriminate].
    destruct (type_expr g i) as [[[|] [|]]|] eqn:Ei; try discriminate.
    injection He as <- _. specialize (IH _ _ eq_refl).
    destruct (eval s i) as [[n|b] ti|ti|]; simpl in IH; try discriminate; try contradiction; [|exact I].
    destruct (wf_array g s a d W Ed) as (cells & -> & _).
    destruct (lookup cells (val n)); [reflexivity | exact I].
Qed.

(** [seq c1 c2] with [c1] not [skip] steps [c1]. *)
Lemma step_seq (s : State) (c1 c2 : Cmd) : c1 <> Cskip -> step s (Cseq c1 c2) = wrap (step s c1) c2.
Proof. destruct c1; intros H; [congruence | reflexivity ..]. Qed.

Lemma skip_or_not (c : Cmd) : c = Cskip \/ c <> Cskip.
Proof. destruct c; [left; reflexivity | right; discriminate ..]. Qed.

(** Only [skip] is done. *)
Lemma step_done (s : State) (c : Cmd) : step s c = done -> c = Cskip.
Proof.
  destruct c as [|x e|a i e|c1 c2|e c1 c2|n c1|x e]; cbn [step]; intros H; try reflexivity; exfalso;
    unfold wrap in H;
    repeat match type of H with context [match ?x with _ => _ end] => destruct x end;
    discriminate.
Qed.

(** * Progress and preservation (D2-TH01, D2-TH02) *)

(** D2-TH01. A well-typed command in a well-formed state is never stuck. *)
Theorem progress :
  forall (g : Env) (c : Cmd) (s : State),
    well_typed g c = true -> wf g s -> step s c <> stuck.
Proof.
  intros g c s Hc W.
  induction c as [|x e|a i e|c1 IH1 c2 IH2|e c1 IH1 c2 IH2|n c IH|x e];
    cbn [well_typed] in Hc.
  - discriminate.
  - (* assign *)
    destruct (lookup (vars g) x) as [d|] eqn:Ed; [|discriminate].
    destruct (type_expr g e) as [[t l]|] eqn:Ee; [|discriminate].
    pose proof (eval_typed g s e t l W Ee) as G. cbn [step].
    destruct (eval s e) as [v tr|tr|]; simpl in G; [|discriminate|contradiction].
    destruct (wf_var g s x d W Ed) as (u & -> & _). discriminate.
  - (* set *)
    destruct (lookup (arrs g) a) as [d|] eqn:Ed; [|discriminate].
    destruct (type_expr g i) as [[[|] [|]]|] eqn:Ei; try discriminate.
    destruct (type_expr g e) as [[[|] l]|] eqn:Ee; try discriminate.
    pose proof (eval_typed g s i _ _ W Ei) as Gi. pose proof (eval_typed g s e _ _ W Ee) as Ge.
    cbn [step].
    destruct (eval s i) as [[n|b] ti|ti|]; simpl in Gi; try discriminate; try contradiction.
    destruct (eval s e) as [[w|b] te|te|]; simpl in Ge; try discriminate; try contradiction.
    destruct (wf_array g s a d W Ed) as (cells & -> & _).
    destruct (lookup cells (val n)); discriminate.
  - (* seq: the first command is skip, or it steps *)
    apply andb_prop in Hc as [H1 _].
    destruct (skip_or_not c1) as [->|Hne]; [discriminate|].
    rewrite step_seq by exact Hne. specialize (IH1 H1).
    pose proof (step_done s c1) as D.
    destruct (step s c1); cbn [wrap]; try discriminate.
    + exfalso. exact (Hne (D eq_refl)).
    + exact IH1.
  - (* cond: a condition of type bool, whatever its label *)
    destruct (type_expr g e) as [[[|] l]|] eqn:Ee; try discriminate.
    pose proof (eval_typed g s e _ _ W Ee) as G. cbn [step].
    destruct (eval s e) as [[w|b] t|t|]; simpl in G; try discriminate; contradiction.
  - (* loop *)
    cbn [step]. destruct n; discriminate.
  - (* declassify *)
    destruct (lookup (vars g) x) as [d|] eqn:Ed; [|discriminate].
    destruct (type_expr g e) as [[t l]|] eqn:Ee; [|discriminate].
    pose proof (eval_typed g s e t l W Ee) as G. cbn [step].
    destruct (eval s e) as [v tr|tr|]; simpl in G; [|discriminate|contradiction].
    destruct (wf_var g s x d W Ed) as (u & -> & _). discriminate.
Qed.

(** D2-TH02. A step of a well-typed command from a well-formed state leads to
    a well-typed command and a well-formed state. *)
Theorem preservation :
  forall (g : Env) (c : Cmd) (s : State) (c' : Cmd) (s' : State) (t : list Obs),
    well_typed g c = true -> wf g s -> step s c = next c' s' t ->
    well_typed g c' = true /\ wf g s'.
Proof.
  intros g c s c' s' t Hc W. revert c' s' t.
  induction c as [|x e|a i e|c1 IH1 c2 IH2|e c1 IH1 c2 IH2|n c IH|x e];
    intros c' s' t E; cbn [well_typed] in Hc.
  - discriminate.
  - (* assign: the value has the variable's type *)
    destruct (lookup (vars g) x) as [d|] eqn:Ed; [|discriminate].
    destruct (type_expr g e) as [[ty l]|] eqn:Ee; [|discriminate].
    apply andb_prop in Hc as [Ht _]. apply ty_eqb_eq in Ht as ->.
    pose proof (eval_typed g s e _ l W Ee) as G. cbn [step] in E.
    destruct (eval s e) as [v tr|tr|]; simpl in G; try discriminate.
    destruct (lookup (values s) x); [|discriminate].
    injection E as <- <- _. split; [reflexivity | exact (wf_assign g s x d v W Ed G)].
  - (* set: the array keeps its size *)
    destruct (lookup (arrs g) a) as [d|] eqn:Ed; [|discriminate].
    destruct (type_expr g i) as [[[|] [|]]|] eqn:Ei; try discriminate.
    destruct (type_expr g e) as [[[|] l]|] eqn:Ee; try discriminate.
    cbn [step] in E.
    destruct (eval s i) as [[n|b] ti|ti|]; try discriminate.
    destruct (eval s e) as [[w|b] te|te|]; try discriminate.
    destruct (lookup (arrays s) a) as [cells|] eqn:Ec; [|discriminate].
    destruct (lookup cells (val n)); [|discriminate].
    injection E as <- <- _. split; [reflexivity | exact (wf_store g s a d cells (val n) w W Ed Ec)].
  - (* seq *)
    apply andb_prop in Hc as [H1 H2].
    destruct (skip_or_not c1) as [->|Hne].
    + cbn [step] in E. injection E as <- <- _. split; assumption.
    + rewrite step_seq in E by exact Hne.
      destruct (step s c1) as [|c1' s1 t1|t1|] eqn:E1; cbn [wrap] in E; try discriminate.
      injection E as <- <- _. destruct (IH1 H1 c1' s1 t1 eq_refl) as [H1' W'].
      cbn [well_typed]. rewrite H1', H2. split; [reflexivity | exact W'].
  - (* cond: both branches are well typed, whatever the condition's label *)
    destruct (type_expr g e) as [[[|] l]|] eqn:Ee; try discriminate.
    assert (H12 : well_typed g c1 = true /\ well_typed g c2 = true).
    { simpl in Hc. destruct l; try discriminate; apply andb_prop; exact Hc. }
    cbn [step] in E.
    destruct (eval s e) as [[w|b] tr|tr|]; try discriminate.
    injection E as <- <- _. split; [destruct b; apply H12 | exact W].
  - (* loop *)
    cbn [step] in E. destruct n; injection E as <- <- _; (split; [|exact W]); cbn [well_typed].
    + reflexivity.
    + rewrite Hc. reflexivity.
  - (* declassify: as assign *)
    destruct (lookup (vars g) x) as [d|] eqn:Ed; [|discriminate].
    destruct (type_expr g e) as [[ty l]|] eqn:Ee; [|discriminate].
    apply andb_prop in Hc as [Ht _]. apply ty_eqb_eq in Ht as ->.
    pose proof (eval_typed g s e _ l W Ee) as G. cbn [step] in E.
    destruct (eval s e) as [v tr|tr|]; simpl in G; try discriminate.
    destruct (lookup (values s) x); [|discriminate].
    injection E as <- <- _. split; [reflexivity | exact (wf_assign g s x d v W Ed G)].
Qed.

(** * Noninterference *)

(** Equality of values is decidable (words are equal when their values are). *)
Lemma val_eq_dec (v1 v2 : Val) : {v1 = v2} + {v1 <> v2}.
Proof.
  destruct v1 as [x|b], v2 as [y|c]; try (right; discriminate).
  - destruct (N.eq_dec (val x) (val y)) as [E|E].
    + left. f_equal. apply word_ext, E.
    + right. intros H. injection H as ->. apply E. reflexivity.
  - destruct (bool_dec b c) as [->|E]; [left; reflexivity | right; congruence].
Qed.

Lemma low_eq_assign (g : Env) (s1 s2 : State) (x : N) (v1 v2 u1 u2 : Val) :
  low_eq g s1 s2 -> lookup (values s1) x = Some u1 -> lookup (values s2) x = Some u2 ->
  (forall d, lookup (vars g) x = Some d -> vlabel d = public -> v1 = v2) ->
  low_eq g (assign_var s1 x v1) (assign_var s2 x v2).
Proof.
  intros [Lv La] E1 E2 P. split; [|exact La]. cbn [assign_var values].
  intros y d Ed Hp. destruct (N.eq_dec x y) as [<-|Hne].
  - rewrite (lookup_put_same _ _ _ _ E1), (lookup_put_same _ _ _ _ E2), (P d Ed Hp). reflexivity.
  - rewrite !lookup_put_other by exact Hne. exact (Lv y d Ed Hp).
Qed.

Lemma low_eq_store (g : Env) (s1 s2 : State) (a : N) (cells1 cells2 : list (Word 8)) (n : N) (w1 w2 : Word 8) :
  low_eq g s1 s2 -> lookup (arrays s1) a = Some cells1 -> lookup (arrays s2) a = Some cells2 ->
  (forall d, lookup (arrs g) a = Some d -> alabel d = public -> w1 = w2) ->
  low_eq g (store s1 a cells1 n w1) (store s2 a cells2 n w2).
Proof.
  intros [Lv La] E1 E2 P. split; [exact Lv|]. cbn [store arrays].
  intros b d Ed Hp. destruct (N.eq_dec a b) as [<-|Hne].
  - rewrite (lookup_put_same _ _ _ _ E1), (lookup_put_same _ _ _ _ E2).
    pose proof (La a d Ed Hp) as C. rewrite E1, E2 in C. injection C as <-.
    rewrite (P d Ed Hp). reflexivity.
  - rewrite !lookup_put_other by exact Hne. exact (La b d Ed Hp).
Qed.

(** Two evaluations that reveal the same: both give values with the same
    observations, and the same value when the label is public, or both fail
    with the same observations. *)
Definition eval_agree (l : Label) (r1 r2 : EvalResult) : Prop :=
  match r1, r2 with
  | eok v1 t1, eok v2 t2 => t1 = t2 /\ (l = public -> v1 = v2)
  | efail t1, efail t2 => t1 = t2
  | _, _ => False
  end.

Lemma eval_binop_agree (f : Word 8 -> Word 8 -> Val) (l1 l2 : Label) (r1 r2 q1 q2 : EvalResult) :
  eval_good ty_word r1 -> eval_good ty_word q1 -> eval_good ty_word r2 -> eval_good ty_word q2 ->
  eval_agree l1 r1 q1 -> eval_agree l2 r2 q2 ->
  eval_agree (join l1 l2) (eval_binop f r1 r2) (eval_binop f q1 q2).
Proof.
  intros G1 G1' G2 G2' A1 A2.
  destruct r1 as [v1 t1|t1|], q1 as [w1 u1|u1|]; simpl in A1; try contradiction; [|exact A1].
  destruct A1 as [<- P1].
  destruct v1 as [x1|], w1 as [y1|]; simpl in G1, G1'; try discriminate.
  destruct r2 as [v2 t2|t2|], q2 as [w2 u2|u2|]; simpl in A2 |- *; try contradiction.
  - destruct A2 as [<- P2].
    destruct v2 as [x2|], w2 as [y2|]; simpl in G2, G2'; try discriminate.
    split; [reflexivity|]. intros Hj. apply join_public in Hj as [-> ->].
    specialize (P1 eq_refl). specialize (P2 eq_refl).
    injection P1 as ->. injection P2 as ->. reflexivity.
  - rewrite A2. reflexivity.
Qed.

Lemma eval_agree_typed (g : Env) (s1 s2 : State) (e : Expr) (t : Ty) (l : Label) :
  wf g s1 -> wf g s2 -> low_eq g s1 s2 -> type_expr g e = Some (t, l) ->
  eval_agree l (eval s1 e) (eval s2 e).
Proof.
  intros W1 W2 L. revert t l.
  induction e as [v|x|e1 IH1 e2 IH2|e1 IH1 e2 IH2|e1 IH1 e2 IH2|e1 IH1 e2 IH2|a i IH];
    intros t l He; cbn [type_expr] in He; cbn [eval].
  - injection He as _ <-. split; reflexivity.
  - destruct (lookup (vars g) x) as [d|] eqn:Ed; [|discriminate].
    injection He as _ <-. pose proof (proj1 L x d Ed) as P.
    destruct (wf_var g s1 x d W1 Ed) as (v1 & E1 & _), (wf_var g s2 x d W2 Ed) as (v2 & E2 & _).
    rewrite E1, E2 in *. split; [reflexivity|]. intros Hp. specialize (P Hp). injection P as ->. reflexivity.
  - apply type_binop_inv in He as (l1 & l2 & E1 & E2 & -> & ->).
    apply eval_binop_agree; eauto using eval_typed.
  - apply type_binop_inv in He as (l1 & l2 & E1 & E2 & -> & ->).
    apply eval_binop_agree; eauto using eval_typed.
  - apply type_binop_inv in He as (l1 & l2 & E1 & E2 & -> & ->).
    apply eval_binop_agree; eauto using eval_typed.
  - apply type_binop_inv in He as (l1 & l2 & E1 & E2 & -> & ->).
    apply eval_binop_agree; eauto using eval_typed.
  - destruct (lookup (arrs g) a) as [d|] eqn:Ed; [|discriminate].
    destruct (type_expr g i) as [[[|] [|]]|] eqn:Ei; try discriminate.
    injection He as _ <-.
    pose proof (IH _ _ eq_refl) as A.
    pose proof (eval_typed g s1 i _ _ W1 Ei) as G1. pose proof (eval_typed g s2 i _ _ W2 Ei) as G2.
    destruct (eval s1 i) as [v1 t1|t1|], (eval s2 i) as [v2 t2|t2|]; simpl in A; try contradiction; [|exact A].
    destruct A as [<- P]. specialize (P eq_refl) as <-.
    destruct v1 as [n|b]; simpl in G1; try discriminate.
    destruct (wf_array g s1 a d W1 Ed) as (cells1 & E1 & S1), (wf_array g s2 a d W2 Ed) as (cells2 & E2 & S2).
    rewrite E1, E2.
    assert (Len : length cells1 = length cells2) by (apply Nat2N.inj; congruence).
    destruct (lookup cells1 (val n)) as [w1|] eqn:C1, (lookup cells2 (val n)) as [w2|] eqn:C2.
    + split; [reflexivity|]. intros Hp.
      pose proof (proj2 L a d Ed Hp) as C. rewrite E1, E2 in C. injection C as <-.
      rewrite C1 in C2. injection C2 as ->. reflexivity.
    + rewrite (lookup_none cells2 cells1 (val n) (eq_sym Len) C2) in C1. discriminate.
    + rewrite (lookup_none cells1 cells2 (val n) Len C1) in C2. discriminate.
    + reflexivity.
Qed.

(** The conclusion of D2-TH03 for two step results. *)
Definition lockstep (g : Env) (r1 r2 : StepResult) : Prop :=
  (r1 = done /\ r2 = done) \/
  (exists t, r1 = fail t /\ r2 = fail t) \/
  (exists c' s1' s2' t, r1 = next c' s1' t /\ r2 = next c' s2' t /\ low_eq g s1' s2') \/
  (exists c' s1' s2' p v1 v2,
      r1 = next c' s1' (p ++ [Orelease v1]) /\ r2 = next c' s2' (p ++ [Orelease v2]) /\ v1 <> v2).

Lemma lockstep_fail (g : Env) (t : list Obs) : lockstep g (fail t) (fail t).
Proof. right; left. exists t. auto. Qed.

Lemma lockstep_next (g : Env) (c : Cmd) (s1 s2 : State) (t : list Obs) :
  low_eq g s1 s2 -> lockstep g (next c s1 t) (next c s2 t).
Proof. intros L. right; right; left. exists c, s1, s2, t. auto. Qed.

Lemma lockstep_release (g : Env) (c : Cmd) (s1 s2 : State) (p : list Obs) (v1 v2 : Val) :
  v1 <> v2 -> lockstep g (next c s1 (p ++ [Orelease v1])) (next c s2 (p ++ [Orelease v2])).
Proof. intros H. right; right; right. exists c, s1, s2, p, v1, v2. auto. Qed.

Lemma lockstep_wrap (g : Env) (r1 r2 : StepResult) (c2 : Cmd) :
  lockstep g r1 r2 -> r1 <> done -> lockstep g (wrap r1 c2) (wrap r2 c2).
Proof.
  intros [[-> ->]|[(t & -> & ->)|[(c' & s1' & s2' & t & -> & -> & L)|(c' & s1' & s2' & p & v1 & v2 & -> & -> & H)]]] D;
    simpl.
  - contradiction.
  - apply lockstep_fail.
  - apply lockstep_next, L.
  - apply lockstep_release, H.
Qed.

(** D2-TH03. From two low-equivalent well-formed states, a well-typed command
    takes one step in lock step: both done, both failing with the same
    observations, both stepping to the same command with the same
    observations and low-equivalent states, or both releasing different
    values after the same observations. *)
Theorem lockstep_noninterference :
  forall (g : Env) (c : Cmd) (s1 s2 : State),
    well_typed g c = true -> wf g s1 -> wf g s2 -> low_eq g s1 s2 ->
    (step s1 c = done /\ step s2 c = done) \/
    (exists t, step s1 c = fail t /\ step s2 c = fail t) \/
    (exists c' s1' s2' t,
        step s1 c = next c' s1' t /\ step s2 c = next c' s2' t /\ low_eq g s1' s2') \/
    (exists c' s1' s2' p v1 v2,
        step s1 c = next c' s1' (p ++ [Orelease v1]) /\
        step s2 c = next c' s2' (p ++ [Orelease v2]) /\ v1 <> v2).
Proof.
  intros g c s1 s2 Hc W1 W2 L. change (lockstep g (step s1 c) (step s2 c)).
  induction c as [|x e|a i e|c1 IH1 c2 IH2|e c1 IH1 c2 IH2|n c IH|x e];
    cbn [well_typed] in Hc.
  - left. split; reflexivity.
  - (* assign: a public variable receives a public value *)
    destruct (lookup (vars g) x) as [d|] eqn:Ed; [|discriminate].
    destruct (type_expr g e) as [[t l]|] eqn:Ee; [|discriminate].
    apply andb_prop in Hc as [_ Hf].
    pose proof (eval_agree_typed g s1 s2 e t l W1 W2 L Ee) as A. cbn [step].
    destruct (eval s1 e) as [v1 t1|t1|], (eval s2 e) as [v2 t2|t2|]; simpl in A; try contradiction.
    + destruct A as [<- P].
      destruct (wf_var g s1 x d W1 Ed) as (u1 & E1 & _), (wf_var g s2 x d W2 Ed) as (u2 & E2 & _).
      rewrite E1, E2. apply lockstep_next.
      apply (low_eq_assign g s1 s2 x v1 v2 u1 u2 L E1 E2).
      intros d' Ed' Hp. rewrite Ed in Ed'. injection Ed' as <-.
      rewrite Hp in Hf. exact (P (flows_public l Hf)).
    + rewrite A. apply lockstep_fail.
  - (* set: a public index, and a public array receives a public value *)
    destruct (lookup (arrs g) a) as [d|] eqn:Ed; [|discriminate].
    destruct (type_expr g i) as [[[|] [|]]|] eqn:Ei; try discriminate.
    destruct (type_expr g e) as [[[|] l]|] eqn:Ee; try discriminate.
    pose proof (eval_agree_typed g s1 s2 i _ _ W1 W2 L Ei) as Ai.
    pose proof (eval_agree_typed g s1 s2 e _ _ W1 W2 L Ee) as Ae.
    pose proof (eval_typed g s1 i _ _ W1 Ei) as Gi.
    pose proof (eval_typed g s1 e _ _ W1 Ee) as Ge1. pose proof (eval_typed g s2 e _ _ W2 Ee) as Ge2.
    cbn [step].
    destruct (eval s1 i) as [vi1 ti1|ti1|], (eval s2 i) as [vi2 ti2|ti2|]; simpl in Ai; try contradiction;
      [|rewrite Ai; apply lockstep_fail].
    destruct Ai as [<- Pi]. specialize (Pi eq_refl) as <-.
    destruct vi1 as [n|b]; simpl in Gi; try discriminate.
    destruct (eval s1 e) as [v1 t1|t1|], (eval s2 e) as [v2 t2|t2|]; simpl in Ae; try contradiction;
      [|rewrite Ae; apply lockstep_fail].
    destruct Ae as [<- P].
    destruct v1 as [w1|], v2 as [w2|]; simpl in Ge1, Ge2; try discriminate.
    destruct (wf_array g s1 a d W1 Ed) as (cells1 & E1 & S1), (wf_array g s2 a d W2 Ed) as (cells2 & E2 & S2).
    rewrite E1, E2.
    assert (Len : length cells1 = length cells2) by (apply Nat2N.inj; congruence).
    destruct (lookup cells1 (val n)) as [c1|] eqn:C1, (lookup cells2 (val n)) as [c2|] eqn:C2.
    + apply lockstep_next, (low_eq_store g s1 s2 a cells1 cells2 (val n) w1 w2 L E1 E2).
      intros d' Ed' Hp. rewrite Ed in Ed'. injection Ed' as <-.
      rewrite Hp in Hc. specialize (P (flows_public l Hc)). injection P as ->. reflexivity.
    + rewrite (lookup_none cells2 cells1 (val n) (eq_sym Len) C2) in C1. discriminate.
    + rewrite (lookup_none cells1 cells2 (val n) Len C1) in C2. discriminate.
    + apply lockstep_fail.
  - (* seq *)
    apply andb_prop in Hc as [H1 _].
    destruct (skip_or_not c1) as [->|Hne]; [apply lockstep_next, L|].
    rewrite !step_seq by exact Hne.
    apply lockstep_wrap; [exact (IH1 H1)|]. intros D. exact (Hne (step_done s1 c1 D)).
  - (* cond: the condition is public, so both runs take the same branch *)
    assert (Ee : type_expr g e = Some (ty_bool, public)).
    { destruct (type_expr g e) as [[[|] [|]]|]; try discriminate; reflexivity. }
    pose proof (eval_agree_typed g s1 s2 e _ _ W1 W2 L Ee) as A.
    pose proof (eval_typed g s1 e _ _ W1 Ee) as G. cbn [step].
    destruct (eval s1 e) as [v1 t1|t1|], (eval s2 e) as [v2 t2|t2|]; simpl in A; try contradiction;
      [|rewrite A; apply lockstep_fail].
    destruct A as [<- P]. specialize (P eq_refl) as <-.
    destruct v1 as [w|b]; simpl in G; try discriminate.
    apply lockstep_next, L.
  - (* loop *)
    cbn [step]. destruct n; apply lockstep_next, L.
  - (* declassify: equal values keep the states low-equivalent; different
       values are the release that D2-TH03 allows *)
    destruct (lookup (vars g) x) as [d|] eqn:Ed; [|discriminate].
    destruct (type_expr g e) as [[t l]|] eqn:Ee; [|discriminate].
    pose proof (eval_agree_typed g s1 s2 e t l W1 W2 L Ee) as A. cbn [step].
    destruct (eval s1 e) as [v1 t1|t1|], (eval s2 e) as [v2 t2|t2|]; simpl in A; try contradiction;
      [|rewrite A; apply lockstep_fail].
    destruct A as [<- _].
    destruct (wf_var g s1 x d W1 Ed) as (u1 & E1 & _), (wf_var g s2 x d W2 Ed) as (u2 & E2 & _).
    rewrite E1, E2.
    destruct (val_eq_dec v1 v2) as [<-|Hne].
    + apply lockstep_next, (low_eq_assign g s1 s2 x v1 v1 u1 u2 L E1 E2). reflexivity.
    + apply lockstep_release, Hne.
Qed.

(** D2-TH04. Two runs of a well-typed command from low-equivalent
    well-formed states make the same observations and end in low-equivalent
    outcomes, or their observations first differ at a release. By induction
    on the fuel, from D2-TH03 and D2-TH02. *)
Theorem release_noninterference :
  forall (n : N) (g : Env) (c : Cmd) (s1 s2 : State),
    well_typed g c = true -> wf g s1 -> wf g s2 -> low_eq g s1 s2 ->
    (run_trace n s1 c = run_trace n s2 c /\
     outcome_low_eq g (run_outcome n s1 c) (run_outcome n s2 c)) \/
    (exists p v1 v2 r1 r2,
        run_trace n s1 c = p ++ Orelease v1 :: r1 /\
        run_trace n s2 c = p ++ Orelease v2 :: r2 /\ v1 <> v2).
Proof.
  intros n g. unfold run_trace, run_outcome.
  induction (N.to_nat n) as [|k IH]; intros c s1 s2 Hc W1 W2 L; cbn [run].
  - left. split; [reflexivity | split; [reflexivity | exact L]].
  - destruct (lockstep_noninterference g c s1 s2 Hc W1 W2 L)
      as [[E1 E2]|[(t & E1 & E2)|[(c' & s1' & s2' & t & E1 & E2 & L')|(c' & s1' & s2' & p & v1 & v2 & E1 & E2 & H)]]].
    + rewrite E1, E2. left. split; [reflexivity | exact L].
    + rewrite E1, E2. left. split; [reflexivity | exact I].
    + destruct (preservation g c s1 c' s1' t Hc W1 E1) as [Hc' W1'].
      destruct (preservation g c s2 c' s2' t Hc W2 E2) as [_ W2'].
      rewrite E1, E2; cbn [fst snd].
      destruct (IH c' s1' s2' Hc' W1' W2' L') as [[Et Eo]|(p & v1 & v2 & r1 & r2 & R1 & R2 & H)].
      * left. rewrite Et. split; [reflexivity | exact Eo].
      * right. exists (t ++ p), v1, v2, r1, r2. rewrite R1, R2, <- !app_assoc. auto.
    + rewrite E1, E2; cbn [fst snd]. right.
      exists p, v1, v2, (snd (run k s1' c')), (snd (run k s2' c')).
      rewrite <- !app_assoc. auto.
Qed.

(** * The witness and the counterexample (D2-TH05, D2-TH06) *)

Lemma p_plus_well_typed : well_typed gamma_plus p_plus = true.
Proof. reflexivity. Qed.

(** D2-TH05. P+ is well typed under Gamma+, so D2-TH04 applies to it. *)
Theorem positive_program_witness :
  forall (n : N) (s1 s2 : State),
    wf gamma_plus s1 -> wf gamma_plus s2 -> low_eq gamma_plus s1 s2 ->
    (run_trace n s1 p_plus = run_trace n s2 p_plus /\
     outcome_low_eq gamma_plus (run_outcome n s1 p_plus) (run_outcome n s2 p_plus)) \/
    (exists p v1 v2 r1 r2,
        run_trace n s1 p_plus = p ++ Orelease v1 :: r1 /\
        run_trace n s2 p_plus = p ++ Orelease v2 :: r2 /\ v1 <> v2).
Proof.
  intros n s1 s2. exact (release_noninterference n gamma_plus p_plus s1 s2 p_plus_well_typed).
Qed.

(** D2-TH06. P- (ill typed: it branches on the secret key) reveals the key
    through its branch observation from two low-equivalent states. *)
Theorem negative_program_leaks :
  wf gamma_plus sigma_a /\ wf gamma_plus sigma_b /\ low_eq gamma_plus sigma_a sigma_b /\
  exists p r1 r2,
    run_trace 8 sigma_a p_minus = p ++ Obranch true :: r1 /\
    run_trace 8 sigma_b p_minus = p ++ Obranch false :: r2.
Proof.
  split; [|split; [|split]].
  - split; repeat constructor.
  - split; repeat constructor.
  - (* the two states differ only in variable 0, which is secret *)
    split; intros x d Ed Hp; unfold lookup in *; destruct (N.to_nat x) as [|[|[|[|i]]]];
      simpl in Ed; try (injection Ed as <-; discriminate Hp); try reflexivity; destruct i; discriminate.
  - exists [], [], []. split; reflexivity.
Qed.
