//! WASM bindings for Wickra. Exposes every indicator with `Float64Array` I/O so
//! the API is essentially the same in the browser as it is in Python and Rust.
//!
//! Build with:
//! ```text
//! wasm-pack build bindings/wasm --target web --release
//! ```

#![allow(clippy::needless_pass_by_value)]
#![allow(missing_debug_implementations)] // wasm_bindgen wrappers expose JS objects, no need for Debug

use js_sys::{Array, Float64Array, Object, Reflect};
use wasm_bindgen::prelude::*;
use wickra_core as wc;
use wickra_core::{BarBuilder, BatchExt, Indicator};

fn map_err(e: wc::Error) -> JsError {
    JsError::new(&e.to_string())
}

fn flatten(values: Vec<Option<f64>>) -> Vec<f64> {
    values.into_iter().map(|v| v.unwrap_or(f64::NAN)).collect()
}

/// Optional helper: install `console.error` panic hook in the browser.
#[wasm_bindgen(js_name = installPanicHook)]
pub fn install_panic_hook() {
    #[cfg(feature = "panic-hook")]
    console_error_panic_hook::set_once();
}

/// Library version (matches the Cargo package version).
#[wasm_bindgen(js_name = version)]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

// ---------- Scalar-input indicators ----------

macro_rules! wasm_scalar_indicator {
    ($name:ident, $py_name:literal, $rust_ty:ty, $($arg:ident: $arg_ty:ty),*) => {
        #[wasm_bindgen(js_name = $py_name)]
        pub struct $name {
            inner: $rust_ty,
        }

        #[wasm_bindgen(js_class = $py_name)]
        impl $name {
            #[wasm_bindgen(constructor)]
            pub fn new($($arg: $arg_ty),*) -> Result<$name, JsError> {
                Ok($name {
                    inner: <$rust_ty>::new($($arg),*).map_err(map_err)?,
                })
            }
            pub fn update(&mut self, value: f64) -> Option<f64> {
                self.inner.update(value)
            }
            pub fn batch(&mut self, prices: &[f64]) -> Float64Array {
                let out = flatten(self.inner.batch(prices));
                Float64Array::from(out.as_slice())
            }
            pub fn reset(&mut self) { self.inner.reset(); }
            #[wasm_bindgen(js_name = isReady)] pub fn is_ready(&self) -> bool { self.inner.is_ready() }
            #[wasm_bindgen(js_name = warmupPeriod)] pub fn warmup_period(&self) -> usize { self.inner.warmup_period() }
        }
    };
}

wasm_scalar_indicator!(WasmSma, "SMA", wc::Sma, period: usize);
wasm_scalar_indicator!(WasmEma, "EMA", wc::Ema, period: usize);
wasm_scalar_indicator!(WasmWma, "WMA", wc::Wma, period: usize);
wasm_scalar_indicator!(WasmRsi, "RSI", wc::Rsi, period: usize);
wasm_scalar_indicator!(WasmDema, "DEMA", wc::Dema, period: usize);
wasm_scalar_indicator!(WasmTema, "TEMA", wc::Tema, period: usize);
wasm_scalar_indicator!(WasmHma, "HMA", wc::Hma, period: usize);
wasm_scalar_indicator!(WasmRoc, "ROC", wc::Roc, period: usize);
wasm_scalar_indicator!(WasmTrix, "TRIX", wc::Trix, period: usize);
wasm_scalar_indicator!(WasmSmma, "SMMA", wc::Smma, period: usize);
wasm_scalar_indicator!(WasmTrima, "TRIMA", wc::Trima, period: usize);
wasm_scalar_indicator!(WasmZlema, "ZLEMA", wc::Zlema, period: usize);
wasm_scalar_indicator!(WasmT3, "T3", wc::T3, period: usize, v: f64);
wasm_scalar_indicator!(WasmAlma, "ALMA", wc::Alma, period: usize, offset: f64, sigma: f64);
wasm_scalar_indicator!(WasmMcGinleyDynamic, "McGinleyDynamic", wc::McGinleyDynamic, period: usize);
wasm_scalar_indicator!(WasmFrama, "FRAMA", wc::Frama, period: usize);
wasm_scalar_indicator!(WasmVidya, "VIDYA", wc::Vidya, period: usize, cmo_period: usize);
wasm_scalar_indicator!(WasmJma, "JMA", wc::Jma, period: usize, phase: f64, power: u32);
wasm_scalar_indicator!(WasmMom, "MOM", wc::Mom, period: usize);
wasm_scalar_indicator!(WasmCmo, "CMO", wc::Cmo, period: usize);
wasm_scalar_indicator!(WasmTsi, "TSI", wc::Tsi, long: usize, short: usize);
wasm_scalar_indicator!(WasmPmo, "PMO", wc::Pmo, smoothing1: usize, smoothing2: usize);
wasm_scalar_indicator!(WasmTii, "TII", wc::Tii, sma_period: usize, dev_period: usize);

#[wasm_bindgen(js_name = KST)]
pub struct WasmKst {
    inner: wc::Kst,
}

#[wasm_bindgen(js_class = KST)]
impl WasmKst {
    #[wasm_bindgen(constructor)]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        roc1: usize,
        roc2: usize,
        roc3: usize,
        roc4: usize,
        sma1: usize,
        sma2: usize,
        sma3: usize,
        sma4: usize,
        signal_period: usize,
    ) -> Result<WasmKst, JsError> {
        Ok(Self {
            inner: wc::Kst::new(
                roc1,
                roc2,
                roc3,
                roc4,
                sma1,
                sma2,
                sma3,
                sma4,
                signal_period,
            )
            .map_err(map_err)?,
        })
    }
    pub fn classic() -> WasmKst {
        Self {
            inner: wc::Kst::classic(),
        }
    }
    pub fn update(&mut self, value: f64) -> JsValue {
        match self.inner.update(value) {
            Some(o) => {
                let obj = Object::new();
                Reflect::set(&obj, &"kst".into(), &o.kst.into()).ok();
                Reflect::set(&obj, &"signal".into(), &o.signal.into()).ok();
                obj.into()
            }
            None => JsValue::NULL,
        }
    }
    /// Returns `[kst0, signal0, kst1, signal1, ...]`, length `2 * n`. Warmup is NaN.
    pub fn batch(&mut self, prices: &[f64]) -> Float64Array {
        let n = prices.len();
        let mut out = vec![f64::NAN; n * 2];
        for (i, &p) in prices.iter().enumerate() {
            if let Some(o) = self.inner.update(p) {
                out[i * 2] = o.kst;
                out[i * 2 + 1] = o.signal;
            }
        }
        Float64Array::from(out.as_slice())
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
}
wasm_scalar_indicator!(WasmStochRsi, "StochRSI", wc::StochRsi, rsi_period: usize, stoch_period: usize);
wasm_scalar_indicator!(WasmDpo, "DPO", wc::Dpo, period: usize);
wasm_scalar_indicator!(WasmPpo, "PPO", wc::Ppo, fast: usize, slow: usize);
wasm_scalar_indicator!(WasmApo, "APO", wc::Apo, fast: usize, slow: usize);
wasm_scalar_indicator!(WasmCfo, "CFO", wc::Cfo, period: usize);
wasm_scalar_indicator!(WasmElderImpulse, "ElderImpulse", wc::ElderImpulse, ema_period: usize, macd_fast: usize, macd_slow: usize, macd_signal: usize);
wasm_scalar_indicator!(WasmStc, "STC", wc::Stc, fast: usize, slow: usize, schaff_period: usize, factor: f64);

#[wasm_bindgen(js_name = ZeroLagMACD)]
pub struct WasmZeroLagMacd {
    inner: wc::ZeroLagMacd,
}

