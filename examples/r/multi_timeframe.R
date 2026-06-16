# Resample a 1-minute series into higher timeframes and run an indicator per timeframe.
library(wickra)
source("_common.R")

resample <- function(bars, factor) {
  if (factor <= 1) return(bars)
  # Native Resampler: bucket by an absolute timeframe (synthetic bars step 60000 ms,
  # so factor minutes == factor*60000 ms). update() yields NA until a bucket closes;
  # flush() returns the final partial bucket. No hand-written bucketing.
  r <- Resampler(factor * 60000)
  out <- list()
  for (i in seq_len(nrow(bars))) {
    c <- update(r, bars$open[i], bars$high[i], bars$low[i], bars$close[i],
                bars$volume[i], bars$timestamp[i])
    if (!is.na(c[1])) out[[length(out) + 1L]] <- c
  }
  f <- flush(r)
  if (!is.null(f)) out[[length(out) + 1L]] <- f
  m <- do.call(rbind, out)
  data.frame(open = m[, 1], high = m[, 2], low = m[, 3], close = m[, 4],
             volume = m[, 5], timestamp = m[, 6])
}

one_minute <- synthetic_candles(1200, step_ms = 60000)
cat("EMA(20) of close across timeframes (resampled from 1-minute bars):\n")
for (factor in c(1, 5, 15)) {
  bars <- resample(one_minute, factor)
  ema <- Ema(20); last <- NA_real_
  for (cl in bars$close) last <- update(ema, cl)
  cat(sprintf("  %2dm: %5d bars  EMA(20) last = %.4f\n", factor, nrow(bars), last))
}
