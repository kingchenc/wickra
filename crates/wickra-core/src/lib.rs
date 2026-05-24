//! `wickra-core`: streaming-first technical indicators.
//!
//! The core engine of Wickra. Every indicator is implemented as a state machine
//! that consumes inputs one at a time via [`Indicator::update`] in constant time.
//! Batch evaluation is provided as a blanket extension trait so the same code
//! path serves both online (tick-by-tick) and offline (historical) workloads.
//!
//! # Design
//!
//! - **Streaming-first.** State is held by the indicator instance, so a new value
//!   only re-computes deltas, not the whole series.
//! - **Batch is free.** [`BatchExt::batch`] is a blanket implementation that
//!   simply replays `update` over a slice. Writing one implementation gives both
//!   APIs.
//! - **Composable.** Indicators implement [`Indicator<Input = f64, Output = f64>`]
//!   wherever they conceptually take a price, so they can be chained via
//!   [`Chain`].
//! - **No `unsafe`.** The crate forbids `unsafe_code` in the workspace lints.
//!
//! # Quick start
//!
//! ```
//! use wickra_core::{BatchExt, Indicator, Sma};
//!
//! // Streaming:
//! let mut sma = Sma::new(3).unwrap();
//! assert_eq!(sma.update(1.0), None);
//! assert_eq!(sma.update(2.0), None);
//! assert_eq!(sma.update(3.0), Some(2.0));
//!
//! // Batch (replays `update` internally):
//! let mut sma = Sma::new(3).unwrap();
//! let out = sma.batch(&[1.0, 2.0, 3.0, 4.0]);
//! assert_eq!(out, vec![None, None, Some(2.0), Some(3.0)]);
//! ```

#![cfg_attr(docsrs, feature(doc_cfg))]

mod error;
mod ohlcv;
mod traits;

pub mod indicators;

pub use error::{Error, Result};
pub use indicators::{
    AcceleratorOscillator, Adl, Adx, AdxOutput, Alma, Aroon, AroonOscillator, AroonOutput, Atr,
    AtrTrailingStop, AwesomeOscillator, BalanceOfPower, BollingerBands, BollingerBandwidth,
    BollingerOutput, Cci, ChaikinMoneyFlow, ChaikinOscillator, ChaikinVolatility, ChandeKrollStop,
    ChandeKrollStopOutput, ChandelierExit, ChandelierExitOutput, ChoppinessIndex, Cmo, Coppock,
    Dema, Donchian, DonchianOutput, Dpo, EaseOfMovement, Ema, ForceIndex, Frama,
    HistoricalVolatility, Hma, Kama, Keltner, KeltnerOutput, LinRegAngle, LinRegSlope,
    LinearRegression, MacdIndicator, MacdOutput, MassIndex, McGinleyDynamic, MedianPrice, Mfi, Mom,
    Natr, Obv, PercentB, Pmo, Ppo, Psar, Roc, RollingVwap, Rsi, Sma, Smma, StdDev, StochRsi,
    Stochastic, StochasticOutput, SuperTrend, SuperTrendOutput, Tema, Trima, Trix, TrueRange, Tsi,
    TypicalPrice, UlcerIndex, UltimateOscillator, VerticalHorizontalFilter, VolumePriceTrend,
    Vortex, VortexOutput, Vwap, Vwma, WeightedClose, WilliamsR, Wma, ZScore, Zlema, T3,
};
pub use ohlcv::{Candle, Tick};
pub use traits::{BatchExt, Chain, Indicator};
