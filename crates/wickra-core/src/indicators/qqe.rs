//! QQE — Quantitative Qualitative Estimation.

use crate::error::{Error, Result};
use crate::indicators::ema::Ema;
use crate::indicators::rsi::Rsi;
use crate::traits::Indicator;

/// One QQE reading: the smoothed RSI and its volatility-trailing line.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct QqeOutput {
    /// The EMA-smoothed RSI (the fast QQE line).
    pub rsi_ma: f64,
    /// The trailing line (the slow QQE line): an ATR-of-RSI trailing stop that
    /// the smoothed RSI rides above in an uptrend and below in a downtrend.
    pub trailing_line: f64,
}

/// QQE — Quantitative Qualitative Estimation (Igor Livshin).
///
/// QQE smooths the RSI, then builds an "ATR of the RSI" trailing stop around it.
/// Crossovers of the smoothed RSI and that trailing line give cleaner momentum
/// signals than the raw RSI:
///
/// ```text
/// rsi_ma   = EMA(RSI(price, rsi_period), smoothing)
/// atr_rsi  = |rsi_ma − rsi_ma_prev|
/// ma_atr   = EMA(atr_rsi, 2·rsi_period − 1)        // Wilder length
/// dar      = EMA(ma_atr, 2·rsi_period − 1) · factor // smoothed band width
///
/// long_band  = (rsi_ma_prev > long_band_prev  && rsi_ma > long_band_prev)
///              ? max(long_band_prev,  rsi_ma − dar) : rsi_ma − dar
/// short_band = (rsi_ma_prev < short_band_prev && rsi_ma < short_band_prev)
///              ? min(short_band_prev, rsi_ma + dar) : rsi_ma + dar
/// trend      = cross-up of short_band → +1, cross-down of long_band → −1, else hold
/// trailing   = trend == +1 ? long_band : short_band
/// ```
///
/// The trailing line ratchets in the trend direction (only ever tightening until
/// the smoothed RSI crosses it), exactly like a [`SuperTrend`](crate::SuperTrend)
/// on the RSI. Livshin's defaults are `rsi_period = 14`, `smoothing = 5`,
/// `factor = 4.236`.
///
/// # Example
///
/// ```
/// use wickra_core::{Indicator, Qqe};
///
/// let mut qqe = Qqe::new(14, 5, 4.236).unwrap();
/// let mut last = None;
/// for i in 0..200 {
///     last = qqe.update(100.0 + (f64::from(i) * 0.1).sin() * 8.0);
/// }
/// assert!(last.is_some());
/// ```
#[derive(Debug, Clone)]
pub struct Qqe {
    rsi: Rsi,
    rsi_ma: Ema,
    ma_atr: Ema,
    dar_ema: Ema,
    factor: f64,
    prev_rsi_ma: Option<f64>,
    bands: Option<(f64, f64, i8)>, // (long_band, short_band, trend)
    last_value: Option<QqeOutput>,
}

impl Qqe {
    /// Construct a QQE with the RSI period, RSI smoothing, and band `factor`.
    ///
    /// # Errors
    ///
    /// Returns [`Error::PeriodZero`] if `rsi_period` or `smoothing` is `0`, or
    /// [`Error::InvalidPeriod`] if `factor` is non-finite or not positive.
    pub fn new(rsi_period: usize, smoothing: usize, factor: f64) -> Result<Self> {
        if rsi_period == 0 || smoothing == 0 {
            return Err(Error::PeriodZero);
        }
        if !factor.is_finite() || factor <= 0.0 {
            return Err(Error::InvalidPeriod {
                message: "QQE factor must be a finite positive value",
            });
        }
        let wilders = 2 * rsi_period - 1;
        Ok(Self {
            rsi: Rsi::new(rsi_period)?,
            rsi_ma: Ema::new(smoothing)?,
            ma_atr: Ema::new(wilders)?,
            dar_ema: Ema::new(wilders)?,
            factor,
            prev_rsi_ma: None,
            bands: None,
            last_value: None,
        })
    }

    /// Configured band factor.
    pub const fn factor(&self) -> f64 {
        self.factor
    }

    /// Current value if available.
    pub const fn value(&self) -> Option<QqeOutput> {
        self.last_value
    }
}

impl Indicator for Qqe {
    type Input = f64;
    type Output = QqeOutput;

