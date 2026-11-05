//! Read-only graph query over a module: callers and callees of one function (directive §10, minimal).

use std::collections::BTreeSet;

use crate::ast::*;
use crate::diag::jstr;

fn collect_calls(e: &Expr, out: &mut BTreeSet<String>) {
    match &e.kind {
        ExprKind::Call { callee, args } => {
            out.insert(callee.clone());
            for a in args {
                collect_calls(a, out);
            }
        }
        ExprKind::Bin { lhs, rhs, .. } => {
            collect_calls(lhs, out);
            collect_calls(rhs, out);
        }
        ExprKind::Group(inner) => collect_calls(inner, out),
        ExprKind::Int(_) | ExprKind::Bool(_) | ExprKind::Ident(_) => {}
    }
}

fn calls_of(f: &FnDef) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for s in &f.body.stmts {
        match s {
            Stmt::Let { value, .. } => collect_calls(value, &mut out),
            Stmt::Expr(e) => collect_calls(e, &mut out),
        }
    }
    if let Some(t) = &f.body.tail {
        collect_calls(t, &mut out);
    }
    out
}

/// JSON description of `name`; None if the function does not exist in the module.
pub fn query_fn(m: &Module, name: &str) -> Option<String> {
    let f = m.items.iter().find(|f| f.name == name)?;
    let callees: Vec<String> = calls_of(f).into_iter().collect();
    let callers: Vec<String> = m
        .items
        .iter()
        .filter(|g| g.name != name && calls_of(g).contains(name))
        .map(|g| g.name.clone())
        .collect();
    let params: Vec<String> = f
        .params
        .iter()
        .map(|p| format!("{{\"name\":{},\"type\":{}}}", jstr(&p.name), jstr(p.ty.name())))
        .collect();
    let list = |items: &[String]| items.iter().map(|s| jstr(s)).collect::<Vec<_>>().join(",");
    Some(format!(
        "{{\"name\":{},\"params\":[{}],\"return\":{},\"uses\":[{}],\"calls\":[{}],\"called_by\":[{}]}}",
        jstr(&f.name),
        params.join(","),
        jstr(f.ret.name()),
        list(&f.uses),
        list(&callees),
        list(&callers),
    ))
}
