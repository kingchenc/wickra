// Strategy example: Bollinger-squeeze breakout with an ATR(14) trailing stop.
//
// Enters long when Bollinger bandwidth makes a new SQUEEZE_LOOKBACK low (a
// volatility squeeze) and price closes above the upper band; exits on an ATR(14)
// trailing stop or when the upper band falls back below the entry. 0.1% fees per
// trade. The Go counterpart of examples/python/strategy_bollinger_squeeze.py,
// printing the same summary.
//
// Uses the checked-in examples/data/btcusdt-1d.csv dataset (daily bars give an
// interpretable ~6-month-low lookback); pass a CSV path to override.
package main

import (
	"log"
	"math"
	"os"

	wickra "github.com/wickra-lib/wickra/bindings/go"
	"github.com/wickra-lib/wickra/examples/go/internal/market"
)

const (
	fee             = 0.001
	bbPeriod        = 20
	bbK             = 2.0
	atrPeriod       = 14
	atrStopMult     = 2.0
	squeezeLookback = 180
)

func main() {
	bars := loadBars()

	bb, _ := wickra.NewBollingerBands(bbPeriod, bbK)
	defer bb.Close()
	atr, _ := wickra.NewAtr(atrPeriod)
	defer atr.Close()

	inPosition := false
	entryPrice := 0.0
	stopLevel := 0.0
	var closedTrades []float64
	equity := 1.0
	var equityCurve []float64
	var bwWindow []float64

	for _, b := range bars {
		band, okBand := bb.Update(b.Close)
		atrVal := atr.Update(b.Open, b.High, b.Low, b.Close, b.Volume, b.Timestamp)
		price := b.Close
		mtm := equity
		if inPosition {
			mtm = equity * (price / entryPrice)
		}
		equityCurve = append(equityCurve, mtm)

		if !okBand || math.IsNaN(atrVal) {
			continue
		}
		upper, middle, lower := band.Upper, band.Middle, band.Lower
		if math.Abs(middle) <= 1e-12 {
			continue
		}
		bandwidth := (upper - lower) / middle
		bwWindow = append(bwWindow, bandwidth)
		if len(bwWindow) > squeezeLookback {
			bwWindow = bwWindow[len(bwWindow)-squeezeLookback:]
		}
		if len(bwWindow) < squeezeLookback {
			continue
		}
		minBw := bwWindow[0]
		for _, v := range bwWindow {
			if v < minBw {
				minBw = v
			}
		}

		if inPosition {
			if price < stopLevel || upper < entryPrice {
				tradeRet := price/entryPrice - 1.0
				closedTrades = append(closedTrades, tradeRet)
				equity *= (1.0 + tradeRet) * (1.0 - fee)
				inPosition = false
			}
		} else {
			isNewLow := math.Abs(bandwidth-minBw) < 1e-12
			if isNewLow && price > upper {
				entryPrice = price
				stopLevel = price - atrStopMult*atrVal
				equity *= 1.0 - fee
				inPosition = true
			}
		}
	}

	if inPosition {
		lastPrice := bars[len(bars)-1].Close
		tradeRet := lastPrice/entryPrice - 1.0
		closedTrades = append(closedTrades, tradeRet)
		equity *= (1.0 + tradeRet) * (1.0 - fee)
	}

	market.PrintSummary("Bollinger Squeeze Breakout (1d, BTCUSDT)",
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
	return market.BundledCandles("btcusdt-1d.csv")
}
