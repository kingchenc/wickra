// Generated from bindings/c/include/wickra.h. Do not edit by hand.
package org.wickra;

import org.wickra.internal.NativeMethods;
import org.wickra.internal.WickraNative;
import java.lang.foreign.Arena;
import java.lang.foreign.MemorySegment;
import java.lang.ref.Cleaner;
import static java.lang.foreign.ValueLayout.*;

/** Streaming ConnorsRsi indicator over the Wickra C ABI. Not thread-safe; close when done. */
public final class ConnorsRsi implements AutoCloseable {
    private final MemorySegment handle;
    private final Cleaner.Cleanable cleanable;

    public ConnorsRsi(int periodRsi, int periodStreak, int periodRank) {
        if (periodRsi < 0) {
            throw new IllegalArgumentException("periodRsi must be non-negative");
        }
        if (periodStreak < 0) {
            throw new IllegalArgumentException("periodStreak must be non-negative");
        }
        if (periodRank < 0) {
            throw new IllegalArgumentException("periodRank must be non-negative");
        }
        MemorySegment h;
        try {
            h = (MemorySegment) NativeMethods.WICKRA_CONNORS_RSI_NEW.invokeExact((long) periodRsi, (long) periodStreak, (long) periodRank);
        } catch (Throwable t) {
            throw WickraNative.rethrow(t);
        }
        if (h.address() == 0L) {
            throw new IllegalArgumentException("invalid ConnorsRsi parameters");
        }
        this.handle = h;
        this.cleanable = WickraNative.register(this, h, NativeMethods.WICKRA_CONNORS_RSI_FREE);
    }

    /** Push one observation; returns the indicator value (NaN during warmup). */
    public double update(double value) {
        try {
            return (double) NativeMethods.WICKRA_CONNORS_RSI_UPDATE.invokeExact(handle, value);
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
            NativeMethods.WICKRA_CONNORS_RSI_BATCH.invokeExact(handle, inputSeg, outSeg, (long) n);
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
            NativeMethods.WICKRA_CONNORS_RSI_RESET.invokeExact(handle);
        } catch (Throwable t) {
            throw WickraNative.rethrow(t);
        }
    }

    @Override public void close() {
        cleanable.clean();
    }
}
