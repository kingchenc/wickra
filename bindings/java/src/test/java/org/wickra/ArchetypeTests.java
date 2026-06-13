package org.wickra;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * One representative per FFI archetype, exercising every marshalling path the
 * generator produces (scalar, candle, pairwise, multi-output, bars, profile,
 * values-profile, array-input). Garbage marshalling surfaces as NaN, wild
 * values, or crashes — so finite/sane assertions are the real check.
 */
class ArchetypeTests {

    private static double[] candle(int i) {
        double close = 100.0 + 10.0 * Math.sin(i * 0.3);
        double open = 100.0 + 10.0 * Math.sin((i - 1) * 0.3);
        double high = Math.max(open, close) + 1.0;
        double low = Math.min(open, close) - 1.0;
        return new double[] {open, high, low, close, 1_000.0};
    }

    @Test
    void scalarEmaIsFiniteAfterWarmup() {
        try (Ema ema = new Ema(3)) {
            double last = Double.NaN;
            for (int i = 1; i <= 10; i++) {
                last = ema.update(i);
            }
            assertTrue(Double.isFinite(last));
            assertTrue(last >= 1.0 && last <= 10.0);
        }
    }

    @Test
    void queryWarmupPeriodAndIsReady() {
        try (Sma sma = new Sma(3)) {
            assertEquals(3, sma.warmupPeriod());
            assertFalse(sma.isReady());
            sma.update(1.0);
            sma.update(2.0);
            assertFalse(sma.isReady());
            sma.update(3.0);
            assertTrue(sma.isReady());
            sma.reset();
            assertFalse(sma.isReady());
        }
    }

    @Test
    void scalarEmaBatchMatchesStreaming() {
        double[] input = new double[50];
        for (int i = 0; i < input.length; i++) {
            input[i] = 100.0 + 10.0 * Math.sin(i * 0.2);
        }
        double streamedLast;
        try (Ema ema = new Ema(10)) {
            double v = Double.NaN;
            for (double x : input) {
                v = ema.update(x);
            }
            streamedLast = v;
        }
        double[] batched;
        try (Ema ema = new Ema(10)) {
            batched = ema.batch(input);
        }
        assertEquals(input.length, batched.length);
        assertEquals(streamedLast, batched[batched.length - 1], 1e-9);
    }

    @Test
    void candleAtrIsFinitePositive() {
        try (Atr atr = new Atr(3)) {
            double last = Double.NaN;
            for (int i = 0; i < 20; i++) {
                double[] c = candle(i);
                last = atr.update(c[0], c[1], c[2], c[3], c[4], i * 60_000L);
            }
            assertTrue(Double.isFinite(last));
            assertTrue(last > 0.0);
        }
    }

    @Test
    void pairwiseBetaIsFinite() {
        try (Beta beta = new Beta(5)) {
            double last = Double.NaN;
            for (int i = 0; i < 30; i++) {
                double market = 100.0 + 10.0 * Math.sin(i * 0.5);
                double asset = 50.0 + 6.0 * Math.sin(i * 0.5 + 0.2);
                last = beta.update(market, asset);
            }
            assertTrue(Double.isFinite(last));
        }
    }

    @Test
    void multiOutputAdxReturnsFiniteRecord() {
        try (Adx adx = new Adx(5)) {
            AdxOutput result = null;
            for (int i = 0; i < 60; i++) {
                double[] c = candle(i);
                result = adx.update(c[0], c[1], c[2], c[3], c[4], i * 60_000L);
            }
            assertNotNull(result);
            assertTrue(Double.isFinite(result.adx()));
            assertTrue(Double.isFinite(result.plusDi()));
            assertTrue(Double.isFinite(result.minusDi()));
        }
    }

    @Test
    void barsDollarBarsEmitsBars() {
        try (DollarBars bars = new DollarBars(5_000.0)) {
            int total = 0;
            for (int i = 0; i < 200; i++) {
                double[] c = candle(i);
                total += bars.update(c[0], c[1], c[2], c[3], c[4], i * 60_000L).length;
            }
            assertTrue(total > 0);
        }
    }

    @Test
    void profileVolumeProfileReturnsValues() {
        try (VolumeProfile profile = new VolumeProfile(20, 8)) {
            VolumeProfileOutputScalars result = null;
            for (int i = 0; i < 60; i++) {
                double[] c = candle(i);
                result = profile.update(c[0], c[1], c[2], c[3], c[4], i * 60_000L);
            }
            assertNotNull(result);
            assertNotNull(result.values());
            assertTrue(result.priceLow() <= result.priceHigh());
        }
    }

    @Test
    void profileValuesDayOfWeekProfileNoCrash() {
        try (DayOfWeekProfile profile = new DayOfWeekProfile(0)) {
            double[] result = null;
            for (int i = 0; i < 60; i++) {
                double close = 100.0 + 5.0 * Math.sin(i * 0.2);
                result = profile.update(close, close + 1, close - 1, close, 1_000.0, i * 86_400_000L);
            }
            if (result != null) {
                for (double v : result) {
                    assertTrue(Double.isFinite(v));
                }
            }
        }
    }

    @Test
    void arrayInputDepthSlopeIsFinite() {
        try (DepthSlope slope = new DepthSlope()) {
            double[] bidPrice = {99.0, 98.0, 97.0};
            double[] bidSize = {10.0, 20.0, 30.0};
            double[] askPrice = {101.0, 102.0, 103.0};
            double[] askSize = {12.0, 22.0, 32.0};
            double result = slope.update(bidPrice, bidSize, askPrice, askSize);
            assertTrue(Double.isFinite(result));
        }
    }
}
