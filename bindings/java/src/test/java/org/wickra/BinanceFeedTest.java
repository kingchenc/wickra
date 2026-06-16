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
}
