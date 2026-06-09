// Generated from bindings/c/include/wickra.h. Do not edit by hand.
package org.wickra;

import org.wickra.internal.NativeMethods;
import org.wickra.internal.WickraNative;
import java.lang.foreign.Arena;
import java.lang.foreign.MemorySegment;
import java.lang.ref.Cleaner;
import static java.lang.foreign.ValueLayout.*;

/** Streaming TimeOfDayReturnProfile indicator over the Wickra C ABI. Not thread-safe; close when done. */
public final class TimeOfDayReturnProfile implements AutoCloseable {
    private final MemorySegment handle;
    private final Cleaner.Cleanable cleanable;
    private final int valuesCapacity;

    public TimeOfDayReturnProfile(int buckets, int utcOffsetMinutes) {
        if (buckets < 0) {
            throw new IllegalArgumentException("buckets must be non-negative");
        }
        MemorySegment h;
        try {
            h = (MemorySegment) NativeMethods.WICKRA_TIME_OF_DAY_RETURN_PROFILE_NEW.invokeExact((long) buckets, utcOffsetMinutes);
        } catch (Throwable t) {
            throw WickraNative.rethrow(t);
        }
        if (h.address() == 0L) {
            throw new IllegalArgumentException("invalid TimeOfDayReturnProfile parameters");
        }
        this.handle = h;
        this.cleanable = WickraNative.register(this, h, NativeMethods.WICKRA_TIME_OF_DAY_RETURN_PROFILE_FREE);
        this.valuesCapacity = buckets;
    }

    /** Push one observation; returns the profile values, or null during warmup. */
    public double[] update(double open, double high, double low, double close, double volume, long timestamp) {
        long cap = valuesCapacity;
        try (Arena a = Arena.ofConfined()) {
            MemorySegment values = a.allocate(JAVA_DOUBLE.byteSize() * cap);
            long len = (long) NativeMethods.WICKRA_TIME_OF_DAY_RETURN_PROFILE_UPDATE.invokeExact(handle, open, high, low, close, volume, timestamp, values, cap);
            if (len < 0) {
                return null;
            }
            int count = (int) Math.min(len, cap);
            double[] v = new double[count];
            MemorySegment.copy(values, JAVA_DOUBLE, 0L, v, 0, count);
            return v;
        } catch (Throwable t) {
            throw WickraNative.rethrow(t);
        }
    }

    /** Reset to the just-constructed state. */
    public void reset() {
        try {
            NativeMethods.WICKRA_TIME_OF_DAY_RETURN_PROFILE_RESET.invokeExact(handle);
        } catch (Throwable t) {
            throw WickraNative.rethrow(t);
        }
    }

    @Override public void close() {
        cleanable.clean();
    }
}
