//! Execute J3's fenced listings and the algorithms corpus test reports.
//! Educational regression evidence only. A passing test is a Match of the
//! Bool the listing writes. It is not a cryptographic security claim, not a
//! constant-time claim, and it does not transcribe FIPS 180-4 §6.2.2.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

const J3: &str =
    include_str!("../../../../docs/book/JOURNEYMAN_J3_THE_CORPUS_AS_ACCEPTANCE_TEST.md");

fn fences<'a>(text: &'a str, language: &str) -> Vec<&'a str> {
    let start = format!("```{language}\n");
    let mut remaining = text;
    let mut blocks = Vec::new();
    while let Some((_, after)) = remaining.split_once(&start) {
        let (body, rest) = after.split_once("\n```").expect("unclosed book fence");
        blocks.push(body);
        remaining = rest;
    }
    blocks
}

fn module_name(source: &str) -> &str {
    source
        .split_whitespace()
        .skip_while(|token| *token != "module")
        .nth(1)
        .expect("complete listing must name its module")
}

fn run(command: &str, source: &str) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_orangec"))
        .args([command, "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("start orangec");
    child
        .stdin
        .take()
        .expect("piped stdin")
        .write_all(source.as_bytes())
        .expect("write the complete listing");
    child.wait_with_output().expect("wait for orangec")
}

fn text_eq(blocks: &[&str], got: &str) -> bool {
    blocks.iter().any(|block| got == format!("{block}\n"))
}

fn algorithms_sources() -> Vec<PathBuf> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../algorithms");
    let mut folders: Vec<PathBuf> = fs::read_dir(&root)
        .expect("algorithms directory")
        .map(|entry| entry.expect("algorithms entry").path())
        .filter(|path| path.is_dir())
        .collect();
    folders.sort();
    let mut sources = Vec::new();
    for folder in folders {
        let mut files: Vec<PathBuf> = fs::read_dir(&folder)
            .expect("algorithm folder")
            .map(|entry| entry.expect("algorithm file").path())
            .filter(|path| path.extension().is_some_and(|extension| extension == "or"))
            .collect();
        files.sort();
        sources.extend(files);
    }
    sources
}

#[test]
fn j3_listings_match_the_compiler_on_this_tree() {
    let sources = fences(J3, "orange");
    let texts = fences(J3, "text");
    let names: Vec<_> = sources.iter().copied().map(module_name).collect();
    assert_eq!(
        names,
        vec![
            "pair",
            "abc_pad",
            "length_field",
            "wrong_vector",
            "agree",
            "count",
            "another",
            "empty_pad",
            "repaired",
        ]
    );
    assert!(J3.contains("orangec 0.0.1 (Orange edition 2026; implemented slice S3t)"));
    assert!(J3.contains("The locked label is J3."));
    assert!(!J3.contains("Chapter 12"));
    assert!(!J3.contains("\n## Chapter "));
    for forbidden in [
        "spec compress(",
        "spec schedule(",
        "spec round(",
        "small_sigma0",
        "small_sigma1",
        "big_sigma0",
        "big_sigma1",
        "0x428a2f98",
        "round_constants",
    ] {
        assert!(!J3.contains(forbidden), "{forbidden}");
    }
    for source in sources {
        let name = module_name(source);
        let check = run("check", source);
        assert!(
            check.status.success(),
            "{name}: {}",
            String::from_utf8_lossy(&check.stderr)
        );
        assert!(check.stdout.is_empty() && check.stderr.is_empty(), "{name}");
        let evaluation = run("eval", source);
        assert!(evaluation.status.success(), "{name}");
        assert!(evaluation.stderr.is_empty(), "{name}");
        let printed = String::from_utf8(evaluation.stdout).expect("UTF-8 value");
        assert!(
            text_eq(&texts, &printed),
            "{name} evaluation was not copied into the lesson:\n{printed}"
        );
        let report = run("test", source);
        assert!(
            report.stderr.is_empty(),
            "{name}: a test report is not a diagnostic"
        );
        let body = String::from_utf8(report.stdout).expect("UTF-8 report");
        assert!(
            text_eq(&texts, &body),
            "{name} test report was not copied into the lesson:\n{body}"
        );
        let failed = body.contains("... FAILED");
        assert_eq!(report.status.success(), !failed, "{name}");
        let again = run("test", source);
        let again_body = String::from_utf8(again.stdout).expect("UTF-8 report");
        assert_eq!(body, again_body, "{name}");
        assert_eq!(report.status.code(), again.status.code(), "{name}");
    }
}

#[test]
fn j3_algorithms_test_reports_match_the_lesson() {
    let texts = fences(J3, "text");
    let sources = algorithms_sources();
    assert_eq!(
        sources.len(),
        39,
        "update J3 when the corpus gains a source"
    );
    let zero = "0 tests: 0 passed, 0 failed\n";
    assert!(
        text_eq(&texts, zero),
        "the zero-test line is not in the lesson"
    );
    let mut zeros = 0usize;
    let mut limbs = 0usize;
    for source in &sources {
        let output = Command::new(env!("CARGO_BIN_EXE_orangec"))
            .arg("test")
            .arg(source)
            .output()
            .expect("run orangec test");
        assert!(
            output.stderr.is_empty(),
            "{}: {}",
            source.display(),
            String::from_utf8_lossy(&output.stderr)
        );
        let stdout = String::from_utf8(output.stdout).expect("UTF-8 report");
        let name = source
            .file_name()
            .and_then(|file| file.to_str())
            .expect("source file name");
        if name == "field25519-limbs.or" {
            limbs += 1;
            assert!(output.status.success(), "{name}");
            assert!(
                text_eq(&texts, &stdout),
                "limb report was not copied into the lesson:\n{stdout}"
            );
        } else {
            zeros += 1;
            assert!(output.status.success(), "{name}");
            assert_eq!(stdout, zero, "{name}");
        }
    }
    assert_eq!(zeros, 38);
    assert_eq!(limbs, 1);
    assert!(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../../algorithms/sha2/sha2.or")
            .is_file()
    );
}
