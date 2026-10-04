(** D-006 v0.3, candidate C-01 (Rocq): the DS-01 Core fragment.

    Every shared T-, C- and F- symbol of DS-01 maps to one declaration here
    or in the Rocq standard library (see adapter.json). Words are naturals
    below [2 ^ w] carried with a boolean bound, so a closed word computes to
    a canonical value and its bound to [I]. Observations are proved by
    [vm_compute; reflexivity], the kernel's bytecode evaluator. *)

From Stdlib Require Import NArith Bool List Lia ZArith.
Import ListNotations.

Local Open Scope N_scope.

(** * Booleans as propositions *)

Definition truth {b : bool} (H : b = true) : Is_true b :=
  match b as b0 return b0 = true -> Is_true b0 with
  | true => fun _ => I
  | false => fun H0 => match diff_false_true H0 with end
  end H.

Lemma Is_true_unique (b : bool) (p q : Is_true b) : p = q.
Proof. destruct b; destruct p; destruct q; reflexivity. Qed.

Lemma Is_true_true (b : bool) : Is_true b -> b = true.
Proof. destruct b; simpl; tauto. Qed.

(** * Exact words (T-03) *)

Record Word (w : N) : Type := mkWord { val : N ; bound : Is_true (N.ltb val (2 ^ w)) }.
Arguments mkWord {w} val bound.
Arguments val {w} _.
Arguments bound {w} _.

Lemma mod_bound (w n : N) : N.ltb (n mod 2 ^ w) (2 ^ w) = true.
Proof. apply N.ltb_lt, N.mod_lt, N.pow_nonzero. discriminate. Qed.

(** F-01 and F-02 *)
Definition word_of_nat (w n : N) : Word w := mkWord (n mod 2 ^ w) (truth (mod_bound w n)).
Definition nat_of_word (w : N) (x : Word w) : N := val x.

Lemma val_bound (w : N) (x : Word w) : val x < 2 ^ w.
Proof. apply N.ltb_lt, Is_true_true, bound. Qed.

Lemma word_ext (w : N) (x y : Word w) : val x = val y -> x = y.
Proof.
  destruct x as [vx px], y as [vy py]; simpl; intros E; subst vy.
  f_equal; apply Is_true_unique.
Qed.

Lemma val_word_of_nat (w n : N) : val (word_of_nat w n) = n mod 2 ^ w.
Proof. reflexivity. Qed.

Lemma word_of_nat_val (w : N) (x : Word w) : word_of_nat w (val x) = x.
Proof. apply word_ext; simpl; apply N.mod_small, val_bound. Qed.

(** F-03 to F-08 *)
Definition word_add (w : N) (x y : Word w) : Word w := word_of_nat w (val x + val y).
Definition word_xor (w : N) (x y : Word w) : Word w := word_of_nat w (N.lxor (val x) (val y)).
Definition word_and (w : N) (x y : Word w) : Word w := word_of_nat w (N.land (val x) (val y)).
Definition word_not (w : N) (x : Word w) : Word w := word_of_nat w (2 ^ w - 1 - val x).
Definition word_rotl (w : N) (x : Word w) (r : N) : Word w :=
  if N.eqb w 0 then x
  else let k := r mod w in word_of_nat w (N.lor (N.shiftl (val x) k) (N.shiftr (val x) (w - k))).
Definition word_shl (w : N) (x : Word w) (r : N) : Word w := word_of_nat w (N.shiftl (val x) r).

(** * Length-indexed sequences (T-04) *)

Record Seq (A : Type) (n : N) : Type :=
  mkSeq { items : list A ; length_ok : Is_true (N.eqb (N.of_nat (length items)) n) }.
Arguments mkSeq {A n} items length_ok.
Arguments items {A n} _.
Arguments length_ok {A n} _.

Lemma seq_ext (A : Type) (n : N) (s t : Seq A n) : items s = items t -> s = t.
Proof.
  destruct s as [l p], t as [m q]; simpl; intros E; subst m.
  f_equal; apply Is_true_unique.
Qed.

Lemma seq_length (A : Type) (n : N) (s : Seq A n) : N.of_nat (length (items s)) = n.
Proof. apply N.eqb_eq, Is_true_true, length_ok. Qed.

(** F-17 and F-20 *)
Definition seq4 (A : Type) (a b c d : A) : Seq A 4 := @mkSeq A 4 [a; b; c; d] I.
Definition seq_to_list (A : Type) (n : N) (s : Seq A n) : list A := items s.

(** F-13 *)
Fixpoint zip_with {A : Type} (f : A -> A -> A) (l m : list A) : list A :=
  match l, m with
  | a :: l', b :: m' => f a b :: zip_with f l' m'
  | _, _ => []
  end.

Lemma zip_with_length {A : Type} (f : A -> A -> A) (l m : list A) :
  length l = length m -> length (zip_with f l m) = length l.
Proof.
  revert m; induction l as [|a l IH]; intros [|b m]; simpl; try discriminate; auto.
Qed.

Lemma seq_map2_ok (A : Type) (n : N) (f : A -> A -> A) (s t : Seq A n) :
  N.eqb (N.of_nat (length (zip_with f (items s) (items t)))) n = true.
Proof.
  apply N.eqb_eq. rewrite zip_with_length; [apply seq_length|].
  apply Nnat.Nat2N.inj. rewrite !seq_length. reflexivity.
Qed.

Definition seq_map2 (A : Type) (n : N) (f : A -> A -> A) (s t : Seq A n) : Seq A n :=
  mkSeq (zip_with f (items s) (items t)) (truth (seq_map2_ok A n f s t)).

(** * Endian conversion (F-09 to F-12) *)

Definition be32 (a b c d : Word 8) : Word 32 :=
  word_of_nat 32 (((val a * 256 + val b) * 256 + val c) * 256 + val d).

Definition byte_list (x : Word 32) : list (Word 8) :=
  let x1 := val x / 256 in
  let x2 := x1 / 256 in
  [word_of_nat 8 (x2 / 256); word_of_nat 8 x2; word_of_nat 8 x1; word_of_nat 8 (val x)].

Definition be32_of_bytes (s : Seq (Word 8) 4) : Word 32 :=
  match items s with
  | [a; b; c; d] => be32 a b c d
  | _ => word_of_nat 32 0
  end.

Definition bytes_of_be32 (x : Word 32) : Seq (Word 8) 4 := @mkSeq _ 4 (byte_list x) I.

Definition le32_of_bytes (s : Seq (Word 8) 4) : Word 32 :=
  match items s with
  | [a; b; c; d] => be32 d c b a
  | _ => word_of_nat 32 0
  end.

Definition bytes_of_le32 (x : Word 32) : Seq (Word 8) 4 := @mkSeq _ 4 (rev (byte_list x)) I.

(** * Results, the decoder and the encoder (T-06, T-07, F-14 to F-16) *)

Inductive Result (A E : Type) : Type :=
| ok : A -> Result A E
| err : E -> Result A E.

Inductive DecodeError : Type := too_many | truncated | trailing.

Definition list_length (A : Type) (l : list A) : N := N.of_nat (length l).

Fixpoint group4 (l : list (Word 8)) : list (Word 32) :=
  match l with
  | a :: b :: c :: d :: rest => be32 a b c d :: group4 rest
  | _ => []
  end.

Definition decode_words (bytes : list (Word 8)) : Result (list (Word 32)) DecodeError :=
  match bytes with
  | [] => err _ _ truncated
  | count :: rest =>
      let n := val count in
      let have := N.of_nat (length rest) in
      if N.ltb 16 n then err _ _ too_many
      else if N.ltb have (4 * n) then err _ _ truncated
      else if N.ltb (4 * n) have then err _ _ trailing
      else ok _ _ (group4 rest)
  end.

Definition encode_words (l : list (Word 32)) : list (Word 8) :=
  word_of_nat 8 (N.of_nat (length l)) :: flat_map byte_list l.

(** F-21 is [N.pow] and F-22 is [app]. *)

(** * The parameterized quarter round (M-01, F-18, F-19) *)

Module Type QRParams.
  Parameter w r1 r2 r3 r4 : N.
End QRParams.

Module QuarterRound (P : QRParams).
  Definition qr (s : Seq (Word P.w) 4) : Seq (Word P.w) 4 :=
    match items s with
    | [a; b; c; d] =>
        let a := word_add P.w a b in let d := word_rotl P.w (word_xor P.w d a) P.r1 in
        let c := word_add P.w c d in let b := word_rotl P.w (word_xor P.w b c) P.r2 in
        let a := word_add P.w a b in let d := word_rotl P.w (word_xor P.w d a) P.r3 in
        let c := word_add P.w c d in let b := word_rotl P.w (word_xor P.w b c) P.r4 in
        seq4 _ a b c d
    | _ => s
    end.
End QuarterRound.

Module ChaChaParams <: QRParams.
  Definition w := 32. Definition r1 := 16. Definition r2 := 12. Definition r3 := 8. Definition r4 := 7.
End ChaChaParams.
Module ToyParams <: QRParams.
  Definition w := 8. Definition r1 := 4. Definition r2 := 3. Definition r3 := 2. Definition r4 := 1.
End ToyParams.

Module ChaChaQR := QuarterRound ChaChaParams.
Module ToyQR := QuarterRound ToyParams.

Definition chacha_qr : Seq (Word 32) 4 -> Seq (Word 32) 4 := ChaChaQR.qr.
Definition toy_qr : Seq (Word 8) 4 -> Seq (Word 8) 4 := ToyQR.qr.

(** * Exhaustive evaluation over a word width (the declared exhaustive method) *)

Fixpoint all_below (k : nat) (f : N -> bool) (base : N) : bool :=
  match k with
  | O => f base
  | S k' => all_below k' f base && all_below k' f (base + 2 ^ N.of_nat k')
  end.

