//! Benchmark skeleton (M7). Times the front end (parse, check, format) on a generated module and
//! prints raw timings. It claims no numbers: there is no reference machine yet (open question Q22)
//! and results depend on the host. Method: docs/performance/BENCHMARK_METHODOLOGY.md.
//!
//! Run: cargo run --release --example bench_frontend -- 200   (argument: number of functions)

use std::time::Instant;

fn generate(n: usize) -> String {
    let mut out = String::from("module bench\n");
    for i in 0..n {
        let body = if i == 0 {
            "x + 1".to_string()
        } else {
            format!("f{}(x) + {}", i - 1, i % 7)
        };
        out.push_str(&format!("\nfn f{i}(x: i32) -> i32 {{\n    {body}\n}}\n"));
    }
    out
}

fn time_us(samples: usize, mut f: impl FnMut()) -> (u128, u128) {
    let mut times: Vec<u128> = (0..samples)
        .map(|_| {
            let start = Instant::now();
            f();
            start.elapsed().as_micros()
        })
        .collect();
    times.sort_unstable();
    (times[0], times[times.len() / 2])
}

fn main() {
    let n: usize = std::env::args()
        .nth(1)
        .and_then(|a| a.parse().ok())
        .unwrap_or(100);
    let src = generate(n);
    let samples = 15;
    let (pmin, pmed) = time_us(samples, || {
        aqelith::parser::parse(&src).expect("generated module parses");
    });
    let (cmin, cmed) = time_us(samples, || {
        assert!(aqelith::check_source(&src).is_empty());
    });
    let (fmin, fmed) = time_us(samples, || {
        aqelith::format_source(&src).expect("generated module formats");
    });
    println!("functions={n} bytes={} samples={samples}", src.len());
    println!("parse_us min={pmin} median={pmed}");
    println!("check_us min={cmin} median={cmed}");
    println!("format_us min={fmin} median={fmed}");
}
