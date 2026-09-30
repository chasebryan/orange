/-!
# D-006 v0.3, candidate C-02 (Lean 4): the DS-01 Core fragment

Every shared T-, C- and F- symbol of DS-01 maps to one declaration here or in
Lean core (see `adapter.json`). Words are core's `BitVec`, sequences are core's
length-indexed `Vector`, and every function is structurally recursive, so a
closed observation evaluates under `decide`.
-/

namespace D006

/-! ## Exact words (T-03, F-01 to F-08) -/

/-- T-03 `Word w`: exact bit vectors of width `w`. Core's `BitVec w` wraps a
`Fin (2 ^ w)`, so it has exactly one value for each natural below `2 ^ w`. -/
abbrev Word (w : Nat) : Type := BitVec w

/-- F-01: `n mod 2 ^ w` as a word. -/
def word_of_nat (w n : Nat) : Word w := BitVec.ofNat w n

/-- F-02: the natural a word denotes. -/
def nat_of_word {w : Nat} (x : Word w) : Nat := x.toNat

/-- F-03: addition mod `2 ^ w`. -/
def word_add {w : Nat} (x y : Word w) : Word w := x + y

/-- F-04: bitwise exclusive or. -/
def word_xor {w : Nat} (x y : Word w) : Word w := x ^^^ y

/-- F-05: bitwise and. -/
def word_and {w : Nat} (x y : Word w) : Word w := x &&& y

/-- F-06: bitwise complement, `(2 ^ w - 1) - x` (see `nat_of_word_not`). -/
def word_not {w : Nat} (x : Word w) : Word w := ~~~x

/-- F-07: left rotation by `r mod w`. Core's `rotateLeft` reduces the amount
mod `w` itself; at width 0 there is only one word, so it is the identity. -/
def word_rotl {w : Nat} (x : Word w) (r : Nat) : Word w := x.rotateLeft r

/-- F-08: left shift, `(x * 2 ^ r) mod 2 ^ w` (see `nat_of_word_shl`). -/
def word_shl {w : Nat} (x : Word w) (r : Nat) : Word w := x <<< r

/-! The word operations are the arithmetic semantics.md states. -/

theorem nat_of_word_of_nat (w n : Nat) : nat_of_word (word_of_nat w n) = n % 2 ^ w :=
  BitVec.toNat_ofNat n w

theorem nat_of_word_add {w : Nat} (x y : Word w) :
    nat_of_word (word_add x y) = (nat_of_word x + nat_of_word y) % 2 ^ w :=
  BitVec.toNat_add x y

theorem nat_of_word_not {w : Nat} (x : Word w) :
    nat_of_word (word_not x) = 2 ^ w - 1 - nat_of_word x :=
  BitVec.toNat_not

theorem nat_of_word_shl {w : Nat} (x : Word w) (r : Nat) :
    nat_of_word (word_shl x r) = nat_of_word x * 2 ^ r % 2 ^ w := by
  simp [nat_of_word, word_shl, Nat.shiftLeft_eq]

theorem word_rotl_mod {w : Nat} (x : Word w) (r : Nat) : word_rotl x (r % w) = word_rotl x r :=
  BitVec.rotateLeft_mod_eq_rotateLeft

theorem word_rotl_width_zero (x : Word 0) (r : Nat) : word_rotl x r = x :=
  Subsingleton.elim _ _

/-- Bit `i` from the top of `rotl x r` is bit `(r + i) mod w` from the top of `x`. -/
theorem getMsbD_word_rotl {w : Nat} (x : Word w) (r i : Nat) :
    (word_rotl x r).getMsbD i = (decide (i < w) && x.getMsbD ((r + i) % w)) :=
  BitVec.getMsbD_rotateLeft

/-! ## Length-indexed sequences (T-04, F-13, F-17, F-20) -/

/-- T-04 `Seq A n`: sequences of exactly `n` elements, core's `Vector`. -/
abbrev Seq (A : Type) (n : Nat) : Type := Vector A n

/-- F-17: the four-element sequence of its arguments in order. -/
def seq4 {A : Type} (a b c d : A) : Seq A 4 := #v[a, b, c, d]

