// Generated from bindings/c/include/wickra.h. Do not edit by hand.
package org.wickra;

import org.wickra.internal.NativeMethods;
import org.wickra.internal.WickraNative;
import java.lang.foreign.Arena;
import java.lang.foreign.MemorySegment;
import java.lang.ref.Cleaner;
import static java.lang.foreign.ValueLayout.*;

/** Streaming ChaikinOscillator indicator over the Wickra C ABI. Not thread-safe; close when done. */
public final class ChaikinOscillator implements AutoCloseable {
    private final MemorySegment handle;
    private final Cleaner.Cleanable cleanable;

    public ChaikinOscillator(int fast, int slow) {
        if (fast < 0) {
            throw new IllegalArgumentException("fast must be non-negative");
        }
        if (slow < 0) {
            throw new IllegalArgumentException("slow must be non-negative");
        }
        MemorySegment h;
        try {
            h = (MemorySegment) NativeMethods.WICKRA_CHAIKIN_OSCILLATOR_NEW.invokeExact((long) fast, (long) slow);
        } catch (Throwable t) {
            throw WickraNative.rethrow(t);
        }
        if (h.address() == 0L) {
            throw new IllegalArgumentException("invalid ChaikinOscillator parameters");
        }
        this.handle = h;
        this.cleanable = WickraNative.register(this, h, NativeMethods.WICKRA_CHAIKIN_OSCILLATOR_FREE);
    }

    /** Push one observation; returns the indicator value (NaN during warmup). */
    public double update(double open, double high, double low, double close, double volume, long timestamp) {
        try {
            return (double) NativeMethods.WICKRA_CHAIKIN_OSCILLATOR_UPDATE.invokeExact(handle, open, high, low, close, volume, timestamp);
        } catch (Throwable t) {
            throw WickraNative.rethrow(t);
        }
    }

    /** Vectorized update over a whole series; NaN at warmup positions. */
    public double[] batch(double[] open, double[] high, double[] low, double[] close, double[] volume, double[] timestamp) {
        int n = open.length;
        if (high.length != n) {
            throw new IllegalArgumentException("all input arrays must have the same length");
        }
        if (low.length != n) {
            throw new IllegalArgumentException("all input arrays must have the same length");
        }
        if (close.length != n) {
            throw new IllegalArgumentException("all input arrays must have the same length");
        }
        if (volume.length != n) {
            throw new IllegalArgumentException("all input arrays must have the same length");
        }
        if (timestamp.length != n) {
            throw new IllegalArgumentException("all input arrays must have the same length");
        }
        try (Arena a = Arena.ofConfined()) {
            MemorySegment openSeg = a.allocateFrom(JAVA_DOUBLE, open);
            MemorySegment highSeg = a.allocateFrom(JAVA_DOUBLE, high);
            MemorySegment lowSeg = a.allocateFrom(JAVA_DOUBLE, low);
            MemorySegment closeSeg = a.allocateFrom(JAVA_DOUBLE, close);
            MemorySegment volumeSeg = a.allocateFrom(JAVA_DOUBLE, volume);
            MemorySegment timestampSeg = a.allocateFrom(JAVA_DOUBLE, timestamp);
            MemorySegment outSeg = a.allocate(JAVA_DOUBLE.byteSize() * n);
            NativeMethods.WICKRA_CHAIKIN_OSCILLATOR_BATCH.invokeExact(handle, openSeg, highSeg, lowSeg, closeSeg, volumeSeg, timestampSeg, outSeg, (long) n);
            double[] out = new double[n];
            MemorySegment.copy(outSeg, JAVA_DOUBLE, 0L, out, 0, n);
            return out;
        } catch (Throwable t) {
            throw WickraNative.rethrow(t);
        }
    }

    /** Reset to the just-constructed state. */
    public void reset() {
        try {
            NativeMethods.WICKRA_CHAIKIN_OSCILLATOR_RESET.invokeExact(handle);
        } catch (Throwable t) {
            throw WickraNative.rethrow(t);
        }
    }

    @Override public void close() {
        cleanable.clean();
    }
}
