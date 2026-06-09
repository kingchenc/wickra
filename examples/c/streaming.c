/* Streaming usage example for the Wickra C ABI.
 *
 * The same five-function shape (new / update / batch / reset / free) drives every
 * scalar indicator. Here an EMA consumes a live tick stream one value at a time;
 * `update` is O(1) per tick and returns NaN until the indicator has warmed up.
 *
 * Build (after `cargo build -p wickra-c --release`):
 *   cc examples/c/streaming.c -I bindings/c/include -L target/release -lwickra -lm -o streaming
 */

#include "wickra.h"
#include <stdio.h>

int main(void) {
    struct Ema *ema = wickra_ema_new(5);
    if (ema == NULL) {
        fprintf(stderr, "failed to create EMA\n");
        return 1;
    }

    const double prices[] = {10.0, 10.5, 11.0, 10.8, 11.2, 11.5, 11.3, 11.8};
    const size_t n = sizeof(prices) / sizeof(prices[0]);

    printf("EMA(5) streaming:\n");
    for (size_t i = 0; i < n; ++i) {
        double value = wickra_ema_update(ema, prices[i]);
        if (value != value) { /* NaN during warmup */
            printf("  tick %zu  price %.2f  ->  (warming up)\n", i, prices[i]);
        } else {
            printf("  tick %zu  price %.2f  ->  %.4f\n", i, prices[i], value);
        }
    }

    wickra_ema_free(ema);
    return 0;
}