#[wasm_bindgen(js_class = ZeroLagMACD)]
impl WasmZeroLagMacd {
    #[wasm_bindgen(constructor)]
    pub fn new(fast: usize, slow: usize, signal: usize) -> Result<WasmZeroLagMacd, JsError> {
        Ok(Self {
            inner: wc::ZeroLagMacd::new(fast, slow, signal).map_err(map_err)?,
        })
    }
    /// Returns `[macd0, signal0, histogram0, ...]`, length `3n`.
    pub fn batch(&mut self, prices: &[f64]) -> Float64Array {
        let n = prices.len();
        let mut out = vec![f64::NAN; n * 3];
        for (i, p) in prices.iter().enumerate() {
            if let Some(o) = self.inner.update(*p) {
                out[i * 3] = o.macd;
                out[i * 3 + 1] = o.signal;
                out[i * 3 + 2] = o.histogram;
            }
        }
        Float64Array::from(out.as_slice())
    }
    /// Returns `{ macd, signal, histogram }` once warm, else `null`.
    pub fn update(&mut self, value: f64) -> JsValue {
        match self.inner.update(value) {
            Some(o) => {
                let obj = Object::new();
                Reflect::set(&obj, &"macd".into(), &o.macd.into()).ok();
                Reflect::set(&obj, &"signal".into(), &o.signal.into()).ok();
                Reflect::set(&obj, &"histogram".into(), &o.histogram.into()).ok();
                obj.into()
            }
            None => JsValue::NULL,
        }
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
    #[wasm_bindgen(js_name = isReady)]
    pub fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    #[wasm_bindgen(js_name = warmupPeriod)]
    pub fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
}
wasm_scalar_indicator!(WasmCoppock, "Coppock", wc::Coppock, roc_long: usize, roc_short: usize, wma_period: usize);
wasm_scalar_indicator!(WasmStdDev, "StdDev", wc::StdDev, period: usize);
wasm_scalar_indicator!(WasmUlcerIndex, "UlcerIndex", wc::UlcerIndex, period: usize);
wasm_scalar_indicator!(WasmHistoricalVolatility, "HistoricalVolatility", wc::HistoricalVolatility, period: usize, trading_periods: usize);
wasm_scalar_indicator!(WasmBollingerBandwidth, "BollingerBandwidth", wc::BollingerBandwidth, period: usize, multiplier: f64);
wasm_scalar_indicator!(WasmPercentB, "PercentB", wc::PercentB, period: usize, multiplier: f64);
wasm_scalar_indicator!(WasmLinearRegression, "LinearRegression", wc::LinearRegression, period: usize);
wasm_scalar_indicator!(WasmLinRegSlope, "LinRegSlope", wc::LinRegSlope, period: usize);
wasm_scalar_indicator!(WasmVerticalHorizontalFilter, "VerticalHorizontalFilter", wc::VerticalHorizontalFilter, period: usize);
wasm_scalar_indicator!(WasmZScore, "ZScore", wc::ZScore, period: usize);
wasm_scalar_indicator!(WasmLinRegAngle, "LinRegAngle", wc::LinRegAngle, period: usize);
wasm_scalar_indicator!(WasmVariance, "Variance", wc::Variance, period: usize);
wasm_scalar_indicator!(WasmCoefficientOfVariation, "CoefficientOfVariation", wc::CoefficientOfVariation, period: usize);
wasm_scalar_indicator!(WasmSkewness, "Skewness", wc::Skewness, period: usize);
wasm_scalar_indicator!(WasmKurtosis, "Kurtosis", wc::Kurtosis, period: usize);
wasm_scalar_indicator!(WasmStandardError, "StandardError", wc::StandardError, period: usize);
wasm_scalar_indicator!(WasmDetrendedStdDev, "DetrendedStdDev", wc::DetrendedStdDev, period: usize);
wasm_scalar_indicator!(WasmRSquared, "RSquared", wc::RSquared, period: usize);
wasm_scalar_indicator!(WasmMedianAbsoluteDeviation, "MedianAbsoluteDeviation", wc::MedianAbsoluteDeviation, period: usize);
wasm_scalar_indicator!(WasmAutocorrelation, "Autocorrelation", wc::Autocorrelation, period: usize, lag: usize);
wasm_scalar_indicator!(WasmHurstExponent, "HurstExponent", wc::HurstExponent, period: usize, chunks: usize);
wasm_scalar_indicator!(WasmRviVolatility, "RVIVolatility", wc::RviVolatility, period: usize);
wasm_scalar_indicator!(WasmLaguerreRsi, "LaguerreRSI", wc::LaguerreRsi, gamma: f64);
wasm_scalar_indicator!(WasmConnorsRsi, "ConnorsRSI", wc::ConnorsRsi, period_rsi: usize, period_streak: usize, period_rank: usize);

// ---------- Yang-Zhang Volatility (OHLC candle, 2 params) ----------

#[wasm_bindgen(js_name = YangZhangVolatility)]
pub struct WasmYangZhangVolatility {
    inner: wc::YangZhangVolatility,
}

#[wasm_bindgen(js_class = YangZhangVolatility)]
impl WasmYangZhangVolatility {
    #[wasm_bindgen(constructor)]
    pub fn new(period: usize, trading_periods: usize) -> Result<WasmYangZhangVolatility, JsError> {
        Ok(Self {
            inner: wc::YangZhangVolatility::new(period, trading_periods).map_err(map_err)?,
        })
    }
    pub fn update(
        &mut self,
        open: f64,
        high: f64,
        low: f64,
        close: f64,
    ) -> Result<Option<f64>, JsError> {
        let c = wc::Candle::new(open, high, low, close, 0.0, 0).map_err(map_err)?;
        Ok(self.inner.update(c))
    }
    pub fn batch(
        &mut self,
        open: &[f64],
        high: &[f64],
        low: &[f64],
        close: &[f64],
    ) -> Result<Float64Array, JsError> {
        let n = open.len();
        if high.len() != n || low.len() != n || close.len() != n {
            return Err(JsError::new("open, high, low, close must be equal length"));
        }
        let mut out = Vec::with_capacity(n);
        for i in 0..n {
            let c = wc::Candle::new(open[i], high[i], low[i], close[i], 0.0, 0).map_err(map_err)?;
            out.push(self.inner.update(c).unwrap_or(f64::NAN));
        }
        Ok(Float64Array::from(out.as_slice()))
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
    #[wasm_bindgen(js_name = isReady)]
    pub fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    #[wasm_bindgen(js_name = warmupPeriod)]
    pub fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
}

// ---------- Rogers-Satchell Volatility (OHLC candle, 2 params) ----------

#[wasm_bindgen(js_name = RogersSatchellVolatility)]
pub struct WasmRogersSatchellVolatility {
    inner: wc::RogersSatchellVolatility,
}

#[wasm_bindgen(js_class = RogersSatchellVolatility)]
impl WasmRogersSatchellVolatility {
    #[wasm_bindgen(constructor)]
    pub fn new(
        period: usize,
        trading_periods: usize,
    ) -> Result<WasmRogersSatchellVolatility, JsError> {
        Ok(Self {
            inner: wc::RogersSatchellVolatility::new(period, trading_periods).map_err(map_err)?,
        })
    }
    pub fn update(
        &mut self,
        open: f64,
        high: f64,
        low: f64,
        close: f64,
    ) -> Result<Option<f64>, JsError> {
        let c = wc::Candle::new(open, high, low, close, 0.0, 0).map_err(map_err)?;
        Ok(self.inner.update(c))
    }
    pub fn batch(
        &mut self,
        open: &[f64],
        high: &[f64],
        low: &[f64],
        close: &[f64],
    ) -> Result<Float64Array, JsError> {
        let n = open.len();
        if high.len() != n || low.len() != n || close.len() != n {
            return Err(JsError::new("open, high, low, close must be equal length"));
        }
        let mut out = Vec::with_capacity(n);
        for i in 0..n {
            let c = wc::Candle::new(open[i], high[i], low[i], close[i], 0.0, 0).map_err(map_err)?;
            out.push(self.inner.update(c).unwrap_or(f64::NAN));
        }
        Ok(Float64Array::from(out.as_slice()))
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
    #[wasm_bindgen(js_name = isReady)]
    pub fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    #[wasm_bindgen(js_name = warmupPeriod)]
    pub fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
}

// ---------- Garman-Klass Volatility (OHLC candle, 2 params) ----------

#[wasm_bindgen(js_name = GarmanKlassVolatility)]
pub struct WasmGarmanKlassVolatility {
    inner: wc::GarmanKlassVolatility,
}

#[wasm_bindgen(js_class = GarmanKlassVolatility)]
impl WasmGarmanKlassVolatility {
    #[wasm_bindgen(constructor)]
    pub fn new(
        period: usize,
        trading_periods: usize,
    ) -> Result<WasmGarmanKlassVolatility, JsError> {
        Ok(Self {
            inner: wc::GarmanKlassVolatility::new(period, trading_periods).map_err(map_err)?,
        })
    }
    pub fn update(
        &mut self,
        open: f64,
        high: f64,
        low: f64,
        close: f64,
    ) -> Result<Option<f64>, JsError> {
        let c = wc::Candle::new(open, high, low, close, 0.0, 0).map_err(map_err)?;
        Ok(self.inner.update(c))
    }
    pub fn batch(
        &mut self,
        open: &[f64],
        high: &[f64],
        low: &[f64],
        close: &[f64],
    ) -> Result<Float64Array, JsError> {
        let n = open.len();
        if high.len() != n || low.len() != n || close.len() != n {
            return Err(JsError::new("open, high, low, close must be equal length"));
        }
        let mut out = Vec::with_capacity(n);
        for i in 0..n {
            let c = wc::Candle::new(open[i], high[i], low[i], close[i], 0.0, 0).map_err(map_err)?;
            out.push(self.inner.update(c).unwrap_or(f64::NAN));
        }
        Ok(Float64Array::from(out.as_slice()))
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
    #[wasm_bindgen(js_name = isReady)]
    pub fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    #[wasm_bindgen(js_name = warmupPeriod)]
    pub fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
}

// ---------- Parkinson Volatility (high-low candle, 2 params) ----------

#[wasm_bindgen(js_name = ParkinsonVolatility)]
pub struct WasmParkinsonVolatility {
    inner: wc::ParkinsonVolatility,
}

#[wasm_bindgen(js_class = ParkinsonVolatility)]
impl WasmParkinsonVolatility {
    #[wasm_bindgen(constructor)]
    pub fn new(period: usize, trading_periods: usize) -> Result<WasmParkinsonVolatility, JsError> {
        Ok(Self {
            inner: wc::ParkinsonVolatility::new(period, trading_periods).map_err(map_err)?,
        })
    }
    pub fn update(&mut self, high: f64, low: f64) -> Result<Option<f64>, JsError> {
        let c = make_candle(high, low, low, 0.0)?;
        Ok(self.inner.update(c))
    }
    pub fn batch(&mut self, high: &[f64], low: &[f64]) -> Result<Float64Array, JsError> {
        if high.len() != low.len() {
            return Err(JsError::new("high and low must be equal length"));
        }
        let mut out = Vec::with_capacity(high.len());
        for i in 0..high.len() {
            let c = make_candle(high[i], low[i], low[i], 0.0)?;
            out.push(self.inner.update(c).unwrap_or(f64::NAN));
        }
        Ok(Float64Array::from(out.as_slice()))
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
    #[wasm_bindgen(js_name = isReady)]
    pub fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    #[wasm_bindgen(js_name = warmupPeriod)]
    pub fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
}

// Family 10 — Ehlers / Cycle scalars
wasm_scalar_indicator!(WasmSuperSmoother, "SuperSmoother", wc::SuperSmoother, period: usize);
wasm_scalar_indicator!(WasmFisherTransform, "FisherTransform", wc::FisherTransform, period: usize);
wasm_scalar_indicator!(WasmInverseFisherTransform, "InverseFisherTransform", wc::InverseFisherTransform, scale: f64);
wasm_scalar_indicator!(WasmDecycler, "Decycler", wc::Decycler, period: usize);
wasm_scalar_indicator!(WasmDecyclerOscillator, "DecyclerOscillator", wc::DecyclerOscillator, fast: usize, slow: usize);
wasm_scalar_indicator!(WasmRoofingFilter, "RoofingFilter", wc::RoofingFilter, lp_period: usize, hp_period: usize);
wasm_scalar_indicator!(WasmCenterOfGravity, "CenterOfGravity", wc::CenterOfGravity, period: usize);
wasm_scalar_indicator!(WasmCyberneticCycle, "CyberneticCycle", wc::CyberneticCycle, period: usize);
wasm_scalar_indicator!(WasmInstantaneousTrendline, "InstantaneousTrendline", wc::InstantaneousTrendline, period: usize);
wasm_scalar_indicator!(WasmEhlersStochastic, "EhlersStochastic", wc::EhlersStochastic, period: usize);
wasm_scalar_indicator!(WasmEmpiricalModeDecomposition, "EmpiricalModeDecomposition", wc::EmpiricalModeDecomposition, period: usize, fraction: f64);
wasm_scalar_indicator!(WasmFama, "FAMA", wc::Fama, fast_limit: f64, slow_limit: f64);

// ---------- Family 12: Two-series indicators (Pearson / Beta / Spearman) ----------

macro_rules! wasm_pair_indicator {
    ($name:ident, $js_name:literal, $rust_ty:ty) => {
        #[wasm_bindgen(js_name = $js_name)]
        pub struct $name {
            inner: $rust_ty,
        }

        #[wasm_bindgen(js_class = $js_name)]
        impl $name {
            #[wasm_bindgen(constructor)]
            pub fn new(period: usize) -> Result<$name, JsError> {
                Ok($name {
                    inner: <$rust_ty>::new(period).map_err(map_err)?,
                })
            }
            pub fn update(&mut self, x: f64, y: f64) -> Option<f64> {
                self.inner.update((x, y))
            }
            /// Batch over two equally-sized arrays. Returns one `f64` per
            /// input position (`NaN` during warmup).
            pub fn batch(&mut self, x: &[f64], y: &[f64]) -> Result<Float64Array, JsError> {
                if x.len() != y.len() {
                    return Err(JsError::new("x and y must be equal length"));
                }
                let mut out = Vec::with_capacity(x.len());
                for i in 0..x.len() {
                    out.push(self.inner.update((x[i], y[i])).unwrap_or(f64::NAN));
                }
                Ok(Float64Array::from(out.as_slice()))
            }
            pub fn reset(&mut self) {
                self.inner.reset();
            }
            #[wasm_bindgen(js_name = isReady)]
            pub fn is_ready(&self) -> bool {
                self.inner.is_ready()
            }
            #[wasm_bindgen(js_name = warmupPeriod)]
            pub fn warmup_period(&self) -> usize {
                self.inner.warmup_period()
            }
        }
    };
}

wasm_pair_indicator!(
    WasmPearsonCorrelation,
    "PearsonCorrelation",
    wc::PearsonCorrelation
);
wasm_pair_indicator!(WasmBeta, "Beta", wc::Beta);
wasm_pair_indicator!(WasmPairwiseBeta, "PairwiseBeta", wc::PairwiseBeta);
wasm_pair_indicator!(
    WasmSpearmanCorrelation,
    "SpearmanCorrelation",
    wc::SpearmanCorrelation
);

// ---------- PairSpreadZScore (two params) ----------

#[wasm_bindgen(js_name = "PairSpreadZScore")]
pub struct WasmPairSpreadZScore {
    inner: wc::PairSpreadZScore,
}

#[wasm_bindgen(js_class = "PairSpreadZScore")]
impl WasmPairSpreadZScore {
    #[wasm_bindgen(constructor)]
    pub fn new(beta_period: usize, z_period: usize) -> Result<WasmPairSpreadZScore, JsError> {
        Ok(Self {
            inner: wc::PairSpreadZScore::new(beta_period, z_period).map_err(map_err)?,
        })
    }
    pub fn update(&mut self, a: f64, b: f64) -> Option<f64> {
        self.inner.update((a, b))
    }
    /// Batch over two equally-sized arrays of prices. Returns one `f64` per
    /// input position (`NaN` during warmup).
    pub fn batch(&mut self, a: &[f64], b: &[f64]) -> Result<Float64Array, JsError> {
        if a.len() != b.len() {
            return Err(JsError::new("a and b must be equal length"));
        }
        let mut out = Vec::with_capacity(a.len());
        for i in 0..a.len() {
            out.push(self.inner.update((a[i], b[i])).unwrap_or(f64::NAN));
        }
        Ok(Float64Array::from(out.as_slice()))
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
    #[wasm_bindgen(js_name = isReady)]
    pub fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    #[wasm_bindgen(js_name = warmupPeriod)]
    pub fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
}

// ---------- LeadLagCrossCorrelation (two params, object output) ----------

#[wasm_bindgen(js_name = "LeadLagCrossCorrelation")]
pub struct WasmLeadLagCrossCorrelation {
    inner: wc::LeadLagCrossCorrelation,
}

#[wasm_bindgen(js_class = "LeadLagCrossCorrelation")]
impl WasmLeadLagCrossCorrelation {
    #[wasm_bindgen(constructor)]
    pub fn new(window: usize, max_lag: usize) -> Result<WasmLeadLagCrossCorrelation, JsError> {
        Ok(Self {
            inner: wc::LeadLagCrossCorrelation::new(window, max_lag).map_err(map_err)?,
        })
    }
    /// Returns `{ lag, correlation }`, or `null` during warmup. Positive lag
    /// means `a` leads `b`.
    pub fn update(&mut self, a: f64, b: f64) -> JsValue {
        match self.inner.update((a, b)) {
            Some(o) => {
                let obj = Object::new();
                Reflect::set(&obj, &"lag".into(), &(o.lag as f64).into()).ok();
                Reflect::set(&obj, &"correlation".into(), &o.correlation.into()).ok();
                obj.into()
            }
            None => JsValue::NULL,
        }
    }
    /// Flat `Float64Array` of length `2 * n`: `[lag0, corr0, lag1, corr1, ...]`.
    /// Warmup positions are NaN.
    pub fn batch(&mut self, a: &[f64], b: &[f64]) -> Result<Float64Array, JsError> {
        if a.len() != b.len() {
            return Err(JsError::new("a and b must be equal length"));
        }
        let n = a.len();
        let mut out = vec![f64::NAN; n * 2];
        for i in 0..n {
            if let Some(o) = self.inner.update((a[i], b[i])) {
                out[i * 2] = o.lag as f64;
                out[i * 2 + 1] = o.correlation;
            }
        }
        Ok(Float64Array::from(out.as_slice()))
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
    #[wasm_bindgen(js_name = isReady)]
    pub fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    #[wasm_bindgen(js_name = warmupPeriod)]
    pub fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
}

// ---------- Cointegration (two params, object output) ----------

#[wasm_bindgen(js_name = "Cointegration")]
pub struct WasmCointegration {
    inner: wc::Cointegration,
}

#[wasm_bindgen(js_class = "Cointegration")]
impl WasmCointegration {
    #[wasm_bindgen(constructor)]
    pub fn new(period: usize, adf_lags: usize) -> Result<WasmCointegration, JsError> {
        Ok(Self {
            inner: wc::Cointegration::new(period, adf_lags).map_err(map_err)?,
        })
    }
    /// Returns `{ hedgeRatio, spread, adfStat }`, or `null` during warmup.
    pub fn update(&mut self, a: f64, b: f64) -> JsValue {
        match self.inner.update((a, b)) {
            Some(o) => {
                let obj = Object::new();
                Reflect::set(&obj, &"hedgeRatio".into(), &o.hedge_ratio.into()).ok();
                Reflect::set(&obj, &"spread".into(), &o.spread.into()).ok();
                Reflect::set(&obj, &"adfStat".into(), &o.adf_stat.into()).ok();
                obj.into()
            }
            None => JsValue::NULL,
        }
    }
    /// Flat `Float64Array` of length `3 * n`:
    /// `[hedgeRatio0, spread0, adfStat0, hedgeRatio1, ...]`. Warmup rows are NaN.
    pub fn batch(&mut self, a: &[f64], b: &[f64]) -> Result<Float64Array, JsError> {
        if a.len() != b.len() {
            return Err(JsError::new("a and b must be equal length"));
        }
        let n = a.len();
        let mut out = vec![f64::NAN; n * 3];
        for i in 0..n {
            if let Some(o) = self.inner.update((a[i], b[i])) {
                out[i * 3] = o.hedge_ratio;
                out[i * 3 + 1] = o.spread;
                out[i * 3 + 2] = o.adf_stat;
            }
        }
        Ok(Float64Array::from(out.as_slice()))
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
    #[wasm_bindgen(js_name = isReady)]
    pub fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    #[wasm_bindgen(js_name = warmupPeriod)]
    pub fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
}

// ---------- RelativeStrengthAB (two params, object output) ----------

#[wasm_bindgen(js_name = "RelativeStrengthAB")]
pub struct WasmRelativeStrengthAb {
    inner: wc::RelativeStrengthAB,
}

#[wasm_bindgen(js_class = "RelativeStrengthAB")]
impl WasmRelativeStrengthAb {
    #[wasm_bindgen(constructor)]
    pub fn new(ma_period: usize, rsi_period: usize) -> Result<WasmRelativeStrengthAb, JsError> {
        Ok(Self {
            inner: wc::RelativeStrengthAB::new(ma_period, rsi_period).map_err(map_err)?,
        })
    }
    /// Returns `{ ratio, ratioMa, ratioRsi }`, or `null` during warmup.
    pub fn update(&mut self, a: f64, b: f64) -> JsValue {
        match self.inner.update((a, b)) {
            Some(o) => {
                let obj = Object::new();
                Reflect::set(&obj, &"ratio".into(), &o.ratio.into()).ok();
                Reflect::set(&obj, &"ratioMa".into(), &o.ratio_ma.into()).ok();
                Reflect::set(&obj, &"ratioRsi".into(), &o.ratio_rsi.into()).ok();
                obj.into()
            }
            None => JsValue::NULL,
        }
    }
    /// Flat `Float64Array` of length `3 * n`:
    /// `[ratio0, ratioMa0, ratioRsi0, ratio1, ...]`. Warmup rows are NaN.
    pub fn batch(&mut self, a: &[f64], b: &[f64]) -> Result<Float64Array, JsError> {
        if a.len() != b.len() {
            return Err(JsError::new("a and b must be equal length"));
        }
        let n = a.len();
        let mut out = vec![f64::NAN; n * 3];
        for i in 0..n {
            if let Some(o) = self.inner.update((a[i], b[i])) {
                out[i * 3] = o.ratio;
                out[i * 3 + 1] = o.ratio_ma;
                out[i * 3 + 2] = o.ratio_rsi;
            }
        }
        Ok(Float64Array::from(out.as_slice()))
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
    #[wasm_bindgen(js_name = isReady)]
    pub fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    #[wasm_bindgen(js_name = warmupPeriod)]
    pub fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
}

// ---------- KAMA (three params) ----------

#[wasm_bindgen(js_name = KAMA)]
pub struct WasmKama {
    inner: wc::Kama,
}

#[wasm_bindgen(js_class = KAMA)]
impl WasmKama {
    #[wasm_bindgen(constructor)]
    pub fn new(er_period: usize, fast: usize, slow: usize) -> Result<WasmKama, JsError> {
        Ok(Self {
            inner: wc::Kama::new(er_period, fast, slow).map_err(map_err)?,
        })
    }
    pub fn update(&mut self, value: f64) -> Option<f64> {
        self.inner.update(value)
    }
    pub fn batch(&mut self, prices: &[f64]) -> Float64Array {
        let out = flatten(self.inner.batch(prices));
        Float64Array::from(out.as_slice())
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
    #[wasm_bindgen(js_name = isReady)]
    pub fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    #[wasm_bindgen(js_name = warmupPeriod)]
    pub fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
}

// ---------- MACD ----------

#[wasm_bindgen(js_name = MACD)]
pub struct WasmMacd {
    inner: wc::MacdIndicator,
}

#[wasm_bindgen(js_class = MACD)]
impl WasmMacd {
    #[wasm_bindgen(constructor)]
    pub fn new(fast: usize, slow: usize, signal: usize) -> Result<WasmMacd, JsError> {
        Ok(Self {
            inner: wc::MacdIndicator::new(fast, slow, signal).map_err(map_err)?,
        })
    }
    pub fn update(&mut self, value: f64) -> JsValue {
        match self.inner.update(value) {
            Some(o) => {
                let obj = Object::new();
                Reflect::set(&obj, &"macd".into(), &o.macd.into()).ok();
                Reflect::set(&obj, &"signal".into(), &o.signal.into()).ok();
                Reflect::set(&obj, &"histogram".into(), &o.histogram.into()).ok();
                obj.into()
            }
            None => JsValue::NULL,
        }
    }
    /// Returns a flat `Float64Array` of length `3 * n`: `[macd0, sig0, hist0, macd1, sig1, hist1, ...]`.
    /// Use `result[3*i + 0/1/2]` to read each column. Warmup positions are NaN.
    pub fn batch(&mut self, prices: &[f64]) -> Float64Array {
        let n = prices.len();
        let mut out = vec![f64::NAN; n * 3];
        for (i, p) in prices.iter().enumerate() {
            if let Some(o) = self.inner.update(*p) {
                out[i * 3] = o.macd;
                out[i * 3 + 1] = o.signal;
                out[i * 3 + 2] = o.histogram;
            }
        }
        Float64Array::from(out.as_slice())
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
    #[wasm_bindgen(js_name = isReady)]
    pub fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
}

// ---------- Bollinger ----------

#[wasm_bindgen(js_name = BollingerBands)]
pub struct WasmBb {
    inner: wc::BollingerBands,
}

#[wasm_bindgen(js_class = BollingerBands)]
impl WasmBb {
    #[wasm_bindgen(constructor)]
    pub fn new(period: usize, multiplier: f64) -> Result<WasmBb, JsError> {
        Ok(Self {
            inner: wc::BollingerBands::new(period, multiplier).map_err(map_err)?,
        })
    }
    pub fn update(&mut self, value: f64) -> JsValue {
        match self.inner.update(value) {
            Some(o) => {
                let obj = Object::new();
                Reflect::set(&obj, &"upper".into(), &o.upper.into()).ok();
                Reflect::set(&obj, &"middle".into(), &o.middle.into()).ok();
                Reflect::set(&obj, &"lower".into(), &o.lower.into()).ok();
                Reflect::set(&obj, &"stddev".into(), &o.stddev.into()).ok();
                obj.into()
            }
            None => JsValue::NULL,
        }
    }
    /// Returns `[u0, m0, l0, sd0, u1, m1, l1, sd1, ...]`, length `4 * n`.
    pub fn batch(&mut self, prices: &[f64]) -> Float64Array {
        let n = prices.len();
        let mut out = vec![f64::NAN; n * 4];
        for (i, p) in prices.iter().enumerate() {
            if let Some(o) = self.inner.update(*p) {
                out[i * 4] = o.upper;
                out[i * 4 + 1] = o.middle;
                out[i * 4 + 2] = o.lower;
                out[i * 4 + 3] = o.stddev;
            }
        }
        Float64Array::from(out.as_slice())
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
}

// ---------- Candle-input indicators ----------

fn make_candle(h: f64, l: f64, c: f64, v: f64) -> Result<wc::Candle, JsError> {
    wc::Candle::new(c, h, l, c, v, 0).map_err(map_err)
}

/// Helper for OHLC-input indicators where `open` matters (`RVI`, `BalanceOfPower`).
fn make_candle_ohlc(o: f64, h: f64, l: f64, c: f64) -> Result<wc::Candle, JsError> {
    wc::Candle::new(o, h, l, c, 0.0, 0).map_err(map_err)
}

#[wasm_bindgen(js_name = SMI)]
pub struct WasmSmi {
    inner: wc::Smi,
}

#[wasm_bindgen(js_class = SMI)]
impl WasmSmi {
    #[wasm_bindgen(constructor)]
    pub fn new(period: usize, d_period: usize, d2_period: usize) -> Result<WasmSmi, JsError> {
        Ok(Self {
            inner: wc::Smi::new(period, d_period, d2_period).map_err(map_err)?,
        })
    }
    pub fn update(&mut self, high: f64, low: f64, close: f64) -> Result<Option<f64>, JsError> {
        let c = make_candle(high, low, close, 0.0)?;
        Ok(self.inner.update(c))
    }
    pub fn batch(
        &mut self,
        high: &[f64],
        low: &[f64],
        close: &[f64],
    ) -> Result<Float64Array, JsError> {
        if !(high.len() == low.len() && low.len() == close.len()) {
            return Err(JsError::new("high, low and close must be equal length"));
        }
        let mut out = Vec::with_capacity(close.len());
        for i in 0..close.len() {
            let c = make_candle(high[i], low[i], close[i], 0.0)?;
            out.push(self.inner.update(c).unwrap_or(f64::NAN));
        }
        Ok(Float64Array::from(out.as_slice()))
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
    #[wasm_bindgen(js_name = isReady)]
    pub fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    #[wasm_bindgen(js_name = warmupPeriod)]
    pub fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
}

#[wasm_bindgen(js_name = PGO)]
pub struct WasmPgo {
    inner: wc::Pgo,
}

#[wasm_bindgen(js_class = PGO)]
impl WasmPgo {
    #[wasm_bindgen(constructor)]
    pub fn new(period: usize) -> Result<WasmPgo, JsError> {
        Ok(Self {
            inner: wc::Pgo::new(period).map_err(map_err)?,
        })
    }
    pub fn update(&mut self, high: f64, low: f64, close: f64) -> Result<Option<f64>, JsError> {
        let c = make_candle(high, low, close, 0.0)?;
        Ok(self.inner.update(c))
    }
    pub fn batch(
        &mut self,
        high: &[f64],
        low: &[f64],
        close: &[f64],
    ) -> Result<Float64Array, JsError> {
        if !(high.len() == low.len() && low.len() == close.len()) {
            return Err(JsError::new("high, low and close must be equal length"));
        }
        let mut out = Vec::with_capacity(close.len());
        for i in 0..close.len() {
            let c = make_candle(high[i], low[i], close[i], 0.0)?;
            out.push(self.inner.update(c).unwrap_or(f64::NAN));
        }
        Ok(Float64Array::from(out.as_slice()))
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
    #[wasm_bindgen(js_name = isReady)]
    pub fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    #[wasm_bindgen(js_name = warmupPeriod)]
    pub fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
}

#[wasm_bindgen(js_name = Inertia)]
pub struct WasmInertia {
    inner: wc::Inertia,
}

#[wasm_bindgen(js_class = Inertia)]
impl WasmInertia {
    #[wasm_bindgen(constructor)]
    pub fn new(rvi_period: usize, linreg_period: usize) -> Result<WasmInertia, JsError> {
        Ok(Self {
            inner: wc::Inertia::new(rvi_period, linreg_period).map_err(map_err)?,
        })
    }
    pub fn update(
        &mut self,
        open: f64,
        high: f64,
        low: f64,
        close: f64,
    ) -> Result<Option<f64>, JsError> {
        let c = make_candle_ohlc(open, high, low, close)?;
        Ok(self.inner.update(c))
    }
    pub fn batch(
        &mut self,
        open: &[f64],
        high: &[f64],
        low: &[f64],
        close: &[f64],
    ) -> Result<Float64Array, JsError> {
        if !(open.len() == high.len() && high.len() == low.len() && low.len() == close.len()) {
            return Err(JsError::new(
                "open, high, low and close must be equal length",
            ));
        }
        let mut out = Vec::with_capacity(close.len());
        for i in 0..close.len() {
            let c = make_candle_ohlc(open[i], high[i], low[i], close[i])?;
            out.push(self.inner.update(c).unwrap_or(f64::NAN));
        }
        Ok(Float64Array::from(out.as_slice()))
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
    #[wasm_bindgen(js_name = isReady)]
    pub fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    #[wasm_bindgen(js_name = warmupPeriod)]
    pub fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
}

#[wasm_bindgen(js_name = RVI)]
pub struct WasmRvi {
    inner: wc::Rvi,
}

#[wasm_bindgen(js_class = RVI)]
impl WasmRvi {
    #[wasm_bindgen(constructor)]
    pub fn new(period: usize) -> Result<WasmRvi, JsError> {
        Ok(Self {
            inner: wc::Rvi::new(period).map_err(map_err)?,
        })
    }
    pub fn update(
        &mut self,
        open: f64,
        high: f64,
        low: f64,
        close: f64,
    ) -> Result<Option<f64>, JsError> {
        let c = make_candle_ohlc(open, high, low, close)?;
        Ok(self.inner.update(c))
    }
    pub fn batch(
        &mut self,
        open: &[f64],
        high: &[f64],
        low: &[f64],
        close: &[f64],
    ) -> Result<Float64Array, JsError> {
        if !(open.len() == high.len() && high.len() == low.len() && low.len() == close.len()) {
            return Err(JsError::new(
                "open, high, low and close must be equal length",
            ));
        }
        let mut out = Vec::with_capacity(close.len());
        for i in 0..close.len() {
            let c = make_candle_ohlc(open[i], high[i], low[i], close[i])?;
            out.push(self.inner.update(c).unwrap_or(f64::NAN));
        }
        Ok(Float64Array::from(out.as_slice()))
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
    #[wasm_bindgen(js_name = isReady)]
    pub fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    #[wasm_bindgen(js_name = warmupPeriod)]
    pub fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
}

#[wasm_bindgen(js_name = ATR)]
pub struct WasmAtr {
    inner: wc::Atr,
}

#[wasm_bindgen(js_class = ATR)]
impl WasmAtr {
    #[wasm_bindgen(constructor)]
    pub fn new(period: usize) -> Result<WasmAtr, JsError> {
        Ok(Self {
            inner: wc::Atr::new(period).map_err(map_err)?,
        })
    }
    pub fn update(&mut self, high: f64, low: f64, close: f64) -> Result<Option<f64>, JsError> {
        let c = make_candle(high, low, close, 0.0)?;
        Ok(self.inner.update(c))
    }
    pub fn batch(
        &mut self,
        high: &[f64],
        low: &[f64],
        close: &[f64],
    ) -> Result<Float64Array, JsError> {
        if high.len() != low.len() || low.len() != close.len() {
            return Err(JsError::new("high, low, close must be equal length"));
        }
        let mut out = Vec::with_capacity(high.len());
        for i in 0..high.len() {
            let c = make_candle(high[i], low[i], close[i], 0.0)?;
            out.push(self.inner.update(c).unwrap_or(f64::NAN));
        }
        Ok(Float64Array::from(out.as_slice()))
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
}

#[wasm_bindgen(js_name = PLUS_DM)]
pub struct WasmPlusDm {
    inner: wc::PlusDm,
}

#[wasm_bindgen(js_class = PLUS_DM)]
impl WasmPlusDm {
    #[wasm_bindgen(constructor)]
    pub fn new(period: usize) -> Result<WasmPlusDm, JsError> {
        Ok(Self {
            inner: wc::PlusDm::new(period).map_err(map_err)?,
        })
    }
    pub fn update(&mut self, high: f64, low: f64, close: f64) -> Result<Option<f64>, JsError> {
        let c = make_candle(high, low, close, 0.0)?;
        Ok(self.inner.update(c))
    }
    pub fn batch(
        &mut self,
        high: &[f64],
        low: &[f64],
        close: &[f64],
    ) -> Result<Float64Array, JsError> {
        if high.len() != low.len() || low.len() != close.len() {
            return Err(JsError::new("high, low, close must be equal length"));
        }
        let mut out = Vec::with_capacity(high.len());
        for i in 0..high.len() {
            let c = make_candle(high[i], low[i], close[i], 0.0)?;
            out.push(self.inner.update(c).unwrap_or(f64::NAN));
        }
        Ok(Float64Array::from(out.as_slice()))
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
}

#[wasm_bindgen(js_name = MINUS_DM)]
pub struct WasmMinusDm {
    inner: wc::MinusDm,
}

#[wasm_bindgen(js_class = MINUS_DM)]
impl WasmMinusDm {
    #[wasm_bindgen(constructor)]
    pub fn new(period: usize) -> Result<WasmMinusDm, JsError> {
        Ok(Self {
            inner: wc::MinusDm::new(period).map_err(map_err)?,
        })
    }
    pub fn update(&mut self, high: f64, low: f64, close: f64) -> Result<Option<f64>, JsError> {
        let c = make_candle(high, low, close, 0.0)?;
        Ok(self.inner.update(c))
    }
    pub fn batch(
        &mut self,
        high: &[f64],
        low: &[f64],
        close: &[f64],
    ) -> Result<Float64Array, JsError> {
        if high.len() != low.len() || low.len() != close.len() {
            return Err(JsError::new("high, low, close must be equal length"));
        }
        let mut out = Vec::with_capacity(high.len());
        for i in 0..high.len() {
            let c = make_candle(high[i], low[i], close[i], 0.0)?;
            out.push(self.inner.update(c).unwrap_or(f64::NAN));
        }
        Ok(Float64Array::from(out.as_slice()))
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
}

#[wasm_bindgen(js_name = PLUS_DI)]
pub struct WasmPlusDi {
    inner: wc::PlusDi,
}

#[wasm_bindgen(js_class = PLUS_DI)]
impl WasmPlusDi {
    #[wasm_bindgen(constructor)]
    pub fn new(period: usize) -> Result<WasmPlusDi, JsError> {
        Ok(Self {
            inner: wc::PlusDi::new(period).map_err(map_err)?,
        })
    }
    pub fn update(&mut self, high: f64, low: f64, close: f64) -> Result<Option<f64>, JsError> {
        let c = make_candle(high, low, close, 0.0)?;
        Ok(self.inner.update(c))
    }
    pub fn batch(
        &mut self,
        high: &[f64],
        low: &[f64],
        close: &[f64],
    ) -> Result<Float64Array, JsError> {
        if high.len() != low.len() || low.len() != close.len() {
            return Err(JsError::new("high, low, close must be equal length"));
        }
        let mut out = Vec::with_capacity(high.len());
        for i in 0..high.len() {
            let c = make_candle(high[i], low[i], close[i], 0.0)?;
            out.push(self.inner.update(c).unwrap_or(f64::NAN));
        }
        Ok(Float64Array::from(out.as_slice()))
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
}

#[wasm_bindgen(js_name = MINUS_DI)]
pub struct WasmMinusDi {
    inner: wc::MinusDi,
}

#[wasm_bindgen(js_class = MINUS_DI)]
impl WasmMinusDi {
    #[wasm_bindgen(constructor)]
    pub fn new(period: usize) -> Result<WasmMinusDi, JsError> {
        Ok(Self {
            inner: wc::MinusDi::new(period).map_err(map_err)?,
        })
    }
    pub fn update(&mut self, high: f64, low: f64, close: f64) -> Result<Option<f64>, JsError> {
        let c = make_candle(high, low, close, 0.0)?;
        Ok(self.inner.update(c))
    }
    pub fn batch(
        &mut self,
        high: &[f64],
        low: &[f64],
        close: &[f64],
    ) -> Result<Float64Array, JsError> {
        if high.len() != low.len() || low.len() != close.len() {
            return Err(JsError::new("high, low, close must be equal length"));
        }
        let mut out = Vec::with_capacity(high.len());
        for i in 0..high.len() {
            let c = make_candle(high[i], low[i], close[i], 0.0)?;
            out.push(self.inner.update(c).unwrap_or(f64::NAN));
        }
        Ok(Float64Array::from(out.as_slice()))
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
}

#[wasm_bindgen(js_name = DX)]
pub struct WasmDx {
    inner: wc::Dx,
}

#[wasm_bindgen(js_class = DX)]
impl WasmDx {
    #[wasm_bindgen(constructor)]
    pub fn new(period: usize) -> Result<WasmDx, JsError> {
        Ok(Self {
            inner: wc::Dx::new(period).map_err(map_err)?,
        })
    }
    pub fn update(&mut self, high: f64, low: f64, close: f64) -> Result<Option<f64>, JsError> {
        let c = make_candle(high, low, close, 0.0)?;
        Ok(self.inner.update(c))
    }
    pub fn batch(
        &mut self,
        high: &[f64],
        low: &[f64],
        close: &[f64],
    ) -> Result<Float64Array, JsError> {
        if high.len() != low.len() || low.len() != close.len() {
            return Err(JsError::new("high, low, close must be equal length"));
        }
        let mut out = Vec::with_capacity(high.len());
        for i in 0..high.len() {
            let c = make_candle(high[i], low[i], close[i], 0.0)?;
            out.push(self.inner.update(c).unwrap_or(f64::NAN));
        }
        Ok(Float64Array::from(out.as_slice()))
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
}

#[wasm_bindgen(js_name = Stochastic)]
pub struct WasmStoch {
    inner: wc::Stochastic,
}

#[wasm_bindgen(js_class = Stochastic)]
impl WasmStoch {
    #[wasm_bindgen(constructor)]
    pub fn new(k_period: usize, d_period: usize) -> Result<WasmStoch, JsError> {
        Ok(Self {
            inner: wc::Stochastic::new(k_period, d_period).map_err(map_err)?,
        })
    }
    /// Returns `[k0, d0, k1, d1, ...]`, length `2 * n`.
    pub fn batch(
        &mut self,
        high: &[f64],
        low: &[f64],
        close: &[f64],
    ) -> Result<Float64Array, JsError> {
        let n = high.len();
        if low.len() != n || close.len() != n {
            return Err(JsError::new("high, low, close must be equal length"));
        }
        let mut out = vec![f64::NAN; n * 2];
        for i in 0..n {
            let c = make_candle(high[i], low[i], close[i], 0.0)?;
            if let Some(o) = self.inner.update(c) {
                out[i * 2] = o.k;
                out[i * 2 + 1] = o.d;
            }
        }
        Ok(Float64Array::from(out.as_slice()))
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
    /// Streaming update. Returns `{ k, d }` once warm, else `null`.
    pub fn update(&mut self, high: f64, low: f64, close: f64) -> Result<JsValue, JsError> {
        let c = make_candle(high, low, close, 0.0)?;
        Ok(match self.inner.update(c) {
            Some(o) => {
                let obj = Object::new();
                Reflect::set(&obj, &"k".into(), &o.k.into()).ok();
                Reflect::set(&obj, &"d".into(), &o.d.into()).ok();
                obj.into()
            }
            None => JsValue::NULL,
        })
    }
    #[wasm_bindgen(js_name = isReady)]
    pub fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    #[wasm_bindgen(js_name = warmupPeriod)]
    pub fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
}

#[wasm_bindgen(js_name = OBV)]
pub struct WasmObv {
    inner: wc::Obv,
}

impl Default for WasmObv {
    fn default() -> Self {
        Self::new()
    }
}

#[wasm_bindgen(js_class = OBV)]
impl WasmObv {
    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmObv {
        Self {
            inner: wc::Obv::new(),
        }
    }
    pub fn batch(&mut self, close: &[f64], volume: &[f64]) -> Result<Float64Array, JsError> {
        if close.len() != volume.len() {
            return Err(JsError::new("close and volume must be equal length"));
        }
        let mut out = Vec::with_capacity(close.len());
        for i in 0..close.len() {
            let c = make_candle(close[i], close[i], close[i], volume[i])?;
            out.push(self.inner.update(c).unwrap_or(f64::NAN));
        }
        Ok(Float64Array::from(out.as_slice()))
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
    pub fn update(&mut self, close: f64, volume: f64) -> Result<Option<f64>, JsError> {
        let c = make_candle(close, close, close, volume)?;
        Ok(self.inner.update(c))
    }
    #[wasm_bindgen(js_name = isReady)]
    pub fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    #[wasm_bindgen(js_name = warmupPeriod)]
    pub fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
}

#[wasm_bindgen(js_name = UltimateOscillator)]
pub struct WasmUltimateOscillator {
    inner: wc::UltimateOscillator,
}

#[wasm_bindgen(js_class = UltimateOscillator)]
impl WasmUltimateOscillator {
    #[wasm_bindgen(constructor)]
    pub fn new(short: usize, mid: usize, long: usize) -> Result<WasmUltimateOscillator, JsError> {
        Ok(Self {
            inner: wc::UltimateOscillator::new(short, mid, long).map_err(map_err)?,
        })
    }
    pub fn update(&mut self, high: f64, low: f64, close: f64) -> Result<Option<f64>, JsError> {
        let c = make_candle(high, low, close, 0.0)?;
        Ok(self.inner.update(c))
    }
    pub fn batch(
        &mut self,
        high: &[f64],
        low: &[f64],
        close: &[f64],
    ) -> Result<Float64Array, JsError> {
        if high.len() != low.len() || low.len() != close.len() {
            return Err(JsError::new("high, low, close must be equal length"));
        }
        let mut out = Vec::with_capacity(high.len());
        for i in 0..high.len() {
            let c = make_candle(high[i], low[i], close[i], 0.0)?;
            out.push(self.inner.update(c).unwrap_or(f64::NAN));
        }
        Ok(Float64Array::from(out.as_slice()))
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
}

#[wasm_bindgen(js_name = ADL)]
pub struct WasmAdl {
    inner: wc::Adl,
}

impl Default for WasmAdl {
    fn default() -> Self {
        Self::new()
    }
}

#[wasm_bindgen(js_class = ADL)]
impl WasmAdl {
    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmAdl {
        Self {
            inner: wc::Adl::new(),
        }
    }
    pub fn update(
        &mut self,
        high: f64,
        low: f64,
        close: f64,
        volume: f64,
    ) -> Result<Option<f64>, JsError> {
        let c = make_candle(high, low, close, volume)?;
        Ok(self.inner.update(c))
    }
    pub fn batch(
        &mut self,
        high: &[f64],
        low: &[f64],
        close: &[f64],
        volume: &[f64],
    ) -> Result<Float64Array, JsError> {
        let n = high.len();
        if low.len() != n || close.len() != n || volume.len() != n {
            return Err(JsError::new(
                "high, low, close, volume must be equal length",
            ));
        }
        let mut out = Vec::with_capacity(n);
        for i in 0..n {
            let c = make_candle(high[i], low[i], close[i], volume[i])?;
            out.push(self.inner.update(c).unwrap_or(f64::NAN));
        }
        Ok(Float64Array::from(out.as_slice()))
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
}

#[wasm_bindgen(js_name = VolumePriceTrend)]
pub struct WasmVolumePriceTrend {
    inner: wc::VolumePriceTrend,
}

impl Default for WasmVolumePriceTrend {
    fn default() -> Self {
        Self::new()
    }
}

#[wasm_bindgen(js_class = VolumePriceTrend)]
impl WasmVolumePriceTrend {
    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmVolumePriceTrend {
        Self {
            inner: wc::VolumePriceTrend::new(),
        }
    }
    pub fn update(&mut self, close: f64, volume: f64) -> Result<Option<f64>, JsError> {
        let c = make_candle(close, close, close, volume)?;
        Ok(self.inner.update(c))
    }
    pub fn batch(&mut self, close: &[f64], volume: &[f64]) -> Result<Float64Array, JsError> {
        if close.len() != volume.len() {
            return Err(JsError::new("close and volume must be equal length"));
        }
        let mut out = Vec::with_capacity(close.len());
        for i in 0..close.len() {
            let c = make_candle(close[i], close[i], close[i], volume[i])?;
            out.push(self.inner.update(c).unwrap_or(f64::NAN));
        }
        Ok(Float64Array::from(out.as_slice()))
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
}

#[wasm_bindgen(js_name = ChaikinMoneyFlow)]
pub struct WasmChaikinMoneyFlow {
    inner: wc::ChaikinMoneyFlow,
}

#[wasm_bindgen(js_class = ChaikinMoneyFlow)]
impl WasmChaikinMoneyFlow {
    #[wasm_bindgen(constructor)]
    pub fn new(period: usize) -> Result<WasmChaikinMoneyFlow, JsError> {
        Ok(Self {
            inner: wc::ChaikinMoneyFlow::new(period).map_err(map_err)?,
        })
    }
    pub fn update(
        &mut self,
        high: f64,
        low: f64,
        close: f64,
        volume: f64,
    ) -> Result<Option<f64>, JsError> {
        let c = make_candle(high, low, close, volume)?;
        Ok(self.inner.update(c))
    }
    pub fn batch(
        &mut self,
        high: &[f64],
        low: &[f64],
        close: &[f64],
        volume: &[f64],
    ) -> Result<Float64Array, JsError> {
        let n = high.len();
        if low.len() != n || close.len() != n || volume.len() != n {
            return Err(JsError::new(
                "high, low, close, volume must be equal length",
            ));
        }
        let mut out = Vec::with_capacity(n);
        for i in 0..n {
            let c = make_candle(high[i], low[i], close[i], volume[i])?;
            out.push(self.inner.update(c).unwrap_or(f64::NAN));
        }
        Ok(Float64Array::from(out.as_slice()))
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
}

#[wasm_bindgen(js_name = ChaikinOscillator)]
pub struct WasmChaikinOscillator {
    inner: wc::ChaikinOscillator,
}

#[wasm_bindgen(js_class = ChaikinOscillator)]
impl WasmChaikinOscillator {
    #[wasm_bindgen(constructor)]
    pub fn new(fast: usize, slow: usize) -> Result<WasmChaikinOscillator, JsError> {
        Ok(Self {
            inner: wc::ChaikinOscillator::new(fast, slow).map_err(map_err)?,
        })
    }
    pub fn update(
        &mut self,
        high: f64,
        low: f64,
        close: f64,
        volume: f64,
    ) -> Result<Option<f64>, JsError> {
        let c = make_candle(high, low, close, volume)?;
        Ok(self.inner.update(c))
    }
    pub fn batch(
        &mut self,
        high: &[f64],
        low: &[f64],
        close: &[f64],
        volume: &[f64],
    ) -> Result<Float64Array, JsError> {
        let n = high.len();
        if low.len() != n || close.len() != n || volume.len() != n {
            return Err(JsError::new(
                "high, low, close, volume must be equal length",
            ));
        }
        let mut out = Vec::with_capacity(n);
        for i in 0..n {
            let c = make_candle(high[i], low[i], close[i], volume[i])?;
            out.push(self.inner.update(c).unwrap_or(f64::NAN));
        }
        Ok(Float64Array::from(out.as_slice()))
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
}

#[wasm_bindgen(js_name = ForceIndex)]
pub struct WasmForceIndex {
    inner: wc::ForceIndex,
}

#[wasm_bindgen(js_class = ForceIndex)]
impl WasmForceIndex {
    #[wasm_bindgen(constructor)]
    pub fn new(period: usize) -> Result<WasmForceIndex, JsError> {
        Ok(Self {
            inner: wc::ForceIndex::new(period).map_err(map_err)?,
        })
    }
    pub fn update(&mut self, close: f64, volume: f64) -> Result<Option<f64>, JsError> {
        let c = make_candle(close, close, close, volume)?;
        Ok(self.inner.update(c))
    }
    pub fn batch(&mut self, close: &[f64], volume: &[f64]) -> Result<Float64Array, JsError> {
        if close.len() != volume.len() {
            return Err(JsError::new("close and volume must be equal length"));
        }
        let mut out = Vec::with_capacity(close.len());
        for i in 0..close.len() {
            let c = make_candle(close[i], close[i], close[i], volume[i])?;
            out.push(self.inner.update(c).unwrap_or(f64::NAN));
        }
        Ok(Float64Array::from(out.as_slice()))
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
}

#[wasm_bindgen(js_name = VolumeOscillator)]
pub struct WasmVolumeOscillator {
    inner: wc::VolumeOscillator,
}

#[wasm_bindgen(js_class = VolumeOscillator)]
impl WasmVolumeOscillator {
    #[wasm_bindgen(constructor)]
    pub fn new(fast: usize, slow: usize) -> Result<WasmVolumeOscillator, JsError> {
        Ok(Self {
            inner: wc::VolumeOscillator::new(fast, slow).map_err(map_err)?,
        })
    }
    pub fn update(&mut self, volume: f64) -> Result<Option<f64>, JsError> {
        let c = make_candle(10.0, 10.0, 10.0, volume)?;
        Ok(self.inner.update(c))
    }
    pub fn batch(&mut self, volume: &[f64]) -> Result<Float64Array, JsError> {
        let mut out = Vec::with_capacity(volume.len());
        for &v in volume {
            let c = make_candle(10.0, 10.0, 10.0, v)?;
            out.push(self.inner.update(c).unwrap_or(f64::NAN));
        }
        Ok(Float64Array::from(out.as_slice()))
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
}

#[wasm_bindgen(js_name = NVI)]
pub struct WasmNvi {
    inner: wc::Nvi,
}

#[wasm_bindgen(js_class = NVI)]
impl WasmNvi {
    #[wasm_bindgen(constructor)]
    pub fn new(baseline: Option<f64>) -> WasmNvi {
        Self {
            inner: wc::Nvi::with_baseline(baseline.unwrap_or(1000.0)),
        }
    }
    pub fn update(&mut self, close: f64, volume: f64) -> Result<Option<f64>, JsError> {
        let c = make_candle(close, close, close, volume)?;
        Ok(self.inner.update(c))
    }
    pub fn batch(&mut self, close: &[f64], volume: &[f64]) -> Result<Float64Array, JsError> {
        if close.len() != volume.len() {
            return Err(JsError::new("close and volume must be equal length"));
        }
        let mut out = Vec::with_capacity(close.len());
        for i in 0..close.len() {
            let c = make_candle(close[i], close[i], close[i], volume[i])?;
            out.push(self.inner.update(c).unwrap_or(f64::NAN));
        }
        Ok(Float64Array::from(out.as_slice()))
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
}

#[wasm_bindgen(js_name = PVI)]
pub struct WasmPvi {
    inner: wc::Pvi,
}

#[wasm_bindgen(js_class = PVI)]
impl WasmPvi {
    #[wasm_bindgen(constructor)]
    pub fn new(baseline: Option<f64>) -> WasmPvi {
        Self {
            inner: wc::Pvi::with_baseline(baseline.unwrap_or(1000.0)),
        }
    }
    pub fn update(&mut self, close: f64, volume: f64) -> Result<Option<f64>, JsError> {
        let c = make_candle(close, close, close, volume)?;
        Ok(self.inner.update(c))
    }
    pub fn batch(&mut self, close: &[f64], volume: &[f64]) -> Result<Float64Array, JsError> {
        if close.len() != volume.len() {
            return Err(JsError::new("close and volume must be equal length"));
        }
        let mut out = Vec::with_capacity(close.len());
        for i in 0..close.len() {
            let c = make_candle(close[i], close[i], close[i], volume[i])?;
            out.push(self.inner.update(c).unwrap_or(f64::NAN));
        }
        Ok(Float64Array::from(out.as_slice()))
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
}

#[wasm_bindgen(js_name = KVO)]
pub struct WasmKvo {
    inner: wc::Kvo,
}

#[wasm_bindgen(js_class = KVO)]
impl WasmKvo {
    #[wasm_bindgen(constructor)]
    pub fn new(fast: usize, slow: usize) -> Result<WasmKvo, JsError> {
        Ok(Self {
            inner: wc::Kvo::new(fast, slow).map_err(map_err)?,
        })
    }
    pub fn update(
        &mut self,
        high: f64,
        low: f64,
        close: f64,
        volume: f64,
    ) -> Result<Option<f64>, JsError> {
        let c = make_candle(high, low, close, volume)?;
        Ok(self.inner.update(c))
    }
    pub fn batch(
        &mut self,
        high: &[f64],
        low: &[f64],
        close: &[f64],
        volume: &[f64],
    ) -> Result<Float64Array, JsError> {
        let n = high.len();
        if low.len() != n || close.len() != n || volume.len() != n {
            return Err(JsError::new(
                "high, low, close, volume must be equal length",
            ));
        }
        let mut out = Vec::with_capacity(n);
        for i in 0..n {
            let c = make_candle(high[i], low[i], close[i], volume[i])?;
            out.push(self.inner.update(c).unwrap_or(f64::NAN));
        }
        Ok(Float64Array::from(out.as_slice()))
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
}

#[wasm_bindgen(js_name = WilliamsAD)]
pub struct WasmAdOscillator {
    inner: wc::AdOscillator,
}

#[wasm_bindgen(js_class = WilliamsAD)]
impl WasmAdOscillator {
    #[wasm_bindgen(constructor)]
    #[allow(clippy::new_without_default)]
    pub fn new() -> WasmAdOscillator {
        Self {
            inner: wc::AdOscillator::new(),
        }
    }
    pub fn update(&mut self, high: f64, low: f64, close: f64) -> Result<Option<f64>, JsError> {
        let c = make_candle(high, low, close, 0.0)?;
        Ok(self.inner.update(c))
    }
    pub fn batch(
        &mut self,
        high: &[f64],
        low: &[f64],
        close: &[f64],
    ) -> Result<Float64Array, JsError> {
        let n = high.len();
        if low.len() != n || close.len() != n {
            return Err(JsError::new("high, low, close must be equal length"));
        }
        let mut out = Vec::with_capacity(n);
        for i in 0..n {
            let c = make_candle(high[i], low[i], close[i], 0.0)?;
            out.push(self.inner.update(c).unwrap_or(f64::NAN));
        }
        Ok(Float64Array::from(out.as_slice()))
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
}

#[wasm_bindgen(js_name = AnchoredRSI)]
pub struct WasmAnchoredRsi {
    inner: wc::AnchoredRsi,
}

#[wasm_bindgen(js_class = AnchoredRSI)]
impl WasmAnchoredRsi {
    #[wasm_bindgen(constructor)]
    #[allow(clippy::new_without_default)]
    pub fn new() -> WasmAnchoredRsi {
        Self {
            inner: wc::AnchoredRsi::new(),
        }
    }
    #[wasm_bindgen(js_name = setAnchor)]
    pub fn set_anchor(&mut self) {
        self.inner.set_anchor();
    }
    pub fn update(&mut self, value: f64) -> Option<f64> {
        self.inner.update(value)
    }
    pub fn batch(&mut self, prices: &[f64]) -> Float64Array {
        let out = flatten(self.inner.batch(prices));
        Float64Array::from(out.as_slice())
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
    #[wasm_bindgen(js_name = isReady)]
    pub fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    #[wasm_bindgen(js_name = warmupPeriod)]
    pub fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
}

#[wasm_bindgen(js_name = AnchoredVWAP)]
pub struct WasmAnchoredVwap {
    inner: wc::AnchoredVwap,
}

#[wasm_bindgen(js_class = AnchoredVWAP)]
impl WasmAnchoredVwap {
    #[wasm_bindgen(constructor)]
    #[allow(clippy::new_without_default)]
    pub fn new() -> WasmAnchoredVwap {
        Self {
            inner: wc::AnchoredVwap::new(),
        }
    }
    #[wasm_bindgen(js_name = setAnchor)]
    pub fn set_anchor(&mut self) {
        self.inner.set_anchor();
    }
    pub fn update(
        &mut self,
        high: f64,
        low: f64,
        close: f64,
        volume: f64,
    ) -> Result<Option<f64>, JsError> {
        let c = make_candle(high, low, close, volume)?;
        Ok(self.inner.update(c))
    }
    pub fn batch(
        &mut self,
        high: &[f64],
        low: &[f64],
        close: &[f64],
        volume: &[f64],
    ) -> Result<Float64Array, JsError> {
        let n = high.len();
        if low.len() != n || close.len() != n || volume.len() != n {
            return Err(JsError::new(
                "high, low, close, volume must be equal length",
            ));
        }
        let mut out = Vec::with_capacity(n);
        for i in 0..n {
            let c = make_candle(high[i], low[i], close[i], volume[i])?;
            out.push(self.inner.update(c).unwrap_or(f64::NAN));
        }
        Ok(Float64Array::from(out.as_slice()))
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
}

#[wasm_bindgen(js_name = DemandIndex)]
pub struct WasmDemandIndex {
    inner: wc::DemandIndex,
}

#[wasm_bindgen(js_class = DemandIndex)]
impl WasmDemandIndex {
    #[wasm_bindgen(constructor)]
    pub fn new(period: usize) -> Result<WasmDemandIndex, JsError> {
        Ok(Self {
            inner: wc::DemandIndex::new(period).map_err(map_err)?,
        })
    }
    pub fn update(
        &mut self,
        high: f64,
        low: f64,
        close: f64,
        volume: f64,
    ) -> Result<Option<f64>, JsError> {
        let c = make_candle(high, low, close, volume)?;
        Ok(self.inner.update(c))
    }
    pub fn batch(
        &mut self,
        high: &[f64],
        low: &[f64],
        close: &[f64],
        volume: &[f64],
    ) -> Result<Float64Array, JsError> {
        let n = high.len();
        if low.len() != n || close.len() != n || volume.len() != n {
            return Err(JsError::new(
                "high, low, close, volume must be equal length",
            ));
        }
        let mut out = Vec::with_capacity(n);
        for i in 0..n {
            let c = make_candle(high[i], low[i], close[i], volume[i])?;
            out.push(self.inner.update(c).unwrap_or(f64::NAN));
        }
        Ok(Float64Array::from(out.as_slice()))
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
}

#[wasm_bindgen(js_name = TSV)]
pub struct WasmTsv {
    inner: wc::Tsv,
}

#[wasm_bindgen(js_class = TSV)]
impl WasmTsv {
    #[wasm_bindgen(constructor)]
    pub fn new(period: usize) -> Result<WasmTsv, JsError> {
        Ok(Self {
            inner: wc::Tsv::new(period).map_err(map_err)?,
        })
    }
    pub fn update(&mut self, close: f64, volume: f64) -> Result<Option<f64>, JsError> {
        let c = make_candle(close, close, close, volume)?;
        Ok(self.inner.update(c))
    }
    pub fn batch(&mut self, close: &[f64], volume: &[f64]) -> Result<Float64Array, JsError> {
        if close.len() != volume.len() {
            return Err(JsError::new("close and volume must be equal length"));
        }
        let mut out = Vec::with_capacity(close.len());
        for i in 0..close.len() {
            let c = make_candle(close[i], close[i], close[i], volume[i])?;
            out.push(self.inner.update(c).unwrap_or(f64::NAN));
        }
        Ok(Float64Array::from(out.as_slice()))
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
}

#[wasm_bindgen(js_name = VZO)]
pub struct WasmVzo {
    inner: wc::Vzo,
}

#[wasm_bindgen(js_class = VZO)]
impl WasmVzo {
    #[wasm_bindgen(constructor)]
    pub fn new(period: usize) -> Result<WasmVzo, JsError> {
        Ok(Self {
            inner: wc::Vzo::new(period).map_err(map_err)?,
        })
    }
    pub fn update(&mut self, close: f64, volume: f64) -> Result<Option<f64>, JsError> {
        let c = make_candle(close, close, close, volume)?;
        Ok(self.inner.update(c))
    }
    pub fn batch(&mut self, close: &[f64], volume: &[f64]) -> Result<Float64Array, JsError> {
        if close.len() != volume.len() {
            return Err(JsError::new("close and volume must be equal length"));
        }
        let mut out = Vec::with_capacity(close.len());
        for i in 0..close.len() {
            let c = make_candle(close[i], close[i], close[i], volume[i])?;
            out.push(self.inner.update(c).unwrap_or(f64::NAN));
        }
        Ok(Float64Array::from(out.as_slice()))
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
}

#[wasm_bindgen(js_name = MarketFacilitationIndex)]
pub struct WasmMarketFacilitationIndex {
    inner: wc::MarketFacilitationIndex,
}

#[wasm_bindgen(js_class = MarketFacilitationIndex)]
impl WasmMarketFacilitationIndex {
    #[wasm_bindgen(constructor)]
    #[allow(clippy::new_without_default)]
    pub fn new() -> WasmMarketFacilitationIndex {
        Self {
            inner: wc::MarketFacilitationIndex::new(),
        }
    }
    pub fn update(&mut self, high: f64, low: f64, volume: f64) -> Result<Option<f64>, JsError> {
        let c = make_candle(high, low, low, volume)?;
        Ok(self.inner.update(c))
    }
    pub fn batch(
        &mut self,
        high: &[f64],
        low: &[f64],
        volume: &[f64],
    ) -> Result<Float64Array, JsError> {
        let n = high.len();
        if low.len() != n || volume.len() != n {
            return Err(JsError::new("high, low, volume must be equal length"));
        }
        let mut out = Vec::with_capacity(n);
        for i in 0..n {
            let c = make_candle(high[i], low[i], low[i], volume[i])?;
            out.push(self.inner.update(c).unwrap_or(f64::NAN));
        }
        Ok(Float64Array::from(out.as_slice()))
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
}

#[wasm_bindgen(js_name = EaseOfMovement)]
pub struct WasmEaseOfMovement {
    inner: wc::EaseOfMovement,
}

#[wasm_bindgen(js_class = EaseOfMovement)]
impl WasmEaseOfMovement {
    #[wasm_bindgen(constructor)]
    pub fn new(period: usize, divisor: f64) -> Result<WasmEaseOfMovement, JsError> {
        Ok(Self {
            inner: wc::EaseOfMovement::with_divisor(period, divisor).map_err(map_err)?,
        })
    }
    pub fn update(&mut self, high: f64, low: f64, volume: f64) -> Result<Option<f64>, JsError> {
        let c = make_candle(high, low, low, volume)?;
        Ok(self.inner.update(c))
    }
    pub fn batch(
        &mut self,
        high: &[f64],
        low: &[f64],
        volume: &[f64],
    ) -> Result<Float64Array, JsError> {
        let n = high.len();
        if low.len() != n || volume.len() != n {
            return Err(JsError::new("high, low, volume must be equal length"));
        }
        let mut out = Vec::with_capacity(n);
        for i in 0..n {
            let c = make_candle(high[i], low[i], low[i], volume[i])?;
            out.push(self.inner.update(c).unwrap_or(f64::NAN));
        }
        Ok(Float64Array::from(out.as_slice()))
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
}

#[wasm_bindgen(js_name = SuperTrend)]
pub struct WasmSuperTrend {
    inner: wc::SuperTrend,
}

#[wasm_bindgen(js_class = SuperTrend)]
impl WasmSuperTrend {
    #[wasm_bindgen(constructor)]
    pub fn new(atr_period: usize, multiplier: f64) -> Result<WasmSuperTrend, JsError> {
        Ok(Self {
            inner: wc::SuperTrend::new(atr_period, multiplier).map_err(map_err)?,
        })
    }
    /// Returns `{ value, direction }` once warm, else `null`.
    pub fn update(&mut self, high: f64, low: f64, close: f64) -> Result<JsValue, JsError> {
        let c = make_candle(high, low, close, 0.0)?;
        Ok(match self.inner.update(c) {
            Some(o) => {
                let obj = Object::new();
                Reflect::set(&obj, &"value".into(), &o.value.into()).ok();
                Reflect::set(&obj, &"direction".into(), &o.direction.into()).ok();
                obj.into()
            }
            None => JsValue::NULL,
        })
    }
    /// Returns `[value0, direction0, value1, direction1, ...]`, length `2 * n`.
    /// Warmup positions are NaN.
    pub fn batch(
        &mut self,
        high: &[f64],
        low: &[f64],
        close: &[f64],
    ) -> Result<Float64Array, JsError> {
        let n = high.len();
        if low.len() != n || close.len() != n {
            return Err(JsError::new("high, low, close must be equal length"));
        }
        let mut out = vec![f64::NAN; n * 2];
        for i in 0..n {
            let c = make_candle(high[i], low[i], close[i], 0.0)?;
            if let Some(o) = self.inner.update(c) {
                out[i * 2] = o.value;
                out[i * 2 + 1] = o.direction;
            }
        }
        Ok(Float64Array::from(out.as_slice()))
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
}

#[wasm_bindgen(js_name = ChandelierExit)]
pub struct WasmChandelierExit {
    inner: wc::ChandelierExit,
}

#[wasm_bindgen(js_class = ChandelierExit)]
impl WasmChandelierExit {
    #[wasm_bindgen(constructor)]
    pub fn new(period: usize, multiplier: f64) -> Result<WasmChandelierExit, JsError> {
        Ok(Self {
            inner: wc::ChandelierExit::new(period, multiplier).map_err(map_err)?,
        })
    }
    /// Returns `{ longStop, shortStop }` once warm, else `null`.
    pub fn update(&mut self, high: f64, low: f64, close: f64) -> Result<JsValue, JsError> {
        let c = make_candle(high, low, close, 0.0)?;
        Ok(match self.inner.update(c) {
            Some(o) => {
                let obj = Object::new();
                Reflect::set(&obj, &"longStop".into(), &o.long_stop.into()).ok();
                Reflect::set(&obj, &"shortStop".into(), &o.short_stop.into()).ok();
                obj.into()
            }
            None => JsValue::NULL,
        })
    }
    /// Returns `[long0, short0, long1, short1, ...]`, length `2 * n`. Warmup is NaN.
    pub fn batch(
        &mut self,
        high: &[f64],
        low: &[f64],
        close: &[f64],
    ) -> Result<Float64Array, JsError> {
        let n = high.len();
        if low.len() != n || close.len() != n {
            return Err(JsError::new("high, low, close must be equal length"));
        }
        let mut out = vec![f64::NAN; n * 2];
        for i in 0..n {
            let c = make_candle(high[i], low[i], close[i], 0.0)?;
            if let Some(o) = self.inner.update(c) {
                out[i * 2] = o.long_stop;
                out[i * 2 + 1] = o.short_stop;
            }
        }
        Ok(Float64Array::from(out.as_slice()))
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
}

#[wasm_bindgen(js_name = ChandeKrollStop)]
pub struct WasmChandeKrollStop {
    inner: wc::ChandeKrollStop,
}

#[wasm_bindgen(js_class = ChandeKrollStop)]
impl WasmChandeKrollStop {
    #[wasm_bindgen(constructor)]
    pub fn new(
        atr_period: usize,
        atr_multiplier: f64,
        stop_period: usize,
    ) -> Result<WasmChandeKrollStop, JsError> {
        Ok(Self {
            inner: wc::ChandeKrollStop::new(atr_period, atr_multiplier, stop_period)
                .map_err(map_err)?,
        })
    }
    /// Returns `{ stopLong, stopShort }` once warm, else `null`.
    pub fn update(&mut self, high: f64, low: f64, close: f64) -> Result<JsValue, JsError> {
        let c = make_candle(high, low, close, 0.0)?;
        Ok(match self.inner.update(c) {
            Some(o) => {
                let obj = Object::new();
                Reflect::set(&obj, &"stopLong".into(), &o.stop_long.into()).ok();
                Reflect::set(&obj, &"stopShort".into(), &o.stop_short.into()).ok();
                obj.into()
            }
            None => JsValue::NULL,
        })
    }
    /// Returns `[long0, short0, long1, short1, ...]`, length `2 * n`. Warmup is NaN.
    pub fn batch(
        &mut self,
        high: &[f64],
        low: &[f64],
        close: &[f64],
    ) -> Result<Float64Array, JsError> {
        let n = high.len();
        if low.len() != n || close.len() != n {
            return Err(JsError::new("high, low, close must be equal length"));
        }
        let mut out = vec![f64::NAN; n * 2];
        for i in 0..n {
            let c = make_candle(high[i], low[i], close[i], 0.0)?;
            if let Some(o) = self.inner.update(c) {
                out[i * 2] = o.stop_long;
                out[i * 2 + 1] = o.stop_short;
            }
        }
        Ok(Float64Array::from(out.as_slice()))
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
}

#[wasm_bindgen(js_name = AtrTrailingStop)]
pub struct WasmAtrTrailingStop {
    inner: wc::AtrTrailingStop,
}

#[wasm_bindgen(js_class = AtrTrailingStop)]
impl WasmAtrTrailingStop {
    #[wasm_bindgen(constructor)]
    pub fn new(atr_period: usize, multiplier: f64) -> Result<WasmAtrTrailingStop, JsError> {
        Ok(Self {
            inner: wc::AtrTrailingStop::new(atr_period, multiplier).map_err(map_err)?,
        })
    }
    pub fn update(&mut self, high: f64, low: f64, close: f64) -> Result<Option<f64>, JsError> {
        let c = make_candle(high, low, close, 0.0)?;
        Ok(self.inner.update(c))
    }
    pub fn batch(
        &mut self,
        high: &[f64],
        low: &[f64],
        close: &[f64],
    ) -> Result<Float64Array, JsError> {
        let n = high.len();
        if low.len() != n || close.len() != n {
            return Err(JsError::new("high, low, close must be equal length"));
        }
        let mut out = Vec::with_capacity(n);
        for i in 0..n {
            let c = make_candle(high[i], low[i], close[i], 0.0)?;
            out.push(self.inner.update(c).unwrap_or(f64::NAN));
        }
        Ok(Float64Array::from(out.as_slice()))
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
}

// ---------- Trailing Stops (family 09) ----------

#[wasm_bindgen(js_name = HiLoActivator)]
pub struct WasmHiLoActivator {
    inner: wc::HiLoActivator,
}

#[wasm_bindgen(js_class = HiLoActivator)]
impl WasmHiLoActivator {
    #[wasm_bindgen(constructor)]
    pub fn new(period: usize) -> Result<WasmHiLoActivator, JsError> {
        Ok(Self {
            inner: wc::HiLoActivator::new(period).map_err(map_err)?,
        })
    }
    pub fn update(&mut self, high: f64, low: f64, close: f64) -> Result<Option<f64>, JsError> {
        let c = make_candle(high, low, close, 0.0)?;
        Ok(self.inner.update(c))
    }
    pub fn batch(
        &mut self,
        high: &[f64],
        low: &[f64],
        close: &[f64],
    ) -> Result<Float64Array, JsError> {
        let n = high.len();
        if low.len() != n || close.len() != n {
            return Err(JsError::new("high, low, close must be equal length"));
        }
        let mut out = Vec::with_capacity(n);
        for i in 0..n {
            let c = make_candle(high[i], low[i], close[i], 0.0)?;
            out.push(self.inner.update(c).unwrap_or(f64::NAN));
        }
        Ok(Float64Array::from(out.as_slice()))
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
}

#[wasm_bindgen(js_name = VoltyStop)]
pub struct WasmVoltyStop {
    inner: wc::VoltyStop,
}

#[wasm_bindgen(js_class = VoltyStop)]
impl WasmVoltyStop {
    #[wasm_bindgen(constructor)]
    pub fn new(atr_period: usize, multiplier: f64) -> Result<WasmVoltyStop, JsError> {
        Ok(Self {
            inner: wc::VoltyStop::new(atr_period, multiplier).map_err(map_err)?,
        })
    }
    pub fn update(&mut self, high: f64, low: f64, close: f64) -> Result<Option<f64>, JsError> {
        let c = make_candle(high, low, close, 0.0)?;
        Ok(self.inner.update(c))
    }
    pub fn batch(
        &mut self,
        high: &[f64],
        low: &[f64],
        close: &[f64],
    ) -> Result<Float64Array, JsError> {
        let n = high.len();
        if low.len() != n || close.len() != n {
            return Err(JsError::new("high, low, close must be equal length"));
        }
        let mut out = Vec::with_capacity(n);
        for i in 0..n {
            let c = make_candle(high[i], low[i], close[i], 0.0)?;
            out.push(self.inner.update(c).unwrap_or(f64::NAN));
        }
        Ok(Float64Array::from(out.as_slice()))
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
}

#[wasm_bindgen(js_name = YoyoExit)]
pub struct WasmYoyoExit {
    inner: wc::YoyoExit,
}

#[wasm_bindgen(js_class = YoyoExit)]
impl WasmYoyoExit {
    #[wasm_bindgen(constructor)]
    pub fn new(atr_period: usize, multiplier: f64) -> Result<WasmYoyoExit, JsError> {
        Ok(Self {
            inner: wc::YoyoExit::new(atr_period, multiplier).map_err(map_err)?,
        })
    }
    pub fn update(&mut self, high: f64, low: f64, close: f64) -> Result<Option<f64>, JsError> {
        let c = make_candle(high, low, close, 0.0)?;
        Ok(self.inner.update(c))
    }
    pub fn batch(
        &mut self,
        high: &[f64],
        low: &[f64],
        close: &[f64],
    ) -> Result<Float64Array, JsError> {
        let n = high.len();
        if low.len() != n || close.len() != n {
            return Err(JsError::new("high, low, close must be equal length"));
        }
        let mut out = Vec::with_capacity(n);
        for i in 0..n {
            let c = make_candle(high[i], low[i], close[i], 0.0)?;
            out.push(self.inner.update(c).unwrap_or(f64::NAN));
        }
        Ok(Float64Array::from(out.as_slice()))
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
    #[wasm_bindgen(js_name = inTrade)]
    pub fn in_trade(&self) -> bool {
        self.inner.in_trade()
    }
}

#[wasm_bindgen(js_name = DonchianStop)]
pub struct WasmDonchianStop {
    inner: wc::DonchianStop,
}

#[wasm_bindgen(js_class = DonchianStop)]
impl WasmDonchianStop {
    #[wasm_bindgen(constructor)]
    pub fn new(period: usize) -> Result<WasmDonchianStop, JsError> {
        Ok(Self {
            inner: wc::DonchianStop::new(period).map_err(map_err)?,
        })
    }
    /// Returns `{ stopLong, stopShort }` once warm, else `null`.
    pub fn update(&mut self, high: f64, low: f64) -> Result<JsValue, JsError> {
        let c = make_candle(high, low, low, 0.0)?;
        Ok(match self.inner.update(c) {
            Some(o) => {
                let obj = Object::new();
                Reflect::set(&obj, &"stopLong".into(), &o.stop_long.into()).ok();
                Reflect::set(&obj, &"stopShort".into(), &o.stop_short.into()).ok();
                obj.into()
            }
            None => JsValue::NULL,
        })
    }
    /// Returns `[long0, short0, long1, short1, ...]`, length `2 * n`. Warmup is NaN.
    pub fn batch(&mut self, high: &[f64], low: &[f64]) -> Result<Float64Array, JsError> {
        let n = high.len();
        if low.len() != n {
            return Err(JsError::new("high and low must be equal length"));
        }
        let mut out = vec![f64::NAN; n * 2];
        for i in 0..n {
            let c = make_candle(high[i], low[i], low[i], 0.0)?;
            if let Some(o) = self.inner.update(c) {
                out[i * 2] = o.stop_long;
                out[i * 2 + 1] = o.stop_short;
            }
        }
        Ok(Float64Array::from(out.as_slice()))
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
}

#[wasm_bindgen(js_name = PercentageTrailingStop)]
pub struct WasmPercentageTrailingStop {
    inner: wc::PercentageTrailingStop,
}

#[wasm_bindgen(js_class = PercentageTrailingStop)]
impl WasmPercentageTrailingStop {
    #[wasm_bindgen(constructor)]
    pub fn new(percent: f64) -> Result<WasmPercentageTrailingStop, JsError> {
        Ok(Self {
            inner: wc::PercentageTrailingStop::new(percent).map_err(map_err)?,
        })
    }
    pub fn update(&mut self, value: f64) -> Option<f64> {
        self.inner.update(value)
    }
    pub fn batch(&mut self, prices: &[f64]) -> Float64Array {
        let out = flatten(self.inner.batch(prices));
        Float64Array::from(out.as_slice())
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
}

#[wasm_bindgen(js_name = StepTrailingStop)]
pub struct WasmStepTrailingStop {
    inner: wc::StepTrailingStop,
}

#[wasm_bindgen(js_class = StepTrailingStop)]
impl WasmStepTrailingStop {
    #[wasm_bindgen(constructor)]
    pub fn new(step_size: f64) -> Result<WasmStepTrailingStop, JsError> {
        Ok(Self {
            inner: wc::StepTrailingStop::new(step_size).map_err(map_err)?,
        })
    }
    pub fn update(&mut self, value: f64) -> Option<f64> {
        self.inner.update(value)
    }
    pub fn batch(&mut self, prices: &[f64]) -> Float64Array {
        let out = flatten(self.inner.batch(prices));
        Float64Array::from(out.as_slice())
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
}

#[wasm_bindgen(js_name = RenkoTrailingStop)]
pub struct WasmRenkoTrailingStop {
    inner: wc::RenkoTrailingStop,
}

#[wasm_bindgen(js_class = RenkoTrailingStop)]
impl WasmRenkoTrailingStop {
    #[wasm_bindgen(constructor)]
    pub fn new(block_size: f64) -> Result<WasmRenkoTrailingStop, JsError> {
        Ok(Self {
            inner: wc::RenkoTrailingStop::new(block_size).map_err(map_err)?,
        })
    }
    pub fn update(&mut self, value: f64) -> Option<f64> {
        self.inner.update(value)
    }
    pub fn batch(&mut self, prices: &[f64]) -> Float64Array {
        let out = flatten(self.inner.batch(prices));
        Float64Array::from(out.as_slice())
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
}

#[wasm_bindgen(js_name = TypicalPrice)]
pub struct WasmTypicalPrice {
    inner: wc::TypicalPrice,
}

impl Default for WasmTypicalPrice {
    fn default() -> Self {
        Self::new()
    }
}

#[wasm_bindgen(js_class = TypicalPrice)]
impl WasmTypicalPrice {
    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmTypicalPrice {
        Self {
            inner: wc::TypicalPrice::new(),
        }
    }
    pub fn update(&mut self, high: f64, low: f64, close: f64) -> Result<Option<f64>, JsError> {
        let c = make_candle(high, low, close, 0.0)?;
        Ok(self.inner.update(c))
    }
    pub fn batch(
        &mut self,
        high: &[f64],
        low: &[f64],
        close: &[f64],
    ) -> Result<Float64Array, JsError> {
        let n = high.len();
        if low.len() != n || close.len() != n {
            return Err(JsError::new("high, low, close must be equal length"));
        }
        let mut out = Vec::with_capacity(n);
        for i in 0..n {
            let c = make_candle(high[i], low[i], close[i], 0.0)?;
            out.push(self.inner.update(c).unwrap_or(f64::NAN));
        }
        Ok(Float64Array::from(out.as_slice()))
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
}

#[wasm_bindgen(js_name = MedianPrice)]
pub struct WasmMedianPrice {
    inner: wc::MedianPrice,
}

impl Default for WasmMedianPrice {
    fn default() -> Self {
        Self::new()
    }
}

#[wasm_bindgen(js_class = MedianPrice)]
impl WasmMedianPrice {
    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmMedianPrice {
        Self {
            inner: wc::MedianPrice::new(),
        }
    }
    pub fn update(&mut self, high: f64, low: f64) -> Result<Option<f64>, JsError> {
        let c = make_candle(high, low, low, 0.0)?;
        Ok(self.inner.update(c))
    }
    pub fn batch(&mut self, high: &[f64], low: &[f64]) -> Result<Float64Array, JsError> {
        if high.len() != low.len() {
            return Err(JsError::new("high and low must be equal length"));
        }
        let mut out = Vec::with_capacity(high.len());
        for i in 0..high.len() {
            let c = make_candle(high[i], low[i], low[i], 0.0)?;
            out.push(self.inner.update(c).unwrap_or(f64::NAN));
        }
        Ok(Float64Array::from(out.as_slice()))
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
}

#[wasm_bindgen(js_name = WeightedClose)]
pub struct WasmWeightedClose {
    inner: wc::WeightedClose,
}

impl Default for WasmWeightedClose {
    fn default() -> Self {
        Self::new()
    }
}

#[wasm_bindgen(js_class = WeightedClose)]
impl WasmWeightedClose {
    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmWeightedClose {
        Self {
            inner: wc::WeightedClose::new(),
        }
    }
    pub fn update(&mut self, high: f64, low: f64, close: f64) -> Result<Option<f64>, JsError> {
        let c = make_candle(high, low, close, 0.0)?;
        Ok(self.inner.update(c))
    }
    pub fn batch(
        &mut self,
        high: &[f64],
        low: &[f64],
        close: &[f64],
    ) -> Result<Float64Array, JsError> {
        let n = high.len();
        if low.len() != n || close.len() != n {
            return Err(JsError::new("high, low, close must be equal length"));
        }
        let mut out = Vec::with_capacity(n);
        for i in 0..n {
            let c = make_candle(high[i], low[i], close[i], 0.0)?;
            out.push(self.inner.update(c).unwrap_or(f64::NAN));
        }
        Ok(Float64Array::from(out.as_slice()))
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
}

#[wasm_bindgen(js_name = AcceleratorOscillator)]
pub struct WasmAcceleratorOscillator {
    inner: wc::AcceleratorOscillator,
}

#[wasm_bindgen(js_class = AcceleratorOscillator)]
impl WasmAcceleratorOscillator {
    #[wasm_bindgen(constructor)]
    pub fn new(
        ao_fast: usize,
        ao_slow: usize,
        signal_period: usize,
    ) -> Result<WasmAcceleratorOscillator, JsError> {
        Ok(Self {
            inner: wc::AcceleratorOscillator::new(ao_fast, ao_slow, signal_period)
                .map_err(map_err)?,
        })
    }
    pub fn update(&mut self, high: f64, low: f64) -> Result<Option<f64>, JsError> {
        let c = make_candle(high, low, low, 0.0)?;
        Ok(self.inner.update(c))
    }
    pub fn batch(&mut self, high: &[f64], low: &[f64]) -> Result<Float64Array, JsError> {
        if high.len() != low.len() {
            return Err(JsError::new("high and low must be equal length"));
        }
        let mut out = Vec::with_capacity(high.len());
        for i in 0..high.len() {
            let c = make_candle(high[i], low[i], low[i], 0.0)?;
            out.push(self.inner.update(c).unwrap_or(f64::NAN));
        }
        Ok(Float64Array::from(out.as_slice()))
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
}

#[wasm_bindgen(js_name = BalanceOfPower)]
pub struct WasmBalanceOfPower {
    inner: wc::BalanceOfPower,
}

impl Default for WasmBalanceOfPower {
    fn default() -> Self {
        Self::new()
    }
}

#[wasm_bindgen(js_class = BalanceOfPower)]
impl WasmBalanceOfPower {
    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmBalanceOfPower {
        Self {
            inner: wc::BalanceOfPower::new(),
        }
    }
    pub fn update(
        &mut self,
        open: f64,
        high: f64,
        low: f64,
        close: f64,
    ) -> Result<Option<f64>, JsError> {
        let c = wc::Candle::new(open, high, low, close, 0.0, 0).map_err(map_err)?;
        Ok(self.inner.update(c))
    }
    pub fn batch(
        &mut self,
        open: &[f64],
        high: &[f64],
        low: &[f64],
        close: &[f64],
    ) -> Result<Float64Array, JsError> {
        let n = open.len();
        if high.len() != n || low.len() != n || close.len() != n {
            return Err(JsError::new("open, high, low, close must be equal length"));
        }
        let mut out = Vec::with_capacity(n);
        for i in 0..n {
            let c = wc::Candle::new(open[i], high[i], low[i], close[i], 0.0, 0).map_err(map_err)?;
            out.push(self.inner.update(c).unwrap_or(f64::NAN));
        }
        Ok(Float64Array::from(out.as_slice()))
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
}

#[wasm_bindgen(js_name = ChoppinessIndex)]
pub struct WasmChoppinessIndex {
    inner: wc::ChoppinessIndex,
}

#[wasm_bindgen(js_class = ChoppinessIndex)]
impl WasmChoppinessIndex {
    #[wasm_bindgen(constructor)]
    pub fn new(period: usize) -> Result<WasmChoppinessIndex, JsError> {
        Ok(Self {
            inner: wc::ChoppinessIndex::new(period).map_err(map_err)?,
        })
    }
    pub fn update(&mut self, high: f64, low: f64, close: f64) -> Result<Option<f64>, JsError> {
        let c = make_candle(high, low, close, 0.0)?;
        Ok(self.inner.update(c))
    }
    pub fn batch(
        &mut self,
        high: &[f64],
        low: &[f64],
        close: &[f64],
    ) -> Result<Float64Array, JsError> {
        let n = high.len();
        if low.len() != n || close.len() != n {
            return Err(JsError::new("high, low, close must be equal length"));
        }
        let mut out = Vec::with_capacity(n);
        for i in 0..n {
            let c = make_candle(high[i], low[i], close[i], 0.0)?;
            out.push(self.inner.update(c).unwrap_or(f64::NAN));
        }
        Ok(Float64Array::from(out.as_slice()))
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
}

#[wasm_bindgen(js_name = TrueRange)]
pub struct WasmTrueRange {
    inner: wc::TrueRange,
}

impl Default for WasmTrueRange {
    fn default() -> Self {
        Self::new()
    }
}

#[wasm_bindgen(js_class = TrueRange)]
impl WasmTrueRange {
    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmTrueRange {
        Self {
            inner: wc::TrueRange::new(),
        }
    }
    pub fn update(&mut self, high: f64, low: f64, close: f64) -> Result<Option<f64>, JsError> {
        let c = make_candle(high, low, close, 0.0)?;
        Ok(self.inner.update(c))
    }
    pub fn batch(
        &mut self,
        high: &[f64],
        low: &[f64],
        close: &[f64],
    ) -> Result<Float64Array, JsError> {
        let n = high.len();
        if low.len() != n || close.len() != n {
            return Err(JsError::new("high, low, close must be equal length"));
        }
        let mut out = Vec::with_capacity(n);
        for i in 0..n {
            let c = make_candle(high[i], low[i], close[i], 0.0)?;
            out.push(self.inner.update(c).unwrap_or(f64::NAN));
        }
        Ok(Float64Array::from(out.as_slice()))
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
}

#[wasm_bindgen(js_name = ChaikinVolatility)]
pub struct WasmChaikinVolatility {
    inner: wc::ChaikinVolatility,
}

#[wasm_bindgen(js_class = ChaikinVolatility)]
impl WasmChaikinVolatility {
    #[wasm_bindgen(constructor)]
    pub fn new(ema_period: usize, roc_period: usize) -> Result<WasmChaikinVolatility, JsError> {
        Ok(Self {
            inner: wc::ChaikinVolatility::new(ema_period, roc_period).map_err(map_err)?,
        })
    }
    pub fn update(&mut self, high: f64, low: f64) -> Result<Option<f64>, JsError> {
        let c = make_candle(high, low, low, 0.0)?;
        Ok(self.inner.update(c))
    }
    pub fn batch(&mut self, high: &[f64], low: &[f64]) -> Result<Float64Array, JsError> {
        if high.len() != low.len() {
            return Err(JsError::new("high and low must be equal length"));
        }
        let mut out = Vec::with_capacity(high.len());
        for i in 0..high.len() {
            let c = make_candle(high[i], low[i], low[i], 0.0)?;
            out.push(self.inner.update(c).unwrap_or(f64::NAN));
        }
        Ok(Float64Array::from(out.as_slice()))
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
}

#[wasm_bindgen(js_name = NATR)]
pub struct WasmNatr {
    inner: wc::Natr,
}

#[wasm_bindgen(js_class = NATR)]
impl WasmNatr {
    #[wasm_bindgen(constructor)]
    pub fn new(period: usize) -> Result<WasmNatr, JsError> {
        Ok(Self {
            inner: wc::Natr::new(period).map_err(map_err)?,
        })
    }
    pub fn update(&mut self, high: f64, low: f64, close: f64) -> Result<Option<f64>, JsError> {
        let c = make_candle(high, low, close, 0.0)?;
        Ok(self.inner.update(c))
    }
    pub fn batch(
        &mut self,
        high: &[f64],
        low: &[f64],
        close: &[f64],
    ) -> Result<Float64Array, JsError> {
        if high.len() != low.len() || low.len() != close.len() {
            return Err(JsError::new("high, low, close must be equal length"));
        }
        let mut out = Vec::with_capacity(high.len());
        for i in 0..high.len() {
            let c = make_candle(high[i], low[i], close[i], 0.0)?;
            out.push(self.inner.update(c).unwrap_or(f64::NAN));
        }
        Ok(Float64Array::from(out.as_slice()))
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
}

#[wasm_bindgen(js_name = AroonOscillator)]
pub struct WasmAroonOscillator {
    inner: wc::AroonOscillator,
}

#[wasm_bindgen(js_class = AroonOscillator)]
impl WasmAroonOscillator {
    #[wasm_bindgen(constructor)]
    pub fn new(period: usize) -> Result<WasmAroonOscillator, JsError> {
        Ok(Self {
            inner: wc::AroonOscillator::new(period).map_err(map_err)?,
        })
    }
    pub fn update(&mut self, high: f64, low: f64) -> Result<Option<f64>, JsError> {
        let c = make_candle(high, low, low, 0.0)?;
        Ok(self.inner.update(c))
    }
    pub fn batch(&mut self, high: &[f64], low: &[f64]) -> Result<Float64Array, JsError> {
        if high.len() != low.len() {
            return Err(JsError::new("high and low must be equal length"));
        }
        let mut out = Vec::with_capacity(high.len());
        for i in 0..high.len() {
            let c = make_candle(high[i], low[i], low[i], 0.0)?;
            out.push(self.inner.update(c).unwrap_or(f64::NAN));
        }
        Ok(Float64Array::from(out.as_slice()))
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
}

#[wasm_bindgen(js_name = Vortex)]
pub struct WasmVortex {
    inner: wc::Vortex,
}

#[wasm_bindgen(js_class = Vortex)]
impl WasmVortex {
    #[wasm_bindgen(constructor)]
    pub fn new(period: usize) -> Result<WasmVortex, JsError> {
        Ok(Self {
            inner: wc::Vortex::new(period).map_err(map_err)?,
        })
    }
    pub fn update(&mut self, high: f64, low: f64, close: f64) -> Result<JsValue, JsError> {
        let c = make_candle(high, low, close, 0.0)?;
        Ok(match self.inner.update(c) {
            Some(o) => {
                let obj = Object::new();
                Reflect::set(&obj, &"plus".into(), &o.plus.into()).ok();
                Reflect::set(&obj, &"minus".into(), &o.minus.into()).ok();
                obj.into()
            }
            None => JsValue::NULL,
        })
    }
    /// Returns `[plus0, minus0, plus1, minus1, ...]`, length `2 * n`. Warmup is NaN.
    pub fn batch(
        &mut self,
        high: &[f64],
        low: &[f64],
        close: &[f64],
    ) -> Result<Float64Array, JsError> {
        let n = high.len();
        if low.len() != n || close.len() != n {
            return Err(JsError::new("high, low, close must be equal length"));
        }
        let mut out = vec![f64::NAN; n * 2];
        for i in 0..n {
            let c = make_candle(high[i], low[i], close[i], 0.0)?;
            if let Some(o) = self.inner.update(c) {
                out[i * 2] = o.plus;
                out[i * 2 + 1] = o.minus;
            }
        }
        Ok(Float64Array::from(out.as_slice()))
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
}

#[wasm_bindgen(js_name = WaveTrend)]
pub struct WasmWaveTrend {
    inner: wc::WaveTrend,
}

#[wasm_bindgen(js_class = WaveTrend)]
impl WasmWaveTrend {
    #[wasm_bindgen(constructor)]
    pub fn new(
        channel_period: usize,
        average_period: usize,
        signal_period: usize,
    ) -> Result<WasmWaveTrend, JsError> {
        Ok(Self {
            inner: wc::WaveTrend::new(channel_period, average_period, signal_period)
                .map_err(map_err)?,
        })
    }
    pub fn classic() -> Result<WasmWaveTrend, JsError> {
        Ok(Self {
            inner: wc::WaveTrend::classic().map_err(map_err)?,
        })
    }
    pub fn update(&mut self, high: f64, low: f64, close: f64) -> Result<JsValue, JsError> {
        let c = make_candle(high, low, close, 0.0)?;
        Ok(match self.inner.update(c) {
            Some(o) => {
                let obj = Object::new();
                Reflect::set(&obj, &"wt1".into(), &o.wt1.into()).ok();
                Reflect::set(&obj, &"wt2".into(), &o.wt2.into()).ok();
                obj.into()
            }
            None => JsValue::NULL,
        })
    }
    pub fn batch(
        &mut self,
        high: &[f64],
        low: &[f64],
        close: &[f64],
    ) -> Result<Float64Array, JsError> {
        let n = high.len();
        if low.len() != n || close.len() != n {
            return Err(JsError::new("high, low, close must be equal length"));
        }
        let mut out = vec![f64::NAN; n * 2];
        for i in 0..n {
            let c = make_candle(high[i], low[i], close[i], 0.0)?;
            if let Some(o) = self.inner.update(c) {
                out[i * 2] = o.wt1;
                out[i * 2 + 1] = o.wt2;
            }
        }
        Ok(Float64Array::from(out.as_slice()))
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
}

#[wasm_bindgen(js_name = RWI)]
pub struct WasmRwi {
    inner: wc::Rwi,
}

#[wasm_bindgen(js_class = RWI)]
impl WasmRwi {
    #[wasm_bindgen(constructor)]
    pub fn new(period: usize) -> Result<WasmRwi, JsError> {
        Ok(Self {
            inner: wc::Rwi::new(period).map_err(map_err)?,
        })
    }
    pub fn update(&mut self, high: f64, low: f64, close: f64) -> Result<JsValue, JsError> {
        let c = make_candle(high, low, close, 0.0)?;
        Ok(match self.inner.update(c) {
            Some(o) => {
                let obj = Object::new();
                Reflect::set(&obj, &"high".into(), &o.high.into()).ok();
                Reflect::set(&obj, &"low".into(), &o.low.into()).ok();
                obj.into()
            }
            None => JsValue::NULL,
        })
    }
    /// Returns `[high0, low0, high1, low1, ...]`, length `2 * n`. Warmup is NaN.
    pub fn batch(
        &mut self,
        high: &[f64],
        low: &[f64],
        close: &[f64],
    ) -> Result<Float64Array, JsError> {
        let n = high.len();
        if low.len() != n || close.len() != n {
            return Err(JsError::new("high, low, close must be equal length"));
        }
        let mut out = vec![f64::NAN; n * 2];
        for i in 0..n {
            let c = make_candle(high[i], low[i], close[i], 0.0)?;
            if let Some(o) = self.inner.update(c) {
                out[i * 2] = o.high;
                out[i * 2 + 1] = o.low;
            }
        }
        Ok(Float64Array::from(out.as_slice()))
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
}

#[wasm_bindgen(js_name = MassIndex)]
pub struct WasmMassIndex {
    inner: wc::MassIndex,
}

#[wasm_bindgen(js_class = MassIndex)]
impl WasmMassIndex {
    #[wasm_bindgen(constructor)]
    pub fn new(ema_period: usize, sum_period: usize) -> Result<WasmMassIndex, JsError> {
        Ok(Self {
            inner: wc::MassIndex::new(ema_period, sum_period).map_err(map_err)?,
        })
    }
    pub fn update(&mut self, high: f64, low: f64) -> Result<Option<f64>, JsError> {
        let c = make_candle(high, low, low, 0.0)?;
        Ok(self.inner.update(c))
    }
    pub fn batch(&mut self, high: &[f64], low: &[f64]) -> Result<Float64Array, JsError> {
        if high.len() != low.len() {
            return Err(JsError::new("high and low must be equal length"));
        }
        let mut out = Vec::with_capacity(high.len());
        for i in 0..high.len() {
            let c = make_candle(high[i], low[i], low[i], 0.0)?;
            out.push(self.inner.update(c).unwrap_or(f64::NAN));
        }
        Ok(Float64Array::from(out.as_slice()))
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
}

#[wasm_bindgen(js_name = EVWMA)]
pub struct WasmEvwma {
    inner: wc::Evwma,
}

#[wasm_bindgen(js_class = EVWMA)]
impl WasmEvwma {
    #[wasm_bindgen(constructor)]
    pub fn new(period: usize) -> Result<WasmEvwma, JsError> {
        Ok(Self {
            inner: wc::Evwma::new(period).map_err(map_err)?,
        })
    }
    pub fn update(&mut self, close: f64, volume: f64) -> Result<Option<f64>, JsError> {
        let c = make_candle(close, close, close, volume)?;
        Ok(self.inner.update(c))
    }
    pub fn batch(&mut self, close: &[f64], volume: &[f64]) -> Result<Float64Array, JsError> {
        if close.len() != volume.len() {
            return Err(JsError::new("close and volume must be equal length"));
        }
        let mut out = Vec::with_capacity(close.len());
        for i in 0..close.len() {
            let c = make_candle(close[i], close[i], close[i], volume[i])?;
            out.push(self.inner.update(c).unwrap_or(f64::NAN));
        }
        Ok(Float64Array::from(out.as_slice()))
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
    #[wasm_bindgen(js_name = isReady)]
    pub fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    #[wasm_bindgen(js_name = warmupPeriod)]
    pub fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
}

#[wasm_bindgen(js_name = VWMA)]
pub struct WasmVwma {
    inner: wc::Vwma,
}

#[wasm_bindgen(js_class = VWMA)]
impl WasmVwma {
    #[wasm_bindgen(constructor)]
    pub fn new(period: usize) -> Result<WasmVwma, JsError> {
        Ok(Self {
            inner: wc::Vwma::new(period).map_err(map_err)?,
        })
    }
    pub fn update(&mut self, close: f64, volume: f64) -> Result<Option<f64>, JsError> {
        let c = make_candle(close, close, close, volume)?;
        Ok(self.inner.update(c))
    }
    pub fn batch(&mut self, close: &[f64], volume: &[f64]) -> Result<Float64Array, JsError> {
        if close.len() != volume.len() {
            return Err(JsError::new("close and volume must be equal length"));
        }
        let mut out = Vec::with_capacity(close.len());
        for i in 0..close.len() {
            let c = make_candle(close[i], close[i], close[i], volume[i])?;
            out.push(self.inner.update(c).unwrap_or(f64::NAN));
        }
        Ok(Float64Array::from(out.as_slice()))
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
}

#[wasm_bindgen(js_name = ADX)]
pub struct WasmAdx {
    inner: wc::Adx,
}

#[wasm_bindgen(js_class = ADX)]
impl WasmAdx {
    #[wasm_bindgen(constructor)]
    pub fn new(period: usize) -> Result<WasmAdx, JsError> {
        Ok(Self {
            inner: wc::Adx::new(period).map_err(map_err)?,
        })
    }
    /// Returns `[plusDi, minusDi, adx]` × n, length `3n`.
    pub fn batch(
        &mut self,
        high: &[f64],
        low: &[f64],
        close: &[f64],
    ) -> Result<Float64Array, JsError> {
        if high.len() != low.len() || low.len() != close.len() {
            return Err(JsError::new("high, low, close must be equal length"));
        }
        let n = high.len();
        let mut out = vec![f64::NAN; n * 3];
        for i in 0..n {
            let c = make_candle(high[i], low[i], close[i], 0.0)?;
            if let Some(o) = self.inner.update(c) {
                out[i * 3] = o.plus_di;
                out[i * 3 + 1] = o.minus_di;
                out[i * 3 + 2] = o.adx;
            }
        }
        Ok(Float64Array::from(out.as_slice()))
    }
    /// Streaming update. Returns `{ plusDi, minusDi, adx }` once warm, else `null`.
    pub fn update(&mut self, high: f64, low: f64, close: f64) -> Result<JsValue, JsError> {
        let c = make_candle(high, low, close, 0.0)?;
        Ok(match self.inner.update(c) {
            Some(o) => {
                let obj = Object::new();
                Reflect::set(&obj, &"plusDi".into(), &o.plus_di.into()).ok();
                Reflect::set(&obj, &"minusDi".into(), &o.minus_di.into()).ok();
                Reflect::set(&obj, &"adx".into(), &o.adx.into()).ok();
                obj.into()
            }
            None => JsValue::NULL,
        })
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
    #[wasm_bindgen(js_name = isReady)]
    pub fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    #[wasm_bindgen(js_name = warmupPeriod)]
    pub fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
}

#[wasm_bindgen(js_name = ADXR)]
pub struct WasmAdxr {
    inner: wc::Adxr,
}

#[wasm_bindgen(js_class = ADXR)]
impl WasmAdxr {
    #[wasm_bindgen(constructor)]
    pub fn new(period: usize) -> Result<WasmAdxr, JsError> {
        Ok(Self {
            inner: wc::Adxr::new(period).map_err(map_err)?,
        })
    }
    pub fn update(&mut self, high: f64, low: f64, close: f64) -> Result<Option<f64>, JsError> {
        let c = make_candle(high, low, close, 0.0)?;
        Ok(self.inner.update(c))
    }
    pub fn batch(
        &mut self,
        high: &[f64],
        low: &[f64],
        close: &[f64],
    ) -> Result<Float64Array, JsError> {
        if high.len() != low.len() || low.len() != close.len() {
            return Err(JsError::new("high, low, close must be equal length"));
        }
        let mut out = Vec::with_capacity(high.len());
        for i in 0..high.len() {
            let c = make_candle(high[i], low[i], close[i], 0.0)?;
            out.push(self.inner.update(c).unwrap_or(f64::NAN));
        }
        Ok(Float64Array::from(out.as_slice()))
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
    #[wasm_bindgen(js_name = isReady)]
    pub fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    #[wasm_bindgen(js_name = warmupPeriod)]
    pub fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
}

#[wasm_bindgen(js_name = WilliamsR)]
pub struct WasmWilliamsR {
    inner: wc::WilliamsR,
}

#[wasm_bindgen(js_class = WilliamsR)]
impl WasmWilliamsR {
    #[wasm_bindgen(constructor)]
    pub fn new(period: usize) -> Result<WasmWilliamsR, JsError> {
        Ok(Self {
            inner: wc::WilliamsR::new(period).map_err(map_err)?,
        })
    }
    pub fn batch(
        &mut self,
        high: &[f64],
        low: &[f64],
        close: &[f64],
    ) -> Result<Float64Array, JsError> {
        if high.len() != low.len() || low.len() != close.len() {
            return Err(JsError::new("high, low, close must be equal length"));
        }
        let mut out = Vec::with_capacity(high.len());
        for i in 0..high.len() {
            let c = make_candle(high[i], low[i], close[i], 0.0)?;
            out.push(self.inner.update(c).unwrap_or(f64::NAN));
        }
        Ok(Float64Array::from(out.as_slice()))
    }
    pub fn update(&mut self, high: f64, low: f64, close: f64) -> Result<Option<f64>, JsError> {
        let c = make_candle(high, low, close, 0.0)?;
        Ok(self.inner.update(c))
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
    #[wasm_bindgen(js_name = isReady)]
    pub fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    #[wasm_bindgen(js_name = warmupPeriod)]
    pub fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
}

#[wasm_bindgen(js_name = CCI)]
pub struct WasmCci {
    inner: wc::Cci,
}

#[wasm_bindgen(js_class = CCI)]
impl WasmCci {
    #[wasm_bindgen(constructor)]
    pub fn new(period: usize) -> Result<WasmCci, JsError> {
        Ok(Self {
            inner: wc::Cci::new(period).map_err(map_err)?,
        })
    }
    pub fn batch(
        &mut self,
        high: &[f64],
        low: &[f64],
        close: &[f64],
    ) -> Result<Float64Array, JsError> {
        if high.len() != low.len() || low.len() != close.len() {
            return Err(JsError::new("high, low, close must be equal length"));
        }
        let mut out = Vec::with_capacity(high.len());
        for i in 0..high.len() {
            let c = make_candle(high[i], low[i], close[i], 0.0)?;
            out.push(self.inner.update(c).unwrap_or(f64::NAN));
        }
        Ok(Float64Array::from(out.as_slice()))
    }
    pub fn update(&mut self, high: f64, low: f64, close: f64) -> Result<Option<f64>, JsError> {
        let c = make_candle(high, low, close, 0.0)?;
        Ok(self.inner.update(c))
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
    #[wasm_bindgen(js_name = isReady)]
    pub fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    #[wasm_bindgen(js_name = warmupPeriod)]
    pub fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
}

#[wasm_bindgen(js_name = MFI)]
pub struct WasmMfi {
    inner: wc::Mfi,
}

#[wasm_bindgen(js_class = MFI)]
impl WasmMfi {
    #[wasm_bindgen(constructor)]
    pub fn new(period: usize) -> Result<WasmMfi, JsError> {
        Ok(Self {
            inner: wc::Mfi::new(period).map_err(map_err)?,
        })
    }
    pub fn batch(
        &mut self,
        high: &[f64],
        low: &[f64],
        close: &[f64],
        volume: &[f64],
    ) -> Result<Float64Array, JsError> {
        if high.len() != low.len() || low.len() != close.len() || close.len() != volume.len() {
            return Err(JsError::new(
                "high, low, close, volume must be equal length",
            ));
        }
        let mut out = Vec::with_capacity(high.len());
        for i in 0..high.len() {
            let c = make_candle(high[i], low[i], close[i], volume[i])?;
            out.push(self.inner.update(c).unwrap_or(f64::NAN));
        }
        Ok(Float64Array::from(out.as_slice()))
    }
    pub fn update(
        &mut self,
        high: f64,
        low: f64,
        close: f64,
        volume: f64,
    ) -> Result<Option<f64>, JsError> {
        let c = make_candle(high, low, close, volume)?;
        Ok(self.inner.update(c))
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
    #[wasm_bindgen(js_name = isReady)]
    pub fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    #[wasm_bindgen(js_name = warmupPeriod)]
    pub fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
}

#[wasm_bindgen(js_name = PSAR)]
pub struct WasmPsar {
    inner: wc::Psar,
}

#[wasm_bindgen(js_class = PSAR)]
impl WasmPsar {
    #[wasm_bindgen(constructor)]
    pub fn new(af_start: f64, af_step: f64, af_max: f64) -> Result<WasmPsar, JsError> {
        Ok(Self {
            inner: wc::Psar::new(af_start, af_step, af_max).map_err(map_err)?,
        })
    }
    pub fn batch(
        &mut self,
        high: &[f64],
        low: &[f64],
        close: &[f64],
    ) -> Result<Float64Array, JsError> {
        if high.len() != low.len() || low.len() != close.len() {
            return Err(JsError::new("high, low, close must be equal length"));
        }
        let mut out = Vec::with_capacity(high.len());
        for i in 0..high.len() {
            let c = make_candle(high[i], low[i], close[i], 0.0)?;
            out.push(self.inner.update(c).unwrap_or(f64::NAN));
        }
        Ok(Float64Array::from(out.as_slice()))
    }
    pub fn update(&mut self, high: f64, low: f64, close: f64) -> Result<Option<f64>, JsError> {
        let c = make_candle(high, low, close, 0.0)?;
        Ok(self.inner.update(c))
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
    #[wasm_bindgen(js_name = isReady)]
    pub fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    #[wasm_bindgen(js_name = warmupPeriod)]
    pub fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
}

#[wasm_bindgen(js_name = Keltner)]
pub struct WasmKeltner {
    inner: wc::Keltner,
}

#[wasm_bindgen(js_class = Keltner)]
impl WasmKeltner {
    #[wasm_bindgen(constructor)]
    pub fn new(
        ema_period: usize,
        atr_period: usize,
        multiplier: f64,
    ) -> Result<WasmKeltner, JsError> {
        Ok(Self {
            inner: wc::Keltner::new(ema_period, atr_period, multiplier).map_err(map_err)?,
        })
    }
    pub fn batch(
        &mut self,
        high: &[f64],
        low: &[f64],
        close: &[f64],
    ) -> Result<Float64Array, JsError> {
        if high.len() != low.len() || low.len() != close.len() {
            return Err(JsError::new("high, low, close must be equal length"));
        }
        let n = high.len();
        let mut out = vec![f64::NAN; n * 3];
        for i in 0..n {
            let c = make_candle(high[i], low[i], close[i], 0.0)?;
            if let Some(o) = self.inner.update(c) {
                out[i * 3] = o.upper;
                out[i * 3 + 1] = o.middle;
                out[i * 3 + 2] = o.lower;
            }
        }
        Ok(Float64Array::from(out.as_slice()))
    }
    /// Streaming update. Returns `{ upper, middle, lower }` once warm, else `null`.
    pub fn update(&mut self, high: f64, low: f64, close: f64) -> Result<JsValue, JsError> {
        let c = make_candle(high, low, close, 0.0)?;
        Ok(match self.inner.update(c) {
            Some(o) => {
                let obj = Object::new();
                Reflect::set(&obj, &"upper".into(), &o.upper.into()).ok();
                Reflect::set(&obj, &"middle".into(), &o.middle.into()).ok();
                Reflect::set(&obj, &"lower".into(), &o.lower.into()).ok();
                obj.into()
            }
            None => JsValue::NULL,
        })
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
    #[wasm_bindgen(js_name = isReady)]
    pub fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    #[wasm_bindgen(js_name = warmupPeriod)]
    pub fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
}

#[wasm_bindgen(js_name = Donchian)]
pub struct WasmDonchian {
    inner: wc::Donchian,
}

#[wasm_bindgen(js_class = Donchian)]
impl WasmDonchian {
    #[wasm_bindgen(constructor)]
    pub fn new(period: usize) -> Result<WasmDonchian, JsError> {
        Ok(Self {
            inner: wc::Donchian::new(period).map_err(map_err)?,
        })
    }
    pub fn batch(&mut self, high: &[f64], low: &[f64]) -> Result<Float64Array, JsError> {
        if high.len() != low.len() {
            return Err(JsError::new("high and low must be equal length"));
        }
        let n = high.len();
        let mut out = vec![f64::NAN; n * 3];
        for i in 0..n {
            let c = make_candle(high[i], low[i], low[i], 0.0)?;
            if let Some(o) = self.inner.update(c) {
                out[i * 3] = o.upper;
                out[i * 3 + 1] = o.middle;
                out[i * 3 + 2] = o.lower;
            }
        }
        Ok(Float64Array::from(out.as_slice()))
    }
    /// Streaming update. Returns `{ upper, middle, lower }` once warm, else `null`.
    pub fn update(&mut self, high: f64, low: f64) -> Result<JsValue, JsError> {
        let c = make_candle(high, low, low, 0.0)?;
        Ok(match self.inner.update(c) {
            Some(o) => {
                let obj = Object::new();
                Reflect::set(&obj, &"upper".into(), &o.upper.into()).ok();
                Reflect::set(&obj, &"middle".into(), &o.middle.into()).ok();
                Reflect::set(&obj, &"lower".into(), &o.lower.into()).ok();
                obj.into()
            }
            None => JsValue::NULL,
        })
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
    #[wasm_bindgen(js_name = isReady)]
    pub fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    #[wasm_bindgen(js_name = warmupPeriod)]
    pub fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
}

#[wasm_bindgen(js_name = VWAP)]
pub struct WasmVwap {
    inner: wc::Vwap,
}

impl Default for WasmVwap {
    fn default() -> Self {
        Self::new()
    }
}

#[wasm_bindgen(js_class = VWAP)]
impl WasmVwap {
    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmVwap {
        Self {
            inner: wc::Vwap::new(),
        }
    }
    pub fn batch(
        &mut self,
        high: &[f64],
        low: &[f64],
        close: &[f64],
        volume: &[f64],
    ) -> Result<Float64Array, JsError> {
        if high.len() != low.len() || low.len() != close.len() || close.len() != volume.len() {
            return Err(JsError::new(
                "high, low, close, volume must be equal length",
            ));
        }
        let mut out = Vec::with_capacity(high.len());
        for i in 0..high.len() {
            let c = make_candle(high[i], low[i], close[i], volume[i])?;
            out.push(self.inner.update(c).unwrap_or(f64::NAN));
        }
        Ok(Float64Array::from(out.as_slice()))
    }
    pub fn update(
        &mut self,
        high: f64,
        low: f64,
        close: f64,
        volume: f64,
    ) -> Result<Option<f64>, JsError> {
        let c = make_candle(high, low, close, volume)?;
        Ok(self.inner.update(c))
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
    #[wasm_bindgen(js_name = isReady)]
    pub fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    #[wasm_bindgen(js_name = warmupPeriod)]
    pub fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
}

#[wasm_bindgen(js_name = RollingVWAP)]
pub struct WasmRollingVwap {
    inner: wc::RollingVwap,
}

#[wasm_bindgen(js_class = RollingVWAP)]
impl WasmRollingVwap {
    #[wasm_bindgen(constructor)]
    pub fn new(period: usize) -> Result<WasmRollingVwap, JsError> {
        Ok(Self {
            inner: wc::RollingVwap::new(period).map_err(map_err)?,
        })
    }
    #[wasm_bindgen(getter)]
    pub fn period(&self) -> usize {
        self.inner.period()
    }
    pub fn update(
        &mut self,
        high: f64,
        low: f64,
        close: f64,
        volume: f64,
    ) -> Result<Option<f64>, JsError> {
        let c = make_candle(high, low, close, volume)?;
        Ok(self.inner.update(c))
    }
    pub fn batch(
        &mut self,
        high: &[f64],
        low: &[f64],
        close: &[f64],
        volume: &[f64],
    ) -> Result<Float64Array, JsError> {
        if high.len() != low.len() || low.len() != close.len() || close.len() != volume.len() {
            return Err(JsError::new(
                "high, low, close, volume must be equal length",
            ));
        }
        let mut out = Vec::with_capacity(high.len());
        for i in 0..high.len() {
            let c = make_candle(high[i], low[i], close[i], volume[i])?;
            out.push(self.inner.update(c).unwrap_or(f64::NAN));
        }
        Ok(Float64Array::from(out.as_slice()))
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
    #[wasm_bindgen(js_name = isReady)]
    pub fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    #[wasm_bindgen(js_name = warmupPeriod)]
    pub fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
}

#[wasm_bindgen(js_name = AwesomeOscillatorHistogram)]
pub struct WasmAoHist {
    inner: wc::AwesomeOscillatorHistogram,
}

#[wasm_bindgen(js_class = AwesomeOscillatorHistogram)]
impl WasmAoHist {
    #[wasm_bindgen(constructor)]
    pub fn new(fast: usize, slow: usize, sma_period: usize) -> Result<WasmAoHist, JsError> {
        Ok(Self {
            inner: wc::AwesomeOscillatorHistogram::new(fast, slow, sma_period).map_err(map_err)?,
        })
    }
    pub fn update(&mut self, high: f64, low: f64) -> Result<Option<f64>, JsError> {
        let c = make_candle(high, low, low, 0.0)?;
        Ok(self.inner.update(c))
    }
    pub fn batch(&mut self, high: &[f64], low: &[f64]) -> Result<Float64Array, JsError> {
        if high.len() != low.len() {
            return Err(JsError::new("high and low must be equal length"));
        }
        let mut out = Vec::with_capacity(high.len());
        for i in 0..high.len() {
            let c = make_candle(high[i], low[i], low[i], 0.0)?;
            out.push(self.inner.update(c).unwrap_or(f64::NAN));
        }
        Ok(Float64Array::from(out.as_slice()))
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
    #[wasm_bindgen(js_name = isReady)]
    pub fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    #[wasm_bindgen(js_name = warmupPeriod)]
    pub fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
}

#[wasm_bindgen(js_name = AwesomeOscillator)]
pub struct WasmAo {
    inner: wc::AwesomeOscillator,
}

#[wasm_bindgen(js_class = AwesomeOscillator)]
impl WasmAo {
    #[wasm_bindgen(constructor)]
    pub fn new(fast: usize, slow: usize) -> Result<WasmAo, JsError> {
        Ok(Self {
            inner: wc::AwesomeOscillator::new(fast, slow).map_err(map_err)?,
        })
    }
    pub fn batch(&mut self, high: &[f64], low: &[f64]) -> Result<Float64Array, JsError> {
        if high.len() != low.len() {
            return Err(JsError::new("high and low must be equal length"));
        }
        let mut out = Vec::with_capacity(high.len());
        for i in 0..high.len() {
            let c = make_candle(high[i], low[i], low[i], 0.0)?;
            out.push(self.inner.update(c).unwrap_or(f64::NAN));
        }
        Ok(Float64Array::from(out.as_slice()))
    }
    pub fn update(&mut self, high: f64, low: f64) -> Result<Option<f64>, JsError> {
        let c = make_candle(high, low, low, 0.0)?;
        Ok(self.inner.update(c))
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
    #[wasm_bindgen(js_name = isReady)]
    pub fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    #[wasm_bindgen(js_name = warmupPeriod)]
    pub fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
}

#[wasm_bindgen(js_name = Alligator)]
pub struct WasmAlligator {
    inner: wc::Alligator,
}

#[wasm_bindgen(js_class = Alligator)]
impl WasmAlligator {
    #[wasm_bindgen(constructor)]
    pub fn new(jaw: usize, teeth: usize, lips: usize) -> Result<WasmAlligator, JsError> {
        Ok(Self {
            inner: wc::Alligator::new(jaw, teeth, lips).map_err(map_err)?,
        })
    }
    /// Returns `[jaw0, teeth0, lips0, jaw1, teeth1, lips1, ...]`, length `3n`.
    pub fn batch(&mut self, high: &[f64], low: &[f64]) -> Result<Float64Array, JsError> {
        if high.len() != low.len() {
            return Err(JsError::new("high and low must be equal length"));
        }
        let n = high.len();
        let mut out = vec![f64::NAN; n * 3];
        for i in 0..n {
            let c = make_candle(high[i], low[i], low[i], 0.0)?;
            if let Some(o) = self.inner.update(c) {
                out[i * 3] = o.jaw;
                out[i * 3 + 1] = o.teeth;
                out[i * 3 + 2] = o.lips;
            }
        }
        Ok(Float64Array::from(out.as_slice()))
    }
    /// Streaming update. Returns `{ jaw, teeth, lips }` once warm, else `null`.
    pub fn update(&mut self, high: f64, low: f64) -> Result<JsValue, JsError> {
        let c = make_candle(high, low, low, 0.0)?;
        Ok(match self.inner.update(c) {
            Some(o) => {
                let obj = Object::new();
                Reflect::set(&obj, &"jaw".into(), &o.jaw.into()).ok();
                Reflect::set(&obj, &"teeth".into(), &o.teeth.into()).ok();
                Reflect::set(&obj, &"lips".into(), &o.lips.into()).ok();
                obj.into()
            }
            None => JsValue::NULL,
        })
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
    #[wasm_bindgen(js_name = isReady)]
    pub fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    #[wasm_bindgen(js_name = warmupPeriod)]
    pub fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
}

#[wasm_bindgen(js_name = Aroon)]
pub struct WasmAroon {
    inner: wc::Aroon,
}

#[wasm_bindgen(js_class = Aroon)]
impl WasmAroon {
    #[wasm_bindgen(constructor)]
    pub fn new(period: usize) -> Result<WasmAroon, JsError> {
        Ok(Self {
            inner: wc::Aroon::new(period).map_err(map_err)?,
        })
    }
    /// Returns `[up0, down0, up1, down1, ...]`, length `2n`.
    pub fn batch(&mut self, high: &[f64], low: &[f64]) -> Result<Float64Array, JsError> {
        if high.len() != low.len() {
            return Err(JsError::new("high and low must be equal length"));
        }
        let n = high.len();
        let mut out = vec![f64::NAN; n * 2];
        for i in 0..n {
            let c = make_candle(high[i], low[i], low[i], 0.0)?;
            if let Some(o) = self.inner.update(c) {
                out[i * 2] = o.up;
                out[i * 2 + 1] = o.down;
            }
        }
        Ok(Float64Array::from(out.as_slice()))
    }
    /// Streaming update. Returns `{ up, down }` once warm, else `null`.
    pub fn update(&mut self, high: f64, low: f64) -> Result<JsValue, JsError> {
        let c = make_candle(high, low, low, 0.0)?;
        Ok(match self.inner.update(c) {
            Some(o) => {
                let obj = Object::new();
                Reflect::set(&obj, &"up".into(), &o.up.into()).ok();
                Reflect::set(&obj, &"down".into(), &o.down.into()).ok();
                obj.into()
            }
            None => JsValue::NULL,
        })
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
    #[wasm_bindgen(js_name = isReady)]
    pub fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    #[wasm_bindgen(js_name = warmupPeriod)]
    pub fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
}

// ============================== Family 10: parameterless / multi-output ==============================

#[wasm_bindgen(js_name = HilbertDominantCycle)]
pub struct WasmHilbertDominantCycle {
    inner: wc::HilbertDominantCycle,
}

#[wasm_bindgen(js_class = HilbertDominantCycle)]
impl WasmHilbertDominantCycle {
    #[wasm_bindgen(constructor)]
    #[allow(clippy::new_without_default)]
    pub fn new() -> WasmHilbertDominantCycle {
        Self {
            inner: wc::HilbertDominantCycle::new(),
        }
    }
    pub fn update(&mut self, value: f64) -> Option<f64> {
        self.inner.update(value)
    }
    pub fn batch(&mut self, prices: &[f64]) -> Float64Array {
        Float64Array::from(flatten(self.inner.batch(prices)).as_slice())
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
    #[wasm_bindgen(js_name = isReady)]
    pub fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    #[wasm_bindgen(js_name = warmupPeriod)]
    pub fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
}

#[wasm_bindgen(js_name = AdaptiveCycle)]
pub struct WasmAdaptiveCycle {
    inner: wc::AdaptiveCycle,
}

#[wasm_bindgen(js_class = AdaptiveCycle)]
impl WasmAdaptiveCycle {
    #[wasm_bindgen(constructor)]
    #[allow(clippy::new_without_default)]
    pub fn new() -> WasmAdaptiveCycle {
        Self {
            inner: wc::AdaptiveCycle::new(),
        }
    }
    pub fn update(&mut self, value: f64) -> Option<f64> {
        self.inner.update(value)
    }
    pub fn batch(&mut self, prices: &[f64]) -> Float64Array {
        Float64Array::from(flatten(self.inner.batch(prices)).as_slice())
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
    #[wasm_bindgen(js_name = isReady)]
    pub fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    #[wasm_bindgen(js_name = warmupPeriod)]
    pub fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
}

#[wasm_bindgen(js_name = SineWave)]
pub struct WasmSineWave {
    inner: wc::SineWave,
}

#[wasm_bindgen(js_class = SineWave)]
impl WasmSineWave {
    #[wasm_bindgen(constructor)]
    #[allow(clippy::new_without_default)]
    pub fn new() -> WasmSineWave {
        Self {
            inner: wc::SineWave::new(),
        }
    }
    pub fn update(&mut self, value: f64) -> Option<f64> {
        self.inner.update(value)
    }
    pub fn batch(&mut self, prices: &[f64]) -> Float64Array {
        Float64Array::from(flatten(self.inner.batch(prices)).as_slice())
    }
    pub fn lead(&self) -> f64 {
        self.inner.lead()
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
    #[wasm_bindgen(js_name = isReady)]
    pub fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    #[wasm_bindgen(js_name = warmupPeriod)]
    pub fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
}

#[wasm_bindgen(js_name = MAMA)]
pub struct WasmMama {
    inner: wc::Mama,
}

#[wasm_bindgen(js_class = MAMA)]
impl WasmMama {
    #[wasm_bindgen(constructor)]
    pub fn new(fast_limit: f64, slow_limit: f64) -> Result<WasmMama, JsError> {
        Ok(Self {
            inner: wc::Mama::new(fast_limit, slow_limit).map_err(map_err)?,
        })
    }
    pub fn update(&mut self, value: f64) -> JsValue {
        match self.inner.update(value) {
            Some(o) => {
                let obj = Object::new();
                Reflect::set(&obj, &"mama".into(), &o.mama.into()).ok();
                Reflect::set(&obj, &"fama".into(), &o.fama.into()).ok();
                obj.into()
            }
            None => JsValue::NULL,
        }
    }
    /// Returns a flat `Float64Array` of length `2 * n`: `[mama0, fama0, mama1, fama1, ...]`.
    pub fn batch(&mut self, prices: &[f64]) -> Float64Array {
        let n = prices.len();
        let mut out = vec![f64::NAN; n * 2];
        for (i, p) in prices.iter().enumerate() {
            if let Some(o) = self.inner.update(*p) {
                out[i * 2] = o.mama;
                out[i * 2 + 1] = o.fama;
            }
        }
        Float64Array::from(out.as_slice())
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
    #[wasm_bindgen(js_name = isReady)]
    pub fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    #[wasm_bindgen(js_name = warmupPeriod)]
    pub fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
}

// ============================== Family 05: Bands & Channels ==============================

// Every indicator below is multi-output (2-5 bands), so they bypass the
// `wasm_scalar_indicator!` macro and follow the hand-rolled pattern used by
// Bollinger Bands, Keltner Channels, etc. above: `update` returns a JS object
// via `Object::new` + `Reflect::set`; `batch` returns a flat interleaved
// `Float64Array`.

// ---------- MA Envelope (scalar input, 3 outputs) ----------

#[wasm_bindgen(js_name = MaEnvelope)]
pub struct WasmMaEnvelope {
    inner: wc::MaEnvelope,
}

#[wasm_bindgen(js_class = MaEnvelope)]
impl WasmMaEnvelope {
    #[wasm_bindgen(constructor)]
    pub fn new(period: usize, percent: f64) -> Result<WasmMaEnvelope, JsError> {
        Ok(Self {
            inner: wc::MaEnvelope::new(period, percent).map_err(map_err)?,
        })
    }
    pub fn update(&mut self, value: f64) -> JsValue {
        match self.inner.update(value) {
            Some(o) => {
                let obj = Object::new();
                Reflect::set(&obj, &"upper".into(), &o.upper.into()).ok();
                Reflect::set(&obj, &"middle".into(), &o.middle.into()).ok();
                Reflect::set(&obj, &"lower".into(), &o.lower.into()).ok();
                obj.into()
            }
            None => JsValue::NULL,
        }
    }
    /// Flat `[upper0, middle0, lower0, upper1, ...]`, length `3 * n`.
    pub fn batch(&mut self, prices: &[f64]) -> Float64Array {
        let n = prices.len();
        let mut out = vec![f64::NAN; n * 3];
        for (i, p) in prices.iter().enumerate() {
            if let Some(o) = self.inner.update(*p) {
                out[i * 3] = o.upper;
                out[i * 3 + 1] = o.middle;
                out[i * 3 + 2] = o.lower;
            }
        }
        Float64Array::from(out.as_slice())
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
    #[wasm_bindgen(js_name = isReady)]
    pub fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    #[wasm_bindgen(js_name = warmupPeriod)]
    pub fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
}

// ---------- Acceleration Bands ----------

#[wasm_bindgen(js_name = AccelerationBands)]
pub struct WasmAccelerationBands {
    inner: wc::AccelerationBands,
}

#[wasm_bindgen(js_class = AccelerationBands)]
impl WasmAccelerationBands {
    #[wasm_bindgen(constructor)]
    pub fn new(period: usize, factor: f64) -> Result<WasmAccelerationBands, JsError> {
        Ok(Self {
            inner: wc::AccelerationBands::new(period, factor).map_err(map_err)?,
        })
    }
    pub fn update(&mut self, high: f64, low: f64, close: f64) -> Result<JsValue, JsError> {
        let c = make_candle(high, low, close, 0.0)?;
        Ok(match self.inner.update(c) {
            Some(o) => {
                let obj = Object::new();
                Reflect::set(&obj, &"upper".into(), &o.upper.into()).ok();
                Reflect::set(&obj, &"middle".into(), &o.middle.into()).ok();
                Reflect::set(&obj, &"lower".into(), &o.lower.into()).ok();
                obj.into()
            }
            None => JsValue::NULL,
        })
    }
    /// Returns `[u0, m0, l0, u1, m1, l1, ...]`, length `3 * n`.
    pub fn batch(
        &mut self,
        high: &[f64],
        low: &[f64],
        close: &[f64],
    ) -> Result<Float64Array, JsError> {
        let n = high.len();
        if low.len() != n || close.len() != n {
            return Err(JsError::new("high, low, close must be equal length"));
        }
        let mut out = vec![f64::NAN; n * 3];
        for i in 0..n {
            let c = make_candle(high[i], low[i], close[i], 0.0)?;
            if let Some(o) = self.inner.update(c) {
                out[i * 3] = o.upper;
                out[i * 3 + 1] = o.middle;
                out[i * 3 + 2] = o.lower;
            }
        }
        Ok(Float64Array::from(out.as_slice()))
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
    #[wasm_bindgen(js_name = isReady)]
    pub fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    #[wasm_bindgen(js_name = warmupPeriod)]
    pub fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
}

// ---------- STARC Bands ----------

#[wasm_bindgen(js_name = StarcBands)]
pub struct WasmStarcBands {
    inner: wc::StarcBands,
}

#[wasm_bindgen(js_class = StarcBands)]
impl WasmStarcBands {
    #[wasm_bindgen(constructor)]
    pub fn new(
        sma_period: usize,
        atr_period: usize,
        multiplier: f64,
    ) -> Result<WasmStarcBands, JsError> {
        Ok(Self {
            inner: wc::StarcBands::new(sma_period, atr_period, multiplier).map_err(map_err)?,
        })
    }
    pub fn update(&mut self, high: f64, low: f64, close: f64) -> Result<JsValue, JsError> {
        let c = make_candle(high, low, close, 0.0)?;
        Ok(match self.inner.update(c) {
            Some(o) => {
                let obj = Object::new();
                Reflect::set(&obj, &"upper".into(), &o.upper.into()).ok();
                Reflect::set(&obj, &"middle".into(), &o.middle.into()).ok();
                Reflect::set(&obj, &"lower".into(), &o.lower.into()).ok();
                obj.into()
            }
            None => JsValue::NULL,
        })
    }
    pub fn batch(
        &mut self,
        high: &[f64],
        low: &[f64],
        close: &[f64],
    ) -> Result<Float64Array, JsError> {
        let n = high.len();
        if low.len() != n || close.len() != n {
            return Err(JsError::new("high, low, close must be equal length"));
        }
        let mut out = vec![f64::NAN; n * 3];
        for i in 0..n {
            let c = make_candle(high[i], low[i], close[i], 0.0)?;
            if let Some(o) = self.inner.update(c) {
                out[i * 3] = o.upper;
                out[i * 3 + 1] = o.middle;
                out[i * 3 + 2] = o.lower;
            }
        }
        Ok(Float64Array::from(out.as_slice()))
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
    #[wasm_bindgen(js_name = isReady)]
    pub fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    #[wasm_bindgen(js_name = warmupPeriod)]
    pub fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
}

// ---------- ATR Bands ----------

#[wasm_bindgen(js_name = AtrBands)]
pub struct WasmAtrBands {
    inner: wc::AtrBands,
}

#[wasm_bindgen(js_class = AtrBands)]
impl WasmAtrBands {
    #[wasm_bindgen(constructor)]
    pub fn new(period: usize, multiplier: f64) -> Result<WasmAtrBands, JsError> {
        Ok(Self {
            inner: wc::AtrBands::new(period, multiplier).map_err(map_err)?,
        })
    }
    pub fn update(&mut self, high: f64, low: f64, close: f64) -> Result<JsValue, JsError> {
        let c = make_candle(high, low, close, 0.0)?;
        Ok(match self.inner.update(c) {
            Some(o) => {
                let obj = Object::new();
                Reflect::set(&obj, &"upper".into(), &o.upper.into()).ok();
                Reflect::set(&obj, &"middle".into(), &o.middle.into()).ok();
                Reflect::set(&obj, &"lower".into(), &o.lower.into()).ok();
                obj.into()
            }
            None => JsValue::NULL,
        })
    }
    pub fn batch(
        &mut self,
        high: &[f64],
        low: &[f64],
        close: &[f64],
    ) -> Result<Float64Array, JsError> {
        let n = high.len();
        if low.len() != n || close.len() != n {
            return Err(JsError::new("high, low, close must be equal length"));
        }
        let mut out = vec![f64::NAN; n * 3];
        for i in 0..n {
            let c = make_candle(high[i], low[i], close[i], 0.0)?;
            if let Some(o) = self.inner.update(c) {
                out[i * 3] = o.upper;
                out[i * 3 + 1] = o.middle;
                out[i * 3 + 2] = o.lower;
            }
        }
        Ok(Float64Array::from(out.as_slice()))
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
    #[wasm_bindgen(js_name = isReady)]
    pub fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    #[wasm_bindgen(js_name = warmupPeriod)]
    pub fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
}

// ---------- Hurst Channel ----------

#[wasm_bindgen(js_name = HurstChannel)]
pub struct WasmHurstChannel {
    inner: wc::HurstChannel,
}

#[wasm_bindgen(js_class = HurstChannel)]
impl WasmHurstChannel {
    #[wasm_bindgen(constructor)]
    pub fn new(period: usize, multiplier: f64) -> Result<WasmHurstChannel, JsError> {
        Ok(Self {
            inner: wc::HurstChannel::new(period, multiplier).map_err(map_err)?,
        })
    }
    pub fn update(&mut self, high: f64, low: f64, close: f64) -> Result<JsValue, JsError> {
        let c = make_candle(high, low, close, 0.0)?;
        Ok(match self.inner.update(c) {
            Some(o) => {
                let obj = Object::new();
                Reflect::set(&obj, &"upper".into(), &o.upper.into()).ok();
                Reflect::set(&obj, &"middle".into(), &o.middle.into()).ok();
                Reflect::set(&obj, &"lower".into(), &o.lower.into()).ok();
                obj.into()
            }
            None => JsValue::NULL,
        })
    }
    pub fn batch(
        &mut self,
        high: &[f64],
        low: &[f64],
        close: &[f64],
    ) -> Result<Float64Array, JsError> {
        let n = high.len();
        if low.len() != n || close.len() != n {
            return Err(JsError::new("high, low, close must be equal length"));
        }
        let mut out = vec![f64::NAN; n * 3];
        for i in 0..n {
            let c = make_candle(high[i], low[i], close[i], 0.0)?;
            if let Some(o) = self.inner.update(c) {
                out[i * 3] = o.upper;
                out[i * 3 + 1] = o.middle;
                out[i * 3 + 2] = o.lower;
            }
        }
        Ok(Float64Array::from(out.as_slice()))
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
    #[wasm_bindgen(js_name = isReady)]
    pub fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    #[wasm_bindgen(js_name = warmupPeriod)]
    pub fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
}

// ---------- LinReg Channel (scalar input) ----------

#[wasm_bindgen(js_name = LinRegChannel)]
pub struct WasmLinRegChannel {
    inner: wc::LinRegChannel,
}

#[wasm_bindgen(js_class = LinRegChannel)]
impl WasmLinRegChannel {
    #[wasm_bindgen(constructor)]
    pub fn new(period: usize, multiplier: f64) -> Result<WasmLinRegChannel, JsError> {
        Ok(Self {
            inner: wc::LinRegChannel::new(period, multiplier).map_err(map_err)?,
        })
    }
    pub fn update(&mut self, value: f64) -> JsValue {
        match self.inner.update(value) {
            Some(o) => {
                let obj = Object::new();
                Reflect::set(&obj, &"upper".into(), &o.upper.into()).ok();
                Reflect::set(&obj, &"middle".into(), &o.middle.into()).ok();
                Reflect::set(&obj, &"lower".into(), &o.lower.into()).ok();
                obj.into()
            }
            None => JsValue::NULL,
        }
    }
    pub fn batch(&mut self, prices: &[f64]) -> Float64Array {
        let n = prices.len();
        let mut out = vec![f64::NAN; n * 3];
        for (i, p) in prices.iter().enumerate() {
            if let Some(o) = self.inner.update(*p) {
                out[i * 3] = o.upper;
                out[i * 3 + 1] = o.middle;
                out[i * 3 + 2] = o.lower;
            }
        }
        Float64Array::from(out.as_slice())
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
    #[wasm_bindgen(js_name = isReady)]
    pub fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    #[wasm_bindgen(js_name = warmupPeriod)]
    pub fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
}

// ---------- Standard Error Bands (scalar input) ----------

#[wasm_bindgen(js_name = StandardErrorBands)]
pub struct WasmStandardErrorBands {
    inner: wc::StandardErrorBands,
}

#[wasm_bindgen(js_class = StandardErrorBands)]
impl WasmStandardErrorBands {
    #[wasm_bindgen(constructor)]
    pub fn new(period: usize, multiplier: f64) -> Result<WasmStandardErrorBands, JsError> {
        Ok(Self {
            inner: wc::StandardErrorBands::new(period, multiplier).map_err(map_err)?,
        })
    }
    pub fn update(&mut self, value: f64) -> JsValue {
        match self.inner.update(value) {
            Some(o) => {
                let obj = Object::new();
                Reflect::set(&obj, &"upper".into(), &o.upper.into()).ok();
                Reflect::set(&obj, &"middle".into(), &o.middle.into()).ok();
                Reflect::set(&obj, &"lower".into(), &o.lower.into()).ok();
                obj.into()
            }
            None => JsValue::NULL,
        }
    }
    pub fn batch(&mut self, prices: &[f64]) -> Float64Array {
        let n = prices.len();
        let mut out = vec![f64::NAN; n * 3];
        for (i, p) in prices.iter().enumerate() {
            if let Some(o) = self.inner.update(*p) {
                out[i * 3] = o.upper;
                out[i * 3 + 1] = o.middle;
                out[i * 3 + 2] = o.lower;
            }
        }
        Float64Array::from(out.as_slice())
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
    #[wasm_bindgen(js_name = isReady)]
    pub fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    #[wasm_bindgen(js_name = warmupPeriod)]
    pub fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
}

// ---------- Double Bollinger (scalar input, 5 outputs) ----------

#[wasm_bindgen(js_name = DoubleBollinger)]
pub struct WasmDoubleBollinger {
    inner: wc::DoubleBollinger,
}

#[wasm_bindgen(js_class = DoubleBollinger)]
impl WasmDoubleBollinger {
    #[wasm_bindgen(constructor)]
    pub fn new(period: usize, k_inner: f64, k_outer: f64) -> Result<WasmDoubleBollinger, JsError> {
        Ok(Self {
            inner: wc::DoubleBollinger::new(period, k_inner, k_outer).map_err(map_err)?,
        })
    }
    pub fn update(&mut self, value: f64) -> JsValue {
        match self.inner.update(value) {
            Some(o) => {
                let obj = Object::new();
                Reflect::set(&obj, &"upperOuter".into(), &o.upper_outer.into()).ok();
                Reflect::set(&obj, &"upperInner".into(), &o.upper_inner.into()).ok();
                Reflect::set(&obj, &"middle".into(), &o.middle.into()).ok();
                Reflect::set(&obj, &"lowerInner".into(), &o.lower_inner.into()).ok();
                Reflect::set(&obj, &"lowerOuter".into(), &o.lower_outer.into()).ok();
                obj.into()
            }
            None => JsValue::NULL,
        }
    }
    /// Flat `[u_o, u_i, m, l_i, l_o, ...]`, length `5 * n`.
    pub fn batch(&mut self, prices: &[f64]) -> Float64Array {
        let n = prices.len();
        let mut out = vec![f64::NAN; n * 5];
        for (i, p) in prices.iter().enumerate() {
            if let Some(o) = self.inner.update(*p) {
                out[i * 5] = o.upper_outer;
                out[i * 5 + 1] = o.upper_inner;
                out[i * 5 + 2] = o.middle;
                out[i * 5 + 3] = o.lower_inner;
                out[i * 5 + 4] = o.lower_outer;
            }
        }
        Float64Array::from(out.as_slice())
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
    #[wasm_bindgen(js_name = isReady)]
    pub fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    #[wasm_bindgen(js_name = warmupPeriod)]
    pub fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
}

// ---------- TTM Squeeze ----------

#[wasm_bindgen(js_name = TtmSqueeze)]
pub struct WasmTtmSqueeze {
    inner: wc::TtmSqueeze,
}

#[wasm_bindgen(js_class = TtmSqueeze)]
impl WasmTtmSqueeze {
    #[wasm_bindgen(constructor)]
    pub fn new(period: usize, bb_mult: f64, kc_mult: f64) -> Result<WasmTtmSqueeze, JsError> {
        Ok(Self {
            inner: wc::TtmSqueeze::new(period, bb_mult, kc_mult).map_err(map_err)?,
        })
    }
    pub fn update(&mut self, high: f64, low: f64, close: f64) -> Result<JsValue, JsError> {
        let c = make_candle(high, low, close, 0.0)?;
        Ok(match self.inner.update(c) {
            Some(o) => {
                let obj = Object::new();
                Reflect::set(&obj, &"squeeze".into(), &o.squeeze.into()).ok();
                Reflect::set(&obj, &"momentum".into(), &o.momentum.into()).ok();
                obj.into()
            }
            None => JsValue::NULL,
        })
    }
    /// Flat `[sq0, mom0, sq1, mom1, ...]`, length `2 * n`.
    pub fn batch(
        &mut self,
        high: &[f64],
        low: &[f64],
        close: &[f64],
    ) -> Result<Float64Array, JsError> {
        let n = high.len();
        if low.len() != n || close.len() != n {
            return Err(JsError::new("high, low, close must be equal length"));
        }
        let mut out = vec![f64::NAN; n * 2];
        for i in 0..n {
            let c = make_candle(high[i], low[i], close[i], 0.0)?;
            if let Some(o) = self.inner.update(c) {
                out[i * 2] = o.squeeze;
                out[i * 2 + 1] = o.momentum;
            }
        }
        Ok(Float64Array::from(out.as_slice()))
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
    #[wasm_bindgen(js_name = isReady)]
    pub fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    #[wasm_bindgen(js_name = warmupPeriod)]
    pub fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
}

// ---------- Fractal Chaos Bands ----------

#[wasm_bindgen(js_name = FractalChaosBands)]
pub struct WasmFractalChaosBands {
    inner: wc::FractalChaosBands,
}

#[wasm_bindgen(js_class = FractalChaosBands)]
impl WasmFractalChaosBands {
    #[wasm_bindgen(constructor)]
    pub fn new(k: usize) -> Result<WasmFractalChaosBands, JsError> {
        Ok(Self {
            inner: wc::FractalChaosBands::new(k).map_err(map_err)?,
        })
    }
    pub fn update(&mut self, high: f64, low: f64) -> Result<JsValue, JsError> {
        let c = make_candle(high, low, low, 0.0)?;
        Ok(match self.inner.update(c) {
            Some(o) => {
                let obj = Object::new();
                Reflect::set(&obj, &"upper".into(), &o.upper.into()).ok();
                Reflect::set(&obj, &"lower".into(), &o.lower.into()).ok();
                obj.into()
            }
            None => JsValue::NULL,
        })
    }
    /// Flat `[u0, l0, u1, l1, ...]`, length `2 * n`.
    pub fn batch(&mut self, high: &[f64], low: &[f64]) -> Result<Float64Array, JsError> {
        let n = high.len();
        if low.len() != n {
            return Err(JsError::new("high and low must be equal length"));
        }
        let mut out = vec![f64::NAN; n * 2];
        for i in 0..n {
            let c = make_candle(high[i], low[i], low[i], 0.0)?;
            if let Some(o) = self.inner.update(c) {
                out[i * 2] = o.upper;
                out[i * 2 + 1] = o.lower;
            }
        }
        Ok(Float64Array::from(out.as_slice()))
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
    #[wasm_bindgen(js_name = isReady)]
    pub fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    #[wasm_bindgen(js_name = warmupPeriod)]
    pub fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
}

// ---------- VWAP StdDev Bands ----------

#[wasm_bindgen(js_name = VwapStdDevBands)]
pub struct WasmVwapStdDevBands {
    inner: wc::VwapStdDevBands,
}

#[wasm_bindgen(js_class = VwapStdDevBands)]
impl WasmVwapStdDevBands {
    #[wasm_bindgen(constructor)]
    pub fn new(multiplier: f64) -> Result<WasmVwapStdDevBands, JsError> {
        Ok(Self {
            inner: wc::VwapStdDevBands::new(multiplier).map_err(map_err)?,
        })
    }
    pub fn update(
        &mut self,
        high: f64,
        low: f64,
        close: f64,
        volume: f64,
    ) -> Result<JsValue, JsError> {
        let c = make_candle(high, low, close, volume)?;
        Ok(match self.inner.update(c) {
            Some(o) => {
                let obj = Object::new();
                Reflect::set(&obj, &"upper".into(), &o.upper.into()).ok();
                Reflect::set(&obj, &"middle".into(), &o.middle.into()).ok();
                Reflect::set(&obj, &"lower".into(), &o.lower.into()).ok();
                Reflect::set(&obj, &"stddev".into(), &o.stddev.into()).ok();
                obj.into()
            }
            None => JsValue::NULL,
        })
    }
    /// Flat `[u0, m0, l0, sd0, ...]`, length `4 * n`.
    pub fn batch(
        &mut self,
        high: &[f64],
        low: &[f64],
        close: &[f64],
        volume: &[f64],
    ) -> Result<Float64Array, JsError> {
        let n = high.len();
        if low.len() != n || close.len() != n || volume.len() != n {
            return Err(JsError::new(
                "high, low, close, volume must be equal length",
            ));
        }
        let mut out = vec![f64::NAN; n * 4];
        for i in 0..n {
            let c = make_candle(high[i], low[i], close[i], volume[i])?;
            if let Some(o) = self.inner.update(c) {
                out[i * 4] = o.upper;
                out[i * 4 + 1] = o.middle;
                out[i * 4 + 2] = o.lower;
                out[i * 4 + 3] = o.stddev;
            }
        }
        Ok(Float64Array::from(out.as_slice()))
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
    #[wasm_bindgen(js_name = isReady)]
    pub fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    #[wasm_bindgen(js_name = warmupPeriod)]
    pub fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
}

// ============================== Pivots & S/R ==============================

#[wasm_bindgen(js_name = ClassicPivots)]
pub struct WasmClassicPivots {
    inner: wc::ClassicPivots,
}

impl Default for WasmClassicPivots {
    fn default() -> Self {
        Self::new()
    }
}

#[wasm_bindgen(js_class = ClassicPivots)]
impl WasmClassicPivots {
    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmClassicPivots {
        Self {
            inner: wc::ClassicPivots::new(),
        }
    }
    pub fn update(&mut self, high: f64, low: f64, close: f64) -> Result<JsValue, JsError> {
        let c = make_candle(high, low, close, 0.0)?;
        Ok(match self.inner.update(c) {
            Some(o) => {
                let obj = Object::new();
                Reflect::set(&obj, &"pp".into(), &o.pp.into()).ok();
                Reflect::set(&obj, &"r1".into(), &o.r1.into()).ok();
                Reflect::set(&obj, &"r2".into(), &o.r2.into()).ok();
                Reflect::set(&obj, &"r3".into(), &o.r3.into()).ok();
                Reflect::set(&obj, &"s1".into(), &o.s1.into()).ok();
                Reflect::set(&obj, &"s2".into(), &o.s2.into()).ok();
                Reflect::set(&obj, &"s3".into(), &o.s3.into()).ok();
                obj.into()
            }
            None => JsValue::NULL,
        })
    }
    pub fn batch(
        &mut self,
        high: &[f64],
        low: &[f64],
        close: &[f64],
    ) -> Result<Float64Array, JsError> {
        if high.len() != low.len() || low.len() != close.len() {
            return Err(JsError::new("high, low, close must be equal length"));
        }
        let n = high.len();
        let mut out = vec![f64::NAN; n * 7];
        for i in 0..n {
            let c = make_candle(high[i], low[i], close[i], 0.0)?;
            if let Some(o) = self.inner.update(c) {
                out[i * 7] = o.pp;
                out[i * 7 + 1] = o.r1;
                out[i * 7 + 2] = o.r2;
                out[i * 7 + 3] = o.r3;
                out[i * 7 + 4] = o.s1;
                out[i * 7 + 5] = o.s2;
                out[i * 7 + 6] = o.s3;
            }
        }
        Ok(Float64Array::from(out.as_slice()))
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
    #[wasm_bindgen(js_name = isReady)]
    pub fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    #[wasm_bindgen(js_name = warmupPeriod)]
    pub fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
}

#[wasm_bindgen(js_name = FibonacciPivots)]
pub struct WasmFibonacciPivots {
    inner: wc::FibonacciPivots,
}

impl Default for WasmFibonacciPivots {
    fn default() -> Self {
        Self::new()
    }
}

#[wasm_bindgen(js_class = FibonacciPivots)]
impl WasmFibonacciPivots {
    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmFibonacciPivots {
        Self {
            inner: wc::FibonacciPivots::new(),
        }
    }
    pub fn update(&mut self, high: f64, low: f64, close: f64) -> Result<JsValue, JsError> {
        let c = make_candle(high, low, close, 0.0)?;
        Ok(match self.inner.update(c) {
            Some(o) => {
                let obj = Object::new();
                Reflect::set(&obj, &"pp".into(), &o.pp.into()).ok();
                Reflect::set(&obj, &"r1".into(), &o.r1.into()).ok();
                Reflect::set(&obj, &"r2".into(), &o.r2.into()).ok();
                Reflect::set(&obj, &"r3".into(), &o.r3.into()).ok();
                Reflect::set(&obj, &"s1".into(), &o.s1.into()).ok();
                Reflect::set(&obj, &"s2".into(), &o.s2.into()).ok();
                Reflect::set(&obj, &"s3".into(), &o.s3.into()).ok();
                obj.into()
            }
            None => JsValue::NULL,
        })
    }
    pub fn batch(
        &mut self,
        high: &[f64],
        low: &[f64],
        close: &[f64],
    ) -> Result<Float64Array, JsError> {
        if high.len() != low.len() || low.len() != close.len() {
            return Err(JsError::new("high, low, close must be equal length"));
        }
        let n = high.len();
        let mut out = vec![f64::NAN; n * 7];
        for i in 0..n {
            let c = make_candle(high[i], low[i], close[i], 0.0)?;
            if let Some(o) = self.inner.update(c) {
                out[i * 7] = o.pp;
                out[i * 7 + 1] = o.r1;
                out[i * 7 + 2] = o.r2;
                out[i * 7 + 3] = o.r3;
                out[i * 7 + 4] = o.s1;
                out[i * 7 + 5] = o.s2;
                out[i * 7 + 6] = o.s3;
            }
        }
        Ok(Float64Array::from(out.as_slice()))
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
    #[wasm_bindgen(js_name = isReady)]
    pub fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    #[wasm_bindgen(js_name = warmupPeriod)]
    pub fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
}

#[wasm_bindgen(js_name = Camarilla)]
pub struct WasmCamarilla {
    inner: wc::Camarilla,
}

impl Default for WasmCamarilla {
    fn default() -> Self {
        Self::new()
    }
}

#[wasm_bindgen(js_class = Camarilla)]
impl WasmCamarilla {
    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmCamarilla {
        Self {
            inner: wc::Camarilla::new(),
        }
    }
    pub fn update(&mut self, high: f64, low: f64, close: f64) -> Result<JsValue, JsError> {
        let c = make_candle(high, low, close, 0.0)?;
        Ok(match self.inner.update(c) {
            Some(o) => {
                let obj = Object::new();
                Reflect::set(&obj, &"pp".into(), &o.pp.into()).ok();
                Reflect::set(&obj, &"r1".into(), &o.r1.into()).ok();
                Reflect::set(&obj, &"r2".into(), &o.r2.into()).ok();
                Reflect::set(&obj, &"r3".into(), &o.r3.into()).ok();
                Reflect::set(&obj, &"r4".into(), &o.r4.into()).ok();
                Reflect::set(&obj, &"s1".into(), &o.s1.into()).ok();
                Reflect::set(&obj, &"s2".into(), &o.s2.into()).ok();
                Reflect::set(&obj, &"s3".into(), &o.s3.into()).ok();
                Reflect::set(&obj, &"s4".into(), &o.s4.into()).ok();
                obj.into()
            }
            None => JsValue::NULL,
        })
    }
    pub fn batch(
        &mut self,
        high: &[f64],
        low: &[f64],
        close: &[f64],
    ) -> Result<Float64Array, JsError> {
        if high.len() != low.len() || low.len() != close.len() {
            return Err(JsError::new("high, low, close must be equal length"));
        }
        let n = high.len();
        let mut out = vec![f64::NAN; n * 9];
        for i in 0..n {
            let c = make_candle(high[i], low[i], close[i], 0.0)?;
            if let Some(o) = self.inner.update(c) {
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
        Ok(Float64Array::from(out.as_slice()))
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
    #[wasm_bindgen(js_name = isReady)]
    pub fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    #[wasm_bindgen(js_name = warmupPeriod)]
    pub fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
}

#[wasm_bindgen(js_name = WoodiePivots)]
pub struct WasmWoodiePivots {
    inner: wc::WoodiePivots,
}

impl Default for WasmWoodiePivots {
    fn default() -> Self {
        Self::new()
    }
}

#[wasm_bindgen(js_class = WoodiePivots)]
impl WasmWoodiePivots {
    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmWoodiePivots {
        Self {
            inner: wc::WoodiePivots::new(),
        }
    }
    pub fn update(&mut self, high: f64, low: f64, close: f64) -> Result<JsValue, JsError> {
        let c = make_candle(high, low, close, 0.0)?;
        Ok(match self.inner.update(c) {
            Some(o) => {
                let obj = Object::new();
                Reflect::set(&obj, &"pp".into(), &o.pp.into()).ok();
                Reflect::set(&obj, &"r1".into(), &o.r1.into()).ok();
                Reflect::set(&obj, &"r2".into(), &o.r2.into()).ok();
                Reflect::set(&obj, &"s1".into(), &o.s1.into()).ok();
                Reflect::set(&obj, &"s2".into(), &o.s2.into()).ok();
                obj.into()
            }
            None => JsValue::NULL,
        })
    }
    pub fn batch(
        &mut self,
        high: &[f64],
        low: &[f64],
        close: &[f64],
    ) -> Result<Float64Array, JsError> {
        if high.len() != low.len() || low.len() != close.len() {
            return Err(JsError::new("high, low, close must be equal length"));
        }
        let n = high.len();
        let mut out = vec![f64::NAN; n * 5];
        for i in 0..n {
            let c = make_candle(high[i], low[i], close[i], 0.0)?;
            if let Some(o) = self.inner.update(c) {
                out[i * 5] = o.pp;
                out[i * 5 + 1] = o.r1;
                out[i * 5 + 2] = o.r2;
                out[i * 5 + 3] = o.s1;
                out[i * 5 + 4] = o.s2;
            }
        }
        Ok(Float64Array::from(out.as_slice()))
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
    #[wasm_bindgen(js_name = isReady)]
    pub fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    #[wasm_bindgen(js_name = warmupPeriod)]
    pub fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
}

#[wasm_bindgen(js_name = DemarkPivots)]
pub struct WasmDemarkPivots {
    inner: wc::DemarkPivots,
}

impl Default for WasmDemarkPivots {
    fn default() -> Self {
        Self::new()
    }
}

#[wasm_bindgen(js_class = DemarkPivots)]
impl WasmDemarkPivots {
    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmDemarkPivots {
        Self {
            inner: wc::DemarkPivots::new(),
        }
    }
    pub fn update(
        &mut self,
        open: f64,
        high: f64,
        low: f64,
        close: f64,
    ) -> Result<JsValue, JsError> {
        let c = wc::Candle::new(open, high, low, close, 0.0, 0).map_err(map_err)?;
        Ok(match self.inner.update(c) {
            Some(o) => {
                let obj = Object::new();
                Reflect::set(&obj, &"pp".into(), &o.pp.into()).ok();
                Reflect::set(&obj, &"r1".into(), &o.r1.into()).ok();
                Reflect::set(&obj, &"s1".into(), &o.s1.into()).ok();
                obj.into()
            }
            None => JsValue::NULL,
        })
    }
    pub fn batch(
        &mut self,
        open: &[f64],
        high: &[f64],
        low: &[f64],
        close: &[f64],
    ) -> Result<Float64Array, JsError> {
        if open.len() != high.len() || high.len() != low.len() || low.len() != close.len() {
            return Err(JsError::new("open, high, low, close must be equal length"));
        }
        let n = open.len();
        let mut out = vec![f64::NAN; n * 3];
        for i in 0..n {
            let c = wc::Candle::new(open[i], high[i], low[i], close[i], 0.0, 0).map_err(map_err)?;
            if let Some(o) = self.inner.update(c) {
                out[i * 3] = o.pp;
                out[i * 3 + 1] = o.r1;
                out[i * 3 + 2] = o.s1;
            }
        }
        Ok(Float64Array::from(out.as_slice()))
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
    #[wasm_bindgen(js_name = isReady)]
    pub fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    #[wasm_bindgen(js_name = warmupPeriod)]
    pub fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
}

#[wasm_bindgen(js_name = WilliamsFractals)]
pub struct WasmWilliamsFractals {
    inner: wc::WilliamsFractals,
}

impl Default for WasmWilliamsFractals {
    fn default() -> Self {
        Self::new()
    }
}

#[wasm_bindgen(js_class = WilliamsFractals)]
impl WasmWilliamsFractals {
    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmWilliamsFractals {
        Self {
            inner: wc::WilliamsFractals::new(),
        }
    }
    /// Returns `{ up, down }` where each is the fractal price or `NaN` when no
    /// fractal was confirmed at the centre of the most recent 5-bar window.
    /// Returns `null` during the four-bar warmup.
    pub fn update(&mut self, high: f64, low: f64) -> Result<JsValue, JsError> {
        let c = make_candle(high, low, low, 0.0)?;
        Ok(match self.inner.update(c) {
            Some(o) => {
                let obj = Object::new();
                Reflect::set(&obj, &"up".into(), &o.up.unwrap_or(f64::NAN).into()).ok();
                Reflect::set(&obj, &"down".into(), &o.down.unwrap_or(f64::NAN).into()).ok();
                obj.into()
            }
            None => JsValue::NULL,
        })
    }
    pub fn batch(&mut self, high: &[f64], low: &[f64]) -> Result<Float64Array, JsError> {
        if high.len() != low.len() {
            return Err(JsError::new("high and low must be equal length"));
        }
        let n = high.len();
        let mut out = vec![f64::NAN; n * 2];
        for i in 0..n {
            let c = make_candle(high[i], low[i], low[i], 0.0)?;
            if let Some(o) = self.inner.update(c) {
                if let Some(v) = o.up {
                    out[i * 2] = v;
                }
                if let Some(v) = o.down {
                    out[i * 2 + 1] = v;
                }
            }
        }
        Ok(Float64Array::from(out.as_slice()))
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
    #[wasm_bindgen(js_name = isReady)]
    pub fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    #[wasm_bindgen(js_name = warmupPeriod)]
    pub fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
}

#[wasm_bindgen(js_name = ZigZag)]
pub struct WasmZigZag {
    inner: wc::ZigZag,
}

#[wasm_bindgen(js_class = ZigZag)]
impl WasmZigZag {
    #[wasm_bindgen(constructor)]
    pub fn new(threshold: f64) -> Result<WasmZigZag, JsError> {
        Ok(Self {
            inner: wc::ZigZag::new(threshold).map_err(map_err)?,
        })
    }
    pub fn update(&mut self, high: f64, low: f64) -> Result<JsValue, JsError> {
        let c = make_candle(high, low, low, 0.0)?;
        Ok(match self.inner.update(c) {
            Some(o) => {
                let obj = Object::new();
                Reflect::set(&obj, &"swing".into(), &o.swing.into()).ok();
                Reflect::set(&obj, &"direction".into(), &o.direction.into()).ok();
                obj.into()
            }
            None => JsValue::NULL,
        })
    }
    pub fn batch(&mut self, high: &[f64], low: &[f64]) -> Result<Float64Array, JsError> {
        if high.len() != low.len() {
            return Err(JsError::new("high and low must be equal length"));
        }
        let n = high.len();
        let mut out = vec![f64::NAN; n * 2];
        for i in 0..n {
            let c = make_candle(high[i], low[i], low[i], 0.0)?;
            if let Some(o) = self.inner.update(c) {
                out[i * 2] = o.swing;
                out[i * 2 + 1] = o.direction;
            }
        }
        Ok(Float64Array::from(out.as_slice()))
    }
    #[wasm_bindgen(js_name = threshold)]
    pub fn threshold(&self) -> f64 {
        self.inner.threshold()
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
    #[wasm_bindgen(js_name = isReady)]
    pub fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    #[wasm_bindgen(js_name = warmupPeriod)]
    pub fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
}
// ---------- TD Setup ----------

#[wasm_bindgen(js_name = TDSetup)]
pub struct WasmTdSetup {
    inner: wc::TdSetup,
}

#[wasm_bindgen(js_class = TDSetup)]
impl WasmTdSetup {
    #[wasm_bindgen(constructor)]
    pub fn new(lookback: usize, target: usize) -> Result<WasmTdSetup, JsError> {
        Ok(Self {
            inner: wc::TdSetup::new(lookback, target).map_err(map_err)?,
        })
    }
    pub fn update(&mut self, high: f64, low: f64, close: f64) -> Result<Option<f64>, JsError> {
        let c = make_candle(high, low, close, 0.0)?;
        Ok(self.inner.update(c))
    }
    pub fn batch(
        &mut self,
        high: &[f64],
        low: &[f64],
        close: &[f64],
    ) -> Result<Float64Array, JsError> {
        if high.len() != low.len() || low.len() != close.len() {
            return Err(JsError::new("high, low, close must be equal length"));
        }
        let mut out = Vec::with_capacity(high.len());
        for i in 0..high.len() {
            let c = make_candle(high[i], low[i], close[i], 0.0)?;
            out.push(self.inner.update(c).unwrap_or(f64::NAN));
        }
        Ok(Float64Array::from(out.as_slice()))
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
    #[wasm_bindgen(js_name = isReady)]
    pub fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    #[wasm_bindgen(js_name = warmupPeriod)]
    pub fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
}

// ---------- TD Sequential ----------

#[wasm_bindgen(js_name = TDSequential)]
pub struct WasmTdSequential {
    inner: wc::TdSequential,
}

#[wasm_bindgen(js_class = TDSequential)]
impl WasmTdSequential {
    #[wasm_bindgen(constructor)]
    pub fn new(
        setup_lookback: usize,
        setup_target: usize,
        countdown_lookback: usize,
        countdown_target: usize,
    ) -> Result<WasmTdSequential, JsError> {
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
    /// Streaming update. Returns `{ setup, countdown, direction }` once warm,
    /// else `null`.
    pub fn update(&mut self, high: f64, low: f64, close: f64) -> Result<JsValue, JsError> {
        let c = make_candle(high, low, close, 0.0)?;
        Ok(match self.inner.update(c) {
            Some(o) => {
                let obj = Object::new();
                Reflect::set(&obj, &"setup".into(), &o.setup.into()).ok();
                Reflect::set(&obj, &"countdown".into(), &o.countdown.into()).ok();
                Reflect::set(&obj, &"direction".into(), &o.direction.into()).ok();
                obj.into()
            }
            None => JsValue::NULL,
        })
    }
    /// Batch returns a flat `Float64Array` `[setup0, countdown0, direction0, ...]`.
    pub fn batch(
        &mut self,
        high: &[f64],
        low: &[f64],
        close: &[f64],
    ) -> Result<Float64Array, JsError> {
        if high.len() != low.len() || low.len() != close.len() {
            return Err(JsError::new("high, low, close must be equal length"));
        }
        let n = high.len();
        let mut out = vec![f64::NAN; n * 3];
        for i in 0..n {
            let c = make_candle(high[i], low[i], close[i], 0.0)?;
            if let Some(o) = self.inner.update(c) {
                out[i * 3] = o.setup;
                out[i * 3 + 1] = o.countdown;
                out[i * 3 + 2] = o.direction;
            }
        }
        Ok(Float64Array::from(out.as_slice()))
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
    #[wasm_bindgen(js_name = isReady)]
    pub fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    #[wasm_bindgen(js_name = warmupPeriod)]
    pub fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
}

// ---------- TD DeMarker ----------

#[wasm_bindgen(js_name = TDDeMarker)]
pub struct WasmTdDeMarker {
    inner: wc::TdDeMarker,
}

#[wasm_bindgen(js_class = TDDeMarker)]
impl WasmTdDeMarker {
    #[wasm_bindgen(constructor)]
    pub fn new(period: usize) -> Result<WasmTdDeMarker, JsError> {
        Ok(Self {
            inner: wc::TdDeMarker::new(period).map_err(map_err)?,
        })
    }
    pub fn update(&mut self, high: f64, low: f64) -> Result<Option<f64>, JsError> {
        let c = make_candle(high, low, low, 0.0)?;
        Ok(self.inner.update(c))
    }
    pub fn batch(&mut self, high: &[f64], low: &[f64]) -> Result<Float64Array, JsError> {
        if high.len() != low.len() {
            return Err(JsError::new("high and low must be equal length"));
        }
        let mut out = Vec::with_capacity(high.len());
        for i in 0..high.len() {
            let c = make_candle(high[i], low[i], low[i], 0.0)?;
            out.push(self.inner.update(c).unwrap_or(f64::NAN));
        }
        Ok(Float64Array::from(out.as_slice()))
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
    #[wasm_bindgen(js_name = isReady)]
    pub fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    #[wasm_bindgen(js_name = warmupPeriod)]
    pub fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
}

// ---------- TD REI ----------

#[wasm_bindgen(js_name = TDREI)]
pub struct WasmTdRei {
    inner: wc::TdRei,
}

#[wasm_bindgen(js_class = TDREI)]
impl WasmTdRei {
    #[wasm_bindgen(constructor)]
    pub fn new(period: usize) -> Result<WasmTdRei, JsError> {
        Ok(Self {
            inner: wc::TdRei::new(period).map_err(map_err)?,
        })
    }
    pub fn update(&mut self, high: f64, low: f64) -> Result<Option<f64>, JsError> {
        let c = make_candle(high, low, low, 0.0)?;
        Ok(self.inner.update(c))
    }
    pub fn batch(&mut self, high: &[f64], low: &[f64]) -> Result<Float64Array, JsError> {
        if high.len() != low.len() {
            return Err(JsError::new("high and low must be equal length"));
        }
        let mut out = Vec::with_capacity(high.len());
        for i in 0..high.len() {
            let c = make_candle(high[i], low[i], low[i], 0.0)?;
            out.push(self.inner.update(c).unwrap_or(f64::NAN));
        }
        Ok(Float64Array::from(out.as_slice()))
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
    #[wasm_bindgen(js_name = isReady)]
    pub fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    #[wasm_bindgen(js_name = warmupPeriod)]
    pub fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
}

// ---------- TD Pressure ----------

#[wasm_bindgen(js_name = TDPressure)]
pub struct WasmTdPressure {
    inner: wc::TdPressure,
}

#[wasm_bindgen(js_class = TDPressure)]
impl WasmTdPressure {
    #[wasm_bindgen(constructor)]
    pub fn new(period: usize) -> Result<WasmTdPressure, JsError> {
        Ok(Self {
            inner: wc::TdPressure::new(period).map_err(map_err)?,
        })
    }
    pub fn update(
        &mut self,
        open: f64,
        high: f64,
        low: f64,
        close: f64,
        volume: f64,
    ) -> Result<Option<f64>, JsError> {
        let c = wc::Candle::new(open, high, low, close, volume, 0).map_err(map_err)?;
        Ok(self.inner.update(c))
    }
    pub fn batch(
        &mut self,
        open: &[f64],
        high: &[f64],
        low: &[f64],
        close: &[f64],
        volume: &[f64],
    ) -> Result<Float64Array, JsError> {
        if open.len() != high.len()
            || high.len() != low.len()
            || low.len() != close.len()
            || close.len() != volume.len()
        {
            return Err(JsError::new(
                "open, high, low, close, volume must be equal length",
            ));
        }
        let mut out = Vec::with_capacity(open.len());
        for i in 0..open.len() {
            let c = wc::Candle::new(open[i], high[i], low[i], close[i], volume[i], 0)
                .map_err(map_err)?;
            out.push(self.inner.update(c).unwrap_or(f64::NAN));
        }
        Ok(Float64Array::from(out.as_slice()))
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
    #[wasm_bindgen(js_name = isReady)]
    pub fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    #[wasm_bindgen(js_name = warmupPeriod)]
    pub fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
}

// ---------- TD Combo ----------

#[wasm_bindgen(js_name = TDCombo)]
pub struct WasmTdCombo {
    inner: wc::TdCombo,
}

#[wasm_bindgen(js_class = TDCombo)]
impl WasmTdCombo {
    #[wasm_bindgen(constructor)]
    pub fn new(
        setup_lookback: usize,
        setup_target: usize,
        countdown_lookback: usize,
        countdown_target: usize,
    ) -> Result<WasmTdCombo, JsError> {
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
    pub fn update(&mut self, high: f64, low: f64, close: f64) -> Result<Option<f64>, JsError> {
        let c = make_candle(high, low, close, 0.0)?;
        Ok(self.inner.update(c))
    }
    pub fn batch(
        &mut self,
        high: &[f64],
        low: &[f64],
        close: &[f64],
    ) -> Result<Float64Array, JsError> {
        if high.len() != low.len() || low.len() != close.len() {
            return Err(JsError::new("high, low, close must be equal length"));
        }
        let mut out = Vec::with_capacity(high.len());
        for i in 0..high.len() {
            let c = make_candle(high[i], low[i], close[i], 0.0)?;
            out.push(self.inner.update(c).unwrap_or(f64::NAN));
        }
        Ok(Float64Array::from(out.as_slice()))
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
    #[wasm_bindgen(js_name = isReady)]
    pub fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    #[wasm_bindgen(js_name = warmupPeriod)]
    pub fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
}

// ---------- TD Countdown ----------

#[wasm_bindgen(js_name = TDCountdown)]
pub struct WasmTdCountdown {
    inner: wc::TdCountdown,
}

#[wasm_bindgen(js_class = TDCountdown)]
impl WasmTdCountdown {
    #[wasm_bindgen(constructor)]
    pub fn new(
        setup_lookback: usize,
        setup_target: usize,
        countdown_lookback: usize,
        countdown_target: usize,
    ) -> Result<WasmTdCountdown, JsError> {
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
    pub fn update(&mut self, high: f64, low: f64, close: f64) -> Result<Option<f64>, JsError> {
        let c = make_candle(high, low, close, 0.0)?;
        Ok(self.inner.update(c))
    }
    pub fn batch(
        &mut self,
        high: &[f64],
        low: &[f64],
        close: &[f64],
    ) -> Result<Float64Array, JsError> {
        if high.len() != low.len() || low.len() != close.len() {
            return Err(JsError::new("high, low, close must be equal length"));
        }
        let mut out = Vec::with_capacity(high.len());
        for i in 0..high.len() {
            let c = make_candle(high[i], low[i], close[i], 0.0)?;
            out.push(self.inner.update(c).unwrap_or(f64::NAN));
        }
        Ok(Float64Array::from(out.as_slice()))
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
    #[wasm_bindgen(js_name = isReady)]
    pub fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    #[wasm_bindgen(js_name = warmupPeriod)]
    pub fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
}

// ---------- TD Lines ----------

#[wasm_bindgen(js_name = TDLines)]
pub struct WasmTdLines {
    inner: wc::TdLines,
}

#[wasm_bindgen(js_class = TDLines)]
impl WasmTdLines {
    #[wasm_bindgen(constructor)]
    pub fn new(lookback: usize, target: usize) -> Result<WasmTdLines, JsError> {
        Ok(Self {
            inner: wc::TdLines::new(lookback, target).map_err(map_err)?,
        })
    }
    /// Streaming update. Returns `{ resistance, support }` once warm, else `null`.
    pub fn update(&mut self, high: f64, low: f64, close: f64) -> Result<JsValue, JsError> {
        let c = make_candle(high, low, close, 0.0)?;
        Ok(match self.inner.update(c) {
            Some(o) => {
                let obj = Object::new();
                Reflect::set(&obj, &"resistance".into(), &o.resistance.into()).ok();
                Reflect::set(&obj, &"support".into(), &o.support.into()).ok();
                obj.into()
            }
            None => JsValue::NULL,
        })
    }
    /// Batch returns a flat `Float64Array` `[resistance0, support0, ...]`.
    pub fn batch(
        &mut self,
        high: &[f64],
        low: &[f64],
        close: &[f64],
    ) -> Result<Float64Array, JsError> {
        if high.len() != low.len() || low.len() != close.len() {
            return Err(JsError::new("high, low, close must be equal length"));
        }
        let n = high.len();
        let mut out = vec![f64::NAN; n * 2];
        for i in 0..n {
            let c = make_candle(high[i], low[i], close[i], 0.0)?;
            if let Some(o) = self.inner.update(c) {
                out[i * 2] = o.resistance;
                out[i * 2 + 1] = o.support;
            }
        }
        Ok(Float64Array::from(out.as_slice()))
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
    #[wasm_bindgen(js_name = isReady)]
    pub fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    #[wasm_bindgen(js_name = warmupPeriod)]
    pub fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
}

// ---------- TD Range Projection ----------

#[wasm_bindgen(js_name = TDRangeProjection)]
pub struct WasmTdRangeProjection {
    inner: wc::TdRangeProjection,
}

#[wasm_bindgen(js_class = TDRangeProjection)]
impl WasmTdRangeProjection {
    #[wasm_bindgen(constructor)]
    #[allow(clippy::new_without_default)]
    pub fn new() -> WasmTdRangeProjection {
        Self {
            inner: wc::TdRangeProjection::new(),
        }
    }
    /// Streaming update. Returns `{ high, low }` projected for the next bar.
    pub fn update(
        &mut self,
        open: f64,
        high: f64,
        low: f64,
        close: f64,
    ) -> Result<JsValue, JsError> {
        let c = wc::Candle::new(open, high, low, close, 0.0, 0).map_err(map_err)?;
        Ok(match self.inner.update(c) {
            Some(o) => {
                let obj = Object::new();
                Reflect::set(&obj, &"high".into(), &o.high.into()).ok();
                Reflect::set(&obj, &"low".into(), &o.low.into()).ok();
                obj.into()
            }
            None => JsValue::NULL,
        })
    }
    /// Batch returns a flat `Float64Array` `[projHigh0, projLow0, ...]`.
    pub fn batch(
        &mut self,
        open: &[f64],
        high: &[f64],
        low: &[f64],
        close: &[f64],
    ) -> Result<Float64Array, JsError> {
        if open.len() != high.len() || high.len() != low.len() || low.len() != close.len() {
            return Err(JsError::new("open, high, low, close must be equal length"));
        }
        let n = open.len();
        let mut out = vec![f64::NAN; n * 2];
        for i in 0..n {
            let c = wc::Candle::new(open[i], high[i], low[i], close[i], 0.0, 0).map_err(map_err)?;
            if let Some(p) = self.inner.update(c) {
                out[i * 2] = p.high;
                out[i * 2 + 1] = p.low;
            }
        }
        Ok(Float64Array::from(out.as_slice()))
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
    #[wasm_bindgen(js_name = isReady)]
    pub fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    #[wasm_bindgen(js_name = warmupPeriod)]
    pub fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
}

// ---------- TD Differential ----------

#[wasm_bindgen(js_name = TDDifferential)]
pub struct WasmTdDifferential {
    inner: wc::TdDifferential,
}

#[wasm_bindgen(js_class = TDDifferential)]
impl WasmTdDifferential {
    #[wasm_bindgen(constructor)]
    #[allow(clippy::new_without_default)]
    pub fn new() -> WasmTdDifferential {
        Self {
            inner: wc::TdDifferential::new(),
        }
    }
    pub fn update(&mut self, high: f64, low: f64, close: f64) -> Result<Option<f64>, JsError> {
        let c = make_candle(high, low, close, 0.0)?;
        Ok(self.inner.update(c))
    }
    pub fn batch(
        &mut self,
        high: &[f64],
        low: &[f64],
        close: &[f64],
    ) -> Result<Float64Array, JsError> {
        if high.len() != low.len() || low.len() != close.len() {
            return Err(JsError::new("high, low, close must be equal length"));
        }
        let mut out = Vec::with_capacity(high.len());
        for i in 0..high.len() {
            let c = make_candle(high[i], low[i], close[i], 0.0)?;
            out.push(self.inner.update(c).unwrap_or(f64::NAN));
        }
        Ok(Float64Array::from(out.as_slice()))
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
    #[wasm_bindgen(js_name = isReady)]
    pub fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    #[wasm_bindgen(js_name = warmupPeriod)]
    pub fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
}

// ---------- TD Open ----------

#[wasm_bindgen(js_name = TDOpen)]
pub struct WasmTdOpen {
    inner: wc::TdOpen,
}

#[wasm_bindgen(js_class = TDOpen)]
impl WasmTdOpen {
    #[wasm_bindgen(constructor)]
    #[allow(clippy::new_without_default)]
    pub fn new() -> WasmTdOpen {
        Self {
            inner: wc::TdOpen::new(),
        }
    }
    pub fn update(
        &mut self,
        open: f64,
        high: f64,
        low: f64,
        close: f64,
    ) -> Result<Option<f64>, JsError> {
        let c = wc::Candle::new(open, high, low, close, 0.0, 0).map_err(map_err)?;
        Ok(self.inner.update(c))
    }
    pub fn batch(
        &mut self,
        open: &[f64],
        high: &[f64],
        low: &[f64],
        close: &[f64],
    ) -> Result<Float64Array, JsError> {
        if open.len() != high.len() || high.len() != low.len() || low.len() != close.len() {
            return Err(JsError::new("open, high, low, close must be equal length"));
        }
        let mut out = Vec::with_capacity(open.len());
        for i in 0..open.len() {
            let c = wc::Candle::new(open[i], high[i], low[i], close[i], 0.0, 0).map_err(map_err)?;
            out.push(self.inner.update(c).unwrap_or(f64::NAN));
        }
        Ok(Float64Array::from(out.as_slice()))
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
    #[wasm_bindgen(js_name = isReady)]
    pub fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    #[wasm_bindgen(js_name = warmupPeriod)]
    pub fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
}

// ---------- TD Risk Level ----------

#[wasm_bindgen(js_name = TDRiskLevel)]
pub struct WasmTdRiskLevel {
    inner: wc::TdRiskLevel,
}

#[wasm_bindgen(js_class = TDRiskLevel)]
impl WasmTdRiskLevel {
    #[wasm_bindgen(constructor)]
    pub fn new(lookback: usize, target: usize) -> Result<WasmTdRiskLevel, JsError> {
        Ok(Self {
            inner: wc::TdRiskLevel::new(lookback, target).map_err(map_err)?,
        })
    }
    /// Streaming update. Returns `{ buyRisk, sellRisk }` once warm, else `null`.
    pub fn update(&mut self, high: f64, low: f64, close: f64) -> Result<JsValue, JsError> {
        let c = make_candle(high, low, close, 0.0)?;
        Ok(match self.inner.update(c) {
            Some(o) => {
                let obj = Object::new();
                Reflect::set(&obj, &"buyRisk".into(), &o.buy_risk.into()).ok();
                Reflect::set(&obj, &"sellRisk".into(), &o.sell_risk.into()).ok();
                obj.into()
            }
            None => JsValue::NULL,
        })
    }
    /// Batch returns a flat `Float64Array` `[buyRisk0, sellRisk0, ...]`.
    pub fn batch(
        &mut self,
        high: &[f64],
        low: &[f64],
        close: &[f64],
    ) -> Result<Float64Array, JsError> {
        if high.len() != low.len() || low.len() != close.len() {
            return Err(JsError::new("high, low, close must be equal length"));
        }
        let n = high.len();
        let mut out = vec![f64::NAN; n * 2];
        for i in 0..n {
            let c = make_candle(high[i], low[i], close[i], 0.0)?;
            if let Some(o) = self.inner.update(c) {
                out[i * 2] = o.buy_risk;
                out[i * 2 + 1] = o.sell_risk;
            }
        }
        Ok(Float64Array::from(out.as_slice()))
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
    #[wasm_bindgen(js_name = isReady)]
    pub fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    #[wasm_bindgen(js_name = warmupPeriod)]
    pub fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
}

// ---------- Ichimoku ----------

#[wasm_bindgen(js_name = Ichimoku)]
pub struct WasmIchimoku {
    inner: wc::Ichimoku,
}

#[wasm_bindgen(js_class = Ichimoku)]
impl WasmIchimoku {
    #[wasm_bindgen(constructor)]
    pub fn new(
        tenkan_period: usize,
        kijun_period: usize,
        senkou_b_period: usize,
        displacement: usize,
    ) -> Result<WasmIchimoku, JsError> {
        Ok(Self {
            inner: wc::Ichimoku::new(tenkan_period, kijun_period, senkou_b_period, displacement)
                .map_err(map_err)?,
        })
    }
    /// Streaming update. Returns `{ tenkan, kijun, senkouA, senkouB, chikou }`
    /// with `NaN` for any line that is not yet defined.
    pub fn update(&mut self, high: f64, low: f64, close: f64) -> Result<JsValue, JsError> {
        let c = make_candle(high, low, close, 0.0)?;
        Ok(match self.inner.update(c) {
            Some(o) => {
                let obj = Object::new();
                Reflect::set(&obj, &"tenkan".into(), &o.tenkan.unwrap_or(f64::NAN).into()).ok();
                Reflect::set(&obj, &"kijun".into(), &o.kijun.unwrap_or(f64::NAN).into()).ok();
                Reflect::set(
                    &obj,
                    &"senkouA".into(),
                    &o.senkou_a.unwrap_or(f64::NAN).into(),
                )
                .ok();
                Reflect::set(
                    &obj,
                    &"senkouB".into(),
                    &o.senkou_b.unwrap_or(f64::NAN).into(),
                )
                .ok();
                Reflect::set(&obj, &"chikou".into(), &o.chikou.unwrap_or(f64::NAN).into()).ok();
                obj.into()
            }
            None => JsValue::NULL,
        })
    }
    /// Returns `[tenkan0, kijun0, senkouA0, senkouB0, chikou0, tenkan1, ...]`,
    /// length `5 * n`. Cells without a defined value are `NaN`.
    pub fn batch(
        &mut self,
        high: &[f64],
        low: &[f64],
        close: &[f64],
    ) -> Result<Float64Array, JsError> {
        if high.len() != low.len() || low.len() != close.len() {
            return Err(JsError::new("high, low, close must be equal length"));
        }
        let n = high.len();
        let mut out = vec![f64::NAN; n * 5];
        for i in 0..n {
            let c = make_candle(high[i], low[i], close[i], 0.0)?;
            if let Some(o) = self.inner.update(c) {
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
        Ok(Float64Array::from(out.as_slice()))
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
    #[wasm_bindgen(js_name = isReady)]
    pub fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    #[wasm_bindgen(js_name = warmupPeriod)]
    pub fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
}

// ---------- Heikin-Ashi ----------

#[wasm_bindgen(js_name = HeikinAshi)]
pub struct WasmHeikinAshi {
    inner: wc::HeikinAshi,
}

impl Default for WasmHeikinAshi {
    fn default() -> Self {
        Self::new()
    }
}

#[wasm_bindgen(js_class = HeikinAshi)]
impl WasmHeikinAshi {
    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmHeikinAshi {
        Self {
            inner: wc::HeikinAshi::new(),
        }
    }
    /// Streaming update. Returns `{ open, high, low, close }` of the
    /// Heikin-Ashi candle.
    pub fn update(
        &mut self,
        open: f64,
        high: f64,
        low: f64,
        close: f64,
    ) -> Result<JsValue, JsError> {
        let c = wc::Candle::new(open, high, low, close, 0.0, 0).map_err(map_err)?;
        Ok(match self.inner.update(c) {
            Some(o) => {
                let obj = Object::new();
                Reflect::set(&obj, &"open".into(), &o.open.into()).ok();
                Reflect::set(&obj, &"high".into(), &o.high.into()).ok();
                Reflect::set(&obj, &"low".into(), &o.low.into()).ok();
                Reflect::set(&obj, &"close".into(), &o.close.into()).ok();
                obj.into()
            }
            None => JsValue::NULL,
        })
    }
    /// Returns `[open0, high0, low0, close0, open1, ...]`, length `4 * n`.
    pub fn batch(
        &mut self,
        open: &[f64],
        high: &[f64],
        low: &[f64],
        close: &[f64],
    ) -> Result<Float64Array, JsError> {
        if open.len() != high.len() || high.len() != low.len() || low.len() != close.len() {
            return Err(JsError::new("open, high, low, close must be equal length"));
        }
        let n = open.len();
        let mut out = vec![f64::NAN; n * 4];
        for i in 0..n {
            let c = wc::Candle::new(open[i], high[i], low[i], close[i], 0.0, 0).map_err(map_err)?;
            if let Some(o) = self.inner.update(c) {
                out[i * 4] = o.open;
                out[i * 4 + 1] = o.high;
                out[i * 4 + 2] = o.low;
                out[i * 4 + 3] = o.close;
            }
        }
        Ok(Float64Array::from(out.as_slice()))
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
    #[wasm_bindgen(js_name = isReady)]
    pub fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    #[wasm_bindgen(js_name = warmupPeriod)]
    pub fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
}

#[wasm_bindgen(js_name = ValueArea)]
pub struct WasmValueArea {
    inner: wc::ValueArea,
}

#[wasm_bindgen(js_class = ValueArea)]
impl WasmValueArea {
    #[wasm_bindgen(constructor)]
    pub fn new(
        period: usize,
        bin_count: usize,
        value_area_pct: f64,
    ) -> Result<WasmValueArea, JsError> {
        Ok(Self {
            inner: wc::ValueArea::new(period, bin_count, value_area_pct).map_err(map_err)?,
        })
    }
    pub fn batch(
        &mut self,
        high: &[f64],
        low: &[f64],
        volume: &[f64],
    ) -> Result<Float64Array, JsError> {
        if high.len() != low.len() || low.len() != volume.len() {
            return Err(JsError::new("high, low, volume must be equal length"));
        }
        let n = high.len();
        let mut out = vec![f64::NAN; n * 3];
        for i in 0..n {
            let mid = f64::midpoint(high[i], low[i]);
            let c = wc::Candle::new(mid, high[i], low[i], mid, volume[i], 0).map_err(map_err)?;
            if let Some(o) = self.inner.update(c) {
                out[i * 3] = o.poc;
                out[i * 3 + 1] = o.vah;
                out[i * 3 + 2] = o.val;
            }
        }
        Ok(Float64Array::from(out.as_slice()))
    }
    /// Streaming update. Returns `{ poc, vah, val }` once warm, else `null`.
    pub fn update(&mut self, high: f64, low: f64, volume: f64) -> Result<JsValue, JsError> {
        let mid = f64::midpoint(high, low);
        let c = wc::Candle::new(mid, high, low, mid, volume, 0).map_err(map_err)?;
        Ok(match self.inner.update(c) {
            Some(o) => {
                let obj = Object::new();
                Reflect::set(&obj, &"poc".into(), &o.poc.into()).ok();
                Reflect::set(&obj, &"vah".into(), &o.vah.into()).ok();
                Reflect::set(&obj, &"val".into(), &o.val.into()).ok();
                obj.into()
            }
            None => JsValue::NULL,
        })
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
    #[wasm_bindgen(js_name = isReady)]
    pub fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    #[wasm_bindgen(js_name = warmupPeriod)]
    pub fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
}

#[wasm_bindgen(js_name = VolumeProfile)]
pub struct WasmVolumeProfile {
    inner: wc::VolumeProfile,
}

#[wasm_bindgen(js_class = VolumeProfile)]
impl WasmVolumeProfile {
    #[wasm_bindgen(constructor)]
    pub fn new(period: usize, bin_count: usize) -> Result<WasmVolumeProfile, JsError> {
        Ok(Self {
            inner: wc::VolumeProfile::new(period, bin_count).map_err(map_err)?,
        })
    }
    pub fn batch(
        &mut self,
        high: &[f64],
        low: &[f64],
        volume: &[f64],
    ) -> Result<Float64Array, JsError> {
        if high.len() != low.len() || low.len() != volume.len() {
            return Err(JsError::new("high, low, volume must be equal length"));
        }
        let k = self.inner.params().1 + 2;
        let n = high.len();
        let mut out = vec![f64::NAN; n * k];
        for i in 0..n {
            let mid = f64::midpoint(high[i], low[i]);
            let c = wc::Candle::new(mid, high[i], low[i], mid, volume[i], 0).map_err(map_err)?;
            if let Some(o) = self.inner.update(c) {
                out[i * k] = o.price_low;
                out[i * k + 1] = o.price_high;
                for (j, b) in o.bins.iter().enumerate() {
                    out[i * k + 2 + j] = *b;
                }
            }
        }
        Ok(Float64Array::from(out.as_slice()))
    }
    /// Streaming update. Returns `{ priceLow, priceHigh, bins }` once warm, else `null`.
    pub fn update(&mut self, high: f64, low: f64, volume: f64) -> Result<JsValue, JsError> {
        let mid = f64::midpoint(high, low);
        let c = wc::Candle::new(mid, high, low, mid, volume, 0).map_err(map_err)?;
        Ok(match self.inner.update(c) {
            Some(o) => {
                let obj = Object::new();
                Reflect::set(&obj, &"priceLow".into(), &o.price_low.into()).ok();
                Reflect::set(&obj, &"priceHigh".into(), &o.price_high.into()).ok();
                let bins = Float64Array::from(o.bins.as_slice());
                Reflect::set(&obj, &"bins".into(), &bins).ok();
                obj.into()
            }
            None => JsValue::NULL,
        })
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
    #[wasm_bindgen(js_name = isReady)]
    pub fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    #[wasm_bindgen(js_name = warmupPeriod)]
    pub fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
}

#[wasm_bindgen(js_name = TpoProfile)]
pub struct WasmTpoProfile {
    inner: wc::TpoProfile,
}

#[wasm_bindgen(js_class = TpoProfile)]
impl WasmTpoProfile {
    #[wasm_bindgen(constructor)]
    pub fn new(period: usize, bin_count: usize) -> Result<WasmTpoProfile, JsError> {
        Ok(Self {
            inner: wc::TpoProfile::new(period, bin_count).map_err(map_err)?,
        })
    }
    pub fn batch(&mut self, high: &[f64], low: &[f64]) -> Result<Float64Array, JsError> {
        if high.len() != low.len() {
            return Err(JsError::new("high, low must be equal length"));
        }
        let k = self.inner.params().1 + 2;
        let n = high.len();
        let mut out = vec![f64::NAN; n * k];
        for i in 0..n {
            let mid = f64::midpoint(high[i], low[i]);
            let c = wc::Candle::new(mid, high[i], low[i], mid, 1.0, 0).map_err(map_err)?;
            if let Some(o) = self.inner.update(c) {
                out[i * k] = o.price_low;
                out[i * k + 1] = o.price_high;
                for (j, count) in o.counts.iter().enumerate() {
                    out[i * k + 2 + j] = *count;
                }
            }
        }
        Ok(Float64Array::from(out.as_slice()))
    }
    /// Streaming update. Returns `{ priceLow, priceHigh, counts }` once warm, else `null`.
    pub fn update(&mut self, high: f64, low: f64) -> Result<JsValue, JsError> {
        let mid = f64::midpoint(high, low);
        let c = wc::Candle::new(mid, high, low, mid, 1.0, 0).map_err(map_err)?;
        Ok(match self.inner.update(c) {
            Some(o) => {
                let obj = Object::new();
                Reflect::set(&obj, &"priceLow".into(), &o.price_low.into()).ok();
                Reflect::set(&obj, &"priceHigh".into(), &o.price_high.into()).ok();
                let counts = Float64Array::from(o.counts.as_slice());
                Reflect::set(&obj, &"counts".into(), &counts).ok();
                obj.into()
            }
            None => JsValue::NULL,
        })
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
    #[wasm_bindgen(js_name = isReady)]
    pub fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    #[wasm_bindgen(js_name = warmupPeriod)]
    pub fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
}

#[wasm_bindgen(js_name = InitialBalance)]
pub struct WasmInitialBalance {
    inner: wc::InitialBalance,
}

#[wasm_bindgen(js_class = InitialBalance)]
impl WasmInitialBalance {
    #[wasm_bindgen(constructor)]
    pub fn new(period: usize) -> Result<WasmInitialBalance, JsError> {
        Ok(Self {
            inner: wc::InitialBalance::new(period).map_err(map_err)?,
        })
    }
    pub fn batch(&mut self, high: &[f64], low: &[f64]) -> Result<Float64Array, JsError> {
        if high.len() != low.len() {
            return Err(JsError::new("high and low must be equal length"));
        }
        let n = high.len();
        let mut out = vec![f64::NAN; n * 2];
        for i in 0..n {
            let mid = f64::midpoint(high[i], low[i]);
            let c = wc::Candle::new(mid, high[i], low[i], mid, 0.0, 0).map_err(map_err)?;
            if let Some(o) = self.inner.update(c) {
                out[i * 2] = o.high;
                out[i * 2 + 1] = o.low;
            }
        }
        Ok(Float64Array::from(out.as_slice()))
    }
    /// Streaming update. Returns `{ high, low }`.
    pub fn update(&mut self, high: f64, low: f64) -> Result<JsValue, JsError> {
        let mid = f64::midpoint(high, low);
        let c = wc::Candle::new(mid, high, low, mid, 0.0, 0).map_err(map_err)?;
        Ok(match self.inner.update(c) {
            Some(o) => {
                let obj = Object::new();
                Reflect::set(&obj, &"high".into(), &o.high.into()).ok();
                Reflect::set(&obj, &"low".into(), &o.low.into()).ok();
                obj.into()
            }
            None => JsValue::NULL,
        })
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
    #[wasm_bindgen(js_name = isReady)]
    pub fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    #[wasm_bindgen(js_name = isLocked)]
    pub fn is_locked(&self) -> bool {
        self.inner.is_locked()
    }
    #[wasm_bindgen(js_name = warmupPeriod)]
    pub fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
}

#[wasm_bindgen(js_name = OpeningRange)]
pub struct WasmOpeningRange {
    inner: wc::OpeningRange,
}

#[wasm_bindgen(js_class = OpeningRange)]
impl WasmOpeningRange {
    #[wasm_bindgen(constructor)]
    pub fn new(period: usize) -> Result<WasmOpeningRange, JsError> {
        Ok(Self {
            inner: wc::OpeningRange::new(period).map_err(map_err)?,
        })
    }
    pub fn batch(
        &mut self,
        high: &[f64],
        low: &[f64],
        close: &[f64],
    ) -> Result<Float64Array, JsError> {
        if high.len() != low.len() || low.len() != close.len() {
            return Err(JsError::new("high, low, close must be equal length"));
        }
        let n = high.len();
        let mut out = vec![f64::NAN; n * 3];
        for i in 0..n {
            let c =
                wc::Candle::new(close[i], high[i], low[i], close[i], 0.0, 0).map_err(map_err)?;
            if let Some(o) = self.inner.update(c) {
                out[i * 3] = o.high;
                out[i * 3 + 1] = o.low;
                out[i * 3 + 2] = o.breakout_distance;
            }
        }
        Ok(Float64Array::from(out.as_slice()))
    }
    /// Streaming update. Returns `{ high, low, breakoutDistance }`.
    pub fn update(&mut self, high: f64, low: f64, close: f64) -> Result<JsValue, JsError> {
        let c = wc::Candle::new(close, high, low, close, 0.0, 0).map_err(map_err)?;
        Ok(match self.inner.update(c) {
            Some(o) => {
                let obj = Object::new();
                Reflect::set(&obj, &"high".into(), &o.high.into()).ok();
                Reflect::set(&obj, &"low".into(), &o.low.into()).ok();
                Reflect::set(
                    &obj,
                    &"breakoutDistance".into(),
                    &o.breakout_distance.into(),
                )
                .ok();
                obj.into()
            }
            None => JsValue::NULL,
        })
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
    #[wasm_bindgen(js_name = isReady)]
    pub fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    #[wasm_bindgen(js_name = isLocked)]
    pub fn is_locked(&self) -> bool {
        self.inner.is_locked()
    }
    #[wasm_bindgen(js_name = warmupPeriod)]
    pub fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
}

// ============================== Candlestick Patterns ==============================
//
// All 15 patterns take Candles (open, high, low, close) and emit a signed f64
// signal per bar: +1.0 bullish, -1.0 bearish, 0.0 no pattern. Doji is
// direction-less by default (0/+1); pass `signed = true` to its constructor for
// the dragonfly/gravestone signed +-1 encoding.

macro_rules! wasm_candle_pattern {
    ($wasm:ident, $inner:ty, $js:ident) => {
        #[wasm_bindgen(js_name = $js)]
        pub struct $wasm {
            inner: $inner,
        }

        impl Default for $wasm {
            fn default() -> Self {
                Self::new()
            }
        }

        #[wasm_bindgen(js_class = $js)]
        impl $wasm {
            #[wasm_bindgen(constructor)]
            pub fn new() -> $wasm {
                Self {
                    inner: <$inner>::new(),
                }
            }
            pub fn update(
                &mut self,
                open: f64,
                high: f64,
                low: f64,
                close: f64,
            ) -> Result<Option<f64>, JsError> {
                let c = wc::Candle::new(open, high, low, close, 0.0, 0).map_err(map_err)?;
                Ok(self.inner.update(c))
            }
            pub fn batch(
                &mut self,
                open: &[f64],
                high: &[f64],
                low: &[f64],
                close: &[f64],
            ) -> Result<Float64Array, JsError> {
                let n = open.len();
                if high.len() != n || low.len() != n || close.len() != n {
                    return Err(JsError::new("open, high, low, close must be equal length"));
                }
                let mut out = Vec::with_capacity(n);
                for i in 0..n {
                    let c = wc::Candle::new(open[i], high[i], low[i], close[i], 0.0, 0)
                        .map_err(map_err)?;
                    out.push(self.inner.update(c).unwrap_or(f64::NAN));
                }
                Ok(Float64Array::from(out.as_slice()))
            }
            pub fn reset(&mut self) {
                self.inner.reset();
            }
            #[wasm_bindgen(js_name = isReady)]
            pub fn is_ready(&self) -> bool {
                self.inner.is_ready()
            }
            #[wasm_bindgen(js_name = warmupPeriod)]
            pub fn warmup_period(&self) -> usize {
                self.inner.warmup_period()
            }
        }
    };
}

// Doji is the one pattern with an opt-in signed mode, so it is hand-written
// rather than generated by `wasm_candle_pattern!`.
#[wasm_bindgen(js_name = Doji)]
pub struct WasmDoji {
    inner: wc::Doji,
}

impl Default for WasmDoji {
    fn default() -> Self {
        Self::new(None)
    }
}

#[wasm_bindgen(js_class = Doji)]
impl WasmDoji {
    #[wasm_bindgen(constructor)]
    pub fn new(signed: Option<bool>) -> WasmDoji {
        let inner = if signed.unwrap_or(false) {
            wc::Doji::new().signed()
        } else {
            wc::Doji::new()
        };
        Self { inner }
    }
    pub fn update(
        &mut self,
        open: f64,
        high: f64,
        low: f64,
        close: f64,
    ) -> Result<Option<f64>, JsError> {
        let c = wc::Candle::new(open, high, low, close, 0.0, 0).map_err(map_err)?;
        Ok(self.inner.update(c))
    }
    pub fn batch(
        &mut self,
        open: &[f64],
        high: &[f64],
        low: &[f64],
        close: &[f64],
    ) -> Result<Float64Array, JsError> {
        let n = open.len();
        if high.len() != n || low.len() != n || close.len() != n {
            return Err(JsError::new("open, high, low, close must be equal length"));
        }
        let mut out = Vec::with_capacity(n);
        for i in 0..n {
            let c = wc::Candle::new(open[i], high[i], low[i], close[i], 0.0, 0).map_err(map_err)?;
            out.push(self.inner.update(c).unwrap_or(f64::NAN));
        }
        Ok(Float64Array::from(out.as_slice()))
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
    #[wasm_bindgen(js_name = isReady)]
    pub fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    #[wasm_bindgen(js_name = warmupPeriod)]
    pub fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
    #[wasm_bindgen(js_name = isSigned)]
    pub fn is_signed(&self) -> bool {
        self.inner.is_signed()
    }
}

wasm_candle_pattern!(WasmHammer, wc::Hammer, Hammer);
wasm_candle_pattern!(WasmInvertedHammer, wc::InvertedHammer, InvertedHammer);
wasm_candle_pattern!(WasmHangingMan, wc::HangingMan, HangingMan);
wasm_candle_pattern!(WasmShootingStar, wc::ShootingStar, ShootingStar);
wasm_candle_pattern!(WasmEngulfing, wc::Engulfing, Engulfing);
wasm_candle_pattern!(WasmHarami, wc::Harami, Harami);
wasm_candle_pattern!(
    WasmMorningEveningStar,
    wc::MorningEveningStar,
    MorningEveningStar
);
wasm_candle_pattern!(
    WasmThreeSoldiersOrCrows,
    wc::ThreeSoldiersOrCrows,
    ThreeSoldiersOrCrows
);
wasm_candle_pattern!(
    WasmPiercingDarkCloud,
    wc::PiercingDarkCloud,
    PiercingDarkCloud
);
wasm_candle_pattern!(WasmMarubozu, wc::Marubozu, Marubozu);
wasm_candle_pattern!(WasmTweezer, wc::Tweezer, Tweezer);
wasm_candle_pattern!(WasmSpinningTop, wc::SpinningTop, SpinningTop);
wasm_candle_pattern!(WasmThreeInside, wc::ThreeInside, ThreeInside);
wasm_candle_pattern!(WasmThreeOutside, wc::ThreeOutside, ThreeOutside);
wasm_candle_pattern!(WasmTwoCrows, wc::TwoCrows, TwoCrows);
wasm_candle_pattern!(
    WasmUpsideGapTwoCrows,
    wc::UpsideGapTwoCrows,
    UpsideGapTwoCrows
);
wasm_candle_pattern!(
    WasmIdenticalThreeCrows,
    wc::IdenticalThreeCrows,
    IdenticalThreeCrows
);
wasm_candle_pattern!(WasmThreeLineStrike, wc::ThreeLineStrike, ThreeLineStrike);
wasm_candle_pattern!(
    WasmThreeStarsInSouth,
    wc::ThreeStarsInSouth,
    ThreeStarsInSouth
);
wasm_candle_pattern!(WasmAbandonedBaby, wc::AbandonedBaby, AbandonedBaby);
wasm_candle_pattern!(WasmAdvanceBlock, wc::AdvanceBlock, AdvanceBlock);
wasm_candle_pattern!(WasmBeltHold, wc::BeltHold, BeltHold);
wasm_candle_pattern!(WasmBreakaway, wc::Breakaway, Breakaway);
wasm_candle_pattern!(WasmCounterattack, wc::Counterattack, Counterattack);
wasm_candle_pattern!(WasmDojiStar, wc::DojiStar, DojiStar);
wasm_candle_pattern!(WasmDragonflyDoji, wc::DragonflyDoji, DragonflyDoji);
wasm_candle_pattern!(WasmGravestoneDoji, wc::GravestoneDoji, GravestoneDoji);
wasm_candle_pattern!(WasmLongLeggedDoji, wc::LongLeggedDoji, LongLeggedDoji);
wasm_candle_pattern!(WasmRickshawMan, wc::RickshawMan, RickshawMan);
wasm_candle_pattern!(WasmEveningDojiStar, wc::EveningDojiStar, EveningDojiStar);
wasm_candle_pattern!(WasmMorningDojiStar, wc::MorningDojiStar, MorningDojiStar);
wasm_candle_pattern!(
    WasmGapSideBySideWhite,
    wc::GapSideBySideWhite,
    GapSideBySideWhite
);
wasm_candle_pattern!(WasmHighWave, wc::HighWave, HighWave);
wasm_candle_pattern!(WasmHikkake, wc::Hikkake, Hikkake);
wasm_candle_pattern!(WasmHikkakeModified, wc::HikkakeModified, HikkakeModified);
wasm_candle_pattern!(WasmHomingPigeon, wc::HomingPigeon, HomingPigeon);
wasm_candle_pattern!(WasmOnNeck, wc::OnNeck, OnNeck);
wasm_candle_pattern!(WasmInNeck, wc::InNeck, InNeck);
wasm_candle_pattern!(WasmThrusting, wc::Thrusting, Thrusting);
wasm_candle_pattern!(WasmSeparatingLines, wc::SeparatingLines, SeparatingLines);
wasm_candle_pattern!(WasmKicking, wc::Kicking, Kicking);
wasm_candle_pattern!(WasmKickingByLength, wc::KickingByLength, KickingByLength);
wasm_candle_pattern!(WasmLadderBottom, wc::LadderBottom, LadderBottom);
wasm_candle_pattern!(WasmMatHold, wc::MatHold, MatHold);
wasm_candle_pattern!(WasmMatchingLow, wc::MatchingLow, MatchingLow);
wasm_candle_pattern!(WasmLongLine, wc::LongLine, LongLine);
wasm_candle_pattern!(WasmShortLine, wc::ShortLine, ShortLine);
wasm_candle_pattern!(
    WasmRisingThreeMethods,
    wc::RisingThreeMethods,
    RisingThreeMethods
);
wasm_candle_pattern!(
    WasmFallingThreeMethods,
    wc::FallingThreeMethods,
    FallingThreeMethods
);
wasm_candle_pattern!(
    WasmUpsideGapThreeMethods,
    wc::UpsideGapThreeMethods,
    UpsideGapThreeMethods
);
wasm_candle_pattern!(
    WasmDownsideGapThreeMethods,
    wc::DownsideGapThreeMethods,
    DownsideGapThreeMethods
);
wasm_candle_pattern!(WasmStalledPattern, wc::StalledPattern, StalledPattern);
wasm_candle_pattern!(WasmStickSandwich, wc::StickSandwich, StickSandwich);
wasm_candle_pattern!(WasmTakuri, wc::Takuri, Takuri);
wasm_candle_pattern!(WasmClosingMarubozu, wc::ClosingMarubozu, ClosingMarubozu);
wasm_candle_pattern!(WasmOpeningMarubozu, wc::OpeningMarubozu, OpeningMarubozu);
wasm_candle_pattern!(WasmTasukiGap, wc::TasukiGap, TasukiGap);
wasm_candle_pattern!(WasmUniqueThreeRiver, wc::UniqueThreeRiver, UniqueThreeRiver);
wasm_candle_pattern!(
    WasmConcealingBabySwallow,
    wc::ConcealingBabySwallow,
    ConcealingBabySwallow
);

// ============================== Microstructure: Order Book ==============================
//
// Order-book indicators consume a depth snapshot rather than OHLCV. Each
// `update(bidPx, bidSz, askPx, askSz)` takes four equal-length typed arrays for
// one snapshot (bids best-first = descending price, asks best-first = ascending
// price) — the streaming model that fits a live browser book feed. Batch over a
// ragged depth history is provided by the Python and Node bindings.

fn build_order_book(
    bid_px: &[f64],
    bid_sz: &[f64],
    ask_px: &[f64],
    ask_sz: &[f64],
) -> Result<wc::OrderBook, JsError> {
    if bid_px.len() != bid_sz.len() || ask_px.len() != ask_sz.len() {
        return Err(JsError::new(
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

macro_rules! wasm_ob_indicator {
    ($wasm:ident, $inner:ty, $js:ident) => {
        #[wasm_bindgen(js_name = $js)]
        pub struct $wasm {
            inner: $inner,
        }

        impl Default for $wasm {
            fn default() -> Self {
                Self::new()
            }
        }

        #[wasm_bindgen(js_class = $js)]
        impl $wasm {
            #[wasm_bindgen(constructor)]
            pub fn new() -> $wasm {
                Self {
                    inner: <$inner>::new(),
                }
            }
            pub fn update(
                &mut self,
                bid_px: &[f64],
                bid_sz: &[f64],
                ask_px: &[f64],
                ask_sz: &[f64],
            ) -> Result<Option<f64>, JsError> {
                let book = build_order_book(bid_px, bid_sz, ask_px, ask_sz)?;
                Ok(self.inner.update(book))
            }
            pub fn reset(&mut self) {
                self.inner.reset();
            }
            #[wasm_bindgen(js_name = isReady)]
            pub fn is_ready(&self) -> bool {
                self.inner.is_ready()
            }
            #[wasm_bindgen(js_name = warmupPeriod)]
            pub fn warmup_period(&self) -> usize {
                self.inner.warmup_period()
            }
        }
    };
}

wasm_ob_indicator!(
    WasmOrderBookImbalanceTop1,
    wc::OrderBookImbalanceTop1,
    OrderBookImbalanceTop1
);
wasm_ob_indicator!(
    WasmOrderBookImbalanceFull,
    wc::OrderBookImbalanceFull,
    OrderBookImbalanceFull
);
wasm_ob_indicator!(WasmMicroprice, wc::Microprice, Microprice);
wasm_ob_indicator!(WasmQuotedSpread, wc::QuotedSpread, QuotedSpread);
wasm_ob_indicator!(WasmDepthSlope, wc::DepthSlope, DepthSlope);

// Top-N imbalance carries a `levels` parameter, so it is hand-written.
#[wasm_bindgen(js_name = OrderBookImbalanceTopN)]
pub struct WasmOrderBookImbalanceTopN {
    inner: wc::OrderBookImbalanceTopN,
}

#[wasm_bindgen(js_class = OrderBookImbalanceTopN)]
impl WasmOrderBookImbalanceTopN {
    #[wasm_bindgen(constructor)]
    pub fn new(levels: usize) -> Result<WasmOrderBookImbalanceTopN, JsError> {
        Ok(Self {
            inner: wc::OrderBookImbalanceTopN::new(levels).map_err(map_err)?,
        })
    }
    pub fn update(
        &mut self,
        bid_px: &[f64],
        bid_sz: &[f64],
        ask_px: &[f64],
        ask_sz: &[f64],
    ) -> Result<Option<f64>, JsError> {
        let book = build_order_book(bid_px, bid_sz, ask_px, ask_sz)?;
        Ok(self.inner.update(book))
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
    #[wasm_bindgen(js_name = isReady)]
    pub fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    #[wasm_bindgen(js_name = warmupPeriod)]
    pub fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
}

// ============================== Microstructure: Trade Flow ==============================
//
// Trade-flow indicators consume a trade tape rather than OHLCV. Each
// `update(price, size, isBuy)` takes one trade (`isBuy=true` for a
// buyer-initiated trade) — the streaming model for a live browser trade feed.

fn build_trade(price: f64, size: f64, is_buy: bool) -> Result<wc::Trade, JsError> {
    let side = if is_buy {
        wc::Side::Buy
    } else {
        wc::Side::Sell
    };
    wc::Trade::new(price, size, side, 0).map_err(map_err)
}

macro_rules! wasm_trade_indicator {
    ($wasm:ident, $inner:ty, $js:ident) => {
        #[wasm_bindgen(js_name = $js)]
        pub struct $wasm {
            inner: $inner,
        }

        impl Default for $wasm {
            fn default() -> Self {
                Self::new()
            }
        }

        #[wasm_bindgen(js_class = $js)]
        impl $wasm {
            #[wasm_bindgen(constructor)]
            pub fn new() -> $wasm {
                Self {
                    inner: <$inner>::new(),
                }
            }
            pub fn update(
                &mut self,
                price: f64,
                size: f64,
                is_buy: bool,
            ) -> Result<Option<f64>, JsError> {
                Ok(self.inner.update(build_trade(price, size, is_buy)?))
            }
            pub fn reset(&mut self) {
                self.inner.reset();
            }
            #[wasm_bindgen(js_name = isReady)]
            pub fn is_ready(&self) -> bool {
                self.inner.is_ready()
            }
            #[wasm_bindgen(js_name = warmupPeriod)]
            pub fn warmup_period(&self) -> usize {
                self.inner.warmup_period()
            }
        }
    };
}

wasm_trade_indicator!(WasmSignedVolume, wc::SignedVolume, SignedVolume);
wasm_trade_indicator!(
    WasmCumulativeVolumeDelta,
    wc::CumulativeVolumeDelta,
    CumulativeVolumeDelta
);

// Trade imbalance carries a `window` parameter, so it is hand-written.
#[wasm_bindgen(js_name = TradeImbalance)]
pub struct WasmTradeImbalance {
    inner: wc::TradeImbalance,
}

#[wasm_bindgen(js_class = TradeImbalance)]
impl WasmTradeImbalance {
    #[wasm_bindgen(constructor)]
    pub fn new(window: usize) -> Result<WasmTradeImbalance, JsError> {
        Ok(Self {
            inner: wc::TradeImbalance::new(window).map_err(map_err)?,
        })
    }
    pub fn update(&mut self, price: f64, size: f64, is_buy: bool) -> Result<Option<f64>, JsError> {
        Ok(self.inner.update(build_trade(price, size, is_buy)?))
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
    #[wasm_bindgen(js_name = isReady)]
    pub fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    #[wasm_bindgen(js_name = warmupPeriod)]
    pub fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
}

// ============================== Microstructure: Price Impact ==============================
//
// Price-impact indicators consume a trade paired with the mid prevailing at
// execution. Each `update(price, size, isBuy, mid)` takes one such trade-quote
// (`isBuy=true` for a buyer-initiated trade) — the streaming model for a live
// browser trade feed. Batch over a tape is provided by the Python and Node
// bindings.

fn build_trade_quote(
    price: f64,
    size: f64,
    is_buy: bool,
    mid: f64,
) -> Result<wc::TradeQuote, JsError> {
    let trade = build_trade(price, size, is_buy)?;
    wc::TradeQuote::new(trade, mid).map_err(map_err)
}

macro_rules! wasm_trade_quote_indicator {
    ($wasm:ident, $inner:ty, $js:ident) => {
        #[wasm_bindgen(js_name = $js)]
        pub struct $wasm {
            inner: $inner,
        }

        impl Default for $wasm {
            fn default() -> Self {
                Self::new()
            }
        }

        #[wasm_bindgen(js_class = $js)]
        impl $wasm {
            #[wasm_bindgen(constructor)]
            pub fn new() -> $wasm {
                Self {
                    inner: <$inner>::new(),
                }
            }
            pub fn update(
                &mut self,
                price: f64,
                size: f64,
                is_buy: bool,
                mid: f64,
            ) -> Result<Option<f64>, JsError> {
                Ok(self
                    .inner
                    .update(build_trade_quote(price, size, is_buy, mid)?))
            }
            pub fn reset(&mut self) {
                self.inner.reset();
            }
            #[wasm_bindgen(js_name = isReady)]
            pub fn is_ready(&self) -> bool {
                self.inner.is_ready()
            }
            #[wasm_bindgen(js_name = warmupPeriod)]
            pub fn warmup_period(&self) -> usize {
                self.inner.warmup_period()
            }
        }
    };
}

wasm_trade_quote_indicator!(WasmEffectiveSpread, wc::EffectiveSpread, EffectiveSpread);

// Realized spread carries a `horizon` parameter, so it is hand-written.
#[wasm_bindgen(js_name = RealizedSpread)]
pub struct WasmRealizedSpread {
    inner: wc::RealizedSpread,
}

#[wasm_bindgen(js_class = RealizedSpread)]
impl WasmRealizedSpread {
    #[wasm_bindgen(constructor)]
    pub fn new(horizon: usize) -> Result<WasmRealizedSpread, JsError> {
        Ok(Self {
            inner: wc::RealizedSpread::new(horizon).map_err(map_err)?,
        })
    }
    pub fn update(
        &mut self,
        price: f64,
        size: f64,
        is_buy: bool,
        mid: f64,
    ) -> Result<Option<f64>, JsError> {
        Ok(self
            .inner
            .update(build_trade_quote(price, size, is_buy, mid)?))
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
    #[wasm_bindgen(js_name = isReady)]
    pub fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    #[wasm_bindgen(js_name = warmupPeriod)]
    pub fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
}

// Kyle's lambda carries a `window` parameter, so it is hand-written.
#[wasm_bindgen(js_name = KylesLambda)]
pub struct WasmKylesLambda {
    inner: wc::KylesLambda,
}

#[wasm_bindgen(js_class = KylesLambda)]
impl WasmKylesLambda {
    #[wasm_bindgen(constructor)]
    pub fn new(window: usize) -> Result<WasmKylesLambda, JsError> {
        Ok(Self {
            inner: wc::KylesLambda::new(window).map_err(map_err)?,
        })
    }
    pub fn update(
        &mut self,
        price: f64,
        size: f64,
        is_buy: bool,
        mid: f64,
    ) -> Result<Option<f64>, JsError> {
        Ok(self
            .inner
            .update(build_trade_quote(price, size, is_buy, mid)?))
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
    #[wasm_bindgen(js_name = isReady)]
    pub fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    #[wasm_bindgen(js_name = warmupPeriod)]
    pub fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
}

// ============================== Microstructure: Footprint ==============================
//
// Footprint is a multi-output, variable-length indicator. Each `update(price,
// size, isBuy)` returns the full bar footprint accumulated since the last
// `reset()` as an array of `{ price, bidVol, askVol }` objects (sorted ascending
// by price) — the streaming model for a live browser trade feed.

#[wasm_bindgen(js_name = Footprint)]
pub struct WasmFootprint {
    inner: wc::Footprint,
}

#[wasm_bindgen(js_class = Footprint)]
impl WasmFootprint {
    #[wasm_bindgen(constructor)]
    pub fn new(tick_size: f64) -> Result<WasmFootprint, JsError> {
        Ok(Self {
            inner: wc::Footprint::new(tick_size).map_err(map_err)?,
        })
    }
    pub fn update(&mut self, price: f64, size: f64, is_buy: bool) -> Result<JsValue, JsError> {
        let out = self
            .inner
            .update(build_trade(price, size, is_buy)?)
            .expect("footprint emits on every trade");
        let levels = Array::new();
        for level in &out.levels {
            let obj = Object::new();
            Reflect::set(&obj, &"price".into(), &level.price.into()).ok();
            Reflect::set(&obj, &"bidVol".into(), &level.bid_vol.into()).ok();
            Reflect::set(&obj, &"askVol".into(), &level.ask_vol.into()).ok();
            levels.push(&obj);
        }
        Ok(levels.into())
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
    #[wasm_bindgen(js_name = isReady)]
    pub fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    #[wasm_bindgen(js_name = warmupPeriod)]
    pub fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
}

// ============================== Derivatives ==============================
//
// Derivatives indicators consume a perpetual / futures tick rather than OHLCV.
// Each `update(...)` takes only the tick fields its indicator reads — the
// streaming model for a live browser derivatives feed. Batch over a tape is
// provided by the Python and Node bindings. The helpers build a fully-valid
// `DerivativesTick`, filling unused fields with neutral defaults.

fn deriv_funding(funding_rate: f64) -> Result<wc::DerivativesTick, JsError> {
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

fn deriv_basis(mark_price: f64, index_price: f64) -> Result<wc::DerivativesTick, JsError> {
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

fn deriv_oi(open_interest: f64) -> Result<wc::DerivativesTick, JsError> {
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

#[wasm_bindgen(js_name = FundingRate)]
pub struct WasmFundingRate {
    inner: wc::FundingRate,
}

impl Default for WasmFundingRate {
    fn default() -> Self {
        Self::new()
    }
}

#[wasm_bindgen(js_class = FundingRate)]
impl WasmFundingRate {
    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmFundingRate {
        Self {
            inner: wc::FundingRate::new(),
        }
    }
    pub fn update(&mut self, funding_rate: f64) -> Result<Option<f64>, JsError> {
        Ok(self.inner.update(deriv_funding(funding_rate)?))
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
    #[wasm_bindgen(js_name = isReady)]
    pub fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    #[wasm_bindgen(js_name = warmupPeriod)]
    pub fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
}

#[wasm_bindgen(js_name = FundingRateMean)]
pub struct WasmFundingRateMean {
    inner: wc::FundingRateMean,
}

#[wasm_bindgen(js_class = FundingRateMean)]
impl WasmFundingRateMean {
    #[wasm_bindgen(constructor)]
    pub fn new(window: usize) -> Result<WasmFundingRateMean, JsError> {
        Ok(Self {
            inner: wc::FundingRateMean::new(window).map_err(map_err)?,
        })
    }
    pub fn update(&mut self, funding_rate: f64) -> Result<Option<f64>, JsError> {
        Ok(self.inner.update(deriv_funding(funding_rate)?))
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
    #[wasm_bindgen(js_name = isReady)]
    pub fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    #[wasm_bindgen(js_name = warmupPeriod)]
    pub fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
}

#[wasm_bindgen(js_name = FundingRateZScore)]
pub struct WasmFundingRateZScore {
    inner: wc::FundingRateZScore,
}

#[wasm_bindgen(js_class = FundingRateZScore)]
impl WasmFundingRateZScore {
    #[wasm_bindgen(constructor)]
    pub fn new(window: usize) -> Result<WasmFundingRateZScore, JsError> {
        Ok(Self {
            inner: wc::FundingRateZScore::new(window).map_err(map_err)?,
        })
    }
    pub fn update(&mut self, funding_rate: f64) -> Result<Option<f64>, JsError> {
        Ok(self.inner.update(deriv_funding(funding_rate)?))
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
    #[wasm_bindgen(js_name = isReady)]
    pub fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    #[wasm_bindgen(js_name = warmupPeriod)]
    pub fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
}

#[wasm_bindgen(js_name = FundingBasis)]
pub struct WasmFundingBasis {
    inner: wc::FundingBasis,
}

impl Default for WasmFundingBasis {
    fn default() -> Self {
        Self::new()
    }
}

#[wasm_bindgen(js_class = FundingBasis)]
impl WasmFundingBasis {
    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmFundingBasis {
        Self {
            inner: wc::FundingBasis::new(),
        }
    }
    pub fn update(&mut self, mark_price: f64, index_price: f64) -> Result<Option<f64>, JsError> {
        Ok(self.inner.update(deriv_basis(mark_price, index_price)?))
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
    #[wasm_bindgen(js_name = isReady)]
    pub fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    #[wasm_bindgen(js_name = warmupPeriod)]
    pub fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
}

#[wasm_bindgen(js_name = OpenInterestDelta)]
pub struct WasmOpenInterestDelta {
    inner: wc::OpenInterestDelta,
}

impl Default for WasmOpenInterestDelta {
    fn default() -> Self {
        Self::new()
    }
}

#[wasm_bindgen(js_class = OpenInterestDelta)]
impl WasmOpenInterestDelta {
    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmOpenInterestDelta {
        Self {
            inner: wc::OpenInterestDelta::new(),
        }
    }
    pub fn update(&mut self, open_interest: f64) -> Result<Option<f64>, JsError> {
        Ok(self.inner.update(deriv_oi(open_interest)?))
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
    #[wasm_bindgen(js_name = isReady)]
    pub fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    #[wasm_bindgen(js_name = warmupPeriod)]
    pub fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
}

fn deriv_oi_mark(open_interest: f64, mark_price: f64) -> Result<wc::DerivativesTick, JsError> {
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

fn deriv_long_short(long_size: f64, short_size: f64) -> Result<wc::DerivativesTick, JsError> {
    wc::DerivativesTick::new(
        0.0, 1.0, 1.0, 1.0, 0.0, long_size, short_size, 0.0, 0.0, 0.0, 0.0, 0,
    )
    .map_err(map_err)
}

fn deriv_taker(
    taker_buy_volume: f64,
    taker_sell_volume: f64,
) -> Result<wc::DerivativesTick, JsError> {
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
) -> Result<wc::DerivativesTick, JsError> {
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

#[wasm_bindgen(js_name = OIPriceDivergence)]
pub struct WasmOIPriceDivergence {
    inner: wc::OIPriceDivergence,
}

#[wasm_bindgen(js_class = OIPriceDivergence)]
impl WasmOIPriceDivergence {
    #[wasm_bindgen(constructor)]
    pub fn new(window: usize) -> Result<WasmOIPriceDivergence, JsError> {
        Ok(Self {
            inner: wc::OIPriceDivergence::new(window).map_err(map_err)?,
        })
    }
    pub fn update(&mut self, open_interest: f64, mark_price: f64) -> Result<Option<f64>, JsError> {
        Ok(self.inner.update(deriv_oi_mark(open_interest, mark_price)?))
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
    #[wasm_bindgen(js_name = isReady)]
    pub fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    #[wasm_bindgen(js_name = warmupPeriod)]
    pub fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
}

#[wasm_bindgen(js_name = OIWeighted)]
pub struct WasmOIWeighted {
    inner: wc::OIWeighted,
}

impl Default for WasmOIWeighted {
    fn default() -> Self {
        Self::new()
    }
}

#[wasm_bindgen(js_class = OIWeighted)]
impl WasmOIWeighted {
    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmOIWeighted {
        Self {
            inner: wc::OIWeighted::new(),
        }
    }
    pub fn update(&mut self, mark_price: f64, open_interest: f64) -> Result<Option<f64>, JsError> {
        Ok(self.inner.update(deriv_oi_mark(open_interest, mark_price)?))
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
    #[wasm_bindgen(js_name = isReady)]
    pub fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    #[wasm_bindgen(js_name = warmupPeriod)]
    pub fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
}

#[wasm_bindgen(js_name = LongShortRatio)]
pub struct WasmLongShortRatio {
    inner: wc::LongShortRatio,
}

impl Default for WasmLongShortRatio {
    fn default() -> Self {
        Self::new()
    }
}

#[wasm_bindgen(js_class = LongShortRatio)]
impl WasmLongShortRatio {
    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmLongShortRatio {
        Self {
            inner: wc::LongShortRatio::new(),
        }
    }
    pub fn update(&mut self, long_size: f64, short_size: f64) -> Result<Option<f64>, JsError> {
        Ok(self.inner.update(deriv_long_short(long_size, short_size)?))
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
    #[wasm_bindgen(js_name = isReady)]
    pub fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    #[wasm_bindgen(js_name = warmupPeriod)]
    pub fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
}

#[wasm_bindgen(js_name = TakerBuySellRatio)]
pub struct WasmTakerBuySellRatio {
    inner: wc::TakerBuySellRatio,
}

impl Default for WasmTakerBuySellRatio {
    fn default() -> Self {
        Self::new()
    }
}

#[wasm_bindgen(js_class = TakerBuySellRatio)]
impl WasmTakerBuySellRatio {
    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmTakerBuySellRatio {
        Self {
            inner: wc::TakerBuySellRatio::new(),
        }
    }
    pub fn update(
        &mut self,
        taker_buy_volume: f64,
        taker_sell_volume: f64,
    ) -> Result<Option<f64>, JsError> {
        Ok(self
            .inner
            .update(deriv_taker(taker_buy_volume, taker_sell_volume)?))
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
    #[wasm_bindgen(js_name = isReady)]
    pub fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    #[wasm_bindgen(js_name = warmupPeriod)]
    pub fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
}

#[wasm_bindgen(js_name = LiquidationFeatures)]
pub struct WasmLiquidationFeatures {
    inner: wc::LiquidationFeatures,
}

impl Default for WasmLiquidationFeatures {
    fn default() -> Self {
        Self::new()
    }
}

#[wasm_bindgen(js_class = LiquidationFeatures)]
impl WasmLiquidationFeatures {
    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmLiquidationFeatures {
        Self {
            inner: wc::LiquidationFeatures::new(),
        }
    }
    pub fn update(
        &mut self,
        long_liquidation: f64,
        short_liquidation: f64,
    ) -> Result<JsValue, JsError> {
        let out = self
            .inner
            .update(deriv_liquidation(long_liquidation, short_liquidation)?)
            .expect("liquidation features emit on every tick");
        let obj = Object::new();
        Reflect::set(&obj, &"long".into(), &out.long.into()).ok();
        Reflect::set(&obj, &"short".into(), &out.short.into()).ok();
        Reflect::set(&obj, &"net".into(), &out.net.into()).ok();
        Reflect::set(&obj, &"total".into(), &out.total.into()).ok();
        Reflect::set(&obj, &"imbalance".into(), &out.imbalance.into()).ok();
        Ok(obj.into())
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
    #[wasm_bindgen(js_name = isReady)]
    pub fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    #[wasm_bindgen(js_name = warmupPeriod)]
    pub fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
}

fn deriv_futures_index(
    futures_price: f64,
    index_price: f64,
) -> Result<wc::DerivativesTick, JsError> {
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

fn deriv_futures_mark(futures_price: f64, mark_price: f64) -> Result<wc::DerivativesTick, JsError> {
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

#[wasm_bindgen(js_name = TermStructureBasis)]
pub struct WasmTermStructureBasis {
    inner: wc::TermStructureBasis,
}

impl Default for WasmTermStructureBasis {
    fn default() -> Self {
        Self::new()
    }
}

#[wasm_bindgen(js_class = TermStructureBasis)]
impl WasmTermStructureBasis {
    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmTermStructureBasis {
        Self {
            inner: wc::TermStructureBasis::new(),
        }
    }
    pub fn update(&mut self, futures_price: f64, index_price: f64) -> Result<Option<f64>, JsError> {
        Ok(self
            .inner
            .update(deriv_futures_index(futures_price, index_price)?))
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
    #[wasm_bindgen(js_name = isReady)]
    pub fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    #[wasm_bindgen(js_name = warmupPeriod)]
    pub fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
}

#[wasm_bindgen(js_name = CalendarSpread)]
pub struct WasmCalendarSpread {
    inner: wc::CalendarSpread,
}

impl Default for WasmCalendarSpread {
    fn default() -> Self {
        Self::new()
    }
}

#[wasm_bindgen(js_class = CalendarSpread)]
impl WasmCalendarSpread {
    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmCalendarSpread {
        Self {
            inner: wc::CalendarSpread::new(),
        }
    }
    pub fn update(&mut self, futures_price: f64, mark_price: f64) -> Result<Option<f64>, JsError> {
        Ok(self
            .inner
            .update(deriv_futures_mark(futures_price, mark_price)?))
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
    #[wasm_bindgen(js_name = isReady)]
    pub fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    #[wasm_bindgen(js_name = warmupPeriod)]
    pub fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::wasm_bindgen_test;

    #[wasm_bindgen_test]
    fn sma_batch_reference_values() {
        // SMA(3) of [2, 4, 6, 8, 10] -> [NaN, NaN, 4, 6, 8].
        let mut sma = WasmSma::new(3).expect("valid period");
        let out = sma.batch(&[2.0, 4.0, 6.0, 8.0, 10.0]);
        assert!(out.get_index(0).is_nan());
        assert!(out.get_index(1).is_nan());
        assert_eq!(out.get_index(2), 4.0);
        assert_eq!(out.get_index(3), 6.0);
        assert_eq!(out.get_index(4), 8.0);
    }

    #[wasm_bindgen_test]
    fn ema_batch_equals_streaming() {
        let prices: Vec<f64> = (1..=60)
            .map(|i| 100.0 + (f64::from(i) * 0.3).sin() * 5.0)
            .collect();
        let batch = WasmEma::new(14).expect("valid").batch(&prices);
        let mut ema = WasmEma::new(14).expect("valid");
        for (i, &p) in prices.iter().enumerate() {
            let b = batch.get_index(i as u32);
            match ema.update(p) {
                Some(v) => assert!((v - b).abs() < 1e-9, "streaming != batch at {i}"),
                None => assert!(b.is_nan(), "expected NaN during warmup at {i}"),
            }
        }
    }

    #[wasm_bindgen_test]
    fn rsi_pure_uptrend_yields_100() {
        let prices: Vec<f64> = (1..=20).map(f64::from).collect();
        let out = WasmRsi::new(14).expect("valid").batch(&prices);
        for i in 14..prices.len() {
            assert_eq!(out.get_index(i as u32), 100.0);
        }
    }

    #[wasm_bindgen_test]
    fn invalid_constructors_return_err() {
        assert!(WasmSma::new(0).is_err());
        assert!(WasmMacd::new(0, 0, 0).is_err());
        assert!(WasmMacd::new(26, 12, 9).is_err());
        assert!(WasmBb::new(20, -1.0).is_err());
        assert!(WasmPsar::new(0.30, 0.02, 0.20).is_err());
    }

    #[wasm_bindgen_test]
    fn batch_rejects_unequal_lengths() {
        let mut atr = WasmAtr::new(14).expect("valid");
        assert!(atr.batch(&[1.0, 2.0], &[1.0], &[1.0, 2.0]).is_err());
        let mut stoch = WasmStoch::new(14, 3).expect("valid");
        assert!(stoch
            .batch(&[1.0, 2.0, 3.0], &[1.0], &[1.0, 2.0, 3.0])
            .is_err());
        let mut mfi = WasmMfi::new(14).expect("valid");
        assert!(mfi
            .batch(&[1.0, 2.0], &[1.0, 2.0], &[1.0, 2.0], &[1.0])
            .is_err());
    }

    // ---------- Streaming-API coverage for the candle-input indicators
    // (Adx, WilliamsR, Cci, Mfi, Psar, Keltner, Donchian, Vwap, AwesomeOscillator,
    //  Aroon, Stochastic, Obv). Verifies that the per-tick `update` produces
    // exactly the same per-row values as `batch` and that the lifecycle methods
    // (`reset`, `isReady`, `warmupPeriod`) honour the same contract as Python /
    // Node and as the other already-wired WASM classes.

    /// Deterministic OHLCV stream long enough to clear every indicator's warmup
    /// (the longest in this group is `Adx(14)` at `2*period = 28` bars).
    fn synthetic_ohlcv(n: usize) -> (Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>) {
        let mut high = Vec::with_capacity(n);
        let mut low = Vec::with_capacity(n);
        let mut close = Vec::with_capacity(n);
        let mut volume = Vec::with_capacity(n);
        for i in 0..n {
            let t = i as f64;
            let mid = 100.0 + (t * 0.17).sin() * 5.0 + t * 0.05;
            let spread = 1.0 + (t * 0.31).cos().abs();
            high.push(mid + spread);
            low.push(mid - spread);
            close.push(mid + (t * 0.07).sin() * 0.5);
            volume.push(1_000.0 + (t * 0.21).sin().abs() * 500.0);
        }
        (high, low, close, volume)
    }

    fn close_enough(a: f64, b: f64) -> bool {
        if a.is_nan() {
            b.is_nan()
        } else if a == b {
            // Exact equality, including matching infinities (e.g. ProfitFactor
            // with no losing trades is +inf in both the streaming and batch
            // passes). `(inf - inf).abs()` is NaN, so the tolerance check below
            // would otherwise reject two equal infinities.
            true
        } else {
            (a - b).abs() < 1e-9
        }
    }

    // Single test deliberately exercises every newly wired class so the lifecycle
    // contract (`update == batch` + `is_ready` + `warmup_period` + `reset`) lives
    // in one auditable place. `h`/`l`/`c`/`v` mirror the OHLCV column names used
    // by the production API and the rest of the test suite.
    #[allow(clippy::too_many_lines, clippy::many_single_char_names)]
    #[wasm_bindgen_test]
    fn candle_input_streaming_matches_batch_and_lifecycle_is_consistent() {
        let (h, l, c, v) = synthetic_ohlcv(40);

        // --- ADX (multi: { plusDi, minusDi, adx }) ---
        let batch = WasmAdx::new(7)
            .expect("valid")
            .batch(&h, &l, &c)
            .expect("valid");
        let mut adx = WasmAdx::new(7).expect("valid");
        assert!(!adx.is_ready());
        assert_eq!(
            adx.warmup_period(),
            wc::Adx::new(7).expect("valid").warmup_period()
        );
        for i in 0..h.len() {
            let stream = adx.update(h[i], l[i], c[i]).expect("valid");
            let bp = batch.get_index((i * 3) as u32);
            let bm = batch.get_index((i * 3 + 1) as u32);
            let ba = batch.get_index((i * 3 + 2) as u32);
            if stream.is_null() {
                assert!(bp.is_nan() && bm.is_nan() && ba.is_nan(), "row {i}");
            } else {
                let obj: &Object = stream.unchecked_ref();
                let p = Reflect::get(obj, &"plusDi".into())
                    .unwrap()
                    .as_f64()
                    .unwrap();
                let m = Reflect::get(obj, &"minusDi".into())
                    .unwrap()
                    .as_f64()
                    .unwrap();
                let a = Reflect::get(obj, &"adx".into()).unwrap().as_f64().unwrap();
                assert!(close_enough(p, bp), "ADX plusDi diverges at {i}");
                assert!(close_enough(m, bm), "ADX minusDi diverges at {i}");
                assert!(close_enough(a, ba), "ADX value diverges at {i}");
            }
        }
        assert!(adx.is_ready());
        adx.reset();
        assert!(!adx.is_ready());

        // --- WilliamsR (single) ---
        let batch = WasmWilliamsR::new(14)
            .expect("valid")
            .batch(&h, &l, &c)
            .expect("valid");
        let mut wr = WasmWilliamsR::new(14).expect("valid");
        for i in 0..h.len() {
            let s = wr
                .update(h[i], l[i], c[i])
                .expect("valid")
                .unwrap_or(f64::NAN);
            assert!(
                close_enough(s, batch.get_index(i as u32)),
                "WilliamsR at {i}"
            );
        }
        assert!(wr.is_ready());
        assert_eq!(
            wr.warmup_period(),
            wc::WilliamsR::new(14).expect("valid").warmup_period()
        );
        wr.reset();
        assert!(!wr.is_ready());

        // --- CCI (single) ---
        let batch = WasmCci::new(14)
            .expect("valid")
            .batch(&h, &l, &c)
            .expect("valid");
        let mut cci = WasmCci::new(14).expect("valid");
        for i in 0..h.len() {
            let s = cci
                .update(h[i], l[i], c[i])
                .expect("valid")
                .unwrap_or(f64::NAN);
            assert!(close_enough(s, batch.get_index(i as u32)), "CCI at {i}");
        }

        // --- MFI (single, with volume) ---
        let batch = WasmMfi::new(14)
            .expect("valid")
            .batch(&h, &l, &c, &v)
            .expect("valid");
        let mut mfi = WasmMfi::new(14).expect("valid");
        for i in 0..h.len() {
            let s = mfi
                .update(h[i], l[i], c[i], v[i])
                .expect("valid")
                .unwrap_or(f64::NAN);
            assert!(close_enough(s, batch.get_index(i as u32)), "MFI at {i}");
        }

        // --- PSAR (single) ---
        let batch = WasmPsar::new(0.02, 0.02, 0.2)
            .expect("valid")
            .batch(&h, &l, &c)
            .expect("valid");
        let mut psar = WasmPsar::new(0.02, 0.02, 0.2).expect("valid");
        for i in 0..h.len() {
            let s = psar
                .update(h[i], l[i], c[i])
                .expect("valid")
                .unwrap_or(f64::NAN);
            assert!(close_enough(s, batch.get_index(i as u32)), "PSAR at {i}");
        }

        // --- Keltner (multi: { upper, middle, lower }) ---
        let batch = WasmKeltner::new(10, 10, 2.0)
            .expect("valid")
            .batch(&h, &l, &c)
            .expect("valid");
        let mut kc = WasmKeltner::new(10, 10, 2.0).expect("valid");
        for i in 0..h.len() {
            let stream = kc.update(h[i], l[i], c[i]).expect("valid");
            let bu = batch.get_index((i * 3) as u32);
            let bm = batch.get_index((i * 3 + 1) as u32);
            let bl = batch.get_index((i * 3 + 2) as u32);
            if stream.is_null() {
                assert!(bu.is_nan() && bm.is_nan() && bl.is_nan(), "Keltner row {i}");
            } else {
                let obj: &Object = stream.unchecked_ref();
                let u = Reflect::get(obj, &"upper".into())
                    .unwrap()
                    .as_f64()
                    .unwrap();
                let m = Reflect::get(obj, &"middle".into())
                    .unwrap()
                    .as_f64()
                    .unwrap();
                let lo = Reflect::get(obj, &"lower".into())
                    .unwrap()
                    .as_f64()
                    .unwrap();
                assert!(close_enough(u, bu), "Keltner upper at {i}");
                assert!(close_enough(m, bm), "Keltner middle at {i}");
                assert!(close_enough(lo, bl), "Keltner lower at {i}");
            }
        }

        // --- Donchian (multi: { upper, middle, lower }) ---
        let batch = WasmDonchian::new(10)
            .expect("valid")
            .batch(&h, &l)
            .expect("valid");
        let mut dc = WasmDonchian::new(10).expect("valid");
        for i in 0..h.len() {
            let stream = dc.update(h[i], l[i]).expect("valid");
            let bu = batch.get_index((i * 3) as u32);
            let bm = batch.get_index((i * 3 + 1) as u32);
            let bl = batch.get_index((i * 3 + 2) as u32);
            if stream.is_null() {
                assert!(
                    bu.is_nan() && bm.is_nan() && bl.is_nan(),
                    "Donchian row {i}"
                );
            } else {
                let obj: &Object = stream.unchecked_ref();
                let u = Reflect::get(obj, &"upper".into())
                    .unwrap()
                    .as_f64()
                    .unwrap();
                let m = Reflect::get(obj, &"middle".into())
                    .unwrap()
                    .as_f64()
                    .unwrap();
                let lo = Reflect::get(obj, &"lower".into())
                    .unwrap()
                    .as_f64()
                    .unwrap();
                assert!(close_enough(u, bu), "Donchian upper at {i}");
                assert!(close_enough(m, bm), "Donchian middle at {i}");
                assert!(close_enough(lo, bl), "Donchian lower at {i}");
            }
        }

        // --- VWAP (single, cumulative, with volume) ---
        let batch = WasmVwap::new().batch(&h, &l, &c, &v).expect("valid");
        let mut vwap = WasmVwap::new();
        for i in 0..h.len() {
            let s = vwap
                .update(h[i], l[i], c[i], v[i])
                .expect("valid")
                .unwrap_or(f64::NAN);
            assert!(close_enough(s, batch.get_index(i as u32)), "VWAP at {i}");
        }

        // --- AwesomeOscillator (single) ---
        let batch = WasmAo::new(5, 34)
            .expect("valid")
            .batch(&h, &l)
            .expect("valid");
        let mut ao = WasmAo::new(5, 34).expect("valid");
        for i in 0..h.len() {
            let s = ao.update(h[i], l[i]).expect("valid").unwrap_or(f64::NAN);
            assert!(close_enough(s, batch.get_index(i as u32)), "AO at {i}");
        }

        // --- Aroon (multi: { up, down }) ---
        let batch = WasmAroon::new(14)
            .expect("valid")
            .batch(&h, &l)
            .expect("valid");
        let mut aroon = WasmAroon::new(14).expect("valid");
        for i in 0..h.len() {
            let stream = aroon.update(h[i], l[i]).expect("valid");
            let bu = batch.get_index((i * 2) as u32);
            let bd = batch.get_index((i * 2 + 1) as u32);
            if stream.is_null() {
                assert!(bu.is_nan() && bd.is_nan(), "Aroon row {i}");
            } else {
                let obj: &Object = stream.unchecked_ref();
                let u = Reflect::get(obj, &"up".into()).unwrap().as_f64().unwrap();
                let d = Reflect::get(obj, &"down".into()).unwrap().as_f64().unwrap();
                assert!(close_enough(u, bu), "Aroon up at {i}");
                assert!(close_enough(d, bd), "Aroon down at {i}");
            }
        }

        // --- Stochastic (multi: { k, d }) ---
        let batch = WasmStoch::new(14, 3)
            .expect("valid")
            .batch(&h, &l, &c)
            .expect("valid");
        let mut st = WasmStoch::new(14, 3).expect("valid");
        for i in 0..h.len() {
            let stream = st.update(h[i], l[i], c[i]).expect("valid");
            let bk = batch.get_index((i * 2) as u32);
            let bd = batch.get_index((i * 2 + 1) as u32);
            if stream.is_null() {
                assert!(bk.is_nan() && bd.is_nan(), "Stoch row {i}");
            } else {
                let obj: &Object = stream.unchecked_ref();
                let k = Reflect::get(obj, &"k".into()).unwrap().as_f64().unwrap();
                let d = Reflect::get(obj, &"d".into()).unwrap().as_f64().unwrap();
                assert!(close_enough(k, bk), "Stoch k at {i}");
                assert!(close_enough(d, bd), "Stoch d at {i}");
            }
        }

        // --- OBV (single, with volume; no warmup) ---
        let batch = WasmObv::new().batch(&c, &v).expect("valid");
        let mut obv = WasmObv::new();
        for i in 0..c.len() {
            let s = obv.update(c[i], v[i]).expect("valid").unwrap_or(f64::NAN);
            assert!(close_enough(s, batch.get_index(i as u32)), "OBV at {i}");
        }
    }

    #[allow(clippy::many_single_char_names)]
    #[wasm_bindgen_test]
    fn rolling_vwap_streaming_matches_batch_and_lifecycle() {
        // R4: RollingVwap is now exposed in WASM (previously Rust-only despite
        // the README listing it as a cross-language indicator).
        let (h, l, c, v) = synthetic_ohlcv(50);
        let batch = WasmRollingVwap::new(10)
            .expect("valid")
            .batch(&h, &l, &c, &v)
            .expect("valid");
        let mut rv = WasmRollingVwap::new(10).expect("valid");
        assert_eq!(rv.warmup_period(), 10);
        assert_eq!(rv.period(), 10);
        assert!(!rv.is_ready());
        for i in 0..h.len() {
            let s = rv
                .update(h[i], l[i], c[i], v[i])
                .expect("valid")
                .unwrap_or(f64::NAN);
            assert!(
                close_enough(s, batch.get_index(i as u32)),
                "RollingVWAP at {i}"
            );
        }
        assert!(rv.is_ready());
        rv.reset();
        assert!(!rv.is_ready());
        assert!(WasmRollingVwap::new(0).is_err());
    }

    #[wasm_bindgen_test]
    fn kama_exposes_warmup_period() {
        // R8: the KAMA wrapper was missing `warmupPeriod`; it now matches the
        // core indicator's value (which is `er_period + 1`).
        let kama = WasmKama::new(10, 2, 30).expect("valid");
        assert_eq!(
            kama.warmup_period(),
            wc::Kama::new(10, 2, 30).expect("valid").warmup_period()
        );
    }

    // === Extended coverage: one wasm_bindgen_test per family ===========
    //
    // The bulk lifecycle test above proves the candle-input contract on a
    // representative set; the cases below add a per-family spot-check so a
    // regression that only manifests for a specific indicator (output shape,
    // multi-output unwrap, etc.) cannot slip through.

    #[wasm_bindgen_test]
    fn macd_multi_output_batch_shape() {
        // Family 4 — Price Oscillators. Macd emits a {macd, signal, histogram}
        // record; batch returns a Float64Array packing all three components
        // back-to-back. Length must be 3 * series_len.
        let prices: Vec<f64> = (0..60).map(|i| 100.0 + f64::from(i) * 0.1).collect();
        let batch = WasmMacd::new(12, 26, 9).expect("valid").batch(&prices);
        assert_eq!(batch.length() as usize, 3 * prices.len());
    }

    #[wasm_bindgen_test]
    fn bollinger_batch_orders_upper_mid_lower() {
        // Family 5 — Bollinger emits {upper, middle, lower} per bar. On any
        // ready output, upper >= middle >= lower must hold.
        let prices: Vec<f64> = (0..60)
            .map(|i| 100.0 + (f64::from(i) * 0.3).sin() * 5.0)
            .collect();
        let batch = WasmBb::new(20, 2.0).expect("valid").batch(&prices);
        // Batch layout is `[u0, m0, l0, sd0, u1, m1, l1, sd1, ...]` — four
        // floats per bar (upper / middle / lower / stddev). First ready bar
        // with period=20 is index 19.
        let start = 19 * 4;
        let upper = batch.get_index(start as u32);
        let middle = batch.get_index((start + 1) as u32);
        let lower = batch.get_index((start + 2) as u32);
        assert!(upper.is_finite() && middle.is_finite() && lower.is_finite());
        assert!(
            upper >= middle,
            "upper {upper} should be >= middle {middle}"
        );
        assert!(
            middle >= lower,
            "middle {middle} should be >= lower {lower}"
        );
    }

    #[wasm_bindgen_test]
    fn fisher_transform_streaming_roundtrip() {
        // Family 10 — Ehlers / Cycle. FisherTransform compresses bounded inputs
        // and is highly recursive — verify streaming runs cleanly across a sine
        // wave without panic or NaN explosion after warmup.
        let prices: Vec<f64> = (0..100)
            .map(|i| 100.0 + (f64::from(i) * 0.2).sin() * 5.0)
            .collect();
        let mut ind = WasmFisherTransform::new(10).expect("valid");
        let mut produced = 0usize;
        for &p in &prices {
            if let Some(v) = ind.update(p) {
                assert!(v.is_finite(), "FisherTransform produced non-finite output");
                produced += 1;
            }
        }
        assert!(
            produced > 50,
            "FisherTransform should emit on most bars after warmup"
        );
    }

    #[wasm_bindgen_test]
    fn sharpe_ratio_reset_clears_state() {
        // Family 16 — Risk / Performance. Confirms reset semantics survive the
        // wasm-bindgen boundary on a metric whose state is the rolling-return
        // window plus running sum / sum-of-squares.
        let mut sr = WasmSharpeRatio::new(10, 0.0).expect("valid");
        for i in 0..15 {
            sr.update(f64::from(i) * 0.001);
        }
        assert!(sr.is_ready(), "SharpeRatio should be ready after warmup");
        sr.reset();
        assert!(
            !sr.is_ready(),
            "SharpeRatio should not be ready after reset"
        );
        assert!(
            sr.update(0.001).is_none(),
            "post-reset update should warmup again"
        );
    }

    #[wasm_bindgen_test]
    fn max_drawdown_monotone_uptrend_yields_zero_after_warmup() {
        // Family 16 — Risk / Performance. Strictly-increasing equity curve has
        // zero drawdown; the bounded-window MaxDrawdown should reflect that.
        let mut md = WasmMaxDrawdown::new(10).expect("valid");
        let mut last = None;
        for i in 1..=20 {
            last = md.update(f64::from(i));
        }
        let final_dd = last.expect("ready after 20 inputs");
        assert!(
            (final_dd).abs() < 1e-12,
            "monotone uptrend should yield max-drawdown == 0, got {final_dd}"
        );
    }

    #[wasm_bindgen_test]
    fn value_at_risk_constructor_rejects_invalid() {
        // Family 16 — Risk / Performance. VaR's confidence must be in (0, 1)
        // and period >= 2; the constructor must surface those as JsError
        // across the binding boundary.
        assert!(
            WasmValueAtRisk::new(1, 0.95).is_err(),
            "period < 2 should reject"
        );
        assert!(
            WasmValueAtRisk::new(20, 0.0).is_err(),
            "confidence == 0 should reject"
        );
        assert!(
            WasmValueAtRisk::new(20, 1.0).is_err(),
            "confidence == 1 should reject"
        );
        assert!(
            WasmValueAtRisk::new(20, 0.95).is_ok(),
            "valid params should construct"
        );
    }

    #[wasm_bindgen_test]
    fn reset_returns_indicator_to_warmup() {
        // Sanity across two macro-generated scalar indicators. Each must
        // report `is_ready() == false` immediately after `reset()`. The
        // hand-coded candle wrappers (ATR, Stoch, etc.) do not expose
        // `is_ready` on the JS surface; their lifecycle is covered by
        // the bulk `candle_input_streaming_matches_batch_and_lifecycle`
        // test above instead.
        let mut sma = WasmSma::new(5).expect("valid");
        for i in 1..=10 {
            sma.update(f64::from(i));
        }
        assert!(sma.is_ready());
        sma.reset();
        assert!(!sma.is_ready());

        let mut ema = WasmEma::new(14).expect("valid");
        for i in 1..=20 {
            ema.update(f64::from(i));
        }
        assert!(ema.is_ready());
        ema.reset();
        assert!(!ema.is_ready());
    }

    #[wasm_bindgen_test]
    fn warmup_period_matches_core_for_baseline_scalar() {
        // The binding must report the same warmup as the underlying wickra-core
        // indicator. Drift here would silently break "wait for first non-NaN"
        // user code.
        let sma = WasmSma::new(20).expect("valid");
        assert_eq!(
            sma.warmup_period(),
            wc::Sma::new(20).expect("valid").warmup_period()
        );
        let ema = WasmEma::new(14).expect("valid");
        assert_eq!(
            ema.warmup_period(),
            wc::Ema::new(14).expect("valid").warmup_period()
        );
        let rsi = WasmRsi::new(14).expect("valid");
        assert_eq!(
            rsi.warmup_period(),
            wc::Rsi::new(14).expect("valid").warmup_period()
        );
    }

    #[wasm_bindgen_test]
    fn nan_input_is_rejected_without_state_mutation() {
        // Non-finite input must be ignored — calling update with NaN must not
        // advance the warmup counter, otherwise streaming code that fed in a
        // spurious NaN would prematurely flip to ready.
        let mut sma = WasmSma::new(5).expect("valid");
        for _ in 0..4 {
            sma.update(f64::NAN);
        }
        assert!(!sma.is_ready(), "NaN inputs must not advance warmup");
        for i in 1..=5 {
            sma.update(f64::from(i));
        }
        assert!(
            sma.is_ready(),
            "ready after 5 finite inputs even with prior NaNs"
        );
    }

    // Streaming `update` must reproduce `batch` value-for-value for every scalar
    // indicator — the core O(1) state-machine invariant. Each entry builds a
    // fresh instance for the batch pass and another for the streaming pass. The
    // constructor arguments mirror the (CI-passing) Node `indicators.test.js`
    // factories, so they are known-valid.
    macro_rules! assert_scalar_stream_eq {
        ($ctor:expr, $prices:expr) => {{
            let prices: &[f64] = $prices;
            let batch = { $ctor }.batch(prices);
            let mut streaming = { $ctor };
            for (i, &p) in prices.iter().enumerate() {
                let b = batch.get_index(i as u32);
                match streaming.update(p) {
                    Some(v) => assert!(
                        close_enough(v, b),
                        "{} streaming != batch at {i}: {v} vs {b}",
                        stringify!($ctor)
                    ),
                    None => assert!(
                        b.is_nan(),
                        "{} expected NaN warmup at {i}",
                        stringify!($ctor)
                    ),
                }
            }
        }};
    }

    #[allow(clippy::too_many_lines)]
    #[wasm_bindgen_test]
    fn scalar_streaming_matches_batch_broad() {
        let prices: Vec<f64> = (0..120)
            .map(|i| {
                let t = f64::from(i);
                100.0 + (t * 0.2).sin() * 10.0 + t * 0.1
            })
            .collect();
        let p = prices.as_slice();

        // Moving averages.
        assert_scalar_stream_eq!(WasmSma::new(14).expect("valid"), p);
        assert_scalar_stream_eq!(WasmWma::new(14).expect("valid"), p);
        assert_scalar_stream_eq!(WasmDema::new(10).expect("valid"), p);
        assert_scalar_stream_eq!(WasmTema::new(10).expect("valid"), p);
        assert_scalar_stream_eq!(WasmHma::new(9).expect("valid"), p);
        assert_scalar_stream_eq!(WasmSmma::new(14).expect("valid"), p);
        assert_scalar_stream_eq!(WasmTrima::new(20).expect("valid"), p);
        assert_scalar_stream_eq!(WasmZlema::new(14).expect("valid"), p);
        assert_scalar_stream_eq!(WasmT3::new(5, 0.7).expect("valid"), p);
        assert_scalar_stream_eq!(WasmAlma::new(9, 0.85, 6.0).expect("valid"), p);
        assert_scalar_stream_eq!(WasmMcGinleyDynamic::new(10).expect("valid"), p);
        assert_scalar_stream_eq!(WasmFrama::new(16).expect("valid"), p);
        assert_scalar_stream_eq!(WasmVidya::new(14, 9).expect("valid"), p);
        assert_scalar_stream_eq!(WasmJma::new(14, 0.0, 2).expect("valid"), p);

        // Momentum / oscillators.
        assert_scalar_stream_eq!(WasmRsi::new(14).expect("valid"), p);
        assert_scalar_stream_eq!(WasmRoc::new(12).expect("valid"), p);
        assert_scalar_stream_eq!(WasmTrix::new(9).expect("valid"), p);
        assert_scalar_stream_eq!(WasmMom::new(10).expect("valid"), p);
        assert_scalar_stream_eq!(WasmCmo::new(14).expect("valid"), p);
        assert_scalar_stream_eq!(WasmTsi::new(25, 13).expect("valid"), p);
        assert_scalar_stream_eq!(WasmPmo::new(35, 20).expect("valid"), p);
        assert_scalar_stream_eq!(WasmTii::new(20, 10).expect("valid"), p);
        assert_scalar_stream_eq!(WasmStochRsi::new(14, 14).expect("valid"), p);
        assert_scalar_stream_eq!(WasmDpo::new(20).expect("valid"), p);
        assert_scalar_stream_eq!(WasmPpo::new(12, 26).expect("valid"), p);
        assert_scalar_stream_eq!(WasmApo::new(12, 26).expect("valid"), p);
        assert_scalar_stream_eq!(WasmCfo::new(14).expect("valid"), p);
        assert_scalar_stream_eq!(WasmStc::new(23, 50, 10, 0.5).expect("valid"), p);
        assert_scalar_stream_eq!(WasmCoppock::new(14, 11, 10).expect("valid"), p);
        assert_scalar_stream_eq!(WasmLaguerreRsi::new(0.5).expect("valid"), p);
        assert_scalar_stream_eq!(WasmConnorsRsi::new(3, 2, 100).expect("valid"), p);

        // Volatility / statistics / regression.
        assert_scalar_stream_eq!(WasmStdDev::new(20).expect("valid"), p);
        assert_scalar_stream_eq!(WasmUlcerIndex::new(14).expect("valid"), p);
        assert_scalar_stream_eq!(WasmHistoricalVolatility::new(20, 252).expect("valid"), p);
        assert_scalar_stream_eq!(WasmBollingerBandwidth::new(20, 2.0).expect("valid"), p);
        assert_scalar_stream_eq!(WasmPercentB::new(20, 2.0).expect("valid"), p);
        assert_scalar_stream_eq!(WasmLinearRegression::new(14).expect("valid"), p);
        assert_scalar_stream_eq!(WasmLinRegSlope::new(14).expect("valid"), p);
        assert_scalar_stream_eq!(WasmLinRegAngle::new(14).expect("valid"), p);
        assert_scalar_stream_eq!(WasmVerticalHorizontalFilter::new(28).expect("valid"), p);
        assert_scalar_stream_eq!(WasmZScore::new(20).expect("valid"), p);
        assert_scalar_stream_eq!(WasmVariance::new(20).expect("valid"), p);
        assert_scalar_stream_eq!(WasmCoefficientOfVariation::new(20).expect("valid"), p);
        assert_scalar_stream_eq!(WasmSkewness::new(20).expect("valid"), p);
        assert_scalar_stream_eq!(WasmKurtosis::new(20).expect("valid"), p);
        assert_scalar_stream_eq!(WasmStandardError::new(14).expect("valid"), p);
        assert_scalar_stream_eq!(WasmDetrendedStdDev::new(14).expect("valid"), p);
        assert_scalar_stream_eq!(WasmRSquared::new(14).expect("valid"), p);
        assert_scalar_stream_eq!(WasmMedianAbsoluteDeviation::new(20).expect("valid"), p);
        assert_scalar_stream_eq!(WasmAutocorrelation::new(20, 1).expect("valid"), p);
        assert_scalar_stream_eq!(WasmHurstExponent::new(40, 4).expect("valid"), p);
        assert_scalar_stream_eq!(WasmRviVolatility::new(10).expect("valid"), p);

        // Ehlers / cycle.
        assert_scalar_stream_eq!(WasmSuperSmoother::new(10).expect("valid"), p);
        assert_scalar_stream_eq!(WasmFisherTransform::new(10).expect("valid"), p);
        assert_scalar_stream_eq!(WasmInverseFisherTransform::new(1.0).expect("valid"), p);
        assert_scalar_stream_eq!(WasmDecycler::new(20).expect("valid"), p);
        assert_scalar_stream_eq!(WasmDecyclerOscillator::new(10, 30).expect("valid"), p);
        assert_scalar_stream_eq!(WasmRoofingFilter::new(10, 48).expect("valid"), p);
        assert_scalar_stream_eq!(WasmCenterOfGravity::new(10).expect("valid"), p);
        assert_scalar_stream_eq!(WasmCyberneticCycle::new(10).expect("valid"), p);
        assert_scalar_stream_eq!(WasmInstantaneousTrendline::new(20).expect("valid"), p);
        assert_scalar_stream_eq!(WasmEhlersStochastic::new(20).expect("valid"), p);
        assert_scalar_stream_eq!(
            WasmEmpiricalModeDecomposition::new(20, 0.5).expect("valid"),
            p
        );
        assert_scalar_stream_eq!(WasmFama::new(0.5, 0.05).expect("valid"), p);

        // Risk / performance (scalar f64 input).
        assert_scalar_stream_eq!(WasmCalmarRatio::new(20).expect("valid"), p);
        assert_scalar_stream_eq!(WasmMaxDrawdown::new(20).expect("valid"), p);
        assert_scalar_stream_eq!(WasmAverageDrawdown::new(20).expect("valid"), p);
        assert_scalar_stream_eq!(WasmPainIndex::new(20).expect("valid"), p);
        assert_scalar_stream_eq!(WasmProfitFactor::new(20).expect("valid"), p);
        assert_scalar_stream_eq!(WasmGainLossRatio::new(20).expect("valid"), p);
        assert_scalar_stream_eq!(WasmKellyCriterion::new(20).expect("valid"), p);
        assert_scalar_stream_eq!(WasmSharpeRatio::new(20, 0.0).expect("valid"), p);
        assert_scalar_stream_eq!(WasmSortinoRatio::new(20, 0.0).expect("valid"), p);
        assert_scalar_stream_eq!(WasmOmegaRatio::new(20, 0.0).expect("valid"), p);
        assert_scalar_stream_eq!(WasmValueAtRisk::new(20, 0.95).expect("valid"), p);
        assert_scalar_stream_eq!(WasmConditionalValueAtRisk::new(20, 0.95).expect("valid"), p);
    }

    // Additional invalid-constructor coverage. These wrap the same fallible core
    // `new` as the Python / Node bindings, where the equivalent calls are
    // confirmed to error.
    #[wasm_bindgen_test]
    fn additional_invalid_constructors_are_rejected() {
        assert!(WasmDecyclerOscillator::new(30, 10).is_err()); // short cutoff >= long
        assert!(WasmRoofingFilter::new(48, 10).is_err()); // lowpass >= highpass
        assert!(WasmInverseFisherTransform::new(0.0).is_err()); // zero scale
        assert!(WasmEmpiricalModeDecomposition::new(20, 0.0).is_err()); // zero fraction
    }
}
// ============================== Family 15: Risk / Performance ==============================

// Most metrics need fallible `new` (period >= 2), so they're written by hand
// rather than going through `wasm_scalar_indicator!`. Single-parameter helpers
// reuse the same patterns as the rest of the file.

wasm_scalar_indicator!(WasmCalmarRatio, "CalmarRatio", wc::CalmarRatio, period: usize);
wasm_scalar_indicator!(WasmMaxDrawdown, "MaxDrawdown", wc::MaxDrawdown, period: usize);
wasm_scalar_indicator!(WasmAverageDrawdown, "AverageDrawdown", wc::AverageDrawdown, period: usize);
wasm_scalar_indicator!(WasmPainIndex, "PainIndex", wc::PainIndex, period: usize);
wasm_scalar_indicator!(WasmProfitFactor, "ProfitFactor", wc::ProfitFactor, period: usize);
wasm_scalar_indicator!(WasmGainLossRatio, "GainLossRatio", wc::GainLossRatio, period: usize);
wasm_scalar_indicator!(WasmKellyCriterion, "KellyCriterion", wc::KellyCriterion, period: usize);
wasm_scalar_indicator!(WasmSharpeRatio, "SharpeRatio", wc::SharpeRatio, period: usize, risk_free: f64);
wasm_scalar_indicator!(WasmSortinoRatio, "SortinoRatio", wc::SortinoRatio, period: usize, mar: f64);
wasm_scalar_indicator!(WasmOmegaRatio, "OmegaRatio", wc::OmegaRatio, period: usize, threshold: f64);
wasm_scalar_indicator!(WasmValueAtRisk, "ValueAtRisk", wc::ValueAtRisk, period: usize, confidence: f64);
wasm_scalar_indicator!(WasmConditionalValueAtRisk, "ConditionalValueAtRisk", wc::ConditionalValueAtRisk, period: usize, confidence: f64);

// --- DrawdownDuration: u32 output, no constructor args ---

#[wasm_bindgen(js_name = DrawdownDuration)]
pub struct WasmDrawdownDuration {
    inner: wc::DrawdownDuration,
}

impl Default for WasmDrawdownDuration {
    fn default() -> Self {
        Self::new()
    }
}

#[wasm_bindgen(js_class = DrawdownDuration)]
impl WasmDrawdownDuration {
    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmDrawdownDuration {
        Self {
            inner: wc::DrawdownDuration::new(),
        }
    }
    pub fn update(&mut self, value: f64) -> Option<u32> {
        self.inner.update(value)
    }
    pub fn batch(&mut self, prices: &[f64]) -> Float64Array {
        let out: Vec<f64> = prices
            .iter()
            .map(|p| self.inner.update(*p).map_or(f64::NAN, f64::from))
            .collect();
        Float64Array::from(out.as_slice())
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
    #[wasm_bindgen(js_name = isReady)]
    pub fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    #[wasm_bindgen(js_name = warmupPeriod)]
    pub fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
}

// --- RecoveryFactor: no constructor args ---

#[wasm_bindgen(js_name = RecoveryFactor)]
pub struct WasmRecoveryFactor {
    inner: wc::RecoveryFactor,
}

impl Default for WasmRecoveryFactor {
    fn default() -> Self {
        Self::new()
    }
}

#[wasm_bindgen(js_class = RecoveryFactor)]
impl WasmRecoveryFactor {
    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmRecoveryFactor {
        Self {
            inner: wc::RecoveryFactor::new(),
        }
    }
    pub fn update(&mut self, value: f64) -> Option<f64> {
        self.inner.update(value)
    }
    pub fn batch(&mut self, prices: &[f64]) -> Float64Array {
        let out = flatten(self.inner.batch(prices));
        Float64Array::from(out.as_slice())
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
    #[wasm_bindgen(js_name = isReady)]
    pub fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    #[wasm_bindgen(js_name = warmupPeriod)]
    pub fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
}

// --- Two-series (asset, benchmark) indicators ---
//
// Family 12 (PR #51) introduces `wasm_pair_indicator!` for Pearson / Beta /
// Spearman. Family 12 is not in main, so Family 15 writes its three pair
// wrappers by hand here; merge with PR #51 keeps the macro and re-uses it.

#[wasm_bindgen(js_name = TreynorRatio)]
pub struct WasmTreynorRatio {
    inner: wc::TreynorRatio,
}

#[wasm_bindgen(js_class = TreynorRatio)]
impl WasmTreynorRatio {
    #[wasm_bindgen(constructor)]
    pub fn new(period: usize, risk_free: f64) -> Result<WasmTreynorRatio, JsError> {
        Ok(Self {
            inner: wc::TreynorRatio::new(period, risk_free).map_err(map_err)?,
        })
    }
    pub fn update(&mut self, asset: f64, benchmark: f64) -> Option<f64> {
        self.inner.update((asset, benchmark))
    }
    pub fn batch(&mut self, asset: &[f64], benchmark: &[f64]) -> Result<Float64Array, JsError> {
        if asset.len() != benchmark.len() {
            return Err(JsError::new("asset and benchmark must be equal length"));
        }
        let mut out = Vec::with_capacity(asset.len());
        for i in 0..asset.len() {
            out.push(
                self.inner
                    .update((asset[i], benchmark[i]))
                    .unwrap_or(f64::NAN),
            );
        }
        Ok(Float64Array::from(out.as_slice()))
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
    #[wasm_bindgen(js_name = isReady)]
    pub fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    #[wasm_bindgen(js_name = warmupPeriod)]
    pub fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
}

#[wasm_bindgen(js_name = InformationRatio)]
pub struct WasmInformationRatio {
    inner: wc::InformationRatio,
}

#[wasm_bindgen(js_class = InformationRatio)]
impl WasmInformationRatio {
    #[wasm_bindgen(constructor)]
    pub fn new(period: usize) -> Result<WasmInformationRatio, JsError> {
        Ok(Self {
            inner: wc::InformationRatio::new(period).map_err(map_err)?,
        })
    }
    pub fn update(&mut self, asset: f64, benchmark: f64) -> Option<f64> {
        self.inner.update((asset, benchmark))
    }
    pub fn batch(&mut self, asset: &[f64], benchmark: &[f64]) -> Result<Float64Array, JsError> {
        if asset.len() != benchmark.len() {
            return Err(JsError::new("asset and benchmark must be equal length"));
        }
        let mut out = Vec::with_capacity(asset.len());
        for i in 0..asset.len() {
            out.push(
                self.inner
                    .update((asset[i], benchmark[i]))
                    .unwrap_or(f64::NAN),
            );
        }
        Ok(Float64Array::from(out.as_slice()))
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
    #[wasm_bindgen(js_name = isReady)]
    pub fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    #[wasm_bindgen(js_name = warmupPeriod)]
    pub fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
}

// ============================== Alt-Chart Bars ==============================
//
// Bar builders consume close prices and emit a variable number of completed bars
// per input. `update(close)` returns a JS array of the bars finished on that
// close; `batch` returns all completed bars concatenated.

#[wasm_bindgen(js_name = RenkoBars)]
pub struct WasmRenkoBars {
    inner: wc::RenkoBars,
}

#[wasm_bindgen(js_class = RenkoBars)]
impl WasmRenkoBars {
    #[wasm_bindgen(constructor)]
    pub fn new(box_size: f64) -> Result<WasmRenkoBars, JsError> {
        Ok(Self {
            inner: wc::RenkoBars::new(box_size).map_err(map_err)?,
        })
    }
    /// Returns an array of `{ open, close, direction }` bricks completed on this close.
    pub fn update(&mut self, close: f64) -> Result<Array, JsError> {
        let candle = wc::Candle::new(close, close, close, close, 1.0, 0).map_err(map_err)?;
        let arr = Array::new();
        for b in self.inner.update(candle) {
            let obj = Object::new();
            Reflect::set(&obj, &"open".into(), &b.open.into()).ok();
            Reflect::set(&obj, &"close".into(), &b.close.into()).ok();
            Reflect::set(&obj, &"direction".into(), &f64::from(b.direction).into()).ok();
            arr.push(&obj);
        }
        Ok(arr)
    }
    pub fn batch(&mut self, close: &[f64]) -> Result<Array, JsError> {
        let arr = Array::new();
        for &price in close {
            let candle = wc::Candle::new(price, price, price, price, 1.0, 0).map_err(map_err)?;
            for b in self.inner.update(candle) {
                let obj = Object::new();
                Reflect::set(&obj, &"open".into(), &b.open.into()).ok();
                Reflect::set(&obj, &"close".into(), &b.close.into()).ok();
                Reflect::set(&obj, &"direction".into(), &f64::from(b.direction).into()).ok();
                arr.push(&obj);
            }
        }
        Ok(arr)
    }
    #[wasm_bindgen(js_name = boxSize)]
    pub fn box_size(&self) -> f64 {
        self.inner.box_size()
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
}

#[wasm_bindgen(js_name = KagiBars)]
pub struct WasmKagiBars {
    inner: wc::KagiBars,
}

#[wasm_bindgen(js_class = KagiBars)]
impl WasmKagiBars {
    #[wasm_bindgen(constructor)]
    pub fn new(reversal: f64) -> Result<WasmKagiBars, JsError> {
        Ok(Self {
            inner: wc::KagiBars::new(reversal).map_err(map_err)?,
        })
    }
    /// Returns an array of `{ start, end, direction }` segments completed on this close.
    pub fn update(&mut self, close: f64) -> Result<Array, JsError> {
        let candle = wc::Candle::new(close, close, close, close, 1.0, 0).map_err(map_err)?;
        let arr = Array::new();
        for b in self.inner.update(candle) {
            let obj = Object::new();
            Reflect::set(&obj, &"start".into(), &b.start.into()).ok();
            Reflect::set(&obj, &"end".into(), &b.end.into()).ok();
            Reflect::set(&obj, &"direction".into(), &f64::from(b.direction).into()).ok();
            arr.push(&obj);
        }
        Ok(arr)
    }
    pub fn batch(&mut self, close: &[f64]) -> Result<Array, JsError> {
        let arr = Array::new();
        for &price in close {
            let candle = wc::Candle::new(price, price, price, price, 1.0, 0).map_err(map_err)?;
            for b in self.inner.update(candle) {
                let obj = Object::new();
                Reflect::set(&obj, &"start".into(), &b.start.into()).ok();
                Reflect::set(&obj, &"end".into(), &b.end.into()).ok();
                Reflect::set(&obj, &"direction".into(), &f64::from(b.direction).into()).ok();
                arr.push(&obj);
            }
        }
        Ok(arr)
    }
    pub fn reversal(&self) -> f64 {
        self.inner.reversal()
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
}

#[wasm_bindgen(js_name = PointAndFigureBars)]
pub struct WasmPointAndFigureBars {
    inner: wc::PointAndFigureBars,
}

#[wasm_bindgen(js_class = PointAndFigureBars)]
impl WasmPointAndFigureBars {
    #[wasm_bindgen(constructor)]
    pub fn new(box_size: f64, reversal: usize) -> Result<WasmPointAndFigureBars, JsError> {
        Ok(Self {
            inner: wc::PointAndFigureBars::new(box_size, reversal).map_err(map_err)?,
        })
    }
    /// Returns an array of `{ direction, high, low }` columns completed on this close.
    pub fn update(&mut self, close: f64) -> Result<Array, JsError> {
        let candle = wc::Candle::new(close, close, close, close, 1.0, 0).map_err(map_err)?;
        let arr = Array::new();
        for col in self.inner.update(candle) {
            let obj = Object::new();
            Reflect::set(&obj, &"direction".into(), &f64::from(col.direction).into()).ok();
            Reflect::set(&obj, &"high".into(), &col.high.into()).ok();
            Reflect::set(&obj, &"low".into(), &col.low.into()).ok();
            arr.push(&obj);
        }
        Ok(arr)
    }
    pub fn batch(&mut self, close: &[f64]) -> Result<Array, JsError> {
        let arr = Array::new();
        for &price in close {
            let candle = wc::Candle::new(price, price, price, price, 1.0, 0).map_err(map_err)?;
            for col in self.inner.update(candle) {
                let obj = Object::new();
                Reflect::set(&obj, &"direction".into(), &f64::from(col.direction).into()).ok();
                Reflect::set(&obj, &"high".into(), &col.high.into()).ok();
                Reflect::set(&obj, &"low".into(), &col.low.into()).ok();
                arr.push(&obj);
            }
        }
        Ok(arr)
    }
    #[wasm_bindgen(js_name = boxSize)]
    pub fn box_size(&self) -> f64 {
        self.inner.box_size()
    }
    pub fn reversal(&self) -> usize {
        self.inner.reversal()
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
}

#[wasm_bindgen(js_name = Alpha)]
pub struct WasmAlpha {
    inner: wc::Alpha,
}

#[wasm_bindgen(js_class = Alpha)]
impl WasmAlpha {
    #[wasm_bindgen(constructor)]
    pub fn new(period: usize, risk_free: f64) -> Result<WasmAlpha, JsError> {
        Ok(Self {
            inner: wc::Alpha::new(period, risk_free).map_err(map_err)?,
        })
    }
    pub fn update(&mut self, asset: f64, benchmark: f64) -> Option<f64> {
        self.inner.update((asset, benchmark))
    }
    pub fn batch(&mut self, asset: &[f64], benchmark: &[f64]) -> Result<Float64Array, JsError> {
        if asset.len() != benchmark.len() {
            return Err(JsError::new("asset and benchmark must be equal length"));
        }
        let mut out = Vec::with_capacity(asset.len());
        for i in 0..asset.len() {
            out.push(
                self.inner
                    .update((asset[i], benchmark[i]))
                    .unwrap_or(f64::NAN),
            );
        }
        Ok(Float64Array::from(out.as_slice()))
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
    #[wasm_bindgen(js_name = isReady)]
    pub fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    #[wasm_bindgen(js_name = warmupPeriod)]
    pub fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }
}
