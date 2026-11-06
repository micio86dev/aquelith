//! Semantic patch API, slice v1 (M5). Specified in docs/compiler/SEMANTIC_PATCH_API.md.
//!
//! Pipeline: parse the patch, compare its base with the graph hash, check each precondition in
//! order against a copy of the module, format the candidate, and run the full checker on it.
//! Only a candidate that the checker accepts and that re-formats to itself is returned.
//! Nothing is written here; the CLI writes the candidate atomically when the patch is not a dry run.
//!
//! Targets are function names, or `id:<hex>` when an id index is given (ids.rs; function level only,
//! node-level ids are not built). Implemented ops: rename, set-return, add-effect, remove-effect,
//! delete, replace and insert (function level), and set-param-type. Expression-level edits are not implemented.

use std::path::Path;

use crate::ast::*;
use crate::diag::{jstr, Diagnostic};
use crate::ids::IdIndex;
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
    Rename {
        target: String,
        expect: String,
        to: String,
    },
    SetReturn {
        target: String,
        expect: Ty,
        to: Ty,
    },
    AddEffect {
        target: String,
        expect: Vec<String>,
        to: String,
    },
    RemoveEffect {
        target: String,
        expect: Vec<String>,
        to: String,
    },
    /// Removes the function named `target` when its content hash is `expect`.
    Delete {
        target: String,
        expect: String,
    },
    /// Replaces the function named `target` when its content hash is `expect`.
    Replace {
        target: String,
        expect: String,
        with: FnDef,
    },
    /// Inserts `with` right after the function named `target` (the anchor).
    Insert {
        target: String,
        with: FnDef,
    },
    /// Changes the type of parameter `param` of `target`, when its current type is `expect`.
    SetParamType {
        target: String,
        param: String,
        expect: Ty,
        to: Ty,
    },
}

impl Op {
    fn target_mut(&mut self) -> &mut String {
        match self {
            Op::Rename { target, .. }
            | Op::SetReturn { target, .. }
            | Op::AddEffect { target, .. }
            | Op::RemoveEffect { target, .. }
            | Op::Delete { target, .. }
            | Op::Replace { target, .. }
            | Op::Insert { target, .. }
            | Op::SetParamType { target, .. } => target,
        }
    }

    fn target(&self) -> &str {
        match self {
            Op::Rename { target, .. }
            | Op::SetReturn { target, .. }
            | Op::AddEffect { target, .. }
            | Op::RemoveEffect { target, .. }
            | Op::Delete { target, .. }
            | Op::Replace { target, .. }
            | Op::Insert { target, .. }
            | Op::SetParamType { target, .. } => target,
        }
    }
}

/// Content hash of one function: the graph hash of its canonical text on its own.
/// Used as the precondition of delete and replace; persistent ids (ids.rs) name the function.
pub fn fn_hash(f: &FnDef) -> String {
    graph_hash(&format::format_module(&Module {
        name: String::new(),
        items: vec![f.clone()],
        pages: Vec::new(),
    }))
}

/// Parses the text of one function definition for `with`. Anything but exactly one function is malformed.
fn parse_fn_source(i: usize, text: &str) -> Result<FnDef, Fail> {
    let module = parser::parse(&format!("module m\n{text}")).map_err(|d| {
        malformed(
            Some(i),
            format!("'with' is not a valid function: {}", d.message),
        )
    })?;
    if !module.pages.is_empty() {
        return Err(malformed(
            Some(i),
            "'with' must hold a function, not a page",
        ));
    }
    let mut items = module.items;
    if items.len() != 1 {
        return Err(malformed(Some(i), "'with' must hold exactly one function"));
    }
    Ok(items.remove(0))
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
    /// Renames in the order they were applied, as (old, new). An id index follows them to keep ids.
    pub renames: Vec<(String, String)>,
}

/// Boxed so that `Result` stays small; a rejection is built once per request.
type Fail = Box<Rejection>;

fn reject(code: &'static str, kind: &'static str, message: impl Into<String>) -> Fail {
    Box::new(Rejection {
        code,
        kind,
        op: None,
        expected: None,
        actual: None,
        message: message.into(),
        diagnostics: Vec::new(),
    })
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
    let base = field(doc, "base")
        .ok_or_else(|| malformed(None, "base must be a string"))?
        .to_string();
    let items = doc
        .get("ops")
        .and_then(Json::as_array)
        .ok_or_else(|| malformed(None, "ops must be an array"))?;
    let mut ops = Vec::with_capacity(items.len());
    for (i, item) in items.iter().enumerate() {
        ops.push(parse_op(i, item)?);
    }
    Ok((base, ops))
}

