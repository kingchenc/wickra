/* Backtest a basket of indicators against an OHLCV CSV with the Wickra C ABI.
 *
 * The C counterpart of `examples/rust/src/bin/backtest.rs` and
 * `examples/node/backtest.js`: load an OHLCV file, run a basket of indicators
 * (scalar ones via `_batch`, candle ones streamed bar by bar), and print the
 * most recent value of each. Defaults to the bundled BTCUSDT daily dataset.
 *
 * Build (after `cargo build -p wickra-c --release`):
 *   cc examples/c/backtest.c -I bindings/c/include -L target/release -lwickra -lm -o backtest
 *   ./backtest [path/to/ohlcv.csv]
 */

#define WICKRA_CSV_IMPL
#include "wickra.h"
#include "wickra_csv.h"
#include <math.h>
#include <stdio.h>
#include <stdlib.h>

#ifndef WICKRA_DATA_DIR
#define WICKRA_DATA_DIR "../data"
#endif

/* Last finite value of a scalar batch result, or NaN if none warmed up. */
static double last_finite(const double *v, size_t n) {
    for (size_t i = n; i-- > 0;) {
        if (isfinite(v[i])) {
            return v[i];
        }
    }
    return NAN;
}

int main(int argc, char **argv) {
    const char *path = (argc > 1) ? argv[1] : WICKRA_DATA_DIR "/btcusdt-1d.csv";

    WickraCandle *candles = NULL;
    size_t n = wickra_load_csv(path, &candles);
    if (n == 0) {
        fprintf(stderr, "backtest: no candles read from %s\n", path);
        return 1;
    }

    double *closes = (double *)malloc(n * sizeof(*closes));
    double *rsi_out = (double *)malloc(n * sizeof(*rsi_out));
    double *ema_out = (double *)malloc(n * sizeof(*ema_out));
    struct Rsi *rsi = wickra_rsi_new(14);
    struct Ema *ema = wickra_ema_new(20);
    struct BollingerBands *bb = wickra_bollinger_bands_new(20, 2.0);
    struct MacdIndicator *macd = wickra_macd_indicator_new(12, 26, 9);
    struct Atr *atr = wickra_atr_new(14);
    struct Adx *adx = wickra_adx_new(14);
    struct Obv *obv = wickra_obv_new();
    if (closes == NULL || rsi_out == NULL || ema_out == NULL || rsi == NULL ||
        ema == NULL || bb == NULL || macd == NULL || atr == NULL || adx == NULL ||
        obv == NULL) {
        fprintf(stderr, "backtest: allocation failed\n");
        return 1;
    }

    for (size_t i = 0; i < n; ++i) {
        closes[i] = candles[i].close;
    }

    /* Scalar indicators: one batch call over the close series. */
    wickra_rsi_batch(rsi, closes, rsi_out, n);
    wickra_ema_batch(ema, closes, ema_out, n);

    /* Multi-output and candle indicators: streamed bar by bar, keeping the
     * last value each one produced. */
    WickraBollingerOutput last_bb = {0};
    int have_bb = 0;
    WickraMacdOutput last_macd = {0};
    int have_macd = 0;
    WickraAdxOutput last_adx = {0};
    int have_adx = 0;
    double last_atr = NAN;
    double last_obv = NAN;
    for (size_t i = 0; i < n; ++i) {
        const WickraCandle *c = &candles[i];
        WickraBollingerOutput bo;
        if (wickra_bollinger_bands_update(bb, c->close, &bo)) {
            last_bb = bo;
            have_bb = 1;
        }
        WickraMacdOutput mo;
        if (wickra_macd_indicator_update(macd, c->close, &mo)) {
            last_macd = mo;
            have_macd = 1;
        }
        double av = wickra_atr_update(atr, c->open, c->high, c->low, c->close,
                                      c->volume, c->timestamp);
        if (isfinite(av)) {
            last_atr = av;
        }
        WickraAdxOutput ao;
        if (wickra_adx_update(adx, c->open, c->high, c->low, c->close, c->volume,
                              c->timestamp, &ao)) {
            last_adx = ao;
            have_adx = 1;
        }
        double ov = wickra_obv_update(obv, c->open, c->high, c->low, c->close,
                                      c->volume, c->timestamp);
        if (isfinite(ov)) {
            last_obv = ov;
        }
    }

    printf("backtest summary for %s (%llu bars)\n", path, (unsigned long long)n);
    printf("  RSI(14)  = %9.4f\n", last_finite(rsi_out, n));
    printf("  EMA(20)  = %9.4f\n", last_finite(ema_out, n));
    if (have_bb) {
        printf("  BB(20,2) upper=%9.4f  middle=%9.4f  lower=%9.4f  sd=%8.4f\n",
               last_bb.upper, last_bb.middle, last_bb.lower, last_bb.stddev);
    }
    if (have_macd) {
        printf("  MACD     macd=%9.4f  signal=%9.4f  hist=%9.4f\n", last_macd.macd,
               last_macd.signal, last_macd.histogram);
    }
    printf("  ATR(14)  = %9.4f\n", last_atr);
    if (have_adx) {
        printf("  ADX(14)  +DI=%6.2f  -DI=%6.2f  ADX=%6.2f\n", last_adx.plus_di,
               last_adx.minus_di, last_adx.adx);
    }
    printf("  OBV      = %14.2f\n", last_obv);

    wickra_rsi_free(rsi);
    wickra_ema_free(ema);
    wickra_bollinger_bands_free(bb);
    wickra_macd_indicator_free(macd);
    wickra_atr_free(atr);
    wickra_adx_free(adx);
    wickra_obv_free(obv);
    free(closes);
    free(rsi_out);
    free(ema_out);
    free(candles);
    return 0;
}
