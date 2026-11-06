#![no_main]
//! Patch API: input is a source text, a line of `\x1e` (record separator), then a patch document.
//! Any accepted patch must produce a candidate that the checker accepts.
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let Ok(text) = std::str::from_utf8(data) else {
        return;
    };
    let Some((src, doc)) = text.split_once('\u{1e}') else {
        return;
    };
    if let Ok(accepted) = aqelith::patch::apply(src, doc) {
        assert!(
            aqelith::check_source(&accepted.text).is_empty(),
            "accepted candidate fails the checker"
        );
    }
});
