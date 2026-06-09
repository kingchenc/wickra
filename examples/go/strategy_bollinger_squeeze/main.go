// Breakout: when Bollinger bandwidth is tight (a "squeeze") and price closes above
// the upper band, go long with an ATR(14) trailing stop.
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

	bollinger, _ := wickra.NewBollingerBands(20, 2.0)
	defer bollinger.Close()
	atr, _ := wickra.NewAtr(14)
	defer atr.Close()

	var returns []float64
	trades := 0
	inPosition := false
	entry := 0.0
	stop := 0.0

	for _, b := range bars {
		band, okBand := bollinger.Update(b.Close)
		atrValue := atr.Update(b.Open, b.High, b.Low, b.Close, b.Volume, b.Timestamp)
		if !okBand || math.IsNaN(atrValue) {
			continue
		}

		bandwidth := math.MaxFloat64
		if band.Middle != 0.0 {
			bandwidth = (band.Upper - band.Lower) / band.Middle
		}

		if !inPosition && bandwidth < 0.06 && b.Close > band.Upper {
			inPosition = true
			entry = b.Close
			stop = b.Close - 2.0*atrValue
			trades++
		} else if inPosition {
			stop = math.Max(stop, b.Close-2.0*atrValue) // trail the stop up
			if b.Close < stop {
				returns = append(returns, (b.Close-entry)/entry)
				inPosition = false
			}
		}
	}

	market.Print("Bollinger squeeze", market.Summarize(returns, trades, 252.0))
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
