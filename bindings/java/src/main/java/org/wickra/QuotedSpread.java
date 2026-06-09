// Generated from bindings/c/include/wickra.h. Do not edit by hand.
package org.wickra;

import org.wickra.internal.NativeMethods;
import org.wickra.internal.WickraNative;
import java.lang.foreign.Arena;
import java.lang.foreign.MemorySegment;
import java.lang.ref.Cleaner;
import static java.lang.foreign.ValueLayout.*;

/** Streaming QuotedSpread indicator over the Wickra C ABI. Not thread-safe; close when done. */
public final class QuotedSpread implements AutoCloseable {
    private final MemorySegment handle;
    private final Cleaner.Cleanable cleanable;

    public QuotedSpread() {
        MemorySegment h;
        try {
            h = (MemorySegment) NativeMethods.WICKRA_QUOTED_SPREAD_NEW.invokeExact();
        } catch (Throwable t) {
            throw WickraNative.rethrow(t);
        }
        if (h.address() == 0L) {
            throw new IllegalArgumentException("invalid QuotedSpread parameters");
        }
        this.handle = h;
        this.cleanable = WickraNative.register(this, h, NativeMethods.WICKRA_QUOTED_SPREAD_FREE);
    }

    /** Push one observation; returns the indicator value (NaN during warmup). */
    public double update(double[] bidPrice, double[] bidSize, double[] askPrice, double[] askSize) {
        if (bidSize.length != bidPrice.length) {
            throw new IllegalArgumentException("input arrays in the same group must have equal length");
        }
        if (askSize.length != askPrice.length) {
            throw new IllegalArgumentException("input arrays in the same group must have equal length");
        }
        try (Arena a = Arena.ofConfined()) {
            MemorySegment bidPriceSeg = a.allocateFrom(JAVA_DOUBLE, bidPrice);
            MemorySegment bidSizeSeg = a.allocateFrom(JAVA_DOUBLE, bidSize);
            MemorySegment askPriceSeg = a.allocateFrom(JAVA_DOUBLE, askPrice);
            MemorySegment askSizeSeg = a.allocateFrom(JAVA_DOUBLE, askSize);
            return (double) NativeMethods.WICKRA_QUOTED_SPREAD_UPDATE.invokeExact(handle, bidPriceSeg, bidSizeSeg, (long) bidPrice.length, askPriceSeg, askSizeSeg, (long) askPrice.length);
        } catch (Throwable t) {
            throw WickraNative.rethrow(t);
        }
    }

    /** Reset to the just-constructed state. */
    public void reset() {
        try {
            NativeMethods.WICKRA_QUOTED_SPREAD_RESET.invokeExact(handle);
        } catch (Throwable t) {
            throw WickraNative.rethrow(t);
        }
    }

    @Override public void close() {
        cleanable.clean();
    }
}
