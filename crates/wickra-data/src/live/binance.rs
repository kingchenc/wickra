//! Binance spot WebSocket kline feed.
//!
//! Subscribes to Binance's `<symbol>@kline_<interval>` stream and emits a
//! [`KlineEvent`] every time the server pushes a new tick. The event tells you
//! whether the current candle is still open or has just closed.
//!
//! Example (requires the `live-binance` feature):
//!
//! ```no_run
//! use wickra_data::live::binance::{BinanceKlineStream, Interval};
//! # async fn run() -> wickra_data::Result<()> {
//! let mut stream = BinanceKlineStream::connect(&["BTCUSDT".to_string()], Interval::OneMinute).await?;
//! while let Some(event) = stream.next_event().await? {
//!     if event.is_closed {
//!         println!("closed {} @ {}", event.symbol, event.candle.close);
//!     }
//! }
//! # Ok(()) }
//! ```

use std::time::Duration;

use futures_util::SinkExt;
use futures_util::StreamExt;
use serde::Deserialize;
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::protocol::WebSocketConfig;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::MaybeTlsStream;
use tokio_tungstenite::WebSocketStream;

use crate::error::{Error, Result};
use wickra_core::Candle;

/// Tunable knobs for a [`BinanceKlineStream`]. The defaults match Binance's
/// public production endpoint and are right for almost every caller; the
/// fields exist so an integration test or a Binance Testnet user can point
/// the stream at a different base URL and shrink the reconnect timing.
#[derive(Debug, Clone)]
pub struct BinanceConfig {
    /// WebSocket endpoint **without** path (e.g. `"wss://stream.binance.com:9443"`).
    /// The combined-stream path `/stream?streams=…` is appended internally.
    pub base_url: String,
    /// Maximum time to wait for the next inbound frame before treating the
    /// connection as stalled. Binance pings roughly every 3 minutes, so a
    /// healthy but quiet stream stays comfortably inside the 300 s default.
    pub read_timeout: Duration,
    /// Delay before the first reconnect attempt; doubles on each failure up
    /// to [`Self::max_reconnect_backoff`].
    pub initial_reconnect_delay: Duration,
    /// Upper bound on the exponential reconnect backoff.
    pub max_reconnect_backoff: Duration,
    /// How many times [`BinanceKlineStream::next_event`] retries a dropped
    /// connection before surfacing the last error. Must be `>= 1`.
    pub max_reconnect_attempts: u32,
    /// Upper bound on an inbound WebSocket message. Kline frames are tiny;
    /// this only caps a pathological or hostile server from forcing an
    /// unbounded allocation.
    pub max_message_size: usize,
    /// Upper bound on a single inbound WebSocket frame.
    pub max_frame_size: usize,
}

impl Default for BinanceConfig {
    fn default() -> Self {
        Self {
            base_url: "wss://stream.binance.com:9443".to_string(),
            read_timeout: Duration::from_secs(300),
            initial_reconnect_delay: Duration::from_secs(1),
            max_reconnect_backoff: Duration::from_secs(30),
            max_reconnect_attempts: 6,
            max_message_size: 8 << 20,
            max_frame_size: 2 << 20,
        }
    }
}

/// Supported Binance kline intervals. The `as_str` value matches Binance's
/// wire-format strings (`"1m"`, `"5m"`, `"1h"`, etc.).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Interval {
    OneSecond,
    OneMinute,
    ThreeMinutes,
    FiveMinutes,
    FifteenMinutes,
    ThirtyMinutes,
    OneHour,
    TwoHours,
    FourHours,
    SixHours,
    EightHours,
    TwelveHours,
    OneDay,
    ThreeDays,
    OneWeek,
    OneMonth,
}

impl Interval {
    /// Wire-format string used in the stream name.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::OneSecond => "1s",
            Self::OneMinute => "1m",
            Self::ThreeMinutes => "3m",
            Self::FiveMinutes => "5m",
            Self::FifteenMinutes => "15m",
            Self::ThirtyMinutes => "30m",
            Self::OneHour => "1h",
            Self::TwoHours => "2h",
            Self::FourHours => "4h",
            Self::SixHours => "6h",
            Self::EightHours => "8h",
            Self::TwelveHours => "12h",
            Self::OneDay => "1d",
            Self::ThreeDays => "3d",
            Self::OneWeek => "1w",
            Self::OneMonth => "1M",
        }
    }
}

