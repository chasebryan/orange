//! Confinement of browser-supplied paths to a root directory.
//!
//! Every path that arrives from the page is a `/`-separated relative path. It
//! is split into plain segments, each segment is checked on its own, and the
//! joined path is resolved against the canonical root. Existing targets are
//! canonicalized again so a symbolic link cannot lead outside the root; new
//! targets must have a canonical parent inside the root and must not already
//! be a symbolic link.

use std::fmt;
use std::fs;
use std::io;
use std::path::{Component, Path, PathBuf};

/// Longest relative path accepted, in bytes.
pub const MAX_RELATIVE_PATH_BYTES: usize = 1024;

/// Longest single path segment accepted, in bytes.
pub const MAX_SEGMENT_BYTES: usize = 255;

/// Deepest relative path accepted, in segments.
pub const MAX_DEPTH: usize = 32;

/// Why a path was refused.
#[derive(Debug)]
pub enum PathError {
    /// The path is empty, malformed, or names something outside the root.
    Invalid(&'static str),
    /// The path does not exist.
    NotFound,
    /// The path already exists.
    Exists,
    /// A filesystem operation failed.
    Io(io::Error),
}

impl fmt::Display for PathError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Invalid(reason) => formatter.write_str(reason),
            Self::NotFound => formatter.write_str("no such file"),
            Self::Exists => formatter.write_str("a file with that name already exists"),
            Self::Io(error) => write!(formatter, "{error}"),
        }
    }
}

impl From<io::Error> for PathError {
    fn from(error: io::Error) -> Self {
        if error.kind() == io::ErrorKind::NotFound {
            Self::NotFound
        } else {
            Self::Io(error)
        }
    }
}

/// A validated relative path: one or more plain, visible segments.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RelativePath {
    segments: Vec<String>,
}

impl RelativePath {
    /// Parses a `/`-separated relative path.
    ///
    /// Segments may not be empty, `.`, or `..`; may not start with `.` (hidden
    /// entries are never exposed); and may not contain `\`, `:`, control
    /// characters, or characters that Windows forbids in file names.
    ///
    /// # Errors
    ///
    /// Returns [`PathError::Invalid`] describing the first problem found.
    pub fn parse(raw: &str) -> Result<Self, PathError> {
        if raw.is_empty() {
            return Err(PathError::Invalid("path is empty"));
        }
        if raw.len() > MAX_RELATIVE_PATH_BYTES {
            return Err(PathError::Invalid("path is too long"));
        }
        if raw.starts_with('/') {
            return Err(PathError::Invalid("path must be relative"));
        }
        let mut segments = Vec::new();
        for segment in raw.split('/') {
            validate_segment(segment)?;
            if segments.len() >= MAX_DEPTH {
                return Err(PathError::Invalid("path is too deep"));
            }
            segments.push(segment.to_owned());
        }
        Ok(Self { segments })
    }

    /// The path's segments.
    #[must_use]
    pub fn segments(&self) -> &[String] {
        &self.segments
    }

    /// The final segment.
    #[must_use]
    pub fn file_name(&self) -> &str {
        self.segments.last().map_or("", String::as_str)
    }

    /// The `/`-joined form.
    #[must_use]
    pub fn as_string(&self) -> String {
        self.segments.join("/")
    }

    /// The path with the final segment removed, or `None` at the root.
    #[must_use]
    pub fn parent(&self) -> Option<Self> {
        let (_, parent) = self.segments.split_last()?;
        if parent.is_empty() {
            None
        } else {
            Some(Self {
                segments: parent.to_vec(),
            })
        }
    }

    /// Whether the final segment ends with `suffix`, ignoring ASCII case.
    #[must_use]
    pub fn has_suffix(&self, suffix: &str) -> bool {
        let name = self.file_name();
        name.len() > suffix.len()
            && name
                .get(name.len().saturating_sub(suffix.len())..)
                .is_some_and(|tail| tail.eq_ignore_ascii_case(suffix))
    }

    /// Joins the segments onto `root` without touching the filesystem.
    #[must_use]
    pub fn join_onto(&self, root: &Path) -> PathBuf {
        let mut path = root.to_path_buf();
        for segment in &self.segments {
            path.push(segment);
        }
        path
    }
}

