package org.wickra.examples;

import org.wickra.Sma;

import java.util.stream.IntStream;

/** Run SMA(20) batch over a panel of assets, serial vs parallel, and report the speedup. */
public final class ParallelAssets {
    public static void main(String[] args) {
        int assets = args.length > 0 ? Integer.parseInt(args[0]) : 500;
        int bars = args.length > 1 ? Integer.parseInt(args[1]) : 20_000;

        double[][] panel = new double[assets][];
        for (int a = 0; a < assets; a++) {
            panel[a] = MarketData.syntheticPrices(bars, 50.0 + a * 0.1);
        }

        // Warm up the JIT and thread pool so the comparison is fair.
        try (Sma warm = new Sma(20)) {
            warm.batch(panel[0]);
        }

        double sink = 0.0;
        long t0 = System.nanoTime();
        for (int a = 0; a < assets; a++) {
            try (Sma sma = new Sma(20)) {
                double[] result = sma.batch(panel[a]);
                sink += result[result.length - 1];
            }
        }
        double serialMs = (System.nanoTime() - t0) / 1e6;

        double[] lasts = new double[assets];
        long t1 = System.nanoTime();
        IntStream.range(0, assets).parallel().forEach(a -> {
            try (Sma sma = new Sma(20)) {
                double[] result = sma.batch(panel[a]);
                lasts[a] = result[result.length - 1];
            }
        });
        double parallelMs = (System.nanoTime() - t1) / 1e6;

        System.out.printf("%d assets x %d bars, SMA(20) batch:%n", assets, bars);
        System.out.printf("  serial   %8.1f ms%n", serialMs);
        System.out.printf("  parallel %8.1f ms  (%.1fx speedup)%n", parallelMs, serialMs / Math.max(parallelMs, 1e-9));
        if (sink == Double.NaN || lasts.length < 0) {
            System.out.println(sink); // keep `sink` live
        }
    }
}
