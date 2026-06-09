/* Parallel multi-asset indicator computation with the Wickra C ABI.
 *
 * The C counterpart of `examples/rust/src/bin/parallel_assets.rs` (rayon) and
 * `examples/python/parallel_assets.py` (the Rust extension drops the GIL). The C
 * ABI is the parallelization primitive itself: each `wickra_<ind>_new` handle is
 * independent, so the caller fans assets out across threads, one fresh handle per
 * asset. Here a serial baseline is compared against an OpenMP `parallel for`.
 *
 * If the compiler has no OpenMP support the "parallel" pass simply runs the same
 * single-threaded loop (speedup ~1x) — the result is honest either way.
 *
 * Build (after `cargo build -p wickra-c --release`):
 *   cc -fopenmp examples/c/parallel_assets.c -I bindings/c/include -L target/release -lwickra -lm -o parallel_assets
 *   ./parallel_assets --assets 200 --bars 5000 --indicator sma
 */

#include "wickra.h"
#include <math.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>

#ifdef _OPENMP
#include <omp.h>
#endif

/* Wall-clock seconds (not CPU time, so threaded speedup is measured correctly). */
static double now_seconds(void) {
#ifdef _OPENMP
    return omp_get_wtime();
#else
    struct timespec ts;
    timespec_get(&ts, TIME_UTC);
    return (double)ts.tv_sec + (double)ts.tv_nsec * 1e-9;
#endif
}

typedef enum { IND_SMA, IND_RSI } Which;

/* Deterministic synthetic (assets, bars) panel, flat row-major. Each asset uses
 * an independent LCG seed so the series are uncorrelated but reproducible. */
static void synthesize_panel(double *panel, size_t assets, size_t bars) {
    for (size_t a = 0; a < assets; ++a) {
        double price = 100.0;
        uint32_t state = (uint32_t)(1234567u + a) * 2654435761u;
        for (size_t b = 0; b < bars; ++b) {
            state = (state * 1103515245u + 12345u) & 0x7FFFFFFFu;
            double r = (double)state / (double)0x7FFFFFFFu;
            price += (r - 0.5) * 0.4;
            panel[a * bars + b] = price;
        }
    }
}

/* Run one asset's series through a fresh handle into its output slice. */
static void run_one(Which which, const double *prices, double *out, size_t bars) {
    if (which == IND_SMA) {
        struct Sma *h = wickra_sma_new(14);
        wickra_sma_batch(h, prices, out, bars);
        wickra_sma_free(h);
    } else {
        struct Rsi *h = wickra_rsi_new(14);
        wickra_rsi_batch(h, prices, out, bars);
        wickra_rsi_free(h);
    }
}

int main(int argc, char **argv) {
    size_t assets = 200;
    size_t bars = 5000;
    Which which = IND_SMA;
    for (int i = 1; i < argc; ++i) {
        if (strcmp(argv[i], "--assets") == 0 && i + 1 < argc) {
            assets = (size_t)strtoul(argv[++i], NULL, 10);
        } else if (strcmp(argv[i], "--bars") == 0 && i + 1 < argc) {
            bars = (size_t)strtoul(argv[++i], NULL, 10);
        } else if (strcmp(argv[i], "--indicator") == 0 && i + 1 < argc) {
            ++i;
            if (strcmp(argv[i], "sma") == 0) {
                which = IND_SMA;
            } else if (strcmp(argv[i], "rsi") == 0) {
                which = IND_RSI;
            } else {
                fprintf(stderr, "--indicator: expected 'sma' or 'rsi'\n");
                return 1;
            }
        } else {
            fprintf(stderr, "usage: parallel_assets [--assets N] [--bars N] "
                            "[--indicator sma|rsi]\n");
            return 1;
        }
    }
    if (assets == 0 || bars == 0) {
        fprintf(stderr, "--assets and --bars must be positive\n");
        return 1;
    }

    const char *ind_name = (which == IND_SMA) ? "sma" : "rsi";
    printf("Generating %llux%llu synthetic panel...\n", (unsigned long long)assets,
           (unsigned long long)bars);
    double *panel = (double *)malloc(assets * bars * sizeof(*panel));
    double *serial = (double *)malloc(assets * bars * sizeof(*serial));
    double *parallel = (double *)malloc(assets * bars * sizeof(*parallel));
    if (panel == NULL || serial == NULL || parallel == NULL) {
        fprintf(stderr, "allocation failed\n");
        return 1;
    }
    synthesize_panel(panel, assets, bars);

    double t0 = now_seconds();
    for (size_t a = 0; a < assets; ++a) {
        run_one(which, &panel[a * bars], &serial[a * bars], bars);
    }
    double t_serial = now_seconds() - t0;
    printf("Serial:   %8.3f s  (%llu assets, indicator=%s)\n", t_serial,
           (unsigned long long)assets, ind_name);

    /* The loop variable is declared outside the `for` and the bound is a plain
     * variable: MSVC's OpenMP 2.0 rejects an in-init declaration or a cast in
     * the condition (error C3015). */
    long a;
    long asset_count = (long)assets;
    t0 = now_seconds();
#ifdef _OPENMP
#pragma omp parallel for schedule(static)
#endif
    for (a = 0; a < asset_count; ++a) {
        run_one(which, &panel[(size_t)a * bars], &parallel[(size_t)a * bars], bars);
    }
    double t_parallel = now_seconds() - t0;
    double denom = t_parallel > 1e-9 ? t_parallel : 1e-9;
#ifdef _OPENMP
    printf("Parallel: %8.3f s  (OpenMP, %d threads, speedup ~%.2fx)\n", t_parallel,
           omp_get_max_threads(), t_serial / denom);
#else
    printf("Parallel: %8.3f s  (no OpenMP at build time — serial, speedup ~%.2fx)\n",
           t_parallel, t_serial / denom);
#endif

    /* The parallel run must reproduce the serial results exactly. */
    for (size_t i = 0; i < assets * bars; ++i) {
        int both_nan = isnan(serial[i]) && isnan(parallel[i]);
        if (!both_nan && serial[i] != parallel[i]) {
            fprintf(stderr, "mismatch at %llu: serial=%g parallel=%g\n",
                    (unsigned long long)i, serial[i], parallel[i]);
            return 1;
        }
    }
    printf("Parallel results match serial results — OK.\n");

    free(panel);
    free(serial);
    free(parallel);
    return 0;
}
