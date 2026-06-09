using Wickra;
using Wickra.Examples;

// Compute a basket of indicators over an OHLCV series and print a summary.
// Pass a CSV path (timestamp,open,high,low,close,volume) or run on synthetic data.
var source = args.Length > 0 ? args[0] : "synthetic";
Bar[] bars = args.Length > 0 ? MarketData.LoadOhlcvCsv(args[0]) : MarketData.SyntheticCandles(1000);

Console.WriteLine($"Backtest over {bars.Length} bars ({source}):");

using var sma = new Sma(20);
using var ema = new Ema(50);
using var rsi = new Rsi(14);
using var atr = new Atr(14);

double lastSma = 0, lastEma = 0, lastRsi = 0, lastAtr = 0;
var oversold = 0;
foreach (var b in bars)
{
    lastSma = sma.Update(b.Close);
    lastEma = ema.Update(b.Close);
    lastRsi = rsi.Update(b.Close);
    lastAtr = atr.Update(b.Open, b.High, b.Low, b.Close, b.Volume, b.Timestamp);
    if (double.IsFinite(lastRsi) && lastRsi < 30.0)
    {
        oversold++;
    }
}

Console.WriteLine($"  SMA(20) last = {lastSma:F4}");
Console.WriteLine($"  EMA(50) last = {lastEma:F4}");
Console.WriteLine($"  RSI(14) last = {lastRsi:F4}  ({oversold} oversold bars)");
Console.WriteLine($"  ATR(14) last = {lastAtr:F4}");
