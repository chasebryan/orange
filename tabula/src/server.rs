//! Tabula's loopback server: the page, and the small API behind it.
//!
//! The server binds only to `127.0.0.1`. Every request must name that address
//! (or `localhost`) with the bound port in its `Host` header, which defeats DNS
//! rebinding. Every API request must also carry the session token in the
//! `X-Tabula-Token` header and, when the browser supplies them, a same-origin
//! `Origin` and `Sec-Fetch-Site`. Reads use `GET`; every change uses `POST`.

use std::io::{self, BufRead, BufReader, BufWriter, Write};
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, TcpListener, TcpStream};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;
use std::time::Duration;

use crate::assets;
use crate::http::{self, Method, Request, Response};
use crate::json::Json;
use crate::library::Library;
use crate::notes::Notebook;
use crate::orangec::{self, Action, Orangec};
use crate::paths::PathError;
use crate::token::SessionToken;
use crate::workspace::Workspace;

/// Most connections served at once.
pub const MAX_CONNECTIONS: usize = 32;

/// Socket read and write timeout.
pub const SOCKET_TIMEOUT: Duration = Duration::from_secs(30);

/// Tabula's version.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Everything a running Tabula needs.
#[derive(Debug)]
pub struct State {
    /// The workspace being edited.
    pub workspace: Workspace,
    /// Its notebook.
    pub notebook: Notebook,
    /// The Orange checkout providing the Library, if found.
    pub library: Option<Library>,
    /// The compiler, if found.
    pub orangec: Option<Orangec>,
    /// The session token.
    pub token: SessionToken,
    /// The bound port.
    pub port: u16,
}

/// A bound, not yet running, server.
#[derive(Debug)]
pub struct Server {
    listener: TcpListener,
    state: Arc<State>,
}

/// Where and how to bind.
#[derive(Debug)]
pub struct Options {
    /// The workspace folder.
    pub workspace: PathBuf,
    /// Preferred port; `0` picks a free one.
    pub port: u16,
    /// Fall back to a free port when `port` is taken.
    pub port_fallback: bool,
    /// An explicit orangec.
    pub orangec: Option<PathBuf>,
    /// An explicit Orange checkout for the Library.
    pub library: Option<PathBuf>,
    /// A fixed token (tests only); `None` draws a fresh one.
    pub token: Option<SessionToken>,
}

impl Server {
    /// Opens the workspace, finds orangec and the Library, and binds.
    ///
    /// # Errors
    ///
    /// Returns a message when the workspace cannot be opened, no random
    /// source exists, or no port can be bound.
    pub fn bind(options: Options) -> Result<Self, String> {
        let workspace = Workspace::open(&options.workspace).map_err(|error| {
            format!(
                "cannot open workspace {}: {error}",
                options.workspace.display()
            )
        })?;
        let library = Library::locate(options.library.as_deref(), workspace.root());
        if options.library.is_some() && library.is_none() {
            return Err(
                "the --library folder is not an Orange checkout (no docs/THE_ORANGE_BOOK.md)"
                    .to_owned(),
            );
        }
        let orangec = Orangec::locate(
            options.orangec.as_deref(),
            library.as_ref().map(Library::root),
        );
        if options.orangec.is_some() && orangec.is_none() {
            return Err("the --orangec program did not answer `--version` like orangec".to_owned());
        }
        let token = match options.token {
            Some(token) => token,
            None => {
                SessionToken::generate().map_err(|error| format!("no random source: {error}"))?
            }
        };
        let address = |port| SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, port));
        let listener = match TcpListener::bind(address(options.port)) {
            Ok(listener) => listener,
            Err(_) if options.port_fallback => TcpListener::bind(address(0))
                .map_err(|error| format!("cannot bind a port: {error}"))?,
            Err(error) => return Err(format!("cannot bind port {}: {error}", options.port)),
        };
        let port = listener
            .local_addr()
            .map_err(|error| format!("cannot read the bound port: {error}"))?
            .port();
        let notebook = Notebook::new(workspace.root());
        Ok(Self {
            listener,
            state: Arc::new(State {
                workspace,
                notebook,
                library,
                orangec,
                token,
                port,
            }),
        })
    }

    /// The shared state.
    #[must_use]
    pub fn state(&self) -> &State {
        &self.state
    }

    /// The link that opens Tabula, carrying the session token.
    #[must_use]
    pub fn launch_url(&self) -> String {
        format!(
            "http://127.0.0.1:{}/?k={}",
            self.state.port,
            self.state.token.as_str()
        )
    }

    /// Serves connections until the process ends.
    pub fn run(self) {
        let active = Arc::new(AtomicUsize::new(0));
        for stream in self.listener.incoming() {
            let Ok(stream) = stream else {
                continue;
            };
            let state = Arc::clone(&self.state);
            let active = Arc::clone(&active);
            if active.fetch_add(1, Ordering::SeqCst) >= MAX_CONNECTIONS {
                active.fetch_sub(1, Ordering::SeqCst);
                let _ = stream.set_write_timeout(Some(SOCKET_TIMEOUT));
                let mut writer = BufWriter::new(&stream);
                let _ = Response::error(503, "Tabula is busy").write_to(&mut writer, true);
                continue;
            }
            let spawned = thread::Builder::new()
                .name("tabula-connection".to_owned())
                .spawn({
                    let active = Arc::clone(&active);
                    move || {
                        serve_connection(&stream, &state);
                        active.fetch_sub(1, Ordering::SeqCst);
                    }
                });
            if spawned.is_err() {
                active.fetch_sub(1, Ordering::SeqCst);
            }
        }
    }
}

