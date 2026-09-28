//! The Library: The Orange Book and Orange's manuals, read live from an Orange
//! checkout.
//!
//! Tabula keeps no copy of any document. It finds the checkout, lists its
//! reader-facing Markdown, and serves the files as they are on disk, so edits
//! to the book or a manual appear the next time the page is opened.

use std::fs;
use std::path::{Path, PathBuf};

use crate::json::Json;
use crate::paths::{self, PathError, RelativePath};
use crate::workspace::read_text;

/// The file that identifies an Orange checkout.
pub const BOOK_PATH: &str = "docs/THE_ORANGE_BOOK.md";

/// Largest document served.
pub const MAX_DOCUMENT_BYTES: usize = 4 * 1024 * 1024;

/// Largest image served.
pub const MAX_ASSET_BYTES: usize = 16 * 1024 * 1024;

/// Most search results returned.
pub const MAX_SEARCH_RESULTS: usize = 80;

/// Longest search query accepted, in bytes.
pub const MAX_QUERY_BYTES: usize = 200;

/// Manuals shown under their own heading, in order, with display titles.
pub const MANUALS: [(&str, &str); 4] = [
    ("docs/LANGUAGE_2026.md", "Language reference"),
    ("docs/SEMANTICS_2026.md", "Semantics"),
    ("compiler/README.md", "The compiler and orangec"),
    ("tabula/README.md", "Tabula"),
];

const TEXT_SUFFIXES: [&str; 12] = [
    ".md", ".or", ".json", ".jsonc", ".toml", ".rs", ".py", ".txt", ".yml", ".yaml", ".sh", ".c",
];

const IMAGE_TYPES: [(&str, &str); 6] = [
    (".png", "image/png"),
    (".jpg", "image/jpeg"),
    (".jpeg", "image/jpeg"),
    (".gif", "image/gif"),
    (".webp", "image/webp"),
    (".svg", "image/svg+xml"),
];

/// A located Orange checkout.
#[derive(Clone, Debug)]
pub struct Library {
    root: PathBuf,
}

impl Library {
    /// Opens `root` if it looks like an Orange checkout.
    #[must_use]
    pub fn open(root: &Path) -> Option<Self> {
        let root = paths::canonical_root(root).ok()?;
        root.join(BOOK_PATH).is_file().then_some(Self { root })
    }

    /// Finds a checkout: the explicit directory, else the nearest ancestor of
    /// the workspace, else the nearest ancestor of the running executable.
    #[must_use]
    pub fn locate(explicit: Option<&Path>, workspace: &Path) -> Option<Self> {
        if let Some(root) = explicit {
            return Self::open(root);
        }
        let executable = std::env::current_exe().ok();
        workspace
            .ancestors()
            .chain(
                executable
                    .as_deref()
                    .map(Path::ancestors)
                    .into_iter()
                    .flatten(),
            )
            .find_map(Self::open)
    }

    /// The checkout root.
    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Lists the book, the manuals, and the other reader-facing documents.
    #[must_use]
    pub fn catalog(&self) -> Json {
        let mut listed: Vec<String> = Vec::new();
        let mut book = Vec::new();
        if let Some(entry) = self.entry(BOOK_PATH, None) {
            listed.push(BOOK_PATH.to_owned());
            book.push(entry);
        }
        for path in self.markdown_in("docs/book") {
            if let Some(entry) = self.entry(&path, None) {
                book.push(entry);
                listed.push(path);
            }
        }
        let mut manuals = Vec::new();
        for (path, title) in MANUALS {
            if let Some(entry) = self.entry(path, Some(title)) {
                manuals.push(entry);
                listed.push(path.to_owned());
            }
        }
        let mut guides = Vec::new();
        for path in self.markdown_in("docs") {
            if !listed.contains(&path)
                && let Some(entry) = self.entry(&path, None)
            {
                guides.push(entry);
                listed.push(path);
            }
        }
        let mut project = Vec::new();
        for path in [
            "README.md",
            "CONTRIBUTING.md",
            "SECURITY.md",
            "GOVERNANCE.md",
        ] {
            if let Some(entry) = self.entry(path, None) {
                project.push(entry);
            }
        }
        let mut governance = Vec::new();
        for folder in [
            "docs/governance/oeps",
            "docs/governance/adrs",
            "docs/security",
            "docs/operations",
        ] {
            for path in self.markdown_in(folder) {
                if let Some(entry) = self.entry(&path, None) {
                    governance.push(entry);
                }
            }
        }
        Json::object([
            ("ok", Json::Bool(true)),
            (
                "sections",
                Json::Array(
                    [
                        ("book", "The Orange Book", book),
                        ("manuals", "Manuals", manuals),
                        ("guides", "Design and research", guides),
                        ("project", "Project", project),
                        ("governance", "Governance and operations", governance),
                    ]
                    .into_iter()
                    .filter(|(_, _, entries)| !entries.is_empty())
                    .map(|(id, title, entries)| {
                        Json::object([
                            ("id", Json::str(id)),
                            ("title", Json::str(title)),
                            ("documents", Json::Array(entries)),
                        ])
                    })
                    .collect(),
                ),
            ),
        ])
    }

