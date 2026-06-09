// Generated from bindings/c/include/wickra.h. Do not edit by hand.
package org.wickra;

import org.wickra.internal.NativeMethods;
import org.wickra.internal.WickraNative;
import java.lang.foreign.Arena;
import java.lang.foreign.MemorySegment;
import java.lang.ref.Cleaner;
import static java.lang.foreign.ValueLayout.*;

/** Streaming McClellanSummationIndex indicator over the Wickra C ABI. Not thread-safe; close when done. */
public final class McClellanSummationIndex implements AutoCloseable {
    private final MemorySegment handle;
    private final Cleaner.Cleanable cleanable;

    public McClellanSummationIndex() {
        MemorySegment h;
        try {
            h = (MemorySegment) NativeMethods.WICKRA_MC_CLELLAN_SUMMATION_INDEX_NEW.invokeExact();
        } catch (Throwable t) {
            throw WickraNative.rethrow(t);
        }
        if (h.address() == 0L) {
            throw new IllegalArgumentException("invalid McClellanSummationIndex parameters");
        }
        this.handle = h;
        this.cleanable = WickraNative.register(this, h, NativeMethods.WICKRA_MC_CLELLAN_SUMMATION_INDEX_FREE);
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
            MemorySegment newHighSeg = a.allocateFrom(JAVA_DOUBLE, newHigh);
            MemorySegment newLowSeg = a.allocateFrom(JAVA_DOUBLE, newLow);
            MemorySegment aboveMaSeg = a.allocateFrom(JAVA_DOUBLE, aboveMa);
            MemorySegment onBuySignalSeg = a.allocateFrom(JAVA_DOUBLE, onBuySignal);
            return (double) NativeMethods.WICKRA_MC_CLELLAN_SUMMATION_INDEX_UPDATE.invokeExact(handle, changeSeg, volumeSeg, newHighSeg, newLowSeg, aboveMaSeg, onBuySignalSeg, (long) change.length, timestamp);
        } catch (Throwable t) {
            throw WickraNative.rethrow(t);
        }
    }

    /** Reset to the just-constructed state. */
    public void reset() {
        try {
            NativeMethods.WICKRA_MC_CLELLAN_SUMMATION_INDEX_RESET.invokeExact(handle);
        } catch (Throwable t) {
            throw WickraNative.rethrow(t);
        }
    }

    @Override public void close() {
        cleanable.clean();
    }
}
