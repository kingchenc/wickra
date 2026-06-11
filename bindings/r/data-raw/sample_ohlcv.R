# Generates data/sample_ohlcv.rda — a deterministic, synthetic daily OHLCV
# series used by the examples, the getting-started vignette, and tests. It is a
# seeded random walk, NOT real market data.
#
# Regenerate (run from the R package root, bindings/r):
#   Rscript data-raw/sample_ohlcv.R

set.seed(42)
n <- 250L
dates <- seq(as.Date("2023-01-02"), by = "day", length.out = n)

# Random-walk close with a mild upward drift; derive OHLC around it.
returns <- rnorm(n, mean = 0.0004, sd = 0.012)
close <- round(100 * cumprod(1 + returns), 2)
open <- round(c(100, head(close, -1)) * (1 + rnorm(n, 0, 0.003)), 2)
high <- round(pmax(open, close) * (1 + abs(rnorm(n, 0, 0.004))), 2)
low <- round(pmin(open, close) * (1 - abs(rnorm(n, 0, 0.004))), 2)
volume <- round(1e6 * exp(rnorm(n, 0, 0.3)))

sample_ohlcv <- data.frame(
  date = dates,
  open = open,
  high = high,
  low = low,
  close = close,
  volume = volume
)

save(sample_ohlcv, file = "data/sample_ohlcv.rda", compress = "xz", version = 2)
cat(sprintf("wrote data/sample_ohlcv.rda: %d rows x %d cols\n",
            nrow(sample_ohlcv), ncol(sample_ohlcv)))
