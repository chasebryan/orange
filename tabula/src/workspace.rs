//! The workspace: the folder of Orange sources that Tabula edits.
//!
//! Only `.or` files are listed, read, or written. Folders are listed when they
//! contain Orange sources (or nothing at all, so a new folder stays visible).
//! Hidden entries, `target`, and `node_modules` are never walked. Deleting
//! moves the entry into `.tabula/trash/` instead of destroying it.

use std::fs;
use std::io::{self, Read as _};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::json::Json;
use crate::paths::{self, EntryKind, PathError, RelativePath};

/// Largest Orange source Tabula reads or writes, matching `orangec`'s
/// per-source limit.
pub const MAX_SOURCE_BYTES: usize = 16 * 1024 * 1024;

/// Most directory entries visited when listing the workspace.
pub const MAX_TREE_ENTRIES: usize = 20_000;

/// Deepest folder level listed.
pub const MAX_TREE_DEPTH: usize = 16;

/// The Orange source file extension.
pub const ORANGE_SUFFIX: &str = ".or";

const SKIPPED_FOLDERS: [&str; 2] = ["target", "node_modules"];

/// A workspace rooted at a canonical directory.
#[derive(Clone, Debug)]
pub struct Workspace {
    root: PathBuf,
}

impl Workspace {
    /// Opens the workspace at `root`.
    ///
    /// # Errors
    ///
    /// Returns an error when `root` is not an existing directory.
    pub fn open(root: &Path) -> Result<Self, PathError> {
        Ok(Self {
            root: paths::canonical_root(root)?,
        })
    }

    /// The canonical root directory.
    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// The root folder's own name, for display.
    #[must_use]
    pub fn name(&self) -> String {
        self.root.file_name().map_or_else(
            || self.root.display().to_string(),
            |name| name.to_string_lossy().into_owned(),
        )
    }

    /// Lists folders and Orange sources as a JSON tree.
    #[must_use]
    pub fn tree(&self) -> Json {
        let mut budget = MAX_TREE_ENTRIES;
        let mut truncated = false;
        let children = walk(&self.root, "", 0, &mut budget, &mut truncated);
        Json::object([
            ("ok", Json::Bool(true)),
            ("name", Json::str(self.name())),
            ("children", Json::Array(children.unwrap_or_default())),
            ("truncated", Json::Bool(truncated)),
        ])
    }

    /// Reads an Orange source.
    ///
    /// # Errors
    ///
    /// Returns an error when the path is invalid, not an `.or` file, too large,
    /// or not UTF-8.
    pub fn read(&self, raw: &str) -> Result<String, PathError> {
        let path = orange_path(raw)?;
        let resolved = paths::resolve_existing(&self.root, &path)?;
        read_text(&resolved, MAX_SOURCE_BYTES)
    }

