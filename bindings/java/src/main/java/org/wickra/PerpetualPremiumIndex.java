// Generated from bindings/c/include/wickra.h. Do not edit by hand.
package org.wickra;

import org.wickra.internal.NativeMethods;
import org.wickra.internal.WickraNative;
import java.lang.foreign.Arena;
import java.lang.foreign.MemorySegment;
import java.lang.ref.Cleaner;
import java.lang.ref.Reference;
import static java.lang.foreign.ValueLayout.*;

/** Streaming PerpetualPremiumIndex indicator over the Wickra C ABI. Not thread-safe; close when done. */
public final class PerpetualPremiumIndex implements AutoCloseable {
    private final MemorySegment handle;
    private final Cleaner.Cleanable cleanable;
    private boolean closed;

    public PerpetualPremiumIndex() {
        MemorySegment h;
        try {
            h = (MemorySegment) NativeMethods.WICKRA_PERPETUAL_PREMIUM_INDEX_NEW.invokeExact();
        } catch (Throwable t) {
            throw WickraNative.rethrow(t);
        }
        if (h.address() == 0L) {
            throw new IllegalArgumentException("invalid PerpetualPremiumIndex parameters");
        }
        this.handle = h;
        this.cleanable = WickraNative.register(this, h, NativeMethods.WICKRA_PERPETUAL_PREMIUM_INDEX_FREE);
    }

    /** Push one observation; returns the indicator value (NaN during warmup). */
    public double update(double fundingRate, double markPrice, double indexPrice, double futuresPrice, double openInterest, double longSize, double shortSize, double takerBuyVolume, double takerSellVolume, double longLiquidation, double shortLiquidation, long timestamp) {
        try {
            return (double) NativeMethods.WICKRA_PERPETUAL_PREMIUM_INDEX_UPDATE.invokeExact(handle(), fundingRate, markPrice, indexPrice, futuresPrice, openInterest, longSize, shortSize, takerBuyVolume, takerSellVolume, longLiquidation, shortLiquidation, timestamp);
        } catch (Throwable t) {
            throw WickraNative.rethrow(t);
        } finally {
            Reference.reachabilityFence(this);
        }
    }

    /** Vectorized update over a whole series; NaN at warmup positions. */
    public double[] batch(double[] fundingRate, double[] markPrice, double[] indexPrice, double[] futuresPrice, double[] openInterest, double[] longSize, double[] shortSize, double[] takerBuyVolume, double[] takerSellVolume, double[] longLiquidation, double[] shortLiquidation, long[] timestamp) {
        int n = fundingRate.length;
        if (markPrice.length != n) {
            throw new IllegalArgumentException("all input arrays must have the same length");
        }
        if (indexPrice.length != n) {
            throw new IllegalArgumentException("all input arrays must have the same length");
        }
        if (futuresPrice.length != n) {
            throw new IllegalArgumentException("all input arrays must have the same length");
        }
        if (openInterest.length != n) {
            throw new IllegalArgumentException("all input arrays must have the same length");
        }
        if (longSize.length != n) {
            throw new IllegalArgumentException("all input arrays must have the same length");
        }
        if (shortSize.length != n) {
            throw new IllegalArgumentException("all input arrays must have the same length");
        }
        if (takerBuyVolume.length != n) {
            throw new IllegalArgumentException("all input arrays must have the same length");
        }
        if (takerSellVolume.length != n) {
            throw new IllegalArgumentException("all input arrays must have the same length");
        }
        if (longLiquidation.length != n) {
            throw new IllegalArgumentException("all input arrays must have the same length");
        }
        if (shortLiquidation.length != n) {
            throw new IllegalArgumentException("all input arrays must have the same length");
        }
        if (timestamp.length != n) {
            throw new IllegalArgumentException("all input arrays must have the same length");
        }
        try (Arena a = Arena.ofConfined()) {
            MemorySegment fundingRateSeg = a.allocateFrom(JAVA_DOUBLE, fundingRate);
            MemorySegment markPriceSeg = a.allocateFrom(JAVA_DOUBLE, markPrice);
            MemorySegment indexPriceSeg = a.allocateFrom(JAVA_DOUBLE, indexPrice);
            MemorySegment futuresPriceSeg = a.allocateFrom(JAVA_DOUBLE, futuresPrice);
            MemorySegment openInterestSeg = a.allocateFrom(JAVA_DOUBLE, openInterest);
            MemorySegment longSizeSeg = a.allocateFrom(JAVA_DOUBLE, longSize);
            MemorySegment shortSizeSeg = a.allocateFrom(JAVA_DOUBLE, shortSize);
            MemorySegment takerBuyVolumeSeg = a.allocateFrom(JAVA_DOUBLE, takerBuyVolume);
            MemorySegment takerSellVolumeSeg = a.allocateFrom(JAVA_DOUBLE, takerSellVolume);
            MemorySegment longLiquidationSeg = a.allocateFrom(JAVA_DOUBLE, longLiquidation);
            MemorySegment shortLiquidationSeg = a.allocateFrom(JAVA_DOUBLE, shortLiquidation);
            MemorySegment timestampSeg = a.allocateFrom(JAVA_LONG, timestamp);
            MemorySegment outSeg = a.allocate(JAVA_DOUBLE.byteSize() * n);
            NativeMethods.WICKRA_PERPETUAL_PREMIUM_INDEX_BATCH.invokeExact(handle(), fundingRateSeg, markPriceSeg, indexPriceSeg, futuresPriceSeg, openInterestSeg, longSizeSeg, shortSizeSeg, takerBuyVolumeSeg, takerSellVolumeSeg, longLiquidationSeg, shortLiquidationSeg, timestampSeg, outSeg, (long) n);
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
            long n = (long) NativeMethods.WICKRA_PERPETUAL_PREMIUM_INDEX_WARMUP_PERIOD.invokeExact(handle());
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
            byte r = (byte) NativeMethods.WICKRA_PERPETUAL_PREMIUM_INDEX_IS_READY.invokeExact(handle());
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
            MemorySegment s = (MemorySegment) NativeMethods.WICKRA_PERPETUAL_PREMIUM_INDEX_NAME.invokeExact(handle());
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
            NativeMethods.WICKRA_PERPETUAL_PREMIUM_INDEX_RESET.invokeExact(handle());
        } catch (Throwable t) {
            throw WickraNative.rethrow(t);
        } finally {
            Reference.reachabilityFence(this);
        }
    }

    /** The native handle, refusing to hand out one that has been released. */
    private MemorySegment handle() {
        if (closed) {
            throw new IllegalStateException("PerpetualPremiumIndex has been closed");
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
