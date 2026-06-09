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
}