fn serve_connection(stream: &TcpStream, state: &State) {
    let _ = stream.set_read_timeout(Some(SOCKET_TIMEOUT));
    let _ = stream.set_write_timeout(Some(SOCKET_TIMEOUT));
    let _ = respond(
        &mut BufReader::new(stream),
        &mut BufWriter::new(stream),
        state,
    );
}

/// Reads one request from `reader` and writes the answer to `writer`: what a
/// connection does, over any byte streams, so tests need no network.
///
/// # Errors
///
/// Returns an error when the answer cannot be written.
pub fn respond(
    reader: &mut impl BufRead,
    writer: &mut impl Write,
    state: &State,
) -> io::Result<()> {
    let (response, include_body) = match http::read_request(reader) {
        Ok(request) => (handle(&request, state), request.method != Method::Head),
        Err(http::HttpError::Closed) => return Ok(()),
        Err(error) => (Response::error(error.status(), &error.to_string()), true),
    };
    response.write_to(writer, include_body)?;
    writer.flush()
}

/// Answers one request. Public so tests can drive the router without sockets.
#[must_use]
pub fn handle(request: &Request, state: &State) -> Response {
    if !host_is_allowed(request.header("host"), state.port) {
        return Response::error(421, "Tabula answers only on 127.0.0.1 and localhost");
    }
    if let Some(path) = request.path.strip_prefix("/api/") {
        if let Err(response) = authorize(request, state) {
            return response;
        }
        return api(path, request, state);
    }
    if !matches!(request.method, Method::Get | Method::Head) {
        return Response::error(405, "method not allowed");
    }
    match assets::lookup(&request.path) {
        Some(asset) => Response::new(200, asset.media_type, asset.bytes.to_vec()),
        None => Response::error(404, "not found"),
    }
}

fn host_is_allowed(host: Option<&str>, port: u16) -> bool {
    let Some(host) = host else {
        return false;
    };
    let expected_ip = format!("127.0.0.1:{port}");
    let expected_name = format!("localhost:{port}");
    host.eq_ignore_ascii_case(&expected_ip) || host.eq_ignore_ascii_case(&expected_name)
}

fn authorize(request: &Request, state: &State) -> Result<(), Response> {
    if let Some(origin) = request.header("origin") {
        let allowed = [
            format!("http://127.0.0.1:{}", state.port),
            format!("http://localhost:{}", state.port),
        ];
        if !allowed
            .iter()
            .any(|candidate| origin.eq_ignore_ascii_case(candidate))
        {
            return Err(Response::error(403, "cross-origin requests are refused"));
        }
    }
    if let Some(site) = request.header("sec-fetch-site")
        && !matches!(site, "same-origin" | "none")
    {
        return Err(Response::error(403, "cross-site requests are refused"));
    }
    let presented = request.header("x-tabula-token").unwrap_or("");
    if !state.token.matches(presented) {
        return Err(Response::error(
            401,
            "this page is not connected to this Tabula session; open the link Tabula printed",
        ));
    }
    Ok(())
}

