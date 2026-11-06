// Hardening checks from review: duplicate parameters, a safe atomic write, and callers of a
// recursive function.

use aqelith::check::check_module;
use aqelith::parser::parse;
use aqelith::patch::write_atomic;
use aqelith::query::query_fn;
use std::fs;
use std::path::PathBuf;

fn scratch(name: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("aq-hard-{name}-{}-{nanos}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn a_parameter_declared_twice_is_refused() {
    let src = "module m\n\nfn f(a: i32, a: u32) -> i32 {\n    1\n}\n";
    let m = parse(src).unwrap();
    let codes: Vec<&str> = check_module(src, &m).iter().map(|d| d.code).collect();
    assert_eq!(codes, vec!["DUP001"]);
}

#[test]
fn the_atomic_write_replaces_the_file_and_leaves_no_temporary() {
    let dir = scratch("write");
    let target = dir.join("site.aq");
    fs::write(&target, "old").unwrap();
    write_atomic(&target, "new").unwrap();
    assert_eq!(fs::read_to_string(&target).unwrap(), "new");
    let leftovers: Vec<_> = fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .filter(|n| n.contains("aqpatch"))
        .collect();
    assert!(leftovers.is_empty(), "temporary files left: {leftovers:?}");
    fs::remove_dir_all(&dir).unwrap();
}

#[cfg(unix)]
#[test]
fn a_planted_symlink_at_the_temporary_name_is_never_followed() {
    use aqelith::patch::write_via_temp;
    let dir = scratch("symlink");
    let victim = dir.join("victim.txt");
    fs::write(&victim, "keep me").unwrap();
    let target = dir.join("site.aq");
    let tmp = dir.join(".site.aq.planted.tmp");
    std::os::unix::fs::symlink(&victim, &tmp).unwrap();
    let err = write_via_temp(&target, "new", &tmp).expect_err("the planted name must be refused");
    assert_eq!(err.kind(), std::io::ErrorKind::AlreadyExists);
    assert_eq!(fs::read_to_string(&victim).unwrap(), "keep me");
    assert!(!target.exists(), "the target must not be written");
    fs::remove_dir_all(&dir).unwrap();
}

fn valid_base(src: &str) -> String {
    let m = parse(src).unwrap();
    aqelith::patch::graph_hash(&aqelith::format::format_module(&m))
}

#[test]
fn a_rename_target_must_be_a_plain_identifier() {
    use aqelith::patch::apply;
    let src = "module m\n\nfn add1(x: i32) -> i32 {\n    x + 1\n}\n";
    let bad = format!(
        r#"{{"api":"aqelith.patch.v1","base":"{}","ops":[{{"op":"rename","target":"add1","expect":{{"name":"add1"}},"to":"a() -> unit {{\n}}\nfn b"}}]}}"#,
        valid_base(src)
    );
    let err = apply(src, &bad).expect_err("an injected name must be refused");
    assert!(
        format!("{err:?}").contains("identifier"),
        "refused for another reason: {err:?}"
    );
    // A plain name with the same shape is accepted, so the refusal is about the name alone.
    let good = format!(
        r#"{{"api":"aqelith.patch.v1","base":"{}","ops":[{{"op":"rename","target":"add1","expect":{{"name":"add1"}},"to":"inc"}}]}}"#,
        valid_base(src)
    );
    assert!(apply(src, &good).is_ok());
}

#[test]
fn a_with_that_holds_a_page_is_refused_not_silently_trimmed() {
    use aqelith::patch::apply;
    let src = "module m\n\nfn add1(x: i32) -> i32 {\n    x + 1\n}\n";
    let insert = format!(
        r#"{{"api":"aqelith.patch.v1","base":"{}","ops":[{{"op":"insert","target":"add1","with":"fn a() -> unit {{\n}}\npage p \"/\" {{\n}}"}}]}}"#,
        valid_base(src)
    );
    let err = apply(src, &insert).expect_err("a page in 'with' must be refused");
    assert!(
        format!("{err:?}").contains("page"),
        "refused for another reason: {err:?}"
    );
}

#[test]
fn a_recursive_function_is_its_own_caller_and_callee() {
    let src = "module m\n\nfn f(a: i32) -> i32 {\n    f(a)\n}\n";
    let m = parse(src).unwrap();
    let out = query_fn(&m, "f").expect("found");
    assert!(out.contains("\"calls\":[\"f\"]"), "{out}");
    assert!(out.contains("\"called_by\":[\"f\"]"), "{out}");
}
