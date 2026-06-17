package org.wickra.examples;

import org.wickra.Rsi;
import org.wickra.examples.MarketData.Bar;

import java.util.ArrayList;
import java.util.List;

/**
 * Strategy example: RSI(14) mean-reversion.
 *
 * <p>Go long when RSI(14) drops below 30 (oversold), exit when it recovers above
 * 70 (overbought). 0.1% fees per trade. The Java counterpart of
 * {@code examples/python/strategy_rsi_mean_reversion.py}, printing the same
 * summary. Uses the checked-in {@code examples/data/btcusdt-1h.csv} dataset
 * (pass a CSV path to override).
 */
public final class StrategyRsiMeanReversion {
    private static final double FEE = 0.001;
    private static final double OVERSOLD = 30.0;
    private static final double OVERBOUGHT = 70.0;

    public static void main(String[] args) throws Exception {
        Bar[] bars = args.length > 0 ? MarketData.loadOhlcvCsv(args[0]) : MarketData.bundledCandles("btcusdt-1h.csv");

        try (Rsi rsi = new Rsi(14)) {
            boolean inPosition = false;
            double entryPrice = 0.0;
            List<Double> closedTrades = new ArrayList<>();
            double equity = 1.0;
            List<Double> equityCurve = new ArrayList<>();

            for (Bar b : bars) {
                double value = rsi.update(b.close());
                double price = b.close();
                equityCurve.add(inPosition ? equity * (price / entryPrice) : equity);
                if (!Double.isFinite(value)) {
                    continue;
                }

                if (!inPosition && value < OVERSOLD) {
                    entryPrice = price;
                    equity *= 1.0 - FEE;
                    inPosition = true;
                } else if (inPosition && value > OVERBOUGHT) {
                    double tradeRet = price / entryPrice - 1.0;
                    closedTrades.add(tradeRet);
                    equity *= (1.0 + tradeRet) * (1.0 - FEE);
                    inPosition = false;
                }
            }

            if (inPosition) {
                double tradeRet = bars[bars.length - 1].close() / entryPrice - 1.0;
                closedTrades.add(tradeRet);
                equity *= (1.0 + tradeRet) * (1.0 - FEE);
            }

            Equity.printSummary("RSI Mean-Reversion (1h, BTCUSDT)",
                    bars[0].close(), bars[bars.length - 1].close(), bars.length, closedTrades, equity, equityCurve);
        }
    }
}
