//! Rust example: download real BTCUSDT spot candles from the Binance REST API
//! and write them as CSV datasets under `examples/data/`.
//!
//! These are the datasets the indicator benchmarks (`crates/wickra/benches/
//! indicators.rs`) and the `example_data` integration test run against. They
//! live at the workspace `examples/data/` directory. Re-run this example to
//! refresh them with the latest market history:
//!
//! ```text
//! cargo run -p wickra-examples --bin fetch_btcusdt
//! ```
//!
//! HTTPS is handled by shelling out to the system `curl` — shipped with
//! Windows 10+, macOS, and every Linux distribution — so this example adds no
//! HTTP or TLS dependency to the crate (`serde_json` is the only extra, and
//! only as a dev-dependency). Each Binance klines response is capped at 1000
//! rows, so larger datasets are paginated backwards through `endTime`. Only
//! fully closed candles are kept; the still-forming candle of the current
//! bucket is dropped so the datasets stay reproducible.

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde_json::Value as Json;
use wickra::Candle;

/// Binance Spot REST endpoint for historical klines.
const KLINES_URL: &str = "https://api.binance.com/api/v3/klines";
/// Trading pair to download.
const SYMBOL: &str = "BTCUSDT";
/// Binance caps a single klines response at 1000 rows.
const PAGE_LIMIT: usize = 1000;
/// Courtesy pause between paginated requests to stay well under the rate limit.
const REQUEST_PAUSE: Duration = Duration::from_millis(200);

/// One dataset to produce: the Binance interval code, the output file name,
/// and how many of the most-recent *closed* candles to collect.
struct Dataset {
    interval: &'static str,
    file: &'static str,
    target: usize,
}

/// The seven datasets, one per timeframe. `12h`/`1d`/`1M` simply collect all
/// the history Binance offers — it is shorter than their `target`.
///
/// The monthly file is named `btcusdt-1month.csv`, not `btcusdt-1M.csv`: on
/// case-insensitive filesystems (Windows, default macOS) the latter would
/// collide with `btcusdt-1m.csv` and one dataset would silently overwrite the
/// other.
const DATASETS: &[Dataset] = &[
    Dataset {
        interval: "1m",
        file: "btcusdt-1m.csv",
        target: 50_000,
    },
    Dataset {
        interval: "5m",
        file: "btcusdt-5m.csv",
        target: 10_000,
    },
    Dataset {
        interval: "15m",
        file: "btcusdt-15m.csv",
        target: 10_000,
    },
    Dataset {
        interval: "1h",
        file: "btcusdt-1h.csv",
        target: 10_000,
    },
    Dataset {
        interval: "12h",
        file: "btcusdt-12h.csv",
        target: 5_000,
    },
    Dataset {
        interval: "1d",
        file: "btcusdt-1d.csv",
        target: 5_000,
    },
    Dataset {
        interval: "1M",
        file: "btcusdt-1month.csv",
        target: 5_000,
    },
];

fn main() -> Result<(), Box<dyn Error>> {
    // A single reference instant for the whole run: any candle whose bucket has
    // not closed by this point is treated as in-progress and dropped.
    let now_ms = i64::try_from(SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis())
        .map_err(|_| "system clock is beyond the i64 millisecond range")?;

    let data_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("data");
    std::fs::create_dir_all(&data_dir)?;

    println!(
        "Fetching {SYMBOL} klines from Binance into {}",
        data_dir.display()
    );
    for ds in DATASETS {
        let candles = collect(ds.interval, ds.target, now_ms)?;
        if candles.is_empty() {
            return Err(format!(
                "Binance returned no closed candles for interval {}",
                ds.interval
            )
            .into());
        }
        let path = data_dir.join(ds.file);
        write_csv(&path, &candles)?;
        println!(
            "  {:>3}  {:>6} candles  ->  examples/data/{}",
            ds.interval,
            candles.len(),
            ds.file
        );
    }
    println!("Done — {} datasets written.", DATASETS.len());
    Ok(())
}

