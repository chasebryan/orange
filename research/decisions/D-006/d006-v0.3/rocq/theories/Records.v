(** D-006 v0.3, candidate C-01 (Rocq): DS-03, the OCR1 canonical record format.

    Every shared R- symbol maps to one declaration here (see
    adapter.d/ds03.json). The decoder reads a byte list left to right and
    reports the first failure with its code and a primitive-string path; the
    encoder writes the one canonical encoding; [records_valid] states the
    validity conditions of semantics.md, section 4, on values, without the
    decoder. Everything is structural recursion (on the input list, on the
    record count or on a reference count), so observations hold by
    [vm_compute; reflexivity]. *)

From D006 Require Import Core.
From Stdlib Require Import NArith ZArith Bool List Lia.
From Stdlib Require PrimString.
Import ListNotations PrimString.PStringNotations.

Local Open Scope N_scope.
Local Open Scope pstring_scope.

#[local] Arguments ok {A E} _.
#[local] Arguments err {A E} _.

Local Abbreviation bytes := (list (Word 8)).
Local Abbreviation string := PrimString.string.

(** The length of a list as a binary natural. *)
Local Abbreviation size l := (N.of_nat (length l)).

(** * Records, error codes and failures (R-T01, R-T02, R-T03) *)

Inductive Record : Type :=
| rdef (name : list (Word 8)) (digest : list (Word 8))
| rthm (name : list (Word 8)) (fingerprint : list (Word 8)) (refs : list N)
| rclaim (name : list (Word 8)) (ref : N) (level : N).

(** Within this module [truncated] is this code, not the DS-01 decoder's
    [D006.Core.truncated]. *)
Inductive ErrorCode : Type :=
| oversized | bad_magic | unknown_version | truncated | malformed_number
| noncanonical_number | unknown_field | invalid_name | invalid_utf8
| duplicate_name | noncanonical_order | reference_escape | cyclic_reference
| reference_kind | invalid_level | trailing_data.

Inductive Failure : Type := failure (code : ErrorCode) (path : PrimString.string).

Definition name_of (r : Record) : bytes :=
  match r with rdef name _ | rthm name _ _ | rclaim name _ _ => name end.

Definition tag_of (r : Record) : N :=
  match r with rdef _ _ => 1 | rthm _ _ _ => 2 | rclaim _ _ _ => 3 end.

Definition is_theorem (r : option Record) : bool :=
  match r with Some (rthm _ _ _) => true | _ => false end.

(** * UTF-8 (R-F04), after the table of RFC 3629, section 4 *)

Definition within (lo hi : N) (b : Word 8) : bool := (lo <=? val b) && (val b <=? hi).

(** A continuation byte. *)
Definition cont : Word 8 -> bool := within 0x80 0xBF.

(** What a lead byte starts: a one-byte sequence, a sequence of two, three
    or four bytes whose second byte lies in the given range (every later
    byte is a continuation byte), or nothing valid. *)
Inductive Lead : Type :=
| single | two (lo hi : N) | three (lo hi : N) | four (lo hi : N) | invalid.

Definition lead (v : N) : Lead :=
  if v <? 0x80 then single
  else if (0xC2 <=? v) && (v <=? 0xDF) then two 0x80 0xBF
  else if v =? 0xE0 then three 0xA0 0xBF
  else if v =? 0xED then three 0x80 0x9F
  else if (0xE1 <=? v) && (v <=? 0xEF) then three 0x80 0xBF
  else if v =? 0xF0 then four 0x90 0xBF
  else if (0xF1 <=? v) && (v <=? 0xF3) then four 0x80 0xBF
  else if v =? 0xF4 then four 0x80 0x8F
  else invalid.

Fixpoint utf8_valid (bs : list (Word 8)) : bool :=
  match bs with
  | [] => true
  | b :: rest =>
      match lead (val b), rest with
      | single, _ => utf8_valid rest
      | two lo hi, c1 :: rest' => within lo hi c1 && utf8_valid rest'
      | three lo hi, c1 :: c2 :: rest' => within lo hi c1 && cont c2 && utf8_valid rest'
      | four lo hi, c1 :: c2 :: c3 :: rest' =>
          within lo hi c1 && cont c2 && cont c3 && utf8_valid rest'
      | _, _ => false
      end
  end.

(** * Bytewise order *)

Fixpoint bytes_compare (x y : bytes) : comparison :=
  match x, y with
  | [], [] => Eq
  | [], _ :: _ => Lt
  | _ :: _, [] => Gt
  | a :: x', b :: y' =>
      match N.compare (val a) (val b) with
      | Eq => bytes_compare x' y'
      | c => c
      end
  end.

Definition bytes_lt (x y : bytes) : bool :=
  match bytes_compare x y with Lt => true | _ => false end.

(** * Encoding (R-F02) *)

Definition byte (n : N) : Word 8 := word_of_nat 8 n.

(** Numbers: a value below 0x80 is one byte; otherwise two bytes, the low
    seven bits with the high bit set, then the rest. *)
Definition uvar (n : N) : bytes :=
  if n <? 128 then [byte n] else [byte (128 + n mod 128); byte (n / 128)].