Lemma all_below_sound (k : nat) (f : N -> bool) (base : N) :
  all_below k f base = true -> forall n, base <= n < base + 2 ^ N.of_nat k -> f n = true.
Proof.
  revert base; induction k as [|k IH]; intros base H n Hn; cbn [all_below] in H.
  - change (2 ^ N.of_nat 0) with 1 in Hn. replace n with base by lia. exact H.
  - apply andb_prop in H as [H1 H2].
    rewrite Nnat.Nat2N.inj_succ, N.pow_succ_r' in Hn.
    destruct (N.lt_ge_cases n (base + 2 ^ N.of_nat k)).
    + apply (IH base H1); lia.
    + apply (IH (base + 2 ^ N.of_nat k) H2); lia.
Qed.

Definition word_eqb {w : N} (x y : Word w) : bool := N.eqb (val x) (val y).

Lemma exhaustive_eq (w v : N) (L R : Word w -> Word v) :
  all_below (N.to_nat w) (fun n => word_eqb (L (word_of_nat w n)) (R (word_of_nat w n))) 0 = true ->
  forall x, L x = R x.
Proof.
  intros H x.
  pose proof (all_below_sound _ _ _ H (val x)) as Hx.
  rewrite Nnat.N2Nat.id in Hx.
  specialize (Hx (conj (N.le_0_l _) (val_bound w x))).
  rewrite word_of_nat_val in Hx.
  apply word_ext, N.eqb_eq, Hx.
