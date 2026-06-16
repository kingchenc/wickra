// Live Binance feed example for the Wickra Node binding.
//
// Connects to Binance's public kline feed (no API key needed) through Wickra's
// native `BinanceFeed` — no third-party WebSocket client — and runs RSI / MACD /
// Bollinger Bands on the incoming close prices. When RSI crosses the common
// overbought / oversold thresholds *and* the MACD histogram confirms the
// direction *and* price pierces the matching Bollinger band, a signal line is
// printed. No orders are placed. It is the Node counterpart of
// examples/python/live_binance.py.
//
// Run it from the repository after building the native binding:
//
//   cd bindings/node && npm install && npx napi build --platform --release
//   cd ../../examples/node && node live_binance.js --symbol BTCUSDT --interval 1m
//
// Stop it with Ctrl+C.

const wickra = require('wickra');

// Kline interval string -> BinanceFeed interval code (the Interval declaration
// order shared by every binding).
const INTERVAL_CODES = {
  '1s': 0, '1m': 1, '3m': 2, '5m': 3, '15m': 4, '30m': 5, '1h': 6, '2h': 7,
  '4h': 8, '6h': 9, '8h': 10, '12h': 11, '1d': 12, '3d': 13, '1w': 14, '1M': 15,
};

// A Binance symbol is strictly alphanumeric (e.g. BTCUSDT).
const SYMBOL_RE = /^[A-Za-z0-9]+$/;

// Parse `--symbol` / `--interval` flags, defaulting to BTCUSDT 1m.
function parseArgs(argv) {
  const args = { symbol: 'BTCUSDT', interval: '1m' };
  for (let i = 0; i < argv.length; i++) {
    if (argv[i] === '--symbol') {
      args.symbol = argv[i + 1];
      i += 1;
    } else if (argv[i] === '--interval') {
      args.interval = argv[i + 1];
      i += 1;
    } else {
      throw new Error(`unexpected argument: ${argv[i]}`);
    }
  }
  return args;
}

// Reject a symbol or interval the feed cannot use.
function validateArgs(symbol, interval) {
  if (typeof symbol !== 'string' || !SYMBOL_RE.test(symbol)) {
    throw new Error(
      `invalid --symbol ${JSON.stringify(symbol)}: expected only letters and digits, e.g. BTCUSDT`,
    );
  }
  if (!(interval in INTERVAL_CODES)) {
    throw new Error(
      `invalid --interval ${JSON.stringify(interval)}: expected one of ` +
        Object.keys(INTERVAL_CODES).join(', '),
    );
  }
}

const fmt = (v) => (v === null || v === undefined || Number.isNaN(v) ? '--' : v.toFixed(2));

function main() {
  let args;
  try {
    args = parseArgs(process.argv.slice(2));
    validateArgs(args.symbol, args.interval);
  } catch (err) {
    console.error(`error: ${err.message}`);
    process.exit(2);
  }

  // One streaming instance of each indicator — the same O(1) update model a
  // real bot would use.
  const rsi = new wickra.RSI(14);
  const macd = new wickra.MACD(12, 26, 9);
  const bb = new wickra.BollingerBands(20, 2.0);

  // Native feed: a blocking poll driving the same tested WebSocket stream as the
  // Rust core. `next(timeoutMs)` returns the next event or null on timeout, so a
  // short timeout in a loop keeps Ctrl+C responsive without a third-party client.
  const feed = new wickra.BinanceFeed(args.symbol, INTERVAL_CODES[args.interval]);
  console.log(
    `Listening for ${args.symbol.toLowerCase()}@kline_${args.interval} klines (Ctrl+C to stop)`,
  );

  let running = true;
  process.on('SIGINT', () => {
    console.log('\nShutting down…');
    running = false;
  });

  while (running) {
    let event;
    try {
      event = feed.next(1000);
    } catch (err) {
      console.error(`feed error: ${err.message}`);
      process.exitCode = 1;
      break;
    }
    if (event === null) {
      continue; // timeout — poll again
    }

    const close = event.close;
    const isClosed = event.isClosed;

    const rsiV = rsi.update(close);
    const macdV = macd.update(close); // { macd, signal, histogram } or null
    const bbV = bb.update(close); // { upper, middle, lower, stddev } or null
    const hist = macdV ? macdV.histogram : null;

    console.log(
      `${isClosed ? 'BAR ' : 'tick'}  close=${fmt(close)}  rsi=${fmt(rsiV)}  ` +
        `hist=${fmt(hist)}  ` +
        `bb=${bbV ? `${fmt(bbV.lower)}/${fmt(bbV.middle)}/${fmt(bbV.upper)}` : '--'}`,
    );

    // Only act once every indicator has warmed up.
    if (rsiV !== null && macdV !== null && bbV !== null) {
      if (rsiV > 70 && hist < 0 && close >= bbV.upper) {
        console.log(
          `  SELL candidate: rsi=${fmt(rsiV)} hist=${fmt(hist)} close >= bb_upper=${fmt(bbV.upper)}`,
        );
      } else if (rsiV < 30 && hist > 0 && close <= bbV.lower) {
        console.log(
          `  BUY candidate: rsi=${fmt(rsiV)} hist=${fmt(hist)} close <= bb_lower=${fmt(bbV.lower)}`,
        );
      }
    }
  }

  feed.close();
  console.log('Connection closed.');
}

main();
