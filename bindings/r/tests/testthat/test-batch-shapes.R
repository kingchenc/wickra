# `batch()` was wired only for the scalar indicators, so 158 of the 514 had a
# native batch R could not call -- 92 of them multi-output, whose per-bar inputs
# are ordinary equal-length columns and whose result is an `n x k` matrix.
#
# The bar-builder `update` shim had the overflow the other bindings had, and in C
# it was undefined behaviour rather than a checked error: it read a 64-element
# stack array up to the count the builder reported.

test_that("a multi-output batch matches feeding the same bars one at a time", {
  n <- 40
  close <- 100 + seq_len(n)
  high <- close + 1
  low <- close - 1
  ts <- as.numeric(seq_len(n))

  got <- batch(Adx(7), close, high, low, close, rep(10, n), ts)

  expect_true(is.matrix(got))
  expect_equal(dim(got), c(n, 3L))
  expect_equal(colnames(got), c("plus_di", "minus_di", "adx"))

  streamed <- Adx(7)
  emitted <- 0L
  for (i in seq_len(n)) {
    want <- update(streamed, close[i], high[i], low[i], close[i], 10, ts[i])
    if (all(is.na(want))) {
      expect_true(all(is.nan(got[i, ]) | is.na(got[i, ])))
      next
    }
    emitted <- emitted + 1L
    expect_equal(unname(got[i, ]), unname(want), tolerance = 1e-12)
  }
  expect_gt(emitted, 0L)
})

test_that("a multi-output batch reports NaN before the indicator is warm", {
  n <- 30
  x <- 100 + seq_len(n)
  got <- batch(BollingerBands(20, 2), x)
  expect_true(is.matrix(got))
  # The first 19 rows precede the first value.
  expect_true(all(is.nan(got[1:19, ])))
  expect_false(any(is.nan(got[20, ])))
})

test_that("a bar builder returns every bar of a large move", {
  # A box size of 1 turns a 500-point move into 500 bricks, far more than the
  # 64-element buffer the shim passes.
  renko <- RenkoBars(1)
  update(renko, 100, 100, 100, 100, 1, 0)
  bricks <- update(renko, 600, 600, 600, 600, 1, 1)

  expect_true(is.matrix(bricks))
  expect_gt(nrow(bricks), 64L)
  # One consecutive ladder: nothing dropped or duplicated at the boundary
  # between the buffer and the drained remainder.
  expect_equal(bricks[, "close"] - bricks[, "open"], rep(1, nrow(bricks)))
  expect_equal(bricks[-1, "open"], bricks[-nrow(bricks), "close"])
})
