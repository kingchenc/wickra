/*
 * Throughput benchmark for the Wickra C ABI.
 *
 * Measures how many indicator updates per second the C ABI sustains, both
 * per-tick (streaming `_update`) and bulk (`_batch`), over a synthetic OHLCV
 * series. It is the C counterpart of the Node throughput.js and the Rust
 * criterion benches: it benchmarks Wickra's own O(1) streaming engine through
 * the raw C boundary (there is no comparable streaming TA library to compare
 * against), so the headline number is raw throughput, not a cross-library
 * ratio. C is the thinnest binding, so these numbers are the floor of the
 * per-binding FFI overhead the higher-level bindings build on.
 *
 * Three indicators are timed, chosen by call-signature archetype rather than
 * algorithm: SMA (1-in -> 1-out), ATR (multi-in -> 1-out), and MACD
 * (1-in -> multi-out). Streaming is timed for all three; batch only for the
 * single-output SMA and ATR (the C ABI has no MACD batch entry point).
 *
 * Build the C ABI library first, then build and run the benchmark:
 *
 *   cargo build -p wickra-c --release
 *   cmake -S bindings/c/benchmarks -B build/cbench -DCMAKE_BUILD_TYPE=Release
 *   cmake --build build/cbench
 *   ./build/cbench/throughput            # 200k bars (default)
 *   ./build/cbench/throughput 1000000
 */
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <math.h>
#include <stdint.h>

#include "wickra.h"

#ifdef _WIN32
#include <windows.h>
static double now_ns(void) {
    static LARGE_INTEGER freq;
    static int init = 0;
    LARGE_INTEGER counter;
    if (!init) {
        QueryPerformanceFrequency(&freq);
        init = 1;
    }
    QueryPerformanceCounter(&counter);
    return (double)counter.QuadPart * 1e9 / (double)freq.QuadPart;
}
#else
#include <time.h>
static double now_ns(void) {
    struct timespec ts;
    clock_gettime(CLOCK_MONOTONIC, &ts);
    return (double)ts.tv_sec * 1e9 + (double)ts.tv_nsec;
}
#endif

static double median3(double a, double b, double c) {
    if ((a <= b && b <= c) || (c <= b && b <= a)) return b;
    if ((b <= a && a <= c) || (c <= a && a <= b)) return a;
    return c;
}

/* Run `body` once as warmup, then time three repetitions and store the median
 * elapsed nanoseconds in `dst`. `body` is a brace-enclosed statement block. */
#define MEASURE(dst, body)                                  \
    do {                                                    \
        body;                                               \
        double s0, s1, s2, t0;                              \
        t0 = now_ns(); body; s0 = now_ns() - t0;            \
        t0 = now_ns(); body; s1 = now_ns() - t0;            \
        t0 = now_ns(); body; s2 = now_ns() - t0;            \
        (dst) = median3(s0, s1, s2);                        \
    } while (0)

int main(int argc, char **argv) {
    size_t bars = 200000;
    if (argc > 1) {
        long n = strtol(argv[1], NULL, 10);
        if (n >= 1000) {
            bars = (size_t)n;
        }
    }
    const size_t n = bars;

    /* Deterministic synthetic OHLCV (no RNG, so runs are comparable). */
    double *open = malloc(n * sizeof(double));
    double *high = malloc(n * sizeof(double));
    double *low = malloc(n * sizeof(double));
    double *close = malloc(n * sizeof(double));
    double *volume = malloc(n * sizeof(double));
    int64_t *timestamp = malloc(n * sizeof(int64_t));
    double *out = malloc(n * sizeof(double)); /* reused batch scratch buffer */
    if (!open || !high || !low || !close || !volume || !timestamp || !out) {
        fprintf(stderr, "allocation failed\n");
        return 1;
    }
    for (size_t i = 0; i < n; i++) {
        double mid = 100 + sin((double)i * 0.001) * 20 + (double)i * 1e-4;
        double c = mid + sin((double)i * 0.05) * 2;
        close[i] = c;
        open[i] = mid;
        high[i] = fmax(c, mid) + 1.5;
        low[i] = fmin(c, mid) - 1.5;
        volume[i] = 1000 + (double)(i % 97) * 13;
        timestamp[i] = (int64_t)i;
    }

    double ns;
    double sma_stream, sma_batch, atr_stream, atr_batch, macd_stream;

    MEASURE(ns, {
        struct Sma *ind = wickra_sma_new(20);
        for (size_t i = 0; i < n; i++) wickra_sma_update(ind, close[i]);
        wickra_sma_free(ind);
    });
    sma_stream = (double)n / (ns / 1e9) / 1e6;

    MEASURE(ns, {
        struct Sma *ind = wickra_sma_new(20);
        wickra_sma_batch(ind, close, out, n);
        wickra_sma_free(ind);
    });
    sma_batch = (double)n / (ns / 1e9) / 1e6;

    MEASURE(ns, {
        struct Atr *ind = wickra_atr_new(14);
        for (size_t i = 0; i < n; i++)
            wickra_atr_update(ind, open[i], high[i], low[i], close[i], volume[i], timestamp[i]);
        wickra_atr_free(ind);
    });
    atr_stream = (double)n / (ns / 1e9) / 1e6;

    MEASURE(ns, {
        struct Atr *ind = wickra_atr_new(14);
        wickra_atr_batch(ind, open, high, low, close, volume, timestamp, out, n);
        wickra_atr_free(ind);
    });
    atr_batch = (double)n / (ns / 1e9) / 1e6;

    MEASURE(ns, {
        struct MacdIndicator *ind = wickra_macd_indicator_new(12, 26, 9);
        struct WickraMacdOutput value;
        for (size_t i = 0; i < n; i++) wickra_macd_indicator_update(ind, close[i], &value);
        wickra_macd_indicator_free(ind);
    });
    macd_stream = (double)n / (ns / 1e9) / 1e6;

    printf("Wickra C throughput - %zu bars (median of 3 runs)\n\n", n);
    printf("%-22s%20s%18s\n", "Indicator", "streaming (Mupd/s)", "batch (Mupd/s)");
    printf("------------------------------------------------------------\n");
    printf("%-22s%20.1f%18.1f\n", "SMA(20)", sma_stream, sma_batch);
    printf("%-22s%20.1f%18.1f\n", "ATR(14)", atr_stream, atr_batch);
    printf("%-22s%20.1f%18s\n", "MACD(12,26,9)", macd_stream, "-");

    printf("\nMupd/s = million indicator updates per second. Streaming is the per-tick\n"
           "`_update` path (one C call per value); batch is the bulk array path (one\n"
           "C call). Higher is better. Numbers are machine-dependent - use them for\n"
           "relative comparison, not as a speed claim.\n");

    free(open);
    free(high);
    free(low);
    free(close);
    free(volume);
    free(timestamp);
    free(out);
    return 0;
}
