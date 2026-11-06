//! `aqelith` command line (v0). Exit codes: 0 success, 1 diagnostics or not found, 2 usage or I/O error.

use std::process::ExitCode;

use aqelith::ids::{self, IdIndex};
use aqelith::{check_source, diag, parser, patch, query};

const USAGE: &str = "usage:
  aqelith check <file> [--json]
  aqelith fmt <file> [--write]
  aqelith query <file> <function>
  aqelith patch <file> <patch.json> [--dry-run]
  aqelith ids <file> [--write]
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
        ["patch", file, doc] => patch_cmd(file, doc, flags.contains(&"--dry-run")),
        ["ids", file] => ids_cmd(file, write),
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

/// Sidecar index next to the source: `<file>.ids.json`.
fn index_path(file: &str) -> String {
    format!("{file}.ids.json")
}

/// A fresh id. The generator is infallible in practice; an error would exit with code 2.
fn fresh_id() -> String {
    ids::random_id().unwrap_or_else(|e| {
        eprintln!("cannot generate an id: {e}");
        std::process::exit(2)
    })
}

/// Loads the index file, or an empty index when there is none. Exits with code 2 on a bad file.
fn load_index(file: &str) -> IdIndex {
    match std::fs::read_to_string(index_path(file)) {
        Ok(text) => IdIndex::from_json(&text).unwrap_or_else(|e| {
            eprintln!("{}: {e}", index_path(file));
            std::process::exit(2)
        }),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => IdIndex::default(),
        Err(e) => {
            eprintln!("{}: {e}", index_path(file));
            std::process::exit(2)
        }
    }
}

fn patch_cmd(file: &str, doc: &str, dry_run: bool) -> ExitCode {
    let src = match load(file) {
        Ok(s) => s,
        Err(code) => return code,
    };
    let patch_text = match std::fs::read_to_string(doc) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("{doc}: {e}");
            return ExitCode::from(2);
        }
    };
    let mut index = load_index(file);
    // The index follows the source as it is now, in case the file was edited outside the index.
    if let Ok(module) = parser::parse(&src) {
        index.sync(&module, &mut fresh_id);
    }
    match patch::apply_indexed(&src, &patch_text, Some(&index)) {
        Err(rejection) => {
            println!("{}", rejection.to_json());
            ExitCode::from(1)
        }
        Ok(accepted) => {
            println!("{}", accepted.to_json(dry_run));
            if dry_run {
                return ExitCode::SUCCESS;
            }
            if let Err(e) = patch::write_atomic(std::path::Path::new(file), &accepted.text) {
                eprintln!("{file}: {e}");
                return ExitCode::from(2);
            }
            for (old, new) in &accepted.renames {
                index.rename(old, new);
            }
            // The candidate was checked, so it parses; sync keeps ids of survivors and drops the rest.
            if let Ok(module) = parser::parse(&accepted.text) {
                index.sync(&module, &mut fresh_id);
            }
            if let Err(e) = patch::write_atomic(std::path::Path::new(&index_path(file)), &index.to_json()) {
                eprintln!("{}: {e}", index_path(file));
                return ExitCode::from(2);
            }
            ExitCode::SUCCESS
        }
    }
}

fn ids_cmd(file: &str, write: bool) -> ExitCode {
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
    let mut index = load_index(file);
    index.sync(&module, &mut fresh_id);
    print!("{}", index.to_json());
    if write {
        if let Err(e) = patch::write_atomic(std::path::Path::new(&index_path(file)), &index.to_json()) {
            eprintln!("{}: {e}", index_path(file));
            return ExitCode::from(2);
        }
    }
    ExitCode::SUCCESS
}
