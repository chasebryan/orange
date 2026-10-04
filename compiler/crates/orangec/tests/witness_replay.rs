//! Independent command-line contracts for concrete Boolean witness replay.
//! Scratch roots are fixed by Cargo at build time and created exclusively.

#[cfg(target_os = "linux")]
use std::ffi::OsStr;
use std::fs::{self, OpenOptions};
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

use orange_compiler::MAX_SOURCE_BYTES;

struct Scratch(PathBuf);

impl Scratch {
    fn new(name: &str) -> Self {
        let root = Path::new(env!("CARGO_TARGET_TMPDIR"));
        fs::create_dir_all(root).unwrap();
        let path = root.join(format!("witness-{name}-{}", std::process::id()));
        fs::create_dir(&path).unwrap();
        Self(path)
    }

    fn write(&self, name: impl AsRef<Path>, bytes: &[u8]) -> PathBuf {
        let path = self.0.join(name);
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .unwrap();
        file.write_all(bytes).unwrap();
        path
    }

    fn run(&self, arguments: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_orangec"))
            .current_dir(&self.0)
            .args(arguments)
            .output()
            .unwrap()
    }

    fn replay(&self, function: &str, witness: &str, extra: &[&str]) -> Output {
        let mut arguments = vec!["replay", "--function", function, "--witness", witness];
        arguments.extend_from_slice(extra);
        arguments.push("subject.or");
        self.run(&arguments)
    }

