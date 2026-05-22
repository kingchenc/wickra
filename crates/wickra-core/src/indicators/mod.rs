//! Built-in indicators. Every indicator implements [`crate::Indicator`].
//!
//! Modules are organised internally by category (trend, momentum, volatility,
//! volume) but every public name is also re-exported flat from this module and
//! from the crate root for convenience.

mod adx;
mod aroon;
mod atr;
mod awesome_oscillator;
mod bollinger;
mod cci;
mod dema;
mod donchian;
mod ema;
mod hma;
mod kama;
mod keltner;
mod macd;
mod mfi;
mod obv;
mod psar;
mod roc;
mod rsi;
mod sma;
mod smma;
mod stochastic;
mod tema;
mod trima;
mod trix;
mod vwap;
mod williams_r;
mod wma;

pub use adx::{Adx, AdxOutput};
pub use aroon::{Aroon, AroonOutput};
pub use atr::Atr;
pub use awesome_oscillator::AwesomeOscillator;
pub use bollinger::{BollingerBands, BollingerOutput};
pub use cci::Cci;
pub use dema::Dema;
pub use donchian::{Donchian, DonchianOutput};
pub use ema::Ema;
pub use hma::Hma;
pub use kama::Kama;
pub use keltner::{Keltner, KeltnerOutput};
pub use macd::{MacdIndicator, MacdOutput};
pub use mfi::Mfi;
pub use obv::Obv;
pub use psar::Psar;
pub use roc::Roc;
pub use rsi::Rsi;
pub use sma::Sma;
pub use smma::Smma;
pub use stochastic::{Stochastic, StochasticOutput};
pub use tema::Tema;
pub use trima::Trima;
pub use trix::Trix;
pub use vwap::{RollingVwap, Vwap};
pub use williams_r::WilliamsR;
pub use wma::Wma;