    fn entry(&self, path: &str, title: Option<&str>) -> Option<Json> {
        let text = self.read(path).ok()?;
        let title = title
            .map(str::to_owned)
            .or_else(|| first_heading(&text).map(str::to_owned))
            .unwrap_or_else(|| path.to_owned());
        Some(Json::object([
            ("path", Json::str(path)),
            ("title", Json::str(title)),
        ]))
    }

    fn markdown_in(&self, folder: &str) -> Vec<String> {
        let Ok(relative) = RelativePath::parse(folder) else {
            return Vec::new();
        };
        let Ok(dir) = paths::resolve_existing(&self.root, &relative) else {
            return Vec::new();
        };
        let Ok(entries) = fs::read_dir(dir) else {
            return Vec::new();
        };
        let mut names: Vec<String> = entries
            .flatten()
            .filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_file()))
            .filter_map(|entry| entry.file_name().into_string().ok())
            .filter(|name| !name.starts_with('.') && name.to_ascii_lowercase().ends_with(".md"))
            .filter(|name| !name.eq_ignore_ascii_case("README.md") || folder == "docs/book")
            .collect();
        names.sort();
        names
            .into_iter()
            .map(|name| format!("{folder}/{name}"))
            .collect()
    }

    /// Reads a text document from the checkout.
    ///
    /// # Errors
    ///
    /// Returns an error for invalid, hidden, missing, non-text, or oversized
    /// paths.
    pub fn read(&self, raw: &str) -> Result<String, PathError> {
        let path = RelativePath::parse(raw)?;
        if !TEXT_SUFFIXES.iter().any(|suffix| path.has_suffix(suffix)) {
            return Err(PathError::Invalid("the Library shows text documents only"));
        }
        if path.segments().iter().any(|segment| segment == "target") {
            return Err(PathError::Invalid(
                "build output is not part of the Library",
            ));
        }
        let resolved = paths::resolve_existing(&self.root, &path)?;
        read_text(&resolved, MAX_DOCUMENT_BYTES)
    }

    /// Reads an image referenced by a document, with its media type.
    ///
    /// # Errors
    ///
    /// Returns an error for invalid, hidden, missing, non-image, or oversized
    /// paths.
    pub fn asset(&self, raw: &str) -> Result<(Vec<u8>, &'static str), PathError> {
        let path = RelativePath::parse(raw)?;
        let media_type = IMAGE_TYPES
            .iter()
            .find(|(suffix, _)| path.has_suffix(suffix))
            .map(|(_, media_type)| *media_type)
            .ok_or(PathError::Invalid("the Library serves images only"))?;
        let resolved = paths::resolve_existing(&self.root, &path)?;
        let metadata = fs::metadata(&resolved)?;
        if !metadata.is_file() {
            return Err(PathError::Invalid("not a regular file"));
        }
        if metadata.len() > u64::try_from(MAX_ASSET_BYTES).unwrap_or(u64::MAX) {
            return Err(PathError::Invalid("image is too large"));
        }
        Ok((fs::read(resolved)?, media_type))
    }

    /// Searches the catalog's documents for `query`, case-insensitively.
    #[must_use]
    pub fn search(&self, query: &str) -> Json {
        let needle = query.trim().to_lowercase();
        let mut results = Vec::new();
        let mut truncated = false;
        if !needle.is_empty() && needle.len() <= MAX_QUERY_BYTES {
            'documents: for path in self.catalog_paths() {
                let Ok(text) = self.read(&path) else {
                    continue;
                };
                let title = first_heading(&text).unwrap_or(&path).to_owned();
                let mut heading = String::new();
                let mut in_fence = false;
                for (index, line) in text.lines().enumerate() {
                    let trimmed = line.trim_start();
                    if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
                        in_fence = !in_fence;
                    }
                    if !in_fence && trimmed.starts_with('#') {
                        heading = trimmed.trim_start_matches('#').trim().to_owned();
                    }
                    if line.to_lowercase().contains(&needle) {
                        if results.len() >= MAX_SEARCH_RESULTS {
                            truncated = true;
                            break 'documents;
                        }
                        results.push(Json::object([
                            ("path", Json::str(path.clone())),
                            ("title", Json::str(title.clone())),
                            ("heading", Json::str(heading.clone())),
                            ("line", Json::usize(index.saturating_add(1))),
                            ("snippet", Json::str(snippet(line, &needle))),
                        ]));
                    }
                }
            }
        }
        Json::object([
            ("ok", Json::Bool(true)),
            ("query", Json::str(query.trim())),
            ("results", Json::Array(results)),
            ("truncated", Json::Bool(truncated)),
        ])
    }

    fn catalog_paths(&self) -> Vec<String> {
        let mut paths = vec![BOOK_PATH.to_owned()];
        paths.extend(self.markdown_in("docs/book"));
        paths.extend(MANUALS.iter().map(|(path, _)| (*path).to_owned()));
        for path in self.markdown_in("docs") {
            if !paths.contains(&path) {
                paths.push(path);
            }
        }
        paths.extend(["README.md".to_owned()]);
        for folder in [
            "docs/governance/oeps",
            "docs/governance/adrs",
            "docs/security",
            "docs/operations",
        ] {
            paths.extend(self.markdown_in(folder));
        }
        paths
    }
}

