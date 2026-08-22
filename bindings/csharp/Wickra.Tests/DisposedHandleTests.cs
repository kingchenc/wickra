using Xunit;

namespace Wickra.Tests;

/// <summary>
/// Every indicator method used to reach the native library through
/// <c>_handle.DangerousGetHandle()</c>, which keeps returning the pointer after
/// <see cref="System.IDisposable.Dispose"/> has already freed it.
/// <c>GC.KeepAlive</c> guarded only against the collector, not against an
/// explicit dispose, so calling any method on a disposed indicator read freed
/// memory — silently wrong numbers on a lucky heap layout, an access violation
/// otherwise, with a Rust panic unwinding across <c>extern "C"</c> in between.
///
/// The methods now take the <c>SafeHandle</c> itself, so the marshaller
/// ref-counts it and raises <see cref="ObjectDisposedException"/> instead.
/// </summary>
public class DisposedHandleTests
{
    [Fact]
    public void UpdateAfterDisposeThrows()
    {
        var sma = new Sma(3);
        sma.Update(1);
        sma.Update(2);
        Assert.Equal(2, sma.Update(3));
        sma.Dispose();

        Assert.Throws<ObjectDisposedException>(() => sma.Update(4));
    }

    [Fact]
    public void ReadOnlyAccessorsAfterDisposeThrow()
    {
        var sma = new Sma(3);
        sma.Dispose();

        Assert.Throws<ObjectDisposedException>(() => sma.IsReady());
        Assert.Throws<ObjectDisposedException>(() => sma.WarmupPeriod());
        Assert.Throws<ObjectDisposedException>(() => sma.Name());
    }

    [Fact]
    public void ResetAfterDisposeThrows()
    {
        var sma = new Sma(3);
        sma.Dispose();

        Assert.Throws<ObjectDisposedException>(sma.Reset);
    }

    [Fact]
    public void BatchAfterDisposeThrows()
    {
        var sma = new Sma(3);
        sma.Dispose();

        Assert.Throws<ObjectDisposedException>(() => sma.Batch(new double[] { 1, 2, 3, 4 }));
    }

    [Fact]
    public void CandleIndicatorAfterDisposeThrows()
    {
        var atr = new Atr(3);
        atr.Update(100, 101, 99, 100, 10, 0);
        atr.Dispose();

        Assert.Throws<ObjectDisposedException>(() => atr.Update(101, 102, 100, 101, 10, 1));
    }

    [Fact]
    public void MultiOutputIndicatorAfterDisposeThrows()
    {
        var bb = new BollingerBands(3, 2.0);
        bb.Dispose();

        Assert.Throws<ObjectDisposedException>(() => bb.Update(1));
    }

    [Fact]
    public void DisposeIsIdempotent()
    {
        var sma = new Sma(3);
        sma.Dispose();
        sma.Dispose();
    }

    [Fact]
    public void AnUndisposedIndicatorIsUnaffected()
    {
        // The guard must not disturb the normal path.
        using var sma = new Sma(3);
        sma.Update(1);
        sma.Update(2);
        Assert.Equal(2, sma.Update(3));
        Assert.True(sma.IsReady());
        Assert.Equal(3, sma.WarmupPeriod());
        Assert.Equal("SMA", sma.Name());
        sma.Reset();
        Assert.False(sma.IsReady());
    }
}
