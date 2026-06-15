// Generated from bindings/c/include/wickra.h. Do not edit by hand.
package org.wickra;

import org.wickra.internal.NativeMethods;
import org.wickra.internal.WickraNative;
import java.lang.foreign.Arena;
import java.lang.foreign.MemorySegment;
import java.lang.ref.Cleaner;
import static java.lang.foreign.ValueLayout.*;

/** Streaming Resampler indicator over the Wickra C ABI. Not thread-safe; close when done. */
public final class Resampler implements AutoCloseable {
    private final MemorySegment handle;
    private final Cleaner.Cleanable cleanable;

    public Resampler(long timeframe) {
        MemorySegment h;
        try {
            h = (MemorySegment) NativeMethods.WICKRA_RESAMPLER_NEW.invokeExact(timeframe);
        } catch (Throwable t) {
            throw WickraNative.rethrow(t);
        }
        if (h.address() == 0L) {
            throw new IllegalArgumentException("invalid Resampler parameters");
        }
        this.handle = h;
        this.cleanable = WickraNative.register(this, h, NativeMethods.WICKRA_RESAMPLER_FREE);
    }

    /** Push one observation; returns the result, or null during warmup. */
    public Candle update(double open, double high, double low, double close, double volume, long timestamp) {
        try (Arena a = Arena.ofConfined()) {
            MemorySegment out = a.allocate(48L);
            byte ok = (byte) NativeMethods.WICKRA_RESAMPLER_UPDATE.invokeExact(handle, open, high, low, close, volume, timestamp, out);
            if (ok == 0) {
                return null;
            }
            return new Candle(
                out.get(JAVA_DOUBLE, 0L),
                out.get(JAVA_DOUBLE, 8L),
                out.get(JAVA_DOUBLE, 16L),
                out.get(JAVA_DOUBLE, 24L),
                out.get(JAVA_DOUBLE, 32L),
                (double) out.get(JAVA_LONG, 40L));
        } catch (Throwable t) {
            throw WickraNative.rethrow(t);
        }
    }

    /** Emit the final, still-open candle (null if none is pending). */
    public Candle flush() {
        try (Arena a = Arena.ofConfined()) {
            MemorySegment out = a.allocate(48L);
            byte ok = (byte) NativeMethods.WICKRA_RESAMPLER_FLUSH.invokeExact(handle, out);
            if (ok == 0) {
                return null;
            }
            return new Candle(
                out.get(JAVA_DOUBLE, 0L),
                out.get(JAVA_DOUBLE, 8L),
                out.get(JAVA_DOUBLE, 16L),
                out.get(JAVA_DOUBLE, 24L),
                out.get(JAVA_DOUBLE, 32L),
                (double) out.get(JAVA_LONG, 40L));
        } catch (Throwable t) {
            throw WickraNative.rethrow(t);
        }
    }

    @Override public void close() {
        cleanable.clean();
    }
}
