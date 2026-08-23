package wickra

import (
	"errors"
	"math"
	"strings"
	"testing"
)

// One indicator per FFI archetype, exercising the full New/Update/Batch/Reset/
// Close surface against the real native library.

func TestScalarKnownValue(t *testing.T) {
	s, err := NewSma(3)
	if err != nil {
		t.Fatalf("NewSma: %v", err)
	}
	defer s.Close()

	var last float64
	for _, v := range []float64{1, 2, 3, 4, 5} {
		last = s.Update(v)
	}
	if math.Abs(last-4.0) > 1e-9 {
		t.Fatalf("sma(3) last = %v, want 4.0", last)
	}
}

func TestWarmupPeriodAndIsReady(t *testing.T) {
	s, err := NewSma(3)
	if err != nil {
		t.Fatalf("NewSma: %v", err)
	}
	defer s.Close()

	if got := s.WarmupPeriod(); got != 3 {
		t.Fatalf("sma(3) WarmupPeriod = %d, want 3", got)
	}
	if s.IsReady() {
		t.Fatal("sma is ready before any update")
	}
	s.Update(1)
	s.Update(2)
	if s.IsReady() {
		t.Fatal("sma is ready mid-warmup")
	}
	s.Update(3)
	if !s.IsReady() {
		t.Fatal("sma is not ready after the warmup period")
	}
	s.Reset()
	if s.IsReady() {
		t.Fatal("sma is ready after reset")
	}
}

func TestScalarBatchMatchesStreaming(t *testing.T) {
	input := []float64{1, 2, 3, 4, 5, 6, 7, 8}

	stream, _ := NewSma(3)
	defer stream.Close()
	want := make([]float64, len(input))
	for i, v := range input {
		want[i] = stream.Update(v)
	}

	batchInd, _ := NewSma(3)
	defer batchInd.Close()
	got := batchInd.Batch(input)

	if len(got) != len(want) {
		t.Fatalf("batch len = %d, want %d", len(got), len(want))
	}
	for i := range want {
		if math.IsNaN(want[i]) && math.IsNaN(got[i]) {
			continue
		}
		if math.Abs(got[i]-want[i]) > 1e-9 {
			t.Fatalf("batch[%d] = %v, streaming = %v", i, got[i], want[i])
		}
	}
}

func TestMultiOutput(t *testing.T) {
	m, err := NewMacdIndicator(3, 6, 3)
	if err != nil {
		t.Fatalf("NewMacdIndicator: %v", err)
	}
	defer m.Close()

	var ok bool
	var out MacdOutput
	for i := 0; i < 30; i++ {
		out, ok = m.Update(100 + float64(i))
	}
	if !ok {
		t.Fatal("macd never produced a value after warmup")
	}
	if math.IsNaN(out.Macd) {
		t.Fatal("macd value is NaN after warmup")
	}
}

func TestBars(t *testing.T) {
	rb, err := NewRangeBars(2.0)
	if err != nil {
		t.Fatalf("NewRangeBars: %v", err)
	}
	defer rb.Close()

	total := 0
	for _, p := range []float64{100, 101, 103, 104, 99, 96, 102, 108, 95, 110} {
		bars := rb.Update(p, p, p, p, 1, 0)
		total += len(bars)
	}
	if total == 0 {
		t.Fatal("range bars produced no bars over a 15-point move")
	}
}

func TestProfile(t *testing.T) {
	vp, err := NewVolumeProfile(10, 24)
	if err != nil {
		t.Fatalf("NewVolumeProfile: %v", err)
	}
	defer vp.Close()

	var ok bool
	var snap VolumeProfileOutputScalars
	for i := 0; i < 50; i++ {
		price := 100 + 5*math.Sin(float64(i)*0.3)
		snap, ok = vp.Update(price, price+1, price-1, price, 1000, int64(i))
	}
	if !ok {
		t.Fatal("volume profile never produced a snapshot")
	}
	if len(snap.Values) == 0 {
		t.Fatal("volume profile returned an empty values buffer")
	}
}

func TestArrayInput(t *testing.T) {
	ob, err := NewOrderBookImbalanceFull()
	if err != nil {
		t.Fatalf("NewOrderBookImbalanceFull: %v", err)
	}
	defer ob.Close()

	bidPrice := []float64{99.9, 99.8, 99.7}
	bidSize := []float64{5, 3, 2}
	askPrice := []float64{100.1, 100.2, 100.3}
	askSize := []float64{1, 1, 1}
	v := ob.Update(bidPrice, bidSize, askPrice, askSize)
	if math.IsNaN(v) {
		t.Fatal("order-book imbalance is NaN on a populated book")
	}
}

func TestResetReturnsToWarmup(t *testing.T) {
	s, _ := NewSma(3)
	defer s.Close()
	for _, v := range []float64{1, 2, 3} {
		s.Update(v)
	}
	s.Reset()
	if got := s.Update(10); !math.IsNaN(got) {
		t.Fatalf("after reset first update = %v, want NaN (warmup)", got)
	}
}

func TestInvalidParams(t *testing.T) {
	if _, err := NewSma(0); err == nil {
		t.Fatal("NewSma(0) should return ErrInvalidParams")
	}
}

// A Go int reaching a uintptr_t parameter wraps: -1 becomes 2^64-1. That value
// happens to be refused downstream for exceeding the maximum window length, but
// only because those two magnitudes sit that way round -- nothing stated the
// relationship, and the caller got "invalid indicator parameters" rather than
// being told they passed a negative number.
func TestNegativePeriodIsRejectedByName(t *testing.T) {
	for _, period := range []int{-1, -1 << 20, math.MinInt} {
		ind, err := NewSma(period)
		if ind != nil {
			ind.Close()
			t.Fatalf("NewSma(%d) returned an indicator", period)
		}
		if !errors.Is(err, ErrInvalidParams) {
			t.Fatalf("NewSma(%d) error = %v, want ErrInvalidParams", period, err)
		}
		if !strings.Contains(err.Error(), "must not be negative") {
			t.Fatalf("NewSma(%d) error = %q, want it to name the problem", period, err)
		}
	}
}

// The guard applies to every unsigned parameter, not just the first.
func TestNegativeSecondParameterIsRejected(t *testing.T) {
	ind, err := NewMacdIndicator(12, -1, 9)
	if ind != nil {
		ind.Close()
		t.Fatal("NewMacdIndicator with a negative slow period returned an indicator")
	}
	if err == nil || !strings.Contains(err.Error(), "must not be negative") {
		t.Fatalf("error = %v, want it to name the negative parameter", err)
	}
}

// A valid period is unaffected.
func TestValidPeriodStillConstructs(t *testing.T) {
	ind, err := NewSma(3)
	if err != nil {
		t.Fatalf("NewSma(3) = %v", err)
	}
	defer ind.Close()
	ind.Update(1)
	ind.Update(2)
	if got := ind.Update(3); got != 2 {
		t.Fatalf("Update(3) = %v, want 2", got)
	}
}

func TestCloseIsIdempotent(t *testing.T) {
	s, _ := NewSma(3)
	s.Close()
	s.Close() // must not panic or double-free
}
