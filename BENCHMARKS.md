# Benchmarks

Read these as **relative** speedups on identical input — absolute µs depend on
CPU, memory clock and OS scheduler, not a universal contract. **Streaming is the
headline**: it is where Wickra's design pays off and where the gap is measured in
orders of magnitude, not percent. The batch numbers come second and are shown
honestly — the leanest crates edge Wickra out on the simple recurrences, and that
is a deliberate trade for warmup/NaN semantics, not a ceiling.

- **Reproduced on:** Windows 11 Pro 26200, AMD Ryzen 9 9950X, 64 GB DDR5,
  Rust 1.92 (release: `lto = "fat"`, `codegen-units = 1`), Python 3.12.
- **Reproduce yourself:**
  - Rust core vs Rust crates: `cargo bench -p wickra-bench`
  - Python vs Python libs: `pip install -e bindings/python[bench]` then
    `python -m benchmarks.compare_libraries` (auto-detects installed peers).

## 1. Streaming — the structural win

Live trading feeds one tick at a time. Wickra updates every indicator in **O(1)**;
batch-only libraries (TA-Lib, tulipy, finta, pandas-ta) have no incremental API
and must recompute the whole history on every tick. Only `talipp` (Python) and
`ta-rs` / `yata` (Rust) carry real per-tick state. This is the gap the library
was built to expose.

**Python — per-tick latency** (seed 5 000 bars, then feed ticks one at a time):

| Indicator        | **★&nbsp;Wickra** | talipp           | TA-Lib (recompute)    |
|------------------|------------------:|------------------|-----------------------|
| SMA(20)          | **0.089 µs ★**    | 0.96 µs (11×)    | 422 µs (4 700×)       |
| EMA(20)          | **0.111 µs ★**    | 1.19 µs (11×)    | 430 µs (3 900×)       |
| RSI(14)          | **0.061 µs ★**    | 0.95 µs (16×)    | 298 µs (4 900×)       |
| MACD(12, 26, 9)  | **0.079 µs ★**    | 3.30 µs (42×)    | 327 µs (4 100×)       |
| Bollinger(20, 2) | **0.089 µs ★**    | 4.97 µs (56×)    | 296 µs (3 300×)       |

Against the only other incremental Python peer Wickra is **11–56× faster**;
against the recompute-on-every-tick libraries it is **2 800–19 000× faster**
(`finta` RSI hits 19 000×). tulipy / pandas-ta land in the same recompute band
as TA-Lib.

**Rust — per-tick latency** (whole 50 000-bar series, lower = faster):

| Indicator        | **★&nbsp;Wickra** | kand | ta-rs | yata |
|------------------|------------------:|-----:|------:|-----:|
| SMA(20)          | 50                | 38   | 47    | 38   |
| EMA(20)          | 154               | 69   | 56    | 69   |
| RSI(14)          | 164               | 216  | 74    | —    |
| MACD(12, 26, 9)  | 275               | 143  | 66    | —    |
| Bollinger(20, 2) | **128 ★**         | 248  | 168   | —    |
| ATR(14)          | 152               | 166  | 61    | —    |

`ta-rs` hands back a bare `f64` from the first tick with no warmup and no
validation; it leads several rows by giving those guarantees up. Against `kand`,
Wickra wins streaming RSI, Bollinger and ATR. `yata` exposes only SMA/EMA as
raw-value methods, so its other rows are omitted rather than faked.

## 2. Batch — competitive, not the headline

Whole series in one call. Here hand-tuned C (`tulipy`, TA-Lib) and the leanest
Rust crate (`kand`) win the simple recurrences — Wickra trades a few µs per pass
for the `None`-warmup, NaN-safety and bit-exact `batch == streaming` guarantees
none of them keep. It still wins several rows outright and beats the rest of the
field everywhere.

**Python** (20 000-bar pass, µs/op, lower = faster):

| Indicator        | Wickra   | TA-Lib   | tulipy   | pandas-ta | finta    |
|------------------|---------:|---------:|---------:|----------:|---------:|
| SMA(20)          | 22.2     | **15.6** | 15.9     | 32.7      | 290.1    |
| EMA(20)          | 30.5     | **30.4** | 30.9     | 46.7      | 198.5    |
| RSI(14)          | 52.3     | 72.0     | **34.2** | 88.8      | 812.3    |
| MACD(12, 26, 9)  | 129.8    | 111.1    | **38.4** | 286.8     | 716.7    |
| Bollinger(20, 2) | 87.2     | 74.6     | **37.9** | 474.3     | 1255.5   |
| ATR(14)          | 74.7     | 87.3     | **35.5** | —         | 3496.4   |

Wickra beats pandas-ta and finta on every row and TA-Lib on RSI and ATR;
tulipy's SIMD C (and TA-Lib on SMA/EMA) lead the remaining rows.

**Rust** (50 000-bar pass, µs, lower = faster). Only Wickra and `kand` expose a
batch API; `ta-rs` and `yata` are streaming-only:

| Indicator        | **★&nbsp;Wickra** | kand   |
|------------------|------------------:|-------:|
| SMA(20)          | 53                | **41** |
| EMA(20)          | 111               | **71** |
| RSI(14)          | **221 ★**         | 259    |
| MACD(12, 26, 9)  | 533               | **327** |
| Bollinger(20, 2) | **404 ★**         | 460    |
| ATR(14)          | **122 ★**         | 169    |

Run the suite yourself:

```bash
cargo bench -p wickra-bench            # Rust core vs kand / ta-rs / yata
pip install -e bindings/python[bench]  # Python peers
python -m benchmarks.compare_libraries
```

## 3. Per-binding throughput — the cost of the boundary

