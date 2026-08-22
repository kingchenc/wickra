// Generated from bindings/c/include/wickra.h. Do not edit by hand.
package org.wickra;

import org.wickra.internal.NativeMethods;
import org.wickra.internal.WickraNative;
import java.lang.foreign.Arena;
import java.lang.foreign.MemorySegment;
import java.lang.ref.Cleaner;
import java.lang.ref.Reference;
import static java.lang.foreign.ValueLayout.*;

/** Streaming TickAggregator indicator over the Wickra C ABI. Not thread-safe; close when done. */
public final class TickAggregator implements AutoCloseable {
    private final MemorySegment handle;
    private final Cleaner.Cleanable cleanable;
    private boolean closed;

    public TickAggregator(long bucket, boolean gapFill) {
        MemorySegment h;
        try {
            h = (MemorySegment) NativeMethods.WICKRA_TICK_AGGREGATOR_NEW.invokeExact(bucket, (byte) (gapFill ? 1 : 0));
        } catch (Throwable t) {
            throw WickraNative.rethrow(t);
        }
        if (h.address() == 0L) {
            throw new IllegalArgumentException("invalid TickAggregator parameters");
        }
        this.handle = h;
        this.cleanable = WickraNative.register(this, h, NativeMethods.WICKRA_TICK_AGGREGATOR_FREE);
    }


    /** Feed one trade tick; returns the candles it closed (possibly empty). */
    public Candle[] push(double price, double size, long timestamp) {
        try {
            long n = (long) NativeMethods.WICKRA_TICK_AGGREGATOR_PUSH.invokeExact(handle(), price, size, timestamp);
            if (n <= 0) {
                return new Candle[0];
            }
            try (Arena a = Arena.ofConfined()) {
                MemorySegment out = a.allocate(48L * n);
                long w = (long) NativeMethods.WICKRA_TICK_AGGREGATOR_DRAIN.invokeExact(handle(), out, n);
                Candle[] result = new Candle[(int) w];
                for (int i = 0; i < w; i++) {
                    long b = (long) i * 48L;
                    result[i] = new Candle(
                        out.get(JAVA_DOUBLE, b + 0L),
                        out.get(JAVA_DOUBLE, b + 8L),
                        out.get(JAVA_DOUBLE, b + 16L),
                        out.get(JAVA_DOUBLE, b + 24L),
                        out.get(JAVA_DOUBLE, b + 32L),
                        (double) out.get(JAVA_LONG, b + 40L));
                }
                return result;
            }
        } catch (Throwable t) {
            throw WickraNative.rethrow(t);
        } finally {
            Reference.reachabilityFence(this);
        }
    }

    /** The native handle, refusing to hand out one that has been released. */
    private MemorySegment handle() {
        if (closed) {
            throw new IllegalStateException("TickAggregator has been closed");
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
