// Conformance of the Linux server target (docs/platforms/SERVER_CONFORMANCE.md).
// Interface under test: loopback HTTP. Each case has an expected status and body, compared exactly.
// A case that cannot run makes its test fail; nothing is skipped silently.

use aqelith::ir::lower_source;
use aqelith::serve::{start, ServeError};
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};

const SITE: &str = "module site\n\nrecord Request {\n    path: str,\n}\n\nrecord Response {\n    body: str,\n}\n\nfn handle(req: Request) -> Response {\n    Response { body: \"ok\" }\n}\n";

fn exchange(addr: SocketAddr, raw: &[u8]) -> String {
    let mut s = TcpStream::connect(addr).unwrap();
    s.set_read_timeout(Some(std::time::Duration::from_secs(8)))
        .unwrap();
    let _ = s.write_all(raw);
    let mut out = Vec::new();
    let _ = s.read_to_end(&mut out);
    String::from_utf8_lossy(&out).into_owned()
}

fn status_and_body(r: &str) -> (String, String) {
    let status = r
        .lines()
        .next()
        .unwrap_or("")
        .trim_end_matches('\r')
        .to_string();
    let body = r.split("\r\n\r\n").nth(1).unwrap_or("").to_string();
    (status, body)
}

struct Case {
    id: &'static str,
    program: &'static str,
    request: fn(u16) -> Vec<u8>,
    expect_status: &'static str,
    expect_body: &'static str,
}

fn get(path: &'static str) -> fn(u16) -> Vec<u8> {
    match path {
        "/" => |p| format!("GET / HTTP/1.1\r\nHost: 127.0.0.1:{p}\r\n\r\n").into_bytes(),
        "/docs?x=1" => {
            |p| format!("GET /docs?x=1 HTTP/1.1\r\nHost: 127.0.0.1:{p}\r\n\r\n").into_bytes()
        }
        other => panic!("the test asks for an unknown path: {other}"),
    }
}

fn foreign(_p: u16) -> Vec<u8> {
    b"GET / HTTP/1.1\r\nHost: evil.example\r\n\r\n".to_vec()
}

fn post(p: u16) -> Vec<u8> {
    format!("POST / HTTP/1.1\r\nHost: 127.0.0.1:{p}\r\nContent-Length: 0\r\n\r\n").into_bytes()
}

fn localhost(p: u16) -> Vec<u8> {
    format!("GET / HTTP/1.1\r\nHost: localhost:{p}\r\n\r\n").into_bytes()
}

#[test]
fn the_loopback_interface_conforms() {
    let cases = [
        Case {
            id: "I1 success path",
            program: SITE,
            request: get("/"),
            expect_status: "HTTP/1.1 200 OK",
            expect_body: "ok",
        },
        Case {
            id: "I1 localhost name accepted",
            program: SITE,
            request: localhost,
            expect_status: "HTTP/1.1 200 OK",
            expect_body: "ok",
        },
        Case {
            id: "I2 typed failure: foreign host",
            program: SITE,
            request: foreign,
            expect_status: "HTTP/1.1 403 Forbidden",
            expect_body: "",
        },
        Case {
            id: "I2 typed failure: method",
            program: SITE,
            request: post,
            expect_status: "HTTP/1.1 405 Method Not Allowed",
            expect_body: "",
        },
    ];
    let mut passed = 0;
    let mut failed = Vec::new();
    for c in &cases {
        let addr = start(lower_source(c.program).unwrap(), 0).unwrap();
        let port = addr.port();
        let (status, body) = status_and_body(&exchange(addr, &(c.request)(port)));
        if status == c.expect_status && body == c.expect_body {
            passed += 1;
        } else {
            failed.push(format!("{}: got {status:?} {body:?}", c.id));
        }
    }
    eprintln!(
        "conformance interface=loopback-http passed={passed} failed={}",
        failed.len()
    );
    assert!(failed.is_empty(), "{failed:?}");
}

