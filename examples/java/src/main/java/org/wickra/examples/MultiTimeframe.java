package org.wickra.examples;

import org.wickra.Ema;
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
        List<Bar> output = new ArrayList<>();
        for (int i = 0; i < source.length; i += factor) {
            int end = Math.min(i + factor, source.length);
            double high = Double.MIN_VALUE;
            double low = Double.MAX_VALUE;
            double volume = 0;
            for (int j = i; j < end; j++) {
                high = Math.max(high, source[j].high());
                low = Math.min(low, source[j].low());
                volume += source[j].volume();
            }
            output.add(new Bar(source[i].open(), high, low, source[end - 1].close(), volume, source[i].timestamp()));
        }
        return output.toArray(new Bar[0]);
    }
}
