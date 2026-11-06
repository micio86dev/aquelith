//! Structured diagnostics with stable codes (docs/language/ERROR_MODEL.md).

/// Every code the compiler can emit, with its explanation (`aqelith explain CODE`).
pub const CODES: &[(&str, &str)] = &[
    ("LEX001", "A non-ASCII character appeared in source. v0 source is ASCII only."),
    ("LEX002", "An unexpected character appeared in source."),
    ("LEX003", "Comments are not supported in v0 (ADR-0010)."),
    ("PAR001", "The parser expected a different token at this position."),
    ("PAR002", "Nesting or expression depth exceeds the limit of 128."),
    ("PAR005", "Comparison operators are non-associative; add parentheses to chain them."),
    ("EFF002", "The effect name is not in the fixed core effect set."),
    ("TYP001", "A value's type does not match the expected type."),
    ("TYP002", "An integer literal does not fit its type."),
    ("TYP003", "A call passes the wrong number of arguments."),
    ("TYP004", "The type name is unknown."),
    ("RES001", "A name (variable or function) is not defined."),
    ("EFF001", "A call requires an effect that the caller does not declare in uses."),
    ("DUP001", "A function name is defined more than once."),
    ("PAT001", "The patch document is malformed: bad JSON, unknown api or op, or a bad value."),
    ("PAT002", "The patch base is not the current graph hash, so the patch is stale. Re-query and resubmit."),
    ("PAT003", "A precondition in the patch does not hold for the current program. Expected and actual are given."),
    ("PAT004", "The patch names a function that does not exist."),
    ("PAT005", "The patched program does not pass the full checker. The diagnostics refer to the candidate text."),
    ("PAT006", "The source does not parse, so it cannot be patched."),
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub code: &'static str,
    pub message: String,
    pub start: usize,
    pub end: usize,
    pub line: usize,
    pub column: usize,
}

impl Diagnostic {
    /// Build a diagnostic; `start` and `end` are byte offsets into `src` (source is ASCII in v0).
    pub fn new(src: &str, code: &'static str, message: impl Into<String>, start: usize, end: usize) -> Self {
        let (line, column) = locate(src, start);
        Diagnostic { code, message: message.into(), start, end, line, column }
    }

    pub fn to_json(&self) -> String {
        format!(
            "{{\"code\":{},\"message\":{},\"start\":{},\"end\":{},\"line\":{},\"column\":{}}}",
            jstr(self.code),
            jstr(&self.message),
            self.start,
            self.end,
            self.line,
            self.column
        )
    }
}

/// 1-based line and column of a byte offset.
pub fn locate(src: &str, offset: usize) -> (usize, usize) {
    let prefix = &src[..offset.min(src.len())];
    let line = prefix.matches('\n').count() + 1;
    let column = prefix.rsplit('\n').next().map_or(0, str::len) + 1;
    (line, column)
}

/// Explanation for a stable code, if the code exists.
pub fn explain(code: &str) -> Option<&'static str> {
    CODES.iter().find(|(c, _)| *c == code).map(|(_, text)| *text)
}

/// JSON string literal, quotes included.
pub fn jstr(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}
