// Resample a 1-minute series into higher timeframes and run an indicator per timeframe.
package main

import (
	"fmt"
	"math"

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
	var out []market.Bar
	for i := 0; i < len(source); i += factor {
		end := i + factor
		if end > len(source) {
			end = len(source)
		}
		high, low, volume := math.Inf(-1), math.Inf(1), 0.0
		for j := i; j < end; j++ {
			high = math.Max(high, source[j].High)
			low = math.Min(low, source[j].Low)
			volume += source[j].Volume
		}
		out = append(out, market.Bar{
			Open:      source[i].Open,
			High:      high,
			Low:       low,
			Close:     source[end-1].Close,
			Volume:    volume,
			Timestamp: source[i].Timestamp,
		})
	}
	return out
}
