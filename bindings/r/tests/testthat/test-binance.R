# The live Binance feed's connect -> read -> reconnect pipeline is covered
# deterministically by the Rust mock-WS-server tests in wickra-data. Here we only
# assert the binding's error paths, which need no network.

test_that("binance feed rejects bad parameters", {
  expect_error(BinanceFeed("", 1L), "BinanceFeed")
  expect_error(BinanceFeed("BTCUSDT", 1L, "ws://127.0.0.1:1"), "BinanceFeed")
})
