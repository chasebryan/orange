//! Tabula, a minimalist and mouse-first writing table for Orange.
//!
//! Tabula is a single executable that serves a small editor to a browser on
//! the same machine. It edits only Orange sources, runs the real `orangec`
//! for checking, evaluation, and tokens, keeps a Markdown notebook beside the
//! workspace, and reads The Orange Book and the manuals live from an Orange
//! checkout. It depends on nothing outside the Rust standard library.

pub mod assets;
pub mod http;
pub mod json;
pub mod library;
pub mod notes;
pub mod orangec;
pub mod paths;
pub mod server;
pub mod token;
pub mod workspace;

/// A fresh, empty folder for one unit test, under this crate's `target`
/// folder. The path is fixed at build time rather than read from the
/// environment, so a test never writes where a variable points.
#[cfg(test)]
pub(crate) fn test_dir(label: &str) -> std::path::PathBuf {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("tabula-tests")
        .join(format!("{label}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    root
}
