//! Generate the language-neutral golden fixtures consumed by the per-binding
//! parity runners (`bindings/{csharp,go,java,r}`).
//!
//! Run from the repo root: `cargo run -p wickra-examples --bin gen_golden`.
//! It writes a deterministic OHLCV input series plus the reference outputs of a
//! curated set of indicators spanning the FFI archetypes (scalar, candle,
//! scalar multi-output, candle multi-output, pairwise), computed by the Rust
//! core. Each binding runner replays the same input through its own FFI and
//! checks it matches these values — catching wiring bugs (swapped params,
//! wrong multi-output index) that the math-only core tests cannot see.
//!
//! `nan` marks a warmup slot where the indicator returned `None`.

use std::fmt::Write as _;
use std::fs;
use std::path::Path;

use wickra::{Adx, Atr, Beta, Candle, Ema, Indicator, MacdIndicator, Rsi, Sma};

const N: usize = 80;

/// Deterministic OHLCV bar `i`: a varied, non-degenerate path so every
/// indicator gets real movement (no constant returns, no zero ranges).
fn bar(i: usize) -> (f64, f64, f64, f64, f64) {
    let t = i as f64;
    let close = 100.0 + 10.0 * (t * 0.3).sin() + 0.5 * t;
    let open = close - (t * 0.5).cos();
    let span = 1.0 + 0.5 * (t * 0.7).sin().abs();
    let high = close.max(open) + span;
    let low = close.min(open) - span;
    let volume = 1000.0 + 100.0 * (t * 0.2).sin().abs();
    (open, high, low, close, volume)
}

fn cell(v: Option<f64>) -> String {
    match v {
        Some(x) => format!("{x}"),
        None => "nan".to_owned(),
    }
}

fn write_csv(dir: &Path, name: &str, header: &str, rows: &[String]) {
    let mut out = String::new();
    let _ = writeln!(out, "{header}");
    for r in rows {
        let _ = writeln!(out, "{r}");
    }
    let path = dir.join(format!("{name}.csv"));
    fs::write(&path, out).expect("write fixture");
    println!("wrote {}", path.display());
}

fn main() {
    let dir = Path::new("testdata/golden");
    fs::create_dir_all(dir).expect("create testdata/golden");

    let candles: Vec<Candle> = (0..N)
        .map(|i| {
            let (o, h, l, c, v) = bar(i);
            Candle::new(o, h, l, c, v, i as i64).expect("valid candle")
        })
        .collect();
    let closes: Vec<f64> = candles.iter().map(|c| c.close).collect();

    // input.csv — the single shared series every runner reads.
    let mut input = vec![String::from("open,high,low,close,volume")];
    for c in &candles {
        input.push(format!(
            "{},{},{},{},{}",
            c.open, c.high, c.low, c.close, c.volume
        ));
    }
    let input_header = input.remove(0);
    write_csv(dir, "input", &input_header, &input);

    // scalar (close-driven), single output. Each is a distinct type, so write
    // them out separately rather than from a heterogeneous collection.
    {
        let mut sma = Sma::new(14).unwrap();
        let rows: Vec<String> = closes.iter().map(|&c| cell(sma.update(c))).collect();
        write_csv(dir, "sma", "sma", &rows);
    }
    {
        let mut ema = Ema::new(14).unwrap();
        let rows: Vec<String> = closes.iter().map(|&c| cell(ema.update(c))).collect();
        write_csv(dir, "ema", "ema", &rows);
    }
    {
        let mut rsi = Rsi::new(14).unwrap();
        let rows: Vec<String> = closes.iter().map(|&c| cell(rsi.update(c))).collect();
        write_csv(dir, "rsi", "rsi", &rows);
    }

    // candle, single output.
    {
        let mut atr = Atr::new(14).unwrap();
        let rows: Vec<String> = candles.iter().map(|&c| cell(atr.update(c))).collect();
        write_csv(dir, "atr", "atr", &rows);
    }

    // scalar multi-output: MACD(12,26,9).
    {
        let mut macd = MacdIndicator::new(12, 26, 9).unwrap();
        let rows: Vec<String> = closes
            .iter()
            .map(|&c| match macd.update(c) {
                Some(o) => format!("{},{},{}", o.macd, o.signal, o.histogram),
                None => "nan,nan,nan".to_owned(),
            })
            .collect();
        write_csv(dir, "macd", "macd,signal,histogram", &rows);
    }

    // candle multi-output: ADX(14).
    {
        let mut adx = Adx::new(14).unwrap();
        let rows: Vec<String> = candles
            .iter()
            .map(|&c| match adx.update(c) {
                Some(o) => format!("{},{},{}", o.plus_di, o.minus_di, o.adx),
                None => "nan,nan,nan".to_owned(),
            })
            .collect();
        write_csv(dir, "adx", "plus_di,minus_di,adx", &rows);
    }

    // pairwise: Beta(20) over (close, open).
    {
        let mut beta = Beta::new(20).unwrap();
        let rows: Vec<String> = candles
            .iter()
            .map(|&c| cell(beta.update((c.close, c.open))))
            .collect();
        write_csv(dir, "beta", "beta", &rows);
    }

    println!("golden fixtures written to {}", dir.display());
}
