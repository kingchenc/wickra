// Generated from bindings/c/include/wickra.h. Do not edit by hand.
package org.wickra;

import org.wickra.internal.NativeMethods;
import org.wickra.internal.WickraNative;
import java.lang.foreign.Arena;
import java.lang.foreign.MemorySegment;
import java.lang.ref.Cleaner;
import static java.lang.foreign.ValueLayout.*;

/** Streaming MaEnvelope indicator over the Wickra C ABI. Not thread-safe; close when done. */
public final class MaEnvelope implements AutoCloseable {
    private final MemorySegment handle;
    private final Cleaner.Cleanable cleanable;

    public MaEnvelope(int period, double percent) {
        if (period < 0) {
            throw new IllegalArgumentException("period must be non-negative");
        }
        MemorySegment h;
        try {
            h = (MemorySegment) NativeMethods.WICKRA_MA_ENVELOPE_NEW.invokeExact((long) period, percent);
        } catch (Throwable t) {
            throw WickraNative.rethrow(t);
        }
        if (h.address() == 0L) {
            throw new IllegalArgumentException("invalid MaEnvelope parameters");
        }
        this.handle = h;
        this.cleanable = WickraNative.register(this, h, NativeMethods.WICKRA_MA_ENVELOPE_FREE);
    }

    /** Push one observation; returns the result, or null during warmup. */
    public MaEnvelopeOutput update(double value) {
        try (Arena a = Arena.ofConfined()) {
            MemorySegment out = a.allocate(24L);
            byte ok = (byte) NativeMethods.WICKRA_MA_ENVELOPE_UPDATE.invokeExact(handle, value, out);
            if (ok == 0) {
                return null;
            }
            return new MaEnvelopeOutput(
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
            NativeMethods.WICKRA_MA_ENVELOPE_RESET.invokeExact(handle);
        } catch (Throwable t) {
            throw WickraNative.rethrow(t);
        }
    }

    @Override public void close() {
        cleanable.clean();
    }
}
