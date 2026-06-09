# Breakout: when Bollinger bandwidth is tight (a "squeeze") and price closes above
# the upper band, go long with an ATR(14) trailing stop.
library(wickra)
source("_common.R")

args <- commandArgs(trailingOnly = TRUE)
bars <- if (length(args) >= 1) load_ohlcv_csv(args[1]) else synthetic_candles(2000)

bb <- BollingerBands(20, 2.0); atr <- Atr(14)
returns <- numeric(0); trades <- 0L; in_pos <- FALSE; entry <- 0; stop <- 0
for (i in seq_len(nrow(bars))) {
  b <- bars[i, ]
  band <- update(bb, b$close)
  atr_value <- update(atr, b$open, b$high, b$low, b$close, b$volume, b$timestamp)
  if (is.na(band[["middle"]]) || !is.finite(atr_value)) next
  bandwidth <- if (band[["middle"]] != 0) (band[["upper"]] - band[["lower"]]) / band[["middle"]] else .Machine$double.xmax
  if (!in_pos && bandwidth < 0.06 && b$close > band[["upper"]]) {
    in_pos <- TRUE; entry <- b$close; stop <- b$close - 2 * atr_value; trades <- trades + 1L
  } else if (in_pos) {
    stop <- max(stop, b$close - 2 * atr_value)
    if (b$close < stop) { returns <- c(returns, (b$close - entry) / entry); in_pos <- FALSE }
  }
}
print_equity("Bollinger squeeze", summarize_equity(returns, trades))
