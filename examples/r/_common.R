# Shared helpers for the offline Wickra R examples: deterministic synthetic
# market data, a small OHLCV CSV loader, and an equity-curve summary. Mirrors the
# helpers used by the C, C# and Go example suites.

synthetic_prices <- function(count, start = 100) {
  i <- seq_len(count) - 1
  start + 12 * sin(i * 0.05) + 5 * sin(i * 0.013) + i * 0.01
}

synthetic_candles <- function(count, start_ts = 0, step_ms = 3600000) {
  prices <- synthetic_prices(count + 1)
  k <- seq_len(count)
  i <- k - 1
  open <- prices[k]
  close <- prices[k + 1]
  data.frame(
    open = open,
    high = pmax(open, close) + 0.5 + abs(sin(i * 0.7)),
    low = pmin(open, close) - 0.5 - abs(cos(i * 0.7)),
    close = close,
    volume = 1000 + 500 * (1 + sin(i * 0.1)),
    timestamp = start_ts + i * step_ms
  )
}

load_ohlcv_csv <- function(path) {
  # Native CandleReader: header validation, BOM and field-whitespace tolerance.
  # read() returns an (n x 6) matrix of open, high, low, close, volume, timestamp.
  m <- read(CandleReader(paste(readLines(path, warn = FALSE), collapse = "\n")))
  data.frame(open = m[, "open"], high = m[, "high"], low = m[, "low"],
             close = m[, "close"], volume = m[, "volume"], timestamp = m[, "timestamp"])
}

summarize_equity <- function(returns, trades, periods_per_year = 252) {
  equity <- 1; peak <- 1; maxdd <- 0
  for (r in returns) {
    equity <- equity * (1 + r)
    peak <- max(peak, equity)
    maxdd <- max(maxdd, (peak - equity) / peak)
  }
  mean_r <- if (length(returns)) mean(returns) else 0
  sd_r <- if (length(returns) > 1) stats::sd(returns) else 0
  sharpe <- if (sd_r > 1e-12) mean_r / sd_r * sqrt(periods_per_year) else 0
  list(total_return_pct = (equity - 1) * 100, sharpe = sharpe,
       max_dd_pct = maxdd * 100, trades = trades)
}

print_equity <- function(name, r) {
  cat(sprintf("%-26s return=%8.2f%%  sharpe=%6.2f  maxDD=%6.2f%%  trades=%d\n",
              name, r$total_return_pct, r$sharpe, r$max_dd_pct, r$trades))
}
