# Stream live BTCUSDT 1-minute klines from Binance and feed each close through EMA(20).
# Uses Wickra's native BinanceFeed — no third-party packages. Requires network
# access. Runs for up to ~60 seconds.
library(wickra)

ema <- Ema(20)
# Interval code 1 == 1m (the Interval declaration order shared by every binding).
feed <- BinanceFeed("BTCUSDT", 1L)
cat("Streaming for up to 60s...\n")

deadline <- Sys.time() + 60
repeat {
  if (Sys.time() > deadline) break
  # binance_next() returns a named list on an event, or NULL on timeout.
  event <- binance_next(feed, 1000)
  if (is.null(event)) next
  cat(sprintf("close=%.2f  EMA(20)=%.2f\n", event$close, update(ema, event$close)))
}
binance_close(feed)
cat("Done (time limit reached).\n")
