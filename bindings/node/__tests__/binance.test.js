// The live Binance feed's connect → read → reconnect pipeline is covered
// deterministically by the Rust mock-WS-server tests in wickra-data. Here we only
// assert the binding's error paths, which need no network.

const test = require('node:test');
const assert = require('node:assert/strict');
const { BinanceFeed, fetchBinanceKlines } = require('..');

test('binance feed rejects an unknown interval', () => {
  assert.throws(() => new BinanceFeed('BTCUSDT', 99));
});

test('binance feed rejects an empty symbol list', () => {
  assert.throws(() => new BinanceFeed('', 1));
});

test('binance feed rejects an unreachable endpoint', () => {
  assert.throws(() => new BinanceFeed('BTCUSDT', 1, 'ws://127.0.0.1:1'));
});

// The REST fetcher's parse/HTTP success path is covered deterministically by the
// Rust mock-HTTP-server tests in wickra-data; here we only assert the binding's
// error paths, which short-circuit before (or fail fast at) the network.
test('fetchBinanceKlines rejects an unknown interval', () => {
  assert.throws(() => fetchBinanceKlines('BTCUSDT', 99, 1));
});

test('fetchBinanceKlines rejects an out-of-range limit', () => {
  assert.throws(() => fetchBinanceKlines('BTCUSDT', 6, 0));
});

test('fetchBinanceKlines surfaces an unreachable endpoint', () => {
  assert.throws(() => fetchBinanceKlines('BTCUSDT', 6, 1, null, null, 'http://127.0.0.1:1'));
});