    fn update(&mut self, price: f64) -> Option<QqeOutput> {
        let rsi = self.rsi.update(price)?;
        let rsi_ma = self.rsi_ma.update(rsi)?;

        let Some(prev_ma) = self.prev_rsi_ma else {
            self.prev_rsi_ma = Some(rsi_ma);
            return None;
        };
        let atr_rsi = (rsi_ma - prev_ma).abs();
        self.prev_rsi_ma = Some(rsi_ma);

        let ma_atr = self.ma_atr.update(atr_rsi)?;
        let dar = self.dar_ema.update(ma_atr)? * self.factor;

        let new_long = rsi_ma - dar;
        let new_short = rsi_ma + dar;

        let (long_band, short_band, trend) = match self.bands {
            Some((lb_prev, sb_prev, tr_prev)) => {
                let lb = if prev_ma > lb_prev && rsi_ma > lb_prev {
                    lb_prev.max(new_long)
                } else {
                    new_long
                };
                let sb = if prev_ma < sb_prev && rsi_ma < sb_prev {
                    sb_prev.min(new_short)
                } else {
                    new_short
                };
                let tr = if prev_ma <= sb_prev && rsi_ma > sb_prev {
                    1
                } else if prev_ma >= lb_prev && rsi_ma < lb_prev {
                    -1
                } else {
                    tr_prev
                };
                (lb, sb, tr)
            }
            None => (new_long, new_short, 1),
        };
        self.bands = Some((long_band, short_band, trend));

        let trailing_line = if trend == 1 { long_band } else { short_band };
        let out = QqeOutput {
            rsi_ma,
            trailing_line,
        };
        self.last_value = Some(out);
        Some(out)
    }

    fn reset(&mut self) {
        self.rsi.reset();
        self.rsi_ma.reset();
        self.ma_atr.reset();
        self.dar_ema.reset();
        self.prev_rsi_ma = None;
        self.bands = None;
        self.last_value = None;
    }

    fn warmup_period(&self) -> usize {
        // RSI (rsi_period + 1) -> rsi_ma EMA -> one bar for the first atr_rsi ->
        // ma_atr EMA -> dar EMA. Expressed via the component warmups so it stays
        // correct if those change.
        self.rsi.warmup_period()
            + self.rsi_ma.warmup_period()
            + self.ma_atr.warmup_period()
            + self.dar_ema.warmup_period()
            - 2
    }

    fn is_ready(&self) -> bool {
        self.last_value.is_some()
    }

    fn name(&self) -> &'static str {
        "QQE"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;
    use approx::assert_relative_eq;

    /// Independent reference replaying the full QQE recurrence.
    fn naive(
        prices: &[f64],
        rsi_period: usize,
        smoothing: usize,
        factor: f64,
    ) -> Vec<Option<QqeOutput>> {
        let mut rsi = Rsi::new(rsi_period).unwrap();
        let mut rsi_ma = Ema::new(smoothing).unwrap();
        let wilders = 2 * rsi_period - 1;
        let mut ma_atr = Ema::new(wilders).unwrap();
        let mut dar_ema = Ema::new(wilders).unwrap();
        let mut prev_ma: Option<f64> = None;
        let mut bands: Option<(f64, f64, i8)> = None;
        let mut out = Vec::with_capacity(prices.len());
        for &p in prices {
            let v = (|| {
                let r = rsi.update(p)?;
                let m = rsi_ma.update(r)?;
                let Some(pm) = prev_ma else {
                    prev_ma = Some(m);
                    return None;
                };
                let atr = (m - pm).abs();
                prev_ma = Some(m);
                let ma = ma_atr.update(atr)?;
                let dar = dar_ema.update(ma)? * factor;
                let nl = m - dar;
                let ns = m + dar;
                let (lb, sb, tr) = match bands {
                    Some((lbp, sbp, trp)) => {
                        let lb = if pm > lbp && m > lbp { lbp.max(nl) } else { nl };
                        let sb = if pm < sbp && m < sbp { sbp.min(ns) } else { ns };
                        let tr = if pm <= sbp && m > sbp {
                            1
                        } else if pm >= lbp && m < lbp {
                            -1
                        } else {
                            trp
                        };
                        (lb, sb, tr)
                    }
                    None => (nl, ns, 1),
                };
                bands = Some((lb, sb, tr));
                Some(QqeOutput {
                    rsi_ma: m,
                    trailing_line: if tr == 1 { lb } else { sb },
                })
            })();
            out.push(v);
        }
        out
    }

