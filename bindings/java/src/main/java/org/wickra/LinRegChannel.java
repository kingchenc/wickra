// Generated from bindings/c/include/wickra.h. Do not edit by hand.
package org.wickra;

import org.wickra.internal.NativeMethods;
import org.wickra.internal.WickraNative;
import java.lang.foreign.Arena;
import java.lang.foreign.MemorySegment;
import java.lang.ref.Cleaner;
import java.lang.ref.Reference;
import static java.lang.foreign.ValueLayout.*;

/** Streaming LinRegChannel indicator over the Wickra C ABI. Not thread-safe; close when done. */
public final class LinRegChannel implements AutoCloseable {
    private final MemorySegment handle;
    private final Cleaner.Cleanable cleanable;
    private boolean closed;

    public LinRegChannel(int period, double multiplier) {
        if (period < 0) {
            throw new IllegalArgumentException("period must be non-negative");
        }
        MemorySegment h;
        try {
            h = (MemorySegment) NativeMethods.WICKRA_LIN_REG_CHANNEL_NEW.invokeExact((long) period, multiplier);
        } catch (Throwable t) {
            throw WickraNative.rethrow(t);
        }
        if (h.address() == 0L) {
            throw new IllegalArgumentException("invalid LinRegChannel parameters");
        }
        this.handle = h;
        this.cleanable = WickraNative.register(this, h, NativeMethods.WICKRA_LIN_REG_CHANNEL_FREE);
    }

    /** Push one observation; returns the result, or null during warmup. */
    public LinRegChannelOutput update(double value) {
        try (Arena a = Arena.ofConfined()) {
            MemorySegment out = a.allocate(24L);
            byte ok = (byte) NativeMethods.WICKRA_LIN_REG_CHANNEL_UPDATE.invokeExact(handle(), value, out);
            if (ok == 0) {
                return null;
            }
            return new LinRegChannelOutput(
                out.get(JAVA_DOUBLE, 0L),
                out.get(JAVA_DOUBLE, 8L),
                out.get(JAVA_DOUBLE, 16L));
        } catch (Throwable t) {
            throw WickraNative.rethrow(t);
        } finally {
            Reference.reachabilityFence(this);
        }
    }

    /**
     * Vectorized update over whole series, one output per input. A row the
     * indicator did not produce -- warmup, or an input it rejected -- carries
     * NaN in every floating-point field.
     */
    public LinRegChannelOutput[] batch(double[] input) {
        int n = input.length;
        try (Arena a = Arena.ofConfined()) {
            MemorySegment inputSeg = a.allocateFrom(JAVA_DOUBLE, input);
            MemorySegment outSeg = a.allocate(24L * n);
            NativeMethods.WICKRA_LIN_REG_CHANNEL_BATCH.invokeExact(handle(), inputSeg, outSeg, (long) n);
            LinRegChannelOutput[] out = new LinRegChannelOutput[n];
            for (int i = 0; i < n; i++) {
                out[i] = new LinRegChannelOutput(
                        outSeg.get(JAVA_DOUBLE, i * 24L + 0L),
                        outSeg.get(JAVA_DOUBLE, i * 24L + 8L),
                        outSeg.get(JAVA_DOUBLE, i * 24L + 16L));
            }
            return out;
        } catch (Throwable t) {
            throw WickraNative.rethrow(t);
        } finally {
            Reference.reachabilityFence(this);
        }
    }

    /** Number of updates required before update() yields a value. */
    public int warmupPeriod() {
        try {
            long n = (long) NativeMethods.WICKRA_LIN_REG_CHANNEL_WARMUP_PERIOD.invokeExact(handle());
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
            byte r = (byte) NativeMethods.WICKRA_LIN_REG_CHANNEL_IS_READY.invokeExact(handle());
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
            MemorySegment s = (MemorySegment) NativeMethods.WICKRA_LIN_REG_CHANNEL_NAME.invokeExact(handle());
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
            NativeMethods.WICKRA_LIN_REG_CHANNEL_RESET.invokeExact(handle());
        } catch (Throwable t) {
            throw WickraNative.rethrow(t);
        } finally {
            Reference.reachabilityFence(this);
        }
    }

    /** The native handle, refusing to hand out one that has been released. */
    private MemorySegment handle() {
        if (closed) {
            throw new IllegalStateException("LinRegChannel has been closed");
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
