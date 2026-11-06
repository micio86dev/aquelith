//! Name, type and effect checking for the v0 subset.
//! One diagnostic per function body (the first error in it); every function is checked.

use std::collections::BTreeMap;

use crate::ast::*;
use crate::diag::Diagnostic;

pub struct Sig {
    pub params: Vec<Ty>,
    pub ret: Ty,
    pub uses: Vec<String>,
}

pub fn check_module(src: &str, m: &Module) -> Vec<Diagnostic> {
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
        if let Err(d) = check_fn(src, &sigs, f) {
            diags.push(d);
        }
    }
    diags.extend(crate::pages::check_pages(src, &m.pages));
    diags
}

struct Ctx<'a> {
    src: &'a str,
    sigs: &'a BTreeMap<String, Sig>,
    uses: &'a [String],
    locals: Vec<(String, Ty)>,
}

fn check_fn(src: &str, sigs: &BTreeMap<String, Sig>, f: &FnDef) -> Result<(), Diagnostic> {
    let mut cx = Ctx {
        src,
        sigs,
        uses: &f.uses,
        locals: f.params.iter().map(|p| (p.name.clone(), p.ty)).collect(),
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
    Ok(())
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
                format!("expected {}, found {}", x.name(), actual.name()),
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

    fn expr(&self, e: &Expr, expected: Option<Ty>) -> Result<Ty, Diagnostic> {
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
                    self.operands(lhs, rhs, None)?;
                    self.want(e, expected, Ty::Bool)
                }
                BinOp::Add | BinOp::Sub | BinOp::Mul => {
                    let t = self.operands(lhs, rhs, expected)?;
                    if matches!(t, Ty::Bool | Ty::Unit) {
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
