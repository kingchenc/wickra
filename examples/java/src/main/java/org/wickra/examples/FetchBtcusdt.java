package org.wickra.examples;

import org.wickra.BinanceFeed;
import org.wickra.BinanceInterval;
import org.wickra.Candle;

import java.io.BufferedWriter;
import java.nio.file.Files;
import java.nio.file.Path;

/**
 * Download real BTCUSDT hourly klines from the Binance REST API into a CSV that the
 * other examples can consume, using Wickra's native fetcher — no third-party HTTP or
 * JSON library. Requires network access (build-only in CI).
 */
public final class FetchBtcusdt {
    public static void main(String[] args) throws Exception {
        System.out.println("Fetching 500 BTCUSDT 1h klines from Binance...");

        Candle[] klines = BinanceFeed.fetchKlines("BTCUSDT", BinanceInterval.ONE_HOUR, 500, -1L, -1L, null);

        Path dir = Path.of("data");
        Files.createDirectories(dir);
        Path path = dir.resolve("btcusdt_1h.csv");

        try (BufferedWriter writer = Files.newBufferedWriter(path)) {
            writer.write("timestamp,open,high,low,close,volume");
            writer.newLine();
            for (Candle k : klines) {
                writer.write((long) k.timestamp() + "," + k.open() + "," + k.high() + ","
                        + k.low() + "," + k.close() + "," + k.volume());
                writer.newLine();
            }
        }

        System.out.printf("Wrote %d klines to %s%n", klines.length, path.toAbsolutePath());
    }
}
