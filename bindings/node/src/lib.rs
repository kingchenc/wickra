//! Node.js bindings for Wickra via napi-rs.
//!
//! Build with:
//! ```text
//! cd bindings/node && npm install && npm run build
//! ```
//!
//! Then `require("@wickra/wickra")` from Node.

#![allow(clippy::needless_pass_by_value)]
#![allow(missing_debug_implementations)] // napi-derive auto-generates the Node-facing types.
#![allow(clippy::unused_self)]
#![allow(clippy::missing_const_for_fn)]

use napi::Error as NapiError;
use napi::Status;
use napi_derive::napi;
use wickra_core as wc;
use wickra_core::{BatchExt, Indicator};

fn map_err(e: wc::Error) -> NapiError {
    NapiError::new(Status::InvalidArg, e.to_string())
}

/// Helper for the scalar-indicator macro only. Scalar `new` functions can fail
/// solely on `period == 0`, which `clamp_period` already rules out, so the
/// `Result` is provably `Ok` here. Candle indicators and the multi-parameter
/// indicators have genuinely fallible parameters and instead use fallible
/// `#[napi(constructor)]`s that return `napi::Result<Self>` and throw a JS error.
fn must<T>(r: Result<T, wc::Error>) -> T {
    r.expect("wickra: scalar indicator parameter clamped to a valid range")
}

/// Clamp a period parameter so the underlying indicator never sees zero. JS
/// callers who pass `0` get a window of `1` instead of a thrown exception —
/// effectively a pass-through indicator that still produces valid outputs.
const fn clamp_period(p: u32) -> usize {
    if p == 0 {
        1
    } else {
        p as usize
    }
}

fn flatten(v: Vec<Option<f64>>) -> Vec<f64> {
    v.into_iter().map(|x| x.unwrap_or(f64::NAN)).collect()
}

/// Library version (matches the Rust crate version).
#[napi]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

// ============================== Scalar indicators ==============================

macro_rules! node_scalar_indicator {
    ($wrapper:ident, $node_name:literal, $rust_ty:ty) => {
        #[napi(js_name = $node_name)]
        pub struct $wrapper {
            inner: $rust_ty,
        }

        #[napi]
        impl $wrapper {
            #[napi(constructor)]
            pub fn new(period: u32) -> Self {
                Self {
                    inner: must(<$rust_ty>::new(clamp_period(period))),
                }
            }
            #[napi]
            pub fn update(&mut self, value: f64) -> Option<f64> {
                self.inner.update(value)
            }
            #[napi]
            pub fn batch(&mut self, prices: Vec<f64>) -> Vec<f64> {
                flatten(self.inner.batch(&prices))
            }
            #[napi]
            pub fn reset(&mut self) {
                self.inner.reset();
            }
            #[napi(js_name = "isReady")]
            pub fn is_ready(&self) -> bool {
                self.inner.is_ready()
            }
            #[napi(js_name = "warmupPeriod")]
            pub fn warmup_period(&self) -> u32 {
                self.inner.warmup_period() as u32
            }
        }
    };
}

node_scalar_indicator!(SmaNode, "SMA", wc::Sma);
node_scalar_indicator!(EmaNode, "EMA", wc::Ema);
node_scalar_indicator!(WmaNode, "WMA", wc::Wma);
node_scalar_indicator!(RsiNode, "RSI", wc::Rsi);
node_scalar_indicator!(DemaNode, "DEMA", wc::Dema);
node_scalar_indicator!(TemaNode, "TEMA", wc::Tema);
node_scalar_indicator!(HmaNode, "HMA", wc::Hma);
node_scalar_indicator!(RocNode, "ROC", wc::Roc);
node_scalar_indicator!(TrixNode, "TRIX", wc::Trix);
node_scalar_indicator!(SmmaNode, "SMMA", wc::Smma);
node_scalar_indicator!(TrimaNode, "TRIMA", wc::Trima);
node_scalar_indicator!(ZlemaNode, "ZLEMA", wc::Zlema);
node_scalar_indicator!(MomNode, "MOM", wc::Mom);
node_scalar_indicator!(CmoNode, "CMO", wc::Cmo);

// ============================== MACD ==============================

/// MACD triple: macd line, signal line, histogram.
#[napi(object)]
pub struct MacdValue {
    pub macd: f64,
    pub signal: f64,
    pub histogram: f64,
}

#[napi(js_name = "MACD")]
pub struct MacdNode {
    inner: wc::MacdIndicator,
}