Qed.

Ltac exhaustive_word := apply exhaustive_eq; vm_compute; reflexivity.

(** * Theorems D1-TH01 to D1-TH12 *)

Theorem word_bounded : forall (w : N) (x : Word w), nat_of_word w x < N.pow 2 w.
Proof. intros; apply val_bound. Qed.

Theorem small_naturals_are_words :
  forall (w n : N), n < N.pow 2 w -> nat_of_word w (word_of_nat w n) = n.
Proof. intros w n H; apply N.mod_small, H. Qed.

Theorem words_are_their_values :
  forall (w : N) (x y : Word w), nat_of_word w x = nat_of_word w y -> x = y.
Proof. intros; apply word_ext; assumption. Qed.

Theorem addition_commutes : forall (w : N) (x y : Word w), word_add w x y = word_add w y x.
Proof. intros; unfold word_add; rewrite N.add_comm; reflexivity. Qed.

Theorem xor_cancels : forall (w : N) (x : Word w), word_xor w x x = word_of_nat w 0.
Proof. intros; unfold word_xor; rewrite N.lxor_nilpotent; reflexivity. Qed.

(** Rotation, bit by bit *)

Lemma testbit_val_high (w : N) (x : Word w) (i : N) : w <= i -> N.testbit (val x) i = false.
Proof.
  intros H. pose proof (val_bound w x) as B.
  destruct (N.eq_dec (val x) 0) as [->|Hne]; [apply N.bits_0|].
  apply N.bits_above_log2. apply N.log2_lt_pow2 in B; [lia|lia].
Qed.

Lemma rotl_bits (w : N) (x : Word w) (r i : N) : w <> 0 ->
  N.testbit (val (word_rotl w x r)) i =
  if N.ltb i w then N.testbit (val x) ((i + (w - r mod w)) mod w) else false.
Proof.
  intros Hw. unfold word_rotl. rewrite <- N.eqb_neq in Hw. rewrite Hw. apply N.eqb_neq in Hw.
  cbn [val word_of_nat]. set (k := r mod w).
  assert (Hk : k < w) by (apply N.mod_lt; exact Hw).
  destruct (N.ltb_spec i w) as [Hi|Hi].
  - rewrite N.mod_pow2_bits_low by exact Hi. rewrite N.lor_spec.
    destruct (N.ltb_spec i k) as [Hik|Hik].
    + rewrite N.shiftl_spec_low by exact Hik. rewrite N.shiftr_spec'. simpl.
      f_equal. rewrite N.mod_small; lia.
    + rewrite N.shiftl_spec_high' by exact Hik. rewrite N.shiftr_spec'.
      rewrite (testbit_val_high w x (i + (w - k))) by lia. rewrite orb_false_r.
      f_equal. replace (i + (w - k)) with ((i - k) + 1 * w) by lia.
      rewrite N.Div0.mod_add. rewrite N.mod_small; lia.
  - apply N.mod_pow2_bits_high; exact Hi.
