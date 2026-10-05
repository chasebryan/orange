# The Orange Book

By Chase Bryan

## Part 1, The Novice

Continuation: Chapters 4–6. Draft 2026-10-05.

Continue from [A Rule You Can Undo](NOVICE_OPENING.md#chapter-3-a-rule-you-can-undo).
The opening is unchanged. The language examples below target the compiler
source at `21ae40f77b691099b41ee22990bad3322350eb46`, the baseline of this
continuation. Implementation is not acceptance of a language proposal.
See [the evidence boundary](#evidence-boundary) for what has been checked.

## Chapter 4: A Place to Work

> “Security is a process, not a product.”
>
> — Bruce Schneier, “The Process of Security” (April 2000). [S4]

### 4.1 The calculation needs an address

You have a rule for XOR and a reason that applying the same mask twice
restores a bit string. You could continue calculating on paper indefinitely.
To ask a machine to perform the calculation, we must put the rule somewhere
it can read and tell it which rule to read.

That sounds like administration. It is also the beginning of reproducible
work: another person must be able to identify what you ran, not merely believe
that you ran something useful.

A **file** is a named object whose contents can be stored and retrieved.
The files we will write contain **source code**: text expressing a computation
in a programming language. The text is data until something interprets it
according to that language's rules.

A **directory**, also called a folder, organizes files and other directories.
A directory can contain another directory, which can contain a file. To locate
that file, we describe a route through the directories. That route is a
**path**.

Imagine this small working area. Indentation means “inside the directory
above,” not additional characters in a name:

```text
orange-study/
  notes.txt
  first.or
  experiments/
    mask.or
```

`orange-study` is a directory. It contains the files `notes.txt` and
`first.or`, and another directory named `experiments`. The file `mask.or`
is inside `experiments`.

Starting in `orange-study`, the path `experiments/mask.or` identifies that
file. The slash separates path components. `mask.or` alone would instead
look for a file with that name directly in the starting directory. A short
name does not search every directory on your computer.

For this example, ordinary directories and files are sufficient. More
advanced filesystems can introduce links and other complications; none are
needed to understand these routes.

### 4.2 Relative to what?

A **relative path** is interpreted from a starting directory. An **absolute
path** identifies a location from the filesystem's root rather than from
your current starting point. On a Unix-style system, `/` at the beginning
marks an absolute path. Native Windows paths commonly include a drive and
backslashes; the command track below uses a POSIX-style shell instead. [T1]

The **working directory** is the directory from which the commands we use
interpret relative paths. If you change the working directory, you can
change which file a relative path identifies without changing the path's
written characters.

The special component `.` means the current directory; `..` means its
parent. From `orange-study/experiments`, the relative path `../first.or`
goes up one level and selects `first.or`. From `orange-study` itself,
`../first.or` means a different location. You cannot interpret a relative
path fully without knowing its starting point.

Keep a note of that. A command and its working directory belong together.
Later, a report that gives you one but not the other may be missing part of
its procedure.

### 4.3 An editor is not the evaluator

A **text editor** lets you enter and save text. Some editors recognize
programming languages and color their words; others do not. The colors are
an aid for your eyes. They are not part of an Orange program's meaning.

Save Orange source as plain UTF-8 text, using the `.or` extension. An
**extension** is the ending of a filename, conventionally separated by a dot.
It helps tools and people recognize an intended file type. It does not
transform the contents. Renaming a photograph to `first.or` does not make it
Orange source.

UTF-8 is a character-encoding convention. For the basic letters, digits and
punctuation used in our first source file, it preserves the familiar ASCII
byte values. We will study text encoding more deeply later; here, selecting
“UTF-8” and “plain text” in your editor gives the compiler the kind of file
it expects. [T2]

Do not save rich-text formatting around the program. Do not replace straight
quotes or ordinary punctuation with typographic substitutes inside code.
A word processor can produce a beautiful page whose underlying file is not
the text the compiler was asked to read.

Save the file before running a command against it. An editor may display
changes that still exist only in the editor's unsaved buffer. The command
reads the saved file, not whatever you happen to be looking at.

When a result seems to ignore your edit, check three things before doubting
the arithmetic: the file was saved; the command names that file; the working
directory makes that path identify the intended file.

### 4.4 The terminal, the shell, and the command

A **terminal** presents a text-based interaction with programs. A **shell**
is a program that reads command lines and arranges for commands to run.
They are related, but not identical. The terminal is where you see the
conversation; the shell interprets the command line. [T1]

This chapter gives one explicit command track: a POSIX-style shell such as
Bash or Zsh on a suitably configured Linux or macOS machine. A Linux shell
inside an existing Windows Subsystem for Linux installation is another
possible environment, but this increment does not supply or validate that
installation procedure. These are not PowerShell instructions.

On a shared or managed computer, obtain permission before installing tools.
The paper work and source-reading sections remain useful without an installed
compiler. Do not treat an installation obstacle as a failure to understand
cryptography.

A shell commonly displays a **prompt** while waiting for input. Its appearance
varies. In some books, a dollar sign before a line represents that prompt.
The command blocks here omit it: type the command, not an invented prompt.
Press Enter to submit a complete command line.

Begin with:

```sh
pwd
```

`pwd` prints the working directory. Next:

```sh
ls
```

`ls` lists directory entries. These commands inspect your location; they do
not edit the study files. Their exact display can differ between systems.

A command can have **arguments**: additional values telling it what to act
on or how to act. In `ls experiments`, `experiments` is an argument naming
the directory to list. Spaces ordinarily separate shell words. Quoting a
path containing spaces keeps it together as one argument:

```sh
ls "study notes"
```

The quotes guide the shell's interpretation; they are not part of the
directory name. At this stage, use simple study filenames without spaces so
that filename questions do not obscure the computation. [T1]

### 4.5 Create a clean study directory

Choose a directory in which you are permitted to create files. From there:

```sh
mkdir orange-study
cd orange-study
pwd
```

`mkdir` creates a directory. `cd` changes the shell's working directory.
The final `pwd` checks where you arrived. Run one command at a time and read
its response. If `orange-study` already exists, inspect it rather than
assuming it is empty. Do not delete an existing directory to make a lesson
look like its transcript.

After you enter the study directory, create `notes.txt` with your editor.
Record the date and the calculation you intend to perform. Save it there,
then use `ls` to confirm its name. This is a small rehearsal of the loop we
will use repeatedly: edit, save, identify, run, inspect.

A command's **exit status** is a small integer reporting how that command
finished. Conventionally, zero reports success; a nonzero value reports some
other outcome whose meaning belongs to the command. In this shell track,
read the preceding command's status immediately with:

```sh
echo "$?"
```

`echo` prints its argument. The shell replaces `$?` with the previous
command's status before invoking it. A later command replaces that saved
status, so do not insert another command between the one being inspected
and the query. Success here means the command's own success condition,
not that the file it processed satisfies every claim you care about. [T1]

### 4.6 The program that reads Orange

The Orange tool is named `orangec`. A **compiler** generally translates
source into another representation. An **evaluator** computes the meaning
of expressions for particular inputs. A tool can perform several of these
jobs. The baseline Orange tool checks source and reference-evaluates its
supported fragment; it does not generate native code or check general
cryptographic proofs. [O1]

A **native executable** is a program built to run in a particular operating
system and machine environment. We will build the native `orangec` tool
from its Rust source. That is not the same thing as compiling the Orange
program we give it into native code. The distinction concerns two different
programs written in two different languages.

The Orange repository uses Rust and Cargo. **Rust** is the implementation
language of the compiler. **Cargo** manages the Rust build. A **toolchain**
is the associated collection of build tools. You do not need to learn Rust
to follow the Orange source examples, but the current pre-alpha tool needs
its build environment. [O1]

Use the Rust project's official installation instructions for your operating
system, including its platform linker prerequisites. A **linker** combines
compiled pieces into the executable that will run. Download and installation
requirements belong to your actual platform; do not replace them with an
unexplained administrator command copied from a different system. [T3]

The baseline repository requests Rust `1.96.1` in `rust-toolchain.toml`.
A file recording a selected version is a **pin**: it makes the requested
version explicit instead of allowing “whatever is newest” to silently
change the build. Obtaining the pinned toolchain can require a network
connection. The later `--offline` build flag does not install a missing
Rust toolchain for you. [O1, T4]

### 4.7 Obtain an identified source revision

A **repository** stores a project's files and revision history. **Git** is
the version-control program used here; GitHub hosts the repository. A
**commit** identifies a recorded revision. A **checkout** is the working
copy of a revision's files. [T5]

Install Git using its official instructions if it is not already available.
Then, from inside the study directory, obtain a separate copy:

```sh
git clone https://github.com/chasebryan/orange.git orange-source
cd orange-source
git checkout --detach 21ae40f77b691099b41ee22990bad3322350eb46
git rev-parse HEAD
```

`clone` creates `orange-source` and downloads the repository. It should not
be directed at an existing directory containing your work. `checkout --detach`
selects the named recorded revision instead of following a moving branch.
A detached checkout is suitable for this read-and-build exercise; Git's
notice about that state is not an error. `rev-parse HEAD` prints the selected
commit identifier. Compare the entire identifier, not just a few characters.
[T5]

The long hexadecimal string is a revision identifier, not a password
or encryption key. Naming a revision supports reproducibility;
it is not, by itself, an authenticity or security guarantee.

These commands select the compiler baseline used to write this continuation.
They are not an instruction to stay on that old revision for unrelated work.
The book's own source may later advance beyond it. A future edition should
record which compiler revision its examples were checked against rather than
quietly changing the meaning of an old transcript.

### 4.8 Build the tool, then identify the tool

You should now be in the root of `orange-source`, where the `compiler`
directory and `rust-toolchain.toml` are visible. Build:

```sh
cargo build --manifest-path compiler/Cargo.toml -p orangec --locked --offline
```

`build` asks Cargo to construct the selected package. `--manifest-path`
identifies its workspace manifest, a file describing the Rust project.
`-p orangec` selects the package named `orangec`. `--locked` refuses changes
to the dependency lockfile; `--offline` prevents Cargo from resolving or
fetching dependencies over the network. The baseline workspace has no
third-party Rust dependencies. These flags do not certify the source or
make the compiler independently reviewed. [O1, T4]

A successful default development build places the executable at this relative
path on the Unix-style track:

```sh
./compiler/target/debug/orangec --version
```

The leading `./` names a path from the working directory. It prevents the
shell from choosing an unrelated program named `orangec` elsewhere in its
command search path. `--version` asks the executable to identify itself.
The baseline reports package version `0.0.1`, Orange edition `2026`, and
implemented slice `S3t`. The slice records implemented behavior, not formal
acceptance of every semantic proposal or a production-release promise. [O1]

Do not run later commands after a failed build and assume an old executable
is the newly built one. Read the first meaningful error. A missing toolchain,
a missing linker, an incorrect working directory and rejected Orange source
are different failures at different stages.

Building the compiler does not require modifying this repository's source.
Keep your exercise files in the parent `orange-study` directory. The path
`../first.or`, read from `orange-source`, will then refer to your first
exercise file.

### 4.9 Record enough to repeat the work

In your notes, record the source revision, version output, working directory,
command, input file and observed result. For a failed attempt, retain the
first useful diagnostic as well. A **diagnostic** is a message intended to
explain a condition such as invalid source or an unavailable file.

A useful record is not “it worked.” It says which question was asked of which
program and what that program reported. Even a perfect account of a run is
only an account of that run. It nevertheless gives someone else something
concrete to inspect.

Schneier's opening sentence concerns security practice, not editor setup.
The connection here is narrower: possession of a tool does not replace a
procedure for using it carefully. Before the first calculation, we have
already encountered a way for two people to appear to follow the same
instructions while actually running different files.

### 4.10 Work at the desk

**Exercise 4.1 — Identify the route.** In the directory picture in §4.1,
start inside `experiments`. Give a relative path to `notes.txt`, then explain
why that same path does not mean the same thing from `orange-study`.

**Exercise 4.2 — Explain the boundary.** You rename a rich-text document
from `first.rtf` to `first.or`. Which thing changed? What has not thereby
been established about its contents?

**Exercise 4.3 — Locate the stale result.** You change a literal in an editor,
run your saved source, and receive the previous result. Give three checks
that should precede changing the mathematical rule.

**Exercise 4.4 — Read the arguments.** Explain each component of the Cargo
build command in §4.8. Which component selects the package? Which selects
the manifest? Does `--offline` supply a missing toolchain?

**Exercise 4.5 — Distinguish two compilations.** You successfully build
`orangec` from Rust source. Does that show that Orange source has been
translated into native code? Identify the two different source programs.

**Exercise 4.6 — Improve the record.** Replace “the answer was right” with
a small experiment record that another reader could repeat. Use invented
filenames and results, and label them as examples rather than observations.

**Exercise 4.7 — Inspect a status.** A command finishes with status zero.
What does that tell you? Why must you still inspect the command's purpose,
its input and its output before stating a cryptographic claim?

You now know where the text goes and which program will read it. Next, make
every character of the text earn its place.

## Chapter 5: Tell the Machine Exactly

> “We can only see a short distance ahead, but we can see plenty there that needs to be done.”
>
> — Alan M. Turing, “Computing Machinery and Intelligence” (1950),
> concluding sentence. [S5]

### 5.1 A complete beginning

Create `first.or` in `orange-study`, not inside `orange-source`. Save exactly
this plain text. The shaded box is source; the box's border is not.

**Listing 5.1 — `first.or`**

```orange
edition 2026;
module first_steps {
  spec answer() -> Word[8] {
    13
  }
}
```

This is a complete file, not a fragment requiring an invisible wrapper.
Its only result is thirteen. That is enough to examine the boundary between
what we intend, what we write and what the tool reads.

Return to your shell in `orange-source`. Confirm the working directory,
then ask the built tool to check the saved file:

```sh
./compiler/target/debug/orangec check ../first.or
```

`check` requests source validation. `../first.or` names the input file.
A successful check does not compute and print the result of `answer`;
evaluation is a separate command. It also does not infer that a function
named `answer` solves any problem you had in mind. [O1]

Before evaluating it, explain why thirteen fits in a byte. Chapter 2 already
gave you the required range. You should not need the machine to settle that
question.

```sh
./compiler/target/debug/orangec eval ../first.or
```

**Expected evaluation output:**

```text
first_steps::answer: Word[8] = 0x0d
```

Hexadecimal `0x0d` is thirteen. The display did not change the value into a
letter or an encrypted message. It chose a representation.

### 5.2 Read the first line

A **keyword** is a word assigned a particular grammatical role by a language.
`edition` is a keyword here. The declaration `edition 2026;` identifies the
language edition for the source. The semicolon ends that declaration.
It does not assert that the source was written in 2026, and it is not the
Rust toolchain version from Chapter 4. [O2]

A **declaration** introduces or states something in the program. This first
one states an edition. The next one introduces a module.

The whitespace between `edition` and `2026` separates them. You may indent
the following lines to make structure visible, but indentation does not
replace the semicolon or braces that the grammar requires.

If you write `edition2026`, you have not compressed the same declaration.
You have formed different text. Reading a program is not guessing which
English phrase its author probably intended.

### 5.3 The module is a named boundary

`module first_steps {` begins a **module** named `first_steps`. A module
organizes declarations under a name. The opening brace `{` begins its body;
the matching closing brace `}` ends it. The final brace in Listing 5.1
closes this module.

`first_steps` is an **identifier**: a name chosen according to the language's
naming rules. Use letters, digits and underscores as the introductory examples
do, beginning with a letter. Case matters: `answer` and `Answer` are different
identifiers in Orange. [O2]

Inside the module is a function named `answer`. The evaluator identifies it
as `first_steps::answer`. The double colon separates the module name from
the function name. This is a **qualified name**, a name that includes its
surrounding context rather than depending on a short name alone.

For our one-file example, the filename and module identifier have different
roles: the path locates the file; the module name identifies declarations
inside it. Later, Orange's rules for loading additional modules connect
module names to neighboring filenames. We are not using that facility yet.
Do not infer its complete rules from this first example. [O1]

### 5.4 A function is a rule with an interface

In mathematics, a **function** associates each permitted input with one
output. Its **domain** is the collection of permitted inputs. The specified
collection in which its outputs must lie is its **codomain**. We will use
functions to describe computations without first tying them to a machine.

For example, “add one to a whole number” associates zero with one, one with
two, and so on. Whether negative numbers are permitted depends on the stated
domain. The verbal rule alone need not settle every part of the interface.

An **interface** states how a function is used: what inputs it accepts and
what kind of output it provides. Orange makes these parts visible in the
function declaration.

In `spec answer() -> Word[8]`, `spec` introduces an executable specification
function in the implemented fragment used here. It does not mean the function
has been proved secure or even that it matches an external standard. [O1]

The name is `answer`. The parentheses contain its parameter list. A
**parameter** is a name standing for an input supplied when the function is
used. This list is empty, so `answer` takes no parameters.

The arrow `->` introduces the result type. A **type** describes the values
allowed in a role and helps determine which operations have meaning there.
`Word[8]` is Orange's eight-bit unsigned word type. Its 256 values have
unsigned representatives zero through 255. The brackets enclose the word
width; they are not a request to select the eighth item in a list. [O2, O3]

The next braces enclose the function's body. The literal `13` is its final
expression. A **literal** writes a value directly. An **expression** is
source that denotes a value or a computation of a value. Here, the expression
is just the literal itself.

The final expression supplies the result. Orange has no `return` keyword,
and there is no semicolon after `13`. A semicolon ends the edition
declaration; it does not end this expression. Do not add punctuation merely
because a different programming language uses it. The permitted grammar
belongs to Orange. [O2]

### 5.5 Use the rule you already understand

Save the following complete file as `masks.or` beside `first.or`.

**Listing 5.2 — `masks.or`**

```orange
edition 2026;
module masks {
  spec apply(x: Word[8], k: Word[8]) -> Word[8] {
    x ^ k
  }

  spec example() -> Word[8] {
    apply(0x0b, 0x06)
  }

  spec recovered() -> Word[8] {
    apply(apply(0x0b, 0x06), 0x06)
  }
}
```

`apply` accepts two parameters. Each colon associates a parameter name with
its type. The comma separates the parameters. Both `x` and `k` are byte-sized
words, and the declared result has the same type.

Within Orange source, `^` is bitwise XOR. It is not exponentiation. In
Chapter 2, `2³` meant a mathematical power. In Chapter 3, `⊕` denoted XOR in
mathematical discussion. The symbol used to write an operation depends on
the notation or language being read. [O3]

A **call** uses a function with supplied input values. In
`apply(0x0b, 0x06)`, those values are **arguments**. Parameters name the input
roles in the definition; arguments supply values for a particular use.
The first argument goes to `x`, and the second goes to `k`.

Read the function body with those values substituted. It asks for
`0x0b ^ 0x06`. In binary, that is the byte-sized form of the operation you
already performed on paper:

```text
00001011
00000110
-------- XOR
00001101
```

The line of dashes labels the paper calculation; it is not Orange source.
The numerical result is thirteen. The byte still has eight positions,
including its leading zeros.

`recovered` contains a call inside another call. The inner result supplies
the first argument of the outer call. Start inside:

```text
apply(0x0b, 0x06) = 0x0d
apply(0x0d, 0x06) = 0x0b
```

Those equations explain the calculation; they are not declarations to paste
into the source file. They instantiate Proposition 3.1 using one retained
mask, not a new proof of confidentiality.

Check and evaluate the complete file:

```sh
./compiler/target/debug/orangec check ../masks.or
./compiler/target/debug/orangec eval ../masks.or
```

**Expected evaluation output:**

```text
masks::example: Word[8] = 0x0d
masks::recovered: Word[8] = 0x0b
```

The baseline evaluator reports the root module's parameterless specifications
in declaration order. It does not choose arbitrary arguments for `apply`
and print a table of every possible call. The absence of such a line is not
an indication that `apply` was ignored: both reported functions call it. [O1]

### 5.6 Scope gives a name its meaning

The parameter `x` names an input inside `apply`. It does not create a
universal variable named `x` in every other function. **Scope** is the region
in which a declaration makes a name available.

You can rename `x` to `value` if you also rename its uses within that scope.
The intended operation remains the same. Rename only the parameter and leave
`x ^ k` in the body, however, and the body refers to a name no longer declared
there. A consistent renaming preserves a relationship; an incomplete renaming
breaks it.

Similarly, `example` is a name, not an instruction to produce an example.
`recovered` is a name, not a certificate of recovery. You could name a
function `secure` and give it a body that simply returns its input. Names
can communicate intent to readers, but the body supplies behavior.

Comments have a different purpose. In Orange, `//` begins a line comment;
the remainder of that line is commentary rather than an expression. For
example, this is a fragment showing where a comment could sit:

```text
// Keep the mask available for the second application.
```

The comment can explain a decision. It cannot make the program retain a mask
that its computation discards. A careful reader checks whether the source
does what its commentary says. [O2]

### 5.7 Make one deliberate error

Keep the working `first.or`. Make a separate copy named `too_large.or`, then
change its function body as shown below. Do not overwrite your correct
example merely to follow the exercise.

**Listing 5.3 — `too_large.or`, intentionally rejected**

```orange
edition 2026;
module too_large {
  spec answer() -> Word[8] {
    256
  }
}
```

The type requires a byte-sized value. The literal is 256, which cannot be
represented as a `Word[8]` literal. Checking this file should reject it; a
successful evaluation output is not expected. [O2]

```sh
./compiler/target/debug/orangec check ../too_large.or
echo "$?"
```

Read the diagnostic, including the named file and indicated source location.
The exact line and column depend on the saved file, so compare the reason
for rejection rather than copying an unrelated screenshot's coordinates.
The program was not rejected because Orange dislikes large numbers. Its
literal conflicts with the type required at that position.

You may wonder whether 256 ought to become zero in an eight-bit word. That
is a question about the difference between constructing a literal and
performing a specified arithmetic operation. Keep it. Chapter 6 answers it
without changing this literal rule.

### 5.8 A successful check does not choose your intention

Change the working example's body from `13` to `12`, save, then check and
evaluate. Twelve fits the type just as thirteen does. The file can therefore
be well-formed and type-correct while failing your unstated requirement to
produce thirteen.

The source checker cannot recover a requirement you never supplied to it.
You need a way to state an expected result and compare the observed result
with it. Later we will place explicit tests beside functions and examine
what those tests cover. For this first run, your paper derivation supplies
the expected result; the transcript supplies the observation.

A **syntax error** concerns whether text has an allowed grammatical form.
A **type error** concerns an inconsistency in the values or operations the
program declares. A **behavioral mistake** can remain after those checks:
the program means something precise, but not the thing you intended.
These are distinctions for diagnosis, not a promise that every compiler
will phrase every diagnostic under exactly those headings. [O1, O2]

### 5.9 Learn the structure, not the incantation

Close the listing and explain how you would rebuild it. You need an edition,
a module, a function name, its input and result types, and a body expressing
the operation. You now have a reason for each piece of source instead of a
string to memorize.

Turing's concluding sentence looked toward further work on intelligent
machines, not toward Orange. Here it marks a more modest threshold: you do
not yet know the whole language, but you have a complete program and specific
questions you can investigate. Progress no longer depends on pretending
that the rest of the subject is already familiar.

### 5.10 Work at the desk

**Exercise 5.1 — Explain the punctuation.** In Listing 5.1, identify the
roles of `;`, `()`, `->`, `[]`, and each pair of braces. Explain why the
parentheses are empty.

**Exercise 5.2 — Trace a call.** Trace `apply(0xa6, 0x3c)` using Listing 5.2.
Write the argument-to-parameter associations, the two binary strings, and
the result. Trace a second application of the same mask.

**Exercise 5.3 — Rename consistently.** Rewrite the definition of `apply`
using parameter names `value` and `mask`. Which occurrences must change?
Which punctuation and types do not change?

**Exercise 5.4 — Add a result.** Add a parameterless function named `zero`
that calls `apply` with the same permitted byte value in both argument
positions. Derive the expected result before running it.

**Exercise 5.5 — Separate failure kinds.** Consider three edits: remove a
closing brace; put `256` in the `Word[8]` literal body; replace `13` with
`12`. Explain why the first two should be rejected and why the third can
pass source checking while missing an intended result.

**Exercise 5.6 — Read the display.** Explain every field in
`masks::recovered: Word[8] = 0x0b`. Does that one line establish the result
for every permitted pair of arguments to `apply`?

**Exercise 5.7 — Distinguish text from execution.** A source comment says
“this function authenticates its input,” but its body only computes XOR.
What evidence does the comment contribute about actual authentication?
What would have to be specified before the claim could be examined?

**Exercise 5.8 — Inspect a supplied label.** Someone renames `answer` to
`proved_answer` without changing the body. State one thing that changes
and one thing that does not.

A type did more than reserve space. It rejected one literal and admitted
another. Now let it determine what happens at the end of its range.

## Chapter 6: Words Have Edges

> “RC5 should be simple. It should be easy to implement.”
>
> — Ronald L. Rivest, “The RC5 Encryption Algorithm,” introduction,
> p. 87. [S6]

### 6.1 Follow the carry

Write the largest byte-sized unsigned value, then add one on paper without
limiting the number of positions:

```text
  11111111
+ 00000001
----------
 100000000
```

The result requires nine positions. Its value is 256. The eight-position
format has not mysteriously gained another value; the calculation has
produced a result outside that format's unsigned range.

Several language designs could respond differently. One might reject the
operation, one might produce a wider result, and one might retain eight
positions according to an explicit wrapping rule. The machine's finite
storage does not, by itself, specify which language rule applies.

The implemented baseline defines `+`, `-`, and `*` on `Word[w]` by arithmetic
modulo two to the power `w`. The letter `w` stands for that width, and the
result keeps it. This is the baseline's rule for the fragment these chapters
run, not an acceptance of the proposal that states it. We will now calculate
the rule by hand. [O3]

### 6.2 Count in complete turns

Draw sixteen labeled positions, zero through fifteen, in a circle or in a
row that returns to its beginning. This is a paper model, not an Orange
`Word[4]` declaration: the baseline word types used here have widths 8, 16,
32 and 64. [O3]

Start at fourteen and advance five steps. You visit fifteen, zero, one,
two, three. Ordinary addition gives nineteen; counting around a sixteen-
position cycle leaves you at three. The difference is one complete turn
of sixteen.

A positive **modulus** specifies the size of the cycle. To **reduce** an
integer modulo that modulus, choose its representative from zero through
one less than the modulus. A **representative** is the selected number used
to stand for all integers differing from it by complete turns.

For modulus sixteen:

```text
19 = (1 × 16) + 3
35 = (2 × 16) + 3
```

Nineteen and thirty-five reduce to three. So does three itself. Their
ordinary numerical values have not become equal. They share the same
representative under the specified modular relation.

We write `19 ≡ 3 (mod 16)` and read it as “nineteen is congruent to three
modulo sixteen.” The symbol `≡` names **congruence**, not ordinary equality.
Two integers are congruent modulo a positive modulus when their difference
is an integer multiple of that modulus.

We will usually choose a modulus of at least two. The width-eight word
uses 256, giving representatives zero through 255. Thus 256 reduces to
zero and 257 reduces to one.

### 6.3 Negative integers belong to the rule too

An **integer** is a whole number, its negative, or zero. A negative integer
can describe steps in the opposite direction from positive steps. Start
at zero in the sixteen-position model and move back one step: you reach
fifteen.

That agrees with the equation:

```text
-1 = (-1 × 16) + 15
```

Multiplication by negative one changes a number to its opposite. The
right side is negative sixteen plus fifteen, which is negative one. Our
representative, fifteen, lies in the required range.

For a byte-sized word, subtracting one from zero similarly gives 255 under
the word operation:

```text
-1 = (-1 × 256) + 255
```

Do not write `-1 = 255` as an ordinary integer equation. They are congruent
modulo 256, not equal as integers.

**Proposition 6.1 — A nonnegative remainder.** Take any integer `n` and any
positive integer `m`. There is exactly one pair of integers `q` and `r`
such that `n = q × m + r` and `0 ≤ r < m`. The symbol `≤` means “less than
or equal to,” and `<` means “strictly less than.” `q` counts complete turns.
`r` is the representative. When `n` is negative, `q` may be negative. [M1]

**Proof.** The positive integer `m` is at least one.

Suppose `n` is at least zero. Consider the finite list whose step `k` is
`k × m`, for `k` from zero through `n + 1`. Step zero is zero, which does
not exceed `n`. Step `n + 1` is `(n + 1) × m`, which is at least `n + 1`
and therefore exceeds `n`. Let `q × m` be the last entry in this list that
does not exceed `n`. The final entry exceeds `n`, so `q × m` has a successor
in the list. Set `r = n - (q × m)`. Then `r` is at least zero. The successor
is `(q × m) + m` and exceeds `n`, so `r` is strictly less than `m`.

Suppose `n` is negative. Let `t` be the positive distance from `n` up to
zero, so `n` lies `t` ones below zero. Start at zero and subtract `m` once
per step, for `t` steps. Each subtraction moves down by at least one,
because `m` is at least one, so `t` subtractions move down by at least `t`
and reach or pass `n`. In that finite list, from the starting zero through
the result of the `t`-th subtraction, take the first entry that is less
than or equal to `n`. That entry is a multiple of `m`; call it `q × m`.
Here `q` is negative. Zero itself is greater than `n`, so this entry has
a preceding one, and that preceding entry is greater than `n`. Consecutive
entries differ by `m`, so `n` is at least `q × m` and strictly less than
`(q × m) + m`. Set `r = n - (q × m)`. Then `0 ≤ r < m`.

For uniqueness, suppose two pairs both express `n` in the required form.
Their remainders then differ by an integer multiple of `m`. Each remainder
lies from zero through `m - 1`, so that difference lies strictly between
`-m` and `m`. The only multiple of `m` strictly between those bounds is
zero. Thus the remainders agree. The two turn counts then differ by an
integer, and that integer's product with `m` is zero. A positive `m` times
a positive integer is positive, and a positive `m` times a negative integer
is negative, so the only integer with product zero is zero. The turn counts
agree as well.

Each integer is therefore congruent to exactly one representative in range.
Congruent integers share that representative.

### 6.4 Let the type choose the arithmetic

Save the following as `word_edges.or` in the study directory.

**Listing 6.1 — `word_edges.or`**

```orange
edition 2026;
module word_edges {
  spec wrapped() -> Word[8] {
    255 + 1
  }

  spec backward() -> Word[8] {
    0 - 1
  }

  spec product() -> Word[8] {
    200 * 2
  }

  spec integer_sum() -> Int {
    255 + 1
  }

  spec wider_sum() -> Word[16] {
    255 + 1
  }
}
```

In Orange source, `*` writes multiplication. The mathematical `×` from our
paper equations is not the operator spelling in this program. `Int` is the
mathematical-integer type. Its arithmetic does not wrap at a fixed machine
width, although a real evaluator still has finite resources and can reject
work exceeding its implementation limits. [O3]

Predict every output before evaluating:

```sh
./compiler/target/debug/orangec eval ../word_edges.or
```

**Expected evaluation output:**

```text
word_edges::wrapped: Word[8] = 0x00
word_edges::backward: Word[8] = 0xff
word_edges::product: Word[8] = 0x90
word_edges::integer_sum: Int = 256
word_edges::wider_sum: Word[16] = 0x0100
```

Two hundred times two gives four hundred as an integer. One complete turn
of 256 leaves 144; hexadecimal `0x90` represents 144. The sixteen-bit
addition has more range, so 256 is still represented directly there.
The `Int` result also has value 256, but for a different reason: it does
not use fixed-width wrapping arithmetic.

Why was Listing 5.3 rejected while `wrapped` works? In `wrapped`, both
literals fit their expected word type and the defined addition produces a
word result. In Listing 5.3, the single literal `256` itself does not fit
`Word[8]`. Literal admission and arithmetic reduction are different rules.
The book must teach both rather than replacing them with “everything wraps.”
[O2, O3]

### 6.5 Reduction forgets a count, not a position

Reduction modulo 256 maps integer zero, 256, 512 and many other integers
to the same word value. From that word alone, you cannot recover how many
complete turns were removed. Information has been discarded by that map.

But adding a retained constant to an already byte-sized value is reversible
within the byte-sized domain: subtract the same constant modulo 256. For
example, 250 plus ten wraps to four, and four minus ten wraps back to 250.

The example is one pair, not the argument. Let `x` be any byte, so
`0 ≤ x < 256`, and let `c` be the retained integer. Proposition 6.1 supplies
`q` and `r` with `x + c = (q × 256) + r` and `0 ≤ r < 256`. Then
`r - c = x - (q × 256)`, so `r - c` and `x` differ by a multiple of 256.
They are congruent. Their shared representative is `x`, because `x` is
already in range. Word subtraction denotes that representative, so
subtracting `c` from `r` returns `x` for every byte.

There is no contradiction. One statement concerns arbitrary integers mapped
into a smaller domain; the other concerns a transformation within a fixed
domain with the added constant retained.

This distinction resembles §3.6, but it is a new application: count which
information is available before asking whether a transformation can be
undone. “It loses information” and “it is reversible” can both become vague
claims when their inputs and retained data are not identified.

Multiplication behaves differently. Modulo 256, both zero and 128 become
zero after multiplication by two. With only the result and the retained
multiplier, you cannot tell those two inputs apart. An operation's familiar
integer name does not guarantee an inverse inside a different arithmetic.
We will derive the exact conditions for modular inverses later.

### 6.6 Move the positions without changing the width

Return to the byte `10000001`, or `0x81`. A **left shift by one** moves
each bit one position toward the more significant end, discards the bit
that leaves that end, and inserts zero at the other end. A **right shift
by one** does the opposite, inserting zero at the most significant end.
Here we are discussing unsigned logical shifts, not signed arithmetic
right shifts from other languages. [O3, O4]

The **most significant** position has the largest binary place value;
the **least significant** has the smallest. For our written byte, these
are the leftmost and rightmost positions, respectively.

```text
Original:      10000001
Left by one:   00000010
Right by one:  01000000
```

The original integer value is 129. Left shift by one corresponds to
multiplication by two followed by reduction to the word width: 258 becomes
two. Right shift by one corresponds to integer division by two with the
nonnegative remainder discarded: 129 gives quotient 64 and remainder one.
For a nonnegative shift amount `s`, the factor is two to the power `s`. [O4]

A **rotation** keeps the departing bits and brings them back at the other
end. It changes positions without dropping the bits:

```text
Original:        10000001
Rotate left 1:   00000011
Rotate right 1:  11000000
```

If those differences feel small, trace the leftmost one in the original.
A left shift discards it. A left rotation brings it into the rightmost
position. The other seven moves are the same in this example; the returning
bit explains the difference between `0x02` and `0x03`.

### 6.7 Read every angle bracket

Orange spells left shift `<<` and right shift `>>`. It spells left rotation
`<<<` and right rotation `>>>`. One additional angle bracket changes the
operation. [O3, O4]

Save this complete file as `movement.or`:

**Listing 6.2 — `movement.or`**

```orange
edition 2026;
module movement {
  spec shifted_left() -> Word[8] {
    0x81 << 1
  }

  spec shifted_right() -> Word[8] {
    0x81 >> 1
  }

  spec rotated_left() -> Word[8] {
    0x81 <<< 1
  }

  spec rotated_right() -> Word[8] {
    0x81 >>> 1
  }

  spec restored() -> Word[8] {
    (0x81 <<< 1) >>> 1
  }
}
```

**Expected evaluation output:**

```text
movement::shifted_left: Word[8] = 0x02
movement::shifted_right: Word[8] = 0x40
movement::rotated_left: Word[8] = 0x03
movement::rotated_right: Word[8] = 0xc0
movement::restored: Word[8] = 0x81
```

Use `check` and `eval` with `../movement.or`, as you did for the previous
files. The final function rotates and then reverses that rotation. It
returns the original byte because every position returns to its starting
place. Number the positions `0` through `w - 1`, starting at the most
significant end. One left rotation sends the bit at position `0` to position
`w - 1` and sends every other bit one step toward position `0`. One right
rotation sends each of those bits back. Repeat the left rotation a retained
number of times, then the right rotation the same number of times: each bit
is back in its starting position. The listing is that accounting for one
step on a byte. It does not depend on which positions held ones, or on the
width being eight.

By contrast, shifting left and then shifting right can fail to restore the
original. In our example, `0x81` shifts left to `0x02`, then right to
`0x01`. The first shift discarded a bit; the second has no way to infer it.
Keeping the width the same did not preserve all the information.

### 6.8 Zero, a complete turn, and one turn too many

A rotation by zero changes no positions. A rotation by exactly the word
width makes one complete turn and also returns the original. A rotation
by the width plus one has the same effect as a rotation by one.
For a positive width `w`, the effective rotation amount is the amount
reduced modulo `w`. That reduction is Proposition 6.1 with modulus `w`.
A negative amount is a positive number of steps in the other direction;
its representative is how many forward steps land in the same place.

A shift is different. In the implemented S3r rules at this baseline, a
computed shift by a distance of the width or more produces zero. There is
no returning bit. A negative amount reverses the direction, and the distance
is then the absolute value. If that distance is still the width or more,
the result is zero. Unlike a rotation, the shift amount is not folded
modulo the width. These are the baseline's reference rules, not a promise
that every host language or processor interprets its shift instructions
the same way. [O4]

One syntax rule matters here. A bare literal amount is checked against the
word width: on a byte, `x << 8` is rejected, because a literal amount must
be from zero through seven. S3r treats a grouped expression, such as `(8)`,
as a computed amount instead. Thus `x << (8)` is accepted and produces
zero. Likewise, `x <<< (8)` expresses a complete computed turn. [O4]

The parentheses do not change the number eight. They change which source
form the checker sees. The literal restriction is a diagnostic guard; the
computed rule supplies a meaning for values obtained from expressions.
Reading only the arithmetic and ignoring that distinction would give you
correct mathematics in a rejected program.

The next listing makes the intended computed boundary amounts explicit.

**Listing 6.3 — `boundary_moves.or`**

```orange
edition 2026;
module boundary_moves {
  spec no_turn() -> Word[8] { 0x81 <<< 0 }
  spec full_turn() -> Word[8] { 0x81 <<< (8) }
  spec extra_turn() -> Word[8] { 0x81 <<< (9) }
  spec lost_left() -> Word[8] { 0x81 << (8) }
  spec lost_right() -> Word[8] { 0x81 >> (9) }
  spec reverse_direction() -> Word[8] { 0x81 <<< (-1) }
}
```

**Expected evaluation output:**

```text
boundary_moves::no_turn: Word[8] = 0x81
boundary_moves::full_turn: Word[8] = 0x81
boundary_moves::extra_turn: Word[8] = 0x03
boundary_moves::lost_left: Word[8] = 0x00
boundary_moves::lost_right: Word[8] = 0x00
boundary_moves::reverse_direction: Word[8] = 0xc0
```

An older compiler may reject the computed forms that this implemented
slice defines. The current compiler still rejects bare out-of-range
literals. That is why both the exact source and the revision record in
Chapter 4 belong with the example. Do not “fix” a book by changing a
negative example into a positive one without also checking which semantics
the book targets.

### 6.9 Group first, then calculate

Consider two calculations on byte-sized values:

```text
(1 + 1) XOR 1
1 + (1 XOR 1)
```

The first adds one and one to obtain two, then XORs with one to obtain
three. The second XORs one with one to obtain zero, then adds one to
obtain one. Grouping changes the result even though the same three
input values and two operations appear.

Orange does not let an unparenthesized mixture of addition and XOR choose
between those meanings by your visual preference. Group the operations
explicitly. [O3]

**Listing 6.4 — `grouping.or`**

```orange
edition 2026;
module grouping {
  spec add_first() -> Word[8] { (1 + 1) ^ 1 }
  spec xor_first() -> Word[8] { 1 + (1 ^ 1) }
}
```

**Expected evaluation output:**

```text
grouping::add_first: Word[8] = 0x03
grouping::xor_first: Word[8] = 0x01
```

The corresponding ungrouped expression is deliberately invalid:

**Listing 6.5 — `ungrouped.or`, intentionally rejected**

```orange
edition 2026;
module ungrouped {
  spec answer() -> Word[8] { 1 + 1 ^ 1 }
}
```

The diagnostic should identify missing grouping between operator families.
The rule is not that Orange refuses every expression with several
operators. It is that these families do not have an implicit relative
precedence that you may rely on. [O3]

### 6.10 A small reversible construction

You now have three operations that can be combined on a byte: add a
retained value, XOR with a retained mask, and rotate by a retained amount.
Each has an inverse in the stated domain. Apply their inverses in the
opposite order to undo their composition.

Use a fixed educational rule: add seven, XOR with `0x3c`, then rotate
left by one. This is **not a secure cipher**. It is a small construction
for studying composition and its inverse; its constants are public,
its domain has only 256 values, and we make no confidentiality claim.

**Listing 6.6 — `small_round.or`**

```orange
edition 2026;
module small_round {
  spec forward(x: Word[8]) -> Word[8] {
    ((x + 7) ^ 0x3c) <<< 1
  }

  spec backward(y: Word[8]) -> Word[8] {
    ((y >>> 1) ^ 0x3c) - 7
  }

  spec example() -> Word[8] {
    forward(0xfa)
  }

  spec recovered() -> Word[8] {
    backward(forward(0xfa))
  }
}
```

Trace the forward operation on `0xfa`, which is 250. Adding seven wraps
to one. XOR with `0x3c` gives `0x3d`. Rotating left by one gives `0x7a`.
Now reverse: rotate right to `0x3d`; XOR the same mask to obtain one;
subtract seven modulo 256 to obtain 250.

**Expected evaluation output:**

```text
small_round::example: Word[8] = 0x7a
small_round::recovered: Word[8] = 0xfa
```

The particular trace is a worked example. The general argument identifies
the intermediate value after each inverse operation. Right rotation undoes
the final left rotation. XOR with the retained mask undoes the preceding
XOR by Proposition 3.1. Subtracting seven modulo 256 undoes the initial
addition, by the argument in §6.5. Every permitted byte therefore returns
to itself under those definitions.

Notice the order. Subtracting seven first would generally act on the
rotated, masked value rather than on the value to which seven was added.
Inverses must be matched to the operations they undo, not collected in an
arbitrary order because they look familiar.

Rivest's epigraph introduces simplicity as a design goal for RC5; its
surrounding discussion also values a structure that can be analyzed.
That historical quotation is not a recommendation to deploy RC5 or our
small construction. Here the useful lesson is visible in six lines of
source: a compact program can be worth a careful derivation. [S6]

### 6.11 Work at the desk

**Exercise 6.1 — Reduce by hand.** Find the representatives of 259, 511,
512 and -2 modulo 256. For each, give an equation of the form
`n = q × 256 + r` with `0 ≤ r < 256`.

**Exercise 6.2 — Distinguish a literal from an operation.** Explain why
`256` is rejected as a `Word[8]` literal while `255 + 1` is permitted as
an expression of that type. What result does the latter expression denote?

**Exercise 6.3 — Follow the type.** Predict `250 + 10` as `Word[8]`, as
`Word[16]`, and as `Int`. Explain which numerical results agree and why
the agreeing types are still not identical.

**Exercise 6.4 — Track a departing bit.** Starting with byte `0xa5`,
calculate left shift one, right shift one, left rotation one and right
rotation one. Show all eight positions each time.

**Exercise 6.5 — Inspect information loss.** Give two distinct bytes with
the same result after a left shift by one. Explain why the shifted result
alone does not identify which original was used.

**Exercise 6.6 — Use a complete turn.** For byte `0xa5`, predict rotation
left by zero, eight and nine positions. Contrast the result of shifting
left by eight. State which baseline semantics you are using and how the
amount eight must be written in Orange source to request the computed rule.

**Exercise 6.7 — Reject an invalid inference.** A reader notices that
modular addition by seven is reversible and concludes that modular
multiplication by any nonzero value must be reversible too. Give a
counterexample modulo 256.

**Exercise 6.8 — Repair the grouping.** Write two parenthesizations of
`1 + 1 ^ 1` that Orange can distinguish. Compute both and explain why the
unparenthesized form is not a harmless abbreviation.

**Exercise 6.9 — Derive before running.** Apply Listing 6.6 to input zero,
then undo the result by hand. Identify the exact order of inverse operations.

**Exercise 6.10 — Delimit the claim.** You have a proof of the small
construction's mathematical reversibility and a successful run of its
Orange example. State a claim supported by each. Name two further claims
that neither fact establishes by itself.

**Exercise 6.11 — Find the first wrong step.** A proposed inverse first
subtracts seven, then XORs `0x3c`, then rotates right. Apply it to the
forward result for input `0xfa`. Find the first intermediate value that
no longer undoes the corresponding forward step.

**Exercise 6.12 — Establish a general fact.** Explain why a fixed-width
rotation preserves the number of one bits. Does preserving that number,
by itself, establish that an arbitrary transformation is a rotation?

The next lesson will name intermediate values and let a program carry
several of them at once. You will be able to inspect a computation at each
stage rather than hiding its order inside one long expression.

## Worked answers: Chapters 4–6

### Chapter 4 answers

**4.1.** `../notes.txt` goes from `experiments` up to `orange-study`, then
to its `notes.txt`. Starting in `orange-study`, the same path goes to
that directory's parent instead. The relative path requires a starting
point to identify a location.

**4.2.** The filename changed. The original file format and contents did
not thereby become plain Orange source. An extension signals intention;
it does not prove that the content follows the intended format.

**4.3.** Check that the buffer was saved, that the command names the intended
file, and that the working directory resolves that relative path correctly.
Also identify the actual executable if several compiler builds exist.

**4.4.** `cargo` is the build tool; `build` requests a build;
`--manifest-path compiler/Cargo.toml` selects the manifest; `-p orangec`
selects the package; `--locked` prohibits lockfile changes; `--offline`
restricts Cargo's network access. The latter does not install a missing
Rust toolchain or linker.

**4.5.** No. Rust source for the compiler was compiled into the native
`orangec` executable. An Orange source file is a separate input processed
by that tool. This baseline reference-evaluates its supported source; the
successful Rust build does not add Orange native-code generation.

**4.6.** A clearly labeled hypothetical record could name revision R,
compiler version V, working directory `/study/orange-source`, saved input
`../trial.or`, the complete `eval` command, status zero, printed output
`trial::answer: Word[8] = 0x0d`, and the hand-derived expectation thirteen.
R and V here are placeholders in an exercise answer, not real observed
revision identifiers. A real record must supply its actual values.

**4.7.** Zero reports that the command finished according to its success
convention. A listing command and a source checker have different success
conditions. Neither becomes a proof of secrecy by returning zero.

### Chapter 5 answers

**5.1.** The semicolon ends the edition declaration. Empty parentheses
state that `answer` takes no parameters. The arrow introduces its result
type; brackets supply the width eight to `Word`. The inner braces enclose
the function body, and the outer braces enclose the module body.

**5.2.** `x` receives `0xa6`, or `10100110`; `k` receives `0x3c`, or
`00111100`. Their XOR is `10011010`, or `0x9a`. Applying `0x3c` again
restores `10100110`, or `0xa6`.

**5.3.** Rename `x` in the parameter declaration and body to `value`, and
`k` in both places to `mask`. The body becomes `value ^ mask`. The types,
comma, colons, arrow, parentheses and braces do not change. This is a
consistent renaming within scope, not a different operation.

**5.4.** One choice is `spec zero() -> Word[8] { apply(0xa6, 0xa6) }`
inside the module. Equal bits XOR to zero at every position, so the
byte result is `0x00`. This fragment needs the surrounding edition,
module and `apply` declaration from Listing 5.2.

**5.5.** Removing a required closing brace violates the grammar. The
literal 256 is outside `Word[8]`'s admitted literal range. Replacing 13
with 12 still supplies an allowed value, but it does not satisfy an
external requirement that the answer be thirteen.

**5.6.** `masks` is the module; `recovered` is the function; `Word[8]`
is the result type; `0x0b` is the displayed value eleven. That line
reports one parameterless computation containing particular calls.
It does not enumerate every argument pair accepted by `apply`.

**5.7.** A comment contributes a statement of claimed intent, not evidence
that the body implements it. An authentication claim needs an identified
construction, an acceptance rule, the protected message and authorized
sources, an adversary model, key assumptions and supporting reasoning or
other appropriately scoped evidence.

**5.8.** The declared name and qualified display name change. The body's
calculation and its evidentiary status do not. Naming a function
`proved_answer` creates no proof object or proof-checking result.

### Chapter 6 answers

**6.1.** `259 = 1 × 256 + 3`; `511 = 1 × 256 + 255`;
`512 = 2 × 256 + 0`; `-2 = -1 × 256 + 254`. The representatives are
3, 255, 0 and 254, each in the required range.

**6.2.** Literal admission requires the literal itself to fit. The
expression instead uses two individually admitted literals and an
addition operation defined to return the reduced word value. That
value is zero. These are different semantic rules, not an exception
invented after a failed test.

**6.3.** The byte result is four. The sixteen-bit and `Int` results both
have value 260. A `Word[16]` still has a fixed domain and wrapping
arithmetic, while `Int` denotes mathematical integers without fixed-width
wraparound. Equal values in this example do not erase those differences.

**6.4.** `0xa5` is `10100101`. Left shift gives `01001010` (`0x4a`);
right shift gives `01010010` (`0x52`); left rotation gives `01001011`
(`0x4b`); right rotation gives `11010010` (`0xd2`). The departing one
is discarded by a shift and retained by a rotation.

**6.5.** `0x01` and `0x81` both shift left by one to `0x02` at width
eight. Their differing most significant bit is discarded. The result
does not contain a choice between those two originals.

**6.6.** Write the computed boundary amount as `(8)`, not bare `8`.
Left rotations by zero and eight both give `0xa5`; by nine,
`0x4b`. Left shift by eight gives `0x00` under the implemented S3r
amount semantics. That boundary behavior must not be inferred for an
unidentified older compiler or another language.

**6.7.** With multiplier two, inputs zero and 128 both produce zero
modulo 256. The multiplier is nonzero, but the transformation is not
one-to-one, so the result and multiplier do not uniquely recover an
input.

**6.8.** `(1 + 1) ^ 1` yields three; `1 + (1 ^ 1)` yields one.
The ungrouped form fails to specify which of these meanings is intended
under Orange's operator-family rules.

**6.9.** Starting at zero, add seven to obtain `0x07`; XOR `0x3c` to
obtain `0x3b`; rotate left to obtain `0x76`. Reverse by rotating right
to `0x3b`, XORing to `0x07`, and subtracting seven to obtain zero.

**6.10.** The mathematical argument establishes reversal for every
byte under the stated word operations. The observed run establishes
what the identified tool reported for that saved example in that run.
Neither alone establishes confidentiality or compiler correctness.
Neither qualifies the construction for production use.

**6.11.** The forward result is `0x7a`. Subtracting seven first gives
`0x73`, which does not undo the last forward operation, the rotation.
XOR then gives `0x4f`; rotating right gives `0xa7`, not `0xfa`.
The inverse went wrong in its first step, not only in its final result.

**6.12.** A rotation moves every bit by the same number of positions around
the word, so each original one occupies exactly one resulting position.
None is created or destroyed. The converse is false. Exchanging only the
two most significant bits preserves the count of ones, but it is not a
rotation. On `10100000` the exchange yields `01100000`. The eight rotations
of `10100000` are `10100000`, `01000001`, `10000010`, `00000101`,
`00001010`, `00010100`, `00101000`, and `01010000`. The exchanged byte is
not among them. Some other bytes hide the difference: exchanging the same
two bits of `10000000` yields `01000000`, which is one of its rotations.
One agreeing byte does not make the operations the same.

## Sources and epigraph record

**[S4] Bruce Schneier.** “The Process of Security,” *Information Security*,
April 2000. The seven-word quotation is the opening sentence of the third
paragraph of the essay on the author's page, the paragraph immediately
before the heading “Will We Ever Learn?”. Wording and context checked
2026-10-05. It concerns security as continuing practice, not a claim that
the shell setup in this book guarantees security.

<https://www.schneier.com/essays/archives/2000/04/the_process_of_secur.html>

**[S5] Alan M. Turing.** “Computing Machinery and Intelligence,” *Mind*
59(236), 1950, pp. 433–460. The nineteen-word quotation is the concluding
sentence. Checked visually on the final page of the 22-page university-
hosted transcription on 2026-10-05. The transcription's first-line volume
number is erroneous; bibliographic volume 59 belongs to the original.
The publisher's record confirms that volume and pagination.
No translation or alteration of the quoted sentence is involved.

<https://www.csee.umbc.edu/courses/471/papers/turing.pdf>

<https://doi.org/10.1093/mind/LIX.236.433>

**[S6] Ronald L. Rivest.** “The RC5 Encryption Algorithm,” *Fast Software
Encryption*, proceedings of the 1994 Leuven workshop, published 1995,
pp. 86–96. Ten quoted words from the introduction's simplicity objective
on printed p. 87, checked visually against the author-hosted paper, PDF
page 2, on 2026-10-05. That objective follows the objectives for a
symmetric cipher, hardware or software, speed, adaptable word length, a
variable number of rounds, and a variable-length key. It is not the first
objective in the list. The historical design objective is not contemporary
security guidance or an endorsement of RC5 deployment.

<https://people.csail.mit.edu/rivest/pubs/Riv94.pdf>

Epigraph verification establishes wording and attribution, not publication-
rights clearance. The surrounding lessons, worked examples and exercises
are original drafting for this book, not adaptations of those papers.

**[O1] Orange compiler guide.** `compiler/README.md` at baseline commit
`21ae40f77b691099b41ee22990bad3322350eb46`, especially “Run,” “Identify the
compiler on your path,” and the implemented-slice boundary. The baseline
has implemented proposals that are still in owner review.

**[O2] Orange source and literal grammar.** `docs/LANGUAGE_2026.md`,
`docs/SEMANTICS_2026.md`, and the CLI/literal conformance tests at the
same baseline. The book changes neither the grammar nor its acceptance
status.

**[O3] Orange pure-expression semantics.** `docs/EXPRESSIONS_2026.md`
and the implemented S3b conformance evidence at the same baseline:
word widths, typed literals, calls, operator families, integer and
word arithmetic, shifts and rotations. Later amount rules are covered
by [O4] rather than silently attributed to the original slice.

**[O4] Orange computed-amount semantics.** `docs/AMOUNTS_2026.md` and
implemented S3r conformance evidence at the same baseline. These distinguish
guarded literal amounts from computed amounts,
including large and negative values. Listing 6.3 uses explicitly grouped
computed boundary amounts; it does not discard the retained literal guard.

**[T1] GNU Bash.** The installed Bash built-in help for `cd`, `pwd` and
`echo`, together with an isolated local check of directory commands,
quoted paths and `$?`, were consulted on 2026-10-05. The online reference
manual could not be retrieved in this session and is linked for further
study, not represented as inspected. The chapter uses a POSIX-style shell
track and makes no claim of PowerShell command equivalence.

<https://www.gnu.org/software/bash/manual/bash.html>

**[T2] IETF RFC 3629.** “UTF-8, a transformation format of ISO 10646,”
November 2003, especially §3. The introductory source uses the ASCII
subset of UTF-8; this is not a complete chapter on text encoding.

<https://www.rfc-editor.org/rfc/rfc3629>

**[T3] Rust project.** *The Rust Programming Language*, “Installation.”
Consulted 2026-10-05. Follow its platform-specific prerequisites rather
than interpreting one command track as a tested installer for every OS.

<https://doc.rust-lang.org/book/ch01-01-installation.html>

**[T4] Rust project.** *The Cargo Book*, `cargo build` reference, and
*The rustup book*, toolchain overrides. Consulted 2026-10-05. These
support build-flag and toolchain-pin explanations, not an assertion
that every reader's environment has been tested.

<https://doc.rust-lang.org/cargo/commands/cargo-build.html>

<https://rust-lang.github.io/rustup/overrides.html>

**[T5] Git project.** `git-clone`, `git-checkout` and `git-rev-parse`
reference manuals. Consulted 2026-10-05. A pinned revision identifies
source but does not by itself authenticate its origin.

<https://git-scm.com/docs/git-clone>

<https://git-scm.com/docs/git-checkout>

<https://git-scm.com/docs/git-rev-parse>

**[M1] Euclidean division.** The existence and uniqueness argument is
provided in §6.3. It uses the ordinary ordered integers and a positive
modulus. Negative values use a nonnegative remainder; readers must not
substitute another language's `%` convention without checking it.

## Evidence boundary

The opening in `NOVICE_OPENING.md` remains byte-for-byte unchanged from
the approved installment. The continuation adds three chapters, nine
complete Orange listings (seven valid and two deliberately rejected),
and 27 exercises with worked answers.

Expected-output blocks are expectations until supported by a recorded
execution of the exact listings. The accompanying Rust integration test
reads the listing blocks from this manuscript and invokes the actual
`orangec` binary. A separate local Python check covers arithmetic,
exercise numbering and document structure; it is not an Orange interpreter.
The delivery validation record distinguishes executed checks from checks
awaiting a build environment. No general compiler proof, secrecy claim,
independent review or platform-wide installation validation is implied.

The new drafting and tests are AI-assisted with ChatGPT (GPT-6 Astra Pro)
at Chase Bryan's direction, 2026-10-05. The previous installment received
owner approval; the newly added prose and implementation checks require
review. The project's Current/Directed/Proposed/Future distinctions,
legal boundaries and original manuscript source disclosures remain in
force. This is a continuation of the same Orange Book, not a separate
beginner product.
