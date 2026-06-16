package wickra

import "testing"

// The live Binance feed's connect → read → reconnect pipeline is covered
// deterministically by the Rust mock-WS-server tests in wickra-data. Here we
// only assert the binding's error paths, which need no network: a bad interval,
// an empty symbol list, and an unreachable endpoint must all surface as an
// error rather than a usable handle.
func TestBinanceFeedRejectsBadParams(t *testing.T) {
	if _, err := NewBinanceFeed("BTCUSDT", BinanceInterval(99), ""); err == nil {
		t.Fatal("expected an error for an unknown interval code")
	}
	if _, err := NewBinanceFeed("", OneMinute, ""); err == nil {
		t.Fatal("expected an error for an empty symbol list")
	}
	if _, err := NewBinanceFeed("BTCUSDT", OneMinute, "ws://127.0.0.1:1"); err == nil {
		t.Fatal("expected an error connecting to an unreachable endpoint")
	}
}

// The REST fetcher's parse/HTTP success path is covered by the Rust
// mock-HTTP-server tests in wickra-data; here we only assert the binding's
// error paths, which need no reachable network.
func TestFetchBinanceKlinesRejectsBadParams(t *testing.T) {
	if _, err := FetchBinanceKlines("BTCUSDT", BinanceInterval(99), 1, -1, -1, ""); err == nil {
		t.Fatal("expected an error for an unknown interval code")
	}
	if _, err := FetchBinanceKlines("BTCUSDT", OneHour, 0, -1, -1, ""); err == nil {
		t.Fatal("expected an error for a zero limit")
	}
	if _, err := FetchBinanceKlines("BTCUSDT", OneHour, 1, -1, -1, "http://127.0.0.1:1"); err == nil {
		t.Fatal("expected an error connecting to an unreachable endpoint")
	}
}
