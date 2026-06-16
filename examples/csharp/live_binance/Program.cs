using Wickra;

// Stream live BTCUSDT 1-minute klines from Binance and feed each close through EMA(20),
// using Wickra's native BinanceFeed — no third-party WebSocket or JSON library.
// Requires network access (build-only in CI). Runs for up to 60 seconds.
Console.WriteLine("Streaming live BTCUSDT 1-minute klines from Binance (up to 60s)...");

// Native feed: a blocking poll over the same tested stream as the Rust core.
// Next(timeout) returns the event, or null on timeout (poll again).
using var feed = new BinanceFeed("BTCUSDT", BinanceInterval.OneMinute);
using var ema = new Ema(20);

var deadline = DateTime.UtcNow.AddSeconds(60);
while (DateTime.UtcNow < deadline)
{
    var ev = feed.Next(TimeSpan.FromSeconds(1));
    if (ev is null)
    {
        continue;
    }
    Console.WriteLine($"close={ev.Value.Close:F2}  EMA(20)={ema.Update(ev.Value.Close):F2}");
}

Console.WriteLine("Done (time limit reached).");
