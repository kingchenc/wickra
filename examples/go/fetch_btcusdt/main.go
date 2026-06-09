// Download real BTCUSDT hourly klines from the Binance REST API into a CSV that the
// other examples can consume. Requires network access (build-only in CI).
package main

import (
	"bufio"
	"encoding/json"
	"fmt"
	"io"
	"log"
	"net/http"
	"os"
	"path/filepath"
)

func main() {
	const url = "https://api.binance.com/api/v3/klines?symbol=BTCUSDT&interval=1h&limit=500"
	fmt.Printf("Fetching %s\n", url)

	resp, err := http.Get(url)
	if err != nil {
		log.Fatalf("request: %v", err)
	}
	defer resp.Body.Close()
	body, err := io.ReadAll(resp.Body)
	if err != nil {
		log.Fatalf("read body: %v", err)
	}

	// Binance kline array: [openTime, open, high, low, close, volume, ...].
	var klines [][]any
	if err := json.Unmarshal(body, &klines); err != nil {
		log.Fatalf("parse json: %v", err)
	}

	dir := "data"
	if err := os.MkdirAll(dir, 0o755); err != nil {
		log.Fatalf("mkdir: %v", err)
	}
	path := filepath.Join(dir, "btcusdt_1h.csv")
	file, err := os.Create(path)
	if err != nil {
		log.Fatalf("create: %v", err)
	}
	defer file.Close()

	writer := bufio.NewWriter(file)
	defer writer.Flush()
	fmt.Fprintln(writer, "timestamp,open,high,low,close,volume")
	count := 0
	for _, k := range klines {
		ts := int64(k[0].(float64))
		fmt.Fprintf(writer, "%d,%s,%s,%s,%s,%s\n", ts, k[1], k[2], k[3], k[4], k[5])
		count++
	}

	fmt.Printf("Wrote %d klines to %s\n", count, path)
}
