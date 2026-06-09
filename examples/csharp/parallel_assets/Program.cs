using System.Diagnostics;
using Wickra;
using Wickra.Examples;

// Run SMA(20) batch over a panel of assets, serial vs Parallel.For, and report the speedup.
var assets = args.Length > 0 ? int.Parse(args[0]) : 500;
var bars = args.Length > 1 ? int.Parse(args[1]) : 20_000;

var panel = new double[assets][];
for (var a = 0; a < assets; a++)
{
    panel[a] = MarketData.SyntheticPrices(bars, start: 50.0 + a * 0.1);
}

// Warm up the JIT and thread pool so the comparison is fair.
using (var warm = new Sma(20))
{
    warm.Batch(panel[0]);
}

var sink = 0.0;
var sw = Stopwatch.StartNew();
for (var a = 0; a < assets; a++)
{
    using var sma = new Sma(20);
    var result = sma.Batch(panel[a]);
    sink += result[^1];
}

sw.Stop();
var serialMs = sw.Elapsed.TotalMilliseconds;

var lasts = new double[assets];
sw.Restart();
Parallel.For(0, assets, a =>
{
    using var sma = new Sma(20);
    var result = sma.Batch(panel[a]);
    lasts[a] = result[^1];
});
sw.Stop();
var parallelMs = sw.Elapsed.TotalMilliseconds;

Console.WriteLine($"{assets} assets x {bars} bars, SMA(20) batch:");
Console.WriteLine($"  serial   {serialMs,8:F1} ms");
Console.WriteLine($"  parallel {parallelMs,8:F1} ms  ({serialMs / Math.Max(parallelMs, 1e-9):F1}x speedup)");
GC.KeepAlive(sink);
