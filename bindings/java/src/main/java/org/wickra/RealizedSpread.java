// Generated from bindings/c/include/wickra.h. Do not edit by hand.
package org.wickra;

import org.wickra.internal.NativeMethods;
import org.wickra.internal.WickraNative;
import java.lang.foreign.Arena;
import java.lang.foreign.MemorySegment;
import java.lang.ref.Cleaner;
import static java.lang.foreign.ValueLayout.*;

/** Streaming RealizedSpread indicator over the Wickra C ABI. Not thread-safe; close when done. */
public final class RealizedSpread implements AutoCloseable {
    private final MemorySegment handle;
    private final Cleaner.Cleanable cleanable;

    public RealizedSpread(int horizon) {
        if (horizon < 0) {
            throw new IllegalArgumentException("horizon must be non-negative");
        }
        MemorySegment h;
        try {
            h = (MemorySegment) NativeMethods.WICKRA_REALIZED_SPREAD_NEW.invokeExact((long) horizon);
        } catch (Throwable t) {
            throw WickraNative.rethrow(t);
        }
        if (h.address() == 0L) {
            throw new IllegalArgumentException("invalid RealizedSpread parameters");
        }
        this.handle = h;
        this.cleanable = WickraNative.register(this, h, NativeMethods.WICKRA_REALIZED_SPREAD_FREE);
    }

    /** Push one observation; returns the indicator value (NaN during warmup). */
    public double update(double price, double size, boolean isBuy, long timestamp, double mid) {
        try {
            return (double) NativeMethods.WICKRA_REALIZED_SPREAD_UPDATE.invokeExact(handle, price, size, (byte) (isBuy ? 1 : 0), timestamp, mid);
        } catch (Throwable t) {
            throw WickraNative.rethrow(t);
        }
    }

    /** Reset to the just-constructed state. */
    public void reset() {
        try {
            NativeMethods.WICKRA_REALIZED_SPREAD_RESET.invokeExact(handle);
        } catch (Throwable t) {
            throw WickraNative.rethrow(t);
        }
    }

    @Override public void close() {
        cleanable.clean();
    }
}
