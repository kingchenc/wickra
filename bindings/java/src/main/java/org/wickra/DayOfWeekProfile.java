// Generated from bindings/c/include/wickra.h. Do not edit by hand.
package org.wickra;

import org.wickra.internal.NativeMethods;
import org.wickra.internal.WickraNative;
import java.lang.foreign.Arena;
import java.lang.foreign.MemorySegment;
import java.lang.ref.Cleaner;
import java.lang.ref.Reference;
import static java.lang.foreign.ValueLayout.*;

/** Streaming DayOfWeekProfile indicator over the Wickra C ABI. Not thread-safe; close when done. */
public final class DayOfWeekProfile implements AutoCloseable {
    private final MemorySegment handle;
    private final Cleaner.Cleanable cleanable;
    private boolean closed;
    private final int valuesCapacity;

    public DayOfWeekProfile(int utcOffsetMinutes) {
        MemorySegment h;
        try {
            h = (MemorySegment) NativeMethods.WICKRA_DAY_OF_WEEK_PROFILE_NEW.invokeExact(utcOffsetMinutes);
        } catch (Throwable t) {
            throw WickraNative.rethrow(t);
        }
        if (h.address() == 0L) {
            throw new IllegalArgumentException("invalid DayOfWeekProfile parameters");
        }
        this.handle = h;
        this.cleanable = WickraNative.register(this, h, NativeMethods.WICKRA_DAY_OF_WEEK_PROFILE_FREE);
        this.valuesCapacity = 4096;
    }

    /** Push one observation; returns the profile values, or null during warmup. */
    public double[] update(double open, double high, double low, double close, double volume, long timestamp) {
        long cap = valuesCapacity;
        try (Arena a = Arena.ofConfined()) {
            MemorySegment values = a.allocate(JAVA_DOUBLE.byteSize() * cap);
            long len = (long) NativeMethods.WICKRA_DAY_OF_WEEK_PROFILE_UPDATE.invokeExact(handle(), open, high, low, close, volume, timestamp, values, cap);
            if (len < 0) {
                return null;
            }
            int count = (int) Math.min(len, cap);
            double[] v = new double[count];
            MemorySegment.copy(values, JAVA_DOUBLE, 0L, v, 0, count);
            return v;
        } catch (Throwable t) {
            throw WickraNative.rethrow(t);
        } finally {
            Reference.reachabilityFence(this);
        }
    }

    /** Number of updates required before update() yields a value. */
    public int warmupPeriod() {
        try {
            long n = (long) NativeMethods.WICKRA_DAY_OF_WEEK_PROFILE_WARMUP_PERIOD.invokeExact(handle());
            return (int) n;
        } catch (Throwable t) {
            throw WickraNative.rethrow(t);
        } finally {
            Reference.reachabilityFence(this);
        }
    }

    /** Whether the indicator has consumed enough input to emit a value. */
    public boolean isReady() {
        try {
            byte r = (byte) NativeMethods.WICKRA_DAY_OF_WEEK_PROFILE_IS_READY.invokeExact(handle());
            return r != 0;
        } catch (Throwable t) {
            throw WickraNative.rethrow(t);
        } finally {
            Reference.reachabilityFence(this);
        }
    }

    /** The indicator's canonical name. */
    public String name() {
        try {
            MemorySegment s = (MemorySegment) NativeMethods.WICKRA_DAY_OF_WEEK_PROFILE_NAME.invokeExact(handle());
            return s.address() == 0 ? "" : s.reinterpret(Long.MAX_VALUE).getString(0);
        } catch (Throwable t) {
            throw WickraNative.rethrow(t);
        } finally {
            Reference.reachabilityFence(this);
        }
    }

    /** Reset to the just-constructed state. */
    public void reset() {
        try {
            NativeMethods.WICKRA_DAY_OF_WEEK_PROFILE_RESET.invokeExact(handle());
        } catch (Throwable t) {
            throw WickraNative.rethrow(t);
        } finally {
            Reference.reachabilityFence(this);
        }
    }

    /** The native handle, refusing to hand out one that has been released. */
    private MemorySegment handle() {
        if (closed) {
            throw new IllegalStateException("DayOfWeekProfile has been closed");
        }
        return handle;
    }

    @Override public void close() {
        if (closed) {
            return;
        }
        closed = true;
        cleanable.clean();
    }
}