fn validate_segment(segment: &str) -> Result<(), PathError> {
    if segment.is_empty() {
        return Err(PathError::Invalid("path has an empty segment"));
    }
    if segment == "." || segment == ".." {
        return Err(PathError::Invalid("path may not contain `.` or `..`"));
    }
    if segment.starts_with('.') {
        return Err(PathError::Invalid("hidden files and folders are not shown"));
    }
    if segment.len() > MAX_SEGMENT_BYTES {
        return Err(PathError::Invalid("a path segment is too long"));
    }
    if segment.ends_with(' ') || segment.ends_with('.') {
        return Err(PathError::Invalid(
            "names may not end with a space or a dot",
        ));
    }
    if segment
        .chars()
        .any(|c| c.is_control() || matches!(c, '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|'))
    {
        return Err(PathError::Invalid(
            "name contains a character that is not allowed",
        ));
    }
    let components: Vec<Component<'_>> = Path::new(segment).components().collect();
    if !matches!(components.as_slice(), [Component::Normal(_)]) {
        return Err(PathError::Invalid("path segment is not a plain name"));
    }
    Ok(())
}

/// Canonicalizes `root`, which must be an existing directory.
///
/// # Errors
///
/// Returns an error when `root` cannot be resolved or is not a directory.
pub fn canonical_root(root: &Path) -> Result<PathBuf, PathError> {
    let canonical = fs::canonicalize(root)?;
    if !fs::metadata(&canonical)?.is_dir() {
        return Err(PathError::Invalid("root is not a directory"));
    }
    Ok(canonical)
}

/// Resolves an existing entry under `root` (which must already be canonical),
/// following symbolic links only if the final target stays inside `root`.
///
/// # Errors
///
/// Returns [`PathError::NotFound`] if nothing exists there, or
/// [`PathError::Invalid`] if the entry resolves outside `root`.
pub fn resolve_existing(root: &Path, path: &RelativePath) -> Result<PathBuf, PathError> {
    let joined = path.join_onto(root);
    let canonical = fs::canonicalize(&joined)?;
    let inside = canonical
        .strip_prefix(root)
        .map_err(|_| PathError::Invalid("path leads outside the workspace"))?;
    let mut components = inside.components().peekable();
    if components.peek().is_none() {
        return Err(PathError::Invalid("path names the root itself"));
    }
    let visible = components.all(|component| match component {
        Component::Normal(name) => !name.to_string_lossy().starts_with('.'),
        _ => false,
    });
    if !visible {
        return Err(PathError::Invalid("path leads to a hidden entry"));
    }
    Ok(canonical)
}

/// Resolves a path under `root` whose final entry may not exist yet. The parent
/// must be an existing directory inside `root`, and an existing final entry
/// must not be a symbolic link.
///
/// # Errors
///
/// Returns [`PathError::NotFound`] if the parent does not exist, or
/// [`PathError::Invalid`] if the parent resolves outside `root` or the final
/// entry is a link or a directory.
pub fn resolve_for_write(root: &Path, path: &RelativePath) -> Result<PathBuf, PathError> {
    let parent = match path.parent() {
        Some(parent) => resolve_existing(root, &parent)?,
        None => root.to_path_buf(),
    };
    if !fs::metadata(&parent)?.is_dir() {
        return Err(PathError::Invalid("parent is not a folder"));
    }
    let target = parent.join(path.file_name());
    match fs::symlink_metadata(&target) {
        Ok(metadata) if metadata.file_type().is_symlink() => Err(PathError::Invalid(
            "refusing to write through a symbolic link",
        )),
        Ok(metadata) if metadata.is_dir() => Err(PathError::Invalid("a folder has that name")),
        Ok(_) => Ok(target),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(target),
        Err(error) => Err(PathError::Io(error)),
    }
}

/// Whether two paths name the same filesystem entry, as happens when a rename
/// only changes letter case on a case-insensitive filesystem.
#[must_use]
pub fn same_entry(left: &Path, right: &Path) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt as _;
        match (fs::symlink_metadata(left), fs::symlink_metadata(right)) {
            (Ok(a), Ok(b)) => a.dev() == b.dev() && a.ino() == b.ino(),
            _ => false,
        }
    }
    #[cfg(not(unix))]
    {
        match (fs::canonicalize(left), fs::canonicalize(right)) {
            (Ok(a), Ok(b)) => a == b,
            _ => false,
        }
    }
}

