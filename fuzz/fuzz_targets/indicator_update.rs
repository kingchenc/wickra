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
AdaptiveCycle, Alma, AnchoredRsi, Apo, Autocorrelation, AverageDrawdown, BatchExt, Beta, BollingerBands, CalmarRatio, CenterOfGravity, Cfo, Cmo, CoefficientOfVariation, ConditionalValueAtRisk, ConnorsRsi, Coppock, CyberneticCycle, Decycler, DecyclerOscillator, Dema, DetrendedStdDev, DoubleBollinger, Dpo, DrawdownDuration, EhlersStochastic, ElderImpulse, Ema, EmpiricalModeDecomposition, Fama, FisherTransform, Frama, GainLossRatio, HilbertDominantCycle, HistoricalVolatility, Hma, HurstExponent, Indicator, InstantaneousTrendline, InverseFisherTransform, Jma, Kama, KellyCriterion, Kst, Kurtosis, LaguerreRsi, LinRegAngle, LinRegChannel, LinRegIntercept, LinRegSlope, LinearRegression, MaEnvelope, MacdFix, MacdIndicator, Mama, MaxDrawdown, McGinleyDynamic, MedianAbsoluteDeviation, MidPoint, Mom, OmegaRatio, PainIndex, PearsonCorrelation, PercentageTrailingStop, Pmo, Ppo, ProfitFactor, RSquared, RecoveryFactor, RenkoTrailingStop, Roc, Rocp, Rocr, Rocr100, RoofingFilter, Rsi, RviVolatility, SharpeRatio, SineWave, Skewness, Sma, Smma, SortinoRatio, SpearmanCorrelation, StandardError, StandardErrorBands, Stc, StdDev, StepTrailingStop, StochRsi, SuperSmoother, Tema, Tii, Trima, Trix, Tsf, Tsi, UlcerIndex, ValueAtRisk, Variance, VerticalHorizontalFilter, Vidya, Wma, ZScore, ZeroLagMacd, Zlema, T3
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
    drive(AnchoredRsi::new, &data);
    drive(|| Dema::new(14).unwrap(), &data);
    drive(|| Tema::new(14).unwrap(), &data);
    drive(|| Hma::new(14).unwrap(), &data);
    drive(|| Roc::new(14).unwrap(), &data);
    drive(|| Rocp::new(14).unwrap(), &data);
    drive(|| Rocr::new(14).unwrap(), &data);
    drive(|| Rocr100::new(14).unwrap(), &data);
    drive(|| Trix::new(14).unwrap(), &data);
    drive(|| Smma::new(14).unwrap(), &data);
    drive(|| Trima::new(14).unwrap(), &data);
    drive(|| Zlema::new(14).unwrap(), &data);
    drive(|| Kama::new(10, 2, 30).unwrap(), &data);
    drive(|| Alma::new(9, 0.85, 6.0).unwrap(), &data);
    drive(|| McGinleyDynamic::new(10).unwrap(), &data);
    drive(|| Frama::new(16).unwrap(), &data);
    drive(|| Vidya::new(14, 9).unwrap(), &data);
    drive(|| Jma::new(14, 0.0, 2).unwrap(), &data);
    drive(|| T3::new(14, 0.7).unwrap(), &data);
    drive(|| Mom::new(14).unwrap(), &data);
    drive(|| Cmo::new(14).unwrap(), &data);
    drive(|| Tsi::new(25, 13).unwrap(), &data);
    drive(|| Pmo::new(35, 20).unwrap(), &data);
    drive(|| Tii::new(60, 30).unwrap(), &data);
    drive(|| StochRsi::new(14, 14).unwrap(), &data);
    drive(|| Dpo::new(14).unwrap(), &data);
    drive(|| Ppo::new(12, 26).unwrap(), &data);
    drive(|| Apo::new(12, 26).unwrap(), &data);
    drive(|| Cfo::new(14).unwrap(), &data);
    drive(|| ElderImpulse::classic(), &data);
    drive(|| Stc::classic(), &data);
    drive(|| Coppock::new(14, 11, 10).unwrap(), &data);
    drive(|| StdDev::new(14).unwrap(), &data);
    drive(|| UlcerIndex::new(14).unwrap(), &data);
    drive(|| HistoricalVolatility::new(14, 252).unwrap(), &data);
    drive(|| LinearRegression::new(14).unwrap(), &data);
    drive(|| MidPoint::new(14).unwrap(), &data);
    drive(|| LinRegSlope::new(14).unwrap(), &data);
    drive(|| LinRegIntercept::new(14).unwrap(), &data);
    drive(|| Tsf::new(14).unwrap(), &data);
    drive(|| LinRegAngle::new(14).unwrap(), &data);
    drive(|| VerticalHorizontalFilter::new(14).unwrap(), &data);
    drive(|| ZScore::new(14).unwrap(), &data);
    drive(|| Variance::new(14).unwrap(), &data);
    drive(|| CoefficientOfVariation::new(14).unwrap(), &data);
    drive(|| Skewness::new(14).unwrap(), &data);
    drive(|| Kurtosis::new(14).unwrap(), &data);
    drive(|| StandardError::new(14).unwrap(), &data);
    drive(|| DetrendedStdDev::new(14).unwrap(), &data);
    drive(|| RSquared::new(14).unwrap(), &data);
    drive(|| MedianAbsoluteDeviation::new(14).unwrap(), &data);
    drive(|| Autocorrelation::new(14, 2).unwrap(), &data);
    // HurstExponent needs `period >= 2 * chunks`; 16/4 is the cheapest fit
    // that still exercises every code path.
    drive(|| HurstExponent::new(16, 4).unwrap(), &data);
    drive(|| RviVolatility::new(10).unwrap(), &data);
    drive(|| LaguerreRsi::new(0.5).unwrap(), &data);
    drive(|| ConnorsRsi::classic(), &data);

    // KST is scalar-input but emits `KstOutput`, so it bypasses the generic
    // `drive` helper. Streaming + batch are still both exercised.
    {
        let mut kst = Kst::classic();
        for &x in &data {
            let _ = kst.update(x);
        }
        let _ = Kst::classic().batch(&data);
    }

    // Zero-Lag MACD shares MACD's multi-output topology, so it gets the
    // same hand-rolled streaming + batch drive as classic MACD below.
    {
        let mut z = ZeroLagMacd::classic();
        for &x in &data {
            let _ = z.update(x);
        }
        let _ = ZeroLagMacd::classic().batch(&data);
    }

    // --- Trailing Stops (scalar) ---
    drive(|| PercentageTrailingStop::new(5.0).unwrap(), &data);
    drive(|| StepTrailingStop::new(1.0).unwrap(), &data);
    drive(|| RenkoTrailingStop::new(1.0).unwrap(), &data);

    // Family 10 — Ehlers / Cycle scalar indicators.
    drive(|| SuperSmoother::new(10).unwrap(), &data);
    drive(|| FisherTransform::new(10).unwrap(), &data);
    drive(|| InverseFisherTransform::new(1.0).unwrap(), &data);
    drive(|| Decycler::new(20).unwrap(), &data);
    drive(|| DecyclerOscillator::new(10, 30).unwrap(), &data);
    drive(|| RoofingFilter::new(10, 48).unwrap(), &data);
    drive(|| CenterOfGravity::new(10).unwrap(), &data);
    drive(|| CyberneticCycle::new(10).unwrap(), &data);
    drive(|| InstantaneousTrendline::new(20).unwrap(), &data);
    drive(|| EhlersStochastic::new(20).unwrap(), &data);
    drive(|| EmpiricalModeDecomposition::new(20, 0.5).unwrap(), &data);
    drive(HilbertDominantCycle::new, &data);
    drive(AdaptiveCycle::new, &data);
    drive(SineWave::new, &data);
    drive(|| Fama::new(0.5, 0.05).unwrap(), &data);

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

    // MACD, Bollinger Bands and MAMA have non-`f64` outputs, so they cannot
    // use the generic `drive` helper above. Streaming + batch are still both
    // exercised.
    {
        let mut macd = MacdIndicator::new(12, 26, 9).unwrap();
        for &x in &data {
            let _ = macd.update(x);
        }
        let _ = MacdIndicator::new(12, 26, 9).unwrap().batch(&data);
    }

    // MACDFIX wraps MacdIndicator(12, 26, signal); same multi-output topology.
    {
        let mut fix = MacdFix::new(9).unwrap();
        for &x in &data {
            let _ = fix.update(x);
        }
        let _ = MacdFix::new(9).unwrap().batch(&data);
    }
    {
        let mut bb = BollingerBands::new(20, 2.0).unwrap();
        for &x in &data {
            let _ = bb.update(x);
        }
        let _ = BollingerBands::new(20, 2.0).unwrap().batch(&data);
    }
    {
        let mut mama = Mama::new(0.5, 0.05).unwrap();
        for &x in &data {
            let _ = mama.update(x);
        }
        let _ = Mama::new(0.5, 0.05).unwrap().batch(&data);
    }

    // --- Family 05: scalar-input band/channel indicators (multi-output) ---
    {
        let mut env = MaEnvelope::new(20, 0.025).unwrap();
        for &x in &data {
            let _ = env.update(x);
        }
        let _ = MaEnvelope::new(20, 0.025).unwrap().batch(&data);
    }
    {
        let mut ch = LinRegChannel::new(20, 2.0).unwrap();
        for &x in &data {
            let _ = ch.update(x);
        }
        let _ = LinRegChannel::new(20, 2.0).unwrap().batch(&data);
    }
    {
        let mut seb = StandardErrorBands::new(21, 2.0).unwrap();
        for &x in &data {
            let _ = seb.update(x);
        }
        let _ = StandardErrorBands::new(21, 2.0).unwrap().batch(&data);
    }
    {
        let mut db = DoubleBollinger::new(20, 1.0, 2.0).unwrap();
        for &x in &data {
            let _ = db.update(x);
        }
        let _ = DoubleBollinger::new(20, 1.0, 2.0).unwrap().batch(&data);
    }

    // Family 12: Two-series indicators — pair adjacent samples of `data`.
    {
        let mut p = PearsonCorrelation::new(14).unwrap();
        let mut b = Beta::new(14).unwrap();
        let mut s = SpearmanCorrelation::new(14).unwrap();
        for w in data.windows(2) {
            let pair = (w[0], w[1]);
            let _ = p.update(pair);
            let _ = b.update(pair);
            let _ = s.update(pair);
        }
    }
});
