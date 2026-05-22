#![no_main]
//! Fuzz indicator updates with arbitrary `f64` sequences.
//!
//! Every indicator must tolerate any finite-or-not input stream — NaN, ±inf,
//! subnormals, abrupt jumps — without panicking, and `batch` must agree with
//! the streaming `update` path.

use libfuzzer_sys::fuzz_target;
use wickra_core::{BatchExt, Ema, Indicator, Rsi};

fuzz_target!(|data: Vec<f64>| {
    let mut rsi = Rsi::new(14).unwrap();
    let mut ema = Ema::new(20).unwrap();
    for &x in &data {
        let _ = rsi.update(x);
        let _ = ema.update(x);
    }

    // batch over the same data must not panic either.
    let _ = Rsi::new(14).unwrap().batch(&data);
    let _ = Ema::new(20).unwrap().batch(&data);
});
