# One indicator per FFI archetype, exercised against the real native library.

test_that("scalar update returns the textbook value", {
  s <- Sma(3)
  v <- NA_real_
  for (x in c(1, 2, 3, 4, 5)) v <- update(s, x)
  expect_equal(v, 4)
})

test_that("warmup_period and is_ready report the warmup transition", {
  s <- Sma(3)
  expect_equal(warmup_period(s), 3L)
  expect_false(is_ready(s))
  update(s, 1)
  update(s, 2)
  expect_false(is_ready(s))
  update(s, 3)
  expect_true(is_ready(s))
  reset(s)
  expect_false(is_ready(s))
})

test_that("batch matches streaming", {
  input <- c(1, 2, 3, 4, 5, 6, 7, 8)
  stream <- Sma(3)
  want <- vapply(input, function(x) update(stream, x), numeric(1))
  got <- batch(Sma(3), input)
  expect_equal(got, want)
})

test_that("multi-output returns a named vector, NA during warmup", {
  m <- MacdIndicator(3, 6, 3)
  out <- update(m, 100)
  expect_true(all(is.na(out)))
  for (i in 1:30) out <- update(m, 100 + i)
  expect_false(any(is.na(out)))
  expect_named(out, c("macd", "signal", "histogram"))
})

test_that("bar builders return a matrix of completed bars", {
  rb <- RangeBars(2.0)
  total <- 0
  for (p in c(100, 101, 103, 104, 99, 96, 102, 108, 95, 110)) {
    total <- total + nrow(update(rb, p, p, p, p, 1, 0))
  }
  expect_gt(total, 0)
})

test_that("profile indicators return scalars plus a values vector", {
  vp <- VolumeProfile(10, 24)
  snap <- NULL
  for (i in 0:49) {
    price <- 100 + 5 * sin(i * 0.3)
    snap <- update(vp, price, price + 1, price - 1, price, 1000, i)
  }
  expect_false(is.null(snap))
  expect_gt(length(snap$values), 0)
})

test_that("array-input indicators consume vectors", {
  ob <- OrderBookImbalanceFull()
  v <- update(ob, c(99.9, 99.8, 99.7), c(5, 3, 2), c(100.1, 100.2, 100.3), c(1, 1, 1))
  expect_false(is.na(v))
})

test_that("reset returns the indicator to warmup", {
  s <- Sma(3)
  for (x in c(1, 2, 3)) update(s, x)
  reset(s)
  expect_true(is.na(update(s, 10)))
})

test_that("invalid parameters raise an error", {
  expect_error(Sma(0))
})
