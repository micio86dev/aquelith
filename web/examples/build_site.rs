//! Builds a static site from an AQELITH source file:
//! `build_site <file.aq> <output-dir> [style.css]`. The output is minified HTML with a
//! Content-Security-Policy meta element. The stylesheet is minified and written to `style.css`
//! in the output, and linked from every page.
//!
//! Exit codes: 0 success, 1 the source does not check or the site is refused, 2 usage or I/O error.
use std::path::PathBuf;

fn fail(code: i32, message: impl std::fmt::Display) -> ! {
    eprintln!("{message}");
    std::process::exit(code);
}

fn main() {
    let mut args = std::env::args().skip(1);
    let (Some(input), Some(out)) = (args.next(), args.next()) else {
        fail(2, "usage: build_site <file.aq> <output-dir> [style.css]");
    };
    let style = args.next();
    let src = std::fs::read_to_string(&input).unwrap_or_else(|e| fail(2, format!("{input}: {e}")));
    let style_css = style
        .as_ref()
        .map(|css| std::fs::read_to_string(css).unwrap_or_else(|e| fail(2, format!("{css}: {e}"))));
    let built = aqelith_web::static_site(&src, style.as_ref().map(|_| "/style.css"));
    let files = match built {
        Ok(files) => files,
        Err(aqelith_web::SourceError::Diagnostics(diags)) => {
            for d in diags {
                eprintln!("{input}:{}:{}: {}: {}", d.line, d.column, d.code, d.message);
            }
            std::process::exit(1);
        }
        Err(e) => fail(1, format!("{e:?}")),
    };
    let dir = PathBuf::from(out);
    for (path, html) in &files {
        let target = dir.join(path);
        let parent = target.parent().unwrap_or(&dir);
        std::fs::create_dir_all(parent).unwrap_or_else(|e| fail(2, format!("{}: {e}", parent.display())));
        std::fs::write(&target, html).unwrap_or_else(|e| fail(2, format!("{}: {e}", target.display())));
    }
    if let Some(css) = style_css {
        let target = dir.join("style.css");
        let minified = aqelith_web::minify_css(&css);
        std::fs::write(&target, minified).unwrap_or_else(|e| fail(2, format!("{}: {e}", target.display())));
    }
    println!("wrote {} pages to {}", files.len(), dir.display());
}
