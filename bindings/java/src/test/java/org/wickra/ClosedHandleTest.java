package org.wickra;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Every generated method used to read the {@code handle} field directly, so a
 * call made after {@link AutoCloseable#close()} had already run the cleaner
 * dereferenced freed memory. There was no {@code closed} flag anywhere in the
 * binding and nothing threw; the JVM died with an access violation, which is
 * precisely what the Panama FFM API is supposed to make impossible from safe
 * Java. The methods now go through a guarded accessor.
 */
class ClosedHandleTest {

    @Test
    void updateAfterCloseThrows() {
        Sma sma = new Sma(3);
        sma.update(1);
        sma.update(2);
        assertEquals(2.0, sma.update(3));
        sma.close();

        assertThrows(IllegalStateException.class, () -> sma.update(4));
    }

    @Test
    void readOnlyAccessorsAfterCloseThrow() {
        Sma sma = new Sma(3);
        sma.close();

        assertThrows(IllegalStateException.class, sma::isReady);
        assertThrows(IllegalStateException.class, sma::warmupPeriod);
        assertThrows(IllegalStateException.class, sma::name);
    }

    @Test
    void resetAfterCloseThrows() {
        Sma sma = new Sma(3);
        sma.close();

        assertThrows(IllegalStateException.class, sma::reset);
    }

    @Test
    void batchAfterCloseThrows() {
        Sma sma = new Sma(3);
        sma.close();

        assertThrows(IllegalStateException.class, () -> sma.batch(new double[] { 1, 2, 3, 4 }));
    }

    @Test
    void candleIndicatorAfterCloseThrows() {
        Atr atr = new Atr(3);
        atr.update(100, 101, 99, 100, 10, 0L);
        atr.close();

        assertThrows(IllegalStateException.class, () -> atr.update(101, 102, 100, 101, 10, 1L));
    }

    @Test
    void closeIsIdempotent() {
        Sma sma = new Sma(3);
        sma.close();
        sma.close();
    }

    @Test
    void anUnclosedIndicatorIsUnaffected() {
        try (Sma sma = new Sma(3)) {
            sma.update(1);
            sma.update(2);
            assertEquals(2.0, sma.update(3));
            assertTrue(sma.isReady());
            assertEquals(3, sma.warmupPeriod());
            assertEquals("SMA", sma.name());
        }
    }
}
