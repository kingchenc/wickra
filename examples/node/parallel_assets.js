// Parallel multi-asset indicator computation via Node.js worker_threads.
//
// Builds a synthetic (assets, bars) panel, runs a serial baseline on the
// main thread, then dispatches the same workload to a pool of workers
// (each loading its own copy of the native wickra binding) and reports
// the speedup. The Node counterpart of examples/python/parallel_assets.py.
//
// Run with:
//   node parallel_assets.js [--assets N] [--bars M]
//                           [--indicator sma|rsi] [--workers W]

const { Worker, isMainThread, workerData, parentPort } = require('node:worker_threads');
const os = require('node:os');

// Deterministic LCG matching the Rust sibling so a side-by-side serial
// computation on both languages produces visibly comparable timings.
function makeRng(seed) {
  let state = seed >>> 0;
  return () => {
    state = (Math.imul(state, 1103515245) + 12345) & 0x7fffffff;
    return state / 0x7fffffff;
  };
}

function synthesizePanel(nAssets, nBars) {
  const panel = new Array(nAssets);
  for (let a = 0; a < nAssets; a++) {
    const rng = makeRng((1234567 + Math.imul(a, 2654435761)) >>> 0);
    const series = new Float64Array(nBars);
    let price = 100.0;
    for (let i = 0; i < nBars; i++) {
      price += (rng() - 0.5) * 0.4;
      series[i] = price;
    }
    panel[a] = series;
  }
  return panel;
}

// Compute the last non-null indicator output for one price series. Kept
// tiny so the same code runs on the main thread (serial baseline) and
// inside each worker (parallel run).
function lastValue(prices, indicator, wickra) {
  const ind = indicator === 'sma' ? new wickra.SMA(14) : new wickra.RSI(14);
  let last = null;
  for (let i = 0; i < prices.length; i++) {
    const v = ind.update(prices[i]);
    if (v !== null) last = v;
  }
  return last;
}

if (!isMainThread) {
  // ---------------- worker side ----------------
  const wickra = require('wickra');
  const { panelSlice, indicator } = workerData;
  const results = new Array(panelSlice.length);
  for (let i = 0; i < panelSlice.length; i++) {
    results[i] = lastValue(panelSlice[i], indicator, wickra);
  }
  parentPort.postMessage(results);
} else {
  // ---------------- main side ----------------
  const wickra = require('wickra');

  function parseArgs(argv) {
    const args = {
      assets: 200,
      bars: 5000,
      indicator: 'sma',
      workers: Math.max(1, os.cpus().length),
    };
    for (let i = 0; i < argv.length; i++) {
      const k = argv[i];
      if (k === '--assets') args.assets = Number(argv[++i]);
      else if (k === '--bars') args.bars = Number(argv[++i]);
      else if (k === '--indicator') args.indicator = argv[++i];
      else if (k === '--workers') args.workers = Number(argv[++i]);
      else throw new Error(`unexpected argument: ${k}`);
    }
    if (!Number.isInteger(args.assets) || args.assets <= 0) {
      throw new Error('--assets must be a positive integer');
    }
    if (!Number.isInteger(args.bars) || args.bars <= 0) {
      throw new Error('--bars must be a positive integer');
    }
    if (args.indicator !== 'sma' && args.indicator !== 'rsi') {
      throw new Error("--indicator: expected 'sma' or 'rsi'");
    }
    if (!Number.isInteger(args.workers) || args.workers <= 0) {
      throw new Error('--workers must be a positive integer');
    }
    return args;
  }

  let args;
  try {
    args = parseArgs(process.argv.slice(2));
  } catch (err) {
    console.error(`error: ${err.message}`);
    process.exit(2);
  }

  console.log(`Generating ${args.assets}×${args.bars} synthetic panel…`);
  const panel = synthesizePanel(args.assets, args.bars);

  // Serial baseline (main thread).
  let t0 = process.hrtime.bigint();
  const serial = new Array(args.assets);
  for (let a = 0; a < args.assets; a++) {
    serial[a] = lastValue(panel[a], args.indicator, wickra);
  }
  const tSerial = Number(process.hrtime.bigint() - t0) / 1e9;
  console.log(
    `Serial:   ${tSerial.toFixed(3).padStart(8)} s  (${args.assets} assets, indicator=${args.indicator})`,
  );

  // Parallel via worker_threads.
  const workerCount = Math.max(1, Math.min(args.workers, args.assets));
  const sliceSize = Math.ceil(args.assets / workerCount);
  t0 = process.hrtime.bigint();
  const promises = [];
  for (let w = 0; w < workerCount; w++) {
    const start = w * sliceSize;
    const end = Math.min(start + sliceSize, args.assets);
    if (start >= end) continue;
    const panelSlice = panel.slice(start, end);
    promises.push(
      new Promise((resolve, reject) => {
        const worker = new Worker(__filename, {
          workerData: { panelSlice, indicator: args.indicator },
        });
        worker.on('message', (msg) => resolve({ start, results: msg }));
        worker.on('error', reject);
        worker.on('exit', (code) => {
          if (code !== 0) reject(new Error(`worker exited with code ${code}`));
        });
      }),
    );
  }

  Promise.all(promises)
    .then((chunks) => {
      const parallel = new Array(args.assets);
      for (const { start, results } of chunks) {
        for (let j = 0; j < results.length; j++) parallel[start + j] = results[j];
      }
      const tParallel = Number(process.hrtime.bigint() - t0) / 1e9;
      const speedup = tSerial / Math.max(tParallel, 1e-9);
      console.log(
        `Parallel: ${tParallel.toFixed(3).padStart(8)} s  ` +
          `(workers=${workerCount}, speedup ~${speedup.toFixed(2)}x)`,
      );

      for (let i = 0; i < args.assets; i++) {
        const s = serial[i];
        const p = parallel[i];
        if (s !== p && !(s === null && p === null)) {
          console.error(`mismatch at asset ${i}: serial=${s} parallel=${p}`);
          process.exit(1);
        }
      }
      console.log('Parallel results match serial results — OK.');
    })
    .catch((err) => {
      console.error(`parallel run failed: ${err.message}`);
      process.exit(1);
    });
}
