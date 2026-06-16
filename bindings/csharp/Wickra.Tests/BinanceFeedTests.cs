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

    // The REST fetcher's parse/HTTP success path is covered by the Rust
    // mock-HTTP-server tests; here we only assert the binding's error paths.
    [Fact]
    public void FetchKlinesRejectsUnknownInterval()
    {
        Assert.Throws<ArgumentException>(() =>
            BinanceFeed.FetchKlines("BTCUSDT", (BinanceInterval)99, 1));
    }

    [Fact]
    public void FetchKlinesRejectsZeroLimit()
    {
        Assert.Throws<ArgumentException>(() =>
            BinanceFeed.FetchKlines("BTCUSDT", BinanceInterval.OneHour, 0));
    }

    [Fact]
    public void FetchKlinesSurfacesUnreachableEndpoint()
    {
        Assert.Throws<ArgumentException>(() =>
            BinanceFeed.FetchKlines("BTCUSDT", BinanceInterval.OneHour, 1, baseUrl: "http://127.0.0.1:1"));
    }
}
