//! Python bindings for Wickra. Built with `PyO3` and exposed under the `wickra` package.
//!
//! This module is the thin glue between `wickra-core` and Python. Every indicator
//! has both a streaming class and a batch helper that takes a `NumPy` array.

#![allow(clippy::needless_pass_by_value)]
// Python `__repr__` is an instance method by protocol, so the `&self` parameter is
// mandatory even when its body does not read state (e.g. parameterless indicators
// like `TypicalPrice`). Clippy's `unused_self` triggers on those signatures.
#![allow(clippy::unused_self)]

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
        | wc::Error::InvalidTick { .. } => PyValueError::new_err(e.to_string()),
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

// ============================== Module ==============================

#[pymodule]
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
    m.add_class::<PyCci>()?;
    m.add_class::<PyRoc>()?;
    m.add_class::<PyWilliamsR>()?;
    m.add_class::<PyAdx>()?;
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
    m.add_class::<PyStochRsi>()?;
    m.add_class::<PyUltimateOscillator>()?;
    m.add_class::<PyPpo>()?;
    m.add_class::<PyDpo>()?;
    m.add_class::<PyCoppock>()?;
    m.add_class::<PyAroonOscillator>()?;
    m.add_class::<PyVortex>()?;
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
    m.add_class::<PyEaseOfMovement>()?;
    m.add_class::<PySuperTrend>()?;
    m.add_class::<PyChandelierExit>()?;
    m.add_class::<PyChandeKrollStop>()?;
    m.add_class::<PyAtrTrailingStop>()?;
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
    Ok(())
}
