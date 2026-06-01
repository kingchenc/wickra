//! Built-in indicators. Every indicator implements [`crate::Indicator`].
//!
//! Modules are listed alphabetically; the canonical family taxonomy lives in
//! [`FAMILIES`]. Every public name is re-exported flat from this module and
//! from the crate root for convenience.

mod acceleration_bands;
mod accelerator_oscillator;
mod ad_oscillator;
mod adaptive_cycle;
mod adl;
mod adx;
mod adxr;
mod alligator;
mod alma;
mod alpha;
mod anchored_vwap;
mod apo;
mod aroon;
mod aroon_oscillator;
mod atr;
mod atr_bands;
mod atr_trailing_stop;
mod autocorrelation;
mod average_drawdown;
mod awesome_oscillator;
mod awesome_oscillator_histogram;
mod balance_of_power;
mod beta;
mod bollinger;
mod bollinger_bandwidth;
mod calmar_ratio;
mod camarilla_pivots;
mod cci;
mod center_of_gravity;
mod cfo;
mod chaikin_oscillator;
mod chaikin_volatility;
mod chande_kroll_stop;
mod chandelier_exit;
mod choppiness_index;
mod classic_pivots;
mod cmf;
mod cmo;
mod coefficient_of_variation;
mod conditional_value_at_risk;
mod connors_rsi;
mod coppock;
mod cybernetic_cycle;
mod decycler;
mod decycler_oscillator;
mod dema;
mod demand_index;
mod demark_pivots;
mod detrended_std_dev;
mod doji;
mod donchian;
mod donchian_stop;
mod double_bollinger;
mod dpo;
mod drawdown_duration;
mod ease_of_movement;
mod ehlers_stochastic;
mod elder_impulse;
mod ema;
mod empirical_mode_decomposition;
mod engulfing;
mod evwma;
mod fama;
mod fibonacci_pivots;
mod fisher_transform;
mod force_index;
mod fractal_chaos_bands;
mod frama;
mod gain_loss_ratio;
mod garman_klass;
mod hammer;
mod hanging_man;
mod harami;
mod heikin_ashi;
mod hilbert_dominant_cycle;
mod hilo_activator;
mod historical_volatility;
mod hma;
mod hurst_channel;
mod hurst_exponent;
mod ichimoku;
mod inertia;
mod information_ratio;
mod initial_balance;
mod instantaneous_trendline;
mod inverse_fisher_transform;
mod inverted_hammer;
mod jma;
mod kama;
mod kelly_criterion;
mod keltner;
mod kst;
mod kurtosis;
mod kvo;
mod laguerre_rsi;
mod linreg;
mod linreg_angle;
mod linreg_channel;
mod linreg_slope;
mod ma_envelope;
mod macd;
mod mama;
mod market_facilitation_index;
mod marubozu;
mod mass_index;
mod max_drawdown;
mod mcginley_dynamic;
mod median_absolute_deviation;
mod median_price;
mod mfi;
mod mom;
mod morning_evening_star;
mod natr;
mod nvi;
mod obv;
mod omega_ratio;
mod opening_range;
mod pain_index;
mod pairwise_beta;
mod parkinson;
mod pearson_correlation;
mod percent_b;
mod percentage_trailing_stop;
mod pgo;
mod piercing_dark_cloud;
mod pmo;
mod ppo;
mod profit_factor;
mod psar;
mod pvi;
mod r_squared;
mod recovery_factor;
mod renko_trailing_stop;
mod roc;
mod rogers_satchell;
mod roofing_filter;
mod rsi;
mod rvi;
mod rvi_volatility;
mod rwi;
mod sharpe_ratio;
mod shooting_star;
mod sine_wave;
mod skewness;
mod sma;
mod smi;
mod smma;
mod sortino_ratio;
mod spearman_correlation;
mod spinning_top;
mod standard_error;
mod standard_error_bands;
mod starc_bands;
mod stc;
mod std_dev;
mod step_trailing_stop;
mod stoch_rsi;
mod stochastic;
mod super_smoother;
mod super_trend;
mod t3;
mod td_combo;
mod td_countdown;
mod td_demarker;
mod td_differential;
mod td_lines;
mod td_open;
mod td_pressure;
mod td_range_projection;
mod td_rei;
mod td_risk_level;
mod td_sequential;
mod td_setup;
mod tema;
mod three_inside;
mod three_outside;
mod three_soldiers_or_crows;
mod tii;
mod treynor_ratio;
mod trima;
mod trix;
mod true_range;
mod tsi;
mod tsv;
mod ttm_squeeze;
mod tweezer;
mod typical_price;
mod ulcer_index;
mod ultimate_oscillator;
mod value_area;
mod value_at_risk;
mod variance;
mod vertical_horizontal_filter;
mod vidya;
mod volty_stop;
mod volume_oscillator;
mod vortex;
mod vpt;
mod vwap;
mod vwap_stddev_bands;
mod vwma;
mod vzo;
mod wave_trend;
mod weighted_close;
mod williams_fractals;
mod williams_r;
mod wma;
mod woodie_pivots;
mod yang_zhang;
mod yoyo_exit;
mod z_score;
mod zero_lag_macd;
mod zig_zag;
mod zlema;

