namespace Wickra.Examples;

/// <summary>Summary statistics for a long-only equity curve.</summary>
public sealed record EquityResult(double TotalReturnPct, double Sharpe, double MaxDrawdownPct, int Trades, double FinalEquity);

/// <summary>
/// Minimal long-only backtest helper: turn a stream of per-bar fractional
/// returns into a PnL / Sharpe / max-drawdown summary. The strategy examples
/// produce the returns; this aggregates them.
/// </summary>
public static class Backtest
{
    /// <param name="periodReturns">Per-bar fractional returns (0.01 == +1%).</param>
    /// <param name="trades">Number of position entries.</param>
    /// <param name="periodsPerYear">Annualisation factor for the Sharpe ratio.</param>
    public static EquityResult Summarize(IReadOnlyList<double> periodReturns, int trades, double periodsPerYear = 252.0)
    {
        double equity = 1.0, peak = 1.0, maxDrawdown = 0.0;
        foreach (var r in periodReturns)
        {
            equity *= 1.0 + r;
            peak = Math.Max(peak, equity);
            if (peak > 0)
            {
                maxDrawdown = Math.Max(maxDrawdown, (peak - equity) / peak);
            }
        }

        var mean = periodReturns.Count > 0 ? periodReturns.Average() : 0.0;
        var variance = periodReturns.Count > 1
            ? periodReturns.Sum(x => (x - mean) * (x - mean)) / (periodReturns.Count - 1)
            : 0.0;
        var stdDev = Math.Sqrt(variance);
        var sharpe = stdDev > 1e-12 ? mean / stdDev * Math.Sqrt(periodsPerYear) : 0.0;

        return new EquityResult((equity - 1.0) * 100.0, sharpe, maxDrawdown * 100.0, trades, equity);
    }

    /// <summary>Prints a one-line summary.</summary>
    public static void Print(string name, EquityResult r)
    {
        Console.WriteLine(
            $"{name,-26} return={r.TotalReturnPct,8:F2}%  sharpe={r.Sharpe,6:F2}  maxDD={r.MaxDrawdownPct,6:F2}%  trades={r.Trades}");
    }

    /// <summary>
    /// Prints the per-trade backtest summary shared verbatim with the Rust,
    /// Python, Node, Go and C example suites (same labels, same numbers).
    /// </summary>
    public static void PrintSummary(string name, double firstPrice, double lastPrice, int bars,
        IReadOnlyList<double> closedTrades, double finalEquity, IReadOnlyList<double> equityCurve)
    {
        var ci = System.Globalization.CultureInfo.InvariantCulture;
        var buyHold = lastPrice / firstPrice;
        var stratReturn = finalEquity - 1.0;
        var bhReturn = buyHold - 1.0;
        int wins = 0, losses = 0;
        double best = 0.0, worst = 0.0;
        for (var i = 0; i < closedTrades.Count; i++)
        {
            var r = closedTrades[i];
            if (r > 0)
            {
                wins++;
            }
            else if (r < 0)
            {
                losses++;
            }

            if (i == 0 || r > best)
            {
                best = r;
            }

            if (i == 0 || r < worst)
            {
                worst = r;
            }
        }

        var n = closedTrades.Count;
        var mean = n > 0 ? closedTrades.Average() : 0.0;
        var variance = n > 1 ? closedTrades.Sum(x => (x - mean) * (x - mean)) / (n - 1) : 0.0;
        var sharpe = variance > 0 ? mean / Math.Sqrt(variance) : 0.0;
        var peak = equityCurve.Count > 0 ? equityCurve[0] : 1.0;
        var maxDd = 0.0;
        foreach (var eq in equityCurve)
        {
            if (eq > peak)
            {
                peak = eq;
            }

            var dd = (peak - eq) / peak;
            if (dd > maxDd)
            {
                maxDd = dd;
            }
        }

        Console.WriteLine($"=== {name} ===");
        Console.WriteLine(string.Create(ci, $"{"Bars:",-23}{bars}"));
        Console.WriteLine(string.Create(ci, $"{"Trades:",-23}{n} (W{wins} / L{losses})"));
        Console.WriteLine(string.Create(ci, $"{"Strategy return:",-23}{stratReturn * 100:+0.00;-0.00}%"));
        Console.WriteLine(string.Create(ci, $"{"Buy & Hold return:",-23}{bhReturn * 100:+0.00;-0.00}%"));
        Console.WriteLine(string.Create(ci, $"{"Excess over BH:",-23}{(stratReturn - bhReturn) * 100:+0.00;-0.00}%"));
        Console.WriteLine(string.Create(ci, $"{"Max drawdown:",-23}{maxDd * 100:0.00}%"));
        Console.WriteLine(string.Create(ci, $"{"Per-trade Sharpe:",-23}{sharpe:0.00}  (mean {mean:+0.0000;-0.0000}, stddev {Math.Sqrt(variance):0.0000})"));
        Console.WriteLine(string.Create(ci, $"{"Best / worst trade:",-23}{best * 100:+0.00;-0.00}% / {worst * 100:+0.00;-0.00}%"));
        Console.WriteLine();
        Console.WriteLine("NOTE: Educational example — fees, slippage, funding costs and tax " +
            "effects are simplified or omitted. Past performance is not " +
            "indicative of future results.");
    }
}
