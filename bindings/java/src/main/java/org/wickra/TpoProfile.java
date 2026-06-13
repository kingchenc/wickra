// Generated from bindings/c/include/wickra.h. Do not edit by hand.
package org.wickra;

import org.wickra.internal.NativeMethods;
import org.wickra.internal.WickraNative;
import java.lang.foreign.Arena;
import java.lang.foreign.MemorySegment;
import java.lang.ref.Cleaner;
import static java.lang.foreign.ValueLayout.*;

/** Streaming TpoProfile indicator over the Wickra C ABI. Not thread-safe; close when done. */
public final class TpoProfile implements AutoCloseable {
    private final MemorySegment handle;
    private final Cleaner.Cleanable cleanable;
    private final int valuesCapacity;

    public TpoProfile(int period, int binCount) {
        if (period < 0) {
            throw new IllegalArgumentException("period must be non-negative");
        }
        if (binCount < 0) {
            throw new IllegalArgumentException("binCount must be non-negative");
        }
        MemorySegment h;
        try {
            h = (MemorySegment) NativeMethods.WICKRA_TPO_PROFILE_NEW.invokeExact((long) period, (long) binCount);
        } catch (Throwable t) {
            throw WickraNative.rethrow(t);
        }
        if (h.address() == 0L) {
            throw new IllegalArgumentException("invalid TpoProfile parameters");
        }
        this.handle = h;
        this.cleanable = WickraNative.register(this, h, NativeMethods.WICKRA_TPO_PROFILE_FREE);
        this.valuesCapacity = binCount;
    }

    /** Push one observation; returns the profile, or null during warmup. */
    public TpoProfileOutputScalars update(double open, double high, double low, double close, double volume, long timestamp) {
        long cap = valuesCapacity;
        try (Arena a = Arena.ofConfined()) {
            MemorySegment scalars = a.allocate(16L);
            MemorySegment values = a.allocate(JAVA_DOUBLE.byteSize() * cap);
            long len = (long) NativeMethods.WICKRA_TPO_PROFILE_UPDATE.invokeExact(handle, open, high, low, close, volume, timestamp, scalars, values, cap);
            if (len < 0) {
                return null;
            }
            int count = (int) Math.min(len, cap);
            double[] v = new double[count];
            MemorySegment.copy(values, JAVA_DOUBLE, 0L, v, 0, count);
            return new TpoProfileOutputScalars(
                scalars.get(JAVA_DOUBLE, 0L),
                scalars.get(JAVA_DOUBLE, 8L),
                v);
        } catch (Throwable t) {
            throw WickraNative.rethrow(t);
        }
    }

    /** Number of updates required before update() yields a value. */
    public int warmupPeriod() {
        try {
            long n = (long) NativeMethods.WICKRA_TPO_PROFILE_WARMUP_PERIOD.invokeExact(handle);
            return (int) n;
        } catch (Throwable t) {
            throw WickraNative.rethrow(t);
        }
    }

    /** Whether the indicator has consumed enough input to emit a value. */
    public boolean isReady() {
        try {
            byte r = (byte) NativeMethods.WICKRA_TPO_PROFILE_IS_READY.invokeExact(handle);
            return r != 0;
        } catch (Throwable t) {
            throw WickraNative.rethrow(t);
        }
    }

    /** Reset to the just-constructed state. */
    public void reset() {
        try {
            NativeMethods.WICKRA_TPO_PROFILE_RESET.invokeExact(handle);
        } catch (Throwable t) {
            throw WickraNative.rethrow(t);
        }
    }

    @Override public void close() {
        cleanable.clean();
    }
}
