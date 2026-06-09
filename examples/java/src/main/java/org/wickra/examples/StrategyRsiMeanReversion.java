package org.wickra.examples;

import org.wickra.Rsi;
import org.wickra.examples.MarketData.Bar;

import java.util.ArrayList;
import java.util.List;

/** Mean reversion: go long when RSI(14) drops below 30, exit when it recovers above 50. */
public final class StrategyRsiMeanReversion {
    public static void main(String[] args) throws Exception {
        Bar[] bars = args.length > 0 ? MarketData.loadOhlcvCsv(args[0]) : MarketData.syntheticCandles(2000);

        try (Rsi rsi = new Rsi(14)) {
            List<Double> returns = new ArrayList<>();
            int trades = 0;
            boolean inPosition = false;
            double entry = 0.0;

            for (Bar b : bars) {
                double value = rsi.update(b.close());
                if (!Double.isFinite(value)) {
                    continue;
                }
                if (!inPosition && value < 30.0) {
                    inPosition = true;
                    entry = b.close();
                    trades++;
                } else if (inPosition && value > 50.0) {
                    returns.add((b.close() - entry) / entry);
                    inPosition = false;
                }
            }

            Equity.print("RSI mean-reversion", Equity.summarize(returns, trades));
        }
    }
}
