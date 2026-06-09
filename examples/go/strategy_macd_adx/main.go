// Trend follower: enter long on a MACD histogram cross up, but only when ADX(14) > 20
// confirms a trend; exit when the histogram crosses back below zero.
package main

import (
	"log"
	"math"
	"os"

	wickra "github.com/wickra-lib/wickra/bindings/go"
	"github.com/wickra-lib/wickra/examples/go/internal/market"
)

func main() {
	bars := loadBars()

	macd, _ := wickra.NewMacdIndicator(12, 26, 9)
	defer macd.Close()
	adx, _ := wickra.NewAdx(14)
	defer adx.Close()

	var returns []float64
	trades := 0
	inPosition := false
	entry := 0.0
	prevHistogram := math.NaN()

	for _, b := range bars {
		m, okMacd := macd.Update(b.Close)
		a, okAdx := adx.Update(b.Open, b.High, b.Low, b.Close, b.Volume, b.Timestamp)
		if !okMacd || !okAdx {
			continue
		}

		trending := a.Adx > 20.0
		if !inPosition && trending && !math.IsNaN(prevHistogram) && prevHistogram <= 0.0 && m.Histogram > 0.0 {
			inPosition = true
			entry = b.Close
			trades++
		} else if inPosition && m.Histogram < 0.0 {
			returns = append(returns, (b.Close-entry)/entry)
			inPosition = false
		}
		prevHistogram = m.Histogram
	}

	market.Print("MACD + ADX trend", market.Summarize(returns, trades, 252.0))
}

func loadBars() []market.Bar {
	if len(os.Args) > 1 {
		bars, err := market.LoadOhlcvCsv(os.Args[1])
		if err != nil {
			log.Fatalf("load csv: %v", err)
		}
		return bars
	}
	return market.SyntheticCandles(2000)
}
