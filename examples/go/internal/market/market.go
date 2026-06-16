// Package market provides deterministic synthetic market data, a small OHLCV
// CSV loader, and an equity-curve summary shared by the offline Go examples so
// they run without network access. It mirrors the helpers used by the Python,
// C, and C# example suites.
package market

import (
	"fmt"
	"math"
	"os"

	wickra "github.com/wickra-lib/wickra/bindings/go"
)

// Bar is one OHLCV bar with a millisecond timestamp.
type Bar struct {
	Open      float64
	High      float64
	Low       float64
	Close     float64
	Volume    float64
	Timestamp int64
}

// SyntheticPrices returns a reproducible price path (trend + two cycles), with
// no randomness, starting at 100.
func SyntheticPrices(count int) []float64 {
	return SyntheticPricesFrom(count, 100.0)
}

// SyntheticPricesFrom is SyntheticPrices with an explicit starting level.
func SyntheticPricesFrom(count int, start float64) []float64 {
	prices := make([]float64, count)
	for i := range prices {
		fi := float64(i)
		prices[i] = start + 12.0*math.Sin(fi*0.05) + 5.0*math.Sin(fi*0.013) + fi*0.01
	}
	return prices
}

// SyntheticCandles returns a reproducible OHLCV series derived from
// SyntheticPrices, one bar per hour.
func SyntheticCandles(count int) []Bar {
	return SyntheticCandlesStep(count, 0, 3_600_000)
}

// SyntheticCandlesStep is SyntheticCandles with an explicit start timestamp and
// per-bar step in milliseconds.
func SyntheticCandlesStep(count int, startTimestamp, stepMs int64) []Bar {
	prices := SyntheticPrices(count + 1)
	bars := make([]Bar, count)
	for i := 0; i < count; i++ {
		fi := float64(i)
		op := prices[i]
		cl := prices[i+1]
		high := math.Max(op, cl) + 0.5 + math.Abs(math.Sin(fi*0.7))
		low := math.Min(op, cl) - 0.5 - math.Abs(math.Cos(fi*0.7))
		volume := 1000.0 + 500.0*(1.0+math.Sin(fi*0.1))
		bars[i] = Bar{op, high, low, cl, volume, startTimestamp + int64(i)*stepMs}
	}
	return bars
}

// LoadOhlcvCsv loads a timestamp,open,high,low,close,volume OHLCV CSV with
// Wickra's native CandleReader (header validation, BOM and field-whitespace
// tolerance) — no manual CSV parsing.
func LoadOhlcvCsv(path string) ([]Bar, error) {
	data, err := os.ReadFile(path)
	if err != nil {
		return nil, err
	}
	reader, err := wickra.NewCandleReader(string(data))
	if err != nil {
		return nil, err
	}
	defer reader.Close()
	candles := reader.Read()
	bars := make([]Bar, len(candles))
	for i, c := range candles {
		bars[i] = Bar{c.Open, c.High, c.Low, c.Close, c.Volume, c.Timestamp}
	}
	return bars, nil
}

// EquityResult holds summary statistics for a long-only equity curve.
type EquityResult struct {
	TotalReturnPct float64
	Sharpe         float64
	MaxDrawdownPct float64
	Trades         int
	FinalEquity    float64
}

// Summarize turns a stream of per-bar fractional returns (0.01 == +1%) into a
// PnL / Sharpe / max-drawdown summary, annualised by periodsPerYear.
func Summarize(periodReturns []float64, trades int, periodsPerYear float64) EquityResult {
	equity, peak, maxDrawdown := 1.0, 1.0, 0.0
	for _, r := range periodReturns {
		equity *= 1.0 + r
		peak = math.Max(peak, equity)
		if peak > 0 {
			maxDrawdown = math.Max(maxDrawdown, (peak-equity)/peak)
		}
	}

	mean := 0.0
	if len(periodReturns) > 0 {
		var sum float64
		for _, r := range periodReturns {
			sum += r
		}
		mean = sum / float64(len(periodReturns))
	}
	variance := 0.0
	if len(periodReturns) > 1 {
		var ss float64
		for _, r := range periodReturns {
			ss += (r - mean) * (r - mean)
		}
		variance = ss / float64(len(periodReturns)-1)
	}
	stdDev := math.Sqrt(variance)
	sharpe := 0.0
	if stdDev > 1e-12 {
		sharpe = mean / stdDev * math.Sqrt(periodsPerYear)
	}

	return EquityResult{
		TotalReturnPct: (equity - 1.0) * 100.0,
		Sharpe:         sharpe,
		MaxDrawdownPct: maxDrawdown * 100.0,
		Trades:         trades,
		FinalEquity:    equity,
	}
}

// Print writes a one-line summary of an equity result.
func Print(name string, r EquityResult) {
	fmt.Printf("%-26s return=%8.2f%%  sharpe=%6.2f  maxDD=%6.2f%%  trades=%d\n",
		name, r.TotalReturnPct, r.Sharpe, r.MaxDrawdownPct, r.Trades)
}
