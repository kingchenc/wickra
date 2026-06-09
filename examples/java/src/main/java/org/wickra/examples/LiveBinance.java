package org.wickra.examples;

import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.ObjectMapper;
import org.wickra.Ema;

import java.net.URI;
import java.net.http.HttpClient;
import java.net.http.WebSocket;
import java.util.concurrent.CountDownLatch;
import java.util.concurrent.TimeUnit;

/**
 * Stream live BTCUSDT 1-minute klines from Binance and feed each close through EMA(20).
 * Requires network access (build-only in CI). Runs for up to 60 seconds.
 */
public final class LiveBinance {
    public static void main(String[] args) throws Exception {
        URI uri = URI.create("wss://stream.binance.com:9443/ws/btcusdt@kline_1m");
        System.out.println("Connecting to " + uri + " (up to 60s)...");

        ObjectMapper mapper = new ObjectMapper();
        CountDownLatch done = new CountDownLatch(1);

        try (Ema ema = new Ema(20)) {
            WebSocket.Listener listener = new WebSocket.Listener() {
                private final StringBuilder buffer = new StringBuilder();

                @Override
                public void onOpen(WebSocket webSocket) {
                    webSocket.request(1);
                }

                @Override
                public java.util.concurrent.CompletionStage<?> onText(WebSocket webSocket, CharSequence data, boolean last) {
                    buffer.append(data);
                    if (last) {
                        try {
                            JsonNode root = mapper.readTree(buffer.toString());
                            JsonNode k = root.get("k");
                            if (k != null) {
                                double close = Double.parseDouble(k.get("c").asText());
                                double value = ema.update(close);
                                System.out.printf("close=%.2f  EMA(20)=%.2f%n", close, value);
                            }
                        } catch (Exception e) {
                            System.err.println("parse error: " + e.getMessage());
                        }
                        buffer.setLength(0);
                    }
                    webSocket.request(1);
                    return null;
                }

                @Override
                public void onError(WebSocket webSocket, Throwable error) {
                    System.err.println("websocket error: " + error.getMessage());
                    done.countDown();
                }
            };

            WebSocket ws = HttpClient.newHttpClient()
                    .newWebSocketBuilder()
                    .buildAsync(uri, listener)
                    .join();

            if (!done.await(60, TimeUnit.SECONDS)) {
                System.out.println("Done (time limit reached).");
            }
            ws.sendClose(WebSocket.NORMAL_CLOSURE, "done");
        }
    }
}