pub use acceleration_bands::{AccelerationBands, AccelerationBandsOutput};
pub use accelerator_oscillator::AcceleratorOscillator;
pub use ad_oscillator::AdOscillator;
pub use adaptive_cycle::AdaptiveCycle;
pub use adl::Adl;
pub use adx::{Adx, AdxOutput};
pub use adxr::Adxr;
pub use alligator::{Alligator, AlligatorOutput};
pub use alma::Alma;
pub use alpha::Alpha;
pub use anchored_vwap::AnchoredVwap;
pub use apo::Apo;
pub use aroon::{Aroon, AroonOutput};
pub use aroon_oscillator::AroonOscillator;
pub use atr::Atr;
pub use atr_bands::{AtrBands, AtrBandsOutput};
pub use atr_trailing_stop::AtrTrailingStop;
pub use autocorrelation::Autocorrelation;
pub use average_drawdown::AverageDrawdown;
pub use awesome_oscillator::AwesomeOscillator;
pub use awesome_oscillator_histogram::AwesomeOscillatorHistogram;
pub use balance_of_power::BalanceOfPower;
pub use beta::Beta;
pub use bollinger::{BollingerBands, BollingerOutput};
pub use bollinger_bandwidth::BollingerBandwidth;
pub use calmar_ratio::CalmarRatio;
pub use camarilla_pivots::{Camarilla, CamarillaPivotsOutput};
pub use cci::Cci;
pub use center_of_gravity::CenterOfGravity;
pub use cfo::Cfo;
pub use chaikin_oscillator::ChaikinOscillator;
pub use chaikin_volatility::ChaikinVolatility;
pub use chande_kroll_stop::{ChandeKrollStop, ChandeKrollStopOutput};
pub use chandelier_exit::{ChandelierExit, ChandelierExitOutput};
pub use choppiness_index::ChoppinessIndex;
pub use classic_pivots::{ClassicPivots, ClassicPivotsOutput};
pub use cmf::ChaikinMoneyFlow;
pub use cmo::Cmo;
pub use coefficient_of_variation::CoefficientOfVariation;
pub use conditional_value_at_risk::ConditionalValueAtRisk;
pub use connors_rsi::ConnorsRsi;
pub use coppock::Coppock;
pub use cybernetic_cycle::CyberneticCycle;
pub use decycler::Decycler;
pub use decycler_oscillator::DecyclerOscillator;
pub use dema::Dema;
pub use demand_index::DemandIndex;
pub use demark_pivots::{DemarkPivots, DemarkPivotsOutput};
pub use detrended_std_dev::DetrendedStdDev;
pub use doji::Doji;
pub use donchian::{Donchian, DonchianOutput};
pub use donchian_stop::{DonchianStop, DonchianStopOutput};
pub use double_bollinger::{DoubleBollinger, DoubleBollingerOutput};
pub use dpo::Dpo;
pub use drawdown_duration::DrawdownDuration;
pub use ease_of_movement::EaseOfMovement;
pub use ehlers_stochastic::EhlersStochastic;
pub use elder_impulse::ElderImpulse;
pub use ema::Ema;
pub use empirical_mode_decomposition::EmpiricalModeDecomposition;
pub use engulfing::Engulfing;
pub use evwma::Evwma;
pub use fama::Fama;
pub use fibonacci_pivots::{FibonacciPivots, FibonacciPivotsOutput};
pub use fisher_transform::FisherTransform;
pub use force_index::ForceIndex;
pub use fractal_chaos_bands::{FractalChaosBands, FractalChaosBandsOutput};
pub use frama::Frama;
pub use gain_loss_ratio::GainLossRatio;
pub use garman_klass::GarmanKlassVolatility;
pub use hammer::Hammer;
pub use hanging_man::HangingMan;
pub use harami::Harami;
pub use heikin_ashi::{HeikinAshi, HeikinAshiOutput};
pub use hilbert_dominant_cycle::HilbertDominantCycle;
pub use hilo_activator::HiLoActivator;
pub use historical_volatility::HistoricalVolatility;
pub use hma::Hma;
pub use hurst_channel::{HurstChannel, HurstChannelOutput};
pub use hurst_exponent::HurstExponent;
pub use ichimoku::{Ichimoku, IchimokuOutput};
pub use inertia::Inertia;
pub use information_ratio::InformationRatio;
pub use initial_balance::{InitialBalance, InitialBalanceOutput};
pub use instantaneous_trendline::InstantaneousTrendline;
pub use inverse_fisher_transform::InverseFisherTransform;
pub use inverted_hammer::InvertedHammer;
pub use jma::Jma;
pub use kama::Kama;
pub use kelly_criterion::KellyCriterion;
pub use keltner::{Keltner, KeltnerOutput};
pub use kst::{Kst, KstOutput};
pub use kurtosis::Kurtosis;
pub use kvo::Kvo;
pub use laguerre_rsi::LaguerreRsi;
pub use linreg::LinearRegression;
pub use linreg_angle::LinRegAngle;
pub use linreg_channel::{LinRegChannel, LinRegChannelOutput};
pub use linreg_slope::LinRegSlope;
pub use ma_envelope::{MaEnvelope, MaEnvelopeOutput};
pub use macd::{MacdIndicator, MacdOutput};
pub use mama::{Mama, MamaOutput};
pub use market_facilitation_index::MarketFacilitationIndex;
pub use marubozu::Marubozu;
pub use mass_index::MassIndex;
pub use max_drawdown::MaxDrawdown;
pub use mcginley_dynamic::McGinleyDynamic;
pub use median_absolute_deviation::MedianAbsoluteDeviation;
pub use median_price::MedianPrice;
pub use mfi::Mfi;
pub use mom::Mom;
pub use morning_evening_star::MorningEveningStar;
pub use natr::Natr;
pub use nvi::Nvi;
pub use obv::Obv;
pub use omega_ratio::OmegaRatio;
pub use opening_range::{OpeningRange, OpeningRangeOutput};
pub use pain_index::PainIndex;
pub use pairwise_beta::PairwiseBeta;
pub use parkinson::ParkinsonVolatility;
pub use pearson_correlation::PearsonCorrelation;
pub use percent_b::PercentB;
pub use percentage_trailing_stop::PercentageTrailingStop;
pub use pgo::Pgo;
pub use piercing_dark_cloud::PiercingDarkCloud;
pub use pmo::Pmo;
pub use ppo::Ppo;
pub use profit_factor::ProfitFactor;
pub use psar::Psar;
pub use pvi::Pvi;
pub use r_squared::RSquared;
pub use recovery_factor::RecoveryFactor;
pub use renko_trailing_stop::RenkoTrailingStop;
pub use roc::Roc;
pub use rogers_satchell::RogersSatchellVolatility;
pub use roofing_filter::RoofingFilter;
pub use rsi::Rsi;
pub use rvi::Rvi;
pub use rvi_volatility::RviVolatility;
pub use rwi::{Rwi, RwiOutput};
pub use sharpe_ratio::SharpeRatio;
pub use shooting_star::ShootingStar;
pub use sine_wave::SineWave;
pub use skewness::Skewness;
pub use sma::Sma;
pub use smi::Smi;
pub use smma::Smma;
pub use sortino_ratio::SortinoRatio;
pub use spearman_correlation::SpearmanCorrelation;
pub use spinning_top::SpinningTop;
pub use standard_error::StandardError;
pub use standard_error_bands::{StandardErrorBands, StandardErrorBandsOutput};
pub use starc_bands::{StarcBands, StarcBandsOutput};
pub use stc::Stc;
pub use std_dev::StdDev;
pub use step_trailing_stop::StepTrailingStop;
pub use stoch_rsi::StochRsi;
pub use stochastic::{Stochastic, StochasticOutput};
pub use super_smoother::SuperSmoother;
pub use super_trend::{SuperTrend, SuperTrendOutput};
pub use t3::T3;
pub use td_combo::TdCombo;
pub use td_countdown::TdCountdown;
pub use td_demarker::TdDeMarker;
pub use td_differential::TdDifferential;
pub use td_lines::{TdLines, TdLinesOutput};
pub use td_open::TdOpen;
pub use td_pressure::TdPressure;
pub use td_range_projection::{TdRangeProjection, TdRangeProjectionOutput};
pub use td_rei::TdRei;
pub use td_risk_level::{TdRiskLevel, TdRiskLevelOutput};
pub use td_sequential::{TdSequential, TdSequentialOutput};
pub use td_setup::TdSetup;
pub use tema::Tema;
pub use three_inside::ThreeInside;
pub use three_outside::ThreeOutside;
pub use three_soldiers_or_crows::ThreeSoldiersOrCrows;
pub use tii::Tii;
pub use treynor_ratio::TreynorRatio;
pub use trima::Trima;
pub use trix::Trix;
pub use true_range::TrueRange;
pub use tsi::Tsi;
pub use tsv::Tsv;
pub use ttm_squeeze::{TtmSqueeze, TtmSqueezeOutput};
pub use tweezer::Tweezer;
pub use typical_price::TypicalPrice;
pub use ulcer_index::UlcerIndex;
pub use ultimate_oscillator::UltimateOscillator;
pub use value_area::{ValueArea, ValueAreaOutput};
pub use value_at_risk::ValueAtRisk;
pub use variance::Variance;
pub use vertical_horizontal_filter::VerticalHorizontalFilter;
pub use vidya::Vidya;
pub use volty_stop::VoltyStop;
pub use volume_oscillator::VolumeOscillator;
pub use vortex::{Vortex, VortexOutput};
pub use vpt::VolumePriceTrend;
pub use vwap::{RollingVwap, Vwap};
pub use vwap_stddev_bands::{VwapStdDevBands, VwapStdDevBandsOutput};
pub use vwma::Vwma;
pub use vzo::Vzo;
pub use wave_trend::{WaveTrend, WaveTrendOutput};
pub use weighted_close::WeightedClose;
pub use williams_fractals::{WilliamsFractals, WilliamsFractalsOutput};
pub use williams_r::WilliamsR;
pub use wma::Wma;
pub use woodie_pivots::{WoodiePivots, WoodiePivotsOutput};
pub use yang_zhang::YangZhangVolatility;
pub use yoyo_exit::YoyoExit;
pub use z_score::ZScore;
pub use zero_lag_macd::{ZeroLagMacd, ZeroLagMacdOutput};
pub use zig_zag::{ZigZag, ZigZagOutput};
pub use zlema::Zlema;

