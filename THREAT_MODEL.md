# Threat model

This document describes Wickra's attack surface and the threats considered,
together with their mitigations. It complements the security assurance case in
[`SECURITY.md`](SECURITY.md). Wickra is a computational technical-analysis
library (a Rust core with Python, Node.js and WebAssembly bindings plus a C ABI
and the .NET and Go bindings built on it),
not a network service or trading system; the attack surface is correspondingly
small.

## Assets

- **Integrity of computed indicator values** — consumers may use them in
  automated decisions, so silently wrong output is the primary concern.
- **Availability of the calling process** — a library must not crash or hang
  its host on malformed input.
- **Integrity of published artifacts** — the crates, wheels and npm packages
  users install.
- **The build and release pipeline** and its secrets (publishing tokens).

## Actors / trust boundaries

- **Library consumer** (trusted) — calls the API with numeric data. Data may
  originate from untrusted sources (e.g. a market feed), so *input values* are
  treated as untrusted even though the caller is trusted.
- **Optional live feed** — with the `live-binance` feature, data crosses a
  network boundary from an exchange over TLS.
- **Contributors** (semi-trusted) — propose changes via pull requests.
- **Supply chain** — upstream dependencies and the CI/CD platform.

## Threats and mitigations

| Threat | Mitigation |
| --- | --- |
| Memory-safety exploit (buffer overflow, UAF) via crafted input | Pure safe Rust; `unsafe` is forbidden/minimised, so the compiler precludes these classes. |
| Misuse of the C ABI FFI boundary (invalid/dangling handle, undersized batch buffer) | The C ABI (`bindings/c`) is the sole `unsafe` surface. Its shim adds no logic, NULL-checks every handle (returning `NaN`/no-op), writes only into caller-sized buffers, and catches panics so none cross the boundary. A caller passing a non-NULL but dangling pointer is undefined behaviour by C's own contract — out of scope, the same as any C library. |
| Denial of service via malformed/degenerate input (NaN, infinities, extreme magnitudes) | Indicators reject non-finite inputs and validate parameters at construction; update paths are exercised by coverage-guided fuzzing and unit tests for edge cases. |
| Silently incorrect results | 100% line coverage on the core crate; reference-value tests against known-good sources; streaming/batch parity tests. |
| Integer overflow / panics | `clippy::pedantic` with `-D warnings`; debug assertions and overflow checks enabled in test/fuzz builds. |
| Adversary-in-the-middle on the optional live feed | Connection uses TLS via the platform library; transport security is delegated to that reviewed implementation. |
| Compromised dependency (supply chain) | Dependencies pinned (`Cargo.lock`, hash-locked CI requirements), monitored by Dependabot, audited by `cargo-deny` (advisories + licenses) on every change. |
| Malicious or accidental change to `main` | Branch protection requires signed commits and blocks force-push and deletion; all changes flow through pull requests with required CI; static analysis (CodeQL, Clippy) and fuzzing run on every change. |
| Compromised CI / leaked secrets | Workflows use least-privilege `permissions:`; secrets live only as encrypted GitHub Actions secrets; secret scanning with push protection is enabled; workflows are linted by `zizmor`. |
| Tampered release artifact | Releases are built in CI, tags are signed, and assets carry build provenance attestations (verifiable with `gh attestation verify`). |

## Out of scope

- Wickra implements no authentication, authorization or cryptography of its own,
  stores no user data, and exposes no network listener; those threat classes do
  not apply.
- Vulnerabilities in third-party dependencies that do not affect Wickra are
  tracked as exploitability (VEX) records (see [`SECURITY.md`](SECURITY.md)).

## Maintenance

This threat model is reviewed when the architecture changes materially (for
example, a new input family, a new network feature, or a new release channel).
