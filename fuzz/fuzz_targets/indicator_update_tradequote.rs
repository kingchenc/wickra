#![no_main]
//! Fuzz price-impact `Indicator<Input = TradeQuote>` implementations with
//! arbitrary trade-quote tapes.
//!
//! Each iteration consumes a byte stream, interprets it as a sequence of `f64`
//! values (8 bytes each), and packs consecutive triples into `(price, size,
//! mid)` trade-quotes whose aggressor side alternates with the sign of the size
//! field. Trade-quotes are built with the `new_unchecked` constructors so the
//! fuzzer can explore degenerate values (non-finite, negative, zero mid) that
//! the validating constructors would reject — the indicators must never panic,
//! streaming or batched.

use libfuzzer_sys::fuzz_target;
use wickra_core::{
    BatchExt, EffectiveSpread, Indicator, KylesLambda, RealizedSpread, Side, Trade, TradeQuote,
};

#[inline(never)]
fn drive<I>(make: impl Fn() -> I, quotes: &[TradeQuote])
where
    I: Indicator<Input = TradeQuote, Output = f64> + BatchExt,
{
    let mut streaming = make();
    for &quote in quotes {
        let _ = streaming.update(quote);
    }
    let _ = make().batch(quotes);
}

fuzz_target!(|data: &[u8]| {
    let floats: Vec<f64> = data
        .chunks_exact(8)
        .map(|c| f64::from_le_bytes(c.try_into().expect("8 bytes")))
        .collect();
    let quotes: Vec<TradeQuote> = floats
        .chunks_exact(3)
        .map(|c| {
            let side = if c[1] >= 0.0 { Side::Buy } else { Side::Sell };
            let trade = Trade::new_unchecked(c[0], c[1], side, 0);
            TradeQuote::new_unchecked(trade, c[2])
        })
        .collect();

    drive(EffectiveSpread::new, &quotes);
    drive(|| RealizedSpread::new(5).unwrap(), &quotes);
    drive(|| KylesLambda::new(5).unwrap(), &quotes);
});
