// Generated from bindings/c/include/wickra.h. Do not edit by hand.
package org.wickra;

import org.wickra.internal.NativeMethods;
import org.wickra.internal.WickraNative;
import java.lang.foreign.Arena;
import java.lang.foreign.MemorySegment;
import java.lang.ref.Cleaner;
import static java.lang.foreign.ValueLayout.*;

/** Streaming KalmanHedgeRatio indicator over the Wickra C ABI. Not thread-safe; close when done. */
public final class KalmanHedgeRatio implements AutoCloseable {
    private final MemorySegment handle;
    private final Cleaner.Cleanable cleanable;

    public KalmanHedgeRatio(double delta, double observationVar) {
        MemorySegment h;
        try {
            h = (MemorySegment) NativeMethods.WICKRA_KALMAN_HEDGE_RATIO_NEW.invokeExact(delta, observationVar);
        } catch (Throwable t) {
            throw WickraNative.rethrow(t);
        }
        if (h.address() == 0L) {
            throw new IllegalArgumentException("invalid KalmanHedgeRatio parameters");
        }
        this.handle = h;
        this.cleanable = WickraNative.register(this, h, NativeMethods.WICKRA_KALMAN_HEDGE_RATIO_FREE);
    }

    /** Push one observation; returns the result, or null during warmup. */
    public KalmanHedgeRatioOutput update(double x, double y) {
        try (Arena a = Arena.ofConfined()) {
            MemorySegment out = a.allocate(24L);
            byte ok = (byte) NativeMethods.WICKRA_KALMAN_HEDGE_RATIO_UPDATE.invokeExact(handle, x, y, out);
            if (ok == 0) {
                return null;
            }
            return new KalmanHedgeRatioOutput(
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
            NativeMethods.WICKRA_KALMAN_HEDGE_RATIO_RESET.invokeExact(handle);
        } catch (Throwable t) {
            throw WickraNative.rethrow(t);
        }
    }

    @Override public void close() {
        cleanable.clean();
    }
}