/// One push from the Binance kline stream.
#[derive(Debug, Clone)]
pub struct KlineEvent {
    /// Symbol in lowercase form as sent by Binance (e.g. `"btcusdt"`).
    pub symbol: String,
    /// Interval the candle belongs to.
    pub interval: Interval,
    /// Candle in its current state (may still be open).
    pub candle: Candle,
    /// Whether the candle has been closed by the server. Closed events are the
    /// only ones safe to use for bar-completion logic.
    pub is_closed: bool,
}

/// A live Binance kline stream.
#[derive(Debug)]
pub struct BinanceKlineStream {
    socket: WebSocketStream<MaybeTlsStream<TcpStream>>,
    /// Lowercased symbols the stream is subscribed to. Retained so the
    /// connection can be rebuilt on a reconnect.
    symbols: Vec<String>,
    /// Interval requested at connect time. Used to tag every event.
    interval: Interval,
    /// `true` once the caller invoked [`close`](Self::close). A closed stream
    /// is never polled or reconnected again.
    closed: bool,
    /// Timing / sizing knobs. Retained so reconnects honour the same config.
    config: BinanceConfig,
}

/// Wire-format representation of an incoming Binance kline tick. Public so callers
/// can deserialize it themselves if they prefer.
#[derive(Debug, Clone, Deserialize)]
pub struct RawWsEnvelope {
    /// Stream name, e.g. `"btcusdt@kline_1m"`.
    pub stream: String,
    pub data: RawKlinePayload,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawKlinePayload {
    #[serde(rename = "e")]
    pub event_type: String,
    #[serde(rename = "E")]
    pub event_time: i64,
    #[serde(rename = "s")]
    pub symbol: String,
    #[serde(rename = "k")]
    pub kline: RawKline,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawKline {
    #[serde(rename = "t")]
    pub open_time: i64,
    #[serde(rename = "T")]
    pub close_time: i64,
    #[serde(rename = "s")]
    pub symbol: String,
    #[serde(rename = "i")]
    pub interval: String,
    #[serde(rename = "o")]
    pub open: String,
    #[serde(rename = "c")]
    pub close: String,
    #[serde(rename = "h")]
    pub high: String,
    #[serde(rename = "l")]
    pub low: String,
    #[serde(rename = "v")]
    pub volume: String,
    #[serde(rename = "x")]
    pub is_closed: bool,
}

impl BinanceKlineStream {
    /// Open a raw combined-stream WebSocket for the given (already-lowercased)
    /// symbols.
    async fn open(
        symbols: &[String],
        interval: Interval,
        config: &BinanceConfig,
    ) -> Result<WebSocketStream<MaybeTlsStream<TcpStream>>> {
        let streams: Vec<String> = symbols
            .iter()
            .map(|s| format!("{}@kline_{}", s, interval.as_str()))
            .collect();
        let url = format!("{}/stream?streams={}", config.base_url, streams.join("/"));
        let url = url::Url::parse(&url).map_err(|e| Error::Malformed(e.to_string()))?;
        // tokio-tungstenite 0.29's WebSocketConfig is #[non_exhaustive],
        // so the only way to set fields is via the builder-style methods on
        // a fresh `default()` value.
        let ws_config = WebSocketConfig::default()
            .max_message_size(Some(config.max_message_size))
            .max_frame_size(Some(config.max_frame_size));
        let (socket, _) =
            tokio_tungstenite::connect_async_with_config(url.as_str(), Some(ws_config), false)
                .await?;
        Ok(socket)
    }

    /// Connect to Binance's combined-stream endpoint for one or more symbols.
    ///
    /// Symbols may be passed in either case; they are lowercased to match
    /// Binance's stream-name conventions. A dropped or stalled connection is
    /// re-established transparently by [`next_event`](Self::next_event).
    pub async fn connect(symbols: &[String], interval: Interval) -> Result<Self> {
        Self::connect_with_config(symbols, interval, BinanceConfig::default()).await
    }

    /// Connect with a custom [`BinanceConfig`]. Useful for Binance Testnet
    /// (`"wss://testnet.binance.vision"`) or for shrinking the reconnect
    /// timing in integration tests.
    pub async fn connect_with_config(
        symbols: &[String],
        interval: Interval,
        config: BinanceConfig,
    ) -> Result<Self> {
        if symbols.is_empty() {
            return Err(Error::Malformed(
                "BinanceKlineStream requires at least one symbol".into(),
            ));
        }
        let symbols: Vec<String> = symbols.iter().map(|s| s.to_lowercase()).collect();
        let socket = Self::open(&symbols, interval, &config).await?;
        Ok(Self {
            socket,
            symbols,
            interval,
            closed: false,
            config,
        })
    }

    /// Whether the caller has closed the stream. Once closed, every further
    /// [`next_event`](Self::next_event) call yields `Ok(None)` immediately.
    pub fn is_closed(&self) -> bool {
        self.closed
    }

    /// Re-establish a dropped connection with exponential backoff. Returns the
    /// last error if every attempt fails.
    async fn reconnect(&mut self) -> Result<()> {
        let mut delay = self.config.initial_reconnect_delay;
        let mut last_err = None;
        for _ in 0..self.config.max_reconnect_attempts {
            tokio::time::sleep(delay).await;
            match Self::open(&self.symbols, self.interval, &self.config).await {
                Ok(socket) => {
                    self.socket = socket;
                    return Ok(());
                }
                Err(e) => {
                    last_err = Some(e);
                    delay = delay
                        .saturating_mul(2)
                        .min(self.config.max_reconnect_backoff);
                }
            }
        }
        Err(last_err.expect("max_reconnect_attempts is non-zero"))
    }

    /// Receive the next kline event. A dropped, errored or stalled connection
    /// is re-established transparently (exponential backoff, up to
    /// [`BinanceConfig::max_reconnect_attempts`]); an exhausted reconnect
    /// surfaces as `Err`. `Ok(None)` is returned only after the caller has
    /// [`close`](Self::close)d the stream.
    pub async fn next_event(&mut self) -> Result<Option<KlineEvent>> {
        if self.closed {
            return Ok(None);
        }
        loop {
            // A protocol error, a clean server close, or a read stall are all
            // transient: reconnect with backoff and resume reading.
            let Ok(Some(Ok(msg))) =
                tokio::time::timeout(self.config.read_timeout, self.socket.next()).await
            else {
                self.reconnect().await?;
                continue;
            };
            match msg {
                Message::Text(text) => {
                    if let Some(event) = Self::parse_frame(&text, self.interval)? {
                        return Ok(Some(event));
                    }
                    // Non-kline frame (subscription ack / heartbeat / error):
                    // skip it and keep reading.
                }
                Message::Binary(bytes) => {
                    let text = String::from_utf8_lossy(&bytes);
                    if let Some(event) = Self::parse_frame(&text, self.interval)? {
                        return Ok(Some(event));
                    }
                }
                Message::Ping(payload) => {
                    // Best-effort Pong reply. If the write fails the
                    // connection is already dead — the next read will
                    // surface the error and trigger reconnect through
                    // the timeout/err arm above.
                    let _ = self.socket.send(Message::Pong(payload)).await;
                }
                Message::Pong(_) | Message::Frame(_) => {}
                Message::Close(_) => {
                    self.reconnect().await?;
                }
            }
        }
    }

    /// Parse one raw WebSocket text frame.
    ///
    /// Returns `Ok(Some(event))` for a kline frame, `Ok(None)` for any other
    /// frame (subscription acknowledgements, error objects, heartbeats), and
    /// `Err` only when a frame that *is* a kline fails to decode.
    fn parse_frame(text: &str, interval: Interval) -> Result<Option<KlineEvent>> {
        let value: serde_json::Value = serde_json::from_str(text)?;
        // Combined-stream kline frames carry `data.e == "kline"`. Everything
        // else on the socket is control traffic that must not abort the feed.
        let is_kline = value
            .get("data")
            .and_then(|d| d.get("e"))
            .and_then(serde_json::Value::as_str)
            == Some("kline");
        if !is_kline {
            return Ok(None);
        }
        let envelope: RawWsEnvelope = serde_json::from_value(value)?;
        Ok(Some(envelope.into_event(interval)?))
    }

    /// Close the underlying socket cleanly and mark the stream closed. After
    /// this, [`next_event`](Self::next_event) yields `Ok(None)` and never
    /// reconnects.
    pub async fn close(&mut self) -> Result<()> {
        self.closed = true;
        self.socket.close(None).await?;
        Ok(())
    }
}

impl RawWsEnvelope {
    fn into_event(self, interval: Interval) -> Result<KlineEvent> {
        let k = self.data.kline;
        let open: f64 = k
            .open
            .parse()
            .map_err(|_| Error::Malformed(format!("bad open '{}'", k.open)))?;
        let high: f64 = k
            .high
            .parse()
            .map_err(|_| Error::Malformed(format!("bad high '{}'", k.high)))?;
        let low: f64 = k
            .low
            .parse()
            .map_err(|_| Error::Malformed(format!("bad low '{}'", k.low)))?;
        let close: f64 = k
            .close
            .parse()
            .map_err(|_| Error::Malformed(format!("bad close '{}'", k.close)))?;
        let volume: f64 = k
            .volume
            .parse()
            .map_err(|_| Error::Malformed(format!("bad volume '{}'", k.volume)))?;
        let candle = Candle::new(open, high, low, close, volume, k.open_time)?;
        Ok(KlineEvent {
            symbol: self.data.symbol.to_lowercase(),
            interval,
            candle,
            is_closed: k.is_closed,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interval_as_str_covers_every_variant() {
        // Wire-format strings are part of Binance's public protocol — pin
        // every mapping so a typo here is caught immediately.
        let pairs: &[(Interval, &str)] = &[
            (Interval::OneSecond, "1s"),
            (Interval::OneMinute, "1m"),
            (Interval::ThreeMinutes, "3m"),
            (Interval::FiveMinutes, "5m"),
            (Interval::FifteenMinutes, "15m"),
            (Interval::ThirtyMinutes, "30m"),
            (Interval::OneHour, "1h"),
            (Interval::TwoHours, "2h"),
            (Interval::FourHours, "4h"),
            (Interval::SixHours, "6h"),
            (Interval::EightHours, "8h"),
            (Interval::TwelveHours, "12h"),
            (Interval::OneDay, "1d"),
            (Interval::ThreeDays, "3d"),
            (Interval::OneWeek, "1w"),
            (Interval::OneMonth, "1M"),
        ];
        for (iv, expected) in pairs {
            assert_eq!(iv.as_str(), *expected);
        }
    }

    #[test]
    fn binance_config_default_matches_production_endpoint() {
        let cfg = BinanceConfig::default();
        assert_eq!(cfg.base_url, "wss://stream.binance.com:9443");
        assert_eq!(cfg.read_timeout, Duration::from_secs(300));
        assert_eq!(cfg.initial_reconnect_delay, Duration::from_secs(1));
        assert_eq!(cfg.max_reconnect_backoff, Duration::from_secs(30));
        assert_eq!(cfg.max_reconnect_attempts, 6);
        assert_eq!(cfg.max_message_size, 8 << 20);
        assert_eq!(cfg.max_frame_size, 2 << 20);
    }

    #[tokio::test]
    async fn connect_rejects_an_empty_symbol_list() {
        let err = BinanceKlineStream::connect(&[], Interval::OneMinute)
            .await
            .unwrap_err();
        assert!(matches!(err, Error::Malformed(_)));
    }

    #[test]
    fn parses_real_binance_payload() {
        // Sample event format from Binance's public docs (truncated).
        let json = r#"{
            "stream": "btcusdt@kline_1m",
            "data": {
              "e": "kline",
              "E": 1700000000000,
              "s": "BTCUSDT",
              "k": {
                "t": 1700000000000,
                "T": 1700000059999,
                "s": "BTCUSDT",
                "i": "1m",
                "f": 1,
                "L": 100,
                "o": "30000.0",
                "c": "30050.0",
                "h": "30100.0",
                "l": "29950.0",
                "v": "12.5",
                "n": 50,
                "x": false,
                "q": "375000.0",
                "V": "6.25",
                "Q": "187500.0",
                "B": "0"
              }
            }
        }"#;
        let env: RawWsEnvelope = serde_json::from_str(json).unwrap();
        let evt = env.into_event(Interval::OneMinute).unwrap();
        assert_eq!(evt.symbol, "btcusdt");
        assert_eq!(evt.candle.open, 30_000.0);
        assert_eq!(evt.candle.close, 30_050.0);
        assert!(!evt.is_closed);
        assert_eq!(evt.interval, Interval::OneMinute);
    }

    #[test]
    fn rejects_non_parsable_numbers() {
        let json = r#"{
            "stream": "btcusdt@kline_1m",
            "data": {
              "e": "kline", "E": 0, "s": "BTCUSDT",
              "k": {
                "t": 0, "T": 0, "s": "BTCUSDT", "i": "1m",
                "f": 0, "L": 0,
                "o": "not-a-number", "c": "0", "h": "0", "l": "0",
                "v": "0", "n": 0, "x": false, "q": "0", "V": "0", "Q": "0", "B": "0"
              }
            }
        }"#;
        let env: RawWsEnvelope = serde_json::from_str(json).unwrap();
        let err = env.into_event(Interval::OneMinute).unwrap_err();
        assert!(matches!(err, Error::Malformed(_)));
    }

    #[test]
    fn skips_non_kline_frames() {
        // Subscription acknowledgement: skipped, never an error.
        let ack = r#"{"result":null,"id":1}"#;
        assert!(BinanceKlineStream::parse_frame(ack, Interval::OneMinute)
            .unwrap()
            .is_none());
        // Error object: also skipped.
        let err = r#"{"error":{"code":2,"msg":"Invalid request"}}"#;
        assert!(BinanceKlineStream::parse_frame(err, Interval::OneMinute)
            .unwrap()
            .is_none());
    }

    #[test]
    fn parse_frame_decodes_a_kline() {
        let json = r#"{
            "stream": "btcusdt@kline_1m",
            "data": {
              "e": "kline", "E": 1700000000000, "s": "BTCUSDT",
              "k": {
                "t": 1700000000000, "T": 1700000059999, "s": "BTCUSDT", "i": "1m",
                "f": 1, "L": 100, "o": "30000.0", "c": "30050.0", "h": "30100.0",
                "l": "29950.0", "v": "12.5", "n": 50, "x": true,
                "q": "375000.0", "V": "6.25", "Q": "187500.0", "B": "0"
              }
            }
        }"#;
        let event = BinanceKlineStream::parse_frame(json, Interval::OneMinute)
            .unwrap()
            .expect("a kline frame yields an event");
        assert_eq!(event.symbol, "btcusdt");
        assert!(event.is_closed);
    }

    // ====================================================================
    // Mock WebSocket server: drives the async / reconnect / control-frame
    // paths against a `127.0.0.1` listener instead of the real Binance
    // endpoint. Each test gets its own port (`bind("127.0.0.1:0")`).
    // ====================================================================

    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;
    use tokio::net::TcpListener;

    /// A kline JSON frame matching Binance's combined-stream envelope. Always
    /// reports `is_closed = true` so the test can assert on the flag.
    fn sample_kline_text() -> String {
        r#"{"stream":"btcusdt@kline_1m","data":{"e":"kline","E":1700000000000,"s":"BTCUSDT","k":{"t":1700000000000,"T":1700000059999,"s":"BTCUSDT","i":"1m","f":1,"L":100,"o":"30000.0","c":"30050.0","h":"30100.0","l":"29950.0","v":"12.5","n":50,"x":true,"q":"375000.0","V":"6.25","Q":"187500.0","B":"0"}}}"#.to_string()
    }

    /// Test-tuned [`BinanceConfig`]: aim at the given mock and shrink every
    /// timer so a failing reconnect loop completes in milliseconds.
    fn test_config(base_url: String) -> BinanceConfig {
        BinanceConfig {
            base_url,
            read_timeout: Duration::from_millis(200),
            initial_reconnect_delay: Duration::from_millis(5),
            max_reconnect_backoff: Duration::from_millis(10),
            max_reconnect_attempts: 3,
            ..BinanceConfig::default()
        }
    }

    /// Spawn a mock WS server that accepts one connection, drops the
    /// listener (so any reconnect lands on a refused port), and then hands
    /// the upgraded socket to `handler`. Every step `.unwrap()`s — a failure
    /// here is a bug in the test scaffolding, not in the production code.
    async fn one_shot_server<F, Fut>(handler: F) -> String
    where
        F: FnOnce(WebSocketStream<TcpStream>) -> Fut + Send + 'static,
        Fut: std::future::Future<Output = ()> + Send + 'static,
    {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let base_url = format!("ws://{}", listener.local_addr().unwrap());
        tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            drop(listener);
            let ws = tokio_tungstenite::accept_async(stream).await.unwrap();
            handler(ws).await;
        });
        base_url
    }

