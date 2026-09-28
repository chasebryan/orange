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
