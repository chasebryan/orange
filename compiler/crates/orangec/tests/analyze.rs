//! Command-line contracts for `orangec analyze`.
//!
//! Every summary and table below was computed by evaluating the fixture at
//! every input and is pinned exactly; the published values for the AES,
//! PRESENT, Ascon and DES S-boxes are cited in `docs/CRYPTANALYSIS_2026.md`.
//! Scratch roots are fixed by Cargo at build time and created exclusively.

use std::fs::{self, OpenOptions};
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn fixtures() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/analyze")
}

fn analyze(arguments: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_orangec"))
        .current_dir(fixtures())
        .arg("analyze")
        .args(arguments)
        .output()
        .unwrap()
}

fn success(output: &Output) -> String {
    assert_eq!(
        output.status.code(),
        Some(0),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stderr.is_empty());
    String::from_utf8(output.stdout.clone()).unwrap()
}

fn failure(output: &Output, status: i32) -> String {
    assert_eq!(
        output.status.code(),
        Some(status),
        "stdout: {}",
        String::from_utf8_lossy(&output.stdout)
    );
    assert!(output.stdout.is_empty());
    String::from_utf8(output.stderr.clone()).unwrap()
}

struct Scratch(PathBuf);

impl Scratch {
    fn new(name: &str) -> Self {
        let root = Path::new(env!("CARGO_TARGET_TMPDIR"));
        fs::create_dir_all(root).unwrap();
        let path = root.join(format!("analyze-{name}-{}", std::process::id()));
        fs::create_dir(&path).unwrap();
        Self(path)
    }

    fn write(&self, name: &str, bytes: &[u8]) {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(self.0.join(name))
            .unwrap();
        file.write_all(bytes).unwrap();
    }

