# Stream live BTCUSDT 1-minute klines from Binance and feed each close through EMA(20).
# Requires network access and the 'websocket' + 'jsonlite' packages. Runs ~60s.
library(wickra)
for (p in c("websocket", "jsonlite", "later")) {
  if (!requireNamespace(p, quietly = TRUE)) stop(sprintf("install '%s' to run this example", p))
}
ema <- Ema(20)
ws <- websocket::WebSocket$new("wss://stream.binance.com:9443/ws/btcusdt@kline_1m")
ws$onOpen(function(event) cat("Connected; streaming for up to 60s...\n"))
ws$onMessage(function(event) {
  msg <- jsonlite::fromJSON(event$data)
  close <- as.numeric(msg$k$c)
  cat(sprintf("close=%.2f  EMA(20)=%.2f\n", close, update(ema, close)))
})
ws$connect()
later::later(function() ws$close(), 60)
while (ws$readyState() <= 1L) later::run_now(timeout = 1)
