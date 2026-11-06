// Dev server (web-dev-loop, S4, S5, S6): real HTTP over loopback, real file changes, real events.
use std::fs;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use aqelith_web::dev::{start, Config, StartError};

const GOOD: &str = "module notes\n\
page home \"/\" {\n    title \"Notes\";\n    h1 \"Notes v1\";\n    link \"Next\" \"/next\";\n}\n\
page next \"/next\" {\n    title \"Next\";\n    h1 \"Next page\";\n    link \"Home\" \"/\";\n}\n";

/// A private scratch directory for one test, removed on drop.
struct Scratch(PathBuf);

impl Scratch {
    fn new(name: &str) -> Scratch {
        // A counter makes every name unique: the clock alone can repeat for parallel tests (its
        // resolution is coarse), and two tests sharing a directory would share a source file.
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let id = NEXT.fetch_add(1, Ordering::Relaxed);
        let dir =
            std::env::temp_dir().join(format!("aq-dev-{name}-{}-{id}-{nanos}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        Scratch(dir)
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn start_with(src: &str) -> (Scratch, PathBuf, aqelith_web::dev::Server) {
    let scratch = Scratch::new("srv");
    let file = scratch.0.join("notes.aq");
    fs::write(&file, src).unwrap();
    let server = start(Config {
        source: file.clone(),
        style: None,
        port: 0,
    })
    .expect("starts");
    (scratch, file, server)
}

/// Sends raw bytes and returns the whole response (the server closes after each response).
fn send(port: u16, raw: &[u8]) -> String {
    let mut stream = TcpStream::connect(("127.0.0.1", port)).unwrap();
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .unwrap();
    // The server may answer and close before the whole request is written (for example 431 on an
    // oversized head); a write error after that is expected, so it is not a test failure.
    let _ = stream.write_all(raw);
    let mut out = Vec::new();
    let _ = stream.read_to_end(&mut out);
    String::from_utf8_lossy(&out).into_owned()
}

fn get(port: u16, path: &str) -> String {
    send(
        port,
        format!("GET {path} HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\n\r\n").as_bytes(),
    )
}

fn status(response: &str) -> &str {
    response.lines().next().unwrap_or("")
}

fn body(response: &str) -> &str {
    response.split("\r\n\r\n").nth(1).unwrap_or("")
}

#[test]
fn serves_pages_with_reload_client_and_security_headers() {
    let (_s, _f, server) = start_with(GOOD);
    let port = server.addr.port();
    let home = get(port, "/");
    assert_eq!(status(&home), "HTTP/1.1 200 OK", "{home}");
    assert!(body(&home).contains("<h1>Notes v1</h1>"), "{home}");
    assert!(
        body(&home).contains("<script src=\"/__aq/reload.js\"></script>"),
        "{home}"
    );
    assert!(home.contains("Content-Security-Policy: default-src 'none'; style-src 'self'; script-src 'self'; connect-src 'self'; img-src 'self'; base-uri 'none'; form-action 'none'; frame-ancestors 'none'"), "{home}");
    assert!(home.contains("X-Content-Type-Options: nosniff"), "{home}");
    assert!(home.contains("Referrer-Policy: no-referrer"), "{home}");
    assert!(home.contains("Cache-Control: no-store"), "{home}");

    let next = get(port, "/next");
    assert_eq!(status(&next), "HTTP/1.1 200 OK", "{next}");
    assert!(body(&next).contains("Next page"), "{next}");

    let script = get(port, "/__aq/reload.js");
    assert_eq!(status(&script), "HTTP/1.1 200 OK");
    assert!(script.contains("Content-Type: text/javascript"), "{script}");
    assert!(body(&script).contains("EventSource"), "{script}");
}

#[test]
fn unknown_paths_and_traversal_are_not_found() {
    let (_s, _f, server) = start_with(GOOD);
    let port = server.addr.port();
    assert_eq!(status(&get(port, "/missing")), "HTTP/1.1 404 Not Found");
    assert_eq!(status(&get(port, "/../notes.aq")), "HTTP/1.1 404 Not Found");
    assert_eq!(
        status(&get(port, "/%2e%2e/notes.aq")),
        "HTTP/1.1 404 Not Found"
    );
}

#[test]
fn only_get_is_allowed() {
    let (_s, _f, server) = start_with(GOOD);
    let port = server.addr.port();
    let post = send(
        port,
        format!("POST / HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nContent-Length: 0\r\n\r\n")
            .as_bytes(),
    );
    assert_eq!(status(&post), "HTTP/1.1 405 Method Not Allowed", "{post}");
    assert!(post.contains("Allow: GET"), "{post}");
}

#[test]
fn only_our_loopback_host_names_are_accepted() {
    let (_s, _f, server) = start_with(GOOD);
    let port = server.addr.port();
    for host in [
        "evil.example".to_string(),
        format!("evil.example:{port}"),
        format!("127.0.0.1.evil.example:{port}"),
        format!("127.0.0.1:{}", port + 1),
    ] {
        let r = send(
            port,
            format!("GET / HTTP/1.1\r\nHost: {host}\r\n\r\n").as_bytes(),
        );
        assert_eq!(status(&r), "HTTP/1.1 403 Forbidden", "host {host}: {r}");
    }
    let no_host = send(port, b"GET / HTTP/1.1\r\n\r\n");
    assert_eq!(status(&no_host), "HTTP/1.1 403 Forbidden", "{no_host}");
    let localhost = send(
        port,
        format!("GET / HTTP/1.1\r\nHost: localhost:{port}\r\n\r\n").as_bytes(),
    );
    assert_eq!(status(&localhost), "HTTP/1.1 200 OK", "{localhost}");
}

#[test]
fn oversized_request_head_is_refused() {
    let (_s, _f, server) = start_with(GOOD);
    let port = server.addr.port();
    let mut raw = format!("GET / HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nX-Pad: ").into_bytes();
    raw.extend(std::iter::repeat_n(b'a', 9 * 1024));
    let r = send(port, &raw);
    assert_eq!(
        status(&r),
        "HTTP/1.1 431 Request Header Fields Too Large",
        "{r}"
    );
}

#[test]
fn malformed_requests_are_bad_requests() {
    let (_s, _f, server) = start_with(GOOD);
    let port = server.addr.port();
    assert_eq!(
        status(&send(port, b"nonsense\r\n\r\n")),
        "HTTP/1.1 400 Bad Request"
    );
    // Two Host headers are ambiguous and are refused as a forbidden host (S4).
    assert_eq!(
        status(&send(
            port,
            format!("GET / HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nHost: 127.0.0.1:{port}\r\n\r\n")
                .as_bytes()
        )),
        "HTTP/1.1 403 Forbidden"
    );
}

#[test]
fn every_method_except_get_is_refused() {
    let (_s, _f, server) = start_with(GOOD);
    let port = server.addr.port();
    for method in ["HEAD", "OPTIONS", "PUT", "DELETE", "get", "CONNECT"] {
        let r = send(
            port,
            format!("{method} / HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\n\r\n").as_bytes(),
        );
        assert_eq!(
            status(&r),
            "HTTP/1.1 405 Method Not Allowed",
            "method {method}: {r}"
        );
    }
}

#[test]
fn request_head_limit_is_exactly_8_kib() {
    let (_s, _f, server) = start_with(GOOD);
    let port = server.addr.port();
    let prefix = format!("GET / HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nX-Pad: ");
    let suffix = "\r\n\r\n";
    // Total head length of exactly 8192 bytes (terminator included) is accepted...
    let pad = 8192 - prefix.len() - suffix.len();
    let at_limit = format!("{prefix}{}{suffix}", "a".repeat(pad));
    assert_eq!(at_limit.len(), 8192);
    assert_eq!(status(&send(port, at_limit.as_bytes())), "HTTP/1.1 200 OK");
    // ...and one byte more is refused.
    let over = format!("{prefix}{}{suffix}", "a".repeat(pad + 1));
    assert_eq!(
        status(&send(port, over.as_bytes())),
        "HTTP/1.1 431 Request Header Fields Too Large"
    );
}

#[test]
fn event_streams_are_bounded() {
    let (_s, _f, server) = start_with(GOOD);
    let port = server.addr.port();
    let mut open = Vec::new();
    for _ in 0..64 {
        let mut s = TcpStream::connect(("127.0.0.1", port)).unwrap();
        s.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
        s.write_all(
            format!("GET /__aq/events HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\n\r\n").as_bytes(),
        )
        .unwrap();
        let mut first = [0u8; 64];
        let n = s.read(&mut first).unwrap();
        assert!(String::from_utf8_lossy(&first[..n]).starts_with("HTTP/1.1 200 OK"));
        open.push(s);
    }
    let refused = send(
        port,
        format!("GET /__aq/events HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\n\r\n").as_bytes(),
    );
    assert_eq!(
        status(&refused),
        "HTTP/1.1 503 Service Unavailable",
        "{refused}"
    );
    drop(open);
}

#[test]
fn a_rewrite_with_an_old_modification_time_still_reloads() {
    let (_s, file, server) = start_with(GOOD);
    let port = server.addr.port();
    let old_mtime = fs::metadata(&file).unwrap().modified().unwrap();
    let events = subscribe(port);
    let listener =
        std::thread::spawn(move || await_event(events, "event: reload", Duration::from_secs(8)));
    std::thread::sleep(Duration::from_millis(400));
    fs::write(&file, GOOD.replace("Notes v1", "Notes restored")).unwrap();
    // Copy-with-preserved-times style: the new content carries the old modification time.
    fs::OpenOptions::new()
        .write(true)
        .open(&file)
        .unwrap()
        .set_modified(old_mtime)
        .unwrap();
    assert!(
        listener.join().unwrap().is_some(),
        "no reload for a same-mtime rewrite"
    );
    assert!(get(port, "/").contains("Notes restored"));
}

/// A subscribed event stream, plus any bytes read past the confirmation (a frame can arrive in the
/// same read as the confirmation; those bytes must not be dropped).
struct Events {
    stream: TcpStream,
    pending: String,
}

/// Opens an event stream and returns it once the server confirms the subscription (`: connected`).
/// Subscribe before the change under test: a frame sent earlier is not replayed to a late client.
fn subscribe(port: u16) -> Events {
    const CONFIRMED: &[u8] = b": connected\n\n";
    let mut stream = TcpStream::connect(("127.0.0.1", port)).unwrap();
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .unwrap();
    stream
        .write_all(
            format!("GET /__aq/events HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\n\r\n").as_bytes(),
        )
        .unwrap();
    let mut seen: Vec<u8> = Vec::new();
    let mut buf = [0u8; 256];
    let after = loop {
        if let Some(i) = seen.windows(CONFIRMED.len()).position(|w| w == CONFIRMED) {
            break i + CONFIRMED.len();
        }
        let n = stream.read(&mut buf).expect("subscription confirmed");
        assert!(n > 0, "server closed the event stream");
        seen.extend_from_slice(&buf[..n]);
    };
    stream
        .set_read_timeout(Some(Duration::from_millis(500)))
        .unwrap();
    Events {
        stream,
        pending: String::from_utf8_lossy(&seen[after..]).into_owned(),
    }
}

/// Reads a subscribed stream until `needle` appears or the deadline passes.
fn await_event(mut events: Events, needle: &str, within: Duration) -> Option<String> {
    if events.pending.contains(needle) {
        return Some(events.pending);
    }
    let deadline = Instant::now() + within;
    let mut seen = events.pending;
    let mut buf = [0u8; 1024];
    while Instant::now() < deadline {
        match events.stream.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => seen.push_str(&String::from_utf8_lossy(&buf[..n])),
            Err(_) => {}
        }
        if seen.contains(needle) {
            return Some(seen);
        }
    }
    None
}

#[test]
fn saving_the_source_sends_reload_and_serves_the_new_page() {
    let (_s, file, server) = start_with(GOOD);
    let port = server.addr.port();
    let events = subscribe(port);
    let listener =
        std::thread::spawn(move || await_event(events, "event: reload", Duration::from_secs(8)));
    std::thread::sleep(Duration::from_millis(400));
    fs::write(&file, GOOD.replace("Notes v1", "Notes v2")).unwrap();
    let seen = listener.join().unwrap();
    assert!(seen.is_some(), "no reload event after a save");
    assert!(
        get(port, "/").contains("Notes v2"),
        "the new build is not served"
    );
}

#[test]
fn broken_source_sends_diagnostics_and_keeps_the_last_good_build() {
    let (_s, file, server) = start_with(GOOD);
    let port = server.addr.port();
    let events = subscribe(port);
    let listener = std::thread::spawn(move || {
        await_event(events, "event: build-error", Duration::from_secs(8))
    });
    std::thread::sleep(Duration::from_millis(400));
    fs::write(
        &file,
        "module notes\npage home \"/\" {\n    title \"Notes\"\n}\n",
    )
    .unwrap();
    let seen = listener.join().unwrap().expect("a build-error event");
    assert!(seen.contains("event: build-error"), "{seen}");
    assert!(
        seen.contains("PAR001"),
        "the diagnostic code is sent: {seen}"
    );
    // The page still shows the last good build.
    assert!(
        get(port, "/").contains("Notes v1"),
        "last good build was replaced"
    );
    assert!(server.error().is_some());
}

#[test]
fn a_page_that_opens_during_an_error_sees_the_banner_at_once() {
    let (_s, file, server) = start_with(GOOD);
    let port = server.addr.port();
    fs::write(
        &file,
        "module notes\npage home \"/\" {\n    title \"Notes\"\n}\n",
    )
    .unwrap();
    let deadline = Instant::now() + Duration::from_secs(8);
    while server.error().is_none() && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(100));
    }
    let seen = await_event(
        subscribe(port),
        "event: build-error",
        Duration::from_secs(3),
    );
    assert!(
        seen.is_some(),
        "a new connection did not get the current error"
    );
}

#[test]
fn fixing_the_source_sends_reload_again() {
    let (_s, file, server) = start_with(GOOD);
    let port = server.addr.port();
    fs::write(
        &file,
        "module notes\npage home \"/\" {\n    title \"Notes\"\n}\n",
    )
    .unwrap();
    let deadline = Instant::now() + Duration::from_secs(8);
    while server.error().is_none() && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(100));
    }
    let events = subscribe(port);
    let listener =
        std::thread::spawn(move || await_event(events, "event: reload", Duration::from_secs(8)));
    std::thread::sleep(Duration::from_millis(400));
    fs::write(&file, GOOD.replace("Notes v1", "Notes fixed")).unwrap();
    assert!(
        listener.join().unwrap().is_some(),
        "no reload after the fix"
    );
    assert!(server.error().is_none());
    assert!(get(port, "/").contains("Notes fixed"));
}

