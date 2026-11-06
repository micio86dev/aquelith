#!/usr/bin/env python3
"""Baseline timing for AQELITH tools and the static build (see docs/performance/BASELINE.md).

Usage: measure.py <repo-root> <input-dir> <plain-html-dir> <out.csv>
Three warm-up runs, then twenty measured runs per command. Each sample is the wall time of one
process, including its start-up. Nothing is cached between samples: output folders are removed first.
"""
import csv, os, shutil, subprocess, sys, tempfile, time

repo, inp, plain, out_csv = sys.argv[1:5]
aq = f"{repo}/compiler/target/release/aqelith"
build = f"{repo}/web/target/release/examples/build_site"
landing = f"{repo}/web/examples/landing/landing.aq"
landing_css = f"{repo}/web/examples/landing/style.css"
scratch = tempfile.mkdtemp(prefix="aq-perf-")

def out_dir(name):
    d = os.path.join(scratch, name)
    shutil.rmtree(d, ignore_errors=True)
    return d

cases = {
    "check_site": lambda: [aq, "check", f"{inp}/site.aq"],
    "fmt_site": lambda: [aq, "fmt", f"{inp}/site.aq"],
    "build_site": lambda: [build, f"{inp}/site.aq", out_dir("site"), f"{inp}/style.css"],
    "check_landing": lambda: [aq, "check", landing],
    "build_landing": lambda: [build, landing, out_dir("landing"), landing_css],
    "copy_plain_html": lambda: ["cp", "-R", plain, out_dir("plain")],
}
WARM, RUNS = 3, 20
rows = []
for name, argv in cases.items():
    for i in range(WARM + RUNS):
        argv_now = argv()
        t0 = time.perf_counter()
        p = subprocess.run(argv_now, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        ms = (time.perf_counter() - t0) * 1000
        if p.returncode != 0:
            sys.exit(f"{name} failed with exit {p.returncode}")
        if i >= WARM:
            rows.append((name, i - WARM + 1, round(ms, 3)))
with open(out_csv, "w", newline="") as f:
    w = csv.writer(f)
    w.writerow(["case", "run", "ms"])
    w.writerows(rows)
shutil.rmtree(scratch, ignore_errors=True)
print(f"wrote {len(rows)} samples to {out_csv}")
