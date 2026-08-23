package org.wickra;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.util.ArrayList;
import java.util.List;
import org.junit.jupiter.api.Test;

/**
 * The cross-section, order-book, bar-builder and profile families take an input
 * shape the batch generator could not express, so 39 indicators had a native
 * batch the wrapper could not reach. Each shape is checked against feeding the
 * same data one bar at a time.
 */
class BatchShapesTest {

    private static final int BARS = 8;

    private static void assertClose(double got, double want, String label) {
        if (Double.isNaN(want)) {
            assertTrue(Double.isNaN(got), label + ": want NaN, got " + got);
            return;
        }
        assertTrue(Math.abs(got - want) <= 1e-12 * Math.max(1, Math.abs(want)),
                label + ": batch " + got + ", streaming " + want);
    }

    private static double[] slice(double[] src, int from, int len) {
        double[] out = new double[len];
        System.arraycopy(src, from, out, 0, len);
        return out;
    }

    private static boolean[] slice(boolean[] src, int from, int len) {
        boolean[] out = new boolean[len];
        System.arraycopy(src, from, out, 0, len);
        return out;
    }

    @Test
    void crossSectionBatchMatchesStreaming() {
        final int members = 3;
        double[] change = new double[BARS * members];
        double[] volume = new double[BARS * members];
        boolean[] newHigh = new boolean[BARS * members];
        boolean[] newLow = new boolean[BARS * members];
        boolean[] aboveMa = new boolean[BARS * members];
        boolean[] onBuy = new boolean[BARS * members];
        long[] stamps = new long[BARS];
        for (int bar = 0; bar < BARS; bar++) {
            stamps[bar] = bar;
            for (int m = 0; m < members; m++) {
                int at = bar * members + m;
                change[at] = at * 0.37 - 2;
                volume[at] = 100 + at * 5;
                newHigh[at] = (bar + m) % 2 == 0;
                newLow[at] = (bar + m) % 3 == 0;
                aboveMa[at] = m % 2 == 0;
                onBuy[at] = bar % 2 == 0;
            }
        }

        double[] got;
        try (AdvanceDecline batched = new AdvanceDecline()) {
            got = batched.batch(change, volume, newHigh, newLow, aboveMa, onBuy, members, stamps);
        }

        try (AdvanceDecline streamed = new AdvanceDecline()) {
            for (int bar = 0; bar < BARS; bar++) {
                int lo = bar * members;
                double want = streamed.update(slice(change, lo, members), slice(volume, lo, members),
                        slice(newHigh, lo, members), slice(newLow, lo, members),
                        slice(aboveMa, lo, members), slice(onBuy, lo, members), stamps[bar]);
                assertClose(got[bar], want, "bar " + bar);
            }
        }
    }

    @Test
    void orderBookBatchMatchesStreaming() {
        final int depth = 2;
        double[] bidPx = new double[BARS * depth];
        double[] bidSz = new double[BARS * depth];
        double[] askPx = new double[BARS * depth];
        double[] askSz = new double[BARS * depth];
        for (int bar = 0; bar < BARS; bar++) {
            for (int lvl = 0; lvl < depth; lvl++) {
                int at = bar * depth + lvl;
                double drift = bar * 0.25;
                double step = lvl * 0.1;
                bidPx[at] = 100 + drift - step;
                bidSz[at] = 5 + step;
                askPx[at] = 100.2 + drift + step;
                askSz[at] = 4 + step;
            }
        }

        double[] got;
        try (Microprice batched = new Microprice()) {
            got = batched.batch(bidPx, bidSz, depth, askPx, askSz, depth);
        }

        try (Microprice streamed = new Microprice()) {
            for (int bar = 0; bar < BARS; bar++) {
                int lo = bar * depth;
                double want = streamed.update(slice(bidPx, lo, depth), slice(bidSz, lo, depth),
                        slice(askPx, lo, depth), slice(askSz, lo, depth));
                assertClose(got[bar], want, "bar " + bar);
            }
        }
    }

    @Test
    void barBuilderBatchMatchesStreaming() {
        final int n = 12;
        double[] closes = new double[n];
        double[] vols = new double[n];
        long[] stamps = new long[n];
        for (int i = 0; i < n; i++) {
            closes[i] = 100 + i * 3 + (i == 6 ? 40 : 0); // a gap completes several bricks
            vols[i] = 1;
            stamps[i] = i;
        }

        RenkoBrick[] got;
        try (RenkoBars batched = new RenkoBars(1.0)) {
            got = batched.batch(closes, closes, closes, closes, vols, stamps);
        }

        List<RenkoBrick> want = new ArrayList<>();
        try (RenkoBars streamed = new RenkoBars(1.0)) {
            for (int i = 0; i < n; i++) {
                for (RenkoBrick b : streamed.update(closes[i], closes[i], closes[i], closes[i], vols[i], stamps[i])) {
                    want.add(b);
                }
            }
        }

        assertEquals(want.size(), got.length);
        for (int i = 0; i < want.size(); i++) {
            assertEquals(want.get(i), got[i], "brick " + i);
        }
    }

    @Test
    void profileBatchMatchesStreaming() {
        final int n = 10;
        double[] closes = new double[n];
        double[] vols = new double[n];
        long[] stamps = new long[n];
        for (int i = 0; i < n; i++) {
            closes[i] = 100 + i;
            vols[i] = 10;
            stamps[i] = (long) i * 86_400_000L; // one day apart
        }

        double[][] got;
        try (DayOfWeekProfile batched = new DayOfWeekProfile(0)) {
            got = batched.batch(closes, closes, closes, closes, vols, stamps);
        }

        int emitted = 0;
        try (DayOfWeekProfile streamed = new DayOfWeekProfile(0)) {
            for (int i = 0; i < n; i++) {
                double[] want = streamed.update(closes[i], closes[i], closes[i], closes[i], vols[i], stamps[i]);
                if (want == null) {
                    for (int k = 0; k < got[i].length; k++) {
                        assertTrue(Double.isNaN(got[i][k]), "warmup row " + i + " bucket " + k);
                    }
                    continue;
                }
                emitted++;
                for (int k = 0; k < want.length; k++) {
                    assertClose(got[i][k], want[k], "row " + i + " bucket " + k);
                }
            }
        }
        assertTrue(emitted > 0, "the fixture must clear warmup");
        assertArrayEquals(new int[] {n}, new int[] {got.length});
    }
}
