package org.wickra.examples;

import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;

import org.wickra.Candle;
import org.wickra.CandleReader;

/**
 * Deterministic synthetic market data plus a small OHLCV CSV loader, shared by
 * the offline examples so they run without network access.
 */
public final class MarketData {
    private MarketData() {
    }

    /** One OHLCV bar with a millisecond timestamp. */
    public record Bar(double open, double high, double low, double close, double volume, long timestamp) {
    }

    /** A reproducible price path (trend + two cycles), no randomness. */
    public static double[] syntheticPrices(int count) {
        return syntheticPrices(count, 100.0);
    }

    public static double[] syntheticPrices(int count, double start) {
        double[] prices = new double[count];
        for (int i = 0; i < count; i++) {
            prices[i] = start + 12.0 * Math.sin(i * 0.05) + 5.0 * Math.sin(i * 0.013) + i * 0.01;
        }
        return prices;
    }

    /** A reproducible OHLCV series derived from {@link #syntheticPrices(int)}. */
    public static Bar[] syntheticCandles(int count) {
        return syntheticCandles(count, 0L, 3_600_000L);
    }

    public static Bar[] syntheticCandles(int count, long startTimestamp, long stepMs) {
        double[] prices = syntheticPrices(count + 1);
        Bar[] bars = new Bar[count];
        for (int i = 0; i < count; i++) {
            double open = prices[i];
            double close = prices[i + 1];
            double high = Math.max(open, close) + 0.5 + Math.abs(Math.sin(i * 0.7));
            double low = Math.min(open, close) - 0.5 - Math.abs(Math.cos(i * 0.7));
            double volume = 1_000.0 + 500.0 * (1.0 + Math.sin(i * 0.1));
            bars[i] = new Bar(open, high, low, close, volume, startTimestamp + (long) i * stepMs);
        }
        return bars;
    }

    /**
     * Loads a {@code timestamp,open,high,low,close,volume} OHLCV CSV with Wickra's
     * native CandleReader (header validation, BOM and field-whitespace tolerance) —
     * no manual CSV parsing.
     */
    public static Bar[] loadOhlcvCsv(String path) throws IOException {
        String csv = Files.readString(Path.of(path));
        try (CandleReader reader = new CandleReader(csv)) {
            Candle[] candles = reader.read();
            Bar[] bars = new Bar[candles.length];
            for (int i = 0; i < candles.length; i++) {
                Candle c = candles[i];
                bars[i] = new Bar(c.open(), c.high(), c.low(), c.close(), c.volume(), (long) c.timestamp());
            }
            return bars;
        }
    }

    /**
     * Loads one of the checked-in datasets under examples/data. The Java
     * examples run from the examples/java directory, so ../data is examples/data.
     */
    public static Bar[] bundledCandles(String filename) throws IOException {
        return loadOhlcvCsv(Path.of("..", "data", filename).toString());
    }
}
