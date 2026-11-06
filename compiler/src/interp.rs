//! Reference interpreter for the canonical IR (milestone S1, see docs/architecture/INTERPRETER.md).
//!
//! Semantics chosen for S1, and reversible until the owner says otherwise:
//! - Integer overflow is checked: an operation whose result leaves the range of its type stops the
//!   run with `RunError::Overflow`. Subtraction below zero on `u32` is an overflow too.
//! - A function that declares an effect cannot run here: no host is provided, so the call stops
//!   with `RunError::EffectNotAvailable`. Pure functions run.
//! - Recursion is bounded by `MAX_DEPTH`; deeper calls stop with `RunError::TooDeep`.

use crate::ast::{BinOp, Ty};
use crate::ir::{Func, Inst, Program, Reg, Value};

/// Most nested calls allowed in one run.
pub const MAX_DEPTH: usize = 256;
/// Most instructions executed in one run. Recursion depth alone does not bound the work: a function
/// that calls itself twice takes exponential time before it reaches the depth limit.
pub const MAX_STEPS: u64 = 1_000_000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Val {
    Int(i64),
    Bool(bool),
    Str(String),
    /// A record value: the index of its type, and its fields in declaration order (S2).
    Rec {
        ty: u32,
        fields: Vec<Val>,
    },
    Unit,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RunError {
    NoSuchFunction(String),
    ArgCount {
        func: String,
        expected: usize,
        got: usize,
    },
    ArgType {
        func: String,
        index: usize,
    },
    Overflow {
        func: String,
        ty: Ty,
    },
    EffectNotAvailable {
        func: String,
        effect: String,
    },
    TooDeep,
    /// The run executed more than `MAX_STEPS` instructions.
    OutOfFuel,
    /// The IR is not well formed. A lowered, accepted program never reaches this.
    Malformed(String),
}

/// Runs the function `name` with `args`, and returns its value.
pub fn run(p: &Program, name: &str, args: &[Val]) -> Result<Val, RunError> {
    let mut steps = MAX_STEPS;
    call(p, name, args, 0, &mut steps)
}

fn find<'a>(p: &'a Program, name: &str) -> Result<&'a Func, RunError> {
    p.funcs
        .iter()
        .find(|f| f.name == name)
        .ok_or_else(|| RunError::NoSuchFunction(name.to_string()))
}

fn call(
    p: &Program,
    name: &str,
    args: &[Val],
    depth: usize,
    steps: &mut u64,
) -> Result<Val, RunError> {
    if depth > MAX_DEPTH {
        return Err(RunError::TooDeep);
    }
    let f = find(p, name)?;
    if let Some(effect) = f.uses.first() {
        return Err(RunError::EffectNotAvailable {
            func: name.to_string(),
            effect: effect.clone(),
        });
    }
    if args.len() != f.params.len() {
        return Err(RunError::ArgCount {
            func: name.to_string(),
            expected: f.params.len(),
            got: args.len(),
        });
    }
    let mut regs: Vec<Option<Val>> = vec![None; f.regs.len()];
    for (i, (arg, ty)) in args.iter().zip(f.params.iter()).enumerate() {
        if !fits(arg, *ty) {
            return Err(RunError::ArgType {
                func: name.to_string(),
                index: i,
            });
        }
        set(&mut regs, Reg(i as u32), arg.clone())?;
    }
    run_body(p, f, &mut regs, depth, steps)
}

