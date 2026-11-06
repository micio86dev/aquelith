//! MCP server (M8): JSON-RPC 2.0, protocol 2024-11-05. Over stdio (one message per line) or, on
//! loopback only, over HTTP (`serve_http`).
//!
//! Every tool works on source text passed as an argument. None of them reads or writes a file,
//! so an agent cannot change the disk through this server. A patch is returned as a candidate
//! text; the caller decides whether to write it, for example with `aqelith patch`.

use crate::diag::{self, jstr};
use crate::json::{self, Json};
use crate::{check_source, format_source, parser, patch, query};

const PROTOCOL: &str = "2024-11-05";

const TOOLS: &str = r#"[
{"name":"aqelith_check","description":"Parse and check AQELITH v0 source text. Returns ok and the diagnostics with stable codes.","inputSchema":{"type":"object","properties":{"source":{"type":"string"}},"required":["source"]}},
{"name":"aqelith_format","description":"Return the canonical form of AQELITH v0 source text. Syntax only; the result is an error when the text does not parse.","inputSchema":{"type":"object","properties":{"source":{"type":"string"}},"required":["source"]}},
{"name":"aqelith_query","description":"Callers, callees and effects of one function in AQELITH v0 source text, as JSON.","inputSchema":{"type":"object","properties":{"source":{"type":"string"},"function":{"type":"string"}},"required":["source","function"]}},
{"name":"aqelith_explain","description":"Meaning of a stable AQELITH diagnostic code.","inputSchema":{"type":"object","properties":{"code":{"type":"string"}},"required":["code"]}},
{"name":"aqelith_patch","description":"Apply a semantic patch (aqelith.patch.v1) to source text and return the verified candidate. Never writes a file. A rejected patch is returned as an error with its code.","inputSchema":{"type":"object","properties":{"source":{"type":"string"},"patch":{"type":"string","description":"The patch document as JSON text"}},"required":["source","patch"]}}
]"#;

/// Handle one line of input. Returns the reply line, or None for a notification.
pub fn handle_line(line: &str) -> Option<String> {
    let msg = match json::parse(line) {
        Ok(m) => m,
        Err(e) => return Some(error_reply(&Json::Null, -32700, &format!("parse error: {e}"))),
    };
    if !matches!(msg, Json::Obj(_)) {
        return Some(error_reply(&Json::Null, -32600, "batches are not supported"));
    }
    // A message without an id is a notification and gets no reply.
    let id = msg.get("id")?.clone();
    let Some(method) = msg.get("method").and_then(Json::as_str) else {
        return Some(error_reply(&id, -32600, "missing method"));
    };
    Some(match method {
        "initialize" => ok_reply(&id, &format!(
            "{{\"protocolVersion\":{},\"capabilities\":{{\"tools\":{{}}}},\"serverInfo\":{{\"name\":\"aqelith\",\"version\":\"0.0.1\"}}}}",
            jstr(PROTOCOL)
        )),
        "ping" => ok_reply(&id, "{}"),
        "tools/list" => ok_reply(&id, &format!("{{\"tools\":{TOOLS}}}")),
        "tools/call" => call(&id, msg.get("params")),
        _ => error_reply(&id, -32601, &format!("method not found: {method}")),
    })
}

fn call(id: &Json, params: Option<&Json>) -> String {
    let Some(params) = params else {
        return error_reply(id, -32602, "params are required");
    };
    let Some(name) = params.get("name").and_then(Json::as_str) else {
        return error_reply(id, -32602, "tool name is required");
    };
    let empty = Json::Obj(Vec::new());
    let args = params.get("arguments").unwrap_or(&empty);
    let arg = |key: &str| args.get(key).and_then(Json::as_str);
    let result = match name {
        "aqelith_check" => arg("source").map(|src| {
            let diags = check_source(src);
            let items: Vec<String> = diags.iter().map(|d| d.to_json()).collect();
            content(&format!("{{\"ok\":{},\"diagnostics\":[{}]}}", diags.is_empty(), items.join(",")), false)
        }),
        "aqelith_format" => arg("source").map(|src| match format_source(src) {
            Ok(text) => content(&text, false),
            Err(d) => content(&d.to_json(), true),
        }),
        "aqelith_query" => match (arg("source"), arg("function")) {
            (Some(src), Some(func)) => Some(match parser::parse(src) {
                Err(d) => content(&d.to_json(), true),
                Ok(m) => match query::query_fn(&m, func) {
                    Some(json) => content(&json, false),
                    None => content(&format!("no function '{func}'"), true),
                },
            }),
            _ => None,
        },
        "aqelith_explain" => arg("code").map(|code| match diag::explain(code) {
            Some(text) => content(&format!("{{\"code\":{},\"explanation\":{}}}", jstr(code), jstr(text)), false),
            None => content(&format!("unknown code '{code}'"), true),
        }),
        "aqelith_patch" => match (arg("source"), arg("patch")) {
            (Some(src), Some(doc)) => Some(match patch::apply(src, doc) {
                Err(r) => content(&r.to_json(), true),
                Ok(acc) => content(
                    &format!(
                        "{{\"api\":{},\"status\":\"candidate\",\"base\":{},\"new_base\":{},\"touched\":[{}],\"candidate\":{}}}",
                        jstr(patch::API),
                        jstr(&acc.base),
                        jstr(&acc.new_base),
                        acc.touched.iter().map(|t| jstr(t)).collect::<Vec<_>>().join(","),
                        jstr(&acc.text)
                    ),
                    false,
                ),
            }),
            _ => None,
        },
        _ => return error_reply(id, -32602, &format!("unknown tool: {name}")),
    };
    match result {
        Some(text) => ok_reply(id, &text),
        None => error_reply(id, -32602, "missing or non-string argument"),
    }
}

