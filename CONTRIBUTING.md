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
| `docs/` | Pointer to the documentation site (docs.wickra.org); the docs live in the `wickra-lib/wickra-docs` repo. |

## Building and testing

### Rust

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo test -p wickra-data --features live-binance
```

The minimum supported Rust version is **1.86** for the workspace crates and
**1.88** for `bindings/node`; the `msrv` CI job enforces both. These floors are
not chosen freely — they are the lowest versions our dependencies allow
(criterion 0.8.2, the bench dev-dependency, requires 1.86; napi-build 2.3.2
requires 1.88). We keep the MSRV at that dependency-forced floor on purpose so
the library builds for the widest possible audience; please don't raise it
without a dependency that actually requires it.

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

## Lockfile policy

| Component | Lockfile | Tracked? | Why |
| --- | --- | --- | --- |
| Workspace (Rust) | `Cargo.lock` | **yes** | The workspace ships binaries (examples, fuzz harness) and CI builds, so the dependency graph is pinned for reproducible builds. |
| `bindings/node` | `package-lock.json` | **yes** | Reproducible `npm install` for the native binding. |
| `examples/node` | `package-lock.json` | **yes** | Same — the runnable Node examples link the binding via a `file:` dependency. |
| `bindings/python` | — | n/a (no lockfile) | The published package pins only `numpy>=1.22` at runtime; its native code is pinned through the workspace `Cargo.lock`. The CI/bench dev tooling it installs is hash-locked separately — see the `.github/requirements` row. |
| `.github/requirements` | `*.txt` (hash-pinned) | **yes** | CI/bench Python tooling, locked with `uv pip compile --generate-hashes` (OpenSSF Scorecard PinnedDependencies). `ci-dev` is split per Python version — `ci-dev-py39.txt` and `ci-dev-py3.txt` — because numpy ships no single release with wheels for both cp39 and cp313; `bench.txt` covers the single-version bench job. |
| `fuzz` | `fuzz/Cargo.lock` | **no** (ignored) | `fuzz/` is a detached crate; `cargo-fuzz init` generates `fuzz/.gitignore` which ignores its `Cargo.lock`. The fuzz smoke job resolves dependencies fresh, so the lock is not needed for reproducibility here. |
| `site` (marketing) | `package-lock.json` | **no** (ghost-ignored) | The VitePress site is a local-only project excluded via `.git/info/exclude`; its lockfile stays local. |

When adding a new committed Node package, commit its `package-lock.json` too and
remove any matching ignore rule. Do **not** add a top-level `package-lock.json` —
the repository root is not an npm package.

To refresh every committed lockfile in the workspace — `Cargo.lock`,
`fuzz/Cargo.lock`, the Node binding lock, and the hash-pinned Python
requirements — run `./scripts/update-lockfiles.sh`. It uses `uv` for the Python
locks (and bootstraps it on Linux/macOS if absent) so each target Python
version's hashed transitive closure can be regenerated without that interpreter
installed. Dependabot also keeps the `.github/requirements` pins current.

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
  [documentation site](https://docs.wickra.org) and the
  `README.md` when behaviour or the public API changes. The docs live in
  a separate git repository: `https://github.com/wickra-lib/wickra-docs`.
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