The sections above compare Wickra against other libraries, which only exists for
Python and Rust (there is no comparable streaming TA library for C, C++, C#, Go, Java,
R or WASM to benchmark against). Every binding calls the **same** Rust
core, so these per-binding benchmarks are **not** a speed claim and **not** a
cross-library ratio — they document the raw cost of crossing each language's FFI
boundary, in million updates per second (Mupd/s).

Each binding ships a small `throughput` benchmark that feeds a synthetic OHLCV
series through three indicators chosen by call-signature archetype — `SMA(20)`
(1-in → 1-out), `ATR(14)` (multi-in → 1-out) and `MACD(12,26,9)` (1-in →
multi-out). Two things fall out of the numbers:

- **Batch crosses once.** A `batch` call crosses the boundary a single time and the
  Rust core computes the whole series internally, so batch throughput stays high for
  most bindings — the exceptions are the ones that copy or box the result array on
  the way out (Node's JS `Array`, Python's stdlib `array.array` now that NumPy is
  optional).
- **Streaming reveals the boundary.** A per-tick `update` crosses the boundary
  once per value, so streaming throughput is where the bindings differ: the raw C
  ABI and P/Invoke-style calls are nearly free, while managed or interpreted
  per-call marshalling (cgo, FFM, the R/WASM boundary) costs more per tick.

The Rust core ships the same benchmark with **no** FFI boundary
(`examples/rust/.../throughput.rs`) — it is the ceiling each binding is measured
against and the value the batch paths converge towards.

`SMA(20)`, 200 000 bars, median of 3 runs, on the reference machine (Windows 11,
AMD Ryzen 9 9950X):

| Target               | streaming (Mupd/s) | batch (Mupd/s) |
|----------------------|-------------------:|---------------:|
| Rust core (no FFI)   |                380 |            498 |
| C / C++              |                365 |            358 |
| C#                   |                348 |            259 |
| Python               |                 31 |             46 |
| Java                 |                 38 |            173 |
| Go                   |                 23 |            394 |
| WASM                 |                 21 |            169 |
| Node.js              |                 16 |              9 |
| R                    |                0.1 |            279 |

Streaming spans more than three orders of magnitude — the raw C ABI (365) sits
just under the FFI-free Rust ceiling (380), while R's per-call interpreter overhead
(0.1) makes streaming ~2800× slower than its own batch. Batch stays high for the
bindings that hand back a contiguous buffer (slices, typed arrays); the two low
outliers are Node — whose napi `batch` boxes every element into a JS `Array` — and
Python, which now copies into a stdlib `array.array` since NumPy became optional.
These are machine-dependent and reflect FFI overhead, not algorithm speed.

These are throughput numbers, not competitive numbers — the "Wickra is fast"
claim lives in sections 1 and 2 (Rust core + the Python/Rust cross-library runs).

Run any target's benchmark (build the C ABI library first where it links one):

```bash
cargo run -p wickra-examples --release --bin throughput           # Rust core baseline (no FFI)

node bindings/node/benchmarks/throughput.js                       # native napi-rs
( cd bindings/python && python -m benchmarks.throughput )         # native PyO3
( cd bindings/wasm && wasm-pack build --target nodejs --out-dir pkg-node --release ) \
  && node bindings/wasm/benchmarks/throughput.mjs                 # wasm boundary

cargo build -p wickra-c --release                                 # the C ABI hub
cmake -S bindings/c/benchmarks -B build/cbench && cmake --build build/cbench \
  && ./build/cbench/throughput                                    # raw C ABI
dotnet run -c Release --project bindings/csharp/benchmarks        # C# (P/Invoke)
( cd bindings/go/benchmarks && go run . )                         # Go (cgo)
mvn -q -f bindings/java install -DskipTests \
  && mvn -q -f bindings/java/benchmarks exec:exec -Dexec.mainClass=org.wickra.benchmarks.Throughput
Rscript bindings/r/benchmarks/throughput.R                        # R (.Call)
```

## 4. Data layer — native I/O throughput

Wickra ships its own data layer — a CSV candle reader, a tick-to-candle
aggregator, and a timeframe resampler — so loading and reshaping market data
needs **no third-party package** (`pandas`, `csv-parse`, manual bucketing,
`pandas.resample`, …) in any of the ten languages. These run on the same Rust
core as the indicators, so every binding reaches these speeds minus the FFI
boundary characterised in section 3 (a `read` / `push` / `flush` call crosses the
boundary once per batch, like `batch`, so bindings land close to the core).

Rust core, 50 000 real BTCUSDT one-minute candles
(`examples/data/btcusdt-1m.csv`), median of 100 samples, on the reference machine
(Windows 11, AMD Ryzen 9 9950X):

| Operation                          | Throughput          | Per element |
|------------------------------------|--------------------:|------------:|
| CSV parse (`CandleReader`)         |   3.0 M candles/s   |      329 ns |
| Tick aggregate → 1m (`TickAggregator`) |  44 M ticks/s   |     22.6 ns |
| Resample 1m → 5m (`Resampler`)     | 234 M candles/s     |      4.3 ns |

Reading and validating a 50 000-row CSV into typed candles takes ~16 ms;
aggregating 50 000 ticks into one-minute bars ~1.1 ms; resampling 50 000
one-minute candles to five-minute bars ~0.2 ms. CSV parsing is the floor because
it does the most per row (UTF-8 scan, field split, six `f64` parses, finiteness
checks); aggregation and resampling are pure arithmetic over already-typed
candles.

The live and historical Binance feeds (`BinanceFeed`, `fetch_binance_klines`) are
network-bound — their throughput is set by the exchange and the socket, not by
Wickra — so they are not micro-benchmarked here; the relevant Wickra cost is the
per-event parse, which is the same arithmetic measured above.

Run it with:

```bash
cargo bench -p wickra --bench data_layer
```
