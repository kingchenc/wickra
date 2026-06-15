#' wickra: streaming-first technical indicators
#'
#' R bindings for the Wickra technical-analysis library over its C ABI hub. Each
#' indicator is a constructor (for example [Sma()], [Rsi()], [MacdIndicator()])
#' returning a `wickra_indicator` object; feed it one observation at a time with
#' [update()], run a whole series in one call with [batch()], and clear its
#' state with [reset()]. The native handle is freed automatically when the object
#' is garbage-collected.
#'
#' @keywords internal
#' @importFrom stats update
"_PACKAGE"

#' Update an indicator with one observation
#'
#' @param object A `wickra_indicator` created by an indicator constructor.
#' @param ... The observation: a single value for scalar indicators, the OHLCV
#'   fields plus a timestamp for candle indicators, or two values for pairwise
#'   indicators.
#' @return The indicator value: a numeric scalar; a named numeric vector for
#'   multi-output indicators (`NA` during warmup); a matrix of completed bars for
#'   bar builders; or a list / numeric vector for profile indicators
#'   (`NULL` during warmup).
#' @examples
#' sma <- Sma(3)
#' for (x in c(1, 2, 3, 4, 5)) v <- update(sma, x)
#' v # 4
#' @export
update.wickra_indicator <- function(object, ...) {
  args <- list(object$ptr, ...)
  if (!is.na(object$values_cap)) {
    args <- c(args, object$values_cap)
  }
  do.call(".Call", c(list(paste0("wk_", object$prefix, "_update")), args,
                     list(PACKAGE = "wickra")))
}

#' Run an indicator over a whole series in one call
#'
#' Available for scalar indicators. The result is identical to feeding the same
#' inputs through [update()] one at a time, with `NA` at warmup positions.
#'
#' @param object A `wickra_indicator`.
#' @param ... The input vector(s).
#' @return A numeric vector the same length as the input.
#' @examples
#' batch(Sma(3), c(1, 2, 3, 4, 5)) # NA NA 2 3 4
#' @export
batch <- function(object, ...) {
  UseMethod("batch")
}

#' @rdname batch
#' @export
batch.wickra_indicator <- function(object, ...) {
  do.call(".Call", c(list(paste0("wk_", object$prefix, "_batch"), object$ptr),
                     list(...), list(PACKAGE = "wickra")))
}

#' Reset an indicator to its warmup state
#'
#' @param object A `wickra_indicator`.
#' @return The indicator, invisibly.
#' @examples
#' sma <- Sma(3)
#' update(sma, 1)
#' reset(sma)
#' @export
reset <- function(object) {
  UseMethod("reset")
}

#' @rdname reset
#' @export
reset.wickra_indicator <- function(object) {
  .Call(paste0("wk_", object$prefix, "_reset"), object$ptr, PACKAGE = "wickra")
  invisible(object)
}

#' Number of updates an indicator needs before it produces a value
#'
#' Not available for the alt-chart bar builders ([RenkoBars()], [KagiBars()],
#' [PointAndFigureBars()], …), which have no warmup.
#'
#' @param object A `wickra_indicator`.
#' @return A single integer: the warmup period.
#' @examples
#' warmup_period(Sma(14)) # 14
#' @export
warmup_period <- function(object) {
  UseMethod("warmup_period")
}

#' @rdname warmup_period
#' @export
warmup_period.wickra_indicator <- function(object) {
  .Call(paste0("wk_", object$prefix, "_warmup_period"), object$ptr, PACKAGE = "wickra")
}

#' Whether an indicator has consumed enough input to emit a value
#'
#' Not available for the alt-chart bar builders, which have no warmup.
#'
#' @param object A `wickra_indicator`.
#' @return A single logical.
#' @examples
#' sma <- Sma(3)
#' is_ready(sma) # FALSE
#' for (x in c(1, 2, 3)) update(sma, x)
#' is_ready(sma) # TRUE
#' @export
is_ready <- function(object) {
  UseMethod("is_ready")
}

#' @rdname is_ready
#' @export
is_ready.wickra_indicator <- function(object) {
  .Call(paste0("wk_", object$prefix, "_is_ready"), object$ptr, PACKAGE = "wickra")
}

#' Canonical name of an indicator
#'
#' Returns the stable, human-readable name of the indicator (the same name
#' reported by every other Wickra binding), e.g. `"SMA"` for [Sma()].
#'
#' @param object A `wickra_indicator`.
#' @return A single character string.
#' @examples
#' name(Sma(14)) # "SMA"
#' @export
name <- function(object) {
  UseMethod("name")
}

#' @rdname name
#' @export
name.wickra_indicator <- function(object) {
  .Call(paste0("wk_", object$prefix, "_name"), object$ptr, PACKAGE = "wickra")
}

#' Push a trade tick into a tick aggregator
#'
#' Feeds one trade tick to a [TickAggregator()] and returns the candles it
#' closed as a numeric matrix with columns `open`, `high`, `low`, `close`,
#' `volume`, `timestamp` (zero rows while the open bar merely grows).
#'
#' @param object A `wickra_indicator` created by [TickAggregator()].
#' @param price Trade price.
#' @param size Trade size (volume).
#' @param timestamp Trade timestamp, in the same unit as the aggregator bucket.
#' @return A numeric matrix with six named columns (possibly zero rows).
#' @examples
#' agg <- TickAggregator(1000)
#' push(agg, 100, 1, 0)
#' push(agg, 102, 1, 1000) # closes the first bucket
#' @export
push <- function(object, price, size, timestamp) {
  UseMethod("push")
}

#' @rdname push
#' @export
push.wickra_indicator <- function(object, price, size, timestamp) {
  out <- .Call(
    paste0("wk_", object$prefix, "_push"),
    object$ptr, price, size, timestamp,
    PACKAGE = "wickra"
  )
  colnames(out) <- c("open", "high", "low", "close", "volume", "timestamp")
  out
}