Qed.

Theorem rotations_compose :
  forall (w : N) (x : Word w) (r s : N), word_rotl w (word_rotl w x r) s = word_rotl w x (r + s).
Proof.
  intros w x r s.
  destruct (N.eq_dec w 0) as [->|Hw]; [reflexivity|].
  apply word_ext, N.bits_inj; intros i.
  rewrite !rotl_bits by exact Hw.
  destruct (N.ltb_spec i w) as [Hi|Hi]; [|reflexivity].
  assert (Hm : (i + (w - s mod w)) mod w < w) by (apply N.mod_lt; exact Hw).
  rewrite <- N.ltb_lt in Hm. rewrite Hm.
  f_equal.
  pose proof (N.mod_lt r w Hw). pose proof (N.mod_lt s w Hw). pose proof (N.mod_lt (r + s) w Hw).
  rewrite N.Div0.add_mod_idemp_l.
  assert (E : (r + s) mod w = (r mod w + s mod w) mod w) by (apply N.Div0.add_mod).
  rewrite E.
  set (a := r mod w) in *. set (b := s mod w) in *.
  apply N.ltb_lt in Hm.
  (* both sides are (i + 2w - a - b) mod w *)
  destruct (N.ltb_spec (a + b) w) as [Hab|Hab].
  - rewrite (N.mod_small (a + b)) by exact Hab.
    replace (i + (w - b) + (w - a)) with ((i + (w - (a + b))) + 1 * w) by lia.
    rewrite N.Div0.mod_add. reflexivity.
  - assert (E2 : (a + b) mod w = a + b - w).
    { assert (E3 : a + b = (a + b - w) + 1 * w) by lia. rewrite E3 at 1.
      rewrite N.Div0.mod_add. apply N.mod_small. lia. }
    rewrite E2. f_equal. lia.
Qed.

(** Byte round trips *)

Lemma div_mod_256 (a : N) : a = (a / 256) * 256 + a mod 256.
Proof. pose proof (N.div_mod a 256 ltac:(discriminate)). lia. Qed.

Lemma horner_div (y d : N) : d < 256 -> (y * 256 + d) / 256 = y.
Proof.
  intros H. rewrite N.div_add_l by discriminate. rewrite N.div_small by exact H. lia.
Qed.

Lemma horner_mod (y d : N) : d < 256 -> (y * 256 + d) mod 256 = d.
Proof.
  intros H. rewrite N.add_comm, N.Div0.mod_add. apply N.mod_small, H.
Qed.

Lemma byte_val (x : Word 8) : val x < 256.
Proof. apply (val_bound 8 x). Qed.

Theorem be32_word_round_trip : forall (x : Word 32), be32_of_bytes (bytes_of_be32 x) = x.
Proof.
  intros x. apply word_ext. unfold be32_of_bytes, bytes_of_be32, byte_list, be32. cbn [items val word_of_nat].
  pose proof (val_bound 32 x) as B. change (2 ^ 32) with 4294967296 in *.
  set (v := val x) in *.
  assert (Q : v / 256 / 256 / 256 < 256).
  { rewrite !N.div_div by discriminate. apply N.div_lt_upper_bound; [discriminate|]. lia. }
  rewrite (N.mod_small (v / 256 / 256 / 256) (2 ^ 8)) by exact Q.
  change (2 ^ 8) with 256.
  rewrite <- (div_mod_256 (v / 256 / 256)), <- (div_mod_256 (v / 256)), <- (div_mod_256 v).
  apply N.mod_small; exact B.
Qed.

Lemma seq4_cases (A : Type) (s : Seq A 4) : exists a b c d, items s = [a; b; c; d].
Proof.
  assert (L : length (items s) = 4%nat).
  { apply Nnat.Nat2N.inj. rewrite seq_length. reflexivity. }
  destruct (items s) as [|a [|b [|c [|d [|e l]]]]]; simpl in L; try lia.
  exists a, b, c, d; reflexivity.
Qed.

Lemma bytes_of_be32_items (a b c d : Word 8) :
  byte_list (be32 a b c d) = [a; b; c; d].
