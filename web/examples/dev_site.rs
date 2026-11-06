//! Development server with live reload for an AQELITH site:
//! `dev_site <file.aq> [style.css] [--port N]`. Serves http://127.0.0.1:N/ (default 8080; 0 picks
//! a free port) and reloads the open page after each save of the source or the stylesheet.
//!
//! Exit codes: 0 stopped by the user, 1 the source does not check at start, 2 usage or bind error.
fn usage() -> ! {
    eprintln!("usage: dev_site <file.aq> [style.css] [--port N]");
    std::process::exit(2);
}

fn main() {
    let mut args = std::env::args().skip(1);
    let mut source = None;
    let mut style = None;
    let mut port: u16 = 8080;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--port" => {
                port = args
                    .next()
                    .and_then(|p| p.parse().ok())
                    .unwrap_or_else(|| usage());
            }
            _ if source.is_none() => source = Some(arg),
            _ if style.is_none() => style = Some(arg),
            _ => usage(),
        }
    }
    let Some(source) = source else { usage() };
    let config = aqelith_web::dev::Config {
        source: source.into(),
        style: style.map(Into::into),
        port,
    };
    let server = match aqelith_web::dev::start(config) {
        Ok(server) => server,
        Err(aqelith_web::dev::StartError::Build(message)) => {
            eprintln!("{message}");
            std::process::exit(1);
        }
        Err(aqelith_web::dev::StartError::Bind(message)) => {
            eprintln!("cannot serve: {message}");
            std::process::exit(2);
        }
    };
    println!(
        "serving http://127.0.0.1:{}/ (Ctrl-C to stop)",
        server.addr.port()
    );
    // The server runs on its own threads; the main thread only keeps the process alive.
    loop {
        std::thread::park();
    }
}