/-- F-20: the elements in order. -/
def seq_to_list {A : Type} {n : Nat} (s : Seq A n) : List A := s.toList

/-- F-13: pointwise combination of two sequences of the same length. It zips
the element lists (structural recursion) rather than calling `Vector.zipWith`,
whose array loop is well-founded recursion: the elaborator does not unfold it
(`decide` and `rfl` get stuck), and only the kernel can, through its
accessibility proof. -/
def seq_map2 {A : Type} {n : Nat} (f : A → A → A) (s t : Seq A n) : Seq A n :=
  ⟨(List.zipWith f s.toList t.toList).toArray, by simp⟩

/-! ## Endian conversion (F-09 to F-12) -/

/-- The word whose big-endian bytes are `a b c d`: `((a·256 + b)·256 + c)·256 + d`. -/
def word32 (a b c d : Word 8) : Word 32 :=
  BitVec.ofNat 32 (((a.toNat * 256 + b.toNat) * 256 + c.toNat) * 256 + d.toNat)

/-- Byte `k` of a 32-bit word, counting from the least significant byte. -/
def byte (x : Word 32) (k : Nat) : Word 8 := x.extractLsb' (8 * k) 8

/-- F-09: big-endian, the first byte is most significant. -/
def be32_of_bytes (s : Seq (Word 8) 4) : Word 32 := word32 s[0] s[1] s[2] s[3]

/-- F-10: inverse of F-09. -/
def bytes_of_be32 (x : Word 32) : Seq (Word 8) 4 := #v[byte x 3, byte x 2, byte x 1, byte x 0]

/-- F-11: little-endian, the first byte is least significant. -/
def le32_of_bytes (s : Seq (Word 8) 4) : Word 32 := word32 s[3] s[2] s[1] s[0]

/-- F-12: inverse of F-11. -/
def bytes_of_le32 (x : Word 32) : Seq (Word 8) 4 := #v[byte x 0, byte x 1, byte x 2, byte x 3]

/-! ## Results, the decoder and the encoder (T-06, T-07, C-01 to C-05, F-14, F-15) -/

/-- T-06: success with an `A` (C-04 `ok`) or typed failure with an `E` (C-05 `err`). -/
inductive Result (A E : Type) : Type where
  | ok : A → Result A E
  | err : E → Result A E
  deriving DecidableEq, Repr

/-- T-07: the decoder's failures. -/
inductive DecodeError : Type where
  /-- C-01: the count byte exceeds 16. -/
  | too_many
  /-- C-02: the input ends before the counted words. -/
  | truncated
  /-- C-03: bytes follow the counted words. -/
  | trailing
  deriving DecidableEq, Repr

/-- Reads consecutive groups of four bytes as big-endian words (a shorter tail is dropped). -/
def group4 : List (Word 8) → List (Word 32)
  | a :: b :: c :: d :: rest => word32 a b c d :: group4 rest
  | _ => []

/-- F-14: a count byte `n` at most 16, then exactly `4 n` bytes read as `n` big-endian words. -/
def decode_words : List (Word 8) → Result (List (Word 32)) DecodeError
  | [] => .err .truncated
  | count :: rest =>
    let n := count.toNat
    if 16 < n then .err .too_many
    else if rest.length < 4 * n then .err .truncated
    else if 4 * n < rest.length then .err .trailing
    else .ok (group4 rest)

/-- F-15: the count (mod 256) as one byte, then each word big-endian. -/
def encode_words (l : List (Word 32)) : List (Word 8) :=
  word_of_nat 8 l.length :: l.flatMap fun x => seq_to_list (bytes_of_be32 x)

/-! F-16 is `List.length`, F-21 is `Nat.pow` and F-22 is `List.append`. -/

/-! ## The parameterized quarter round (M-01, F-18, F-19) -/

/-- M-01 `QuarterRound`, defined once over its parameters: the width `w` and the
four rotation amounts. -/
def quarterRound (w r1 r2 r3 r4 : Nat) (s : Seq (Word w) 4) : Seq (Word w) 4 :=
  let a := s[0]; let b := s[1]; let c := s[2]; let d := s[3]
  let a := word_add a b; let d := word_rotl (word_xor d a) r1
  let c := word_add c d; let b := word_rotl (word_xor b c) r2
  let a := word_add a b; let d := word_rotl (word_xor d a) r3
  let c := word_add c d; let b := word_rotl (word_xor b c) r4
  seq4 a b c d