fn api(path: &str, request: &Request, state: &State) -> Response {
    let reading = matches!(request.method, Method::Get | Method::Head);
    let writing = request.method == Method::Post;
    match (path, reading, writing) {
        ("session", true, _) => session(state),
        ("tree", true, _) => Response::json(200, &state.workspace.tree()),
        ("file", true, _) => match state.workspace.read(param(request, "path")) {
            Ok(text) => ok_json([
                ("path", Json::str(param(request, "path"))),
                ("text", Json::str(text)),
            ]),
            Err(error) => path_error(&error),
        },
        ("file/save", _, true) => with_text(request, |text| {
            state
                .workspace
                .write(param(request, "path"), text, false)
                .map(|()| Json::Null)
        }),
        ("file/create", _, true) => with_text(request, |text| {
            state
                .workspace
                .write(param(request, "path"), text, true)
                .map(|()| Json::Null)
        }),
        ("folder/create", _, true) => done(
            state
                .workspace
                .create_folder(param(request, "path"))
                .map(|()| Json::Null),
        ),
        ("file/rename", _, true) => done(
            state
                .workspace
                .rename(param(request, "from"), param(request, "to"))
                .map(|()| Json::Null),
        ),
        ("file/delete", _, true) => done(
            state
                .workspace
                .delete(param(request, "path"))
                .map(Json::str),
        ),
        ("orangec/check", _, true) => compile(request, state, Action::Check),
        ("orangec/eval", _, true) => compile(request, state, Action::Eval),
        ("orangec/lex", _, true) => compile(request, state, Action::Lex),
        ("library", true, _) => match &state.library {
            Some(library) => Response::json(200, &library.catalog()),
            None => Response::error(404, "no Orange checkout was found for the Library"),
        },
        ("library/doc", true, _) => match &state.library {
            Some(library) => match library.read(param(request, "path")) {
                Ok(text) => ok_json([
                    ("path", Json::str(param(request, "path"))),
                    ("text", Json::str(text)),
                ]),
                Err(error) => path_error(&error),
            },
            None => Response::error(404, "no Orange checkout was found for the Library"),
        },
        ("library/asset", true, _) => match &state.library {
            Some(library) => match library.asset(param(request, "path")) {
                Ok((bytes, media_type)) => Response::new(200, media_type, bytes),
                Err(error) => path_error(&error),
            },
            None => Response::error(404, "no Orange checkout was found for the Library"),
        },
        ("library/search", true, _) => match &state.library {
            Some(library) => Response::json(200, &library.search(param(request, "q"))),
            None => Response::error(404, "no Orange checkout was found for the Library"),
        },
        ("notes", true, _) => Response::json(200, &state.notebook.list()),
        ("note", true, _) => match state.notebook.read(param(request, "name")) {
            Ok(text) => ok_json([
                ("name", Json::str(param(request, "name"))),
                ("text", Json::str(text)),
            ]),
            Err(error) => path_error(&error),
        },
        ("note/save", _, true) => with_text(request, |text| {
            state
                .notebook
                .save(param(request, "name"), text)
                .map(|()| Json::Null)
        }),
        ("note/create", _, true) => with_text(request, |text| {
            let requested = request.param("name").filter(|name| !name.is_empty());
            state.notebook.create(requested, text).map(Json::str)
        }),
        ("note/rename", _, true) => done(
            state
                .notebook
                .rename(param(request, "from"), param(request, "to"))
                .map(|()| Json::Null),
        ),
        ("note/delete", _, true) => done(
            state
                .notebook
                .delete(param(request, "name"))
                .map(|()| Json::Null),
        ),
        (
            "session" | "tree" | "file" | "library" | "library/doc" | "library/asset"
            | "library/search" | "notes" | "note",
            false,
            _,
        )
        | (
            "file/save" | "file/create" | "folder/create" | "file/rename" | "file/delete"
            | "orangec/check" | "orangec/eval" | "orangec/lex" | "note/save" | "note/create"
            | "note/rename" | "note/delete",
            _,
            false,
        ) => Response::error(405, "method not allowed"),
        _ => Response::error(404, "no such API"),
    }
}

