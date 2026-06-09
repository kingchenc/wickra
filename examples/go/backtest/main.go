// Compute a basket of indicators over an OHLCV series and print a summary.
// Pass a CSV path (timestamp,open,high,low,close,volume) or run on synthetic data.
package main

import (
	"fmt"
	"log"
	"math"
	"os"

	wickra "github.com/wickra-lib/wickra/bindings/go"
	"github.com/wickra-lib/wickra/examples/go/internal/market"
)

func main() {
	source := "synthetic"
	var bars []market.Bar
	if len(os.Args) > 1 {
		source = os.Args[1]
		loaded, err := market.LoadOhlcvCsv(os.Args[1])
		if err != nil {
			log.Fatalf("load csv: %v", err)
		}
		bars = loaded
	} else {
		bars = market.SyntheticCandles(1000)
	}

	fmt.Printf("Backtest over %d bars (%s):\n", len(bars), source)

	sma, _ := wickra.NewSma(20)
	defer sma.Close()
	ema, _ := wickra.NewEma(50)
	defer ema.Close()
	rsi, _ := wickra.NewRsi(14)
	defer rsi.Close()
	atr, _ := wickra.NewAtr(14)
	defer atr.Close()

	var lastSma, lastEma, lastRsi, lastAtr float64
	oversold := 0
	for _, b := range bars {
		lastSma = sma.Update(b.Close)
		lastEma = ema.Update(b.Close)
		lastRsi = rsi.Update(b.Close)
		lastAtr = atr.Update(b.Open, b.High, b.Low, b.Close, b.Volume, b.Timestamp)
		if !math.IsNaN(lastRsi) && lastRsi < 30.0 {
			oversold++
		}
	}

	fmt.Printf("  SMA(20) last = %.4f\n", lastSma)
	fmt.Printf("  EMA(50) last = %.4f\n", lastEma)
	fmt.Printf("  RSI(14) last = %.4f  (%d oversold bars)\n", lastRsi, oversold)
	fmt.Printf("  ATR(14) last = %.4f\n", lastAtr)
}
