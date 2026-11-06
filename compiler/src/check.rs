//! Name, type and effect checking for the v0 subset.
//! One diagnostic per function body (the first error in it); every function is checked.

use std::cell::RefCell;
use std::collections::BTreeMap;

use crate::ast::*;
use crate::diag::Diagnostic;

pub struct Sig {
    pub params: Vec<Ty>,
    pub ret: Ty,
    pub uses: Vec<String>,
}

/// The type of every expression of one checked function, keyed by its byte span `(start, end)`.
/// Spans are unique per expression node, so the span identifies the node (see `type_module`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FnTypes {
    pub name: String,
    pub expr: BTreeMap<(usize, usize), Ty>,
}

/// Checks a module and returns its function types, in source order, only when it has no
/// diagnostics. This is the entry point for lowering: a module the checker rejects gets no IR.
pub fn type_module(src: &str, m: &Module) -> Result<Vec<FnTypes>, Vec<Diagnostic>> {
    let (diags, types) = analyze(src, m);
    if !diags.is_empty() {
        return Err(diags);
    }
    Ok(types.into_iter().flatten().collect())
}

pub fn check_module(src: &str, m: &Module) -> Vec<Diagnostic> {
    analyze(src, m).0
}

/// Diagnostics of the module, and per function the recorded types (`None` for a function that
/// has a diagnostic). Diagnostics are the same as `check_module` returns.
fn analyze(src: &str, m: &Module) -> (Vec<Diagnostic>, Vec<Option<FnTypes>>) {
    let mut types: Vec<Option<FnTypes>> = Vec::new();
    let mut diags = Vec::new();
    let mut sigs: BTreeMap<String, Sig> = BTreeMap::new();
    for f in &m.items {
        if sigs.contains_key(&f.name) {
            diags.push(Diagnostic::new(
                src,
                "DUP001",
                format!("function '{}' is defined twice", f.name),
                f.name_start,
                f.name_end,
            ));
            continue;
        }
        sigs.insert(
            f.name.clone(),
            Sig {
                params: f.params.iter().map(|p| p.ty).collect(),
                ret: f.ret,
                uses: f.uses.clone(),
            },
        );
    }
    for f in &m.items {
        match check_fn(src, &sigs, &m.records, f) {
            Ok(expr) => types.push(Some(FnTypes {
                name: f.name.clone(),
                expr,
            })),
            Err(d) => {
                diags.push(d);
                types.push(None);
            }
        }
    }
    diags.extend(crate::pages::check_pages(src, &m.pages));
    (diags, types)
}

struct Ctx<'a> {
    src: &'a str,
    sigs: &'a BTreeMap<String, Sig>,
    records: &'a [RecordDef],
    uses: &'a [String],
    locals: Vec<(String, Ty)>,
    /// Recorded type of each expression that has been checked, by span.
    types: RefCell<BTreeMap<(usize, usize), Ty>>,
}

fn check_fn(
    src: &str,
    sigs: &BTreeMap<String, Sig>,
    records: &[RecordDef],
    f: &FnDef,
) -> Result<BTreeMap<(usize, usize), Ty>, Diagnostic> {
    for (i, p) in f.params.iter().enumerate() {
        if f.params[..i].iter().any(|q| q.name == p.name) {
            return Err(Diagnostic::new(
                src,
                "DUP001",
                format!("parameter '{}' is declared twice", p.name),
                f.name_start,
                f.name_end,
            ));
        }
    }
    let mut cx = Ctx {
        src,
        sigs,
        records,
        uses: &f.uses,
        locals: f.params.iter().map(|p| (p.name.clone(), p.ty)).collect(),
        types: RefCell::new(BTreeMap::new()),
    };
    for stmt in &f.body.stmts {
        match stmt {
            Stmt::Let {
                name, ty, value, ..
            } => {
                let vt = cx.expr(value, *ty)?;
                cx.locals.push((name.clone(), ty.unwrap_or(vt)));
            }
            Stmt::Expr(e) => {
                cx.expr(e, None)?;
            }
        }
    }
    match &f.body.tail {
        Some(t) => {
            cx.expr(t, Some(f.ret))?;
        }
        None if f.ret != Ty::Unit => {
            return Err(Diagnostic::new(
                src,
                "TYP001",
                format!("function '{}' must return {}", f.name, f.ret.name()),
                f.name_start,
                f.name_end,
            ));
        }
        None => {}
    }
    Ok(cx.types.into_inner())
}

