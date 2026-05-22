//! Python bindings for Wickra. Built with PyO3 and exposed under the `wickra` package.
//!
//! This module is the thin glue between `wickra-core` and Python. Every indicator
//! has both a streaming class and a batch helper that takes a NumPy array.

#![allow(clippy::needless_pass_by_value)]

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
        | wc::Error::NonFiniteInput => PyValueError::new_err(e.to_string()),
        wc::Error::InvalidCandle { .. } => PyValueError::new_err(e.to_string()),
    }
}

fn opt_to_nan(v: Option<f64>) -> f64 {
    v.unwrap_or(f64::NAN)
}

/// Convert a slice of `Option<f64>` to a flat `Vec<f64>` with NaNs for warmup.
fn flatten(values: Vec<Option<f64>>) -> Vec<f64> {
    values.into_iter().map(opt_to_nan).collect()
}

/// Raised instead of panicking when a NumPy input is not C-contiguous.
const NON_CONTIGUOUS: &str = "array must be C-contiguous; pass np.ascontiguousarray(arr)";

// ============================== SMA ==============================

#[pyclass(name = "SMA", module = "wickra._wickra")]
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
        Ok(flatten(self.inner.batch(slice)).into_pyarray_bound(py))
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

#[pyclass(name = "EMA", module = "wickra._wickra")]
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
        Ok(flatten(self.inner.batch(slice)).into_pyarray_bound(py))
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

#[pyclass(name = "WMA", module = "wickra._wickra")]
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
        Ok(flatten(self.inner.batch(slice)).into_pyarray_bound(py))
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

#[pyclass(name = "RSI", module = "wickra._wickra")]
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
        Ok(flatten(self.inner.batch(slice)).into_pyarray_bound(py))
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

#[pyclass(name = "MACD", module = "wickra._wickra")]
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
            .into_pyarray_bound(py))
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

#[pyclass(name = "BollingerBands", module = "wickra._wickra")]
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
            .into_pyarray_bound(py))
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
    if let Ok(dict) = d.downcast::<PyDict>() {
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

#[pyclass(name = "ATR", module = "wickra._wickra")]
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
        Ok(out.into_pyarray_bound(py))
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

#[pyclass(name = "Stochastic", module = "wickra._wickra")]
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
            .into_pyarray_bound(py))
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

#[pyclass(name = "OBV", module = "wickra._wickra")]
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
        Ok(out.into_pyarray_bound(py))
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

#[pyclass(name = "DEMA", module = "wickra._wickra")]
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
        Ok(flatten(self.inner.batch(s)).into_pyarray_bound(py))
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

#[pyclass(name = "TEMA", module = "wickra._wickra")]
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
        Ok(flatten(self.inner.batch(s)).into_pyarray_bound(py))
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

#[pyclass(name = "HMA", module = "wickra._wickra")]
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
        Ok(flatten(self.inner.batch(s)).into_pyarray_bound(py))
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

#[pyclass(name = "KAMA", module = "wickra._wickra")]
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
        Ok(flatten(self.inner.batch(s)).into_pyarray_bound(py))
    }
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

#[pyclass(name = "CCI", module = "wickra._wickra")]
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
        Ok(out.into_pyarray_bound(py))
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

#[pyclass(name = "ROC", module = "wickra._wickra")]
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
        Ok(flatten(self.inner.batch(s)).into_pyarray_bound(py))
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

#[pyclass(name = "WilliamsR", module = "wickra._wickra")]
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
        Ok(out.into_pyarray_bound(py))
    }
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

#[pyclass(name = "ADX", module = "wickra._wickra")]
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
            .into_pyarray_bound(py))
    }
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

#[pyclass(name = "MFI", module = "wickra._wickra")]
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
        Ok(out.into_pyarray_bound(py))
    }
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

#[pyclass(name = "TRIX", module = "wickra._wickra")]
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
        Ok(flatten(self.inner.batch(s)).into_pyarray_bound(py))
    }
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

#[pyclass(name = "PSAR", module = "wickra._wickra")]
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
        Ok(out.into_pyarray_bound(py))
    }
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

#[pyclass(name = "Keltner", module = "wickra._wickra")]
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
            .into_pyarray_bound(py))
    }
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

#[pyclass(name = "Donchian", module = "wickra._wickra")]
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
            .into_pyarray_bound(py))
    }
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

#[pyclass(name = "VWAP", module = "wickra._wickra")]
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
        Ok(out.into_pyarray_bound(py))
    }
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

// ============================== Awesome Oscillator ==============================

#[pyclass(name = "AwesomeOscillator", module = "wickra._wickra")]
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
        Ok(out.into_pyarray_bound(py))
    }
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

#[pyclass(name = "Aroon", module = "wickra._wickra")]
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
            .into_pyarray_bound(py))
    }
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

// ============================== SMMA ==============================

#[pyclass(name = "SMMA", module = "wickra._wickra")]
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
        Ok(flatten(self.inner.batch(slice)).into_pyarray_bound(py))
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

#[pyclass(name = "TRIMA", module = "wickra._wickra")]
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
        Ok(flatten(self.inner.batch(slice)).into_pyarray_bound(py))
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
    m.add_class::<PyAo>()?;
    m.add_class::<PyAroon>()?;
    m.add_class::<PySmma>()?;
    m.add_class::<PyTrima>()?;
    Ok(())
}
