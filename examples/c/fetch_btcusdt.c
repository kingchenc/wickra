/* Download real BTCUSDT spot candles from the Binance REST API and write them as
 * CSV datasets under examples/data/ — the C counterpart of
 * `examples/rust/src/bin/fetch_btcusdt.rs` and `examples/node/fetch_btcusdt.js`.
 *
 * Like the Rust example, HTTPS is handled by shelling out to the system `curl`
 * (shipped with Windows 10+, macOS and every Linux distro), so this example adds
 * no HTTP/TLS dependency. Each klines response is capped at 1000 rows, so larger
 * datasets are paginated backwards through `endTime`. Only fully closed candles
 * are kept.
 *
 * This example talks to the network, so it is built but NOT run as a ctest.
 *
 * Build (after `cargo build -p wickra-c --release`):
 *   cc examples/c/fetch_btcusdt.c -I bindings/c/include -L target/release -lwickra -lm -o fetch_btcusdt
 *   ./fetch_btcusdt
 */

#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>

#ifdef _WIN32
#define POPEN _popen
#define PCLOSE _pclose
#else
#define POPEN popen
#define PCLOSE pclose
#endif

#ifndef WICKRA_DATA_DIR
#define WICKRA_DATA_DIR "../data"
#endif

#define SYMBOL "BTCUSDT"
#define PAGE_LIMIT 1000

typedef struct {
    int64_t open_time;
    int64_t close_time;
    double open, high, low, close, volume;
} Kline;

typedef struct {
    const char *interval;
    const char *file;
    size_t target;
} Dataset;

/* One dataset per timeframe. The monthly file is `btcusdt-1month.csv`, not
 * `-1M`, so it does not collide with `-1m` on case-insensitive filesystems. */
static const Dataset DATASETS[] = {
    {"1m", "btcusdt-1m.csv", 50000},  {"5m", "btcusdt-5m.csv", 10000},
    {"15m", "btcusdt-15m.csv", 10000}, {"1h", "btcusdt-1h.csv", 10000},
    {"12h", "btcusdt-12h.csv", 5000}, {"1d", "btcusdt-1d.csv", 5000},
    {"1M", "btcusdt-1month.csv", 5000},
};

/* Run `curl <url>` and return its stdout as a malloc'd, NUL-terminated buffer
 * (caller frees), or NULL on failure. */
static char *curl_get(const char *url) {
    char cmd[512];
    snprintf(cmd, sizeof(cmd),
             "curl --silent --show-error --fail --max-time 30 \"%s\"", url);
    FILE *p = POPEN(cmd, "r");
    if (p == NULL) {
        fprintf(stderr, "could not run curl (install it / put it on PATH)\n");
        return NULL;
    }
    size_t cap = 1 << 16, len = 0;
    char *buf = (char *)malloc(cap);
    if (buf == NULL) {
        PCLOSE(p);
        return NULL;
    }
    size_t got;
    char tmp[8192];
    while ((got = fread(tmp, 1, sizeof(tmp), p)) > 0) {
        if (len + got + 1 > cap) {
            cap *= 2;
            char *grown = (char *)realloc(buf, cap);
            if (grown == NULL) {
                free(buf);
                PCLOSE(p);
                return NULL;
            }
            buf = grown;
        }
        memcpy(buf + len, tmp, got);
        len += got;
    }
    int rc = PCLOSE(p);
    buf[len] = '\0';
    if (rc != 0 || len == 0) {
        fprintf(stderr, "curl failed for %s\n", url);
        free(buf);
        return NULL;
    }
    return buf;
}

/* Parse a Binance klines JSON array. Each row is
 * [openTime, "open", "high", "low", "close", "volume", closeTime, ...].
 * Appends parsed rows to *rows (grown via realloc) and returns the new count. */
static size_t parse_klines(const char *body, Kline **rows, size_t count, size_t *cap) {
    int depth = 0;
    int in_string = 0;
    int field = -1; /* -1 = not inside a row yet */
    char tok[64];
    size_t tok_len = 0;
    Kline cur = {0};

    for (const char *s = body; *s; ++s) {
        char ch = *s;
        if (in_string) {
            if (ch == '"') {
                in_string = 0;
            } else if (tok_len + 1 < sizeof(tok)) {
                tok[tok_len++] = ch;
            }
            continue;
        }
        switch (ch) {
        case '[':
            depth++;
            if (depth == 2) {
                field = 0;
                tok_len = 0;
                memset(&cur, 0, sizeof(cur));
            }
            break;
        case ']':
            if (depth == 2) {
                /* close the final field of this row, then emit it */
                tok[tok_len] = '\0';
                if (field == 6) {
                    cur.close_time = strtoll(tok, NULL, 10);
                }
                if (*cap == count) {
                    *cap = *cap ? *cap * 2 : 1024;
                    Kline *grown = (Kline *)realloc(*rows, *cap * sizeof(Kline));
                    if (grown == NULL) {
                        return count;
                    }
                    *rows = grown;
                }
                (*rows)[count++] = cur;
                field = -1;
            }
            depth--;
            break;
        case '"':
            in_string = 1;
            break;
        case ',':
            if (depth == 2) {
                tok[tok_len] = '\0';
                switch (field) {
                case 0: cur.open_time = strtoll(tok, NULL, 10); break;
                case 1: cur.open = strtod(tok, NULL); break;
                case 2: cur.high = strtod(tok, NULL); break;
                case 3: cur.low = strtod(tok, NULL); break;
                case 4: cur.close = strtod(tok, NULL); break;
                case 5: cur.volume = strtod(tok, NULL); break;
                case 6: cur.close_time = strtoll(tok, NULL, 10); break;
                default: break;
                }
                field++;
                tok_len = 0;
            }
            break;
        default:
            if (depth == 2 && tok_len + 1 < sizeof(tok)) {
                tok[tok_len++] = ch;
            }
            break;
        }
    }
    return count;
}