/// Writes `bytes` to `target` by writing a sibling temporary file and renaming
/// it over the target, so a crash never leaves a half-written file.
///
/// # Errors
///
/// Returns the underlying I/O error.
pub fn write_atomically(target: &Path, bytes: &[u8]) -> io::Result<()> {
    use std::io::Write as _;

    let parent = target
        .parent()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "target has no parent"))?;
    let name = target
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "target has no name"))?;
    let mut attempt = 0_u32;
    loop {
        let temporary = parent.join(format!(
            ".{name}.tabula-{}-{attempt}.tmp",
            std::process::id()
        ));
        match fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)
        {
            Ok(mut file) => {
                let written = file.write_all(bytes).and_then(|()| file.sync_all());
                drop(file);
                if let Err(error) = written.and_then(|()| fs::rename(&temporary, target)) {
                    let _ = fs::remove_file(&temporary);
                    return Err(error);
                }
                return Ok(());
            }
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists && attempt < 64 => {
                attempt = attempt.saturating_add(1);
            }
            Err(error) => return Err(error),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{PathError, RelativePath, canonical_root, resolve_existing, resolve_for_write};
    use std::fs;

    fn temp_root(name: &str) -> std::path::PathBuf {
        let root = std::env::temp_dir().join(format!("tabula-paths-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        canonical_root(&root).unwrap()
    }

    #[test]
    fn accepts_plain_relative_paths() {
        let path = RelativePath::parse("crypto/chacha20.or").unwrap();
        assert_eq!(path.segments(), ["crypto", "chacha20.or"]);
        assert_eq!(path.file_name(), "chacha20.or");
        assert!(path.has_suffix(".or"));
        assert!(path.has_suffix(".OR"));
        assert!(!RelativePath::parse(".or").is_ok_and(|p| p.has_suffix(".or")));
        assert_eq!(path.parent().unwrap().as_string(), "crypto");
        assert!(RelativePath::parse("a.or").unwrap().parent().is_none());
    }

    #[test]
    fn rejects_escapes_and_odd_names() {
        for raw in [
            "",
            "/etc/passwd",
            "../x.or",
            "a/../../x.or",
            "a/./b.or",
            "a//b.or",
            ".git/config",
            "a/.hidden.or",
            "C:\\x.or",
            "c:x.or",
            "a\\b.or",
            "a\u{0}b.or",
            "trailing.",
            "space ",
            "a/b/",
            "x\n.or",
            "what?.or",
            "pipe|.or",
        ] {
            assert!(
                matches!(RelativePath::parse(raw), Err(PathError::Invalid(_))),
                "accepted {raw:?}"
            );
        }
        let deep = vec!["d"; 40].join("/");
        assert!(RelativePath::parse(&deep).is_err());
        assert!(RelativePath::parse(&"a".repeat(300)).is_err());
    }

    #[test]
    fn resolves_inside_root_and_refuses_symlink_escapes() {
        let root = temp_root("resolve");
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(root.join("src/demo.or"), "edition 2026;").unwrap();
        let demo = RelativePath::parse("src/demo.or").unwrap();
        assert_eq!(
            resolve_existing(&root, &demo).unwrap(),
            root.join("src/demo.or")
        );
        assert!(matches!(
            resolve_existing(&root, &RelativePath::parse("src/missing.or").unwrap()),
            Err(PathError::NotFound)
        ));
        assert_eq!(
            resolve_for_write(&root, &RelativePath::parse("src/new.or").unwrap()).unwrap(),
            root.join("src/new.or")
        );
        assert!(matches!(
            resolve_for_write(&root, &RelativePath::parse("nowhere/new.or").unwrap()),
            Err(PathError::NotFound)
        ));

        #[cfg(unix)]
        {
            let outside = temp_root("outside");
            fs::write(outside.join("secret.or"), "secret").unwrap();
            std::os::unix::fs::symlink(outside.join("secret.or"), root.join("link.or")).unwrap();
            std::os::unix::fs::symlink(&outside, root.join("escape")).unwrap();
            assert!(matches!(
                resolve_existing(&root, &RelativePath::parse("link.or").unwrap()),
                Err(PathError::Invalid(_))
            ));
            assert!(matches!(
                resolve_for_write(&root, &RelativePath::parse("link.or").unwrap()),
                Err(PathError::Invalid(_))
            ));
            assert!(matches!(
                resolve_for_write(&root, &RelativePath::parse("escape/new.or").unwrap()),
                Err(PathError::Invalid(_))
            ));
            fs::remove_dir_all(&outside).unwrap();
        }
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn writes_atomically() {
        let root = temp_root("atomic");
        let target = root.join("a.or");
        super::write_atomically(&target, b"one").unwrap();
        super::write_atomically(&target, b"two").unwrap();
        assert_eq!(fs::read(&target).unwrap(), b"two");
        let leftovers: Vec<_> = fs::read_dir(&root)
            .unwrap()
            .filter_map(Result::ok)
            .filter(|entry| entry.file_name().to_string_lossy().ends_with(".tmp"))
            .collect();
        assert!(leftovers.is_empty());
        fs::remove_dir_all(&root).unwrap();
    }
}
