//! `aqelith-serve <file.aq> [--port N]`: serves the program's `handle` function on 127.0.0.1.
//! Runs until it is stopped by a signal (Ctrl-C). Exit codes: 1 the program does not lower or has no valid
//! handler, 2 usage or bind error. A stop by signal has no exit code of its own.
use aqelith::ir::lower_source;
use aqelith::serve::{start, ServeError};

fn usage() -> ! {
    eprintln!("usage: aqelith-serve <file.aq> [--port N]");
    std::process::exit(2);
}

fn main() {
    let mut args = std::env::args().skip(1);
    let mut file = None;
    let mut port: u16 = 8080;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--port" => {
                port = args
                    .next()
                    .and_then(|p| p.parse().ok())
                    .unwrap_or_else(|| usage());
            }
            _ if file.is_none() => file = Some(arg),
            _ => usage(),
        }
    }
    let Some(file) = file else { usage() };
    let src = std::fs::read_to_string(&file).unwrap_or_else(|e| {
        eprintln!("{file}: {e}");
        std::process::exit(2);
    });
    let program = lower_source(&src).unwrap_or_else(|e| {
        eprintln!("{file}: {e:?}");
        std::process::exit(1);
    });
    match start(program, port) {
        Ok(addr) => println!("serving http://{addr}/ (Ctrl-C to stop)"),
        Err(ServeError::BadHandler(m)) => {
            eprintln!("{file}: {m}");
            std::process::exit(1);
        }
        Err(ServeError::Bind(m)) => {
            eprintln!("cannot serve: {m}");
            std::process::exit(2);
        }
    }
    loop {
        std::thread::park();
    }
}
