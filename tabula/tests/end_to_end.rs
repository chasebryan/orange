//! End-to-end tests: Tabula's server driven the way the page drives it, with
//! the real `orangec` and this repository as the Library.
//!
//! The tests need a built `orangec`. They use `TABULA_TEST_ORANGEC` when it is
//! set, and otherwise `compiler/target/release/orangec` or
//! `compiler/target/debug/orangec` in this checkout.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use tabula::http::{Method, Request, Response, parse_query};
use tabula::server::{Options, Server, handle, respond};
use tabula::token::SessionToken;

const TOKEN_BYTES: [u8; 16] = [0x5a; 16];

fn checkout() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..")
}

fn orangec() -> PathBuf {
    if let Some(path) = std::env::var_os("TABULA_TEST_ORANGEC") {
        return PathBuf::from(path);
    }
    let name = if cfg!(windows) {
        "orangec.exe"
    } else {
        "orangec"
    };
    ["release", "debug"]
        .iter()
        .map(|profile| checkout().join("compiler").join("target").join(profile).join(name))
        .find(|path| path.is_file())
        .unwrap_or_else(|| {
            panic!(
                "these tests need a built orangec: run `cargo build --manifest-path compiler/Cargo.toml -p orangec` \
                 or set TABULA_TEST_ORANGEC"
            )
        })
}

struct Fixture {
    server: Server,
    root: PathBuf,
    token: String,
}

