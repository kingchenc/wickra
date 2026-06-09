# Download real BTCUSDT hourly klines from the Binance REST API into a CSV that the
# other examples can consume. Requires network access and the 'jsonlite' package.
if (!requireNamespace("jsonlite", quietly = TRUE)) stop("install 'jsonlite' to run this example")
url <- "https://api.binance.com/api/v3/klines?symbol=BTCUSDT&interval=1h&limit=500"
cat("Fetching", url, "\n")
klines <- jsonlite::fromJSON(url)
dir.create("data", showWarnings = FALSE)
df <- data.frame(
  timestamp = as.numeric(klines[, 1]), open = klines[, 2], high = klines[, 3],
  low = klines[, 4], close = klines[, 5], volume = klines[, 6]
)
utils::write.csv(df, "data/btcusdt_1h.csv", row.names = FALSE, quote = FALSE)
cat(sprintf("Wrote %d klines to data/btcusdt_1h.csv\n", nrow(df)))
