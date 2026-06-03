# Governance

Wickra is an open-source project maintained under a **single-maintainer
("BDFL") model**. This document describes how decisions are made and how the
project is run, so contributors know what to expect.

## Roles

- **Maintainer.** The maintainer (see [`MAINTAINERS.md`](MAINTAINERS.md)) is
  responsible for the project's direction, reviews and merges changes, cuts
  releases, and has final say on all technical and project decisions.
- **Contributors.** Anyone who proposes changes via pull requests, files
  issues, improves documentation, or otherwise participates. Contributors do
  not need any special status to take part.

## Decision-making

- Day-to-day technical decisions (APIs, indicator implementations, refactors)
  are made by the maintainer, informed by discussion on issues and pull
  requests.
- Proposals are raised as GitHub issues or pull requests. Significant or
  breaking changes should be opened as an issue first to agree on the approach
  before implementation.
- The maintainer aims to act transparently: rationale for non-trivial decisions
  is recorded in the relevant issue, pull request, or commit message.

## Contribution flow

All changes — including the maintainer's own — go through pull requests so that
CI (tests, linting, static analysis) runs against them, and so the change
history is reviewable. Contribution requirements are documented in
[`CONTRIBUTING.md`](CONTRIBUTING.md), including the Developer Certificate of
Origin sign-off that every commit must carry.

## Becoming a maintainer

The project currently has one maintainer. Maintainership may be extended to
contributors who have demonstrated sustained, high-quality involvement, at the
current maintainer's discretion. If the project grows to multiple maintainers,
this document will be updated to describe shared decision-making.

## Code of conduct

All participants are expected to follow the
[Code of Conduct](CODE_OF_CONDUCT.md).

## Changes to this document

This governance model may evolve as the project grows. Changes are made via
pull request and take effect once merged.
