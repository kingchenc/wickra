//! MACD with selectable moving-average types (MACDEXT).

use crate::error::{Error, Result};
use crate::indicators::dema::Dema;
use crate::indicators::ema::Ema;
use crate::indicators::macd::MacdOutput;
use crate::indicators::sma::Sma;
use crate::indicators::tema::Tema;
use crate::indicators::trima::Trima;
use crate::indicators::wma::Wma;
use crate::traits::Indicator;

/// Moving-average type selector for [`MacdExt`] and other multi-MA indicators.
///
/// The variants map to TA-Lib's `MA_Type` codes `0..=5` — the period-only
/// moving averages. (TA-Lib's KAMA / MAMA / T3 take additional shape parameters
/// and are not selectable here.)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MaType {
    /// Simple moving average (TA-Lib code `0`).
    Sma,
    /// Exponential moving average (TA-Lib code `1`).
    Ema,
    /// Weighted moving average (TA-Lib code `2`).
    Wma,
    /// Double exponential moving average (TA-Lib code `3`).
    Dema,
    /// Triple exponential moving average (TA-Lib code `4`).
    Tema,
    /// Triangular moving average (TA-Lib code `5`).
    Trima,
}

impl MaType {
    /// Map a TA-Lib `MA_Type` integer code (`0..=5`) to a [`MaType`].
    ///
    /// # Errors
    /// Returns [`Error::InvalidPeriod`] for codes outside `0..=5` (the period-only
    /// moving averages); codes `6..=8` (KAMA / MAMA / T3) are not supported.
    pub fn from_code(code: u32) -> Result<Self> {
        match code {
            0 => Ok(Self::Sma),
            1 => Ok(Self::Ema),
            2 => Ok(Self::Wma),
            3 => Ok(Self::Dema),
            4 => Ok(Self::Tema),
            5 => Ok(Self::Trima),
            _ => Err(Error::InvalidPeriod {
                message: "unsupported moving-average type code (expected 0..=5)",
            }),
        }
    }
}

/// A concrete period-only moving average instance, dispatched by [`MaType`].
#[derive(Debug, Clone)]
enum Ma {
    Sma(Sma),
    Ema(Ema),
    Wma(Wma),
    Dema(Dema),
    Tema(Tema),
    Trima(Trima),
}

impl Ma {
    fn new(kind: MaType, period: usize) -> Result<Self> {
        Ok(match kind {
            MaType::Sma => Self::Sma(Sma::new(period)?),
            MaType::Ema => Self::Ema(Ema::new(period)?),
            MaType::Wma => Self::Wma(Wma::new(period)?),
            MaType::Dema => Self::Dema(Dema::new(period)?),
            MaType::Tema => Self::Tema(Tema::new(period)?),
            MaType::Trima => Self::Trima(Trima::new(period)?),
        })
    }

    fn update(&mut self, value: f64) -> Option<f64> {
        match self {
            Self::Sma(m) => m.update(value),
            Self::Ema(m) => m.update(value),
            Self::Wma(m) => m.update(value),
            Self::Dema(m) => m.update(value),
            Self::Tema(m) => m.update(value),
            Self::Trima(m) => m.update(value),
        }
    }

    fn reset(&mut self) {
        match self {
            Self::Sma(m) => m.reset(),
            Self::Ema(m) => m.reset(),
            Self::Wma(m) => m.reset(),
            Self::Dema(m) => m.reset(),
            Self::Tema(m) => m.reset(),
            Self::Trima(m) => m.reset(),
        }
    }

    fn warmup_period(&self) -> usize {
        match self {
            Self::Sma(m) => m.warmup_period(),
            Self::Ema(m) => m.warmup_period(),
            Self::Wma(m) => m.warmup_period(),
            Self::Dema(m) => m.warmup_period(),
            Self::Tema(m) => m.warmup_period(),
            Self::Trima(m) => m.warmup_period(),
        }
    }
}

/// MACD Extended (`MACDEXT`): MACD with an independently selectable
/// [`MaType`] for each of the fast, slow and signal lines.
///
/// Classic [`MacdIndicator`](crate::MacdIndicator) hard-wires the exponential
/// moving average everywhere; `MACDEXT` lets each line use any period-only
/// moving average. The MACD line is `fast_ma(price) − slow_ma(price)`, the signal
/// line is `signal_ma(macd)`, and the histogram is `macd − signal`. The first
/// full [`MacdOutput`] is emitted once the slow and signal averages are both warm.
///
/// # Example
///
/// ```
/// use wickra_core::{Indicator, MacdExt, MaType};
///
/// let mut indicator =
///     MacdExt::new(12, MaType::Ema, 26, MaType::Ema, 9, MaType::Sma).unwrap();
/// let mut last = None;
/// for i in 0..120 {
///     last = indicator.update(100.0 + f64::from(i));
/// }
/// assert!(last.is_some());
/// ```
#[derive(Debug, Clone)]
pub struct MacdExt {
    fast: Ma,
    slow: Ma,
    signal: Ma,
    has_emitted: bool,
}

