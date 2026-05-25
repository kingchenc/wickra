//! Node.js bindings for Wickra via napi-rs.
//!
//! Build with:
//! ```text
//! cd bindings/node && npm install && npm run build
//! ```
//!
//! Then `require("wickra")` from Node.

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
node_scalar_indicator!(DpoNode, "DPO", wc::Dpo);
node_scalar_indicator!(StdDevNode, "StdDev", wc::StdDev);
node_scalar_indicator!(UlcerIndexNode, "UlcerIndex", wc::UlcerIndex);
node_scalar_indicator!(
    VerticalHorizontalFilterNode,
    "VerticalHorizontalFilter",
    wc::VerticalHorizontalFilter
);
node_scalar_indicator!(ZScoreNode, "ZScore", wc::ZScore);

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

#[napi(js_name = "RollingVWAP")]
pub struct RollingVwapNode {
    inner: wc::RollingVwap,
}
#[napi]
impl RollingVwapNode {
    #[napi(constructor)]
    pub fn new(period: u32) -> napi::Result<Self> {
        Ok(Self {
            inner: wc::RollingVwap::new(period as usize).map_err(map_err)?,
        })
    }
    #[napi(getter)]
    pub fn period(&self) -> u32 {
        self.inner.period() as u32
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

// ============================== ADL ==============================

#[napi(js_name = "ADL")]
pub struct AdlNode {
    inner: wc::Adl,
}

impl Default for AdlNode {
    fn default() -> Self {
        Self::new()
    }
}

#[napi]
impl AdlNode {
    #[napi(constructor)]
    pub fn new() -> Self {
        Self {
            inner: wc::Adl::new(),
        }
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

// ============================== Volume-Price Trend ==============================

#[napi(js_name = "VolumePriceTrend")]
pub struct VolumePriceTrendNode {
    inner: wc::VolumePriceTrend,
}

impl Default for VolumePriceTrendNode {
    fn default() -> Self {
        Self::new()
    }
}

#[napi]
impl VolumePriceTrendNode {
    #[napi(constructor)]
    pub fn new() -> Self {
        Self {
            inner: wc::VolumePriceTrend::new(),
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

// ============================== Chaikin Money Flow ==============================

#[napi(js_name = "ChaikinMoneyFlow")]
pub struct ChaikinMoneyFlowNode {
    inner: wc::ChaikinMoneyFlow,
}

#[napi]
impl ChaikinMoneyFlowNode {
    #[napi(constructor)]
    pub fn new(period: u32) -> napi::Result<Self> {
        Ok(Self {
            inner: wc::ChaikinMoneyFlow::new(period as usize).map_err(map_err)?,
        })
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

// ============================== Chaikin Oscillator ==============================

#[napi(js_name = "ChaikinOscillator")]
pub struct ChaikinOscillatorNode {
    inner: wc::ChaikinOscillator,
}

#[napi]
impl ChaikinOscillatorNode {
    #[napi(constructor)]
    pub fn new(fast: u32, slow: u32) -> napi::Result<Self> {
        Ok(Self {
            inner: wc::ChaikinOscillator::new(fast as usize, slow as usize).map_err(map_err)?,
        })
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

// ============================== Force Index ==============================

#[napi(js_name = "ForceIndex")]
pub struct ForceIndexNode {
    inner: wc::ForceIndex,
}

#[napi]
impl ForceIndexNode {
    #[napi(constructor)]
    pub fn new(period: u32) -> napi::Result<Self> {
        Ok(Self {
            inner: wc::ForceIndex::new(period as usize).map_err(map_err)?,
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

// ============================== Ease of Movement ==============================

#[napi(js_name = "EaseOfMovement")]
pub struct EaseOfMovementNode {
    inner: wc::EaseOfMovement,
}

#[napi]
impl EaseOfMovementNode {
    #[napi(constructor)]
    pub fn new(period: u32, divisor: f64) -> napi::Result<Self> {
        Ok(Self {
            inner: wc::EaseOfMovement::with_divisor(period as usize, divisor).map_err(map_err)?,
        })
    }
    #[napi]
    pub fn update(&mut self, high: f64, low: f64, volume: f64) -> napi::Result<Option<f64>> {
        Ok(self.inner.update(cnd(high, low, low, volume)?))
    }
    #[napi]
    pub fn batch(
        &mut self,
        high: Vec<f64>,
        low: Vec<f64>,
        volume: Vec<f64>,
    ) -> napi::Result<Vec<f64>> {
        if high.len() != low.len() || low.len() != volume.len() {
            return Err(NapiError::from_reason(
                "high, low, volume must be equal length".to_string(),
            ));
        }
        let mut out = Vec::with_capacity(high.len());
        for i in 0..high.len() {
            out.push(
                self.inner
                    .update(cnd(high[i], low[i], low[i], volume[i])?)
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

// ============================== SuperTrend ==============================

#[napi(object)]
pub struct SuperTrendValue {
    pub value: f64,
    pub direction: f64,
}

#[napi(js_name = "SuperTrend")]
pub struct SuperTrendNode {
    inner: wc::SuperTrend,
}

#[napi]
impl SuperTrendNode {
    #[napi(constructor)]
    pub fn new(atr_period: u32, multiplier: f64) -> napi::Result<Self> {
        Ok(Self {
            inner: wc::SuperTrend::new(atr_period as usize, multiplier).map_err(map_err)?,
        })
    }
    #[napi]
    pub fn update(
        &mut self,
        high: f64,
        low: f64,
        close: f64,
    ) -> napi::Result<Option<SuperTrendValue>> {
        Ok(self
            .inner
            .update(cnd(high, low, close, 0.0)?)
            .map(|o| SuperTrendValue {
                value: o.value,
                direction: o.direction,
            }))
    }
    /// Returns `[value0, direction0, value1, direction1, ...]`, length `2 * n`.
    /// Warmup positions are `NaN`.
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
                out[i * 2] = o.value;
                out[i * 2 + 1] = o.direction;
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

// ============================== Chandelier Exit ==============================

#[napi(object)]
pub struct ChandelierExitValue {
    pub long_stop: f64,
    pub short_stop: f64,
}

#[napi(js_name = "ChandelierExit")]
pub struct ChandelierExitNode {
    inner: wc::ChandelierExit,
}

#[napi]
impl ChandelierExitNode {
    #[napi(constructor)]
    pub fn new(period: u32, multiplier: f64) -> napi::Result<Self> {
        Ok(Self {
            inner: wc::ChandelierExit::new(period as usize, multiplier).map_err(map_err)?,
        })
    }
    #[napi]
    pub fn update(
        &mut self,
        high: f64,
        low: f64,
        close: f64,
    ) -> napi::Result<Option<ChandelierExitValue>> {
        Ok(self
            .inner
            .update(cnd(high, low, close, 0.0)?)
            .map(|o| ChandelierExitValue {
                long_stop: o.long_stop,
                short_stop: o.short_stop,
            }))
    }
    /// Returns `[long0, short0, long1, short1, ...]`, length `2 * n`. Warmup
    /// positions are `NaN`.
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
                out[i * 2] = o.long_stop;
                out[i * 2 + 1] = o.short_stop;
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

// ============================== Chande Kroll Stop ==============================

#[napi(object)]
pub struct ChandeKrollStopValue {
    pub stop_long: f64,
    pub stop_short: f64,
}

#[napi(js_name = "ChandeKrollStop")]
pub struct ChandeKrollStopNode {
    inner: wc::ChandeKrollStop,
}

#[napi]
impl ChandeKrollStopNode {
    #[napi(constructor)]
    pub fn new(atr_period: u32, atr_multiplier: f64, stop_period: u32) -> napi::Result<Self> {
        Ok(Self {
            inner: wc::ChandeKrollStop::new(
                atr_period as usize,
                atr_multiplier,
                stop_period as usize,
            )
            .map_err(map_err)?,
        })
    }
    #[napi]
    pub fn update(
        &mut self,
        high: f64,
        low: f64,
        close: f64,
    ) -> napi::Result<Option<ChandeKrollStopValue>> {
        Ok(self
            .inner
            .update(cnd(high, low, close, 0.0)?)
            .map(|o| ChandeKrollStopValue {
                stop_long: o.stop_long,
                stop_short: o.stop_short,
            }))
    }
    /// Returns `[long0, short0, long1, short1, ...]`, length `2 * n`. Warmup
    /// positions are `NaN`.
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
                out[i * 2] = o.stop_long;
                out[i * 2 + 1] = o.stop_short;
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

// ============================== ATR Trailing Stop ==============================

#[napi(js_name = "AtrTrailingStop")]
pub struct AtrTrailingStopNode {
    inner: wc::AtrTrailingStop,
}

#[napi]
impl AtrTrailingStopNode {
    #[napi(constructor)]
    pub fn new(atr_period: u32, multiplier: f64) -> napi::Result<Self> {
        Ok(Self {
            inner: wc::AtrTrailingStop::new(atr_period as usize, multiplier).map_err(map_err)?,
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

// ============================== Typical Price ==============================

#[napi(js_name = "TypicalPrice")]
pub struct TypicalPriceNode {
    inner: wc::TypicalPrice,
}

impl Default for TypicalPriceNode {
    fn default() -> Self {
        Self::new()
    }
}

#[napi]
impl TypicalPriceNode {
    #[napi(constructor)]
    pub fn new() -> Self {
        Self {
            inner: wc::TypicalPrice::new(),
        }
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

// ============================== Median Price ==============================

#[napi(js_name = "MedianPrice")]
pub struct MedianPriceNode {
    inner: wc::MedianPrice,
}

impl Default for MedianPriceNode {
    fn default() -> Self {
        Self::new()
    }
}

#[napi]
impl MedianPriceNode {
    #[napi(constructor)]
    pub fn new() -> Self {
        Self {
            inner: wc::MedianPrice::new(),
        }
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

// ============================== Weighted Close ==============================

#[napi(js_name = "WeightedClose")]
pub struct WeightedCloseNode {
    inner: wc::WeightedClose,
}

impl Default for WeightedCloseNode {
    fn default() -> Self {
        Self::new()
    }
}

#[napi]
impl WeightedCloseNode {
    #[napi(constructor)]
    pub fn new() -> Self {
        Self {
            inner: wc::WeightedClose::new(),
        }
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

// ============================== Linear Regression ==============================

#[napi(js_name = "LinearRegression")]
pub struct LinearRegressionNode {
    inner: wc::LinearRegression,
}

#[napi]
impl LinearRegressionNode {
    #[napi(constructor)]
    pub fn new(period: u32) -> napi::Result<Self> {
        Ok(Self {
            inner: wc::LinearRegression::new(period as usize).map_err(map_err)?,
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

// ============================== Linear Regression Slope ==============================

#[napi(js_name = "LinRegSlope")]
pub struct LinRegSlopeNode {
    inner: wc::LinRegSlope,
}

#[napi]
impl LinRegSlopeNode {
    #[napi(constructor)]
    pub fn new(period: u32) -> napi::Result<Self> {
        Ok(Self {
            inner: wc::LinRegSlope::new(period as usize).map_err(map_err)?,
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

// ============================== Accelerator Oscillator ==============================

#[napi(js_name = "AcceleratorOscillator")]
pub struct AcceleratorOscillatorNode {
    inner: wc::AcceleratorOscillator,
}

#[napi]
impl AcceleratorOscillatorNode {
    #[napi(constructor)]
    pub fn new(ao_fast: u32, ao_slow: u32, signal_period: u32) -> napi::Result<Self> {
        Ok(Self {
            inner: wc::AcceleratorOscillator::new(
                ao_fast as usize,
                ao_slow as usize,
                signal_period as usize,
            )
            .map_err(map_err)?,
        })
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

// ============================== Balance of Power ==============================

#[napi(js_name = "BalanceOfPower")]
pub struct BalanceOfPowerNode {
    inner: wc::BalanceOfPower,
}

impl Default for BalanceOfPowerNode {
    fn default() -> Self {
        Self::new()
    }
}

#[napi]
impl BalanceOfPowerNode {
    #[napi(constructor)]
    pub fn new() -> Self {
        Self {
            inner: wc::BalanceOfPower::new(),
        }
    }
    #[napi]
    pub fn update(
        &mut self,
        open: f64,
        high: f64,
        low: f64,
        close: f64,
    ) -> napi::Result<Option<f64>> {
        let candle = wc::Candle::new(open, high, low, close, 0.0, 0).map_err(map_err)?;
        Ok(self.inner.update(candle))
    }
    #[napi]
    pub fn batch(
        &mut self,
        open: Vec<f64>,
        high: Vec<f64>,
        low: Vec<f64>,
        close: Vec<f64>,
    ) -> napi::Result<Vec<f64>> {
        if open.len() != high.len() || high.len() != low.len() || low.len() != close.len() {
            return Err(NapiError::from_reason(
                "open, high, low, close must be equal length".to_string(),
            ));
        }
        let mut out = Vec::with_capacity(open.len());
        for i in 0..open.len() {
            let candle =
                wc::Candle::new(open[i], high[i], low[i], close[i], 0.0, 0).map_err(map_err)?;
            out.push(self.inner.update(candle).unwrap_or(f64::NAN));
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

// ============================== Choppiness Index ==============================

#[napi(js_name = "ChoppinessIndex")]
pub struct ChoppinessIndexNode {
    inner: wc::ChoppinessIndex,
}

#[napi]
impl ChoppinessIndexNode {
    #[napi(constructor)]
    pub fn new(period: u32) -> napi::Result<Self> {
        Ok(Self {
            inner: wc::ChoppinessIndex::new(period as usize).map_err(map_err)?,
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

// ============================== True Range ==============================

#[napi(js_name = "TrueRange")]
pub struct TrueRangeNode {
    inner: wc::TrueRange,
}

impl Default for TrueRangeNode {
    fn default() -> Self {
        Self::new()
    }
}

#[napi]
impl TrueRangeNode {
    #[napi(constructor)]
    pub fn new() -> Self {
        Self {
            inner: wc::TrueRange::new(),
        }
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

// ============================== Chaikin Volatility ==============================

#[napi(js_name = "ChaikinVolatility")]
pub struct ChaikinVolatilityNode {
    inner: wc::ChaikinVolatility,
}

#[napi]
impl ChaikinVolatilityNode {
    #[napi(constructor)]
    pub fn new(ema_period: u32, roc_period: u32) -> napi::Result<Self> {
        Ok(Self {
            inner: wc::ChaikinVolatility::new(ema_period as usize, roc_period as usize)
                .map_err(map_err)?,
        })
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

// ============================== Linear Regression Angle ==============================

#[napi(js_name = "LinRegAngle")]
pub struct LinRegAngleNode {
    inner: wc::LinRegAngle,
}

#[napi]
impl LinRegAngleNode {
    #[napi(constructor)]
    pub fn new(period: u32) -> napi::Result<Self> {
        Ok(Self {
            inner: wc::LinRegAngle::new(period as usize).map_err(map_err)?,
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

// ============================== Bollinger Bandwidth ==============================

#[napi(js_name = "BollingerBandwidth")]
pub struct BollingerBandwidthNode {
    inner: wc::BollingerBandwidth,
}

#[napi]
impl BollingerBandwidthNode {
    #[napi(constructor)]
    pub fn new(period: u32, multiplier: f64) -> napi::Result<Self> {
        Ok(Self {
            inner: wc::BollingerBandwidth::new(period as usize, multiplier).map_err(map_err)?,
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

// ============================== Percent B ==============================

#[napi(js_name = "PercentB")]
pub struct PercentBNode {
    inner: wc::PercentB,
}

#[napi]
impl PercentBNode {
    #[napi(constructor)]
    pub fn new(period: u32, multiplier: f64) -> napi::Result<Self> {
        Ok(Self {
            inner: wc::PercentB::new(period as usize, multiplier).map_err(map_err)?,
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

// ============================== NATR ==============================

#[napi(js_name = "NATR")]
pub struct NatrNode {
    inner: wc::Natr,
}

#[napi]
impl NatrNode {
    #[napi(constructor)]
    pub fn new(period: u32) -> napi::Result<Self> {
        Ok(Self {
            inner: wc::Natr::new(period as usize).map_err(map_err)?,
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

// ============================== Historical Volatility ==============================

#[napi(js_name = "HistoricalVolatility")]
pub struct HistoricalVolatilityNode {
    inner: wc::HistoricalVolatility,
}

#[napi]
impl HistoricalVolatilityNode {
    #[napi(constructor)]
    pub fn new(period: u32, trading_periods: u32) -> napi::Result<Self> {
        Ok(Self {
            inner: wc::HistoricalVolatility::new(period as usize, trading_periods as usize)
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

// ============================== Aroon Oscillator ==============================

#[napi(js_name = "AroonOscillator")]
pub struct AroonOscillatorNode {
    inner: wc::AroonOscillator,
}

#[napi]
impl AroonOscillatorNode {
    #[napi(constructor)]
    pub fn new(period: u32) -> napi::Result<Self> {
        Ok(Self {
            inner: wc::AroonOscillator::new(period as usize).map_err(map_err)?,
        })
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

// ============================== Vortex ==============================

/// Vortex Indicator pair: `VI+` and `VI-`.
#[napi(object)]
pub struct VortexValue {
    pub plus: f64,
    pub minus: f64,
}

#[napi(js_name = "Vortex")]
pub struct VortexNode {
    inner: wc::Vortex,
}

#[napi]
impl VortexNode {
    #[napi(constructor)]
    pub fn new(period: u32) -> napi::Result<Self> {
        Ok(Self {
            inner: wc::Vortex::new(period as usize).map_err(map_err)?,
        })
    }
    #[napi]
    pub fn update(&mut self, high: f64, low: f64, close: f64) -> napi::Result<Option<VortexValue>> {
        Ok(self
            .inner
            .update(cnd(high, low, close, 0.0)?)
            .map(|o| VortexValue {
                plus: o.plus,
                minus: o.minus,
            }))
    }
    /// Returns `[plus0, minus0, plus1, minus1, ...]`, length `2 * n`. Warmup is NaN.
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
                out[i * 2] = o.plus;
                out[i * 2 + 1] = o.minus;
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

// ============================== Mass Index ==============================

#[napi(js_name = "MassIndex")]
pub struct MassIndexNode {
    inner: wc::MassIndex,
}

#[napi]
impl MassIndexNode {
    #[napi(constructor)]
    pub fn new(ema_period: u32, sum_period: u32) -> napi::Result<Self> {
        Ok(Self {
            inner: wc::MassIndex::new(ema_period as usize, sum_period as usize).map_err(map_err)?,
        })
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

// ============================== PPO ==============================

#[napi(js_name = "PPO")]
pub struct PpoNode {
    inner: wc::Ppo,
}

#[napi]
impl PpoNode {
    #[napi(constructor)]
    pub fn new(fast: u32, slow: u32) -> napi::Result<Self> {
        Ok(Self {
            inner: wc::Ppo::new(fast as usize, slow as usize).map_err(map_err)?,
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

// ============================== Coppock ==============================

#[napi(js_name = "Coppock")]
pub struct CoppockNode {
    inner: wc::Coppock,
}

#[napi]
impl CoppockNode {
    #[napi(constructor)]
    pub fn new(roc_long: u32, roc_short: u32, wma_period: u32) -> napi::Result<Self> {
        Ok(Self {
            inner: wc::Coppock::new(roc_long as usize, roc_short as usize, wma_period as usize)
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
// ============================== Pivots & S/R ==============================

#[napi(object)]
pub struct ClassicPivotsValue {
    pub pp: f64,
    pub r1: f64,
    pub r2: f64,
    pub r3: f64,
    pub s1: f64,
    pub s2: f64,
    pub s3: f64,
}

#[napi(js_name = "ClassicPivots")]
pub struct ClassicPivotsNode {
    inner: wc::ClassicPivots,
}

#[napi]
impl ClassicPivotsNode {
    #[napi(constructor)]
    pub fn new() -> Self {
        Self {
            inner: wc::ClassicPivots::new(),
        }
    }
    #[napi]
    pub fn update(
        &mut self,
        high: f64,
        low: f64,
        close: f64,
    ) -> napi::Result<Option<ClassicPivotsValue>> {
        Ok(self
            .inner
            .update(cnd(high, low, close, 0.0)?)
            .map(|o| ClassicPivotsValue {
                pp: o.pp,
                r1: o.r1,
                r2: o.r2,
                r3: o.r3,
                s1: o.s1,
                s2: o.s2,
                s3: o.s3,
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
        let mut out = vec![f64::NAN; n * 7];
        for i in 0..n {
            if let Some(o) = self.inner.update(cnd(high[i], low[i], close[i], 0.0)?) {
                out[i * 7] = o.pp;
                out[i * 7 + 1] = o.r1;
                out[i * 7 + 2] = o.r2;
                out[i * 7 + 3] = o.r3;
                out[i * 7 + 4] = o.s1;
                out[i * 7 + 5] = o.s2;
                out[i * 7 + 6] = o.s3;
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

impl Default for ClassicPivotsNode {
    fn default() -> Self {
        Self::new()
    }
}

#[napi(object)]
pub struct FibonacciPivotsValue {
    pub pp: f64,
    pub r1: f64,
    pub r2: f64,
    pub r3: f64,
    pub s1: f64,
    pub s2: f64,
    pub s3: f64,
}

#[napi(js_name = "FibonacciPivots")]
pub struct FibonacciPivotsNode {
    inner: wc::FibonacciPivots,
}

#[napi]
impl FibonacciPivotsNode {
    #[napi(constructor)]
    pub fn new() -> Self {
        Self {
            inner: wc::FibonacciPivots::new(),
        }
    }
    #[napi]
    pub fn update(
        &mut self,
        high: f64,
        low: f64,
        close: f64,
    ) -> napi::Result<Option<FibonacciPivotsValue>> {
        Ok(self
            .inner
            .update(cnd(high, low, close, 0.0)?)
            .map(|o| FibonacciPivotsValue {
                pp: o.pp,
                r1: o.r1,
                r2: o.r2,
                r3: o.r3,
                s1: o.s1,
                s2: o.s2,
                s3: o.s3,
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
        let mut out = vec![f64::NAN; n * 7];
        for i in 0..n {
            if let Some(o) = self.inner.update(cnd(high[i], low[i], close[i], 0.0)?) {
                out[i * 7] = o.pp;
                out[i * 7 + 1] = o.r1;
                out[i * 7 + 2] = o.r2;
                out[i * 7 + 3] = o.r3;
                out[i * 7 + 4] = o.s1;
                out[i * 7 + 5] = o.s2;
                out[i * 7 + 6] = o.s3;
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

impl Default for FibonacciPivotsNode {
    fn default() -> Self {
        Self::new()
    }
}

#[napi(object)]
pub struct CamarillaValue {
    pub pp: f64,
    pub r1: f64,
    pub r2: f64,
    pub r3: f64,
    pub r4: f64,
    pub s1: f64,
    pub s2: f64,
    pub s3: f64,
    pub s4: f64,
}

#[napi(js_name = "Camarilla")]
pub struct CamarillaNode {
    inner: wc::Camarilla,
}

#[napi]
impl CamarillaNode {
    #[napi(constructor)]
    pub fn new() -> Self {
        Self {
            inner: wc::Camarilla::new(),
        }
    }
    #[napi]
    pub fn update(
        &mut self,
        high: f64,
        low: f64,
        close: f64,
    ) -> napi::Result<Option<CamarillaValue>> {
        Ok(self
            .inner
            .update(cnd(high, low, close, 0.0)?)
            .map(|o| CamarillaValue {
                pp: o.pp,
                r1: o.r1,
                r2: o.r2,
                r3: o.r3,
                r4: o.r4,
                s1: o.s1,
                s2: o.s2,
                s3: o.s3,
                s4: o.s4,
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
        let mut out = vec![f64::NAN; n * 9];
        for i in 0..n {
            if let Some(o) = self.inner.update(cnd(high[i], low[i], close[i], 0.0)?) {
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

impl Default for CamarillaNode {
    fn default() -> Self {
        Self::new()
    }
}

#[napi(object)]
pub struct WoodiePivotsValue {
    pub pp: f64,
    pub r1: f64,
    pub r2: f64,
    pub s1: f64,
    pub s2: f64,
}

#[napi(js_name = "WoodiePivots")]
pub struct WoodiePivotsNode {
    inner: wc::WoodiePivots,
}

#[napi]
impl WoodiePivotsNode {
    #[napi(constructor)]
    pub fn new() -> Self {
        Self {
            inner: wc::WoodiePivots::new(),
        }
    }
    #[napi]
    pub fn update(
        &mut self,
        high: f64,
        low: f64,
        close: f64,
    ) -> napi::Result<Option<WoodiePivotsValue>> {
        Ok(self
            .inner
            .update(cnd(high, low, close, 0.0)?)
            .map(|o| WoodiePivotsValue {
                pp: o.pp,
                r1: o.r1,
                r2: o.r2,
                s1: o.s1,
                s2: o.s2,
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
        let mut out = vec![f64::NAN; n * 5];
        for i in 0..n {
            if let Some(o) = self.inner.update(cnd(high[i], low[i], close[i], 0.0)?) {
                out[i * 5] = o.pp;
                out[i * 5 + 1] = o.r1;
                out[i * 5 + 2] = o.r2;
                out[i * 5 + 3] = o.s1;
                out[i * 5 + 4] = o.s2;
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

impl Default for WoodiePivotsNode {
    fn default() -> Self {
        Self::new()
    }
}

#[napi(object)]
pub struct DemarkPivotsValue {
    pub pp: f64,
    pub r1: f64,
    pub s1: f64,
}

#[napi(js_name = "DemarkPivots")]
pub struct DemarkPivotsNode {
    inner: wc::DemarkPivots,
}

#[napi]
impl DemarkPivotsNode {
    #[napi(constructor)]
    pub fn new() -> Self {
        Self {
            inner: wc::DemarkPivots::new(),
        }
    }
    #[napi]
    pub fn update(
        &mut self,
        open: f64,
        high: f64,
        low: f64,
        close: f64,
    ) -> napi::Result<Option<DemarkPivotsValue>> {
        let candle = wc::Candle::new(open, high, low, close, 0.0, 0).map_err(map_err)?;
        Ok(self.inner.update(candle).map(|o| DemarkPivotsValue {
            pp: o.pp,
            r1: o.r1,
            s1: o.s1,
        }))
    }
    #[napi]
    pub fn batch(
        &mut self,
        open: Vec<f64>,
        high: Vec<f64>,
        low: Vec<f64>,
        close: Vec<f64>,
    ) -> napi::Result<Vec<f64>> {
        if open.len() != high.len() || high.len() != low.len() || low.len() != close.len() {
            return Err(NapiError::from_reason(
                "open, high, low, close must be equal length".to_string(),
            ));
        }
        let n = open.len();
        let mut out = vec![f64::NAN; n * 3];
        for i in 0..n {
            let candle =
                wc::Candle::new(open[i], high[i], low[i], close[i], 0.0, 0).map_err(map_err)?;
            if let Some(o) = self.inner.update(candle) {
                out[i * 3] = o.pp;
                out[i * 3 + 1] = o.r1;
                out[i * 3 + 2] = o.s1;
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

impl Default for DemarkPivotsNode {
    fn default() -> Self {
        Self::new()
    }
}

#[napi(object)]
pub struct WilliamsFractalsValue {
    /// Up fractal price; NaN when no up fractal was confirmed on this bar.
    pub up: f64,
    /// Down fractal price; NaN when no down fractal was confirmed on this bar.
    pub down: f64,
}

#[napi(js_name = "WilliamsFractals")]
pub struct WilliamsFractalsNode {
    inner: wc::WilliamsFractals,
}

#[napi]
impl WilliamsFractalsNode {
    #[napi(constructor)]
    pub fn new() -> Self {
        Self {
            inner: wc::WilliamsFractals::new(),
        }
    }
    #[napi]
    pub fn update(&mut self, high: f64, low: f64) -> napi::Result<Option<WilliamsFractalsValue>> {
        Ok(self
            .inner
            .update(cnd(high, low, low, 0.0)?)
            .map(|o| WilliamsFractalsValue {
                up: o.up.unwrap_or(f64::NAN),
                down: o.down.unwrap_or(f64::NAN),
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
                if let Some(v) = o.up {
                    out[i * 2] = v;
                }
                if let Some(v) = o.down {
                    out[i * 2 + 1] = v;
                }
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

impl Default for WilliamsFractalsNode {
    fn default() -> Self {
        Self::new()
    }
}

#[napi(object)]
pub struct ZigZagValue {
    pub swing: f64,
    pub direction: f64,
}

#[napi(js_name = "ZigZag")]
pub struct ZigZagNode {
    inner: wc::ZigZag,
}

#[napi]
impl ZigZagNode {
    #[napi(constructor)]
    pub fn new(threshold: f64) -> napi::Result<Self> {
        Ok(Self {
            inner: wc::ZigZag::new(threshold).map_err(map_err)?,
        })
    }
    #[napi]
    pub fn update(&mut self, high: f64, low: f64) -> napi::Result<Option<ZigZagValue>> {
        Ok(self
            .inner
            .update(cnd(high, low, low, 0.0)?)
            .map(|o| ZigZagValue {
                swing: o.swing,
                direction: o.direction,
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
                out[i * 2] = o.swing;
                out[i * 2 + 1] = o.direction;
            }
        }
        Ok(out)
    }
    #[napi(getter)]
    pub fn threshold(&self) -> f64 {
        self.inner.threshold()
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