/// Paginate the Binance REST API backwards until `target` closed candles have
/// been collected (or the exchange runs out of history), then return the most
/// recent `target` of them in ascending time order.
fn collect(interval: &str, target: usize, now_ms: i64) -> Result<Vec<Candle>, Box<dyn Error>> {
    // Keyed by open time: the map sorts chronologically and dedupes the small
    // overlap that can occur between adjacent pages.
    let mut by_open: BTreeMap<i64, Candle> = BTreeMap::new();
    let mut end_time: Option<i64> = None;
    let mut pages = 0usize;

    loop {
        let page = fetch_page(interval, end_time)?;
        pages += 1;
        if page.is_empty() {
            break;
        }

        let mut oldest_open = i64::MAX;
        for raw in &page {
            let Some((open_time, close_time, candle)) = parse_kline(raw) else {
                continue;
            };
            oldest_open = oldest_open.min(open_time);
            // Keep only fully closed candles — drop the in-progress bucket.
            if close_time < now_ms {
                by_open.insert(open_time, candle);
            }
        }
        eprint!(
            "\r  {interval}: collected {} candles over {pages} page(s)…",
            by_open.len()
        );

        // Stop once enough is collected or Binance has no older history left.
        if by_open.len() >= target || page.len() < PAGE_LIMIT {
            break;
        }
        if oldest_open == i64::MAX {
            return Err(format!("Binance page for {interval} held no parseable klines").into());
        }
        // Step the window strictly before the oldest open time seen so far.
        end_time = Some(oldest_open - 1);
        std::thread::sleep(REQUEST_PAUSE);
    }
    eprintln!();

    let mut candles: Vec<Candle> = by_open.into_values().collect();
    if candles.len() > target {
        // Trim the oldest surplus, keeping the most recent `target` candles.
        candles.drain(..candles.len() - target);
    }
    Ok(candles)
}

/// Fetch one page of klines. `end_time`, when set, caps the open time so the
/// caller can walk backwards through history.
fn fetch_page(interval: &str, end_time: Option<i64>) -> Result<Vec<Json>, Box<dyn Error>> {
    let mut url = format!("{KLINES_URL}?symbol={SYMBOL}&interval={interval}&limit={PAGE_LIMIT}");
    if let Some(end) = end_time {
        write!(url, "&endTime={end}").expect("writing to a String never fails");
    }

    let output = Command::new("curl")
        .args([
            "--silent",
            "--show-error",
            "--fail",
            "--max-time",
            "30",
            &url,
        ])
        .output()
        .map_err(|e| format!("could not run `curl` (install it / put it on PATH): {e}"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("curl failed for {url}: {}", stderr.trim()).into());
    }

    let body = String::from_utf8(output.stdout)?;
    match serde_json::from_str(&body)? {
        Json::Array(rows) => Ok(rows),
        other => Err(format!("expected a JSON array of klines from Binance, got: {other}").into()),
    }
}

/// Parse one raw Binance kline array into `(open_time, close_time, Candle)`.
///
/// A Binance kline is `[openTime, "open", "high", "low", "close", "volume",
/// closeTime, …]`; the OHLCV fields arrive as decimal strings. Returns `None`
/// for any row that is malformed or fails `Candle::new`'s OHLC validation.
fn parse_kline(raw: &Json) -> Option<(i64, i64, Candle)> {
    let arr = raw.as_array()?;
    if arr.len() < 7 {
        return None;
    }
    let open_time = arr[0].as_i64()?;
    let close_time = arr[6].as_i64()?;
    let field = |i: usize| -> Option<f64> { arr[i].as_str()?.parse::<f64>().ok() };
    let candle = Candle::new(
        field(1)?,
        field(2)?,
        field(3)?,
        field(4)?,
        field(5)?,
        open_time,
    )
    .ok()?;
    Some((open_time, close_time, candle))
}

/// Write candles as a standard `timestamp,open,high,low,close,volume` CSV that
/// [`wickra_data::csv::CandleReader`] can read back.
fn write_csv(path: &Path, candles: &[Candle]) -> Result<(), Box<dyn Error>> {
    let mut out = String::with_capacity(candles.len() * 80 + 32);
    out.push_str("timestamp,open,high,low,close,volume\n");
    for c in candles {
        writeln!(
            out,
            "{},{},{},{},{},{}",
            c.timestamp, c.open, c.high, c.low, c.close, c.volume
        )?;
    }
    std::fs::write(path, out)?;
    Ok(())
}
