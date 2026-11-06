//! AQELITH compiler foundation, v0 subset (M4). See docs/language/GRAMMAR.md and ADR-0009/0010.

pub mod ast;
pub mod check;
pub mod diag;
pub mod format;
pub mod ids;
pub mod interp;
pub mod ir;
pub mod json;
pub mod lexer;
pub mod mcp;
pub mod pages;
pub mod parser;
pub mod patch;
pub mod query;
pub mod repair;

pub use diag::Diagnostic;

/// Parse and check a source text. Empty result means the program is accepted.
pub fn check_source(src: &str) -> Vec<Diagnostic> {
    match parser::parse(src) {
        Err(d) => vec![d],
        Ok(m) => check::check_module(src, &m),
    }
}

/// Canonical text of a source text. Syntax only: ill-typed programs format too.
pub fn format_source(src: &str) -> Result<String, Diagnostic> {
    Ok(format::format_module(&parser::parse(src)?))
}
