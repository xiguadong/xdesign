//! Minimal HTTP/1.1 request/response layer for the embedded operator server.
//!
//! The server is a local operator surface, not a general web framework: one
//! request per connection, `GET` and `POST` only, explicit `Content-Length`,
//! a 1 MiB request-body limit, and a 10 s read deadline. Everything is
//! standard library plus `serde_json`.

use serde_json::Value;
use std::io::{Read, Write};
use std::time::Duration;

/// Largest accepted request body. Larger declared lengths are rejected before
/// any body byte is read.
pub const MAX_BODY_BYTES: usize = 1024 * 1024;
/// Deadline for reading one request (request line, headers, and body).
pub const READ_TIMEOUT: Duration = Duration::from_secs(10);
/// Upper bound for the request line plus headers.
const MAX_HEADER_BYTES: usize = 64 * 1024;

const REASON_OK: &str = "OK";
const REASON_BAD_REQUEST: &str = "Bad Request";
const REASON_NOT_FOUND: &str = "Not Found";
const REASON_METHOD_NOT_ALLOWED: &str = "Method Not Allowed";
const REASON_CONFLICT: &str = "Conflict";
const REASON_PAYLOAD_TOO_LARGE: &str = "Payload Too Large";
const REASON_INTERNAL_ERROR: &str = "Internal Server Error";

const JSON_CONTENT_TYPE: &str = "application/json; charset=utf-8";
const TEXT_CONTENT_TYPE: &str = "text/plain; charset=utf-8";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Method {
    Get,
    Post,
}

#[derive(Clone, Debug)]
pub struct Request {
    pub method: Method,
    /// Decoded request path, without the query string. Always starts with `/`.
    pub path: String,
    /// Decoded query parameters in wire order.
    pub query: Vec<(String, String)>,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

impl Request {
    pub fn get(path: impl Into<String>) -> Self {
        Self {
            method: Method::Get,
            path: path.into(),
            query: Vec::new(),
            headers: Vec::new(),
            body: Vec::new(),
        }
    }

    pub fn post(path: impl Into<String>, body: impl Into<Vec<u8>>) -> Self {
        Self {
            method: Method::Post,
            path: path.into(),
            query: Vec::new(),
            headers: vec![(
                "content-type".into(),
                "application/json; charset=utf-8".into(),
            )],
            body: body.into(),
        }
    }

    pub fn with_query(mut self, query: &str) -> Self {
        self.query = parse_query(query);
        self
    }

    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(key, _)| key.eq_ignore_ascii_case(name))
            .map(|(_, value)| value.as_str())
    }

    pub fn query_param(&self, name: &str) -> Option<&str> {
        self.query
            .iter()
            .find(|(key, _)| key == name)
            .map(|(_, value)| value.as_str())
    }

    pub fn body_json(&self) -> Result<Value, Response> {
        serde_json::from_slice(&self.body).map_err(|error| {
            json_error_with_detail(
                400,
                "invalid_json",
                "The request body is not valid JSON.",
                Value::String(error.to_string()),
            )
        })
    }
}

/// A complete HTTP response. Every response carries an explicit
/// `Content-Length`; connections are always closed after one response.
#[derive(Clone, Debug)]
pub struct Response {
    pub status: u16,
    pub content_type: &'static str,
    pub body: Vec<u8>,
    pub extra_headers: Vec<(String, String)>,
}

impl Response {
    pub fn new(status: u16, content_type: &'static str, body: Vec<u8>) -> Self {
        Self {
            status,
            content_type,
            body,
            extra_headers: Vec::new(),
        }
    }

    pub fn empty(status: u16) -> Self {
        Self::new(status, TEXT_CONTENT_TYPE, Vec::new())
    }

    fn reason(&self) -> &'static str {
        match self.status {
            200 => REASON_OK,
            400 => REASON_BAD_REQUEST,
            404 => REASON_NOT_FOUND,
            405 => REASON_METHOD_NOT_ALLOWED,
            409 => REASON_CONFLICT,
            413 => REASON_PAYLOAD_TOO_LARGE,
            _ => REASON_INTERNAL_ERROR,
        }
    }
}

/// A request that cannot be accepted. Carries the HTTP status, a stable error
/// code, and a human-readable message so the connection handler can turn it
/// into the same JSON failure envelope used by API handlers.
#[derive(Clone, Debug)]
pub struct ParseError {
    pub status: u16,
    pub code: &'static str,
    pub message: String,
}

