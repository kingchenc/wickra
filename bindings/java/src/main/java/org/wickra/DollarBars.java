// Generated from bindings/c/include/wickra.h. Do not edit by hand.
package org.wickra;

import org.wickra.internal.NativeMethods;
import org.wickra.internal.WickraNative;
import java.lang.foreign.Arena;
import java.lang.foreign.MemorySegment;
import java.lang.ref.Cleaner;
import static java.lang.foreign.ValueLayout.*;

/** Streaming DollarBars indicator over the Wickra C ABI. Not thread-safe; close when done. */
public final class DollarBars implements AutoCloseable {
    private final MemorySegment handle;
    private final Cleaner.Cleanable cleanable;

    public DollarBars(double dollarPerBar) {
        MemorySegment h;
        try {
            h = (MemorySegment) NativeMethods.WICKRA_DOLLAR_BARS_NEW.invokeExact(dollarPerBar);
        } catch (Throwable t) {
            throw WickraNative.rethrow(t);
        }
        if (h.address() == 0L) {
            throw new IllegalArgumentException("invalid DollarBars parameters");
        }
        this.handle = h;
        this.cleanable = WickraNative.register(this, h, NativeMethods.WICKRA_DOLLAR_BARS_FREE);
    }

    /** Push one observation; returns the bars completed by it (possibly empty). */
    public DollarBar[] update(double open, double high, double low, double close, double volume, long timestamp) {
        final long cap = 64L;
        try (Arena a = Arena.ofConfined()) {
            MemorySegment out = a.allocate(48L * cap);
            long n = (long) NativeMethods.WICKRA_DOLLAR_BARS_UPDATE.invokeExact(handle, open, high, low, close, volume, timestamp, out, cap);
            if (n <= 0) {
                return new DollarBar[0];
            }
            DollarBar[] result = new DollarBar[(int) n];
            for (int i = 0; i < n; i++) {
                long b = (long) i * 48L;
                result[i] = new DollarBar(
                    out.get(JAVA_DOUBLE, b + 0L),
                    out.get(JAVA_DOUBLE, b + 8L),
                    out.get(JAVA_DOUBLE, b + 16L),
                    out.get(JAVA_DOUBLE, b + 24L),
                    out.get(JAVA_DOUBLE, b + 32L),
                    out.get(JAVA_DOUBLE, b + 40L));
            }
            return result;
        } catch (Throwable t) {
            throw WickraNative.rethrow(t);
        }
    }

    /** Reset to the just-constructed state. */
    public void reset() {
        try {
            NativeMethods.WICKRA_DOLLAR_BARS_RESET.invokeExact(handle);
        } catch (Throwable t) {
            throw WickraNative.rethrow(t);
        }
    }

    @Override public void close() {
        cleanable.clean();
    }
}
