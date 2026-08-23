// Generated from bindings/c/include/wickra.h. Do not edit by hand.
package org.wickra;

import org.wickra.internal.NativeMethods;
import org.wickra.internal.WickraNative;
import java.lang.foreign.Arena;
import java.lang.foreign.MemorySegment;
import java.lang.ref.Cleaner;
import java.lang.ref.Reference;
import static java.lang.foreign.ValueLayout.*;

/** Streaming TradeImbalance indicator over the Wickra C ABI. Not thread-safe; close when done. */
public final class TradeImbalance implements AutoCloseable {
    private final MemorySegment handle;
    private final Cleaner.Cleanable cleanable;
    private boolean closed;

    public TradeImbalance(int window) {
        if (window < 0) {
            throw new IllegalArgumentException("window must be non-negative");
        }
        MemorySegment h;
        try {
            h = (MemorySegment) NativeMethods.WICKRA_TRADE_IMBALANCE_NEW.invokeExact((long) window);
        } catch (Throwable t) {
            throw WickraNative.rethrow(t);
        }
        if (h.address() == 0L) {
            throw new IllegalArgumentException("invalid TradeImbalance parameters");
        }
        this.handle = h;
        this.cleanable = WickraNative.register(this, h, NativeMethods.WICKRA_TRADE_IMBALANCE_FREE);
    }

    /** Push one observation; returns the indicator value (NaN during warmup). */
    public double update(double price, double size, boolean isBuy, long timestamp) {
        try {
            return (double) NativeMethods.WICKRA_TRADE_IMBALANCE_UPDATE.invokeExact(handle(), price, size, (byte) (isBuy ? 1 : 0), timestamp);
        } catch (Throwable t) {
            throw WickraNative.rethrow(t);
        } finally {
            Reference.reachabilityFence(this);
        }
    }

    /** Vectorized update over a whole series; NaN at warmup positions. */
    public double[] batch(double[] price, double[] size, boolean[] isBuy, long[] timestamp) {
        int n = price.length;
        if (size.length != n) {
            throw new IllegalArgumentException("all input arrays must have the same length");
        }
        if (isBuy.length != n) {
            throw new IllegalArgumentException("all input arrays must have the same length");
        }
        if (timestamp.length != n) {
            throw new IllegalArgumentException("all input arrays must have the same length");
        }
        try (Arena a = Arena.ofConfined()) {
            MemorySegment priceSeg = a.allocateFrom(JAVA_DOUBLE, price);
            MemorySegment sizeSeg = a.allocateFrom(JAVA_DOUBLE, size);
            MemorySegment isBuySeg = WickraNative.boolSegment(a, isBuy);
            MemorySegment timestampSeg = a.allocateFrom(JAVA_LONG, timestamp);
            MemorySegment outSeg = a.allocate(JAVA_DOUBLE.byteSize() * n);
            NativeMethods.WICKRA_TRADE_IMBALANCE_BATCH.invokeExact(handle(), priceSeg, sizeSeg, isBuySeg, timestampSeg, outSeg, (long) n);
            double[] out = new double[n];
            MemorySegment.copy(outSeg, JAVA_DOUBLE, 0L, out, 0, n);
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
            long n = (long) NativeMethods.WICKRA_TRADE_IMBALANCE_WARMUP_PERIOD.invokeExact(handle());
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
            byte r = (byte) NativeMethods.WICKRA_TRADE_IMBALANCE_IS_READY.invokeExact(handle());
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
            MemorySegment s = (MemorySegment) NativeMethods.WICKRA_TRADE_IMBALANCE_NAME.invokeExact(handle());
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
            NativeMethods.WICKRA_TRADE_IMBALANCE_RESET.invokeExact(handle());
        } catch (Throwable t) {
            throw WickraNative.rethrow(t);
        } finally {
            Reference.reachabilityFence(this);
        }
    }

    /** The native handle, refusing to hand out one that has been released. */
    private MemorySegment handle() {
        if (closed) {
            throw new IllegalStateException("TradeImbalance has been closed");
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
