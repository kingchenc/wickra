// Generated from bindings/c/include/wickra.h. Do not edit by hand.
package org.wickra;

import org.wickra.internal.NativeMethods;
import org.wickra.internal.WickraNative;
import java.lang.foreign.Arena;
import java.lang.foreign.MemorySegment;
import java.lang.ref.Cleaner;
import java.lang.ref.Reference;
import static java.lang.foreign.ValueLayout.*;

/** Streaming TickBars indicator over the Wickra C ABI. Not thread-safe; close when done. */
public final class TickBars implements AutoCloseable {
    private final MemorySegment handle;
    private final Cleaner.Cleanable cleanable;
    private boolean closed;

    public TickBars(int ticks) {
        if (ticks < 0) {
            throw new IllegalArgumentException("ticks must be non-negative");
        }
        MemorySegment h;
        try {
            h = (MemorySegment) NativeMethods.WICKRA_TICK_BARS_NEW.invokeExact((long) ticks);
        } catch (Throwable t) {
            throw WickraNative.rethrow(t);
        }
        if (h.address() == 0L) {
            throw new IllegalArgumentException("invalid TickBars parameters");
        }
        this.handle = h;
        this.cleanable = WickraNative.register(this, h, NativeMethods.WICKRA_TICK_BARS_FREE);
    }

    /** Push one observation; returns the bars completed by it (possibly empty). */
    public TickBar[] update(double open, double high, double low, double close, double volume, long timestamp) {
        final long cap = 64L;
        try (Arena a = Arena.ofConfined()) {
            MemorySegment out = a.allocate(40L * cap);
            long n = (long) NativeMethods.WICKRA_TICK_BARS_UPDATE.invokeExact(handle(), open, high, low, close, volume, timestamp, out, cap);
            if (n <= 0) {
                return new TickBar[0];
            }
            TickBar[] result = new TickBar[(int) n];
            long written = Math.min(n, cap);
            for (int i = 0; i < written; i++) {
                long b = (long) i * 40L;
                result[i] = new TickBar(
                    out.get(JAVA_DOUBLE, b + 0L),
                    out.get(JAVA_DOUBLE, b + 8L),
                    out.get(JAVA_DOUBLE, b + 16L),
                    out.get(JAVA_DOUBLE, b + 24L),
                    out.get(JAVA_DOUBLE, b + 32L));
            }
            if (n > cap) {
                // One input produced more elements than the buffer holds;
                // the surplus waits on the handle rather than being dropped.
                MemorySegment more = a.allocate(40L * (n - cap));
                long drained = (long) NativeMethods.WICKRA_TICK_BARS_DRAIN.invokeExact(handle(), more, n - cap);
                for (int i = 0; i < drained; i++) {
                    long b = (long) i * 40L;
                    result[(int) cap + i] = new TickBar(
                    more.get(JAVA_DOUBLE, b + 0L),
                    more.get(JAVA_DOUBLE, b + 8L),
                    more.get(JAVA_DOUBLE, b + 16L),
                    more.get(JAVA_DOUBLE, b + 24L),
                    more.get(JAVA_DOUBLE, b + 32L));
                }
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
            MemorySegment s = (MemorySegment) NativeMethods.WICKRA_TICK_BARS_NAME.invokeExact(handle());
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
            NativeMethods.WICKRA_TICK_BARS_RESET.invokeExact(handle());
        } catch (Throwable t) {
            throw WickraNative.rethrow(t);
        } finally {
            Reference.reachabilityFence(this);
        }
    }

    /** The native handle, refusing to hand out one that has been released. */
    private MemorySegment handle() {
        if (closed) {
            throw new IllegalStateException("TickBars has been closed");
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
