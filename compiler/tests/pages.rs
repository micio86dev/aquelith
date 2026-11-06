//! String literals and page items (ADR-0015). Pages are target-neutral: nothing here mentions HTML.

use aqelith::lexer::{lex, Tok};
use aqelith::{check_source, format_source};

fn codes(src: &str) -> Vec<&'static str> {
    check_source(src).iter().map(|d| d.code).collect()
}

const SITE: &str = "module site\n\npage home \"/\" {\n    title \"Home\";\n    h1 \"Build faster\";\n    p \"One paragraph.\";\n    link \"Docs\" \"/docs\";\n}\n\npage docs \"/docs\" {\n    title \"Docs\";\n    h1 \"Documentation\";\n}\n";

#[test]
fn string_literal_is_one_token() {
    let toks = lex("\"hello world\"").unwrap();
    assert_eq!(toks.len(), 1);
    assert_eq!(toks[0].tok, Tok::Str("hello world".to_string()));
}

#[test]
fn unterminated_string_is_lex004() {
    assert_eq!(codes("module m\npage a \"/\" {\n    title \"open;\n}\n"), vec!["LEX004"]);
}

#[test]
fn backslash_and_control_characters_are_lex005() {
    assert_eq!(codes("module m\npage a \"/\" {\n    title \"a\\b\";\n}\n"), vec!["LEX005"]);
    assert_eq!(codes("module m\npage a \"/\" {\n    title \"a\tb\";\n}\n"), vec!["LEX005"]);
}

#[test]
fn non_ascii_inside_a_string_is_lex005() {
    assert_eq!(codes("module m\npage a \"/\" {\n    title \"caf\u{e9}\";\n}\n"), vec!["LEX005"]);
}

#[test]
fn a_valid_site_is_accepted() {
    assert!(check_source(SITE).is_empty(), "{:?}", check_source(SITE));
}

#[test]
fn a_page_without_title_is_page001() {
    assert_eq!(codes("module m\npage a \"/\" {\n    h1 \"x\";\n}\n"), vec!["PAGE001"]);
}

#[test]
fn a_title_given_twice_is_page001() {
    assert_eq!(codes("module m\npage a \"/\" {\n    title \"x\";\n    title \"y\";\n}\n"), vec!["PAGE001"]);
}

#[test]
fn empty_text_is_page002() {
    assert_eq!(codes("module m\npage a \"/\" {\n    title \"  \";\n}\n"), vec!["PAGE002"]);
}

#[test]
fn a_bad_route_is_page003() {
    assert_eq!(codes("module m\npage a \"docs\" {\n    title \"x\";\n}\n"), vec!["PAGE003"]);
    assert_eq!(codes("module m\npage a \"/docs/\" {\n    title \"x\";\n}\n"), vec!["PAGE003"]);
}

#[test]
fn a_duplicate_route_is_page004() {
    assert_eq!(
        codes("module m\npage a \"/\" {\n    title \"x\";\n}\npage b \"/\" {\n    title \"y\";\n}\n"),
        vec!["PAGE004"]
    );
}

#[test]
fn a_link_to_a_missing_route_is_page005() {
    assert_eq!(
        codes("module m\npage a \"/\" {\n    title \"x\";\n    link \"Nowhere\" \"/gone\";\n}\n"),
        vec!["PAGE005"]
    );
}

#[test]
fn a_link_with_an_external_target_is_page003() {
    assert_eq!(
        codes("module m\npage a \"/\" {\n    title \"x\";\n    link \"Out\" \"https://example.com\";\n}\n"),
        vec!["PAGE003"]
    );
}

#[test]
fn formatting_pages_is_idempotent() {
    let once = format_source(SITE).unwrap();
    assert_eq!(format_source(&once).unwrap(), once);
    assert!(once.contains("page home \"/\" {"));
}

#[test]
fn functions_and_pages_share_one_module() {
    let src = "module m\n\nfn one() -> u32 {\n    1\n}\n\npage a \"/\" {\n    title \"x\";\n}\n";
    assert!(check_source(src).is_empty(), "{:?}", check_source(src));
}

#[test]
fn a_function_patch_keeps_the_pages_of_the_module() {
    let src = "module m\n\nfn one() -> u32 {\n    1\n}\n\npage a \"/\" {\n    title \"Home\";\n    h1 \"Hi\";\n}\n";
    let base = aqelith::patch::graph_hash(&format_source(src).unwrap());
    let doc = format!(
        "{{\"api\":\"aqelith.patch.v1\",\"base\":\"{base}\",\"ops\":[{{\"op\":\"rename\",\"target\":\"one\",\"expect\":{{\"name\":\"one\"}},\"to\":\"two\"}}]}}"
    );
    let accepted = aqelith::patch::apply(src, &doc).unwrap_or_else(|r| panic!("{}", r.message));
    assert!(accepted.text.contains("fn two() -> u32 {"), "{}", accepted.text);
    assert!(accepted.text.contains("page a \"/\" {"), "{}", accepted.text);
    assert!(accepted.text.contains("h1 \"Hi\";"), "{}", accepted.text);
}
