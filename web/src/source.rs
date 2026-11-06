//! Builds a static site from AQELITH source (ADR-0015). The compiler parses and checks the
//! neutral page tree; this module is one backend that turns it into HTML. Other backends would
//! read the same tree without changing the language.

use aqelith::ast::{Page as SourcePage, PageItem};
use aqelith::diag::Diagnostic;

use crate::{escape_html, render_site, BuildError, Page, Site, View};

#[derive(Debug)]
pub enum SourceError {
    /// The source does not parse or does not pass the checker.
    Diagnostics(Vec<Diagnostic>),
    /// The web backend refused the rendered site (for example an unsafe route or text).
    Build(BuildError),
}

/// Parses, checks and renders the source. Returns (file path, html) for every page.
pub fn site_from_source(src: &str) -> Result<Vec<(String, String)>, SourceError> {
    let module = aqelith::parser::parse(src).map_err(|d| SourceError::Diagnostics(vec![d]))?;
    let diags = aqelith::check::check_module(src, &module);
    if !diags.is_empty() {
        return Err(SourceError::Diagnostics(diags));
    }
    let routes = module
        .pages
        .iter()
        .map(|p| (p.route.value.clone(), to_page(p)))
        .collect();
    render_site(&Site { routes }).map_err(SourceError::Build)
}

/// Same as `site_from_source`, and links `href` as a stylesheet in the head of every page.
/// Styles are a backend asset, not part of the language: the core stays target-neutral (S1).
pub fn site_from_source_with_style(
    src: &str,
    href: &str,
) -> Result<Vec<(String, String)>, SourceError> {
    let link = format!(
        "<link rel=\"stylesheet\" href=\"{}\">\n</head>",
        escape_html(href)
    );
    let files = site_from_source(src)?;
    Ok(files
        .into_iter()
        .map(|(path, html)| (path, html.replacen("</head>", &link, 1)))
        .collect())
}

/// Content-Security-Policy of a static site (S4). A static page runs no script and loads only its
/// own stylesheet and images, so every other source is `none`. Written as a meta element because
/// static hosts cannot set headers. `frame-ancestors` is not honoured in a meta element, so the
/// dev server sends it as a header instead.
pub const STATIC_CSP: &str =
    "default-src 'none'; style-src 'self'; img-src 'self'; base-uri 'none'; form-action 'none'";

/// The static build: the pages with their stylesheet link, the CSP meta element, and minified
/// HTML (S2, S4). The stylesheet itself is minified by the caller, which copies the file.
pub fn static_site(
    src: &str,
    style_href: Option<&str>,
) -> Result<Vec<(String, String)>, SourceError> {
    let files = match style_href {
        Some(href) => site_from_source_with_style(src, href)?,
        None => site_from_source(src)?,
    };
    // STATIC_CSP holds no double quote or ampersand, so it needs no escaping inside the
    // double-quoted attribute; single quotes stay literal (a `&#39;` each would cost 4 bytes).
    debug_assert!(!STATIC_CSP.contains(['"', '&', '<']));
    let csp =
        format!("<meta http-equiv=\"Content-Security-Policy\" content=\"{STATIC_CSP}\">\n</head>");
    Ok(files
        .into_iter()
        .map(|(path, html)| {
            let with_csp = html.replacen("</head>", &csp, 1);
            (path, crate::minify::minify_html(&with_csp))
        })
        .collect())
}

fn to_page(p: &SourcePage) -> Page {
    let title = p
        .items
        .iter()
        .find_map(|i| match i {
            PageItem::Title(t) => Some(t.value.clone()),
            _ => None,
        })
        .unwrap_or_default();
    let children = p
        .items
        .iter()
        .filter_map(|item| match item {
            PageItem::Title(_) => None,
            PageItem::Heading { level, text } => Some(View::Element {
                tag: format!("h{level}"),
                children: vec![View::Text(text.value.clone())],
            }),
            PageItem::Para(t) => Some(View::Element {
                tag: "p".to_string(),
                children: vec![View::Text(t.value.clone())],
            }),
            PageItem::Link { label, href } => Some(View::Link {
                label: label.value.clone(),
                href: href.value.clone(),
            }),
        })
        .collect();
    Page {
        title,
        cells: Default::default(),
        body: View::Element {
            tag: "main".to_string(),
            children,
        },
    }
}
