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
        "aes::sbox[]: 256 calls, 843264 steps\nlargest call: 3357 of 1048576 steps\n"
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
        "orangec: option `--table` takes values, ddt, lat, bct, anf, or matrix\n",
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

#[test]
fn aes_mix_columns_is_mds_over_the_aes_field() {
    assert_eq!(
        success(&analyze(&[
            "--function",
            "aes::mix_column",
            "--linear",
            "aes.or"
        ])),
        "aes::mix_column[]  32 bits as 4 words of 8 bits

checked                   the 529 inputs of at most 2 bits: no term of degree 2, higher degrees unchecked
form                      linear
rank                      32 of 32, invertible
fixed points              2^8
involution                no
xor count, row by row     152

differential branch       5 of at most 5 (MDS)
linear branch             5 of at most 5 (MDS)

field                     GF(2^8) modulo 0x11b
field matrix              02 03 01 01
                          01 02 03 01
                          01 01 02 03
                          03 01 01 02
"
    );
    let output = analyze(&[
        "--function=aes::mix_column",
        "--linear",
        "--stats",
        "aes.or",
    ]);
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(
        String::from_utf8(output.stderr).unwrap(),
        "aes::mix_column[]: 529 calls, 61364 steps\nlargest call: 116 of 1048576 steps\n"
    );
    let matrix = success(&analyze(&[
        "--function",
        "aes::mix_column",
        "--linear",
        "--table",
        "matrix",
        "aes.or",
    ]));
    assert_eq!(matrix.lines().count(), 32);
    assert!(matrix.starts_with(
        "y0  00000001 10000001 10000000 10000000\ny1  10000001 11000001 01000000 01000000\n"
    ));
    assert!(matrix.ends_with("y31 00000011 00000001 00000001 00000010\n"));
}

#[test]
fn the_aes_round_layer_has_branch_number_five_of_seventeen() {
    let summary = success(&analyze(&[
        "--function",
        "aes::linear",
        "--linear",
        "aes.or",
    ]));
    assert!(summary.starts_with(
        "aes::linear[]  128 bits as 16 words of 8 bits

checked                   the 8257 inputs of at most 2 bits: no term of degree 2, higher degrees unchecked
form                      linear
rank                      128 of 128, invertible
fixed points              2^16
involution                no
xor count, row by row     608

differential branch       5 of at most 17
linear branch             5 of at most 17

field                     GF(2^8) modulo 0x11b
field matrix              02 00 00 00 00 03 00 00 00 00 01 00 00 00 00 01
                          01 00 00 00 00 02 00 00 00 00 03 00 00 00 00 01
"
    ));
    assert!(
        summary.ends_with(
            "                          00 01 00 00 00 00 01 00 00 00 00 02 03 00 00 00\n"
        )
    );
    assert_eq!(summary.lines().count(), 29);
}

#[test]
fn bit_permutations_and_binary_layers() {
    assert_eq!(
        success(&analyze(&[
            "--function",
            "present::player",
            "--linear",
            "--word",
            "4",
            "present.or"
        ])),
        "present::player[]  64 bits as 16 words of 4 bits

checked                   the 2081 inputs of at most 2 bits: no term of degree 2, higher degrees unchecked
form                      linear
rank                      64 of 64, invertible
fixed points              2^24
involution                no
xor count, row by row     0

differential branch       2 of at most 17
linear branch             2 of at most 17

field                     none: some block is not a product in any GF(2^4)
"
    );
    assert_eq!(
        success(&analyze(&[
            "--function",
            "ascon::sigma0",
            "--linear",
            "--word=1",
            "ascon.or"
        ])),
        "ascon::sigma0[]  64 bits as 64 words of 1 bit

checked                   the 2081 inputs of at most 2 bits: no term of degree 2, higher degrees unchecked
form                      linear
rank                      64 of 64, invertible
fixed points              2^1
involution                no
xor count, row by row     128

differential branch       4 of at most 65
linear branch             4 of at most 65
"
    );
    assert_eq!(
        success(&analyze(&[
            "--function",
            "midori::mix_column",
            "--linear",
            "--word",
            "4",
            "midori.or"
        ])),
        "midori::mix_column[]  16 bits as 4 words of 4 bits

checked                   all 65536 inputs
form                      linear
rank                      16 of 16, invertible
fixed points              2^12
involution                yes
xor count, row by row     32

differential branch       4 of at most 5
linear branch             4 of at most 5

field                     every GF(2^4): each block is 0 or 1
field matrix              0 1 1 1
                          1 0 1 1
                          1 1 0 1
                          1 1 1 0
"
    );
    assert_eq!(
        success(&analyze(&[
            "--function",
            "midori::mix_column",
            "--linear",
            "--word",
            "4",
            "--table",
            "matrix",
            "midori.or"
        ])),
        "y0  0000 1000 1000 1000
y1  0000 0100 0100 0100
y2  0000 0010 0010 0010
y3  0000 0001 0001 0001
y4  1000 0000 1000 1000
y5  0100 0000 0100 0100
y6  0010 0000 0010 0010
y7  0001 0000 0001 0001
y8  1000 1000 0000 1000
y9  0100 0100 0000 0100
y10 0010 0010 0000 0010
y11 0001 0001 0000 0001
y12 1000 1000 1000 0000
y13 0100 0100 0100 0000
y14 0010 0010 0010 0000
y15 0001 0001 0001 0000
"
    );
}

