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
        // Native CandleReader: header validation, BOM and field-whitespace tolerance.
        // No manual CSV parsing.
        using var reader = new Wickra.CandleReader(File.ReadAllText(path));
        var candles = reader.Read();
        var bars = new Bar[candles.Length];
        for (var i = 0; i < candles.Length; i++)
        {
            var c = candles[i];
            bars[i] = new Bar(c.Open, c.High, c.Low, c.Close, c.Volume, (long)c.Timestamp);
        }

        return bars;
    }
}
