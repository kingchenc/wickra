/* Streaming/batch equivalence through the C ABI.
 *
 * Every indicator exposes both `update` (one input at a time) and `batch` (a
 * whole series at once). They must agree: a consumer that switches from a live
 * feed to a backfill, or the other way round, gets the same numbers, or the
 * library is wrong about one of the two paths.
 *
 * The other bindings assert this -- Node in __tests__/indicators.test.js,
 * Python in test_streaming_vs_batch.py -- and C did not, which left the ABI
 * every other binding sits on as the one surface where the two were never
 * compared.
 *
 * Input series, tolerance and NaN handling deliberately mirror the Node suite,
 * so a disagreement between the two languages is a disagreement about the
 * library rather than about the test.
 */
#include <math.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include "wickra.h"

#define N 120

static double close_[N], high_[N], low_[N], open_[N], volume_[N];
static int64_t ts_[N];
static int failures = 0;

static void build_series(void) {
    for (int i = 0; i < N; i++) {
        close_[i] = 100.0 + sin(i * 0.2) * 10.0 + i * 0.1;
        high_[i] = close_[i] + 1.5;
        low_[i] = close_[i] - 1.5;
        open_[i] = close_[i] - 0.5;
        volume_[i] = 1000.0 + (i % 7) * 50.0;
        ts_[i] = (int64_t)i * 60000;
    }
}

/* NaN equals NaN here: an indicator that has not warmed up reports NaN from
 * both paths, and that agreement is part of what is being checked. */
static int same(double a, double b) {
    if (isnan(a) || isnan(b)) return isnan(a) && isnan(b);
    if (isinf(a) || isinf(b)) return a == b;
    return fabs(a - b) < 1e-9;
}

static void report(const char *name, int i, double streamed, double batched) {
    fprintf(stderr, "%s: streaming and batch disagree at %d: %.17g vs %.17g\n",
            name, i, streamed, batched);
    failures++;
}

static void pass(const char *name) {
    printf("  %-8s streaming == batch over %d inputs\n", name, N);
}

/* One scalar indicator: `batch` over the close series against `update` per
 * element on a fresh handle. */
#define CHECK_SCALAR(NAME, CTOR)                                              \
    do {                                                                      \
        double batched[N];                                                    \
        void *b = (CTOR);                                                     \
        void *s = (CTOR);                                                     \
        int ok = 1;                                                           \
        if (b == NULL || s == NULL) {                                         \
            fprintf(stderr, "%s: constructor returned NULL\n", #NAME);        \
            failures++;                                                       \
            break;                                                            \
        }                                                                     \
        wickra_##NAME##_batch(b, close_, batched, N);                         \
        for (int i = 0; i < N; i++) {                                         \
            double streamed = wickra_##NAME##_update(s, close_[i]);           \
            if (!same(streamed, batched[i])) {                                \
                report(#NAME, i, streamed, batched[i]);                       \
                ok = 0;                                                       \
                break;                                                        \
            }                                                                 \
        }                                                                     \
        wickra_##NAME##_free(b);                                              \
        wickra_##NAME##_free(s);                                              \
        if (ok) pass(#NAME);                                                  \
    } while (0)

int main(void) {
    build_series();
    printf("streaming/batch equivalence through the C ABI:\n");

    CHECK_SCALAR(sma, wickra_sma_new(14));
    CHECK_SCALAR(ema, wickra_ema_new(14));
    CHECK_SCALAR(rsi, wickra_rsi_new(14));

    /* A candle indicator takes six parallel arrays rather than one. That is a
     * different marshalling path on both sides, so it gets its own check
     * instead of being assumed covered by the scalar case. */
    {
        double batched[N];
        struct Atr *b = wickra_atr_new(14);
        struct Atr *s = wickra_atr_new(14);
        int ok = 1;
        if (b == NULL || s == NULL) {
            fprintf(stderr, "atr: constructor returned NULL\n");
            failures++;
        } else {
            wickra_atr_batch(b, open_, high_, low_, close_, volume_, ts_, batched, N);
            for (int i = 0; i < N; i++) {
                double streamed = wickra_atr_update(s, open_[i], high_[i], low_[i],
                                                    close_[i], volume_[i], ts_[i]);
                if (!same(streamed, batched[i])) {
                    report("atr", i, streamed, batched[i]);
                    ok = 0;
                    break;
                }
            }
            wickra_atr_free(b);
            wickra_atr_free(s);
            if (ok) pass("atr");
        }
    }

    if (failures > 0) {
        fprintf(stderr, "\n%d indicator(s) disagree between the two paths\n", failures);
        return 1;
    }
    printf("\nboth paths agree for every indicator checked.\n");
    return 0;
}
