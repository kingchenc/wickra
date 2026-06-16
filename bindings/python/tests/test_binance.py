"""The live Binance feed's connect -> read -> reconnect pipeline is covered
deterministically by the Rust mock-WS-server tests in wickra-data. Here we only
assert the binding's error paths, which need no network.
"""
import pytest
import wickra as ta


def test_binance_feed_rejects_bad_params():
    with pytest.raises(ValueError):
        ta.BinanceFeed("BTCUSDT", 99)  # unknown interval code
    with pytest.raises(ValueError):
        ta.BinanceFeed("", 1)  # empty symbol list
    with pytest.raises(ValueError):
        ta.BinanceFeed("BTCUSDT", 1, "ws://127.0.0.1:1")  # unreachable endpoint


def test_fetch_binance_klines_rejects_bad_params():
    # The parse/HTTP success path is covered by the Rust mock-HTTP-server tests;
    # here we only assert the binding's error paths.
    with pytest.raises(ValueError):
        ta.fetch_binance_klines("BTCUSDT", 99, 1)  # unknown interval code
    with pytest.raises(ValueError):
        ta.fetch_binance_klines("BTCUSDT", 6, 0)  # out-of-range limit
    with pytest.raises(ValueError):
        ta.fetch_binance_klines("BTCUSDT", 6, 1, base_url="http://127.0.0.1:1")  # unreachable
