//! A deliberately small HTTP/1.1 subset for Tabula's loopback server.
//!
//! Tabula serves exactly one browser on the same machine, so the server reads
//! one request per connection, answers it, and closes the connection. Every
//! size is bounded before bytes are buffered, and anything outside the subset
//! (chunked bodies, folded headers, absolute-form targets) is refused rather
//! than guessed at.

use std::fmt;
use std::io::{self, BufRead, Read, Write};

/// Largest request head (request line plus headers) Tabula will buffer.
pub const MAX_HEAD_BYTES: usize = 16 * 1024;

/// Largest request body Tabula will buffer: one 16 MiB Orange source plus
/// headroom for notes and small envelopes.
pub const MAX_BODY_BYTES: usize = 17 * 1024 * 1024;

/// Largest number of request headers accepted.
pub const MAX_HEADERS: usize = 64;

/// An HTTP request method understood by Tabula.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Method {
    /// `GET`
    Get,
    /// `HEAD`, answered like `GET` without a body.
    Head,
    /// `POST`
    Post,
}

/// A parsed request.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Request {
    /// The request method.
    pub method: Method,
    /// The percent-decoded path component of the target.
    pub path: String,
    /// Percent-decoded query parameters in order.
    pub query: Vec<(String, String)>,
    /// Header fields with lowercase names, in order.
    pub headers: Vec<(String, String)>,
    /// The request body.
    pub body: Vec<u8>,
}

impl Request {
    /// Returns the value of the first header with this lowercase name.
    #[must_use]
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(candidate, _)| candidate == name)
            .map(|(_, value)| value.as_str())
    }

    /// Returns the value of the first query parameter with this name.
    #[must_use]
    pub fn param(&self, name: &str) -> Option<&str> {
        self.query
            .iter()
            .find(|(candidate, _)| candidate == name)
            .map(|(_, value)| value.as_str())
    }
}

/// Why a request could not be read.
#[derive(Debug)]
pub enum HttpError {
    /// The connection closed before a complete request arrived.
    Closed,
    /// The request is malformed or outside Tabula's HTTP subset.
    BadRequest(&'static str),
    /// The request head or body is larger than Tabula accepts.
    TooLarge,
    /// The method is not one Tabula serves.
    MethodNotAllowed,
    /// The request uses a feature Tabula does not implement.
    NotImplemented(&'static str),
    /// The underlying stream failed.
    Io(io::Error),
}

impl fmt::Display for HttpError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Closed => formatter.write_str("connection closed"),
            Self::BadRequest(reason) => write!(formatter, "bad request: {reason}"),
            Self::TooLarge => formatter.write_str("request too large"),
            Self::MethodNotAllowed => formatter.write_str("method not allowed"),
            Self::NotImplemented(reason) => write!(formatter, "not implemented: {reason}"),
            Self::Io(error) => write!(formatter, "i/o error: {error}"),
        }
    }
}

impl HttpError {
    /// The status code that best describes this error.
    #[must_use]
    pub const fn status(&self) -> u16 {
        match self {
            Self::Closed | Self::Io(_) | Self::BadRequest(_) => 400,
            Self::TooLarge => 413,
            Self::MethodNotAllowed => 405,
            Self::NotImplemented(_) => 501,
        }
    }
}