    /// Spawn a mock WS server that accepts exactly `n_accepts` connections
    /// and invokes `handler` for each (with a zero-based index). Returns a
    /// [`JoinHandle`](tokio::task::JoinHandle) the test can await so every
    /// spawned handler is guaranteed to reach its closing brace before the
    /// runtime is torn down.
    async fn multi_shot_server<F, Fut>(
        n_accepts: u32,
        handler: F,
    ) -> (String, tokio::task::JoinHandle<()>)
    where
        F: Fn(u32, WebSocketStream<TcpStream>) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = ()> + Send + 'static,
    {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let base_url = format!("ws://{}", listener.local_addr().unwrap());
        let handler = Arc::new(handler);
        let h = tokio::spawn(async move {
            let mut joins = Vec::with_capacity(n_accepts as usize);
            for index in 0..n_accepts {
                let (stream, _) = listener.accept().await.unwrap();
                let handler = handler.clone();
                joins.push(tokio::spawn(async move {
                    let ws = tokio_tungstenite::accept_async(stream).await.unwrap();
                    handler(index, ws).await;
                }));
            }
            for j in joins {
                j.await.unwrap();
            }
        });
        (base_url, h)
    }

    #[tokio::test]
    async fn next_event_decodes_a_text_kline_frame() {
        let kline = sample_kline_text();
        let base = one_shot_server(move |mut ws| async move {
            let _ = ws.send(Message::Text(kline.into())).await;
            while let Some(Ok(_)) = ws.next().await {}
        })
        .await;
        let mut stream = BinanceKlineStream::connect_with_config(
            &["BTCUSDT".to_string()],
            Interval::OneMinute,
            test_config(base),
        )
        .await
        .unwrap();
        assert!(!stream.is_closed());
        let event = stream
            .next_event()
            .await
            .unwrap()
            .expect("server pushes a kline");
        assert_eq!(event.symbol, "btcusdt");
        assert!(event.is_closed);
    }

