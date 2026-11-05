# AQELITH Benchmark Methodology

Status: **Proposed (M2).** Applies to every benchmark published by the project, including comparisons (directive §15.2).

## 1. Equivalence

A comparison is valid only if workloads, hardware, optimization modes, input data and compiler or runtime versions are the same across the compared systems. Release builds are compared with release builds; a debug build is never compared with a release build.

## 2. Hardware and environment record

Each run records: CPU model, core count, RAM, OS and version, power state, Docker or VM limits if used, filesystem and whether the workspace is on an external volume, and concurrency level.

Note: the M1 machine used for competitor audits (Apple M1, 8 GiB, Docker VM 4 GiB, external SSD) is not a valid reference machine. Its timings are observations only.

## 3. Runs and statistics

- At least 10 measured runs after warm-up, separated into warm and cold.
- Report median, p95, minimum, maximum, and standard deviation or interquartile range.
- Do not report a single run.
- Outliers are not removed silently; any exclusion is listed with its reason.

## 4. Measurement

- Wall time and CPU time reported separately.
- Peak memory via the OS accounting tool, recorded per run.
- Timing inside the program is preferred over timing the launcher, unless the launcher is the thing measured. The Astra benchmark page admits its harness included interpreter start-up in per-run timings; AQELITH must not repeat that error. Benchmarks state which parts they include.

## 5. Reporting

Three separate categories, never merged:

1. **Reproduced:** AQELITH's scripts run by a third party or in CI, with raw results committed.
2. **Upstream-reported:** the competitor's own figures, labelled as such, with source link and version.
3. **AQELITH-own:** numbers from AQELITH's scripts, labelled with the commit.

Raw results (CSV or JSON) are committed next to the script. A summary without raw data is not published.

## 6. What a single benchmark can show

No single benchmark is proof of general superiority (directive §15.2). Benchmarks are reported per workload with their limits stated.

## 7. Forbidden practices

- Micro-benchmarks on fixtures too small to represent the workload (the Orison "sub-100 µs" figure measured an 11-line module).
- Comparing optimized and unoptimized builds.
- Unreproducible scripts.
- Changing the workload after seeing results.
