// Module worker for `examples/wasm/parallel_assets.html`. Each worker loads
// its own copy of the WebAssembly module, then processes the slice of the
// (assets, bars) panel its parent dispatches via `postMessage` and sends
// the per-asset "last non-null indicator value" back.

import init, { SMA, RSI } from "../../bindings/wasm/pkg/wickra_wasm.js";

const ready = init();

self.addEventListener("message", async (event) => {
  await ready;
  const { panelSlice, indicator } = event.data;

  const results = new Array(panelSlice.length);
  for (let i = 0; i < panelSlice.length; i++) {
    const prices = panelSlice[i];
    const ind = indicator === "sma" ? new SMA(14) : new RSI(14);
    let last = null;
    for (let j = 0; j < prices.length; j++) {
      const v = ind.update(prices[j]);
      if (v !== null) last = v;
    }
    results[i] = last;
  }

  self.postMessage(results);
});
