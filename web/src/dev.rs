//! Development server with live reload (feature web-dev-loop, S4 and S5).
//!
//! It builds the source on start, serves the result from memory on 127.0.0.1, and watches the
//! source and stylesheet files. When one changes, it rebuilds and tells every open page to reload
//! (Server-Sent Events). A source that does not check leaves the last good build on screen and
//! sends the diagnostics to the page as a banner.
//!
//! Security: GET only; a `Host` header other than `127.0.0.1:<port>` or `localhost:<port>` gets
//! 403 (this blocks DNS rebinding); requests are served from memory, so no request reaches the
//! disk and no path can escape the site; the request head is limited to 8 KiB; every response
//! carries a Content-Security-Policy and `nosniff`.

use std::collections::hash_map::DefaultHasher;
use std::collections::BTreeMap;
use std::hash::{Hash, Hasher};
use std::io::{ErrorKind, Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{channel, RecvTimeoutError, Sender};
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread;
use std::time::{Duration, Instant};

use crate::{json_string, site_from_source, site_from_source_with_style, SourceError};

/// Path of the reload client that the dev server injects into every page.
pub const RELOAD_PATH: &str = "/__aq/reload.js";
/// Path of the event stream that the reload client listens to.
pub const EVENTS_PATH: &str = "/__aq/events";
/// Largest request head accepted (request line and headers).
pub const MAX_HEAD: usize = 8 * 1024;

/// Dev policy: the injected client script and the live connection come from this origin.
pub const DEV_CSP: &str = "default-src 'none'; style-src 'self'; script-src 'self'; connect-src 'self'; img-src 'self'; base-uri 'none'; form-action 'none'; frame-ancestors 'none'";

const RELOAD_JS: &str = include_str!("../runtime/reload.js");
const POLL_EVERY: Duration = Duration::from_millis(250);
const SETTLE: Duration = Duration::from_millis(100);
const SETTLE_ROUNDS: usize = 20;
const PING_EVERY: Duration = Duration::from_secs(15);
const IO_TIMEOUT: Duration = Duration::from_secs(5);
/// Total time allowed to send a request head. IO_TIMEOUT is per read, so a client that sends one
/// byte at a time would otherwise hold a connection for as long as it likes.
const HEAD_DEADLINE: Duration = Duration::from_secs(5);
/// Connections served at once, of any kind. Event streams are also capped by MAX_STREAMS.
pub const MAX_CONNECTIONS: usize = 128;

/// One file of the built site, held in memory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Asset {
    pub content_type: &'static str,
    pub body: Vec<u8>,
}

/// What the dev server watches and serves.
#[derive(Debug, Clone)]
pub struct Config {
    pub source: PathBuf,
    pub style: Option<PathBuf>,
    /// 0 asks the system for a free port.
    pub port: u16,
}

#[derive(Debug)]
pub enum StartError {
    /// The source does not check, or a file cannot be read. The message lists the diagnostics.
    Build(String),
    /// The address cannot be bound.
    Bind(String),
}

impl std::fmt::Display for StartError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StartError::Build(m) | StartError::Bind(m) => f.write_str(m),
        }
    }
}

/// A running dev server. Dropping it does not stop the threads; the process ends them.
pub struct Server {
    pub addr: SocketAddr,
    state: Arc<Mutex<State>>,
}

impl Server {
    /// The current built files (the last good build, even while the source has an error).
    pub fn files(&self) -> Arc<BTreeMap<String, Asset>> {
        lock(&self.state).files.clone()
    }

    /// The diagnostics of the last failed build, if the source does not check now.
    pub fn error(&self) -> Option<String> {
        lock(&self.state).error.clone()
    }
}

struct State {
    files: Arc<BTreeMap<String, Asset>>,
    error: Option<String>,
    /// Live event streams by id. An entry is removed when its stream ends, so the list never
    /// holds more entries than there are open streams.
    subscribers: Vec<(u64, Sender<String>)>,
    next_id: u64,
    /// Open event streams. Bounded by `MAX_STREAMS` so that a loop of connections cannot exhaust
    /// the process.
    streams: usize,
}

/// Most event streams open at once. A page holds one; a dev session needs a handful.
pub const MAX_STREAMS: usize = 64;

