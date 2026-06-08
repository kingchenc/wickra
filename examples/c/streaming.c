/* Streaming indicators with the Wickra C ABI.
 *
 * Feeds a synthetic price series through several indicators tick by tick — the
 * same O(1)-per-update model a live trading bot would use — and prints a status
 * line once every indicator has warmed up. The C counterpart of
 * `examples/rust/src/bin/streaming.rs`, `examples/python/streaming.py` and
 * `examples/node/streaming.js`, using the same seeded LCG so a side-by-side run
 * produces visibly comparable streams.
 *
 * Build (after `cargo build -p wickra-c --release`):
 *   cc examples/c/streaming.c -I bindings/c/include -L target/release -lwickra -lm -o streaming
 */

#include "wickra.h"
#include <math.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define DEFAULT_TICKS 120

/* Deterministic synthetic series matching the sibling examples' seeded LCG. */
static void make_series(double *prices, size_t n) {
    uint64_t seed = 1234567u;
    for (size_t t = 0; t < n; ++t) {
        seed = (seed * 1103515245u + 12345u) & 0x7FFFFFFFu;
        double rnd = (double)seed / (double)0x7FFFFFFFu;
        double tf = (double)t;
        prices[t] = 100.0 + tf * 0.05 + sin(tf * 0.07) * 8.0 + cos(tf * 0.21) * 3.0 +
                    (rnd - 0.5);
    }
}

/* Format an indicator value, rendering warmup (NaN) as a dashed placeholder. */
static void fmt(char *buf, size_t buflen, double v) {
    if (isfinite(v)) {
        snprintf(buf, buflen, "%7.2f", v);
    } else {
        snprintf(buf, buflen, "  --   ");
    }
}

int main(int argc, char **argv) {
    size_t ticks = DEFAULT_TICKS;
    if (argc == 3 && strcmp(argv[1], "--ticks") == 0) {
        long parsed = strtol(argv[2], NULL, 10);
        if (parsed <= 0) {
            fprintf(stderr, "--ticks must be positive\n");
            return 1;
        }
        ticks = (size_t)parsed;
    } else if (argc != 1) {
        fprintf(stderr, "usage: streaming [--ticks N]\n");
        return 1;
    }

    struct Sma *sma = wickra_sma_new(20);
    struct Ema *ema = wickra_ema_new(20);
    struct Rsi *rsi = wickra_rsi_new(14);
    struct MacdIndicator *macd = wickra_macd_indicator_new(12, 26, 9);
    double *prices = (double *)malloc(ticks * sizeof(*prices));
    if (sma == NULL || ema == NULL || rsi == NULL || macd == NULL || prices == NULL) {
        fprintf(stderr, "allocation failed\n");
        return 1;
    }
    make_series(prices, ticks);

    printf("Wickra streaming indicator demo (C)\n\n");

    size_t signals = 0;
    for (size_t t = 0; t < ticks; ++t) {
        double price = prices[t];
        double sv = wickra_sma_update(sma, price);
        double ev = wickra_ema_update(ema, price);
        double rv = wickra_rsi_update(rsi, price);
        WickraMacdOutput m;
        int macd_ready = wickra_macd_indicator_update(macd, price, &m);

        /* Only act once every indicator has produced a value. */
        if (!isfinite(sv) || !isfinite(ev) || !isfinite(rv) || !macd_ready) {
            continue;
        }

        int overbought = rv > 70.0 && m.histogram < 0.0;
        int oversold = rv < 30.0 && m.histogram > 0.0;
        const char *tag = overbought ? "SELL?" : (oversold ? "BUY? " : "     ");
        if (overbought || oversold) {
            signals++;
        }

        char b_price[16], b_sma[16], b_ema[16], b_rsi[16], b_hist[16];
        fmt(b_price, sizeof(b_price), price);
        fmt(b_sma, sizeof(b_sma), sv);
        fmt(b_ema, sizeof(b_ema), ev);
        fmt(b_rsi, sizeof(b_rsi), rv);
        fmt(b_hist, sizeof(b_hist), m.histogram);
        printf("t=%3llu  price=%s  sma=%s  ema=%s  rsi=%s  macd_hist=%s  %s\n",
               (unsigned long long)t, b_price, b_sma, b_ema, b_rsi, b_hist, tag);
    }

    printf("\nDone — %llu candidate signal(s) over %llu ticks.\n",
           (unsigned long long)signals, (unsigned long long)ticks);

    wickra_sma_free(sma);
    wickra_ema_free(ema);
    wickra_rsi_free(rsi);
    wickra_macd_indicator_free(macd);
    free(prices);
    return 0;
}
