// Generated from bindings/c/include/wickra.h. Do not edit by hand.
package org.wickra;

import org.wickra.internal.NativeMethods;
import org.wickra.internal.WickraNative;
import java.lang.foreign.Arena;
import java.lang.foreign.MemorySegment;
import java.lang.ref.Cleaner;
import static java.lang.foreign.ValueLayout.*;

/** Streaming HeikinAshi indicator over the Wickra C ABI. Not thread-safe; close when done. */
public final class HeikinAshi implements AutoCloseable {
    private final MemorySegment handle;
    private final Cleaner.Cleanable cleanable;

    public HeikinAshi() {
        MemorySegment h;
        try {
            h = (MemorySegment) NativeMethods.WICKRA_HEIKIN_ASHI_NEW.invokeExact();
        } catch (Throwable t) {
            throw WickraNative.rethrow(t);
        }
        if (h.address() == 0L) {
            throw new IllegalArgumentException("invalid HeikinAshi parameters");
        }
        this.handle = h;
        this.cleanable = WickraNative.register(this, h, NativeMethods.WICKRA_HEIKIN_ASHI_FREE);
    }

    /** Push one observation; returns the result, or null during warmup. */
    public HeikinAshiOutput update(double open, double high, double low, double close, double volume, long timestamp) {
        try (Arena a = Arena.ofConfined()) {
            MemorySegment out = a.allocate(32L);
            byte ok = (byte) NativeMethods.WICKRA_HEIKIN_ASHI_UPDATE.invokeExact(handle, open, high, low, close, volume, timestamp, out);
            if (ok == 0) {
                return null;
            }
            return new HeikinAshiOutput(
                out.get(JAVA_DOUBLE, 0L),
                out.get(JAVA_DOUBLE, 8L),
                out.get(JAVA_DOUBLE, 16L),
                out.get(JAVA_DOUBLE, 24L));
        } catch (Throwable t) {
            throw WickraNative.rethrow(t);
        }
    }

    /** Reset to the just-constructed state. */
    public void reset() {
        try {
            NativeMethods.WICKRA_HEIKIN_ASHI_RESET.invokeExact(handle);
        } catch (Throwable t) {
            throw WickraNative.rethrow(t);
        }
    }

    @Override public void close() {
        cleanable.clean();
    }
}
