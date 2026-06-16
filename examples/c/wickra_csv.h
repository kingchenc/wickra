/* Shared OHLCV CSV loader for the Wickra C examples.
 *
 * Thin wrapper over the native C-ABI `wickra_candle_reader_*` (the same
 * `wickra-data::csv::CandleReader` the Rust examples use): it reads the whole
 * file into memory and parses the standard `timestamp,open,high,low,close,volume`
 * files shipped under `examples/data/` through the library — no hand-written CSV
 * parsing.
 *
 * Header-only: define WICKRA_CSV_IMPL in exactly one translation unit (each
 * example is a single .c file, so it just defines it before including this).
 * `wickra.h` must be included before this header.
 */
#ifndef WICKRA_CSV_H
#define WICKRA_CSV_H

#include <stddef.h>
#include <stdint.h>

typedef struct WickraBar {
    int64_t timestamp;
    double open;
    double high;
    double low;
    double close;
    double volume;
} WickraBar;

/* Load an OHLCV CSV into a malloc'd array. Returns the candle count and stores
 * the array in *out (caller frees with free()). Returns 0 and leaves *out NULL
 * on any error (missing file, malformed header or row). */
size_t wickra_load_csv(const char *path, WickraBar **out);

#ifdef WICKRA_CSV_IMPL

#include <stdio.h>
#include <stdlib.h>

#include "wickra.h"

size_t wickra_load_csv(const char *path, WickraBar **out) {
    *out = NULL;
    FILE *f = fopen(path, "rb");
    if (f == NULL) {
        fprintf(stderr, "wickra_load_csv: cannot open %s\n", path);
        return 0;
    }
    fseek(f, 0, SEEK_END);
    long size = ftell(f);
    fseek(f, 0, SEEK_SET);
    if (size <= 0) {
        fclose(f);
        return 0;
    }
    char *buf = (char *)malloc((size_t)size);
    if (buf == NULL) {
        fclose(f);
        return 0;
    }
    size_t read_bytes = fread(buf, 1, (size_t)size, f);
    fclose(f);

    /* Parse the whole buffer with the native reader. */
    struct CandleReader *reader =
        wickra_candle_reader_new((const uint8_t *)buf, (uintptr_t)read_bytes);
    free(buf);
    if (reader == NULL) {
        return 0;
    }
    size_t n = (size_t)wickra_candle_reader_count(reader);
    if (n == 0) {
        wickra_candle_reader_free(reader);
        return 0;
    }
    struct WickraCandle *candles = (struct WickraCandle *)malloc(n * sizeof(*candles));
    if (candles == NULL) {
        wickra_candle_reader_free(reader);
        return 0;
    }
    size_t got = (size_t)wickra_candle_reader_read(reader, candles, (uintptr_t)n);
    wickra_candle_reader_free(reader);

    WickraBar *rows = (WickraBar *)malloc(got * sizeof(*rows));
    if (rows == NULL) {
        free(candles);
        return 0;
    }
    for (size_t i = 0; i < got; ++i) {
        rows[i].timestamp = candles[i].timestamp;
        rows[i].open = candles[i].open;
        rows[i].high = candles[i].high;
        rows[i].low = candles[i].low;
        rows[i].close = candles[i].close;
        rows[i].volume = candles[i].volume;
    }
    free(candles);
    *out = rows;
    return got;
}

#endif /* WICKRA_CSV_IMPL */
#endif /* WICKRA_CSV_H */
