// Generated from bindings/c/include/wickra.h. Do not edit by hand.
package org.wickra;

import org.wickra.internal.NativeMethods;
import org.wickra.internal.WickraNative;
import java.lang.foreign.Arena;
import java.lang.foreign.MemorySegment;
import java.lang.ref.Cleaner;
import static java.lang.foreign.ValueLayout.*;

/** Streaming Tii indicator over the Wickra C ABI. Not thread-safe; close when done. */
public final class Tii implements AutoCloseable {
    private final MemorySegment handle;
    private final Cleaner.Cleanable cleanable;

    public Tii(int smaPeriod, int devPeriod) {
        if (smaPeriod < 0) {
            throw new IllegalArgumentException("smaPeriod must be non-negative");
        }
        if (devPeriod < 0) {
            throw new IllegalArgumentException("devPeriod must be non-negative");
        }
        MemorySegment h;
        try {
            h = (MemorySegment) NativeMethods.WICKRA_TII_NEW.invokeExact((long) smaPeriod, (long) devPeriod);
        } catch (Throwable t) {
            throw WickraNative.rethrow(t);
        }
        if (h.address() == 0L) {
            throw new IllegalArgumentException("invalid Tii parameters");
        }
        this.handle = h;
        this.cleanable = WickraNative.register(this, h, NativeMethods.WICKRA_TII_FREE);
    }

    /** Push one observation; returns the indicator value (NaN during warmup). */
    public double update(double value) {
        try {
            return (double) NativeMethods.WICKRA_TII_UPDATE.invokeExact(handle, value);
        } catch (Throwable t) {
            throw WickraNative.rethrow(t);
        }
    }

    /** Vectorized update over a whole series; NaN at warmup positions. */
    public double[] batch(double[] input) {
        int n = input.length;
        try (Arena a = Arena.ofConfined()) {
            MemorySegment inputSeg = a.allocateFrom(JAVA_DOUBLE, input);
            MemorySegment outSeg = a.allocate(JAVA_DOUBLE.byteSize() * n);
            NativeMethods.WICKRA_TII_BATCH.invokeExact(handle, inputSeg, outSeg, (long) n);
            double[] out = new double[n];
            MemorySegment.copy(outSeg, JAVA_DOUBLE, 0L, out, 0, n);
            return out;
        } catch (Throwable t) {
            throw WickraNative.rethrow(t);
        }
    }

    /** Number of updates required before update() yields a value. */
    public int warmupPeriod() {
        try {
            long n = (long) NativeMethods.WICKRA_TII_WARMUP_PERIOD.invokeExact(handle);
            return (int) n;
        } catch (Throwable t) {
            throw WickraNative.rethrow(t);
        }
    }

    /** Whether the indicator has consumed enough input to emit a value. */
    public boolean isReady() {
        try {
            byte r = (byte) NativeMethods.WICKRA_TII_IS_READY.invokeExact(handle);
            return r != 0;
        } catch (Throwable t) {
            throw WickraNative.rethrow(t);
        }
    }

    /** The indicator's canonical name. */
    public String name() {
        try {
            MemorySegment s = (MemorySegment) NativeMethods.WICKRA_TII_NAME.invokeExact(handle);
            return s.address() == 0 ? "" : s.reinterpret(Long.MAX_VALUE).getString(0);
        } catch (Throwable t) {
            throw WickraNative.rethrow(t);
        }
    }

    /** Reset to the just-constructed state. */
    public void reset() {
        try {
            NativeMethods.WICKRA_TII_RESET.invokeExact(handle);
        } catch (Throwable t) {
            throw WickraNative.rethrow(t);
        }
    }

    @Override public void close() {
        cleanable.clean();
    }
}
