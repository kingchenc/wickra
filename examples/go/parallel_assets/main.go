// Run SMA(20) batch over a panel of assets, serial vs goroutine fan-out, and
// report the speedup.
package main

import (
	"fmt"
	"os"
	"runtime"
	"strconv"
	"sync"
	"time"

	wickra "github.com/wickra-lib/wickra/bindings/go"
	"github.com/wickra-lib/wickra/examples/go/internal/market"
)

func main() {
	assets := argInt(1, 500)
	bars := argInt(2, 20_000)

	panel := make([][]float64, assets)
	for a := 0; a < assets; a++ {
		panel[a] = market.SyntheticPricesFrom(bars, 50.0+float64(a)*0.1)
	}

	// Warm up so the comparison is fair.
	if warm, err := wickra.NewSma(20); err == nil {
		warm.Batch(panel[0])
		warm.Close()
	}

	sink := 0.0
	start := time.Now()
	for a := 0; a < assets; a++ {
		sma, _ := wickra.NewSma(20)
		result := sma.Batch(panel[a])
		sma.Close()
		sink += result[len(result)-1]
	}
	serial := time.Since(start)

	lasts := make([]float64, assets)
	start = time.Now()
	var wg sync.WaitGroup
	work := make(chan int, assets)
	for w := 0; w < runtime.GOMAXPROCS(0); w++ {
		wg.Add(1)
		go func() {
			defer wg.Done()
			for a := range work {
				sma, _ := wickra.NewSma(20)
				result := sma.Batch(panel[a])
				sma.Close()
				lasts[a] = result[len(result)-1]
			}
		}()
	}
	for a := 0; a < assets; a++ {
		work <- a
	}
	close(work)
	wg.Wait()
	parallel := time.Since(start)

	serialMs := float64(serial.Microseconds()) / 1000.0
	parallelMs := float64(parallel.Microseconds()) / 1000.0
	fmt.Printf("%d assets x %d bars, SMA(20) batch:\n", assets, bars)
	fmt.Printf("  serial   %8.1f ms\n", serialMs)
	fmt.Printf("  parallel %8.1f ms  (%.1fx speedup)\n", parallelMs, serialMs/max(parallelMs, 1e-9))
	_ = sink
}

func argInt(i, def int) int {
	if len(os.Args) > i {
		if v, err := strconv.Atoi(os.Args[i]); err == nil {
			return v
		}
	}
	return def
}
