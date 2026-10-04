import Lean.Replay

/-!
# An independent check of one compiled module (`.olean`)

`lean --run Loader/OleanCheck.lean FILE` checks the compiled module in FILE
in two stages and exits with 0 only when both pass. The adapter uses it for
D2-M05, where the compiled module holding D2-TH03 is cut in half.

1. Structure. Lean maps an `.olean` into memory and uses the object graph in
   it as it is, without bounds checks, so a damaged file makes `lean` and
   `leanchecker` die with a segmentation fault and no message. This stage
   reads the file as bytes and checks what loading relies on: the header
   (the `olean` marker, and the Lean version and commit that wrote it, which
   must be this toolchain's); that the objects after the header tile the
   rest of the file exactly, each object's size (from its header, laid out
   as in `lean.h`) ending inside the file; and that the root pointer and
   every pointer field of every object reachable from it point at the start
   of an object in the file.
2. Kernel. Only a structurally sound file is then loaded, the way
   `leanchecker` loads one, and every declaration in it is replayed through
   the kernel against its imports.

Every failure is one line on standard output that starts with `error:` and
names the file: a "malformed compiled module" or an "incompatible compiled
module" from stage 1, a "kernel check failed" from stage 2.

This file is a tool, not part of the library: no module imports it, and
`lean --run` elaborates it from source each time it runs.
-/

open Lean

namespace OleanCheck

/-- The header: the marker `olean` (5 bytes), a format version (1), flags
(1), the Lean version (33, NUL-padded), the commit (40) and the address the
file was written for (8); then the compacted region, which starts with the
root object's address (8) and continues with the objects. -/
def headerSize : Nat := 96

def u8 (b : ByteArray) (i : Nat) : Nat := (b.get! i).toNat

/-- The little-endian unsigned integer in the `n` bytes at offset `i`. -/
def uint (b : ByteArray) (i n : Nat) : Nat := Id.run do
  let mut v := 0
  for k in [0:n] do
    v := v ||| (u8 b (i + k) <<< (8 * k))
  return v

def align8 (n : Nat) : Nat := (n + 7) / 8 * 8

/-- The NUL-padded ASCII text in the `n` bytes at offset `i`. -/
def text (b : ByteArray) (i n : Nat) : String :=
  String.ofList ((List.range n).map (fun k => Char.ofNat (u8 b (i + k))) |>.takeWhile (· ≠ '\x00'))

/-- What is wrong with a file that fails stage 1. -/
inductive Defect where
  | malformed (why : String)
  | incompatible (why : String)

/-- One object: its size in bytes and the file offsets of its pointer fields. -/
structure Obj where
  size : Nat
  fields : Array Nat

/-- Reads the object at file offset `o` (layouts as in `lean.h`). -/
def readObj (b : ByteArray) (o : Nat) : Except Defect Obj := do
  let fits (n : Nat) : Except Defect Unit :=
    if o + n ≤ b.size then pure ()
    else throw (.malformed s!"the object at offset {o} needs {n} bytes, but the file ends at {b.size}")
  fits 8
  let csSz := uint b (o + 4) 2
  let other := u8 b (o + 6)
  let tag := u8 b (o + 7)
  let obj ← if tag ≤ 244 then
      -- constructor: `other` pointer fields, then scalars; `cs_sz` is its size
      if csSz < 8 + 8 * other then
        throw (.malformed s!"the constructor at offset {o} has size {csSz} but {other} fields")
      else pure ⟨csSz, (Array.range other).map (o + 8 + 8 * ·)⟩
    else if tag == 246 then do
      -- array: size, capacity, then `size` pointers
      fits 24
      let sz := uint b (o + 8) 8
      let cap := uint b (o + 16) 8
      if sz > cap then
        throw (.malformed s!"the array at offset {o} has size {sz} above its capacity {cap}")
      pure ⟨24 + 8 * cap, (Array.range sz).map (o + 24 + 8 * ·)⟩
    else if tag == 248 then do
      -- scalar array: size, capacity, then elements of `other` bytes
      fits 24
      pure ⟨align8 (24 + other * uint b (o + 16) 8), #[]⟩
    else if tag == 249 then do
      -- string: byte size, capacity, length, then the bytes
      fits 32
      pure ⟨align8 (32 + uint b (o + 16) 8), #[]⟩
    else if tag == 250 then
      -- big number: its limbs follow it; `cs_sz` is its size
      if csSz < 24 then throw (.malformed s!"the number at offset {o} has size {csSz}")
      else pure ⟨csSz, #[]⟩
    else if tag == 251 || tag == 252 then
      -- thunk or task: its value, then a pointer that is null in a module
      pure ⟨24, #[o + 8]⟩
    else if tag == 253 then
      -- reference: its value
      pure ⟨16, #[o + 8]⟩
    else
      throw (.malformed s!"the object at offset {o} has kind {tag}, which a module cannot hold")
  fits obj.size
  return obj