/-- F-18: M-01 at width 32 with rotations 16, 12, 8, 7 (the ChaCha quarter round). -/
def chacha_qr : Seq (Word 32) 4 → Seq (Word 32) 4 := quarterRound 32 16 12 8 7

/-- F-19: M-01 at width 8 with rotations 4, 3, 2, 1. -/
def toy_qr : Seq (Word 8) 4 → Seq (Word 8) 4 := quarterRound 8 4 3 2 1

/-! ## Theorems D1-TH01 to D1-TH12 -/

/-- D1-TH01: word values are bounded. -/
theorem word_bounded (w : Nat) (x : Word w) : nat_of_word x < 2 ^ w :=
  x.isLt

/-- D1-TH02: small naturals are words. -/
theorem small_naturals_are_words (w n : Nat) (h : n < 2 ^ w) :
    nat_of_word (word_of_nat w n) = n := by
  rw [nat_of_word_of_nat, Nat.mod_eq_of_lt h]

/-- D1-TH03: words are their values. -/
theorem words_are_their_values (w : Nat) (x y : Word w) (h : nat_of_word x = nat_of_word y) :
    x = y :=
  BitVec.eq_of_toNat_eq h

/-- D1-TH04: addition commutes. -/
theorem addition_commutes (w : Nat) (x y : Word w) : word_add x y = word_add y x :=
  BitVec.add_comm x y

/-- D1-TH05: exclusive or cancels. -/
theorem xor_cancels (w : Nat) (x : Word w) : word_xor x x = word_of_nat w 0 :=
  BitVec.xor_self

/-- D1-TH06: rotations compose. -/
theorem rotations_compose (w : Nat) (x : Word w) (r s : Nat) :
    word_rotl (word_rotl x r) s = word_rotl x (r + s) := by
  apply BitVec.eq_of_getMsbD_eq
  intro i hi
  have hw : 0 < w := by omega
  simp only [getMsbD_word_rotl, hi, Nat.mod_lt _ hw, decide_true, Bool.true_and]
  congr 1
  rw [Nat.add_mod_mod, ← Nat.add_assoc]

/-! Byte round trips -/

theorem toNat_byte (x : Word 32) (k : Nat) : (byte x k).toNat = x.toNat / 2 ^ (8 * k) % 256 := by
  simp [byte, Nat.shiftRight_eq_div_pow]

theorem toNat_word32 (a b c d : Word 8) :
    (word32 a b c d).toNat = ((a.toNat * 256 + b.toNat) * 256 + c.toNat) * 256 + d.toNat := by
  have := a.isLt; have := b.isLt; have := c.isLt; have := d.isLt
  simp only [word32, BitVec.toNat_ofNat]
  omega

theorem word32_bytes (x : Word 32) : word32 (byte x 3) (byte x 2) (byte x 1) (byte x 0) = x := by
  apply BitVec.eq_of_toNat_eq
  have := x.isLt
  simp only [toNat_word32, toNat_byte]
  omega

theorem byte_word32 (a b c d : Word 8) :
    byte (word32 a b c d) 3 = a ∧ byte (word32 a b c d) 2 = b ∧
    byte (word32 a b c d) 1 = c ∧ byte (word32 a b c d) 0 = d := by
  have := a.isLt; have := b.isLt; have := c.isLt; have := d.isLt
  refine ⟨?_, ?_, ?_, ?_⟩ <;> apply BitVec.eq_of_toNat_eq <;>
    simp only [toNat_byte, toNat_word32] <;> omega

theorem seq4_cases {A : Type} (s : Seq A 4) : ∃ a b c d, s = seq4 a b c d := by
  refine ⟨s[0], s[1], s[2], s[3], Vector.ext fun i hi => ?_⟩
  match i, hi with
  | 0, _ | 1, _ | 2, _ | 3, _ => rfl

theorem seq_to_list_bytes_of_be32 (x : Word 32) :
    seq_to_list (bytes_of_be32 x) = [byte x 3, byte x 2, byte x 1, byte x 0] := rfl