fn param<'a>(request: &'a Request, name: &str) -> &'a str {
    request.param(name).unwrap_or("")
}

fn session(state: &State) -> Response {
    let orangec = state.orangec.as_ref().map_or(Json::Null, |compiler| {
        Json::object([
            ("path", Json::str(compiler.path().display().to_string())),
            ("version", Json::str(compiler.version())),
        ])
    });
    let library = state.library.as_ref().map_or(Json::Null, |library| {
        Json::object([("root", Json::str(library.root().display().to_string()))])
    });
    ok_json([
        ("tabula", Json::str(VERSION)),
        ("workspace", Json::str(state.workspace.name())),
        (
            "workspaceRoot",
            Json::str(state.workspace.root().display().to_string()),
        ),
        ("orangec", orangec),
        ("library", library),
    ])
}

fn compile(request: &Request, state: &State, action: Action) -> Response {
    let Some(compiler) = &state.orangec else {
        return Response::error(
            503,
            "orangec was not found; start Tabula with --orangec <path>",
        );
    };
    let Ok(source) = std::str::from_utf8(&request.body) else {
        return Response::error(415, "source must be UTF-8");
    };
    let output = match compiler.run(action, source) {
        Ok(output) => output,
        Err(error) => return Response::error(500, &format!("could not run orangec: {error}")),
    };
    let diagnostics = orangec::parse_diagnostics(&output.stderr, source);
    let mut members = vec![
        ("ok".to_owned(), Json::Bool(true)),
        ("action".to_owned(), Json::str(action.word())),
        (
            "status".to_owned(),
            output
                .status
                .map_or(Json::Null, |code| Json::Signed(i64::from(code))),
        ),
        (
            "success".to_owned(),
            Json::Bool(output.status == Some(0) && !output.timed_out),
        ),
        ("timedOut".to_owned(), Json::Bool(output.timed_out)),
        ("truncated".to_owned(), Json::Bool(output.truncated)),
        ("millis".to_owned(), Json::Number(output.millis)),
        ("version".to_owned(), Json::str(compiler.version())),
        (
            "diagnostics".to_owned(),
            Json::Array(
                diagnostics
                    .iter()
                    .map(orangec::Diagnostic::to_json)
                    .collect(),
            ),
        ),
        ("stderr".to_owned(), Json::str(output.stderr.clone())),
    ];
    match action {
        Action::Check => {}
        Action::Eval => {
            let values = orangec::parse_values(&output.stdout)
                .into_iter()
                .map(|value| {
                    Json::object([
                        ("module", Json::str(value.module)),
                        ("name", Json::str(value.name)),
                        ("type", Json::str(value.ty)),
                        ("value", Json::str(value.value)),
                    ])
                })
                .collect();
            members.push(("values".to_owned(), Json::Array(values)));
            members.push(("stdout".to_owned(), Json::str(output.stdout)));
        }
        Action::Lex => {
            let tokens = orangec::parse_tokens(&output.stdout, source)
                .into_iter()
                .map(|token| {
                    Json::object([
                        ("start", Json::usize(token.start)),
                        ("end", Json::usize(token.end)),
                        ("kind", Json::str(token.kind)),
                        ("spelling", Json::str(token.spelling)),
                    ])
                })
                .collect();
            members.push(("tokens".to_owned(), Json::Array(tokens)));
        }
    }
    Response::json(200, &Json::Object(members))
}

fn with_text(request: &Request, action: impl FnOnce(&str) -> Result<Json, PathError>) -> Response {
    match std::str::from_utf8(&request.body) {
        Ok(text) => done(action(text)),
        Err(_) => Response::error(415, "text must be UTF-8"),
    }
}