    /// Replaces an existing Orange source, or creates it if `create` is set.
    ///
    /// # Errors
    ///
    /// Returns an error when the path is invalid, the text is too large, or
    /// the file is missing (for a save) or present (for a create).
    pub fn write(&self, raw: &str, text: &str, create: bool) -> Result<(), PathError> {
        let path = orange_path(raw)?;
        if text.len() > MAX_SOURCE_BYTES {
            return Err(PathError::Invalid("source is larger than 16 MiB"));
        }
        let target = paths::resolve_for_write(&self.root, &path)?;
        let exists = paths::entry_kind(&target)? != EntryKind::Missing;
        if create && exists {
            return Err(PathError::Exists);
        }
        if !create && !exists {
            return Err(PathError::NotFound);
        }
        if create {
            use std::io::Write as _;
            let mut file = fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&target)
                .map_err(|error| {
                    if error.kind() == io::ErrorKind::AlreadyExists {
                        PathError::Exists
                    } else {
                        PathError::Io(error)
                    }
                })?;
            file.write_all(text.as_bytes())?;
            file.sync_all()?;
            Ok(())
        } else {
            paths::write_atomically(&target, text.as_bytes()).map_err(PathError::Io)
        }
    }

    /// Creates a folder.
    ///
    /// # Errors
    ///
    /// Returns an error when the path is invalid or already exists.
    pub fn create_folder(&self, raw: &str) -> Result<(), PathError> {
        let path = RelativePath::parse(raw)?;
        if paths::resolve_existing(&self.root, &path).is_ok() {
            return Err(PathError::Exists);
        }
        let target = paths::resolve_for_write(&self.root, &path)?;
        fs::create_dir(&target).map_err(|error| {
            if error.kind() == io::ErrorKind::AlreadyExists {
                PathError::Exists
            } else {
                PathError::Io(error)
            }
        })
    }

    /// Renames a source or folder. Sources must keep the `.or` suffix.
    ///
    /// # Errors
    ///
    /// Returns an error when either path is invalid, the source is missing, or
    /// the destination exists.
    pub fn rename(&self, from: &str, to: &str) -> Result<(), PathError> {
        let from_path = RelativePath::parse(from)?;
        let to_path = RelativePath::parse(to)?;
        let source = paths::resolve_existing(&self.root, &from_path)?;
        let kind = paths::entry_kind(&source)?;
        if kind == EntryKind::File
            && !(from_path.has_suffix(ORANGE_SUFFIX) && to_path.has_suffix(ORANGE_SUFFIX))
        {
            return Err(PathError::Invalid("Orange sources must end in .or"));
        }
        if kind != EntryKind::File && kind != EntryKind::Dir {
            return Err(PathError::Invalid("only files and folders can be renamed"));
        }
        let target = paths::resolve_for_write(&self.root, &to_path)?;
        if paths::entry_kind(&target)? != EntryKind::Missing && !paths::same_entry(&source, &target)
        {
            return Err(PathError::Exists);
        }
        if kind == EntryKind::Dir && target.starts_with(&source) {
            return Err(PathError::Invalid("a folder cannot move inside itself"));
        }
        fs::rename(&source, &target).map_err(PathError::Io)
    }

    /// Moves a source or folder into `.tabula/trash/`.
    ///
    /// # Errors
    ///
    /// Returns an error when the path is invalid or missing.
    pub fn delete(&self, raw: &str) -> Result<String, PathError> {
        let path = RelativePath::parse(raw)?;
        let source = paths::resolve_existing(&self.root, &path)?;
        if paths::entry_kind(&source)? == EntryKind::File && !path.has_suffix(ORANGE_SUFFIX) {
            return Err(PathError::Invalid(
                "only Orange sources can be deleted here",
            ));
        }
        let trash = trash_dir(&self.root)?;
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |elapsed| elapsed.as_secs());
        let mut attempt = 0_u32;
        loop {
            let name = if attempt == 0 {
                format!("{stamp}-{}", path.file_name())
            } else {
                format!("{stamp}-{attempt}-{}", path.file_name())
            };
            let target = trash.join(&name);
            if matches!(paths::entry_kind(&target), Ok(EntryKind::Missing)) {
                fs::rename(&source, &target)?;
                return Ok(format!(".tabula/trash/{name}"));
            }
            attempt = attempt.saturating_add(1);
            if attempt > 1000 {
                return Err(PathError::Exists);
            }
        }
    }
}

/// Parses a path that must name an Orange source.
fn orange_path(raw: &str) -> Result<RelativePath, PathError> {
    let path = RelativePath::parse(raw)?;
    if !path.has_suffix(ORANGE_SUFFIX) {
        return Err(PathError::Invalid("Tabula only opens Orange sources (.or)"));
    }
    Ok(path)
}

/// Returns (creating as needed) a directory under `<root>/.tabula/`, refusing
/// to follow a symbolic link at any level.
///
/// # Errors
///
/// Returns an error when a level exists but is not a real directory.
pub fn private_dir(root: &Path, levels: &[&str]) -> Result<PathBuf, PathError> {
    let mut current = root.join(".tabula");
    ensure_real_dir(&current)?;
    for level in levels {
        current.push(level);
        ensure_real_dir(&current)?;
    }
    Ok(current)
}

/// Opens `.tabula/trash/`, creating it with a `.gitignore` that keeps
/// deleted files out of version control.
///
/// # Errors
///
/// Returns an error when the folder cannot be made or is not a plain folder.
pub fn trash_dir(root: &Path) -> Result<PathBuf, PathError> {
    let trash = private_dir(root, &["trash"])?;
    match fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(trash.join(".gitignore"))
    {
        Ok(mut file) => io::Write::write_all(
            &mut file,
            b"# Tabula's trash: deleted sources and notes.\n*\n",
        )?,
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
        Err(error) => return Err(PathError::Io(error)),
    }
    Ok(trash)
}

fn ensure_real_dir(path: &Path) -> Result<(), PathError> {
    let kind = match paths::entry_kind(path)? {
        EntryKind::Missing => match fs::create_dir(path) {
            Ok(()) => return Ok(()),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => paths::entry_kind(path)?,
            Err(error) => return Err(PathError::Io(error)),
        },
        kind => kind,
    };
    if kind == EntryKind::Dir {
        Ok(())
    } else {
        Err(PathError::Invalid(
            "Tabula's private folder is not a plain folder",
        ))
    }
}

