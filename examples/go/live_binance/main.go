// Stream live BTCUSDT 1-minute klines from Binance and feed each close through EMA(20).
// Uses Wickra's native BinanceFeed — no third-party WebSocket client. Requires
// network access (build-only in CI). Runs for up to 60 seconds.
package main

import (
	"fmt"
	"log"
	"time"

	wickra "github.com/wickra-lib/wickra/bindings/go"
)

func main() {
	fmt.Println("Streaming live BTCUSDT 1-minute klines from Binance (up to 60s)...")

	// Native feed: a blocking poll over the same tested stream as the Rust core.
	feed, err := wickra.NewBinanceFeed("BTCUSDT", wickra.OneMinute, "")
	if err != nil {
		log.Fatalf("connect: %v", err)
	}
	defer feed.Close()

	ema, _ := wickra.NewEma(20)
	defer ema.Close()

	deadline := time.Now().Add(60 * time.Second)
	for time.Now().Before(deadline) {
		// next() returns the event and ok=true, ok=false on timeout (poll again),
		// or an error once the stream is closed.
		event, ok, err := feed.Next(time.Second)
		if err != nil {
			fmt.Println("Done (feed closed).")
			return
		}
		if !ok {
			continue
		}
		fmt.Printf("close=%.2f  EMA(20)=%.2f\n", event.Close, ema.Update(event.Close))
	}
	fmt.Println("Done (time limit reached).")
}