/// Reads one request from `reader`.
///
/// # Errors
///
/// Returns an [`HttpError`] when the stream closes early, the request falls
/// outside Tabula's HTTP subset, or a size bound is exceeded.
pub fn read_request(reader: &mut impl BufRead) -> Result<Request, HttpError> {
    let head = read_head(reader)?;
    let head =
        std::str::from_utf8(&head).map_err(|_| HttpError::BadRequest("head is not UTF-8"))?;
    let mut lines = head.split("\r\n");
    let request_line = lines
        .next()
        .ok_or(HttpError::BadRequest("missing request line"))?;
    let mut parts = request_line.split(' ');
    let method = parts
        .next()
        .ok_or(HttpError::BadRequest("missing method"))?;
    let target = parts
        .next()
        .ok_or(HttpError::BadRequest("missing target"))?;
    let version = parts
        .next()
        .ok_or(HttpError::BadRequest("missing version"))?;
    if parts.next().is_some() {
        return Err(HttpError::BadRequest("malformed request line"));
    }
    if version != "HTTP/1.1" && version != "HTTP/1.0" {
        return Err(HttpError::BadRequest("unsupported HTTP version"));
    }
    let method = match method {
        "GET" => Method::Get,
        "HEAD" => Method::Head,
        "POST" => Method::Post,
        _ => return Err(HttpError::MethodNotAllowed),
    };
    if !target.starts_with('/') {
        return Err(HttpError::BadRequest(
            "only origin-form targets are accepted",
        ));
    }
    let (raw_path, raw_query) = target.split_once('?').unwrap_or((target, ""));
    let path = percent_decode(raw_path).ok_or(HttpError::BadRequest("invalid path encoding"))?;
    let query = parse_query(raw_query).ok_or(HttpError::BadRequest("invalid query encoding"))?;

    let mut headers = Vec::new();
    for line in lines {
        if line.is_empty() {
            continue;
        }
        if line.starts_with(' ') || line.starts_with('\t') {
            return Err(HttpError::BadRequest("folded headers are not accepted"));
        }
        let (name, value) = line
            .split_once(':')
            .ok_or(HttpError::BadRequest("malformed header"))?;
        if name.is_empty() || !name.bytes().all(is_token_byte) {
            return Err(HttpError::BadRequest("malformed header name"));
        }
        if headers.len() >= MAX_HEADERS {
            return Err(HttpError::TooLarge);
        }
        headers.push((
            name.to_ascii_lowercase(),
            value.trim_matches([' ', '\t']).to_owned(),
        ));
    }

    if headers.iter().any(|(name, _)| name == "transfer-encoding") {
        return Err(HttpError::NotImplemented("transfer-encoding"));
    }
    let mut lengths = headers
        .iter()
        .filter(|(name, _)| name == "content-length")
        .map(|(_, value)| value.as_str());
    let length = match (lengths.next(), lengths.next()) {
        (None, _) => 0,
        (Some(value), None) => parse_content_length(value)?,
        (Some(_), Some(_)) => return Err(HttpError::BadRequest("repeated content-length")),
    };
    if length > MAX_BODY_BYTES {
        return Err(HttpError::TooLarge);
    }
    let mut body = Vec::new();
    body.try_reserve_exact(length)
        .map_err(|_| HttpError::TooLarge)?;
    let limit = u64::try_from(length).map_err(|_| HttpError::TooLarge)?;
    reader
        .take(limit)
        .read_to_end(&mut body)
        .map_err(HttpError::Io)?;
    if body.len() != length {
        return Err(HttpError::Closed);
    }
    Ok(Request {
        method,
        path,
        query,
        headers,
        body,
    })
}

fn read_head(reader: &mut impl BufRead) -> Result<Vec<u8>, HttpError> {
    let mut head: Vec<u8> = Vec::new();
    loop {
        let available = match reader.fill_buf() {
            Ok(bytes) => bytes,
            Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
            Err(error) => return Err(HttpError::Io(error)),
        };
        if available.is_empty() {
            return Err(HttpError::Closed);
        }
        let mut consumed = 0_usize;
        let mut complete = false;
        for byte in available {
            head.push(*byte);
            consumed = consumed.saturating_add(1);
            if head.ends_with(b"\r\n\r\n") {
                complete = true;
                break;
            }
            if head.len() > MAX_HEAD_BYTES {
                return Err(HttpError::TooLarge);
            }
        }
        reader.consume(consumed);
        if complete {
            let end = head.len().saturating_sub(4);
            head.truncate(end);
            return Ok(head);
        }
    }
}

fn parse_content_length(value: &str) -> Result<usize, HttpError> {
    if value.is_empty() || value.len() > 12 || !value.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(HttpError::BadRequest("invalid content-length"));
    }
    value
        .parse::<usize>()
        .map_err(|_| HttpError::BadRequest("invalid content-length"))
}

const fn is_token_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric()
        || matches!(
            byte,
            b'!' | b'#'
                | b'$'
                | b'%'
                | b'&'
                | b'\''
                | b'*'
                | b'+'
                | b'-'
                | b'.'
                | b'^'
                | b'_'
                | b'`'
                | b'|'
                | b'~'
        )
}

/// Decodes `%XX` escapes as UTF-8. `+` is kept literally.
#[must_use]
pub fn percent_decode(value: &str) -> Option<String> {
    let mut bytes = Vec::with_capacity(value.len());
    let mut input = value.bytes();
    while let Some(byte) = input.next() {
        if byte == b'%' {
            let high = hex_value(input.next()?)?;
            let low = hex_value(input.next()?)?;
            bytes.push(high.checked_mul(16)?.checked_add(low)?);
        } else {
            bytes.push(byte);
        }
    }
    String::from_utf8(bytes).ok()
}

/// Parses an `application/x-www-form-urlencoded` style query string, treating
/// `+` as a space as browsers do for `URLSearchParams`.
#[must_use]
pub fn parse_query(query: &str) -> Option<Vec<(String, String)>> {
    let mut pairs = Vec::new();
    for pair in query.split('&').filter(|pair| !pair.is_empty()) {
        let (name, value) = pair.split_once('=').unwrap_or((pair, ""));
        let name = percent_decode(&name.replace('+', " "))?;
        let value = percent_decode(&value.replace('+', " "))?;
        pairs.push((name, value));
    }
    Some(pairs)
}

const fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => byte.checked_sub(b'0'),
        b'a'..=b'f' => match byte.checked_sub(b'a') {
            Some(value) => value.checked_add(10),
            None => None,
        },
        b'A'..=b'F' => match byte.checked_sub(b'A') {
            Some(value) => value.checked_add(10),
            None => None,
        },
        _ => None,
    }
}

/// A response ready to be written.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Response {
    /// The status code.
    pub status: u16,
    /// The `Content-Type` header value.
    pub content_type: &'static str,
    /// Extra headers beyond Tabula's fixed security headers.
    pub headers: Vec<(&'static str, String)>,
    /// The body bytes.
    pub body: Vec<u8>,
}

impl Response {
    /// A response with the given status, type, and body.
    #[must_use]
    pub fn new(status: u16, content_type: &'static str, body: Vec<u8>) -> Self {
        Self {
            status,
            content_type,
            headers: Vec::new(),
            body,
        }
    }

    /// A JSON response.
    #[must_use]
    pub fn json(status: u16, body: &crate::json::Json) -> Self {
        Self::new(
            status,
            "application/json; charset=utf-8",
            body.render().into_bytes(),
        )
    }

    /// A JSON error response of the form `{"ok":false,"error":"..."}`.
    #[must_use]
    pub fn error(status: u16, message: &str) -> Self {
        Self::json(
            status,
            &crate::json::Json::object([
                ("ok", crate::json::Json::Bool(false)),
                ("error", crate::json::Json::str(message)),
            ]),
        )
    }

    /// Writes the response and flushes the stream. `HEAD` responses omit the body.
    ///
    /// # Errors
    ///
    /// Returns the underlying I/O error.
    pub fn write_to(&self, writer: &mut impl Write, include_body: bool) -> io::Result<()> {
        let mut head = format!(
            "HTTP/1.1 {} {}\r\nContent-Type: {}\r\nContent-Length: {}\r\n",
            self.status,
            reason(self.status),
            self.content_type,
            self.body.len()
        );
        for (name, value) in SECURITY_HEADERS {
            head.push_str(name);
            head.push_str(": ");
            head.push_str(value);
            head.push_str("\r\n");
        }
        for (name, value) in &self.headers {
            if value.contains(['\r', '\n']) {
                continue;
            }
            head.push_str(name);
            head.push_str(": ");
            head.push_str(value);
            head.push_str("\r\n");
        }
        head.push_str("Connection: close\r\n\r\n");
        writer.write_all(head.as_bytes())?;
        if include_body {
            writer.write_all(&self.body)?;
        }
        writer.flush()
    }
}

/// Headers sent with every response. The content security policy admits only
/// Tabula's own embedded scripts and styles, so documents and notes rendered
/// in the page cannot run script even if a renderer bug let markup through.
pub const SECURITY_HEADERS: [(&str, &str); 8] = [
    (
        "Content-Security-Policy",
        "default-src 'none'; script-src 'self'; style-src 'self'; img-src 'self' data: blob:; \
         connect-src 'self'; font-src 'self'; base-uri 'none'; form-action 'none'; \
         frame-ancestors 'none'",
    ),
    ("X-Content-Type-Options", "nosniff"),
    ("Referrer-Policy", "no-referrer"),
    ("Cache-Control", "no-store"),
    ("Cross-Origin-Opener-Policy", "same-origin"),
    ("Cross-Origin-Resource-Policy", "same-origin"),
    ("X-Frame-Options", "DENY"),
    (
        "Permissions-Policy",
        "camera=(), microphone=(), geolocation=(), usb=(), serial=()",
    ),
];

const fn reason(status: u16) -> &'static str {
    match status {
        200 => "OK",
        201 => "Created",
        204 => "No Content",
        400 => "Bad Request",
        401 => "Unauthorized",
        403 => "Forbidden",
        404 => "Not Found",
        405 => "Method Not Allowed",
        409 => "Conflict",
        413 => "Content Too Large",
        415 => "Unsupported Media Type",
        421 => "Misdirected Request",
        422 => "Unprocessable Content",
        500 => "Internal Server Error",
        501 => "Not Implemented",
        503 => "Service Unavailable",
        504 => "Gateway Timeout",
        _ => "Status",
    }
}

#[cfg(test)]
mod tests {
    use super::{HttpError, MAX_HEAD_BYTES, Method, Response, percent_decode, read_request};
    use std::io::Cursor;

    fn parse(raw: &[u8]) -> Result<super::Request, HttpError> {
        read_request(&mut Cursor::new(raw.to_vec()))
    }

