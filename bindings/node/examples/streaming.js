// Streaming indicators with the Wickra Node binding.
//
// Feeds a synthetic price series through several indicators tick by tick —
// the same O(1)-per-update model a live trading bot would use — and prints a
// status line whenever every indicator has warmed up.
//
// Run it from the repository after building the native module:
//
//   cd bindings/node
//   npm install
//   npx napi build --platform --release
//   node examples/streaming.js
//
// In your own project, install the package and use `require('wickra')`
// instead of the relative `require('..')` below.

const wickra = require('..');

// A deterministic synthetic series: a slow trend with two oscillations and a
// little noise from a seeded generator (so every run prints the same thing).
function makeSeries(n) {
  let seed = 1234567;
  const rand = () => {
    seed = (seed * 1103515245 + 12345) & 0x7fffffff;
    return seed / 0x7fffffff;
  };
  const prices = [];
  for (let t = 0; t < n; t++) {
    const price =
      100 +
      t * 0.05 +
      Math.sin(t * 0.07) * 8 +
      Math.cos(t * 0.21) * 3 +
      (rand() - 0.5);
    prices.push(price);
  }
  return prices;
}

const fmt = (v) =>
  v === null || v === undefined || Number.isNaN(v) ? '  --  ' : v.toFixed(2);

function main() {
  console.log(`Wickra ${wickra.version()} — streaming indicator demo\n`);

  const sma = new wickra.SMA(20);
  const ema = new wickra.EMA(20);
  const rsi = new wickra.RSI(14);
  const macd = new wickra.MACD(12, 26, 9);

  const prices = makeSeries(120);
  let signals = 0;

  for (let t = 0; t < prices.length; t++) {
    const price = prices[t];
    const smaV = sma.update(price);
    const emaV = ema.update(price);
    const rsiV = rsi.update(price);
    const macdV = macd.update(price); // { macd, signal, histogram } or null

    // Only act once every indicator has produced a value.
    if (smaV == null || emaV == null || rsiV == null || macdV == null) {
      continue;
    }

    const overbought = rsiV > 70 && macdV.histogram < 0;
    const oversold = rsiV < 30 && macdV.histogram > 0;
    const tag = overbought ? 'SELL?' : oversold ? 'BUY? ' : '     ';
    if (overbought || oversold) signals++;

    console.log(
      `t=${String(t).padStart(3)}  price=${fmt(price)}  ` +
        `sma=${fmt(smaV)}  ema=${fmt(emaV)}  rsi=${fmt(rsiV)}  ` +
        `macd_hist=${fmt(macdV.histogram)}  ${tag}`,
    );
  }

  console.log(`\nDone — ${signals} candidate signal(s) over ${prices.length} ticks.`);
}

main();
