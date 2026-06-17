# Strategy example: MACD crossover with ADX trend-strength filter.
#
# Enters long on a MACD histogram cross up (the histogram turns positive) while
# ADX(14) > 20 (a directional market); exits on the opposite MACD crossover
# regardless of ADX. 0.1% fees per trade. The R counterpart of
# examples/python/strategy_macd_adx.py, printing the same summary. Uses the
# checked-in examples/data/btcusdt-1h.csv dataset (pass a CSV path to override).
suppressPackageStartupMessages(library(wickra))
source("_common.R")

FEE <- 0.001
ADX_FLOOR <- 20

args <- commandArgs(trailingOnly = TRUE)
bars <- if (length(args) >= 1) load_ohlcv_csv(args[1]) else bundled_candles("btcusdt-1h.csv")

opens <- bars$open; highs <- bars$high; lows <- bars$low
closes <- bars$close; vols <- bars$volume; ts <- bars$timestamp
n_bars <- length(closes)

macd <- MacdIndicator(12, 26, 9); adx <- Adx(14)
in_pos <- FALSE; entry_price <- 0; closed <- numeric(0); equity <- 1
equity_curve <- numeric(n_bars); have_prev <- FALSE; prev_sign <- FALSE

for (i in seq_len(n_bars)) {
  m <- update(macd, closes[i])
  a <- update(adx, opens[i], highs[i], lows[i], closes[i], vols[i], ts[i])
  price <- closes[i]
  equity_curve[i] <- if (in_pos) equity * (price / entry_price) else equity
  if (is.na(m[["macd"]]) || is.na(a[["adx"]])) next

  hist_sign <- m[["histogram"]] > 0
  cross_up <- have_prev && !prev_sign && hist_sign
  cross_down <- have_prev && prev_sign && !hist_sign
  have_prev <- TRUE; prev_sign <- hist_sign

  if (!in_pos && cross_up && a[["adx"]] > ADX_FLOOR) {
    entry_price <- price; equity <- equity * (1 - FEE); in_pos <- TRUE
  } else if (in_pos && cross_down) {
    trade_ret <- price / entry_price - 1
    closed <- c(closed, trade_ret)
    equity <- equity * (1 + trade_ret) * (1 - FEE)
    in_pos <- FALSE
  }
}

if (in_pos) {
  trade_ret <- closes[n_bars] / entry_price - 1
  closed <- c(closed, trade_ret)
  equity <- equity * (1 + trade_ret) * (1 - FEE)
}

print_summary("MACD + ADX Trend Filter (1h, BTCUSDT)",
              closes[1], closes[n_bars], n_bars, closed, equity, equity_curve)
