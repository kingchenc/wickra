using Wickra;
using Xunit;

namespace Wickra.Tests;

/// <summary>
/// One representative per FFI archetype, exercising every marshalling path the
/// generator produces (scalar, candle, pairwise, multi-output, bars, profile,
/// values-profile, array-input). Garbage marshalling surfaces as NaN, wild
/// values, or crashes — so finite/sane assertions are the real check.
/// </summary>
public class ArchetypeTests
{
    private static (double open, double high, double low, double close, double volume, long ts) Candle(int i)
    {
        var close = 100.0 + 10.0 * Math.Sin(i * 0.3);
        var open = 100.0 + 10.0 * Math.Sin((i - 1) * 0.3);
        var high = Math.Max(open, close) + 1.0;
        var low = Math.Min(open, close) - 1.0;
        return (open, high, low, close, 1_000.0, i * 60_000L);
    }

    [Fact]
    public void Scalar_Ema_IsFiniteAfterWarmup()
    {
        using var ema = new Ema(3);
        double last = double.NaN;
        for (var i = 1; i <= 10; i++)
        {
            last = ema.Update(i);
        }

        Assert.True(double.IsFinite(last));
        Assert.InRange(last, 1.0, 10.0);
    }

    [Fact]
    public void Candle_Atr_IsFinitePositive()
    {
        using var atr = new Atr(3);
        double last = double.NaN;
        for (var i = 0; i < 20; i++)
        {
            var (o, h, l, c, v, ts) = Candle(i);
            last = atr.Update(o, h, l, c, v, ts);
        }

        Assert.True(double.IsFinite(last));
        Assert.True(last > 0.0);
    }

    [Fact]
    public void Pairwise_Beta_IsFinite()
    {
        using var beta = new Beta(5);
        double last = double.NaN;
        for (var i = 0; i < 30; i++)
        {
            var market = 100.0 + 10.0 * Math.Sin(i * 0.5);
            var asset = 50.0 + 6.0 * Math.Sin(i * 0.5 + 0.2);
            last = beta.Update(market, asset);
        }

        Assert.True(double.IsFinite(last));
    }

    [Fact]
    public void MultiOutput_Adx_ReturnsFiniteStruct()
    {
        using var adx = new Adx(5);
        AdxOutput? result = null;
        for (var i = 0; i < 60; i++)
        {
            var (o, h, l, c, v, ts) = Candle(i);
            result = adx.Update(o, h, l, c, v, ts);
        }

        Assert.NotNull(result);
        Assert.True(double.IsFinite(result!.Value.Adx));
        Assert.True(double.IsFinite(result.Value.PlusDi));
        Assert.True(double.IsFinite(result.Value.MinusDi));
    }

    [Fact]
    public void Bars_DollarBars_EmitsBars()
    {
        using var bars = new DollarBars(5_000.0);
        var total = 0;
        for (var i = 0; i < 200; i++)
        {
            var (o, h, l, c, v, ts) = Candle(i);
            total += bars.Update(o, h, l, c, v, ts).Length;
        }

        Assert.True(total > 0);
    }

    [Fact]
    public void Profile_VolumeProfile_ReturnsValues()
    {
        using var profile = new VolumeProfile(20, 8);
        VolumeProfileOutputScalars? result = null;
        for (var i = 0; i < 60; i++)
        {
            var (o, h, l, c, v, ts) = Candle(i);
            result = profile.Update(o, h, l, c, v, ts);
        }

        Assert.NotNull(result);
        Assert.NotNull(result!.Value.Values);
        Assert.True(result.Value.PriceLow <= result.Value.PriceHigh);
    }

    [Fact]
    public void ProfileValues_DayOfWeekProfile_NoCrash()
    {
        using var profile = new DayOfWeekProfile(0);
        double[]? result = null;
        for (var i = 0; i < 60; i++)
        {
            var close = 100.0 + 5.0 * Math.Sin(i * 0.2);
            // one day apart so the day-of-week buckets fill
            result = profile.Update(close, close + 1, close - 1, close, 1_000.0, i * 86_400_000L);
        }

        if (result is not null)
        {
            Assert.All(result, v => Assert.True(double.IsFinite(v)));
        }
    }

    [Fact]
    public void ArrayInput_DepthSlope_IsFinite()
    {
        using var slope = new DepthSlope();
        ReadOnlySpan<double> bidPrice = stackalloc double[] { 99.0, 98.0, 97.0 };
        ReadOnlySpan<double> bidSize = stackalloc double[] { 10.0, 20.0, 30.0 };
        ReadOnlySpan<double> askPrice = stackalloc double[] { 101.0, 102.0, 103.0 };
        ReadOnlySpan<double> askSize = stackalloc double[] { 12.0, 22.0, 32.0 };

        var result = slope.Update(bidPrice, bidSize, askPrice, askSize);
        Assert.True(double.IsFinite(result));
    }
}