(** "OCR" and version 1. *)
Definition magic : list N := [79; 67; 82].
Definition version : N := 1.

Definition encode_body (r : Record) : bytes :=
  match r with
  | rdef _ digest => digest
  | rthm _ fingerprint refs => fingerprint ++ uvar (size refs) ++ flat_map uvar refs
  | rclaim _ ref level => uvar ref ++ [byte level]
  end.

Definition encode_record (r : Record) : bytes :=
  byte (tag_of r) :: uvar (size (name_of r)) ++ name_of r ++ encode_body r.

Definition encode_records (l : list Record) : list (Word 8) :=
  map byte magic ++ byte version :: uvar (size l) ++ flat_map encode_record l.

(** * Validity (R-F03), from the stated conditions *)

(** A name is 1 to 200 bytes of well-formed UTF-8. *)
Definition name_ok (name : bytes) : bool :=
  (1 <=? size name) && (size name <=? 200) && utf8_valid name.

(** Strictly increasing under [lt]: every element is below the next one. *)
Fixpoint increasing {A : Type} (lt : A -> A -> bool) (l : list A) : bool :=
  match l with
  | [] => true
  | x :: rest =>
      match rest with
      | [] => true
      | y :: _ => lt x y && increasing lt rest
      end
  end.

Definition below (bound ref : N) : bool := ref <? bound.

(** The conditions on the body of record [r], whose index is the number of
    records [earlier] that come before it: a 32-byte digest; a 32-byte
    fingerprint and at most 64 strictly increasing references, each below
    the record's own index; or a reference below the record's own index to
    a theorem, and a level of at most 3. *)
Definition body_ok (earlier : list Record) (r : Record) : bool :=
  match r with
  | rdef _ digest => size digest =? 32
  | rthm _ fingerprint refs =>
      (size fingerprint =? 32) && (size refs <=? 64) && increasing N.ltb refs
      && forallb (below (size earlier)) refs
  | rclaim _ ref level =>
      below (size earlier) ref && is_theorem (nth_error earlier (N.to_nat ref)) && (level <=? 3)
  end.

