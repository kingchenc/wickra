//! Strategy example: Bollinger-Squeeze breakout with ATR-based stop.
//!
//! Enters long when the Bollinger Bandwidth has just printed a fresh
//! 6-month low (the *squeeze*) and price closes above the upper band
//! (the *release*). Exits when price closes below the entry minus 2 *
//! ATR(14), or when the upper band starts trailing below the entry
//! price (the squeeze pattern has played out). 0.1% fees per trade.
//!
//! Educational example. **Not** a live trading recommendation.
//!
//! Build with:
//! ```text
//! cargo run --release -p wickra-examples --bin strategy_bollinger_squeeze
//! ```
//!
//! Uses the checked-in `examples/data/btcusdt-1d.csv` dataset because
//! daily bars give an interpretable "6-month low" lookback (≈180 bars).

use std::collections::VecDeque;

use wickra::{Atr, BollingerBands, Indicator};
use wickra_data::csv::CandleReader;

const FEE: f64 = 0.001;
const BB_PERIOD: usize = 20;
const BB_K: f64 = 2.0;
const ATR_PERIOD: usize = 14;
const ATR_STOP_MULT: f64 = 2.0;
const SQUEEZE_LOOKBACK: usize = 180; // ≈ 6 months of daily bars

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../data/btcusdt-1d.csv");
    let mut reader = CandleReader::open(path)?;
    let candles = reader.read_all()?;
    if candles.len() < SQUEEZE_LOOKBACK + BB_PERIOD {
        return Err(format!(
            "dataset has only {} bars; need at least {}",
            candles.len(),
            SQUEEZE_LOOKBACK + BB_PERIOD
        )
        .into());
    }

    let mut bb = BollingerBands::new(BB_PERIOD, BB_K)?;
    let mut atr = Atr::new(ATR_PERIOD)?;

    let mut bw_window: VecDeque<f64> = VecDeque::with_capacity(SQUEEZE_LOOKBACK);

    let mut in_position = false;
    let mut entry_price = 0.0_f64;
    let mut stop_level = 0.0_f64;
    let mut closed_trades: Vec<f64> = Vec::new();
    let mut equity = 1.0_f64;
    let mut equity_curve: Vec<f64> = Vec::with_capacity(candles.len());

    for candle in &candles {
        let bb_out = bb.update(candle.close);
        let atr_out = atr.update(*candle);
        let price = candle.close;

        let mtm_equity = if in_position {
            equity * (price / entry_price)
        } else {
            equity
        };
        equity_curve.push(mtm_equity);

        let Some(b) = bb_out else { continue };
        let Some(a) = atr_out else { continue };

        // Bandwidth = (upper - lower) / middle; track its rolling minimum
        // over the squeeze lookback so we know what "tight" looks like
        // in this regime.
        let bandwidth = if b.middle.abs() > f64::EPSILON {
            (b.upper - b.lower) / b.middle
        } else {
            f64::NAN
        };
        if bandwidth.is_finite() {
            if bw_window.len() == SQUEEZE_LOOKBACK {
                bw_window.pop_front();
            }
            bw_window.push_back(bandwidth);
        }

        if bw_window.len() < SQUEEZE_LOOKBACK || !bandwidth.is_finite() {
            continue;
        }
        let min_bw = bw_window.iter().copied().fold(f64::INFINITY, f64::min);

        if in_position {
            // Exit: hit ATR-stop OR upper-band has rolled back under
            // the entry (squeeze is exhausted).
            let stop_hit = price < stop_level;
            let upper_collapse = b.upper < entry_price;
            if stop_hit || upper_collapse {
                let trade_ret = price / entry_price - 1.0;
                closed_trades.push(trade_ret);
                equity *= (1.0 + trade_ret) * (1.0 - FEE);
                in_position = false;
            }
        } else {
            // Entry trigger: current bandwidth is the new 6-month low AND
            // price has just punched above the upper band.
            let is_new_low = (bandwidth - min_bw).abs() < 1e-12;
            let breakout = price > b.upper;
            if is_new_low && breakout {
                entry_price = price;
                stop_level = price - ATR_STOP_MULT * a;
                equity *= 1.0 - FEE;
                in_position = true;
            }
        }
    }

    if in_position {
        let last_price = candles.last().expect("non-empty above").close;
        let trade_ret = last_price / entry_price - 1.0;
        closed_trades.push(trade_ret);
        equity *= (1.0 + trade_ret) * (1.0 - FEE);
    }

    print_summary(
        "Bollinger Squeeze Breakout (1d, BTCUSDT)",
        candles.first().unwrap().close,
        candles.last().unwrap().close,
        candles.len(),
        &closed_trades,
        equity,
        &equity_curve,
    );

    Ok(())
}

fn print_summary(
    name: &str,
    first_price: f64,
    last_price: f64,
    bars: usize,
    closed_trades: &[f64],
    final_equity: f64,
    equity_curve: &[f64],
) {
    let buy_hold = last_price / first_price;
    let strat_return = final_equity - 1.0;
    let bh_return = buy_hold - 1.0;

    let mut wins = 0usize;
    let mut losses = 0usize;
    let mut best = f64::NEG_INFINITY;
    let mut worst = f64::INFINITY;
    let mut sum_ret = 0.0_f64;
    let mut sum_sq = 0.0_f64;
    for &r in closed_trades {
        if r > 0.0 {
            wins += 1;
        } else if r < 0.0 {
            losses += 1;
        }
        best = best.max(r);
        worst = worst.min(r);
        sum_ret += r;
        sum_sq += r * r;
    }
    let n = closed_trades.len() as f64;
    let mean_ret = if n > 0.0 { sum_ret / n } else { 0.0 };
    let var_ret = if n > 1.0 {
        (sum_sq - n * mean_ret * mean_ret) / (n - 1.0)
    } else {
        0.0
    };
    let sharpe = if var_ret > 0.0 {
        mean_ret / var_ret.sqrt()
    } else {
        0.0
    };

    let mut peak = equity_curve.first().copied().unwrap_or(1.0);
    let mut max_dd = 0.0_f64;
    for &eq in equity_curve {
        peak = peak.max(eq);
        let dd = (peak - eq) / peak;
        if dd > max_dd {
            max_dd = dd;
        }
    }

    println!("=== {name} ===");
    println!("Bars:                  {bars}");
    println!(
        "Trades:                {} (W{wins} / L{losses})",
        closed_trades.len()
    );
    println!("Strategy return:       {:+.2}%", strat_return * 100.0);
    println!("Buy & Hold return:     {:+.2}%", bh_return * 100.0);
    println!(
        "Excess over BH:        {:+.2}%",
        (strat_return - bh_return) * 100.0
    );
    println!("Max drawdown:          {:.2}%", max_dd * 100.0);
    println!(
        "Per-trade Sharpe:      {sharpe:.2}  (mean {:+.4}, stddev {:.4})",
        mean_ret,
        var_ret.sqrt()
    );
    println!(
        "Best / worst trade:    {:+.2}% / {:+.2}%",
        best * 100.0,
        worst * 100.0
    );
    println!();
    println!(
        "NOTE: Educational example — fees, slippage, funding costs and tax effects \
         are simplified or omitted. Past performance is not indicative of future results."
    );
}
