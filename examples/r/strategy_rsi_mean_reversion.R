# Strategy example: RSI(14) mean-reversion.
#
# Go long when RSI(14) drops below 30 (oversold), exit when it recovers above 70
# (overbought). 0.1% fees per trade. The R counterpart of
# examples/python/strategy_rsi_mean_reversion.py, printing the same summary. Uses
# the checked-in examples/data/btcusdt-1h.csv dataset (pass a CSV path to override).
suppressPackageStartupMessages(library(wickra))
source("_common.R")

FEE <- 0.001
OVERSOLD <- 30
OVERBOUGHT <- 70

args <- commandArgs(trailingOnly = TRUE)
bars <- if (length(args) >= 1) load_ohlcv_csv(args[1]) else bundled_candles("btcusdt-1h.csv")

closes <- bars$close
n_bars <- length(closes)

rsi <- Rsi(14)
in_pos <- FALSE; entry_price <- 0; closed <- numeric(0); equity <- 1
equity_curve <- numeric(n_bars)

for (i in seq_len(n_bars)) {
  value <- update(rsi, closes[i])
  price <- closes[i]
  equity_curve[i] <- if (in_pos) equity * (price / entry_price) else equity
  if (!is.finite(value)) next

  if (!in_pos && value < OVERSOLD) {
    entry_price <- price; equity <- equity * (1 - FEE); in_pos <- TRUE
  } else if (in_pos && value > OVERBOUGHT) {
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

print_summary("RSI Mean-Reversion (1h, BTCUSDT)",
              closes[1], closes[n_bars], n_bars, closed, equity, equity_curve)
