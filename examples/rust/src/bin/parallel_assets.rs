//! Parallel multi-asset indicator computation via rayon.
//!
//! Builds a synthetic `(assets, bars)` panel, runs a serial baseline, then
//! runs the same computation with [`BatchExt::batch_parallel`] (which is
//! gated behind wickra's default `parallel` feature) and reports the
//! speedup. The Rust counterpart of `examples/python/parallel_assets.py`.
//!
//! Run with:
//! ```text
//! cargo run --release -p wickra-examples --bin parallel_assets -- \
//!     --assets 200 --bars 5000
//! ```

use std::env;
use std::time::Instant;

use wickra::{BatchExt, Rsi, Sma};

#[derive(Debug, Clone)]
enum Which {
    Sma,
    Rsi,
}

impl Which {
    fn parse(s: &str) -> Result<Self, Box<dyn std::error::Error>> {
        match s {
            "sma" => Ok(Self::Sma),
            "rsi" => Ok(Self::Rsi),
            other => Err(format!("--indicator: expected 'sma' or 'rsi', got {other}").into()),
        }
    }
}

struct Args {
    assets: usize,
    bars: usize,
    indicator: Which,
}

fn parse_args() -> Result<Args, Box<dyn std::error::Error>> {
    let mut assets = 200_usize;
    let mut bars = 5_000_usize;
    let mut indicator = Which::Sma;
    let mut it = env::args().skip(1);
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "--assets" => {
                assets = it.next().ok_or("--assets needs a value")?.parse()?;
            }
            "--bars" => {
                bars = it.next().ok_or("--bars needs a value")?.parse()?;
            }
            "--indicator" => {
                indicator = Which::parse(&it.next().ok_or("--indicator needs a value")?)?;
            }
            other => return Err(format!("unexpected argument: {other}").into()),
        }
    }
    if assets == 0 || bars == 0 {
        return Err("--assets and --bars must be positive".into());
    }
    Ok(Args {
        assets,
        bars,
        indicator,
    })
}

/// Deterministic synthetic `(assets, bars)` panel. Each asset uses an
/// independent LCG seed so the series are uncorrelated but reproducible.
fn synthesize_panel(n_assets: usize, n_bars: usize) -> Vec<Vec<f64>> {
    let mut series = Vec::with_capacity(n_assets);
    for a in 0..n_assets {
        let mut s = Vec::with_capacity(n_bars);
        let mut price = 100.0_f64;
        let mut state: u32 = 1_234_567_u32
            .wrapping_add(a as u32)
            .wrapping_mul(2_654_435_761);
        for _ in 0..n_bars {
            state = state.wrapping_mul(1_103_515_245).wrapping_add(12_345) & 0x7FFF_FFFF;
            let r = f64::from(state) / f64::from(0x7FFF_FFFF_u32);
            price += (r - 0.5) * 0.4;
            s.push(price);
        }
        series.push(s);
    }
    series
}

fn run_serial(panel: &[Vec<f64>], indicator: &Which) -> Vec<Vec<Option<f64>>> {
    panel
        .iter()
        .map(|prices| match indicator {
            Which::Sma => Sma::new(14).expect("SMA(14)").batch(prices),
            Which::Rsi => Rsi::new(14).expect("RSI(14)").batch(prices),
        })
        .collect()
}

fn run_parallel(panel: &[Vec<f64>], indicator: &Which) -> Vec<Vec<Option<f64>>> {
    match indicator {
        Which::Sma => Sma::batch_parallel(panel, || Sma::new(14).expect("SMA(14)")),
        Which::Rsi => Rsi::batch_parallel(panel, || Rsi::new(14).expect("RSI(14)")),
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = parse_args()?;
    println!("Generating {}×{} synthetic panel…", args.assets, args.bars);
    let panel = synthesize_panel(args.assets, args.bars);

    let t0 = Instant::now();
    let serial = run_serial(&panel, &args.indicator);
    let t_serial = t0.elapsed();
    println!(
        "Serial:   {:>8.3} s  ({} assets, indicator={:?})",
        t_serial.as_secs_f64(),
        args.assets,
        args.indicator
    );

    let t0 = Instant::now();
    let parallel = run_parallel(&panel, &args.indicator);
    let t_parallel = t0.elapsed();
    let speedup = t_serial.as_secs_f64() / t_parallel.as_secs_f64().max(1e-9);
    println!(
        "Parallel: {:>8.3} s  (rayon, speedup ~{:.2}x)",
        t_parallel.as_secs_f64(),
        speedup
    );

    assert_eq!(serial.len(), parallel.len(), "asset count mismatch");
    for (i, (s, p)) in serial.iter().zip(parallel.iter()).enumerate() {
        if s != p {
            return Err(format!("asset {i}: serial and parallel results disagree").into());
        }
    }
    println!("Parallel results match serial results — OK.");
    Ok(())
}