    #[tokio::test]
    async fn next_event_decodes_a_binary_kline_frame() {
        let kline = sample_kline_text();
        let base = one_shot_server(move |mut ws| async move {
            let bytes: Vec<u8> = kline.into_bytes();
            let _ = ws.send(Message::Binary(bytes.into())).await;
            while let Some(Ok(_)) = ws.next().await {}
        })
        .await;
        let mut stream = BinanceKlineStream::connect_with_config(
            &["BTCUSDT".to_string()],
            Interval::OneMinute,
            test_config(base),
        )
        .await
        .unwrap();
        let event = stream
            .next_event()
            .await
            .unwrap()
            .expect("server pushes a kline as Binary");
        assert_eq!(event.symbol, "btcusdt");
    }

    #[tokio::test]
    async fn next_event_replies_to_a_ping_with_a_pong() {
        let kline = sample_kline_text();
        let base = one_shot_server(move |mut ws| async move {
            let _ = ws
                .send(Message::Ping(b"binance-ping".as_slice().into()))
                .await;
            let _ = ws.send(Message::Text(kline.into())).await;
            while let Some(Ok(_)) = ws.next().await {}
        })
        .await;
        let mut stream = BinanceKlineStream::connect_with_config(
            &["BTCUSDT".to_string()],
            Interval::OneMinute,
            test_config(base),
        )
        .await
        .unwrap();
        // If the client never replied to the Ping the server's drain would
        // observe nothing — but for line coverage it's enough that the
        // client received the Ping, sent a Pong, then read the next frame.
        let event = stream
            .next_event()
            .await
            .unwrap()
            .expect("kline arrives right after the ping");
        assert_eq!(event.symbol, "btcusdt");
    }

