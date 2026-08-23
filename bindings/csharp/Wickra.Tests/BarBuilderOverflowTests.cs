using Xunit;

namespace Wickra.Tests;

/// <summary>
/// One candle can complete far more bars than the fixed buffer <c>Update</c>
/// passes to the native side. A Renko builder with a box size of 1 turns a
/// 500-point move into 500 bricks; the wrapper indexed its 64-element buffer
/// with the returned count and threw <see cref="System.IndexOutOfRangeException"/>
/// until the surplus was drained off the handle.
/// </summary>
public class BarBuilderOverflowTests
{
    [Fact]
    public void UpdateReturnsEveryBrickOfALargeMove()
    {
        using var renko = new RenkoBars(1.0);
        renko.Update(100, 100, 100, 100, 1, 0);

        var bricks = renko.Update(600, 600, 600, 600, 1, 1);

        Assert.True(
            bricks.Length > 64,
            $"the move must complete more bricks than the buffer holds, got {bricks.Length}");
        // One consecutive ladder, so nothing was dropped or duplicated across
        // the boundary between the buffer and the drained remainder.
        for (var i = 0; i < bricks.Length; i++)
        {
            Assert.Equal(1.0, bricks[i].Close - bricks[i].Open, 9);
            if (i > 0)
            {
                Assert.Equal(bricks[i - 1].Close, bricks[i].Open, 9);
            }
        }
    }
}
