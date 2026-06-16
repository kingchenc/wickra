// The live Binance feed's connect → read → reconnect pipeline is covered
// deterministically by the Rust mock-WS-server tests in wickra-data. Here we only
// assert the binding's error paths, which need no network.

const test = require('node:test');
const assert = require('node:assert/strict');
const { BinanceFeed } = require('..');

test('binance feed rejects an unknown interval', () => {
  assert.throws(() => new BinanceFeed('BTCUSDT', 99));
});

test('binance feed rejects an empty symbol list', () => {
  assert.throws(() => new BinanceFeed('', 1));
});

test('binance feed rejects an unreachable endpoint', () => {
  assert.throws(() => new BinanceFeed('BTCUSDT', 1, 'ws://127.0.0.1:1'));
});