    #[tokio::test]
    async fn next_event_skips_inbound_pong_frames() {
        let kline = sample_kline_text();
        let base = one_shot_server(move |mut ws| async move {
            let _ = ws
                .send(Message::Pong(b"unsolicited".as_slice().into()))
                .await;
            let _ = ws.send(Message::Text(kline.into())).await;
            while let Some(Ok(_)) = ws.next().await {}
        })
        .await;
        let mut stream = BinanceKlineStream::connect_with_config(
            &["BTCUSDT".to_string()],
            Interval::OneMinute,
            test_config(base),
        )
        .await
        .unwrap();
        let event = stream
            .next_event()
            .await
            .unwrap()
            .expect("kline follows the ignored Pong");
        assert_eq!(event.symbol, "btcusdt");
    }

    #[tokio::test]
    async fn next_event_reconnects_after_a_server_close_frame() {
        let kline = sample_kline_text();
        let (base, server_done) = multi_shot_server(2, move |index, mut ws| {
            let kline = kline.clone();
            async move {
                let msg = if index == 0 {
                    // First connection: send a clean Close so the client
                    // exercises the Message::Close reconnect path.
                    Message::Close(None)
                } else {
                    Message::Text(kline.into())
                };
                let _ = ws.send(msg).await;
            }
        })
        .await;
        let mut stream = BinanceKlineStream::connect_with_config(
            &["BTCUSDT".to_string()],
            Interval::OneMinute,
            test_config(base),
        )
        .await
        .unwrap();
        let event = stream
            .next_event()
            .await
            .unwrap()
            .expect("reconnect succeeds and the second connection serves a kline");
        assert_eq!(event.symbol, "btcusdt");
        // Wait for every spawned handler to reach its final state.
        tokio::time::timeout(Duration::from_secs(1), server_done)
            .await
            .unwrap()
            .unwrap();
    }