/// Tool result envelope. `text` is itself JSON or plain text; it is sent as a string.
fn content(text: &str, is_error: bool) -> String {
    format!("{{\"content\":[{{\"type\":\"text\",\"text\":{}}}],\"isError\":{is_error}}}", jstr(text))
}

fn id_json(id: &Json) -> String {
    match id {
        Json::Int(n) => n.to_string(),
        Json::Str(s) => jstr(s),
        _ => "null".to_string(),
    }
}

fn ok_reply(id: &Json, result: &str) -> String {
    format!("{{\"jsonrpc\":\"2.0\",\"id\":{},\"result\":{result}}}", id_json(id))
}

fn error_reply(id: &Json, code: i64, message: &str) -> String {
    format!("{{\"jsonrpc\":\"2.0\",\"id\":{},\"error\":{{\"code\":{code},\"message\":{}}}}}", id_json(id), jstr(message))
}

/// Largest request body accepted over HTTP (1 MiB).
const MAX_BODY: usize = 1 << 20;

/// Serves MCP over HTTP: `POST /mcp` with one JSON-RPC message per request. Answers 200 with the
/// reply, or 202 with no body for a notification. Only loopback peers are meant to reach it: the
/// `Host` header must name localhost, which blocks DNS rebinding from a web page. `max_requests`
/// stops the loop after that many connections (for tests); None serves until an error.
pub fn serve_http(listener: &std::net::TcpListener, max_requests: Option<usize>) -> std::io::Result<()> {
    for (served, stream) in listener.incoming().enumerate() {
        let mut stream = stream?;
        // A broken client must not stop the server: its I/O error ends only its connection.
        let _ = handle_connection(&mut stream);
        if max_requests.is_some_and(|max| served + 1 >= max) {
            break;
        }
    }
    Ok(())
}

/// Limits that keep one slow or hostile local client from holding the server.
const READ_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(5);
const MAX_HEADER_LINES: usize = 64;

fn handle_connection(stream: &mut std::net::TcpStream) -> std::io::Result<()> {
    use std::io::{BufRead, BufReader, Read, Write};
    stream.set_read_timeout(Some(READ_TIMEOUT))?;
    let mut reader = BufReader::new(stream.try_clone()?);
    let mut request_line = String::new();
    reader.read_line(&mut request_line)?;
    let mut parts = request_line.split_whitespace();
    let (method, path) = (parts.next().unwrap_or(""), parts.next().unwrap_or(""));

    let mut host: Option<String> = None;
    let mut length: Option<usize> = None;
    for _ in 0..MAX_HEADER_LINES {
        let mut line = String::new();
        if reader.read_line(&mut line)? == 0 || line == "\r\n" || line == "\n" {
            break;
        }
        if let Some((name, value)) = line.split_once(':') {
            let value = value.trim();
            if name.eq_ignore_ascii_case("host") {
                host = Some(value.to_string());
            } else if name.eq_ignore_ascii_case("content-length") {
                length = value.parse().ok();
            }
        }
    }

    let respond = |stream: &mut std::net::TcpStream, status: &str, body: &str| -> std::io::Result<()> {
        let type_line = if body.is_empty() { "" } else { "Content-Type: application/json\r\n" };
        write!(
            stream,
            "HTTP/1.1 {status}\r\n{type_line}Content-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        )?;
        stream.flush()
    };

    if !host.as_deref().is_some_and(is_local_host) {
        return respond(stream, "403 Forbidden", "");
    }
    if method != "POST" || path != "/mcp" {
        return respond(stream, "404 Not Found", "");
    }
    let Some(length) = length.filter(|n| *n <= MAX_BODY) else {
        return respond(stream, "413 Payload Too Large", "");
    };
    let mut body = vec![0u8; length];
    reader.read_exact(&mut body)?;
    let Ok(text) = String::from_utf8(body) else {
        return respond(stream, "400 Bad Request", "");
    };
    match handle_line(text.trim()) {
        Some(reply) => respond(stream, "200 OK", &reply),
        None => respond(stream, "202 Accepted", ""),
    }
}

/// Host header values that name this machine: localhost, 127.0.0.1 or [::1], with an optional port.
pub fn is_local_host(host: &str) -> bool {
    let name = match host.strip_prefix('[') {
        Some(rest) => rest.split_once(']').map_or("", |(ip, _)| ip),
        None => host.split(':').next().unwrap_or(""),
    };
    matches!(name, "localhost" | "127.0.0.1" | "::1")
}