#[test]
fn the_unsupported_capability_path_is_a_typed_refusal() {
    // I3: a handler that declares an effect is refused before the socket opens.
    let src = "module site\n\nrecord Request {\n    path: str,\n}\n\nrecord Response {\n    body: str,\n}\n\nfn handle(req: Request) -> Response uses net {\n    Response { body: \"no\" }\n}\n";
    let p = lower_source(src).unwrap();
    assert!(matches!(start(p, 0), Err(ServeError::BadHandler(_))));
}

#[test]
fn the_query_string_is_not_part_of_the_path() {
    // I1: the handler sees the path only; a query cannot change routing.
    let src = "module site\n\nrecord Request {\n    path: str,\n}\n\nrecord Response {\n    body: str,\n}\n\nfn handle(req: Request) -> Response {\n    Response { body: req.path }\n}\n";
    let addr = start(lower_source(src).unwrap(), 0).unwrap();
    let (status, body) = status_and_body(&exchange(addr, &get("/docs?x=1")(addr.port())));
    assert_eq!(status, "HTTP/1.1 200 OK");
    assert_eq!(body, "/docs");
}

#[test]
fn the_server_answers_after_a_refused_request() {
    // I2 and robustness: a typed failure must not stop the next request from being served.
    let addr = start(lower_source(SITE).unwrap(), 0).unwrap();
    let _ = exchange(addr, &foreign(addr.port()));
    let (status, body) = status_and_body(&exchange(addr, &get("/")(addr.port())));
    assert_eq!(status, "HTTP/1.1 200 OK");
    assert_eq!(body, "ok");
}

/// Canonical text of a run-time value, for comparison with the expected file:
/// integers and booleans as is, strings quoted, records as `Name { field: value, ... }`, unit as `unit`.
fn show(p: &aqelith::ir::Program, v: &aqelith::interp::Val) -> String {
    use aqelith::interp::Val;
    match v {
        Val::Int(i) => i.to_string(),
        Val::Bool(b) => b.to_string(),
        Val::Str(s) => format!("\"{s}\""),
        Val::Unit => "unit".to_string(),
        Val::Rec { ty, fields } => {
            let decl = &p.records[*ty as usize];
            let parts: Vec<String> = decl
                .fields
                .iter()
                .zip(fields.iter())
                .map(|((name, _), v)| format!("{name}: {}", show(p, v)))
                .collect();
            format!("{} {{ {} }}", decl.name, parts.join(", "))
        }
    }
}

#[test]
fn the_language_corpus_runs_on_the_server_target() {
    // C1: every corpus file in tests/corpus/server has an .expected file with one line per
    // zero-argument function, in source order. The expected values were derived by hand from the
    // language semantics, not by running the compiler; a mismatch fails the test.
    let dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/corpus/server");
    let mut files: Vec<_> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|x| x == "aq"))
        .collect();
    files.sort();
    assert!(!files.is_empty(), "no server corpus files");
    let mut ran = 0;
    for path in files {
        let src = std::fs::read_to_string(&path).unwrap();
        let expected = std::fs::read_to_string(path.with_extension("expected")).unwrap();
        let p = lower_source(&src).unwrap_or_else(|e| panic!("{}: {e:?}", path.display()));
        let mut got = Vec::new();
        // Every function is listed, so none is dropped silently: a function with parameters is
        // listed as needing arguments, and one with a declared effect as refused.
        for f in &p.funcs {
            if !f.params.is_empty() {
                got.push(format!("{} = needs {} argument(s)", f.name, f.params.len()));
                continue;
            }
            match aqelith::interp::run(&p, &f.name, &[]) {
                Ok(v) => {
                    got.push(format!("{} = {}", f.name, show(&p, &v)));
                    ran += 1;
                }
                Err(aqelith::interp::RunError::EffectNotAvailable { effect, .. }) => {
                    got.push(format!("{} = refused: effect {effect}", f.name));
                }
                Err(e) => panic!("{}: {}: {e:?}", path.display(), f.name),
            }
        }
        let want: Vec<&str> = expected.lines().collect();
        assert_eq!(got, want, "{}", path.display());
    }
    eprintln!("conformance corpus=C1 ran={ran}");
}