static int cmp_open(const void *a, const void *b) {
    int64_t x = ((const Kline *)a)->open_time;
    int64_t y = ((const Kline *)b)->open_time;
    return (x > y) - (x < y);
}

/* Paginate backwards until `target` closed candles are collected. */
static size_t collect(const char *interval, size_t target, int64_t now_ms,
                      Kline **out) {
    Kline *rows = NULL;
    size_t count = 0, cap = 0;
    int64_t end_time = 0; /* 0 = no endTime cap (most recent page) */
    int pages = 0;

    for (;;) {
        char url[256];
        if (end_time > 0) {
            snprintf(url, sizeof(url),
                     "https://api.binance.com/api/v3/klines?symbol=%s&interval=%s"
                     "&limit=%d&endTime=%lld",
                     SYMBOL, interval, PAGE_LIMIT, (long long)end_time);
        } else {
            snprintf(url, sizeof(url),
                     "https://api.binance.com/api/v3/klines?symbol=%s&interval=%s"
                     "&limit=%d",
                     SYMBOL, interval, PAGE_LIMIT);
        }
        char *body = curl_get(url);
        if (body == NULL) {
            free(rows);
            return 0;
        }
        Kline *page = NULL;
        size_t page_cap = 0;
        size_t page_n = parse_klines(body, &page, 0, &page_cap);
        free(body);
        pages++;

        int64_t oldest_open = INT64_MAX;
        for (size_t i = 0; i < page_n; ++i) {
            if (page[i].open_time < oldest_open) {
                oldest_open = page[i].open_time;
            }
            if (page[i].close_time < now_ms) { /* keep only closed candles */
                if (count == cap) {
                    cap = cap ? cap * 2 : 1024;
                    Kline *grown = (Kline *)realloc(rows, cap * sizeof(Kline));
                    if (grown == NULL) {
                        free(page);
                        free(rows);
                        return 0;
                    }
                    rows = grown;
                }
                rows[count++] = page[i];
            }
        }
        fprintf(stderr, "\r  %s: collected %llu candles over %d page(s)...",
                interval, (unsigned long long)count, pages);

        int page_full = page_n >= PAGE_LIMIT;
        free(page);
        if (count >= target || !page_full || oldest_open == INT64_MAX) {
            break;
        }
        end_time = oldest_open - 1;
    }
    fprintf(stderr, "\n");

    /* Sort ascending, drop duplicate open times, keep the most recent target. */
    qsort(rows, count, sizeof(Kline), cmp_open);
    size_t uniq = 0;
    for (size_t i = 0; i < count; ++i) {
        if (uniq == 0 || rows[i].open_time != rows[uniq - 1].open_time) {
            rows[uniq++] = rows[i];
        }
    }
    if (uniq > target) {
        memmove(rows, rows + (uniq - target), target * sizeof(Kline));
        uniq = target;
    }
    *out = rows;
    return uniq;
}

static int write_csv(const char *path, const Kline *rows, size_t n) {
    FILE *f = fopen(path, "w");
    if (f == NULL) {
        return 0;
    }
    fputs("timestamp,open,high,low,close,volume\n", f);
    for (size_t i = 0; i < n; ++i) {
        fprintf(f, "%lld,%g,%g,%g,%g,%g\n", (long long)rows[i].open_time,
                rows[i].open, rows[i].high, rows[i].low, rows[i].close,
                rows[i].volume);
    }
    fclose(f);
    return 1;
}

int main(void) {
    int64_t now_ms = (int64_t)time(NULL) * 1000;

    printf("Fetching %s klines from Binance into %s\n", SYMBOL, WICKRA_DATA_DIR);
    size_t n_datasets = sizeof(DATASETS) / sizeof(DATASETS[0]);
    for (size_t d = 0; d < n_datasets; ++d) {
        const Dataset *ds = &DATASETS[d];
        Kline *rows = NULL;
        size_t n = collect(ds->interval, ds->target, now_ms, &rows);
        if (n == 0) {
            fprintf(stderr, "Binance returned no closed candles for %s\n",
                    ds->interval);
            free(rows);
            return 1;
        }
        char path[256];
        snprintf(path, sizeof(path), "%s/%s", WICKRA_DATA_DIR, ds->file);
        if (!write_csv(path, rows, n)) {
            fprintf(stderr, "could not write %s\n", path);
            free(rows);
            return 1;
        }
        printf("  %3s  %6llu candles  ->  %s\n", ds->interval, (unsigned long long)n,
               path);
        free(rows);
    }
    printf("Done — %llu datasets written.\n", (unsigned long long)n_datasets);
    return 0;
}
