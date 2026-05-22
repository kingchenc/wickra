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
}
