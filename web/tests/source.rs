//! From `.aq` source to a static site (ADR-0015): the web backend renders the neutral page tree.

use aqelith_web::{site_from_source, site_from_source_with_style, SourceError};

const LANDING: &str = "module site\n\npage home \"/\" {\n    title \"AQELITH\";\n    h1 \"Build software faster\";\n    p \"A < B & C\";\n    link \"Docs\" \"/docs\";\n}\n\npage docs \"/docs\" {\n    title \"Docs\";\n    h1 \"Documentation\";\n}\n";

fn file<'a>(files: &'a [(String, String)], name: &str) -> &'a str {
    files
        .iter()
        .find(|(f, _)| f == name)
        .map(|(_, html)| html.as_str())
        .unwrap_or_else(|| panic!("no {name}"))
}

#[test]
fn every_route_becomes_a_file() {
    let files = site_from_source(LANDING).unwrap();
    let names: Vec<&str> = files.iter().map(|(n, _)| n.as_str()).collect();
    assert_eq!(names, vec!["index.html", "docs/index.html"]);
}

#[test]
fn content_is_rendered_and_text_is_escaped() {
    let files = site_from_source(LANDING).unwrap();
    let home = file(&files, "index.html");
    assert!(home.contains("<title>AQELITH</title>"), "{home}");
    assert!(home.contains("<h1>Build software faster</h1>"), "{home}");
    assert!(home.contains("<p>A &lt; B &amp; C</p>"), "{home}");
    assert!(home.contains("<a href=\"/docs\">Docs</a>"), "{home}");
}

#[test]
fn static_pages_carry_no_script() {
    let files = site_from_source(LANDING).unwrap();
    for (name, html) in &files {
        assert!(
            !html.contains("aqelith-runtime.js"),
            "{name} loads the runtime"
        );
        assert!(!html.contains("<script"), "{name} has a script element");
    }
}

#[test]
fn every_page_links_the_stylesheet_in_its_head() {
    let files = site_from_source_with_style(LANDING, "/style.css").unwrap();
    for (name, html) in &files {
        let head = html.split("</head>").next().unwrap();
        assert!(
            head.contains("<link rel=\"stylesheet\" href=\"/style.css\">"),
            "{name} head: {head}"
        );
        assert!(
            !html.contains("aqelith-runtime.js"),
            "{name} loads the runtime"
        );
    }
}

#[test]
fn the_stylesheet_link_is_escaped() {
    let files = site_from_source_with_style(LANDING, "/a\"b.css").unwrap();
    assert!(
        files[0].1.contains("href=\"/a&quot;b.css\""),
        "{}",
        files[0].1
    );
}

#[test]
fn invalid_source_returns_its_diagnostics() {
    match site_from_source("module m\n\npage a \"/\" {\n    h1 \"no title\";\n}\n") {
        Err(SourceError::Diagnostics(d)) => assert_eq!(d[0].code, "PAGE001"),
        other => panic!("expected diagnostics, got {other:?}"),
    }
}
