// Strategy example: Bollinger-Squeeze breakout with ATR-based stop.
//
// Enters long when the Bollinger Bandwidth has just printed a fresh 6-month low
// (the squeeze) and price closes above the upper band (the release). Exits when
// price closes below entry minus 2 * ATR(14), or when the upper band rolls back
// under the entry price. 0.1% fees per trade.
//
// Educational example. NOT a live trading recommendation. The Node counterpart
// of `examples/python/strategy_bollinger_squeeze.py` and the Rust
// `examples/rust/src/bin/strategy_bollinger_squeeze.rs`, printing the same
// summary.
//
// Build the native binding once, then run it:
//
//   cd bindings/node && npm install && npx napi build --platform --release
//   cd ../../examples/node && npm install
//   node strategy_bollinger_squeeze.js
//
// Uses the checked-in `examples/data/btcusdt-1d.csv` dataset because daily bars
// give an interpretable 6-month-low lookback (~180 bars).

const fs = require('node:fs');
const path = require('node:path');

const wickra = require('wickra');

const FEE = 0.001;
const BB_PERIOD = 20;
const BB_K = 2.0;
const ATR_PERIOD = 14;
const ATR_STOP_MULT = 2.0;
const SQUEEZE_LOOKBACK = 180;

const DEFAULT_CSV = path.join(__dirname, '..', 'data', 'btcusdt-1d.csv');

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
  if (candles.length < SQUEEZE_LOOKBACK + BB_PERIOD) {
    console.error(
      `error: dataset has only ${candles.length} bars; need at least ` +
        `${SQUEEZE_LOOKBACK + BB_PERIOD}`,
    );
    process.exit(1);
  }

  const bb = new wickra.BollingerBands(BB_PERIOD, BB_K);
  const atr = new wickra.ATR(ATR_PERIOD);
  // Rolling window of the last SQUEEZE_LOOKBACK bandwidths (Python deque(maxlen)).
  const bwWindow = [];

  let inPosition = false;
  let entryPrice = 0.0;
  let stopLevel = 0.0;
  const closedTrades = [];
  let equity = 1.0;
  const equityCurve = [];

  for (const c of candles) {
    const bbOut = bb.update(c.close);
    const atrVal = atr.update(c.high, c.low, c.close);
    const price = c.close;
    const mtm = inPosition ? equity * (price / entryPrice) : equity;
    equityCurve.push(mtm);

    if (bbOut == null || atrVal == null) continue;

    const { upper, middle, lower } = bbOut;
    const bandwidth = Math.abs(middle) > 1e-12 ? (upper - lower) / middle : NaN;

    if (Number.isNaN(bandwidth)) continue;
    bwWindow.push(bandwidth);
    if (bwWindow.length > SQUEEZE_LOOKBACK) bwWindow.shift();
    if (bwWindow.length < SQUEEZE_LOOKBACK) continue;
    const minBw = bwWindow.reduce((m, v) => (v < m ? v : m), Infinity);

    if (inPosition) {
      const stopHit = price < stopLevel;
      const upperCollapse = upper < entryPrice;
      if (stopHit || upperCollapse) {
        const tradeRet = price / entryPrice - 1.0;
        closedTrades.push(tradeRet);
        equity *= (1.0 + tradeRet) * (1.0 - FEE);
        inPosition = false;
      }
    } else {
      const isNewLow = Math.abs(bandwidth - minBw) < 1e-12;
      const breakout = price > upper;
      if (isNewLow && breakout) {
        entryPrice = price;
        stopLevel = price - ATR_STOP_MULT * atrVal;
        equity *= 1.0 - FEE;
        inPosition = true;
      }
    }
  }

  if (inPosition) {
    const lastPrice = candles[candles.length - 1].close;
    const tradeRet = lastPrice / entryPrice - 1.0;
    closedTrades.push(tradeRet);
    equity *= (1.0 + tradeRet) * (1.0 - FEE);
  }

  printSummary(
    'Bollinger Squeeze Breakout (1d, BTCUSDT)',
    candles[0].close,
    candles[candles.length - 1].close,
    candles.length,
    closedTrades,
    equity,
    equityCurve,
  );
}

main();
