//! Black-box tests for sealing files: `orangec keygen`, `enc`, `dec`, and
//! `schemes`.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const SCHEMES: [(&str, usize, usize); 3] = [
    ("xchacha20_poly1305", 32, 24),
    ("chacha20_poly1305", 32, 12),
    ("ascon_aead128", 16, 16),
];

/// A scratch directory with its own home, so the default key path is private
/// to the test.
struct Scratch {
    root: PathBuf,
}

impl Scratch {
    fn new(name: &str) -> Self {
        let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
            .join(format!("crypt-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("home")).unwrap();
        Self { root }
    }

    fn path(&self, name: &str) -> PathBuf {
        self.root.join(name)
    }

    fn write(&self, name: &str, bytes: &[u8]) -> PathBuf {
        let path = self.path(name);
        fs::write(&path, bytes).unwrap();
        path
    }

    fn orangec(&self, arguments: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_orangec"))
            .args(arguments)
            .current_dir(&self.root)
            .env("HOME", self.root.join("home"))
            .env_remove("XDG_CONFIG_HOME")
            .output()
            .unwrap()
    }

    fn succeeds(&self, arguments: &[&str]) -> String {
        let output = self.orangec(arguments);
        assert_eq!(
            output.status.code(),
            Some(0),
            "{arguments:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(output.stderr, b"", "{arguments:?}");
        String::from_utf8(output.stdout).unwrap()
    }

    /// Runs a command that must fail with `status` and returns its standard
    /// error.
    fn fails(&self, status: i32, arguments: &[&str]) -> String {
        let output = self.orangec(arguments);
        assert_eq!(
            output.status.code(),
            Some(status),
            "{arguments:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(output.stdout, b"", "{arguments:?}");
        String::from_utf8(output.stderr).unwrap()
    }

    fn keygen(&self, scheme: &str, name: &str) -> PathBuf {
        let output = self.succeeds(&["keygen", "--scheme", scheme, "-o", name]);
        assert!(output.starts_with("wrote a "), "{output}");
        self.path(name)
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn counting(length: usize) -> Vec<u8> {
    (0..length)
        .map(|i| u8::try_from(i % 256).unwrap())
        .collect()
}

fn sealed_length(plaintext: usize) -> usize {
    64 + 256 * (plaintext / 240 + 1)
}

fn assert_absent(path: &Path) {
    assert!(
        path.symlink_metadata().is_err(),
        "{} exists",
        path.display()
    );
    let mut partial = path.as_os_str().to_owned();
    partial.push(".partial");
    assert!(
        Path::new(&partial).symlink_metadata().is_err(),
        "{} exists",
        Path::new(&partial).display()
    );
}

#[test]
fn every_builtin_scheme_seals_and_opens_files_of_every_shape() {
    let scratch = Scratch::new("round-trip");
    for (scheme, _, nonce) in SCHEMES {
        let key = scratch.keygen(scheme, &format!("{scheme}.key"));
        let key = key.to_str().unwrap();
        for length in [0, 1, 239, 240, 241, 480, 1000] {
            let plaintext = counting(length);
            let name = format!("{scheme}-{length}");
            scratch.write(&name, &plaintext);
            let sealed_name = format!("{name}.orange");
            scratch.succeeds(&["enc", "--key", key, &name]);
            let sealed = fs::read(scratch.path(&sealed_name)).unwrap();
            assert_eq!(sealed.len(), sealed_length(length), "{name}");
            assert_eq!(&sealed[..8], b"orange\x00\x01");
            assert_eq!(&sealed[16..16 + scheme.len()], scheme.as_bytes());
            assert!(sealed[40 + nonce - 5..64].iter().all(|byte| *byte == 0));

            let opened = format!("{name}.opened");
            scratch.succeeds(&["dec", "--key", key, "-o", &opened, &sealed_name]);
            assert_eq!(
                fs::read(scratch.path(&opened)).unwrap(),
                plaintext,
                "{name}"
            );
        }
    }
}

#[test]
fn keygen_writes_a_private_key_once() {
    let scratch = Scratch::new("keygen");
    let default = scratch.path("home/.config/orange/key");
    let message = scratch.succeeds(&["keygen"]);
    assert_eq!(
        message,
        format!(
            "wrote a 256-bit xchacha20_poly1305 key to `{}`\n",
            default.display()
        )
    );
    let text = fs::read_to_string(&default).unwrap();
    let line = text.lines().find(|line| !line.starts_with('#')).unwrap();
    let fields = line.split(' ').collect::<Vec<_>>();
    assert_eq!(fields[..3], ["orange-key", "1", "xchacha20_poly1305"]);
    assert_eq!(fields[3].len(), 64);
    assert!(
        fields[3]
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    );
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        let mode = |path: &Path| fs::metadata(path).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode(&default), 0o600);
        assert_eq!(mode(default.parent().unwrap()), 0o700);
    }

    let refused = scratch.fails(1, &["keygen"]);
    assert!(refused.starts_with("error[ORC1009]: a file already exists at `"));
    assert_eq!(fs::read_to_string(&default).unwrap(), text);

    let ascon = scratch.keygen("ascon_aead128", "ascon.key");
    let text = fs::read_to_string(ascon).unwrap();
    assert!(text.contains("\norange-key 1 ascon_aead128 "));
    assert_eq!(text.trim_end().rsplit(' ').next().unwrap().len(), 32);

    let xdg = scratch.path("config");
    let output = Command::new(env!("CARGO_BIN_EXE_orangec"))
        .arg("keygen")
        .env("XDG_CONFIG_HOME", &xdg)
        .env_remove("HOME")
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0));
    assert!(xdg.join("orange/key").is_file());

    let unknown = scratch.fails(1, &["keygen", "--scheme", "rot13"]);
    assert!(unknown.starts_with("error[ORC1010]: unknown scheme `rot13`\n"));
}

#[test]
fn two_seals_of_one_file_differ_and_both_open() {
    let scratch = Scratch::new("fresh");
    scratch.succeeds(&["keygen"]);
    scratch.write("note", b"the same plaintext");
    scratch.succeeds(&["enc", "-o", "one", "note"]);
    scratch.succeeds(&["enc", "-o", "two", "note"]);
    let one = fs::read(scratch.path("one")).unwrap();
    let two = fs::read(scratch.path("two")).unwrap();
    assert_ne!(one[40..64], two[40..64]);
    assert_ne!(one[64..], two[64..]);
    for sealed in ["one", "two"] {
        let opened = format!("{sealed}.opened");
        scratch.succeeds(&["dec", "-o", &opened, sealed]);
        assert_eq!(
            fs::read(scratch.path(&opened)).unwrap(),
            b"the same plaintext"
        );
    }
    scratch.succeeds(&["enc", "note"]);
    fs::remove_file(scratch.path("note")).unwrap();
    scratch.succeeds(&["dec", "note.orange"]);
    assert_eq!(
        fs::read(scratch.path("note")).unwrap(),
        b"the same plaintext"
    );
}

#[test]
fn altered_sealed_files_open_to_nothing() {
    let scratch = Scratch::new("tamper");
    scratch.succeeds(&["keygen"]);
    scratch.write("data", &counting(600));
    scratch.succeeds(&["enc", "data"]);
    let sealed = fs::read(scratch.path("data.orange")).unwrap();
    assert_eq!(sealed.len(), 64 + 3 * 256);
    let chunk = |index: usize| sealed[64 + 256 * index..64 + 256 * (index + 1)].to_vec();
    let flipped = |at: usize| {
        let mut altered = sealed.clone();
        altered[at] ^= 0x01;
        altered
    };
    let joined = |parts: &[&[u8]]| parts.concat();
    let cases: Vec<(&str, Vec<u8>, &str)> = vec![
        ("magic", flipped(0), "error[ORC1013]"),
        ("prefix", flipped(40), "error[ORC1014]: chunk 0 of"),
        ("first", flipped(64 + 3), "error[ORC1014]: chunk 0 of"),
        ("tag", flipped(64 + 256 + 250), "error[ORC1014]: chunk 1 of"),
        (
            "final",
            flipped(sealed.len() - 1),
            "error[ORC1014]: chunk 2 of",
        ),
        (
            "dropped",
            sealed[..sealed.len() - 256].to_vec(),
            "error[ORC1014]: chunk 1 of",
        ),
        (
            "cut",
            sealed[..sealed.len() - 7].to_vec(),
            "it ends inside chunk 2",
        ),
        (
            "swapped",
            joined(&[&sealed[..64], &chunk(1), &chunk(0), &chunk(2)]),
            "error[ORC1014]: chunk 0 of",
        ),
        (
            "appended",
            joined(&[&sealed, &chunk(2)]),
            "error[ORC1014]: chunk 2 of",
        ),
        ("header", sealed[..64].to_vec(), "error[ORC1013]"),
        (
            "short",
            sealed[..30].to_vec(),
            "shorter than the 64-byte header",
        ),
    ];
    for (name, bytes, expected) in cases {
        let sealed_name = format!("{name}.orange");
        scratch.write(&sealed_name, &bytes);
        let error = scratch.fails(1, &["dec", &sealed_name]);
        assert!(error.contains(expected), "{name}: {error}");
        assert_absent(&scratch.path(name));
    }

    scratch.keygen("xchacha20_poly1305", "other.key");
    let error = scratch.fails(
        1,
        &["dec", "--key", "other.key", "-o", "other", "data.orange"],
    );
    assert!(error.starts_with("error[ORC1014]: chunk 0 of `data.orange` is not authentic\n"));
    assert_absent(&scratch.path("other"));

    scratch.keygen("ascon_aead128", "ascon.key");
    let error = scratch.fails(
        1,
        &["dec", "--key", "ascon.key", "-o", "ascon", "data.orange"],
    );
    assert!(error.starts_with(
        "error[ORC1009]: `data.orange` was sealed with the scheme `xchacha20_poly1305`, but the key is for `ascon_aead128`\n"
    ));
    assert_absent(&scratch.path("ascon"));
}

#[test]
fn outputs_and_keys_are_never_replaced_or_exposed() {
    let scratch = Scratch::new("files");
    scratch.succeeds(&["keygen"]);
    scratch.write("data", b"data");
    scratch.write("data.orange", b"already here");
    let error = scratch.fails(1, &["enc", "data"]);
    assert!(error.starts_with("error[ORC1012]: `data.orange` already exists\n"));
    assert_eq!(
        fs::read(scratch.path("data.orange")).unwrap(),
        b"already here"
    );

    scratch.write("busy.partial", b"someone else's");
    let error = scratch.fails(1, &["enc", "-o", "busy", "data"]);
    assert!(error.starts_with("error[ORC1012]: could not create `busy.partial`\n"));
    assert_eq!(
        fs::read(scratch.path("busy.partial")).unwrap(),
        b"someone else's"
    );

    scratch.succeeds(&["enc", "-o", "sealed", "data"]);
    let error = scratch.fails(1, &["dec", "sealed"]);
    assert!(error.contains("`sealed` does not end in `.orange`"));

    let error = scratch.fails(1, &["enc", "missing"]);
    assert!(error.starts_with("error[ORC1011]: could not read `missing`\n"));
    let error = scratch.fails(1, &["enc", "home"]);
    assert!(error.contains("path does not name a regular file"));

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        std::os::unix::fs::symlink("data", scratch.path("link")).unwrap();
        let error = scratch.fails(1, &["enc", "link"]);
        assert!(error.contains("symbolic link") || error.contains("could not read `link`"));

        let key = scratch.path("home/.config/orange/key");
        fs::set_permissions(&key, fs::Permissions::from_mode(0o644)).unwrap();
        let error = scratch.fails(1, &["enc", "-o", "exposed", "data"]);
        assert!(error.contains("can be read or written by other users"));
        assert_absent(&scratch.path("exposed"));
    }

    let error = scratch.fails(1, &["enc", "--key", "absent.key", "data"]);
    assert!(error.starts_with("error[ORC1009]: there is no key at `absent.key`\n"));
    scratch.write("bad.key", b"orange-key 1 xchacha20_poly1305 ABCD\n");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        fs::set_permissions(scratch.path("bad.key"), fs::Permissions::from_mode(0o600)).unwrap();
    }
    let error = scratch.fails(1, &["enc", "--key", "bad.key", "data"]);
    assert!(error.contains("its key is not lowercase hexadecimal"));
}

