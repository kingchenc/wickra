//! Strategy example: RSI mean-reversion on hourly BTCUSDT data.
//!
//! Goes long when RSI(14) crosses below 30 (oversold), exits when RSI
//! crosses above 70 (overbought). Position is binary (full-in / full-out),
//! fees are 0.1% per trade (Binance maker tier), no stop-loss.
//!
//! Educational example. **Not** a recommended trading strategy in real
//! markets — mean reversion on BTC has been historically losing over long
//! horizons. The point is to show how Wickra streaming indicators wire up
//! into a complete signal → fill → `PnL` → equity loop in a single file.
//!
//! Build with:
//! ```text
//! cargo run --release -p wickra-examples --bin strategy_rsi_mean_reversion
//! ```
//!
//! Uses the checked-in `examples/data/btcusdt-1h.csv` dataset.

use wickra::{Indicator, Rsi};
use wickra_data::csv::CandleReader;

const FEE: f64 = 0.001; // 0.1% per trade (Binance maker)
const RSI_PERIOD: usize = 14;
const OVERSOLD: f64 = 30.0;
const OVERBOUGHT: f64 = 70.0;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../data/btcusdt-1h.csv");
    let mut reader = CandleReader::open(path)?;
    let candles = reader.read_all()?;
    if candles.len() < RSI_PERIOD * 4 {
        return Err(format!("dataset too small: {}", candles.len()).into());
    }

    let mut rsi = Rsi::new(RSI_PERIOD)?;

    // Walk through bars, generate signals, track an equity curve.
    let mut in_position = false;
    let mut entry_price = 0.0_f64;
    let mut closed_trades: Vec<f64> = Vec::new(); // per-trade returns
    let mut equity = 1.0_f64;
    let mut equity_curve: Vec<f64> = Vec::with_capacity(candles.len());

    for candle in &candles {
        let rsi_val = rsi.update(candle.close);
        let price = candle.close;

        // Mark-to-market the open position so the equity curve moves
        // bar-by-bar even between trades.
        let mtm_equity = if in_position {
            equity * (price / entry_price)
        } else {
            equity
        };
        equity_curve.push(mtm_equity);

        let Some(r) = rsi_val else { continue };

        if !in_position && r < OVERSOLD {
            // Enter long. Pay entry fee out of equity.
            entry_price = price;
            equity *= 1.0 - FEE;
            in_position = true;
        } else if in_position && r > OVERBOUGHT {
            // Exit long. Realise trade PnL, pay exit fee.
            let trade_ret = price / entry_price - 1.0;
            closed_trades.push(trade_ret);
            equity *= (1.0 + trade_ret) * (1.0 - FEE);
            in_position = false;
        }
    }

    // If we ended a still open trade, mark it closed at the last bar so
    // metrics don't omit a half-trade.
    if in_position {
        let last_price = candles.last().expect("non-empty by guard above").close;
        let trade_ret = last_price / entry_price - 1.0;
        closed_trades.push(trade_ret);
        equity *= (1.0 + trade_ret) * (1.0 - FEE);
    }

    print_summary(
        "RSI Mean-Reversion (1h, BTCUSDT)",
        candles.first().unwrap().close,
        candles.last().unwrap().close,
        candles.len(),
        &closed_trades,
        equity,
        &equity_curve,
    );

    Ok(())
}

/// Print a one-screen summary of an equity-curve plus per-trade list.
/// Kept inline (not factored out) so each strategy example stays a
/// single-file read.
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

    // Max-drawdown on the equity curve.
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
