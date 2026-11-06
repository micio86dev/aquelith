//! Deterministic repair for missing effects (M5, docs/compiler/SEMANTIC_PATCH_API.md §8).
//!
//! A call requires the callee's effects; a caller that does not declare them gets EFF001. This
//! module computes the same requirement the checker uses and proposes the `add-effect` operations
//! that satisfy it, in a fixed order (source order of callers, then callee order of effects).
//! Adding an effect can make the caller's own callers need it too, so the proposal is repeated
//! until nothing is missing. The result is a patch document; it is verified with `patch::apply`
//! before it is returned, so a proposal that does not hold is never returned as a fix.

use crate::ast::*;
use crate::diag::{jstr, Diagnostic};
use crate::{format, parser, patch};

const MAX_ROUNDS: usize = 16;

/// Returns a patch document that makes the module pass the effect rules, or None when no
/// change is needed. Syntax errors are returned as the parser's diagnostic.
pub fn propose(src: &str) -> Result<Option<String>, String> {
    let module = parser::parse(src)
        .map_err(|d: Diagnostic| format!("{}:{}: {}: {}", d.line, d.column, d.code, d.message))?;
    let base = patch::graph_hash(&format::format_module(&module));
    let mut m = module.clone();
    let mut ops: Vec<String> = Vec::new();
    for _ in 0..MAX_ROUNDS {
        let missing = missing_effects(&m);
        if missing.is_empty() {
            break;
        }
        for (caller, effect) in missing {
            if m.items[caller].uses.contains(&effect) {
                continue;
            }
            let current = m.items[caller].uses.clone();
            ops.push(format!(
                "{{\"op\":\"add-effect\",\"target\":{},\"expect\":{{\"uses\":[{}]}},\"to\":{}}}",
                jstr(&m.items[caller].name),
                current
                    .iter()
                    .map(|u| jstr(u))
                    .collect::<Vec<_>>()
                    .join(","),
                jstr(&effect)
            ));
            m.items[caller].uses.push(effect);
        }
    }
    if ops.is_empty() {
        return Ok(None);
    }
    if !missing_effects(&m).is_empty() {
        return Err(format!("no fixed point within {MAX_ROUNDS} rounds"));
    }
    let doc = format!(
        "{{\"api\":{},\"base\":{},\"ops\":[{}]}}",
        jstr(patch::API),
        jstr(&base),
        ops.join(",")
    );
    match patch::apply(src, &doc) {
        Ok(_) => Ok(Some(doc)),
        Err(r) => Err(format!(
            "proposal rejected by the patch verifier: {}",
            r.message
        )),
    }
}

/// (caller index, effect) pairs the module requires but the caller does not declare, in source order.
fn missing_effects(m: &Module) -> Vec<(usize, String)> {
    let mut out: Vec<(usize, String)> = Vec::new();
    for (ci, caller) in m.items.iter().enumerate() {
        let mut callees: Vec<String> = Vec::new();
        block_calls(&caller.body, &mut callees);
        for callee in callees {
            let Some(def) = m.items.iter().find(|f| f.name == callee) else {
                continue;
            };
            for eff in &def.uses {
                if !caller.uses.contains(eff) && !out.iter().any(|(c, e)| *c == ci && e == eff) {
                    out.push((ci, eff.clone()));
                }
            }
        }
    }
    out
}

fn block_calls(b: &Block, out: &mut Vec<String>) {
    for s in &b.stmts {
        match s {
            Stmt::Let { value, .. } => expr_calls(value, out),
            Stmt::Expr(e) => expr_calls(e, out),
        }
    }
    if let Some(t) = &b.tail {
        expr_calls(t, out);
    }
}

fn expr_calls(e: &Expr, out: &mut Vec<String>) {
    match &e.kind {
        ExprKind::Call { callee, args } => {
            out.push(callee.clone());
            for a in args {
                expr_calls(a, out);
            }
        }
        ExprKind::Bin { lhs, rhs, .. } => {
            expr_calls(lhs, out);
            expr_calls(rhs, out);
        }
        ExprKind::Group(inner) => expr_calls(inner, out),
        ExprKind::Int(_) | ExprKind::Bool(_) | ExprKind::Ident(_) => {}
    }
}
