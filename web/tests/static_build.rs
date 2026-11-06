// The static build (S2, S4): minified output, the CSP meta element on every page, and visible text
// that is unchanged by minification.
use aqelith_web::{static_site, STATIC_CSP};

const SRC: &str = "module notes\n\
page home \"/\" {\n    title \"Notes\";\n    h1 \"Notes\";\n    p \"Text  with   spaces.\";\n    link \"Next\" \"/next\";\n}\n\
page next \"/next\" {\n    title \"Next\";\n    h1 \"Next\";\n    link \"Home\" \"/\";\n}\n";

fn page<'a>(files: &'a [(String, String)], path: &str) -> &'a str {
    files
        .iter()
        .find(|(p, _)| p == path)
        .map(|(_, html)| html.as_str())
        .unwrap_or_else(|| panic!("no page {path}"))
}

#[test]
fn every_page_carries_the_static_csp() {
    let files = static_site(SRC, Some("/style.css")).expect("builds");
    for (path, html) in &files {
        let meta =
            format!("<meta http-equiv=\"Content-Security-Policy\" content=\"{STATIC_CSP}\">");
        assert!(
            html.contains(&meta),
            "{path} lacks the CSP meta element: {html}"
        );
    }
    assert_eq!(
        STATIC_CSP,
        "default-src 'none'; style-src 'self'; img-src 'self'; base-uri 'none'; form-action 'none'"
    );
}

#[test]
fn output_is_minified_and_keeps_visible_text() {
    let files = static_site(SRC, None).expect("builds");
    let home = page(&files, "index.html");
    assert!(!home.contains("\n"), "line breaks remain: {home:?}");
    assert!(
        home.contains("<h1>Notes</h1><p>Text  with   spaces.</p>"),
        "text changed: {home}"
    );
    assert!(home.contains("<a href=\"/next\">Next</a>"), "{home}");
}

#[test]
fn static_output_has_no_inline_style_for_the_csp_to_block() {
    // style-src 'self' blocks <style> elements and style attributes. The generator emits neither.
    let files = static_site(SRC, Some("/style.css")).expect("builds");
    for (path, html) in &files {
        assert!(
            !html.contains("style=") && !html.contains("<style"),
            "{path} has inline style: {html}"
        );
    }
}

#[test]
fn no_style_means_no_stylesheet_link() {
    let files = static_site(SRC, None).expect("builds");
    assert!(!page(&files, "index.html").contains("stylesheet"));
}

#[test]
fn style_link_is_present_when_requested() {
    let files = static_site(SRC, Some("/style.css")).expect("builds");
    assert!(
        page(&files, "next/index.html").contains("<link rel=\"stylesheet\" href=\"/style.css\">")
    );
}

#[test]
fn routes_that_could_escape_the_output_directory_are_refused() {
    // The build joins each route onto the output directory. A route that is absolute or climbs out
    // of it must never reach the disk: it has to fail the checker or the renderer.
    for route in ["/../x", "//etc/x", "/a/../../x", "/x/..", "/C:/x"] {
        let src = format!(
            "module escape\npage home \"{route}\" {{\n    title \"Home\";\n    h1 \"Home\";\n}}\n"
        );
        assert!(
            static_site(&src, None).is_err(),
            "route {route} was accepted"
        );
    }
}
