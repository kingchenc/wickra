#![no_main]
//! Fuzz the alt-chart bar builders (Renko, Kagi, Point-and-Figure) with arbitrary
//! candle streams.
//!
//! Each fuzz iteration runs the same candle sequence through every bar builder
//! twice — once as a streaming `update` loop, once as a full `batch` call — and
//! asserts neither path panics. Renko's brick count per candle is proportional
//! to `(price_move / box_size)`, so the candle magnitudes are bounded here (a
//! sensibly scaled `box_size` relative to the instrument price is a documented
//! precondition); the fuzzer still freely varies ordering, gaps, equal prices
//! and zero ranges within that band.

use libfuzzer_sys::fuzz_target;
use wickra_core::{BarBuilder, Candle, KagiBars, PointAndFigureBars, RenkoBars};

/// Reinterpret the fuzz bytes as `[open, high, low, close, volume]` groups,
/// keeping only structurally-valid candles whose magnitudes stay in a band that
/// keeps Renko's brick count bounded.
fn candles_from(data: &[f64]) -> Vec<Candle> {
    data.chunks_exact(5)
        .enumerate()
        .filter_map(|(i, ch)| Candle::new(ch[0], ch[1], ch[2], ch[3], ch[4], i as i64).ok())
        .filter(|candle| candle.high.abs() < 1.0e4 && candle.low.abs() < 1.0e4)
        .collect()
}

#[inline(never)]
fn drive<B: BarBuilder>(mut builder: B, candles: &[Candle]) {
    for candle in candles {
        let _ = builder.update(*candle);
    }
}

fuzz_target!(|data: Vec<f64>| {
    let candles = candles_from(&data);
    if candles.is_empty() {
        return;
    }

    drive(RenkoBars::new(5.0).unwrap(), &candles);
    drive(KagiBars::new(5.0).unwrap(), &candles);
    drive(PointAndFigureBars::new(5.0, 3).unwrap(), &candles);

    let _ = RenkoBars::new(5.0).unwrap().batch(&candles);
    let _ = KagiBars::new(5.0).unwrap().batch(&candles);
    let _ = PointAndFigureBars::new(5.0, 3).unwrap().batch(&candles);
});
