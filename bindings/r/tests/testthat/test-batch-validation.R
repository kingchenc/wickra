# `batch()` forwards its columns straight into a native routine that takes its
# row count from the first column and indexes every other column with it. A
# shorter column was therefore read past its end, which segfaulted R from
# ordinary user code. These tests pin the guards that make that unreachable.

test_that("batch rejects columns of differing length", {
  atr <- Atr(3)
  n <- 8
  ok <- as.numeric(seq_len(n))
  expect_error(
    batch(atr, ok, ok, ok, ok, ok, as.numeric(seq_len(n - 1))),
    "same length"
  )
})

test_that("batch rejects a short timestamp column", {
  # The regression case: five full OHLCV columns and a three-element timestamp.
  atr <- Atr(14)
  n <- 200
  o <- as.numeric(seq_len(n))
  expect_error(batch(atr, o, o, o, o, o, as.numeric(0:2)), "same length")
})

test_that("batch rejects a non-numeric column", {
  expect_error(batch(Sma(3), c("a", "b", "c")), "must be numeric")
})

test_that("batch rejects being called with no columns", {
  expect_error(batch(Sma(3)), "at least one input column")
})

test_that("batch accepts an integer vector by coercing it", {
  # `1:8` is an INTSXP; the native routine reads doubles, so it must be coerced
  # rather than reinterpreted.
  expect_equal(batch(Sma(3), 1:8), batch(Sma(3), as.numeric(1:8)))
})

test_that("a matched multi-column batch still works", {
  atr <- Atr(3)
  n <- 10
  high <- 100 + seq_len(n)
  low <- 98 + seq_len(n)
  close <- 99 + seq_len(n)
  open <- close
  volume <- rep(1, n)
  timestamp <- as.numeric(seq_len(n))
  got <- batch(atr, open, high, low, close, volume, timestamp)
  expect_length(got, n)
  expect_true(all(is.na(got[1:2])))
  expect_false(any(is.na(got[3:n])))
})
