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
AbandonedBaby, AccelerationBands, AcceleratorOscillator, AdOscillator, Adl, AdvanceBlock, Adx, Adxr, Alligator, AnchoredVwap, Aroon, AroonOscillator, Atr, AtrBands, AtrTrailingStop, AwesomeOscillator, AwesomeOscillatorHistogram, BalanceOfPower, BatchExt, BeltHold, Breakaway, Camarilla, Candle, Cci, ChaikinMoneyFlow, ChaikinOscillator, ChaikinVolatility, ChandeKrollStop, ChandelierExit, ChoppinessIndex, ClassicPivots, Counterattack, DemandIndex, DemarkPivots, Doji, DojiStar, Donchian, DonchianStop, DragonflyDoji, EaseOfMovement, Engulfing, Evwma, FibonacciPivots, ForceIndex, FractalChaosBands, GarmanKlassVolatility, GravestoneDoji, Hammer, HangingMan, Harami, HeikinAshi, HiLoActivator, HurstChannel, Ichimoku, IdenticalThreeCrows, Indicator, Inertia, InitialBalance, InvertedHammer, Keltner, Kvo, LongLeggedDoji, MarketFacilitationIndex, Marubozu, MassIndex, MedianPrice, Mfi, MorningEveningStar, Natr, Nvi, Obv, OpeningRange, ParkinsonVolatility, Pgo, PiercingDarkCloud, Psar, Pvi, RickshawMan, RogersSatchellVolatility, RollingVwap, Rvi, Rwi, ShootingStar, Smi, SpinningTop, StarcBands, Stochastic, SuperTrend, TdCombo, TdCountdown, TdDeMarker, TdDifferential, TdLines, TdOpen, TdPressure, TdRangeProjection, TdRei, TdRiskLevel, TdSequential, TdSetup, ThreeInside, ThreeLineStrike, ThreeOutside, ThreeSoldiersOrCrows, ThreeStarsInSouth, TrueRange, Tsv, TtmSqueeze, Tweezer, TwoCrows, TypicalPrice, UltimateOscillator, UpsideGapTwoCrows, ValueArea, VoltyStop, VolumeOscillator, VolumePriceTrend, Vortex, Vwap, VwapStdDevBands, Vwma, Vzo, WaveTrend, WeightedClose, WilliamsFractals, WilliamsR, WoodiePivots, YangZhangVolatility, YoyoExit, ZigZag
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
    drive(|| ParkinsonVolatility::new(20, 252).unwrap(), &candles);
    drive(|| GarmanKlassVolatility::new(20, 252).unwrap(), &candles);
    drive(|| RogersSatchellVolatility::new(20, 252).unwrap(), &candles);
    drive(|| YangZhangVolatility::new(20, 252).unwrap(), &candles);

    // --- Bands & Channels ---
    drive(|| Keltner::new(20, 10, 2.0).unwrap(), &candles);
    drive(|| Donchian::new(20).unwrap(), &candles);

    // --- Trailing Stops ---
    drive(|| Psar::new(0.02, 0.02, 0.20).unwrap(), &candles);
    drive(|| SuperTrend::new(14, 3.0).unwrap(), &candles);
    drive(|| ChandelierExit::new(22, 3.0).unwrap(), &candles);
    drive(|| ChandeKrollStop::new(10, 1.0, 9).unwrap(), &candles);
    drive(|| AtrTrailingStop::new(14, 3.0).unwrap(), &candles);
    drive(|| HiLoActivator::new(3).unwrap(), &candles);
    drive(|| VoltyStop::new(14, 2.0).unwrap(), &candles);
    drive(|| YoyoExit::new(14, 2.0).unwrap(), &candles);

    // --- Trend & Directional ---
    drive(|| Adx::new(14).unwrap(), &candles);
    drive(|| Adxr::new(14).unwrap(), &candles);
    drive(|| Aroon::new(14).unwrap(), &candles);
    drive(|| Alligator::new(13, 8, 5).unwrap(), &candles);
    drive(|| AroonOscillator::new(14).unwrap(), &candles);
    drive(|| Vortex::new(14).unwrap(), &candles);
    drive(|| Rwi::new(14).unwrap(), &candles);
    drive(|| WaveTrend::classic().unwrap(), &candles);
    drive(|| MassIndex::new(9, 25).unwrap(), &candles);
    drive(|| ChoppinessIndex::new(14).unwrap(), &candles);

    // --- Momentum & Oscillators ---
    drive(|| Cci::new(20).unwrap(), &candles);
    drive(|| Rvi::new(10).unwrap(), &candles);
    drive(|| Inertia::new(14, 20).unwrap(), &candles);
    drive(|| Pgo::new(14).unwrap(), &candles);
    drive(|| Smi::classic(), &candles);
    drive(|| WilliamsR::new(14).unwrap(), &candles);
    drive(|| AwesomeOscillator::new(5, 34).unwrap(), &candles);
    drive(
        || AwesomeOscillatorHistogram::new(5, 34, 5).unwrap(),
        &candles,
    );
    drive(|| AcceleratorOscillator::new(5, 34, 5).unwrap(), &candles);
    drive(|| UltimateOscillator::new(7, 14, 28).unwrap(), &candles);
    drive(BalanceOfPower::new, &candles);

    // --- Volume ---
    drive(Obv::new, &candles);
    drive(|| Mfi::new(14).unwrap(), &candles);
    drive(Vwap::new, &candles);
    drive(|| RollingVwap::new(20).unwrap(), &candles);
    drive(|| Vwma::new(20).unwrap(), &candles);
    drive(|| Evwma::new(20).unwrap(), &candles);
    drive(Adl::new, &candles);
    drive(VolumePriceTrend::new, &candles);
    drive(|| ChaikinMoneyFlow::new(20).unwrap(), &candles);
    drive(|| ChaikinOscillator::new(3, 10).unwrap(), &candles);
    drive(|| ForceIndex::new(13).unwrap(), &candles);
    drive(|| EaseOfMovement::with_divisor(14, 1e8).unwrap(), &candles);
    drive(|| Kvo::new(34, 55).unwrap(), &candles);
    drive(|| VolumeOscillator::new(14, 28).unwrap(), &candles);
    drive(Nvi::new, &candles);
    drive(Pvi::new, &candles);
    drive(AdOscillator::new, &candles);
    drive(AnchoredVwap::new, &candles);
    drive(|| DemandIndex::new(10).unwrap(), &candles);
    drive(|| Tsv::new(18).unwrap(), &candles);
    drive(|| Vzo::new(14).unwrap(), &candles);
    drive(MarketFacilitationIndex::new, &candles);

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

    // --- Market Profile (multi-output) ---
    drive(|| ValueArea::new(20, 50, 0.70).unwrap(), &candles);
    drive(|| InitialBalance::new(12).unwrap(), &candles);
    drive(|| OpeningRange::new(6).unwrap(), &candles);

    // --- Ichimoku (5 lines, hand-rolled because of multi-Option output) ---
    {
        let mut ichi = Ichimoku::classic();
        for c in &candles {
            let _ = ichi.update(*c);
        }
        let _ = Ichimoku::classic().batch(&candles);
    }

    // --- Heikin-Ashi (4-field candle transform) ---
    {
        let mut ha = HeikinAshi::new();
        for c in &candles {
            let _ = ha.update(*c);
        }
        let _ = HeikinAshi::new().batch(&candles);
    }

    // --- DeMark family ---
    drive(|| TdSetup::new(4, 9).unwrap(), &candles);
    drive(|| TdDeMarker::new(14).unwrap(), &candles);
    drive(|| TdRei::new(5).unwrap(), &candles);
    drive(|| TdPressure::new(5).unwrap(), &candles);
    drive(|| TdCombo::new(4, 9, 2, 13).unwrap(), &candles);
    drive(|| TdCountdown::new(4, 9, 2, 13).unwrap(), &candles);
    drive(TdDifferential::new, &candles);
    drive(TdOpen::new, &candles);
    drive(TdRangeProjection::new, &candles);
    {
        let mut s = TdSequential::new(4, 9, 2, 13).unwrap();
        for c in &candles {
            let _ = s.update(*c);
        }
        let _ = TdSequential::new(4, 9, 2, 13).unwrap().batch(&candles);
    }
    {
        let mut s = TdLines::new(4, 9).unwrap();
        for c in &candles {
            let _ = s.update(*c);
        }
        let _ = TdLines::new(4, 9).unwrap().batch(&candles);
    }
    {
        let mut s = TdRiskLevel::new(4, 9).unwrap();
        for c in &candles {
            let _ = s.update(*c);
        }
        let _ = TdRiskLevel::new(4, 9).unwrap().batch(&candles);
    }

    // --- Pivots & Support/Resistance (multi-output) ---
    drive(ClassicPivots::new, &candles);
    drive(FibonacciPivots::new, &candles);
    drive(Camarilla::new, &candles);
    drive(WoodiePivots::new, &candles);
    drive(DemarkPivots::new, &candles);
    drive(WilliamsFractals::new, &candles);
    drive(|| ZigZag::new(0.05).unwrap(), &candles);

    // --- Donchian Stop (multi-output) ---
    {
        let mut s = DonchianStop::new(10).unwrap();
        for c in &candles {
            let _ = s.update(*c);
        }
        let _ = DonchianStop::new(10).unwrap().batch(&candles);
    }

    // --- Family 05: candle-input band/channel indicators (multi-output) ---
    {
        let mut ab = AccelerationBands::new(20, 0.001).unwrap();
        for c in &candles {
            let _ = ab.update(*c);
        }
        let _ = AccelerationBands::new(20, 0.001).unwrap().batch(&candles);
    }
    {
        let mut sb = StarcBands::new(6, 15, 2.0).unwrap();
        for c in &candles {
            let _ = sb.update(*c);
        }
        let _ = StarcBands::new(6, 15, 2.0).unwrap().batch(&candles);
    }
    {
        let mut atrb = AtrBands::new(14, 3.0).unwrap();
        for c in &candles {
            let _ = atrb.update(*c);
        }
        let _ = AtrBands::new(14, 3.0).unwrap().batch(&candles);
    }
    {
        let mut hc = HurstChannel::new(10, 0.5).unwrap();
        for c in &candles {
            let _ = hc.update(*c);
        }
        let _ = HurstChannel::new(10, 0.5).unwrap().batch(&candles);
    }
    {
        let mut ts = TtmSqueeze::new(20, 2.0, 1.5).unwrap();
        for c in &candles {
            let _ = ts.update(*c);
        }
        let _ = TtmSqueeze::new(20, 2.0, 1.5).unwrap().batch(&candles);
    }
    {
        let mut fc = FractalChaosBands::new(2).unwrap();
        for c in &candles {
            let _ = fc.update(*c);
        }
        let _ = FractalChaosBands::new(2).unwrap().batch(&candles);
    }
    {
        let mut vb = VwapStdDevBands::new(2.0).unwrap();
        for c in &candles {
            let _ = vb.update(*c);
        }
        let _ = VwapStdDevBands::new(2.0).unwrap().batch(&candles);
    }

    // --- Candlestick Patterns (family 14) ---
    drive(RickshawMan::new, &candles);
    drive(LongLeggedDoji::new, &candles);
    drive(GravestoneDoji::new, &candles);
    drive(DragonflyDoji::new, &candles);
    drive(DojiStar::new, &candles);
    drive(Counterattack::new, &candles);
    drive(Breakaway::new, &candles);
    drive(BeltHold::new, &candles);
    drive(AbandonedBaby::new, &candles);
    drive(AdvanceBlock::new, &candles);
    drive(Doji::new, &candles);
    drive(|| Doji::new().signed(), &candles);
    drive(Hammer::new, &candles);
    drive(InvertedHammer::new, &candles);
    drive(HangingMan::new, &candles);
    drive(ShootingStar::new, &candles);
    drive(Engulfing::new, &candles);
    drive(Harami::new, &candles);
    drive(MorningEveningStar::new, &candles);
    drive(ThreeSoldiersOrCrows::new, &candles);
    drive(ThreeStarsInSouth::new, &candles);
    drive(PiercingDarkCloud::new, &candles);
    drive(Marubozu::new, &candles);
    drive(Tweezer::new, &candles);
    drive(SpinningTop::new, &candles);
    drive(ThreeInside::new, &candles);
    drive(ThreeLineStrike::new, &candles);
    drive(ThreeOutside::new, &candles);
    drive(TwoCrows::new, &candles);
    drive(UpsideGapTwoCrows::new, &candles);
    drive(IdenticalThreeCrows::new, &candles);
});
