using Wickra;
using Wickra.Examples;

// Mean reversion: go long when RSI(14) drops below 30, exit when it recovers above 50.
var bars = args.Length > 0 ? MarketData.LoadOhlcvCsv(args[0]) : MarketData.SyntheticCandles(2000);

using var rsi = new Rsi(14);
var returns = new List<double>();
var trades = 0;
var inPosition = false;
var entry = 0.0;

foreach (var b in bars)
{
    var value = rsi.Update(b.Close);
    if (!double.IsFinite(value))
    {
        continue;
    }

    if (!inPosition && value < 30.0)
    {
        inPosition = true;
        entry = b.Close;
        trades++;
    }
    else if (inPosition && value > 50.0)
    {
        returns.Add((b.Close - entry) / entry);
        inPosition = false;
    }
}

Backtest.Print("RSI mean-reversion", Backtest.Summarize(returns, trades));
