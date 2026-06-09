// Generated from bindings/c/include/wickra.h. Do not edit by hand.
package org.wickra;

import org.wickra.internal.NativeMethods;
import org.wickra.internal.WickraNative;
import java.lang.foreign.Arena;
import java.lang.foreign.MemorySegment;
import java.lang.ref.Cleaner;
import static java.lang.foreign.ValueLayout.*;

/** Streaming Ichimoku indicator over the Wickra C ABI. Not thread-safe; close when done. */
public final class Ichimoku implements AutoCloseable {
    private final MemorySegment handle;
    private final Cleaner.Cleanable cleanable;

    public Ichimoku(int tenkanPeriod, int kijunPeriod, int senkouBPeriod, int displacement) {
        if (tenkanPeriod < 0) {
            throw new IllegalArgumentException("tenkanPeriod must be non-negative");
        }
        if (kijunPeriod < 0) {
            throw new IllegalArgumentException("kijunPeriod must be non-negative");
        }
        if (senkouBPeriod < 0) {
            throw new IllegalArgumentException("senkouBPeriod must be non-negative");
        }
        if (displacement < 0) {
            throw new IllegalArgumentException("displacement must be non-negative");
        }
        MemorySegment h;
        try {
            h = (MemorySegment) NativeMethods.WICKRA_ICHIMOKU_NEW.invokeExact((long) tenkanPeriod, (long) kijunPeriod, (long) senkouBPeriod, (long) displacement);
        } catch (Throwable t) {
            throw WickraNative.rethrow(t);
        }
        if (h.address() == 0L) {
            throw new IllegalArgumentException("invalid Ichimoku parameters");
        }
        this.handle = h;
        this.cleanable = WickraNative.register(this, h, NativeMethods.WICKRA_ICHIMOKU_FREE);
    }

    /** Push one observation; returns the result, or null during warmup. */
    public IchimokuOutput update(double open, double high, double low, double close, double volume, long timestamp) {
        try (Arena a = Arena.ofConfined()) {
            MemorySegment out = a.allocate(40L);
            byte ok = (byte) NativeMethods.WICKRA_ICHIMOKU_UPDATE.invokeExact(handle, open, high, low, close, volume, timestamp, out);
            if (ok == 0) {
                return null;
            }
            return new IchimokuOutput(
                out.get(JAVA_DOUBLE, 0L),
                out.get(JAVA_DOUBLE, 8L),
                out.get(JAVA_DOUBLE, 16L),
                out.get(JAVA_DOUBLE, 24L),
                out.get(JAVA_DOUBLE, 32L));
        } catch (Throwable t) {
            throw WickraNative.rethrow(t);
        }
    }

    /** Reset to the just-constructed state. */
    public void reset() {
        try {
            NativeMethods.WICKRA_ICHIMOKU_RESET.invokeExact(handle);
        } catch (Throwable t) {
            throw WickraNative.rethrow(t);
        }
    }

    @Override public void close() {
        cleanable.clean();
    }
}
