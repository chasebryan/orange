//! The notebook: Markdown notes kept beside the workspace.
//!
//! Notes live in `<workspace>/.tabula/notes/<name>.md`, so they travel with
//! the sources they discuss and can be committed with them. A note can cite a
//! place in the code with a `tabula:` link such as
//! `[round constants](tabula:chacha.or#spec.quarter_round)`; the notebook
//! index reports those citations so the editor can mark cited declarations.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::json::Json;
use crate::paths::{self, EntryKind, PathError};
use crate::workspace::{private_dir, read_text, trash_dir};

/// Largest note Tabula stores.
pub const MAX_NOTE_BYTES: usize = 1024 * 1024;

/// Most notes listed.
pub const MAX_NOTES: usize = 1000;

/// Longest note name, in bytes.
pub const MAX_NOTE_NAME_BYTES: usize = 80;

/// Most citations reported per note.
pub const MAX_CITATIONS_PER_NOTE: usize = 256;

/// The notebook for one workspace.
#[derive(Clone, Debug)]
pub struct Notebook {
    workspace_root: PathBuf,
}

impl Notebook {
    /// The notebook for the workspace rooted at `workspace_root`.
    #[must_use]
    pub fn new(workspace_root: &Path) -> Self {
        Self {
            workspace_root: workspace_root.to_path_buf(),
        }
    }

    fn dir(&self) -> Result<PathBuf, PathError> {
        private_dir(&self.workspace_root, &["notes"])
    }

    fn existing_dir(&self) -> Option<PathBuf> {
        let parent = self.workspace_root.join(".tabula");
        let dir = parent.join("notes");
        let real = matches!(paths::entry_kind(&parent), Ok(EntryKind::Dir))
            && matches!(paths::entry_kind(&dir), Ok(EntryKind::Dir));
        real.then_some(dir)
    }

    /// Lists notes, most recently changed first, with their citations.
    #[must_use]
    pub fn list(&self) -> Json {
        let mut notes: Vec<(u64, String, String, Vec<Json>)> = Vec::new();
        if let Some(dir) = self.existing_dir()
            && let Ok(entries) = fs::read_dir(&dir)
        {
            for entry in entries.flatten().take(MAX_NOTES.saturating_mul(2)) {
                let Ok(file_name) = entry.file_name().into_string() else {
                    continue;
                };
                let Some(name) = file_name.strip_suffix(".md") else {
                    continue;
                };
                if validate_name(name).is_err() {
                    continue;
                }
                let Ok(file_type) = entry.file_type() else {
                    continue;
                };
                if !file_type.is_file() {
                    continue;
                }
                let modified = entry
                    .metadata()
                    .ok()
                    .and_then(|metadata| metadata.modified().ok())
                    .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
                    .map_or(0, |elapsed| elapsed.as_secs());
                let text = paths::confine(&dir, &entry.path())
                    .and_then(|path| read_text(&path, MAX_NOTE_BYTES))
                    .unwrap_or_default();
                let title = note_title(&text).unwrap_or(name).to_owned();
                let citations = citations(&text)
                    .into_iter()
                    .map(|(path, anchor)| {
                        Json::object([("path", Json::str(path)), ("anchor", Json::str(anchor))])
                    })
                    .collect();
                notes.push((modified, name.to_owned(), title, citations));
                if notes.len() >= MAX_NOTES {
                    break;
                }
            }
        }
        notes.sort_by(|left, right| right.0.cmp(&left.0).then_with(|| left.1.cmp(&right.1)));
        Json::object([
            ("ok", Json::Bool(true)),
            (
                "notes",
                Json::Array(
                    notes
                        .into_iter()
                        .map(|(modified, name, title, citations)| {
                            Json::object([
                                ("name", Json::str(name)),
                                ("title", Json::str(title)),
                                ("modified", Json::Number(modified)),
                                ("citations", Json::Array(citations)),
                            ])
                        })
                        .collect(),
                ),
            ),
        ])
    }

    /// Reads a note.
    ///
    /// # Errors
    ///
    /// Returns an error when the name is invalid or the note is missing.
    pub fn read(&self, name: &str) -> Result<String, PathError> {
        validate_name(name)?;
        let dir = self.existing_dir().ok_or(PathError::NotFound)?;
        let path = dir.join(format!("{name}.md"));
        require_note_file(&path)?;
        read_text(&path, MAX_NOTE_BYTES)
    }