fn parse_op(i: usize, j: &Json) -> Result<Op, Fail> {
    let at = |msg: &str| malformed(Some(i), msg.to_string());
    let kind = field(j, "op").ok_or_else(|| at("op must be a string"))?;
    let target = field(j, "target")
        .ok_or_else(|| at("target must be a function name"))?
        .to_string();
    let no_expect = Json::Obj(Vec::new());
    let expect = j.get("expect").unwrap_or(&no_expect);
    if matches!(kind, "delete" | "replace") {
        let hash = field(expect, "hash")
            .ok_or_else(|| at("expect.hash must be a string"))?
            .to_string();
        if kind == "delete" {
            return Ok(Op::Delete {
                target,
                expect: hash,
            });
        }
        let with = parse_fn_source(
            i,
            field(j, "with").ok_or_else(|| at("with must be a function source string"))?,
        )?;
        return Ok(Op::Replace {
            target,
            expect: hash,
            with,
        });
    }
    if kind == "insert" {
        let with = parse_fn_source(
            i,
            field(j, "with").ok_or_else(|| at("with must be a function source string"))?,
        )?;
        return Ok(Op::Insert { target, with });
    }
    let to = field(j, "to").ok_or_else(|| at("to must be a string"))?;
    let effects = |key: &str| -> Result<Vec<String>, Fail> {
        let arr = expect
            .get(key)
            .and_then(Json::as_array)
            .ok_or_else(|| at("expect.uses must be an array"))?;
        arr.iter()
            .map(|v| {
                v.as_str()
                    .map(str::to_string)
                    .ok_or_else(|| at("expect.uses must hold strings"))
            })
            .collect()
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
            if !crate::parser::is_identifier(to) {
                return Err(at("rename target must be a plain identifier"));
            }
            Ok(Op::Rename {
                target,
                expect: old.to_string(),
                to: to.to_string(),
            })
        }
        "set-return" => {
            let old = field(expect, "ret")
                .and_then(Ty::from_name)
                .ok_or_else(|| at("expect.ret must be a type"))?;
            let new = Ty::from_name(to).ok_or_else(|| at(&format!("unknown type '{to}'")))?;
            Ok(Op::SetReturn {
                target,
                expect: old,
                to: new,
            })
        }
        "add-effect" => Ok(Op::AddEffect {
            target,
            expect: effects("uses")?,
            to: effect_name()?,
        }),
        "set-param-type" => {
            let param = field(expect, "param")
                .ok_or_else(|| at("expect.param must be a string"))?
                .to_string();
            let old = field(expect, "type")
                .and_then(Ty::from_name)
                .ok_or_else(|| at("expect.type must be a type"))?;
            let new = Ty::from_name(to).ok_or_else(|| at(&format!("unknown type '{to}'")))?;
            Ok(Op::SetParamType {
                target,
                param,
                expect: old,
                to: new,
            })
        }
        "remove-effect" => Ok(Op::RemoveEffect {
            target,
            expect: effects("uses")?,
            to: effect_name()?,
        }),
        other => Err(at(&format!("unknown op '{other}'"))),
    }
}

/// Apply a patch to source text and return the verified candidate, or the first rejection.
/// Targets are function names; `id:` targets need an index (see `apply_indexed`).
pub fn apply(src: &str, patch_text: &str) -> Result<Accepted, Fail> {
    apply_indexed(src, patch_text, None)
}

/// As `apply`, with an optional id index. A target `id:<hex>` is resolved to its current name once,
/// before the first op runs, so every op of one patch sees the names the patch was written against.
pub fn apply_indexed(
    src: &str,
    patch_text: &str,
    index: Option<&IdIndex>,
) -> Result<Accepted, Fail> {
    let doc = json::parse(patch_text).map_err(|e| malformed(None, format!("invalid JSON: {e}")))?;
    let (base, mut ops) = parse_ops(&doc)?;
    resolve_ids(&mut ops, index)?;

    let module = parser::parse(src).map_err(|d| {
        let mut r = reject(
            "PAT006",
            "source",
            format!("the source does not parse: {}", d.message),
        );
        r.diagnostics = vec![d];
        r
    })?;
    let current = graph_hash(&format::format_module(&module));
    if base != current {
        let mut r = reject(
            "PAT002",
            "stale",
            "the patch base is not the current graph hash; re-query and resubmit",
        );
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
        let mut r = reject(
            "PAT005",
            "validation",
            "the patched program does not pass the full checker",
        );
        r.diagnostics = diagnostics;
        return Err(r);
    }
    if format_source(&candidate).ok().as_deref() != Some(candidate.as_str()) {
        return Err(reject(
            "PAT005",
            "validation",
            "the candidate does not re-format to itself",
        ));
    }

    let mut touched: Vec<String> = Vec::new();
    for op in &ops {
        let name = match op {
            Op::Rename { to, .. } => to.clone(),
            Op::Replace { with, .. } | Op::Insert { with, .. } => with.name.clone(),
            other => other.target().to_string(),
        };
        if !touched.contains(&name) {
            touched.push(name);
        }
    }
    let renames: Vec<(String, String)> = ops
        .iter()
        .filter_map(|op| match op {
            Op::Rename { target, to, .. } => Some((target.clone(), to.clone())),
            _ => None,
        })
        .collect();
    Ok(Accepted {
        text: candidate.clone(),
        base,
        new_base: graph_hash(&candidate),
        touched,
        renames,
    })
}

