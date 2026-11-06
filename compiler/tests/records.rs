// Records (S2, docs/language/RECORDS.md): declaration, literal, field access, checks, format and run.

use aqelith::check::check_module;
use aqelith::format::format_module;
use aqelith::interp::{run, Val};
use aqelith::ir::lower_source;
use aqelith::parser::parse;

fn codes(src: &str) -> Vec<&'static str> {
    let m = parse(src).unwrap_or_else(|d| panic!("parse: {d:?}"));
    check_module(src, &m).iter().map(|d| d.code).collect()
}

fn messages(src: &str) -> Vec<String> {
    let m = parse(src).unwrap();
    check_module(src, &m)
        .iter()
        .map(|d| d.message.clone())
        .collect()
}

const REQ: &str =
    "record Request {\n    path: str,\n}\n\nrecord Response {\n    body: str,\n    code: i32,\n}\n";

fn with(body: &str) -> String {
    format!("module site\n\n{REQ}\n{body}")
}

#[test]
fn a_handler_reads_a_field_and_builds_a_record() {
    let src = with(
        "fn handle(req: Request) -> Response {\n    Response { body: req.path, code: 200 }\n}\n",
    );
    let p = lower_source(&src).expect("accepted");
    let req = Val::Rec {
        ty: 0,
        fields: vec![Val::Str("/docs".into())],
    };
    assert_eq!(
        run(&p, "handle", &[req]),
        Ok(Val::Rec {
            ty: 1,
            fields: vec![Val::Str("/docs".into()), Val::Int(200)],
        })
    );
}

#[test]
fn a_literal_may_list_fields_in_any_order_and_runs_in_declaration_order() {
    let src = with("fn make() -> Response {\n    Response { code: 404, body: \"gone\" }\n}\n");
    let p = lower_source(&src).expect("accepted");
    assert_eq!(
        run(&p, "make", &[]),
        Ok(Val::Rec {
            ty: 1,
            fields: vec![Val::Str("gone".into()), Val::Int(404)],
        })
    );
}

#[test]
fn a_record_can_be_used_before_it_is_declared() {
    let src = "module m\n\nfn first(r: Later) -> str {\n    r.name\n}\n\nrecord Later {\n    name: str,\n}\n";
    assert!(codes(src).is_empty(), "{:?}", messages(src));
}

#[test]
fn a_missing_field_is_a_type_error_naming_the_field() {
    let src = with("fn make() -> Response {\n    Response { body: \"x\" }\n}\n");
    assert_eq!(codes(&src), vec!["TYP001"]);
    assert!(
        messages(&src)[0].contains("missing field 'code'"),
        "{:?}",
        messages(&src)
    );
}

#[test]
fn an_unknown_field_is_a_resolution_error() {
    let src = with("fn make() -> Response {\n    Response { body: \"x\", code: 1, extra: 2 }\n}\n");
    assert_eq!(codes(&src), vec!["RES001"]);
}

#[test]
fn a_field_set_twice_is_refused() {
    let src =
        with("fn make() -> Response {\n    Response { body: \"x\", body: \"y\", code: 1 }\n}\n");
    assert_eq!(codes(&src), vec!["DUP001"]);
}

#[test]
fn a_record_or_field_declared_twice_is_refused() {
    // A repeated record name is refused while parsing, with the same code.
    let dup_record =
        parse("module m\n\nrecord A {\n    x: i32,\n}\n\nrecord A {\n    y: i32,\n}\n");
    assert_eq!(dup_record.err().map(|d| d.code), Some("DUP001"));
    let dup_field = parse("module m\n\nrecord A {\n    x: i32,\n    x: str,\n}\n");
    assert_eq!(dup_field.err().map(|d| d.code), Some("DUP001"));
}

#[test]
fn a_wrong_field_type_names_both_types() {
    let src = with("fn make() -> Response {\n    Response { body: 7, code: 1 }\n}\n");
    assert_eq!(codes(&src), vec!["TYP001"]);
    assert!(
        messages(&src)[0].contains("expected str, found integer"),
        "{:?}",
        messages(&src)
    );
}

#[test]
fn a_record_where_a_number_is_expected_names_the_record() {
    let src = with("fn f(r: Request) -> i32 {\n    r\n}\n");
    assert_eq!(codes(&src), vec!["TYP001"]);
    assert!(
        messages(&src)[0].contains("found Request"),
        "{:?}",
        messages(&src)
    );
}

#[test]
fn records_have_no_equality() {
    let src = with("fn same(a: Request, b: Request) -> bool {\n    a == b\n}\n");
    assert_eq!(codes(&src), vec!["TYP001"]);
}

#[test]
fn a_field_access_needs_a_record_and_an_existing_field() {
    assert_eq!(
        codes(&with("fn f(a: i32) -> i32 {\n    a.x\n}\n")),
        vec!["TYP001"]
    );
    assert_eq!(
        codes(&with("fn f(r: Request) -> str {\n    r.nope\n}\n")),
        vec!["RES001"]
    );
}

#[test]
fn the_formatter_is_stable_on_records() {
    let src = with(
        "fn handle(req: Request) -> Response {\n    Response { body: req.path, code: 200 }\n}\n",
    );
    let once = aqelith::format_source(&src).expect("formats");
    assert!(
        once.starts_with("module site\n\nrecord Request {\n    path: str,\n}\n"),
        "{once}"
    );
    let twice = aqelith::format_source(&once).expect("formats again");
    assert_eq!(once, twice);
    let m = parse(&once).unwrap();
    assert_eq!(format_module(&m), once);
}

#[test]
fn a_parenthesised_base_is_printed_without_redundant_parentheses() {
    // Well-typed input: a parenthesised variable is the same expression, so the formatter drops the
    // parentheses, and the text is stable.
    let src = "module m\n\nrecord P {\n    x: i32,\n}\n\nfn f(p: P) -> i32 {\n    (p).x\n}\n";
    let once = aqelith::format_source(src).expect("formats");
    assert!(once.contains("    p.x\n"), "{once}");
    assert_eq!(aqelith::format_source(&once).unwrap(), once);
}

#[test]
fn a_record_type_change_changes_the_function_hash() {
    // A stale precondition must not pass after a parameter's record type changes (review finding).
    let hash = |ty: &str| {
        let src = format!(
            "module m\n\nrecord A {{\n    x: i32,\n}}\n\nrecord B {{\n    x: i32,\n}}\n\nfn f(p: {ty}) -> i32 {{\n    p.x\n}}\n"
        );
        let m = parse(&src).unwrap();
        aqelith::patch::fn_hash(&m.items[0], &m.records)
    };
    assert_ne!(hash("A"), hash("B"));
    assert_eq!(hash("A"), hash("A"));
}

#[test]
fn the_record_slice_does_not_change_existing_diagnostics() {
    // A function-only module keeps its diagnostics: no record appears in its messages.
    assert_eq!(
        codes("module m\n\nfn f() -> i32 {\n    true\n}\n"),
        vec!["TYP001"]
    );
}