    /// Saves an existing note.
    ///
    /// # Errors
    ///
    /// Returns an error when the name is invalid, the note is missing, or the
    /// text is too large.
    pub fn save(&self, name: &str, text: &str) -> Result<(), PathError> {
        validate_name(name)?;
        if text.len() > MAX_NOTE_BYTES {
            return Err(PathError::Invalid("note is larger than 1 MiB"));
        }
        let dir = self.dir()?;
        let path = dir.join(format!("{name}.md"));
        require_note_file(&path)?;
        paths::write_atomically(&path, text.as_bytes()).map_err(PathError::Io)
    }

    /// Creates a note. With no requested name, picks `Note 1`, `Note 2`, ….
    /// Returns the chosen name.
    ///
    /// # Errors
    ///
    /// Returns an error when the name is invalid or taken.
    pub fn create(&self, requested: Option<&str>, text: &str) -> Result<String, PathError> {
        if text.len() > MAX_NOTE_BYTES {
            return Err(PathError::Invalid("note is larger than 1 MiB"));
        }
        let dir = self.dir()?;
        let candidates: Vec<String> = match requested {
            Some(name) => {
                validate_name(name)?;
                vec![name.to_owned()]
            }
            None => (1..=MAX_NOTES)
                .map(|index| format!("Note {index}"))
                .collect(),
        };
        for name in candidates {
            let path = dir.join(format!("{name}.md"));
            match fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&path)
            {
                Ok(mut file) => {
                    use std::io::Write as _;
                    file.write_all(text.as_bytes())?;
                    file.sync_all()?;
                    return Ok(name);
                }
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
                Err(error) => return Err(PathError::Io(error)),
            }
        }
        Err(PathError::Exists)
    }

    /// Renames a note.
    ///
    /// # Errors
    ///
    /// Returns an error when either name is invalid, the note is missing, or
    /// the new name is taken.
    pub fn rename(&self, from: &str, to: &str) -> Result<(), PathError> {
        validate_name(from)?;
        validate_name(to)?;
        let dir = self.existing_dir().ok_or(PathError::NotFound)?;
        let source = dir.join(format!("{from}.md"));
        let target = dir.join(format!("{to}.md"));
        require_note_file(&source)?;
        if from == to {
            return Ok(());
        }
        if paths::entry_kind(&target)? != EntryKind::Missing && !paths::same_entry(&source, &target)
        {
            return Err(PathError::Exists);
        }
        fs::rename(&source, &target).map_err(PathError::Io)
    }

    /// Moves a note into `.tabula/trash/`.
    ///
    /// # Errors
    ///
    /// Returns an error when the name is invalid or the note is missing.
    pub fn delete(&self, name: &str) -> Result<(), PathError> {
        validate_name(name)?;
        let dir = self.existing_dir().ok_or(PathError::NotFound)?;
        let source = dir.join(format!("{name}.md"));
        require_note_file(&source)?;
        let trash = trash_dir(&self.workspace_root)?;
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |elapsed| elapsed.as_secs());
        let mut attempt = 0_u32;
        loop {
            let target = trash.join(format!("{stamp}-{attempt}-note-{name}.md"));
            if matches!(paths::entry_kind(&target), Ok(EntryKind::Missing)) {
                return fs::rename(&source, &target).map_err(PathError::Io);
            }
            attempt = attempt.saturating_add(1);
            if attempt > 1000 {
                return Err(PathError::Exists);
            }
        }
    }
}

/// Requires `path` to be a regular note file, not a link or a folder.
fn require_note_file(path: &Path) -> Result<(), PathError> {
    match paths::entry_kind(path)? {
        EntryKind::File => Ok(()),
        EntryKind::Missing => Err(PathError::NotFound),
        EntryKind::Dir | EntryKind::Link | EntryKind::Other => {
            Err(PathError::Invalid("note is not a regular file"))
        }
    }
}

/// Checks a note name: letters, digits, spaces, `-`, `_`, and `.`, starting
/// with a letter or digit, not ending in a space or dot.
///
/// # Errors
///
/// Returns [`PathError::Invalid`] for any other name.
pub fn validate_name(name: &str) -> Result<(), PathError> {
    let first_ok = name
        .chars()
        .next()
        .is_some_and(|c| c.is_ascii_alphanumeric());
    let chars_ok = name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, ' ' | '-' | '_' | '.'));
    if name.is_empty()
        || name.len() > MAX_NOTE_NAME_BYTES
        || !first_ok
        || !chars_ok
        || name.ends_with(' ')
        || name.ends_with('.')
        || name.contains("..")
    {
        return Err(PathError::Invalid(
            "note names use letters, digits, spaces, '-', '_' and '.', up to 80 characters",
        ));
    }
    Ok(())
}

