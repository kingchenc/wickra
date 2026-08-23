#!/usr/bin/env python3
"""Assert that every binding exposes the same indicator surface.

Each binding is generated or hand-written separately, and each has its own test
suite, so a method that goes missing in one of them fails nowhere: the WASM
binding shipped 73 classes without `isReady`/`warmupPeriod` and 63 without
`batch`, and the Node loader shipped 518 exports that resolved to `undefined`,
both for months. Nothing compared the bindings *to each other*.

The C ABI header is the source of truth -- every binding is a consumer of it --
so this reads `wickra_<snake>_<method>` out of the header, derives what each
indicator must expose, and checks that claim against each binding's own public
surface, spelled the way that language spells it.

Two shapes exist. An `Indicator` carries a warmup and a ready flag; a
`BarBuilder` does not, because one candle can complete any number of bars, so
there is nothing for it to be ready for. The check is two-sided: a bar builder
that grew an `isReady` is as wrong as an indicator that lost one.

Bindings may expose more than the contract -- Node and Python publish parameter
accessors that Go and C# do not, which is a language-idiom difference and not
drift -- so this checks that the contract is present, not that nothing else is.

WASM is the one binding whose surface lives in a build artifact rather than in
the repository, so it cannot be read statically. It is held to the same contract
at runtime by bindings/wasm/tests/completeness.test.js, which derives its
expectations from the same manifest this script uses.

Run from the repository root:  python scripts/check_binding_surface.py
"""

from __future__ import annotations

import glob
import json
import os
import re
import sys

