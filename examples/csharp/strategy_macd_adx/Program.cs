using Wickra;
using Wickra.Examples;

// Trend follower: enter long on a MACD histogram cross up, but only when ADX(14) > 20
// confirms a trend; exit when the histogram crosses back below zero.
var bars = args.Length > 0 ? MarketData.LoadOhlcvCsv(args[0]) : MarketData.SyntheticCandles(2000);

using var macd = new MacdIndicator(12, 26, 9);
using var adx = new Adx(14);

var returns = new List<double>();
var trades = 0;
var inPosition = false;
var entry = 0.0;
var prevHistogram = double.NaN;

foreach (var b in bars)
{
    var m = macd.Update(b.Close);
    var a = adx.Update(b.Open, b.High, b.Low, b.Close, b.Volume, b.Timestamp);
    if (m is not { } macdValue || a is not { } adxValue)
    {
        continue;
    }

    var trending = adxValue.Adx > 20.0;
    if (!inPosition && trending && double.IsFinite(prevHistogram) && prevHistogram <= 0.0 && macdValue.Histogram > 0.0)
    {
        inPosition = true;
        entry = b.Close;
        trades++;
    }
    else if (inPosition && macdValue.Histogram < 0.0)
    {
        returns.Add((b.Close - entry) / entry);
        inPosition = false;
    }

    prevHistogram = macdValue.Histogram;
}

Backtest.Print("MACD + ADX trend", Backtest.Summarize(returns, trades));
