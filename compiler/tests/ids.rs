//! Persistent function ids (ADR-0004): ids survive rename and same-name replace, are never reused,
//! and id targets are resolved through the index. Runs the real binary on scratch files.

use std::path::{Path, PathBuf};
use std::process::Command;

use aqelith::ids::IdIndex;
use aqelith::json::Json;
use aqelith::patch::{apply, apply_indexed, graph_hash};

const SRC: &str = "module demo

fn add1(x: i32) -> i32 {
    x + 1
}

fn main() -> i32 uses io {
    add1(41)
}
";

fn scratch(name: &str, text: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("aq-ids-{}-{name}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("demo.aq");
    std::fs::write(&path, text).unwrap();
    path
}

fn cli(args: &[&str]) -> (i32, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_aqelith"))
        .args(args)
        .output()
        .unwrap();
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).into_owned(),
    )
}

fn index_of(file: &Path) -> IdIndex {
    let text =
        std::fs::read_to_string(format!("{}.ids.json", file.display())).expect("index exists");
    IdIndex::from_json(&text).unwrap()
}

fn base(src: &str) -> String {
    graph_hash(&aqelith::format_source(src).unwrap())
}

fn run_patch(file: &Path, doc: &str, extra: &[&str]) -> (i32, String) {
    let doc_path = file.with_extension("patch.json");
    std::fs::write(&doc_path, doc).unwrap();
    let mut args = vec!["patch", file.to_str().unwrap(), doc_path.to_str().unwrap()];
    args.extend_from_slice(extra);
    cli(&args)
}

#[test]
fn rename_by_id_keeps_the_id() {
    let file = scratch("rename", SRC);
    let (code, _) = run_patch(
        &file,
        &format!(
            r#"{{"api":"aqelith.patch.v1","base":"{}","ops":[]}}"#,
            base(SRC)
        ),
        &[],
    );
    assert_eq!(code, 0, "an empty patch is accepted and creates the index");
    let id = index_of(&file).id_of("add1").unwrap().to_string();

    let doc = format!(
        r#"{{"api":"aqelith.patch.v1","base":"{}","ops":[{{"op":"rename","target":"id:{id}","expect":{{"name":"add1"}},"to":"inc"}}]}}"#,
        base(SRC)
    );
    let (code, out) = run_patch(&file, &doc, &[]);
    assert_eq!(code, 0, "{out}");
    let index = index_of(&file);
    assert_eq!(
        index.id_of("inc"),
        Some(id.as_str()),
        "the renamed function keeps its id"
    );
    assert_eq!(index.id_of("add1"), None);
    assert!(std::fs::read_to_string(&file).unwrap().contains("fn inc("));
}

#[test]
fn same_name_replace_keeps_the_id_and_a_deleted_id_is_never_reused() {
    let file = scratch("replace", SRC);
    run_patch(
        &file,
        &format!(
            r#"{{"api":"aqelith.patch.v1","base":"{}","ops":[]}}"#,
            base(SRC)
        ),
        &[],
    );
    let before = index_of(&file);
    let add1_id = before.id_of("add1").unwrap().to_string();
    let main_id = before.id_of("main").unwrap().to_string();

    // Replace add1 with the same name: its id stays.
    let hash = {
        let module = aqelith::parser::parse(SRC).unwrap();
        aqelith::patch::fn_hash(
            module.items.iter().find(|f| f.name == "add1").unwrap(),
            &module.records,
        )
    };
    let replace = format!(
        r#"{{"api":"aqelith.patch.v1","base":"{}","ops":[{{"op":"replace","target":"add1","expect":{{"hash":"{hash}"}},"with":"fn add1(x: i32) -> i32 {{\n    x + 2\n}}\n"}}]}}"#,
        base(SRC)
    );
    let (code, out) = run_patch(&file, &replace, &[]);
    assert_eq!(code, 0, "{out}");
    let after = index_of(&file);
    assert_eq!(after.id_of("add1"), Some(add1_id.as_str()));
    assert_eq!(after.id_of("main"), Some(main_id.as_str()));

    // Deleting add1 here would break main, so the deletion case uses a helper that nothing calls.
    let with_unused = format!("{SRC}\nfn unused() -> i32 {{\n    0\n}}\n");
    let file2 = scratch("reuse", &with_unused);
    run_patch(
        &file2,
        &format!(
            r#"{{"api":"aqelith.patch.v1","base":"{}","ops":[]}}"#,
            base(&with_unused)
        ),
        &[],
    );
    let unused_id = index_of(&file2).id_of("unused").unwrap().to_string();
    let module = aqelith::parser::parse(&with_unused).unwrap();
    let unused_hash = aqelith::patch::fn_hash(
        module.items.iter().find(|f| f.name == "unused").unwrap(),
        &module.records,
    );
    let delete = format!(
        r#"{{"api":"aqelith.patch.v1","base":"{}","ops":[{{"op":"delete","target":"unused","expect":{{"hash":"{unused_hash}"}}}}]}}"#,
        base(&with_unused)
    );
    let (code, out) = run_patch(&file2, &delete, &[]);
    assert_eq!(code, 0, "{out}");
    assert_eq!(index_of(&file2).id_of("unused"), None);

    // A later function gets a new id, never the deleted one.
    let grown = format!("{SRC}\nfn later() -> i32 {{\n    1\n}}\n");
    std::fs::write(&file2, &grown).unwrap();
    let (code, _) = cli(&["ids", file2.to_str().unwrap(), "--write"]);
    assert_eq!(code, 0);
    let later_id = index_of(&file2).id_of("later").unwrap().to_string();
    assert_ne!(later_id, unused_id, "ids are never reused");
}

