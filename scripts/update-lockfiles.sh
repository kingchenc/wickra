#!/usr/bin/env bash
#
# Regenerate every committed lockfile in the workspace:
#   - Rust:   Cargo.lock, fuzz/Cargo.lock        (cargo update)
#   - Node:   bindings/node/package-lock.json     (npm install --package-lock-only)
#   - Python: .github/requirements/*.txt          (uv pip compile --generate-hashes)
#
# Run from anywhere; the script cd's to the repo root itself:
#
#     ./scripts/update-lockfiles.sh
#
# The Python locks are hash-pinned (OpenSSF Scorecard PinnedDependencies) and
# generated with uv rather than pip-tools because uv can resolve a *target*
# Python version's full transitive closure — with hashes — without that
# interpreter being installed locally. That is required for the numpy cp39/cp313
# split: numpy ships no single release with wheels for both, so ci-dev is locked
# twice (Python 3.9 and Python 3.10+). If uv is not on PATH the script
# bootstraps a local copy (Linux/macOS); on Windows install uv first
# (https://docs.astral.sh/uv/getting-started/installation/).
#
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

echo "==> Rust (Cargo.lock, fuzz/Cargo.lock)"
cargo update
(cd fuzz && cargo update)

echo "==> Node (bindings/node/package-lock.json)"
(cd bindings/node && npm install --package-lock-only --no-audit --no-fund)

echo "==> Python (.github/requirements/*.txt via uv)"
if ! command -v uv >/dev/null 2>&1; then
  echo "    uv not found on PATH; bootstrapping a local copy..."
  curl -LsSf https://astral.sh/uv/install.sh | sh
  export PATH="$HOME/.local/bin:$PATH"
fi

req=".github/requirements"
cc="./scripts/update-lockfiles.sh"
uv pip compile --quiet --python-version 3.9  --generate-hashes --custom-compile-command "$cc" "$req/ci-dev-py39.in" -o "$req/ci-dev-py39.txt"
uv pip compile --quiet --python-version 3.11 --generate-hashes --custom-compile-command "$cc" "$req/ci-dev-py3.in"  -o "$req/ci-dev-py3.txt"
uv pip compile --quiet --python-version 3.11 --generate-hashes --custom-compile-command "$cc" "$req/bench.in"       -o "$req/bench.txt"

echo "==> Done. Review 'git diff' before committing."
