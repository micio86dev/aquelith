//! MCP over HTTP (M8): loopback serving, Host-header guard, status codes, JSON bodies.

use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};

use aqelith::mcp::{is_local_host, serve_http};

fn start(max: usize) -> SocketAddr {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    std::thread::spawn(move || {
        let _ = serve_http(&listener, Some(max));
    });
    addr
}

fn send(addr: SocketAddr, raw: &str) -> String {
    let mut stream = TcpStream::connect(addr).unwrap();
    stream.write_all(raw.as_bytes()).unwrap();
    let mut out = String::new();
    stream.read_to_string(&mut out).unwrap();
    out
}

fn post(host: &str, body: &str) -> String {
    format!(
        "POST /mcp HTTP/1.1\r\nHost: {host}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    )
}

#[test]
fn a_request_gets_the_json_reply_with_200() {
    let addr = start(1);
    let body = r#"{"jsonrpc":"2.0","id":1,"method":"tools/list"}"#;
    let reply = send(addr, &post(&format!("127.0.0.1:{}", addr.port()), body));
    assert!(reply.starts_with("HTTP/1.1 200 OK"), "{reply}");
    assert!(reply.contains("application/json"), "{reply}");
    assert!(reply.contains("aqelith_patch"), "{reply}");
}

#[test]
fn a_notification_gets_202_and_no_body() {
    let addr = start(1);
    let body = r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#;
    let reply = send(addr, &post("localhost", body));
    assert!(reply.starts_with("HTTP/1.1 202 Accepted"), "{reply}");
    assert!(reply.ends_with("\r\n\r\n"), "{reply}");
}

#[test]
fn a_foreign_host_header_is_refused_against_dns_rebinding() {
    let addr = start(1);
    let reply = send(addr, &post("evil.example", r#"{"jsonrpc":"2.0","id":1,"method":"ping"}"#));
    assert!(reply.starts_with("HTTP/1.1 403"), "{reply}");
}

#[test]
fn other_methods_and_paths_are_not_found() {
    let addr = start(1);
    let raw = "GET /mcp HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n";
    assert!(send(addr, raw).starts_with("HTTP/1.1 404"));
}

#[test]
fn only_local_host_names_pass_the_guard() {
    assert!(is_local_host("localhost:8080"));
    assert!(is_local_host("127.0.0.1:1"));
    assert!(is_local_host("[::1]:9"));
    assert!(!is_local_host("localhost.evil.example"));
    assert!(!is_local_host("192.168.0.10"));
}
