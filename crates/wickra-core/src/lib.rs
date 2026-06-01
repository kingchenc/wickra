//! `wickra-core`: streaming-first technical indicators.
//!
//! The core engine of Wickra. Every indicator is implemented as a state machine
//! that consumes inputs one at a time via [`Indicator::update`] in constant time.
//! Batch evaluation is provided as a blanket extension trait so the same code
//! path serves both online (tick-by-tick) and offline (historical) workloads.
//!
//! # Design
//!
//! - **Streaming-first.** State is held by the indicator instance, so a new value
//!   only re-computes deltas, not the whole series.
//! - **Batch is free.** [`BatchExt::batch`] is a blanket implementation that
//!   simply replays `update` over a slice. Writing one implementation gives both
//!   APIs.
//! - **Composable.** Indicators implement [`Indicator<Input = f64, Output = f64>`]
//!   wherever they conceptually take a price, so they can be chained via
//!   [`Chain`].
//! - **No `unsafe`.** The crate forbids `unsafe_code` in the workspace lints.
//!
//! # Quick start
//!
//! ```
//! use wickra_core::{BatchExt, Indicator, Sma};
//!
//! // Streaming:
//! let mut sma = Sma::new(3).unwrap();
//! assert_eq!(sma.update(1.0), None);
//! assert_eq!(sma.update(2.0), None);
//! assert_eq!(sma.update(3.0), Some(2.0));
//!
//! // Batch (replays `update` internally):
//! let mut sma = Sma::new(3).unwrap();
//! let out = sma.batch(&[1.0, 2.0, 3.0, 4.0]);
//! assert_eq!(out, vec![None, None, Some(2.0), Some(3.0)]);
//! ```

#![cfg_attr(docsrs, feature(doc_cfg))]

mod error;
mod microstructure;
mod ohlcv;
mod traits;

pub mod indicators;