impl ParseError {
    fn new(status: u16, code: &'static str, message: impl Into<String>) -> Self {
        Self {
            status,
            code,
            message: message.into(),
        }
    }

    pub fn into_response(self) -> Response {
        json_error(self.status, self.code, self.message)
    }
}

pub fn json_ok(data: Value) -> Response {
    json_response(200, serde_json::json!({"ok": true, "data": data}))
}

pub fn json_error(status: u16, code: &'static str, message: impl Into<String>) -> Response {
    json_response(
        status,
        serde_json::json!({
            "ok": false,
            "error": {"code": code, "message": message.into()},
        }),
    )
}

pub fn json_error_with_detail(
    status: u16,
    code: &'static str,
    message: impl Into<String>,
    detail: Value,
) -> Response {
    json_response(
        status,
        serde_json::json!({
            "ok": false,
            "error": {"code": code, "message": message.into(), "detail": detail},
        }),
    )
}

fn json_response(status: u16, value: Value) -> Response {
    // Every envelope member is serializable; fall back to a fixed body instead
    // of panicking if that ever stops being true.
    let bytes = serde_json::to_vec(&value).unwrap_or_else(|_| {
        b"{\"ok\":false,\"error\":{\"code\":\"internal_error\",\"message\":\"cannot encode response\"}}"
            .to_vec()
    });
    Response::new(status, JSON_CONTENT_TYPE, bytes)
}

pub fn raw(content_type: &'static str, body: Vec<u8>) -> Response {
    Response::new(200, content_type, body)
}

pub fn text_error(status: u16, message: impl Into<String>) -> Response {
    Response::new(status, TEXT_CONTENT_TYPE, message.into().into_bytes())
}

pub fn method_not_allowed() -> Response {
    json_error(
        405,
        "method_not_allowed",
        "This route accepts only GET and POST requests.",
    )
}

/// Content type for a material or asset path, selected by extension.
pub fn content_type_for_extension(path: &str) -> &'static str {
    match path
        .rsplit_once('.')
        .map(|(_, extension)| extension.to_ascii_lowercase())
        .as_deref()
    {
        Some("json") => "application/json; charset=utf-8",
        Some("md" | "txt" | "sysml") => "text/plain; charset=utf-8",
        Some("html") => "text/html; charset=utf-8",
        Some("js" | "mjs") => "text/javascript; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("svg") => "image/svg+xml",
        Some("png") => "image/png",
        Some("jpg" | "jpeg") => "image/jpeg",
        Some("webp") => "image/webp",
        Some("ico") => "image/x-icon",
        Some("woff2") => "font/woff2",
        Some("woff") => "font/woff",
        _ => "application/octet-stream",
    }
}