    #[test]
    fn rejects_bad_params() {
        assert!(matches!(Qqe::new(0, 5, 4.236), Err(Error::PeriodZero)));
        assert!(matches!(Qqe::new(14, 0, 4.236), Err(Error::PeriodZero)));
        assert!(matches!(
            Qqe::new(14, 5, 0.0),
            Err(Error::InvalidPeriod { .. })
        ));
        assert!(matches!(
            Qqe::new(14, 5, f64::NAN),
            Err(Error::InvalidPeriod { .. })
        ));
    }

    /// Cover the const accessors `factor` + `value` and the Indicator-impl
    /// `name`. `warmup_period` is covered by `first_emission_matches_warmup`.
    #[test]
    fn accessors_and_metadata() {
        let qqe = Qqe::new(14, 5, 4.236).unwrap();
        assert_relative_eq!(qqe.factor(), 4.236, epsilon = 1e-12);
        assert_eq!(qqe.value(), None);
        assert_eq!(qqe.name(), "QQE");
    }

    #[test]
    fn first_emission_matches_warmup() {
        // A long trend-up-then-down series exercises both trend flips and the
        // band tighten/reset branches.
        let prices: Vec<f64> = (0..200)
            .map(|i| 100.0 + (f64::from(i) * 0.06).sin() * 20.0)
            .collect();
        let mut qqe = Qqe::new(14, 5, 4.236).unwrap();
        let out = qqe.batch(&prices);
        let warmup = qqe.warmup_period();
        for (i, v) in out.iter().enumerate().take(warmup - 1) {
            assert!(v.is_none(), "index {i} must be None during warmup");
        }
        assert!(
            out[warmup - 1].is_some(),
            "first value at warmup_period - 1"
        );
    }

    #[test]
    fn matches_naive_over_full_cycle() {
        // Up, range, and down phases so every band/trend branch is traversed.
        let prices: Vec<f64> = (0..220)
            .map(|i| {
                let t = f64::from(i);
                100.0 + (t * 0.05).sin() * 18.0 + (t * 0.2).cos() * 4.0
            })
            .collect();
        let mut qqe = Qqe::new(14, 5, 4.236).unwrap();
        let got = qqe.batch(&prices);
        let want = naive(&prices, 14, 5, 4.236);
        for (i, (g, w)) in got.iter().zip(want.iter()).enumerate() {
            assert_eq!(g.is_some(), w.is_some(), "readiness mismatch at {i}");
            if let (Some(a), Some(b)) = (g, w) {
                assert_relative_eq!(a.rsi_ma, b.rsi_ma, epsilon = 1e-9);
                assert_relative_eq!(a.trailing_line, b.trailing_line, epsilon = 1e-9);
            }
        }
    }

    #[test]
    fn trailing_line_below_rsi_ma_in_uptrend() {
        // Sustained rise: trend resolves to +1 and the trailing (long) band sits
        // below the smoothed RSI.
        let prices: Vec<f64> = (1..=120).map(f64::from).collect();
        let mut qqe = Qqe::new(14, 5, 4.236).unwrap();
        let last = qqe.batch(&prices).into_iter().flatten().last().unwrap();
        assert!(
            last.trailing_line <= last.rsi_ma,
            "uptrend trailing {} should sit at/below rsi_ma {}",
            last.trailing_line,
            last.rsi_ma
        );
    }

    #[test]
    fn reset_clears_state() {
        let mut qqe = Qqe::new(14, 5, 4.236).unwrap();
        qqe.batch(
            &(0..120)
                .map(|i| 100.0 + (f64::from(i) * 0.1).sin() * 8.0)
                .collect::<Vec<_>>(),
        );
        assert!(qqe.is_ready());
        qqe.reset();
        assert!(!qqe.is_ready());
        assert_eq!(qqe.update(1.0), None);
    }

    #[test]
    fn batch_equals_streaming() {
        let prices: Vec<f64> = (0..150)
            .map(|i| 50.0 + (f64::from(i) * 0.12).sin() * 12.0)
            .collect();
        let mut a = Qqe::new(14, 5, 4.236).unwrap();
        let mut b = Qqe::new(14, 5, 4.236).unwrap();
        assert_eq!(
            a.batch(&prices),
            prices.iter().map(|p| b.update(*p)).collect::<Vec<_>>()
        );
    }
}