#[test]
fn a_source_that_does_not_check_at_start_is_refused() {
    let scratch = Scratch::new("bad");
    let file = scratch.0.join("bad.aq");
    fs::write(&file, "module bad\npage \"/\" { }\n").unwrap();
    let result = start(Config {
        source: file,
        style: None,
        port: 0,
    });
    assert!(matches!(result, Err(StartError::Build(_))));
}

#[test]
fn an_occupied_port_is_a_bind_error() {
    let (_s, _f, first) = start_with(GOOD);
    let scratch = Scratch::new("busy");
    let file = scratch.0.join("notes.aq");
    fs::write(&file, GOOD).unwrap();
    let result = start(Config {
        source: file,
        style: None,
        port: first.addr.port(),
    });
    assert!(matches!(result, Err(StartError::Bind(_))));
}

#[test]
fn a_slow_request_head_is_timed_out_as_a_whole() {
    let (_s, _f, server) = start_with(GOOD);
    let port = server.addr.port();
    let mut stream = TcpStream::connect(("127.0.0.1", port)).unwrap();
    stream
        .set_read_timeout(Some(Duration::from_secs(15)))
        .unwrap();
    // One byte at a time, each well inside the per-read timeout: only a total deadline stops it.
    let started = Instant::now();
    for byte in b"GET / HTTP/1.1\r\nHost: 127.0.0.1" {
        if stream.write_all(&[*byte]).is_err() {
            break;
        }
        std::thread::sleep(Duration::from_millis(400));
    }
    let mut out = Vec::new();
    let _ = stream.read_to_end(&mut out);
    let text = String::from_utf8_lossy(&out);
    assert!(text.starts_with("HTTP/1.1 408"), "{text}");
    assert!(
        started.elapsed() < Duration::from_secs(12),
        "the head was allowed to stay open"
    );
}