/// Read and parse one request from `reader`.
///
/// The caller is responsible for setting the socket read deadline; this
/// function only enforces the size limits.
pub fn read_request(reader: &mut dyn Read) -> Result<Request, ParseError> {
    let head = read_head(reader)?;
    let head = String::from_utf8(head)
        .map_err(|_| ParseError::new(400, "invalid_request", "The request head is not UTF-8."))?;
    let mut lines = head.split('\n').map(|line| line.trim_end_matches('\r'));
    let request_line = lines
        .next()
        .ok_or_else(|| ParseError::new(400, "invalid_request", "The request line is empty."))?;
    let mut parts = request_line.split(' ');
    let (Some(method_token), Some(target), Some(version), None) =
        (parts.next(), parts.next(), parts.next(), parts.next())
    else {
        return Err(ParseError::new(
            400,
            "invalid_request",
            "The request line must be `METHOD TARGET VERSION`.",
        ));
    };
    if !matches!(version, "HTTP/1.1" | "HTTP/1.0") {
        return Err(ParseError::new(
            400,
            "unsupported_http_version",
            "Only HTTP/1.0 and HTTP/1.1 are supported.",
        ));
    }
    let method = match method_token {
        "GET" => Method::Get,
        "POST" => Method::Post,
        token if token.is_empty() || !token.bytes().all(|byte| byte.is_ascii_uppercase()) => {
            return Err(ParseError::new(
                400,
                "invalid_request",
                "The request method is malformed.",
            ))
        }
        _ => return Err(method_not_allowed_parse_error()),
    };
    let (raw_path, raw_query) = match target.split_once('?') {
        Some((path, query)) => (path, Some(query)),
        None => (target, None),
    };
    if !raw_path.starts_with('/') {
        return Err(ParseError::new(
            400,
            "invalid_request",
            "The request target must start with `/`.",
        ));
    }
    let path = percent_decode(raw_path).ok_or_else(|| {
        ParseError::new(400, "invalid_request", "The request path has a bad escape.")
    })?;
    let query = match raw_query {
        Some(query) => parse_query_checked(query)?,
        None => Vec::new(),
    };
    let mut headers = Vec::new();
    let mut content_length: Option<usize> = None;
    let mut transfer_encoding = false;
    for line in lines {
        if line.is_empty() {
            continue;
        }
        let Some((name, value)) = line.split_once(':') else {
            return Err(ParseError::new(
                400,
                "invalid_request",
                "A request header is malformed.",
            ));
        };
        let name = name.trim().to_ascii_lowercase();
        let value = value.trim().to_owned();
        if name.is_empty() {
            return Err(ParseError::new(
                400,
                "invalid_request",
                "A request header has an empty name.",
            ));
        }
        match name.as_str() {
            "content-length" => {
                if content_length.is_some() {
                    return Err(ParseError::new(
                        400,
                        "invalid_request",
                        "Multiple Content-Length headers are not supported.",
                    ));
                }
                let length = value.parse::<u64>().map_err(|_| {
                    ParseError::new(400, "invalid_request", "Content-Length is not an integer.")
                })?;
                let length = usize::try_from(length).map_err(|_| {
                    ParseError::new(413, "payload_too_large", "The request body is too large.")
                })?;
                content_length = Some(length);
            }
            "transfer-encoding" => transfer_encoding = true,
            _ => {}
        }
        headers.push((name, value));
    }
    if transfer_encoding {
        return Err(ParseError::new(
            400,
            "invalid_request",
            "Transfer-Encoding is not supported; send a Content-Length body.",
        ));
    }
    let length = content_length.unwrap_or(0);
    if length > MAX_BODY_BYTES {
        return Err(ParseError::new(
            413,
            "payload_too_large",
            format!("The request body exceeds the {MAX_BODY_BYTES}-byte limit."),
        ));
    }
    let mut body = vec![0u8; length];
    if length > 0 {
        reader.read_exact(&mut body).map_err(|error| {
            ParseError::new(
                400,
                "invalid_request",
                format!("The request body could not be read: {error}"),
            )
        })?;
    }
    Ok(Request {
        method,
        path,
        query,
        headers,
        body,
    })
}

fn method_not_allowed_parse_error() -> ParseError {
    ParseError::new(
        405,
        "method_not_allowed",
        "Only GET and POST are supported.",
    )
}

/// Read the request line and headers up to (and including) the blank line.
fn read_head(reader: &mut dyn Read) -> Result<Vec<u8>, ParseError> {
    let mut buffer = Vec::with_capacity(1024);
    let mut byte = [0u8; 1];
    loop {
        match reader.read(&mut byte) {
            Ok(0) => {
                if buffer.is_empty() {
                    return Err(ParseError::new(
                        400,
                        "invalid_request",
                        "The connection closed before a request was sent.",
                    ));
                }
                return Err(ParseError::new(
                    400,
                    "invalid_request",
                    "The request head ended unexpectedly.",
                ));
            }
            Ok(_) => {
                buffer.push(byte[0]);
                if buffer.ends_with(b"\r\n\r\n") || buffer.ends_with(b"\n\n") {
                    let content = buffer;
                    let head_length = if content.ends_with(b"\r\n\r\n") {
                        content.len() - 4
                    } else {
                        content.len() - 2
                    };
                    let mut head = content;
                    head.truncate(head_length);
                    return Ok(head);
                }
                if buffer.len() > MAX_HEADER_BYTES {
                    return Err(ParseError::new(
                        400,
                        "request_too_large",
                        "The request head exceeds the header size limit.",
                    ));
                }
            }
            Err(error) => {
                return Err(ParseError::new(
                    400,
                    "invalid_request",
                    format!("The request head could not be read: {error}"),
                ))
            }
        }
    }
}