    fn analyze(&self, arguments: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_orangec"))
            .current_dir(&self.0)
            .arg("analyze")
            .args(arguments)
            .output()
            .unwrap()
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn the_aes_sbox_has_its_published_properties() {
    assert_eq!(
        success(&analyze(&["--function", "aes::sbox", "aes.or"])),
        "aes::sbox[]  8 bits to 8 bits

bijective                 yes
fixed points              0
cycle type                87 81 59 27 2

differential uniformity   4 (probability 2^-6, 255 pairs)
differential spectrum     0: 32895  2: 32130  4: 255
differential branch       2

linearity                 32 (nonlinearity 112, correlation 2^-3)
walsh spectrum            0: 4335  4: 12240  8: 9180  12: 10200  16: 8670  20: 6120  24: 9180  28: 4080  32: 1275
linear branch             2

algebraic degree          7 (every component)
inverse degree            7
quadratic equations       39 (23 bi-affine)
boomerang uniformity      6
"
    );
    let output = analyze(&["--function", "aes::sbox", "--stats", "aes.or"]);
    assert_eq!(output.status.code(), Some(0));
    assert!(
        String::from_utf8(output.stdout)
            .unwrap()
            .starts_with("aes::sbox[]  8 bits")
    );
    assert_eq!(
        String::from_utf8(output.stderr).unwrap(),
        "aes::sbox[]: 256 calls, 843008 steps\nlargest call: 3356 of 1048576 steps\n"
    );
}

#[test]
fn the_present_sbox_has_its_published_properties() {
    assert_eq!(
        success(&analyze(&[
            "--function",
            "present::sbox",
            "--bits",
            "4",
            "present.or"
        ])),
        "present::sbox[]  4 bits to 4 bits

bijective                 yes
fixed points              0
cycle type                7 4 3 2

differential uniformity   4 (probability 2^-2, 24 pairs)
differential spectrum     0: 144  2: 72  4: 24
differential branch       3

linearity                 8 (nonlinearity 4, correlation 2^-1)
walsh spectrum            0: 108  4: 96  8: 36
linear branch             2

algebraic degree          3 (components 2 to 3)
inverse degree            3
quadratic equations       21 (9 bi-affine)
boomerang uniformity      16
"
    );
}

#[test]
fn ascon_and_chi_share_spectra_but_not_branches() {
    assert_eq!(
        success(&analyze(&[
            "--function",
            "ascon::sbox",
            "--bits",
            "5",
            "ascon.or"
        ])),
        "ascon::sbox[]  5 bits to 5 bits

bijective                 yes
fixed points              0
cycle type                26 6

differential uniformity   8 (probability 2^-2, 20 pairs)
differential spectrum     0: 676  2: 176  4: 120  8: 20
differential branch       3

linearity                 16 (nonlinearity 8, correlation 2^-1)
walsh spectrum            0: 616  8: 336  16: 40
linear branch             3

algebraic degree          2 (every component)
inverse degree            3
quadratic equations       25 (10 bi-affine)
boomerang uniformity      16
"
    );
    assert_eq!(
        success(&analyze(&["--function=ascon::chi", "--bits=5", "ascon.or"])),
        "ascon::chi[]  5 bits to 5 bits

bijective                 yes
fixed points              2
cycle type                4^5 2^5 1^2

differential uniformity   8 (probability 2^-2, 20 pairs)
differential spectrum     0: 676  2: 176  4: 120  8: 20
differential branch       2

linearity                 16 (nonlinearity 8, correlation 2^-1)
walsh spectrum            0: 616  8: 336  16: 40
linear branch             2

algebraic degree          2 (every component)
inverse degree            3
quadratic equations       25 (10 bi-affine)
boomerang uniformity      16
"
    );
}

#[test]
fn des_s1_is_a_balanced_six_to_four_bit_function() {
    assert_eq!(
        success(&analyze(&[
            "--function",
            "des::s1",
            "--bits",
            "6,4",
            "des.or"
        ])),
        "des::s1[]  6 bits to 4 bits

balanced                  yes

differential uniformity   16 (probability 2^-2, 1 pair)
differential spectrum     0: 195  2: 246  4: 232  6: 168  8: 84  10: 46  12: 24  14: 12  16: 1
differential branch       2

linearity                 36 (nonlinearity 14, correlation 9/2^4)
walsh spectrum            0: 243  4: 311  8: 219  12: 116  16: 41  20: 18  24: 9  28: 2  36: 1
linear branch             2

algebraic degree          5 (components 4 to 5)
quadratic equations       1 (0 bi-affine)
"
    );
    assert_eq!(
        success(&analyze(&[
            "--function",
            "des::s1",
            "--bits",
            "6,4",
            "--table",
            "values",
            "des.or"
        ])),
        "00 e 0 4 f d 7 1 4 2 e f 2 b d 8 1
10 3 a a 6 6 c c b 5 9 9 5 0 3 7 8
20 4 f 1 c e 8 8 2 d 4 6 9 2 1 b 7
30 f 5 c b 9 3 7 e 3 a a 0 5 6 0 d
"
    );
}

#[test]
fn boolean_results_report_weight_indicator_and_immunity() {
    assert_eq!(
        success(&analyze(&[
            "--function",
            "boolean::majority",
            "--bits",
            "3",
            "boolean.or"
        ])),
        "boolean::majority[]  3 bits to 1 bit

balanced                  yes
weight                    4 of 8

differential uniformity   8 (probability 1, 1 pair)
differential spectrum     0: 1  4: 12  8: 1
absolute indicator        8

linearity                 4 (nonlinearity 2, correlation 2^-1)
walsh spectrum            0: 4  4: 4
correlation immunity      0

algebraic degree          2
quadratic equations       3 (0 bi-affine)
"
    );
    assert_eq!(
        success(&analyze(&[
            "--function",
            "boolean::bent",
            "--bits",
            "4",
            "boolean.or"
        ])),
        "boolean::bent[]  4 bits to 1 bit

balanced                  no
weight                    6 of 16

differential uniformity   8 (probability 2^-1, 30 pairs)
differential spectrum     8: 30
absolute indicator        0

linearity                 4 (nonlinearity 6, correlation 2^-2)
walsh spectrum            4: 16
correlation immunity      0

algebraic degree          2
quadratic equations       1 (0 bi-affine)
"
    );
    assert_eq!(
        success(&analyze(&[
            "--function",
            "boolean::majority",
            "--bits",
            "3",
            "--table",
            "anf",
            "boolean.or"
        ])),
        "y0 = x0x1 + x0x2 + x1x2\n"
    );
    assert_eq!(
        success(&analyze(&[
            "--function",
            "boolean::majority",
            "--bits",
            "3",
            "--table",
            "lat",
            "boolean.or"
        ])),
        "   0  1
0  4  0
1  0  2
2  0  2
3  0  0
4  0  2
5  0  0
6  0  0
7  0 -2
"
    );
}

#[test]
fn the_present_tables_print_in_full() {
    let table = |name: &str| {
        success(&analyze(&[
            "--function",
            "present::sbox",
            "--bits",
            "4",
            "--table",
            name,
            "present.or",
        ]))
    };
    assert_eq!(table("values"), "0 c 5 6 b 9 0 a d 3 e f 8 4 7 1 2\n");
    assert_eq!(
        table("ddt"),
        "   0  1  2  3  4  5  6  7  8  9  a  b  c  d  e  f
0 16  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0
1  0  0  0  4  0  0  0  4  0  4  0  0  0  4  0  0
2  0  0  0  2  0  4  2  0  0  0  2  0  2  2  2  0
3  0  2  0  2  2  0  4  2  0  0  2  2  0  0  0  0
4  0  0  0  0  0  4  2  2  0  2  2  0  2  0  2  0
5  0  2  0  0  2  0  0  0  0  2  2  2  4  2  0  0
6  0  0  2  0  0  0  2  0  2  0  0  4  2  0  0  4
7  0  4  2  0  0  0  2  0  2  0  0  0  2  0  0  4
8  0  0  0  2  0  0  0  2  0  2  0  4  0  2  0  4
9  0  0  2  0  4  0  2  0  2  0  0  0  2  0  4  0
a  0  0  2  2  0  4  0  0  2  0  2  0  0  2  2  0
b  0  2  0  0  2  0  0  0  4  2  2  2  0  2  0  0
c  0  0  2  0  0  4  0  2  2  2  2  0  0  0  2  0
d  0  2  4  2  2  0  0  2  0  0  2  2  0  0  0  0
e  0  0  2  2  0  0  2  2  2  2  0  0  2  2  0  0
f  0  4  0  0  4  0  0  0  0  0  0  0  0  0  4  4
"
    );
    assert_eq!(
        table("lat"),
        "   0  1  2  3  4  5  6  7  8  9  a  b  c  d  e  f
0  8  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0
1  0  0  0  0  0 -4  0 -4  0  0  0  0  0 -4  0  4
2  0  0  2  2 -2 -2  0  0  2 -2  0  4  0  4 -2  2
3  0  0  2  2  2 -2 -4  0 -2  2 -4  0  0  0 -2 -2
4  0  0 -2  2 -2 -2  0  4 -2 -2  0 -4  0  0 -2  2
5  0  0 -2  2 -2  2  0  0  2  2 -4  0  4  0  2  2
6  0  0  0 -4  0  0 -4  0  0 -4  0  0  4  0  0  0
7  0  0  0  4  4  0  0  0  0 -4  0  0  0  0  4  0
8  0  0  2 -2  0  0 -2  2 -2  2  0  0 -2  2  4  4
9  0  4 -2 -2  0  0  2 -2 -2 -2 -4  0 -2  2  0  0
a  0  0  4  0  2  2  2 -2  0  0  0 -4  2  2 -2  2
b  0 -4  0  0 -2 -2  2 -2 -4  0  0  0  2  2  2 -2
c  0  0  0  0 -2 -2 -2 -2  4  0  0 -4 -2  2  2 -2
d  0  4  4  0 -2 -2  2  2  0  0  0  0  2 -2  2 -2
e  0  0  2  2 -4  4 -2 -2 -2 -2  0  0 -2 -2  0  0
f  0  4 -2  2  0  0 -2 -2 -2  2  4  0  2  2  0  0
"
    );
    assert_eq!(
        table("bct"),
        "   0  1  2  3  4  5  6  7  8  9  a  b  c  d  e  f
0 16 16 16 16 16 16 16 16 16 16 16 16 16 16 16 16
1 16  0  4  4  0 16  4  4  4  4  0  0  4  4  0  0
2 16  0  0  6  0  4  6  0  0  0  2  0  2  2  2  0
3 16  2  0  6  2  4  4  2  0  0  2  2  0  0  0  0
4 16  0  0  0  0  4  2  2  0  6  2  0  6  0  2  0
5 16  2  0  0  2  4  0  0  0  6  2  2  4  2  0  0
6 16  4  2  0  4  0  2  0  2  0  0  4  2  0  4  8
7 16  4  2  0  4  0  2  0  2  0  0  4  2  0  4  8
8 16  4  0  2  4  0  0  2  0  2  0  4  0  2  4  8
9 16  4  2  0  4  0  2  0  2  0  0  4  2  0  4  8
a 16  0  2  2  0  4  0  0  6  0  2  0  0  6  2  0
b 16  2  0  0  2  4  0  0  4  2  2  2  0  6  0  0
c 16  0  6  0  0  4  0  6  2  2  2  0  0  0  2  0
d 16  2  4  2  2  4  0  6  0  0  2  2  0  0  0  0
e 16  0  2  2  0  0  2  2  2  2  0  0  2  2  0  0
f 16  8  0  0  8  0  0  0  0  0  0  8  0  0  8 16
"
    );
    assert_eq!(
        table("anf"),
        "y0 = x0 + x2 + x3 + x1x2
y1 = x1 + x3 + x1x3 + x2x3 + x0x1x2 + x0x1x3 + x0x2x3
y2 = 1 + x2 + x3 + x0x1 + x0x3 + x1x3 + x0x1x3 + x0x2x3
y3 = 1 + x0 + x1 + x3 + x1x2 + x0x1x2 + x0x1x3 + x0x2x3
"
    );
}

#[test]
fn narrowed_and_injective_functions_and_the_operation_limit() {
    assert_eq!(
        success(&analyze(&[
            "--function",
            "shapes::spill",
            "--bits",
            "4,5",
            "shapes.or"
        ])),
        "shapes::spill[]  4 bits to 5 bits

injective                 yes

differential uniformity   16 (probability 1, 15 pairs)
differential spectrum     0: 465  16: 15
differential branch       2

linearity                 16 (nonlinearity 0, correlation 1)
walsh spectrum            0: 465  16: 31
linear branch             1

algebraic degree          1 (components 0 to 1)
quadratic equations       35 (19 bi-affine)
"
    );
    assert_eq!(
        success(&analyze(&[
            "--function",
            "shapes::wide",
            "--bits",
            "12",
            "shapes.or"
        ])),
        "shapes::wide[]  12 bits to 12 bits

bijective                 yes
fixed points              8
cycle type                4^1008 2^28 1^8

differential uniformity   4096 (probability 1, 4095 pairs)
differential spectrum     0: 16769025  4096: 4095
differential branch       2

linearity                 4096 (nonlinearity 0, correlation 1)
walsh spectrum            0: 16769025  4096: 4095
linear branch             2

algebraic degree          1 (every component)
inverse degree            1
quadratic equations       222 (90 bi-affine)
boomerang uniformity      not computed: about 2^36 operations, over the limit of 2^32
"
    );
}

#[test]
fn instances_select_the_analyzed_width() {
    let scratch = Scratch::new("instances");
    scratch.write(
        "gray.or",
        b"edition 2026;module g{spec gray[T in {Word[8],Word[16]}](x:T)->T{x ^ (x >> 1)}}",
    );
    let narrow =
        success(&scratch.analyze(&["--function", "g::gray", "--instance", "0", "gray.or"]));
    assert!(narrow.starts_with("g::gray[0]  8 bits to 8 bits\n\nbijective                 yes\nfixed points              2\ncycle type                8^30 4^3 2 1^2\n"));
    let wide = success(&scratch.analyze(&[
        "--function",
        "g::gray",
        "--instance=1",
        "--bits",
        "6",
        "gray.or",
    ]));
    assert!(wide.starts_with("g::gray[1]  6 bits to 6 bits\n"));
    assert!(wide.contains("\nalgebraic degree          1 (every component)\n"));
    let missing = failure(&scratch.analyze(&["--function", "g::gray", "gray.or"]), 1);
    assert!(
        missing.starts_with(
            "error[ORC1016]: no function `g::gray` has the selected numeric instance\n"
        )
    );
}

#[test]
fn unanalyzable_functions_are_refused_with_a_diagnostic() {
    for name in ["shapes::pair", "shapes::count"] {
        assert_eq!(
            failure(&analyze(&["--function", name, "shapes.or"]), 1),
            "error[ORC1016]: analysis requires one word parameter and a word or Bool result
  = note: select a function such as `spec sbox(x: Word[8]) -> Word[8]`
"
        );
    }
    assert_eq!(
        failure(
            &analyze(&["--function", "shapes::spill", "--bits", "4", "shapes.or"]),
            1
        ),
        "error[ORC1017]: `shapes::spill[]` at input 0x0 is 0x10, which has more than 4 output bits
  = note: `--bits N,M` analyzes N input bits and M output bits
"
    );
    assert_eq!(
        failure(&analyze(&["--function", "shapes::wide", "shapes.or"]), 1),
        "error[ORC1017]: a 32-bit to 32-bit function is too wide to analyze completely
  = note: `--bits N,M` selects the low N input and M output bits, at most 16 each
"
    );
    assert!(
        failure(&analyze(&["--function", "present::sbox", "--bits", "16,16", "present.or"]), 1)
            .starts_with("error[ORC1017]: `--bits` selects 16 input and 16 output bits, wider than `Word[8] -> Word[8]`\n")
    );
    assert_eq!(
        failure(
            &analyze(&[
                "--function",
                "shapes::wide",
                "--bits",
                "11",
                "--table",
                "ddt",
                "shapes.or"
            ]),
            1
        ),
        "error[ORC1017]: complete tables are printed for at most 10 input and 10 output bits
  = note: the summary, without `--table`, covers up to 16 bits
"
    );
    assert_eq!(
        failure(
            &analyze(&[
                "--function",
                "des::s1",
                "--bits",
                "6,4",
                "--table",
                "bct",
                "des.or"
            ]),
            1
        ),
        "error[ORC1017]: the boomerang connectivity table needs a permutation
  = note: the analyzed function is not a bijection of its input bits
"
    );
    let stopped = failure(
        &analyze(&[
            "--function",
            "present::sbox",
            "--bits",
            "4",
            "--steps",
            "3",
            "present.or",
        ]),
        1,
    );
    assert!(stopped.starts_with("error[ORC0301]: reference evaluation step limit exceeded\n"));
    assert!(stopped.contains(
        "  = note: `orangec analyze --steps N` sets the budget, up to 1073741824 steps\n"
    ));
    assert!(stopped.ends_with("  = note: the analysis stopped at input 0x0\n"));
}

#[test]
fn malformed_analysis_options_are_usage_errors() {
    let bits = "orangec: option `--bits` takes N or N,M, input and output bits from 1 through 16\n";
    for value in ["0", "17", "04", "4,0", "4,4,4", "4,", ",4", "+4", "4 ", ""] {
        let stderr = failure(
            &analyze(&["--function", "present::sbox", "--bits", value, "present.or"]),
            2,
        );
        assert!(stderr.starts_with(bits), "--bits {value:?}: {stderr}");
    }
    let usage = |arguments: &[&str], message: &str| {
        let output = Command::new(env!("CARGO_BIN_EXE_orangec"))
            .current_dir(fixtures())
            .args(arguments)
            .output()
            .unwrap();
        let stderr = failure(&output, 2);
        assert!(stderr.starts_with(message), "{arguments:?}: {stderr}");
    };
    usage(
        &[
            "analyze",
            "--function",
            "present::sbox",
            "--table",
            "walsh",
            "present.or",
        ],
        "orangec: option `--table` takes values, ddt, lat, bct, or anf\n",
    );
    usage(
        &["check", "--bits", "4", "present.or"],
        "orangec: option `--bits` applies only to analyze\n",
    );
    usage(
        &[
            "replay",
            "--function",
            "present::sbox",
            "--witness",
            "w",
            "--table",
            "ddt",
            "present.or",
        ],
        "orangec: option `--table` applies only to analyze\n",
    );
    usage(
        &["analyze", "present.or"],
        "orangec: command `analyze` requires `--function MODULE::NAME`\n",
    );
    usage(
        &[
            "analyze",
            "--function",
            "present::sbox",
            "--bits",
            "4",
            "--bits",
            "4",
            "present.or",
        ],
        "orangec: option `--bits` may be specified at most once\n",
    );
    usage(
        &[
            "analyze",
            "--function",
            "present::sbox",
            "--table",
            "ddt",
            "--table",
            "lat",
            "present.or",
        ],
        "orangec: option `--table` may be specified at most once\n",
    );
    usage(
        &[
            "analyze",
            "--function",
            "present::sbox",
            "--witness",
            "w",
            "present.or",
        ],
        "orangec: option `--witness` applies only to replay\n",
    );
    usage(
        &[
            "analyze",
            "--function",
            "present::sbox",
            "present.or",
            "des.or",
        ],
        "orangec: command `analyze` requires exactly one source file\n",
    );
    usage(
        &["analyze", "--function", "present::sbox"],
        "orangec: command `analyze` requires at least one source file\n",
    );
}
