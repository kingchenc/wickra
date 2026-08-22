package org.wickra;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * The C ABI declares candle timestamps as {@code const int64_t *}, but the
 * generated {@code batch} methods typed every input array as {@code double[]}
 * and allocated it with {@code JAVA_DOUBLE}. The native side then reinterpreted
 * the IEEE-754 bit pattern as an integer, so a millisecond epoch such as
 * 1700000000000 arrived as roughly 4.8e18. That was inert for indicators which
 * ignore the timestamp, and silently wrong for every session- or
 * calendar-aware one — and it went unnoticed because the Java suite called
 * {@code batch} exactly once, on a scalar indicator with no timestamp at all.
 *
 * <p>Streaming already took a {@code long}, so streaming-versus-batch agreement
 * on a time-aware indicator is the sharpest available check: the two paths
 * cannot agree unless both interpret the timestamp the same way.
 */
class TimestampMarshallingTest {

    private static final int BARS = 400;
    /** One bar a minute, so the series crosses several session boundaries. */
    private static final long START_MILLIS = 1_700_000_000_000L;

    private static long[] timestamps() {
        long[] t = new long[BARS];
        for (int i = 0; i < BARS; i++) {
            t[i] = START_MILLIS + i * 60_000L;
        }
        return t;
    }

    private static double[] column(int field) {
        double[] c = new double[BARS];
        for (int i = 0; i < BARS; i++) {
            double base = 100.0 + Math.sin(i * 0.13) * 5.0;
            c[i] = switch (field) {
                case 0 -> base;
                case 1 -> base + 1.0;
                case 2 -> base - 1.0;
                case 3 -> base + 0.25;
                default -> 10.0 + (i % 7);
            };
        }
        return c;
    }

    @Test
    void sessionVwapBatchMatchesStreaming() {
        double[] open = column(0);
        double[] high = column(1);
        double[] low = column(2);
        double[] close = column(3);
        double[] volume = column(4);
        long[] timestamp = timestamps();

        double[] streamed = new double[BARS];
        try (SessionVwap streaming = new SessionVwap(0)) {
            for (int i = 0; i < BARS; i++) {
                streamed[i] = streaming.update(
                        open[i], high[i], low[i], close[i], volume[i], timestamp[i]);
            }
        }

        double[] batched;
        try (SessionVwap vectorized = new SessionVwap(0)) {
            batched = vectorized.batch(open, high, low, close, volume, timestamp);
        }

        boolean anyValue = false;
        for (int i = 0; i < BARS; i++) {
            if (Double.isNaN(streamed[i]) && Double.isNaN(batched[i])) {
                continue;
            }
            anyValue = true;
            assertEquals(streamed[i], batched[i], "bar " + i);
        }
        assertTrue(anyValue, "the indicator must emit at least one value");
    }

    @Test
    void turnOfMonthBatchMatchesStreaming() {
        // A calendar-aware indicator on daily bars, so the timestamp drives the
        // month boundary rather than an intraday session reset.
        int bars = 200;
        double[] open = new double[bars];
        double[] high = new double[bars];
        double[] low = new double[bars];
        double[] close = new double[bars];
        double[] volume = new double[bars];
        long[] timestamp = new long[bars];
        for (int i = 0; i < bars; i++) {
            double base = 100.0 + Math.cos(i * 0.21) * 3.0;
            open[i] = base;
            high[i] = base + 0.8;
            low[i] = base - 0.8;
            close[i] = base + 0.1;
            volume[i] = 5.0 + (i % 4);
            timestamp[i] = START_MILLIS + i * 86_400_000L;
        }

        double[] streamed = new double[bars];
        try (TurnOfMonth streaming = new TurnOfMonth(3, 3, 0)) {
            for (int i = 0; i < bars; i++) {
                streamed[i] = streaming.update(
                        open[i], high[i], low[i], close[i], volume[i], timestamp[i]);
            }
        }

        double[] batched;
        try (TurnOfMonth vectorized = new TurnOfMonth(3, 3, 0)) {
            batched = vectorized.batch(open, high, low, close, volume, timestamp);
        }

        for (int i = 0; i < bars; i++) {
            if (Double.isNaN(streamed[i]) && Double.isNaN(batched[i])) {
                continue;
            }
            assertEquals(streamed[i], batched[i], "bar " + i);
        }
    }
}
