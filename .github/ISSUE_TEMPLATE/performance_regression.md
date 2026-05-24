---
name: Performance regression
about: Report a measurable slowdown, memory blowup, or throughput drop.
title: "[Perf] <indicator / API> regressed in <version>"
labels: ["performance", "regression", "triage"]
assignees: []
---

## Summary

<!-- Which code path got slower, by how much, and since when? -->

## Affected code path

- Indicator / API: `e.g. EMA.update`
- Binding: `Rust / Python / Node / Wasm`
- Hot loop or one-shot call?

## Versions compared

| Version  | Throughput / latency / memory | Notes |
| -------- | ----------------------------- | ----- |
| `0.4.1`  | `e.g. 12.3 ns/iter`           | baseline (Good) |
| `0.4.2`  | `e.g. 38.7 ns/iter`           | regressed |

## Benchmark / reproducer

<!--
Paste the criterion / pytest-benchmark / hyperfine command and its output.
For one-off measurements, include the timing snippet inline.
-->

```bash
cargo bench --bench ema -- --save-baseline new
```

```
ema/update              time:   [38.5 ns 38.7 ns 38.9 ns]
                        change: [+213.4% +214.8% +216.1%] (p = 0.00 < 0.05)
                        Performance has regressed.
```

## Hardware / environment

| Field        | Value                                  |
| ------------ | -------------------------------------- |
| CPU          | `e.g. Ryzen 9 7950X, AVX2 + AVX512`    |
| OS / arch    | `e.g. Linux 6.8 x86_64`                |
| Toolchain    | `rustc 1.x.y`                          |
| Build flags  | `RUSTFLAGS=...`, `--release`, profile  |

## Suspected cause

<!-- Optional. Link the commit / PR if you've bisected it. -->
