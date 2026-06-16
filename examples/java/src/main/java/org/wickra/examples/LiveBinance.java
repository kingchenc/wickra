package org.wickra.examples;

import org.wickra.BinanceFeed;
import org.wickra.BinanceInterval;
import org.wickra.Ema;
import org.wickra.KlineEvent;

/**
 * Stream live BTCUSDT 1-minute klines from Binance and feed each close through EMA(20),
 * using Wickra's native BinanceFeed — no third-party WebSocket or JSON library.
 * Requires network access (build-only in CI). Runs for up to 60 seconds.
 */
public final class LiveBinance {
    public static void main(String[] args) {
        System.out.println("Streaming live BTCUSDT 1-minute klines from Binance (up to 60s)...");

        // Native feed: a blocking poll over the same tested stream as the Rust core.
        // next(timeoutMillis) returns the event, or null on timeout (poll again).
        try (BinanceFeed feed = new BinanceFeed("BTCUSDT", BinanceInterval.ONE_MINUTE);
                Ema ema = new Ema(20)) {
            long deadline = System.currentTimeMillis() + 60_000L;
            while (System.currentTimeMillis() < deadline) {
                KlineEvent event = feed.next(1000);
                if (event == null) {
                    continue;
                }
                System.out.printf("close=%.2f  EMA(20)=%.2f%n", event.close(), ema.update(event.close()));
            }
        }
        System.out.println("Done (time limit reached).");
    }
}