/// The text of the first level-one heading.
#[must_use]
pub fn first_heading(text: &str) -> Option<&str> {
    text.lines()
        .find_map(|line| line.strip_prefix("# "))
        .map(str::trim)
        .filter(|heading| !heading.is_empty())
}

fn snippet(line: &str, needle: &str) -> String {
    const CONTEXT: usize = 70;
    let characters: Vec<char> = line.trim().chars().collect();
    let lowered: Vec<char> = characters.iter().flat_map(|c| c.to_lowercase()).collect();
    let position = if lowered.len() == characters.len() {
        let needle: Vec<char> = needle.chars().collect();
        lowered
            .windows(needle.len().max(1))
            .position(|window| window == needle.as_slice())
            .unwrap_or(0)
    } else {
        0
    };
    let start = position.saturating_sub(CONTEXT);
    let end = position
        .saturating_add(needle.chars().count())
        .saturating_add(CONTEXT)
        .min(characters.len());
    let mut text: String = characters.get(start..end).unwrap_or(&[]).iter().collect();
    if start > 0 {
        text.insert(0, '…');
    }
    if end < characters.len() {
        text.push('…');
    }
    text
}

#[cfg(test)]
mod tests {
    use super::{Library, first_heading, snippet};
    use std::fs;

    fn checkout(name: &str) -> std::path::PathBuf {
        let root = std::env::temp_dir().join(format!("tabula-lib-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("docs/images")).unwrap();
        fs::create_dir_all(root.join("compiler")).unwrap();
        fs::create_dir_all(root.join(".git")).unwrap();
        fs::write(root.join("docs/THE_ORANGE_BOOK.md"), "# The Orange Book\n\n## Chapter 1\n\nClaims, not labels.\n```orange\nspec claims() {}\n```\n").unwrap();
        fs::write(
            root.join("docs/LANGUAGE_2026.md"),
            "# Orange 2026 language\n\nORC0203 means unsupported type.\n",
        )
        .unwrap();
        fs::write(root.join("docs/ROADMAP.md"), "# Roadmap\n").unwrap();
        fs::write(root.join("compiler/README.md"), "# Orange compiler\n").unwrap();
        fs::write(root.join("README.md"), "# Orange\n").unwrap();
        fs::write(root.join("docs/images/a.png"), [0x89, b'P', b'N', b'G']).unwrap();
        fs::write(root.join(".git/config"), "secret").unwrap();
        root
    }

    #[test]
    fn requires_the_book_to_open() {
        let root = std::env::temp_dir().join(format!("tabula-lib-none-{}", std::process::id()));
        fs::create_dir_all(&root).unwrap();
        assert!(Library::open(&root).is_none());
        let checkout = checkout("open");
        let nested = checkout.join("examples/deep");
        fs::create_dir_all(&nested).unwrap();
        let found = Library::locate(None, &fs::canonicalize(&nested).unwrap()).unwrap();
        assert_eq!(found.root(), fs::canonicalize(&checkout).unwrap());
        fs::remove_dir_all(&checkout).unwrap();
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn catalogs_reads_and_searches() {
        let root = checkout("catalog");
        let library = Library::open(&root).unwrap();
        let catalog = library.catalog().render();
        assert!(catalog.contains(r#""title":"The Orange Book""#));
        assert!(catalog.contains(r#""path":"docs/LANGUAGE_2026.md","title":"Language reference""#));
        assert!(
            catalog.contains(r#""path":"compiler/README.md","title":"The compiler and orangec""#)
        );
        assert!(catalog.contains(r#""path":"docs/ROADMAP.md","title":"Roadmap""#));
        assert!(!catalog.contains("SEMANTICS"));

        assert!(
            library
                .read("docs/ROADMAP.md")
                .unwrap()
                .starts_with("# Roadmap")
        );
        assert!(library.read(".git/config").is_err());
        assert!(library.read("../etc/passwd").is_err());
        assert!(library.read("docs/images/a.png").is_err());
        assert_eq!(library.asset("docs/images/a.png").unwrap().1, "image/png");
        assert!(library.asset("docs/ROADMAP.md").is_err());

        let results = library.search("orc0203").render();
        assert!(results.contains(r#""path":"docs/LANGUAGE_2026.md""#));
        assert!(results.contains(r#""line":3"#));
        assert!(library.search("   ").render().contains(r#""results":[]"#));
        let heading = library.search("claims, not").render();
        assert!(heading.contains(r#""heading":"Chapter 1""#));
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn headings_and_snippets() {
        assert_eq!(first_heading("text\n# Title \n# Second"), Some("Title"));
        let long = format!("{}needle{}", "a".repeat(100), "b".repeat(100));
        let clipped = snippet(&long, "needle");
        assert!(clipped.starts_with('…') && clipped.ends_with('…') && clipped.contains("needle"));
        assert_eq!(snippet("short needle", "needle"), "short needle");
    }
}
