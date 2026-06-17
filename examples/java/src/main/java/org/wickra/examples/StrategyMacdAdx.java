package org.wickra.examples;

import org.wickra.Adx;
import org.wickra.AdxOutput;
import org.wickra.MacdIndicator;
import org.wickra.MacdOutput;
import org.wickra.examples.MarketData.Bar;

import java.util.ArrayList;
import java.util.List;

/**
 * Strategy example: MACD crossover with ADX trend-strength filter.
 *
 * <p>Enters long on a MACD histogram cross up (the histogram turns positive)
 * while ADX(14) &gt; 20 (a directional market); exits on the opposite MACD
 * crossover regardless of ADX. 0.1% fees per trade. The Java counterpart of
 * {@code examples/python/strategy_macd_adx.py}, printing the same summary. Uses
 * the checked-in {@code examples/data/btcusdt-1h.csv} dataset (pass a CSV path to
 * override).
 */
public final class StrategyMacdAdx {
    private static final double FEE = 0.001;
    private static final double ADX_FLOOR = 20.0;

    public static void main(String[] args) throws Exception {
        Bar[] bars = args.length > 0 ? MarketData.loadOhlcvCsv(args[0]) : MarketData.bundledCandles("btcusdt-1h.csv");

        try (MacdIndicator macd = new MacdIndicator(12, 26, 9);
             Adx adx = new Adx(14)) {

            boolean inPosition = false;
            double entryPrice = 0.0;
            List<Double> closedTrades = new ArrayList<>();
            double equity = 1.0;
            List<Double> equityCurve = new ArrayList<>();
            boolean havePrev = false;
            boolean prevSign = false;

            for (Bar b : bars) {
                MacdOutput m = macd.update(b.close());
                AdxOutput a = adx.update(b.open(), b.high(), b.low(), b.close(), b.volume(), b.timestamp());
                double price = b.close();
                equityCurve.add(inPosition ? equity * (price / entryPrice) : equity);

                if (m == null || a == null) {
                    continue;
                }

                boolean histSign = m.histogram() > 0.0;
                boolean crossUp = havePrev && !prevSign && histSign;
                boolean crossDown = havePrev && prevSign && !histSign;
                havePrev = true;
                prevSign = histSign;

                if (!inPosition && crossUp && a.adx() > ADX_FLOOR) {
                    entryPrice = price;
                    equity *= 1.0 - FEE;
                    inPosition = true;
                } else if (inPosition && crossDown) {
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

            Equity.printSummary("MACD + ADX Trend Filter (1h, BTCUSDT)",
                    bars[0].close(), bars[bars.length - 1].close(), bars.length, closedTrades, equity, equityCurve);
        }
    }
}