fn admit_stream(state: &Mutex<State>) -> bool {
    let mut s = lock(state);
    if s.streams >= MAX_STREAMS {
        return false;
    }
    s.streams += 1;
    true
}

fn lock(state: &Mutex<State>) -> MutexGuard<'_, State> {
    // A panicked connection thread must not take the whole server down.
    state.lock().unwrap_or_else(|e| e.into_inner())
}

/// Builds the site for development: the same pages as the static build, without the static CSP
/// and minifier (so the output stays readable), with the reload client injected before `</body>`.
pub fn build(source: &Path, style: Option<&Path>) -> Result<BTreeMap<String, Asset>, String> {
    let src = std::fs::read_to_string(source).map_err(|e| format!("{}: {e}", source.display()))?;
    let files = match style {
        Some(_) => site_from_source_with_style(&src, "/style.css"),
        None => site_from_source(&src),
    }
    .map_err(|e| describe(source, e))?;
    let mut out = BTreeMap::new();
    for (path, html) in files {
        let injected = html.replace(
            "</body>",
            &format!("<script src=\"{RELOAD_PATH}\"></script>\n</body>"),
        );
        out.insert(
            path,
            Asset {
                content_type: "text/html; charset=utf-8",
                body: injected.into_bytes(),
            },
        );
    }
    if let Some(css) = style {
        let body = std::fs::read(css).map_err(|e| format!("{}: {e}", css.display()))?;
        out.insert(
            "style.css".to_string(),
            Asset {
                content_type: "text/css; charset=utf-8",
                body,
            },
        );
    }
    Ok(out)
}

fn describe(source: &Path, e: SourceError) -> String {
    match e {
        SourceError::Diagnostics(diags) => diags
            .iter()
            .map(|d| {
                format!(
                    "{}:{}:{}: {}: {}",
                    source.display(),
                    d.line,
                    d.column,
                    d.code,
                    d.message
                )
            })
            .collect::<Vec<_>>()
            .join("\n"),
        SourceError::Build(e) => format!("{}: {e:?}", source.display()),
    }
}

/// Starts the server and the watcher. Returns once the address is bound and the first build passed.
pub fn start(config: Config) -> Result<Server, StartError> {
    // Stamps are taken before the first build: a save that lands during the build is seen by the
    // watcher, not lost between the build and the start of the watch loop.
    let initial_stamps = stamps(&config);
    let files = build(&config.source, config.style.as_deref()).map_err(StartError::Build)?;
    let listener = TcpListener::bind(("127.0.0.1", config.port))
        .map_err(|e| StartError::Bind(format!("127.0.0.1:{}: {e}", config.port)))?;
    let addr = listener
        .local_addr()
        .map_err(|e| StartError::Bind(e.to_string()))?;
    let state = Arc::new(Mutex::new(State {
        files: Arc::new(files),
        error: None,
        subscribers: Vec::new(),
        next_id: 0,
        streams: 0,
    }));
    {
        let state = Arc::clone(&state);
        thread::spawn(move || watch(config, state, initial_stamps));
    }
    {
        let state = Arc::clone(&state);
        let port = addr.port();
        let active = Arc::new(AtomicUsize::new(0));
        thread::spawn(move || {
            for incoming in listener.incoming() {
                let stream = match incoming {
                    Ok(stream) => stream,
                    Err(_) => {
                        // Accept can fail for lack of file descriptors; pausing avoids a busy loop.
                        thread::sleep(Duration::from_millis(50));
                        continue;
                    }
                };
                // Over the limit, the connection is closed at once instead of taking a thread.
                if active.fetch_add(1, Ordering::SeqCst) >= MAX_CONNECTIONS {
                    active.fetch_sub(1, Ordering::SeqCst);
                    continue;
                }
                let guard = ConnectionGuard(Arc::clone(&active));
                let state = Arc::clone(&state);
                thread::spawn(move || {
                    let _guard = guard;
                    handle(stream, &state, port);
                });
            }
        });
    }
    Ok(Server { addr, state })
}

