// Loopback HTTP host, milestone S3 (docs/architecture/SERVE.md): real sockets on 127.0.0.1.

use aqelith::ir::lower_source;
use aqelith::serve::{start, ServeError};
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};

const HANDLER: &str = "module site\n\nfn handle(path: str) -> str {\n    \"hello\"\n}\n\nfn same(p: str) -> bool {\n    p == \"/\"\n}\n";

fn request(addr: SocketAddr, raw: &[u8]) -> String {
    let mut s = TcpStream::connect(addr).unwrap();
    s.set_read_timeout(Some(std::time::Duration::from_secs(5)))
        .unwrap();
    let _ = s.write_all(raw);
    let mut out = Vec::new();
    let _ = s.read_to_end(&mut out);
    String::from_utf8_lossy(&out).into_owned()
}

fn get(addr: SocketAddr, path: &str) -> String {
    request(
        addr,
        format!(
            "GET {path} HTTP/1.1\r\nHost: 127.0.0.1:{}\r\n\r\n",
            addr.port()
        )
        .as_bytes(),
    )
}

fn status(r: &str) -> &str {
    r.lines().next().unwrap_or("")
}

#[test]
fn a_get_returns_the_handler_text_with_security_headers() {
    let p = lower_source(HANDLER).unwrap();
    let addr = start(p, 0).expect("starts");
    let r = get(addr, "/any/path?q=1");
    assert_eq!(status(&r), "HTTP/1.1 200 OK", "{r}");
    assert!(r.ends_with("hello"), "{r}");
    assert!(r.contains("X-Content-Type-Options: nosniff"), "{r}");
    assert!(r.contains("Content-Security-Policy:"), "{r}");
}

#[test]
fn the_handler_receives_the_path_without_the_query() {
    let src = "module site\n\nfn handle(path: str) -> str {\n    path\n}\n";
    let addr = start(lower_source(src).unwrap(), 0).unwrap();
    let r = get(addr, "/docs?secret=1");
    assert!(r.ends_with("/docs"), "{r}");
}

#[test]
fn only_get_is_served() {
    let addr = start(lower_source(HANDLER).unwrap(), 0).unwrap();
    let r = request(
        addr,
        format!(
            "POST / HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nContent-Length: 0\r\n\r\n",
            addr.port()
        )
        .as_bytes(),
    );
    assert_eq!(status(&r), "HTTP/1.1 405 Method Not Allowed", "{r}");
}

#[test]
fn foreign_host_names_are_refused_before_the_handler_runs() {
    let addr = start(lower_source(HANDLER).unwrap(), 0).unwrap();
    let r = request(addr, b"GET / HTTP/1.1\r\nHost: evil.example\r\n\r\n");
    assert_eq!(status(&r), "HTTP/1.1 403 Forbidden", "{r}");
    let none = request(addr, b"GET / HTTP/1.1\r\n\r\n");
    assert_eq!(status(&none), "HTTP/1.1 403 Forbidden", "{none}");
}

#[test]
fn an_oversized_head_is_refused_and_the_server_keeps_serving() {
    let addr = start(lower_source(HANDLER).unwrap(), 0).unwrap();
    let mut raw = format!(
        "GET / HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nX-Pad: ",
        addr.port()
    )
    .into_bytes();
    raw.extend(std::iter::repeat_n(b'a', 9 * 1024));
    assert_eq!(
        status(&request(addr, &raw)),
        "HTTP/1.1 431 Request Header Fields Too Large"
    );
    assert_eq!(status(&get(addr, "/")), "HTTP/1.1 200 OK");
}

#[test]
fn a_handler_that_declares_an_effect_is_not_started() {
    let src = "module site\n\nfn handle(path: str) -> str uses net {\n    \"no\"\n}\n";
    let p = lower_source(src).unwrap();
    assert!(matches!(start(p, 0), Err(ServeError::BadHandler(_))));
}

#[test]
fn a_program_without_the_right_handler_is_refused() {
    let p = lower_source("module m\n\nfn other() -> i32 {\n    1\n}\n").unwrap();
    assert!(matches!(start(p, 0), Err(ServeError::BadHandler(_))));
    let wrong = lower_source("module m\n\nfn handle(p: i32) -> str {\n    \"x\"\n}\n").unwrap();
    assert!(matches!(start(wrong, 0), Err(ServeError::BadHandler(_))));
}
