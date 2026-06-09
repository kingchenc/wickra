# Wickra — .NET

[![CI](https://github.com/wickra-lib/wickra/actions/workflows/ci.yml/badge.svg)](https://github.com/wickra-lib/wickra/actions/workflows/ci.yml)
[![NuGet](https://img.shields.io/nuget/v/Wickra.svg?logo=nuget&color=blue)](https://www.nuget.org/packages/Wickra)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT_OR_Apache--2.0-blue)](https://github.com/wickra-lib/wickra#license)

**Streaming-first technical indicators for .NET. `dotnet add package Wickra` —
prebuilt native library, no system dependencies.**

Wickra is a multi-language technical-analysis library with a Rust core and
bindings for Python, Node.js and WebAssembly, plus a C ABI for C/C++ and any
other C-capable language. Every indicator is an O(1) streaming state machine, so
live trading bots and historical backtests share the exact same implementation.
This package is the .NET binding: it consumes the [C ABI hub](../c) through
`[LibraryImport]` source-generated P/Invoke and exposes 500+ streaming-first
indicators as idiomatic, `IDisposable` classes.

## Install

```bash
dotnet add package Wickra
```

The native library ships prebuilt per platform (Linux, macOS, Windows — x64 and
arm64) under `runtimes/<rid>/native/` and is selected automatically. There is
nothing to compile.

## Quick start

```csharp
using Wickra;

// Batch: run an indicator over a whole series (NaN at warmup positions).
var prices = Enumerable.Range(0, 1000).Select(i => 100.0 + i * 0.1).ToArray();
using var rsi = new Rsi(14);
double[] values = rsi.Batch(prices);

// Streaming: the same indicator, fed tick by tick in O(1).
using var live = new Rsi(14);
foreach (var price in feed)
{
    var value = live.Update(price); // NaN during warmup, no recomputation
}
```

Multi-output indicators return a nullable `record struct` (null while warming up):

```csharp
using var macd = new Macd(12, 26, 9);
foreach (var price in feed)
{
    if (macd.Update(price) is { } m)
    {
        Console.WriteLine($"{m.Macd} {m.Signal} {m.Histogram}");
    }
}
```

## API shape

Each indicator is an `IDisposable` class over an opaque native handle:

- **Constructor** — `new Sma(period)`. Throws `ArgumentException` on invalid
  parameters (the native constructor rejects them).
- **`Update(...)`** — feed one point; returns the output (`NaN` during warmup
  for scalar indicators, `null` for multi-output / profile indicators).
- **`Batch(ReadOnlySpan<...>)`** — run a whole series (scalar / candle / pairwise
  indicators); one output per input, `NaN` at warmup positions.
- **`Reset()`** — clear all state back to the warmup phase.
- **`Dispose()`** — release the native handle. A `SafeHandle` also frees it from
  the finalizer, so a missed `Dispose` leaks nothing permanently — but prefer
  `using` for deterministic cleanup.

Input families map to method overloads: scalar (`Update(double)`), candle
(`Update(open, high, low, close, volume, timestamp)`), pairwise
(`Update(double, double)`), order-book / cross-sectional (`Update(spans…)`), and
bar builders / profiles (returning arrays).

## How it is built

`Wickra/Generated/{NativeMethods,Indicators}.g.cs` are generated from
[`bindings/c/include/wickra.h`](../c/include/wickra.h) — the single source of
truth. The binding owns no indicator maths; it marshals types across the C ABI
boundary. `bool` crosses as `[MarshalAs(UnmanagedType.U1)]` (Rust `bool` is one
byte); opaque handles cross as `nint` kept alive across each call.

## License

MIT OR Apache-2.0, at your option.
