// Generated from bindings/c/include/wickra.h. Do not edit by hand.
package org.wickra;

import org.wickra.internal.NativeMethods;
import org.wickra.internal.WickraNative;
import java.lang.foreign.Arena;
import java.lang.foreign.MemorySegment;
import java.lang.ref.Cleaner;
import static java.lang.foreign.ValueLayout.*;

/** Streaming OrderBookImbalanceTopN indicator over the Wickra C ABI. Not thread-safe; close when done. */
public final class OrderBookImbalanceTopN implements AutoCloseable {
    private final MemorySegment handle;
    private final Cleaner.Cleanable cleanable;

    public OrderBookImbalanceTopN(int levels) {
        if (levels < 0) {
            throw new IllegalArgumentException("levels must be non-negative");
        }
        MemorySegment h;
        try {
            h = (MemorySegment) NativeMethods.WICKRA_ORDER_BOOK_IMBALANCE_TOP_N_NEW.invokeExact((long) levels);
        } catch (Throwable t) {
            throw WickraNative.rethrow(t);
        }
        if (h.address() == 0L) {
            throw new IllegalArgumentException("invalid OrderBookImbalanceTopN parameters");
        }
        this.handle = h;
        this.cleanable = WickraNative.register(this, h, NativeMethods.WICKRA_ORDER_BOOK_IMBALANCE_TOP_N_FREE);
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
            return (double) NativeMethods.WICKRA_ORDER_BOOK_IMBALANCE_TOP_N_UPDATE.invokeExact(handle, bidPriceSeg, bidSizeSeg, (long) bidPrice.length, askPriceSeg, askSizeSeg, (long) askPrice.length);
        } catch (Throwable t) {
            throw WickraNative.rethrow(t);
        }
    }

    /** Reset to the just-constructed state. */
    public void reset() {
        try {
            NativeMethods.WICKRA_ORDER_BOOK_IMBALANCE_TOP_N_RESET.invokeExact(handle);
        } catch (Throwable t) {
            throw WickraNative.rethrow(t);
        }
    }

    @Override public void close() {
        cleanable.clean();
    }
}
