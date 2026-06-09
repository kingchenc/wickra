using Wickra;
using Xunit;

namespace Wickra.Tests;

public class SmaTests
{
    [Fact]
    public void StreamingMatchesReference()
    {
        using var sma = new Sma(3);
        Assert.True(double.IsNaN(sma.Update(1)));
        Assert.True(double.IsNaN(sma.Update(2)));
        Assert.Equal(2.0, sma.Update(3), 9);
        Assert.Equal(3.0, sma.Update(4), 9);
        Assert.Equal(4.0, sma.Update(5), 9);
    }

    [Fact]
    public void BatchMatchesStreaming()
    {
        using var sma = new Sma(3);
        var output = sma.Batch(new double[] { 1, 2, 3, 4, 5 });

        Assert.True(double.IsNaN(output[0]));
        Assert.True(double.IsNaN(output[1]));
        Assert.Equal(2.0, output[2], 9);
        Assert.Equal(3.0, output[3], 9);
        Assert.Equal(4.0, output[4], 9);
    }

    [Fact]
    public void ResetClearsState()
    {
        using var sma = new Sma(3);
        sma.Update(1);
        sma.Update(2);
        sma.Update(3);
        sma.Reset();
        Assert.True(double.IsNaN(sma.Update(10)));
    }

    [Fact]
    public void ZeroPeriodThrows()
    {
        // Zero is rejected by the native constructor (returns NULL) -> ArgumentException;
        // a negative period is caught earlier by the wrapper guard.
        Assert.Throws<ArgumentException>(() => new Sma(0));
        Assert.Throws<ArgumentOutOfRangeException>(() => new Sma(-1));
    }
}
