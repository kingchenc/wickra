// Strategy example: MACD crossover with ADX trend-strength filter.
//
// Enters long on a MACD histogram cross up (the histogram turns positive) while
// ADX(14) > 20 (a directional market); exits on the opposite MACD crossover
// regardless of ADX. 0.1% fees per trade. The Go counterpart of
// examples/python/strategy_macd_adx.py and the Rust strategy_macd_adx.rs,
// printing the same summary.
//
// Uses the checked-in examples/data/btcusdt-1h.csv dataset (pass a CSV path to
// override).
package main

import (
	"log"
	"os"

	wickra "github.com/wickra-lib/wickra/bindings/go"
	"github.com/wickra-lib/wickra/examples/go/internal/market"
)

const (
	fee      = 0.001
	adxFloor = 20.0
)

func main() {
	bars := loadBars()

	macd, _ := wickra.NewMacdIndicator(12, 26, 9)
	defer macd.Close()
	adx, _ := wickra.NewAdx(14)
	defer adx.Close()

	inPosition := false
	entryPrice := 0.0
	var closedTrades []float64
	equity := 1.0
	var equityCurve []float64
	havePrev := false
	prevSign := false

	for _, b := range bars {
		m, okMacd := macd.Update(b.Close)
		a, okAdx := adx.Update(b.Open, b.High, b.Low, b.Close, b.Volume, b.Timestamp)
		price := b.Close
		mtm := equity
		if inPosition {
			mtm = equity * (price / entryPrice)
		}
		equityCurve = append(equityCurve, mtm)

		if !okMacd || !okAdx {
			continue
		}

		histSign := m.Histogram > 0.0
		crossUp := havePrev && !prevSign && histSign
		crossDown := havePrev && prevSign && !histSign
		havePrev = true
		prevSign = histSign

		if !inPosition && crossUp && a.Adx > adxFloor {
			entryPrice = price
			equity *= 1.0 - fee
			inPosition = true
		} else if inPosition && crossDown {
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

	market.PrintSummary("MACD + ADX Trend Filter (1h, BTCUSDT)",
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