/-- Stage 1. Returns the number of objects. -/
def checkStructure (b : ByteArray) : Except Defect Nat := do
  if b.size < headerSize then
    throw (.malformed s!"the file has {b.size} bytes, fewer than the {headerSize}-byte header")
  unless text b 0 5 == "olean" do
    throw (.malformed "the file does not start with the `olean` marker")
  let version := text b 7 33
  let commit := text b 40 40
  unless version == Lean.versionStringCore && commit == Lean.githash do
    throw (.incompatible s!"written by Lean {version} (commit {commit}), \
      but this is Lean {Lean.versionStringCore} (commit {Lean.githash})")
  -- the objects tile the rest of the file exactly
  let mut objects : Std.HashMap Nat Obj := {}
  let mut o := headerSize
  while o < b.size do
    let obj ← readObj b o
    objects := objects.insert o obj
    o := o + align8 obj.size
  unless o == b.size do
    throw (.malformed s!"the last object ends at offset {o}, past the end of the file at {b.size}")
  -- the root and every pointer reachable from it point at an object
  let base := uint b 80 8
  let target (source : String) (p : Nat) : Except Defect (Option Nat) :=
    if p % 2 == 1 then pure none  -- a boxed scalar, not a pointer
    else if base + headerSize ≤ p && objects.contains (p - base) then pure (some (p - base))
    else throw (.malformed s!"{source} points at address {p}, which is not an object \
      in the file (offsets {headerSize} to {b.size} from address {base})")
  let mut todo : Array Nat := #[]
  let mut seen : Std.HashSet Nat := {}
  if let some r ← target "the root pointer" (uint b 88 8) then
    todo := todo.push r
    seen := seen.insert r
  while h : todo.size > 0 do
    let r := todo[todo.size - 1]
    todo := todo.pop
    for f in (objects.getD r ⟨0, #[]⟩).fields do
      if let some t ← target s!"the field at offset {f}" (uint b f 8) then
        unless seen.contains t do
          seen := seen.insert t
          todo := todo.push t
  return objects.size

/-- Stage 2: loads the module as `leanchecker` does and replays every
declaration in it through the kernel. Returns how many were checked. The
regions stay mapped until the process exits, since objects in them are still
referenced here. -/
unsafe def checkKernel (file : System.FilePath) : IO Nat := do
  let (mod, _) ← readModuleData file
  let (_, s) ← importModulesCore mod.imports |>.run
  let env ← finalizeImport s mod.imports {} 0 false false (isModule := true)
  let mut constants : Std.HashMap Name ConstantInfo := {}
  for name in mod.constNames, ci in mod.constants do
    constants := constants.insert name ci
  discard <| env.toKernelEnv.replay constants
  return constants.size

end OleanCheck

open OleanCheck in
unsafe def main (args : List String) : IO UInt32 := do
  let [file] := args
    | IO.eprintln "usage: lean --run Loader/OleanCheck.lean FILE.olean"; return 2
  let fail (msg : String) : IO UInt32 := do
    IO.println s!"error: {file}: {msg}"
    return 1
  match checkStructure (← IO.FS.readBinFile file) with
  | .error (.malformed why) => fail s!"malformed compiled module: {why}"
  | .error (.incompatible why) => fail s!"incompatible compiled module: {why}"
  | .ok objects =>
    try
      initSearchPath (← findSysroot)
      let n ← checkKernel file
      IO.println s!"{file}: {objects} objects; {n} declarations replayed through the kernel"
      return 0
    catch e => fail s!"kernel check failed: {e}"