/// Content fingerprint of each watched file (`None` when it cannot be read). The content is
/// compared, not the modification time: a rewrite with an old mtime (copy with preserved times,
/// archive extraction, rsync -t) is still a change.
fn stamps(config: &Config) -> Vec<Option<u64>> {
    let mut paths = vec![config.source.clone()];
    paths.extend(config.style.clone());
    paths
        .iter()
        .map(|p| {
            std::fs::read(p).ok().map(|bytes| {
                let mut hasher = DefaultHasher::new();
                bytes.hash(&mut hasher);
                hasher.finish()
            })
        })
        .collect()
}

fn watch(config: Config, state: Arc<Mutex<State>>, initial: Vec<Option<u64>>) {
    let mut last = initial;
    loop {
        thread::sleep(POLL_EVERY);
        let mut now = stamps(&config);
        if now == last {
            continue;
        }
        // A write can take several steps (truncate, then write; or an editor's temp file). Rebuild
        // only once the content has been still for one settle period, so no intermediate state
        // reaches the page as a build error or as a false reload. Bounded, so a writer that never
        // stops cannot hold the watcher forever.
        for _ in 0..SETTLE_ROUNDS {
            thread::sleep(SETTLE);
            let again = stamps(&config);
            if again == now {
                break;
            }
            now = again;
        }
        last = now;
        match build(&config.source, config.style.as_deref()) {
            Ok(files) => {
                let mut s = lock(&state);
                s.files = Arc::new(files);
                s.error = None;
                broadcast(&mut s, &frame("reload", "{}"));
                eprintln!("rebuilt {}", config.source.display());
            }
            Err(message) => {
                let mut s = lock(&state);
                s.error = Some(message.clone());
                broadcast(&mut s, &frame("build-error", &json_string(&message)));
                eprintln!("{message}");
            }
        }
    }
}

fn frame(event: &str, data: &str) -> String {
    format!("event: {event}\ndata: {data}\n\n")
}

fn broadcast(state: &mut State, frame: &str) {
    state
        .subscribers
        .retain(|(_, tx)| tx.send(frame.to_string()).is_ok());
}

/// Releases one event-stream slot when the stream ends, even if it panics.
struct StreamSlot<'a>(&'a Mutex<State>);

impl Drop for StreamSlot<'_> {
    fn drop(&mut self) {
        lock(self.0).streams -= 1;
    }
}

/// Releases one connection slot when the connection thread ends, even if it panics.
struct ConnectionGuard(Arc<AtomicUsize>);

impl Drop for ConnectionGuard {
    fn drop(&mut self) {
        self.0.fetch_sub(1, Ordering::SeqCst);
    }
}

fn handle(mut stream: TcpStream, state: &Mutex<State>, port: u16) {
    let _ = stream.set_read_timeout(Some(IO_TIMEOUT));
    let _ = stream.set_write_timeout(Some(IO_TIMEOUT));
    let head = match read_head(&mut stream) {
        Ok(head) => head,
        Err(status) => {
            respond(&mut stream, status, "text/plain; charset=utf-8", &[], b"");
            return linger_close(stream);
        }
    };
    let request = match parse_head(&head) {
        Some(request) => request,
        None => {
            return respond(
                &mut stream,
                "400 Bad Request",
                "text/plain; charset=utf-8",
                &[],
                b"bad request",
            )
        }
    };
    if request.method != "GET" {
        return respond(
            &mut stream,
            "405 Method Not Allowed",
            "text/plain; charset=utf-8",
            &[("Allow", "GET")],
            b"method not allowed",
        );
    }
    if !host_allowed(request.host.as_deref(), port) {
        return respond(
            &mut stream,
            "403 Forbidden",
            "text/plain; charset=utf-8",
            &[],
            b"forbidden host",
        );
    }
    let path = request.target.split('?').next().unwrap_or("/");
    if path == EVENTS_PATH {
        if !admit_stream(state) {
            return respond(
                &mut stream,
                "503 Service Unavailable",
                "text/plain; charset=utf-8",
                &[],
                b"too many live connections",
            );
        }
        // The slot is released by the guard, so a panic inside `events` cannot leak it.
        let _slot = StreamSlot(state);
        events(stream, state);
        return;
    }
    if path == RELOAD_PATH {
        return respond(
            &mut stream,
            "200 OK",
            "text/javascript; charset=utf-8",
            &[],
            RELOAD_JS.as_bytes(),
        );
    }
    let files = Arc::clone(&lock(state).files);
    match lookup(&files, path) {
        Some(asset) => respond(&mut stream, "200 OK", asset.content_type, &[], &asset.body),
        None => respond(
            &mut stream,
            "404 Not Found",
            "text/plain; charset=utf-8",
            &[],
            b"not found",
        ),
    }
}

