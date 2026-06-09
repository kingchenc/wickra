# Resample a 1-minute series into higher timeframes and run an indicator per timeframe.
library(wickra)
source("_common.R")

resample <- function(bars, factor) {
  if (factor <= 1) return(bars)
  idx <- seq(1, nrow(bars), by = factor)
  do.call(rbind, lapply(idx, function(i) {
    j <- min(i + factor - 1, nrow(bars))
    data.frame(open = bars$open[i], high = max(bars$high[i:j]),
               low = min(bars$low[i:j]), close = bars$close[j],
               volume = sum(bars$volume[i:j]), timestamp = bars$timestamp[i])
  }))
}

one_minute <- synthetic_candles(1200, step_ms = 60000)
cat("EMA(20) of close across timeframes (resampled from 1-minute bars):\n")
for (factor in c(1, 5, 15)) {
  bars <- resample(one_minute, factor)
  ema <- Ema(20); last <- NA_real_
  for (cl in bars$close) last <- update(ema, cl)
  cat(sprintf("  %2dm: %5d bars  EMA(20) last = %.4f\n", factor, nrow(bars), last))
}
