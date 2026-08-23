package org.wickra;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

import org.junit.jupiter.api.Test;

/**
 * One candle can complete far more bars than the fixed buffer {@code update}
 * passes to the native side. A Renko builder with a box size of 1 turns a
 * 500-point move into 500 bricks; the wrapper read past its 64-element segment
 * with the returned count until the surplus was drained off the handle.
 */
class BarBuilderOverflowTest {

    @Test
    void updateReturnsEveryBrickOfALargeMove() {
        try (RenkoBars renko = new RenkoBars(1.0)) {
            renko.update(100, 100, 100, 100, 1, 0);

            RenkoBrick[] bricks = renko.update(600, 600, 600, 600, 1, 1);

            assertTrue(bricks.length > 64,
                    "the move must complete more bricks than the buffer holds, got " + bricks.length);
            // One consecutive ladder, so nothing was dropped or duplicated
            // across the boundary between the buffer and the drained remainder.
            for (int i = 0; i < bricks.length; i++) {
                assertEquals(1.0, bricks[i].close() - bricks[i].open(), 1e-9);
                if (i > 0) {
                    assertEquals(bricks[i - 1].close(), bricks[i].open(), 1e-9);
                }
            }
        }
    }
}
