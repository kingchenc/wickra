# Contributing to Wickra

Thanks for your interest in improving Wickra. This document explains how to
build the project, the standards a change must meet, and how to get it merged.

## License of contributions

Wickra is licensed under the **PolyForm Noncommercial License 1.0.0** (see
[`LICENSE`](LICENSE)). By submitting a contribution you agree that it is
licensed to the project under those same terms. The Noncommercial license
permits use for any purpose **other than** a commercial one; keep that in mind
when proposing features or depending on Wickra elsewhere.

## Project layout

| Path | Contents |
| --- | --- |
| `crates/wickra-core` | The indicator engine — every indicator lives here. |
| `crates/wickra` | Thin umbrella crate re-exporting `wickra-core`. |
| `crates/wickra-data` | CSV reader, tick aggregator, resampler, Binance feed. |
| `bindings/python` | PyO3 bindings (`wickra` on PyPI). |
| `bindings/node` | napi-rs bindings (`wickra` on npm). |
| `bindings/wasm` | wasm-bindgen bindings (`wickra-wasm` on npm). |
| `examples/` | Runnable examples. |
| `docs/` | Pointer to the project Wiki, which holds all documentation. |

## Building and testing

### Rust

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo test -p wickra-data --features live-binance
```

The minimum supported Rust version is **1.75** for the workspace crates and
**1.77** for `bindings/node`; the `msrv` CI job enforces both.

### Python

```bash
cd bindings/python
python -m maturin build --release --out dist
python -m pip install --force-reinstall --no-deps dist/wickra-*.whl
python -m pytest -q
```

### Node

```bash
cd bindings/node
npm install
npx napi build --platform --release
node --test __tests__/
```

### WASM

```bash
wasm-pack build bindings/wasm --target web --release --features panic-hook
wasm-pack test --node bindings/wasm
```

## Standards for a change

- **Formatting & lints.** `cargo fmt` must leave the tree unchanged and
  `cargo clippy ... -D warnings` must be clean. CI gates both.
- **Tests.** New behaviour needs tests; bug fixes need a regression test.
- **Indicator correctness.** A new or changed indicator must have a
  reference-value test against a known-good source (TA-Lib, pandas-ta, or a
  hand-computed value) and a `reset` test.
- **Streaming parity.** An indicator's `batch` output must equal the sequence
  of `update` calls.
- **Bindings.** A change to a public indicator API must be mirrored across the
  Python, Node, and WASM bindings, including their type stubs / `.d.ts`.
- **Docs.** Update the relevant page on the
  [project Wiki](https://github.com/kingchenc/wickra/wiki) and the
  `README.md` when behaviour or the public API changes. The Wiki lives in
  a separate git repository: `https://github.com/kingchenc/wickra.wiki.git`.
- **Changelog.** Add an entry under `## [Unreleased]` in `CHANGELOG.md`.

## Commit and pull-request workflow

1. Branch off `main`.
2. Keep commits focused — one logical change per commit, with an imperative
   subject line and a body explaining *why*.
3. Open a pull request against `main` and fill in the template.
4. CI must be green before review.

## Reporting bugs and proposing features

Use the issue templates under
[`.github/ISSUE_TEMPLATE`](.github/ISSUE_TEMPLATE). For security-sensitive
reports, follow [`SECURITY.md`](SECURITY.md) instead of opening a public issue.