#[napi]
impl MacdNode {
    #[napi(constructor)]
    pub fn new(fast: u32, slow: u32, signal: u32) -> napi::Result<Self> {
        Ok(Self {
            inner: wc::MacdIndicator::new(fast as usize, slow as usize, signal as usize)
                .map_err(map_err)?,
        })
    }
    #[napi]
    pub fn update(&mut self, value: f64) -> Option<MacdValue> {
        self.inner.update(value).map(|o| MacdValue {
            macd: o.macd,
            signal: o.signal,
            histogram: o.histogram,
        })
    }
    /// Batch over a price array. Returns a flat array of length `3 * n`,
    /// interleaved per row as `[macd0, signal0, histogram0, macd1, ...]`.
    /// Read column `j` of row `i` as `result[i * 3 + j]`. Warmup rows are `NaN`.
    #[napi]
    pub fn batch(&mut self, prices: Vec<f64>) -> Vec<f64> {
        let mut out = vec![f64::NAN; prices.len() * 3];
        for (i, p) in prices.iter().enumerate() {
            if let Some(o) = self.inner.update(*p) {
                out[i * 3] = o.macd;
                out[i * 3 + 1] = o.signal;
                out[i * 3 + 2] = o.histogram;
            }
        }
        out
    }
    #[napi]
    pub fn reset(&mut self) {
        self.inner.reset();
    }
    #[napi(js_name = "isReady")]
    pub fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    #[napi(js_name = "warmupPeriod")]
    pub fn warmup_period(&self) -> u32 {
        self.inner.warmup_period() as u32
    }
}

// ============================== Bollinger ==============================

#[napi(object)]
pub struct BollingerValue {
    pub upper: f64,
    pub middle: f64,
    pub lower: f64,
    pub stddev: f64,
}

#[napi(js_name = "BollingerBands")]
pub struct BollingerNode {
    inner: wc::BollingerBands,
}

#[napi]
impl BollingerNode {
    #[napi(constructor)]
    pub fn new(period: u32, multiplier: f64) -> napi::Result<Self> {
        Ok(Self {
            inner: wc::BollingerBands::new(period as usize, multiplier).map_err(map_err)?,
        })
    }
    #[napi]
    pub fn update(&mut self, value: f64) -> Option<BollingerValue> {
        self.inner.update(value).map(|o| BollingerValue {
            upper: o.upper,
            middle: o.middle,
            lower: o.lower,
            stddev: o.stddev,
        })
    }
    /// Batch over a price array. Returns a flat array of length `4 * n`,
    /// interleaved per row as `[upper0, middle0, lower0, stddev0, upper1, ...]`.
    /// Read column `j` of row `i` as `result[i * 4 + j]`. Warmup rows are `NaN`.
    #[napi]
    pub fn batch(&mut self, prices: Vec<f64>) -> Vec<f64> {
        let mut out = vec![f64::NAN; prices.len() * 4];
        for (i, p) in prices.iter().enumerate() {
            if let Some(o) = self.inner.update(*p) {
                out[i * 4] = o.upper;
                out[i * 4 + 1] = o.middle;
                out[i * 4 + 2] = o.lower;
                out[i * 4 + 3] = o.stddev;
            }
        }
        out
    }
    #[napi]
    pub fn reset(&mut self) {
        self.inner.reset();
    }
    #[napi(js_name = "isReady")]
    pub fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    #[napi(js_name = "warmupPeriod")]
    pub fn warmup_period(&self) -> u32 {
        self.inner.warmup_period() as u32
    }
}

// ============================== Candle-input helpers ==============================

fn cnd(h: f64, l: f64, c: f64, v: f64) -> napi::Result<wc::Candle> {
    wc::Candle::new(c, h, l, c, v, 0).map_err(map_err)
}

#[napi(js_name = "ATR")]
pub struct AtrNode {
    inner: wc::Atr,
}

#[napi]
impl AtrNode {
    #[napi(constructor)]
    pub fn new(period: u32) -> napi::Result<Self> {
        Ok(Self {
            inner: wc::Atr::new(period as usize).map_err(map_err)?,
        })
    }
    #[napi]
    pub fn update(&mut self, high: f64, low: f64, close: f64) -> napi::Result<Option<f64>> {
        Ok(self.inner.update(cnd(high, low, close, 0.0)?))
    }
    #[napi]
    pub fn batch(
        &mut self,
        high: Vec<f64>,
        low: Vec<f64>,
        close: Vec<f64>,
    ) -> napi::Result<Vec<f64>> {
        if high.len() != low.len() || low.len() != close.len() {
            return Err(NapiError::from_reason(
                "high, low, close must be equal length".to_string(),
            ));
        }
        let mut out = Vec::with_capacity(high.len());
        for i in 0..high.len() {
            out.push(
                self.inner
                    .update(cnd(high[i], low[i], close[i], 0.0)?)
                    .unwrap_or(f64::NAN),
            );
        }
        Ok(out)
    }
    #[napi]
    pub fn reset(&mut self) {
        self.inner.reset();
    }
    #[napi(js_name = "isReady")]
    pub fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    #[napi(js_name = "warmupPeriod")]
    pub fn warmup_period(&self) -> u32 {
        self.inner.warmup_period() as u32
    }
}

