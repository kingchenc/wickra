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

## Continuity and succession

The project is designed to survive the loss of any single individual, so that
issues can be triaged, proposed changes accepted, and releases published within
one week of confirmed loss of the maintainer:

- **Credentials.** All credentials required to operate the project — the
  `wickra-lib` GitHub organization, the publishing tokens for crates.io, PyPI
  and npm, and the `wickra.org` domain registrar — are stored in a password
  manager. A trusted contact (a family member) holds **emergency access** to
  that password manager and can obtain these credentials if the maintainer can
  no longer continue.
- **Continuity actions.** With that access, the trusted contact (or a delegate
  they appoint) can create and close issues, accept pull requests, and publish
  releases through the existing CI/CD workflows.
- **Account recovery.** The maintainer's GitHub account has recovery configured,
  and ownership of the `wickra-lib` organization can be transferred to a new
  maintainer.
- **Legal rights.** Legal rights to the project name and DNS are covered by the
  maintainer's estate arrangements.

## Code of conduct

All participants are expected to follow the
[Code of Conduct](CODE_OF_CONDUCT.md).

## Changes to this document

This governance model may evolve as the project grows. Changes are made via
pull request and take effect once merged.
