<!--
Thanks for contributing to Wickra!
Please fill in the sections below. Delete any that don't apply.
-->

## Summary

<!-- 1–3 sentences: what does this PR change and why? -->

## Type of change

- [ ] Bug fix (Non-breaking change which fixes an issue)
- [ ] New feature (Non-breaking change which adds functionality)
- [ ] Breaking change (Fix or feature that changes existing public API)
- [ ] Performance improvement
- [ ] Refactor (No functional change)
- [ ] Documentation only
- [ ] CI / build / tooling

## Affected surfaces

- [ ] Rust crate (`crates/wickra`)
- [ ] Python binding (`bindings/python`)
- [ ] Node.js binding (`bindings/node`)
- [ ] WebAssembly binding (`bindings/wasm`)
- [ ] Examples / docs

## Linked issues

<!-- "Closes #123", "Refs #456". One per line. -->

Closes #

## How was this tested?

<!--
- Unit tests added / updated under `crates/*/tests/` or `bindings/*/tests/`
- Property / fuzz tests touched? (Under `fuzz/`)
- Manual repro steps, if applicable
- Benchmarks run (Paste before/after if perf-sensitive)
-->

## Numerical correctness (If you touched an indicator)

- [ ] Output matches an existing reference (TA-Lib, paper, prior Wickra release) within documented tolerance
- [ ] Streaming `update()` matches batch / `from_slice` output on the same input
- [ ] Edge cases covered: empty input, single point, NaN, leading warm-up window

## Performance impact (If applicable)

| Benchmark | Before | After | Δ |
| --------- | ------ | ----- | - |
|           |        |       |   |

## Checklist

- [ ] `cargo fmt --all` and `cargo clippy --all-targets -- -D warnings` are clean
- [ ] `cargo test --workspace` passes locally
- [ ] Binding tests run (If a binding changed)
- [ ] Public API changes are reflected in `CHANGELOG.md`
- [ ] Public API changes are reflected in rustdoc / README / examples
- [ ] No `todo*.md` or other local-only notes are staged
- [ ] License header / `LICENSE` reference unchanged (PolyForm-NC-1.0.0)

## Notes for reviewers

<!-- Anything reviewers should look at first, known follow-ups, deliberately out-of-scope items. -->
