// Generated from bindings/c/include/wickra.h. Do not edit by hand.
package org.wickra;

/** One event from the live Binance feed. */
public record KlineEvent(String symbol, double open, double high, double low,
                         double close, double volume, long openTime, boolean isClosed) {}