impl Fixture {
    fn new(name: &str) -> Self {
        // Cargo's per-target scratch folder, fixed at build time.
        let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join(format!("tabula-e2e-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("ciphers")).unwrap();
        fs::write(
            root.join("ciphers/chacha.or"),
            "edition 2026;\n\n// Constants → exact values.\nmodule chacha {\n  spec rounds() -> Int { 20 }\n  spec sigma() -> Word[8] { 0x65 }\n  impl block() {}\n}\n",
        )
        .unwrap();
        let server = Server::bind(Options {
            workspace: root.clone(),
            port: 0,
            port_fallback: true,
            orangec: Some(orangec()),
            library: Some(checkout()),
            token: Some(SessionToken::from_bytes(&TOKEN_BYTES)),
        })
        .unwrap();
        let token = SessionToken::from_bytes(&TOKEN_BYTES).as_str().to_owned();
        Self {
            server,
            root,
            token,
        }
    }

    fn call(&self, method: Method, target: &str, body: &str) -> (u16, String) {
        let response = self.raw(method, target, body, true);
        (response.status, String::from_utf8(response.body).unwrap())
    }

    fn raw(&self, method: Method, target: &str, body: &str, with_token: bool) -> Response {
        let (path, query) = target.split_once('?').unwrap_or((target, ""));
        let mut headers = vec![
            (
                "host".to_owned(),
                format!("127.0.0.1:{}", self.server.state().port),
            ),
            (
                "origin".to_owned(),
                format!("http://127.0.0.1:{}", self.server.state().port),
            ),
            ("sec-fetch-site".to_owned(), "same-origin".to_owned()),
        ];
        if with_token {
            headers.push(("x-tabula-token".to_owned(), self.token.clone()));
        }
        let request = Request {
            method,
            path: path.to_owned(),
            query: parse_query(query).unwrap(),
            headers,
            body: body.as_bytes().to_vec(),
        };
        handle(&request, self.server.state())
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

#[test]
fn checks_evaluates_and_lexes_with_the_real_compiler() {
    let fixture = Fixture::new("compile");
    let (status, session) = fixture.call(Method::Get, "/api/session", "");
    assert_eq!(status, 200, "{session}");
    assert!(session.contains(r#""version":"orangec "#), "{session}");

    let source = fs::read_to_string(fixture.root.join("ciphers/chacha.or")).unwrap();
    let (status, checked) = fixture.call(Method::Post, "/api/orangec/check", &source);
    assert_eq!(status, 200, "{checked}");
    assert!(checked.contains(r#""success":true"#), "{checked}");
    assert!(checked.contains(r#""diagnostics":[]"#), "{checked}");

    let (_, evaluated) = fixture.call(Method::Post, "/api/orangec/eval", &source);
    assert!(evaluated.contains(r#""success":true"#), "{evaluated}");
    assert!(
        evaluated.contains(r#"{"module":"chacha","name":"rounds","type":"Int","value":"20"}"#),
        "{evaluated}"
    );
    assert!(
        evaluated.contains(r#"{"module":"chacha","name":"sigma","type":"Word[8]","value":"0x65"}"#),
        "{evaluated}"
    );

    // Offsets are UTF-16 code units in the text that was sent: the arrow in
    // the comment is one unit but three bytes, so `module` starts at 44.
    let (_, lexed) = fixture.call(Method::Post, "/api/orangec/lex", &source);
    assert!(
        lexed.contains(r#"{"start":0,"end":7,"kind":"KW_EDITION","spelling":"edition"}"#),
        "{lexed}"
    );
    assert!(
        lexed.contains(r#"{"start":44,"end":50,"kind":"KW_MODULE","spelling":"module"}"#),
        "{lexed}"
    );

    let broken = "edition 2026;\nmodule m {\n  spec bad() -> Word[8] { 0x1ff }\n  game g() {}\n}\n";
    let (_, rejected) = fixture.call(Method::Post, "/api/orangec/eval", broken);
    assert!(rejected.contains(r#""success":false"#), "{rejected}");
    assert!(rejected.contains(r#""code":"ORC0103""#), "{rejected}");
    assert!(rejected.contains(r#""phase":"syntax""#), "{rejected}");
    assert!(rejected.contains(r#""line":4,"column":3"#), "{rejected}");

    let (_, word) = fixture.call(
        Method::Post,
        "/api/orangec/check",
        "edition 2026;\nmodule m {\n  spec bad() -> Word[8] { 0x1ff }\n}\n",
    );
    assert!(word.contains(r#""code":"ORC0207""#), "{word}");
    assert!(word.contains(r#""phase":"semantic""#), "{word}");
}

#[test]
fn edits_the_workspace_and_keeps_deleted_files_in_the_trash() {
    let fixture = Fixture::new("files");
    let (status, tree) = fixture.call(Method::Get, "/api/tree", "");
    assert_eq!(status, 200);
    assert!(tree.contains(r#""path":"ciphers/chacha.or""#), "{tree}");

    let text = "edition 2026;\nmodule aes {\n  spec rounds() -> Int { 10 }\n}\n";
    assert_eq!(
        fixture
            .call(Method::Post, "/api/file/create?path=ciphers/aes.or", text)
            .0,
        200
    );
    assert_eq!(
        fixture
            .call(Method::Post, "/api/file/create?path=ciphers/aes.or", text)
            .0,
        409
    );
    assert_eq!(
        fixture
            .call(
                Method::Post,
                "/api/file/save?path=ciphers/aes.or",
                "edition 2026;\n"
            )
            .0,
        200
    );
    let (_, read) = fixture.call(Method::Get, "/api/file?path=ciphers/aes.or", "");
    assert!(read.contains(r#""text":"edition 2026;\n""#), "{read}");

    assert_eq!(
        fixture
            .call(Method::Post, "/api/file/create?path=notes.txt", "x")
            .0,
        422
    );
    assert_eq!(
        fixture
            .call(Method::Get, "/api/file?path=../escape.or", "")
            .0,
        422
    );
    assert_eq!(
        fixture
            .call(Method::Get, "/api/file?path=.tabula/x.or", "")
            .0,
        422
    );

    assert_eq!(
        fixture
            .call(
                Method::Post,
                "/api/file/rename?from=ciphers/aes.or&to=ciphers/aes128.or",
                ""
            )
            .0,
        200
    );
    let (status, deleted) =
        fixture.call(Method::Post, "/api/file/delete?path=ciphers/aes128.or", "");
    assert_eq!(status, 200, "{deleted}");
    assert!(deleted.contains(r#""result":".tabula/trash/"#), "{deleted}");
    assert!(!fixture.root.join("ciphers/aes128.or").exists());
    let ignore = fs::read_to_string(fixture.root.join(".tabula/trash/.gitignore")).unwrap();
    assert!(ignore.ends_with("*\n"));
}

#[test]
fn keeps_notes_with_citations() {
    let fixture = Fixture::new("notes");
    let (status, created) = fixture.call(Method::Post, "/api/note/create", "");
    assert_eq!(status, 200, "{created}");
    assert!(created.contains(r#""result":"Note 1""#), "{created}");
    let note = "# Rounds\n\nSee [rounds](tabula:ciphers/chacha.or#spec.rounds).\n";
    assert_eq!(
        fixture
            .call(Method::Post, "/api/note/save?name=Note%201", note)
            .0,
        200
    );
    let (_, listed) = fixture.call(Method::Get, "/api/notes", "");
    assert!(listed.contains(r#""title":"Rounds""#), "{listed}");
    assert!(
        listed.contains(r#""path":"ciphers/chacha.or","anchor":"spec.rounds""#),
        "{listed}"
    );
    assert!(fixture.root.join(".tabula/notes/Note 1.md").is_file());
    assert_eq!(
        fixture
            .call(Method::Post, "/api/note/rename?from=Note%201&to=Rounds", "")
            .0,
        200
    );
    assert_eq!(
        fixture
            .call(Method::Post, "/api/note/delete?name=Rounds", "")
            .0,
        200
    );
    let (_, empty) = fixture.call(Method::Get, "/api/notes", "");
    assert!(empty.contains(r#""notes":[]"#), "{empty}");
}

#[test]
fn reads_the_book_and_manuals_from_this_checkout() {
    let fixture = Fixture::new("library");
    let (status, catalog) = fixture.call(Method::Get, "/api/library", "");
    assert_eq!(status, 200, "{catalog}");
    assert!(catalog.contains(r#""id":"book""#), "{catalog}");
    assert!(
        catalog.contains(r#""path":"docs/THE_ORANGE_BOOK.md""#),
        "{catalog}"
    );
    assert!(
        catalog.contains(r#""path":"docs/LANGUAGE_2026.md","title":"Language reference""#),
        "{catalog}"
    );
    assert!(
        catalog.contains(r#""path":"tabula/README.md","title":"Tabula""#),
        "{catalog}"
    );

    let (status, manual) = fixture.call(Method::Get, "/api/library/doc?path=tabula/README.md", "");
    assert_eq!(status, 200);
    let on_disk = fs::read_to_string(checkout().join("tabula/README.md")).unwrap();
    assert!(manual.contains("# Tabula"));
    assert!(on_disk.starts_with("# Tabula\n"));

    // Results stop at 80, the Book's first, so the query is one the Book
    // uses rarely and the language reference often, whatever the Book's length.
    let (_, found) = fixture.call(Method::Get, "/api/library/search?q=reserved%20word", "");
    assert!(
        found.contains(r#""path":"docs/LANGUAGE_2026.md""#),
        "{found}"
    );

    for refused in [
        "/api/library/doc?path=../etc/passwd",
        "/api/library/doc?path=.git/config",
        "/api/library/doc?path=compiler/target/debug/orangec",
        "/api/library/doc?path=tabula/target/x.md",
    ] {
        let (status, body) = fixture.call(Method::Get, refused, "");
        assert!(status == 422 || status == 404, "{refused}: {status} {body}");
    }
}

#[test]
fn answers_raw_http_like_a_connection() {
    let fixture = Fixture::new("wire");
    let port = fixture.server.state().port;
    let exchange = |request: String| {
        let mut reader = io::Cursor::new(request.into_bytes());
        let mut written = Vec::new();
        respond(&mut reader, &mut written, fixture.server.state()).unwrap();
        String::from_utf8(written).unwrap()
    };
    let page = exchange(format!(
        "GET / HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nConnection: close\r\n\r\n"
    ));
    assert!(page.starts_with("HTTP/1.1 200"), "{page}");
    assert!(
        page.contains("Content-Security-Policy: default-src 'none'"),
        "{page}"
    );
    assert!(page.contains("<title>Tabula</title>"), "{page}");

    let anonymous = exchange(format!(
        "GET /api/tree HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\n\r\n"
    ));
    assert!(anonymous.starts_with("HTTP/1.1 401"), "{anonymous}");

    let rebound = exchange(format!(
        "GET /api/tree HTTP/1.1\r\nHost: tabula.example:{port}\r\nX-Tabula-Token: {}\r\n\r\n",
        fixture.token
    ));
    assert!(rebound.starts_with("HTTP/1.1 421"), "{rebound}");

    let body = "edition 2026;\nmodule m {\n  spec answer() -> Int { 42 }\n}\n";
    let evaluated = exchange(format!(
        "POST /api/orangec/eval HTTP/1.1\r\nHost: localhost:{port}\r\nX-Tabula-Token: {}\r\nContent-Length: {}\r\n\r\n{body}",
        fixture.token,
        body.len()
    ));
    assert!(evaluated.starts_with("HTTP/1.1 200"), "{evaluated}");
    assert!(
        evaluated.contains(r#""name":"answer","type":"Int","value":"42""#),
        "{evaluated}"
    );

    let head = exchange(format!("HEAD / HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\n\r\n"));
    assert!(
        head.starts_with("HTTP/1.1 200") && head.ends_with("\r\n\r\n"),
        "{head}"
    );
    assert_eq!(exchange(String::new()), "");
}
