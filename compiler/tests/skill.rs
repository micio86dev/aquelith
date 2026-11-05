//! The agent skill must not drift from the compiler: every stable code and every command is named in it.

use std::fs;
use std::path::Path;

use aqelith::diag::CODES;

fn skill_text() -> String {
    let p = Path::new(env!("CARGO_MANIFEST_DIR")).join("../skills/aqelith/SKILL.md");
    fs::read_to_string(&p).unwrap_or_else(|e| panic!("{}: {e}", p.display()))
}

#[test]
fn skill_names_every_diagnostic_code() {
    let text = skill_text();
    for (code, _) in CODES {
        assert!(text.contains(code), "SKILL.md does not mention {code}");
    }
}

#[test]
fn skill_names_every_command() {
    let text = skill_text();
    for cmd in ["aqelith check", "aqelith fmt", "aqelith query", "aqelith explain"] {
        assert!(text.contains(cmd), "SKILL.md does not mention `{cmd}`");
    }
}

#[test]
fn skill_has_frontmatter_name_and_description() {
    let text = skill_text();
    assert!(text.starts_with("---\nname: aqelith\ndescription: "), "bad frontmatter");
}
