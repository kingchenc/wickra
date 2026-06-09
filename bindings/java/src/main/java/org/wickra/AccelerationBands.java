// Generated from bindings/c/include/wickra.h. Do not edit by hand.
package org.wickra;

import org.wickra.internal.NativeMethods;
import org.wickra.internal.WickraNative;
import java.lang.foreign.Arena;
import java.lang.foreign.MemorySegment;
import java.lang.ref.Cleaner;
import static java.lang.foreign.ValueLayout.*;

/** Streaming AccelerationBands indicator over the Wickra C ABI. Not thread-safe; close when done. */
public final class AccelerationBands implements AutoCloseable {
    private final MemorySegment handle;
    private final Cleaner.Cleanable cleanable;

    public AccelerationBands(int period, double factor) {
        if (period < 0) {
            throw new IllegalArgumentException("period must be non-negative");
        }
        MemorySegment h;
        try {
            h = (MemorySegment) NativeMethods.WICKRA_ACCELERATION_BANDS_NEW.invokeExact((long) period, factor);
        } catch (Throwable t) {
            throw WickraNative.rethrow(t);
        }
        if (h.address() == 0L) {
            throw new IllegalArgumentException("invalid AccelerationBands parameters");
        }
        this.handle = h;
        this.cleanable = WickraNative.register(this, h, NativeMethods.WICKRA_ACCELERATION_BANDS_FREE);
    }

    /** Push one observation; returns the result, or null during warmup. */
    public AccelerationBandsOutput update(double open, double high, double low, double close, double volume, long timestamp) {
        try (Arena a = Arena.ofConfined()) {
            MemorySegment out = a.allocate(24L);
            byte ok = (byte) NativeMethods.WICKRA_ACCELERATION_BANDS_UPDATE.invokeExact(handle, open, high, low, close, volume, timestamp, out);
            if (ok == 0) {
                return null;
            }
            return new AccelerationBandsOutput(
                out.get(JAVA_DOUBLE, 0L),
                out.get(JAVA_DOUBLE, 8L),
                out.get(JAVA_DOUBLE, 16L));
        } catch (Throwable t) {
            throw WickraNative.rethrow(t);
        }
    }

    /** Reset to the just-constructed state. */
    public void reset() {
        try {
            NativeMethods.WICKRA_ACCELERATION_BANDS_RESET.invokeExact(handle);
        } catch (Throwable t) {
            throw WickraNative.rethrow(t);
        }
    }

    @Override public void close() {
        cleanable.clean();
    }
}
