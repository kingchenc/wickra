# Trend follower: enter long on a MACD histogram cross up, but only when ADX(14) > 20
# confirms a trend; exit when the histogram crosses back below zero.
library(wickra)
source("_common.R")

args <- commandArgs(trailingOnly = TRUE)
bars <- if (length(args) >= 1) load_ohlcv_csv(args[1]) else synthetic_candles(2000)

macd <- MacdIndicator(12, 26, 9); adx <- Adx(14)
returns <- numeric(0); trades <- 0L; in_pos <- FALSE; entry <- 0; prev_hist <- NA_real_
for (i in seq_len(nrow(bars))) {
  b <- bars[i, ]
  m <- update(macd, b$close)
  a <- update(adx, b$open, b$high, b$low, b$close, b$volume, b$timestamp)
  if (is.na(m[["macd"]]) || is.na(a[["adx"]])) next
  trending <- a[["adx"]] > 20
  if (!in_pos && trending && is.finite(prev_hist) && prev_hist <= 0 && m[["histogram"]] > 0) {
    in_pos <- TRUE; entry <- b$close; trades <- trades + 1L
  } else if (in_pos && m[["histogram"]] < 0) {
    returns <- c(returns, (b$close - entry) / entry); in_pos <- FALSE
  }
  prev_hist <- m[["histogram"]]
}
print_equity("MACD + ADX trend", summarize_equity(returns, trades))
