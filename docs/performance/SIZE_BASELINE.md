# Size baseline: static landing build

Status: **measurement only.** No budget is set. This follows [PERFORMANCE_BUDGETS.md](PERFORMANCE_BUDGETS.md): a budget needs a declared reference machine and repeated timing runs, and neither exists yet. These numbers describe the output of one build; they say nothing about speed.

Machine: Apple M1, macOS 25.5, development machine, not a reference platform (see [PERFORMANCE_BUDGETS.md](PERFORMANCE_BUDGETS.md)). Input: `web/examples/landing/landing.aq` with `style.css`, three pages. Gzip: `gzip -9`, bytes of the compressed stream.

Method:

```sh
cd "<repo>/web"
cargo run -q --example build_site -- examples/landing/landing.aq <out> examples/landing/style.css
wc -c <out>/index.html <out>/how/index.html <out>/start/index.html <out>/style.css
gzip -9 -c <out>/index.html | wc -c   # and likewise for each file
```

## Results

| File | Before (raw / gzip) | After minify + CSP (raw / gzip) |
|---|---|---|
| `index.html` | 462 / 307 | 593 / 373 |
| `how/index.html` | 407 / 300 | 538 / 370 |
| `start/index.html` | 371 / 277 | 502 / 341 |
| `style.css` | 552 / 349 | 352 / 261 |
| **Total** | **1792 / 1233** | **1985 / 1345** |

Before: the generator output as of commit 5a39fb6 (no minifier, no CSP). After: this feature branch.

## Reading the numbers

- The stylesheet shrinks by about 36% raw (comments and indentation removed).
- The HTML grows. The cause is the Content-Security-Policy meta element (about 130 bytes per page, raw), which the minifier cannot remove. It is the price of a policy that a static host cannot set as a header.
- The net effect on the landing is +112 bytes gzip (+9%). Hosts that can send the CSP as a header may drop the meta element; the policy itself does not change.

## Limits

- Three pages, one stylesheet, no images. A larger page will change the ratio.
- Only gzip is measured. Brotli would be smaller; the server chooses the encoding, not the compiler.
- Not measured: transfer time, parse time, render time.
