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
node_scalar_indicator!(McGinleyDynamicNode, "McGinleyDynamic", wc::McGinleyDynamic);
node_scalar_indicator!(FramaNode, "FRAMA", wc::Frama);

// RviVolatility (Relative Volatility Index, Donald Dorsey). Disambiguated
// from `RVI` = Relative Vigor Index in Family 02. Takes a single `period`
// parameter and additionally rejects `period == 1` (a 1-bar standard
// deviation is always zero), so the `clamp_period`-to-1 strategy from
// `node_scalar_indicator!` would panic via `must`. Hand-rolled fallible
// constructor instead, throws a JS error on bad period.
#[napi(js_name = "RVIVolatility")]
pub struct RviVolatilityNode {
    inner: wc::RviVolatility,
}

#[napi]
impl RviVolatilityNode {
    #[napi(constructor)]
    pub fn new(period: u32) -> napi::Result<Self> {
        Ok(Self {
            inner: wc::RviVolatility::new(period as usize).map_err(map_err)?,
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

// Helper for OHLC-input indicators that need the open price (not provided
// by `cnd` which fakes open == close).
fn cnd4(open: f64, high: f64, low: f64, close: f64) -> napi::Result<wc::Candle> {
    wc::Candle::new(open, high, low, close, 0.0, 0).map_err(map_err)
}

#[napi(js_name = "Inertia")]
pub struct InertiaNode {
    inner: wc::Inertia,
}
#[napi]
impl InertiaNode {
    #[napi(constructor)]
    pub fn new(rvi_period: u32, linreg_period: u32) -> napi::Result<Self> {
        Ok(Self {
            inner: wc::Inertia::new(clamp_period(rvi_period), clamp_period(linreg_period))
                .map_err(map_err)?,
        })
    }
    #[napi]
    pub fn update(
        &mut self,
        open: f64,
        high: f64,
        low: f64,
        close: f64,
    ) -> napi::Result<Option<f64>> {
        Ok(self.inner.update(cnd4(open, high, low, close)?))
    }
    #[napi]
    pub fn batch(
        &mut self,
        open: Vec<f64>,
        high: Vec<f64>,
        low: Vec<f64>,
        close: Vec<f64>,
    ) -> napi::Result<Vec<f64>> {
        if !(open.len() == high.len() && high.len() == low.len() && low.len() == close.len()) {
            return Err(NapiError::from_reason(
                "open, high, low and close must be equal length".to_string(),
            ));
        }
        let mut out = Vec::with_capacity(close.len());
        for i in 0..close.len() {
            out.push(
                self.inner
                    .update(cnd4(open[i], high[i], low[i], close[i])?)
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

#[napi(js_name = "ConnorsRSI")]
pub struct ConnorsRsiNode {
    inner: wc::ConnorsRsi,
}
#[napi]
impl ConnorsRsiNode {
    #[napi(constructor)]
    pub fn new(period_rsi: u32, period_streak: u32, period_rank: u32) -> napi::Result<Self> {
        Ok(Self {
            inner: wc::ConnorsRsi::new(
                clamp_period(period_rsi),
                clamp_period(period_streak),
                clamp_period(period_rank),
            )
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

#[napi(js_name = "LaguerreRSI")]
pub struct LaguerreRsiNode {
    inner: wc::LaguerreRsi,
}
#[napi]
impl LaguerreRsiNode {
    #[napi(constructor)]
    pub fn new(gamma: f64) -> napi::Result<Self> {
        Ok(Self {
            inner: wc::LaguerreRsi::new(gamma).map_err(map_err)?,
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

#[napi(js_name = "SMI")]
pub struct SmiNode {
    inner: wc::Smi,
}
#[napi]
impl SmiNode {
    #[napi(constructor)]
    pub fn new(period: u32, d_period: u32, d2_period: u32) -> napi::Result<Self> {
        Ok(Self {
            inner: wc::Smi::new(
                clamp_period(period),
                clamp_period(d_period),
                clamp_period(d2_period),
            )
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
        if !(high.len() == low.len() && low.len() == close.len()) {
            return Err(NapiError::from_reason(
                "high, low and close must be equal length".to_string(),
            ));
        }
        let mut out = Vec::with_capacity(close.len());
        for i in 0..close.len() {
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
pub struct KstValue {
    pub kst: f64,
    pub signal: f64,
}

#[napi(js_name = "KST")]
pub struct KstNode {
    inner: wc::Kst,
}
#[napi]
impl KstNode {
    #[napi(constructor)]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        roc1: u32,
        roc2: u32,
        roc3: u32,
        roc4: u32,
        sma1: u32,
        sma2: u32,
        sma3: u32,
        sma4: u32,
        signal: u32,
    ) -> napi::Result<Self> {
        Ok(Self {
            inner: wc::Kst::new(
                clamp_period(roc1),
                clamp_period(roc2),
                clamp_period(roc3),
                clamp_period(roc4),
                clamp_period(sma1),
                clamp_period(sma2),
                clamp_period(sma3),
                clamp_period(sma4),
                clamp_period(signal),
            )
            .map_err(map_err)?,
        })
    }
    #[napi]
    pub fn update(&mut self, value: f64) -> Option<KstValue> {
        self.inner.update(value).map(|o| KstValue {
            kst: o.kst,
            signal: o.signal,
        })
    }
    #[napi]
    pub fn batch(&mut self, prices: Vec<f64>) -> Vec<f64> {
        let n = prices.len();
        let mut out = vec![f64::NAN; n * 2];
        for (i, p) in prices.iter().enumerate() {
            if let Some(o) = self.inner.update(*p) {
                out[i * 2] = o.kst;
                out[i * 2 + 1] = o.signal;
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

#[napi(js_name = "PGO")]
pub struct PgoNode {
    inner: wc::Pgo,
}
#[napi]
impl PgoNode {
    #[napi(constructor)]
    pub fn new(period: u32) -> napi::Result<Self> {
        Ok(Self {
            inner: wc::Pgo::new(clamp_period(period)).map_err(map_err)?,
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
        if !(high.len() == low.len() && low.len() == close.len()) {
            return Err(NapiError::from_reason(
                "high, low and close must be equal length".to_string(),
            ));
        }
        let mut out = Vec::with_capacity(close.len());
        for i in 0..close.len() {
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

#[napi(js_name = "RVI")]
pub struct RviNode {
    inner: wc::Rvi,
}
#[napi]
impl RviNode {
    #[napi(constructor)]
    pub fn new(period: u32) -> napi::Result<Self> {
        Ok(Self {
            inner: wc::Rvi::new(clamp_period(period)).map_err(map_err)?,
        })
    }
    #[napi]
    pub fn update(
        &mut self,
        open: f64,
        high: f64,
        low: f64,
        close: f64,
    ) -> napi::Result<Option<f64>> {
        Ok(self.inner.update(cnd4(open, high, low, close)?))
    }
    #[napi]
    pub fn batch(
        &mut self,
        open: Vec<f64>,
        high: Vec<f64>,
        low: Vec<f64>,
        close: Vec<f64>,
    ) -> napi::Result<Vec<f64>> {
        if !(open.len() == high.len() && high.len() == low.len() && low.len() == close.len()) {
            return Err(NapiError::from_reason(
                "open, high, low and close must be equal length".to_string(),
            ));
        }
        let mut out = Vec::with_capacity(close.len());
        for i in 0..close.len() {
            out.push(
                self.inner
                    .update(cnd4(open[i], high[i], low[i], close[i])?)
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

#[napi(js_name = "AwesomeOscillatorHistogram")]
pub struct AwesomeOscillatorHistogramNode {
    inner: wc::AwesomeOscillatorHistogram,
}
#[napi]
impl AwesomeOscillatorHistogramNode {
    #[napi(constructor)]
    pub fn new(fast: u32, slow: u32, sma_period: u32) -> napi::Result<Self> {
        Ok(Self {
            inner: wc::AwesomeOscillatorHistogram::new(
                clamp_period(fast),
                clamp_period(slow),
                clamp_period(sma_period),
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

#[napi(js_name = "STC")]
pub struct StcNode {
    inner: wc::Stc,
}
#[napi]
impl StcNode {
    #[napi(constructor)]
    pub fn new(fast: u32, slow: u32, schaff_period: u32, factor: f64) -> napi::Result<Self> {
        Ok(Self {
            inner: wc::Stc::new(
                clamp_period(fast),
                clamp_period(slow),
                clamp_period(schaff_period),
                factor,
            )
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

#[napi(js_name = "ElderImpulse")]
pub struct ElderImpulseNode {
    inner: wc::ElderImpulse,
}
#[napi]
impl ElderImpulseNode {
    #[napi(constructor)]
    pub fn new(
        ema_period: u32,
        macd_fast: u32,
        macd_slow: u32,
        macd_signal: u32,
    ) -> napi::Result<Self> {
        Ok(Self {
            inner: wc::ElderImpulse::new(
                clamp_period(ema_period),
                clamp_period(macd_fast),
                clamp_period(macd_slow),
                clamp_period(macd_signal),
            )
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

#[napi(object)]
pub struct ZeroLagMacdValue {
    pub macd: f64,
    pub signal: f64,
    pub histogram: f64,
}

#[napi(js_name = "ZeroLagMACD")]
pub struct ZeroLagMacdNode {
    inner: wc::ZeroLagMacd,
}
#[napi]
impl ZeroLagMacdNode {
    #[napi(constructor)]
    pub fn new(fast: u32, slow: u32, signal: u32) -> napi::Result<Self> {
        Ok(Self {
            inner: wc::ZeroLagMacd::new(
                clamp_period(fast),
                clamp_period(slow),
                clamp_period(signal),
            )
            .map_err(map_err)?,
        })
    }
    #[napi]
    pub fn update(&mut self, value: f64) -> Option<ZeroLagMacdValue> {
        self.inner.update(value).map(|o| ZeroLagMacdValue {
            macd: o.macd,
            signal: o.signal,
            histogram: o.histogram,
        })
    }
    #[napi]
    pub fn batch(&mut self, prices: Vec<f64>) -> Vec<f64> {
        let n = prices.len();
        let mut out = vec![f64::NAN; n * 3];
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

#[napi(js_name = "CFO")]
pub struct CfoNode {
    inner: wc::Cfo,
}
#[napi]
impl CfoNode {
    #[napi(constructor)]
    pub fn new(period: u32) -> napi::Result<Self> {
        Ok(Self {
            inner: wc::Cfo::new(clamp_period(period)).map_err(map_err)?,
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

#[napi(js_name = "APO")]
pub struct ApoNode {
    inner: wc::Apo,
}
#[napi]
impl ApoNode {
    #[napi(constructor)]
    pub fn new(fast: u32, slow: u32) -> napi::Result<Self> {
        Ok(Self {
            inner: wc::Apo::new(clamp_period(fast), clamp_period(slow)).map_err(map_err)?,
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

// ============================== EVWMA ==============================

#[napi(js_name = "EVWMA")]
pub struct EvwmaNode {
    inner: wc::Evwma,
}
#[napi]
impl EvwmaNode {
    #[napi(constructor)]
    pub fn new(period: u32) -> napi::Result<Self> {
        Ok(Self {
            inner: wc::Evwma::new(clamp_period(period)).map_err(map_err)?,
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

// ============================== Alligator ==============================

#[napi(object)]
pub struct AlligatorValue {
    pub jaw: f64,
    pub teeth: f64,
    pub lips: f64,
}

#[napi(js_name = "Alligator")]
pub struct AlligatorNode {
    inner: wc::Alligator,
}
#[napi]
impl AlligatorNode {
    #[napi(constructor)]
    pub fn new(jaw: u32, teeth: u32, lips: u32) -> napi::Result<Self> {
        Ok(Self {
            inner: wc::Alligator::new(clamp_period(jaw), clamp_period(teeth), clamp_period(lips))
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
    pub fn update(&mut self, high: f64, low: f64) -> napi::Result<Option<AlligatorValue>> {
        Ok(self
            .inner
            .update(cnd(high, low, low, 0.0)?)
            .map(|o| AlligatorValue {
                jaw: o.jaw,
                teeth: o.teeth,
                lips: o.lips,
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
                out[i * 3] = o.jaw;
                out[i * 3 + 1] = o.teeth;
                out[i * 3 + 2] = o.lips;
            }
        }
        Ok(out)
    }
}

// ============================== JMA ==============================

#[napi(js_name = "JMA")]
pub struct JmaNode {
    inner: wc::Jma,
}
#[napi]
impl JmaNode {
    #[napi(constructor)]
    pub fn new(period: u32, phase: f64, power: u32) -> napi::Result<Self> {
        Ok(Self {
            inner: wc::Jma::new(clamp_period(period), phase, power).map_err(map_err)?,
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

// ============================== VIDYA ==============================

#[napi(js_name = "VIDYA")]
pub struct VidyaNode {
    inner: wc::Vidya,
}
#[napi]
impl VidyaNode {
    #[napi(constructor)]
    pub fn new(period: u32, cmo_period: u32) -> napi::Result<Self> {
        Ok(Self {
            inner: wc::Vidya::new(clamp_period(period), clamp_period(cmo_period))
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

// ============================== ALMA ==============================

#[napi(js_name = "ALMA")]
pub struct AlmaNode {
    inner: wc::Alma,
}
#[napi]
impl AlmaNode {
    #[napi(constructor)]
    pub fn new(period: u32, offset: f64, sigma: f64) -> napi::Result<Self> {
        Ok(Self {
            inner: wc::Alma::new(clamp_period(period), offset, sigma).map_err(map_err)?,
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

// ============================== Yang-Zhang Volatility ==============================

#[napi(js_name = "YangZhangVolatility")]
pub struct YangZhangVolatilityNode {
    inner: wc::YangZhangVolatility,
}

#[napi]
impl YangZhangVolatilityNode {
    #[napi(constructor)]
    pub fn new(period: u32, trading_periods: u32) -> napi::Result<Self> {
        Ok(Self {
            inner: wc::YangZhangVolatility::new(period as usize, trading_periods as usize)
                .map_err(map_err)?,
        })
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
        let n = open.len();
        if high.len() != n || low.len() != n || close.len() != n {
            return Err(NapiError::from_reason(
                "open, high, low, close must be equal length".to_string(),
            ));
        }
        let mut out = Vec::with_capacity(n);
        for i in 0..n {
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

// ============================== Rogers-Satchell Volatility ==============================

#[napi(js_name = "RogersSatchellVolatility")]
pub struct RogersSatchellVolatilityNode {
    inner: wc::RogersSatchellVolatility,
}

#[napi]
impl RogersSatchellVolatilityNode {
    #[napi(constructor)]
    pub fn new(period: u32, trading_periods: u32) -> napi::Result<Self> {
        Ok(Self {
            inner: wc::RogersSatchellVolatility::new(period as usize, trading_periods as usize)
                .map_err(map_err)?,
        })
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
        let n = open.len();
        if high.len() != n || low.len() != n || close.len() != n {
            return Err(NapiError::from_reason(
                "open, high, low, close must be equal length".to_string(),
            ));
        }
        let mut out = Vec::with_capacity(n);
        for i in 0..n {
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

// ============================== Garman-Klass Volatility ==============================

#[napi(js_name = "GarmanKlassVolatility")]
pub struct GarmanKlassVolatilityNode {
    inner: wc::GarmanKlassVolatility,
}

#[napi]
impl GarmanKlassVolatilityNode {
    #[napi(constructor)]
    pub fn new(period: u32, trading_periods: u32) -> napi::Result<Self> {
        Ok(Self {
            inner: wc::GarmanKlassVolatility::new(period as usize, trading_periods as usize)
                .map_err(map_err)?,
        })
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
        let n = open.len();
        if high.len() != n || low.len() != n || close.len() != n {
            return Err(NapiError::from_reason(
                "open, high, low, close must be equal length".to_string(),
            ));
        }
        let mut out = Vec::with_capacity(n);
        for i in 0..n {
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

// ============================== Parkinson Volatility ==============================

#[napi(js_name = "ParkinsonVolatility")]
pub struct ParkinsonVolatilityNode {
    inner: wc::ParkinsonVolatility,
}

#[napi]
impl ParkinsonVolatilityNode {
    #[napi(constructor)]
    pub fn new(period: u32, trading_periods: u32) -> napi::Result<Self> {
        Ok(Self {
            inner: wc::ParkinsonVolatility::new(period as usize, trading_periods as usize)
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

// ============================== Family 05: Bands & Channels ==============================

// ---------- MA Envelope ----------

#[napi(object)]
pub struct MaEnvelopeValue {
    pub upper: f64,
    pub middle: f64,
    pub lower: f64,
}

#[napi(js_name = "MaEnvelope")]
pub struct MaEnvelopeNode {
    inner: wc::MaEnvelope,
}

#[napi]
impl MaEnvelopeNode {
    #[napi(constructor)]
    pub fn new(period: u32, percent: f64) -> napi::Result<Self> {
        Ok(Self {
            inner: wc::MaEnvelope::new(period as usize, percent).map_err(map_err)?,
        })
    }
    #[napi]
    pub fn update(&mut self, value: f64) -> Option<MaEnvelopeValue> {
        self.inner.update(value).map(|o| MaEnvelopeValue {
            upper: o.upper,
            middle: o.middle,
            lower: o.lower,
        })
    }
    /// Flat `[upper0, middle0, lower0, upper1, ...]`, length `3 * n`.
    #[napi]
    pub fn batch(&mut self, prices: Vec<f64>) -> Vec<f64> {
        let mut out = vec![f64::NAN; prices.len() * 3];
        for (i, p) in prices.iter().enumerate() {
            if let Some(o) = self.inner.update(*p) {
                out[i * 3] = o.upper;
                out[i * 3 + 1] = o.middle;
                out[i * 3 + 2] = o.lower;
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

// ---------- Acceleration Bands ----------

#[napi(object)]
pub struct AccelerationBandsValue {
    pub upper: f64,
    pub middle: f64,
    pub lower: f64,
}

#[napi(js_name = "AccelerationBands")]
pub struct AccelerationBandsNode {
    inner: wc::AccelerationBands,
}

#[napi]
impl AccelerationBandsNode {
    #[napi(constructor)]
    pub fn new(period: u32, factor: f64) -> napi::Result<Self> {
        Ok(Self {
            inner: wc::AccelerationBands::new(period as usize, factor).map_err(map_err)?,
        })
    }
    #[napi]
    pub fn update(
        &mut self,
        high: f64,
        low: f64,
        close: f64,
    ) -> napi::Result<Option<AccelerationBandsValue>> {
        Ok(self
            .inner
            .update(cnd(high, low, close, 0.0)?)
            .map(|o| AccelerationBandsValue {
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

// ---------- STARC Bands ----------

#[napi(object)]
pub struct StarcBandsValue {
    pub upper: f64,
    pub middle: f64,
    pub lower: f64,
}

#[napi(js_name = "StarcBands")]
pub struct StarcBandsNode {
    inner: wc::StarcBands,
}

#[napi]
impl StarcBandsNode {
    #[napi(constructor)]
    pub fn new(sma_period: u32, atr_period: u32, multiplier: f64) -> napi::Result<Self> {
        Ok(Self {
            inner: wc::StarcBands::new(sma_period as usize, atr_period as usize, multiplier)
                .map_err(map_err)?,
        })
    }
    #[napi]
    pub fn update(
        &mut self,
        high: f64,
        low: f64,
        close: f64,
    ) -> napi::Result<Option<StarcBandsValue>> {
        Ok(self
            .inner
            .update(cnd(high, low, close, 0.0)?)
            .map(|o| StarcBandsValue {
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

// ---------- ATR Bands ----------

#[napi(object)]
pub struct AtrBandsValue {
    pub upper: f64,
    pub middle: f64,
    pub lower: f64,
}

#[napi(js_name = "AtrBands")]
pub struct AtrBandsNode {
    inner: wc::AtrBands,
}

#[napi]
impl AtrBandsNode {
    #[napi(constructor)]
    pub fn new(period: u32, multiplier: f64) -> napi::Result<Self> {
        Ok(Self {
            inner: wc::AtrBands::new(period as usize, multiplier).map_err(map_err)?,
        })
    }
    #[napi]
    pub fn update(
        &mut self,
        high: f64,
        low: f64,
        close: f64,
    ) -> napi::Result<Option<AtrBandsValue>> {
        Ok(self
            .inner
            .update(cnd(high, low, close, 0.0)?)
            .map(|o| AtrBandsValue {
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

// ---------- Hurst Channel ----------

#[napi(object)]
pub struct HurstChannelValue {
    pub upper: f64,
    pub middle: f64,
    pub lower: f64,
}

#[napi(js_name = "HurstChannel")]
pub struct HurstChannelNode {
    inner: wc::HurstChannel,
}

#[napi]
impl HurstChannelNode {
    #[napi(constructor)]
    pub fn new(period: u32, multiplier: f64) -> napi::Result<Self> {
        Ok(Self {
            inner: wc::HurstChannel::new(period as usize, multiplier).map_err(map_err)?,
        })
    }
    #[napi]
    pub fn update(
        &mut self,
        high: f64,
        low: f64,
        close: f64,
    ) -> napi::Result<Option<HurstChannelValue>> {
        Ok(self
            .inner
            .update(cnd(high, low, close, 0.0)?)
            .map(|o| HurstChannelValue {
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

// ---------- LinReg Channel ----------

#[napi(object)]
pub struct LinRegChannelValue {
    pub upper: f64,
    pub middle: f64,
    pub lower: f64,
}

#[napi(js_name = "LinRegChannel")]
pub struct LinRegChannelNode {
    inner: wc::LinRegChannel,
}

#[napi]
impl LinRegChannelNode {
    #[napi(constructor)]
    pub fn new(period: u32, multiplier: f64) -> napi::Result<Self> {
        Ok(Self {
            inner: wc::LinRegChannel::new(period as usize, multiplier).map_err(map_err)?,
        })
    }
    #[napi]
    pub fn update(&mut self, value: f64) -> Option<LinRegChannelValue> {
        self.inner.update(value).map(|o| LinRegChannelValue {
            upper: o.upper,
            middle: o.middle,
            lower: o.lower,
        })
    }
    #[napi]
    pub fn batch(&mut self, prices: Vec<f64>) -> Vec<f64> {
        let mut out = vec![f64::NAN; prices.len() * 3];
        for (i, p) in prices.iter().enumerate() {
            if let Some(o) = self.inner.update(*p) {
                out[i * 3] = o.upper;
                out[i * 3 + 1] = o.middle;
                out[i * 3 + 2] = o.lower;
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

// ---------- Standard Error Bands ----------

#[napi(object)]
pub struct StandardErrorBandsValue {
    pub upper: f64,
    pub middle: f64,
    pub lower: f64,
}

#[napi(js_name = "StandardErrorBands")]
pub struct StandardErrorBandsNode {
    inner: wc::StandardErrorBands,
}

#[napi]
impl StandardErrorBandsNode {
    #[napi(constructor)]
    pub fn new(period: u32, multiplier: f64) -> napi::Result<Self> {
        Ok(Self {
            inner: wc::StandardErrorBands::new(period as usize, multiplier).map_err(map_err)?,
        })
    }
    #[napi]
    pub fn update(&mut self, value: f64) -> Option<StandardErrorBandsValue> {
        self.inner.update(value).map(|o| StandardErrorBandsValue {
            upper: o.upper,
            middle: o.middle,
            lower: o.lower,
        })
    }
    #[napi]
    pub fn batch(&mut self, prices: Vec<f64>) -> Vec<f64> {
        let mut out = vec![f64::NAN; prices.len() * 3];
        for (i, p) in prices.iter().enumerate() {
            if let Some(o) = self.inner.update(*p) {
                out[i * 3] = o.upper;
                out[i * 3 + 1] = o.middle;
                out[i * 3 + 2] = o.lower;
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

// ---------- Double Bollinger ----------

#[napi(object)]
pub struct DoubleBollingerValue {
    #[napi(js_name = "upperOuter")]
    pub upper_outer: f64,
    #[napi(js_name = "upperInner")]
    pub upper_inner: f64,
    pub middle: f64,
    #[napi(js_name = "lowerInner")]
    pub lower_inner: f64,
    #[napi(js_name = "lowerOuter")]
    pub lower_outer: f64,
}

#[napi(js_name = "DoubleBollinger")]
pub struct DoubleBollingerNode {
    inner: wc::DoubleBollinger,
}

#[napi]
impl DoubleBollingerNode {
    #[napi(constructor)]
    pub fn new(period: u32, k_inner: f64, k_outer: f64) -> napi::Result<Self> {
        Ok(Self {
            inner: wc::DoubleBollinger::new(period as usize, k_inner, k_outer).map_err(map_err)?,
        })
    }
    #[napi]
    pub fn update(&mut self, value: f64) -> Option<DoubleBollingerValue> {
        self.inner.update(value).map(|o| DoubleBollingerValue {
            upper_outer: o.upper_outer,
            upper_inner: o.upper_inner,
            middle: o.middle,
            lower_inner: o.lower_inner,
            lower_outer: o.lower_outer,
        })
    }
    /// Flat `[u_o, u_i, m, l_i, l_o, ...]`, length `5 * n`.
    #[napi]
    pub fn batch(&mut self, prices: Vec<f64>) -> Vec<f64> {
        let mut out = vec![f64::NAN; prices.len() * 5];
        for (i, p) in prices.iter().enumerate() {
            if let Some(o) = self.inner.update(*p) {
                out[i * 5] = o.upper_outer;
                out[i * 5 + 1] = o.upper_inner;
                out[i * 5 + 2] = o.middle;
                out[i * 5 + 3] = o.lower_inner;
                out[i * 5 + 4] = o.lower_outer;
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

// ---------- TTM Squeeze ----------

#[napi(object)]
pub struct TtmSqueezeValue {
    pub squeeze: f64,
    pub momentum: f64,
}

#[napi(js_name = "TtmSqueeze")]
pub struct TtmSqueezeNode {
    inner: wc::TtmSqueeze,
}

#[napi]
impl TtmSqueezeNode {
    #[napi(constructor)]
    pub fn new(period: u32, bb_mult: f64, kc_mult: f64) -> napi::Result<Self> {
        Ok(Self {
            inner: wc::TtmSqueeze::new(period as usize, bb_mult, kc_mult).map_err(map_err)?,
        })
    }
    #[napi]
    pub fn update(
        &mut self,
        high: f64,
        low: f64,
        close: f64,
    ) -> napi::Result<Option<TtmSqueezeValue>> {
        Ok(self
            .inner
            .update(cnd(high, low, close, 0.0)?)
            .map(|o| TtmSqueezeValue {
                squeeze: o.squeeze,
                momentum: o.momentum,
            }))
    }
    /// Flat `[sq0, mom0, sq1, mom1, ...]`, length `2 * n`.
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
                out[i * 2] = o.squeeze;
                out[i * 2 + 1] = o.momentum;
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

// ---------- Fractal Chaos Bands ----------

#[napi(object)]
pub struct FractalChaosBandsValue {
    pub upper: f64,
    pub lower: f64,
}

#[napi(js_name = "FractalChaosBands")]
pub struct FractalChaosBandsNode {
    inner: wc::FractalChaosBands,
}

#[napi]
impl FractalChaosBandsNode {
    #[napi(constructor)]
    pub fn new(k: u32) -> napi::Result<Self> {
        Ok(Self {
            inner: wc::FractalChaosBands::new(k as usize).map_err(map_err)?,
        })
    }
    #[napi]
    pub fn update(&mut self, high: f64, low: f64) -> napi::Result<Option<FractalChaosBandsValue>> {
        Ok(self
            .inner
            .update(cnd(high, low, low, 0.0)?)
            .map(|o| FractalChaosBandsValue {
                upper: o.upper,
                lower: o.lower,
            }))
    }
    /// Flat `[u0, l0, u1, l1, ...]`, length `2 * n`.
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
                out[i * 2] = o.upper;
                out[i * 2 + 1] = o.lower;
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

// ---------- VWAP StdDev Bands ----------

#[napi(object)]
pub struct VwapStdDevBandsValue {
    pub upper: f64,
    pub middle: f64,
    pub lower: f64,
    pub stddev: f64,
}

#[napi(js_name = "VwapStdDevBands")]
pub struct VwapStdDevBandsNode {
    inner: wc::VwapStdDevBands,
}

#[napi]
impl VwapStdDevBandsNode {
    #[napi(constructor)]
    pub fn new(multiplier: f64) -> napi::Result<Self> {
        Ok(Self {
            inner: wc::VwapStdDevBands::new(multiplier).map_err(map_err)?,
        })
    }
    #[napi]
    pub fn update(
        &mut self,
        high: f64,
        low: f64,
        close: f64,
        volume: f64,
    ) -> napi::Result<Option<VwapStdDevBandsValue>> {
        Ok(self
            .inner
            .update(cnd(high, low, close, volume)?)
            .map(|o| VwapStdDevBandsValue {
                upper: o.upper,
                middle: o.middle,
                lower: o.lower,
                stddev: o.stddev,
            }))
    }
    /// Flat `[u0, m0, l0, sd0, ...]`, length `4 * n`.
    #[napi]
    pub fn batch(
        &mut self,
        high: Vec<f64>,
        low: Vec<f64>,
        close: Vec<f64>,
        volume: Vec<f64>,
    ) -> napi::Result<Vec<f64>> {
        let n = high.len();
        if low.len() != n || close.len() != n || volume.len() != n {
            return Err(NapiError::from_reason(
                "high, low, close, volume must be equal length".to_string(),
            ));
        }
        let mut out = vec![f64::NAN; n * 4];
        for i in 0..n {
            if let Some(o) = self
                .inner
                .update(cnd(high[i], low[i], close[i], volume[i])?)
            {
                out[i * 4] = o.upper;
                out[i * 4 + 1] = o.middle;
                out[i * 4 + 2] = o.lower;
                out[i * 4 + 3] = o.stddev;
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
