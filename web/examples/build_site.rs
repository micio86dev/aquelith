//! Builds a static site from an AQELITH source file:
//! `build_site <file.aq> <output-dir> [style.css]`. The stylesheet is copied to `style.css`
//! in the output and linked from every page.
use std::path::PathBuf;

fn main() {
    let mut args = std::env::args().skip(1);
    let (Some(input), Some(out)) = (args.next(), args.next()) else {
        eprintln!("usage: build_site <file.aq> <output-dir> [style.css]");
        std::process::exit(2);
    };
    let style = args.next();
    let src = std::fs::read_to_string(&input).unwrap_or_else(|e| {
        eprintln!("{input}: {e}");
        std::process::exit(2);
    });
    let built = match &style {
        Some(_) => aqelith_web::site_from_source_with_style(&src, "/style.css"),
        None => aqelith_web::site_from_source(&src),
    };
    let files = match built {
        Ok(files) => files,
        Err(aqelith_web::SourceError::Diagnostics(diags)) => {
            for d in diags {
                eprintln!("{input}:{}:{}: {}: {}", d.line, d.column, d.code, d.message);
            }
            std::process::exit(1);
        }
        Err(e) => {
            eprintln!("{e:?}");
            std::process::exit(1);
        }
    };
    let dir = PathBuf::from(out);
    for (path, html) in &files {
        let target = dir.join(path);
        std::fs::create_dir_all(target.parent().unwrap_or(&dir)).expect("create directory");
        std::fs::write(&target, html).expect("write page");
    }
    if let Some(css) = style {
        std::fs::copy(&css, dir.join("style.css")).unwrap_or_else(|e| {
            eprintln!("{css}: {e}");
            std::process::exit(2);
        });
    }
    println!("wrote {} pages to {}", files.len(), dir.display());
}