#[test]
fn id_targets_need_an_index_and_must_exist() {
    let doc = format!(
        r#"{{"api":"aqelith.patch.v1","base":"{}","ops":[{{"op":"rename","target":"id:{}","expect":{{"name":"add1"}},"to":"x"}}]}}"#,
        base(SRC),
        "0".repeat(32)
    );
    let no_index = apply(SRC, &doc).expect_err("id targets without an index are rejected");
    assert_eq!(no_index.code, "PAT001");

    let mut index = IdIndex::default();
    let module = aqelith::parser::parse(SRC).unwrap();
    let mut n = 0u128;
    index.sync(&module, &mut || {
        n += 1;
        format!("{n:032x}")
    });
    let unknown = apply_indexed(SRC, &doc, Some(&index)).expect_err("an unknown id is rejected");
    assert_eq!(unknown.code, "PAT004");
}

#[test]
fn a_dry_run_writes_neither_the_source_nor_the_index() {
    let file = scratch("dry", SRC);
    let doc = format!(
        r#"{{"api":"aqelith.patch.v1","base":"{}","ops":[{{"op":"rename","target":"add1","expect":{{"name":"add1"}},"to":"inc"}}]}}"#,
        base(SRC)
    );
    let (code, out) = run_patch(&file, &doc, &["--dry-run"]);
    assert_eq!(code, 0, "{out}");
    assert_eq!(std::fs::read_to_string(&file).unwrap(), SRC);
    assert!(!Path::new(&format!("{}.ids.json", file.display())).exists());
}

#[test]
fn a_damaged_index_file_is_reported_not_ignored() {
    let file = scratch("damaged", SRC);
    std::fs::write(format!("{}.ids.json", file.display()), "{\"api\":\"nope\"}").unwrap();
    let doc = format!(
        r#"{{"api":"aqelith.patch.v1","base":"{}","ops":[]}}"#,
        base(SRC)
    );
    let (code, _) = run_patch(&file, &doc, &[]);
    assert_eq!(code, 2);
    assert_eq!(
        std::fs::read_to_string(&file).unwrap(),
        SRC,
        "the source is untouched"
    );
}

#[test]
fn the_index_file_is_valid_json_with_the_documented_shape() {
    let file = scratch("shape", SRC);
    let (code, out) = cli(&["ids", file.to_str().unwrap()]);
    assert_eq!(code, 0);
    let doc = aqelith::json::parse(&out).unwrap();
    assert_eq!(
        doc.get("api").and_then(Json::as_str),
        Some("aqelith.ids.v1")
    );
    assert_eq!(
        doc.get("functions")
            .and_then(Json::as_array)
            .map(|a| a.len()),
        Some(2)
    );
}
