#![no_main]
//! Fuzz scalar-input indicator updates with arbitrary `f64` sequences.
//!
//! Every scalar indicator must tolerate any finite-or-not input stream — NaN,
//! ±inf, subnormals, abrupt jumps — without panicking. Each fuzz iteration
//! runs the **same** input sequence through every scalar indicator twice:
//! once as a streaming `update` loop and once as a full `batch` call. Neither
//! path may panic; `batch` is also expected to agree with the streaming path
//! (the `BatchExt` blanket implementation replays `update` internally, so the
//! agreement is structural — but exercising both paths surfaces any
//! state-mutation bugs in `update` that would only manifest mid-batch).
//!
//! Audit finding R9: the previous version covered only `Rsi(14)` and
//! `Ema(20)`. This target now covers every scalar indicator in the catalogue.

use libfuzzer_sys::fuzz_target;
use wickra_core::{
    AverageDrawdown, BatchExt, BollingerBands, CalmarRatio, Cmo, ConditionalValueAtRisk, Coppock,
    Dema, Dpo, DrawdownDuration, Ema, GainLossRatio, HistoricalVolatility, Hma, Indicator, Kama,
    KellyCriterion, LinRegAngle, LinRegSlope, LinearRegression, MacdIndicator, MaxDrawdown, Mom,
    OmegaRatio, PainIndex, Pmo, Ppo, ProfitFactor, RecoveryFactor, Roc, Rsi, SharpeRatio, Sma,
    Smma, SortinoRatio, StdDev, StochRsi, T3, Tema, Trima, Trix, Tsi, UlcerIndex, ValueAtRisk,
    VerticalHorizontalFilter, Wma, ZScore, Zlema,
};

/// Drive a single streaming + batch run through one scalar indicator. Marked
/// `#[inline(never)]` so a panic backtrace pin-points the specific indicator.
#[inline(never)]
fn drive<I>(make: impl Fn() -> I, data: &[f64])
where
    I: Indicator<Input = f64, Output = f64> + BatchExt,
{
    let mut streaming = make();
    for &x in data {
        let _ = streaming.update(x);
    }
    let _ = make().batch(data);
}

fuzz_target!(|data: Vec<f64>| {
    // Bounded periods keep each iteration cheap and bias the fuzzer toward
    // adversarial input patterns rather than enormous windows. The constants
    // mirror the README's "common defaults" so we cover the parameterisations
    // most users actually instantiate.
    drive(|| Sma::new(14).unwrap(), &data);
    drive(|| Ema::new(20).unwrap(), &data);
    drive(|| Wma::new(14).unwrap(), &data);
    drive(|| Rsi::new(14).unwrap(), &data);
    drive(|| Dema::new(14).unwrap(), &data);
    drive(|| Tema::new(14).unwrap(), &data);
    drive(|| Hma::new(14).unwrap(), &data);
    drive(|| Roc::new(14).unwrap(), &data);
    drive(|| Trix::new(14).unwrap(), &data);
    drive(|| Smma::new(14).unwrap(), &data);
    drive(|| Trima::new(14).unwrap(), &data);
    drive(|| Zlema::new(14).unwrap(), &data);
    drive(|| Kama::new(10, 2, 30).unwrap(), &data);
    drive(|| T3::new(14, 0.7).unwrap(), &data);
    drive(|| Mom::new(14).unwrap(), &data);
    drive(|| Cmo::new(14).unwrap(), &data);
    drive(|| Tsi::new(25, 13).unwrap(), &data);
    drive(|| Pmo::new(35, 20).unwrap(), &data);
    drive(|| StochRsi::new(14, 14).unwrap(), &data);
    drive(|| Dpo::new(14).unwrap(), &data);
    drive(|| Ppo::new(12, 26).unwrap(), &data);
    drive(|| Coppock::new(14, 11, 10).unwrap(), &data);
    drive(|| StdDev::new(14).unwrap(), &data);
    drive(|| UlcerIndex::new(14).unwrap(), &data);
    drive(|| HistoricalVolatility::new(14, 252).unwrap(), &data);
    drive(|| LinearRegression::new(14).unwrap(), &data);
    drive(|| LinRegSlope::new(14).unwrap(), &data);
    drive(|| LinRegAngle::new(14).unwrap(), &data);
    drive(|| VerticalHorizontalFilter::new(14).unwrap(), &data);
    drive(|| ZScore::new(14).unwrap(), &data);

    // Family 15 — Risk / Performance metrics (scalar inputs).
    drive(|| SharpeRatio::new(20, 0.0).unwrap(), &data);
    drive(|| SortinoRatio::new(20, 0.0).unwrap(), &data);
    drive(|| CalmarRatio::new(20).unwrap(), &data);
    drive(|| OmegaRatio::new(20, 0.0).unwrap(), &data);
    drive(|| MaxDrawdown::new(20).unwrap(), &data);
    drive(|| AverageDrawdown::new(20).unwrap(), &data);
    drive(|| PainIndex::new(20).unwrap(), &data);
    drive(|| ValueAtRisk::new(20, 0.95).unwrap(), &data);
    drive(|| ConditionalValueAtRisk::new(20, 0.95).unwrap(), &data);
    drive(|| ProfitFactor::new(20).unwrap(), &data);
    drive(|| GainLossRatio::new(20).unwrap(), &data);
    drive(|| KellyCriterion::new(20).unwrap(), &data);

    // RecoveryFactor and DrawdownDuration produce non-`f64` outputs / have
    // no `period` knob, so they cannot use the `drive` helper directly.
    {
        let mut rf = RecoveryFactor::new();
        for &x in &data {
            let _ = rf.update(x);
        }
        let _ = RecoveryFactor::new().batch(&data);
    }
    {
        let mut dd = DrawdownDuration::new();
        for &x in &data {
            let _ = dd.update(x);
        }
        let _ = DrawdownDuration::new().batch(&data);
    }

    // MACD and Bollinger Bands have non-`f64` outputs, so they cannot use the
    // generic `drive` helper above. Streaming + batch are still both exercised.
    {
        let mut macd = MacdIndicator::new(12, 26, 9).unwrap();
        for &x in &data {
            let _ = macd.update(x);
        }
        let _ = MacdIndicator::new(12, 26, 9).unwrap().batch(&data);
    }
    {
        let mut bb = BollingerBands::new(20, 2.0).unwrap();
        for &x in &data {
            let _ = bb.update(x);
        }
        let _ = BollingerBands::new(20, 2.0).unwrap().batch(&data);
    }
});
