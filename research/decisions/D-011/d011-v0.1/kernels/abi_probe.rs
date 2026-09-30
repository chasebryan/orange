//! D-011 v0.1 Rust ABI probe: a hosted Rust caller of the stand-in C
//! kernels through `extern "C"`, standing in for the generated Rust wrapper
//! of the D-011 recommendation. It speaks the laboratory driver's request
//! protocol on standard input and output for the portable operations, so
//! the same known-answer subjects check the C ABI from a second language.
//! Contributor-written; not Orange output and not a product wrapper.

use std::io::{self, Read, Write};

unsafe extern "C" {
    fn d011_sha256(out: *mut u8, msg: *const u8, len: usize);
    fn d011_sha256_probe(out: *mut u8, block: *const u8);
    fn d011_sha512(out: *mut u8, msg: *const u8, len: usize);
    fn d011_hmac_sha256(out: *mut u8, key: *const u8, key_len: usize, msg: *const u8, msg_len: usize);
    fn d011_hkdf_sha256(
        out: *mut u8,
        out_len: usize,
        ikm: *const u8,
        ikm_len: usize,
        salt: *const u8,
        salt_len: usize,
        info: *const u8,
        info_len: usize,
    ) -> i32;
    fn d011_chacha20_block(out: *mut u8, key: *const u8, counter: u32, nonce: *const u8);
    fn d011_chacha20_xor(out: *mut u8, input: *const u8, len: usize, key: *const u8, counter: u32, nonce: *const u8);
    fn d011_poly1305(tag: *mut u8, msg: *const u8, len: usize, key: *const u8);
    fn d011_aead_seal(out: *mut u8, key: *const u8, nonce: *const u8, aad: *const u8, aad_len: usize, pt: *const u8, pt_len: usize);
    fn d011_aead_open(out: *mut u8, key: *const u8, nonce: *const u8, aad: *const u8, aad_len: usize, ct: *const u8, ct_len: usize) -> i32;
    fn d011_x25519(out: *mut u8, scalar: *const u8, u: *const u8);
    fn d011_aes_encrypt_block(out: *mut u8, key: *const u8, key_len: usize, input: *const u8) -> i32;
    fn d011_aes_gcm_seal(
        out: *mut u8,
        key: *const u8,
        key_len: usize,
        iv: *const u8,
        aad: *const u8,
        aad_len: usize,
        pt: *const u8,
        pt_len: usize,
    ) -> i32;
    fn d011_aes_gcm_open(
        out: *mut u8,
        key: *const u8,
        key_len: usize,
        iv: *const u8,
        aad: *const u8,
        aad_len: usize,
        ct: *const u8,
        ct_len: usize,
    ) -> i32;
    fn d011_sha3_256(out: *mut u8, msg: *const u8, len: usize);
    fn d011_shake128(out: *mut u8, out_len: usize, msg: *const u8, len: usize);
}

const OK: u8 = 0;
const REJECTED: u8 = 1;
const UNSUPPORTED: u8 = 2;
const MALFORMED: u8 = 3;
const OUT_MAX: usize = 16400;

fn le32(bytes: &[u8]) -> u32 {
    u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
}

fn lens(args: &[Vec<u8>], expected: &[Option<usize>]) -> bool {
    args.len() == expected.len() && args.iter().zip(expected).all(|(arg, want)| want.is_none_or(|n| arg.len() == n))
}

fn aes_key(arg: &[u8]) -> bool {
    arg.len() == 16 || arg.len() == 32
}