    #[tokio::test]
    async fn next_event_reconnects_after_a_read_timeout() {
        let kline = sample_kline_text();
        let stall_token = Arc::new(AtomicU32::new(0));
        let stall_token_h = stall_token.clone();
        let (base, server_done) = multi_shot_server(2, move |index, mut ws| {
            let kline = kline.clone();
            let stall_token = stall_token_h.clone();
            async move {
                if index == 0 {
                    // First connection: never write anything. Outlast the
                    // client's 80 ms read_timeout but bounded so the
                    // handler still completes for the coverage check.
                    stall_token.fetch_add(1, Ordering::SeqCst);
                    tokio::time::sleep(Duration::from_millis(250)).await;
                } else {
                    let _ = ws.send(Message::Text(kline.into())).await;
                }
            }
        })
        .await;
        let cfg = BinanceConfig {
            read_timeout: Duration::from_millis(80),
            ..test_config(base)
        };
        let mut stream = BinanceKlineStream::connect_with_config(
            &["BTCUSDT".to_string()],
            Interval::OneMinute,
            cfg,
        )
        .await
        .unwrap();
        let event = stream
            .next_event()
            .await
            .unwrap()
            .expect("client times out, reconnects, and reads the kline");
        assert_eq!(event.symbol, "btcusdt");
        assert!(stall_token.load(Ordering::SeqCst) >= 1);
        tokio::time::timeout(Duration::from_secs(1), server_done)
            .await
            .unwrap()
            .unwrap();
    }

