# Feed a synthetic price series through several indicators tick by tick (O(1) each).
library(wickra)
source("_common.R")

prices <- synthetic_prices(500)
sma <- Sma(20); ema <- Ema(20); rsi <- Rsi(14); macd <- MacdIndicator(12, 26, 9)

last_sma <- last_ema <- last_rsi <- NA_real_
last_macd <- NULL
for (price in prices) {
  last_sma <- update(sma, price)
  last_ema <- update(ema, price)
  last_rsi <- update(rsi, price)
  last_macd <- update(macd, price)
}

cat(sprintf("Streamed %d prices through SMA(20), EMA(20), RSI(14), MACD(12,26,9):\n", length(prices)))
cat(sprintf("  SMA  = %.4f\n", last_sma))
cat(sprintf("  EMA  = %.4f\n", last_ema))
cat(sprintf("  RSI  = %.4f\n", last_rsi))
cat(sprintf("  MACD = %.4f  signal=%.4f  hist=%.4f\n",
            last_macd[["macd"]], last_macd[["signal"]], last_macd[["histogram"]]))
