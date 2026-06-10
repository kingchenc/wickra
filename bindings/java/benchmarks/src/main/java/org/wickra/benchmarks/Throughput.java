package org.wickra.benchmarks;

import java.util.Arrays;
import java.util.Locale;
import org.wickra.Atr;
import org.wickra.MacdIndicator;
import org.wickra.Sma;

/**
 * Throughput benchmark for the Wickra Java binding.
 *
 * <p>Measures how many indicator updates per second the binding sustains, both
 * per-tick (streaming {@code update}) and bulk ({@code batch}), over a synthetic
 * OHLCV series. It is the Java counterpart of the Node {@code throughput.js} and
 * the Rust criterion benches: it benchmarks Wickra's own O(1) streaming engine
 * across the Java FFM &lt;-&gt; C-ABI boundary (there is no comparable streaming
 * TA library on Maven Central to compare against), so the headline number is raw
 * per-binding throughput / FFI overhead, not a cross-library ratio.
 *
 * <p>Three indicators are timed, chosen by FFI call-signature archetype rather
 * than algorithm: SMA (1-in -&gt; 1-out), ATR (multi-in -&gt; 1-out), and MACD
 * (1-in -&gt; multi-out). Streaming is timed for all three; batch only for the
 * single-output SMA and ATR (multi-output batch is not exposed uniformly).
 *
 * <p>Install the binding and build the C ABI library first, then run from the
 * repo root:
 *
 * <pre>
 *   cargo build -p wickra-c --release
 *   mvn -q -f bindings/java install -DskipTests
 *   mvn -q -f bindings/java/benchmarks exec:exec -Dexec.mainClass=org.wickra.benchmarks.Throughput
 * </pre>
 */
public final class Throughput {
    private Throughput() {}

    public static void main(String[] args) {
        int bars = 200_000;
        for (int i = 0; i < args.length - 1; i++) {
            if (args[i].equals("--bars")) {
                try {
                    int n = Integer.parseInt(args[i + 1]);
                    if (n >= 1000) {
                        bars = n;
                    }
                } catch (NumberFormatException ignored) {
                    // keep default
                }
            }
        }

        // Deterministic synthetic OHLCV (no RNG, so runs are comparable).
        double[] open = new double[bars];
        double[] high = new double[bars];
        double[] low = new double[bars];
        double[] close = new double[bars];
        double[] volume = new double[bars];
        double[] timestamp = new double[bars];
        for (int i = 0; i < bars; i++) {
            double mid = 100 + Math.sin(i * 0.001) * 20 + i * 1e-4;
            double c = mid + Math.sin(i * 0.05) * 2;
            close[i] = c;
            open[i] = mid;
            high[i] = Math.max(c, mid) + 1.5;
            low[i] = Math.min(c, mid) - 1.5;
            volume[i] = 1000 + (i % 97) * 13;
            timestamp[i] = i;
        }

        final int n = bars;

        // SMA (scalar 1-in/1-out), ATR (multi-in/1-out), MACD (1-in/multi-out).
        Indicator[] indicators = {
            new Indicator("SMA(20)",
                () -> {
                    try (Sma ind = new Sma(20)) {
                        for (int i = 0; i < n; i++) {
                            ind.update(close[i]);
                        }
                    }
                },
                () -> {
                    try (Sma ind = new Sma(20)) {
                        ind.batch(close);
                    }
                }),
            new Indicator("ATR(14)",
                () -> {
                    try (Atr ind = new Atr(14)) {
                        for (int i = 0; i < n; i++) {
                            ind.update(open[i], high[i], low[i], close[i], volume[i], (long) timestamp[i]);
                        }
                    }
                },
                () -> {
                    try (Atr ind = new Atr(14)) {
                        ind.batch(open, high, low, close, volume, timestamp);
                    }
                }),
            new Indicator("MACD(12,26,9)",
                () -> {
                    try (MacdIndicator ind = new MacdIndicator(12, 26, 9)) {
                        for (int i = 0; i < n; i++) {
                            ind.update(close[i]);
                        }
                    }
                },
                null), // multi-output: streaming only
        };

        System.out.printf(Locale.ROOT, "Wickra Java throughput - %,d bars (median of 3 runs)%n%n", bars);
        System.out.printf(Locale.ROOT, "%-22s%20s%18s%n", "Indicator", "streaming (Mupd/s)", "batch (Mupd/s)");
        System.out.println("------------------------------------------------------------");

        for (Indicator ind : indicators) {
            String streamMups = String.format(Locale.ROOT, "%.1f", mups(bars, timeNs(ind.stream)));
            String batchMups = ind.batch == null
                ? "-"
                : String.format(Locale.ROOT, "%.1f", mups(bars, timeNs(ind.batch)));
            System.out.printf(Locale.ROOT, "%-22s%20s%18s%n", ind.name, streamMups, batchMups);
        }

        System.out.println(
            "\nMupd/s = million indicator updates per second. Streaming is the per-tick\n"
            + "update path crossing the Java FFM<->C-ABI boundary once per value; batch is\n"
            + "the bulk array path (one boundary crossing). Higher is better. Numbers are\n"
            + "machine-dependent - use them for relative comparison, not as a speed claim.");
    }

    private static double mups(int bars, double ns) {
        return bars / (ns / 1e9) / 1e6;
    }

    // Median elapsed-ns over a few repetitions, after one warmup pass.
    private static double timeNs(Runnable fn) {
        fn.run(); // warmup (JIT + cache)
        final int reps = 3;
        double[] samples = new double[reps];
        for (int r = 0; r < reps; r++) {
            long t0 = System.nanoTime();
            fn.run();
            samples[r] = System.nanoTime() - t0;
        }
        Arrays.sort(samples);
        return samples[reps / 2];
    }

    private record Indicator(String name, Runnable stream, Runnable batch) {}
}
