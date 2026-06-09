package org.wickra.examples;

import org.wickra.Atr;
import org.wickra.Ema;
import org.wickra.Rsi;
import org.wickra.Sma;
import org.wickra.examples.MarketData.Bar;

/**
 * Compute a basket of indicators over an OHLCV series and print a summary.
 * Pass a CSV path (timestamp,open,high,low,close,volume) or run on synthetic data.
 */
public final class Backtest {
    public static void main(String[] args) throws Exception {
        String source = args.length > 0 ? args[0] : "synthetic";
        Bar[] bars = args.length > 0 ? MarketData.loadOhlcvCsv(args[0]) : MarketData.syntheticCandles(1000);

        System.out.printf("Backtest over %d bars (%s):%n", bars.length, source);

        try (Sma sma = new Sma(20);
             Ema ema = new Ema(50);
             Rsi rsi = new Rsi(14);
             Atr atr = new Atr(14)) {

            double lastSma = 0, lastEma = 0, lastRsi = 0, lastAtr = 0;
            int oversold = 0;
            for (Bar b : bars) {
                lastSma = sma.update(b.close());
                lastEma = ema.update(b.close());
                lastRsi = rsi.update(b.close());
                lastAtr = atr.update(b.open(), b.high(), b.low(), b.close(), b.volume(), b.timestamp());
                if (Double.isFinite(lastRsi) && lastRsi < 30.0) {
                    oversold++;
                }
            }

            System.out.printf("  SMA(20) last = %.4f%n", lastSma);
            System.out.printf("  EMA(50) last = %.4f%n", lastEma);
            System.out.printf("  RSI(14) last = %.4f  (%d oversold bars)%n", lastRsi, oversold);
            System.out.printf("  ATR(14) last = %.4f%n", lastAtr);
        }
    }
}
