using System;
using Wickra;
using Xunit;

// The live Binance feed's connect → read → reconnect pipeline is covered
// deterministically by the Rust mock-WS-server tests in wickra-data. Here we
// only assert the binding's error paths, which need no network.
public class BinanceFeedTests
{
    [Fact]
    public void RejectsUnknownInterval()
    {
        Assert.Throws<ArgumentException>(() =>
            new BinanceFeed("BTCUSDT", (BinanceInterval)99));
    }

    [Fact]
    public void RejectsEmptySymbols()
    {
        Assert.Throws<ArgumentException>(() =>
            new BinanceFeed("", BinanceInterval.OneMinute));
    }

    [Fact]
    public void RejectsUnreachableEndpoint()
    {
        Assert.Throws<ArgumentException>(() =>
            new BinanceFeed("BTCUSDT", BinanceInterval.OneMinute, "ws://127.0.0.1:1"));
    }
}
