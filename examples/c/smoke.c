/* Smoke test for the Wickra C ABI.
 *
 * This is the one test the Rust unit tests structurally cannot do: it links a
 * foreign C consumer against the generated `wickra.h` + the compiled library and
 * exercises the real FFI boundary (symbol export, header correctness, opaque
 * handle, pointer ownership, `_free`). If this passes, every C-capable language
 * (C, C++, Go, C#, Java, R) can link the same way.
 *
 * Build (from the workspace root, after `cargo build -p wickra-c --release`):
 *   cc examples/c/smoke.c -I bindings/c/include target/release/<lib> -lm -o smoke
 */

#include "wickra.h"
#include <math.h>
#include <stdio.h>

static int near(double a, double b) { return fabs(a - b) < 1e-9; }

int main(void) {
    struct Sma *sma = wickra_sma_new(3);
    if (sma == NULL) {
        printf("FAIL: wickra_sma_new returned NULL\n");
        return 1;
    }

    /* SMA(3): first two outputs are warmup (NaN), then the trailing mean. */
    double in[5] = {1.0, 2.0, 3.0, 4.0, 5.0};
    double r0 = wickra_sma_update(sma, in[0]); /* NaN  (1/3) */
    double r1 = wickra_sma_update(sma, in[1]); /* NaN  (2/3) */
    double r2 = wickra_sma_update(sma, in[2]); /* 2.0  (1+2+3)/3 */
    double r3 = wickra_sma_update(sma, in[3]); /* 3.0  (2+3+4)/3 */

    if (!isnan(r0) || !isnan(r1)) {
        printf("FAIL: warmup not NaN (%f %f)\n", r0, r1);
        return 1;
    }
    if (!near(r2, 2.0) || !near(r3, 3.0)) {
        printf("FAIL: streaming values (%f %f), expected (2.0 3.0)\n", r2, r3);
        return 1;
    }

    /* Batch over a reset instance must reproduce the streaming result. */
    wickra_sma_reset(sma);
    double out[5];
    wickra_sma_batch(sma, in, out, 5);
    if (!isnan(out[0]) || !isnan(out[1]) ||
        !near(out[2], 2.0) || !near(out[3], 3.0) || !near(out[4], 4.0)) {
        printf("FAIL: batch mismatch (%f %f %f %f %f)\n",
               out[0], out[1], out[2], out[3], out[4]);
        return 1;
    }

    /* NULL handle is a defined no-op / NaN, never a crash. */
    if (!isnan(wickra_sma_update(NULL, 1.0))) {
        printf("FAIL: NULL update did not return NaN\n");
        return 1;
    }
    wickra_sma_reset(NULL);
    wickra_sma_free(NULL);

    wickra_sma_free(sma);
    printf("OK: wickra C ABI smoke passed (SMA streaming + batch + reset + NULL-safety + free)\n");
    return 0;
}
