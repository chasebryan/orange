(** D-006 v0.3, candidate C-01 (Rocq): DS-05, the standalone checker.

    The standalone checker is these definitions and those of DS-03 and
    DS-04, extracted to OCaml by [Extract.v]. Each entry point takes and
    returns primitive strings, so the hand-written driver ([driver.ml])
    passes file contents and prints results unchanged:

    - [records_line]: the verdict line of semantics.md, section 4, for
      [decode_records] of the input's bytes;
    - [cnf_text] (DS-04), the canonical CNF text of an obligation;
    - [lrat_line]: [lrat_verdict] (DS-04) as [accept] or
      [reject CODE LINE].

    These are the few definitions between the verified functions and the
    command line, extracted with them; the only fact proved here is that
    decoding a prefix of the input gives the verdict of the whole
    ([decode_prefix]). *)

From D006 Require Import Core Records Lrat.
From Stdlib Require Import PeanoNat NArith List Lia.
From Stdlib Require PrimString.
From Corelib Require PrimInt63.
Import ListNotations PrimString.PStringNotations.

Local Open Scope N_scope.
Local Open Scope pstring_scope.

#[local] Arguments ok {A E} _.
#[local] Arguments err {A E} _.

Local Abbreviation string := PrimString.string.
Local Abbreviation int := PrimInt63.int.

Definition concat (l : list string) : string := fold_right PrimString.cat "" l.

(** * The bytes of a primitive string *)

(** A search tree over the 256 byte values keyed by primitive integers:
    [Bnode mid below above] holds the bytes below [mid] in [below]. *)
Inductive ByteTree : Type :=
| Bleaf (b : Word 8)
| Bnode (mid : int) (below above : ByteTree).

(** The bytes [lo] to [lo + 2 ^ d - 1]; [lo_i] is [lo] as a primitive
    integer. *)
Fixpoint byte_tree (d : nat) (lo : N) (lo_i : int) : ByteTree :=
  match d with
  | O => Bleaf (word_of_nat 8 lo)
  | S d' =>
      let half := Nat.iter d' xO xH in
      let mid := PrimInt63.add lo_i (Chars.int_of_pos half) in
      Bnode mid (byte_tree d' lo lo_i) (byte_tree d' (lo + Npos half) mid)
  end.

(** Built once; every byte of an input is one of its leaves. *)
Definition all_bytes : ByteTree := byte_tree 8 0 Chars.start.

Fixpoint find_byte (t : ByteTree) (c : int) : Word 8 :=
  match t with
  | Bleaf b => b
  | Bnode mid below above => if PrimInt63.ltb c mid then find_byte below c else find_byte above c
  end.

(** Up to [2 ^ depth] steps from the state [(i, acc)]; each step puts the
    byte at [i] on [acc] (so [acc] is reversed) or finds the end of the
    string ([inr]). *)
Fixpoint scan (depth : nat) (s : string) (len : int) (st : int * list (Word 8))
  : (int * list (Word 8)) + list (Word 8) :=
  match depth with
  | O =>
      let (i, acc) := st in
      if PrimInt63.leb len i then inr acc
      else inl (PrimInt63.add i Chars.one, find_byte all_bytes (PrimString.get s i) :: acc)
  | S d =>
      match scan d s len st with
      | inl st' => scan d s len st'
      | inr acc => inr acc
      end
  end.

(** The bytes of [s] in order. A primitive string has at most
    [PrimString.max_length] (16777211) bytes, so the end is found within
    [2 ^ 24] steps and the [inl] branch is never taken. *)
Definition bytes_of (s : string) : list (Word 8) :=
  match scan 24 s (PrimString.length s) (Chars.start, []) with
  | inr acc | inl (_, acc) => rev_append acc []
  end.

(** * The records verdict line (semantics.md, section 4) *)

Definition hex_digit (d : N) : string :=
  nth (N.to_nat d) ["0"; "1"; "2"; "3"; "4"; "5"; "6"; "7";
                    "8"; "9"; "a"; "b"; "c"; "d"; "e"; "f"] "?".

(** Bytes in lowercase hexadecimal, two digits each. *)
Definition hex (bs : list (Word 8)) : string :=
  concat (map (fun b => PrimString.cat (hex_digit (val b / 16)) (hex_digit (val b mod 16))) bs).

(** Decimals separated by commas. *)
Fixpoint decimals (l : list N) : string :=
  match l with
  | [] => ""
  | [n] => Records.decimal n
  | n :: rest => concat [Records.decimal n; ","; decimals rest]
  end.

Definition record_field (r : Record) : string :=
  match r with
  | rdef name digest => concat ["def:"; hex name; ":"; hex digest]
  | rthm name fingerprint refs => concat ["thm:"; hex name; ":"; hex fingerprint; ":"; decimals refs]
  | rclaim name ref level => concat ["claim:"; hex name; ":"; Records.decimal ref; ":"; Records.decimal level]
  end.

Definition code_name (c : ErrorCode) : string :=
  match c with
  | oversized => "oversized"
  | bad_magic => "bad_magic"
  | unknown_version => "unknown_version"
  | Records.truncated => "truncated"
  | malformed_number => "malformed_number"
  | noncanonical_number => "noncanonical_number"
  | unknown_field => "unknown_field"
  | invalid_name => "invalid_name"
  | invalid_utf8 => "invalid_utf8"
  | duplicate_name => "duplicate_name"
  | noncanonical_order => "noncanonical_order"
  | reference_escape => "reference_escape"
  | cyclic_reference => "cyclic_reference"
  | reference_kind => "reference_kind"
  | invalid_level => "invalid_level"
  | trailing_data => "trailing_data"
  end.

(** [accept] and one field per record, or [reject CODE PATH]. *)
Definition records_verdict (r : Result (list Record) Failure) : string :=
  match r with
  | ok l => concat ("accept" :: map (fun x => PrimString.cat " " (record_field x)) l)
  | err (failure code path) => concat ["reject "; code_name code; " "; path]
  end.

(** [decode_records] refuses more than 4096 bytes before it reads any, so
    its verdict depends only on the first 4097 bytes. The checker decodes
    that prefix: the extracted list functions are not tail recursive, and
    measuring a long input whole would exhaust the OCaml stack. *)
Definition prefix_length : nat := N.to_nat 4097.

Lemma decode_prefix (l : list (Word 8)) :
  decode_records (firstn prefix_length l) = decode_records l.
Proof.
  change prefix_length with 4097%nat.
  destruct (Nat.le_gt_cases (length l) 4097) as [H | H].
  - now rewrite firstn_all2.
  - unfold decode_records.
    rewrite firstn_length_le by lia.
    replace (4096 <? N.of_nat (length l)) with true; [reflexivity |].
    symmetry; apply N.ltb_lt; lia.
Qed.

Definition records_line (input : string) : string :=
  records_verdict (decode_records (firstn prefix_length (bytes_of input))).

(** * The certificate verdict line *)

Definition verdict_line (v : Verdict) : string :=
  match v with
  | accept => "accept"
  | reject code line => concat ["reject "; code; " "; Lrat.decimal line]
  end.

Definition lrat_line (o : Obligation) (cnf certificate : string) : string :=
  verdict_line (lrat_verdict o cnf certificate).
