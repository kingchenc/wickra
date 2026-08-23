// Resample a 1-minute series into higher timeframes and run an indicator per timeframe.
package main

import (
	"fmt"

	wickra "github.com/wickra-lib/wickra/bindings/go"
	"github.com/wickra-lib/wickra/examples/go/internal/market"
)

func main() {
	oneMinute := market.SyntheticCandlesStep(1200, 0, 60_000)

	fmt.Println("EMA(20) of close across timeframes (resampled from 1-minute bars):")
	for _, factor := range []int{1, 5, 15} {
		bars := resample(oneMinute, factor)
		ema, _ := wickra.NewEma(20)
		var last float64
		for _, b := range bars {
			last = ema.Update(b.Close)
		}
		ema.Close()
		fmt.Printf("  %2dm: %5d bars  EMA(20) last = %.4f\n", factor, len(bars), last)
	}
}

func resample(source []market.Bar, factor int) []market.Bar {
	if factor <= 1 {
		return source
	}
	// Native Resampler: bucket by an absolute timeframe (the synthetic bars step
	// 60_000 ms, so factor minutes == factor*60_000 ms). No hand-written bucketing.
	// Push returns the candles that bar closed — normally none or one, but with
	// gap filling on it would be one per skipped bucket, so always iterate.
	r, _ := wickra.NewResampler(int64(factor)*60_000, false)
	defer r.Close()
	var out []market.Bar
	emit := func(c wickra.Candle) {
		out = append(out, market.Bar{Open: c.Open, High: c.High, Low: c.Low, Close: c.Close, Volume: c.Volume, Timestamp: c.Timestamp})
	}
	for _, b := range source {
		for _, c := range r.Push(b.Open, b.High, b.Low, b.Close, b.Volume, b.Timestamp) {
			emit(c)
		}
	}
	if c, ok := r.Flush(); ok {
		emit(c)
	}
	return out
}
