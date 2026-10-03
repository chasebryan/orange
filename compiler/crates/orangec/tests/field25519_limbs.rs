//! Executable evidence for the OEP-0022 P2 mathematical representations.
//!
//! Independent binary multiplication and long division check reconstruction,
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

fn decimal<const N: usize>(mut words: [u64; N]) -> String {
    let mut digits = Vec::new();
    while words != [0; N] {
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
fn residue<const N: usize>(integer: &[u64; N]) -> [u64; 4] {
    let mut result = [0u64; 4];
    for position in (0..(64 * N)).rev() {
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

fn add_power<const N: usize>(words: &mut [u64; N], position: usize) {
    let mut index = position / 64;
    let mut carry = 1 << (position % 64);
    loop {
        let (sum, overflow) = words[index].overflowing_add(carry);
        words[index] = sum;
        if !overflow {
            return;
        }
        index += 1;
        assert!(index < N, "binary reference capacity exceeded");
        carry = 1;
    }
}

// Multiply reconstructed storage using individual binary-bit products. There
// is no radix-2^51 convolution, modular 19-fold, or product carry schedule here.
fn binary_product(x: &[u64; 5], y: &[u64; 5]) -> [u64; 10] {
    let x = reconstruct(x);
    let y = reconstruct(y);
    let mut product = [0; 10];
    for left in 0..320 {
        if (x[left / 64] >> (left % 64)) & 1 == 0 {
            continue;
        }
        for right in 0..320 {
            if (y[right / 64] >> (right % 64)) & 1 != 0 {
                add_power(&mut product, left + right);
            }
        }
    }
    product
}

fn reconstruct_accumulators(h: &[u128; 5]) -> [u64; 6] {
    let mut result = [0; 6];
    for (axis, value) in h.iter().enumerate() {
        for bit in 0..128 {
            if (value >> bit) & 1 != 0 {
                add_power(&mut result, 51 * axis + bit);
            }
        }
    }
    result
}

// Extract digit windows and the top quotient from a complete binary integer,
// independently of the per-limb carry propagation used in the Orange source.
fn split_binary<const N: usize>(words: &[u64; N]) -> ([u64; 5], u128) {
    assert!(N >= 4);
    let digits = std::array::from_fn(|axis| {
        let mut value = 0;
        for bit in 0..51 {
            let position = 51 * axis + bit;
            value |= ((words[position / 64] >> (position % 64)) & 1) << bit;
        }
        value
    });
    let mut top = 0;
    for offset in 0..128 {
        let position = 255 + offset;
        if position < N * 64 {
            top |= u128::from((words[position / 64] >> (position % 64)) & 1) << offset;
        }
    }
    for position in 383..(N * 64) {
        assert_eq!((words[position / 64] >> (position % 64)) & 1, 0);
    }
    (digits, top)
}

fn add_low<const N: usize>(words: &mut [u64; N], value: u128) {
    for bit in 0..128 {
        if (value >> bit) & 1 != 0 {
            add_power(words, bit);
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
struct ProductTrace {
    digits: [[u64; 5]; 3],
    carries: [u128; 3],
}

fn binary_trace(h: &[u128; 5]) -> ProductTrace {
    let (first, first_carry) = split_binary(&reconstruct_accumulators(h));
    let mut next = reconstruct(&first);
    add_low(&mut next, first_carry.checked_mul(19).unwrap());
    let (second, second_carry) = split_binary(&next);
    next = reconstruct(&second);
    add_low(&mut next, second_carry.checked_mul(19).unwrap());
    let (third, third_carry) = split_binary(&next);
    ProductTrace {
        digits: [first, second, third],
        carries: [first_carry, second_carry, third_carry],
    }
}

fn parse_accumulators(value: &str) -> [u128; 5] {
    value
        .strip_prefix("Int^5 = [")
        .unwrap()
        .strip_suffix(']')
        .unwrap()
        .split(", ")
        .map(|field| field.parse().unwrap())
        .collect::<Vec<_>>()
        .try_into()
        .unwrap()
}

fn parse_product_trace(value: &str) -> ProductTrace {
    let fields: Vec<_> = value
        .strip_prefix("(Word[64]^5, Int, Word[64]^5, Int, Word[64]^5, Int) = (")
        .unwrap()
        .strip_suffix(')')
        .unwrap()
        .split(", ")
        .collect();
    assert_eq!(fields.len(), 18);
    ProductTrace {
        digits: std::array::from_fn(|stage| {
            std::array::from_fn(|axis| {
                u64::from_str_radix(
                    fields[stage * 6 + axis]
                        .trim_start_matches('[')
                        .trim_end_matches(']')
                        .strip_prefix("0x")
                        .unwrap(),
                    16,
                )
                .unwrap()
            })
        }),
        carries: std::array::from_fn(|stage| fields[stage * 6 + 5].parse().unwrap()),
    }
}

// This direct nine-coefficient schoolbook sum independently checks the cyclic
// indexing in product_accumulators. The binary oracle separately checks the
// complete product, reduction and output representation.
fn schoolbook_coefficients(x: &[u64; 5], y: &[u64; 5]) -> [u128; 5] {
    assert!(x.iter().chain(y).all(|value| *value < RADIX));
    let mut convolution = [0u128; 9];
    for (left, x) in x.iter().enumerate() {
        for (right, y) in y.iter().enumerate() {
            convolution[left + right] = convolution[left + right]
                .checked_add(u128::from(*x).checked_mul(u128::from(*y)).unwrap())
                .unwrap();
        }
    }
    std::array::from_fn(|axis| {
        convolution[axis]
            .checked_add(if axis < 4 {
                convolution[axis + 5].checked_mul(19).unwrap()
            } else {
                0
            })
            .unwrap()
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
    assert_eq!(pairs, 12);
    assert!(
        String::from_utf8(evaluated.stderr)
            .unwrap()
            .ends_with("total: 19999 of 1048576 steps\n")
    );
    let tested = successful(&source, &["test", "--stats"]);
    assert!(
        String::from_utf8(tested.stdout)
            .unwrap()
            .ends_with("10 tests: 10 passed, 0 failed\n")
    );
    assert!(
        String::from_utf8(tested.stderr)
            .unwrap()
            .ends_with("total: 15751 of 1048576 steps\n")
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

fn product_pairs() -> Vec<([u64; 5], [u64; 5])> {
    let mut pairs = Vec::new();
    let boundary = [
        [0; 5],
        [RADIX - 20, RADIX - 1, RADIX - 1, RADIX - 1, RADIX - 1],
        [RADIX - 19, RADIX - 1, RADIX - 1, RADIX - 1, RADIX - 1],
        [RADIX - 18, RADIX - 1, RADIX - 1, RADIX - 1, RADIX - 1],
        [RADIX - 1; 5],
    ];
    for x in boundary {
        for y in boundary {
            pairs.push((x, y));
        }
    }
    for magnitude in [1, RADIX - 1] {
        for left in 0..5 {
            for right in 0..5 {
                let mut x = [0; 5];
                let mut y = [0; 5];
                x[left] = magnitude;
                y[right] = magnitude;
                pairs.push((x, y));
            }
        }
    }
    let tight: Vec<_> = cases()
        .into_iter()
        .filter(|limbs| limbs.iter().all(|limb| *limb < RADIX))
        .collect();
    pairs.extend(tight.iter().copied().zip(tight.iter().rev().copied()));
    let third_pass = [125_099_989_649_181, 0, 0, 0, 0];
    pairs.extend([([RADIX - 1; 5], third_pass), (third_pass, [RADIX - 1; 5])]);
    pairs
}

#[test]
fn field25519_products_and_each_carry_stage_match_independent_binary_arithmetic() {
    let pairs = product_pairs();
    assert_eq!(pairs.len(), 129);
    let mut extra = String::new();
    for (index, (x, y)) in pairs.iter().enumerate() {
        extra.push_str(&format!(
            "spec coeff_{index}() -> ProductAccumulators {{ product_accumulators({}, {}) }}\n\
             spec coeff_reconstruction_{index}() -> Int {{ reconstruct_product(coeff_{index}()) }}\n\
             spec trace_{index}() -> (Limbs, Int, Limbs, Int, Limbs, Int) {{ product_carry_trace(coeff_{index}()) }}\n\
             spec tight_product_{index}() -> Limbs {{ multiply_tight({}, {}) }}\n\
             spec canonical_product_{index}() -> Limbs {{ multiply_canonical({}, {}) }}\n",
            literal(x), literal(y), literal(x), literal(y), literal(x), literal(y),
        ));
    }
    let program = Program::new(&extra);
    let values = values(&successful(&program.0, &["eval", "--stats"]));
    let maximum = u128::from(RADIX - 1).pow(2);
    let bounds = [77u128, 59, 41, 23, 5].map(|weight| weight * maximum);
    let mut large_product = false;
    let mut folded_top = false;
    let mut third_pass_needed = false;
    for (index, (x, y)) in pairs.iter().enumerate() {
        let h = parse_accumulators(&values[&format!("coeff_{index}")]);
        assert_eq!(h, schoolbook_coefficients(x, y), "pair {index}");
        assert_eq!(
            values[&format!("coeff_reconstruction_{index}")],
            format!("Int = {}", decimal(reconstruct_accumulators(&h))),
            "pair {index}"
        );
        assert!(
            h.iter()
                .zip(bounds)
                .all(|(value, maximum)| *value <= maximum)
        );
        large_product |= h.iter().any(|value| *value > u128::from(u64::MAX));

        let trace = parse_product_trace(&values[&format!("trace_{index}")]);
        assert_eq!(trace, binary_trace(&h), "pair {index}");
        assert!(trace.digits.iter().flatten().all(|digit| *digit < RADIX));
        assert!(trace.carries[0] <= 5 * u128::from(RADIX) + 12);
        assert!(trace.carries[1] <= 1);
        assert_eq!(trace.carries[2], 0);
        folded_top |= trace.carries[1] == 1;
        third_pass_needed |=
            u128::from(trace.digits[1][0]) + 19 * trace.carries[1] >= u128::from(RADIX);

        let reduced = residue(&binary_product(x, y));
        assert_eq!(
            residue(&reconstruct_accumulators(&h)),
            reduced,
            "pair {index}"
        );
        let tight = parse_word_value(&values[&format!("tight_product_{index}")]);
        assert_eq!(tight, trace.digits[2]);
        assert_eq!(residue(&reconstruct(&tight)), reduced, "pair {index}");
        let canonical = parse_word_value(&values[&format!("canonical_product_{index}")]);
        assert_eq!(canonical, canonical_digits(&reduced), "pair {index}");
    }
    assert!(
        large_product,
        "examples must exceed a wrapping Word[64] product"
    );
    assert!(
        folded_top,
        "examples must exercise the second top-carry fold"
    );
    assert!(
        third_pass_needed,
        "examples must detect the algebraically correct nontight shortcut"
    );
}

#[test]
fn field25519_accumulator_and_input_contract_boundaries_are_explicit() {
    let maximum = u128::from(RADIX - 1).pow(2);
    let bounds = [77u128, 59, 41, 23, 5].map(|weight| weight * maximum);
    let mut extra = String::new();
    let mut expected = Vec::new();
    for (axis, bound) in bounds.iter().enumerate() {
        for (delta, value) in [bound - 1, *bound, bound + 1].into_iter().enumerate() {
            let mut h = [0; 5];
            h[axis] = value;
            let literal = format!("[{}]", h.map(|value| value.to_string()).join(", "));
            let name = format!("accumulator_bound_{axis}_{delta}");
            extra.push_str(&format!(
                "spec {name}() -> (Bool, Bool) {{\n\
                 let h: ProductAccumulators = {literal};\n\
                 (bounded_product(h), (alpha(carry_product(h)) == (reconstruct_product(h) as Field)))\n}}\n"
            ));
            expected.push((name, format!("(Bool, Bool) = ({}, true)", delta != 2)));
        }
        for (case, bound) in [RADIX - 1, RADIX, RADIX + 1, 2 * RADIX, u64::MAX]
            .into_iter()
            .enumerate()
        {
            let mut x = [0; 5];
            x[axis] = bound;
            let name = format!("input_bound_{axis}_{case}");
            extra.push_str(&format!(
                "spec {name}() -> (Bool, Bool) {{\n\
                 let x: Limbs = {}; let y: Limbs = [1, 0, 0, 0, 0];\n\
                 (tight(x) && tight(y), (alpha(multiply_tight(x, y)) == (alpha(x) * alpha(y))))\n}}\n",
                literal(&x),
            ));
            expected.push((name, format!("(Bool, Bool) = ({}, true)", bound < RADIX)));
        }
    }
    extra.push_str("spec negative_accumulator() -> Bool { bounded_product([-1, 0, 0, 0, 0]) }\n");
    expected.push((
        String::from("negative_accumulator"),
        String::from("Bool = false"),
    ));
    let program = Program::new(&extra);
    let values = values(&successful(&program.0, &["eval", "--stats"]));
    for (name, expected) in expected {
        assert_eq!(values[&name], expected, "{name}");
    }
}

#[test]
fn field25519_third_product_pass_has_exact_fail_closed_step_boundary() {
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../algorithms/x25519/field25519-limbs.or");
    let output = successful(
        &source,
        &[
            "eval",
            "--spec",
            "product_third_pass",
            "--steps",
            "1474",
            "--stats",
        ],
    );
    assert_eq!(
        String::from_utf8(output.stderr).unwrap(),
        "field25519::product_third_pass: 1474 steps\ntotal: 1474 of 1474 steps\n"
    );
    let short = run(
        &source,
        &["eval", "--spec", "product_third_pass", "--steps", "1473"],
    );
    assert!(!short.status.success());
    assert!(short.stdout.is_empty());
    assert!(
        String::from_utf8(short.stderr)
            .unwrap()
            .contains("error[ORC0301]")
    );
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