fn resolve_ids(ops: &mut [Op], index: Option<&IdIndex>) -> Result<(), Fail> {
    for (i, op) in ops.iter_mut().enumerate() {
        let id = op.target_mut().strip_prefix("id:").map(str::to_string);
        let Some(id) = id else { continue };
        let Some(index) = index else {
            return Err(malformed(Some(i), "id targets need an id index"));
        };
        let Some(name) = index.name_of(&id) else {
            let mut r = reject("PAT004", "target", format!("no function with id '{id}'"));
            r.op = Some(i);
            return Err(r);
        };
        *op.target_mut() = name.to_string();
    }
    Ok(())
}

fn apply_op(m: &mut Module, op: &Op) -> Result<(), Fail> {
    let idx = m
        .items
        .iter()
        .position(|f| f.name == op.target())
        .ok_or_else(|| {
            reject(
                "PAT004",
                "target",
                format!("no function named '{}'", op.target()),
            )
        })?;
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
        Op::Delete { expect, .. } => {
            precondition(&fn_hash(&m.items[idx]), expect)?;
            m.items.remove(idx);
        }
        Op::Replace { expect, with, .. } => {
            precondition(&fn_hash(&m.items[idx]), expect)?;
            m.items[idx] = with.clone();
        }
        Op::Insert { with, .. } => {
            m.items.insert(idx + 1, with.clone());
        }
        Op::SetParamType {
            param, expect, to, ..
        } => {
            let Some(p) = m.items[idx].params.iter_mut().find(|p| p.name == *param) else {
                return Err(reject(
                    "PAT003",
                    "precondition",
                    format!("no parameter named '{param}'"),
                ));
            };
            if p.ty != *expect {
                return Err(mismatch(p.ty.name(), expect.name()));
            }
            p.ty = *to;
        }
        Op::AddEffect { expect, to, .. } => {
            let uses = &mut m.items[idx].uses;
            check_list(uses, expect)?;
            if uses.contains(to) {
                return Err(reject(
                    "PAT003",
                    "precondition",
                    format!("effect '{to}' is already declared"),
                ));
            }
            uses.push(to.clone());
        }
        Op::RemoveEffect { expect, to, .. } => {
            let uses = &mut m.items[idx].uses;
            check_list(uses, expect)?;
            if !uses.contains(to) {
                return Err(reject(
                    "PAT003",
                    "precondition",
                    format!("effect '{to}' is not declared"),
                ));
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
    let mut r = reject(
        "PAT003",
        "precondition",
        "the precondition does not hold for the current program",
    );
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
        ExprKind::Int(_) | ExprKind::Bool(_) | ExprKind::Str(_) | ExprKind::Ident(_) => {}
    }
}

/// Write text atomically: a temporary file in the same directory, then rename over the target.
pub fn write_atomic(path: &Path, text: &str) -> std::io::Result<()> {
    let dir = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let name = path
        .file_name()
        .map_or_else(|| "out".to_string(), |n| n.to_string_lossy().into_owned());
    // The temporary name carries the time, so it is not guessable; a name already taken is skipped.
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_nanos());
    for attempt in 0..16u32 {
        let tmp = dir.join(format!(
            ".{name}.aqpatch-{}-{nanos}-{attempt}.tmp",
            std::process::id()
        ));
        match write_via_temp(path, text, &tmp) {
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
            other => return other,
        }
    }
    Err(std::io::Error::new(
        std::io::ErrorKind::AlreadyExists,
        "no free temporary name",
    ))
}

/// Writes `text` to `tmp` (which must not exist; it is never followed or truncated), then renames it
/// over `path`. A temporary file that this call created is removed if the write or the rename fails.
pub fn write_via_temp(path: &Path, text: &str, tmp: &Path) -> std::io::Result<()> {
    use std::io::Write;
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(tmp)?;
    let written = file
        .write_all(text.as_bytes())
        .and_then(|()| file.sync_all());
    drop(file);
    let result = written.and_then(|()| std::fs::rename(tmp, path));
    if result.is_err() {
        let _ = std::fs::remove_file(tmp);
    }
    result
}
