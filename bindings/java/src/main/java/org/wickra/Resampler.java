// Generated from bindings/c/include/wickra.h. Do not edit by hand.
package org.wickra;

import org.wickra.internal.NativeMethods;
import org.wickra.internal.WickraNative;
import java.lang.foreign.Arena;
import java.lang.foreign.MemorySegment;
import java.lang.ref.Cleaner;
import java.lang.ref.Reference;
import static java.lang.foreign.ValueLayout.*;

/** Streaming Resampler indicator over the Wickra C ABI. Not thread-safe; close when done. */
public final class Resampler implements AutoCloseable {
    private final MemorySegment handle;
    private final Cleaner.Cleanable cleanable;
    private boolean closed;

    public Resampler(long timeframe, boolean gapFill) {
        MemorySegment h;
        try {
            h = (MemorySegment) NativeMethods.WICKRA_RESAMPLER_NEW.invokeExact(timeframe, (byte) (gapFill ? 1 : 0));
        } catch (Throwable t) {
            throw WickraNative.rethrow(t);
        }
        if (h.address() == 0L) {
            throw new IllegalArgumentException("invalid Resampler parameters");
        }
        this.handle = h;
        this.cleanable = WickraNative.register(this, h, NativeMethods.WICKRA_RESAMPLER_FREE);
    }


    /** Feed one trade tick; returns the candles it closed (possibly empty). */
    public Candle[] push(double open, double high, double low, double close, double volume, long timestamp) {
        try {
            long n = (long) NativeMethods.WICKRA_RESAMPLER_PUSH.invokeExact(handle(), open, high, low, close, volume, timestamp);
            if (n <= 0) {
                return new Candle[0];
            }
            try (Arena a = Arena.ofConfined()) {
                MemorySegment out = a.allocate(48L * n);
                long w = (long) NativeMethods.WICKRA_RESAMPLER_DRAIN.invokeExact(handle(), out, n);
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

    /** Emit the final, still-open candle (null if none is pending). */
    public Candle flush() {
        try (Arena a = Arena.ofConfined()) {
            MemorySegment out = a.allocate(48L);
            byte ok = (byte) NativeMethods.WICKRA_RESAMPLER_FLUSH.invokeExact(handle(), out);
            if (ok == 0) {
                return null;
            }
            return new Candle(
                out.get(JAVA_DOUBLE, 0L),
                out.get(JAVA_DOUBLE, 8L),
                out.get(JAVA_DOUBLE, 16L),
                out.get(JAVA_DOUBLE, 24L),
                out.get(JAVA_DOUBLE, 32L),
                (double) out.get(JAVA_LONG, 40L));
        } catch (Throwable t) {
            throw WickraNative.rethrow(t);
        } finally {
            Reference.reachabilityFence(this);
        }
    }

    /** The native handle, refusing to hand out one that has been released. */
    private MemorySegment handle() {
        if (closed) {
            throw new IllegalStateException("Resampler has been closed");
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
