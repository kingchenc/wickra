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

mod calendar;
mod cross_section;
mod derivatives;
mod error;
mod microstructure;
mod ohlcv;
mod traits;

pub mod indicators;

pub use cross_section::{CrossSection, Member};
pub use derivatives::DerivativesTick;
pub use error::{Error, Result};
pub use indicators::{
    AbandonedBaby, Abcd, AbsoluteBreadthIndex, AccelerationBands, AccelerationBandsOutput,
    AcceleratorOscillator, AdOscillator, AdVolumeLine, AdaptiveCci, AdaptiveCycle,
    AdaptiveLaguerreFilter, AdaptiveRsi, Adl, AdvanceBlock, AdvanceDecline, AdvanceDeclineRatio,
    Adx, AdxOutput, Adxr, Alligator, AlligatorOutput, Alma, Alpha, AmihudIlliquidity, AnchoredRsi,
    AnchoredVwap, AndrewsPitchfork, AndrewsPitchforkOutput, Apo, Aroon, AroonOscillator,
    AroonOutput, Atr, AtrBands, AtrBandsOutput, AtrRatchet, AtrRatchetOutput, AtrTrailingStop,
    AutoFib, AutoFibOutput, Autocorrelation, AutocorrelationPeriodogram, AverageDailyRange,
    AverageDrawdown, AvgPrice, AwesomeOscillator, AwesomeOscillatorHistogram, BalanceOfPower,
    BandpassFilter, Bat, BeltHold, Beta, BetaNeutralSpread, BetterVolume, BipowerVariation,
    BodySizePct, BollingerBands, BollingerBandwidth, BollingerOutput, BomarBands, BomarBandsOutput,
    BreadthThrust, Breakaway, BullishPercentIndex, Butterfly, CalendarSpread, CalmarRatio,
    Camarilla, CamarillaPivotsOutput, Cci, CenterOfGravity, CentralPivotRange,
    CentralPivotRangeOutput, Cfo, ChaikinMoneyFlow, ChaikinOscillator, ChaikinVolatility,
    ChandeKrollStop, ChandeKrollStopOutput, ChandelierExit, ChandelierExitOutput, ChoppinessIndex,
    ClassicPivots, ClassicPivotsOutput, CloseVsOpen, ClosingMarubozu, Cmo, CoefficientOfVariation,
    Cointegration, CointegrationOutput, ConcealingBabySwallow, ConditionalValueAtRisk, ConnorsRsi,
    Coppock, CorrelationTrendIndicator, Counterattack, Crab, CumulativeVolumeDelta,
    CumulativeVolumeIndex, CupAndHandle, CyberneticCycle, Cypher, DayOfWeekProfile,
    DayOfWeekProfileOutput, Decycler, DecyclerOscillator, Dema, DemandIndex, DemarkPivots,
    DemarkPivotsOutput, DepthSlope, DerivativeOscillator, DetrendedStdDev, DisparityIndex,
    DistanceSsd, Doji, DojiStar, Donchian, DonchianOutput, DonchianStop, DonchianStopOutput,
    DoubleBollinger, DoubleBollingerOutput, DoubleTopBottom, DownsideGapThreeMethods, Dpo,
    DragonflyDoji, DrawdownDuration, Dx, DynamicMomentumIndex, EaseOfMovement, EffectiveSpread,
    EhlersStochastic, Ehma, ElderImpulse, ElderRay, ElderRayOutput, ElderSafeZone,
    ElderSafeZoneOutput, Ema, EmpiricalModeDecomposition, Engulfing, EvenBetterSinewave,
    EveningDojiStar, Evwma, EwmaVolatility, Expectancy, FallingThreeMethods, Fama, FibArcs,
    FibArcsOutput, FibChannel, FibChannelOutput, FibConfluence, FibConfluenceOutput, FibExtension,
    FibExtensionOutput, FibFan, FibFanOutput, FibProjection, FibProjectionOutput, FibRetracement,
    FibRetracementOutput, FibTimeZones, FibTimeZonesOutput, FibonacciPivots, FibonacciPivotsOutput,
    FisherRsi, FisherTransform, FlagPennant, Footprint, FootprintOutput, ForceIndex,
    FractalChaosBands, FractalChaosBandsOutput, Frama, FundingBasis, FundingRate, FundingRateMean,
    FundingRateZScore, GainLossRatio, GapSideBySideWhite, Garch11, GarmanKlassVolatility, Gartley,
    GatorOscillator, GatorOscillatorOutput, GeneralizedDema, GeometricMa, GoldenPocket,
    GoldenPocketOutput, GrangerCausality, GravestoneDoji, Hammer, HangingMan, Harami,
    HeadAndShoulders, HeikinAshi, HeikinAshiOutput, HiLoActivator, HighLowIndex, HighLowRange,
    HighWave, HighpassFilter, Hikkake, HikkakeModified, HilbertDominantCycle, HistoricalVolatility,
    Hma, HoltWinters, HomingPigeon, HtDcPhase, HtPhasor, HtPhasorOutput, HtTrendMode, HurstChannel,
    HurstChannelOutput, HurstExponent, Ichimoku, IchimokuOutput, IdenticalThreeCrows, InNeck,
    Inertia, InformationRatio, InitialBalance, InitialBalanceOutput, InstantaneousTrendline,
    IntradayIntensity, IntradayMomentumIndex, IntradayVolatilityProfile,
    IntradayVolatilityProfileOutput, InverseFisherTransform, InvertedHammer, JarqueBera, Jma,
    JumpIndicator, KagiBars, KalmanHedgeRatio, KalmanHedgeRatioOutput, Kama, KaseDevStop,
    KaseDevStopOutput, KasePermissionStochastic, KasePermissionStochasticOutput, KellyCriterion,
    Keltner, KeltnerOutput, KendallTau, Kicking, KickingByLength, Kst, KstOutput, Kurtosis, Kvo,
    KylesLambda, LadderBottom, LaguerreRsi, LeadLagCrossCorrelation, LeadLagCrossCorrelationOutput,
    LinRegAngle, LinRegChannel, LinRegChannelOutput, LinRegIntercept, LinRegSlope,
    LinearRegression, LiquidationFeatures, LiquidationFeaturesOutput, LogReturn, LongLeggedDoji,
    LongLine, LongShortRatio, MaEnvelope, MaEnvelopeOutput, MacdExt, MacdFix, MacdHistogram,
    MacdIndicator, MacdOutput, Mama, MamaOutput, MarketFacilitationIndex, Marubozu, MassIndex,
    MatHold, MatchingLow, MaxDrawdown, McClellanOscillator, McClellanSummationIndex,
    McGinleyDynamic, MedianAbsoluteDeviation, MedianChannel, MedianChannelOutput, MedianMa,
    MedianPrice, Mfi, Microprice, MidPoint, MidPrice, MinusDi, MinusDm, ModifiedMaStop,
    ModifiedMaStopOutput, Mom, MorningDojiStar, MorningEveningStar, MurreyMathLines,
    MurreyMathLinesOutput, Natr, NewHighsNewLows, Nrtr, NrtrOutput, Nvi, OIPriceDivergence,
    OIWeighted, Obv, OmegaRatio, OnNeck, OpenInterestDelta, OpeningMarubozu, OpeningRange,
    OpeningRangeOutput, OrderBookImbalanceFull, OrderBookImbalanceTop1, OrderBookImbalanceTopN,
    OrderFlowImbalance, OuHalfLife, OvernightGap, OvernightIntradayReturn,
    OvernightIntradayReturnOutput, PainIndex, PairSpreadZScore, PairwiseBeta, ParkinsonVolatility,
    PearsonCorrelation, PercentAboveMa, PercentB, PercentageTrailingStop, Pgo, PiercingDarkCloud,
    PivotReversal, PlusDi, PlusDm, Pmo, PointAndFigureBars, PolarizedFractalEfficiency, Ppo,
    PpoHistogram, ProfitFactor, ProjectionBands, ProjectionBandsOutput, ProjectionOscillator, Psar,
    Pvi, Qqe, QqeOutput, Qstick, QuartileBands, QuartileBandsOutput, QuotedSpread, RSquared,
    RealizedSpread, RealizedVolatility, RecoveryFactor, RectangleRange, Reflex, RegimeLabel,
    RelativeStrengthAB, RelativeStrengthOutput, RenkoBars, RenkoTrailingStop, RickshawMan,
    RisingThreeMethods, Rmi, Roc, Rocp, Rocr, Rocr100, RogersSatchellVolatility, RollMeasure,
    RollingCorrelation, RollingCovariance, RollingIqr, RollingMinMaxScaler, RollingPercentileRank,
    RollingQuantile, RollingVwap, RoofingFilter, Rsi, Rsx, Rvi, RviVolatility, Rwi, RwiOutput,
    SampleEntropy, SarExt, SeasonalZScore, SeparatingLines, SessionHighLow, SessionHighLowOutput,
    SessionRange, SessionRangeOutput, SessionVwap, ShannonEntropy, Shark, SharpeRatio,
    ShootingStar, ShortLine, SignedVolume, SineWave, SineWeightedMa, Skewness, Sma, Smi, Smma,
    SortinoRatio, SpearmanCorrelation, SpinningTop, SpreadAr1Coefficient, SpreadBollingerBands,
    SpreadBollingerBandsOutput, SpreadHurst, StalledPattern, StandardError, StandardErrorBands,
    StandardErrorBandsOutput, StarcBands, StarcBandsOutput, Stc, StdDev, StepTrailingStop,
    StickSandwich, StochRsi, Stochastic, StochasticCci, StochasticOutput, SuperSmoother,
    SuperTrend, SuperTrendOutput, TakerBuySellRatio, Takuri, TasukiGap, TdCombo, TdCountdown,
    TdDeMarker, TdDifferential, TdLines, TdLinesOutput, TdOpen, TdPressure, TdRangeProjection,
    TdRangeProjectionOutput, TdRei, TdRiskLevel, TdRiskLevelOutput, TdSequential,
    TdSequentialOutput, TdSetup, Tema, TermStructureBasis, ThreeDrives, ThreeInside,
    ThreeLineStrike, ThreeOutside, ThreeSoldiersOrCrows, ThreeStarsInSouth, Thrusting, TickIndex,
    Tii, TimeBasedStop, TimeOfDayReturnProfile, TimeOfDayReturnProfileOutput, TpoProfile,
    TpoProfileOutput, TradeImbalance, TradeVolumeIndex, TrendLabel, TrendStrengthIndex, Trendflex,
    TreynorRatio, Triangle, Trima, Trin, TripleTopBottom, Trix, TrueRange, Tsf, TsfOscillator, Tsi,
    Tsv, TtmSqueeze, TtmSqueezeOutput, TtmTrend, TurnOfMonth, Tweezer, TwiggsMoneyFlow, TwoCrows,
    TypicalPrice, UlcerIndex, UltimateOscillator, UniqueThreeRiver, UniversalOscillator,
    UpDownVolumeRatio, UpsideGapThreeMethods, UpsideGapTwoCrows, ValueArea, ValueAreaOutput,
    ValueAtRisk, Variance, VarianceRatio, VerticalHorizontalFilter, Vidya, VolatilityCone,
    VolatilityConeOutput, VolatilityOfVolatility, VolatilityRatio, VoltyStop, VolumeByTimeProfile,
    VolumeByTimeProfileOutput, VolumeOscillator, VolumePriceTrend, VolumeProfile,
    VolumeProfileOutput, VolumeRsi, VolumeWeightedMacd, VolumeWeightedMacdOutput, VolumeWeightedSr,
    VolumeWeightedSrOutput, Vortex, VortexOutput, Vpin, Vwap, VwapStdDevBands,
    VwapStdDevBandsOutput, Vwma, Vzo, Wad, WavePm, WaveTrend, WaveTrendOutput, Wedge,
    WeightedClose, WickRatio, WilliamsFractals, WilliamsFractalsOutput, WilliamsR, WinRate, Wma,
    WoodiePivots, WoodiePivotsOutput, YangZhangVolatility, YoyoExit, ZScore, ZeroLagMacd,
    ZeroLagMacdOutput, ZigZag, ZigZagOutput, Zlema, FAMILIES, T3,
};
// `FootprintLevel` is a row element of `FootprintOutput`, re-exported on its own
// line so the indicator-count tooling (which scans the braced block above and
// strips only `*Output` companions) does not count it as a separate indicator.
pub use indicators::FootprintLevel;
// `MaType` is a moving-average selector enum used by `MacdExt`, re-exported on
// its own line so the indicator-count tooling does not count it as an indicator.
pub use indicators::MaType;
// Bar element types for the alt-chart builders, re-exported on their own lines so
// the indicator-count tooling (which scans only the braced block above) does not
// count them as separate indicators.
pub use indicators::KagiBar;
pub use indicators::PnfColumn;
pub use indicators::RenkoBrick;
pub use microstructure::{Level, OrderBook, Side, Trade, TradeQuote};
pub use ohlcv::{Candle, Tick};
pub use traits::{BarBuilder, BatchExt, Chain, Indicator};
