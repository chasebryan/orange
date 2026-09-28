# Tabula

Tabula is a small writing table for Orange. It is a workbench for the language,
not part of the language: a local program that opens in your web browser and
does one thing, which is to help you write Orange the way Orange is meant to be
written.

An Orange artifact has three parts: a mathematical specification, an
implementation linked to it, and evidence that can be checked. Tabula keeps
those three in view at once. You write the specification and the implementation
in one file, `orangec` checks them while you type, and each run leaves a record
that names the exact bytes it checked. The Orange Book, the manuals, and your own
notes sit beside the code.

Tabula is operated with the mouse. Every action is a labelled button, and the
only keyboard shortcut is the usual one for saving (Ctrl+S, or ⌘S on a Mac).

## Start Tabula

Tabula is a single program with no dependencies beyond the pinned Rust
toolchain. Build it and `orangec` from the root of an Orange checkout:

```sh
cargo build --release --manifest-path compiler/Cargo.toml -p orangec
cargo build --release --manifest-path tabula/Cargo.toml
```

Then open a folder of Orange sources. That folder is your workspace; an empty
folder is a fine place to start.

```sh
tabula/target/release/tabula path/to/workspace
```

Tabula prints a link such as `http://127.0.0.1:2026/?k=…` and opens it in your
browser. The `k` part is a fresh key for this session; a page without it cannot
reach your files. Keep the terminal open while you work, and close it (or press
Ctrl+C there) to stop Tabula.

| Option | Meaning |
| --- | --- |
| `WORKSPACE` | The folder to open. The default is the current folder. |
| `--port PORT` | Serve on this local port. The default is 2026; `0` picks a free one. |
| `--orangec PATH` | Use this `orangec`. |
| `--library DIR` | Read The Orange Book and the manuals from this Orange checkout. |
| `--no-open` | Print the link without opening a browser. |

Without `--orangec`, Tabula looks for `orangec` in the `ORANGEC` environment
variable, next to the `tabula` program, in the checkout's `compiler/target`
folder, and on your `PATH`, in that order. Without `--library`, it uses the
nearest Orange checkout that contains the workspace or the `tabula` program.

## The window

The window has five parts. Each can be resized by dragging its edge, and a
double click on an edge puts it back.

| Part | What it holds |
| --- | --- |
| Title bar | New file, Save, Check, Evaluate, and Cite in note. The two buttons on the right show or hide the side panel and the results panel. |
| Side panel | Files, Strata, or the Library, chosen with the buttons on the left edge. The notebook button there opens the notebook. |
| Editor | Open files and documents as tabs. A dot on a tab means unsaved changes. |
| Results panel | Problems, Values, Evidence, and Tokens for the file in front. |
| Notebook | Your notes, on the right. |

The palette button at the bottom left opens the theme picker. Click a theme to
use it at once; Tabula remembers the choice in this browser. Until you pick
one, Tabula follows your system and switches between Dark and Light with it.

| Theme | Look |
| --- | --- |
| Dark | Tabula's own dark theme, with Orange accents. |
| Light | Warm paper tones for daylight. |
| Tokyo | A deep blue night, after the Tokyo Night colours. |
| Corporate | A black frame around a white page, in greys and navy blue. Red is kept for errors and one thin line at the top. |

The information button shows where this session's files, compiler, and Library
are.

## Files

The Files view lists the Orange sources (`.or` files) in the workspace and the
folders that hold them. Hover over a row to rename or delete it, or to add a
file to a folder. New files start as a small module with one typed spec:

```orange
edition 2026;

module untitled {
  spec answer() -> Int { 42 }
}
```

Deleting never destroys anything. Tabula moves the file or folder into
`.tabula/trash/` inside the workspace, and that folder is kept out of version
control. To recover a file, move it back from there.

Tabula saves only when you ask it to: press Save, or Ctrl+S. It keeps a file's
line endings as it found them.

## Writing Orange

The editor colours Orange as `orangec` reads it: the strata keywords each have
their own colour, integers dim their radix prefix and digit separators, and
anything the lexer would reject is shown in red. Enter keeps the indentation,
and Tab indents.

The Strata view is the outline of the file in front. It groups the
declarations by stratum:

- `spec` holds the mathematical definition. A typed spec such as
  `spec rounds() -> Int { 20 }` states an exact value.
- `impl` holds the implementation. Orange 2026 accepts only empty `impl`
  functions, such as `impl quarter_round() {}`, while implementation semantics
  are designed.
- `game`, `proof`, and `claim` are reserved for later layers. `orangec` rejects
  them today, and Tabula shows them greyed out.

Click a declaration to jump to it. The Insert buttons add a new `spec` or
`impl` after the declaration under the cursor and select its name so you can
type a better one.

## Checking and evaluating

While live checks are on (the status bar says so), Tabula sends the file to
`orangec eval` a moment after you stop typing. You can also press Check, which
runs `orangec check`, or Evaluate, which runs `orangec eval`, at any time. Each
run sends the text in the editor, saved or not, to `orangec` on its standard
input. `orangec` is the only authority: Tabula shows what it reports and adds
nothing of its own.

