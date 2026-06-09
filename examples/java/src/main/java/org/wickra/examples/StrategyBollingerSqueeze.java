package org.wickra.examples;

import org.wickra.Atr;
import org.wickra.BollingerBands;
import org.wickra.BollingerOutput;
import org.wickra.examples.MarketData.Bar;

import java.util.ArrayList;
import java.util.List;

/**
 * Breakout: when Bollinger bandwidth is tight (a "squeeze") and price closes above the
 * upper band, go long with an ATR(14) trailing stop.
 */
public final class StrategyBollingerSqueeze {
    public static void main(String[] args) throws Exception {
        Bar[] bars = args.length > 0 ? MarketData.loadOhlcvCsv(args[0]) : MarketData.syntheticCandles(2000);

        try (BollingerBands bollinger = new BollingerBands(20, 2.0);
             Atr atr = new Atr(14)) {

            List<Double> returns = new ArrayList<>();
            int trades = 0;
            boolean inPosition = false;
            double entry = 0.0;
            double stop = 0.0;

            for (Bar b : bars) {
                BollingerOutput band = bollinger.update(b.close());
                double atrValue = atr.update(b.open(), b.high(), b.low(), b.close(), b.volume(), b.timestamp());
                if (band == null || !Double.isFinite(atrValue)) {
                    continue;
                }

                double bandwidth = band.middle() != 0.0
                        ? (band.upper() - band.lower()) / band.middle()
                        : Double.MAX_VALUE;

                if (!inPosition && bandwidth < 0.06 && b.close() > band.upper()) {
                    inPosition = true;
                    entry = b.close();
                    stop = b.close() - 2.0 * atrValue;
                    trades++;
                } else if (inPosition) {
                    stop = Math.max(stop, b.close() - 2.0 * atrValue); // trail the stop up
                    if (b.close() < stop) {
                        returns.add((b.close() - entry) / entry);
                        inPosition = false;
                    }
                }
            }

            Equity.print("Bollinger squeeze", Equity.summarize(returns, trades));
        }
    }
}
