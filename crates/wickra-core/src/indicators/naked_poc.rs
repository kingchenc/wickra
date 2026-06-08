//! Naked POC — the nearest prior-session point of control price has not yet revisited.

use std::collections::VecDeque;

use crate::error::{Error, Result};
use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// Naked (Virgin) POC — the nearest **untested** point of control from a prior
/// session: a heavily-traded price the market has not traded back through since.
///
/// ```text
/// every `session_len` candles forms a session; its POC (heaviest-volume price) is
///   recorded as "naked"
/// a naked POC becomes "tested" once a later candle's high-low range covers it
/// output = the nearest still-naked POC to the current close (or the close itself
///   if every prior POC has been revisited)
/// ```
///
/// A point of control is a magnet — price tends to return to fair value. A *naked*
/// (or virgin) POC is one that has not yet been revisited, so it carries an
/// outstanding "pull": untested POCs are high-probability targets and
/// support/resistance on the approach. This indicator records each completed
/// session's POC, marks them tested as price trades through them, and reports the
/// closest one still outstanding.
///
/// The first value lands after `session_len` candles (the first session's POC).
/// Each `update` is O(`session_len · bins` + naked-count).
///
/// # Example
///
/// ```
/// use wickra_core::{Candle, Indicator, NakedPoc};
///
/// let mut indicator = NakedPoc::new(20, 24).unwrap();
/// let mut last = None;
/// for i in 0..60 {
///     let base = 100.0 + (f64::from(i) * 0.2).sin() * 5.0;
///     let c = Candle::new(base, base + 1.0, base - 1.0, base, 1_000.0, 0).unwrap();
///     last = indicator.update(c);
/// }
/// assert!(last.is_some());
/// ```
#[derive(Debug, Clone)]
pub struct NakedPoc {
    session_len: usize,
    bins: usize,
    session: VecDeque<Candle>,
    naked: Vec<f64>,
    last_close: f64,
    ready: bool,
    last: Option<f64>,
}

impl NakedPoc {
    /// Construct a Naked POC tracker with the given `session_len` and profile
    /// `bins`.
    ///
    /// # Errors
    ///
    /// Returns [`Error::PeriodZero`] if `session_len` or `bins` is zero.
    pub fn new(session_len: usize, bins: usize) -> Result<Self> {
        if session_len == 0 || bins == 0 {
            return Err(Error::PeriodZero);
        }
        Ok(Self {
            session_len,
            bins,
            session: VecDeque::with_capacity(session_len),
            naked: Vec::new(),
            last_close: 0.0,
            ready: false,
            last: None,
        })
    }

    /// Configured `(session_len, bins)`.
    pub const fn params(&self) -> (usize, usize) {
        (self.session_len, self.bins)
    }

    /// Number of currently-naked POCs.
    pub fn naked_count(&self) -> usize {
        self.naked.len()
    }

    /// Current value if available.
    pub const fn value(&self) -> Option<f64> {
        self.last
    }

    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    fn session_poc(&self) -> f64 {
        let mut low = f64::INFINITY;
        let mut high = f64::NEG_INFINITY;
        for c in &self.session {
            low = low.min(c.low);
            high = high.max(c.high);
        }
        let span = high - low;
        if span <= 0.0 {
            return low;
        }
        let width = span / self.bins as f64;
        let mut hist = vec![0.0; self.bins];
        for c in &self.session {
            if c.volume == 0.0 {
                continue;
            }
            let lo_idx = (((c.low - low) / width).floor() as usize).min(self.bins - 1);
            let hi_idx = (((c.high - low) / width).floor() as usize).min(self.bins - 1);
            let share = c.volume / (hi_idx - lo_idx + 1) as f64;
            for bin in hist.iter_mut().take(hi_idx + 1).skip(lo_idx) {
                *bin += share;
            }
        }
        let mut poc = 0;
        let mut poc_vol = f64::NEG_INFINITY;
        for (idx, &vol) in hist.iter().enumerate() {
            if vol > poc_vol {
                poc_vol = vol;
                poc = idx;
            }
        }
        low + (poc as f64 + 0.5) * width
    }
}

impl Indicator for NakedPoc {
    type Input = Candle;
    type Output = f64;

    fn update(&mut self, candle: Candle) -> Option<f64> {
        // Test outstanding naked POCs against this candle's range.
        self.naked
            .retain(|&poc| !(candle.low <= poc && poc <= candle.high));
        self.last_close = candle.close;

        // Accumulate the session; finalize a POC at the boundary.
        self.session.push_back(candle);
        if self.session.len() == self.session_len {
            let poc = self.session_poc();
            self.naked.push(poc);
            self.session.clear();
            self.ready = true;
        }

        if !self.ready {
            return None;
        }
        let nearest = self
            .naked
            .iter()
            .copied()
            .min_by(|a, b| {
                (a - self.last_close)
                    .abs()
                    .total_cmp(&(b - self.last_close).abs())
            })
            .unwrap_or(self.last_close);
        self.last = Some(nearest);
        Some(nearest)
    }

    fn reset(&mut self) {
        self.session.clear();
        self.naked.clear();
        self.last_close = 0.0;
        self.ready = false;
        self.last = None;
    }

    fn warmup_period(&self) -> usize {
        self.session_len
    }

    fn is_ready(&self) -> bool {
        self.last.is_some()
    }

