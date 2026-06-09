// Generated from bindings/c/include/wickra.h. Do not edit by hand.
package org.wickra;

import org.wickra.internal.NativeMethods;
import org.wickra.internal.WickraNative;
import java.lang.foreign.Arena;
import java.lang.foreign.MemorySegment;
import java.lang.ref.Cleaner;
import static java.lang.foreign.ValueLayout.*;

/** Streaming MacdIndicator indicator over the Wickra C ABI. Not thread-safe; close when done. */
public final class MacdIndicator implements AutoCloseable {
    private final MemorySegment handle;
    private final Cleaner.Cleanable cleanable;

    public MacdIndicator(int fast, int slow, int signal) {
        if (fast < 0) {
            throw new IllegalArgumentException("fast must be non-negative");
        }
        if (slow < 0) {
            throw new IllegalArgumentException("slow must be non-negative");
        }
        if (signal < 0) {
            throw new IllegalArgumentException("signal must be non-negative");
        }
        MemorySegment h;
        try {
            h = (MemorySegment) NativeMethods.WICKRA_MACD_INDICATOR_NEW.invokeExact((long) fast, (long) slow, (long) signal);
        } catch (Throwable t) {
            throw WickraNative.rethrow(t);
        }
        if (h.address() == 0L) {
            throw new IllegalArgumentException("invalid MacdIndicator parameters");
        }
        this.handle = h;
        this.cleanable = WickraNative.register(this, h, NativeMethods.WICKRA_MACD_INDICATOR_FREE);
    }

    /** Push one observation; returns the result, or null during warmup. */
    public MacdOutput update(double value) {
        try (Arena a = Arena.ofConfined()) {
            MemorySegment out = a.allocate(24L);
            byte ok = (byte) NativeMethods.WICKRA_MACD_INDICATOR_UPDATE.invokeExact(handle, value, out);
            if (ok == 0) {
                return null;
            }
            return new MacdOutput(
                out.get(JAVA_DOUBLE, 0L),
                out.get(JAVA_DOUBLE, 8L),
                out.get(JAVA_DOUBLE, 16L));
        } catch (Throwable t) {
            throw WickraNative.rethrow(t);
        }
    }

    /** Reset to the just-constructed state. */
    public void reset() {
        try {
            NativeMethods.WICKRA_MACD_INDICATOR_RESET.invokeExact(handle);
        } catch (Throwable t) {
            throw WickraNative.rethrow(t);
        }
    }

    @Override public void close() {
        cleanable.clean();
    }
}
