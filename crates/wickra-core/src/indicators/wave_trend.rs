//! Wave Trend Oscillator (`LazyBear`).

use crate::error::{Error, Result};
use crate::indicators::ema::Ema;
use crate::indicators::sma::Sma;
use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// Wave Trend Oscillator output: the two lines `wt1` (the oscillator) and
/// `wt2` (the signal SMA).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WaveTrendOutput {
    /// `wt1` — the smoothed channel index.
    pub wt1: f64,
    /// `wt2` — the SMA-smoothed signal line.
    pub wt2: f64,
}

/// `LazyBear`'s Wave Trend Oscillator — a two-line momentum gauge built from
/// the typical price and three cascaded EMAs.
///
/// For each candle let `ap_t = (high + low + close) / 3`:
///
/// ```text
/// esa_t = EMA(ap, channel_period)
/// d_t   = EMA(|ap − esa|, channel_period)
/// ci_t  = (ap_t − esa_t) / (0.015 * d_t)
/// wt1_t = EMA(ci, average_period)
/// wt2_t = SMA(wt1, signal_period)
/// ```
///
/// Bullish trigger: `wt1` crossing above `wt2` from an oversold region
/// (typically `wt1 < -60`); bearish trigger: the mirror crossover above
/// `+60`. The indicator is mean-reverting around zero, so it is most useful
/// at extremes.
///
/// The canonical `LazyBear` defaults are
/// `(channel_period = 10, average_period = 21, signal_period = 4)`; warmup is
/// `channel_period + average_period + signal_period − 2`.
///
/// Non-finite `d` (a zero-volatility seed where the absolute-deviation EMA
/// has not yet recorded any movement) collapses the channel index to zero.
///
/// # Example
///
/// ```
/// use wickra_core::{Candle, Indicator, WaveTrend};
///
/// let mut indicator = WaveTrend::classic().unwrap();
/// let mut last = None;
/// for i in 0..80 {
///     let base = 100.0 + f64::from(i);
///     let candle =
///         Candle::new(base, base + 2.0, base - 2.0, base + 1.0, 10.0, i64::from(i)).unwrap();
///     last = indicator.update(candle);
/// }
/// assert!(last.is_some());
/// ```
#[derive(Debug, Clone)]
pub struct WaveTrend {
    channel_period: usize,
    average_period: usize,
    signal_period: usize,
    esa: Ema,
    dev_ema: Ema,
    tci: Ema,
    signal: Sma,
    last: Option<WaveTrendOutput>,
}

impl WaveTrend {
    /// Construct a new Wave Trend Oscillator with explicit periods.
    ///
    /// # Errors
    ///
    /// Returns [`Error::PeriodZero`] if any period is `0`.
    pub fn new(channel_period: usize, average_period: usize, signal_period: usize) -> Result<Self> {
        if channel_period == 0 || average_period == 0 || signal_period == 0 {
            return Err(Error::PeriodZero);
        }
        Ok(Self {
            channel_period,
            average_period,
            signal_period,
            esa: Ema::new(channel_period)?,
            dev_ema: Ema::new(channel_period)?,
            tci: Ema::new(average_period)?,
            signal: Sma::new(signal_period)?,
            last: None,
        })
    }

    /// `LazyBear`'s classic Wave Trend: `(channel = 10, average = 21, signal = 4)`.
    ///
    /// # Errors
    ///
    /// None in practice — all periods are non-zero.
    pub fn classic() -> Result<Self> {
        Self::new(10, 21, 4)
    }

    /// Configured `(channel_period, average_period, signal_period)`.
    pub const fn periods(&self) -> (usize, usize, usize) {
        (self.channel_period, self.average_period, self.signal_period)
    }

    /// Current value if available.
    pub const fn value(&self) -> Option<WaveTrendOutput> {
        self.last
    }
}

impl Indicator for WaveTrend {
    type Input = Candle;
    type Output = WaveTrendOutput;

