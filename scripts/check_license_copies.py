#!/usr/bin/env python3
"""Every published package must carry the licence texts it claims.

The repository is dual-licensed and every manifest says so, but an SPDX
expression is a reference to two documents, not the documents. A package that
ships the expression alone leaves whoever received it with terms they have to go
and find.

The npm packages are handled at publish time (see release.yml), because npm is
happy to pack a file that appears in the working tree moments beforehand. Cargo
is not: it decides what to package from git, so a copy that is untracked makes
`cargo publish` refuse the dirty tree, and a copy that is gitignored is dropped
from the .crate entirely. Committed copies are the only thing that works, and the
cost of a committed copy is drift -- which is what this checks.

Locations are derived, not listed: every workspace member that can go to
crates.io, plus the Python binding, whose wheel and sdist are built by maturin
from that directory. Add a publishable crate and this starts requiring its
licences without anyone remembering to edit the list.

Run from the repository root:  python scripts/check_license_copies.py
"""

from __future__ import annotations

import json
import os
import re
import sys

ROOT = os.path.normpath(os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
LICENCES = ("LICENSE-MIT", "LICENSE-APACHE")
# maturin builds the wheel and sdist from here, and picks up any LICEN[CS]E* it
# finds beside the manifest (PEP 639). Not a crates.io package -- its Cargo.toml
# says publish = false -- so it cannot be derived from the workspace below.
EXTRA = ("bindings/python",)

# SPDX-named copies at LICENSES/<identifier>.txt. Not a package -- this is the
# layout licence scanners look for, so the `MIT OR Apache-2.0` that every
# manifest declares resolves to the actual texts without anyone having to guess
# which root file is which. Byte-identical to that pair, hence checked here
# rather than trusted.
SPDX_COPIES = {
    "LICENSES/MIT.txt": "LICENSE-MIT",
    "LICENSES/Apache-2.0.txt": "LICENSE-APACHE",
}


def workspace_members() -> list[str]:
    with open(os.path.join(ROOT, "Cargo.toml"), encoding="utf-8") as handle:
        text = handle.read()
    block = re.search(r"(?ms)^members\s*=\s*\[(.*?)\]", text)
    if block is None:
        raise SystemExit("no [workspace] members list in Cargo.toml")
    return re.findall(r'"([^"]+)"', block.group(1))


def publishable(member: str) -> bool:
    """False when the member's manifest opts out of crates.io."""
    manifest = os.path.join(ROOT, member, "Cargo.toml")
    if not os.path.isfile(manifest):
        raise SystemExit(f"workspace member {member} has no Cargo.toml")
    with open(manifest, encoding="utf-8") as handle:
        return re.search(r"(?m)^publish\s*=\s*false", handle.read()) is None


def main() -> int:
    originals = {}
    for name in LICENCES:
        path = os.path.join(ROOT, name)
        if not os.path.isfile(path):
            print(f"{name} is missing from the repository root", file=sys.stderr)
            return 1
        with open(path, "rb") as handle:
            originals[name] = handle.read()

    directories = [m for m in workspace_members() if publishable(m)] + list(EXTRA)
    failures = []
    for directory in directories:
        problems = []
        for name in LICENCES:
            path = os.path.join(ROOT, directory, name)
            if not os.path.isfile(path):
                problems.append(f"{directory}/{name} is missing")
                continue
            with open(path, "rb") as handle:
                if handle.read() != originals[name]:
                    problems.append(f"{directory}/{name} differs from the root copy")
        failures.extend(problems)
        status = "licence texts present" if not problems else f"{len(problems)} problem(s)"
        print(f"  {directory:<32} {status}")

    spdx_problems = []
    for copy, original in sorted(SPDX_COPIES.items()):
        path = os.path.join(ROOT, copy)
        if not os.path.isfile(path):
            spdx_problems.append(f"{copy} is missing")
            continue
        with open(path, "rb") as handle:
            if handle.read() != originals[original]:
                spdx_problems.append(f"{copy} differs from {original}")
    failures.extend(spdx_problems)
    status = ("SPDX-named copies present" if not spdx_problems
              else f"{len(spdx_problems)} problem(s)")
    print(f"  {'LICENSES/':<32} {status}")

    if failures:
        print("\npublished packages would ship without their licence texts:", file=sys.stderr)
        for failure in failures:
            print(f"  {failure}", file=sys.stderr)
        print("\ncopy LICENSE-MIT and LICENSE-APACHE from the repository root.", file=sys.stderr)
        return 1

    npm_problems = check_npm()
    if npm_problems:
        print("\nthe npm packages would ship without their licence texts:",
              file=sys.stderr)
        for failure in npm_problems:
            print(f"  {failure}", file=sys.stderr)
        return 1

    print(f"\n{len(directories)} published packages carry both licence texts, "
          "and the npm side is staged and allow-listed.")
    return 0


def check_npm() -> list[str]:
    """The npm side: the staging step must exist, and `files` must name the texts."""
    problems = []
    release_yml = os.path.join(ROOT, ".github", "workflows", "release.yml")
    if not os.path.isfile(release_yml):
        return ["release.yml is missing"]
    with open(release_yml, encoding="utf-8") as handle:
        workflow = handle.read()
    if "cp ../../LICENSE-MIT ../../LICENSE-APACHE" not in workflow:
        problems.append(
            "release.yml stages no licence copies for the npm packages -- the "
            "delegation in this file's header would be to nothing")

    manifests = [os.path.join(ROOT, "bindings", "node", "package.json")]
    npm_dir = os.path.join(ROOT, "bindings", "node", "npm")
    if os.path.isdir(npm_dir):
        manifests += [os.path.join(npm_dir, name, "package.json")
                      for name in sorted(os.listdir(npm_dir))]
    for manifest in manifests:
        if not os.path.isfile(manifest):
            continue
        with open(manifest, encoding="utf-8") as handle:
            declared = json.load(handle)
        listed = declared.get("files")
        rel = os.path.relpath(manifest, ROOT).replace(os.sep, "/")
        if listed is None:
            continue  # no allowlist: npm packs everything, nothing can be dropped
        absent = [n for n in LICENCES if n not in listed]
        if absent:
            problems.append(f"{rel}: `files` omits {' and '.join(absent)}")
        else:
            print(f"  {rel:<44} allow-listed")
    return problems


if __name__ == "__main__":
    sys.exit(main())
