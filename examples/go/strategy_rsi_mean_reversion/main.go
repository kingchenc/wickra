// Strategy example: RSI(14) mean-reversion.
//
// Go long when RSI(14) drops below 30 (oversold), exit when it recovers above
// 70 (overbought). 0.1% fees per trade. The Go counterpart of
// examples/python/strategy_rsi_mean_reversion.py, printing the same summary.
//
// Uses the checked-in examples/data/btcusdt-1h.csv dataset (pass a CSV path to
// override).
package main

import (
	"log"
	"math"
	"os"

	wickra "github.com/wickra-lib/wickra/bindings/go"
	"github.com/wickra-lib/wickra/examples/go/internal/market"
)

const (
	fee        = 0.001
	oversold   = 30.0
	overbought = 70.0
)

func main() {
	bars := loadBars()

	rsi, _ := wickra.NewRsi(14)
	defer rsi.Close()

	inPosition := false
	entryPrice := 0.0
	var closedTrades []float64
	equity := 1.0
	var equityCurve []float64

	for _, b := range bars {
		value := rsi.Update(b.Close)
		price := b.Close
		mtm := equity
		if inPosition {
			mtm = equity * (price / entryPrice)
		}
		equityCurve = append(equityCurve, mtm)
		if math.IsNaN(value) {
			continue
		}

		if !inPosition && value < oversold {
			entryPrice = price
			equity *= 1.0 - fee
			inPosition = true
		} else if inPosition && value > overbought {
			tradeRet := price/entryPrice - 1.0
			closedTrades = append(closedTrades, tradeRet)
			equity *= (1.0 + tradeRet) * (1.0 - fee)
			inPosition = false
		}
	}

	if inPosition {
		lastPrice := bars[len(bars)-1].Close
		tradeRet := lastPrice/entryPrice - 1.0
		closedTrades = append(closedTrades, tradeRet)
		equity *= (1.0 + tradeRet) * (1.0 - fee)
	}

	market.PrintSummary("RSI Mean-Reversion (1h, BTCUSDT)",
		bars[0].Close, bars[len(bars)-1].Close, len(bars), closedTrades, equity, equityCurve)
}

func loadBars() []market.Bar {
	if len(os.Args) > 1 {
		bars, err := market.LoadOhlcvCsv(os.Args[1])
		if err != nil {
			log.Fatalf("load csv: %v", err)
		}
		return bars
	}
	return market.BundledCandles("btcusdt-1h.csv")
}
