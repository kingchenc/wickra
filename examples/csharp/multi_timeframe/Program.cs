using Wickra;
using Wickra.Examples;

// Resample a 1-minute series into higher timeframes and run an indicator per timeframe.
var oneMinute = MarketData.SyntheticCandles(1200, startTimestamp: 0, stepMs: 60_000);

Console.WriteLine("EMA(20) of close across timeframes (resampled from 1-minute bars):");
foreach (var factor in new[] { 1, 5, 15 })
{
    var bars = Resample(oneMinute, factor);
    using var ema = new Ema(20);
    double last = 0;
    foreach (var b in bars)
    {
        last = ema.Update(b.Close);
    }

    Console.WriteLine($"  {factor,2}m: {bars.Length,5} bars  EMA(20) last = {last:F4}");
}

static Bar[] Resample(Bar[] source, int factor)
{
    if (factor <= 1)
    {
        return source;
    }

    // Native Resampler: bucket by an absolute timeframe (the synthetic bars step
    // 60_000 ms, so factor minutes == factor*60_000 ms). No hand-written bucketing.
    // Push returns the candles that bar closed — normally none or one, but with
    // gap filling on it would be one per skipped bucket, so always iterate.
    using var r = new Resampler((long)factor * 60_000, gapFill: false);
    var output = new List<Bar>();
    foreach (var b in source)
    {
        foreach (var c in r.Push(b.Open, b.High, b.Low, b.Close, b.Volume, b.Timestamp))
        {
            output.Add(ToBar(c));
        }
    }

    var last = r.Flush();
    if (last is not null)
    {
        output.Add(ToBar(last.Value));
    }

    return output.ToArray();
}

static Bar ToBar(Candle c) => new(c.Open, c.High, c.Low, c.Close, c.Volume, (long)c.Timestamp);
