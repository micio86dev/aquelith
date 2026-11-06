//! `aqelith-mcp`: MCP server over stdio (M8). One JSON-RPC message per line; see src/mcp.rs.
//! Exit code 0 when stdin closes, 2 when stdout cannot be written.

use std::io::{self, BufRead, Write};

fn main() -> std::process::ExitCode {
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
