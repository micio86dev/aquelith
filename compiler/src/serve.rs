//! Loopback HTTP host for a v0 handler (milestone S3, see docs/architecture/SERVE.md).
//!
//! The program provides `fn handle(path: str) -> str` with no declared effects. The host, not the
//! language, owns the socket: it accepts connections on 127.0.0.1 only, runs the handler for each
//! `GET`, and sends the returned text as the body. The `net` effect therefore stays refused inside
//! the language (`interp`), and the host is the only place that opens a socket.
//!
//! Limits: request head at most 8 KiB and 5 seconds; `GET` only; Host must be the loopback name
//! and port; one connection at a time; a handler that fails answers 500 with no detail.

use std::io::{ErrorKind, Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::time::{Duration, Instant};

use crate::interp::{run, Val};
use crate::ir::Program;

pub const MAX_HEAD: usize = 8 * 1024;
const HEAD_DEADLINE: Duration = Duration::from_secs(5);
const IO_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Debug)]
pub enum ServeError {
    /// The program has no `handle` function of the right shape, or it declares an effect.
    BadHandler(String),
    /// The address cannot be bound.
    Bind(String),
}

/// Checks the handler, binds the loopback port, and serves on a background thread. Returns the
/// bound address (use port 0 to pick a free port).
pub fn start(program: Program, port: u16) -> Result<SocketAddr, ServeError> {
    let handler = program
        .funcs
        .iter()
        .find(|f| f.name == "handle")
        .ok_or_else(|| ServeError::BadHandler("no function 'handle'".into()))?;
    if handler.params != [crate::ast::Ty::Str] || handler.ret != crate::ast::Ty::Str {
        return Err(ServeError::BadHandler(
            "'handle' must take one str and return str".into(),
        ));
    }
    if let Some(effect) = handler.uses.first() {
        return Err(ServeError::BadHandler(format!(
            "'handle' declares effect '{effect}'; a handler runs without effects"
        )));
    }
    let listener = TcpListener::bind(("127.0.0.1", port))
        .map_err(|e| ServeError::Bind(format!("127.0.0.1:{port}: {e}")))?;
    let addr = listener
        .local_addr()
        .map_err(|e| ServeError::Bind(e.to_string()))?;
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            match stream {
                // A panic in one request must not stop the accept loop: that connection is dropped
                // and the server keeps serving.
                Ok(stream) => {
                    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        serve_one(stream, &program, addr.port())
                    }));
                }
                Err(_) => std::thread::sleep(Duration::from_millis(50)),
            }
        }
    });
    Ok(addr)
}

fn serve_one(mut stream: TcpStream, program: &Program, port: u16) {
    let _ = stream.set_read_timeout(Some(IO_TIMEOUT));
    let _ = stream.set_write_timeout(Some(IO_TIMEOUT));
    let head = match read_head(&mut stream) {
        Ok(head) => head,
        Err(status) => {
            respond(&mut stream, status, b"");
            return linger_close(stream);
        }
    };
    let Some(req) = parse_head(&head) else {
        return respond(&mut stream, "400 Bad Request", b"");
    };
    if req.method != "GET" {
        return respond(&mut stream, "405 Method Not Allowed", b"");
    }
    if req.host.as_deref() != Some(&format!("127.0.0.1:{port}"))
        && req.host.as_deref() != Some(&format!("localhost:{port}"))
    {
        return respond(&mut stream, "403 Forbidden", b"");
    }
    let path = req.target.split('?').next().unwrap_or("/").to_string();
    match run(program, "handle", &[Val::Str(path)]) {
        Ok(Val::Str(body)) => respond(&mut stream, "200 OK", body.as_bytes()),
        Ok(_) | Err(_) => {
            // The detail stays on the host; the client gets no information about the program.
            eprintln!("handler failed for a request");
            respond(&mut stream, "500 Internal Server Error", b"");
        }
    }
}

/// Closes after an error answer. Closing with unread request bytes makes the kernel send a reset,
/// which can discard the answer the client has not read yet; so stop sending, then drain briefly.
fn linger_close(mut stream: TcpStream) {
    let _ = stream.shutdown(std::net::Shutdown::Write);
    let _ = stream.set_read_timeout(Some(Duration::from_millis(200)));
    let mut sink = [0u8; 4096];
    let mut total = 0usize;
    while total < 64 * 1024 {
        match stream.read(&mut sink) {
            Ok(0) | Err(_) => break,
            Ok(n) => total += n,
        }
    }
}

fn respond(stream: &mut TcpStream, status: &str, body: &[u8]) {
    let head = format!(
        "HTTP/1.1 {status}\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Length: {}\r\nContent-Security-Policy: default-src 'none'; frame-ancestors 'none'\r\nX-Content-Type-Options: nosniff\r\nCache-Control: no-store\r\nConnection: close\r\n\r\n",
        body.len()
    );
    let _ = stream.write_all(head.as_bytes());
    let _ = stream.write_all(body);
}

fn read_head(stream: &mut TcpStream) -> Result<String, &'static str> {
    let deadline = Instant::now() + HEAD_DEADLINE;
    let mut buf: Vec<u8> = Vec::with_capacity(512);
    loop {
        if let Some(end) = buf.windows(4).position(|w| w == b"\r\n\r\n") {
            buf.truncate(end + 4);
            return String::from_utf8(buf).map_err(|_| "400 Bad Request");
        }
        let room = MAX_HEAD.saturating_sub(buf.len());
        if room == 0 {
            return Err("431 Request Header Fields Too Large");
        }
        let left = match deadline.checked_duration_since(Instant::now()) {
            Some(left) if !left.is_zero() => left,
            _ => return Err("408 Request Timeout"),
        };
        let _ = stream.set_read_timeout(Some(left));
        let mut chunk = vec![0u8; room.min(1024)];
        match stream.read(&mut chunk) {
            Ok(0) => return Err("400 Bad Request"),
            Ok(n) => buf.extend_from_slice(&chunk[..n]),
            Err(e) if matches!(e.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut) => {
                return Err("408 Request Timeout")
            }
            Err(_) => return Err("400 Bad Request"),
        }
    }
}

struct Req {
    method: String,
    target: String,
    host: Option<String>,
}

fn parse_head(head: &str) -> Option<Req> {
    let mut lines = head.split("\r\n");
    let mut parts = lines.next()?.split(' ');
    let method = parts.next()?.to_string();
    let target = parts.next()?.to_string();
    let version = parts.next()?;
    if parts.next().is_some() || !version.starts_with("HTTP/1.") || !target.starts_with('/') {
        return None;
    }
    let mut hosts = Vec::new();
    for line in lines {
        if line.is_empty() {
            break;
        }
        let (name, value) = line.split_once(':')?;
        if name.trim().eq_ignore_ascii_case("host") {
            hosts.push(value.trim().to_string());
        }
    }
    let host = if hosts.len() == 1 { hosts.pop() } else { None };
    Some(Req {
        method,
        target,
        host,
    })
}