The results panel has four views.

**Problems** lists every diagnostic, with its `ORC` code, message, and notes.
Click one to select the text it points at; the same place is underlined in the
editor, and hovering there shows the message. Look up searches the Library
for the code.

**Values** shows the exact value of each typed spec. Choose decimal,
hexadecimal, or binary; every value is shown as a valid Orange literal, digit
separators included, so it can be copied straight back into a spec. `Word[8]`
values also show their bits. The same values appear at the end of each spec's
line in the editor. After an edit, the last values stay visible, marked as
coming from before the edit, until the next successful evaluation.

**Evidence** is the record of the last run: the file, the SHA-256 of the exact
bytes that were checked, their size, the `orangec` version, and the outcome of
each phase (lexical, syntax, semantic, evaluation). Copy record puts the whole
record on the clipboard, and Add to note puts it in the open note. The record
says what this compiler reported for these bytes. It is evidence about the
pre-alpha Orange 2026 checks, not a proof that a design is correct or secure.

**Tokens** shows the token stream `orangec lex` produces for the file. Click a
token to select it in the editor.

## The Library

The Library view lists The Orange Book, the manuals (the language reference,
the semantics, the compiler, and this manual), and the rest of the checkout's
documents. Tabula reads them from the checkout every time you open one, so
what you read is always the current text of the repository.

While a document is open, the Library view shows its contents; click a heading
to go there. The search box searches every listed document. Links between
documents open in new tabs. Orange examples in a document have a button, shown
when you point at the example, that copies the example into a new file in your
workspace.

## The notebook

The notebook keeps Markdown notes in `.tabula/notes/` inside the workspace, one
file per note, so notes can be committed alongside the sources they discuss.
Notes are saved as you write. The buttons above a note add headings, bold,
italics, code, lists, and Orange code blocks; the eye button switches between
writing and a formatted preview. The name at the top is the note's file name;
edit it to rename the note.

Cite in note links the place in front of you into the open note:

- In the editor, it cites the declaration under the cursor, or the line when
  the cursor is outside a declaration. If you selected text, the selection is
  quoted below the link.
- In a document, it cites the section you are reading.

Citations are ordinary Markdown links with two kinds of address:

```text
[spec rounds](tabula:ciphers/chacha.or#spec.rounds)
[chacha.or line 12](tabula:ciphers/chacha.or#L12)
[Language reference › Integer tokens](library:docs/LANGUAGE_2026.md#23-integer-tokens)
```

In the preview, a citation opens the file or document it names. In the editor,
a small bookmark in the margin marks each line a note cites; click it to open
that note.

## Security

Tabula is built to be safe to leave running on a shared machine.

- It listens only on `127.0.0.1`, and answers only requests addressed to
  `127.0.0.1` or `localhost` on its own port, which defeats DNS rebinding.
- Every request for data must carry the session key from the launch link. The
  key is 128 random bits, compared in constant time, and never written to a
  file by Tabula. The page keeps it in the browser's storage for this address
  only, and a new session makes a new key.
- Requests from other web pages are refused, and the page runs under a strict
  Content Security Policy with no outside scripts, styles, or fonts.
- Tabula writes only `.or` files inside the workspace and notes inside
  `.tabula/notes/`, and it only reads the Library's documents and images. It
  refuses hidden paths and paths that leave their folder, including through
  symbolic links. Every write replaces the file in one step, so a crash never
  leaves half a file.
- Documents and notes are rendered by building page elements directly, so text
  can never become script. Raw HTML in Markdown is shown as text, and links
  leave Tabula only for `https:`, `http:`, and `mailto:` addresses.
- `orangec` runs with an empty environment, a 20-second time limit, and a cap
  on its output.

## For contributors

Tabula lives in `tabula/`, apart from the compiler, which stays free of
dependencies of any kind. Tabula has no third-party dependencies either: the
server is the Rust standard library, and the page is plain HTML, CSS, and
JavaScript embedded in the program at build time.

| Path | Contents |
| --- | --- |
| `src/server.rs` | The local server, its checks, and the API routes. |
| `src/http.rs` | A small HTTP/1.1 reader and writer. |
| `src/workspace.rs`, `src/paths.rs` | Workspace files and path confinement. |
| `src/notes.rs` | The notebook. |
| `src/library.rs` | The Library catalog, documents, images, and search. |
| `src/orangec.rs` | Running `orangec` and reading its output. |
| `web/` | The page: `app.js` ties together `editor.js`, `orange.js` (the Orange lexical rules), `markdown.js`, and `values.js`. |
| `tests/` | End-to-end tests that drive the server with a real `orangec`. |

The repository check builds and tests Tabula alongside the compiler. To run
Tabula's own checks:

```sh
cargo fmt --manifest-path tabula/Cargo.toml --check
cargo clippy --manifest-path tabula/Cargo.toml --all-targets --locked -- -D warnings
cargo test --manifest-path tabula/Cargo.toml --locked
```

The end-to-end tests need a built `orangec`. They use `TABULA_TEST_ORANGEC`
when it is set, and otherwise `compiler/target/release/orangec` or
`compiler/target/debug/orangec`.
