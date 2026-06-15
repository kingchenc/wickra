package org.wickra;

import org.junit.jupiter.api.Test;

import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Cross-language data-layer parity: replay the shared golden tick stream through
 * the TickAggregator and check the candles against the Rust reference, with and
 * without gap filling.
 */
class DataLayerTest {
    private static Path goldenDir() {
        java.io.File d = new java.io.File("").getAbsoluteFile();
        while (d != null) {
            java.io.File g = new java.io.File(d, "testdata/golden");
            if (g.isDirectory()) {
                return g.toPath();
            }
            d = d.getParentFile();
        }
        throw new IllegalStateException("testdata/golden not found");
    }

    private static double[][] read(String name) throws IOException {
        List<String> lines = Files.readAllLines(goldenDir().resolve(name + ".csv"));
        List<double[]> rows = new ArrayList<>();
        for (int i = 1; i < lines.size(); i++) {
            if (lines.get(i).isEmpty()) {
                continue;
            }
            String[] p = lines.get(i).split(",");
            double[] r = new double[p.length];
            for (int k = 0; k < p.length; k++) {
                r[k] = Double.parseDouble(p[k]);
            }
            rows.add(r);
        }
        return rows.toArray(new double[0][]);
    }

    @Test
    void resamplerMatchesGolden() throws IOException {
        double[][] input = read("input"); // open,high,low,close,volume (timestamp = row index)
        try (Resampler r = new Resampler(5)) {
            List<double[]> got = new ArrayList<>();
            for (int i = 0; i < input.length; i++) {
                Candle c = r.update(input[i][0], input[i][1], input[i][2], input[i][3], input[i][4], i);
                if (c != null) {
                    got.add(new double[]{c.open(), c.high(), c.low(), c.close(), c.volume(), (double) c.timestamp()});
                }
            }
            Candle f = r.flush();
            if (f != null) {
                got.add(new double[]{f.open(), f.high(), f.low(), f.close(), f.volume(), (double) f.timestamp()});
            }
            double[][] want = read("data_resampled");
            assertEquals(want.length, got.size(), "resample candle count");
            for (int i = 0; i < got.size(); i++) {
                for (int j = 0; j < 6; j++) {
                    double w = want[i][j];
                    assertTrue(Math.abs(got.get(i)[j] - w) <= 1e-9 * Math.max(1, Math.abs(w)),
                        "resample row " + i + " col " + j + ": " + got.get(i)[j] + " vs " + w);
                }
            }
        }
    }

    @Test
    void candleReaderMatchesGolden() throws IOException {
        String csv = Files.readString(goldenDir().resolve("data_csv.csv"));
        try (CandleReader reader = new CandleReader(csv)) {
            Candle[] candles = reader.read();
            double[][] want = read("data_csv_candles");
            assertEquals(want.length, candles.length, "candle reader count");
            for (int i = 0; i < candles.length; i++) {
                Candle k = candles[i];
                double[] got = {k.open(), k.high(), k.low(), k.close(), k.volume(), (double) k.timestamp()};
                for (int j = 0; j < 6; j++) {
                    double w = want[i][j];
                    assertTrue(Math.abs(got[j] - w) <= 1e-9 * Math.max(1, Math.abs(w)),
                        "candle reader row " + i + " col " + j + ": " + got[j] + " vs " + w);
                }
            }
        }
    }

    @Test
    void tickAggregatorMatchesGolden() throws IOException {
        double[][] ticks = read("data_ticks");
        String[][] cases = {{"data_candles", "false"}, {"data_candles_gap", "true"}};
        for (String[] c : cases) {
            boolean gap = Boolean.parseBoolean(c[1]);
            try (TickAggregator a = new TickAggregator(1000, gap)) {
                List<double[]> got = new ArrayList<>();
                for (double[] t : ticks) {
                    for (Candle k : a.push(t[0], t[1], (long) t[2])) {
                        got.add(new double[]{
                            k.open(), k.high(), k.low(), k.close(), k.volume(), (double) k.timestamp()
                        });
                    }
                }
                double[][] want = read(c[0]);
                assertEquals(want.length, got.size(), c[0] + " candle count");
                for (int i = 0; i < got.size(); i++) {
                    for (int j = 0; j < 6; j++) {
                        double w = want[i][j];
                        assertTrue(Math.abs(got.get(i)[j] - w) <= 1e-9 * Math.max(1, Math.abs(w)),
                            c[0] + " row " + i + " col " + j + ": " + got.get(i)[j] + " vs " + w);
                    }
                }
            }
        }
    }
}
