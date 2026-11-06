// String values in functions, milestone S2 (docs/architecture/INTERPRETER.md, section "S2").
// Strings are printable ASCII literals with equality and return only. No concatenation (S2 scope).

use aqelith::check::check_module;
use aqelith::interp::{run, Val};
use aqelith::ir::lower_source;
use aqelith::parser::parse;

fn codes(src: &str) -> Vec<&'static str> {
    let m = parse(src).unwrap_or_else(|d| panic!("parse: {d:?}"));
    check_module(src, &m).iter().map(|d| d.code).collect()
}

#[test]
fn a_string_function_returns_its_text() {
    let src = "module m\n\nfn greet() -> str {\n    \"hello\"\n}\n";
    let p = lower_source(src).expect("accepted");
    assert_eq!(run(&p, "greet", &[]), Ok(Val::Str("hello".into())));
}

#[test]
fn string_equality_compares_the_text() {
    let src = "module m\n\nfn same() -> bool {\n    \"a\" == \"a\"\n}\n\nfn differ() -> bool {\n    \"a\" == \"b\"\n}\n";
    let p = lower_source(src).expect("accepted");
    assert_eq!(run(&p, "same", &[]), Ok(Val::Bool(true)));
    assert_eq!(run(&p, "differ", &[]), Ok(Val::Bool(false)));
}

#[test]
fn a_string_passes_through_parameters_and_calls() {
    let src = "module m\n\nfn echo(s: str) -> str {\n    s\n}\n\nfn run() -> str {\n    echo(\"ok\")\n}\n";
    let p = lower_source(src).expect("accepted");
    assert_eq!(run(&p, "run", &[]), Ok(Val::Str("ok".into())));
    assert_eq!(
        run(&p, "echo", &[Val::Str("x".into())]),
        Ok(Val::Str("x".into()))
    );
}

#[test]
fn a_string_where_a_number_is_expected_is_a_type_error() {
    assert_eq!(
        codes("module m\n\nfn f() -> i32 {\n    \"x\"\n}\n"),
        vec!["TYP001"]
    );
    assert_eq!(
        codes("module m\n\nfn g() -> str {\n    1\n}\n"),
        vec!["TYP001"]
    );
}

#[test]
fn arithmetic_on_strings_is_rejected() {
    assert_eq!(
        codes("module m\n\nfn f() -> str {\n    \"a\" + \"b\"\n}\n"),
        vec!["TYP001"]
    );
}

#[test]
fn a_backslash_in_a_function_string_is_refused_with_its_code() {
    let src = "module m\n\nfn f() -> str {\n    \"a\\b\"\n}\n";
    let err = parse(src).expect_err("a backslash is not allowed in a string");
    assert_eq!(err.code, "LEX005");
    assert!(lower_source(src).is_err());
}

#[test]
fn a_quote_inside_a_function_string_ends_the_literal_and_is_refused() {
    let src = "module m\n\nfn f() -> str {\n    \"a\"b\"\n}\n";
    assert!(lower_source(src).is_err());
}

#[test]
fn the_formatter_keeps_string_literals_and_is_stable() {
    let src = "module m\n\nfn greet() -> str {\n    \"hello\"\n}\n";
    let once = aqelith::format_source(src).expect("formats");
    assert!(once.contains("\"hello\""), "{once}");
    let twice = aqelith::format_source(&once).expect("formats again");
    assert_eq!(once, twice);
}
