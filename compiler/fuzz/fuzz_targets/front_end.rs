#![no_main]
//! Front end: parsing and checking never panic; a text the formatter accepts is a fixed point.
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let Ok(text) = std::str::from_utf8(data) else {
        return;
    };
    let _ = aqelith::check_source(text);
    if let Ok(formatted) = aqelith::format_source(text) {
        let again = aqelith::format_source(&formatted).expect("formatted text parses");
        assert_eq!(again, formatted, "formatter is not idempotent");
    }
});