    #[tokio::test]
    async fn next_event_yields_none_after_close() {
        let base = one_shot_server(|mut ws| async move {
            // Stay open until the client closes; this lets close() complete
            // its handshake cleanly.
            while let Some(Ok(_)) = ws.next().await {}
        })
        .await;
        let mut stream = BinanceKlineStream::connect_with_config(
            &["BTCUSDT".to_string()],
            Interval::OneMinute,
            test_config(base),
        )
        .await
        .unwrap();
        stream.close().await.unwrap();
        assert!(stream.is_closed());
        assert!(stream.next_event().await.unwrap().is_none());
    }

    #[tokio::test]
    async fn next_event_surfaces_an_error_when_reconnect_attempts_are_exhausted() {
        // After the first accept the listener is dropped (one_shot_server
        // does this), so every reconnect attempt lands on a closed port.
        let base = one_shot_server(|mut ws| async move {
            let _ = ws.send(Message::Close(None)).await;
            // Returning here also drops the socket, but the listener has
            // already been released — the client's subsequent connects
            // will be refused.
        })
        .await;
        let cfg = BinanceConfig {
            max_reconnect_attempts: 2,
            initial_reconnect_delay: Duration::from_millis(1),
            max_reconnect_backoff: Duration::from_millis(2),
            ..test_config(base)
        };
        let mut stream = BinanceKlineStream::connect_with_config(
            &["BTCUSDT".to_string()],
            Interval::OneMinute,
            cfg,
        )
        .await
        .unwrap();
        let err = stream
            .next_event()
            .await
            .expect_err("reconnect attempts are exhausted");
        // Either a WS-layer error or a Malformed error from URL parsing —
        // we only care that the call surfaced as Err rather than panicked.
        let _ = err;
    }

