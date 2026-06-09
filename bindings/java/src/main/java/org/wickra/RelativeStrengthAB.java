// Generated from bindings/c/include/wickra.h. Do not edit by hand.
package org.wickra;

import org.wickra.internal.NativeMethods;
import org.wickra.internal.WickraNative;
import java.lang.foreign.Arena;
import java.lang.foreign.MemorySegment;
import java.lang.ref.Cleaner;
import static java.lang.foreign.ValueLayout.*;

/** Streaming RelativeStrengthAB indicator over the Wickra C ABI. Not thread-safe; close when done. */
public final class RelativeStrengthAB implements AutoCloseable {
    private final MemorySegment handle;
    private final Cleaner.Cleanable cleanable;

    public RelativeStrengthAB(int maPeriod, int rsiPeriod) {
        if (maPeriod < 0) {
            throw new IllegalArgumentException("maPeriod must be non-negative");
        }
        if (rsiPeriod < 0) {
            throw new IllegalArgumentException("rsiPeriod must be non-negative");
        }
        MemorySegment h;
        try {
            h = (MemorySegment) NativeMethods.WICKRA_RELATIVE_STRENGTH_AB_NEW.invokeExact((long) maPeriod, (long) rsiPeriod);
        } catch (Throwable t) {
            throw WickraNative.rethrow(t);
        }
        if (h.address() == 0L) {
            throw new IllegalArgumentException("invalid RelativeStrengthAB parameters");
        }
        this.handle = h;
        this.cleanable = WickraNative.register(this, h, NativeMethods.WICKRA_RELATIVE_STRENGTH_AB_FREE);
    }

    /** Push one observation; returns the result, or null during warmup. */
    public RelativeStrengthOutput update(double x, double y) {
        try (Arena a = Arena.ofConfined()) {
            MemorySegment out = a.allocate(24L);
            byte ok = (byte) NativeMethods.WICKRA_RELATIVE_STRENGTH_AB_UPDATE.invokeExact(handle, x, y, out);
            if (ok == 0) {
                return null;
            }
            return new RelativeStrengthOutput(
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
            NativeMethods.WICKRA_RELATIVE_STRENGTH_AB_RESET.invokeExact(handle);
        } catch (Throwable t) {
            throw WickraNative.rethrow(t);
        }
    }

    @Override public void close() {
        cleanable.clean();
    }
}
