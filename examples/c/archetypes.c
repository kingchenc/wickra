/*
 * Archetype coverage test for the Wickra C ABI.
 *
 * Exercises one indicator per FFI call-signature archetype through the real C
 * boundary — scalar (+ batch == streaming), multi-output, alt-chart bars,
 * market profile and array input — plus reset, invalid-parameter and NULL-safety
 * behaviour. It is the C counterpart of the Go / R / Java archetype suites; the
 * existing smoke test covers symbol/header/link, this one covers the data
 * contracts. Run as a ctest (exit 0 on success).
 */
#include <math.h>
#include <stddef.h>
#include <stdint.h>
#include <stdio.h>

#include "wickra.h"

static int failures = 0;

#define CHECK(cond, msg)                          \
    do {                                          \
        if (!(cond)) {                            \
            fprintf(stderr, "FAIL: %s\n", (msg)); \
            failures += 1;                        \
        }                                         \
    } while (0)

static int is_nan(double value) {
    return value != value;
}

int main(void) {
    /* 1. Scalar archetype: SMA known value over the FFI boundary. */
    {
        struct Sma *sma = wickra_sma_new(3);
        CHECK(sma != NULL, "sma_new(3) returned NULL");
        double last = NAN;
        const double xs[] = {1.0, 2.0, 3.0, 4.0, 5.0};
        for (size_t i = 0; i < 5; i++) {
            last = wickra_sma_update(sma, xs[i]);
        }
        CHECK(fabs(last - 4.0) < 1e-9, "SMA(3) over [.. 3 4 5] should be 4");
        wickra_sma_free(sma);
    }

    /* 2. Scalar batch must equal streaming. */
    {
        const double xs[] = {1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0};
        const size_t n = 8;
        struct Sma *stream = wickra_sma_new(3);
        double want[8];
        for (size_t i = 0; i < n; i++) {
            want[i] = wickra_sma_update(stream, xs[i]);
        }
        wickra_sma_free(stream);

        struct Sma *batched = wickra_sma_new(3);
        double got[8];
        wickra_sma_batch(batched, xs, got, n);
        wickra_sma_free(batched);

        int equal = 1;
        for (size_t i = 0; i < n; i++) {
            if (is_nan(want[i]) != is_nan(got[i])) {
                equal = 0;
            } else if (!is_nan(want[i]) && fabs(want[i] - got[i]) > 1e-9) {
                equal = 0;
            }
        }
        CHECK(equal, "SMA batch must equal streaming");
    }

    /* 3. Multi-output archetype: MACD writes a struct and returns a bool. */
    {
        struct MacdIndicator *macd = wickra_macd_indicator_new(3, 6, 3);
        CHECK(macd != NULL, "macd_indicator_new returned NULL");
        struct WickraMacdOutput out = {0};
        int produced = 0;
        for (int i = 0; i < 30; i++) {
            if (wickra_macd_indicator_update(macd, 100.0 + (double)i, &out)) {
                produced = 1;
            }
        }
        CHECK(produced, "MACD produced no value after warmup");
        CHECK(!is_nan(out.macd), "MACD value is NaN after warmup");
        wickra_macd_indicator_free(macd);
    }

    /* 4. Alt-chart bars archetype: RangeBars writes 0..n bars per candle. */
    {
        struct RangeBars *bars = wickra_range_bars_new(2.0);
        CHECK(bars != NULL, "range_bars_new returned NULL");
        struct WickraRangeBar out[16];
        size_t total = 0;
        for (int i = 0; i < 15; i++) {
            double price = 100.0 + (double)i;
            total += wickra_range_bars_update(bars, price, price, price, price, 1.0,
                                              (int64_t)i, out, 16);
        }
        CHECK(total > 0, "range bars produced no bars over a 15-point move");
        wickra_range_bars_free(bars);
    }

    /* 5. Market-profile archetype: scalars + a caller-owned values buffer. */
    {
        struct VolumeProfile *profile = wickra_volume_profile_new(10, 24);
        CHECK(profile != NULL, "volume_profile_new returned NULL");
        struct WickraVolumeProfileOutputScalars scalars = {0};
        double values[24];
        intptr_t len = -1;
        for (int i = 0; i < 20; i++) {
            double price = 100.0 + (double)i;
            len = wickra_volume_profile_update(profile, price, price + 1.0, price - 1.0,
                                               price, 1000.0, (int64_t)i, &scalars,
                                               values, 24);
        }
        CHECK(len > 0, "volume profile never produced a snapshot");
        wickra_volume_profile_free(profile);
    }

    /* 6. Array-input archetype: a full order-book snapshot per side. */
    {
        struct OrderBookImbalanceFull *book = wickra_order_book_imbalance_full_new();
        CHECK(book != NULL, "order_book_imbalance_full_new returned NULL");
        const double bid_price[] = {99.9, 99.8, 99.7};
        const double bid_size[] = {5.0, 3.0, 2.0};
        const double ask_price[] = {100.1, 100.2, 100.3};
        const double ask_size[] = {1.0, 1.0, 1.0};
        double imbalance = wickra_order_book_imbalance_full_update(
            book, bid_price, bid_size, 3, ask_price, ask_size, 3);
        CHECK(!is_nan(imbalance), "order book imbalance returned NaN");
        wickra_order_book_imbalance_full_free(book);
    }

    /* 7. Reset returns the indicator to its warmup state. */
    {
        struct Sma *sma = wickra_sma_new(3);
        for (int i = 0; i < 3; i++) {
            wickra_sma_update(sma, (double)(i + 1));
        }
        wickra_sma_reset(sma);
        CHECK(is_nan(wickra_sma_update(sma, 10.0)), "SMA after reset must be NaN");
        wickra_sma_free(sma);
    }

    /* 8. Invalid parameters return NULL; freeing NULL is a no-op. */
    {
        CHECK(wickra_sma_new(0) == NULL, "sma_new(0) should return NULL");
        wickra_sma_free(NULL);
    }

    /* 9. A NULL handle update is a no-op returning NaN, never a crash. */
    CHECK(is_nan(wickra_sma_update(NULL, 1.0)), "update(NULL) should return NaN");

    if (failures == 0) {
        printf("all archetypes passed\n");
        return 0;
    }
    fprintf(stderr, "%d archetype check(s) failed\n", failures);
    return 1;
}
