/* Strategy example: RSI mean-reversion on hourly BTCUSDT data (Wickra C ABI).
 *
 * Goes long when RSI(14) crosses below 30 (oversold), exits when RSI crosses
 * above 70 (overbought). Position is binary (full-in / full-out), fees are 0.1%
 * per trade (Binance maker tier), no stop-loss. The C counterpart of
 * `examples/rust/src/bin/strategy_rsi_mean_reversion.rs`.
 *
 * Educational example. NOT a recommended trading strategy — the point is to
 * show how a Wickra streaming indicator wires into a signal -> fill -> PnL ->
 * equity loop. Uses the checked-in `examples/data/btcusdt-1h.csv` dataset.
 *
 * Build (after `cargo build -p wickra-c --release`):
 *   cc examples/c/strategy_rsi_mean_reversion.c -I bindings/c/include -L target/release -lwickra -lm -o strat_rsi
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
#define RSI_PERIOD 14
#define OVERSOLD 30.0
#define OVERBOUGHT 70.0

int main(int argc, char **argv) {
    const char *path = (argc > 1) ? argv[1] : WICKRA_DATA_DIR "/btcusdt-1h.csv";

    WickraCandle *candles = NULL;
    size_t n = wickra_load_csv(path, &candles);
    if (n < RSI_PERIOD * 4) {
        fprintf(stderr, "dataset too small: %llu\n", (unsigned long long)n);
        free(candles);
        return 1;
    }

    struct Rsi *rsi = wickra_rsi_new(RSI_PERIOD);
    double *trades = (double *)malloc(n * sizeof(*trades));
    double *equity_curve = (double *)malloc(n * sizeof(*equity_curve));
    if (rsi == NULL || trades == NULL || equity_curve == NULL) {
        fprintf(stderr, "allocation failed\n");
        return 1;
    }

    int in_position = 0;
    double entry_price = 0.0;
    size_t n_trades = 0;
    double equity = 1.0;

    for (size_t i = 0; i < n; ++i) {
        double price = candles[i].close;
        double r = wickra_rsi_update(rsi, price);

        /* Mark-to-market so the equity curve moves bar-by-bar between trades. */
        equity_curve[i] = in_position ? equity * (price / entry_price) : equity;

        if (!isfinite(r)) {
            continue;
        }
        if (!in_position && r < OVERSOLD) {
            entry_price = price;
            equity *= 1.0 - FEE;
            in_position = 1;
        } else if (in_position && r > OVERBOUGHT) {
            double trade_ret = price / entry_price - 1.0;
            trades[n_trades++] = trade_ret;
            equity *= (1.0 + trade_ret) * (1.0 - FEE);
            in_position = 0;
        }
    }
    /* Close any still-open trade at the last bar so metrics include it. */
    if (in_position) {
        double trade_ret = candles[n - 1].close / entry_price - 1.0;
        trades[n_trades++] = trade_ret;
        equity *= (1.0 + trade_ret) * (1.0 - FEE);
    }

    wickra_print_summary("RSI Mean-Reversion (1h, BTCUSDT)", candles[0].close,
                         candles[n - 1].close, n, trades, n_trades, equity,
                         equity_curve, n);

    wickra_rsi_free(rsi);
    free(trades);
    free(equity_curve);
    free(candles);
    return 0;
}
