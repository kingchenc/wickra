/* Strategy example: Bollinger-Squeeze breakout with ATR stop (Wickra C ABI).
 *
 * Enters long when the Bollinger Bandwidth has just printed a fresh 6-month low
 * (the squeeze) and price closes above the upper band (the release). Exits when
 * price closes below entry minus 2*ATR(14), or when the upper band trails back
 * below the entry price (the squeeze has played out). 0.1% fees per trade. The
 * C counterpart of `examples/rust/src/bin/strategy_bollinger_squeeze.rs`.
 *
 * Educational example. NOT a live trading recommendation. Uses the checked-in
 * `examples/data/btcusdt-1d.csv` dataset because daily bars give an
 * interpretable "6-month low" lookback (~180 bars).
 *
 * Build (after `cargo build -p wickra-c --release`):
 *   cc examples/c/strategy_bollinger_squeeze.c -I bindings/c/include -L target/release -lwickra -lm -o strat_bb
 */

#define WICKRA_CSV_IMPL
#define WICKRA_STRATEGY_IMPL
#include "wickra.h"
#include "wickra_csv.h"
#include "wickra_strategy.h"
#include <math.h>
#include <stdio.h>
#include <stdlib.h>

#ifndef WICKRA_DATA_DIR
#define WICKRA_DATA_DIR "../data"
#endif

#define FEE 0.001
#define BB_PERIOD 20
#define BB_K 2.0
#define ATR_PERIOD 14
#define ATR_STOP_MULT 2.0
#define SQUEEZE_LOOKBACK 180 /* ~6 months of daily bars */

int main(int argc, char **argv) {
    const char *path = (argc > 1) ? argv[1] : WICKRA_DATA_DIR "/btcusdt-1d.csv";

    WickraCandle *candles = NULL;
    size_t n = wickra_load_csv(path, &candles);
    if (n < SQUEEZE_LOOKBACK + BB_PERIOD) {
        fprintf(stderr, "dataset has only %llu bars; need at least %d\n",
                (unsigned long long)n, SQUEEZE_LOOKBACK + BB_PERIOD);
        free(candles);
        return 1;
    }

    struct BollingerBands *bb = wickra_bollinger_bands_new(BB_PERIOD, BB_K);
    struct Atr *atr = wickra_atr_new(ATR_PERIOD);
    double *trades = (double *)malloc(n * sizeof(*trades));
    double *equity_curve = (double *)malloc(n * sizeof(*equity_curve));
    /* Circular buffer of recent bandwidth values for the squeeze lookback. */
    double bw_window[SQUEEZE_LOOKBACK];
    size_t bw_len = 0, bw_head = 0;
    if (bb == NULL || atr == NULL || trades == NULL || equity_curve == NULL) {
        fprintf(stderr, "allocation failed\n");
        return 1;
    }

    int in_position = 0;
    double entry_price = 0.0, stop_level = 0.0;
    size_t n_trades = 0;
    double equity = 1.0;

    for (size_t i = 0; i < n; ++i) {
        const WickraCandle *c = &candles[i];
        double price = c->close;
        WickraBollingerOutput b;
        int bb_ready = wickra_bollinger_bands_update(bb, price, &b);
        double a = wickra_atr_update(atr, c->open, c->high, c->low, c->close,
                                     c->volume, c->timestamp);

        equity_curve[i] = in_position ? equity * (price / entry_price) : equity;

        if (!bb_ready || !isfinite(a)) {
            continue;
        }

        double bandwidth =
            fabs(b.middle) > 1e-15 ? (b.upper - b.lower) / b.middle : NAN;
        if (isfinite(bandwidth)) {
            if (bw_len == SQUEEZE_LOOKBACK) {
                bw_window[bw_head] = bandwidth;
                bw_head = (bw_head + 1) % SQUEEZE_LOOKBACK;
            } else {
                bw_window[bw_len++] = bandwidth;
            }
        }

        if (bw_len < SQUEEZE_LOOKBACK || !isfinite(bandwidth)) {
            continue;
        }
        double min_bw = INFINITY;
        for (size_t k = 0; k < bw_len; ++k) {
            if (bw_window[k] < min_bw) {
                min_bw = bw_window[k];
            }
        }

        if (in_position) {
            int stop_hit = price < stop_level;
            int upper_collapse = b.upper < entry_price;
            if (stop_hit || upper_collapse) {
                double trade_ret = price / entry_price - 1.0;
                trades[n_trades++] = trade_ret;
                equity *= (1.0 + trade_ret) * (1.0 - FEE);
                in_position = 0;
            }
        } else {
            int is_new_low = fabs(bandwidth - min_bw) < 1e-12;
            int breakout = price > b.upper;
            if (is_new_low && breakout) {
                entry_price = price;
                stop_level = price - ATR_STOP_MULT * a;
                equity *= 1.0 - FEE;
                in_position = 1;
            }
        }
    }
    if (in_position) {
        double trade_ret = candles[n - 1].close / entry_price - 1.0;
        trades[n_trades++] = trade_ret;
        equity *= (1.0 + trade_ret) * (1.0 - FEE);
    }

    wickra_print_summary("Bollinger Squeeze Breakout (1d, BTCUSDT)", candles[0].close,
                         candles[n - 1].close, n, trades, n_trades, equity,
                         equity_curve, n);

    wickra_bollinger_bands_free(bb);
    wickra_atr_free(atr);
    free(trades);
    free(equity_curve);
    free(candles);
    return 0;
}