/// Reads a regular file as UTF-8 text of at most `limit` bytes.
///
/// # Errors
///
/// Returns an error when the file is not regular, too large, or not UTF-8.
pub fn read_text(path: &Path, limit: usize) -> Result<String, PathError> {
    let file = match fs::File::open(path) {
        Ok(file) => file,
        Err(error) => return Err(PathError::from(error)),
    };
    match file.metadata() {
        Ok(metadata) if metadata.is_file() => {}
        Ok(_) => return Err(PathError::Invalid("not a regular file")),
        Err(error) => return Err(PathError::from(error)),
    }
    let mut bytes = Vec::new();
    let cap = u64::try_from(limit).map_or(u64::MAX, |value| value.saturating_add(1));
    file.take(cap).read_to_end(&mut bytes)?;
    if bytes.len() > limit {
        return Err(PathError::Invalid("file is too large"));
    }
    String::from_utf8(bytes).map_err(|_| PathError::Invalid("file is not UTF-8 text"))
}

fn walk(
    dir: &Path,
    prefix: &str,
    depth: usize,
    budget: &mut usize,
    truncated: &mut bool,
) -> Option<Vec<Json>> {
    let Ok(entries) = fs::read_dir(dir) else {
        return Some(Vec::new());
    };
    let mut folders: Vec<(String, PathBuf)> = Vec::new();
    let mut files: Vec<String> = Vec::new();
    for entry in entries.flatten() {
        if *budget == 0 {
            *truncated = true;
            break;
        }
        *budget = budget.saturating_sub(1);
        let Ok(name) = entry.file_name().into_string() else {
            continue;
        };
        if name.starts_with('.') || RelativePath::parse(&name).is_err() {
            continue;
        }
        let Ok(file_type) = entry.file_type() else {
            continue;
        };
        if file_type.is_dir() {
            if !SKIPPED_FOLDERS.contains(&name.as_str())
                && let Ok(path) = paths::confine(dir, &entry.path())
            {
                folders.push((name, path));
            }
        } else if file_type.is_file()
            && RelativePath::parse(&name).is_ok_and(|path| path.has_suffix(ORANGE_SUFFIX))
        {
            files.push(name);
        }
    }
    folders.sort_by_key(|(name, _)| natural_key(name));
    files.sort_by_key(|name| natural_key(name));

    let mut children = Vec::new();
    for (name, path) in folders {
        let relative = join(prefix, &name);
        if depth >= MAX_TREE_DEPTH {
            *truncated = true;
            continue;
        }
        let nested =
            walk(&path, &relative, depth.saturating_add(1), budget, truncated).unwrap_or_default();
        let empty = fs::read_dir(&path)
            .map(|mut entries| entries.next().is_none())
            .unwrap_or(false);
        if nested.is_empty() && !empty {
            continue;
        }
        children.push(Json::object([
            ("kind", Json::str("folder")),
            ("name", Json::str(name)),
            ("path", Json::str(relative)),
            ("children", Json::Array(nested)),
        ]));
    }
    for name in files {
        let relative = join(prefix, &name);
        children.push(Json::object([
            ("kind", Json::str("file")),
            ("name", Json::str(name)),
            ("path", Json::str(relative)),
        ]));
    }
    Some(children)
}

fn join(prefix: &str, name: &str) -> String {
    if prefix.is_empty() {
        name.to_owned()
    } else {
        format!("{prefix}/{name}")
    }
}

/// Sort key that orders `round2` before `round10`.
fn natural_key(name: &str) -> Vec<(u8, String)> {
    let mut key = Vec::new();
    let mut digits = String::new();
    let mut text = String::new();
    for character in name.chars() {
        if character.is_ascii_digit() {
            if !text.is_empty() {
                key.push((1, std::mem::take(&mut text)));
            }
            digits.push(character);
        } else {
            if !digits.is_empty() {
                key.push((0, pad_digits(&std::mem::take(&mut digits))));
            }
            text.extend(character.to_lowercase());
        }
    }
    if !digits.is_empty() {
        key.push((0, pad_digits(&digits)));
    }
    if !text.is_empty() {
        key.push((1, text));
    }
    key
}

fn pad_digits(digits: &str) -> String {
    let trimmed = digits.trim_start_matches('0');
    format!("{:0>20}", trimmed)
}