/-- D1-TH07: big-endian word round trip. -/
theorem be32_word_round_trip (x : Word 32) : be32_of_bytes (bytes_of_be32 x) = x :=
  word32_bytes x

/-- D1-TH08: big-endian bytes round trip. -/
theorem be32_bytes_round_trip (s : Seq (Word 8) 4) : bytes_of_be32 (be32_of_bytes s) = s := by
  obtain ⟨a, b, c, d, rfl⟩ := seq4_cases s
  obtain ⟨e3, e2, e1, e0⟩ := byte_word32 a b c d
  show #v[byte (word32 a b c d) 3, byte (word32 a b c d) 2,
      byte (word32 a b c d) 1, byte (word32 a b c d) 0] = seq4 a b c d
  rw [e3, e2, e1, e0]
  rfl

/-- D1-TH09: little-endian word round trip. -/
theorem le32_word_round_trip (x : Word 32) : le32_of_bytes (bytes_of_le32 x) = x :=
  word32_bytes x

/-! Decoder and encoder -/

theorem group4_flatMap (l : List (Word 32)) :
    group4 (l.flatMap fun x => seq_to_list (bytes_of_be32 x)) = l := by
  induction l with
  | nil => rfl
  | cons x l ih =>
    rw [List.flatMap_cons, seq_to_list_bytes_of_be32]
    simp only [List.cons_append, List.nil_append, group4, word32_bytes, ih]

theorem length_flatMap_bytes (l : List (Word 32)) :
    (l.flatMap fun x => seq_to_list (bytes_of_be32 x)).length = 4 * l.length := by
  induction l with
  | nil => rfl
  | cons x l ih =>
    rw [List.flatMap_cons, seq_to_list_bytes_of_be32, List.length_append, ih]
    simp only [List.length_cons, List.length_nil]
    omega

/-- D1-TH10: decoder inverts the encoder. -/
theorem decoder_inverts_encoder (l : List (Word 32)) (h : l.length ≤ 16) :
    decode_words (encode_words l) = .ok l := by
  have hc : (word_of_nat 8 l.length).toNat = l.length := by
    rw [← nat_of_word, nat_of_word_of_nat]; omega
  simp only [encode_words, decode_words, hc, length_flatMap_bytes, group4_flatMap]
  simp [Nat.not_lt.mpr h]

/-- Splitting `4 n` bytes into words and back gives `n` words and the same bytes. -/
theorem group4_spec (n : Nat) (rest : List (Word 8)) (h : rest.length = 4 * n) :
    (group4 rest).length = n ∧
      ((group4 rest).flatMap fun x => seq_to_list (bytes_of_be32 x)) = rest := by
  induction n generalizing rest with
  | zero => cases rest with
    | nil => exact ⟨rfl, rfl⟩
    | cons _ _ => simp at h
  | succ n ih =>
    match rest, h with
    | a :: b :: c :: d :: rest, h =>
      obtain ⟨h1, h2⟩ := ih rest (by simp at h; omega)
      obtain ⟨e3, e2, e1, e0⟩ := byte_word32 a b c d
      refine ⟨by simp [group4, h1], ?_⟩
      rw [group4, List.flatMap_cons, seq_to_list_bytes_of_be32, h2, e3, e2, e1, e0]
      rfl

/-- D1-TH11: decoding is canonical. -/
theorem decoding_is_canonical (b : List (Word 8)) (l : List (Word 32))
    (h : decode_words b = .ok l) : encode_words l = b := by
  match b, h with
  | count :: rest, h =>
    simp only [decode_words] at h
    split at h
    · cases h
    split at h
    · cases h
    split at h
    · cases h
    cases h
    obtain ⟨h1, h2⟩ := group4_spec count.toNat rest (by omega)
    simp only [encode_words, h1, h2]
    congr
    apply BitVec.eq_of_toNat_eq
    rw [← nat_of_word, nat_of_word_of_nat]
    exact Nat.mod_eq_of_lt count.isLt

/-- D1-TH12: sequences have their index length. -/
theorem sequences_have_their_index_length (A : Type) (n : Nat) (s : Seq A n) :
    (seq_to_list s).length = n :=
  s.length_toList

end D006
