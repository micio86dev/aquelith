//! Semantic patch API, slice v1 (M5). Specified in docs/compiler/SEMANTIC_PATCH_API.md.
//!
//! Pipeline: parse the patch, compare its base with the graph hash, check each precondition in
//! order against a copy of the module, format the candidate, and run the full checker on it.
//! Only a candidate that the checker accepts and that re-formats to itself is returned.
//! Nothing is written here; the CLI writes the candidate atomically when the patch is not a dry run.
//!
//! Limits of this slice: targets are function names, not persistent ids (ADR-0004 is not
//! implemented in the compiler). Implemented ops: rename, set-return, add-effect, remove-effect.
//! replace, insert, delete and set-field are not implemented.

use std::path::Path;

use crate::ast::*;
use crate::diag::{jstr, Diagnostic};
use crate::json::{self, Json};
use crate::{check_source, format, format_source, parser};

pub const API: &str = "aqelith.patch.v1";

/// Graph hash: FNV-1a 64 over the canonical text. It detects stale patches; it is not a security control.
pub fn graph_hash(canonical: &str) -> String {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in canonical.bytes() {
        h ^= u64::from(b);
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("graph:{h:016x}")
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Op {
    Rename { target: String, expect: String, to: String },
    SetReturn { target: String, expect: Ty, to: Ty },
    AddEffect { target: String, expect: Vec<String>, to: String },
    RemoveEffect { target: String, expect: Vec<String>, to: String },
}

impl Op {
    fn target(&self) -> &str {
        match self {
            Op::Rename { target, .. }
            | Op::SetReturn { target, .. }
            | Op::AddEffect { target, .. }
            | Op::RemoveEffect { target, .. } => target,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rejection {
    pub code: &'static str,
    pub kind: &'static str,
    pub op: Option<usize>,
    pub expected: Option<String>,
    pub actual: Option<String>,
    pub message: String,
    /// Diagnostics of the candidate text (positions refer to the candidate, not to the source).
    pub diagnostics: Vec<Diagnostic>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Accepted {
    /// Canonical text of the candidate. It is written only when the patch is not a dry run.
    pub text: String,
    pub base: String,
    pub new_base: String,
    pub touched: Vec<String>,
}

/// Boxed so that `Result` stays small; a rejection is built once per request.
type Fail = Box<Rejection>;

fn reject(code: &'static str, kind: &'static str, message: impl Into<String>) -> Fail {
    Box::new(Rejection { code, kind, op: None, expected: None, actual: None, message: message.into(), diagnostics: Vec::new() })
}

fn malformed(op: Option<usize>, message: impl Into<String>) -> Fail {
    let mut r = reject("PAT001", "malformed", message);
    r.op = op;
    r
}

impl Rejection {
    pub fn to_json(&self) -> String {
        let diags: Vec<String> = self.diagnostics.iter().map(Diagnostic::to_json).collect();
        format!(
            "{{\"api\":{},\"status\":\"rejected\",\"kind\":{},\"code\":{},\"op\":{},\"expected\":{},\"actual\":{},\"message\":{},\"diagnostics\":[{}]}}",
            jstr(API),
            jstr(self.kind),
            jstr(self.code),
            self.op.map_or("null".to_string(), |i| i.to_string()),
            opt_json(&self.expected),
            opt_json(&self.actual),
            jstr(&self.message),
            diags.join(",")
        )
    }
}

fn opt_json(v: &Option<String>) -> String {
    v.as_ref().map_or("null".to_string(), |s| jstr(s))
}

impl Accepted {
    pub fn to_json(&self, dry_run: bool) -> String {
        let touched: Vec<String> = self.touched.iter().map(|t| jstr(t)).collect();
        format!(
            "{{\"api\":{},\"status\":{},\"base\":{},\"new_base\":{},\"touched\":[{}]}}",
            jstr(API),
            jstr(if dry_run { "dry-run" } else { "accepted" }),
            jstr(&self.base),
            jstr(&self.new_base),
            touched.join(",")
        )
    }
}

fn field<'a>(obj: &'a Json, key: &str) -> Option<&'a str> {
    obj.get(key).and_then(Json::as_str)
}

fn parse_ops(doc: &Json) -> Result<(String, Vec<Op>), Fail> {
    if field(doc, "api") != Some(API) {
        return Err(malformed(None, format!("api must be \"{API}\"")));
    }
    let base = field(doc, "base").ok_or_else(|| malformed(None, "base must be a string"))?.to_string();
    let items = doc.get("ops").and_then(Json::as_array).ok_or_else(|| malformed(None, "ops must be an array"))?;
    let mut ops = Vec::with_capacity(items.len());
    for (i, item) in items.iter().enumerate() {
        ops.push(parse_op(i, item)?);
    }
    Ok((base, ops))
}

fn parse_op(i: usize, j: &Json) -> Result<Op, Fail> {
    let at = |msg: &str| malformed(Some(i), msg.to_string());
    let kind = field(j, "op").ok_or_else(|| at("op must be a string"))?;
    let target = field(j, "target").ok_or_else(|| at("target must be a function name"))?.to_string();
    let expect = j.get("expect").ok_or_else(|| at("expect is required"))?;
    let to = field(j, "to").ok_or_else(|| at("to must be a string"))?;
    let effects = |key: &str| -> Result<Vec<String>, Fail> {
        let arr = expect.get(key).and_then(Json::as_array).ok_or_else(|| at("expect.uses must be an array"))?;
        arr.iter().map(|v| v.as_str().map(str::to_string).ok_or_else(|| at("expect.uses must hold strings"))).collect()
    };
    let effect_name = || -> Result<String, Fail> {
        if EFFECTS.contains(&to) {
            Ok(to.to_string())
        } else {
            Err(at(&format!("unknown effect '{to}'")))
        }
    };
    match kind {
        "rename" => {
            let old = field(expect, "name").ok_or_else(|| at("expect.name must be a string"))?;
            Ok(Op::Rename { target, expect: old.to_string(), to: to.to_string() })
        }
        "set-return" => {
            let old = field(expect, "ret").and_then(Ty::from_name).ok_or_else(|| at("expect.ret must be a type"))?;
            let new = Ty::from_name(to).ok_or_else(|| at(&format!("unknown type '{to}'")))?;
            Ok(Op::SetReturn { target, expect: old, to: new })
        }
        "add-effect" => Ok(Op::AddEffect { target, expect: effects("uses")?, to: effect_name()? }),
        "remove-effect" => Ok(Op::RemoveEffect { target, expect: effects("uses")?, to: effect_name()? }),
        other => Err(at(&format!("unknown op '{other}'"))),
    }
}

/// Apply a patch to source text and return the verified candidate, or the first rejection.
pub fn apply(src: &str, patch_text: &str) -> Result<Accepted, Fail> {
    let doc = json::parse(patch_text).map_err(|e| malformed(None, format!("invalid JSON: {e}")))?;
    let (base, ops) = parse_ops(&doc)?;

    let module = parser::parse(src).map_err(|d| {
        let mut r = reject("PAT006", "source", format!("the source does not parse: {}", d.message));
        r.diagnostics = vec![d];
        r
    })?;
    let current = graph_hash(&format::format_module(&module));
    if base != current {
        let mut r = reject("PAT002", "stale", "the patch base is not the current graph hash; re-query and resubmit");
        r.expected = Some(current);
        r.actual = Some(base);
        return Err(r);
    }

    let mut m = module;
    for (i, op) in ops.iter().enumerate() {
        apply_op(&mut m, op).map_err(|mut r| {
            r.op = Some(i);
            r
        })?;
    }

    let candidate = format::format_module(&m);
    let diagnostics = check_source(&candidate);
    if !diagnostics.is_empty() {
        let mut r = reject("PAT005", "validation", "the patched program does not pass the full checker");
        r.diagnostics = diagnostics;
        return Err(r);
    }
    if format_source(&candidate).ok().as_deref() != Some(candidate.as_str()) {
        return Err(reject("PAT005", "validation", "the candidate does not re-format to itself"));
    }

    let mut touched: Vec<String> = Vec::new();
    for op in &ops {
        let name = match op {
            Op::Rename { to, .. } => to.clone(),
            other => other.target().to_string(),
        };
        if !touched.contains(&name) {
            touched.push(name);
        }
    }
    Ok(Accepted { text: candidate.clone(), base, new_base: graph_hash(&candidate), touched })
}

fn apply_op(m: &mut Module, op: &Op) -> Result<(), Fail> {
    let idx = m
        .items
        .iter()
        .position(|f| f.name == op.target())
        .ok_or_else(|| reject("PAT004", "target", format!("no function named '{}'", op.target())))?;
    match op {
        Op::Rename { expect, to, .. } => {
            let current = m.items[idx].name.clone();
            precondition(&current, expect)?;
            for f in &mut m.items {
                rename_calls_block(&mut f.body, &current, to);
            }
            m.items[idx].name = to.clone();
        }
        Op::SetReturn { expect, to, .. } => {
            let current = m.items[idx].ret;
            if current != *expect {
                return Err(mismatch(current.name(), expect.name()));
            }
            m.items[idx].ret = *to;
        }
        Op::AddEffect { expect, to, .. } => {
            let uses = &mut m.items[idx].uses;
            check_list(uses, expect)?;
            if uses.contains(to) {
                return Err(reject("PAT003", "precondition", format!("effect '{to}' is already declared")));
            }
            uses.push(to.clone());
        }
        Op::RemoveEffect { expect, to, .. } => {
            let uses = &mut m.items[idx].uses;
            check_list(uses, expect)?;
            if !uses.contains(to) {
                return Err(reject("PAT003", "precondition", format!("effect '{to}' is not declared")));
            }
            uses.retain(|u| u != to);
        }
    }
    Ok(())
}

fn precondition(current: &str, expected: &str) -> Result<(), Fail> {
    if current == expected {
        Ok(())
    } else {
        Err(mismatch(current, expected))
    }
}

fn mismatch(actual: &str, expected: &str) -> Fail {
    let mut r = reject("PAT003", "precondition", "the precondition does not hold for the current program");
    r.expected = Some(expected.to_string());
    r.actual = Some(actual.to_string());
    r
}

fn check_list(current: &[String], expected: &[String]) -> Result<(), Fail> {
    if current == expected {
        Ok(())
    } else {
        Err(mismatch(&current.join(","), &expected.join(",")))
    }
}

fn rename_calls_block(b: &mut Block, old: &str, new: &str) {
    for s in &mut b.stmts {
        match s {
            Stmt::Let { value, .. } => rename_calls_expr(value, old, new),
            Stmt::Expr(e) => rename_calls_expr(e, old, new),
        }
    }
    if let Some(t) = &mut b.tail {
        rename_calls_expr(t, old, new);
    }
}

fn rename_calls_expr(e: &mut Expr, old: &str, new: &str) {
    match &mut e.kind {
        ExprKind::Call { callee, args } => {
            if callee == old {
                *callee = new.to_string();
            }
            for a in args {
                rename_calls_expr(a, old, new);
            }
        }
        ExprKind::Bin { lhs, rhs, .. } => {
            rename_calls_expr(lhs, old, new);
            rename_calls_expr(rhs, old, new);
        }
        ExprKind::Group(inner) => rename_calls_expr(inner, old, new),
        ExprKind::Int(_) | ExprKind::Bool(_) | ExprKind::Ident(_) => {}
    }
}

/// Write text atomically: a temporary file in the same directory, then rename over the target.
pub fn write_atomic(path: &Path, text: &str) -> std::io::Result<()> {
    let dir = path.parent().filter(|p| !p.as_os_str().is_empty()).unwrap_or(Path::new("."));
    let name = path.file_name().map_or_else(|| "out".to_string(), |n| n.to_string_lossy().into_owned());
    let tmp = dir.join(format!(".{name}.aqpatch-{}.tmp", std::process::id()));
    std::fs::write(&tmp, text)?;
    std::fs::rename(&tmp, path).inspect_err(|_| {
        let _ = std::fs::remove_file(&tmp);
    })
}