    fn update(&mut self, candle: Candle) -> Option<WaveTrendOutput> {
        let ap = (candle.high + candle.low + candle.close) / 3.0;

        // Stage 1: ESA = EMA(ap, channel_period). Must be ready before we
        // can compute the absolute deviation EMA against it.
        let esa = self.esa.update(ap)?;

        // Stage 2: deviation EMA tracks |ap - esa|.
        let d = self.dev_ema.update((ap - esa).abs())?;

        // Stage 3: channel index. On a perfectly flat market `(ap - esa)`
        // and `d` are both within an ULP or two of zero; their ratio is
        // mathematically indeterminate and would otherwise produce garbage
        // like `-66.67 = -1 / 0.015`. Treat any sub-ULP deviation as zero,
        // matching pandas-ta's flat-market behaviour. The threshold scales
        // with `esa` so it adapts to any price magnitude.
        let flat_tol = esa.abs().max(1.0) * 16.0 * f64::EPSILON;
        let ci = if d <= flat_tol {
            0.0
        } else {
            (ap - esa) / (0.015 * d)
        };

        // Stage 4: wt1 = EMA(ci, average_period).
        let wt1 = self.tci.update(ci)?;

        // Stage 5: wt2 = SMA(wt1, signal_period).
        let wt2 = self.signal.update(wt1)?;

        let out = WaveTrendOutput { wt1, wt2 };
        self.last = Some(out);
        Some(out)
    }

    fn reset(&mut self) {
        self.esa.reset();
        self.dev_ema.reset();
        self.tci.reset();
        self.signal.reset();
        self.last = None;
    }

    fn warmup_period(&self) -> usize {
        // EMA(esa) first emits at input `channel_period`; the second EMA
        // (deviation) takes its input from the same bar and emits at the
        // same `channel_period`-th input (it can already start computing
        // |ap - esa| as soon as esa is ready, and the EMA-of-EMA construction
        // uses the inner EMA's first valid output as its first input —
        // however because we gate via `?` on both stages, the second EMA's
        // first valid input is at the channel_period-th input, then itself
        // needs channel_period - 1 more inputs to warm... but our Ema
        // implementation seeds via SMA on the first `period` inputs, so the
        // dev_ema needs channel_period inputs of |ap - esa| values.
        //
        // Actually: esa emits at input `channel_period` (1-based). dev_ema
        // gets fed starting at that input, and needs `channel_period` inputs
        // of its own to first emit: at the `2 * channel_period - 1`-th input
        // dev_ema is ready (it has consumed channel_period inputs starting
        // from the channel_period-th). tci then needs `average_period`
        // inputs of `ci`, so it's ready at `2 * channel_period - 1 +
        // average_period - 1`. Signal needs `signal_period` inputs of wt1
        // → ready at `2 * channel_period - 1 + average_period - 1 +
        // signal_period - 1` = `2 * channel_period + average_period +
        // signal_period - 3`.
        2 * self.channel_period + self.average_period + self.signal_period - 3
    }

    fn is_ready(&self) -> bool {
        self.last.is_some()
    }

    fn name(&self) -> &'static str {
        "WaveTrend"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;

    fn candle(h: f64, l: f64, c: f64, ts: i64) -> Candle {
        Candle::new(c, h, l, c, 1.0, ts).unwrap()
    }

    #[test]
    fn rejects_zero_period() {
        assert!(matches!(WaveTrend::new(0, 21, 4), Err(Error::PeriodZero)));
        assert!(matches!(WaveTrend::new(10, 0, 4), Err(Error::PeriodZero)));
        assert!(matches!(WaveTrend::new(10, 21, 0), Err(Error::PeriodZero)));
    }

    #[test]
    fn accessors_and_metadata() {
        let mut w = WaveTrend::classic().unwrap();
        assert_eq!(w.periods(), (10, 21, 4));
        assert_eq!(w.name(), "WaveTrend");
        // 2 * 10 + 21 + 4 - 3 = 42.
        assert_eq!(w.warmup_period(), 42);
        assert!(w.value().is_none());
        let candles: Vec<Candle> = (0..80_i64)
            .map(|i| {
                let p = 100.0 + ((i as f64) * 0.3).sin() * 5.0;
                candle(p + 1.0, p - 1.0, p, i)
            })
            .collect();
        for c in &candles {
            w.update(*c);
        }
        assert!(w.value().is_some());
    }

