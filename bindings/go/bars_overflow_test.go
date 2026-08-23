package wickra

import "testing"

// One candle can complete far more bars than the fixed buffer `Update` uses. A
// Renko builder with a box size of 1 turns a 500-point move into 500 bricks; the
// wrapper read past its 64-element buffer and panicked with
// "index out of range [64] with length 64" until the surplus was drained.
func TestRenkoUpdateReturnsEveryBrickOfALargeMove(t *testing.T) {
	r, err := NewRenkoBars(1.0)
	if err != nil {
		t.Fatal(err)
	}
	defer r.Close()

	r.Update(100, 100, 100, 100, 1, 0)
	bricks := r.Update(600, 600, 600, 600, 1, 1)

	if len(bricks) <= 64 {
		t.Fatalf("the move must complete more bricks than the buffer holds, got %d", len(bricks))
	}
	// The bricks form one consecutive ladder, so nothing was dropped or
	// duplicated across the buffer boundary.
	for i, b := range bricks {
		if b.Close-b.Open != 1 {
			t.Fatalf("brick %d spans %v, want one box", i, b.Close-b.Open)
		}
		if i > 0 && b.Open != bricks[i-1].Close {
			t.Fatalf("brick %d starts at %v, previous closed at %v", i, b.Open, bricks[i-1].Close)
		}
	}
}
