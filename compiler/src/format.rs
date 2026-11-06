//! Canonical formatter (ADR-0010). A pure function of the syntax tree: comments do not exist in v0.

use crate::ast::*;

/// Canonical order: the module header, then functions, then pages (ADR-0015).
pub fn format_module(m: &Module) -> String {
    let mut out = format!("module {}\n", m.name);
    for f in &m.items {
        out.push('\n');
        out.push_str(&format_fn(f));
    }
    for p in &m.pages {
        out.push('\n');
        out.push_str(&format_page(p));
    }
    out
}

fn format_page(p: &Page) -> String {
    let mut out = format!("page {} \"{}\" {{\n", p.name, p.route.value);
    for item in &p.items {
        let line = match item {
            PageItem::Title(t) => format!("title \"{}\";", t.value),
            PageItem::Heading { level, text } => format!("h{level} \"{}\";", text.value),
            PageItem::Para(t) => format!("p \"{}\";", t.value),
            PageItem::Link { label, href } => format!("link \"{}\" \"{}\";", label.value, href.value),
        };
        out.push_str("    ");
        out.push_str(&line);
        out.push('\n');
    }
    out.push_str("}\n");
    out
}

fn format_fn(f: &FnDef) -> String {
    let params: Vec<String> = f.params.iter().map(|p| format!("{}: {}", p.name, p.ty.name())).collect();
    let mut head = format!("fn {}({})", f.name, params.join(", "));
    if f.ret != Ty::Unit {
        head.push_str(&format!(" -> {}", f.ret.name()));
    }
    if !f.uses.is_empty() {
        head.push_str(&format!(" uses {}", f.uses.join(", ")));
    }
    let mut out = format!("{head} {{\n");
    for s in &f.body.stmts {
        out.push_str("    ");
        out.push_str(&format_stmt(s));
        out.push('\n');
    }
    if let Some(t) = &f.body.tail {
        out.push_str("    ");
        out.push_str(&expr(t, 0));
        out.push('\n');
    }
    out.push_str("}\n");
    out
}

fn format_stmt(s: &Stmt) -> String {
    match s {
        Stmt::Let { name, mutable, ty, value } => {
            let mut_kw = if *mutable { "mut " } else { "" };
            let annot = ty.map_or(String::new(), |t| format!(": {}", t.name()));
            format!("let {mut_kw}{name}{annot} = {};", expr(value, 0))
        }
        Stmt::Expr(e) => format!("{};", expr(e, 0)),
    }
}

/// `needed` is the minimum precedence the surrounding context accepts without parentheses.
fn expr(e: &Expr, needed: u8) -> String {
    match &e.kind {
        ExprKind::Int(text) => text.parse::<u64>().map_or_else(|_| text.clone(), |v| v.to_string()),
        ExprKind::Bool(b) => b.to_string(),
        ExprKind::Ident(n) => n.clone(),
        ExprKind::Call { callee, args } => {
            let parts: Vec<String> = args.iter().map(|a| expr(a, 0)).collect();
            format!("{callee}({})", parts.join(", "))
        }
        ExprKind::Group(inner) => expr(inner, needed),
        ExprKind::Bin { op, lhs, rhs } => {
            let p = op.prec();
            // Equality is non-associative, so a left child of equal precedence also needs parentheses.
            let left_needed = if *op == BinOp::Eq { p + 1 } else { p };
            let s = format!("{} {} {}", expr(lhs, left_needed), op.symbol(), expr(rhs, p + 1));
            if p < needed {
                format!("({s})")
            } else {
                s
            }
        }
    }
}
