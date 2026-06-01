//! Python bindings for Wickra. Built with `PyO3` and exposed under the `wickra` package.
//!
//! This module is the thin glue between `wickra-core` and Python. Every indicator
//! has both a streaming class and a batch helper that takes a `NumPy` array.

#![allow(clippy::needless_pass_by_value)]
// Python `__repr__` is an instance method by protocol, so the `&self` parameter is
// mandatory even when its body does not read state (e.g. parameterless indicators
// like `TypicalPrice`). Clippy's `unused_self` triggers on those signatures.
#![allow(clippy::unused_self)]
// OHLCV batch helpers bind the conventional single-letter column names
// (o/h/l/c/v) that match the domain and the NumPy call sites.
#![allow(clippy::many_single_char_names)]

use numpy::{IntoPyArray, PyArray1, PyArray2, PyReadonlyArray1};
use pyo3::exceptions::{PyTypeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::PyDict;
use wickra_core as wc;
use wickra_core::{BatchExt, Indicator};

fn map_err(e: wc::Error) -> PyErr {
    match e {
        wc::Error::PeriodZero
        | wc::Error::InvalidPeriod { .. }
        | wc::Error::NonPositiveMultiplier
        | wc::Error::NonFiniteInput
        | wc::Error::InvalidCandle { .. }
        | wc::Error::InvalidTick { .. }
        | wc::Error::InvalidOrderBook { .. }
        | wc::Error::InvalidTrade { .. }
        | wc::Error::InvalidDerivatives { .. } => PyValueError::new_err(e.to_string()),
    }
}

fn opt_to_nan(v: Option<f64>) -> f64 {
    v.unwrap_or(f64::NAN)
}

/// Convert a slice of `Option<f64>` to a flat `Vec<f64>` with NaNs for warmup.
fn flatten(values: Vec<Option<f64>>) -> Vec<f64> {
    values.into_iter().map(opt_to_nan).collect()
}

/// Raised instead of panicking when a `NumPy` input is not C-contiguous.
const NON_CONTIGUOUS: &str = "array must be C-contiguous; pass np.ascontiguousarray(arr)";

/// `(pp, r1, r2, r3, s1, s2, s3)` pivot levels returned by Classic/Fibonacci pivots.
type PivotLevels = (f64, f64, f64, f64, f64, f64, f64);
/// `(pp, r1, r2, s1, s2)` pivot levels returned by Woodie pivots.
type WoodieLevels = (f64, f64, f64, f64, f64);
/// `(tenkan, kijun, senkou_a, senkou_b, chikou)` Ichimoku lines, each optional during warmup.
type IchimokuLines = (
    Option<f64>,
    Option<f64>,
    Option<f64>,
    Option<f64>,
    Option<f64>,
);

// ============================== SMA ==============================

#[pyclass(name = "SMA", module = "wickra._wickra", skip_from_py_object)]
#[derive(Clone)]
struct PySma {
    inner: wc::Sma,
}

#[pymethods]
impl PySma {
    #[new]
    fn new(period: usize) -> PyResult<Self> {
        Ok(Self {
            inner: wc::Sma::new(period).map_err(map_err)?,
        })
    }
    fn update(&mut self, value: f64) -> Option<f64> {
        self.inner.update(value)
    }
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        prices: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let slice = prices
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        Ok(flatten(self.inner.batch(slice)).into_pyarray(py))
    }
    #[getter]
    fn period(&self) -> usize {
        self.inner.period()
    }
    #[getter]
    fn value(&self) -> Option<f64> {
        self.inner.value()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        format!("SMA(period={})", self.inner.period())
    }
}

// ============================== EMA ==============================

#[pyclass(name = "EMA", module = "wickra._wickra", skip_from_py_object)]
#[derive(Clone)]
struct PyEma {
    inner: wc::Ema,
}

#[pymethods]
impl PyEma {
    #[new]
    fn new(period: usize) -> PyResult<Self> {
        Ok(Self {
            inner: wc::Ema::new(period).map_err(map_err)?,
        })
    }
    fn update(&mut self, value: f64) -> Option<f64> {
        self.inner.update(value)
    }
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        prices: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let slice = prices
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        Ok(flatten(self.inner.batch(slice)).into_pyarray(py))
    }
    #[getter]
    fn period(&self) -> usize {
        self.inner.period()
    }
    #[getter]
    fn alpha(&self) -> f64 {
        self.inner.alpha()
    }
    #[getter]
    fn value(&self) -> Option<f64> {
        self.inner.value()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        format!("EMA(period={})", self.inner.period())
    }
}

// ============================== WMA ==============================

#[pyclass(name = "WMA", module = "wickra._wickra", skip_from_py_object)]
#[derive(Clone)]
struct PyWma {
    inner: wc::Wma,
}

#[pymethods]
impl PyWma {
    #[new]
    fn new(period: usize) -> PyResult<Self> {
        Ok(Self {
            inner: wc::Wma::new(period).map_err(map_err)?,
        })
    }
    fn update(&mut self, value: f64) -> Option<f64> {
        self.inner.update(value)
    }
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        prices: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let slice = prices
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        Ok(flatten(self.inner.batch(slice)).into_pyarray(py))
    }
    #[getter]
    fn period(&self) -> usize {
        self.inner.period()
    }
    #[getter]
    fn value(&self) -> Option<f64> {
        self.inner.value()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        format!("WMA(period={})", self.inner.period())
    }
}

// ============================== RSI ==============================

#[pyclass(name = "RSI", module = "wickra._wickra", skip_from_py_object)]
#[derive(Clone)]
struct PyRsi {
    inner: wc::Rsi,
}

#[pymethods]
impl PyRsi {
    #[new]
    #[pyo3(signature = (period=14))]
    fn new(period: usize) -> PyResult<Self> {
        Ok(Self {
            inner: wc::Rsi::new(period).map_err(map_err)?,
        })
    }
    fn update(&mut self, value: f64) -> Option<f64> {
        self.inner.update(value)
    }
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        prices: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let slice = prices
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        Ok(flatten(self.inner.batch(slice)).into_pyarray(py))
    }
    #[getter]
    fn period(&self) -> usize {
        self.inner.period()
    }
    #[getter]
    fn value(&self) -> Option<f64> {
        self.inner.value()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        format!("RSI(period={})", self.inner.period())
    }
}

// ============================== MACD ==============================

#[pyclass(name = "MACD", module = "wickra._wickra", skip_from_py_object)]
#[derive(Clone)]
struct PyMacd {
    inner: wc::MacdIndicator,
}

#[pymethods]
impl PyMacd {
    #[new]
    #[pyo3(signature = (fast=12, slow=26, signal=9))]
    fn new(fast: usize, slow: usize, signal: usize) -> PyResult<Self> {
        Ok(Self {
            inner: wc::MacdIndicator::new(fast, slow, signal).map_err(map_err)?,
        })
    }
    /// Returns `(macd, signal, histogram)` or `None` during warmup.
    fn update(&mut self, value: f64) -> Option<(f64, f64, f64)> {
        self.inner
            .update(value)
            .map(|o| (o.macd, o.signal, o.histogram))
    }
    /// Batch over a numpy array of closes. Returns a 2D array of shape `(n, 3)`
    /// with columns `[macd, signal, histogram]`. Warmup rows are NaN.
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        prices: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray2<f64>>> {
        let slice = prices
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let n = slice.len();
        let mut out = vec![f64::NAN; n * 3];
        for (i, p) in slice.iter().enumerate() {
            if let Some(o) = self.inner.update(*p) {
                out[i * 3] = o.macd;
                out[i * 3 + 1] = o.signal;
                out[i * 3 + 2] = o.histogram;
            }
        }
        Ok(numpy::ndarray::Array2::from_shape_vec((n, 3), out)
            .expect("shape consistent")
            .into_pyarray(py))
    }
    #[getter]
    fn periods(&self) -> (usize, usize, usize) {
        self.inner.periods()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        let (f, s, sig) = self.inner.periods();
        format!("MACD(fast={f}, slow={s}, signal={sig})")
    }
}

// ============================== Bollinger Bands ==============================

#[pyclass(
    name = "BollingerBands",
    module = "wickra._wickra",
    skip_from_py_object
)]
#[derive(Clone)]
struct PyBb {
    inner: wc::BollingerBands,
}

#[pymethods]
impl PyBb {
    #[new]
    #[pyo3(signature = (period=20, multiplier=2.0))]
    fn new(period: usize, multiplier: f64) -> PyResult<Self> {
        Ok(Self {
            inner: wc::BollingerBands::new(period, multiplier).map_err(map_err)?,
        })
    }
    /// Returns `(upper, middle, lower, stddev)` or `None` during warmup.
    fn update(&mut self, value: f64) -> Option<(f64, f64, f64, f64)> {
        self.inner
            .update(value)
            .map(|o| (o.upper, o.middle, o.lower, o.stddev))
    }
    /// Batch returns shape `(n, 4)` columns `[upper, middle, lower, stddev]`.
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        prices: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray2<f64>>> {
        let slice = prices
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let n = slice.len();
        let mut out = vec![f64::NAN; n * 4];
        for (i, p) in slice.iter().enumerate() {
            if let Some(o) = self.inner.update(*p) {
                out[i * 4] = o.upper;
                out[i * 4 + 1] = o.middle;
                out[i * 4 + 2] = o.lower;
                out[i * 4 + 3] = o.stddev;
            }
        }
        Ok(numpy::ndarray::Array2::from_shape_vec((n, 4), out)
            .expect("shape consistent")
            .into_pyarray(py))
    }
    #[getter]
    fn period(&self) -> usize {
        self.inner.period()
    }
    #[getter]
    fn multiplier(&self) -> f64 {
        self.inner.multiplier()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        format!(
            "BollingerBands(period={}, multiplier={})",
            self.inner.period(),
            self.inner.multiplier()
        )
    }
}

// ============================== ATR ==============================

fn extract_candle(d: &Bound<'_, PyAny>) -> PyResult<wc::Candle> {
    // Accept either a dict-like with open/high/low/close/volume/timestamp,
    // or a tuple (open, high, low, close, volume, timestamp).
    if let Ok(tup) = d.extract::<(f64, f64, f64, f64, f64, i64)>() {
        return wc::Candle::new(tup.0, tup.1, tup.2, tup.3, tup.4, tup.5).map_err(map_err);
    }
    if let Ok(dict) = d.cast::<PyDict>() {
        let g = |k: &str| -> PyResult<f64> {
            dict.get_item(k)?
                .ok_or_else(|| PyValueError::new_err(format!("candle missing key '{k}'")))?
                .extract::<f64>()
        };
        let ts = dict
            .get_item("timestamp")?
            .map(|v| v.extract::<i64>())
            .transpose()?
            .unwrap_or(0);
        return wc::Candle::new(
            g("open")?,
            g("high")?,
            g("low")?,
            g("close")?,
            g("volume")?,
            ts,
        )
        .map_err(map_err);
    }
    Err(PyTypeError::new_err(
        "candle must be a 6-tuple (open, high, low, close, volume, timestamp) or a dict",
    ))
}

#[pyclass(name = "ATR", module = "wickra._wickra", skip_from_py_object)]
#[derive(Clone)]
struct PyAtr {
    inner: wc::Atr,
}

#[pymethods]
impl PyAtr {
    #[new]
    #[pyo3(signature = (period=14))]
    fn new(period: usize) -> PyResult<Self> {
        Ok(Self {
            inner: wc::Atr::new(period).map_err(map_err)?,
        })
    }
    fn update(&mut self, candle: &Bound<'_, PyAny>) -> PyResult<Option<f64>> {
        let c = extract_candle(candle)?;
        Ok(self.inner.update(c))
    }
    /// Batch over numpy columns: high, low, close (all 1-D, equal length).
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        high: PyReadonlyArray1<'py, f64>,
        low: PyReadonlyArray1<'py, f64>,
        close: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let h = high
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let l = low
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let c = close
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        if h.len() != l.len() || l.len() != c.len() {
            return Err(PyValueError::new_err(
                "high, low, close must be equal length",
            ));
        }
        let mut out = Vec::with_capacity(h.len());
        for i in 0..h.len() {
            let candle = wc::Candle::new(c[i], h[i], l[i], c[i], 0.0, 0).map_err(map_err)?;
            out.push(self.inner.update(candle).unwrap_or(f64::NAN));
        }
        Ok(out.into_pyarray(py))
    }
    #[getter]
    fn period(&self) -> usize {
        self.inner.period()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        format!("ATR(period={})", self.inner.period())
    }
}

// ============================== Stochastic ==============================

#[pyclass(name = "Stochastic", module = "wickra._wickra", skip_from_py_object)]
#[derive(Clone)]
struct PyStoch {
    inner: wc::Stochastic,
}

#[pymethods]
impl PyStoch {
    #[new]
    #[pyo3(signature = (k_period=14, d_period=3))]
    fn new(k_period: usize, d_period: usize) -> PyResult<Self> {
        Ok(Self {
            inner: wc::Stochastic::new(k_period, d_period).map_err(map_err)?,
        })
    }
    fn update(&mut self, candle: &Bound<'_, PyAny>) -> PyResult<Option<(f64, f64)>> {
        let c = extract_candle(candle)?;
        Ok(self.inner.update(c).map(|o| (o.k, o.d)))
    }
    /// Batch over high/low/close numpy columns. Returns shape `(n, 2)` for `[k, d]`.
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        high: PyReadonlyArray1<'py, f64>,
        low: PyReadonlyArray1<'py, f64>,
        close: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray2<f64>>> {
        let h = high
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let l = low
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let c = close
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        if h.len() != l.len() || l.len() != c.len() {
            return Err(PyValueError::new_err(
                "high, low, close must be equal length",
            ));
        }
        let n = h.len();
        let mut out = vec![f64::NAN; n * 2];
        for i in 0..n {
            let candle = wc::Candle::new(c[i], h[i], l[i], c[i], 0.0, 0).map_err(map_err)?;
            if let Some(o) = self.inner.update(candle) {
                out[i * 2] = o.k;
                out[i * 2 + 1] = o.d;
            }
        }
        Ok(numpy::ndarray::Array2::from_shape_vec((n, 2), out)
            .expect("shape consistent")
            .into_pyarray(py))
    }
    #[getter]
    fn periods(&self) -> (usize, usize) {
        self.inner.periods()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        let (k, d) = self.inner.periods();
        format!("Stochastic(k_period={k}, d_period={d})")
    }
}

// ============================== OBV ==============================

#[pyclass(name = "OBV", module = "wickra._wickra", skip_from_py_object)]
#[derive(Clone)]
struct PyObv {
    inner: wc::Obv,
}

#[pymethods]
impl PyObv {
    #[new]
    fn new() -> Self {
        Self {
            inner: wc::Obv::new(),
        }
    }
    fn update(&mut self, candle: &Bound<'_, PyAny>) -> PyResult<Option<f64>> {
        let c = extract_candle(candle)?;
        Ok(self.inner.update(c))
    }
    /// Batch over numpy close + volume arrays.
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        close: PyReadonlyArray1<'py, f64>,
        volume: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let c = close
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let v = volume
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        if c.len() != v.len() {
            return Err(PyValueError::new_err(
                "close and volume must be equal length",
            ));
        }
        let mut out = Vec::with_capacity(c.len());
        for i in 0..c.len() {
            let candle = wc::Candle::new(c[i], c[i], c[i], c[i], v[i], 0).map_err(map_err)?;
            out.push(self.inner.update(candle).unwrap_or(f64::NAN));
        }
        Ok(out.into_pyarray(py))
    }
    #[getter]
    fn value(&self) -> Option<f64> {
        self.inner.value()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        "OBV()".to_string()
    }
}

// ============================== DEMA ==============================

#[pyclass(name = "DEMA", module = "wickra._wickra", skip_from_py_object)]
#[derive(Clone)]
struct PyDema {
    inner: wc::Dema,
}

#[pymethods]
impl PyDema {
    #[new]
    fn new(period: usize) -> PyResult<Self> {
        Ok(Self {
            inner: wc::Dema::new(period).map_err(map_err)?,
        })
    }
    fn update(&mut self, value: f64) -> Option<f64> {
        self.inner.update(value)
    }
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        prices: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let s = prices
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        Ok(flatten(self.inner.batch(s)).into_pyarray(py))
    }
    #[getter]
    fn period(&self) -> usize {
        self.inner.period()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        format!("DEMA(period={})", self.inner.period())
    }
}

// ============================== TEMA ==============================

#[pyclass(name = "TEMA", module = "wickra._wickra", skip_from_py_object)]
#[derive(Clone)]
struct PyTema {
    inner: wc::Tema,
}

#[pymethods]
impl PyTema {
    #[new]
    fn new(period: usize) -> PyResult<Self> {
        Ok(Self {
            inner: wc::Tema::new(period).map_err(map_err)?,
        })
    }
    fn update(&mut self, value: f64) -> Option<f64> {
        self.inner.update(value)
    }
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        prices: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let s = prices
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        Ok(flatten(self.inner.batch(s)).into_pyarray(py))
    }
    #[getter]
    fn period(&self) -> usize {
        self.inner.period()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        format!("TEMA(period={})", self.inner.period())
    }
}

// ============================== HMA ==============================

#[pyclass(name = "HMA", module = "wickra._wickra", skip_from_py_object)]
#[derive(Clone)]
struct PyHma {
    inner: wc::Hma,
}

#[pymethods]
impl PyHma {
    #[new]
    fn new(period: usize) -> PyResult<Self> {
        Ok(Self {
            inner: wc::Hma::new(period).map_err(map_err)?,
        })
    }
    fn update(&mut self, value: f64) -> Option<f64> {
        self.inner.update(value)
    }
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        prices: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let s = prices
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        Ok(flatten(self.inner.batch(s)).into_pyarray(py))
    }
    #[getter]
    fn period(&self) -> usize {
        self.inner.period()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        format!("HMA(period={})", self.inner.period())
    }
}

// ============================== KAMA ==============================

#[pyclass(name = "KAMA", module = "wickra._wickra", skip_from_py_object)]
#[derive(Clone)]
struct PyKama {
    inner: wc::Kama,
}

#[pymethods]
impl PyKama {
    #[new]
    #[pyo3(signature = (er_period=10, fast=2, slow=30))]
    fn new(er_period: usize, fast: usize, slow: usize) -> PyResult<Self> {
        Ok(Self {
            inner: wc::Kama::new(er_period, fast, slow).map_err(map_err)?,
        })
    }
    fn update(&mut self, value: f64) -> Option<f64> {
        self.inner.update(value)
    }
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        prices: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let s = prices
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        Ok(flatten(self.inner.batch(s)).into_pyarray(py))
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        "KAMA".to_string()
    }
}

// ============================== Inertia ==============================

#[pyclass(name = "Inertia", module = "wickra._wickra", skip_from_py_object)]
#[derive(Clone)]
struct PyInertia {
    inner: wc::Inertia,
}

#[pymethods]
impl PyInertia {
    #[new]
    #[pyo3(signature = (rvi_period=14, linreg_period=20))]
    fn new(rvi_period: usize, linreg_period: usize) -> PyResult<Self> {
        Ok(Self {
            inner: wc::Inertia::new(rvi_period, linreg_period).map_err(map_err)?,
        })
    }
    fn update(&mut self, candle: &Bound<'_, PyAny>) -> PyResult<Option<f64>> {
        let c = extract_candle(candle)?;
        Ok(self.inner.update(c))
    }
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        open: PyReadonlyArray1<'py, f64>,
        high: PyReadonlyArray1<'py, f64>,
        low: PyReadonlyArray1<'py, f64>,
        close: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let o = open
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let h = high
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let l = low
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let c = close
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        if !(o.len() == h.len() && h.len() == l.len() && l.len() == c.len()) {
            return Err(PyValueError::new_err(
                "open, high, low and close must be equal length",
            ));
        }
        let mut out = Vec::with_capacity(c.len());
        for i in 0..c.len() {
            let candle = wc::Candle::new(o[i], h[i], l[i], c[i], 0.0, 0).map_err(map_err)?;
            out.push(self.inner.update(candle).unwrap_or(f64::NAN));
        }
        Ok(out.into_pyarray(py))
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        let (r, l) = self.inner.periods();
        format!("Inertia(rvi_period={r}, linreg_period={l})")
    }
}

// ============================== Connors RSI ==============================

#[pyclass(name = "ConnorsRSI", module = "wickra._wickra", skip_from_py_object)]
#[derive(Clone)]
struct PyConnorsRsi {
    inner: wc::ConnorsRsi,
}

#[pymethods]
impl PyConnorsRsi {
    #[new]
    #[pyo3(signature = (period_rsi=3, period_streak=2, period_rank=100))]
    fn new(period_rsi: usize, period_streak: usize, period_rank: usize) -> PyResult<Self> {
        Ok(Self {
            inner: wc::ConnorsRsi::new(period_rsi, period_streak, period_rank).map_err(map_err)?,
        })
    }
    fn update(&mut self, value: f64) -> Option<f64> {
        self.inner.update(value)
    }
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        prices: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let s = prices
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        Ok(flatten(self.inner.batch(s)).into_pyarray(py))
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        let (r, s, k) = self.inner.periods();
        format!("ConnorsRSI(period_rsi={r}, period_streak={s}, period_rank={k})")
    }
}

// ============================== Laguerre RSI ==============================

#[pyclass(name = "LaguerreRSI", module = "wickra._wickra", skip_from_py_object)]
#[derive(Clone)]
struct PyLaguerreRsi {
    inner: wc::LaguerreRsi,
}

#[pymethods]
impl PyLaguerreRsi {
    #[new]
    #[pyo3(signature = (gamma=0.5))]
    fn new(gamma: f64) -> PyResult<Self> {
        Ok(Self {
            inner: wc::LaguerreRsi::new(gamma).map_err(map_err)?,
        })
    }
    fn update(&mut self, value: f64) -> Option<f64> {
        self.inner.update(value)
    }
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        prices: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let s = prices
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        Ok(flatten(self.inner.batch(s)).into_pyarray(py))
    }
    #[getter]
    fn gamma(&self) -> f64 {
        self.inner.gamma()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        format!("LaguerreRSI(gamma={})", self.inner.gamma())
    }
}

// ============================== SMI ==============================

#[pyclass(name = "SMI", module = "wickra._wickra", skip_from_py_object)]
#[derive(Clone)]
struct PySmi {
    inner: wc::Smi,
}

#[pymethods]
impl PySmi {
    #[new]
    #[pyo3(signature = (period=5, d_period=3, d2_period=3))]
    fn new(period: usize, d_period: usize, d2_period: usize) -> PyResult<Self> {
        Ok(Self {
            inner: wc::Smi::new(period, d_period, d2_period).map_err(map_err)?,
        })
    }
    fn update(&mut self, candle: &Bound<'_, PyAny>) -> PyResult<Option<f64>> {
        let c = extract_candle(candle)?;
        Ok(self.inner.update(c))
    }
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        high: PyReadonlyArray1<'py, f64>,
        low: PyReadonlyArray1<'py, f64>,
        close: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let h = high
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let l = low
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let c = close
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        if !(h.len() == l.len() && l.len() == c.len()) {
            return Err(PyValueError::new_err(
                "high, low and close must be equal length",
            ));
        }
        let mut out = Vec::with_capacity(c.len());
        for i in 0..c.len() {
            let candle = wc::Candle::new(c[i], h[i], l[i], c[i], 0.0, 0).map_err(map_err)?;
            out.push(self.inner.update(candle).unwrap_or(f64::NAN));
        }
        Ok(out.into_pyarray(py))
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        let (p, d, d2) = self.inner.periods();
        format!("SMI(period={p}, d_period={d}, d2_period={d2})")
    }
}

// ============================== KST ==============================

#[pyclass(name = "KST", module = "wickra._wickra", skip_from_py_object)]
#[derive(Clone)]
struct PyKst {
    inner: wc::Kst,
}

#[pymethods]
impl PyKst {
    #[new]
    #[pyo3(signature = (roc1=10, roc2=15, roc3=20, roc4=30, sma1=10, sma2=10, sma3=10, sma4=15, signal=9))]
    #[allow(clippy::too_many_arguments)]
    fn new(
        roc1: usize,
        roc2: usize,
        roc3: usize,
        roc4: usize,
        sma1: usize,
        sma2: usize,
        sma3: usize,
        sma4: usize,
        signal: usize,
    ) -> PyResult<Self> {
        Ok(Self {
            inner: wc::Kst::new(roc1, roc2, roc3, roc4, sma1, sma2, sma3, sma4, signal)
                .map_err(map_err)?,
        })
    }
    #[staticmethod]
    fn classic() -> Self {
        Self {
            inner: wc::Kst::classic(),
        }
    }
    fn update(&mut self, value: f64) -> Option<(f64, f64)> {
        self.inner.update(value).map(|o| (o.kst, o.signal))
    }
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        prices: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray2<f64>>> {
        let slice = prices
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let n = slice.len();
        let mut out = vec![f64::NAN; n * 2];
        for (i, p) in slice.iter().enumerate() {
            if let Some(o) = self.inner.update(*p) {
                out[i * 2] = o.kst;
                out[i * 2 + 1] = o.signal;
            }
        }
        Ok(numpy::ndarray::Array2::from_shape_vec((n, 2), out)
            .expect("shape consistent")
            .into_pyarray(py))
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        "KST".to_string()
    }
}

// ============================== PGO ==============================

#[pyclass(name = "PGO", module = "wickra._wickra", skip_from_py_object)]
#[derive(Clone)]
struct PyPgo {
    inner: wc::Pgo,
}

#[pymethods]
impl PyPgo {
    #[new]
    #[pyo3(signature = (period=14))]
    fn new(period: usize) -> PyResult<Self> {
        Ok(Self {
            inner: wc::Pgo::new(period).map_err(map_err)?,
        })
    }
    fn update(&mut self, candle: &Bound<'_, PyAny>) -> PyResult<Option<f64>> {
        let c = extract_candle(candle)?;
        Ok(self.inner.update(c))
    }
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        high: PyReadonlyArray1<'py, f64>,
        low: PyReadonlyArray1<'py, f64>,
        close: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let h = high
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let l = low
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let c = close
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        if !(h.len() == l.len() && l.len() == c.len()) {
            return Err(PyValueError::new_err(
                "high, low and close must be equal length",
            ));
        }
        let mut out = Vec::with_capacity(c.len());
        for i in 0..c.len() {
            let candle = wc::Candle::new(c[i], h[i], l[i], c[i], 0.0, 0).map_err(map_err)?;
            out.push(self.inner.update(candle).unwrap_or(f64::NAN));
        }
        Ok(out.into_pyarray(py))
    }
    #[getter]
    fn period(&self) -> usize {
        self.inner.period()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        format!("PGO(period={})", self.inner.period())
    }
}

// ============================== RVI ==============================

#[pyclass(name = "RVI", module = "wickra._wickra", skip_from_py_object)]
#[derive(Clone)]
struct PyRvi {
    inner: wc::Rvi,
}

#[pymethods]
impl PyRvi {
    #[new]
    #[pyo3(signature = (period=10))]
    fn new(period: usize) -> PyResult<Self> {
        Ok(Self {
            inner: wc::Rvi::new(period).map_err(map_err)?,
        })
    }
    fn update(&mut self, candle: &Bound<'_, PyAny>) -> PyResult<Option<f64>> {
        let c = extract_candle(candle)?;
        Ok(self.inner.update(c))
    }
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        open: PyReadonlyArray1<'py, f64>,
        high: PyReadonlyArray1<'py, f64>,
        low: PyReadonlyArray1<'py, f64>,
        close: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let o = open
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let h = high
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let l = low
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let c = close
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        if !(o.len() == h.len() && h.len() == l.len() && l.len() == c.len()) {
            return Err(PyValueError::new_err(
                "open, high, low and close must be equal length",
            ));
        }
        let mut out = Vec::with_capacity(c.len());
        for i in 0..c.len() {
            let candle = wc::Candle::new(o[i], h[i], l[i], c[i], 0.0, 0).map_err(map_err)?;
            out.push(self.inner.update(candle).unwrap_or(f64::NAN));
        }
        Ok(out.into_pyarray(py))
    }
    #[getter]
    fn period(&self) -> usize {
        self.inner.period()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        format!("RVI(period={})", self.inner.period())
    }
}

// ============================== FRAMA ==============================

#[pyclass(name = "FRAMA", module = "wickra._wickra", skip_from_py_object)]
#[derive(Clone)]
struct PyFrama {
    inner: wc::Frama,
}

#[pymethods]
impl PyFrama {
    #[new]
    #[pyo3(signature = (period=16))]
    fn new(period: usize) -> PyResult<Self> {
        Ok(Self {
            inner: wc::Frama::new(period).map_err(map_err)?,
        })
    }
    fn update(&mut self, value: f64) -> Option<f64> {
        self.inner.update(value)
    }
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        prices: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let s = prices
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        Ok(flatten(self.inner.batch(s)).into_pyarray(py))
    }
    #[getter]
    fn period(&self) -> usize {
        self.inner.period()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        format!("FRAMA(period={})", self.inner.period())
    }
}

// ============================== EVWMA ==============================

#[pyclass(name = "EVWMA", module = "wickra._wickra", skip_from_py_object)]
#[derive(Clone)]
struct PyEvwma {
    inner: wc::Evwma,
}

#[pymethods]
impl PyEvwma {
    #[new]
    #[pyo3(signature = (period=20))]
    fn new(period: usize) -> PyResult<Self> {
        Ok(Self {
            inner: wc::Evwma::new(period).map_err(map_err)?,
        })
    }
    fn update(&mut self, candle: &Bound<'_, PyAny>) -> PyResult<Option<f64>> {
        let c = extract_candle(candle)?;
        Ok(self.inner.update(c))
    }
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        close: PyReadonlyArray1<'py, f64>,
        volume: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let c = close
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let v = volume
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        if c.len() != v.len() {
            return Err(PyValueError::new_err(
                "close and volume must be equal length",
            ));
        }
        let mut out = Vec::with_capacity(c.len());
        for i in 0..c.len() {
            let candle = wc::Candle::new(c[i], c[i], c[i], c[i], v[i], 0).map_err(map_err)?;
            out.push(self.inner.update(candle).unwrap_or(f64::NAN));
        }
        Ok(out.into_pyarray(py))
    }
    #[getter]
    fn period(&self) -> usize {
        self.inner.period()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        format!("EVWMA(period={})", self.inner.period())
    }
}

// ============================== Alligator ==============================

#[pyclass(name = "Alligator", module = "wickra._wickra", skip_from_py_object)]
#[derive(Clone)]
struct PyAlligator {
    inner: wc::Alligator,
}

#[pymethods]
impl PyAlligator {
    #[new]
    #[pyo3(signature = (jaw=13, teeth=8, lips=5))]
    fn new(jaw: usize, teeth: usize, lips: usize) -> PyResult<Self> {
        Ok(Self {
            inner: wc::Alligator::new(jaw, teeth, lips).map_err(map_err)?,
        })
    }
    fn update(&mut self, candle: &Bound<'_, PyAny>) -> PyResult<Option<(f64, f64, f64)>> {
        let c = extract_candle(candle)?;
        Ok(self.inner.update(c).map(|o| (o.jaw, o.teeth, o.lips)))
    }
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        high: PyReadonlyArray1<'py, f64>,
        low: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray2<f64>>> {
        let h = high
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let l = low
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        if h.len() != l.len() {
            return Err(PyValueError::new_err("high and low must be equal length"));
        }
        let n = h.len();
        let mut out = vec![f64::NAN; n * 3];
        for i in 0..n {
            let candle = wc::Candle::new(l[i], h[i], l[i], l[i], 0.0, 0).map_err(map_err)?;
            if let Some(o) = self.inner.update(candle) {
                out[i * 3] = o.jaw;
                out[i * 3 + 1] = o.teeth;
                out[i * 3 + 2] = o.lips;
            }
        }
        Ok(numpy::ndarray::Array2::from_shape_vec((n, 3), out)
            .expect("shape consistent")
            .into_pyarray(py))
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        let (j, t, l) = self.inner.periods();
        format!("Alligator(jaw={j}, teeth={t}, lips={l})")
    }
}

// ============================== JMA ==============================

#[pyclass(name = "JMA", module = "wickra._wickra", skip_from_py_object)]
#[derive(Clone)]
struct PyJma {
    inner: wc::Jma,
}

#[pymethods]
impl PyJma {
    #[new]
    #[pyo3(signature = (period=14, phase=0.0, power=2))]
    fn new(period: usize, phase: f64, power: u32) -> PyResult<Self> {
        Ok(Self {
            inner: wc::Jma::new(period, phase, power).map_err(map_err)?,
        })
    }
    fn update(&mut self, value: f64) -> Option<f64> {
        self.inner.update(value)
    }
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        prices: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let s = prices
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        Ok(flatten(self.inner.batch(s)).into_pyarray(py))
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        let (p, ph, pw) = self.inner.params();
        format!("JMA(period={p}, phase={ph}, power={pw})")
    }
}

// ============================== VIDYA ==============================

#[pyclass(name = "VIDYA", module = "wickra._wickra", skip_from_py_object)]
#[derive(Clone)]
struct PyVidya {
    inner: wc::Vidya,
}

#[pymethods]
impl PyVidya {
    #[new]
    #[pyo3(signature = (period=14, cmo_period=9))]
    fn new(period: usize, cmo_period: usize) -> PyResult<Self> {
        Ok(Self {
            inner: wc::Vidya::new(period, cmo_period).map_err(map_err)?,
        })
    }
    fn update(&mut self, value: f64) -> Option<f64> {
        self.inner.update(value)
    }
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        prices: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let s = prices
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        Ok(flatten(self.inner.batch(s)).into_pyarray(py))
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        let (p, c) = self.inner.periods();
        format!("VIDYA(period={p}, cmo_period={c})")
    }
}

// ============================== McGinley Dynamic ==============================

#[pyclass(
    name = "McGinleyDynamic",
    module = "wickra._wickra",
    skip_from_py_object
)]
#[derive(Clone)]
struct PyMcGinleyDynamic {
    inner: wc::McGinleyDynamic,
}

#[pymethods]
impl PyMcGinleyDynamic {
    #[new]
    #[pyo3(signature = (period=10))]
    fn new(period: usize) -> PyResult<Self> {
        Ok(Self {
            inner: wc::McGinleyDynamic::new(period).map_err(map_err)?,
        })
    }
    fn update(&mut self, value: f64) -> Option<f64> {
        self.inner.update(value)
    }
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        prices: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let s = prices
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        Ok(flatten(self.inner.batch(s)).into_pyarray(py))
    }
    #[getter]
    fn period(&self) -> usize {
        self.inner.period()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        format!("McGinleyDynamic(period={})", self.inner.period())
    }
}

// ============================== ALMA ==============================

#[pyclass(name = "ALMA", module = "wickra._wickra", skip_from_py_object)]
#[derive(Clone)]
struct PyAlma {
    inner: wc::Alma,
}

#[pymethods]
impl PyAlma {
    #[new]
    #[pyo3(signature = (period=9, offset=0.85, sigma=6.0))]
    fn new(period: usize, offset: f64, sigma: f64) -> PyResult<Self> {
        Ok(Self {
            inner: wc::Alma::new(period, offset, sigma).map_err(map_err)?,
        })
    }
    fn update(&mut self, value: f64) -> Option<f64> {
        self.inner.update(value)
    }
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        prices: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let s = prices
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        Ok(flatten(self.inner.batch(s)).into_pyarray(py))
    }
    #[getter]
    fn period(&self) -> usize {
        self.inner.period()
    }
    #[getter]
    fn offset(&self) -> f64 {
        self.inner.offset()
    }
    #[getter]
    fn sigma(&self) -> f64 {
        self.inner.sigma()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        format!(
            "ALMA(period={}, offset={}, sigma={})",
            self.inner.period(),
            self.inner.offset(),
            self.inner.sigma()
        )
    }
}

// ============================== AwesomeOscillatorHistogram ==============================

#[pyclass(
    name = "AwesomeOscillatorHistogram",
    module = "wickra._wickra",
    skip_from_py_object
)]
#[derive(Clone)]
struct PyAoHist {
    inner: wc::AwesomeOscillatorHistogram,
}

#[pymethods]
impl PyAoHist {
    #[new]
    #[pyo3(signature = (fast=5, slow=34, sma_period=5))]
    fn new(fast: usize, slow: usize, sma_period: usize) -> PyResult<Self> {
        Ok(Self {
            inner: wc::AwesomeOscillatorHistogram::new(fast, slow, sma_period).map_err(map_err)?,
        })
    }
    fn update(&mut self, candle: &Bound<'_, PyAny>) -> PyResult<Option<f64>> {
        let c = extract_candle(candle)?;
        Ok(self.inner.update(c))
    }
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        high: PyReadonlyArray1<'py, f64>,
        low: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let h = high
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let l = low
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        if h.len() != l.len() {
            return Err(PyValueError::new_err("high and low must be equal length"));
        }
        let mut out = Vec::with_capacity(h.len());
        for i in 0..h.len() {
            let candle = wc::Candle::new(l[i], h[i], l[i], l[i], 0.0, 0).map_err(map_err)?;
            out.push(self.inner.update(candle).unwrap_or(f64::NAN));
        }
        Ok(out.into_pyarray(py))
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        let (f, s, k) = self.inner.periods();
        format!("AwesomeOscillatorHistogram(fast={f}, slow={s}, sma_period={k})")
    }
}

// ============================== STC ==============================

#[pyclass(name = "STC", module = "wickra._wickra", skip_from_py_object)]
#[derive(Clone)]
struct PyStc {
    inner: wc::Stc,
}

#[pymethods]
impl PyStc {
    #[new]
    #[pyo3(signature = (fast=23, slow=50, schaff_period=10, factor=0.5))]
    fn new(fast: usize, slow: usize, schaff_period: usize, factor: f64) -> PyResult<Self> {
        Ok(Self {
            inner: wc::Stc::new(fast, slow, schaff_period, factor).map_err(map_err)?,
        })
    }
    fn update(&mut self, value: f64) -> Option<f64> {
        self.inner.update(value)
    }
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        prices: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let s = prices
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        Ok(flatten(self.inner.batch(s)).into_pyarray(py))
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        let (f, s, p, k) = self.inner.params();
        format!("STC(fast={f}, slow={s}, schaff_period={p}, factor={k})")
    }
}

// ============================== ElderImpulse ==============================

#[pyclass(name = "ElderImpulse", module = "wickra._wickra", skip_from_py_object)]
#[derive(Clone)]
struct PyElderImpulse {
    inner: wc::ElderImpulse,
}

#[pymethods]
impl PyElderImpulse {
    #[new]
    #[pyo3(signature = (ema_period=13, macd_fast=12, macd_slow=26, macd_signal=9))]
    fn new(
        ema_period: usize,
        macd_fast: usize,
        macd_slow: usize,
        macd_signal: usize,
    ) -> PyResult<Self> {
        Ok(Self {
            inner: wc::ElderImpulse::new(ema_period, macd_fast, macd_slow, macd_signal)
                .map_err(map_err)?,
        })
    }
    fn update(&mut self, value: f64) -> Option<f64> {
        self.inner.update(value)
    }
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        prices: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let s = prices
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        Ok(flatten(self.inner.batch(s)).into_pyarray(py))
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        let (e, f, s, sig) = self.inner.periods();
        format!("ElderImpulse(ema_period={e}, macd_fast={f}, macd_slow={s}, macd_signal={sig})")
    }
}

// ============================== ZeroLagMACD ==============================

#[pyclass(name = "ZeroLagMACD", module = "wickra._wickra", skip_from_py_object)]
#[derive(Clone)]
struct PyZeroLagMacd {
    inner: wc::ZeroLagMacd,
}

#[pymethods]
impl PyZeroLagMacd {
    #[new]
    #[pyo3(signature = (fast=12, slow=26, signal=9))]
    fn new(fast: usize, slow: usize, signal: usize) -> PyResult<Self> {
        Ok(Self {
            inner: wc::ZeroLagMacd::new(fast, slow, signal).map_err(map_err)?,
        })
    }
    fn update(&mut self, value: f64) -> Option<(f64, f64, f64)> {
        self.inner
            .update(value)
            .map(|o| (o.macd, o.signal, o.histogram))
    }
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        prices: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray2<f64>>> {
        let slice = prices
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let n = slice.len();
        let mut out = vec![f64::NAN; n * 3];
        for (i, p) in slice.iter().enumerate() {
            if let Some(o) = self.inner.update(*p) {
                out[i * 3] = o.macd;
                out[i * 3 + 1] = o.signal;
                out[i * 3 + 2] = o.histogram;
            }
        }
        Ok(numpy::ndarray::Array2::from_shape_vec((n, 3), out)
            .expect("shape consistent")
            .into_pyarray(py))
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        let (f, s, sig) = self.inner.periods();
        format!("ZeroLagMACD(fast={f}, slow={s}, signal={sig})")
    }
}

// ============================== CFO ==============================

#[pyclass(name = "CFO", module = "wickra._wickra", skip_from_py_object)]
#[derive(Clone)]
struct PyCfo {
    inner: wc::Cfo,
}

#[pymethods]
impl PyCfo {
    #[new]
    #[pyo3(signature = (period=14))]
    fn new(period: usize) -> PyResult<Self> {
        Ok(Self {
            inner: wc::Cfo::new(period).map_err(map_err)?,
        })
    }
    fn update(&mut self, value: f64) -> Option<f64> {
        self.inner.update(value)
    }
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        prices: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let s = prices
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        Ok(flatten(self.inner.batch(s)).into_pyarray(py))
    }
    #[getter]
    fn period(&self) -> usize {
        self.inner.period()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        format!("CFO(period={})", self.inner.period())
    }
}

// ============================== APO ==============================

#[pyclass(name = "APO", module = "wickra._wickra", skip_from_py_object)]
#[derive(Clone)]
struct PyApo {
    inner: wc::Apo,
}

#[pymethods]
impl PyApo {
    #[new]
    #[pyo3(signature = (fast=12, slow=26))]
    fn new(fast: usize, slow: usize) -> PyResult<Self> {
        Ok(Self {
            inner: wc::Apo::new(fast, slow).map_err(map_err)?,
        })
    }
    fn update(&mut self, value: f64) -> Option<f64> {
        self.inner.update(value)
    }
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        prices: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let s = prices
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        Ok(flatten(self.inner.batch(s)).into_pyarray(py))
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        let (f, s) = self.inner.periods();
        format!("APO(fast={f}, slow={s})")
    }
}

// ============================== CCI ==============================

#[pyclass(name = "CCI", module = "wickra._wickra", skip_from_py_object)]
#[derive(Clone)]
struct PyCci {
    inner: wc::Cci,
}

#[pymethods]
impl PyCci {
    #[new]
    #[pyo3(signature = (period=20))]
    fn new(period: usize) -> PyResult<Self> {
        Ok(Self {
            inner: wc::Cci::new(period).map_err(map_err)?,
        })
    }
    fn update(&mut self, candle: &Bound<'_, PyAny>) -> PyResult<Option<f64>> {
        let c = extract_candle(candle)?;
        Ok(self.inner.update(c))
    }
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        high: PyReadonlyArray1<'py, f64>,
        low: PyReadonlyArray1<'py, f64>,
        close: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let h = high
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let l = low
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let c = close
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        if h.len() != l.len() || l.len() != c.len() {
            return Err(PyValueError::new_err(
                "high, low, close must be equal length",
            ));
        }
        let mut out = Vec::with_capacity(h.len());
        for i in 0..h.len() {
            let candle = wc::Candle::new(c[i], h[i], l[i], c[i], 0.0, 0).map_err(map_err)?;
            out.push(self.inner.update(candle).unwrap_or(f64::NAN));
        }
        Ok(out.into_pyarray(py))
    }
    #[getter]
    fn period(&self) -> usize {
        self.inner.period()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        format!("CCI(period={})", self.inner.period())
    }
}

// ============================== ROC ==============================

#[pyclass(name = "ROC", module = "wickra._wickra", skip_from_py_object)]
#[derive(Clone)]
struct PyRoc {
    inner: wc::Roc,
}

#[pymethods]
impl PyRoc {
    #[new]
    #[pyo3(signature = (period=10))]
    fn new(period: usize) -> PyResult<Self> {
        Ok(Self {
            inner: wc::Roc::new(period).map_err(map_err)?,
        })
    }
    fn update(&mut self, value: f64) -> Option<f64> {
        self.inner.update(value)
    }
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        prices: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let s = prices
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        Ok(flatten(self.inner.batch(s)).into_pyarray(py))
    }
    #[getter]
    fn period(&self) -> usize {
        self.inner.period()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        format!("ROC(period={})", self.inner.period())
    }
}

// ============================== Williams %R ==============================

#[pyclass(name = "WilliamsR", module = "wickra._wickra", skip_from_py_object)]
#[derive(Clone)]
struct PyWilliamsR {
    inner: wc::WilliamsR,
}

#[pymethods]
impl PyWilliamsR {
    #[new]
    #[pyo3(signature = (period=14))]
    fn new(period: usize) -> PyResult<Self> {
        Ok(Self {
            inner: wc::WilliamsR::new(period).map_err(map_err)?,
        })
    }
    fn update(&mut self, candle: &Bound<'_, PyAny>) -> PyResult<Option<f64>> {
        let c = extract_candle(candle)?;
        Ok(self.inner.update(c))
    }
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        high: PyReadonlyArray1<'py, f64>,
        low: PyReadonlyArray1<'py, f64>,
        close: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let h = high
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let l = low
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let c = close
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        if h.len() != l.len() || l.len() != c.len() {
            return Err(PyValueError::new_err(
                "high, low, close must be equal length",
            ));
        }
        let mut out = Vec::with_capacity(h.len());
        for i in 0..h.len() {
            let candle = wc::Candle::new(c[i], h[i], l[i], c[i], 0.0, 0).map_err(map_err)?;
            out.push(self.inner.update(candle).unwrap_or(f64::NAN));
        }
        Ok(out.into_pyarray(py))
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
}

// ============================== ADX ==============================

#[pyclass(name = "ADX", module = "wickra._wickra", skip_from_py_object)]
#[derive(Clone)]
struct PyAdx {
    inner: wc::Adx,
}

#[pymethods]
impl PyAdx {
    #[new]
    #[pyo3(signature = (period=14))]
    fn new(period: usize) -> PyResult<Self> {
        Ok(Self {
            inner: wc::Adx::new(period).map_err(map_err)?,
        })
    }
    /// Returns `(plus_di, minus_di, adx)` or None during warmup.
    fn update(&mut self, candle: &Bound<'_, PyAny>) -> PyResult<Option<(f64, f64, f64)>> {
        let c = extract_candle(candle)?;
        Ok(self.inner.update(c).map(|o| (o.plus_di, o.minus_di, o.adx)))
    }
    /// Batch returns shape `(n, 3)`: `[plus_di, minus_di, adx]`.
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        high: PyReadonlyArray1<'py, f64>,
        low: PyReadonlyArray1<'py, f64>,
        close: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray2<f64>>> {
        let h = high
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let l = low
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let c = close
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        if h.len() != l.len() || l.len() != c.len() {
            return Err(PyValueError::new_err(
                "high, low, close must be equal length",
            ));
        }
        let n = h.len();
        let mut out = vec![f64::NAN; n * 3];
        for i in 0..n {
            let candle = wc::Candle::new(c[i], h[i], l[i], c[i], 0.0, 0).map_err(map_err)?;
            if let Some(o) = self.inner.update(candle) {
                out[i * 3] = o.plus_di;
                out[i * 3 + 1] = o.minus_di;
                out[i * 3 + 2] = o.adx;
            }
        }
        Ok(numpy::ndarray::Array2::from_shape_vec((n, 3), out)
            .expect("shape consistent")
            .into_pyarray(py))
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
}

// ============================== ADXR ==============================

#[pyclass(name = "ADXR", module = "wickra._wickra", skip_from_py_object)]
#[derive(Clone)]
struct PyAdxr {
    inner: wc::Adxr,
}

#[pymethods]
impl PyAdxr {
    #[new]
    #[pyo3(signature = (period=14))]
    fn new(period: usize) -> PyResult<Self> {
        Ok(Self {
            inner: wc::Adxr::new(period).map_err(map_err)?,
        })
    }
    fn update(&mut self, candle: &Bound<'_, PyAny>) -> PyResult<Option<f64>> {
        let c = extract_candle(candle)?;
        Ok(self.inner.update(c))
    }
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        high: PyReadonlyArray1<'py, f64>,
        low: PyReadonlyArray1<'py, f64>,
        close: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let h = high
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let l = low
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let c = close
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        if h.len() != l.len() || l.len() != c.len() {
            return Err(PyValueError::new_err(
                "high, low, close must be equal length",
            ));
        }
        let n = h.len();
        let mut out = vec![f64::NAN; n];
        for i in 0..n {
            let candle = wc::Candle::new(c[i], h[i], l[i], c[i], 0.0, 0).map_err(map_err)?;
            if let Some(v) = self.inner.update(candle) {
                out[i] = v;
            }
        }
        Ok(out.into_pyarray(py))
    }
    #[getter]
    fn period(&self) -> usize {
        self.inner.period()
    }
    #[getter]
    fn value(&self) -> Option<f64> {
        self.inner.value()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        format!("ADXR(period={})", self.inner.period())
    }
}

// ============================== MFI ==============================

#[pyclass(name = "MFI", module = "wickra._wickra", skip_from_py_object)]
#[derive(Clone)]
struct PyMfi {
    inner: wc::Mfi,
}

#[pymethods]
impl PyMfi {
    #[new]
    #[pyo3(signature = (period=14))]
    fn new(period: usize) -> PyResult<Self> {
        Ok(Self {
            inner: wc::Mfi::new(period).map_err(map_err)?,
        })
    }
    fn update(&mut self, candle: &Bound<'_, PyAny>) -> PyResult<Option<f64>> {
        let c = extract_candle(candle)?;
        Ok(self.inner.update(c))
    }
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        high: PyReadonlyArray1<'py, f64>,
        low: PyReadonlyArray1<'py, f64>,
        close: PyReadonlyArray1<'py, f64>,
        volume: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let h = high
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let l = low
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let c = close
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let v = volume
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        if h.len() != l.len() || l.len() != c.len() || c.len() != v.len() {
            return Err(PyValueError::new_err(
                "high, low, close, volume must be equal length",
            ));
        }
        let mut out = Vec::with_capacity(h.len());
        for i in 0..h.len() {
            let candle = wc::Candle::new(c[i], h[i], l[i], c[i], v[i], 0).map_err(map_err)?;
            out.push(self.inner.update(candle).unwrap_or(f64::NAN));
        }
        Ok(out.into_pyarray(py))
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
}

// ============================== TRIX ==============================

#[pyclass(name = "TRIX", module = "wickra._wickra", skip_from_py_object)]
#[derive(Clone)]
struct PyTrix {
    inner: wc::Trix,
}

#[pymethods]
impl PyTrix {
    #[new]
    #[pyo3(signature = (period=30))]
    fn new(period: usize) -> PyResult<Self> {
        Ok(Self {
            inner: wc::Trix::new(period).map_err(map_err)?,
        })
    }
    fn update(&mut self, value: f64) -> Option<f64> {
        self.inner.update(value)
    }
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        prices: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let s = prices
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        Ok(flatten(self.inner.batch(s)).into_pyarray(py))
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
}

// ============================== PSAR ==============================

#[pyclass(name = "PSAR", module = "wickra._wickra", skip_from_py_object)]
#[derive(Clone)]
struct PyPsar {
    inner: wc::Psar,
}

#[pymethods]
impl PyPsar {
    #[new]
    #[pyo3(signature = (af_start=0.02, af_step=0.02, af_max=0.20))]
    fn new(af_start: f64, af_step: f64, af_max: f64) -> PyResult<Self> {
        Ok(Self {
            inner: wc::Psar::new(af_start, af_step, af_max).map_err(map_err)?,
        })
    }
    fn update(&mut self, candle: &Bound<'_, PyAny>) -> PyResult<Option<f64>> {
        let c = extract_candle(candle)?;
        Ok(self.inner.update(c))
    }
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        high: PyReadonlyArray1<'py, f64>,
        low: PyReadonlyArray1<'py, f64>,
        close: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let h = high
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let l = low
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let c = close
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        if h.len() != l.len() || l.len() != c.len() {
            return Err(PyValueError::new_err(
                "high, low, close must be equal length",
            ));
        }
        let mut out = Vec::with_capacity(h.len());
        for i in 0..h.len() {
            let candle = wc::Candle::new(c[i], h[i], l[i], c[i], 0.0, 0).map_err(map_err)?;
            out.push(self.inner.update(candle).unwrap_or(f64::NAN));
        }
        Ok(out.into_pyarray(py))
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
}

// ============================== Keltner Channels ==============================

#[pyclass(name = "Keltner", module = "wickra._wickra", skip_from_py_object)]
#[derive(Clone)]
struct PyKeltner {
    inner: wc::Keltner,
}

#[pymethods]
impl PyKeltner {
    #[new]
    #[pyo3(signature = (ema_period=20, atr_period=10, multiplier=2.0))]
    fn new(ema_period: usize, atr_period: usize, multiplier: f64) -> PyResult<Self> {
        Ok(Self {
            inner: wc::Keltner::new(ema_period, atr_period, multiplier).map_err(map_err)?,
        })
    }
    /// Returns `(upper, middle, lower)` or None during warmup.
    fn update(&mut self, candle: &Bound<'_, PyAny>) -> PyResult<Option<(f64, f64, f64)>> {
        let c = extract_candle(candle)?;
        Ok(self.inner.update(c).map(|o| (o.upper, o.middle, o.lower)))
    }
    /// Returns shape `(n, 3)` for `[upper, middle, lower]`.
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        high: PyReadonlyArray1<'py, f64>,
        low: PyReadonlyArray1<'py, f64>,
        close: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray2<f64>>> {
        let h = high
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let l = low
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let c = close
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        if h.len() != l.len() || l.len() != c.len() {
            return Err(PyValueError::new_err(
                "high, low, close must be equal length",
            ));
        }
        let n = h.len();
        let mut out = vec![f64::NAN; n * 3];
        for i in 0..n {
            let candle = wc::Candle::new(c[i], h[i], l[i], c[i], 0.0, 0).map_err(map_err)?;
            if let Some(o) = self.inner.update(candle) {
                out[i * 3] = o.upper;
                out[i * 3 + 1] = o.middle;
                out[i * 3 + 2] = o.lower;
            }
        }
        Ok(numpy::ndarray::Array2::from_shape_vec((n, 3), out)
            .expect("shape consistent")
            .into_pyarray(py))
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
}

// ============================== Donchian Channels ==============================

#[pyclass(name = "Donchian", module = "wickra._wickra", skip_from_py_object)]
#[derive(Clone)]
struct PyDonchian {
    inner: wc::Donchian,
}

#[pymethods]
impl PyDonchian {
    #[new]
    #[pyo3(signature = (period=20))]
    fn new(period: usize) -> PyResult<Self> {
        Ok(Self {
            inner: wc::Donchian::new(period).map_err(map_err)?,
        })
    }
    fn update(&mut self, candle: &Bound<'_, PyAny>) -> PyResult<Option<(f64, f64, f64)>> {
        let c = extract_candle(candle)?;
        Ok(self.inner.update(c).map(|o| (o.upper, o.middle, o.lower)))
    }
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        high: PyReadonlyArray1<'py, f64>,
        low: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray2<f64>>> {
        let h = high
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let l = low
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        if h.len() != l.len() {
            return Err(PyValueError::new_err("high and low must be equal length"));
        }
        let n = h.len();
        let mut out = vec![f64::NAN; n * 3];
        for i in 0..n {
            let candle = wc::Candle::new(l[i], h[i], l[i], l[i], 0.0, 0).map_err(map_err)?;
            if let Some(o) = self.inner.update(candle) {
                out[i * 3] = o.upper;
                out[i * 3 + 1] = o.middle;
                out[i * 3 + 2] = o.lower;
            }
        }
        Ok(numpy::ndarray::Array2::from_shape_vec((n, 3), out)
            .expect("shape consistent")
            .into_pyarray(py))
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
}

// ============================== VWAP ==============================

#[pyclass(name = "VWAP", module = "wickra._wickra", skip_from_py_object)]
#[derive(Clone)]
struct PyVwap {
    inner: wc::Vwap,
}

#[pymethods]
impl PyVwap {
    #[new]
    fn new() -> Self {
        Self {
            inner: wc::Vwap::new(),
        }
    }
    fn update(&mut self, candle: &Bound<'_, PyAny>) -> PyResult<Option<f64>> {
        let c = extract_candle(candle)?;
        Ok(self.inner.update(c))
    }
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        high: PyReadonlyArray1<'py, f64>,
        low: PyReadonlyArray1<'py, f64>,
        close: PyReadonlyArray1<'py, f64>,
        volume: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let h = high
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let l = low
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let c = close
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let v = volume
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        if h.len() != l.len() || l.len() != c.len() || c.len() != v.len() {
            return Err(PyValueError::new_err(
                "high, low, close, volume must be equal length",
            ));
        }
        let mut out = Vec::with_capacity(h.len());
        for i in 0..h.len() {
            let candle = wc::Candle::new(c[i], h[i], l[i], c[i], v[i], 0).map_err(map_err)?;
            out.push(self.inner.update(candle).unwrap_or(f64::NAN));
        }
        Ok(out.into_pyarray(py))
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
}

// ============================== Rolling VWAP ==============================

#[pyclass(name = "RollingVWAP", module = "wickra._wickra", skip_from_py_object)]
#[derive(Clone)]
struct PyRollingVwap {
    inner: wc::RollingVwap,
}

#[pymethods]
impl PyRollingVwap {
    #[new]
    fn new(period: usize) -> PyResult<Self> {
        Ok(Self {
            inner: wc::RollingVwap::new(period).map_err(map_err)?,
        })
    }
    fn update(&mut self, candle: &Bound<'_, PyAny>) -> PyResult<Option<f64>> {
        let c = extract_candle(candle)?;
        Ok(self.inner.update(c))
    }
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        high: PyReadonlyArray1<'py, f64>,
        low: PyReadonlyArray1<'py, f64>,
        close: PyReadonlyArray1<'py, f64>,
        volume: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let h = high
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let l = low
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let c = close
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let v = volume
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        if h.len() != l.len() || l.len() != c.len() || c.len() != v.len() {
            return Err(PyValueError::new_err(
                "high, low, close, volume must be equal length",
            ));
        }
        let mut out = Vec::with_capacity(h.len());
        for i in 0..h.len() {
            let candle = wc::Candle::new(c[i], h[i], l[i], c[i], v[i], 0).map_err(map_err)?;
            out.push(self.inner.update(candle).unwrap_or(f64::NAN));
        }
        Ok(out.into_pyarray(py))
    }
    #[getter]
    fn period(&self) -> usize {
        self.inner.period()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        format!("RollingVWAP(period={})", self.inner.period())
    }
}

// ============================== Awesome Oscillator ==============================

#[pyclass(
    name = "AwesomeOscillator",
    module = "wickra._wickra",
    skip_from_py_object
)]
#[derive(Clone)]
struct PyAo {
    inner: wc::AwesomeOscillator,
}

#[pymethods]
impl PyAo {
    #[new]
    #[pyo3(signature = (fast=5, slow=34))]
    fn new(fast: usize, slow: usize) -> PyResult<Self> {
        Ok(Self {
            inner: wc::AwesomeOscillator::new(fast, slow).map_err(map_err)?,
        })
    }
    fn update(&mut self, candle: &Bound<'_, PyAny>) -> PyResult<Option<f64>> {
        let c = extract_candle(candle)?;
        Ok(self.inner.update(c))
    }
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        high: PyReadonlyArray1<'py, f64>,
        low: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let h = high
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let l = low
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        if h.len() != l.len() {
            return Err(PyValueError::new_err("high and low must be equal length"));
        }
        let mut out = Vec::with_capacity(h.len());
        for i in 0..h.len() {
            let candle = wc::Candle::new(l[i], h[i], l[i], l[i], 0.0, 0).map_err(map_err)?;
            out.push(self.inner.update(candle).unwrap_or(f64::NAN));
        }
        Ok(out.into_pyarray(py))
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
}

// ============================== Aroon ==============================

#[pyclass(name = "Aroon", module = "wickra._wickra", skip_from_py_object)]
#[derive(Clone)]
struct PyAroon {
    inner: wc::Aroon,
}

#[pymethods]
impl PyAroon {
    #[new]
    #[pyo3(signature = (period=14))]
    fn new(period: usize) -> PyResult<Self> {
        Ok(Self {
            inner: wc::Aroon::new(period).map_err(map_err)?,
        })
    }
    fn update(&mut self, candle: &Bound<'_, PyAny>) -> PyResult<Option<(f64, f64)>> {
        let c = extract_candle(candle)?;
        Ok(self.inner.update(c).map(|o| (o.up, o.down)))
    }
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        high: PyReadonlyArray1<'py, f64>,
        low: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray2<f64>>> {
        let h = high
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let l = low
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        if h.len() != l.len() {
            return Err(PyValueError::new_err("high and low must be equal length"));
        }
        let n = h.len();
        let mut out = vec![f64::NAN; n * 2];
        for i in 0..n {
            let candle = wc::Candle::new(l[i], h[i], l[i], l[i], 0.0, 0).map_err(map_err)?;
            if let Some(o) = self.inner.update(candle) {
                out[i * 2] = o.up;
                out[i * 2 + 1] = o.down;
            }
        }
        Ok(numpy::ndarray::Array2::from_shape_vec((n, 2), out)
            .expect("shape consistent")
            .into_pyarray(py))
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
}

// ============================== ADL ==============================

#[pyclass(name = "ADL", module = "wickra._wickra", skip_from_py_object)]
#[derive(Clone)]
struct PyAdl {
    inner: wc::Adl,
}

#[pymethods]
impl PyAdl {
    #[new]
    fn new() -> Self {
        Self {
            inner: wc::Adl::new(),
        }
    }
    fn update(&mut self, candle: &Bound<'_, PyAny>) -> PyResult<Option<f64>> {
        let c = extract_candle(candle)?;
        Ok(self.inner.update(c))
    }
    /// Batch over numpy columns: high, low, close, volume (all equal length).
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        high: PyReadonlyArray1<'py, f64>,
        low: PyReadonlyArray1<'py, f64>,
        close: PyReadonlyArray1<'py, f64>,
        volume: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let h = high
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let l = low
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let c = close
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let v = volume
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        if h.len() != l.len() || l.len() != c.len() || c.len() != v.len() {
            return Err(PyValueError::new_err(
                "high, low, close, volume must be equal length",
            ));
        }
        let mut out = Vec::with_capacity(h.len());
        for i in 0..h.len() {
            let candle = wc::Candle::new(c[i], h[i], l[i], c[i], v[i], 0).map_err(map_err)?;
            out.push(self.inner.update(candle).unwrap_or(f64::NAN));
        }
        Ok(out.into_pyarray(py))
    }
    #[getter]
    fn value(&self) -> Option<f64> {
        self.inner.value()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        "ADL()".to_string()
    }
}

// ============================== Volume-Price Trend ==============================

#[pyclass(
    name = "VolumePriceTrend",
    module = "wickra._wickra",
    skip_from_py_object
)]
#[derive(Clone)]
struct PyVolumePriceTrend {
    inner: wc::VolumePriceTrend,
}

#[pymethods]
impl PyVolumePriceTrend {
    #[new]
    fn new() -> Self {
        Self {
            inner: wc::VolumePriceTrend::new(),
        }
    }
    fn update(&mut self, candle: &Bound<'_, PyAny>) -> PyResult<Option<f64>> {
        let c = extract_candle(candle)?;
        Ok(self.inner.update(c))
    }
    /// Batch over numpy close + volume arrays (both 1-D, equal length).
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        close: PyReadonlyArray1<'py, f64>,
        volume: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let c = close
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let v = volume
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        if c.len() != v.len() {
            return Err(PyValueError::new_err(
                "close and volume must be equal length",
            ));
        }
        let mut out = Vec::with_capacity(c.len());
        for i in 0..c.len() {
            let candle = wc::Candle::new(c[i], c[i], c[i], c[i], v[i], 0).map_err(map_err)?;
            out.push(self.inner.update(candle).unwrap_or(f64::NAN));
        }
        Ok(out.into_pyarray(py))
    }
    #[getter]
    fn value(&self) -> Option<f64> {
        self.inner.value()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        "VolumePriceTrend()".to_string()
    }
}

// ============================== Bollinger Bandwidth ==============================

#[pyclass(
    name = "BollingerBandwidth",
    module = "wickra._wickra",
    skip_from_py_object
)]
#[derive(Clone)]
struct PyBollingerBandwidth {
    inner: wc::BollingerBandwidth,
}

#[pymethods]
impl PyBollingerBandwidth {
    #[new]
    #[pyo3(signature = (period=20, multiplier=2.0))]
    fn new(period: usize, multiplier: f64) -> PyResult<Self> {
        Ok(Self {
            inner: wc::BollingerBandwidth::new(period, multiplier).map_err(map_err)?,
        })
    }
    fn update(&mut self, value: f64) -> Option<f64> {
        self.inner.update(value)
    }
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        prices: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let slice = prices
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        Ok(flatten(self.inner.batch(slice)).into_pyarray(py))
    }
    #[getter]
    fn period(&self) -> usize {
        self.inner.period()
    }
    #[getter]
    fn multiplier(&self) -> f64 {
        self.inner.multiplier()
    }
    #[getter]
    fn value(&self) -> Option<f64> {
        self.inner.value()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        format!(
            "BollingerBandwidth(period={}, multiplier={})",
            self.inner.period(),
            self.inner.multiplier()
        )
    }
}

// ============================== Percent B ==============================

#[pyclass(name = "PercentB", module = "wickra._wickra", skip_from_py_object)]
#[derive(Clone)]
struct PyPercentB {
    inner: wc::PercentB,
}

#[pymethods]
impl PyPercentB {
    #[new]
    #[pyo3(signature = (period=20, multiplier=2.0))]
    fn new(period: usize, multiplier: f64) -> PyResult<Self> {
        Ok(Self {
            inner: wc::PercentB::new(period, multiplier).map_err(map_err)?,
        })
    }
    fn update(&mut self, value: f64) -> Option<f64> {
        self.inner.update(value)
    }
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        prices: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let slice = prices
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        Ok(flatten(self.inner.batch(slice)).into_pyarray(py))
    }
    #[getter]
    fn period(&self) -> usize {
        self.inner.period()
    }
    #[getter]
    fn multiplier(&self) -> f64 {
        self.inner.multiplier()
    }
    #[getter]
    fn value(&self) -> Option<f64> {
        self.inner.value()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        format!(
            "PercentB(period={}, multiplier={})",
            self.inner.period(),
            self.inner.multiplier()
        )
    }
}

// ============================== NATR ==============================

#[pyclass(name = "NATR", module = "wickra._wickra", skip_from_py_object)]
#[derive(Clone)]
struct PyNatr {
    inner: wc::Natr,
}

#[pymethods]
impl PyNatr {
    #[new]
    #[pyo3(signature = (period=14))]
    fn new(period: usize) -> PyResult<Self> {
        Ok(Self {
            inner: wc::Natr::new(period).map_err(map_err)?,
        })
    }
    fn update(&mut self, candle: &Bound<'_, PyAny>) -> PyResult<Option<f64>> {
        let c = extract_candle(candle)?;
        Ok(self.inner.update(c))
    }
    /// Batch over numpy columns: high, low, close (all 1-D, equal length).
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        high: PyReadonlyArray1<'py, f64>,
        low: PyReadonlyArray1<'py, f64>,
        close: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let h = high
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let l = low
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let c = close
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        if h.len() != l.len() || l.len() != c.len() {
            return Err(PyValueError::new_err(
                "high, low, close must be equal length",
            ));
        }
        let mut out = Vec::with_capacity(h.len());
        for i in 0..h.len() {
            let candle = wc::Candle::new(c[i], h[i], l[i], c[i], 0.0, 0).map_err(map_err)?;
            out.push(self.inner.update(candle).unwrap_or(f64::NAN));
        }
        Ok(out.into_pyarray(py))
    }
    #[getter]
    fn period(&self) -> usize {
        self.inner.period()
    }
    #[getter]
    fn value(&self) -> Option<f64> {
        self.inner.value()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        format!("NATR(period={})", self.inner.period())
    }
}

// ============================== StdDev ==============================

#[pyclass(name = "StdDev", module = "wickra._wickra", skip_from_py_object)]
#[derive(Clone)]
struct PyStdDev {
    inner: wc::StdDev,
}

#[pymethods]
impl PyStdDev {
    #[new]
    #[pyo3(signature = (period=20))]
    fn new(period: usize) -> PyResult<Self> {
        Ok(Self {
            inner: wc::StdDev::new(period).map_err(map_err)?,
        })
    }
    fn update(&mut self, value: f64) -> Option<f64> {
        self.inner.update(value)
    }
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        prices: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let slice = prices
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        Ok(flatten(self.inner.batch(slice)).into_pyarray(py))
    }
    #[getter]
    fn period(&self) -> usize {
        self.inner.period()
    }
    #[getter]
    fn value(&self) -> Option<f64> {
        self.inner.value()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        format!("StdDev(period={})", self.inner.period())
    }
}

// ============================== Ulcer Index ==============================

#[pyclass(name = "UlcerIndex", module = "wickra._wickra", skip_from_py_object)]
#[derive(Clone)]
struct PyUlcerIndex {
    inner: wc::UlcerIndex,
}

#[pymethods]
impl PyUlcerIndex {
    #[new]
    #[pyo3(signature = (period=14))]
    fn new(period: usize) -> PyResult<Self> {
        Ok(Self {
            inner: wc::UlcerIndex::new(period).map_err(map_err)?,
        })
    }
    fn update(&mut self, value: f64) -> Option<f64> {
        self.inner.update(value)
    }
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        prices: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let slice = prices
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        Ok(flatten(self.inner.batch(slice)).into_pyarray(py))
    }
    #[getter]
    fn period(&self) -> usize {
        self.inner.period()
    }
    #[getter]
    fn value(&self) -> Option<f64> {
        self.inner.value()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        format!("UlcerIndex(period={})", self.inner.period())
    }
}

// ============================== Historical Volatility ==============================

#[pyclass(
    name = "HistoricalVolatility",
    module = "wickra._wickra",
    skip_from_py_object
)]
#[derive(Clone)]
struct PyHistoricalVolatility {
    inner: wc::HistoricalVolatility,
}

#[pymethods]
impl PyHistoricalVolatility {
    #[new]
    #[pyo3(signature = (period=20, trading_periods=252))]
    fn new(period: usize, trading_periods: usize) -> PyResult<Self> {
        Ok(Self {
            inner: wc::HistoricalVolatility::new(period, trading_periods).map_err(map_err)?,
        })
    }
    fn update(&mut self, value: f64) -> Option<f64> {
        self.inner.update(value)
    }
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        prices: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let slice = prices
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        Ok(flatten(self.inner.batch(slice)).into_pyarray(py))
    }
    #[getter]
    fn periods(&self) -> (usize, usize) {
        self.inner.periods()
    }
    #[getter]
    fn value(&self) -> Option<f64> {
        self.inner.value()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        let (p, t) = self.inner.periods();
        format!("HistoricalVolatility(period={p}, trading_periods={t})")
    }
}

// ============================== Aroon Oscillator ==============================

#[pyclass(
    name = "AroonOscillator",
    module = "wickra._wickra",
    skip_from_py_object
)]
#[derive(Clone)]
struct PyAroonOscillator {
    inner: wc::AroonOscillator,
}

#[pymethods]
impl PyAroonOscillator {
    #[new]
    #[pyo3(signature = (period=14))]
    fn new(period: usize) -> PyResult<Self> {
        Ok(Self {
            inner: wc::AroonOscillator::new(period).map_err(map_err)?,
        })
    }
    fn update(&mut self, candle: &Bound<'_, PyAny>) -> PyResult<Option<f64>> {
        let c = extract_candle(candle)?;
        Ok(self.inner.update(c))
    }
    /// Batch over numpy high + low columns (both 1-D, equal length).
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        high: PyReadonlyArray1<'py, f64>,
        low: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let h = high
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let l = low
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        if h.len() != l.len() {
            return Err(PyValueError::new_err("high and low must be equal length"));
        }
        let mut out = Vec::with_capacity(h.len());
        for i in 0..h.len() {
            let candle = wc::Candle::new(l[i], h[i], l[i], l[i], 0.0, 0).map_err(map_err)?;
            out.push(self.inner.update(candle).unwrap_or(f64::NAN));
        }
        Ok(out.into_pyarray(py))
    }
    #[getter]
    fn period(&self) -> usize {
        self.inner.period()
    }
    #[getter]
    fn value(&self) -> Option<f64> {
        self.inner.value()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        format!("AroonOscillator(period={})", self.inner.period())
    }
}

// ============================== Vortex ==============================

#[pyclass(name = "Vortex", module = "wickra._wickra", skip_from_py_object)]
#[derive(Clone)]
struct PyVortex {
    inner: wc::Vortex,
}

#[pymethods]
impl PyVortex {
    #[new]
    #[pyo3(signature = (period=14))]
    fn new(period: usize) -> PyResult<Self> {
        Ok(Self {
            inner: wc::Vortex::new(period).map_err(map_err)?,
        })
    }
    /// Returns `(plus, minus)` or `None` during warmup.
    fn update(&mut self, candle: &Bound<'_, PyAny>) -> PyResult<Option<(f64, f64)>> {
        let c = extract_candle(candle)?;
        Ok(self.inner.update(c).map(|o| (o.plus, o.minus)))
    }
    /// Batch over high/low/close numpy columns. Returns shape `(n, 2)` for `[plus, minus]`.
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        high: PyReadonlyArray1<'py, f64>,
        low: PyReadonlyArray1<'py, f64>,
        close: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray2<f64>>> {
        let h = high
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let l = low
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let c = close
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        if h.len() != l.len() || l.len() != c.len() {
            return Err(PyValueError::new_err(
                "high, low, close must be equal length",
            ));
        }
        let n = h.len();
        let mut out = vec![f64::NAN; n * 2];
        for i in 0..n {
            let candle = wc::Candle::new(c[i], h[i], l[i], c[i], 0.0, 0).map_err(map_err)?;
            if let Some(o) = self.inner.update(candle) {
                out[i * 2] = o.plus;
                out[i * 2 + 1] = o.minus;
            }
        }
        Ok(numpy::ndarray::Array2::from_shape_vec((n, 2), out)
            .expect("shape consistent")
            .into_pyarray(py))
    }
    #[getter]
    fn period(&self) -> usize {
        self.inner.period()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        format!("Vortex(period={})", self.inner.period())
    }
}

// ============================== RWI ==============================

#[pyclass(name = "RWI", module = "wickra._wickra", skip_from_py_object)]
#[derive(Clone)]
struct PyRwi {
    inner: wc::Rwi,
}

#[pymethods]
impl PyRwi {
    #[new]
    #[pyo3(signature = (period=14))]
    fn new(period: usize) -> PyResult<Self> {
        Ok(Self {
            inner: wc::Rwi::new(period).map_err(map_err)?,
        })
    }
    /// Returns `(high, low)` or `None` during warmup.
    fn update(&mut self, candle: &Bound<'_, PyAny>) -> PyResult<Option<(f64, f64)>> {
        let c = extract_candle(candle)?;
        Ok(self.inner.update(c).map(|o| (o.high, o.low)))
    }
    /// Batch over high/low/close numpy columns. Returns shape `(n, 2)` for `[high, low]`.
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        high: PyReadonlyArray1<'py, f64>,
        low: PyReadonlyArray1<'py, f64>,
        close: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray2<f64>>> {
        let h = high
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let l = low
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let c = close
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        if h.len() != l.len() || l.len() != c.len() {
            return Err(PyValueError::new_err(
                "high, low, close must be equal length",
            ));
        }
        let n = h.len();
        let mut out = vec![f64::NAN; n * 2];
        for i in 0..n {
            let candle = wc::Candle::new(c[i], h[i], l[i], c[i], 0.0, 0).map_err(map_err)?;
            if let Some(o) = self.inner.update(candle) {
                out[i * 2] = o.high;
                out[i * 2 + 1] = o.low;
            }
        }
        Ok(numpy::ndarray::Array2::from_shape_vec((n, 2), out)
            .expect("shape consistent")
            .into_pyarray(py))
    }
    #[getter]
    fn period(&self) -> usize {
        self.inner.period()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        format!("RWI(period={})", self.inner.period())
    }
}

// ============================== WaveTrend ==============================

#[pyclass(name = "WaveTrend", module = "wickra._wickra", skip_from_py_object)]
#[derive(Clone)]
struct PyWaveTrend {
    inner: wc::WaveTrend,
}

#[pymethods]
impl PyWaveTrend {
    #[new]
    #[pyo3(signature = (channel_period=10, average_period=21, signal_period=4))]
    fn new(channel_period: usize, average_period: usize, signal_period: usize) -> PyResult<Self> {
        Ok(Self {
            inner: wc::WaveTrend::new(channel_period, average_period, signal_period)
                .map_err(map_err)?,
        })
    }
    #[staticmethod]
    fn classic() -> PyResult<Self> {
        Ok(Self {
            inner: wc::WaveTrend::classic().map_err(map_err)?,
        })
    }
    fn update(&mut self, candle: &Bound<'_, PyAny>) -> PyResult<Option<(f64, f64)>> {
        let c = extract_candle(candle)?;
        Ok(self.inner.update(c).map(|o| (o.wt1, o.wt2)))
    }
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        high: PyReadonlyArray1<'py, f64>,
        low: PyReadonlyArray1<'py, f64>,
        close: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray2<f64>>> {
        let h = high
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let l = low
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let c = close
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        if h.len() != l.len() || l.len() != c.len() {
            return Err(PyValueError::new_err(
                "high, low, close must be equal length",
            ));
        }
        let n = h.len();
        let mut out = vec![f64::NAN; n * 2];
        for i in 0..n {
            let candle = wc::Candle::new(c[i], h[i], l[i], c[i], 0.0, 0).map_err(map_err)?;
            if let Some(o) = self.inner.update(candle) {
                out[i * 2] = o.wt1;
                out[i * 2 + 1] = o.wt2;
            }
        }
        Ok(numpy::ndarray::Array2::from_shape_vec((n, 2), out)
            .expect("shape consistent")
            .into_pyarray(py))
    }
    #[getter]
    fn periods(&self) -> (usize, usize, usize) {
        self.inner.periods()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        let (cp, ap, sp) = self.inner.periods();
        format!("WaveTrend(channel_period={cp}, average_period={ap}, signal_period={sp})")
    }
}

// ============================== Mass Index ==============================

#[pyclass(name = "MassIndex", module = "wickra._wickra", skip_from_py_object)]
#[derive(Clone)]
struct PyMassIndex {
    inner: wc::MassIndex,
}

#[pymethods]
impl PyMassIndex {
    #[new]
    #[pyo3(signature = (ema_period=9, sum_period=25))]
    fn new(ema_period: usize, sum_period: usize) -> PyResult<Self> {
        Ok(Self {
            inner: wc::MassIndex::new(ema_period, sum_period).map_err(map_err)?,
        })
    }
    fn update(&mut self, candle: &Bound<'_, PyAny>) -> PyResult<Option<f64>> {
        let c = extract_candle(candle)?;
        Ok(self.inner.update(c))
    }
    /// Batch over numpy high + low columns (both 1-D, equal length).
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        high: PyReadonlyArray1<'py, f64>,
        low: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let h = high
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let l = low
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        if h.len() != l.len() {
            return Err(PyValueError::new_err("high and low must be equal length"));
        }
        let mut out = Vec::with_capacity(h.len());
        for i in 0..h.len() {
            let candle = wc::Candle::new(l[i], h[i], l[i], l[i], 0.0, 0).map_err(map_err)?;
            out.push(self.inner.update(candle).unwrap_or(f64::NAN));
        }
        Ok(out.into_pyarray(py))
    }
    #[getter]
    fn periods(&self) -> (usize, usize) {
        self.inner.periods()
    }
    #[getter]
    fn value(&self) -> Option<f64> {
        self.inner.value()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        let (e, s) = self.inner.periods();
        format!("MassIndex(ema_period={e}, sum_period={s})")
    }
}

// ============================== PPO ==============================

#[pyclass(name = "PPO", module = "wickra._wickra", skip_from_py_object)]
#[derive(Clone)]
struct PyPpo {
    inner: wc::Ppo,
}

#[pymethods]
impl PyPpo {
    #[new]
    #[pyo3(signature = (fast=12, slow=26))]
    fn new(fast: usize, slow: usize) -> PyResult<Self> {
        Ok(Self {
            inner: wc::Ppo::new(fast, slow).map_err(map_err)?,
        })
    }
    fn update(&mut self, value: f64) -> Option<f64> {
        self.inner.update(value)
    }
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        prices: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let slice = prices
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        Ok(flatten(self.inner.batch(slice)).into_pyarray(py))
    }
    #[getter]
    fn periods(&self) -> (usize, usize) {
        self.inner.periods()
    }
    #[getter]
    fn value(&self) -> Option<f64> {
        self.inner.value()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        let (f, s) = self.inner.periods();
        format!("PPO(fast={f}, slow={s})")
    }
}

// ============================== DPO ==============================

#[pyclass(name = "DPO", module = "wickra._wickra", skip_from_py_object)]
#[derive(Clone)]
struct PyDpo {
    inner: wc::Dpo,
}

#[pymethods]
impl PyDpo {
    #[new]
    #[pyo3(signature = (period=20))]
    fn new(period: usize) -> PyResult<Self> {
        Ok(Self {
            inner: wc::Dpo::new(period).map_err(map_err)?,
        })
    }
    fn update(&mut self, value: f64) -> Option<f64> {
        self.inner.update(value)
    }
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        prices: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let slice = prices
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        Ok(flatten(self.inner.batch(slice)).into_pyarray(py))
    }
    #[getter]
    fn period(&self) -> usize {
        self.inner.period()
    }
    #[getter]
    fn shift(&self) -> usize {
        self.inner.shift()
    }
    #[getter]
    fn value(&self) -> Option<f64> {
        self.inner.value()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        format!("DPO(period={})", self.inner.period())
    }
}

// ============================== Coppock ==============================

#[pyclass(name = "Coppock", module = "wickra._wickra", skip_from_py_object)]
#[derive(Clone)]
struct PyCoppock {
    inner: wc::Coppock,
}

#[pymethods]
impl PyCoppock {
    #[new]
    #[pyo3(signature = (roc_long=14, roc_short=11, wma_period=10))]
    fn new(roc_long: usize, roc_short: usize, wma_period: usize) -> PyResult<Self> {
        Ok(Self {
            inner: wc::Coppock::new(roc_long, roc_short, wma_period).map_err(map_err)?,
        })
    }
    fn update(&mut self, value: f64) -> Option<f64> {
        self.inner.update(value)
    }
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        prices: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let slice = prices
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        Ok(flatten(self.inner.batch(slice)).into_pyarray(py))
    }
    #[getter]
    fn periods(&self) -> (usize, usize, usize) {
        self.inner.periods()
    }
    #[getter]
    fn value(&self) -> Option<f64> {
        self.inner.value()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        let (l, s, w) = self.inner.periods();
        format!("Coppock(roc_long={l}, roc_short={s}, wma_period={w})")
    }
}

// ============================== StochRSI ==============================

#[pyclass(name = "StochRSI", module = "wickra._wickra", skip_from_py_object)]
#[derive(Clone)]
struct PyStochRsi {
    inner: wc::StochRsi,
}

#[pymethods]
impl PyStochRsi {
    #[new]
    #[pyo3(signature = (rsi_period=14, stoch_period=14))]
    fn new(rsi_period: usize, stoch_period: usize) -> PyResult<Self> {
        Ok(Self {
            inner: wc::StochRsi::new(rsi_period, stoch_period).map_err(map_err)?,
        })
    }
    fn update(&mut self, value: f64) -> Option<f64> {
        self.inner.update(value)
    }
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        prices: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let slice = prices
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        Ok(flatten(self.inner.batch(slice)).into_pyarray(py))
    }
    #[getter]
    fn periods(&self) -> (usize, usize) {
        self.inner.periods()
    }
    #[getter]
    fn value(&self) -> Option<f64> {
        self.inner.value()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        let (r, s) = self.inner.periods();
        format!("StochRSI(rsi_period={r}, stoch_period={s})")
    }
}

// ============================== Ultimate Oscillator ==============================

#[pyclass(
    name = "UltimateOscillator",
    module = "wickra._wickra",
    skip_from_py_object
)]
#[derive(Clone)]
struct PyUltimateOscillator {
    inner: wc::UltimateOscillator,
}

#[pymethods]
impl PyUltimateOscillator {
    #[new]
    #[pyo3(signature = (short=7, mid=14, long=28))]
    fn new(short: usize, mid: usize, long: usize) -> PyResult<Self> {
        Ok(Self {
            inner: wc::UltimateOscillator::new(short, mid, long).map_err(map_err)?,
        })
    }
    fn update(&mut self, candle: &Bound<'_, PyAny>) -> PyResult<Option<f64>> {
        let c = extract_candle(candle)?;
        Ok(self.inner.update(c))
    }
    /// Batch over numpy columns: high, low, close (all 1-D, equal length).
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        high: PyReadonlyArray1<'py, f64>,
        low: PyReadonlyArray1<'py, f64>,
        close: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let h = high
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let l = low
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let c = close
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        if h.len() != l.len() || l.len() != c.len() {
            return Err(PyValueError::new_err(
                "high, low, close must be equal length",
            ));
        }
        let mut out = Vec::with_capacity(h.len());
        for i in 0..h.len() {
            let candle = wc::Candle::new(c[i], h[i], l[i], c[i], 0.0, 0).map_err(map_err)?;
            out.push(self.inner.update(candle).unwrap_or(f64::NAN));
        }
        Ok(out.into_pyarray(py))
    }
    #[getter]
    fn periods(&self) -> (usize, usize, usize) {
        self.inner.periods()
    }
    #[getter]
    fn value(&self) -> Option<f64> {
        self.inner.value()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        let (s, m, l) = self.inner.periods();
        format!("UltimateOscillator(short={s}, mid={m}, long={l})")
    }
}

// ============================== MOM ==============================

#[pyclass(name = "MOM", module = "wickra._wickra", skip_from_py_object)]
#[derive(Clone)]
struct PyMom {
    inner: wc::Mom,
}

#[pymethods]
impl PyMom {
    #[new]
    #[pyo3(signature = (period=10))]
    fn new(period: usize) -> PyResult<Self> {
        Ok(Self {
            inner: wc::Mom::new(period).map_err(map_err)?,
        })
    }
    fn update(&mut self, value: f64) -> Option<f64> {
        self.inner.update(value)
    }
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        prices: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let slice = prices
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        Ok(flatten(self.inner.batch(slice)).into_pyarray(py))
    }
    #[getter]
    fn period(&self) -> usize {
        self.inner.period()
    }
    #[getter]
    fn value(&self) -> Option<f64> {
        self.inner.value()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        format!("MOM(period={})", self.inner.period())
    }
}

// ============================== CMO ==============================

#[pyclass(name = "CMO", module = "wickra._wickra", skip_from_py_object)]
#[derive(Clone)]
struct PyCmo {
    inner: wc::Cmo,
}

#[pymethods]
impl PyCmo {
    #[new]
    #[pyo3(signature = (period=14))]
    fn new(period: usize) -> PyResult<Self> {
        Ok(Self {
            inner: wc::Cmo::new(period).map_err(map_err)?,
        })
    }
    fn update(&mut self, value: f64) -> Option<f64> {
        self.inner.update(value)
    }
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        prices: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let slice = prices
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        Ok(flatten(self.inner.batch(slice)).into_pyarray(py))
    }
    #[getter]
    fn period(&self) -> usize {
        self.inner.period()
    }
    #[getter]
    fn value(&self) -> Option<f64> {
        self.inner.value()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        format!("CMO(period={})", self.inner.period())
    }
}

// ============================== TSI ==============================

#[pyclass(name = "TSI", module = "wickra._wickra", skip_from_py_object)]
#[derive(Clone)]
struct PyTsi {
    inner: wc::Tsi,
}

#[pymethods]
impl PyTsi {
    #[new]
    #[pyo3(signature = (long=25, short=13))]
    fn new(long: usize, short: usize) -> PyResult<Self> {
        Ok(Self {
            inner: wc::Tsi::new(long, short).map_err(map_err)?,
        })
    }
    fn update(&mut self, value: f64) -> Option<f64> {
        self.inner.update(value)
    }
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        prices: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let slice = prices
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        Ok(flatten(self.inner.batch(slice)).into_pyarray(py))
    }
    #[getter]
    fn periods(&self) -> (usize, usize) {
        self.inner.periods()
    }
    #[getter]
    fn value(&self) -> Option<f64> {
        self.inner.value()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        let (l, s) = self.inner.periods();
        format!("TSI(long={l}, short={s})")
    }
}

// ============================== PMO ==============================

#[pyclass(name = "PMO", module = "wickra._wickra", skip_from_py_object)]
#[derive(Clone)]
struct PyPmo {
    inner: wc::Pmo,
}

#[pymethods]
impl PyPmo {
    #[new]
    #[pyo3(signature = (smoothing1=35, smoothing2=20))]
    fn new(smoothing1: usize, smoothing2: usize) -> PyResult<Self> {
        Ok(Self {
            inner: wc::Pmo::new(smoothing1, smoothing2).map_err(map_err)?,
        })
    }
    fn update(&mut self, value: f64) -> Option<f64> {
        self.inner.update(value)
    }
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        prices: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let slice = prices
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        Ok(flatten(self.inner.batch(slice)).into_pyarray(py))
    }
    #[getter]
    fn periods(&self) -> (usize, usize) {
        self.inner.periods()
    }
    #[getter]
    fn value(&self) -> Option<f64> {
        self.inner.value()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        let (s1, s2) = self.inner.periods();
        format!("PMO(smoothing1={s1}, smoothing2={s2})")
    }
}

// ============================== TII ==============================

#[pyclass(name = "TII", module = "wickra._wickra", skip_from_py_object)]
#[derive(Clone)]
struct PyTii {
    inner: wc::Tii,
}

#[pymethods]
impl PyTii {
    #[new]
    #[pyo3(signature = (sma_period=60, dev_period=30))]
    fn new(sma_period: usize, dev_period: usize) -> PyResult<Self> {
        Ok(Self {
            inner: wc::Tii::new(sma_period, dev_period).map_err(map_err)?,
        })
    }
    fn update(&mut self, value: f64) -> Option<f64> {
        self.inner.update(value)
    }
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        prices: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let slice = prices
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        Ok(flatten(self.inner.batch(slice)).into_pyarray(py))
    }
    #[getter]
    fn periods(&self) -> (usize, usize) {
        self.inner.periods()
    }
    #[getter]
    fn value(&self) -> Option<f64> {
        self.inner.value()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        let (s, d) = self.inner.periods();
        format!("TII(sma_period={s}, dev_period={d})")
    }
}

// ============================== ZLEMA ==============================

#[pyclass(name = "ZLEMA", module = "wickra._wickra", skip_from_py_object)]
#[derive(Clone)]
struct PyZlema {
    inner: wc::Zlema,
}

#[pymethods]
impl PyZlema {
    #[new]
    fn new(period: usize) -> PyResult<Self> {
        Ok(Self {
            inner: wc::Zlema::new(period).map_err(map_err)?,
        })
    }
    fn update(&mut self, value: f64) -> Option<f64> {
        self.inner.update(value)
    }
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        prices: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let slice = prices
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        Ok(flatten(self.inner.batch(slice)).into_pyarray(py))
    }
    #[getter]
    fn period(&self) -> usize {
        self.inner.period()
    }
    #[getter]
    fn lag(&self) -> usize {
        self.inner.lag()
    }
    #[getter]
    fn value(&self) -> Option<f64> {
        self.inner.value()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        format!("ZLEMA(period={})", self.inner.period())
    }
}

// ============================== T3 ==============================

#[pyclass(name = "T3", module = "wickra._wickra", skip_from_py_object)]
#[derive(Clone)]
struct PyT3 {
    inner: wc::T3,
}

#[pymethods]
impl PyT3 {
    #[new]
    #[pyo3(signature = (period, v=0.7))]
    fn new(period: usize, v: f64) -> PyResult<Self> {
        Ok(Self {
            inner: wc::T3::new(period, v).map_err(map_err)?,
        })
    }
    fn update(&mut self, value: f64) -> Option<f64> {
        self.inner.update(value)
    }
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        prices: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let slice = prices
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        Ok(flatten(self.inner.batch(slice)).into_pyarray(py))
    }
    #[getter]
    fn period(&self) -> usize {
        self.inner.period()
    }
    #[getter]
    fn volume_factor(&self) -> f64 {
        self.inner.volume_factor()
    }
    #[getter]
    fn value(&self) -> Option<f64> {
        self.inner.value()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        format!(
            "T3(period={}, v={})",
            self.inner.period(),
            self.inner.volume_factor()
        )
    }
}

// ============================== VWMA ==============================

#[pyclass(name = "VWMA", module = "wickra._wickra", skip_from_py_object)]
#[derive(Clone)]
struct PyVwma {
    inner: wc::Vwma,
}

#[pymethods]
impl PyVwma {
    #[new]
    fn new(period: usize) -> PyResult<Self> {
        Ok(Self {
            inner: wc::Vwma::new(period).map_err(map_err)?,
        })
    }
    fn update(&mut self, candle: &Bound<'_, PyAny>) -> PyResult<Option<f64>> {
        let c = extract_candle(candle)?;
        Ok(self.inner.update(c))
    }
    /// Batch over numpy close + volume arrays (both 1-D, equal length).
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        close: PyReadonlyArray1<'py, f64>,
        volume: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let c = close
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let v = volume
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        if c.len() != v.len() {
            return Err(PyValueError::new_err(
                "close and volume must be equal length",
            ));
        }
        let mut out = Vec::with_capacity(c.len());
        for i in 0..c.len() {
            let candle = wc::Candle::new(c[i], c[i], c[i], c[i], v[i], 0).map_err(map_err)?;
            out.push(self.inner.update(candle).unwrap_or(f64::NAN));
        }
        Ok(out.into_pyarray(py))
    }
    #[getter]
    fn period(&self) -> usize {
        self.inner.period()
    }
    #[getter]
    fn value(&self) -> Option<f64> {
        self.inner.value()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        format!("VWMA(period={})", self.inner.period())
    }
}

// ============================== SMMA ==============================

#[pyclass(name = "SMMA", module = "wickra._wickra", skip_from_py_object)]
#[derive(Clone)]
struct PySmma {
    inner: wc::Smma,
}

#[pymethods]
impl PySmma {
    #[new]
    fn new(period: usize) -> PyResult<Self> {
        Ok(Self {
            inner: wc::Smma::new(period).map_err(map_err)?,
        })
    }
    fn update(&mut self, value: f64) -> Option<f64> {
        self.inner.update(value)
    }
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        prices: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let slice = prices
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        Ok(flatten(self.inner.batch(slice)).into_pyarray(py))
    }
    #[getter]
    fn period(&self) -> usize {
        self.inner.period()
    }
    #[getter]
    fn value(&self) -> Option<f64> {
        self.inner.value()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        format!("SMMA(period={})", self.inner.period())
    }
}

// ============================== TRIMA ==============================

#[pyclass(name = "TRIMA", module = "wickra._wickra", skip_from_py_object)]
#[derive(Clone)]
struct PyTrima {
    inner: wc::Trima,
}

#[pymethods]
impl PyTrima {
    #[new]
    fn new(period: usize) -> PyResult<Self> {
        Ok(Self {
            inner: wc::Trima::new(period).map_err(map_err)?,
        })
    }
    fn update(&mut self, value: f64) -> Option<f64> {
        self.inner.update(value)
    }
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        prices: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let slice = prices
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        Ok(flatten(self.inner.batch(slice)).into_pyarray(py))
    }
    #[getter]
    fn period(&self) -> usize {
        self.inner.period()
    }
    #[getter]
    fn value(&self) -> Option<f64> {
        self.inner.value()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        format!("TRIMA(period={})", self.inner.period())
    }
}

// ============================== Chaikin Money Flow ==============================

#[pyclass(
    name = "ChaikinMoneyFlow",
    module = "wickra._wickra",
    skip_from_py_object
)]
#[derive(Clone)]
struct PyChaikinMoneyFlow {
    inner: wc::ChaikinMoneyFlow,
}

#[pymethods]
impl PyChaikinMoneyFlow {
    #[new]
    #[pyo3(signature = (period=20))]
    fn new(period: usize) -> PyResult<Self> {
        Ok(Self {
            inner: wc::ChaikinMoneyFlow::new(period).map_err(map_err)?,
        })
    }
    fn update(&mut self, candle: &Bound<'_, PyAny>) -> PyResult<Option<f64>> {
        let c = extract_candle(candle)?;
        Ok(self.inner.update(c))
    }
    /// Batch over numpy columns: high, low, close, volume (all equal length).
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        high: PyReadonlyArray1<'py, f64>,
        low: PyReadonlyArray1<'py, f64>,
        close: PyReadonlyArray1<'py, f64>,
        volume: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let h = high
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let l = low
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let c = close
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let v = volume
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        if h.len() != l.len() || l.len() != c.len() || c.len() != v.len() {
            return Err(PyValueError::new_err(
                "high, low, close, volume must be equal length",
            ));
        }
        let mut out = Vec::with_capacity(h.len());
        for i in 0..h.len() {
            let candle = wc::Candle::new(c[i], h[i], l[i], c[i], v[i], 0).map_err(map_err)?;
            out.push(self.inner.update(candle).unwrap_or(f64::NAN));
        }
        Ok(out.into_pyarray(py))
    }
    #[getter]
    fn period(&self) -> usize {
        self.inner.period()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        format!("ChaikinMoneyFlow(period={})", self.inner.period())
    }
}

// ============================== Chaikin Oscillator ==============================

#[pyclass(
    name = "ChaikinOscillator",
    module = "wickra._wickra",
    skip_from_py_object
)]
#[derive(Clone)]
struct PyChaikinOscillator {
    inner: wc::ChaikinOscillator,
}

#[pymethods]
impl PyChaikinOscillator {
    #[new]
    #[pyo3(signature = (fast=3, slow=10))]
    fn new(fast: usize, slow: usize) -> PyResult<Self> {
        Ok(Self {
            inner: wc::ChaikinOscillator::new(fast, slow).map_err(map_err)?,
        })
    }
    fn update(&mut self, candle: &Bound<'_, PyAny>) -> PyResult<Option<f64>> {
        let c = extract_candle(candle)?;
        Ok(self.inner.update(c))
    }
    /// Batch over numpy columns: high, low, close, volume (all equal length).
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        high: PyReadonlyArray1<'py, f64>,
        low: PyReadonlyArray1<'py, f64>,
        close: PyReadonlyArray1<'py, f64>,
        volume: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let h = high
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let l = low
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let c = close
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let v = volume
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        if h.len() != l.len() || l.len() != c.len() || c.len() != v.len() {
            return Err(PyValueError::new_err(
                "high, low, close, volume must be equal length",
            ));
        }
        let mut out = Vec::with_capacity(h.len());
        for i in 0..h.len() {
            let candle = wc::Candle::new(c[i], h[i], l[i], c[i], v[i], 0).map_err(map_err)?;
            out.push(self.inner.update(candle).unwrap_or(f64::NAN));
        }
        Ok(out.into_pyarray(py))
    }
    #[getter]
    fn periods(&self) -> (usize, usize) {
        self.inner.periods()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        let (fast, slow) = self.inner.periods();
        format!("ChaikinOscillator(fast={fast}, slow={slow})")
    }
}

// ============================== Force Index ==============================

#[pyclass(name = "ForceIndex", module = "wickra._wickra", skip_from_py_object)]
#[derive(Clone)]
struct PyForceIndex {
    inner: wc::ForceIndex,
}

#[pymethods]
impl PyForceIndex {
    #[new]
    #[pyo3(signature = (period=13))]
    fn new(period: usize) -> PyResult<Self> {
        Ok(Self {
            inner: wc::ForceIndex::new(period).map_err(map_err)?,
        })
    }
    fn update(&mut self, candle: &Bound<'_, PyAny>) -> PyResult<Option<f64>> {
        let c = extract_candle(candle)?;
        Ok(self.inner.update(c))
    }
    /// Batch over numpy close + volume arrays (both 1-D, equal length).
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        close: PyReadonlyArray1<'py, f64>,
        volume: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let c = close
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let v = volume
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        if c.len() != v.len() {
            return Err(PyValueError::new_err(
                "close and volume must be equal length",
            ));
        }
        let mut out = Vec::with_capacity(c.len());
        for i in 0..c.len() {
            let candle = wc::Candle::new(c[i], c[i], c[i], c[i], v[i], 0).map_err(map_err)?;
            out.push(self.inner.update(candle).unwrap_or(f64::NAN));
        }
        Ok(out.into_pyarray(py))
    }
    #[getter]
    fn period(&self) -> usize {
        self.inner.period()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        format!("ForceIndex(period={})", self.inner.period())
    }
}

// ============================== Negative Volume Index ==============================

#[pyclass(name = "NVI", module = "wickra._wickra", skip_from_py_object)]
#[derive(Clone)]
struct PyNvi {
    inner: wc::Nvi,
}

#[pymethods]
impl PyNvi {
    #[new]
    #[pyo3(signature = (baseline=1000.0))]
    fn new(baseline: f64) -> Self {
        Self {
            inner: wc::Nvi::with_baseline(baseline),
        }
    }
    fn update(&mut self, candle: &Bound<'_, PyAny>) -> PyResult<Option<f64>> {
        let c = extract_candle(candle)?;
        Ok(self.inner.update(c))
    }
    /// Batch over close + volume numpy arrays.
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        close: PyReadonlyArray1<'py, f64>,
        volume: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let c = close
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let v = volume
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        if c.len() != v.len() {
            return Err(PyValueError::new_err(
                "close and volume must be equal length",
            ));
        }
        let mut out = Vec::with_capacity(c.len());
        for i in 0..c.len() {
            let candle = wc::Candle::new(c[i], c[i], c[i], c[i], v[i], 0).map_err(map_err)?;
            out.push(self.inner.update(candle).unwrap_or(f64::NAN));
        }
        Ok(out.into_pyarray(py))
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        "NVI()".to_string()
    }
}

// ============================== Positive Volume Index ==============================

#[pyclass(name = "PVI", module = "wickra._wickra", skip_from_py_object)]
#[derive(Clone)]
struct PyPvi {
    inner: wc::Pvi,
}

#[pymethods]
impl PyPvi {
    #[new]
    #[pyo3(signature = (baseline=1000.0))]
    fn new(baseline: f64) -> Self {
        Self {
            inner: wc::Pvi::with_baseline(baseline),
        }
    }
    fn update(&mut self, candle: &Bound<'_, PyAny>) -> PyResult<Option<f64>> {
        let c = extract_candle(candle)?;
        Ok(self.inner.update(c))
    }
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        close: PyReadonlyArray1<'py, f64>,
        volume: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let c = close
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let v = volume
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        if c.len() != v.len() {
            return Err(PyValueError::new_err(
                "close and volume must be equal length",
            ));
        }
        let mut out = Vec::with_capacity(c.len());
        for i in 0..c.len() {
            let candle = wc::Candle::new(c[i], c[i], c[i], c[i], v[i], 0).map_err(map_err)?;
            out.push(self.inner.update(candle).unwrap_or(f64::NAN));
        }
        Ok(out.into_pyarray(py))
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        "PVI()".to_string()
    }
}

// ============================== Volume Oscillator ==============================

#[pyclass(
    name = "VolumeOscillator",
    module = "wickra._wickra",
    skip_from_py_object
)]
#[derive(Clone)]
struct PyVolumeOscillator {
    inner: wc::VolumeOscillator,
}

#[pymethods]
impl PyVolumeOscillator {
    #[new]
    #[pyo3(signature = (fast=14, slow=28))]
    fn new(fast: usize, slow: usize) -> PyResult<Self> {
        Ok(Self {
            inner: wc::VolumeOscillator::new(fast, slow).map_err(map_err)?,
        })
    }
    fn update(&mut self, candle: &Bound<'_, PyAny>) -> PyResult<Option<f64>> {
        let c = extract_candle(candle)?;
        Ok(self.inner.update(c))
    }
    /// Batch over a 1-D numpy volume array.
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        volume: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let v = volume
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let mut out = Vec::with_capacity(v.len());
        for &vol in v {
            let candle = wc::Candle::new(10.0, 10.0, 10.0, 10.0, vol, 0).map_err(map_err)?;
            out.push(self.inner.update(candle).unwrap_or(f64::NAN));
        }
        Ok(out.into_pyarray(py))
    }
    #[getter]
    fn periods(&self) -> (usize, usize) {
        self.inner.periods()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        let (fast, slow) = self.inner.periods();
        format!("VolumeOscillator(fast={fast}, slow={slow})")
    }
}

// ============================== Klinger Volume Oscillator ==============================

#[pyclass(name = "KVO", module = "wickra._wickra", skip_from_py_object)]
#[derive(Clone)]
struct PyKvo {
    inner: wc::Kvo,
}

#[pymethods]
impl PyKvo {
    #[new]
    #[pyo3(signature = (fast=34, slow=55))]
    fn new(fast: usize, slow: usize) -> PyResult<Self> {
        Ok(Self {
            inner: wc::Kvo::new(fast, slow).map_err(map_err)?,
        })
    }
    fn update(&mut self, candle: &Bound<'_, PyAny>) -> PyResult<Option<f64>> {
        let c = extract_candle(candle)?;
        Ok(self.inner.update(c))
    }
    /// Batch over high/low/close/volume numpy columns.
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        high: PyReadonlyArray1<'py, f64>,
        low: PyReadonlyArray1<'py, f64>,
        close: PyReadonlyArray1<'py, f64>,
        volume: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let h = high
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let l = low
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let c = close
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let v = volume
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        if h.len() != l.len() || l.len() != c.len() || c.len() != v.len() {
            return Err(PyValueError::new_err(
                "high, low, close, volume must be equal length",
            ));
        }
        let mut out = Vec::with_capacity(h.len());
        for i in 0..h.len() {
            let candle = wc::Candle::new(c[i], h[i], l[i], c[i], v[i], 0).map_err(map_err)?;
            out.push(self.inner.update(candle).unwrap_or(f64::NAN));
        }
        Ok(out.into_pyarray(py))
    }
    #[getter]
    fn periods(&self) -> (usize, usize) {
        self.inner.periods()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        let (fast, slow) = self.inner.periods();
        format!("KVO(fast={fast}, slow={slow})")
    }
}

// ============================== Williams A/D Oscillator ==============================

#[pyclass(name = "WilliamsAD", module = "wickra._wickra", skip_from_py_object)]
#[derive(Clone)]
struct PyAdOscillator {
    inner: wc::AdOscillator,
}

#[pymethods]
impl PyAdOscillator {
    #[new]
    fn new() -> Self {
        Self {
            inner: wc::AdOscillator::new(),
        }
    }
    fn update(&mut self, candle: &Bound<'_, PyAny>) -> PyResult<Option<f64>> {
        let c = extract_candle(candle)?;
        Ok(self.inner.update(c))
    }
    /// Batch over high/low/close numpy columns.
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        high: PyReadonlyArray1<'py, f64>,
        low: PyReadonlyArray1<'py, f64>,
        close: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let h = high
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let l = low
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let c = close
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        if h.len() != l.len() || l.len() != c.len() {
            return Err(PyValueError::new_err(
                "high, low, close must be equal length",
            ));
        }
        let mut out = Vec::with_capacity(h.len());
        for i in 0..h.len() {
            let candle = wc::Candle::new(c[i], h[i], l[i], c[i], 0.0, 0).map_err(map_err)?;
            out.push(self.inner.update(candle).unwrap_or(f64::NAN));
        }
        Ok(out.into_pyarray(py))
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        "WilliamsAD()".to_string()
    }
}

// ============================== Anchored VWAP ==============================

#[pyclass(name = "AnchoredVWAP", module = "wickra._wickra", skip_from_py_object)]
#[derive(Clone)]
struct PyAnchoredVwap {
    inner: wc::AnchoredVwap,
}

#[pymethods]
impl PyAnchoredVwap {
    #[new]
    fn new() -> Self {
        Self {
            inner: wc::AnchoredVwap::new(),
        }
    }
    fn update(&mut self, candle: &Bound<'_, PyAny>) -> PyResult<Option<f64>> {
        let c = extract_candle(candle)?;
        Ok(self.inner.update(c))
    }
    /// Re-anchor the cumulative window at the next bar that arrives.
    fn set_anchor(&mut self) {
        self.inner.set_anchor();
    }
    /// Batch over high/low/close/volume numpy columns.
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        high: PyReadonlyArray1<'py, f64>,
        low: PyReadonlyArray1<'py, f64>,
        close: PyReadonlyArray1<'py, f64>,
        volume: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let h = high
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let l = low
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let c = close
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let v = volume
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        if h.len() != l.len() || l.len() != c.len() || c.len() != v.len() {
            return Err(PyValueError::new_err(
                "high, low, close, volume must be equal length",
            ));
        }
        let mut out = Vec::with_capacity(h.len());
        for i in 0..h.len() {
            let candle = wc::Candle::new(c[i], h[i], l[i], c[i], v[i], 0).map_err(map_err)?;
            out.push(self.inner.update(candle).unwrap_or(f64::NAN));
        }
        Ok(out.into_pyarray(py))
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        "AnchoredVWAP()".to_string()
    }
}

// ============================== Demand Index ==============================

#[pyclass(name = "DemandIndex", module = "wickra._wickra", skip_from_py_object)]
#[derive(Clone)]
struct PyDemandIndex {
    inner: wc::DemandIndex,
}

#[pymethods]
impl PyDemandIndex {
    #[new]
    #[pyo3(signature = (period=10))]
    fn new(period: usize) -> PyResult<Self> {
        Ok(Self {
            inner: wc::DemandIndex::new(period).map_err(map_err)?,
        })
    }
    fn update(&mut self, candle: &Bound<'_, PyAny>) -> PyResult<Option<f64>> {
        let c = extract_candle(candle)?;
        Ok(self.inner.update(c))
    }
    /// Batch over high/low/close/volume numpy columns.
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        high: PyReadonlyArray1<'py, f64>,
        low: PyReadonlyArray1<'py, f64>,
        close: PyReadonlyArray1<'py, f64>,
        volume: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let h = high
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let l = low
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let c = close
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let v = volume
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        if h.len() != l.len() || l.len() != c.len() || c.len() != v.len() {
            return Err(PyValueError::new_err(
                "high, low, close, volume must be equal length",
            ));
        }
        let mut out = Vec::with_capacity(h.len());
        for i in 0..h.len() {
            let candle = wc::Candle::new(c[i], h[i], l[i], c[i], v[i], 0).map_err(map_err)?;
            out.push(self.inner.update(candle).unwrap_or(f64::NAN));
        }
        Ok(out.into_pyarray(py))
    }
    #[getter]
    fn period(&self) -> usize {
        self.inner.period()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        format!("DemandIndex(period={})", self.inner.period())
    }
}

// ============================== Time Segmented Volume ==============================

#[pyclass(name = "TSV", module = "wickra._wickra", skip_from_py_object)]
#[derive(Clone)]
struct PyTsv {
    inner: wc::Tsv,
}

#[pymethods]
impl PyTsv {
    #[new]
    #[pyo3(signature = (period=18))]
    fn new(period: usize) -> PyResult<Self> {
        Ok(Self {
            inner: wc::Tsv::new(period).map_err(map_err)?,
        })
    }
    fn update(&mut self, candle: &Bound<'_, PyAny>) -> PyResult<Option<f64>> {
        let c = extract_candle(candle)?;
        Ok(self.inner.update(c))
    }
    /// Batch over close + volume numpy columns.
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        close: PyReadonlyArray1<'py, f64>,
        volume: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let c = close
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let v = volume
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        if c.len() != v.len() {
            return Err(PyValueError::new_err(
                "close and volume must be equal length",
            ));
        }
        let mut out = Vec::with_capacity(c.len());
        for i in 0..c.len() {
            let candle = wc::Candle::new(c[i], c[i], c[i], c[i], v[i], 0).map_err(map_err)?;
            out.push(self.inner.update(candle).unwrap_or(f64::NAN));
        }
        Ok(out.into_pyarray(py))
    }
    #[getter]
    fn period(&self) -> usize {
        self.inner.period()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        format!("TSV(period={})", self.inner.period())
    }
}

// ============================== Volume Zone Oscillator ==============================

#[pyclass(name = "VZO", module = "wickra._wickra", skip_from_py_object)]
#[derive(Clone)]
struct PyVzo {
    inner: wc::Vzo,
}

#[pymethods]
impl PyVzo {
    #[new]
    #[pyo3(signature = (period=14))]
    fn new(period: usize) -> PyResult<Self> {
        Ok(Self {
            inner: wc::Vzo::new(period).map_err(map_err)?,
        })
    }
    fn update(&mut self, candle: &Bound<'_, PyAny>) -> PyResult<Option<f64>> {
        let c = extract_candle(candle)?;
        Ok(self.inner.update(c))
    }
    /// Batch over close + volume numpy columns.
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        close: PyReadonlyArray1<'py, f64>,
        volume: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let c = close
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let v = volume
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        if c.len() != v.len() {
            return Err(PyValueError::new_err(
                "close and volume must be equal length",
            ));
        }
        let mut out = Vec::with_capacity(c.len());
        for i in 0..c.len() {
            let candle = wc::Candle::new(c[i], c[i], c[i], c[i], v[i], 0).map_err(map_err)?;
            out.push(self.inner.update(candle).unwrap_or(f64::NAN));
        }
        Ok(out.into_pyarray(py))
    }
    #[getter]
    fn period(&self) -> usize {
        self.inner.period()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        format!("VZO(period={})", self.inner.period())
    }
}

// ============================== Market Facilitation Index ==============================

#[pyclass(
    name = "MarketFacilitationIndex",
    module = "wickra._wickra",
    skip_from_py_object
)]
#[derive(Clone)]
struct PyMarketFacilitationIndex {
    inner: wc::MarketFacilitationIndex,
}

#[pymethods]
impl PyMarketFacilitationIndex {
    #[new]
    fn new() -> Self {
        Self {
            inner: wc::MarketFacilitationIndex::new(),
        }
    }
    fn update(&mut self, candle: &Bound<'_, PyAny>) -> PyResult<Option<f64>> {
        let c = extract_candle(candle)?;
        Ok(self.inner.update(c))
    }
    /// Batch over high/low/volume numpy columns.
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        high: PyReadonlyArray1<'py, f64>,
        low: PyReadonlyArray1<'py, f64>,
        volume: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let h = high
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let l = low
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let v = volume
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        if h.len() != l.len() || l.len() != v.len() {
            return Err(PyValueError::new_err(
                "high, low, volume must be equal length",
            ));
        }
        let mut out = Vec::with_capacity(h.len());
        for i in 0..h.len() {
            let candle = wc::Candle::new(l[i], h[i], l[i], l[i], v[i], 0).map_err(map_err)?;
            out.push(self.inner.update(candle).unwrap_or(f64::NAN));
        }
        Ok(out.into_pyarray(py))
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        "MarketFacilitationIndex()".to_string()
    }
}

// ============================== Ease of Movement ==============================

#[pyclass(
    name = "EaseOfMovement",
    module = "wickra._wickra",
    skip_from_py_object
)]
#[derive(Clone)]
struct PyEaseOfMovement {
    inner: wc::EaseOfMovement,
}

#[pymethods]
impl PyEaseOfMovement {
    #[new]
    #[pyo3(signature = (period=14, divisor=100_000_000.0))]
    fn new(period: usize, divisor: f64) -> PyResult<Self> {
        Ok(Self {
            inner: wc::EaseOfMovement::with_divisor(period, divisor).map_err(map_err)?,
        })
    }
    fn update(&mut self, candle: &Bound<'_, PyAny>) -> PyResult<Option<f64>> {
        let c = extract_candle(candle)?;
        Ok(self.inner.update(c))
    }
    /// Batch over numpy columns: high, low, volume (all equal length).
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        high: PyReadonlyArray1<'py, f64>,
        low: PyReadonlyArray1<'py, f64>,
        volume: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let h = high
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let l = low
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let v = volume
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        if h.len() != l.len() || l.len() != v.len() {
            return Err(PyValueError::new_err(
                "high, low, volume must be equal length",
            ));
        }
        let mut out = Vec::with_capacity(h.len());
        for i in 0..h.len() {
            let candle = wc::Candle::new(l[i], h[i], l[i], l[i], v[i], 0).map_err(map_err)?;
            out.push(self.inner.update(candle).unwrap_or(f64::NAN));
        }
        Ok(out.into_pyarray(py))
    }
    #[getter]
    fn period(&self) -> usize {
        self.inner.period()
    }
    #[getter]
    fn divisor(&self) -> f64 {
        self.inner.divisor()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        format!(
            "EaseOfMovement(period={}, divisor={})",
            self.inner.period(),
            self.inner.divisor()
        )
    }
}

// ============================== SuperTrend ==============================

#[pyclass(name = "SuperTrend", module = "wickra._wickra", skip_from_py_object)]
#[derive(Clone)]
struct PySuperTrend {
    inner: wc::SuperTrend,
}

#[pymethods]
impl PySuperTrend {
    #[new]
    #[pyo3(signature = (atr_period=10, multiplier=3.0))]
    fn new(atr_period: usize, multiplier: f64) -> PyResult<Self> {
        Ok(Self {
            inner: wc::SuperTrend::new(atr_period, multiplier).map_err(map_err)?,
        })
    }
    fn update(&mut self, candle: &Bound<'_, PyAny>) -> PyResult<Option<(f64, f64)>> {
        let c = extract_candle(candle)?;
        Ok(self.inner.update(c).map(|o| (o.value, o.direction)))
    }
    /// Batch over numpy columns high, low, close. Returns shape `(n, 2)` with
    /// columns `[value, direction]`; warmup rows are `NaN`.
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        high: PyReadonlyArray1<'py, f64>,
        low: PyReadonlyArray1<'py, f64>,
        close: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray2<f64>>> {
        let h = high
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let l = low
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let c = close
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        if h.len() != l.len() || l.len() != c.len() {
            return Err(PyValueError::new_err(
                "high, low, close must be equal length",
            ));
        }
        let n = h.len();
        let mut out = vec![f64::NAN; n * 2];
        for i in 0..n {
            let candle = wc::Candle::new(c[i], h[i], l[i], c[i], 0.0, 0).map_err(map_err)?;
            if let Some(o) = self.inner.update(candle) {
                out[i * 2] = o.value;
                out[i * 2 + 1] = o.direction;
            }
        }
        Ok(numpy::ndarray::Array2::from_shape_vec((n, 2), out)
            .expect("shape consistent")
            .into_pyarray(py))
    }
    #[getter]
    fn params(&self) -> (usize, f64) {
        self.inner.params()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        let (atr_period, multiplier) = self.inner.params();
        format!("SuperTrend(atr_period={atr_period}, multiplier={multiplier})")
    }
}

// ============================== Chandelier Exit ==============================

#[pyclass(
    name = "ChandelierExit",
    module = "wickra._wickra",
    skip_from_py_object
)]
#[derive(Clone)]
struct PyChandelierExit {
    inner: wc::ChandelierExit,
}

#[pymethods]
impl PyChandelierExit {
    #[new]
    #[pyo3(signature = (period=22, multiplier=3.0))]
    fn new(period: usize, multiplier: f64) -> PyResult<Self> {
        Ok(Self {
            inner: wc::ChandelierExit::new(period, multiplier).map_err(map_err)?,
        })
    }
    fn update(&mut self, candle: &Bound<'_, PyAny>) -> PyResult<Option<(f64, f64)>> {
        let c = extract_candle(candle)?;
        Ok(self.inner.update(c).map(|o| (o.long_stop, o.short_stop)))
    }
    /// Batch over numpy columns high, low, close. Returns shape `(n, 2)` with
    /// columns `[long_stop, short_stop]`; warmup rows are `NaN`.
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        high: PyReadonlyArray1<'py, f64>,
        low: PyReadonlyArray1<'py, f64>,
        close: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray2<f64>>> {
        let h = high
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let l = low
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let c = close
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        if h.len() != l.len() || l.len() != c.len() {
            return Err(PyValueError::new_err(
                "high, low, close must be equal length",
            ));
        }
        let n = h.len();
        let mut out = vec![f64::NAN; n * 2];
        for i in 0..n {
            let candle = wc::Candle::new(c[i], h[i], l[i], c[i], 0.0, 0).map_err(map_err)?;
            if let Some(o) = self.inner.update(candle) {
                out[i * 2] = o.long_stop;
                out[i * 2 + 1] = o.short_stop;
            }
        }
        Ok(numpy::ndarray::Array2::from_shape_vec((n, 2), out)
            .expect("shape consistent")
            .into_pyarray(py))
    }
    #[getter]
    fn params(&self) -> (usize, f64) {
        self.inner.params()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        let (period, multiplier) = self.inner.params();
        format!("ChandelierExit(period={period}, multiplier={multiplier})")
    }
}

// ============================== Chande Kroll Stop ==============================

#[pyclass(
    name = "ChandeKrollStop",
    module = "wickra._wickra",
    skip_from_py_object
)]
#[derive(Clone)]
struct PyChandeKrollStop {
    inner: wc::ChandeKrollStop,
}

#[pymethods]
impl PyChandeKrollStop {
    #[new]
    #[pyo3(signature = (atr_period=10, atr_multiplier=1.0, stop_period=9))]
    fn new(atr_period: usize, atr_multiplier: f64, stop_period: usize) -> PyResult<Self> {
        Ok(Self {
            inner: wc::ChandeKrollStop::new(atr_period, atr_multiplier, stop_period)
                .map_err(map_err)?,
        })
    }
    fn update(&mut self, candle: &Bound<'_, PyAny>) -> PyResult<Option<(f64, f64)>> {
        let c = extract_candle(candle)?;
        Ok(self.inner.update(c).map(|o| (o.stop_long, o.stop_short)))
    }
    /// Batch over numpy columns high, low, close. Returns shape `(n, 2)` with
    /// columns `[stop_long, stop_short]`; warmup rows are `NaN`.
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        high: PyReadonlyArray1<'py, f64>,
        low: PyReadonlyArray1<'py, f64>,
        close: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray2<f64>>> {
        let h = high
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let l = low
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let c = close
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        if h.len() != l.len() || l.len() != c.len() {
            return Err(PyValueError::new_err(
                "high, low, close must be equal length",
            ));
        }
        let n = h.len();
        let mut out = vec![f64::NAN; n * 2];
        for i in 0..n {
            let candle = wc::Candle::new(c[i], h[i], l[i], c[i], 0.0, 0).map_err(map_err)?;
            if let Some(o) = self.inner.update(candle) {
                out[i * 2] = o.stop_long;
                out[i * 2 + 1] = o.stop_short;
            }
        }
        Ok(numpy::ndarray::Array2::from_shape_vec((n, 2), out)
            .expect("shape consistent")
            .into_pyarray(py))
    }
    #[getter]
    fn params(&self) -> (usize, f64, usize) {
        self.inner.params()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        let (atr_period, atr_multiplier, stop_period) = self.inner.params();
        format!(
            "ChandeKrollStop(atr_period={atr_period}, atr_multiplier={atr_multiplier}, stop_period={stop_period})"
        )
    }
}

// ============================== ATR Trailing Stop ==============================

#[pyclass(
    name = "AtrTrailingStop",
    module = "wickra._wickra",
    skip_from_py_object
)]
#[derive(Clone)]
struct PyAtrTrailingStop {
    inner: wc::AtrTrailingStop,
}

#[pymethods]
impl PyAtrTrailingStop {
    #[new]
    #[pyo3(signature = (atr_period=14, multiplier=3.0))]
    fn new(atr_period: usize, multiplier: f64) -> PyResult<Self> {
        Ok(Self {
            inner: wc::AtrTrailingStop::new(atr_period, multiplier).map_err(map_err)?,
        })
    }
    fn update(&mut self, candle: &Bound<'_, PyAny>) -> PyResult<Option<f64>> {
        let c = extract_candle(candle)?;
        Ok(self.inner.update(c))
    }
    /// Batch over numpy columns high, low, close (all equal length).
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        high: PyReadonlyArray1<'py, f64>,
        low: PyReadonlyArray1<'py, f64>,
        close: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let h = high
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let l = low
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let c = close
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        if h.len() != l.len() || l.len() != c.len() {
            return Err(PyValueError::new_err(
                "high, low, close must be equal length",
            ));
        }
        let mut out = Vec::with_capacity(h.len());
        for i in 0..h.len() {
            let candle = wc::Candle::new(c[i], h[i], l[i], c[i], 0.0, 0).map_err(map_err)?;
            out.push(self.inner.update(candle).unwrap_or(f64::NAN));
        }
        Ok(out.into_pyarray(py))
    }
    #[getter]
    fn params(&self) -> (usize, f64) {
        self.inner.params()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        let (atr_period, multiplier) = self.inner.params();
        format!("AtrTrailingStop(atr_period={atr_period}, multiplier={multiplier})")
    }
}

// ============================== HiLo Activator ==============================

#[pyclass(name = "HiLoActivator", module = "wickra._wickra", skip_from_py_object)]
#[derive(Clone)]
struct PyHiLoActivator {
    inner: wc::HiLoActivator,
}

#[pymethods]
impl PyHiLoActivator {
    #[new]
    #[pyo3(signature = (period=3))]
    fn new(period: usize) -> PyResult<Self> {
        Ok(Self {
            inner: wc::HiLoActivator::new(period).map_err(map_err)?,
        })
    }
    fn update(&mut self, candle: &Bound<'_, PyAny>) -> PyResult<Option<f64>> {
        let c = extract_candle(candle)?;
        Ok(self.inner.update(c))
    }
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        high: PyReadonlyArray1<'py, f64>,
        low: PyReadonlyArray1<'py, f64>,
        close: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let h = high
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let l = low
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let c = close
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        if h.len() != l.len() || l.len() != c.len() {
            return Err(PyValueError::new_err(
                "high, low, close must be equal length",
            ));
        }
        let mut out = Vec::with_capacity(h.len());
        for i in 0..h.len() {
            let candle = wc::Candle::new(c[i], h[i], l[i], c[i], 0.0, 0).map_err(map_err)?;
            out.push(self.inner.update(candle).unwrap_or(f64::NAN));
        }
        Ok(out.into_pyarray(py))
    }
    #[getter]
    fn period(&self) -> usize {
        self.inner.period()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        format!("HiLoActivator(period={})", self.inner.period())
    }
}

// ============================== Volty Stop ==============================

#[pyclass(name = "VoltyStop", module = "wickra._wickra", skip_from_py_object)]
#[derive(Clone)]
struct PyVoltyStop {
    inner: wc::VoltyStop,
}

#[pymethods]
impl PyVoltyStop {
    #[new]
    #[pyo3(signature = (atr_period=14, multiplier=2.0))]
    fn new(atr_period: usize, multiplier: f64) -> PyResult<Self> {
        Ok(Self {
            inner: wc::VoltyStop::new(atr_period, multiplier).map_err(map_err)?,
        })
    }
    fn update(&mut self, candle: &Bound<'_, PyAny>) -> PyResult<Option<f64>> {
        let c = extract_candle(candle)?;
        Ok(self.inner.update(c))
    }
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        high: PyReadonlyArray1<'py, f64>,
        low: PyReadonlyArray1<'py, f64>,
        close: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let h = high
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let l = low
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let c = close
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        if h.len() != l.len() || l.len() != c.len() {
            return Err(PyValueError::new_err(
                "high, low, close must be equal length",
            ));
        }
        let mut out = Vec::with_capacity(h.len());
        for i in 0..h.len() {
            let candle = wc::Candle::new(c[i], h[i], l[i], c[i], 0.0, 0).map_err(map_err)?;
            out.push(self.inner.update(candle).unwrap_or(f64::NAN));
        }
        Ok(out.into_pyarray(py))
    }
    #[getter]
    fn params(&self) -> (usize, f64) {
        self.inner.params()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        let (p, m) = self.inner.params();
        format!("VoltyStop(atr_period={p}, multiplier={m})")
    }
}

// ============================== Yo-Yo Exit ==============================

#[pyclass(name = "YoyoExit", module = "wickra._wickra", skip_from_py_object)]
#[derive(Clone)]
struct PyYoyoExit {
    inner: wc::YoyoExit,
}

#[pymethods]
impl PyYoyoExit {
    #[new]
    #[pyo3(signature = (atr_period=14, multiplier=2.0))]
    fn new(atr_period: usize, multiplier: f64) -> PyResult<Self> {
        Ok(Self {
            inner: wc::YoyoExit::new(atr_period, multiplier).map_err(map_err)?,
        })
    }
    fn update(&mut self, candle: &Bound<'_, PyAny>) -> PyResult<Option<f64>> {
        let c = extract_candle(candle)?;
        Ok(self.inner.update(c))
    }
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        high: PyReadonlyArray1<'py, f64>,
        low: PyReadonlyArray1<'py, f64>,
        close: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let h = high
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let l = low
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let c = close
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        if h.len() != l.len() || l.len() != c.len() {
            return Err(PyValueError::new_err(
                "high, low, close must be equal length",
            ));
        }
        let mut out = Vec::with_capacity(h.len());
        for i in 0..h.len() {
            let candle = wc::Candle::new(c[i], h[i], l[i], c[i], 0.0, 0).map_err(map_err)?;
            out.push(self.inner.update(candle).unwrap_or(f64::NAN));
        }
        Ok(out.into_pyarray(py))
    }
    #[getter]
    fn params(&self) -> (usize, f64) {
        self.inner.params()
    }
    #[getter]
    fn in_trade(&self) -> bool {
        self.inner.in_trade()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        let (p, m) = self.inner.params();
        format!("YoyoExit(atr_period={p}, multiplier={m})")
    }
}

// ============================== Donchian Stop ==============================

#[pyclass(name = "DonchianStop", module = "wickra._wickra", skip_from_py_object)]
#[derive(Clone)]
struct PyDonchianStop {
    inner: wc::DonchianStop,
}

#[pymethods]
impl PyDonchianStop {
    #[new]
    #[pyo3(signature = (period=10))]
    fn new(period: usize) -> PyResult<Self> {
        Ok(Self {
            inner: wc::DonchianStop::new(period).map_err(map_err)?,
        })
    }
    fn update(&mut self, candle: &Bound<'_, PyAny>) -> PyResult<Option<(f64, f64)>> {
        let c = extract_candle(candle)?;
        Ok(self.inner.update(c).map(|o| (o.stop_long, o.stop_short)))
    }
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        high: PyReadonlyArray1<'py, f64>,
        low: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray2<f64>>> {
        let h = high
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let l = low
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        if h.len() != l.len() {
            return Err(PyValueError::new_err("high and low must be equal length"));
        }
        let n = h.len();
        let mut out = vec![f64::NAN; n * 2];
        for i in 0..n {
            let candle = wc::Candle::new(l[i], h[i], l[i], l[i], 0.0, 0).map_err(map_err)?;
            if let Some(o) = self.inner.update(candle) {
                out[i * 2] = o.stop_long;
                out[i * 2 + 1] = o.stop_short;
            }
        }
        Ok(numpy::ndarray::Array2::from_shape_vec((n, 2), out)
            .expect("shape consistent")
            .into_pyarray(py))
    }
    #[getter]
    fn period(&self) -> usize {
        self.inner.period()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        format!("DonchianStop(period={})", self.inner.period())
    }
}

// ============================== Percentage Trailing Stop ==============================

#[pyclass(
    name = "PercentageTrailingStop",
    module = "wickra._wickra",
    skip_from_py_object
)]
#[derive(Clone)]
struct PyPercentageTrailingStop {
    inner: wc::PercentageTrailingStop,
}

#[pymethods]
impl PyPercentageTrailingStop {
    #[new]
    #[pyo3(signature = (percent=5.0))]
    fn new(percent: f64) -> PyResult<Self> {
        Ok(Self {
            inner: wc::PercentageTrailingStop::new(percent).map_err(map_err)?,
        })
    }
    fn update(&mut self, value: f64) -> Option<f64> {
        self.inner.update(value)
    }
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        prices: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let slice = prices
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        Ok(flatten(self.inner.batch(slice)).into_pyarray(py))
    }
    #[getter]
    fn percent(&self) -> f64 {
        self.inner.percent()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        format!("PercentageTrailingStop(percent={})", self.inner.percent())
    }
}

// ============================== Step Trailing Stop ==============================

#[pyclass(
    name = "StepTrailingStop",
    module = "wickra._wickra",
    skip_from_py_object
)]
#[derive(Clone)]
struct PyStepTrailingStop {
    inner: wc::StepTrailingStop,
}

#[pymethods]
impl PyStepTrailingStop {
    #[new]
    #[pyo3(signature = (step_size=1.0))]
    fn new(step_size: f64) -> PyResult<Self> {
        Ok(Self {
            inner: wc::StepTrailingStop::new(step_size).map_err(map_err)?,
        })
    }
    fn update(&mut self, value: f64) -> Option<f64> {
        self.inner.update(value)
    }
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        prices: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let slice = prices
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        Ok(flatten(self.inner.batch(slice)).into_pyarray(py))
    }
    #[getter]
    fn step_size(&self) -> f64 {
        self.inner.step_size()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        format!("StepTrailingStop(step_size={})", self.inner.step_size())
    }
}

// ============================== Renko Trailing Stop ==============================

#[pyclass(
    name = "RenkoTrailingStop",
    module = "wickra._wickra",
    skip_from_py_object
)]
#[derive(Clone)]
struct PyRenkoTrailingStop {
    inner: wc::RenkoTrailingStop,
}

#[pymethods]
impl PyRenkoTrailingStop {
    #[new]
    #[pyo3(signature = (block_size=1.0))]
    fn new(block_size: f64) -> PyResult<Self> {
        Ok(Self {
            inner: wc::RenkoTrailingStop::new(block_size).map_err(map_err)?,
        })
    }
    fn update(&mut self, value: f64) -> Option<f64> {
        self.inner.update(value)
    }
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        prices: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let slice = prices
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        Ok(flatten(self.inner.batch(slice)).into_pyarray(py))
    }
    #[getter]
    fn block_size(&self) -> f64 {
        self.inner.block_size()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        format!("RenkoTrailingStop(block_size={})", self.inner.block_size())
    }
}

// ============================== Typical Price ==============================

#[pyclass(name = "TypicalPrice", module = "wickra._wickra", skip_from_py_object)]
#[derive(Clone)]
struct PyTypicalPrice {
    inner: wc::TypicalPrice,
}

#[pymethods]
impl PyTypicalPrice {
    #[new]
    fn new() -> Self {
        Self {
            inner: wc::TypicalPrice::new(),
        }
    }
    fn update(&mut self, candle: &Bound<'_, PyAny>) -> PyResult<Option<f64>> {
        let c = extract_candle(candle)?;
        Ok(self.inner.update(c))
    }
    /// Batch over numpy columns high, low, close (all equal length).
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        high: PyReadonlyArray1<'py, f64>,
        low: PyReadonlyArray1<'py, f64>,
        close: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let h = high
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let l = low
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let c = close
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        if h.len() != l.len() || l.len() != c.len() {
            return Err(PyValueError::new_err(
                "high, low, close must be equal length",
            ));
        }
        let mut out = Vec::with_capacity(h.len());
        for i in 0..h.len() {
            let candle = wc::Candle::new(c[i], h[i], l[i], c[i], 0.0, 0).map_err(map_err)?;
            out.push(self.inner.update(candle).unwrap_or(f64::NAN));
        }
        Ok(out.into_pyarray(py))
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        "TypicalPrice()".to_string()
    }
}

// ============================== Median Price ==============================

#[pyclass(name = "MedianPrice", module = "wickra._wickra", skip_from_py_object)]
#[derive(Clone)]
struct PyMedianPrice {
    inner: wc::MedianPrice,
}

#[pymethods]
impl PyMedianPrice {
    #[new]
    fn new() -> Self {
        Self {
            inner: wc::MedianPrice::new(),
        }
    }
    fn update(&mut self, candle: &Bound<'_, PyAny>) -> PyResult<Option<f64>> {
        let c = extract_candle(candle)?;
        Ok(self.inner.update(c))
    }
    /// Batch over numpy columns high, low (both equal length).
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        high: PyReadonlyArray1<'py, f64>,
        low: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let h = high
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let l = low
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        if h.len() != l.len() {
            return Err(PyValueError::new_err("high and low must be equal length"));
        }
        let mut out = Vec::with_capacity(h.len());
        for i in 0..h.len() {
            let candle = wc::Candle::new(l[i], h[i], l[i], l[i], 0.0, 0).map_err(map_err)?;
            out.push(self.inner.update(candle).unwrap_or(f64::NAN));
        }
        Ok(out.into_pyarray(py))
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        "MedianPrice()".to_string()
    }
}

// ============================== Weighted Close ==============================

#[pyclass(name = "WeightedClose", module = "wickra._wickra", skip_from_py_object)]
#[derive(Clone)]
struct PyWeightedClose {
    inner: wc::WeightedClose,
}

#[pymethods]
impl PyWeightedClose {
    #[new]
    fn new() -> Self {
        Self {
            inner: wc::WeightedClose::new(),
        }
    }
    fn update(&mut self, candle: &Bound<'_, PyAny>) -> PyResult<Option<f64>> {
        let c = extract_candle(candle)?;
        Ok(self.inner.update(c))
    }
    /// Batch over numpy columns high, low, close (all equal length).
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        high: PyReadonlyArray1<'py, f64>,
        low: PyReadonlyArray1<'py, f64>,
        close: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let h = high
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let l = low
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let c = close
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        if h.len() != l.len() || l.len() != c.len() {
            return Err(PyValueError::new_err(
                "high, low, close must be equal length",
            ));
        }
        let mut out = Vec::with_capacity(h.len());
        for i in 0..h.len() {
            let candle = wc::Candle::new(c[i], h[i], l[i], c[i], 0.0, 0).map_err(map_err)?;
            out.push(self.inner.update(candle).unwrap_or(f64::NAN));
        }
        Ok(out.into_pyarray(py))
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        "WeightedClose()".to_string()
    }
}

// ============================== Linear Regression ==============================

#[pyclass(
    name = "LinearRegression",
    module = "wickra._wickra",
    skip_from_py_object
)]
#[derive(Clone)]
struct PyLinearRegression {
    inner: wc::LinearRegression,
}

#[pymethods]
impl PyLinearRegression {
    #[new]
    #[pyo3(signature = (period=14))]
    fn new(period: usize) -> PyResult<Self> {
        Ok(Self {
            inner: wc::LinearRegression::new(period).map_err(map_err)?,
        })
    }
    fn update(&mut self, value: f64) -> Option<f64> {
        self.inner.update(value)
    }
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        prices: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let slice = prices
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        Ok(flatten(self.inner.batch(slice)).into_pyarray(py))
    }
    #[getter]
    fn period(&self) -> usize {
        self.inner.period()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        format!("LinearRegression(period={})", self.inner.period())
    }
}

// ============================== Linear Regression Slope ==============================

#[pyclass(name = "LinRegSlope", module = "wickra._wickra", skip_from_py_object)]
#[derive(Clone)]
struct PyLinRegSlope {
    inner: wc::LinRegSlope,
}

#[pymethods]
impl PyLinRegSlope {
    #[new]
    #[pyo3(signature = (period=14))]
    fn new(period: usize) -> PyResult<Self> {
        Ok(Self {
            inner: wc::LinRegSlope::new(period).map_err(map_err)?,
        })
    }
    fn update(&mut self, value: f64) -> Option<f64> {
        self.inner.update(value)
    }
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        prices: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let slice = prices
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        Ok(flatten(self.inner.batch(slice)).into_pyarray(py))
    }
    #[getter]
    fn period(&self) -> usize {
        self.inner.period()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        format!("LinRegSlope(period={})", self.inner.period())
    }
}

// ============================== Accelerator Oscillator ==============================

#[pyclass(
    name = "AcceleratorOscillator",
    module = "wickra._wickra",
    skip_from_py_object
)]
#[derive(Clone)]
struct PyAcceleratorOscillator {
    inner: wc::AcceleratorOscillator,
}

#[pymethods]
impl PyAcceleratorOscillator {
    #[new]
    #[pyo3(signature = (ao_fast=5, ao_slow=34, signal_period=5))]
    fn new(ao_fast: usize, ao_slow: usize, signal_period: usize) -> PyResult<Self> {
        Ok(Self {
            inner: wc::AcceleratorOscillator::new(ao_fast, ao_slow, signal_period)
                .map_err(map_err)?,
        })
    }
    fn update(&mut self, candle: &Bound<'_, PyAny>) -> PyResult<Option<f64>> {
        let c = extract_candle(candle)?;
        Ok(self.inner.update(c))
    }
    /// Batch over numpy columns high, low (both equal length).
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        high: PyReadonlyArray1<'py, f64>,
        low: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let h = high
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let l = low
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        if h.len() != l.len() {
            return Err(PyValueError::new_err("high and low must be equal length"));
        }
        let mut out = Vec::with_capacity(h.len());
        for i in 0..h.len() {
            let candle = wc::Candle::new(l[i], h[i], l[i], l[i], 0.0, 0).map_err(map_err)?;
            out.push(self.inner.update(candle).unwrap_or(f64::NAN));
        }
        Ok(out.into_pyarray(py))
    }
    #[getter]
    fn params(&self) -> (usize, usize, usize) {
        self.inner.params()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        let (f, s, sig) = self.inner.params();
        format!("AcceleratorOscillator(ao_fast={f}, ao_slow={s}, signal_period={sig})")
    }
}

// ============================== Balance of Power ==============================

#[pyclass(
    name = "BalanceOfPower",
    module = "wickra._wickra",
    skip_from_py_object
)]
#[derive(Clone)]
struct PyBalanceOfPower {
    inner: wc::BalanceOfPower,
}

#[pymethods]
impl PyBalanceOfPower {
    #[new]
    fn new() -> Self {
        Self {
            inner: wc::BalanceOfPower::new(),
        }
    }
    fn update(&mut self, candle: &Bound<'_, PyAny>) -> PyResult<Option<f64>> {
        let c = extract_candle(candle)?;
        Ok(self.inner.update(c))
    }
    /// Batch over numpy columns open, high, low, close (all equal length).
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        open: PyReadonlyArray1<'py, f64>,
        high: PyReadonlyArray1<'py, f64>,
        low: PyReadonlyArray1<'py, f64>,
        close: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let o = open
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let h = high
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let l = low
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let c = close
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        if o.len() != h.len() || h.len() != l.len() || l.len() != c.len() {
            return Err(PyValueError::new_err(
                "open, high, low, close must be equal length",
            ));
        }
        let mut out = Vec::with_capacity(o.len());
        for i in 0..o.len() {
            let candle = wc::Candle::new(o[i], h[i], l[i], c[i], 0.0, 0).map_err(map_err)?;
            out.push(self.inner.update(candle).unwrap_or(f64::NAN));
        }
        Ok(out.into_pyarray(py))
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        "BalanceOfPower()".to_string()
    }
}

// ============================== Choppiness Index ==============================

#[pyclass(
    name = "ChoppinessIndex",
    module = "wickra._wickra",
    skip_from_py_object
)]
#[derive(Clone)]
struct PyChoppinessIndex {
    inner: wc::ChoppinessIndex,
}

#[pymethods]
impl PyChoppinessIndex {
    #[new]
    #[pyo3(signature = (period=14))]
    fn new(period: usize) -> PyResult<Self> {
        Ok(Self {
            inner: wc::ChoppinessIndex::new(period).map_err(map_err)?,
        })
    }
    fn update(&mut self, candle: &Bound<'_, PyAny>) -> PyResult<Option<f64>> {
        let c = extract_candle(candle)?;
        Ok(self.inner.update(c))
    }
    /// Batch over numpy columns high, low, close (all equal length).
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        high: PyReadonlyArray1<'py, f64>,
        low: PyReadonlyArray1<'py, f64>,
        close: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let h = high
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let l = low
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let c = close
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        if h.len() != l.len() || l.len() != c.len() {
            return Err(PyValueError::new_err(
                "high, low, close must be equal length",
            ));
        }
        let mut out = Vec::with_capacity(h.len());
        for i in 0..h.len() {
            let candle = wc::Candle::new(c[i], h[i], l[i], c[i], 0.0, 0).map_err(map_err)?;
            out.push(self.inner.update(candle).unwrap_or(f64::NAN));
        }
        Ok(out.into_pyarray(py))
    }
    #[getter]
    fn period(&self) -> usize {
        self.inner.period()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        format!("ChoppinessIndex(period={})", self.inner.period())
    }
}

// ============================== Vertical Horizontal Filter ==============================

#[pyclass(
    name = "VerticalHorizontalFilter",
    module = "wickra._wickra",
    skip_from_py_object
)]
#[derive(Clone)]
struct PyVerticalHorizontalFilter {
    inner: wc::VerticalHorizontalFilter,
}

#[pymethods]
impl PyVerticalHorizontalFilter {
    #[new]
    #[pyo3(signature = (period=28))]
    fn new(period: usize) -> PyResult<Self> {
        Ok(Self {
            inner: wc::VerticalHorizontalFilter::new(period).map_err(map_err)?,
        })
    }
    fn update(&mut self, value: f64) -> Option<f64> {
        self.inner.update(value)
    }
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        prices: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let slice = prices
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        Ok(flatten(self.inner.batch(slice)).into_pyarray(py))
    }
    #[getter]
    fn period(&self) -> usize {
        self.inner.period()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        format!("VerticalHorizontalFilter(period={})", self.inner.period())
    }
}

// ============================== True Range ==============================

#[pyclass(name = "TrueRange", module = "wickra._wickra", skip_from_py_object)]
#[derive(Clone)]
struct PyTrueRange {
    inner: wc::TrueRange,
}

#[pymethods]
impl PyTrueRange {
    #[new]
    fn new() -> Self {
        Self {
            inner: wc::TrueRange::new(),
        }
    }
    fn update(&mut self, candle: &Bound<'_, PyAny>) -> PyResult<Option<f64>> {
        let c = extract_candle(candle)?;
        Ok(self.inner.update(c))
    }
    /// Batch over numpy columns high, low, close (all equal length).
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        high: PyReadonlyArray1<'py, f64>,
        low: PyReadonlyArray1<'py, f64>,
        close: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let h = high
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let l = low
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let c = close
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        if h.len() != l.len() || l.len() != c.len() {
            return Err(PyValueError::new_err(
                "high, low, close must be equal length",
            ));
        }
        let mut out = Vec::with_capacity(h.len());
        for i in 0..h.len() {
            let candle = wc::Candle::new(c[i], h[i], l[i], c[i], 0.0, 0).map_err(map_err)?;
            out.push(self.inner.update(candle).unwrap_or(f64::NAN));
        }
        Ok(out.into_pyarray(py))
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        "TrueRange()".to_string()
    }
}

// ============================== Chaikin Volatility ==============================

#[pyclass(
    name = "ChaikinVolatility",
    module = "wickra._wickra",
    skip_from_py_object
)]
#[derive(Clone)]
struct PyChaikinVolatility {
    inner: wc::ChaikinVolatility,
}

#[pymethods]
impl PyChaikinVolatility {
    #[new]
    #[pyo3(signature = (ema_period=10, roc_period=10))]
    fn new(ema_period: usize, roc_period: usize) -> PyResult<Self> {
        Ok(Self {
            inner: wc::ChaikinVolatility::new(ema_period, roc_period).map_err(map_err)?,
        })
    }
    fn update(&mut self, candle: &Bound<'_, PyAny>) -> PyResult<Option<f64>> {
        let c = extract_candle(candle)?;
        Ok(self.inner.update(c))
    }
    /// Batch over numpy columns high, low (both equal length).
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        high: PyReadonlyArray1<'py, f64>,
        low: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let h = high
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let l = low
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        if h.len() != l.len() {
            return Err(PyValueError::new_err("high and low must be equal length"));
        }
        let mut out = Vec::with_capacity(h.len());
        for i in 0..h.len() {
            let candle = wc::Candle::new(l[i], h[i], l[i], l[i], 0.0, 0).map_err(map_err)?;
            out.push(self.inner.update(candle).unwrap_or(f64::NAN));
        }
        Ok(out.into_pyarray(py))
    }
    #[getter]
    fn periods(&self) -> (usize, usize) {
        self.inner.periods()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        let (ema, roc) = self.inner.periods();
        format!("ChaikinVolatility(ema_period={ema}, roc_period={roc})")
    }
}

// ============================== Z-Score ==============================

#[pyclass(name = "ZScore", module = "wickra._wickra", skip_from_py_object)]
#[derive(Clone)]
struct PyZScore {
    inner: wc::ZScore,
}

#[pymethods]
impl PyZScore {
    #[new]
    #[pyo3(signature = (period=20))]
    fn new(period: usize) -> PyResult<Self> {
        Ok(Self {
            inner: wc::ZScore::new(period).map_err(map_err)?,
        })
    }
    fn update(&mut self, value: f64) -> Option<f64> {
        self.inner.update(value)
    }
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        prices: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let slice = prices
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        Ok(flatten(self.inner.batch(slice)).into_pyarray(py))
    }
    #[getter]
    fn period(&self) -> usize {
        self.inner.period()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        format!("ZScore(period={})", self.inner.period())
    }
}

// ============================== Linear Regression Angle ==============================

#[pyclass(name = "LinRegAngle", module = "wickra._wickra", skip_from_py_object)]
#[derive(Clone)]
struct PyLinRegAngle {
    inner: wc::LinRegAngle,
}

#[pymethods]
impl PyLinRegAngle {
    #[new]
    #[pyo3(signature = (period=14))]
    fn new(period: usize) -> PyResult<Self> {
        Ok(Self {
            inner: wc::LinRegAngle::new(period).map_err(map_err)?,
        })
    }
    fn update(&mut self, value: f64) -> Option<f64> {
        self.inner.update(value)
    }
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        prices: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let slice = prices
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        Ok(flatten(self.inner.batch(slice)).into_pyarray(py))
    }
    #[getter]
    fn period(&self) -> usize {
        self.inner.period()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        format!("LinRegAngle(period={})", self.inner.period())
    }
}

#[pyclass(
    name = "YangZhangVolatility",
    module = "wickra._wickra",
    skip_from_py_object
)]
#[derive(Clone)]
struct PyYangZhangVolatility {
    inner: wc::YangZhangVolatility,
}

#[pymethods]
impl PyYangZhangVolatility {
    #[new]
    #[pyo3(signature = (period=20, trading_periods=252))]
    fn new(period: usize, trading_periods: usize) -> PyResult<Self> {
        Ok(Self {
            inner: wc::YangZhangVolatility::new(period, trading_periods).map_err(map_err)?,
        })
    }
    fn update(&mut self, candle: &Bound<'_, PyAny>) -> PyResult<Option<f64>> {
        let c = extract_candle(candle)?;
        Ok(self.inner.update(c))
    }
    /// Batch over numpy columns open, high, low, close (all equal length).
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        open: PyReadonlyArray1<'py, f64>,
        high: PyReadonlyArray1<'py, f64>,
        low: PyReadonlyArray1<'py, f64>,
        close: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let o = open
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let h = high
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let l = low
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let cl = close
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        if o.len() != h.len() || h.len() != l.len() || l.len() != cl.len() {
            return Err(PyValueError::new_err(
                "open, high, low, close must be equal length",
            ));
        }
        let mut out = Vec::with_capacity(o.len());
        for i in 0..o.len() {
            let candle = wc::Candle::new(o[i], h[i], l[i], cl[i], 0.0, 0).map_err(map_err)?;
            out.push(self.inner.update(candle).unwrap_or(f64::NAN));
        }
        Ok(out.into_pyarray(py))
    }
    #[getter]
    fn periods(&self) -> (usize, usize) {
        self.inner.periods()
    }
    #[getter]
    fn k(&self) -> f64 {
        self.inner.k()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        let (p, t) = self.inner.periods();
        format!("YangZhangVolatility(period={p}, trading_periods={t})")
    }
}

// ============================== Rogers-Satchell Volatility ==============================

#[pyclass(
    name = "RogersSatchellVolatility",
    module = "wickra._wickra",
    skip_from_py_object
)]
#[derive(Clone)]
struct PyRogersSatchellVolatility {
    inner: wc::RogersSatchellVolatility,
}

#[pymethods]
impl PyRogersSatchellVolatility {
    #[new]
    #[pyo3(signature = (period=20, trading_periods=252))]
    fn new(period: usize, trading_periods: usize) -> PyResult<Self> {
        Ok(Self {
            inner: wc::RogersSatchellVolatility::new(period, trading_periods).map_err(map_err)?,
        })
    }
    fn update(&mut self, candle: &Bound<'_, PyAny>) -> PyResult<Option<f64>> {
        let c = extract_candle(candle)?;
        Ok(self.inner.update(c))
    }
    /// Batch over numpy columns open, high, low, close (all equal length).
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        open: PyReadonlyArray1<'py, f64>,
        high: PyReadonlyArray1<'py, f64>,
        low: PyReadonlyArray1<'py, f64>,
        close: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let o = open
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let h = high
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let l = low
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let cl = close
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        if o.len() != h.len() || h.len() != l.len() || l.len() != cl.len() {
            return Err(PyValueError::new_err(
                "open, high, low, close must be equal length",
            ));
        }
        let mut out = Vec::with_capacity(o.len());
        for i in 0..o.len() {
            let candle = wc::Candle::new(o[i], h[i], l[i], cl[i], 0.0, 0).map_err(map_err)?;
            out.push(self.inner.update(candle).unwrap_or(f64::NAN));
        }
        Ok(out.into_pyarray(py))
    }
    #[getter]
    fn periods(&self) -> (usize, usize) {
        self.inner.periods()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        let (p, t) = self.inner.periods();
        format!("RogersSatchellVolatility(period={p}, trading_periods={t})")
    }
}

// ============================== Garman-Klass Volatility ==============================

#[pyclass(
    name = "GarmanKlassVolatility",
    module = "wickra._wickra",
    skip_from_py_object
)]
#[derive(Clone)]
struct PyGarmanKlassVolatility {
    inner: wc::GarmanKlassVolatility,
}

#[pymethods]
impl PyGarmanKlassVolatility {
    #[new]
    #[pyo3(signature = (period=20, trading_periods=252))]
    fn new(period: usize, trading_periods: usize) -> PyResult<Self> {
        Ok(Self {
            inner: wc::GarmanKlassVolatility::new(period, trading_periods).map_err(map_err)?,
        })
    }
    fn update(&mut self, candle: &Bound<'_, PyAny>) -> PyResult<Option<f64>> {
        let c = extract_candle(candle)?;
        Ok(self.inner.update(c))
    }
    /// Batch over numpy columns open, high, low, close (all equal length).
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        open: PyReadonlyArray1<'py, f64>,
        high: PyReadonlyArray1<'py, f64>,
        low: PyReadonlyArray1<'py, f64>,
        close: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let o = open
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let h = high
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let l = low
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let cl = close
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        if o.len() != h.len() || h.len() != l.len() || l.len() != cl.len() {
            return Err(PyValueError::new_err(
                "open, high, low, close must be equal length",
            ));
        }
        let mut out = Vec::with_capacity(o.len());
        for i in 0..o.len() {
            let candle = wc::Candle::new(o[i], h[i], l[i], cl[i], 0.0, 0).map_err(map_err)?;
            out.push(self.inner.update(candle).unwrap_or(f64::NAN));
        }
        Ok(out.into_pyarray(py))
    }
    #[getter]
    fn periods(&self) -> (usize, usize) {
        self.inner.periods()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        let (p, t) = self.inner.periods();
        format!("GarmanKlassVolatility(period={p}, trading_periods={t})")
    }
}

// ============================== Parkinson Volatility ==============================

#[pyclass(
    name = "ParkinsonVolatility",
    module = "wickra._wickra",
    skip_from_py_object
)]
#[derive(Clone)]
struct PyParkinsonVolatility {
    inner: wc::ParkinsonVolatility,
}

#[pymethods]
impl PyParkinsonVolatility {
    #[new]
    #[pyo3(signature = (period=20, trading_periods=252))]
    fn new(period: usize, trading_periods: usize) -> PyResult<Self> {
        Ok(Self {
            inner: wc::ParkinsonVolatility::new(period, trading_periods).map_err(map_err)?,
        })
    }
    fn update(&mut self, candle: &Bound<'_, PyAny>) -> PyResult<Option<f64>> {
        let c = extract_candle(candle)?;
        Ok(self.inner.update(c))
    }
    /// Batch over numpy columns high, low (both equal length).
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        high: PyReadonlyArray1<'py, f64>,
        low: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let h = high
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let l = low
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        if h.len() != l.len() {
            return Err(PyValueError::new_err("high and low must be equal length"));
        }
        let mut out = Vec::with_capacity(h.len());
        for i in 0..h.len() {
            let candle = wc::Candle::new(l[i], h[i], l[i], l[i], 0.0, 0).map_err(map_err)?;
            out.push(self.inner.update(candle).unwrap_or(f64::NAN));
        }
        Ok(out.into_pyarray(py))
    }
    #[getter]
    fn periods(&self) -> (usize, usize) {
        self.inner.periods()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        let (p, t) = self.inner.periods();
        format!("ParkinsonVolatility(period={p}, trading_periods={t})")
    }
}

// ============================== RVI (Volatility) ==============================
//
// Named `RVIVolatility` rather than plain `RVI` to disambiguate from
// Relative Vigor Index (a separate momentum indicator that lives in
// Family 02 with the shorter `RVI` name).

#[pyclass(name = "RVIVolatility", module = "wickra._wickra", skip_from_py_object)]
#[derive(Clone)]
struct PyRviVolatility {
    inner: wc::RviVolatility,
}

#[pymethods]
impl PyRviVolatility {
    #[new]
    #[pyo3(signature = (period=10))]
    fn new(period: usize) -> PyResult<Self> {
        Ok(Self {
            inner: wc::RviVolatility::new(period).map_err(map_err)?,
        })
    }
    fn update(&mut self, value: f64) -> Option<f64> {
        self.inner.update(value)
    }
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        prices: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let s = prices
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        Ok(flatten(self.inner.batch(s)).into_pyarray(py))
    }
    #[getter]
    fn period(&self) -> usize {
        self.inner.period()
    }
    #[getter]
    fn value(&self) -> Option<f64> {
        self.inner.value()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        format!("RVIVolatility(period={})", self.inner.period())
    }
}

// ============================== MA Envelope ==============================

#[pyclass(name = "MaEnvelope", module = "wickra._wickra", skip_from_py_object)]
#[derive(Clone)]
struct PyMaEnvelope {
    inner: wc::MaEnvelope,
}

#[pymethods]
impl PyMaEnvelope {
    #[new]
    #[pyo3(signature = (period=20, percent=0.025))]
    fn new(period: usize, percent: f64) -> PyResult<Self> {
        Ok(Self {
            inner: wc::MaEnvelope::new(period, percent).map_err(map_err)?,
        })
    }
    /// Returns `(upper, middle, lower)` or `None` during warmup.
    fn update(&mut self, value: f64) -> Option<(f64, f64, f64)> {
        self.inner
            .update(value)
            .map(|o| (o.upper, o.middle, o.lower))
    }
    /// Batch returns shape `(n, 3)` columns `[upper, middle, lower]`.
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        prices: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray2<f64>>> {
        let slice = prices
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let n = slice.len();
        let mut out = vec![f64::NAN; n * 3];
        for (i, p) in slice.iter().enumerate() {
            if let Some(o) = self.inner.update(*p) {
                out[i * 3] = o.upper;
                out[i * 3 + 1] = o.middle;
                out[i * 3 + 2] = o.lower;
            }
        }
        Ok(numpy::ndarray::Array2::from_shape_vec((n, 3), out)
            .expect("shape consistent")
            .into_pyarray(py))
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
}

// ============================== Acceleration Bands ==============================

#[pyclass(
    name = "AccelerationBands",
    module = "wickra._wickra",
    skip_from_py_object
)]
#[derive(Clone)]
struct PyAccelerationBands {
    inner: wc::AccelerationBands,
}

#[pymethods]
impl PyAccelerationBands {
    #[new]
    #[pyo3(signature = (period=20, factor=0.001))]
    fn new(period: usize, factor: f64) -> PyResult<Self> {
        Ok(Self {
            inner: wc::AccelerationBands::new(period, factor).map_err(map_err)?,
        })
    }
    fn update(&mut self, candle: &Bound<'_, PyAny>) -> PyResult<Option<(f64, f64, f64)>> {
        let c = extract_candle(candle)?;
        Ok(self.inner.update(c).map(|o| (o.upper, o.middle, o.lower)))
    }
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        high: PyReadonlyArray1<'py, f64>,
        low: PyReadonlyArray1<'py, f64>,
        close: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray2<f64>>> {
        let h = high
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let l = low
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let c = close
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        if h.len() != l.len() || l.len() != c.len() {
            return Err(PyValueError::new_err(
                "high, low, close must be equal length",
            ));
        }
        let n = h.len();
        let mut out = vec![f64::NAN; n * 3];
        for i in 0..n {
            let candle = wc::Candle::new(c[i], h[i], l[i], c[i], 0.0, 0).map_err(map_err)?;
            if let Some(o) = self.inner.update(candle) {
                out[i * 3] = o.upper;
                out[i * 3 + 1] = o.middle;
                out[i * 3 + 2] = o.lower;
            }
        }
        Ok(numpy::ndarray::Array2::from_shape_vec((n, 3), out)
            .expect("shape consistent")
            .into_pyarray(py))
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
}

// ============================== STARC Bands ==============================

#[pyclass(name = "StarcBands", module = "wickra._wickra", skip_from_py_object)]
#[derive(Clone)]
struct PyStarcBands {
    inner: wc::StarcBands,
}

#[pymethods]
impl PyStarcBands {
    #[new]
    #[pyo3(signature = (sma_period=6, atr_period=15, multiplier=2.0))]
    fn new(sma_period: usize, atr_period: usize, multiplier: f64) -> PyResult<Self> {
        Ok(Self {
            inner: wc::StarcBands::new(sma_period, atr_period, multiplier).map_err(map_err)?,
        })
    }
    fn update(&mut self, candle: &Bound<'_, PyAny>) -> PyResult<Option<(f64, f64, f64)>> {
        let c = extract_candle(candle)?;
        Ok(self.inner.update(c).map(|o| (o.upper, o.middle, o.lower)))
    }
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        high: PyReadonlyArray1<'py, f64>,
        low: PyReadonlyArray1<'py, f64>,
        close: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray2<f64>>> {
        let h = high
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let l = low
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let c = close
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        if h.len() != l.len() || l.len() != c.len() {
            return Err(PyValueError::new_err(
                "high, low, close must be equal length",
            ));
        }
        let n = h.len();
        let mut out = vec![f64::NAN; n * 3];
        for i in 0..n {
            let candle = wc::Candle::new(c[i], h[i], l[i], c[i], 0.0, 0).map_err(map_err)?;
            if let Some(o) = self.inner.update(candle) {
                out[i * 3] = o.upper;
                out[i * 3 + 1] = o.middle;
                out[i * 3 + 2] = o.lower;
            }
        }
        Ok(numpy::ndarray::Array2::from_shape_vec((n, 3), out)
            .expect("shape consistent")
            .into_pyarray(py))
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
}

// ============================== ATR Bands ==============================

#[pyclass(name = "AtrBands", module = "wickra._wickra", skip_from_py_object)]
#[derive(Clone)]
struct PyAtrBands {
    inner: wc::AtrBands,
}

#[pymethods]
impl PyAtrBands {
    #[new]
    #[pyo3(signature = (period=14, multiplier=3.0))]
    fn new(period: usize, multiplier: f64) -> PyResult<Self> {
        Ok(Self {
            inner: wc::AtrBands::new(period, multiplier).map_err(map_err)?,
        })
    }
    fn update(&mut self, candle: &Bound<'_, PyAny>) -> PyResult<Option<(f64, f64, f64)>> {
        let c = extract_candle(candle)?;
        Ok(self.inner.update(c).map(|o| (o.upper, o.middle, o.lower)))
    }
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        high: PyReadonlyArray1<'py, f64>,
        low: PyReadonlyArray1<'py, f64>,
        close: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray2<f64>>> {
        let h = high
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let l = low
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let c = close
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        if h.len() != l.len() || l.len() != c.len() {
            return Err(PyValueError::new_err(
                "high, low, close must be equal length",
            ));
        }
        let n = h.len();
        let mut out = vec![f64::NAN; n * 3];
        for i in 0..n {
            let candle = wc::Candle::new(c[i], h[i], l[i], c[i], 0.0, 0).map_err(map_err)?;
            if let Some(o) = self.inner.update(candle) {
                out[i * 3] = o.upper;
                out[i * 3 + 1] = o.middle;
                out[i * 3 + 2] = o.lower;
            }
        }
        Ok(numpy::ndarray::Array2::from_shape_vec((n, 3), out)
            .expect("shape consistent")
            .into_pyarray(py))
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
}

// ============================== Hurst Channel ==============================

#[pyclass(name = "HurstChannel", module = "wickra._wickra", skip_from_py_object)]
#[derive(Clone)]
struct PyHurstChannel {
    inner: wc::HurstChannel,
}

#[pymethods]
impl PyHurstChannel {
    #[new]
    #[pyo3(signature = (period=10, multiplier=0.5))]
    fn new(period: usize, multiplier: f64) -> PyResult<Self> {
        Ok(Self {
            inner: wc::HurstChannel::new(period, multiplier).map_err(map_err)?,
        })
    }
    fn update(&mut self, candle: &Bound<'_, PyAny>) -> PyResult<Option<(f64, f64, f64)>> {
        let c = extract_candle(candle)?;
        Ok(self.inner.update(c).map(|o| (o.upper, o.middle, o.lower)))
    }
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        high: PyReadonlyArray1<'py, f64>,
        low: PyReadonlyArray1<'py, f64>,
        close: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray2<f64>>> {
        let h = high
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let l = low
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let c = close
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        if h.len() != l.len() || l.len() != c.len() {
            return Err(PyValueError::new_err(
                "high, low, close must be equal length",
            ));
        }
        let n = h.len();
        let mut out = vec![f64::NAN; n * 3];
        for i in 0..n {
            let candle = wc::Candle::new(c[i], h[i], l[i], c[i], 0.0, 0).map_err(map_err)?;
            if let Some(o) = self.inner.update(candle) {
                out[i * 3] = o.upper;
                out[i * 3 + 1] = o.middle;
                out[i * 3 + 2] = o.lower;
            }
        }
        Ok(numpy::ndarray::Array2::from_shape_vec((n, 3), out)
            .expect("shape consistent")
            .into_pyarray(py))
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
}

// ============================== LinReg Channel ==============================

#[pyclass(name = "LinRegChannel", module = "wickra._wickra", skip_from_py_object)]
#[derive(Clone)]
struct PyLinRegChannel {
    inner: wc::LinRegChannel,
}

#[pymethods]
impl PyLinRegChannel {
    #[new]
    #[pyo3(signature = (period=20, multiplier=2.0))]
    fn new(period: usize, multiplier: f64) -> PyResult<Self> {
        Ok(Self {
            inner: wc::LinRegChannel::new(period, multiplier).map_err(map_err)?,
        })
    }
    fn update(&mut self, value: f64) -> Option<(f64, f64, f64)> {
        self.inner
            .update(value)
            .map(|o| (o.upper, o.middle, o.lower))
    }
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        prices: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray2<f64>>> {
        let slice = prices
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let n = slice.len();
        let mut out = vec![f64::NAN; n * 3];
        for (i, p) in slice.iter().enumerate() {
            if let Some(o) = self.inner.update(*p) {
                out[i * 3] = o.upper;
                out[i * 3 + 1] = o.middle;
                out[i * 3 + 2] = o.lower;
            }
        }
        Ok(numpy::ndarray::Array2::from_shape_vec((n, 3), out)
            .expect("shape consistent")
            .into_pyarray(py))
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
}

// ============================== Standard Error Bands ==============================

#[pyclass(
    name = "StandardErrorBands",
    module = "wickra._wickra",
    skip_from_py_object
)]
#[derive(Clone)]
struct PyStandardErrorBands {
    inner: wc::StandardErrorBands,
}

#[pymethods]
impl PyStandardErrorBands {
    #[new]
    #[pyo3(signature = (period=21, multiplier=2.0))]
    fn new(period: usize, multiplier: f64) -> PyResult<Self> {
        Ok(Self {
            inner: wc::StandardErrorBands::new(period, multiplier).map_err(map_err)?,
        })
    }
    fn update(&mut self, value: f64) -> Option<(f64, f64, f64)> {
        self.inner
            .update(value)
            .map(|o| (o.upper, o.middle, o.lower))
    }
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        prices: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray2<f64>>> {
        let slice = prices
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let n = slice.len();
        let mut out = vec![f64::NAN; n * 3];
        for (i, p) in slice.iter().enumerate() {
            if let Some(o) = self.inner.update(*p) {
                out[i * 3] = o.upper;
                out[i * 3 + 1] = o.middle;
                out[i * 3 + 2] = o.lower;
            }
        }
        Ok(numpy::ndarray::Array2::from_shape_vec((n, 3), out)
            .expect("shape consistent")
            .into_pyarray(py))
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
}

// ============================== Double Bollinger ==============================

#[pyclass(
    name = "DoubleBollinger",
    module = "wickra._wickra",
    skip_from_py_object
)]
#[derive(Clone)]
struct PyDoubleBollinger {
    inner: wc::DoubleBollinger,
}

#[pymethods]
impl PyDoubleBollinger {
    #[new]
    #[pyo3(signature = (period=20, k_inner=1.0, k_outer=2.0))]
    fn new(period: usize, k_inner: f64, k_outer: f64) -> PyResult<Self> {
        Ok(Self {
            inner: wc::DoubleBollinger::new(period, k_inner, k_outer).map_err(map_err)?,
        })
    }
    /// Returns `(upper_outer, upper_inner, middle, lower_inner, lower_outer)`.
    fn update(&mut self, value: f64) -> Option<(f64, f64, f64, f64, f64)> {
        self.inner.update(value).map(|o| {
            (
                o.upper_outer,
                o.upper_inner,
                o.middle,
                o.lower_inner,
                o.lower_outer,
            )
        })
    }
    /// Returns shape `(n, 5)` columns
    /// `[upper_outer, upper_inner, middle, lower_inner, lower_outer]`.
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        prices: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray2<f64>>> {
        let slice = prices
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let n = slice.len();
        let mut out = vec![f64::NAN; n * 5];
        for (i, p) in slice.iter().enumerate() {
            if let Some(o) = self.inner.update(*p) {
                out[i * 5] = o.upper_outer;
                out[i * 5 + 1] = o.upper_inner;
                out[i * 5 + 2] = o.middle;
                out[i * 5 + 3] = o.lower_inner;
                out[i * 5 + 4] = o.lower_outer;
            }
        }
        Ok(numpy::ndarray::Array2::from_shape_vec((n, 5), out)
            .expect("shape consistent")
            .into_pyarray(py))
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
}

// ============================== TTM Squeeze ==============================

#[pyclass(name = "TtmSqueeze", module = "wickra._wickra", skip_from_py_object)]
#[derive(Clone)]
struct PyTtmSqueeze {
    inner: wc::TtmSqueeze,
}

#[pymethods]
impl PyTtmSqueeze {
    #[new]
    #[pyo3(signature = (period=20, bb_mult=2.0, kc_mult=1.5))]
    fn new(period: usize, bb_mult: f64, kc_mult: f64) -> PyResult<Self> {
        Ok(Self {
            inner: wc::TtmSqueeze::new(period, bb_mult, kc_mult).map_err(map_err)?,
        })
    }
    /// Returns `(squeeze, momentum)` or `None` during warmup. `squeeze` is
    /// `1.0` while BB ⊂ KC, `0.0` otherwise.
    fn update(&mut self, candle: &Bound<'_, PyAny>) -> PyResult<Option<(f64, f64)>> {
        let c = extract_candle(candle)?;
        Ok(self.inner.update(c).map(|o| (o.squeeze, o.momentum)))
    }
    /// Returns shape `(n, 2)` columns `[squeeze, momentum]`.
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        high: PyReadonlyArray1<'py, f64>,
        low: PyReadonlyArray1<'py, f64>,
        close: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray2<f64>>> {
        let h = high
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let l = low
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let c = close
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        if h.len() != l.len() || l.len() != c.len() {
            return Err(PyValueError::new_err(
                "high, low, close must be equal length",
            ));
        }
        let n = h.len();
        let mut out = vec![f64::NAN; n * 2];
        for i in 0..n {
            let candle = wc::Candle::new(c[i], h[i], l[i], c[i], 0.0, 0).map_err(map_err)?;
            if let Some(o) = self.inner.update(candle) {
                out[i * 2] = o.squeeze;
                out[i * 2 + 1] = o.momentum;
            }
        }
        Ok(numpy::ndarray::Array2::from_shape_vec((n, 2), out)
            .expect("shape consistent")
            .into_pyarray(py))
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
}

// ============================== Fractal Chaos Bands ==============================

#[pyclass(
    name = "FractalChaosBands",
    module = "wickra._wickra",
    skip_from_py_object
)]
#[derive(Clone)]
struct PyFractalChaosBands {
    inner: wc::FractalChaosBands,
}

#[pymethods]
impl PyFractalChaosBands {
    #[new]
    #[pyo3(signature = (k=2))]
    fn new(k: usize) -> PyResult<Self> {
        Ok(Self {
            inner: wc::FractalChaosBands::new(k).map_err(map_err)?,
        })
    }
    /// Returns `(upper, lower)` or `None` until both fractals have confirmed.
    fn update(&mut self, candle: &Bound<'_, PyAny>) -> PyResult<Option<(f64, f64)>> {
        let c = extract_candle(candle)?;
        Ok(self.inner.update(c).map(|o| (o.upper, o.lower)))
    }
    /// Returns shape `(n, 2)` columns `[upper, lower]`.
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        high: PyReadonlyArray1<'py, f64>,
        low: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray2<f64>>> {
        let h = high
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let l = low
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        if h.len() != l.len() {
            return Err(PyValueError::new_err("high and low must be equal length"));
        }
        let n = h.len();
        let mut out = vec![f64::NAN; n * 2];
        for i in 0..n {
            let candle = wc::Candle::new(l[i], h[i], l[i], l[i], 0.0, 0).map_err(map_err)?;
            if let Some(o) = self.inner.update(candle) {
                out[i * 2] = o.upper;
                out[i * 2 + 1] = o.lower;
            }
        }
        Ok(numpy::ndarray::Array2::from_shape_vec((n, 2), out)
            .expect("shape consistent")
            .into_pyarray(py))
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
}

// ============================== VWAP StdDev Bands ==============================

#[pyclass(
    name = "VwapStdDevBands",
    module = "wickra._wickra",
    skip_from_py_object
)]
#[derive(Clone)]
struct PyVwapStdDevBands {
    inner: wc::VwapStdDevBands,
}

#[pymethods]
impl PyVwapStdDevBands {
    #[new]
    #[pyo3(signature = (multiplier=2.0))]
    fn new(multiplier: f64) -> PyResult<Self> {
        Ok(Self {
            inner: wc::VwapStdDevBands::new(multiplier).map_err(map_err)?,
        })
    }
    /// Returns `(upper, middle, lower, stddev)` or `None` until volume is non-zero.
    fn update(&mut self, candle: &Bound<'_, PyAny>) -> PyResult<Option<(f64, f64, f64, f64)>> {
        let c = extract_candle(candle)?;
        Ok(self
            .inner
            .update(c)
            .map(|o| (o.upper, o.middle, o.lower, o.stddev)))
    }
    /// Returns shape `(n, 4)` columns `[upper, middle, lower, stddev]`.
    #[allow(clippy::many_single_char_names)]
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        high: PyReadonlyArray1<'py, f64>,
        low: PyReadonlyArray1<'py, f64>,
        close: PyReadonlyArray1<'py, f64>,
        volume: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray2<f64>>> {
        let h = high
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let l = low
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let c = close
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let v = volume
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        if h.len() != l.len() || l.len() != c.len() || c.len() != v.len() {
            return Err(PyValueError::new_err(
                "high, low, close, volume must be equal length",
            ));
        }
        let n = h.len();
        let mut out = vec![f64::NAN; n * 4];
        for i in 0..n {
            let candle = wc::Candle::new(c[i], h[i], l[i], c[i], v[i], 0).map_err(map_err)?;
            if let Some(o) = self.inner.update(candle) {
                out[i * 4] = o.upper;
                out[i * 4 + 1] = o.middle;
                out[i * 4 + 2] = o.lower;
                out[i * 4 + 3] = o.stddev;
            }
        }
        Ok(numpy::ndarray::Array2::from_shape_vec((n, 4), out)
            .expect("shape consistent")
            .into_pyarray(py))
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
}

// ============================== Classic Pivots ==============================

#[pyclass(name = "ClassicPivots", module = "wickra._wickra", skip_from_py_object)]
#[derive(Clone)]
struct PyClassicPivots {
    inner: wc::ClassicPivots,
}

#[pymethods]
impl PyClassicPivots {
    #[new]
    fn new() -> Self {
        Self {
            inner: wc::ClassicPivots::new(),
        }
    }
    /// Returns `(pp, r1, r2, r3, s1, s2, s3)` or None during warmup.
    fn update(&mut self, candle: &Bound<'_, PyAny>) -> PyResult<Option<PivotLevels>> {
        let c = extract_candle(candle)?;
        Ok(self
            .inner
            .update(c)
            .map(|o| (o.pp, o.r1, o.r2, o.r3, o.s1, o.s2, o.s3)))
    }
    /// Batch over numpy columns high, low, close. Returns shape `(n, 7)` for
    /// `[pp, r1, r2, r3, s1, s2, s3]`.
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        high: PyReadonlyArray1<'py, f64>,
        low: PyReadonlyArray1<'py, f64>,
        close: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray2<f64>>> {
        let h = high
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let l = low
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let c = close
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        if h.len() != l.len() || l.len() != c.len() {
            return Err(PyValueError::new_err(
                "high, low, close must be equal length",
            ));
        }
        let n = h.len();
        let mut out = vec![f64::NAN; n * 7];
        for i in 0..n {
            let candle = wc::Candle::new(c[i], h[i], l[i], c[i], 0.0, 0).map_err(map_err)?;
            if let Some(o) = self.inner.update(candle) {
                out[i * 7] = o.pp;
                out[i * 7 + 1] = o.r1;
                out[i * 7 + 2] = o.r2;
                out[i * 7 + 3] = o.r3;
                out[i * 7 + 4] = o.s1;
                out[i * 7 + 5] = o.s2;
                out[i * 7 + 6] = o.s3;
            }
        }
        Ok(numpy::ndarray::Array2::from_shape_vec((n, 7), out)
            .expect("shape consistent")
            .into_pyarray(py))
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
}

// ============================== Fibonacci Pivots ==============================

#[pyclass(
    name = "FibonacciPivots",
    module = "wickra._wickra",
    skip_from_py_object
)]
#[derive(Clone)]
struct PyFibonacciPivots {
    inner: wc::FibonacciPivots,
}

#[pymethods]
impl PyFibonacciPivots {
    #[new]
    fn new() -> Self {
        Self {
            inner: wc::FibonacciPivots::new(),
        }
    }
    fn update(&mut self, candle: &Bound<'_, PyAny>) -> PyResult<Option<PivotLevels>> {
        let c = extract_candle(candle)?;
        Ok(self
            .inner
            .update(c)
            .map(|o| (o.pp, o.r1, o.r2, o.r3, o.s1, o.s2, o.s3)))
    }
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        high: PyReadonlyArray1<'py, f64>,
        low: PyReadonlyArray1<'py, f64>,
        close: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray2<f64>>> {
        let h = high
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let l = low
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let c = close
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        if h.len() != l.len() || l.len() != c.len() {
            return Err(PyValueError::new_err(
                "high, low, close must be equal length",
            ));
        }
        let n = h.len();
        let mut out = vec![f64::NAN; n * 7];
        for i in 0..n {
            let candle = wc::Candle::new(c[i], h[i], l[i], c[i], 0.0, 0).map_err(map_err)?;
            if let Some(o) = self.inner.update(candle) {
                out[i * 7] = o.pp;
                out[i * 7 + 1] = o.r1;
                out[i * 7 + 2] = o.r2;
                out[i * 7 + 3] = o.r3;
                out[i * 7 + 4] = o.s1;
                out[i * 7 + 5] = o.s2;
                out[i * 7 + 6] = o.s3;
            }
        }
        Ok(numpy::ndarray::Array2::from_shape_vec((n, 7), out)
            .expect("shape consistent")
            .into_pyarray(py))
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
}

// ============================== Camarilla Pivots ==============================

#[pyclass(name = "Camarilla", module = "wickra._wickra", skip_from_py_object)]
#[derive(Clone)]
struct PyCamarilla {
    inner: wc::Camarilla,
}

#[pymethods]
impl PyCamarilla {
    #[new]
    fn new() -> Self {
        Self {
            inner: wc::Camarilla::new(),
        }
    }
    /// Returns `(pp, r1, r2, r3, r4, s1, s2, s3, s4)` or None during warmup.
    #[allow(clippy::type_complexity)]
    fn update(
        &mut self,
        candle: &Bound<'_, PyAny>,
    ) -> PyResult<Option<(f64, f64, f64, f64, f64, f64, f64, f64, f64)>> {
        let c = extract_candle(candle)?;
        Ok(self
            .inner
            .update(c)
            .map(|o| (o.pp, o.r1, o.r2, o.r3, o.r4, o.s1, o.s2, o.s3, o.s4)))
    }
    /// Batch over numpy columns high, low, close. Returns shape `(n, 9)` for
    /// `[pp, r1, r2, r3, r4, s1, s2, s3, s4]`.
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        high: PyReadonlyArray1<'py, f64>,
        low: PyReadonlyArray1<'py, f64>,
        close: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray2<f64>>> {
        let h = high
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let l = low
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let c = close
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        if h.len() != l.len() || l.len() != c.len() {
            return Err(PyValueError::new_err(
                "high, low, close must be equal length",
            ));
        }
        let n = h.len();
        let mut out = vec![f64::NAN; n * 9];
        for i in 0..n {
            let candle = wc::Candle::new(c[i], h[i], l[i], c[i], 0.0, 0).map_err(map_err)?;
            if let Some(o) = self.inner.update(candle) {
                out[i * 9] = o.pp;
                out[i * 9 + 1] = o.r1;
                out[i * 9 + 2] = o.r2;
                out[i * 9 + 3] = o.r3;
                out[i * 9 + 4] = o.r4;
                out[i * 9 + 5] = o.s1;
                out[i * 9 + 6] = o.s2;
                out[i * 9 + 7] = o.s3;
                out[i * 9 + 8] = o.s4;
            }
        }
        Ok(numpy::ndarray::Array2::from_shape_vec((n, 9), out)
            .expect("shape consistent")
            .into_pyarray(py))
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
}

// ============================== Woodie Pivots ==============================

#[pyclass(name = "WoodiePivots", module = "wickra._wickra", skip_from_py_object)]
#[derive(Clone)]
struct PyWoodiePivots {
    inner: wc::WoodiePivots,
}

#[pymethods]
impl PyWoodiePivots {
    #[new]
    fn new() -> Self {
        Self {
            inner: wc::WoodiePivots::new(),
        }
    }
    /// Returns `(pp, r1, r2, s1, s2)` or None during warmup.
    fn update(&mut self, candle: &Bound<'_, PyAny>) -> PyResult<Option<WoodieLevels>> {
        let c = extract_candle(candle)?;
        Ok(self.inner.update(c).map(|o| (o.pp, o.r1, o.r2, o.s1, o.s2)))
    }
    /// Batch over numpy columns high, low, close. Returns shape `(n, 5)` for
    /// `[pp, r1, r2, s1, s2]`.
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        high: PyReadonlyArray1<'py, f64>,
        low: PyReadonlyArray1<'py, f64>,
        close: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray2<f64>>> {
        let h = high
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let l = low
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let c = close
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        if h.len() != l.len() || l.len() != c.len() {
            return Err(PyValueError::new_err(
                "high, low, close must be equal length",
            ));
        }
        let n = h.len();
        let mut out = vec![f64::NAN; n * 5];
        for i in 0..n {
            let candle = wc::Candle::new(c[i], h[i], l[i], c[i], 0.0, 0).map_err(map_err)?;
            if let Some(o) = self.inner.update(candle) {
                out[i * 5] = o.pp;
                out[i * 5 + 1] = o.r1;
                out[i * 5 + 2] = o.r2;
                out[i * 5 + 3] = o.s1;
                out[i * 5 + 4] = o.s2;
            }
        }
        Ok(numpy::ndarray::Array2::from_shape_vec((n, 5), out)
            .expect("shape consistent")
            .into_pyarray(py))
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
}

// ============================== DeMark Pivots ==============================

#[pyclass(name = "DemarkPivots", module = "wickra._wickra", skip_from_py_object)]
#[derive(Clone)]
struct PyDemarkPivots {
    inner: wc::DemarkPivots,
}

#[pymethods]
impl PyDemarkPivots {
    #[new]
    fn new() -> Self {
        Self {
            inner: wc::DemarkPivots::new(),
        }
    }
    /// Returns `(pp, r1, s1)` or None during warmup.
    fn update(&mut self, candle: &Bound<'_, PyAny>) -> PyResult<Option<(f64, f64, f64)>> {
        let c = extract_candle(candle)?;
        Ok(self.inner.update(c).map(|o| (o.pp, o.r1, o.s1)))
    }
    /// Batch over numpy columns open, high, low, close. Returns shape `(n, 3)`
    /// for `[pp, r1, s1]`.
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        open: PyReadonlyArray1<'py, f64>,
        high: PyReadonlyArray1<'py, f64>,
        low: PyReadonlyArray1<'py, f64>,
        close: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray2<f64>>> {
        let o = open
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let h = high
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let l = low
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let c = close
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        if o.len() != h.len() || h.len() != l.len() || l.len() != c.len() {
            return Err(PyValueError::new_err(
                "open, high, low, close must be equal length",
            ));
        }
        let n = o.len();
        let mut out = vec![f64::NAN; n * 3];
        for i in 0..n {
            let candle = wc::Candle::new(o[i], h[i], l[i], c[i], 0.0, 0).map_err(map_err)?;
            if let Some(v) = self.inner.update(candle) {
                out[i * 3] = v.pp;
                out[i * 3 + 1] = v.r1;
                out[i * 3 + 2] = v.s1;
            }
        }
        Ok(numpy::ndarray::Array2::from_shape_vec((n, 3), out)
            .expect("shape consistent")
            .into_pyarray(py))
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
}

// ============================== Williams Fractals ==============================

#[pyclass(
    name = "WilliamsFractals",
    module = "wickra._wickra",
    skip_from_py_object
)]
#[derive(Clone)]
struct PyWilliamsFractals {
    inner: wc::WilliamsFractals,
}

#[pymethods]
impl PyWilliamsFractals {
    #[new]
    fn new() -> Self {
        Self {
            inner: wc::WilliamsFractals::new(),
        }
    }
    /// Returns `(up, down)` where each component is either the fractal price
    /// or `None` if no fractal was confirmed at the centre of the current
    /// 5-bar window. The outer `None` is returned during warmup (first 4 bars).
    fn update(
        &mut self,
        candle: &Bound<'_, PyAny>,
    ) -> PyResult<Option<(Option<f64>, Option<f64>)>> {
        let c = extract_candle(candle)?;
        Ok(self.inner.update(c).map(|o| (o.up, o.down)))
    }
    /// Batch over numpy columns high, low. Returns shape `(n, 2)` for
    /// `[up_fractal, down_fractal]`. Values are NaN both during warmup and on
    /// bars where no fractal was confirmed.
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        high: PyReadonlyArray1<'py, f64>,
        low: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray2<f64>>> {
        let h = high
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let l = low
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        if h.len() != l.len() {
            return Err(PyValueError::new_err("high and low must be equal length"));
        }
        let n = h.len();
        let mut out = vec![f64::NAN; n * 2];
        for i in 0..n {
            let candle = wc::Candle::new(l[i], h[i], l[i], l[i], 0.0, 0).map_err(map_err)?;
            if let Some(o) = self.inner.update(candle) {
                if let Some(v) = o.up {
                    out[i * 2] = v;
                }
                if let Some(v) = o.down {
                    out[i * 2 + 1] = v;
                }
            }
        }
        Ok(numpy::ndarray::Array2::from_shape_vec((n, 2), out)
            .expect("shape consistent")
            .into_pyarray(py))
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
}

// ============================== ZigZag ==============================

#[pyclass(name = "ZigZag", module = "wickra._wickra", skip_from_py_object)]
#[derive(Clone)]
struct PyZigZag {
    inner: wc::ZigZag,
}

#[pymethods]
impl PyZigZag {
    #[new]
    #[pyo3(signature = (threshold=0.05))]
    fn new(threshold: f64) -> PyResult<Self> {
        Ok(Self {
            inner: wc::ZigZag::new(threshold).map_err(map_err)?,
        })
    }
    /// Returns `(swing, direction)` if a swing was confirmed on this bar,
    /// else `None`.
    fn update(&mut self, candle: &Bound<'_, PyAny>) -> PyResult<Option<(f64, f64)>> {
        let c = extract_candle(candle)?;
        Ok(self.inner.update(c).map(|o| (o.swing, o.direction)))
    }
    /// Batch over numpy columns high, low. Returns shape `(n, 2)` for
    /// `[swing_price, direction]`. NaN on bars without a confirmed swing.
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        high: PyReadonlyArray1<'py, f64>,
        low: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray2<f64>>> {
        let h = high
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let l = low
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        if h.len() != l.len() {
            return Err(PyValueError::new_err("high and low must be equal length"));
        }
        let n = h.len();
        let mut out = vec![f64::NAN; n * 2];
        for i in 0..n {
            let candle = wc::Candle::new(l[i], h[i], l[i], l[i], 0.0, 0).map_err(map_err)?;
            if let Some(o) = self.inner.update(candle) {
                out[i * 2] = o.swing;
                out[i * 2 + 1] = o.direction;
            }
        }
        Ok(numpy::ndarray::Array2::from_shape_vec((n, 2), out)
            .expect("shape consistent")
            .into_pyarray(py))
    }
    #[getter]
    fn threshold(&self) -> f64 {
        self.inner.threshold()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
}
// ============================== TD Setup ==============================

#[pyclass(name = "TDSetup", module = "wickra._wickra", skip_from_py_object)]
#[derive(Clone)]
struct PyTdSetup {
    inner: wc::TdSetup,
}

#[pymethods]
impl PyTdSetup {
    #[new]
    #[pyo3(signature = (lookback=4, target=9))]
    fn new(lookback: usize, target: usize) -> PyResult<Self> {
        Ok(Self {
            inner: wc::TdSetup::new(lookback, target).map_err(map_err)?,
        })
    }
    fn update(&mut self, candle: &Bound<'_, PyAny>) -> PyResult<Option<f64>> {
        let c = extract_candle(candle)?;
        Ok(self.inner.update(c))
    }
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        high: PyReadonlyArray1<'py, f64>,
        low: PyReadonlyArray1<'py, f64>,
        close: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let h = high
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let l = low
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let c = close
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        if h.len() != l.len() || l.len() != c.len() {
            return Err(PyValueError::new_err(
                "high, low, close must be equal length",
            ));
        }
        let mut out = Vec::with_capacity(h.len());
        for i in 0..h.len() {
            let candle = wc::Candle::new(c[i], h[i], l[i], c[i], 0.0, 0).map_err(map_err)?;
            out.push(self.inner.update(candle).unwrap_or(f64::NAN));
        }
        Ok(out.into_pyarray(py))
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    #[getter]
    fn value(&self) -> Option<f64> {
        self.inner.value()
    }
    fn __repr__(&self) -> String {
        let (lb, tg) = self.inner.params();
        format!("TDSetup(lookback={lb}, target={tg})")
    }
}

// ============================== TD Sequential ==============================

#[pyclass(name = "TDSequential", module = "wickra._wickra", skip_from_py_object)]
#[derive(Clone)]
struct PyTdSequential {
    inner: wc::TdSequential,
}

#[pymethods]
impl PyTdSequential {
    #[new]
    #[pyo3(signature = (setup_lookback=4, setup_target=9, countdown_lookback=2, countdown_target=13))]
    fn new(
        setup_lookback: usize,
        setup_target: usize,
        countdown_lookback: usize,
        countdown_target: usize,
    ) -> PyResult<Self> {
        Ok(Self {
            inner: wc::TdSequential::new(
                setup_lookback,
                setup_target,
                countdown_lookback,
                countdown_target,
            )
            .map_err(map_err)?,
        })
    }
    /// Returns `(setup, countdown, direction)` or `None` during warmup.
    fn update(&mut self, candle: &Bound<'_, PyAny>) -> PyResult<Option<(f64, f64, f64)>> {
        let c = extract_candle(candle)?;
        Ok(self
            .inner
            .update(c)
            .map(|o| (o.setup, o.countdown, o.direction)))
    }
    /// Batch returns shape `(n, 3)`: `[setup, countdown, direction]`.
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        high: PyReadonlyArray1<'py, f64>,
        low: PyReadonlyArray1<'py, f64>,
        close: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray2<f64>>> {
        let h = high
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let l = low
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let c = close
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        if h.len() != l.len() || l.len() != c.len() {
            return Err(PyValueError::new_err(
                "high, low, close must be equal length",
            ));
        }
        let n = h.len();
        let mut out = vec![f64::NAN; n * 3];
        for i in 0..n {
            let candle = wc::Candle::new(c[i], h[i], l[i], c[i], 0.0, 0).map_err(map_err)?;
            if let Some(o) = self.inner.update(candle) {
                out[i * 3] = o.setup;
                out[i * 3 + 1] = o.countdown;
                out[i * 3 + 2] = o.direction;
            }
        }
        Ok(numpy::ndarray::Array2::from_shape_vec((n, 3), out)
            .expect("shape consistent")
            .into_pyarray(py))
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
}

// ============================== TD DeMarker ==============================

#[pyclass(name = "TDDeMarker", module = "wickra._wickra", skip_from_py_object)]
#[derive(Clone)]
struct PyTdDeMarker {
    inner: wc::TdDeMarker,
}

#[pymethods]
impl PyTdDeMarker {
    #[new]
    #[pyo3(signature = (period=14))]
    fn new(period: usize) -> PyResult<Self> {
        Ok(Self {
            inner: wc::TdDeMarker::new(period).map_err(map_err)?,
        })
    }
    fn update(&mut self, candle: &Bound<'_, PyAny>) -> PyResult<Option<f64>> {
        let c = extract_candle(candle)?;
        Ok(self.inner.update(c))
    }
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        high: PyReadonlyArray1<'py, f64>,
        low: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let h = high
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let l = low
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        if h.len() != l.len() {
            return Err(PyValueError::new_err("high and low must be equal length"));
        }
        let mut out = Vec::with_capacity(h.len());
        for i in 0..h.len() {
            let candle = wc::Candle::new(l[i], h[i], l[i], l[i], 0.0, 0).map_err(map_err)?;
            out.push(self.inner.update(candle).unwrap_or(f64::NAN));
        }
        Ok(out.into_pyarray(py))
    }
    #[getter]
    fn period(&self) -> usize {
        self.inner.period()
    }
    #[getter]
    fn value(&self) -> Option<f64> {
        self.inner.value()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        format!("TDDeMarker(period={})", self.inner.period())
    }
}

// ============================== TD REI ==============================

#[pyclass(name = "TDREI", module = "wickra._wickra", skip_from_py_object)]
#[derive(Clone)]
struct PyTdRei {
    inner: wc::TdRei,
}

#[pymethods]
impl PyTdRei {
    #[new]
    #[pyo3(signature = (period=5))]
    fn new(period: usize) -> PyResult<Self> {
        Ok(Self {
            inner: wc::TdRei::new(period).map_err(map_err)?,
        })
    }
    fn update(&mut self, candle: &Bound<'_, PyAny>) -> PyResult<Option<f64>> {
        let c = extract_candle(candle)?;
        Ok(self.inner.update(c))
    }
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        high: PyReadonlyArray1<'py, f64>,
        low: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let h = high
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let l = low
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        if h.len() != l.len() {
            return Err(PyValueError::new_err("high and low must be equal length"));
        }
        let mut out = Vec::with_capacity(h.len());
        for i in 0..h.len() {
            let candle = wc::Candle::new(l[i], h[i], l[i], l[i], 0.0, 0).map_err(map_err)?;
            out.push(self.inner.update(candle).unwrap_or(f64::NAN));
        }
        Ok(out.into_pyarray(py))
    }
    #[getter]
    fn period(&self) -> usize {
        self.inner.period()
    }
    #[getter]
    fn value(&self) -> Option<f64> {
        self.inner.value()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        format!("TDREI(period={})", self.inner.period())
    }
}

// ============================== TD Pressure ==============================

#[pyclass(name = "TDPressure", module = "wickra._wickra", skip_from_py_object)]
#[derive(Clone)]
struct PyTdPressure {
    inner: wc::TdPressure,
}

#[pymethods]
impl PyTdPressure {
    #[new]
    #[pyo3(signature = (period=5))]
    fn new(period: usize) -> PyResult<Self> {
        Ok(Self {
            inner: wc::TdPressure::new(period).map_err(map_err)?,
        })
    }
    fn update(&mut self, candle: &Bound<'_, PyAny>) -> PyResult<Option<f64>> {
        let c = extract_candle(candle)?;
        Ok(self.inner.update(c))
    }
    /// Batch over numpy columns: open, high, low, close, volume.
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        open: PyReadonlyArray1<'py, f64>,
        high: PyReadonlyArray1<'py, f64>,
        low: PyReadonlyArray1<'py, f64>,
        close: PyReadonlyArray1<'py, f64>,
        volume: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let o = open
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let h = high
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let l = low
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let c = close
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let v = volume
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        if o.len() != h.len() || h.len() != l.len() || l.len() != c.len() || c.len() != v.len() {
            return Err(PyValueError::new_err(
                "open, high, low, close, volume must be equal length",
            ));
        }
        let mut out = Vec::with_capacity(o.len());
        for i in 0..o.len() {
            let candle = wc::Candle::new(o[i], h[i], l[i], c[i], v[i], 0).map_err(map_err)?;
            out.push(self.inner.update(candle).unwrap_or(f64::NAN));
        }
        Ok(out.into_pyarray(py))
    }
    #[getter]
    fn period(&self) -> usize {
        self.inner.period()
    }
    #[getter]
    fn value(&self) -> Option<f64> {
        self.inner.value()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        format!("TDPressure(period={})", self.inner.period())
    }
}

// ============================== TD Combo ==============================

#[pyclass(name = "TDCombo", module = "wickra._wickra", skip_from_py_object)]
#[derive(Clone)]
struct PyTdCombo {
    inner: wc::TdCombo,
}

#[pymethods]
impl PyTdCombo {
    #[new]
    #[pyo3(signature = (setup_lookback=4, setup_target=9, countdown_lookback=2, countdown_target=13))]
    fn new(
        setup_lookback: usize,
        setup_target: usize,
        countdown_lookback: usize,
        countdown_target: usize,
    ) -> PyResult<Self> {
        Ok(Self {
            inner: wc::TdCombo::new(
                setup_lookback,
                setup_target,
                countdown_lookback,
                countdown_target,
            )
            .map_err(map_err)?,
        })
    }
    fn update(&mut self, candle: &Bound<'_, PyAny>) -> PyResult<Option<f64>> {
        let c = extract_candle(candle)?;
        Ok(self.inner.update(c))
    }
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        high: PyReadonlyArray1<'py, f64>,
        low: PyReadonlyArray1<'py, f64>,
        close: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let h = high
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let l = low
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let c = close
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        if h.len() != l.len() || l.len() != c.len() {
            return Err(PyValueError::new_err(
                "high, low, close must be equal length",
            ));
        }
        let mut out = Vec::with_capacity(h.len());
        for i in 0..h.len() {
            let candle = wc::Candle::new(c[i], h[i], l[i], c[i], 0.0, 0).map_err(map_err)?;
            out.push(self.inner.update(candle).unwrap_or(f64::NAN));
        }
        Ok(out.into_pyarray(py))
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
}

// ============================== TD Countdown ==============================

#[pyclass(name = "TDCountdown", module = "wickra._wickra", skip_from_py_object)]
#[derive(Clone)]
struct PyTdCountdown {
    inner: wc::TdCountdown,
}

#[pymethods]
impl PyTdCountdown {
    #[new]
    #[pyo3(signature = (setup_lookback=4, setup_target=9, countdown_lookback=2, countdown_target=13))]
    fn new(
        setup_lookback: usize,
        setup_target: usize,
        countdown_lookback: usize,
        countdown_target: usize,
    ) -> PyResult<Self> {
        Ok(Self {
            inner: wc::TdCountdown::new(
                setup_lookback,
                setup_target,
                countdown_lookback,
                countdown_target,
            )
            .map_err(map_err)?,
        })
    }
    fn update(&mut self, candle: &Bound<'_, PyAny>) -> PyResult<Option<f64>> {
        let c = extract_candle(candle)?;
        Ok(self.inner.update(c))
    }
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        high: PyReadonlyArray1<'py, f64>,
        low: PyReadonlyArray1<'py, f64>,
        close: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let h = high
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let l = low
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let c = close
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        if h.len() != l.len() || l.len() != c.len() {
            return Err(PyValueError::new_err(
                "high, low, close must be equal length",
            ));
        }
        let mut out = Vec::with_capacity(h.len());
        for i in 0..h.len() {
            let candle = wc::Candle::new(c[i], h[i], l[i], c[i], 0.0, 0).map_err(map_err)?;
            out.push(self.inner.update(candle).unwrap_or(f64::NAN));
        }
        Ok(out.into_pyarray(py))
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
}

// ============================== TD Lines ==============================

#[pyclass(name = "TDLines", module = "wickra._wickra", skip_from_py_object)]
#[derive(Clone)]
struct PyTdLines {
    inner: wc::TdLines,
}

#[pymethods]
impl PyTdLines {
    #[new]
    #[pyo3(signature = (lookback=4, target=9))]
    fn new(lookback: usize, target: usize) -> PyResult<Self> {
        Ok(Self {
            inner: wc::TdLines::new(lookback, target).map_err(map_err)?,
        })
    }
    /// Returns `(resistance, support)` (with `NaN` for unset levels) or
    /// `None` during warmup.
    fn update(&mut self, candle: &Bound<'_, PyAny>) -> PyResult<Option<(f64, f64)>> {
        let c = extract_candle(candle)?;
        Ok(self.inner.update(c).map(|o| (o.resistance, o.support)))
    }
    /// Batch returns shape `(n, 2)`: `[resistance, support]`.
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        high: PyReadonlyArray1<'py, f64>,
        low: PyReadonlyArray1<'py, f64>,
        close: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray2<f64>>> {
        let h = high
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let l = low
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let c = close
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        if h.len() != l.len() || l.len() != c.len() {
            return Err(PyValueError::new_err(
                "high, low, close must be equal length",
            ));
        }
        let n = h.len();
        let mut out = vec![f64::NAN; n * 2];
        for i in 0..n {
            let candle = wc::Candle::new(c[i], h[i], l[i], c[i], 0.0, 0).map_err(map_err)?;
            if let Some(o) = self.inner.update(candle) {
                out[i * 2] = o.resistance;
                out[i * 2 + 1] = o.support;
            }
        }
        Ok(numpy::ndarray::Array2::from_shape_vec((n, 2), out)
            .expect("shape consistent")
            .into_pyarray(py))
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
}

// ============================== TD Range Projection ==============================

#[pyclass(
    name = "TDRangeProjection",
    module = "wickra._wickra",
    skip_from_py_object
)]
#[derive(Clone, Default)]
struct PyTdRangeProjection {
    inner: wc::TdRangeProjection,
}

#[pymethods]
impl PyTdRangeProjection {
    #[new]
    fn new() -> Self {
        Self {
            inner: wc::TdRangeProjection::new(),
        }
    }
    /// Returns `(projected_high, projected_low)` for the next bar.
    fn update(&mut self, candle: &Bound<'_, PyAny>) -> PyResult<Option<(f64, f64)>> {
        let c = extract_candle(candle)?;
        Ok(self.inner.update(c).map(|o| (o.high, o.low)))
    }
    /// Batch returns shape `(n, 2)`: `[projected_high, projected_low]`.
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        open: PyReadonlyArray1<'py, f64>,
        high: PyReadonlyArray1<'py, f64>,
        low: PyReadonlyArray1<'py, f64>,
        close: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray2<f64>>> {
        let o = open
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let h = high
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let l = low
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let c = close
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        if o.len() != h.len() || h.len() != l.len() || l.len() != c.len() {
            return Err(PyValueError::new_err(
                "open, high, low, close must be equal length",
            ));
        }
        let n = o.len();
        let mut out = vec![f64::NAN; n * 2];
        for i in 0..n {
            let candle = wc::Candle::new(o[i], h[i], l[i], c[i], 0.0, 0).map_err(map_err)?;
            if let Some(p) = self.inner.update(candle) {
                out[i * 2] = p.high;
                out[i * 2 + 1] = p.low;
            }
        }
        Ok(numpy::ndarray::Array2::from_shape_vec((n, 2), out)
            .expect("shape consistent")
            .into_pyarray(py))
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
}

// ============================== TD Differential ==============================

#[pyclass(
    name = "TDDifferential",
    module = "wickra._wickra",
    skip_from_py_object
)]
#[derive(Clone, Default)]
struct PyTdDifferential {
    inner: wc::TdDifferential,
}

#[pymethods]
impl PyTdDifferential {
    #[new]
    fn new() -> Self {
        Self {
            inner: wc::TdDifferential::new(),
        }
    }
    fn update(&mut self, candle: &Bound<'_, PyAny>) -> PyResult<Option<f64>> {
        let c = extract_candle(candle)?;
        Ok(self.inner.update(c))
    }
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        high: PyReadonlyArray1<'py, f64>,
        low: PyReadonlyArray1<'py, f64>,
        close: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let h = high
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let l = low
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let c = close
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        if h.len() != l.len() || l.len() != c.len() {
            return Err(PyValueError::new_err(
                "high, low, close must be equal length",
            ));
        }
        let mut out = Vec::with_capacity(h.len());
        for i in 0..h.len() {
            let candle = wc::Candle::new(c[i], h[i], l[i], c[i], 0.0, 0).map_err(map_err)?;
            out.push(self.inner.update(candle).unwrap_or(f64::NAN));
        }
        Ok(out.into_pyarray(py))
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
}

// ============================== TD Open ==============================

#[pyclass(name = "TDOpen", module = "wickra._wickra", skip_from_py_object)]
#[derive(Clone, Default)]
struct PyTdOpen {
    inner: wc::TdOpen,
}

#[pymethods]
impl PyTdOpen {
    #[new]
    fn new() -> Self {
        Self {
            inner: wc::TdOpen::new(),
        }
    }
    fn update(&mut self, candle: &Bound<'_, PyAny>) -> PyResult<Option<f64>> {
        let c = extract_candle(candle)?;
        Ok(self.inner.update(c))
    }
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        open: PyReadonlyArray1<'py, f64>,
        high: PyReadonlyArray1<'py, f64>,
        low: PyReadonlyArray1<'py, f64>,
        close: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let o = open
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let h = high
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let l = low
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let c = close
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        if o.len() != h.len() || h.len() != l.len() || l.len() != c.len() {
            return Err(PyValueError::new_err(
                "open, high, low, close must be equal length",
            ));
        }
        let mut out = Vec::with_capacity(o.len());
        for i in 0..o.len() {
            let candle = wc::Candle::new(o[i], h[i], l[i], c[i], 0.0, 0).map_err(map_err)?;
            out.push(self.inner.update(candle).unwrap_or(f64::NAN));
        }
        Ok(out.into_pyarray(py))
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
}

// ============================== TD Risk Level ==============================

#[pyclass(name = "TDRiskLevel", module = "wickra._wickra", skip_from_py_object)]
#[derive(Clone)]
struct PyTdRiskLevel {
    inner: wc::TdRiskLevel,
}

#[pymethods]
impl PyTdRiskLevel {
    #[new]
    #[pyo3(signature = (lookback=4, target=9))]
    fn new(lookback: usize, target: usize) -> PyResult<Self> {
        Ok(Self {
            inner: wc::TdRiskLevel::new(lookback, target).map_err(map_err)?,
        })
    }
    /// Returns `(buy_risk, sell_risk)` (with `NaN` for unset levels) or
    /// `None` during warmup.
    fn update(&mut self, candle: &Bound<'_, PyAny>) -> PyResult<Option<(f64, f64)>> {
        let c = extract_candle(candle)?;
        Ok(self.inner.update(c).map(|o| (o.buy_risk, o.sell_risk)))
    }
    /// Batch returns shape `(n, 2)`: `[buy_risk, sell_risk]`.
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        high: PyReadonlyArray1<'py, f64>,
        low: PyReadonlyArray1<'py, f64>,
        close: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray2<f64>>> {
        let h = high
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let l = low
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let c = close
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        if h.len() != l.len() || l.len() != c.len() {
            return Err(PyValueError::new_err(
                "high, low, close must be equal length",
            ));
        }
        let n = h.len();
        let mut out = vec![f64::NAN; n * 2];
        for i in 0..n {
            let candle = wc::Candle::new(c[i], h[i], l[i], c[i], 0.0, 0).map_err(map_err)?;
            if let Some(o) = self.inner.update(candle) {
                out[i * 2] = o.buy_risk;
                out[i * 2 + 1] = o.sell_risk;
            }
        }
        Ok(numpy::ndarray::Array2::from_shape_vec((n, 2), out)
            .expect("shape consistent")
            .into_pyarray(py))
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
}

// ============================== Ehlers / Cycle (Family 10) ==============================

macro_rules! py_scalar_one_period {
    ($wrapper:ident, $py_name:literal, $rust_ty:ty) => {
        #[pyclass(name = $py_name, module = "wickra._wickra", skip_from_py_object)]
        #[derive(Clone)]
        struct $wrapper {
            inner: $rust_ty,
        }

        #[pymethods]
        impl $wrapper {
            #[new]
            fn new(period: usize) -> PyResult<Self> {
                Ok(Self {
                    inner: <$rust_ty>::new(period).map_err(map_err)?,
                })
            }
            fn update(&mut self, value: f64) -> Option<f64> {
                self.inner.update(value)
            }
            fn batch<'py>(
                &mut self,
                py: Python<'py>,
                prices: PyReadonlyArray1<'py, f64>,
            ) -> PyResult<Bound<'py, PyArray1<f64>>> {
                let slice = prices
                    .as_slice()
                    .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
                Ok(flatten(self.inner.batch(slice)).into_pyarray(py))
            }
            #[getter]
            fn period(&self) -> usize {
                self.inner.period()
            }
            #[getter]
            fn value(&self) -> Option<f64> {
                self.inner.value()
            }
            fn reset(&mut self) {
                self.inner.reset();
            }
            fn is_ready(&self) -> bool {
                self.inner.is_ready()
            }
            fn warmup_period(&self) -> usize {
                self.inner.warmup_period()
            }
            fn __repr__(&self) -> String {
                format!("{}(period={})", $py_name, self.inner.period())
            }
        }
    };
}

py_scalar_one_period!(PySuperSmoother, "SuperSmoother", wc::SuperSmoother);
py_scalar_one_period!(PyFisherTransform, "FisherTransform", wc::FisherTransform);
py_scalar_one_period!(PyDecycler, "Decycler", wc::Decycler);
py_scalar_one_period!(PyCenterOfGravity, "CenterOfGravity", wc::CenterOfGravity);
py_scalar_one_period!(PyCyberneticCycle, "CyberneticCycle", wc::CyberneticCycle);
py_scalar_one_period!(
    PyInstantaneousTrendline,
    "InstantaneousTrendline",
    wc::InstantaneousTrendline
);
py_scalar_one_period!(PyEhlersStochastic, "EhlersStochastic", wc::EhlersStochastic);

// --- InverseFisherTransform: single f64 `scale` param ---

#[pyclass(
    name = "InverseFisherTransform",
    module = "wickra._wickra",
    skip_from_py_object
)]
#[derive(Clone)]
struct PyInverseFisherTransform {
    inner: wc::InverseFisherTransform,
}

#[pymethods]
impl PyInverseFisherTransform {
    #[new]
    #[pyo3(signature = (scale=1.0))]
    fn new(scale: f64) -> PyResult<Self> {
        Ok(Self {
            inner: wc::InverseFisherTransform::new(scale).map_err(map_err)?,
        })
    }
    fn update(&mut self, value: f64) -> Option<f64> {
        self.inner.update(value)
    }
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        prices: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let slice = prices
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        Ok(flatten(self.inner.batch(slice)).into_pyarray(py))
    }
    #[getter]
    fn scale(&self) -> f64 {
        self.inner.scale()
    }
    #[getter]
    fn value(&self) -> Option<f64> {
        self.inner.value()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        format!("InverseFisherTransform(scale={})", self.inner.scale())
    }
}

// --- DecyclerOscillator: two-period ---

#[pyclass(
    name = "DecyclerOscillator",
    module = "wickra._wickra",
    skip_from_py_object
)]
#[derive(Clone)]
struct PyDecyclerOscillator {
    inner: wc::DecyclerOscillator,
}

#[pymethods]
impl PyDecyclerOscillator {
    #[new]
    fn new(fast: usize, slow: usize) -> PyResult<Self> {
        Ok(Self {
            inner: wc::DecyclerOscillator::new(fast, slow).map_err(map_err)?,
        })
    }
    fn update(&mut self, value: f64) -> Option<f64> {
        self.inner.update(value)
    }
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        prices: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let slice = prices
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        Ok(flatten(self.inner.batch(slice)).into_pyarray(py))
    }
    #[getter]
    fn periods(&self) -> (usize, usize) {
        self.inner.periods()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        let (f, s) = self.inner.periods();
        format!("DecyclerOscillator(fast={f}, slow={s})")
    }
}

// --- RoofingFilter: two-period (lp, hp) ---

#[pyclass(name = "RoofingFilter", module = "wickra._wickra", skip_from_py_object)]
#[derive(Clone)]
struct PyRoofingFilter {
    inner: wc::RoofingFilter,
}

#[pymethods]
impl PyRoofingFilter {
    #[new]
    #[pyo3(signature = (lp_period=10, hp_period=48))]
    fn new(lp_period: usize, hp_period: usize) -> PyResult<Self> {
        Ok(Self {
            inner: wc::RoofingFilter::new(lp_period, hp_period).map_err(map_err)?,
        })
    }
    fn update(&mut self, value: f64) -> Option<f64> {
        self.inner.update(value)
    }
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        prices: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let slice = prices
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        Ok(flatten(self.inner.batch(slice)).into_pyarray(py))
    }
    #[getter]
    fn periods(&self) -> (usize, usize) {
        self.inner.periods()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        let (lp, hp) = self.inner.periods();
        format!("RoofingFilter(lp_period={lp}, hp_period={hp})")
    }
}

// --- EmpiricalModeDecomposition: period + fraction ---

#[pyclass(
    name = "EmpiricalModeDecomposition",
    module = "wickra._wickra",
    skip_from_py_object
)]
#[derive(Clone)]
struct PyEmd {
    inner: wc::EmpiricalModeDecomposition,
}

#[pymethods]
impl PyEmd {
    #[new]
    #[pyo3(signature = (period=20, fraction=0.5))]
    fn new(period: usize, fraction: f64) -> PyResult<Self> {
        Ok(Self {
            inner: wc::EmpiricalModeDecomposition::new(period, fraction).map_err(map_err)?,
        })
    }
    fn update(&mut self, value: f64) -> Option<f64> {
        self.inner.update(value)
    }
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        prices: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let slice = prices
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        Ok(flatten(self.inner.batch(slice)).into_pyarray(py))
    }
    #[getter]
    fn period(&self) -> usize {
        self.inner.period()
    }
    #[getter]
    fn fraction(&self) -> f64 {
        self.inner.fraction()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        format!(
            "EmpiricalModeDecomposition(period={}, fraction={})",
            self.inner.period(),
            self.inner.fraction()
        )
    }
}

// --- HilbertDominantCycle / SineWave / AdaptiveCycle: parameterless ---

macro_rules! py_no_params_scalar {
    ($wrapper:ident, $py_name:literal, $rust_ty:ty) => {
        #[pyclass(name = $py_name, module = "wickra._wickra", skip_from_py_object)]
        #[derive(Clone)]
        struct $wrapper {
            inner: $rust_ty,
        }

        #[pymethods]
        impl $wrapper {
            #[new]
            fn new() -> Self {
                Self {
                    inner: <$rust_ty>::new(),
                }
            }
            fn update(&mut self, value: f64) -> Option<f64> {
                self.inner.update(value)
            }
            fn batch<'py>(
                &mut self,
                py: Python<'py>,
                prices: PyReadonlyArray1<'py, f64>,
            ) -> PyResult<Bound<'py, PyArray1<f64>>> {
                let slice = prices
                    .as_slice()
                    .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
                Ok(flatten(self.inner.batch(slice)).into_pyarray(py))
            }
            #[getter]
            fn value(&self) -> Option<f64> {
                self.inner.value()
            }
            fn reset(&mut self) {
                self.inner.reset();
            }
            fn is_ready(&self) -> bool {
                self.inner.is_ready()
            }
            fn warmup_period(&self) -> usize {
                self.inner.warmup_period()
            }
            fn __repr__(&self) -> String {
                format!("{}()", $py_name)
            }
        }
    };
}

py_no_params_scalar!(
    PyHilbertDominantCycle,
    "HilbertDominantCycle",
    wc::HilbertDominantCycle
);
py_no_params_scalar!(PyAdaptiveCycle, "AdaptiveCycle", wc::AdaptiveCycle);

// SineWave needs a `lead` accessor in addition to scalar value, but otherwise
// matches the parameterless surface.
#[pyclass(name = "SineWave", module = "wickra._wickra", skip_from_py_object)]
#[derive(Clone)]
struct PySineWave {
    inner: wc::SineWave,
}

#[pymethods]
impl PySineWave {
    #[new]
    fn new() -> Self {
        Self {
            inner: wc::SineWave::new(),
        }
    }
    fn update(&mut self, value: f64) -> Option<f64> {
        self.inner.update(value)
    }
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        prices: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let slice = prices
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        Ok(flatten(self.inner.batch(slice)).into_pyarray(py))
    }
    #[getter]
    fn value(&self) -> Option<f64> {
        self.inner.value()
    }
    #[getter]
    fn lead(&self) -> f64 {
        self.inner.lead()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        "SineWave()".to_string()
    }
}

// --- MAMA: multi-output (mama, fama), shape (n, 2) ---

#[pyclass(name = "MAMA", module = "wickra._wickra", skip_from_py_object)]
#[derive(Clone)]
struct PyMama {
    inner: wc::Mama,
}

#[pymethods]
impl PyMama {
    #[new]
    #[pyo3(signature = (fast_limit=0.5, slow_limit=0.05))]
    fn new(fast_limit: f64, slow_limit: f64) -> PyResult<Self> {
        Ok(Self {
            inner: wc::Mama::new(fast_limit, slow_limit).map_err(map_err)?,
        })
    }
    /// Returns `(mama, fama)` or `None` during warmup.
    fn update(&mut self, value: f64) -> Option<(f64, f64)> {
        self.inner.update(value).map(|o| (o.mama, o.fama))
    }
    /// Batch returns shape `(n, 2)` columns `[mama, fama]`. Warmup rows NaN.
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        prices: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray2<f64>>> {
        let slice = prices
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let n = slice.len();
        let mut out = vec![f64::NAN; n * 2];
        for (i, p) in slice.iter().enumerate() {
            if let Some(o) = self.inner.update(*p) {
                out[i * 2] = o.mama;
                out[i * 2 + 1] = o.fama;
            }
        }
        Ok(numpy::ndarray::Array2::from_shape_vec((n, 2), out)
            .expect("shape consistent")
            .into_pyarray(py))
    }
    #[getter]
    fn limits(&self) -> (f64, f64) {
        self.inner.limits()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        let (f, s) = self.inner.limits();
        format!("MAMA(fast_limit={f}, slow_limit={s})")
    }
}

// --- FAMA: scalar wrapper exposing only the fama line ---

#[pyclass(name = "FAMA", module = "wickra._wickra", skip_from_py_object)]
#[derive(Clone)]
struct PyFama {
    inner: wc::Fama,
}

#[pymethods]
impl PyFama {
    #[new]
    #[pyo3(signature = (fast_limit=0.5, slow_limit=0.05))]
    fn new(fast_limit: f64, slow_limit: f64) -> PyResult<Self> {
        Ok(Self {
            inner: wc::Fama::new(fast_limit, slow_limit).map_err(map_err)?,
        })
    }
    fn update(&mut self, value: f64) -> Option<f64> {
        self.inner.update(value)
    }
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        prices: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let slice = prices
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        Ok(flatten(self.inner.batch(slice)).into_pyarray(py))
    }
    #[getter]
    fn limits(&self) -> (f64, f64) {
        self.inner.limits()
    }
    #[getter]
    fn value(&self) -> Option<f64> {
        self.inner.value()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        let (f, s) = self.inner.limits();
        format!("FAMA(fast_limit={f}, slow_limit={s})")
    }
}

// ============================== Ichimoku ==============================

#[pyclass(name = "Ichimoku", module = "wickra._wickra", skip_from_py_object)]
#[derive(Clone)]
struct PyIchimoku {
    inner: wc::Ichimoku,
}

#[pymethods]
impl PyIchimoku {
    #[new]
    #[pyo3(signature = (tenkan_period=9, kijun_period=26, senkou_b_period=52, displacement=26))]
    fn new(
        tenkan_period: usize,
        kijun_period: usize,
        senkou_b_period: usize,
        displacement: usize,
    ) -> PyResult<Self> {
        Ok(Self {
            inner: wc::Ichimoku::new(tenkan_period, kijun_period, senkou_b_period, displacement)
                .map_err(map_err)?,
        })
    }
    /// Returns `(tenkan, kijun, senkou_a, senkou_b, chikou)` as a 5-tuple
    /// where each element is `float` or `None`.
    fn update(&mut self, candle: &Bound<'_, PyAny>) -> PyResult<Option<IchimokuLines>> {
        let c = extract_candle(candle)?;
        Ok(self
            .inner
            .update(c)
            .map(|o| (o.tenkan, o.kijun, o.senkou_a, o.senkou_b, o.chikou)))
    }
    /// Batch over high/low/close numpy columns. Returns shape `(n, 5)` with
    /// columns `[tenkan, kijun, senkou_a, senkou_b, chikou]`. Any cell whose
    /// underlying line is undefined at that bar is `NaN`.
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        high: PyReadonlyArray1<'py, f64>,
        low: PyReadonlyArray1<'py, f64>,
        close: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray2<f64>>> {
        let h = high
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let l = low
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let c = close
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        if h.len() != l.len() || l.len() != c.len() {
            return Err(PyValueError::new_err(
                "high, low, close must be equal length",
            ));
        }
        let n = h.len();
        let mut out = vec![f64::NAN; n * 5];
        for i in 0..n {
            let candle = wc::Candle::new(c[i], h[i], l[i], c[i], 0.0, 0).map_err(map_err)?;
            if let Some(o) = self.inner.update(candle) {
                if let Some(v) = o.tenkan {
                    out[i * 5] = v;
                }
                if let Some(v) = o.kijun {
                    out[i * 5 + 1] = v;
                }
                if let Some(v) = o.senkou_a {
                    out[i * 5 + 2] = v;
                }
                if let Some(v) = o.senkou_b {
                    out[i * 5 + 3] = v;
                }
                if let Some(v) = o.chikou {
                    out[i * 5 + 4] = v;
                }
            }
        }
        Ok(numpy::ndarray::Array2::from_shape_vec((n, 5), out)
            .expect("shape consistent")
            .into_pyarray(py))
    }
    #[getter]
    fn periods(&self) -> (usize, usize, usize, usize) {
        self.inner.periods()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        let (t, k, sb, d) = self.inner.periods();
        format!(
            "Ichimoku(tenkan_period={t}, kijun_period={k}, senkou_b_period={sb}, displacement={d})"
        )
    }
}

// ============================== Heikin-Ashi ==============================

#[pyclass(name = "HeikinAshi", module = "wickra._wickra", skip_from_py_object)]
#[derive(Clone, Default)]
struct PyHeikinAshi {
    inner: wc::HeikinAshi,
}

#[pymethods]
impl PyHeikinAshi {
    #[new]
    fn new() -> Self {
        Self::default()
    }
    /// Returns `(ha_open, ha_high, ha_low, ha_close)`.
    fn update(&mut self, candle: &Bound<'_, PyAny>) -> PyResult<Option<(f64, f64, f64, f64)>> {
        let c = extract_candle(candle)?;
        Ok(self
            .inner
            .update(c)
            .map(|o| (o.open, o.high, o.low, o.close)))
    }
    /// Batch over OHLC numpy columns. Returns shape `(n, 4)` with columns
    /// `[ha_open, ha_high, ha_low, ha_close]`.
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        open: PyReadonlyArray1<'py, f64>,
        high: PyReadonlyArray1<'py, f64>,
        low: PyReadonlyArray1<'py, f64>,
        close: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray2<f64>>> {
        let o = open
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let h = high
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let l = low
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let c = close
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        if o.len() != h.len() || h.len() != l.len() || l.len() != c.len() {
            return Err(PyValueError::new_err(
                "open, high, low, close must be equal length",
            ));
        }
        let n = o.len();
        let mut out = vec![f64::NAN; n * 4];
        for i in 0..n {
            let candle = wc::Candle::new(o[i], h[i], l[i], c[i], 0.0, 0).map_err(map_err)?;
            if let Some(v) = self.inner.update(candle) {
                out[i * 4] = v.open;
                out[i * 4 + 1] = v.high;
                out[i * 4 + 2] = v.low;
                out[i * 4 + 3] = v.close;
            }
        }
        Ok(numpy::ndarray::Array2::from_shape_vec((n, 4), out)
            .expect("shape consistent")
            .into_pyarray(py))
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        "HeikinAshi()".to_string()
    }
}

#[pyclass(name = "Variance", module = "wickra._wickra", skip_from_py_object)]
#[derive(Clone)]
struct PyVariance {
    inner: wc::Variance,
}

#[pymethods]
impl PyVariance {
    #[new]
    #[pyo3(signature = (period=20))]
    fn new(period: usize) -> PyResult<Self> {
        Ok(Self {
            inner: wc::Variance::new(period).map_err(map_err)?,
        })
    }
    fn update(&mut self, value: f64) -> Option<f64> {
        self.inner.update(value)
    }
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        prices: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let s = prices
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        Ok(flatten(self.inner.batch(s)).into_pyarray(py))
    }
    #[getter]
    fn period(&self) -> usize {
        self.inner.period()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        format!("Variance(period={})", self.inner.period())
    }
}

// ============================== CoefficientOfVariation ==============================

#[pyclass(
    name = "CoefficientOfVariation",
    module = "wickra._wickra",
    skip_from_py_object
)]
#[derive(Clone)]
struct PyCoefficientOfVariation {
    inner: wc::CoefficientOfVariation,
}

#[pymethods]
impl PyCoefficientOfVariation {
    #[new]
    #[pyo3(signature = (period=20))]
    fn new(period: usize) -> PyResult<Self> {
        Ok(Self {
            inner: wc::CoefficientOfVariation::new(period).map_err(map_err)?,
        })
    }
    fn update(&mut self, value: f64) -> Option<f64> {
        self.inner.update(value)
    }
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        prices: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let s = prices
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        Ok(flatten(self.inner.batch(s)).into_pyarray(py))
    }
    #[getter]
    fn period(&self) -> usize {
        self.inner.period()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        format!("CoefficientOfVariation(period={})", self.inner.period())
    }
}

// ============================== Skewness ==============================

#[pyclass(name = "Skewness", module = "wickra._wickra", skip_from_py_object)]
#[derive(Clone)]
struct PySkewness {
    inner: wc::Skewness,
}

#[pymethods]
impl PySkewness {
    #[new]
    #[pyo3(signature = (period=20))]
    fn new(period: usize) -> PyResult<Self> {
        Ok(Self {
            inner: wc::Skewness::new(period).map_err(map_err)?,
        })
    }
    fn update(&mut self, value: f64) -> Option<f64> {
        self.inner.update(value)
    }
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        prices: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let s = prices
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        Ok(flatten(self.inner.batch(s)).into_pyarray(py))
    }
    #[getter]
    fn period(&self) -> usize {
        self.inner.period()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        format!("Skewness(period={})", self.inner.period())
    }
}

// ============================== Kurtosis ==============================

#[pyclass(name = "Kurtosis", module = "wickra._wickra", skip_from_py_object)]
#[derive(Clone)]
struct PyKurtosis {
    inner: wc::Kurtosis,
}

#[pymethods]
impl PyKurtosis {
    #[new]
    #[pyo3(signature = (period=20))]
    fn new(period: usize) -> PyResult<Self> {
        Ok(Self {
            inner: wc::Kurtosis::new(period).map_err(map_err)?,
        })
    }
    fn update(&mut self, value: f64) -> Option<f64> {
        self.inner.update(value)
    }
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        prices: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let s = prices
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        Ok(flatten(self.inner.batch(s)).into_pyarray(py))
    }
    #[getter]
    fn period(&self) -> usize {
        self.inner.period()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        format!("Kurtosis(period={})", self.inner.period())
    }
}

// ============================== StandardError ==============================

#[pyclass(name = "StandardError", module = "wickra._wickra", skip_from_py_object)]
#[derive(Clone)]
struct PyStandardError {
    inner: wc::StandardError,
}

#[pymethods]
impl PyStandardError {
    #[new]
    #[pyo3(signature = (period=14))]
    fn new(period: usize) -> PyResult<Self> {
        Ok(Self {
            inner: wc::StandardError::new(period).map_err(map_err)?,
        })
    }
    fn update(&mut self, value: f64) -> Option<f64> {
        self.inner.update(value)
    }
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        prices: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let s = prices
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        Ok(flatten(self.inner.batch(s)).into_pyarray(py))
    }
    #[getter]
    fn period(&self) -> usize {
        self.inner.period()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        format!("StandardError(period={})", self.inner.period())
    }
}

// ============================== DetrendedStdDev ==============================

#[pyclass(
    name = "DetrendedStdDev",
    module = "wickra._wickra",
    skip_from_py_object
)]
#[derive(Clone)]
struct PyDetrendedStdDev {
    inner: wc::DetrendedStdDev,
}

#[pymethods]
impl PyDetrendedStdDev {
    #[new]
    #[pyo3(signature = (period=14))]
    fn new(period: usize) -> PyResult<Self> {
        Ok(Self {
            inner: wc::DetrendedStdDev::new(period).map_err(map_err)?,
        })
    }
    fn update(&mut self, value: f64) -> Option<f64> {
        self.inner.update(value)
    }
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        prices: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let s = prices
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        Ok(flatten(self.inner.batch(s)).into_pyarray(py))
    }
    #[getter]
    fn period(&self) -> usize {
        self.inner.period()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        format!("DetrendedStdDev(period={})", self.inner.period())
    }
}

// ============================== RSquared ==============================

#[pyclass(name = "RSquared", module = "wickra._wickra", skip_from_py_object)]
#[derive(Clone)]
struct PyRSquared {
    inner: wc::RSquared,
}

#[pymethods]
impl PyRSquared {
    #[new]
    #[pyo3(signature = (period=14))]
    fn new(period: usize) -> PyResult<Self> {
        Ok(Self {
            inner: wc::RSquared::new(period).map_err(map_err)?,
        })
    }
    fn update(&mut self, value: f64) -> Option<f64> {
        self.inner.update(value)
    }
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        prices: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let s = prices
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        Ok(flatten(self.inner.batch(s)).into_pyarray(py))
    }
    #[getter]
    fn period(&self) -> usize {
        self.inner.period()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        format!("RSquared(period={})", self.inner.period())
    }
}

// ============================== Autocorrelation ==============================

#[pyclass(
    name = "Autocorrelation",
    module = "wickra._wickra",
    skip_from_py_object
)]
#[derive(Clone)]
struct PyAutocorrelation {
    inner: wc::Autocorrelation,
}

#[pymethods]
impl PyAutocorrelation {
    #[new]
    #[pyo3(signature = (period=20, lag=1))]
    fn new(period: usize, lag: usize) -> PyResult<Self> {
        Ok(Self {
            inner: wc::Autocorrelation::new(period, lag).map_err(map_err)?,
        })
    }
    fn update(&mut self, value: f64) -> Option<f64> {
        self.inner.update(value)
    }
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        prices: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let s = prices
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        Ok(flatten(self.inner.batch(s)).into_pyarray(py))
    }
    #[getter]
    fn period(&self) -> usize {
        self.inner.period()
    }
    #[getter]
    fn lag(&self) -> usize {
        self.inner.lag()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        format!(
            "Autocorrelation(period={}, lag={})",
            self.inner.period(),
            self.inner.lag()
        )
    }
}

// ============================== MedianAbsoluteDeviation ==============================

#[pyclass(
    name = "MedianAbsoluteDeviation",
    module = "wickra._wickra",
    skip_from_py_object
)]
#[derive(Clone)]
struct PyMedianAbsoluteDeviation {
    inner: wc::MedianAbsoluteDeviation,
}

#[pymethods]
impl PyMedianAbsoluteDeviation {
    #[new]
    #[pyo3(signature = (period=20))]
    fn new(period: usize) -> PyResult<Self> {
        Ok(Self {
            inner: wc::MedianAbsoluteDeviation::new(period).map_err(map_err)?,
        })
    }
    fn update(&mut self, value: f64) -> Option<f64> {
        self.inner.update(value)
    }
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        prices: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let s = prices
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        Ok(flatten(self.inner.batch(s)).into_pyarray(py))
    }
    #[getter]
    fn period(&self) -> usize {
        self.inner.period()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        format!("MedianAbsoluteDeviation(period={})", self.inner.period())
    }
}

// ============================== HurstExponent ==============================

#[pyclass(name = "HurstExponent", module = "wickra._wickra", skip_from_py_object)]
#[derive(Clone)]
struct PyHurstExponent {
    inner: wc::HurstExponent,
}

#[pymethods]
impl PyHurstExponent {
    #[new]
    #[pyo3(signature = (period=100, chunks=4))]
    fn new(period: usize, chunks: usize) -> PyResult<Self> {
        Ok(Self {
            inner: wc::HurstExponent::new(period, chunks).map_err(map_err)?,
        })
    }
    fn update(&mut self, value: f64) -> Option<f64> {
        self.inner.update(value)
    }
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        prices: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let s = prices
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        Ok(flatten(self.inner.batch(s)).into_pyarray(py))
    }
    #[getter]
    fn period(&self) -> usize {
        self.inner.period()
    }
    #[getter]
    fn chunks(&self) -> usize {
        self.inner.chunks()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        format!(
            "HurstExponent(period={}, chunks={})",
            self.inner.period(),
            self.inner.chunks()
        )
    }
}

// ============================== PearsonCorrelation ==============================

#[pyclass(
    name = "PearsonCorrelation",
    module = "wickra._wickra",
    skip_from_py_object
)]
#[derive(Clone)]
struct PyPearsonCorrelation {
    inner: wc::PearsonCorrelation,
}

#[pymethods]
impl PyPearsonCorrelation {
    #[new]
    #[pyo3(signature = (period=20))]
    fn new(period: usize) -> PyResult<Self> {
        Ok(Self {
            inner: wc::PearsonCorrelation::new(period).map_err(map_err)?,
        })
    }
    fn update(&mut self, x: f64, y: f64) -> Option<f64> {
        self.inner.update((x, y))
    }
    /// Batch over two equally-sized numpy arrays.
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        x: PyReadonlyArray1<'py, f64>,
        y: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let xs = x
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let ys = y
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        if xs.len() != ys.len() {
            return Err(PyValueError::new_err("x and y must be equal length"));
        }
        let mut out = Vec::with_capacity(xs.len());
        for i in 0..xs.len() {
            out.push(self.inner.update((xs[i], ys[i])).unwrap_or(f64::NAN));
        }
        Ok(out.into_pyarray(py))
    }
    #[getter]
    fn period(&self) -> usize {
        self.inner.period()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        format!("PearsonCorrelation(period={})", self.inner.period())
    }
}

// ============================== Beta ==============================

#[pyclass(name = "Beta", module = "wickra._wickra", skip_from_py_object)]
#[derive(Clone)]
struct PyBeta {
    inner: wc::Beta,
}

#[pymethods]
impl PyBeta {
    #[new]
    #[pyo3(signature = (period=20))]
    fn new(period: usize) -> PyResult<Self> {
        Ok(Self {
            inner: wc::Beta::new(period).map_err(map_err)?,
        })
    }
    fn update(&mut self, asset: f64, benchmark: f64) -> Option<f64> {
        self.inner.update((asset, benchmark))
    }
    /// Batch over two equally-sized numpy arrays: asset and benchmark.
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        asset: PyReadonlyArray1<'py, f64>,
        benchmark: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let a = asset
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let b = benchmark
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        if a.len() != b.len() {
            return Err(PyValueError::new_err(
                "asset and benchmark must be equal length",
            ));
        }
        let mut out = Vec::with_capacity(a.len());
        for i in 0..a.len() {
            out.push(self.inner.update((a[i], b[i])).unwrap_or(f64::NAN));
        }
        Ok(out.into_pyarray(py))
    }
    #[getter]
    fn period(&self) -> usize {
        self.inner.period()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        format!("Beta(period={})", self.inner.period())
    }
}

// ============================== PairwiseBeta ==============================

#[pyclass(name = "PairwiseBeta", module = "wickra._wickra", skip_from_py_object)]
#[derive(Clone)]
struct PyPairwiseBeta {
    inner: wc::PairwiseBeta,
}

#[pymethods]
impl PyPairwiseBeta {
    #[new]
    #[pyo3(signature = (period=20))]
    fn new(period: usize) -> PyResult<Self> {
        Ok(Self {
            inner: wc::PairwiseBeta::new(period).map_err(map_err)?,
        })
    }
    fn update(&mut self, a: f64, b: f64) -> Option<f64> {
        self.inner.update((a, b))
    }
    /// Batch over two equally-sized numpy arrays of prices: `a` and `b`.
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        a: PyReadonlyArray1<'py, f64>,
        b: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let xs = a
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let ys = b
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        if xs.len() != ys.len() {
            return Err(PyValueError::new_err("a and b must be equal length"));
        }
        let mut out = Vec::with_capacity(xs.len());
        for i in 0..xs.len() {
            out.push(self.inner.update((xs[i], ys[i])).unwrap_or(f64::NAN));
        }
        Ok(out.into_pyarray(py))
    }
    #[getter]
    fn period(&self) -> usize {
        self.inner.period()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        format!("PairwiseBeta(period={})", self.inner.period())
    }
}

// ============================== PairSpreadZScore ==============================

#[pyclass(
    name = "PairSpreadZScore",
    module = "wickra._wickra",
    skip_from_py_object
)]
#[derive(Clone)]
struct PyPairSpreadZScore {
    inner: wc::PairSpreadZScore,
}

#[pymethods]
impl PyPairSpreadZScore {
    #[new]
    #[pyo3(signature = (beta_period=20, z_period=20))]
    fn new(beta_period: usize, z_period: usize) -> PyResult<Self> {
        Ok(Self {
            inner: wc::PairSpreadZScore::new(beta_period, z_period).map_err(map_err)?,
        })
    }
    fn update(&mut self, a: f64, b: f64) -> Option<f64> {
        self.inner.update((a, b))
    }
    /// Batch over two equally-sized numpy arrays of prices: `a` and `b`.
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        a: PyReadonlyArray1<'py, f64>,
        b: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let xs = a
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let ys = b
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        if xs.len() != ys.len() {
            return Err(PyValueError::new_err("a and b must be equal length"));
        }
        let mut out = Vec::with_capacity(xs.len());
        for i in 0..xs.len() {
            out.push(self.inner.update((xs[i], ys[i])).unwrap_or(f64::NAN));
        }
        Ok(out.into_pyarray(py))
    }
    #[getter]
    fn beta_period(&self) -> usize {
        self.inner.beta_period()
    }
    #[getter]
    fn z_period(&self) -> usize {
        self.inner.z_period()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        format!(
            "PairSpreadZScore(beta_period={}, z_period={})",
            self.inner.beta_period(),
            self.inner.z_period()
        )
    }
}

// ============================== LeadLagCrossCorrelation ==============================

#[pyclass(
    name = "LeadLagCrossCorrelation",
    module = "wickra._wickra",
    skip_from_py_object
)]
#[derive(Clone)]
struct PyLeadLagCrossCorrelation {
    inner: wc::LeadLagCrossCorrelation,
}

#[pymethods]
impl PyLeadLagCrossCorrelation {
    #[new]
    #[pyo3(signature = (window=20, max_lag=10))]
    fn new(window: usize, max_lag: usize) -> PyResult<Self> {
        Ok(Self {
            inner: wc::LeadLagCrossCorrelation::new(window, max_lag).map_err(map_err)?,
        })
    }
    /// Returns `(lag, correlation)` or `None` during warmup. A positive lag
    /// means `a` leads `b`.
    fn update(&mut self, a: f64, b: f64) -> Option<(i64, f64)> {
        self.inner.update((a, b)).map(|o| (o.lag, o.correlation))
    }
    /// Batch over two equally-sized numpy arrays. Returns a 2D array of shape
    /// `(n, 2)` with columns `[lag, correlation]`. Warmup rows are NaN.
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        a: PyReadonlyArray1<'py, f64>,
        b: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray2<f64>>> {
        let xs = a
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let ys = b
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        if xs.len() != ys.len() {
            return Err(PyValueError::new_err("a and b must be equal length"));
        }
        let n = xs.len();
        let mut out = vec![f64::NAN; n * 2];
        for i in 0..n {
            if let Some(o) = self.inner.update((xs[i], ys[i])) {
                out[i * 2] = o.lag as f64;
                out[i * 2 + 1] = o.correlation;
            }
        }
        Ok(numpy::ndarray::Array2::from_shape_vec((n, 2), out)
            .expect("shape consistent")
            .into_pyarray(py))
    }
    #[getter]
    fn window(&self) -> usize {
        self.inner.window()
    }
    #[getter]
    fn max_lag(&self) -> usize {
        self.inner.max_lag()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        format!(
            "LeadLagCrossCorrelation(window={}, max_lag={})",
            self.inner.window(),
            self.inner.max_lag()
        )
    }
}

// ============================== Cointegration ==============================

#[pyclass(name = "Cointegration", module = "wickra._wickra", skip_from_py_object)]
#[derive(Clone)]
struct PyCointegration {
    inner: wc::Cointegration,
}

#[pymethods]
impl PyCointegration {
    #[new]
    #[pyo3(signature = (period=30, adf_lags=1))]
    fn new(period: usize, adf_lags: usize) -> PyResult<Self> {
        Ok(Self {
            inner: wc::Cointegration::new(period, adf_lags).map_err(map_err)?,
        })
    }
    /// Returns `(hedge_ratio, spread, adf_stat)` or `None` during warmup.
    fn update(&mut self, a: f64, b: f64) -> Option<(f64, f64, f64)> {
        self.inner
            .update((a, b))
            .map(|o| (o.hedge_ratio, o.spread, o.adf_stat))
    }
    /// Batch over two equally-sized numpy arrays. Returns a 2D array of shape
    /// `(n, 3)` with columns `[hedge_ratio, spread, adf_stat]`. Warmup rows are
    /// NaN.
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        a: PyReadonlyArray1<'py, f64>,
        b: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray2<f64>>> {
        let xs = a
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let ys = b
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        if xs.len() != ys.len() {
            return Err(PyValueError::new_err("a and b must be equal length"));
        }
        let n = xs.len();
        let mut out = vec![f64::NAN; n * 3];
        for i in 0..n {
            if let Some(o) = self.inner.update((xs[i], ys[i])) {
                out[i * 3] = o.hedge_ratio;
                out[i * 3 + 1] = o.spread;
                out[i * 3 + 2] = o.adf_stat;
            }
        }
        Ok(numpy::ndarray::Array2::from_shape_vec((n, 3), out)
            .expect("shape consistent")
            .into_pyarray(py))
    }
    #[getter]
    fn period(&self) -> usize {
        self.inner.period()
    }
    #[getter]
    fn adf_lags(&self) -> usize {
        self.inner.adf_lags()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        format!(
            "Cointegration(period={}, adf_lags={})",
            self.inner.period(),
            self.inner.adf_lags()
        )
    }
}

// ============================== RelativeStrengthAB ==============================

#[pyclass(
    name = "RelativeStrengthAB",
    module = "wickra._wickra",
    skip_from_py_object
)]
#[derive(Clone)]
struct PyRelativeStrengthAB {
    inner: wc::RelativeStrengthAB,
}

#[pymethods]
impl PyRelativeStrengthAB {
    #[new]
    #[pyo3(signature = (ma_period=20, rsi_period=14))]
    fn new(ma_period: usize, rsi_period: usize) -> PyResult<Self> {
        Ok(Self {
            inner: wc::RelativeStrengthAB::new(ma_period, rsi_period).map_err(map_err)?,
        })
    }
    /// Returns `(ratio, ratio_ma, ratio_rsi)` or `None` during warmup.
    fn update(&mut self, a: f64, b: f64) -> Option<(f64, f64, f64)> {
        self.inner
            .update((a, b))
            .map(|o| (o.ratio, o.ratio_ma, o.ratio_rsi))
    }
    /// Batch over two equally-sized numpy arrays. Returns a 2D array of shape
    /// `(n, 3)` with columns `[ratio, ratio_ma, ratio_rsi]`. Warmup rows are
    /// NaN.
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        a: PyReadonlyArray1<'py, f64>,
        b: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray2<f64>>> {
        let xs = a
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let ys = b
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        if xs.len() != ys.len() {
            return Err(PyValueError::new_err("a and b must be equal length"));
        }
        let n = xs.len();
        let mut out = vec![f64::NAN; n * 3];
        for i in 0..n {
            if let Some(o) = self.inner.update((xs[i], ys[i])) {
                out[i * 3] = o.ratio;
                out[i * 3 + 1] = o.ratio_ma;
                out[i * 3 + 2] = o.ratio_rsi;
            }
        }
        Ok(numpy::ndarray::Array2::from_shape_vec((n, 3), out)
            .expect("shape consistent")
            .into_pyarray(py))
    }
    #[getter]
    fn ma_period(&self) -> usize {
        self.inner.ma_period()
    }
    #[getter]
    fn rsi_period(&self) -> usize {
        self.inner.rsi_period()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        format!(
            "RelativeStrengthAB(ma_period={}, rsi_period={})",
            self.inner.ma_period(),
            self.inner.rsi_period()
        )
    }
}

// ============================== SpearmanCorrelation ==============================

#[pyclass(
    name = "SpearmanCorrelation",
    module = "wickra._wickra",
    skip_from_py_object
)]
#[derive(Clone)]
struct PySpearmanCorrelation {
    inner: wc::SpearmanCorrelation,
}

#[pymethods]
impl PySpearmanCorrelation {
    #[new]
    #[pyo3(signature = (period=20))]
    fn new(period: usize) -> PyResult<Self> {
        Ok(Self {
            inner: wc::SpearmanCorrelation::new(period).map_err(map_err)?,
        })
    }
    fn update(&mut self, x: f64, y: f64) -> Option<f64> {
        self.inner.update((x, y))
    }
    /// Batch over two equally-sized numpy arrays.
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        x: PyReadonlyArray1<'py, f64>,
        y: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let xs = x
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let ys = y
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        if xs.len() != ys.len() {
            return Err(PyValueError::new_err("x and y must be equal length"));
        }
        let mut out = Vec::with_capacity(xs.len());
        for i in 0..xs.len() {
            out.push(self.inner.update((xs[i], ys[i])).unwrap_or(f64::NAN));
        }
        Ok(out.into_pyarray(py))
    }
    #[getter]
    fn period(&self) -> usize {
        self.inner.period()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        format!("SpearmanCorrelation(period={})", self.inner.period())
    }
}

// ============================== ValueArea ==============================

#[pyclass(name = "ValueArea", module = "wickra._wickra", skip_from_py_object)]
#[derive(Clone)]
struct PyValueArea {
    inner: wc::ValueArea,
}

#[pymethods]
impl PyValueArea {
    #[new]
    #[pyo3(signature = (period=20, bin_count=50, value_area_pct=0.70))]
    fn new(period: usize, bin_count: usize, value_area_pct: f64) -> PyResult<Self> {
        Ok(Self {
            inner: wc::ValueArea::new(period, bin_count, value_area_pct).map_err(map_err)?,
        })
    }
    fn update(&mut self, candle: &Bound<'_, PyAny>) -> PyResult<Option<(f64, f64, f64)>> {
        let c = extract_candle(candle)?;
        Ok(self.inner.update(c).map(|o| (o.poc, o.vah, o.val)))
    }
    /// Batch over numpy columns high, low, volume. Returns shape `(n, 3)`
    /// with columns `[poc, vah, val]`; warmup rows are `NaN`.
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        high: PyReadonlyArray1<'py, f64>,
        low: PyReadonlyArray1<'py, f64>,
        volume: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray2<f64>>> {
        let h = high
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let l = low
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let v = volume
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        if h.len() != l.len() || l.len() != v.len() {
            return Err(PyValueError::new_err(
                "high, low, volume must be equal length",
            ));
        }
        let n = h.len();
        let mut out = vec![f64::NAN; n * 3];
        for i in 0..n {
            // open / close pinned to the midpoint so the candle validates.
            let mid = f64::midpoint(h[i], l[i]);
            let candle = wc::Candle::new(mid, h[i], l[i], mid, v[i], 0).map_err(map_err)?;
            if let Some(o) = self.inner.update(candle) {
                out[i * 3] = o.poc;
                out[i * 3 + 1] = o.vah;
                out[i * 3 + 2] = o.val;
            }
        }
        Ok(numpy::ndarray::Array2::from_shape_vec((n, 3), out)
            .expect("shape consistent")
            .into_pyarray(py))
    }
    #[getter]
    fn params(&self) -> (usize, usize, f64) {
        self.inner.params()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        let (period, bin_count, pct) = self.inner.params();
        format!("ValueArea(period={period}, bin_count={bin_count}, value_area_pct={pct})")
    }
}

// ============================== InitialBalance ==============================

#[pyclass(
    name = "InitialBalance",
    module = "wickra._wickra",
    skip_from_py_object
)]
#[derive(Clone)]
struct PyInitialBalance {
    inner: wc::InitialBalance,
}

#[pymethods]
impl PyInitialBalance {
    #[new]
    #[pyo3(signature = (period=12))]
    fn new(period: usize) -> PyResult<Self> {
        Ok(Self {
            inner: wc::InitialBalance::new(period).map_err(map_err)?,
        })
    }
    fn update(&mut self, candle: &Bound<'_, PyAny>) -> PyResult<Option<(f64, f64)>> {
        let c = extract_candle(candle)?;
        Ok(self.inner.update(c).map(|o| (o.high, o.low)))
    }
    /// Batch over numpy columns high, low. Returns shape `(n, 2)` with
    /// columns `[high, low]`.
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        high: PyReadonlyArray1<'py, f64>,
        low: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray2<f64>>> {
        let h = high
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let l = low
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        if h.len() != l.len() {
            return Err(PyValueError::new_err("high and low must be equal length"));
        }
        let n = h.len();
        let mut out = vec![f64::NAN; n * 2];
        for i in 0..n {
            let mid = f64::midpoint(h[i], l[i]);
            let candle = wc::Candle::new(mid, h[i], l[i], mid, 0.0, 0).map_err(map_err)?;
            if let Some(o) = self.inner.update(candle) {
                out[i * 2] = o.high;
                out[i * 2 + 1] = o.low;
            }
        }
        Ok(numpy::ndarray::Array2::from_shape_vec((n, 2), out)
            .expect("shape consistent")
            .into_pyarray(py))
    }
    #[getter]
    fn period(&self) -> usize {
        self.inner.period()
    }
    fn is_locked(&self) -> bool {
        self.inner.is_locked()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        format!("InitialBalance(period={})", self.inner.period())
    }
}

// ============================== OpeningRange ==============================

#[pyclass(name = "OpeningRange", module = "wickra._wickra", skip_from_py_object)]
#[derive(Clone)]
struct PyOpeningRange {
    inner: wc::OpeningRange,
}

#[pymethods]
impl PyOpeningRange {
    #[new]
    #[pyo3(signature = (period=6))]
    fn new(period: usize) -> PyResult<Self> {
        Ok(Self {
            inner: wc::OpeningRange::new(period).map_err(map_err)?,
        })
    }
    fn update(&mut self, candle: &Bound<'_, PyAny>) -> PyResult<Option<(f64, f64, f64)>> {
        let c = extract_candle(candle)?;
        Ok(self
            .inner
            .update(c)
            .map(|o| (o.high, o.low, o.breakout_distance)))
    }
    /// Batch over numpy columns high, low, close. Returns shape `(n, 3)`
    /// with columns `[high, low, breakout_distance]`.
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        high: PyReadonlyArray1<'py, f64>,
        low: PyReadonlyArray1<'py, f64>,
        close: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray2<f64>>> {
        let h = high
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let l = low
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let c = close
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        if h.len() != l.len() || l.len() != c.len() {
            return Err(PyValueError::new_err(
                "high, low, close must be equal length",
            ));
        }
        let n = h.len();
        let mut out = vec![f64::NAN; n * 3];
        for i in 0..n {
            let candle = wc::Candle::new(c[i], h[i], l[i], c[i], 0.0, 0).map_err(map_err)?;
            if let Some(o) = self.inner.update(candle) {
                out[i * 3] = o.high;
                out[i * 3 + 1] = o.low;
                out[i * 3 + 2] = o.breakout_distance;
            }
        }
        Ok(numpy::ndarray::Array2::from_shape_vec((n, 3), out)
            .expect("shape consistent")
            .into_pyarray(py))
    }
    #[getter]
    fn period(&self) -> usize {
        self.inner.period()
    }
    fn is_locked(&self) -> bool {
        self.inner.is_locked()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        format!("OpeningRange(period={})", self.inner.period())
    }
}

// ============================== Candlestick Patterns ==============================
//
// All 15 patterns take Candles and emit a signed f64 signal per bar:
//   +1.0 bullish, -1.0 bearish, 0.0 no pattern. Doji is direction-less by
// default (+1.0 / 0.0); construct it with `signed=True` for the
// dragonfly/gravestone signed +-1 encoding.

macro_rules! candle_pattern_no_param {
    ($name:ident, $inner:ty, $repr:expr) => {
        #[pyclass(name = $repr, module = "wickra._wickra", skip_from_py_object)]
        #[derive(Clone)]
        struct $name {
            inner: $inner,
        }

        #[pymethods]
        impl $name {
            #[new]
            fn new() -> Self {
                Self {
                    inner: <$inner>::new(),
                }
            }
            fn update(&mut self, candle: &Bound<'_, PyAny>) -> PyResult<Option<f64>> {
                let c = extract_candle(candle)?;
                Ok(self.inner.update(c))
            }
            fn batch<'py>(
                &mut self,
                py: Python<'py>,
                open: PyReadonlyArray1<'py, f64>,
                high: PyReadonlyArray1<'py, f64>,
                low: PyReadonlyArray1<'py, f64>,
                close: PyReadonlyArray1<'py, f64>,
            ) -> PyResult<Bound<'py, PyArray1<f64>>> {
                let o = open
                    .as_slice()
                    .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
                let h = high
                    .as_slice()
                    .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
                let l = low
                    .as_slice()
                    .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
                let c = close
                    .as_slice()
                    .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
                if o.len() != h.len() || h.len() != l.len() || l.len() != c.len() {
                    return Err(PyValueError::new_err(
                        "open, high, low, close must be equal length",
                    ));
                }
                let mut out = Vec::with_capacity(o.len());
                for i in 0..o.len() {
                    let candle =
                        wc::Candle::new(o[i], h[i], l[i], c[i], 0.0, 0).map_err(map_err)?;
                    out.push(self.inner.update(candle).unwrap_or(f64::NAN));
                }
                Ok(out.into_pyarray(py))
            }
            fn reset(&mut self) {
                self.inner.reset();
            }
            fn is_ready(&self) -> bool {
                self.inner.is_ready()
            }
            fn warmup_period(&self) -> usize {
                self.inner.warmup_period()
            }
            fn __repr__(&self) -> String {
                format!("{}()", $repr)
            }
        }
    };
}

// Doji is the one pattern with an opt-in signed mode, so it is hand-written
// rather than generated by `candle_pattern_no_param!`.
#[pyclass(name = "Doji", module = "wickra._wickra", skip_from_py_object)]
#[derive(Clone)]
struct PyDoji {
    inner: wc::Doji,
}

#[pymethods]
impl PyDoji {
    #[new]
    #[pyo3(signature = (signed = false))]
    fn new(signed: bool) -> Self {
        let inner = if signed {
            wc::Doji::new().signed()
        } else {
            wc::Doji::new()
        };
        Self { inner }
    }
    fn update(&mut self, candle: &Bound<'_, PyAny>) -> PyResult<Option<f64>> {
        let c = extract_candle(candle)?;
        Ok(self.inner.update(c))
    }
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        open: PyReadonlyArray1<'py, f64>,
        high: PyReadonlyArray1<'py, f64>,
        low: PyReadonlyArray1<'py, f64>,
        close: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let o = open
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let h = high
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let l = low
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let c = close
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        if o.len() != h.len() || h.len() != l.len() || l.len() != c.len() {
            return Err(PyValueError::new_err(
                "open, high, low, close must be equal length",
            ));
        }
        let mut out = Vec::with_capacity(o.len());
        for i in 0..o.len() {
            let candle = wc::Candle::new(o[i], h[i], l[i], c[i], 0.0, 0).map_err(map_err)?;
            out.push(self.inner.update(candle).unwrap_or(f64::NAN));
        }
        Ok(out.into_pyarray(py))
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn is_signed(&self) -> bool {
        self.inner.is_signed()
    }
    fn __repr__(&self) -> String {
        format!(
            "Doji(signed={})",
            if self.inner.is_signed() {
                "True"
            } else {
                "False"
            }
        )
    }
}

candle_pattern_no_param!(PyHammer, wc::Hammer, "Hammer");
candle_pattern_no_param!(PyInvertedHammer, wc::InvertedHammer, "InvertedHammer");
candle_pattern_no_param!(PyHangingMan, wc::HangingMan, "HangingMan");
candle_pattern_no_param!(PyShootingStar, wc::ShootingStar, "ShootingStar");
candle_pattern_no_param!(PyEngulfing, wc::Engulfing, "Engulfing");
candle_pattern_no_param!(PyHarami, wc::Harami, "Harami");
candle_pattern_no_param!(
    PyMorningEveningStar,
    wc::MorningEveningStar,
    "MorningEveningStar"
);
candle_pattern_no_param!(
    PyThreeSoldiersOrCrows,
    wc::ThreeSoldiersOrCrows,
    "ThreeSoldiersOrCrows"
);
candle_pattern_no_param!(
    PyPiercingDarkCloud,
    wc::PiercingDarkCloud,
    "PiercingDarkCloud"
);
candle_pattern_no_param!(PyMarubozu, wc::Marubozu, "Marubozu");
candle_pattern_no_param!(PyTweezer, wc::Tweezer, "Tweezer");
candle_pattern_no_param!(PySpinningTop, wc::SpinningTop, "SpinningTop");
candle_pattern_no_param!(PyThreeInside, wc::ThreeInside, "ThreeInside");
candle_pattern_no_param!(PyThreeOutside, wc::ThreeOutside, "ThreeOutside");

// ============================== Microstructure: Order Book ==============================
//
// Order-book indicators consume a depth snapshot rather than OHLCV. Streaming
// `update(bid_px, bid_sz, ask_px, ask_sz)` takes four equal-length sequences
// describing one snapshot (bids best-first = descending price, asks best-first
// = ascending price); `batch` takes a list of such `(bid_px, bid_sz, ask_px,
// ask_sz)` tuples and returns one value per snapshot.

fn build_order_book(
    bid_px: &[f64],
    bid_sz: &[f64],
    ask_px: &[f64],
    ask_sz: &[f64],
) -> PyResult<wc::OrderBook> {
    if bid_px.len() != bid_sz.len() || ask_px.len() != ask_sz.len() {
        return Err(PyValueError::new_err(
            "bid/ask price and size arrays must be equal length",
        ));
    }
    let bids = bid_px
        .iter()
        .zip(bid_sz)
        .map(|(&p, &s)| wc::Level::new_unchecked(p, s))
        .collect();
    let asks = ask_px
        .iter()
        .zip(ask_sz)
        .map(|(&p, &s)| wc::Level::new_unchecked(p, s))
        .collect();
    wc::OrderBook::new(bids, asks).map_err(map_err)
}

macro_rules! py_ob_indicator {
    ($name:ident, $inner:ty, $repr:expr) => {
        #[pyclass(name = $repr, module = "wickra._wickra", skip_from_py_object)]
        #[derive(Clone)]
        struct $name {
            inner: $inner,
        }

        #[pymethods]
        impl $name {
            #[new]
            fn new() -> Self {
                Self {
                    inner: <$inner>::new(),
                }
            }
            fn update(
                &mut self,
                bid_px: Vec<f64>,
                bid_sz: Vec<f64>,
                ask_px: Vec<f64>,
                ask_sz: Vec<f64>,
            ) -> PyResult<Option<f64>> {
                let book = build_order_book(&bid_px, &bid_sz, &ask_px, &ask_sz)?;
                Ok(self.inner.update(book))
            }
            #[allow(clippy::type_complexity)]
            fn batch<'py>(
                &mut self,
                py: Python<'py>,
                snapshots: Vec<(Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>)>,
            ) -> PyResult<Bound<'py, PyArray1<f64>>> {
                let mut out = Vec::with_capacity(snapshots.len());
                for (bid_px, bid_sz, ask_px, ask_sz) in &snapshots {
                    let book = build_order_book(bid_px, bid_sz, ask_px, ask_sz)?;
                    out.push(self.inner.update(book).unwrap_or(f64::NAN));
                }
                Ok(out.into_pyarray(py))
            }
            fn reset(&mut self) {
                self.inner.reset();
            }
            fn is_ready(&self) -> bool {
                self.inner.is_ready()
            }
            fn warmup_period(&self) -> usize {
                self.inner.warmup_period()
            }
            fn __repr__(&self) -> String {
                format!("{}()", $repr)
            }
        }
    };
}

py_ob_indicator!(
    PyOrderBookImbalanceTop1,
    wc::OrderBookImbalanceTop1,
    "OrderBookImbalanceTop1"
);
py_ob_indicator!(
    PyOrderBookImbalanceFull,
    wc::OrderBookImbalanceFull,
    "OrderBookImbalanceFull"
);
py_ob_indicator!(PyMicroprice, wc::Microprice, "Microprice");
py_ob_indicator!(PyQuotedSpread, wc::QuotedSpread, "QuotedSpread");
py_ob_indicator!(PyDepthSlope, wc::DepthSlope, "DepthSlope");

// Top-N imbalance carries a `levels` parameter, so it is hand-written.
#[pyclass(
    name = "OrderBookImbalanceTopN",
    module = "wickra._wickra",
    skip_from_py_object
)]
#[derive(Clone)]
struct PyOrderBookImbalanceTopN {
    inner: wc::OrderBookImbalanceTopN,
}

#[pymethods]
impl PyOrderBookImbalanceTopN {
    #[new]
    fn new(levels: usize) -> PyResult<Self> {
        Ok(Self {
            inner: wc::OrderBookImbalanceTopN::new(levels).map_err(map_err)?,
        })
    }
    fn update(
        &mut self,
        bid_px: Vec<f64>,
        bid_sz: Vec<f64>,
        ask_px: Vec<f64>,
        ask_sz: Vec<f64>,
    ) -> PyResult<Option<f64>> {
        let book = build_order_book(&bid_px, &bid_sz, &ask_px, &ask_sz)?;
        Ok(self.inner.update(book))
    }
    #[allow(clippy::type_complexity)]
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        snapshots: Vec<(Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>)>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let mut out = Vec::with_capacity(snapshots.len());
        for (bid_px, bid_sz, ask_px, ask_sz) in &snapshots {
            let book = build_order_book(bid_px, bid_sz, ask_px, ask_sz)?;
            out.push(self.inner.update(book).unwrap_or(f64::NAN));
        }
        Ok(out.into_pyarray(py))
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        format!("OrderBookImbalanceTopN(levels={})", self.inner.levels())
    }
}

// ============================== Microstructure: Trade Flow ==============================
//
// Trade-flow indicators consume a trade tape rather than OHLCV. Streaming
// `update(price, size, is_buy)` takes one trade (`is_buy=True` for a
// buyer-initiated trade); `batch` takes three equal-length arrays.

fn build_trade(price: f64, size: f64, is_buy: bool) -> PyResult<wc::Trade> {
    let side = if is_buy {
        wc::Side::Buy
    } else {
        wc::Side::Sell
    };
    wc::Trade::new(price, size, side, 0).map_err(map_err)
}

macro_rules! py_trade_indicator {
    ($name:ident, $inner:ty, $repr:expr) => {
        #[pyclass(name = $repr, module = "wickra._wickra", skip_from_py_object)]
        #[derive(Clone)]
        struct $name {
            inner: $inner,
        }

        #[pymethods]
        impl $name {
            #[new]
            fn new() -> Self {
                Self {
                    inner: <$inner>::new(),
                }
            }
            fn update(&mut self, price: f64, size: f64, is_buy: bool) -> PyResult<Option<f64>> {
                Ok(self.inner.update(build_trade(price, size, is_buy)?))
            }
            fn batch<'py>(
                &mut self,
                py: Python<'py>,
                price: Vec<f64>,
                size: Vec<f64>,
                is_buy: Vec<bool>,
            ) -> PyResult<Bound<'py, PyArray1<f64>>> {
                if price.len() != size.len() || size.len() != is_buy.len() {
                    return Err(PyValueError::new_err(
                        "price, size, is_buy must be equal length",
                    ));
                }
                let mut out = Vec::with_capacity(price.len());
                for i in 0..price.len() {
                    let trade = build_trade(price[i], size[i], is_buy[i])?;
                    out.push(self.inner.update(trade).unwrap_or(f64::NAN));
                }
                Ok(out.into_pyarray(py))
            }
            fn reset(&mut self) {
                self.inner.reset();
            }
            fn is_ready(&self) -> bool {
                self.inner.is_ready()
            }
            fn warmup_period(&self) -> usize {
                self.inner.warmup_period()
            }
            fn __repr__(&self) -> String {
                format!("{}()", $repr)
            }
        }
    };
}

py_trade_indicator!(PySignedVolume, wc::SignedVolume, "SignedVolume");
py_trade_indicator!(
    PyCumulativeVolumeDelta,
    wc::CumulativeVolumeDelta,
    "CumulativeVolumeDelta"
);

// Trade imbalance carries a `window` parameter, so it is hand-written.
#[pyclass(
    name = "TradeImbalance",
    module = "wickra._wickra",
    skip_from_py_object
)]
#[derive(Clone)]
struct PyTradeImbalance {
    inner: wc::TradeImbalance,
}

#[pymethods]
impl PyTradeImbalance {
    #[new]
    fn new(window: usize) -> PyResult<Self> {
        Ok(Self {
            inner: wc::TradeImbalance::new(window).map_err(map_err)?,
        })
    }
    fn update(&mut self, price: f64, size: f64, is_buy: bool) -> PyResult<Option<f64>> {
        Ok(self.inner.update(build_trade(price, size, is_buy)?))
    }
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        price: Vec<f64>,
        size: Vec<f64>,
        is_buy: Vec<bool>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        if price.len() != size.len() || size.len() != is_buy.len() {
            return Err(PyValueError::new_err(
                "price, size, is_buy must be equal length",
            ));
        }
        let mut out = Vec::with_capacity(price.len());
        for i in 0..price.len() {
            let trade = build_trade(price[i], size[i], is_buy[i])?;
            out.push(self.inner.update(trade).unwrap_or(f64::NAN));
        }
        Ok(out.into_pyarray(py))
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        format!("TradeImbalance(window={})", self.inner.window())
    }
}

// ============================== Microstructure: Price Impact ==============================
//
// Price-impact indicators consume a trade paired with the mid prevailing at
// execution. Streaming `update(price, size, is_buy, mid)` takes one such
// trade-quote (`is_buy=True` for a buyer-initiated trade); `batch` takes four
// equal-length arrays.

fn build_trade_quote(price: f64, size: f64, is_buy: bool, mid: f64) -> PyResult<wc::TradeQuote> {
    let trade = build_trade(price, size, is_buy)?;
    wc::TradeQuote::new(trade, mid).map_err(map_err)
}

macro_rules! py_trade_quote_indicator {
    ($name:ident, $inner:ty, $repr:expr) => {
        #[pyclass(name = $repr, module = "wickra._wickra", skip_from_py_object)]
        #[derive(Clone)]
        struct $name {
            inner: $inner,
        }

        #[pymethods]
        impl $name {
            #[new]
            fn new() -> Self {
                Self {
                    inner: <$inner>::new(),
                }
            }
            fn update(
                &mut self,
                price: f64,
                size: f64,
                is_buy: bool,
                mid: f64,
            ) -> PyResult<Option<f64>> {
                Ok(self
                    .inner
                    .update(build_trade_quote(price, size, is_buy, mid)?))
            }
            fn batch<'py>(
                &mut self,
                py: Python<'py>,
                price: Vec<f64>,
                size: Vec<f64>,
                is_buy: Vec<bool>,
                mid: Vec<f64>,
            ) -> PyResult<Bound<'py, PyArray1<f64>>> {
                if price.len() != size.len()
                    || size.len() != is_buy.len()
                    || is_buy.len() != mid.len()
                {
                    return Err(PyValueError::new_err(
                        "price, size, is_buy, mid must be equal length",
                    ));
                }
                let mut out = Vec::with_capacity(price.len());
                for i in 0..price.len() {
                    let quote = build_trade_quote(price[i], size[i], is_buy[i], mid[i])?;
                    out.push(self.inner.update(quote).unwrap_or(f64::NAN));
                }
                Ok(out.into_pyarray(py))
            }
            fn reset(&mut self) {
                self.inner.reset();
            }
            fn is_ready(&self) -> bool {
                self.inner.is_ready()
            }
            fn warmup_period(&self) -> usize {
                self.inner.warmup_period()
            }
            fn __repr__(&self) -> String {
                format!("{}()", $repr)
            }
        }
    };
}

py_trade_quote_indicator!(PyEffectiveSpread, wc::EffectiveSpread, "EffectiveSpread");

// Realized spread carries a `horizon` parameter, so it is hand-written.
#[pyclass(
    name = "RealizedSpread",
    module = "wickra._wickra",
    skip_from_py_object
)]
#[derive(Clone)]
struct PyRealizedSpread {
    inner: wc::RealizedSpread,
}

#[pymethods]
impl PyRealizedSpread {
    #[new]
    fn new(horizon: usize) -> PyResult<Self> {
        Ok(Self {
            inner: wc::RealizedSpread::new(horizon).map_err(map_err)?,
        })
    }
    fn update(&mut self, price: f64, size: f64, is_buy: bool, mid: f64) -> PyResult<Option<f64>> {
        Ok(self
            .inner
            .update(build_trade_quote(price, size, is_buy, mid)?))
    }
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        price: Vec<f64>,
        size: Vec<f64>,
        is_buy: Vec<bool>,
        mid: Vec<f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        if price.len() != size.len() || size.len() != is_buy.len() || is_buy.len() != mid.len() {
            return Err(PyValueError::new_err(
                "price, size, is_buy, mid must be equal length",
            ));
        }
        let mut out = Vec::with_capacity(price.len());
        for i in 0..price.len() {
            let quote = build_trade_quote(price[i], size[i], is_buy[i], mid[i])?;
            out.push(self.inner.update(quote).unwrap_or(f64::NAN));
        }
        Ok(out.into_pyarray(py))
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        format!("RealizedSpread(horizon={})", self.inner.horizon())
    }
}

// Kyle's lambda carries a `window` parameter, so it is hand-written.
#[pyclass(name = "KylesLambda", module = "wickra._wickra", skip_from_py_object)]
#[derive(Clone)]
struct PyKylesLambda {
    inner: wc::KylesLambda,
}

#[pymethods]
impl PyKylesLambda {
    #[new]
    fn new(window: usize) -> PyResult<Self> {
        Ok(Self {
            inner: wc::KylesLambda::new(window).map_err(map_err)?,
        })
    }
    fn update(&mut self, price: f64, size: f64, is_buy: bool, mid: f64) -> PyResult<Option<f64>> {
        Ok(self
            .inner
            .update(build_trade_quote(price, size, is_buy, mid)?))
    }
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        price: Vec<f64>,
        size: Vec<f64>,
        is_buy: Vec<bool>,
        mid: Vec<f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        if price.len() != size.len() || size.len() != is_buy.len() || is_buy.len() != mid.len() {
            return Err(PyValueError::new_err(
                "price, size, is_buy, mid must be equal length",
            ));
        }
        let mut out = Vec::with_capacity(price.len());
        for i in 0..price.len() {
            let quote = build_trade_quote(price[i], size[i], is_buy[i], mid[i])?;
            out.push(self.inner.update(quote).unwrap_or(f64::NAN));
        }
        Ok(out.into_pyarray(py))
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        format!("KylesLambda(window={})", self.inner.window())
    }
}

// ============================== Microstructure: Footprint ==============================
//
// Footprint is a multi-output, variable-length indicator: each `update(price,
// size, is_buy)` returns the full bar footprint accumulated since the last
// `reset()` as a `(k, 3)` array with columns `[price, bid_vol, ask_vol]`, one
// row per touched price bucket (sorted ascending by price). `batch` returns a
// list of such arrays, one per trade.

fn footprint_to_array<'py>(
    py: Python<'py>,
    out: &wc::FootprintOutput,
) -> Bound<'py, PyArray2<f64>> {
    let rows = out.levels.len();
    let mut data = Vec::with_capacity(rows * 3);
    for level in &out.levels {
        data.push(level.price);
        data.push(level.bid_vol);
        data.push(level.ask_vol);
    }
    numpy::ndarray::Array2::from_shape_vec((rows, 3), data)
        .expect("shape consistent")
        .into_pyarray(py)
}

#[pyclass(name = "Footprint", module = "wickra._wickra", skip_from_py_object)]
#[derive(Clone)]
struct PyFootprint {
    inner: wc::Footprint,
}

#[pymethods]
impl PyFootprint {
    #[new]
    fn new(tick_size: f64) -> PyResult<Self> {
        Ok(Self {
            inner: wc::Footprint::new(tick_size).map_err(map_err)?,
        })
    }
    fn update<'py>(
        &mut self,
        py: Python<'py>,
        price: f64,
        size: f64,
        is_buy: bool,
    ) -> PyResult<Bound<'py, PyArray2<f64>>> {
        let out = self
            .inner
            .update(build_trade(price, size, is_buy)?)
            .expect("footprint emits on every trade");
        Ok(footprint_to_array(py, &out))
    }
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        price: Vec<f64>,
        size: Vec<f64>,
        is_buy: Vec<bool>,
    ) -> PyResult<Vec<Bound<'py, PyArray2<f64>>>> {
        if price.len() != size.len() || size.len() != is_buy.len() {
            return Err(PyValueError::new_err(
                "price, size, is_buy must be equal length",
            ));
        }
        let mut out = Vec::with_capacity(price.len());
        for i in 0..price.len() {
            let snapshot = self
                .inner
                .update(build_trade(price[i], size[i], is_buy[i])?)
                .expect("footprint emits on every trade");
            out.push(footprint_to_array(py, &snapshot));
        }
        Ok(out)
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        format!("Footprint(tick_size={})", self.inner.tick_size())
    }
}

// ============================== Derivatives ==============================
//
// Derivatives indicators consume a perpetual / futures tick rather than OHLCV.
// Each wrapper exposes only the tick fields its indicator reads; the helpers
// below build a fully-valid `DerivativesTick`, filling the unused fields with
// neutral defaults (prices `1.0`, sizes / rates `0.0`).

fn deriv_funding(funding_rate: f64) -> PyResult<wc::DerivativesTick> {
    wc::DerivativesTick::new(
        funding_rate,
        1.0,
        1.0,
        1.0,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        0,
    )
    .map_err(map_err)
}

fn deriv_basis(mark_price: f64, index_price: f64) -> PyResult<wc::DerivativesTick> {
    wc::DerivativesTick::new(
        0.0,
        mark_price,
        index_price,
        1.0,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        0,
    )
    .map_err(map_err)
}

fn deriv_oi(open_interest: f64) -> PyResult<wc::DerivativesTick> {
    wc::DerivativesTick::new(
        0.0,
        1.0,
        1.0,
        1.0,
        open_interest,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        0,
    )
    .map_err(map_err)
}

fn deriv_oi_mark(open_interest: f64, mark_price: f64) -> PyResult<wc::DerivativesTick> {
    wc::DerivativesTick::new(
        0.0,
        mark_price,
        1.0,
        1.0,
        open_interest,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        0,
    )
    .map_err(map_err)
}

fn deriv_long_short(long_size: f64, short_size: f64) -> PyResult<wc::DerivativesTick> {
    wc::DerivativesTick::new(
        0.0, 1.0, 1.0, 1.0, 0.0, long_size, short_size, 0.0, 0.0, 0.0, 0.0, 0,
    )
    .map_err(map_err)
}

fn deriv_taker(taker_buy_volume: f64, taker_sell_volume: f64) -> PyResult<wc::DerivativesTick> {
    wc::DerivativesTick::new(
        0.0,
        1.0,
        1.0,
        1.0,
        0.0,
        0.0,
        0.0,
        taker_buy_volume,
        taker_sell_volume,
        0.0,
        0.0,
        0,
    )
    .map_err(map_err)
}

fn deriv_liquidation(
    long_liquidation: f64,
    short_liquidation: f64,
) -> PyResult<wc::DerivativesTick> {
    wc::DerivativesTick::new(
        0.0,
        1.0,
        1.0,
        1.0,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        long_liquidation,
        short_liquidation,
        0,
    )
    .map_err(map_err)
}

fn deriv_futures_index(futures_price: f64, index_price: f64) -> PyResult<wc::DerivativesTick> {
    wc::DerivativesTick::new(
        0.0,
        1.0,
        index_price,
        futures_price,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        0,
    )
    .map_err(map_err)
}

fn deriv_futures_mark(futures_price: f64, mark_price: f64) -> PyResult<wc::DerivativesTick> {
    wc::DerivativesTick::new(
        0.0,
        mark_price,
        1.0,
        futures_price,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        0,
    )
    .map_err(map_err)
}

// FundingRate takes no parameters; streaming `update(funding_rate)`, `batch`
// over one funding-rate array.
#[pyclass(name = "FundingRate", module = "wickra._wickra", skip_from_py_object)]
#[derive(Clone)]
struct PyFundingRate {
    inner: wc::FundingRate,
}

#[pymethods]
impl PyFundingRate {
    #[new]
    fn new() -> Self {
        Self {
            inner: wc::FundingRate::new(),
        }
    }
    fn update(&mut self, funding_rate: f64) -> PyResult<Option<f64>> {
        Ok(self.inner.update(deriv_funding(funding_rate)?))
    }
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        funding_rate: Vec<f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let mut out = Vec::with_capacity(funding_rate.len());
        for rate in funding_rate {
            out.push(self.inner.update(deriv_funding(rate)?).unwrap_or(f64::NAN));
        }
        Ok(out.into_pyarray(py))
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        "FundingRate()".to_string()
    }
}

// FundingRateMean carries a `window` parameter.
#[pyclass(
    name = "FundingRateMean",
    module = "wickra._wickra",
    skip_from_py_object
)]
#[derive(Clone)]
struct PyFundingRateMean {
    inner: wc::FundingRateMean,
}

#[pymethods]
impl PyFundingRateMean {
    #[new]
    fn new(window: usize) -> PyResult<Self> {
        Ok(Self {
            inner: wc::FundingRateMean::new(window).map_err(map_err)?,
        })
    }
    fn update(&mut self, funding_rate: f64) -> PyResult<Option<f64>> {
        Ok(self.inner.update(deriv_funding(funding_rate)?))
    }
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        funding_rate: Vec<f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let mut out = Vec::with_capacity(funding_rate.len());
        for rate in funding_rate {
            out.push(self.inner.update(deriv_funding(rate)?).unwrap_or(f64::NAN));
        }
        Ok(out.into_pyarray(py))
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        format!("FundingRateMean(window={})", self.inner.window())
    }
}

// FundingRateZScore carries a `window` parameter.
#[pyclass(
    name = "FundingRateZScore",
    module = "wickra._wickra",
    skip_from_py_object
)]
#[derive(Clone)]
struct PyFundingRateZScore {
    inner: wc::FundingRateZScore,
}

#[pymethods]
impl PyFundingRateZScore {
    #[new]
    fn new(window: usize) -> PyResult<Self> {
        Ok(Self {
            inner: wc::FundingRateZScore::new(window).map_err(map_err)?,
        })
    }
    fn update(&mut self, funding_rate: f64) -> PyResult<Option<f64>> {
        Ok(self.inner.update(deriv_funding(funding_rate)?))
    }
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        funding_rate: Vec<f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let mut out = Vec::with_capacity(funding_rate.len());
        for rate in funding_rate {
            out.push(self.inner.update(deriv_funding(rate)?).unwrap_or(f64::NAN));
        }
        Ok(out.into_pyarray(py))
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        format!("FundingRateZScore(window={})", self.inner.window())
    }
}

// FundingBasis takes no parameters; streaming `update(mark_price, index_price)`.
#[pyclass(name = "FundingBasis", module = "wickra._wickra", skip_from_py_object)]
#[derive(Clone)]
struct PyFundingBasis {
    inner: wc::FundingBasis,
}

#[pymethods]
impl PyFundingBasis {
    #[new]
    fn new() -> Self {
        Self {
            inner: wc::FundingBasis::new(),
        }
    }
    fn update(&mut self, mark_price: f64, index_price: f64) -> PyResult<Option<f64>> {
        Ok(self.inner.update(deriv_basis(mark_price, index_price)?))
    }
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        mark_price: Vec<f64>,
        index_price: Vec<f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        if mark_price.len() != index_price.len() {
            return Err(PyValueError::new_err(
                "mark_price and index_price must be equal length",
            ));
        }
        let mut out = Vec::with_capacity(mark_price.len());
        for i in 0..mark_price.len() {
            out.push(
                self.inner
                    .update(deriv_basis(mark_price[i], index_price[i])?)
                    .unwrap_or(f64::NAN),
            );
        }
        Ok(out.into_pyarray(py))
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        "FundingBasis()".to_string()
    }
}

// OpenInterestDelta takes no parameters; streaming `update(open_interest)`.
#[pyclass(
    name = "OpenInterestDelta",
    module = "wickra._wickra",
    skip_from_py_object
)]
#[derive(Clone)]
struct PyOpenInterestDelta {
    inner: wc::OpenInterestDelta,
}

#[pymethods]
impl PyOpenInterestDelta {
    #[new]
    fn new() -> Self {
        Self {
            inner: wc::OpenInterestDelta::new(),
        }
    }
    fn update(&mut self, open_interest: f64) -> PyResult<Option<f64>> {
        Ok(self.inner.update(deriv_oi(open_interest)?))
    }
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        open_interest: Vec<f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let mut out = Vec::with_capacity(open_interest.len());
        for oi in open_interest {
            out.push(self.inner.update(deriv_oi(oi)?).unwrap_or(f64::NAN));
        }
        Ok(out.into_pyarray(py))
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        "OpenInterestDelta()".to_string()
    }
}

// OIPriceDivergence carries a `window` parameter; streaming
// `update(open_interest, mark_price)`.
#[pyclass(
    name = "OIPriceDivergence",
    module = "wickra._wickra",
    skip_from_py_object
)]
#[derive(Clone)]
struct PyOIPriceDivergence {
    inner: wc::OIPriceDivergence,
}

#[pymethods]
impl PyOIPriceDivergence {
    #[new]
    fn new(window: usize) -> PyResult<Self> {
        Ok(Self {
            inner: wc::OIPriceDivergence::new(window).map_err(map_err)?,
        })
    }
    fn update(&mut self, open_interest: f64, mark_price: f64) -> PyResult<Option<f64>> {
        Ok(self.inner.update(deriv_oi_mark(open_interest, mark_price)?))
    }
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        open_interest: Vec<f64>,
        mark_price: Vec<f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        if open_interest.len() != mark_price.len() {
            return Err(PyValueError::new_err(
                "open_interest and mark_price must be equal length",
            ));
        }
        let mut out = Vec::with_capacity(open_interest.len());
        for i in 0..open_interest.len() {
            out.push(
                self.inner
                    .update(deriv_oi_mark(open_interest[i], mark_price[i])?)
                    .unwrap_or(f64::NAN),
            );
        }
        Ok(out.into_pyarray(py))
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        format!("OIPriceDivergence(window={})", self.inner.window())
    }
}

// OIWeighted takes no parameters; streaming `update(mark_price, open_interest)`.
#[pyclass(name = "OIWeighted", module = "wickra._wickra", skip_from_py_object)]
#[derive(Clone)]
struct PyOIWeighted {
    inner: wc::OIWeighted,
}

#[pymethods]
impl PyOIWeighted {
    #[new]
    fn new() -> Self {
        Self {
            inner: wc::OIWeighted::new(),
        }
    }
    fn update(&mut self, mark_price: f64, open_interest: f64) -> PyResult<Option<f64>> {
        Ok(self.inner.update(deriv_oi_mark(open_interest, mark_price)?))
    }
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        mark_price: Vec<f64>,
        open_interest: Vec<f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        if mark_price.len() != open_interest.len() {
            return Err(PyValueError::new_err(
                "mark_price and open_interest must be equal length",
            ));
        }
        let mut out = Vec::with_capacity(mark_price.len());
        for i in 0..mark_price.len() {
            out.push(
                self.inner
                    .update(deriv_oi_mark(open_interest[i], mark_price[i])?)
                    .unwrap_or(f64::NAN),
            );
        }
        Ok(out.into_pyarray(py))
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        "OIWeighted()".to_string()
    }
}

// LongShortRatio takes no parameters; streaming `update(long_size, short_size)`.
#[pyclass(
    name = "LongShortRatio",
    module = "wickra._wickra",
    skip_from_py_object
)]
#[derive(Clone)]
struct PyLongShortRatio {
    inner: wc::LongShortRatio,
}

#[pymethods]
impl PyLongShortRatio {
    #[new]
    fn new() -> Self {
        Self {
            inner: wc::LongShortRatio::new(),
        }
    }
    fn update(&mut self, long_size: f64, short_size: f64) -> PyResult<Option<f64>> {
        Ok(self.inner.update(deriv_long_short(long_size, short_size)?))
    }
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        long_size: Vec<f64>,
        short_size: Vec<f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        if long_size.len() != short_size.len() {
            return Err(PyValueError::new_err(
                "long_size and short_size must be equal length",
            ));
        }
        let mut out = Vec::with_capacity(long_size.len());
        for i in 0..long_size.len() {
            out.push(
                self.inner
                    .update(deriv_long_short(long_size[i], short_size[i])?)
                    .unwrap_or(f64::NAN),
            );
        }
        Ok(out.into_pyarray(py))
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        "LongShortRatio()".to_string()
    }
}

// TakerBuySellRatio takes no parameters; streaming
// `update(taker_buy_volume, taker_sell_volume)`.
#[pyclass(
    name = "TakerBuySellRatio",
    module = "wickra._wickra",
    skip_from_py_object
)]
#[derive(Clone)]
struct PyTakerBuySellRatio {
    inner: wc::TakerBuySellRatio,
}

#[pymethods]
impl PyTakerBuySellRatio {
    #[new]
    fn new() -> Self {
        Self {
            inner: wc::TakerBuySellRatio::new(),
        }
    }
    fn update(&mut self, taker_buy_volume: f64, taker_sell_volume: f64) -> PyResult<Option<f64>> {
        Ok(self
            .inner
            .update(deriv_taker(taker_buy_volume, taker_sell_volume)?))
    }
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        taker_buy_volume: Vec<f64>,
        taker_sell_volume: Vec<f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        if taker_buy_volume.len() != taker_sell_volume.len() {
            return Err(PyValueError::new_err(
                "taker_buy_volume and taker_sell_volume must be equal length",
            ));
        }
        let mut out = Vec::with_capacity(taker_buy_volume.len());
        for i in 0..taker_buy_volume.len() {
            out.push(
                self.inner
                    .update(deriv_taker(taker_buy_volume[i], taker_sell_volume[i])?)
                    .unwrap_or(f64::NAN),
            );
        }
        Ok(out.into_pyarray(py))
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        "TakerBuySellRatio()".to_string()
    }
}

// LiquidationFeatures is a multi-output indicator: streaming
// `update(long_liquidation, short_liquidation)` returns a 5-tuple
// `(long, short, net, total, imbalance)`; `batch` returns an `(n, 5)` array.
#[pyclass(
    name = "LiquidationFeatures",
    module = "wickra._wickra",
    skip_from_py_object
)]
#[derive(Clone)]
struct PyLiquidationFeatures {
    inner: wc::LiquidationFeatures,
}

#[pymethods]
impl PyLiquidationFeatures {
    #[new]
    fn new() -> Self {
        Self {
            inner: wc::LiquidationFeatures::new(),
        }
    }
    /// Returns `(long, short, net, total, imbalance)` or None during warmup.
    #[allow(clippy::type_complexity)]
    fn update(
        &mut self,
        long_liquidation: f64,
        short_liquidation: f64,
    ) -> PyResult<Option<(f64, f64, f64, f64, f64)>> {
        Ok(self
            .inner
            .update(deriv_liquidation(long_liquidation, short_liquidation)?)
            .map(|o| (o.long, o.short, o.net, o.total, o.imbalance)))
    }
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        long_liquidation: Vec<f64>,
        short_liquidation: Vec<f64>,
    ) -> PyResult<Bound<'py, PyArray2<f64>>> {
        if long_liquidation.len() != short_liquidation.len() {
            return Err(PyValueError::new_err(
                "long_liquidation and short_liquidation must be equal length",
            ));
        }
        let rows = long_liquidation.len();
        let mut data = Vec::with_capacity(rows * 5);
        for i in 0..rows {
            let out = self
                .inner
                .update(deriv_liquidation(
                    long_liquidation[i],
                    short_liquidation[i],
                )?)
                .expect("liquidation features emit on every tick");
            data.push(out.long);
            data.push(out.short);
            data.push(out.net);
            data.push(out.total);
            data.push(out.imbalance);
        }
        Ok(numpy::ndarray::Array2::from_shape_vec((rows, 5), data)
            .expect("shape consistent")
            .into_pyarray(py))
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        "LiquidationFeatures()".to_string()
    }
}

// TermStructureBasis takes no parameters; streaming
// `update(futures_price, index_price)`.
#[pyclass(
    name = "TermStructureBasis",
    module = "wickra._wickra",
    skip_from_py_object
)]
#[derive(Clone)]
struct PyTermStructureBasis {
    inner: wc::TermStructureBasis,
}

#[pymethods]
impl PyTermStructureBasis {
    #[new]
    fn new() -> Self {
        Self {
            inner: wc::TermStructureBasis::new(),
        }
    }
    fn update(&mut self, futures_price: f64, index_price: f64) -> PyResult<Option<f64>> {
        Ok(self
            .inner
            .update(deriv_futures_index(futures_price, index_price)?))
    }
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        futures_price: Vec<f64>,
        index_price: Vec<f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        if futures_price.len() != index_price.len() {
            return Err(PyValueError::new_err(
                "futures_price and index_price must be equal length",
            ));
        }
        let mut out = Vec::with_capacity(futures_price.len());
        for i in 0..futures_price.len() {
            out.push(
                self.inner
                    .update(deriv_futures_index(futures_price[i], index_price[i])?)
                    .unwrap_or(f64::NAN),
            );
        }
        Ok(out.into_pyarray(py))
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        "TermStructureBasis()".to_string()
    }
}

// CalendarSpread takes no parameters; streaming `update(futures_price, mark_price)`.
#[pyclass(
    name = "CalendarSpread",
    module = "wickra._wickra",
    skip_from_py_object
)]
#[derive(Clone)]
struct PyCalendarSpread {
    inner: wc::CalendarSpread,
}

#[pymethods]
impl PyCalendarSpread {
    #[new]
    fn new() -> Self {
        Self {
            inner: wc::CalendarSpread::new(),
        }
    }
    fn update(&mut self, futures_price: f64, mark_price: f64) -> PyResult<Option<f64>> {
        Ok(self
            .inner
            .update(deriv_futures_mark(futures_price, mark_price)?))
    }
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        futures_price: Vec<f64>,
        mark_price: Vec<f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        if futures_price.len() != mark_price.len() {
            return Err(PyValueError::new_err(
                "futures_price and mark_price must be equal length",
            ));
        }
        let mut out = Vec::with_capacity(futures_price.len());
        for i in 0..futures_price.len() {
            out.push(
                self.inner
                    .update(deriv_futures_mark(futures_price[i], mark_price[i])?)
                    .unwrap_or(f64::NAN),
            );
        }
        Ok(out.into_pyarray(py))
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        "CalendarSpread()".to_string()
    }
}

// ============================== Family 15: Risk / Performance ==============================

#[pyclass(name = "SharpeRatio", module = "wickra._wickra", skip_from_py_object)]
#[derive(Clone)]
struct PySharpeRatio {
    inner: wc::SharpeRatio,
}

#[pymethods]
impl PySharpeRatio {
    #[new]
    #[pyo3(signature = (period, risk_free=0.0))]
    fn new(period: usize, risk_free: f64) -> PyResult<Self> {
        Ok(Self {
            inner: wc::SharpeRatio::new(period, risk_free).map_err(map_err)?,
        })
    }
    fn update(&mut self, value: f64) -> Option<f64> {
        self.inner.update(value)
    }
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        prices: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let slice = prices
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        Ok(flatten(self.inner.batch(slice)).into_pyarray(py))
    }
    #[getter]
    fn period(&self) -> usize {
        self.inner.period()
    }
    #[getter]
    fn risk_free(&self) -> f64 {
        self.inner.risk_free()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        format!(
            "SharpeRatio(period={}, risk_free={})",
            self.inner.period(),
            self.inner.risk_free()
        )
    }
}

#[pyclass(name = "SortinoRatio", module = "wickra._wickra", skip_from_py_object)]
#[derive(Clone)]
struct PySortinoRatio {
    inner: wc::SortinoRatio,
}

#[pymethods]
impl PySortinoRatio {
    #[new]
    #[pyo3(signature = (period, mar=0.0))]
    fn new(period: usize, mar: f64) -> PyResult<Self> {
        Ok(Self {
            inner: wc::SortinoRatio::new(period, mar).map_err(map_err)?,
        })
    }
    fn update(&mut self, value: f64) -> Option<f64> {
        self.inner.update(value)
    }
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        prices: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let slice = prices
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        Ok(flatten(self.inner.batch(slice)).into_pyarray(py))
    }
    #[getter]
    fn period(&self) -> usize {
        self.inner.period()
    }
    #[getter]
    fn mar(&self) -> f64 {
        self.inner.mar()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        format!(
            "SortinoRatio(period={}, mar={})",
            self.inner.period(),
            self.inner.mar()
        )
    }
}

#[pyclass(name = "CalmarRatio", module = "wickra._wickra", skip_from_py_object)]
#[derive(Clone)]
struct PyCalmarRatio {
    inner: wc::CalmarRatio,
}

#[pymethods]
impl PyCalmarRatio {
    #[new]
    fn new(period: usize) -> PyResult<Self> {
        Ok(Self {
            inner: wc::CalmarRatio::new(period).map_err(map_err)?,
        })
    }
    fn update(&mut self, value: f64) -> Option<f64> {
        self.inner.update(value)
    }
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        prices: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let slice = prices
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        Ok(flatten(self.inner.batch(slice)).into_pyarray(py))
    }
    #[getter]
    fn period(&self) -> usize {
        self.inner.period()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        format!("CalmarRatio(period={})", self.inner.period())
    }
}

#[pyclass(name = "OmegaRatio", module = "wickra._wickra", skip_from_py_object)]
#[derive(Clone)]
struct PyOmegaRatio {
    inner: wc::OmegaRatio,
}

#[pymethods]
impl PyOmegaRatio {
    #[new]
    #[pyo3(signature = (period, threshold=0.0))]
    fn new(period: usize, threshold: f64) -> PyResult<Self> {
        Ok(Self {
            inner: wc::OmegaRatio::new(period, threshold).map_err(map_err)?,
        })
    }
    fn update(&mut self, value: f64) -> Option<f64> {
        self.inner.update(value)
    }
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        prices: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let slice = prices
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        Ok(flatten(self.inner.batch(slice)).into_pyarray(py))
    }
    #[getter]
    fn period(&self) -> usize {
        self.inner.period()
    }
    #[getter]
    fn threshold(&self) -> f64 {
        self.inner.threshold()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        format!(
            "OmegaRatio(period={}, threshold={})",
            self.inner.period(),
            self.inner.threshold()
        )
    }
}

#[pyclass(name = "MaxDrawdown", module = "wickra._wickra", skip_from_py_object)]
#[derive(Clone)]
struct PyMaxDrawdown {
    inner: wc::MaxDrawdown,
}

#[pymethods]
impl PyMaxDrawdown {
    #[new]
    fn new(period: usize) -> PyResult<Self> {
        Ok(Self {
            inner: wc::MaxDrawdown::new(period).map_err(map_err)?,
        })
    }
    fn update(&mut self, value: f64) -> Option<f64> {
        self.inner.update(value)
    }
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        prices: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let slice = prices
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        Ok(flatten(self.inner.batch(slice)).into_pyarray(py))
    }
    #[getter]
    fn period(&self) -> usize {
        self.inner.period()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        format!("MaxDrawdown(period={})", self.inner.period())
    }
}

#[pyclass(
    name = "AverageDrawdown",
    module = "wickra._wickra",
    skip_from_py_object
)]
#[derive(Clone)]
struct PyAverageDrawdown {
    inner: wc::AverageDrawdown,
}

#[pymethods]
impl PyAverageDrawdown {
    #[new]
    fn new(period: usize) -> PyResult<Self> {
        Ok(Self {
            inner: wc::AverageDrawdown::new(period).map_err(map_err)?,
        })
    }
    fn update(&mut self, value: f64) -> Option<f64> {
        self.inner.update(value)
    }
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        prices: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let slice = prices
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        Ok(flatten(self.inner.batch(slice)).into_pyarray(py))
    }
    #[getter]
    fn period(&self) -> usize {
        self.inner.period()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        format!("AverageDrawdown(period={})", self.inner.period())
    }
}

#[pyclass(
    name = "DrawdownDuration",
    module = "wickra._wickra",
    skip_from_py_object
)]
#[derive(Clone)]
struct PyDrawdownDuration {
    inner: wc::DrawdownDuration,
}

#[pymethods]
impl PyDrawdownDuration {
    #[new]
    fn new() -> Self {
        Self {
            inner: wc::DrawdownDuration::new(),
        }
    }
    fn update(&mut self, value: f64) -> Option<u32> {
        self.inner.update(value)
    }
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        prices: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let slice = prices
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let out: Vec<f64> = self
            .inner
            .batch(slice)
            .into_iter()
            .map(|v| v.map_or(f64::NAN, f64::from))
            .collect();
        Ok(out.into_pyarray(py))
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        "DrawdownDuration()".to_string()
    }
}

#[pyclass(name = "PainIndex", module = "wickra._wickra", skip_from_py_object)]
#[derive(Clone)]
struct PyPainIndex {
    inner: wc::PainIndex,
}

#[pymethods]
impl PyPainIndex {
    #[new]
    fn new(period: usize) -> PyResult<Self> {
        Ok(Self {
            inner: wc::PainIndex::new(period).map_err(map_err)?,
        })
    }
    fn update(&mut self, value: f64) -> Option<f64> {
        self.inner.update(value)
    }
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        prices: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let slice = prices
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        Ok(flatten(self.inner.batch(slice)).into_pyarray(py))
    }
    #[getter]
    fn period(&self) -> usize {
        self.inner.period()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        format!("PainIndex(period={})", self.inner.period())
    }
}

#[pyclass(name = "ValueAtRisk", module = "wickra._wickra", skip_from_py_object)]
#[derive(Clone)]
struct PyValueAtRisk {
    inner: wc::ValueAtRisk,
}

#[pymethods]
impl PyValueAtRisk {
    #[new]
    #[pyo3(signature = (period, confidence=0.95))]
    fn new(period: usize, confidence: f64) -> PyResult<Self> {
        Ok(Self {
            inner: wc::ValueAtRisk::new(period, confidence).map_err(map_err)?,
        })
    }
    fn update(&mut self, value: f64) -> Option<f64> {
        self.inner.update(value)
    }
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        prices: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let slice = prices
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        Ok(flatten(self.inner.batch(slice)).into_pyarray(py))
    }
    #[getter]
    fn period(&self) -> usize {
        self.inner.period()
    }
    #[getter]
    fn confidence(&self) -> f64 {
        self.inner.confidence()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        format!(
            "ValueAtRisk(period={}, confidence={})",
            self.inner.period(),
            self.inner.confidence()
        )
    }
}

#[pyclass(
    name = "ConditionalValueAtRisk",
    module = "wickra._wickra",
    skip_from_py_object
)]
#[derive(Clone)]
struct PyConditionalValueAtRisk {
    inner: wc::ConditionalValueAtRisk,
}

#[pymethods]
impl PyConditionalValueAtRisk {
    #[new]
    #[pyo3(signature = (period, confidence=0.95))]
    fn new(period: usize, confidence: f64) -> PyResult<Self> {
        Ok(Self {
            inner: wc::ConditionalValueAtRisk::new(period, confidence).map_err(map_err)?,
        })
    }
    fn update(&mut self, value: f64) -> Option<f64> {
        self.inner.update(value)
    }
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        prices: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let slice = prices
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        Ok(flatten(self.inner.batch(slice)).into_pyarray(py))
    }
    #[getter]
    fn period(&self) -> usize {
        self.inner.period()
    }
    #[getter]
    fn confidence(&self) -> f64 {
        self.inner.confidence()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        format!(
            "ConditionalValueAtRisk(period={}, confidence={})",
            self.inner.period(),
            self.inner.confidence()
        )
    }
}

#[pyclass(name = "ProfitFactor", module = "wickra._wickra", skip_from_py_object)]
#[derive(Clone)]
struct PyProfitFactor {
    inner: wc::ProfitFactor,
}

#[pymethods]
impl PyProfitFactor {
    #[new]
    fn new(period: usize) -> PyResult<Self> {
        Ok(Self {
            inner: wc::ProfitFactor::new(period).map_err(map_err)?,
        })
    }
    fn update(&mut self, value: f64) -> Option<f64> {
        self.inner.update(value)
    }
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        prices: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let slice = prices
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        Ok(flatten(self.inner.batch(slice)).into_pyarray(py))
    }
    #[getter]
    fn period(&self) -> usize {
        self.inner.period()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        format!("ProfitFactor(period={})", self.inner.period())
    }
}

#[pyclass(name = "GainLossRatio", module = "wickra._wickra", skip_from_py_object)]
#[derive(Clone)]
struct PyGainLossRatio {
    inner: wc::GainLossRatio,
}

#[pymethods]
impl PyGainLossRatio {
    #[new]
    fn new(period: usize) -> PyResult<Self> {
        Ok(Self {
            inner: wc::GainLossRatio::new(period).map_err(map_err)?,
        })
    }
    fn update(&mut self, value: f64) -> Option<f64> {
        self.inner.update(value)
    }
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        prices: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let slice = prices
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        Ok(flatten(self.inner.batch(slice)).into_pyarray(py))
    }
    #[getter]
    fn period(&self) -> usize {
        self.inner.period()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        format!("GainLossRatio(period={})", self.inner.period())
    }
}

#[pyclass(
    name = "RecoveryFactor",
    module = "wickra._wickra",
    skip_from_py_object
)]
#[derive(Clone)]
struct PyRecoveryFactor {
    inner: wc::RecoveryFactor,
}

#[pymethods]
impl PyRecoveryFactor {
    #[new]
    fn new() -> Self {
        Self {
            inner: wc::RecoveryFactor::new(),
        }
    }
    fn update(&mut self, value: f64) -> Option<f64> {
        self.inner.update(value)
    }
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        prices: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let slice = prices
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        Ok(flatten(self.inner.batch(slice)).into_pyarray(py))
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        "RecoveryFactor()".to_string()
    }
}

#[pyclass(
    name = "KellyCriterion",
    module = "wickra._wickra",
    skip_from_py_object
)]
#[derive(Clone)]
struct PyKellyCriterion {
    inner: wc::KellyCriterion,
}

#[pymethods]
impl PyKellyCriterion {
    #[new]
    fn new(period: usize) -> PyResult<Self> {
        Ok(Self {
            inner: wc::KellyCriterion::new(period).map_err(map_err)?,
        })
    }
    fn update(&mut self, value: f64) -> Option<f64> {
        self.inner.update(value)
    }
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        prices: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let slice = prices
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        Ok(flatten(self.inner.batch(slice)).into_pyarray(py))
    }
    #[getter]
    fn period(&self) -> usize {
        self.inner.period()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        format!("KellyCriterion(period={})", self.inner.period())
    }
}

// --- Pair (asset, benchmark) indicators ---

#[pyclass(name = "TreynorRatio", module = "wickra._wickra", skip_from_py_object)]
#[derive(Clone)]
struct PyTreynorRatio {
    inner: wc::TreynorRatio,
}

#[pymethods]
impl PyTreynorRatio {
    #[new]
    #[pyo3(signature = (period, risk_free=0.0))]
    fn new(period: usize, risk_free: f64) -> PyResult<Self> {
        Ok(Self {
            inner: wc::TreynorRatio::new(period, risk_free).map_err(map_err)?,
        })
    }
    fn update(&mut self, asset: f64, benchmark: f64) -> Option<f64> {
        self.inner.update((asset, benchmark))
    }
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        asset: PyReadonlyArray1<'py, f64>,
        benchmark: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let a = asset
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let b = benchmark
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        if a.len() != b.len() {
            return Err(PyValueError::new_err(
                "asset and benchmark must have equal length",
            ));
        }
        let mut out = Vec::with_capacity(a.len());
        for i in 0..a.len() {
            out.push(self.inner.update((a[i], b[i])).unwrap_or(f64::NAN));
        }
        Ok(out.into_pyarray(py))
    }
    #[getter]
    fn period(&self) -> usize {
        self.inner.period()
    }
    #[getter]
    fn risk_free(&self) -> f64 {
        self.inner.risk_free()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        format!(
            "TreynorRatio(period={}, risk_free={})",
            self.inner.period(),
            self.inner.risk_free()
        )
    }
}

#[pyclass(
    name = "InformationRatio",
    module = "wickra._wickra",
    skip_from_py_object
)]
#[derive(Clone)]
struct PyInformationRatio {
    inner: wc::InformationRatio,
}

#[pymethods]
impl PyInformationRatio {
    #[new]
    fn new(period: usize) -> PyResult<Self> {
        Ok(Self {
            inner: wc::InformationRatio::new(period).map_err(map_err)?,
        })
    }
    fn update(&mut self, asset: f64, benchmark: f64) -> Option<f64> {
        self.inner.update((asset, benchmark))
    }
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        asset: PyReadonlyArray1<'py, f64>,
        benchmark: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let a = asset
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let b = benchmark
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        if a.len() != b.len() {
            return Err(PyValueError::new_err(
                "asset and benchmark must have equal length",
            ));
        }
        let mut out = Vec::with_capacity(a.len());
        for i in 0..a.len() {
            out.push(self.inner.update((a[i], b[i])).unwrap_or(f64::NAN));
        }
        Ok(out.into_pyarray(py))
    }
    #[getter]
    fn period(&self) -> usize {
        self.inner.period()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        format!("InformationRatio(period={})", self.inner.period())
    }
}

#[pyclass(name = "Alpha", module = "wickra._wickra", skip_from_py_object)]
#[derive(Clone)]
struct PyAlpha {
    inner: wc::Alpha,
}

#[pymethods]
impl PyAlpha {
    #[new]
    #[pyo3(signature = (period, risk_free=0.0))]
    fn new(period: usize, risk_free: f64) -> PyResult<Self> {
        Ok(Self {
            inner: wc::Alpha::new(period, risk_free).map_err(map_err)?,
        })
    }
    fn update(&mut self, asset: f64, benchmark: f64) -> Option<f64> {
        self.inner.update((asset, benchmark))
    }
    fn batch<'py>(
        &mut self,
        py: Python<'py>,
        asset: PyReadonlyArray1<'py, f64>,
        benchmark: PyReadonlyArray1<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let a = asset
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        let b = benchmark
            .as_slice()
            .map_err(|_| PyValueError::new_err(NON_CONTIGUOUS))?;
        if a.len() != b.len() {
            return Err(PyValueError::new_err(
                "asset and benchmark must have equal length",
            ));
        }
        let mut out = Vec::with_capacity(a.len());
        for i in 0..a.len() {
            out.push(self.inner.update((a[i], b[i])).unwrap_or(f64::NAN));
        }
        Ok(out.into_pyarray(py))
    }
    #[getter]
    fn period(&self) -> usize {
        self.inner.period()
    }
    #[getter]
    fn risk_free(&self) -> f64 {
        self.inner.risk_free()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    fn __repr__(&self) -> String {
        format!(
            "Alpha(period={}, risk_free={})",
            self.inner.period(),
            self.inner.risk_free()
        )
    }
}

// ============================== Module ==============================

#[pymodule]
#[allow(clippy::too_many_lines)]
fn _wickra(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    m.add_class::<PySma>()?;
    m.add_class::<PyEma>()?;
    m.add_class::<PyWma>()?;
    m.add_class::<PyRsi>()?;
    m.add_class::<PyMacd>()?;
    m.add_class::<PyBb>()?;
    m.add_class::<PyAtr>()?;
    m.add_class::<PyStoch>()?;
    m.add_class::<PyObv>()?;
    m.add_class::<PyDema>()?;
    m.add_class::<PyTema>()?;
    m.add_class::<PyHma>()?;
    m.add_class::<PyKama>()?;
    m.add_class::<PyRvi>()?;
    m.add_class::<PyPgo>()?;
    m.add_class::<PyKst>()?;
    m.add_class::<PySmi>()?;
    m.add_class::<PyLaguerreRsi>()?;
    m.add_class::<PyConnorsRsi>()?;
    m.add_class::<PyInertia>()?;
    m.add_class::<PyCci>()?;
    m.add_class::<PyRoc>()?;
    m.add_class::<PyWilliamsR>()?;
    m.add_class::<PyAdx>()?;
    m.add_class::<PyAdxr>()?;
    m.add_class::<PyMfi>()?;
    m.add_class::<PyTrix>()?;
    m.add_class::<PyPsar>()?;
    m.add_class::<PyKeltner>()?;
    m.add_class::<PyDonchian>()?;
    m.add_class::<PyVwap>()?;
    m.add_class::<PyRollingVwap>()?;
    m.add_class::<PyAo>()?;
    m.add_class::<PyAroon>()?;
    m.add_class::<PySmma>()?;
    m.add_class::<PyTrima>()?;
    m.add_class::<PyZlema>()?;
    m.add_class::<PyT3>()?;
    m.add_class::<PyVwma>()?;
    m.add_class::<PyMom>()?;
    m.add_class::<PyCmo>()?;
    m.add_class::<PyTsi>()?;
    m.add_class::<PyPmo>()?;
    m.add_class::<PyTii>()?;
    m.add_class::<PyKst>()?;
    m.add_class::<PyStochRsi>()?;
    m.add_class::<PyUltimateOscillator>()?;
    m.add_class::<PyPpo>()?;
    m.add_class::<PyDpo>()?;
    m.add_class::<PyCoppock>()?;
    m.add_class::<PyAroonOscillator>()?;
    m.add_class::<PyVortex>()?;
    m.add_class::<PyRwi>()?;
    m.add_class::<PyWaveTrend>()?;
    m.add_class::<PyMassIndex>()?;
    m.add_class::<PyNatr>()?;
    m.add_class::<PyStdDev>()?;
    m.add_class::<PyUlcerIndex>()?;
    m.add_class::<PyHistoricalVolatility>()?;
    m.add_class::<PyBollingerBandwidth>()?;
    m.add_class::<PyPercentB>()?;
    m.add_class::<PyAdl>()?;
    m.add_class::<PyVolumePriceTrend>()?;
    m.add_class::<PyChaikinMoneyFlow>()?;
    m.add_class::<PyChaikinOscillator>()?;
    m.add_class::<PyForceIndex>()?;
    m.add_class::<PyKvo>()?;
    m.add_class::<PyVolumeOscillator>()?;
    m.add_class::<PyNvi>()?;
    m.add_class::<PyPvi>()?;
    m.add_class::<PyAdOscillator>()?;
    m.add_class::<PyAnchoredVwap>()?;
    m.add_class::<PyDemandIndex>()?;
    m.add_class::<PyTsv>()?;
    m.add_class::<PyVzo>()?;
    m.add_class::<PyMarketFacilitationIndex>()?;
    m.add_class::<PyEaseOfMovement>()?;
    m.add_class::<PySuperTrend>()?;
    m.add_class::<PyChandelierExit>()?;
    m.add_class::<PyChandeKrollStop>()?;
    m.add_class::<PyAtrTrailingStop>()?;
    m.add_class::<PyHiLoActivator>()?;
    m.add_class::<PyVoltyStop>()?;
    m.add_class::<PyYoyoExit>()?;
    m.add_class::<PyDonchianStop>()?;
    m.add_class::<PyPercentageTrailingStop>()?;
    m.add_class::<PyStepTrailingStop>()?;
    m.add_class::<PyRenkoTrailingStop>()?;
    m.add_class::<PyTypicalPrice>()?;
    m.add_class::<PyMedianPrice>()?;
    m.add_class::<PyWeightedClose>()?;
    m.add_class::<PyLinearRegression>()?;
    m.add_class::<PyLinRegSlope>()?;
    m.add_class::<PyAcceleratorOscillator>()?;
    m.add_class::<PyBalanceOfPower>()?;
    m.add_class::<PyChoppinessIndex>()?;
    m.add_class::<PyVerticalHorizontalFilter>()?;
    m.add_class::<PyTrueRange>()?;
    m.add_class::<PyChaikinVolatility>()?;
    m.add_class::<PyZScore>()?;
    m.add_class::<PyLinRegAngle>()?;
    m.add_class::<PyAlma>()?;
    m.add_class::<PyFrama>()?;
    m.add_class::<PyMcGinleyDynamic>()?;
    m.add_class::<PyVidya>()?;
    m.add_class::<PyJma>()?;
    m.add_class::<PyAlligator>()?;
    m.add_class::<PyEvwma>()?;
    m.add_class::<PyApo>()?;
    m.add_class::<PyAoHist>()?;
    m.add_class::<PyCfo>()?;
    m.add_class::<PyZeroLagMacd>()?;
    m.add_class::<PyElderImpulse>()?;
    m.add_class::<PyStc>()?;
    m.add_class::<PyRviVolatility>()?;
    m.add_class::<PyParkinsonVolatility>()?;
    m.add_class::<PyGarmanKlassVolatility>()?;
    m.add_class::<PyRogersSatchellVolatility>()?;
    m.add_class::<PyYangZhangVolatility>()?;
    m.add_class::<PyMaEnvelope>()?;
    m.add_class::<PyAccelerationBands>()?;
    m.add_class::<PyStarcBands>()?;
    m.add_class::<PyAtrBands>()?;
    m.add_class::<PyHurstChannel>()?;
    m.add_class::<PyLinRegChannel>()?;
    m.add_class::<PyStandardErrorBands>()?;
    m.add_class::<PyDoubleBollinger>()?;
    m.add_class::<PyTtmSqueeze>()?;
    m.add_class::<PyFractalChaosBands>()?;
    m.add_class::<PyVwapStdDevBands>()?;
    m.add_class::<PyClassicPivots>()?;
    m.add_class::<PyFibonacciPivots>()?;
    m.add_class::<PyCamarilla>()?;
    m.add_class::<PyWoodiePivots>()?;
    m.add_class::<PyDemarkPivots>()?;
    m.add_class::<PyWilliamsFractals>()?;
    m.add_class::<PyZigZag>()?;
    m.add_class::<PyTdSetup>()?;
    m.add_class::<PyTdSequential>()?;
    m.add_class::<PyTdDeMarker>()?;
    m.add_class::<PyTdRei>()?;
    m.add_class::<PyTdPressure>()?;
    m.add_class::<PyTdCombo>()?;
    m.add_class::<PyTdCountdown>()?;
    m.add_class::<PyTdLines>()?;
    m.add_class::<PyTdRangeProjection>()?;
    m.add_class::<PyTdDifferential>()?;
    m.add_class::<PyTdOpen>()?;
    m.add_class::<PyTdRiskLevel>()?;
    // Family 10 — Ehlers / Cycle
    m.add_class::<PySuperSmoother>()?;
    m.add_class::<PyFisherTransform>()?;
    m.add_class::<PyInverseFisherTransform>()?;
    m.add_class::<PyDecycler>()?;
    m.add_class::<PyDecyclerOscillator>()?;
    m.add_class::<PyRoofingFilter>()?;
    m.add_class::<PyCenterOfGravity>()?;
    m.add_class::<PyCyberneticCycle>()?;
    m.add_class::<PyInstantaneousTrendline>()?;
    m.add_class::<PyEhlersStochastic>()?;
    m.add_class::<PyEmd>()?;
    m.add_class::<PyHilbertDominantCycle>()?;
    m.add_class::<PyAdaptiveCycle>()?;
    m.add_class::<PySineWave>()?;
    m.add_class::<PyMama>()?;
    m.add_class::<PyFama>()?;
    // Family 13 — Ichimoku & alternative charts
    m.add_class::<PyIchimoku>()?;
    m.add_class::<PyHeikinAshi>()?;
    m.add_class::<PyVariance>()?;
    m.add_class::<PyCoefficientOfVariation>()?;
    m.add_class::<PySkewness>()?;
    m.add_class::<PyKurtosis>()?;
    m.add_class::<PyStandardError>()?;
    m.add_class::<PyDetrendedStdDev>()?;
    m.add_class::<PyRSquared>()?;
    m.add_class::<PyAutocorrelation>()?;
    m.add_class::<PyMedianAbsoluteDeviation>()?;
    m.add_class::<PyHurstExponent>()?;
    m.add_class::<PyPearsonCorrelation>()?;
    m.add_class::<PyBeta>()?;
    m.add_class::<PyPairwiseBeta>()?;
    m.add_class::<PyPairSpreadZScore>()?;
    m.add_class::<PyLeadLagCrossCorrelation>()?;
    m.add_class::<PyCointegration>()?;
    m.add_class::<PyRelativeStrengthAB>()?;
    m.add_class::<PySpearmanCorrelation>()?;
    m.add_class::<PyValueArea>()?;
    m.add_class::<PyInitialBalance>()?;
    m.add_class::<PyOpeningRange>()?;
    // Candlestick patterns.
    m.add_class::<PyDoji>()?;
    m.add_class::<PyHammer>()?;
    m.add_class::<PyInvertedHammer>()?;
    m.add_class::<PyHangingMan>()?;
    m.add_class::<PyShootingStar>()?;
    m.add_class::<PyEngulfing>()?;
    m.add_class::<PyHarami>()?;
    m.add_class::<PyMorningEveningStar>()?;
    m.add_class::<PyThreeSoldiersOrCrows>()?;
    m.add_class::<PyPiercingDarkCloud>()?;
    m.add_class::<PyMarubozu>()?;
    m.add_class::<PyTweezer>()?;
    m.add_class::<PySpinningTop>()?;
    m.add_class::<PyThreeInside>()?;
    m.add_class::<PyThreeOutside>()?;
    // Microstructure: order book.
    m.add_class::<PyOrderBookImbalanceTop1>()?;
    m.add_class::<PyOrderBookImbalanceTopN>()?;
    m.add_class::<PyOrderBookImbalanceFull>()?;
    m.add_class::<PyMicroprice>()?;
    m.add_class::<PyQuotedSpread>()?;
    m.add_class::<PyDepthSlope>()?;
    // Microstructure: trade flow.
    m.add_class::<PySignedVolume>()?;
    m.add_class::<PyCumulativeVolumeDelta>()?;
    m.add_class::<PyTradeImbalance>()?;
    // Microstructure: price impact.
    m.add_class::<PyEffectiveSpread>()?;
    m.add_class::<PyRealizedSpread>()?;
    m.add_class::<PyKylesLambda>()?;
    // Microstructure: footprint.
    m.add_class::<PyFootprint>()?;
    // Derivatives.
    m.add_class::<PyFundingRate>()?;
    m.add_class::<PyFundingRateMean>()?;
    m.add_class::<PyFundingRateZScore>()?;
    m.add_class::<PyFundingBasis>()?;
    m.add_class::<PyOpenInterestDelta>()?;
    m.add_class::<PyOIPriceDivergence>()?;
    m.add_class::<PyOIWeighted>()?;
    m.add_class::<PyLongShortRatio>()?;
    m.add_class::<PyTakerBuySellRatio>()?;
    m.add_class::<PyLiquidationFeatures>()?;
    m.add_class::<PyTermStructureBasis>()?;
    m.add_class::<PyCalendarSpread>()?;
    // Family 15: Risk / Performance metrics.
    m.add_class::<PySharpeRatio>()?;
    m.add_class::<PySortinoRatio>()?;
    m.add_class::<PyCalmarRatio>()?;
    m.add_class::<PyOmegaRatio>()?;
    m.add_class::<PyMaxDrawdown>()?;
    m.add_class::<PyAverageDrawdown>()?;
    m.add_class::<PyDrawdownDuration>()?;
    m.add_class::<PyPainIndex>()?;
    m.add_class::<PyValueAtRisk>()?;
    m.add_class::<PyConditionalValueAtRisk>()?;
    m.add_class::<PyProfitFactor>()?;
    m.add_class::<PyGainLossRatio>()?;
    m.add_class::<PyRecoveryFactor>()?;
    m.add_class::<PyKellyCriterion>()?;
    m.add_class::<PyTreynorRatio>()?;
    m.add_class::<PyInformationRatio>()?;
    m.add_class::<PyAlpha>()?;
    Ok(())
}
