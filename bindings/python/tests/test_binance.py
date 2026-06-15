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
