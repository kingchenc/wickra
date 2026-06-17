package org.wickra.examples;

import java.util.List;
import java.util.Locale;

/**
 * Minimal long-only backtest helper: turn a stream of per-bar fractional returns
 * into a PnL / Sharpe / max-drawdown summary. The strategy examples produce the
 * returns; this aggregates them.
 */
public final class Equity {
    private Equity() {
    }

    /** Summary statistics for a long-only equity curve. */
    public record Result(double totalReturnPct, double sharpe, double maxDrawdownPct, int trades, double finalEquity) {
    }

    /**
     * @param periodReturns per-bar fractional returns (0.01 == +1%)
     * @param trades        number of position entries
     */
    public static Result summarize(List<Double> periodReturns, int trades) {
        return summarize(periodReturns, trades, 252.0);
    }

    public static Result summarize(List<Double> periodReturns, int trades, double periodsPerYear) {
        double equity = 1.0;
        double peak = 1.0;
        double maxDrawdown = 0.0;
        for (double r : periodReturns) {
            equity *= 1.0 + r;
            peak = Math.max(peak, equity);
            if (peak > 0) {
                maxDrawdown = Math.max(maxDrawdown, (peak - equity) / peak);
            }
        }

        double mean = 0.0;
        for (double r : periodReturns) {
            mean += r;
        }
        mean = periodReturns.isEmpty() ? 0.0 : mean / periodReturns.size();

        double variance = 0.0;
        if (periodReturns.size() > 1) {
            for (double r : periodReturns) {
                variance += (r - mean) * (r - mean);
            }
            variance /= periodReturns.size() - 1;
        }
        double stdDev = Math.sqrt(variance);
        double sharpe = stdDev > 1e-12 ? mean / stdDev * Math.sqrt(periodsPerYear) : 0.0;

        return new Result((equity - 1.0) * 100.0, sharpe, maxDrawdown * 100.0, trades, equity);
    }

    /** Prints a one-line summary. */
    public static void print(String name, Result r) {
        System.out.printf(
                "%-26s return=%8.2f%%  sharpe=%6.2f  maxDD=%6.2f%%  trades=%d%n",
                name, r.totalReturnPct(), r.sharpe(), r.maxDrawdownPct(), r.trades());
    }

    /**
     * Prints the per-trade backtest summary shared verbatim with the Rust,
     * Python, Node, Go, C, C# and R example suites (same labels, same numbers).
     */
    public static void printSummary(String name, double firstPrice, double lastPrice, int bars,
            List<Double> closedTrades, double finalEquity, List<Double> equityCurve) {
        double buyHold = lastPrice / firstPrice;
        double stratReturn = finalEquity - 1.0;
        double bhReturn = buyHold - 1.0;
        int wins = 0;
        int losses = 0;
        double best = 0.0;
        double worst = 0.0;
        for (int i = 0; i < closedTrades.size(); i++) {
            double r = closedTrades.get(i);
            if (r > 0) {
                wins++;
            } else if (r < 0) {
                losses++;
            }
            if (i == 0 || r > best) {
                best = r;
            }
            if (i == 0 || r < worst) {
                worst = r;
            }
        }

        int n = closedTrades.size();
        double mean = 0.0;
        for (double r : closedTrades) {
            mean += r;
        }
        mean = n > 0 ? mean / n : 0.0;
        double variance = 0.0;
        if (n > 1) {
            for (double r : closedTrades) {
                variance += (r - mean) * (r - mean);
            }
            variance /= n - 1;
        }
        double sharpe = variance > 0 ? mean / Math.sqrt(variance) : 0.0;
        double peak = equityCurve.isEmpty() ? 1.0 : equityCurve.get(0);
        double maxDd = 0.0;
        for (double eq : equityCurve) {
            if (eq > peak) {
                peak = eq;
            }
            double dd = (peak - eq) / peak;
            if (dd > maxDd) {
                maxDd = dd;
            }
        }

        System.out.printf(Locale.ROOT, "=== %s ===%n", name);
        System.out.printf(Locale.ROOT, "%-23s%d%n", "Bars:", bars);
        System.out.printf(Locale.ROOT, "%-23s%d (W%d / L%d)%n", "Trades:", n, wins, losses);
        System.out.printf(Locale.ROOT, "%-23s%+.2f%%%n", "Strategy return:", stratReturn * 100);
        System.out.printf(Locale.ROOT, "%-23s%+.2f%%%n", "Buy & Hold return:", bhReturn * 100);
        System.out.printf(Locale.ROOT, "%-23s%+.2f%%%n", "Excess over BH:", (stratReturn - bhReturn) * 100);
        System.out.printf(Locale.ROOT, "%-23s%.2f%%%n", "Max drawdown:", maxDd * 100);
        System.out.printf(Locale.ROOT, "%-23s%.2f  (mean %+.4f, stddev %.4f)%n",
                "Per-trade Sharpe:", sharpe, mean, Math.sqrt(variance));
        System.out.printf(Locale.ROOT, "%-23s%+.2f%% / %+.2f%%%n", "Best / worst trade:", best * 100, worst * 100);
        System.out.println();
        System.out.println("NOTE: Educational example — fees, slippage, funding costs and tax "
                + "effects are simplified or omitted. Past performance is not "
                + "indicative of future results.");
    }
}
