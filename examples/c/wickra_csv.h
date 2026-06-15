/* Shared OHLCV CSV loader for the Wickra C examples.
 *
 * The C ABI exposes only the indicators, not the wickra-data IO layer, so the
 * examples read CSV themselves. This header-only helper is the C counterpart of
 * `wickra_data::csv::CandleReader` used by the Rust examples: it parses the
 * standard `timestamp,open,high,low,close,volume` files shipped under
 * `examples/data/`.
 *
 * Header-only: define WICKRA_CSV_IMPL in exactly one translation unit (each
 * example is a single .c file, so it just defines it before including this).
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
 * on any error (missing file, no parseable rows). A leading header line whose
 * first field is non-numeric is skipped. */
size_t wickra_load_csv(const char *path, WickraBar **out);

#ifdef WICKRA_CSV_IMPL

#include <stdio.h>
#include <stdlib.h>

size_t wickra_load_csv(const char *path, WickraBar **out) {
    *out = NULL;
    FILE *f = fopen(path, "r");
    if (f == NULL) {
        fprintf(stderr, "wickra_load_csv: cannot open %s\n", path);
        return 0;
    }

    size_t cap = 1024;
    size_t n = 0;
    WickraBar *rows = (WickraBar *)malloc(cap * sizeof(*rows));
    if (rows == NULL) {
        fclose(f);
        return 0;
    }

    char line[512];
    while (fgets(line, (int)sizeof(line), f) != NULL) {
        WickraBar c;
        long long ts = 0;
        /* sscanf returns the number of fields successfully matched. A header
         * row ("timestamp,...") matches 0 and is skipped. */
        int matched = sscanf(line, "%lld,%lf,%lf,%lf,%lf,%lf", &ts, &c.open,
                             &c.high, &c.low, &c.close, &c.volume);
        if (matched != 6) {
            continue;
        }
        c.timestamp = (int64_t)ts;

        if (n == cap) {
            cap *= 2;
            WickraBar *grown = (WickraBar *)realloc(rows, cap * sizeof(*rows));
            if (grown == NULL) {
                free(rows);
                fclose(f);
                return 0;
            }
            rows = grown;
        }
        rows[n++] = c;
    }
    fclose(f);

    if (n == 0) {
        free(rows);
        return 0;
    }
    *out = rows;
    return n;
}

#endif /* WICKRA_CSV_IMPL */
#endif /* WICKRA_CSV_H */
