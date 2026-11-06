//! Writes the form page and its runtime to a directory (default: web/examples/form).
use std::path::PathBuf;

fn main() {
    let dir = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/examples/form")));
    std::fs::create_dir_all(&dir).expect("create output directory");
    let html = aqelith_web::render_page(&aqelith_web::form_page()).expect("form page is valid");
    std::fs::write(dir.join("index.html"), html).expect("write index.html");
    std::fs::copy(
        concat!(env!("CARGO_MANIFEST_DIR"), "/runtime/aqelith-runtime.js"),
        dir.join("aqelith-runtime.js"),
    )
    .expect("copy runtime");
    println!("wrote {}", dir.display());
}
