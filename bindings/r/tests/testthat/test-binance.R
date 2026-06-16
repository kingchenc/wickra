# The live Binance feed's connect -> read -> reconnect pipeline is covered
# deterministically by the Rust mock-WS-server tests in wickra-data. Here we only
# assert the binding's error paths, which need no network.

test_that("binance feed rejects bad parameters", {
  expect_error(BinanceFeed("", 1L), "BinanceFeed")
  expect_error(BinanceFeed("BTCUSDT", 1L, "ws://127.0.0.1:1"), "BinanceFeed")
})

# The REST fetcher's parse/HTTP success path is covered by the Rust
# mock-HTTP-server tests; here we only assert the binding's error paths.
test_that("fetch_binance_klines rejects bad parameters", {
  expect_error(fetch_binance_klines("BTCUSDT", 6L, 0L), "limit")
  expect_error(
    fetch_binance_klines("BTCUSDT", 6L, 1L, base_url = "http://127.0.0.1:1"),
    "fetch_binance_klines"
  )
})
