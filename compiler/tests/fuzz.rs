//! Deterministic fuzzing (M7). Mutates the accepted corpus, JSON and patch documents with a
//! fixed-seed generator, so any failure reproduces with the same seed. Not a substitute for
//! coverage-guided fuzzing in CI (not built yet, see docs/testing/TEST_STRATEGY.md).
//!
//! Invariants checked on every input:
//! - no panic in parse, check, format, JSON reading or patch application;
//! - if the formatter accepts a text, formatting it again gives the same text;
//! - a patch that is accepted produces a candidate that the checker accepts.

use std::fs;
use std::path::Path;

use aqelith::{check_source, format_source, json, patch};

struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }

    fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }
}

const ALPHABET: &[char] = &[
    'a',
    'z',
    '_',
    '0',
    '9',
    ' ',
    '\n',
    '(',
    ')',
    '{',
    '}',
    ',',
    ':',
    ';',
    '=',
    '+',
    '-',
    '*',
    '"',
    '\\',
    '\u{e9}',
    '\u{2028}',
    '\u{1F600}',
    '\0',
    '\t',
    '<',
    '&',
];

fn mutate(rng: &mut Rng, src: &str) -> String {
    let mut chars: Vec<char> = src.chars().collect();
    for _ in 0..(1 + rng.below(4)) {
        let len = chars.len();
        match rng.below(4) {
            0 if len > 0 => {
                let at = rng.below(len);
                chars.remove(at);
            }
            1 => {
                let at = rng.below(len + 1);
                chars.insert(at, ALPHABET[rng.below(ALPHABET.len())]);
            }
            2 if len > 0 => {
                let from = rng.below(len);
                let to = (from + rng.below(12)).min(len);
                let piece: Vec<char> = chars[from..to].to_vec();
                let at = rng.below(len + 1);
                chars.splice(at..at, piece);
            }
            _ if len > 0 => {
                let at = rng.below(len);
                chars[at] = ALPHABET[rng.below(ALPHABET.len())];
            }
            _ => {}
        }
    }
    chars.into_iter().collect()
}

fn accepted_corpus() -> Vec<String> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/corpus/accept");
    let mut out = Vec::new();
    for entry in fs::read_dir(dir).unwrap() {
        out.push(fs::read_to_string(entry.unwrap().path()).unwrap());
    }
    out.sort();
    out
}

fn check_source_invariants(text: &str) {
    let _ = check_source(text);
    if let Ok(formatted) = format_source(text) {
        let again = format_source(&formatted).expect("formatted text must parse");
        assert_eq!(again, formatted, "formatter is not idempotent for {text:?}");
    }
}

#[test]
fn mutated_sources_never_panic_and_format_is_idempotent() {
    let mut rng = Rng(0x9e37_79b9_7f4a_7c15);
    for base in accepted_corpus() {
        for _ in 0..1500 {
            let text = mutate(&mut rng, &base);
            check_source_invariants(&text);
        }
    }
}

#[test]
fn mutated_patch_documents_never_panic_and_accepted_candidates_pass_the_checker() {
    let src = "module demo\n\nfn add1(x: i32) -> i32 {\n    x + 1\n}\n\nfn main() -> i32 uses io {\n    add1(41)\n}\n";
    let base = aqelith::patch::graph_hash(&format_source(src).unwrap());
    let seeds = [
        format!(
            r#"{{"api":"aqelith.patch.v1","base":"{base}","ops":[{{"op":"rename","target":"add1","expect":{{"name":"add1"}},"to":"inc"}}]}}"#
        ),
        format!(
            r#"{{"api":"aqelith.patch.v1","base":"{base}","ops":[{{"op":"add-effect","target":"main","expect":{{"uses":["io"]}},"to":"net"}}]}}"#
        ),
        format!(
            r#"{{"api":"aqelith.patch.v1","base":"{base}","ops":[{{"op":"set-return","target":"add1","expect":{{"ret":"i32"}},"to":"bool"}}]}}"#
        ),
    ];
    let mut rng = Rng(0x1234_5678_9abc_def1);
    let mut accepted = 0usize;
    for seed in seeds {
        for _ in 0..1500 {
            let doc = mutate(&mut rng, &seed);
            if let Ok(ok) = patch::apply(src, &doc) {
                accepted += 1;
                assert!(
                    check_source(&ok.text).is_empty(),
                    "accepted candidate fails the checker: {doc}"
                );
            }
        }
    }
    // The mutations are small, so some documents should still apply; this keeps the test honest.
    assert!(
        accepted > 0,
        "no mutated patch was accepted; the generator may be broken"
    );
}

#[test]
fn mutated_json_never_panics() {
    // The seed holds an integer beyond i64 on purpose: it must be an error, never a panic.
    let seed = r#"{"a":[1,-2,true,false,null,"xA",10000000000000000000],"b":{"c":"d"}}"#;
    let mut rng = Rng(0xdead_beef_cafe_f00d);
    for _ in 0..5000 {
        let text = mutate(&mut rng, seed);
        let _ = json::parse(&text);
    }
}
