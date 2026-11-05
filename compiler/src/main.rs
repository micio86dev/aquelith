//! `aqelith` command line (v0). Exit codes: 0 success, 1 diagnostics or not found, 2 usage or I/O error.

use std::process::ExitCode;

use aqelith::{check_source, diag, parser, query};

const USAGE: &str = "usage:
  aqelith check <file> [--json]
  aqelith fmt <file> [--write]
  aqelith query <file> <function>
  aqelith explain <CODE>";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let flags: Vec<&str> = args.iter().filter(|a| a.starts_with("--")).map(String::as_str).collect();
    let pos: Vec<&str> = args.iter().filter(|a| !a.starts_with("--")).map(String::as_str).collect();
    let json = flags.contains(&"--json");
    let write = flags.contains(&"--write");
    match pos.as_slice() {
        ["check", file] => check(file, json),
        ["fmt", file] => fmt(file, write),
        ["query", file, name] => query_cmd(file, name),
        ["explain", code] => explain(code),
        _ => {
            eprintln!("{USAGE}");
            ExitCode::from(2)
        }
    }
}

fn load(file: &str) -> Result<String, ExitCode> {
    std::fs::read_to_string(file).map_err(|e| {
        eprintln!("{file}: {e}");
        ExitCode::from(2)
    })
}

fn check(file: &str, json: bool) -> ExitCode {
    let src = match load(file) {
        Ok(s) => s,
        Err(code) => return code,
    };
    let diags = check_source(&src);
    if json {
        let items: Vec<String> = diags.iter().map(|d| d.to_json()).collect();
        println!("{{\"ok\":{},\"diagnostics\":[{}]}}", diags.is_empty(), items.join(","));
    } else if diags.is_empty() {
        println!("ok");
    } else {
        for d in &diags {
            println!("{file}:{}:{}: {}: {}", d.line, d.column, d.code, d.message);
        }
    }
    if diags.is_empty() {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    }
}

fn fmt(file: &str, write: bool) -> ExitCode {
    let src = match load(file) {
        Ok(s) => s,
        Err(code) => return code,
    };
    match aqelith::format_source(&src) {
        Err(d) => {
            eprintln!("{file}:{}:{}: {}: {}", d.line, d.column, d.code, d.message);
            ExitCode::from(1)
        }
        Ok(out) if write => match std::fs::write(file, &out) {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("{file}: {e}");
                ExitCode::from(2)
            }
        },
        Ok(out) => {
            print!("{out}");
            ExitCode::SUCCESS
        }
    }
}

fn query_cmd(file: &str, name: &str) -> ExitCode {
    let src = match load(file) {
        Ok(s) => s,
        Err(code) => return code,
    };
    let module = match parser::parse(&src) {
        Ok(m) => m,
        Err(d) => {
            eprintln!("{file}:{}:{}: {}: {}", d.line, d.column, d.code, d.message);
            return ExitCode::from(1);
        }
    };
    match query::query_fn(&module, name) {
        Some(json) => {
            println!("{json}");
            ExitCode::SUCCESS
        }
        None => {
            eprintln!("no function '{name}' in {file}");
            ExitCode::from(1)
        }
    }
}

fn explain(code: &str) -> ExitCode {
    match diag::explain(code) {
        Some(text) => {
            println!("{code}: {text}");
            ExitCode::SUCCESS
        }
        None => {
            eprintln!("unknown code '{code}'");
            ExitCode::from(1)
        }
    }
}
