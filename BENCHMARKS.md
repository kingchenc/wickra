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
| SMA(20)          | **0.063 µs ★**    | 0.59 µs (9×)     | 204 µs (3 300×)       |
| EMA(20)          | **0.060 µs ★**    | 0.72 µs (12×)    | 212 µs (3 500×)       |
| RSI(14)          | **0.065 µs ★**    | 1.06 µs (16×)    | 230 µs (3 600×)       |
| MACD(12, 26, 9)  | **0.078 µs ★**    | 4.22 µs (54×)    | 245 µs (3 100×)       |
| Bollinger(20, 2) | **0.088 µs ★**    | 5.15 µs (58×)    | 229 µs (2 600×)       |

Against the only other incremental Python peer Wickra is **9–58× faster**;
against the recompute-on-every-tick libraries it is **2 600–14 000× faster**
(`finta` RSI hits 14 000×). tulipy / pandas-ta land in the same recompute band
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

| Indicator        | Wickra   | TA-Lib | tulipy | pandas-ta |
|------------------|---------:|-------:|-------:|----------:|
| SMA(20)          | 22.7     | **15.4** | 15.9 | 33.7      |
| EMA(20)          | 30.8     | **30.3** | 31.1 | 48.8      |
| RSI(14)          | 58.9     | 72.5   | **38.5** | 94.8    |
| MACD(12, 26, 9)  | 71.7     | 99.1   | **33.5** | 207.6   |
| Bollinger(20, 2) | 84.9     | 65.7   | **32.3** | 336.4   |
| ATR(14)          | 52.0     | 79.4   | **31.9** | —        |

Wickra beats TA-Lib on RSI, MACD and ATR and the whole Python field on every
row; tulipy's SIMD C stays ahead on the heavier indicators.

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
