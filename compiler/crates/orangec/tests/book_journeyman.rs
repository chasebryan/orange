//! Execute J4's fenced listings, not copied fixtures.
//! Educational regression evidence only; not a compiler or security proof.

use std::io::Write;
use std::process::{Command, Output, Stdio};

const J4: &str =
    include_str!("../../../../docs/book/JOURNEYMAN_J4_BYTE_ORDER_AND_FORMAT_BOUNDARIES.md");

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

#[test]
fn j4_listings_match_the_compiler_on_this_tree() {
    let sources = fences(J4, "orange");
    let texts = fences(J4, "text");
    assert_eq!(sources.len(), 18, "update coverage when adding listings");
    assert!(J4.contains("orangec 0.0.1 (Orange edition 2026; implemented slice S3t)"));
    assert!(!J4.contains("0x428a2f98"));
    assert!(!J4.contains("spec compress("));
    assert!(!J4.contains("spec schedule("));
    assert!(J4.contains("The locked label is J4."));
    for source in sources {
        let name = module_name(source);
        if name == "short" {
            let check = run("check", source);
            assert_eq!(check.status.code(), Some(1), "{name}");
            assert!(check.stdout.is_empty(), "{name}");
            let diagnostic = String::from_utf8(check.stderr).expect("UTF-8 diagnostic");
            assert!(
                text_eq(&texts, &diagnostic),
                "{name} diagnostic was not copied into the lesson:\n{diagnostic}"
            );
            for command in ["eval", "test"] {
                let result = run(command, source);
                assert_eq!(result.status.code(), Some(1), "{name}: {command}");
                assert!(result.stdout.is_empty(), "{name}: {command}");
                let again = String::from_utf8(result.stderr).expect("UTF-8 diagnostic");
                assert_eq!(again, diagnostic, "{name}: {command}");
            }
            continue;
        }
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
        let claimed: Vec<_> = texts
            .iter()
            .filter(|block| block.starts_with(&format!("{name}::")))
            .collect();
        if printed.is_empty() {
            assert!(
                claimed.is_empty(),
                "{name} evaluated to nothing, but a fence claims a value"
            );
        } else {
            assert_eq!(claimed.len(), 1, "{name}");
            assert_eq!(printed, format!("{}\n", claimed[0]), "{name}");
            let again = run("eval", source);
            let again_printed = String::from_utf8(again.stdout).expect("UTF-8 value");
            assert_eq!(printed, again_printed, "{name}");
            assert_eq!(evaluation.status.code(), again.status.code(), "{name}");
        }
        if !source.contains("test \"") {
            let report = run("test", source);
            assert!(report.status.success(), "{name}");
            assert_eq!(
                String::from_utf8(report.stdout).expect("UTF-8 report"),
                "0 tests: 0 passed, 0 failed\n",
                "{name}"
            );
            assert!(report.stderr.is_empty(), "{name}");
            continue;
        }
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
