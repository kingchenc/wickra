/* Live BTCUSDT indicator with the Wickra C ABI.
 *
 * The C counterpart of `examples/rust/src/bin/live_binance.rs`,
 * `examples/python/live_binance.py` and `examples/node/live_binance.js`. Those
 * stream Binance over a WebSocket; the C ABI ships only the indicators and no
 * socket layer, so this example polls the Binance REST klines endpoint via the
 * system `curl` once per interval and feeds each newly *closed* candle into a
 * streaming RSI(14). Same "live feed -> incremental indicator" shape, no extra
 * dependency.
 *
 * This example talks to the network and runs until interrupted (Ctrl+C), so it
 * is built but NOT run as a ctest.
 *
 * Build (after `cargo build -p wickra-c --release`):
 *   cc examples/c/live_binance.c -I bindings/c/include -L target/release -lwickra -lm -o live_binance
 *   ./live_binance [SYMBOL]
 */

#include "wickra.h"
#include <math.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#ifdef _WIN32
#include <windows.h>
#define SLEEP_MS(ms) Sleep(ms)
#define POPEN _popen
#define PCLOSE _pclose
#else
#include <time.h>
#define POPEN popen
#define PCLOSE pclose
static void SLEEP_MS(long ms) {
    struct timespec ts = {ms / 1000, (ms % 1000) * 1000000L};
    nanosleep(&ts, NULL);
}
#endif

/* Run `curl <url>` and return stdout as a malloc'd, NUL-terminated buffer. */
static char *curl_get(const char *url) {
    char cmd[512];
    snprintf(cmd, sizeof(cmd),
             "curl --silent --show-error --fail --max-time 15 \"%s\"", url);
    FILE *p = POPEN(cmd, "r");
    if (p == NULL) {
        return NULL;
    }
    size_t cap = 1 << 15, len = 0;
    char *buf = (char *)malloc(cap);
    if (buf == NULL) {
        PCLOSE(p);
        return NULL;
    }
    size_t got;
    char tmp[4096];
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
        free(buf);
        return NULL;
    }
    return buf;
}

/* Extract the first kline's open time and close price from a klines response of
 * the form [[openTime,"open","high","low","close",...],...]. Returns 1 on
 * success. The first row (limit=2) is the most recent fully closed candle. */
static int first_kline(const char *body, int64_t *open_time, double *close) {
    const char *s = strchr(body, '[');
    if (s == NULL) {
        return 0;
    }
    s = strchr(s + 1, '['); /* into the first row */
    if (s == NULL) {
        return 0;
    }
    s++;
    char tok[64];
    int field = 0;
    while (*s && *s != ']') {
        size_t tl = 0;
        while (*s && *s != ',' && *s != ']') {
            if (*s != '"' && tl + 1 < sizeof(tok)) {
                tok[tl++] = *s;
            }
            s++;
        }
        tok[tl] = '\0';
        if (field == 0) {
            *open_time = strtoll(tok, NULL, 10);
        } else if (field == 4) {
            *close = strtod(tok, NULL);
            return 1;
        }
        field++;
        if (*s == ',') {
            s++;
        }
    }
    return 0;
}

/* A Binance symbol is uppercase alphanumeric and nothing else.
 *
 * It matters here because the symbol ends up inside a shell command: curl_get
 * builds `curl "<url>"` and hands it to popen. The quotes around the URL do not
 * make that safe -- a symbol containing a double quote closes them and the rest
 * is run as a command. Rejecting anything outside [A-Z0-9] at the boundary is
 * both the smaller change and the honest one for an example, which is code
 * people copy. */
static int symbol_is_sane(const char *symbol) {
    if (symbol[0] == '\0' || strlen(symbol) > 20) {
        return 0;
    }
    for (const char *c = symbol; *c != '\0'; c++) {
        int upper = (*c >= 'A' && *c <= 'Z');
        int digit = (*c >= '0' && *c <= '9');
        if (!upper && !digit) {
            return 0;
        }
    }
    return 1;
}

int main(int argc, char **argv) {
    const char *symbol = (argc > 1) ? argv[1] : "BTCUSDT";
    if (!symbol_is_sane(symbol)) {
        fprintf(stderr,
                "symbol must be uppercase letters and digits, e.g. BTCUSDT\n");
        return 1;
    }
    char url[256];
    snprintf(url, sizeof(url),
             "https://api.binance.com/api/v3/klines?symbol=%s&interval=1m&limit=2",
             symbol);

    struct Rsi *rsi = wickra_rsi_new(14);
    if (rsi == NULL) {
        fprintf(stderr, "failed to create RSI\n");
        return 1;
    }
    printf("Listening for %s 1m closes (REST poll, Ctrl+C to stop)...\n", symbol);

    int64_t last_open = 0;
    for (;;) {
        char *body = curl_get(url);
        if (body != NULL) {
            int64_t open_time = 0;
            double close = 0.0;
            if (first_kline(body, &open_time, &close) && open_time != last_open) {
                last_open = open_time;
                double v = wickra_rsi_update(rsi, close);
                if (isfinite(v)) {
                    printf("%s  close=%.4f  rsi=%.2f\n", symbol, close, v);
                } else {
                    printf("%s  close=%.4f  rsi=...warmup\n", symbol, close);
                }
                fflush(stdout);
            }
            free(body);
        }
        SLEEP_MS(2000);
    }
    /* Unreachable in normal use (interrupted by Ctrl+C). */
}
