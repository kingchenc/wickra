using Wickra;
using Wickra.Examples;

// Strategy example: MACD crossover with ADX trend-strength filter.
//
// Enters long on a MACD histogram cross up (the histogram turns positive) while
// ADX(14) > 20 (a directional market); exits on the opposite MACD crossover
// regardless of ADX. 0.1% fees per trade. The C# counterpart of
// examples/python/strategy_macd_adx.py, printing the same summary. Uses the
// checked-in examples/data/btcusdt-1h.csv dataset (pass a CSV path to override).
const double Fee = 0.001;
const double AdxFloor = 20.0;

var bars = args.Length > 0 ? MarketData.LoadOhlcvCsv(args[0]) : MarketData.BundledCandles("btcusdt-1h.csv");

using var macd = new MacdIndicator(12, 26, 9);
using var adx = new Adx(14);

var inPosition = false;
var entryPrice = 0.0;
var closedTrades = new List<double>();
var equity = 1.0;
var equityCurve = new List<double>();
bool? prevSign = null;

foreach (var b in bars)
{
    var m = macd.Update(b.Close);
    var a = adx.Update(b.Open, b.High, b.Low, b.Close, b.Volume, b.Timestamp);
    var price = b.Close;
    equityCurve.Add(inPosition ? equity * (price / entryPrice) : equity);

    if (m is not { } macdValue || a is not { } adxValue)
    {
        continue;
    }

    var histSign = macdValue.Histogram > 0.0;
    var crossUp = prevSign == false && histSign;
    var crossDown = prevSign == true && !histSign;
    prevSign = histSign;

    if (!inPosition && crossUp && adxValue.Adx > AdxFloor)
    {
        entryPrice = price;
        equity *= 1.0 - Fee;
        inPosition = true;
    }
    else if (inPosition && crossDown)
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

Backtest.PrintSummary("MACD + ADX Trend Filter (1h, BTCUSDT)",
    bars[0].Close, bars[^1].Close, bars.Length, closedTrades, equity, equityCurve);