impl Ctx<'_> {
    fn diag(&self, e: &Expr, code: &'static str, msg: impl Into<String>) -> Diagnostic {
        Diagnostic::new(self.src, code, msg, e.start, e.end)
    }

    /// Accept `actual` when no type is expected or it matches; otherwise TYP001.
    fn want(&self, e: &Expr, expected: Option<Ty>, actual: Ty) -> Result<Ty, Diagnostic> {
        match expected {
            Some(x) if x != actual => Err(self.diag(
                e,
                "TYP001",
                format!(
                    "expected {}, found {}",
                    x.type_name(self.records),
                    actual.type_name(self.records)
                ),
            )),
            _ => Ok(actual),
        }
    }

    /// Type of a binary operation's operands. A literal left operand takes the right operand's type.
    fn operands(&self, lhs: &Expr, rhs: &Expr, expected: Option<Ty>) -> Result<Ty, Diagnostic> {
        if matches!(lhs.kind, ExprKind::Int(_)) && !matches!(rhs.kind, ExprKind::Int(_)) {
            let rt = self.expr(rhs, expected)?;
            self.expr(lhs, Some(rt))?;
            Ok(rt)
        } else {
            let lt = self.expr(lhs, expected)?;
            self.expr(rhs, Some(lt))?;
            Ok(lt)
        }
    }

    /// Checks one expression and records its type. A node is recorded once, with the type it has
    /// in its context, which is what lowering needs.
    fn expr(&self, e: &Expr, expected: Option<Ty>) -> Result<Ty, Diagnostic> {
        let ty = self.expr_inner(e, expected)?;
        self.types.borrow_mut().insert((e.start, e.end), ty);
        Ok(ty)
    }

    fn expr_inner(&self, e: &Expr, expected: Option<Ty>) -> Result<Ty, Diagnostic> {
        match &e.kind {
            ExprKind::Int(text) => {
                let value: u64 = text.parse().map_err(|_| {
                    self.diag(
                        e,
                        "TYP002",
                        format!("literal {text} does not fit any supported type"),
                    )
                })?;
                let ty = match expected {
                    Some(t @ (Ty::I32 | Ty::U32)) => t,
                    Some(other) => {
                        return Err(self.diag(
                            e,
                            "TYP001",
                            format!("expected {}, found integer", other.name()),
                        ));
                    }
                    None => Ty::I32,
                };
                let max = if ty == Ty::U32 {
                    u64::from(u32::MAX)
                } else {
                    u64::from(i32::MAX as u32)
                };
                if value > max {
                    return Err(self.diag(
                        e,
                        "TYP002",
                        format!("literal {value} does not fit {}", ty.name()),
                    ));
                }
                Ok(ty)
            }
            ExprKind::Bool(_) => self.want(e, expected, Ty::Bool),
            ExprKind::Str(_) => self.want(e, expected, Ty::Str),
            ExprKind::Ident(name) => {
                let ty = self
                    .locals
                    .iter()
                    .rev()
                    .find(|(n, _)| n == name)
                    .map(|(_, t)| *t)
                    .ok_or_else(|| self.diag(e, "RES001", format!("unknown name '{name}'")))?;
                self.want(e, expected, ty)
            }
            ExprKind::Group(inner) => self.expr(inner, expected),
            ExprKind::Record { name, fields, .. } => {
                let index = self
                    .records
                    .iter()
                    .position(|r| r.name == *name)
                    .ok_or_else(|| self.diag(e, "RES001", format!("unknown record '{name}'")))?;
                let decl = &self.records[index];
                let mut set: Vec<&str> = Vec::new();
                for f in fields {
                    let (_, fty) =
                        decl.fields
                            .iter()
                            .find(|(n, _)| *n == f.name)
                            .ok_or_else(|| {
                                self.diag(
                                    e,
                                    "RES001",
                                    format!("record '{name}' has no field '{}'", f.name),
                                )
                            })?;
                    if set.contains(&f.name.as_str()) {
                        return Err(self.diag(
                            e,
                            "DUP001",
                            format!("field '{}' is set twice", f.name),
                        ));
                    }
                    set.push(&f.name);
                    self.expr(&f.value, Some(*fty))?;
                }
                if let Some((missing, _)) =
                    decl.fields.iter().find(|(n, _)| !set.contains(&n.as_str()))
                {
                    return Err(self.diag(
                        e,
                        "TYP001",
                        format!("record '{name}' is missing field '{missing}'"),
                    ));
                }
                self.want(e, expected, Ty::Rec(index as u32))
            }
            ExprKind::Field { base, name, .. } => {
                let bt = self.expr(base, None)?;
                let Ty::Rec(index) = bt else {
                    return Err(self.diag(
                        e,
                        "TYP001",
                        format!(
                            "field access needs a record, found {}",
                            bt.type_name(self.records)
                        ),
                    ));
                };
                let decl = &self.records[index as usize];
                let (_, fty) = decl.fields.iter().find(|(n, _)| n == name).ok_or_else(|| {
                    self.diag(
                        e,
                        "RES001",
                        format!("record '{}' has no field '{name}'", decl.name),
                    )
                })?;
                self.want(e, expected, *fty)
            }
            ExprKind::Call { callee, args } => {
                let sig = self.sigs.get(callee).ok_or_else(|| {
                    self.diag(e, "RES001", format!("unknown function '{callee}'"))
                })?;
                if args.len() != sig.params.len() {
                    return Err(self.diag(
                        e,
                        "TYP003",
                        format!(
                            "'{callee}' takes {} argument(s), got {}",
                            sig.params.len(),
                            args.len()
                        ),
                    ));
                }
                for eff in &sig.uses {
                    if !self.uses.contains(eff) {
                        return Err(self.diag(
                            e,
                            "EFF001",
                            format!("'{callee}' requires effect '{eff}', which the caller does not declare"),
                        ));
                    }
                }
                for (a, p) in args.iter().zip(sig.params.iter()) {
                    self.expr(a, Some(*p))?;
                }
                self.want(e, expected, sig.ret)
            }
            ExprKind::Bin { op, lhs, rhs } => match op {
                BinOp::Eq => {
                    let t = self.operands(lhs, rhs, None)?;
                    if matches!(t, Ty::Rec(_)) {
                        return Err(self.diag(e, "TYP001", "'==' is not defined on records"));
                    }
                    self.want(e, expected, Ty::Bool)
                }
                BinOp::Add | BinOp::Sub | BinOp::Mul => {
                    let t = self.operands(lhs, rhs, expected)?;
                    if !matches!(t, Ty::I32 | Ty::U32) {
                        return Err(self.diag(
                            e,
                            "TYP001",
                            format!("operator {} needs integer operands", op.symbol()),
                        ));
                    }
                    Ok(t)
                }
            },
        }
    }
}
