//! Lexer for the v0 subset. Source must be ASCII; comments are rejected (ADR-0010).

use crate::diag::Diagnostic;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Tok {
    Ident(String),
    Int(String),
    Sym(&'static str),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub tok: Tok,
    pub start: usize,
    pub end: usize,
}

fn one_char(c: u8) -> Option<&'static str> {
    Some(match c {
        b'(' => "(",
        b')' => ")",
        b'{' => "{",
        b'}' => "}",
        b',' => ",",
        b':' => ":",
        b';' => ";",
        b'=' => "=",
        b'+' => "+",
        b'-' => "-",
        b'*' => "*",
        _ => return None,
    })
}

pub fn lex(src: &str) -> Result<Vec<Token>, Diagnostic> {
    let bytes = src.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        let c = bytes[i];
        if matches!(c, b' ' | b'\t' | b'\n' | b'\r') {
            i += 1;
            continue;
        }
        if c >= 0x80 {
            return Err(Diagnostic::new(src, "LEX001", "non-ASCII character", i, i + 1));
        }
        let start = i;
        if c.is_ascii_alphabetic() || c == b'_' {
            while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
                i += 1;
            }
            out.push(Token { tok: Tok::Ident(src[start..i].to_string()), start, end: i });
        } else if c.is_ascii_digit() {
            while i < bytes.len() && bytes[i].is_ascii_digit() {
                i += 1;
            }
            out.push(Token { tok: Tok::Int(src[start..i].to_string()), start, end: i });
        } else if src[i..].starts_with("->") {
            i += 2;
            out.push(Token { tok: Tok::Sym("->"), start, end: i });
        } else if src[i..].starts_with("==") {
            i += 2;
            out.push(Token { tok: Tok::Sym("=="), start, end: i });
        } else if c == b'/' {
            return Err(Diagnostic::new(src, "LEX003", "comments are not supported in v0", i, i + 1));
        } else if let Some(sym) = one_char(c) {
            i += 1;
            out.push(Token { tok: Tok::Sym(sym), start, end: i });
        } else {
            return Err(Diagnostic::new(src, "LEX002", format!("unexpected character '{}'", c as char), i, i + 1));
        }
    }
    Ok(out)
}