#[test]
fn affine_singular_and_wide_word_layers() {
    assert_eq!(
        success(&analyze(&[
            "--function",
            "aes::affine",
            "--linear",
            "--word",
            "1",
            "aes.or"
        ])),
        "aes::affine[]  8 bits as 8 words of 1 bit

checked                   all 256 inputs
form                      affine, constant 0x63
rank                      8 of 8, invertible
fixed points              none
involution                no
xor count, row by row     32

differential branch       4 of at most 9
linear branch             4 of at most 9
"
    );
    assert_eq!(
        success(&analyze(&[
            "--function",
            "shapes::fold",
            "--linear",
            "shapes.or"
        ])),
        "shapes::fold[]  16 bits as 2 words of 8 bits

checked                   all 65536 inputs
form                      linear
rank                      8 of 16, singular
fixed points              1
involution                no
xor count, row by row     16

differential branch       2 of at most 3
linear branch             2 of at most 3

field                     every GF(2^8): each block is 0 or 1
field matrix              01 01
                          01 01
"
    );
    assert_eq!(
        success(&analyze(&[
            "--function",
            "shapes::wide",
            "--linear",
            "--word",
            "32",
            "shapes.or"
        ])),
        "shapes::wide[]  32 bits as 1 word of 32 bits

checked                   the 529 inputs of at most 2 bits: no term of degree 2, higher degrees unchecked
form                      linear
rank                      32 of 32, invertible
fixed points              2^3
involution                no
xor count, row by row     29

differential branch       not computed: about 2^34 operations, over the limit of 2^32
linear branch             not computed: about 2^34 operations, over the limit of 2^32

field                     not searched for words of more than 8 bits
"
    );
}

#[test]
fn layers_that_are_not_affine_or_not_layers_are_refused() {
    assert_eq!(
        failure(
            &analyze(&["--function", "shapes::spill", "--linear", "shapes.or"]),
            1
        ),
        "error[ORC1017]: `shapes::spill[]` is not affine over GF(2): at input 0x30 it is 0x40, but its values at 0 and at single bits give 0x00
  = note: `--linear` analyzes a map x -> M x + c; analyze an S-box without `--linear`
"
    );
    assert!(
        failure(
            &analyze(&["--function", "aes::sbox", "--linear", "aes.or"]),
            1
        )
        .starts_with(
            "error[ORC1017]: `aes::sbox[]` is not affine over GF(2): at input 0x03 it is 0x7b, but its values at 0 and at single bits give 0x68\n"
        )
    );
    for name in ["shapes::pair", "shapes::count"] {
        assert_eq!(
            failure(&analyze(&["--function", name, "--linear", "shapes.or"]), 1),
            "error[ORC1016]: linear analysis requires one parameter of a word or array type and a result of the same type
  = note: select a function such as `spec mix(a: Word[8]^4) -> Word[8]^4`
"
        );
    }
    assert_eq!(
        failure(
            &analyze(&["--function", "shapes::state", "--linear", "shapes.or"]),
            1
        ),
        "error[ORC1017]: `Word[64]^4` has 256 bits, too wide for a linear layer
  = note: a linear layer is analyzed over at most 128 bits
"
    );
    assert_eq!(
        failure(
            &analyze(&[
                "--function",
                "present::sbox",
                "--linear",
                "--word",
                "16",
                "present.or"
            ]),
            1
        ),
        "error[ORC1017]: words of 16 bits do not divide the 8 bits of `Word[8]`
  = note: `--word W` groups the bits of a layer into words of W bits, a power of two dividing its width
"
    );
    let stopped = failure(
        &analyze(&[
            "--function",
            "aes::mix_column",
            "--linear",
            "--steps",
            "10",
            "aes.or",
        ]),
        1,
    );
    assert!(stopped.starts_with("error[ORC0301]: reference evaluation step limit exceeded\n"));
    assert!(
        stopped.ends_with("  = note: the analysis stopped at input [0x00, 0x00, 0x00, 0x00]\n")
    );
}

