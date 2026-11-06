//! Web target, first vertical slice (ADR-0012).
//! A closed view tree renders to semantic HTML. Events are declarative operations
//! carried as data (no code is shipped or evaluated). State is serialized so that it
//! cannot close its own script element (ADR-0007).

use std::collections::BTreeMap;

/// Element tags the renderer accepts. Anything else is a build error.
pub const ALLOWED_TAGS: &[&str] = &["main", "section", "header", "footer", "p", "h1", "h2", "h3", "ul", "li", "span", "div"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum View {
    Element { tag: String, children: Vec<View> },
    Text(String),
    /// Accessible name is `label`; it must not be empty.
    Button { label: String, op: Op },
    /// Displays the current value of a declared state cell.
    Bind { cell: String },
    /// Controlled integer input for one declared cell (ADR-0013). The label is its accessible name.
    Input { label: String, cell: String },
    /// Link to a route of the same site (ADR-0014). `href` must be a path starting with one `/`.
    Link { label: String, href: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Op {
    /// Adds a constant to a cell (button click).
    Add { cell: String, delta: i64 },
    /// Sets a cell to the integer the user typed (input change). Invalid text is rejected at run time.
    Set { cell: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Page {
    pub title: String,
    pub cells: BTreeMap<String, i64>,
    pub body: View,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BuildError {
    BadTag(String),
    EmptyLabel,
    UnknownCell(String),
    /// A link whose href is not a same-site path (external URLs, schemes, protocol-relative `//`).
    BadHref(String),
    /// A route that is not a clean path, or appears twice.
    BadRoute(String),
    /// A link to a route the site does not have.
    BrokenLink(String),
    /// The page title is empty; a title is required for every page (WCAG 2.4.2).
    EmptyTitle,
}

struct Out {
    html: String,
    handlers: Vec<(String, Op)>,
    next: usize,
}

fn walk(v: &View, cells: &BTreeMap<String, i64>, out: &mut Out) -> Result<(), BuildError> {
    match v {
        View::Element { tag, children } => {
            if !ALLOWED_TAGS.contains(&tag.as_str()) {
                return Err(BuildError::BadTag(tag.clone()));
            }
            out.html.push_str(&format!("<{tag}>"));
            for c in children {
                walk(c, cells, out)?;
            }
            out.html.push_str(&format!("</{tag}>"));
        }
        View::Text(t) => out.html.push_str(&escape_html(t)),
        View::Button { label, op } => {
            if label.trim().is_empty() {
                return Err(BuildError::EmptyLabel);
            }
            check_op(op, cells)?;
            out.next += 1;
            let id = format!("h{}", out.next);
            out.html.push_str(&format!(
                "<button type=\"button\" data-handler=\"{id}\">{}</button>",
                escape_html(label)
            ));
            out.handlers.push((id, op.clone()));
        }
        View::Bind { cell } => {
            let value = cells.get(cell).ok_or_else(|| BuildError::UnknownCell(cell.clone()))?;
            out.html.push_str(&format!("<span data-bind=\"{}\">{value}</span>", escape_html(cell)));
        }
        View::Link { label, href } => {
            if label.trim().is_empty() {
                return Err(BuildError::EmptyLabel);
            }
            if !is_same_site_path(href) {
                return Err(BuildError::BadHref(href.clone()));
            }
            out.html.push_str(&format!("<a href=\"{}\">{}</a>", escape_html(href), escape_html(label)));
        }
        View::Input { label, cell } => {
            if label.trim().is_empty() {
                return Err(BuildError::EmptyLabel);
            }
            let value = cells.get(cell).ok_or_else(|| BuildError::UnknownCell(cell.clone()))?;
            out.next += 1;
            let id = format!("h{}", out.next);
            let dom_id = format!("aq-{id}");
            out.html.push_str(&format!(
                "<label for=\"{dom_id}\">{}</label> <input id=\"{dom_id}\" type=\"number\" step=\"1\" \
data-handler=\"{id}\" data-input=\"{}\" value=\"{value}\">",
                escape_html(label),
                escape_html(cell)
            ));
            out.handlers.push((id, Op::Set { cell: cell.clone() }));
        }
    }
    Ok(())
}

/// A same-site path: starts with exactly one `/`, has no backslash, control character or dot segment.
fn is_same_site_path(href: &str) -> bool {
    href.starts_with('/')
        && !href.starts_with("//")
        && !href.contains('\\')
        && !href.chars().any(char::is_control)
        && !href.split('/').any(|seg| seg == "." || seg == "..")
}

/// Route paths: `/` or `/a/b` with no trailing slash, dot segments or query parts.
fn is_route(path: &str) -> bool {
    path == "/" || (is_same_site_path(path) && !path.ends_with('/') && !path.contains('?') && !path.contains('#'))
}

/// Files for one route: `/` is `index.html`; `/about` is `about/index.html`.
pub fn route_file(route: &str) -> String {
    if route == "/" {
        "index.html".to_string()
    } else {
        format!("{}/index.html", route.trim_start_matches('/'))
    }
}

/// A site: each route renders one page. Pages keep the single-page rules of `render_page`.
pub struct Site {
    pub routes: Vec<(String, Page)>,
}

/// Renders every route to a (file path, html) pair. Checks route syntax, duplicates and that every
/// link names a route of the site.
pub fn render_site(site: &Site) -> Result<Vec<(String, String)>, BuildError> {
    let mut seen: Vec<&str> = Vec::new();
    for (route, _) in &site.routes {
        if !is_route(route) || seen.contains(&route.as_str()) {
            return Err(BuildError::BadRoute(route.clone()));
        }
        seen.push(route);
    }
    let mut out = Vec::new();
    for (route, page) in &site.routes {
        // Rendering first: it validates each href's form before the site-level link check.
        let html = render_page(page)?;
        let mut links = Vec::new();
        collect_links(&page.body, &mut links);
        if let Some(missing) = links.iter().find(|href| !seen.contains(&href.as_str())) {
            return Err(BuildError::BrokenLink(missing.clone()));
        }
        out.push((route_file(route), html));
    }
    Ok(out)
}

/// Accessibility findings that are not build errors (heuristics, WCAG 1.3.1 and 2.4.6): a page
/// without a level-one heading, and a heading level that is skipped (for example h1 then h3).
/// An empty list means no finding, not that the page is accessible.
pub fn accessibility_findings(page: &Page) -> Vec<String> {
    let mut levels: Vec<u8> = Vec::new();
    collect_heading_levels(&page.body, &mut levels);
    let mut findings = Vec::new();
    if !levels.contains(&1) {
        findings.push("no level-one heading (h1)".to_string());
    }
    for pair in levels.windows(2) {
        if pair[1] > pair[0] + 1 {
            findings.push(format!("heading level skipped: h{} to h{}", pair[0], pair[1]));
        }
    }
    findings
}

fn collect_heading_levels(v: &View, out: &mut Vec<u8>) {
    match v {
        View::Element { tag, children } => {
            match tag.as_str() {
                "h1" => out.push(1),
                "h2" => out.push(2),
                "h3" => out.push(3),
                _ => {}
            }
            children.iter().for_each(|c| collect_heading_levels(c, out));
        }
        View::Text(_) | View::Button { .. } | View::Bind { .. } | View::Input { .. } | View::Link { .. } => {}
    }
}

fn collect_links(v: &View, out: &mut Vec<String>) {
    match v {
        View::Element { children, .. } => children.iter().for_each(|c| collect_links(c, out)),
        View::Link { href, .. } => out.push(href.clone()),
        View::Text(_) | View::Button { .. } | View::Bind { .. } | View::Input { .. } => {}
    }
}

fn check_op(op: &Op, cells: &BTreeMap<String, i64>) -> Result<(), BuildError> {
    let cell = match op {
        Op::Add { cell, .. } | Op::Set { cell } => cell,
    };
    if cells.contains_key(cell) {
        Ok(())
    } else {
        Err(BuildError::UnknownCell(cell.clone()))
    }
}

/// Render a complete document. Output is deterministic for the same page.
pub fn render_page(page: &Page) -> Result<String, BuildError> {
    if page.title.trim().is_empty() {
        return Err(BuildError::EmptyTitle);
    }
    let mut out = Out { html: String::new(), handlers: Vec::new(), next: 0 };
    walk(&page.body, &page.cells, &mut out)?;
    let handlers: Vec<String> = out
        .handlers
        .iter()
        .map(|(id, op)| format!("{}:{}", json_string(id), op_json(op)))
        .collect();
    let cells: Vec<String> = page.cells.iter().map(|(k, v)| format!("{}:{v}", json_string(k))).collect();
    Ok(format!(
        "<!doctype html>\n<html lang=\"en\">\n<head>\n<meta charset=\"utf-8\">\n\
<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n\
<title>{title}</title>\n</head>\n<body>\n{body}\n\
<script type=\"application/json\" id=\"aq-state\">{{{cells}}}</script>\n\
<script type=\"application/json\" id=\"aq-handlers\">{{{handlers}}}</script>\n\
<script src=\"aqelith-runtime.js\" defer></script>\n</body>\n</html>\n",
        title = escape_html(&page.title),
        body = out.html,
        cells = cells.join(","),
        handlers = handlers.join(","),
    ))
}

fn op_json(op: &Op) -> String {
    match op {
        Op::Add { cell, delta } => format!("{{\"op\":\"add\",\"cell\":{},\"delta\":{delta}}}", json_string(cell)),
        Op::Set { cell } => format!("{{\"op\":\"set\",\"cell\":{}}}", json_string(cell)),
    }
}

/// HTML text and attribute escaping.
pub fn escape_html(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            c => out.push(c),
        }
    }
    out
}

/// JSON string literal that is also safe inside an HTML script element: `<`, `>`, `&`
/// and the JavaScript line separators are written as escapes.
pub fn json_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '<' => out.push_str("\\u003c"),
            '>' => out.push_str("\\u003e"),
            '&' => out.push_str("\\u0026"),
            '\'' => out.push_str("\\u0027"),
            '\u{2028}' => out.push_str("\\u2028"),
            '\u{2029}' => out.push_str("\\u2029"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// A form slice (ADR-0013): one number input bound to a cell, shown next to a button.
pub fn form_page() -> Page {
    Page {
        title: "Quantity".to_string(),
        cells: BTreeMap::from([("quantity".to_string(), 1)]),
        body: View::Element {
            tag: "main".to_string(),
            children: vec![
                View::Element { tag: "h1".to_string(), children: vec![View::Text("Order quantity".to_string())] },
                View::Element {
                    tag: "p".to_string(),
                    children: vec![View::Text("Current: ".to_string()), View::Bind { cell: "quantity".to_string() }],
                },
                View::Input { label: "Quantity".to_string(), cell: "quantity".to_string() },
                View::Button { label: "Add one".to_string(), op: Op::Add { cell: "quantity".to_string(), delta: 1 } },
            ],
        },
    }
}

/// The first vertical slice: a counter with two buttons.
pub fn counter_page() -> Page {
    Page {
        title: "Counter".to_string(),
        cells: BTreeMap::from([("count".to_string(), 0)]),
        body: View::Element {
            tag: "main".to_string(),
            children: vec![
                View::Element { tag: "h1".to_string(), children: vec![View::Text("Counter".to_string())] },
                View::Element {
                    tag: "p".to_string(),
                    children: vec![View::Text("Value: ".to_string()), View::Bind { cell: "count".to_string() }],
                },
                View::Button { label: "Increment".to_string(), op: Op::Add { cell: "count".to_string(), delta: 1 } },
                View::Button { label: "Decrement".to_string(), op: Op::Add { cell: "count".to_string(), delta: -1 } },
            ],
        },
    }
}