#[cfg(test)]
mod tests {
    use super::Workspace;
    use crate::paths::PathError;
    use std::fs;

    fn workspace(name: &str) -> Workspace {
        Workspace::open(&crate::test_dir(&format!("workspace-{name}"))).unwrap()
    }

    #[test]
    fn lists_only_orange_sources() {
        let ws = workspace("tree");
        let root = ws.root().to_path_buf();
        fs::create_dir_all(root.join("crypto/round")).unwrap();
        fs::create_dir_all(root.join("docs")).unwrap();
        fs::create_dir_all(root.join("target")).unwrap();
        fs::create_dir_all(root.join(".git")).unwrap();
        fs::create_dir_all(root.join("empty")).unwrap();
        fs::write(root.join("crypto/round/r10.or"), "").unwrap();
        fs::write(root.join("crypto/round/r2.or"), "").unwrap();
        fs::write(root.join("main.or"), "").unwrap();
        fs::write(root.join("notes.txt"), "").unwrap();
        fs::write(root.join("docs/readme.md"), "").unwrap();
        fs::write(root.join("target/x.or"), "").unwrap();
        fs::write(root.join(".git/x.or"), "").unwrap();
        let rendered = ws.tree().render();
        assert!(rendered.contains(r#""path":"crypto/round/r2.or""#));
        assert!(rendered.find("r2.or").unwrap() < rendered.find("r10.or").unwrap());
        assert!(rendered.contains(r#""path":"main.or""#));
        assert!(rendered.contains(r#""path":"empty""#));
        assert!(!rendered.contains("notes.txt"));
        assert!(!rendered.contains("docs"));
        assert!(!rendered.contains("target"));
        assert!(!rendered.contains(".git"));
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn reads_writes_renames_and_trashes() {
        let ws = workspace("crud");
        let root = ws.root().to_path_buf();
        ws.write("a.or", "edition 2026;\n", true).unwrap();
        assert!(matches!(
            ws.write("a.or", "x", true),
            Err(PathError::Exists)
        ));
        assert!(matches!(
            ws.write("b.or", "x", false),
            Err(PathError::NotFound)
        ));
        ws.write("a.or", "edition 2026;\nmodule m {}\n", false)
            .unwrap();
        assert_eq!(ws.read("a.or").unwrap(), "edition 2026;\nmodule m {}\n");
        assert!(matches!(ws.read("a.txt"), Err(PathError::Invalid(_))));
        assert!(matches!(
            ws.write("a.txt", "x", true),
            Err(PathError::Invalid(_))
        ));

        ws.create_folder("specs").unwrap();
        assert!(matches!(ws.create_folder("specs"), Err(PathError::Exists)));
        ws.rename("a.or", "specs/a.or").unwrap();
        assert!(matches!(
            ws.rename("specs/a.or", "specs/a.txt"),
            Err(PathError::Invalid(_))
        ));
        ws.write("b.or", "", true).unwrap();
        assert!(matches!(
            ws.rename("b.or", "specs/a.or"),
            Err(PathError::Exists)
        ));
        assert!(matches!(
            ws.rename("specs", "specs/inner"),
            Err(PathError::Invalid(_))
        ));

        let trashed = ws.delete("specs/a.or").unwrap();
        assert!(trashed.starts_with(".tabula/trash/"));
        assert!(root.join(&trashed).is_file());
        assert!(
            fs::read_to_string(root.join(".tabula/trash/.gitignore"))
                .unwrap()
                .ends_with("*\n")
        );
        assert!(matches!(ws.read("specs/a.or"), Err(PathError::NotFound)));
        assert!(matches!(
            ws.read(".tabula/trash/x.or"),
            Err(PathError::Invalid(_))
        ));

        fs::write(root.join("bad.or"), [0xff, 0xfe]).unwrap();
        assert!(matches!(ws.read("bad.or"), Err(PathError::Invalid(_))));
        fs::remove_dir_all(&root).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn refuses_links_to_hidden_entries() {
        let ws = workspace("hidden-link");
        let root = ws.root().to_path_buf();
        fs::create_dir_all(root.join(".secret")).unwrap();
        fs::write(root.join(".secret/key.or"), "k").unwrap();
        std::os::unix::fs::symlink(root.join(".secret/key.or"), root.join("key.or")).unwrap();
        assert!(matches!(ws.read("key.or"), Err(PathError::Invalid(_))));
        fs::remove_dir_all(&root).unwrap();
    }
}