#[napi(object)]
pub struct StochValue {
    pub k: f64,
    pub d: f64,
}

#[napi(js_name = "Stochastic")]
pub struct StochNode {
    inner: wc::Stochastic,
}

#[napi]
impl StochNode {
    #[napi(constructor)]
    pub fn new(k_period: u32, d_period: u32) -> napi::Result<Self> {
        Ok(Self {
            inner: wc::Stochastic::new(k_period as usize, d_period as usize).map_err(map_err)?,
        })
    }
    #[napi]
    pub fn update(&mut self, high: f64, low: f64, close: f64) -> napi::Result<Option<StochValue>> {
        Ok(self
            .inner
            .update(cnd(high, low, close, 0.0)?)
            .map(|o| StochValue { k: o.k, d: o.d }))
    }
    #[napi]
    pub fn batch(
        &mut self,
        high: Vec<f64>,
        low: Vec<f64>,
        close: Vec<f64>,
    ) -> napi::Result<Vec<f64>> {
        if high.len() != low.len() || low.len() != close.len() {
            return Err(NapiError::from_reason(
                "high, low, close must be equal length".to_string(),
            ));
        }
        let n = high.len();
        let mut out = vec![f64::NAN; n * 2];
        for i in 0..n {
            if let Some(o) = self.inner.update(cnd(high[i], low[i], close[i], 0.0)?) {
                out[i * 2] = o.k;
                out[i * 2 + 1] = o.d;
            }
        }
        Ok(out)
    }
    #[napi]
    pub fn reset(&mut self) {
        self.inner.reset();
    }
    #[napi(js_name = "isReady")]
    pub fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    #[napi(js_name = "warmupPeriod")]
    pub fn warmup_period(&self) -> u32 {
        self.inner.warmup_period() as u32
    }
}

#[napi(js_name = "OBV")]
pub struct ObvNode {
    inner: wc::Obv,
}

impl Default for ObvNode {
    fn default() -> Self {
        Self::new()
    }
}

#[napi]
impl ObvNode {
    #[napi(constructor)]
    pub fn new() -> Self {
        Self {
            inner: wc::Obv::new(),
        }
    }
    #[napi]
    pub fn update(&mut self, close: f64, volume: f64) -> napi::Result<Option<f64>> {
        Ok(self.inner.update(cnd(close, close, close, volume)?))
    }
    #[napi]
    pub fn batch(&mut self, close: Vec<f64>, volume: Vec<f64>) -> napi::Result<Vec<f64>> {
        if close.len() != volume.len() {
            return Err(NapiError::from_reason(
                "close and volume must be equal length".to_string(),
            ));
        }
        let mut out = Vec::with_capacity(close.len());
        for i in 0..close.len() {
            out.push(
                self.inner
                    .update(cnd(close[i], close[i], close[i], volume[i])?)
                    .unwrap_or(f64::NAN),
            );
        }
        Ok(out)
    }
    #[napi]
    pub fn reset(&mut self) {
        self.inner.reset();
    }
    #[napi(js_name = "isReady")]
    pub fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    #[napi(js_name = "warmupPeriod")]
    pub fn warmup_period(&self) -> u32 {
        self.inner.warmup_period() as u32
    }
}

#[napi(object)]
pub struct AdxValue {
    #[napi(js_name = "plusDi")]
    pub plus_di: f64,
    #[napi(js_name = "minusDi")]
    pub minus_di: f64,
    pub adx: f64,
}

#[napi(js_name = "ADX")]
pub struct AdxNode {
    inner: wc::Adx,
}

#[napi]
impl AdxNode {
    #[napi(constructor)]
    pub fn new(period: u32) -> napi::Result<Self> {
        Ok(Self {
            inner: wc::Adx::new(period as usize).map_err(map_err)?,
        })
    }
    #[napi]
    pub fn update(&mut self, high: f64, low: f64, close: f64) -> napi::Result<Option<AdxValue>> {
        Ok(self
            .inner
            .update(cnd(high, low, close, 0.0)?)
            .map(|o| AdxValue {
                plus_di: o.plus_di,
                minus_di: o.minus_di,
                adx: o.adx,
            }))
    }
    #[napi]
    pub fn batch(
        &mut self,
        high: Vec<f64>,
        low: Vec<f64>,
        close: Vec<f64>,
    ) -> napi::Result<Vec<f64>> {
        if high.len() != low.len() || low.len() != close.len() {
            return Err(NapiError::from_reason(
                "high, low, close must be equal length".to_string(),
            ));
        }
        let n = high.len();
        let mut out = vec![f64::NAN; n * 3];
        for i in 0..n {
            if let Some(o) = self.inner.update(cnd(high[i], low[i], close[i], 0.0)?) {
                out[i * 3] = o.plus_di;
                out[i * 3 + 1] = o.minus_di;
                out[i * 3 + 2] = o.adx;
            }
        }
        Ok(out)
    }
    #[napi]
    pub fn reset(&mut self) {
        self.inner.reset();
    }
    #[napi(js_name = "isReady")]
    pub fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    #[napi(js_name = "warmupPeriod")]
    pub fn warmup_period(&self) -> u32 {
        self.inner.warmup_period() as u32
    }
}

