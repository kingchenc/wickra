// Generated from bindings/c/include/wickra.h. Do not edit by hand.
package org.wickra;

import org.wickra.internal.NativeMethods;
import org.wickra.internal.WickraNative;
import java.lang.foreign.Arena;
import java.lang.foreign.MemorySegment;
import java.lang.ref.Cleaner;
import static java.lang.foreign.ValueLayout.*;

/** Streaming WaveTrend indicator over the Wickra C ABI. Not thread-safe; close when done. */
public final class WaveTrend implements AutoCloseable {
    private final MemorySegment handle;
    private final Cleaner.Cleanable cleanable;

    public WaveTrend(int channelPeriod, int averagePeriod, int signalPeriod) {
        if (channelPeriod < 0) {
            throw new IllegalArgumentException("channelPeriod must be non-negative");
        }
        if (averagePeriod < 0) {
            throw new IllegalArgumentException("averagePeriod must be non-negative");
        }
        if (signalPeriod < 0) {
            throw new IllegalArgumentException("signalPeriod must be non-negative");
        }
        MemorySegment h;
        try {
            h = (MemorySegment) NativeMethods.WICKRA_WAVE_TREND_NEW.invokeExact((long) channelPeriod, (long) averagePeriod, (long) signalPeriod);
        } catch (Throwable t) {
            throw WickraNative.rethrow(t);
        }
        if (h.address() == 0L) {
            throw new IllegalArgumentException("invalid WaveTrend parameters");
        }
        this.handle = h;
        this.cleanable = WickraNative.register(this, h, NativeMethods.WICKRA_WAVE_TREND_FREE);
    }

    /** Push one observation; returns the result, or null during warmup. */
    public WaveTrendOutput update(double open, double high, double low, double close, double volume, long timestamp) {
        try (Arena a = Arena.ofConfined()) {
            MemorySegment out = a.allocate(16L);
            byte ok = (byte) NativeMethods.WICKRA_WAVE_TREND_UPDATE.invokeExact(handle, open, high, low, close, volume, timestamp, out);
            if (ok == 0) {
                return null;
            }
            return new WaveTrendOutput(
                out.get(JAVA_DOUBLE, 0L),
                out.get(JAVA_DOUBLE, 8L));
        } catch (Throwable t) {
            throw WickraNative.rethrow(t);
        }
    }

    /** Reset to the just-constructed state. */
    public void reset() {
        try {
            NativeMethods.WICKRA_WAVE_TREND_RESET.invokeExact(handle);
        } catch (Throwable t) {
            throw WickraNative.rethrow(t);
        }
    }

    @Override public void close() {
        cleanable.clean();
    }
}
