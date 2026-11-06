//! Canonical IR for the v0 function subset (milestone S0, see docs/architecture/IR.md).
//!
//! Lowering is only defined for a module the checker accepts: it reads the types that the checker
//! recorded for each expression, so no typing rule is repeated here. Registers are numbered in the
//! order the source is read, so the same source always lowers to the same text.
//! Page items are not lowered in S0; they keep their own renderer (`pages` and the web backend).

use std::collections::BTreeMap;
use std::fmt::Write as _;

use crate::ast::{BinOp, Expr, ExprKind, FnDef, Module, RecordDef, Stmt, Ty};
use crate::check::{type_module, FnTypes};
use crate::diag::Diagnostic;

/// A virtual register. Parameters are `r0..rN-1`; every instruction that makes a value makes a new one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Reg(pub u32);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    Int(i64),
    Bool(bool),
    Str(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Inst {
    /// `dst = value`, of type `ty`.
    Const { dst: Reg, ty: Ty, value: Value },
    /// `dst = lhs op rhs` on integers of type `ty`; the result has the same type.
    Arith {
        dst: Reg,
        ty: Ty,
        op: BinOp,
        lhs: Reg,
        rhs: Reg,
    },
    /// `dst = lhs == rhs` on operands of type `ty`; the result is `bool`.
    Equal {
        dst: Reg,
        ty: Ty,
        lhs: Reg,
        rhs: Reg,
    },
    /// `dst = callee(args)`; the result has type `ty` (`unit` when the callee returns nothing).
    Call {
        dst: Reg,
        ty: Ty,
        callee: String,
        args: Vec<Reg>,
    },
    /// `dst = Record { ... }`: the fields in declaration order, one register each (S2).
    MakeRecord { dst: Reg, ty: Ty, fields: Vec<Reg> },
    /// `dst = base.<index>`: field `index` of the record in `base`, of type `ty` (S2).
    GetField {
        dst: Reg,
        ty: Ty,
        base: Reg,
        index: usize,
    },
    /// Return a value, or nothing for `unit`.
    Ret(Option<Reg>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Func {
    pub name: String,
    pub params: Vec<Ty>,
    pub ret: Ty,
    pub uses: Vec<String>,
    /// Type of each register, indexed by `Reg.0`.
    pub regs: Vec<Ty>,
    pub body: Vec<Inst>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Program {
    /// Record types, indexed as `Ty::Rec` indexes them.
    pub records: Vec<RecordDef>,
    pub funcs: Vec<Func>,
}

#[derive(Debug)]
pub enum LowerError {
    /// The checker rejected the module; the diagnostics are its own.
    Rejected(Vec<Diagnostic>),
    /// A node that the checker should have typed has no recorded type. This is a compiler bug.
    MissingType { start: usize, end: usize },
    /// A name the checker should have resolved is not bound. This is a compiler bug.
    Unbound { name: String },
    /// A record or field the checker should have resolved is missing from the table. This is a compiler bug.
    Inconsistent { what: String },
}

/// Lowers an accepted module to IR. A module with any diagnostic gives `Rejected`.
pub fn lower_module(src: &str, m: &Module) -> Result<Program, LowerError> {
    let types = type_module(src, m).map_err(LowerError::Rejected)?;
    let mut funcs = Vec::with_capacity(m.items.len());
    for (f, ft) in m.items.iter().zip(types.iter()) {
        funcs.push(lower_fn(f, ft, &m.records)?);
    }
    Ok(Program {
        records: m.records.clone(),
        funcs,
    })
}

/// Parses and lowers source text. A parse error is reported as `Rejected`.
pub fn lower_source(src: &str) -> Result<Program, LowerError> {
    let m = crate::parser::parse(src).map_err(|d| LowerError::Rejected(vec![d]))?;
    lower_module(src, &m)
}

fn lower_fn(f: &FnDef, ft: &FnTypes, records: &[RecordDef]) -> Result<Func, LowerError> {
    let mut lw = Lowering {
        types: &ft.expr,
        records,
        regs: f.params.iter().map(|p| p.ty).collect(),
        body: Vec::new(),
        locals: f
            .params
            .iter()
            .enumerate()
            .map(|(i, p)| (p.name.clone(), Reg(i as u32)))
            .collect(),
    };
    for stmt in &f.body.stmts {
        match stmt {
            Stmt::Let { name, value, .. } => {
                let r = lw.expr(value)?;
                lw.locals.push((name.clone(), r));
            }
            Stmt::Expr(e) => {
                lw.expr(e)?;
            }
        }
    }
    match &f.body.tail {
        Some(t) => {
            let r = lw.expr(t)?;
            lw.body.push(Inst::Ret((f.ret != Ty::Unit).then_some(r)));
        }
        None => lw.body.push(Inst::Ret(None)),
    }
    Ok(Func {
        name: f.name.clone(),
        params: f.params.iter().map(|p| p.ty).collect(),
        ret: f.ret,
        uses: f.uses.clone(),
        regs: lw.regs,
        body: lw.body,
    })
}

struct Lowering<'a> {
    types: &'a BTreeMap<(usize, usize), Ty>,
    records: &'a [RecordDef],
    regs: Vec<Ty>,
    body: Vec<Inst>,
    locals: Vec<(String, Reg)>,
}

impl Lowering<'_> {
    fn fresh(&mut self, ty: Ty) -> Reg {
        let r = Reg(self.regs.len() as u32);
        self.regs.push(ty);
        r
    }

    fn ty_of(&self, e: &Expr) -> Result<Ty, LowerError> {
        self.types
            .get(&(e.start, e.end))
            .copied()
            .ok_or(LowerError::MissingType {
                start: e.start,
                end: e.end,
            })
    }

    /// Lowers an expression and returns the register that holds its value. Operands are evaluated
    /// left to right.
    fn expr(&mut self, e: &Expr) -> Result<Reg, LowerError> {
        match &e.kind {
            ExprKind::Int(text) => {
                let ty = self.ty_of(e)?;
                let value = text.parse::<i64>().map_err(|_| LowerError::MissingType {
                    start: e.start,
                    end: e.end,
                })?;
                let dst = self.fresh(ty);
                self.body.push(Inst::Const {
                    dst,
                    ty,
                    value: Value::Int(value),
                });
                Ok(dst)
            }
            ExprKind::Bool(b) => {
                let dst = self.fresh(Ty::Bool);
                self.body.push(Inst::Const {
                    dst,
                    ty: Ty::Bool,
                    value: Value::Bool(*b),
                });
                Ok(dst)
            }
            ExprKind::Str(s) => {
                let dst = self.fresh(Ty::Str);
                self.body.push(Inst::Const {
                    dst,
                    ty: Ty::Str,
                    value: Value::Str(s.clone()),
                });
                Ok(dst)
            }
            ExprKind::Record { name, fields, .. } => {
                let ty = self.ty_of(e)?;
                let index = match ty {
                    Ty::Rec(i) => i as usize,
                    _ => {
                        return Err(LowerError::Inconsistent {
                            what: format!("record literal '{name}' has no record type"),
                        })
                    }
                };
                // Values are evaluated in the order they are written; the record holds them in
                // declaration order.
                let mut written: Vec<(usize, Reg)> = Vec::with_capacity(fields.len());
                for f in fields {
                    let pos = self.records[index]
                        .fields
                        .iter()
                        .position(|(n, _)| *n == f.name)
                        .ok_or_else(|| LowerError::Unbound {
                            name: f.name.clone(),
                        })?;
                    written.push((pos, self.expr(&f.value)?));
                }
                let mut regs = vec![None; self.records[index].fields.len()];
                for (pos, r) in written {
                    regs[pos] = Some(r);
                }
                let fields: Vec<Reg> = regs
                    .into_iter()
                    .collect::<Option<Vec<Reg>>>()
                    .ok_or_else(|| LowerError::Unbound { name: name.clone() })?;
                let dst = self.fresh(ty);
                self.body.push(Inst::MakeRecord { dst, ty, fields });
                Ok(dst)
            }
            ExprKind::Field { base, name, .. } => {
                let base_reg = self.expr(base)?;
                let index = match self.ty_of(base)? {
                    Ty::Rec(i) => self.records[i as usize]
                        .fields
                        .iter()
                        .position(|(n, _)| n == name)
                        .ok_or_else(|| LowerError::Inconsistent {
                            what: format!("record has no field '{name}'"),
                        })?,
                    _ => {
                        return Err(LowerError::Inconsistent {
                            what: format!("field '{name}' read from a non-record"),
                        })
                    }
                };
                let ty = self.ty_of(e)?;
                let dst = self.fresh(ty);
                self.body.push(Inst::GetField {
                    dst,
                    ty,
                    base: base_reg,
                    index,
                });
                Ok(dst)
            }
            ExprKind::Ident(name) => self
                .locals
                .iter()
                .rev()
                .find(|(n, _)| n == name)
                .map(|(_, r)| *r)
                .ok_or_else(|| LowerError::Unbound { name: name.clone() }),
            ExprKind::Group(inner) => self.expr(inner),
            ExprKind::Call { callee, args } => {
                let mut regs = Vec::with_capacity(args.len());
                for a in args {
                    regs.push(self.expr(a)?);
                }
                let ty = self.ty_of(e)?;
                let dst = self.fresh(ty);
                self.body.push(Inst::Call {
                    dst,
                    ty,
                    callee: callee.clone(),
                    args: regs,
                });
                Ok(dst)
            }
            ExprKind::Bin { op, lhs, rhs } => {
                let l = self.expr(lhs)?;
                let r = self.expr(rhs)?;
                match op {
                    BinOp::Eq => {
                        let ty = self.ty_of(lhs)?;
                        let dst = self.fresh(Ty::Bool);
                        self.body.push(Inst::Equal {
                            dst,
                            ty,
                            lhs: l,
                            rhs: r,
                        });
                        Ok(dst)
                    }
                    BinOp::Add | BinOp::Sub | BinOp::Mul => {
                        let ty = self.ty_of(e)?;
                        let dst = self.fresh(ty);
                        self.body.push(Inst::Arith {
                            dst,
                            ty,
                            op: *op,
                            lhs: l,
                            rhs: r,
                        });
                        Ok(dst)
                    }
                }
            }
        }
    }
}

impl Program {
    /// Canonical text of the program. Deterministic: the same program always prints the same.
    pub fn dump(&self) -> String {
        let mut out = String::new();
        for f in &self.funcs {
            let params: Vec<String> = f
                .params
                .iter()
                .enumerate()
                .map(|(i, t)| format!("r{i}: {}", t.type_name(&self.records)))
                .collect();
            let uses = if f.uses.is_empty() {
                String::new()
            } else {
                format!(" uses {}", f.uses.join(" "))
            };
            let _ = writeln!(
                out,
                "fn {}({}) -> {}{} {{",
                f.name,
                params.join(", "),
                f.ret.type_name(&self.records),
                uses
            );
            for inst in &f.body {
                let line = match inst {
                    Inst::Const { dst, ty, value } => {
                        let v = match value {
                            Value::Int(i) => i.to_string(),
                            Value::Bool(b) => b.to_string(),
                            Value::Str(s) => format!("\"{s}\""),
                        };
                        format!("{}: {} = const {v}", reg(*dst), ty.type_name(&self.records))
                    }
                    Inst::Arith {
                        dst,
                        ty,
                        op,
                        lhs,
                        rhs,
                    } => format!(
                        "{}: {} = {} {}, {}",
                        reg(*dst),
                        ty.type_name(&self.records),
                        op_name(*op),
                        reg(*lhs),
                        reg(*rhs)
                    ),
                    Inst::Equal { dst, ty, lhs, rhs } => format!(
                        "{}: bool = eq {} {}, {}",
                        reg(*dst),
                        ty.type_name(&self.records),
                        reg(*lhs),
                        reg(*rhs)
                    ),
                    Inst::Call {
                        dst,
                        ty,
                        callee,
                        args,
                    } => {
                        let a: Vec<String> = args.iter().map(|r| reg(*r)).collect();
                        format!(
                            "{}: {} = call {callee}({})",
                            reg(*dst),
                            ty.type_name(&self.records),
                            a.join(", ")
                        )
                    }
                    Inst::Ret(Some(r)) => format!("ret {}", reg(*r)),
                    Inst::MakeRecord { dst, ty, fields } => {
                        let parts: Vec<String> = fields.iter().map(|r| reg(*r)).collect();
                        format!(
                            "{}: {} = record {{ {} }}",
                            reg(*dst),
                            ty.type_name(&self.records),
                            parts.join(", ")
                        )
                    }
                    Inst::GetField {
                        dst,
                        ty,
                        base,
                        index,
                    } => format!(
                        "{}: {} = field {}.{index}",
                        reg(*dst),
                        ty.type_name(&self.records),
                        reg(*base)
                    ),
                    Inst::Ret(None) => "ret".to_string(),
                };
                let _ = writeln!(out, "  {line}");
            }
            out.push_str("}\n");
        }
        out
    }
}

fn reg(r: Reg) -> String {
    format!("r{}", r.0)
}

fn op_name(op: BinOp) -> &'static str {
    match op {
        BinOp::Add => "add",
        BinOp::Sub => "sub",
        BinOp::Mul => "mul",
        BinOp::Eq => "eq",
    }
}
