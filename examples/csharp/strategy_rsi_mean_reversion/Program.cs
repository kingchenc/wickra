using Wickra;
using Wickra.Examples;

// Strategy example: RSI(14) mean-reversion.
//
// Go long when RSI(14) drops below 30 (oversold), exit when it recovers above
// 70 (overbought). 0.1% fees per trade. The C# counterpart of
// examples/python/strategy_rsi_mean_reversion.py, printing the same summary.
// Uses the checked-in examples/data/btcusdt-1h.csv dataset (pass a CSV path to override).
const double Fee = 0.001;
const double Oversold = 30.0;
const double Overbought = 70.0;

var bars = args.Length > 0 ? MarketData.LoadOhlcvCsv(args[0]) : MarketData.BundledCandles("btcusdt-1h.csv");

using var rsi = new Rsi(14);

var inPosition = false;
var entryPrice = 0.0;
var closedTrades = new List<double>();
var equity = 1.0;
var equityCurve = new List<double>();

foreach (var b in bars)
{
    var value = rsi.Update(b.Close);
    var price = b.Close;
    equityCurve.Add(inPosition ? equity * (price / entryPrice) : equity);
    if (!double.IsFinite(value))
    {
        continue;
    }

    if (!inPosition && value < Oversold)
    {
        entryPrice = price;
        equity *= 1.0 - Fee;
        inPosition = true;
    }
    else if (inPosition && value > Overbought)
    {
        var tradeRet = price / entryPrice - 1.0;
        closedTrades.Add(tradeRet);
        equity *= (1.0 + tradeRet) * (1.0 - Fee);
        inPosition = false;
    }
}

if (inPosition)
{
    var tradeRet = bars[^1].Close / entryPrice - 1.0;
    closedTrades.Add(tradeRet);
    equity *= (1.0 + tradeRet) * (1.0 - Fee);
}

Backtest.PrintSummary("RSI Mean-Reversion (1h, BTCUSDT)",
    bars[0].Close, bars[^1].Close, bars.Length, closedTrades, equity, equityCurve);