#[napi(js_name = "CCI")]
pub struct CciNode {
    inner: wc::Cci,
}
#[napi]
impl CciNode {
    #[napi(constructor)]
    pub fn new(period: u32) -> napi::Result<Self> {
        Ok(Self {
            inner: wc::Cci::new(period as usize).map_err(map_err)?,
        })
    }
    #[napi]
    pub fn reset(&mut self) {
        self.inner.reset();
    }
    #[napi(js_name = "isReady")]
    pub fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    #[napi(js_name = "warmupPeriod")]
    pub fn warmup_period(&self) -> u32 {
        self.inner.warmup_period() as u32
    }
    #[napi]
    pub fn update(&mut self, high: f64, low: f64, close: f64) -> napi::Result<Option<f64>> {
        Ok(self.inner.update(cnd(high, low, close, 0.0)?))
    }
    #[napi]
    pub fn batch(
        &mut self,
        high: Vec<f64>,
        low: Vec<f64>,
        close: Vec<f64>,
    ) -> napi::Result<Vec<f64>> {
        if high.len() != low.len() || low.len() != close.len() {
            return Err(NapiError::from_reason(
                "high, low, close must be equal length".to_string(),
            ));
        }
        let mut out = Vec::with_capacity(high.len());
        for i in 0..high.len() {
            out.push(
                self.inner
                    .update(cnd(high[i], low[i], close[i], 0.0)?)
                    .unwrap_or(f64::NAN),
            );
        }
        Ok(out)
    }
}

#[napi(js_name = "WilliamsR")]
pub struct WilliamsRNode {
    inner: wc::WilliamsR,
}
#[napi]
impl WilliamsRNode {
    #[napi(constructor)]
    pub fn new(period: u32) -> napi::Result<Self> {
        Ok(Self {
            inner: wc::WilliamsR::new(period as usize).map_err(map_err)?,
        })
    }
    #[napi]
    pub fn reset(&mut self) {
        self.inner.reset();
    }
    #[napi(js_name = "isReady")]
    pub fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    #[napi(js_name = "warmupPeriod")]
    pub fn warmup_period(&self) -> u32 {
        self.inner.warmup_period() as u32
    }
    #[napi]
    pub fn update(&mut self, high: f64, low: f64, close: f64) -> napi::Result<Option<f64>> {
        Ok(self.inner.update(cnd(high, low, close, 0.0)?))
    }
    #[napi]
    pub fn batch(
        &mut self,
        high: Vec<f64>,
        low: Vec<f64>,
        close: Vec<f64>,
    ) -> napi::Result<Vec<f64>> {
        if high.len() != low.len() || low.len() != close.len() {
            return Err(NapiError::from_reason(
                "high, low, close must be equal length".to_string(),
            ));
        }
        let mut out = Vec::with_capacity(high.len());
        for i in 0..high.len() {
            out.push(
                self.inner
                    .update(cnd(high[i], low[i], close[i], 0.0)?)
                    .unwrap_or(f64::NAN),
            );
        }
        Ok(out)
    }
}

#[napi(js_name = "MFI")]
pub struct MfiNode {
    inner: wc::Mfi,
}
#[napi]
impl MfiNode {
    #[napi(constructor)]
    pub fn new(period: u32) -> napi::Result<Self> {
        Ok(Self {
            inner: wc::Mfi::new(period as usize).map_err(map_err)?,
        })
    }
    #[napi]
    pub fn reset(&mut self) {
        self.inner.reset();
    }
    #[napi(js_name = "isReady")]
    pub fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    #[napi(js_name = "warmupPeriod")]
    pub fn warmup_period(&self) -> u32 {
        self.inner.warmup_period() as u32
    }
    #[napi]
    pub fn update(
        &mut self,
        high: f64,
        low: f64,
        close: f64,
        volume: f64,
    ) -> napi::Result<Option<f64>> {
        Ok(self.inner.update(cnd(high, low, close, volume)?))
    }
    #[napi]
    pub fn batch(
        &mut self,
        high: Vec<f64>,
        low: Vec<f64>,
        close: Vec<f64>,
        volume: Vec<f64>,
    ) -> napi::Result<Vec<f64>> {
        if high.len() != low.len() || low.len() != close.len() || close.len() != volume.len() {
            return Err(NapiError::from_reason(
                "high, low, close, volume must be equal length".to_string(),
            ));
        }
        let mut out = Vec::with_capacity(high.len());
        for i in 0..high.len() {
            out.push(
                self.inner
                    .update(cnd(high[i], low[i], close[i], volume[i])?)
                    .unwrap_or(f64::NAN),
            );
        }
        Ok(out)
    }
}

