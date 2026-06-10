// Throughput benchmark for the Wickra Go bindings.
//
// Measures how many indicator updates per second the cgo binding sustains,
// both per-tick (streaming Update) and bulk (Batch), over a synthetic OHLCV
// series. It is the Go counterpart of the Node throughput.js and the Rust
// criterion benches: it benchmarks Wickra's own O(1) streaming engine across
// the Go<->C-ABI boundary (there is no comparable streaming TA library to
// compare against), so the headline number is raw per-binding throughput /
// FFI overhead, not a cross-library ratio.
//
// Three indicators are timed, chosen by FFI call-signature archetype rather
// than algorithm: SMA (1-in -> 1-out), ATR (multi-in -> 1-out), and MACD
// (1-in -> multi-out). Streaming is timed for all three; batch only for the
// single-output SMA and ATR (multi-output batch is not exposed uniformly).
//
// Provision the C ABI library first (see bindings/go/README.md), then run:
//
//	cd bindings/go/benchmarks
//	go run .                 # 200k bars (default)
//	go run . -bars 1000000
package main

import (
	"flag"
	"fmt"
	"math"
	"sort"
	"time"

	wickra "github.com/wickra-lib/wickra/bindings/go"
)

func main() {
	bars := flag.Int("bars", 200_000, "number of synthetic bars to feed")
	flag.Parse()
	n := *bars
	if n < 1000 {
		fmt.Println("-bars must be >= 1000")
		return
	}

	// Deterministic synthetic OHLCV (no RNG, so runs are comparable).
	open := make([]float64, n)
	high := make([]float64, n)
	low := make([]float64, n)
	closeP := make([]float64, n)
	volume := make([]float64, n)
	timestamp := make([]int64, n)
	for i := 0; i < n; i++ {
		mid := 100 + math.Sin(float64(i)*0.001)*20 + float64(i)*1e-4
		c := mid + math.Sin(float64(i)*0.05)*2
		closeP[i] = c
		open[i] = mid
		high[i] = math.Max(c, mid) + 1.5
		low[i] = math.Min(c, mid) - 1.5
		volume[i] = 1000 + float64(i%97)*13
		timestamp[i] = int64(i)
	}

	mups := func(d time.Duration) float64 {
		return float64(n) / d.Seconds() / 1e6
	}

	// Median elapsed over a few repetitions, after one warmup pass.
	timeFn := func(fn func()) time.Duration {
		fn() // warmup
		const reps = 3
		samples := make([]time.Duration, reps)
		for r := 0; r < reps; r++ {
			t0 := time.Now()
			fn()
			samples[r] = time.Since(t0)
		}
		sort.Slice(samples, func(a, b int) bool { return samples[a] < samples[b] })
		return samples[reps/2]
	}

	type indicator struct {
		name   string
		stream func()
		batch  func() // nil -> streaming only
	}

	indicators := []indicator{
		{
			name: "SMA(20)",
			stream: func() {
				ind, _ := wickra.NewSma(20)
				for i := 0; i < n; i++ {
					ind.Update(closeP[i])
				}
				ind.Close()
			},
			batch: func() {
				ind, _ := wickra.NewSma(20)
				ind.Batch(closeP)
				ind.Close()
			},
		},
		{
			name: "ATR(14)",
			stream: func() {
				ind, _ := wickra.NewAtr(14)
				for i := 0; i < n; i++ {
					ind.Update(open[i], high[i], low[i], closeP[i], volume[i], timestamp[i])
				}
				ind.Close()
			},
			batch: func() {
				ind, _ := wickra.NewAtr(14)
				ind.Batch(open, high, low, closeP, volume, timestamp)
				ind.Close()
			},
		},
		{
			name: "MACD(12,26,9)",
			stream: func() {
				ind, _ := wickra.NewMacdIndicator(12, 26, 9)
				for i := 0; i < n; i++ {
					ind.Update(closeP[i])
				}
				ind.Close()
			},
			batch: nil, // multi-output: streaming only
		},
	}

	fmt.Printf("Wickra Go throughput - %d bars (median of 3 runs)\n\n", n)
	fmt.Printf("%-22s%20s%18s\n", "Indicator", "streaming (Mupd/s)", "batch (Mupd/s)")
	fmt.Println("------------------------------------------------------------")

	for _, ind := range indicators {
		streamMups := fmt.Sprintf("%.1f", mups(timeFn(ind.stream)))
		batchMups := "-"
		if ind.batch != nil {
			batchMups = fmt.Sprintf("%.1f", mups(timeFn(ind.batch)))
		}
		fmt.Printf("%-22s%20s%18s\n", ind.name, streamMups, batchMups)
	}

	fmt.Print("\nMupd/s = million indicator updates per second. Streaming is the per-tick\n",
		"Update path crossing the Go<->C-ABI boundary once per value; batch is the\n",
		"bulk slice path (one boundary crossing). Higher is better. Numbers are\n",
		"machine-dependent - use them for relative comparison, not as a speed claim.\n")
}
