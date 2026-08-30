#!/usr/bin/env python3
"""Binding READMEs must not use repository-relative links.

Each `bindings/*/README.md` is, or is one workflow line away from being, the long
description of a published package: PyPI renders the Python one, NuGet the C#
one, pkg.go.dev the Go one, r-universe the R one. A link like
`../../docs/COOKBOOK.md` resolves on GitHub and nowhere else -- on a registry page
it is simply broken, and nothing in the build says so, because the file it points
at does exist in the repository.

So the rule is: anything that ships as package metadata links absolutely. The
repository's own README is exempt and deliberately keeps relative links -- it is
read on GitHub far more than anywhere else, and that is the convention the main
wickra repository uses too.

Run from the repository root:  python scripts/check_readme_links.py
"""

from __future__ import annotations

import glob
import os
import re
import sys

ROOT = os.path.normpath(os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))

# A markdown link target that is neither absolute nor a same-page anchor. Also
# catches HTML `src=`/`href=` attributes, which the banner markup uses.
LINK = re.compile(r"\]\(\s*(?!https?://|#|mailto:)([^)\s]+)")
ATTR = re.compile(r"(?:src|href)=\"(?!https?://|#|mailto:)([^\"]+)\"")


def relative_targets(text: str, package_dir: str) -> list[str]:
    """Relative targets that will not resolve once the package is unpacked.

    A relative link is only broken off GitHub when it points at something that
    does not travel with the package. One that stays inside the package
    directory does travel and resolves wherever the package is unpacked:
    `man/figures/logo.png` in the R binding is the standard R convention for
    exactly that, and both CRAN and r-universe render it.

    So the test is not "is it relative" but "does it leave the package, or point
    at nothing".
    """
    found = ([m.group(1) for m in LINK.finditer(text)]
             + [m.group(1) for m in ATTR.finditer(text)])
    escaping = []
    for target in found:
        clean = target.split("#", 1)[0].split("?", 1)[0]
        if not clean:
            continue
        resolved = os.path.normpath(os.path.join(package_dir, clean))
        inside = os.path.commonpath([resolved, package_dir]) == package_dir
        if not inside or not os.path.exists(resolved):
            escaping.append(target)
    return escaping


def main() -> int:
    paths = sorted(glob.glob(os.path.join(ROOT, "bindings", "*", "README.md")))
    if not paths:
        print("no binding READMEs found", file=sys.stderr)
        return 1

    failures = []
    for path in paths:
        rel = os.path.relpath(path, ROOT).replace(os.sep, "/")
        with open(path, encoding="utf-8") as handle:
            found = relative_targets(handle.read(), os.path.dirname(path))
        if found:
            failures.append(f"{rel}: {', '.join(sorted(set(found)))}")
        print(f"  {rel:<28} {'broken links: ' + str(len(found)) if found else 'all links resolve'}")

    if failures:
        print(
            "\nthese READMEs ship as package long descriptions, where a link "
            "that leaves the package, or points at nothing, is dead:",
            file=sys.stderr,
        )
        for failure in failures:
            print(f"  {failure}", file=sys.stderr)
        print(
            "\nuse https://github.com/wickra-lib/wickra/blob/main/<path> "
            "instead.",
            file=sys.stderr,
        )
        return 1

    print(f"\nall {len(paths)} binding READMEs link only to what travels with them.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
