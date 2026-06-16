# Download real BTCUSDT hourly klines from the Binance REST API into a CSV that the
# other examples can consume, using Wickra's native fetcher — no third-party packages.
# Requires network access.
library(wickra)

cat("Fetching 500 BTCUSDT 1h klines from Binance...\n")
# Interval code 6 == 1h. Returns an (n x 6) matrix with columns
# open, high, low, close, volume, timestamp.
m <- fetch_binance_klines("BTCUSDT", 6L, 500L)

dir.create("data", showWarnings = FALSE)
df <- data.frame(
  timestamp = m[, "timestamp"], open = m[, "open"], high = m[, "high"],
  low = m[, "low"], close = m[, "close"], volume = m[, "volume"]
)
utils::write.csv(df, "data/btcusdt_1h.csv", row.names = FALSE, quote = FALSE)
cat(sprintf("Wrote %d klines to data/btcusdt_1h.csv\n", nrow(df)))
