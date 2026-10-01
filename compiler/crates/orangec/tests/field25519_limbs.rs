//! Executable evidence for the OEP-0022 P2 mathematical representations.
//!
//! An independent binary long-division reference checks exact reconstruction,
//! residues, and canonical digits through the real CLI. These examples do not
//! establish universally checked representation contracts or native properties.

use std::collections::BTreeMap;
use std::fs;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

const SOURCE: &str = include_str!("../../../../algorithms/x25519/field25519-limbs.or");
const RADIX: u64 = 1 << 51;
const PRIME: [u64; 4] = [u64::MAX - 18, u64::MAX, u64::MAX, (1 << 63) - 1];
static NEXT: AtomicUsize = AtomicUsize::new(0);

struct Program(PathBuf);

impl Program {
    fn new(extra: &str) -> Self {
        // Cargo fixes the scratch root at build time; runtime temporary-path
        // environment variables cannot redirect these generated programs.
        let path = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!(
            "orange-field25519-{}-{}.or",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let definitions = SOURCE.trim_end().strip_suffix('}').unwrap();
        fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .unwrap()
            .write_all(format!("{definitions}\n{extra}\n}}\n").as_bytes())
            .unwrap();
        Self(path)
    }
}

impl Drop for Program {
    fn drop(&mut self) {
        fs::remove_file(&self.0).unwrap();
    }
}

fn run(path: &Path, arguments: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_orangec"))
        .args(arguments)
        .arg(path)
        .output()
        .unwrap()
}

fn successful(path: &Path, arguments: &[&str]) -> Output {
    let first = run(path, arguments);
    assert!(
        first.status.success(),
        "{arguments:?}: {}{}",
        String::from_utf8_lossy(&first.stdout),
        String::from_utf8_lossy(&first.stderr)
    );
    let second = run(path, arguments);
    assert_eq!(first.status.code(), second.status.code());
    assert_eq!(first.stdout, second.stdout);
    assert_eq!(first.stderr, second.stderr);
    first
}

fn values(output: &Output) -> BTreeMap<String, String> {
    String::from_utf8(output.stdout.clone())
        .unwrap()
        .lines()
        .map(|line| {
            let (qualified, value) = line.split_once(": ").unwrap();
            (
                qualified.strip_prefix("field25519::").unwrap().to_owned(),
                value.to_owned(),
            )
        })
        .collect()
}

fn literal(limbs: &[u64; 5]) -> String {
    format!("[{}]", limbs.map(|limb| limb.to_string()).join(", "))
}

fn word_value(limbs: &[u64; 5]) -> String {
    format!(
        "Word[64]^5 = [{}]",
        limbs.map(|limb| format!("0x{limb:016x}")).join(", ")
    )
}

// Reconstruct into a 320-bit binary integer by adding each set storage bit at
// its mathematical position. This does not reuse Orange's Horner schedule.
fn reconstruct(limbs: &[u64; 5]) -> [u64; 5] {
    let mut integer = [0u64; 5];
    for (limb_index, limb) in limbs.iter().enumerate() {
        for bit in 0..64 {
            if limb & (1 << bit) == 0 {
                continue;
            }
            let position = 51 * limb_index + bit;
            let mut word = position / 64;
            let mut carry = 1 << (position % 64);
            loop {
                let (sum, overflow) = integer[word].overflowing_add(carry);
                integer[word] = sum;
                if !overflow {
                    break;
                }
                word += 1;
                carry = 1;
            }
        }
    }
    integer
}

fn decimal(mut words: [u64; 5]) -> String {
    let mut digits = Vec::new();
    while words != [0; 5] {
        let mut remainder = 0u128;
        for word in words.iter_mut().rev() {
            let dividend = (remainder << 64) | u128::from(*word);
            *word = (dividend / 10) as u64;
            remainder = dividend % 10;
        }
        digits.push(b'0' + remainder as u8);
    }
    if digits.is_empty() {
        return String::from("0");
    }
    digits.reverse();
    String::from_utf8(digits).unwrap()
}

fn at_least_prime(words: &[u64; 4]) -> bool {
    words.iter().rev().cmp(PRIME.iter().rev()).is_ge()
}

// Binary long division by p; the accumulator is a canonical 256-bit residue
// throughout. There is no radix-2^51 carry or 19-fold in this reference.
fn residue(integer: &[u64; 5]) -> [u64; 4] {
    let mut result = [0u64; 4];
    for position in (0..320).rev() {
        let mut incoming = (integer[position / 64] >> (position % 64)) & 1;
        for word in &mut result {
            let outgoing = *word >> 63;
            *word = (*word << 1) | incoming;
            incoming = outgoing;
        }
        assert_eq!(incoming, 0);
        if at_least_prime(&result) {
            let mut borrow = false;
            for (word, subtrahend) in result.iter_mut().zip(PRIME) {
                let (difference, first) = word.overflowing_sub(subtrahend);
                let (difference, second) = difference.overflowing_sub(u64::from(borrow));
                *word = difference;
                borrow = first || second;
            }
            assert!(!borrow);
        }
    }
    result
}

fn residue_decimal(words: [u64; 4]) -> String {
    decimal([words[0], words[1], words[2], words[3], 0])
}

fn canonical_digits(words: &[u64; 4]) -> [u64; 5] {
    std::array::from_fn(|limb| {
        let mut digit = 0;
        for bit in 0..51 {
            let position = 51 * limb + bit;
            digit |= ((words[position / 64] >> (position % 64)) & 1) << bit;
        }
        digit
    })
}

fn parse_word_value(value: &str) -> [u64; 5] {
    let words: Vec<u64> = value
        .strip_prefix("Word[64]^5 = [")
        .unwrap()
        .strip_suffix(']')
        .unwrap()
        .split(", ")
        .map(|word| u64::from_str_radix(word.strip_prefix("0x").unwrap(), 16).unwrap())
        .collect();
    words.try_into().unwrap()
}

fn cases() -> Vec<[u64; 5]> {
    let mut cases = vec![
        [0; 5],
        [RADIX - 1; 5],
        [2 * RADIX - 1; 5],
        [u64::MAX; 5],
        [RADIX - 20, RADIX - 1, RADIX - 1, RADIX - 1, RADIX - 1],
        [RADIX - 19, RADIX - 1, RADIX - 1, RADIX - 1, RADIX - 1],
        [RADIX - 18, RADIX - 1, RADIX - 1, RADIX - 1, RADIX - 1],
        [
            RADIX - 1,
            2 * RADIX - 1,
            2 * RADIX - 2,
            2 * RADIX - 2,
            2 * RADIX - 2,
        ],
    ];
    for axis in 0..5 {
        for bound in [
            RADIX - 1,
            RADIX,
            RADIX + 1,
            2 * RADIX - 1,
            2 * RADIX,
            2 * RADIX + 1,
        ] {
            let mut limbs = [0; 5];
            limbs[axis] = bound;
            cases.push(limbs);
        }
    }
    let mut state = 0x6a09_e667_f3bc_c909u64;
    for index in 0..80 {
        let limbs = std::array::from_fn(|_| {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            state % (if index % 2 == 0 { RADIX } else { 2 * RADIX })
        });
        cases.push(limbs);
    }
    cases
}

#[test]
fn field25519_named_boundaries_and_examples_are_repeatable() {
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../algorithms/x25519/field25519-limbs.or");
    let checked = successful(&source, &["check"]);
    assert!(checked.stdout.is_empty() && checked.stderr.is_empty());
    let evaluated = successful(&source, &["eval", "--stats"]);
    let values = values(&evaluated);
    let mut pairs = 0;
    for (name, expected) in &values {
        if let Some(computed) = name.strip_suffix("_expected") {
            assert_eq!(values.get(computed), Some(expected), "{computed}");
            pairs += 1;
        }
    }
    assert_eq!(pairs, 7);
    assert!(
        String::from_utf8(evaluated.stderr)
            .unwrap()
            .ends_with("total: 5232 of 1048576 steps\n")
    );
    let tested = successful(&source, &["test", "--stats"]);
    assert!(
        String::from_utf8(tested.stdout)
            .unwrap()
            .ends_with("6 tests: 6 passed, 0 failed\n")
    );
    assert!(
        String::from_utf8(tested.stderr)
            .unwrap()
            .ends_with("total: 4445 of 1048576 steps\n")
    );
}

#[test]
fn field25519_bounds_and_normalization_match_binary_reference() {
    let cases = cases();
    let mut extra = String::new();
    for (index, limbs) in cases.iter().enumerate() {
        let x = literal(limbs);
        extra.push_str(&format!(
            "spec representation_{index}() -> (Int, Field, Bool, Bool, Bool) {{\n\
             let x: Limbs = {x}; (reconstruct(x), alpha(x), tight(x), loose(x), canonical(x))\n}}\n"
        ));
        if limbs.iter().all(|limb| *limb < 2 * RADIX) {
            extra.push_str(&format!(
                "spec carried_{index}() -> Limbs {{ carry_loose({x}) }}\n\
                 spec normalized_{index}() -> Limbs {{ canonical_output({x}) }}\n"
            ));
        }
        if limbs.iter().all(|limb| *limb < RADIX) {
            extra.push_str(&format!(
                "spec canonicalized_{index}() -> Limbs {{ canonicalize_tight({x}) }}\n"
            ));
        }
    }
    let program = Program::new(&extra);
    let checked = successful(&program.0, &["check"]);
    assert!(checked.stdout.is_empty() && checked.stderr.is_empty());
    let evaluated = successful(&program.0, &["eval", "--stats"]);
    let values = values(&evaluated);
    for (index, limbs) in cases.iter().enumerate() {
        let integer = reconstruct(limbs);
        let reduced = residue(&integer);
        let tight = limbs.iter().all(|limb| *limb < RADIX);
        let loose = limbs.iter().all(|limb| *limb < 2 * RADIX);
        let below_prime = integer[4] == 0 && !at_least_prime(&integer[..4].try_into().unwrap());
        let expected = format!(
            "(Int, Mod[(1 << 255) - 19], Bool, Bool, Bool) = ({}, {}, {tight}, {loose}, {})",
            decimal(integer),
            residue_decimal(reduced),
            tight && below_prime
        );
        assert_eq!(
            values[&format!("representation_{index}")],
            expected,
            "input {limbs:?}"
        );
        let canonical = word_value(&canonical_digits(&reduced));
        if loose {
            let carried = parse_word_value(&values[&format!("carried_{index}")]);
            assert!(carried.iter().all(|limb| *limb < RADIX), "input {limbs:?}");
            assert_eq!(residue(&reconstruct(&carried)), reduced, "input {limbs:?}");
            assert_eq!(
                values[&format!("normalized_{index}")],
                canonical,
                "input {limbs:?}"
            );
        }
        if tight {
            assert_eq!(
                values[&format!("canonicalized_{index}")],
                canonical,
                "input {limbs:?}"
            );
        }
    }
}

#[test]
fn field25519_tight_addition_preserves_exact_limb_sums() {
    let tight: Vec<[u64; 5]> = cases()
        .into_iter()
        .filter(|limbs| limbs.iter().all(|limb| *limb < RADIX))
        .collect();
    let mut extra = String::new();
    let pairs: Vec<([u64; 5], [u64; 5])> = tight
        .iter()
        .copied()
        .zip(tight.iter().rev().copied())
        .collect();
    for (index, (x, y)) in pairs.iter().enumerate() {
        extra.push_str(&format!(
            "spec sum_{index}() -> Limbs {{ add_tight({}, {}) }}\n\
             spec normalized_sum_{index}() -> Limbs {{ canonical_output(sum_{index}()) }}\n",
            literal(x),
            literal(y)
        ));
    }
    let program = Program::new(&extra);
    let values = values(&successful(&program.0, &["eval", "--stats"]));
    for (index, (x, y)) in pairs.iter().enumerate() {
        let sum = std::array::from_fn(|axis| x[axis] + y[axis]);
        assert!(sum.iter().all(|limb| *limb < 2 * RADIX));
        assert_eq!(values[&format!("sum_{index}")], word_value(&sum));
        assert_eq!(
            values[&format!("normalized_sum_{index}")],
            word_value(&canonical_digits(&residue(&reconstruct(&sum))))
        );
    }
}

#[test]
fn field25519_exact_carry_budget_stops_before_partial_output() {
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../algorithms/x25519/field25519-limbs.or");
    let output = successful(
        &source,
        &["eval", "--spec", "second_fold", "--steps", "380", "--stats"],
    );
    assert_eq!(
        String::from_utf8(output.stderr).unwrap(),
        "field25519::second_fold: 380 steps\ntotal: 380 of 380 steps\n"
    );
    let first = run(
        &source,
        &["eval", "--spec", "second_fold", "--steps", "379"],
    );
    let second = run(
        &source,
        &["eval", "--spec", "second_fold", "--steps", "379"],
    );
    assert!(!first.status.success());
    assert!(first.stdout.is_empty());
    assert_eq!(first.stdout, second.stdout);
    assert_eq!(first.stderr, second.stderr);
    assert!(
        String::from_utf8(first.stderr)
            .unwrap()
            .contains("error[ORC0301]")
    );
}
