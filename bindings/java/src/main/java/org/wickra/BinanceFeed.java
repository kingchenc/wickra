// Generated from bindings/c/include/wickra.h. Do not edit by hand.
package org.wickra;

import org.wickra.internal.NativeMethods;
import org.wickra.internal.WickraNative;
import java.lang.foreign.Arena;
import java.lang.foreign.MemorySegment;
import java.lang.ref.Cleaner;
import java.nio.charset.StandardCharsets;
import static java.lang.foreign.ValueLayout.*;

/** A live Binance kline stream over the Wickra C ABI. Not thread-safe; close when done. */
public final class BinanceFeed implements AutoCloseable {
    private final MemorySegment handle;
    private final Cleaner.Cleanable cleanable;

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
            int code = (int) NativeMethods.WICKRA_BINANCE_NEXT.invokeExact(handle, out, timeoutMillis);
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

    @Override public void close() {
        try {
            NativeMethods.WICKRA_BINANCE_CLOSE.invokeExact(handle);
        } catch (Throwable t) {
            throw WickraNative.rethrow(t);
        }
        cleanable.clean();
    }
}
