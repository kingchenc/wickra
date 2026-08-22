// Generated from bindings/c/include/wickra.h. Do not edit by hand.
package org.wickra;

import org.wickra.internal.NativeMethods;
import org.wickra.internal.WickraNative;
import java.lang.foreign.Arena;
import java.lang.foreign.MemorySegment;
import java.lang.ref.Cleaner;
import java.lang.ref.Reference;
import static java.lang.foreign.ValueLayout.*;

/** Streaming RunBars indicator over the Wickra C ABI. Not thread-safe; close when done. */
public final class RunBars implements AutoCloseable {
    private final MemorySegment handle;
    private final Cleaner.Cleanable cleanable;
    private boolean closed;

    public RunBars(int runLength) {
        if (runLength < 0) {
            throw new IllegalArgumentException("runLength must be non-negative");
        }
        MemorySegment h;
        try {
            h = (MemorySegment) NativeMethods.WICKRA_RUN_BARS_NEW.invokeExact((long) runLength);
        } catch (Throwable t) {
            throw WickraNative.rethrow(t);
        }
        if (h.address() == 0L) {
            throw new IllegalArgumentException("invalid RunBars parameters");
        }
        this.handle = h;
        this.cleanable = WickraNative.register(this, h, NativeMethods.WICKRA_RUN_BARS_FREE);
    }

    /** Push one observation; returns the bars completed by it (possibly empty). */
    public RunBar[] update(double open, double high, double low, double close, double volume, long timestamp) {
        final long cap = 64L;
        try (Arena a = Arena.ofConfined()) {
            MemorySegment out = a.allocate(48L * cap);
            long n = (long) NativeMethods.WICKRA_RUN_BARS_UPDATE.invokeExact(handle(), open, high, low, close, volume, timestamp, out, cap);
            if (n <= 0) {
                return new RunBar[0];
            }
            RunBar[] result = new RunBar[(int) n];
            for (int i = 0; i < n; i++) {
                long b = (long) i * 48L;
                result[i] = new RunBar(
                    out.get(JAVA_DOUBLE, b + 0L),
                    out.get(JAVA_DOUBLE, b + 8L),
                    out.get(JAVA_DOUBLE, b + 16L),
                    out.get(JAVA_DOUBLE, b + 24L),
                    (double) out.get(JAVA_LONG, b + 32L),
                    (double) out.get(JAVA_BYTE, b + 40L));
            }
            return result;
        } catch (Throwable t) {
            throw WickraNative.rethrow(t);
        } finally {
            Reference.reachabilityFence(this);
        }
    }

    /** The indicator's canonical name. */
    public String name() {
        try {
            MemorySegment s = (MemorySegment) NativeMethods.WICKRA_RUN_BARS_NAME.invokeExact(handle());
            return s.address() == 0 ? "" : s.reinterpret(Long.MAX_VALUE).getString(0);
        } catch (Throwable t) {
            throw WickraNative.rethrow(t);
        } finally {
            Reference.reachabilityFence(this);
        }
    }

    /** Reset to the just-constructed state. */
    public void reset() {
        try {
            NativeMethods.WICKRA_RUN_BARS_RESET.invokeExact(handle());
        } catch (Throwable t) {
            throw WickraNative.rethrow(t);
        } finally {
            Reference.reachabilityFence(this);
        }
    }

    /** The native handle, refusing to hand out one that has been released. */
    private MemorySegment handle() {
        if (closed) {
            throw new IllegalStateException("RunBars has been closed");
        }
        return handle;
    }

    @Override public void close() {
        if (closed) {
            return;
        }
        closed = true;
        cleanable.clean();
    }
}
