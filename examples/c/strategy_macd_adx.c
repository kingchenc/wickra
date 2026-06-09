/* Strategy example: MACD crossover with ADX trend-strength filter (Wickra C ABI).
 *
 * Long-only trend follower. Entries fire when the MACD line crosses above the
 * signal line while ADX(14) > 20 (a market with at least mild directional
 * strength); exits on the opposite MACD crossover regardless of ADX. 0.1% fees
 * per trade. The C counterpart of `examples/rust/src/bin/strategy_macd_adx.rs`.
 *
 * The ADX filter is the point: pure MACD on sideways markets chops in and out;
 * gating entries on directional strength cuts the worst losing streak.
 *
 * Educational example. NOT a live trading recommendation. Uses the checked-in
 * `examples/data/btcusdt-1h.csv` dataset.
 *
 * Build (after `cargo build -p wickra-c --release`):
 *   cc examples/c/strategy_macd_adx.c -I bindings/c/include -L target/release -lwickra -lm -o strat_macd
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
#define ADX_FLOOR 20.0

int main(int argc, char **argv) {
    const char *path = (argc > 1) ? argv[1] : WICKRA_DATA_DIR "/btcusdt-1h.csv";

    WickraCandle *candles = NULL;
    size_t n = wickra_load_csv(path, &candles);
    if (n == 0) {
        fprintf(stderr, "CSV is empty: %s\n", path);
        return 1;
    }

    struct MacdIndicator *macd = wickra_macd_indicator_new(12, 26, 9);
    struct Adx *adx = wickra_adx_new(14);
    double *trades = (double *)malloc(n * sizeof(*trades));
    double *equity_curve = (double *)malloc(n * sizeof(*equity_curve));
    if (macd == NULL || adx == NULL || trades == NULL || equity_curve == NULL) {
        fprintf(stderr, "allocation failed\n");
        return 1;
    }

    int in_position = 0;
    double entry_price = 0.0;
    size_t n_trades = 0;
    double equity = 1.0;
    /* Previous histogram sign to detect MACD-line crossovers: -1 unset, 0/1 sign. */
    int prev_hist_sign = -1;

    for (size_t i = 0; i < n; ++i) {
        const WickraCandle *c = &candles[i];
        double price = c->close;
        WickraMacdOutput m;
        int macd_ready = wickra_macd_indicator_update(macd, price, &m);
        WickraAdxOutput a;
        int adx_ready = wickra_adx_update(adx, c->open, c->high, c->low, c->close,
                                          c->volume, c->timestamp, &a);

        equity_curve[i] = in_position ? equity * (price / entry_price) : equity;

        if (!macd_ready || !adx_ready) {
            continue;
        }

        int hist_sign = m.histogram > 0.0 ? 1 : 0;
        int cross_up = prev_hist_sign == 0 && hist_sign == 1;
        int cross_down = prev_hist_sign == 1 && hist_sign == 0;
        prev_hist_sign = hist_sign;

        if (!in_position && cross_up && a.adx > ADX_FLOOR) {
            entry_price = price;
            equity *= 1.0 - FEE;
            in_position = 1;
        } else if (in_position && cross_down) {
            double trade_ret = price / entry_price - 1.0;
            trades[n_trades++] = trade_ret;
            equity *= (1.0 + trade_ret) * (1.0 - FEE);
            in_position = 0;
        }
    }
    if (in_position) {
        double trade_ret = candles[n - 1].close / entry_price - 1.0;
        trades[n_trades++] = trade_ret;
        equity *= (1.0 + trade_ret) * (1.0 - FEE);
    }

    wickra_print_summary("MACD + ADX Trend Filter (1h, BTCUSDT)", candles[0].close,
                         candles[n - 1].close, n, trades, n_trades, equity,
                         equity_curve, n);

    wickra_macd_indicator_free(macd);
    wickra_adx_free(adx);
    free(trades);
    free(equity_curve);
    free(candles);
    return 0;
}
