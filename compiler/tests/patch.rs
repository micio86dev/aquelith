//! Semantic patch API slice v1: acceptance, stale base, preconditions, validation, and file safety.

use std::process::Command;

use aqelith::check_source;
use aqelith::format_source;
use aqelith::patch::{apply, fn_hash, graph_hash, Rejection};

const SRC: &str = "module demo

fn add1(x: i32) -> i32 {
    x + 1
}

fn main() -> i32 uses io {
    add1(41)
}
";

fn base_of(src: &str) -> String {
    graph_hash(&format_source(src).unwrap())
}

fn doc(base: &str, ops: &str) -> String {
    format!(r#"{{"api":"aqelith.patch.v1","base":"{base}","ops":[{ops}]}}"#)
}

fn rejected(src: &str, patch: &str) -> Rejection {
    *apply(src, patch).expect_err("the patch should be rejected")
}

#[test]
fn rename_changes_the_definition_and_every_call() {
    let patch = doc(
        &base_of(SRC),
        r#"{"op":"rename","target":"add1","expect":{"name":"add1"},"to":"inc"}"#,
    );
    let accepted = apply(SRC, &patch).expect("the patch should be accepted");
    assert!(
        accepted.text.contains("fn inc(x: i32) -> i32"),
        "{}",
        accepted.text
    );
    assert!(accepted.text.contains("inc(41)"), "{}", accepted.text);
    assert!(!accepted.text.contains("add1"), "{}", accepted.text);
    assert!(check_source(&accepted.text).is_empty());
    assert_ne!(accepted.new_base, accepted.base);
    assert_eq!(accepted.touched, vec!["inc".to_string()]);
}

#[test]
fn a_base_from_canonical_text_is_accepted_on_any_layout() {
    // The graph hash is over canonical text, so spacing in the source does not make a base stale.
    let spaced = "module demo\n\nfn add1( x: i32 )->i32{x+1}\n\nfn main()->i32 uses io{add1(41)}\n";
    assert_eq!(base_of(spaced), base_of(SRC));
    let patch = doc(
        &base_of(SRC),
        r#"{"op":"rename","target":"add1","expect":{"name":"add1"},"to":"inc"}"#,
    );
    let accepted =
        apply(spaced, &patch).expect("the same base should be accepted on the spaced source");
    assert!(
        accepted.text.contains("fn inc(x: i32) -> i32"),
        "{}",
        accepted.text
    );
}

#[test]
fn a_stale_base_is_rejected_before_any_op_runs() {
    let patch = doc(
        "graph:0000000000000000",
        r#"{"op":"rename","target":"add1","expect":{"name":"add1"},"to":"inc"}"#,
    );
    let r = rejected(SRC, &patch);
    assert_eq!((r.code, r.kind), ("PAT002", "stale"));
    assert_eq!(r.expected, Some(base_of(SRC)));
}

#[test]
fn a_failed_precondition_reports_expected_and_actual() {
    let patch = doc(
        &base_of(SRC),
        r#"{"op":"rename","target":"add1","expect":{"name":"nope"},"to":"inc"}"#,
    );
    let r = rejected(SRC, &patch);
    assert_eq!((r.code, r.op), ("PAT003", Some(0)));
    assert_eq!(
        (r.expected.as_deref(), r.actual.as_deref()),
        (Some("nope"), Some("add1"))
    );
}

#[test]
fn an_unknown_target_is_rejected() {
    let patch = doc(
        &base_of(SRC),
        r#"{"op":"rename","target":"missing","expect":{"name":"missing"},"to":"x"}"#,
    );
    assert_eq!(rejected(SRC, &patch).code, "PAT004");
}

#[test]
fn the_full_checker_has_the_last_word_on_the_candidate() {
    // Renaming add1 to main creates a duplicate definition: the checker must reject it.
    let patch = doc(
        &base_of(SRC),
        r#"{"op":"rename","target":"add1","expect":{"name":"add1"},"to":"main"}"#,
    );
    let r = rejected(SRC, &patch);
    assert_eq!(r.code, "PAT005");
    assert!(
        r.diagnostics.iter().any(|d| d.code == "DUP001"),
        "{:?}",
        r.diagnostics
    );
}

#[test]
fn changing_a_return_type_breaks_the_body_and_is_rejected() {
    let patch = doc(
        &base_of(SRC),
        r#"{"op":"set-return","target":"add1","expect":{"ret":"i32"},"to":"bool"}"#,
    );
    let r = rejected(SRC, &patch);
    assert_eq!(r.code, "PAT005");
    assert!(
        r.diagnostics.iter().any(|d| d.code == "TYP001"),
        "{:?}",
        r.diagnostics
    );
}

#[test]
fn removing_an_effect_that_a_call_needs_is_rejected() {
    let src = "module demo

fn writer() -> i32 uses io {
    7
}

fn main() -> i32 uses io {
    writer()
}
";
    let patch = doc(
        &base_of(src),
        r#"{"op":"remove-effect","target":"main","expect":{"uses":["io"]},"to":"io"}"#,
    );
    let r = rejected(src, &patch);
    assert_eq!(r.code, "PAT005");
    assert!(
        r.diagnostics.iter().any(|d| d.code == "EFF001"),
        "{:?}",
        r.diagnostics
    );
}

#[test]
fn adding_an_effect_is_accepted_and_only_known_effects_are_allowed() {
    // main declares io and calls add1; declaring net on main is harmless.
    let good = doc(
        &base_of(SRC),
        r#"{"op":"add-effect","target":"main","expect":{"uses":["io"]},"to":"net"}"#,
    );
    let accepted = apply(SRC, &good).expect("adding a known effect should be accepted");
    assert!(
        accepted.text.contains("fn main() -> i32 uses io, net {"),
        "{}",
        accepted.text
    );

    // add1 gaining net would make its caller main need net too: the checker rejects it.
    let caller_needs = doc(
        &base_of(SRC),
        r#"{"op":"add-effect","target":"add1","expect":{"uses":[]},"to":"net"}"#,
    );
    assert_eq!(rejected(SRC, &caller_needs).code, "PAT005");

    let unknown = doc(
        &base_of(SRC),
        r#"{"op":"add-effect","target":"main","expect":{"uses":["io"]},"to":"magic"}"#,
    );
    assert_eq!(rejected(SRC, &unknown).code, "PAT001");
}

#[test]
fn malformed_documents_are_rejected_with_pat001() {
    let base = base_of(SRC);
    let cases = [
        "not json".to_string(),
        format!(r#"{{"api":"other","base":"{base}","ops":[]}}"#),
        doc(
            &base,
            r#"{"op":"delete","target":"add1","expect":{},"to":"x"}"#,
        ),
        doc(
            &base,
            r#"{"op":"set-return","target":"add1","expect":{"ret":"i32"},"to":"f64"}"#,
        ),
        doc(
            &base,
            r#"{"op":"rename","target":"add1","expect":{"name":1.5},"to":"x"}"#,
        ),
    ];
    for case in cases {
        assert_eq!(rejected(SRC, &case).code, "PAT001", "{case}");
    }
}

#[test]
fn a_source_that_does_not_parse_cannot_be_patched() {
    let broken = "module demo\n\nfn main( {\n";
    let patch = doc(
        &graph_hash(broken),
        r#"{"op":"rename","target":"main","expect":{"name":"main"},"to":"x"}"#,
    );
    assert_eq!(rejected(broken, &patch).code, "PAT006");
}

#[test]
fn all_ops_are_checked_before_anything_is_returned() {
    // The first op is fine, the second has a stale precondition: the whole patch is rejected.
    let patch = doc(
        &base_of(SRC),
        r#"{"op":"rename","target":"add1","expect":{"name":"add1"},"to":"inc"},
           {"op":"rename","target":"inc","expect":{"name":"other"},"to":"x"}"#,
    );
    let r = rejected(SRC, &patch);
    assert_eq!((r.code, r.op), ("PAT003", Some(1)));
}

fn scratch_file(name: &str, text: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("aq-patch-{}-{name}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("demo.aq");
    std::fs::write(&path, text).unwrap();
    path
}

fn run_patch(file: &std::path::Path, patch_text: &str, extra: &[&str]) -> (i32, String) {
    let doc_path = file.with_extension("json");
    std::fs::write(&doc_path, patch_text).unwrap();
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_aqelith"));
    cmd.arg("patch").arg(file).arg(&doc_path).args(extra);
    let out = cmd.output().unwrap();
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).into_owned(),
    )
}

#[test]
fn cli_dry_run_reports_without_touching_the_file() {
    let file = scratch_file("dry", SRC);
    let patch = doc(
        &base_of(SRC),
        r#"{"op":"rename","target":"add1","expect":{"name":"add1"},"to":"inc"}"#,
    );
    let (code, stdout) = run_patch(&file, &patch, &["--dry-run"]);
    assert_eq!(code, 0, "{stdout}");
    assert!(stdout.contains("\"status\":\"dry-run\""), "{stdout}");
    assert_eq!(std::fs::read_to_string(&file).unwrap(), SRC);
}

#[test]
fn cli_rejection_exits_1_and_leaves_the_file_byte_identical() {
    let file = scratch_file("reject", SRC);
    let patch = doc(
        "graph:0000000000000000",
        r#"{"op":"rename","target":"add1","expect":{"name":"add1"},"to":"inc"}"#,
    );
    let (code, stdout) = run_patch(&file, &patch, &[]);
    assert_eq!(code, 1, "{stdout}");
    assert!(stdout.contains("\"code\":\"PAT002\""), "{stdout}");
    assert_eq!(std::fs::read(&file).unwrap(), SRC.as_bytes());
}

#[test]
fn cli_accepted_patch_writes_the_candidate_atomically() {
    let file = scratch_file("write", SRC);
    let patch = doc(
        &base_of(SRC),
        r#"{"op":"rename","target":"add1","expect":{"name":"add1"},"to":"inc"}"#,
    );
    let (code, stdout) = run_patch(&file, &patch, &[]);
    assert_eq!(code, 0, "{stdout}");
    let written = std::fs::read_to_string(&file).unwrap();
    assert!(written.contains("fn inc(x: i32)"), "{written}");
    assert!(check_source(&written).is_empty());
    let leftovers: Vec<_> = std::fs::read_dir(file.parent().unwrap())
        .unwrap()
        .filter_map(Result::ok)
        .filter(|e| e.file_name().to_string_lossy().contains(".tmp"))
        .collect();
    assert!(leftovers.is_empty(), "temporary files left behind");
}

#[test]
fn every_patch_code_is_produced_by_some_scenario() {
    let base = base_of(SRC);
    let scenarios = [
        (
            "PAT001",
            doc(
                &base,
                r#"{"op":"delete","target":"add1","expect":{},"to":"x"}"#,
            ),
            SRC.to_string(),
        ),
        (
            "PAT002",
            doc(
                "graph:0000000000000000",
                r#"{"op":"rename","target":"add1","expect":{"name":"add1"},"to":"x"}"#,
            ),
            SRC.to_string(),
        ),
        (
            "PAT003",
            doc(
                &base,
                r#"{"op":"rename","target":"add1","expect":{"name":"nope"},"to":"x"}"#,
            ),
            SRC.to_string(),
        ),
        (
            "PAT004",
            doc(
                &base,
                r#"{"op":"rename","target":"nope","expect":{"name":"nope"},"to":"x"}"#,
            ),
            SRC.to_string(),
        ),
        (
            "PAT005",
            doc(
                &base,
                r#"{"op":"rename","target":"add1","expect":{"name":"add1"},"to":"main"}"#,
            ),
            SRC.to_string(),
        ),
        (
            "PAT006",
            doc(
                &graph_hash("module demo\n\nfn main( {\n"),
                r#"{"op":"rename","target":"main","expect":{"name":"main"},"to":"x"}"#,
            ),
            "module demo\n\nfn main( {\n".to_string(),
        ),
    ];
    for (code, patch, src) in scenarios {
        assert_eq!(rejected(&src, &patch).code, code);
    }
    for (code, _) in aqelith::diag::CODES
        .iter()
        .filter(|(c, _)| c.starts_with("PAT"))
    {
        assert!(["PAT001", "PAT002", "PAT003", "PAT004", "PAT005", "PAT006"].contains(code));
    }
}

fn fn_hash_of(src: &str, name: &str) -> String {
    let module = aqelith::parser::parse(src).unwrap();
    let f = module
        .items
        .iter()
        .find(|f| f.name == name)
        .expect("function exists");
    fn_hash(f)
}

const WITH_UNUSED: &str = "module demo

fn add1(x: i32) -> i32 {
    x + 1
}

fn unused() -> i32 {
    0
}

fn main() -> i32 uses io {
    add1(41)
}
";

#[test]
fn delete_removes_an_unused_function_when_the_hash_matches() {
    let hash = fn_hash_of(WITH_UNUSED, "unused");
    let patch = doc(
        &base_of(WITH_UNUSED),
        &format!(r#"{{"op":"delete","target":"unused","expect":{{"hash":"{hash}"}}}}"#),
    );
    let accepted =
        apply(WITH_UNUSED, &patch).expect("deleting an unused function should be accepted");
    assert!(!accepted.text.contains("unused"), "{}", accepted.text);
    assert!(check_source(&accepted.text).is_empty());
}

#[test]
fn delete_is_rejected_while_a_caller_still_needs_the_function() {
    let hash = fn_hash_of(SRC, "add1");
    let patch = doc(
        &base_of(SRC),
        &format!(r#"{{"op":"delete","target":"add1","expect":{{"hash":"{hash}"}}}}"#),
    );
    let r = rejected(SRC, &patch);
    assert_eq!(r.code, "PAT005");
    assert!(
        r.diagnostics.iter().any(|d| d.code == "RES001"),
        "{:?}",
        r.diagnostics
    );
}

#[test]
fn delete_with_a_stale_hash_reports_expected_and_actual() {
    let patch = doc(
        &base_of(WITH_UNUSED),
        r#"{"op":"delete","target":"unused","expect":{"hash":"graph:0000000000000000"}}"#,
    );
    let r = rejected(WITH_UNUSED, &patch);
    assert_eq!((r.code, r.op), ("PAT003", Some(0)));
    assert_eq!(r.actual, Some(fn_hash_of(WITH_UNUSED, "unused")));
}

#[test]
fn replace_swaps_the_body_when_the_hash_matches() {
    let hash = fn_hash_of(SRC, "add1");
    let with = "fn add1(x: i32) -> i32 {\n    x + 2\n}\n";
    let patch = doc(
        &base_of(SRC),
        &format!(
            r#"{{"op":"replace","target":"add1","expect":{{"hash":"{hash}"}},"with":{}}}"#,
            json_text(with)
        ),
    );
    let accepted = apply(SRC, &patch).expect("replacing with a valid function should be accepted");
    assert!(accepted.text.contains("x + 2"), "{}", accepted.text);
    assert!(!accepted.text.contains("x + 1"), "{}", accepted.text);
    assert_eq!(accepted.touched, vec!["add1".to_string()]);
}

#[test]
fn replace_that_renames_the_function_breaks_its_callers_and_is_rejected() {
    let hash = fn_hash_of(SRC, "add1");
    let with = "fn inc(x: i32) -> i32 {\n    x + 1\n}\n";
    let patch = doc(
        &base_of(SRC),
        &format!(
            r#"{{"op":"replace","target":"add1","expect":{{"hash":"{hash}"}},"with":{}}}"#,
            json_text(with)
        ),
    );
    let r = rejected(SRC, &patch);
    assert_eq!(r.code, "PAT005");
    assert!(
        r.diagnostics.iter().any(|d| d.code == "RES001"),
        "{:?}",
        r.diagnostics
    );
}

#[test]
fn replace_and_insert_need_exactly_one_valid_function_in_with() {
    let hash = fn_hash_of(SRC, "add1");
    let two = "fn a() -> i32 {\n    1\n}\n\nfn b() -> i32 {\n    2\n}\n";
    let not_fn = "this is not a function";
    for with in [two, not_fn] {
        let patch = doc(
            &base_of(SRC),
            &format!(
                r#"{{"op":"replace","target":"add1","expect":{{"hash":"{hash}"}},"with":{}}}"#,
                json_text(with)
            ),
        );
        assert_eq!(rejected(SRC, &patch).code, "PAT001", "{with}");
    }
}

#[test]
fn insert_places_a_function_after_its_anchor() {
    let with = "fn zero() -> i32 {\n    0\n}\n";
    let patch = doc(
        &base_of(SRC),
        &format!(
            r#"{{"op":"insert","target":"add1","with":{}}}"#,
            json_text(with)
        ),
    );
    let accepted = apply(SRC, &patch).expect("inserting a valid function should be accepted");
    let add1_at = accepted.text.find("fn add1").unwrap();
    let zero_at = accepted.text.find("fn zero").unwrap();
    let main_at = accepted.text.find("fn main").unwrap();
    assert!(add1_at < zero_at && zero_at < main_at, "{}", accepted.text);
}

#[test]
fn insert_rejects_an_unknown_anchor_and_a_duplicate_name() {
    let with = "fn zero() -> i32 {\n    0\n}\n";
    let unknown = doc(
        &base_of(SRC),
        &format!(
            r#"{{"op":"insert","target":"nope","with":{}}}"#,
            json_text(with)
        ),
    );
    assert_eq!(rejected(SRC, &unknown).code, "PAT004");

    let dup = "fn main() -> i32 {\n    0\n}\n";
    let duplicate = doc(
        &base_of(SRC),
        &format!(
            r#"{{"op":"insert","target":"add1","with":{}}}"#,
            json_text(dup)
        ),
    );
    let r = rejected(SRC, &duplicate);
    assert_eq!(r.code, "PAT005");
    assert!(
        r.diagnostics.iter().any(|d| d.code == "DUP001"),
        "{:?}",
        r.diagnostics
    );
}

/// Encodes a Rust string as a JSON string literal (for the `with` field).
fn json_text(s: &str) -> String {
    format!(
        "\"{}\"",
        s.replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('\n', "\\n")
    )
}

#[test]
fn set_param_type_changes_one_parameter_when_its_type_matches() {
    // The body does not use x, so the new type cannot break it.
    let src = "module demo\n\nfn konst(x: i32) -> i32 {\n    7\n}\n";
    let good = doc(
        &base_of(src),
        r#"{"op":"set-param-type","target":"konst","expect":{"param":"x","type":"i32"},"to":"u32"}"#,
    );
    let accepted = apply(src, &good).expect("the parameter type can change");
    assert!(
        accepted.text.contains("fn konst(x: u32) -> i32"),
        "{}",
        accepted.text
    );

    let stale = doc(
        &base_of(src),
        r#"{"op":"set-param-type","target":"konst","expect":{"param":"x","type":"bool"},"to":"u32"}"#,
    );
    let r = rejected(src, &stale);
    assert_eq!((r.code, r.expected.as_deref()), ("PAT003", Some("bool")));

    // A body that uses the parameter with another type is rejected by the checker.
    let used = "module demo\n\nfn add1(x: i32) -> i32 {\n    x + 1\n}\n";
    let breaks = doc(
        &base_of(used),
        r#"{"op":"set-param-type","target":"add1","expect":{"param":"x","type":"i32"},"to":"u32"}"#,
    );
    assert_eq!(rejected(used, &breaks).code, "PAT005");

    let unknown = doc(
        &base_of(src),
        r#"{"op":"set-param-type","target":"konst","expect":{"param":"y","type":"i32"},"to":"u32"}"#,
    );
    assert_eq!(rejected(src, &unknown).code, "PAT003");
}
