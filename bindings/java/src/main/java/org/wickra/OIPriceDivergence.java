// Generated from bindings/c/include/wickra.h. Do not edit by hand.
package org.wickra;

import org.wickra.internal.NativeMethods;
import org.wickra.internal.WickraNative;
import java.lang.foreign.Arena;
import java.lang.foreign.MemorySegment;
import java.lang.ref.Cleaner;
import static java.lang.foreign.ValueLayout.*;

/** Streaming OIPriceDivergence indicator over the Wickra C ABI. Not thread-safe; close when done. */
public final class OIPriceDivergence implements AutoCloseable {
    private final MemorySegment handle;
    private final Cleaner.Cleanable cleanable;

    public OIPriceDivergence(int window) {
        if (window < 0) {
            throw new IllegalArgumentException("window must be non-negative");
        }
        MemorySegment h;
        try {
            h = (MemorySegment) NativeMethods.WICKRA_OI_PRICE_DIVERGENCE_NEW.invokeExact((long) window);
        } catch (Throwable t) {
            throw WickraNative.rethrow(t);
        }
        if (h.address() == 0L) {
            throw new IllegalArgumentException("invalid OIPriceDivergence parameters");
        }
        this.handle = h;
        this.cleanable = WickraNative.register(this, h, NativeMethods.WICKRA_OI_PRICE_DIVERGENCE_FREE);
    }

    /** Push one observation; returns the indicator value (NaN during warmup). */
    public double update(double fundingRate, double markPrice, double indexPrice, double futuresPrice, double openInterest, double longSize, double shortSize, double takerBuyVolume, double takerSellVolume, double longLiquidation, double shortLiquidation, long timestamp) {
        try {
            return (double) NativeMethods.WICKRA_OI_PRICE_DIVERGENCE_UPDATE.invokeExact(handle, fundingRate, markPrice, indexPrice, futuresPrice, openInterest, longSize, shortSize, takerBuyVolume, takerSellVolume, longLiquidation, shortLiquidation, timestamp);
        } catch (Throwable t) {
            throw WickraNative.rethrow(t);
        }
    }

    /** Reset to the just-constructed state. */
    public void reset() {
        try {
            NativeMethods.WICKRA_OI_PRICE_DIVERGENCE_RESET.invokeExact(handle);
        } catch (Throwable t) {
            throw WickraNative.rethrow(t);
        }
    }

    @Override public void close() {
        cleanable.clean();
    }
}
