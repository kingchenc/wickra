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
