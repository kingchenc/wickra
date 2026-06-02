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
// The libtest harness collects every `#[test]` into a compiler-generated array
// of test references. With 2000+ unit tests that array exceeds clippy's 16 KB
// `large_stack_arrays` threshold; the diagnostic is spanless libtest scaffolding,
// not our code, so it cannot be silenced at a call site. Suppress it only in test
// builds — library code is still linted for genuinely large stack arrays.
#![cfg_attr(test, allow(clippy::large_stack_arrays))]

mod derivatives;
mod error;
mod microstructure;
mod ohlcv;
mod traits;

pub mod indicators;

pub use derivatives::DerivativesTick;
pub use error::{Error, Result};
pub use indicators::{
    AbandonedBaby, AccelerationBands, AccelerationBandsOutput, AcceleratorOscillator, AdOscillator,
    AdaptiveCycle, Adl, AdvanceBlock, Adx, AdxOutput, Adxr, Alligator, AlligatorOutput, Alma,
    Alpha, AnchoredRsi, AnchoredVwap, Apo, Aroon, AroonOscillator, AroonOutput, Atr, AtrBands,
    AtrBandsOutput, AtrTrailingStop, Autocorrelation, AverageDrawdown, AwesomeOscillator,
    AwesomeOscillatorHistogram, BalanceOfPower, BeltHold, Beta, BollingerBands, BollingerBandwidth,
    BollingerOutput, Breakaway, CalendarSpread, CalmarRatio, Camarilla, CamarillaPivotsOutput, Cci,
    CenterOfGravity, Cfo, ChaikinMoneyFlow, ChaikinOscillator, ChaikinVolatility, ChandeKrollStop,
    ChandeKrollStopOutput, ChandelierExit, ChandelierExitOutput, ChoppinessIndex, ClassicPivots,
    ClassicPivotsOutput, ClosingMarubozu, Cmo, CoefficientOfVariation, Cointegration,
    CointegrationOutput, ConcealingBabySwallow, ConditionalValueAtRisk, ConnorsRsi, Coppock,
    Counterattack, CumulativeVolumeDelta, CyberneticCycle, Decycler, DecyclerOscillator, Dema,
    DemandIndex, DemarkPivots, DemarkPivotsOutput, DepthSlope, DetrendedStdDev, Doji, DojiStar,
    Donchian, DonchianOutput, DonchianStop, DonchianStopOutput, DoubleBollinger,
    DoubleBollingerOutput, DownsideGapThreeMethods, Dpo, DragonflyDoji, DrawdownDuration,
    EaseOfMovement, EffectiveSpread, EhlersStochastic, ElderImpulse, Ema,
    EmpiricalModeDecomposition, Engulfing, EveningDojiStar, Evwma, FallingThreeMethods, Fama,
    FibonacciPivots, FibonacciPivotsOutput, FisherTransform, Footprint, FootprintOutput,
    ForceIndex, FractalChaosBands, FractalChaosBandsOutput, Frama, FundingBasis, FundingRate,
    FundingRateMean, FundingRateZScore, GainLossRatio, GapSideBySideWhite, GarmanKlassVolatility,
    GravestoneDoji, Hammer, HangingMan, Harami, HeikinAshi, HeikinAshiOutput, HiLoActivator,
    HighWave, Hikkake, HikkakeModified, HilbertDominantCycle, HistoricalVolatility, Hma,
    HomingPigeon, HurstChannel, HurstChannelOutput, HurstExponent, Ichimoku, IchimokuOutput,
    IdenticalThreeCrows, InNeck, Inertia, InformationRatio, InitialBalance, InitialBalanceOutput,
    InstantaneousTrendline, InverseFisherTransform, InvertedHammer, Jma, Kama, KellyCriterion,
    Keltner, KeltnerOutput, Kicking, KickingByLength, Kst, KstOutput, Kurtosis, Kvo, KylesLambda,
    LadderBottom, LaguerreRsi, LeadLagCrossCorrelation, LeadLagCrossCorrelationOutput, LinRegAngle,
    LinRegChannel, LinRegChannelOutput, LinRegSlope, LinearRegression, LiquidationFeatures,
    LiquidationFeaturesOutput, LongLeggedDoji, LongLine, LongShortRatio, MaEnvelope,
    MaEnvelopeOutput, MacdIndicator, MacdOutput, Mama, MamaOutput, MarketFacilitationIndex,
    Marubozu, MassIndex, MatHold, MatchingLow, MaxDrawdown, McGinleyDynamic,
    MedianAbsoluteDeviation, MedianPrice, Mfi, Microprice, Mom, MorningDojiStar,
    MorningEveningStar, Natr, Nvi, OIPriceDivergence, OIWeighted, Obv, OmegaRatio, OnNeck,
    OpenInterestDelta, OpeningMarubozu, OpeningRange, OpeningRangeOutput, OrderBookImbalanceFull,
    OrderBookImbalanceTop1, OrderBookImbalanceTopN, PainIndex, PairSpreadZScore, PairwiseBeta,
    ParkinsonVolatility, PearsonCorrelation, PercentB, PercentageTrailingStop, Pgo,
    PiercingDarkCloud, Pmo, Ppo, ProfitFactor, Psar, Pvi, QuotedSpread, RSquared, RealizedSpread,
    RecoveryFactor, RelativeStrengthAB, RelativeStrengthOutput, RenkoTrailingStop, RickshawMan,
    RisingThreeMethods, Roc, RogersSatchellVolatility, RollingVwap, RoofingFilter, Rsi, Rvi,
    RviVolatility, Rwi, RwiOutput, SeparatingLines, SharpeRatio, ShootingStar, ShortLine,
    SignedVolume, SineWave, Skewness, Sma, Smi, Smma, SortinoRatio, SpearmanCorrelation,
    SpinningTop, StalledPattern, StandardError, StandardErrorBands, StandardErrorBandsOutput,
    StarcBands, StarcBandsOutput, Stc, StdDev, StepTrailingStop, StickSandwich, StochRsi,
    Stochastic, StochasticOutput, SuperSmoother, SuperTrend, SuperTrendOutput, TakerBuySellRatio,
    Takuri, TasukiGap, TdCombo, TdCountdown, TdDeMarker, TdDifferential, TdLines, TdLinesOutput,
    TdOpen, TdPressure, TdRangeProjection, TdRangeProjectionOutput, TdRei, TdRiskLevel,
    TdRiskLevelOutput, TdSequential, TdSequentialOutput, TdSetup, Tema, TermStructureBasis,
    ThreeInside, ThreeLineStrike, ThreeOutside, ThreeSoldiersOrCrows, ThreeStarsInSouth, Thrusting,
    Tii, TradeImbalance, TreynorRatio, Trima, Trix, TrueRange, Tsi, Tsv, TtmSqueeze,
    TtmSqueezeOutput, Tweezer, TwoCrows, TypicalPrice, UlcerIndex, UltimateOscillator,
    UniqueThreeRiver, UpsideGapThreeMethods, UpsideGapTwoCrows, ValueArea, ValueAreaOutput,
    ValueAtRisk, Variance, VerticalHorizontalFilter, Vidya, VoltyStop, VolumeOscillator,
    VolumePriceTrend, Vortex, VortexOutput, Vwap, VwapStdDevBands, VwapStdDevBandsOutput, Vwma,
    Vzo, WaveTrend, WaveTrendOutput, WeightedClose, WilliamsFractals, WilliamsFractalsOutput,
    WilliamsR, Wma, WoodiePivots, WoodiePivotsOutput, YangZhangVolatility, YoyoExit, ZScore,
    ZeroLagMacd, ZeroLagMacdOutput, ZigZag, ZigZagOutput, Zlema, FAMILIES, T3,
};
// `FootprintLevel` is a row element of `FootprintOutput`, re-exported on its own
// line so the indicator-count tooling (which scans the braced block above and
// strips only `*Output` companions) does not count it as a separate indicator.
pub use indicators::FootprintLevel;
pub use microstructure::{Level, OrderBook, Side, Trade, TradeQuote};
pub use ohlcv::{Candle, Tick};
pub use traits::{BatchExt, Chain, Indicator};
