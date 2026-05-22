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

use futures_util::SinkExt;
use futures_util::StreamExt;
use serde::Deserialize;
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::MaybeTlsStream;
use tokio_tungstenite::WebSocketStream;

use crate::error::{Error, Result};
use wickra_core::Candle;

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
    /// Interval requested at connect time. Used to tag every event.
    interval: Interval,
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
    /// Connect to Binance's combined-stream endpoint for one or more symbols.
    ///
    /// Symbols may be passed in either case; they are lowercased to match
    /// Binance's stream-name conventions.
    pub async fn connect(symbols: &[String], interval: Interval) -> Result<Self> {
        if symbols.is_empty() {
            return Err(Error::Malformed(
                "BinanceKlineStream requires at least one symbol".into(),
            ));
        }
        let streams: Vec<String> = symbols
            .iter()
            .map(|s| format!("{}@kline_{}", s.to_lowercase(), interval.as_str()))
            .collect();
        let url = format!(
            "wss://stream.binance.com:9443/stream?streams={}",
            streams.join("/")
        );
        let url = url::Url::parse(&url).map_err(|e| Error::Malformed(e.to_string()))?;
        let (socket, _) = tokio_tungstenite::connect_async(url.as_str()).await?;
        Ok(Self { socket, interval })
    }

    /// Receive the next kline event. Yields `Ok(None)` when the server closes
    /// the connection cleanly.
    pub async fn next_event(&mut self) -> Result<Option<KlineEvent>> {
        loop {
            let msg = match self.socket.next().await {
                Some(Ok(m)) => m,
                Some(Err(e)) => return Err(Error::from(e)),
                None => return Ok(None),
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
                    self.socket.send(Message::Pong(payload)).await?;
                }
                Message::Pong(_) | Message::Frame(_) => {}
                Message::Close(_) => return Ok(None),
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

    /// Close the underlying socket cleanly.
    pub async fn close(mut self) -> Result<()> {
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
