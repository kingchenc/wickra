// Live trading skeleton for the Wickra Node binding.
//
// Connects to Binance's public WebSocket feed (no API key needed) and runs
// RSI / MACD / Bollinger Bands on the incoming close prices. When RSI crosses
// the common overbought / oversold thresholds *and* the MACD histogram
// confirms the direction *and* price pierces the matching Bollinger band, a
// signal line is printed. No orders are placed. It is the Node counterpart of
// examples/python/live_trading.py.
//
// Run it from the repository after building the native binding:
//
//   cd bindings/node && npm install && npx napi build --platform --release
//   cd ../../examples/node && npm install     # pulls `ws` for this example
//   node live_trading.js --symbol BTCUSDT --interval 1m
//
// Stop it with Ctrl+C.

const wickra = require('wickra');

let WebSocket;
try {
  WebSocket = require('ws');
} catch (err) {
  console.error('This example needs the `ws` package — run `npm install` in examples/node.');
  process.exit(1);
}

const BINANCE_WS = 'wss://stream.binance.com:9443/stream';

// Kline intervals the public Binance WebSocket API recognises.
const VALID_INTERVALS = new Set([
  '1s', '1m', '3m', '5m', '15m', '30m', '1h', '2h', '4h', '6h', '8h', '12h',
  '1d', '3d', '1w', '1M',
]);

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

// Reject a symbol or interval that is not safe to splice into the WS URL.
// Both are interpolated straight into the stream name, so validating up front
// keeps the URL well-formed without needing to escape it.
function validateArgs(symbol, interval) {
  if (typeof symbol !== 'string' || !SYMBOL_RE.test(symbol)) {
    throw new Error(
      `invalid --symbol ${JSON.stringify(symbol)}: expected only letters and digits, e.g. BTCUSDT`,
    );
  }
  if (!VALID_INTERVALS.has(interval)) {
    throw new Error(
      `invalid --interval ${JSON.stringify(interval)}: expected one of ` +
        [...VALID_INTERVALS].join(', '),
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

  const stream = `${args.symbol.toLowerCase()}@kline_${args.interval}`;
  const url = `${BINANCE_WS}?streams=${stream}`;
  console.log(`Connecting to ${url}`);

  const ws = new WebSocket(url);

  ws.on('open', () => {
    console.log(`Connected, listening for ${stream} klines (Ctrl+C to stop)`);
  });

  ws.on('message', (raw) => {
    let envelope;
    try {
      envelope = JSON.parse(raw.toString());
    } catch (err) {
      return; // ignore non-JSON frames
    }
    const k = (envelope.data && envelope.data.k) || null;
    if (!k || k.c === undefined) {
      // Subscription acks, heartbeats and error frames carry no kline payload —
      // skip them instead of crashing on Number(undefined).
      return;
    }
    const close = Number(k.c);
    const isClosed = Boolean(k.x);

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
  });

  ws.on('error', (err) => {
    console.error(`websocket error: ${err.message}`);
    process.exitCode = 1;
  });

  ws.on('close', () => {
    console.log('Connection closed.');
  });

  // Translate Ctrl+C into a clean shutdown.
  process.on('SIGINT', () => {
    console.log('\nShutting down…');
    ws.close();
    process.exit(0);
  });
}

main();
