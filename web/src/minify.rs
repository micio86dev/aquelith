//! Conservative minifiers for the static build (S2). They remove only what cannot change what a
//! browser shows: HTML comments and line breaks between tags, and CSS comments and whitespace
//! next to punctuation. Text, headings, links, attribute values and CSS string literals are kept
//! byte for byte. Spaces between two inline elements are kept, because they are visible.

/// Minifies HTML written by the site generator. Whitespace that contains a line break and sits
/// between a `>` and a `<` is dropped (indentation), a single space is kept (it can be visible),
/// and `<!-- ... -->` comments are removed.
pub fn minify_html(html: &str) -> String {
    let without_comments = strip_html_comments(html);
    let mut out = String::with_capacity(without_comments.len());
    let mut rest = without_comments.as_str();
    while let Some(i) = rest.find('>') {
        out.push_str(&rest[..=i]);
        rest = &rest[i + 1..];
        let ws = &rest[..rest.len() - rest.trim_start().len()];
        if ws.contains('\n') && rest[ws.len()..].starts_with('<') {
            rest = &rest[ws.len()..];
        }
    }
    // Whitespace after the last tag is layout only; it is dropped.
    if !rest.trim().is_empty() {
        out.push_str(rest);
    }
    out
}

fn strip_html_comments(html: &str) -> String {
    let mut out = String::with_capacity(html.len());
    let mut rest = html;
    while let Some(start) = rest.find("<!--") {
        out.push_str(&rest[..start]);
        match rest[start..].find("-->") {
            Some(end) => rest = &rest[start + end + 3..],
            None => {
                // Unterminated comment: keep the remainder unchanged rather than guess.
                rest = &rest[start..];
                break;
            }
        }
    }
    out.push_str(rest);
    out
}

/// Characters after which a space is never needed in CSS, and characters before which it is not.
const NO_SPACE_AFTER: &[char] = &['{', '}', ';', ',', ':'];
const NO_SPACE_BEFORE: &[char] = &['{', '}', ';', ','];

/// Minifies a stylesheet: removes `/* */` comments, collapses whitespace to one space, and drops
/// spaces around `{ } ; ,` and after `:`. Spaces are kept elsewhere, because `a :hover` (a
/// descendant selector) and `calc(1px + 2px)` change meaning without them. String literals are
/// copied unchanged, escapes included.
pub fn minify_css(css: &str) -> String {
    let chars: Vec<char> = css.chars().collect();
    let mut out = String::with_capacity(css.len());
    let mut pending_space = false;
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c.is_whitespace() {
            pending_space = true;
            i += 1;
            continue;
        }
        if c == '/' && chars.get(i + 1) == Some(&'*') {
            let mut j = i + 2;
            while j + 1 < chars.len() && !(chars[j] == '*' && chars[j + 1] == '/') {
                j += 1;
            }
            i = (j + 2).min(chars.len());
            pending_space = true;
            continue;
        }
        if pending_space {
            let after_ok = match out.chars().next_back() {
                Some(last) => !NO_SPACE_AFTER.contains(&last),
                None => false,
            };
            if after_ok && !NO_SPACE_BEFORE.contains(&c) {
                out.push(' ');
            }
            pending_space = false;
        }
        if c == '"' || c == '\'' {
            out.push(c);
            i += 1;
            while i < chars.len() {
                let d = chars[i];
                out.push(d);
                i += 1;
                if d == '\\' && i < chars.len() {
                    out.push(chars[i]);
                    i += 1;
                } else if d == c {
                    break;
                }
            }
            continue;
        }
        if c == '}' && out.ends_with(';') {
            // The last declaration's semicolon is optional before a closing brace.
            out.pop();
        }
        out.push(c);
        i += 1;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn html_drops_indentation_between_tags() {
        let html = "<main>\n  <h1>Title</h1>\n  <p>Text</p>\n</main>\n";
        assert_eq!(minify_html(html), "<main><h1>Title</h1><p>Text</p></main>");
    }

    #[test]
    fn html_keeps_text_and_inline_spacing() {
        // A space between two links is visible and must stay; a line break is layout only.
        let html = "<a href=\"/a\">A</a> <a href=\"/b\">B</a>";
        assert_eq!(minify_html(html), html);
        assert_eq!(minify_html("<p>a  b</p>"), "<p>a  b</p>");
    }

    #[test]
    fn html_removes_comments_but_not_unterminated_ones() {
        assert_eq!(minify_html("<p>x</p><!-- note --><p>y</p>"), "<p>x</p><p>y</p>");
        assert_eq!(minify_html("<p>x</p><!-- open"), "<p>x</p><!-- open");
    }

    #[test]
    fn css_removes_comments_and_spaces_around_punctuation() {
        let css = "/* header */\nbody {\n  margin: 0 auto;\n  color: red;\n}\n";
        assert_eq!(minify_css(css), "body{margin:0 auto;color:red}");
    }

    #[test]
    fn css_keeps_descendant_space_and_calc_operators() {
        assert_eq!(minify_css("a :hover { x: 1 }"), "a :hover{x:1}");
        assert_eq!(minify_css("p { width: calc(1px + 2px) }"), "p{width:calc(1px + 2px)}");
    }

    #[test]
    fn css_keeps_string_literals_exactly() {
        assert_eq!(minify_css("p::before { content: \"a   /* b */  c\" }"), "p::before{content:\"a   /* b */  c\"}");
        assert_eq!(minify_css("q { content: 'it\\'s  ok' }"), "q{content:'it\\'s  ok'}");
    }

    #[test]
    fn css_unterminated_comment_is_dropped_to_end() {
        assert_eq!(minify_css("a{x:1}/* open"), "a{x:1}");
    }
}
