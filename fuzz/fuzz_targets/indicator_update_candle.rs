#![no_main]
//! Fuzz OHLCV-input indicator updates with arbitrary candle sequences.
//!
//! Every candle-input indicator must tolerate any sequence of validated OHLCV
//! candles — extreme magnitudes, micro-spreads, zero-volume bars, abrupt
//! reversals — without panicking. The fuzzer chunks the raw `f64` stream into
//! `[open, high, low, close, volume]` tuples and constructs each candle via
//! `Candle::new`; entries that fail OHLCV-invariant validation are skipped so
//! the indicator only ever sees structurally-valid candles. Each iteration
//! then drives that candle stream through every candle-input indicator twice
//! (streaming `update` + batch).
//!
//! Audit finding R9: the previous fuzz suite had no candle-input coverage at
//! all. This target now covers every candle-input indicator including the
//! ones the audit named explicitly (ATR, ADX, Stochastic, PSAR) plus the
//! complete catalogue: Keltner, Donchian, SuperTrend, Chandelier Exit, ATR
//! Trailing Stop, Aroon, AwesomeOscillator, CCI, WilliamsR, MFI, OBV, VWAP,
//! RollingVWAP, ADL, VPT, ChaikinMoneyFlow, ChaikinOscillator, ForceIndex,
//! EaseOfMovement, NATR, AroonOscillator, ChandeKrollStop, Vortex, MassIndex,
//! ChoppinessIndex, TrueRange, ChaikinVolatility, AcceleratorOscillator,
//! BalanceOfPower, UltimateOscillator, VWMA, TypicalPrice, MedianPrice,
//! WeightedClose.

use libfuzzer_sys::fuzz_target;
use wickra_core::{
    AcceleratorOscillator, Adl, Adx, Aroon, AroonOscillator, Atr, AtrTrailingStop,
    AwesomeOscillator, BalanceOfPower, BatchExt, Candle, Cci, ChaikinMoneyFlow, ChaikinOscillator,
    ChaikinVolatility, ChandeKrollStop, ChandelierExit, ChoppinessIndex, Donchian, EaseOfMovement,
    ForceIndex, Indicator, Keltner, Kvo, MassIndex, MedianPrice, Mfi, Natr, Obv, Psar, RollingVwap,
    Stochastic, SuperTrend, TrueRange, TypicalPrice, UltimateOscillator, VolumePriceTrend, Vortex,
    Vwap, Vwma, WeightedClose, WilliamsR,
};

/// Convert a flat `f64` stream into a `Vec<Candle>` by chunking it into
/// `[open, high, low, close, volume]` groups. Tuples that fail OHLCV
/// validation are dropped so the indicator under test only ever sees a
/// structurally-valid candle stream (the *parser* is fuzz-tested elsewhere;
/// this target focuses on indicator robustness).
fn candles_from(data: &[f64]) -> Vec<Candle> {
    data.chunks_exact(5)
        .enumerate()
        .filter_map(|(i, ch)| {
            // A monotonic timestamp avoids surprising any indicator that might
            // care about ordering. The fuzz input drives OHLCV; time is just a
            // tie-breaker.
            Candle::new(ch[0], ch[1], ch[2], ch[3], ch[4], i as i64).ok()
        })
        .collect()
}

/// Streaming + batch sweep through one candle-input indicator. `#[inline(never)]`
/// keeps each indicator on its own frame in any panic backtrace.
#[inline(never)]
fn drive<I, O>(make: impl Fn() -> I, candles: &[Candle])
where
    I: Indicator<Input = Candle, Output = O> + BatchExt,
{
    let mut streaming = make();
    for c in candles {
        let _ = streaming.update(*c);
    }
    let _ = make().batch(candles);
}

fuzz_target!(|data: Vec<f64>| {
    let candles = candles_from(&data);
    if candles.is_empty() {
        return;
    }

    // --- Volatility & ATR family ---
    drive(|| Atr::new(14).unwrap(), &candles);
    drive(|| Natr::new(14).unwrap(), &candles);
    drive(TrueRange::new, &candles);
    drive(|| ChaikinVolatility::new(10, 10).unwrap(), &candles);

    // --- Bands & Channels ---
    drive(|| Keltner::new(20, 10, 2.0).unwrap(), &candles);
    drive(|| Donchian::new(20).unwrap(), &candles);

    // --- Trailing Stops ---
    drive(|| Psar::new(0.02, 0.02, 0.20).unwrap(), &candles);
    drive(|| SuperTrend::new(14, 3.0).unwrap(), &candles);
    drive(|| ChandelierExit::new(22, 3.0).unwrap(), &candles);
    drive(|| ChandeKrollStop::new(10, 1.0, 9).unwrap(), &candles);
    drive(|| AtrTrailingStop::new(14, 3.0).unwrap(), &candles);

    // --- Trend & Directional ---
    drive(|| Adx::new(14).unwrap(), &candles);
    drive(|| Aroon::new(14).unwrap(), &candles);
    drive(|| AroonOscillator::new(14).unwrap(), &candles);
    drive(|| Vortex::new(14).unwrap(), &candles);
    drive(|| MassIndex::new(9, 25).unwrap(), &candles);
    drive(|| ChoppinessIndex::new(14).unwrap(), &candles);

    // --- Momentum & Oscillators ---
    drive(|| Cci::new(20).unwrap(), &candles);
    drive(|| WilliamsR::new(14).unwrap(), &candles);
    drive(|| AwesomeOscillator::new(5, 34).unwrap(), &candles);
    drive(|| AcceleratorOscillator::new(5, 34, 5).unwrap(), &candles);
    drive(|| UltimateOscillator::new(7, 14, 28).unwrap(), &candles);
    drive(BalanceOfPower::new, &candles);

    // --- Volume ---
    drive(Obv::new, &candles);
    drive(|| Mfi::new(14).unwrap(), &candles);
    drive(Vwap::new, &candles);
    drive(|| RollingVwap::new(20).unwrap(), &candles);
    drive(|| Vwma::new(20).unwrap(), &candles);
    drive(Adl::new, &candles);
    drive(VolumePriceTrend::new, &candles);
    drive(|| ChaikinMoneyFlow::new(20).unwrap(), &candles);
    drive(|| ChaikinOscillator::new(3, 10).unwrap(), &candles);
    drive(|| ForceIndex::new(13).unwrap(), &candles);
    drive(|| EaseOfMovement::with_divisor(14, 1e8).unwrap(), &candles);
    drive(|| Kvo::new(34, 55).unwrap(), &candles);

    // --- Price transformations ---
    drive(TypicalPrice::new, &candles);
    drive(MedianPrice::new, &candles);
    drive(WeightedClose::new, &candles);

    // --- Stochastic (multi-output) ---
    {
        let mut s = Stochastic::new(14, 3).unwrap();
        for c in &candles {
            let _ = s.update(*c);
        }
        let _ = Stochastic::new(14, 3).unwrap().batch(&candles);
    }
});