ROOT = os.path.normpath(os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
GOLDEN = os.path.join(ROOT, "testdata", "golden")

# The contract, in language-neutral terms. `dispose` is deliberately absent: a
# handle-owning binding needs one and a garbage-collected one does not, which is
# a property of the language rather than of the indicator.
CONTRACT = ("update", "batch", "reset", "name")
STATEFUL = ("warmup", "ready")


def read(path: str) -> str:
    with open(os.path.join(ROOT, path), encoding="utf-8") as handle:
        return handle.read()


def snake(name: str) -> str:
    """`BollingerBands` -> `bollinger_bands`, the C ABI's symbol spelling."""
    spaced = re.sub(r"(?<=[a-z0-9])(?=[A-Z])", "_", name)
    return re.sub(r"(?<=[A-Z])(?=[A-Z][a-z])", "_", spaced).lower()


def truth() -> dict[str, frozenset[str]]:
    """The expected capability set per indicator, read from the C ABI header."""
    header = read("bindings/c/include/wickra.h")
    symbols = set(re.findall(r"\bwickra_([a-z0-9_]+)\s*\(", header))
    expected = {}
    for canonical in json.load(open(os.path.join(GOLDEN, "names.json"), encoding="utf-8")):
        prefix = snake(canonical)
        if f"{prefix}_new" not in symbols:
            raise SystemExit(f"{canonical}: no wickra_{prefix}_new in the C ABI header")
        caps = set(CONTRACT)
        if f"{prefix}_warmup_period" in symbols:
            caps.add("warmup")
        if f"{prefix}_is_ready" in symbols:
            caps.add("ready")
        expected[canonical] = frozenset(caps)
    return expected


# --- one extractor per binding: canonical -> the capabilities it exposes -----


def surface_c() -> dict[str, set[str]]:
    header = read("bindings/c/include/wickra.h")
    symbols = set(re.findall(r"\bwickra_([a-z0-9_]+)\s*\(", header))
    spelling = {"update": "update", "batch": "batch", "reset": "reset", "name": "name",
                "warmup": "warmup_period", "ready": "is_ready"}
    return {
        canonical: {cap for cap, sym in spelling.items() if f"{snake(canonical)}_{sym}" in symbols}
        for canonical in truth()
    }


def surface_go() -> dict[str, set[str]]:
    methods: dict[str, set[str]] = {}
    for match in re.finditer(r"^func \(ind \*(\w+)\) (\w+)\(", read("bindings/go/indicators_gen.go"), re.M):
        methods.setdefault(match.group(1), set()).add(match.group(2))
    spelling = {"update": "Update", "batch": "Batch", "reset": "Reset", "name": "Name",
                "warmup": "WarmupPeriod", "ready": "IsReady"}
    return {c: {cap for cap, m in spelling.items() if m in ms} for c, ms in methods.items()}


def surface_csharp() -> dict[str, set[str]]:
    source = read("bindings/csharp/Wickra/Generated/Indicators.g.cs")
    spelling = {"update": "Update", "batch": "Batch", "reset": "Reset", "name": "Name",
                "warmup": "WarmupPeriod", "ready": "IsReady"}
    out = {}
    for block in re.finditer(r"public sealed class (\w+) : IDisposable\s*\{(.*?)\n\}", source, re.S):
        members = set(re.findall(r"public [\w\[\]<>?., ]+ (\w+)\(", block.group(2)))
        out[block.group(1)] = {cap for cap, m in spelling.items() if m in members}
    return out


def surface_java() -> dict[str, set[str]]:
    spelling = {"update": "update", "batch": "batch", "reset": "reset", "name": "name",
                "warmup": "warmupPeriod", "ready": "isReady"}
    out = {}
    for path in glob.glob(os.path.join(ROOT, "bindings/java/src/main/java/org/wickra/*.java")):
        with open(path, encoding="utf-8") as handle:
            members = set(re.findall(r"public [\w\[\]<>., ]+ (\w+)\(", handle.read()))
        out[os.path.basename(path)[:-5]] = {cap for cap, m in spelling.items() if m in members}
    return out


def surface_node() -> dict[str, set[str]]:
    declarations = read("bindings/node/index.d.ts")
    spelling = {"update": "update", "batch": "batch", "reset": "reset", "name": "name",
                "warmup": "warmupPeriod", "ready": "isReady"}
    classes = {}
    for block in re.finditer(r"export declare class (\w+) \{(.*?)\n\}", declarations, re.S):
        members = set(re.findall(r"^\s{2}(\w+)\(", block.group(2), re.M))
        classes[block.group(1)] = {cap for cap, m in spelling.items() if m in members}
    manifest = json.load(open(os.path.join(GOLDEN, "node_manifest.json"), encoding="utf-8"))
    return {
        entry["canonical"]: classes[entry["native"]]
        for entry in manifest
        if entry["native"] in classes
    }


def python_class_names() -> dict[str, str]:
    """canonical -> the name the Python module registers the class under.

    Two forms: a `#[pyclass(name = "...")]` on a struct that holds the core type
    (`PyBb` holds `wc::BollingerBands`, so the struct name is not the canonical),
    and a macro invocation whose first argument is `Py<Canonical>`.
    """
    source = read("bindings/python/src/lib.rs")
    mapping: dict[str, str] = {}
    for match in re.finditer(r"#\[pyclass\(([^)]*)\)\]", source):
        name_attr = re.search(r'name\s*=\s*"([^"]+)"', match.group(1))
        if name_attr is None:
            continue
        tail = source[match.end() : match.end() + 600]
        core = re.search(r"inner:\s*(?:wc|wickra_core|wickra_data)::(\w+)", tail)
        if core is not None:
            mapping[core.group(1)] = name_attr.group(1)
            continue
        struct = re.search(r"\bstruct\s+Py(\w+)", tail)
        if struct is not None:
            mapping[struct.group(1)] = name_attr.group(1)
    for match in re.finditer(r"^\s*\w+!\s*\(\s*Py(\w+)\s*,([^;]*?)\);", source, re.M | re.S):
        literals = re.findall(r'"([^"]+)"', match.group(2))
        if len(literals) == 1:
            mapping.setdefault(match.group(1), literals[0])
    return mapping


def surface_python() -> dict[str, set[str]]:
    stubs = read("bindings/python/python/wickra/__init__.pyi")
    spelling = {"update": "update", "batch": "batch", "reset": "reset", "name": "name",
                "warmup": "warmup_period", "ready": "is_ready"}
    classes = {}
    for block in re.finditer(r"^class (\w+):\n((?:(?!^class ).*\n)*)", stubs, re.M):
        members = set(re.findall(r"^\s{4}def (\w+)\(", block.group(2), re.M))
        classes[block.group(1)] = {cap for cap, m in spelling.items() if m in members}
    return {
        canonical: classes[registered]
        for canonical, registered in python_class_names().items()
        if registered in classes
    }


def surface_r() -> dict[str, set[str]]:
    """R's surface is its `.Call` shims: `methods.R` dispatches generically, so a
    missing shim is the only way a method can be absent."""
    shims = set(re.findall(r"^SEXP wk_([a-z0-9_]+)\(SEXP", read("bindings/r/src/wickra.c"), re.M))
    constructors = set(re.findall(r"^(\w+) <- function\(", read("bindings/r/R/indicators.R"), re.M))
    spelling = {"update": "update", "batch": "batch", "reset": "reset", "name": "name",
                "warmup": "warmup_period", "ready": "is_ready"}
    out = {}
    for canonical in truth():
        if canonical not in constructors:
            continue
        prefix = snake(canonical)
        out[canonical] = {cap for cap, m in spelling.items() if f"{prefix}_{m}" in shims}
    return out


BINDINGS = {
    "C": surface_c,
    "Go": surface_go,
    "C#": surface_csharp,
    "Java": surface_java,
    "Node": surface_node,
    "Python": surface_python,
    "R": surface_r,
}

# R has no batch shim for the families whose per-bar input is a snapshot or whose
# output length is data-dependent: `batch()` forwards equal-length numeric
# columns and cannot express a per-bar width. Every other binding has one. This
# is a real gap, pinned here so it cannot widen without the number changing --
# see D3d in the audit and the R golden suite, which asserts the same split.
R_BATCH_GAP = 39


def main() -> int:
    expected = truth()
    failures: list[str] = []

    for label, extract in BINDINGS.items():
        actual = extract()
        missing_classes = sorted(set(expected) - set(actual))
        if missing_classes:
            failures.append(
                f"{label}: {len(missing_classes)} indicators absent, e.g. {missing_classes[:5]}"
            )
            continue

        gaps: list[str] = []
        extras: list[str] = []
        waived = 0
        for canonical, want in expected.items():
            have = actual[canonical]
            for cap in sorted(want - have):
                if label == "R" and cap == "batch":
                    waived += 1
                    continue
                gaps.append(f"{canonical}.{cap}")
            # The other direction: a bar builder that grew a warmup or a ready
            # flag has drifted just as far as an indicator that lost one.
            for cap in sorted((have & set(STATEFUL)) - want):
                extras.append(f"{canonical}.{cap}")

        before = len(failures)
        if gaps:
            failures.append(f"{label}: {len(gaps)} missing, e.g. {gaps[:5]}")
        if extras:
            failures.append(f"{label}: {len(extras)} beyond the C ABI shape, e.g. {extras[:5]}")
        if label == "R" and waived != R_BATCH_GAP:
            failures.append(
                f"R: {waived} indicators without a batch shim, expected {R_BATCH_GAP}"
                " — update R_BATCH_GAP here and in the R golden suite"
            )

        note = f"  ({waived} without a batch shim)" if waived else ""
        verdict = "contract complete" if len(failures) == before else "DRIFTED"
        print(f"  {label:<7} {len(expected):>4} indicators, {verdict}{note}")

    stateful = sum(1 for caps in expected.values() if "ready" in caps)
    print(
        f"\n  C ABI declares {len(expected)} indicators:"
        f" {stateful} with a warmup and a ready flag,"
        f" {len(expected) - stateful} bar builders without one."
    )
    print("  WASM is checked against the same contract at runtime by"
          " bindings/wasm/tests/completeness.test.js.")

    if failures:
        print("\nbinding surfaces disagree:", file=sys.stderr)
        for line in failures:
            print(f"  {line}", file=sys.stderr)
        return 1
    print("\nevery binding exposes the contract the C ABI declares.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
