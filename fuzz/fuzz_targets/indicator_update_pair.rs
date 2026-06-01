#![no_main]
//! Fuzz two-input `Indicator<(f64, f64)>` implementations with arbitrary
//! `(asset, benchmark)` return pairs.
//!
//! Each iteration consumes a byte stream and interprets it as a sequence of
//! `(f64, f64)` pairs (8 bytes per `f64`), then drives every two-series
//! indicator over the sequence both streaming and as a batch. No path may
//! panic.

use libfuzzer_sys::fuzz_target;
use wickra_core::{Alpha, BatchExt, Indicator, InformationRatio, PairwiseBeta, TreynorRatio};

#[inline(never)]
fn drive<I>(make: impl Fn() -> I, data: &[(f64, f64)])
where
    I: Indicator<Input = (f64, f64), Output = f64> + BatchExt,
{
    let mut streaming = make();
    for &x in data {
        let _ = streaming.update(x);
    }
    let _ = make().batch(data);
}

fuzz_target!(|data: &[u8]| {
    // Pack two consecutive 8-byte chunks into one `(f64, f64)` pair.
    let pairs: Vec<(f64, f64)> = data
        .chunks_exact(16)
        .map(|c| {
            let a = f64::from_le_bytes(c[..8].try_into().expect("8 bytes"));
            let b = f64::from_le_bytes(c[8..].try_into().expect("8 bytes"));
            (a, b)
        })
        .collect();

    drive(|| TreynorRatio::new(10, 0.0).unwrap(), &pairs);
    drive(|| InformationRatio::new(10).unwrap(), &pairs);
    drive(|| Alpha::new(10, 0.0).unwrap(), &pairs);
    drive(|| PairwiseBeta::new(10).unwrap(), &pairs);
});