#[napi(js_name = "PSAR")]
pub struct PsarNode {
    inner: wc::Psar,
}
#[napi]
impl PsarNode {
    #[napi(constructor)]
    pub fn new(af_start: f64, af_step: f64, af_max: f64) -> napi::Result<Self> {
        Ok(Self {
            inner: wc::Psar::new(af_start, af_step, af_max).map_err(map_err)?,
        })
    }
    #[napi]
    pub fn reset(&mut self) {
        self.inner.reset();
    }
    #[napi(js_name = "isReady")]
    pub fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    #[napi(js_name = "warmupPeriod")]
    pub fn warmup_period(&self) -> u32 {
        self.inner.warmup_period() as u32
    }
    #[napi]
    pub fn update(&mut self, high: f64, low: f64, close: f64) -> napi::Result<Option<f64>> {
        Ok(self.inner.update(cnd(high, low, close, 0.0)?))
    }
    #[napi]
    pub fn batch(
        &mut self,
        high: Vec<f64>,
        low: Vec<f64>,
        close: Vec<f64>,
    ) -> napi::Result<Vec<f64>> {
        if high.len() != low.len() || low.len() != close.len() {
            return Err(NapiError::from_reason(
                "high, low, close must be equal length".to_string(),
            ));
        }
        let mut out = Vec::with_capacity(high.len());
        for i in 0..high.len() {
            out.push(
                self.inner
                    .update(cnd(high[i], low[i], close[i], 0.0)?)
                    .unwrap_or(f64::NAN),
            );
        }
        Ok(out)
    }
}

#[napi(object)]
pub struct KeltnerValue {
    pub upper: f64,
    pub middle: f64,
    pub lower: f64,
}

#[napi(js_name = "Keltner")]
pub struct KeltnerNode {
    inner: wc::Keltner,
}
#[napi]
impl KeltnerNode {
    #[napi(constructor)]
    pub fn new(ema_period: u32, atr_period: u32, multiplier: f64) -> napi::Result<Self> {
        Ok(Self {
            inner: wc::Keltner::new(ema_period as usize, atr_period as usize, multiplier)
                .map_err(map_err)?,
        })
    }
    #[napi]
    pub fn reset(&mut self) {
        self.inner.reset();
    }
    #[napi(js_name = "isReady")]
    pub fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    #[napi(js_name = "warmupPeriod")]
    pub fn warmup_period(&self) -> u32 {
        self.inner.warmup_period() as u32
    }
    #[napi]
    pub fn update(
        &mut self,
        high: f64,
        low: f64,
        close: f64,
    ) -> napi::Result<Option<KeltnerValue>> {
        Ok(self
            .inner
            .update(cnd(high, low, close, 0.0)?)
            .map(|o| KeltnerValue {
                upper: o.upper,
                middle: o.middle,
                lower: o.lower,
            }))
    }
    #[napi]
    pub fn batch(
        &mut self,
        high: Vec<f64>,
        low: Vec<f64>,
        close: Vec<f64>,
    ) -> napi::Result<Vec<f64>> {
        if high.len() != low.len() || low.len() != close.len() {
            return Err(NapiError::from_reason(
                "high, low, close must be equal length".to_string(),
            ));
        }
        let n = high.len();
        let mut out = vec![f64::NAN; n * 3];
        for i in 0..n {
            if let Some(o) = self.inner.update(cnd(high[i], low[i], close[i], 0.0)?) {
                out[i * 3] = o.upper;
                out[i * 3 + 1] = o.middle;
                out[i * 3 + 2] = o.lower;
            }
        }
        Ok(out)
    }
}

#[napi(object)]
pub struct DonchianValue {
    pub upper: f64,
    pub middle: f64,
    pub lower: f64,
}

