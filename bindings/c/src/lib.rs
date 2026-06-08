//! C ABI for Wickra — the hub every C-capable language (C, C++, Go, C#, Java, R)
//! links against. Each indicator is exposed as a set of `extern "C"` functions
//! over an opaque handle:
//!
//! - `wickra_<ind>_new(...)` — construct; returns `NULL` on invalid parameters.
//! - `wickra_<ind>_update(h, value)` — feed one point; returns the output, or
//!   `NaN` while warming up or if `h` is `NULL`.
//! - `wickra_<ind>_batch(h, in, out, n)` — write one output per input into the
//!   caller-owned `out` buffer (length `n`), `NaN` at warmup positions.
//! - `wickra_<ind>_reset(h)` — clear all state.
//! - `wickra_<ind>_free(h)` — destroy the handle. Every `_new` must be paired
//!   with exactly one `_free`; there is no RAII across the C boundary.
//!
//! The bodies are written as plain `#[no_mangle]` functions (not a `macro_rules!`)
//! so cbindgen can see them on stable Rust without macro expansion. They are
//! generated mechanically by the `ScriptHelpers` `capi` wrapper.

use core::ptr;
use core::slice;
use wickra_core::{Indicator, Sma};

/// Create an `SMA` indicator with the given period.
///
/// Returns `NULL` if the period is invalid (e.g. zero). The returned handle must
/// be released exactly once with [`wickra_sma_free`].
#[no_mangle]
pub extern "C" fn wickra_sma_new(period: usize) -> *mut Sma {
    match Sma::new(period) {
        Ok(ind) => Box::into_raw(Box::new(ind)),
        Err(_) => ptr::null_mut(),
    }
}

/// Feed one value and return the freshly computed output, or `NaN` while the
/// indicator is still warming up or if `handle` is `NULL`.
///
/// # Safety
/// `handle` must be a valid pointer returned by [`wickra_sma_new`] and not yet
/// freed, or `NULL`.
#[no_mangle]
pub unsafe extern "C" fn wickra_sma_update(handle: *mut Sma, value: f64) -> f64 {
    match handle.as_mut() {
        Some(ind) => ind.update(value).unwrap_or(f64::NAN),
        None => f64::NAN,
    }
}

/// Run the indicator over `input[0..n]`, writing one output per input into
/// `out[0..n]`. Warmup positions are written as `NaN`. No-op if any pointer is
/// `NULL`.
///
/// # Safety
/// `handle` must be valid (from [`wickra_sma_new`], not freed). `input` and `out`
/// must each point to at least `n` readable / writable `double`s.
#[no_mangle]
pub unsafe extern "C" fn wickra_sma_batch(
    handle: *mut Sma,
    input: *const f64,
    out: *mut f64,
    n: usize,
) {
    if handle.is_null() || input.is_null() || out.is_null() {
        return;
    }
    let ind = &mut *handle;
    let inputs = slice::from_raw_parts(input, n);
    let outputs = slice::from_raw_parts_mut(out, n);
    for (slot, &value) in outputs.iter_mut().zip(inputs) {
        *slot = ind.update(value).unwrap_or(f64::NAN);
    }
}

/// Reset all internal state, equivalent to a freshly constructed indicator.
/// No-op if `handle` is `NULL`.
///
/// # Safety
/// `handle` must be valid (from [`wickra_sma_new`], not freed), or `NULL`.
#[no_mangle]
pub unsafe extern "C" fn wickra_sma_reset(handle: *mut Sma) {
    if let Some(ind) = handle.as_mut() {
        ind.reset();
    }
}

/// Destroy a handle created by [`wickra_sma_new`]. No-op if `handle` is `NULL`.
///
/// # Safety
/// `handle` must have been returned by [`wickra_sma_new`] and not previously
/// freed, or `NULL`. Using `handle` after this call is undefined behavior.
#[no_mangle]
pub unsafe extern "C" fn wickra_sma_free(handle: *mut Sma) {
    if !handle.is_null() {
        drop(Box::from_raw(handle));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_rejects_zero_period() {
        assert!(wickra_sma_new(0).is_null());
    }

    #[test]
    fn streaming_batch_and_lifecycle() {
        let handle = wickra_sma_new(3);
        assert!(!handle.is_null());
        unsafe {
            assert!(wickra_sma_update(handle, 1.0).is_nan());
            assert!(wickra_sma_update(handle, 2.0).is_nan());
            assert!((wickra_sma_update(handle, 3.0) - 2.0).abs() < 1e-9);

            wickra_sma_reset(handle);
            let input = [1.0_f64, 2.0, 3.0, 4.0, 5.0];
            let mut out = [0.0_f64; 5];
            wickra_sma_batch(handle, input.as_ptr(), out.as_mut_ptr(), 5);
            assert!(out[0].is_nan());
            assert!(out[1].is_nan());
            assert!((out[2] - 2.0).abs() < 1e-9);
            assert!((out[4] - 4.0).abs() < 1e-9);

            wickra_sma_free(handle);
        }
    }

    #[test]
    fn null_handle_is_a_defined_noop() {
        unsafe {
            assert!(wickra_sma_update(ptr::null_mut(), 1.0).is_nan());
            wickra_sma_reset(ptr::null_mut());
            wickra_sma_free(ptr::null_mut());
            let mut out = [0.0_f64; 1];
            wickra_sma_batch(ptr::null_mut(), ptr::null(), out.as_mut_ptr(), 1);
        }
    }
}