fn done(result: Result<Json, PathError>) -> Response {
    match result {
        Ok(Json::Null) => ok_json([]),
        Ok(value) => ok_json([("result", value)]),
        Err(error) => path_error(&error),
    }
}

fn ok_json<const N: usize>(members: [(&str, Json); N]) -> Response {
    let mut object = vec![("ok".to_owned(), Json::Bool(true))];
    object.extend(
        members
            .into_iter()
            .map(|(name, value)| (name.to_owned(), value)),
    );
    Response::json(200, &Json::Object(object))
}

fn path_error(error: &PathError) -> Response {
    let status = match error {
        PathError::Invalid(_) => 422,
        PathError::NotFound => 404,
        PathError::Exists => 409,
        PathError::Io(_) => 500,
    };
    Response::error(status, &error.to_string())
}

#[cfg(test)]
mod tests {
    use super::{Options, Server, handle};
    use crate::http::{Method, Request};
    use crate::token::SessionToken;
    use std::fs;

    fn server(name: &str) -> (Server, std::path::PathBuf) {
        let root =
            std::env::temp_dir().join(format!("tabula-server-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        let server = Server::bind(Options {
            workspace: root.clone(),
            port: 0,
            port_fallback: true,
            orangec: None,
            library: None,
            token: Some(SessionToken::from_bytes(&[7; 16])),
        })
        .unwrap();
        (server, root)
    }

    fn request(server: &Server, method: Method, path: &str, headers: &[(&str, &str)]) -> Request {
        let port = server.state().port;
        let mut all = vec![("host".to_owned(), format!("127.0.0.1:{port}"))];
        all.extend(
            headers
                .iter()
                .map(|(name, value)| ((*name).to_owned(), (*value).to_owned())),
        );
        let (path, query) = path.split_once('?').unwrap_or((path, ""));
        Request {
            method,
            path: path.to_owned(),
            query: crate::http::parse_query(query).unwrap(),
            headers: all,
            body: Vec::new(),
        }
    }

    #[test]
    fn serves_the_page_without_a_token_but_guards_the_api() {
        let (server, root) = server("guard");
        let token = "07".repeat(16);
        let page = handle(&request(&server, Method::Get, "/", &[]), server.state());
        assert_eq!(page.status, 200);
        assert!(page.content_type.starts_with("text/html"));

        let no_token = handle(
            &request(&server, Method::Get, "/api/session", &[]),
            server.state(),
        );
        assert_eq!(no_token.status, 401);
        let good = handle(
            &request(
                &server,
                Method::Get,
                "/api/session",
                &[("x-tabula-token", &token)],
            ),
            server.state(),
        );
        assert_eq!(good.status, 200);
        let cross = handle(
            &request(
                &server,
                Method::Get,
                "/api/session",
                &[
                    ("x-tabula-token", &token),
                    ("origin", "https://evil.example"),
                ],
            ),
            server.state(),
        );
        assert_eq!(cross.status, 403);
        let cross_site = handle(
            &request(
                &server,
                Method::Get,
                "/api/session",
                &[("x-tabula-token", &token), ("sec-fetch-site", "cross-site")],
            ),
            server.state(),
        );
        assert_eq!(cross_site.status, 403);

        let mut rebound = request(&server, Method::Get, "/", &[]);
        rebound.headers = vec![("host".to_owned(), "attacker.example".to_owned())];
        assert_eq!(handle(&rebound, server.state()).status, 421);

        let wrong_method = handle(
            &request(
                &server,
                Method::Get,
                "/api/file/save?path=a.or",
                &[("x-tabula-token", &token)],
            ),
            server.state(),
        );
        assert_eq!(wrong_method.status, 405);
        let missing = handle(&request(&server, Method::Get, "/nope", &[]), server.state());
        assert_eq!(missing.status, 404);
        let no_compiler = handle(
            &request(
                &server,
                Method::Post,
                "/api/orangec/check",
                &[("x-tabula-token", &token)],
            ),
            server.state(),
        );
        assert!(no_compiler.status == 503 || no_compiler.status == 200);
        assert!(server.launch_url().ends_with(&format!("/?k={token}")));
        fs::remove_dir_all(&root).unwrap();
    }
}
