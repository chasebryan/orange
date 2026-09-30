(* D-006 v0.3, candidate C-01 (Rocq): DS-05. Rocq's 63-bit primitive
   integers for the extracted checker.

   The extraction mappings Rocq ships (ExtrOCamlInt63, ExtrOCamlPString) name
   the kernel's module Uint63, and the kernel's pstring.ml, compiled from the
   toolchain unchanged, uses it too. This file is the part of that module
   (Rocq 9.2.0, lib/rocq-runtime/kernel/uint63.ml, from kernel/uint63_63.ml)
   that the extracted code and pstring.ml use, each definition copied
   unchanged. The rest is left out, notably the float conversion, an
   external whose C stub lives in the Rocq VM's library, which neither the
   bytecode runtime nor this program has. *)

type t = int

let _ = assert (Sys.word_size = 64)

let of_int i = i
[@@ocaml.inline always]

let zero = 0

let l_and x y = x land y
[@@ocaml.inline always]

    (* addition of int63 *)
let add x y = x + y
[@@ocaml.inline always]

    (* comparison *)
let lt (x : int) (y : int) =
  (x lxor 0x4000000000000000) < (y lxor 0x4000000000000000)
[@@ocaml.inline always]

let le (x : int) (y : int) =
  (x lxor 0x4000000000000000) <= (y lxor 0x4000000000000000)
[@@ocaml.inline always]

let to_int_min n m =
  if lt n m then n else m
[@@ocaml.inline always]

let equal (x : int) (y : int) = x = y
[@@ocaml.inline always]