#[test]
fn any_orange_program_with_the_interface_is_a_scheme() {
    let scratch = Scratch::new("custom");
    let builtin =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../schemes/chacha20_poly1305.or");
    let text = fs::read_to_string(builtin)
        .unwrap()
        .replace("module chacha20_poly1305 {", "module my_cipher {");
    scratch.write("my_cipher.or", text.as_bytes());

    let table = scratch.succeeds(&["schemes", "my_cipher.or"]);
    assert!(
        table
            .lines()
            .nth(1)
            .unwrap()
            .starts_with("my_cipher  256     96  128    240  ")
    );
    assert!(table.lines().nth(1).unwrap().ends_with("  my_cipher.or"));

    scratch.succeeds(&["keygen", "--scheme", "./my_cipher.or", "-o", "my.key"]);
    scratch.write("data", &counting(500));
    scratch.succeeds(&["enc", "--key", "my.key", "--scheme", "my_cipher.or", "data"]);
    let sealed = fs::read(scratch.path("data.orange")).unwrap();
    assert_eq!(&sealed[16..25], b"my_cipher");

    let error = scratch.fails(1, &["dec", "--key", "my.key", "-o", "out", "data.orange"]);
    assert!(error.starts_with(
        "error[ORC1010]: the key is for the scheme `my_cipher`, which is not built in\n"
    ));
    scratch.succeeds(&[
        "dec",
        "--key",
        "my.key",
        "--scheme",
        "my_cipher.or",
        "-o",
        "out",
        "data.orange",
    ]);
    assert_eq!(fs::read(scratch.path("out")).unwrap(), counting(500));

    let error = scratch.fails(
        1,
        &[
            "enc",
            "--key",
            "my.key",
            "--scheme",
            "chacha20_poly1305",
            "data",
        ],
    );
    assert!(error.contains("the key is for the scheme `my_cipher`, not `chacha20_poly1305`"));

    let half = concat!(
        "edition 2026;\n",
        "module half {\n",
        "  spec seal(key: Word[8]^16, nonce: Word[8]^16, ad: Word[8]^64, plaintext: Word[8]^16) -> Word[8]^32 { [0; 32] }\n",
        "  spec open(key: Word[8]^16, nonce: Word[8]^16, ad: Word[8]^64, sealed: Word[8]^32) -> Word[8]^16 { [0; 16] }\n",
        "}\n",
    );
    scratch.write("half.or", half.as_bytes());
    let error = scratch.fails(1, &["schemes", "half.or"]);
    assert_eq!(
        error,
        "error[ORC1010]: `half` at half.or does not implement the sealing interface\n  = note: it has no spec named `authentic`\n"
    );
    scratch.write(
        "narrow.or",
        half.replace("ad: Word[8]^64, plaintext", "ad: Word[8]^32, plaintext")
            .as_bytes(),
    );
    let error = scratch.fails(1, &["schemes", "narrow.or"]);
    assert!(error.contains(
        "expected `spec seal(key: Word[8]^K, nonce: Word[8]^N, ad: Word[8]^64, plaintext: Word[8]^C) -> Word[8]^S`"
    ));
    // A `seal` with size parameters is a family of functions, not the one
    // the interface names.
    scratch.write(
        "sized.or",
        half.replace("module half {", "module sized {")
            .replace("spec seal(", "spec seal[n in 1..3](")
            .as_bytes(),
    );
    let error = scratch.fails(1, &["schemes", "sized.or"]);
    assert_eq!(
        error,
        "error[ORC1010]: `sized` at sized.or does not implement the sealing interface\n  = note: `seal` must declare no size parameters\n"
    );
}