/// Closes a connection that was refused with unread request bytes still arriving. Closing with
/// unread input makes the kernel send a reset, which can discard the response the client has not
/// read yet. So: stop sending, then read and discard what is left, for a short time and a bounded
/// amount.
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

/// Reads the request head up to the blank line, within HEAD_DEADLINE in total. Errors are the
/// status to answer with.
fn read_head(stream: &mut TcpStream) -> Result<String, &'static str> {
    let deadline = Instant::now() + HEAD_DEADLINE;
    let mut buf: Vec<u8> = Vec::with_capacity(512);
    loop {
        if let Some(end) = find(&buf, b"\r\n\r\n") {
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

fn find(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack.windows(needle.len()).position(|w| w == needle)
}

#[derive(Debug, PartialEq, Eq)]
pub struct Request {
    pub method: String,
    pub target: String,
    /// The value of the single Host header, if present.
    pub host: Option<String>,
}

/// Parses an HTTP/1.x request head. `None` if the request line is malformed, the version is not
/// HTTP/1.x, or a header line has no colon. Two Host headers give `host: None` (refused later).
pub fn parse_head(head: &str) -> Option<Request> {
    let mut lines = head.split("\r\n");
    let mut parts = lines.next()?.split(' ');
    let method = parts.next()?.to_string();
    let target = parts.next()?.to_string();
    let version = parts.next()?;
    if parts.next().is_some() || !version.starts_with("HTTP/1.") || !target.starts_with('/') {
        return None;
    }
    let mut hosts: Vec<String> = Vec::new();
    for line in lines {
        if line.is_empty() {
            break;
        }
        let (name, value) = line.split_once(':')?;
        if name.trim().eq_ignore_ascii_case("host") {
            hosts.push(value.trim().to_string());
        }
    }
    // Two Host headers are ambiguous: the request gets no host, so it is refused as a forbidden host.
    let host = if hosts.len() == 1 { hosts.pop() } else { None };
    Some(Request {
        method,
        target,
        host,
    })
}

/// True only for the loopback names of this server's own port.
pub fn host_allowed(host: Option<&str>, port: u16) -> bool {
    match host {
        Some(h) => h == format!("127.0.0.1:{port}") || h == format!("localhost:{port}"),
        None => false,
    }
}

/// Finds the file for a request path in the built site, or `None`.
pub fn lookup<'a>(files: &'a BTreeMap<String, Asset>, path: &str) -> Option<&'a Asset> {
    if !path.starts_with('/') || path.contains("..") || path.contains('\\') || path.contains('\0') {
        return None;
    }
    let rel = &path[1..];
    let candidates: Vec<String> = if rel.is_empty() {
        vec!["index.html".to_string()]
    } else if rel.ends_with('/') {
        vec![format!("{rel}index.html")]
    } else {
        vec![rel.to_string(), format!("{rel}/index.html")]
    };
    candidates.iter().find_map(|k| files.get(k))
}

fn respond(
    stream: &mut TcpStream,
    status: &str,
    content_type: &str,
    extra: &[(&str, &str)],
    body: &[u8],
) {
    let mut head = format!(
        "HTTP/1.1 {status}\r\n\
Content-Type: {content_type}\r\n\
Content-Length: {}\r\n\
Content-Security-Policy: {DEV_CSP}\r\n\
Cache-Control: no-store\r\n\
X-Content-Type-Options: nosniff\r\n\
Referrer-Policy: no-referrer\r\n\
Connection: close\r\n",
        body.len()
    );
    for (name, value) in extra {
        head.push_str(&format!("{name}: {value}\r\n"));
    }
    head.push_str("\r\n");
    let _ = stream.write_all(head.as_bytes());
    let _ = stream.write_all(body);
}

