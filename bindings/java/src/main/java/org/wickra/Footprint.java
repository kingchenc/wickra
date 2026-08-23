// Generated from bindings/c/include/wickra.h. Do not edit by hand.
package org.wickra;

import org.wickra.internal.NativeMethods;
import org.wickra.internal.WickraNative;
import java.lang.foreign.Arena;
import java.lang.foreign.MemorySegment;
import java.lang.ref.Cleaner;
import java.lang.ref.Reference;
import static java.lang.foreign.ValueLayout.*;

/** Streaming Footprint indicator over the Wickra C ABI. Not thread-safe; close when done. */
public final class Footprint implements AutoCloseable {
    private final MemorySegment handle;
    private final Cleaner.Cleanable cleanable;
    private boolean closed;

    public Footprint(double tickSize) {
        MemorySegment h;
        try {
            h = (MemorySegment) NativeMethods.WICKRA_FOOTPRINT_NEW.invokeExact(tickSize);
        } catch (Throwable t) {
            throw WickraNative.rethrow(t);
        }
        if (h.address() == 0L) {
            throw new IllegalArgumentException("invalid Footprint parameters");
        }
        this.handle = h;
        this.cleanable = WickraNative.register(this, h, NativeMethods.WICKRA_FOOTPRINT_FREE);
    }

    /** Push one observation; returns the bars completed by it (possibly empty). */
    public FootprintLevel[] update(double price, double size, boolean isBuy, long timestamp) {
        final long cap = 64L;
        try (Arena a = Arena.ofConfined()) {
            MemorySegment out = a.allocate(24L * cap);
            long n = (long) NativeMethods.WICKRA_FOOTPRINT_UPDATE.invokeExact(handle(), price, size, (byte) (isBuy ? 1 : 0), timestamp, out, cap);
            if (n <= 0) {
                return new FootprintLevel[0];
            }
            FootprintLevel[] result = new FootprintLevel[(int) n];
            long written = Math.min(n, cap);
            for (int i = 0; i < written; i++) {
                long b = (long) i * 24L;
                result[i] = new FootprintLevel(
                    out.get(JAVA_DOUBLE, b + 0L),
                    out.get(JAVA_DOUBLE, b + 8L),
                    out.get(JAVA_DOUBLE, b + 16L));
            }
            if (n > cap) {
                // One input produced more elements than the buffer holds;
                // the surplus waits on the handle rather than being dropped.
                MemorySegment more = a.allocate(24L * (n - cap));
                long drained = (long) NativeMethods.WICKRA_FOOTPRINT_DRAIN.invokeExact(handle(), more, n - cap);
                for (int i = 0; i < drained; i++) {
                    long b = (long) i * 24L;
                    result[(int) cap + i] = new FootprintLevel(
                    more.get(JAVA_DOUBLE, b + 0L),
                    more.get(JAVA_DOUBLE, b + 8L),
                    more.get(JAVA_DOUBLE, b + 16L));
                }
            }
            return result;
        } catch (Throwable t) {
            throw WickraNative.rethrow(t);
        } finally {
            Reference.reachabilityFence(this);
        }
    }

    /** Number of updates required before update() yields a value. */
    public int warmupPeriod() {
        try {
            long n = (long) NativeMethods.WICKRA_FOOTPRINT_WARMUP_PERIOD.invokeExact(handle());
            return (int) n;
        } catch (Throwable t) {
            throw WickraNative.rethrow(t);
        } finally {
            Reference.reachabilityFence(this);
        }
    }

    /** Whether the indicator has consumed enough input to emit a value. */
    public boolean isReady() {
        try {
            byte r = (byte) NativeMethods.WICKRA_FOOTPRINT_IS_READY.invokeExact(handle());
            return r != 0;
        } catch (Throwable t) {
            throw WickraNative.rethrow(t);
        } finally {
            Reference.reachabilityFence(this);
        }
    }

    /** The indicator's canonical name. */
    public String name() {
        try {
            MemorySegment s = (MemorySegment) NativeMethods.WICKRA_FOOTPRINT_NAME.invokeExact(handle());
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
            NativeMethods.WICKRA_FOOTPRINT_RESET.invokeExact(handle());
        } catch (Throwable t) {
            throw WickraNative.rethrow(t);
        } finally {
            Reference.reachabilityFence(this);
        }
    }

    /** The native handle, refusing to hand out one that has been released. */
    private MemorySegment handle() {
        if (closed) {
            throw new IllegalStateException("Footprint has been closed");
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