#[test]
fn a_scheme_program_may_use_modules_beside_it() {
    let scratch = Scratch::new("modules");
    let builtin =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../schemes/chacha20_poly1305.or");
    let text = fs::read_to_string(builtin)
        .unwrap()
        .replace("module chacha20_poly1305 {", "module aead {");
    scratch.write("aead.or", text.as_bytes());
    let layered = concat!(
        "edition 2026;\n",
        "module layered {\n",
        "  use aead;\n",
        "  spec seal(key: Word[8]^32, nonce: Word[8]^12, ad: Word[8]^64, plaintext: Word[8]^240) -> Word[8]^256 {\n",
        "    aead::seal(key, nonce, ad, plaintext)\n",
        "  }\n",
        "  spec open(key: Word[8]^32, nonce: Word[8]^12, ad: Word[8]^64, sealed: Word[8]^256) -> Word[8]^240 {\n",
        "    aead::open(key, nonce, ad, sealed)\n",
        "  }\n",
        "  spec authentic(key: Word[8]^32, nonce: Word[8]^12, ad: Word[8]^64, sealed: Word[8]^256) -> Bool {\n",
        "    aead::authentic(key, nonce, ad, sealed)\n",
        "  }\n",
        "}\n",
    );
    scratch.write("layered.or", layered.as_bytes());

    let table = scratch.succeeds(&["schemes", "layered.or"]);
    assert!(table.lines().nth(1).unwrap().starts_with("layered "));
    scratch.succeeds(&["keygen", "--scheme", "./layered.or", "-o", "layered.key"]);
    scratch.write("data", &counting(500));
    scratch.succeeds(&[
        "enc",
        "--key",
        "layered.key",
        "--scheme",
        "layered.or",
        "data",
    ]);
    scratch.succeeds(&[
        "dec",
        "--key",
        "layered.key",
        "--scheme",
        "layered.or",
        "-o",
        "out",
        "data.orange",
    ]);
    assert_eq!(fs::read(scratch.path("out")).unwrap(), counting(500));

    // The interface is the root's own: a used module's `authentic` does not
    // stand in for a missing one.
    let hollow = layered
        .replace("module layered {", "module hollow {")
        .replace("spec authentic(", "spec checked(");
    scratch.write("hollow.or", hollow.as_bytes());
    let error = scratch.fails(1, &["schemes", "hollow.or"]);
    assert_eq!(
        error,
        "error[ORC1010]: `hollow` at hollow.or does not implement the sealing interface\n  = note: it has no spec named `authentic`\n"
    );

    let lonely = layered
        .replace("module layered {", "module lonely {")
        .replace("use aead;", "use absent;");
    scratch.write("lonely.or", lonely.as_bytes());
    let error = scratch.fails(1, &["schemes", "lonely.or"]);
    assert!(error.starts_with("error[ORC1001]: "), "{error}");
    assert!(
        error.contains(
            "  = note: `use absent;` in module `lonely` reads the module `absent` from this file\n"
        ),
        "{error}"
    );
}