fn run(op: u8, a: &[Vec<u8>]) -> (u8, Vec<u8>) {
    let mut out = vec![0u8; OUT_MAX];
    let n = match op {
        1 if lens(a, &[None]) => {
            unsafe { d011_sha256(out.as_mut_ptr(), a[0].as_ptr(), a[0].len()) };
            32
        }
        2 if lens(a, &[Some(64)]) => {
            unsafe { d011_sha256_probe(out.as_mut_ptr(), a[0].as_ptr()) };
            72
        }
        3 if lens(a, &[None]) => {
            unsafe { d011_sha512(out.as_mut_ptr(), a[0].as_ptr(), a[0].len()) };
            64
        }
        4 if lens(a, &[None, None]) => {
            unsafe { d011_hmac_sha256(out.as_mut_ptr(), a[0].as_ptr(), a[0].len(), a[1].as_ptr(), a[1].len()) };
            32
        }
        5 if lens(a, &[None, None, None, Some(4)]) && le32(&a[3]) <= 255 * 32 => {
            let len = le32(&a[3]) as usize;
            let status = unsafe {
                d011_hkdf_sha256(out.as_mut_ptr(), len, a[0].as_ptr(), a[0].len(), a[1].as_ptr(), a[1].len(), a[2].as_ptr(), a[2].len())
            };
            if status != 0 {
                return (MALFORMED, Vec::new());
            }
            len
        }
        6 if lens(a, &[Some(32), Some(4), Some(12)]) => {
            unsafe { d011_chacha20_block(out.as_mut_ptr(), a[0].as_ptr(), le32(&a[1]), a[2].as_ptr()) };
            64
        }
        7 if lens(a, &[Some(32), Some(4), Some(12), None]) && a[3].len() <= OUT_MAX => {
            unsafe { d011_chacha20_xor(out.as_mut_ptr(), a[3].as_ptr(), a[3].len(), a[0].as_ptr(), le32(&a[1]), a[2].as_ptr()) };
            a[3].len()
        }
        8 if lens(a, &[Some(32), None]) => {
            unsafe { d011_poly1305(out.as_mut_ptr(), a[1].as_ptr(), a[1].len(), a[0].as_ptr()) };
            16
        }
        9 if lens(a, &[Some(32), Some(12), None, None]) && a[3].len() + 16 <= OUT_MAX => {
            unsafe { d011_aead_seal(out.as_mut_ptr(), a[0].as_ptr(), a[1].as_ptr(), a[2].as_ptr(), a[2].len(), a[3].as_ptr(), a[3].len()) };
            a[3].len() + 16
        }
        10 if lens(a, &[Some(32), Some(12), None, None]) && a[3].len() >= 16 && a[3].len() <= OUT_MAX => {
            let status = unsafe {
                d011_aead_open(out.as_mut_ptr(), a[0].as_ptr(), a[1].as_ptr(), a[2].as_ptr(), a[2].len(), a[3].as_ptr(), a[3].len())
            };
            if status != 0 {
                return (REJECTED, Vec::new());
            }
            a[3].len() - 16
        }
        11 if lens(a, &[Some(32), Some(32)]) => {
            unsafe { d011_x25519(out.as_mut_ptr(), a[0].as_ptr(), a[1].as_ptr()) };
            32
        }
        12 if a.len() == 2 && aes_key(&a[0]) && a[1].len() == 16 => {
            unsafe { d011_aes_encrypt_block(out.as_mut_ptr(), a[0].as_ptr(), a[0].len(), a[1].as_ptr()) };
            16
        }
        13 if a.len() == 4 && aes_key(&a[0]) && a[1].len() == 12 && a[3].len() + 16 <= OUT_MAX => {
            unsafe {
                d011_aes_gcm_seal(out.as_mut_ptr(), a[0].as_ptr(), a[0].len(), a[1].as_ptr(), a[2].as_ptr(), a[2].len(), a[3].as_ptr(), a[3].len())
            };
            a[3].len() + 16
        }
        14 if a.len() == 4 && aes_key(&a[0]) && a[1].len() == 12 && a[3].len() >= 16 && a[3].len() <= OUT_MAX => {
            let status = unsafe {
                d011_aes_gcm_open(out.as_mut_ptr(), a[0].as_ptr(), a[0].len(), a[1].as_ptr(), a[2].as_ptr(), a[2].len(), a[3].as_ptr(), a[3].len())
            };
            if status != 0 {
                return (REJECTED, Vec::new());
            }
            a[3].len() - 16
        }
        15 if lens(a, &[None]) => {
            unsafe { d011_sha3_256(out.as_mut_ptr(), a[0].as_ptr(), a[0].len()) };
            32
        }
        16 if lens(a, &[None, Some(4)]) && (le32(&a[1]) as usize) <= OUT_MAX => {
            let len = le32(&a[1]) as usize;
            unsafe { d011_shake128(out.as_mut_ptr(), len, a[0].as_ptr(), a[0].len()) };
            len
        }
        1..=16 => return (MALFORMED, Vec::new()),
        _ => return (UNSUPPORTED, Vec::new()),
    };
    out.truncate(n);
    (OK, out)
}

fn main() {
    let mut input = Vec::new();
    if io::stdin().read_to_end(&mut input).is_err() {
        std::process::exit(2);
    }
    let mut output = Vec::new();
    let mut at = 0usize;
    while at < input.len() {
        if input.len() - at < 2 {
            std::process::exit(2);
        }
        let (op, count) = (input[at], input[at + 1] as usize);
        at += 2;
        let mut args = Vec::with_capacity(count);
        for _ in 0..count {
            if input.len() - at < 4 {
                std::process::exit(2);
            }
            let len = le32(&input[at..at + 4]) as usize;
            at += 4;
            if input.len() - at < len {
                std::process::exit(2);
            }
            args.push(input[at..at + len].to_vec());
            at += len;
        }
        let (status, bytes) = run(op, &args);
        output.push(status);
        output.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
        output.extend_from_slice(&bytes);
    }
    if io::stdout().write_all(&output).is_err() {
        std::process::exit(3);
    }
}
