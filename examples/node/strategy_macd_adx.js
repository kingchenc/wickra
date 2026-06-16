// Strategy example: MACD crossover with ADX trend-strength filter.
//
// Long-only trend follower. Entries fire on a MACD-line-crosses-above-signal
// event (histogram turns positive) while ADX(14) > 20 (i.e. a directional
// market). Exits on the opposite MACD crossover regardless of ADX. 0.1% fees
// per trade.
//
// Educational example. NOT a live trading recommendation. The Node counterpart
// of `examples/python/strategy_macd_adx.py` and the Rust
// `examples/rust/src/bin/strategy_macd_adx.rs`, printing the same summary.
//
// Build the native binding once, then run it:
//
//   cd bindings/node && npm install && npx napi build --platform --release
//   cd ../../examples/node && npm install
//   node strategy_macd_adx.js
//
// Uses the checked-in `examples/data/btcusdt-1h.csv` dataset.

const fs = require('node:fs');
const path = require('node:path');

const wickra = require('wickra');

const FEE = 0.001;
const ADX_FLOOR = 20.0;

const DEFAULT_CSV = path.join(__dirname, '..', 'data', 'btcusdt-1h.csv');

function loadCandles(csvPath) {
  // Native CandleReader: validates the header, tolerates a UTF-8 BOM and field
  // whitespace, and throws on a malformed row. Yields { open, high, low, close,
  // volume, timestamp } objects.
  const text = fs.readFileSync(csvPath, 'utf8');
  return new wickra.CandleReader(text).read();
}

function signed(value, digits) {
  return (value >= 0 ? '+' : '') + value.toFixed(digits);
}

function printSummary(name, firstPrice, lastPrice, bars, closedTrades, finalEquity, equityCurve) {
  const buyHold = lastPrice / firstPrice;
  const stratReturn = finalEquity - 1.0;
  const bhReturn = buyHold - 1.0;
  const wins = closedTrades.filter((r) => r > 0).length;
  const losses = closedTrades.filter((r) => r < 0).length;
  const best = closedTrades.length ? Math.max(...closedTrades) : 0.0;
  const worst = closedTrades.length ? Math.min(...closedTrades) : 0.0;
  const n = closedTrades.length;
  const meanRet = n ? closedTrades.reduce((a, r) => a + r, 0) / n : 0.0;
  const varRet =
    n > 1 ? closedTrades.reduce((a, r) => a + (r - meanRet) ** 2, 0) / (n - 1) : 0.0;
  const stddev = Math.sqrt(varRet);
  const sharpe = varRet > 0 ? meanRet / stddev : 0.0;

  let peak = equityCurve.length ? equityCurve[0] : 1.0;
  let maxDd = 0.0;
  for (const eq of equityCurve) {
    if (eq > peak) peak = eq;
    const dd = (peak - eq) / peak;
    if (dd > maxDd) maxDd = dd;
  }

  const label = (s) => s.padEnd(23);
  console.log(`=== ${name} ===`);
  console.log(`${label('Bars:')}${bars}`);
  console.log(`${label('Trades:')}${n} (W${wins} / L${losses})`);
  console.log(`${label('Strategy return:')}${signed(stratReturn * 100, 2)}%`);
  console.log(`${label('Buy & Hold return:')}${signed(bhReturn * 100, 2)}%`);
  console.log(`${label('Excess over BH:')}${signed((stratReturn - bhReturn) * 100, 2)}%`);
  console.log(`${label('Max drawdown:')}${(maxDd * 100).toFixed(2)}%`);
  console.log(
    `${label('Per-trade Sharpe:')}${sharpe.toFixed(2)}  ` +
      `(mean ${signed(meanRet, 4)}, stddev ${stddev.toFixed(4)})`,
  );
  console.log(`${label('Best / worst trade:')}${signed(best * 100, 2)}% / ${signed(worst * 100, 2)}%`);
  console.log();
  console.log(
    'NOTE: Educational example — fees, slippage, funding costs and tax effects ' +
      'are simplified or omitted. Past performance is not indicative of future results.',
  );
}

function main() {
  const csvPath = process.argv[2] || DEFAULT_CSV;

  let candles;
  try {
    candles = loadCandles(csvPath);
  } catch (err) {
    console.error(`error: ${err.message}`);
    process.exit(1);
  }

  const macd = new wickra.MACD(12, 26, 9);
  const adx = new wickra.ADX(14);

  let inPosition = false;
  let entryPrice = 0.0;
  const closedTrades = [];
  let equity = 1.0;
  const equityCurve = [];
  // null until the first warm bar, then a boolean — matches the Python
  // `prev_hist_sign: bool | None`, so a cross only fires after a real prior sign.
  let prevHistSign = null;

  for (const c of candles) {
    const macdOut = macd.update(c.close);
    const adxOut = adx.update(c.high, c.low, c.close);
    const price = c.close;
    const mtm = inPosition ? equity * (price / entryPrice) : equity;
    equityCurve.push(mtm);

    if (macdOut == null || adxOut == null) continue;

    const histogram = macdOut.histogram;
    const adxValue = adxOut.adx;

    const histSign = histogram > 0.0;
    const crossUp = prevHistSign === false && histSign;
    const crossDown = prevHistSign === true && !histSign;
    prevHistSign = histSign;

    if (!inPosition && crossUp && adxValue > ADX_FLOOR) {
      entryPrice = price;
      equity *= 1.0 - FEE;
      inPosition = true;
    } else if (inPosition && crossDown) {
      const tradeRet = price / entryPrice - 1.0;
      closedTrades.push(tradeRet);
      equity *= (1.0 + tradeRet) * (1.0 - FEE);
      inPosition = false;
    }
  }

  if (inPosition) {
    const lastPrice = candles[candles.length - 1].close;
    const tradeRet = lastPrice / entryPrice - 1.0;
    closedTrades.push(tradeRet);
    equity *= (1.0 + tradeRet) * (1.0 - FEE);
  }

  printSummary(
    'MACD + ADX Trend Filter (1h, BTCUSDT)',
    candles[0].close,
    candles[candles.length - 1].close,
    candles.length,
    closedTrades,
    equity,
    equityCurve,
  );
}

main();
