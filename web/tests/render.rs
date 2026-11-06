use std::collections::BTreeMap;

use aqelith_web::{counter_page, escape_html, form_page, json_string, render_page, BuildError, Op, Page, View};

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

#[test]
fn form_input_is_labelled_bound_and_wired_to_a_set_operation() {
    let html = render_page(&form_page()).unwrap();
    assert!(html.contains("<label for=\"aq-h1\">Quantity</label>"), "{html}");
    assert!(html.contains("<input id=\"aq-h1\" type=\"number\" step=\"1\" data-handler=\"h1\" data-input=\"quantity\" value=\"1\">"), "{html}");
    assert!(html.contains("\"h1\":{\"op\":\"set\",\"cell\":\"quantity\"}"), "{html}");
    assert!(html.contains("<span data-bind=\"quantity\">1</span>"), "{html}");
}

#[test]
fn form_input_needs_a_non_empty_label() {
    let body = View::Input { label: " ".into(), cell: "count".into() };
    assert_eq!(render_page(&page_with(body)).unwrap_err(), BuildError::EmptyLabel);
}

#[test]
fn form_input_cells_must_be_declared() {
    let body = View::Input { label: "x".into(), cell: "missing".into() };
    assert_eq!(render_page(&page_with(body)).unwrap_err(), BuildError::UnknownCell("missing".into()));
}

#[test]
fn form_labels_are_escaped() {
    let body = View::Input { label: "<b>q</b>".into(), cell: "count".into() };
    let html = render_page(&page_with(body)).unwrap();
    assert!(html.contains("<label for=\"aq-h1\">&lt;b&gt;q&lt;/b&gt;</label>"), "{html}");
}

#[test]
fn an_empty_title_is_a_build_error() {
    let mut page = counter_page();
    page.title = "  ".into();
    assert_eq!(render_page(&page).unwrap_err(), BuildError::EmptyTitle);
}

#[test]
fn accessibility_findings_report_missing_h1_and_skipped_levels() {
    assert!(aqelith_web::accessibility_findings(&counter_page()).is_empty());
    let no_h1 = page_with(View::Element { tag: "p".into(), children: vec![] });
    assert_eq!(aqelith_web::accessibility_findings(&no_h1), vec!["no level-one heading (h1)".to_string()]);
    let skipped = page_with(View::Element {
        tag: "main".into(),
        children: vec![
            View::Element { tag: "h1".into(), children: vec![] },
            View::Element { tag: "h3".into(), children: vec![] },
        ],
    });
    assert_eq!(aqelith_web::accessibility_findings(&skipped), vec!["heading level skipped: h1 to h3".to_string()]);
}