Proof.
  pose proof (byte_val a) as Ha; pose proof (byte_val b) as Hb.
  pose proof (byte_val c) as Hc; pose proof (byte_val d) as Hd.
  assert (E0 : val (be32 a b c d) = ((val a * 256 + val b) * 256 + val c) * 256 + val d).
  { unfold be32; cbn [val word_of_nat]. apply N.mod_small. change (2 ^ 32) with 4294967296. lia. }
  unfold byte_list. rewrite E0.
  rewrite !horner_div by assumption.
  repeat f_equal; apply word_ext; cbn [val word_of_nat]; change (2 ^ 8) with 256;
    first [apply horner_mod; assumption | apply N.mod_small; assumption].
Qed.

Theorem be32_bytes_round_trip : forall (s : Seq (Word 8) 4), bytes_of_be32 (be32_of_bytes s) = s.
Proof.
  intros s. apply seq_ext. destruct (seq4_cases _ s) as (a & b & c & d & E).
  unfold be32_of_bytes, bytes_of_be32. cbn [items]. rewrite E. apply bytes_of_be32_items.
Qed.

Theorem le32_word_round_trip : forall (x : Word 32), le32_of_bytes (bytes_of_le32 x) = x.
Proof.
  intros x. pose proof (be32_word_round_trip x) as H.
  unfold be32_of_bytes, bytes_of_be32 in H. unfold le32_of_bytes, bytes_of_le32. cbn [items] in *.
  unfold byte_list in *. simpl rev. exact H.
Qed.

(** Decoder and encoder *)

Lemma group4_bytes (l : list (Word 32)) : group4 (flat_map byte_list l) = l.
Proof.
  induction l as [|x l IH]; [reflexivity|].
  simpl flat_map. pose proof (be32_word_round_trip x) as H.
  unfold be32_of_bytes, bytes_of_be32 in H. cbn [items] in H.
  unfold byte_list in *. simpl. rewrite IH. f_equal. exact H.
Qed.

Lemma flat_map_length (l : list (Word 32)) : length (flat_map byte_list l) = (4 * length l)%nat.
Proof. induction l as [|x l IH]; simpl; [reflexivity|]. rewrite IH. lia. Qed.

Theorem decoder_inverts_encoder :
  forall (l : list (Word 32)),
    list_length (Word 32) l <= 16 -> decode_words (encode_words l) = ok _ _ l.
Proof.
  intros l H. unfold list_length in H. unfold decode_words, encode_words.
  cbn [val word_of_nat]. change (2 ^ 8) with 256.
  rewrite (N.mod_small (N.of_nat (length l))) by lia.
  rewrite flat_map_length.
  replace (N.of_nat (4 * length l)) with (4 * N.of_nat (length l)) by lia.
  destruct (N.ltb_spec 16 (N.of_nat (length l))); [lia|].
  rewrite N.ltb_irrefl. rewrite group4_bytes. reflexivity.
Qed.

Lemma group4_length (rest : list (Word 8)) (n : nat) :
  length rest = (4 * n)%nat -> length (group4 rest) = n /\ flat_map byte_list (group4 rest) = rest.
Proof.
  revert rest; induction n as [|n IH]; intros rest L.
  - destruct rest; [split; reflexivity|discriminate].
  - destruct rest as [|a [|b [|c [|d rest]]]]; simpl in L; try lia.
    assert (L' : length rest = (4 * n)%nat) by lia.
    destruct (IH rest L') as [H1 H2]. cbn [group4 flat_map length]. split; [lia|].
    rewrite bytes_of_be32_items, H2. reflexivity.
Qed.

Theorem decoding_is_canonical :
  forall (b : list (Word 8)) (l : list (Word 32)),
    decode_words b = ok _ _ l -> encode_words l = b.
Proof.
  intros [|count rest] l H; [discriminate|].
  unfold decode_words in H.
  destruct (N.ltb_spec 16 (val count)); [discriminate|].
  destruct (N.ltb_spec (N.of_nat (length rest)) (4 * val count)); [discriminate|].
  destruct (N.ltb_spec (4 * val count) (N.of_nat (length rest))); [discriminate|].
  injection H as <-.
  assert (L : length rest = (4 * N.to_nat (val count))%nat) by lia.
  destruct (group4_length rest _ L) as [G1 G2].
  unfold encode_words. rewrite G1, G2, Nnat.N2Nat.id. f_equal.
  rewrite <- (word_of_nat_val 8 count) at 2. reflexivity.
Qed.

Theorem sequences_have_their_index_length :
  forall (A : Type) (n : N) (s : Seq A n), list_length A (seq_to_list A n s) = n.
Proof. intros; apply seq_length. Qed.
