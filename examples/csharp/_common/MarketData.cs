namespace Wickra.Examples;

/// <summary>One OHLCV bar with a millisecond timestamp.</summary>
public readonly record struct Bar(double Open, double High, double Low, double Close, double Volume, long Timestamp);

/// <summary>
/// Deterministic synthetic market data plus a small OHLCV CSV loader, shared by
/// the offline examples so they run without network access.
/// </summary>
public static class MarketData
{
    /// <summary>A reproducible price path (trend + two cycles), no randomness.</summary>
    public static double[] SyntheticPrices(int count, double start = 100.0)
    {
        var prices = new double[count];
        for (var i = 0; i < count; i++)
        {
            prices[i] = start + 12.0 * Math.Sin(i * 0.05) + 5.0 * Math.Sin(i * 0.013) + i * 0.01;
        }

        return prices;
    }

    /// <summary>A reproducible OHLCV series derived from <see cref="SyntheticPrices"/>.</summary>
    public static Bar[] SyntheticCandles(int count, long startTimestamp = 0, long stepMs = 3_600_000)
    {
        var prices = SyntheticPrices(count + 1);
        var bars = new Bar[count];
        for (var i = 0; i < count; i++)
        {
            var open = prices[i];
            var close = prices[i + 1];
            var high = Math.Max(open, close) + 0.5 + Math.Abs(Math.Sin(i * 0.7));
            var low = Math.Min(open, close) - 0.5 - Math.Abs(Math.Cos(i * 0.7));
            var volume = 1_000.0 + 500.0 * (1.0 + Math.Sin(i * 0.1));
            bars[i] = new Bar(open, high, low, close, volume, startTimestamp + i * stepMs);
        }

        return bars;
    }

    /// <summary>
    /// Loads an OHLCV CSV. Accepts rows of <c>timestamp,open,high,low,close,volume</c>
    /// or <c>open,high,low,close,volume</c>; a non-numeric first row is treated as a header.
    /// </summary>
    public static Bar[] LoadOhlcvCsv(string path)
    {
        var bars = new List<Bar>();
        foreach (var rawLine in File.ReadLines(path))
        {
            var line = rawLine.Trim();
            if (line.Length == 0)
            {
                continue;
            }

            var cols = line.Split(',');
            if (!double.TryParse(cols[0], System.Globalization.CultureInfo.InvariantCulture, out _) &&
                !long.TryParse(cols[0], out _))
            {
                continue; // header row
            }

            double F(int i) => double.Parse(cols[i], System.Globalization.CultureInfo.InvariantCulture);

            if (cols.Length >= 6)
            {
                bars.Add(new Bar(F(1), F(2), F(3), F(4), F(5), long.Parse(cols[0])));
            }
            else
            {
                bars.Add(new Bar(F(0), F(1), F(2), F(3), F(4), bars.Count));
            }
        }

        return bars.ToArray();
    }
}