fn run_body(
    p: &Program,
    f: &Func,
    regs: &mut [Option<Val>],
    depth: usize,
    steps: &mut u64,
) -> Result<Val, RunError> {
    for inst in &f.body {
        if *steps == 0 {
            return Err(RunError::OutOfFuel);
        }
        *steps -= 1;
        match inst {
            Inst::Const { dst, value, .. } => {
                let v = match value {
                    Value::Int(i) => Val::Int(*i),
                    Value::Bool(b) => Val::Bool(*b),
                    Value::Str(s) => Val::Str(s.clone()),
                };
                set(regs, *dst, v)?;
            }
            Inst::Arith {
                dst,
                ty,
                op,
                lhs,
                rhs,
            } => {
                let a = int(get(regs, *lhs)?, &f.name)?;
                let b = int(get(regs, *rhs)?, &f.name)?;
                let wide = match op {
                    BinOp::Add => a.checked_add(b),
                    BinOp::Sub => a.checked_sub(b),
                    BinOp::Mul => a.checked_mul(b),
                    BinOp::Eq => {
                        return Err(RunError::Malformed(format!(
                            "{}: '==' lowered as arithmetic",
                            f.name
                        )))
                    }
                };
                let r = wide
                    .filter(|r| in_range(*r, *ty))
                    .ok_or_else(|| RunError::Overflow {
                        func: f.name.clone(),
                        ty: *ty,
                    })?;
                set(regs, *dst, Val::Int(r))?;
            }
            Inst::Equal { dst, lhs, rhs, .. } => {
                let eq = get(regs, *lhs)? == get(regs, *rhs)?;
                set(regs, *dst, Val::Bool(eq))?;
            }
            Inst::Call {
                dst, callee, args, ..
            } => {
                let vals: Vec<Val> = args
                    .iter()
                    .map(|r| get(regs, *r).cloned())
                    .collect::<Result<_, _>>()?;
                let v = call(p, callee, &vals, depth + 1, steps)?;
                set(regs, *dst, v)?;
            }
            Inst::MakeRecord { dst, ty, fields } => {
                let values: Vec<Val> = fields
                    .iter()
                    .map(|r| get(regs, *r).cloned())
                    .collect::<Result<_, _>>()?;
                let index = match ty {
                    Ty::Rec(i) => *i,
                    _ => {
                        return Err(RunError::Malformed(format!(
                            "{}: record without a record type",
                            f.name
                        )))
                    }
                };
                set(
                    regs,
                    *dst,
                    Val::Rec {
                        ty: index,
                        fields: values,
                    },
                )?;
            }
            Inst::GetField {
                dst, base, index, ..
            } => {
                let value = match get(regs, *base)? {
                    Val::Rec { fields, .. } => fields.get(*index).cloned().ok_or_else(|| {
                        RunError::Malformed(format!("{}: field index out of range", f.name))
                    })?,
                    _ => {
                        return Err(RunError::Malformed(format!(
                            "{}: field of a non-record",
                            f.name
                        )))
                    }
                };
                set(regs, *dst, value)?;
            }
            Inst::Ret(Some(r)) => return get(regs, *r).cloned(),
            Inst::Ret(None) => return Ok(Val::Unit),
        }
    }
    Err(RunError::Malformed(format!("{}: no return", f.name)))
}

fn get(regs: &[Option<Val>], r: Reg) -> Result<&Val, RunError> {
    regs.get(r.0 as usize)
        .and_then(|v| v.as_ref())
        .ok_or_else(|| RunError::Malformed(format!("register r{} read before it is set", r.0)))
}

fn set(regs: &mut [Option<Val>], r: Reg, v: Val) -> Result<(), RunError> {
    let slot = regs
        .get_mut(r.0 as usize)
        .ok_or_else(|| RunError::Malformed(format!("register r{} out of range", r.0)))?;
    *slot = Some(v);
    Ok(())
}

fn int(v: &Val, func: &str) -> Result<i64, RunError> {
    match v {
        Val::Int(i) => Ok(*i),
        _ => Err(RunError::Malformed(format!(
            "{func}: integer operand expected"
        ))),
    }
}

fn in_range(v: i64, ty: Ty) -> bool {
    match ty {
        Ty::I32 => (i64::from(i32::MIN)..=i64::from(i32::MAX)).contains(&v),
        Ty::U32 => (0..=i64::from(u32::MAX)).contains(&v),
        _ => false,
    }
}

fn fits(v: &Val, ty: Ty) -> bool {
    match (v, ty) {
        (Val::Int(i), Ty::I32 | Ty::U32) => in_range(*i, ty),
        (Val::Bool(_), Ty::Bool) => true,
        (Val::Str(_), Ty::Str) => true,
        (Val::Rec { ty: v, .. }, Ty::Rec(t)) => *v == t,
        (Val::Unit, Ty::Unit) => true,
        _ => false,
    }
}
