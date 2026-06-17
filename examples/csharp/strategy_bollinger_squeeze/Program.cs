using Wickra;
using Wickra.Examples;

// Strategy example: Bollinger-squeeze breakout with an ATR(14) trailing stop.
//
// Enters long when Bollinger bandwidth makes a new SqueezeLookback low (a
// volatility squeeze) and price closes above the upper band; exits on an ATR(14)
// trailing stop or when the upper band falls back below the entry. 0.1% fees per
// trade. The C# counterpart of examples/python/strategy_bollinger_squeeze.py,
// printing the same summary. Uses the checked-in examples/data/btcusdt-1d.csv
// dataset (pass a CSV path to override).
const double Fee = 0.001;
const double AtrStopMult = 2.0;
const int SqueezeLookback = 180;

var bars = args.Length > 0 ? MarketData.LoadOhlcvCsv(args[0]) : MarketData.BundledCandles("btcusdt-1d.csv");

using var bollinger = new BollingerBands(20, 2.0);
using var atr = new Atr(14);

var inPosition = false;
var entryPrice = 0.0;
var stopLevel = 0.0;
var closedTrades = new List<double>();
var equity = 1.0;
var equityCurve = new List<double>();
var bwWindow = new Queue<double>();

foreach (var b in bars)
{
    var bands = bollinger.Update(b.Close);
    var atrValue = atr.Update(b.Open, b.High, b.Low, b.Close, b.Volume, b.Timestamp);
    var price = b.Close;
    equityCurve.Add(inPosition ? equity * (price / entryPrice) : equity);

    if (bands is not { } band || !double.IsFinite(atrValue))
    {
        continue;
    }

    if (Math.Abs(band.Middle) <= 1e-12)
    {
        continue;
    }

    var bandwidth = (band.Upper - band.Lower) / band.Middle;
    bwWindow.Enqueue(bandwidth);
    if (bwWindow.Count > SqueezeLookback)
    {
        bwWindow.Dequeue();
    }

    if (bwWindow.Count < SqueezeLookback)
    {
        continue;
    }

    var minBw = bwWindow.Min();

    if (inPosition)
    {
        if (price < stopLevel || band.Upper < entryPrice)
        {
            var tradeRet = price / entryPrice - 1.0;
            closedTrades.Add(tradeRet);
            equity *= (1.0 + tradeRet) * (1.0 - Fee);
            inPosition = false;
        }
    }
    else
    {
        var isNewLow = Math.Abs(bandwidth - minBw) < 1e-12;
        if (isNewLow && price > band.Upper)
        {
            entryPrice = price;
            stopLevel = price - AtrStopMult * atrValue;
            equity *= 1.0 - Fee;
            inPosition = true;
        }
    }
}

if (inPosition)
{
    var tradeRet = bars[^1].Close / entryPrice - 1.0;
    closedTrades.Add(tradeRet);
    equity *= (1.0 + tradeRet) * (1.0 - Fee);
}

Backtest.PrintSummary("Bollinger Squeeze Breakout (1d, BTCUSDT)",
    bars[0].Close, bars[^1].Close, bars.Length, closedTrades, equity, equityCurve);