/// The text of the first `# ` heading, if any.
#[must_use]
pub fn note_title(text: &str) -> Option<&str> {
    text.lines()
        .find_map(|line| line.strip_prefix("# "))
        .map(str::trim)
        .filter(|title| !title.is_empty())
}

/// Extracts `(path, anchor)` pairs from `](tabula:path#anchor)` links.
#[must_use]
pub fn citations(text: &str) -> Vec<(String, String)> {
    let mut found = Vec::new();
    let mut rest = text;
    while let Some(start) = rest.find("](tabula:") {
        let after = rest.get(start.saturating_add(9)..).unwrap_or("");
        let Some(end) = after.find(')') else {
            break;
        };
        let target = after.get(..end).unwrap_or("");
        let (path, anchor) = target.split_once('#').unwrap_or((target, ""));
        if !path.is_empty()
            && path.len() <= 1024
            && anchor.len() <= 256
            && !target.contains(char::is_whitespace)
        {
            found.push((path.to_owned(), anchor.to_owned()));
            if found.len() >= MAX_CITATIONS_PER_NOTE {
                break;
            }
        }
        rest = after.get(end..).unwrap_or("");
    }
    found
}

#[cfg(test)]
mod tests {
    use super::{Notebook, citations, note_title, validate_name};
    use crate::paths::PathError;
    use std::fs;

    fn notebook(name: &str) -> (Notebook, std::path::PathBuf) {
        let root = fs::canonicalize(crate::test_dir(&format!("notes-{name}"))).unwrap();
        (Notebook::new(&root), root)
    }

    #[test]
    fn validates_names() {
        for good in ["Note 1", "round-constants", "attack_ideas.v2", "a"] {
            assert!(validate_name(good).is_ok(), "{good}");
        }
        for bad in [
            "",
            " x",
            ".x",
            "x.",
            "x ",
            "../x",
            "a/b",
            "a\\b",
            "é",
            "a..b",
            &"n".repeat(81),
        ] {
            assert!(validate_name(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn finds_titles_and_citations() {
        let text = "intro\n# Round constants\nSee [q](tabula:chacha.or#spec.quarter_round) and \
                    [line](tabula:a/b.or#L12), not [web](https://x) or [bad](tabula:has space.or).";
        assert_eq!(note_title(text), Some("Round constants"));
        assert_eq!(
            citations(text),
            vec![
                ("chacha.or".to_owned(), "spec.quarter_round".to_owned()),
                ("a/b.or".to_owned(), "L12".to_owned()),
            ]
        );
        assert_eq!(note_title("no heading"), None);
    }

    #[test]
    fn creates_saves_lists_renames_and_trashes() {
        let (book, root) = notebook("crud");
        assert!(book.list().render().contains(r#""notes":[]"#));
        let first = book.create(None, "# First\n").unwrap();
        assert_eq!(first, "Note 1");
        let second = book.create(None, "").unwrap();
        assert_eq!(second, "Note 2");
        assert!(matches!(
            book.create(Some("Note 1"), ""),
            Err(PathError::Exists)
        ));
        book.save("Note 1", "# First\n[x](tabula:a.or#L1)\n")
            .unwrap();
        assert_eq!(
            book.read("Note 1").unwrap(),
            "# First\n[x](tabula:a.or#L1)\n"
        );
        assert!(matches!(
            book.save("Missing", "x"),
            Err(PathError::NotFound)
        ));
        let listed = book.list().render();
        assert!(listed.contains(r#""title":"First""#));
        assert!(listed.contains(r#""path":"a.or","anchor":"L1""#));
        book.rename("Note 1", "Round constants").unwrap();
        assert!(matches!(
            book.rename("Note 2", "Round constants"),
            Err(PathError::Exists)
        ));
        assert_eq!(
            book.read("Round constants").unwrap(),
            "# First\n[x](tabula:a.or#L1)\n"
        );
        book.delete("Round constants").unwrap();
        assert!(matches!(
            book.read("Round constants"),
            Err(PathError::NotFound)
        ));
        let trashed = fs::read_dir(root.join(".tabula/trash"))
            .unwrap()
            .filter(|entry| {
                !entry
                    .as_ref()
                    .unwrap()
                    .file_name()
                    .to_string_lossy()
                    .starts_with('.')
            })
            .count();
        assert_eq!(trashed, 1);
        assert!(matches!(
            book.read("../../etc/passwd"),
            Err(PathError::Invalid(_))
        ));
        fs::remove_dir_all(&root).unwrap();
    }
}
