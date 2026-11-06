//! `aqelith-mcp`: MCP server (M8). Over stdio by default: one JSON-RPC message per line.
//! With `--http ADDR`: `POST /mcp` on a loopback address only (see mcp::serve_http).
//! Exit code 0 when input ends, 2 on a usage or I/O error.

use std::io::{self, BufRead, Write};

fn main() -> std::process::ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if let Some(pos) = args.iter().position(|a| a == "--http") {
        let Some(addr) = args.get(pos + 1) else {
            eprintln!("usage: aqelith-mcp --http 127.0.0.1:PORT");
            return std::process::ExitCode::from(2);
        };
        return serve_http(addr);
    }
    serve_stdio()
}

fn serve_stdio() -> std::process::ExitCode {
    let stdin = io::stdin();
    let mut stdout = io::stdout().lock();
    for line in stdin.lock().lines() {
        let Ok(line) = line else { break };
        if line.trim().is_empty() {
            continue;
        }
        if let Some(reply) = aqelith::mcp::handle_line(&line) {
            if writeln!(stdout, "{reply}").and_then(|()| stdout.flush()).is_err() {
                return std::process::ExitCode::from(2);
            }
        }
    }
    std::process::ExitCode::SUCCESS
}

fn serve_http(addr: &str) -> std::process::ExitCode {
    let socket: std::net::SocketAddr = match addr.parse() {
        Ok(a) => a,
        Err(e) => {
            eprintln!("invalid address '{addr}': {e}");
            return std::process::ExitCode::from(2);
        }
    };
    if !socket.ip().is_loopback() {
        eprintln!("refusing to serve on a non-loopback address; use 127.0.0.1 or ::1");
        return std::process::ExitCode::from(2);
    }
    let listener = match std::net::TcpListener::bind(socket) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("cannot bind {socket}: {e}");
            return std::process::ExitCode::from(2);
        }
    };
    eprintln!("listening on http://{socket}/mcp");
    match aqelith::mcp::serve_http(&listener, None) {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("server stopped: {e}");
            std::process::ExitCode::from(2)
        }
    }
}
