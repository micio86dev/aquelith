# Performance baseline: AQELITH tools and the static build

Date: 2026-11-06. Commit: 71ff333. Status: **baseline measured, no budget set.** A budget changes only through a recorded decision, after repeated runs on the machine declared here (see [PERFORMANCE_BUDGETS.md](PERFORMANCE_BUDGETS.md), section 3).

## Declared reference machine

This baseline declares the following machine as its reference. It is the owner's development computer; the owner should confirm that it is the one to measure against.

| Item | Value |
|---|---|
| CPU | Apple M1, 8 cores, arm64 |
| Memory | 8 GiB |
| OS | macOS 26.5.2 (build 25F84) |
| Toolchain | rustc 1.89.0 (Homebrew), release profile |
| Load | not controlled; the desktop was in normal use |

## Method

Script: [data/measure.py](data/measure.py). Raw samples: [data/baseline-2026-11-06.csv](data/baseline-2026-11-06.csv) (120 rows).

- Inputs are fixed files: the four-page site from the trial run (`site.aq` with `style.css`), and the landing example in `web/examples/landing/`.
- Each command runs three times to warm up, then twenty times measured. Each sample is the wall time of one process, including process start-up.
- Output folders are removed before each build, so no sample reuses earlier output.
- Times in milliseconds, from `time.perf_counter`.

## Results

| Case | Median | p95 | Min | Max |
|---|---|---|---|---|
| `aqelith check`, four-page site | 2.92 | 3.48 | 2.54 | 3.67 |
| `aqelith fmt`, four-page site | 3.13 | 5.27 | 2.55 | 5.28 |
| `build_site`, four-page site with stylesheet | 4.07 | 6.47 | 3.65 | 7.16 |
| `aqelith check`, landing | 2.83 | 4.43 | 2.48 | 5.16 |
| `build_site`, landing with stylesheet | 3.39 | 4.08 | 3.26 | 6.91 |
| Plain HTML: `cp -R` of the same site (the only "build" a plain site needs) | 6.07 | 6.89 | 5.54 | 7.14 |

## What this does and does not show

- It shows the cost of each command on one machine under normal use, with the spread of twenty samples.
- The copy row is a process timing, not a comparison of compilers. A plain site has no build step, so the comparison is only indicative.
- The times include process start-up, which dominates at these sizes. Measuring the work inside the process would need a benchmark harness, not built yet.
- Not measured: browser rendering, network transfer, memory use, a larger site, other machines, other operating systems.
- Samples come from one session. A budget needs repeated sessions and a check that the spread is stable.

## Next steps

1. Repeat the measurement in at least three sessions on different days, and record whether the medians agree within the spread.
2. Add a larger input (for example twenty pages) so that work inside the process dominates start-up.
3. Only then propose a budget, as a recorded decision.
