#![no_main]
//! Fuzz composed indicator chains with arbitrary `f64` sequences.
//!
//! [`Chain`](wickra_core::Chain) feeds one indicator's output into the next, so
//! it exercises a path no single-indicator target reaches: the second stage sees
//! a stream produced by the first, not the raw input. That stream is a different
//! shape — it starts later (the first stage withholds during its own warmup), it
//! can be constant for long stretches, and for indicators that emit ratios or
//! oscillators it is bounded in ways raw prices are not. A stage that copes with
//! arbitrary prices can still be surprised by it.
//!
//! Chains also compose `warmup_period` and `is_ready` across stages, which is
//! arithmetic on values the fuzzer controls indirectly.
//!
//! Every chain here must tolerate any input stream — NaN, ±inf, subnormals,
//! abrupt jumps — without panicking, through both the streaming and the batch
//! path.

use libfuzzer_sys::fuzz_target;
use wickra_core::{BatchExt, Chain, Ema, Indicator, Rsi, Roc, Sma, StdDev, Wma, ZScore};

/// Drive one chain through streaming and batch. `#[inline(never)]` so a panic
/// backtrace names the composition that produced it.
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

    // Readiness and warmup are composed across stages; reading them at every
    // point of the stream covers the arithmetic that composition performs.
    let mut probe = make();
    let _ = probe.warmup_period();
    for &x in data {
        let _ = probe.update(x);
        let _ = probe.is_ready();
    }
    probe.reset();
    let _ = probe.is_ready();
}

fuzz_target!(|data: Vec<f64>| {
    // Two stages: a smoother feeding an oscillator, and the reverse. The second
    // ordering is the interesting one — an oscillator's output is bounded and
    // often constant, which is not what a smoother's window normally holds.
    drive(
        || Chain::new(Ema::new(20).unwrap(), Rsi::new(14).unwrap()),
        &data,
    );
    drive(
        || Chain::new(Rsi::new(14).unwrap(), Ema::new(20).unwrap()),
        &data,
    );

    // A rate of change fed into a dispersion measure: the first stage divides by
    // its own previous value, so it manufactures infinities and NaNs out of
    // finite input for the second stage to handle.
    drive(
        || Chain::new(Roc::new(10).unwrap(), StdDev::new(14).unwrap()),
        &data,
    );

    // Dispersion into a z-score: the second stage divides by a dispersion that
    // the first stage can drive to exactly zero.
    drive(
        || Chain::new(StdDev::new(14).unwrap(), ZScore::new(20).unwrap()),
        &data,
    );

    // Three stages, built through `then`, so the composed warmup is nested.
    drive(
        || {
            Chain::new(Sma::new(5).unwrap(), Ema::new(10).unwrap())
                .then(Wma::new(20).unwrap())
        },
        &data,
    );
});
