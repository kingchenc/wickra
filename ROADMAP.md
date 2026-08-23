# Roadmap

This roadmap describes the project's direction at a high level. It is
intentionally non-binding: priorities shift with feedback and available time,
and the authoritative, up-to-date view of planned work is the
[issue tracker](https://github.com/wickra-lib/wickra/issues). Shipped changes
are recorded in [`CHANGELOG.md`](CHANGELOG.md).

## Status

Wickra is **pre-1.0**. The public API is largely stable but may still change in
minor releases; breaking changes are called out in the changelog.

## Themes

- **Indicator coverage.** Continue broadening the indicator catalogue across
  families (trend, momentum, volatility, volume, statistics, market profile,
  and more), each with the same streaming/batch parity and test guarantees.
- **API stabilization toward 1.0.** Settle the public `Indicator` and
  `BarBuilder` traits and the binding surfaces, then commit to semantic
  versioning stability for a 1.0 release.
- **Performance.** Keep a tick free of any pass over the history behind it, and maintain the benchmark suite;
  investigate further allocation and cache improvements.
- **Bindings parity.** Keep the Python, Node.js and WASM bindings — plus
  the C ABI and the C#, Go, Java and R bindings generated from it — in lockstep with the
  Rust core, including type stubs and platform coverage.
- **Documentation.** Maintain a deep-dive page per indicator on
  <https://docs.wickra.org>, plus quickstarts and cookbook material.
- **Project health.** Maintain test coverage, static and dynamic analysis,
  signed releases, and supply-chain monitoring.

## How to influence the roadmap

Open or comment on an issue, or start with the
[feature-request template](.github/ISSUE_TEMPLATE/feature_request.md).
Well-scoped proposals and pull requests are the most effective way to move an
item forward.