/// Holds the connection open and pushes events to it until the client goes away. The subscriber
/// entry is removed on every exit path.
fn events(stream: TcpStream, state: &Mutex<State>) {
    let (id, rx) = {
        let (tx, rx) = channel::<String>();
        let mut s = lock(state);
        if let Some(message) = &s.error {
            // A page that opens while the source is broken sees the banner at once.
            let _ = tx.send(frame("build-error", &json_string(message)));
        }
        s.next_id += 1;
        let id = s.next_id;
        s.subscribers.push((id, tx));
        (id, rx)
    };
    pump(stream, &rx);
    lock(state).subscribers.retain(|(i, _)| *i != id);
}

fn pump(mut stream: TcpStream, rx: &std::sync::mpsc::Receiver<String>) {
    let head = format!(
        "HTTP/1.1 200 OK\r\n\
Content-Type: text/event-stream; charset=utf-8\r\n\
Content-Security-Policy: {DEV_CSP}\r\n\
Cache-Control: no-store\r\n\
X-Content-Type-Options: nosniff\r\n\
Connection: close\r\n\r\n: connected\n\n"
    );
    if stream.write_all(head.as_bytes()).is_err() {
        return;
    }
    loop {
        match rx.recv_timeout(PING_EVERY) {
            Ok(message) => {
                if stream.write_all(message.as_bytes()).is_err() {
                    return;
                }
            }
            Err(RecvTimeoutError::Timeout) => {
                if stream.write_all(b": ping\n\n").is_err() {
                    return;
                }
            }
            Err(RecvTimeoutError::Disconnected) => return,
        }
        let _ = stream.flush();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn asset(body: &str) -> Asset {
        Asset {
            content_type: "text/html; charset=utf-8",
            body: body.as_bytes().to_vec(),
        }
    }

    #[test]
    fn parse_accepts_a_plain_get() {
        let r = parse_head("GET /how HTTP/1.1\r\nHost: 127.0.0.1:8080\r\n\r\n").unwrap();
        assert_eq!(r.method, "GET");
        assert_eq!(r.target, "/how");
        assert_eq!(r.host.as_deref(), Some("127.0.0.1:8080"));
    }

    #[test]
    fn parse_rejects_bad_lines_and_duplicate_hosts() {
        assert!(parse_head("GET\r\n\r\n").is_none());
        assert!(parse_head("GET / HTTP/2\r\n\r\n").is_none());
        assert!(parse_head("GET relative HTTP/1.1\r\n\r\n").is_none());
        assert!(parse_head("GET / HTTP/1.1\r\nHost: a\r\nhost: b\r\n\r\n")
            .unwrap()
            .host
            .is_none());
    }

    #[test]
    fn host_must_be_our_loopback_name_and_port() {
        assert!(host_allowed(Some("127.0.0.1:8080"), 8080));
        assert!(host_allowed(Some("localhost:8080"), 8080));
        assert!(!host_allowed(Some("127.0.0.1:9999"), 8080));
        assert!(!host_allowed(Some("evil.example:8080"), 8080));
        assert!(!host_allowed(Some("127.0.0.1.evil.example:8080"), 8080));
        assert!(!host_allowed(None, 8080));
    }

    #[test]
    fn lookup_maps_routes_to_files_and_refuses_escapes() {
        let mut files = BTreeMap::new();
        files.insert("index.html".to_string(), asset("home"));
        files.insert("how/index.html".to_string(), asset("how"));
        files.insert(
            "style.css".to_string(),
            Asset {
                content_type: "text/css; charset=utf-8",
                body: vec![],
            },
        );
        assert_eq!(lookup(&files, "/").unwrap().body, b"home");
        assert_eq!(lookup(&files, "/how").unwrap().body, b"how");
        assert_eq!(lookup(&files, "/how/").unwrap().body, b"how");
        assert!(lookup(&files, "/style.css").is_some());
        assert!(lookup(&files, "/nope").is_none());
        assert!(lookup(&files, "/../secret").is_none());
        assert!(lookup(&files, "/how/../../etc/passwd").is_none());
        assert!(lookup(&files, "/a\\b").is_none());
        assert!(lookup(&files, "no-leading-slash").is_none());
    }
}
