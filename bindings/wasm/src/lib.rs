//! WASM bindings for Wickra. Exposes every indicator with `Float64Array` I/O so
//! the API is essentially the same in the browser as it is in Python and Rust.
//!
//! Build with:
//! ```text
//! wasm-pack build bindings/wasm --target web --release
//! ```

#![allow(clippy::needless_pass_by_value)]
#![allow(missing_debug_implementations)] // wasm_bindgen wrappers expose JS objects, no need for Debug

use js_sys::{Float64Array, Object, Reflect};
use wasm_bindgen::prelude::*;
use wickra_core as wc;
use wickra_core::{BatchExt, Indicator};

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
wasm_scalar_indicator!(WasmMom, "MOM", wc::Mom, period: usize);
wasm_scalar_indicator!(WasmCmo, "CMO", wc::Cmo, period: usize);
wasm_scalar_indicator!(WasmTsi, "TSI", wc::Tsi, long: usize, short: usize);
wasm_scalar_indicator!(WasmPmo, "PMO", wc::Pmo, smoothing1: usize, smoothing2: usize);
wasm_scalar_indicator!(WasmStochRsi, "StochRSI", wc::StochRsi, rsi_period: usize, stoch_period: usize);
wasm_scalar_indicator!(WasmDpo, "DPO", wc::Dpo, period: usize);
wasm_scalar_indicator!(WasmPpo, "PPO", wc::Ppo, fast: usize, slow: usize);
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
        signal: usize,
    ) -> Result<WasmKst, JsError> {
        Ok(Self {
            inner: wc::Kst::new(roc1, roc2, roc3, roc4, sma1, sma2, sma3, sma4, signal)
                .map_err(map_err)?,
        })
    }
    /// Returns `[kst0, signal0, kst1, signal1, ...]`, length `2n`.
    pub fn batch(&mut self, prices: &[f64]) -> Float64Array {
        let n = prices.len();
        let mut out = vec![f64::NAN; n * 2];
        for (i, p) in prices.iter().enumerate() {
            if let Some(o) = self.inner.update(*p) {
                out[i * 2] = o.kst;
                out[i * 2 + 1] = o.signal;
            }
        }
        Float64Array::from(out.as_slice())
    }
    /// Streaming update. Returns `{ kst, signal }` once warm, else `null`.
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
}