/// Serialize a response on the wire with an explicit length and `close`
/// connection handling.
pub fn write_response(writer: &mut dyn Write, response: &Response) -> std::io::Result<()> {
    let mut head = format!(
        "HTTP/1.1 {} {}\r\n\
         Content-Type: {}\r\n\
         Content-Length: {}\r\n\
         Connection: close\r\n\
         Cache-Control: no-store\r\n\
         X-Content-Type-Options: nosniff\r\n\
         X-Frame-Options: DENY\r\n\
         Referrer-Policy: no-referrer\r\n",
        response.status,
        response.reason(),
        response.content_type,
        response.body.len(),
    );
    for (name, value) in &response.extra_headers {
        head.push_str(name);
        head.push_str(": ");
        head.push_str(value);
        head.push_str("\r\n");
    }
    head.push_str("\r\n");
    writer.write_all(head.as_bytes())?;
    writer.write_all(&response.body)?;
    writer.flush()
}

/// Decode `%XX` escapes in a path or query component. `+` stays literal:
/// material URIs are paths, not form data.
pub fn percent_decode(input: &str) -> Option<String> {
    let bytes = input.as_bytes();
    let mut output = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        match bytes[index] {
            b'%' => {
                let high = *bytes.get(index + 1)?;
                let low = *bytes.get(index + 2)?;
                output.push(hex_value(high)? << 4 | hex_value(low)?);
                index += 3;
            }
            byte => {
                output.push(byte);
                index += 1;
            }
        }
    }
    String::from_utf8(output).ok()
}

fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

/// Decode a `key=value&...` query string. Invalid escapes are dropped from the
/// returned pairs; [`read_request`] validates the string before calling this.
pub fn parse_query(query: &str) -> Vec<(String, String)> {
    parse_query_checked(query).unwrap_or_default()
}

