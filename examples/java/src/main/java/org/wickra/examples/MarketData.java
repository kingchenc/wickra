package org.wickra.examples;

import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.List;

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
     * Loads an OHLCV CSV. Accepts rows of {@code timestamp,open,high,low,close,volume}
     * or {@code open,high,low,close,volume}; a non-numeric first row is treated as a header.
     */
    public static Bar[] loadOhlcvCsv(String path) throws IOException {
        List<Bar> bars = new ArrayList<>();
        for (String rawLine : Files.readAllLines(Path.of(path))) {
            String line = rawLine.trim();
            if (line.isEmpty()) {
                continue;
            }
            String[] cols = line.split(",");
            if (!isNumeric(cols[0])) {
                continue; // header row
            }
            if (cols.length >= 6) {
                bars.add(new Bar(
                        Double.parseDouble(cols[1]), Double.parseDouble(cols[2]),
                        Double.parseDouble(cols[3]), Double.parseDouble(cols[4]),
                        Double.parseDouble(cols[5]), Long.parseLong(cols[0])));
            } else {
                bars.add(new Bar(
                        Double.parseDouble(cols[0]), Double.parseDouble(cols[1]),
                        Double.parseDouble(cols[2]), Double.parseDouble(cols[3]),
                        Double.parseDouble(cols[4]), bars.size()));
            }
        }
        return bars.toArray(new Bar[0]);
    }

    private static boolean isNumeric(String s) {
        try {
            Double.parseDouble(s);
            return true;
        } catch (NumberFormatException e) {
            return false;
        }
    }
}
