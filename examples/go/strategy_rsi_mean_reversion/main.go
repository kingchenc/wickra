// Mean reversion: go long when RSI(14) drops below 30, exit when it recovers above 50.
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

	rsi, _ := wickra.NewRsi(14)
	defer rsi.Close()

	var returns []float64
	trades := 0
	inPosition := false
	entry := 0.0

	for _, b := range bars {
		value := rsi.Update(b.Close)
		if math.IsNaN(value) {
			continue
		}
		if !inPosition && value < 30.0 {
			inPosition = true
			entry = b.Close
			trades++
		} else if inPosition && value > 50.0 {
			returns = append(returns, (b.Close-entry)/entry)
			inPosition = false
		}
	}

	market.Print("RSI mean-reversion", market.Summarize(returns, trades, 252.0))
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
