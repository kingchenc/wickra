using System.Globalization;
using System.Net.WebSockets;
using System.Text;
using System.Text.Json;
using Wickra;

// Stream live BTCUSDT 1-minute klines from Binance and feed each close through EMA(20).
// Requires network access (build-only in CI). Runs for up to 60 seconds.
var uri = new Uri("wss://stream.binance.com:9443/ws/btcusdt@kline_1m");
Console.WriteLine($"Connecting to {uri} (up to 60s)...");

using var ws = new ClientWebSocket();
using var cts = new CancellationTokenSource(TimeSpan.FromSeconds(60));
using var ema = new Ema(20);
var buffer = new byte[8192];

try
{
    await ws.ConnectAsync(uri, cts.Token);
    while (ws.State == WebSocketState.Open && !cts.IsCancellationRequested)
    {
        var result = await ws.ReceiveAsync(buffer, cts.Token);
        if (result.MessageType == WebSocketMessageType.Close)
        {
            break;
        }

        using var doc = JsonDocument.Parse(Encoding.UTF8.GetString(buffer, 0, result.Count));
        if (doc.RootElement.TryGetProperty("k", out var k))
        {
            var close = double.Parse(k.GetProperty("c").GetString()!, CultureInfo.InvariantCulture);
            var value = ema.Update(close);
            Console.WriteLine($"close={close:F2}  EMA(20)={value:F2}");
        }
    }
}
catch (OperationCanceledException)
{
    Console.WriteLine("Done (time limit reached).");
}
