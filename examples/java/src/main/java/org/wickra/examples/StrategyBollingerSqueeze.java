package org.wickra.examples;

import org.wickra.Atr;
import org.wickra.BollingerBands;
import org.wickra.BollingerOutput;
import org.wickra.examples.MarketData.Bar;

import java.util.ArrayDeque;
import java.util.ArrayList;
import java.util.Deque;
import java.util.List;

/**
 * Strategy example: Bollinger-squeeze breakout with an ATR(14) trailing stop.
 *
 * <p>Enters long when Bollinger bandwidth makes a new SQUEEZE_LOOKBACK low (a
 * volatility squeeze) and price closes above the upper band; exits on an ATR(14)
 * trailing stop or when the upper band falls back below the entry. 0.1% fees per
 * trade. The Java counterpart of
 * {@code examples/python/strategy_bollinger_squeeze.py}, printing the same
 * summary. Uses the checked-in {@code examples/data/btcusdt-1d.csv} dataset
 * (pass a CSV path to override).
 */
public final class StrategyBollingerSqueeze {
    private static final double FEE = 0.001;
    private static final double ATR_STOP_MULT = 2.0;
    private static final int SQUEEZE_LOOKBACK = 180;

    public static void main(String[] args) throws Exception {
        Bar[] bars = args.length > 0 ? MarketData.loadOhlcvCsv(args[0]) : MarketData.bundledCandles("btcusdt-1d.csv");

        try (BollingerBands bollinger = new BollingerBands(20, 2.0);
             Atr atr = new Atr(14)) {

            boolean inPosition = false;
            double entryPrice = 0.0;
            double stopLevel = 0.0;
            List<Double> closedTrades = new ArrayList<>();
            double equity = 1.0;
            List<Double> equityCurve = new ArrayList<>();
            Deque<Double> bwWindow = new ArrayDeque<>();

            for (Bar b : bars) {
                BollingerOutput band = bollinger.update(b.close());
                double atrValue = atr.update(b.open(), b.high(), b.low(), b.close(), b.volume(), b.timestamp());
                double price = b.close();
                equityCurve.add(inPosition ? equity * (price / entryPrice) : equity);

                if (band == null || !Double.isFinite(atrValue)) {
                    continue;
                }
                double middle = band.middle();
                if (Math.abs(middle) <= 1e-12) {
                    continue;
                }
                double upper = band.upper();
                double bandwidth = (upper - band.lower()) / middle;
                bwWindow.addLast(bandwidth);
                if (bwWindow.size() > SQUEEZE_LOOKBACK) {
                    bwWindow.removeFirst();
                }
                if (bwWindow.size() < SQUEEZE_LOOKBACK) {
                    continue;
                }
                double minBw = Double.POSITIVE_INFINITY;
                for (double v : bwWindow) {
                    if (v < minBw) {
                        minBw = v;
                    }
                }

                if (inPosition) {
                    if (price < stopLevel || upper < entryPrice) {
                        double tradeRet = price / entryPrice - 1.0;
                        closedTrades.add(tradeRet);
                        equity *= (1.0 + tradeRet) * (1.0 - FEE);
                        inPosition = false;
                    }
                } else {
                    boolean isNewLow = Math.abs(bandwidth - minBw) < 1e-12;
                    if (isNewLow && price > upper) {
                        entryPrice = price;
                        stopLevel = price - ATR_STOP_MULT * atrValue;
                        equity *= 1.0 - FEE;
                        inPosition = true;
                    }
                }
            }

            if (inPosition) {
                double tradeRet = bars[bars.length - 1].close() / entryPrice - 1.0;
                closedTrades.add(tradeRet);
                equity *= (1.0 + tradeRet) * (1.0 - FEE);
            }

            Equity.printSummary("Bollinger Squeeze Breakout (1d, BTCUSDT)",
                    bars[0].close(), bars[bars.length - 1].close(), bars.length, closedTrades, equity, equityCurve);
        }
    }
}
