// Canonical IR, milestone S0 (docs/architecture/IR.md): lowering is defined for accepted modules only,
// is deterministic, and uses the types the checker recorded for every expression node.

use aqelith::ast::{Block, Expr, ExprKind, Module, Stmt};
use aqelith::check::type_module;
use aqelith::ir::{lower_module, lower_source, LowerError};
use std::fs;
use std::path::PathBuf;

fn corpus(kind: &str) -> Vec<(String, String)> {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/corpus")
        .join(kind);
    let mut files: Vec<(String, String)> = fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("{}: {e}", dir.display()))
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|x| x == "aq"))
        .map(|p| {
            (
                p.file_name().unwrap().to_string_lossy().into_owned(),
                fs::read_to_string(&p).unwrap(),
            )
        })
        .collect();
    files.sort();
    assert!(!files.is_empty(), "no corpus files in {kind}");
    files
}

#[test]
fn a_known_module_lowers_to_the_documented_text() {
    let src = "module demo\n\nfn add1(x: i32) -> i32 {\n    x + 1\n}\n\nfn same(a: u32) -> bool {\n    a == 3\n}\n";
    let p = lower_source(src).expect("accepted");
    assert_eq!(
        p.dump(),
        "fn add1(r0: i32) -> i32 {\n  r1: i32 = const 1\n  r2: i32 = add r0, r1\n  ret r2\n}\n\
fn same(r0: u32) -> bool {\n  r1: u32 = const 3\n  r2: bool = eq u32 r0, r1\n  ret r2\n}\n"
    );
}

#[test]
fn unit_functions_and_calls_lower_with_their_effects() {
    let src = "module m\n\nfn log() -> unit uses io {\n}\n\nfn run() -> i32 uses io {\n    log();\n    7\n}\n";
    let p = lower_source(src).expect("accepted");
    let text = p.dump();
    assert!(
        text.contains("fn log() -> unit uses io {\n  ret\n}"),
        "{text}"
    );
    assert!(text.contains("r0: unit = call log()"), "{text}");
    assert!(text.contains("ret r1"), "{text}");
}

#[test]
fn every_accepted_corpus_file_lowers_and_lowers_the_same_way_twice() {
    for (name, src) in corpus("accept") {
        let first = lower_source(&src).unwrap_or_else(|e| panic!("{name}: {e:?}"));
        let second = lower_source(&src).unwrap();
        assert_eq!(first, second, "{name}: lowering is not deterministic");
        assert_eq!(first.dump(), second.dump(), "{name}");
    }
}

#[test]
fn every_rejected_corpus_file_is_refused_and_has_no_ir() {
    for (name, src) in corpus("reject") {
        match lower_source(&src) {
            Err(LowerError::Rejected(diags)) => {
                assert!(!diags.is_empty(), "{name}: rejected without a diagnostic")
            }
            Err(other) => panic!("{name}: expected Rejected, got {other:?}"),
            Ok(p) => panic!("{name}: a rejected program lowered to {}", p.dump()),
        }
    }
}

/// Counts the expression nodes of a module, in every function body.
fn count_exprs(m: &Module) -> usize {
    fn expr(e: &Expr) -> usize {
        1 + match &e.kind {
            ExprKind::Int(_) | ExprKind::Bool(_) | ExprKind::Str(_) | ExprKind::Ident(_) => 0,
            ExprKind::Group(inner) => expr(inner),
            ExprKind::Call { args, .. } => args.iter().map(expr).sum(),
            ExprKind::Bin { lhs, rhs, .. } => expr(lhs) + expr(rhs),
        }
    }
    fn block(b: &Block) -> usize {
        let stmts: usize = b
            .stmts
            .iter()
            .map(|s| match s {
                Stmt::Let { value, .. } => expr(value),
                Stmt::Expr(e) => expr(e),
            })
            .sum();
        stmts + b.tail.as_ref().map_or(0, expr)
    }
    m.items.iter().map(|f| block(&f.body)).sum()
}

#[test]
fn every_expression_node_has_its_own_span_and_its_own_type() {
    // Lowering looks types up by span, so two nodes sharing a span would share a type silently.
    for (name, src) in corpus("accept") {
        let m = aqelith::parser::parse(&src).unwrap();
        let types = type_module(&src, &m).unwrap_or_else(|e| panic!("{name}: {e:?}"));
        let typed: usize = types.iter().map(|f| f.expr.len()).sum();
        assert_eq!(
            typed,
            count_exprs(&m),
            "{name}: spans are not unique per node"
        );
    }
}

#[test]
fn lower_module_refuses_a_module_with_diagnostics() {
    let src = "module m\n\nfn f() -> i32 {\n    true\n}\n";
    let m = aqelith::parser::parse(src).unwrap();
    assert!(matches!(
        lower_module(src, &m),
        Err(LowerError::Rejected(_))
    ));
}
