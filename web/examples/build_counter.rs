//! Writes the counter page and its runtime to a directory (default: web/examples/counter).
use std::path::PathBuf;

fn main() {
    let dir = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/examples/counter")));
    std::fs::create_dir_all(&dir).expect("create output directory");
    let html =
        aqelith_web::render_page(&aqelith_web::counter_page()).expect("counter page is valid");
    std::fs::write(dir.join("index.html"), html).expect("write index.html");
    std::fs::copy(
        concat!(env!("CARGO_MANIFEST_DIR"), "/runtime/aqelith-runtime.js"),
        dir.join("aqelith-runtime.js"),
    )
    .expect("copy runtime");
    println!("wrote {}", dir.display());
}
