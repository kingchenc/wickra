// Generated from bindings/c/include/wickra.h. Do not edit by hand.
package org.wickra;

import org.wickra.internal.NativeMethods;
import org.wickra.internal.WickraNative;
import java.lang.foreign.Arena;
import java.lang.foreign.MemorySegment;
import java.lang.ref.Cleaner;
import static java.lang.foreign.ValueLayout.*;

/** Streaming VolumeBars indicator over the Wickra C ABI. Not thread-safe; close when done. */
public final class VolumeBars implements AutoCloseable {
    private final MemorySegment handle;
    private final Cleaner.Cleanable cleanable;

    public VolumeBars(double volumePerBar) {
        MemorySegment h;
        try {
            h = (MemorySegment) NativeMethods.WICKRA_VOLUME_BARS_NEW.invokeExact(volumePerBar);
        } catch (Throwable t) {
            throw WickraNative.rethrow(t);
        }
        if (h.address() == 0L) {
            throw new IllegalArgumentException("invalid VolumeBars parameters");
        }
        this.handle = h;
        this.cleanable = WickraNative.register(this, h, NativeMethods.WICKRA_VOLUME_BARS_FREE);
    }

    /** Push one observation; returns the bars completed by it (possibly empty). */
    public VolumeBar[] update(double open, double high, double low, double close, double volume, long timestamp) {
        final long cap = 64L;
        try (Arena a = Arena.ofConfined()) {
            MemorySegment out = a.allocate(40L * cap);
            long n = (long) NativeMethods.WICKRA_VOLUME_BARS_UPDATE.invokeExact(handle, open, high, low, close, volume, timestamp, out, cap);
            if (n <= 0) {
                return new VolumeBar[0];
            }
            VolumeBar[] result = new VolumeBar[(int) n];
            for (int i = 0; i < n; i++) {
                long b = (long) i * 40L;
                result[i] = new VolumeBar(
                    out.get(JAVA_DOUBLE, b + 0L),
                    out.get(JAVA_DOUBLE, b + 8L),
                    out.get(JAVA_DOUBLE, b + 16L),
                    out.get(JAVA_DOUBLE, b + 24L),
                    out.get(JAVA_DOUBLE, b + 32L));
            }
            return result;
        } catch (Throwable t) {
            throw WickraNative.rethrow(t);
        }
    }

    /** The indicator's canonical name. */
    public String name() {
        try {
            MemorySegment s = (MemorySegment) NativeMethods.WICKRA_VOLUME_BARS_NAME.invokeExact(handle);
            return s.address() == 0 ? "" : s.reinterpret(Long.MAX_VALUE).getString(0);
        } catch (Throwable t) {
            throw WickraNative.rethrow(t);
        }
    }

    /** Reset to the just-constructed state. */
    public void reset() {
        try {
            NativeMethods.WICKRA_VOLUME_BARS_RESET.invokeExact(handle);
        } catch (Throwable t) {
            throw WickraNative.rethrow(t);
        }
    }

    @Override public void close() {
        cleanable.clean();
    }
}
