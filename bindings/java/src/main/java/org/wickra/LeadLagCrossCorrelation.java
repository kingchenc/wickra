// Generated from bindings/c/include/wickra.h. Do not edit by hand.
package org.wickra;

import org.wickra.internal.NativeMethods;
import org.wickra.internal.WickraNative;
import java.lang.foreign.Arena;
import java.lang.foreign.MemorySegment;
import java.lang.ref.Cleaner;
import static java.lang.foreign.ValueLayout.*;

/** Streaming LeadLagCrossCorrelation indicator over the Wickra C ABI. Not thread-safe; close when done. */
public final class LeadLagCrossCorrelation implements AutoCloseable {
    private final MemorySegment handle;
    private final Cleaner.Cleanable cleanable;

    public LeadLagCrossCorrelation(int window, int maxLag) {
        if (window < 0) {
            throw new IllegalArgumentException("window must be non-negative");
        }
        if (maxLag < 0) {
            throw new IllegalArgumentException("maxLag must be non-negative");
        }
        MemorySegment h;
        try {
            h = (MemorySegment) NativeMethods.WICKRA_LEAD_LAG_CROSS_CORRELATION_NEW.invokeExact((long) window, (long) maxLag);
        } catch (Throwable t) {
            throw WickraNative.rethrow(t);
        }
        if (h.address() == 0L) {
            throw new IllegalArgumentException("invalid LeadLagCrossCorrelation parameters");
        }
        this.handle = h;
        this.cleanable = WickraNative.register(this, h, NativeMethods.WICKRA_LEAD_LAG_CROSS_CORRELATION_FREE);
    }

    /** Push one observation; returns the result, or null during warmup. */
    public LeadLagCrossCorrelationOutput update(double x, double y) {
        try (Arena a = Arena.ofConfined()) {
            MemorySegment out = a.allocate(16L);
            byte ok = (byte) NativeMethods.WICKRA_LEAD_LAG_CROSS_CORRELATION_UPDATE.invokeExact(handle, x, y, out);
            if (ok == 0) {
                return null;
            }
            return new LeadLagCrossCorrelationOutput(
                (double) out.get(JAVA_LONG, 0L),
                out.get(JAVA_DOUBLE, 8L));
        } catch (Throwable t) {
            throw WickraNative.rethrow(t);
        }
    }

    /** Number of updates required before update() yields a value. */
    public int warmupPeriod() {
        try {
            long n = (long) NativeMethods.WICKRA_LEAD_LAG_CROSS_CORRELATION_WARMUP_PERIOD.invokeExact(handle);
            return (int) n;
        } catch (Throwable t) {
            throw WickraNative.rethrow(t);
        }
    }

    /** Whether the indicator has consumed enough input to emit a value. */
    public boolean isReady() {
        try {
            byte r = (byte) NativeMethods.WICKRA_LEAD_LAG_CROSS_CORRELATION_IS_READY.invokeExact(handle);
            return r != 0;
        } catch (Throwable t) {
            throw WickraNative.rethrow(t);
        }
    }

    /** Reset to the just-constructed state. */
    public void reset() {
        try {
            NativeMethods.WICKRA_LEAD_LAG_CROSS_CORRELATION_RESET.invokeExact(handle);
        } catch (Throwable t) {
            throw WickraNative.rethrow(t);
        }
    }

    @Override public void close() {
        cleanable.clean();
    }
}
