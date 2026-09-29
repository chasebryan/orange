//! Reproduction of the recorded vectors of every algorithm entry under
//! `algorithms/`.
//!
//! Each `.or` source there carries pairs of parameterless specs `<name>` and
//! `<name>_expected`: the first computes a value with the algorithm, the
//! second states the value published with the standard or vector source that
//! the file's comments cite. This test runs the real `orangec` binary over
//! every source, requires `check` to pass without diagnostics and `eval` to
//! succeed, and requires the two members of every pair to print the same type
//! and value. `algorithms/verify.py` makes the same check from the command
//! line.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn algorithms_directory() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../algorithms")
}

/// Every algorithm folder with its Orange sources, in path order.
fn entries() -> Vec<(PathBuf, Vec<PathBuf>)> {
    let mut folders: Vec<PathBuf> = fs::read_dir(algorithms_directory())
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.is_dir())
        .collect();
    folders.sort();
    folders
        .into_iter()
        .map(|folder| {
            let mut sources: Vec<PathBuf> = fs::read_dir(&folder)
                .unwrap()
                .map(|entry| entry.unwrap().path())
                .filter(|path| path.extension().is_some_and(|extension| extension == "or"))
                .collect();
            sources.sort();
            (folder, sources)
        })
        .collect()
}

fn orangec(command: &str, source: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_orangec"))
        .arg(command)
        .arg(source)
        .output()
        .unwrap()
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8(bytes.to_vec()).unwrap()
}

/// The `name` and `Type = value` of one `module::name: Type = value` line.
fn value_line(line: &str) -> Option<(String, String)> {
    let (qualified, rest) = line.split_once(": ")?;
    let (_module, name) = qualified.split_once("::")?;
    Some((name.to_owned(), rest.to_owned()))
}

#[test]
fn every_algorithm_folder_has_a_readme_and_sources() {
    let entries = entries();
    assert!(!entries.is_empty(), "algorithms/ holds no entry");
    for (folder, sources) in &entries {
        assert!(
            folder.join("README.md").is_file(),
            "{} has no README.md",
            folder.display()
        );
        assert!(
            !sources.is_empty(),
            "{} has no .or source",
            folder.display()
        );
    }
}

#[test]
fn every_algorithm_source_checks_without_diagnostics() {
    for (_folder, sources) in entries() {
        for source in sources {
            let output = orangec("check", &source);
            assert!(
                output.status.success() && output.stderr.is_empty(),
                "{} does not check:\n{}{}",
                source.display(),
                text(&output.stdout),
                text(&output.stderr)
            );
        }
    }
}

#[test]
fn every_algorithm_source_reproduces_its_recorded_vectors() {
    for (_folder, sources) in entries() {
        for source in sources {
            let output = orangec("eval", &source);
            assert!(
                output.status.success() && output.stderr.is_empty(),
                "{} does not evaluate:\n{}{}",
                source.display(),
                text(&output.stdout),
                text(&output.stderr)
            );
            let mut values = BTreeMap::new();
            for line in text(&output.stdout).lines() {
                let (name, value) = value_line(line)
                    .unwrap_or_else(|| panic!("{}: unrecognized line {line}", source.display()));
                values.insert(name, value);
            }
            let mut reproduced = 0usize;
            for (name, expected) in &values {
                let Some(computed_name) = name.strip_suffix("_expected") else {
                    continue;
                };
                let computed = values
                    .get(computed_name)
                    .unwrap_or_else(|| panic!("{}: {name} has no computed twin", source.display()));
                assert_eq!(
                    computed,
                    expected,
                    "{}: {computed_name} differs from {name}",
                    source.display()
                );
                reproduced += 1;
            }
            assert!(
                reproduced > 0,
                "{} records no `<name>` and `<name>_expected` pair",
                source.display()
            );
        }
    }
}
