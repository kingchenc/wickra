// Generated from bindings/c/include/wickra.h. Do not edit by hand.
package org.wickra;

import org.wickra.internal.NativeMethods;
import org.wickra.internal.WickraNative;
import java.lang.foreign.Arena;
import java.lang.foreign.MemorySegment;
import java.lang.ref.Cleaner;
import static java.lang.foreign.ValueLayout.*;

/** Streaming Alligator indicator over the Wickra C ABI. Not thread-safe; close when done. */
public final class Alligator implements AutoCloseable {
    private final MemorySegment handle;
    private final Cleaner.Cleanable cleanable;

    public Alligator(int jawPeriod, int teethPeriod, int lipsPeriod) {
        if (jawPeriod < 0) {
            throw new IllegalArgumentException("jawPeriod must be non-negative");
        }
        if (teethPeriod < 0) {
            throw new IllegalArgumentException("teethPeriod must be non-negative");
        }
        if (lipsPeriod < 0) {
            throw new IllegalArgumentException("lipsPeriod must be non-negative");
        }
        MemorySegment h;
        try {
            h = (MemorySegment) NativeMethods.WICKRA_ALLIGATOR_NEW.invokeExact((long) jawPeriod, (long) teethPeriod, (long) lipsPeriod);
        } catch (Throwable t) {
            throw WickraNative.rethrow(t);
        }
        if (h.address() == 0L) {
            throw new IllegalArgumentException("invalid Alligator parameters");
        }
        this.handle = h;
        this.cleanable = WickraNative.register(this, h, NativeMethods.WICKRA_ALLIGATOR_FREE);
    }

    /** Push one observation; returns the result, or null during warmup. */
    public AlligatorOutput update(double open, double high, double low, double close, double volume, long timestamp) {
        try (Arena a = Arena.ofConfined()) {
            MemorySegment out = a.allocate(24L);
            byte ok = (byte) NativeMethods.WICKRA_ALLIGATOR_UPDATE.invokeExact(handle, open, high, low, close, volume, timestamp, out);
            if (ok == 0) {
                return null;
            }
            return new AlligatorOutput(
                out.get(JAVA_DOUBLE, 0L),
                out.get(JAVA_DOUBLE, 8L),
                out.get(JAVA_DOUBLE, 16L));
        } catch (Throwable t) {
            throw WickraNative.rethrow(t);
        }
    }

    /** Number of updates required before update() yields a value. */
    public int warmupPeriod() {
        try {
            long n = (long) NativeMethods.WICKRA_ALLIGATOR_WARMUP_PERIOD.invokeExact(handle);
            return (int) n;
        } catch (Throwable t) {
            throw WickraNative.rethrow(t);
        }
    }

    /** Whether the indicator has consumed enough input to emit a value. */
    public boolean isReady() {
        try {
            byte r = (byte) NativeMethods.WICKRA_ALLIGATOR_IS_READY.invokeExact(handle);
            return r != 0;
        } catch (Throwable t) {
            throw WickraNative.rethrow(t);
        }
    }

    /** The indicator's canonical name. */
    public String name() {
        try {
            MemorySegment s = (MemorySegment) NativeMethods.WICKRA_ALLIGATOR_NAME.invokeExact(handle);
            return s.address() == 0 ? "" : s.reinterpret(Long.MAX_VALUE).getString(0);
        } catch (Throwable t) {
            throw WickraNative.rethrow(t);
        }
    }

    /** Reset to the just-constructed state. */
    public void reset() {
        try {
            NativeMethods.WICKRA_ALLIGATOR_RESET.invokeExact(handle);
        } catch (Throwable t) {
            throw WickraNative.rethrow(t);
        }
    }

    @Override public void close() {
        cleanable.clean();
    }
}
