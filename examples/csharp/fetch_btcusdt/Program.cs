using System.Globalization;
using System.Text.Json;

// Download real BTCUSDT hourly klines from the Binance REST API into a CSV that the
// other examples can consume. Requires network access (build-only in CI).
const string url = "https://api.binance.com/api/v3/klines?symbol=BTCUSDT&interval=1h&limit=500";

using var http = new HttpClient();
Console.WriteLine($"Fetching {url}");
var json = await http.GetStringAsync(url);

using var doc = JsonDocument.Parse(json);
var dir = Path.Combine(AppContext.BaseDirectory, "data");
Directory.CreateDirectory(dir);
var path = Path.Combine(dir, "btcusdt_1h.csv");

using var writer = new StreamWriter(path);
writer.WriteLine("timestamp,open,high,low,close,volume");
var count = 0;
foreach (var kline in doc.RootElement.EnumerateArray())
{
    // Binance kline array: [openTime, open, high, low, close, volume, ...]
    var ts = kline[0].GetInt64();
    var o = kline[1].GetString();
    var h = kline[2].GetString();
    var l = kline[3].GetString();
    var c = kline[4].GetString();
    var v = kline[5].GetString();
    writer.WriteLine(string.Create(CultureInfo.InvariantCulture, $"{ts},{o},{h},{l},{c},{v}"));
    count++;
}

Console.WriteLine($"Wrote {count} klines to {path}");
