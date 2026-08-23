"""The type stubs must describe the extension that is actually installed.

`__init__.pyi` was hand-written and had drifted a long way: it declared 72 of
the 520 exported names and annotated them with `numpy.typing.NDArray`, a type
the binding stopped returning when the NumPy dependency was dropped. It is
generated now (`scripts/gen_python_stubs.py`), and these tests are what notices
if it stops matching.
"""

from __future__ import annotations

import inspect
import re
import subprocess
import sys
from pathlib import Path

import pytest

import wickra
import wickra._wickra as _native

REPO_ROOT = Path(__file__).resolve().parents[3]
GENERATOR = REPO_ROOT / "scripts" / "gen_python_stubs.py"
STUB = Path(wickra.__file__).with_name("__init__.pyi")

STUB_SOURCE = STUB.read_text(encoding="utf-8")
STUB_CLASSES = set(re.findall(r"^class (\w+):", STUB_SOURCE, re.M))
STUB_FUNCTIONS = set(re.findall(r"^def (\w+)\(", STUB_SOURCE, re.M))
# Module-level annotated names, of which `__version__` is the only one.
STUB_VARIABLES = set(re.findall(r"^(\w+): ", STUB_SOURCE, re.M))


def native_names() -> set[str]:
    # `__version__` is a public export despite the dunder spelling; nothing else
    # underscored is.
    return {name for name in dir(_native) if not name.startswith("_")} | {"__version__"}


def test_every_exported_name_is_declared_in_the_stub():
    declared = STUB_CLASSES | STUB_FUNCTIONS | STUB_VARIABLES
    missing = sorted(native_names() - declared)
    assert missing == [], f"exported but not in __init__.pyi: {missing}"


def test_the_stub_declares_nothing_the_extension_does_not_export():
    extra = sorted((STUB_CLASSES | STUB_FUNCTIONS | STUB_VARIABLES) - native_names())
    assert extra == [], f"declared in __init__.pyi but not exported: {extra}"


def test_all_matches_the_extension_and_has_no_duplicates():
    listed = list(wickra.__all__)
    duplicates = sorted({name for name in listed if listed.count(name) > 1})
    assert duplicates == [], f"duplicated in __all__: {duplicates}"
    assert set(listed) == native_names()


def test_every_name_in_all_is_importable():
    missing = sorted(name for name in wickra.__all__ if not hasattr(wickra, name))
    assert missing == [], f"in __all__ but not importable from wickra: {missing}"


def test_stub_members_exist_on_the_runtime_class():
    # A method renamed in the binding but not in the stub would otherwise only
    # show up as a type-checker error in someone else's project.
    missing: list[str] = []
    for block in re.finditer(r"^class (\w+):\n((?:    .*\n)*)", STUB_SOURCE, re.M):
        name, body = block.group(1), block.group(2)
        runtime = getattr(_native, name, None)
        if runtime is None:
            continue  # covered by test_the_stub_declares_nothing...
        for member in re.findall(r"^    def (\w+)\(", body, re.M):
            attribute = "__init__" if member == "__init__" else member
            if not hasattr(runtime, attribute):
                missing.append(f"{name}.{member}")
        for prop in re.findall(r"^    @property\n    def (\w+)\(", body, re.M):
            # A PyO3 getter is a `getset_descriptor`, not a `property`, so check
            # that it is a data descriptor rather than testing for `property`.
            descriptor = inspect.getattr_static(runtime, prop, None)
            if descriptor is None or not hasattr(type(descriptor), "__get__"):
                missing.append(f"{name}.{prop} (property)")
    assert missing == [], f"declared in the stub but absent at runtime: {missing}"


def test_the_stub_does_not_annotate_with_numpy():
    # The package has no third-party dependencies; batch results are a stdlib
    # `array.array("d")` or a `Matrix`. The header mentions NumPy in prose, to
    # say that an `ndarray` is an accepted input, so look for the import and the
    # annotations rather than for the word.
    assert "import numpy" not in STUB_SOURCE
    assert "NDArray" not in STUB_SOURCE
    assert not re.search(r"\bnp\.", STUB_SOURCE)


@pytest.mark.skipif(not GENERATOR.exists(), reason="generator is not in the sdist")
def test_the_committed_stub_is_what_the_generator_produces():
    result = subprocess.run(
        [sys.executable, str(GENERATOR), "--check"],
        cwd=REPO_ROOT,
        capture_output=True,
        text=True,
        check=False,
    )
    assert result.returncode == 0, result.stderr or result.stdout
