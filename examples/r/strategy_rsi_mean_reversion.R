# Mean reversion: go long when RSI(14) drops below 30, exit when it recovers above 50.
library(wickra)
source("_common.R")

args <- commandArgs(trailingOnly = TRUE)
bars <- if (length(args) >= 1) load_ohlcv_csv(args[1]) else synthetic_candles(2000)

rsi <- Rsi(14)
returns <- numeric(0); trades <- 0L; in_pos <- FALSE; entry <- 0
for (i in seq_len(nrow(bars))) {
  cl <- bars$close[i]
  value <- update(rsi, cl)
  if (!is.finite(value)) next
  if (!in_pos && value < 30) {
    in_pos <- TRUE; entry <- cl; trades <- trades + 1L
  } else if (in_pos && value > 50) {
    returns <- c(returns, (cl - entry) / entry); in_pos <- FALSE
  }
}
print_equity("RSI mean-reversion", summarize_equity(returns, trades))
