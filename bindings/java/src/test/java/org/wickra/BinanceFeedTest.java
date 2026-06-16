package org.wickra;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertThrows;

/**
 * The live Binance feed's connect → read → reconnect pipeline is covered
 * deterministically by the Rust mock-WS-server tests in wickra-data. Here we
 * only assert the binding's error paths, which need no network.
 */
class BinanceFeedTest {
    @Test
    void rejectsEmptySymbols() {
        assertThrows(IllegalArgumentException.class,
            () -> new BinanceFeed("", BinanceInterval.ONE_MINUTE));
    }

    @Test
    void rejectsUnreachableEndpoint() {
        assertThrows(IllegalArgumentException.class,
            () -> new BinanceFeed("BTCUSDT", BinanceInterval.ONE_MINUTE, "ws://127.0.0.1:1"));
    }

    // The REST fetcher's parse/HTTP success path is covered by the Rust
    // mock-HTTP-server tests; here we only assert the binding's error paths.
    @Test
    void fetchKlinesRejectsZeroLimit() {
        assertThrows(IllegalArgumentException.class,
            () -> BinanceFeed.fetchKlines("BTCUSDT", BinanceInterval.ONE_HOUR, 0, -1L, -1L, null));
    }

    @Test
    void fetchKlinesSurfacesUnreachableEndpoint() {
        assertThrows(IllegalArgumentException.class,
            () -> BinanceFeed.fetchKlines("BTCUSDT", BinanceInterval.ONE_HOUR, 1, -1L, -1L, "http://127.0.0.1:1"));
    }
}
