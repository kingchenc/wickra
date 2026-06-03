# Security Policy

## Supported versions

Wickra is pre-1.0. Security fixes are applied to the latest released `0.5.x`
version only; please upgrade to the newest release before reporting an issue.

| Version | Supported |
| --- | --- |
| 0.5.x (latest) | :white_check_mark: |
| older 0.5.x | :x: |

## Reporting a vulnerability

**Do not open a public issue for a security vulnerability.**

Report it privately through one of:

- GitHub's [private vulnerability reporting](https://github.com/wickra-lib/wickra/security/advisories/new)
  ("Report a vulnerability" under the repository's *Security* tab), or
- email to **support@wickra.org** with a subject line starting with
  `[wickra security]`.

Please include:

- the affected version(s) and platform / language binding,
- a description of the issue and its impact,
- steps to reproduce, ideally a minimal proof of concept.

## What to expect

- An acknowledgement within **5 working days**.
- An assessment and, if confirmed, a planned fix with a target release.
- Coordinated disclosure: we will agree on a disclosure date with you and
  credit you in the release notes unless you prefer to stay anonymous.

## Scope

In scope: the published crates (`wickra-core`, `wickra-data`, `wickra`), the
PyPI/npm packages, and the build/release workflows in `.github/workflows/`.

Out of scope: vulnerabilities in third-party dependencies (report those
upstream; we track them via Dependabot and `cargo-deny`).
