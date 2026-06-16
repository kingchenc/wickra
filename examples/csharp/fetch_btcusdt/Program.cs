using System.Globalization;
using Wickra;

// Download real BTCUSDT hourly klines from the Binance REST API into a CSV that the
// other examples can consume, using Wickra's native fetcher — no third-party HTTP or
// JSON library. Requires network access (build-only in CI).
Console.WriteLine("Fetching 500 BTCUSDT 1h klines from Binance...");

var klines = BinanceFeed.FetchKlines("BTCUSDT", BinanceInterval.OneHour, 500);

var dir = Path.Combine(AppContext.BaseDirectory, "data");
Directory.CreateDirectory(dir);
var path = Path.Combine(dir, "btcusdt_1h.csv");

using var writer = new StreamWriter(path);
writer.WriteLine("timestamp,open,high,low,close,volume");
foreach (var k in klines)
{
    writer.WriteLine(string.Create(CultureInfo.InvariantCulture,
        $"{(long)k.Timestamp},{k.Open},{k.High},{k.Low},{k.Close},{k.Volume}"));
}

Console.WriteLine($"Wrote {klines.Length} klines to {path}");
