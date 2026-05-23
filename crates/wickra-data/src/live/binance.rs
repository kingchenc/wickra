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
    OneWeek,
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
            Self::OneWeek => "1w",
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
                    if self.socket.send(Message::Pong(payload)).await.is_err() {
                        self.reconnect().await?;
                    }
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
            (Interval::OneWeek, "1w"),
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
}