fn parse_query_checked(query: &str) -> Result<Vec<(String, String)>, ParseError> {
    if query.is_empty() {
        return Ok(Vec::new());
    }
    let mut pairs = Vec::new();
    for part in query.split('&') {
        if part.is_empty() {
            continue;
        }
        let (key, value) = part.split_once('=').unwrap_or((part, ""));
        let key = percent_decode(key).ok_or_else(|| {
            ParseError::new(
                400,
                "invalid_request",
                "A query parameter has a bad escape.",
            )
        })?;
        let value = percent_decode(value).ok_or_else(|| {
            ParseError::new(
                400,
                "invalid_request",
                "A query parameter has a bad escape.",
            )
        })?;
        pairs.push((key, value));
    }
    Ok(pairs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn parse(raw: &[u8]) -> Result<Request, ParseError> {
        let mut cursor = Cursor::new(raw.to_vec());
        read_request(&mut cursor)
    }

    #[test]
    fn parses_get_with_query() {
        let request = parse(
            b"GET /api/wishes/wish-1/material?uri=cache%2Fwishes%2Fwish-1.sysml HTTP/1.1\r\n\
              Host: 127.0.0.1:7788\r\n\r\n",
        )
        .unwrap();
        assert_eq!(request.method, Method::Get);
        assert_eq!(request.path, "/api/wishes/wish-1/material");
        assert_eq!(
            request.query_param("uri"),
            Some("cache/wishes/wish-1.sysml")
        );
        assert_eq!(request.header("HOST"), Some("127.0.0.1:7788"));
        assert!(request.body.is_empty());
    }

    #[test]
    fn parses_post_body_and_content_type() {
        let request = parse(
            b"POST /api/wishes HTTP/1.1\r\nContent-Length: 7\r\nContent-Type: application/json\r\n\r\n{\"a\":1}",
        )
        .unwrap();
        assert_eq!(request.method, Method::Post);
        assert_eq!(request.body, b"{\"a\":1}");
        assert_eq!(request.header("content-type"), Some("application/json"));
    }

    #[test]
    fn rejects_unknown_method_with_405() {
        let error = parse(b"PUT /api/health HTTP/1.1\r\n\r\n").unwrap_err();
        assert_eq!(error.status, 405);
        assert_eq!(error.code, "method_not_allowed");
    }

    #[test]
    fn rejects_malformed_request_line_with_400() {
        assert_eq!(parse(b"GET /api/health\r\n\r\n").unwrap_err().status, 400);
        assert_eq!(
            parse(b"GET api/health HTTP/1.1\r\n\r\n")
                .unwrap_err()
                .status,
            400
        );
        assert_eq!(
            parse(b"get /api/health HTTP/1.1\r\n\r\n")
                .unwrap_err()
                .status,
            400
        );
    }

    #[test]
    fn rejects_oversized_body_before_reading_it() {
        let error =
            parse(b"POST /api/wishes HTTP/1.1\r\nContent-Length: 1048577\r\n\r\n").unwrap_err();
        assert_eq!(error.status, 413);
        assert_eq!(error.code, "payload_too_large");
    }

    #[test]
    fn rejects_short_body_with_400() {
        let error =
            parse(b"POST /api/wishes HTTP/1.1\r\nContent-Length: 10\r\n\r\nshort").unwrap_err();
        assert_eq!(error.status, 400);
    }

    #[test]
    fn rejects_transfer_encoding_with_400() {
        let error =
            parse(b"POST /api/wishes HTTP/1.1\r\nTransfer-Encoding: chunked\r\n\r\n").unwrap_err();
        assert_eq!(error.status, 400);
    }

    #[test]
    fn rejects_bad_escapes() {
        let error = parse(b"GET /api/%zz HTTP/1.1\r\n\r\n").unwrap_err();
        assert_eq!(error.status, 400);
        let error = parse(b"GET /api/health?uri=%zz HTTP/1.1\r\n\r\n").unwrap_err();
        assert_eq!(error.status, 400);
    }

    #[test]
    fn parses_bare_lf_request_termination() {
        let request = parse(b"GET /api/health HTTP/1.1\nHost: x\n\n").unwrap();
        assert_eq!(request.path, "/api/health");
    }

    #[test]
    fn writes_explicit_content_length_and_security_headers() {
        let response = json_ok(serde_json::json!({"hello": "world"}));
        let mut output = Vec::new();
        write_response(&mut output, &response).unwrap();
        let text = String::from_utf8(output).unwrap();
        assert!(text.starts_with("HTTP/1.1 200 OK\r\n"), "{text}");
        assert!(text.contains(&format!("Content-Length: {}\r\n", response.body.len())));
        assert!(text.contains("Content-Type: application/json; charset=utf-8\r\n"));
        assert!(text.contains("Connection: close\r\n"));
        assert!(text.contains("Cache-Control: no-store\r\n"));
        assert!(text.contains("X-Content-Type-Options: nosniff\r\n"));
        let (head, body) = text.split_once("\r\n\r\n").unwrap();
        assert_eq!(body.as_bytes(), response.body.as_slice());
        assert_eq!(
            head.rsplit("Content-Length: ")
                .next()
                .unwrap()
                .lines()
                .next()
                .unwrap(),
            body.len().to_string()
        );
    }

    #[test]
    fn envelopes_use_the_documented_shape() {
        let ok = json_ok(serde_json::json!({"value": 1}));
        let value: Value = serde_json::from_slice(&ok.body).unwrap();
        assert_eq!(value, serde_json::json!({"ok": true, "data": {"value": 1}}));

        let failure = json_error(404, "unknown_wish", "No such wish.");
        let value: Value = serde_json::from_slice(&failure.body).unwrap();
        assert_eq!(
            value,
            serde_json::json!({
                "ok": false,
                "error": {"code": "unknown_wish", "message": "No such wish."}
            })
        );

        let detailed = json_error_with_detail(
            500,
            "check_failed",
            "boom",
            serde_json::json!({"exit_code": 2}),
        );
        let value: Value = serde_json::from_slice(&detailed.body).unwrap();
        assert_eq!(value["error"]["detail"]["exit_code"], 2);
    }

    #[test]
    fn parse_error_maps_to_the_same_envelope() {
        let response = ParseError::new(413, "payload_too_large", "too big").into_response();
        assert_eq!(response.status, 413);
        let value: Value = serde_json::from_slice(&response.body).unwrap();
        assert_eq!(value["ok"], false);
        assert_eq!(value["error"]["code"], "payload_too_large");
    }

    #[test]
    fn content_types_follow_the_material_rules() {
        assert_eq!(
            content_type_for_extension("cache/x.json"),
            "application/json; charset=utf-8"
        );
        assert_eq!(
            content_type_for_extension("docs/readme.md"),
            "text/plain; charset=utf-8"
        );
        assert_eq!(
            content_type_for_extension("cache/wishes/wish-1.sysml"),
            "text/plain; charset=utf-8"
        );
        assert_eq!(content_type_for_extension("assets/logo.png"), "image/png");
        assert_eq!(
            content_type_for_extension("cache/evidence/noext"),
            "application/octet-stream"
        );
    }
}
