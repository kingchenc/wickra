// Throughput benchmark for the Wickra C# binding.
//
// Measures how many indicator updates per second the binding sustains, both
// per-tick (streaming Update) and bulk (Batch), over a synthetic OHLCV series.
// It is the C# counterpart of the Node throughput.js and the Rust criterion
// benches: it benchmarks Wickra's own O(1) streaming engine across the
// managed<->C-ABI boundary (there is no comparable streaming TA library on
// NuGet to compare against), so the headline number is raw per-binding
// throughput / FFI overhead, not a cross-library ratio.
//
// Three indicators are timed, chosen by FFI call-signature archetype rather
// than algorithm: SMA (1-in -> 1-out), ATR (multi-in -> 1-out), and MACD
// (1-in -> multi-out). Streaming is timed for all three; batch only for the
// single-output SMA and ATR (multi-output batch is not exposed uniformly).
//
//   cargo build -p wickra-c --release
//   dotnet run -c Release --project bindings/csharp/benchmarks            # 200k bars
//   dotnet run -c Release --project bindings/csharp/benchmarks -- --bars 1000000

using System.Diagnostics;
using System.Globalization;
using Wickra;

// Deterministic, locale-independent number formatting for the report.
CultureInfo.CurrentCulture = CultureInfo.InvariantCulture;

int bars = 200_000;
for (int i = 0; i < args.Length - 1; i++)
{
    if (args[i] == "--bars" && int.TryParse(args[i + 1], out var n) && n >= 1000)
    {
        bars = n;
    }
}

// Deterministic synthetic OHLCV (no RNG, so runs are comparable).
var open = new double[bars];
var high = new double[bars];
var low = new double[bars];
var close = new double[bars];
var volume = new double[bars];
var timestamp = new long[bars];
for (int i = 0; i < bars; i++)
{
    double mid = 100 + Math.Sin(i * 0.001) * 20 + i * 1e-4;
    double c = mid + Math.Sin(i * 0.05) * 2;
    close[i] = c;
    open[i] = mid;
    high[i] = Math.Max(c, mid) + 1.5;
    low[i] = Math.Min(c, mid) - 1.5;
    volume[i] = 1000 + (i % 97) * 13;
    timestamp[i] = i;
}

double Mups(double ns) => bars / (ns / 1e9) / 1e6;

// Median elapsed-ns over a few repetitions, after one warmup pass.
double TimeNs(Action fn, int reps = 3)
{
    fn(); // warmup (JIT + cache)
    var samples = new double[reps];
    for (int r = 0; r < reps; r++)
    {
        long t0 = Stopwatch.GetTimestamp();
        fn();
        samples[r] = (Stopwatch.GetTimestamp() - t0) * (1e9 / Stopwatch.Frequency);
    }
    Array.Sort(samples);
    return samples[reps / 2];
}

// SMA (scalar 1-in/1-out), ATR (multi-in/1-out), MACD (1-in/multi-out).
var indicators = new (string Name, Action Stream, Action? Batch)[]
{
    ("SMA(20)",
        () => { using var ind = new Sma(20); for (int i = 0; i < bars; i++) ind.Update(close[i]); },
        () => { using var ind = new Sma(20); ind.Batch(close); }),
    ("ATR(14)",
        () => { using var ind = new Atr(14); for (int i = 0; i < bars; i++) ind.Update(open[i], high[i], low[i], close[i], volume[i], timestamp[i]); },
        () => { using var ind = new Atr(14); ind.Batch(open, high, low, close, volume, timestamp); }),
    ("MACD(12,26,9)",
        () => { using var ind = new MacdIndicator(12, 26, 9); for (int i = 0; i < bars; i++) ind.Update(close[i]); },
        null), // multi-output: streaming only
};

Console.WriteLine($"Wickra C# throughput - {bars:N0} bars (median of 3 runs)\n");
Console.WriteLine($"{"Indicator",-22}{"streaming (Mupd/s)",20}{"batch (Mupd/s)",18}");
Console.WriteLine(new string('-', 60));

foreach (var (name, stream, batch) in indicators)
{
    string streamMups = Mups(TimeNs(stream)).ToString("F1");
    string batchMups = batch is null ? "-" : Mups(TimeNs(batch)).ToString("F1");
    Console.WriteLine($"{name,-22}{streamMups,20}{batchMups,18}");
}

Console.WriteLine(
    "\nMupd/s = million indicator updates per second. Streaming is the per-tick\n" +
    "Update path crossing the managed<->C-ABI boundary once per value; batch is\n" +
    "the bulk array path (one boundary crossing). Higher is better. Numbers are\n" +
    "machine-dependent - use them for relative comparison, not as a speed claim.");
