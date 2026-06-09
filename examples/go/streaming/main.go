// Feed a synthetic price series through several indicators tick by tick (O(1) each).
package main

import (
	"fmt"

	wickra "github.com/wickra-lib/wickra/bindings/go"
	"github.com/wickra-lib/wickra/examples/go/internal/market"
)

func main() {
	prices := market.SyntheticPrices(500)

	sma, _ := wickra.NewSma(20)
	defer sma.Close()
	ema, _ := wickra.NewEma(20)
	defer ema.Close()
	rsi, _ := wickra.NewRsi(14)
	defer rsi.Close()
	macd, _ := wickra.NewMacdIndicator(12, 26, 9)
	defer macd.Close()

	var lastSma, lastEma, lastRsi float64
	var lastMacd wickra.MacdOutput
	var haveMacd bool
	for _, price := range prices {
		lastSma = sma.Update(price)
		lastEma = ema.Update(price)
		lastRsi = rsi.Update(price)
		lastMacd, haveMacd = macd.Update(price)
	}

	fmt.Printf("Streamed %d prices through SMA(20), EMA(20), RSI(14), MACD(12,26,9):\n", len(prices))
	fmt.Printf("  SMA  = %.4f\n", lastSma)
	fmt.Printf("  EMA  = %.4f\n", lastEma)
	fmt.Printf("  RSI  = %.4f\n", lastRsi)
	if haveMacd {
		fmt.Printf("  MACD = %.4f  signal=%.4f  hist=%.4f\n", lastMacd.Macd, lastMacd.Signal, lastMacd.Histogram)
	}
}
