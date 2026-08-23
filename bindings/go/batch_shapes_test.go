package wickra

import (
	"math"
	"testing"
)

// The cross-section, order-book, bar-builder and profile families take an input
// shape the batch generator could not express, so 39 indicators had a C ABI
// batch that Go could not reach. These check the four shapes against feeding the
// same data one bar at a time.

func closeEnough(a, b float64) bool {
	if math.IsNaN(a) && math.IsNaN(b) {
		return true
	}
	return math.Abs(a-b) <= 1e-12*math.Max(1, math.Abs(b))
}

func TestCrossSectionBatchMatchesStreaming(t *testing.T) {
	const bars, members = 8, 3
	change := make([]float64, bars*members)
	volume := make([]float64, bars*members)
	newHigh := make([]bool, bars*members)
	newLow := make([]bool, bars*members)
	aboveMa := make([]bool, bars*members)
	onBuy := make([]bool, bars*members)
	stamps := make([]int64, bars)
	for bar := 0; bar < bars; bar++ {
		stamps[bar] = int64(bar)
		for m := 0; m < members; m++ {
			at := bar*members + m
			change[at] = float64(at)*0.37 - 2
			volume[at] = 100 + float64(at)*5
			newHigh[at] = (bar+m)%2 == 0
			newLow[at] = (bar+m)%3 == 0
			aboveMa[at] = m%2 == 0
			onBuy[at] = bar%2 == 0
		}
	}

	batched, err := NewAdvanceDecline()
	if err != nil {
		t.Fatal(err)
	}
	defer batched.Close()
	got := batched.Batch(change, volume, newHigh, newLow, aboveMa, onBuy, members, stamps)

	streamed, err := NewAdvanceDecline()
	if err != nil {
		t.Fatal(err)
	}
	defer streamed.Close()
	for bar := 0; bar < bars; bar++ {
		lo, hi := bar*members, (bar+1)*members
		want := streamed.Update(change[lo:hi], volume[lo:hi], newHigh[lo:hi], newLow[lo:hi],
			aboveMa[lo:hi], onBuy[lo:hi], stamps[bar])
		if !closeEnough(got[bar], want) {
			t.Fatalf("bar %d: batch %v, streaming %v", bar, got[bar], want)
		}
	}
}

func TestOrderBookBatchMatchesStreaming(t *testing.T) {
	const bars, depth = 6, 2
	bidPx := make([]float64, bars*depth)
	bidSz := make([]float64, bars*depth)
	askPx := make([]float64, bars*depth)
	askSz := make([]float64, bars*depth)
	for bar := 0; bar < bars; bar++ {
		for lvl := 0; lvl < depth; lvl++ {
			at := bar*depth + lvl
			drift, step := float64(bar)*0.25, float64(lvl)*0.1
			bidPx[at], bidSz[at] = 100+drift-step, 5+step
			askPx[at], askSz[at] = 100.2+drift+step, 4+step
		}
	}

	batched, err := NewMicroprice()
	if err != nil {
		t.Fatal(err)
	}
	defer batched.Close()
	got := batched.Batch(bidPx, bidSz, depth, askPx, askSz, depth)

	streamed, err := NewMicroprice()
	if err != nil {
		t.Fatal(err)
	}
	defer streamed.Close()
	for bar := 0; bar < bars; bar++ {
		lo, hi := bar*depth, (bar+1)*depth
		want := streamed.Update(bidPx[lo:hi], bidSz[lo:hi], askPx[lo:hi], askSz[lo:hi])
		if !closeEnough(got[bar], want) {
			t.Fatalf("bar %d: batch %v, streaming %v", bar, got[bar], want)
		}
	}
}

func TestBarBuilderBatchMatchesStreaming(t *testing.T) {
	const n = 12
	closes := make([]float64, n)
	vols := make([]float64, n)
	stamps := make([]int64, n)
	for i := range closes {
		closes[i] = 100 + float64(i)*3
		if i == 6 {
			closes[i] += 40 // a gap, so one candle completes several bricks
		}
		vols[i], stamps[i] = 1, int64(i)
	}

	batched, err := NewRenkoBars(1)
	if err != nil {
		t.Fatal(err)
	}
	defer batched.Close()
	got := batched.Batch(closes, closes, closes, closes, vols, stamps)

	streamed, err := NewRenkoBars(1)
	if err != nil {
		t.Fatal(err)
	}
	defer streamed.Close()
	var want []RenkoBrick
	for i := range closes {
		want = append(want, streamed.Update(closes[i], closes[i], closes[i], closes[i], vols[i], stamps[i])...)
	}

	if len(got) != len(want) {
		t.Fatalf("batch produced %d bricks, streaming %d", len(got), len(want))
	}
	for i := range want {
		if got[i] != want[i] {
			t.Fatalf("brick %d: batch %+v, streaming %+v", i, got[i], want[i])
		}
	}
}

func TestProfileBatchMatchesStreaming(t *testing.T) {
	const n = 10
	closes := make([]float64, n)
	vols := make([]float64, n)
	stamps := make([]int64, n)
	for i := range closes {
		closes[i], vols[i] = 100+float64(i), 10
		stamps[i] = int64(i) * 86_400_000 // one day apart
	}

	batched, err := NewDayOfWeekProfile(0)
	if err != nil {
		t.Fatal(err)
	}
	defer batched.Close()
	got := batched.Batch(closes, closes, closes, closes, vols, stamps)

	// The width comes from the indicator now. It used to be guessed from the
	// constructor parameter names, and this one takes only a UTC offset, so it
	// fell back to 4096 -- which a batch multiplies by the series length.
	if len(got) != n {
		t.Fatalf("expected one row per bar, got %d", len(got))
	}
	if len(got[0]) != 7 {
		t.Fatalf("a weekday profile is 7 wide, got %d", len(got[0]))
	}

	streamed, err := NewDayOfWeekProfile(0)
	if err != nil {
		t.Fatal(err)
	}
	defer streamed.Close()
	emitted := 0
	for i := range closes {
		want, ok := streamed.Update(closes[i], closes[i], closes[i], closes[i], vols[i], stamps[i])
		if !ok {
			for k, v := range got[i] {
				if !math.IsNaN(v) {
					t.Fatalf("row %d bucket %d: want NaN during warmup, got %v", i, k, v)
				}
			}
			continue
		}
		emitted++
		for k := range want {
			if !closeEnough(got[i][k], want[k]) {
				t.Fatalf("row %d bucket %d: batch %v, streaming %v", i, k, got[i][k], want[k])
			}
		}
	}
	if emitted == 0 {
		t.Fatal("the fixture must clear warmup")
	}
}
