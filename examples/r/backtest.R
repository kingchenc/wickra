# Compute a basket of indicators over an OHLCV series and print a summary.
# Pass a CSV path (timestamp,open,high,low,close,volume) or run on synthetic data.
library(wickra)
source("_common.R")

args <- commandArgs(trailingOnly = TRUE)
if (length(args) >= 1) {
  source_name <- args[1]; bars <- load_ohlcv_csv(args[1])
} else {
  source_name <- "synthetic"; bars <- synthetic_candles(1000)
}

cat(sprintf("Backtest over %d bars (%s):\n", nrow(bars), source_name))
sma <- Sma(20); ema <- Ema(50); rsi <- Rsi(14); atr <- Atr(14)
last_sma <- last_ema <- last_rsi <- last_atr <- NA_real_
oversold <- 0L
for (i in seq_len(nrow(bars))) {
  b <- bars[i, ]
  last_sma <- update(sma, b$close)
  last_ema <- update(ema, b$close)
  last_rsi <- update(rsi, b$close)
  last_atr <- update(atr, b$open, b$high, b$low, b$close, b$volume, b$timestamp)
  if (is.finite(last_rsi) && last_rsi < 30) oversold <- oversold + 1L
}
cat(sprintf("  SMA(20) last = %.4f\n", last_sma))
cat(sprintf("  EMA(50) last = %.4f\n", last_ema))
cat(sprintf("  RSI(14) last = %.4f  (%d oversold bars)\n", last_rsi, oversold))
cat(sprintf("  ATR(14) last = %.4f\n", last_atr))
