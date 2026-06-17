# Strategy example: Bollinger-squeeze breakout with an ATR(14) trailing stop.
#
# Enters long when Bollinger bandwidth makes a new SQUEEZE_LOOKBACK low (a
# volatility squeeze) and price closes above the upper band; exits on an ATR(14)
# trailing stop or when the upper band falls back below the entry. 0.1% fees per
# trade. The R counterpart of examples/python/strategy_bollinger_squeeze.py,
# printing the same summary. Uses the checked-in examples/data/btcusdt-1d.csv
# dataset (pass a CSV path to override).
suppressPackageStartupMessages(library(wickra))
source("_common.R")

FEE <- 0.001
ATR_STOP_MULT <- 2.0
SQUEEZE_LOOKBACK <- 180

args <- commandArgs(trailingOnly = TRUE)
bars <- if (length(args) >= 1) load_ohlcv_csv(args[1]) else bundled_candles("btcusdt-1d.csv")

opens <- bars$open; highs <- bars$high; lows <- bars$low
closes <- bars$close; vols <- bars$volume; ts <- bars$timestamp
n_bars <- length(closes)

bb <- BollingerBands(20, 2.0); atr <- Atr(14)
in_pos <- FALSE; entry_price <- 0; stop_level <- 0
closed <- numeric(0); equity <- 1; equity_curve <- numeric(n_bars)
bw_window <- numeric(0)

for (i in seq_len(n_bars)) {
  band <- update(bb, closes[i])
  atr_value <- update(atr, opens[i], highs[i], lows[i], closes[i], vols[i], ts[i])
  price <- closes[i]
  equity_curve[i] <- if (in_pos) equity * (price / entry_price) else equity
  if (is.na(band[["middle"]]) || !is.finite(atr_value)) next

  middle <- band[["middle"]]
  if (abs(middle) <= 1e-12) next
  upper <- band[["upper"]]; lower <- band[["lower"]]
  bandwidth <- (upper - lower) / middle
  bw_window <- c(bw_window, bandwidth)
  if (length(bw_window) > SQUEEZE_LOOKBACK) {
    bw_window <- bw_window[(length(bw_window) - SQUEEZE_LOOKBACK + 1):length(bw_window)]
  }
  if (length(bw_window) < SQUEEZE_LOOKBACK) next
  min_bw <- min(bw_window)

  if (in_pos) {
    if (price < stop_level || upper < entry_price) {
      trade_ret <- price / entry_price - 1
      closed <- c(closed, trade_ret)
      equity <- equity * (1 + trade_ret) * (1 - FEE)
      in_pos <- FALSE
    }
  } else {
    is_new_low <- abs(bandwidth - min_bw) < 1e-12
    if (is_new_low && price > upper) {
      entry_price <- price; stop_level <- price - ATR_STOP_MULT * atr_value
      equity <- equity * (1 - FEE); in_pos <- TRUE
    }
  }
}

if (in_pos) {
  trade_ret <- closes[n_bars] / entry_price - 1
  closed <- c(closed, trade_ret)
  equity <- equity * (1 + trade_ret) * (1 - FEE)
}

print_summary("Bollinger Squeeze Breakout (1d, BTCUSDT)",
              closes[1], closes[n_bars], n_bars, closed, equity, equity_curve)