impl MacdExt {
    /// Construct a MACDEXT with per-line periods and moving-average types.
    ///
    /// # Errors
    /// Returns [`Error::PeriodZero`] if any period is zero and
    /// [`Error::InvalidPeriod`] if `fast >= slow`, propagating any moving-average
    /// construction error.
    pub fn new(
        fast: usize,
        fast_type: MaType,
        slow: usize,
        slow_type: MaType,
        signal: usize,
        signal_type: MaType,
    ) -> Result<Self> {
        if fast == 0 || slow == 0 || signal == 0 {
            return Err(Error::PeriodZero);
        }
        if fast >= slow {
            return Err(Error::InvalidPeriod {
                message: "fast period must be < slow period",
            });
        }
        Ok(Self {
            fast: Ma::new(fast_type, fast)?,
            slow: Ma::new(slow_type, slow)?,
            signal: Ma::new(signal_type, signal)?,
            has_emitted: false,
        })
    }
}

impl Indicator for MacdExt {
    type Input = f64;
    type Output = MacdOutput;

    fn update(&mut self, value: f64) -> Option<MacdOutput> {
        let fast_v = self.fast.update(value);
        let slow_v = self.slow.update(value);
        let (Some(fast_v), Some(slow_v)) = (fast_v, slow_v) else {
            return None;
        };
        let macd = fast_v - slow_v;
        let signal = self.signal.update(macd)?;
        self.has_emitted = true;
        Some(MacdOutput {
            macd,
            signal,
            histogram: macd - signal,
        })
    }

    fn reset(&mut self) {
        self.fast.reset();
        self.slow.reset();
        self.signal.reset();
        self.has_emitted = false;
    }

    fn warmup_period(&self) -> usize {
        self.slow.warmup_period() + self.signal.warmup_period()
    }

    fn is_ready(&self) -> bool {
        self.has_emitted
    }

    fn name(&self) -> &'static str {
        "MACDEXT"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;

    const TYPES: [MaType; 6] = [
        MaType::Sma,
        MaType::Ema,
        MaType::Wma,
        MaType::Dema,
        MaType::Tema,
        MaType::Trima,
    ];

    #[test]
    fn from_code_maps_all_supported_types() {
        assert_eq!(MaType::from_code(0).unwrap(), MaType::Sma);
        assert_eq!(MaType::from_code(1).unwrap(), MaType::Ema);
        assert_eq!(MaType::from_code(2).unwrap(), MaType::Wma);
        assert_eq!(MaType::from_code(3).unwrap(), MaType::Dema);
        assert_eq!(MaType::from_code(4).unwrap(), MaType::Tema);
        assert_eq!(MaType::from_code(5).unwrap(), MaType::Trima);
        assert!(MaType::from_code(6).is_err());
    }

    #[test]
    fn rejects_invalid_periods() {
        assert!(matches!(
            MacdExt::new(0, MaType::Ema, 26, MaType::Ema, 9, MaType::Ema),
            Err(Error::PeriodZero)
        ));
        assert!(matches!(
            MacdExt::new(26, MaType::Ema, 12, MaType::Ema, 9, MaType::Ema),
            Err(Error::InvalidPeriod { .. })
        ));
    }

    #[test]
    fn accessors_and_metadata() {
        let m = MacdExt::new(12, MaType::Ema, 26, MaType::Sma, 9, MaType::Sma).unwrap();
        assert_eq!(m.name(), "MACDEXT");
        assert!(!m.is_ready());
        assert!(m.warmup_period() >= 26);
    }

    #[test]
    fn every_ma_type_produces_a_consistent_histogram() {
        let prices: Vec<f64> = (0..120)
            .map(|i| 100.0 + (f64::from(i) * 0.2).sin() * 6.0)
            .collect();
        for &t in &TYPES {
            let mut m = MacdExt::new(5, t, 10, t, 4, t).unwrap();
            let out: Vec<Option<MacdOutput>> = m.batch(&prices);
            assert!(out.iter().any(Option::is_some), "{t:?} never emitted");
            for o in out.into_iter().flatten() {
                assert!((o.histogram - (o.macd - o.signal)).abs() < 1e-9);
            }
            // Exercise the warmup accessor for this variant's inner averages.
            assert!(m.warmup_period() >= 10);
            assert!(m.is_ready());
            m.reset();
            assert!(!m.is_ready());
        }
    }

    #[test]
    fn mixed_ma_types_per_line() {
        let prices: Vec<f64> = (0..120).map(|i| 100.0 + f64::from(i)).collect();
        let mut m = MacdExt::new(12, MaType::Wma, 26, MaType::Dema, 9, MaType::Trima).unwrap();
        let last = m.batch(&prices).into_iter().flatten().last();
        assert!(last.is_some());
    }
}
