package org.wickra.examples;

import org.wickra.Ema;
import org.wickra.MacdIndicator;
import org.wickra.MacdOutput;
import org.wickra.Rsi;
import org.wickra.Sma;

/** Feed a synthetic price series through several indicators tick by tick (O(1) each). */
public final class Streaming {
    public static void main(String[] args) {
        double[] prices = MarketData.syntheticPrices(500);

        try (Sma sma = new Sma(20);
             Ema ema = new Ema(20);
             Rsi rsi = new Rsi(14);
             MacdIndicator macd = new MacdIndicator(12, 26, 9)) {

            double lastSma = 0, lastEma = 0, lastRsi = 0;
            MacdOutput lastMacd = null;
            for (double price : prices) {
                lastSma = sma.update(price);
                lastEma = ema.update(price);
                lastRsi = rsi.update(price);
                lastMacd = macd.update(price);
            }

            System.out.printf("Streamed %d prices through SMA(20), EMA(20), RSI(14), MACD(12,26,9):%n", prices.length);
            System.out.printf("  SMA  = %.4f%n", lastSma);
            System.out.printf("  EMA  = %.4f%n", lastEma);
            System.out.printf("  RSI  = %.4f%n", lastRsi);
            if (lastMacd != null) {
                System.out.printf("  MACD = %.4f  signal=%.4f  hist=%.4f%n",
                        lastMacd.macd(), lastMacd.signal(), lastMacd.histogram());
            }
        }
    }
}
