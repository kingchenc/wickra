// C++ smoke test for the Wickra C ABI via the optional RAII wrapper (`wickra.hpp`).
//
// Validates that the header compiles as C++ and that `wickra::Handle` constructs,
// moves, and frees correctly across the boundary.

#include "wickra.hpp"

#include <cmath>
#include <cstdio>
#include <utility>

int main() {
    wickra::Handle<Sma, wickra_sma_free> sma(wickra_sma_new(3));
    if (!sma) {
        std::puts("FAIL: wickra_sma_new returned null");
        return 1;
    }

    (void)wickra_sma_update(sma.get(), 1.0);
    (void)wickra_sma_update(sma.get(), 2.0);
    double value = wickra_sma_update(sma.get(), 3.0);
    if (std::fabs(value - 2.0) > 1e-9) {
        std::printf("FAIL: SMA(3) value %.6f, expected 2.0\n", value);
        return 1;
    }

    // Move transfers ownership; the moved-from handle must not double-free.
    wickra::Handle<Sma, wickra_sma_free> moved(std::move(sma));
    if (static_cast<bool>(sma) || !static_cast<bool>(moved)) {
        std::puts("FAIL: move semantics");
        return 1;
    }

    std::puts("OK: wickra C++ RAII smoke passed (Handle construct + move + auto-free)");
    return 0;
}
