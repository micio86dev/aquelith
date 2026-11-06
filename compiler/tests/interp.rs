// Reference interpreter, milestone S1 (docs/architecture/INTERPRETER.md): values, checked overflow,
// refused effects, bounded recursion.

use aqelith::interp::{run, RunError, Val};
use aqelith::ir::lower_source;

fn program(src: &str) -> aqelith::ir::Program {
    lower_source(src).unwrap_or_else(|e| panic!("does not lower: {e:?}"))
}

#[test]
fn a_pure_function_returns_its_value() {
    let p = program("module m\n\nfn add1(x: i32) -> i32 {\n    x + 1\n}\n");
    assert_eq!(run(&p, "add1", &[Val::Int(41)]), Ok(Val::Int(42)));
}

#[test]
fn comparison_and_calls_compose() {
    let src = "module m\n\nfn twice(x: u32) -> u32 {\n    x + x\n}\n\nfn check(a: u32) -> bool {\n    twice(a) == 6\n}\n";
    let p = program(src);
    assert_eq!(run(&p, "check", &[Val::Int(3)]), Ok(Val::Bool(true)));
    assert_eq!(run(&p, "check", &[Val::Int(4)]), Ok(Val::Bool(false)));
}

#[test]
fn unit_functions_return_unit() {
    let p = program("module m\n\nfn nothing() -> unit {\n}\n");
    assert_eq!(run(&p, "nothing", &[]), Ok(Val::Unit));
}

#[test]
fn i32_overflow_stops_the_run_at_the_boundary() {
    let p = program("module m\n\nfn inc(x: i32) -> i32 {\n    x + 1\n}\n");
    assert_eq!(
        run(&p, "inc", &[Val::Int(i64::from(i32::MAX) - 1)]),
        Ok(Val::Int(i64::from(i32::MAX)))
    );
    assert_eq!(
        run(&p, "inc", &[Val::Int(i64::from(i32::MAX))]),
        Err(RunError::Overflow {
            func: "inc".into(),
            ty: aqelith::ast::Ty::I32
        })
    );
}

#[test]
fn u32_underflow_is_an_overflow_not_a_wrap() {
    let p = program("module m\n\nfn diff(a: u32, b: u32) -> u32 {\n    a - b\n}\n");
    assert_eq!(
        run(&p, "diff", &[Val::Int(5), Val::Int(2)]),
        Ok(Val::Int(3))
    );
    assert!(matches!(
        run(&p, "diff", &[Val::Int(1), Val::Int(2)]),
        Err(RunError::Overflow { .. })
    ));
}

#[test]
fn multiplication_is_checked_too() {
    let p = program("module m\n\nfn sq(x: i32) -> i32 {\n    x * x\n}\n");
    assert_eq!(
        run(&p, "sq", &[Val::Int(46340)]),
        Ok(Val::Int(2_147_395_600))
    );
    assert!(matches!(
        run(&p, "sq", &[Val::Int(46341)]),
        Err(RunError::Overflow { .. })
    ));
}

#[test]
fn a_declared_effect_is_refused_because_no_host_is_provided() {
    let p = program(
        "module m\n\nfn log() -> unit uses io {\n}\n\nfn run() -> unit uses io {\n    log();\n}\n",
    );
    assert_eq!(
        run(&p, "log", &[]),
        Err(RunError::EffectNotAvailable {
            func: "log".into(),
            effect: "io".into()
        })
    );
    assert!(matches!(
        run(&p, "run", &[]),
        Err(RunError::EffectNotAvailable { .. })
    ));
}

#[test]
fn unbounded_recursion_stops_at_the_depth_limit() {
    let p = program("module m\n\nfn spin(a: i32) -> i32 {\n    spin(a)\n}\n");
    assert_eq!(run(&p, "spin", &[Val::Int(0)]), Err(RunError::TooDeep));
}

#[test]
fn bad_calls_are_reported_not_run() {
    let p = program("module m\n\nfn f(x: i32) -> i32 {\n    x\n}\n");
    assert!(matches!(
        run(&p, "missing", &[]),
        Err(RunError::NoSuchFunction(_))
    ));
    assert!(matches!(run(&p, "f", &[]), Err(RunError::ArgCount { .. })));
    assert!(matches!(
        run(&p, "f", &[Val::Bool(true)]),
        Err(RunError::ArgType { .. })
    ));
    assert!(matches!(
        run(&p, "f", &[Val::Int(1 << 40)]),
        Err(RunError::ArgType { .. })
    ));
}

#[test]
fn every_accepted_corpus_function_with_no_parameters_runs_without_panicking() {
    let dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/corpus/accept");
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        let src = std::fs::read_to_string(&path).unwrap();
        let Ok(p) = lower_source(&src) else { continue };
        for f in p
            .funcs
            .iter()
            .filter(|f| f.params.is_empty() && f.uses.is_empty())
        {
            // Either a value or a clear refusal; never a panic or a malformed IR.
            match run(&p, &f.name, &[]) {
                Ok(_) => {}
                Err(RunError::Overflow { .. }) => {}
                Err(e) => panic!("{}: {}: {e:?}", path.display(), f.name),
            }
        }
    }
}
