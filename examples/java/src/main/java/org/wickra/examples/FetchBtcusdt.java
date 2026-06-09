package org.wickra.examples;

import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.ObjectMapper;

import java.io.BufferedWriter;
import java.net.URI;
import java.net.http.HttpClient;
import java.net.http.HttpRequest;
import java.net.http.HttpResponse;
import java.nio.file.Files;
import java.nio.file.Path;

/**
 * Download real BTCUSDT hourly klines from the Binance REST API into a CSV that the
 * other examples can consume. Requires network access (build-only in CI).
 */
public final class FetchBtcusdt {
    public static void main(String[] args) throws Exception {
        String url = "https://api.binance.com/api/v3/klines?symbol=BTCUSDT&interval=1h&limit=500";
        System.out.println("Fetching " + url);

        HttpClient http = HttpClient.newHttpClient();
        HttpRequest request = HttpRequest.newBuilder(URI.create(url)).GET().build();
        HttpResponse<String> response = http.send(request, HttpResponse.BodyHandlers.ofString());

        JsonNode klines = new ObjectMapper().readTree(response.body());
        Path dir = Path.of("data");
        Files.createDirectories(dir);
        Path path = dir.resolve("btcusdt_1h.csv");

        int count = 0;
        try (BufferedWriter writer = Files.newBufferedWriter(path)) {
            writer.write("timestamp,open,high,low,close,volume");
            writer.newLine();
            for (JsonNode kline : klines) {
                // Binance kline array: [openTime, open, high, low, close, volume, ...]
                writer.write(kline.get(0).asLong() + "," + kline.get(1).asText() + ","
                        + kline.get(2).asText() + "," + kline.get(3).asText() + ","
                        + kline.get(4).asText() + "," + kline.get(5).asText());
                writer.newLine();
                count++;
            }
        }

        System.out.printf("Wrote %d klines to %s%n", count, path.toAbsolutePath());
    }
}
