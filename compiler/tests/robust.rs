//! Robustness: the front end must never panic or overflow the stack on any input.
//! Random inputs use a fixed-seed generator so failures reproduce.

use aqelith::check_source;

struct Lcg(u64);

impl Lcg {
    fn next(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        self.0 >> 33
    }
}

const PIECES: &[&str] = &[
    "fn ",
    "let ",
    "mut ",
    "module ",
    "uses ",
    "(",
    ")",
    "{",
    "}",
    ",",
    ":",
    ";",
    "=",
    "->",
    "==",
    "+",
    "-",
    "*",
    "1",
    "4294967296",
    "x",
    "f",
    "i32",
    "u32",
    "bool",
    "true",
    "\n",
    " ",
    "/",
    "é",
    "io",
    "float",
];

#[test]
fn random_token_soup_never_panics() {
    let mut rng = Lcg(7);
    for _ in 0..5_000 {
        let n = (rng.next() % 60) as usize;
        let mut src = String::from("module m\n");
        for _ in 0..n {
            src.push_str(PIECES[(rng.next() as usize) % PIECES.len()]);
        }
        for d in check_source(&src) {
            assert!(d.line >= 1 && d.column >= 1, "bad position in {src:?}");
        }
    }
}

#[test]
fn hundred_thousand_open_parentheses_are_rejected_not_crashed() {
    let src = format!(
        "module m\nfn f() -> i32 {{ {}1{} }}",
        "(".repeat(100_000),
        ")".repeat(100_000)
    );
    assert_eq!(check_source(&src)[0].code, "PAR002");
}

#[test]
fn hundred_thousand_term_chain_is_rejected_not_crashed() {
    let chain = vec!["1"; 100_000].join(" + ");
    let src = format!("module m\nfn f() -> i32 {{ {chain} }}");
    assert_eq!(check_source(&src)[0].code, "PAR002");
}

#[test]
fn deep_but_legal_nesting_is_accepted() {
    let depth = 60;
    let src = format!(
        "module m\nfn f() -> i32 {{ {}1{} }}",
        "(".repeat(depth),
        ")".repeat(depth)
    );
    assert!(check_source(&src).is_empty());
}