(** Every record's body is good given the records before it. *)
Fixpoint bodies_ok (earlier l : list Record) : bool :=
  match l with
  | [] => true
  | r :: rest => body_ok earlier r && bodies_ok (earlier ++ [r]) rest
  end.

(** At most 64 records, good names in strictly increasing bytewise order,
    good bodies, and an encoding of at most 4096 bytes. *)
Definition records_valid (l : list Record) : bool :=
  (size l <=? 64)
  && forallb name_ok (map name_of l)
  && increasing bytes_lt (map name_of l)
  && bodies_ok [] l
  && (size (encode_records l) <=? 4096).

(** * Decoding (R-F01) *)

Definition fail {A : Type} (code : ErrorCode) (path : string) : Result A Failure :=
  err (failure code path).

Definition bind {A B : Type} (r : Result A Failure) (k : A -> Result B Failure) : Result B Failure :=
  match r with
  | ok a => k a
  | err e => err e
  end.

Local Notation "'let?' p ':=' r 'in' k" := (bind r (fun p => k))
  (at level 200, p pattern, r at level 100, k at level 200).

(** Decimal numerals for failure paths. A number has at most as many
    decimal digits as binary ones, which bounds the recursion. *)
Definition digit (d : N) : string :=
  nth (N.to_nat d) ["0"; "1"; "2"; "3"; "4"; "5"; "6"; "7"; "8"; "9"] "?".

Fixpoint decimal_from (fuel : nat) (n : N) (acc : string) : string :=
  match fuel with
  | O => acc
  | S fuel' =>
      let acc := PrimString.cat (digit (n mod 10)) acc in
      if n <? 10 then acc else decimal_from fuel' (n / 10) acc
  end.

Definition decimal (n : N) : string := decimal_from (S (N.size_nat n)) n "".

(** [records/INDEX] followed by [field]. *)
Definition record_path (index : N) (field : string) : string :=
  PrimString.cat (PrimString.cat "records/" (decimal index)) field.

(** Each reader takes the remaining bytes and returns a value and the bytes
    after it. *)

Definition read_byte (path : string) (bs : bytes) : Result (Word 8 * bytes) Failure :=
  match bs with
  | [] => fail truncated path
  | b :: rest => ok (b, rest)
  end.

Fixpoint split_at (n : nat) (bs : bytes) : option (bytes * bytes) :=
  match n, bs with
  | O, _ => Some ([], bs)
  | S n', b :: bs' =>
      match split_at n' bs' with
      | Some (chunk, rest) => Some (b :: chunk, rest)
      | None => None
      end
  | S _, [] => None
  end.

Definition read_bytes (n : N) (path : string) (bs : bytes) : Result (bytes * bytes) Failure :=
  match split_at (N.to_nat n) bs with
  | Some (chunk, rest) => ok (chunk, rest)
  | None => fail truncated path
  end.

Definition read_uvar (path : string) (bs : bytes) : Result (N * bytes) Failure :=
  match bs with
  | [] => fail truncated path
  | b0 :: bs0 =>
      if val b0 <? 128 then ok (val b0, bs0)
      else
        match bs0 with
        | [] => fail truncated path
        | b1 :: bs1 =>
            if 128 <=? val b1 then fail malformed_number path
            else if val b1 =? 0 then fail noncanonical_number path
            else ok (val b0 - 128 + 128 * val b1, bs1)
        end
  end.

(** The magic, byte by byte. *)
Fixpoint read_magic (expected : list N) (bs : bytes) : Result bytes Failure :=
  match expected, bs with
  | [], _ => ok bs
  | _ :: _, [] => fail truncated "header"
  | e :: es, b :: bs' => if val b =? e then read_magic es bs' else fail bad_magic "header"
  end.

(** [name] comes strictly after [previous], if there is one. *)
Definition after (previous : option bytes) (name : bytes) : bool :=
  match previous with
  | None => true
  | Some p => bytes_lt p name
  end.

Definition read_name (path : string) (previous : option bytes) (bs : bytes)
  : Result (bytes * bytes) Failure :=
  let? (len, bs) := read_uvar path bs in
  if (len =? 0) || (200 <? len) then fail invalid_name path else
  let? (name, bs) := read_bytes len path bs in
  if negb (utf8_valid name) then fail invalid_utf8 path else
  match previous with
  | None => ok (name, bs)
  | Some p =>
      match bytes_compare p name with
      | Lt => ok (name, bs)
      | Eq => fail duplicate_name path
      | Gt => fail noncanonical_order path
      end
  end.

(** [ref] comes strictly after [previous], if there is one. *)
Definition ref_after (previous : option N) (ref : N) : bool :=
  match previous with
  | None => true
  | Some p => p <? ref
  end.

(** [todo] references of the theorem at [index], from slot [slot] on. *)
Fixpoint read_refs (todo : nat) (count index slot : N) (previous : option N) (bs : bytes)
  : Result (list N * bytes) Failure :=
  match todo with
  | O => ok ([], bs)
  | S todo' =>
      let path := PrimString.cat (record_path index "/refs/") (decimal slot) in
      let? (ref, bs) := read_uvar path bs in
      if count <=? ref then fail reference_escape path
      else if index <=? ref then fail cyclic_reference path
      else if negb (ref_after previous ref) then fail noncanonical_order path
      else
        let? (refs, bs) := read_refs todo' count index (slot + 1) (Some ref) bs in
        ok (ref :: refs, bs)
  end.

(** The name of the last record, if there is one. *)
Definition last_name (earlier : list Record) : option bytes :=
  last (map (fun r => Some (name_of r)) earlier) None.

(** The record after [earlier] in a set of [count] records. *)
Definition read_record (count : N) (earlier : list Record) (bs : bytes)
  : Result (Record * bytes) Failure :=
  let index := size earlier in
  let at_ := record_path index in
  let? (tag, bs) := read_byte (at_ "/tag") bs in
  if (val tag =? 0) || (3 <? val tag) then fail unknown_field (at_ "/tag") else
  let? (name, bs) := read_name (at_ "/name") (last_name earlier) bs in
  if val tag =? 1 then
    let? (digest, bs) := read_bytes 32 (at_ "/digest") bs in
    ok (rdef name digest, bs)
  else if val tag =? 2 then
    let? (fingerprint, bs) := read_bytes 32 (at_ "/fingerprint") bs in
    let? (total, bs) := read_uvar (at_ "/refs") bs in
    if 64 <? total then fail oversized (at_ "/refs") else
    let? (refs, bs) := read_refs (N.to_nat total) count index 0 None bs in
    ok (rthm name fingerprint refs, bs)
  else
    let? (ref, bs) := read_uvar (at_ "/ref") bs in
    if count <=? ref then fail reference_escape (at_ "/ref")
    else if index <=? ref then fail cyclic_reference (at_ "/ref")
    else if negb (is_theorem (nth_error earlier (N.to_nat ref)))
    then fail reference_kind (at_ "/ref")
    else
      let? (level, bs) := read_byte (at_ "/level") bs in
      if 3 <? val level then fail invalid_level (at_ "/level") else
      ok (rclaim name ref (val level), bs).

(** [todo] more records after [earlier]. *)
Fixpoint read_records (todo : nat) (count : N) (earlier : list Record) (bs : bytes)
  : Result (list Record * bytes) Failure :=
  match todo with
  | O => ok (earlier, bs)
  | S todo' =>
      let? (r, bs) := read_record count earlier bs in
      read_records todo' count (earlier ++ [r]) bs
  end.

Definition decode_records (input : list (Word 8)) : Result (list Record) Failure :=
  if 4096 <? size input then fail oversized "input" else
  let? rest := read_magic magic input in
  let? (v, rest) := read_byte "header" rest in
  if negb (val v =? version) then fail unknown_version "header" else
  let? (count, rest) := read_uvar "count" rest in
  if 64 <? count then fail oversized "count" else
  let? (l, rest) := read_records (N.to_nat count) count [] rest in
  match rest with
  | [] => ok l
  | _ :: _ => fail trailing_data "trailing"
  end.

(** * Proofs *)

(** Linear arithmetic with division and remainder by constants. *)
Local Ltac arith := zify; Z.quot_rem_to_equations; Z.div_mod_to_equations; lia.

(** ** Bytes *)

Lemma byte_lt (b : Word 8) : val b < 256.
Proof. exact (val_bound 8 b). Qed.

Lemma val_byte (n : N) : n < 256 -> val (byte n) = n.
Proof. intros H. apply N.mod_small. exact H. Qed.

Lemma byte_eq (b : Word 8) (n : N) : val b = n -> b = byte n.
Proof. intros <-. symmetry. apply word_of_nat_val. Qed.

Lemma bind_ok {A B : Type} (r : Result A Failure) (k : A -> Result B Failure) (y : B) :
  bind r k = ok y -> exists x, r = ok x /\ k x = ok y.
Proof. destruct r as [x|e]; simpl; [eauto|discriminate]. Qed.

(** [injection] would normalize the components; this keeps them as they are. *)
Lemma ok_pair {A B : Type} (a c : A) (b d : B) :
  @ok (A * B) Failure (a, b) = ok (c, d) -> a = c /\ b = d.
Proof. intros H. injection H as -> ->. auto. Qed.

(** ** Readers: what they accept is the encoding of what they return
    ([_ok]), and they return what was encoded ([_app]). *)

Lemma split_at_ok (n : nat) (bs chunk rest : bytes) :
  split_at n bs = Some (chunk, rest) -> bs = chunk ++ rest /\ length chunk = n.
Proof.
  revert bs chunk rest; induction n as [|n IH]; intros [|b bs] chunk rest H; simpl in H.
  - injection H as <- <-. auto.
  - injection H as <- <-. auto.
  - discriminate.
  - destruct (split_at n bs) as [[c r]|] eqn:E; [|discriminate].
    injection H as <- <-. destruct (IH _ _ _ E) as [-> <-]. auto.
Qed.

Lemma split_at_app (chunk rest : bytes) :
  split_at (length chunk) (chunk ++ rest) = Some (chunk, rest).
Proof. induction chunk as [|b chunk IH]; simpl; [reflexivity|]. rewrite IH. reflexivity. Qed.

Lemma read_bytes_ok n p bs chunk rest :
  read_bytes n p bs = ok (chunk, rest) -> bs = chunk ++ rest /\ size chunk = n.
Proof.
  unfold read_bytes. destruct (split_at (N.to_nat n) bs) as [[c r]|] eqn:E; [|discriminate].
  intros H. apply ok_pair in H as [<- <-]. apply split_at_ok in E as [-> E].
  split; [reflexivity|]. rewrite E. apply N2Nat.id.
Qed.

Lemma read_bytes_app p chunk rest : read_bytes (size chunk) p (chunk ++ rest) = ok (chunk, rest).
Proof. unfold read_bytes. rewrite Nat2N.id, split_at_app. reflexivity. Qed.

Lemma read_byte_ok p bs b rest : read_byte p bs = ok (b, rest) -> bs = b :: rest.
Proof. destruct bs; simpl; [discriminate|]. intros H. apply ok_pair in H as [<- <-]. reflexivity. Qed.

Lemma read_uvar_ok p bs n rest : read_uvar p bs = ok (n, rest) -> bs = uvar n ++ rest.
Proof.
  destruct bs as [|b0 bs]; cbn [read_uvar]; [discriminate|].
  pose proof (byte_lt b0) as B0.
  destruct (N.ltb_spec (val b0) 128) as [L0|L0].
  - intros H; apply ok_pair in H as [<- <-]. unfold uvar.
    rewrite (proj2 (N.ltb_lt _ _) L0). simpl. f_equal. apply byte_eq. reflexivity.
  - destruct bs as [|b1 bs]; [discriminate|].
    pose proof (byte_lt b1) as B1.
    destruct (N.leb_spec 128 (val b1)) as [|M]; [discriminate|].
    destruct (N.eqb_spec (val b1) 0) as [|Z]; [discriminate|].
    intros H; apply ok_pair in H as [<- <-].
    unfold uvar. destruct (N.ltb_spec (val b0 - 128 + 128 * val b1) 128); [lia|].
    cbn [app]. f_equal; [|f_equal]; apply byte_eq; arith.
Qed.

Lemma read_uvar_uvar p n rest : n < 16384 -> read_uvar p (uvar n ++ rest) = ok (n, rest).
Proof.
  intros Hn. unfold uvar. destruct (N.ltb_spec n 128) as [L|L]; cbn [app read_uvar].
  - rewrite val_byte by lia. rewrite (proj2 (N.ltb_lt _ _) L). reflexivity.
  - rewrite !val_byte by arith.
    replace (128 + n mod 128 <? 128) with false by (symmetry; apply N.ltb_ge; arith).
    replace (128 <=? n / 128) with false by (symmetry; apply N.leb_gt; arith).
    replace (n / 128 =? 0) with false by (symmetry; apply N.eqb_neq; arith).
    do 3 f_equal. arith.
Qed.

Lemma read_magic_ok es bs rest : read_magic es bs = ok rest -> bs = map byte es ++ rest.
Proof.
  revert bs; induction es as [|e es IH]; intros [|b bs] H; simpl in H.
  - injection H as <-. reflexivity.
  - injection H as <-. reflexivity.
  - discriminate.
  - destruct (N.eqb_spec (val b) e); [|discriminate].
    simpl. rewrite (IH _ H). f_equal. apply byte_eq. assumption.
Qed.

Lemma read_magic_magic rest : read_magic magic (map byte magic ++ rest) = ok rest.
Proof. reflexivity. Qed.

Lemma read_name_ok p prev bs name rest :
  read_name p prev bs = ok (name, rest) ->
  bs = uvar (size name) ++ name ++ rest /\ name_ok name = true /\ after prev name = true.
Proof.
  unfold read_name. intros H.
  apply bind_ok in H as [[len bs1] [H1 H]]. cbn beta iota in H.
  destruct ((len =? 0) || (200 <? len)) eqn:L; [discriminate|].
  apply bind_ok in H as [[nm bs2] [H2 H]]. cbn beta iota in H.
  destruct (utf8_valid nm) eqn:U; cbn [negb] in H; [|discriminate].
  apply read_uvar_ok in H1 as ->. apply read_bytes_ok in H2 as [-> S2].
  assert (nm = name /\ bs2 = rest /\ after prev nm = true) as (-> & -> & A).
  { destruct prev as [q|]; [destruct (bytes_compare q nm) eqn:C|];
      try discriminate; apply ok_pair in H as [<- <-]; unfold after, bytes_lt; try rewrite C; auto. }
  rewrite S2. repeat split; auto.
  apply orb_false_iff in L as [L1 L2]. apply N.eqb_neq in L1. apply N.ltb_ge in L2.
  unfold name_ok. rewrite U, S2, andb_true_r.
  apply andb_true_intro; split; apply N.leb_le; lia.
Qed.

Lemma read_name_app p prev name rest :
  name_ok name = true -> after prev name = true ->
  read_name p prev (uvar (size name) ++ name ++ rest) = ok (name, rest).
Proof.
  intros N A. unfold name_ok in N. apply andb_prop in N as [N U]. apply andb_prop in N as [N1 N2].
  apply N.leb_le in N1. apply N.leb_le in N2.
  unfold read_name. rewrite read_uvar_uvar by lia. cbn [bind].
  replace ((size name =? 0) || (200 <? size name)) with false
    by (symmetry; apply orb_false_iff; split; [apply N.eqb_neq | apply N.ltb_ge]; lia).
  rewrite read_bytes_app. cbn [bind]. rewrite U. cbn [negb].
  destruct prev as [q|]; [|reflexivity]. unfold after, bytes_lt in A.
  destruct (bytes_compare q name); [discriminate|reflexivity|discriminate].
Qed.

(** The first reference comes after [previous]. *)
Definition first_after (previous : option N) (refs : list N) : bool :=
  match refs with [] => true | r :: _ => ref_after previous r end.

Lemma read_refs_ok todo count index slot prev bs refs rest :
  read_refs todo count index slot prev bs = ok (refs, rest) ->
  bs = flat_map uvar refs ++ rest /\ length refs = todo /\ forallb (below index) refs = true
  /\ increasing N.ltb refs = true /\ first_after prev refs = true.
Proof.
  revert slot prev bs refs rest.
  induction todo as [|todo IH]; intros slot prev bs refs rest H; cbn [read_refs] in H.
  - apply ok_pair in H as [<- <-]. auto.
  - apply bind_ok in H as [[r bs1] [H1 H]]. cbn beta iota in H.
    destruct (N.leb_spec count r) as [|Rc]; [discriminate|].
    destruct (N.leb_spec index r) as [|Ri]; [discriminate|].
    destruct (ref_after prev r) eqn:A; cbn [negb] in H; [|discriminate].
    apply bind_ok in H as [[rs bs2] [H2 H]]. cbn beta iota in H. apply ok_pair in H as [<- <-].
    apply read_uvar_ok in H1 as ->.
    destruct (IH _ _ _ _ _ H2) as (-> & L & F & I & FA).
    cbn [flat_map length forallb first_after]. rewrite <- app_assoc, L, F, A.
    unfold below. rewrite (proj2 (N.ltb_lt _ _)) by lia.
    repeat split. destruct rs as [|x rs]; [reflexivity|].
    cbn [first_after ref_after] in FA. cbn [increasing]. rewrite FA. exact I.
Qed.

Lemma read_refs_app count index slot prev refs rest :
  index <= count -> count <= 64 ->
  forallb (below index) refs = true -> increasing N.ltb refs = true -> first_after prev refs = true ->
  read_refs (length refs) count index slot prev (flat_map uvar refs ++ rest) = ok (refs, rest).
Proof.
  intros Hi Hc. revert slot prev.
  induction refs as [|r refs IH]; intros slot prev F I A; [reflexivity|].
  cbn [forallb] in F. apply andb_prop in F as [F1 F]. unfold below in F1. apply N.ltb_lt in F1.
  cbn [length flat_map read_refs]. rewrite <- app_assoc, read_uvar_uvar by lia. cbn [bind].
  replace (count <=? r) with false by (symmetry; apply N.leb_gt; lia).
  replace (index <=? r) with false by (symmetry; apply N.leb_gt; lia).
  cbn [first_after] in A. rewrite A. cbn [negb].
  rewrite IH; [reflexivity|exact F| |].
  - destruct refs as [|x refs]; [reflexivity|]. cbn [increasing] in I.
    apply andb_prop in I as [_ I]. exact I.
  - destruct refs as [|x refs]; [reflexivity|]. cbn [increasing] in I.
    apply andb_prop in I as [I _]. exact I.
Qed.

Lemma read_record_ok count earlier bs r rest :
  read_record count earlier bs = ok (r, rest) ->
  bs = encode_record r ++ rest /\ name_ok (name_of r) = true
  /\ after (last_name earlier) (name_of r) = true /\ body_ok earlier r = true.
Proof.
  unfold read_record. cbv zeta. intros H.
  apply bind_ok in H as [[tag bs1] [H1 H]]. cbn beta iota in H.
  apply read_byte_ok in H1 as ->.
  destruct ((val tag =? 0) || (3 <? val tag)) eqn:T; [discriminate|].
  apply orb_false_iff in T as [T0 T3]. apply N.eqb_neq in T0. apply N.ltb_ge in T3.
  apply bind_ok in H as [[name bs2] [H2 H]]. cbn beta iota in H.
  apply read_name_ok in H2 as (-> & NO & AF).
  destruct (N.eqb_spec (val tag) 1) as [T1|T1]; [|destruct (N.eqb_spec (val tag) 2) as [T2|T2]].
  - apply bind_ok in H as [[digest bs3] [H3 H]]. cbn beta iota in H.
    apply ok_pair in H as [<- <-]. apply read_bytes_ok in H3 as [-> S3].
    unfold encode_record, body_ok. cbn [tag_of name_of encode_body].
    cbn [app]. rewrite S3, <- (byte_eq tag 1 T1), <- !app_assoc. auto.
  - apply bind_ok in H as [[fp bs3] [H3 H]]. cbn beta iota in H.
    apply bind_ok in H as [[total bs4] [H4 H]]. cbn beta iota in H.
    destruct (N.ltb_spec 64 total) as [|Tt]; [discriminate|].
    apply bind_ok in H as [[refs bs5] [H5 H]]. cbn beta iota in H.
    apply ok_pair in H as [<- <-].
    apply read_bytes_ok in H3 as [-> S3]. apply read_uvar_ok in H4 as ->.
    apply read_refs_ok in H5 as (-> & L5 & F5 & I5 & _).
    assert (Sr : size refs = total) by (rewrite L5; apply N2Nat.id).
    unfold encode_record, body_ok. cbn [tag_of name_of encode_body app].
    rewrite S3, Sr, F5, I5, <- (byte_eq tag 2 T2), <- !app_assoc.
    repeat split; auto. rewrite N.eqb_refl, (proj2 (N.leb_le _ _) Tt). reflexivity.
  - assert (T3' : val tag = 3) by lia.
    apply bind_ok in H as [[ref bs3] [H3 H]]. cbn beta iota in H.
    destruct (N.leb_spec count ref) as [|Rc]; [discriminate|].
    destruct (N.leb_spec (size earlier) ref) as [|Ri]; [discriminate|].
    destruct (is_theorem (nth_error earlier (N.to_nat ref))) eqn:K; cbn [negb] in H; [|discriminate].
    apply bind_ok in H as [[level bs4] [H4 H]]. cbn beta iota in H.
    destruct (N.ltb_spec 3 (val level)) as [|Lv]; [discriminate|].
    apply ok_pair in H as [<- <-].
    apply read_uvar_ok in H3 as ->. apply read_byte_ok in H4 as ->.
    unfold encode_record, body_ok, below. cbn [tag_of name_of encode_body app].
    rewrite K, <- (byte_eq tag 3 T3'), <- (byte_eq level (val level) eq_refl), <- !app_assoc.
    repeat split; auto.
    rewrite (proj2 (N.ltb_lt _ _) Ri), (proj2 (N.leb_le _ _) Lv). reflexivity.
Qed.

Lemma read_record_app count earlier r rest :
  size earlier < count -> count <= 64 ->
  name_ok (name_of r) = true -> after (last_name earlier) (name_of r) = true ->
  body_ok earlier r = true ->
  read_record count earlier (encode_record r ++ rest) = ok (r, rest).
Proof.
  intros Hi Hc NO AF BO. unfold read_record, encode_record. cbv zeta.
  cbn [app read_byte bind]. rewrite val_byte by (destruct r; cbn; lia).
  destruct r as [name digest|name fp refs|name ref level]; cbn [tag_of name_of encode_body] in *;
    cbn [N.eqb N.ltb orb Pos.eqb N.compare Pos.compare Pos.compare_cont];
    rewrite <- !app_assoc, read_name_app by assumption; cbn [bind]; unfold body_ok in BO.
  - apply N.eqb_eq in BO. rewrite <- BO, read_bytes_app. reflexivity.
  - apply andb_prop in BO as [BO F]. apply andb_prop in BO as [BO I].
    apply andb_prop in BO as [S L]. apply N.eqb_eq in S. apply N.leb_le in L.
    rewrite <- S, read_bytes_app. cbn [bind]. rewrite read_uvar_uvar by lia. cbn [bind].
    replace (64 <? size refs) with false by (symmetry; apply N.ltb_ge; exact L).
    rewrite Nat2N.id, read_refs_app by (lia || assumption || (destruct refs; reflexivity)).
    reflexivity.
  - unfold below in BO. apply andb_prop in BO as [BO Lv]. apply andb_prop in BO as [Ri K].
    apply N.ltb_lt in Ri. apply N.leb_le in Lv.
    rewrite read_uvar_uvar by lia. cbn [bind].
    replace (count <=? ref) with false by (symmetry; apply N.leb_gt; lia).
    replace (size earlier <=? ref) with false by (symmetry; apply N.leb_gt; lia).
    rewrite K. cbn [negb app read_byte bind]. rewrite val_byte by lia.
    replace (3 <? level) with false by (symmetry; apply N.ltb_ge; lia).
    reflexivity.
Qed.

(** ** The record loop *)

(** What the decoder checks of the records so far: good names in strictly
    increasing order and good bodies. *)
Definition prefix_ok (l : list Record) : bool :=
  forallb name_ok (map name_of l) && increasing bytes_lt (map name_of l) && bodies_ok [] l.

Lemma increasing_app_l {A : Type} (lt : A -> A -> bool) (xs ys : list A) :
  increasing lt (xs ++ ys) = true -> increasing lt xs = true.
Proof.
  induction xs as [|a xs IH]; [reflexivity|]. destruct xs as [|b xs]; [reflexivity|].
  cbn [app increasing]. intros H. apply andb_prop in H as [H1 H2]. rewrite H1. apply IH. exact H2.
Qed.

Lemma increasing_snoc {A : Type} (lt : A -> A -> bool) (xs : list A) (x : A) :
  increasing lt (xs ++ [x])
  = increasing lt xs && match last (map Some xs) None with None => true | Some y => lt y x end.
Proof.
  induction xs as [|a xs IH]; [reflexivity|].
  destruct xs as [|b xs]; cbn [app increasing map last].
  - rewrite andb_true_r. reflexivity.
  - cbn [app increasing map last] in IH. rewrite IH, andb_assoc. reflexivity.
Qed.

Lemma bodies_ok_app (pre xs ys : list Record) :
  bodies_ok pre (xs ++ ys) = bodies_ok pre xs && bodies_ok (pre ++ xs) ys.
Proof.
  revert pre; induction xs as [|x xs IH]; intros pre; cbn [app bodies_ok].
  - rewrite app_nil_r. reflexivity.
  - rewrite IH, <- app_assoc, andb_assoc. reflexivity.
Qed.

Lemma prefix_ok_snoc earlier r :
  prefix_ok (earlier ++ [r]) = true <->
  prefix_ok earlier = true /\ name_ok (name_of r) = true
  /\ after (last_name earlier) (name_of r) = true /\ body_ok earlier r = true.
Proof.
  unfold prefix_ok, last_name, after. rewrite !map_app. cbn [map].
  rewrite forallb_app, increasing_snoc, bodies_ok_app, map_map. cbn [forallb bodies_ok].
  rewrite !andb_true_r, !andb_true_iff. tauto.
Qed.

Lemma prefix_ok_app_l xs ys : prefix_ok (xs ++ ys) = true -> prefix_ok xs = true.
Proof.
  unfold prefix_ok. rewrite !map_app, forallb_app, bodies_ok_app, !andb_true_iff.
  intros [[[F _] I] [B _]]. apply increasing_app_l in I. auto.
Qed.

Lemma read_records_ok todo count earlier bs l rest :
  read_records todo count earlier bs = ok (l, rest) ->
  exists new, l = earlier ++ new /\ length new = todo /\ bs = flat_map encode_record new ++ rest
    /\ (prefix_ok earlier = true -> prefix_ok l = true).
Proof.
  revert earlier bs; induction todo as [|todo IH]; intros earlier bs H; cbn [read_records] in H.
  - apply ok_pair in H as [<- <-]. exists []. rewrite app_nil_r. auto.
  - apply bind_ok in H as [[r bs1] [H1 H]]. cbn beta iota in H.
    destruct (IH _ _ H) as (new & -> & L & -> & P).
    apply read_record_ok in H1 as (-> & NO & AF & BO).
    exists (r :: new). rewrite <- app_assoc. cbn [length flat_map]. repeat split.
    + rewrite L. reflexivity.
    + rewrite app_assoc. reflexivity.
    + intros P0. rewrite app_assoc. apply P, prefix_ok_snoc. auto.
Qed.

Lemma read_records_app count earlier new rest :
  prefix_ok (earlier ++ new) = true -> size (earlier ++ new) = count -> count <= 64 ->
  read_records (length new) count earlier (flat_map encode_record new ++ rest)
  = ok (earlier ++ new, rest).
Proof.
  revert earlier; induction new as [|r new IH]; intros earlier P C Hc; cbn [length read_records].
  - rewrite app_nil_r. reflexivity.
  - cbn [flat_map]. rewrite <- app_assoc.
    replace (earlier ++ r :: new) with ((earlier ++ [r]) ++ new) in *
      by (rewrite <- app_assoc; reflexivity).
    pose proof (prefix_ok_app_l _ _ P) as P1. apply prefix_ok_snoc in P1 as (_ & NO & AF & BO).
    assert (Hi : size earlier < count) by (rewrite !length_app in C; cbn [length] in C; lia).
    rewrite read_record_app by assumption. cbn [bind]. apply IH; assumption.
Qed.

(** ** The theorems D3-TH01 to D3-TH05 *)

Lemma decode_ok b l :
  decode_records b = ok l ->
  b = encode_records l /\ size b <= 4096 /\ size l <= 64 /\ prefix_ok l = true.
Proof.
  unfold decode_records. intros H.
  destruct (N.ltb_spec 4096 (size b)) as [|Hb]; [discriminate|].
  apply bind_ok in H as [bs1 [H1 H]]. cbn beta in H.
  apply bind_ok in H as [[v bs2] [H2 H]]. cbn beta iota in H.
  destruct (N.eqb_spec (val v) version) as [V|]; cbn [negb] in H; [|discriminate].
  apply bind_ok in H as [[count bs3] [H3 H]]. cbn beta iota in H.
  destruct (N.ltb_spec 64 count) as [|Hc]; [discriminate|].
  apply bind_ok in H as [[l' bs4] [H4 H]]. cbn beta iota in H.
  destruct bs4 as [|x bs4]; [|discriminate]. injection H as <-.
  apply read_magic_ok in H1 as ->. apply read_byte_ok in H2 as ->. apply read_uvar_ok in H3 as ->.
  apply read_records_ok in H4 as (new & -> & L & -> & P). cbn [app] in *.
  assert (Sn : size new = count) by (rewrite L; apply N2Nat.id).
  unfold encode_records. rewrite <- (byte_eq v version V), Sn, app_nil_r.
  repeat split; auto. rewrite app_nil_r in Hb. exact Hb.
Qed.

Theorem encoding_round_trips : forall (l : list Record),
  records_valid l = true -> decode_records (encode_records l) = ok l.
Proof.
  intros l V. unfold records_valid in V. rewrite !andb_true_iff in V.
  destruct V as [[[[Hl F] I] B] E]. apply N.leb_le in Hl. apply N.leb_le in E.
  unfold decode_records.
  replace (4096 <? size (encode_records l)) with false by (symmetry; apply N.ltb_ge; exact E).
  unfold encode_records. rewrite read_magic_magic. cbn [bind read_byte].
  rewrite val_byte by (unfold version; lia). rewrite N.eqb_refl. cbn [negb].
  rewrite read_uvar_uvar by lia. cbn [bind].
  replace (64 <? size l) with false by (symmetry; apply N.ltb_ge; exact Hl).
  rewrite Nat2N.id, <- (app_nil_r (flat_map encode_record l)).
  pose proof (read_records_app (size l) [] l [])  as R. cbn [app] in R.
  rewrite R; [reflexivity| |reflexivity|exact Hl].
  unfold prefix_ok. rewrite F, I, B. reflexivity.
Qed.

Theorem accepted_bytes_are_canonical : forall (b : list (Word 8)) (l : list Record),
  decode_records b = ok l -> encode_records l = b.
Proof. intros b l H. apply decode_ok in H as (-> & _). reflexivity. Qed.

Theorem accepted_values_are_valid : forall (b : list (Word 8)) (l : list Record),
  decode_records b = ok l -> records_valid l = true.
Proof.
  intros b l H. apply decode_ok in H as (-> & Hb & Hl & P).
  unfold prefix_ok in P. rewrite !andb_true_iff in P. destruct P as [[F I] B].
  unfold records_valid. rewrite F, I, B, (proj2 (N.leb_le _ _) Hl), (proj2 (N.leb_le _ _) Hb).
  reflexivity.
Qed.

Theorem accepted_inputs_are_bounded : forall (b : list (Word 8)) (l : list Record),
  decode_records b = ok l -> list_length (Word 8) b <= 4096.
Proof. intros b l H. apply decode_ok in H as (_ & H & _). exact H. Qed.

Theorem one_value_has_one_encoding : forall (b1 b2 : list (Word 8)) (l : list Record),
  decode_records b1 = ok l -> decode_records b2 = ok l -> b1 = b2.
Proof.
  intros b1 b2 l H1 H2.
  rewrite <- (accepted_bytes_are_canonical _ _ H1). exact (accepted_bytes_are_canonical _ _ H2).
Qed.
