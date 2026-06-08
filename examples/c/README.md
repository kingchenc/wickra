# Wickra — C / C++ examples

The Wickra C ABI is a single shared/static library plus a generated header
([`bindings/c/include/wickra.h`](../../bindings/c/include/wickra.h)). Any
C-capable language links against the same artifact; these examples show the
plain-C path.

## Build the library

From the workspace root:

```sh
cargo build -p wickra-c --release
```

This produces, in `target/release/`:

| Platform | Shared library | Link target |
|----------|----------------|-------------|
| Linux    | `libwickra.so`     | `-lwickra` |
| macOS    | `libwickra.dylib`  | `-lwickra` |
| Windows (MSVC) | `wickra.dll` | `wickra.dll.lib` (import lib) |

A static library (`libwickra.a` / `wickra.lib`) is emitted alongside.

## Build and run the smoke example

### With CMake (portable, used by CI)

```sh
cmake -S examples/c -B examples/c/build -DWICKRA_LIB_DIR="$PWD/target/release"
cmake --build examples/c/build
ctest --test-dir examples/c/build --output-on-failure
```

### Directly with a compiler

```sh
# Linux / macOS
cc examples/c/smoke.c -I bindings/c/include -L target/release -lwickra -lm -o smoke
LD_LIBRARY_PATH=target/release ./smoke        # macOS: DYLD_LIBRARY_PATH

# Windows (MinGW gcc, linking the DLL directly)
gcc examples/c/smoke.c -I bindings/c/include target/release/wickra.dll -lm -o smoke.exe
```

Expected output:

```
OK: wickra C ABI smoke passed (SMA streaming + batch + reset + NULL-safety + free)
```

## Usage shape

Every indicator follows the same five-function pattern over an opaque handle:

```c
#include "wickra.h"

struct Sma *sma = wickra_sma_new(14);     /* NULL on invalid params */
double v = wickra_sma_update(sma, 42.0);  /* NaN during warmup */
wickra_sma_reset(sma);                     /* back to fresh state */
wickra_sma_free(sma);                      /* exactly once per _new */
```

There is no RAII across the C boundary: every `wickra_<ind>_new` must be paired
with exactly one `wickra_<ind>_free`. All functions are NULL-safe (a NULL handle
yields `NaN` / a no-op, never a crash).