#[test]
fn schemes_lists_the_builtins() {
    let scratch = Scratch::new("schemes");
    let table = scratch.succeeds(&["schemes"]);
    let lines = table.lines().collect::<Vec<_>>();
    assert_eq!(
        lines[0],
        "scheme              key  nonce  tag  chunk  seal steps  source"
    );
    assert!(lines[1].starts_with("xchacha20_poly1305  256    192  128    240  "));
    assert!(lines[1].ends_with("  draft-irtf-cfrg-xchacha-03 over RFC 8439 (default)"));
    assert!(lines[2].starts_with("chacha20_poly1305   256     96  128    240  "));
    assert!(lines[2].ends_with("  RFC 8439"));
    assert!(lines[3].starts_with("ascon_aead128       128    128  128    240  "));
    assert!(lines[3].ends_with("  NIST SP 800-232"));
    assert_eq!(
        lines[4..],
        [
            "",
            "Key, nonce, and tag sizes are in bits; a chunk is that many bytes of plaintext."
        ]
    );
}

#[test]
fn every_builtin_scheme_passes_its_known_answers() {
    for (scheme, _, _) in SCHEMES {
        let path =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!("../../schemes/{scheme}.or"));
        let output = Command::new(env!("CARGO_BIN_EXE_orangec"))
            .arg("eval")
            .arg(&path)
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(0), "{scheme}");
        let values = String::from_utf8(output.stdout).unwrap();
        let answers = values
            .lines()
            .filter(|line| line.contains(": Bool = "))
            .collect::<Vec<_>>();
        assert!(answers.len() >= 3, "{scheme}: {values}");
        for answer in answers {
            assert!(answer.ends_with(": Bool = true"), "{scheme}: {answer}");
        }
    }
}

