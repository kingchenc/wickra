// Stream live BTCUSDT 1-minute klines from Binance and feed each close through EMA(20).
// Requires network access (build-only in CI). Runs for up to 60 seconds.
package main

import (
	"context"
	"encoding/json"
	"fmt"
	"log"
	"strconv"
	"time"

	"github.com/coder/websocket"
	wickra "github.com/wickra-lib/wickra/bindings/go"
)

func main() {
	const url = "wss://stream.binance.com:9443/ws/btcusdt@kline_1m"
	fmt.Printf("Connecting to %s (up to 60s)...\n", url)

	ctx, cancel := context.WithTimeout(context.Background(), 60*time.Second)
	defer cancel()

	conn, _, err := websocket.Dial(ctx, url, nil)
	if err != nil {
		log.Fatalf("dial: %v", err)
	}
	defer conn.CloseNow()

	ema, _ := wickra.NewEma(20)
	defer ema.Close()

	for {
		_, data, err := conn.Read(ctx)
		if err != nil {
			fmt.Println("Done (time limit reached).")
			return
		}

		var msg struct {
			K struct {
				Close string `json:"c"`
			} `json:"k"`
		}
		if err := json.Unmarshal(data, &msg); err != nil || msg.K.Close == "" {
			continue
		}
		closePx, err := strconv.ParseFloat(msg.K.Close, 64)
		if err != nil {
			continue
		}
		fmt.Printf("close=%.2f  EMA(20)=%.2f\n", closePx, ema.Update(closePx))
	}
}