#[test]
fn malformed_linear_options_are_usage_errors() {
    for value in ["0", "3", "128", "08", "+8", "8 ", ""] {
        let stderr = failure(
            &analyze(&[
                "--function",
                "aes::mix_column",
                "--linear",
                "--word",
                value,
                "aes.or",
            ]),
            2,
        );
        assert!(
            stderr.starts_with("orangec: option `--word` takes 1, 2, 4, 8, 16, 32, or 64\n"),
            "--word {value:?}: {stderr}"
        );
    }
    for (arguments, message) in [
        (
            &["--linear", "--bits", "4"][..],
            "orangec: option `--bits` does not apply with `--linear`\n",
        ),
        (
            &["--linear", "--table", "ddt"][..],
            "orangec: with `--linear`, option `--table` takes only matrix\n",
        ),
        (
            &["--word", "8"][..],
            "orangec: option `--word` applies only with `--linear`\n",
        ),
        (
            &["--table", "matrix"][..],
            "orangec: table `matrix` requires `--linear`\n",
        ),
        (
            &["--linear", "--linear"][..],
            "orangec: option `--linear` may be specified at most once\n",
        ),
        (
            &["--linear", "--word", "8", "--word=8"][..],
            "orangec: option `--word` may be specified at most once\n",
        ),
    ] {
        let mut all = vec!["--function", "aes::mix_column"];
        all.extend_from_slice(arguments);
        all.push("aes.or");
        let stderr = failure(&analyze(&all), 2);
        assert!(stderr.starts_with(message), "{arguments:?}: {stderr}");
    }
    for (option, value) in [("--linear", None), ("--word", Some("8"))] {
        let mut arguments = vec!["check", option];
        arguments.extend(value);
        arguments.push("aes.or");
        let output = Command::new(env!("CARGO_BIN_EXE_orangec"))
            .current_dir(fixtures())
            .args(&arguments)
            .output()
            .unwrap();
        let stderr = failure(&output, 2);
        assert!(
            stderr.starts_with(&format!(
                "orangec: option `{option}` applies only to analyze\n"
            )),
            "{arguments:?}: {stderr}"
        );
    }
}

#[test]
fn present_trails_match_the_published_bounds() {
    let output = analyze(&[
        "--function",
        "present::sbox",
        "--bits",
        "4",
        "--layer",
        "present::player",
        "--rounds",
        "4",
        "present.or",
    ]);
    assert_eq!(
        success(&output),
        "round                     present::sbox[] on 16 words of 4 bits, then present::player[]
layer checked             the 2081 inputs of at most 2 bits: no term of degree 2, higher degrees unchecked
full diffusion            3 rounds

differential trails       a trail of weight w has probability 2^-w
rounds  active S-boxes  least weight
1       1               2
2       2               4
3       4               8
4       6               12

linear trails             a trail of weight w has correlation 2^-w in magnitude
rounds  active S-boxes  least weight
1       1               1
2       2               2
3       3               4
4       4               6
"
    );
    let output = analyze(&[
        "--function=present::sbox",
        "--bits=4",
        "--layer=present::player",
        "--rounds=1",
        "--stats",
        "present.or",
    ]);
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(
        String::from_utf8(output.stderr).unwrap(),
        "present::sbox[]: 16 calls, 608 steps
largest call: 38 of 1048576 steps
present::player[]: 2081 calls, 2784378 steps
largest call: 1338 of 1048576 steps
"
    );
}

