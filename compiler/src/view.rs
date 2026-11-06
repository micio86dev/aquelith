//! Page view export for host platforms (desktop, see docs/platforms/DESKTOP_MACOS.md).
//! The export is a JSON description of the pages of a checked module: a host draws it, the language
//! does not know about windows or fonts.

use crate::ast::{Module, PageItem};
use crate::diag::{jstr, Diagnostic};

/// JSON of every page of a checked module. A module with any diagnostic gives the diagnostics.
pub fn view_json(src: &str) -> Result<String, Vec<Diagnostic>> {
    let m = crate::parser::parse(src).map_err(|d| vec![d])?;
    let diags = crate::check::check_module(src, &m);
    if !diags.is_empty() {
        return Err(diags);
    }
    Ok(render(&m))
}

fn render(m: &Module) -> String {
    let pages: Vec<String> = m.pages.iter().map(render_page).collect();
    format!("{{\"pages\":[{}]}}", pages.join(","))
}

fn render_page(p: &crate::ast::Page) -> String {
    let title = p
        .items
        .iter()
        .find_map(|i| match i {
            PageItem::Title(t) => Some(t.value.as_str()),
            _ => None,
        })
        .unwrap_or("");
    let items: Vec<String> = p
        .items
        .iter()
        .filter_map(|item| match item {
            PageItem::Title(_) => None,
            PageItem::Heading { level, text } => Some(format!(
                "{{\"kind\":\"heading\",\"level\":{level},\"text\":{}}}",
                jstr(&text.value)
            )),
            PageItem::Para(t) => Some(format!("{{\"kind\":\"para\",\"text\":{}}}", jstr(&t.value))),
            PageItem::Link { label, href } => Some(format!(
                "{{\"kind\":\"link\",\"label\":{},\"href\":{}}}",
                jstr(&label.value),
                jstr(&href.value)
            )),
        })
        .collect();
    format!(
        "{{\"name\":{},\"route\":{},\"title\":{},\"items\":[{}]}}",
        jstr(&p.name),
        jstr(&p.route.value),
        jstr(title),
        items.join(",")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_page_exports_its_title_and_items_in_order() {
        let src = "module m\n\npage home \"/\" {\n    title \"Home\";\n    h1 \"Hi\";\n    p \"Text.\";\n    link \"Next\" \"/n\";\n}\n\npage n \"/n\" {\n    title \"Next\";\n    h2 \"Sub\";\n}\n";
        let json = view_json(src).expect("checked");
        assert_eq!(
            json,
            "{\"pages\":[{\"name\":\"home\",\"route\":\"/\",\"title\":\"Home\",\"items\":[{\"kind\":\"heading\",\"level\":1,\"text\":\"Hi\"},{\"kind\":\"para\",\"text\":\"Text.\"},{\"kind\":\"link\",\"label\":\"Next\",\"href\":\"/n\"}]},{\"name\":\"n\",\"route\":\"/n\",\"title\":\"Next\",\"items\":[{\"kind\":\"heading\",\"level\":2,\"text\":\"Sub\"}]}]}"
        );
    }

    #[test]
    fn a_module_with_diagnostics_exports_nothing() {
        let src = "module m\n\npage home \"/\" {\n    h1 \"no title\";\n}\n";
        assert!(view_json(src).is_err());
    }
}