#[test]
fn sealing_options_are_checked_before_anything_runs() {
    let scratch = Scratch::new("usage");
    for (arguments, message) in [
        (
            &["enc"][..],
            "orangec: command `enc` requires exactly one file\n",
        ),
        (
            &["dec", "a", "b"][..],
            "orangec: command `dec` requires exactly one file\n",
        ),
        (
            &["enc", "-"][..],
            "orangec: command `enc` reads a file, not standard input\n",
        ),
        (
            &["keygen", "file"][..],
            "orangec: command `keygen` takes no file; name the new key file with -o\n",
        ),
        (
            &["keygen", "--key", "k"][..],
            "orangec: command `keygen` writes its key to -o, not --key\n",
        ),
        (
            &["schemes", "--key", "k"][..],
            "orangec: command `schemes` takes scheme names or paths as operands and no options\n",
        ),
        (
            &["check", "--key", "k", "file.or"][..],
            "orangec: option `--key` applies only to keygen, enc, dec, and schemes\n",
        ),
        (
            &["enc", "--scheme", "a", "--scheme=b", "file"][..],
            "orangec: option `--scheme` may be specified at most once\n",
        ),
        (
            &["enc", "-o"][..],
            "orangec: option `-o` requires a value\n",
        ),
    ] {
        let error = scratch.fails(2, arguments);
        assert!(error.starts_with(message), "{arguments:?}: {error}");
    }
}

