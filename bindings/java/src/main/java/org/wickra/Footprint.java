// Generated from bindings/c/include/wickra.h. Do not edit by hand.
package org.wickra;

import org.wickra.internal.NativeMethods;
import org.wickra.internal.WickraNative;
import java.lang.foreign.Arena;
import java.lang.foreign.MemorySegment;
import java.lang.ref.Cleaner;
import static java.lang.foreign.ValueLayout.*;

/** Streaming Footprint indicator over the Wickra C ABI. Not thread-safe; close when done. */
public final class Footprint implements AutoCloseable {
    private final MemorySegment handle;
    private final Cleaner.Cleanable cleanable;

    public Footprint(double tickSize) {
        MemorySegment h;
        try {
            h = (MemorySegment) NativeMethods.WICKRA_FOOTPRINT_NEW.invokeExact(tickSize);
        } catch (Throwable t) {
            throw WickraNative.rethrow(t);
        }
        if (h.address() == 0L) {
            throw new IllegalArgumentException("invalid Footprint parameters");
        }
        this.handle = h;
        this.cleanable = WickraNative.register(this, h, NativeMethods.WICKRA_FOOTPRINT_FREE);
    }

    /** Push one observation; returns the bars completed by it (possibly empty). */
    public FootprintLevel[] update(double price, double size, boolean isBuy, long timestamp) {
        final long cap = 64L;
        try (Arena a = Arena.ofConfined()) {
            MemorySegment out = a.allocate(24L * cap);
            long n = (long) NativeMethods.WICKRA_FOOTPRINT_UPDATE.invokeExact(handle, price, size, (byte) (isBuy ? 1 : 0), timestamp, out, cap);
            if (n <= 0) {
                return new FootprintLevel[0];
            }
            FootprintLevel[] result = new FootprintLevel[(int) n];
            for (int i = 0; i < n; i++) {
                long b = (long) i * 24L;
                result[i] = new FootprintLevel(
                    out.get(JAVA_DOUBLE, b + 0L),
                    out.get(JAVA_DOUBLE, b + 8L),
                    out.get(JAVA_DOUBLE, b + 16L));
            }
            return result;
        } catch (Throwable t) {
            throw WickraNative.rethrow(t);
        }
    }

    /** Reset to the just-constructed state. */
    public void reset() {
        try {
            NativeMethods.WICKRA_FOOTPRINT_RESET.invokeExact(handle);
        } catch (Throwable t) {
            throw WickraNative.rethrow(t);
        }
    }

    @Override public void close() {
        cleanable.clean();
    }
}
