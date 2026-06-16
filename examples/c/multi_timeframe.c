/* Multi-timeframe indicators with the Wickra C ABI.
 *
 * The C counterpart of `examples/rust/src/bin/multi_timeframe.rs` and
 * `examples/python/multi_timeframe.py`: read the bundled 1-minute BTCUSDT CSV
 * (or a path on the command line), resample it to 5m / 15m / 1h / 4h / 1d, and
 * print the last RSI(14), MACD(12,26,9) histogram and ADX(14) at each timeframe.
 *
 * The Rust/Python stack resamples through `wickra-data`; the C ABI ships only
 * the indicators, so the time-bucket aggregation is done here (open = first,
 * high = max, low = min, close = last, volume = sum per bucket).
 *
 * Build (after `cargo build -p wickra-c --release`):
 *   cc examples/c/multi_timeframe.c -I bindings/c/include -L target/release -lwickra -lm -o multi_timeframe
 *   ./multi_timeframe [path/to/1m.csv]
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

#define ONE_MINUTE_MS 60000LL

/* Aggregate `in` into fixed-width time buckets of `tf_ms`. Writes the bucketed
 * candles into the caller-owned `out` (capacity >= count) and returns how many
 * buckets were produced. */
static size_t resample(const WickraBar *in, size_t count, int64_t tf_ms,
                       WickraBar *out) {
    /* Native Resampler — no hand-written bucketing. update() emits a closed
     * candle on a bucket boundary; flush() yields the final partial bucket. */
    struct Resampler *r = wickra_resampler_new(tf_ms);
    if (r == NULL) {
        return 0;
    }
    size_t produced = 0;
    struct WickraCandle c;
    for (size_t i = 0; i < count; ++i) {
        if (wickra_resampler_update(r, in[i].open, in[i].high, in[i].low,
                                    in[i].close, in[i].volume, in[i].timestamp,
                                    &c)) {
            out[produced].timestamp = c.timestamp;
            out[produced].open = c.open;
            out[produced].high = c.high;
            out[produced].low = c.low;
            out[produced].close = c.close;
            out[produced].volume = c.volume;
            produced++;
        }
    }
    if (wickra_resampler_flush(r, &c)) {
        out[produced].timestamp = c.timestamp;
        out[produced].open = c.open;
        out[produced].high = c.high;
        out[produced].low = c.low;
        out[produced].close = c.close;
        out[produced].volume = c.volume;
        produced++;
    }
    wickra_resampler_free(r);
    return produced;
}

static void summarize(const char *label, const WickraBar *candles, size_t n) {
    if (n == 0) {
        printf("  %-5s (empty)\n", label);
        return;
    }
    struct Rsi *rsi = wickra_rsi_new(14);
    struct MacdIndicator *macd = wickra_macd_indicator_new(12, 26, 9);
    struct Adx *adx = wickra_adx_new(14);

    double last_rsi = NAN;
    double last_hist = NAN;
    double last_adx = NAN;
    for (size_t i = 0; i < n; ++i) {
        const WickraBar *c = &candles[i];
        double r = wickra_rsi_update(rsi, c->close);
        if (isfinite(r)) {
            last_rsi = r;
        }
        WickraMacdOutput m;
        if (wickra_macd_indicator_update(macd, c->close, &m)) {
            last_hist = m.histogram;
        }
        WickraAdxOutput a;
        if (wickra_adx_update(adx, c->open, c->high, c->low, c->close, c->volume,
                              c->timestamp, &a)) {
            last_adx = a.adx;
        }
    }

    char b_rsi[16], b_hist[16], b_adx[16];
    if (isfinite(last_rsi)) {
        snprintf(b_rsi, sizeof(b_rsi), "%6.2f", last_rsi);
    } else {
        snprintf(b_rsi, sizeof(b_rsi), "    --");
    }
    if (isfinite(last_hist)) {
        snprintf(b_hist, sizeof(b_hist), "%+6.2f", last_hist);
    } else {
        snprintf(b_hist, sizeof(b_hist), " --   ");
    }
    if (isfinite(last_adx)) {
        snprintf(b_adx, sizeof(b_adx), "%6.2f", last_adx);
    } else {
        snprintf(b_adx, sizeof(b_adx), "    --");
    }
    printf("  %-5s bars=%5llu  last_close=%10.2f  rsi=%s  macd_hist=%s  adx=%s\n",
           label, (unsigned long long)n, candles[n - 1].close, b_rsi, b_hist, b_adx);

    wickra_rsi_free(rsi);
    wickra_macd_indicator_free(macd);
    wickra_adx_free(adx);
}

int main(int argc, char **argv) {
    const char *path = (argc > 1) ? argv[1] : WICKRA_DATA_DIR "/btcusdt-1m.csv";

    WickraBar *ones = NULL;
    size_t n = wickra_load_csv(path, &ones);
    if (n == 0) {
        fprintf(stderr, "multi_timeframe: no candles read from %s\n", path);
        return 1;
    }
    /* Resampling only ever produces fewer candles than the 1m source. */
    WickraBar *buf = (WickraBar *)malloc(n * sizeof(*buf));
    if (buf == NULL) {
        fprintf(stderr, "multi_timeframe: allocation failed\n");
        free(ones);
        return 1;
    }

    printf("Multi-timeframe view of %s\n", path);
    summarize("1m", ones, n);

    const struct {
        const char *label;
        int64_t minutes;
    } frames[] = {{"5m", 5}, {"15m", 15}, {"1h", 60}, {"4h", 240}, {"1d", 1440}};
    for (size_t f = 0; f < sizeof(frames) / sizeof(frames[0]); ++f) {
        size_t m = resample(ones, n, frames[f].minutes * ONE_MINUTE_MS, buf);
        summarize(frames[f].label, buf, m);
    }

    free(buf);
    free(ones);
    return 0;
}
