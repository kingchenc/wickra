using Wickra;
using Wickra.Examples;

// Feed a synthetic price series through several indicators tick by tick (O(1) each).
var prices = MarketData.SyntheticPrices(500);

using var sma = new Sma(20);
using var ema = new Ema(20);
using var rsi = new Rsi(14);
using var macd = new MacdIndicator(12, 26, 9);

double lastSma = 0, lastEma = 0, lastRsi = 0;
MacdOutput? lastMacd = null;
foreach (var price in prices)
{
    lastSma = sma.Update(price);
    lastEma = ema.Update(price);
    lastRsi = rsi.Update(price);
    lastMacd = macd.Update(price);
}

Console.WriteLine($"Streamed {prices.Length} prices through SMA(20), EMA(20), RSI(14), MACD(12,26,9):");
Console.WriteLine($"  SMA  = {lastSma:F4}");
Console.WriteLine($"  EMA  = {lastEma:F4}");
Console.WriteLine($"  RSI  = {lastRsi:F4}");
if (lastMacd is { } m)
{
    Console.WriteLine($"  MACD = {m.Macd:F4}  signal={m.Signal:F4}  hist={m.Histogram:F4}");
}
