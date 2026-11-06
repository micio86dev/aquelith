//! Routing (ADR-0014): one HTML file per route, same-site links only, and no broken links.

use std::collections::BTreeMap;

use aqelith_web::{render_site, route_file, BuildError, Page, Site, View};

fn page(title: &str, body: Vec<View>) -> Page {
    Page { title: title.into(), cells: BTreeMap::new(), body: View::Element { tag: "main".into(), children: body } }
}

fn link(href: &str) -> View {
    View::Link { label: "Go".into(), href: href.into() }
}

fn two_pages() -> Site {
    Site {
        routes: vec![
            ("/".into(), page("Home", vec![link("/about")])),
            ("/about".into(), page("About", vec![link("/")])),
        ],
    }
}

#[test]
fn each_route_becomes_a_file_and_links_are_rendered() {
    let files = render_site(&two_pages()).unwrap();
    let names: Vec<&str> = files.iter().map(|(n, _)| n.as_str()).collect();
    assert_eq!(names, ["index.html", "about/index.html"]);
    assert!(files[0].1.contains(r#"<a href="/about">Go</a>"#), "{}", files[0].1);
}

#[test]
fn route_files_are_named_predictably() {
    assert_eq!(route_file("/"), "index.html");
    assert_eq!(route_file("/a/b"), "a/b/index.html");
}

#[test]
fn a_link_to_a_missing_route_is_refused() {
    let site = Site { routes: vec![("/".into(), page("Home", vec![link("/nope")]))] };
    assert_eq!(render_site(&site).unwrap_err(), BuildError::BrokenLink("/nope".into()));
}

#[test]
fn only_same_site_paths_are_accepted_as_links() {
    for href in ["https://example.com", "//example.com", "javascript:alert(1)", "/a/../b", "a/b", "/a\\b"] {
        let site = Site { routes: vec![("/".into(), page("Home", vec![link(href)]))] };
        assert_eq!(render_site(&site).unwrap_err(), BuildError::BadHref(href.into()), "{href}");
    }
}

#[test]
fn route_paths_must_be_clean_and_unique() {
    for bad in ["about", "/about/", "/a?x=1", "/a#frag", "/a/../b"] {
        let site = Site { routes: vec![(bad.into(), page("T", vec![]))] };
        assert_eq!(render_site(&site).unwrap_err(), BuildError::BadRoute(bad.into()), "{bad}");
    }
    let dup = Site { routes: vec![("/x".into(), page("A", vec![])), ("/x".into(), page("B", vec![]))] };
    assert_eq!(render_site(&dup).unwrap_err(), BuildError::BadRoute("/x".into()));
}

#[test]
fn a_link_needs_a_non_empty_label() {
    let empty = View::Link { label: " ".into(), href: "/".into() };
    let site = Site { routes: vec![("/".into(), page("Home", vec![empty]))] };
    assert_eq!(render_site(&site).unwrap_err(), BuildError::EmptyLabel);
}
