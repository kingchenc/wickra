using Xunit;

namespace Wickra.Tests;

/// <summary>
/// The cross-section, order-book, bar-builder and profile families take an input
/// shape the batch generator could not express, so 39 indicators had a native
/// batch the wrapper could not reach. Each shape is checked against feeding the
/// same data one bar at a time.
/// </summary>
public class BatchShapesTests
{
    private const int Bars = 8;

    private static void AssertClose(double got, double want, string label)
    {
        if (double.IsNaN(want))
        {
            Assert.True(double.IsNaN(got), $"{label}: want NaN, got {got}");
            return;
        }

        Assert.True(
            System.Math.Abs(got - want) <= 1e-12 * System.Math.Max(1, System.Math.Abs(want)),
            $"{label}: batch {got}, streaming {want}");
    }

    [Fact]
    public void CrossSectionBatchMatchesStreaming()
    {
        const int members = 3;
        var change = new double[Bars * members];
        var volume = new double[Bars * members];
        var newHigh = new bool[Bars * members];
        var newLow = new bool[Bars * members];
        var aboveMa = new bool[Bars * members];
        var onBuy = new bool[Bars * members];
        var stamps = new long[Bars];
        for (var bar = 0; bar < Bars; bar++)
        {
            stamps[bar] = bar;
            for (var m = 0; m < members; m++)
            {
                var at = bar * members + m;
                change[at] = at * 0.37 - 2;
                volume[at] = 100 + at * 5;
                newHigh[at] = (bar + m) % 2 == 0;
                newLow[at] = (bar + m) % 3 == 0;
                aboveMa[at] = m % 2 == 0;
                onBuy[at] = bar % 2 == 0;
            }
        }

        using var batched = new AdvanceDecline();
        var got = batched.Batch(change, volume, newHigh, newLow, aboveMa, onBuy, members, stamps);

        using var streamed = new AdvanceDecline();
        for (var bar = 0; bar < Bars; bar++)
        {
            var lo = bar * members;
            var want = streamed.Update(
                change.AsSpan(lo, members), volume.AsSpan(lo, members),
                newHigh.AsSpan(lo, members), newLow.AsSpan(lo, members),
                aboveMa.AsSpan(lo, members), onBuy.AsSpan(lo, members), stamps[bar]);
            AssertClose(got[bar], want, $"bar {bar}");
        }
    }

    [Fact]
    public void OrderBookBatchMatchesStreaming()
    {
        const int depth = 2;
        var bidPx = new double[Bars * depth];
        var bidSz = new double[Bars * depth];
        var askPx = new double[Bars * depth];
        var askSz = new double[Bars * depth];
        for (var bar = 0; bar < Bars; bar++)
        {
            for (var lvl = 0; lvl < depth; lvl++)
            {
                var at = bar * depth + lvl;
                double drift = bar * 0.25, step = lvl * 0.1;
                bidPx[at] = 100 + drift - step;
                bidSz[at] = 5 + step;
                askPx[at] = 100.2 + drift + step;
                askSz[at] = 4 + step;
            }
        }

        using var batched = new Microprice();
        var got = batched.Batch(bidPx, bidSz, depth, askPx, askSz, depth);

        using var streamed = new Microprice();
        for (var bar = 0; bar < Bars; bar++)
        {
            var lo = bar * depth;
            var want = streamed.Update(
                bidPx.AsSpan(lo, depth), bidSz.AsSpan(lo, depth),
                askPx.AsSpan(lo, depth), askSz.AsSpan(lo, depth));
            AssertClose(got[bar], want, $"bar {bar}");
        }
    }

    [Fact]
    public void BarBuilderBatchMatchesStreaming()
    {
        const int n = 12;
        var closes = new double[n];
        var vols = new double[n];
        var stamps = new long[n];
        for (var i = 0; i < n; i++)
        {
            closes[i] = 100 + i * 3 + (i == 6 ? 40 : 0); // a gap completes several bricks
            vols[i] = 1;
            stamps[i] = i;
        }

        using var batched = new RenkoBars(1.0);
        var got = batched.Batch(closes, closes, closes, closes, vols, stamps);

        using var streamed = new RenkoBars(1.0);
        var want = new System.Collections.Generic.List<RenkoBrick>();
        for (var i = 0; i < n; i++)
        {
            want.AddRange(streamed.Update(closes[i], closes[i], closes[i], closes[i], vols[i], stamps[i]));
        }

        Assert.Equal(want.Count, got.Length);
        for (var i = 0; i < want.Count; i++)
        {
            Assert.Equal(want[i], got[i]);
        }
    }

    [Fact]
    public void ProfileBatchMatchesStreaming()
    {
        const int n = 10;
        var closes = new double[n];
        var vols = new double[n];
        var stamps = new long[n];
        for (var i = 0; i < n; i++)
        {
            closes[i] = 100 + i;
            vols[i] = 10;
            stamps[i] = (long)i * 86_400_000; // one day apart
        }

        using var batched = new DayOfWeekProfile(0);
        var got = batched.Batch(closes, closes, closes, closes, vols, stamps);

        using var streamed = new DayOfWeekProfile(0);
        var emitted = 0;
        for (var i = 0; i < n; i++)
        {
            var want = streamed.Update(closes[i], closes[i], closes[i], closes[i], vols[i], stamps[i]);
            if (want is null)
            {
                Assert.All(got[i], v => Assert.True(double.IsNaN(v), "warmup rows must be NaN"));
                continue;
            }

            emitted++;
            for (var k = 0; k < want.Length; k++)
            {
                AssertClose(got[i][k], want[k], $"row {i} bucket {k}");
            }
        }

        Assert.True(emitted > 0, "the fixture must clear warmup");
    }
}