    fn stdin(&self, arguments: &[&str], bytes: &[u8]) -> Output {
        let mut child = Command::new(env!("CARGO_BIN_EXE_orangec"))
            .current_dir(&self.0)
            .args(arguments)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        if let Err(error) = child.stdin.take().unwrap().write_all(bytes) {
            assert_eq!(error.kind(), std::io::ErrorKind::BrokenPipe);
        }
        child.wait_with_output().unwrap()
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

fn success(output: &Output) -> &str {
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        output.stderr.is_empty(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    std::str::from_utf8(&output.stdout).unwrap()
}

fn failure(output: &Output, status: i32, diagnostic: &str) {
    assert_eq!(
        output.status.code(),
        Some(status),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stdout.is_empty());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains(diagnostic),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn replay_reports_exact_observations_and_all_scalar_types() {
    let scratch = Scratch::new("scalars");
    let source = b"edition 2026;module m{spec p(i:Int,b:Bool,a:Word[8],c:Word[16],d:Word[32],e:Word[64],r:Mod[7])->Bool{(i == -17) && b && (a == 0xff) && (c == 0x1234) && (d == 0x89abcdef) && (e == 0xffffffffffffffff) && (r == 6)}}";
    scratch.write("subject.or", source);
    let vector = "[-17, true, 0xff, 0x1234, 0x89abcdef, 0xffffffffffffffff, 6]";
    scratch.write("true.values", vector.as_bytes());
    scratch.write("false.values", vector.replace("true", "false").as_bytes());
    let output = scratch.replay("m::p", "true.values", &[]);
    assert_eq!(
        success(&output),
        format!(
            "m::p[]: holds_for_this_witness\nparameter_types: [Int, Bool, Word[8], Word[16], Word[32], Word[64], Mod[7]]\narguments: {vector}\n"
        )
    );
    assert_eq!(
        output.stdout,
        scratch.replay("m::p", "true.values", &[]).stdout
    );
    let output = scratch.replay("m::p", "false.values", &[]);
    assert_eq!(
        success(&output),
        format!(
            "m::p[]: falsified\nparameter_types: [Int, Bool, Word[8], Word[16], Word[32], Word[64], Mod[7]]\narguments: {}\n",
            vector.replace("true", "false")
        )
    );
    assert_eq!(fs::read(scratch.0.join("subject.or")).unwrap(), source);
    assert_eq!(
        fs::read(scratch.0.join("true.values")).unwrap(),
        vector.as_bytes()
    );
}

#[test]
fn replay_decodes_arrays_matrices_and_tuple_parameters() {
    let scratch = Scratch::new("aggregates");
    scratch.write("subject.or", b"edition 2026;module m{type Row=Mod[7]^2;type Matrix=Row^2;spec p(a:Int^3,b:Bool^2,c:Matrix,d:(Int,Word[8]^2,Matrix))->Bool{(a == [-1,0,1]) && (b == [true,false]) && (c == [[1,2],[3,4]]) && (d == (9,[0x00,0xff],[[6,5],[4,3]]))}}" );
    let vector =
        "[[-1, 0, 1], [true, false], [[1, 2], [3, 4]], (9, [0x00, 0xff], [[6, 5], [4, 3]])]";
    scratch.write("good.values", vector.as_bytes());
    let output = scratch.replay("m::p", "good.values", &[]);
    assert_eq!(
        success(&output),
        format!(
            "m::p[]: holds_for_this_witness\nparameter_types: [Int^3, Bool^2, (Mod[7]^2)^2, (Int, Word[8]^2, (Mod[7]^2)^2)]\narguments: {vector}\n"
        )
    );
    scratch.write("bad.values", vector.replace("[4, 3]", "[4, 2]").as_bytes());
    assert!(success(&scratch.replay("m::p", "bad.values", &[])).contains(": falsified\n"));
}

#[test]
fn replay_selects_complete_linked_numeric_instances() {
    let scratch = Scratch::new("instances");
    scratch.write(
        "subject.or",
        b"edition 2026;module root{use helper;spec p(x:Int)->Bool{x == 99}}",
    );
    scratch.write("helper.or", b"edition 2026;module helper{spec p[m in 3..5,n in 1..3,T in {Int,Word[8]}](r:Mod[m],a:T^n)->Bool{(r == 0) && (a == [0;n])}}" );
    scratch.write("good.values", b"[0, [0x00, 0x00]]\n");
    let output = scratch.replay("helper::p", "good.values", &["--instance=4,2,1"]);
    assert_eq!(
        success(&output),
        "helper::p[4, 2, 1]: holds_for_this_witness\nparameter_types: [Mod[4], Word[8]^2]\narguments: [0, [0x00, 0x00]]\n"
    );
    for extra in [
        &[][..],
        &["--instance", "4,2"],
        &["--instance", "4,2,0,1"],
        &["--instance", "5,2,1"],
        &["--instance", "4,2,2"],
    ] {
        failure(
            &scratch.replay("helper::p", "good.values", extra),
            1,
            "ORC1016",
        );
    }
    scratch.write("wrong.values", b"[0, [0, 0]]");
    failure(
        &scratch.replay("helper::p", "wrong.values", &["--instance", "4,2,1"]),
        1,
        "ORC0270",
    );
    scratch.write("root.values", b"[99]");
    assert!(
        success(&scratch.replay("root::p", "root.values", &[]))
            .starts_with("root::p[]: holds_for_this_witness\n")
    );
}

#[test]
fn replay_metadata_never_uses_source_spelled_instance_comments() {
    let scratch = Scratch::new("controls");
    scratch.write("subject.or", b"edition 2026;module m{spec p[T in {Mod[7/*\0\x1b[31m\xe2\x80\xae*/]}](x:T)->Bool{x == 6}}" );
    scratch.write("good.values", b"[6]");
    let output = scratch.replay("m::p", "good.values", &["--instance", "0", "--stats"]);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        output.stdout,
        b"m::p[0]: holds_for_this_witness\nparameter_types: [Mod[7]]\narguments: [6]\n"
    );
    let statistics = String::from_utf8(output.stderr).unwrap();
    assert!(statistics.starts_with("m::p[0]: "));
    assert!(statistics.contains("\ntotal: "));
    assert!(
        statistics
            .bytes()
            .all(|byte| byte == b'\n' || (32..=126).contains(&byte))
    );
    assert!(!statistics.contains("/*"));
}

#[test]
fn replay_refuses_nonboolean_tests_and_missing_selectors() {
    let scratch = Scratch::new("entrypoints");
    scratch.write("subject.or", b"edition 2026;module m{spec number(x:Int)->Int{x}spec sized[n in 1..2]()->Bool{true}test \"known answer\"{let x:Int=1;x == 1}}" );
    scratch.write("empty.values", b"[]");
    for (name, extra) in [
        ("m::number", &[][..]),
        ("m::test", &[][..]),
        ("m::sized", &[][..]),
        ("m::missing", &[][..]),
        ("other::sized", &["--instance", "1"][..]),
    ] {
        failure(&scratch.replay(name, "empty.values", extra), 1, "ORC1016");
    }
    assert_eq!(
        success(&scratch.replay("m::sized", "empty.values", &["--instance", "1"])),
        "m::sized[1]: holds_for_this_witness\nparameter_types: []\narguments: []\n"
    );
}

#[test]
fn replay_accepts_maximum_array_axis_and_matrix_leaf_product() {
    let scratch = Scratch::new("shape-boundaries");
    scratch.write("subject.or", b"edition 2026;module m{type Row=Word[8]^256;spec array(x:Word[8]^65536)->Bool{x[65535] == 0xff}spec matrix(x:Row^256)->Bool{x[255][255] == 0xff}}" );
    let mut leaves = vec!["0x00"; 65536];
    leaves[65535] = "0xff";
    let array = format!("[[{}]]", leaves.join(", "));
    scratch.write("array.values", array.as_bytes());
    let array_output = scratch.replay("m::array", "array.values", &[]);
    assert!(success(&array_output).contains(&format!("arguments: {array}\n")));
    let rows = leaves
        .chunks(256)
        .map(|row| format!("[{}]", row.join(", ")))
        .collect::<Vec<_>>();
    let matrix = format!("[[{}]]", rows.join(", "));
    scratch.write("matrix.values", matrix.as_bytes());
    let matrix_output = scratch.replay("m::matrix", "matrix.values", &[]);
    assert!(success(&matrix_output).contains(&format!("arguments: {matrix}\n")));
    scratch.write("wrong.values", array.as_bytes());
    failure(
        &scratch.replay("m::matrix", "wrong.values", &[]),
        1,
        "ORC0271",
    );
}

#[test]
fn replay_rejects_noncanonical_packets_and_values_without_stdout() {
    let scratch = Scratch::new("canonical");
    scratch.write(
        "subject.or",
        b"edition 2026;module m{spec p(x:Int)->Bool{x == 7}}",
    );
    for (index, packet) in [
        "7",
        "[07]",
        "[+7]",
        "[-0]",
        "[0x07]",
        "[ 7]",
        "[7 ]",
        "[7,]",
        "[7] ",
        "[7]\r\n",
        "[7]\n\n",
        "[7/*comment*/]",
        "[7 + 0]",
        "[7]\0",
        "[٧]",
        "[7]\u{202e}",
        "[7",
        "[]",
        "[7, 8]",
    ]
    .iter()
    .enumerate()
    {
        let name = format!("bad-{index}.values");
        scratch.write(&name, packet.as_bytes());
        let output = scratch.replay("m::p", &name, &[]);
        failure(&output, 1, "ORC027");
        assert!(!String::from_utf8_lossy(&output.stderr).contains("proof"));
    }
    scratch.write("good.values", b"[7]\n");
    assert!(success(&scratch.replay("m::p", "good.values", &[])).contains("arguments: [7]\n"));
}

#[test]
fn replay_rejects_wrong_scalar_domains_and_aggregate_shapes() {
    let scratch = Scratch::new("mismatch");
    scratch.write("subject.or", b"edition 2026;module m{type Row=Mod[7]^2;spec p(b:Bool,w:Word[8],r:Mod[7],m:Row^2,t:(Int,Bool))->Bool{b && (w == 0xff) && (r == 6) && (m == [[1,2],[3,4]]) && (t == (1,true))}}" );
    let good = "[true, 0xff, 6, [[1, 2], [3, 4]], (1, true)]";
    for (index, bad) in [
        good.replace("true, 0xff", "1, 0xff"),
        good.replace("0xff", "0xFF"),
        good.replace("0xff", "0xf"),
        good.replace("0xff", "255"),
        good.replace(", 6,", ", 7,"),
        good.replace(", 6,", ", -1,"),
        good.replace("[3, 4]", "[3]"),
        good.replace("[[1, 2], [3, 4]]", "[1, 2, 3, 4]"),
        good.replace("[3, 4]", "[3, 7]"),
        good.replace("(1, true)", "(1, true,)"),
        good.replace("(1, true)", "(1, true, false)"),
        good.replace(", ", ","),
    ]
    .iter()
    .enumerate()
    {
        let name = format!("bad-{index}.values");
        scratch.write(&name, bad.as_bytes());
        failure(&scratch.replay("m::p", &name, &[]), 1, "ORC027");
    }
    scratch.write("good.values", good.as_bytes());
    assert!(
        success(&scratch.replay("m::p", "good.values", &[])).contains(": holds_for_this_witness\n")
    );
}

#[test]
fn replay_steps_and_stats_describe_only_completed_output() {
    let scratch = Scratch::new("steps");
    scratch.write(
        "subject.or",
        b"edition 2026;module m{spec p(x:Int)->Bool{(x + 1) == 8}}",
    );
    scratch.write("good.values", b"[7]");
    let ordinary = scratch.replay("m::p", "good.values", &[]);
    success(&ordinary);
    let measured = scratch.replay("m::p", "good.values", &["--stats"]);
    assert_eq!(measured.status.code(), Some(0));
    assert_eq!(measured.stdout, ordinary.stdout);
    let statistics = std::str::from_utf8(&measured.stderr).unwrap();
    let (first, total) = statistics.trim_end().split_once('\n').unwrap();
    let steps: usize = first
        .strip_prefix("m::p[]: ")
        .unwrap()
        .split_once(' ')
        .unwrap()
        .0
        .parse()
        .unwrap();
    assert!(steps > 1);
    assert_eq!(total, format!("total: {steps} of 1048576 steps"));
    let exact = steps.to_string();
    let output = scratch.replay("m::p", "good.values", &["--steps", &exact, "--stats"]);
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(output.stdout, ordinary.stdout);
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .ends_with(&format!("total: {steps} of {steps} steps\n"))
    );
    let short = (steps - 1).to_string();
    let output = scratch.replay("m::p", "good.values", &["--steps", &short, "--stats"]);
    failure(&output, 1, "ORC0301");
    let error = String::from_utf8(output.stderr).unwrap();
    assert!(error.contains("`orangec replay --steps N`"));
    assert!(!error.contains("\ntotal:"));
}

#[test]
fn replay_supports_one_stdin_role_and_option_marker_paths() {
    let scratch = Scratch::new("stdin");
    let source = b"edition 2026;module m{spec p(x:Int)->Bool{x == 7}}";
    scratch.write("subject.or", source);
    scratch.write("--witness.values", b"[7]");
    scratch.write("--subject.or", source);
    let from_witness = scratch.stdin(
        &["replay", "--function=m::p", "--witness=-", "subject.or"],
        b"[7]\n",
    );
    let from_source = scratch.stdin(
        &[
            "--edition=2026",
            "replay",
            "--function",
            "m::p",
            "--witness",
            "--witness.values",
            "-",
        ],
        source,
    );
    assert_eq!(success(&from_source), success(&from_witness));
    let marker = scratch.run(&[
        "replay",
        "--function",
        "m::p",
        "--witness",
        "--witness.values",
        "--",
        "--subject.or",
    ]);
    assert_eq!(success(&marker), success(&from_witness));
    failure(
        &scratch.stdin(
            &["replay", "--function", "m::p", "--witness", "-", "-"],
            b"not read",
        ),
        2,
        "cannot both read standard input",
    );
}

#[test]
fn replay_usage_contract_rejects_duplicates_and_incomplete_selectors() {
    let scratch = Scratch::new("usage");
    for arguments in [
        &["replay", "subject.or"][..],
        &["replay", "--function", "m::p", "subject.or"],
        &["replay", "--witness", "values", "subject.or"],
        &[
            "replay",
            "--function",
            "p",
            "--witness",
            "values",
            "subject.or",
        ],
        &[
            "replay",
            "--function",
            "m::p::x",
            "--witness",
            "values",
            "subject.or",
        ],
        &[
            "replay",
            "--function",
            "m::p",
            "--function",
            "m::p",
            "--witness",
            "values",
            "subject.or",
        ],
        &[
            "replay",
            "--function",
            "m::p",
            "--witness",
            "values",
            "--witness",
            "values",
            "subject.or",
        ],
        &[
            "replay",
            "--function",
            "m::p",
            "--witness",
            "values",
            "a.or",
            "b.or",
        ],
        &[
            "replay",
            "--function",
            "m::p",
            "--witness",
            "values",
            "--spec",
            "p",
            "subject.or",
        ],
        &[
            "replay",
            "--function",
            "m::p",
            "--witness",
            "values",
            "--check",
            "subject.or",
        ],
        &[
            "replay",
            "--function",
            "m::p",
            "--witness",
            "values",
            "-o",
            "out",
            "subject.or",
        ],
        &["eval", "--function", "m::p", "subject.or"],
        &["test", "--witness", "values", "subject.or"],
        &["check", "--instance", "1", "subject.or"],
    ] {
        failure(&scratch.run(arguments), 2, "orangec:");
    }
    for value in [
        "",
        "01",
        "-1",
        "+1",
        "1,",
        ",1",
        "1, 2",
        "1,,2",
        "4294967296",
        "1,2,3,4,5",
    ] {
        failure(
            &scratch.run(&[
                "replay",
                "--function",
                "m::p",
                "--witness",
                "values",
                "--instance",
                value,
                "subject.or",
            ]),
            2,
            "option `--instance`",
        );
    }
    failure(
        &scratch.run(&[
            "replay",
            "--function",
            "m::p",
            "--witness",
            "values",
            "--instance",
            "1",
            "--instance",
            "1",
            "subject.or",
        ]),
        2,
        "at most once",
    );
}

#[test]
fn replay_input_failures_and_source_validation_emit_no_observation() {
    let scratch = Scratch::new("input");
    scratch.write(
        "subject.or",
        b"edition 2026;module m{spec p(x:Int)->Bool{x == 7}}",
    );
    scratch.write("invalid.values", &[0xff]);
    failure(&scratch.replay("m::p", "missing.values", &[]), 1, "ORC1001");
    failure(&scratch.replay("m::p", "invalid.values", &[]), 1, "ORC1002");
    scratch.write("syntax.or", b"edition 2026;module m{spec p(x:Int)->Bool{}}");
    scratch.write(
        "types.or",
        b"edition 2026;module m{spec p(x:Int)->Bool{x + true}}",
    );
    scratch.write(
        "imports.or",
        b"edition 2026;module m{use missing;spec p(x:Int)->Bool{x == 7}}",
    );
    for source in ["syntax.or", "types.or", "imports.or"] {
        let output = scratch.run(&[
            "replay",
            "--function",
            "m::p",
            "--witness",
            "missing.values",
            source,
        ]);
        failure(&output, 1, "ORC");
        assert!(!String::from_utf8_lossy(&output.stderr).contains("missing.values"));
    }
    failure(
        &scratch.stdin(
            &[
                "replay",
                "--function",
                "m::p",
                "--witness",
                "-",
                "subject.or",
            ],
            &vec![b' '; MAX_SOURCE_BYTES + 1],
        ),
        1,
        "ORC1003",
    );
    let mut exact_source = b"edition 2026;module m{spec p(x:Int)->Bool{x == 7}}/*".to_vec();
    exact_source.resize(MAX_SOURCE_BYTES - 2, b' ');
    exact_source.extend_from_slice(b"*/");
    scratch.write("good.values", b"[7]");
    assert!(
        success(&scratch.stdin(
            &[
                "replay",
                "--function",
                "m::p",
                "--witness",
                "good.values",
                "-"
            ],
            &exact_source
        ))
        .contains("holds_for_this_witness")
    );
    exact_source.push(b' ');
    failure(
        &scratch.stdin(
            &[
                "replay",
                "--function",
                "m::p",
                "--witness",
                "good.values",
                "-",
            ],
            &exact_source,
        ),
        1,
        "ORC1003",
    );
}

#[test]
#[cfg(target_os = "linux")]
fn replay_preserves_non_utf8_file_paths_and_safe_diagnostics() {
    use std::os::unix::ffi::OsStrExt as _;
    let scratch = Scratch::new("raw-paths");
    let source_name = OsStr::from_bytes(b"subject-\xff.or");
    let witness_name = OsStr::from_bytes(b"witness-\xfe.values");
    let source = scratch.write(
        source_name,
        b"edition 2026;module m{spec p(x:Int)->Bool{x == 7}}",
    );
    let witness = scratch.write(witness_name, b"[7]");
    let output = Command::new(env!("CARGO_BIN_EXE_orangec"))
        .args([
            OsStr::new("replay"),
            OsStr::new("--function"),
            OsStr::new("m::p"),
            OsStr::new("--witness"),
            witness.as_os_str(),
            source.as_os_str(),
        ])
        .output()
        .unwrap();
    assert!(success(&output).starts_with("m::p[]: holds_for_this_witness\n"));
    let missing = scratch.0.join(OsStr::from_bytes(b"missing-\xff.values"));
    let output = Command::new(env!("CARGO_BIN_EXE_orangec"))
        .args([
            OsStr::new("replay"),
            OsStr::new("--function"),
            OsStr::new("m::p"),
            OsStr::new("--witness"),
            missing.as_os_str(),
            source.as_os_str(),
        ])
        .output()
        .unwrap();
    failure(&output, 1, "ORC1001");
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .contains("missing-\\xff.values")
    );
}
