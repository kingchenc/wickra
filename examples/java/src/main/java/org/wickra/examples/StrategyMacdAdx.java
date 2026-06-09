package org.wickra.examples;

import org.wickra.Adx;
import org.wickra.AdxOutput;
import org.wickra.MacdIndicator;
import org.wickra.MacdOutput;
import org.wickra.examples.MarketData.Bar;

import java.util.ArrayList;
import java.util.List;

/**
 * Trend follower: enter long on a MACD histogram cross up, but only when ADX(14) &gt; 20
 * confirms a trend; exit when the histogram crosses back below zero.
 */
public final class StrategyMacdAdx {
    public static void main(String[] args) throws Exception {
        Bar[] bars = args.length > 0 ? MarketData.loadOhlcvCsv(args[0]) : MarketData.syntheticCandles(2000);

        try (MacdIndicator macd = new MacdIndicator(12, 26, 9);
             Adx adx = new Adx(14)) {

            List<Double> returns = new ArrayList<>();
            int trades = 0;
            boolean inPosition = false;
            double entry = 0.0;
            double prevHistogram = Double.NaN;

            for (Bar b : bars) {
                MacdOutput m = macd.update(b.close());
                AdxOutput a = adx.update(b.open(), b.high(), b.low(), b.close(), b.volume(), b.timestamp());
                if (m == null || a == null) {
                    continue;
                }

                boolean trending = a.adx() > 20.0;
                if (!inPosition && trending && Double.isFinite(prevHistogram)
                        && prevHistogram <= 0.0 && m.histogram() > 0.0) {
                    inPosition = true;
                    entry = b.close();
                    trades++;
                } else if (inPosition && m.histogram() < 0.0) {
                    returns.add((b.close() - entry) / entry);
                    inPosition = false;
                }

                prevHistogram = m.histogram();
            }

            Equity.print("MACD + ADX trend", Equity.summarize(returns, trades));
        }
    }
}
