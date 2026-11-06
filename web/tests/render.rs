use std::collections::BTreeMap;

use aqelith_web::{counter_page, escape_html, json_string, render_page, BuildError, Op, Page, View};

fn page_with(body: View) -> Page {
    Page { title: "T".into(), cells: BTreeMap::from([("count".to_string(), 0)]), body }
}

#[test]
fn counter_buttons_are_semantic_and_bound_to_handlers() {
    let html = render_page(&counter_page()).unwrap();
    assert!(html.contains("<button type=\"button\" data-handler=\"h1\">Increment</button>"));
    assert!(html.contains("<button type=\"button\" data-handler=\"h2\">Decrement</button>"));
    assert!(html.contains("<span data-bind=\"count\">0</span>"));
    assert!(html.contains("id=\"aq-handlers\">{\"h1\":{\"op\":\"add\",\"cell\":\"count\",\"delta\":1},\"h2\":{\"op\":\"add\",\"cell\":\"count\",\"delta\":-1}}"));
}

#[test]
fn rendering_is_deterministic() {
    assert_eq!(render_page(&counter_page()).unwrap(), render_page(&counter_page()).unwrap());
}

#[test]
fn text_is_escaped_so_markup_cannot_be_injected() {
    let body = View::Element { tag: "p".into(), children: vec![View::Text("<img src=x onerror=alert(1)>".into())] };
    let html = render_page(&page_with(body)).unwrap();
    assert!(!html.contains("<img"), "{html}");
    assert!(html.contains("&lt;img src=x onerror=alert(1)&gt;"));
}

#[test]
fn state_cannot_close_its_script_element() {
    let mut page = counter_page();
    page.cells.insert("count".to_string(), 0);
    page.title = "</script><script>alert(1)</script>".into();
    let html = render_page(&page).unwrap();
    // Only the three known scripts exist: the two JSON islands and the external runtime.
    assert_eq!(html.matches("<script").count(), 3, "{html}");
    assert!(!html.contains("</script><script>alert"));
    assert!(html.contains("&lt;/script&gt;&lt;script&gt;alert(1)&lt;/script&gt;"));
}

#[test]
fn json_strings_escape_html_and_line_separators() {
    assert_eq!(json_string("</script>"), "\"\\u003c/script\\u003e\"");
    assert_eq!(json_string("a\u{2028}b"), "\"a\\u2028b\"");
    assert_eq!(json_string("q\"\\"), "\"q\\\"\\\\\"");
    assert_eq!(escape_html("a&b'c\""), "a&amp;b&#39;c&quot;");
}

#[test]
fn no_content_is_hidden_with_css() {
    let html = render_page(&counter_page()).unwrap();
    assert!(!html.contains("display:none"));
    assert!(!html.contains("display: none"));
    assert!(!html.contains("hidden"));
}

#[test]
fn only_the_external_runtime_is_loaded_as_code() {
    let html = render_page(&counter_page()).unwrap();
    for part in html.split("<script").skip(1) {
        let attrs = part.split('>').next().unwrap();
        assert!(attrs.contains("src=\"aqelith-runtime.js\"") || attrs.contains("type=\"application/json\""), "{attrs}");
    }
}

#[test]
fn disallowed_tags_are_rejected() {
    let body = View::Element { tag: "script".into(), children: vec![] };
    assert_eq!(render_page(&page_with(body)).unwrap_err(), BuildError::BadTag("script".into()));
}

#[test]
fn buttons_need_an_accessible_name() {
    let body = View::Button { label: "   ".into(), op: Op::Add { cell: "count".into(), delta: 1 } };
    assert_eq!(render_page(&page_with(body)).unwrap_err(), BuildError::EmptyLabel);
}

#[test]
fn unknown_cells_are_rejected_in_bindings_and_operations() {
    let bind = View::Bind { cell: "missing".into() };
    assert_eq!(render_page(&page_with(bind)).unwrap_err(), BuildError::UnknownCell("missing".into()));
    let op = View::Button { label: "x".into(), op: Op::Add { cell: "missing".into(), delta: 1 } };
    assert_eq!(render_page(&page_with(op)).unwrap_err(), BuildError::UnknownCell("missing".into()));
}
