//! Deterministic effect repair (M5): proposals are patches, verified before they are returned,
//! and applying them makes the checker accept the program.

use aqelith::check_source;
use aqelith::patch::apply;
use aqelith::repair::propose;

const MISSING_DIRECT: &str = "module demo

fn writer() -> i32 uses net {
    7
}

fn main() -> i32 uses io {
    writer()
}
";

const MISSING_TRANSITIVE: &str = "module demo

fn leaf() -> i32 uses net {
    1
}

fn middle() -> i32 uses io {
    leaf()
}

fn top() -> i32 {
    middle()
}
";

#[test]
fn a_missing_effect_is_proposed_and_the_patch_fixes_the_program() {
    let doc = propose(MISSING_DIRECT).unwrap().expect("a fix is needed");
    assert!(doc.contains(r#""op":"add-effect","target":"main""#), "{doc}");
    let accepted = apply(MISSING_DIRECT, &doc).expect("the proposal is verified by the patch API");
    assert!(accepted.text.contains("fn main() -> i32 uses io, net {"), "{}", accepted.text);
    assert!(check_source(&accepted.text).is_empty());
}

#[test]
fn a_fix_propagates_to_callers_until_nothing_is_missing() {
    let doc = propose(MISSING_TRANSITIVE).unwrap().expect("fixes are needed");
    let accepted = apply(MISSING_TRANSITIVE, &doc).unwrap();
    assert!(accepted.text.contains("fn middle() -> i32 uses io, net {"), "{}", accepted.text);
    // top calls middle, which now needs io and net, so top declares both.
    assert!(accepted.text.contains("fn top() -> i32 uses io, net {"), "{}", accepted.text);
    assert!(check_source(&accepted.text).is_empty());
}

#[test]
fn a_clean_program_needs_no_change() {
    let clean = "module demo\n\nfn f() -> i32 uses io {\n    1\n}\n";
    assert_eq!(propose(clean).unwrap(), None);
}

#[test]
fn the_proposal_is_deterministic() {
    assert_eq!(propose(MISSING_TRANSITIVE).unwrap(), propose(MISSING_TRANSITIVE).unwrap());
}

#[test]
fn a_syntax_error_is_reported_not_repaired() {
    let err = propose("module demo\nfn {").unwrap_err();
    assert!(err.contains("PAR001") || err.contains(":"), "{err}");
}
