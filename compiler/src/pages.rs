//! Checks for page items (ADR-0015). Target-neutral: the rules are about content and routes only.

use crate::ast::*;
use crate::diag::Diagnostic;

/// A route is `/` or a path of lowercase ASCII segments separated by single `/`, with no trailing
/// slash. Anything else (scheme, host, query, fragment, dot segments, spaces) is not a route.
pub fn valid_route(r: &str) -> bool {
    r == "/"
        || (r.starts_with('/')
            && !r.ends_with('/')
            && r[1..].split('/').all(|seg| {
                !seg.is_empty() && seg.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-' || b == b'_')
            }))
}

pub fn check_pages(src: &str, pages: &[Page]) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    let mut names: Vec<&str> = Vec::new();
    let mut routes: Vec<&str> = Vec::new();
    for p in pages {
        if names.contains(&p.name.as_str()) {
            diags.push(Diagnostic::new(src, "DUP001", format!("page '{}' is defined twice", p.name), p.name_start, p.name_end));
        } else {
            names.push(&p.name);
        }
        if !valid_route(&p.route.value) {
            diags.push(Diagnostic::new(src, "PAGE003", format!("'{}' is not a route", p.route.value), p.route.start, p.route.end));
        } else if routes.contains(&p.route.value.as_str()) {
            diags.push(Diagnostic::new(src, "PAGE004", format!("route '{}' is used twice", p.route.value), p.route.start, p.route.end));
        } else {
            routes.push(&p.route.value);
        }
    }
    for p in pages {
        check_page(src, p, &routes, &mut diags);
    }
    diags
}

fn check_page(src: &str, p: &Page, routes: &[&str], diags: &mut Vec<Diagnostic>) {
    let titles: Vec<&Text> = p
        .items
        .iter()
        .filter_map(|i| match i {
            PageItem::Title(t) => Some(t),
            _ => None,
        })
        .collect();
    match titles.len() {
        0 => diags.push(Diagnostic::new(src, "PAGE001", format!("page '{}' has no title", p.name), p.name_start, p.name_end)),
        1 => {}
        _ => diags.push(Diagnostic::new(src, "PAGE001", "title is given more than once", titles[1].start, titles[1].end)),
    }
    for item in &p.items {
        match item {
            PageItem::Title(t) | PageItem::Para(t) | PageItem::Heading { text: t, .. } => {
                if t.value.trim().is_empty() {
                    diags.push(Diagnostic::new(src, "PAGE002", "text must not be empty", t.start, t.end));
                }
            }
            PageItem::Link { label, href } => {
                if label.value.trim().is_empty() {
                    diags.push(Diagnostic::new(src, "PAGE002", "link label must not be empty", label.start, label.end));
                }
                if !valid_route(&href.value) {
                    diags.push(Diagnostic::new(src, "PAGE003", format!("'{}' is not a route", href.value), href.start, href.end));
                } else if !routes.contains(&href.value.as_str()) {
                    diags.push(Diagnostic::new(src, "PAGE005", format!("no page has route '{}'", href.value), href.start, href.end));
                }
            }
        }
    }
}