#[test]
fn aes_trails_meet_the_wide_trail_bound_over_two_rounds() {
    assert_eq!(
        success(&analyze(&[
            "--function",
            "aes::sbox",
            "--layer",
            "aes::linear",
            "--rounds",
            "2",
            "aes.or",
        ])),
        "round                     aes::sbox[] on 16 words of 8 bits, then aes::linear[]
layer checked             the 8257 inputs of at most 2 bits: no term of degree 2, higher degrees unchecked
full diffusion            2 rounds

differential trails       a trail of weight w has probability 2^-w
rounds  active S-boxes  least weight
1       1               6
2       5               30

linear trails             weights not computed: some |W(a, b)| is not a power of two
rounds  active S-boxes
1       1
2       5
"
    );
}

#[test]
fn heys_trails_count_active_sboxes_only() {
    assert_eq!(
        success(&analyze(&[
            "--function",
            "heys::sbox",
            "--bits",
            "4",
            "--layer",
            "heys::permute",
            "--rounds",
            "4",
            "heys.or",
        ])),
        "round                     heys::sbox[] on 4 words of 4 bits, then heys::permute[]
layer checked             all 65536 inputs
full diffusion            2 rounds

differential trails       weights not computed: some DDT entry is not a power of two
rounds  active S-boxes
1       1
2       2
3       4
4       6

linear trails             weights not computed: some |W(a, b)| is not a power of two
rounds  active S-boxes
1       1
2       2
3       3
4       4
"
    );
}

#[test]
fn rounds_that_are_not_substitution_permutation_rounds_are_refused() {
    let scratch = Scratch::new("trails");
    scratch.write(
        "network.or",
        b"edition 2026;
module network {
  spec sbox(x: Word[8]) -> Word[8] {
    let table: Word[8]^16 = [0xc, 0x5, 0x6, 0xb, 0x9, 0x0, 0xa, 0xd, 0x3, 0xe, 0xf, 0x8, 0x4, 0x7, 0x1, 0x2];
    table[x & 0x0f]
  }
  spec same(x: Word[16]) -> Word[16] { x }
  spec rotate(x: Word[16]) -> Word[16] { x <<< 4 }
  spec fold(x: Word[16]) -> Word[16] { x ^ (x <<< 8) }
  spec spill(x: Word[16]) -> Word[16] { x + 1 }
  spec pair(x: Word[16], y: Word[16]) -> Word[16] { x ^ y }
}
",
    );
    let refused = |arguments: &[&str]| {
        let mut all = arguments.to_vec();
        all.extend(["--rounds", "2", "network.or"]);
        failure(&scratch.analyze(&all), 1)
    };
    assert_eq!(
        refused(&["--function", "network::sbox", "--layer", "network::fold"]),
        "error[ORC1017]: `network::sbox[]` is not a permutation of 8 bits
  = note: a trail search needs an invertible S-box; `--bits N` selects its low N bits
"
    );
    for (bits, noun) in [("1", "bit"), ("9", "bits")] {
        assert_eq!(
            refused(&[
                "--function",
                "network::same",
                "--bits",
                bits,
                "--layer",
                "network::rotate"
            ]),
            format!(
                "error[ORC1017]: an S-box of {bits} {noun} is outside the 2 to 8 bits of a trail search
  = note: `--bits N` selects the low N bits of the S-box's parameter and result
"
            )
        );
    }
    assert_eq!(
        refused(&[
            "--function",
            "network::same",
            "--bits",
            "3",
            "--layer",
            "network::rotate"
        ]),
        "error[ORC1017]: the 16 bits of `Word[16]` are not a whole number of 3-bit S-boxes
  = note: a round applies the S-box to each word of the layer's bits, word c holding bits c s through c s + s - 1
"
    );
    assert_eq!(
        refused(&[
            "--function",
            "network::sbox",
            "--bits",
            "4",
            "--layer",
            "network::fold"
        ]),
        "error[ORC1017]: `network::fold[]` is not invertible over GF(2)
  = note: a round's linear layer must be a bijection; `--linear` reports its rank
"
    );
    assert_eq!(
        refused(&[
            "--function",
            "network::sbox",
            "--bits",
            "4",
            "--layer",
            "network::spill"
        ]),
        "error[ORC1017]: `network::spill[]` is not affine over GF(2): at input 0x0003 it is 0x0004, but its values at 0 and at single bits give 0x0000
  = note: `--layer` selects the linear layer of a round, a map x -> M x + c
"
    );
    assert_eq!(
        refused(&[
            "--function",
            "network::sbox",
            "--bits",
            "4",
            "--layer",
            "network::pair"
        ]),
        "error[ORC1016]: linear analysis requires one parameter of a word or array type and a result of the same type
  = note: select a function such as `spec mix(a: Word[8]^4) -> Word[8]^4`
"
    );
    assert_eq!(
        refused(&[
            "--function",
            "network::sbox",
            "--bits",
            "4",
            "--layer",
            "network::none"
        ]),
        "error[ORC1016]: no function `network::none` without size or type parameters
  = note: `--layer` names a linked module's function with no size or type parameters
"
    );
    let output = scratch.analyze(&[
        "--function",
        "network::sbox",
        "--bits",
        "4",
        "--layer",
        "network::rotate",
        "--rounds",
        "3",
        "network.or",
    ]);
    let report = success(&output);
    assert!(
        report.contains(
            "full diffusion            never: some output bit depends on some input bit after no number of rounds\n"
        ),
        "{report}"
    );
    assert!(
        report.contains(
            "1       1               2\n2       2               4\n3       3               6\n"
        ),
        "{report}"
    );
}

