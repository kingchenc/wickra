using Wickra;
using Wickra.Examples;

// Breakout: when Bollinger bandwidth is tight (a "squeeze") and price closes above the
// upper band, go long with an ATR(14) trailing stop.
var bars = args.Length > 0 ? MarketData.LoadOhlcvCsv(args[0]) : MarketData.SyntheticCandles(2000);

using var bollinger = new BollingerBands(20, 2.0);
using var atr = new Atr(14);

var returns = new List<double>();
var trades = 0;
var inPosition = false;
var entry = 0.0;
var stop = 0.0;

foreach (var b in bars)
{
    var bands = bollinger.Update(b.Close);
    var atrValue = atr.Update(b.Open, b.High, b.Low, b.Close, b.Volume, b.Timestamp);
    if (bands is not { } band || !double.IsFinite(atrValue))
    {
        continue;
    }

    var bandwidth = band.Middle != 0.0 ? (band.Upper - band.Lower) / band.Middle : double.MaxValue;

    if (!inPosition && bandwidth < 0.06 && b.Close > band.Upper)
    {
        inPosition = true;
        entry = b.Close;
        stop = b.Close - 2.0 * atrValue;
        trades++;
    }
    else if (inPosition)
    {
        stop = Math.Max(stop, b.Close - 2.0 * atrValue); // trail the stop up
        if (b.Close < stop)
        {
            returns.Add((b.Close - entry) / entry);
            inPosition = false;
        }
    }
}

Backtest.Print("Bollinger squeeze", Backtest.Summarize(returns, trades));
