# AQELITH Performance Budgets

Status: **Proposed (M2). No numeric budget is set.** Directive §7.5: budgets are provisional until a baseline exists on a declared hardware and workload set. Writing numbers now would turn aspirations into claims, which the directive forbids.

## 1. Required header for every budget

Each budget entry must state: hardware, operating system, build mode, dataset, warm or cold, measurement method, number of runs, median and p95, and variance. A budget without these fields is invalid and fails review.

## 2. Budget table (to be filled)

| Metric | Workload | Hardware | Baseline status | Budget |
|---|---|---|---|---|
| Cold compile | Reference app, to be defined in M4 | Declared in [BENCHMARK_METHODOLOGY.md](BENCHMARK_METHODOLOGY.md) §2 | not measured | not set |
| Incremental compile after one-line edit | same | same | not measured | not set |
| Interactive event to paint (web) | Reference web app, M6 | same plus browser | not measured | not set |
| Peak runtime memory | Reference app | same | not measured | not set |
| Startup of generated web app | Reference app, cold cache | same plus browser | not measured | not set |

## 3. Rule for setting a budget

1. Measure the baseline on the declared hardware, with at least 10 runs, warm and cold separated.
2. Record median, p95 and variance.
3. Set the budget only after the baseline is committed with its raw results.
4. A budget changes only through a recorded decision.

## 4. Current hardware for context

Apple M1, 8 GiB RAM, external SSD for workspace (see [../reports/ENVIRONMENT_AUDIT.md](../reports/ENVIRONMENT_AUDIT.md)). This is a development machine, not a reference platform. A reference machine must be declared before any budget is set.