pub use error::{Error, Result};
pub use indicators::{
    AccelerationBands, AccelerationBandsOutput, AcceleratorOscillator, AdOscillator, AdaptiveCycle,
    Adl, Adx, AdxOutput, Adxr, Alligator, AlligatorOutput, Alma, Alpha, AnchoredVwap, Apo, Aroon,
    AroonOscillator, AroonOutput, Atr, AtrBands, AtrBandsOutput, AtrTrailingStop, Autocorrelation,
    AverageDrawdown, AwesomeOscillator, AwesomeOscillatorHistogram, BalanceOfPower, Beta,
    BollingerBands, BollingerBandwidth, BollingerOutput, CalmarRatio, Camarilla,
    CamarillaPivotsOutput, Cci, CenterOfGravity, Cfo, ChaikinMoneyFlow, ChaikinOscillator,
    ChaikinVolatility, ChandeKrollStop, ChandeKrollStopOutput, ChandelierExit,
    ChandelierExitOutput, ChoppinessIndex, ClassicPivots, ClassicPivotsOutput, Cmo,
    CoefficientOfVariation, Cointegration, CointegrationOutput, ConditionalValueAtRisk, ConnorsRsi,
    Coppock, CumulativeVolumeDelta, CyberneticCycle, Decycler, DecyclerOscillator, Dema,
    DemandIndex, DemarkPivots, DemarkPivotsOutput, DetrendedStdDev, Doji, Donchian, DonchianOutput,
    DonchianStop, DonchianStopOutput, DoubleBollinger, DoubleBollingerOutput, Dpo,
    DrawdownDuration, EaseOfMovement, EffectiveSpread, EhlersStochastic, ElderImpulse, Ema,
    EmpiricalModeDecomposition, Engulfing, Evwma, Fama, FibonacciPivots, FibonacciPivotsOutput,
    FisherTransform, ForceIndex, FractalChaosBands, FractalChaosBandsOutput, Frama, GainLossRatio,
    GarmanKlassVolatility, Hammer, HangingMan, Harami, HeikinAshi, HeikinAshiOutput, HiLoActivator,
    HilbertDominantCycle, HistoricalVolatility, Hma, HurstChannel, HurstChannelOutput,
    HurstExponent, Ichimoku, IchimokuOutput, Inertia, InformationRatio, InitialBalance,
    InitialBalanceOutput, InstantaneousTrendline, InverseFisherTransform, InvertedHammer, Jma,
    Kama, KellyCriterion, Keltner, KeltnerOutput, Kst, KstOutput, Kurtosis, Kvo, KylesLambda,
    LaguerreRsi, LeadLagCrossCorrelation, LeadLagCrossCorrelationOutput, LinRegAngle,
    LinRegChannel, LinRegChannelOutput, LinRegSlope, LinearRegression, MaEnvelope,
    MaEnvelopeOutput, MacdIndicator, MacdOutput, Mama, MamaOutput, MarketFacilitationIndex,
    Marubozu, MassIndex, MaxDrawdown, McGinleyDynamic, MedianAbsoluteDeviation, MedianPrice, Mfi,
    Microprice, Mom, MorningEveningStar, Natr, Nvi, Obv, OmegaRatio, OpeningRange,
    OpeningRangeOutput, OrderBookImbalanceFull, OrderBookImbalanceTop1, OrderBookImbalanceTopN,
    PainIndex, PairSpreadZScore, PairwiseBeta, ParkinsonVolatility, PearsonCorrelation, PercentB,
    PercentageTrailingStop, Pgo, PiercingDarkCloud, Pmo, Ppo, ProfitFactor, Psar, Pvi,
    QuotedSpread, RSquared, RealizedSpread, RecoveryFactor, RelativeStrengthAB,
    RelativeStrengthOutput, RenkoTrailingStop, Roc, RogersSatchellVolatility, RollingVwap,
    RoofingFilter, Rsi, Rvi, RviVolatility, Rwi, RwiOutput, SharpeRatio, ShootingStar,
    SignedVolume, SineWave, Skewness, Sma, Smi, Smma, SortinoRatio, SpearmanCorrelation,
    SpinningTop, StandardError, StandardErrorBands, StandardErrorBandsOutput, StarcBands,
    StarcBandsOutput, Stc, StdDev, StepTrailingStop, StochRsi, Stochastic, StochasticOutput,
    SuperSmoother, SuperTrend, SuperTrendOutput, TdCombo, TdCountdown, TdDeMarker, TdDifferential,
    TdLines, TdLinesOutput, TdOpen, TdPressure, TdRangeProjection, TdRangeProjectionOutput, TdRei,
    TdRiskLevel, TdRiskLevelOutput, TdSequential, TdSequentialOutput, TdSetup, Tema, ThreeInside,
    ThreeOutside, ThreeSoldiersOrCrows, Tii, TradeImbalance, TreynorRatio, Trima, Trix, TrueRange,
    Tsi, Tsv, TtmSqueeze, TtmSqueezeOutput, Tweezer, TypicalPrice, UlcerIndex, UltimateOscillator,
    ValueArea, ValueAreaOutput, ValueAtRisk, Variance, VerticalHorizontalFilter, Vidya, VoltyStop,
    VolumeOscillator, VolumePriceTrend, Vortex, VortexOutput, Vwap, VwapStdDevBands,
    VwapStdDevBandsOutput, Vwma, Vzo, WaveTrend, WaveTrendOutput, WeightedClose, WilliamsFractals,
    WilliamsFractalsOutput, WilliamsR, Wma, WoodiePivots, WoodiePivotsOutput, YangZhangVolatility,
    YoyoExit, ZScore, ZeroLagMacd, ZeroLagMacdOutput, ZigZag, ZigZagOutput, Zlema, FAMILIES, T3,
};
pub use microstructure::{Level, OrderBook, Side, Trade, TradeQuote};
pub use ohlcv::{Candle, Tick};
pub use traits::{BatchExt, Chain, Indicator};
