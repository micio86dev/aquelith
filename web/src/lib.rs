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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Op {
    Add { cell: String, delta: i64 },
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
    }
    Ok(())
}

fn check_op(op: &Op, cells: &BTreeMap<String, i64>) -> Result<(), BuildError> {
    match op {
        Op::Add { cell, .. } if !cells.contains_key(cell) => Err(BuildError::UnknownCell(cell.clone())),
        Op::Add { .. } => Ok(()),
    }
}

/// Render a complete document. Output is deterministic for the same page.
pub fn render_page(page: &Page) -> Result<String, BuildError> {
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
