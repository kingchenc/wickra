<!-- Thanks for contributing to Wickra. Please fill in the sections below.

     Changing the release process, the bindings' shared surface, or the public
     API? There is a longer template that asks what such a change has to answer:
     reopen this pull request with ?template=detailed.md appended to the URL.
     GitHub offers no picker for a second template, so it is only reachable that
     way. -->

## Summary

<!-- What does this PR change, and why? -->

## Related issue

<!-- e.g. Closes #123 -->

## Type of change

- [ ] Bug fix
- [ ] New feature
- [ ] Indicator addition / change
- [ ] Documentation
- [ ] CI / build / tooling

## Checklist

- [ ] `cargo fmt --all --check` is clean.
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` is clean.
- [ ] `cargo test --workspace` passes.
- [ ] New behaviour has tests; bug fixes have a regression test.
- [ ] Public API changes are mirrored in the Python / Node.js / WASM bindings, and the C ABI + C# + Go + Java + R bindings are regenerated
      and their type stubs (If applicable).
- [ ] The relevant page on the [documentation site](https://docs.wickra.org)
      and the `README.md` are updated (If applicable). Docs edits go to a
      separate repository: `https://github.com/wickra-lib/wickra-docs`.
- [ ] An entry was added under `## [Unreleased]` in `CHANGELOG.md`.

## Notes for reviewers

<!-- Anything that needs extra attention, trade-offs, follow-ups. -->
