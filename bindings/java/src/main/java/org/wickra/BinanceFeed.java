// Generated from bindings/c/include/wickra.h. Do not edit by hand.
package org.wickra;

import org.wickra.internal.NativeMethods;
import org.wickra.internal.WickraNative;
import java.lang.foreign.Arena;
import java.lang.foreign.MemorySegment;
import java.lang.ref.Cleaner;
import java.lang.ref.Reference;
import java.nio.charset.StandardCharsets;
import static java.lang.foreign.ValueLayout.*;

/** A live Binance kline stream over the Wickra C ABI. Not thread-safe; close when done. */
public final class BinanceFeed implements AutoCloseable {
    private final MemorySegment handle;
    private final Cleaner.Cleanable cleanable;
    private boolean closed;

    /** Connect to Binance's live kline stream for the given comma-separated
     *  symbols (case-insensitive) at interval. baseUrl overrides the endpoint
     *  (null = production; pass a ws:// URL to target a test server). */
    public BinanceFeed(String symbols, BinanceInterval interval, String baseUrl) {
        if (symbols == null) {
            throw new NullPointerException("symbols");
        }
        MemorySegment h;
        try (Arena a = Arena.ofConfined()) {
            byte[] sb = symbols.getBytes(StandardCharsets.UTF_8);
            MemorySegment sym = a.allocate(sb.length + 1L);
            MemorySegment.copy(sb, 0, sym, JAVA_BYTE, 0L, sb.length);
            MemorySegment url = MemorySegment.NULL;
            if (baseUrl != null) {
                byte[] ub = baseUrl.getBytes(StandardCharsets.UTF_8);
                url = a.allocate(ub.length + 1L);
                MemorySegment.copy(ub, 0, url, JAVA_BYTE, 0L, ub.length);
            }
            h = (MemorySegment) NativeMethods.WICKRA_BINANCE_CONNECT.invokeExact(
                sym, (byte) interval.ordinal(), url);
        } catch (Throwable t) {
            throw WickraNative.rethrow(t);
        }
        if (h.address() == 0L) {
            throw new IllegalArgumentException("invalid BinanceFeed parameters");
        }
        this.handle = h;
        this.cleanable = WickraNative.register(this, h, NativeMethods.WICKRA_BINANCE_FREE);
    }

    /** Connect with the production endpoint. */
    public BinanceFeed(String symbols, BinanceInterval interval) {
        this(symbols, interval, null);
    }

    /** Poll for the next kline event, waiting up to timeoutMillis. Returns the
     *  event, or null on timeout (call again). Throws once the stream is closed. */
    public KlineEvent next(long timeoutMillis) {
        try (Arena a = Arena.ofConfined()) {
            MemorySegment out = a.allocate(72L);
            int code = (int) NativeMethods.WICKRA_BINANCE_NEXT.invokeExact(handle(), out, timeoutMillis);
            if (code == 0) {
                return null;
            }
            if (code != 1) {
                throw new IllegalStateException("binance feed closed");
            }
            byte[] symBytes = new byte[16];
            MemorySegment.copy(out, JAVA_BYTE, 0L, symBytes, 0, 16);
            int n = 0;
            while (n < 16 && symBytes[n] != 0) {
                n++;
            }
            return new KlineEvent(
                new String(symBytes, 0, n, StandardCharsets.UTF_8),
                out.get(JAVA_DOUBLE, 16L),
                out.get(JAVA_DOUBLE, 24L),
                out.get(JAVA_DOUBLE, 32L),
                out.get(JAVA_DOUBLE, 40L),
                out.get(JAVA_DOUBLE, 48L),
                out.get(JAVA_LONG, 56L),
                out.get(JAVA_BYTE, 64L) != 0);
        } catch (Throwable t) {
            throw WickraNative.rethrow(t);
        }
    }

    /** Fetch historical klines from Binance's REST endpoint. symbol is the
     *  trading pair (case-insensitive), limit the number of candles (1..=1000).
     *  startMs/endMs are inclusive Unix-millisecond bounds (negative = unset);
     *  baseUrl overrides the host (null = production). Blocks until done. */
    public static Candle[] fetchKlines(String symbol, BinanceInterval interval, int limit,
                                       long startMs, long endMs, String baseUrl) {
        if (symbol == null) {
            throw new NullPointerException("symbol");
        }
        if (limit <= 0) {
            throw new IllegalArgumentException("limit must be in 1..=1000");
        }
        try (Arena a = Arena.ofConfined()) {
            byte[] sb = symbol.getBytes(StandardCharsets.UTF_8);
            MemorySegment sym = a.allocate(sb.length + 1L);
            MemorySegment.copy(sb, 0, sym, JAVA_BYTE, 0L, sb.length);
            MemorySegment url = MemorySegment.NULL;
            if (baseUrl != null) {
                byte[] ub = baseUrl.getBytes(StandardCharsets.UTF_8);
                url = a.allocate(ub.length + 1L);
                MemorySegment.copy(ub, 0, url, JAVA_BYTE, 0L, ub.length);
            }
            MemorySegment out = a.allocate(48L * limit);
            long n = (long) NativeMethods.WICKRA_BINANCE_FETCH_KLINES.invokeExact(
                sym, (byte) interval.ordinal(), limit, startMs, endMs, url, out, (long) limit);
            if (n < 0) {
                throw new IllegalArgumentException("invalid fetchKlines parameters or transport error");
            }
            Candle[] result = new Candle[(int) n];
            for (int i = 0; i < n; i++) {
                long b = (long) i * 48L;
                result[i] = new Candle(
                    out.get(JAVA_DOUBLE, b + 0L),
                    out.get(JAVA_DOUBLE, b + 8L),
                    out.get(JAVA_DOUBLE, b + 16L),
                    out.get(JAVA_DOUBLE, b + 24L),
                    out.get(JAVA_DOUBLE, b + 32L),
                    (double) out.get(JAVA_LONG, b + 40L));
            }
            return result;
        } catch (Throwable t) {
            throw WickraNative.rethrow(t);
        }
    }

    /** The native handle, refusing to hand out one that has been released. */
    private MemorySegment handle() {
        if (closed) {
            throw new IllegalStateException("BinanceFeed has been closed");
        }
        return handle;
    }

    @Override public void close() {
        if (closed) {
            return;
        }
        try {
            NativeMethods.WICKRA_BINANCE_CLOSE.invokeExact(handle());
        } catch (Throwable t) {
            throw WickraNative.rethrow(t);
        } finally {
            Reference.reachabilityFence(this);
        }
        closed = true;
        cleanable.clean();
    }
}