    #[test]
    fn first_emission_at_warmup_period() {
        let candles: Vec<Candle> = (0..60_i64)
            .map(|i| {
                let p = 100.0 + ((i as f64) * 0.25).sin() * 6.0;
                candle(p + 1.0, p - 1.0, p, i)
            })
            .collect();
        let mut w = WaveTrend::new(5, 8, 3).unwrap();
        let warmup = 2 * 5 + 8 + 3 - 3; // 18
        assert_eq!(w.warmup_period(), warmup);
        let out = w.batch(&candles);
        for v in out.iter().take(warmup - 1) {
            assert!(v.is_none());
        }
        assert!(out[warmup - 1].is_some());
    }

    #[test]
    fn constant_series_yields_zero_lines() {
        // Flat market: every ap equals esa within an ULP, so the
        // flat-tolerance guard collapses ci to 0 and both lines remain at 0.
        let candles: Vec<Candle> = (0..80_i64).map(|i| candle(10.0, 10.0, 10.0, i)).collect();
        let mut w = WaveTrend::new(5, 8, 3).unwrap();
        let last = w.batch(&candles).into_iter().flatten().last().unwrap();
        assert_eq!(last.wt1, 0.0);
        assert_eq!(last.wt2, 0.0);
    }

    #[test]
    fn pure_uptrend_is_positive() {
        let candles: Vec<Candle> = (0..120_i64)
            .map(|i| {
                let base = 100.0 + (i as f64) * 0.5;
                candle(base + 1.0, base - 0.5, base + 0.5, i)
            })
            .collect();
        let mut w = WaveTrend::classic().unwrap();
        let last = w.batch(&candles).into_iter().flatten().last().unwrap();
        assert!(
            last.wt1 > 0.0,
            "uptrend wt1 should be positive, got {}",
            last.wt1
        );
        assert!(
            last.wt2 > 0.0,
            "uptrend wt2 should be positive, got {}",
            last.wt2
        );
    }

    #[test]
    fn pure_downtrend_is_negative() {
        let candles: Vec<Candle> = (0..120_i64)
            .map(|i| {
                let base = 200.0 - (i as f64) * 0.5;
                candle(base + 1.0, base - 0.5, base - 0.5, i)
            })
            .collect();
        let mut w = WaveTrend::classic().unwrap();
        let last = w.batch(&candles).into_iter().flatten().last().unwrap();
        assert!(last.wt1 < 0.0);
        assert!(last.wt2 < 0.0);
    }

    #[test]
    fn outputs_remain_finite() {
        let candles: Vec<Candle> = (0..200_i64)
            .map(|i| {
                let p = 100.0 + ((i as f64) * 0.3).sin() * 8.0;
                candle(p + 2.0, p - 2.0, p, i)
            })
            .collect();
        let mut w = WaveTrend::classic().unwrap();
        for v in w.batch(&candles).into_iter().flatten() {
            assert!(v.wt1.is_finite() && v.wt2.is_finite());
        }
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0..120_i64)
            .map(|i| {
                let p = 100.0 + ((i as f64) * 0.27).sin() * 6.0;
                candle(p + 1.5, p - 1.5, p, i)
            })
            .collect();
        let mut a = WaveTrend::classic().unwrap();
        let mut b = WaveTrend::classic().unwrap();
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|c| b.update(*c)).collect::<Vec<_>>()
        );
    }

    #[test]
    fn reset_clears_state() {
        let candles: Vec<Candle> = (0..80_i64).map(|i| candle(11.0, 9.0, 10.0, i)).collect();
        let mut w = WaveTrend::classic().unwrap();
        w.batch(&candles);
        assert!(w.is_ready());
        w.reset();
        assert!(!w.is_ready());
        assert_eq!(w.update(candles[0]), None);
    }
}