#[napi(js_name = "Donchian")]
pub struct DonchianNode {
    inner: wc::Donchian,
}
#[napi]
impl DonchianNode {
    #[napi(constructor)]
    pub fn new(period: u32) -> napi::Result<Self> {
        Ok(Self {
            inner: wc::Donchian::new(period as usize).map_err(map_err)?,
        })
    }
    #[napi]
    pub fn reset(&mut self) {
        self.inner.reset();
    }
    #[napi(js_name = "isReady")]
    pub fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    #[napi(js_name = "warmupPeriod")]
    pub fn warmup_period(&self) -> u32 {
        self.inner.warmup_period() as u32
    }
    #[napi]
    pub fn update(&mut self, high: f64, low: f64) -> napi::Result<Option<DonchianValue>> {
        Ok(self
            .inner
            .update(cnd(high, low, low, 0.0)?)
            .map(|o| DonchianValue {
                upper: o.upper,
                middle: o.middle,
                lower: o.lower,
            }))
    }
    #[napi]
    pub fn batch(&mut self, high: Vec<f64>, low: Vec<f64>) -> napi::Result<Vec<f64>> {
        if high.len() != low.len() {
            return Err(NapiError::from_reason(
                "high and low must be equal length".to_string(),
            ));
        }
        let n = high.len();
        let mut out = vec![f64::NAN; n * 3];
        for i in 0..n {
            if let Some(o) = self.inner.update(cnd(high[i], low[i], low[i], 0.0)?) {
                out[i * 3] = o.upper;
                out[i * 3 + 1] = o.middle;
                out[i * 3 + 2] = o.lower;
            }
        }
        Ok(out)
    }
}

#[napi(js_name = "VWAP")]
pub struct VwapNode {
    inner: wc::Vwap,
}
impl Default for VwapNode {
    fn default() -> Self {
        Self::new()
    }
}
#[napi]
impl VwapNode {
    #[napi(constructor)]
    pub fn new() -> Self {
        Self {
            inner: wc::Vwap::new(),
        }
    }
    #[napi]
    pub fn reset(&mut self) {
        self.inner.reset();
    }
    #[napi(js_name = "isReady")]
    pub fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    #[napi(js_name = "warmupPeriod")]
    pub fn warmup_period(&self) -> u32 {
        self.inner.warmup_period() as u32
    }
    #[napi]
    pub fn update(
        &mut self,
        high: f64,
        low: f64,
        close: f64,
        volume: f64,
    ) -> napi::Result<Option<f64>> {
        Ok(self.inner.update(cnd(high, low, close, volume)?))
    }
    #[napi]
    pub fn batch(
        &mut self,
        high: Vec<f64>,
        low: Vec<f64>,
        close: Vec<f64>,
        volume: Vec<f64>,
    ) -> napi::Result<Vec<f64>> {
        if high.len() != low.len() || low.len() != close.len() || close.len() != volume.len() {
            return Err(NapiError::from_reason(
                "high, low, close, volume must be equal length".to_string(),
            ));
        }
        let mut out = Vec::with_capacity(high.len());
        for i in 0..high.len() {
            out.push(
                self.inner
                    .update(cnd(high[i], low[i], close[i], volume[i])?)
                    .unwrap_or(f64::NAN),
            );
        }
        Ok(out)
    }
}

#[napi(js_name = "AwesomeOscillator")]
pub struct AoNode {
    inner: wc::AwesomeOscillator,
}
#[napi]
impl AoNode {
    #[napi(constructor)]
    pub fn new(fast: u32, slow: u32) -> napi::Result<Self> {
        Ok(Self {
            inner: wc::AwesomeOscillator::new(fast as usize, slow as usize).map_err(map_err)?,
        })
    }
    #[napi]
    pub fn reset(&mut self) {
        self.inner.reset();
    }
    #[napi(js_name = "isReady")]
    pub fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    #[napi(js_name = "warmupPeriod")]
    pub fn warmup_period(&self) -> u32 {
        self.inner.warmup_period() as u32
    }
    #[napi]
    pub fn update(&mut self, high: f64, low: f64) -> napi::Result<Option<f64>> {
        Ok(self.inner.update(cnd(high, low, low, 0.0)?))
    }
    #[napi]
    pub fn batch(&mut self, high: Vec<f64>, low: Vec<f64>) -> napi::Result<Vec<f64>> {
        if high.len() != low.len() {
            return Err(NapiError::from_reason(
                "high and low must be equal length".to_string(),
            ));
        }
        let mut out = Vec::with_capacity(high.len());
        for i in 0..high.len() {
            out.push(
                self.inner
                    .update(cnd(high[i], low[i], low[i], 0.0)?)
                    .unwrap_or(f64::NAN),
            );
        }
        Ok(out)
    }
}

#[napi(object)]
pub struct AroonValue {
    pub up: f64,
    pub down: f64,
}