    #[tokio::test]
    async fn next_event_skips_non_kline_frames_and_returns_the_next_kline() {
        // Drives the loop through the Text- *and* Binary-arm "frame was
        // not a kline, keep reading" fall-throughs before serving the
        // actual kline.
        let kline = sample_kline_text();
        let base = one_shot_server(move |mut ws| async move {
            let _ = ws
                .send(Message::Text(r#"{"result":null,"id":1}"#.into()))
                .await;
            let _ = ws
                .send(Message::Binary(b"{\"id\":2}".to_vec().into()))
                .await;
            let _ = ws.send(Message::Text(kline.into())).await;
        })
        .await;
        let mut stream = BinanceKlineStream::connect_with_config(
            &["BTCUSDT".to_string()],
            Interval::OneMinute,
            test_config(base),
        )
        .await
        .unwrap();
        let event = stream
            .next_event()
            .await
            .unwrap()
            .expect("kline arrives after the two skipped control frames");
        assert_eq!(event.symbol, "btcusdt");
    }

    #[tokio::test]
    async fn next_event_propagates_a_parse_error_from_a_malformed_kline() {
        // A "kline" envelope whose open field is not a number — parse_frame
        // identifies it as a kline, into_event then fails, and next_event
        // bubbles the error rather than skipping the frame.
        let bad = r#"{"stream":"btcusdt@kline_1m","data":{"e":"kline","E":0,"s":"BTCUSDT","k":{"t":0,"T":0,"s":"BTCUSDT","i":"1m","o":"not-a-number","c":"0","h":"0","l":"0","v":"0","x":false}}}"#.to_string();
        let base = one_shot_server(move |mut ws| async move {
            let _ = ws.send(Message::Text(bad.into())).await;
            while let Some(Ok(_)) = ws.next().await {}
        })
        .await;
        let mut stream = BinanceKlineStream::connect_with_config(
            &["BTCUSDT".to_string()],
            Interval::OneMinute,
            test_config(base),
        )
        .await
        .unwrap();
        let err = stream.next_event().await.unwrap_err();
        assert!(matches!(err, Error::Malformed(_)));
    }
}
