// Strategy example: RSI mean-reversion on hourly BTCUSDT data.
//
// Goes long when RSI(14) crosses below 30 (oversold), exits when RSI crosses
// above 70 (overbought). Position is binary (full-in / full-out), fees are
// 0.1% per trade (Binance maker tier), no stop-loss.
//
// Educational example. NOT a recommended trading strategy in real markets.
// The point is to show how Wickra streaming indicators wire up into a complete
// signal -> fill -> PnL -> equity loop in a single file. It is the Node
// counterpart of `examples/python/strategy_rsi_mean_reversion.py` and the Rust
// `examples/rust/src/bin/strategy_rsi_mean_reversion.rs`, and prints the same
// summary.
//
// Build the native binding once, then run it:
//
//   cd bindings/node && npm install && npx napi build --platform --release
//   cd ../../examples/node && npm install
//   node strategy_rsi_mean_reversion.js
//
// Uses the checked-in `examples/data/btcusdt-1h.csv` dataset.

const fs = require('node:fs');
const path = require('node:path');

const wickra = require('wickra');

const FEE = 0.001;
const RSI_PERIOD = 14;
const OVERSOLD = 30.0;
const OVERBOUGHT = 70.0;

const DEFAULT_CSV = path.join(__dirname, '..', 'data', 'btcusdt-1h.csv');

// Parse a plain OHLCV CSV into an array of candle objects. The Wickra CSV
// layout is plain numeric — no quoted fields, no embedded commas — so splitting
// on `,` is a complete and correct parse for it.
function loadCandles(csvPath) {
  // Native CandleReader: validates the header, tolerates a UTF-8 BOM and field
  // whitespace, and throws on a malformed row. Yields { open, high, low, close,
  // volume, timestamp } objects.
  const text = fs.readFileSync(csvPath, 'utf8');
  return new wickra.CandleReader(text).read();
}

// Forced-sign fixed-point (matches Python's `{:+.Nf}`): the value's own minus is
// preserved by toFixed; we only add an explicit '+' for non-negative values.
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
  if (candles.length < RSI_PERIOD * 4) {
    console.error(`error: dataset too small: ${candles.length}`);
    process.exit(1);
  }

  const rsi = new wickra.RSI(RSI_PERIOD);

  let inPosition = false;
  let entryPrice = 0.0;
  const closedTrades = [];
  let equity = 1.0;
  const equityCurve = [];

  for (const c of candles) {
    const rsiVal = rsi.update(c.close);
    const price = c.close;
    const mtm = inPosition ? equity * (price / entryPrice) : equity;
    equityCurve.push(mtm);

    if (rsiVal == null) continue;

    if (!inPosition && rsiVal < OVERSOLD) {
      entryPrice = price;
      equity *= 1.0 - FEE;
      inPosition = true;
    } else if (inPosition && rsiVal > OVERBOUGHT) {
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
    'RSI Mean-Reversion (1h, BTCUSDT)',
    candles[0].close,
    candles[candles.length - 1].close,
    candles.length,
    closedTrades,
    equity,
    equityCurve,
  );
}

main();
