//! Reproduction of the published vectors of every algorithm entry under
//! `algorithms/`.
//!
//! Each `.or` source there states its vectors in one of two forms. A `test`
//! block claims that the algorithm reproduces one published value, as the
//! title cites it; `orangec test` must pass every block. The first entries
//! instead carry pairs of parameterless specs `<name>` and `<name>_expected`,
//! the computed value and the published one, which `orangec eval` must print
//! identically. A source with neither form must be a module that a sibling
//! root `use`s. This test runs the real `orangec` binary over every source
//! with the largest step budget it admits, requires `check` to pass without
//! diagnostics, and requires every claim to hold. Sources run on as many
//! threads as the machine offers, so the slowest entry, not the sum of all of
//! them, sets the test's time. `algorithms/verify.py` makes the same check
//! from the command line.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;

/// The largest budget `orangec eval` and `orangec test` admit.
const STEP_BUDGET: &str = "1073741824";

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

/// Calls `run` with every source and its folder's sources, spreading the
/// sources over the machine's threads; a source whose claims fail panics its
/// thread, and the scope then fails the test.
fn for_each_source(run: impl Fn(&Path, &[PathBuf]) + Sync) {
    let sources: Vec<(PathBuf, Vec<PathBuf>)> = entries()
        .into_iter()
        .flat_map(|(_folder, sources)| {
            sources
                .iter()
                .map(|source| (source.clone(), sources.clone()))
                .collect::<Vec<_>>()
        })
        .collect();
    let threads = thread::available_parallelism().map_or(1, usize::from);
    let next = AtomicUsize::new(0);
    thread::scope(|scope| {
        for _ in 0..threads {
            scope.spawn(|| {
                while let Some((source, siblings)) =
                    sources.get(next.fetch_add(1, Ordering::Relaxed))
                {
                    run(source, siblings);
                }
            });
        }
    });
}

fn orangec(arguments: &[&str], source: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_orangec"))
        .args(arguments)
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

/// Whether the source declares at least one `test "..."` block.
fn declares_tests(source_text: &str) -> bool {
    source_text
        .lines()
        .any(|line| line.trim_start().starts_with("test \""))
}

/// Whether the source declares a parameterless spec named `<name>_expected`.
fn declares_pairs(source_text: &str) -> bool {
    source_text.lines().any(|line| {
        let line = line.trim_start();
        line.strip_prefix("spec ")
            .and_then(|rest| rest.split_once('('))
            .is_some_and(|(name, _)| name.trim_end().ends_with("_expected"))
    })
}

/// Whether a sibling source in the same folder declares `use <stem>;`.
fn used_by_sibling(source: &Path, sources: &[PathBuf]) -> bool {
    let stem = source.file_stem().unwrap().to_str().unwrap();
    let declaration = format!("use {stem};");
    sources
        .iter()
        .filter(|other| *other != source)
        .any(|other| {
            fs::read_to_string(other)
                .unwrap()
                .lines()
                .any(|line| line.split_whitespace().collect::<Vec<_>>().join(" ") == declaration)
        })
}

/// Runs the source's tests and returns how many passed; panics on any failure.
fn passing_tests(source: &Path) -> usize {
    let output = orangec(&["test", "--steps", STEP_BUDGET], source);
    let stdout = text(&output.stdout);
    assert!(
        output.status.success() && output.stderr.is_empty(),
        "{} has failing tests:\n{stdout}{}",
        source.display(),
        text(&output.stderr)
    );
    let summary = stdout.lines().last().unwrap_or_default();
    let (count, rest) = summary
        .split_once(' ')
        .unwrap_or_else(|| panic!("{}: unrecognized test report {summary}", source.display()));
    let passed: usize = count.parse().unwrap();
    assert!(
        rest.ends_with(" failed") && !stdout.contains("... FAILED"),
        "{}: unrecognized test report {summary}",
        source.display()
    );
    passed
}

/// Evaluates the source's `<name>` and `<name>_expected` pairs and returns how
/// many agreed; panics on any difference.
fn reproduced_pairs(source: &Path) -> usize {
    let output = orangec(&["eval", "--steps", STEP_BUDGET], source);
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
    reproduced
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
    for_each_source(|source, _siblings| {
        let output = orangec(&["check"], source);
        assert!(
            output.status.success() && output.stderr.is_empty(),
            "{} does not check:\n{}{}",
            source.display(),
            text(&output.stdout),
            text(&output.stderr)
        );
    });
}

#[test]
fn every_algorithm_source_passes_its_tests_or_reproduces_its_vectors() {
    for_each_source(|source, siblings| {
        let source_text = fs::read_to_string(source).unwrap();
        let mut claims = 0usize;
        if declares_tests(&source_text) {
            let passed = passing_tests(source);
            assert!(
                passed > 0,
                "{} declares tests but ran none",
                source.display()
            );
            claims += passed;
        }
        if declares_pairs(&source_text) {
            let reproduced = reproduced_pairs(source);
            assert!(
                reproduced > 0,
                "{} declares `_expected` specs but reproduced none",
                source.display()
            );
            claims += reproduced;
        }
        assert!(
            claims > 0 || used_by_sibling(source, siblings),
            "{} states no vector: no `test` block, no `<name>_expected` pair, and no sibling `use`s it",
            source.display()
        );
    });
}
