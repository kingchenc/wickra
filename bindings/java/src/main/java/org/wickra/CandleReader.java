// Generated from bindings/c/include/wickra.h. Do not edit by hand.
package org.wickra;

import org.wickra.internal.NativeMethods;
import org.wickra.internal.WickraNative;
import java.lang.foreign.Arena;
import java.lang.foreign.MemorySegment;
import java.lang.ref.Cleaner;
import java.lang.ref.Reference;
import static java.lang.foreign.ValueLayout.*;

/** CSV candle reader over the Wickra C ABI. Not thread-safe; close when done. */
public final class CandleReader implements AutoCloseable {
    private final MemorySegment handle;
    private final Cleaner.Cleanable cleanable;
    private boolean closed;

    /** Parse a timestamp,open,high,low,close,volume CSV string (a leading
     *  UTF-8 BOM and field whitespace are tolerated). */
    public CandleReader(String csv) {
        if (csv == null) {
            throw new NullPointerException("csv");
        }
        byte[] bytes = csv.getBytes(java.nio.charset.StandardCharsets.UTF_8);
        MemorySegment h;
        try (Arena a = Arena.ofConfined()) {
            MemorySegment data = a.allocate(Math.max(1L, bytes.length));
            MemorySegment.copy(bytes, 0, data, JAVA_BYTE, 0L, bytes.length);
            h = (MemorySegment) NativeMethods.WICKRA_CANDLE_READER_NEW.invokeExact(data, (long) bytes.length);
        } catch (Throwable t) {
            throw WickraNative.rethrow(t);
        }
        if (h.address() == 0L) {
            throw new IllegalArgumentException("invalid CandleReader CSV");
        }
        this.handle = h;
        this.cleanable = WickraNative.register(this, h, NativeMethods.WICKRA_CANDLE_READER_FREE);
    }

    /** Every candle parsed from the CSV, in file order. */
    public Candle[] read() {
        try {
            long n = (long) NativeMethods.WICKRA_CANDLE_READER_COUNT.invokeExact(handle());
            if (n <= 0) {
                return new Candle[0];
            }
            try (Arena a = Arena.ofConfined()) {
                MemorySegment out = a.allocate(48L * n);
                long w = (long) NativeMethods.WICKRA_CANDLE_READER_READ.invokeExact(handle(), out, n);
                Candle[] result = new Candle[(int) w];
                for (int i = 0; i < w; i++) {
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
            }
        } catch (Throwable t) {
            throw WickraNative.rethrow(t);
        } finally {
            Reference.reachabilityFence(this);
        }
    }

    /** The native handle, refusing to hand out one that has been released. */
    private MemorySegment handle() {
        if (closed) {
            throw new IllegalStateException("CandleReader has been closed");
        }
        return handle;
    }

    @Override public void close() {
        if (closed) {
            return;
        }
        closed = true;
        cleanable.clean();
    }
}
