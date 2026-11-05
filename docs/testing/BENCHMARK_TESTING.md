# AQELITH Benchmark Testing

Status: **Proposed (M2).** How benchmarks themselves are tested, so a broken benchmark cannot produce a false number.

## 1. Tests for benchmarks

- Each benchmark has a correctness check: the output of the benchmarked program is compared with the expected output before timing is reported. The Astra audit noted a benchmark page whose earlier table was wrong; a correctness gate prevents timing a program that failed.
- The timed region is declared in the script; a test asserts that the region contains the workload and excludes start-up unless start-up is the subject.
- A benchmark whose run-to-run variance exceeds a declared threshold is reported as unstable, not as a result.

## 2. Regression gates

- Performance regression gates compare against a committed baseline on the same reference machine (directive §7). Gates fail on a declared percentage change beyond the measured variance.
- A gate on a shared CI runner without a fixed machine profile is advisory, not blocking.

## 3. Raw results

Every run writes raw results (JSON) with environment metadata ([../performance/BENCHMARK_METHODOLOGY.md](../performance/BENCHMARK_METHODOLOGY.md) §2). A test asserts the metadata fields are present.

## 4. Status

No benchmark exists. The `benchmarks/` directory holds only its README.