    #[test]
    fn parses_get_with_query_and_headers() {
        let request = parse(
            b"GET /api/file?path=src%2Fdemo.or&x=a+b HTTP/1.1\r\nHost: 127.0.0.1:2026\r\nX-Tabula-Token:  abc \r\n\r\n",
        )
        .unwrap();
        assert_eq!(request.method, Method::Get);
        assert_eq!(request.path, "/api/file");
        assert_eq!(request.param("path"), Some("src/demo.or"));
        assert_eq!(request.param("x"), Some("a b"));
        assert_eq!(request.header("host"), Some("127.0.0.1:2026"));
        assert_eq!(request.header("x-tabula-token"), Some("abc"));
        assert!(request.body.is_empty());
    }

    #[test]
    fn parses_post_body_exactly() {
        let request =
            parse(b"POST /api/check HTTP/1.1\r\nContent-Length: 5\r\n\r\nhelloEXTRA").unwrap();
        assert_eq!(request.method, Method::Post);
        assert_eq!(request.body, b"hello");
    }

    #[test]
    fn rejects_truncated_body() {
        assert!(matches!(
            parse(b"POST / HTTP/1.1\r\nContent-Length: 10\r\n\r\nshort"),
            Err(HttpError::Closed)
        ));
    }

    #[test]
    fn rejects_outside_subset() {
        assert!(matches!(
            parse(b"PUT / HTTP/1.1\r\n\r\n"),
            Err(HttpError::MethodNotAllowed)
        ));
        assert!(matches!(
            parse(b"GET http://evil/ HTTP/1.1\r\n\r\n"),
            Err(HttpError::BadRequest(_))
        ));
        assert!(matches!(
            parse(b"POST / HTTP/1.1\r\nTransfer-Encoding: chunked\r\n\r\n"),
            Err(HttpError::NotImplemented(_))
        ));
        assert!(matches!(
            parse(b"POST / HTTP/1.1\r\nContent-Length: 1\r\nContent-Length: 1\r\n\r\na"),
            Err(HttpError::BadRequest(_))
        ));
        assert!(matches!(
            parse(b"POST / HTTP/1.1\r\nContent-Length: -1\r\n\r\n"),
            Err(HttpError::BadRequest(_))
        ));
        assert!(matches!(
            parse(b"GET / HTTP/1.1\r\nX: a\r\n folded\r\n\r\n"),
            Err(HttpError::BadRequest(_))
        ));
        assert!(matches!(
            parse(b"GET / HTTP/2\r\n\r\n"),
            Err(HttpError::BadRequest(_))
        ));
        assert!(matches!(
            parse(b"GET /%zz HTTP/1.1\r\n\r\n"),
            Err(HttpError::BadRequest(_))
        ));
        assert!(matches!(
            parse(b"POST / HTTP/1.1\r\nContent-Length: 999999999999\r\n\r\n"),
            Err(HttpError::TooLarge)
        ));
    }

    #[test]
    fn bounds_the_head() {
        let mut raw = b"GET / HTTP/1.1\r\nX: ".to_vec();
        raw.extend(std::iter::repeat_n(b'a', MAX_HEAD_BYTES));
        raw.extend(b"\r\n\r\n");
        assert!(matches!(parse(&raw), Err(HttpError::TooLarge)));
        assert!(matches!(
            parse(b"GET / HTTP/1.1\r\n"),
            Err(HttpError::Closed)
        ));
    }

    #[test]
    fn decodes_percent_escapes_strictly() {
        assert_eq!(percent_decode("a%20b%C3%A9").as_deref(), Some("a bé"));
        assert_eq!(percent_decode("%"), None);
        assert_eq!(percent_decode("%4"), None);
        assert_eq!(percent_decode("%ff"), None);
    }

    #[test]
    fn writes_security_headers_and_skips_injected_headers() {
        let mut response = Response::new(200, "text/plain", b"ok".to_vec());
        response.headers.push(("X-Test", "fine".to_owned()));
        response
            .headers
            .push(("X-Bad", "a\r\nSet-Cookie: x".to_owned()));
        let mut out = Vec::new();
        response.write_to(&mut out, true).unwrap();
        let text = String::from_utf8(out).unwrap();
        assert!(text.starts_with("HTTP/1.1 200 OK\r\n"));
        assert!(text.contains("Content-Length: 2\r\n"));
        assert!(text.contains("Content-Security-Policy: default-src 'none'"));
        assert!(text.contains("X-Test: fine\r\n"));
        assert!(!text.contains("Set-Cookie"));
        assert!(text.ends_with("\r\n\r\nok"));

        let mut head_only = Vec::new();
        response.write_to(&mut head_only, false).unwrap();
        assert!(String::from_utf8(head_only).unwrap().ends_with("\r\n\r\n"));
    }
}