#[test]
fn malformed_trail_options_are_usage_errors() {
    let usage = |arguments: &[&str]| {
        let mut all = vec!["--function", "present::sbox", "--bits", "4"];
        all.extend_from_slice(arguments);
        all.push("present.or");
        failure(&analyze(&all), 2)
    };
    for value in ["0", "33", "01", "+3", "3 ", "", "x"] {
        let stderr = usage(&["--layer", "present::player", "--rounds", value]);
        assert!(
            stderr.starts_with(
                "orangec: option `--rounds` takes a number of rounds from 1 through 32\n"
            ),
            "--rounds {value:?}: {stderr}"
        );
    }
    for value in [
        "present",
        "present::",
        "::player",
        "a::b::c",
        "1a::b",
        "a-b::c",
    ] {
        let stderr = usage(&["--layer", value, "--rounds", "2"]);
        assert!(
            stderr
                .starts_with("orangec: option `--layer` takes exactly MODULE::NAME identifiers\n"),
            "--layer {value:?}: {stderr}"
        );
    }
    for (arguments, message) in [
        (
            &["--layer", "present::player"][..],
            "orangec: option `--layer` requires `--rounds`\n",
        ),
        (
            &["--rounds", "2"][..],
            "orangec: option `--rounds` requires `--layer`\n",
        ),
        (
            &["--layer", "present::player", "--rounds", "2", "--linear"][..],
            "orangec: option `--layer` does not apply with `--linear`\n",
        ),
        (
            &[
                "--layer",
                "present::player",
                "--rounds",
                "2",
                "--table",
                "ddt",
            ][..],
            "orangec: option `--table` does not apply with `--layer`\n",
        ),
        (
            &["--layer", "present::player", "--rounds", "2", "--word", "4"][..],
            "orangec: option `--word` applies only with `--linear`\n",
        ),
        (
            &[
                "--layer",
                "present::player",
                "--layer=present::player",
                "--rounds",
                "2",
            ][..],
            "orangec: option `--layer` may be specified at most once\n",
        ),
        (
            &["--layer", "present::player", "--rounds", "2", "--rounds=2"][..],
            "orangec: option `--rounds` may be specified at most once\n",
        ),
    ] {
        let stderr = usage(arguments);
        assert!(stderr.starts_with(message), "{arguments:?}: {stderr}");
    }
    for (option, value) in [("--layer", "present::player"), ("--rounds", "2")] {
        let output = Command::new(env!("CARGO_BIN_EXE_orangec"))
            .current_dir(fixtures())
            .args(["check", option, value, "present.or"])
            .output()
            .unwrap();
        let stderr = failure(&output, 2);
        assert!(
            stderr.starts_with(&format!(
                "orangec: option `{option}` applies only to analyze\n"
            )),
            "{option}: {stderr}"
        );
    }
}