/// Files sealed by an independent implementation of format 1, written in
/// Python on pycryptodome 3.23.0 and pyascon (SP 800-232), with fixed nonce
/// prefixes: orangec must open them to the expected plaintext.
#[test]
fn files_sealed_by_an_independent_implementation_open() {
    let scratch = Scratch::new("interop");
    // Scheme, key, sealed file in hexadecimal, and plaintext.
    type Case<'hex> = (&'static str, Vec<u8>, &'hex [&'hex str], Vec<u8>);
    let cases: [Case<'_>; 3] = [
        (
            "xchacha20_poly1305",
            counting(32),
            &[
                "6f72616e6765000120181000000000f07863686163686132305f706f6c7931333035000000000000a0a1a2a3a4a5a6a7",
                "a8a9aaabacadaeafb0b1b20000000000d26cee623238bd2ae1ee5813b82d131da3be16f90cb240f5df6aacf63885b7ca",
                "804fa535ea3dc56e777a3907bc3412ca8f883d4ab8ae5e0ef4a7c01d61aed9884e7e2a15f99f7ddc94fba0ac63f45747",
                "c4b3d750474dbb1369810799fe4a7185adf9b550b82b963c84e16268567e32baf2feaac207f704b95652cc6f5f03d67e",
                "5ee51faa4f6ff46f67d9318520c836f5874f1746378d6a8eaadffe8bac12bf6cc1d552f558337b72b372a7ab1c4d7642",
                "1efe4bec01917e76f48b80bf341a04c8e48a3c5dd8a85c5d27782d79b4e19780a5b31f1065d68f9a6511d02e89ec84f0",
                "3a1f4ca57823db4469965669ac0f2f11b2669aa07a815e09d214591c15b681f27d1f1ff38d661299ca6bcc74e77cc741",
                "ba437a83c9ff4f157215083b8970e0ce668a8f473f228a71ed468e3201b9e3ceb5ab5e27d884259bc592bc2682874008",
                "0f0d931900e3dffd05e3d73d4bff75b2ccee9eb1c4678bfec9f10322499e3de52f10a08805bcf6d725b0f39381d27928",
                "435d38a72e078f094d7ce4d3efcfcb15afebde7f5d1cb0b028dc04760a8d66a93eacae8574ca465f37effc909f39fd74",
                "612fb315c0656a3fb447366f37f74e0325df3d919fd58409e31a1eb7875cd3e7f3d7c4be15c5b80b06bffcae15421a01",
                "50d686a30dde50113d27ce55df324edc7ebe4218fb19d2013aef594263e01fe6db9be12b0ddd0c8567396dd5813d105f",
            ],
            counting(300),
        ),
        (
            "chacha20_poly1305",
            counting(32),
            &[
                "6f72616e67650001200c1000000000f063686163686132305f706f6c793133303500000000000000a0a1a2a3a4a5a600",
                "00000000000000000000000000000000e14f062d5840ae2a343f27f86c0e4c720e98f625de73c543c74b294c99e8de6a",
                "f2010b983fb5ac4ef0e28442df511a96af9154859a24b14242f57c61698726edf6e2e4d102e529b0c8b16190f9ee305e",
                "05dd99250779d831f638e05b2e518bc9158532bdeb88635ba070c7ea42f4ab3aedb1da30116b659123f80c782935ff15",
                "011dca5bda5951fcdb4e10ed8faf2f638cd66ba1e8f71c5d6365e1c7e001d992155e09607c31b773d4e16c6ece0df922",
                "7b6e5a7aaac1964e0fc8ec489f73253e27a8a3e24e53d4a34e915a0a36da9e0e75d958e43c6bf0099255256ddd8bb05e",
                "da0e660f47cbcf80b6b54909c0fdd82b59efdbf767c6564cb4086b94ded7b79d",
            ],
            b"Sealed by an independent implementation of format 1.\n".to_vec(),
        ),
        (
            "ascon_aead128",
            counting(16),
            &[
                "6f72616e6765000110101000000000f06173636f6e5f616561643132380000000000000000000000a0a1a2a3a4a5a6a7",
                "a8a9aa00000000000000000000000000105fb5646dd9687b0696b9b6637107afb10611ce069503f4b52e4b0ce2c68aa1",
                "1caf55b89efa69af411d5415d5654cd96dcdc50f5673a3115ab46ebda29add8c1cfe9ac5d69743005c1604bddbb348ce",
                "81fd4b2eafabe05294e40c91af89425045f9d9410615093c21379a36544d1b3220d538d4192f48c57fb6146645733a2a",
                "5b8326a04d50ce3112c27faa88bef605101a0983c446244984be06fc2bbb700a4f82f9b39bfda31e64e79a57236c51b2",
                "b745f8f1e84fd729b713a8a38995d38d648ccb183a5f108edbd156cc840f89c3a9f0accdec40dcb8322efa4fc6917401",
                "3d90bc2f4829e77e8716c0d0ab53d1b5181b87e835a0bd13710f474aadffc11d",
            ],
            Vec::new(),
        ),
    ];
    for (scheme, key, hex, plaintext) in cases {
        let key_name = format!("{scheme}.key");
        let key_hex = key
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        let key_path = scratch.write(
            &key_name,
            format!("orange-key 1 {scheme} {key_hex}\n").as_bytes(),
        );
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt as _;
            fs::set_permissions(&key_path, fs::Permissions::from_mode(0o600)).unwrap();
        }
        let hex = hex.concat();
        let sealed = (0..hex.len())
            .step_by(2)
            .map(|at| u8::from_str_radix(&hex[at..at + 2], 16).unwrap())
            .collect::<Vec<_>>();
        let sealed_name = format!("{scheme}.orange");
        scratch.write(&sealed_name, &sealed);
        scratch.succeeds(&["dec", "--key", &key_name, &sealed_name]);
        assert_eq!(
            fs::read(scratch.path(scheme)).unwrap(),
            plaintext,
            "{scheme}"
        );
    }
}
