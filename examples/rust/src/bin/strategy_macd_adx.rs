//! Strategy example: MACD crossover with ADX trend-strength filter.
//!
//! Long-only trend follower. Entries fire on a MACD-line-crosses-above-
//! signal-line event while ADX(14) > 20 (i.e. a market with at least mild
//! directional strength). Exits on the opposite MACD crossover regardless
//! of ADX. 0.1% fees per trade.
//!
//! The ADX filter is the whole point of the strategy: pure MACD on
//! sideways markets chops in and out; gating entries on directional-
//! strength cuts the worst losing streak.
//!
//! Educational example. **Not** a live trading recommendation.
//!
//! Build with:
//! ```text
//! cargo run --release -p wickra-examples --bin strategy_macd_adx
//! ```
//!
//! Uses the checked-in `examples/data/btcusdt-1h.csv` dataset.

use wickra::{Adx, Indicator, MacdIndicator};
use wickra_data::csv::CandleReader;

const FEE: f64 = 0.001;
const ADX_FLOOR: f64 = 20.0;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../data/btcusdt-1h.csv");
    let mut reader = CandleReader::open(path)?;
    let candles = reader.read_all()?;
    if candles.is_empty() {
        return Err("CSV is empty".into());
    }

    let mut macd = MacdIndicator::classic();
    let mut adx = Adx::new(14)?;

    let mut in_position = false;
    let mut entry_price = 0.0_f64;
    let mut closed_trades: Vec<f64> = Vec::new();
    let mut equity = 1.0_f64;
    let mut equity_curve: Vec<f64> = Vec::with_capacity(candles.len());

    // Track the previous histogram sign to detect MACD-line crossovers.
    let mut prev_hist_sign: Option<bool> = None;

    for candle in &candles {
        let macd_out = macd.update(candle.close);
        let adx_out = adx.update(*candle);
        let price = candle.close;

        let mtm_equity = if in_position {
            equity * (price / entry_price)
        } else {
            equity
        };
        equity_curve.push(mtm_equity);

        let Some(m) = macd_out else { continue };
        let Some(a) = adx_out else { continue };

        let hist_sign = m.histogram > 0.0;
        let cross_up = prev_hist_sign == Some(false) && hist_sign;
        let cross_down = prev_hist_sign == Some(true) && !hist_sign;
        prev_hist_sign = Some(hist_sign);

        if !in_position && cross_up && a.adx > ADX_FLOOR {
            // Enter long: directional regime with positive momentum.
            entry_price = price;
            equity *= 1.0 - FEE;
            in_position = true;
        } else if in_position && cross_down {
            // Exit on opposite cross — ADX gating only the entries
            // keeps us from being trapped in a long trade as a trend dies.
            let trade_ret = price / entry_price - 1.0;
            closed_trades.push(trade_ret);
            equity *= (1.0 + trade_ret) * (1.0 - FEE);
            in_position = false;
        }
    }

    if in_position {
        let last_price = candles.last().expect("non-empty above").close;
        let trade_ret = last_price / entry_price - 1.0;
        closed_trades.push(trade_ret);
        equity *= (1.0 + trade_ret) * (1.0 - FEE);
    }

    print_summary(
        "MACD + ADX Trend Filter (1h, BTCUSDT)",
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
