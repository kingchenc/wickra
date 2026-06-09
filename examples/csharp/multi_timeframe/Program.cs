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

    var output = new List<Bar>();
    for (var i = 0; i < source.Length; i += factor)
    {
        var end = Math.Min(i + factor, source.Length);
        double high = double.MinValue, low = double.MaxValue, volume = 0;
        for (var j = i; j < end; j++)
        {
            high = Math.Max(high, source[j].High);
            low = Math.Min(low, source[j].Low);
            volume += source[j].Volume;
        }

        output.Add(new Bar(source[i].Open, high, low, source[end - 1].Close, volume, source[i].Timestamp));
    }

    return output.ToArray();
}
