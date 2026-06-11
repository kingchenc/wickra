#' Synthetic daily OHLCV sample series
#'
#' A deterministic, synthetic daily OHLCV (open / high / low / close / volume)
#' price series for use in the examples, the *Getting started* vignette, and
#' tests. It is a seeded random walk, **not** real market data. Regenerate with
#' `data-raw/sample_ohlcv.R`.
#'
#' @format A data frame with 250 rows and 6 columns:
#' \describe{
#'   \item{date}{Trading date (`Date`).}
#'   \item{open}{Opening price.}
#'   \item{high}{Session high.}
#'   \item{low}{Session low.}
#'   \item{close}{Closing price.}
#'   \item{volume}{Traded volume.}
#' }
"sample_ohlcv"
