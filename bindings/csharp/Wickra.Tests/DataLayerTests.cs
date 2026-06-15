using System;
using System.Collections.Generic;
using System.Globalization;
using System.IO;
using Wickra;
using Xunit;

// Cross-language data-layer parity: replay the shared golden tick stream through
// the TickAggregator and check the candles against the Rust reference, with and
// without gap filling.
public class DataLayerTests
{
    private static string GoldenDir([System.Runtime.CompilerServices.CallerFilePath] string file = "") =>
        Path.GetFullPath(Path.Combine(Path.GetDirectoryName(file)!, "..", "..", "..", "testdata", "golden"));

    private static double[][] Read(string name)
    {
        var lines = File.ReadAllLines(Path.Combine(GoldenDir(), name + ".csv"));
        var rows = new List<double[]>();
        for (var i = 1; i < lines.Length; i++)
        {
            if (lines[i].Length == 0)
            {
                continue;
            }
            rows.Add(Array.ConvertAll(lines[i].Split(','), s => double.Parse(s, CultureInfo.InvariantCulture)));
        }
        return rows.ToArray();
    }

    [Fact]
    public void ResamplerMatchesGolden()
    {
        var input = Read("input"); // open,high,low,close,volume (timestamp = row index)
        using var r = new Resampler(5);
        var got = new List<double[]>();
        for (var i = 0; i < input.Length; i++)
        {
            var c = r.Update(input[i][0], input[i][1], input[i][2], input[i][3], input[i][4], i);
            if (c.HasValue)
            {
                got.Add(new[] { c.Value.Open, c.Value.High, c.Value.Low, c.Value.Close, c.Value.Volume, (double)c.Value.Timestamp });
            }
        }
        var f = r.Flush();
        if (f.HasValue)
        {
            got.Add(new[] { f.Value.Open, f.Value.High, f.Value.Low, f.Value.Close, f.Value.Volume, (double)f.Value.Timestamp });
        }
        var want = Read("data_resampled");
        Assert.Equal(want.Length, got.Count);
        for (var i = 0; i < got.Count; i++)
        {
            for (var j = 0; j < 6; j++)
            {
                var tol = 1e-9 * Math.Max(1, Math.Abs(want[i][j]));
                Assert.True(Math.Abs(got[i][j] - want[i][j]) <= tol, $"resample row {i} col {j}: {got[i][j]} vs {want[i][j]}");
            }
        }
    }

    [Fact]
    public void CandleReaderMatchesGolden()
    {
        var csv = File.ReadAllText(Path.Combine(GoldenDir(), "data_csv.csv"));
        using var reader = new CandleReader(csv);
        var candles = reader.Read();
        var want = Read("data_csv_candles");
        Assert.Equal(want.Length, candles.Length);
        for (var i = 0; i < candles.Length; i++)
        {
            var c = candles[i];
            var got = new[] { c.Open, c.High, c.Low, c.Close, c.Volume, (double)c.Timestamp };
            for (var j = 0; j < 6; j++)
            {
                var tol = 1e-9 * Math.Max(1, Math.Abs(want[i][j]));
                Assert.True(Math.Abs(got[j] - want[i][j]) <= tol, $"candle reader row {i} col {j}: {got[j]} vs {want[i][j]}");
            }
        }
    }

    [Theory]
    [InlineData(false, "data_candles")]
    [InlineData(true, "data_candles_gap")]
    public void TickAggregatorMatchesGolden(bool gapFill, string fixture)
    {
        var ticks = Read("data_ticks");
        using var agg = new TickAggregator(1000, gapFill);
        var got = new List<double[]>();
        foreach (var t in ticks)
        {
            foreach (var c in agg.Push(t[0], t[1], (long)t[2]))
            {
                got.Add(new[] { c.Open, c.High, c.Low, c.Close, c.Volume, (double)c.Timestamp });
            }
        }
        var want = Read(fixture);
        Assert.Equal(want.Length, got.Count);
        for (var i = 0; i < got.Count; i++)
        {
            for (var j = 0; j < 6; j++)
            {
                var tol = 1e-9 * Math.Max(1, Math.Abs(want[i][j]));
                Assert.True(
                    Math.Abs(got[i][j] - want[i][j]) <= tol,
                    $"{fixture} row {i} col {j}: {got[i][j]} vs {want[i][j]}");
            }
        }
    }
}
