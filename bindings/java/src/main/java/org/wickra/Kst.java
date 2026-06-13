// Generated from bindings/c/include/wickra.h. Do not edit by hand.
package org.wickra;

import org.wickra.internal.NativeMethods;
import org.wickra.internal.WickraNative;
import java.lang.foreign.Arena;
import java.lang.foreign.MemorySegment;
import java.lang.ref.Cleaner;
import static java.lang.foreign.ValueLayout.*;

/** Streaming Kst indicator over the Wickra C ABI. Not thread-safe; close when done. */
public final class Kst implements AutoCloseable {
    private final MemorySegment handle;
    private final Cleaner.Cleanable cleanable;

    public Kst(int roc1, int roc2, int roc3, int roc4, int sma1, int sma2, int sma3, int sma4, int signal) {
        if (roc1 < 0) {
            throw new IllegalArgumentException("roc1 must be non-negative");
        }
        if (roc2 < 0) {
            throw new IllegalArgumentException("roc2 must be non-negative");
        }
        if (roc3 < 0) {
            throw new IllegalArgumentException("roc3 must be non-negative");
        }
        if (roc4 < 0) {
            throw new IllegalArgumentException("roc4 must be non-negative");
        }
        if (sma1 < 0) {
            throw new IllegalArgumentException("sma1 must be non-negative");
        }
        if (sma2 < 0) {
            throw new IllegalArgumentException("sma2 must be non-negative");
        }
        if (sma3 < 0) {
            throw new IllegalArgumentException("sma3 must be non-negative");
        }
        if (sma4 < 0) {
            throw new IllegalArgumentException("sma4 must be non-negative");
        }
        if (signal < 0) {
            throw new IllegalArgumentException("signal must be non-negative");
        }
        MemorySegment h;
        try {
            h = (MemorySegment) NativeMethods.WICKRA_KST_NEW.invokeExact((long) roc1, (long) roc2, (long) roc3, (long) roc4, (long) sma1, (long) sma2, (long) sma3, (long) sma4, (long) signal);
        } catch (Throwable t) {
            throw WickraNative.rethrow(t);
        }
        if (h.address() == 0L) {
            throw new IllegalArgumentException("invalid Kst parameters");
        }
        this.handle = h;
        this.cleanable = WickraNative.register(this, h, NativeMethods.WICKRA_KST_FREE);
    }

    /** Push one observation; returns the result, or null during warmup. */
    public KstOutput update(double value) {
        try (Arena a = Arena.ofConfined()) {
            MemorySegment out = a.allocate(16L);
            byte ok = (byte) NativeMethods.WICKRA_KST_UPDATE.invokeExact(handle, value, out);
            if (ok == 0) {
                return null;
            }
            return new KstOutput(
                out.get(JAVA_DOUBLE, 0L),
                out.get(JAVA_DOUBLE, 8L));
        } catch (Throwable t) {
            throw WickraNative.rethrow(t);
        }
    }

    /** Number of updates required before update() yields a value. */
    public int warmupPeriod() {
        try {
            long n = (long) NativeMethods.WICKRA_KST_WARMUP_PERIOD.invokeExact(handle);
            return (int) n;
        } catch (Throwable t) {
            throw WickraNative.rethrow(t);
        }
    }

    /** Whether the indicator has consumed enough input to emit a value. */
    public boolean isReady() {
        try {
            byte r = (byte) NativeMethods.WICKRA_KST_IS_READY.invokeExact(handle);
            return r != 0;
        } catch (Throwable t) {
            throw WickraNative.rethrow(t);
        }
    }

    /** Reset to the just-constructed state. */
    public void reset() {
        try {
            NativeMethods.WICKRA_KST_RESET.invokeExact(handle);
        } catch (Throwable t) {
            throw WickraNative.rethrow(t);
        }
    }

    @Override public void close() {
        cleanable.clean();
    }
}