#[napi(js_name = "Aroon")]
pub struct AroonNode {
    inner: wc::Aroon,
}
#[napi]
impl AroonNode {
    #[napi(constructor)]
    pub fn new(period: u32) -> napi::Result<Self> {
        Ok(Self {
            inner: wc::Aroon::new(period as usize).map_err(map_err)?,
        })
    }
    #[napi]
    pub fn reset(&mut self) {
        self.inner.reset();
    }
    #[napi(js_name = "isReady")]
    pub fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    #[napi(js_name = "warmupPeriod")]
    pub fn warmup_period(&self) -> u32 {
        self.inner.warmup_period() as u32
    }
    #[napi]
    pub fn update(&mut self, high: f64, low: f64) -> napi::Result<Option<AroonValue>> {
        Ok(self
            .inner
            .update(cnd(high, low, low, 0.0)?)
            .map(|o| AroonValue {
                up: o.up,
                down: o.down,
            }))
    }
    #[napi]
    pub fn batch(&mut self, high: Vec<f64>, low: Vec<f64>) -> napi::Result<Vec<f64>> {
        if high.len() != low.len() {
            return Err(NapiError::from_reason(
                "high and low must be equal length".to_string(),
            ));
        }
        let n = high.len();
        let mut out = vec![f64::NAN; n * 2];
        for i in 0..n {
            if let Some(o) = self.inner.update(cnd(high[i], low[i], low[i], 0.0)?) {
                out[i * 2] = o.up;
                out[i * 2 + 1] = o.down;
            }
        }
        Ok(out)
    }
}

#[napi(js_name = "KAMA")]
pub struct KamaNode {
    inner: wc::Kama,
}
#[napi]
impl KamaNode {
    #[napi(constructor)]
    pub fn new(er_period: u32, fast: u32, slow: u32) -> napi::Result<Self> {
        Ok(Self {
            inner: wc::Kama::new(er_period as usize, fast as usize, slow as usize)
                .map_err(map_err)?,
        })
    }
    #[napi]
    pub fn reset(&mut self) {
        self.inner.reset();
    }
    #[napi(js_name = "isReady")]
    pub fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    #[napi(js_name = "warmupPeriod")]
    pub fn warmup_period(&self) -> u32 {
        self.inner.warmup_period() as u32
    }
    #[napi]
    pub fn update(&mut self, value: f64) -> Option<f64> {
        self.inner.update(value)
    }
    #[napi]
    pub fn batch(&mut self, prices: Vec<f64>) -> Vec<f64> {
        flatten(self.inner.batch(&prices))
    }
}

// ============================== T3 ==============================

#[napi(js_name = "T3")]
pub struct T3Node {
    inner: wc::T3,
}

#[napi]
impl T3Node {
    #[napi(constructor)]
    pub fn new(period: u32, v: f64) -> napi::Result<Self> {
        Ok(Self {
            inner: wc::T3::new(period as usize, v).map_err(map_err)?,
        })
    }
    #[napi]
    pub fn update(&mut self, value: f64) -> Option<f64> {
        self.inner.update(value)
    }
    #[napi]
    pub fn batch(&mut self, prices: Vec<f64>) -> Vec<f64> {
        flatten(self.inner.batch(&prices))
    }
    #[napi]
    pub fn reset(&mut self) {
        self.inner.reset();
    }
    #[napi(js_name = "isReady")]
    pub fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    #[napi(js_name = "warmupPeriod")]
    pub fn warmup_period(&self) -> u32 {
        self.inner.warmup_period() as u32
    }
}

// ============================== TSI ==============================

#[napi(js_name = "TSI")]
pub struct TsiNode {
    inner: wc::Tsi,
}

#[napi]
impl TsiNode {
    #[napi(constructor)]
    pub fn new(long: u32, short: u32) -> napi::Result<Self> {
        Ok(Self {
            inner: wc::Tsi::new(long as usize, short as usize).map_err(map_err)?,
        })
    }
    #[napi]
    pub fn update(&mut self, value: f64) -> Option<f64> {
        self.inner.update(value)
    }
    #[napi]
    pub fn batch(&mut self, prices: Vec<f64>) -> Vec<f64> {
        flatten(self.inner.batch(&prices))
    }
    #[napi]
    pub fn reset(&mut self) {
        self.inner.reset();
    }
    #[napi(js_name = "isReady")]
    pub fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    #[napi(js_name = "warmupPeriod")]
    pub fn warmup_period(&self) -> u32 {
        self.inner.warmup_period() as u32
    }
}

// ============================== PMO ==============================

#[napi(js_name = "PMO")]
pub struct PmoNode {
    inner: wc::Pmo,
}