/// Family classification of every built-in indicator. The (family,
/// indicators) list is the single source of truth used by `family_tests`
/// below; README and Wiki taxonomy tables should be kept in sync with it.
///
/// Each indicator appears in exactly one family. Names are the public
/// struct identifiers re-exported from this module (and the crate root).
pub const FAMILIES: &[(&str, &[&str])] = &[
    (
        "Moving Averages",
        &[
            "Sma",
            "Ema",
            "Wma",
            "Dema",
            "Tema",
            "Hma",
            "Kama",
            "Smma",
            "Trima",
            "Zlema",
            "T3",
            "Vwma",
            "Alma",
            "McGinleyDynamic",
            "Frama",
            "Vidya",
            "Jma",
            "Alligator",
            "Evwma",
        ],
    ),
    (
        "Momentum Oscillators",
        &[
            "Rsi",
            "Stochastic",
            "Cci",
            "Roc",
            "WilliamsR",
            "Mfi",
            "AwesomeOscillator",
            "Mom",
            "Cmo",
            "Tsi",
            "Pmo",
            "StochRsi",
            "UltimateOscillator",
            "Rvi",
            "Pgo",
            "Kst",
            "Smi",
            "LaguerreRsi",
            "ConnorsRsi",
            "Inertia",
        ],
    ),
    (
        "Trend & Directional",
        &[
            "MacdIndicator",
            "Adx",
            "Adxr",
            "Aroon",
            "Trix",
            "AroonOscillator",
            "Vortex",
            "Rwi",
            "Tii",
            "WaveTrend",
            "MassIndex",
            "ChoppinessIndex",
            "VerticalHorizontalFilter",
        ],
    ),
    (
        "Price Oscillators",
        &[
            "Ppo",
            "Dpo",
            "Coppock",
            "AcceleratorOscillator",
            "BalanceOfPower",
            "Apo",
            "AwesomeOscillatorHistogram",
            "Cfo",
            "ZeroLagMacd",
            "ElderImpulse",
            "Stc",
        ],
    ),
    (
        "Volatility & Bands",
        &[
            "Atr",
            "BollingerBands",
            "Keltner",
            "Donchian",
            "Natr",
            "StdDev",
            "UlcerIndex",
            "HistoricalVolatility",
            "BollingerBandwidth",
            "PercentB",
            "TrueRange",
            "ChaikinVolatility",
            "RviVolatility",
            "ParkinsonVolatility",
            "GarmanKlassVolatility",
            "RogersSatchellVolatility",
            "YangZhangVolatility",
        ],
    ),
    (
        "Bands & Channels",
        &[
            "MaEnvelope",
            "AccelerationBands",
            "StarcBands",
            "AtrBands",
            "HurstChannel",
            "LinRegChannel",
            "StandardErrorBands",
            "DoubleBollinger",
            "TtmSqueeze",
            "FractalChaosBands",
            "VwapStdDevBands",
        ],
    ),
    (
        "Trailing Stops",
        &[
            "Psar",
            "SuperTrend",
            "ChandelierExit",
            "ChandeKrollStop",
            "AtrTrailingStop",
            "HiLoActivator",
            "VoltyStop",
            "YoyoExit",
            "DonchianStop",
            "PercentageTrailingStop",
            "StepTrailingStop",
            "RenkoTrailingStop",
        ],
    ),
    (
        "Volume",
        &[
            "Obv",
            "Vwap",
            "RollingVwap",
            "Adl",
            "VolumePriceTrend",
            "ChaikinMoneyFlow",
            "ChaikinOscillator",
            "ForceIndex",
            "EaseOfMovement",
            "Kvo",
            "VolumeOscillator",
            "Nvi",
            "Pvi",
            "AdOscillator",
            "AnchoredVwap",
            "DemandIndex",
            "Tsv",
            "Vzo",
            "MarketFacilitationIndex",
        ],
    ),
    (
        "Price Statistics",
        &[
            "TypicalPrice",
            "MedianPrice",
            "WeightedClose",
            "LinearRegression",
            "LinRegSlope",
            "ZScore",
            "LinRegAngle",
            "Variance",
            "CoefficientOfVariation",
            "Skewness",
            "Kurtosis",
            "StandardError",
            "DetrendedStdDev",
            "RSquared",
            "MedianAbsoluteDeviation",
            "Autocorrelation",
            "HurstExponent",
            "PearsonCorrelation",
            "Beta",
            "SpearmanCorrelation",
        ],
    ),
    (
        "Ehlers / Cycle (DSP)",
        &[
            "Mama",
            "Fama",
            "FisherTransform",
            "InverseFisherTransform",
            "SuperSmoother",
            "HilbertDominantCycle",
            "SineWave",
            "Decycler",
            "DecyclerOscillator",
            "RoofingFilter",
            "CenterOfGravity",
            "CyberneticCycle",
            "AdaptiveCycle",
            "EmpiricalModeDecomposition",
            "EhlersStochastic",
            "InstantaneousTrendline",
        ],
    ),
    (
        "Pivots & S/R",
        &[
            "ClassicPivots",
            "FibonacciPivots",
            "Camarilla",
            "WoodiePivots",
            "DemarkPivots",
            "WilliamsFractals",
            "ZigZag",
        ],
    ),
    (
        "DeMark",
        &[
            "TdSetup",
            "TdSequential",
            "TdDeMarker",
            "TdRei",
            "TdPressure",
            "TdCombo",
            "TdCountdown",
            "TdLines",
            "TdRangeProjection",
            "TdDifferential",
            "TdOpen",
            "TdRiskLevel",
        ],
    ),
    ("Ichimoku & Charts", &["Ichimoku", "HeikinAshi"]),
    (
        "Candlestick Patterns",
        &[
            "Doji",
            "Hammer",
            "InvertedHammer",
            "HangingMan",
            "ShootingStar",
            "Engulfing",
            "Harami",
            "MorningEveningStar",
            "ThreeSoldiersOrCrows",
            "PiercingDarkCloud",
            "Marubozu",
            "Tweezer",
            "SpinningTop",
            "ThreeInside",
            "ThreeOutside",
        ],
    ),
    (
        "Market Profile",
        &["ValueArea", "InitialBalance", "OpeningRange"],
    ),
    (
        "Risk / Performance",
        &[
            "SharpeRatio",
            "SortinoRatio",
            "CalmarRatio",
            "OmegaRatio",
            "MaxDrawdown",
            "AverageDrawdown",
            "DrawdownDuration",
            "PainIndex",
            "ValueAtRisk",
            "ConditionalValueAtRisk",
            "ProfitFactor",
            "GainLossRatio",
            "RecoveryFactor",
            "KellyCriterion",
            "TreynorRatio",
            "InformationRatio",
            "Alpha",
        ],
    ),
];

#[cfg(test)]
mod family_tests {
    use super::FAMILIES;

    #[test]
    fn no_duplicates_across_families() {
        let mut names: Vec<&str> = FAMILIES
            .iter()
            .flat_map(|(_, ns)| ns.iter().copied())
            .collect();
        let len_before = names.len();
        names.sort_unstable();
        names.dedup();
        assert_eq!(
            names.len(),
            len_before,
            "duplicate indicator across families"
        );
    }

    #[test]
    fn total_count_matches_expected() {
        // Bump together with new indicators. Drift between this number and
        // the actual indicator count is the early-warning signal that an
        // indicator was added without being assigned a family.
        let total: usize = FAMILIES.iter().map(|(_, ns)| ns.len()).sum();
        assert_eq!(total, 214, "FAMILIES total drifted from indicator count");
    }
}