    fn name(&self) -> &'static str {
        "NakedPoc"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;

    fn c(high: f64, low: f64, close: f64, volume: f64) -> Candle {
        Candle::new_unchecked(f64::midpoint(high, low), high, low, close, volume, 0)
    }

    #[test]
    fn rejects_zero_params() {
        assert!(matches!(NakedPoc::new(0, 24), Err(Error::PeriodZero)));
        assert!(matches!(NakedPoc::new(20, 0), Err(Error::PeriodZero)));
    }

    #[test]
    fn accessors_and_metadata() {
        let n = NakedPoc::new(20, 24).unwrap();
        assert_eq!(n.params(), (20, 24));
        assert_eq!(n.naked_count(), 0);
        assert_eq!(n.warmup_period(), 20);
        assert_eq!(n.name(), "NakedPoc");
        assert!(!n.is_ready());
        assert_eq!(n.value(), None);
    }

    #[test]
    fn first_emission_at_session_end() {
        let mut n = NakedPoc::new(4, 8).unwrap();
        let candles: Vec<Candle> = (0..6).map(|_| c(101.0, 99.0, 100.0, 1_000.0)).collect();
        let out = n.batch(&candles);
        for v in out.iter().take(3) {
            assert!(v.is_none());
        }
        assert!(out[3].is_some());
    }

    #[test]
    fn records_session_poc() {
        let mut n = NakedPoc::new(4, 16).unwrap();
        // A session clustered around 100 -> POC near 100.
        n.batch(&[c(101.0, 99.0, 100.0, 5_000.0); 4]);
        assert_eq!(n.naked_count(), 1);
        let poc = n.value().unwrap();
        assert!(
            (poc - 100.0).abs() < 2.0,
            "POC should be near 100, got {poc}"
        );
    }

    #[test]
    fn revisit_marks_poc_tested() {
        let mut n = NakedPoc::new(4, 16).unwrap();
        // Session 1 around 100 -> naked POC ~100.
        n.batch(&[c(101.0, 99.0, 100.0, 5_000.0); 4]);
        assert_eq!(n.naked_count(), 1);
        // Trade away at 120 (does not cover 100) -> still naked.
        n.update(c(121.0, 119.0, 120.0, 1_000.0));
        assert_eq!(n.naked_count(), 1);
        // A candle whose range covers 100 -> POC tested -> removed.
        n.update(c(121.0, 95.0, 100.0, 1_000.0));
        assert_eq!(n.naked_count(), 0);
    }

    #[test]
    fn empty_naked_reports_close() {
        let mut n = NakedPoc::new(4, 16).unwrap();
        n.batch(&[c(101.0, 99.0, 100.0, 5_000.0); 4]);
        // Wipe the naked POC with a covering candle.
        let out = n.update(c(121.0, 95.0, 117.0, 1_000.0)).unwrap();
        assert_eq!(n.naked_count(), 0);
        assert!(
            (out - 117.0).abs() < 1e-9,
            "with no naked POC, output is the close"
        );
    }

    #[test]
    fn reset_clears_state() {
        let mut n = NakedPoc::new(4, 8).unwrap();
        n.batch(&[c(101.0, 99.0, 100.0, 1_000.0); 6]);
        assert!(n.is_ready());
        n.reset();
        assert!(!n.is_ready());
        assert_eq!(n.value(), None);
        assert_eq!(n.naked_count(), 0);
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0..80)
            .map(|i| {
                let b = 100.0 + (f64::from(i) * 0.25).sin() * 9.0;
                c(b + 1.0, b - 1.0, b, 1_000.0 + f64::from(i))
            })
            .collect();
        let batch = NakedPoc::new(20, 24).unwrap().batch(&candles);
        let mut b = NakedPoc::new(20, 24).unwrap();
        let streamed: Vec<_> = candles.iter().map(|x| b.update(*x)).collect();
        assert_eq!(batch, streamed);
    }

    #[test]
    fn flat_session_reports_price() {
        // A session with zero high-low span returns the session price directly.
        let mut n = NakedPoc::new(2, 4).unwrap();
        n.update(c(50.0, 50.0, 50.0, 10.0));
        assert_eq!(n.update(c(50.0, 50.0, 50.0, 10.0)), Some(50.0));
    }

    #[test]
    fn zero_volume_session_is_handled() {
        // Zero-volume candles are skipped in the histogram; a POC still emits.
        let mut n = NakedPoc::new(2, 4).unwrap();
        n.update(c(60.0, 40.0, 50.0, 0.0));
        assert!(n.update(c(60.0, 40.0, 50.0, 0.0)).is_some());
    }

    #[test]
    fn nearest_of_two_naked_pocs() {
        // Two untouched POCs at distant prices accumulate; the one nearest the
        // last close is reported (exercises the min-by comparison).
        let mut n = NakedPoc::new(2, 4).unwrap();
        n.update(c(11.0, 9.0, 10.0, 100.0));
        n.update(c(11.0, 9.0, 10.0, 100.0)); // POC near 10
        n.update(c(101.0, 99.0, 100.0, 100.0));
        let v = n.update(c(101.0, 99.0, 100.0, 100.0)).unwrap(); // POC near 100
        assert!(
            v > 50.0,
            "nearest to close 100 should be the upper POC, got {v}"
        );
    }
}