#[napi]
impl PmoNode {
    #[napi(constructor)]
    pub fn new(smoothing1: u32, smoothing2: u32) -> napi::Result<Self> {
        Ok(Self {
            inner: wc::Pmo::new(smoothing1 as usize, smoothing2 as usize).map_err(map_err)?,
        })
    }
    #[napi]
    pub fn update(&mut self, value: f64) -> Option<f64> {
        self.inner.update(value)
    }
    #[napi]
    pub fn batch(&mut self, prices: Vec<f64>) -> Vec<f64> {
        flatten(self.inner.batch(&prices))
    }
    #[napi]
    pub fn reset(&mut self) {
        self.inner.reset();
    }
    #[napi(js_name = "isReady")]
    pub fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    #[napi(js_name = "warmupPeriod")]
    pub fn warmup_period(&self) -> u32 {
        self.inner.warmup_period() as u32
    }
}

// ============================== VWMA ==============================

// ============================== StochRSI ==============================

#[napi(js_name = "StochRSI")]
pub struct StochRsiNode {
    inner: wc::StochRsi,
}

#[napi]
impl StochRsiNode {
    #[napi(constructor)]
    pub fn new(rsi_period: u32, stoch_period: u32) -> napi::Result<Self> {
        Ok(Self {
            inner: wc::StochRsi::new(rsi_period as usize, stoch_period as usize)
                .map_err(map_err)?,
        })
    }
    #[napi]
    pub fn update(&mut self, value: f64) -> Option<f64> {
        self.inner.update(value)
    }
    #[napi]
    pub fn batch(&mut self, prices: Vec<f64>) -> Vec<f64> {
        flatten(self.inner.batch(&prices))
    }
    #[napi]
    pub fn reset(&mut self) {
        self.inner.reset();
    }
    #[napi(js_name = "isReady")]
    pub fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    #[napi(js_name = "warmupPeriod")]
    pub fn warmup_period(&self) -> u32 {
        self.inner.warmup_period() as u32
    }
}

// ============================== Ultimate Oscillator ==============================

#[napi(js_name = "UltimateOscillator")]
pub struct UltimateOscillatorNode {
    inner: wc::UltimateOscillator,
}

#[napi]
impl UltimateOscillatorNode {
    #[napi(constructor)]
    pub fn new(short: u32, mid: u32, long: u32) -> napi::Result<Self> {
        Ok(Self {
            inner: wc::UltimateOscillator::new(short as usize, mid as usize, long as usize)
                .map_err(map_err)?,
        })
    }
    #[napi]
    pub fn update(&mut self, high: f64, low: f64, close: f64) -> napi::Result<Option<f64>> {
        Ok(self.inner.update(cnd(high, low, close, 0.0)?))
    }
    #[napi]
    pub fn batch(
        &mut self,
        high: Vec<f64>,
        low: Vec<f64>,
        close: Vec<f64>,
    ) -> napi::Result<Vec<f64>> {
        if high.len() != low.len() || low.len() != close.len() {
            return Err(NapiError::from_reason(
                "high, low, close must be equal length".to_string(),
            ));
        }
        let mut out = Vec::with_capacity(high.len());
        for i in 0..high.len() {
            out.push(
                self.inner
                    .update(cnd(high[i], low[i], close[i], 0.0)?)
                    .unwrap_or(f64::NAN),
            );
        }
        Ok(out)
    }
    #[napi]
    pub fn reset(&mut self) {
        self.inner.reset();
    }
    #[napi(js_name = "isReady")]
    pub fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    #[napi(js_name = "warmupPeriod")]
    pub fn warmup_period(&self) -> u32 {
        self.inner.warmup_period() as u32
    }
}

#[napi(js_name = "VWMA")]
pub struct VwmaNode {
    inner: wc::Vwma,
}

#[napi]
impl VwmaNode {
    #[napi(constructor)]
    pub fn new(period: u32) -> napi::Result<Self> {
        Ok(Self {
            inner: wc::Vwma::new(period as usize).map_err(map_err)?,
        })
    }
    #[napi]
    pub fn update(&mut self, close: f64, volume: f64) -> napi::Result<Option<f64>> {
        Ok(self.inner.update(cnd(close, close, close, volume)?))
    }
    #[napi]
    pub fn batch(&mut self, close: Vec<f64>, volume: Vec<f64>) -> napi::Result<Vec<f64>> {
        if close.len() != volume.len() {
            return Err(NapiError::from_reason(
                "close and volume must be equal length".to_string(),
            ));
        }
        let mut out = Vec::with_capacity(close.len());
        for i in 0..close.len() {
            out.push(
                self.inner
                    .update(cnd(close[i], close[i], close[i], volume[i])?)
                    .unwrap_or(f64::NAN),
            );
        }
        Ok(out)
    }
    #[napi]
    pub fn reset(&mut self) {
        self.inner.reset();
    }
    #[napi(js_name = "isReady")]
    pub fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    #[napi(js_name = "warmupPeriod")]
    pub fn warmup_period(&self) -> u32 {
        self.inner.warmup_period() as u32
    }
}
