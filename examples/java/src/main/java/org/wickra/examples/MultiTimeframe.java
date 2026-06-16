package org.wickra.examples;

import org.wickra.Candle;
import org.wickra.Ema;
import org.wickra.Resampler;
import org.wickra.examples.MarketData.Bar;

import java.util.ArrayList;
import java.util.List;

/** Resample a 1-minute series into higher timeframes and run an indicator per timeframe. */
public final class MultiTimeframe {
    public static void main(String[] args) {
        Bar[] oneMinute = MarketData.syntheticCandles(1200, 0L, 60_000L);

        System.out.println("EMA(20) of close across timeframes (resampled from 1-minute bars):");
        for (int factor : new int[] {1, 5, 15}) {
            Bar[] bars = resample(oneMinute, factor);
            try (Ema ema = new Ema(20)) {
                double last = 0;
                for (Bar b : bars) {
                    last = ema.update(b.close());
                }
                System.out.printf("  %2dm: %5d bars  EMA(20) last = %.4f%n", factor, bars.length, last);
            }
        }
    }

    private static Bar[] resample(Bar[] source, int factor) {
        if (factor <= 1) {
            return source;
        }
        // Native Resampler: bucket by an absolute timeframe (the synthetic bars step
        // 60_000 ms, so factor minutes == factor*60_000 ms). No hand-written bucketing.
        List<Bar> output = new ArrayList<>();
        try (Resampler r = new Resampler((long) factor * 60_000L)) {
            for (Bar b : source) {
                Candle c = r.update(b.open(), b.high(), b.low(), b.close(), b.volume(), b.timestamp());
                if (c != null) {
                    output.add(toBar(c));
                }
            }
            Candle last = r.flush();
            if (last != null) {
                output.add(toBar(last));
            }
        }
        return output.toArray(new Bar[0]);
    }

    private static Bar toBar(Candle c) {
        return new Bar(c.open(), c.high(), c.low(), c.close(), c.volume(), (long) c.timestamp());
    }
}
