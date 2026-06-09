package org.wickra.examples;

import java.util.List;

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
}
