// Generated from bindings/c/include/wickra.h. Do not edit by hand.
package org.wickra;

import org.wickra.internal.NativeMethods;
import org.wickra.internal.WickraNative;
import java.lang.foreign.Arena;
import java.lang.foreign.MemorySegment;
import java.lang.ref.Cleaner;
import static java.lang.foreign.ValueLayout.*;

/** Streaming RenkoBars indicator over the Wickra C ABI. Not thread-safe; close when done. */
public final class RenkoBars implements AutoCloseable {
    private final MemorySegment handle;
    private final Cleaner.Cleanable cleanable;

    public RenkoBars(double boxSize) {
        MemorySegment h;
        try {
            h = (MemorySegment) NativeMethods.WICKRA_RENKO_BARS_NEW.invokeExact(boxSize);
        } catch (Throwable t) {
            throw WickraNative.rethrow(t);
        }
        if (h.address() == 0L) {
            throw new IllegalArgumentException("invalid RenkoBars parameters");
        }
        this.handle = h;
        this.cleanable = WickraNative.register(this, h, NativeMethods.WICKRA_RENKO_BARS_FREE);
    }

    /** Push one observation; returns the bars completed by it (possibly empty). */
    public RenkoBrick[] update(double open, double high, double low, double close, double volume, long timestamp) {
        final long cap = 64L;
        try (Arena a = Arena.ofConfined()) {
            MemorySegment out = a.allocate(24L * cap);
            long n = (long) NativeMethods.WICKRA_RENKO_BARS_UPDATE.invokeExact(handle, open, high, low, close, volume, timestamp, out, cap);
            if (n <= 0) {
                return new RenkoBrick[0];
            }
            RenkoBrick[] result = new RenkoBrick[(int) n];
            for (int i = 0; i < n; i++) {
                long b = (long) i * 24L;
                result[i] = new RenkoBrick(
                    out.get(JAVA_DOUBLE, b + 0L),
                    out.get(JAVA_DOUBLE, b + 8L),
                    (double) out.get(JAVA_BYTE, b + 16L));
            }
            return result;
        } catch (Throwable t) {
            throw WickraNative.rethrow(t);
        }
    }

    /** Reset to the just-constructed state. */
    public void reset() {
        try {
            NativeMethods.WICKRA_RENKO_BARS_RESET.invokeExact(handle);
        } catch (Throwable t) {
            throw WickraNative.rethrow(t);
        }
    }

    @Override public void close() {
        cleanable.clean();
    }
}
