#![no_main]
//! Fuzz trade-flow `Indicator<Input = Trade>` implementations with arbitrary
//! trade tapes.
//!
//! Each iteration consumes a byte stream, interprets it as a sequence of `f64`
//! values (8 bytes each), and packs consecutive values into `(price, size)`
//! trades whose aggressor side alternates with the sign of the size field.
//! Trades are built with `Trade::new_unchecked` so the fuzzer can explore
//! degenerate values (non-finite, negative) that the validating constructor
//! would reject — the indicators must never panic, streaming or batched.

use libfuzzer_sys::fuzz_target;
use wickra_core::{AmihudIlliquidity, BatchExt, CumulativeVolumeDelta, Footprint, Indicator, RollMeasure, Side, SignedVolume, Trade, TradeImbalance, Vpin};

#[inline(never)]
fn drive<I>(make: impl Fn() -> I, trades: &[Trade])
where
    I: Indicator<Input = Trade, Output = f64> + BatchExt,
{
    let mut streaming = make();
    for &trade in trades {
        let _ = streaming.update(trade);
    }
    let _ = make().batch(trades);
}

fuzz_target!(|data: &[u8]| {
    let floats: Vec<f64> = data
        .chunks_exact(8)
        .map(|c| f64::from_le_bytes(c.try_into().expect("8 bytes")))
        .collect();
    let trades: Vec<Trade> = floats
        .chunks_exact(2)
        .map(|c| {
            let side = if c[1] >= 0.0 { Side::Buy } else { Side::Sell };
            Trade::new_unchecked(c[0], c[1], side, 0)
        })
        .collect();

    drive(SignedVolume::new, &trades);
    drive(CumulativeVolumeDelta::new, &trades);
    drive(|| TradeImbalance::new(5).unwrap(), &trades);
    drive(|| Vpin::new(8.0, 5).unwrap(), &trades);
    drive(|| AmihudIlliquidity::new(20).unwrap(), &trades);
    drive(|| RollMeasure::new(20).unwrap(), &trades);

    // Footprint emits a variable-length `FootprintOutput` rather than an `f64`,
    // so it is driven directly rather than through the scalar-output helper.
    let mut footprint = Footprint::new(0.5).unwrap();
    for &trade in &trades {
        let _ = footprint.update(trade);
    }
    footprint.reset();
    let _ = Footprint::new(0.5).unwrap().batch(&trades);
});
