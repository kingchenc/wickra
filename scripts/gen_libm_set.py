"""Emit the set of indicators whose output can depend on the platform math
library, for the golden runners to consult when choosing a tolerance.

Every binding calls into the same Rust core, so an indicator built only from
IEEE-754 arithmetic (`+ - * / sqrt`, comparisons, min/max) produces bit-identical
results everywhere: those operations are correctly rounded by the standard, Rust
performs no reassociation and no FMA contraction without opt-in, and the
supported targets are all SSE2/NEON, so there is no x87 excess-precision to
worry about.

An indicator that reaches a transcendental is different. No mainstream libm
rounds `ln`, `sin`, `atan` and friends correctly, and implementations disagree in
the last bit -- measured on `LinRegAngle` against 200-bit arithmetic, Rust's
`atan(x).to_degrees()` differs from the correctly rounded result on 24 of 67
bars of the golden input. That bit is a property of the machine, so those
indicators need a looser comparison than the rest.

Writes `testdata/golden/libm_dependent.txt`, one canonical indicator name per
line. Run from the repository root:

    python scripts/gen_libm_set.py

The runners read that file rather than each carrying their own copy of the list,
so it cannot drift out of sync with the catalogue.
"""

import glob
import io
import os
import re
import sys

LIBM = re.compile(
    r"\.(sin|cos|tan|asin|acos|atan|atan2|sinh|cosh|tanh|exp|exp2|exp_m1"
    r"|ln|ln_1p|log|log2|log10|powf|cbrt|hypot)\("
)

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
IND = os.path.join(ROOT, "crates", "wickra-core", "src", "indicators")
OUT = os.path.join(ROOT, "testdata", "golden", "libm_dependent.txt")


def real_code(path):
    """The file with its test module, doc examples and comments removed.

    Doc comments carry `.sin()` in their examples and the test modules build
    their inputs from sine waves, so scanning the raw text classifies most of
    the catalogue as libm-dependent when almost none of it is.
    """
    src = io.open(path, encoding="utf-8").read()
    cut = src.find("#[cfg(test)]")
    if cut != -1:
        src = src[:cut]
    return "\n".join(
        line
        for line in src.split("\n")
        if not line.lstrip().startswith(("///", "//!", "//"))
    )


def main():
    modules = sorted(
        os.path.basename(p)[:-3]
        for p in glob.glob(os.path.join(IND, "*.rs"))
        if os.path.basename(p) != "mod.rs"
    )
    code = {m: real_code(os.path.join(IND, m + ".rs")) for m in modules}

    # Public type -> declaring module, from the re-export block in mod.rs.
    mod_rs = io.open(os.path.join(IND, "mod.rs"), encoding="utf-8").read()
    owner = {}
    for m in re.finditer(r"^pub use (\w+)::\{?([^;}]+)\}?;", mod_rs, re.M):
        module = m.group(1)
        for name in m.group(2).split(","):
            name = name.strip()
            if name:
                owner[name] = module

    tainted = {m for m in modules if LIBM.search(code[m])}
    direct = len(tainted)

    # An indicator that composes a libm-dependent one inherits the dependency.
    changed = True
    while changed:
        changed = False
        for m in modules:
            if m in tainted:
                continue
            for type_name, module in owner.items():
                if (
                    module in tainted
                    and module != m
                    and re.search(r"\b%s\b" % re.escape(type_name), code[m])
                ):
                    tainted.add(m)
                    changed = True
                    break

    # Only names that name a fixture: the re-export block also carries the
    # `*Output` structs, which no runner ever looks up.
    fixtures = os.path.join(ROOT, "testdata", "golden")
    names = sorted(
        t
        for t, module in owner.items()
        if module in tainted
        and os.path.isfile(os.path.join(fixtures, "g_%s.csv" % t))
    )
    io.open(OUT, "w", encoding="utf-8", newline="\n").write("\n".join(names) + "\n")

    exact = sum(
        1
        for t in owner
        if os.path.isfile(os.path.join(fixtures, "g_%s.csv" % t))
    ) - len(names)
    print(
        "wrote %s: %d libm-dependent names from %d modules "
        "(%d direct, %d transitive); %d further fixtures are IEEE-exact"
        % (
            os.path.relpath(OUT, ROOT).replace(os.sep, "/"),
            len(names),
            len(tainted),
            direct,
            len(tainted) - direct,
            exact,
        )
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
