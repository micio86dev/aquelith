use std::fs;
use std::path::{Path, PathBuf};

use aqelith::{check_source, diag, format_source, query, parser};

fn corpus(kind: &str) -> Vec<PathBuf> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/corpus").join(kind);
    let mut files: Vec<PathBuf> = fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("{}: {e}", dir.display()))
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|x| x == "aq"))
        .collect();
    files.sort();
    assert!(!files.is_empty(), "no fixtures in {}", dir.display());
    files
}

fn read(p: &Path) -> String {
    fs::read_to_string(p).unwrap()
}

#[test]
fn accepted_programs_check_clean() {
    for f in corpus("accept") {
        let d = check_source(&read(&f));
        assert!(d.is_empty(), "{}: {d:?}", f.display());
    }
}

#[test]
fn formatting_is_idempotent_and_still_valid() {
    for f in corpus("accept") {
        let once = format_source(&read(&f)).unwrap();
        let twice = format_source(&once).unwrap();
        assert_eq!(once, twice, "{}", f.display());
        assert!(check_source(&once).is_empty(), "{}", f.display());
    }
}

#[test]
fn rejected_programs_report_their_own_code_first() {
    for f in corpus("reject") {
        let stem = f.file_stem().unwrap().to_string_lossy().to_string();
        let expected = stem.split('_').next().unwrap();
        let d = check_source(&read(&f));
        assert_eq!(d.first().map(|x| x.code), Some(expected), "{}: {d:?}", f.display());
    }
}

#[test]
fn every_documented_code_has_a_reject_fixture() {
    let stems: Vec<String> = corpus("reject")
        .iter()
        .map(|f| f.file_stem().unwrap().to_string_lossy().split('_').next().unwrap().to_string())
        .collect();
    // PAT codes come from patch documents, not source files; tests/patch.rs covers them.
    for (code, _) in diag::CODES.iter().filter(|(code, _)| !code.starts_with("PAT")) {
        assert!(stems.iter().any(|s| s == code), "no reject fixture for {code}");
    }
}

#[test]
fn every_fixture_code_is_documented() {
    for f in corpus("reject") {
        let code = f.file_stem().unwrap().to_string_lossy().split('_').next().unwrap().to_string();
        assert!(diag::explain(&code).is_some(), "{code} has no explanation");
    }
}

#[test]
fn golden_formatting_of_groups() {
    let out = format_source(&read(&Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/corpus/accept/groups.aq"))).unwrap();
    assert_eq!(out, "module groups\n\nfn f(a: u32) -> u32 {\n    (a + 1) * 2\n}\n");
}

#[test]
fn formatter_keeps_only_needed_parentheses() {
    let src = "module p\n\nfn f(a: u32) -> u32 {\n    ((a)) + (2 * 3) + (a + 1) * 2\n}\n";
    let out = format_source(src).unwrap();
    assert!(out.contains("    a + 2 * 3 + (a + 1) * 2\n"), "got: {out}");
}

#[test]
fn query_reports_callers_and_callees() {
    let src = read(&Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/corpus/accept/arith.aq"));
    let m = parser::parse(&src).unwrap();
    let add = query::query_fn(&m, "add").unwrap();
    assert!(add.contains("\"called_by\":[\"main\"]"), "{add}");
    let main = query::query_fn(&m, "main").unwrap();
    assert!(main.contains("\"calls\":[\"add\"]"), "{main}");
    assert!(main.contains("\"uses\":[\"io\"]"), "{main}");
    assert!(query::query_fn(&m, "missing").is_none());
}
