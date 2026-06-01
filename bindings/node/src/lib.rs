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
            pub fn new(period: u32) -> napi::Result<Self> {
                Ok(Self {
                    inner: <$rust_ty>::new(period as usize).map_err(map_err)?,
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

// Family 10 — Ehlers / Cycle: single-period scalars.
node_scalar_indicator!(SuperSmootherNode, "SuperSmoother", wc::SuperSmoother);
node_scalar_indicator!(FisherTransformNode, "FisherTransform", wc::FisherTransform);
node_scalar_indicator!(DecyclerNode, "Decycler", wc::Decycler);
node_scalar_indicator!(CenterOfGravityNode, "CenterOfGravity", wc::CenterOfGravity);
node_scalar_indicator!(CyberneticCycleNode, "CyberneticCycle", wc::CyberneticCycle);
node_scalar_indicator!(
    InstantaneousTrendlineNode,
    "InstantaneousTrendline",
    wc::InstantaneousTrendline
);
node_scalar_indicator!(
    EhlersStochasticNode,
    "EhlersStochastic",
    wc::EhlersStochastic
);

// RviVolatility (Relative Volatility Index, Donald Dorsey). Disambiguated
// from `RVI` = Relative Vigor Index in Family 02. Takes a single `period` and
// rejects both `period == 0` and `period == 1` (a 1-bar standard deviation is
// always zero). Hand-rolled, but behaves like `node_scalar_indicator!`: the
// fallible `new` propagates the core error and throws a JS error on bad period.
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

node_scalar_indicator!(VarianceNode, "Variance", wc::Variance);
node_scalar_indicator!(
    CoefficientOfVariationNode,
    "CoefficientOfVariation",
    wc::CoefficientOfVariation
);
node_scalar_indicator!(SkewnessNode, "Skewness", wc::Skewness);
node_scalar_indicator!(KurtosisNode, "Kurtosis", wc::Kurtosis);
node_scalar_indicator!(StandardErrorNode, "StandardError", wc::StandardError);
node_scalar_indicator!(DetrendedStdDevNode, "DetrendedStdDev", wc::DetrendedStdDev);
node_scalar_indicator!(RSquaredNode, "RSquared", wc::RSquared);
node_scalar_indicator!(
    MedianAbsoluteDeviationNode,
    "MedianAbsoluteDeviation",
    wc::MedianAbsoluteDeviation
);

// ============================== Autocorrelation (period + lag) ==============================

#[napi(js_name = "Autocorrelation")]
pub struct AutocorrelationNode {
    inner: wc::Autocorrelation,
}

#[napi]
impl AutocorrelationNode {
    #[napi(constructor)]
    pub fn new(period: u32, lag: u32) -> napi::Result<Self> {
        Ok(Self {
            inner: wc::Autocorrelation::new(period as usize, lag as usize).map_err(map_err)?,
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

// ============================== HurstExponent (period + chunks) ==============================

#[napi(js_name = "HurstExponent")]
pub struct HurstExponentNode {
    inner: wc::HurstExponent,
}

#[napi]
impl HurstExponentNode {
    #[napi(constructor)]
    pub fn new(period: u32, chunks: u32) -> napi::Result<Self> {
        Ok(Self {
            inner: wc::HurstExponent::new(period as usize, chunks as usize).map_err(map_err)?,
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

// ============================== Two-series indicators (Pearson / Beta / Spearman) ==============================

macro_rules! node_pair_indicator {
    ($wrapper:ident, $node_name:literal, $rust_ty:ty) => {
        #[napi(js_name = $node_name)]
        pub struct $wrapper {
            inner: $rust_ty,
        }

        #[napi]
        impl $wrapper {
            #[napi(constructor)]
            pub fn new(period: u32) -> napi::Result<Self> {
                Ok(Self {
                    inner: <$rust_ty>::new(period as usize).map_err(map_err)?,
                })
            }
            #[napi]
            pub fn update(&mut self, x: f64, y: f64) -> Option<f64> {
                self.inner.update((x, y))
            }
            /// Batch over two equally-sized arrays. Returns a length-`n` array
            /// with `NaN` for warmup positions.
            #[napi]
            pub fn batch(&mut self, x: Vec<f64>, y: Vec<f64>) -> napi::Result<Vec<f64>> {
                if x.len() != y.len() {
                    return Err(NapiError::new(
                        Status::InvalidArg,
                        "x and y must be equal length".to_string(),
                    ));
                }
                let mut out = Vec::with_capacity(x.len());
                for i in 0..x.len() {
                    out.push(self.inner.update((x[i], y[i])).unwrap_or(f64::NAN));
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
    };
}

node_pair_indicator!(
    PearsonCorrelationNode,
    "PearsonCorrelation",
    wc::PearsonCorrelation
);
node_pair_indicator!(BetaNode, "Beta", wc::Beta);
node_pair_indicator!(PairwiseBetaNode, "PairwiseBeta", wc::PairwiseBeta);
node_pair_indicator!(
    SpearmanCorrelationNode,
    "SpearmanCorrelation",
    wc::SpearmanCorrelation
);

// ============================== PairSpreadZScore ==============================

/// Pair spread z-score: two ctor params (`betaPeriod`, `zPeriod`), one `(a, b)`
/// price pair per update, a single z-score out.
#[napi(js_name = "PairSpreadZScore")]
pub struct PairSpreadZScoreNode {
    inner: wc::PairSpreadZScore,
}

#[napi]
impl PairSpreadZScoreNode {
    #[napi(constructor)]
    pub fn new(beta_period: u32, z_period: u32) -> napi::Result<Self> {
        Ok(Self {
            inner: wc::PairSpreadZScore::new(beta_period as usize, z_period as usize)
                .map_err(map_err)?,
        })
    }
    #[napi]
    pub fn update(&mut self, a: f64, b: f64) -> Option<f64> {
        self.inner.update((a, b))
    }
    /// Batch over two equally-sized arrays of prices. Returns a length-`n`
    /// array with `NaN` for warmup positions.
    #[napi]
    pub fn batch(&mut self, a: Vec<f64>, b: Vec<f64>) -> napi::Result<Vec<f64>> {
        if a.len() != b.len() {
            return Err(NapiError::new(
                Status::InvalidArg,
                "a and b must be equal length".to_string(),
            ));
        }
        let mut out = Vec::with_capacity(a.len());
        for i in 0..a.len() {
            out.push(self.inner.update((a[i], b[i])).unwrap_or(f64::NAN));
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

#[napi(js_name = "ADXR")]
pub struct AdxrNode {
    inner: wc::Adxr,
}

#[napi]
impl AdxrNode {
    #[napi(constructor)]
    pub fn new(period: u32) -> napi::Result<Self> {
        Ok(Self {
            inner: wc::Adxr::new(period as usize).map_err(map_err)?,
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
        let n = high.len();
        let mut out = vec![f64::NAN; n];
        for i in 0..n {
            if let Some(v) = self.inner.update(cnd(high[i], low[i], close[i], 0.0)?) {
                out[i] = v;
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
            inner: wc::Inertia::new(rvi_period as usize, linreg_period as usize)
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
                period_rsi as usize,
                period_streak as usize,
                period_rank as usize,
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
            inner: wc::Smi::new(period as usize, d_period as usize, d2_period as usize)
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
                roc1 as usize,
                roc2 as usize,
                roc3 as usize,
                roc4 as usize,
                sma1 as usize,
                sma2 as usize,
                sma3 as usize,
                sma4 as usize,
                signal as usize,
            )
            .map_err(map_err)?,
        })
    }
    #[napi(factory)]
    pub fn classic() -> Self {
        Self {
            inner: wc::Kst::classic(),
        }
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
            inner: wc::Pgo::new(period as usize).map_err(map_err)?,
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
            inner: wc::Rvi::new(period as usize).map_err(map_err)?,
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
                fast as usize,
                slow as usize,
                sma_period as usize,
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
            inner: wc::Stc::new(fast as usize, slow as usize, schaff_period as usize, factor)
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
                ema_period as usize,
                macd_fast as usize,
                macd_slow as usize,
                macd_signal as usize,
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
            inner: wc::ZeroLagMacd::new(fast as usize, slow as usize, signal as usize)
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
            inner: wc::Cfo::new(period as usize).map_err(map_err)?,
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
            inner: wc::Apo::new(fast as usize, slow as usize).map_err(map_err)?,
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
            inner: wc::Evwma::new(period as usize).map_err(map_err)?,
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
            inner: wc::Alligator::new(jaw as usize, teeth as usize, lips as usize)
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
            inner: wc::Jma::new(period as usize, phase, power).map_err(map_err)?,
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
            inner: wc::Vidya::new(period as usize, cmo_period as usize).map_err(map_err)?,
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
            inner: wc::Alma::new(period as usize, offset, sigma).map_err(map_err)?,
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

// ============================== TII ==============================

#[napi(js_name = "TII")]
pub struct TiiNode {
    inner: wc::Tii,
}

#[napi]
impl TiiNode {
    #[napi(constructor)]
    pub fn new(sma_period: u32, dev_period: u32) -> napi::Result<Self> {
        Ok(Self {
            inner: wc::Tii::new(sma_period as usize, dev_period as usize).map_err(map_err)?,
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

// ============================== Negative Volume Index ==============================

#[napi(js_name = "NVI")]
pub struct NviNode {
    inner: wc::Nvi,
}

#[napi]
impl NviNode {
    #[napi(constructor)]
    pub fn new(baseline: Option<f64>) -> Self {
        Self {
            inner: wc::Nvi::with_baseline(baseline.unwrap_or(1000.0)),
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

// ============================== Positive Volume Index ==============================

#[napi(js_name = "PVI")]
pub struct PviNode {
    inner: wc::Pvi,
}

#[napi]
impl PviNode {
    #[napi(constructor)]
    pub fn new(baseline: Option<f64>) -> Self {
        Self {
            inner: wc::Pvi::with_baseline(baseline.unwrap_or(1000.0)),
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

// ============================== Volume Oscillator ==============================

#[napi(js_name = "VolumeOscillator")]
pub struct VolumeOscillatorNode {
    inner: wc::VolumeOscillator,
}

#[napi]
impl VolumeOscillatorNode {
    #[napi(constructor)]
    pub fn new(fast: u32, slow: u32) -> napi::Result<Self> {
        Ok(Self {
            inner: wc::VolumeOscillator::new(fast as usize, slow as usize).map_err(map_err)?,
        })
    }
    #[napi]
    pub fn update(&mut self, volume: f64) -> napi::Result<Option<f64>> {
        Ok(self.inner.update(cnd(10.0, 10.0, 10.0, volume)?))
    }
    #[napi]
    pub fn batch(&mut self, volume: Vec<f64>) -> napi::Result<Vec<f64>> {
        let mut out = Vec::with_capacity(volume.len());
        for &v in &volume {
            out.push(
                self.inner
                    .update(cnd(10.0, 10.0, 10.0, v)?)
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

// ============================== Klinger Volume Oscillator ==============================

#[napi(js_name = "KVO")]
pub struct KvoNode {
    inner: wc::Kvo,
}

#[napi]
impl KvoNode {
    #[napi(constructor)]
    pub fn new(fast: u32, slow: u32) -> napi::Result<Self> {
        Ok(Self {
            inner: wc::Kvo::new(fast as usize, slow as usize).map_err(map_err)?,
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

// ============================== Williams A/D ==============================

#[napi(js_name = "WilliamsAD")]
pub struct AdOscillatorNode {
    inner: wc::AdOscillator,
}

#[napi]
impl AdOscillatorNode {
    #[napi(constructor)]
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            inner: wc::AdOscillator::new(),
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

// ============================== Anchored VWAP ==============================

#[napi(js_name = "AnchoredVWAP")]
pub struct AnchoredVwapNode {
    inner: wc::AnchoredVwap,
}

#[napi]
impl AnchoredVwapNode {
    #[napi(constructor)]
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            inner: wc::AnchoredVwap::new(),
        }
    }
    #[napi(js_name = "setAnchor")]
    pub fn set_anchor(&mut self) {
        self.inner.set_anchor();
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

// ============================== Demand Index ==============================

#[napi(js_name = "DemandIndex")]
pub struct DemandIndexNode {
    inner: wc::DemandIndex,
}

#[napi]
impl DemandIndexNode {
    #[napi(constructor)]
    pub fn new(period: u32) -> napi::Result<Self> {
        Ok(Self {
            inner: wc::DemandIndex::new(period as usize).map_err(map_err)?,
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

// ============================== Time Segmented Volume ==============================

#[napi(js_name = "TSV")]
pub struct TsvNode {
    inner: wc::Tsv,
}

#[napi]
impl TsvNode {
    #[napi(constructor)]
    pub fn new(period: u32) -> napi::Result<Self> {
        Ok(Self {
            inner: wc::Tsv::new(period as usize).map_err(map_err)?,
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

// ============================== Volume Zone Oscillator ==============================

#[napi(js_name = "VZO")]
pub struct VzoNode {
    inner: wc::Vzo,
}

#[napi]
impl VzoNode {
    #[napi(constructor)]
    pub fn new(period: u32) -> napi::Result<Self> {
        Ok(Self {
            inner: wc::Vzo::new(period as usize).map_err(map_err)?,
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

// ============================== Market Facilitation Index ==============================

#[napi(js_name = "MarketFacilitationIndex")]
pub struct MarketFacilitationIndexNode {
    inner: wc::MarketFacilitationIndex,
}

#[napi]
impl MarketFacilitationIndexNode {
    #[napi(constructor)]
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            inner: wc::MarketFacilitationIndex::new(),
        }
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

// ============================== HiLo Activator ==============================

#[napi(js_name = "HiLoActivator")]
pub struct HiLoActivatorNode {
    inner: wc::HiLoActivator,
}

#[napi]
impl HiLoActivatorNode {
    #[napi(constructor)]
    pub fn new(period: u32) -> napi::Result<Self> {
        Ok(Self {
            inner: wc::HiLoActivator::new(period as usize).map_err(map_err)?,
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

// ============================== Volty Stop ==============================

#[napi(js_name = "VoltyStop")]
pub struct VoltyStopNode {
    inner: wc::VoltyStop,
}

#[napi]
impl VoltyStopNode {
    #[napi(constructor)]
    pub fn new(atr_period: u32, multiplier: f64) -> napi::Result<Self> {
        Ok(Self {
            inner: wc::VoltyStop::new(atr_period as usize, multiplier).map_err(map_err)?,
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

// ============================== Yo-Yo Exit ==============================

#[napi(js_name = "YoyoExit")]
pub struct YoyoExitNode {
    inner: wc::YoyoExit,
}

#[napi]
impl YoyoExitNode {
    #[napi(constructor)]
    pub fn new(atr_period: u32, multiplier: f64) -> napi::Result<Self> {
        Ok(Self {
            inner: wc::YoyoExit::new(atr_period as usize, multiplier).map_err(map_err)?,
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
    #[napi(js_name = "inTrade")]
    pub fn in_trade(&self) -> bool {
        self.inner.in_trade()
    }
}

// ============================== Donchian Stop ==============================

#[napi(object)]
pub struct DonchianStopValue {
    pub stop_long: f64,
    pub stop_short: f64,
}

#[napi(js_name = "DonchianStop")]
pub struct DonchianStopNode {
    inner: wc::DonchianStop,
}

#[napi]
impl DonchianStopNode {
    #[napi(constructor)]
    pub fn new(period: u32) -> napi::Result<Self> {
        Ok(Self {
            inner: wc::DonchianStop::new(period as usize).map_err(map_err)?,
        })
    }
    #[napi]
    pub fn update(&mut self, high: f64, low: f64) -> napi::Result<Option<DonchianStopValue>> {
        Ok(self
            .inner
            .update(cnd(high, low, low, 0.0)?)
            .map(|o| DonchianStopValue {
                stop_long: o.stop_long,
                stop_short: o.stop_short,
            }))
    }
    /// Returns `[long0, short0, long1, short1, ...]`, length `2 * n`. Warmup
    /// positions are `NaN`.
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

// ============================== Percentage Trailing Stop ==============================

#[napi(js_name = "PercentageTrailingStop")]
pub struct PercentageTrailingStopNode {
    inner: wc::PercentageTrailingStop,
}

#[napi]
impl PercentageTrailingStopNode {
    #[napi(constructor)]
    pub fn new(percent: f64) -> napi::Result<Self> {
        Ok(Self {
            inner: wc::PercentageTrailingStop::new(percent).map_err(map_err)?,
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

// ============================== Step Trailing Stop ==============================

#[napi(js_name = "StepTrailingStop")]
pub struct StepTrailingStopNode {
    inner: wc::StepTrailingStop,
}

#[napi]
impl StepTrailingStopNode {
    #[napi(constructor)]
    pub fn new(step_size: f64) -> napi::Result<Self> {
        Ok(Self {
            inner: wc::StepTrailingStop::new(step_size).map_err(map_err)?,
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

// ============================== Renko Trailing Stop ==============================

#[napi(js_name = "RenkoTrailingStop")]
pub struct RenkoTrailingStopNode {
    inner: wc::RenkoTrailingStop,
}

#[napi]
impl RenkoTrailingStopNode {
    #[napi(constructor)]
    pub fn new(block_size: f64) -> napi::Result<Self> {
        Ok(Self {
            inner: wc::RenkoTrailingStop::new(block_size).map_err(map_err)?,
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

/// Random Walk Index pair: `RWI_High` and `RWI_Low`.
#[napi(object)]
pub struct RwiValue {
    pub high: f64,
    pub low: f64,
}

/// Wave Trend Oscillator pair: `wt1` (channel index) and `wt2` (signal SMA).
#[napi(object)]
pub struct WaveTrendValue {
    pub wt1: f64,
    pub wt2: f64,
}

#[napi(js_name = "WaveTrend")]
pub struct WaveTrendNode {
    inner: wc::WaveTrend,
}

#[napi]
impl WaveTrendNode {
    #[napi(constructor)]
    pub fn new(channel_period: u32, average_period: u32, signal_period: u32) -> napi::Result<Self> {
        Ok(Self {
            inner: wc::WaveTrend::new(
                channel_period as usize,
                average_period as usize,
                signal_period as usize,
            )
            .map_err(map_err)?,
        })
    }
    #[napi(factory)]
    pub fn classic() -> napi::Result<Self> {
        Ok(Self {
            inner: wc::WaveTrend::classic().map_err(map_err)?,
        })
    }
    #[napi]
    pub fn update(
        &mut self,
        high: f64,
        low: f64,
        close: f64,
    ) -> napi::Result<Option<WaveTrendValue>> {
        Ok(self
            .inner
            .update(cnd(high, low, close, 0.0)?)
            .map(|o| WaveTrendValue {
                wt1: o.wt1,
                wt2: o.wt2,
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
        let mut out = vec![f64::NAN; n * 2];
        for i in 0..n {
            if let Some(o) = self.inner.update(cnd(high[i], low[i], close[i], 0.0)?) {
                out[i * 2] = o.wt1;
                out[i * 2 + 1] = o.wt2;
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

#[napi(js_name = "RWI")]
pub struct RwiNode {
    inner: wc::Rwi,
}

#[napi]
impl RwiNode {
    #[napi(constructor)]
    pub fn new(period: u32) -> napi::Result<Self> {
        Ok(Self {
            inner: wc::Rwi::new(period as usize).map_err(map_err)?,
        })
    }
    #[napi]
    pub fn update(&mut self, high: f64, low: f64, close: f64) -> napi::Result<Option<RwiValue>> {
        Ok(self
            .inner
            .update(cnd(high, low, close, 0.0)?)
            .map(|o| RwiValue {
                high: o.high,
                low: o.low,
            }))
    }
    /// Returns `[high0, low0, high1, low1, ...]`, length `2 * n`. Warmup is NaN.
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
                out[i * 2] = o.high;
                out[i * 2 + 1] = o.low;
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
// ============================== TD Setup ==============================

#[napi(js_name = "TDSetup")]
pub struct TdSetupNode {
    inner: wc::TdSetup,
}

#[napi]
impl TdSetupNode {
    #[napi(constructor)]
    pub fn new(lookback: u32, target: u32) -> napi::Result<Self> {
        Ok(Self {
            inner: wc::TdSetup::new(lookback as usize, target as usize).map_err(map_err)?,
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

// ============================== TD Sequential ==============================

/// TD Sequential output triple: setup count, countdown count, direction.
#[napi(object)]
pub struct TdSequentialValue {
    pub setup: f64,
    pub countdown: f64,
    pub direction: f64,
}

#[napi(js_name = "TDSequential")]
pub struct TdSequentialNode {
    inner: wc::TdSequential,
}

#[napi]
impl TdSequentialNode {
    #[napi(constructor)]
    pub fn new(
        setup_lookback: u32,
        setup_target: u32,
        countdown_lookback: u32,
        countdown_target: u32,
    ) -> napi::Result<Self> {
        Ok(Self {
            inner: wc::TdSequential::new(
                setup_lookback as usize,
                setup_target as usize,
                countdown_lookback as usize,
                countdown_target as usize,
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
    ) -> napi::Result<Option<TdSequentialValue>> {
        Ok(self
            .inner
            .update(cnd(high, low, close, 0.0)?)
            .map(|o| TdSequentialValue {
                setup: o.setup,
                countdown: o.countdown,
                direction: o.direction,
            }))
    }
    /// Batch returns a flat array `[setup0, countdown0, direction0, setup1, ...]`.
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
                out[i * 3] = o.setup;
                out[i * 3 + 1] = o.countdown;
                out[i * 3 + 2] = o.direction;
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

// ============================== TD DeMarker ==============================

#[napi(js_name = "TDDeMarker")]
pub struct TdDeMarkerNode {
    inner: wc::TdDeMarker,
}

#[napi]
impl TdDeMarkerNode {
    #[napi(constructor)]
    pub fn new(period: u32) -> napi::Result<Self> {
        Ok(Self {
            inner: wc::TdDeMarker::new(period as usize).map_err(map_err)?,
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

// ============================== TD REI ==============================

#[napi(js_name = "TDREI")]
pub struct TdReiNode {
    inner: wc::TdRei,
}

#[napi]
impl TdReiNode {
    #[napi(constructor)]
    pub fn new(period: u32) -> napi::Result<Self> {
        Ok(Self {
            inner: wc::TdRei::new(period as usize).map_err(map_err)?,
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

// ============================== TD Pressure ==============================

#[napi(js_name = "TDPressure")]
pub struct TdPressureNode {
    inner: wc::TdPressure,
}

#[napi]
impl TdPressureNode {
    #[napi(constructor)]
    pub fn new(period: u32) -> napi::Result<Self> {
        Ok(Self {
            inner: wc::TdPressure::new(period as usize).map_err(map_err)?,
        })
    }
    #[napi]
    pub fn update(
        &mut self,
        open: f64,
        high: f64,
        low: f64,
        close: f64,
        volume: f64,
    ) -> napi::Result<Option<f64>> {
        let candle = wc::Candle::new(open, high, low, close, volume, 0).map_err(map_err)?;
        Ok(self.inner.update(candle))
    }
    #[napi]
    pub fn batch(
        &mut self,
        open: Vec<f64>,
        high: Vec<f64>,
        low: Vec<f64>,
        close: Vec<f64>,
        volume: Vec<f64>,
    ) -> napi::Result<Vec<f64>> {
        if open.len() != high.len()
            || high.len() != low.len()
            || low.len() != close.len()
            || close.len() != volume.len()
        {
            return Err(NapiError::from_reason(
                "open, high, low, close, volume must be equal length".to_string(),
            ));
        }
        let mut out = Vec::with_capacity(open.len());
        for i in 0..open.len() {
            let candle = wc::Candle::new(open[i], high[i], low[i], close[i], volume[i], 0)
                .map_err(map_err)?;
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

// ============================== TD Combo ==============================

#[napi(js_name = "TDCombo")]
pub struct TdComboNode {
    inner: wc::TdCombo,
}

#[napi]
impl TdComboNode {
    #[napi(constructor)]
    pub fn new(
        setup_lookback: u32,
        setup_target: u32,
        countdown_lookback: u32,
        countdown_target: u32,
    ) -> napi::Result<Self> {
        Ok(Self {
            inner: wc::TdCombo::new(
                setup_lookback as usize,
                setup_target as usize,
                countdown_lookback as usize,
                countdown_target as usize,
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

// ============================== TD Countdown ==============================

#[napi(js_name = "TDCountdown")]
pub struct TdCountdownNode {
    inner: wc::TdCountdown,
}

#[napi]
impl TdCountdownNode {
    #[napi(constructor)]
    pub fn new(
        setup_lookback: u32,
        setup_target: u32,
        countdown_lookback: u32,
        countdown_target: u32,
    ) -> napi::Result<Self> {
        Ok(Self {
            inner: wc::TdCountdown::new(
                setup_lookback as usize,
                setup_target as usize,
                countdown_lookback as usize,
                countdown_target as usize,
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

// ============================== TD Lines ==============================

/// TD Lines output pair: latest TDST resistance / support (NaN if unset).
#[napi(object)]
pub struct TdLinesValue {
    pub resistance: f64,
    pub support: f64,
}

#[napi(js_name = "TDLines")]
pub struct TdLinesNode {
    inner: wc::TdLines,
}

#[napi]
impl TdLinesNode {
    #[napi(constructor)]
    pub fn new(lookback: u32, target: u32) -> napi::Result<Self> {
        Ok(Self {
            inner: wc::TdLines::new(lookback as usize, target as usize).map_err(map_err)?,
        })
    }
    #[napi]
    pub fn update(
        &mut self,
        high: f64,
        low: f64,
        close: f64,
    ) -> napi::Result<Option<TdLinesValue>> {
        Ok(self
            .inner
            .update(cnd(high, low, close, 0.0)?)
            .map(|o| TdLinesValue {
                resistance: o.resistance,
                support: o.support,
            }))
    }
    /// Batch returns a flat array `[resistance0, support0, resistance1, ...]`.
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
                out[i * 2] = o.resistance;
                out[i * 2 + 1] = o.support;
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

// ============================== TD Range Projection ==============================

/// TD Range Projection output pair: projected next-bar high / low.
#[napi(object)]
pub struct TdRangeProjectionValue {
    pub high: f64,
    pub low: f64,
}

#[napi(js_name = "TDRangeProjection")]
pub struct TdRangeProjectionNode {
    inner: wc::TdRangeProjection,
}

impl Default for TdRangeProjectionNode {
    fn default() -> Self {
        Self::new()
    }
}

#[napi]
impl TdRangeProjectionNode {
    #[napi(constructor)]
    pub fn new() -> Self {
        Self {
            inner: wc::TdRangeProjection::new(),
        }
    }
    #[napi]
    pub fn update(
        &mut self,
        open: f64,
        high: f64,
        low: f64,
        close: f64,
    ) -> napi::Result<Option<TdRangeProjectionValue>> {
        let candle = wc::Candle::new(open, high, low, close, 0.0, 0).map_err(map_err)?;
        Ok(self.inner.update(candle).map(|o| TdRangeProjectionValue {
            high: o.high,
            low: o.low,
        }))
    }
    /// Batch returns a flat array `[high0, low0, high1, low1, ...]`.
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
        let mut out = vec![f64::NAN; n * 2];
        for i in 0..n {
            let candle =
                wc::Candle::new(open[i], high[i], low[i], close[i], 0.0, 0).map_err(map_err)?;
            if let Some(p) = self.inner.update(candle) {
                out[i * 2] = p.high;
                out[i * 2 + 1] = p.low;
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

// ============================== TD Differential ==============================

#[napi(js_name = "TDDifferential")]
pub struct TdDifferentialNode {
    inner: wc::TdDifferential,
}

impl Default for TdDifferentialNode {
    fn default() -> Self {
        Self::new()
    }
}

#[napi]
impl TdDifferentialNode {
    #[napi(constructor)]
    pub fn new() -> Self {
        Self {
            inner: wc::TdDifferential::new(),
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

// ============================== TD Open ==============================

#[napi(js_name = "TDOpen")]
pub struct TdOpenNode {
    inner: wc::TdOpen,
}

impl Default for TdOpenNode {
    fn default() -> Self {
        Self::new()
    }
}

#[napi]
impl TdOpenNode {
    #[napi(constructor)]
    pub fn new() -> Self {
        Self {
            inner: wc::TdOpen::new(),
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

// ============================== TD Risk Level ==============================

/// TD Risk Level output pair: buy-side / sell-side protective stop levels
/// (NaN if unset).
#[napi(object)]
pub struct TdRiskLevelValue {
    pub buy_risk: f64,
    pub sell_risk: f64,
}

#[napi(js_name = "TDRiskLevel")]
pub struct TdRiskLevelNode {
    inner: wc::TdRiskLevel,
}

#[napi]
impl TdRiskLevelNode {
    #[napi(constructor)]
    pub fn new(lookback: u32, target: u32) -> napi::Result<Self> {
        Ok(Self {
            inner: wc::TdRiskLevel::new(lookback as usize, target as usize).map_err(map_err)?,
        })
    }
    #[napi]
    pub fn update(
        &mut self,
        high: f64,
        low: f64,
        close: f64,
    ) -> napi::Result<Option<TdRiskLevelValue>> {
        Ok(self
            .inner
            .update(cnd(high, low, close, 0.0)?)
            .map(|o| TdRiskLevelValue {
                buy_risk: o.buy_risk,
                sell_risk: o.sell_risk,
            }))
    }
    /// Batch returns a flat array `[buyRisk0, sellRisk0, buyRisk1, ...]`.
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
                out[i * 2] = o.buy_risk;
                out[i * 2 + 1] = o.sell_risk;
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

// ============================== Family 10 — Ehlers / Cycle ==============================

#[napi(js_name = "InverseFisherTransform")]
pub struct InverseFisherTransformNode {
    inner: wc::InverseFisherTransform,
}

#[napi]
impl InverseFisherTransformNode {
    #[napi(constructor)]
    pub fn new(scale: f64) -> napi::Result<Self> {
        Ok(Self {
            inner: wc::InverseFisherTransform::new(scale).map_err(map_err)?,
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

#[napi(js_name = "DecyclerOscillator")]
pub struct DecyclerOscillatorNode {
    inner: wc::DecyclerOscillator,
}

#[napi]
impl DecyclerOscillatorNode {
    #[napi(constructor)]
    pub fn new(fast: u32, slow: u32) -> napi::Result<Self> {
        Ok(Self {
            inner: wc::DecyclerOscillator::new(fast as usize, slow as usize).map_err(map_err)?,
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

#[napi(js_name = "RoofingFilter")]
pub struct RoofingFilterNode {
    inner: wc::RoofingFilter,
}

#[napi]
impl RoofingFilterNode {
    #[napi(constructor)]
    pub fn new(lp_period: u32, hp_period: u32) -> napi::Result<Self> {
        Ok(Self {
            inner: wc::RoofingFilter::new(lp_period as usize, hp_period as usize)
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

#[napi(js_name = "EmpiricalModeDecomposition")]
pub struct EmpiricalModeDecompositionNode {
    inner: wc::EmpiricalModeDecomposition,
}

#[napi]
impl EmpiricalModeDecompositionNode {
    #[napi(constructor)]
    pub fn new(period: u32, fraction: f64) -> napi::Result<Self> {
        Ok(Self {
            inner: wc::EmpiricalModeDecomposition::new(period as usize, fraction)
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

#[napi(js_name = "HilbertDominantCycle")]
pub struct HilbertDominantCycleNode {
    inner: wc::HilbertDominantCycle,
}

impl Default for HilbertDominantCycleNode {
    fn default() -> Self {
        Self::new()
    }
}

#[napi]
impl HilbertDominantCycleNode {
    #[napi(constructor)]
    pub fn new() -> Self {
        Self {
            inner: wc::HilbertDominantCycle::new(),
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

#[napi(js_name = "AdaptiveCycle")]
pub struct AdaptiveCycleNode {
    inner: wc::AdaptiveCycle,
}

impl Default for AdaptiveCycleNode {
    fn default() -> Self {
        Self::new()
    }
}

#[napi]
impl AdaptiveCycleNode {
    #[napi(constructor)]
    pub fn new() -> Self {
        Self {
            inner: wc::AdaptiveCycle::new(),
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

#[napi(js_name = "SineWave")]
pub struct SineWaveNode {
    inner: wc::SineWave,
}

impl Default for SineWaveNode {
    fn default() -> Self {
        Self::new()
    }
}

#[napi]
impl SineWaveNode {
    #[napi(constructor)]
    pub fn new() -> Self {
        Self {
            inner: wc::SineWave::new(),
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
    pub fn lead(&self) -> f64 {
        self.inner.lead()
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
pub struct MamaValue {
    pub mama: f64,
    pub fama: f64,
}

#[napi(js_name = "MAMA")]
pub struct MamaNode {
    inner: wc::Mama,
}

#[napi]
impl MamaNode {
    #[napi(constructor)]
    pub fn new(fast_limit: f64, slow_limit: f64) -> napi::Result<Self> {
        Ok(Self {
            inner: wc::Mama::new(fast_limit, slow_limit).map_err(map_err)?,
        })
    }
    #[napi]
    pub fn update(&mut self, value: f64) -> Option<MamaValue> {
        self.inner.update(value).map(|o| MamaValue {
            mama: o.mama,
            fama: o.fama,
        })
    }
    /// Returns a flat array of length `2 * n`: `[mama0, fama0, mama1, fama1, ...]`.
    #[napi]
    pub fn batch(&mut self, prices: Vec<f64>) -> Vec<f64> {
        let mut out = vec![f64::NAN; prices.len() * 2];
        for (i, p) in prices.iter().enumerate() {
            if let Some(o) = self.inner.update(*p) {
                out[i * 2] = o.mama;
                out[i * 2 + 1] = o.fama;
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

#[napi(js_name = "FAMA")]
pub struct FamaNode {
    inner: wc::Fama,
}

#[napi]
impl FamaNode {
    #[napi(constructor)]
    pub fn new(fast_limit: f64, slow_limit: f64) -> napi::Result<Self> {
        Ok(Self {
            inner: wc::Fama::new(fast_limit, slow_limit).map_err(map_err)?,
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

// ============================== Ichimoku ==============================

/// Ichimoku output: five lines, any of which may be `NaN` while the indicator
/// is still warming up.
#[napi(object)]
pub struct IchimokuValue {
    pub tenkan: f64,
    pub kijun: f64,
    #[napi(js_name = "senkouA")]
    pub senkou_a: f64,
    #[napi(js_name = "senkouB")]
    pub senkou_b: f64,
    pub chikou: f64,
}

#[napi(js_name = "Ichimoku")]
pub struct IchimokuNode {
    inner: wc::Ichimoku,
}

#[napi]
impl IchimokuNode {
    #[napi(constructor)]
    pub fn new(
        tenkan_period: u32,
        kijun_period: u32,
        senkou_b_period: u32,
        displacement: u32,
    ) -> napi::Result<Self> {
        Ok(Self {
            inner: wc::Ichimoku::new(
                tenkan_period as usize,
                kijun_period as usize,
                senkou_b_period as usize,
                displacement as usize,
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
    ) -> napi::Result<Option<IchimokuValue>> {
        Ok(self
            .inner
            .update(cnd(high, low, close, 0.0)?)
            .map(|o| IchimokuValue {
                tenkan: o.tenkan.unwrap_or(f64::NAN),
                kijun: o.kijun.unwrap_or(f64::NAN),
                senkou_a: o.senkou_a.unwrap_or(f64::NAN),
                senkou_b: o.senkou_b.unwrap_or(f64::NAN),
                chikou: o.chikou.unwrap_or(f64::NAN),
            }))
    }
    /// Returns `[tenkan0, kijun0, senkouA0, senkouB0, chikou0, tenkan1, ...]`,
    /// length `5 * n`. Cells without a defined value are `NaN`.
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

// ============================== Heikin-Ashi ==============================

#[napi(object)]
pub struct HeikinAshiValue {
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
}

#[napi(js_name = "HeikinAshi")]
pub struct HeikinAshiNode {
    inner: wc::HeikinAshi,
}

impl Default for HeikinAshiNode {
    fn default() -> Self {
        Self::new()
    }
}

#[napi]
impl HeikinAshiNode {
    #[napi(constructor)]
    pub fn new() -> Self {
        Self {
            inner: wc::HeikinAshi::new(),
        }
    }
    #[napi]
    pub fn update(
        &mut self,
        open: f64,
        high: f64,
        low: f64,
        close: f64,
    ) -> napi::Result<Option<HeikinAshiValue>> {
        let c = wc::Candle::new(open, high, low, close, 0.0, 0).map_err(map_err)?;
        Ok(self.inner.update(c).map(|o| HeikinAshiValue {
            open: o.open,
            high: o.high,
            low: o.low,
            close: o.close,
        }))
    }
    /// Returns `[open0, high0, low0, close0, open1, ...]`, length `4 * n`.
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

// ============================== ValueArea ==============================

#[napi(object)]
pub struct ValueAreaValue {
    pub poc: f64,
    pub vah: f64,
    pub val: f64,
}

#[napi(js_name = "ValueArea")]
pub struct ValueAreaNode {
    inner: wc::ValueArea,
}
#[napi]
impl ValueAreaNode {
    #[napi(constructor)]
    pub fn new(period: u32, bin_count: u32, value_area_pct: f64) -> napi::Result<Self> {
        Ok(Self {
            inner: wc::ValueArea::new(period as usize, bin_count as usize, value_area_pct)
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
        volume: f64,
    ) -> napi::Result<Option<ValueAreaValue>> {
        let mid = f64::midpoint(high, low);
        let candle = wc::Candle::new(mid, high, low, mid, volume, 0).map_err(map_err)?;
        Ok(self.inner.update(candle).map(|o| ValueAreaValue {
            poc: o.poc,
            vah: o.vah,
            val: o.val,
        }))
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
        let n = high.len();
        let mut out = vec![f64::NAN; n * 3];
        for i in 0..n {
            let mid = f64::midpoint(high[i], low[i]);
            let candle =
                wc::Candle::new(mid, high[i], low[i], mid, volume[i], 0).map_err(map_err)?;
            if let Some(o) = self.inner.update(candle) {
                out[i * 3] = o.poc;
                out[i * 3 + 1] = o.vah;
                out[i * 3 + 2] = o.val;
            }
        }
        Ok(out)
    }
}

// ============================== InitialBalance ==============================

#[napi(object)]
pub struct InitialBalanceValue {
    pub high: f64,
    pub low: f64,
}

#[napi(js_name = "InitialBalance")]
pub struct InitialBalanceNode {
    inner: wc::InitialBalance,
}
#[napi]
impl InitialBalanceNode {
    #[napi(constructor)]
    pub fn new(period: u32) -> napi::Result<Self> {
        Ok(Self {
            inner: wc::InitialBalance::new(period as usize).map_err(map_err)?,
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
    #[napi(js_name = "isLocked")]
    pub fn is_locked(&self) -> bool {
        self.inner.is_locked()
    }
    #[napi(js_name = "warmupPeriod")]
    pub fn warmup_period(&self) -> u32 {
        self.inner.warmup_period() as u32
    }
    #[napi]
    pub fn update(&mut self, high: f64, low: f64) -> napi::Result<Option<InitialBalanceValue>> {
        let mid = f64::midpoint(high, low);
        let candle = wc::Candle::new(mid, high, low, mid, 0.0, 0).map_err(map_err)?;
        Ok(self.inner.update(candle).map(|o| InitialBalanceValue {
            high: o.high,
            low: o.low,
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
            let mid = f64::midpoint(high[i], low[i]);
            let candle = wc::Candle::new(mid, high[i], low[i], mid, 0.0, 0).map_err(map_err)?;
            if let Some(o) = self.inner.update(candle) {
                out[i * 2] = o.high;
                out[i * 2 + 1] = o.low;
            }
        }
        Ok(out)
    }
}

// ============================== OpeningRange ==============================

#[napi(object)]
pub struct OpeningRangeValue {
    pub high: f64,
    pub low: f64,
    pub breakout_distance: f64,
}

#[napi(js_name = "OpeningRange")]
pub struct OpeningRangeNode {
    inner: wc::OpeningRange,
}
#[napi]
impl OpeningRangeNode {
    #[napi(constructor)]
    pub fn new(period: u32) -> napi::Result<Self> {
        Ok(Self {
            inner: wc::OpeningRange::new(period as usize).map_err(map_err)?,
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
    #[napi(js_name = "isLocked")]
    pub fn is_locked(&self) -> bool {
        self.inner.is_locked()
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
    ) -> napi::Result<Option<OpeningRangeValue>> {
        let candle = wc::Candle::new(close, high, low, close, 0.0, 0).map_err(map_err)?;
        Ok(self.inner.update(candle).map(|o| OpeningRangeValue {
            high: o.high,
            low: o.low,
            breakout_distance: o.breakout_distance,
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
            let candle =
                wc::Candle::new(close[i], high[i], low[i], close[i], 0.0, 0).map_err(map_err)?;
            if let Some(o) = self.inner.update(candle) {
                out[i * 3] = o.high;
                out[i * 3 + 1] = o.low;
                out[i * 3 + 2] = o.breakout_distance;
            }
        }
        Ok(out)
    }
}
// ============================== Candlestick Patterns ==============================
//
// All 15 patterns take Candles (open, high, low, close) and emit a signed f64
// signal per bar: +1.0 bullish, -1.0 bearish, 0.0 no pattern. Doji is
// direction-less and emits 0/+1 only.

macro_rules! node_candle_pattern {
    ($node:ident, $inner:ty, $js:literal) => {
        #[napi(js_name = $js)]
        pub struct $node {
            inner: $inner,
        }

        impl Default for $node {
            fn default() -> Self {
                Self::new()
            }
        }

        #[napi]
        impl $node {
            #[napi(constructor)]
            pub fn new() -> Self {
                Self {
                    inner: <$inner>::new(),
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
                    let candle = wc::Candle::new(open[i], high[i], low[i], close[i], 0.0, 0)
                        .map_err(map_err)?;
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
    };
}

node_candle_pattern!(DojiNode, wc::Doji, "Doji");
node_candle_pattern!(HammerNode, wc::Hammer, "Hammer");
node_candle_pattern!(InvertedHammerNode, wc::InvertedHammer, "InvertedHammer");
node_candle_pattern!(HangingManNode, wc::HangingMan, "HangingMan");
node_candle_pattern!(ShootingStarNode, wc::ShootingStar, "ShootingStar");
node_candle_pattern!(EngulfingNode, wc::Engulfing, "Engulfing");
node_candle_pattern!(HaramiNode, wc::Harami, "Harami");
node_candle_pattern!(
    MorningEveningStarNode,
    wc::MorningEveningStar,
    "MorningEveningStar"
);
node_candle_pattern!(
    ThreeSoldiersOrCrowsNode,
    wc::ThreeSoldiersOrCrows,
    "ThreeSoldiersOrCrows"
);
node_candle_pattern!(
    PiercingDarkCloudNode,
    wc::PiercingDarkCloud,
    "PiercingDarkCloud"
);
node_candle_pattern!(MarubozuNode, wc::Marubozu, "Marubozu");
node_candle_pattern!(TweezerNode, wc::Tweezer, "Tweezer");
node_candle_pattern!(SpinningTopNode, wc::SpinningTop, "SpinningTop");
node_candle_pattern!(ThreeInsideNode, wc::ThreeInside, "ThreeInside");
node_candle_pattern!(ThreeOutsideNode, wc::ThreeOutside, "ThreeOutside");

// ============================== Family 15: Risk / Performance ==============================

// Risk metrics with fallible `new` (most need `period >= 2`), so each wrapper
// is written by hand rather than going through the `node_scalar_indicator!`
// macro above.

#[napi(js_name = "SharpeRatio")]
pub struct SharpeRatioNode {
    inner: wc::SharpeRatio,
}

#[napi]
impl SharpeRatioNode {
    #[napi(constructor)]
    pub fn new(period: u32, risk_free: f64) -> napi::Result<Self> {
        Ok(Self {
            inner: wc::SharpeRatio::new(period as usize, risk_free).map_err(map_err)?,
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

#[napi(js_name = "SortinoRatio")]
pub struct SortinoRatioNode {
    inner: wc::SortinoRatio,
}

#[napi]
impl SortinoRatioNode {
    #[napi(constructor)]
    pub fn new(period: u32, mar: f64) -> napi::Result<Self> {
        Ok(Self {
            inner: wc::SortinoRatio::new(period as usize, mar).map_err(map_err)?,
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

#[napi(js_name = "CalmarRatio")]
pub struct CalmarRatioNode {
    inner: wc::CalmarRatio,
}

#[napi]
impl CalmarRatioNode {
    #[napi(constructor)]
    pub fn new(period: u32) -> napi::Result<Self> {
        Ok(Self {
            inner: wc::CalmarRatio::new(period as usize).map_err(map_err)?,
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

#[napi(js_name = "OmegaRatio")]
pub struct OmegaRatioNode {
    inner: wc::OmegaRatio,
}

#[napi]
impl OmegaRatioNode {
    #[napi(constructor)]
    pub fn new(period: u32, threshold: f64) -> napi::Result<Self> {
        Ok(Self {
            inner: wc::OmegaRatio::new(period as usize, threshold).map_err(map_err)?,
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

#[napi(js_name = "MaxDrawdown")]
pub struct MaxDrawdownNode {
    inner: wc::MaxDrawdown,
}

#[napi]
impl MaxDrawdownNode {
    #[napi(constructor)]
    pub fn new(period: u32) -> napi::Result<Self> {
        Ok(Self {
            inner: wc::MaxDrawdown::new(period as usize).map_err(map_err)?,
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

#[napi(js_name = "AverageDrawdown")]
pub struct AverageDrawdownNode {
    inner: wc::AverageDrawdown,
}

#[napi]
impl AverageDrawdownNode {
    #[napi(constructor)]
    pub fn new(period: u32) -> napi::Result<Self> {
        Ok(Self {
            inner: wc::AverageDrawdown::new(period as usize).map_err(map_err)?,
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

#[napi(js_name = "DrawdownDuration")]
pub struct DrawdownDurationNode {
    inner: wc::DrawdownDuration,
}

impl Default for DrawdownDurationNode {
    fn default() -> Self {
        Self::new()
    }
}

#[napi]
impl DrawdownDurationNode {
    #[napi(constructor)]
    pub fn new() -> Self {
        Self {
            inner: wc::DrawdownDuration::new(),
        }
    }
    #[napi]
    pub fn update(&mut self, value: f64) -> Option<u32> {
        self.inner.update(value)
    }
    #[napi]
    pub fn batch(&mut self, prices: Vec<f64>) -> Vec<f64> {
        prices
            .iter()
            .map(|p| self.inner.update(*p).map_or(f64::NAN, f64::from))
            .collect()
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

#[napi(js_name = "PainIndex")]
pub struct PainIndexNode {
    inner: wc::PainIndex,
}

#[napi]
impl PainIndexNode {
    #[napi(constructor)]
    pub fn new(period: u32) -> napi::Result<Self> {
        Ok(Self {
            inner: wc::PainIndex::new(period as usize).map_err(map_err)?,
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

#[napi(js_name = "ValueAtRisk")]
pub struct ValueAtRiskNode {
    inner: wc::ValueAtRisk,
}

#[napi]
impl ValueAtRiskNode {
    #[napi(constructor)]
    pub fn new(period: u32, confidence: f64) -> napi::Result<Self> {
        Ok(Self {
            inner: wc::ValueAtRisk::new(period as usize, confidence).map_err(map_err)?,
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

#[napi(js_name = "ConditionalValueAtRisk")]
pub struct ConditionalValueAtRiskNode {
    inner: wc::ConditionalValueAtRisk,
}

#[napi]
impl ConditionalValueAtRiskNode {
    #[napi(constructor)]
    pub fn new(period: u32, confidence: f64) -> napi::Result<Self> {
        Ok(Self {
            inner: wc::ConditionalValueAtRisk::new(period as usize, confidence).map_err(map_err)?,
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

#[napi(js_name = "ProfitFactor")]
pub struct ProfitFactorNode {
    inner: wc::ProfitFactor,
}

#[napi]
impl ProfitFactorNode {
    #[napi(constructor)]
    pub fn new(period: u32) -> napi::Result<Self> {
        Ok(Self {
            inner: wc::ProfitFactor::new(period as usize).map_err(map_err)?,
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

#[napi(js_name = "GainLossRatio")]
pub struct GainLossRatioNode {
    inner: wc::GainLossRatio,
}

#[napi]
impl GainLossRatioNode {
    #[napi(constructor)]
    pub fn new(period: u32) -> napi::Result<Self> {
        Ok(Self {
            inner: wc::GainLossRatio::new(period as usize).map_err(map_err)?,
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

#[napi(js_name = "RecoveryFactor")]
pub struct RecoveryFactorNode {
    inner: wc::RecoveryFactor,
}

impl Default for RecoveryFactorNode {
    fn default() -> Self {
        Self::new()
    }
}

#[napi]
impl RecoveryFactorNode {
    #[napi(constructor)]
    pub fn new() -> Self {
        Self {
            inner: wc::RecoveryFactor::new(),
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

#[napi(js_name = "KellyCriterion")]
pub struct KellyCriterionNode {
    inner: wc::KellyCriterion,
}

#[napi]
impl KellyCriterionNode {
    #[napi(constructor)]
    pub fn new(period: u32) -> napi::Result<Self> {
        Ok(Self {
            inner: wc::KellyCriterion::new(period as usize).map_err(map_err)?,
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

// --- Two-series (asset, benchmark) indicators ---
//
// Family 12 (statistik-regression, PR #51) introduces a
// `node_pair_indicator!` macro for Pearson / Beta / Spearman. Family 12 is
// not yet in main, so Family 15 inlines its pair wrappers below by hand.
// When PR #51 lands, the merge conflict on this file is resolved by keeping
// the macro from Family 12 and re-using it for Treynor / IR / Alpha.

#[napi(js_name = "TreynorRatio")]
pub struct TreynorRatioNode {
    inner: wc::TreynorRatio,
}

#[napi]
impl TreynorRatioNode {
    #[napi(constructor)]
    pub fn new(period: u32, risk_free: f64) -> napi::Result<Self> {
        Ok(Self {
            inner: wc::TreynorRatio::new(period as usize, risk_free).map_err(map_err)?,
        })
    }
    #[napi]
    pub fn update(&mut self, asset: f64, benchmark: f64) -> Option<f64> {
        self.inner.update((asset, benchmark))
    }
    #[napi]
    pub fn batch(&mut self, asset: Vec<f64>, benchmark: Vec<f64>) -> napi::Result<Vec<f64>> {
        if asset.len() != benchmark.len() {
            return Err(NapiError::from_reason(
                "asset and benchmark must be equal length".to_string(),
            ));
        }
        let mut out = Vec::with_capacity(asset.len());
        for i in 0..asset.len() {
            out.push(
                self.inner
                    .update((asset[i], benchmark[i]))
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

#[napi(js_name = "InformationRatio")]
pub struct InformationRatioNode {
    inner: wc::InformationRatio,
}

#[napi]
impl InformationRatioNode {
    #[napi(constructor)]
    pub fn new(period: u32) -> napi::Result<Self> {
        Ok(Self {
            inner: wc::InformationRatio::new(period as usize).map_err(map_err)?,
        })
    }
    #[napi]
    pub fn update(&mut self, asset: f64, benchmark: f64) -> Option<f64> {
        self.inner.update((asset, benchmark))
    }
    #[napi]
    pub fn batch(&mut self, asset: Vec<f64>, benchmark: Vec<f64>) -> napi::Result<Vec<f64>> {
        if asset.len() != benchmark.len() {
            return Err(NapiError::from_reason(
                "asset and benchmark must be equal length".to_string(),
            ));
        }
        let mut out = Vec::with_capacity(asset.len());
        for i in 0..asset.len() {
            out.push(
                self.inner
                    .update((asset[i], benchmark[i]))
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

#[napi(js_name = "Alpha")]
pub struct AlphaNode {
    inner: wc::Alpha,
}

#[napi]
impl AlphaNode {
    #[napi(constructor)]
    pub fn new(period: u32, risk_free: f64) -> napi::Result<Self> {
        Ok(Self {
            inner: wc::Alpha::new(period as usize, risk_free).map_err(map_err)?,
        })
    }
    #[napi]
    pub fn update(&mut self, asset: f64, benchmark: f64) -> Option<f64> {
        self.inner.update((asset, benchmark))
    }
    #[napi]
    pub fn batch(&mut self, asset: Vec<f64>, benchmark: Vec<f64>) -> napi::Result<Vec<f64>> {
        if asset.len() != benchmark.len() {
            return Err(NapiError::from_reason(
                "asset and benchmark must be equal length".to_string(),
            ));
        }
        let mut out = Vec::with_capacity(asset.len());
        for i in 0..asset.len() {
            out.push(
                self.inner
                    .update((asset[i], benchmark[i]))
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
