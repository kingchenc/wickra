// Generated from bindings/c/include/wickra.h. Do not edit by hand.
package org.wickra;

import org.wickra.internal.NativeMethods;
import org.wickra.internal.WickraNative;
import java.lang.foreign.Arena;
import java.lang.foreign.MemorySegment;
import java.lang.ref.Cleaner;
import static java.lang.foreign.ValueLayout.*;

/** Streaming BreadthThrust indicator over the Wickra C ABI. Not thread-safe; close when done. */
public final class BreadthThrust implements AutoCloseable {
    private final MemorySegment handle;
    private final Cleaner.Cleanable cleanable;

    public BreadthThrust(int period) {
        if (period < 0) {
            throw new IllegalArgumentException("period must be non-negative");
        }
        MemorySegment h;
        try {
            h = (MemorySegment) NativeMethods.WICKRA_BREADTH_THRUST_NEW.invokeExact((long) period);
        } catch (Throwable t) {
            throw WickraNative.rethrow(t);
        }
        if (h.address() == 0L) {
            throw new IllegalArgumentException("invalid BreadthThrust parameters");
        }
        this.handle = h;
        this.cleanable = WickraNative.register(this, h, NativeMethods.WICKRA_BREADTH_THRUST_FREE);
    }

    /** Push one observation; returns the indicator value (NaN during warmup). */
    public double update(double[] change, double[] volume, double[] newHigh, double[] newLow, double[] aboveMa, double[] onBuySignal, long timestamp) {
        if (volume.length != change.length) {
            throw new IllegalArgumentException("input arrays in the same group must have equal length");
        }
        if (newHigh.length != change.length) {
            throw new IllegalArgumentException("input arrays in the same group must have equal length");
        }
        if (newLow.length != change.length) {
            throw new IllegalArgumentException("input arrays in the same group must have equal length");
        }
        if (aboveMa.length != change.length) {
            throw new IllegalArgumentException("input arrays in the same group must have equal length");
        }
        if (onBuySignal.length != change.length) {
            throw new IllegalArgumentException("input arrays in the same group must have equal length");
        }
        try (Arena a = Arena.ofConfined()) {
            MemorySegment changeSeg = a.allocateFrom(JAVA_DOUBLE, change);
            MemorySegment volumeSeg = a.allocateFrom(JAVA_DOUBLE, volume);
            MemorySegment newHighSeg = WickraNative.boolSegment(a, newHigh);
            MemorySegment newLowSeg = WickraNative.boolSegment(a, newLow);
            MemorySegment aboveMaSeg = WickraNative.boolSegment(a, aboveMa);
            MemorySegment onBuySignalSeg = WickraNative.boolSegment(a, onBuySignal);
            return (double) NativeMethods.WICKRA_BREADTH_THRUST_UPDATE.invokeExact(handle, changeSeg, volumeSeg, newHighSeg, newLowSeg, aboveMaSeg, onBuySignalSeg, (long) change.length, timestamp);
        } catch (Throwable t) {
            throw WickraNative.rethrow(t);
        }
    }

    /** Number of updates required before update() yields a value. */
    public int warmupPeriod() {
        try {
            long n = (long) NativeMethods.WICKRA_BREADTH_THRUST_WARMUP_PERIOD.invokeExact(handle);
            return (int) n;
        } catch (Throwable t) {
            throw WickraNative.rethrow(t);
        }
    }

    /** Whether the indicator has consumed enough input to emit a value. */
    public boolean isReady() {
        try {
            byte r = (byte) NativeMethods.WICKRA_BREADTH_THRUST_IS_READY.invokeExact(handle);
            return r != 0;
        } catch (Throwable t) {
            throw WickraNative.rethrow(t);
        }
    }

    /** Reset to the just-constructed state. */
    public void reset() {
        try {
            NativeMethods.WICKRA_BREADTH_THRUST_RESET.invokeExact(handle);
        } catch (Throwable t) {
            throw WickraNative.rethrow(t);
        }
    }

    @Override public void close() {
        cleanable.clean();
    }
}
