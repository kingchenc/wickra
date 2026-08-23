// Generated from bindings/c/include/wickra.h. Do not edit by hand.
package org.wickra;

import org.wickra.internal.NativeMethods;
import org.wickra.internal.WickraNative;
import java.lang.foreign.Arena;
import java.lang.foreign.MemorySegment;
import java.lang.ref.Cleaner;
import java.lang.ref.Reference;
import static java.lang.foreign.ValueLayout.*;

/** Streaming SpreadBollingerBands indicator over the Wickra C ABI. Not thread-safe; close when done. */
public final class SpreadBollingerBands implements AutoCloseable {
    private final MemorySegment handle;
    private final Cleaner.Cleanable cleanable;
    private boolean closed;

    public SpreadBollingerBands(int period, double numStd) {
        if (period < 0) {
            throw new IllegalArgumentException("period must be non-negative");
        }
        MemorySegment h;
        try {
            h = (MemorySegment) NativeMethods.WICKRA_SPREAD_BOLLINGER_BANDS_NEW.invokeExact((long) period, numStd);
        } catch (Throwable t) {
            throw WickraNative.rethrow(t);
        }
        if (h.address() == 0L) {
            throw new IllegalArgumentException("invalid SpreadBollingerBands parameters");
        }
        this.handle = h;
        this.cleanable = WickraNative.register(this, h, NativeMethods.WICKRA_SPREAD_BOLLINGER_BANDS_FREE);
    }

    /** Push one observation; returns the result, or null during warmup. */
    public SpreadBollingerBandsOutput update(double x, double y) {
        try (Arena a = Arena.ofConfined()) {
            MemorySegment out = a.allocate(32L);
            byte ok = (byte) NativeMethods.WICKRA_SPREAD_BOLLINGER_BANDS_UPDATE.invokeExact(handle(), x, y, out);
            if (ok == 0) {
                return null;
            }
            return new SpreadBollingerBandsOutput(
                out.get(JAVA_DOUBLE, 0L),
                out.get(JAVA_DOUBLE, 8L),
                out.get(JAVA_DOUBLE, 16L),
                out.get(JAVA_DOUBLE, 24L));
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
    public SpreadBollingerBandsOutput[] batch(double[] x, double[] y) {
        int n = x.length;
        if (y.length != n) {
            throw new IllegalArgumentException("all input arrays must have the same length");
        }
        try (Arena a = Arena.ofConfined()) {
            MemorySegment xSeg = a.allocateFrom(JAVA_DOUBLE, x);
            MemorySegment ySeg = a.allocateFrom(JAVA_DOUBLE, y);
            MemorySegment outSeg = a.allocate(32L * n);
            NativeMethods.WICKRA_SPREAD_BOLLINGER_BANDS_BATCH.invokeExact(handle(), xSeg, ySeg, outSeg, (long) n);
            SpreadBollingerBandsOutput[] out = new SpreadBollingerBandsOutput[n];
            for (int i = 0; i < n; i++) {
                out[i] = new SpreadBollingerBandsOutput(
                        outSeg.get(JAVA_DOUBLE, i * 32L + 0L),
                        outSeg.get(JAVA_DOUBLE, i * 32L + 8L),
                        outSeg.get(JAVA_DOUBLE, i * 32L + 16L),
                        outSeg.get(JAVA_DOUBLE, i * 32L + 24L));
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
            long n = (long) NativeMethods.WICKRA_SPREAD_BOLLINGER_BANDS_WARMUP_PERIOD.invokeExact(handle());
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
            byte r = (byte) NativeMethods.WICKRA_SPREAD_BOLLINGER_BANDS_IS_READY.invokeExact(handle());
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
            MemorySegment s = (MemorySegment) NativeMethods.WICKRA_SPREAD_BOLLINGER_BANDS_NAME.invokeExact(handle());
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
            NativeMethods.WICKRA_SPREAD_BOLLINGER_BANDS_RESET.invokeExact(handle());
        } catch (Throwable t) {
            throw WickraNative.rethrow(t);
        } finally {
            Reference.reachabilityFence(this);
        }
    }

    /** The native handle, refusing to hand out one that has been released. */
    private MemorySegment handle() {
        if (closed) {
            throw new IllegalStateException("SpreadBollingerBands has been closed");
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
